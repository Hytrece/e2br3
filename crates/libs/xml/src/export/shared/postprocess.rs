use super::*;
use crate::export::sections::c::apply_literature_section;
use crate::export::sections::c::apply_primary_source_section;
use crate::export::sections::c::apply_report_relationships_section;
use crate::export::sections::c::apply_study_section;
use crate::export::sections::h::{
	apply_case_summary_section, apply_sender_diagnosis_section,
};
use crate::export::sections::n::apply_section_n;
use crate::export::shared::patch_doc::postprocess_export_doc;

fn normalize_namespace_artifacts(mut xml: String) -> String {
	xml = xml.replace("xmlns:default=\"urn:hl7-org:v3\"", "");
	xml = xml.replace("xmlns:default=\"urn:hl7-org:v3\" ", "");
	xml = xml.replace("<default:", "<");
	xml = xml.replace("</default:", "</");
	for ty in ["BL", "CE", "ED", "IVL_TS", "PQ", "ST", "TS"] {
		xml = xml.replace(
			&format!(" type=\"{ty}\" xsi:type=\"{ty}\""),
			&format!(" xsi:type=\"{ty}\""),
		);
		xml =
			xml.replace(&format!(" type=\"{ty}\""), &format!(" xsi:type=\"{ty}\""));
	}
	xml
}

pub(crate) async fn apply_section_postprocess(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xml: String,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	let parser = Parser::default();
	let mut doc = parser.parse_string(&xml).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error (patched): {err}"),
		line: None,
		column: None,
	})?;
	let mut xpath = Context::new(&doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath context".to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ =
		xpath.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");
	apply_section_n(ctx, &mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_patient_section(
		ctx, &mut doc, &parser, mm, case_id, &mut xpath, authority,
	)
	.await?;
	apply_primary_source_section(
		&mut doc, &parser, mm, case_id, &mut xpath, authority,
	)
	.await?;
	apply_report_relationships_section(&mut doc, &parser, mm, case_id, &mut xpath)
		.await?;
	apply_literature_section(&mut doc, &parser, mm, case_id, &mut xpath).await?;
	apply_study_section(&mut doc, &parser, mm, case_id, &mut xpath, authority)
		.await?;
	apply_sender_diagnosis_section(ctx, &mut doc, &parser, mm, case_id, &mut xpath)
		.await?;
	apply_case_summary_section(ctx, &mut doc, &parser, mm, case_id, &mut xpath)
		.await?;
	postprocess_export_doc(&mut doc, &mut xpath);

	Ok(normalize_namespace_artifacts(doc.to_string()))
}

async fn apply_patient_section(
	ctx: &Ctx,
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let Some(patient) = fetch_patient_information(ctx, mm, case_id).await? else {
		return Ok(());
	};
	let identifiers = fetch_patient_identifiers(ctx, mm, patient.id).await?;
	let parent = fetch_parent_information(ctx, mm, patient.id).await?;
	let parent_past_drugs = if let Some(parent) = parent.as_ref() {
		fetch_parent_past_drug_history(ctx, mm, parent.id).await?
	} else {
		Vec::new()
	};
	let parent_medical_history = if let Some(parent) = parent.as_ref() {
		fetch_parent_medical_history(ctx, mm, parent.id).await?
	} else {
		Vec::new()
	};
	let medical_history =
		fetch_medical_history_episodes(ctx, mm, patient.id).await?;
	let past_drugs = fetch_past_drug_history(ctx, mm, patient.id).await?;
	let death_info = fetch_patient_death_information(mm, patient.id).await?;

	if let Some(v) = patient.patient_initials.as_deref() {
		set_text_first(xpath, "//hl7:primaryRole/hl7:player1/hl7:name", v);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:name",
			"nullFlavor",
		);
	} else if let Some(null_flavor) = patient.patient_initials_null_flavor.as_deref()
	{
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:name",
			"nullFlavor",
			null_flavor,
		);
	}
	if let Some(v) = patient.birth_date {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"value",
			&fmt_date(v),
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"nullFlavor",
		);
	} else if let Some(null_flavor) = patient.birth_date_null_flavor.as_deref() {
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"value",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"nullFlavor",
			null_flavor,
		);
	}
	if patient.age_at_time_of_onset.is_some() {
		ensure_patient_observation(xpath, doc, parser, "3", "PQ")?;
		let age_xpath =
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value";
		if let Some(v) = patient.age_at_time_of_onset.as_ref() {
			set_attr_first(xpath, age_xpath, "value", &v.to_string());
			if let Some(unit) = patient.age_unit.as_deref() {
				set_attr_first(xpath, age_xpath, "unit", unit);
			}
			remove_attr_first(xpath, age_xpath, "nullFlavor");
		}
	}
	if let Some(v) = patient.sex.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"code",
			v,
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"nullFlavor",
		);
	} else if let Some(null_flavor) = patient.sex_null_flavor.as_deref() {
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"code",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"nullFlavor",
			null_flavor,
		);
	}
	if let Some(v) = patient.race_code.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"xsi:type",
			"CE",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"code",
			write_fda_d_11_r_1(v),
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"nullFlavor",
		);
	} else if let Some(null_flavor) = patient.race_code_null_flavor.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"xsi:type",
			"CE",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"code",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"displayName",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"codeSystem",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C17049']]/hl7:value",
			"nullFlavor",
			null_flavor,
		);
	}
	if let Some(v) = patient.ethnicity_code.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"xsi:type",
			"CE",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"code",
			write_fda_d_12(v),
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"nullFlavor",
		);
	} else if let Some(null_flavor) = patient.ethnicity_code_null_flavor.as_deref() {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"xsi:type",
			"CE",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"code",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"displayName",
		);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"codeSystem",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='C16564']]/hl7:value",
			"nullFlavor",
			null_flavor,
		);
	}
	if let Some(v) = patient.last_menstrual_period_date {
		ensure_patient_observation(xpath, doc, parser, "22", "TS")?;
		write_d_6(xpath, v);
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
			"nullFlavor",
		);
	} else if let Some(null_flavor) =
		patient.last_menstrual_period_date_null_flavor.as_deref()
	{
		ensure_patient_observation(xpath, doc, parser, "22", "TS")?;
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
			"value",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
			"nullFlavor",
			null_flavor,
		);
	}
	if let Some(v) = patient.medical_history_text.as_deref() {
		ensure_patient_history_text(xpath, doc, parser)?;
		remove_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
			"nullFlavor",
		);
		write_d_7_2(xpath, v);
	} else if let Some(null_flavor) =
		patient.medical_history_text_null_flavor.as_deref()
	{
		ensure_patient_history_text(xpath, doc, parser)?;
		set_text_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
			"",
		);
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value",
			"nullFlavor",
			null_flavor,
		);
	}
	apply_medical_history_section(doc, parser, xpath, &medical_history)?;
	if patient.gestation_period.is_some() || patient.gestation_period_unit.is_some()
	{
		ensure_patient_observation(xpath, doc, parser, "16", "PQ")?;
		if let Some(v) = patient.gestation_period.as_ref() {
			write_d_2_2_1a(xpath, &v.to_string());
		}
		if let Some(v) = patient.gestation_period_unit.as_deref() {
			write_d_2_2_1b(xpath, v);
		}
	}
	if let Some(v) = patient.age_group.as_deref() {
		ensure_patient_observation(xpath, doc, parser, "4", "CE")?;
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='4']]/hl7:value",
			"xsi:type",
			"CE",
		);
		write_d_2_3(xpath, v);
	}
	if let Some(v) = patient.concomitant_therapy {
		ensure_patient_history_organizer(xpath, doc, parser)?;
		let therapy_xpath = "//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='11']]/hl7:value";
		if xpath
			.findnodes(therapy_xpath, None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				doc,
				parser,
				xpath,
				"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]",
				"<component typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"11\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\"/></observation></component>",
			)?;
		}
		write_d_7_3(xpath, therapy_xpath, v);
	}

	for ident in &identifiers {
		ensure_patient_identifier(xpath, doc, parser, &ident.identifier_type_code)?;
		let id_xpath = format!(
			"//hl7:primaryRole/hl7:player1/hl7:asIdentifiedEntity[hl7:code[@code='{}']]/hl7:id",
			ident.identifier_type_code
		);
		match ident.identifier_type_code.as_str() {
			"1" => write_d_1_1_1(xpath, &id_xpath, ident),
			"2" => write_d_1_1_2(xpath, &id_xpath, ident),
			"3" => write_d_1_1_3(xpath, &id_xpath, ident),
			"4" => write_d_1_1_4(xpath, &id_xpath, ident),
			_ => write_patient_identifier(xpath, &id_xpath, ident),
		}
	}

	if let Some(parent) = parent {
		ensure_parent_role(xpath, doc, parser)?;
		if let Some(v) = parent.parent_identification.as_deref() {
			let name_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:name";
			write_d_10_1(xpath, name_xpath, v);
			remove_attr_first(xpath, name_xpath, "nullFlavor");
		} else if let Some(null_flavor) =
			parent.parent_identification_null_flavor.as_deref()
		{
			let name_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:name";
			set_text_first(xpath, name_xpath, "");
			set_attr_first(xpath, name_xpath, "nullFlavor", null_flavor);
		}
		if let Some(v) = parent.parent_birth_date {
			write_d_10_2_1(xpath, v);
			remove_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:birthTime",
				"nullFlavor",
			);
		} else if let Some(null_flavor) =
			parent.parent_birth_date_null_flavor.as_deref()
		{
			remove_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:birthTime",
				"value",
			);
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:birthTime",
				"nullFlavor",
				null_flavor,
			);
		}
		if let Some(v) = parent.sex.as_deref() {
			let gender_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:administrativeGenderCode";
			if xpath
				.findnodes(gender_xpath, None)
				.map(|nodes| nodes.is_empty())
				.unwrap_or(true)
			{
				append_fragment_child(
					doc,
					parser,
					xpath,
					"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson",
					"<administrativeGenderCode/>",
				)?;
			}
			write_d_10_6(xpath, gender_xpath, v);
			remove_attr_first(xpath, gender_xpath, "nullFlavor");
		} else if let Some(null_flavor) = parent.sex_null_flavor.as_deref() {
			let gender_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:administrativeGenderCode";
			if xpath
				.findnodes(gender_xpath, None)
				.map(|nodes| nodes.is_empty())
				.unwrap_or(true)
			{
				append_fragment_child(
					doc,
					parser,
					xpath,
					"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson",
					"<administrativeGenderCode/>",
				)?;
			}
			remove_attr_first(xpath, gender_xpath, "code");
			set_attr_first(xpath, gender_xpath, "nullFlavor", null_flavor);
		}
		if let Some(v) = parent.last_menstrual_period_date {
			write_d_10_3(xpath, v);
			remove_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
				"nullFlavor",
			);
		} else if let Some(null_flavor) =
			parent.last_menstrual_period_date_null_flavor.as_deref()
		{
			remove_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
				"value",
			);
			set_attr_first(
				xpath,
				"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value",
				"nullFlavor",
				null_flavor,
			);
		}
		if let Some(v) = parent.medical_history_text.as_deref() {
			write_d_10_7_2(xpath, v);
		}
		if let Some(v) = parent.weight_kg.as_ref() {
			let weight_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='7']]/hl7:value";
			if xpath
				.findnodes(weight_xpath, None)
				.map(|nodes| nodes.is_empty())
				.unwrap_or(true)
			{
				append_fragment_child(
					doc,
					parser,
					xpath,
					"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]",
					"<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"7\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"PQ\"/></observation></subjectOf2>",
				)?;
			}
			write_d_10_4(xpath, weight_xpath, &v.to_string());
			set_attr_first(xpath, weight_xpath, "unit", "kg");
		}
		if let Some(v) = parent.height_cm.as_ref() {
			let height_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='17']]/hl7:value";
			if xpath
				.findnodes(height_xpath, None)
				.map(|nodes| nodes.is_empty())
				.unwrap_or(true)
			{
				append_fragment_child(
					doc,
					parser,
					xpath,
					"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]",
					"<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"17\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"PQ\"/></observation></subjectOf2>",
				)?;
			}
			write_d_10_5(xpath, height_xpath, &v.to_string());
			set_attr_first(xpath, height_xpath, "unit", "cm");
		}
		if parent.parent_age.is_some() || parent.parent_age_unit.is_some() {
			let age_value_xpath = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='3']]/hl7:value";
			if xpath
				.findnodes(age_value_xpath, None)
				.map(|nodes| nodes.is_empty())
				.unwrap_or(true)
			{
				append_fragment_child(
					doc,
					parser,
					xpath,
					"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]",
					"<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"3\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"PQ\"/></observation></subjectOf2>",
				)?;
			}
			if let Some(v) = parent.parent_age.as_ref() {
				write_d_10_2_2a(xpath, age_value_xpath, &v.to_string());
			}
			if let Some(v) = parent.parent_age_unit.as_deref() {
				write_d_10_2_2b(xpath, age_value_xpath, v);
			}
			remove_attr_first(xpath, age_value_xpath, "nullFlavor");
		}
		apply_parent_past_drug_history_section(
			doc,
			parser,
			xpath,
			&parent_past_drugs,
			matches!(authority, lib_core::regulatory::RegulatoryAuthority::Mfds),
		)?;
		apply_parent_medical_history_section(
			doc,
			parser,
			xpath,
			&parent_medical_history,
		)?;
	}

	apply_past_drug_history_section(
		doc,
		parser,
		xpath,
		&past_drugs,
		matches!(authority, lib_core::regulatory::RegulatoryAuthority::Mfds),
	)?;
	if !matches!(authority, lib_core::regulatory::RegulatoryAuthority::Fda) {
		remove_nodes(
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2[hl7:observation/hl7:code[@code='C17049' or @code='C16564']]",
		);
	}
	apply_patient_death_null_flavor(doc, parser, xpath, &death_info)?;

	Ok(())
}

/// e2b:FDA.D.11.r.1
fn write_fda_d_11_r_1(value: &str) -> &str {
	value
}

/// e2b:FDA.D.12
fn write_fda_d_12(value: &str) -> &str {
	value
}

fn write_patient_identifier(
	xpath: &mut Context,
	path: &str,
	value: &PatientIdentifier,
) {
	if let Some(null_flavor) = value.identifier_value_null_flavor.as_deref() {
		remove_attr_first(xpath, path, "extension");
		set_attr_first(xpath, path, "nullFlavor", null_flavor);
	} else if let Some(identifier) = value.identifier_value.as_deref() {
		remove_attr_first(xpath, path, "nullFlavor");
		set_attr_first(xpath, path, "extension", identifier);
	}
}

/// e2b:D.1.1.1
fn write_d_1_1_1(xpath: &mut Context, path: &str, value: &PatientIdentifier) {
	write_patient_identifier(xpath, path, value);
}

/// e2b:D.1.1.2
fn write_d_1_1_2(xpath: &mut Context, path: &str, value: &PatientIdentifier) {
	write_patient_identifier(xpath, path, value);
}

/// e2b:D.1.1.3
fn write_d_1_1_3(xpath: &mut Context, path: &str, value: &PatientIdentifier) {
	write_patient_identifier(xpath, path, value);
}

/// e2b:D.1.1.4
fn write_d_1_1_4(xpath: &mut Context, path: &str, value: &PatientIdentifier) {
	write_patient_identifier(xpath, path, value);
}

/// e2b:D.10.1
fn write_d_10_1(xpath: &mut Context, path: &str, value: &str) {
	set_text_first(xpath, path, value);
}

/// e2b:D.10.2.1
fn write_d_10_2_1(xpath: &mut Context, value: time::Date) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:associatedPerson/hl7:birthTime", "value", &fmt_date(value));
}

/// e2b:D.10.2.2a
fn write_d_10_2_2a(xpath: &mut Context, path: &str, value: &str) {
	set_attr_first(xpath, path, "value", value);
}

/// e2b:D.10.2.2b
fn write_d_10_2_2b(xpath: &mut Context, path: &str, value: &str) {
	set_attr_first(xpath, path, "unit", value);
}

/// e2b:D.10.3
fn write_d_10_3(xpath: &mut Context, value: time::Date) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value", "value", &fmt_date(value));
}

/// e2b:D.10.4
fn write_d_10_4(xpath: &mut Context, path: &str, value: &str) {
	set_attr_first(xpath, path, "value", value);
}

/// e2b:D.10.5
fn write_d_10_5(xpath: &mut Context, path: &str, value: &str) {
	set_attr_first(xpath, path, "value", value);
}

/// e2b:D.10.6
fn write_d_10_6(xpath: &mut Context, path: &str, value: &str) {
	set_attr_first(xpath, path, "code", value);
}

/// e2b:D.10.7.2
fn write_d_10_7_2(xpath: &mut Context, value: &str) {
	set_text_first(xpath, "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value", value);
}

/// e2b:D.2.2.1a
fn write_d_2_2_1a(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='16']]/hl7:value", "value", value);
}

/// e2b:D.2.2.1b
fn write_d_2_2_1b(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='16']]/hl7:value", "unit", value);
}

/// e2b:D.2.3
fn write_d_2_3(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='4']]/hl7:value", "code", value);
}

/// e2b:D.6
fn write_d_6(xpath: &mut Context, value: sqlx::types::time::Date) {
	set_attr_first(xpath, "//hl7:primaryRole/hl7:subjectOf2/hl7:observation[hl7:code[@code='22']]/hl7:value", "value", &fmt_date(value));
}

/// e2b:D.7.2
fn write_d_7_2(xpath: &mut Context, value: &str) {
	set_text_first(xpath, "//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@code='18']]/hl7:value", value);
}

/// e2b:D.7.3
fn write_d_7_3(xpath: &mut Context, path: &str, value: bool) {
	set_attr_first(xpath, path, "value", if value { "true" } else { "false" });
}

fn apply_parent_medical_history_section(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	episodes: &[ParentMedicalHistory],
) -> Result<()> {
	if episodes.is_empty() {
		return Ok(());
	}
	let organizer = "//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]";
	remove_nodes(xpath, &format!("{organizer}/hl7:component/hl7:observation[hl7:code[@codeSystem='2.16.840.1.113883.6.163']]"));
	let mut rows = episodes.to_vec();
	rows.sort_by_key(|row| row.sequence_number);
	for episode in rows {
		let mut attrs = String::from("codeSystem=\"2.16.840.1.113883.6.163\"");
		if let Some(code) = write_d_10_7_1_r_1b(&episode) {
			attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
		}
		if let Some(version) = write_d_10_7_1_r_1a(&episode) {
			attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		let (start, start_null) = write_d_10_7_1_r_2(&episode);
		let (end, end_null) = write_d_10_7_1_r_4(&episode);
		let effective_time =
			history_effective_time(start, start_null, end, end_null);
		let continuing = write_d_10_7_1_r_3(&episode);
		let comments = write_d_10_7_1_r_5(&episode);
		let fragment = format!("<component typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code {attrs}/>{effective_time}{continuing}{comments}</observation></component>");
		append_fragment_child(doc, parser, xpath, organizer, &fragment)?;
	}
	Ok(())
}

/// e2b:D.10.7.1.r.1a
fn write_d_10_7_1_r_1a(value: &ParentMedicalHistory) -> Option<&str> {
	value.meddra_version.as_deref()
}

/// e2b:D.10.7.1.r.1b
fn write_d_10_7_1_r_1b(value: &ParentMedicalHistory) -> Option<&str> {
	value.meddra_code.as_deref()
}

/// e2b:D.10.7.1.r.2
fn write_d_10_7_1_r_2(
	value: &ParentMedicalHistory,
) -> (Option<time::Date>, Option<&str>) {
	(value.start_date, value.start_date_null_flavor.as_deref())
}

/// e2b:D.10.7.1.r.3
fn write_d_10_7_1_r_3(value: &ParentMedicalHistory) -> String {
	value.continuing.map(|v| format!("<inboundRelationship typeCode=\"REFR\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"13\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"{}\"/></observation></inboundRelationship>", if v { "true" } else { "false" })).unwrap_or_default()
}

/// e2b:D.10.7.1.r.4
fn write_d_10_7_1_r_4(
	value: &ParentMedicalHistory,
) -> (Option<time::Date>, Option<&str>) {
	(value.end_date, value.end_date_null_flavor.as_deref())
}

/// e2b:D.10.7.1.r.5
fn write_d_10_7_1_r_5(value: &ParentMedicalHistory) -> String {
	value.comments.as_deref().map(|v| format!("<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"10\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value>{}</value></observation></outboundRelationship2>", xml_escape(v))).unwrap_or_default()
}

fn apply_parent_past_drug_history_section(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	past_drugs: &[ParentPastDrugHistory],
	include_mfds: bool,
) -> Result<()> {
	if past_drugs.is_empty() {
		return Ok(());
	}

	let parent_role_xpath =
		"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]";
	remove_nodes(
		xpath,
		"//hl7:primaryRole/hl7:player1/hl7:role[hl7:code[@code='PRN']]/hl7:subjectOf2[hl7:organizer/hl7:code[@code='2']]",
	);

	let mut rows = past_drugs.to_vec();
	rows.sort_by_key(|row| row.sequence_number);

	for drug in rows {
		append_fragment_child(
			doc,
			parser,
			xpath,
			parent_role_xpath,
			&parent_past_drug_history_fragment(&drug, include_mfds),
		)?;
	}

	Ok(())
}

/// e2b:D.10.8.r.1
fn write_d_10_8_r_1(value: &ParentPastDrugHistory) -> Option<&str> {
	value.drug_name.as_deref()
}

/// e2b:D.10.8.r.1.KR.1a
fn write_d_10_8_r_1_kr_1a(value: &ParentPastDrugHistory) -> Option<&str> {
	value.mfds_medicinal_product_version.as_deref()
}

/// e2b:D.10.8.r.1.KR.1b
fn write_d_10_8_r_1_kr_1b(value: &ParentPastDrugHistory) -> Option<&str> {
	value.mfds_medicinal_product_id.as_deref()
}

/// e2b:D.10.8.r.2a
fn write_d_10_8_r_2a(value: &ParentPastDrugHistory) -> Option<&str> {
	value.mpid_version.as_deref()
}

/// e2b:D.10.8.r.2b
fn write_d_10_8_r_2b(value: &ParentPastDrugHistory) -> Option<&str> {
	value.mpid.as_deref()
}

/// e2b:D.10.8.r.3a
fn write_d_10_8_r_3a(value: &ParentPastDrugHistory) -> Option<&str> {
	value.phpid_version.as_deref()
}

/// e2b:D.10.8.r.3b
fn write_d_10_8_r_3b(value: &ParentPastDrugHistory) -> Option<&str> {
	value.phpid.as_deref()
}

/// e2b:D.10.8.r.4
fn write_d_10_8_r_4(
	value: &ParentPastDrugHistory,
) -> (Option<time::Date>, Option<&str>) {
	(value.start_date, value.start_date_null_flavor.as_deref())
}

/// e2b:D.10.8.r.5
fn write_d_10_8_r_5(
	value: &ParentPastDrugHistory,
) -> (Option<time::Date>, Option<&str>) {
	(value.end_date, value.end_date_null_flavor.as_deref())
}

/// e2b:D.10.8.r.6a
fn write_d_10_8_r_6a(value: &ParentPastDrugHistory) -> Option<&str> {
	value.indication_meddra_version.as_deref()
}

/// e2b:D.10.8.r.6b
fn write_d_10_8_r_6b(value: &ParentPastDrugHistory) -> Option<&str> {
	value.indication_meddra_code.as_deref()
}

/// e2b:D.10.8.r.7a
fn write_d_10_8_r_7a(value: &ParentPastDrugHistory) -> Option<&str> {
	value.reaction_meddra_version.as_deref()
}

/// e2b:D.10.8.r.7b
fn write_d_10_8_r_7b(value: &ParentPastDrugHistory) -> Option<&str> {
	value.reaction_meddra_code.as_deref()
}

fn parent_past_drug_history_fragment(
	drug: &ParentPastDrugHistory,
	include_mfds: bool,
) -> String {
	let name_fragment = if let Some(name) = write_d_10_8_r_1(drug) {
		format!("<name>{}</name>", xml_escape(name))
	} else {
		"<name/>".to_string()
	};

	let mfds_code = if include_mfds
		&& (write_d_10_8_r_1_kr_1b(drug).is_some()
			|| write_d_10_8_r_1_kr_1a(drug).is_some())
	{
		let mut attrs = String::new();
		if let Some(id) = write_d_10_8_r_1_kr_1b(drug) {
			attrs.push_str(&format!(" code=\"{}\"", xml_escape(id)));
		}
		if let Some(version) = write_d_10_8_r_1_kr_1a(drug) {
			attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		format!("<code{attrs}/>")
	} else {
		String::new()
	};

	let mut identifiers = String::new();
	if write_d_10_8_r_2b(drug).is_some() || write_d_10_8_r_2a(drug).is_some() {
		let mut code_attrs = String::from(
			"code=\"MPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\"",
		);
		if let Some(version) = write_d_10_8_r_2a(drug) {
			code_attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		identifiers.push_str(&format!(
			"<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"{}\"/><code {code_attrs}/></asIdentifiedEntity>",
			xml_escape(write_d_10_8_r_2b(drug).unwrap_or(""))
		));
	}
	if write_d_10_8_r_3b(drug).is_some() || write_d_10_8_r_3a(drug).is_some() {
		let mut code_attrs = String::from(
			"code=\"PHPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\"",
		);
		if let Some(version) = write_d_10_8_r_3a(drug) {
			code_attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		identifiers.push_str(&format!(
			"<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"{}\"/><code {code_attrs}/></asIdentifiedEntity>",
			xml_escape(write_d_10_8_r_3b(drug).unwrap_or(""))
		));
	}

	let (start, start_null) = write_d_10_8_r_4(drug);
	let (end, end_null) = write_d_10_8_r_5(drug);
	let effective_time = history_effective_time(start, start_null, end, end_null);

	let indication = if drug.indication_meddra_version.is_some()
		|| drug.indication_meddra_code.is_some()
	{
		let mut value_attrs = String::from("xsi:type=\"CE\"");
		if let Some(code) = write_d_10_8_r_6b(drug) {
			value_attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
		}
		if let Some(version) = write_d_10_8_r_6a(drug) {
			value_attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		format!(
			"<outboundRelationship2 typeCode=\"RSON\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"19\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" codeSystemVersion=\"1.1\" displayName=\"indication\"/><value {value_attrs}/></observation></outboundRelationship2>"
		)
	} else {
		String::new()
	};

	let reaction = if write_d_10_8_r_7a(drug).is_some()
		|| write_d_10_8_r_7b(drug).is_some()
	{
		let mut value_attrs = String::from("xsi:type=\"CE\"");
		if let Some(code) = write_d_10_8_r_7b(drug) {
			value_attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
		}
		if let Some(version) = write_d_10_8_r_7a(drug) {
			value_attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		format!(
			"<outboundRelationship2 typeCode=\"CAUS\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"29\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" codeSystemVersion=\"1.1\" displayName=\"reaction\"/><value {value_attrs}/></observation></outboundRelationship2>"
		)
	} else {
		String::new()
	};

	format!(
		"<subjectOf2 typeCode=\"SBJ\"><organizer classCode=\"CATEGORY\" moodCode=\"EVN\"><code code=\"2\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.20\" displayName=\"drugHistory\"/><component typeCode=\"COMP\"><substanceAdministration classCode=\"SBADM\" moodCode=\"EVN\">{effective_time}<consumable typeCode=\"CSM\"><instanceOfKind classCode=\"INST\"><kindOfProduct classCode=\"MMAT\" determinerCode=\"KIND\">{mfds_code}{name_fragment}{identifiers}</kindOfProduct></instanceOfKind></consumable>{indication}{reaction}</substanceAdministration></component></organizer></subjectOf2>"
	)
}

/// e2b:D.7.1.r.1a
fn write_d_7_1_r_1a(value: &MedicalHistoryEpisode) -> Option<&str> {
	value.meddra_version.as_deref()
}

/// e2b:D.7.1.r.1b
fn write_d_7_1_r_1b(value: &MedicalHistoryEpisode) -> Option<&str> {
	value.meddra_code.as_deref()
}

/// e2b:D.7.1.r.2
fn write_d_7_1_r_2(
	value: &MedicalHistoryEpisode,
) -> (Option<sqlx::types::time::Date>, Option<&str>) {
	(value.start_date, value.start_date_null_flavor.as_deref())
}

/// e2b:D.7.1.r.3
fn write_d_7_1_r_3(value: &MedicalHistoryEpisode) -> String {
	if let Some(null_flavor) = value.continuing_null_flavor.as_deref() {
		return format!("<inboundRelationship typeCode=\"REFR\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"13\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" nullFlavor=\"{}\"/></observation></inboundRelationship>", xml_escape(null_flavor));
	}
	value.continuing.map(|v| format!("<inboundRelationship typeCode=\"REFR\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"13\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"{}\"/></observation></inboundRelationship>", if v { "true" } else { "false" })).unwrap_or_default()
}

/// e2b:D.7.1.r.4
fn write_d_7_1_r_4(
	value: &MedicalHistoryEpisode,
) -> (Option<sqlx::types::time::Date>, Option<&str>) {
	(value.end_date, value.end_date_null_flavor.as_deref())
}

/// e2b:D.7.1.r.5
fn write_d_7_1_r_5(value: &MedicalHistoryEpisode) -> String {
	value.comments.as_deref().map(|v| format!("<outboundRelationship2 typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"10\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value>{}</value></observation></outboundRelationship2>", xml_escape(v))).unwrap_or_default()
}

/// e2b:D.7.1.r.6
fn write_d_7_1_r_6(value: &MedicalHistoryEpisode) -> String {
	value.family_history.map(|v| format!("<outboundRelationship2 typeCode=\"PERT\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"38\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\"/><value xsi:type=\"BL\" value=\"{}\"/></observation></outboundRelationship2>", if v { "true" } else { "false" })).unwrap_or_default()
}

fn apply_medical_history_section(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	episodes: &[MedicalHistoryEpisode],
) -> Result<()> {
	if episodes.is_empty() {
		return Ok(());
	}

	ensure_patient_history_organizer(xpath, doc, parser)?;
	remove_nodes(
		xpath,
		"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]/hl7:component/hl7:observation[hl7:code[@codeSystem='2.16.840.1.113883.6.163']]",
	);

	let mut rows = episodes.to_vec();
	rows.sort_by_key(|row| row.sequence_number);
	for episode in rows {
		let mut code_attrs = String::from("codeSystem=\"2.16.840.1.113883.6.163\"");
		if let Some(code) = write_d_7_1_r_1b(&episode) {
			code_attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
		}
		if let Some(version) = write_d_7_1_r_1a(&episode) {
			code_attrs.push_str(&format!(
				" codeSystemVersion=\"{}\"",
				xml_escape(version)
			));
		}
		let (start, start_null) = write_d_7_1_r_2(&episode);
		let (end, end_null) = write_d_7_1_r_4(&episode);
		let effective_time =
			history_effective_time(start, start_null, end, end_null);
		let continuing = write_d_7_1_r_3(&episode);
		let comments = write_d_7_1_r_5(&episode);
		let family_history = write_d_7_1_r_6(&episode);
		let fragment = format!(
			"<component typeCode=\"COMP\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code {code_attrs}/>{effective_time}{continuing}{comments}{family_history}</observation></component>"
		);
		append_fragment_child(
			doc,
			parser,
			xpath,
			"//hl7:primaryRole/hl7:subjectOf2/hl7:organizer[hl7:code[@code='1']]",
			&fragment,
		)?;
	}
	Ok(())
}

/// e2b:D.8.r.1
fn write_d_8_r_1(value: &PastDrugHistory) -> Option<&str> {
	value.drug_name.as_deref()
}

/// e2b:D.8.r.1.KR.1a
fn write_d_8_r_1_kr_1a(value: &PastDrugHistory) -> Option<&str> {
	value
		.mfds_medicinal_product_version
		.as_deref()
		.filter(|v| !v.trim().is_empty())
}

/// e2b:D.8.r.1.KR.1b
fn write_d_8_r_1_kr_1b(value: &PastDrugHistory) -> Option<&str> {
	value
		.mfds_medicinal_product_id
		.as_deref()
		.filter(|v| !v.trim().is_empty())
}

/// e2b:D.8.r.2a
fn write_d_8_r_2a(value: &PastDrugHistory) -> Option<&str> {
	value.mpid_version.as_deref()
}

/// e2b:D.8.r.2b
fn write_d_8_r_2b(value: &PastDrugHistory) -> Option<&str> {
	value.mpid.as_deref()
}

/// e2b:D.8.r.3a
fn write_d_8_r_3a(value: &PastDrugHistory) -> Option<&str> {
	value.phpid_version.as_deref()
}

/// e2b:D.8.r.3b
fn write_d_8_r_3b(value: &PastDrugHistory) -> Option<&str> {
	value.phpid.as_deref()
}

/// e2b:D.8.r.4
fn write_d_8_r_4(value: &PastDrugHistory) -> (Option<time::Date>, Option<&str>) {
	(value.start_date, value.start_date_null_flavor.as_deref())
}

/// e2b:D.8.r.5
fn write_d_8_r_5(value: &PastDrugHistory) -> (Option<time::Date>, Option<&str>) {
	(value.end_date, value.end_date_null_flavor.as_deref())
}

/// e2b:D.8.r.6a
fn write_d_8_r_6a(value: &PastDrugHistory) -> Option<&str> {
	value.indication_meddra_version.as_deref()
}

/// e2b:D.8.r.6b
fn write_d_8_r_6b(value: &PastDrugHistory) -> Option<&str> {
	value.indication_meddra_code.as_deref()
}

/// e2b:D.8.r.7a
fn write_d_8_r_7a(value: &PastDrugHistory) -> Option<&str> {
	value.reaction_meddra_version.as_deref()
}

/// e2b:D.8.r.7b
fn write_d_8_r_7b(value: &PastDrugHistory) -> Option<&str> {
	value.reaction_meddra_code.as_deref()
}

fn apply_past_drug_history_section(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	past_drugs: &[PastDrugHistory],
	include_mfds: bool,
) -> Result<()> {
	if past_drugs.is_empty() {
		return Ok(());
	}

	remove_nodes(
		xpath,
		"//hl7:primaryRole/hl7:subjectOf2[hl7:organizer/hl7:code[@code='2']]",
	);

	let mut rows = past_drugs.to_vec();
	rows.sort_by_key(|row| row.sequence_number);

	for drug in rows {
		let name_fragment = if let Some(name) = write_d_8_r_1(&drug) {
			format!("<name>{}</name>", xml_escape(name))
		} else if let Some(null_flavor) = drug.drug_name_null_flavor.as_deref() {
			format!("<name nullFlavor=\"{}\"/>", xml_escape(null_flavor))
		} else {
			"<name/>".to_string()
		};

		let mut identifiers = String::new();
		let mfds_product_id = write_d_8_r_1_kr_1b(&drug);
		let mfds_product_version = write_d_8_r_1_kr_1a(&drug);
		let mfds_code = if include_mfds
			&& (mfds_product_id.is_some() || mfds_product_version.is_some())
		{
			let mut attrs = String::new();
			if let Some(id) = mfds_product_id {
				attrs.push_str(&format!(" code=\"{}\"", xml_escape(id)));
			}
			if let Some(version) = mfds_product_version {
				attrs.push_str(&format!(
					" codeSystemVersion=\"{}\"",
					xml_escape(version)
				));
			}
			format!("<code{attrs}/>")
		} else {
			String::new()
		};
		if write_d_8_r_2b(&drug).is_some() || write_d_8_r_2a(&drug).is_some() {
			let mut code_attrs = String::from(
				"code=\"MPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\"",
			);
			if let Some(version) = write_d_8_r_2a(&drug) {
				code_attrs.push_str(&format!(
					" codeSystemVersion=\"{}\"",
					xml_escape(version)
				));
			}
			identifiers.push_str(&format!(
				"<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"{}\"/><code {code_attrs}/></asIdentifiedEntity>",
				xml_escape(write_d_8_r_2b(&drug).unwrap_or(""))
			));
		}
		if write_d_8_r_3b(&drug).is_some() || write_d_8_r_3a(&drug).is_some() {
			let mut code_attrs = String::from(
				"code=\"PHPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\"",
			);
			if let Some(version) = write_d_8_r_3a(&drug) {
				code_attrs.push_str(&format!(
					" codeSystemVersion=\"{}\"",
					xml_escape(version)
				));
			}
			identifiers.push_str(&format!(
				"<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"{}\"/><code {code_attrs}/></asIdentifiedEntity>",
				xml_escape(write_d_8_r_3b(&drug).unwrap_or(""))
			));
		}

		let (start, start_null) = write_d_8_r_4(&drug);
		let (end, end_null) = write_d_8_r_5(&drug);
		let effective_time =
			history_effective_time(start, start_null, end, end_null);

		let indication = if drug.indication_meddra_version.is_some()
			|| drug.indication_meddra_code.is_some()
		{
			let mut value_attrs = String::from("xsi:type=\"CE\"");
			if let Some(code) = write_d_8_r_6b(&drug) {
				value_attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
			}
			if let Some(version) = write_d_8_r_6a(&drug) {
				value_attrs.push_str(&format!(
					" codeSystemVersion=\"{}\"",
					xml_escape(version)
				));
			}
			format!(
				"<outboundRelationship2 typeCode=\"RSON\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"19\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" codeSystemVersion=\"1.1\" displayName=\"indication\"/><value {value_attrs}/></observation></outboundRelationship2>"
			)
		} else {
			String::new()
		};

		let reaction = if drug.reaction_meddra_version.is_some()
			|| drug.reaction_meddra_code.is_some()
		{
			let mut value_attrs = String::from("xsi:type=\"CE\"");
			if let Some(code) = write_d_8_r_7b(&drug) {
				value_attrs.push_str(&format!(" code=\"{}\"", xml_escape(code)));
			}
			if let Some(version) = write_d_8_r_7a(&drug) {
				value_attrs.push_str(&format!(
					" codeSystemVersion=\"{}\"",
					xml_escape(version)
				));
			}
			format!(
				"<outboundRelationship2 typeCode=\"CAUS\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"29\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" codeSystemVersion=\"1.1\" displayName=\"reaction\"/><value {value_attrs}/></observation></outboundRelationship2>"
			)
		} else {
			String::new()
		};

		let fragment = format!(
			"<subjectOf2 typeCode=\"SBJ\"><organizer classCode=\"CATEGORY\" moodCode=\"EVN\"><code code=\"2\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.20\" displayName=\"drugHistory\"/><component typeCode=\"COMP\"><substanceAdministration classCode=\"SBADM\" moodCode=\"EVN\">{effective_time}<consumable typeCode=\"CSM\"><instanceOfKind classCode=\"INST\"><kindOfProduct classCode=\"MMAT\" determinerCode=\"KIND\">{mfds_code}{name_fragment}{identifiers}</kindOfProduct></instanceOfKind></consumable>{indication}{reaction}</substanceAdministration></component></organizer></subjectOf2>"
		);
		append_fragment_child(doc, parser, xpath, "//hl7:primaryRole", &fragment)?;
	}

	Ok(())
}

fn history_effective_time(
	start_date: Option<time::Date>,
	start_null_flavor: Option<&str>,
	end_date: Option<time::Date>,
	end_null_flavor: Option<&str>,
) -> String {
	if start_date.is_none()
		&& start_null_flavor.is_none()
		&& end_date.is_none()
		&& end_null_flavor.is_none()
	{
		return String::new();
	}

	let low = match (start_date, start_null_flavor) {
		(Some(value), _) => format!("<low value=\"{}\"/>", fmt_date(value)),
		(None, Some(null_flavor)) => {
			format!("<low nullFlavor=\"{}\"/>", xml_escape(null_flavor))
		}
		(None, None) => "<low/>".to_string(),
	};
	let high = match (end_date, end_null_flavor) {
		(Some(value), _) => format!("<high value=\"{}\"/>", fmt_date(value)),
		(None, Some(null_flavor)) => {
			format!("<high nullFlavor=\"{}\"/>", xml_escape(null_flavor))
		}
		(None, None) => "<high/>".to_string(),
	};

	format!("<effectiveTime xsi:type=\"IVL_TS\">{low}{high}</effectiveTime>")
}

fn apply_patient_death_null_flavor(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	death_info: &Option<PatientDeathInformation>,
) -> Result<()> {
	let Some(death) = death_info.as_ref() else {
		return Ok(());
	};
	if death.date_of_death.is_some() {
		remove_attr_first(xpath, "//hl7:primaryRole/hl7:deceasedTime", "nullFlavor");
		return Ok(());
	}
	if let Some(null_flavor) = death.date_of_death_null_flavor.as_deref() {
		if xpath
			.findnodes("//hl7:primaryRole/hl7:deceasedTime", None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				doc,
				parser,
				xpath,
				"//hl7:primaryRole",
				"<deceasedTime/>",
			)?;
		}
		remove_attr_first(xpath, "//hl7:primaryRole/hl7:deceasedTime", "value");
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:deceasedTime",
			"nullFlavor",
			null_flavor,
		);
	}
	Ok(())
}

async fn fetch_patient_death_information(
	mm: &ModelManager,
	patient_id: sqlx::types::Uuid,
) -> Result<Option<PatientDeathInformation>> {
	let sql =
		"SELECT * FROM patient_death_information WHERE patient_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(
			sqlx::query_as::<_, PatientDeathInformation>(sql).bind(patient_id),
		)
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

#[cfg(test)]
mod tests {
	use super::*;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Uuid;
	use std::collections::BTreeSet;

	#[test]
	fn past_drug_fragment_exports_mfds_code_separate_from_identifiers() {
		let drug = PastDrugHistory {
			id: Uuid::nil(),
			patient_id: Uuid::nil(),
			sequence_number: 1,
			deleted: false,
			drug_name: Some("Past & <drug> \"A\"".to_string()),
			drug_name_null_flavor: None,
			mfds_medicinal_product_version: Some("MFV&<>\"".to_string()),
			mfds_medicinal_product_id: Some("MF&<>\"".to_string()),
			mpid: Some("MP&<>\"".to_string()),
			mpid_version: Some("MPV&<>\"".to_string()),
			phpid: Some("PH&<>\"".to_string()),
			phpid_version: Some("PHV&<>\"".to_string()),
			start_date: None,
			start_date_null_flavor: None,
			end_date: None,
			end_date_null_flavor: None,
			indication_meddra_version: None,
			indication_meddra_code: None,
			reaction_meddra_version: None,
			reaction_meddra_code: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		};

		let parser = Parser::default();
		let mut doc = parser
			.parse_string(
				"<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\"><primaryRole/></MCCI_IN200100UV01>",
			)
			.expect("doc");
		let mut xpath = Context::new(&doc).expect("xpath");
		let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
		apply_past_drug_history_section(
			&mut doc,
			&parser,
			&mut xpath,
			&[drug],
			true,
		)
		.expect("apply");
		let fragment = doc.to_string();

		let name = "<name>Past &amp; &lt;drug&gt; \"A\"</name>";
		let mpid = "<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"MP&amp;&lt;&gt;&quot;\"/><code code=\"MPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\" codeSystemVersion=\"MPV&amp;&lt;&gt;&quot;\"/></asIdentifiedEntity>";
		let phpid = "<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"PH&amp;&lt;&gt;&quot;\"/><code code=\"PHPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\" codeSystemVersion=\"PHV&amp;&lt;&gt;&quot;\"/></asIdentifiedEntity>";

		let mfds_index = fragment
			.find("code=\"MF&amp;&lt;&gt;&quot;\"")
			.expect("MFDS product code");
		assert!(fragment.contains("codeSystemVersion=\"MFV&amp;&lt;&gt;&quot;\""));
		let name_index = fragment.find(name).expect("drug name");
		let mpid_index = fragment.find(mpid).expect("MPID identifier");
		let phpid_index = fragment.find(phpid).expect("PhPID identifier");

		assert!(mfds_index < name_index);
		assert!(name_index < mpid_index);
		assert!(mpid_index < phpid_index);
	}

	#[test]
	fn past_drug_fragment_omits_blank_mfds_code() {
		let drug = PastDrugHistory {
			id: Uuid::nil(),
			patient_id: Uuid::nil(),
			sequence_number: 1,
			deleted: false,
			drug_name: Some("Past Drug".to_string()),
			drug_name_null_flavor: None,
			mfds_medicinal_product_version: Some(" ".to_string()),
			mfds_medicinal_product_id: Some(String::new()),
			mpid: Some("MPID-EXACT".to_string()),
			mpid_version: Some("MPID-V1".to_string()),
			phpid: None,
			phpid_version: None,
			start_date: None,
			start_date_null_flavor: None,
			end_date: None,
			end_date_null_flavor: None,
			indication_meddra_version: None,
			indication_meddra_code: None,
			reaction_meddra_version: None,
			reaction_meddra_code: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		};

		let parser = Parser::default();
		let mut doc = parser
			.parse_string(
				"<MCCI_IN200100UV01 xmlns=\"urn:hl7-org:v3\"><primaryRole/></MCCI_IN200100UV01>",
			)
			.expect("doc");
		let mut xpath = Context::new(&doc).expect("xpath");
		let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
		apply_past_drug_history_section(
			&mut doc,
			&parser,
			&mut xpath,
			&[drug],
			true,
		)
		.expect("apply");
		let fragment = doc.to_string();

		assert!(!fragment.contains("<code code=\"\""));
		assert!(!fragment.contains("codeSystemVersion=\" \""));
		assert!(fragment.contains("code=\"MPID\""));
	}

	#[test]
	fn parent_past_drug_fragment_exports_mfds_code_separate_from_identifiers() {
		let drug = ParentPastDrugHistory {
			id: Uuid::nil(),
			parent_id: Uuid::nil(),
			sequence_number: 1,
			deleted: false,
			drug_name: Some("Parent & <drug> \"A\" 'B'".to_string()),
			mpid: Some("MP&<>\"'".to_string()),
			mpid_version: Some("MPV&<>\"'".to_string()),
			mfds_medicinal_product_version: Some("MFV&<>\"'".to_string()),
			mfds_medicinal_product_id: Some("MF&<>\"'".to_string()),
			phpid: Some("PH&<>\"'".to_string()),
			phpid_version: Some("PHV&<>\"'".to_string()),
			start_date: None,
			start_date_null_flavor: None,
			end_date: None,
			end_date_null_flavor: None,
			indication_meddra_version: None,
			indication_meddra_code: None,
			reaction_meddra_version: None,
			reaction_meddra_code: None,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		};

		let fragment = parent_past_drug_history_fragment(&drug, true);
		let non_mfds_fragment = parent_past_drug_history_fragment(&drug, false);

		let mfds_code = "<code code=\"MF&amp;&lt;&gt;&quot;&apos;\" codeSystemVersion=\"MFV&amp;&lt;&gt;&quot;&apos;\"/>";
		let name =
			"<name>Parent &amp; &lt;drug&gt; &quot;A&quot; &apos;B&apos;</name>";
		let mpid = "<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"MP&amp;&lt;&gt;&quot;&apos;\"/><code code=\"MPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\" codeSystemVersion=\"MPV&amp;&lt;&gt;&quot;&apos;\"/></asIdentifiedEntity>";
		let phpid = "<asIdentifiedEntity classCode=\"IDENT\"><id extension=\"PH&amp;&lt;&gt;&quot;&apos;\"/><code code=\"PHPID\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.4\" codeSystemVersion=\"PHV&amp;&lt;&gt;&quot;&apos;\"/></asIdentifiedEntity>";

		let mfds_index = fragment.find(mfds_code).expect("MFDS product code");
		let name_index = fragment.find(name).expect("drug name");
		let mpid_index = fragment.find(mpid).expect("MPID identifier");
		let phpid_index = fragment.find(phpid).expect("PhPID identifier");

		assert!(mfds_index < name_index);
		assert!(!non_mfds_fragment.contains("MF&amp;"));
		assert!(name_index < mpid_index);
		assert!(mpid_index < phpid_index);
	}

	#[test]
	fn section_d_writers_cover_registry_fields() {
		let registry: serde_json::Value = serde_json::from_str(include_str!(
			"../../../../../../registry/sections/d-patient.json"
		))
		.expect("section D registry");
		let expected = registry
			.as_array()
			.expect("registry array")
			.iter()
			.filter(|entry| entry["local_only"] != true)
			.filter_map(|entry| entry["e2br3_code"].as_str())
			.collect::<BTreeSet<_>>();
		let source = format!(
			"{}\n{}",
			include_str!("postprocess.rs"),
			include_str!("../roundtrip/d_patient.rs")
		);
		let implemented = source
			.lines()
			.filter_map(|line| line.trim().strip_prefix("/// e2b:"))
			.collect::<BTreeSet<_>>();

		assert_eq!(implemented, expected);
	}
}
