use super::helpers::{
	validate_constraint, validate_length, validate_meddra, validate_value,
	validate_violation, RuleValue,
};
use crate::allowed_value::ConstraintValue;
use crate::{
	has_text, should_require_case_narrative, RegulatoryAuthority, RuleFacts,
	ValidationContext, ValidationIssue,
};
use lib_core::model::narrative::{
	CaseSummaryInformation, NarrativeInformation, SenderDiagnosis,
};
use std::borrow::Cow;

/// ICH.H.1.REQUIRED
/// ICH.H.1.LENGTH.MAX
fn h_1(narrative: Option<&NarrativeInformation>, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "narrative.caseNarrative";
	let value = match narrative {
		Some(narrative) if should_require_case_narrative(narrative) => {
			Some(narrative.case_narrative.as_str())
		}
		Some(_) => Some("present"),
		None => None,
	};
	validate_value(
		issues,
		"ICH.H.1.REQUIRED",
		PATH,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.H.1.LENGTH.MAX",
		PATH,
		narrative.map(|narrative| narrative.case_narrative.as_str()),
	);
}

/// ICH.H.2.LENGTH.MAX
fn h_2(narrative: &NarrativeInformation, issues: &mut Vec<ValidationIssue>) {
	validate_length(
		issues,
		"ICH.H.2.LENGTH.MAX",
		"narrative.reporterComments",
		narrative.reporter_comments.as_deref(),
	);
}

/// ICH.H.3.r.1a.REQUIRED
/// ICH.H.3.r.1a.LENGTH.MAX
fn h_3_r_1a(
	idx: usize,
	diagnosis: &SenderDiagnosis,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("narrative.senderDiagnoses.{idx}.diagnosisMeddraVersion");
	validate_violation(
		issues,
		"ICH.H.3.r.1a.REQUIRED",
		&path,
		has_text(diagnosis.diagnosis_meddra_code.as_deref())
			&& !has_text(diagnosis.diagnosis_meddra_version.as_deref()),
	);
	validate_length(
		issues,
		"ICH.H.3.r.1a.LENGTH.MAX",
		&path,
		diagnosis.diagnosis_meddra_version.as_deref(),
	);
}

/// ICH.H.3.r.1b.REQUIRED
/// ICH.H.3.r.1b.LENGTH.MAX
fn h_3_r_1b(
	idx: usize,
	diagnosis: &SenderDiagnosis,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("narrative.senderDiagnoses.{idx}.diagnosisMeddraCode");
	validate_violation(
		issues,
		"ICH.H.3.r.1b.REQUIRED",
		&path,
		has_text(diagnosis.diagnosis_meddra_version.as_deref())
			&& !has_text(diagnosis.diagnosis_meddra_code.as_deref()),
	);
	validate_length(
		issues,
		"ICH.H.3.r.1b.LENGTH.MAX",
		&path,
		diagnosis.diagnosis_meddra_code.as_deref(),
	);
}

/// ICH.H.3.r.1a.ALLOWED.VALUE
/// ICH.H.3.r.1a.VOCABULARY
/// ICH.H.3.r.1b.ALLOWED.VALUE
/// ICH.H.3.r.1b.VOCABULARY
fn h_3_r_1(
	idx: usize,
	diagnosis: &SenderDiagnosis,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_meddra(
		issues,
		vocabulary,
		"ICH.H.3.r.1a.ALLOWED.VALUE",
		"ICH.H.3.r.1b.ALLOWED.VALUE",
		"ICH.H.3.r.1a.VOCABULARY",
		"ICH.H.3.r.1b.VOCABULARY",
		format!("narrative.senderDiagnoses.{idx}.diagnosisMeddraVersion"),
		format!("narrative.senderDiagnoses.{idx}.diagnosisMeddraCode"),
		diagnosis.diagnosis_meddra_version.as_deref(),
		diagnosis.diagnosis_meddra_code.as_deref(),
	);
}

/// ICH.H.4.LENGTH.MAX
fn h_4(narrative: &NarrativeInformation, issues: &mut Vec<ValidationIssue>) {
	validate_length(
		issues,
		"ICH.H.4.LENGTH.MAX",
		"narrative.senderComments",
		narrative.sender_comments.as_deref(),
	);
}

/// ICH.H.5.r.1a.LENGTH.MAX
fn h_5_r_1a(
	idx: usize,
	summary: &CaseSummaryInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("narrative.caseSummaries.{idx}.summaryText");
	validate_length(
		issues,
		"ICH.H.5.r.1a.LENGTH.MAX",
		&path,
		summary.summary_text.as_deref(),
	);
}

/// ICH.H.5.r.1b.REQUIRED
/// ICH.H.5.r.1b.LENGTH.MAX
/// ICH.H.5.r.1b.ALLOWED.VALUE
fn h_5_r_1b(
	idx: usize,
	summary: &CaseSummaryInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("narrative.caseSummaries.{idx}.languageCode");
	validate_violation(
		issues,
		"ICH.H.5.r.1b.REQUIRED",
		&path,
		has_text(summary.summary_text.as_deref())
			&& !has_text(summary.language_code.as_deref()),
	);
	validate_length(
		issues,
		"ICH.H.5.r.1b.LENGTH.MAX",
		&path,
		summary.language_code.as_deref(),
	);
	validate_constraint(
		issues,
		"ICH.H.5.r.1b.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(summary.language_code.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
}

pub(crate) fn collect(
	issues: &mut Vec<ValidationIssue>,
	authority: RegulatoryAuthority,
	validation_ctx: &ValidationContext,
) {
	let _ = authority;
	collect_ich_issues(validation_ctx, issues);
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	h_1(validation_ctx.narrative.as_ref(), issues);
	if let Some(narrative) = validation_ctx.narrative.as_ref() {
		h_2(narrative, issues);
		h_4(narrative, issues);
	}
	for (idx, diagnosis) in validation_ctx.sender_diagnoses.iter().enumerate() {
		h_3_r_1a(idx, diagnosis, issues);
		h_3_r_1b(idx, diagnosis, issues);
		h_3_r_1(idx, diagnosis, &validation_ctx.vocabulary, issues);
	}
	for (idx, summary) in validation_ctx.case_summaries.iter().enumerate() {
		h_5_r_1a(idx, summary, issues);
		h_5_r_1b(idx, summary, &validation_ctx.vocabulary, issues);
	}
}

#[cfg(test)]
pub(super) fn constraint_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.H.3.r.1a.ALLOWED.VALUE",
		"ICH.H.3.r.1b.ALLOWED.VALUE",
		"ICH.H.5.r.1b.ALLOWED.VALUE",
	]
}

#[cfg(test)]
pub(super) fn implemented_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.H.1.REQUIRED",
		"ICH.H.1.LENGTH.MAX",
		"ICH.H.2.LENGTH.MAX",
		"ICH.H.3.r.1a.REQUIRED",
		"ICH.H.3.r.1a.LENGTH.MAX",
		"ICH.H.3.r.1a.ALLOWED.VALUE",
		"ICH.H.3.r.1a.VOCABULARY",
		"ICH.H.3.r.1b.REQUIRED",
		"ICH.H.3.r.1b.LENGTH.MAX",
		"ICH.H.3.r.1b.ALLOWED.VALUE",
		"ICH.H.3.r.1b.VOCABULARY",
		"ICH.H.4.LENGTH.MAX",
		"ICH.H.5.r.1a.LENGTH.MAX",
		"ICH.H.5.r.1b.REQUIRED",
		"ICH.H.5.r.1b.LENGTH.MAX",
		"ICH.H.5.r.1b.ALLOWED.VALUE",
	]
}

#[cfg(test)]
mod tests {
	use super::*;
	use lib_core::model::case::Case;
	use lib_core::model::narrative::{
		CaseSummaryInformation, NarrativeInformation, SenderDiagnosis,
	};
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Uuid;

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

	fn narrative() -> NarrativeInformation {
		NarrativeInformation {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_narrative_presave_id: None,
			case_narrative: String::new(),
			reporter_comments: None,
			sender_comments: None,
			additional_information: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn sender_diagnosis() -> SenderDiagnosis {
		SenderDiagnosis {
			id: Uuid::nil(),
			narrative_id: Uuid::nil(),
			sequence_number: 1,
			deleted: false,
			diagnosis_meddra_version: None,
			diagnosis_meddra_code: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn case_summary() -> CaseSummaryInformation {
		CaseSummaryInformation {
			id: Uuid::nil(),
			narrative_id: Uuid::nil(),
			sequence_number: 1,
			deleted: false,
			language_code: None,
			summary_text: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn length_issue(code: &str, path: &str) -> (String, String) {
		(code.to_string(), path.to_string())
	}

	fn length_issues(ctx: &ValidationContext) -> Vec<(String, String)> {
		let mut issues = Vec::new();
		collect_ich_issues(ctx, &mut issues);
		let mut out = issues
			.into_iter()
			.filter(|issue| issue.code.contains(".LENGTH.MAX"))
			.map(|issue| (issue.code, issue.path))
			.collect::<Vec<_>>();
		out.sort();
		out
	}

	fn codes_for(ctx: &ValidationContext) -> Vec<String> {
		let mut issues = Vec::new();
		collect_ich_issues(ctx, &mut issues);
		issues.into_iter().map(|issue| issue.code).collect()
	}

	#[test]
	fn meddra_vocabulary_rules_cover_sender_diagnosis_codes() {
		let mut ctx = empty_ctx();
		ctx.vocabulary =
			crate::context::VocabularyContext::for_meddra(&[("26.1", "10000001")]);
		let mut diagnosis = sender_diagnosis();
		diagnosis.diagnosis_meddra_version = Some("99.9".to_string());
		diagnosis.diagnosis_meddra_code = Some("99999999".to_string());
		ctx.sender_diagnoses = vec![diagnosis];

		let codes = codes_for(&ctx);
		assert!(codes.contains(&"ICH.H.3.r.1a.VOCABULARY".to_string()));
		assert!(codes.contains(&"ICH.H.3.r.1b.VOCABULARY".to_string()));
	}

	#[test]
	fn max_length_rules_cover_h_narrative_text_fields() {
		let mut narrative = narrative();
		narrative.case_narrative = "N".repeat(100001);
		narrative.reporter_comments = Some("R".repeat(20001));
		narrative.sender_comments = Some("S".repeat(20001));
		let mut diagnosis = sender_diagnosis();
		diagnosis.diagnosis_meddra_version = Some("V".repeat(5));
		diagnosis.diagnosis_meddra_code = Some("C".repeat(9));
		let mut summary = case_summary();
		summary.summary_text = Some("T".repeat(100001));
		summary.language_code = Some("LANG".to_string());
		let mut ctx = empty_ctx();
		ctx.narrative = Some(narrative);
		ctx.sender_diagnoses = vec![diagnosis];
		ctx.case_summaries = vec![summary];

		assert_eq!(
			length_issues(&ctx),
			vec![
				length_issue("ICH.H.1.LENGTH.MAX", "narrative.caseNarrative"),
				length_issue("ICH.H.2.LENGTH.MAX", "narrative.reporterComments"),
				length_issue(
					"ICH.H.3.r.1a.LENGTH.MAX",
					"narrative.senderDiagnoses.0.diagnosisMeddraVersion"
				),
				length_issue(
					"ICH.H.3.r.1b.LENGTH.MAX",
					"narrative.senderDiagnoses.0.diagnosisMeddraCode"
				),
				length_issue("ICH.H.4.LENGTH.MAX", "narrative.senderComments"),
				length_issue(
					"ICH.H.5.r.1a.LENGTH.MAX",
					"narrative.caseSummaries.0.summaryText"
				),
				length_issue(
					"ICH.H.5.r.1b.LENGTH.MAX",
					"narrative.caseSummaries.0.languageCode"
				),
			]
		);
	}
}
