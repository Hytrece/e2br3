// Section E - Reaction/Event

use crate::ctx::Ctx;
use crate::model::base::DbBmc;
use crate::model::base_uuid;
use crate::model::store::dbx;
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{FilterNodes, OpValsBool, OpValsValue};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::types::time::{Date, OffsetDateTime};
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- Reaction

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Reaction {
	pub id: Uuid,
	pub case_id: Uuid,
	pub sequence_number: i32,

	// E.i.1.1 - Reaction as reported
	pub primary_source_reaction: String,
	pub reaction_language: Option<String>,

	// E.i.2.1 - MedDRA coding
	pub reaction_meddra_version: Option<String>,
	pub reaction_meddra_code: Option<String>,

	// E.i.3 - Term Highlighted by Reporter
	pub term_highlighted: Option<bool>,

	// E.i.3.1 - Seriousness (MANDATORY if serious)
	pub serious: Option<bool>,

	// E.i.3.2 - Seriousness Criteria
	pub criteria_death: bool,
	pub criteria_life_threatening: bool,
	pub criteria_hospitalization: bool,
	pub criteria_disabling: bool,
	pub criteria_congenital_anomaly: bool,
	pub criteria_other_medically_important: bool,

	// E.i.4-6 - Timing
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,
	pub duration_value: Option<Decimal>,
	pub duration_unit: Option<String>,

	// E.i.7 - Outcome
	pub outcome: Option<String>,

	// E.i.8 - Medical Confirmation
	pub medical_confirmation: Option<bool>,

	// E.i.9 - Country
	pub country_code: Option<String>,

	// Timestamps
	pub created_at: OffsetDateTime,
	pub updated_at: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct ReactionForCreate {
	pub case_id: Uuid,
	pub sequence_number: i32,
	pub primary_source_reaction: String,
}

#[derive(Fields, Deserialize)]
pub struct ReactionForUpdate {
	pub primary_source_reaction: Option<String>,
	pub reaction_meddra_code: Option<String>,
	pub reaction_meddra_version: Option<String>,
	pub serious: Option<bool>,
	pub criteria_death: Option<bool>,
	pub criteria_life_threatening: Option<bool>,
	pub criteria_hospitalization: Option<bool>,
	pub start_date: Option<Date>,
	pub end_date: Option<Date>,
	pub outcome: Option<String>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct ReactionFilter {
	pub case_id: Option<OpValsValue>,
	pub serious: Option<OpValsBool>,
}

// -- BMC

pub struct ReactionBmc;
impl DbBmc for ReactionBmc {
	const TABLE: &'static str = "reactions";
}

impl ReactionBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		reaction_c: ReactionForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, reaction_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<Reaction> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		reaction_u: ReactionForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, reaction_u).await
	}

	pub async fn list_by_case(
		_ctx: &Ctx,
		mm: &ModelManager,
		case_id: Uuid,
	) -> Result<Vec<Reaction>> {
		let sql = format!(
			"SELECT * FROM {} WHERE case_id = $1 ORDER BY sequence_number",
			Self::TABLE
		);
		let reactions = sqlx::query_as::<_, Reaction>(&sql)
			.bind(case_id)
			.fetch_all(mm.dbx().db())
			.await
			.map_err(|e| dbx::Error::from(e))?;
		Ok(reactions)
	}
}
