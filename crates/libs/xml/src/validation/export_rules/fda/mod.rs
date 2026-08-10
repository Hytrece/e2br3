use crate::XmlValidationError;
use libxml::xpath::Context;
use time::{format_description, OffsetDateTime, PrimitiveDateTime};

mod c;
mod d;
mod e;
mod f;
mod g;
mod h;
mod n;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	n::run(xpath, errors);
	c::run(xpath, errors);
	d::run(xpath, errors);
	e::run(xpath, errors);
	f::run(xpath, errors);
	g::run(xpath, errors);
	h::run(xpath, errors);
}

fn parse_hl7_datetime(value: &str) -> Option<OffsetDateTime> {
	let value = value.trim();
	let local =
		format_description::parse("[year][month][day][hour][minute][second]")
			.ok()?;
	match value.len() {
		14 => PrimitiveDateTime::parse(value, &local)
			.ok()
			.map(PrimitiveDateTime::assume_utc),
		19 => {
			let offset = format_description::parse(
				"[year][month][day][hour][minute][second][offset_hour sign:mandatory][offset_minute]",
			)
			.ok()?;
			OffsetDateTime::parse(value, &offset).ok()
		}
		_ => None,
	}
}

pub(super) fn reject_future_datetime(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	expression: &str,
	now: OffsetDateTime,
) {
	let future = xpath
		.findvalues(expression, None)
		.unwrap_or_default()
		.into_iter()
		.filter_map(|value| parse_hl7_datetime(&value))
		.any(|value| value > now);
	if future {
		errors.push(XmlValidationError {
			message: format!("[{code}] FDA business rule failed."),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(true),
			line: None,
			column: None,
		});
	}
}
