use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	r0097(xpath, errors);
	r0098(xpath, errors);
	r0099(xpath, errors);
}
fn r0097(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0097","//hl7:observationEvent[hl7:code[@code='15']]/hl7:value[@code and not(@codeSystemVersion)]");
}
fn r0098(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0098","//hl7:observationEvent[hl7:code[@code='15']]/hl7:value[@codeSystemVersion and not(@code)]");
}
fn r0099(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0099","//hl7:observationEvent[hl7:code[@code='36']]/hl7:value[normalize-space() and not(@language)]");
}
fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	expression: &str,
) {
	if super::super::matches(xpath, expression) {
		errors.push(XmlValidationError {
			message: format!("[{code}] FDA business rule failed."),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}
