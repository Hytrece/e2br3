use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	h_1(xpath, errors);
}

fn h_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	super::super::reject(
		xpath,
		errors,
		"H.1",
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:controlActProcess/hl7:subject/hl7:investigationEvent[not(hl7:text[normalize-space(.) != ''])]",
		"Each ICSR must include a case narrative (H.1).",
	);
}
