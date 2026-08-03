use super::helpers::{
	validate_constraint, validate_future_date, validate_length, validate_meddra,
	validate_value, validate_violation, DateValues, RuleValue,
};
use crate::allowed_value::{true_marker_value, ConstraintValue};
use crate::{
	has_text, should_case_validation_require_required_intervention,
	FdaValidationContext, RegulatoryAuthority, RuleFacts, ValidationContext,
	ValidationIssue,
};
use lib_core::model::reaction::Reaction;
use sqlx::types::Decimal;
use std::borrow::Cow;

fn decimal_text(value: Option<Decimal>) -> Option<String> {
	value.map(|value| value.to_string())
}

fn bool_text(value: Option<bool>) -> Option<&'static str> {
	value.map(|value| if value { "true" } else { "false" })
}

/// ICH.E.i.1.1a.REQUIRED
/// ICH.E.i.1.1a.LENGTH.MAX
fn e_i_1_1a(
	idx: usize,
	reaction: Option<&Reaction>,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("reactions.{idx}.primarySourceReaction");
	let value = reaction.map(|reaction| reaction.primary_source_reaction.as_str());
	validate_value(
		issues,
		"ICH.E.i.1.1a.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
	validate_length(issues, "ICH.E.i.1.1a.LENGTH.MAX", &path, value);
}

/// ICH.E.i.1.1b.REQUIRED
/// ICH.E.i.1.1b.ALLOWED.VALUE
/// ICH.E.i.1.1b.LENGTH.MAX
fn e_i_1_1b(
	idx: usize,
	reaction: &Reaction,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("reactions.{idx}.reactionLanguage");
	validate_violation(
		issues,
		"ICH.E.i.1.1b.REQUIRED",
		&path,
		has_text(Some(reaction.primary_source_reaction.as_str()))
			&& !has_text(reaction.reaction_language.as_deref()),
	);
	validate_constraint(
		issues,
		"ICH.E.i.1.1b.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(
			reaction.reaction_language.as_deref().map(Cow::Borrowed),
		),
		&validation_ctx.vocabulary,
	);
	validate_length(
		issues,
		"ICH.E.i.1.1b.LENGTH.MAX",
		&path,
		reaction.reaction_language.as_deref(),
	);
}

/// ICH.E.i.1.2.LENGTH.MAX
fn e_i_1_2(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	validate_length(
		issues,
		"ICH.E.i.1.2.LENGTH.MAX",
		&format!("reactions.{idx}.primarySourceReactionTranslation"),
		reaction.primary_source_reaction_translation.as_deref(),
	);
}

/// ICH.E.i.2.1a.REQUIRED
/// ICH.E.i.2.1a.LENGTH.MAX
fn e_i_2_1a(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	let path = format!("reactions.{idx}.reactionMeddraVersion");
	validate_violation(
		issues,
		"ICH.E.i.2.1a.REQUIRED",
		&path,
		has_text(reaction.reaction_meddra_code.as_deref())
			&& !has_text(reaction.reaction_meddra_version.as_deref()),
	);
	validate_length(
		issues,
		"ICH.E.i.2.1a.LENGTH.MAX",
		&path,
		reaction.reaction_meddra_version.as_deref(),
	);
}

/// ICH.E.i.2.1b.REQUIRED
/// ICH.E.i.2.1b.LENGTH.MAX
fn e_i_2_1b(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	let path = format!("reactions.{idx}.reactionMeddraCode");
	validate_violation(
		issues,
		"ICH.E.i.2.1b.REQUIRED",
		&path,
		!has_text(reaction.reaction_meddra_code.as_deref()),
	);
	validate_length(
		issues,
		"ICH.E.i.2.1b.LENGTH.MAX",
		&path,
		reaction.reaction_meddra_code.as_deref(),
	);
}

/// ICH.E.i.2.1a.ALLOWED.VALUE
/// ICH.E.i.2.1a.VOCABULARY
/// ICH.E.i.2.1b.ALLOWED.VALUE
/// ICH.E.i.2.1b.VOCABULARY
fn e_i_2_1_meddra(
	idx: usize,
	reaction: &Reaction,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_meddra(
		issues,
		&validation_ctx.vocabulary,
		"ICH.E.i.2.1a.ALLOWED.VALUE",
		"ICH.E.i.2.1b.ALLOWED.VALUE",
		"ICH.E.i.2.1a.VOCABULARY",
		"ICH.E.i.2.1b.VOCABULARY",
		format!("reactions.{idx}.reactionMeddraVersion"),
		format!("reactions.{idx}.reactionMeddraCode"),
		reaction.reaction_meddra_version.as_deref(),
		reaction.reaction_meddra_code.as_deref(),
	);
}

/// ICH.E.i.3.1.LENGTH.MAX
fn e_i_3_1(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	validate_length(
		issues,
		"ICH.E.i.3.1.LENGTH.MAX",
		&format!("reactions.{idx}.termHighlightedByReporter"),
		reaction.term_highlighted.as_deref(),
	);
}

fn e_i_3_2_marker(
	issues: &mut Vec<ValidationIssue>,
	required_code: &str,
	allowed_code: &str,
	path: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
	validation_ctx: &ValidationContext,
) {
	validate_value(
		issues,
		required_code,
		path,
		RuleValue::borrowed(bool_text(value), null_flavor),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		allowed_code,
		path,
		true_marker_value(value, null_flavor),
		&validation_ctx.vocabulary,
	);
}

/// ICH.E.i.3.2.CRITERIA.REQUIRED
/// ICH.E.i.3.2.NI.ONLY
fn e_i_3_2(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	let any_criteria_true = [
		reaction.criteria_death,
		reaction.criteria_life_threatening,
		reaction.criteria_hospitalization,
		reaction.criteria_disabling,
		reaction.criteria_congenital_anomaly,
		reaction.criteria_other_medically_important,
	]
	.into_iter()
	.flatten()
	.any(|value| value);
	let has_non_ni_null_flavor = [
		reaction.criteria_death_null_flavor.as_deref(),
		reaction.criteria_life_threatening_null_flavor.as_deref(),
		reaction.criteria_hospitalization_null_flavor.as_deref(),
		reaction.criteria_disabling_null_flavor.as_deref(),
		reaction.criteria_congenital_anomaly_null_flavor.as_deref(),
		reaction
			.criteria_other_medically_important_null_flavor
			.as_deref(),
	]
	.into_iter()
	.flatten()
	.any(|value| !value.trim().eq_ignore_ascii_case("NI"));
	let path = format!("reactions.{idx}.seriousnessCriteria");
	validate_violation(
		issues,
		"ICH.E.i.3.2.CRITERIA.REQUIRED",
		&path,
		reaction.serious == Some(true) && !any_criteria_true,
	);
	validate_violation(issues, "ICH.E.i.3.2.NI.ONLY", &path, has_non_ni_null_flavor);
}

macro_rules! reaction_marker_field {
	($name:ident, $suffix:literal, $path:literal, $value:ident, $null_flavor:ident) => {
		#[doc = concat!("ICH.E.i.3.2", $suffix, ".REQUIRED")]
		#[doc = concat!("ICH.E.i.3.2", $suffix, ".ALLOWED.VALUE")]
		fn $name(
			idx: usize,
			reaction: &Reaction,
			validation_ctx: &ValidationContext,
			issues: &mut Vec<ValidationIssue>,
		) {
			e_i_3_2_marker(
				issues,
				concat!("ICH.E.i.3.2", $suffix, ".REQUIRED"),
				concat!("ICH.E.i.3.2", $suffix, ".ALLOWED.VALUE"),
				&format!("reactions.{idx}.{}", $path),
				reaction.$value,
				reaction.$null_flavor.as_deref(),
				validation_ctx,
			);
		}
	};
}

reaction_marker_field!(
	e_i_3_2a,
	"a",
	"criteriaDeath",
	criteria_death,
	criteria_death_null_flavor
);
reaction_marker_field!(
	e_i_3_2b,
	"b",
	"criteriaLifeThreatening",
	criteria_life_threatening,
	criteria_life_threatening_null_flavor
);
reaction_marker_field!(
	e_i_3_2c,
	"c",
	"criteriaHospitalization",
	criteria_hospitalization,
	criteria_hospitalization_null_flavor
);
reaction_marker_field!(
	e_i_3_2d,
	"d",
	"criteriaDisabling",
	criteria_disabling,
	criteria_disabling_null_flavor
);
reaction_marker_field!(
	e_i_3_2e,
	"e",
	"criteriaCongenitalAnomaly",
	criteria_congenital_anomaly,
	criteria_congenital_anomaly_null_flavor
);
reaction_marker_field!(
	e_i_3_2f,
	"f",
	"criteriaOtherMedicallyImportant",
	criteria_other_medically_important,
	criteria_other_medically_important_null_flavor
);

/// ICH.E.i.4-5.FUTURE_DATE.FORBIDDEN
fn e_i_4_5(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	validate_future_date(
		issues,
		"ICH.E.i.4-5.FUTURE_DATE.FORBIDDEN",
		&format!("reactions.{idx}.reactionDateRange"),
		DateValues::Two(reaction.start_date, reaction.end_date),
	);
}

/// ICH.E.i.6a.REQUIRED
/// ICH.E.i.6a.LENGTH.MAX
fn e_i_6a(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	let path = format!("reactions.{idx}.durationValue");
	validate_violation(
		issues,
		"ICH.E.i.6a.REQUIRED",
		&path,
		has_text(reaction.duration_unit.as_deref())
			&& reaction.duration_value.is_none(),
	);
	let value = decimal_text(reaction.duration_value);
	validate_length(issues, "ICH.E.i.6a.LENGTH.MAX", &path, value.as_deref());
}

/// ICH.E.i.6b.REQUIRED
/// ICH.E.i.6b.LENGTH.MAX
fn e_i_6b(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	let path = format!("reactions.{idx}.durationUnit");
	validate_violation(
		issues,
		"ICH.E.i.6b.REQUIRED",
		&path,
		reaction.duration_value.is_some()
			&& !has_text(reaction.duration_unit.as_deref()),
	);
	validate_length(
		issues,
		"ICH.E.i.6b.LENGTH.MAX",
		&path,
		reaction.duration_unit.as_deref(),
	);
}

/// ICH.E.i.7.REQUIRED
/// ICH.E.i.7.ALLOWED.VALUE
/// ICH.E.i.7.LENGTH.MAX
fn e_i_7(
	idx: usize,
	reaction: Option<&Reaction>,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("reactions.{idx}.reactionOutcome");
	let value = reaction.and_then(|reaction| reaction.outcome.as_deref());
	validate_value(
		issues,
		"ICH.E.i.7.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
	if reaction.is_some() {
		validate_constraint(
			issues,
			"ICH.E.i.7.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(value.map(Cow::Borrowed)),
			&validation_ctx.vocabulary,
		);
		validate_length(issues, "ICH.E.i.7.LENGTH.MAX", &path, value);
	}
}

/// ICH.E.i.9.VOCABULARY
/// ICH.E.i.9.LENGTH.MAX
fn e_i_9(
	idx: usize,
	reaction: &Reaction,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("reactions.{idx}.reactionCountry");
	validate_constraint(
		issues,
		"ICH.E.i.9.VOCABULARY",
		&path,
		ConstraintValue::Text(reaction.country_code.as_deref().map(Cow::Borrowed)),
		&validation_ctx.vocabulary,
	);
	validate_length(
		issues,
		"ICH.E.i.9.LENGTH.MAX",
		&path,
		reaction.country_code.as_deref(),
	);
}

/// FDA.E.i.3.2h.REQUIRED
fn fda_e_i_3_2h(idx: usize, reaction: &Reaction, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"FDA.E.i.3.2h.REQUIRED",
		&format!("reactions.{idx}.requiredIntervention"),
		RuleValue::borrowed(bool_text(reaction.required_intervention), None),
		RuleFacts {
			fda_reaction_other_medically_important: reaction
				.criteria_other_medically_important,
			..RuleFacts::default()
		},
	);
}

pub(crate) fn collect(
	issues: &mut Vec<ValidationIssue>,
	authority: RegulatoryAuthority,
	validation_ctx: &ValidationContext,
	fda_ctx: Option<&FdaValidationContext>,
) {
	let _ = fda_ctx;
	collect_ich_issues(validation_ctx, issues);
	if authority == RegulatoryAuthority::Fda {
		collect_fda_issues(validation_ctx, issues);
	}
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	if validation_ctx.reactions.is_empty() {
		e_i_1_1a(0, None, issues);
		e_i_7(0, None, validation_ctx, issues);
		return;
	}
	for (idx, reaction) in validation_ctx.reactions.iter().enumerate() {
		e_i_1_1a(idx, Some(reaction), issues);
		e_i_1_1b(idx, reaction, validation_ctx, issues);
		e_i_1_2(idx, reaction, issues);
		e_i_2_1a(idx, reaction, issues);
		e_i_2_1b(idx, reaction, issues);
		e_i_2_1_meddra(idx, reaction, validation_ctx, issues);
		e_i_3_1(idx, reaction, issues);
		e_i_3_2(idx, reaction, issues);
		e_i_3_2a(idx, reaction, validation_ctx, issues);
		e_i_3_2b(idx, reaction, validation_ctx, issues);
		e_i_3_2c(idx, reaction, validation_ctx, issues);
		e_i_3_2d(idx, reaction, validation_ctx, issues);
		e_i_3_2e(idx, reaction, validation_ctx, issues);
		e_i_3_2f(idx, reaction, validation_ctx, issues);
		e_i_4_5(idx, reaction, issues);
		e_i_6a(idx, reaction, issues);
		e_i_6b(idx, reaction, issues);
		e_i_7(idx, Some(reaction), validation_ctx, issues);
		e_i_9(idx, reaction, validation_ctx, issues);
	}
}

pub(crate) fn collect_fda_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	if should_case_validation_require_required_intervention() {
		for (idx, reaction) in validation_ctx.reactions.iter().enumerate() {
			fda_e_i_3_2h(idx, reaction, issues);
		}
	}
}

#[cfg(test)]
pub(super) fn constraint_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.E.i.1.1b.ALLOWED.VALUE",
		"ICH.E.i.7.ALLOWED.VALUE",
		"ICH.E.i.9.VOCABULARY",
		"ICH.E.i.3.2a.ALLOWED.VALUE",
		"ICH.E.i.3.2b.ALLOWED.VALUE",
		"ICH.E.i.3.2c.ALLOWED.VALUE",
		"ICH.E.i.3.2d.ALLOWED.VALUE",
		"ICH.E.i.3.2e.ALLOWED.VALUE",
		"ICH.E.i.3.2f.ALLOWED.VALUE",
		"ICH.E.i.2.1a.ALLOWED.VALUE",
		"ICH.E.i.2.1b.ALLOWED.VALUE",
	]
}

#[cfg(test)]
pub(super) fn implemented_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.E.i.1.1a.REQUIRED",
		"ICH.E.i.7.REQUIRED",
		"ICH.E.i.3.2a.REQUIRED",
		"ICH.E.i.3.2b.REQUIRED",
		"ICH.E.i.3.2c.REQUIRED",
		"ICH.E.i.3.2d.REQUIRED",
		"ICH.E.i.3.2e.REQUIRED",
		"ICH.E.i.3.2f.REQUIRED",
		"ICH.E.i.4-5.FUTURE_DATE.FORBIDDEN",
		"ICH.E.i.1.1b.ALLOWED.VALUE",
		"ICH.E.i.7.ALLOWED.VALUE",
		"ICH.E.i.9.VOCABULARY",
		"ICH.E.i.3.2a.ALLOWED.VALUE",
		"ICH.E.i.3.2b.ALLOWED.VALUE",
		"ICH.E.i.3.2c.ALLOWED.VALUE",
		"ICH.E.i.3.2d.ALLOWED.VALUE",
		"ICH.E.i.3.2e.ALLOWED.VALUE",
		"ICH.E.i.3.2f.ALLOWED.VALUE",
		"ICH.E.i.1.1a.LENGTH.MAX",
		"ICH.E.i.1.1b.LENGTH.MAX",
		"ICH.E.i.1.2.LENGTH.MAX",
		"ICH.E.i.2.1a.LENGTH.MAX",
		"ICH.E.i.2.1b.LENGTH.MAX",
		"ICH.E.i.6b.LENGTH.MAX",
		"ICH.E.i.7.LENGTH.MAX",
		"ICH.E.i.9.LENGTH.MAX",
		"ICH.E.i.3.1.LENGTH.MAX",
		"ICH.E.i.6a.LENGTH.MAX",
		"ICH.E.i.2.1a.REQUIRED",
		"ICH.E.i.2.1b.REQUIRED",
		"ICH.E.i.6a.REQUIRED",
		"ICH.E.i.6b.REQUIRED",
		"ICH.E.i.1.1b.REQUIRED",
		"FDA.E.i.3.2h.REQUIRED",
		"ICH.E.i.3.2.CRITERIA.REQUIRED",
		"ICH.E.i.3.2.NI.ONLY",
		"ICH.E.i.2.1a.ALLOWED.VALUE",
		"ICH.E.i.2.1b.ALLOWED.VALUE",
		"ICH.E.i.2.1a.VOCABULARY",
		"ICH.E.i.2.1b.VOCABULARY",
	]
}

#[cfg(test)]
mod tests {
	use super::*;
	use lib_core::model::case::Case;
	use lib_core::model::reaction::Reaction;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::{Decimal, Uuid};

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

	fn empty_ctx() -> ValidationContext {
		ValidationContext {
			vocabulary: Default::default(),
			case: dummy_case(),
			safety_report: None,
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

	fn reaction() -> Reaction {
		Reaction {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			sequence_number: 1,
			primary_source_reaction: String::new(),
			primary_source_reaction_translation: None,
			reaction_language: None,
			reaction_meddra_version: None,
			reaction_meddra_code: None,
			term_highlighted: None,
			serious: None,
			criteria_death: None,
			criteria_death_null_flavor: None,
			criteria_life_threatening: None,
			criteria_life_threatening_null_flavor: None,
			criteria_hospitalization: None,
			criteria_hospitalization_null_flavor: None,
			criteria_disabling: None,
			criteria_disabling_null_flavor: None,
			criteria_congenital_anomaly: None,
			criteria_congenital_anomaly_null_flavor: None,
			criteria_other_medically_important: None,
			criteria_other_medically_important_null_flavor: None,
			required_intervention: None,
			required_intervention_null_flavor: None,
			expectedness: None,
			severity: None,
			mfds_device_ae_classification: None,
			mfds_device_ae_outcome: None,
			mfds_device_cause_medical_device: None,
			mfds_device_cause_procedure_issue: None,
			mfds_device_cause_patient_condition: None,
			mfds_device_cause_unable_to_assess: None,
			mfds_device_cause_other: None,
			mfds_device_action_reason: None,
			mfds_device_action_recall: None,
			mfds_device_action_repair: None,
			mfds_device_action_inspection: None,
			mfds_device_action_replacement: None,
			mfds_device_action_improvement: None,
			mfds_device_action_monitoring: None,
			mfds_device_action_notification: None,
			mfds_device_action_label_change: None,
			mfds_device_action_other: None,
			start_date: None,
			start_date_null_flavor: None,
			end_date: None,
			end_date_null_flavor: None,
			duration_value: None,
			duration_unit: None,
			outcome: None,
			medical_confirmation: None,
			country_code: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn length_issue(code: &str, path: &str) -> (String, String) {
		(code.to_string(), path.to_string())
	}

	fn length_issues_for(reaction: Reaction) -> Vec<(String, String)> {
		let mut ctx = empty_ctx();
		ctx.reactions = vec![reaction];
		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		let mut out = issues
			.into_iter()
			.filter(|issue| issue.code.contains(".LENGTH.MAX"))
			.map(|issue| (issue.code, issue.path))
			.collect::<Vec<_>>();
		out.sort();
		out
	}

	#[test]
	fn allowed_value_rule_flags_invalid_reaction_outcome() {
		let mut reaction = reaction();
		reaction.outcome = Some("9".to_string());
		let mut ctx = empty_ctx();
		ctx.reactions = vec![reaction];
		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);

		assert!(issues.iter().any(|issue| {
			issue.code == "ICH.E.i.7.ALLOWED.VALUE"
				&& issue.path == "reactions.0.reactionOutcome"
		}));
	}

	#[test]
	fn fda_reaction_rule_uses_catalog_condition_and_concrete_path() {
		let mut reaction = reaction();
		reaction.criteria_other_medically_important = Some(true);
		let mut issues = Vec::new();
		fda_e_i_3_2h(3, &reaction, &mut issues);
		assert_eq!(issues.len(), 1);
		assert_eq!(issues[0].code, "FDA.E.i.3.2h.REQUIRED");
		assert_eq!(
			issues[0].field_path.as_deref(),
			Some("reactions.3.requiredIntervention")
		);

		issues.clear();
		reaction.criteria_other_medically_important = Some(false);
		fda_e_i_3_2h(3, &reaction, &mut issues);
		assert!(issues.is_empty());
	}

	#[test]
	fn true_marker_rules_accept_absent_values_and_honor_null_flavor() {
		let mut reaction = reaction();
		reaction.criteria_death_null_flavor = Some("NI".to_string());
		reaction.criteria_life_threatening_null_flavor = Some("NI".to_string());
		reaction.criteria_hospitalization_null_flavor = Some("NI".to_string());
		reaction.criteria_disabling_null_flavor = Some("NI".to_string());
		reaction.criteria_congenital_anomaly_null_flavor = Some("NI".to_string());
		reaction.criteria_other_medically_important = Some(true);
		let mut ctx = empty_ctx();
		ctx.reactions = vec![reaction];
		let mut issues = Vec::new();

		collect_ich_issues(&ctx, &mut issues);

		let marker_issues = issues
			.iter()
			.filter(|issue| issue.code.starts_with("ICH.E.i.3.2"))
			.collect::<Vec<_>>();
		assert!(marker_issues.is_empty(), "{marker_issues:?}");
		assert!(!marker_issues
			.iter()
			.any(|issue| issue.code == "ICH.E.i.3.2a.ALLOWED.VALUE"));
	}

	#[test]
	fn max_length_rules_cover_e_reaction_text_fields() {
		let mut reaction = reaction();
		reaction.primary_source_reaction = "R".repeat(251);
		reaction.reaction_language = Some("LANG".to_string());
		reaction.primary_source_reaction_translation = Some("T".repeat(251));
		reaction.reaction_meddra_version = Some("V".repeat(5));
		reaction.reaction_meddra_code = Some("M".repeat(9));
		reaction.duration_value = Some(Decimal::new(123456, 0));
		reaction.duration_unit = Some("U".repeat(51));
		reaction.outcome = Some("OC".to_string());
		reaction.country_code = Some("USA".to_string());

		assert_eq!(
			length_issues_for(reaction),
			vec![
				length_issue(
					"ICH.E.i.1.1a.LENGTH.MAX",
					"reactions.0.primarySourceReaction"
				),
				length_issue(
					"ICH.E.i.1.1b.LENGTH.MAX",
					"reactions.0.reactionLanguage"
				),
				length_issue(
					"ICH.E.i.1.2.LENGTH.MAX",
					"reactions.0.primarySourceReactionTranslation"
				),
				length_issue(
					"ICH.E.i.2.1a.LENGTH.MAX",
					"reactions.0.reactionMeddraVersion"
				),
				length_issue(
					"ICH.E.i.2.1b.LENGTH.MAX",
					"reactions.0.reactionMeddraCode"
				),
				length_issue("ICH.E.i.6a.LENGTH.MAX", "reactions.0.durationValue"),
				length_issue("ICH.E.i.6b.LENGTH.MAX", "reactions.0.durationUnit"),
				length_issue("ICH.E.i.7.LENGTH.MAX", "reactions.0.reactionOutcome"),
				length_issue("ICH.E.i.9.LENGTH.MAX", "reactions.0.reactionCountry"),
			]
		);
	}
}
