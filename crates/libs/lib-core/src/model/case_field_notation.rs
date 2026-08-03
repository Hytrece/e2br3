use crate::ctx::Ctx;
use crate::model::store::set_full_context_from_ctx_dbx;
use crate::model::{ModelManager, Result};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct CaseFieldNotation {
	pub id: Uuid,
	pub notation: String,
}

pub struct CaseFieldNotationBmc;

impl CaseFieldNotationBmc {
	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		record_id: Option<Uuid>,
		field_path: &str,
	) -> Result<Option<CaseFieldNotation>> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.fetch_optional(
				sqlx::query_as::<_, CaseFieldNotation>(
					"SELECT id, notation FROM case_field_notations
					 WHERE case_id = $1 AND record_id IS NOT DISTINCT FROM $2
					   AND field_path = $3",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(field_path),
			)
			.await;
		match result {
			Ok(row) => {
				mm.dbx().commit_txn().await?;
				Ok(row)
			}
			Err(err) => {
				let _ = mm.dbx().rollback_txn().await;
				Err(err.into())
			}
		}
	}

	pub async fn upsert(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		record_id: Option<Uuid>,
		field_path: &str,
		notation: &str,
	) -> Result<CaseFieldNotation> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.fetch_one(
				sqlx::query_as::<_, CaseFieldNotation>(
					"INSERT INTO case_field_notations
					 (case_id, record_id, field_path, notation, created_by, updated_by)
					 VALUES ($1, $2, $3, $4, $5, $5)
					 ON CONFLICT (case_id, record_id, field_path)
					 DO UPDATE SET notation = EXCLUDED.notation, updated_by = EXCLUDED.updated_by
					 RETURNING id, notation",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(field_path)
				.bind(notation)
				.bind(ctx.user_id()),
			)
			.await;
		match result {
			Ok(row) => {
				mm.dbx().commit_txn().await?;
				Ok(row)
			}
			Err(err) => {
				let _ = mm.dbx().rollback_txn().await;
				Err(err.into())
			}
		}
	}

	pub async fn delete(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		record_id: Option<Uuid>,
		field_path: &str,
	) -> Result<()> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.execute(
				sqlx::query(
					"DELETE FROM case_field_notations
					 WHERE case_id = $1 AND record_id IS NOT DISTINCT FROM $2
					   AND field_path = $3",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(field_path),
			)
			.await;
		match result {
			Ok(_) => {
				mm.dbx().commit_txn().await?;
				Ok(())
			}
			Err(err) => {
				let _ = mm.dbx().rollback_txn().await;
				Err(err.into())
			}
		}
	}
}
