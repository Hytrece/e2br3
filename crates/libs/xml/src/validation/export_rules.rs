use crate::{Error, Result, XmlValidationError};
use lib_core::regulatory::RegulatoryAuthority;
use libxml::parser::Parser;
use libxml::xpath::Context;

mod authority;

/// Validates XML invariants that would make an authority payload internally
/// inconsistent. Case business rules are intentionally not evaluated here.
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
	reject(
		&mut xpath,
		&mut errors,
		"N.2.r.1",
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV[hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension != hl7:controlActProcess/hl7:subject/hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']/@extension]",
		"N.2.r.1 must be identical to C.1.1.",
	);
	reject(
		&mut xpath,
		&mut errors,
		"N.2.r.4",
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV[hl7:creationTime/@value != hl7:controlActProcess/hl7:effectiveTime/@value]",
		"N.2.r.4 must be identical to C.1.2.",
	);
	authority::run(&mut xpath, authority, &mut errors);
	Ok(errors)
}

fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	expression: &str,
	message: &str,
) {
	if matches(xpath, expression) {
		errors.push(XmlValidationError {
			message: message.to_string(),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}

fn matches(xpath: &mut Context, expression: &str) -> bool {
	!xpath
		.evaluate(expression)
		.expect("static XML-integrity XPath must compile")
		.get_nodes_as_vec()
		.is_empty()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn reports_only_xml_integrity_errors() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><id root="2.16.840.1.113883.3.989.2.1.3.1" extension="message"/><creationTime value="20260810"/><controlActProcess><effectiveTime value="20260811"/><subject><investigationEvent><id root="2.16.840.1.113883.3.989.2.1.3.1" extension="case"/></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let errors = validate_export_rules(xml, RegulatoryAuthority::Ich).unwrap();
		let codes = errors
			.iter()
			.filter_map(|error| error.code.as_deref())
			.collect::<Vec<_>>();
		assert_eq!(codes, ["N.2.r.1", "N.2.r.4"]);
	}

	#[test]
	fn ignores_case_business_rules() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><name code="wrong"/><PORR_IN049016UV><id root="2.16.840.1.113883.3.989.2.1.3.1" extension="not-a-country-profile"/><creationTime value="20990101"/><controlActProcess><effectiveTime value="20990101"/><subject><investigationEvent><id root="2.16.840.1.113883.3.989.2.1.3.1" extension="not-a-country-profile"/></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		assert!(validate_export_rules(xml, RegulatoryAuthority::Ich)
			.unwrap()
			.is_empty());
	}

	#[test]
	fn regional_fields_are_a_final_authority_invariant() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><raceCode/></MCCI_IN200100UV01>"#;
		assert!(validate_export_rules(xml, RegulatoryAuthority::Fda)
			.unwrap()
			.is_empty());
		let errors = validate_export_rules(xml, RegulatoryAuthority::Ich).unwrap();
		assert_eq!(errors.len(), 1);
		assert_eq!(
			errors[0].message,
			"XML for ICH contains FDA regional fields."
		);
	}
}
