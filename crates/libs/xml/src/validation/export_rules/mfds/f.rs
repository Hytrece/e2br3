use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	f_r_1(xpath, errors);
}

fn f_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	let code = "F.r.1";
	if super::super::matches(xpath, "//hl7:organizer[hl7:code[@code='3']]//hl7:effectiveTime[@nullFlavor and not(@nullFlavor='MSK')]") { errors.push(XmlValidationError { message: format!("[{code}] Only MSK is allowed."), code: Some(code.into()), section: Some("F".into()), field_path: None, blocking: Some(true), line: None, column: None }); }
}
