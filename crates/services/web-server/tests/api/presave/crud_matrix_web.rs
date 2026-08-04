use super::helpers::*;
use crate::common::{cookie_header, init_test_mm, seed_org_with_users, Result};
use axum::http::{Method, StatusCode};
use lib_auth::token::generate_web_token;
use serde_json::json;

#[tokio::test]
async fn test_section_presave_sender_receiver_product_reporter_rest_contract(
) -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let admin_token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let admin_cookie = cookie_header(&admin_token.to_string());
	let app = web_server::app(mm);

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		"/api/presaves/senders".to_string(),
		Some(json!({ "data": { "rows": {
			"sender": { "senderType": "1", "organizationName": "REST Sender Org", "countryCode": "US", "email": "sender@example.com" },
			"gateways": [], "responsiblePersons": []
		} } })),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	assert!(
		value["data"]["rows"]["sender"].get("name").is_none(),
		"{value:?}"
	);
	assert!(
		value["data"]["rows"]["sender"].get("comments").is_none(),
		"{value:?}"
	);
	let sender_id = data_rows_id(&value, "sender")?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		format!("/api/presaves/senders/{sender_id}/gateways"),
		Some(json!({
			"data": {
				"sequence_number": 1,
				"gateway_authority": "fda",
				"sender_identifier": "REST-SENDER"
			}
		})),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	let gateway_id = data_id(&value)?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		format!("/api/presaves/senders/{sender_id}/responsible-persons"),
		Some(json!({
			"data": {
				"sequence_number": 1,
				"person_given_name": "Ada",
				"person_family_name": "Lovelace"
			}
		})),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	let responsible_id = data_id(&value)?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		"/api/presaves/receivers".to_string(),
		Some(json!({ "data": { "rows": {
			"receiver": { "receiverType": "Regulatory Authority", "organizationName": "REST Receiver Org" },
			"consignees": [], "routes": []
		} } })),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	assert!(
		value["data"]["rows"]["receiver"].get("name").is_none(),
		"{value:?}"
	);
	assert!(
		value["data"]["rows"]["receiver"].get("comments").is_none(),
		"{value:?}"
	);
	let receiver_id = data_rows_id(&value, "receiver")?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		format!("/api/presaves/receivers/{receiver_id}/consignees"),
		Some(json!({
			"data": {
				"sequence_number": 1,
				"name": "REST Consignee",
				"email": "consignee@example.com"
			}
		})),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	let consignee_id = data_id(&value)?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		"/api/presaves/products".to_string(),
		Some(json!({ "data": { "rows": {
			"product": { "senderPresaveId": sender_id, "productId": "REST-PRODUCT-CANONICAL", "medicinalProduct": "REST Product Canonical" },
			"activeSubstances": []
		} } })),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	assert!(
		value["data"]["rows"]["product"].get("name").is_none(),
		"{value:?}"
	);
	assert!(
		value["data"]["rows"]["product"].get("comments").is_none(),
		"{value:?}"
	);
	let product_id = data_rows_id(&value, "product")?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		format!("/api/presaves/products/{product_id}/active-substances"),
		Some(json!({
			"data": {
				"sequence_number": 1,
				"substance_name": "REST Substance",
				"strength_value": "10.5",
				"strength_unit": "mg"
			}
		})),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	let substance_id = data_id(&value)?;

	let (status, value) = request_json(
		&app,
		&admin_cookie,
		Method::POST,
		"/api/presaves/reporters".to_string(),
		Some(json!({ "data": { "rows": { "reporter": {
				"reporterGivenName": "Grace",
				"reporterFamilyName": "Hopper",
				"organization": "REST Reporter Org",
				"countryCode": "US",
				"qualification": "1"
			} } } })),
	)
	.await?;
	assert_eq!(status, StatusCode::CREATED, "{value:?}");
	assert!(
		value["data"]["rows"]["reporter"].get("name").is_none(),
		"{value:?}"
	);
	assert!(
		value["data"]["rows"]["reporter"].get("comments").is_none(),
		"{value:?}"
	);
	let reporter_id = data_rows_id(&value, "reporter")?;

	for (uri, id) in [
		("/api/presaves/senders".to_string(), sender_id),
		("/api/presaves/receivers".to_string(), receiver_id),
		("/api/presaves/products".to_string(), product_id),
		("/api/presaves/reporters".to_string(), reporter_id),
	] {
		let (status, value) =
			request_json(&app, &admin_cookie, Method::GET, uri, None).await?;
		assert_eq!(status, StatusCode::OK, "{value:?}");
		assert!(
			value["data"]
				.as_array()
				.ok_or("presave list data is not array")?
				.iter()
				.any(|row| {
					row["id"].as_str() == Some(&id.to_string())
						|| row["rows"]
							.as_object()
							.and_then(|rows| {
								rows.values().find_map(|value| value["id"].as_str())
							})
							.is_some_and(|value| value == id.to_string())
				}),
			"{value:?}"
		);
	}

	for uri in [
		format!("/api/presaves/senders/{sender_id}/gateways/{gateway_id}"),
		format!(
			"/api/presaves/senders/{sender_id}/responsible-persons/{responsible_id}"
		),
		format!("/api/presaves/receivers/{receiver_id}/consignees/{consignee_id}"),
		format!(
			"/api/presaves/products/{product_id}/active-substances/{substance_id}"
		),
	] {
		let (status, value) =
			request_json(&app, &admin_cookie, Method::GET, uri, None).await?;
		assert_eq!(status, StatusCode::OK, "{value:?}");
	}

	for (uri, body, field, expected) in [
		(
			format!("/api/presaves/senders/{sender_id}"),
			json!({ "data": { "organizationName": "REST Sender Org Updated" } }),
			"organizationName",
			"REST Sender Org Updated",
		),
		(
			format!("/api/presaves/receivers/{receiver_id}"),
			json!({ "data": { "description": "REST receiver updated" } }),
			"description",
			"REST receiver updated",
		),
		(
			format!("/api/presaves/products/{product_id}"),
			json!({ "data": { "drugBrandName": "REST Brand Updated" } }),
			"drugBrandName",
			"REST Brand Updated",
		),
		(
			format!("/api/presaves/reporters/{reporter_id}"),
			json!({ "data": { "rows": { "reporter": { "reporterGivenName": "Grace Updated" } } } }),
			"reporterGivenName",
			"Grace Updated",
		),
	] {
		let (status, value) =
			request_json(&app, &admin_cookie, Method::PATCH, uri, Some(body))
				.await?;
		assert_eq!(status, StatusCode::OK, "{value:?}");
		let response_row = value["data"]["rows"]
			.as_object()
			.and_then(|rows| rows.values().next())
			.unwrap_or(&value["data"]);
		assert_eq!(response_row[field].as_str(), Some(expected));
	}

	for uri in [
		format!("/api/presaves/senders/{sender_id}/gateways/{gateway_id}"),
		format!(
			"/api/presaves/senders/{sender_id}/responsible-persons/{responsible_id}"
		),
		format!("/api/presaves/receivers/{receiver_id}/consignees/{consignee_id}"),
		format!(
			"/api/presaves/products/{product_id}/active-substances/{substance_id}"
		),
		format!("/api/presaves/reporters/{reporter_id}"),
		format!("/api/presaves/products/{product_id}"),
		format!("/api/presaves/receivers/{receiver_id}"),
		format!("/api/presaves/senders/{sender_id}"),
	] {
		let (status, value) =
			request_json(&app, &admin_cookie, Method::DELETE, uri.clone(), None)
				.await?;
		assert_eq!(status, StatusCode::NO_CONTENT, "{value:?}");
	}

	Ok(())
}
