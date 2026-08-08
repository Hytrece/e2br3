use crate::{Error, Result};
use input_contracts::{FieldInput, InputIssue, InputValue};

fn check(
	field: &str,
	value: InputValue<'_>,
	null_flavor: Option<&str>,
	check: impl for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
) -> Result<()> {
	if let Some(issue) = check(FieldInput::new(value, null_flavor))
		.into_iter()
		.next()
	{
		return Err(Error::InvalidXml {
			message: format!("{} ({field}): {}", issue.code, issue.message),
			line: None,
			column: None,
		});
	}
	Ok(())
}

pub(crate) fn string<F>(
	field: &str,
	value: Option<&str>,
	null_flavor: Option<&str>,
	check_field: F,
) -> Result<()>
where
	F: for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
{
	check(
		field,
		value.map_or(InputValue::Missing, InputValue::String),
		null_flavor,
		check_field,
	)
}

pub(crate) fn boolean<F>(
	field: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
	check_field: F,
) -> Result<()>
where
	F: for<'a> Fn(FieldInput<'a>) -> Vec<InputIssue>,
{
	check(
		field,
		value.map_or(InputValue::Missing, InputValue::Boolean),
		null_flavor,
		check_field,
	)
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
	check(
		field,
		number
			.as_ref()
			.map_or(InputValue::Missing, InputValue::Number),
		None,
		check_field,
	)
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
	use super::normalize_decimal_lexeme;

	#[test]
	fn accepts_xml_decimal_leading_point() {
		assert_eq!(normalize_decimal_lexeme(".5"), "0.5");
		assert_eq!(normalize_decimal_lexeme("-.5"), "-0.5");
		assert_eq!(normalize_decimal_lexeme(" 1.5 "), "1.5");
	}
}
