use crate::{Error, Result, XmlValidationError};
use lib_core::regulatory::RegulatoryAuthority;
use libxml::parser::Parser;
use libxml::xpath::Context;

pub fn validate_export_rules(
	xml: &[u8],
	authority: RegulatoryAuthority,
) -> Result<Vec<XmlValidationError>> {
	let xml = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let document =
		Parser::default()
			.parse_string(xml)
			.map_err(|err| Error::InvalidXml {
				message: format!("XML parse error: {err}"),
				line: None,
				column: None,
			})?;
	let mut xpath = Context::new(&document).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ =
		xpath.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");

	let mut errors = Vec::new();
	validate_ich_xml_root_itsversion_required(&mut xpath, &mut errors);
	validate_ich_xml_root_schemalocation_required(&mut xpath, &mut errors);
	validate_ich_xml_bl_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_bl_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_code_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_code_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_country_code_format_required(&mut xpath, &mut errors);
	validate_ich_xml_dose_quantity_value_unit_required(&mut xpath, &mut errors);
	validate_ich_xml_effectivetime_width_requires_bound(&mut xpath, &mut errors);
	validate_ich_xml_inv_char_bl_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_inv_char_bl_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_ivl_ts_operator_a_bound_required(&mut xpath, &mut errors);
	validate_ich_xml_low_high_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_low_high_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_meddra_code_format_required(&mut xpath, &mut errors);
	validate_ich_xml_meddra_version_required(&mut xpath, &mut errors);
	validate_ich_xml_period_value_unit_required(&mut xpath, &mut errors);
	validate_ich_xml_pivl_ts_period_required(&mut xpath, &mut errors);
	validate_ich_xml_pivl_ts_period_value_unit_required(&mut xpath, &mut errors);
	validate_ich_xml_sxpr_ts_comp_required(&mut xpath, &mut errors);
	validate_ich_xml_telecom_format_required(&mut xpath, &mut errors);
	validate_ich_xml_telecom_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_telecom_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_testresult_ivl_pq_component_required(&mut xpath, &mut errors);
	validate_ich_xml_testresult_ivl_pq_value_unit_required(&mut xpath, &mut errors);
	validate_ich_xml_testresult_pq_value_unit_required(&mut xpath, &mut errors);
	validate_ich_xml_testresult_xsi_type_unsupported(&mut xpath, &mut errors);
	validate_ich_xml_text_nullflavor_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_text_nullflavor_required(&mut xpath, &mut errors);
	validate_ich_xml_placeholder_value_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_document_text_compression_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_summary_language_ja_forbidden(&mut xpath, &mut errors);
	validate_ich_xml_structural_empty_prune(&mut xpath, &mut errors);
	validate_authority_field_filtering(&mut xpath, authority, &mut errors);
	Ok(errors)
}

fn validate_authority_field_filtering(
	xpath: &mut Context,
	authority: RegulatoryAuthority,
	errors: &mut Vec<XmlValidationError>,
) {
	match authority {
		RegulatoryAuthority::Fda => validate_fda_authority_fields(xpath, errors),
		RegulatoryAuthority::Mfds => validate_mfds_authority_fields(xpath, errors),
		RegulatoryAuthority::Ich => validate_ich_authority_fields(xpath, errors),
	}
}

// export-policy:FDA forbids MFDS regional fields
fn validate_fda_authority_fields(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_authority_error_if_present(
		xpath,
		errors,
		mfds_regional_xpath(),
		"FDA",
		"MFDS",
	);
}

// export-policy:MFDS forbids FDA regional fields
fn validate_mfds_authority_fields(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_authority_error_if_present(
		xpath,
		errors,
		fda_regional_xpath(),
		"MFDS",
		"FDA",
	);
}

// export-policy:ICH forbids regional fields
fn validate_ich_authority_fields(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_authority_error_if_present(
		xpath,
		errors,
		mfds_regional_xpath(),
		"ICH",
		"MFDS",
	);
	push_authority_error_if_present(
		xpath,
		errors,
		fda_regional_xpath(),
		"ICH",
		"FDA",
	);
}

fn mfds_regional_xpath() -> &'static str {
	"//*[@codeSystem and starts-with(@codeSystem, '2.16.840.1.113883.3.989.5.1.10.')] | //*[@root and starts-with(@root, '2.16.840.1.113883.3.989.5.1.10.')] | //*[@code and contains(@code, '.KR.') ]"
}

fn fda_regional_xpath() -> &'static str {
	"//*[@codeSystem and starts-with(@codeSystem, '2.16.840.1.113883.3.989.5.1.2.')] | //*[@root and starts-with(@root, '2.16.840.1.113883.3.989.5.1.2.')] | //hl7:partProduct[@classCode='DEV'] | //hl7:raceCode | //hl7:observation[hl7:code[@code='C16564']] | //hl7:characteristic[hl7:code[@code='C54026' or @code='C54592' or @code='C54451' or @code='C54594' or @code='C54595' or @code='C94031']]"
}

fn push_authority_error_if_present(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	expression: &str,
	authority: &str,
	regional_authority: &str,
) {
	let present = xpath
		.evaluate(expression)
		.map(|value| !value.get_nodes_as_vec().is_empty())
		.unwrap_or(false);
	if present {
		errors.push(XmlValidationError {
			message: format!(
				"{authority} export contains {regional_authority} regional fields."
			),
			code: None,
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}

// e2b:ICH.XML.CODE.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_code_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.CODE.NULLFLAVOR.FORBIDDEN", "//*[self::hl7:code or @xsi:type='CE' or @xsi:type='CD'][(@code or @codeSystem) and @nullFlavor]", "Coded values cannot carry code data and nullFlavor together.");
}

// e2b:ICH.XML.CODE.NULLFLAVOR.REQUIRED
fn validate_ich_xml_code_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.CODE.NULLFLAVOR.REQUIRED", "//*[self::hl7:code or @xsi:type='CE' or @xsi:type='CD'][not(@code) and not(@codeSystem) and not(hl7:originalText) and not(@nullFlavor)]", "Coded values without code data or originalText require nullFlavor.");
}

// e2b:ICH.XML.DOSE_QUANTITY.VALUE_UNIT.REQUIRED
fn validate_ich_xml_dose_quantity_value_unit_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.DOSE_QUANTITY.VALUE_UNIT.REQUIRED",
		"//hl7:doseQuantity[not(@nullFlavor) and (not(@value) or not(@unit))]",
		"doseQuantity requires value and unit.",
	);
}

// e2b:ICH.XML.EFFECTIVETIME.WIDTH.REQUIRES_BOUND
fn validate_ich_xml_effectivetime_width_requires_bound(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.EFFECTIVETIME.WIDTH.REQUIRES_BOUND",
		"//hl7:effectiveTime[hl7:width and not(hl7:low or hl7:high)]",
		"effectiveTime with width requires low or high.",
	);
}

// e2b:ICH.XML.IVL_TS.OPERATOR_A.BOUND_REQUIRED
fn validate_ich_xml_ivl_ts_operator_a_bound_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.IVL_TS.OPERATOR_A.BOUND_REQUIRED", "//hl7:comp[@xsi:type='IVL_TS' and @operator='A' and not(hl7:low or hl7:high or hl7:width)]", "IVL_TS operator A requires low, high, or width.");
}

// e2b:ICH.XML.LOW_HIGH.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_low_high_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.LOW_HIGH.NULLFLAVOR.FORBIDDEN",
		"//*[self::hl7:low or self::hl7:high][@value and @nullFlavor]",
		"low/high cannot carry value and nullFlavor together.",
	);
}

// e2b:ICH.XML.LOW_HIGH.NULLFLAVOR.REQUIRED
fn validate_ich_xml_low_high_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.LOW_HIGH.NULLFLAVOR.REQUIRED",
		"//*[self::hl7:low or self::hl7:high][not(@value) and not(@nullFlavor)]",
		"low/high without value requires nullFlavor.",
	);
}

// e2b:ICH.XML.PERIOD.VALUE_UNIT.REQUIRED
fn validate_ich_xml_period_value_unit_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.PERIOD.VALUE_UNIT.REQUIRED",
		"//hl7:period[not(@nullFlavor) and (not(@value) or not(@unit))]",
		"period requires value and unit.",
	);
}

// e2b:ICH.XML.PIVL_TS.PERIOD.REQUIRED
fn validate_ich_xml_pivl_ts_period_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.PIVL_TS.PERIOD.REQUIRED",
		"//hl7:comp[@xsi:type='PIVL_TS' and not(hl7:period)]",
		"PIVL_TS requires period.",
	);
}

// e2b:ICH.XML.PIVL_TS.PERIOD.VALUE_UNIT.REQUIRED
fn validate_ich_xml_pivl_ts_period_value_unit_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.PIVL_TS.PERIOD.VALUE_UNIT.REQUIRED", "//hl7:comp[@xsi:type='PIVL_TS']/hl7:period[not(@nullFlavor) and (not(@value) or not(@unit))]", "PIVL_TS period requires value and unit.");
}

// e2b:ICH.XML.SXPR_TS.COMP.REQUIRED
fn validate_ich_xml_sxpr_ts_comp_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.SXPR_TS.COMP.REQUIRED",
		"//hl7:effectiveTime[@xsi:type='SXPR_TS' and not(hl7:comp)]",
		"SXPR_TS requires at least one comp.",
	);
}

const TEST_RESULT_VALUE_XPATH: &str =
	"//hl7:organizer[hl7:code[@code='3']]/hl7:component/hl7:observation/hl7:value";

// e2b:ICH.XML.TESTRESULT.IVL_PQ.COMPONENT.REQUIRED
fn validate_ich_xml_testresult_ivl_pq_component_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.TESTRESULT.IVL_PQ.COMPONENT.REQUIRED", &format!("{TEST_RESULT_VALUE_XPATH}[@xsi:type='IVL_PQ' and not(hl7:low or hl7:high or hl7:center)]"), "IVL_PQ test results require low, high, or center.");
}

// e2b:ICH.XML.TESTRESULT.IVL_PQ.VALUE_UNIT.REQUIRED
fn validate_ich_xml_testresult_ivl_pq_value_unit_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.TESTRESULT.IVL_PQ.VALUE_UNIT.REQUIRED", &format!("{TEST_RESULT_VALUE_XPATH}[@xsi:type='IVL_PQ']/*[self::hl7:low or self::hl7:high or self::hl7:center][not(@nullFlavor) and (not(@value) or not(@unit))]"), "IVL_PQ components require value and unit.");
}

// e2b:ICH.XML.TESTRESULT.PQ.VALUE_UNIT.REQUIRED
fn validate_ich_xml_testresult_pq_value_unit_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.TESTRESULT.PQ.VALUE_UNIT.REQUIRED", &format!("{TEST_RESULT_VALUE_XPATH}[@xsi:type='PQ' and not(@nullFlavor) and (not(@value) or not(@unit))]"), "PQ test results require value and unit.");
}

// e2b:ICH.XML.TESTRESULT.XSI_TYPE.UNSUPPORTED
fn validate_ich_xml_testresult_xsi_type_unsupported(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.TESTRESULT.XSI_TYPE.UNSUPPORTED", &format!("{TEST_RESULT_VALUE_XPATH}[@xsi:type and not(@xsi:type='IVL_PQ' or @xsi:type='PQ' or @xsi:type='ED' or @xsi:type='ST' or @xsi:type='BL' or @xsi:type='CE')]"), "Unsupported test result xsi:type.");
}

// e2b:ICH.XML.ROOT.ITSVERSION.REQUIRED
fn validate_ich_xml_root_itsversion_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.ROOT.ITSVERSION.REQUIRED",
		"/*[not(@ITSVersion='XML_1.0')]",
		"Root ITSVersion must be XML_1.0.",
	);
}

// e2b:ICH.XML.ROOT.SCHEMALOCATION.REQUIRED
fn validate_ich_xml_root_schemalocation_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.ROOT.SCHEMALOCATION.REQUIRED",
		"/*[not(@xsi:schemaLocation) or not(contains(@xsi:schemaLocation, concat(local-name(), '.xsd')))]",
		"Root schemaLocation must reference its root schema.",
	);
}

// e2b:ICH.XML.BL.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_bl_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.BL.NULLFLAVOR.FORBIDDEN",
		"//*[@xsi:type='BL' and not(parent::hl7:investigationCharacteristic) and @value and @nullFlavor]",
		"BL values cannot carry value and nullFlavor together.",
	);
}

// e2b:ICH.XML.BL.NULLFLAVOR.REQUIRED
fn validate_ich_xml_bl_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.BL.NULLFLAVOR.REQUIRED",
		"//*[@xsi:type='BL' and not(parent::hl7:investigationCharacteristic) and not(@value) and not(@nullFlavor)]",
		"BL values without value require nullFlavor.",
	);
}

// e2b:ICH.XML.INV_CHAR_BL.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_inv_char_bl_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.INV_CHAR_BL.NULLFLAVOR.FORBIDDEN", "//hl7:investigationCharacteristic/hl7:value[@xsi:type='BL' and @value and @nullFlavor]", "Investigation-characteristic BL cannot carry value and nullFlavor together.");
}

// e2b:ICH.XML.INV_CHAR_BL.NULLFLAVOR.REQUIRED
fn validate_ich_xml_inv_char_bl_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(xpath, errors, "ICH.XML.INV_CHAR_BL.NULLFLAVOR.REQUIRED", "//hl7:investigationCharacteristic/hl7:value[@xsi:type='BL' and not(@value) and not(@nullFlavor)]", "Investigation-characteristic BL without value requires nullFlavor.");
}

// e2b:ICH.XML.COUNTRY.CODE.FORMAT.REQUIRED
fn validate_ich_xml_country_code_format_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.COUNTRY.CODE.FORMAT.REQUIRED",
		"//*[@codeSystem='1.0.3166.1.2.2' and @code and (string-length(@code) != 2 or @code != translate(@code, 'abcdefghijklmnopqrstuvwxyz', 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'))]",
		"ISO country codes must be two uppercase letters.",
	);
}

// e2b:ICH.XML.MEDDRA.CODE.FORMAT.REQUIRED
fn validate_ich_xml_meddra_code_format_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.MEDDRA.CODE.FORMAT.REQUIRED",
		"//*[@codeSystem='2.16.840.1.113883.6.163' and @code and (string-length(@code) != 8 or translate(@code, '0123456789', '') != '')]",
		"MedDRA codes must contain exactly eight digits.",
	);
}

// e2b:ICH.XML.MEDDRA.VERSION.REQUIRED
fn validate_ich_xml_meddra_version_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.MEDDRA.VERSION.REQUIRED",
		"//*[@codeSystem='2.16.840.1.113883.6.163' and @code and not(@codeSystemVersion)]",
		"MedDRA codes require codeSystemVersion.",
	);
}

// e2b:ICH.XML.TELECOM.FORMAT.REQUIRED
fn validate_ich_xml_telecom_format_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.TELECOM.FORMAT.REQUIRED",
		"//hl7:telecom[@value and not(starts-with(@value, 'tel:') or starts-with(@value, 'fax:') or starts-with(@value, 'mailto:'))]",
		"Telecom values must use tel:, fax:, or mailto:.",
	);
}

// e2b:ICH.XML.TELECOM.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_telecom_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.TELECOM.NULLFLAVOR.FORBIDDEN",
		"//hl7:telecom[@value and @nullFlavor]",
		"Telecom cannot carry value and nullFlavor together.",
	);
}

// e2b:ICH.XML.TELECOM.NULLFLAVOR.REQUIRED
fn validate_ich_xml_telecom_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.TELECOM.NULLFLAVOR.REQUIRED",
		"//hl7:telecom[not(@value) and not(@nullFlavor)]",
		"Telecom without value requires nullFlavor.",
	);
}

// e2b:ICH.XML.TEXT.NULLFLAVOR.FORBIDDEN
fn validate_ich_xml_text_nullflavor_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.TEXT.NULLFLAVOR.FORBIDDEN",
		"//*[self::hl7:text or self::hl7:originalText][normalize-space(.) != '' and @nullFlavor]",
		"Text cannot carry content and nullFlavor together.",
	);
}

// e2b:ICH.XML.TEXT.NULLFLAVOR.REQUIRED
fn validate_ich_xml_text_nullflavor_required(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.TEXT.NULLFLAVOR.REQUIRED",
		"//*[self::hl7:text or self::hl7:originalText][normalize-space(.) = '' and not(@nullFlavor) and not(*)]",
		"Empty text requires nullFlavor.",
	);
}

// e2b:ICH.XML.PLACEHOLDER.VALUE.FORBIDDEN
fn validate_ich_xml_placeholder_value_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.PLACEHOLDER.VALUE.FORBIDDEN",
		"//*[@value='G.k.10.r' or @value='C.1.11.1' or @value='D.2.3' or @unit='D.2.2.1b']",
		"Placeholder values are not allowed in exported XML.",
	);
}

// e2b:ICH.XML.DOCUMENT.TEXT.COMPRESSION.FORBIDDEN
fn validate_ich_xml_document_text_compression_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.DOCUMENT.TEXT.COMPRESSION.FORBIDDEN",
		"//hl7:document/hl7:text[@compression]",
		"Exported document text must not carry compression.",
	);
}

// e2b:ICH.XML.SUMMARY.LANGUAGE.JA.FORBIDDEN
fn validate_ich_xml_summary_language_ja_forbidden(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.SUMMARY.LANGUAGE.JA.FORBIDDEN",
		"//hl7:component/hl7:observationEvent[hl7:code[@code='36']]/hl7:value[@language='JA']",
		"Exported case summaries must not carry language='JA'.",
	);
}

// e2b:ICH.XML.STRUCTURAL.EMPTY.PRUNE
fn validate_ich_xml_structural_empty_prune(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
) {
	push_if_present(
		xpath,
		errors,
		"ICH.XML.STRUCTURAL.EMPTY.PRUNE",
		"//*[self::hl7:outboundRelationship2 or self::hl7:component or self::hl7:component1 or self::hl7:subjectOf2 or self::hl7:observation or self::hl7:organizer][not(*) and not(@*) and not(normalize-space())]",
		"Empty optional structural nodes must be removed before export.",
	);
}

fn push_if_present(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &'static str,
	expression: &str,
	message: &'static str,
) {
	let present = xpath
		.evaluate(expression)
		.map(|value| !value.get_nodes_as_vec().is_empty())
		.unwrap_or(false);
	if present {
		errors.push(XmlValidationError {
			message: format!("[{code}] {message}"),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
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

	#[test]
	fn each_export_rule_reports_its_own_code() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3">
			<component/>
			<document><text compression="GZ"/></document>
			<observation><value value="G.k.10.r"/></observation>
		</MCCI_IN200100UV01>"#;
		let codes = validate_export_rules(xml, RegulatoryAuthority::Ich)
			.expect("export rules")
			.into_iter()
			.filter_map(|error| error.code)
			.collect::<Vec<_>>();
		assert!(codes.contains(&"ICH.XML.PLACEHOLDER.VALUE.FORBIDDEN".to_string()));
		assert!(codes
			.contains(&"ICH.XML.DOCUMENT.TEXT.COMPRESSION.FORBIDDEN".to_string()));
		assert!(codes.contains(&"ICH.XML.STRUCTURAL.EMPTY.PRUNE".to_string()));
	}

	#[test]
	fn authority_filtering_has_no_fake_rule_code() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><value codeSystem="2.16.840.1.113883.3.989.5.1.10.1.1"/></MCCI_IN200100UV01>"#;
		let errors = validate_export_rules(xml, RegulatoryAuthority::Fda)
			.expect("authority filtering");
		assert!(errors.iter().any(|error| {
			error.code.is_none() && error.message.contains("MFDS regional fields")
		}));
	}
}
