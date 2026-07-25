// Narrative sub-resources REST endpoints (H.3.r, H.5.r)

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use lib_core::model::narrative::{
	CaseSummaryInformation, CaseSummaryInformationBmc, CaseSummaryInformationFilter,
	CaseSummaryInformationForCreate, CaseSummaryInformationForUpdate,
	NarrativeInformationBmc, SenderDiagnosis, SenderDiagnosisBmc,
	SenderDiagnosisFilter, SenderDiagnosisForCreate, SenderDiagnosisForUpdate,
};
use lib_core::model::patient::{PatientInformation, PatientInformationBmc};
use lib_core::model::{self, ModelManager};
use lib_core::narrative_template::{render_template, template_tokens};
use lib_rest_core::rest_result::DataRestResult;
use lib_rest_core::Result;
use lib_web::middleware::mw_auth::CtxW;
use modql::filter::{ListOptions, OpValValue, OpValsValue};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NarrativePreviewRequest {
	pub template: String,
}

#[derive(Debug, Serialize)]
pub struct NarrativePreviewToken {
	pub code: String,
	pub resolved: bool,
}

#[derive(Debug, Serialize)]
pub struct NarrativePreviewResponse {
	pub rendered: String,
	pub tokens: Vec<NarrativePreviewToken>,
}

fn patient_sex_display(value: &str) -> Option<&'static str> {
	match value.trim() {
		"1" => Some("남성"),
		"2" => Some("여성"),
		"0" => Some("알 수 없음"),
		_ => None,
	}
}

fn resolve_patient_template_code(
	patient: Option<&PatientInformation>,
	code: &str,
) -> Option<String> {
	let patient = patient?;
	match code {
		"D.2.2a" => patient
			.age_at_time_of_onset
			.map(|value| value.normalize().to_string()),
		"D.5" => patient
			.sex
			.as_deref()
			.and_then(patient_sex_display)
			.map(str::to_string),
		_ => None,
	}
}

async fn patient_for_case_optional(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Option<PatientInformation>> {
	match PatientInformationBmc::get_by_case(ctx, mm, case_id).await {
		Ok(patient) => Ok(Some(patient)),
		Err(model::Error::EntityUuidNotFound { .. }) => Ok(None),
		Err(err) => Err(err.into()),
	}
}

async fn narrative_id_for_case(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Uuid> {
	let narrative = NarrativeInformationBmc::get_by_case(ctx, mm, case_id).await?;
	Ok(narrative.id)
}

async fn ensure_narrative_scope(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	entity_narrative_id: Uuid,
	entity_id: Uuid,
	entity: &'static str,
) -> Result<()> {
	let expected_narrative_id = narrative_id_for_case(ctx, mm, case_id).await?;
	if expected_narrative_id != entity_narrative_id {
		return Err(model::Error::EntityUuidNotFound {
			entity,
			id: entity_id,
		}
		.into());
	}
	Ok(())
}

/// POST /api/cases/{case_id}/narrative/preview
pub async fn preview_narrative_template(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Json(params): Json<NarrativePreviewRequest>,
) -> Result<(StatusCode, Json<DataRestResult<NarrativePreviewResponse>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"narrative-preview",
		move |ctx, mm| {
			Box::pin(async move {
				let patient = patient_for_case_optional(ctx, mm, case_id).await?;
				let tokens = template_tokens(&params.template);
				let rendered = render_template(&params.template, |code| {
					resolve_patient_template_code(patient.as_ref(), code)
				});
				let tokens = tokens
					.into_iter()
					.map(|code| {
						let resolved =
							resolve_patient_template_code(patient.as_ref(), &code)
								.is_some();
						NarrativePreviewToken { code, resolved }
					})
					.collect();

				Ok((
					StatusCode::OK,
					Json(DataRestResult {
						data: NarrativePreviewResponse { rendered, tokens },
					}),
				))
			})
		},
	)
	.await
}

// -- Sender Diagnosis (H.3.r)
lib_rest_core::generate_patient_child_rest_fns! {
	Bmc: SenderDiagnosisBmc,
	Entity: SenderDiagnosis,
	ForCreate: SenderDiagnosisForCreate,
	ForUpdate: SenderDiagnosisForUpdate,
	Filter: SenderDiagnosisFilter,
	CreateFn: create_sender_diagnosis,
	ListFn: list_sender_diagnoses_generated,
	GetFn: get_sender_diagnosis,
	UpdateFn: update_sender_diagnosis,
	DeleteFn: delete_sender_diagnosis,
	RestoreFn: restore_sender_diagnosis,
	ParentField: narrative_id,
	ResolveParentFn: narrative_id_for_case,
	ScopeFn: ensure_narrative_scope,
	EntityName: "sender_diagnoses",
	DeleteResult: StatusCode,
	DeleteResponse: no_content
}

pub async fn list_sender_diagnoses(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(StatusCode, Json<DataRestResult<Vec<SenderDiagnosis>>>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"sender_diagnoses:list",
		move |ctx, mm| {
			Box::pin(async move {
				let Some(narrative) =
					NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id)
						.await?
				else {
					return Ok((
						StatusCode::OK,
						Json(DataRestResult { data: vec![] }),
					));
				};
				let filter = SenderDiagnosisFilter {
					narrative_id: Some(OpValsValue::from(vec![OpValValue::Eq(
						json!(narrative.id.to_string()),
					)])),
					..Default::default()
				};
				let entities = SenderDiagnosisBmc::list(
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

// -- Case Summary Information (H.5.r)
lib_rest_core::generate_patient_child_rest_fns! {
	Bmc: CaseSummaryInformationBmc,
	Entity: CaseSummaryInformation,
	ForCreate: CaseSummaryInformationForCreate,
	ForUpdate: CaseSummaryInformationForUpdate,
	Filter: CaseSummaryInformationFilter,
	CreateFn: create_case_summary_information,
	ListFn: list_case_summary_information_generated,
	GetFn: get_case_summary_information,
	UpdateFn: update_case_summary_information,
	DeleteFn: delete_case_summary_information,
	RestoreFn: restore_case_summary_information,
	ParentField: narrative_id,
	ResolveParentFn: narrative_id_for_case,
	ScopeFn: ensure_narrative_scope,
	EntityName: "case_summary_information",
	DeleteResult: StatusCode,
	DeleteResponse: no_content
}

pub async fn list_case_summary_information(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
) -> Result<(
	StatusCode,
	Json<DataRestResult<Vec<CaseSummaryInformation>>>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"case_summary_information:list",
		move |ctx, mm| {
			Box::pin(async move {
				let Some(narrative) =
					NarrativeInformationBmc::get_by_case_optional(ctx, mm, case_id)
						.await?
				else {
					return Ok((
						StatusCode::OK,
						Json(DataRestResult { data: vec![] }),
					));
				};
				let filter = CaseSummaryInformationFilter {
					narrative_id: Some(OpValsValue::from(vec![OpValValue::Eq(
						json!(narrative.id.to_string()),
					)])),
					..Default::default()
				};
				let entities = CaseSummaryInformationBmc::list(
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
