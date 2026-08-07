use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	d_8_r_1_kr_1a(xpath, errors);
	d_8_r_1_kr_1b(xpath, errors);
	d_10_8_r_1_kr_1a(xpath, errors);
	d_10_8_r_1_kr_1b(xpath, errors);
}

const PRODUCT: &str = "hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.1' or @codeSystem='2.16.840.1.113883.6.294']";
fn d_8_r_1_kr_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(xpath, errors, "D.8.r.1.KR.1a", &format!("//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//{PRODUCT}/@codeSystemVersion"), 20);
}
fn d_8_r_1_kr_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(xpath, errors, "D.8.r.1.KR.1b", &format!("//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//{PRODUCT}/@code"), 10);
}
fn d_10_8_r_1_kr_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(xpath, errors, "D.10.8.r.1.KR.1a", &format!("//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//{PRODUCT}/@codeSystemVersion"), 20);
}
fn d_10_8_r_1_kr_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(xpath, errors, "D.10.8.r.1.KR.1b", &format!("//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//{PRODUCT}/@code"), 10);
}
fn length(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	path: &str,
	max: usize,
) {
	let expression = format!("{path}[string-length(.) > {max}]");
	if super::super::matches(xpath, &expression) {
		errors.push(XmlValidationError {
			message: format!("[{code}] Value is too long."),
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
	fn d_8_r_1_kr_1b_emits_exact_code() {
		let doc=Parser::default().parse_string(r#"<primaryRole xmlns="urn:hl7-org:v3"><subjectOf2><organizer><code code="2"/><consumable><instanceOfKind><kindOfProduct><code codeSystem="2.16.840.1.113883.3.989.5.1.10.2.1" code="12345678901"/></kindOfProduct></instanceOfKind></consumable></organizer></subjectOf2></primaryRole>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		d_8_r_1_kr_1b(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("D.8.r.1.KR.1b"));
	}
}
