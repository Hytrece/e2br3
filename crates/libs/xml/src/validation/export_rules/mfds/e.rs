use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	e_i_9(xpath, errors);
}

fn e_i_9(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	let code = "E.i.9";
	if super::super::matches(
		xpath,
		"//hl7:observationEvent/hl7:location//hl7:locatedPlace/hl7:code[@code='EU']",
	) {
		errors.push(XmlValidationError {
			message: format!("[{code}] EU is not allowed by MFDS."),
			code: Some(code.into()),
			section: Some("E".into()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}
