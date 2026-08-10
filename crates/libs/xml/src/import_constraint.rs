use crate::{Error, Result};
use input_contracts::{FieldInput, InputIssue};

pub(crate) fn string<F>(
	field: &str,
	value: Option<&str>,
	null_flavor: Option<&str>,
	_check_field: F,
) -> Result<()>
where
	F: for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
{
	reject_value_and_null_flavor(field, value.is_some(), null_flavor)?;
	Ok(())
}

pub(crate) fn boolean<F>(
	field: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
	_check_field: F,
) -> Result<()>
where
	F: for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
{
	reject_value_and_null_flavor(field, value.is_some(), null_flavor)?;
	Ok(())
}

fn reject_value_and_null_flavor(
	field: &str,
	has_value: bool,
	null_flavor: Option<&str>,
) -> Result<()> {
	if has_value && null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message: format!("{field}: value and nullFlavor cannot both be present"),
			line: None,
			column: None,
		});
	}
	Ok(())
}

pub(crate) fn number_string<F>(
	field: &str,
	value: Option<&str>,
	check_field: F,
) -> Result<()>
where
	F: for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
{
	let number = value
		.map(normalize_decimal_lexeme)
		.map(|value| value.parse::<serde_json::Number>())
		.transpose()
		.map_err(|_| Error::InvalidXml {
			message: format!("{field}: invalid numeric value"),
			line: None,
			column: None,
		})?;
	let _ = (number, check_field);
	Ok(())
}

pub(crate) fn normalize_decimal_lexeme(value: &str) -> String {
	let trimmed = value.trim();
	match trimmed.as_bytes() {
		[b'.', ..] => format!("0{trimmed}"),
		[b'-', b'.', ..] => format!("-0{}", &trimmed[1..]),
		[b'+', b'.', ..] => format!("+0{}", &trimmed[1..]),
		_ => trimmed.to_string(),
	}
}

#[cfg(test)]
mod tests {
	use super::{normalize_decimal_lexeme, string};

	#[test]
	fn business_rules_do_not_block_import_parsing() {
		string(
			"safetyReportId",
			Some("invalid"),
			None,
			input_contracts::generated::c::c_1_1,
		)
		.expect("business validation runs after import");
	}

	#[test]
	fn rejects_value_with_null_flavor() {
		let err = string(
			"testDate",
			Some("20260810"),
			Some("UNK"),
			input_contracts::generated::f::f_r_1,
		)
		.unwrap_err();
		assert!(err
			.to_string()
			.contains("value and nullFlavor cannot both be present"));
	}

	#[test]
	fn accepts_xml_decimal_leading_point() {
		assert_eq!(normalize_decimal_lexeme(".5"), "0.5");
		assert_eq!(normalize_decimal_lexeme("-.5"), "-0.5");
		assert_eq!(normalize_decimal_lexeme(" 1.5 "), "1.5");
	}
}
