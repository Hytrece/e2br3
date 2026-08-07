use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	r0027(xpath, errors);
	r0029(xpath, errors);
	r0030(xpath, errors);
	r0031(xpath, errors);
	r0032(xpath, errors);
	r0033(xpath, errors);
	r0034(xpath, errors);
	r0035(xpath, errors);
	r0037(xpath, errors);
	r0038(xpath, errors);
	r0039(xpath, errors);
	r0040(xpath, errors);
	r0041(xpath, errors);
	r0042(xpath, errors);
	r0043(xpath, errors);
	r0044(xpath, errors);
	r0045(xpath, errors);
	r0046(xpath, errors);
	r0047(xpath, errors);
	r0048(xpath, errors);
	r0049(xpath, errors);
	r0050(xpath, errors);
	r0051(xpath, errors);
	r0052(xpath, errors);
	r0053(xpath, errors);
	r0054(xpath, errors);
	r0055(xpath, errors);
	r0056(xpath, errors);
	r0057(xpath, errors);
	fda_d_11_r_1(xpath, errors);
	fda_d_12(xpath, errors);
	w0003(xpath, errors);
	w0004(xpath, errors);
	w0010(xpath, errors);
}

fn r0027(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0027", "//hl7:PORR_IN049016UV[.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value[@value='true'] and .//hl7:characteristic[hl7:code[@code='C54026']]/hl7:value[@value='true'] and .//hl7:observation/hl7:value[@code='10067482' and @codeSystem='2.16.840.1.113883.6.163'] and not(.//hl7:primaryRole/hl7:player1/hl7:name[@nullFlavor='NA'])]", true);
}

fn r0029(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0029",
		"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@unit and not(@value)]",
		true,
	);
}
fn r0030(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0030",
		"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@value and not(@unit)]",
		true,
	);
}
fn r0031(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0031",
		"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='16']]/hl7:value[@unit and not(@value)]",
		true,
	);
}
fn r0032(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"R0032",
		"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='16']]/hl7:value[@value and not(@unit)]",
		true,
	);
}
fn r0033(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0033", "//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation/hl7:code[@codeSystem='2.16.840.1.113883.6.163' and @code and not(@codeSystemVersion)]", true);
}
fn r0034(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0034", "//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation/hl7:code[@codeSystem='2.16.840.1.113883.6.163' and @codeSystemVersion and not(@code)]", true);
}
fn r0035(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0035", "//hl7:PORR_IN049016UV[not(.//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation/hl7:code/@code) and not(.//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='18']]/hl7:value[normalize-space() or @nullFlavor])]", true);
}

const D8: &str = "//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//hl7:substanceAdministration";
fn r0037(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0037",
		D8,
		"RSON",
		"19",
		"@code",
		"@codeSystemVersion",
	);
}
fn r0038(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0038",
		D8,
		"RSON",
		"19",
		"@codeSystemVersion",
		"@code",
	);
}
fn r0039(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0039",
		D8,
		"CAUS",
		"29",
		"@code",
		"@codeSystemVersion",
	);
}
fn r0040(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0040",
		D8,
		"CAUS",
		"29",
		"@codeSystemVersion",
		"@code",
	);
}

fn r0041(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0041", "//hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//hl7:investigationCharacteristic[hl7:code[@code='1']]/hl7:value[@code='2'] and .//hl7:observation[hl7:code[@code='34']]/hl7:value[@value='true'] and not(.//hl7:primaryRole/hl7:player1/hl7:deceasedTime[@value])]", true);
}
fn r0042(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	death_pair(xpath, errors, "R0042", "32", "@code", "@codeSystemVersion");
}
fn r0043(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	death_pair(xpath, errors, "R0043", "32", "@codeSystemVersion", "@code");
}
fn r0044(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0044", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='32']]/hl7:value[@code and @codeSystemVersion and not(normalize-space(hl7:originalText))]", true);
}
fn r0045(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0045", "//hl7:primaryRole[hl7:player1/hl7:deceasedTime[@value] and not(hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]/hl7:value[@value or @nullFlavor])]", true);
}
fn r0046(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	autopsy_pair(xpath, errors, "R0046", "@code", "@codeSystemVersion");
}
fn r0047(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	autopsy_pair(xpath, errors, "R0047", "@codeSystemVersion", "@code");
}
fn r0048(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0048", "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]//hl7:observation[hl7:code[@code='8']]/hl7:value[@code and @codeSystemVersion and not(normalize-space(hl7:originalText))]", true);
}

fn r0049(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0049", "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@unit and not(@value)]", true);
}
fn r0050(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0050", "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value[@value and not(@unit)]", true);
}
fn r0051(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0051", "//hl7:role[@classCode='PRS' and * and not(hl7:associatedPerson/hl7:administrativeGenderCode[@code or @nullFlavor])]", true);
}
fn r0052(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0052", "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation/hl7:code[@codeSystem='2.16.840.1.113883.6.163' and @code and not(@codeSystemVersion)]", true);
}
fn r0053(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0053", "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation/hl7:code[@codeSystem='2.16.840.1.113883.6.163' and @codeSystemVersion and not(@code)]", true);
}
const D10: &str = "//hl7:role[@classCode='PRS']/hl7:subjectOf2/hl7:organizer[hl7:code[@code='2']]//hl7:substanceAdministration";
fn r0054(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0054",
		D10,
		"RSON",
		"19",
		"@code",
		"@codeSystemVersion",
	);
}
fn r0055(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0055",
		D10,
		"RSON",
		"19",
		"@codeSystemVersion",
		"@code",
	);
}
fn r0056(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0056",
		D10,
		"CAUS",
		"29",
		"@code",
		"@codeSystemVersion",
	);
}
fn r0057(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0057",
		D10,
		"CAUS",
		"29",
		"@codeSystemVersion",
		"@code",
	);
}

fn pair(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	rule: &str,
	base: &str,
	relationship: &str,
	observation: &str,
	present: &str,
	missing: &str,
) {
	reject(xpath, errors, rule, &format!("{base}/hl7:outboundRelationship2[@typeCode='{relationship}']/hl7:observation[hl7:code[@code='{observation}']]/hl7:value[{present} and not({missing})]"), true);
}
fn death_pair(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	rule: &str,
	observation: &str,
	present: &str,
	missing: &str,
) {
	reject(xpath, errors, rule, &format!("//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='{observation}']]/hl7:value[{present} and not({missing})]"), true);
}
fn autopsy_pair(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	rule: &str,
	present: &str,
	missing: &str,
) {
	reject(xpath, errors, rule, &format!("//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='5']]//hl7:observation[hl7:code[@code='8']]/hl7:value[{present} and not({missing})]"), true);
}

fn fda_d_11_r_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "FDA.D.11.r.1", "//hl7:PORR_IN049016UV[not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='5') and not(.//hl7:raceCode[@code or @nullFlavor]) and not(.//hl7:observation[hl7:code[@code='C17049']]/hl7:value[@code or @nullFlavor])]", true);
}
fn fda_d_12(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "FDA.D.12", "//hl7:PORR_IN049016UV[not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='5') and not(.//hl7:observation[hl7:code[@code='C16564']]/hl7:value[@code or @nullFlavor])] | //hl7:observation[hl7:code[@code='C16564']]/hl7:value[@code and not(@code='C17459' or @code='C41222')]", true);
}
fn w0003(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0003", "//hl7:primaryRole[(.//hl7:name/@nullFlavor='NA' or .//hl7:name='SUMMARY' or .//hl7:name='AGGREGATE') and not(.//hl7:observation[hl7:code[@code='C17049']]/hl7:value/@nullFlavor='NA')]", false);
}
fn w0004(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0004", "//hl7:primaryRole[(.//hl7:name/@nullFlavor='NA' or .//hl7:name='SUMMARY' or .//hl7:name='AGGREGATE') and not(.//hl7:observation[hl7:code[@code='C16564']]/hl7:value/@nullFlavor='NA')]", false);
}
fn w0010(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0010", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[.//hl7:investigationCharacteristic[hl7:code[@code='1']]/hl7:value/@code='2' and .//hl7:relatedInvestigation[hl7:code[@nullFlavor='NA']]//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']/@extension and .//hl7:studyRegistration/hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension and not(.//hl7:primaryRole/hl7:player1/hl7:name[normalize-space()='AGGREGATE'])]", false);
}

fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	expression: &str,
	blocking: bool,
) {
	if super::super::matches(xpath, expression) {
		errors.push(XmlValidationError {
			message: format!("[{code}] FDA business rule failed."),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(blocking),
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
	fn warning_is_non_blocking() {
		let doc=Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><primaryRole><name>AGGREGATE</name></primaryRole></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		run(&mut xpath, &mut errors);
		assert!(errors.iter().any(
			|e| e.code.as_deref() == Some("W0003") && e.blocking == Some(false)
		));
	}

	#[test]
	fn meddra_pair_rules_keep_patient_and_parent_scope() {
		let doc=Parser::default().parse_string(br#"<primaryRole xmlns="urn:hl7-org:v3"><subjectOf2><organizer><code code="2"/><substanceAdministration><outboundRelationship2 typeCode="RSON"><observation><code code="19"/><value code="123"/></observation></outboundRelationship2></substanceAdministration></organizer></subjectOf2><player1><role classCode="PRS"><subjectOf2><organizer><code code="2"/><substanceAdministration><outboundRelationship2 typeCode="RSON"><observation><code code="19"/><value code="456"/></observation></outboundRelationship2></substanceAdministration></organizer></subjectOf2></role></player1></primaryRole>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		r0037(&mut xpath, &mut errors);
		r0054(&mut xpath, &mut errors);
		assert_eq!(
			errors
				.iter()
				.filter_map(|e| e.code.as_deref())
				.collect::<Vec<_>>(),
			vec!["R0037", "R0054"]
		);
	}
}
