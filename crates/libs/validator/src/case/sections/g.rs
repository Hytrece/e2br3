use super::helpers::{
	validate_constraint, validate_future_date, validate_length, validate_meddra,
	validate_value, validate_violation, validate_vocabulary_variant, DateValues,
	RuleValue,
};
use crate::allowed_value::{true_marker_value, ConstraintValue};
use crate::{
	has_text, is_fda_ind_message_receiver, is_fda_postmarket_batch_receiver,
	is_fda_premarket_message_receiver, is_mfds_clinical_trial_receiver,
	is_mfds_compassionate_use_receiver, is_mfds_domestic_receiver,
	is_mfds_foreign_postmarket_receiver, list_fda_devices, FdaValidationContext,
	MfdsValidationContext, RegulatoryAuthority, RuleFacts, ValidationContext,
	ValidationIssue,
};
use lib_core::ctx::Ctx;
use lib_core::model::drug::{
	parse_drug_additional_info_codes_json, DosageInformation, DrugActiveSubstance,
	DrugIndication, DrugInformation,
};
use lib_core::model::drug_reaction_assessment::{
	DrugReactionAssessment, RelatednessAssessment,
};
use lib_core::model::{ModelManager, Result};
use sqlx::types::Decimal;
use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

fn decimal_text(value: Option<Decimal>) -> Option<String> {
	value.map(|value| value.to_string())
}

fn resolve_drug_child_indices(
	drug_indices: &HashMap<sqlx::types::Uuid, usize>,
	drug_id: sqlx::types::Uuid,
	sequence_number: i32,
) -> Option<(usize, usize)> {
	let drug_index = drug_indices.get(&drug_id).copied()?;
	let child_index = sequence_number
		.checked_sub(1)
		.and_then(|value| usize::try_from(value).ok())?;
	Some((drug_index, child_index))
}

fn sequence_idx(sequence_number: i32, fallback: usize) -> usize {
	sequence_number
		.checked_sub(1)
		.and_then(|value| usize::try_from(value).ok())
		.unwrap_or(fallback)
}

fn additional_info_codes(drug: &DrugInformation) -> Vec<String> {
	parse_drug_additional_info_codes_json(
		drug.drug_additional_info_codes_json.as_ref(),
	)
	.into_iter()
	.filter_map(|entry| entry.value_code)
	.collect()
}

/// ICH.G.k.1.REQUIRED
/// ICH.G.k.1.ALLOWED.VALUE
/// ICH.G.k.1.LENGTH.MAX
fn g_k_1(
	drugs: &[DrugInformation],
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"ICH.G.k.1.REQUIRED",
		"drugs.0.drugCharacterization",
		RuleValue::borrowed((!drugs.is_empty()).then_some("present"), None),
		RuleFacts::default(),
	);
	for (idx, drug) in drugs.iter().enumerate() {
		let path = format!("drugs.{idx}.drugCharacterization");
		validate_value(
			issues,
			"ICH.G.k.1.REQUIRED",
			&path,
			RuleValue::borrowed(Some(drug.drug_characterization.as_str()), None),
			RuleFacts::default(),
		);
		validate_constraint(
			issues,
			"ICH.G.k.1.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(Some(Cow::Borrowed(
				drug.drug_characterization.as_str(),
			))),
			vocabulary,
		);
		validate_length(
			issues,
			"ICH.G.k.1.LENGTH.MAX",
			&path,
			Some(drug.drug_characterization.as_str()),
		);
	}
	if !drugs.is_empty()
		&& !drugs
			.iter()
			.any(|drug| matches!(drug.drug_characterization.trim(), "1" | "3" | "4"))
	{
		crate::push_business_issue(
			issues,
			"ICH.G.k.1.AGGREGATE.REQUIRED",
			"drugs.0.drugCharacterization",
			"At least one drug must be Suspect, Interacting, or Drug Not Administered.",
		);
	}
}

/// ICH.G.k.2.1.1a.LENGTH.MAX
/// ICH.G.k.2.1.1a.REQUIRED
fn g_k_2_1_1a(
	idx: usize,
	drug: &DrugInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.mpidVersion");
	validate_length(
		issues,
		"ICH.G.k.2.1.1a.LENGTH.MAX",
		&path,
		drug.mpid_version.as_deref(),
	);
	validate_violation(
		issues,
		"ICH.G.k.2.1.1a.REQUIRED",
		&path,
		has_text(drug.mpid.as_deref()) && !has_text(drug.mpid_version.as_deref()),
	);
}

/// ICH.G.k.2.1.1b.ALLOWED.VALUE
/// ICH.G.k.2.1.1b.LENGTH.MAX
fn g_k_2_1_1b(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.mpid");
	validate_constraint(
		issues,
		"ICH.G.k.2.1.1b.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(drug.mpid.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.2.1.1b.LENGTH.MAX",
		&path,
		drug.mpid.as_deref(),
	);
}

/// ICH.G.k.2.1.2a.LENGTH.MAX
fn g_k_2_1_2a(
	idx: usize,
	drug: &DrugInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.phpidVersion");
	validate_length(
		issues,
		"ICH.G.k.2.1.2a.LENGTH.MAX",
		&path,
		drug.phpid_version.as_deref(),
	);
}

/// ICH.G.k.2.1.2b.ALLOWED.VALUE
/// ICH.G.k.2.1.2b.LENGTH.MAX
fn g_k_2_1_2b(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.phpid");
	validate_constraint(
		issues,
		"ICH.G.k.2.1.2b.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(drug.phpid.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.2.1.2b.LENGTH.MAX",
		&path,
		drug.phpid.as_deref(),
	);
	if has_text(drug.mpid.as_deref()) && has_text(drug.phpid.as_deref()) {
		crate::push_business_issue(
			issues,
			"ICH.G.k.2.1.MPID_PHPID.EXCLUSIVE",
			&path,
			"A drug may contain MPID or PhPID, but not both.",
		);
	}
}

/// ICH.G.k.2.2.REQUIRED
/// ICH.G.k.2.2.LENGTH.MAX
fn g_k_2_2(drugs: &[DrugInformation], issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.G.k.2.2.REQUIRED",
		"drugs.0.medicinalProduct",
		RuleValue::borrowed((!drugs.is_empty()).then_some("present"), None),
		RuleFacts::default(),
	);
	for (idx, drug) in drugs.iter().enumerate() {
		let path = format!("drugs.{idx}.medicinalProduct");
		let value = Some(drug.medicinal_product.as_str());
		validate_value(
			issues,
			"ICH.G.k.2.2.REQUIRED",
			&path,
			RuleValue::borrowed(value, None),
			RuleFacts::default(),
		);
		validate_length(issues, "ICH.G.k.2.2.LENGTH.MAX", &path, value);
	}
}

/// ICH.G.k.2.4.VOCABULARY
/// ICH.G.k.2.4.LENGTH.MAX
fn g_k_2_4(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.obtainDrugCountry");
	validate_constraint(
		issues,
		"ICH.G.k.2.4.VOCABULARY",
		&path,
		ConstraintValue::Text(
			drug.obtain_drug_country.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.2.4.LENGTH.MAX",
		&path,
		drug.obtain_drug_country.as_deref(),
	);
}

/// ICH.G.k.2.5.ALLOWED.VALUE
fn g_k_2_5(
	idx: usize,
	drug: &DrugInformation,
	report_type_is_study: bool,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.investigationalProductBlinded");
	validate_constraint(
		issues,
		"ICH.G.k.2.5.ALLOWED.VALUE",
		&path,
		true_marker_value(drug.investigational_product_blinded, None),
		vocabulary,
	);
	if drug.investigational_product_blinded.is_some() && !report_type_is_study {
		crate::push_business_issue(
			issues,
			"ICH.G.k.2.5.STUDY.ONLY",
			&path,
			"Investigational Product Blinded must only be sent for a clinical-trial report.",
		);
	}
}

/// ICH.G.k.2.3.r.REQUIRED
fn g_k_2_3_r(
	drug_idx: usize,
	drug: &DrugInformation,
	substances: &[DrugActiveSubstance],
	issues: &mut Vec<ValidationIssue>,
) {
	if !has_text(drug.mpid.as_deref())
		&& !has_text(drug.phpid.as_deref())
		&& !substances.iter().any(|substance| {
			substance.drug_id == drug.id
				&& (has_text(substance.substance_name.as_deref())
					|| has_text(substance.substance_termid.as_deref()))
		}) {
		crate::push_business_issue(
			issues,
			"ICH.G.k.2.3.r.REQUIRED",
			format!("drugs.{drug_idx}.activeSubstances.0.substanceName"),
			"At least one active ingredient is required when neither MPID nor PhPID is available.",
		);
	}
}

/// ICH.G.k.3.1.LENGTH.MAX
fn g_k_3_1(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.drugAuthorizationNumber");
	validate_length(
		issues,
		"ICH.G.k.3.1.LENGTH.MAX",
		&path,
		drug.drug_authorization_number.as_deref(),
	);
}

/// ICH.G.k.3.2.REQUIRED
/// ICH.G.k.3.2.VOCABULARY
/// ICH.G.k.3.2.LENGTH.MAX
fn g_k_3_2(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.drugAuthorizationCountry");
	validate_violation(
		issues,
		"ICH.G.k.3.2.REQUIRED",
		&path,
		has_text(drug.drug_authorization_number.as_deref())
			&& !has_text(drug.manufacturer_country.as_deref()),
	);
	validate_constraint(
		issues,
		"ICH.G.k.3.2.VOCABULARY",
		&path,
		ConstraintValue::Text(
			drug.manufacturer_country.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.3.2.LENGTH.MAX",
		&path,
		drug.manufacturer_country.as_deref(),
	);
}

/// ICH.G.k.3.3.LENGTH.MAX
fn g_k_3_3(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.manufacturerName");
	validate_length(
		issues,
		"ICH.G.k.3.3.LENGTH.MAX",
		&path,
		drug.manufacturer_name.as_deref(),
	);
}

/// ICH.G.k.5a.REQUIRED
/// ICH.G.k.5a.LENGTH.MAX
fn g_k_5a(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.cumulativeDoseFirstReactionValue");
	validate_violation(
		issues,
		"ICH.G.k.5a.REQUIRED",
		&path,
		has_text(drug.cumulative_dose_first_reaction_unit.as_deref())
			&& drug.cumulative_dose_first_reaction_value.is_none(),
	);
	let value = decimal_text(drug.cumulative_dose_first_reaction_value);
	validate_length(issues, "ICH.G.k.5a.LENGTH.MAX", &path, value.as_deref());
}

/// ICH.G.k.5b.REQUIRED
/// ICH.G.k.5b.LENGTH.MAX
fn g_k_5b(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.cumulativeDoseFirstReactionUnit");
	validate_violation(
		issues,
		"ICH.G.k.5b.REQUIRED",
		&path,
		drug.cumulative_dose_first_reaction_value.is_some()
			&& !has_text(drug.cumulative_dose_first_reaction_unit.as_deref()),
	);
	validate_length(
		issues,
		"ICH.G.k.5b.LENGTH.MAX",
		&path,
		drug.cumulative_dose_first_reaction_unit.as_deref(),
	);
}

/// ICH.G.k.6a.REQUIRED
/// ICH.G.k.6a.LENGTH.MAX
fn g_k_6a(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.gestationPeriodExposureValue");
	validate_violation(
		issues,
		"ICH.G.k.6a.REQUIRED",
		&path,
		has_text(drug.gestation_period_exposure_unit.as_deref())
			&& drug.gestation_period_exposure_value.is_none(),
	);
	let value = decimal_text(drug.gestation_period_exposure_value);
	validate_length(issues, "ICH.G.k.6a.LENGTH.MAX", &path, value.as_deref());
}

/// ICH.G.k.6b.REQUIRED
/// ICH.G.k.6b.LENGTH.MAX
fn g_k_6b(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.gestationPeriodExposureUnit");
	validate_violation(
		issues,
		"ICH.G.k.6b.REQUIRED",
		&path,
		drug.gestation_period_exposure_value.is_some()
			&& !has_text(drug.gestation_period_exposure_unit.as_deref()),
	);
	validate_length(
		issues,
		"ICH.G.k.6b.LENGTH.MAX",
		&path,
		drug.gestation_period_exposure_unit.as_deref(),
	);
}

/// ICH.G.k.8.ALLOWED.VALUE
/// ICH.G.k.8.LENGTH.MAX
fn g_k_8(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.actionTaken");
	validate_constraint(
		issues,
		"ICH.G.k.8.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(drug.action_taken.as_deref().map(Cow::Borrowed)),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.8.LENGTH.MAX",
		&path,
		drug.action_taken.as_deref(),
	);
}

/// ICH.G.k.10.r.ALLOWED.VALUE
/// ICH.G.k.10.r.LENGTH.MAX
fn g_k_10_r(
	idx: usize,
	drug: &DrugInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.drugAdditionalInformationCodes");
	let values = additional_info_codes(drug);
	validate_constraint(
		issues,
		"ICH.G.k.10.r.ALLOWED.VALUE",
		&path,
		ConstraintValue::Texts(
			values
				.iter()
				.map(|value| Cow::Borrowed(value.as_str()))
				.collect(),
		),
		vocabulary,
	);
	let longest = values.iter().max_by_key(|value| value.chars().count());
	validate_length(
		issues,
		"ICH.G.k.10.r.LENGTH.MAX",
		&path,
		longest.map(String::as_str),
	);
}

/// ICH.G.k.11.LENGTH.MAX
fn g_k_11(idx: usize, drug: &DrugInformation, issues: &mut Vec<ValidationIssue>) {
	let path = format!("drugs.{idx}.drugAdditionalInformation");
	validate_length(
		issues,
		"ICH.G.k.11.LENGTH.MAX",
		&path,
		drug.drug_additional_information.as_deref(),
	);
}

/// ICH.G.k.2.3.r.1.REQUIRED
/// ICH.G.k.2.3.r.1.LENGTH.MAX
fn g_k_2_3_r_1(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	substance: &DrugActiveSubstance,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.activeSubstances.{flat_idx}.substanceName");
	validate_violation(
		issues,
		"ICH.G.k.2.3.r.1.REQUIRED",
		&required_path,
		!has_text(substance.substance_termid.as_deref())
			&& !has_text(substance.substance_name.as_deref()),
	);
	if let Some((drug_idx, idx)) = nested {
		let path = format!("drugs.{drug_idx}.activeSubstances.{idx}.substanceName");
		validate_length(
			issues,
			"ICH.G.k.2.3.r.1.LENGTH.MAX",
			&path,
			substance.substance_name.as_deref(),
		);
	}
}

/// ICH.G.k.2.3.r.2a.REQUIRED
/// ICH.G.k.2.3.r.2a.LENGTH.MAX
fn g_k_2_3_r_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	substance: &DrugActiveSubstance,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.activeSubstances.{flat_idx}.substanceTermIdVersion");
	validate_violation(
		issues,
		"ICH.G.k.2.3.r.2a.REQUIRED",
		&required_path,
		has_text(substance.substance_termid.as_deref())
			&& !has_text(substance.substance_termid_version.as_deref()),
	);
	if let Some((drug_idx, idx)) = nested {
		let path = format!(
			"drugs.{drug_idx}.activeSubstances.{idx}.substanceTermIdVersion"
		);
		validate_length(
			issues,
			"ICH.G.k.2.3.r.2a.LENGTH.MAX",
			&path,
			substance.substance_termid_version.as_deref(),
		);
	}
}

/// ICH.G.k.2.3.r.2b.ALLOWED.VALUE
/// ICH.G.k.2.3.r.2b.LENGTH.MAX
fn g_k_2_3_r_2b(
	nested: Option<(usize, usize)>,
	substance: &DrugActiveSubstance,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some((drug_idx, idx)) = nested else {
		return;
	};
	let path = format!("drugs.{drug_idx}.activeSubstances.{idx}.substanceTermId");
	validate_constraint(
		issues,
		"ICH.G.k.2.3.r.2b.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(
			substance.substance_termid.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.2.3.r.2b.LENGTH.MAX",
		&path,
		substance.substance_termid.as_deref(),
	);
}

/// ICH.G.k.2.3.r.3a.LENGTH.MAX
fn g_k_2_3_r_3a(
	nested: Option<(usize, usize)>,
	substance: &DrugActiveSubstance,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some((drug_idx, idx)) = nested else {
		return;
	};
	let path = format!("drugs.{drug_idx}.activeSubstances.{idx}.strengthValue");
	let value = decimal_text(substance.strength_value);
	validate_length(
		issues,
		"ICH.G.k.2.3.r.3a.LENGTH.MAX",
		&path,
		value.as_deref(),
	);
}

/// ICH.G.k.2.3.r.3b.REQUIRED
/// ICH.G.k.2.3.r.3b.ALLOWED.VALUE
/// ICH.G.k.2.3.r.3b.LENGTH.MAX
fn g_k_2_3_r_3b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	substance: &DrugActiveSubstance,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.activeSubstances.{flat_idx}.strengthUnit");
	validate_violation(
		issues,
		"ICH.G.k.2.3.r.3b.REQUIRED",
		&required_path,
		substance.strength_value.is_some()
			&& !has_text(substance.strength_unit.as_deref()),
	);
	if let Some((drug_idx, idx)) = nested {
		let path = format!("drugs.{drug_idx}.activeSubstances.{idx}.strengthUnit");
		validate_constraint(
			issues,
			"ICH.G.k.2.3.r.3b.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(
				substance.strength_unit.as_deref().map(Cow::Borrowed),
			),
			vocabulary,
		);
		validate_length(
			issues,
			"ICH.G.k.2.3.r.3b.LENGTH.MAX",
			&path,
			substance.strength_unit.as_deref(),
		);
	}
}

fn dosage_path(nested: Option<(usize, usize)>, field: &str) -> Option<String> {
	nested.map(|(drug_idx, idx)| format!("drugs.{drug_idx}.dosages.{idx}.{field}"))
}

/// ICH.G.k.4.r.1a.LENGTH.MAX
fn g_k_4_r_1a(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some(path) = dosage_path(nested, "doseValue") else {
		return;
	};
	let value = decimal_text(dosage.dose_value);
	validate_length(issues, "ICH.G.k.4.r.1a.LENGTH.MAX", &path, value.as_deref());
}

/// ICH.G.k.4.r.1b.REQUIRED
/// ICH.G.k.4.r.1b.LENGTH.MAX
fn g_k_4_r_1b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.doseUnit");
	validate_violation(
		issues,
		"ICH.G.k.4.r.1b.REQUIRED",
		&required_path,
		dosage.dose_value.is_some() && !has_text(dosage.dose_unit.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "doseUnit") {
		validate_length(
			issues,
			"ICH.G.k.4.r.1b.LENGTH.MAX",
			&path,
			dosage.dose_unit.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.2.LENGTH.MAX
fn g_k_4_r_2(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some(path) = dosage_path(nested, "numberOfUnits") else {
		return;
	};
	let value = decimal_text(dosage.number_of_units);
	validate_length(issues, "ICH.G.k.4.r.2.LENGTH.MAX", &path, value.as_deref());
}

/// ICH.G.k.4.r.3.REQUIRED
/// ICH.G.k.4.r.3.ALLOWED.VALUE
/// ICH.G.k.4.r.3.LENGTH.MAX
fn g_k_4_r_3(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.frequencyUnit");
	validate_violation(
		issues,
		"ICH.G.k.4.r.3.REQUIRED",
		&required_path,
		dosage.number_of_units.is_some()
			&& !has_text(dosage.frequency_unit.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "frequencyUnit") {
		validate_constraint(
			issues,
			"ICH.G.k.4.r.3.ALLOWED.VALUE",
			&path,
			ConstraintValue::Text(
				dosage.frequency_unit.as_deref().map(Cow::Borrowed),
			),
			vocabulary,
		);
		validate_length(
			issues,
			"ICH.G.k.4.r.3.LENGTH.MAX",
			&path,
			dosage.frequency_unit.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.4-5.FUTURE_DATE.FORBIDDEN
fn g_k_4_r_4_5(
	flat_idx: usize,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.0.dosageInformation.{flat_idx}.dateRange");
	validate_future_date(
		issues,
		"ICH.G.k.4.r.4-5.FUTURE_DATE.FORBIDDEN",
		&path,
		DateValues::Two(
			dosage.first_administration_date,
			dosage.last_administration_date,
		),
	);
}

/// ICH.G.k.4.r.6a.REQUIRED
/// ICH.G.k.4.r.6a.LENGTH.MAX
fn g_k_4_r_6a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.durationValue");
	validate_violation(
		issues,
		"ICH.G.k.4.r.6a.REQUIRED",
		&required_path,
		has_text(dosage.duration_unit.as_deref()) && dosage.duration_value.is_none(),
	);
	if let Some(path) = dosage_path(nested, "durationValue") {
		let value = decimal_text(dosage.duration_value);
		validate_length(
			issues,
			"ICH.G.k.4.r.6a.LENGTH.MAX",
			&path,
			value.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.6b.REQUIRED
/// ICH.G.k.4.r.6b.LENGTH.MAX
fn g_k_4_r_6b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.durationUnit");
	validate_violation(
		issues,
		"ICH.G.k.4.r.6b.REQUIRED",
		&required_path,
		dosage.duration_value.is_some()
			&& !has_text(dosage.duration_unit.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "durationUnit") {
		validate_length(
			issues,
			"ICH.G.k.4.r.6b.LENGTH.MAX",
			&path,
			dosage.duration_unit.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.7.LENGTH.MAX
fn g_k_4_r_7(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "batchLotNumber") {
		validate_length(
			issues,
			"ICH.G.k.4.r.7.LENGTH.MAX",
			&path,
			dosage.batch_lot_number.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.8.LENGTH.MAX
fn g_k_4_r_8(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "dosageText") {
		validate_length(
			issues,
			"ICH.G.k.4.r.8.LENGTH.MAX",
			&path,
			dosage.dosage_text.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.9.1.LENGTH.MAX
fn g_k_4_r_9_1(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "doseForm") {
		validate_length(
			issues,
			"ICH.G.k.4.r.9.1.LENGTH.MAX",
			&path,
			dosage.dose_form.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.9.2a.REQUIRED
/// ICH.G.k.4.r.9.2a.LENGTH.MAX
fn g_k_4_r_9_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.doseFormTermIdVersion");
	validate_violation(
		issues,
		"ICH.G.k.4.r.9.2a.REQUIRED",
		&required_path,
		has_text(dosage.dose_form_termid.as_deref())
			&& !has_text(dosage.dose_form_termid_version.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "doseFormTermIdVersion") {
		validate_length(
			issues,
			"ICH.G.k.4.r.9.2a.LENGTH.MAX",
			&path,
			dosage.dose_form_termid_version.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.9.2b.LENGTH.MAX
fn g_k_4_r_9_2b(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "doseFormTermId") {
		validate_length(
			issues,
			"ICH.G.k.4.r.9.2b.LENGTH.MAX",
			&path,
			dosage.dose_form_termid.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.10.1.LENGTH.MAX
fn g_k_4_r_10_1(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "routeOfAdministration") {
		validate_length(
			issues,
			"ICH.G.k.4.r.10.1.LENGTH.MAX",
			&path,
			dosage.route_of_administration.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.10.2a.REQUIRED
/// ICH.G.k.4.r.10.2a.LENGTH.MAX
fn g_k_4_r_10_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!("drugs.0.dosages.{flat_idx}.routeTermIdVersion");
	validate_violation(
		issues,
		"ICH.G.k.4.r.10.2a.REQUIRED",
		&required_path,
		has_text(dosage.route_of_administration.as_deref())
			&& !has_text(dosage.route_termid_version.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "routeTermIdVersion") {
		validate_length(
			issues,
			"ICH.G.k.4.r.10.2a.LENGTH.MAX",
			&path,
			dosage.route_termid_version.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.10.2b.LENGTH.MAX
fn g_k_4_r_10_2b(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "routeTermId") {
		validate_length(
			issues,
			"ICH.G.k.4.r.10.2b.LENGTH.MAX",
			&path,
			dosage.route_termid.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.11.1.LENGTH.MAX
fn g_k_4_r_11_1(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "parentRoute") {
		validate_length(
			issues,
			"ICH.G.k.4.r.11.1.LENGTH.MAX",
			&path,
			dosage.parent_route.as_deref(),
		);
	}
}

/// ICH.G.k.4.r.11.2a.REQUIRED
/// ICH.G.k.4.r.11.2a.LENGTH.MAX
fn g_k_4_r_11_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.dosages.{flat_idx}.parentRouteTermIdVersion");
	validate_violation(
		issues,
		"ICH.G.k.4.r.11.2a.REQUIRED",
		&required_path,
		has_text(dosage.parent_route_termid.as_deref())
			&& !has_text(dosage.parent_route_termid_version.as_deref()),
	);
	if let Some(path) = dosage_path(nested, "parentRouteTermIdVersion") {
		validate_length(
			issues,
			"ICH.G.k.4.r.11.2a.LENGTH.MAX",
			&path,
			dosage.parent_route_termid_version.as_deref(),
		);
	}
}
/// ICH.G.k.4.r.11.2b.LENGTH.MAX
fn g_k_4_r_11_2b(
	nested: Option<(usize, usize)>,
	dosage: &DosageInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = dosage_path(nested, "parentRouteTermId") {
		validate_length(
			issues,
			"ICH.G.k.4.r.11.2b.LENGTH.MAX",
			&path,
			dosage.parent_route_termid.as_deref(),
		);
	}
}

/// ICH.G.k.7.r.1.LENGTH.MAX
fn g_k_7_r_1(
	nested: Option<(usize, usize)>,
	indication: &DrugIndication,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some((drug_idx, idx)) = nested else {
		return;
	};
	let path = format!("drugs.{drug_idx}.indications.{idx}.indicationText");
	validate_length(
		issues,
		"ICH.G.k.7.r.1.LENGTH.MAX",
		&path,
		indication.indication_text.as_deref(),
	);
}

/// ICH.G.k.7.r.2a.REQUIRED
/// ICH.G.k.7.r.2a.LENGTH.MAX
fn g_k_7_r_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	indication: &DrugIndication,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.indications.{flat_idx}.indicationMeddraVersion");
	validate_violation(
		issues,
		"ICH.G.k.7.r.2a.REQUIRED",
		&required_path,
		has_text(indication.indication_meddra_code.as_deref())
			&& !has_text(indication.indication_meddra_version.as_deref()),
	);
	if let Some((drug_idx, idx)) = nested {
		let path =
			format!("drugs.{drug_idx}.indications.{idx}.indicationMeddraVersion");
		validate_length(
			issues,
			"ICH.G.k.7.r.2a.LENGTH.MAX",
			&path,
			indication.indication_meddra_version.as_deref(),
		);
	}
}

/// ICH.G.k.7.r.2b.REQUIRED
/// ICH.G.k.7.r.2b.LENGTH.MAX
fn g_k_7_r_2b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	indication: &DrugIndication,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.indications.{flat_idx}.indicationMeddraCode");
	validate_violation(
		issues,
		"ICH.G.k.7.r.2b.REQUIRED",
		&required_path,
		has_text(indication.indication_meddra_version.as_deref())
			&& !has_text(indication.indication_meddra_code.as_deref()),
	);
	if let Some((drug_idx, idx)) = nested {
		let path =
			format!("drugs.{drug_idx}.indications.{idx}.indicationMeddraCode");
		validate_length(
			issues,
			"ICH.G.k.7.r.2b.LENGTH.MAX",
			&path,
			indication.indication_meddra_code.as_deref(),
		);
	}
}

/// ICH.G.k.7.r.2a.ALLOWED.VALUE
/// ICH.G.k.7.r.2a.VOCABULARY
/// ICH.G.k.7.r.2b.ALLOWED.VALUE
/// ICH.G.k.7.r.2b.VOCABULARY
fn g_k_7_r_2(
	nested: Option<(usize, usize)>,
	indication: &DrugIndication,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some((drug_idx, idx)) = nested else {
		return;
	};
	validate_meddra(
		issues,
		vocabulary,
		"ICH.G.k.7.r.2a.ALLOWED.VALUE",
		"ICH.G.k.7.r.2b.ALLOWED.VALUE",
		"ICH.G.k.7.r.2a.VOCABULARY",
		"ICH.G.k.7.r.2b.VOCABULARY",
		format!("drugs.{drug_idx}.indications.{idx}.indicationMeddraVersion"),
		format!("drugs.{drug_idx}.indications.{idx}.indicationMeddraCode"),
		indication.indication_meddra_version.as_deref(),
		indication.indication_meddra_code.as_deref(),
	);
}

fn assessment_path(nested: Option<(usize, usize)>, field: &str) -> Option<String> {
	nested.map(|(drug_idx, idx)| {
		format!("drugs.{drug_idx}.reactionAssessments.{idx}.{field}")
	})
}

/// ICH.G.k.9.i.3.1a.REQUIRED
/// ICH.G.k.9.i.3.1a.LENGTH.MAX
fn g_k_9_i_3_1a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	assessment: &DrugReactionAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!(
		"drugs.0.reactionAssessments.{flat_idx}.administrationStartIntervalValue"
	);
	validate_violation(
		issues,
		"ICH.G.k.9.i.3.1a.REQUIRED",
		&required_path,
		has_text(assessment.administration_start_interval_unit.as_deref())
			&& assessment.administration_start_interval_value.is_none(),
	);
	if let Some(path) = assessment_path(nested, "administrationStartIntervalValue") {
		let value = decimal_text(assessment.administration_start_interval_value);
		validate_length(
			issues,
			"ICH.G.k.9.i.3.1a.LENGTH.MAX",
			&path,
			value.as_deref(),
		);
	}
}

/// ICH.G.k.9.i.3.1b.REQUIRED
/// ICH.G.k.9.i.3.1b.LENGTH.MAX
fn g_k_9_i_3_1b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	assessment: &DrugReactionAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path = format!(
		"drugs.0.reactionAssessments.{flat_idx}.administrationStartIntervalUnit"
	);
	validate_violation(
		issues,
		"ICH.G.k.9.i.3.1b.REQUIRED",
		&required_path,
		assessment.administration_start_interval_value.is_some()
			&& !has_text(assessment.administration_start_interval_unit.as_deref()),
	);
	if let Some(path) = assessment_path(nested, "administrationStartIntervalUnit") {
		validate_length(
			issues,
			"ICH.G.k.9.i.3.1b.LENGTH.MAX",
			&path,
			assessment.administration_start_interval_unit.as_deref(),
		);
	}
}

/// ICH.G.k.9.i.3.2a.REQUIRED
/// ICH.G.k.9.i.3.2a.LENGTH.MAX
fn g_k_9_i_3_2a(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	assessment: &DrugReactionAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.reactionAssessments.{flat_idx}.lastDoseIntervalValue");
	validate_violation(
		issues,
		"ICH.G.k.9.i.3.2a.REQUIRED",
		&required_path,
		has_text(assessment.last_dose_interval_unit.as_deref())
			&& assessment.last_dose_interval_value.is_none(),
	);
	if let Some(path) = assessment_path(nested, "lastDoseIntervalValue") {
		let value = decimal_text(assessment.last_dose_interval_value);
		validate_length(
			issues,
			"ICH.G.k.9.i.3.2a.LENGTH.MAX",
			&path,
			value.as_deref(),
		);
	}
}

/// ICH.G.k.9.i.3.2b.REQUIRED
/// ICH.G.k.9.i.3.2b.LENGTH.MAX
fn g_k_9_i_3_2b(
	flat_idx: usize,
	nested: Option<(usize, usize)>,
	assessment: &DrugReactionAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	let required_path =
		format!("drugs.0.reactionAssessments.{flat_idx}.lastDoseIntervalUnit");
	validate_violation(
		issues,
		"ICH.G.k.9.i.3.2b.REQUIRED",
		&required_path,
		assessment.last_dose_interval_value.is_some()
			&& !has_text(assessment.last_dose_interval_unit.as_deref()),
	);
	if let Some(path) = assessment_path(nested, "lastDoseIntervalUnit") {
		validate_length(
			issues,
			"ICH.G.k.9.i.3.2b.LENGTH.MAX",
			&path,
			assessment.last_dose_interval_unit.as_deref(),
		);
	}
}

/// ICH.G.k.9.i.4.ALLOWED.VALUE
/// ICH.G.k.9.i.4.LENGTH.MAX
fn g_k_9_i_4(
	nested: Option<(usize, usize)>,
	assessment: &DrugReactionAssessment,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some(path) = assessment_path(nested, "reactionRecurred") else {
		return;
	};
	validate_constraint(
		issues,
		"ICH.G.k.9.i.4.ALLOWED.VALUE",
		&path,
		ConstraintValue::Text(
			assessment.reaction_recurred.as_deref().map(Cow::Borrowed),
		),
		vocabulary,
	);
	validate_length(
		issues,
		"ICH.G.k.9.i.4.LENGTH.MAX",
		&path,
		assessment.reaction_recurred.as_deref(),
	);
}

fn relatedness_path(
	nested: Option<(usize, usize, usize)>,
	field: &str,
) -> Option<String> {
	nested.map(|(drug_idx, assessment_idx, idx)| format!("drugs.{drug_idx}.reactionAssessments.{assessment_idx}.relatednessAssessments.{idx}.{field}"))
}

/// ICH.G.k.9.i.2.r.1.LENGTH.MAX
fn g_k_9_i_2_r_1(
	nested: Option<(usize, usize, usize)>,
	relatedness: &RelatednessAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = relatedness_path(nested, "sourceOfAssessment") {
		validate_length(
			issues,
			"ICH.G.k.9.i.2.r.1.LENGTH.MAX",
			&path,
			relatedness.source_of_assessment.as_deref(),
		);
	}
}
/// ICH.G.k.9.i.2.r.2.LENGTH.MAX
fn g_k_9_i_2_r_2(
	nested: Option<(usize, usize, usize)>,
	relatedness: &RelatednessAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = relatedness_path(nested, "methodOfAssessment") {
		validate_length(
			issues,
			"ICH.G.k.9.i.2.r.2.LENGTH.MAX",
			&path,
			relatedness.method_of_assessment.as_deref(),
		);
	}
}
/// ICH.G.k.9.i.2.r.3.LENGTH.MAX
fn g_k_9_i_2_r_3(
	nested: Option<(usize, usize, usize)>,
	relatedness: &RelatednessAssessment,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(path) = relatedness_path(nested, "resultOfAssessment") {
		validate_length(
			issues,
			"ICH.G.k.9.i.2.r.3.LENGTH.MAX",
			&path,
			relatedness.result_of_assessment.as_deref(),
		);
	}
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
			collect_fda_issues(ctx, mm, validation_ctx, fda_ctx, issues).await?
		}
		RegulatoryAuthority::Mfds => {
			if let Some(mfds_ctx) = mfds_ctx {
				collect_mfds_issues(validation_ctx, mfds_ctx, issues);
			}
		}
	}
	Ok(())
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let is_clinical_trial = validation_ctx.studies.iter().any(|study| {
		study.study_type_reaction.as_deref().map(str::trim) == Some("1")
	});
	g_k_1(&validation_ctx.drugs, &validation_ctx.vocabulary, issues);
	g_k_2_2(&validation_ctx.drugs, issues);
	for (idx, drug) in validation_ctx.drugs.iter().enumerate() {
		g_k_2_1_1a(idx, drug, issues);
		g_k_2_1_1b(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_2_1_2a(idx, drug, issues);
		g_k_2_1_2b(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_2_4(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_2_5(
			idx,
			drug,
			is_clinical_trial,
			&validation_ctx.vocabulary,
			issues,
		);
		g_k_2_3_r(idx, drug, &validation_ctx.active_substances, issues);
		g_k_3_1(idx, drug, issues);
		g_k_3_2(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_3_3(idx, drug, issues);
		g_k_5a(idx, drug, issues);
		g_k_5b(idx, drug, issues);
		g_k_6a(idx, drug, issues);
		g_k_6b(idx, drug, issues);
		g_k_8(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_10_r(idx, drug, &validation_ctx.vocabulary, issues);
		g_k_11(idx, drug, issues);
	}

	let drug_indices = validation_ctx
		.drugs
		.iter()
		.enumerate()
		.map(|(idx, drug)| (drug.id, idx))
		.collect::<HashMap<_, _>>();
	let mut fallback = HashMap::new();
	for (flat_idx, substance) in validation_ctx.active_substances.iter().enumerate()
	{
		let nested = drug_indices
			.get(&substance.drug_id)
			.copied()
			.map(|drug_idx| {
				let fallback_idx = fallback.entry(substance.drug_id).or_insert(0);
				let idx = sequence_idx(substance.sequence_number, *fallback_idx);
				*fallback_idx += 1;
				(drug_idx, idx)
			});
		g_k_2_3_r_1(flat_idx, nested, substance, issues);
		g_k_2_3_r_2a(flat_idx, nested, substance, issues);
		g_k_2_3_r_2b(nested, substance, &validation_ctx.vocabulary, issues);
		g_k_2_3_r_3a(nested, substance, issues);
		g_k_2_3_r_3b(
			flat_idx,
			nested,
			substance,
			&validation_ctx.vocabulary,
			issues,
		);
	}

	let mut fallback = HashMap::new();
	for (flat_idx, dosage) in validation_ctx.dosages.iter().enumerate() {
		let nested = drug_indices.get(&dosage.drug_id).copied().map(|drug_idx| {
			let fallback_idx = fallback.entry(dosage.drug_id).or_insert(0);
			let idx = sequence_idx(dosage.sequence_number, *fallback_idx);
			*fallback_idx += 1;
			(drug_idx, idx)
		});
		g_k_4_r_1a(nested, dosage, issues);
		g_k_4_r_1b(flat_idx, nested, dosage, issues);
		g_k_4_r_2(nested, dosage, issues);
		g_k_4_r_3(flat_idx, nested, dosage, &validation_ctx.vocabulary, issues);
		g_k_4_r_4_5(flat_idx, dosage, issues);
		g_k_4_r_6a(flat_idx, nested, dosage, issues);
		g_k_4_r_6b(flat_idx, nested, dosage, issues);
		g_k_4_r_7(nested, dosage, issues);
		g_k_4_r_8(nested, dosage, issues);
		g_k_4_r_9_1(nested, dosage, issues);
		g_k_4_r_9_2a(flat_idx, nested, dosage, issues);
		g_k_4_r_9_2b(nested, dosage, issues);
		g_k_4_r_10_1(nested, dosage, issues);
		g_k_4_r_10_2a(flat_idx, nested, dosage, issues);
		g_k_4_r_10_2b(nested, dosage, issues);
		g_k_4_r_11_1(nested, dosage, issues);
		g_k_4_r_11_2a(flat_idx, nested, dosage, issues);
		g_k_4_r_11_2b(nested, dosage, issues);
	}

	let mut fallback = HashMap::new();
	for (flat_idx, indication) in validation_ctx.indications.iter().enumerate() {
		let nested =
			drug_indices
				.get(&indication.drug_id)
				.copied()
				.map(|drug_idx| {
					let fallback_idx =
						fallback.entry(indication.drug_id).or_insert(0);
					let idx =
						sequence_idx(indication.sequence_number, *fallback_idx);
					*fallback_idx += 1;
					(drug_idx, idx)
				});
		g_k_7_r_1(nested, indication, issues);
		g_k_7_r_2a(flat_idx, nested, indication, issues);
		g_k_7_r_2b(flat_idx, nested, indication, issues);
		g_k_7_r_2(nested, indication, &validation_ctx.vocabulary, issues);
	}

	let mut fallback = HashMap::new();
	let mut assessment_indices = HashMap::new();
	for (flat_idx, assessment) in
		validation_ctx.drug_reaction_assessments.iter().enumerate()
	{
		let nested =
			drug_indices
				.get(&assessment.drug_id)
				.copied()
				.map(|drug_idx| {
					let idx = *fallback.entry(assessment.drug_id).or_insert(0);
					*fallback.get_mut(&assessment.drug_id).expect("entry exists") +=
						1;
					assessment_indices.insert(assessment.id, (drug_idx, idx));
					(drug_idx, idx)
				});
		g_k_9_i_3_1a(flat_idx, nested, assessment, issues);
		g_k_9_i_3_1b(flat_idx, nested, assessment, issues);
		g_k_9_i_3_2a(flat_idx, nested, assessment, issues);
		g_k_9_i_3_2b(flat_idx, nested, assessment, issues);
		g_k_9_i_4(nested, assessment, &validation_ctx.vocabulary, issues);
	}
	let mut fallback = HashMap::new();
	for relatedness in &validation_ctx.relatedness_assessments {
		let assessment_id = relatedness.drug_reaction_assessment_id;
		let nested = assessment_indices.get(&assessment_id).copied().map(
			|(drug_idx, assessment_idx)| {
				let fallback_idx = fallback.entry(assessment_id).or_insert(0);
				let idx = sequence_idx(relatedness.sequence_number, *fallback_idx);
				*fallback_idx += 1;
				(drug_idx, assessment_idx, idx)
			},
		);
		g_k_9_i_2_r_1(nested, relatedness, issues);
		g_k_9_i_2_r_2(nested, relatedness, issues);
		g_k_9_i_2_r_3(nested, relatedness, issues);
	}
}

fn fda_required(
	code: &str,
	path: &str,
	value: Option<&str>,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		code,
		path,
		RuleValue::borrowed(value, None),
		RuleFacts::default(),
	);
}

/// FDA.G.k.1.a.REQUIRED
fn fda_g_k_1_a(idx: usize, value: Option<&str>, issues: &mut Vec<ValidationIssue>) {
	fda_required(
		"FDA.G.k.1.a.REQUIRED",
		&format!("drugs.{idx}.fdaOtherCharacterization"),
		value,
		issues,
	);
}
/// FDA.G.K.12.REQUIRED
fn fda_g_k_12(value: Option<&str>, issues: &mut Vec<ValidationIssue>) {
	fda_required(
		"FDA.G.K.12.REQUIRED",
		"drugs.0.fdaDevices.0.malfunction",
		value,
		issues,
	);
}
/// FDA.G.K.1.A.CONDITIONAL
fn fda_g_k_1_a_conditional(invalid: bool, issues: &mut Vec<ValidationIssue>) {
	validate_violation(
		issues,
		"FDA.G.K.1.A.CONDITIONAL",
		"drugs.0.deviceCharacteristics.0.valueCode",
		invalid,
	);
}

/// FDA.D.1 R0027: a combination-product malfunction without an adverse event
/// must identify the unavailable patient with nullFlavor NA.
fn fda_d_1_malfunction(
	validation_ctx: &ValidationContext,
	combination_true: bool,
	has_malfunction: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let malfunction_without_event =
		validation_ctx.reactions.iter().any(|reaction| {
			reaction.reaction_meddra_code.as_deref().map(str::trim)
				== Some("10067482")
		});
	if combination_true
		&& has_malfunction
		&& malfunction_without_event
		&& validation_ctx.patient.as_ref().is_none_or(|patient| {
			patient
				.patient_initials_null_flavor
				.as_deref()
				.map(str::trim)
				!= Some("NA")
		}) {
		crate::push_business_issue(
			issues,
			"FDA.D.1.R0027",
			"patientInformation.patientInitialsNullFlavor",
			"Patient identification must use null flavor NA for a combination-product malfunction without an adverse event.",
		);
	}
}

/// FDA.G.k.1.ROUTE
fn fda_g_k_1_route(
	validation_ctx: &ValidationContext,
	first_product_malfunction: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let Some(first) = validation_ctx.drugs.first() else {
		return;
	};
	let batch_receiver = validation_ctx
		.message_header
		.as_ref()
		.and_then(|header| header.batch_receiver_identifier.as_deref());
	let message_receiver = validation_ctx
		.message_header
		.as_ref()
		.map(|header| header.message_receiver_identifier.as_str());
	let role = first.drug_characterization.trim();
	if is_fda_ind_message_receiver(message_receiver) {
		let report_type_is_study = validation_ctx
			.safety_report
			.as_ref()
			.and_then(|report| report.report_type.as_deref())
			.map(str::trim)
			== Some("2");
		if report_type_is_study {
			for (idx, drug) in validation_ctx.drugs.iter().enumerate() {
				if !matches!(drug.drug_characterization.trim(), "1" | "2" | "3") {
					crate::push_business_issue(
						issues,
						"FDA.G.k.1.ROUTE",
						format!("drugs.{idx}.drugCharacterization"),
						"Drug characterization is not valid for the selected FDA route.",
					);
				}
			}
		}
		return;
	}
	let vaers = [batch_receiver, message_receiver]
		.into_iter()
		.flatten()
		.any(|receiver| receiver.to_ascii_uppercase().contains("VAERS"));
	let invalid = (is_fda_postmarket_batch_receiver(batch_receiver)
		&& !matches!(role, "1" | "3")
		&& !(first_product_malfunction && role == "4"))
		|| (vaers && !matches!(role, "1" | "3"));
	if invalid {
		crate::push_business_issue(
			issues,
			"FDA.G.k.1.ROUTE",
			"drugs.0.drugCharacterization",
			"Drug characterization is not valid for the selected FDA route.",
		);
	}
}

/// FDA.G.k.10a.REQUIRED
fn fda_g_k_10a(
	idx: usize,
	value: Option<&str>,
	null_flavor: Option<&str>,
	pre_anda_present: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	if !pre_anda_present {
		return;
	}
	let value = value.map(str::trim).filter(|value| !value.is_empty());
	let null_flavor = null_flavor.map(str::trim).filter(|value| !value.is_empty());
	let invalid = !matches!(value, Some("1" | "2")) && null_flavor != Some("NA");
	if invalid {
		crate::push_business_issue(
			issues,
			"FDA.G.k.10a.REQUIRED",
			format!("drugs.{idx}.fdaAdditionalInfoCoded"),
			"FDA.G.k.10a must be 1 or 2 for an IND-exempt BA/BE study.",
		);
	}
}

/// FDA.G.k.9.i.2.r.1.REQUIRED
/// FDA.G.k.9.i.2.r.2.REQUIRED
/// FDA.G.k.9.i.2.r.3.REQUIRED
fn fda_g_k_9(
	validation_ctx: &ValidationContext,
	ind_number_present: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let report_type_is_study = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|report| report.report_type.as_deref())
		.map(str::trim)
		== Some("2");
	if !report_type_is_study || !ind_number_present {
		return;
	}
	let drug_indices = validation_ctx
		.drugs
		.iter()
		.enumerate()
		.map(|(idx, drug)| (drug.id, idx))
		.collect::<HashMap<_, _>>();
	let first_suspect_idx = validation_ctx
		.drugs
		.iter()
		.position(|drug| drug.drug_characterization.trim() == "1");
	if validation_ctx.drug_reaction_assessments.is_empty() {
		let drug_idx = first_suspect_idx.unwrap_or(0);
		for (code, field, message) in [
			(
				"FDA.G.k.9.i.2.r.1.REQUIRED",
				"sourceOfAssessment",
				"Source of Assessment is required for an IND safety report.",
			),
			(
				"FDA.G.k.9.i.2.r.2.REQUIRED",
				"methodOfAssessment",
				"Method of Assessment is required for an IND safety report.",
			),
			(
				"FDA.G.k.9.i.2.r.3.REQUIRED",
				"resultOfAssessment",
				"A suspect product relatedness result is required for an IND safety report.",
			),
		] {
			crate::push_business_issue(
				issues,
				code,
				format!("drugs.{drug_idx}.reactionAssessments.0.relatednessAssessments.0.{field}"),
				message,
			);
		}
		return;
	}
	let mut assessment_indices = HashMap::new();
	let mut has_suspect_result = false;
	for assessment in &validation_ctx.drug_reaction_assessments {
		let Some(drug_idx) = drug_indices.get(&assessment.drug_id).copied() else {
			continue;
		};
		let is_suspect =
			validation_ctx.drugs[drug_idx].drug_characterization.trim() == "1";
		let assessment_idx =
			assessment_indices.entry(assessment.drug_id).or_insert(0);
		let rows = validation_ctx
			.relatedness_assessments
			.iter()
			.filter(|row| row.drug_reaction_assessment_id == assessment.id)
			.collect::<Vec<_>>();
		for (fallback_idx, row) in rows.iter().enumerate() {
			let relatedness_idx = sequence_idx(row.sequence_number, fallback_idx);
			let has_source = row.source_of_assessment.as_deref().map(str::trim)
				== Some("Sponsor");
			let has_method =
				row.method_of_assessment.as_deref().map(str::trim) == Some("FDA");
			let has_result = matches!(
				row.result_of_assessment.as_deref().map(str::trim),
				Some("Suspected" | "Not Suspected")
			);
			if is_suspect {
				has_suspect_result |= has_result;
			}
			for (code, field, present, message) in [
				(
					"FDA.G.k.9.i.2.r.1.REQUIRED",
					"sourceOfAssessment",
					has_source,
					"Source of Assessment is required for an IND safety report.",
				),
				(
					"FDA.G.k.9.i.2.r.2.REQUIRED",
					"methodOfAssessment",
					has_method,
					"Method of Assessment is required for an IND safety report.",
				),
			] {
				if !present {
					crate::push_business_issue(
						issues,
						code,
						format!("drugs.{drug_idx}.reactionAssessments.{assessment_idx}.relatednessAssessments.{relatedness_idx}.{field}"),
						message,
					);
				}
			}
		}
		if rows.is_empty() {
			for (code, field) in [
				("FDA.G.k.9.i.2.r.1.REQUIRED", "sourceOfAssessment"),
				("FDA.G.k.9.i.2.r.2.REQUIRED", "methodOfAssessment"),
			] {
				crate::push_business_issue(
					issues,
					code,
					format!("drugs.{drug_idx}.reactionAssessments.{assessment_idx}.relatednessAssessments.0.{field}"),
					"A relatedness assessment value is required for an IND safety report.",
				);
			}
		}
		*assessment_idx += 1;
	}
	if !has_suspect_result {
		let drug_idx = first_suspect_idx.unwrap_or(0);
		crate::push_business_issue(
			issues,
			"FDA.G.k.9.i.2.r.3.REQUIRED",
			format!("drugs.{drug_idx}.reactionAssessments.0.relatednessAssessments.0.resultOfAssessment"),
			"At least one suspect product must have a relatedness result for an IND safety report.",
		);
	}
}

pub(crate) async fn collect_fda_issues(
	ctx: &Ctx,
	mm: &ModelManager,
	validation_ctx: &ValidationContext,
	fda_ctx: Option<&FdaValidationContext>,
	issues: &mut Vec<ValidationIssue>,
) -> Result<()> {
	let local_criteria = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|r| r.local_criteria_report_type.as_deref());
	let combination_true = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|r| r.combination_product_report_indicator.as_deref())
		.map(str::trim)
		.is_some_and(|value| value == "1" || value.eq_ignore_ascii_case("true"));
	let message_receiver = validation_ctx
		.message_header
		.as_ref()
		.map(|header| header.message_receiver_identifier.as_str());
	let batch_receiver = validation_ctx
		.message_header
		.as_ref()
		.and_then(|header| header.batch_receiver_identifier.as_deref());
	let postmarket = is_fda_postmarket_batch_receiver(batch_receiver);
	let device_rules_apply = !is_fda_premarket_message_receiver(message_receiver);

	let mut has_malfunction_suspect = false;
	let mut has_malfunction = false;
	let mut has_invalid_gk1a = false;
	let mut first_product_malfunction = false;
	let mut combination_device_identity_present = false;
	let pre_anda_present = fda_ctx.is_some_and(|ctx| {
		ctx.studies
			.iter()
			.any(|study| has_text(study.fda_pre_anda_number_occurred.as_deref()))
	});
	let ind_number_present = fda_ctx.is_some_and(|ctx| {
		ctx.studies
			.iter()
			.any(|study| has_text(study.fda_ind_number_occurred.as_deref()))
	});

	for (drug_idx, drug) in validation_ctx.drugs.iter().enumerate() {
		let (devices, device_codes) = list_fda_devices(ctx, mm, drug.id).await?;
		let malfunction_this_drug = devices
			.iter()
			.any(|device| device.malfunction == Some(true));
		if drug_idx == 0 {
			first_product_malfunction = malfunction_this_drug;
		}
		let gk1a_required = combination_true
			&& malfunction_this_drug
			&& drug.drug_characterization == "4";
		fda_g_k_1_a(
			drug_idx,
			if gk1a_required {
				drug.fda_other_characterization.as_deref()
			} else {
				Some("not-applicable")
			},
			issues,
		);
		for (device_idx, device) in devices.iter().enumerate() {
			if !device_rules_apply {
				continue;
			}
			let path = format!("drugs.{drug_idx}.fdaDevices.{device_idx}");
			let malfunction = device.malfunction == Some(true);
			let has_identity = has_text(device.device_brand_name.as_deref())
				|| has_text(device.common_device_name.as_deref())
				|| has_text(device.device_product_code.as_deref());
			combination_device_identity_present |= has_identity;
			if ((postmarket && combination_true) || malfunction) && !has_identity {
				crate::push_business_issue(
					issues,
					"FDA.G.k.12.r.4-6.AT_LEAST_ONE",
					format!("{path}.deviceBrandName"),
					"A malfunctioning device requires a non-null brand name, common name, or product code.",
				);
			}
			if !malfunction {
				continue;
			}
			let has_problem = device_codes.iter().any(|code| {
				code.device_id == device.id
					&& code.element == "device_problem"
					&& has_text(Some(&code.value_code))
			});
			if !has_problem {
				crate::push_business_issue(
					issues,
					"FDA.G.K.12.R.3.REQUIRED",
					format!("{path}.deviceProblemCodes.0.valueCode"),
					"A device problem code is required for each malfunctioning device.",
				);
			}
			let has_remedial = device_codes.iter().any(|code| {
				code.device_id == device.id
					&& code.element == "remedial_action"
					&& has_text(Some(&code.value_code))
			});
			if local_criteria == Some("4") && !has_remedial {
				crate::push_business_issue(
					issues,
					"FDA.G.K.12.R.11.REQUIRED",
					format!("{path}.remedialActions.0.valueCode"),
					"A remedial action is required for each malfunctioning device in a 5-day report.",
				);
			}
		}
		if malfunction_this_drug {
			has_malfunction = true;
			if drug.drug_characterization == "1" {
				has_malfunction_suspect = true;
			}
		}
		fda_g_k_10a(
			drug_idx,
			drug.fda_additional_info_coded.as_deref(),
			drug.fda_additional_info_coded_null_flavor.as_deref(),
			pre_anda_present,
			issues,
		);
		let has_gk1a_one = drug.fda_other_characterization.as_deref() == Some("1");
		if has_gk1a_one
			&& !(combination_true
				&& malfunction_this_drug
				&& drug.drug_characterization == "4")
		{
			has_invalid_gk1a = true;
		}
	}
	if postmarket && combination_true && !combination_device_identity_present {
		crate::push_business_issue(
			issues,
			"FDA.G.k.12.r.4-6.AT_LEAST_ONE",
			"drugs.0.fdaDevices.0.deviceBrandName",
			"A combination product requires device brand name, common name, or product code.",
		);
	}
	fda_d_1_malfunction(validation_ctx, combination_true, has_malfunction, issues);
	fda_g_k_12(
		if local_criteria == Some("5") {
			has_malfunction_suspect.then_some("present")
		} else {
			Some("not-applicable")
		},
		issues,
	);
	fda_g_k_1_route(validation_ctx, first_product_malfunction, issues);
	fda_g_k_9(validation_ctx, ind_number_present, issues);
	fda_g_k_1_a_conditional(has_invalid_gk1a, issues);
	Ok(())
}

/// MFDS.G.k.2.1.KR.1b.REQUIRED
/// MFDS.G.k.2.1.KR.1b.VOCABULARY
/// MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED
/// MFDS.KR.FOREIGN.WHOMPID.REQUIRED
fn mfds_g_k_2_1_kr_1b(
	idx: usize,
	value: Option<&str>,
	receiver: Option<&str>,
	facts: RuleFacts,
	vocabulary: &crate::context::VocabularyContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{idx}.mfdsMpid");
	for code in [
		"MFDS.G.k.2.1.KR.1b.REQUIRED",
		"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		"MFDS.KR.FOREIGN.WHOMPID.REQUIRED",
	] {
		validate_value(issues, code, &path, RuleValue::borrowed(value, None), facts);
	}
	validate_vocabulary_variant(
		issues,
		"MFDS.G.k.2.1.KR.1b.VOCABULARY",
		&path,
		receiver,
		value,
		vocabulary,
	);
}

/// MFDS.G.k.2.1.1b.REQUIRED
/// MFDS.G.k.2.1.2a.REQUIRED
/// MFDS.G.k.2.1.2b.REQUIRED
fn mfds_g_k_2_1_companions(
	idx: usize,
	drug: &DrugInformation,
	issues: &mut Vec<ValidationIssue>,
) {
	for (code, field, missing) in [
		(
			"MFDS.G.k.2.1.1b.REQUIRED",
			"mpid",
			has_text(drug.mpid_version.as_deref())
				&& !has_text(drug.mpid.as_deref()),
		),
		(
			"MFDS.G.k.2.1.2a.REQUIRED",
			"phpidVersion",
			has_text(drug.phpid.as_deref())
				&& !has_text(drug.phpid_version.as_deref()),
		),
		(
			"MFDS.G.k.2.1.2b.REQUIRED",
			"phpid",
			has_text(drug.phpid_version.as_deref())
				&& !has_text(drug.phpid.as_deref()),
		),
	] {
		validate_violation(issues, code, &format!("drugs.{idx}.{field}"), missing);
	}
}

/// MFDS.G.k.2.3.r.2b.REQUIRED
fn mfds_g_k_2_3_r_2b(
	drug_idx: usize,
	idx: usize,
	substance: &DrugActiveSubstance,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_violation(
		issues,
		"MFDS.G.k.2.3.r.2b.REQUIRED",
		&format!("drugs.{drug_idx}.activeSubstances.{idx}.substanceTermId"),
		has_text(substance.substance_termid_version.as_deref())
			&& !has_text(substance.substance_termid.as_deref()),
	);
}

/// MFDS.G.k.2.1.KR.1a.REQUIRED
fn mfds_g_k_2_1_kr_1a(
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"MFDS.G.k.2.1.KR.1a.REQUIRED",
		&format!("drugs.{idx}.mfdsMpidVersion"),
		RuleValue::borrowed(value, None),
		facts,
	);
}

/// MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED
/// MFDS.G.k.2.3.r.1.KR.1b.REQUIRED
fn mfds_g_k_2_3_r_1_kr_1b(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!("drugs.{drug_idx}.activeSubstances.{idx}.mfdsId");
	for code in [
		"MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED",
		"MFDS.G.k.2.3.r.1.KR.1b.REQUIRED",
	] {
		validate_value(issues, code, &path, RuleValue::borrowed(value, None), facts);
	}
}

/// MFDS.G.k.2.3.r.1.KR.1a.REQUIRED
fn mfds_g_k_2_3_r_1_kr_1a(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"MFDS.G.k.2.3.r.1.KR.1a.REQUIRED",
		&format!("drugs.{drug_idx}.activeSubstances.{idx}.mfdsVersion"),
		RuleValue::borrowed(value, None),
		facts,
	);
}

/// MFDS.G.k.9.i.2.r.1.REQUIRED
fn mfds_g_k_9_i_2_r_1(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"MFDS.G.k.9.i.2.r.1.REQUIRED",
		&format!(
			"drugs.{drug_idx}.drugReactionAssessments.{idx}.sourceOfAssessment"
		),
		RuleValue::borrowed(value, None),
		facts,
	);
}

/// MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED
#[allow(clippy::too_many_arguments)]
fn mfds_g_k_9_i_2_r_2_kr_1(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	receiver_is_ct_or_cu: bool,
	receiver_is_kr: bool,
	receiver_is_fr: bool,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!(
		"drugs.{drug_idx}.drugReactionAssessments.{idx}.methodOfAssessmentKr1"
	);
	validate_value(
		issues,
		"MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED",
		&path,
		RuleValue::borrowed(value, None),
		facts,
	);
	let invalid = value.map(str::trim).is_some_and(|code| {
		!matches!(code, "1" | "2")
			|| if receiver_is_ct_or_cu {
				code != "2"
			} else if receiver_is_kr {
				code != "1"
			} else {
				receiver_is_fr
			}
	});
	validate_violation(issues, "MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED", &path, invalid);
}

/// MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED
fn mfds_g_k_9_i_2_r_3_kr_1(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	null_flavor: Option<&str>,
	method: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	let path = format!(
		"drugs.{drug_idx}.drugReactionAssessments.{idx}.resultOfAssessmentKr1"
	);
	validate_value(
		issues,
		"MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED",
		&path,
		RuleValue::borrowed(value, null_flavor),
		facts,
	);
	let invalid = method.map(str::trim) == Some("1")
		&& value.map(str::trim).is_some_and(|code| {
			!code.is_empty() && !matches!(code, "1" | "2" | "3" | "4" | "5" | "6")
		});
	validate_violation(issues, "MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED", &path, invalid);
}

fn mfds_kr1_result_required(
	receiver_is_kr: bool,
	has_source: bool,
	method_is_who_umc: bool,
) -> bool {
	receiver_is_kr && has_source && method_is_who_umc
}

/// MFDS.G.k.9.i.2.r.3.KR.2.REQUIRED
fn mfds_g_k_9_i_2_r_3_kr_2(
	drug_idx: usize,
	idx: usize,
	value: Option<&str>,
	facts: RuleFacts,
	issues: &mut Vec<ValidationIssue>,
) {
	validate_value(
		issues,
		"MFDS.G.k.9.i.2.r.3.KR.2.REQUIRED",
		&format!(
			"drugs.{drug_idx}.drugReactionAssessments.{idx}.resultOfAssessmentKr2"
		),
		RuleValue::borrowed(value, None),
		facts,
	);
}

pub(crate) fn collect_mfds_issues(
	validation_ctx: &ValidationContext,
	mfds_ctx: &MfdsValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let report_type_is_study = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|r| r.report_type.as_deref())
		== Some("2");
	let msg_receiver = validation_ctx
		.message_header
		.as_ref()
		.map(|h| h.message_receiver_identifier.as_str());
	let receiver_is_kr = is_mfds_domestic_receiver(msg_receiver);
	let receiver_is_fr = is_mfds_foreign_postmarket_receiver(msg_receiver);
	let vocabulary_receiver = if receiver_is_kr {
		Some("KR")
	} else if receiver_is_fr {
		Some("FR")
	} else {
		None
	};
	let receiver_is_ct_or_cu = is_mfds_clinical_trial_receiver(msg_receiver)
		|| is_mfds_compassionate_use_receiver(msg_receiver);

	let mut domestic_drug_ids = HashSet::new();
	let mut drug_index_by_id = HashMap::new();
	let mut drug_has_mfds_mpid_by_id = HashMap::new();

	for (idx, drug) in validation_ctx.drugs.iter().enumerate() {
		drug_index_by_id.insert(drug.id, idx);
		let has_mfds_mpid = has_text(drug.mfds_mpid.as_deref());
		drug_has_mfds_mpid_by_id.insert(drug.id, has_mfds_mpid);
		let country = drug.obtain_drug_country.as_deref().map(str::trim);
		let is_domestic_kr = matches!(country, Some("KR"));
		let is_foreign_non_kr =
			matches!(country, Some(other) if !other.is_empty() && other != "KR");
		if is_domestic_kr {
			domestic_drug_ids.insert(drug.id);
		}
		let facts = RuleFacts {
			mfds_product_code_required_context: Some(
				receiver_is_kr || receiver_is_fr,
			),
			mfds_product_version_required_context: Some(
				receiver_is_fr && has_mfds_mpid,
			),
			mfds_drug_domestic_kr: Some(is_domestic_kr),
			mfds_drug_foreign_non_kr: Some(is_foreign_non_kr),
			..RuleFacts::default()
		};
		mfds_g_k_2_1_kr_1b(
			idx,
			drug.mfds_mpid.as_deref(),
			vocabulary_receiver,
			facts,
			&validation_ctx.vocabulary,
			issues,
		);
		mfds_g_k_2_1_kr_1a(idx, drug.mfds_mpid_version.as_deref(), facts, issues);
		mfds_g_k_2_1_companions(idx, drug, issues);
		// MFDS G.k.8: required for clinical-trial and compassionate-use reports.
		if receiver_is_ct_or_cu && !has_text(drug.action_taken.as_deref()) {
			crate::push_business_issue(
				issues,
				"MFDS.G.k.8.REQUIRED",
				format!("drugs.{idx}.actionTaken"),
				"Action Taken with Drug is required for CT and CU reports.",
			);
		}
	}

	for substance in &mfds_ctx.active_substances {
		let Some((drug_index, substance_index)) = resolve_drug_child_indices(
			&drug_index_by_id,
			substance.drug_id,
			substance.sequence_number,
		) else {
			continue;
		};
		let drug_has_mfds_mpid = drug_has_mfds_mpid_by_id
			.get(&substance.drug_id)
			.copied()
			.unwrap_or(false);
		let facts = RuleFacts {
			mfds_drug_domestic_kr: Some(
				domestic_drug_ids.contains(&substance.drug_id),
			),
			mfds_substance_code_required_context: Some(
				(receiver_is_kr || receiver_is_fr) && !drug_has_mfds_mpid,
			),
			mfds_substance_version_required_context: Some(
				receiver_is_fr && has_text(substance.mfds_id.as_deref()),
			),
			..RuleFacts::default()
		};
		mfds_g_k_2_3_r_1_kr_1b(
			drug_index,
			substance_index,
			substance.mfds_id.as_deref(),
			facts,
			issues,
		);
		mfds_g_k_2_3_r_1_kr_1a(
			drug_index,
			substance_index,
			substance.mfds_version.as_deref(),
			facts,
			issues,
		);
		mfds_g_k_2_3_r_2b(drug_index, substance_index, substance, issues);
	}
	if receiver_is_kr || receiver_is_fr {
		for (drug_idx, drug) in validation_ctx.drugs.iter().enumerate() {
			if !has_text(drug.mfds_mpid.as_deref())
				&& !mfds_ctx.active_substances.iter().any(|substance| {
					substance.drug_id == drug.id
						&& has_text(substance.mfds_id.as_deref())
				}) {
				crate::push_business_issue(
					issues,
					"MFDS.G.k.2.3.r.1.KR.1b.REQUIRED",
					format!("drugs.{drug_idx}.activeSubstances.0.mfdsId"),
					"An MFDS ingredient code is required when the MFDS product code is unavailable.",
				);
			}
		}
	}

	for r in &mfds_ctx.relatedness {
		let Some((drug_index, assessment_index)) = resolve_drug_child_indices(
			&drug_index_by_id,
			r.drug_id,
			r.relatedness_sequence_number,
		) else {
			continue;
		};
		let has_source = has_text(r.source_of_assessment.as_deref());
		let has_method = has_text(r.method_of_assessment_kr1.as_deref());
		let has_result_kr1 = has_text(r.result_of_assessment_kr1.as_deref())
			|| has_text(r.result_of_assessment_kr1_null_flavor.as_deref());
		let has_result_kr2 = has_text(r.result_of_assessment_kr2.as_deref());
		let has_any_result = has_result_kr1 || has_result_kr2;
		let method_code = r.method_of_assessment_kr1.as_deref().map(str::trim);
		let method_is_who_umc = method_code == Some("1");
		let method_is_krct = method_code == Some("2");
		let method_required_context = has_source || receiver_is_ct_or_cu;
		let kr2_required_context = has_source
			&& method_is_krct
			&& (report_type_is_study || receiver_is_ct_or_cu);
		let facts = RuleFacts {
			mfds_relatedness_method_required_context: Some(method_required_context),
			mfds_relatedness_kr1_required_context: Some(mfds_kr1_result_required(
				receiver_is_kr,
				has_source,
				method_is_who_umc,
			)),
			mfds_relatedness_kr2_required_context: Some(kr2_required_context),
			mfds_relatedness_method_present: Some(has_method),
			mfds_relatedness_result_present: Some(has_any_result),
			..RuleFacts::default()
		};
		mfds_g_k_9_i_2_r_2_kr_1(
			drug_index,
			assessment_index,
			r.method_of_assessment_kr1.as_deref(),
			facts,
			receiver_is_ct_or_cu,
			receiver_is_kr,
			receiver_is_fr,
			issues,
		);
		mfds_g_k_9_i_2_r_3_kr_1(
			drug_index,
			assessment_index,
			r.result_of_assessment_kr1.as_deref(),
			r.result_of_assessment_kr1_null_flavor.as_deref(),
			r.method_of_assessment_kr1.as_deref(),
			facts,
			issues,
		);
		mfds_g_k_9_i_2_r_3_kr_2(
			drug_index,
			assessment_index,
			r.result_of_assessment_kr2.as_deref(),
			facts,
			issues,
		);
		mfds_g_k_9_i_2_r_1(
			drug_index,
			assessment_index,
			r.source_of_assessment.as_deref(),
			facts,
			issues,
		);
	}
}

#[cfg(test)]
pub(super) fn constraint_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.G.k.2.1.1b.ALLOWED.VALUE",
		"ICH.G.k.2.1.2b.ALLOWED.VALUE",
		"ICH.G.k.1.ALLOWED.VALUE",
		"ICH.G.k.8.ALLOWED.VALUE",
		"ICH.G.k.2.4.VOCABULARY",
		"ICH.G.k.3.2.VOCABULARY",
		"ICH.G.k.10.r.ALLOWED.VALUE",
		"ICH.G.k.2.5.ALLOWED.VALUE",
		"ICH.G.k.9.i.4.ALLOWED.VALUE",
		"ICH.G.k.2.3.r.2b.ALLOWED.VALUE",
		"ICH.G.k.2.3.r.3b.ALLOWED.VALUE",
		"ICH.G.k.4.r.3.ALLOWED.VALUE",
		"ICH.G.k.7.r.2a.ALLOWED.VALUE",
		"ICH.G.k.7.r.2b.ALLOWED.VALUE",
	]
}

#[cfg(test)]
pub(super) fn implemented_rule_codes() -> Vec<&'static str> {
	vec![
		"FDA.G.K.1.A.CONDITIONAL",
		"FDA.G.K.12.R.11.REQUIRED",
		"FDA.G.K.12.R.3.REQUIRED",
		"FDA.G.K.12.REQUIRED",
		"FDA.G.k.1.a.REQUIRED",
		"FDA.G.k.12.r.1.REQUIRED",
		"FDA.G.k.12.r.11.r.REQUIRED",
		"FDA.G.k.12.r.3.r.REQUIRED",
		"FDA.G.k.12.r.4.REQUIRED",
		"FDA.G.k.12.r.5.REQUIRED",
		"FDA.G.k.12.r.6.REQUIRED",
		"ICH.G.k.1.ALLOWED.VALUE",
		"ICH.G.k.1.LENGTH.MAX",
		"ICH.G.k.1.REQUIRED",
		"ICH.G.k.10.r.ALLOWED.VALUE",
		"ICH.G.k.10.r.LENGTH.MAX",
		"ICH.G.k.11.LENGTH.MAX",
		"ICH.G.k.2.1.1a.LENGTH.MAX",
		"ICH.G.k.2.1.1b.ALLOWED.VALUE",
		"ICH.G.k.2.1.1b.LENGTH.MAX",
		"ICH.G.k.2.1.2a.LENGTH.MAX",
		"ICH.G.k.2.1.2b.ALLOWED.VALUE",
		"ICH.G.k.2.1.2b.LENGTH.MAX",
		"ICH.G.k.2.2.LENGTH.MAX",
		"ICH.G.k.2.2.REQUIRED",
		"ICH.G.k.2.3.r.1.LENGTH.MAX",
		"ICH.G.k.2.3.r.1.REQUIRED",
		"ICH.G.k.2.3.r.2a.LENGTH.MAX",
		"ICH.G.k.2.3.r.2a.REQUIRED",
		"ICH.G.k.2.3.r.2b.ALLOWED.VALUE",
		"ICH.G.k.2.3.r.2b.LENGTH.MAX",
		"ICH.G.k.2.3.r.3a.LENGTH.MAX",
		"ICH.G.k.2.3.r.3b.ALLOWED.VALUE",
		"ICH.G.k.2.3.r.3b.LENGTH.MAX",
		"ICH.G.k.2.3.r.3b.REQUIRED",
		"ICH.G.k.2.4.LENGTH.MAX",
		"ICH.G.k.2.4.VOCABULARY",
		"ICH.G.k.2.5.ALLOWED.VALUE",
		"ICH.G.k.3.1.LENGTH.MAX",
		"ICH.G.k.3.2.LENGTH.MAX",
		"ICH.G.k.3.2.REQUIRED",
		"ICH.G.k.3.2.VOCABULARY",
		"ICH.G.k.3.3.LENGTH.MAX",
		"ICH.G.k.4.r.10.1.LENGTH.MAX",
		"ICH.G.k.4.r.10.2a.LENGTH.MAX",
		"ICH.G.k.4.r.10.2a.REQUIRED",
		"ICH.G.k.4.r.10.2b.LENGTH.MAX",
		"ICH.G.k.4.r.11.1.LENGTH.MAX",
		"ICH.G.k.4.r.11.2a.LENGTH.MAX",
		"ICH.G.k.4.r.11.2a.REQUIRED",
		"ICH.G.k.4.r.11.2b.LENGTH.MAX",
		"ICH.G.k.4.r.1a.LENGTH.MAX",
		"ICH.G.k.4.r.1b.LENGTH.MAX",
		"ICH.G.k.4.r.1b.REQUIRED",
		"ICH.G.k.4.r.2.LENGTH.MAX",
		"ICH.G.k.4.r.3.ALLOWED.VALUE",
		"ICH.G.k.4.r.3.LENGTH.MAX",
		"ICH.G.k.4.r.3.REQUIRED",
		"ICH.G.k.4.r.4-5.FUTURE_DATE.FORBIDDEN",
		"ICH.G.k.4.r.6a.LENGTH.MAX",
		"ICH.G.k.4.r.6a.REQUIRED",
		"ICH.G.k.4.r.6b.LENGTH.MAX",
		"ICH.G.k.4.r.6b.REQUIRED",
		"ICH.G.k.4.r.7.LENGTH.MAX",
		"ICH.G.k.4.r.8.LENGTH.MAX",
		"ICH.G.k.4.r.9.1.LENGTH.MAX",
		"ICH.G.k.4.r.9.2a.LENGTH.MAX",
		"ICH.G.k.4.r.9.2a.REQUIRED",
		"ICH.G.k.4.r.9.2b.LENGTH.MAX",
		"ICH.G.k.5a.LENGTH.MAX",
		"ICH.G.k.5a.REQUIRED",
		"ICH.G.k.5b.LENGTH.MAX",
		"ICH.G.k.5b.REQUIRED",
		"ICH.G.k.6a.LENGTH.MAX",
		"ICH.G.k.6a.REQUIRED",
		"ICH.G.k.6b.LENGTH.MAX",
		"ICH.G.k.6b.REQUIRED",
		"ICH.G.k.7.r.1.LENGTH.MAX",
		"ICH.G.k.7.r.2a.ALLOWED.VALUE",
		"ICH.G.k.7.r.2a.LENGTH.MAX",
		"ICH.G.k.7.r.2a.REQUIRED",
		"ICH.G.k.7.r.2a.VOCABULARY",
		"ICH.G.k.7.r.2b.ALLOWED.VALUE",
		"ICH.G.k.7.r.2b.LENGTH.MAX",
		"ICH.G.k.7.r.2b.REQUIRED",
		"ICH.G.k.7.r.2b.VOCABULARY",
		"ICH.G.k.8.ALLOWED.VALUE",
		"ICH.G.k.8.LENGTH.MAX",
		"ICH.G.k.9.i.2.r.1.LENGTH.MAX",
		"ICH.G.k.9.i.2.r.2.LENGTH.MAX",
		"ICH.G.k.9.i.2.r.3.LENGTH.MAX",
		"ICH.G.k.9.i.3.1a.LENGTH.MAX",
		"ICH.G.k.9.i.3.1a.REQUIRED",
		"ICH.G.k.9.i.3.1b.LENGTH.MAX",
		"ICH.G.k.9.i.3.1b.REQUIRED",
		"ICH.G.k.9.i.3.2a.LENGTH.MAX",
		"ICH.G.k.9.i.3.2a.REQUIRED",
		"ICH.G.k.9.i.3.2b.LENGTH.MAX",
		"ICH.G.k.9.i.3.2b.REQUIRED",
		"ICH.G.k.9.i.4.ALLOWED.VALUE",
		"ICH.G.k.9.i.4.LENGTH.MAX",
		"MFDS.G.k.2.1.KR.1a.REQUIRED",
		"MFDS.G.k.2.1.KR.1b.REQUIRED",
		"MFDS.G.k.2.1.KR.1b.VOCABULARY",
		"MFDS.G.k.2.3.r.1.KR.1a.REQUIRED",
		"MFDS.G.k.2.3.r.1.KR.1b.REQUIRED",
		"MFDS.G.k.9.i.2.r.1.REQUIRED",
		"MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED",
		"MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED",
		"MFDS.G.k.9.i.2.r.3.KR.2.REQUIRED",
		"MFDS.KR.DOMESTIC.INGREDIENTCODE.REQUIRED",
		"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
		"MFDS.KR.FOREIGN.WHOMPID.REQUIRED",
	]
}
#[cfg(test)]
mod conditioned_catalog_rule_tests {
	use super::*;
	use sqlx::types::Uuid;

	#[test]
	fn drug_rules_cover_domestic_foreign_and_unrelated_contexts() {
		let mut issues = Vec::new();
		let vocabulary = crate::context::VocabularyContext::default();
		for (index, mpid, mpid_version, facts) in [
			(
				1,
				None,
				None,
				RuleFacts {
					mfds_product_code_required_context: Some(true),
					mfds_product_version_required_context: Some(false),
					mfds_drug_domestic_kr: Some(true),
					mfds_drug_foreign_non_kr: Some(false),
					..RuleFacts::default()
				},
			),
			(
				2,
				Some("product"),
				None,
				RuleFacts {
					mfds_product_code_required_context: Some(true),
					mfds_product_version_required_context: Some(true),
					mfds_drug_domestic_kr: Some(false),
					mfds_drug_foreign_non_kr: Some(true),
					..RuleFacts::default()
				},
			),
			(3, None, None, RuleFacts::default()),
		] {
			mfds_g_k_2_1_kr_1b(index, mpid, None, facts, &vocabulary, &mut issues);
			mfds_g_k_2_1_kr_1a(index, mpid_version, facts, &mut issues);
		}

		assert_eq!(
			issues
				.iter()
				.map(|issue| issue.code.as_str())
				.collect::<Vec<_>>(),
			[
				"MFDS.G.k.2.1.KR.1b.REQUIRED",
				"MFDS.KR.DOMESTIC.PRODUCTCODE.REQUIRED",
				"MFDS.G.k.2.1.KR.1a.REQUIRED",
			]
		);
		assert!(issues
			.iter()
			.all(|issue| issue.field_path.as_deref() != Some("drugs.3.mfdsMpid")));
	}

	#[test]
	fn substance_rules_preserve_resolved_drug_and_substance_indices() {
		let mut issues = Vec::new();
		for (drug_index, substance_index, id, version, facts) in [
			(
				1,
				2,
				None,
				None,
				RuleFacts {
					mfds_drug_domestic_kr: Some(true),
					mfds_substance_code_required_context: Some(true),
					mfds_substance_version_required_context: Some(false),
					..RuleFacts::default()
				},
			),
			(
				3,
				4,
				Some("ingredient"),
				None,
				RuleFacts {
					mfds_drug_domestic_kr: Some(false),
					mfds_substance_code_required_context: Some(true),
					mfds_substance_version_required_context: Some(true),
					..RuleFacts::default()
				},
			),
		] {
			mfds_g_k_2_3_r_1_kr_1b(
				drug_index,
				substance_index,
				id,
				facts,
				&mut issues,
			);
			mfds_g_k_2_3_r_1_kr_1a(
				drug_index,
				substance_index,
				version,
				facts,
				&mut issues,
			);
		}

		assert_eq!(issues.len(), 3);
		assert_eq!(
			issues
				.iter()
				.map(|issue| issue.field_path.as_deref().unwrap())
				.collect::<Vec<_>>(),
			[
				"drugs.1.activeSubstances.2.mfdsId",
				"drugs.1.activeSubstances.2.mfdsId",
				"drugs.3.activeSubstances.4.mfdsVersion",
			]
		);
	}

	#[test]
	fn relatedness_rules_cover_method_results_and_source_companion() {
		let mut issues = Vec::new();
		for (
			assessment_index,
			source,
			method,
			result_kr1,
			result_kr1_null_flavor,
			result_kr2,
			receiver_is_ct_or_cu,
			receiver_is_kr,
			receiver_is_fr,
			facts,
		) in [
			(
				2,
				Some("source"),
				None,
				None,
				None,
				None,
				false,
				false,
				false,
				RuleFacts {
					mfds_relatedness_method_required_context: Some(true),
					..RuleFacts::default()
				},
			),
			(
				3,
				Some("source"),
				Some("1"),
				None,
				None,
				None,
				false,
				false,
				false,
				RuleFacts {
					mfds_relatedness_method_required_context: Some(true),
					mfds_relatedness_kr1_required_context: Some(true),
					..RuleFacts::default()
				},
			),
			(
				4,
				Some("source"),
				Some("2"),
				None,
				None,
				None,
				true,
				false,
				false,
				RuleFacts {
					mfds_relatedness_method_required_context: Some(true),
					mfds_relatedness_kr2_required_context: Some(true),
					..RuleFacts::default()
				},
			),
			(
				5,
				None,
				Some("1"),
				None,
				None,
				None,
				false,
				true,
				false,
				RuleFacts {
					mfds_relatedness_method_present: Some(true),
					mfds_relatedness_result_present: Some(false),
					..RuleFacts::default()
				},
			),
			(
				6,
				Some("source"),
				Some("1"),
				None,
				Some("NA"),
				None,
				false,
				false,
				false,
				RuleFacts {
					mfds_relatedness_kr1_required_context: Some(true),
					..RuleFacts::default()
				},
			),
		] {
			mfds_g_k_9_i_2_r_2_kr_1(
				1,
				assessment_index,
				method,
				facts,
				receiver_is_ct_or_cu,
				receiver_is_kr,
				receiver_is_fr,
				&mut issues,
			);
			mfds_g_k_9_i_2_r_3_kr_1(
				1,
				assessment_index,
				result_kr1,
				result_kr1_null_flavor,
				method,
				facts,
				&mut issues,
			);
			mfds_g_k_9_i_2_r_3_kr_2(
				1,
				assessment_index,
				result_kr2,
				facts,
				&mut issues,
			);
			mfds_g_k_9_i_2_r_1(1, assessment_index, source, facts, &mut issues);
		}

		assert_eq!(
			issues
				.iter()
				.map(|issue| issue.code.as_str())
				.collect::<Vec<_>>(),
			[
				"MFDS.G.k.9.i.2.r.2.KR.1.REQUIRED",
				"MFDS.G.k.9.i.2.r.3.KR.1.REQUIRED",
				"MFDS.G.k.9.i.2.r.3.KR.2.REQUIRED",
				"MFDS.G.k.9.i.2.r.1.REQUIRED",
			]
		);
	}

	#[test]
	fn mfds_who_umc_kr1_result_is_scoped_to_domestic_reports() {
		assert!(mfds_kr1_result_required(true, true, true));
		assert!(!mfds_kr1_result_required(false, true, true));
		assert!(!mfds_kr1_result_required(true, false, true));
		assert!(!mfds_kr1_result_required(true, true, false));
	}

	#[test]
	fn child_indices_have_no_owner_or_sequence_fallback() {
		let known_drug = Uuid::new_v4();
		let unknown_drug = Uuid::new_v4();
		let indices = HashMap::from([(known_drug, 2)]);

		assert_eq!(
			resolve_drug_child_indices(&indices, known_drug, 4),
			Some((2, 3))
		);
		assert_eq!(resolve_drug_child_indices(&indices, unknown_drug, 4), None);
		assert_eq!(resolve_drug_child_indices(&indices, known_drug, 0), None);
	}
}

#[cfg(test)]
mod golden_g_required_tests {
	use super::*;
	use lib_core::model::case::Case;
	use serde_json::json;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Decimal;
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

	fn drug() -> DrugInformation {
		DrugInformation {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_product_presave_id: None,
			sequence_number: 1,
			drug_characterization: String::new(),
			medicinal_product: String::new(),
			mpid: None,
			mpid_version: None,
			mfds_mpid_version: None,
			mfds_mpid: None,
			phpid: None,
			phpid_version: None,
			investigational_product_blinded: None,
			obtain_drug_country: None,
			drug_authorization_number: None,
			manufacturer_name: None,
			manufacturer_country: None,
			batch_lot_number: None,
			cumulative_dose_first_reaction_value: None,
			cumulative_dose_first_reaction_unit: None,
			gestation_period_exposure_value: None,
			gestation_period_exposure_unit: None,
			action_taken: None,
			fda_additional_info_coded: None,
			fda_additional_info_coded_null_flavor: None,
			drug_additional_info_codes_json: None,
			drug_additional_information: None,
			fda_specialized_product_category: None,
			fda_other_characterization: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn substance() -> DrugActiveSubstance {
		DrugActiveSubstance {
			id: Uuid::nil(),
			drug_id: Uuid::nil(),
			sequence_number: 1,
			substance_name: None,
			substance_termid: None,
			substance_termid_version: None,
			mfds_version: None,
			mfds_id: None,
			strength_value: None,
			strength_unit: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn dosage() -> DosageInformation {
		DosageInformation {
			id: Uuid::nil(),
			drug_id: Uuid::nil(),
			sequence_number: 1,
			dose_value: None,
			dose_unit: None,
			number_of_units: None,
			frequency_unit: None,
			first_administration_date: None,
			last_administration_date: None,
			duration_value: None,
			duration_unit: None,
			continuing: None,
			batch_lot_number: None,
			dosage_text: None,
			dose_form: None,
			dose_form_null_flavor: None,
			dose_form_termid: None,
			dose_form_termid_version: None,
			route_of_administration: None,
			route_of_administration_null_flavor: None,
			route_termid: None,
			route_termid_version: None,
			parent_route: None,
			parent_route_null_flavor: None,
			parent_route_termid: None,
			parent_route_termid_version: None,
			first_administration_date_null_flavor: None,
			last_administration_date_null_flavor: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn indication() -> DrugIndication {
		DrugIndication {
			id: Uuid::nil(),
			drug_id: Uuid::nil(),
			sequence_number: 1,
			indication_text: None,
			indication_text_null_flavor: None,
			indication_meddra_version: None,
			indication_meddra_code: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn assessment() -> DrugReactionAssessment {
		DrugReactionAssessment {
			id: Uuid::nil(),
			drug_id: Uuid::nil(),
			reaction_id: Uuid::nil(),
			administration_start_interval_value: None,
			administration_start_interval_unit: None,
			last_dose_interval_value: None,
			last_dose_interval_unit: None,
			recurrence_action: None,
			reaction_recurred: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn relatedness() -> RelatednessAssessment {
		RelatednessAssessment {
			id: Uuid::nil(),
			drug_reaction_assessment_id: Uuid::nil(),
			sequence_number: 1,
			source_of_assessment: None,
			method_of_assessment: None,
			method_of_assessment_kr1: None,
			result_of_assessment: None,
			result_of_assessment_kr1: None,
			result_of_assessment_kr1_null_flavor: None,
			result_of_assessment_kr2: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn codes_for(ctx: &ValidationContext) -> Vec<String> {
		let mut issues = Vec::new();
		collect_ich_issues(ctx, &mut issues);
		issues.into_iter().map(|issue| issue.code).collect()
	}

	fn length_issues(ctx: &ValidationContext) -> Vec<(String, String)> {
		let mut issues = Vec::new();
		collect_ich_issues(ctx, &mut issues);
		issues
			.into_iter()
			.filter(|issue| issue.code.ends_with(".LENGTH.MAX"))
			.map(|issue| (issue.code, issue.field_path.unwrap_or_default()))
			.collect()
	}

	#[test]
	fn allowed_value_rules_cover_g_drug_and_reaction_codes() {
		let mut ctx = empty_ctx();
		let mut drug = drug();
		drug.id = Uuid::from_u128(1);
		drug.drug_characterization = "9".to_string();
		drug.investigational_product_blinded = Some(false);
		drug.action_taken = Some("8".to_string());
		drug.drug_additional_info_codes_json = Some(json!([
			{ "value_code": "12" }
		]));
		ctx.drugs.push(drug);

		let mut assessment = assessment();
		assessment.drug_id = Uuid::from_u128(1);
		assessment.reaction_recurred = Some("9".to_string());
		ctx.drug_reaction_assessments.push(assessment);

		let codes = codes_for(&ctx);
		assert!(codes.contains(&"ICH.G.k.1.ALLOWED.VALUE".to_string()));
		assert!(codes.contains(&"ICH.G.k.2.5.ALLOWED.VALUE".to_string()));
		assert!(codes.contains(&"ICH.G.k.8.ALLOWED.VALUE".to_string()));
		assert!(codes.contains(&"ICH.G.k.9.i.4.ALLOWED.VALUE".to_string()));
		assert!(codes.contains(&"ICH.G.k.10.r.ALLOWED.VALUE".to_string()));
	}

	#[test]
	fn meddra_vocabulary_rules_cover_g_indication_codes() {
		let mut ctx = empty_ctx();
		ctx.vocabulary =
			crate::context::VocabularyContext::for_meddra(&[("26.1", "10000001")]);

		let mut drug = drug();
		drug.id = Uuid::from_u128(1);
		ctx.drugs.push(drug);

		let mut indication = indication();
		indication.drug_id = Uuid::from_u128(1);
		indication.indication_meddra_version = Some("99.9".to_string());
		indication.indication_meddra_code = Some("99999999".to_string());
		ctx.indications.push(indication);

		let codes = codes_for(&ctx);
		assert!(codes.contains(&"ICH.G.k.7.r.2a.VOCABULARY".to_string()));
		assert!(codes.contains(&"ICH.G.k.7.r.2b.VOCABULARY".to_string()));
	}

	#[test]
	fn empty_drug_collection_flags_placeholder_drug_rules() {
		assert_eq!(
			codes_for(&empty_ctx()),
			vec![
				"ICH.G.k.1.REQUIRED".to_string(),
				"ICH.G.k.2.2.REQUIRED".to_string(),
			]
		);
	}

	#[test]
	fn drug_required_and_pair_rules_are_preserved() {
		let mut ctx = empty_ctx();
		let mut drug = drug();
		drug.cumulative_dose_first_reaction_unit = Some("mg".to_string());
		drug.gestation_period_exposure_value = Some("1".parse().unwrap());
		ctx.drugs.push(drug);

		assert_eq!(
			codes_for(&ctx),
			vec![
				"ICH.G.k.1.REQUIRED".to_string(),
				"ICH.G.k.1.AGGREGATE.REQUIRED".to_string(),
				"ICH.G.k.2.2.REQUIRED".to_string(),
				"ICH.G.k.2.3.r.REQUIRED".to_string(),
				"ICH.G.k.5a.REQUIRED".to_string(),
				"ICH.G.k.6b.REQUIRED".to_string(),
			]
		);
	}

	#[test]
	fn nested_collection_companion_rules_are_preserved() {
		let mut ctx = empty_ctx();
		let mut substance = substance();
		substance.substance_termid = Some("SUB123".to_string());
		substance.strength_value = Some("1".parse().unwrap());
		ctx.active_substances.push(substance);

		let mut dosage = dosage();
		dosage.dose_value = Some("1".parse().unwrap());
		dosage.duration_unit = Some("d".to_string());
		dosage.route_of_administration = Some("030".to_string());
		ctx.dosages.push(dosage);

		assert_eq!(
			codes_for(&ctx),
			vec![
				"ICH.G.k.1.REQUIRED".to_string(),
				"ICH.G.k.2.2.REQUIRED".to_string(),
				"ICH.G.k.2.3.r.2a.REQUIRED".to_string(),
				"ICH.G.k.2.3.r.3b.REQUIRED".to_string(),
				"ICH.G.k.4.r.1b.REQUIRED".to_string(),
				"ICH.G.k.4.r.6a.REQUIRED".to_string(),
				"ICH.G.k.4.r.10.2a.REQUIRED".to_string(),
			]
		);
	}

	#[test]
	fn dosage_frequency_unit_is_required_from_number_of_units() {
		let mut ctx = empty_ctx();
		let mut dosage = dosage();
		dosage.number_of_units = Some(Decimal::new(5, 1));
		ctx.dosages.push(dosage);

		assert!(codes_for(&ctx)
			.iter()
			.any(|code| code == "ICH.G.k.4.r.3.REQUIRED"));
	}

	#[test]
	fn dosage_frequency_unit_uses_frequency_vocabulary_scope() {
		const ALLOWED: [&str; 9] = [
			"a",
			"mo",
			"wk",
			"d",
			"h",
			"min",
			"{cyclical}",
			"{asnecessary}",
			"{total}",
		];
		let active = ALLOWED
			.map(|code| ("ICH-UCUM", crate::VocabularyScope::Frequency, code));

		for unit in ALLOWED {
			let mut ctx = empty_ctx();
			ctx.vocabulary =
				crate::context::VocabularyContext::for_active_codes(&active);
			let mut drug = drug();
			drug.id = Uuid::from_u128(1);
			ctx.drugs.push(drug);
			let mut dosage = dosage();
			dosage.drug_id = Uuid::from_u128(1);
			dosage.frequency_unit = Some(unit.to_string());
			ctx.dosages.push(dosage);

			assert!(
				!codes_for(&ctx)
					.iter()
					.any(|code| code == "ICH.G.k.4.r.3.ALLOWED.VALUE"),
				"approved unit {unit} was rejected"
			);
		}

		let mut ctx = empty_ctx();
		ctx.vocabulary =
			crate::context::VocabularyContext::for_active_codes(&active);
		let mut drug = drug();
		drug.id = Uuid::from_u128(1);
		ctx.drugs.push(drug);
		let mut dosage = dosage();
		dosage.drug_id = Uuid::from_u128(1);
		dosage.frequency_unit = Some("fortnight".to_string());
		ctx.dosages.push(dosage);

		assert!(codes_for(&ctx)
			.iter()
			.any(|code| code == "ICH.G.k.4.r.3.ALLOWED.VALUE"));
	}

	#[test]
	fn indication_and_reaction_assessment_pair_rules_are_preserved() {
		let mut ctx = empty_ctx();
		let mut indication = indication();
		indication.indication_meddra_version = Some("26.1".to_string());
		ctx.indications.push(indication);

		let mut assessment = assessment();
		assessment.administration_start_interval_value = Some("1".parse().unwrap());
		assessment.last_dose_interval_unit = Some("d".to_string());
		ctx.drug_reaction_assessments.push(assessment);

		assert_eq!(
			codes_for(&ctx),
			vec![
				"ICH.G.k.1.REQUIRED".to_string(),
				"ICH.G.k.2.2.REQUIRED".to_string(),
				"ICH.G.k.7.r.2b.REQUIRED".to_string(),
				"ICH.G.k.9.i.3.1b.REQUIRED".to_string(),
				"ICH.G.k.9.i.3.2a.REQUIRED".to_string(),
			]
		);
	}

	#[test]
	fn max_length_rules_cover_g_drug_fields() {
		let mut ctx = empty_ctx();
		let mut drug = drug();
		drug.drug_characterization = "12".to_string();
		drug.mpid_version = Some("12345678901".to_string());
		drug.mpid = Some("x".repeat(1001));
		drug.phpid_version = Some("12345678901".to_string());
		drug.phpid = Some("x".repeat(251));
		drug.medicinal_product = "x".repeat(251);
		drug.obtain_drug_country = Some("USA".to_string());
		drug.drug_authorization_number = Some("x".repeat(36));
		drug.manufacturer_country = Some("USA".to_string());
		drug.manufacturer_name = Some("x".repeat(61));
		drug.cumulative_dose_first_reaction_value =
			Some(Decimal::new(12_345_678_901, 0));
		drug.cumulative_dose_first_reaction_unit = Some("x".repeat(51));
		drug.gestation_period_exposure_value = Some(Decimal::new(1234, 0));
		drug.gestation_period_exposure_unit = Some("x".repeat(51));
		drug.action_taken = Some("12".to_string());
		drug.drug_additional_info_codes_json = Some(json!([
			{ "value_code": "123" }
		]));
		drug.drug_additional_information = Some("x".repeat(2001));
		ctx.drugs.push(drug);

		assert_eq!(
			length_issues(&ctx),
			vec![
				(
					"ICH.G.k.1.LENGTH.MAX".to_string(),
					"drugs.0.drugCharacterization".to_string()
				),
				(
					"ICH.G.k.2.2.LENGTH.MAX".to_string(),
					"drugs.0.medicinalProduct".to_string()
				),
				(
					"ICH.G.k.2.1.1a.LENGTH.MAX".to_string(),
					"drugs.0.mpidVersion".to_string()
				),
				(
					"ICH.G.k.2.1.1b.LENGTH.MAX".to_string(),
					"drugs.0.mpid".to_string()
				),
				(
					"ICH.G.k.2.1.2a.LENGTH.MAX".to_string(),
					"drugs.0.phpidVersion".to_string()
				),
				(
					"ICH.G.k.2.1.2b.LENGTH.MAX".to_string(),
					"drugs.0.phpid".to_string()
				),
				(
					"ICH.G.k.2.4.LENGTH.MAX".to_string(),
					"drugs.0.obtainDrugCountry".to_string()
				),
				(
					"ICH.G.k.3.1.LENGTH.MAX".to_string(),
					"drugs.0.drugAuthorizationNumber".to_string()
				),
				(
					"ICH.G.k.3.2.LENGTH.MAX".to_string(),
					"drugs.0.drugAuthorizationCountry".to_string()
				),
				(
					"ICH.G.k.3.3.LENGTH.MAX".to_string(),
					"drugs.0.manufacturerName".to_string()
				),
				(
					"ICH.G.k.5a.LENGTH.MAX".to_string(),
					"drugs.0.cumulativeDoseFirstReactionValue".to_string()
				),
				(
					"ICH.G.k.5b.LENGTH.MAX".to_string(),
					"drugs.0.cumulativeDoseFirstReactionUnit".to_string()
				),
				(
					"ICH.G.k.6a.LENGTH.MAX".to_string(),
					"drugs.0.gestationPeriodExposureValue".to_string()
				),
				(
					"ICH.G.k.6b.LENGTH.MAX".to_string(),
					"drugs.0.gestationPeriodExposureUnit".to_string()
				),
				(
					"ICH.G.k.8.LENGTH.MAX".to_string(),
					"drugs.0.actionTaken".to_string()
				),
				(
					"ICH.G.k.10.r.LENGTH.MAX".to_string(),
					"drugs.0.drugAdditionalInformationCodes".to_string()
				),
				(
					"ICH.G.k.11.LENGTH.MAX".to_string(),
					"drugs.0.drugAdditionalInformation".to_string()
				),
			]
		);
	}

	#[test]
	fn max_length_rules_cover_g_nested_drug_collections() {
		let mut ctx = empty_ctx();
		let mut drug = drug();
		drug.id = Uuid::from_u128(1);
		ctx.drugs.push(drug);

		let mut substance = substance();
		substance.drug_id = Uuid::from_u128(1);
		substance.substance_name = Some("x".repeat(251));
		substance.substance_termid_version = Some("x".repeat(11));
		substance.substance_termid = Some("x".repeat(101));
		substance.strength_value = Some(Decimal::new(12_345_678_901, 0));
		substance.strength_unit = Some("x".repeat(51));
		ctx.active_substances.push(substance);

		let mut dosage = dosage();
		dosage.drug_id = Uuid::from_u128(1);
		dosage.dose_value = Some(Decimal::new(123_456_789, 0));
		dosage.dose_unit = Some("x".repeat(51));
		dosage.number_of_units = Some(Decimal::new(12_345, 0));
		dosage.frequency_unit = Some("x".repeat(51));
		dosage.duration_value = Some(Decimal::new(123_456, 0));
		dosage.duration_unit = Some("x".repeat(51));
		dosage.batch_lot_number = Some("x".repeat(36));
		dosage.dosage_text = Some("x".repeat(2001));
		dosage.dose_form = Some("x".repeat(61));
		dosage.dose_form_termid_version = Some("x".repeat(11));
		dosage.dose_form_termid = Some("x".repeat(101));
		dosage.route_of_administration = Some("x".repeat(61));
		dosage.route_termid_version = Some("x".repeat(11));
		dosage.route_termid = Some("x".repeat(101));
		dosage.parent_route = Some("x".repeat(61));
		dosage.parent_route_termid_version = Some("x".repeat(11));
		dosage.parent_route_termid = Some("x".repeat(101));
		ctx.dosages.push(dosage);

		let mut indication = indication();
		indication.drug_id = Uuid::from_u128(1);
		indication.indication_text = Some("x".repeat(251));
		indication.indication_meddra_version = Some("x".repeat(5));
		indication.indication_meddra_code = Some("x".repeat(9));
		ctx.indications.push(indication);

		let mut assessment = assessment();
		assessment.id = Uuid::from_u128(2);
		assessment.drug_id = Uuid::from_u128(1);
		assessment.administration_start_interval_value =
			Some(Decimal::new(123_456, 0));
		assessment.administration_start_interval_unit = Some("x".repeat(51));
		assessment.last_dose_interval_value = Some(Decimal::new(123_456, 0));
		assessment.last_dose_interval_unit = Some("x".repeat(51));
		assessment.reaction_recurred = Some("12".to_string());
		ctx.drug_reaction_assessments.push(assessment);

		let mut relatedness = relatedness();
		relatedness.drug_reaction_assessment_id = Uuid::from_u128(2);
		relatedness.source_of_assessment = Some("x".repeat(61));
		relatedness.method_of_assessment = Some("x".repeat(61));
		relatedness.result_of_assessment = Some("x".repeat(61));
		ctx.relatedness_assessments.push(relatedness);

		assert_eq!(length_issues(&ctx).len(), 33);
		assert!(length_issues(&ctx).contains(&(
			"ICH.G.k.2.3.r.1.LENGTH.MAX".to_string(),
			"drugs.0.activeSubstances.0.substanceName".to_string()
		)));
		assert!(length_issues(&ctx).contains(&(
			"ICH.G.k.4.r.1a.LENGTH.MAX".to_string(),
			"drugs.0.dosages.0.doseValue".to_string()
		)));
		assert!(length_issues(&ctx).contains(&(
			"ICH.G.k.7.r.2b.LENGTH.MAX".to_string(),
			"drugs.0.indications.0.indicationMeddraCode".to_string()
		)));
		assert!(length_issues(&ctx).contains(&(
			"ICH.G.k.9.i.4.LENGTH.MAX".to_string(),
			"drugs.0.reactionAssessments.0.reactionRecurred".to_string()
		)));
		assert!(length_issues(&ctx).contains(&(
			"ICH.G.k.9.i.2.r.1.LENGTH.MAX".to_string(),
			"drugs.0.reactionAssessments.0.relatednessAssessments.0.sourceOfAssessment"
				.to_string()
		)));
	}

	#[test]
	fn aggregate_identity_rules_cover_drug_and_ingredient_fallbacks() {
		let mut ctx = empty_ctx();
		let mut drug = drug();
		drug.id = Uuid::from_u128(1);
		drug.drug_characterization = "2".to_string();
		ctx.drugs.push(drug);

		let codes = codes_for(&ctx);
		assert!(codes.contains(&"ICH.G.k.1.AGGREGATE.REQUIRED".to_string()));
		assert!(codes.contains(&"ICH.G.k.2.3.r.REQUIRED".to_string()));

		ctx.drugs[0].drug_characterization = "1".to_string();
		let mut substance = substance();
		substance.drug_id = Uuid::from_u128(1);
		substance.substance_name = Some("ingredient".to_string());
		ctx.active_substances.push(substance);
		let codes = codes_for(&ctx);
		assert!(!codes.contains(&"ICH.G.k.1.AGGREGATE.REQUIRED".to_string()));
		assert!(!codes.contains(&"ICH.G.k.2.3.r.REQUIRED".to_string()));
	}

	#[test]
	fn blinded_product_is_limited_to_clinical_trials() {
		let mut value = drug();
		value.investigational_product_blinded = Some(true);
		let mut issues = Vec::new();
		g_k_2_5(0, &value, false, &Default::default(), &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "ICH.G.k.2.5.STUDY.ONLY"));

		issues.clear();
		g_k_2_5(0, &value, true, &Default::default(), &mut issues);
		assert!(!issues
			.iter()
			.any(|issue| issue.code == "ICH.G.k.2.5.STUDY.ONLY"));
	}

	#[test]
	fn mfds_identifier_versions_and_ids_are_paired() {
		let mut value = drug();
		value.mpid_version = Some("1".to_string());
		value.phpid = Some("PHPID".to_string());
		let mut issues = Vec::new();
		mfds_g_k_2_1_companions(0, &value, &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "MFDS.G.k.2.1.1b.REQUIRED"));
		assert!(issues
			.iter()
			.any(|issue| issue.code == "MFDS.G.k.2.1.2a.REQUIRED"));
	}
}
