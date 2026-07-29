// Relatedness Assessment REST endpoints (G.k.9.i.2.r)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::drug::DrugInformationBmc;
use lib_core::model::drug_reaction_assessment::{
	DrugReactionAssessmentBmc, RelatednessAssessment, RelatednessAssessmentBmc,
	RelatednessAssessmentFilter, RelatednessAssessmentForCreate,
	RelatednessAssessmentForUpdate,
};
use lib_core::model::{self, ModelManager};
use lib_rest_core::rest_params::{ParamsForCreate, ParamsForUpdate};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::Result;
use lib_web::middleware::mw_auth::CtxW;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde_json::json;
use uuid::Uuid;

async fn ensure_assessment_path(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	drug_id: Uuid,
	assessment_id: Uuid,
) -> Result<()> {
	DrugInformationBmc::get_in_case(ctx, mm, case_id, drug_id).await?;
	let assessment = DrugReactionAssessmentBmc::get(ctx, mm, assessment_id).await?;
	if assessment.drug_id != drug_id {
		return Err(model::Error::EntityUuidNotFound {
			entity: "drug_reaction_assessments",
			id: assessment_id,
		}
		.into());
	}
	Ok(())
}

fn ensure_relatedness_scope(
	assessment_id: Uuid,
	entity: &RelatednessAssessment,
) -> Result<()> {
	if entity.drug_reaction_assessment_id != assessment_id {
		return Err(model::Error::EntityUuidNotFound {
			entity: "relatedness_assessments",
			id: entity.id,
		}
		.into());
	}
	Ok(())
}

pub async fn create_relatedness_assessment(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id)): Path<(Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForCreate<RelatednessAssessmentForCreate>>,
) -> Result<(StatusCode, Json<DataRestResult<RelatednessAssessment>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:new:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let ParamsForCreate { data } = params;
				let mut data = data;
				data.drug_reaction_assessment_id = assessment_id;
				let id = RelatednessAssessmentBmc::create(ctx, mm, data).await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				Ok((StatusCode::CREATED, Json(DataRestResult { data: entity })))
			})
		},
	)
	.await
}

pub async fn list_relatedness_assessments(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<RelatednessAssessment>>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:list:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let filter = RelatednessAssessmentFilter {
					drug_reaction_assessment_id: Some(OpValsValue::from(vec![
						OpValValue::Eq(json!(assessment_id.to_string())),
					])),
					..Default::default()
				};
				let entities = RelatednessAssessmentBmc::list(
					ctx,
					mm,
					Some(vec![filter]),
					Some(ListOptions::default()),
				)
				.await?;
				Ok((StatusCode::OK, Json(DataRestResult { data: entities })))
			})
		},
	)
	.await
}

pub async fn get_relatedness_assessment(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id, id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<RelatednessAssessment>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:{id}:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				ensure_relatedness_scope(assessment_id, &entity)?;
				Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
			})
		},
	)
	.await
}

pub async fn update_relatedness_assessment(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id, id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
	Json(params): Json<ParamsForUpdate<RelatednessAssessmentForUpdate>>,
) -> Result<(StatusCode, Json<DataRestResult<RelatednessAssessment>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:{id}:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				ensure_relatedness_scope(assessment_id, &entity)?;
				let ParamsForUpdate { data } = params;
				RelatednessAssessmentBmc::update(ctx, mm, id, data).await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
			})
		},
	)
	.await
}

pub async fn delete_relatedness_assessment(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id, id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<StatusCode> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:{id}:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				ensure_relatedness_scope(assessment_id, &entity)?;
				RelatednessAssessmentBmc::delete(ctx, mm, id).await?;
				Ok(StatusCode::NO_CONTENT)
			})
		},
	)
	.await
}

pub async fn restore_relatedness_assessment(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, drug_id, assessment_id, id)): Path<(Uuid, Uuid, Uuid, Uuid)>,
) -> Result<(StatusCode, Json<DataRestResult<RelatednessAssessment>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_mutation(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("relatedness-assessment:{id}:assessment:{assessment_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				ensure_assessment_path(ctx, mm, case_id, drug_id, assessment_id)
					.await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				ensure_relatedness_scope(assessment_id, &entity)?;
				RelatednessAssessmentBmc::restore(ctx, mm, id).await?;
				let entity = RelatednessAssessmentBmc::get(ctx, mm, id).await?;
				Ok((StatusCode::OK, Json(DataRestResult { data: entity })))
			})
		},
	)
	.await
}
