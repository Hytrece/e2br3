use super::helpers::{
	e2b_datetime_date, validate_constraint, validate_future_date, validate_length,
	validate_value, validate_violation, DateValues, RuleValue,
};
use crate::allowed_value::{true_marker_value, ConstraintValue};
use crate::{
	has_any_primary_source_content, has_text, is_fda_ind_message_receiver,
	is_fda_pre_anda_message_receiver, list_study_registrations,
	FdaValidationContext, MfdsValidationContext, RegulatoryAuthority, RuleFacts,
	ValidationContext, ValidationIssue,
};
use lib_core::ctx::Ctx;
use lib_core::model::case_identifiers::{LinkedReportNumber, OtherCaseIdentifier};
use lib_core::model::safety_report::{
	DocumentsHeldBySender, LiteratureReference, PrimarySource,
	SafetyReportIdentification, SenderInformation, StudyInformation,
	StudyRegistrationNumber,
};
use lib_core::model::{ModelManager, Result};
use std::borrow::Cow;
use std::collections::HashMap;

fn is_later_than(
	value: Option<sqlx::types::time::Date>,
	other: Option<sqlx::types::time::Date>,
) -> bool {
	matches!((value, other), (Some(value), Some(other)) if value > other)
}

fn index_from_sequence(sequence_number: i32, fallback_idx: usize) -> usize {
	sequence_number
		.checked_sub(1)
		.and_then(|value| usize::try_from(value).ok())
		.unwrap_or(fallback_idx)
}

pub(crate) async fn collect(
	issues: &mut Vec<ValidationIssue>,
	authority: RegulatoryAuthority,
	mm: &ModelManager,
	ctx: &Ctx,
	validation_ctx: &ValidationContext,
	fda_ctx: Option<&FdaValidationContext>,
	mfds_ctx: Option<&MfdsValidationContext>,
) -> Result<()> {
	collect_ich_issues(validation_ctx, issues);
	match authority {
		RegulatoryAuthority::Ich => {}
		RegulatoryAuthority::Fda => {
			if let Some(fda_ctx) = fda_ctx {
				collect_fda_issues(ctx, mm, validation_ctx, fda_ctx, issues).await?;
			}
		}
		RegulatoryAuthority::Mfds => {
			if let Some(mfds_ctx) = mfds_ctx {
				collect_mfds_issues(validation_ctx, mfds_ctx, issues);
			}
		}
	}
	Ok(())
}

/// ICH.C.1.REQUIRED
fn c_1(
	report: Option<&SafetyReportIdentification>,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"ICH.C.1.REQUIRED",
		"safetyReportIdentification",
		RuleValue::borrowed(report.map(|_| "present"), None),
		RuleFacts::default(),
	);
}

/// ICH.C.1.1.REQUIRED
/// ICH.C.1.1.LENGTH.MAX
fn c_1_1(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.C.1.1.REQUIRED",
		"safetyReportIdentification.safetyReportId",
		RuleValue::borrowed(report.safety_report_id.as_deref(), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.C.1.1.LENGTH.MAX",
		"safetyReportIdentification.safetyReportId",
		report.safety_report_id.as_deref(),
	);
}

/// ICH.C.1.2.REQUIRED
/// ICH.C.1.2.FUTURE_DATE.FORBIDDEN
/// ICH.C.1.2.ALLOWED.VALUE
fn c_1_2(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.transmissionDate";
	validate_value(
		issues,
		"ICH.C.1.2.REQUIRED",
		PATH,
		RuleValue::borrowed(report.transmission_date.as_deref(), None),
		RuleFacts::default(),
	);
	validate_future_date(
		issues,
		"ICH.C.1.2.FUTURE_DATE.FORBIDDEN",
		PATH,
		DateValues::One(e2b_datetime_date(report.transmission_date.as_deref())),
	);
	validate_constraint(
		issues,
		"ICH.C.1.2.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(
			report.transmission_date.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
}

/// ICH.C.1.3.REQUIRED
/// ICH.C.1.3.ALLOWED.VALUE
/// ICH.C.1.3.LENGTH.MAX
fn c_1_3(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.reportType";
	validate_value(
		issues,
		"ICH.C.1.3.REQUIRED",
		PATH,
		RuleValue::borrowed(report.report_type.as_deref(), None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.1.3.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(report.report_type.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.1.3.LENGTH.MAX",
		PATH,
		report.report_type.as_deref(),
	);
}

/// ICH.C.1.4.REQUIRED
/// ICH.C.1.4.FUTURE_DATE.FORBIDDEN
/// ICH.C.1.4.AFTER_C.1.2.FORBIDDEN
/// ICH.C.1.4.AFTER_C.1.5.FORBIDDEN
fn c_1_4(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "safetyReportIdentification.dateFirstReceivedFromSource";
	validate_value(
		issues,
		"ICH.C.1.4.REQUIRED",
		PATH,
		RuleValue::owned(
			report
				.date_first_received_from_source
				.map(|value| value.to_string()),
			None,
		),
		RuleFacts::default(),
	);
	validate_future_date(
		issues,
		"ICH.C.1.4.FUTURE_DATE.FORBIDDEN",
		PATH,
		DateValues::One(report.date_first_received_from_source),
	);
	validate_violation(
		issues,
		"ICH.C.1.4.AFTER_C.1.2.FORBIDDEN",
		PATH,
		is_later_than(
			report.date_first_received_from_source,
			e2b_datetime_date(report.transmission_date.as_deref()),
		),
	);
	validate_violation(
		issues,
		"ICH.C.1.4.AFTER_C.1.5.FORBIDDEN",
		PATH,
		is_later_than(
			report.date_first_received_from_source,
			report.date_of_most_recent_information,
		),
	);
}

/// ICH.C.1.5.REQUIRED
/// ICH.C.1.5.FUTURE_DATE.FORBIDDEN
/// ICH.C.1.5.AFTER_C.1.2.FORBIDDEN
fn c_1_5(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "safetyReportIdentification.dateOfMostRecentInformation";
	validate_value(
		issues,
		"ICH.C.1.5.REQUIRED",
		PATH,
		RuleValue::owned(
			report
				.date_of_most_recent_information
				.map(|value| value.to_string()),
			None,
		),
		RuleFacts::default(),
	);
	validate_future_date(
		issues,
		"ICH.C.1.5.FUTURE_DATE.FORBIDDEN",
		PATH,
		DateValues::One(report.date_of_most_recent_information),
	);
	validate_violation(
		issues,
		"ICH.C.1.5.AFTER_C.1.2.FORBIDDEN",
		PATH,
		is_later_than(
			report.date_of_most_recent_information,
			e2b_datetime_date(report.transmission_date.as_deref()),
		),
	);
}

/// ICH.C.1.6.1.REQUIRED
fn c_1_6_1(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.C.1.6.1.REQUIRED",
		"safetyReportIdentification.additionalDocumentsAvailable",
		RuleValue::borrowed(
			report.additional_documents_available.map(|value| {
				if value {
					"true"
				} else {
					"false"
				}
			}),
			None,
		),
		RuleFacts::default(),
	);
}

/// ICH.C.1.7.REQUIRED
fn c_1_7(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.C.1.7.REQUIRED",
		"safetyReportIdentification.fulfilExpeditedCriteria",
		RuleValue::borrowed(
			report
				.fulfil_expedited_criteria
				.map(|value| if value { "1" } else { "2" }),
			report.fulfil_expedited_criteria_null_flavor.as_deref(),
		),
		RuleFacts::default(),
	);
}

/// ICH.C.1.8.1.REQUIRED
/// ICH.C.1.8.1.ALLOWED.VALUE
/// ICH.C.1.8.1.LENGTH.MAX
fn c_1_8_1(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.worldwideUniqueId";
	validate_value(
		issues,
		"ICH.C.1.8.1.REQUIRED",
		PATH,
		RuleValue::borrowed(report.worldwide_unique_id.as_deref(), None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.1.8.1.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(
			report.worldwide_unique_id.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.1.8.1.LENGTH.MAX",
		PATH,
		report.worldwide_unique_id.as_deref(),
	);
}

/// ICH.C.1.8.2.REQUIRED
/// ICH.C.1.8.2.ALLOWED.VALUE
/// ICH.C.1.8.2.LENGTH.MAX
fn c_1_8_2(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.firstSenderType";
	validate_value(
		issues,
		"ICH.C.1.8.2.REQUIRED",
		PATH,
		RuleValue::borrowed(report.first_sender_type.as_deref(), None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.1.8.2.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(
			report.first_sender_type.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.1.8.2.LENGTH.MAX",
		PATH,
		report.first_sender_type.as_deref(),
	);
}

/// ICH.C.1.9.1.REQUIRED
/// ICH.C.1.9.1.ALLOWED.VALUE
fn c_1_9_1(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.otherCaseIdentifiersExist";
	validate_value(
		issues,
		"ICH.C.1.9.1.REQUIRED",
		PATH,
		RuleValue::borrowed(
			report.other_case_identifiers_exist.map(|value| {
				if value {
					"true"
				} else {
					"false"
				}
			}),
			report.other_case_identifiers_exist_null_flavor.as_deref(),
		),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.1.9.1.ALLOWED.VALUE",
		PATH,
		true_marker_value(
			report.other_case_identifiers_exist,
			report.other_case_identifiers_exist_null_flavor.as_deref(),
		),
		vocabulary,
	);
}

/// ICH.C.1.11.1.ALLOWED.VALUE
/// ICH.C.1.11.1.LENGTH.MAX
fn c_1_11_1(
	report: &SafetyReportIdentification,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "safetyReportIdentification.nullificationAmendmentCode";
	validate_constraint(
		issues,
		"ICH.C.1.11.1.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(
			report.nullification_code.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.1.11.1.LENGTH.MAX",
		PATH,
		report.nullification_code.as_deref(),
	);
}

/// ICH.C.1.11.2.REQUIRED
/// ICH.C.1.11.2.LENGTH.MAX
fn c_1_11_2(report: &SafetyReportIdentification, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.C.1.11.2.REQUIRED",
		"safetyReportIdentification.nullificationReason",
		RuleValue::borrowed(report.nullification_reason.as_deref(), None),
		RuleFacts {
			ich_nullification_code_present: Some(has_text(
				report.nullification_code.as_deref(),
			)),
			..RuleFacts::default()
		},
	);
	validate_length(
		issues,
		"ICH.C.1.11.2.LENGTH.MAX",
		"safetyReportIdentification.nullificationReason",
		report.nullification_reason.as_deref(),
	);
}

fn primary_source_regulatory_is_one(source: &PrimarySource) -> bool {
	source.primary_source_regulatory.as_deref().map(str::trim) == Some("1")
}

/// ICH.C.2.r.3.REQUIRED
/// ICH.C.2.r.3.LENGTH.MAX
/// ICH.C.2.r.3.VOCABULARY
fn c_2_r_3(
	idx: usize,
	source: &PrimarySource,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("primarySources.{idx}.reporterCountry");
	if primary_source_regulatory_is_one(source) {
		validate_value(
			issues,
			"ICH.C.2.r.3.REQUIRED",
			&path,
			RuleValue::borrowed(
				source.country_code.as_deref(),
				source.country_code_null_flavor.as_deref(),
			),
			RuleFacts::default(),
		);
	}
	validate_length(
		issues,
		"ICH.C.2.r.3.LENGTH.MAX",
		&path,
		source.country_code.as_deref(),
	);
	validate_constraint(
		issues,
		"ICH.C.2.r.3.VOCABULARY",
		&path,
		ConstraintValue::Text(source.country_code.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
}

/// FDA.C.2.r.2.8.REQUIRED
fn c_2_r_2_8(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	let path = format!("primarySources.{idx}.reporterEmail");
	validate_value(
		issues,
		"FDA.C.2.r.2.8.REQUIRED",
		&path,
		RuleValue::borrowed(
			source.email.as_deref(),
			source.email_null_flavor.as_deref(),
		),
		RuleFacts::default(),
	);
}

fn c_2_length(
	idx: usize,
	code: &str,
	field: &str,
	value: Option<&str>,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("primarySources.{idx}.{field}");
	validate_length(issues, code, &path, value);
}

/// ICH.C.2.r.1.1.LENGTH.MAX
fn c_2_r_1_1(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.1.1.LENGTH.MAX",
		"reporterTitle",
		source.reporter_title.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.1.2.LENGTH.MAX
fn c_2_r_1_2(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.1.2.LENGTH.MAX",
		"reporterGivenName",
		source.reporter_given_name.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.1.3.LENGTH.MAX
fn c_2_r_1_3(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.1.3.LENGTH.MAX",
		"reporterMiddleName",
		source.reporter_middle_name.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.1.4.LENGTH.MAX
fn c_2_r_1_4(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.1.4.LENGTH.MAX",
		"reporterFamilyName",
		source.reporter_family_name.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.1.REQUIRED
/// ICH.C.2.r.2.1.LENGTH.MAX
fn c_2_r_2_1(
	sources: &[PrimarySource],
	report_type_is_study: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let (value, null_flavor) = sources
		.iter()
		.find_map(|source| {
			let value = source
				.organization
				.as_deref()
				.map(str::trim)
				.filter(|value| !value.is_empty());
			let null_flavor = source
				.organization_null_flavor
				.as_deref()
				.map(str::trim)
				.filter(|value| !value.is_empty());
			(value.is_some() || null_flavor.is_some())
				.then_some((value, null_flavor))
		})
		.unwrap_or((None, None));
	validate_value(
		issues,
		"ICH.C.2.r.2.1.REQUIRED",
		"primarySources.0.reporterOrganization",
		RuleValue::borrowed(value, null_flavor),
		RuleFacts {
			ich_report_type_is_study: Some(report_type_is_study),
			..RuleFacts::default()
		},
	);
	for (idx, source) in sources.iter().enumerate() {
		c_2_length(
			idx,
			"ICH.C.2.r.2.1.LENGTH.MAX",
			"reporterOrganization",
			source.organization.as_deref(),
			issues,
		);
	}
}

/// ICH.C.2.r.2.2.LENGTH.MAX
fn c_2_r_2_2(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.2.LENGTH.MAX",
		"reporterDepartment",
		source.department.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.3.LENGTH.MAX
fn c_2_r_2_3(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.3.LENGTH.MAX",
		"reporterStreet",
		source.street.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.4.LENGTH.MAX
fn c_2_r_2_4(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.4.LENGTH.MAX",
		"reporterCity",
		source.city.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.5.LENGTH.MAX
fn c_2_r_2_5(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.5.LENGTH.MAX",
		"reporterState",
		source.state.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.6.LENGTH.MAX
fn c_2_r_2_6(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.6.LENGTH.MAX",
		"reporterPostcode",
		source.postcode.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.2.7.LENGTH.MAX
fn c_2_r_2_7(idx: usize, source: &PrimarySource, issues: &mut Vec<ValidationIssue>) {
	c_2_length(
		idx,
		"ICH.C.2.r.2.7.LENGTH.MAX",
		"reporterTelephone",
		source.telephone.as_deref(),
		issues,
	);
}

/// ICH.C.2.r.4.REQUIRED
/// ICH.C.2.r.4.ALLOWED.VALUE
/// ICH.C.2.r.4.LENGTH.MAX
fn c_2_r_4(
	idx: usize,
	source: &PrimarySource,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("primarySources.{idx}.qualification");
	if has_any_primary_source_content(source) {
		validate_value(
			issues,
			"ICH.C.2.r.4.REQUIRED",
			&path,
			RuleValue::borrowed(source.qualification.as_deref(), None),
			RuleFacts::default(),
		);
	}
	validate_constraint(
		issues,
		"ICH.C.2.r.4.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(source.qualification.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.2.r.4.LENGTH.MAX",
		&path,
		source.qualification.as_deref(),
	);
}

/// ICH.C.2.r.5.REQUIRED
/// ICH.C.2.r.5.ALLOWED.VALUE
/// ICH.C.2.r.5.LENGTH.MAX
fn c_2_r_5(
	sources: &[PrimarySource],
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	for (idx, source) in sources.iter().enumerate() {
		let path =
			format!("primarySources.{idx}.primarySourceForRegulatoryPurposes");
		validate_constraint(
			issues,
			"ICH.C.2.r.5.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(
				source
					.primary_source_regulatory
					.as_deref()
					.map(Cow::Borrowed),
			),
			vocabulary,
		);
		validate_length(
			issues,
			"ICH.C.2.r.5.LENGTH.MAX",
			&path,
			source.primary_source_regulatory.as_deref(),
		);
	}
	validate_value(
		issues,
		"ICH.C.2.r.5.REQUIRED",
		"primarySources.0.primarySourceForRegulatoryPurposes",
		RuleValue::borrowed(
			sources
				.iter()
				.any(primary_source_regulatory_is_one)
				.then_some("present"),
			None,
		),
		RuleFacts::default(),
	);
}

fn c_3_length(
	code: &str,
	field: &str,
	value: Option<&str>,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("senderInformation.{field}");
	validate_length(issues, code, &path, value);
}

/// ICH.C.3.1.REQUIRED
/// ICH.C.3.1.ALLOWED.VALUE
/// ICH.C.3.1.LENGTH.MAX
fn c_3_1(
	sender: Option<&SenderInformation>,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "senderInformation.senderType";
	let value = sender.and_then(|sender| sender.sender_type.as_deref());
	validate_value(
		issues,
		"ICH.C.3.1.REQUIRED",
		PATH,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.3.1.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(value.map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(issues, "ICH.C.3.1.LENGTH.MAX", PATH, value);
}

/// ICH.C.3.2.REQUIRED
/// ICH.C.3.2.LENGTH.MAX
fn c_3_2(sender: Option<&SenderInformation>, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "senderInformation.organizationName";
	let value = sender.and_then(|sender| sender.organization_name.as_deref());
	validate_value(
		issues,
		"ICH.C.3.2.REQUIRED",
		PATH,
		RuleValue::borrowed(value, None),
		RuleFacts {
			ich_sender_organization_required: Some(
				sender
					.and_then(|sender| sender.sender_type.as_deref())
					.map(str::trim) != Some("7"),
			),
			..RuleFacts::default()
		},
	);
	validate_length(issues, "ICH.C.3.2.LENGTH.MAX", PATH, value);
}

/// ICH.C.3.3.1.LENGTH.MAX
fn c_3_3_1(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.3.1.LENGTH.MAX",
		"department",
		sender.department.as_deref(),
		issues,
	);
}

/// ICH.C.3.3.2.LENGTH.MAX
fn c_3_3_2(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.3.2.LENGTH.MAX",
		"personTitle",
		sender.person_title.as_deref(),
		issues,
	);
}

/// ICH.C.3.3.3.LENGTH.MAX
fn c_3_3_3(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.3.3.LENGTH.MAX",
		"personGivenName",
		sender.person_given_name.as_deref(),
		issues,
	);
}

/// ICH.C.3.3.4.LENGTH.MAX
fn c_3_3_4(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.3.4.LENGTH.MAX",
		"personMiddleName",
		sender.person_middle_name.as_deref(),
		issues,
	);
}

/// ICH.C.3.3.5.LENGTH.MAX
fn c_3_3_5(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.3.5.LENGTH.MAX",
		"personFamilyName",
		sender.person_family_name.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.1.LENGTH.MAX
fn c_3_4_1(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.1.LENGTH.MAX",
		"streetAddress",
		sender.street_address.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.2.LENGTH.MAX
fn c_3_4_2(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.2.LENGTH.MAX",
		"city",
		sender.city.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.3.LENGTH.MAX
fn c_3_4_3(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.3.LENGTH.MAX",
		"state",
		sender.state.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.4.LENGTH.MAX
fn c_3_4_4(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.4.LENGTH.MAX",
		"postcode",
		sender.postcode.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.5.VOCABULARY
/// ICH.C.3.4.5.LENGTH.MAX
fn c_3_4_5(
	sender: &SenderInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "senderInformation.countryCode";
	validate_constraint(
		issues,
		"ICH.C.3.4.5.VOCABULARY",
		PATH,
		ConstraintValue::Text(sender.country_code.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.C.3.4.5.LENGTH.MAX",
		PATH,
		sender.country_code.as_deref(),
	);
}

/// ICH.C.3.4.6.LENGTH.MAX
fn c_3_4_6(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.6.LENGTH.MAX",
		"telephone",
		sender.telephone.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.7.LENGTH.MAX
fn c_3_4_7(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.7.LENGTH.MAX",
		"fax",
		sender.fax.as_deref(),
		issues,
	);
}

/// ICH.C.3.4.8.LENGTH.MAX
fn c_3_4_8(sender: &SenderInformation, issues: &mut Vec<ValidationIssue>) {
	c_3_length(
		"ICH.C.3.4.8.LENGTH.MAX",
		"email",
		sender.email.as_deref(),
		issues,
	);
}

/// ICH.C.1.6.1.r.1.REQUIRED
/// ICH.C.1.6.1.r.1.LENGTH.MAX
fn c_1_6_1_r_1(
	idx: usize,
	document: &DocumentsHeldBySender,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("documentsHeldBySender.{idx}.documentDescription");
	validate_value(
		issues,
		"ICH.C.1.6.1.r.1.REQUIRED",
		&path,
		RuleValue::borrowed(document.title.as_deref(), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.C.1.6.1.r.1.LENGTH.MAX",
		&path,
		document.title.as_deref(),
	);
}

/// ICH.C.1.6.1.r.2.ALLOWED.VALUE
fn c_1_6_1_r_2(
	idx: usize,
	document: &DocumentsHeldBySender,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("documentsHeldBySender.{idx}.includedDocument");
	validate_constraint(
		issues,
		"ICH.C.1.6.1.r.2.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(
			document.document_base64.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
}

/// ICH.C.1.9.1.r.1.REQUIRED
/// ICH.C.1.9.1.r.1.LENGTH.MAX
fn c_1_9_1_r_1(
	idx: usize,
	identifier: &OtherCaseIdentifier,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("otherCaseIdentifiers.{idx}.source");
	validate_value(
		issues,
		"ICH.C.1.9.1.r.1.REQUIRED",
		&path,
		RuleValue::borrowed(Some(identifier.source_of_identifier.as_str()), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.C.1.9.1.r.1.LENGTH.MAX",
		&path,
		Some(identifier.source_of_identifier.as_str()),
	);
}

/// ICH.C.1.9.1.r.2.REQUIRED
/// ICH.C.1.9.1.r.2.ALLOWED.VALUE
/// ICH.C.1.9.1.r.2.LENGTH.MAX
fn c_1_9_1_r_2(
	idx: usize,
	identifier: &OtherCaseIdentifier,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("otherCaseIdentifiers.{idx}.caseIdentifier");
	let value = Some(identifier.case_identifier.as_str());
	validate_value(
		issues,
		"ICH.C.1.9.1.r.2.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.C.1.9.1.r.2.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(value.map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(issues, "ICH.C.1.9.1.r.2.LENGTH.MAX", &path, value);
}

/// ICH.C.1.10.r.LENGTH.MAX
fn c_1_10_r(
	idx: usize,
	report: &LinkedReportNumber,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("linkedReports.{idx}.linkedReportNumber");
	validate_length(
		issues,
		"ICH.C.1.10.r.LENGTH.MAX",
		&path,
		Some(report.linked_report_number.as_str()),
	);
}

/// ICH.C.4.r.1.LENGTH.MAX
fn c_4_r_1(
	idx: usize,
	reference: &LiteratureReference,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("literatureReferences.{idx}.referenceText");
	validate_length(
		issues,
		"ICH.C.4.r.1.LENGTH.MAX",
		&path,
		reference.reference_text.as_deref(),
	);
}

/// ICH.C.4.r.2.ALLOWED.VALUE
fn c_4_r_2(
	idx: usize,
	reference: &LiteratureReference,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("literatureReferences.{idx}.documentBase64");
	validate_constraint(
		issues,
		"ICH.C.4.r.2.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(
			reference.document_base64.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
}

/// ICH.C.5.1.r.1.LENGTH.MAX
fn c_5_1_r_1(
	study_idx: usize,
	idx: usize,
	registration: &StudyRegistrationNumber,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!(
		"studyInformation.{study_idx}.registrations.{idx}.registrationNumber"
	);
	validate_length(
		issues,
		"ICH.C.5.1.r.1.LENGTH.MAX",
		&path,
		Some(registration.registration_number.as_str()),
	);
}

/// ICH.C.5.1.r.2.LENGTH.MAX
/// ICH.C.5.1.r.2.VOCABULARY
fn c_5_1_r_2(
	study_idx: usize,
	idx: usize,
	registration: &StudyRegistrationNumber,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!(
		"studyInformation.{study_idx}.registrations.{idx}.registrationCountry"
	);
	validate_length(
		issues,
		"ICH.C.5.1.r.2.LENGTH.MAX",
		&path,
		registration.country_code.as_deref(),
	);
	validate_constraint(
		issues,
		"ICH.C.5.1.r.2.VOCABULARY",
		&path,
		ConstraintValue::Text(
			registration.country_code.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
}

/// ICH.C.5.2.LENGTH.MAX
fn c_5_2(idx: usize, study: &StudyInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("studyInformation.{idx}.studyName");
	validate_length(
		issues,
		"ICH.C.5.2.LENGTH.MAX",
		&path,
		study.study_name.as_deref(),
	);
}

/// ICH.C.5.3.REQUIRED
/// ICH.C.5.3.LENGTH.MAX
fn c_5_3(
	idx: usize,
	study: &StudyInformation,
	report_type_is_study: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("studyInformation.{idx}.sponsorStudyNumber");
	if report_type_is_study {
		validate_value(
			issues,
			"ICH.C.5.3.REQUIRED",
			&path,
			RuleValue::borrowed(study.sponsor_study_number.as_deref(), None),
			RuleFacts {
				ich_report_type_is_study: Some(true),
				..RuleFacts::default()
			},
		);
	}
	validate_length(
		issues,
		"ICH.C.5.3.LENGTH.MAX",
		&path,
		study.sponsor_study_number.as_deref(),
	);
}

/// ICH.C.5.4.REQUIRED
/// ICH.C.5.4.ALLOWED.VALUE
/// ICH.C.5.4.LENGTH.MAX
fn c_5_4(
	studies: &[StudyInformation],
	report_type_is_study: bool,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"ICH.C.5.4.REQUIRED",
		"studyInformation.0.studyTypeReaction",
		RuleValue::borrowed((!studies.is_empty()).then_some("present"), None),
		RuleFacts {
			ich_report_type_is_study: Some(report_type_is_study),
			..RuleFacts::default()
		},
	);
	for (idx, study) in studies.iter().enumerate() {
		let path = format!("studyInformation.{idx}.studyTypeReaction");
		if report_type_is_study {
			validate_value(
				issues,
				"ICH.C.5.4.REQUIRED",
				&path,
				RuleValue::borrowed(study.study_type_reaction.as_deref(), None),
				RuleFacts {
					ich_report_type_is_study: Some(true),
					..RuleFacts::default()
				},
			);
		}
		validate_constraint(
			issues,
			"ICH.C.5.4.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(
				study.study_type_reaction.as_deref().map(Cow::Borrowed),
			),
			vocabulary,
		);
		validate_length(
			issues,
			"ICH.C.5.4.LENGTH.MAX",
			&path,
			study.study_type_reaction.as_deref(),
		);
	}
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	c_1(validation_ctx.safety_report.as_ref(), issues);
	if let Some(report) = validation_ctx.safety_report.as_ref() {
		c_1_1(report, issues);
		c_1_2(report, &validation_ctx.vocabulary, issues);
		c_1_3(report, &validation_ctx.vocabulary, issues);
		c_1_4(report, issues);
		c_1_5(report, issues);
		c_1_6_1(report, issues);
		c_1_7(report, issues);
		c_1_8_1(report, &validation_ctx.vocabulary, issues);
		c_1_8_2(report, &validation_ctx.vocabulary, issues);
		c_1_9_1(report, &validation_ctx.vocabulary, issues);
		c_1_11_1(report, &validation_ctx.vocabulary, issues);
		c_1_11_2(report, issues);
	} else {
		push_missing_safety_report_field_issues(issues);
	}

	let report_type_is_study =
		validation_ctx.safety_report.as_ref().is_some_and(|report| {
			report.report_type.as_deref().map(str::trim) == Some("2")
		});
	for (idx, source) in validation_ctx.primary_sources.iter().enumerate() {
		c_2_r_3(idx, source, &validation_ctx.vocabulary, issues);
		c_2_r_4(idx, source, &validation_ctx.vocabulary, issues);
		c_2_r_1_1(idx, source, issues);
		c_2_r_1_2(idx, source, issues);
		c_2_r_1_3(idx, source, issues);
		c_2_r_1_4(idx, source, issues);
		c_2_r_2_2(idx, source, issues);
		c_2_r_2_3(idx, source, issues);
		c_2_r_2_4(idx, source, issues);
		c_2_r_2_5(idx, source, issues);
		c_2_r_2_6(idx, source, issues);
		c_2_r_2_7(idx, source, issues);
		c_2_r_2_8(idx, source, issues);
	}
	c_2_r_2_1(
		&validation_ctx.primary_sources,
		report_type_is_study,
		issues,
	);
	c_2_r_5(
		&validation_ctx.primary_sources,
		&validation_ctx.vocabulary,
		issues,
	);

	for (idx, document) in validation_ctx.documents_held_by_sender.iter().enumerate()
	{
		c_1_6_1_r_1(idx, document, issues);
		c_1_6_1_r_2(idx, document, &validation_ctx.vocabulary, issues);
	}
	for (idx, reference) in validation_ctx.literature_references.iter().enumerate() {
		c_4_r_1(idx, reference, issues);
		c_4_r_2(idx, reference, &validation_ctx.vocabulary, issues);
	}
	for (idx, identifier) in validation_ctx.other_case_identifiers.iter().enumerate()
	{
		c_1_9_1_r_1(idx, identifier, issues);
		c_1_9_1_r_2(idx, identifier, &validation_ctx.vocabulary, issues);
	}
	for (idx, report) in validation_ctx.linked_report_numbers.iter().enumerate() {
		c_1_10_r(idx, report, issues);
	}
	for (idx, study) in validation_ctx.studies.iter().enumerate() {
		c_5_2(idx, study, issues);
		c_5_3(idx, study, report_type_is_study, issues);
	}
	c_5_4(
		&validation_ctx.studies,
		report_type_is_study,
		&validation_ctx.vocabulary,
		issues,
	);
	let study_indices = validation_ctx
		.studies
		.iter()
		.enumerate()
		.map(|(idx, study)| (study.id, idx))
		.collect::<HashMap<_, _>>();
	let mut fallback_idx_by_study = HashMap::new();
	for registration in &validation_ctx.study_registrations {
		let study_id = registration.study_information_id;
		let Some(study_idx) = study_indices.get(&study_id).copied() else {
			continue;
		};
		let fallback_idx = fallback_idx_by_study.entry(study_id).or_insert(0);
		let idx = index_from_sequence(registration.sequence_number, *fallback_idx);
		*fallback_idx += 1;
		c_5_1_r_1(study_idx, idx, registration, issues);
		c_5_1_r_2(
			study_idx,
			idx,
			registration,
			&validation_ctx.vocabulary,
			issues,
		);
	}

	c_3_1(
		validation_ctx.sender.as_ref(),
		&validation_ctx.vocabulary,
		issues,
	);
	c_3_2(validation_ctx.sender.as_ref(), issues);
	if let Some(sender) = validation_ctx.sender.as_ref() {
		c_3_3_1(sender, issues);
		c_3_3_2(sender, issues);
		c_3_3_3(sender, issues);
		c_3_3_4(sender, issues);
		c_3_3_5(sender, issues);
		c_3_4_1(sender, issues);
		c_3_4_2(sender, issues);
		c_3_4_3(sender, issues);
		c_3_4_4(sender, issues);
		c_3_4_5(sender, &validation_ctx.vocabulary, issues);
		c_3_4_6(sender, issues);
		c_3_4_7(sender, issues);
		c_3_4_8(sender, issues);
	}

	if validation_ctx.primary_sources.is_empty() {
		crate::push_issue_by_code(
			issues,
			"ICH.C.2.r.4.REQUIRED",
			"primarySources.0.qualification",
		);
	}
}

fn push_missing_safety_report_field_issues(issues: &mut Vec<ValidationIssue>) {
	for (code, path) in [
		(
			"ICH.C.1.1.REQUIRED",
			"safetyReportIdentification.safetyReportId",
		),
		(
			"ICH.C.1.2.REQUIRED",
			"safetyReportIdentification.transmissionDate",
		),
		(
			"ICH.C.1.3.REQUIRED",
			"safetyReportIdentification.reportType",
		),
		(
			"ICH.C.1.4.REQUIRED",
			"safetyReportIdentification.dateFirstReceivedFromSource",
		),
		(
			"ICH.C.1.5.REQUIRED",
			"safetyReportIdentification.dateOfMostRecentInformation",
		),
		(
			"ICH.C.1.7.REQUIRED",
			"safetyReportIdentification.fulfilExpeditedCriteria",
		),
	] {
		crate::push_issue_by_code(issues, code, path);
	}
}

/// FDA.C.1.7.1.REQUIRED
fn fda_c_1_7_1(
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"FDA.C.1.7.1.REQUIRED",
		"safetyReportIdentification.localCriteriaReportType",
		RuleValue::borrowed(value, None),
		facts,
	);
}

/// FDA.C.1.12.REQUIRED
/// FDA.C.1.12.RECOMMENDED
fn fda_c_1_12(
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str =
		"safetyReportIdentification.combinationProductReportIndicator";
	for code in ["FDA.C.1.12.REQUIRED", "FDA.C.1.12.RECOMMENDED"] {
		validate_value(issues, code, PATH, RuleValue::borrowed(value, None), facts);
	}
}

/// FDA.C.2.r.2.EMAIL.REQUIRED
fn fda_c_2_r_2_email(
	idx: usize,
	value: Option<&str>,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("primarySources.{idx}.reporterEmail");
	validate_value(
		issues,
		"FDA.C.2.r.2.EMAIL.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts {
			fda_primary_source_present: Some(true),
			..RuleFacts::default()
		},
	);
}

/// FDA.C.5.5a.REQUIRED
fn fda_c_5_5a(
	study_number: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"FDA.C.5.5a.REQUIRED",
		"studyInformation.sponsorStudyNumber",
		RuleValue::borrowed(study_number, None),
		facts,
	);
}

/// FDA.C.5.5b.REQUIRED
fn fda_c_5_5b(
	study_number: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"FDA.C.5.5b.REQUIRED",
		"studyInformation.sponsorStudyNumber",
		RuleValue::borrowed(study_number, None),
		facts,
	);
}

/// FDA.C.5.6.r.REQUIRED
fn fda_c_5_6_r(
	has_cross_reported: bool,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"FDA.C.5.6.r.REQUIRED",
		"studyInformation.registrations.0.registrationNumber",
		RuleValue::borrowed(has_cross_reported.then_some("present"), None),
		facts,
	);
}

/// MFDS.C.3.1.KR.1.REQUIRED
fn mfds_c_3_1_kr_1(
	idx: usize,
	value: Option<&str>,
	sender_is_health_professional: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("senderInformation.{idx}.healthProfessionalTypeKr1");
	validate_value(
		issues,
		"MFDS.C.3.1.KR.1.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts {
			mfds_sender_type_is_health_professional: Some(
				sender_is_health_professional,
			),
			..RuleFacts::default()
		},
	);
}

/// MFDS.C.2.r.4.KR.1.REQUIRED
fn mfds_c_2_r_4_kr_1(
	idx: usize,
	value: Option<&str>,
	qualification_is_three: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("primarySources.{idx}.qualificationKr1");
	validate_value(
		issues,
		"MFDS.C.2.r.4.KR.1.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts {
			mfds_primary_source_qualification_is_three: Some(qualification_is_three),
			..RuleFacts::default()
		},
	);
}

/// MFDS.C.5.4.KR.1.REQUIRED
fn mfds_c_5_4_kr_1(
	idx: usize,
	value: Option<&str>,
	study_type_is_three: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("studyInformation.{idx}.studyTypeReactionKr1");
	validate_value(
		issues,
		"MFDS.C.5.4.KR.1.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts {
			mfds_study_type_reaction_is_three: Some(study_type_is_three),
			..RuleFacts::default()
		},
	);
}

pub(crate) async fn collect_fda_issues(
	ctx: &Ctx,
	mm: &ModelManager,
	validation_ctx: &ValidationContext,
	fda_ctx: &FdaValidationContext,
	issues: &mut Vec<ValidationIssue>,
) -> Result<()> {
	if let Some(report) = validation_ctx.safety_report.as_ref() {
		let facts = RuleFacts {
			fda_fulfil_expedited_criteria: Some(
				report.fulfil_expedited_criteria.unwrap_or(false),
			),
			fda_combination_product_true: Some(
				report.combination_product_report_indicator.as_deref()
					== Some("true"),
			),
			..RuleFacts::default()
		};
		fda_c_1_7_1(report.local_criteria_report_type.as_deref(), facts, issues);
		fda_c_1_12(
			report.combination_product_report_indicator.as_deref(),
			facts,
			issues,
		);
	}

	let type_of_report = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|r| r.report_type.as_deref());
	let message_receiver = validation_ctx
		.message_header
		.as_ref()
		.map(|h| h.message_receiver_identifier.as_str());
	let study_number = fda_ctx
		.studies
		.first()
		.and_then(|s| s.sponsor_study_number.as_deref())
		.map(str::trim)
		.filter(|v| !v.is_empty());
	let has_ind_number = study_number.is_some();
	let has_cross_reported = if has_ind_number {
		if let Some(study) = fda_ctx.studies.first() {
			list_study_registrations(ctx, mm, study.id)
				.await?
				.iter()
				.any(|reg| !reg.registration_number.trim().is_empty())
		} else {
			false
		}
	} else {
		false
	};
	let facts = RuleFacts {
		fda_type_of_report_is_one_or_two: Some(matches!(
			type_of_report,
			Some("1") | Some("2")
		)),
		fda_type_of_report_is_two: Some(type_of_report == Some("2")),
		fda_msg_receiver_is_cder_ind_or_cber_ind: Some(is_fda_ind_message_receiver(
			message_receiver,
		)),
		fda_msg_receiver_is_cder_ind_exempt_ba_be: Some(
			is_fda_pre_anda_message_receiver(message_receiver),
		),
		fda_has_ind_number: Some(has_ind_number),
		..RuleFacts::default()
	};
	fda_c_5_5a(study_number, facts, issues);
	fda_c_5_5b(study_number, facts, issues);
	fda_c_5_6_r(has_cross_reported, facts, issues);

	for (idx, source) in validation_ctx.primary_sources.iter().enumerate() {
		if has_any_primary_source_content(source) {
			fda_c_2_r_2_email(idx, source.email.as_deref(), issues);
		}
	}
	Ok(())
}

pub(crate) fn collect_mfds_issues(
	validation_ctx: &ValidationContext,
	mfds_ctx: &MfdsValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	for (idx, sender) in mfds_ctx.senders.iter().enumerate() {
		mfds_c_3_1_kr_1(
			idx,
			sender.health_professional_type_kr1.as_deref(),
			sender.sender_type.as_deref().map(str::trim) == Some("3"),
			issues,
		);
	}
	for (idx, source) in validation_ctx.primary_sources.iter().enumerate() {
		mfds_c_2_r_4_kr_1(
			idx,
			source.qualification_kr1.as_deref(),
			source.qualification.as_deref().map(str::trim) == Some("3"),
			issues,
		);
	}
	for (idx, study) in mfds_ctx.studies.iter().enumerate() {
		mfds_c_5_4_kr_1(
			idx,
			study.study_type_reaction_kr1.as_deref(),
			study.study_type_reaction.as_deref().map(str::trim) == Some("3"),
			issues,
		);
	}
}

#[cfg(test)]
pub(super) fn constraint_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.C.1.2.ALLOWED.VALUE",
		"ICH.C.1.3.ALLOWED.VALUE",
		"ICH.C.1.8.1.ALLOWED.VALUE",
		"ICH.C.1.8.2.ALLOWED.VALUE",
		"ICH.C.1.9.1.ALLOWED.VALUE",
		"ICH.C.1.11.1.ALLOWED.VALUE",
		"ICH.C.1.6.1.r.2.ALLOWED.VALUE",
		"ICH.C.4.r.2.ALLOWED.VALUE",
		"ICH.C.1.9.1.r.2.ALLOWED.VALUE",
		"ICH.C.2.r.4.ALLOWED.VALUE",
		"ICH.C.2.r.5.ALLOWED.VALUE",
		"ICH.C.2.r.3.VOCABULARY",
		"ICH.C.3.1.ALLOWED.VALUE",
		"ICH.C.3.4.5.VOCABULARY",
		"ICH.C.5.4.ALLOWED.VALUE",
		"ICH.C.5.1.r.2.VOCABULARY",
	]
}

#[cfg(test)]
pub(super) fn implemented_rule_codes() -> Vec<&'static str> {
	let codes = vec![
		"ICH.C.1.1.REQUIRED",
		"ICH.C.1.2.REQUIRED",
		"ICH.C.1.3.REQUIRED",
		"ICH.C.1.4.REQUIRED",
		"ICH.C.1.5.REQUIRED",
		"ICH.C.1.6.1.REQUIRED",
		"ICH.C.1.7.REQUIRED",
		"ICH.C.1.8.1.REQUIRED",
		"ICH.C.1.8.2.REQUIRED",
		"ICH.C.1.9.1.REQUIRED",
		"ICH.C.1.2.FUTURE_DATE.FORBIDDEN",
		"ICH.C.1.4.FUTURE_DATE.FORBIDDEN",
		"ICH.C.1.5.FUTURE_DATE.FORBIDDEN",
		"ICH.C.1.2.ALLOWED.VALUE",
		"ICH.C.1.3.ALLOWED.VALUE",
		"ICH.C.1.8.1.ALLOWED.VALUE",
		"ICH.C.1.8.2.ALLOWED.VALUE",
		"ICH.C.1.9.1.ALLOWED.VALUE",
		"ICH.C.1.11.1.ALLOWED.VALUE",
		"ICH.C.1.1.LENGTH.MAX",
		"ICH.C.1.3.LENGTH.MAX",
		"ICH.C.1.8.1.LENGTH.MAX",
		"ICH.C.1.8.2.LENGTH.MAX",
		"ICH.C.1.11.1.LENGTH.MAX",
		"ICH.C.1.11.2.LENGTH.MAX",
		"ICH.C.1.4.AFTER_C.1.2.FORBIDDEN",
		"ICH.C.1.4.AFTER_C.1.5.FORBIDDEN",
		"ICH.C.1.5.AFTER_C.1.2.FORBIDDEN",
		"ICH.C.1.11.2.REQUIRED",
		"ICH.C.2.r.3.REQUIRED",
		"ICH.C.2.r.3.LENGTH.MAX",
		"ICH.C.2.r.3.VOCABULARY",
		"ICH.C.2.r.4.REQUIRED",
		"ICH.C.2.r.4.ALLOWED.VALUE",
		"ICH.C.2.r.4.LENGTH.MAX",
		"ICH.C.2.r.1.1.LENGTH.MAX",
		"ICH.C.2.r.1.2.LENGTH.MAX",
		"ICH.C.2.r.1.3.LENGTH.MAX",
		"ICH.C.2.r.1.4.LENGTH.MAX",
		"ICH.C.2.r.2.1.LENGTH.MAX",
		"ICH.C.2.r.2.2.LENGTH.MAX",
		"ICH.C.2.r.2.3.LENGTH.MAX",
		"ICH.C.2.r.2.4.LENGTH.MAX",
		"ICH.C.2.r.2.5.LENGTH.MAX",
		"ICH.C.2.r.2.6.LENGTH.MAX",
		"ICH.C.2.r.2.7.LENGTH.MAX",
		"ICH.C.2.r.5.REQUIRED",
		"ICH.C.2.r.5.ALLOWED.VALUE",
		"ICH.C.2.r.5.LENGTH.MAX",
		"FDA.C.2.r.2.8.REQUIRED",
		"ICH.C.3.1.REQUIRED",
		"ICH.C.3.1.ALLOWED.VALUE",
		"ICH.C.3.1.LENGTH.MAX",
		"ICH.C.3.2.REQUIRED",
		"ICH.C.3.2.LENGTH.MAX",
		"ICH.C.3.3.1.LENGTH.MAX",
		"ICH.C.3.3.2.LENGTH.MAX",
		"ICH.C.3.3.3.LENGTH.MAX",
		"ICH.C.3.3.4.LENGTH.MAX",
		"ICH.C.3.3.5.LENGTH.MAX",
		"ICH.C.3.4.1.LENGTH.MAX",
		"ICH.C.3.4.2.LENGTH.MAX",
		"ICH.C.3.4.3.LENGTH.MAX",
		"ICH.C.3.4.4.LENGTH.MAX",
		"ICH.C.3.4.5.VOCABULARY",
		"ICH.C.3.4.5.LENGTH.MAX",
		"ICH.C.3.4.6.LENGTH.MAX",
		"ICH.C.3.4.7.LENGTH.MAX",
		"ICH.C.3.4.8.LENGTH.MAX",
		"ICH.C.1.6.1.r.1.REQUIRED",
		"ICH.C.1.6.1.r.1.LENGTH.MAX",
		"ICH.C.1.6.1.r.2.ALLOWED.VALUE",
		"ICH.C.1.9.1.r.1.REQUIRED",
		"ICH.C.1.9.1.r.1.LENGTH.MAX",
		"ICH.C.1.9.1.r.2.REQUIRED",
		"ICH.C.1.9.1.r.2.ALLOWED.VALUE",
		"ICH.C.1.9.1.r.2.LENGTH.MAX",
		"ICH.C.1.10.r.LENGTH.MAX",
		"ICH.C.4.r.1.LENGTH.MAX",
		"ICH.C.4.r.2.ALLOWED.VALUE",
		"ICH.C.5.1.r.1.LENGTH.MAX",
		"ICH.C.5.1.r.2.LENGTH.MAX",
		"ICH.C.5.1.r.2.VOCABULARY",
		"ICH.C.5.2.LENGTH.MAX",
		"ICH.C.5.3.REQUIRED",
		"ICH.C.5.3.LENGTH.MAX",
		"ICH.C.5.4.REQUIRED",
		"ICH.C.5.4.ALLOWED.VALUE",
		"ICH.C.5.4.LENGTH.MAX",
		"FDA.C.1.7.1.REQUIRED",
		"FDA.C.1.12.REQUIRED",
		"FDA.C.1.12.RECOMMENDED",
		"FDA.C.2.r.2.EMAIL.REQUIRED",
		"FDA.C.5.5a.REQUIRED",
		"FDA.C.5.5b.REQUIRED",
		"FDA.C.5.6.r.REQUIRED",
		"MFDS.C.3.1.KR.1.REQUIRED",
		"MFDS.C.2.r.4.KR.1.REQUIRED",
		"MFDS.C.5.4.KR.1.REQUIRED",
		"ICH.C.1.REQUIRED",
		"ICH.C.2.r.2.1.REQUIRED",
	];
	codes
}

#[cfg(test)]
mod conditioned_catalog_rule_tests {
	use super::*;

	#[test]
	fn fda_report_rules_emit_and_pass_from_catalog() {
		let facts = RuleFacts {
			fda_fulfil_expedited_criteria: Some(true),
			fda_combination_product_true: Some(false),
			..RuleFacts::default()
		};
		let mut issues = Vec::new();
		fda_c_1_7_1(None, facts, &mut issues);
		fda_c_1_12(None, facts, &mut issues);
		assert_eq!(
			issues
				.iter()
				.map(|issue| issue.code.as_str())
				.collect::<Vec<_>>(),
			[
				"FDA.C.1.7.1.REQUIRED",
				"FDA.C.1.12.REQUIRED",
				"FDA.C.1.12.RECOMMENDED",
			]
		);

		issues.clear();
		fda_c_1_7_1(Some("1"), facts, &mut issues);
		fda_c_1_12(Some("true"), facts, &mut issues);
		assert!(issues.is_empty());
	}

	#[test]
	fn mfds_rules_preserve_nonzero_paths_and_catalog_conditions() {
		let mut issues = Vec::new();
		mfds_c_3_1_kr_1(2, None, true, &mut issues);
		mfds_c_2_r_4_kr_1(3, None, true, &mut issues);
		mfds_c_5_4_kr_1(4, None, true, &mut issues);

		assert_eq!(issues.len(), 3);
		assert_eq!(
			issues
				.iter()
				.map(|issue| issue.field_path.as_deref().unwrap())
				.collect::<Vec<_>>(),
			[
				"senderInformation.2.healthProfessionalTypeKr1",
				"primarySources.3.qualificationKr1",
				"studyInformation.4.studyTypeReactionKr1",
			]
		);

		issues.clear();
		mfds_c_5_4_kr_1(4, None, false, &mut issues);
		assert!(issues.is_empty());
	}
}

#[cfg(test)]
mod golden_c1_value_tests {
	//! Characterization tests for the one-to-one presence/value rules inside
	//! `collect_ich_issues` (C.1.2 / C.1.3 / C.1.4 / C.1.5 / C.1.7).
	//!
	//! These freeze *current* behavior (code + path + blocking) so the
	//! table-driven refactor can be proven to change nothing. Deliberately
	//! excluded from scope: C.1.1 (fires outside the `if let Some(report)`
	//! block), cross-field date rules (`*.FUTURE_DATE`, `*.AFTER_*`), and the
	//! C.1.7 nullFlavor parity with the dictionary.
	use super::*;
	use lib_core::model::case::Case;
	use lib_core::model::case_identifiers::OtherCaseIdentifier;
	use lib_core::model::safety_report::{
		DocumentsHeldBySender, LiteratureReference, PrimarySource,
		SafetyReportIdentification, SenderInformation, StudyInformation,
		StudyRegistrationNumber,
	};
	use sqlx::types::time::{Date, OffsetDateTime};
	use sqlx::types::Uuid;
	use time::Month;

	const TARGET_CODES: &[&str] = &[
		"ICH.C.1.2.REQUIRED",
		"ICH.C.1.3.REQUIRED",
		"ICH.C.1.4.REQUIRED",
		"ICH.C.1.5.REQUIRED",
		"ICH.C.1.7.REQUIRED",
	];

	const INDEXED_CODES: &[&str] = &[
		"ICH.C.1.6.1.r.1.REQUIRED",
		"ICH.C.1.9.1.r.1.REQUIRED",
		"ICH.C.1.9.1.r.2.REQUIRED",
	];

	const STUDY_CODES: &[&str] = &["ICH.C.5.3.REQUIRED", "ICH.C.5.4.REQUIRED"];

	const LENGTH_CODES: &[&str] = &[
		"ICH.C.1.1.LENGTH.MAX",
		"ICH.C.1.3.LENGTH.MAX",
		"ICH.C.1.6.1.r.1.LENGTH.MAX",
		"ICH.C.1.8.1.LENGTH.MAX",
		"ICH.C.1.8.2.LENGTH.MAX",
		"ICH.C.1.9.1.r.1.LENGTH.MAX",
		"ICH.C.1.9.1.r.2.LENGTH.MAX",
		"ICH.C.1.10.r.LENGTH.MAX",
		"ICH.C.1.11.1.LENGTH.MAX",
		"ICH.C.1.11.2.LENGTH.MAX",
		"ICH.C.5.3.LENGTH.MAX",
		"ICH.C.5.4.LENGTH.MAX",
	];

	const C23_LENGTH_CODES: &[&str] = &[
		"ICH.C.2.r.1.1.LENGTH.MAX",
		"ICH.C.2.r.1.2.LENGTH.MAX",
		"ICH.C.2.r.1.3.LENGTH.MAX",
		"ICH.C.2.r.1.4.LENGTH.MAX",
		"ICH.C.2.r.2.1.LENGTH.MAX",
		"ICH.C.2.r.2.2.LENGTH.MAX",
		"ICH.C.2.r.2.3.LENGTH.MAX",
		"ICH.C.2.r.2.4.LENGTH.MAX",
		"ICH.C.2.r.2.5.LENGTH.MAX",
		"ICH.C.2.r.2.6.LENGTH.MAX",
		"ICH.C.2.r.2.7.LENGTH.MAX",
		"ICH.C.2.r.3.LENGTH.MAX",
		"ICH.C.2.r.4.LENGTH.MAX",
		"ICH.C.2.r.5.LENGTH.MAX",
		"ICH.C.3.1.LENGTH.MAX",
		"ICH.C.3.2.LENGTH.MAX",
		"ICH.C.3.3.1.LENGTH.MAX",
		"ICH.C.3.3.2.LENGTH.MAX",
		"ICH.C.3.3.3.LENGTH.MAX",
		"ICH.C.3.3.4.LENGTH.MAX",
		"ICH.C.3.3.5.LENGTH.MAX",
		"ICH.C.3.4.1.LENGTH.MAX",
		"ICH.C.3.4.2.LENGTH.MAX",
		"ICH.C.3.4.3.LENGTH.MAX",
		"ICH.C.3.4.4.LENGTH.MAX",
		"ICH.C.3.4.5.LENGTH.MAX",
		"ICH.C.3.4.6.LENGTH.MAX",
		"ICH.C.3.4.7.LENGTH.MAX",
		"ICH.C.3.4.8.LENGTH.MAX",
		"ICH.C.5.2.LENGTH.MAX",
	];

	const C45_LENGTH_CODES: &[&str] = &[
		"ICH.C.4.r.1.LENGTH.MAX",
		"ICH.C.5.1.r.1.LENGTH.MAX",
		"ICH.C.5.1.r.2.LENGTH.MAX",
	];

	fn dummy_case() -> Case {
		Case {
			id: Uuid::nil(),
			organization_id: Uuid::nil(),
			dg_prd_key: None,
			status: String::new(),
			status_before_lock: None,
			review_receivers_json: None,
			workflow_routes_json: None,
			workflow_status: String::new(),
			workflow_assigned_role: None,
			workflow_assigned_user_id: None,
			workflow_due_at: None,
			workflow_description: None,
			workflow_updated_at: OffsetDateTime::UNIX_EPOCH,
			mfds_report_type: None,
			fda_report_type: None,
			report_year: None,
			created_by: Uuid::nil(),
			updated_by: None,
			submitted_by: None,
			submitted_at: None,
			raw_xml: None,
			dirty_c: false,
			dirty_d: false,
			dirty_e: false,
			dirty_f: false,
			dirty_g: false,
			dirty_h: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
		}
	}

	fn base_report() -> SafetyReportIdentification {
		SafetyReportIdentification {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			safety_report_id: None,
			version: 0,
			transmission_date: None,
			report_type: None,
			date_first_received_from_source: None,
			date_of_most_recent_information: None,
			fulfil_expedited_criteria: None,
			fulfil_expedited_criteria_null_flavor: None,
			local_criteria_report_type: None,
			combination_product_report_indicator: None,
			combination_product_report_indicator_null_flavor: None,
			worldwide_unique_id: None,
			first_sender_type: None,
			additional_documents_available: None,
			other_case_identifiers_exist: None,
			other_case_identifiers_exist_null_flavor: None,
			nullification_code: None,
			nullification_reason: None,
			receiver_organization: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn ctx_with(report: SafetyReportIdentification) -> ValidationContext {
		ValidationContext {
			vocabulary: Default::default(),
			case: dummy_case(),
			safety_report: Some(report),
			message_header: None,
			sender: None,
			patient: None,
			narrative: None,
			sender_diagnoses: Vec::new(),
			case_summaries: Vec::new(),
			medical_history: Vec::new(),
			past_drugs: Vec::new(),
			death_info: None,
			reported_causes_of_death: Vec::new(),
			autopsy_causes_of_death: Vec::new(),
			parents: Vec::new(),
			parent_medical_history: Vec::new(),
			parent_past_drugs: Vec::new(),
			primary_sources: Vec::new(),
			documents_held_by_sender: Vec::new(),
			literature_references: Vec::new(),
			other_case_identifiers: Vec::new(),
			linked_report_numbers: Vec::new(),
			studies: Vec::new(),
			study_registrations: Vec::new(),
			reactions: Vec::new(),
			tests: Vec::new(),
			drugs: Vec::new(),
			active_substances: Vec::new(),
			indications: Vec::new(),
			dosages: Vec::new(),
			drug_reaction_assessments: Vec::new(),
			relatedness_assessments: Vec::new(),
			patient_identifiers: Vec::new(),
		}
	}

	/// Runs `collect_ich_issues` and returns only the in-scope C.1 value rules
	/// as a sorted `(code, path, blocking)` snapshot. Issue *ordering* is not a
	/// contract (`build_report` aggregates by section), so we compare as a set.
	fn snapshot(report: SafetyReportIdentification) -> Vec<(String, String, bool)> {
		let mut issues = Vec::new();
		collect_ich_issues(&ctx_with(report), &mut issues);
		let mut out: Vec<(String, String, bool)> = issues
			.into_iter()
			.filter(|issue| TARGET_CODES.contains(&issue.code.as_str()))
			.map(|issue| (issue.code, issue.path, issue.blocking))
			.collect();
		out.sort();
		out
	}

	fn issue(code: &str, path: &str, blocking: bool) -> (String, String, bool) {
		(code.to_string(), path.to_string(), blocking)
	}

	/// Sorted `(code, path, blocking)` snapshot filtered to `targets`, for
	/// contexts built with repeated-field fixtures.
	fn filtered(
		ctx: &ValidationContext,
		targets: &[&str],
	) -> Vec<(String, String, bool)> {
		let mut issues = Vec::new();
		collect_ich_issues(ctx, &mut issues);
		let mut out: Vec<(String, String, bool)> = issues
			.into_iter()
			.filter(|issue| targets.contains(&issue.code.as_str()))
			.map(|issue| (issue.code, issue.path, issue.blocking))
			.collect();
		out.sort();
		out
	}

	fn document(title: Option<&str>) -> DocumentsHeldBySender {
		DocumentsHeldBySender {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			title: title.map(str::to_string),
			document_base64: None,
			media_type: None,
			representation: None,
			compression: None,
			sequence_number: 0,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn other_identifier(source: &str, case_identifier: &str) -> OtherCaseIdentifier {
		OtherCaseIdentifier {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			sequence_number: 0,
			source_of_identifier: source.to_string(),
			case_identifier: case_identifier.to_string(),
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn linked_report_number(value: String) -> LinkedReportNumber {
		LinkedReportNumber {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			sequence_number: 1,
			linked_report_number: value,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn primary_source() -> PrimarySource {
		PrimarySource {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_reporter_presave_id: None,
			sequence_number: 0,
			reporter_title: None,
			reporter_title_null_flavor: None,
			reporter_given_name: None,
			reporter_given_name_null_flavor: None,
			reporter_middle_name: None,
			reporter_middle_name_null_flavor: None,
			reporter_family_name: None,
			reporter_family_name_null_flavor: None,
			organization: None,
			organization_null_flavor: None,
			department: None,
			department_null_flavor: None,
			street: None,
			street_null_flavor: None,
			city: None,
			city_null_flavor: None,
			state: None,
			state_null_flavor: None,
			postcode: None,
			postcode_null_flavor: None,
			telephone: None,
			telephone_null_flavor: None,
			country_code: None,
			country_code_null_flavor: None,
			email: None,
			email_null_flavor: None,
			qualification: None,
			qualification_null_flavor: None,
			qualification_kr1: None,
			primary_source_regulatory: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn sender() -> SenderInformation {
		SenderInformation {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_sender_presave_id: None,
			sender_type: None,
			health_professional_type_kr1: None,
			organization_name: None,
			department: None,
			street_address: None,
			city: None,
			state: None,
			postcode: None,
			country_code: None,
			person_title: None,
			person_given_name: None,
			person_middle_name: None,
			person_family_name: None,
			telephone: None,
			fax: None,
			email: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn literature_reference(reference_text: String) -> LiteratureReference {
		LiteratureReference {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			reference_text: Some(reference_text),
			reference_text_null_flavor: None,
			sequence_number: 0,
			document_base64: None,
			media_type: None,
			representation: None,
			compression: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn study_registration(
		study_information_id: Uuid,
		registration_number: String,
		country_code: Option<String>,
		sequence_number: i32,
	) -> StudyRegistrationNumber {
		StudyRegistrationNumber {
			id: Uuid::nil(),
			study_information_id,
			registration_number,
			registration_number_null_flavor: None,
			country_code,
			country_code_null_flavor: None,
			sequence_number,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn study(
		study_type_reaction: Option<&str>,
		sponsor_study_number: Option<&str>,
	) -> StudyInformation {
		StudyInformation {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_study_presave_id: None,
			study_name: None,
			study_name_null_flavor: None,
			sponsor_study_number: sponsor_study_number.map(str::to_string),
			sponsor_study_number_null_flavor: None,
			study_type_reaction: study_type_reaction.map(str::to_string),
			study_type_reaction_kr1: None,
			fda_ind_number_occurred: None,
			fda_pre_anda_number_occurred: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn study_report() -> SafetyReportIdentification {
		let mut report = base_report();
		report.report_type = Some("2".to_string());
		report
	}

	#[test]
	fn all_missing_flags_every_value_rule() {
		assert_eq!(
			snapshot(base_report()),
			vec![
				issue(
					"ICH.C.1.2.REQUIRED",
					"safetyReportIdentification.transmissionDate",
					true
				),
				issue(
					"ICH.C.1.3.REQUIRED",
					"safetyReportIdentification.reportType",
					true
				),
				issue(
					"ICH.C.1.4.REQUIRED",
					"safetyReportIdentification.dateFirstReceivedFromSource",
					true
				),
				issue(
					"ICH.C.1.5.REQUIRED",
					"safetyReportIdentification.dateOfMostRecentInformation",
					true
				),
				issue(
					"ICH.C.1.7.REQUIRED",
					"safetyReportIdentification.fulfilExpeditedCriteria",
					true
				),
			]
		);
	}

	#[test]
	fn all_present_flags_nothing() {
		let mut report = base_report();
		report.safety_report_id = Some("US-ABC-1".to_string());
		report.transmission_date = Some("20200101120000".to_string());
		report.report_type = Some("1".to_string());
		report.date_first_received_from_source =
			Some(Date::from_calendar_date(2020, Month::January, 1).unwrap());
		report.date_of_most_recent_information =
			Some(Date::from_calendar_date(2020, Month::January, 1).unwrap());
		report.fulfil_expedited_criteria = Some(true);
		assert_eq!(snapshot(report), Vec::new());
	}

	#[test]
	fn c1_7_nullflavor_only_satisfies_required_value() {
		let mut report = base_report();
		report.fulfil_expedited_criteria = None;
		report.fulfil_expedited_criteria_null_flavor = Some("NI".to_string());
		let snap = snapshot(report);
		assert!(
			!snap.iter().any(|(code, _, _)| code == "ICH.C.1.7.REQUIRED"),
			"expected C.1.7 nullFlavor-only to satisfy required value, got {snap:?}"
		);
	}

	#[test]
	fn indexed_document_missing_title_flags_matching_index() {
		let mut ctx = ctx_with(base_report());
		ctx.documents_held_by_sender =
			vec![document(Some("attached")), document(None)];
		assert_eq!(
			filtered(&ctx, INDEXED_CODES),
			vec![issue(
				"ICH.C.1.6.1.r.1.REQUIRED",
				"documentsHeldBySender.1.documentDescription",
				true
			)]
		);
	}

	#[test]
	fn indexed_other_identifiers_flag_empty_fields_per_index() {
		let mut ctx = ctx_with(base_report());
		ctx.other_case_identifiers =
			vec![other_identifier("SRC", "ID"), other_identifier("", "")];
		assert_eq!(
			filtered(&ctx, INDEXED_CODES),
			vec![
				issue(
					"ICH.C.1.9.1.r.1.REQUIRED",
					"otherCaseIdentifiers.1.source",
					true
				),
				issue(
					"ICH.C.1.9.1.r.2.REQUIRED",
					"otherCaseIdentifiers.1.caseIdentifier",
					true
				),
			]
		);
	}

	#[test]
	fn ci_repeating_issues_use_editor_field_names() {
		let mut ctx = ctx_with(base_report());
		let mut document = document(None);
		document.document_base64 = Some("not-base64".to_string());
		ctx.documents_held_by_sender = vec![document];
		ctx.other_case_identifiers = vec![other_identifier("", "")];
		ctx.linked_report_numbers = vec![linked_report_number("L".repeat(101))];

		assert_eq!(
			filtered(
				&ctx,
				&[
					"ICH.C.1.6.1.r.1.REQUIRED",
					"ICH.C.1.6.1.r.2.ALLOWED.VALUE",
					"ICH.C.1.9.1.r.1.REQUIRED",
					"ICH.C.1.9.1.r.2.REQUIRED",
					"ICH.C.1.10.r.LENGTH.MAX",
				],
			),
			vec![
				issue(
					"ICH.C.1.10.r.LENGTH.MAX",
					"linkedReports.0.linkedReportNumber",
					true
				),
				issue(
					"ICH.C.1.6.1.r.1.REQUIRED",
					"documentsHeldBySender.0.documentDescription",
					true
				),
				issue(
					"ICH.C.1.6.1.r.2.ALLOWED.VALUE",
					"documentsHeldBySender.0.includedDocument",
					true
				),
				issue(
					"ICH.C.1.9.1.r.1.REQUIRED",
					"otherCaseIdentifiers.0.source",
					true
				),
				issue(
					"ICH.C.1.9.1.r.2.REQUIRED",
					"otherCaseIdentifiers.0.caseIdentifier",
					true
				),
			]
		);
	}

	#[test]
	fn study_rules_flag_missing_fields_when_study_report() {
		let mut ctx = ctx_with(study_report());
		ctx.studies = vec![study(None, None)];
		assert_eq!(
			filtered(&ctx, STUDY_CODES),
			vec![
				issue(
					"ICH.C.5.3.REQUIRED",
					"studyInformation.0.sponsorStudyNumber",
					true
				),
				issue(
					"ICH.C.5.4.REQUIRED",
					"studyInformation.0.studyTypeReaction",
					true
				),
			]
		);
	}

	#[test]
	fn study_rules_silent_when_not_study_report() {
		// report_type != "2" gates the whole study block off.
		let mut ctx = ctx_with(base_report());
		ctx.studies = vec![study(None, None)];
		assert_eq!(filtered(&ctx, STUDY_CODES), Vec::new());
	}

	#[test]
	fn study_rules_pass_when_fields_present() {
		let mut ctx = ctx_with(study_report());
		ctx.studies = vec![study(Some("1"), Some("SPONSOR-1"))];
		assert_eq!(filtered(&ctx, STUDY_CODES), Vec::new());
	}

	#[test]
	fn study_reporter_organization_null_flavor_satisfies_required_rule() {
		let mut source = primary_source();
		source.organization_null_flavor = Some("NASK".to_string());
		let mut ctx = ctx_with(study_report());
		ctx.primary_sources = vec![source];

		assert_eq!(filtered(&ctx, &["ICH.C.2.r.2.1.REQUIRED"]), Vec::new());
	}

	#[test]
	fn fda_primary_source_email_rule_emits_once() {
		let mut ctx = ctx_with(base_report());
		ctx.primary_sources = vec![primary_source()];

		assert_eq!(
			filtered(&ctx, &["FDA.C.2.r.2.8.REQUIRED"]),
			vec![issue(
				"FDA.C.2.r.2.8.REQUIRED",
				"primarySources.0.reporterEmail",
				true
			)]
		);
	}

	#[test]
	fn allowed_value_rule_flags_invalid_report_type() {
		let mut report = base_report();
		report.report_type = Some("9".to_string());
		let ctx = ctx_with(report);

		assert_eq!(
			filtered(&ctx, &["ICH.C.1.3.ALLOWED.VALUE"]),
			vec![issue(
				"ICH.C.1.3.ALLOWED.VALUE",
				"safetyReportIdentification.reportType",
				true
			)]
		);
	}

	#[test]
	fn datetime_format_rule_flags_invalid_transmission_date() {
		let mut report = base_report();
		report.transmission_date = Some("not-a-date".to_string());
		let ctx = ctx_with(report);

		assert_eq!(
			filtered(&ctx, &["ICH.C.1.2.ALLOWED.VALUE"]),
			vec![issue(
				"ICH.C.1.2.ALLOWED.VALUE",
				"safetyReportIdentification.transmissionDate",
				true
			)]
		);
	}

	#[test]
	fn allowed_value_rules_cover_c_sender_source_and_study_codes() {
		let mut report = base_report();
		report.first_sender_type = Some("9".to_string());
		report.other_case_identifiers_exist = Some(false);
		report.nullification_code = Some("9".to_string());
		let mut ctx = ctx_with(report);

		let mut source = primary_source();
		source.qualification = Some("9".to_string());
		source.primary_source_regulatory = Some("9".to_string());
		ctx.primary_sources = vec![source];

		let mut sender = sender();
		sender.sender_type = Some("9".to_string());
		ctx.sender = Some(sender);

		ctx.studies = vec![study(Some("9"), Some("SPONSOR-1"))];

		assert_eq!(
			filtered(
				&ctx,
				&[
					"ICH.C.1.8.2.ALLOWED.VALUE",
					"ICH.C.1.9.1.ALLOWED.VALUE",
					"ICH.C.1.11.1.ALLOWED.VALUE",
					"ICH.C.2.r.4.ALLOWED.VALUE",
					"ICH.C.2.r.5.ALLOWED.VALUE",
					"ICH.C.3.1.ALLOWED.VALUE",
					"ICH.C.5.4.ALLOWED.VALUE",
				],
			),
			vec![
				issue(
					"ICH.C.1.11.1.ALLOWED.VALUE",
					"safetyReportIdentification.nullificationAmendmentCode",
					true
				),
				issue(
					"ICH.C.1.8.2.ALLOWED.VALUE",
					"safetyReportIdentification.firstSenderType",
					true
				),
				issue(
					"ICH.C.1.9.1.ALLOWED.VALUE",
					"safetyReportIdentification.otherCaseIdentifiersExist",
					true
				),
				issue(
					"ICH.C.2.r.4.ALLOWED.VALUE",
					"primarySources.0.qualification",
					true
				),
				issue(
					"ICH.C.2.r.5.ALLOWED.VALUE",
					"primarySources.0.primarySourceForRegulatoryPurposes",
					true
				),
				issue(
					"ICH.C.3.1.ALLOWED.VALUE",
					"senderInformation.senderType",
					true
				),
				issue(
					"ICH.C.5.4.ALLOWED.VALUE",
					"studyInformation.0.studyTypeReaction",
					true
				),
			]
		);
	}

	#[test]
	fn true_marker_allows_dictionary_null_flavor() {
		let mut report = base_report();
		report.other_case_identifiers_exist = Some(false);
		report.other_case_identifiers_exist_null_flavor = Some("NI".to_string());
		let ctx = ctx_with(report);

		assert_eq!(filtered(&ctx, &["ICH.C.1.9.1.ALLOWED.VALUE"]), Vec::new());
	}

	#[test]
	fn max_length_rules_cover_c1_and_indexed_fields() {
		let mut report = base_report();
		report.safety_report_id = Some("S".repeat(101));
		report.report_type = Some("22".to_string());
		report.worldwide_unique_id = Some("W".repeat(101));
		report.first_sender_type = Some("12".to_string());
		report.nullification_code = Some("12".to_string());
		report.nullification_reason = Some("R".repeat(2001));
		let mut ctx = ctx_with(report);
		ctx.documents_held_by_sender = vec![document(Some(&"D".repeat(2001)))];
		ctx.other_case_identifiers =
			vec![other_identifier(&"S".repeat(101), &"I".repeat(101))];
		ctx.linked_report_numbers = vec![linked_report_number("L".repeat(101))];
		ctx.studies = vec![study(Some("12"), Some(&"N".repeat(51)))];

		assert_eq!(
			filtered(&ctx, LENGTH_CODES),
			vec![
				issue(
					"ICH.C.1.1.LENGTH.MAX",
					"safetyReportIdentification.safetyReportId",
					true
				),
				issue(
					"ICH.C.1.10.r.LENGTH.MAX",
					"linkedReports.0.linkedReportNumber",
					true
				),
				issue(
					"ICH.C.1.11.1.LENGTH.MAX",
					"safetyReportIdentification.nullificationAmendmentCode",
					true
				),
				issue(
					"ICH.C.1.11.2.LENGTH.MAX",
					"safetyReportIdentification.nullificationReason",
					true
				),
				issue(
					"ICH.C.1.3.LENGTH.MAX",
					"safetyReportIdentification.reportType",
					true
				),
				issue(
					"ICH.C.1.6.1.r.1.LENGTH.MAX",
					"documentsHeldBySender.0.documentDescription",
					true
				),
				issue(
					"ICH.C.1.8.1.LENGTH.MAX",
					"safetyReportIdentification.worldwideUniqueId",
					true
				),
				issue(
					"ICH.C.1.8.2.LENGTH.MAX",
					"safetyReportIdentification.firstSenderType",
					true
				),
				issue(
					"ICH.C.1.9.1.r.1.LENGTH.MAX",
					"otherCaseIdentifiers.0.source",
					true
				),
				issue(
					"ICH.C.1.9.1.r.2.LENGTH.MAX",
					"otherCaseIdentifiers.0.caseIdentifier",
					true
				),
				issue(
					"ICH.C.5.3.LENGTH.MAX",
					"studyInformation.0.sponsorStudyNumber",
					true
				),
				issue(
					"ICH.C.5.4.LENGTH.MAX",
					"studyInformation.0.studyTypeReaction",
					true
				),
			]
		);
	}

	#[test]
	fn max_length_rules_cover_c2_c3_and_study_name_fields() {
		let mut source = primary_source();
		source.reporter_title = Some("T".repeat(51));
		source.reporter_given_name = Some("G".repeat(61));
		source.reporter_middle_name = Some("M".repeat(61));
		source.reporter_family_name = Some("F".repeat(61));
		source.organization = Some("O".repeat(61));
		source.department = Some("D".repeat(61));
		source.street = Some("S".repeat(101));
		source.city = Some("C".repeat(36));
		source.state = Some("S".repeat(41));
		source.postcode = Some("P".repeat(16));
		source.telephone = Some("T".repeat(34));
		source.country_code = Some("USA".to_string());
		source.qualification = Some("12".to_string());
		source.primary_source_regulatory = Some("12".to_string());

		let mut sender = sender();
		sender.sender_type = Some("12".to_string());
		sender.organization_name = Some("O".repeat(101));
		sender.department = Some("D".repeat(61));
		sender.person_title = Some("T".repeat(51));
		sender.person_given_name = Some("G".repeat(61));
		sender.person_middle_name = Some("M".repeat(61));
		sender.person_family_name = Some("F".repeat(61));
		sender.street_address = Some("S".repeat(101));
		sender.city = Some("C".repeat(36));
		sender.state = Some("S".repeat(41));
		sender.postcode = Some("P".repeat(16));
		sender.country_code = Some("USA".to_string());
		sender.telephone = Some("T".repeat(34));
		sender.fax = Some("F".repeat(34));
		sender.email = Some("E".repeat(101));

		let mut study = study(Some("1"), Some("SPONSOR-1"));
		study.study_name = Some("S".repeat(2001));

		let mut ctx = ctx_with(base_report());
		ctx.primary_sources = vec![source];
		ctx.sender = Some(sender);
		ctx.studies = vec![study];

		assert_eq!(
			filtered(&ctx, C23_LENGTH_CODES),
			vec![
				issue(
					"ICH.C.2.r.1.1.LENGTH.MAX",
					"primarySources.0.reporterTitle",
					true,
				),
				issue(
					"ICH.C.2.r.1.2.LENGTH.MAX",
					"primarySources.0.reporterGivenName",
					true,
				),
				issue(
					"ICH.C.2.r.1.3.LENGTH.MAX",
					"primarySources.0.reporterMiddleName",
					true,
				),
				issue(
					"ICH.C.2.r.1.4.LENGTH.MAX",
					"primarySources.0.reporterFamilyName",
					true,
				),
				issue(
					"ICH.C.2.r.2.1.LENGTH.MAX",
					"primarySources.0.reporterOrganization",
					true,
				),
				issue(
					"ICH.C.2.r.2.2.LENGTH.MAX",
					"primarySources.0.reporterDepartment",
					true,
				),
				issue(
					"ICH.C.2.r.2.3.LENGTH.MAX",
					"primarySources.0.reporterStreet",
					true,
				),
				issue(
					"ICH.C.2.r.2.4.LENGTH.MAX",
					"primarySources.0.reporterCity",
					true,
				),
				issue(
					"ICH.C.2.r.2.5.LENGTH.MAX",
					"primarySources.0.reporterState",
					true,
				),
				issue(
					"ICH.C.2.r.2.6.LENGTH.MAX",
					"primarySources.0.reporterPostcode",
					true,
				),
				issue(
					"ICH.C.2.r.2.7.LENGTH.MAX",
					"primarySources.0.reporterTelephone",
					true,
				),
				issue(
					"ICH.C.2.r.3.LENGTH.MAX",
					"primarySources.0.reporterCountry",
					true,
				),
				issue(
					"ICH.C.2.r.4.LENGTH.MAX",
					"primarySources.0.qualification",
					true,
				),
				issue(
					"ICH.C.2.r.5.LENGTH.MAX",
					"primarySources.0.primarySourceForRegulatoryPurposes",
					true,
				),
				issue("ICH.C.3.1.LENGTH.MAX", "senderInformation.senderType", true,),
				issue(
					"ICH.C.3.2.LENGTH.MAX",
					"senderInformation.organizationName",
					true,
				),
				issue(
					"ICH.C.3.3.1.LENGTH.MAX",
					"senderInformation.department",
					true
				),
				issue(
					"ICH.C.3.3.2.LENGTH.MAX",
					"senderInformation.personTitle",
					true
				),
				issue(
					"ICH.C.3.3.3.LENGTH.MAX",
					"senderInformation.personGivenName",
					true,
				),
				issue(
					"ICH.C.3.3.4.LENGTH.MAX",
					"senderInformation.personMiddleName",
					true,
				),
				issue(
					"ICH.C.3.3.5.LENGTH.MAX",
					"senderInformation.personFamilyName",
					true,
				),
				issue(
					"ICH.C.3.4.1.LENGTH.MAX",
					"senderInformation.streetAddress",
					true,
				),
				issue("ICH.C.3.4.2.LENGTH.MAX", "senderInformation.city", true),
				issue("ICH.C.3.4.3.LENGTH.MAX", "senderInformation.state", true),
				issue("ICH.C.3.4.4.LENGTH.MAX", "senderInformation.postcode", true),
				issue(
					"ICH.C.3.4.5.LENGTH.MAX",
					"senderInformation.countryCode",
					true
				),
				issue(
					"ICH.C.3.4.6.LENGTH.MAX",
					"senderInformation.telephone",
					true
				),
				issue("ICH.C.3.4.7.LENGTH.MAX", "senderInformation.fax", true),
				issue("ICH.C.3.4.8.LENGTH.MAX", "senderInformation.email", true),
				issue("ICH.C.5.2.LENGTH.MAX", "studyInformation.0.studyName", true),
			],
		);
	}

	#[test]
	fn max_length_rules_cover_literature_and_study_registration_fields() {
		let study_id = Uuid::nil();
		let mut study = study(Some("1"), Some("SPONSOR-1"));
		study.id = study_id;

		let mut ctx = ctx_with(base_report());
		ctx.literature_references = vec![literature_reference("R".repeat(501))];
		ctx.studies = vec![study];
		ctx.study_registrations = vec![study_registration(
			study_id,
			"N".repeat(51),
			Some("USA".to_string()),
			1,
		)];

		assert_eq!(
			filtered(&ctx, C45_LENGTH_CODES),
			vec![
				issue(
					"ICH.C.4.r.1.LENGTH.MAX",
					"literatureReferences.0.referenceText",
					true,
				),
				issue(
					"ICH.C.5.1.r.1.LENGTH.MAX",
					"studyInformation.0.registrations.0.registrationNumber",
					true,
				),
				issue(
					"ICH.C.5.1.r.2.LENGTH.MAX",
					"studyInformation.0.registrations.0.registrationCountry",
					true,
				),
			],
		);
	}
}
