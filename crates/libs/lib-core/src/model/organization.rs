use crate::ctx::Ctx;
use crate::model::base::{self, DbBmc};
use crate::model::ModelManager;
use crate::model::Result;
use modql::field::Fields;
use modql::filter::{
	FilterNodes, ListOptions, OpValsBool, OpValsInt64, OpValsString,
};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::FromRow;

// -- Types

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct Organization {
	pub id: i64,
	pub name: String,
	#[serde(rename = "type")]
	pub org_type: Option<String>,
	pub address: Option<String>,
	pub city: Option<String>,
	pub state: Option<String>,
	pub postcode: Option<String>,
	pub country_code: Option<String>,
	pub contact_email: Option<String>,
	pub contact_phone: Option<String>,
	pub active: bool,

	// Timestamps
	pub cid: i64,
	pub ctime: OffsetDateTime,
	pub mid: i64,
	pub mtime: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct OrganizationForCreate {
	pub name: String,
	#[serde(rename = "type")]
	pub org_type: Option<String>,
	pub address: Option<String>,
	pub contact_email: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct OrganizationForUpdate {
	pub name: Option<String>,
	#[serde(rename = "type")]
	pub org_type: Option<String>,
	pub address: Option<String>,
	pub city: Option<String>,
	pub state: Option<String>,
	pub postcode: Option<String>,
	pub country_code: Option<String>,
	pub contact_email: Option<String>,
	pub contact_phone: Option<String>,
	pub active: Option<bool>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct OrganizationFilter {
	pub id: Option<OpValsInt64>,
	pub name: Option<OpValsString>,
	pub active: Option<OpValsBool>,
}

// -- OrganizationBmc

pub struct OrganizationBmc;

impl DbBmc for OrganizationBmc {
	const TABLE: &'static str = "organizations";
}

impl OrganizationBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		org_c: OrganizationForCreate,
	) -> Result<i64> {
		base::create::<Self, _>(ctx, mm, org_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<Organization> {
		base::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<OrganizationFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<Organization>> {
		base::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: i64,
		org_u: OrganizationForUpdate,
	) -> Result<()> {
		base::update::<Self, _>(ctx, mm, id, org_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: i64) -> Result<()> {
		base::delete::<Self>(ctx, mm, id).await
	}
}
