use crate::{FormatName, InputIssue, InputValue, NumericShape};
use base64::engine::{general_purpose, Engine};

pub(crate) fn max_length(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	value: InputValue<'_>,
	limit: usize,
) {
	let valid = match normalized(value) {
		InputValue::Missing => true,
		InputValue::String(value) => value.chars().count() <= limit,
		InputValue::Number(value) => value.to_string().chars().count() <= limit,
		_ => false,
	};
	if !valid {
		push(
			issues,
			code,
			format!("must contain at most {limit} characters"),
		);
	}
}

pub(crate) fn allowed_values(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	value: InputValue<'_>,
	allowed: &'static [&'static str],
) {
	let valid = match normalized(value) {
		InputValue::Missing => true,
		InputValue::String(value) => allowed.contains(&value),
		_ => false,
	};
	if !valid {
		push(
			issues,
			code,
			format!("must be one of: {}", allowed.join(", ")),
		);
	}
}

pub(crate) fn boolean(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	value: InputValue<'_>,
) {
	if !matches!(
		normalized(value),
		InputValue::Missing | InputValue::Boolean(_)
	) {
		push(issues, code, "must be a boolean".to_string());
	}
}

pub(crate) fn numeric(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	value: InputValue<'_>,
	shape: NumericShape,
) {
	let valid = match normalized(value) {
		InputValue::Missing | InputValue::Number(_) => true,
		InputValue::String(value) => match shape {
			NumericShape::Decimal => valid_decimal(value),
			NumericShape::DottedVersion => valid_dotted_version(value),
		},
		_ => false,
	};
	if !valid {
		push(
			issues,
			code,
			"must have the expected numeric form".to_string(),
		);
	}
}

pub(crate) fn format(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	value: InputValue<'_>,
	format: FormatName,
) {
	let valid = match normalized(value) {
		InputValue::Missing => true,
		InputValue::String(value) => match format {
			FormatName::E2bDatetime => valid_e2b_datetime(value),
			FormatName::Base64 => general_purpose::STANDARD.decode(value).is_ok(),
		},
		_ => false,
	};
	if !valid {
		push(issues, code, "must have the expected format".to_string());
	}
}

pub(crate) fn null_flavor(
	issues: &mut Vec<InputIssue>,
	code: &'static str,
	null_flavor: Option<&str>,
	allowed: &'static [&'static str],
) {
	if present(null_flavor).is_some_and(|value| !allowed.contains(&value)) {
		push(
			issues,
			code,
			format!("nullFlavor must be one of: {}", allowed.join(", ")),
		);
	}
}

fn push(issues: &mut Vec<InputIssue>, code: &'static str, message: String) {
	issues.push(InputIssue { code, message });
}

fn present(value: Option<&str>) -> Option<&str> {
	value.map(str::trim).filter(|value| !value.is_empty())
}

fn normalized(value: InputValue<'_>) -> InputValue<'_> {
	match value {
		InputValue::String(value) => present(Some(value))
			.map(InputValue::String)
			.unwrap_or(InputValue::Missing),
		value => value,
	}
}

fn valid_decimal(value: &str) -> bool {
	let value = value.trim();
	if value.is_empty() {
		return false;
	}
	let value = value.strip_prefix(['+', '-']).unwrap_or(value);
	let mut parts = value.split('.');
	let whole = parts.next().unwrap_or_default();
	let fraction = parts.next();
	parts.next().is_none()
		&& whole.bytes().all(|byte| byte.is_ascii_digit())
		&& fraction.is_none_or(|part| {
			!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
		}) && (!whole.is_empty() || fraction.is_some())
}

fn valid_dotted_version(value: &str) -> bool {
	let mut parts = value.trim().split('.');
	let valid_part = |part: &str| {
		!part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())
	};
	parts.next().is_some_and(valid_part)
		&& parts.next().is_some_and(valid_part)
		&& parts.next().is_none()
}

fn valid_e2b_datetime(value: &str) -> bool {
	let (local, offset) = match value
		.char_indices()
		.skip(4)
		.find(|(_, character)| matches!(character, '+' | '-'))
	{
		Some((index, _)) => (&value[..index], Some(&value[index..])),
		None => (value, None),
	};
	if let Some(offset) = offset {
		let bytes = offset.as_bytes();
		if bytes.len() != 5
			|| !matches!(bytes[0], b'+' | b'-')
			|| !bytes[1..].iter().all(u8::is_ascii_digit)
			|| !matches!(offset[1..3].parse::<u8>().ok(), Some(0..=14))
			|| !matches!(offset[3..5].parse::<u8>().ok(), Some(0..=59))
		{
			return false;
		}
	}

	let (digits, fraction) = local
		.split_once('.')
		.map_or((local, None), |(digits, fraction)| (digits, Some(fraction)));
	if !matches!(digits.len(), 4 | 6 | 8 | 10 | 12 | 14)
		|| !digits.bytes().all(|byte| byte.is_ascii_digit())
		|| fraction.is_some_and(|fraction| {
			digits.len() != 14
				|| fraction.is_empty()
				|| fraction.len() > 4
				|| !fraction.bytes().all(|byte| byte.is_ascii_digit())
		}) {
		return false;
	}

	let number = |range: std::ops::Range<usize>| {
		digits.get(range).and_then(|value| value.parse::<u8>().ok())
	};
	if digits.len() >= 6 && !matches!(number(4..6), Some(1..=12)) {
		return false;
	}
	if digits.len() >= 8 {
		let year = digits[0..4].parse::<u16>().unwrap_or_default();
		let month = digits[4..6].parse::<u8>().unwrap_or_default();
		let day = digits[6..8].parse::<u8>().unwrap_or_default();
		let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
		let days = match month {
			2 if leap => 29,
			2 => 28,
			4 | 6 | 9 | 11 => 30,
			1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
			_ => return false,
		};
		if day == 0 || day > days {
			return false;
		}
	}
	(digits.len() < 10 || matches!(number(8..10), Some(0..=23)))
		&& (digits.len() < 12 || matches!(number(10..12), Some(0..=59)))
		&& (digits.len() < 14 || matches!(number(12..14), Some(0..=59)))
}

#[cfg(test)]
mod tests {
	use super::{valid_decimal, valid_dotted_version, valid_e2b_datetime};

	#[test]
	fn primitive_contracts_cover_boundaries() {
		assert!(valid_decimal("-12.5"));
		assert!(valid_decimal(".5"));
		assert!(!valid_decimal("12."));
		assert!(valid_dotted_version("2.1"));
		assert!(!valid_dotted_version("2"));
		assert!(valid_e2b_datetime("20240229123059+0900"));
		assert!(!valid_e2b_datetime("20230229"));
	}
}
