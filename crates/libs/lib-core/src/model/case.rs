use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::base_uuid;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, ListOptions, OpValsInt64, OpValsString};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- Types

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Case {
	pub id: Uuid,
	pub organization_id: i64,

	// E2B fields
	pub safety_report_id: String,
	pub version: i32,
	pub status: String,

	// Workflow
	pub created_by: Uuid,
	pub updated_by: Option<Uuid>,
	pub submitted_by: Option<Uuid>,
	pub submitted_at: Option<OffsetDateTime>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct CaseForCreate {
	pub organization_id: i64,
	pub safety_report_id: String,
	pub status: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct CaseForUpdate {
	pub safety_report_id: Option<String>,
	pub status: Option<String>,
	pub updated_by: Option<Uuid>,
	pub submitted_by: Option<Uuid>,
	pub submitted_at: Option<OffsetDateTime>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct CaseFilter {
	pub organization_id: Option<OpValsInt64>,
	pub safety_report_id: Option<OpValsString>,
	pub status: Option<OpValsString>,
}

// -- CaseBmc (Business Model Controller)

pub struct CaseBmc;

impl DbBmc for CaseBmc {
	const TABLE: &'static str = "cases";
}

impl CaseBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		case_c: CaseForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, case_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Case> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<CaseFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Case>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		case_u: CaseForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, case_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}
}
