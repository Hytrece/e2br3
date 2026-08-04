use super::*;

pub fn patch_d_patient(raw_xml: &[u8], patch: &DPatientPatch) -> Result<String> {
	let xml_str = std::str::from_utf8(raw_xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let mut doc = parser
		.parse_string(xml_str)
		.map_err(|err| Error::InvalidXml {
			message: format!("XML parse error: {err}"),
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

	ensure_primary_role(&mut doc, &parser, &mut xpath)?;

	write_d_1(&mut xpath, patch.patient_name);
	write_d_2_1(&mut xpath, patch.birth_date);
	write_d_5(&mut xpath, patch.sex);

	if let Some(age) = patch.age_value {
		ensure_subject_observation(
			&mut doc,
			&parser,
			&mut xpath,
			"3",
			"2.16.840.1.113883.3.989.2.1.1.19",
			"PQ",
		)?;
		write_d_2_2a(&mut xpath, age);
		write_d_2_2b(&mut xpath, patch.age_unit);
	}

	if let Some(weight) = patch.weight_kg {
		ensure_subject_observation(
			&mut doc,
			&parser,
			&mut xpath,
			"7",
			"2.16.840.1.113883.3.989.2.1.1.19",
			"PQ",
		)?;
		write_d_3(&mut xpath, weight);
	}

	if let Some(height) = patch.height_cm {
		ensure_subject_observation(
			&mut doc,
			&parser,
			&mut xpath,
			"17",
			"2.16.840.1.113883.3.989.2.1.1.19",
			"PQ",
		)?;
		write_d_4(&mut xpath, height);
	}

	remove_nodes(&mut xpath, "//hl7:primaryRole/hl7:deceasedTime");
	if let Some(date_of_death) = patch.date_of_death {
		append_fragment_child(
			&mut doc,
			&parser,
			&mut xpath,
			"//hl7:primaryRole",
			&write_d_9_1(date_of_death),
		)?;
	}

	remove_nodes(
		&mut xpath,
		"//hl7:primaryRole/hl7:subjectOf2[hl7:observation/hl7:code[@code='32' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]",
	);
	for cause in patch.reported_causes {
		append_fragment_child(
			&mut doc,
			&parser,
			&mut xpath,
			"//hl7:primaryRole",
			&reported_cause_fragment(cause),
		)?;
	}

	remove_nodes(
		&mut xpath,
		"//hl7:primaryRole/hl7:subjectOf2[hl7:observation/hl7:code[@code='5' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]",
	);
	if patch.autopsy_performed.is_some()
		|| patch.autopsy_performed_null_flavor.is_some()
		|| !patch.autopsy_causes.is_empty()
	{
		append_fragment_child(
			&mut doc,
			&parser,
			&mut xpath,
			"//hl7:primaryRole",
			&write_d_9_3(
				patch.autopsy_performed,
				patch.autopsy_performed_null_flavor,
				patch.autopsy_causes,
			),
		)?;
	}

	Ok(doc.to_string())
}

/// e2b:D.1
fn write_d_1(xpath: &mut Context, value: Option<&str>) {
	if let Some(value) = value {
		set_text_first(xpath, "//hl7:primaryRole/hl7:player1/hl7:name", value);
	}
}

/// e2b:D.2.1
fn write_d_2_1(xpath: &mut Context, value: Option<Date>) {
	if let Some(value) = value {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:birthTime",
			"value",
			&fmt_date(value),
		);
	}
}

/// e2b:D.2.2a
fn write_d_2_2a(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:subjectOf2/hl7:observation[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value", "value", value);
}

/// e2b:D.2.2b
fn write_d_2_2b(xpath: &mut Context, value: Option<&str>) {
	if let Some(value) = value {
		set_attr_first(xpath, "//hl7:subjectOf2/hl7:observation[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value", "unit", value);
	}
}

/// e2b:D.3
fn write_d_3(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:subjectOf2/hl7:observation[hl7:code[@code='7' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value", "value", value);
}

/// e2b:D.4
fn write_d_4(xpath: &mut Context, value: &str) {
	set_attr_first(xpath, "//hl7:subjectOf2/hl7:observation[hl7:code[@code='17' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value", "value", value);
}

/// e2b:D.5
fn write_d_5(xpath: &mut Context, value: Option<&str>) {
	if let Some(value) = value {
		set_attr_first(
			xpath,
			"//hl7:primaryRole/hl7:player1/hl7:administrativeGenderCode",
			"code",
			value,
		);
	}
}

/// e2b:D.9.1
fn write_d_9_1(value: Date) -> String {
	format!("<deceasedTime value=\"{}\"/>", fmt_date(value))
}

/// e2b:D.9.2.r.1a
fn write_d_9_2_r_1a<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.meddra_version
}

/// e2b:D.9.2.r.1b
fn write_d_9_2_r_1b<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.meddra_code
}

/// e2b:D.9.2.r.2
fn write_d_9_2_r_2<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.comments
}

fn reported_cause_fragment(cause: &DPatientDeathCausePatch<'_>) -> String {
	let mut out = String::from(
		"<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"32\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" displayName=\"reportedCauseOfDeath\"/><value xsi:type=\"CE\"",
	);
	if let Some(code) = write_d_9_2_r_1b(cause) {
		out.push_str(" code=\"");
		out.push_str(&xml_escape(code));
		out.push('"');
	}
	if let Some(version) = write_d_9_2_r_1a(cause) {
		out.push_str(" codeSystemVersion=\"");
		out.push_str(&xml_escape(version));
		out.push('"');
	}
	if cause.meddra_code.is_some() {
		out.push_str(" codeSystem=\"2.16.840.1.113883.6.163\"");
	}
	out.push('>');
	if let Some(comments) = write_d_9_2_r_2(cause) {
		out.push_str("<originalText>");
		out.push_str(&xml_escape(comments));
		out.push_str("</originalText>");
	}
	out.push_str("</value></observation></subjectOf2>");
	out
}

/// e2b:D.9.4.r.1a
fn write_d_9_4_r_1a<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.meddra_version
}

/// e2b:D.9.4.r.1b
fn write_d_9_4_r_1b<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.meddra_code
}

/// e2b:D.9.4.r.2
fn write_d_9_4_r_2<'a>(value: &DPatientDeathCausePatch<'a>) -> Option<&'a str> {
	value.comments
}

/// e2b:D.9.3
fn write_d_9_3(
	autopsy_performed: Option<bool>,
	autopsy_performed_null_flavor: Option<&str>,
	causes: &[DPatientDeathCausePatch<'_>],
) -> String {
	let mut out = String::from(
		"<subjectOf2 typeCode=\"SBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"5\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" displayName=\"autopsy\"/><value xsi:type=\"BL\"",
	);
	match autopsy_performed {
		Some(true) => out.push_str(" value=\"true\""),
		Some(false) => out.push_str(" value=\"false\""),
		None if autopsy_performed_null_flavor.is_some() => {
			out.push_str(" nullFlavor=\"");
			out.push_str(&xml_escape(autopsy_performed_null_flavor.unwrap()));
			out.push('"');
		}
		None => return String::new(),
	}
	out.push_str("/>");
	for cause in causes {
		out.push_str("<outboundRelationship2 typeCode=\"DRIV\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"8\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.19\" displayName=\"causeOfDeath\"/><value xsi:type=\"CE\"");
		if let Some(code) = write_d_9_4_r_1b(cause) {
			out.push_str(" code=\"");
			out.push_str(&xml_escape(code));
			out.push('"');
		}
		if let Some(version) = write_d_9_4_r_1a(cause) {
			out.push_str(" codeSystemVersion=\"");
			out.push_str(&xml_escape(version));
			out.push('"');
		}
		if cause.meddra_code.is_some() {
			out.push_str(" codeSystem=\"2.16.840.1.113883.6.163\"");
		}
		out.push('>');
		if let Some(comments) = write_d_9_4_r_2(cause) {
			out.push_str("<originalText>");
			out.push_str(&xml_escape(comments));
			out.push_str("</originalText>");
		}
		out.push_str("</value></observation></outboundRelationship2>");
	}
	out.push_str("</observation></subjectOf2>");
	out
}

fn xml_escape(value: &str) -> String {
	value
		.replace('&', "&amp;")
		.replace('<', "&lt;")
		.replace('>', "&gt;")
		.replace('"', "&quot;")
		.replace('\'', "&apos;")
}
