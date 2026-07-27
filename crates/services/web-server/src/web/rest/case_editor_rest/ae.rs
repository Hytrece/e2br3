use super::common::*;

const REACTION_ROW_ALIASES: &[(&str, &[&str])] = &[
	("primary_source_reaction", &["reactionPrimarySourceNative"]),
	(
		"primary_source_reaction_translation",
		&["reactionPrimarySourceTranslation"],
	),
	("reaction_language", &["reactionLanguage"]),
	("reaction_meddra_version", &["meddraVersion"]),
	("reaction_meddra_code", &["meddraCode"]),
	("term_highlighted", &["termHighlighted"]),
	("serious", &["seriousness.serious"]),
	("criteria_death", &["seriousness.criteriaResultsInDeath"]),
	(
		"criteria_life_threatening",
		&["seriousness.criteriaLifeThreatening"],
	),
	(
		"criteria_hospitalization",
		&["seriousness.criteriaHospitalization"],
	),
	("criteria_disabling", &["seriousness.criteriaDisabling"]),
	(
		"criteria_congenital_anomaly",
		&["seriousness.criteriaCongenitalAnomaly"],
	),
	(
		"criteria_other_medically_important",
		&["seriousness.criteriaOtherMedicallyImportant"],
	),
	("required_intervention", &["requiredIntervention"]),
	("expectedness", &["expectedness"]),
	("severity", &["severity"]),
	(
		"mfds_device_ae_classification",
		&["mfdsDeviceAe.aeClassification"],
	),
	("mfds_device_ae_outcome", &["mfdsDeviceAe.aeOutcome"]),
	(
		"mfds_device_cause_medical_device",
		&["mfdsDeviceAe.causeMedicalDevice"],
	),
	(
		"mfds_device_cause_procedure_issue",
		&["mfdsDeviceAe.causeProcedureIssue"],
	),
	(
		"mfds_device_cause_patient_condition",
		&["mfdsDeviceAe.causePatientCondition"],
	),
	(
		"mfds_device_cause_unable_to_assess",
		&["mfdsDeviceAe.causeUnableToAssess"],
	),
	("mfds_device_cause_other", &["mfdsDeviceAe.causeOther"]),
	("mfds_device_action_reason", &["mfdsDeviceAe.actionReason"]),
	("mfds_device_action_recall", &["mfdsDeviceAe.actionRecall"]),
	("mfds_device_action_repair", &["mfdsDeviceAe.actionRepair"]),
	(
		"mfds_device_action_inspection",
		&["mfdsDeviceAe.actionInspection"],
	),
	(
		"mfds_device_action_replacement",
		&["mfdsDeviceAe.actionReplacement"],
	),
	(
		"mfds_device_action_improvement",
		&["mfdsDeviceAe.actionImprovement"],
	),
	(
		"mfds_device_action_monitoring",
		&["mfdsDeviceAe.actionMonitoring"],
	),
	(
		"mfds_device_action_notification",
		&["mfdsDeviceAe.actionNotification"],
	),
	(
		"mfds_device_action_label_change",
		&["mfdsDeviceAe.actionLabelChange"],
	),
	("mfds_device_action_other", &["mfdsDeviceAe.actionOther"]),
	("start_date", &["reactionStartDate"]),
	("end_date", &["reactionEndDate"]),
	("duration_value", &["reactionDuration.value"]),
	("duration_unit", &["reactionDuration.unit"]),
	("outcome", &["outcome", "reactionOutcome"]),
	("medical_confirmation", &["medicalConfirmation"]),
	("country_code", &["reactionCountry"]),
	("sequence_number", &["sequenceNumber"]),
];

async fn load_editor_ae_list_rows(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	include_deleted: bool,
) -> Result<Vec<CaseEditorAeListRowDto>> {
	Ok(
		ReactionBmc::list_by_case_with_deleted(ctx, mm, case_id, include_deleted)
			.await?
			.into_iter()
			.map(|reaction| CaseEditorAeListRowDto {
				id: reaction.id,
				sequence_number: reaction.sequence_number,
				deleted: reaction.deleted,
				reaction_primary_source_native: reaction.primary_source_reaction,
				reaction_primary_source_translation: reaction
					.primary_source_reaction_translation,
				meddra_version: reaction.reaction_meddra_version,
				meddra_code: reaction.reaction_meddra_code,
				seriousness: reaction.serious,
			})
			.collect(),
	)
}

repeatable_list_handler!(
	list_editor_ae,
	CaseEditorAeListRowDto,
	load_editor_ae_list_rows,
	include_deleted,
);

pub async fn get_editor_ae_page_projection(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path(case_id): Path<Uuid>,
	Query(query): Query<CaseEditorPageProjectionQuery>,
) -> Result<(
	axum::http::StatusCode,
	Json<CaseEditorPageProjectionResponse>,
)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		"editor/AE",
		move |ctx, mm| {
			Box::pin(async move {
				let rows = load_editor_ae_list_rows(
					ctx,
					mm,
					case_id,
					query.include_deleted.unwrap_or(false),
				)
				.await?;
				let projection = repeatable_page_projection_response(
					case_id,
					"AE",
					query_authorities_csv(&query)?,
					json!({ "rows": rows }),
				)?;
				Ok((axum::http::StatusCode::OK, Json(projection)))
			})
		},
	)
	.await
}

pub async fn get_editor_ae(
	State(mm): State<ModelManager>,
	ctx_w: CtxW,
	snapshot: lib_web::middleware::mw_authorization_snapshot::AuthorizationSnapshotW,
	Path((case_id, reaction_id)): Path<(Uuid, Uuid)>,
) -> Result<(axum::http::StatusCode, Json<CaseEditorRowDetailResponse>)> {
	let ctx = ctx_w.0;
	lib_rest_core::with_authorized_case_child_read(
		&ctx,
		&snapshot,
		&mm,
		case_id,
		format!("editor/AE/{reaction_id}"),
		move |ctx, mm| {
			Box::pin(async move {
				let reaction =
					ReactionBmc::get_in_case(ctx, mm, case_id, reaction_id).await?;
				Ok((
					axum::http::StatusCode::OK,
					Json(CaseEditorRowDetailResponse {
						case_id,
						row_id: reaction_id,
						data: json!({ "reactions": [reaction] }),
					}),
				))
			})
		},
	)
	.await
}

repeatable_page_row_read_handler!(
	get_editor_ae_page_row,
	build_editor_ae_page_row_response,
);

async fn build_editor_ae_page_row_response(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
	row_id: Uuid,
	authorities: Option<String>,
) -> Result<Value> {
	let persisted = ReactionBmc::get_in_case(&ctx, &mm, case_id, row_id).await?;
	let start_date = ci_date(persisted.start_date);
	let end_date = ci_date(persisted.end_date);
	let mut reaction = json!(persisted);
	if let Value::Object(ref mut map) = reaction {
		map.insert("start_date".to_string(), json!(start_date));
		map.insert("end_date".to_string(), json!(end_date));
	}
	editor_page_row_response(
		case_id,
		"AE",
		row_id,
		authorities,
		json!({ "reaction": reaction }),
	)
}

repeatable_page_row_create_handler!(
	create_editor_ae_page_row,
	section: "AE",
	row_key: "reaction",
	bmc: ReactionBmc,
	model: ReactionForCreate,
	aliases: REACTION_ROW_ALIASES,
	extras: |case_id, row| [
		("case_id", json!(case_id)),
		(
			"sequence_number",
			json!(i32_field(row, &["sequenceNumber", "sequence_number"]).unwrap_or(1)),
		),
	],
	build_response: build_editor_ae_page_row_response,
);

repeatable_page_row_patch_handler!(
	patch_editor_ae_page_row,
	section: "AE",
	row_key: "reaction",
	bmc: ReactionBmc,
	model: ReactionForUpdate,
	aliases: REACTION_ROW_ALIASES,
	build_response: build_editor_ae_page_row_response,
);

repeatable_page_row_delete_restore_handlers!(
	delete: delete_editor_ae_page_row,
	restore: restore_editor_ae_page_row,
	bmc: ReactionBmc,
	build_response: build_editor_ae_page_row_response,
);
