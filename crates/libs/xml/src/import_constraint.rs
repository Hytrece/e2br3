use crate::{Error, Result};
use validator::{bindings_for_section, validate_portable_value, PortableInputValue};

fn validate(
	section: &str,
	request_path: &str,
	value: PortableInputValue<'_>,
	null_flavor: Option<&str>,
) -> Result<()> {
	let binding = bindings_for_section(section)
		.find(|binding| binding.request_path == request_path)
		.ok_or_else(|| Error::InvalidXml {
			message: format!(
				"missing portable constraint binding for {section}.{request_path}"
			),
			line: None,
			column: None,
		})?;
	for code in binding.rule_codes {
		validate_portable_value(code, value, null_flavor).map_err(|violation| {
			Error::InvalidXml {
				message: format!("{}: {}", violation.code, violation.message),
				line: None,
				column: None,
			}
		})?;
	}
	Ok(())
}

pub(crate) fn string(
	section: &str,
	request_path: &str,
	value: Option<&str>,
	null_flavor: Option<&str>,
) -> Result<()> {
	validate(
		section,
		request_path,
		value.map_or(PortableInputValue::Missing, PortableInputValue::String),
		null_flavor,
	)
}

pub(crate) fn boolean(
	section: &str,
	request_path: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> Result<()> {
	validate(
		section,
		request_path,
		value.map_or(PortableInputValue::Missing, PortableInputValue::Boolean),
		null_flavor,
	)
}

pub(crate) fn number_string(
	section: &str,
	request_path: &str,
	value: Option<&str>,
) -> Result<()> {
	let number = value
		.map(str::parse::<serde_json::Number>)
		.transpose()
		.map_err(|_| Error::InvalidXml {
			message: format!("{section}.{request_path}: invalid numeric value"),
			line: None,
			column: None,
		})?;
	validate(
		section,
		request_path,
		number
			.as_ref()
			.map_or(PortableInputValue::Missing, PortableInputValue::Number),
		None,
	)
}
