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
pub struct E2br3User {
	pub id: Uuid,
	pub organization_id: i64,
	pub email: String,
	pub username: String,

	// Auth fields (not serialized)
	#[serde(skip)]
	pub pwd: Option<String>,
	#[serde(skip)]
	pub pwd_salt: Uuid,
	#[serde(skip)]
	pub token_salt: Uuid,

	pub role: String,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub active: bool,
	pub last_login_at: Option<OffsetDateTime>,

	// Timestamps
	pub cid: i64,
	pub ctime: OffsetDateTime,
	pub mid: i64,
	pub mtime: OffsetDateTime,
}

#[derive(Fields, Deserialize)]
pub struct E2br3UserForCreate {
	pub organization_id: i64,
	pub email: String,
	pub username: String,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
}

#[derive(Fields, Deserialize)]
pub struct E2br3UserForUpdate {
	pub email: Option<String>,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub active: Option<bool>,
	pub last_login_at: Option<OffsetDateTime>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct E2br3UserFilter {
	pub organization_id: Option<OpValsInt64>,
	pub email: Option<OpValsString>,
	pub username: Option<OpValsString>,
	pub role: Option<OpValsString>,
}

// -- E2br3UserBmc

pub struct E2br3UserBmc;

impl DbBmc for E2br3UserBmc {
	const TABLE: &'static str = "users";
}

impl E2br3UserBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		user_c: E2br3UserForCreate,
	) -> Result<Uuid> {
		base_uuid::create::<Self, _>(ctx, mm, user_c).await
	}

	pub async fn get(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<E2br3User> {
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<E2br3UserFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<E2br3User>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		user_u: E2br3UserForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, user_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}

	pub async fn first_by_email(
		ctx: &Ctx,
		mm: &ModelManager,
		email: &str,
	) -> Result<Option<E2br3User>> {
		let users = Self::list(
			ctx,
			mm,
			Some(vec![E2br3UserFilter {
				email: Some(email.into()),
				..Default::default()
			}]),
			None,
		)
		.await?;

		Ok(users.into_iter().next())
	}
}
