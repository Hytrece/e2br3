use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	e_i_1_1b(xpath, errors);
	e_i_2_1a(xpath, errors);
	e_i_2_1b(xpath, errors);
	e_i_3_2a(xpath, errors);
	e_i_3_2b(xpath, errors);
	e_i_3_2c(xpath, errors);
	e_i_3_2d(xpath, errors);
	e_i_3_2e(xpath, errors);
	e_i_3_2f(xpath, errors);
	e_i_6b(xpath, errors);
	e_i_7(xpath, errors);
	e_i_8(xpath, errors);
	e_i_9(xpath, errors);
}

fn e_i_1_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.1.1b", "//hl7:observationEvent/hl7:value/hl7:originalText[normalize-space() and not(@language)] | //hl7:observationEvent/hl7:value/hl7:originalText[@language and (string-length(@language) != 3 or translate(@language, 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '')]");
}
fn e_i_2_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.2.1a", "//hl7:observationEvent/hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @codeSystemVersion and (not(contains(@codeSystemVersion, '.')) or translate(@codeSystemVersion, '0123456789.', '') != '')]");
}
fn e_i_2_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.2.1b", "//hl7:observationEvent/hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @code and translate(@code, '0123456789', '') != '']");
}
fn e_i_3_2a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2a", "34");
}
fn e_i_3_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2b", "21");
}
fn e_i_3_2c(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2c", "33");
}
fn e_i_3_2d(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2d", "35");
}
fn e_i_3_2e(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2e", "12");
}
fn e_i_3_2f(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "E.i.3.2f", "26");
}
fn e_i_6b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.6b", "//hl7:observationEvent//hl7:width[@unit and not(@unit='10.a' or @unit='a' or @unit='mo' or @unit='wk' or @unit='d' or @unit='h' or @unit='min' or @unit='s')]");
}
fn e_i_7(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.7", "//hl7:observationEvent//hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.11' and @code and not(@code='0' or @code='1' or @code='2' or @code='3' or @code='4' or @code='5')]");
}
fn e_i_8(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.8", "//hl7:observationEvent//hl7:value[@xsi:type='BL' and @value and not(@value='true' or @value='false')]");
}
fn e_i_9(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.9", "//hl7:observationEvent//hl7:location//hl7:code[@code and (string-length(@code) != 2 or translate(@code, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '')]");
}

fn true_marker(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	observation_code: &str,
) {
	reject(xpath, errors, code, &format!("//hl7:observationEvent//hl7:observation[hl7:code[@code='{observation_code}']]/hl7:value[@value and not(@value='true')]"));
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
			section: Some("E".into()),
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
	fn e_i_8_reports_exact_code() {
		let doc = Parser::default().parse_string(r#"<observationEvent xmlns="urn:hl7-org:v3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><value xsi:type="BL" value="1"/></observationEvent>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		xpath
			.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance")
			.unwrap();
		let mut errors = vec![];
		e_i_8(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("E.i.8"));
	}
}
