use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	expression: &str,
	message: &'static str,
) {
	if super::super::matches(xpath, expression) {
		errors.push(XmlValidationError {
			message: format!("[{code}] {message}"),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}

pub(super) fn reject_max_len(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	expression: &str,
	max: usize,
) {
	reject(
		xpath,
		errors,
		code,
		&format!("{expression}[string-length(.) > {max}]"),
		"Value exceeds the ICH maximum length.",
	);
}

pub(super) fn reject_code(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	expression: &str,
	allowed: &str,
) {
	reject(
		xpath,
		errors,
		code,
		&format!("{expression}[not({allowed})]"),
		"Value is not allowed by ICH.",
	);
}
