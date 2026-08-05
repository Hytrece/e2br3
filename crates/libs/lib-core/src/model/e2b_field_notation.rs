use crate::ctx::Ctx;
use crate::model::store::set_full_context_from_ctx_dbx;
use crate::model::{ModelManager, Result};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct E2bFieldNotation {
	pub id: Uuid,
	pub record_id: Option<Uuid>,
	pub e2b_code: String,
	pub notation: String,
}

pub struct E2bFieldNotationBmc;

impl E2bFieldNotationBmc {
	pub async fn get(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
		record_id: Option<Uuid>,
		e2b_code: &str,
	) -> Result<Option<E2bFieldNotation>> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.fetch_optional(
				sqlx::query_as::<_, E2bFieldNotation>(
					"SELECT id, record_id, e2b_code, notation
					 FROM case_e2b_field_notations
					 WHERE case_id = $1 AND record_id IS NOT DISTINCT FROM $2
					   AND e2b_code = $3",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(e2b_code),
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

	pub async fn list_by_case(
		ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<E2bFieldNotation>> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.fetch_all(
				sqlx::query_as::<_, E2bFieldNotation>(
					"SELECT id, record_id, e2b_code, notation
					 FROM case_e2b_field_notations
					 WHERE case_id = $1 ORDER BY e2b_code, record_id",
				)
				.bind(case_id),
			)
			.await;
		match result {
			Ok(rows) => {
				mm.dbx().commit_txn().await?;
				Ok(rows)
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
		e2b_code: &str,
		notation: &str,
	) -> Result<E2bFieldNotation> {
		mm.dbx().begin_txn().await?;
		if let Err(err) = set_full_context_from_ctx_dbx(mm.dbx(), ctx).await {
			let _ = mm.dbx().rollback_txn().await;
			return Err(err);
		}
		let result = mm
			.dbx()
			.fetch_one(
				sqlx::query_as::<_, E2bFieldNotation>(
					"INSERT INTO case_e2b_field_notations
					 (case_id, record_id, e2b_code, notation, created_by, updated_by)
					 VALUES ($1, $2, $3, $4, $5, $5)
					 ON CONFLICT (case_id, record_id, e2b_code)
					 DO UPDATE SET notation = EXCLUDED.notation, updated_by = EXCLUDED.updated_by
					 RETURNING id, record_id, e2b_code, notation",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(e2b_code)
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
		e2b_code: &str,
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
					"DELETE FROM case_e2b_field_notations
					 WHERE case_id = $1 AND record_id IS NOT DISTINCT FROM $2
					   AND e2b_code = $3",
				)
				.bind(case_id)
				.bind(record_id)
				.bind(e2b_code),
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
