use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	f_r_2_2a(xpath, errors);
	f_r_2_2b(xpath, errors);
	f_r_3_1(xpath, errors);
	f_r_7(xpath, errors);
}

fn f_r_2_2a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "F.r.2.2a", "//hl7:observation[hl7:code[@codeSystem='2.16.840.1.113883.6.163']]/hl7:code[@codeSystemVersion and (not(contains(@codeSystemVersion, '.')) or translate(@codeSystemVersion, '0123456789.', '') != '')]");
}
fn f_r_2_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "F.r.2.2b", "//hl7:observation/hl7:code[@codeSystem='2.16.840.1.113883.6.163' and @code and translate(@code, '0123456789', '') != '']");
}
fn f_r_3_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "F.r.3.1", "//hl7:organizer[hl7:code[@code='3']]//hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.12' and @code and not(@code='1' or @code='2' or @code='3' or @code='4')]");
}
fn f_r_7(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "F.r.7", "//hl7:organizer[hl7:code[@code='3']]//hl7:value[@xsi:type='BL' and @value and not(@value='true' or @value='false')]");
}

fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	path: &str,
) {
	if super::super::matches(xpath, path) {
		errors.push(XmlValidationError {
			message: format!("[{code}] Invalid value."),
			code: Some(code.into()),
			section: Some("F".into()),
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
	fn f_r_7_reports_exact_code() {
		let doc=Parser::default().parse_string(r#"<organizer xmlns="urn:hl7-org:v3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><code code="3"/><value xsi:type="BL" value="0"/></organizer>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		xpath
			.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance")
			.unwrap();
		let mut errors = vec![];
		f_r_7(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("F.r.7"));
	}
}
