// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone, Debug)]
pub struct Ctx {
	user_audit_id: i64,
	user_id: uuid::Uuid,
}

// Constructors.
impl Ctx {
	pub fn root_ctx() -> Self {
		Ctx {
			user_audit_id: 0,
			user_id: uuid::Uuid::nil(),
		}
	}

	pub fn new(user_audit_id: i64) -> Result<Self> {
		Self::new_with_ids(user_audit_id, uuid::Uuid::nil())
	}

	pub fn new_with_uuid(user_id: uuid::Uuid) -> Result<Self> {
		if user_id.is_nil() {
			return Err(Error::CtxCannotNewNilUuid);
		}

		Ok(Self {
			user_audit_id: 0,
			user_id,
		})
	}

	pub fn new_with_ids(user_audit_id: i64, user_id: uuid::Uuid) -> Result<Self> {
		if user_audit_id == 0 && user_id.is_nil() {
			return Err(Error::CtxCannotNewRootCtx);
		}

		Ok(Self {
			user_audit_id,
			user_id,
		})
	}

}

// Property Accessors.
impl Ctx {
	pub fn user_audit_id(&self) -> i64 {
		self.user_audit_id
	}

	pub fn user_id(&self) -> uuid::Uuid {
		self.user_id
	}
}
