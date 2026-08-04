// Section C importer (Safety Report Identification) - FDA mapping.

use crate::error::Error;
use crate::import_constraint;
use crate::mapping::fda::c_safety_report::CSafetyReportPaths;
use crate::Result;
use libxml::parser::Parser;
use libxml::xpath::Context;
use sqlx::types::time::Date;
use time::Month;

mod helpers;
mod runtime;
pub use runtime::apply_c_safety_report_import_settings;
pub(crate) use runtime::import_section_c;

#[derive(Debug)]
pub struct CSafetyReportImport {
	pub transmission_date: String,
	pub report_type: String,
	pub date_first_received_from_source: Date,
	pub date_of_most_recent_information: Date,
	pub fulfil_expedited_criteria: bool,
	pub additional_documents_available: Option<bool>,
	pub local_criteria_report_type: Option<String>,
	pub combination_product_report_indicator: Option<String>,
	pub combination_product_report_indicator_null_flavor: Option<String>,
	pub worldwide_unique_id: Option<String>,
	pub first_sender_type: Option<String>,
	pub other_case_identifiers_exist: Option<bool>,
	pub other_case_identifiers_exist_null_flavor: Option<String>,
	pub nullification_code: Option<String>,
	pub nullification_reason: Option<String>,
}

/// Parse Section C values using FDA/ICH mapping paths.
pub fn parse_c_safety_report(xml: &[u8]) -> Result<Option<CSafetyReportImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
		.parse_string(xml_str)
		.map_err(|err| Error::InvalidXml {
			message: format!("XML parse error: {err}"),
			line: None,
			column: None,
		})?;
	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");

	let transmission_date = read_c_1_2(&mut xpath)?;

	let report_type = read_c_1_3(&mut xpath)?;

	let date_first_received_from_source = read_c_1_4(&mut xpath)?;
	let date_of_most_recent_information = read_c_1_5(&mut xpath)?;
	let additional_documents_available = read_c_1_6_1(&mut xpath)?;
	let fulfil_expedited_criteria = read_c_1_7(&mut xpath)?;
	let local_criteria_report_type = read_fda_c_1_7_1(&mut xpath)?;
	let (
		combination_product_report_indicator,
		combination_product_report_indicator_null_flavor,
	) = read_fda_c_1_12(&mut xpath)?;
	let worldwide_unique_id = read_c_1_8_1(&mut xpath)?;
	let first_sender_type = read_c_1_8_2(&mut xpath)?;
	let (other_case_identifiers_exist, other_case_identifiers_exist_null_flavor) =
		read_c_1_9_1(&mut xpath)?;
	let nullification_code = read_c_1_11_1(&mut xpath)?;
	let nullification_reason = read_c_1_11_2(&mut xpath)?;

	Ok(Some(CSafetyReportImport {
		transmission_date,
		report_type,
		date_first_received_from_source,
		date_of_most_recent_information,
		fulfil_expedited_criteria,
		additional_documents_available,
		local_criteria_report_type,
		combination_product_report_indicator,
		combination_product_report_indicator_null_flavor,
		worldwide_unique_id,
		first_sender_type,
		other_case_identifiers_exist,
		other_case_identifiers_exist_null_flavor,
		nullification_code,
		nullification_reason,
	}))
}

/// e2b:C.1.2
fn read_c_1_2(xpath: &mut Context) -> Result<String> {
	let raw = first_value_root(xpath, CSafetyReportPaths::DATE_OF_CREATION)
		.ok_or_else(|| Error::InvalidXml {
			message: "ICH.C.1.2.REQUIRED: transmission date missing".to_string(),
			line: None,
			column: None,
		})?;
	let value = normalize_datetime(&raw).ok_or_else(|| Error::InvalidXml {
		message: "ICH.C.1.2: invalid transmission date".to_string(),
		line: None,
		column: None,
	})?;
	import_constraint::string(
		"transmissionDate",
		Some(&value),
		None,
		input_contracts::generated::c::c_1_2,
	)?;
	Ok(value)
}

/// e2b:C.1.3
fn read_c_1_3(xpath: &mut Context) -> Result<String> {
	let value = first_value_root(xpath, CSafetyReportPaths::TYPE_OF_REPORT_CODE)
		.ok_or_else(|| Error::InvalidXml {
			message: "ICH.C.1.3.REQUIRED: type of report missing".to_string(),
			line: None,
			column: None,
		})?;
	import_constraint::string(
		"reportType",
		Some(&value),
		None,
		input_contracts::generated::c::c_1_3,
	)?;
	Ok(value)
}

/// e2b:C.1.4
fn read_c_1_4(xpath: &mut Context) -> Result<Date> {
	let value = first_value_root(xpath, CSafetyReportPaths::DATE_FIRST_RECEIVED)
		.ok_or_else(|| Error::InvalidXml {
			message: "ICH.C.1.4.REQUIRED: first received date missing".to_string(),
			line: None,
			column: None,
		})?;
	import_constraint::string(
		"dateFirstReceivedFromSource",
		Some(&value),
		None,
		input_contracts::generated::c::c_1_4,
	)?;
	parse_date(value).ok_or_else(|| Error::InvalidXml {
		message: "ICH.C.1.4: invalid first received date".to_string(),
		line: None,
		column: None,
	})
}

/// e2b:C.1.5
fn read_c_1_5(xpath: &mut Context) -> Result<Date> {
	let value = first_value_root(xpath, CSafetyReportPaths::DATE_MOST_RECENT)
		.ok_or_else(|| Error::InvalidXml {
			message: "ICH.C.1.5.REQUIRED: most recent information date missing"
				.to_string(),
			line: None,
			column: None,
		})?;
	import_constraint::string(
		"dateOfMostRecentInformation",
		Some(&value),
		None,
		input_contracts::generated::c::c_1_5,
	)?;
	parse_date(value).ok_or_else(|| Error::InvalidXml {
		message: "ICH.C.1.5: invalid most recent information date".to_string(),
		line: None,
		column: None,
	})
}

/// e2b:C.1.6.1
fn read_c_1_6_1(xpath: &mut Context) -> Result<Option<bool>> {
	let value = parse_bool_value(first_value_root(
		xpath,
		CSafetyReportPaths::ADDITIONAL_DOCUMENTS_AVAILABLE,
	));
	import_constraint::boolean(
		"additionalDocumentsAvailable",
		value,
		None,
		input_contracts::generated::c::c_1_6_1,
	)?;
	Ok(value)
}

/// e2b:C.1.7
fn read_c_1_7(xpath: &mut Context) -> Result<bool> {
	let value = parse_bool_value(first_value_root(
		xpath,
		CSafetyReportPaths::FULFIL_EXPEDITED,
	));
	let null_flavor =
		first_value_root(xpath, CSafetyReportPaths::FULFIL_EXPEDITED_NULL_FLAVOR);
	if value.is_none() && null_flavor.is_none() {
		return Err(Error::InvalidXml {
			message: "ICH.C.1.7.REQUIRED: expedited criteria missing".to_string(),
			line: None,
			column: None,
		});
	}
	import_constraint::boolean(
		"fulfilExpeditedCriteria",
		value,
		null_flavor.as_deref(),
		input_contracts::generated::c::c_1_7,
	)?;
	value.ok_or_else(|| Error::InvalidXml {
		message: "ICH.C.1.7: NI requires verified E2B(R2)-origin provenance"
			.to_string(),
		line: None,
		column: None,
	})
}

#[cfg(test)]
mod c_1_7_tests {
	use super::*;

	#[test]
	fn rejects_ni_without_verified_r2_provenance() {
		let xml = r#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><component><observationEvent><code code="23" codeSystem="2.16.840.1.113883.3.989.2.1.1.19"/><value nullFlavor="NI"/></observationEvent></component></MCCI_IN200100UV01>"#;
		let parser = Parser::default();
		let doc = parser.parse_string(xml).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		assert!(read_c_1_7(&mut xpath).is_err());
	}
}

/// e2b:FDA.C.1.7.1
fn read_fda_c_1_7_1(xpath: &mut Context) -> Result<Option<String>> {
	let value = first_value_root(
		xpath,
		CSafetyReportPaths::FDA_LOCAL_CRITERIA_REPORT_TYPE_CODE,
	);
	import_constraint::string(
		"localCriteriaReportType",
		value.as_deref(),
		None,
		input_contracts::generated::c::fda_c_1_7_1,
	)?;
	Ok(value)
}

/// e2b:FDA.C.1.12
fn read_fda_c_1_12(xpath: &mut Context) -> Result<(Option<String>, Option<String>)> {
	let raw = first_value_root(
		xpath,
		CSafetyReportPaths::FDA_COMBINATION_PRODUCT_INDICATOR_VALUE,
	);
	let value = normalize_fda_combination_product_indicator(raw.clone());
	let null_flavor = first_value_root(
		xpath,
		CSafetyReportPaths::FDA_COMBINATION_PRODUCT_INDICATOR_NULL_FLAVOR,
	);
	if raw.is_some() && value.is_none() {
		return Err(Error::InvalidXml {
			message: "FDA.C.1.12: invalid boolean value".to_string(),
			line: None,
			column: None,
		});
	}
	if value.is_some() && null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message: "FDA.C.1.12: value and nullFlavor cannot both be set"
				.to_string(),
			line: None,
			column: None,
		});
	}
	import_constraint::boolean(
		"combinationProductReportIndicator",
		value.as_deref().and_then(|value| value.parse().ok()),
		null_flavor.as_deref(),
		input_contracts::generated::c::fda_c_1_12,
	)?;
	Ok((value, null_flavor))
}

/// e2b:C.1.8.1
fn read_c_1_8_1(xpath: &mut Context) -> Result<Option<String>> {
	let value = first_value_root(xpath, CSafetyReportPaths::WORLDWIDE_UNIQUE_ID_EXT);
	import_constraint::string(
		"worldwideUniqueId",
		value.as_deref(),
		None,
		input_contracts::generated::c::c_1_8_1,
	)?;
	Ok(value)
}

/// e2b:C.1.8.2
fn read_c_1_8_2(xpath: &mut Context) -> Result<Option<String>> {
	let value = first_value_root(xpath, CSafetyReportPaths::FIRST_SENDER_TYPE);
	import_constraint::string(
		"firstSenderType",
		value.as_deref(),
		None,
		input_contracts::generated::c::c_1_8_2,
	)?;
	Ok(value)
}

/// e2b:C.1.9.1
fn read_c_1_9_1(xpath: &mut Context) -> Result<(Option<bool>, Option<String>)> {
	let raw =
		first_value_root(xpath, CSafetyReportPaths::OTHER_CASE_IDENTIFIERS_EXIST);
	let value = parse_bool_value(raw.clone());
	let null_flavor = first_value_root(
		xpath,
		CSafetyReportPaths::OTHER_CASE_IDENTIFIERS_EXIST_NULL_FLAVOR,
	);
	if raw.is_some() && value.is_none() {
		return Err(Error::InvalidXml {
			message: "ICH.C.1.9.1: invalid boolean value".to_string(),
			line: None,
			column: None,
		});
	}
	if value.is_some() && null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message: "ICH.C.1.9.1: value and nullFlavor cannot both be set"
				.to_string(),
			line: None,
			column: None,
		});
	}
	import_constraint::boolean(
		"otherCaseIdentifiersExist",
		value,
		null_flavor.as_deref(),
		input_contracts::generated::c::c_1_9_1,
	)?;
	Ok((value, null_flavor))
}

/// e2b:C.1.11.1
fn read_c_1_11_1(xpath: &mut Context) -> Result<Option<String>> {
	let value = first_value_root(xpath, CSafetyReportPaths::NULLIFICATION_CODE);
	import_constraint::string(
		"nullificationAmendmentCode",
		value.as_deref(),
		None,
		input_contracts::generated::c::c_1_11_1,
	)?;
	Ok(value)
}

/// e2b:C.1.11.2
fn read_c_1_11_2(xpath: &mut Context) -> Result<Option<String>> {
	let value = first_text_root(xpath, CSafetyReportPaths::NULLIFICATION_REASON);
	import_constraint::string(
		"nullificationReason",
		value.as_deref(),
		None,
		input_contracts::generated::c::c_1_11_2,
	)?;
	Ok(value)
}

fn first_value_root(xpath: &mut Context, path: &str) -> Option<String> {
	match xpath.findvalue(path, None) {
		Ok(value) if !value.trim().is_empty() => Some(value),
		_ => None,
	}
}

fn first_text_root(xpath: &mut Context, path: &str) -> Option<String> {
	match xpath.findvalue(path, None) {
		Ok(value) if !value.trim().is_empty() => Some(value),
		_ => None,
	}
}

fn normalize_fda_combination_product_indicator(
	value: Option<String>,
) -> Option<String> {
	parse_bool_value(value).map(|value| value.to_string())
}

fn parse_bool_value(value: Option<String>) -> Option<bool> {
	value.and_then(|raw| match raw.trim().to_ascii_lowercase().as_str() {
		"true" | "1" | "yes" => Some(true),
		"false" | "0" | "no" => Some(false),
		_ => None,
	})
}

fn parse_date(value: String) -> Option<Date> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	if digits.len() < 8 {
		return None;
	}
	let y: i32 = digits[0..4].parse().ok()?;
	let m: u8 = digits[4..6].parse().ok()?;
	let d: u8 = digits[6..8].parse().ok()?;
	let month = Month::try_from(m).ok()?;
	Date::from_calendar_date(y, month, d).ok()
}

fn normalize_datetime(value: &str) -> Option<String> {
	let digits: String = value.chars().filter(|c| c.is_ascii_digit()).collect();
	if digits.len() < 8 {
		return None;
	}
	let date = parse_date(digits.clone())?;
	if digits.len() >= 14 {
		Some(digits[..14].to_string())
	} else {
		Some(format_datetime(date))
	}
}

fn format_datetime(date: Date) -> String {
	format!(
		"{:04}{:02}{:02}000000",
		date.year(),
		u8::from(date.month()),
		date.day()
	)
}

#[cfg(test)]
mod tests {
	use super::normalize_fda_combination_product_indicator;

	#[test]
	fn fda_combination_product_import_normalizes_to_boolean_strings() {
		assert_eq!(
			normalize_fda_combination_product_indicator(Some("true".to_string())),
			Some("true".to_string())
		);
		assert_eq!(
			normalize_fda_combination_product_indicator(Some("1".to_string())),
			Some("true".to_string())
		);
		assert_eq!(
			normalize_fda_combination_product_indicator(Some("0".to_string())),
			Some("false".to_string())
		);
		assert_eq!(
			normalize_fda_combination_product_indicator(Some("unknown".to_string())),
			None
		);
	}
}
