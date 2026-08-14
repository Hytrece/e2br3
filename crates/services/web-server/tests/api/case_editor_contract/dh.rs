use super::support::field_contract_test;
const PAGE_ID: &str = "DH";
include!("generated/dh_fields.rs");

#[serial_test::serial]
#[tokio::test]
async fn drug_name_can_be_cleared_and_reloaded() -> crate::common::Result<()> {
	use super::support::{create_case_for_editor, get_json, patch_json, post_json};
	use crate::common::{cookie_header, init_test_mm, seed_org_with_users};
	use axum::http::StatusCode;
	use lib_auth::token::generate_web_token;
	use serde_json::{json, Value};

	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-DH-CLEAR", &["ich"]).await?;
	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient"),
		json!({"data": {"case_id": case_id, "patient_initials": "FIXTURE"}}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let patient_id = body["data"]["id"].as_str().ok_or("missing patient id")?;
	let (status, body) = post_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/patient/past-drugs"),
		json!({"data": {
			"patient_id": patient_id,
			"sequence_number": 1,
			"drug_name": "Clearable prior drug"
		}}),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{body}");
	let row_id = body["data"]["id"].as_str().ok_or("missing past drug id")?;
	let uri = format!("/api/cases/{case_id}/editor/pages/DH/rows/{row_id}");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&uri,
		json!({"authorities": ["ich"], "rows": {
			"pastDrugHistory": {"drugName": null}
		}}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = get_json(&app, &cookie, &uri).await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(body["data"]["pastDrugHistory"]["drug_name"], Value::Null);
	Ok(())
}
