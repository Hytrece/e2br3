use crate::error::Result;
use crate::middleware::mw_auth::{CtxExtError, CtxW};
use axum::body::Body;
use axum::http::{Method, Request};
use axum::middleware::Next;
use axum::response::Response;

pub async fn mw_ctx_require_and_set_dbx(
	ctx: Result<CtxW>,
	req: Request<Body>,
	next: Next,
) -> Result<Response> {
	let ctx = ctx?;
	if ctx.1
		&& !(req.method() == Method::POST
			&& req.uri().path() == "/users/me/password")
	{
		return Err(CtxExtError::PasswordChangeRequired.into());
	}
	Ok(next.run(req).await)
}
