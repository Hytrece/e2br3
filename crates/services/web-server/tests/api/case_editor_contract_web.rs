use crate::common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use lib_auth::token::generate_web_token;
use lib_core::ctx::ROLE_SPONSOR_ADMIN_CRO;
use lib_core::model::store::set_full_context_dbx;
use lib_core::model::ModelManager;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;
use uuid::Uuid;
use validator::portable_constraints;

fn portable_constraint_message(code: &str) -> String {
	portable_constraints()
		.into_iter()
		.find(|constraint| constraint.code == code)
		.expect("portable Catalog constraint exists")
		.message
}

async fn post_json(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
	body: Value,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("POST")
		.uri(uri)
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	Ok((status, serde_json::from_slice::<Value>(&body)?))
}

async fn patch_json(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
	body: Value,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("PATCH")
		.uri(uri)
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let body = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
	Ok((status, body))
}

async fn put_json(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
	body: Value,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("PUT")
		.uri(uri)
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(body.to_string()))?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let body = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
	Ok((status, body))
}

async fn delete_json(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("DELETE")
		.uri(uri)
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let body = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
	Ok((status, body))
}

async fn get_json(
	app: &axum::Router,
	cookie: &str,
	uri: &str,
) -> Result<(StatusCode, Value)> {
	let req = Request::builder()
		.method("GET")
		.uri(uri)
		.header("cookie", cookie)
		.body(Body::empty())?;
	let res = app.clone().oneshot(req).await?;
	let status = res.status();
	let body = to_bytes(res.into_body(), usize::MAX).await?;
	let body = serde_json::from_slice::<Value>(&body).unwrap_or(Value::Null);
	Ok((status, body))
}

async fn stale_validation_summary_count(
	mm: &ModelManager,
	user_id: Uuid,
	org_id: Uuid,
	case_id: &str,
) -> Result<i64> {
	let case_uuid = Uuid::parse_str(case_id)?;
	mm.dbx().begin_txn().await?;
	set_full_context_dbx(mm.dbx(), user_id, org_id, ROLE_SPONSOR_ADMIN_CRO).await?;
	let count = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (i64,)>(
				"SELECT COUNT(*)::bigint
				   FROM case_validation_summaries
				  WHERE case_id = $1
				    AND stale = true",
			)
			.bind(case_uuid),
		)
		.await?
		.0;
	mm.dbx().commit_txn().await?;
	Ok(count)
}

async fn validation_summary_row_versions(
	mm: &ModelManager,
	user_id: Uuid,
	org_id: Uuid,
	case_id: &str,
) -> Result<Value> {
	let case_uuid = Uuid::parse_str(case_id)?;
	mm.dbx().begin_txn().await?;
	set_full_context_dbx(mm.dbx(), user_id, org_id, ROLE_SPONSOR_ADMIN_CRO).await?;
	let snapshot = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (Value,)>(
				"SELECT COALESCE(
				            jsonb_agg(
				                jsonb_build_object(
				                    'appendix', appendix,
				                    'pageId', page_id,
				                    'rowVersion', xmin::text
				                )
				                ORDER BY appendix, page_id
				            ),
				            '[]'::jsonb
				        )
				   FROM case_validation_summaries
				  WHERE case_id = $1",
			)
			.bind(case_uuid),
		)
		.await?
		.0;
	mm.dbx().commit_txn().await?;
	Ok(snapshot)
}

#[tokio::test]
#[serial]
async fn c11_identity_is_not_stored_on_cases_table() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());

	let case_id = create_case(&app, &cookie, "C11-SCHEMA").await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	mm.dbx().begin_txn().await?;
	set_full_context_dbx(
		mm.dbx(),
		seed.admin.id,
		seed.org_id,
		ROLE_SPONSOR_ADMIN_CRO,
	)
	.await?;
	let cases_column_count: (i64,) = mm
		.dbx()
		.fetch_one(sqlx::query_as(
			"SELECT COUNT(*)::bigint
				   FROM information_schema.columns
				  WHERE table_name = 'cases'
				    AND column_name = 'safety_report_id'",
		))
		.await?;
	let section_row: (Option<String>, i32) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as(
				"SELECT safety_report_id, version
				   FROM safety_report_identification
				  WHERE case_id = $1",
			)
			.bind(Uuid::parse_str(&case_id)?),
		)
		.await?;
	mm.dbx().commit_txn().await?;

	assert_eq!(cases_column_count.0, 0);
	assert!(
		section_row
			.0
			.as_deref()
			.is_some_and(|value| value.starts_with("C11-SCHEMA-")),
		"{section_row:?}"
	);
	assert_eq!(section_row.1, 1);
	Ok(())
}

async fn create_case(
	app: &axum::Router,
	cookie: &str,
	safety_report_prefix: &str,
) -> Result<String> {
	let safety_report_id = format!("{safety_report_prefix}-{}", Uuid::new_v4());
	let (status, body) = post_json(
		app,
		cookie,
		"/api/cases",
		json!({
			"data": {
				"safetyReportIdentification": {"safetyReportId": safety_report_id},
				"status": "draft"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing created case id")?
		.to_string())
}

async fn create_case_for_editor(
	app: &axum::Router,
	cookie: &str,
	safety_report_prefix: &str,
	_appendices: &[&str],
) -> Result<String> {
	let safety_report_id = format!("{safety_report_prefix}-{}", Uuid::new_v4());
	let (status, body) = post_json(
		app,
		cookie,
		"/api/cases",
		json!({
			"data": {
				"safetyReportIdentification": {"safetyReportId": safety_report_id},
				"status": "draft"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing created case id")?
		.to_string())
}

async fn create_reaction_fixture(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
) -> Result<String> {
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/reactions"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"primary_source_reaction": "Headache",
				"primary_source_reaction_translation": "Head pain",
				"reaction_meddra_version": "27.1",
				"reaction_meddra_code": "10019211",
				"serious": true,
				"outcome": "1"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing reaction id")?
		.to_string())
}

async fn create_test_result_fixture(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
) -> Result<String> {
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/test-results"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"test_name": "ALT",
				"test_result_value": "42",
				"test_result_unit": "U/L"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing test result id")?
		.to_string())
}

async fn create_drug_fixture(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
) -> Result<String> {
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/drugs"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"drug_characterization": "1",
				"medicinal_product": "Example Product",
				"action_taken": "1"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing drug id")?
		.to_string())
}

async fn create_patient_fixture(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
) -> Result<String> {
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "ABC"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing patient id")?
		.to_string())
}

async fn create_past_drug_history_fixture(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
) -> Result<String> {
	let patient_id = create_patient_fixture(app, cookie, case_id).await?;
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/patient/past-drugs"),
		json!({
			"data": {
				"patient_id": patient_id,
				"sequence_number": 1,
				"drug_name": "Prior Drug",
				"indication_meddra_code": "10012345"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	Ok(body["data"]["id"]
		.as_str()
		.ok_or("missing past drug id")?
		.to_string())
}

async fn create_safety_report(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
	report_type: &str,
	fulfil_expedited_criteria: bool,
) -> Result<()> {
	create_safety_report_with_local_criteria(
		app,
		cookie,
		case_id,
		report_type,
		fulfil_expedited_criteria,
		None,
	)
	.await
}

async fn create_safety_report_with_local_criteria(
	app: &axum::Router,
	cookie: &str,
	case_id: &str,
	report_type: &str,
	fulfil_expedited_criteria: bool,
	local_criteria_report_type: Option<&str>,
) -> Result<()> {
	let (status, body) = post_json(
		app,
		cookie,
		&format!("/api/cases/{case_id}/safety-report"),
		json!({
			"data": {
				"case_id": case_id,
				"transmission_date": [2024, 1],
				"report_type": report_type,
				"date_first_received_from_source": [2024, 1],
				"date_of_most_recent_information": [2024, 1],
				"fulfil_expedited_criteria": fulfil_expedited_criteria,
				"local_criteria_report_type": local_criteria_report_type
			}
		}),
	)
	.await?;
	assert!(
		status == StatusCode::CREATED || status == StatusCode::OK,
		"{body}"
	);
	Ok(())
}

fn assert_no_ae_lb_dg_payload(data: &Value) {
	assert!(data.get("reactions").is_none(), "{data}");
	assert!(data.get("testResults").is_none(), "{data}");
	assert!(data.get("drugs").is_none(), "{data}");
}

#[serial]
#[tokio::test]
async fn editor_shell_returns_only_case_header_workflow_and_permissions(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let safety_report_id = format!("EDITOR-SHELL-{}", Uuid::new_v4());

	let (status, body) = post_json(
		&app,
		&cookie,
		"/api/cases",
		json!({
			"data": {
				"safetyReportIdentification": {"safetyReportId": safety_report_id},
				"status": "draft",
				"dgPrdKey": "DG-EDITOR-SHELL"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED);
	let case_id = body["data"]["id"]
		.as_str()
		.ok_or("missing created case id")?
		.to_string();

	let (status, body) =
		get_json(&app, &cookie, &format!("/api/cases/{case_id}/editor/shell"))
			.await?;

	assert_eq!(status, StatusCode::OK);
	assert_eq!(body["id"], case_id);
	assert_eq!(
		body["safetyReportIdentification"]["safetyReportId"],
		safety_report_id
	);
	assert!(body.get("status").is_some());
	assert!(body.get("appendices").is_none());
	assert!(body.get("canActOnWorkflow").is_some());
	assert!(body.get("reactions").is_none());
	assert!(body.get("testResults").is_none());
	assert!(body.get("drugs").is_none());
	assert!(body.get("patientInformation").is_none());
	assert!(body.get("messageHeader").is_none());
	assert_eq!(
		body["safetyReportIdentification"]
			.as_object()
			.map(serde_json::Map::len),
		Some(1)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_returns_ci_payload_only() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI").await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	let (status, body) =
		get_json(&app, &cookie, &format!("/api/cases/{case_id}/editor/CI")).await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let data = body.get("data").ok_or("missing data")?;
	let safety_report = data
		.get("safetyReportIdentification")
		.ok_or("missing safetyReportIdentification")?;
	assert!(safety_report["safetyReportId"].is_string(), "{body}");
	assert!(data["case"].get("safety_report_id").is_none(), "{body}");
	assert!(data.get("receiverInfo").is_none(), "{body}");
	assert!(data.get("receiverInformation").is_none(), "{body}");
	assert!(data.get("receiver").is_none(), "{body}");
	assert!(data["otherCaseIdentifiers"].is_array(), "{body}");
	assert!(data["linkedReports"].is_array(), "{body}");
	assert!(data["documentsHeldBySender"].is_array(), "{body}");
	assert!(
		safety_report.get("otherCaseIdentifiers").is_none(),
		"{body}"
	);
	assert!(safety_report.get("linkedReports").is_none(), "{body}");
	assert!(
		safety_report.get("documentsHeldBySender").is_none(),
		"{body}"
	);
	assert!(data.get("messageHeader").is_none(), "{body}");
	assert_no_ae_lb_dg_payload(data);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_returns_direct_page_rows_without_field_issues(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-CI-PROJECTION",
		&["ich", "fda"],
	)
	.await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=fda"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["pageId"], "CI");
	assert_eq!(body["authorities"], json!(["fda"]));
	assert!(body.get("appendices").is_none(), "{body}");
	assert!(body["saved"].as_bool().is_some(), "{body}");
	assert!(body["requiredCount"].as_u64().is_some(), "{body}");
	assert!(
		body["rows"]["safetyReportIdentification"].is_object(),
		"{body}"
	);
	assert!(body["rows"].get("messageHeader").is_none(), "{body}");
	assert!(body["rows"].get("receiverInfo").is_none(), "{body}");
	assert!(body["rows"]["otherCaseIdentifiers"].is_array(), "{body}");
	assert!(body["rows"]["linkedReports"].is_array(), "{body}");
	assert!(body["rows"]["documentsHeldBySender"].is_array(), "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["reportType"],
		"2"
	);
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);
	assert_eq!(body["requiredCount"], 0, "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_projection_matches_field_contract() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-CONTRACT", &["ich"])
			.await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"].as_object().ok_or("missing CI rows")?;
	let mut owners = rows.keys().map(String::as_str).collect::<Vec<_>>();
	owners.sort_unstable();
	assert_eq!(
		owners,
		vec![
			"case",
			"documentsHeldBySender",
			"linkedReports",
			"otherCaseIdentifiers",
			"safetyReportIdentification",
			"sourceDocuments",
		]
	);
	let case = rows["case"].as_object().ok_or("missing CI case owner")?;
	let mut case_fields = case.keys().map(String::as_str).collect::<Vec<_>>();
	case_fields.sort_unstable();
	assert_eq!(
		case_fields,
		vec!["fdaReportType", "mfdsReportType", "reportYear"]
	);
	assert!(rows["sourceDocuments"].is_array(), "{body}");
	assert!(rows.get("receiverInfo").is_none(), "{body}");
	assert!(rows.get("receiverInformation").is_none(), "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_complete_fields_round_trip() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-ROUNDTRIP", &["ich"])
			.await?;
	let roundtrip_safety_report_id = format!("CI-ROUNDTRIP-{case_id}");
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {
				"safetyReportIdentification": {
					"safetyReportId": roundtrip_safety_report_id.clone(),
					"transmissionDate": "20260722120000+0900",
					"reportType": "2",
					"dateFirstReceivedFromSource": "20260721",
					"dateOfMostRecentInformation": "20260722",
					"additionalDocumentsAvailable": true,
					"fulfilExpeditedCriteria": true,
					"localCriteriaReportType": "1",
					"combinationProductReportIndicator": "1",
					"worldwideUniqueId": "US-QVIS-2026-0001",
					"firstSenderType": "1",
					"otherCaseIdentifiersExist": true,
					"nullificationAmendmentCode": "2",
					"nullificationReason": "CI roundtrip reason"
				},
				"case": {
					"reportYear": "2026",
					"fdaReportType": "4",
					"mfdsReportType": "6"
				},
				"documentsHeldBySender": [{
					"documentDescription": "CI document",
					"includedDocument": "Q0ktZG9jdW1lbnQ="
				}],
				"otherCaseIdentifiers": [{
					"source": "CI source",
					"caseIdentifier": "CI-OTHER-001"
				}],
				"linkedReports": [{
					"linkedReportNumber": "CI-LINK-001"
				}],
				"sourceDocuments": [{
					"sourceDocumentName": "ci-source.txt"
				}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = &body["rows"];
	assert_eq!(rows["case"]["reportYear"], "2026", "{body}");
	assert_eq!(rows["case"]["fdaReportType"], "4", "{body}");
	assert_eq!(rows["case"]["mfdsReportType"], "6", "{body}");
	let report = &rows["safetyReportIdentification"];
	for (field, expected) in [
		("safetyReportId", json!(roundtrip_safety_report_id)),
		("transmissionDate", json!("20260722120000+0900")),
		("reportType", json!("2")),
		("dateFirstReceivedFromSource", json!("20260721")),
		("dateOfMostRecentInformation", json!("20260722")),
		("additionalDocumentsAvailable", json!(true)),
		("fulfilExpeditedCriteria", json!(true)),
		("localCriteriaReportType", json!("1")),
		("combinationProductReportIndicator", json!("1")),
		("worldwideUniqueId", json!("US-QVIS-2026-0001")),
		("firstSenderType", json!("1")),
		("otherCaseIdentifiersExist", json!(true)),
		("nullificationAmendmentCode", json!("2")),
		("nullificationReason", json!("CI roundtrip reason")),
	] {
		assert_eq!(report[field], expected, "{field}: {body}");
	}
	assert_eq!(
		rows["documentsHeldBySender"][0]["documentDescription"], "CI document",
		"{body}"
	);
	assert_eq!(
		rows["documentsHeldBySender"][0]["includedDocument"], "Q0ktZG9jdW1lbnQ=",
		"{body}"
	);
	assert_eq!(
		rows["otherCaseIdentifiers"][0]["source"], "CI source",
		"{body}"
	);
	assert_eq!(
		rows["otherCaseIdentifiers"][0]["caseIdentifier"], "CI-OTHER-001",
		"{body}"
	);
	assert_eq!(
		rows["linkedReports"][0]["linkedReportNumber"], "CI-LINK-001",
		"{body}"
	);
	assert_eq!(
		rows["sourceDocuments"][0]["sourceDocumentName"], "ci-source.txt",
		"{body}"
	);

	for (null_flavor_field, value_field, restore_value) in [
		(
			"fulfilExpeditedCriteriaNullFlavor",
			"fulfilExpeditedCriteria",
			json!(true),
		),
		(
			"combinationProductReportIndicatorNullFlavor",
			"combinationProductReportIndicator",
			json!("1"),
		),
		(
			"otherCaseIdentifiersExistNullFlavor",
			"otherCaseIdentifiersExist",
			json!(true),
		),
	] {
		let (status, null_flavor_body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			json!({
				"authorities": ["ich", "fda", "mfds"],
				"rows": { "safetyReportIdentification": { null_flavor_field: "NI" } }
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{null_flavor_body}");
		assert_eq!(
			null_flavor_body["rows"]["safetyReportIdentification"]
				[null_flavor_field],
			"NI",
			"{null_flavor_body}"
		);

		let (status, restored_body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			json!({
				"authorities": ["ich", "fda", "mfds"],
				"rows": { "safetyReportIdentification": { value_field: restore_value.clone() } }
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{restored_body}");
		assert_eq!(
			restored_body["rows"]["safetyReportIdentification"][value_field],
			restore_value,
			"{restored_body}"
		);
	}

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=ich,fda,mfds"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], body["rows"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_repeating_rows_create_update_delete_and_restore() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-ROWS").await?;

	for (owner, create_row, update_field, updated_value) in [
		(
			"documentsHeldBySender",
			json!({ "documentDescription": "created document" }),
			"documentDescription",
			json!("updated document"),
		),
		(
			"otherCaseIdentifiers",
			json!({ "source": "created source", "caseIdentifier": "CREATED-ID" }),
			"source",
			json!("updated source"),
		),
		(
			"linkedReports",
			json!({ "linkedReportNumber": "CREATED-LINK" }),
			"linkedReportNumber",
			json!("UPDATED-LINK"),
		),
	] {
		let mut create_request = json!({ "authorities": ["ich"], "rows": {} });
		create_request["rows"][owner] = json!([create_row]);
		let (status, created) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			create_request,
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{owner}: {created}");
		let id = created["rows"][owner][0]["id"]
			.as_str()
			.ok_or("created CI row is missing id")?
			.to_string();

		let mut update_row = json!({ "id": id });
		update_row[update_field] = updated_value.clone();
		let mut update_request = json!({ "authorities": ["ich"], "rows": {} });
		update_request["rows"][owner] = json!([update_row]);
		let (status, updated) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			update_request,
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{owner}: {updated}");
		assert_eq!(updated["rows"][owner][0][update_field], updated_value);

		let mut delete_request = json!({ "authorities": ["ich"], "rows": {} });
		delete_request["rows"][owner] = json!([{ "id": id, "deleted": true }]);
		let (status, deleted) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			delete_request,
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{owner}: {deleted}");
		assert!(
			deleted["rows"][owner].as_array().is_some_and(Vec::is_empty),
			"{owner}: {deleted}"
		);

		let mut restore_row = json!({ "id": id, "deleted": false });
		restore_row[update_field] = updated_value.clone();
		let mut restore_request = json!({ "authorities": ["ich"], "rows": {} });
		restore_request["rows"][owner] = json!([restore_row]);
		let (status, restored) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			restore_request,
		)
		.await?;
		assert_eq!(status, StatusCode::OK, "{owner}: {restored}");
		assert_eq!(restored["rows"][owner][0]["id"], id);
		assert_eq!(restored["rows"][owner][0][update_field], updated_value);
	}

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich"],
			"rows": { "sourceDocuments": [{ "sourceDocumentName": "created.txt" }] }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	let source_id = created["rows"]["sourceDocuments"][0]["id"]
		.as_str()
		.ok_or("created source document is missing id")?;
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich"],
			"rows": { "sourceDocuments": [{ "id": source_id, "sourceDocumentName": "updated.txt" }] }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["sourceDocuments"][0]["sourceDocumentName"],
		"updated.txt"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_repeating_constraint_rejects_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-ROW-GATE").await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"documentsHeldBySender": [{
					"documentDescription": "D".repeat(2001)
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.C.1.6.1.r.1.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"documentsHeldBySender.0.documentDescription"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["documentsHeldBySender"]
		.as_array()
		.is_some_and(Vec::is_empty));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_incomplete_registry_fields_enforce_portable_constraints(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-CI-FIELD-GATES",
		&["ich", "fda"],
	)
	.await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	for (field, invalid_value, rule_code, path) in [
		(
			"safetyReportId",
			json!("S".repeat(101)),
			"ICH.C.1.1.LENGTH.MAX",
			"safetyReportIdentification.safetyReportId",
		),
		(
			"transmissionDate",
			json!("not-an-e2b-date"),
			"ICH.C.1.2.ALLOWED.VALUE",
			"safetyReportIdentification.transmissionDate",
		),
		(
			"reportType",
			json!("9"),
			"ICH.C.1.3.ALLOWED.VALUE",
			"safetyReportIdentification.reportType",
		),
		(
			"fulfilExpeditedCriteria",
			json!("not-a-boolean"),
			"ICH.C.1.7.ALLOWED.VALUE",
			"safetyReportIdentification.fulfilExpeditedCriteria",
		),
		(
			"localCriteriaReportType",
			json!("12"),
			"FDA.C.1.7.1.LENGTH.MAX",
			"safetyReportIdentification.localCriteriaReportType",
		),
		(
			"nullificationAmendmentCode",
			json!("9"),
			"ICH.C.1.11.1.ALLOWED.VALUE",
			"safetyReportIdentification.nullificationAmendmentCode",
		),
	] {
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/CI"),
			json!({
				"authorities": ["ich", "fda"],
				"rows": { "safetyReportIdentification": { field: invalid_value } }
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}: {body}");
		assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
		assert_eq!(
			body["error"]["data"]["detail"]["ruleCode"], rule_code,
			"{field}: {body}"
		);
		assert_eq!(
			body["error"]["data"]["detail"]["path"], path,
			"{field}: {body}"
		);
	}

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich", "fda"],
			"rows": {
				"linkedReports": [{
					"linkedReportNumber": "L".repeat(101)
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.C.1.10.r.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"linkedReports.0.linkedReportNumber"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=ich,fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["linkedReports"]
		.as_array()
		.is_some_and(Vec::is_empty));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_accepts_multiple_profiles() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-MULTI-PROFILE").await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=fda,mfds"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["pageId"], "CI");
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_accepts_multiple_authorities() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-MULTI-AUTHORITY").await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=fda,mfds"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_keeps_profile_context_without_field_visibility(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-USKR-FIELDS").await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=mfds,fda"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["authorities"], json!(["mfds", "fda"]));
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_returns_profiles_context() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-PROFILES-ONLY").await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["authorities"], json!(["ich"]));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_patch_updates_only_report_type_and_returns_projection(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-PATCH", &["ich"]).await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"rows": { "safetyReportIdentification": { "reportType": "3" } }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["reportType"],
		"3"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["fulfilExpeditedCriteria"],
		false
	);
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/safety-report"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["report_type"], "3");
	assert_eq!(body["data"]["fulfil_expedited_criteria"], false);

	Ok(())
}

#[serial]
#[tokio::test]
async fn ci_patch_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-GATE", &["ich"]).await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;
	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"rows": { "safetyReportIdentification": { "otherCaseIdentifiersExist": true } }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let stale_before =
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(stale_before, 0);
	let cache_versions_before =
		validation_summary_row_versions(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"rows": { "safetyReportIdentification": { "otherCaseIdentifiersExist": false } }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.C.1.9.1.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"safetyReportIdentification.otherCaseIdentifiersExist"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["message"],
		portable_constraint_message("ICH.C.1.9.1.ALLOWED.VALUE")
	);
	assert!(body["error"]["data"]["req_uuid"].is_string());
	let stale_after =
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(stale_after, stale_before);
	let cache_versions_after =
		validation_summary_row_versions(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(cache_versions_after, cache_versions_before);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/safety-report"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["other_case_identifiers_exist"], true, "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_editor_direct_content_save_invalidates_summary_until_live_validation(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-REFRESH", &["ich"]).await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;
	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["ich"],
			"rows": { "safetyReportIdentification": { "reportType": "3" } }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	assert!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await? > 0,
		"save must invalidate summary without running full validation"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["authority"], "ich", "{body}");
	assert_eq!(body["data"]["case_id"], case_id, "{body}");
	assert_eq!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?,
		0,
		"live validation must refresh summary rows"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_patch_accepts_profiles() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-PATCH-PROFILES").await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["fda", "mfds"],
			"rows": {}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));
	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_patch_accepts_authorities() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-PATCH-AUTHORITIES").await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["fda", "mfds"],
			"rows": {}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));
	assert_eq!(body["authorities"], json!(["fda", "mfds"]));
	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_patch_rejects_invalid_profiles_before_mutation() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-CI-BAD-PROFILES").await?;
	create_safety_report(&app, &cookie, &case_id, "1", false).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["unknown"],
			"rows": { "safetyReportIdentification": { "reportType": "3" } }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_ne!(
		body["rows"]["safetyReportIdentification"]["reportType"], "3",
		"{body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_patch_can_clear_profile_specific_field() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-CLEAR", &["fda"]).await?;
	create_safety_report_with_local_criteria(
		&app,
		&cookie,
		&case_id,
		"2",
		true,
		Some("1"),
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI"),
		json!({
			"authorities": ["fda"],
			"rows": { "safetyReportIdentification": { "localCriteriaReportType": null } }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["localCriteriaReportType"],
		Value::Null
	);
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/safety-report"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["local_criteria_report_type"], Value::Null);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ci_page_projection_preserves_request_profile_context() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-CI-REQUEST-APPENDIX",
		&["fda"],
	)
	.await?;
	create_safety_report(&app, &cookie, &case_id, "2", true).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=ich"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert!(body.get("appendices").is_none(), "{body}");
	assert_eq!(body["authorities"], json!(["ich"]));
	assert!(
		body["fields"]
			.as_object()
			.ok_or("missing fields object")?
			.is_empty(),
		"{body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_page_projection_rejects_unknown_profile_context() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-BAD-APPENDIX", &["ich"])
			.await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/CI?authorities=unknown"),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_remaining_direct_pages_have_projection_routes() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-PAGES", &["ich"]).await?;

	for (section, expected_key) in [
		("RP", "primarySources"),
		("SD", "senderInformation"),
		("LR", "literatureReferences"),
		("SI", "studyInformation"),
		("DM", "patientInformation"),
		("NR", "narrative"),
	] {
		let (status, body) = get_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}?authorities=fda"),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert_eq!(body["caseId"], case_id);
		assert_eq!(body["pageId"], section);
		assert!(body.get("appendices").is_none(), "{section}: {body}");
		assert!(body["saved"].as_bool().is_some(), "{section}: {body}");
		assert!(
			body["requiredCount"].as_u64().is_some(),
			"{section}: {body}"
		);
		assert!(body["fields"].is_object(), "{section}: {body}");
		assert!(body["rows"].is_object(), "{section}: {body}");
		assert!(
			body["rows"].get(expected_key).is_some(),
			"{section}: {body}"
		);
		if matches!(section, "DM" | "NR" | "SI") {
			assert_eq!(body["saved"], false, "{section}: {body}");
		}
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_remaining_direct_pages_accept_page_patch_with_profiles() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-PAGES-PATCH", &["ich"])
			.await?;

	for section in ["RP", "SD", "LR", "SI", "DM", "NR"] {
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}"),
			json!({
				"authorities": ["fda"]
			}),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert!(body.get("appendices").is_none(), "{section}: {body}");
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_remaining_direct_pages_accept_rows_with_profiles() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DIRECT-DELTAS", &["ich"])
			.await?;

	let requests = [
		(
			"RP",
			json!({ "primarySources": [{ "reporterGivenName": "Jane" }] }),
		),
		(
			"SD",
			json!({ "senderInformation": { "organizationName": "Sender Org" } }),
		),
		(
			"LR",
			json!({ "literatureReferences": [{ "referenceText": "PMID:1" }] }),
		),
		(
			"SI",
			json!({ "studyInformation": { "studyName": "Study A" } }),
		),
		(
			"DM",
			json!({ "patientInformation": { "patientInitials": "AB" } }),
		),
		(
			"NR",
			json!({ "narrative": { "caseNarrative": "Narrative" } }),
		),
	];

	for (section, rows) in requests {
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}"),
			json!({
				"authorities": ["fda", "mfds"],
				"rows": rows
			}),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert_eq!(
			body["authorities"],
			json!(["fda", "mfds"]),
			"{section}: {body}"
		);
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_direct_page_patch_rejects_unknown_row() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-PATCH-UNKNOWN", &["ich"])
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["fda"],
			"rows": { "notAReporterRow": {} }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_direct_page_patch_rejects_unknown_profile() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-PATCH-BAD-APPENDIX", &["ich"])
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["unknown"]
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_nr_page_patch_persists_narrative_row() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-NR-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"narrative": {
					"caseNarrative": "Narrative saved through page patch"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["narrative"]["case_narrative"],
		"Narrative saved through page patch"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_page_patch_persists_primary_source_row() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-RP-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"primarySources": [{
					"sequenceNumber": 1,
					"qualification": "1"
				}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["rows"]["primarySources"][0]["qualification"], "1");
	let source_id = body["rows"]["primarySources"][0]["id"]
		.as_str()
		.ok_or("missing primary source id")?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"primarySources": [{
					"id": source_id,
					"sequenceNumber": 1,
					"deleted": true
				}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["rows"]["primarySources"], json!([]));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_rows_accept_frontend_canonical_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-RP-CHANGES", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich"],
			"rows": { "primarySources": [{
				"reporterGivenName": "Canonical",
				"reporterCountryNullFlavor": "UNK",
				"reporterEmail": "canonical@example.test",
				"qualification": "1",
				"primarySourceForRegulatoryPurposes": "1"
			}] }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	let row = &body["rows"]["primarySources"][0];
	assert_eq!(row["reporterGivenName"], "Canonical");
	assert_eq!(row["reporterCountryNullFlavor"], "UNK");
	assert_eq!(row["reporterEmail"], "canonical@example.test");
	assert_eq!(row["qualification"], "1");
	assert_eq!(row["primarySourceForRegulatoryPurposes"], "1");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_projection_does_not_leak_rows_from_another_case() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let populated_case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-RP-POPULATED", &["ich"])
			.await?;
	let empty_case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-RP-EMPTY", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{populated_case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"primarySources": [{
					"sequenceNumber": 1,
					"reporterGivenName": "Only populated case"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{empty_case_id}/editor/pages/RP?authorities=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["rows"]["primarySources"], json!([]), "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_complete_fields_round_trip() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-RP-COMPLETE",
		&["ich", "fda", "mfds"],
	)
	.await?;

	let concrete = json!({
		"sequenceNumber": 1,
		"reporterTitle": "Dr",
		"reporterGivenName": "Mina",
		"reporterMiddleName": "J",
		"reporterFamilyName": "Kim",
		"reporterOrganization": "QVIS Safety",
		"reporterDepartment": "Pharmacovigilance",
		"reporterStreet": "1 Main Street",
		"reporterCity": "Seoul",
		"reporterState": "Seoul",
		"reporterPostcode": "04524",
		"reporterTelephone": "+821012345678",
		"reporterCountry": "KR",
		"reporterEmail": "reporter@example.test",
		"qualification": "1",
		"qualificationKr1": "2",
		"primarySourceForRegulatoryPurposes": "1"
	});
	let null_flavors = json!({
		"sequenceNumber": 2,
		"reporterTitleNullFlavor": "MSK",
		"reporterGivenNameNullFlavor": "MSK",
		"reporterMiddleNameNullFlavor": "MSK",
		"reporterFamilyNameNullFlavor": "MSK",
		"reporterOrganizationNullFlavor": "MSK",
		"reporterDepartmentNullFlavor": "MSK",
		"reporterStreetNullFlavor": "MSK",
		"reporterCityNullFlavor": "MSK",
		"reporterStateNullFlavor": "MSK",
		"reporterPostcodeNullFlavor": "MSK",
		"reporterTelephoneNullFlavor": "NASK",
		"reporterCountryNullFlavor": "UNK",
		"reporterEmailNullFlavor": "NASK",
		"qualificationNullFlavor": "UNK"
	});

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {"primarySources": [concrete, null_flavors]}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP?authorities=ich,fda,mfds"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"]["primarySources"]
		.as_array()
		.ok_or("missing primarySources rows")?;
	assert_eq!(rows.len(), 2, "{body}");
	assert_eq!(rows[0]["reporterTitle"], "Dr");
	assert_eq!(rows[0]["reporterGivenName"], "Mina");
	assert_eq!(rows[0]["reporterMiddleName"], "J");
	assert_eq!(rows[0]["reporterFamilyName"], "Kim");
	assert_eq!(rows[0]["reporterOrganization"], "QVIS Safety");
	assert_eq!(rows[0]["reporterDepartment"], "Pharmacovigilance");
	assert_eq!(rows[0]["reporterStreet"], "1 Main Street");
	assert_eq!(rows[0]["reporterCity"], "Seoul");
	assert_eq!(rows[0]["reporterState"], "Seoul");
	assert_eq!(rows[0]["reporterPostcode"], "04524");
	assert_eq!(rows[0]["reporterTelephone"], "+821012345678");
	assert_eq!(rows[0]["reporterCountry"], "KR");
	assert_eq!(rows[0]["reporterEmail"], "reporter@example.test");
	assert_eq!(rows[0]["qualification"], "1");
	assert_eq!(rows[0]["qualificationKr1"], "2");
	assert_eq!(rows[0]["primarySourceForRegulatoryPurposes"], "1");
	assert_eq!(rows[1]["reporterTitleNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterGivenNameNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterMiddleNameNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterFamilyNameNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterOrganizationNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterDepartmentNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterStreetNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterCityNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterStateNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterPostcodeNullFlavor"], "MSK");
	assert_eq!(rows[1]["reporterTelephoneNullFlavor"], "NASK");
	assert_eq!(rows[1]["reporterCountryNullFlavor"], "UNK");
	assert_eq!(rows[1]["reporterEmailNullFlavor"], "NASK");
	assert_eq!(rows[1]["qualificationNullFlavor"], "UNK");
	assert!(rows[0].get("reporter_given_name").is_none(), "{body}");

	mm.dbx().begin_txn().await?;
	set_full_context_dbx(
		mm.dbx(),
		seed.admin.id,
		seed.org_id,
		ROLE_SPONSOR_ADMIN_CRO,
	)
	.await?;
	let stored = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (Value,)>(
				"SELECT jsonb_agg(jsonb_build_object(
				'reporter_given_name', reporter_given_name,
				'qualification_kr1', qualification_kr1,
				'reporter_given_name_null_flavor', reporter_given_name_null_flavor,
				'qualification_null_flavor', qualification_null_flavor
			) ORDER BY sequence_number)
			FROM primary_sources WHERE case_id = $1",
			)
			.bind(Uuid::parse_str(&case_id)?),
		)
		.await?
		.0;
	mm.dbx().commit_txn().await?;
	assert_eq!(stored[0]["reporter_given_name"], "Mina");
	assert_eq!(stored[0]["qualification_kr1"], "2");
	assert_eq!(stored[1]["reporter_given_name_null_flavor"], "MSK");
	assert_eq!(stored[1]["qualification_null_flavor"], "UNK");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_portable_constraints_return_structured_paths() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-RP-CONSTRAINTS",
		&["ich", "fda", "mfds"],
	)
	.await?;

	let too_long = Value::String("X".repeat(10_001));
	let cases = [
		(
			"reporterTitle",
			too_long.clone(),
			"ICH.C.2.r.1.1.LENGTH.MAX",
		),
		(
			"reporterGivenName",
			too_long.clone(),
			"ICH.C.2.r.1.2.LENGTH.MAX",
		),
		(
			"reporterMiddleName",
			too_long.clone(),
			"ICH.C.2.r.1.3.LENGTH.MAX",
		),
		(
			"reporterFamilyName",
			too_long.clone(),
			"ICH.C.2.r.1.4.LENGTH.MAX",
		),
		(
			"reporterOrganization",
			too_long.clone(),
			"ICH.C.2.r.2.1.LENGTH.MAX",
		),
		(
			"reporterDepartment",
			too_long.clone(),
			"ICH.C.2.r.2.2.LENGTH.MAX",
		),
		(
			"reporterStreet",
			too_long.clone(),
			"ICH.C.2.r.2.3.LENGTH.MAX",
		),
		("reporterCity", too_long.clone(), "ICH.C.2.r.2.4.LENGTH.MAX"),
		(
			"reporterState",
			too_long.clone(),
			"ICH.C.2.r.2.5.LENGTH.MAX",
		),
		(
			"reporterPostcode",
			too_long.clone(),
			"ICH.C.2.r.2.6.LENGTH.MAX",
		),
		(
			"reporterTelephone",
			too_long.clone(),
			"ICH.C.2.r.2.7.LENGTH.MAX",
		),
		("reporterCountry", json!("KOR"), "ICH.C.2.r.3.LENGTH.MAX"),
		(
			"reporterEmail",
			too_long.clone(),
			"FDA.C.2.r.2.8.LENGTH.MAX",
		),
		("qualification", json!("9"), "ICH.C.2.r.4.ALLOWED.VALUE"),
		(
			"qualificationKr1",
			too_long.clone(),
			"MFDS.C.2.r.4.KR.1.LENGTH.MAX",
		),
		(
			"primarySourceForRegulatoryPurposes",
			json!("2"),
			"ICH.C.2.r.5.ALLOWED.VALUE",
		),
		(
			"reporterTitleNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.1.1.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterGivenNameNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.1.2.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterMiddleNameNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.1.3.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterFamilyNameNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.1.4.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterOrganizationNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.1.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterDepartmentNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.2.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterStreetNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.3.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterCityNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.4.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterStateNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.5.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterPostcodeNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.6.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterTelephoneNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.2.7.NULLFLAVOR.ALLOWED",
		),
		(
			"reporterCountryNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.3.NULLFLAVOR.ALLOWED",
		),
		(
			"qualificationNullFlavor",
			json!("BAD"),
			"ICH.C.2.r.4.NULLFLAVOR.ALLOWED",
		),
	];

	for (field, invalid, expected_rule) in cases {
		let mut source = serde_json::Map::new();
		source.insert("sequenceNumber".to_string(), json!(1));
		source.insert(field.to_string(), invalid);
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/RP"),
			json!({
				"authorities": ["ich", "fda", "mfds"],
				"rows": {"primarySources": [Value::Object(source)]}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}: {body}");
		assert_eq!(
			body["error"]["message"], "CONSTRAINT_VIOLATION",
			"{field}: {body}"
		);
		assert_eq!(
			body["error"]["data"]["detail"]["ruleCode"], expected_rule,
			"{field}: {body}"
		);
		assert_eq!(
			body["error"]["data"]["detail"]["path"],
			format!("primarySources.0.{field}"),
			"{field}: {body}"
		);
	}

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich"],
			"rows": {"primarySources": [
				{"sequenceNumber": 1, "reporterTitle": "Dr"},
				{"sequenceNumber": 2, "reporterTitle": "X".repeat(10_001)}
			]}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.C.2.r.1.1.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"primarySources.1.reporterTitle"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_rp_business_validation_paths_are_canonical() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-RP-BUSINESS",
		&["ich", "fda", "mfds"],
	)
	.await?;

	let assert_issue_path = |body: &Value, code: &str, expected_path: &str| {
		let issue = body["data"]["issues"]
			.as_array()
			.and_then(|issues| {
				issues
					.iter()
					.find(|issue| issue["code"].as_str() == Some(code))
			})
			.unwrap_or_else(|| panic!("missing {code}: {body}"));
		assert_eq!(issue["path"], expected_path, "{code}: {body}");
		assert_eq!(issue["field_path"], expected_path, "{code}: {body}");
	};

	let (status, ich_body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{ich_body}");
	assert_issue_path(
		&ich_body,
		"ICH.C.2.r.4.REQUIRED",
		"primarySources.0.qualification",
	);
	assert_issue_path(
		&ich_body,
		"ICH.C.2.r.5.REQUIRED",
		"primarySources.0.primarySourceForRegulatoryPurposes",
	);

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/RP"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {"primarySources": [{
				"sequenceNumber": 1,
				"reporterOrganization": "QVIS Safety",
				"qualification": "3"
			}]}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, fda_body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{fda_body}");
	assert_issue_path(
		&fda_body,
		"FDA.C.2.r.2.8.REQUIRED",
		"primarySources.0.reporterEmail",
	);

	let (status, mfds_body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=mfds"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{mfds_body}");
	assert_issue_path(
		&mfds_body,
		"MFDS.C.2.r.4.KR.1.REQUIRED",
		"primarySources.0.qualificationKr1",
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_sd_page_patch_persists_sender_information_row() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-SD-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SD"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"senderInformation": {
					"senderType": "3",
					"organizationName": "Sender Org",
					"healthProfessionalTypeKr1": "4",
					"personGivenName": "Sora",
					"email": "sender@example.test"
				},
				"receiverInformation": {
					"receiverType": "2",
					"organizationName": "Receiver Org",
					"stateProvince": "Seoul",
					"countryCode": "KR",
					"email": "receiver@example.test"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["senderOrganization"],
		"Sender Org"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]
			["senderHealthProfessionalTypeKr1"],
		"4"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["senderPersonGivenName"],
		"Sora"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["receiverOrganization"],
		"Receiver Org"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["receiverState"],
		"Seoul"
	);
	assert!(body["rows"].get("messageHeader").is_none(), "{body}");

	// A second row patch must update the existing singleton instead of
	// attempting to create another sender row.
	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SD"),
		json!({
			"rows": {
				"senderInformation": { "organizationName": "Sender Org Updated" },
				"receiverInformation": { "organizationName": "Receiver Org Updated" }
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["senderOrganization"],
		"Sender Org Updated"
	);
	assert_eq!(
		body["rows"]["safetyReportIdentification"]["receiverOrganization"],
		"Receiver Org Updated"
	);
	assert_eq!(
		body["rows"]["senderInformation"].as_array().map(Vec::len),
		Some(1)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_sd_complete_fields_round_trip() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-SD-COMPLETE",
		&["ich", "fda", "mfds"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SD"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {
				"senderInformation": {
					"senderType": "3", "healthProfessionalTypeKr1": "4",
					"organizationName": "Sender Org", "department": "Safety",
					"personTitle": "Dr", "personGivenName": "Sora",
					"personMiddleName": "J", "personFamilyName": "Kim",
					"streetAddress": "1 Sender Street", "city": "Seoul",
					"state": "Seoul", "postcode": "04524", "countryCode": "KR",
					"telephone": "+821012345678", "fax": "+8221234567",
					"email": "sender@example.test"
				},
				"receiverInformation": {
					"receiverType": "2", "organizationName": "Receiver Org",
					"department": "Intake", "streetAddress": "2 Receiver Street",
					"city": "Busan", "stateProvince": "Busan", "postcode": "48000",
					"countryCode": "KR", "telephone": "+82511234567",
					"fax": "+82517654321", "email": "receiver@example.test"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SD?authorities=ich,fda,mfds"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	let report = &reloaded["rows"]["safetyReportIdentification"];
	for (field, expected) in [
		("senderType", json!("3")),
		("senderHealthProfessionalTypeKr1", json!("4")),
		("senderOrganization", json!("Sender Org")),
		("senderDepartment", json!("Safety")),
		("senderPersonTitle", json!("Dr")),
		("senderPersonGivenName", json!("Sora")),
		("senderPersonMiddleName", json!("J")),
		("senderPersonFamilyName", json!("Kim")),
		("senderStreetAddress", json!("1 Sender Street")),
		("senderCity", json!("Seoul")),
		("senderState", json!("Seoul")),
		("senderPostcode", json!("04524")),
		("senderCountryCode", json!("KR")),
		("senderTelephone", json!("+821012345678")),
		("senderFax", json!("+8221234567")),
		("senderEmail", json!("sender@example.test")),
		("receiverType", json!("2")),
		("receiverOrganization", json!("Receiver Org")),
		("receiverDepartment", json!("Intake")),
		("receiverStreet", json!("2 Receiver Street")),
		("receiverCity", json!("Busan")),
		("receiverState", json!("Busan")),
		("receiverPostcode", json!("48000")),
		("receiverCountry", json!("KR")),
		("receiverTelephone", json!("+82511234567")),
		("receiverFax", json!("+82517654321")),
		("receiverEmail", json!("receiver@example.test")),
	] {
		assert_eq!(report[field], expected, "{field}: {reloaded}");
	}
	assert!(
		reloaded["rows"].get("messageHeader").is_none(),
		"{reloaded}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_sd_portable_constraints_return_structured_paths() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-SD-CONSTRAINTS",
		&["ich", "fda", "mfds"],
	)
	.await?;
	let too_long = Value::String("X".repeat(10_001));

	for (field, invalid, expected_rule) in [
		("senderType", json!("9"), "ICH.C.3.1.ALLOWED.VALUE"),
		(
			"senderHealthProfessionalTypeKr1",
			too_long.clone(),
			"MFDS.C.3.1.KR.1.LENGTH.MAX",
		),
		(
			"senderOrganization",
			too_long.clone(),
			"ICH.C.3.2.LENGTH.MAX",
		),
		(
			"senderDepartment",
			too_long.clone(),
			"ICH.C.3.3.1.LENGTH.MAX",
		),
		(
			"senderPersonTitle",
			too_long.clone(),
			"ICH.C.3.3.2.LENGTH.MAX",
		),
		(
			"senderPersonGivenName",
			too_long.clone(),
			"ICH.C.3.3.3.LENGTH.MAX",
		),
		(
			"senderPersonMiddleName",
			too_long.clone(),
			"ICH.C.3.3.4.LENGTH.MAX",
		),
		(
			"senderPersonFamilyName",
			too_long.clone(),
			"ICH.C.3.3.5.LENGTH.MAX",
		),
		(
			"senderStreetAddress",
			too_long.clone(),
			"ICH.C.3.4.1.LENGTH.MAX",
		),
		("senderCity", too_long.clone(), "ICH.C.3.4.2.LENGTH.MAX"),
		("senderState", too_long.clone(), "ICH.C.3.4.3.LENGTH.MAX"),
		("senderPostcode", too_long.clone(), "ICH.C.3.4.4.LENGTH.MAX"),
		("senderCountryCode", json!("KOR"), "ICH.C.3.4.5.LENGTH.MAX"),
		(
			"senderTelephone",
			too_long.clone(),
			"ICH.C.3.4.6.LENGTH.MAX",
		),
		("senderFax", too_long.clone(), "ICH.C.3.4.7.LENGTH.MAX"),
		("senderEmail", too_long.clone(), "ICH.C.3.4.8.LENGTH.MAX"),
	] {
		let row_field = match field {
			"senderHealthProfessionalTypeKr1" => "healthProfessionalTypeKr1",
			"senderOrganization" => "organizationName",
			"senderDepartment" => "department",
			"senderPersonTitle" => "personTitle",
			"senderPersonGivenName" => "personGivenName",
			"senderPersonMiddleName" => "personMiddleName",
			"senderPersonFamilyName" => "personFamilyName",
			"senderStreetAddress" => "streetAddress",
			"senderCity" => "city",
			"senderState" => "state",
			"senderPostcode" => "postcode",
			"senderCountryCode" => "countryCode",
			"senderTelephone" => "telephone",
			"senderFax" => "fax",
			"senderEmail" => "email",
			_ => field,
		};
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/SD"),
			json!({
				"authorities": ["ich", "fda", "mfds"],
				"rows": { "senderInformation": { row_field: invalid } }
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}: {body}");
		assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
		assert_eq!(
			body["error"]["data"]["detail"]["ruleCode"], expected_rule,
			"{field}: {body}"
		);
		assert_eq!(
			body["error"]["data"]["detail"]["path"],
			format!("safetyReportIdentification.{field}"),
			"{field}: {body}"
		);
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_sd_page_patch_rejects_export_owned_message_header_change(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-SD-BATCH", &["ich"]).await?;

	// message_number is globally unique; use a fresh value to avoid colliding
	// with seed data (db/seed/001-demo-seed.sql uses "MSG-001").
	let message_number = format!("MSG-{}", Uuid::new_v4());
	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_number": message_number,
				"message_sender_identifier": "SENDER",
				"message_receiver_identifier": "OLD-RECEIVER",
				"message_date": "20260603120000"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SD"),
		json!({
			"authorities": ["fda"],
			"rows": { "messageHeader": {
				"messageReceiverIdentifier": "CDER",
				"batchReceiverIdentifier": "ZZFDA",
				"batchTransmissionDate": "20260724153045"
			} }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
	assert_eq!(
		body["error"]["data"]["detail"], "unknown SD row 'messageHeader'",
		"{body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lr_page_patch_persists_literature_reference_row() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-LR-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LR"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"literatureReferences": [
					{
						"sequenceNumber": 1,
						"referenceText": "Smith 2026",
						"documentBase64": "UEZERg==",
						"mediaType": "application/pdf",
						"representation": "B64"
					},
					{
						"sequenceNumber": 2,
						"referenceText": "Kim 2026"
					}
				]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["literatureReferences"][0]["referenceText"],
		"Smith 2026"
	);
	assert_eq!(
		body["rows"]["literatureReferences"][0]["mediaType"],
		"application/pdf"
	);
	assert_eq!(
		body["rows"]["literatureReferences"][1]["referenceText"],
		"Kim 2026"
	);

	let first_id = body["rows"]["literatureReferences"][0]["id"]
		.as_str()
		.expect("first literature id");
	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LR"),
		json!({
			"rows": {
				"literatureReferences": [{
					"id": first_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["literatureReferences"]
			.as_array()
			.map(Vec::len),
		Some(1)
	);
	assert_eq!(
		body["rows"]["literatureReferences"][0]["referenceText"],
		"Kim 2026"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lr_contract_round_trip_and_portable_constraints() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-LR-CONTRACT", &["ich", "fda"])
			.await?;

	let (status, saved) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LR"),
		json!({
			"authorities": ["ich", "fda"],
			"rows": {
				"literatureReferences": [
					{
						"sequenceNumber": 1,
						"referenceText": "Smith et al. Safety study, 2026",
						"documentBase64": "UEZERg==",
						"mediaType": "application/pdf",
						"representation": "B64"
					},
					{
						"sequenceNumber": 2,
						"referenceTextNullFlavor": "ASKU"
					}
				]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{saved}");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LR?authorities=ich,fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	let rows = &reloaded["rows"]["literatureReferences"];
	assert_eq!(rows[0]["referenceText"], "Smith et al. Safety study, 2026");
	assert_eq!(rows[0]["documentBase64"], "UEZERg==");
	assert_eq!(rows[0]["mediaType"], "application/pdf");
	assert_eq!(rows[0]["representation"], "B64");
	assert_eq!(rows[1]["referenceTextNullFlavor"], "ASKU");

	for (field, invalid, expected_rule) in [
		(
			"referenceText",
			json!("X".repeat(10_001)),
			"ICH.C.4.r.1.LENGTH.MAX",
		),
		(
			"referenceTextNullFlavor",
			json!("BAD"),
			"ICH.C.4.r.1.NULLFLAVOR.ALLOWED",
		),
		(
			"documentBase64",
			json!("not-base64"),
			"ICH.C.4.r.2.ALLOWED.VALUE",
		),
	] {
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/LR"),
			json!({
				"authorities": ["ich", "fda"],
				"rows": {
					"literatureReferences": [{
						"sequenceNumber": 3,
						field: invalid
					}]
				}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{field}: {body}");
		assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
		assert_eq!(
			body["error"]["data"]["detail"]["ruleCode"], expected_rule,
			"{field}: {body}"
		);
		assert_eq!(
			body["error"]["data"]["detail"]["path"],
			format!("literatureReferences.0.{field}"),
			"{field}: {body}"
		);
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_si_page_patch_round_trips_every_contract_field() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-SI-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SI"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"studyInformation": {
					"studyName": "Study 001",
					"sponsorStudyNumber": "SP-2026-001",
					"studyTypeReaction": "2",
					"studyTypeReactionKr1": "1",
					"fdaIndNumberOccurred": "123456",
					"fdaPreAndaNumberOccurred": "234567",
					"fdaCrossReportedIndNumbers": [{
						"indNumber": "654321",
						"indNumberNullFlavor": "UNK",
						"sequenceNumber": 1
					}]
				},
				"studyRegistrationNumbers": [{
					"registrationNumber": "NCT-2026-001",
					"countryCode": "KR",
					"sequenceNumber": 1
				}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["rows"]["studyInformation"]["study_name"], "Study 001");
	assert_eq!(
		body["rows"]["studyInformation"]["sponsor_study_number"],
		"SP-2026-001"
	);
	assert_eq!(body["rows"]["studyInformation"]["study_type_reaction"], "2");
	assert_eq!(
		body["rows"]["studyInformation"]["study_type_reaction_kr1"],
		"1"
	);
	assert_eq!(
		body["rows"]["studyInformation"]["fda_ind_number_occurred"],
		"123456"
	);
	assert_eq!(
		body["rows"]["studyInformation"]["fda_pre_anda_number_occurred"],
		"234567"
	);
	assert_eq!(
		body["rows"]["studyRegistrationNumbers"][0]["registration_number"],
		"NCT-2026-001"
	);
	assert_eq!(
		body["rows"]["studyRegistrationNumbers"][0]["country_code"],
		"KR"
	);
	assert_eq!(
		body["rows"]["studyInformation"]["fdaCrossReportedIndNumbers"][0]
			["ind_number"],
		"654321"
	);
	assert_eq!(
		body["rows"]["studyInformation"]["fdaCrossReportedIndNumbers"][0]
			["ind_number_null_flavor"],
		"UNK"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SI"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], body["rows"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_si_portable_constraints_return_structured_paths() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-SI-CONSTRAINTS",
		&["ich", "fda", "mfds"],
	)
	.await?;
	let too_long = "X".repeat(10_001);
	let cases = [
		(
			json!({
				"studyInformation": {},
				"studyRegistrationNumbers": [{"registrationNumber": too_long}]
			}),
			"ICH.C.5.1.r.1.LENGTH.MAX",
			"studyInformation.studyRegistrationNumbers.0.registrationNumber",
		),
		(
			json!({
				"studyInformation": {},
				"studyRegistrationNumbers": [{"countryCode": "KOR"}]
			}),
			"ICH.C.5.1.r.2.LENGTH.MAX",
			"studyInformation.studyRegistrationNumbers.0.countryCode",
		),
		(
			json!({"studyInformation": {"studyName": "X".repeat(2001)}}),
			"ICH.C.5.2.LENGTH.MAX",
			"studyInformation.studyName",
		),
		(
			json!({"studyInformation": {"sponsorStudyNumber": "X".repeat(101)}}),
			"ICH.C.5.3.LENGTH.MAX",
			"studyInformation.sponsorStudyNumber",
		),
		(
			json!({"studyInformation": {"studyTypeReaction": "9"}}),
			"ICH.C.5.4.ALLOWED.VALUE",
			"studyInformation.studyTypeReaction",
		),
		(
			json!({"studyInformation": {"studyTypeReactionKr1": "12"}}),
			"MFDS.C.5.4.KR.1.LENGTH.MAX",
			"studyInformation.studyTypeReactionKr1",
		),
		(
			json!({"studyInformation": {"fdaIndNumberOccurred": "X".repeat(11)}}),
			"FDA.C.5.5a.LENGTH.MAX",
			"studyInformation.fdaIndNumberOccurred",
		),
		(
			json!({"studyInformation": {"fdaPreAndaNumberOccurred": "X".repeat(11)}}),
			"FDA.C.5.5b.LENGTH.MAX",
			"studyInformation.fdaPreAndaNumberOccurred",
		),
		(
			json!({"studyInformation": {"fdaCrossReportedIndNumbers": [{"indNumber": "X".repeat(11)}]}}),
			"FDA.C.5.6.r.LENGTH.MAX",
			"studyInformation.fdaCrossReportedIndNumbers.0.indNumber",
		),
		(
			json!({"studyInformation": {"fdaCrossReportedIndNumbers": [{"indNumberNullFlavor": "BAD"}]}}),
			"FDA.C.5.6.r.NULLFLAVOR.ALLOWED",
			"studyInformation.fdaCrossReportedIndNumbers.0.indNumberNullFlavor",
		),
	];

	for (rows, expected_rule, expected_path) in cases {
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/SI"),
			json!({
				"authorities": ["ich", "fda", "mfds"],
				"rows": rows
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
		assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
		assert_eq!(body["error"]["data"]["detail"]["ruleCode"], expected_rule);
		assert_eq!(body["error"]["data"]["detail"]["path"], expected_path);
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_si_fda_save_ignores_mfds_study_type_field() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-SI-FDA-ONLY", &["ich", "fda"])
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/SI"),
		json!({
			"authorities": ["ich", "fda"],
			"rows": {
				"studyInformation": {
					"studyTypeReaction": "1",
					"studyTypeReactionKr1": "4"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["rows"]["studyInformation"]["study_type_reaction"], "1");
	assert!(body["rows"]["studyInformation"]["study_type_reaction_kr1"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_patch_round_trips_base_patient_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-PATCH", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"patientInformation": {
					"patientInitials": "ABC",
					"patientBirthDate": "19900102",
					"patientAge": 36.5, "unit": "a",
					"gestationPeriod": 22, "unit": "wk",
					"patientAgeGroup": "5",
					"patientWeight": 62.5,
					"patientHeight": 171,
					"patientSex": "2",
					"lastMenstrualPeriodDate": "20260102",
					"medicalHistoryText": "History text",
					"concomitantTherapies": true,
					"raceCode": "C41260",
					"ethnicityCode": "C41222"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["patientInformation"]["patient_initials"],
		"ABC"
	);
	assert_eq!(body["rows"]["patientInformation"]["birth_date"], "19900102");
	assert_eq!(
		body["rows"]["patientInformation"]["age_at_time_of_onset"],
		"36.50"
	);
	assert_eq!(body["rows"]["patientInformation"]["age_unit"], "a");
	assert_eq!(
		body["rows"]["patientInformation"]["gestation_period"],
		"22.00"
	);
	assert_eq!(
		body["rows"]["patientInformation"]["gestation_period_unit"],
		"wk"
	);
	assert_eq!(body["rows"]["patientInformation"]["age_group"], "5");
	assert_eq!(body["rows"]["patientInformation"]["weight_kg"], "62.50");
	assert_eq!(body["rows"]["patientInformation"]["height_cm"], "171.00");
	assert_eq!(body["rows"]["patientInformation"]["sex"], "2");
	assert_eq!(
		body["rows"]["patientInformation"]["last_menstrual_period_date"],
		"20260102"
	);
	assert_eq!(
		body["rows"]["patientInformation"]["medical_history_text"],
		"History text"
	);
	assert_eq!(
		body["rows"]["patientInformation"]["concomitant_therapy"],
		true
	);
	assert_eq!(body["rows"]["patientInformation"]["race_code"], "C41260");
	assert_eq!(
		body["rows"]["patientInformation"]["ethnicity_code"],
		"C41222"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], body["rows"]);

	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {
					"patientInitials": "XYZ",
					"patientWeight": 63.0
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["patientInformation"]["patient_initials"],
		"XYZ"
	);
	assert_eq!(updated["rows"]["patientInformation"]["weight_kg"], "63.00");
	assert_eq!(
		updated["rows"]["patientInformation"]["birth_date"],
		"19900102"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rows_round_trip_d_7_2_value_and_null_flavor() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-D72-ROWS", &["ich"])
			.await?;

	let (status, saved_value) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": { "patientInformation": {
				"medicalHistoryText": "Relevant history"
			} }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{saved_value}");
	assert_eq!(
		saved_value["rows"]["patientInformation"]["medical_history_text"],
		"Relevant history"
	);

	let (status, saved_null_flavor) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": { "patientInformation": {
				"medicalHistoryTextNullFlavor": "UNK"
			} }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{saved_null_flavor}");
	assert_eq!(
		saved_null_flavor["rows"]["patientInformation"]["medical_history_text"],
		Value::Null
	);
	assert_eq!(
		saved_null_flavor["rows"]["patientInformation"]
			["medical_history_text_null_flavor"],
		"UNK"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], saved_null_flavor["rows"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_page_patch_rejects_removed_changes_contract() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-ROWS-ONLY", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"changes": {
				"medicalHistoryText": "must be rejected"
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-CONSTRAINT", &["ich"])
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {
					"patientAgeGroup": "9"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.2.3.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.patientAgeGroup"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_cruds_medical_history_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-HISTORY", &["ich"]).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"medicalHistoryEpisodes": [{
					"sequenceNumber": 1,
					"meddraVersion": "26.0",
					"meddraCode": "10000001",
					"startDate": "20200102",
					"continuing": false,
					"endDate": "20210102",
					"comments": "resolved",
					"familyHistory": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["meddra_version"],
		"26.0"
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["meddra_code"],
		"10000001"
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["start_date"],
		"20200102"
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["continuing"],
		false
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["end_date"],
		"20210102"
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["comments"],
		"resolved"
	);
	assert_eq!(
		body["rows"]["medicalHistoryEpisodes"][0]["family_history"],
		true
	);

	let row_id = body["rows"]["medicalHistoryEpisodes"][0]["id"]
		.as_str()
		.expect("medical history id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"medicalHistoryEpisodes": [{
					"id": row_id,
					"comments": "updated"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["medicalHistoryEpisodes"][0]["comments"],
		"updated"
	);
	assert_eq!(
		updated["rows"]["medicalHistoryEpisodes"][0]["meddra_code"],
		"10000001"
	);

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"medicalHistoryEpisodes": [{
					"id": row_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["rows"]["medicalHistoryEpisodes"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_medical_history_catalog_constraint_before_write(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-HISTORY-CONSTRAINT",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"medicalHistoryEpisodes": [{
					"continuing": "yes"
				}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.7.1.r.3.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.medicalHistoryEpisodes.0.continuing"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());
	assert_eq!(
		reloaded["rows"]["medicalHistoryEpisodes"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_cruds_patient_death_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-DEATH", &["ich"]).await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"deathInfo": {
					"dateOfDeath": "20240102",
					"autopsyPerformed": true
				},
				"reportedCauses": [{
					"sequenceNumber": 1,
					"meddraVersion": "26.0",
					"meddraCode": "10000001",
					"causeText": "reported cause"
				}],
				"autopsyCauses": [{
					"sequenceNumber": 1,
					"meddraVersion": "26.0",
					"meddraCode": "10000001",
					"causeText": "autopsy cause"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	assert_eq!(created["rows"]["deathInfo"]["date_of_death"], "20240102");
	assert_eq!(created["rows"]["deathInfo"]["autopsy_performed"], true);
	assert_eq!(
		created["rows"]["reportedCauses"][0]["meddra_version"],
		"26.0"
	);
	assert_eq!(
		created["rows"]["reportedCauses"][0]["comments"],
		"reported cause"
	);
	assert_eq!(
		created["rows"]["autopsyCauses"][0]["comments"],
		"autopsy cause"
	);

	let reported_id = created["rows"]["reportedCauses"][0]["id"]
		.as_str()
		.expect("reported cause id");
	let autopsy_id = created["rows"]["autopsyCauses"][0]["id"]
		.as_str()
		.expect("autopsy cause id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"deathInfo": {"autopsyPerformed": false},
				"reportedCauses": [{
					"id": reported_id,
					"causeText": "updated cause"
				}],
				"autopsyCauses": [{
					"id": autopsy_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(updated["rows"]["deathInfo"]["autopsy_performed"], false);
	assert_eq!(updated["rows"]["deathInfo"]["date_of_death"], "20240102");
	assert_eq!(
		updated["rows"]["reportedCauses"][0]["comments"],
		"updated cause"
	);
	assert_eq!(
		updated["rows"]["autopsyCauses"].as_array().map(Vec::len),
		Some(0)
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], updated["rows"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_patient_death_catalog_constraint_before_write(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-DEATH-CONSTRAINT",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"deathInfo": {"autopsyPerformed": "yes"}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.9.3.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.patientDeath.autopsyPerformed"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());
	assert!(reloaded["rows"]["deathInfo"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_round_trips_parent_information() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-PARENT", &["ich"]).await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {
					"parentIdentification": "MOTHER-01",
					"parentBirthDate": "19700102",
					"parentAge": 54, "unit": "a",
					"parentLastMenstrualPeriodDate": "20230102",
					"parentWeight": 64.5,
					"parentHeight": 168,
					"parentSex": "2",
					"medicalHistoryText": "Parent history"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	assert_eq!(
		created["rows"]["parentInfo"]["parent_identification"],
		"MOTHER-01"
	);
	assert_eq!(
		created["rows"]["parentInfo"]["parent_birth_date"],
		"19700102"
	);
	assert_eq!(created["rows"]["parentInfo"]["parent_age"], "54.00");
	assert_eq!(created["rows"]["parentInfo"]["parent_age_unit"], "a");
	assert_eq!(
		created["rows"]["parentInfo"]["last_menstrual_period_date"],
		"20230102"
	);
	assert_eq!(created["rows"]["parentInfo"]["weight_kg"], "64.50");
	assert_eq!(created["rows"]["parentInfo"]["height_cm"], "168.00");
	assert_eq!(created["rows"]["parentInfo"]["sex"], "2");
	assert_eq!(
		created["rows"]["parentInfo"]["medical_history_text"],
		"Parent history"
	);

	let parent_id = created["rows"]["parentInfo"]["id"]
		.as_str()
		.expect("parent id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"parentInfo": {
					"id": parent_id,
					"parentWeight": 65
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(updated["rows"]["parentInfo"]["weight_kg"], "65.00");
	assert_eq!(
		updated["rows"]["parentInfo"]["parent_identification"],
		"MOTHER-01"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"], updated["rows"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_parent_catalog_constraint_before_write() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-PARENT-CONSTRAINT",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {"parentSex": "9"}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.10.6.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.parentInformation.parentSex"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());
	assert!(reloaded["rows"]["parentInfo"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_cruds_parent_medical_history_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-PARENT-HISTORY", &["ich"])
			.await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {"parentIdentification": "MOTHER-01"},
				"parentMedicalHistory": [{
					"sequenceNumber": 1,
					"meddraVersion": "26.0",
					"meddraCode": "10000001",
					"startDate": "20200102",
					"continuing": false,
					"endDate": "20210102",
					"comments": "resolved"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	assert_eq!(
		created["rows"]["parentMedicalHistory"][0]["meddra_version"],
		"26.0"
	);
	assert_eq!(
		created["rows"]["parentMedicalHistory"][0]["start_date"],
		"20200102"
	);
	assert_eq!(
		created["rows"]["parentMedicalHistory"][0]["comments"],
		"resolved"
	);

	let row_id = created["rows"]["parentMedicalHistory"][0]["id"]
		.as_str()
		.expect("parent medical history id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"parentMedicalHistory": [{
					"id": row_id,
					"comments": "updated"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["parentMedicalHistory"][0]["comments"],
		"updated"
	);
	assert_eq!(
		updated["rows"]["parentMedicalHistory"][0]["meddra_code"],
		"10000001"
	);

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"parentMedicalHistory": [{
					"id": row_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["rows"]["parentMedicalHistory"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_parent_history_catalog_constraint_before_write(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-PARENT-HISTORY-CONSTRAINT",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {"parentIdentification": "MOTHER-01"},
				"parentMedicalHistory": [{"continuing": "yes"}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.10.7.1.r.3.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.parentInformation.medicalHistoryEpisodes.0.continuing"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());
	assert!(reloaded["rows"]["parentInfo"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_cruds_parent_past_drug_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-PARENT-DRUG",
		&["ich", "mfds"],
	)
	.await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {"parentIdentification": "MOTHER-01"},
				"parentPastDrugs": [{
					"sequenceNumber": 1,
					"drugName": "Parent drug",
					"mfdsMedicinalProductVersion": "2026",
					"mfdsMedicinalProductId": "MFDS-001",
					"mpidVersion": "1",
					"mpid": "MPID-001",
					"phpidVersion": "1",
					"phpid": "PHPID-001",
					"startDate": "20200102",
					"endDate": "20210102",
					"indicationMeddraVersion": "26.0",
					"indicationMeddraCode": "10000001",
					"reactionMeddraVersion": "26.0",
					"reactionMeddraCode": "10000001"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	let row = &created["rows"]["parentPastDrugs"][0];
	assert_eq!(row["drug_name"], "Parent drug");
	assert_eq!(row["mfds_medicinal_product_id"], "MFDS-001");
	assert_eq!(row["mpid"], "MPID-001");
	assert_eq!(row["phpid"], "PHPID-001");
	assert_eq!(row["start_date"], "20200102");
	assert_eq!(row["end_date"], "20210102");
	assert_eq!(row["indication_meddra_code"], "10000001");
	assert_eq!(row["reaction_meddra_code"], "10000001");

	let row_id = row["id"].as_str().expect("parent past drug id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"parentPastDrugs": [{
					"id": row_id,
					"drugName": "Updated parent drug"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["parentPastDrugs"][0]["drug_name"],
		"Updated parent drug"
	);
	assert_eq!(updated["rows"]["parentPastDrugs"][0]["mpid"], "MPID-001");

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"parentPastDrugs": [{
					"id": row_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["rows"]["parentPastDrugs"].as_array().map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_rejects_parent_past_drug_constraint_before_write(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-DM-PARENT-DRUG-CONSTRAINT",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"parentInfo": {"parentIdentification": "MOTHER-01"},
				"parentPastDrugs": [{"startDate": "not-a-date"}]
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.10.8.r.4.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.parentInformation.pastDrugHistory.0.startDate"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert!(reloaded["rows"]["patientInformation"].is_null());
	assert!(reloaded["rows"]["parentInfo"].is_null());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_page_cruds_local_patient_identifier_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DM-IDENTIFIER", &["ich"])
			.await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientInformation": {"patientInitials": "ABC"},
				"patientIdentifiers": [{
					"sequenceNumber": 1,
					"identifierTypeCode": "1",
					"identifierValue": "GP-001"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	assert_eq!(
		created["rows"]["patientIdentifiers"][0]["identifier_type_code"],
		"1"
	);
	assert_eq!(
		created["rows"]["patientIdentifiers"][0]["identifier_value"],
		"GP-001"
	);

	let row_id = created["rows"]["patientIdentifiers"][0]["id"]
		.as_str()
		.expect("patient identifier id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientIdentifiers": [{
					"id": row_id,
					"identifierValue": "GP-002"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["patientIdentifiers"][0]["identifier_value"],
		"GP-002"
	);
	assert_eq!(
		updated["rows"]["patientIdentifiers"][0]["identifier_type_code"],
		"1"
	);

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"rows": {
				"patientIdentifiers": [{
					"id": row_id,
					"deleted": true
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["rows"]["patientIdentifiers"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dm_returns_patient_payload_without_dh_list_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DM").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "ABC"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let patient_id = body["data"]["id"].as_str().ok_or("missing patient id")?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient/past-drugs"),
		json!({
			"data": {
				"patient_id": patient_id,
				"sequence_number": 1,
				"drug_name": "Prior Drug"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) =
		get_json(&app, &cookie, &format!("/api/cases/{case_id}/editor/DM")).await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let data = body.get("data").ok_or("missing data")?;
	let patient_information = data
		.get("patientInformation")
		.ok_or("missing patientInformation")?;
	assert!(
		patient_information.get("pastDrugHistory").is_none(),
		"{body}"
	);
	assert!(patient_information.get("patientDeath").is_none(), "{body}");
	assert!(data["patientIdentifiers"].is_array(), "{body}");
	assert!(data["medicalHistoryEpisodes"].is_array(), "{body}");
	assert!(data.get("deathInfo").is_some(), "{body}");
	assert!(data["reportedCauses"].is_array(), "{body}");
	assert!(data["autopsyCauses"].is_array(), "{body}");
	assert!(data.get("parentInfo").is_some(), "{body}");
	assert!(data["parentMedicalHistory"].is_array(), "{body}");
	assert!(data["parentPastDrugs"].is_array(), "{body}");
	assert!(data.get("pastDrugHistory").is_none(), "{body}");
	assert_no_ae_lb_dg_payload(data);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_nr_returns_narrative_payload_only() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-NR").await?;

	let (status, body) =
		get_json(&app, &cookie, &format!("/api/cases/{case_id}/editor/NR")).await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let data = body.get("data").ok_or("missing data")?;
	let narrative = data.get("narrative").ok_or("missing narrative")?;
	assert!(narrative.get("senderDiagnoses").is_none(), "{body}");
	assert!(data["senderDiagnoses"].is_array(), "{body}");
	assert!(data["caseSummaryInformation"].is_array(), "{body}");
	assert_no_ae_lb_dg_payload(data);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_remaining_direct_sections_return_only_their_payloads() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DIRECT-SECTIONS").await?;

	for (section, expected_key) in [
		("RP", "primarySources"),
		("SD", "senderInformation"),
		("LR", "literatureReferences"),
		("SI", "studyInformation"),
	] {
		let (status, body) = get_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/{section}"),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert_eq!(body["caseId"], case_id);
		let data = body.get("data").ok_or("missing data")?;
		assert!(data.get(expected_key).is_some(), "{section}: {body}");
		if section == "SI" {
			assert!(
				data["studyRegistrationNumbers"].is_array(),
				"{section}: {body}"
			);
			assert!(
				data["studyInformation"]
					.get("studyRegistrationNumbers")
					.is_none(),
				"{section}: {body}"
			);
		}
		assert_no_ae_lb_dg_payload(data);
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_list_returns_reaction_rows_without_detail_fanout() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-LIST").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/reactions"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"primary_source_reaction": "Headache",
				"primary_source_reaction_translation": "Head pain",
				"reaction_meddra_version": "27.1",
				"reaction_meddra_code": "10019211",
				"serious": true,
				"outcome": "1"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/AE/list"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let rows = body["rows"].as_array().ok_or("missing rows array")?;
	assert!(!rows.is_empty(), "{body}");
	let row = &rows[0];
	assert!(row.get("id").is_some(), "{row}");
	assert_eq!(row["sequenceNumber"], 1);
	assert_eq!(row["reactionPrimarySourceNative"], "Headache");
	assert_eq!(row["reactionPrimarySourceTranslation"], "Head pain");
	assert_eq!(row["meddraVersion"], "27.1");
	assert_eq!(row["meddraCode"], "10019211");
	assert!(row.get("seriousness").is_some(), "{row}");
	assert!(row.get("outcome").is_none(), "{row}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_repeatable_pages_have_list_projection_routes() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-REPEATABLE-PAGES").await?;

	for (section, expected_key) in [
		("DH", "rows"),
		("AE", "rows"),
		("LB", "rows"),
		("DG", "rows"),
	] {
		let (status, body) = get_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}?authorities=fda"),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert_eq!(body["caseId"], case_id);
		assert_eq!(body["pageId"], section);
		assert!(body.get("appendices").is_none(), "{section}: {body}");
		assert!(body["rows"][expected_key].is_array(), "{section}: {body}");
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_page_projections_do_not_embed_full_validation_issues() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-PROJECTION-CONTRACT",
		&["ich"],
	)
	.await?;

	create_safety_report(&app, &cookie, &case_id, "2", true).await?;
	create_reaction_fixture(&app, &cookie, &case_id).await?;
	create_test_result_fixture(&app, &cookie, &case_id).await?;
	create_drug_fixture(&app, &cookie, &case_id).await?;
	create_past_drug_history_fixture(&app, &cookie, &case_id).await?;

	for page_id in [
		"CI", "RP", "SD", "LR", "SI", "DM", "NR", "AE", "LB", "DG", "DH",
	] {
		let (status, body) = get_json(
			&app,
			&cookie,
			&format!(
				"/api/cases/{case_id}/editor/pages/{page_id}?authorities=ich,fda,mfds"
			),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{page_id}: {body}");
		assert!(
			body["fields"]
				.as_object()
				.map(|fields| fields.is_empty())
				.unwrap_or(false),
			"{page_id} projection must not embed field issue details: {body}"
		);
		assert_eq!(body["requiredCount"], 0, "{page_id}: {body}");
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_page_projection_returns_created_drug_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-PROJECTION").await?;

	let drug_id = create_drug_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG?authorities=fda,mfds"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing DG projection rows")?;
	assert!(
		rows.iter().any(|row| row["id"] == drug_id),
		"DG projection should include the created drug row: {body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_detail_returns_one_reaction_by_uuid() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-DETAIL").await?;

	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/AE/{reaction_id}"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["rowId"], reaction_id);
	let reactions = body["data"]["reactions"]
		.as_array()
		.ok_or("missing reactions array")?;
	assert_eq!(reactions.len(), 1, "{body}");
	assert_eq!(reactions[0]["id"], reaction_id);
	assert!(reactions[0].get("primary_source_reaction").is_some());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_repeatable_page_rows_return_row_detail_by_uuid() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-REPEATABLE-ROWS").await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}?authorities=fda"
		),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["section"], "AE");
	assert_eq!(body["rowId"], reaction_id);
	assert!(body.get("appendices").is_none(), "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_page_row_patch_updates_one_reaction() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-ROW-PATCH").await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Updated reaction"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["section"], "AE");
	assert_eq!(body["rowId"], reaction_id);
	assert_eq!(
		body["data"]["reaction"]["primary_source_reaction"],
		"Updated reaction"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn portable_ae_patch_rejects_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id = create_case(&app, &cookie, "EDITOR-AE-PORTABLE-GATE").await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;
	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let stale_before =
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(stale_before, 0);
	let cache_versions_before =
		validation_summary_row_versions(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}"),
		json!({
			"rows": { "reaction": {
				"reactionPrimarySourceNative": "X".repeat(251)
			} }
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.E.i.1.1a.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"reactions.0.primarySourceReaction"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["message"],
		portable_constraint_message("ICH.E.i.1.1a.LENGTH.MAX")
	);
	assert!(body["error"]["data"]["req_uuid"].is_string());
	let stale_after =
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(stale_after, stale_before);
	let cache_versions_after =
		validation_summary_row_versions(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?;
	assert_eq!(cache_versions_after, cache_versions_before);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["data"]["reaction"]["primary_source_reaction"],
		"Headache"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn portable_direct_rows_patch_rejects_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-NR-PORTABLE-GATE", &["ich"])
			.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"rows": {
				"narrative": { "caseNarrative": "original narrative" }
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"rows": {
				"narrative": { "caseNarrative": "X".repeat(100_001) }
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.H.1.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"narrative.caseNarrative"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["message"],
		portable_constraint_message("ICH.H.1.LENGTH.MAX")
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["narrative"]["case_narrative"],
		"original narrative"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_editor_repeatable_row_save_invalidates_summary_until_live_validation(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-AE-REFRESH", &["ich"]).await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;
	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}"),
		json!({
			"authorities": ["ich"],
			"rows": { "reaction": {
				"reactionPrimarySourceNative": "Headache updated"
			} }
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	assert!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await? > 0
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=ich"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["authority"], "ich", "{body}");
	assert_eq!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?,
		0
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lb_page_row_patch_updates_one_test_result() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-ROW-PATCH").await?;
	let test_result_id = create_test_result_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows/{test_result_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"testResult": {
					"testName": "Updated lab"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["section"], "LB");
	assert_eq!(body["rowId"], test_result_id);
	assert_eq!(body["data"]["testResult"]["test_name"], "Updated lab");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_page_row_patch_updates_one_drug() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-ROW-PATCH").await?;
	let drug_id = create_drug_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{drug_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"drug": {
					"medicinalProduct": "Updated product"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["section"], "DG");
	assert_eq!(body["rowId"], drug_id);
	assert_eq!(body["data"]["drug"]["medicinal_product"], "Updated product");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_page_row_patch_updates_one_drug_history() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-ROW-PATCH").await?;
	let past_drug_id =
		create_past_drug_history_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH/rows/{past_drug_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"pastDrugHistory": {
					"drugName": "Updated prior drug"
				}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["section"], "DH");
	assert_eq!(body["rowId"], past_drug_id);
	assert_eq!(
		body["data"]["pastDrugHistory"]["drug_name"],
		"Updated prior drug"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_page_row_round_trips_all_catalog_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-ALL-FIELDS").await?;
	create_patient_fixture(&app, &cookie, &case_id).await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH/rows"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"pastDrugHistory": {
					"sequenceNumber": 1,
					"drugName": "Prior drug",
					"mfdsMedicinalProductVersion": "2026",
					"mfdsMedicinalProductId": "MFDS-001",
					"mpidVersion": "1",
					"mpid": "MPID-001",
					"phpidVersion": "1",
					"phpid": "PHPID-001",
					"startDate": "20200102",
					"endDate": "20210102",
					"indicationMeddraVersion": "26.0",
					"indicationMeddraCode": "10000001",
					"reactionMeddraVersion": "26.0",
					"reactionMeddraCode": "10000001"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	let row = &created["data"]["pastDrugHistory"];
	assert_eq!(row["drug_name"], "Prior drug");
	assert_eq!(row["mfds_medicinal_product_id"], "MFDS-001");
	assert_eq!(row["mpid"], "MPID-001");
	assert_eq!(row["phpid"], "PHPID-001");
	assert_eq!(row["start_date"], "20200102");
	assert_eq!(row["end_date"], "20210102");
	assert_eq!(row["indication_meddra_code"], "10000001");
	assert_eq!(row["reaction_meddra_code"], "10000001");

	let row_id = created["rowId"].as_str().expect("past drug id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH/rows/{row_id}"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"pastDrugHistory": {
					"drugName": "Updated prior drug"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["data"]["pastDrugHistory"]["drug_name"],
		"Updated prior drug"
	);
	assert_eq!(updated["data"]["pastDrugHistory"]["mpid"], "MPID-001");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH/rows/{row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["data"], updated["data"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-CONSTRAINT").await?;
	create_patient_fixture(&app, &cookie, &case_id).await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH/rows"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"pastDrugHistory": {
					"drugName": "Prior drug",
					"startDate": "not-a-date"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.D.8.r.4.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"patientInformation.pastDrugHistory.0.startDate"
	);

	let (status, list) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DH"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{list}");
	assert_eq!(list["rows"]["rows"].as_array().map(Vec::len), Some(0));

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_page_row_round_trips_supported_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-ALL-FIELDS").await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {
				"reaction": {
					"sequenceNumber": 1,
					"reactionPrimarySourceNative": "Headache",
					"reactionPrimarySourceTranslation": "Head pain",
					"reactionLanguage": "eng",
					"meddraVersion": "26.0",
					"meddraCode": "10000001",
					"termHighlighted": "4",
					"seriousness": {
						"serious": true,
						"criteriaResultsInDeath": true,
						"criteriaLifeThreatening": true,
						"criteriaHospitalization": true,
						"criteriaDisabling": true,
						"criteriaCongenitalAnomaly": true,
						"criteriaOtherMedicallyImportant": true
					},
					"requiredIntervention": true,
					"expectedness": "1",
					"severity": "moderate",
					"reactionStartDate": "20200102",
					"reactionEndDate": "20200103",
					"reactionDuration": 1, "unit": "d",
					"outcome": "1",
					"medicalConfirmation": true,
					"reactionCountry": "KR",
					"mfdsDeviceAe": {
						"aeClassification": "0",
						"aeOutcome": "10",
						"causeMedicalDevice": true,
						"causeProcedureIssue": true,
						"causePatientCondition": true,
						"causeUnableToAssess": true,
						"causeOther": "Other cause",
						"actionReason": "Action reason",
						"actionRecall": true,
						"actionRepair": true,
						"actionInspection": true,
						"actionReplacement": true,
						"actionImprovement": true,
						"actionMonitoring": true,
						"actionNotification": true,
						"actionLabelChange": true,
						"actionOther": "Other action"
					}
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	let row = &created["data"]["reaction"];
	assert_eq!(row["primary_source_reaction"], "Headache");
	assert_eq!(row["reaction_language"], "eng");
	assert_eq!(row["term_highlighted"], "4");
	assert_eq!(row["criteria_death"], true);
	assert_eq!(row["start_date"], "20200102");
	assert_eq!(row["duration_value"], "1.00");
	assert_eq!(row["country_code"], "KR");
	for (field, expected) in [
		("mfds_device_ae_classification", json!("0")),
		("mfds_device_ae_outcome", json!("10")),
		("mfds_device_cause_medical_device", json!(true)),
		("mfds_device_cause_procedure_issue", json!(true)),
		("mfds_device_cause_patient_condition", json!(true)),
		("mfds_device_cause_unable_to_assess", json!(true)),
		("mfds_device_cause_other", json!("Other cause")),
		("mfds_device_action_reason", json!("Action reason")),
		("mfds_device_action_recall", json!(true)),
		("mfds_device_action_repair", json!(true)),
		("mfds_device_action_inspection", json!(true)),
		("mfds_device_action_replacement", json!(true)),
		("mfds_device_action_improvement", json!(true)),
		("mfds_device_action_monitoring", json!(true)),
		("mfds_device_action_notification", json!(true)),
		("mfds_device_action_label_change", json!(true)),
		("mfds_device_action_other", json!("Other action")),
	] {
		assert_eq!(row[field], expected, "{field}: {created}");
	}

	let row_id = created["rowId"].as_str().expect("reaction id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{row_id}"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Updated headache"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["data"]["reaction"]["primary_source_reaction"],
		"Updated headache"
	);
	assert_eq!(updated["data"]["reaction"]["country_code"], "KR");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["data"], updated["data"]);
	assert_eq!(reloaded["data"]["reaction"]["term_highlighted"], "4");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-CONSTRAINT").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Headache",
					"termHighlighted": "9"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.E.i.3.1.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"reactions.0.termHighlighted"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_page_rejects_invalid_fda_required_intervention() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-FDA-CONSTRAINT").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Headache",
					"requiredIntervention": "arbitrary"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"FDA.E.i.3.2h.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"reactions.0.requiredIntervention"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_ae_page_round_trips_fda_required_intervention_representations(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-FDA-ROUNDTRIP").await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Headache",
					"requiredIntervention": true
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	assert_eq!(created["data"]["reaction"]["required_intervention"], "true");
	assert_eq!(
		created["data"]["reaction"]["required_intervention_null_flavor"],
		Value::Null
	);
	let row_id = created["rowId"].as_str().expect("reaction id");

	let (status, ni) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{row_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {"reaction": {"requiredIntervention": "NI"}}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{ni}");
	assert_eq!(ni["data"]["reaction"]["required_intervention"], Value::Null);
	assert_eq!(
		ni["data"]["reaction"]["required_intervention_null_flavor"],
		"NI"
	);

	let (status, truth) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{row_id}"),
		json!({
			"authorities": ["fda"],
			"rows": {"reaction": {"requiredIntervention": true}}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{truth}");
	assert_eq!(truth["data"]["reaction"]["required_intervention"], "true");
	assert_eq!(
		truth["data"]["reaction"]["required_intervention_null_flavor"],
		Value::Null
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lb_page_row_round_trips_all_catalog_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-ALL-FIELDS").await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"testResult": {
					"sequenceNumber": 1,
					"testDate": "20200102",
					"testName": "ALT",
					"testMeddraVersion": "26.0",
					"testMeddraCode": "10000001",
					"testResultCode": "1",
					"testResult": "12.5",
					"testUnit": "mg/dL",
					"testResultUnstructured": "Normal",
					"lowRange": "10",
					"highRange": "20",
					"comments": "Within range",
					"moreInformationAvailable": true
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	let row = &created["data"]["testResult"];
	assert_eq!(row["test_date"], "20200102");
	assert_eq!(row["test_name"], "ALT");
	assert_eq!(row["test_meddra_version"], "26.0");
	assert_eq!(row["test_meddra_code"], "10000001");
	assert_eq!(row["test_result_code"], "1");
	assert_eq!(row["test_result_value"], "12.5");
	assert_eq!(row["test_result_unit"], "mg/dL");
	assert_eq!(row["result_unstructured"], "Normal");
	assert_eq!(row["normal_low_value"], "10");
	assert_eq!(row["normal_high_value"], "20");
	assert_eq!(row["comments"], "Within range");
	assert_eq!(row["more_info_available"], true);

	let row_id = created["rowId"].as_str().expect("test result id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows/{row_id}"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"testResult": {
					"testName": "Updated ALT"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(updated["data"]["testResult"]["test_name"], "Updated ALT");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows/{row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["data"], updated["data"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lb_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-CONSTRAINT").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"testResult": {
					"testName": "ALT",
					"testDate": "not-a-date"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.F.r.1.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"testResults.0.testDate"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_page_row_round_trips_drug_information_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-ALL-DRUG-FIELDS").await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows"),
		json!({
			"authorities": ["ich", "fda", "mfds"],
			"rows": {
				"drug": {
					"sequenceNumber": 1,
					"drugCharacterization": "1",
					"medicinalProduct": "Product A",
					"drugBatchNumber": "LOT-001",
					"drugActionTaken": "1",
					"mpidVersion": "1",
					"mpid": "MPID-001",
					"phpidVersion": "1",
					"phpid": "PHPID-001",
					"mfdsMpidVersion": "2026",
					"mfdsMpid": "MFDS-001",
					"obtainDrugCountry": "KR",
					"investigationalProductBlinded": true,
					"drugAuthorizationNumber": "AUTH-001",
					"drugAuthorizationCountry": "KR",
					"drugAuthorizationHolder": "Holder",
					"cumulativeDoseValue": 12.5,
					"cumulativeDoseUnit": "mg",
					"gestationPeriodExposureValue": 4,
					"gestationPeriodExposureUnit": "wk",
					"drugAdditionalInformation": "Additional information",
					"drugAdditionalInformationCodes": ["1"],
					"fdaAdditionalInfoCoded": "1",
					"fdaSpecializedProductCategory": "1",
					"fdaOtherCharacterization": "1"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	let row = &created["data"]["drug"];
	assert_eq!(row["drug_characterization"], "1");
	assert_eq!(row["medicinal_product"], "Product A");
	assert_eq!(row["batch_lot_number"], "LOT-001");
	assert_eq!(row["action_taken"], "1");
	assert_eq!(row["mpid_version"], "1");
	assert_eq!(row["mpid"], "MPID-001");
	assert_eq!(row["phpid_version"], "1");
	assert_eq!(row["phpid"], "PHPID-001");
	assert_eq!(row["mfds_mpid_version"], "2026");
	assert_eq!(row["mfds_mpid"], "MFDS-001");
	assert_eq!(row["obtain_drug_country"], "KR");
	assert_eq!(row["investigational_product_blinded"], true);
	assert_eq!(row["drug_authorization_number"], "AUTH-001");
	assert_eq!(row["manufacturer_country"], "KR");
	assert_eq!(row["manufacturer_name"], "Holder");
	assert_eq!(row["cumulative_dose_first_reaction_value"], "12.50000");
	assert_eq!(row["cumulative_dose_first_reaction_unit"], "mg");
	assert_eq!(row["gestation_period_exposure_value"], "4.00");
	assert_eq!(row["gestation_period_exposure_unit"], "wk");
	assert_eq!(row["drug_additional_information"], "Additional information");
	assert_eq!(row["drug_additional_info_codes_json"], json!(["1"]));
	assert_eq!(row["fda_additional_info_coded"], "1");
	assert_eq!(row["fda_specialized_product_category"], "1");
	assert_eq!(row["fda_other_characterization"], "1");

	let row_id = created["rowId"].as_str().expect("drug id");
	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{row_id}"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"drug": {
					"medicinalProduct": "Updated product"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["data"]["drug"]["medicinal_product"],
		"Updated product"
	);

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["data"], updated["data"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-CONSTRAINT").await?;

	let cases = [
		(
			json!({"medicinalProduct": "X".repeat(2001)}),
			"ICH.G.k.2.2.LENGTH.MAX",
			"drugs.0.medicinalProduct",
		),
		(
			json!({"activeSubstances": [{"substanceName": "X".repeat(251)}]}),
			"ICH.G.k.2.3.r.1.LENGTH.MAX",
			"drugs.0.activeSubstances.0.substanceName",
		),
		(
			json!({"dosageInformation": [{"dosageText": "X".repeat(2001)}]}),
			"ICH.G.k.4.r.8.LENGTH.MAX",
			"drugs.0.dosageInformation.0.dosageText",
		),
		(
			json!({"indications": [{"indicationText": "X".repeat(251)}]}),
			"ICH.G.k.7.r.1.LENGTH.MAX",
			"drugs.0.indications.0.indicationText",
		),
		(
			json!({"drugReactionAssessments": [{"sourceOfAssessment": "X".repeat(61)}]}),
			"ICH.G.k.9.i.2.r.1.LENGTH.MAX",
			"drugs.0.drugReactionAssessments.0.sourceOfAssessment",
		),
	];
	for (invalid_fields, rule_code, path) in cases {
		let mut drug = invalid_fields.as_object().expect("object").clone();
		drug.insert("drugCharacterization".to_string(), json!("1"));
		let (status, body) = post_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/DG/rows"),
			json!({
				"authorities": ["ich"],
				"rows": {"drug": drug}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
		assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
		assert_eq!(body["error"]["data"]["detail"]["ruleCode"], rule_code);
		assert_eq!(body["error"]["data"]["detail"]["path"], path);
	}

	let (status, projection) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{projection}");
	assert_eq!(
		projection["rows"]["rows"].as_array().map(Vec::len),
		Some(0),
		"constraint failures must happen before the parent drug is written"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_page_round_trips_nested_active_substances() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-ACTIVE-SUBSTANCE").await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"drug": {
					"drugCharacterization": "1",
					"medicinalProduct": "Product A",
					"activeSubstances": [{
						"sequenceNumber": 1,
						"substanceName": "Substance A",
						"substanceTermIdVersion": "1",
						"substanceTermId": "SUB-1",
						"mfdsVersion": "2026",
						"mfdsId": "KR-SUB-1",
						"substanceStrengthValue": 10.5,
						"substanceStrengthUnit": "mg"
					}],
					"dosageInformation": [{
						"sequenceNumber": 1,
						"doseValue": 2.5,
						"doseUnit": "mg",
						"numberOfUnits": 1,
						"frequencyUnit": "d",
						"firstAdministrationDate": "20200101",
						"lastAdministrationDate": "20200102",
						"durationValue": 2,
						"durationUnit": "d",
						"continuing": false,
						"batchNumber": "LOT-1",
						"dosageText": "One tablet daily",
						"doseForm": "Tablet",
						"doseFormTermIdVersion": "1",
						"doseFormTermId": "DF-1",
						"routeOfAdministration": "048",
						"routeTermIdVersion": "1",
						"routeTermId": "ROUTE-1",
						"parentRouteOfAdministration": "Oral",
						"parentRouteTermIdVersion": "1",
						"parentRouteTermId": "PROUTE-1"
					}],
					"indications": [{
						"sequenceNumber": 1,
						"indicationText": "Pain",
						"indicationMeddraVersion": "26.0",
						"indicationMeddraCode": "10033371"
					}],
					"drugReactionAssessments": [{
						"reactionId": reaction_id,
						"administrationStartIntervalValue": 2,
						"administrationStartIntervalUnit": "d",
						"lastDoseIntervalValue": 1,
						"lastDoseIntervalUnit": "d",
						"recurrenceAction": "1",
						"reactionRecurred": "2",
						"sourceOfAssessment": "Reporter",
						"methodOfAssessment": "WHO-UMC",
						"resultOfAssessment": "Possible",
						"resultOfAssessmentKr2": "1"
					}]
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	let substance = &created["data"]["drug"]["activeSubstances"][0];
	assert_eq!(substance["substance_name"], "Substance A");
	assert_eq!(substance["substance_termid_version"], "1");
	assert_eq!(substance["substance_termid"], "SUB-1");
	assert_eq!(substance["mfds_version"], "2026");
	assert_eq!(substance["mfds_id"], "KR-SUB-1");
	assert_eq!(substance["strength_value"], "10.50000");
	assert_eq!(substance["strength_unit"], "mg");
	let dosage = &created["data"]["drug"]["dosageInformation"][0];
	assert_eq!(dosage["dose_value"], "2.50000");
	assert_eq!(dosage["dose_unit"], "mg");
	assert_eq!(dosage["number_of_units"], "1.00000");
	assert_eq!(dosage["frequency_unit"], "d");
	assert_eq!(dosage["first_administration_date"], "20200101");
	assert_eq!(dosage["last_administration_date"], "20200102");
	assert_eq!(dosage["duration_value"], "2.00");
	assert_eq!(dosage["duration_unit"], "d");
	assert_eq!(dosage["continuing"], false);
	assert_eq!(dosage["batch_lot_number"], "LOT-1");
	assert_eq!(dosage["dosage_text"], "One tablet daily");
	assert_eq!(dosage["dose_form"], "Tablet");
	assert_eq!(dosage["dose_form_termid_version"], "1");
	assert_eq!(dosage["dose_form_termid"], "DF-1");
	assert_eq!(dosage["route_of_administration"], "048");
	assert_eq!(dosage["route_termid_version"], "1");
	assert_eq!(dosage["route_termid"], "ROUTE-1");
	assert_eq!(dosage["parent_route"], "Oral");
	assert_eq!(dosage["parent_route_termid_version"], "1");
	assert_eq!(dosage["parent_route_termid"], "PROUTE-1");
	let indication = &created["data"]["drug"]["indications"][0];
	assert_eq!(indication["indication_text"], "Pain");
	assert_eq!(indication["indication_meddra_version"], "26.0");
	assert_eq!(indication["indication_meddra_code"], "10033371");
	let assessment = &created["data"]["drug"]["drugReactionAssessments"][0];
	assert_eq!(assessment["reactionId"], reaction_id);
	assert_eq!(assessment["administrationStartIntervalValue"], "2.00");
	assert_eq!(assessment["administrationStartIntervalUnit"], "d");
	assert_eq!(assessment["lastDoseIntervalValue"], "1.00");
	assert_eq!(assessment["lastDoseIntervalUnit"], "d");
	assert_eq!(assessment["recurrenceAction"], "1");
	assert_eq!(assessment["reactionRecurred"], "2");
	assert_eq!(assessment["sourceOfAssessment"], "Reporter");
	assert_eq!(assessment["methodOfAssessment"], "WHO-UMC");
	assert_eq!(assessment["resultOfAssessment"], "Possible");
	assert_eq!(assessment["resultOfAssessmentKr2"], "1");
	let drug_id = created["rowId"].as_str().expect("drug id");
	let substance_id = substance["id"].as_str().expect("substance id");
	let dosage_id = dosage["id"].as_str().expect("dosage id");
	let indication_id = indication["id"].as_str().expect("indication id");
	let relatedness_id = assessment["id"].as_str().expect("relatedness id");
	let assessment_id = assessment["drugReactionAssessmentId"]
		.as_str()
		.expect("assessment id");

	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{drug_id}"),
		json!({
			"authorities": ["ich", "mfds"],
			"rows": {
				"drug": {
					"activeSubstances": [{
						"id": substance_id,
						"substanceName": "Updated substance"
					}],
					"dosageInformation": [{
						"id": dosage_id,
						"dosageText": "Updated dosage"
					}],
					"indications": [{
						"id": indication_id,
						"indicationText": "Updated pain"
					}],
					"drugReactionAssessments": [{
						"id": relatedness_id,
						"drugReactionAssessmentId": assessment_id,
						"reactionId": reaction_id,
						"administrationStartIntervalValue": 3,
						"methodOfAssessment": "Updated method"
					}]
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["data"]["drug"]["activeSubstances"][0]["substance_name"],
		"Updated substance"
	);
	assert_eq!(
		updated["data"]["drug"]["dosageInformation"][0]["dosage_text"],
		"Updated dosage"
	);
	assert_eq!(
		updated["data"]["drug"]["indications"][0]["indication_text"],
		"Updated pain"
	);
	assert_eq!(
		updated["data"]["drug"]["drugReactionAssessments"][0]
			["administrationStartIntervalValue"],
		"3.00"
	);
	assert_eq!(
		updated["data"]["drug"]["drugReactionAssessments"][0]["methodOfAssessment"],
		"Updated method"
	);

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{drug_id}"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"drug": {
					"activeSubstances": [{
						"id": substance_id,
						"_delete": true
					}],
					"dosageInformation": [{
						"id": dosage_id,
						"_delete": true
					}],
					"indications": [{
						"id": indication_id,
						"_delete": true
					}]
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["data"]["drug"]["activeSubstances"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);
	assert_eq!(
		deleted["data"]["drug"]["dosageInformation"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);
	assert_eq!(
		deleted["data"]["drug"]["indications"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_nr_page_round_trips_narrative_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-NR-ALL-FIELDS").await?;

	let (status, created) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"narrative": {
					"caseNarrative": "Case narrative",
					"reporterComments": "Reporter comments",
					"senderComments": "Sender comments",
					"additionalInformation": "Additional information"
				},
				"senderDiagnoses": [{
					"sequenceNumber": 1,
					"diagnosisMeddraVersion": "27.1",
					"diagnosisMeddraCode": "10019211"
				}],
				"caseSummaryInformation": [{
					"sequenceNumber": 1,
					"summaryType": "1",
					"languageCode": "en",
					"summaryText": "Case summary"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{created}");
	let narrative = &created["rows"]["narrative"];
	assert_eq!(narrative["case_narrative"], "Case narrative");
	assert_eq!(narrative["reporter_comments"], "Reporter comments");
	assert_eq!(narrative["sender_comments"], "Sender comments");
	assert_eq!(
		narrative["additional_information"],
		"Additional information"
	);
	let diagnosis = &created["rows"]["senderDiagnoses"][0];
	assert_eq!(diagnosis["diagnosis_meddra_version"], "27.1");
	assert_eq!(diagnosis["diagnosis_meddra_code"], "10019211");
	let summary = &created["rows"]["caseSummaryInformation"][0];
	assert_eq!(summary["summary_type"], "1");
	assert_eq!(summary["language_code"], "en");
	assert_eq!(summary["summary_text"], "Case summary");
	let diagnosis_id = diagnosis["id"].as_str().expect("diagnosis id");
	let summary_id = summary["id"].as_str().expect("summary id");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["rows"]["narrative"], created["rows"]["narrative"]);

	let (status, updated) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"narrative": {
					"caseNarrative": "Updated narrative"
				},
				"senderDiagnoses": [{
					"id": diagnosis_id,
					"diagnosisMeddraCode": "10000001"
				}],
				"caseSummaryInformation": [{
					"id": summary_id,
					"summaryText": "Updated summary"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(
		updated["rows"]["narrative"]["case_narrative"],
		"Updated narrative"
	);
	assert_eq!(
		updated["rows"]["senderDiagnoses"][0]["diagnosis_meddra_code"],
		"10000001"
	);
	assert_eq!(
		updated["rows"]["caseSummaryInformation"][0]["summary_text"],
		"Updated summary"
	);

	let (status, deleted) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"senderDiagnoses": [{"id": diagnosis_id, "_delete": true}],
				"caseSummaryInformation": [{"id": summary_id, "_delete": true}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{deleted}");
	assert_eq!(
		deleted["rows"]["senderDiagnoses"].as_array().map(Vec::len),
		Some(0)
	);
	assert_eq!(
		deleted["rows"]["caseSummaryInformation"]
			.as_array()
			.map(Vec::len),
		Some(0)
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_nr_page_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-NR-CONSTRAINT").await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"narrative": {
					"caseNarrative": "Case narrative",
					"reporterComments": "X".repeat(20001)
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.H.2.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"narrative.reporterComments"
	);

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/NR"),
		json!({
			"authorities": ["ich"],
			"rows": {
				"narrative": {"caseNarrative": "Case narrative"},
				"senderDiagnoses": [{
					"diagnosisMeddraVersion": "12345"
				}]
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.H.3.r.1a.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"narrative.senderDiagnoses.0.diagnosisMeddraVersion"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn message_header_api_round_trips_submission_fields() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "MESSAGE-HEADER-ROUNDTRIP").await?;
	let message_number = format!("MSG-{}", Uuid::new_v4());

	let (status, created) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_number": message_number,
				"message_sender_identifier": "SENDER",
				"message_receiver_identifier": "RECEIVER",
				"message_date": "20260725120000"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{created}");
	assert_eq!(created["data"]["message_number"], message_number);
	assert_eq!(created["data"]["message_sender_identifier"], "SENDER");
	assert_eq!(created["data"]["message_receiver_identifier"], "RECEIVER");
	assert_eq!(created["data"]["message_date"], "20260725120000");

	let (status, updated) = put_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"batch_number": "BATCH-001",
				"batch_sender_identifier": "BATCH-SENDER",
				"batch_receiver_identifier": "BATCH-RECEIVER",
				"batch_transmission_date": [2026, 206, 12, 30, 45, 0, 0, 0, 0],
				"message_sender_identifier": "UPDATED-SENDER",
				"message_receiver_identifier": "UPDATED-RECEIVER",
				"message_date": "20260725123045"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{updated}");
	assert_eq!(updated["data"]["batch_number"], "BATCH-001");
	assert_eq!(updated["data"]["batch_sender_identifier"], "BATCH-SENDER");
	assert_eq!(
		updated["data"]["batch_receiver_identifier"],
		"BATCH-RECEIVER"
	);
	assert_eq!(
		updated["data"]["batch_transmission_date"],
		json!([2026, 206, 12, 30, 45, 0, 0, 0, 0])
	);
	assert_eq!(
		updated["data"]["message_sender_identifier"],
		"UPDATED-SENDER"
	);
	assert_eq!(
		updated["data"]["message_receiver_identifier"],
		"UPDATED-RECEIVER"
	);
	assert_eq!(updated["data"]["message_date"], "20260725123045");

	let (status, reloaded) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{reloaded}");
	assert_eq!(reloaded["data"], updated["data"]);

	Ok(())
}

#[serial]
#[tokio::test]
async fn message_header_api_rejects_catalog_constraint_before_write() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "MESSAGE-HEADER-CONSTRAINT").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_number": "X".repeat(101),
				"message_sender_identifier": "SENDER",
				"message_receiver_identifier": "RECEIVER",
				"message_date": "20260725120000"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(body["error"]["message"], "CONSTRAINT_VIOLATION");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.N.2.r.1.LENGTH.MAX"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"messageHeader.messageNumber"
	);

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/message-header"),
		json!({
			"data": {
				"case_id": case_id,
				"message_number": format!("MSG-{}", Uuid::new_v4()),
				"message_sender_identifier": "SENDER",
				"message_receiver_identifier": "RECEIVER",
				"message_date": "not-a-date"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY, "{body}");
	assert_eq!(
		body["error"]["data"]["detail"]["ruleCode"],
		"ICH.N.2.r.4.ALLOWED.VALUE"
	);
	assert_eq!(
		body["error"]["data"]["detail"]["path"],
		"messageHeader.messageDate"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_repeatable_page_rows_accept_rows_with_profiles() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-ROW-DELTAS").await?;
	let reaction_id = create_reaction_fixture(&app, &cookie, &case_id).await?;
	let test_result_id = create_test_result_fixture(&app, &cookie, &case_id).await?;
	let drug_id = create_drug_fixture(&app, &cookie, &case_id).await?;
	let past_drug_id =
		create_past_drug_history_fixture(&app, &cookie, &case_id).await?;

	let requests = [
		(
			"AE",
			reaction_id,
			"reaction",
			json!({ "reactionPrimarySourceNative": "Headache" }),
		),
		(
			"LB",
			test_result_id,
			"testResult",
			json!({ "resultValue": "10" }),
		),
		(
			"DG",
			drug_id,
			"drug",
			json!({ "medicinalProduct": "Drug A" }),
		),
		(
			"DH",
			past_drug_id,
			"pastDrugHistory",
			json!({ "drugName": "Past Drug" }),
		),
	];

	for (section, row_id, row_key, row) in requests {
		let mut rows = serde_json::Map::new();
		rows.insert(row_key.to_string(), row);
		let (status, body) = patch_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}/rows/{row_id}"),
			json!({
				"authorities": ["fda"],
				"rows": rows
			}),
		)
		.await?;

		assert_eq!(status, StatusCode::OK, "{section}: {body}");
		assert_eq!(body["authorities"], json!(["fda"]), "{section}: {body}");
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_repeatable_page_row_create_and_delete_routes_work_for_all_sections(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case(&app, &cookie, "EDITOR-REPEATABLE-ROW-CREATE-DELETE").await?;
	create_patient_fixture(&app, &cookie, &case_id).await?;

	let create_requests = [
		(
			"AE",
			json!({
				"authorities": ["fda"],
				"rows": {
					"reaction": {
						"reactionPrimarySourceNative": "Created reaction"
					}
				}
			}),
			"reaction",
		),
		(
			"LB",
			json!({
				"authorities": ["fda"],
				"rows": {
					"testResult": {
						"testName": "Created lab"
					}
				}
			}),
			"testResult",
		),
		(
			"DG",
			json!({
				"authorities": ["fda"],
				"rows": {
					"drug": {
						"medicinalProduct": "Created product"
					}
				}
			}),
			"drug",
		),
		(
			"DH",
			json!({
				"authorities": ["fda"],
				"rows": {
					"pastDrugHistory": {
						"drugName": "Created prior drug"
					}
				}
			}),
			"pastDrugHistory",
		),
	];

	for (section, request, response_key) in create_requests {
		let (status, body) = post_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}/rows"),
			request,
		)
		.await?;
		assert_eq!(status, StatusCode::CREATED, "{section}: {body}");
		assert_eq!(body["section"], section);
		assert!(body["data"][response_key].is_object(), "{section}: {body}");
		let row_id = body["rowId"]
			.as_str()
			.ok_or("missing created page row id")?;

		let (status, body) = delete_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/editor/pages/{section}/rows/{row_id}"),
		)
		.await?;
		assert_eq!(status, StatusCode::NO_CONTENT, "{section}: {body}");
	}

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_editor_ae_soft_delete_returns_deleted_row_when_requested() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-AE-SOFT-DELETE").await?;

	let mut created_ids = Vec::new();
	for (sequence_number, label) in
		[(1, "Soft-deleted reaction"), (2, "Active reaction")]
	{
		let (status, body) = post_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/reactions"),
			json!({
				"data": {
					"case_id": case_id,
					"sequence_number": sequence_number,
					"primary_source_reaction": label
				}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::CREATED, "{body}");
		created_ids.push(
			body["data"]["id"]
				.as_str()
				.ok_or("missing created reaction id")?
				.to_string(),
		);
	}
	let deleted_row_id = created_ids
		.first()
		.ok_or("missing deleted reaction id")?
		.to_string();

	let (status, body) = delete_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{deleted_row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing active AE rows array")?;
	assert!(
		!active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active AE projection should exclude soft-deleted row: {body}"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/AE?authorities=fda&include_deleted=true"
		),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing include-deleted AE rows array")?;
	assert!(
		rows.iter().any(|row| {
			row["id"].as_str() == Some(deleted_row_id.as_str())
				&& row["deleted"].as_bool() == Some(true)
		}),
		"include_deleted AE projection should include deleted row with deleted=true: {body}"
	);

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/AE/rows/{deleted_row_id}/restore"
		),
		json!({}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["reaction"]["deleted"].as_bool(), Some(false));

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing restored active AE rows array")?;
	assert!(
		active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active AE projection should include restored row: {body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_editor_lb_soft_delete_returns_deleted_row_when_requested() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-SOFT-DELETE").await?;

	let mut created_ids = Vec::new();
	for (sequence_number, label) in [(1, "Soft-deleted lab"), (2, "Active lab")] {
		let (status, body) = post_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/test-results"),
			json!({
				"data": {
					"case_id": case_id,
					"sequence_number": sequence_number,
					"test_name": label,
					"test_result_value": "42",
					"test_result_unit": "U/L"
				}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::CREATED, "{body}");
		created_ids.push(
			body["data"]["id"]
				.as_str()
				.ok_or("missing created test result id")?
				.to_string(),
		);
	}
	let deleted_row_id = created_ids
		.first()
		.ok_or("missing deleted test result id")?
		.to_string();

	let (status, body) = delete_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB/rows/{deleted_row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing active LB rows array")?;
	assert!(
		!active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active LB projection should exclude soft-deleted row: {body}"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/LB?authorities=fda&include_deleted=true"
		),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing include-deleted LB rows array")?;
	assert!(
		rows.iter().any(|row| {
			row["id"].as_str() == Some(deleted_row_id.as_str())
				&& row["deleted"].as_bool() == Some(true)
		}),
		"include_deleted LB projection should include deleted row with deleted=true: {body}"
	);

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/LB/rows/{deleted_row_id}/restore"
		),
		json!({}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["testResult"]["deleted"].as_bool(), Some(false));

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/LB?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing restored active LB rows array")?;
	assert!(
		active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active LB projection should include restored row: {body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn test_editor_dg_soft_delete_returns_deleted_row_when_requested() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-SOFT-DELETE").await?;

	let mut created_ids = Vec::new();
	for (sequence_number, label) in
		[(1, "Soft-deleted product"), (2, "Active product")]
	{
		let (status, body) = post_json(
			&app,
			&cookie,
			&format!("/api/cases/{case_id}/drugs"),
			json!({
				"data": {
					"case_id": case_id,
					"sequence_number": sequence_number,
					"drug_characterization": "1",
					"medicinal_product": label,
					"action_taken": "1"
				}
			}),
		)
		.await?;
		assert_eq!(status, StatusCode::CREATED, "{body}");
		created_ids.push(
			body["data"]["id"]
				.as_str()
				.ok_or("missing created drug id")?
				.to_string(),
		);
	}
	let deleted_row_id = created_ids
		.first()
		.ok_or("missing deleted drug id")?
		.to_string();

	let (status, body) = delete_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG/rows/{deleted_row_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::NO_CONTENT, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing active DG rows array")?;
	assert!(
		!active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active DG projection should exclude soft-deleted row: {body}"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/DG?authorities=fda&include_deleted=true"
		),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing include-deleted DG rows array")?;
	assert!(
		rows.iter().any(|row| {
			row["id"].as_str() == Some(deleted_row_id.as_str())
				&& row["deleted"].as_bool() == Some(true)
		}),
		"include_deleted DG projection should include deleted row with deleted=true: {body}"
	);

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!(
			"/api/cases/{case_id}/editor/pages/DG/rows/{deleted_row_id}/restore"
		),
		json!({}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["drug"]["deleted"].as_bool(), Some(false));

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DG?authorities=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let active_rows = body["rows"]["rows"]
		.as_array()
		.ok_or("missing restored active DG rows array")?;
	assert!(
		active_rows
			.iter()
			.any(|row| row["id"].as_str() == Some(deleted_row_id.as_str())),
		"active DG projection should include restored row: {body}"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_repeatable_page_row_create_and_delete_mark_validation_cache_stale(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());
	let case_id = create_case(&app, &cookie, "EDITOR-ROW-STALE").await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?,
		0
	);

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows"),
		json!({
			"authorities": ["fda"],
			"rows": {
				"reaction": {
					"reactionPrimarySourceNative": "Created stale reaction"
				}
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let reaction_id = body["rowId"]
		.as_str()
		.ok_or("missing created reaction row id")?
		.to_string();
	assert!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await? > 0,
		"row create should mark cached validation summaries stale"
	);

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/validation?authority=fda"),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await?,
		0
	);

	let (status, body) = delete_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/AE/rows/{reaction_id}"),
	)
	.await?;
	assert_eq!(status, StatusCode::NO_CONTENT, "{body}");
	assert!(
		stale_validation_summary_count(&mm, seed.admin.id, seed.org_id, &case_id)
			.await? > 0,
		"row delete should mark cached validation summaries stale"
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lb_list_returns_test_rows_without_detail_fanout() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-LIST").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/test-results"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"test_name": "ALT",
				"test_result_value": "42",
				"test_result_unit": "U/L"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/LB/list"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let rows = body["rows"].as_array().ok_or("missing rows array")?;
	assert!(!rows.is_empty(), "{body}");
	let row = &rows[0];
	assert!(row.get("id").is_some(), "{row}");
	assert_eq!(row["sequenceNumber"], 1);
	assert_eq!(row["testName"], "ALT");
	assert_eq!(row["resultValue"], "42");
	assert_eq!(row["resultUnit"], "U/L");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_lb_detail_returns_one_test_result_by_uuid() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-LB-DETAIL").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/test-results"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"test_name": "ALT",
				"test_result_value": "42",
				"test_result_unit": "U/L"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let test_result_id = body["data"]["id"]
		.as_str()
		.ok_or("missing test result id")?
		.to_string();

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/LB/{test_result_id}"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["rowId"], test_result_id);
	let test_results = body["data"]["testResults"]
		.as_array()
		.ok_or("missing testResults array")?;
	assert_eq!(test_results.len(), 1, "{body}");
	assert_eq!(test_results[0]["id"], test_result_id);
	assert_eq!(test_results[0]["test_name"], "ALT");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_list_returns_drug_rows_without_nested_drug_children() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-LIST").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/drugs"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"drug_characterization": "1",
				"medicinal_product": "Example Product",
				"action_taken": "1"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/DG/list"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let rows = body["rows"].as_array().ok_or("missing rows array")?;
	assert!(!rows.is_empty(), "{body}");
	let row = &rows[0];
	assert!(row.get("id").is_some(), "{row}");
	assert_eq!(row["sequenceNumber"], 1);
	assert_eq!(row["drugRole"], "1");
	assert_eq!(row["medicinalProduct"], "Example Product");
	assert_eq!(row["actionTaken"], "1");
	assert!(row.get("dosageInformation").is_none(), "{row}");
	assert!(row.get("drugReactionAssessments").is_none(), "{row}");
	assert!(row.get("activeSubstances").is_none(), "{row}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dg_detail_returns_one_drug_with_nested_children() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DG-DETAIL").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/drugs"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"drug_characterization": "1",
				"medicinal_product": "Example Product",
				"action_taken": "1"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let drug_id = body["data"]["id"]
		.as_str()
		.ok_or("missing drug id")?
		.to_string();

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/drugs/{drug_id}/dosages"),
		json!({
			"data": {
				"drug_id": drug_id,
				"sequence_number": 1,
				"dose_value": 10,
				"dose_unit": "mg",
				"dosage_text": "10 mg daily"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/reactions"),
		json!({
			"data": {
				"case_id": case_id,
				"sequence_number": 1,
				"primary_source_reaction": "Headache"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let reaction_id = body["data"]["id"]
		.as_str()
		.ok_or("missing reaction id")?
		.to_string();

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments"),
		json!({
			"data": {
				"drug_id": drug_id,
				"reaction_id": reaction_id
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/DG/{drug_id}"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["rowId"], drug_id);
	let drugs = body["data"]["drugs"]
		.as_array()
		.ok_or("missing drugs array")?;
	assert_eq!(drugs.len(), 1, "{body}");
	let drug = &drugs[0];
	assert_eq!(drug["id"], drug_id);
	assert!(!drug["dosageInformation"]
		.as_array()
		.ok_or("missing dosageInformation array")?
		.is_empty());
	assert!(!drug["drugReactionAssessments"]
		.as_array()
		.ok_or("missing drugReactionAssessments array")?
		.is_empty());

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_row_detail_rejects_numeric_row_position_as_identifier() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-NUMERIC-DETAIL").await?;

	let (status, body) =
		get_json(&app, &cookie, &format!("/api/cases/{case_id}/editor/AE/1"))
			.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_list_returns_past_drug_rows() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-LIST").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "ABC"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let patient_id = body["data"]["id"].as_str().ok_or("missing patient id")?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient/past-drugs"),
		json!({
			"data": {
				"patient_id": patient_id,
				"sequence_number": 1,
				"drug_name": "Prior Drug",
				"indication_meddra_code": "10012345",
				"start_date": "2024-01-02",
				"end_date": "2024-02-03"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/DH/list"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let rows = body["rows"].as_array().ok_or("missing rows array")?;
	assert!(!rows.is_empty(), "{body}");
	let row = &rows[0];
	assert!(row.get("id").is_some(), "{row}");
	assert_eq!(row["sequenceNumber"], 1);
	assert_eq!(row["drugName"], "Prior Drug");
	assert_eq!(row["indication"], "10012345");
	assert_eq!(row["startDate"], "2024-01-02");
	assert_eq!(row["endDate"], "2024-02-03");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_detail_returns_one_past_drug_history_by_uuid() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-DETAIL").await?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient"),
		json!({
			"data": {
				"case_id": case_id,
				"patient_initials": "ABC"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let patient_id = body["data"]["id"].as_str().ok_or("missing patient id")?;

	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient/past-drugs"),
		json!({
			"data": {
				"patient_id": patient_id,
				"sequence_number": 1,
				"drug_name": "Prior Drug",
				"indication_meddra_code": "10012345"
			}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let past_drug_id = body["data"]["id"]
		.as_str()
		.ok_or("missing past drug id")?
		.to_string();

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/DH/{past_drug_id}"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	assert_eq!(body["rowId"], past_drug_id);
	let past_drug_history = body["data"]["patientInformation"]["pastDrugHistory"]
		.as_array()
		.ok_or("missing patientInformation.pastDrugHistory array")?;
	assert_eq!(past_drug_history.len(), 1, "{body}");
	assert_eq!(past_drug_history[0]["id"], past_drug_id);
	assert_eq!(past_drug_history[0]["drug_name"], "Prior Drug");

	Ok(())
}

#[serial]
#[tokio::test]
async fn editor_dh_list_returns_empty_rows_when_patient_missing() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case(&app, &cookie, "EDITOR-DH-MISSING-PATIENT").await?;

	let (status, body) = get_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/DH/list"),
	)
	.await?;

	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["caseId"], case_id);
	let rows = body["rows"].as_array().ok_or("missing rows array")?;
	assert!(rows.is_empty(), "{body}");

	Ok(())
}
