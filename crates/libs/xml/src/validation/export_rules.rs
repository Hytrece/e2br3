use crate::{Error, Result, XmlValidationError};
use lib_core::regulatory::RegulatoryAuthority;
use libxml::parser::Parser;
use libxml::xpath::Context;

mod authority;
mod fda;
mod ich;
mod mfds;

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
	ich::run(&mut xpath, &mut errors);
	authority::run(&mut xpath, authority, &mut errors);
	match authority {
		RegulatoryAuthority::Fda => fda::run(&mut xpath, &mut errors),
		RegulatoryAuthority::Mfds => mfds::run(&mut xpath, &mut errors),
		RegulatoryAuthority::Ich => {}
	}
	Ok(errors)
}

fn matches(xpath: &mut Context, expression: &str) -> bool {
	!xpath
		.evaluate(expression)
		.expect("static export-rule XPath must compile")
		.get_nodes_as_vec()
		.is_empty()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parses_once_and_reports_official_code() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><name codeSystem="2.16.840.1.113883.3.989.2.1.1.1" code="2"/></MCCI_IN200100UV01>"#;
		let errors = validate_export_rules(xml, RegulatoryAuthority::Ich).unwrap();
		assert!(errors
			.iter()
			.any(|error| error.code.as_deref() == Some("N.1.1")));
		assert!(errors.iter().all(|error| error
			.code
			.as_deref()
			.map_or(true, |code| !code.contains(".XML."))));
	}
}
