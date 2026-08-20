use crate::middleware;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use derive_more::From;
use lib_auth::{pwd, token};
use lib_core::model;
use serde::Serialize;
use serde_with::{serde_as, DisplayFromStr};
use std::sync::Arc;
use tracing::debug;
use uuid::Uuid;

pub type Result<T> = core::result::Result<T, Error>;

#[serde_as]
#[derive(Debug, Serialize, From, strum_macros::AsRefStr)]
#[serde(tag = "type", content = "data")]
pub enum Error {
	// -- Login
	LoginFailUsernameNotFound,
	LoginFailEmailNotFound,
	LoginFailUserHasNoPwd {
		user_id: Uuid,
	},
	LoginFailPwdNotMatching {
		user_id: Uuid,
	},
	LoginFailUserCtxCreate {
		user_id: Uuid,
	},

	// -- Authorization
	AccessDenied {
		required_role: String,
	},
	PermissionDenied {
		required_permission: String,
	},
	OrganizationAccessDenied {
		user_org: Uuid,
		resource_org: Uuid,
	},

	// -- CtxExtError
	#[from]
	CtxExt(middleware::mw_auth::CtxExtError),

	// -- Extractors
	ReqStampNotInReqExt,

	// -- Modules
	#[from]
	Model(model::Error),
	#[from]
	Pwd(pwd::Error),
	#[from]
	Token(token::Error),
	#[from]
	Rest(lib_rest_core::Error),

	// -- External Modules
	#[from]
	SerdeJson(#[serde_as(as = "DisplayFromStr")] serde_json::Error),
}

// region:    --- Axum IntoResponse
impl IntoResponse for Error {
	fn into_response(self) -> Response {
		debug!("{:<12} - model::Error {self:?}", "INTO_RES");

		// Create a placeholder Axum reponse.
		let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();

		// Insert the Error into the reponse.
		response.extensions_mut().insert(Arc::new(self));

		response
	}
}
// endregion: --- Axum IntoResponse

// region:    --- Error Boilerplate
impl core::fmt::Display for Error {
	fn fmt(
		&self,
		fmt: &mut core::fmt::Formatter,
	) -> core::result::Result<(), core::fmt::Error> {
		write!(fmt, "{self:?}")
	}
}

impl std::error::Error for Error {}
// endregion: --- Error Boilerplate

// region:    --- Client Error

/// From the root error to the http status code and ClientError
impl Error {
	pub fn client_status_and_error(&self) -> (StatusCode, ClientError) {
		use Error::*; // TODO: should change to `use web::Error as E`

		match self {
			// -- Login
			LoginFailUsernameNotFound
			| LoginFailEmailNotFound
			| LoginFailPwdNotMatching { .. } => (
				StatusCode::FORBIDDEN,
				ClientError::LOGIN_INVALID_CREDENTIALS,
			),
			LoginFailUserHasNoPwd { .. } => (
				StatusCode::FORBIDDEN,
				ClientError::LOGIN_PASSWORD_NOT_CONFIGURED,
			),
			LoginFailUserCtxCreate { .. } => (
				StatusCode::FORBIDDEN,
				ClientError::LOGIN_ACCOUNT_UNAVAILABLE,
			),

			// -- Auth
			CtxExt(middleware::mw_auth::CtxExtError::PasswordChangeRequired) => {
				(StatusCode::FORBIDDEN, ClientError::PASSWORD_CHANGE_REQUIRED)
			}
			CtxExt(_) => (StatusCode::FORBIDDEN, ClientError::NO_AUTH),

			// -- Authorization
			AccessDenied { required_role } => (
				StatusCode::FORBIDDEN,
				ClientError::ACCESS_DENIED {
					required_role: required_role.clone(),
				},
			),
			PermissionDenied {
				required_permission,
			} => (
				StatusCode::FORBIDDEN,
				ClientError::PERMISSION_DENIED {
					required_permission: required_permission.clone(),
				},
			),
			OrganizationAccessDenied { .. } => (
				StatusCode::FORBIDDEN,
				ClientError::ORGANIZATION_ACCESS_DENIED,
			),

			// -- Model
			Model(model::Error::EntityNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_NOT_FOUND { entity, id: *id },
			),
			Model(model::Error::EntityUuidNotFound { entity, id }) => (
				StatusCode::BAD_REQUEST,
				ClientError::ENTITY_UUID_NOT_FOUND {
					entity,
					id: id.to_string(),
				},
			),

			// -- Fallback.
			_ => (
				StatusCode::INTERNAL_SERVER_ERROR,
				ClientError::SERVICE_ERROR,
			),
		}
	}
}

#[derive(Debug, Serialize, strum_macros::AsRefStr)]
#[serde(tag = "message", content = "detail")]
#[allow(non_camel_case_types)]
pub enum ClientError {
	LOGIN_INVALID_CREDENTIALS,
	LOGIN_PASSWORD_NOT_CONFIGURED,
	LOGIN_ACCOUNT_UNAVAILABLE,
	NO_AUTH,
	PASSWORD_CHANGE_REQUIRED,
	ACCESS_DENIED { required_role: String },
	PERMISSION_DENIED { required_permission: String },
	ORGANIZATION_ACCESS_DENIED,
	ENTITY_NOT_FOUND { entity: &'static str, id: i64 },
	ENTITY_UUID_NOT_FOUND { entity: &'static str, id: String },
	CONSTRAINT_VIOLATION,
	INVALID_REQUEST,
	SERVICE_ERROR,
}
// endregion: --- Client Error

#[cfg(test)]
mod tests {
	use super::{Error, StatusCode};
	use serde_json::Value;
	use uuid::Uuid;

	fn assert_login_error_mapping(error: Error, expected_message: &str) {
		let (status, client_error) = error.client_status_and_error();
		let serialized = serde_json::to_value(client_error)
			.expect("client error should serialize");

		assert_eq!(status, StatusCode::FORBIDDEN);
		assert_eq!(
			serialized.get("message"),
			Some(&Value::String(expected_message.to_string()))
		);
		assert!(serialized.get("detail").is_none());
	}

	#[test]
	fn login_error_classifies_unknown_identity_as_invalid_credentials() {
		assert_login_error_mapping(
			Error::LoginFailEmailNotFound,
			"LOGIN_INVALID_CREDENTIALS",
		);
	}

	#[test]
	fn login_error_classifies_wrong_password_as_invalid_credentials() {
		assert_login_error_mapping(
			Error::LoginFailPwdNotMatching {
				user_id: Uuid::nil(),
			},
			"LOGIN_INVALID_CREDENTIALS",
		);
	}

	#[test]
	fn login_error_classifies_missing_password_as_not_configured() {
		assert_login_error_mapping(
			Error::LoginFailUserHasNoPwd {
				user_id: Uuid::nil(),
			},
			"LOGIN_PASSWORD_NOT_CONFIGURED",
		);
	}

	#[test]
	fn login_error_classifies_invalid_context_as_account_unavailable() {
		assert_login_error_mapping(
			Error::LoginFailUserCtxCreate {
				user_id: Uuid::nil(),
			},
			"LOGIN_ACCOUNT_UNAVAILABLE",
		);
	}
}
