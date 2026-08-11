use super::*;
use lib_core::model::reaction::ReactionBmc;

pub(crate) async fn export_patch(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	raw_xml: &[u8],
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	let reactions = ReactionBmc::list_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::from)?;
	crate::export::roundtrip::patch_e_reactions_for_authority(
		raw_xml, &reactions, authority,
	)
}

use crate::export::policy::{
	normalize_outcome_code, outcome_display_name,
	should_emit_required_intervention_null_flavor_ni,
};
use sqlx::types::time::Date;

pub fn export_e_reactions_xml(reactions: &[Reaction]) -> Result<String> {
	export_e_reactions_xml_for_authority(
		reactions,
		lib_core::regulatory::RegulatoryAuthority::Ich,
	)
}

pub fn export_e_reactions_xml_for_authority(
	reactions: &[Reaction],
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	let mut ordered: Vec<&Reaction> = reactions.iter().collect();
	ordered.sort_by_key(|reaction| reaction.sequence_number);

	let mut reactions_xml = String::new();
	for reaction in ordered {
		reactions_xml
			.push_str(&reaction_fragment_for_authority(reaction, authority)?);
	}
	let xml = base_e_reaction_skeleton().replace("{REACTIONS}", &reactions_xml);
	Ok(xml)
}

pub(crate) fn reaction_fragment_for_authority(
	reaction: &Reaction,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	let mut out = String::new();
	out.push_str("<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\">");
	out.push_str("<id root=\"");
	out.push_str(&xml_escape(&reaction.id.to_string()));
	out.push_str("\"/>");
	out.push_str(
		"<code code=\"29\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/>",
	);
	if reaction.start_date.is_some()
		|| reaction.start_date_null_flavor.is_some()
		|| reaction.end_date.is_some()
		|| reaction.end_date_null_flavor.is_some()
		|| reaction.duration_value.is_some()
	{
		let has_duration = reaction.duration_value.is_some();
		if has_duration {
			out.push_str("<effectiveTime xsi:type=\"SXPR_TS\">");
		} else {
			out.push_str("<effectiveTime xsi:type=\"IVL_TS\">");
		}
		write_e_i_4(
			&mut out,
			"low",
			reaction.start_date,
			reaction.start_date_null_flavor.as_deref(),
			has_duration,
		);
		write_e_i_5(
			&mut out,
			"high",
			reaction.end_date,
			reaction.end_date_null_flavor.as_deref(),
			has_duration,
		);
		if let Some(width) = write_e_i_6a(reaction) {
			out.push_str("<comp xsi:type=\"IVL_TS\" operator=\"A\"><width value=\"");
			out.push_str(&xml_escape(&width.to_string()));
			out.push_str("\"");
			if let Some(unit) = write_e_i_6b(reaction) {
				out.push_str(" unit=\"");
				out.push_str(&xml_escape(unit));
				out.push_str("\"");
			}
			out.push_str("/></comp>");
		}
		out.push_str("</effectiveTime>");
	}
	out.push_str("<value xsi:type=\"CE\"");
	if let Some(meddracode) = write_e_i_2_1b(reaction) {
		out.push_str(" code=\"");
		out.push_str(&xml_escape(meddracode));
		out.push_str("\" codeSystem=\"2.16.840.1.113883.6.163\"");
		if let Some(version) = write_e_i_2_1a(reaction) {
			out.push_str(" codeSystemVersion=\"");
			out.push_str(&xml_escape(version));
			out.push_str("\"");
		}
	} else {
		out.push_str(" nullFlavor=\"NI\"");
	}
	if let Some(text) = write_e_i_1_1a(reaction) {
		out.push_str("><originalText");
		if let Some(lang) = write_e_i_1_1b(reaction) {
			out.push_str(" language=\"");
			out.push_str(&xml_escape(lang));
			out.push_str("\"");
		}
		out.push_str(">");
		out.push_str(&xml_escape(text));
		out.push_str("</originalText></value>");
	} else {
		out.push_str("/>");
	}
	if let Some(country) = write_e_i_9(reaction) {
		let country = country.trim();
		if !country.is_empty() {
			out.push_str("<location typeCode=\"LOC\"><locatedEntity classCode=\"LOCE\"><locatedPlace classCode=\"COUNTRY\" determinerCode=\"INSTANCE\"><code code=\"");
			out.push_str(&xml_escape(country));
			out.push_str("\" codeSystem=\"1.0.3166.1.2.2\"/></locatedPlace></locatedEntity></location>");
		}
	}
	out.push_str(&write_e_i_1_2(reaction));
	if let Some(term_code) = write_e_i_3_1(reaction) {
		out.push_str(&observation_rel_code("37", term_code));
	}
	out.push_str(&write_e_i_3_2a(
		"34",
		reaction.criteria_death,
		reaction.criteria_death_null_flavor.as_deref(),
	));
	out.push_str(&write_e_i_3_2b(
		"21",
		reaction.criteria_life_threatening,
		reaction.criteria_life_threatening_null_flavor.as_deref(),
	));
	out.push_str(&write_e_i_3_2c(
		"33",
		reaction.criteria_hospitalization,
		reaction.criteria_hospitalization_null_flavor.as_deref(),
	));
	out.push_str(&write_e_i_3_2d(
		"35",
		reaction.criteria_disabling,
		reaction.criteria_disabling_null_flavor.as_deref(),
	));
	out.push_str(&write_e_i_3_2e(
		"12",
		reaction.criteria_congenital_anomaly,
		reaction.criteria_congenital_anomaly_null_flavor.as_deref(),
	));
	out.push_str(&write_e_i_3_2f(
		"26",
		reaction.criteria_other_medically_important,
		reaction
			.criteria_other_medically_important_null_flavor
			.as_deref(),
	));
	if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Fda) {
		out.push_str(&write_fda_e_i_3_2h(
			reaction.required_intervention,
			reaction.required_intervention_null_flavor.as_deref(),
		));
	}
	append_extension_code(
		&mut out,
		"AE_EXPECTEDNESS",
		reaction.expectedness.as_deref(),
	);
	append_extension_code(&mut out, "AE_SEVERITY", reaction.severity.as_deref());
	out.push_str(&write_e_i_7(reaction.outcome.as_deref()));
	if let Some(value) = write_e_i_8(reaction) {
		out.push_str(&observation_rel_bool("24", value));
	}
	out.push_str("</observation></subjectOf2>");
	Ok(out)
}

/// e2b:E.i.1.1a
fn write_e_i_1_1a(value: &Reaction) -> Option<&str> {
	value
		.primary_source_reaction
		.as_deref()
		.filter(|text| !text.trim().is_empty())
}

/// e2b:E.i.1.1b
fn write_e_i_1_1b(value: &Reaction) -> Option<&str> {
	value.reaction_language.as_deref()
}

/// e2b:E.i.1.2
fn write_e_i_1_2(value: &Reaction) -> String {
	observation_rel_translation(value)
}

/// e2b:E.i.2.1a
fn write_e_i_2_1a(value: &Reaction) -> Option<&str> {
	value.reaction_meddra_version.as_deref()
}

/// e2b:E.i.2.1b
fn write_e_i_2_1b(value: &Reaction) -> Option<&str> {
	value
		.reaction_meddra_code
		.as_deref()
		.map(str::trim)
		.filter(|code| !code.is_empty())
}

/// e2b:E.i.3.1
fn write_e_i_3_1(value: &Reaction) -> Option<&str> {
	value.term_highlighted.as_deref()
}

/// e2b:E.i.3.2a
fn write_e_i_3_2a(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:E.i.3.2b
fn write_e_i_3_2b(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:E.i.3.2c
fn write_e_i_3_2c(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:E.i.3.2d
fn write_e_i_3_2d(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:E.i.3.2e
fn write_e_i_3_2e(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:E.i.3.2f
fn write_e_i_3_2f(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	observation_rel_bool_or_null_flavor(code, value, null_flavor)
}

/// e2b:FDA.E.i.3.2h
fn write_fda_e_i_3_2h(value: Option<bool>, null_flavor: Option<&str>) -> String {
	observation_rel_required_intervention(value, null_flavor)
}

/// e2b:E.i.4
fn write_e_i_4(
	out: &mut String,
	tag: &str,
	date: Option<Date>,
	null_flavor: Option<&str>,
	has_duration: bool,
) {
	append_time_boundary_fragment(out, tag, date, null_flavor, has_duration);
}

/// e2b:E.i.5
fn write_e_i_5(
	out: &mut String,
	tag: &str,
	date: Option<Date>,
	null_flavor: Option<&str>,
	has_duration: bool,
) {
	append_time_boundary_fragment(out, tag, date, null_flavor, has_duration);
}

/// e2b:E.i.6a
fn write_e_i_6a(value: &Reaction) -> Option<&rust_decimal::Decimal> {
	value.duration_value.as_ref()
}

/// e2b:E.i.6b
fn write_e_i_6b(value: &Reaction) -> Option<&'static str> {
	let unit = value.duration_unit.as_deref()?;
	crate::mapping::fda::e_reaction::reaction_duration_unit_to_ucum(unit)
}

/// e2b:E.i.7
fn write_e_i_7(value: Option<&str>) -> String {
	observation_rel_outcome(value)
}

/// e2b:E.i.8
fn write_e_i_8(value: &Reaction) -> Option<bool> {
	value.medical_confirmation
}

/// e2b:E.i.9
fn write_e_i_9(value: &Reaction) -> Option<&str> {
	value.country_code.as_deref()
}

fn observation_rel_bool(code: &str, value: bool) -> String {
	let v = if value { "true" } else { "false" };
	format!(
		"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"{v}\"/></observation></outboundRelationship2>"
	)
}

fn append_extension_code(out: &mut String, code: &str, value: Option<&str>) {
	let Some(value) = value.map(str::trim).filter(|v| !v.is_empty()) else {
		return;
	};
	out.push_str("<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"");
	out.push_str(&xml_escape(code));
	out.push_str("\"/><value xsi:type=\"CE\" code=\"");
	out.push_str(&xml_escape(value));
	out.push_str("\"/></observation></outboundRelationship2>");
}

fn observation_rel_code(code: &str, value: &str) -> String {
	format!(
		"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"CE\" code=\"{}\"/></observation></outboundRelationship2>",
		xml_escape(value)
	)
}

fn observation_rel_bool_or_null_flavor(
	code: &str,
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	let value = value.filter(|value| *value);
	match (value, null_flavor) {
		(Some(value), None) => observation_rel_bool(code, value),
		(None, null_flavor) => format!(
			"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" nullFlavor=\"{}\"/></observation></outboundRelationship2>",
			xml_escape(null_flavor.unwrap_or("NI"))
		),
		(Some(_), Some(_)) => unreachable!(
			"database invariant forbids a reaction criterion value and NullFlavor together"
		),
	}
}

fn append_time_boundary_fragment(
	out: &mut String,
	tag: &str,
	date: Option<Date>,
	null_flavor: Option<&str>,
	has_duration: bool,
) {
	match (date, null_flavor) {
		(Some(value), _) => {
			if has_duration {
				out.push_str("<comp xsi:type=\"IVL_TS\" operator=\"A\"><");
				out.push_str(tag);
				out.push_str(" value=\"");
				out.push_str(&fmt_date(value));
				out.push_str("\"/></comp>");
			} else {
				out.push('<');
				out.push_str(tag);
				out.push_str(" value=\"");
				out.push_str(&fmt_date(value));
				out.push_str("\"/>");
			}
		}
		(None, Some(null_flavor)) => {
			if has_duration {
				out.push_str("<comp xsi:type=\"IVL_TS\" operator=\"A\"><");
				out.push_str(tag);
				out.push_str(" nullFlavor=\"");
				out.push_str(&xml_escape(null_flavor));
				out.push_str("\"/></comp>");
			} else {
				out.push('<');
				out.push_str(tag);
				out.push_str(" nullFlavor=\"");
				out.push_str(&xml_escape(null_flavor));
				out.push_str("\"/>");
			}
		}
		(None, None) => {}
	}
}

fn observation_rel_outcome(value: Option<&str>) -> String {
	let Some(code) = normalize_outcome_code(value) else {
		return "<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"27\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"CE\" nullFlavor=\"NI\"/></observation></outboundRelationship2>".to_string();
	};
	let display_name = outcome_display_name(code);
	format!(
		"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"27\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"CE\" code=\"{}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.11\" displayName=\"{}\"/></observation></outboundRelationship2>",
		xml_escape(code),
		xml_escape(display_name)
	)
}

fn observation_rel_required_intervention(
	value: Option<bool>,
	null_flavor: Option<&str>,
) -> String {
	if let Some(value) = value {
		let v = if value { "true" } else { "false" };
		return format!(
			"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"7\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.3\"/><value xsi:type=\"BL\" value=\"{v}\"/></observation></outboundRelationship2>"
		);
	}
	if let Some(null_flavor) = null_flavor {
		return format!(
			"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"7\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.3\"/><value xsi:type=\"BL\" nullFlavor=\"{}\"/></observation></outboundRelationship2>",
			xml_escape(null_flavor)
		);
	}
	if should_emit_required_intervention_null_flavor_ni() {
		"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"7\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.3\"/><value xsi:type=\"BL\" nullFlavor=\"NI\"/></observation></outboundRelationship2>".to_string()
	} else {
		"<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"7\" codeSystem=\"2.16.840.1.113883.3.989.5.1.2.2.1.3\"/><value xsi:type=\"BL\" value=\"true\"/></observation></outboundRelationship2>".to_string()
	}
}

#[cfg(test)]
mod split_null_flavor_tests {
	use super::observation_rel_required_intervention;

	#[test]
	fn exports_required_intervention_companion_as_boolean_null_flavor() {
		let xml = observation_rel_required_intervention(None, Some("NI"));
		assert!(xml.contains("xsi:type=\"BL\" nullFlavor=\"NI\""));
		assert!(!xml.contains(" value=\""));
	}
}

fn observation_rel_translation(reaction: &Reaction) -> String {
	let Some(text) = reaction
		.primary_source_reaction_translation
		.as_deref()
		.filter(|v| !v.trim().is_empty())
	else {
		return String::new();
	};
	let mut out = String::new();
	out.push_str("<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"30\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"ED\"");
	if let Some(lang) = reaction.reaction_language.as_deref() {
		out.push_str(" language=\"");
		out.push_str(&xml_escape(lang));
		out.push_str("\"");
	}
	out.push_str(">");
	out.push_str(&xml_escape(text));
	out.push_str("</value></observation></outboundRelationship2>");
	out
}

fn fmt_date(date: Date) -> String {
	format!(
		"{:04}{:02}{:02}",
		date.year(),
		u8::from(date.month()),
		date.day()
	)
}

fn xml_escape(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}

fn base_e_reaction_skeleton() -> &'static str {
	"<?xml version=\"1.0\" encoding=\"utf-8\"?>\
<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\" ITSVersion=\"XML_1.0\">\
\t<PORR_IN049016UV>\
\t\t<controlActProcess classCode=\"CACT\" moodCode=\"EVN\">\
\t\t\t<code code=\"PORR_TE049016UV\" codeSystem=\"2.16.840.1.113883.1.18\"/>\
\t\t\t<subject>\
\t\t\t\t<investigationEvent classCode=\"INVSTG\" moodCode=\"EVN\">\
\t\t\t\t\t<component typeCode=\"COMP\">\
\t\t\t\t\t\t<adverseEventAssessment classCode=\"INVSTG\" moodCode=\"EVN\">\
\t\t\t\t\t\t\t<subject1 typeCode=\"SBJ\">\
\t\t\t\t\t\t\t\t<primaryRole classCode=\"INVSBJ\">\
\t\t\t\t\t\t\t\t\t<player1 classCode=\"PSN\" determinerCode=\"INSTANCE\"><name/></player1>\
\t\t\t\t\t\t\t\t\t{REACTIONS}\
\t\t\t\t\t\t\t\t</primaryRole>\
\t\t\t\t\t\t\t</subject1>\
\t\t\t\t\t\t</adverseEventAssessment>\
\t\t\t\t\t</component>\
\t\t\t\t</investigationEvent>\
\t\t\t</subject>\
\t\t</controlActProcess>\
\t</PORR_IN049016UV>\
</MCCI_IN200100UV01>"
}

#[cfg(test)]
mod registry_coverage_tests {
	use std::collections::BTreeSet;

	#[test]
	fn section_e_writers_cover_registry_fields() {
		let registry: serde_json::Value = serde_json::from_str(include_str!(
			"../../../../../../registry/sections/e-reaction.json"
		))
		.expect("section E registry");
		let expected = registry
			.as_array()
			.expect("registry array")
			.iter()
			.filter(|entry| entry["local_only"] != true)
			.filter_map(|entry| entry["e2br3_code"].as_str())
			.collect::<BTreeSet<_>>();
		let implemented = include_str!("e.rs")
			.lines()
			.filter_map(|line| line.trim().strip_prefix("/// e2b:"))
			.collect::<BTreeSet<_>>();

		assert_eq!(implemented, expected);
	}
}

#[cfg(test)]
mod meddra_requirement_tests {
	use super::*;
	use sqlx::types::Uuid;
	use time::OffsetDateTime;

	fn reaction() -> Reaction {
		Reaction {
			id: Uuid::new_v4(),
			case_id: Uuid::new_v4(),
			sequence_number: 1,
			primary_source_reaction: Some("Headache".to_string()),
			primary_source_reaction_translation: None,
			reaction_language: Some("en".to_string()),
			reaction_meddra_version: Some("24.1".to_string()),
			reaction_meddra_code: Some("10019211".to_string()),
			term_highlighted: Some("4".to_string()),
			serious: Some(false),
			criteria_death: Some(false),
			criteria_death_null_flavor: None,
			criteria_life_threatening: Some(false),
			criteria_life_threatening_null_flavor: None,
			criteria_hospitalization: Some(false),
			criteria_hospitalization_null_flavor: None,
			criteria_disabling: Some(false),
			criteria_disabling_null_flavor: None,
			criteria_congenital_anomaly: Some(false),
			criteria_congenital_anomaly_null_flavor: None,
			criteria_other_medically_important: Some(false),
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
			outcome: Some("1".to_string()),
			medical_confirmation: Some(true),
			country_code: Some("US".to_string()),
			deleted: false,
			created_at: OffsetDateTime::now_utc(),
			updated_at: OffsetDateTime::now_utc(),
			created_by: Uuid::new_v4(),
			updated_by: None,
		}
	}

	#[test]
	fn export_uses_ni_for_missing_or_blank_meddra_code() {
		for code in [None, Some("  ".to_string())] {
			let mut reaction = reaction();
			reaction.reaction_meddra_code = code;

			let xml = export_e_reactions_xml(&[reaction])
				.expect("semantic MedDRA issue must not block export");
			assert!(xml.contains("<value xsi:type=\"CE\" nullFlavor=\"NI\""));
		}
	}

	#[test]
	fn export_omits_absent_reported_text_without_inventing_null_flavor() {
		let mut reaction = reaction();
		reaction.primary_source_reaction = None;
		reaction.primary_source_reaction_translation = None;
		reaction.reaction_language = None;

		let xml = export_e_reactions_xml(&[reaction]).expect("reaction XML");
		assert!(!xml.contains("<originalText"));
		assert!(!xml.contains("<code code=\"30\""));
		assert!(!xml.contains("<value xsi:type=\"CE\" nullFlavor="));
		assert!(xml.contains("code=\"10019211\""));
		assert!(xml.contains("codeSystemVersion=\"24.1\""));
	}

	#[test]
	fn exports_location_before_xsd_outbound_relationships() {
		let reaction = reaction();
		let xml = export_e_reactions_xml(std::slice::from_ref(&reaction))
			.expect("reaction XML");
		let value = xml.find("</originalText></value>").expect("reaction value");
		let location = xml
			.find("<location typeCode=\"LOC\"><locatedEntity classCode=\"LOCE\"><locatedPlace classCode=\"COUNTRY\" determinerCode=\"INSTANCE\"><code code=\"US\" codeSystem=\"1.0.3166.1.2.2\"/></locatedPlace></locatedEntity></location>")
			.expect("official E.i.9 location structure");
		let relationship = xml
			.find("<outboundRelationship2")
			.expect("reaction relationships");

		assert!(value < location && location < relationship);
		assert!(!xml.contains("<value xsi:type=\"BL\" value=\"false\"/>"));
		for code in ["34", "21", "33", "35", "12", "26"] {
			assert!(xml.contains(&format!(
				"<code code=\"{code}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" nullFlavor=\"NI\"/>"
			)));
		}

		let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.parent()
			.and_then(|path| path.parent())
			.and_then(|path| path.parent())
			.expect("workspace root")
			.to_path_buf();
		let source =
			std::fs::read(root.join("docs/exporter/fda/FAERS2022Scenario1.xml"))
				.expect("official FDA example");
		let exported = crate::export::roundtrip::patch_e_reactions(
			&source,
			std::slice::from_ref(&reaction),
		)
		.expect("patch official FDA example");
		let schema = crate::default_xsd_path().expect("official ICH schema");
		let errors =
			crate::validation::validate_e2b_xml_xsd(exported.as_bytes(), &schema)
				.expect("validate XSD");
		assert!(errors.is_empty(), "{errors:#?}");
	}

	#[test]
	fn exports_internal_duration_codes_as_ucum_and_omits_unknown_units() {
		let mut reaction = reaction();
		reaction.duration_value = Some(1.into());
		for (stored, ucum) in [
			("800", "10.a"),
			("801", "a"),
			("802", "mo"),
			("803", "wk"),
			("804", "d"),
			("805", "h"),
		] {
			reaction.duration_unit = Some(stored.to_string());
			let xml = export_e_reactions_xml(std::slice::from_ref(&reaction))
				.expect("supported reaction duration unit");
			assert!(xml.contains(&format!("<width value=\"1\" unit=\"{ucum}\"/>")));
		}

		reaction.duration_unit = Some("d".to_string());
		let xml = export_e_reactions_xml(&[reaction])
			.expect("semantic unit issue must not block export");
		assert!(xml.contains("<width value=\"1\"/>"));
		assert!(!xml.contains("unit=\"d\""));
	}
}
