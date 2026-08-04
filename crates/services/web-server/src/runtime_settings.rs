use chrono::{DateTime, Datelike, Utc};
use chrono_tz::Tz;
use lib_core::model::admin_settings::AdminSettingsBmc;
use lib_core::model::ModelManager;
use lib_core::regulatory::RegulatoryAuthority;
use lib_rest_core::{Error, Result};
use serde_json::Value;
use std::collections::HashSet;
use time::{Date, Month};

pub const SETTINGS_KEY: &str = "system";
pub const DEFAULT_TIMEZONE: &str = "Asia/Seoul";
pub const DEFAULT_NOTATION: bool = false;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ImportDateSettings {
	pub update_date_of_creation: bool,
	pub update_most_recent_info_date: bool,
	pub update_report_first_received_date: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeSettings {
	pub timezone: String,
	pub appendices: Vec<RegulatoryAuthority>,
	pub notation: bool,
	pub import_dates: ImportDateSettings,
	pub apply_sender_info_to_imported_cases: bool,
	pub orientation: String,
	pub data_ordering: String,
}

impl Default for RuntimeSettings {
	fn default() -> Self {
		Self {
			timezone: DEFAULT_TIMEZONE.to_string(),
			appendices: vec![RegulatoryAuthority::Ich],
			notation: DEFAULT_NOTATION,
			import_dates: ImportDateSettings::default(),
			apply_sender_info_to_imported_cases: false,
			orientation: "Landscape".to_string(),
			data_ordering: "Primary data will appear first".to_string(),
		}
	}
}

impl RuntimeSettings {
	fn from_value(value: Option<&Value>) -> Self {
		let defaults = Self::default();
		let Some(value) = value else {
			return defaults;
		};
		let import_dates =
			value.get("import_date_update").and_then(Value::as_object);
		Self {
			timezone: value
				.get("timezone")
				.and_then(Value::as_str)
				.map(str::trim)
				.filter(|value| !value.is_empty())
				.unwrap_or(DEFAULT_TIMEZONE)
				.to_string(),
			appendices: parse_appendices(value.get("appendices")),
			notation: value
				.get("notation")
				.and_then(Value::as_bool)
				.unwrap_or(DEFAULT_NOTATION),
			import_dates: ImportDateSettings {
				update_date_of_creation: import_dates
					.and_then(|value| value.get("date_of_creation"))
					.and_then(Value::as_bool)
					.unwrap_or(false),
				update_most_recent_info_date: import_dates
					.and_then(|value| value.get("most_recent_info_date"))
					.and_then(Value::as_bool)
					.unwrap_or(false),
				update_report_first_received_date: import_dates
					.and_then(|value| value.get("report_first_received_date"))
					.and_then(Value::as_bool)
					.unwrap_or(false),
			},
			apply_sender_info_to_imported_cases: value
				.get("apply_sender_info_to_imported_cases")
				.and_then(Value::as_bool)
				.unwrap_or(false),
			orientation: value
				.get("orientation")
				.and_then(Value::as_str)
				.map(str::trim)
				.filter(|value| !value.is_empty())
				.unwrap_or(&defaults.orientation)
				.to_string(),
			data_ordering: value
				.get("data_ordering")
				.and_then(Value::as_str)
				.map(str::trim)
				.filter(|value| !value.is_empty())
				.unwrap_or(&defaults.data_ordering)
				.to_string(),
		}
	}

	pub fn resolve_notation(&self, requested: Option<bool>) -> bool {
		requested.unwrap_or(self.notation)
	}

	pub fn import_date(&self) -> Date {
		let timezone = self
			.timezone
			.parse::<Tz>()
			.expect("runtime settings timezone is validated when loaded");
		let now = Utc::now();
		let local = DateTime::<Utc>::from_timestamp(
			now.timestamp(),
			now.timestamp_subsec_nanos(),
		)
		.expect("current UTC timestamp is valid")
		.with_timezone(&timezone)
		.date_naive();
		Date::from_calendar_date(
			local.year(),
			Month::try_from(local.month() as u8).expect("chrono month is valid"),
			local.day() as u8,
		)
		.expect("current local date is valid")
	}
}

pub fn validate_timezone(value: &str) -> Option<String> {
	let value = value.trim();
	if value.is_empty() {
		return None;
	}
	value
		.parse::<Tz>()
		.ok()
		.map(|timezone| timezone.to_string())
}

pub async fn load(
	ctx: &lib_core::ctx::Ctx,
	mm: &ModelManager,
) -> Result<RuntimeSettings> {
	let value = AdminSettingsBmc::get(ctx, mm, SETTINGS_KEY)
		.await
		.map_err(Error::Model)?;
	let settings = RuntimeSettings::from_value(value.as_ref());
	if validate_timezone(&settings.timezone).is_none() {
		return Err(Error::BadRequest {
			message: "stored timezone must be a valid IANA timezone".to_string(),
		});
	}
	if !matches!(settings.orientation.as_str(), "Portrait" | "Landscape") {
		return Err(Error::BadRequest {
			message: "stored orientation must be Portrait or Landscape".to_string(),
		});
	}
	Ok(settings)
}

pub fn normalize_appendices(value: Option<&[String]>) -> Vec<String> {
	let selected = value
		.unwrap_or(&[])
		.iter()
		.map(|value| value.trim().to_ascii_uppercase())
		.collect::<HashSet<_>>();
	let values = ["ICH", "FDA", "MFDS"]
		.into_iter()
		.filter(|value| selected.contains(*value))
		.map(str::to_string)
		.collect::<Vec<_>>();
	values
}

fn parse_appendices(value: Option<&Value>) -> Vec<RegulatoryAuthority> {
	let appendices = value
		.and_then(Value::as_array)
		.map(|values| {
			values
				.iter()
				.filter_map(Value::as_str)
				.filter_map(RegulatoryAuthority::parse)
				.collect::<Vec<_>>()
		})
		.unwrap_or_default();
	appendices
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn validates_only_real_iana_timezones() {
		assert_eq!(
			validate_timezone(" Asia/Seoul "),
			Some("Asia/Seoul".to_string())
		);
		assert_eq!(validate_timezone("not-a-timezone"), None);
	}

	#[test]
	fn resolves_notation_and_appendices_from_admin_settings() {
		let value = serde_json::json!({
			"notation": true,
			"appendices": ["FDA"],
		});
		let settings = RuntimeSettings::from_value(Some(&value));

		assert!(settings.resolve_notation(None));
		assert!(!settings.resolve_notation(Some(false)));
		assert_eq!(settings.appendices, vec![RegulatoryAuthority::Fda]);
	}

	#[test]
	fn does_not_invent_an_appendix_for_empty_settings() {
		let value = serde_json::json!({"appendices": []});
		let settings = RuntimeSettings::from_value(Some(&value));

		assert!(settings.appendices.is_empty());
		assert!(normalize_appendices(Some(&[] as &[String])).is_empty());
	}
}
