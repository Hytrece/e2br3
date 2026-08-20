use crate::error::Result;
use crate::error::{ClientError, Error};
use crate::middleware::mw_req_stamp::ReqStamp;
use axum::http::{Method, StatusCode, Uri};
use lib_core::ctx::Ctx;
use lib_utils::time::{format_time, now_utc};
use time::Duration;
use tracing::{info, warn};

pub async fn log_request(
	http_method: Method,
	uri: Uri,
	req_stamp: ReqStamp,
	ctx: Option<Ctx>,
	status: StatusCode,
	web_error: Option<&Error>,
	client_error: Option<ClientError>,
) -> Result<()> {
	let error_type = web_error.map(|se| se.as_ref().to_string());
	let ReqStamp { uuid, time_in } = req_stamp;
	let now = now_utc();
	let duration: Duration = now - time_in;
	let duration_ms = (duration.as_seconds_f64() * 1_000_000.).floor() / 1_000.;
	let user_id = ctx.map(|ctx| ctx.user_id().to_string());
	let client_error_type = client_error.map(|error| error.as_ref().to_string());
	let http_path = sanitized_http_path(&uri);
	if status.is_client_error() || status.is_server_error() {
		warn!(
			event = "http_request",
			request_id = %uuid,
			timestamp = %format_time(now),
			time_in = %format_time(time_in),
			duration_ms,
			http_method = %http_method,
			http_path,
			status = status.as_u16(),
			user_id = user_id.as_deref().unwrap_or(""),
			client_error_type = client_error_type.as_deref().unwrap_or(""),
			error_type = error_type.as_deref().unwrap_or(""),
			"request completed with error"
		);
	} else {
		info!(
			event = "http_request",
			request_id = %uuid,
			timestamp = %format_time(now),
			time_in = %format_time(time_in),
			duration_ms,
			http_method = %http_method,
			http_path,
			status = status.as_u16(),
			user_id = user_id.as_deref().unwrap_or(""),
			"request completed"
		);
	}

	Ok(())
}

fn sanitized_http_path(uri: &Uri) -> &str {
	uri.path()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn request_log_path_excludes_query_data() {
		let uri: Uri = "/api/cases?patient=secret".parse().unwrap();
		assert_eq!(sanitized_http_path(&uri), "/api/cases");
	}
}
