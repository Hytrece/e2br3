use crate::common::{
	cookie_header, init_test_mm, insert_user_organization_membership,
	seed_two_orgs_users_cases, Result,
};
use axum::body::{to_bytes, Body};
use axum::http::{header, Request, StatusCode};
use lib_auth::token::generate_web_token;
use serde_json::{json, Value};
use serial_test::serial;
use tower::ServiceExt;

async fn request_json(
	app: &axum::Router,
	method: &str,
	cookie: &str,
	uri: &str,
	body: Option<Value>,
) -> Result<(StatusCode, Value)> {
	let mut req = Request::builder().method(method).uri(uri);
	if !cookie.is_empty() {
		req = req.header("cookie", cookie);
	}
	if body.is_some() {
		req = req.header("content-type", "application/json");
	}
	let res = app
		.clone()
		.oneshot(req.body(match body {
			Some(body) => Body::from(body.to_string()),
			None => Body::empty(),
		})?)
		.await?;
	let status = res.status();
	let bytes = to_bytes(res.into_body(), usize::MAX).await?;
	let value = serde_json::from_slice(&bytes)
		.unwrap_or_else(|_| json!({ "raw": String::from_utf8_lossy(&bytes) }));
	Ok((status, value))
}

#[serial]
#[tokio::test]
async fn profile_does_not_treat_membership_as_an_organization_account() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	insert_user_organization_membership(&mm, seed.user1.id, seed.org2_id).await?;
	let token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);

	let (status, profile) =
		request_json(&app, "GET", &cookie, "/api/users/me/profile", None).await?;

	assert_eq!(status, StatusCode::OK, "{profile:?}");
	let orgs = profile["data"]["availableOrganizations"]
		.as_array()
		.ok_or("missing availableOrganizations")?;
	let ids = orgs
		.iter()
		.map(|org| org["id"].as_str().unwrap_or_default())
		.collect::<Vec<_>>();
	assert!(
		ids.contains(&seed.org1_id.to_string().as_str()),
		"{profile:?}"
	);
	assert!(
		!ids.contains(&seed.org2_id.to_string().as_str()),
		"{profile:?}"
	);
	assert_eq!(
		profile["data"]["activeOrganization"]["id"].as_str(),
		Some(seed.org1_id.to_string().as_str())
	);
	Ok(())
}

#[serial]
#[tokio::test]
async fn same_email_accounts_list_and_switch_by_organization() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	let mut tx = mm.dbx().db().begin().await?;
	lib_core::model::store::set_user_context(
		&mut tx,
		crate::common::system_user_id(),
	)
	.await?;
	lib_core::model::store::set_org_context(
		&mut tx,
		crate::common::system_org_id(),
		lib_core::ctx::ROLE_SYSTEM_ADMIN,
	)
	.await?;
	sqlx::query("UPDATE users SET email = $1 WHERE id = $2")
		.bind(&seed.user1.email)
		.bind(seed.user2.id)
		.execute(&mut *tx)
		.await?;
	tx.commit().await?;

	let legacy_token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let app = web_server::app(mm.clone());
	let (status, ambiguous) = request_json(
		&app,
		"GET",
		&cookie_header(&legacy_token.to_string()),
		"/api/users/me",
		None,
	)
	.await?;
	assert_eq!(status, StatusCode::FORBIDDEN, "{ambiguous:?}");

	let token = generate_web_token(
		&format!("{}|{}", seed.user1.email, seed.org1_id),
		seed.user1.token_salt,
	)?;
	let cookie = cookie_header(&token.to_string());
	let (status, profile) =
		request_json(&app, "GET", &cookie, "/api/users/me/profile", None).await?;
	assert_eq!(status, StatusCode::OK, "{profile:?}");
	let orgs = profile["data"]["availableOrganizations"]
		.as_array()
		.ok_or("missing availableOrganizations")?;
	assert!(
		orgs.iter().any(|org| org["id"] == seed.org1_id.to_string())
			&& orgs.iter().any(|org| org["id"] == seed.org2_id.to_string()),
		"{profile:?}"
	);
	let (status, routing) = request_json(
		&app,
		"GET",
		&cookie,
		&format!("/api/users/me/routing?organizationId={}", seed.org2_id),
		None,
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{routing:?}");

	let switch_req = Request::builder()
		.method("PUT")
		.uri("/api/users/me/organization")
		.header("cookie", cookie)
		.header("content-type", "application/json")
		.body(Body::from(
			json!({ "data": { "organization_id": seed.org2_id } }).to_string(),
		))?;
	let switch_res = app.clone().oneshot(switch_req).await?;
	assert_eq!(switch_res.status(), StatusCode::OK);
	let switched_cookie = switch_res
		.headers()
		.get_all(header::SET_COOKIE)
		.iter()
		.filter_map(|value| value.to_str().ok())
		.find_map(|value| value.strip_prefix("auth-token=")?.split(';').next())
		.ok_or("missing switched auth-token cookie")?
		.to_string();
	let (status, me) = request_json(
		&app,
		"GET",
		&cookie_header(&switched_cookie),
		"/api/users/me",
		None,
	)
	.await?;
	assert_eq!(status, StatusCode::OK, "{me:?}");
	assert_eq!(me["data"]["id"], seed.user2.id.to_string());
	assert_eq!(me["data"]["organizationId"], seed.org2_id.to_string());
	Ok(())
}

#[serial]
#[tokio::test]
async fn membership_without_same_email_account_cannot_switch_organization(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	insert_user_organization_membership(&mm, seed.user1.id, seed.org2_id).await?;
	let token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());

	let (status, switched) = request_json(
		&app,
		"PUT",
		&cookie,
		"/api/users/me/organization",
		Some(json!({ "data": { "organization_id": seed.org2_id } })),
	)
	.await?;

	assert_eq!(status, StatusCode::FORBIDDEN, "{switched:?}");

	let (status, profile) =
		request_json(&app, "GET", &cookie, "/api/users/me/profile", None).await?;
	assert_eq!(status, StatusCode::OK, "{profile:?}");
	assert_eq!(
		profile["data"]["activeOrganization"]["id"].as_str(),
		Some(seed.org1_id.to_string().as_str())
	);

	Ok(())
}

#[serial]
#[tokio::test]
async fn membership_without_same_email_account_cannot_preview_routing() -> Result<()>
{
	let mm = init_test_mm().await?;
	let seed = seed_two_orgs_users_cases(&mm).await?;
	insert_user_organization_membership(&mm, seed.user1.id, seed.org2_id).await?;
	let token = generate_web_token(&seed.user1.email, seed.user1.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm.clone());

	let (status, preview) = request_json(
		&app,
		"GET",
		&cookie,
		&format!("/api/users/me/routing?organizationId={}", seed.org2_id),
		None,
	)
	.await?;
	assert_eq!(status, StatusCode::FORBIDDEN, "{preview:?}");

	let (status, profile) =
		request_json(&app, "GET", &cookie, "/api/users/me/profile", None).await?;
	assert_eq!(status, StatusCode::OK, "{profile:?}");
	assert_eq!(
		profile["data"]["activeOrganization"]["id"].as_str(),
		Some(seed.org1_id.to_string().as_str())
	);

	Ok(())
}
