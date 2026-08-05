use super::*;
use crate::mfds::codes::KR_C_3_1_1;

pub fn patch_c_safety_report(
	raw_xml: &[u8],
	patch: &CSafetyReportPatch,
) -> Result<String> {
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

	write_c_1_1(&mut doc, &parser, &mut xpath, patch.report_unique_id)?;
	write_c_1_2(&mut doc, &parser, &mut xpath, patch)?;
	write_c_1_4(&mut doc, &parser, &mut xpath, patch.date_first_received)?;
	write_c_1_5(&mut doc, &parser, &mut xpath, patch.date_most_recent)?;

	write_c_1_7(&mut doc, &parser, &mut xpath, patch.fulfil_expedited)?;
	write_c_1_6_1(
		&mut doc,
		&parser,
		&mut xpath,
		patch.additional_documents_available,
	)?;
	write_c_1_8_1(&mut doc, &parser, &mut xpath, patch.worldwide_unique_id)?;

	write_fda_c_1_7_1(
		&mut doc,
		&parser,
		&mut xpath,
		patch.local_criteria_report_type,
	)?;
	write_fda_c_1_12(
		&mut doc,
		&parser,
		&mut xpath,
		patch.combination_product_indicator,
		patch.combination_product_indicator_null_flavor,
	)?;
	write_c_1_3(&mut doc, &parser, &mut xpath, patch.report_type)?;
	write_c_1_9_1(&mut doc, &parser, &mut xpath, patch)?;

	write_c_1_11_1(&mut doc, &parser, &mut xpath, patch.nullification_code)?;
	write_c_1_11_2(&mut doc, &parser, &mut xpath, patch.nullification_reason)?;

	write_c_1_8_2(&mut xpath, patch.first_sender_type);

	// C.3 Sender information (best-effort)
	let sender_base = "//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:author/hl7:assignedEntity";
	if let Some(v) = patch.sender_type {
		write_c_3_1(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_health_professional_type_kr1 {
		let kr1_path = &format!(
			"{sender_base}/hl7:subjectOf2/hl7:observation[hl7:code[@code='{KR_C_3_1_1}']]"
		);
		if xpath
			.findnodes(kr1_path, None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				sender_base,
				&format!(
					"<subjectOf2 typeCode=\"SUBJ\">\
						<observation classCode=\"OBS\" moodCode=\"EVN\">\
							<code code=\"{KR_C_3_1_1}\"/>\
							<value xsi:type=\"CE\"/>\
						</observation>\
					</subjectOf2>"
				),
			)?;
		}
		write_c_3_1_kr_1(&mut xpath, sender_base, v);
	} else {
		remove_nodes(
			&mut xpath,
			&format!(
				"{sender_base}/hl7:subjectOf2[hl7:observation/hl7:code[@code='{KR_C_3_1_1}']]"
			),
		);
	}
	if let Some(v) = patch.sender_street_address {
		if xpath
			.findnodes(&format!("{sender_base}/hl7:addr"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				sender_base,
				"<addr/>",
			)?;
		}
		if xpath
			.findnodes(
				&format!("{sender_base}/hl7:addr/hl7:streetAddressLine"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:addr"),
				"<streetAddressLine/>",
			)?;
		}
		write_c_3_4_1(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_city {
		write_c_3_4_2(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_state {
		write_c_3_4_3(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_postcode {
		write_c_3_4_4(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_country_code {
		write_c_3_4_5(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_person_title {
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				sender_base,
				"<assignedPerson/>",
			)?;
		}
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson/hl7:name"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson"),
				"<name/>",
			)?;
		}
		if xpath
			.findnodes(
				&format!("{sender_base}/hl7:assignedPerson/hl7:name/hl7:prefix"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson/hl7:name"),
				"<prefix/>",
			)?;
		}
		write_c_3_3_2(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_person_given_name {
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				sender_base,
				"<assignedPerson/>",
			)?;
		}
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson/hl7:name"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson"),
				"<name/>",
			)?;
		}
		if xpath
			.findnodes(
				&format!("{sender_base}/hl7:assignedPerson/hl7:name/hl7:given"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson/hl7:name"),
				"<given/>",
			)?;
		}
		write_c_3_3_3(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_person_middle_name {
		if xpath
			.findnodes(
				&format!("{sender_base}//hl7:assignedPerson/hl7:name/hl7:given[2]"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}//hl7:assignedPerson/hl7:name"),
				"<given/>",
			)?;
		}
		write_c_3_3_4(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_person_family_name {
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				sender_base,
				"<assignedPerson/>",
			)?;
		}
		if xpath
			.findnodes(&format!("{sender_base}/hl7:assignedPerson/hl7:name"), None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson"),
				"<name/>",
			)?;
		}
		if xpath
			.findnodes(
				&format!("{sender_base}/hl7:assignedPerson/hl7:name/hl7:family"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				&mut doc,
				&parser,
				&mut xpath,
				&format!("{sender_base}/hl7:assignedPerson/hl7:name"),
				"<family/>",
			)?;
		}
		write_c_3_3_5(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_department {
		write_c_3_3_1(&mut xpath, sender_base, v);
	}
	if let Some(v) = patch.sender_org_name {
		write_c_3_2(&mut xpath, sender_base, v);
	}
	write_c_3_4_6(&mut xpath, sender_base, patch.sender_telephone);
	write_c_3_4_7(&mut xpath, sender_base, patch.sender_fax);
	write_c_3_4_8(&mut xpath, sender_base, patch.sender_email);

	Ok(doc.to_string())
}

/// e2b:C.1.1
fn write_c_1_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	report_unique_id: &str,
) -> Result<()> {
	ensure_investigation_id(doc, parser, xpath, "2.16.840.1.113883.3.989.2.1.3.1")?;
	set_attr_first(
		xpath,
		"//hl7:controlActProcess/hl7:subject/hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.1']",
		"extension",
		report_unique_id,
	);
	Ok(())
}

/// e2b:C.1.2
fn write_c_1_2(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	patch: &CSafetyReportPatch<'_>,
) -> Result<()> {
	ensure_control_act_effective_time(doc, parser, xpath)?;
	let path = "//hl7:controlActProcess/hl7:effectiveTime";
	if let Some(transmission_date) = patch.transmission_date {
		remove_attr_first(xpath, path, "nullFlavor");
		let value = patch
			.transmission_date_value
			.filter(|value| is_14_digit_datetime(value))
			.map(str::to_owned)
			.or_else(|| patch.transmission_date_time.map(fmt_offset_datetime))
			.unwrap_or_else(|| transmission_date.to_string());
		set_attr_first(xpath, path, "value", &value);
	}
	Ok(())
}

/// e2b:C.1.4
fn write_c_1_4(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	date_first_received: Option<Date>,
) -> Result<()> {
	ensure_investigation_effective_time(doc, parser, xpath)?;
	let path = "//hl7:investigationEvent/hl7:effectiveTime/hl7:low";
	if let Some(value) = date_first_received {
		remove_attr_first(xpath, path, "nullFlavor");
		set_attr_first(xpath, path, "value", &fmt_date(value));
	}
	Ok(())
}

/// e2b:C.1.5
fn write_c_1_5(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	date_most_recent: Option<Date>,
) -> Result<()> {
	ensure_investigation_availability_time(doc, parser, xpath)?;
	let path = "//hl7:investigationEvent/hl7:availabilityTime";
	if let Some(value) = date_most_recent {
		remove_attr_first(xpath, path, "nullFlavor");
		set_attr_first(xpath, path, "value", &fmt_date(value));
	}
	Ok(())
}

/// e2b:C.1.6.1
fn write_c_1_6_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	additional_documents_available: Option<bool>,
) -> Result<()> {
	let path = "//hl7:component/hl7:observationEvent[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]";
	if let Some(value) = additional_documents_available {
		ensure_observation_event_component(
			doc,
			parser,
			xpath,
			"1",
			"2.16.840.1.113883.3.989.2.1.1.19",
			"BL",
		)?;
		set_attr_first(
			xpath,
			&format!("{path}/hl7:value"),
			"value",
			if value { "true" } else { "false" },
		);
	} else {
		remove_nodes(xpath, path);
	}
	Ok(())
}

/// e2b:C.1.7
fn write_c_1_7(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	fulfil_expedited: bool,
) -> Result<()> {
	ensure_observation_event_component(
		doc,
		parser,
		xpath,
		"23",
		"2.16.840.1.113883.3.989.2.1.1.19",
		"BL",
	)?;
	set_attr_first(
		xpath,
		"//hl7:component/hl7:observationEvent[hl7:code[@code='23' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.19']]/hl7:value",
		"value",
		if fulfil_expedited { "true" } else { "false" },
	);
	Ok(())
}

/// e2b:C.1.8.1
fn write_c_1_8_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	worldwide_id: Option<&str>,
) -> Result<()> {
	let path = "//hl7:controlActProcess/hl7:subject/hl7:investigationEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.2']";
	if let Some(value) = worldwide_id {
		ensure_investigation_id(
			doc,
			parser,
			xpath,
			"2.16.840.1.113883.3.989.2.1.3.2",
		)?;
		set_attr_first(xpath, path, "extension", value);
	} else {
		remove_nodes(xpath, path);
	}
	Ok(())
}

/// e2b:C.1.3
fn write_c_1_3(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	report_type: &str,
) -> Result<()> {
	// Run after component writers to preserve investigationEvent schema order.
	ensure_investigation_characteristic(
		doc,
		parser,
		xpath,
		"1",
		"2.16.840.1.113883.3.989.2.1.1.23",
		Some("2.16.840.1.113883.3.989.2.1.1.2"),
	)?;
	let path = "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]/hl7:value";
	set_attr_first(xpath, path, "type", "CE");
	set_attr_first(xpath, path, "code", report_type);
	remove_nodes(xpath, &format!("{path}/hl7:originalText"));
	Ok(())
}

/// e2b:C.1.9.1
fn write_c_1_9_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	patch: &CSafetyReportPatch,
) -> Result<()> {
	let characteristic = "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]";
	if patch.other_case_identifiers_exist.is_none()
		&& patch.other_case_identifiers_exist_null_flavor.is_none()
	{
		remove_nodes(xpath, characteristic);
		return Ok(());
	}
	ensure_investigation_characteristic(
		doc,
		parser,
		xpath,
		"2",
		"2.16.840.1.113883.3.989.2.1.1.23",
		None,
	)?;
	let path = &format!("{characteristic}/hl7:value");
	remove_attr_first(xpath, path, "type");
	for attr in ["code", "codeSystem", "codeSystemVersion", "displayName"] {
		remove_attr_first(xpath, path, attr);
	}
	set_attr_first(xpath, path, "xsi:type", "BL");
	remove_nodes(xpath, &format!("{path}/hl7:originalText"));
	if let Some(value) = patch.other_case_identifiers_exist {
		remove_attr_first(xpath, path, "nullFlavor");
		set_attr_first(xpath, path, "value", if value { "true" } else { "false" });
	} else if let Some(null_flavor) = patch.other_case_identifiers_exist_null_flavor
	{
		remove_attr_first(xpath, path, "value");
		set_attr_first(xpath, path, "nullFlavor", null_flavor);
	}
	Ok(())
}

/// e2b:C.1.8.2
fn write_c_1_8_2(xpath: &mut Context, first_sender_type: Option<&str>) {
	let relationship =
		"//hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='1']]";
	if let Some(value) = first_sender_type {
		set_attr_first(
			xpath,
			&format!("{relationship}/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:code"),
			"code",
			value,
		);
	} else {
		remove_nodes(xpath, relationship);
	}
}

/// e2b:C.1.11.1
fn write_c_1_11_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	nullification_code: Option<&str>,
) -> Result<()> {
	let characteristic = "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='3' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]";
	if let Some(value) = nullification_code {
		ensure_investigation_characteristic(
			doc,
			parser,
			xpath,
			"3",
			"2.16.840.1.113883.3.989.2.1.1.23",
			None,
		)?;
		let path = format!("{characteristic}/hl7:value");
		set_attr_first(xpath, &path, "type", "CE");
		set_attr_first(xpath, &path, "code", value);
	} else {
		remove_nodes(xpath, characteristic);
	}
	Ok(())
}

/// e2b:C.1.11.2
fn write_c_1_11_2(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	nullification_reason: Option<&str>,
) -> Result<()> {
	let characteristic = "//hl7:investigationEvent/hl7:subjectOf2/hl7:investigationCharacteristic[hl7:code[@code='4' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.23']]";
	if let Some(value) = nullification_reason {
		ensure_investigation_characteristic(
			doc,
			parser,
			xpath,
			"4",
			"2.16.840.1.113883.3.989.2.1.1.23",
			None,
		)?;
		set_text_first(
			xpath,
			&format!("{characteristic}/hl7:value/hl7:originalText"),
			value,
		);
	} else {
		remove_nodes(xpath, characteristic);
	}
	Ok(())
}

/// e2b:FDA.C.1.7.1
fn write_fda_c_1_7_1(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	local_criteria_report_type: Option<&str>,
) -> Result<()> {
	let component = "//hl7:component/hl7:observationEvent[hl7:code[@code='C54588' and @codeSystem='2.16.840.1.113883.3.26.1.1']]";
	if let Some(value) = local_criteria_report_type {
		ensure_observation_event_component(
			doc,
			parser,
			xpath,
			"C54588",
			"2.16.840.1.113883.3.26.1.1",
			"CE",
		)?;
		let path = format!("{component}/hl7:value");
		set_attr_first(xpath, &path, "type", "CE");
		set_attr_first(xpath, &path, "code", value);
		clear_null_flavor_if_export_policy(xpath, "FDA.C.1.7.1.REQUIRED", &path);
	} else {
		remove_nodes(xpath, component);
	}
	Ok(())
}

/// e2b:FDA.C.1.12
fn write_fda_c_1_12(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	combination_product_indicator: Option<&str>,
	combination_product_indicator_null_flavor: Option<&str>,
) -> Result<()> {
	let component = "//hl7:component/hl7:observationEvent[hl7:code[@code='C156384' and @codeSystem='2.16.840.1.113883.3.26.1.1']]";
	if combination_product_indicator.is_none()
		&& combination_product_indicator_null_flavor.is_none()
	{
		remove_nodes(xpath, component);
		return Ok(());
	}
	ensure_observation_event_component(
		doc,
		parser,
		xpath,
		"C156384",
		"2.16.840.1.113883.3.26.1.1",
		"BL",
	)?;
	let path = format!("{component}/hl7:value");
	if let Some(value) = combination_product_indicator {
		set_attr_first(
			xpath,
			&path,
			"value",
			normalize_bl_value(value).unwrap_or("false"),
		);
		clear_null_flavor_if_export_policy(xpath, "FDA.C.1.12.REQUIRED", &path);
	} else if let Some(null_flavor) = combination_product_indicator_null_flavor {
		remove_attr_first(xpath, &path, "value");
		set_attr_first(xpath, &path, "nullFlavor", null_flavor);
	}
	Ok(())
}

/// e2b:C.3.1
fn write_c_3_1(xpath: &mut Context, base: &str, value: &str) {
	set_attr_first(xpath, &format!("{base}/hl7:code"), "code", value);
}

/// e2b:C.3.1.KR.1
fn write_c_3_1_kr_1(xpath: &mut Context, base: &str, value: &str) {
	set_attr_first(
		xpath,
		&format!("{base}/hl7:subjectOf2/hl7:observation[hl7:code[@code='{KR_C_3_1_1}']]/hl7:value"),
		"code",
		value,
	);
}

/// e2b:C.3.2
fn write_c_3_2(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}/hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name"),
		value,
	);
}

/// e2b:C.3.3.1
fn write_c_3_3_1(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}/hl7:representedOrganization/hl7:name"),
		value,
	);
}

/// e2b:C.3.3.2
fn write_c_3_3_2(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}//hl7:assignedPerson/hl7:name/hl7:prefix"),
		value,
	);
}

/// e2b:C.3.3.3
fn write_c_3_3_3(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}//hl7:assignedPerson/hl7:name/hl7:given"),
		value,
	);
}

/// e2b:C.3.3.4
fn write_c_3_3_4(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}//hl7:assignedPerson/hl7:name/hl7:given[2]"),
		value,
	);
}

/// e2b:C.3.3.5
fn write_c_3_3_5(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}//hl7:assignedPerson/hl7:name/hl7:family"),
		value,
	);
}

/// e2b:C.3.4.1
fn write_c_3_4_1(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(
		xpath,
		&format!("{base}/hl7:addr/hl7:streetAddressLine"),
		value,
	);
}

/// e2b:C.3.4.2
fn write_c_3_4_2(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(xpath, &format!("{base}/hl7:addr/hl7:city"), value);
}

/// e2b:C.3.4.3
fn write_c_3_4_3(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(xpath, &format!("{base}/hl7:addr/hl7:state"), value);
}

/// e2b:C.3.4.4
fn write_c_3_4_4(xpath: &mut Context, base: &str, value: &str) {
	set_text_first(xpath, &format!("{base}/hl7:addr/hl7:postalCode"), value);
}

/// e2b:C.3.4.5
fn write_c_3_4_5(xpath: &mut Context, base: &str, value: &str) {
	set_attr_first(
		xpath,
		&format!(
			"{base}//hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code"
		),
		"code",
		value,
	);
}

/// e2b:C.3.4.6
fn write_c_3_4_6(xpath: &mut Context, base: &str, value: Option<&str>) {
	write_c_3_4_telecom(xpath, base, "tel:", value);
}

/// e2b:C.3.4.7
fn write_c_3_4_7(xpath: &mut Context, base: &str, value: Option<&str>) {
	write_c_3_4_telecom(xpath, base, "fax:", value);
}

/// e2b:C.3.4.8
fn write_c_3_4_8(xpath: &mut Context, base: &str, value: Option<&str>) {
	write_c_3_4_telecom(xpath, base, "mailto:", value);
}

fn write_c_3_4_telecom(
	xpath: &mut Context,
	base: &str,
	prefix: &str,
	value: Option<&str>,
) {
	let path = format!("{base}/hl7:telecom[starts-with(@value,'{prefix}')]");
	let value = value
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.and_then(|value| {
			let body = value.strip_prefix(prefix).unwrap_or(value).trim();
			(!body.is_empty()).then(|| format!("{prefix}{body}"))
		});

	if let Some(value) = value {
		set_attr_first(xpath, &path, "value", &value);
	} else {
		remove_nodes(xpath, &path);
	}
}
