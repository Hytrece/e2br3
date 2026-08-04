mod c_reporter_policy;
mod c_safety_report_policy;
pub mod case;
mod context;
mod d_patient_policy;
mod e_reaction_policy;
mod f_test_result_policy;
mod fda_context;
mod g_drug_policy;
mod h_narrative_policy;
mod mfds_context;
pub use c_reporter_policy::has_any_primary_source_content;
pub use c_safety_report_policy::{
	has_report_type, should_clear_combination_product_null_flavor_on_value,
	should_clear_local_criteria_null_flavor_on_value,
	should_require_fda_local_criteria_report_type,
	should_warn_fda_combination_product_indicator_missing,
};
pub use case::{validate_case_for_authorities, validate_case_for_authority};
pub use context::{
	load_base_validation_context, ValidationContext, VocabularyScope,
};
pub use d_patient_policy::{
	has_fda_ethnicity, has_fda_race, has_patient_initials, has_patient_payload,
	should_require_fda_ethnicity, should_require_fda_race,
	should_require_patient_initials,
};
pub use e_reaction_policy::{
	normalize_outcome_code, outcome_display_name,
	should_case_validation_require_required_intervention,
	should_emit_required_intervention_null_flavor_ni,
};
pub use f_test_result_policy::{has_test_name, has_test_payload};
pub use fda_context::{
	list_fda_devices, list_study_registrations, load_fda_validation_context,
	FdaValidationContext,
};
pub use g_drug_policy::{
	drug_characterization_display_name, has_drug_characterization,
	has_medicinal_product, normalize_drug_characterization,
};
pub use h_narrative_policy::{
	has_case_narrative, has_narrative_payload, should_require_case_narrative,
};
pub use lib_core::regulatory::*;
pub use lib_core::validation_report::{
	CaseValidationReport, ValidationIssue, ValidationSectionSummary,
	ValidationSubsectionSummary,
};
pub use mfds_context::{
	load_mfds_validation_context, MfdsValidationContext, ParentPastDrugByCase,
	PastDrugByCase, RelatednessWithDrug,
};
use sqlx::types::Uuid;
use std::collections::BTreeMap;

pub fn has_text(value: Option<&str>) -> bool {
	value.map(|v| !v.trim().is_empty()).unwrap_or(false)
}

fn push_direct_business_issue(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: impl Into<String>,
	message: impl Into<String>,
	blocking: bool,
) {
	let path = path.into();
	let section = case::sections::resolve_validation_section(code, Some(&path));
	push_field_issue(issues, code, path, section, message, blocking);
}

pub(crate) fn push_field_issue(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: impl Into<String>,
	section: impl Into<String>,
	message: impl Into<String>,
	blocking: bool,
) {
	let path = path.into();
	let field_path = case::sections::resolve_validation_field_path(Some(&path));
	let subsection =
		case::sections::resolve_validation_subsection(code, Some(&path));
	issues.push(ValidationIssue {
		code: code.to_string(),
		message: message.into(),
		field_path,
		path,
		section: section.into(),
		subsection,
		blocking,
	});
}

pub(crate) fn push_business_issue(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: impl Into<String>,
	message: impl Into<String>,
) {
	push_direct_business_issue(issues, code, path, message, true);
}

pub(crate) fn push_business_warning(
	issues: &mut Vec<ValidationIssue>,
	code: &str,
	path: impl Into<String>,
	message: impl Into<String>,
) {
	push_direct_business_issue(issues, code, path, message, false);
}

pub fn build_report(
	authority: RegulatoryAuthority,
	case_id: Uuid,
	issues: Vec<ValidationIssue>,
) -> CaseValidationReport {
	let blocking_count = issues.iter().filter(|issue| issue.blocking).count();
	let non_blocking_count = issues.len().saturating_sub(blocking_count);
	let mut by_section: BTreeMap<String, (usize, usize)> = BTreeMap::new();
	let mut by_subsection: BTreeMap<(String, String), (usize, usize)> =
		BTreeMap::new();
	for issue in &issues {
		let section_counts = by_section.entry(issue.section.clone()).or_default();
		let subsection_counts = by_subsection
			.entry((issue.section.clone(), issue.subsection.clone()))
			.or_default();
		if issue.blocking {
			section_counts.0 += 1;
			subsection_counts.0 += 1;
		} else {
			section_counts.1 += 1;
			subsection_counts.1 += 1;
		}
	}
	let section_summaries = by_section
		.into_iter()
		.map(|(section, (blocking_count, non_blocking_count))| {
			ValidationSectionSummary {
				section,
				blocking_count,
				non_blocking_count,
			}
		})
		.collect();
	let subsection_summaries = by_subsection
		.into_iter()
		.map(
			|((section, subsection), (blocking_count, non_blocking_count))| {
				ValidationSubsectionSummary {
					section,
					subsection,
					blocking_count,
					non_blocking_count,
				}
			},
		)
		.collect();
	let authority = authority.as_str().to_string();
	CaseValidationReport {
		authority,
		case_id,
		ok: blocking_count == 0,
		blocking_count,
		non_blocking_count,
		section_summaries,
		subsection_summaries,
		issues,
	}
}

#[cfg(test)]
mod direct_business_issue_tests {
	use super::*;

	#[test]
	fn direct_business_issue_is_blocking_without_catalog_metadata() {
		let mut issues = Vec::new();
		push_business_issue(
			&mut issues,
			"FDA.R0011",
			"safetyReportIdentification.safetyReportId",
			"invalid identifier profile",
		);

		assert_eq!(issues.len(), 1);
		assert!(issues[0].blocking);
		assert_eq!(issues[0].section, "C");
		assert_eq!(issues[0].subsection, "C.1");
		assert_eq!(issues[0].message, "invalid identifier profile");
	}
}
