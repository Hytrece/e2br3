use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	h_3_r_1a(xpath, errors);
	h_3_r_1b(xpath, errors);
	h_5_r_1b(xpath, errors);
}

fn h_3_r_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "H.3.r.1a", "//hl7:observationEvent[hl7:code[@code='15']]/hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @codeSystemVersion and (not(contains(@codeSystemVersion, '.')) or translate(@codeSystemVersion, '0123456789.', '') != '')]");
}
fn h_3_r_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "H.3.r.1b", "//hl7:observationEvent[hl7:code[@code='15']]/hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @code and translate(@code, '0123456789', '') != '']");
}
fn h_5_r_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "H.5.r.1b", "//hl7:observationEvent[hl7:code[@code='36']]/hl7:value[@language and (string-length(@language) != 3 or translate(@language, 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '')]");
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
			section: Some("H".into()),
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
	fn h_3_r_1b_reports_exact_code() {
		let doc=Parser::default().parse_string(r#"<observationEvent xmlns="urn:hl7-org:v3"><code code="15"/><value codeSystem="2.16.840.1.113883.6.163" code="x"/></observationEvent>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		h_3_r_1b(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("H.3.r.1b"));
	}
}
