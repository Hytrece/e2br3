use crate::XmlValidationError;
use libxml::xpath::Context;

const REPORT: &str = "//hl7:PORR_IN049016UV";
const DRUG: &str = "//hl7:PORR_IN049016UV//hl7:organizer[hl7:code[@code='4']]/hl7:component/hl7:substanceAdministration";
const DOSAGE: &str = "//hl7:PORR_IN049016UV//hl7:organizer[hl7:code[@code='4']]/hl7:component/hl7:substanceAdministration/hl7:outboundRelationship2[@typeCode='COMP']/hl7:substanceAdministration";
const IND: &str = "2.16.840.1.113883.3.989.5.1.2.2.1.2.1";
const PRE_ANDA: &str = "2.16.840.1.113883.3.989.5.1.2.2.1.2.2";

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	w0005(xpath, errors);
	r0069(xpath, errors);
	r0070(xpath, errors);
	r0071(xpath, errors);
	r0072(xpath, errors);
	r0073(xpath, errors);
	r0114(xpath, errors);
	r0074(xpath, errors);
	r0075(xpath, errors);
	r0076(xpath, errors);
	r0077(xpath, errors);
	r0078(xpath, errors);
	r0079(xpath, errors);
	r0080(xpath, errors);
	r0081(xpath, errors);
	r0082(xpath, errors);
	r0083(xpath, errors);
	r0084(xpath, errors);
	r0085(xpath, errors);
	r0086(xpath, errors);
	r0087(xpath, errors);
	r0088(xpath, errors);
	r0089(xpath, errors);
	r0090(xpath, errors);
	r0091(xpath, errors);
	w0006(xpath, errors);
	r0092(xpath, errors);
	r0093(xpath, errors);
	w0007(xpath, errors);
	r0096(xpath, errors);
}

fn roles() -> &'static str {
	".//hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.13']/@code"
}

fn w0005(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(
		xpath,
		errors,
		"W0005",
		"//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and (.//hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.13']/@code)[1][not(.='1' or .='3' or .='4')]]",
		false,
	);
}

fn r0069(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0069",&format!("{REPORT}[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and not({}[.='1' or .='3' or .='4'])]",roles()),true);
}

fn r0070(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0070",&format!("{REPORT}[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not({}[.='1' or .='3'])]",roles()),true);
}

fn r0071(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0071",&format!("{REPORT}[hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not({}[.='1' or .='3' or .='4'])]",roles()),true);
}

fn r0072(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0072", &format!("{REPORT}[.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='true']//hl7:organizer[hl7:code[@code='4']]/hl7:component/hl7:substanceAdministration[.//hl7:characteristic[hl7:code[@code='C54026']]/hl7:value/@value='true' and hl7:id/@root = ancestor::hl7:PORR_IN049016UV//hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.13' and @code='4']/../hl7:subject2/hl7:productUseReference/hl7:id/@root and not(hl7:id/@root = ancestor::hl7:PORR_IN049016UV//hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value[@codeSystem='2.16.840.1.113883.3.989.5.1.2.1.1.8' and @code='1']/../hl7:subject2/hl7:productUseReference/hl7:id/@root)]"), true);
}

fn r0073(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0073",
		&format!("{DRUG}//hl7:ingredient/hl7:quantity/hl7:numerator"),
		"@value",
		"@unit",
	);
}

fn r0114(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0114", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code[.='2' or .='3']]//hl7:approval/hl7:id/@extension[contains(translate(., 'abcdefghijklmnopqrstuvwxyz', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'), 'IND') or contains(translate(., 'abcdefghijklmnopqrstuvwxyz-', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'), 'PANDA')]", true);
}

fn r0074(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0074", &format!("{DRUG}//hl7:approval[hl7:id/@extension and not(hl7:author/hl7:territorialAuthority/hl7:territory/hl7:code/@code)]"), true);
}

fn r0075(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0075",
		&format!("{DOSAGE}/hl7:doseQuantity"),
		"@value",
		"@unit",
	);
}

fn r0076(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0076",
		&format!("{DOSAGE}//hl7:period"),
		"@value",
		"@unit",
	);
}

fn r0077(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0077",
		&format!("{DOSAGE}//hl7:width"),
		"@unit",
		"@value",
	);
}

fn r0078(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0078",
		&format!("{DOSAGE}//hl7:width"),
		"@value",
		"@unit",
	);
}

fn r0079(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0079", &format!("{DRUG}/hl7:outboundRelationship2[@typeCode='SUMM']/hl7:observation[hl7:code[@code='14']]/hl7:value"), "@unit", "@value");
}

fn r0080(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0080", &format!("{DRUG}/hl7:outboundRelationship2[@typeCode='SUMM']/hl7:observation[hl7:code[@code='14']]/hl7:value"), "@value", "@unit");
}

fn r0081(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0081", &format!("{DRUG}/hl7:outboundRelationship2[@typeCode='PERT']/hl7:observation[hl7:code[@code='16']]/hl7:value"), "@unit", "@value");
}

fn r0082(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0082", &format!("{DRUG}/hl7:outboundRelationship2[@typeCode='PERT']/hl7:observation[hl7:code[@code='16']]/hl7:value"), "@value", "@unit");
}

fn r0083(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0083", &format!("{DRUG}/hl7:inboundRelationship[@typeCode='RSON']/hl7:observation[hl7:code[@code='19']]/hl7:value"), "@code", "@codeSystemVersion");
}

fn r0084(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(xpath, errors, "R0084", &format!("{DRUG}/hl7:inboundRelationship[@typeCode='RSON']/hl7:observation[hl7:code[@code='19']]/hl7:value"), "@codeSystemVersion", "@code");
}

fn premarket_assessment(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	value: &str,
) {
	let condition = format!(".//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and .//hl7:id[@root='{IND}']/@extension");
	reject(xpath, errors, code, &format!("{REPORT}[{condition}]//hl7:causalityAssessment[hl7:code[@code='39'] and not({value})] | {REPORT}[{condition} and not(.//hl7:causalityAssessment[hl7:code[@code='39']])]"), true);
}

fn r0085(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	premarket_assessment(
		xpath,
		errors,
		"R0085",
		"hl7:author/hl7:assignedEntity/hl7:code/hl7:originalText[normalize-space()]",
	);
}

fn r0086(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	premarket_assessment(
		xpath,
		errors,
		"R0086",
		"hl7:methodCode/hl7:originalText[normalize-space()]",
	);
}

fn r0087(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	premarket_assessment(xpath, errors, "R0087", "hl7:value[normalize-space()='Suspected' or normalize-space()='Not Suspected']");
}

fn r0088(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0088",
		&format!(
			"{DRUG}/hl7:outboundRelationship1[@typeCode='SAS']/hl7:pauseQuantity"
		),
		"@unit",
		"@value",
	);
}

fn r0089(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0089",
		&format!(
			"{DRUG}/hl7:outboundRelationship1[@typeCode='SAS']/hl7:pauseQuantity"
		),
		"@value",
		"@unit",
	);
}

fn r0090(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0090",
		&format!(
			"{DRUG}/hl7:outboundRelationship1[@typeCode='SAE']/hl7:pauseQuantity"
		),
		"@unit",
		"@value",
	);
}

fn r0091(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	pair(
		xpath,
		errors,
		"R0091",
		&format!(
			"{DRUG}/hl7:outboundRelationship1[@typeCode='SAE']/hl7:pauseQuantity"
		),
		"@value",
		"@unit",
	);
}

fn w0006(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0006", &format!("{REPORT}[.//hl7:id[@root='{PRE_ANDA}']/@extension]//hl7:organizer[hl7:code[@code='4']]/hl7:component/hl7:substanceAdministration[not(hl7:outboundRelationship2[@typeCode='REFR']/hl7:observation[hl7:code[@code='9']]/hl7:value[@code='1' or @code='2' or @nullFlavor='NA'])]"), false);
}

fn r0092(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0092","//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='5' and not(.//hl7:organizer[hl7:code[@code='4']]/hl7:component/hl7:substanceAdministration[.//hl7:characteristic[hl7:code[@code='C54026']]/hl7:value/@value='true' and hl7:id/@root = ancestor::hl7:PORR_IN049016UV//hl7:causalityAssessment[hl7:code[@code='20']]/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.13' and @code='1']/../hl7:subject2/hl7:productUseReference/hl7:id/@root])]",true);
}

fn r0093(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath,errors,"R0093","//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER']//hl7:partProduct[@classCode='DEV']/hl7:asManufacturedProduct[hl7:subjectOf/hl7:characteristic[hl7:code[@code='C54026']]/hl7:value/@value='true' and not(hl7:subjectOf/hl7:characteristic[hl7:code[@code='C54451']]/hl7:value/@code)]",true);
}

fn w0007(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0007", "//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='4']//hl7:partProduct[@classCode='DEV']/hl7:asManufacturedProduct[hl7:subjectOf/hl7:characteristic[hl7:code[@code='C54026']]/hl7:value/@value='true' and not(hl7:subjectOf/hl7:characteristic[hl7:code[@code='C54594']]/hl7:value/@code)]", false);
}

fn missing_device_names() -> &'static str {
	"/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='true']//hl7:partProduct[@classCode='DEV'][not(hl7:name[1][normalize-space()]) and not(hl7:name[2][normalize-space()]) and not(hl7:code/@code)]"
}

fn r0096(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0096", missing_device_names(), true);
}

fn pair(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	node: &str,
	present: &str,
	required: &str,
) {
	reject(
		xpath,
		errors,
		code,
		&format!("{node}[{present} and not({required})]"),
		true,
	);
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

	fn validate(
		xml: &[u8],
		rule: fn(&mut Context, &mut Vec<XmlValidationError>),
	) -> Vec<XmlValidationError> {
		let doc = Parser::default().parse_string(xml).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		rule(&mut xpath, &mut errors);
		errors
	}

	#[test]
	fn warning_code_and_severity_are_exact() {
		let errors = validate(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><receiver><device><id extension="CDER"/></device></receiver><causalityAssessment><code code="20"/><value code="2" codeSystem="2.16.840.1.113883.3.989.2.1.1.13"/></causalityAssessment></PORR_IN049016UV></MCCI_IN200100UV01>"#, w0005);
		assert!(errors.iter().any(
			|e| e.code.as_deref() == Some("W0005") && e.blocking == Some(false)
		));
	}

	#[test]
	fn malfunction_in_another_report_does_not_satisfy_r0092() {
		let errors = validate(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><receiver><device><id extension="CDER"/></device></receiver><observationEvent><code code="C54588"/><value code="5"/></observationEvent></PORR_IN049016UV><PORR_IN049016UV><receiver><device><id extension="CDER"/></device></receiver><organizer><code code="4"/><component><substanceAdministration><id root="drug-2"/><characteristic><code code="C54026"/><value value="true"/></characteristic></substanceAdministration></component></organizer><causalityAssessment><code code="20"/><value code="1" codeSystem="2.16.840.1.113883.3.989.2.1.1.13"/><subject2><productUseReference><id root="drug-2"/></productUseReference></subject2></causalityAssessment></PORR_IN049016UV></MCCI_IN200100UV01>"#, r0092);
		assert_eq!(errors[0].code.as_deref(), Some("R0092"));
	}

	#[test]
	fn strength_pair_is_scoped_to_one_ingredient() {
		let errors = validate(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><organizer><code code="4"/><component><substanceAdministration><ingredient><quantity><numerator value="10"/></quantity></ingredient><ingredient><quantity><numerator unit="mg"/></quantity></ingredient></substanceAdministration></component></organizer></PORR_IN049016UV></MCCI_IN200100UV01>"#, r0073);
		assert_eq!(errors[0].code.as_deref(), Some("R0073"));
	}
}
