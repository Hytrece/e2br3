//! Shared primitives used by explicit field validators.

use crate::allowed_value::{
	is_allowed_value_valid, is_named_vocabulary_value_valid, ConstraintValue,
};
use crate::context::VocabularyContext;
use crate::{
	max_length_for_rule, push_issue_by_code, push_issue_if_rule_invalid,
	vocabulary_for_rule, vocabulary_variant_for_rule, RuleFacts, ValidationIssue,
};
use sqlx::types::time::{Date, OffsetDateTime};
use std::borrow::Cow;

pub(crate) enum RuleValue<'a> {
	Text {
		value: Option<Cow<'a, str>>,
		null_flavor: Option<&'a str>,
	},
}

impl<'a> RuleValue<'a> {
	pub(crate) fn borrowed(
		value: Option<&'a str>,
		null_flavor: Option<&'a str>,
	) -> Self {
		Self::Text {
			value: value.map(Cow::Borrowed),
			null_flavor,
		}
	}

	pub(crate) fn owned(
		value: Option<String>,
		null_flavor: Option<&'a str>,
	) -> Self {
		Self::Text {
			value: value.map(Cow::Owned),
			null_flavor,
		}
	}
}

pub(crate) enum DateValues {
	One(Option<Date>),
	Two(Option<Date>, Option<Date>),
}

impl DateValues {
	fn any_future(self) -> bool {
		match self {
			Self::One(value) => is_future_date(value),
			Self::Two(left, right) => is_future_date(left) || is_future_date(right),
		}
	}
}

fn is_future_date(value: Option<Date>) -> bool {
	value.is_some_and(|value| value > OffsetDateTime::now_utc().date())
}

pub(crate) fn e2b_datetime_date(value: Option<&str>) -> Option<Date> {
	value.and_then(lib_core::serde::flex_date::e2b_datetime_date)
}

pub(crate) fn validate_constraint(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	value: ConstraintValue<'_>,
	vocabulary: &VocabularyContext,
) {
	if !is_allowed_value_valid(code, value, vocabulary) {
		push_issue_by_code(issues, code, path);
	}
}

pub(crate) fn validate_value(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	value: RuleValue<'_>,
	facts: RuleFacts,
) {
	let RuleValue::Text { value, null_flavor } = value;
	let _ = push_issue_if_rule_invalid(
		issues,
		code,
		path,
		value.as_deref(),
		null_flavor,
		facts,
	);
}

pub(crate) fn validate_violation(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	violated: bool,
) {
	if violated {
		push_issue_by_code(issues, code, path);
	}
}

pub(crate) fn validate_future_date(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	dates: DateValues,
) {
	if dates.any_future() {
		push_issue_by_code(issues, code, path);
	}
}

pub(crate) fn validate_length(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	value: Option<&str>,
) {
	let Some(value) = value else {
		return;
	};
	let max_length =
		max_length_for_rule(code).expect("length rule code should exist in catalog");
	if value.chars().count() > max_length {
		push_issue_by_code(issues, code, path);
	}
}

pub(crate) fn validate_vocabulary_variant(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: &str,
	receiver: Option<&str>,
	value: Option<&str>,
	vocabulary: &VocabularyContext,
) {
	let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
		return;
	};
	let Some(variant) = receiver
		.map(str::trim)
		.filter(|receiver| !receiver.is_empty())
		.and_then(|receiver| vocabulary_variant_for_rule(code, receiver))
	else {
		return;
	};
	if !is_named_vocabulary_value_valid(
		variant.vocabulary,
		variant.scope,
		value,
		vocabulary,
	) {
		push_issue_by_code(issues, code, path);
	}
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_meddra(
	issues: &mut Vec<ValidationIssue>,
	vocabulary: &VocabularyContext,
	version_allowed_code: &str,
	code_allowed_code: &str,
	version_code: &str,
	code_code: &str,
	version_path: String,
	code_path: String,
	version: Option<&str>,
	code: Option<&str>,
) {
	for (allowed_code, vocabulary_code, path, value) in [
		(
			version_allowed_code,
			version_code,
			version_path.clone(),
			version,
		),
		(code_allowed_code, code_code, code_path.clone(), code),
	] {
		if !is_allowed_value_valid(
			vocabulary_code,
			ConstraintValue::Text(value.map(Cow::Borrowed)),
			vocabulary,
		) {
			push_issue_by_code(issues, allowed_code, path);
		}
	}
	if !vocabulary.meddra_available() {
		return;
	}
	assert_eq!(vocabulary_for_rule(version_code), Some("MedDRA"));
	assert_eq!(vocabulary_for_rule(code_code), Some("MedDRA"));
	let version = version.map(str::trim).filter(|value| !value.is_empty());
	let code = code.map(str::trim).filter(|value| !value.is_empty());
	if version.is_some_and(|value| !vocabulary.contains_meddra_version(value)) {
		push_issue_by_code(issues, version_code, version_path);
	}
	if let (Some(version), Some(code)) = (version, code) {
		if !vocabulary.contains_meddra_term(version, code) {
			push_issue_by_code(issues, code_code, code_path);
		}
	}
}
