use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	let now = time::OffsetDateTime::now_utc();
	let now = format!(
		"{:04}{:02}{:02}{:02}{:02}{:02}",
		now.year(),
		now.month() as u8,
		now.day(),
		now.hour(),
		now.minute(),
		now.second()
	);
	c_1_2(xpath, errors, &now);
	r0009(xpath, errors);
	r0010(xpath, errors);
	r0012(xpath, errors);
	r0013(xpath, errors);
	r0014(xpath, errors);
	r0015(xpath, errors);
	r0016(xpath, errors);
	r0017(xpath, errors);
	r0018(xpath, errors);
	r0019(xpath, errors);
	r0020(xpath, errors);
	r0021(xpath, errors);
	r0022(xpath, errors);
	r0023(xpath, errors);
	r0008(xpath, errors);
	r0024(xpath, errors);
	r0025(xpath, errors);
	r0026(xpath, errors);
	r0102(xpath, errors);
	r0103(xpath, errors);
	r0104(xpath, errors);
	r0107(xpath, errors);
	r0108(xpath, errors);
	r0109(xpath, errors);
	r0110(xpath, errors);
	r0111(xpath, errors);
	r0112(xpath, errors);
	r0113(xpath, errors);
	w0001(xpath, errors);
	w0002(xpath, errors);
	c_3_3_1(xpath, errors);
	c_3_3_2(xpath, errors);
	c_3_3_3(xpath, errors);
	c_3_3_5(xpath, errors);
	c_3_4_1(xpath, errors);
	c_3_4_2(xpath, errors);
	c_3_4_4(xpath, errors);
	c_3_4_5(xpath, errors);
	c_3_4_6(xpath, errors);
	c_3_4_8(xpath, errors);
}

fn c_1_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>, now: &str) {
	reject(xpath, errors, "C.1.2", &format!("//hl7:PORR_IN049016UV/hl7:controlActProcess/hl7:effectiveTime/@value[number(substring(.,1,14)) > {now}]"), true);
}

fn r0009(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0009", "//hl7:investigationEvent[hl7:component/hl7:observationEvent[hl7:code[@code='1']]/hl7:value/@value='true' and not(hl7:reference/hl7:document/hl7:title[normalize-space()])]", true);
}

fn r0010(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0010", "//hl7:document/hl7:text[@compression or (normalize-space() and not(@representation='B64'))]", true);
}

fn r0012(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0012", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='true' and .//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@value='true' and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='1' or .//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='4')]", true);
}

fn r0013(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0013", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='true' and (.//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@value='false' or .//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@nullFlavor='NI') and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='2' or .//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='5')]", true);
}

fn r0014(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0014", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[((.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='false') or (.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@nullFlavor='NI')) and .//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@value='true' and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='1')]", true);
}

fn r0015(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0015", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[((.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@value='false') or (.//hl7:observationEvent[hl7:code[@code='C156384']]/hl7:value/@nullFlavor='NI')) and ((.//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@value='false') or (.//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@nullFlavor='NI')) and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='2')]", true);
}

fn r0016(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0016", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[.//hl7:observationEvent[hl7:code[@code='23']]/hl7:value/@value='true' and ((.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='1' and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='1')) or (.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not(.//hl7:observationEvent[hl7:code[@code='C54588']]/hl7:value/@code='6')))]", true);
}

fn r0017(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0017", "//hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2']]/hl7:value/@value='true' and not(hl7:subjectOf1/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']/@assigningAuthorityName)]", true);
}

fn r0018(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0018", "//hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3'][@assigningAuthorityName and not(@extension)]", true);
}

fn r0019(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0019", "//hl7:relatedInvestigation[hl7:code[@code='2']][hl7:subjectOf2/hl7:controlActEvent/hl7:priorityNumber/@value='1' and not(.//*[@codeSystem='1.0.3166.1.2.2']/@code)]", true);
}

fn r0020(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0020", "//hl7:relatedInvestigation[hl7:code[@code='2']][hl7:subjectOf2/hl7:controlActEvent/hl7:priorityNumber/@value='1' and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.6']/@code or .//*[@codeSystem='2.16.840.1.113883.3.26.1.1']/@code)]", true);
}

fn r0021(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0021", "//hl7:PORR_IN049016UV[not(.//hl7:relatedInvestigation[hl7:code[@code='2']]/hl7:subjectOf2/hl7:controlActEvent/hl7:priorityNumber/@value='1')]", true);
}

fn r0022(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0022", "//hl7:assignedEntity[hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7' and not(@code='7')] and not(hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name[normalize-space()])]", true);
}

fn r0023(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0023", "//hl7:investigationEvent[hl7:subjectOf2/hl7:investigationCharacteristic/hl7:value[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2' and @code='2'] and not(.//hl7:researchStudy/hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code)]", true);
}

fn r0008(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0008", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE') and (.//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension or .//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']/@extension) and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code[.='1' or .='2' or .='3'] and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2')]", true);
}

fn r0024(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0024", "//hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code[.='1' or .='2'] and not(.//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension[string-length(.)=6 and translate(.,'0123456789','')=''])] | //hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension[string-length(.) != 6 or translate(.,'0123456789','') != '']", true);
}

fn r0025(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0025", "//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not(.//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']/@extension[string-length(.)=6 and translate(.,'0123456789','')=''])] | //hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']/@extension[string-length(.) != 6 or translate(.,'0123456789','') != '']", true);
}

fn r0026(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0026", "//hl7:PORR_IN049016UV[.//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension and not(.//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.3'][@nullFlavor='NA' or @extension])] | //hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.3'][@nullFlavor and not(@nullFlavor='NA')]", true);
}

fn r0102(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0102", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE') and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code[.='1' or .='2' or .='3'])]", true);
}

fn r0103(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0103", "//hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='1' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code]", true);
}

fn r0104(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0104", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2' and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code[.='1' or .='2' or .='3'])]", true);
}

fn r0107(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0107", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER' or hl7:receiver/hl7:device/hl7:id/@extension='CBER') and .//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']]", true);
}

fn r0108(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0108", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER' or hl7:receiver/hl7:device/hl7:id/@extension='CBER') and .//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']]", true);
}

fn r0109(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0109", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER' or hl7:receiver/hl7:device/hl7:id/@extension='CBER') and .//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.3']]", true);
}

fn r0110(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0110", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']/@extension and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code) and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='1')]", true);
}

fn r0111(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0111", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE' and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='2')]", true);
}

fn r0112(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0112", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code[.='3' or .='4']]", true);
}

fn r0113(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0113", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND') and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.2']/@code='1' and .//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code]", true);
}

fn w0001(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0001", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[.//hl7:primaryRole/hl7:player1/hl7:name[normalize-space()='AGGREGATE' or normalize-space()='AGGREGRATE'] and not(.//hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']/@extension)]", false);
}

fn w0002(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "W0002", "//hl7:PORR_IN049016UV[.//hl7:primaryRole/hl7:player1/hl7:name[normalize-space()='AGGREGATE' or normalize-space()='AGGREGRATE'] and not(.//*[@codeSystem='2.16.840.1.113883.3.989.2.1.1.8']/@code='1')]", false);
}

fn sender_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	relative: &str,
) {
	reject(xpath, errors, code, &format!("//hl7:assignedEntity[hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7'] and not({relative})]"), true);
}

fn c_3_3_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.3.1",
		"hl7:representedOrganization/hl7:name[normalize-space()]",
	);
}
fn c_3_3_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.3.2",
		"hl7:assignedPerson/hl7:name/hl7:prefix[normalize-space()]",
	);
}
fn c_3_3_3(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.3.3",
		"hl7:assignedPerson/hl7:name/hl7:given[1][normalize-space()]",
	);
}
fn c_3_3_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.3.5",
		"hl7:assignedPerson/hl7:name/hl7:family[normalize-space()]",
	);
}
fn c_3_4_1(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.1",
		"hl7:addr/hl7:streetAddressLine[1][normalize-space()]",
	);
}
fn c_3_4_2(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.2",
		"hl7:addr/hl7:city[normalize-space()]",
	);
}
fn c_3_4_4(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.4",
		"hl7:addr/hl7:postalCode[normalize-space()]",
	);
}
fn c_3_4_5(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.5",
		".//*[@codeSystem='1.0.3166.1.2.2']/@code",
	);
}
fn c_3_4_6(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.6",
		"hl7:telecom[starts-with(@value,'tel:')]",
	);
}
fn c_3_4_8(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	sender_required(
		xpath,
		errors,
		"C.3.4.8",
		"hl7:telecom[starts-with(@value,'mailto:')]",
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
	#[test]
	fn c_rule_uses_exact_code() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV/></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		run(&mut xpath, &mut errors);
		assert!(errors
			.iter()
			.any(|error| error.code.as_deref() == Some("R0021")
				&& error.blocking == Some(true)));
	}

	#[test]
	fn premarket_and_aggregate_rules_keep_ids_and_severity() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><receiver><device><id extension="ZZFDA_PREMKT"/></device></receiver><PORR_IN049016UV><receiver><device><id extension="CDER_IND"/></device></receiver><controlActProcess><subject><investigationEvent><subjectOf2><investigationCharacteristic><value code="1" codeSystem="2.16.840.1.113883.3.989.2.1.1.2"/></investigationCharacteristic></subjectOf2><component><adverseEventAssessment><subject1><primaryRole><player1><name>AGGREGATE</name></player1></primaryRole></subject1></adverseEventAssessment></component></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		run(&mut xpath, &mut errors);
		assert!(errors
			.iter()
			.any(|error| error.code.as_deref() == Some("R0024")
				&& error.blocking == Some(true)));
		assert!(errors
			.iter()
			.any(|error| error.code.as_deref() == Some("W0002")
				&& error.blocking == Some(false)));
	}

	#[test]
	fn route_specific_rules_do_not_lose_official_ids() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><receiver><device><id extension="ZZFDA"/></device></receiver><PORR_IN049016UV><receiver><device><id extension="CDER"/></device></receiver><id root="2.16.840.1.113883.3.989.5.1.2.2.1.2.1" extension="123456"/><observationEvent><code code="C156384"/><value value="false"/></observationEvent><observationEvent><code code="23"/><value value="true"/></observationEvent></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		r0014(&mut xpath, &mut errors);
		r0107(&mut xpath, &mut errors);
		assert_eq!(
			errors
				.iter()
				.filter_map(|error| error.code.as_deref())
				.collect::<Vec<_>>(),
			["R0014", "R0107"]
		);
	}
}
