use crate::ctx::Ctx;
use crate::model::base::{prep_fields_for_update, DbBmc};
use crate::model::base_uuid;
use crate::model::{Error, ModelManager, Result};
use lib_auth::pwd::{self, ContentToHash};
use modql::field::{Fields, HasSeaFields, SeaField, SeaFields};
use modql::filter::{FilterNodes, ListOptions, OpValsInt64, OpValsString};
use sea_query::{Expr, Iden, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use sqlx::FromRow;

// -- Types

#[derive(Debug, Clone, Fields, FromRow, Serialize)]
pub struct User {
	pub id: Uuid,
	pub audit_id: i64, // For audit trail compatibility (cid/mid/owner_id)
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

#[derive(Deserialize)]
pub struct UserForCreate {
	pub organization_id: i64,
	pub email: String,
	pub username: String,
	pub pwd_clear: String,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
}

#[derive(Fields)]
pub struct UserForInsert {
	pub organization_id: i64,
	pub email: String,
	pub username: String,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForLogin {
	pub id: Uuid,
	pub audit_id: i64,
	pub organization_id: i64,
	pub email: String,
	pub username: String,

	// -- pwd and token info
	pub pwd: Option<String>, // encrypted
	pub pwd_salt: Uuid,
	pub token_salt: Uuid,
}

#[derive(Clone, FromRow, Fields, Debug)]
pub struct UserForAuth {
	pub id: Uuid,
	pub audit_id: i64,
	pub organization_id: i64,
	pub email: String,
	pub username: String,

	// -- token info
	pub token_salt: Uuid,
}

#[derive(Fields, Deserialize)]
pub struct UserForUpdate {
	pub email: Option<String>,
	pub role: Option<String>,
	pub first_name: Option<String>,
	pub last_name: Option<String>,
	pub active: Option<bool>,
	pub last_login_at: Option<OffsetDateTime>,
}

#[derive(FilterNodes, Deserialize, Default)]
pub struct UserFilter {
	pub organization_id: Option<OpValsInt64>,
	pub email: Option<OpValsString>,
	pub username: Option<OpValsString>,
	pub role: Option<OpValsString>,
}

/// Marker trait for different User representations
pub trait UserBy: HasSeaFields + for<'r> FromRow<'r, PgRow> + Unpin + Send {}

impl UserBy for User {}
impl UserBy for UserForLogin {}
impl UserBy for UserForAuth {}

#[derive(Iden)]
enum UserIden {
	Id,
	Email,
	Pwd,
}

// -- UserBmc

pub struct UserBmc;

impl DbBmc for UserBmc {
	const TABLE: &'static str = "users";
}

impl UserBmc {
	pub async fn create(
		ctx: &Ctx,
		mm: &ModelManager,
		user_c: UserForCreate,
	) -> Result<Uuid> {
		let UserForCreate {
			organization_id,
			email,
			username,
			pwd_clear,
			role,
			first_name,
			last_name,
		} = user_c;

		// -- Create the user row
		let user_fi = UserForInsert {
			organization_id,
			email: email.clone(),
			username,
			role,
			first_name,
			last_name,
		};

		// Start the transaction
		let mm = mm.new_with_txn()?;

		mm.dbx().begin_txn().await?;

		let user_id = base_uuid::create::<Self, _>(ctx, &mm, user_fi)
			.await
			.map_err(|model_error| {
				Error::resolve_unique_violation(
					model_error,
					Some(|table: &str, constraint: &str| {
						if table == "users" && constraint.contains("email") {
							Some(Error::UserAlreadyExists { email })
						} else {
							None
						}
					}),
				)
			})?;

		// -- Update the password
		Self::update_pwd(ctx, &mm, user_id, &pwd_clear).await?;

		// Commit the transaction
		mm.dbx().commit_txn().await?;

		Ok(user_id)
	}

	pub async fn get<E>(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<E>
	where
		E: UserBy,
	{
		base_uuid::get::<Self, _>(ctx, mm, id).await
	}

	pub async fn list(
		ctx: &Ctx,
		mm: &ModelManager,
		filters: Option<Vec<UserFilter>>,
		list_options: Option<ListOptions>,
	) -> Result<Vec<User>> {
		base_uuid::list::<Self, _, _>(ctx, mm, filters, list_options).await
	}

	pub async fn update(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		user_u: UserForUpdate,
	) -> Result<()> {
		base_uuid::update::<Self, _>(ctx, mm, id, user_u).await
	}

	pub async fn delete(ctx: &Ctx, mm: &ModelManager, id: Uuid) -> Result<()> {
		base_uuid::delete::<Self>(ctx, mm, id).await
	}

	pub async fn first_by_email<E>(
		_ctx: &Ctx,
		mm: &ModelManager,
		email: &str,
	) -> Result<Option<E>>
	where
		E: UserBy,
	{
		// -- Build query
		let mut query = Query::select();
		query
			.from(Self::table_ref())
			.columns(E::sea_idens())
			.and_where(Expr::col(UserIden::Email).eq(email));

		// -- Execute query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);

		let sqlx_query = sqlx::query_as_with::<_, E, _>(&sql, values);
		let entity = mm.dbx().fetch_optional(sqlx_query).await?;

		Ok(entity)
	}

	pub async fn update_pwd(
		ctx: &Ctx,
		mm: &ModelManager,
		id: Uuid,
		pwd_clear: &str,
	) -> Result<()> {
		// -- Prep password
		let user: UserForLogin = Self::get(ctx, mm, id).await?;
		let pwd = pwd::hash_pwd(ContentToHash {
			content: pwd_clear.to_string(),
			salt: user.pwd_salt,
		})
		.await?;

		// -- Prep the data
		let mut fields =
			SeaFields::new(vec![SeaField::new(UserIden::Pwd, pwd)]);
		prep_fields_for_update::<Self>(&mut fields, ctx.user_audit_id());

		// -- Build query
		let fields = fields.for_sea_update();
		let mut query = Query::update();
		query
			.table(Self::table_ref())
			.values(fields)
			.and_where(Expr::col(UserIden::Id).eq(id));

		// -- Exec query
		let (sql, values) = query.build_sqlx(PostgresQueryBuilder);
		let sqlx_query = sqlx::query_with(&sql, values);
		let _count = mm.dbx().execute(sqlx_query).await?;

		Ok(())
	}
}
