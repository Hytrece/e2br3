use lib_core::ctx::Ctx;
use lib_core::model::drug::{FdaDeviceCode, FdaDeviceInformation};
use lib_core::model::safety_report::{StudyInformation, StudyRegistrationNumber};
use lib_core::model::store::set_full_context_dbx_or_rollback;
use lib_core::model::{ModelManager, Result};
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct FdaValidationContext {
	pub studies: Vec<StudyInformation>,
}

pub async fn load_fda_validation_context(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<FdaValidationContext> {
	let studies = list_studies(ctx, mm, case_id).await?;
	Ok(FdaValidationContext { studies })
}

pub async fn list_study_registrations(
	ctx: &Ctx,
	mm: &ModelManager,
	study_id: Uuid,
) -> Result<Vec<StudyRegistrationNumber>> {
	let sql = "SELECT * FROM study_registration_numbers WHERE study_information_id = $1 ORDER BY sequence_number";
	mm.dbx().begin_txn().await?;
	set_full_context_dbx_or_rollback(
		mm.dbx(),
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await?;
	let rows = mm
		.dbx()
		.fetch_all(sqlx::query_as::<_, StudyRegistrationNumber>(sql).bind(study_id))
		.await?;
	mm.dbx().commit_txn().await?;
	Ok(rows)
}

pub async fn list_fda_devices(
	ctx: &Ctx,
	mm: &ModelManager,
	drug_id: Uuid,
) -> Result<(Vec<FdaDeviceInformation>, Vec<FdaDeviceCode>)> {
	mm.dbx().begin_txn().await?;
	set_full_context_dbx_or_rollback(
		mm.dbx(),
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await?;
	let devices = mm
		.dbx()
		.fetch_all(
			sqlx::query_as::<_, FdaDeviceInformation>(
				"SELECT * FROM fda_device_information WHERE drug_id = $1 AND deleted = false ORDER BY sequence_number",
			)
			.bind(drug_id),
		)
		.await?;
	let device_ids: Vec<_> = devices.iter().map(|device| device.id).collect();
	let codes = if device_ids.is_empty() {
		Vec::new()
	} else {
		mm.dbx()
			.fetch_all(
				sqlx::query_as::<_, FdaDeviceCode>(
					"SELECT * FROM fda_device_codes WHERE device_id = ANY($1) AND deleted = false ORDER BY device_id, element, sequence_number",
				)
				.bind(&device_ids),
			)
			.await?
	};
	mm.dbx().commit_txn().await?;
	Ok((devices, codes))
}

async fn list_studies(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<Vec<StudyInformation>> {
	let sql =
		"SELECT * FROM study_information WHERE case_id = $1 ORDER BY created_at, id";
	mm.dbx().begin_txn().await?;
	set_full_context_dbx_or_rollback(
		mm.dbx(),
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await?;
	let rows = mm
		.dbx()
		.fetch_all(sqlx::query_as::<_, StudyInformation>(sql).bind(case_id))
		.await?;
	mm.dbx().commit_txn().await?;
	Ok(rows)
}
