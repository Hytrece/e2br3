use super::helpers::{
	e2b_datetime_date, max_length, reject_future_date, reject_when, require,
	valid_e2b_datetime, DateValues,
};
use crate::{has_text, RegulatoryAuthority, ValidationContext, ValidationIssue};
use lib_core::model::message_header::MessageHeader;
use lib_core::regulatory::{
	FDA_BATCH_RECEIVER_POSTMARKET, FDA_BATCH_RECEIVER_PREMARKET,
	FDA_MSG_RECEIVER_CBER_IND, FDA_MSG_RECEIVER_CDER, FDA_MSG_RECEIVER_CDER_IND,
	FDA_MSG_RECEIVER_CDER_IND_EXEMPT_BA_BE, MFDS_KNOWN_RECEIVERS,
};

const SECTION: &str = "case-identification";
const MAX_LENGTH_MESSAGE: &str = "Dictionary max length exceeded.";
const ALLOWED_VALUE_MESSAGE: &str = "Dictionary allowed values constraint.";

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
	let is_vaers = |value: &str| {
		matches!(
			value.trim().to_ascii_uppercase().as_str(),
			"CBER_VAERS" | "CBER VAERS"
		)
	};
	let vaers_message_receiver = is_vaers(message_receiver);
	let vaers_batch_receiver = batch_receiver.is_some_and(is_vaers);
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
			&& !vaers_message_receiver,
		"FDA.R0006",
		"messageHeader.messageReceiverIdentifier",
		"FDA postmarket N.2.r.3 must be CDER when N.1.4 is ZZFDA.",
	);
	push_business_violation(
		issues,
		(vaers_message_receiver || vaers_batch_receiver)
			&& batch_receiver != Some(message_receiver),
		"FDA.VAERS.N.ROUTE.PAIR",
		"messageHeader.batchReceiverIdentifier",
		"VAERS N.1.4 and N.2.r.3 must use the same CBER VAERS or CBER_VAERS identifier.",
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

/// MFDS.N.1.4.ALLOWED.VALUE
/// MFDS.N.2.r.3.ALLOWED.VALUE
/// MFDS.N.ROUTE.PAIR
fn mfds_n_routing(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	let batch_receiver = trimmed(header.batch_receiver_identifier.as_deref());
	let message_receiver = trimmed(Some(&header.message_receiver_identifier));
	push_business_violation(
		issues,
		batch_receiver.is_some_and(|value| !MFDS_KNOWN_RECEIVERS.contains(&value)),
		"MFDS.N.1.4.ALLOWED.VALUE",
		"messageHeader.batchReceiverIdentifier",
		"MFDS N.1.4 must use an official MFDS operational or test receiver identifier.",
	);
	push_business_violation(
		issues,
		message_receiver.is_some_and(|value| !MFDS_KNOWN_RECEIVERS.contains(&value)),
		"MFDS.N.2.r.3.ALLOWED.VALUE",
		"messageHeader.messageReceiverIdentifier",
		"MFDS N.2.r.3 must use an official MFDS operational or test receiver identifier.",
	);
	push_business_violation(
		issues,
		matches!((batch_receiver, message_receiver), (Some(batch), Some(message)) if batch != message),
		"MFDS.N.ROUTE.PAIR",
		"messageHeader.messageReceiverIdentifier",
		"MFDS N.1.4 and N.2.r.3 must use the same receiver identifier.",
	);
}

/// ICH.N.1.1.ALLOWED.VALUE
/// ICH.N.1.1.LENGTH.MAX
fn n_1_1(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageType";
	let code = message_type_code(header).map(str::trim);
	reject_when(
		issues,
		"ICH.N.1.1.ALLOWED.VALUE",
		PATH,
		SECTION,
		ALLOWED_VALUE_MESSAGE,
		code.is_some_and(|code| !code.is_empty() && code != "1"),
	);
	max_length(
		issues,
		"ICH.N.1.1.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		message_type_code(header),
		2,
	);
}

/// ICH.N.1.2.REQUIRED
/// ICH.N.1.2.LENGTH.MAX
fn n_1_2(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchNumber";
	require(
		issues,
		"ICH.N.1.2.REQUIRED",
		PATH,
		SECTION,
		"[N.1.2] Batch number is required.",
		has_text(header.batch_number.as_deref()),
	);
	max_length(
		issues,
		"ICH.N.1.2.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		header.batch_number.as_deref(),
		100,
	);
}

/// ICH.N.1.3.REQUIRED
/// ICH.N.1.3.LENGTH.MAX
fn n_1_3(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchSenderIdentifier";
	require(
		issues,
		"ICH.N.1.3.REQUIRED",
		PATH,
		SECTION,
		"[N.1.3] Batch sender identifier is required.",
		has_text(header.batch_sender_identifier.as_deref()),
	);
	max_length(
		issues,
		"ICH.N.1.3.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		header.batch_sender_identifier.as_deref(),
		60,
	);
}

/// ICH.N.1.4.REQUIRED
/// ICH.N.1.4.LENGTH.MAX
fn n_1_4(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchReceiverIdentifier";
	require(
		issues,
		"ICH.N.1.4.REQUIRED",
		PATH,
		SECTION,
		"[N.1.4] Batch receiver identifier is required.",
		has_text(header.batch_receiver_identifier.as_deref()),
	);
	max_length(
		issues,
		"ICH.N.1.4.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		header.batch_receiver_identifier.as_deref(),
		60,
	);
}

/// ICH.N.1.5.REQUIRED
/// ICH.N.1.5.FUTURE_DATE.FORBIDDEN
fn n_1_5(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.batchTransmissionDate";
	require(
		issues,
		"ICH.N.1.5.REQUIRED",
		PATH,
		SECTION,
		"[N.1.5] Date of batch transmission is required.",
		header.batch_transmission_date.is_some(),
	);
	reject_future_date(
		issues,
		"ICH.N.1.5.FUTURE_DATE.FORBIDDEN",
		PATH,
		SECTION,
		"[N.1.5] Date of batch transmission must not be later than today.",
		DateValues::One(header.batch_transmission_date.map(|value| value.date())),
	);
}

/// ICH.N.2.r.1.LENGTH.MAX
fn n_2_r_1(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageNumber";
	max_length(
		issues,
		"ICH.N.2.r.1.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		Some(header.message_number.as_str()),
		100,
	);
}

/// ICH.N.2.r.2.REQUIRED
/// ICH.N.2.r.2.LENGTH.MAX
fn n_2_r_2(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageSenderIdentifier";
	require(
		issues,
		"ICH.N.2.r.2.REQUIRED",
		PATH,
		SECTION,
		"[N.2.r.2] Message sender identifier is required.",
		has_text(Some(header.message_sender_identifier.as_str())),
	);
	max_length(
		issues,
		"ICH.N.2.r.2.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		Some(header.message_sender_identifier.as_str()),
		60,
	);
}

/// ICH.N.2.r.3.REQUIRED
/// ICH.N.2.r.3.LENGTH.MAX
fn n_2_r_3(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageReceiverIdentifier";
	require(
		issues,
		"ICH.N.2.r.3.REQUIRED",
		PATH,
		SECTION,
		"[N.2.r.3] Message receiver identifier is required.",
		has_text(Some(header.message_receiver_identifier.as_str())),
	);
	max_length(
		issues,
		"ICH.N.2.r.3.LENGTH.MAX",
		PATH,
		SECTION,
		MAX_LENGTH_MESSAGE,
		Some(header.message_receiver_identifier.as_str()),
		60,
	);
}

/// ICH.N.2.r.4.ALLOWED.VALUE
/// ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN
fn n_2_r_4(header: &MessageHeader, issues: &mut Vec<ValidationIssue>) {
	const PATH: &str = "messageHeader.messageDate";
	let value = header.message_date.trim();
	reject_when(
		issues,
		"ICH.N.2.r.4.ALLOWED.VALUE",
		PATH,
		SECTION,
		ALLOWED_VALUE_MESSAGE,
		!value.is_empty() && !valid_e2b_datetime(value),
	);
	reject_future_date(
		issues,
		"ICH.N.2.r.4.FUTURE_DATE.FORBIDDEN",
		PATH,
		SECTION,
		"[N.2.r.4] Message date must not be later than today.",
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
		} else if authority == RegulatoryAuthority::Mfds {
			mfds_n_routing(header, issues);
		}
	}
}

pub(crate) fn collect_ich_issues(
	validation_ctx: &ValidationContext,
	issues: &mut Vec<ValidationIssue>,
) {
	if let Some(header) = validation_ctx.message_header.as_ref() {
		n_1_1(header, issues);
		n_1_2(header, issues);
		n_1_3(header, issues);
		n_1_4(header, issues);
		n_1_5(header, issues);
		n_2_r_1(header, issues);
		n_2_r_2(header, issues);
		n_2_r_3(header, issues);
		n_2_r_4(header, issues);
	}
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
	fn mfds_routing_accepts_only_matching_official_identifiers() {
		let mut header = message_header();
		header.batch_receiver_identifier = Some("MFDS-O-CT".to_string());
		header.message_receiver_identifier = "MFDS-O-CT".to_string();
		let mut issues = Vec::new();
		mfds_n_routing(&header, &mut issues);
		assert!(issues.is_empty());

		header.message_receiver_identifier = "CT".to_string();
		mfds_n_routing(&header, &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "MFDS.N.2.r.3.ALLOWED.VALUE"));
		assert!(issues.iter().any(|issue| issue.code == "MFDS.N.ROUTE.PAIR"));
	}

	#[test]
	fn fda_vaers_receiver_requires_the_matching_esg_identifier() {
		let mut header = message_header();
		header.message_receiver_identifier = "CBER_VAERS".to_string();
		header.batch_receiver_identifier = Some("CBER VAERS".to_string());
		let mut issues = Vec::new();
		fda_n_routing(&header, &mut issues);
		assert!(issues
			.iter()
			.any(|issue| issue.code == "FDA.VAERS.N.ROUTE.PAIR"));

		header.batch_receiver_identifier = Some("CBER_VAERS".to_string());
		issues.clear();
		fda_n_routing(&header, &mut issues);
		assert!(!issues
			.iter()
			.any(|issue| issue.code == "FDA.VAERS.N.ROUTE.PAIR"));
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

		for value in ["", "   ", " 1 "] {
			ctx.message_header.as_mut().unwrap().message_type = value.to_string();
			issues.clear();
			collect_ich_issues(&ctx, &mut issues);
			assert!(!issues
				.iter()
				.any(|issue| issue.code == "ICH.N.1.1.ALLOWED.VALUE"));
		}
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

	#[test]
	fn golden_n_issue_metadata() {
		let mut ctx = empty_ctx();
		let mut header = message_header();
		header.message_type = "other".to_string();
		header.batch_number = None;
		header.message_sender_identifier = "S".repeat(61);
		ctx.message_header = Some(header);

		let mut issues = Vec::new();
		collect_ich_issues(&ctx, &mut issues);
		let mut out = issues
			.into_iter()
			.filter(|issue| {
				matches!(
					issue.code.as_str(),
					"ICH.N.1.1.ALLOWED.VALUE"
						| "ICH.N.1.2.REQUIRED"
						| "ICH.N.2.r.2.LENGTH.MAX"
				)
			})
			.map(|issue| {
				(
					issue.code,
					issue.message,
					issue.path,
					issue.field_path,
					issue.section,
					issue.subsection,
					issue.blocking,
				)
			})
			.collect::<Vec<_>>();
		out.sort_by(|left, right| left.0.cmp(&right.0));

		assert_eq!(
			out,
			vec![
				(
					"ICH.N.1.1.ALLOWED.VALUE".to_string(),
					"Dictionary allowed values constraint.".to_string(),
					"messageHeader.messageType".to_string(),
					Some("messageHeader.messageType".to_string()),
					"case-identification".to_string(),
					"N".to_string(),
					true,
				),
				(
					"ICH.N.1.2.REQUIRED".to_string(),
					"[N.1.2] Batch number is required.".to_string(),
					"messageHeader.batchNumber".to_string(),
					Some("messageHeader.batchNumber".to_string()),
					"case-identification".to_string(),
					"N".to_string(),
					true,
				),
				(
					"ICH.N.2.r.2.LENGTH.MAX".to_string(),
					"Dictionary max length exceeded.".to_string(),
					"messageHeader.messageSenderIdentifier".to_string(),
					Some("messageHeader.messageSenderIdentifier".to_string()),
					"case-identification".to_string(),
					"N".to_string(),
					true,
				),
			],
		);
	}
}
