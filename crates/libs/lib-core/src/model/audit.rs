// Audit Logs and Case Versions

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::filter::{FilterNodes, ListOptions, OpValsString};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- CaseVersion

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct CaseVersion {
	pub id: Uuid,
	pub case_id: Uuid,
	pub version: i32,
	pub snapshot: JsonValue, // Full case data snapshot
	pub changed_by: Uuid,
	pub change_reason: Option<String>,
	pub created_at: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct CaseVersionForCreate {
	pub case_id: Uuid,
	pub version: i32,
	pub snapshot: JsonValue,
	pub change_reason: Option<String>,
}

// -- AuditLog

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct AuditLog {
	pub id: i64,
	pub table_name: String,
	pub record_id: Uuid,
	pub action: String, // CREATE, UPDATE, DELETE, SUBMIT, NULLIFY
	pub user_id: Uuid,
	pub old_values: Option<JsonValue>,
	pub new_values: Option<JsonValue>,
	pub ip_address: Option<String>, // Stored as TEXT in DB
	pub user_agent: Option<String>,
	pub created_at: OffsetDateTime,
}

#[derive(Deserialize)]
pub struct AuditLogForCreate {
	pub table_name: String,
	pub record_id: Uuid,
	pub action: String,
	pub old_values: Option<JsonValue>,
	pub new_values: Option<JsonValue>,
	pub ip_address: Option<String>, // Stored as TEXT in DB
	pub user_agent: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct AuditLogFilter {
	pub table_name: Option<OpValsString>,
	pub action: Option<OpValsString>,
}

// -- BMCs

pub struct CaseVersionBmc;
impl DbBmc for CaseVersionBmc {
	const TABLE: &'static str = "case_versions";
}

impl CaseVersionBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		version_c: CaseVersionForCreate,
	) -> Result<Uuid> {
		let user_id = ctx.user_id();
		let sql = "INSERT INTO case_versions (case_id, version, snapshot, change_reason, changed_by) VALUES ($1, $2, $3, $4, $5) RETURNING id";

		let (id,) = sqlx::query_as::<_, (Uuid,)>(sql)
			.bind(version_c.case_id)
			.bind(version_c.version)
			.bind(version_c.snapshot)
			.bind(version_c.change_reason)
			.bind(user_id)
			.fetch_one(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;

		Ok(id)
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<CaseVersion>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY version DESC",
			Self::TABLE
		);
		let versions = sqlx::query_as::<_, CaseVersion>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(versions)
	}
}

pub struct AuditLogBmc;
impl DbBmc for AuditLogBmc {
	const TABLE: &'static str = "audit_logs";
}

impl AuditLogBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		audit_c: AuditLogForCreate,
	) -> Result<i64> {
		let user_id = ctx.user_id();
		let sql = "INSERT INTO audit_logs (table_name, record_id, action, user_id, old_values, new_values, ip_address, user_agent) VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id";

		let (id,) = sqlx::query_as::<_, (i64,)>(sql)
			.bind(audit_c.table_name)
			.bind(audit_c.record_id)
			.bind(audit_c.action)
			.bind(user_id)
			.bind(audit_c.old_values)
			.bind(audit_c.new_values)
			.bind(audit_c.ip_address)
			.bind(audit_c.user_agent)
			.fetch_one(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;

		Ok(id)
	}

	pub async fn list(
		_ctx: &Ctx,
		mm: &ModelManager,
		_filters: Option<Vec<AuditLogFilter>>,
		_list_options: Option<ListOptions>,
	) -> Result<Vec<AuditLog>> {
		// Simple implementation - can be enhanced with filters later
		let sql = "SELECT * FROM audit_logs ORDER BY created_at DESC LIMIT 1000";
		let logs = sqlx::query_as::<_, AuditLog>(sql)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(logs)
	}

	pub async fn list_by_record(
		_ctx: &Ctx,
		mm: &ModelManager,
		table_name: &str,
		record_id: Uuid,
	) -> Result<Vec<AuditLog>> {
		let sql = format!(
			"SELECT * FROM {} WHERE table_name = $1 AND record_id = $2 ORDER BY created_at DESC",
			Self::TABLE
		);
		let logs = sqlx::query_as::<_, AuditLog>(&sql)
			.bind(table_name)
			.bind(record_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(logs)
	}
}
