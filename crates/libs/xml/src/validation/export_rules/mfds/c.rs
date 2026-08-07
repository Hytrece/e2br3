use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	c_2_r_4_kr_1(xpath, errors);
	c_3_1_kr_1(xpath, errors);
	c_5_4_kr_1(xpath, errors);
}

fn c_2_r_4_kr_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	allowed(
		xpath,
		errors,
		"C.2.r.4.KR.1",
		"//*[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.1']",
		"@code='1' or @code='2'",
	);
}
fn c_3_1_kr_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	allowed(
		xpath,
		errors,
		"C.3.1.KR.1",
		"(//*[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.2'] | //hl7:observation[hl7:code[@code='C.3.1.KR.1']]/hl7:value)",
		"@code='1' or @code='2' or @code='3' or @code='4'",
	);
}
fn c_5_4_kr_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	allowed(
		xpath,
		errors,
		"C.5.4.KR.1",
		"(//*[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.3'] | //hl7:observation[hl7:code[@code='C.5.4.KR.1']]/hl7:value)",
		"@code='1' or @code='2' or @code='3' or @code='4'",
	);
}

fn allowed(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	element: &'static str,
	path: &str,
	values: &str,
) {
	reject(
		xpath,
		errors,
		element,
		&format!("{path}[string-length(@code) > 1]"),
	);
	reject(
		xpath,
		errors,
		element,
		&format!("{path}[@code and not({values})]"),
	);
}
fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	path: &str,
) {
	if super::super::matches(xpath, path) {
		errors.push(XmlValidationError {
			message: format!("[{code}] Invalid MFDS value."),
			code: Some(code.into()),
			section: Some("C".into()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use libxml::parser::Parser;
	#[test]
	fn c_2_r_4_kr_1_emits_exact_code() {
		let doc = Parser::default()
			.parse_string(
				r#"<code codeSystem="2.16.840.1.113883.3.989.5.1.10.1.1" code="3"/>"#,
			)
			.unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		let mut errors = vec![];
		c_2_r_4_kr_1(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("C.2.r.4.KR.1"));
	}
}
