use super::support::field_contract_test;
const PAGE_ID: &str = "RP";
include!("generated/rp_fields.rs");

#[serial_test::serial]
#[tokio::test]
async fn reporter_title_can_be_cleared_without_changing_siblings(
) -> crate::common::Result<()> {
	use super::support::{create_case_for_editor, get_json, patch_json};
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
		create_case_for_editor(&app, &cookie, "EDITOR-RP-CLEAR", &["ich"]).await?;
	let uri = format!("/api/cases/{case_id}/editor/pages/RP");

	let (status, body) = patch_json(
		&app,
		&cookie,
		&uri,
		json!({"authorities": ["ich"], "rows": {"primarySources": [{
			"reporterTitle": "Dr",
			"reporterGivenName": "Sibling"
		}]}}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let source_id = body["rows"]["primarySources"][0]["id"]
		.as_str()
		.ok_or("missing primary source id")?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&uri,
		json!({"authorities": ["ich"], "rows": {"primarySources": [{
			"id": source_id,
			"reporterTitle": null
		}]}}),
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{body}");

	let (status, body) = get_json(&app, &cookie, &uri).await?;
	assert_eq!(status, StatusCode::OK, "{body}");
	let source = &body["rows"]["primarySources"][0];
	assert_eq!(source["reporterTitle"], Value::Null);
	assert_eq!(source["reporterGivenName"], "Sibling");
	Ok(())
}
