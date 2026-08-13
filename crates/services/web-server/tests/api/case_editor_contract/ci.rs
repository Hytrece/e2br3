use super::support::field_contract_test;
const PAGE_ID: &str = "CI";
include!("generated/ci_fields.rs");

#[serial_test::serial]
#[tokio::test]
async fn fulfil_expedited_criteria_null_flavor_round_trips(
) -> crate::common::Result<()> {
	use super::support::{create_case_for_editor, get_json, patch_json};
	use crate::common::{cookie_header, init_test_mm, seed_org_with_users};
	use axum::http::StatusCode;
	use lib_auth::token::generate_web_token;
	use serde_json::json;

	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id =
		create_case_for_editor(&app, &cookie, "EDITOR-CI-NF", &["ich"]).await?;
	let uri = format!("/api/cases/{case_id}/editor/pages/CI");
	let (status, body) = patch_json(
		&app,
		&cookie,
		&uri,
		json!({
			"authorities": ["ich"],
			"rows": {"safetyReportIdentification": {
				"fulfilExpeditedCriteriaNullFlavor": "NI"
			}}
		}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = get_json(&app, &cookie, &uri).await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	assert_eq!(
		body["rows"]["safetyReportIdentification"]
			["fulfilExpeditedCriteriaNullFlavor"],
		"NI"
	);
	Ok(())
}
