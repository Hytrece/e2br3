// region:    --- Modules

mod error;

pub use self::error::{Error, Result};

// endregion: --- Modules

#[cfg_attr(feature = "with-rpc", derive(rpc_router::RpcResource))]
#[derive(Clone, Debug)]
pub struct Ctx {
	user_id: i64,
	user_uuid: uuid::Uuid,

	/// Note: For the future ACS (Access Control System)
	conv_id: Option<i64>,
}

// Constructors.
impl Ctx {
	pub fn root_ctx() -> Self {
		Ctx {
			user_id: 0,
			user_uuid: uuid::Uuid::nil(),
			conv_id: None,
		}
	}

	pub fn new(user_id: i64) -> Result<Self> {
		Self::new_with_ids(user_id, uuid::Uuid::nil())
	}

	pub fn new_with_uuid(user_uuid: uuid::Uuid) -> Result<Self> {
		if user_uuid.is_nil() {
			return Err(Error::CtxCannotNewNilUuid);
		}

		Ok(Self {
			user_id: 0,
			user_uuid,
			conv_id: None,
		})
	}

	pub fn new_with_ids(user_id: i64, user_uuid: uuid::Uuid) -> Result<Self> {
		if user_id == 0 && user_uuid.is_nil() {
			return Err(Error::CtxCannotNewRootCtx);
		}

		Ok(Self {
			user_id,
			user_uuid,
			conv_id: None,
		})
	}

	/// Note: For the future ACS (Access Control System)
	pub fn add_conv_id(&self, conv_id: i64) -> Ctx {
		let mut ctx = self.clone();
		ctx.conv_id = Some(conv_id);
		ctx
	}
}

// Property Accessors.
impl Ctx {
	pub fn user_id(&self) -> i64 {
		self.user_id
	}

	pub fn user_uuid(&self) -> uuid::Uuid {
		self.user_uuid
	}

	//. /// Note: For the future ACS (Access Control System)
	pub fn conv_id(&self) -> Option<i64> {
		self.conv_id
	}
}
