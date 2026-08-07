use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	d_2_2b(xpath, errors);
	d_2_2_1b(xpath, errors);
	d_2_3(xpath, errors);
	d_5(xpath, errors);
	d_7_1_r_3(xpath, errors);
	d_7_1_r_6(xpath, errors);
	d_7_3(xpath, errors);
	d_9_3(xpath, errors);
	d_10_2_2b(xpath, errors);
	d_10_6(xpath, errors);
}

fn d_2_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.2.2b", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@unit and not(@unit='10.a' or @unit='a' or @unit='mo' or @unit='wk' or @unit='d' or @unit='h')]");
}

fn d_2_2_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.2.2.1b", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='16']]/hl7:value[@unit and not(@unit='mo' or @unit='wk' or @unit='d' or @unit='{Trimester}')]");
}

fn d_2_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.2.3", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='4']]/hl7:value[@code and not(@code='0' or @code='1' or @code='2' or @code='3' or @code='4' or @code='5' or @code='6')]");
}

fn d_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.5", "//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode[@code and not(@code='1' or @code='2')]");
}

fn d_7_1_r_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.7.1.r.3", "//hl7:organizer[hl7:code[@code='1']]//hl7:observation[hl7:code[@code='13']]/hl7:value[@value and not(@value='true' or @value='false')]");
}

fn d_7_1_r_6(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "D.7.1.r.6", "//hl7:organizer[hl7:code[@code='1']]//hl7:observation[hl7:code[@code='38']]/hl7:value");
}

fn d_7_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	true_marker(xpath, errors, "D.7.3", "//hl7:organizer[hl7:code[@code='1']]//hl7:observation[hl7:code[@code='11']]/hl7:value");
}

fn d_9_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.9.3", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]//hl7:value[@xsi:type='BL' and @value and not(@value='true' or @value='false')]");
}

fn d_10_2_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.10.2.2b", "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@unit and not(@unit='a' or @unit='10.a')]");
}

fn d_10_6(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "D.10.6", "//hl7:role[@classCode='PRS']/hl7:associatedPerson/hl7:administrativeGenderCode[@code and not(@code='1' or @code='2')]");
}

fn true_marker(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	path: &str,
) {
	reject(
		xpath,
		errors,
		code,
		&format!("{path}[@value and not(@value='true')]"),
	);
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
			section: Some("D".into()),
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
	fn d_10_2_2b_reports_exact_code() {
		let doc = Parser::default().parse_string(r#"<r xmlns="urn:hl7-org:v3"><role classCode="PRS"><subjectOf2><observation><code code="3"/><value unit="mo"/></observation></subjectOf2></role></r>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		d_10_2_2b(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("D.10.2.2b"));
	}
}
