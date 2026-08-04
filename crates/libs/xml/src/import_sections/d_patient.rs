// Section D importer (Patient) - FDA mapping.

use crate::error::Error;
use crate::import_constraint;
use crate::mapping::fda::d_patient::DPatientPaths;
use crate::Result;
use lib_core::regulatory::{
	infer_regulatory_authority_from_receivers, RegulatoryAuthority,
};
use libxml::parser::Parser;
use libxml::xpath::Context;
use rust_decimal::Decimal;
use sqlx::types::time::Date;
use time::Month;

pub(crate) mod helpers;
mod runtime;
pub(crate) use runtime::import_section_d;

#[derive(Debug)]
pub struct DPatientImport {
	pub patient_initials: Option<String>,
	pub patient_initials_null_flavor: Option<String>,
	pub birth_date: Option<Date>,
	pub birth_date_null_flavor: Option<String>,
	pub sex: Option<String>,
	pub sex_null_flavor: Option<String>,
	pub age_at_time_of_onset: Option<Decimal>,
	pub age_unit: Option<String>,
	pub gestation_period: Option<Decimal>,
	pub gestation_period_unit: Option<String>,
	pub age_group: Option<String>,
	pub weight_kg: Option<Decimal>,
	pub height_cm: Option<Decimal>,
	pub race_codes: Vec<String>,
	pub race_code_null_flavor: Option<String>,
	pub ethnicity_code: Option<String>,
	pub ethnicity_code_null_flavor: Option<String>,
	pub last_menstrual_period_date: Option<Date>,
	pub last_menstrual_period_date_null_flavor: Option<String>,
	pub medical_history_text: Option<String>,
	pub medical_history_text_null_flavor: Option<String>,
	pub concomitant_therapy: Option<bool>,
}

#[derive(Debug)]
pub struct DParentImport {
	pub parent_identification: Option<String>,
	pub parent_identification_null_flavor: Option<String>,
	pub sex: Option<String>,
	pub sex_null_flavor: Option<String>,
}

/// Parse the canonical parent value/NullFlavor pairs used by Section D.
pub fn parse_d_parent(xml: &[u8]) -> Result<Option<DParentImport>> {
	Ok(
		helpers::parse_parent_information(xml)?.map(|parent| DParentImport {
			parent_identification: parent.parent_identification,
			parent_identification_null_flavor: parent
				.parent_identification_null_flavor,
			sex: parent.sex,
			sex_null_flavor: parent.sex_null_flavor,
		}),
	)
}

/// Parse Section D values using FDA/ICH mapping paths.
pub fn parse_d_patient(xml: &[u8]) -> Result<Option<DPatientImport>> {
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

	let header = crate::import_sections::shared::extract_message_header(xml)?;
	let authority = infer_regulatory_authority_from_receivers(
		header.batch_receiver.as_deref(),
		header.message_receiver.as_deref(),
	);
	let (patient_initials, patient_initials_null_flavor) =
		read_d_1(&mut xpath, authority)?;
	let (birth_date, birth_date_null_flavor) = read_d_2_1(&mut xpath)?;
	let age_at_time_of_onset = read_d_2_2a(&mut xpath)?;
	let age_unit = read_d_2_2b(&mut xpath)?;
	let gestation_period = read_d_2_2_1a(&mut xpath)?;
	let gestation_period_unit = read_d_2_2_1b(&mut xpath)?;
	let age_group = read_d_2_3(&mut xpath)?;
	let weight_kg = read_d_3(&mut xpath)?;
	let height_cm = read_d_4(&mut xpath)?;
	let (sex, sex_null_flavor) = read_d_5(&mut xpath)?;
	let (last_menstrual_period_date, last_menstrual_period_date_null_flavor) =
		read_d_6(&mut xpath)?;
	let (medical_history_text, medical_history_text_null_flavor) =
		read_d_7_2(&mut xpath)?;
	let concomitant_therapy = read_d_7_3(&mut xpath)?;
	let (race_codes, race_code_null_flavor) = read_fda_d_11_r_1(&mut xpath)?;
	let (ethnicity_code, ethnicity_code_null_flavor) = read_fda_d_12(&mut xpath)?;

	if patient_initials.is_none()
		&& sex.is_none()
		&& age_at_time_of_onset.is_none()
		&& gestation_period.is_none()
		&& weight_kg.is_none()
		&& height_cm.is_none()
		&& patient_initials_null_flavor.is_none()
		&& birth_date_null_flavor.is_none()
		&& sex_null_flavor.is_none()
		&& last_menstrual_period_date_null_flavor.is_none()
		&& race_codes.is_empty()
		&& race_code_null_flavor.is_none()
		&& ethnicity_code_null_flavor.is_none()
	{
		return Ok(None);
	}

	Ok(Some(DPatientImport {
		patient_initials,
		patient_initials_null_flavor,
		birth_date,
		birth_date_null_flavor,
		sex,
		sex_null_flavor,
		age_at_time_of_onset,
		age_unit,
		gestation_period,
		gestation_period_unit,
		age_group,
		weight_kg,
		height_cm,
		race_codes,
		race_code_null_flavor,
		ethnicity_code,
		ethnicity_code_null_flavor,
		last_menstrual_period_date,
		last_menstrual_period_date_null_flavor,
		medical_history_text,
		medical_history_text_null_flavor,
		concomitant_therapy,
	}))
}

/// e2b:D.1
fn read_d_1(
	xpath: &mut Context,
	authority: RegulatoryAuthority,
) -> Result<(Option<String>, Option<String>)> {
	let contract = if authority == RegulatoryAuthority::Fda {
		input_contracts::generated::d::fda_d_1
	} else {
		input_contracts::generated::d::d_1
	};
	string_pair(
		first_text_root(xpath, DPatientPaths::PATIENT_NAME),
		first_value_root(xpath, DPatientPaths::PATIENT_NAME_NULL_FLAVOR),
		"patientInitials",
		"patientInitialsNullFlavor",
		contract,
	)
}

/// e2b:D.2.1
fn read_d_2_1(xpath: &mut Context) -> Result<(Option<Date>, Option<String>)> {
	date_pair(
		first_value_root(xpath, DPatientPaths::BIRTH_DATE),
		first_value_root(xpath, DPatientPaths::BIRTH_DATE_NULL_FLAVOR),
		"patientBirthDate",
		"patientBirthDateNullFlavor",
		input_contracts::generated::d::d_2_1,
	)
}

/// e2b:D.2.2a
fn read_d_2_2a(xpath: &mut Context) -> Result<Option<Decimal>> {
	if first_value_root(xpath, DPatientPaths::AGE_NULL_FLAVOR).is_some() {
		return Err(Error::InvalidXml {
			message: "D.2.2a does not permit nullFlavor".to_string(),
			line: None,
			column: None,
		});
	}
	let raw = first_value_root(xpath, DPatientPaths::AGE_VALUE);
	import_constraint::number_string(
		"patientAge.value",
		raw.as_deref(),
		input_contracts::generated::d::d_2_2a,
	)?;
	Ok(raw.and_then(|value| value.parse().ok()))
}

/// e2b:D.2.2b
fn read_d_2_2b(xpath: &mut Context) -> Result<Option<String>> {
	input_string(
		first_value_root(xpath, DPatientPaths::AGE_UNIT),
		"patientAge.unit",
		input_contracts::generated::d::d_2_2b,
	)
}

/// e2b:D.2.2.1a
fn read_d_2_2_1a(xpath: &mut Context) -> Result<Option<Decimal>> {
	input_decimal(
		first_value_root(xpath, DPatientPaths::GESTATION_VALUE),
		"gestationPeriod.value",
		input_contracts::generated::d::d_2_2_1a,
	)
}

/// e2b:D.2.2.1b
fn read_d_2_2_1b(xpath: &mut Context) -> Result<Option<String>> {
	input_string(
		first_value_root(xpath, DPatientPaths::GESTATION_UNIT),
		"gestationPeriod.unit",
		input_contracts::generated::d::d_2_2_1b,
	)
}

/// e2b:D.2.3
fn read_d_2_3(xpath: &mut Context) -> Result<Option<String>> {
	input_string(
		first_value_root(xpath, DPatientPaths::AGE_GROUP_CODE),
		"patientAgeGroup",
		input_contracts::generated::d::d_2_3,
	)
}

/// e2b:D.3
fn read_d_3(xpath: &mut Context) -> Result<Option<Decimal>> {
	if first_value_root(xpath, DPatientPaths::WEIGHT_NULL_FLAVOR).is_some() {
		return Err(Error::InvalidXml {
			message: "D.3 does not permit nullFlavor".to_string(),
			line: None,
			column: None,
		});
	}
	input_decimal(
		first_value_root(xpath, DPatientPaths::WEIGHT_VALUE),
		"patientWeight.value",
		input_contracts::generated::d::d_3,
	)
}

/// e2b:D.4
fn read_d_4(xpath: &mut Context) -> Result<Option<Decimal>> {
	if first_value_root(xpath, DPatientPaths::HEIGHT_NULL_FLAVOR).is_some() {
		return Err(Error::InvalidXml {
			message: "D.4 does not permit nullFlavor".to_string(),
			line: None,
			column: None,
		});
	}
	input_decimal(
		first_value_root(xpath, DPatientPaths::HEIGHT_VALUE),
		"patientHeight.value",
		input_contracts::generated::d::d_4,
	)
}

/// e2b:D.5
fn read_d_5(xpath: &mut Context) -> Result<(Option<String>, Option<String>)> {
	string_pair(
		first_value_root(xpath, DPatientPaths::SEX_CODE),
		first_value_root(xpath, DPatientPaths::SEX_NULL_FLAVOR),
		"patientSex",
		"patientSexNullFlavor",
		input_contracts::generated::d::d_5,
	)
}

/// e2b:D.6
fn read_d_6(xpath: &mut Context) -> Result<(Option<Date>, Option<String>)> {
	date_pair(
		first_value_root(xpath, DPatientPaths::LMP_DATE),
		first_value_root(xpath, DPatientPaths::LMP_DATE_NULL_FLAVOR),
		"lastMenstrualPeriodDate",
		"lastMenstrualPeriodDateNullFlavor",
		input_contracts::generated::d::d_6,
	)
}

/// e2b:D.7.2
fn read_d_7_2(xpath: &mut Context) -> Result<(Option<String>, Option<String>)> {
	string_pair(
		first_text_root(xpath, DPatientPaths::MEDICAL_HISTORY_TEXT),
		first_value_root(xpath, DPatientPaths::MEDICAL_HISTORY_TEXT_NULL_FLAVOR),
		"medicalHistoryText",
		"medicalHistoryTextNullFlavor",
		input_contracts::generated::d::d_7_2,
	)
}

/// e2b:D.7.3
fn read_d_7_3(xpath: &mut Context) -> Result<Option<bool>> {
	let value = parse_bool_value(first_value_root(
		xpath,
		DPatientPaths::CONCOMITANT_THERAPY_VALUE,
	));
	import_constraint::boolean(
		"concomitantTherapies",
		value,
		None,
		input_contracts::generated::d::d_7_3,
	)?;
	Ok(value)
}

/// e2b:FDA.D.11.r.1
fn read_fda_d_11_r_1(xpath: &mut Context) -> Result<(Vec<String>, Option<String>)> {
	let race_codes = xpath
		.findvalues(DPatientPaths::RACE_CODE, None)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to read FDA.D.11.r.1 race codes".to_string(),
			line: None,
			column: None,
		})?
		.into_iter()
		.map(|value| value.trim().to_string())
		.filter(|value| !value.is_empty())
		.collect::<Vec<_>>();
	let null_flavor = first_value_root(xpath, DPatientPaths::RACE_CODE_NULL_FLAVOR);
	if !race_codes.is_empty() && null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message: "FDA.D.11.r.1 race codes and NullFlavor cannot both be set"
				.to_string(),
			line: None,
			column: None,
		});
	}
	for (index, value) in race_codes.iter().enumerate() {
		import_constraint::string(
			&format!("raceCodes.{index}"),
			Some(value),
			None,
			input_contracts::generated::d::fda_d_11_r_1,
		)?;
	}
	import_constraint::string(
		"raceCodeNullFlavor",
		None,
		null_flavor.as_deref(),
		input_contracts::generated::d::fda_d_11_r_1,
	)?;
	Ok((race_codes, null_flavor))
}

/// e2b:FDA.D.12
fn read_fda_d_12(xpath: &mut Context) -> Result<(Option<String>, Option<String>)> {
	string_pair(
		first_value_root(xpath, DPatientPaths::ETHNICITY_CODE),
		first_value_root(xpath, DPatientPaths::ETHNICITY_CODE_NULL_FLAVOR),
		"ethnicityCode",
		"ethnicityCodeNullFlavor",
		input_contracts::generated::d::fda_d_12,
	)
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

fn input_string(
	value: Option<String>,
	field: &str,
	check: impl for<'a> Fn(
		input_contracts::FieldInput<'a>,
	) -> Vec<input_contracts::InputIssue>,
) -> Result<Option<String>> {
	import_constraint::string(field, value.as_deref(), None, check)?;
	Ok(value)
}

fn input_decimal(
	value: Option<String>,
	field: &str,
	check: impl for<'a> Fn(
		input_contracts::FieldInput<'a>,
	) -> Vec<input_contracts::InputIssue>,
) -> Result<Option<Decimal>> {
	import_constraint::number_string(field, value.as_deref(), check)?;
	Ok(value.and_then(|value| value.parse().ok()))
}

fn string_pair(
	value: Option<String>,
	null_flavor: Option<String>,
	field: &str,
	_null_field: &str,
	check: impl for<'a> Fn(
		input_contracts::FieldInput<'a>,
	) -> Vec<input_contracts::InputIssue>,
) -> Result<(Option<String>, Option<String>)> {
	import_constraint::string(
		field,
		value.as_deref(),
		null_flavor.as_deref(),
		check,
	)?;
	Ok((value, null_flavor))
}

fn date_pair(
	value: Option<String>,
	null_flavor: Option<String>,
	field: &str,
	null_field: &str,
	check: impl for<'a> Fn(
		input_contracts::FieldInput<'a>,
	) -> Vec<input_contracts::InputIssue>,
) -> Result<(Option<Date>, Option<String>)> {
	let (value, null_flavor) =
		string_pair(value, null_flavor, field, null_field, check)?;
	Ok((value.and_then(parse_date), null_flavor))
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rejects_null_flavor_on_numeric_only_fields() {
		for code in ["3", "7", "17"] {
			let xml = format!(
				r#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <primaryRole><subjectOf2><observation>
    <code code="{code}" codeSystem="2.16.840.1.113883.3.989.2.1.1.19"/>
    <value xsi:type="PQ" nullFlavor="UNK"/>
  </observation></subjectOf2></primaryRole>
</MCCI_IN200100UV01>"#
			);

			assert!(parse_d_patient(xml.as_bytes()).is_err(), "code {code}");
		}
	}

	#[test]
	fn fda_patient_initials_accepts_regional_na() {
		let xml = r#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3">
  <receiver><device><id extension="ZZFDA"/></device></receiver>
  <primaryRole><player1><name nullFlavor="NA"/></player1></primaryRole>
</MCCI_IN200100UV01>"#;
		let patient = parse_d_patient(xml.as_bytes()).unwrap().unwrap();
		assert_eq!(patient.patient_initials_null_flavor.as_deref(), Some("NA"));
	}
}
