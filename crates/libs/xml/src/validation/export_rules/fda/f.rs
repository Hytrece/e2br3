use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	r0061(xpath, errors);
	r0062(xpath, errors);
	r0063(xpath, errors);
	r0064(xpath, errors);
	r0065(xpath, errors);
	r0067(xpath, errors);
}

fn test_observation() -> &'static str {
	"//hl7:organizer[hl7:code[@code='3']]/hl7:component/hl7:observation"
}
fn r0061(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0061",&format!("{}[(hl7:code/@code or hl7:code/hl7:originalText[normalize-space()]) and not(hl7:effectiveTime/@value or hl7:effectiveTime/@nullFlavor)]",test_observation()));
}
fn r0062(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0062",&format!("{}[hl7:effectiveTime and not(hl7:code/@code) and not(hl7:code/hl7:originalText[normalize-space()])]",test_observation()));
}
fn r0063(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0063",
		&format!(
			"{}/hl7:code[@code and not(@codeSystemVersion)]",
			test_observation()
		),
	);
}
fn r0064(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0064",&format!("{}[hl7:effectiveTime and not(hl7:code/@code) and not(hl7:code/hl7:originalText[normalize-space()])]",test_observation()));
}
fn no_result() -> String {
	format!("{}[(hl7:code/@code or hl7:code/hl7:originalText[normalize-space()]) and not(hl7:value or hl7:interpretationCode/@code)]",test_observation())
}
fn r0065(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0065", &no_result());
}
fn r0067(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0067",
		&format!("{0}/hl7:value[@xsi:type='PQ' and @value and not(@unit)] | {0}/hl7:value[@xsi:type='IVL_PQ']/*[self::hl7:low or self::hl7:high][@value and not(@unit)]", test_observation()),
	);
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
