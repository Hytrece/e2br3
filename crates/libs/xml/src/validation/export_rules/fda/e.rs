use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	r0058(xpath, errors);
	r0059(xpath, errors);
	r0060(xpath, errors);
	e_i_2_1a(xpath, errors);
	e_i_2_1b(xpath, errors);
	e_i_3_2(xpath, errors);
	e_i_7(xpath, errors);
}

fn reaction() -> &'static str {
	"//hl7:observation[hl7:code[@code='29' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]"
}
fn r0058(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0058", &format!("{}[hl7:value/hl7:originalText[normalize-space()] and not(hl7:value/hl7:originalText/@language)]", reaction()));
}
fn r0059(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0059",
		&format!(
			"{}/hl7:effectiveTime/hl7:width[@unit and not(@value)]",
			reaction()
		),
	);
}
fn r0060(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0060",
		&format!(
			"{}/hl7:effectiveTime/hl7:width[@value and not(@unit)]",
			reaction()
		),
	);
}
fn e_i_2_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.2.1a", &format!("{}[not(hl7:value[@codeSystem='2.16.840.1.113883.6.163']/@codeSystemVersion)]", reaction()));
}
fn e_i_2_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"E.i.2.1b",
		&format!(
			"{}[not(hl7:value[@codeSystem='2.16.840.1.113883.6.163']/@code)]",
			reaction()
		),
	);
}
fn e_i_3_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.3.2", &format!("{}[count(.//hl7:observation[hl7:code[@code='34' or @code='21' or @code='33' or @code='35' or @code='12' or @code='26']]/hl7:value[@value='true' or @nullFlavor='NI']) != 6]", reaction()));
}
fn e_i_7(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "E.i.7", &format!("{}[not(.//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='0' or .//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='1' or .//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='2' or .//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='3' or .//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='4' or .//hl7:observation[hl7:code[@code='27']]/hl7:value/@code='5')]", reaction()));
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
