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
		.map(str::parse::<serde_json::Number>)
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
