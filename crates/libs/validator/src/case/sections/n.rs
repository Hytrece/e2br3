use super::helpers::{
	e2b_datetime_date, validate_constraint, validate_future_date, validate_length,
	validate_value, DateValues, RuleValue,
};
use crate::allowed_value::ConstraintValue;
use crate::{RegulatoryAuthority, RuleFacts, ValidationContext, ValidationIssue};
use lib_core::model::message_header::MessageHeader;
use lib_core::regulatory::{
	FDA_BATCH_RECEIVER_POSTMARKET, FDA_BATCH_RECEIVER_PREMARKET,
	FDA_MSG_RECEIVER_CBER_IND, FDA_MSG_RECEIVER_CDER, FDA_MSG_RECEIVER_CDER_IND,
	FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE,
};
use std::borrow::Cow;

fn message_type_code(header: &MessageHeader) -> Option<&str> {
	Some(if header.message_type == "ichicsr" {
		"1"
	} else {
		header.message_type.as_str()
	})
}

fn trimmed(value: Option<&str>) -> Option<&str> {
	value.map(str::trim).filter(|value| !value.is_empty())
}

fn push_business_violation(
	issues: &mut Vec<ValidationIssue>,
	violated: bool,
	code: &str,
	path: &str,
	message: &str,
) {
	if violated {
		crate::push_business_issue(issues, code, path, message);
	}
}

/// ICH.N.2.r.1.MATCH.C.1.1
fn n_2_r_1_matches_c_1_1(
	header: &MessageHeader,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let safety_report_id = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|report| trimmed(report.safety_report_id.as_deref()));
	push_business_violation(
		issues,
		safety_report_id.is_some_and(|value| value != header.message_number.trim()),
		"ICH.N.2.r.1.MATCH.C.1.1",
		"messageHeader.messageNumber",
		"N.2.r.1 must be identical to C.1.1.",
	);
}

/// ICH.C.1.2.MATCH.N.2.r.4
fn c_1_2_matches_n_2_r_4(
	header: &MessageHeader,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	let transmission_date = validation_ctx
		.safety_report
		.as_ref()
		.and_then(|report| trimmed(report.transmission_date.as_deref()));
	push_business_violation(
		issues,
		transmission_date.is_some_and(|value| value != header.message_date.trim()),
		"ICH.C.1.2.MATCH.N.2.r.4",
		"safetyReportIdentification.transmissionDate",
		"C.1.2 must be identical to N.2.r.4.",
	);
}

/// FDA.R0004
/// FDA.R0005
/// FDA.R0006
/// FDA.R0007
/// FDA.R0100
fn fda_n_routing(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	let batch_receiver = trimmed(header.batch_receiver_identifier.as_deref());
	let message_receiver = header.message_receiver_identifier.trim();
	let vaers_receiver = matches!(
		message_receiver.to_ascii_uppercase().as_str(),
		"CBER_VAERS" | "CBER VAERS"
	);
	let premarket_receiver = matches!(
		message_receiver,
		FDA_MSG_RECEIVER_CDER_IND
			| FDA_MSG_RECEIVER_CBER_IND
			| FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE
	);
	push_business_violation(
		issues,
		message_receiver == FDA_MSG_RECEIVER_CDER
			&& batch_receiver != Some(FDA_BATCH_RECEIVER_POSTMARKET),
		"FDA.R0004",
		"messageHeader.batchReceiverIdentifier",
		"FDA postmarket N.1.4 must be ZZFDA.",
	);
	push_business_violation(
		issues,
		premarket_receiver && batch_receiver != Some(FDA_BATCH_RECEIVER_PREMARKET),
		"FDA.R0005",
		"messageHeader.batchReceiverIdentifier",
		"FDA premarket N.1.4 must be ZZFDA_PREMKT.",
	);
	push_business_violation(
		issues,
		batch_receiver == Some(FDA_BATCH_RECEIVER_POSTMARKET)
			&& message_receiver != FDA_MSG_RECEIVER_CDER
			&& !vaers_receiver,
		"FDA.R0006",
		"messageHeader.messageReceiverIdentifier",
		"FDA postmarket N.2.r.3 must be CDER when N.1.4 is ZZFDA.",
	);
	push_business_violation(
		issues,
		vaers_receiver && batch_receiver != Some(FDA_BATCH_RECEIVER_POSTMARKET),
		"FDA.R0004",
		"messageHeader.batchReceiverIdentifier",
		"FDA VAERS N.1.4 must be ZZFDA.",
	);
	push_business_violation(
		issues,
		batch_receiver == Some(FDA_BATCH_RECEIVER_PREMARKET)
			&& !premarket_receiver,
		"FDA.R0007",
		"messageHeader.messageReceiverIdentifier",
		"FDA premarket N.2.r.3 must be CDER_IND, CBER_IND, or CDER_IND_EXEMPT_BA_BE.",
	);
	push_business_violation(
		issues,
		trimmed(header.batch_sender_identifier.as_deref())
			.is_some_and(|sender| sender != header.message_sender_identifier.trim()),
		"FDA.R0100",
		"messageHeader.messageSenderIdentifier",
		"FDA N.2.r.2 must match N.1.3.",
	);
}

/// ICH.N.REQUIRED
fn n(header: Option<&MessageHeader>, issues: &mut Vec<ValidationIssue>) {
	validate_value(
		issues,
		"ICH.N.REQUIRED",
		"messageHeader",
		RuleValue::borrowed(header.map(|_| "present"), None),
		RuleFacts::default(),
	);
}

/// ICH.N.1.1.REQUIRED
/// ICH.N.1.1.ALLOWED.VALUE
/// ICH.N.1.1.LENGTH.MAX
fn n_1_1(
	header: &MessageHeader,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "messageHeader.messageType";
	validate_value(
		issues,
		"ICH.N.1.1.REQUIRED",
		PATH,
		RuleValue::borrowed(Some(header.message_type.as_str()), None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.N.1.1.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(message_type_code(header).map(Cow::Borrowed)),
		&validation_ctx.vocabulary,
	);
	validate_length(
		issues,
		"ICH.N.1.1.LENGTH.MAX",
		PATH,
		message_type_code(header),
	);
}

/// ICH.N.1.2.REQUIRED
/// ICH.N.1.2.LENGTH.MAX
fn n_1_2(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchNumber";
	validate_value(
		issues,
		"ICH.N.1.2.REQUIRED",
		PATH,
		RuleValue::borrowed(header.batch_number.as_deref(), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.1.2.LENGTH.MAX",
		PATH,
		header.batch_number.as_deref(),
	);
}

/// ICH.N.1.3.REQUIRED
/// ICH.N.1.3.LENGTH.MAX
fn n_1_3(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchSenderIdentifier";
	validate_value(
		issues,
		"ICH.N.1.3.REQUIRED",
		PATH,
		RuleValue::borrowed(header.batch_sender_identifier.as_deref(), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.1.3.LENGTH.MAX",
		PATH,
		header.batch_sender_identifier.as_deref(),
	);
}

/// ICH.N.1.4.REQUIRED
/// ICH.N.1.4.LENGTH.MAX
fn n_1_4(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchReceiverIdentifier";
	validate_value(
		issues,
		"ICH.N.1.4.REQUIRED",
		PATH,
		RuleValue::borrowed(header.batch_receiver_identifier.as_deref(), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.1.4.LENGTH.MAX",
		PATH,
		header.batch_receiver_identifier.as_deref(),
	);
}

/// ICH.N.1.5.REQUIRED
/// ICH.N.1.5.FUTURE_DATE.FORBIDDEN
fn n_1_5(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchTransmissionDate";
	validate_value(
		issues,
		"ICH.N.1.5.REQUIRED",
		PATH,
		RuleValue::borrowed(
			if header.batch_transmission_date.is_some() {
				Some("1")
			} else {
				None
			},
			None,
		),
		RuleFacts::default(),
	);
	validate_future_date(
		issues,
		"ICH.N.1.5.FUTURE_DATE.FORBIDDEN",
		PATH,
		DateValues::One(header.batch_transmission_date.map(|value| value.date())),
	);
}

/// ICH.N.2.r.1.REQUIRED
/// ICH.N.2.r.1.LENGTH.MAX
fn n_2_r_1(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageNumber";
	validate_value(
		issues,
		"ICH.N.2.r.1.REQUIRED",
		PATH,
		RuleValue::borrowed(Some(header.message_number.as_str()), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.2.r.1.LENGTH.MAX",
		PATH,
		Some(header.message_number.as_str()),
	);
}

/// ICH.N.2.r.2.REQUIRED
/// ICH.N.2.r.2.LENGTH.MAX
fn n_2_r_2(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageSenderIdentifier";
	validate_value(
		issues,
		"ICH.N.2.r.2.REQUIRED",
		PATH,
		RuleValue::borrowed(Some(header.message_sender_identifier.as_str()), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.2.r.2.LENGTH.MAX",
		PATH,
		Some(header.message_sender_identifier.as_str()),
	);
}

/// ICH.N.2.r.3.REQUIRED
/// ICH.N.2.r.3.LENGTH.MAX
fn n_2_r_3(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageReceiverIdentifier";
	validate_value(
		issues,
		"ICH.N.2.r.3.REQUIRED",
		PATH,
		RuleValue::borrowed(Some(header.message_receiver_identifier.as_str()), None),
		RuleFacts::default(),
	);
	validate_length(
		issues,
		"ICH.N.2.r.3.LENGTH.MAX",
		PATH,
		Some(header.message_receiver_identifier.as_str()),
	);
}

/// ICH.N.2.r.4.REQUIRED
/// ICH.N.2.r.4.ALLOWED.VALUE
/// ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN
fn n_2_r_4(
	header: &MessageHeader,
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	const PATH: &str = "messageHeader.messageDate";
	validate_value(
		issues,
		"ICH.N.2.r.4.REQUIRED",
		PATH,
		RuleValue::borrowed(Some(header.message_date.as_str()), None),
		RuleFacts::default(),
	);
	validate_constraint(
		issues,
		"ICH.N.2.r.4.ALLOWED.VALUE",
		PATH,
		ConstraintValue::Text(Some(Cow::Borrowed(header.message_date.as_str()))),
		&validation_ctx.vocabulary,
	);
	validate_future_date(
		issues,
		"ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN",
		PATH,
		DateValues::One(e2b_datetime_date(Some(header.message_date.as_str()))),
	);
}

pub(crate) fn collect(
	issues: &mut Vec<ValidationIssue>,
	authority: RegulatoryAuthority,
	validation_ctx: &ValidationContext,
) {
	collect_ich_issues(validation_ctx, issues);
	if let Some(header) = validation_ctx.message_header.as_ref() {
		n_2_r_1_matches_c_1_1(header, validation_ctx, issues);
		c_1_2_matches_n_2_r_4(header, validation_ctx, issues);
		if authority == RegulatoryAuthority::Fda {
			fda_n_routing(header, issues);
		}
	}
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	n(validation_ctx.message_header.as_ref(), issues);
	if let Some(header) = validation_ctx.message_header.as_ref() {
		n_1_1(header, validation_ctx, issues);
		n_1_2(header, issues);
		n_1_3(header, issues);
		n_1_4(header, issues);
		n_1_5(header, issues);
		n_2_r_1(header, issues);
		n_2_r_2(header, issues);
		n_2_r_3(header, issues);
		n_2_r_4(header, validation_ctx, issues);
	}
}

#[cfg(test)]
pub(super) fn constraint_rule_codes() -> Vec<&'static str> {
	vec!["ICH.N.1.1.ALLOWED.VALUE", "ICH.N.2.r.4.ALLOWED.VALUE"]
}

#[cfg(test)]
pub(super) fn implemented_rule_codes() -> Vec<&'static str> {
	vec![
		"ICH.N.REQUIRED",
		"ICH.N.1.1.REQUIRED",
		"ICH.N.1.1.ALLOWED.VALUE",
		"ICH.N.1.1.LENGTH.MAX",
		"ICH.N.1.2.REQUIRED",
		"ICH.N.1.2.LENGTH.MAX",
		"ICH.N.1.3.REQUIRED",
		"ICH.N.1.3.LENGTH.MAX",
		"ICH.N.1.4.REQUIRED",
		"ICH.N.1.4.LENGTH.MAX",
		"ICH.N.1.5.REQUIRED",
		"ICH.N.1.5.FUTURE_DATE.FORBIDDEN",
		"ICH.N.2.r.1.REQUIRED",
		"ICH.N.2.r.1.LENGTH.MAX",
		"ICH.N.2.r.2.REQUIRED",
		"ICH.N.2.r.2.LENGTH.MAX",
		"ICH.N.2.r.3.REQUIRED",
		"ICH.N.2.r.3.LENGTH.MAX",
		"ICH.N.2.r.4.REQUIRED",
		"ICH.N.2.r.4.ALLOWED.VALUE",
		"ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN",
	]
}

#[cfg(test)]
mod tests {
	use super::*;
	use lib_core::model::case::Case;
	use lib_core::model::safety_report::SafetyReportIdentification;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Uuid;
	use time::Duration;

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

	fn message_header() -> MessageHeader {
		MessageHeader {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			batch_number: Some("batch".to_string()),
			batch_sender_identifier: Some("sender".to_string()),
			batch_receiver_identifier: Some("receiver".to_string()),
			batch_transmission_date: None,
			message_type: "ichicsr".to_string(),
			message_format_version: "2.1".to_string(),
			message_format_release: "2.0".to_string(),
			message_number: "msg-1".to_string(),
			message_sender_identifier: "sender".to_string(),
			message_receiver_identifier: "receiver".to_string(),
			message_date_format: "204".to_string(),
			message_date: "20200101000000".to_string(),
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	fn safety_report() -> SafetyReportIdentification {
		SafetyReportIdentification {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			safety_report_id: Some("US-SENDER-1".to_string()),
			version: 1,
			transmission_date: Some("20200102000000".to_string()),
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

	#[test]
	fn cross_section_identifiers_and_dates_must_match() {
		let mut ctx = empty_ctx();
		ctx.safety_report = Some(safety_report());
		ctx.message_header = Some(message_header());

		let mut issues = Vec::new();
		collect(&mut issues, RegulatoryAuthority::Ich, &ctx);

		assert!(issues
			.iter()
			.any(|issue| issue.code == "ICH.N.2.r.1.MATCH.C.1.1"));
		assert!(issues
			.iter()
			.any(|issue| issue.code == "ICH.C.1.2.MATCH.N.2.r.4"));
	}

	#[test]
	fn fda_routing_checks_batch_receiver_and_sender_pair() {
		let mut header = message_header();
		header.batch_receiver_identifier =
			Some(FDA_BATCH_RECEIVER_PREMARKET.to_string());
		header.message_receiver_identifier = FDA_MSG_RECEIVER_CDER.to_string();
		header.batch_sender_identifier = Some("batch-sender".to_string());
		let mut issues = Vec::new();

		fda_n_routing(&header, &mut issues);

		assert!(issues.iter().any(|issue| issue.code == "FDA.R0007"));
		assert!(issues.iter().any(|issue| issue.code == "FDA.R0100"));
	}

	#[test]
	fn fda_vaers_receiver_requires_postmarket_batch_route() {
		let mut header = message_header();
		header.message_receiver_identifier = "CBER_VAERS".to_string();
		header.batch_receiver_identifier =
			Some(FDA_BATCH_RECEIVER_PREMARKET.to_string());
		let mut issues = Vec::new();
		fda_n_routing(&header, &mut issues);
		assert!(issues.iter().any(|issue| issue.code == "FDA.R0004"));
	}

	#[test]
	fn missing_header_is_deferred_to_export_validation() {
		let ctx = empty_ctx();
		let mut issues = Vec::new();

		collect_ich_issues(&ctx, &mut issues);

		assert!(!issues.iter().any(|issue| issue.code == "ICH.N.REQUIRED"));
	}

	#[test]
	fn future_date_rules_cover_n_date_time_fields() {
		let mut ctx = empty_ctx();
		let mut header = message_header();
		header.batch_transmission_date =
			Some(OffsetDateTime::now_utc() + Duration::days(1));
		header.message_date = "29990101000000".to_string();
		ctx.message_header = Some(header);

		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		let mut out = issues
			.into_iter()
			.filter(|issue| issue.code.contains(".FUTURE_DATE."))
			.map(|issue| (issue.code, issue.path))
			.collect::<Vec<_>>();
		out.sort();

		assert_eq!(
			out,
			vec![
				(
					"ICH.N.1.5.FUTURE_DATE.FORBIDDEN".to_string(),
					"messageHeader.batchTransmissionDate".to_string()
				),
				(
					"ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN".to_string(),
					"messageHeader.messageDate".to_string()
				),
			]
		);
	}

	#[test]
	fn allowed_value_rule_uses_official_message_type_code() {
		let mut ctx = empty_ctx();
		ctx.message_header = Some(message_header());

		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		assert!(!issues
			.iter()
			.any(|issue| issue.code == "ICH.N.1.1.ALLOWED.VALUE"));

		ctx.message_header.as_mut().unwrap().message_type = "other".to_string();
		issues.clear();
		collect_ich_issues(&ctx, &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "ICH.N.1.1.ALLOWED.VALUE"));
	}

	#[test]
	fn datetime_format_rule_flags_invalid_message_date() {
		let mut ctx = empty_ctx();
		let mut header = message_header();
		header.message_date = "not-a-date".to_string();
		ctx.message_header = Some(header);

		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "ICH.N.2.r.4.ALLOWED.VALUE"));
	}

	#[test]
	fn max_length_rules_cover_n_text_fields() {
		let mut ctx = empty_ctx();
		let mut header = message_header();
		header.message_type = "ABC".to_string();
		header.batch_number = Some("B".repeat(101));
		header.batch_sender_identifier = Some("S".repeat(61));
		header.batch_receiver_identifier = Some("R".repeat(61));
		header.message_number = "M".repeat(101);
		header.message_sender_identifier = "S".repeat(61);
		header.message_receiver_identifier = "R".repeat(61);
		ctx.message_header = Some(header);

		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		let mut out = issues
			.into_iter()
			.filter(|issue| issue.code.contains(".LENGTH.MAX"))
			.map(|issue| (issue.code, issue.path))
			.collect::<Vec<_>>();
		out.sort();

		assert_eq!(
			out,
			vec![
				(
					"ICH.N.1.1.LENGTH.MAX".to_string(),
					"messageHeader.messageType".to_string()
				),
				(
					"ICH.N.1.2.LENGTH.MAX".to_string(),
					"messageHeader.batchNumber".to_string()
				),
				(
					"ICH.N.1.3.LENGTH.MAX".to_string(),
					"messageHeader.batchSenderIdentifier".to_string()
				),
				(
					"ICH.N.1.4.LENGTH.MAX".to_string(),
					"messageHeader.batchReceiverIdentifier".to_string()
				),
				(
					"ICH.N.2.r.1.LENGTH.MAX".to_string(),
					"messageHeader.messageNumber".to_string()
				),
				(
					"ICH.N.2.r.2.LENGTH.MAX".to_string(),
					"messageHeader.messageSenderIdentifier".to_string()
				),
				(
					"ICH.N.2.r.3.LENGTH.MAX".to_string(),
					"messageHeader.messageReceiverIdentifier".to_string()
				),
			]
		);
	}
}
