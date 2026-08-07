use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	g_k_2_1_kr_1a(xpath, errors);
	g_k_2_1_kr_1b(xpath, errors);
	g_k_2_3_r_1_kr_1a(xpath, errors);
	g_k_2_3_r_1_kr_1b(xpath, errors);
	g_k_9_i_2_r_2_kr_1(xpath, errors);
	g_k_9_i_2_r_3_kr_1(xpath, errors);
	g_k_9_i_2_r_3_kr_2(xpath, errors);
}

const PRODUCT: &str = "//hl7:organizer[hl7:code[@code='4']]//hl7:substanceAdministration/hl7:consumable/hl7:instanceOfKind/hl7:kindOfProduct/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.1' or @codeSystem='2.16.840.1.113883.6.294']";
const INGREDIENT: &str = "//hl7:organizer[hl7:code[@code='4']]//hl7:ingredientSubstance/hl7:code[@codeSystem='2.16.840.1.113883.3.989.5.1.10.2.2' or @codeSystem='2.16.840.1.113883.6.294']";

fn g_k_2_1_kr_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(
		xpath,
		errors,
		"G.k.2.1.KR.1a",
		&format!("{PRODUCT}/@codeSystemVersion"),
		20,
	);
}
fn g_k_2_1_kr_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(
		xpath,
		errors,
		"G.k.2.1.KR.1b",
		&format!("{PRODUCT}/@code"),
		10,
	);
}
fn g_k_2_3_r_1_kr_1a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(
		xpath,
		errors,
		"G.k.2.3.r.1.KR.1a",
		&format!("{INGREDIENT}/@codeSystemVersion"),
		20,
	);
}
fn g_k_2_3_r_1_kr_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	length(
		xpath,
		errors,
		"G.k.2.3.r.1.KR.1b",
		&format!("{INGREDIENT}/@code"),
		10,
	);
}
fn g_k_9_i_2_r_2_kr_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	allowed(xpath, errors, "G.k.9.i.2.r.2.KR.1", "//hl7:causalityAssessment/hl7:methodCode[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.4']", "@code='1' or @code='2'");
}
fn g_k_9_i_2_r_3_kr_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	let path="//hl7:causalityAssessment/hl7:value[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.5' or @nullFlavor]";
	allowed(
		xpath,
		errors,
		"G.k.9.i.2.r.3.KR.1",
		path,
		"@code='1' or @code='2' or @code='3' or @code='4' or @code='5' or @code='6'",
	);
	reject(
		xpath,
		errors,
		"G.k.9.i.2.r.3.KR.1",
		&format!("{path}[@nullFlavor and not(@nullFlavor='NA')]"),
	);
}
fn g_k_9_i_2_r_3_kr_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	allowed(xpath, errors, "G.k.9.i.2.r.3.KR.2", "//hl7:causalityAssessment/hl7:value[@codeSystem='2.16.840.1.113883.3.989.5.1.10.1.6']", "@code='1' or @code='2'");
}

fn allowed(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	element: &'static str,
	path: &str,
	values: &str,
) {
	length(xpath, errors, element, &format!("{path}/@code"), 1);
	reject(
		xpath,
		errors,
		element,
		&format!("{path}[@code and not({values})]"),
	);
}
fn length(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	path: &str,
	max: usize,
) {
	reject(
		xpath,
		errors,
		code,
		&format!("{path}[string-length(.) > {max}]"),
	);
}
fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	path: &str,
) {
	if super::super::matches(xpath, path) {
		errors.push(XmlValidationError {
			message: format!("[{code}] Invalid MFDS value."),
			code: Some(code.into()),
			section: Some("G".into()),
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
	fn g_k_9_i_2_r_3_kr_1_emits_exact_codes() {
		let doc=Parser::default().parse_string(r#"<causalityAssessment xmlns="urn:hl7-org:v3"><value codeSystem="2.16.840.1.113883.3.989.5.1.10.1.5" code="7"/></causalityAssessment>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		g_k_9_i_2_r_3_kr_1(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("G.k.9.i.2.r.3.KR.1"));
	}
}
