use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::case_field_notation::CaseFieldNotationBmc;
use lib_core::model::ModelManager;
use lib_rest_core::{Error, Result};
use lib_web::middleware::mw_auth::CtxW;
use lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldNotationQuery {
	field_path: String,
	record_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveFieldNotationRequest {
	field_path: String,
	record_id: Option<Uuid>,
	notation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldNotationResponse {
	id: Option<Uuid>,
	field_path: String,
	record_id: Option<Uuid>,
	notation: String,
}

fn validate_field_path(field_path: &str) -> Result<&str> {
	let field_path = field_path.trim();
	if field_path.is_empty() || field_path.len() > 255 {
		return Err(Error::BadRequest {
			message: "fieldPath must contain 1-255 characters".to_string(),
		});
	}
	Ok(field_path)
}

pub async fn get_field_notation(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<FieldNotationQuery>,
) -> Result<(StatusCode, Json<FieldNotationResponse>)> {
	let ctx = ctx_w.0;
	let field_path = validate_field_path(&query.field_path)?.to_string();
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"field-notation",
		move |ctx, mm| {
			Box::pin(async move {
				let row = CaseFieldNotationBmc::get(
					ctx,
					mm,
					case_id,
					query.record_id,
					&field_path,
				)
				.await
				.map_err(Error::Model)?;
				Ok((
					StatusCode::OK,
					Json(FieldNotationResponse {
						id: row.as_ref().map(|row| row.id),
						field_path,
						record_id: query.record_id,
						notation: row.map(|row| row.notation).unwrap_or_default(),
					}),
				))
			})
		},
	)
	.await
}

pub async fn save_field_notation(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(request): Json<SaveFieldNotationRequest>,
) -> Result<(StatusCode, Json<FieldNotationResponse>)> {
	let ctx = ctx_w.0;
	let field_path = validate_field_path(&request.field_path)?.to_string();
	if request.notation.len() > 10_000 {
		return Err(Error::BadRequest {
			message: "notation must not exceed 10000 characters".to_string(),
		});
	}
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"field-notation",
		move |ctx, mm| {
			Box::pin(async move {
				let row = CaseFieldNotationBmc::upsert(
					ctx,
					mm,
					case_id,
					request.record_id,
					&field_path,
					request.notation.trim(),
				)
				.await
				.map_err(Error::Model)?;
				Ok((
					StatusCode::OK,
					Json(FieldNotationResponse {
						id: Some(row.id),
						field_path,
						record_id: request.record_id,
						notation: row.notation,
					}),
				))
			})
		},
	)
	.await
}

pub async fn delete_field_notation(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<FieldNotationQuery>,
) -> Result<(StatusCode, Json<FieldNotationResponse>)> {
	let ctx = ctx_w.0;
	let field_path = validate_field_path(&query.field_path)?.to_string();
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"field-notation",
		move |ctx, mm| {
			Box::pin(async move {
				CaseFieldNotationBmc::delete(
					ctx,
					mm,
					case_id,
					query.record_id,
					&field_path,
				)
				.await
				.map_err(Error::Model)?;
				Ok((
					StatusCode::OK,
					Json(FieldNotationResponse {
						id: None,
						field_path,
						record_id: query.record_id,
						notation: String::new(),
					}),
				))
			})
		},
	)
	.await
}
