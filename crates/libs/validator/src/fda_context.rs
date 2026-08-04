use lib_core::ctx::Ctx;
use lib_core::model::drug::{FdaDeviceCode, FdaDeviceInformation};
use lib_core::model::safety_report::{
	StudyFdaCrossReportedInd, StudyInformation, StudyRegistrationNumber,
};
use lib_core::model::store::set_full_context_dbx_or_rollback;
use lib_core::model::{ModelManager, Result};
use sqlx::types::Uuid;

#[derive(Debug, Clone)]
pub struct FdaValidationContext {
	pub studies: Vec<StudyInformation>,
	pub cross_reported_inds: Vec<StudyFdaCrossReportedInd>,
	pub has_prior_submission: bool,
}

pub async fn load_fda_validation_context(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: Uuid,
) -> Result<FdaValidationContext> {
	mm.dbx().begin_txn().await?;
	set_full_context_dbx_or_rollback(
		mm.dbx(),
		ctx.user_id(),
		ctx.organization_id(),
		ctx.role(),
	)
	.await?;
	let studies = mm
		.dbx()
		.fetch_all(
			sqlx::query_as::<_, StudyInformation>(
				"SELECT * FROM study_information WHERE case_id = $1 ORDER BY created_at, id",
			)
			.bind(case_id),
		)
		.await?;
	let cross_reported_inds = mm
		.dbx()
		.fetch_all(
			sqlx::query_as::<_, StudyFdaCrossReportedInd>(
				"SELECT cross_ind.*
				   FROM study_fda_cross_reported_inds cross_ind
				   JOIN study_information study ON study.id = cross_ind.study_information_id
				  WHERE study.case_id = $1 AND cross_ind.deleted = false
				  ORDER BY cross_ind.sequence_number, cross_ind.created_at, cross_ind.id",
			)
			.bind(case_id),
		)
		.await?;
	let (has_prior_submission,) = mm
		.dbx()
		.fetch_one(
			sqlx::query_as::<_, (bool,)>(
				"SELECT EXISTS (SELECT 1 FROM case_submissions WHERE case_id = $1)",
			)
			.bind(case_id),
		)
		.await?;
	mm.dbx().commit_txn().await?;
	Ok(FdaValidationContext {
		studies,
		cross_reported_inds,
		has_prior_submission,
	})
}

pub async fn list_study_registrations(
	ctx: &Ctx,
	mm: &ModelManager,
	study_id: Uuid,
) -> Result<Vec<StudyRegistrationNumber>> {
	let sql = "SELECT * FROM study_registration_numbers WHERE study_information_id = $1 AND deleted = false ORDER BY sequence_number";
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
