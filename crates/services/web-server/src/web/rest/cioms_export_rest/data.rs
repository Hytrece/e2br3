use super::*;
use crate::runtime_settings;
use lib_core::model::patient::{MedicalHistoryEpisode, PastDrugHistory};

#[derive(Debug, sqlx::FromRow)]
struct CiomsFieldNotationRow {
	field_path: String,
	notation: String,
}

async fn load_list_by_patient<T>(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	table: &'static str,
	patient_id: Uuid,
) -> Result<Vec<T>>
where
	for<'r> T: sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
	let sql = format!(
		"SELECT * FROM {table} WHERE patient_id = $1 AND deleted IS NOT TRUE ORDER BY sequence_number"
	);
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(sqlx::query_as::<_, T>(&sql).bind(patient_id))
				.await
				.map_err(ModelError::Dbx)
				.map_err(Error::Model)
		})
	})
	.await
}

#[derive(Clone, Copy)]
enum CiomsCaseTable {
	SafetyReportIdentification,
	PatientInformation,
	NarrativeInformation,
	PrimarySources,
	SenderInformation,
}

impl CiomsCaseTable {
	fn as_str(self) -> &'static str {
		match self {
			Self::SafetyReportIdentification => "safety_report_identification",
			Self::PatientInformation => "patient_information",
			Self::NarrativeInformation => "narrative_information",
			Self::PrimarySources => "primary_sources",
			Self::SenderInformation => "sender_information",
		}
	}
}

pub(super) async fn load_cioms_settings(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
) -> Result<CiomsSettings> {
	let settings = runtime_settings::load(ctx, mm).await?;
	Ok(CiomsSettings {
		orientation: settings.orientation,
		data_ordering: settings.data_ordering,
		notation: settings.notation,
	})
}

async fn load_optional_by_case<T>(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	table: CiomsCaseTable,
	case_id: Uuid,
) -> Result<Option<T>>
where
	for<'r> T: sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
	let table = table.as_str();
	let sql = format!("SELECT * FROM {table} WHERE case_id = $1 LIMIT 1");
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_optional(sqlx::query_as::<_, T>(&sql).bind(case_id))
				.await
				.map_err(ModelError::Dbx)
				.map_err(Error::Model)
		})
	})
	.await
}

async fn load_list_by_case<T>(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	table: CiomsCaseTable,
	case_id: Uuid,
) -> Result<Vec<T>>
where
	for<'r> T: sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
	let table = table.as_str();
	let sql = format!(
		"SELECT * FROM {table} WHERE case_id = $1 AND deleted IS NOT TRUE ORDER BY sequence_number"
	);
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(sqlx::query_as::<_, T>(&sql).bind(case_id))
				.await
				.map_err(ModelError::Dbx)
				.map_err(Error::Model)
		})
	})
	.await
}

async fn load_unordered_list_by_case<T>(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	table: CiomsCaseTable,
	case_id: Uuid,
) -> Result<Vec<T>>
where
	for<'r> T: sqlx::FromRow<'r, sqlx::postgres::PgRow> + Send + Unpin,
{
	let table = table.as_str();
	let sql = format!("SELECT * FROM {table} WHERE case_id = $1 ORDER BY id");
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(sqlx::query_as::<_, T>(&sql).bind(case_id))
				.await
				.map_err(ModelError::Dbx)
				.map_err(Error::Model)
		})
	})
	.await
}

pub(super) async fn load_dosages_by_case(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<DosageInformation>> {
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(
				sqlx::query_as::<_, DosageInformation>(
					"SELECT dosage_information.*
				 FROM dosage_information
				 JOIN drug_information ON drug_information.id = dosage_information.drug_id
				 WHERE drug_information.case_id = $1
				   AND drug_information.deleted IS NOT TRUE
				   AND dosage_information.deleted IS NOT TRUE
				 ORDER BY drug_information.sequence_number, dosage_information.sequence_number",
				)
				.bind(case_id),
			)
			.await
			.map_err(ModelError::Dbx)
			.map_err(Error::Model)
		})
	})
	.await
}

pub(super) async fn load_indications_by_case(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<DrugIndication>> {
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(
				sqlx::query_as::<_, DrugIndication>(
					"SELECT drug_indications.*
				 FROM drug_indications
				 JOIN drug_information ON drug_information.id = drug_indications.drug_id
				 WHERE drug_information.case_id = $1
				   AND drug_information.deleted IS NOT TRUE
				   AND drug_indications.deleted IS NOT TRUE
				 ORDER BY drug_information.sequence_number, drug_indications.sequence_number",
				)
				.bind(case_id),
			)
			.await
			.map_err(ModelError::Dbx)
			.map_err(Error::Model)
		})
	})
	.await
}

async fn load_causality_rows_by_case(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<CiomsDrugReactionCausalityRow>> {
	lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(
				sqlx::query_as::<_, CiomsDrugReactionCausalityRow>(
					"SELECT dra.drug_id,
				        dra.reaction_id,
				        di.drug_characterization,
				        dra.administration_start_interval_value,
				        dra.administration_start_interval_unit,
				        dra.last_dose_interval_value,
				        dra.last_dose_interval_unit,
				        dra.recurrence_action,
				        dra.reaction_recurred,
				        dra.dechallenge_result,
				        ra.sequence_number AS relatedness_sequence_number,
				        ra.source_of_assessment AS relatedness_source,
				        ra.method_of_assessment AS relatedness_method,
				        ra.method_of_assessment_kr1 AS relatedness_method_kr1,
				        ra.result_of_assessment AS relatedness_result,
				        ra.result_of_assessment_kr1 AS relatedness_result_kr1,
				        ra.result_of_assessment_kr2 AS relatedness_result_kr2
				 FROM drug_reaction_assessments dra
				 JOIN drug_information di ON di.id = dra.drug_id
				 JOIN reactions r ON r.id = dra.reaction_id
				 LEFT JOIN relatedness_assessments ra
				   ON ra.drug_reaction_assessment_id = dra.id
				  AND ra.deleted IS NOT TRUE
				 WHERE di.case_id = $1
				   AND di.deleted IS NOT TRUE
				   AND r.case_id = $1
				   AND r.deleted IS NOT TRUE
				 ORDER BY di.sequence_number, r.sequence_number, dra.id, ra.sequence_number",
				)
				.bind(case_id),
			)
			.await
			.map_err(ModelError::Dbx)
			.map_err(Error::Model)
		})
	})
	.await
}

pub(super) async fn load_cioms_case_data(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<CiomsCaseData> {
	let report = load_optional_by_case::<SafetyReportIdentification>(
		ctx,
		mm,
		CiomsCaseTable::SafetyReportIdentification,
		case_id,
	)
	.await?;
	let case_number = report
		.as_ref()
		.and_then(|report| report.safety_report_id.clone())
		.filter(|value| !value.trim().is_empty())
		.ok_or_else(|| Error::BadRequest {
			message: format!("case {case_id} has no safety report ID"),
		})?;
	let patient = load_optional_by_case::<PatientInformation>(
		ctx,
		mm,
		CiomsCaseTable::PatientInformation,
		case_id,
	)
	.await?;
	let narrative = load_optional_by_case::<NarrativeInformation>(
		ctx,
		mm,
		CiomsCaseTable::NarrativeInformation,
		case_id,
	)
	.await?;
	let reactions = ReactionBmc::list_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::Model)?;
	let drugs = DrugInformationBmc::list_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::Model)?;
	let dosages = load_dosages_by_case(ctx, mm, case_id).await?;
	let indications = load_indications_by_case(ctx, mm, case_id).await?;
	let test_results = TestResultBmc::list_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::Model)?;
	let causality_rows = load_causality_rows_by_case(ctx, mm, case_id).await?;
	let (medical_history_episodes, past_drug_history) = match patient.as_ref() {
		Some(patient) => (
			load_list_by_patient::<MedicalHistoryEpisode>(
				ctx,
				mm,
				"medical_history_episodes",
				patient.id,
			)
			.await?,
			load_list_by_patient::<PastDrugHistory>(
				ctx,
				mm,
				"past_drug_history",
				patient.id,
			)
			.await?,
		),
		None => (Vec::new(), Vec::new()),
	};
	let field_notations = lib_rest_core::with_rls_read(mm, ctx, |dbx| {
		Box::pin(async move {
			dbx.fetch_all(sqlx::query_as::<_, CiomsFieldNotationRow>(
				"SELECT field_path, notation FROM case_field_notations WHERE case_id = $1 ORDER BY field_path, record_id",
			)
			.bind(case_id))
			.await
			.map_err(ModelError::Dbx)
			.map_err(Error::Model)
		})
	})
	.await?
	.into_iter()
	.map(|row| CiomsFieldNotation {
		field_path: row.field_path,
		notation: row.notation,
	})
	.collect();
	let primary_sources = load_list_by_case::<PrimarySource>(
		ctx,
		mm,
		CiomsCaseTable::PrimarySources,
		case_id,
	)
	.await?;
	let senders = load_unordered_list_by_case::<SenderInformation>(
		ctx,
		mm,
		CiomsCaseTable::SenderInformation,
		case_id,
	)
	.await?;
	Ok(CiomsCaseData {
		case_number,
		report,
		patient,
		reactions,
		drugs,
		dosages,
		indications,
		test_results,
		primary_sources,
		senders,
		narrative,
		field_notations,
		causality_rows,
		medical_history_episodes,
		past_drug_history,
	})
}
