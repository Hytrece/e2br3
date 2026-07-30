use super::*;
use lib_core::e2b::null_flavor::E2bNullFlavorValue;

pub(crate) async fn export_patch(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	raw_xml: &[u8],
) -> Result<String> {
	let tests = fetch_test_results(mm, case_id).await?;
	patch_f_test_results(raw_xml, &tests)
}

async fn fetch_test_results(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<TestResult>> {
	mm.dbx()
		.fetch_all(
			sqlx::query_as::<_, TestResult>(
				"SELECT * FROM test_results WHERE case_id = $1 AND deleted = false ORDER BY sequence_number",
			)
			.bind(case_id),
		)
		.await
		.map_err(model::Error::from)
		.map_err(Error::from)
}

use sqlx::types::time::Date;

pub(crate) fn test_result_fragment(result: &TestResult) -> Result<String> {
	let mut out = String::new();
	out.push_str("<subjectOf2 typeCode=\"SBJ\"><organizer classCode=\"CATEGORY\" moodCode=\"EVN\">");
	out.push_str(
		"<code code=\"3\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.20\"/>",
	);
	out.push_str("<component typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\">");
	out.push_str("<code");
	if let Some(code) = write_f_r_2_2b(result) {
		out.push_str(" code=\"");
		out.push_str(&xml_escape(code));
		out.push_str("\"");
	}
	if let Some(version) = write_f_r_2_2a(result) {
		out.push_str(" codeSystemVersion=\"");
		out.push_str(&xml_escape(version));
		out.push_str("\"");
	}
	out.push_str(" displayName=\"");
	out.push_str(&write_f_r_2_1(result));
	out.push_str("\">");
	out.push_str("<originalText>");
	out.push_str(&write_f_r_2_1(result));
	out.push_str("</originalText>");
	out.push_str("</code>");
	out.push_str(&write_f_r_1(result)?);
	if let Some(code) = write_f_r_3_1(result) {
		out.push_str("<interpretationCode code=\"");
		out.push_str(&xml_escape(code));
		out.push_str("\"/>");
	}
	if result.test_result_value.is_some()
		|| result.test_result_null_flavor.is_some()
		|| result.result_unstructured.is_some()
	{
		out.push_str("<value");
		if let Some(null_flavor) = result.test_result_null_flavor.as_deref() {
			out.push_str(" nullFlavor=\"");
			out.push_str(&xml_escape(null_flavor));
			out.push_str("\"");
		} else if let Some(val) = write_f_r_3_2(result) {
			out.push_str(" value=\"");
			out.push_str(&xml_escape(val));
			out.push_str("\"");
		}
		if let Some(unit) = write_f_r_3_3(result) {
			out.push_str(" unit=\"");
			out.push_str(&xml_escape(unit));
			out.push_str("\"");
		}
		out.push_str(">");
		if let Some(text) = write_f_r_3_4(result) {
			out.push_str(&xml_escape(text));
		}
		out.push_str("</value>");
	}
	if result.normal_low_value.is_some() || result.normal_high_value.is_some() {
		out.push_str("<referenceRange>");
		if let Some(low) = write_f_r_4(result) {
			out.push_str(
				"<observationRange><interpretationCode code=\"L\"/><value value=\"",
			);
			out.push_str(&xml_escape(low));
			out.push_str("\"/></observationRange>");
		}
		if let Some(high) = write_f_r_5(result) {
			out.push_str(
				"<observationRange><interpretationCode code=\"H\"/><value value=\"",
			);
			out.push_str(&xml_escape(high));
			out.push_str("\"/></observationRange>");
		}
		out.push_str("</referenceRange>");
	}
	if let Some(comments) = write_f_r_6(result) {
		out.push_str("<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"10\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value>");
		out.push_str(&xml_escape(comments));
		out.push_str("</value></observation></outboundRelationship2>");
	}
	if let Some(value) = write_f_r_7(result) {
		let val = if value { "true" } else { "false" };
		out.push_str("<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"11\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"");
		out.push_str(val);
		out.push_str("\"/></observation></outboundRelationship2>");
	}
	out.push_str("</observation></component></organizer></subjectOf2>");
	Ok(out)
}

/// e2b:F.r.1
fn write_f_r_1(result: &TestResult) -> Result<String> {
	let field = E2bNullFlavorValue::from_parts(
		result.test_date,
		result.test_date_null_flavor.as_deref(),
	)
	.map_err(|err| Error::InvalidXml {
		message: format!("Invalid F.r.1 test date nullFlavor: {err}"),
		line: None,
		column: None,
	})?;

	match field {
		Some(E2bNullFlavorValue::Value { value }) => {
			Ok(format!("<effectiveTime value=\"{}\"/>", fmt_date(value)))
		}
		Some(E2bNullFlavorValue::NullFlavor { null_flavor }) => Ok(format!(
			"<effectiveTime nullFlavor=\"{}\"/>",
			xml_escape(null_flavor.as_str())
		)),
		None => Ok(String::new()),
	}
}

/// e2b:F.r.2.1
fn write_f_r_2_1(value: &TestResult) -> String {
	xml_escape(&value.test_name)
}

/// e2b:F.r.2.2a
fn write_f_r_2_2a(value: &TestResult) -> Option<&str> {
	value.test_meddra_version.as_deref()
}

/// e2b:F.r.2.2b
fn write_f_r_2_2b(value: &TestResult) -> Option<&str> {
	value.test_meddra_code.as_deref()
}

/// e2b:F.r.3.1
fn write_f_r_3_1(value: &TestResult) -> Option<&str> {
	value.test_result_code.as_deref()
}

/// e2b:F.r.3.2
fn write_f_r_3_2(value: &TestResult) -> Option<&str> {
	value.test_result_value.as_deref()
}

/// e2b:F.r.3.3
fn write_f_r_3_3(value: &TestResult) -> Option<&str> {
	value.test_result_unit.as_deref()
}

/// e2b:F.r.3.4
fn write_f_r_3_4(value: &TestResult) -> Option<&str> {
	value.result_unstructured.as_deref()
}

/// e2b:F.r.4
fn write_f_r_4(value: &TestResult) -> Option<&str> {
	value.normal_low_value.as_deref()
}

/// e2b:F.r.5
fn write_f_r_5(value: &TestResult) -> Option<&str> {
	value.normal_high_value.as_deref()
}

/// e2b:F.r.6
fn write_f_r_6(value: &TestResult) -> Option<&str> {
	value.comments.as_deref()
}

/// e2b:F.r.7
fn write_f_r_7(value: &TestResult) -> Option<bool> {
	value.more_info_available
}

fn fmt_date(date: Date) -> String {
	format!(
		"{:04}{:02}{:02}",
		date.year(),
		u8::from(date.month()),
		date.day()
	)
}

fn xml_escape(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

#[cfg(test)]
mod registry_coverage_tests {
	use std::collections::BTreeSet;

	#[test]
	fn section_f_writers_cover_registry_fields() {
		let registry: serde_json::Value = serde_json::from_str(include_str!(
			"../../../../../../registry/sections/f-test.json"
		))
		.expect("section F registry");
		let expected = registry
			.as_array()
			.expect("registry array")
			.iter()
			.filter(|entry| entry["local_only"] != true)
			.filter_map(|entry| entry["e2br3_code"].as_str())
			.collect::<BTreeSet<_>>();
		let implemented = include_str!("f.rs")
			.lines()
			.filter_map(|line| line.trim().strip_prefix("/// e2b:"))
			.collect::<BTreeSet<_>>();

		assert_eq!(implemented, expected);
	}
}

#[cfg(test)]
mod tests {
	use super::test_result_fragment;
	use lib_core::model::test_result::TestResult;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Uuid;

	#[test]
	fn exports_test_result_companion_as_xml_null_flavor_attribute() {
		let now = OffsetDateTime::now_utc();
		let mut result = TestResult {
			id: Uuid::new_v4(),
			case_id: Uuid::new_v4(),
			sequence_number: 1,
			test_date: None,
			test_date_null_flavor: None,
			test_name: "Result".to_string(),
			test_meddra_version: None,
			test_meddra_code: None,
			test_result_code: None,
			test_result_value: None,
			test_result_null_flavor: Some("NINF".to_string()),
			test_result_unit: None,
			result_unstructured: None,
			normal_low_value: None,
			normal_high_value: None,
			comments: None,
			more_info_available: None,
			deleted: false,
			created_at: now,
			updated_at: now,
			created_by: Uuid::new_v4(),
			updated_by: None,
		};
		let xml = test_result_fragment(&result).expect("export");
		assert!(xml.contains("<value nullFlavor=\"NINF\"></value>"));

		result.test_result_null_flavor = None;
		result.test_date_null_flavor = Some("ASKU".to_string());
		let xml = test_result_fragment(&result).expect("export date NullFlavor");
		assert!(xml.contains("<effectiveTime nullFlavor=\"ASKU\"/>"));
	}
}
