use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	g_k_1(xpath, errors);
	g_k_2_4(xpath, errors);
	g_k_2_5(xpath, errors);
	g_k_3_2(xpath, errors);
	g_k_4_r_3(xpath, errors);
	g_k_4_r_6b(xpath, errors);
	g_k_6b(xpath, errors);
	g_k_7_r_2a(xpath, errors);
	g_k_7_r_2b(xpath, errors);
	g_k_8(xpath, errors);
	g_k_9_i_3_1b(xpath, errors);
	g_k_9_i_3_2b(xpath, errors);
	g_k_9_i_4(xpath, errors);
	g_k_10_r(xpath, errors);
}

const DRUG: &str =
	"//hl7:organizer[hl7:code[@code='4']]//hl7:substanceAdministration";
const TIME_UNITS: &str = "@unit='10.a' or @unit='a' or @unit='mo' or @unit='wk' or @unit='d' or @unit='h' or @unit='min' or @unit='s'";

fn g_k_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.1", &format!("{DRUG}/hl7:code[@code and not(@code='1' or @code='2' or @code='3' or @code='4')]"));
}
fn g_k_2_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	country(
		xpath,
		errors,
		"G.k.2.4",
		&format!("{DRUG}//hl7:territory/hl7:code"),
	);
}
fn g_k_2_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.2.5", &format!("{DRUG}//hl7:observation[hl7:code[@code='6']]/hl7:value[@value and not(@value='true')]"));
}
fn g_k_3_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	country(
		xpath,
		errors,
		"G.k.3.2",
		&format!("{DRUG}//hl7:territorialAuthority/hl7:territory/hl7:code"),
	);
}
fn g_k_4_r_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.4.r.3", &format!("{DRUG}//hl7:period[@unit and not({TIME_UNITS} or @unit='{{cyclical}}' or @unit='{{asnecessary}}' or @unit='{{total}}')]"));
}
fn g_k_4_r_6b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"G.k.4.r.6b",
		&format!("{DRUG}//hl7:width[@unit and not({TIME_UNITS})]"),
	);
}
fn g_k_6b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.6b", &format!("{DRUG}//hl7:observation[hl7:code[@code='16']]/hl7:value[@unit and not(@unit='mo' or @unit='wk' or @unit='d' or @unit='{{Trimester}}')]"));
}
fn g_k_7_r_2a(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.7.r.2a", &format!("{DRUG}//hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @codeSystemVersion and (not(contains(@codeSystemVersion, '.')) or translate(@codeSystemVersion, '0123456789.', '') != '')]"));
}
fn g_k_7_r_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.7.r.2b", &format!("{DRUG}//hl7:value[@codeSystem='2.16.840.1.113883.6.163' and @code and translate(@code, '0123456789', '') != '']"));
}
fn g_k_8(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.8", &format!("{DRUG}//hl7:act/hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.15' and @code and not(@code='0' or @code='1' or @code='2' or @code='3' or @code='4' or @code='9')]"));
}
fn g_k_9_i_3_1b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	time(xpath, errors, "G.k.9.i.3.1b", "SAS");
}
fn g_k_9_i_3_2b(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	time(xpath, errors, "G.k.9.i.3.2b", "SAE");
}
fn g_k_9_i_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.9.i.4", &format!("{DRUG}//hl7:observation[hl7:code[@code='31']]/hl7:value[@code and not(@code='1' or @code='2' or @code='3' or @code='4')]"));
}
fn g_k_10_r(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "G.k.10.r", &format!("{DRUG}//hl7:observation[hl7:code[@code='9']]/hl7:value[@code and not(@code='1' or @code='2' or @code='3' or @code='4' or @code='5' or @code='6' or @code='7' or @code='8' or @code='9' or @code='10' or @code='11')]"));
}

fn country(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	path: &str,
) {
	reject(xpath, errors, code, &format!("{path}[@code and (string-length(@code) != 2 or translate(@code, 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '')]"));
}
fn time(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	relation: &str,
) {
	reject(xpath, errors, code, &format!("{DRUG}/hl7:outboundRelationship1[@typeCode='{relation}']/hl7:pauseQuantity[@unit and not({TIME_UNITS})]"));
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
	fn g_k_6b_preserves_official_case_and_code() {
		let doc=Parser::default().parse_string(r#"<organizer xmlns="urn:hl7-org:v3"><code code="4"/><substanceAdministration><observation><code code="16"/><value unit="{trimester}"/></observation></substanceAdministration></organizer>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		g_k_6b(&mut xpath, &mut errors);
		assert_eq!(errors[0].code.as_deref(), Some("G.k.6b"));
	}
}
