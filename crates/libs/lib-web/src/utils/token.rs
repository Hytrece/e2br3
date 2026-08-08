pub use crate::error::ClientError;
pub use crate::error::{Error, Result};
use lib_auth::token::generate_web_token;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

// endregion: --- Modules

pub(crate) const AUTH_TOKEN: &str = "auth-token";

pub fn set_token_cookie(
	cookies: &Cookies,
	email: &str,
	organization_id: Uuid,
	salt: Uuid,
) -> Result<()> {
	// Email is not globally unique. Bind the selected tenant into the signed
	// identity so a token can never resolve to another same-email account.
	let identity = format!("{email}|{organization_id}");
	let token = generate_web_token(&identity, salt)?;

	let mut cookie = Cookie::new(AUTH_TOKEN, token.to_string());
	cookie.set_http_only(true);
	cookie.set_path("/");

	cookies.add(cookie);

	Ok(())
}

pub(crate) fn remove_token_cookie(cookies: &Cookies) -> Result<()> {
	let mut cookie = Cookie::from(AUTH_TOKEN);
	cookie.set_path("/");

	cookies.remove(cookie);

	Ok(())
}
