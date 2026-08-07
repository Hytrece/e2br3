use crate::XmlValidationError;
use libxml::xpath::Context;

use super::common;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	n_1_1(xpath, errors);
	n_1_2(xpath, errors);
	n_1_3(xpath, errors);
	n_1_4(xpath, errors);
	n_2_r_1(xpath, errors);
	n_2_r_2(xpath, errors);
	n_2_r_3(xpath, errors);
	n_2_r_4(xpath, errors);
}

fn n_1_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(xpath, errors, "N.1.1", "/hl7:MCCI_IN200100UV01/hl7:name[@codeSystem='2.16.840.1.113883.3.989.2.1.1.1']/@code", ".='1'");
}

fn n_1_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "N.1.2", "/hl7:MCCI_IN200100UV01/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.22']/@extension", 100);
}

fn n_1_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "N.1.3", "/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.13']/@extension", 60);
}

fn n_1_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "N.1.4", "/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.14']/@extension", 60);
}

fn n_2_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(
		xpath,
		errors,
		"N.2.r.1",
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV[hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension != hl7:controlActProcess/hl7:subject/hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension] | /hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension[string-length(.) > 100]",
		"Message identifier must equal C.1.1 and not exceed 100 characters.",
	);
}

fn n_2_r_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "N.2.r.2", "/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:sender/hl7:device/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.11']/@extension", 60);
}

fn n_2_r_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "N.2.r.3", "/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.12']/@extension", 60);
}

fn n_2_r_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "N.2.r.4", "/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV[hl7:creationTime/@value != hl7:controlActProcess/hl7:effectiveTime/@value]", "Message creation date must equal C.1.2.");
}

#[cfg(test)]
mod tests {
	use super::*;
	use libxml::parser::Parser;

	#[test]
	fn n_rules_emit_official_codes() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><name codeSystem="2.16.840.1.113883.3.989.2.1.1.1" code="2"/><PORR_IN049016UV><creationTime value="a"/><controlActProcess><effectiveTime value="b"/></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = Vec::new();
		run(&mut xpath, &mut errors);
		let codes = errors
			.into_iter()
			.filter_map(|error| error.code)
			.collect::<Vec<_>>();
		assert!(codes.contains(&"N.1.1".to_string()));
		assert!(codes.contains(&"N.2.r.4".to_string()));
	}
}
