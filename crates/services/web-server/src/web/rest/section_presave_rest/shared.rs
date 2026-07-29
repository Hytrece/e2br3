//! Shared imports, scope guards, and parent-scope helpers
//! used across the presave section entity modules.

pub(super) use axum::extract::{Path, State};
pub(super) use axum::http::StatusCode;
pub(super) use axum::Json;
pub(super) use lib_core::authorization::EnforcedScopeFilter;
pub(super) use lib_core::model::authorization::PresaveAuthorizationKind;
pub(super) use lib_core::model::presave::{
	NarrativePresave, NarrativePresaveBmc, NarrativePresaveForCreate,
	NarrativePresaveForUpdate, ProductPresave, ProductPresaveActiveSubstance,
	ProductPresaveActiveSubstanceBmc, ProductPresaveActiveSubstanceForCreate,
	ProductPresaveActiveSubstanceForUpdate, ProductPresaveBmc,
	ProductPresaveForCreate, ProductPresaveForUpdate, ReceiverPresave,
	ReceiverPresaveBmc, ReceiverPresaveConsignee, ReceiverPresaveConsigneeBmc,
	ReceiverPresaveConsigneeForCreate, ReceiverPresaveConsigneeForUpdate,
	ReceiverPresaveForCreate, ReceiverPresaveForUpdate, ReceiverPresaveRouteBmc,
	ReceiverPresaveRouteForCreate, ReceiverPresaveRouteForUpdate, ReporterPresave,
	ReporterPresaveBmc, ReporterPresaveForCreate, ReporterPresaveForUpdate,
	SenderPresave, SenderPresaveBmc, SenderPresaveForCreate, SenderPresaveForUpdate,
	SenderPresaveGateway, SenderPresaveGatewayBmc, SenderPresaveGatewayForCreate,
	SenderPresaveGatewayForUpdate, SenderPresaveResponsiblePerson,
	SenderPresaveResponsiblePersonBmc, SenderPresaveResponsiblePersonForCreate,
	SenderPresaveResponsiblePersonForUpdate, StudyPresave, StudyPresaveBmc,
	StudyPresaveFdaCrossReportedIndNumberBmc,
	StudyPresaveFdaCrossReportedIndNumberForCreate,
	StudyPresaveFdaCrossReportedIndNumberForUpdate, StudyPresaveForCreate,
	StudyPresaveForUpdate, StudyPresaveProduct, StudyPresaveProductBmc,
	StudyPresaveProductForCreate, StudyPresaveProductForUpdate,
	StudyPresaveRegistrationNumber, StudyPresaveRegistrationNumberBmc,
	StudyPresaveRegistrationNumberForCreate,
	StudyPresaveRegistrationNumberForUpdate, StudyPresaveReporter,
	StudyPresaveReporterBmc, StudyPresaveReporterForCreate,
	StudyPresaveReporterForUpdate,
};
pub(super) use lib_core::model::presave_lifecycle::{
	PresaveKind, PresaveLifecycleService,
};
pub(super) use lib_core::model::{self, ModelManager};
pub(super) use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate};
pub(super) use lib_rest_core::rest_result::DataRestResult;
pub(super) use lib_rest_core::{
	with_authorized_presave_atomic_create, with_authorized_presave_atomic_update,
	with_authorized_presave_collection, with_authorized_presave_create,
	with_authorized_presave_read, with_authorized_presave_update, Error, Result,
};
pub(super) use lib_web::middleware::mw_auth::CtxW;
pub(super) use lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW;
pub(super) use serde::{Deserialize, Serialize};
pub(super) use std::collections::HashSet;
pub(super) use uuid::Uuid;

#[allow(unused_macros)]
macro_rules! generate_simple_presave_rest_fns {
	(
		Bmc: $bmc:ident,
		Entity: $entity:ident,
		ForCreate: $for_create:ident,
		ForUpdate: $for_update:ident,
		CreateFn: $create_fn:ident,
		ListFn: $list_fn:ident,
		GetFn: $get_fn:ident,
		UpdateFn: $update_fn:ident,
		DeleteFn: $delete_fn:ident,
		Kind: $kind:ident
	) => {
		pub async fn $create_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Json(params): Json<ParamsForCreate<$for_create>>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_create(
				&ctx,
				&snapshot,
				&mm,
				stringify!($kind),
				move |ctx, mm| {
					Box::pin(async move {
						let ParamsForCreate { data } = params;
						let id = $bmc::create(ctx, mm, data).await?;
						Ok(rest_created($bmc::get(ctx, mm, id).await?))
					})
				},
			)
			.await
		}

		pub async fn $list_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
		) -> Result<(StatusCode, Json<DataRestResult<Vec<$entity>>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_collection(
				&ctx,
				&snapshot,
				&mm,
				|ctx, mm, _scope| {
					Box::pin(
						async move { Ok(rest_ok($bmc::list(ctx, mm, None).await?)) },
					)
				},
			)
			.await
		}

		pub async fn $get_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_read(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				|ctx, mm| {
					Box::pin(
						async move { Ok(rest_ok($bmc::get(ctx, mm, id).await?)) },
					)
				},
			)
			.await
		}

		pub async fn $update_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
			Json(params): Json<ParamsForUpdate<$for_update>>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				move |ctx, mm| {
					Box::pin(async move {
						let ParamsForUpdate { data } = params;
						if data.deleted == Some(true) {
							PresaveLifecycleService::archive(
								ctx,
								mm,
								PresaveKind::$kind,
								id,
							)
							.await?;
						} else {
							$bmc::update(ctx, mm, id, data).await?;
						}
						Ok(rest_ok($bmc::get(ctx, mm, id).await?))
					})
				},
			)
			.await
		}

		pub async fn $delete_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
		) -> Result<StatusCode> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				|ctx, mm| {
					Box::pin(async move {
						PresaveLifecycleService::archive(
							ctx,
							mm,
							PresaveKind::$kind,
							id,
						)
						.await?;
						Ok(StatusCode::NO_CONTENT)
					})
				},
			)
			.await
		}
	};
}

#[allow(unused_imports)]
pub(super) use generate_simple_presave_rest_fns;

macro_rules! generate_single_row_presave_rest_fns {
	(
		Bmc: $bmc:ident,
		Entity: $entity:ident,
		ForCreate: $for_create:ident,
		ForUpdate: $for_update:ident,
		Row: $row:ident,
		Details: $details:ident,
		Rows: $rows:ident,
		CreateRequest: $create_request:ident,
		CreateRows: $create_rows:ident,
		UpdateRequest: $update_request:ident,
		UpdateRows: $update_rows:ident,
		CreateFn: $create_fn:ident,
		ListFn: $list_fn:ident,
		GetFn: $get_fn:ident,
		UpdateFn: $update_fn:ident,
		DeleteFn: $delete_fn:ident,
		Kind: $kind:ident
	) => {
		#[derive(Debug, Serialize)]
		pub struct $rows {
			pub $row: $entity,
		}
		#[derive(Debug, Serialize)]
		pub struct $details {
			pub rows: $rows,
		}
		#[derive(Deserialize)]
		#[serde(rename_all = "camelCase", deny_unknown_fields)]
		pub struct $create_request {
			pub rows: $create_rows,
		}
		#[derive(Deserialize)]
		#[serde(rename_all = "camelCase", deny_unknown_fields)]
		pub struct $create_rows {
			pub $row: $for_create,
		}
		#[derive(Deserialize)]
		#[serde(rename_all = "camelCase", deny_unknown_fields)]
		pub struct $update_request {
			pub rows: $update_rows,
		}
		#[derive(Deserialize)]
		#[serde(rename_all = "camelCase", deny_unknown_fields)]
		pub struct $update_rows {
			pub $row: $for_update,
		}

		fn details(entity: $entity) -> $details {
			$details {
				rows: $rows { $row: entity },
			}
		}

		pub async fn $create_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Json(params): Json<ParamsForCreate<$create_request>>,
		) -> Result<(StatusCode, Json<DataRestResult<$details>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_create(
				&ctx,
				&snapshot,
				&mm,
				stringify!($kind),
				move |ctx, mm| {
					Box::pin(async move {
						let id =
							$bmc::create(ctx, mm, params.data.rows.$row).await?;
						Ok(rest_created(details($bmc::get(ctx, mm, id).await?)))
					})
				},
			)
			.await
		}

		pub async fn $list_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
		) -> Result<(StatusCode, Json<DataRestResult<Vec<$details>>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_collection(
				&ctx,
				&snapshot,
				&mm,
				|ctx, mm, _scope| {
					Box::pin(async move {
						Ok(rest_ok(
							$bmc::list(ctx, mm, None)
								.await?
								.into_iter()
								.map(details)
								.collect(),
						))
					})
				},
			)
			.await
		}

		pub async fn $get_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
		) -> Result<(StatusCode, Json<DataRestResult<$details>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_read(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				|ctx, mm| {
					Box::pin(async move {
						Ok(rest_ok(details($bmc::get(ctx, mm, id).await?)))
					})
				},
			)
			.await
		}

		pub async fn $update_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
			Json(params): Json<ParamsForUpdate<$update_request>>,
		) -> Result<(StatusCode, Json<DataRestResult<$details>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				move |ctx, mm| {
					Box::pin(async move {
						let data = params.data.rows.$row;
						if data.deleted == Some(true) {
							PresaveLifecycleService::archive(
								ctx,
								mm,
								PresaveKind::$kind,
								id,
							)
							.await?;
						} else {
							$bmc::update(ctx, mm, id, data).await?;
						}
						Ok(rest_ok(details($bmc::get(ctx, mm, id).await?)))
					})
				},
			)
			.await
		}

		pub async fn $delete_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(id): Path<Uuid>,
		) -> Result<StatusCode> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$kind,
				id,
				|ctx, mm| {
					Box::pin(async move {
						PresaveLifecycleService::archive(
							ctx,
							mm,
							PresaveKind::$kind,
							id,
						)
						.await?;
						Ok(StatusCode::NO_CONTENT)
					})
				},
			)
			.await
		}
	};
}

pub(super) use generate_single_row_presave_rest_fns;

macro_rules! delete_presave_child {
	(hard, $bmc:ident, $for_update:ident, $ctx:ident, $mm:ident, $id:ident) => {
		$bmc::delete(&$ctx, &$mm, $id).await?;
	};
	(soft, $bmc:ident, $for_update:ident, $ctx:ident, $mm:ident, $id:ident) => {
		$bmc::update(
			&$ctx,
			&$mm,
			$id,
			$for_update {
				deleted: Some(true),
				..Default::default()
			},
		)
		.await?;
	};
}

pub(super) use delete_presave_child;

macro_rules! generate_presave_child_rest_fns {
	(
		Bmc: $bmc:ident,
		Entity: $entity:ident,
		RestCreate: $rest_create:ident,
		ForUpdate: $for_update:ident,
		CreateFn: $create_fn:ident,
		ListFn: $list_fn:ident,
		GetFn: $get_fn:ident,
		UpdateFn: $update_fn:ident,
		DeleteFn: $delete_fn:ident,
		ParentField: $parent_field:ident,
		ParentKind: $parent_kind:ident,
		EntityName: $entity_name:literal,
		DeleteMode: $delete_mode:ident
	) => {
		pub async fn $create_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(parent_id): Path<Uuid>,
			Json(params): Json<ParamsForCreate<$rest_create>>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$parent_kind,
				parent_id,
				move |ctx, mm| {
					Box::pin(async move {
						let ParamsForCreate { data } = params;
						let id =
							$bmc::create(ctx, mm, data.into_core(parent_id)).await?;
						Ok(rest_created($bmc::get(ctx, mm, id).await?))
					})
				},
			)
			.await
		}

		pub async fn $list_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path(parent_id): Path<Uuid>,
		) -> Result<(StatusCode, Json<DataRestResult<Vec<$entity>>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_read(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$parent_kind,
				parent_id,
				|ctx, mm| {
					Box::pin(async move {
						Ok(rest_ok($bmc::list_by_parent(ctx, mm, parent_id).await?))
					})
				},
			)
			.await
		}

		pub async fn $get_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path((parent_id, id)): Path<(Uuid, Uuid)>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_read(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$parent_kind,
				parent_id,
				|ctx, mm| {
					Box::pin(async move {
						let entity = $bmc::get(ctx, mm, id).await?;
						ensure_parent_scope(
							parent_id,
							entity.$parent_field,
							id,
							$entity_name,
						)?;
						Ok(rest_ok(entity))
					})
				},
			)
			.await
		}

		pub async fn $update_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path((parent_id, id)): Path<(Uuid, Uuid)>,
			Json(params): Json<ParamsForUpdate<$for_update>>,
		) -> Result<(StatusCode, Json<DataRestResult<$entity>>)> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$parent_kind,
				parent_id,
				move |ctx, mm| {
					Box::pin(async move {
						let ParamsForUpdate { data } = params;
						let entity = $bmc::get(ctx, mm, id).await?;
						ensure_parent_scope(
							parent_id,
							entity.$parent_field,
							id,
							$entity_name,
						)?;
						$bmc::update(ctx, mm, id, data).await?;
						Ok(rest_ok($bmc::get(ctx, mm, id).await?))
					})
				},
			)
			.await
		}

		pub async fn $delete_fn(
			State(mm): State<ModelManager>,
			ctx_w: CtxW,
			snapshot: AuthorizationSnapshotW,
			Path((parent_id, id)): Path<(Uuid, Uuid)>,
		) -> Result<StatusCode> {
			let ctx = ctx_w.0;
			with_authorized_presave_update(
				&ctx,
				&snapshot,
				&mm,
				PresaveAuthorizationKind::$parent_kind,
				parent_id,
				|ctx, mm| {
					Box::pin(async move {
						let entity = $bmc::get(ctx, mm, id).await?;
						ensure_parent_scope(
							parent_id,
							entity.$parent_field,
							id,
							$entity_name,
						)?;
						delete_presave_child!(
							$delete_mode,
							$bmc,
							$for_update,
							ctx,
							mm,
							id
						);
						Ok(StatusCode::NO_CONTENT)
					})
				},
			)
			.await
		}
	};
}

pub(super) use generate_presave_child_rest_fns;

pub(super) fn normalized_set(values: &[String]) -> HashSet<String> {
	values
		.iter()
		.map(|value| value.trim().to_ascii_lowercase())
		.filter(|value| !value.is_empty())
		.collect()
}

fn identifier_allowed(allowed: &[String], identifier: Uuid) -> bool {
	let allowed = normalized_set(allowed);
	if allowed.is_empty() {
		return true;
	}
	allowed.contains(&identifier.to_string().to_ascii_lowercase())
}

pub(crate) fn product_presave_allowed(
	scope: &EnforcedScopeFilter,
	entity: &ProductPresave,
) -> bool {
	identifier_allowed(scope.product_ids(), entity.id)
}

pub(super) fn deny_presave_scope() -> Error {
	Error::PermissionDenied {
		required_permission: "PresaveTemplate.Scope".to_string(),
	}
}

pub(super) fn presave_case_link_conflict(message: &str) -> Error {
	model::Error::Conflict {
		message: message.to_string(),
	}
	.into()
}

pub(super) fn rest_ok<T: Serialize>(
	data: T,
) -> (StatusCode, Json<DataRestResult<T>>) {
	(StatusCode::OK, Json(DataRestResult { data }))
}

pub(super) fn rest_created<T: Serialize>(
	data: T,
) -> (StatusCode, Json<DataRestResult<T>>) {
	(StatusCode::CREATED, Json(DataRestResult { data }))
}

pub(super) async fn presave_scope_assigned_to_users(
	mm: &ModelManager,
	organization_id: Uuid,
	scope_column: &str,
	identifiers: Vec<String>,
) -> Result<bool> {
	if identifiers.is_empty() {
		return Ok(false);
	}
	let sql = match scope_column {
		"access_sender_ids" => {
			r#"
			SELECT EXISTS (
				SELECT 1
				FROM users u
				CROSS JOIN LATERAL jsonb_array_elements_text(
					CASE
						WHEN u.access_sender_ids IS NULL OR btrim(u.access_sender_ids) = ''
							THEN '[]'::jsonb
						ELSE u.access_sender_ids::jsonb
					END
				) AS scope_value(value)
				WHERE u.organization_id = $1
				  AND u.active = true
				  AND lower(btrim(scope_value.value)) = ANY($2)
			)
			"#
		}
		"access_product_ids" => {
			r#"
			SELECT EXISTS (
				SELECT 1
				FROM users u
				CROSS JOIN LATERAL jsonb_array_elements_text(
					CASE
						WHEN u.access_product_ids IS NULL OR btrim(u.access_product_ids) = ''
							THEN '[]'::jsonb
						ELSE u.access_product_ids::jsonb
					END
				) AS scope_value(value)
				WHERE u.organization_id = $1
				  AND u.active = true
				  AND lower(btrim(scope_value.value)) = ANY($2)
			)
			"#
		}
		"access_study_ids" => {
			r#"
			SELECT EXISTS (
				SELECT 1
				FROM users u
				CROSS JOIN LATERAL jsonb_array_elements_text(
					CASE
						WHEN u.access_study_ids IS NULL OR btrim(u.access_study_ids) = ''
							THEN '[]'::jsonb
						ELSE u.access_study_ids::jsonb
					END
				) AS scope_value(value)
				WHERE u.organization_id = $1
				  AND u.active = true
				  AND lower(btrim(scope_value.value)) = ANY($2)
			)
			"#
		}
		_ => return Ok(false),
	};
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(sql)
				.bind(organization_id)
				.bind(identifiers),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) async fn sender_presave_used_by_cases(
	mm: &ModelManager,
	organization_id: Uuid,
	id: Uuid,
) -> Result<bool> {
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				r#"
				SELECT EXISTS (
					SELECT 1
					FROM sender_information sender
					JOIN cases c ON c.id = sender.case_id
					WHERE c.organization_id = $1
					  AND sender.source_sender_presave_id = $2
				)
				"#,
			)
			.bind(organization_id)
			.bind(id),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) async fn product_presave_used_by_cases(
	mm: &ModelManager,
	organization_id: Uuid,
	id: Uuid,
) -> Result<bool> {
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				r#"
				SELECT EXISTS (
					SELECT 1
					FROM drug_information drug
					JOIN cases c ON c.id = drug.case_id
					WHERE c.organization_id = $1
					  AND drug.source_product_presave_id = $2
				)
				"#,
			)
			.bind(organization_id)
			.bind(id),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) async fn study_presave_used_by_cases(
	mm: &ModelManager,
	organization_id: Uuid,
	id: Uuid,
) -> Result<bool> {
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				r#"
				SELECT EXISTS (
					SELECT 1
					FROM study_information study
					JOIN cases c ON c.id = study.case_id
					WHERE c.organization_id = $1
					  AND study.source_study_presave_id = $2
				)
				"#,
			)
			.bind(organization_id)
			.bind(id),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) async fn reporter_presave_used_by_cases(
	mm: &ModelManager,
	organization_id: Uuid,
	id: Uuid,
) -> Result<bool> {
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				r#"
				SELECT EXISTS (
					SELECT 1
					FROM primary_sources source
					JOIN cases c ON c.id = source.case_id
					WHERE c.organization_id = $1
					  AND source.source_reporter_presave_id = $2
				)
				"#,
			)
			.bind(organization_id)
			.bind(id),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) async fn narrative_presave_used_by_cases(
	mm: &ModelManager,
	organization_id: Uuid,
	id: Uuid,
) -> Result<bool> {
	let (exists,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				r#"
				SELECT EXISTS (
					SELECT 1
					FROM narrative_information narrative
					JOIN cases c ON c.id = narrative.case_id
					WHERE c.organization_id = $1
					  AND narrative.source_narrative_presave_id = $2
				)
				"#,
			)
			.bind(organization_id)
			.bind(id),
		)
		.await
		.map_err(|err| Error::from(model::Error::from(err)))?;
	Ok(exists)
}

pub(super) fn filter_sender_presaves_for_scope(
	scope: &EnforcedScopeFilter,
	entities: Vec<SenderPresave>,
) -> Vec<SenderPresave> {
	entities
		.into_iter()
		.filter(|entity| identifier_allowed(scope.sender_ids(), entity.id))
		.collect()
}

pub(super) fn filter_product_presaves_for_scope(
	scope: &EnforcedScopeFilter,
	entities: Vec<ProductPresave>,
) -> Vec<ProductPresave> {
	entities
		.into_iter()
		.filter(|entity| identifier_allowed(scope.product_ids(), entity.id))
		.collect()
}

pub(super) fn filter_study_presaves_for_scope(
	scope: &EnforcedScopeFilter,
	entities: Vec<StudyPresave>,
) -> Vec<StudyPresave> {
	entities
		.into_iter()
		.filter(|entity| identifier_allowed(scope.study_ids(), entity.id))
		.collect()
}

pub(super) fn text_present(value: Option<&str>) -> bool {
	value.is_some_and(|value| !value.trim().is_empty())
}

pub(super) fn ensure_parent_scope(
	path_parent_id: Uuid,
	entity_parent_id: Uuid,
	entity_id: Uuid,
	entity: &'static str,
) -> Result<()> {
	if path_parent_id != entity_parent_id {
		return Err(model::Error::EntityUuidNotFound {
			entity,
			id: entity_id,
		}
		.into());
	}
	Ok(())
}

pub(super) fn ensure_detail_parent_scope(
	path_parent_id: Uuid,
	entity_parent_id: Uuid,
	entity_id: Uuid,
	parent_name: &'static str,
	entity: &'static str,
) -> Result<()> {
	ensure_parent_scope(path_parent_id, entity_parent_id, entity_id, entity).map_err(
		|_| Error::BadRequest {
			message: format!(
				"{entity} child does not belong to {parent_name} {path_parent_id}"
			),
		},
	)
}
