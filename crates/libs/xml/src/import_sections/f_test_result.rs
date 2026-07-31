// Section F importer (Tests and Procedures) - FDA mapping.

use crate::error::Error;
use crate::import_constraint;
use crate::mapping::fda::f_test_result::FTestResultPaths;
use crate::Result;
use lib_core::ctx::Ctx;
use lib_core::e2b::null_flavor::E2bNullFlavorValue;
use lib_core::model::store::set_full_context_dbx;
use lib_core::model::test_result::{TestResultBmc, TestResultForCreate};
use lib_core::model::ModelManager;
use libxml::parser::Parser;
use libxml::tree::Node;
use libxml::xpath::Context;
use sqlx::types::time::Date;
use time::Month;

#[derive(Debug)]
pub struct FTestResultImport {
	pub test_name: String,
	pub test_date: Option<Date>,
	pub test_date_null_flavor: Option<String>,
	pub test_meddra_version: Option<String>,
	pub test_meddra_code: Option<String>,
	pub test_result_code: Option<String>,
	pub test_result_value: Option<String>,
	pub test_result_null_flavor: Option<String>,
	pub test_result_unit: Option<String>,
	pub result_unstructured: Option<String>,
	pub normal_low_value: Option<String>,
	pub normal_high_value: Option<String>,
	pub comments: Option<String>,
	pub more_info_available: Option<bool>,
}

pub fn parse_f_test_results(xml: &[u8]) -> Result<Vec<FTestResultImport>> {
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

	let nodes =
		xpath
			.findnodes(FTestResultPaths::TEST_NODE, None)
			.map_err(|_| Error::InvalidXml {
				message: "Failed to query test results".to_string(),
				line: None,
				column: None,
			})?;

	let mut items = Vec::new();
	for (idx, node) in nodes.into_iter().enumerate() {
		let test_name = read_f_r_2_1(&mut xpath, &node, idx)?;
		let (test_date, test_date_null_flavor) = read_f_r_1(&mut xpath, &node)?;
		let (test_result_value, test_result_null_flavor) =
			read_f_r_3_2(&mut xpath, &node)?;

		items.push(FTestResultImport {
			test_name,
			test_date,
			test_date_null_flavor,
			test_meddra_version: read_f_r_2_2a(&mut xpath, &node)?,
			test_meddra_code: read_f_r_2_2b(&mut xpath, &node)?,
			test_result_code: read_f_r_3_1(&mut xpath, &node)?,
			test_result_value,
			test_result_null_flavor,
			test_result_unit: read_f_r_3_3(&mut xpath, &node)?,
			result_unstructured: read_f_r_3_4(&mut xpath, &node)?,
			normal_low_value: read_f_r_4(&mut xpath, &node)?,
			normal_high_value: read_f_r_5(&mut xpath, &node)?,
			comments: read_f_r_6(&mut xpath, &node)?,
			more_info_available: read_f_r_7(&mut xpath, &node)?,
		});
	}

	Ok(items)
}

pub(crate) async fn import_section_f(
	ctx: &Ctx,
	mm: &ModelManager,
	xml: &[u8],
	case_id: sqlx::types::Uuid,
) -> Result<()> {
	let tests = parse_f_test_results(xml)?;
	set_full_context_dbx(mm.dbx(), ctx.user_id(), ctx.organization_id(), ctx.role())
		.await
		.map_err(Error::Model)?;
	for (idx, entry) in tests.into_iter().enumerate() {
		TestResultBmc::create(
			ctx,
			mm,
			TestResultForCreate {
				case_id,
				sequence_number: (idx + 1) as i32,
				test_date: entry.test_date,
				test_date_null_flavor: entry.test_date_null_flavor,
				test_name: entry.test_name,
				test_meddra_version: entry.test_meddra_version,
				test_meddra_code: entry.test_meddra_code,
				test_result_code: entry.test_result_code,
				test_result_value: entry.test_result_value,
				test_result_null_flavor: entry.test_result_null_flavor,
				test_result_unit: entry.test_result_unit,
				result_unstructured: entry.result_unstructured,
				normal_low_value: entry.normal_low_value,
				normal_high_value: entry.normal_high_value,
				comments: entry.comments,
				more_info_available: entry.more_info_available,
			},
		)
		.await?;
	}
	Ok(())
}

/// e2b:F.r.1
fn read_f_r_1(
	xpath: &mut Context,
	node: &Node,
) -> Result<(Option<Date>, Option<String>)> {
	let value =
		first_attr(xpath, node, FTestResultPaths::TEST_DATE).and_then(parse_date);
	let null_flavor =
		first_attr(xpath, node, FTestResultPaths::TEST_DATE_NULL_FLAVOR);
	let raw = first_attr(xpath, node, FTestResultPaths::TEST_DATE);
	import_constraint::string(
		"LB",
		"testDate",
		raw.as_deref(),
		null_flavor.as_deref(),
	)?;
	import_constraint::string(
		"LB",
		"testDateNullFlavor",
		null_flavor.as_deref(),
		None,
	)?;
	let field = E2bNullFlavorValue::from_parts(value, null_flavor.as_deref())
		.map_err(|err| Error::InvalidXml {
			message: format!("Invalid F.r.1 test date nullFlavor: {err}"),
			line: None,
			column: None,
		})?;
	Ok(field
		.map(E2bNullFlavorValue::into_parts)
		.unwrap_or_default())
}

/// e2b:F.r.2.1
fn read_f_r_2_1(xpath: &mut Context, node: &Node, index: usize) -> Result<String> {
	let value = first_text(xpath, node, FTestResultPaths::TEST_NAME)
		.or_else(|| first_attr(xpath, node, FTestResultPaths::TEST_NAME_DISPLAY))
		.unwrap_or_else(|| {
			eprintln!(
				"[import_e2b_xml] test_results[{index}] missing F.r.2.1; importing empty test_name for downstream validation"
			);
			String::new()
		});
	import_constraint::string("LB", "testName", Some(&value), None)?;
	Ok(value)
}

/// e2b:F.r.2.2a
fn read_f_r_2_2a(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::TEST_MEDDRA_VERSION),
		"testMeddraVersion",
	)
}

/// e2b:F.r.2.2b
fn read_f_r_2_2b(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::TEST_MEDDRA_CODE),
		"testMeddraCode",
	)
}

/// e2b:F.r.3.1
fn read_f_r_3_1(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::RESULT_CODE),
		"testResultCode",
	)
}

/// e2b:F.r.3.2
fn read_f_r_3_2(
	xpath: &mut Context,
	node: &Node,
) -> Result<(Option<String>, Option<String>)> {
	let value =
		first_attr(xpath, node, FTestResultPaths::RESULT_VALUE).or_else(|| {
			first_attr(xpath, node, FTestResultPaths::RESULT_VALUE_FALLBACK)
		});
	let null_flavor = first_attr(xpath, node, FTestResultPaths::RESULT_NULL_FLAVOR);
	if value.is_some() && null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message: "F.r.3.2 value and nullFlavor cannot both be set".to_string(),
			line: None,
			column: None,
		});
	}
	import_constraint::string(
		"LB",
		"testResult",
		value.as_deref(),
		null_flavor.as_deref(),
	)?;
	import_constraint::string(
		"LB",
		"testResultNullFlavor",
		null_flavor.as_deref(),
		None,
	)?;
	Ok((value, null_flavor))
}

/// e2b:F.r.3.3
fn read_f_r_3_3(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::RESULT_UNIT).or_else(|| {
			first_attr(xpath, node, FTestResultPaths::RESULT_UNIT_FALLBACK)
		}),
		"testUnit",
	)
}

/// e2b:F.r.3.4
fn read_f_r_3_4(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_text(xpath, node, FTestResultPaths::RESULT_UNSTRUCTURED),
		"testResultUnstructured",
	)
}

/// e2b:F.r.4
fn read_f_r_4(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::NORMAL_LOW),
		"lowRange",
	)
}

/// e2b:F.r.5
fn read_f_r_5(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_attr(xpath, node, FTestResultPaths::NORMAL_HIGH),
		"highRange",
	)
}

/// e2b:F.r.6
fn read_f_r_6(xpath: &mut Context, node: &Node) -> Result<Option<String>> {
	portable_string(
		first_text(xpath, node, FTestResultPaths::COMMENTS),
		"comments",
	)
}

/// e2b:F.r.7
fn read_f_r_7(xpath: &mut Context, node: &Node) -> Result<Option<bool>> {
	let value =
		parse_bool_value(first_attr(xpath, node, FTestResultPaths::MORE_INFO));
	import_constraint::boolean("LB", "moreInformationAvailable", value, None)?;
	Ok(value)
}

fn first_attr(xpath: &mut Context, node: &Node, expr: &str) -> Option<String> {
	xpath
		.findvalues(expr, Some(node))
		.ok()?
		.into_iter()
		.find(|v| !v.trim().is_empty())
}

fn first_text(xpath: &mut Context, node: &Node, expr: &str) -> Option<String> {
	let nodes = xpath.findnodes(expr, Some(node)).ok()?;
	for n in nodes {
		let content = n.get_content();
		if !content.trim().is_empty() {
			return Some(content);
		}
	}
	None
}

fn portable_string(value: Option<String>, field: &str) -> Result<Option<String>> {
	import_constraint::string("LB", field, value.as_deref(), None)?;
	Ok(value)
}

fn parse_bool_value(value: Option<String>) -> Option<bool> {
	let val = value?;
	match val.to_ascii_lowercase().as_str() {
		"true" | "1" => Some(true),
		"false" | "0" => Some(false),
		_ => None,
	}
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

#[cfg(test)]
mod tests {
	use super::parse_f_test_results;

	#[test]
	fn imports_test_result_null_flavor_into_companion() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><organizer><code code="3" codeSystem="2.16.840.1.113883.3.989.2.1.1.20"/><component><observation><code><originalText>Result</originalText></code><value nullFlavor="NINF"/></observation></component></organizer></MCCI_IN200100UV01>"#;
		let results = parse_f_test_results(xml).expect("parse");
		assert_eq!(results[0].test_result_value, None);
		assert_eq!(results[0].test_result_null_flavor.as_deref(), Some("NINF"));
	}
}
