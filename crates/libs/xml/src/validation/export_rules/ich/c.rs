use crate::XmlValidationError;
use libxml::xpath::Context;

use super::common;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	c_1_1(xpath, errors);
	c_1_2(xpath, errors);
	c_1_3(xpath, errors);
	c_1_6_1(xpath, errors);
	c_1_6_1_r_1(xpath, errors);
	c_1_7(xpath, errors);
	c_1_8_1(xpath, errors);
	c_1_8_2(xpath, errors);
	c_1_9_1(xpath, errors);
	c_1_9_1_r_1(xpath, errors);
	c_1_9_1_r_2(xpath, errors);
	c_1_11_1(xpath, errors);
	c_1_11_2(xpath, errors);
	c_2_r_3(xpath, errors);
	c_2_r_4(xpath, errors);
	c_2_r_5(xpath, errors);
	c_3_1(xpath, errors);
	c_3_4_5(xpath, errors);
	c_4_r_1(xpath, errors);
	c_5_1_r_1(xpath, errors);
	c_5_1_r_2(xpath, errors);
	c_5_2(xpath, errors);
	c_5_3(xpath, errors);
	c_5_4(xpath, errors);
}

fn c_1_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.1", "//hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension[string-length(.) > 100 or string-length(.) < 4 or substring(., 3, 1) != '-' or translate(substring(., 1, 2), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '']", "Identifier must be at most 100 characters and start with an uppercase two-letter country code and hyphen.");
}

fn c_1_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.2", "//hl7:PORR_IN049016UV/hl7:controlActProcess/hl7:effectiveTime/@value[(string-length(.) != 14 and string-length(.) != 19) or translate(substring(., 1, 14), '0123456789', '') != '' or (string-length(.) = 19 and (not(substring(., 15, 1)='+' or substring(., 15, 1)='-') or translate(substring(., 16, 4), '0123456789', '') != ''))]", "Date of creation must use CCYYMMDDhhmmss with an optional numeric UTC offset.");
}

fn c_1_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(
		xpath,
		errors,
		"C.1.3",
		"//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code",
		".='1' or .='2' or .='3' or .='4'",
	);
}

fn c_1_6_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(xpath, errors, "C.1.6.1", "//hl7:observationEvent[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@value", ".='false' or .='true'");
}

fn c_1_6_1_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.6.1.r.1", "//hl7:investigationEvent[hl7:component/hl7:observationEvent[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@value='true' and not(hl7:reference/hl7:document/hl7:title[normalize-space()])] | //hl7:reference/hl7:document/hl7:title/text()[string-length(.) > 2000]", "Available documents require a title of at most 2000 characters.");
}

fn c_1_7(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(xpath, errors, "C.1.7", "//hl7:observationEvent[hl7:code[@code='23' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value/@value", ".='false' or .='true'");
}

fn c_1_8_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.8.1", "//hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']/@extension[string-length(.) > 100 or string-length(.) < 4 or substring(., 3, 1) != '-' or translate(substring(., 1, 2), 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '']", "Worldwide identifier must be at most 100 characters and start with an uppercase two-letter country code and hyphen.");
}

fn c_1_8_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(
		xpath,
		errors,
		"C.1.8.2",
		"//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.3']/@code",
		".='1' or .='2'",
	);
}

fn c_1_9_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(xpath, errors, "C.1.9.1", "//hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@value", ".='true'");
}

fn c_1_9_1_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.9.1.r.1", "//hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@value='true' and not(hl7:subjectOf1/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']/@assigningAuthorityName)] | //hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']/@assigningAuthorityName[string-length(.) > 100]", "Previous identifiers require a source of at most 100 characters.");
}

fn c_1_9_1_r_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.9.1.r.2", "//hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value/@value='true' and not(hl7:subjectOf1/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']/@extension)] | //hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']/@extension[string-length(.) > 100]", "Previous identifiers require a case identifier of at most 100 characters.");
}

fn c_1_11_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(
		xpath,
		errors,
		"C.1.11.1",
		"//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.5']/@code",
		".='1' or .='2'",
	);
}

fn c_1_11_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.1.11.2", "//hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.5']/@code and not(hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='4']]/hl7:value/hl7:originalText[normalize-space()])] | //hl7:investigationCharacteristic[hl7:code[@code='4']]/hl7:value/hl7:originalText/text()[string-length(.) > 2000]", "Nullification or amendment requires a reason of at most 2000 characters.");
}

fn c_2_r_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.2.r.3", "//hl7:relatedInvestigation[hl7:code[@code='2']]//*[@codeSystem='1.0.3166.1.2.2']/@code[string-length(.) != 2 or translate(., 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '']", "Reporter country must be a two-letter uppercase code.");
}

fn c_2_r_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(xpath, errors, "C.2.r.4", "//hl7:relatedInvestigation[hl7:code[@code='2']]//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.6']/@code", ".='1' or .='2' or .='3' or .='4' or .='5'");
}

fn c_2_r_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.2.r.5", "//hl7:PORR_IN049016UV[count(.//hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:priorityNumber[@value='1']) != 1] | //hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:priorityNumber[not(@value='1')]", "Each ICSR must flag exactly one primary source with value 1.");
}

fn c_3_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_code(
		xpath,
		errors,
		"C.3.1",
		"//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7']/@code",
		".='1' or .='2' or .='3' or .='4' or .='5' or .='6' or .='7'",
	);
}

fn c_3_4_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.3.4.5", "//hl7:subjectOf1[hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7']]//*[@codeSystem='1.0.3166.1.2.2']/@code[string-length(.) != 2 or translate(., 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '']", "Sender country must be a two-letter uppercase code.");
}

fn c_4_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "C.4.r.1", "//hl7:document[hl7:code[@code='2']]/hl7:bibliographicDesignationText/text()", 500);
}

fn c_5_1_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "C.5.1.r.1", "//hl7:studyRegistration/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.6']/@extension", 50);
}

fn c_5_1_r_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.5.1.r.2", "//hl7:studyRegistration//*[@codeSystem='1.0.3166.1.2.2']/@code[string-length(.) != 2 or translate(., 'ABCDEFGHIJKLMNOPQRSTUVWXYZ', '') != '']", "Study registration country must be a two-letter uppercase code.");
}

fn c_5_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(
		xpath,
		errors,
		"C.5.2",
		"//hl7:researchStudy/hl7:title/text()",
		2000,
	);
}

fn c_5_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject_max_len(xpath, errors, "C.5.3", "//hl7:researchStudy/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.5']/@extension", 50);
}

fn c_5_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	common::reject(xpath, errors, "C.5.4", "//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code[not(.='1' or .='2' or .='3')] | //hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2' and @code='2'] and not(.//hl7:researchStudy/hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code)]", "Study reports require an allowed study type.");
}

#[cfg(test)]
mod tests {
	use super::*;
	use libxml::parser::Parser;

	fn c_2_r_5_errors(xml: &[u8]) -> Vec<XmlValidationError> {
		let doc = Parser::default().parse_string(xml).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = Vec::new();
		c_2_r_5(&mut xpath, &mut errors);
		errors
	}

	#[test]
	fn c_2_r_5_accepts_priority_on_source_relationship() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><priorityNumber value="1"/><relatedInvestigation><code code="2"/></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		assert!(c_2_r_5_errors(xml).is_empty());
	}

	#[test]
	fn c_2_r_5_rejects_missing_and_duplicate_primary_markers() {
		let missing = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><relatedInvestigation><code code="2"/></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let duplicate = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><priorityNumber value="1"/><relatedInvestigation><code code="2"/></relatedInvestigation></outboundRelationship><outboundRelationship><priorityNumber value="1"/><relatedInvestigation><code code="2"/></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let invalid = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><priorityNumber value="2"/><relatedInvestigation><code code="2"/></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		assert_eq!(c_2_r_5_errors(missing).len(), 1);
		assert_eq!(c_2_r_5_errors(duplicate).len(), 1);
		assert_eq!(c_2_r_5_errors(invalid).len(), 1);
	}

	#[test]
	fn c_rules_emit_official_codes() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><subjectOf2><investigationCharacteristic><code code="2" codeSystem="2.16.840.1.113883.3.989.2.1.1.23"/><value value="false"/></investigationCharacteristic></subjectOf2></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = Vec::new();
		run(&mut xpath, &mut errors);
		let codes = errors
			.into_iter()
			.filter_map(|error| error.code)
			.collect::<Vec<_>>();
		assert!(codes.contains(&"C.1.9.1".to_string()));
		assert!(codes.contains(&"C.2.r.5".to_string()));
	}
}
