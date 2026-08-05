use crate::common::{
	cookie_header, init_test_mm, insert_case_version, seed_two_orgs_manager_cases,
	seed_two_orgs_users_cases, system_org_id, system_user_id, Result,
	TEST_CUSTOM_MANAGER_ROLE,
};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lib_auth::token::generate_web_token;
use lib_core::ctx::{Ctx, ROLE_SPONSOR_ADMIN_CRO};
use lib_core::model::drug::{
	DrugDeviceCharacteristicBmc, DrugDeviceCharacteristicFilter, FdaDeviceCodeBmc,
	FdaDeviceCodeFilter, FdaDeviceInformationBmc, FdaDeviceInformationFilter,
};
use lib_core::model::narrative::{
	NarrativeInformationBmc, NarrativeInformationForCreate,
};
use lib_core::model::patient::{PatientInformationBmc, PatientInformationForCreate};
use lib_core::model::store::set_full_context_dbx;
use lib_web::Error as WebError;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;

#[serial]
#[tokio::test]
async fn test_users_endpoints_require_admin_guard() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	let token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri("/api/users")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/users/{}", seed.user2.id))
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::FORBIDDEN);

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_rls_list_cases_filters_org() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	let token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri("/api/cases")
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);

	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let value: Value = serde_json::from_slice(&body)?;
	let cases = value
		.get("data")
		.and_then(|v| v.as_array())
		.ok_or("missing data array")?;

	let case_org1 = seed.case_org1.to_string();
	let case_org2 = seed.case_org2.to_string();
	assert!(cases
		.iter()
		.any(|c| c.get("id").and_then(|v| v.as_str()) == Some(&case_org1)));
	assert!(!cases
		.iter()
		.any(|c| c.get("id").and_then(|v| v.as_str()) == Some(&case_org2)));

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/cases/{case_org2}"))
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	let status = res.status();
	if status != StatusCode::BAD_REQUEST && status != StatusCode::NOT_FOUND {
		let err = res
			.extensions()
			.get::<std::sync::Arc<WebError>>()
			.map(|e| format!("{e:?}"));
		let body = to_bytes(res.into_body(), usize::MAX).await?;
		return Err(format!(
			"case get status {} body {} err {:?}",
			status,
			String::from_utf8_lossy(&body),
			err
		)
		.into());
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_rls_case_versions_filters_org() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_manager_cases(&mm).await?;

	let dbx = mm.dbx();
	dbx.begin_txn().await?;
	set_full_context_dbx(
		dbx,
		system_user_id(),
		system_org_id(),
		ROLE_SPONSOR_ADMIN_CRO,
	)
	.await?;
	insert_case_version(&mm, seed.case_org1, 1, seed.manager.id).await?;
	insert_case_version(&mm, seed.case_org2, 1, seed.user2.id).await?;
	dbx.commit_txn().await?;

	let token = generate_web_token(&seed.manager.email, seed.manager.token_salt)?;
	let cookie = cookie_header(&token.to_string());

	let app = web_server::app(mm);
	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/cases/{}/versions", seed.case_org1))
		.header("cookie", cookie.clone())
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::OK);

	let req = Request::builder()
		.method("GET")
		.uri(format!("/api/cases/{}/versions", seed.case_org2))
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.oneshot(req).await?;
	assert_eq!(res.status(), StatusCode::NOT_FOUND);

	Ok(())
}

#[serial]
#[tokio::test(flavor = "multi_thread")]
async fn device_rows_are_tenant_scoped_and_targeted_export_ignores_org_volume(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_manager_cases(&mm).await?;
	let background_case_id = uuid::Uuid::new_v4();
	let background_drug_id = uuid::Uuid::new_v4();
	let target_drug_id = uuid::Uuid::new_v4();
	let other_org_drug_id = uuid::Uuid::new_v4();
	let target_device_id =
		uuid::Uuid::parse_str("ffffffff-ffff-4fff-8fff-ffffffffffff")?;
	let other_org_device_id =
		uuid::Uuid::parse_str("eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee")?;

	let mut tx = mm.dbx().db().begin().await?;
	lib_core::model::store::set_user_context(&mut tx, system_user_id()).await?;
	lib_core::model::store::set_org_context(
		&mut tx,
		system_org_id(),
		lib_core::ctx::ROLE_SYSTEM_ADMIN,
	)
	.await?;
	sqlx::query(
		"INSERT INTO cases (id, organization_id, created_by, updated_by)
		 VALUES ($1, $2, $3, $3)",
	)
	.bind(background_case_id)
	.bind(seed.org1_id)
	.bind(system_user_id())
	.execute(&mut *tx)
	.await?;
	for (id, case_id, product) in [
		(background_drug_id, background_case_id, "Background product"),
		(target_drug_id, seed.case_org1, "Target product"),
		(other_org_drug_id, seed.case_org2, "Other org product"),
	] {
		sqlx::query(
			"INSERT INTO drug_information
			 (id, case_id, sequence_number, drug_characterization, medicinal_product, created_by)
			 VALUES ($1, $2, 1, '1', $3, $4)",
		)
		.bind(id)
		.bind(case_id)
		.bind(product)
		.bind(system_user_id())
		.execute(&mut *tx)
		.await?;
	}
	sqlx::query(
		"INSERT INTO fda_device_information
		 (id, drug_id, sequence_number, device_brand_name, common_device_name,
		  device_product_code, created_by)
		 SELECT ('00000000-0000-4000-8000-' || lpad(n::text, 12, '0'))::uuid,
		        $1, n, 'BACKGROUND-DEVICE', 'Background device', 'BKG', $2
		 FROM generate_series(1, 1000) AS n",
	)
	.bind(background_drug_id)
	.bind(system_user_id())
	.execute(&mut *tx)
	.await?;
	for (id, drug_id, brand) in [
		(target_device_id, target_drug_id, "TARGET-DEVICE-BRAND"),
		(other_org_device_id, other_org_drug_id, "OTHER-ORG-DEVICE"),
	] {
		sqlx::query(
			"INSERT INTO fda_device_information
			 (id, drug_id, sequence_number, device_brand_name, common_device_name,
			  device_product_code, created_by)
			 VALUES ($1, $2, 1, $3, 'Regression device', 'REG', $4)",
		)
		.bind(id)
		.bind(drug_id)
		.bind(brand)
		.bind(system_user_id())
		.execute(&mut *tx)
		.await?;
	}
	for (id, device_id, value_code) in [
		(uuid::Uuid::new_v4(), target_device_id, "7654321"),
		(uuid::Uuid::new_v4(), other_org_device_id, "1234567"),
	] {
		sqlx::query(
			"INSERT INTO fda_device_codes
			 (id, device_id, element, sequence_number, value_code, created_by)
			 VALUES ($1, $2, 'device_problem', 1, $3, $4)",
		)
		.bind(id)
		.bind(device_id)
		.bind(value_code)
		.bind(system_user_id())
		.execute(&mut *tx)
		.await?;
	}
	for (drug_id, code) in [
		(target_drug_id, "TARGET.CHAR"),
		(other_org_drug_id, "OTHER.ORG.CHAR"),
	] {
		sqlx::query(
			"INSERT INTO drug_device_characteristics
			 (drug_id, sequence_number, code, value_type, value_value, created_by)
			 VALUES ($1, 1, $2, 'ST', 'device characteristic', $3)",
		)
		.bind(drug_id)
		.bind(code)
		.bind(system_user_id())
		.execute(&mut *tx)
		.await?;
	}
	tx.commit().await?;

	let ctx = Ctx::new(
		seed.manager.id,
		seed.org1_id,
		TEST_CUSTOM_MANAGER_ROLE.to_string(),
	)?;
	PatientInformationBmc::create(
		&ctx,
		&mm,
		PatientInformationForCreate {
			case_id: seed.case_org1,
			patient_initials: Some("RLS".to_string()),
			patient_initials_null_flavor: None,
			birth_date: None,
			birth_date_null_flavor: None,
			age_at_time_of_onset: None,
			age_unit: None,
			gestation_period: None,
			gestation_period_unit: None,
			age_group: None,
			weight_kg: None,
			height_cm: None,
			sex: None,
			sex_null_flavor: Some("UNK".to_string()),
			race_codes: Vec::new(),
			race_code_null_flavor: None,
			ethnicity_code: None,
			ethnicity_code_null_flavor: None,
			last_menstrual_period_date: None,
			last_menstrual_period_date_null_flavor: None,
			medical_history_text: None,
			medical_history_text_null_flavor: Some("UNK".to_string()),
			concomitant_therapy: None,
		},
	)
	.await?;
	NarrativeInformationBmc::create(
		&ctx,
		&mm,
		NarrativeInformationForCreate {
			case_id: seed.case_org1,
			source_narrative_presave_id: None,
			case_narrative: "Device export RLS regression".to_string(),
			reporter_comments: None,
			sender_comments: None,
			additional_information: None,
		},
	)
	.await?;
	let uuid_filter = |id: uuid::Uuid| {
		Some(OpValsValue::from(vec![OpValValue::Eq(json!(
			id.to_string()
		))]))
	};
	assert!(DrugDeviceCharacteristicBmc::list(
		&ctx,
		&mm,
		Some(vec![DrugDeviceCharacteristicFilter {
			drug_id: uuid_filter(other_org_drug_id),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?
	.is_empty());
	assert!(FdaDeviceInformationBmc::list(
		&ctx,
		&mm,
		Some(vec![FdaDeviceInformationFilter {
			drug_id: uuid_filter(other_org_drug_id),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?
	.is_empty());
	assert!(FdaDeviceCodeBmc::list(
		&ctx,
		&mm,
		Some(vec![FdaDeviceCodeFilter {
			device_id: uuid_filter(other_org_device_id),
			..Default::default()
		}]),
		Some(ListOptions::default()),
	)
	.await?
	.is_empty());

	let exported = tokio::task::block_in_place(|| {
		tokio::runtime::Handle::current().block_on(
			xml::export::serialize_case_xml_for_authority(
				&ctx,
				&mm,
				seed.case_org1,
				lib_core::regulatory::RegulatoryAuthority::Fda,
			),
		)
	})?;
	assert!(exported.contains("TARGET-DEVICE-BRAND"));
	assert!(exported.contains("code=\"7654321\""));
	assert!(!exported.contains("BACKGROUND-DEVICE"));
	assert!(!exported.contains("OTHER-ORG-DEVICE"));

	Ok(())
}
