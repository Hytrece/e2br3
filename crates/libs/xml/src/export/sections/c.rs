use super::n::fetch_message_header;
use super::n::fetch_primary_source;
use super::*;
use crate::export::roundtrip::{patch_c_safety_report, CSafetyReportPatch};
use crate::mfds::codes::KR_C_5_4_1;
use lib_core::model::case_identifiers::{LinkedReportNumber, OtherCaseIdentifier};
use lib_core::model::safety_report::{
	DocumentsHeldBySender, SafetyReportIdentification, StudyFdaCrossReportedInd,
};

pub(crate) async fn export_patch(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	case: &Case,
	raw_xml: &[u8],
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	let report = SafetyReportIdentificationBmc::get_by_case(ctx, mm, case_id)
		.await
		.map_err(Error::from)?;
	let sender = fetch_sender_information(mm, case_id).await?;
	let header = fetch_message_header(ctx, mm, case_id).await?;
	export_c_safety_report_patch(
		raw_xml,
		case,
		&report,
		header.as_ref(),
		sender.as_ref(),
		authority,
	)
}

async fn fetch_sender_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<SenderInformation>> {
	mm.dbx()
		.fetch_optional(
			sqlx::query_as::<_, SenderInformation>(
				"SELECT * FROM sender_information WHERE case_id = $1 ORDER BY created_at LIMIT 1",
			)
			.bind(case_id),
		)
		.await
		.map_err(model::Error::from)
		.map_err(Error::from)
}

fn set_text_or_null_flavor(
	xpath: &mut Context,
	path: &str,
	value: Option<&str>,
	null_flavor: Option<&str>,
) {
	let value = value.map(str::trim).filter(|value| !value.is_empty());
	let null_flavor = null_flavor.map(str::trim).filter(|value| !value.is_empty());
	if let Ok(nodes) = xpath.findnodes(path, None) {
		for mut node in nodes.into_iter().take(1) {
			if let Some(value) = value {
				let _ = node.remove_attribute("nullFlavor");
				let _ = node.set_content(value);
			} else if let Some(null_flavor) = null_flavor {
				let _ = node.set_content("");
				let _ = node.set_attribute("nullFlavor", null_flavor);
			}
		}
	}
}

fn set_telecom_or_null_flavor(
	xpath: &mut Context,
	path: &str,
	value: Option<&str>,
	null_flavor: Option<&str>,
) {
	let value = value.map(str::trim).filter(|value| !value.is_empty());
	let null_flavor = null_flavor.map(str::trim).filter(|value| !value.is_empty());
	if let Ok(nodes) = xpath.findnodes(path, None) {
		for mut node in nodes.into_iter().take(1) {
			if let Some(value) = value {
				let telecom_value = if value.contains(':') {
					value.to_string()
				} else {
					format!("tel:{value}")
				};
				let _ = node.remove_attribute("nullFlavor");
				let _ = node.set_attribute("value", &telecom_value);
			} else if let Some(null_flavor) = null_flavor {
				let _ = node.set_attribute("value", "tel");
				let _ = node.set_attribute("nullFlavor", null_flavor);
			}
		}
	}
}

pub(crate) async fn apply_primary_source_section(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let Some(primary) = fetch_primary_source(mm, case_id).await? else {
		return Ok(());
	};
	apply_primary_source_values(doc, parser, xpath, &primary, authority)
}

fn apply_primary_source_values(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	primary: &PrimarySource,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let base = "//hl7:investigationEvent/hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity";
	ensure_primary_source_author_nodes(doc, parser, xpath)?;
	if xpath
		.findnodes(&format!("{base}/hl7:representedOrganization"), None)
		.map(|nodes| nodes.is_empty())
		.unwrap_or(true)
	{
		append_fragment_child(
			doc,
			parser,
			xpath,
			base,
			"<representedOrganization classCode=\"ORG\" determinerCode=\"INSTANCE\"><name/></representedOrganization>",
		)?;
	}

	write_c_2_r_1_1(xpath, base, primary);
	write_c_2_r_1_2(xpath, base, primary);
	if primary
		.reporter_middle_name
		.as_deref()
		.is_some_and(|v| !v.trim().is_empty())
		|| primary
			.reporter_middle_name_null_flavor
			.as_deref()
			.is_some_and(|v| !v.trim().is_empty())
	{
		if xpath
			.findnodes(
				&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given[2]"),
				None,
			)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				doc,
				parser,
				xpath,
				&format!("{base}/hl7:assignedPerson/hl7:name"),
				"<given/>",
			)?;
		}
		write_c_2_r_1_3(xpath, base, primary);
	}
	write_c_2_r_1_4(xpath, base, primary);
	let has_department = primary
		.department
		.as_deref()
		.is_some_and(|v| !v.trim().is_empty())
		|| primary
			.department_null_flavor
			.as_deref()
			.is_some_and(|v| !v.trim().is_empty());
	let organization_path = if has_department {
		let nested = format!("{base}/hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name");
		if xpath
			.findnodes(&nested, None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				doc,
				parser,
				xpath,
				&format!("{base}/hl7:representedOrganization"),
				"<assignedEntity classCode=\"ASSIGNED\"><representedOrganization classCode=\"ORG\" determinerCode=\"INSTANCE\"><name/></representedOrganization></assignedEntity>",
			)?;
		}
		nested
	} else {
		format!("{base}/hl7:representedOrganization/hl7:name")
	};
	write_c_2_r_2_1(xpath, &organization_path, primary);
	if has_department {
		write_c_2_r_2_2(xpath, base, primary);
	}
	write_c_2_r_2_3(xpath, base, primary);
	write_c_2_r_2_4(xpath, base, primary);
	write_c_2_r_2_5(xpath, base, primary);
	write_c_2_r_2_6(xpath, base, primary);
	write_c_2_r_2_7(xpath, base, primary);
	if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Fda) {
		write_fda_c_2_r_2_8(xpath, base, primary);
	} else {
		remove_nodes(
			xpath,
			&format!("{base}/hl7:telecom[starts-with(@value,'mailto:')]"),
		);
	}
	write_c_2_r_3(xpath, base, primary);
	if primary.qualification.is_some() || primary.qualification_null_flavor.is_some()
	{
		let path = format!("{base}/hl7:assignedPerson/hl7:asQualifiedEntity");
		if xpath
			.findnodes(&path, None)
			.map(|nodes| nodes.is_empty())
			.unwrap_or(true)
		{
			append_fragment_child(
				doc,
				parser,
				xpath,
				&format!("{base}/hl7:assignedPerson"),
				"<asQualifiedEntity><code/></asQualifiedEntity>",
			)?;
		}
	}
	write_c_2_r_4(xpath, base, primary);
	if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Mfds) {
		write_c_2_r_4_kr_1(xpath, base, primary);
	}
	write_c_2_r_5(xpath, primary);

	Ok(())
}

/// e2b:C.2.r.1.1
fn write_c_2_r_1_1(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:assignedPerson/hl7:name/hl7:prefix"),
		value.reporter_title.as_deref(),
		value.reporter_title_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.1.2
fn write_c_2_r_1_2(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given[1]"),
		value.reporter_given_name.as_deref(),
		value.reporter_given_name_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.1.3
fn write_c_2_r_1_3(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:assignedPerson/hl7:name/hl7:given[2]"),
		value.reporter_middle_name.as_deref(),
		value.reporter_middle_name_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.1.4
fn write_c_2_r_1_4(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:assignedPerson/hl7:name/hl7:family"),
		value.reporter_family_name.as_deref(),
		value.reporter_family_name_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.1
fn write_c_2_r_2_1(xpath: &mut Context, path: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		path,
		value.organization.as_deref(),
		value.organization_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.2
fn write_c_2_r_2_2(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:representedOrganization/hl7:name"),
		value.department.as_deref(),
		value.department_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.3
fn write_c_2_r_2_3(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:addr/hl7:streetAddressLine"),
		value.street.as_deref(),
		value.street_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.4
fn write_c_2_r_2_4(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:addr/hl7:city"),
		value.city.as_deref(),
		value.city_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.5
fn write_c_2_r_2_5(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:addr/hl7:state"),
		value.state.as_deref(),
		value.state_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.6
fn write_c_2_r_2_6(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_text_or_null_flavor(
		xpath,
		&format!("{base}/hl7:addr/hl7:postalCode"),
		value.postcode.as_deref(),
		value.postcode_null_flavor.as_deref(),
	);
}

/// e2b:C.2.r.2.7
fn write_c_2_r_2_7(xpath: &mut Context, base: &str, value: &PrimarySource) {
	set_telecom_or_null_flavor(
		xpath,
		&format!("{base}/hl7:telecom[starts-with(@value,'tel:') or @nullFlavor]"),
		value.telephone.as_deref(),
		value.telephone_null_flavor.as_deref(),
	);
}

/// e2b:FDA.C.2.r.2.8
fn write_fda_c_2_r_2_8(xpath: &mut Context, base: &str, value: &PrimarySource) {
	let path = format!("{base}/hl7:telecom[starts-with(@value,'mailto')]");
	if let Some(email) = value.email.as_deref() {
		let email = if email.contains(':') {
			email.to_string()
		} else {
			format!("mailto:{email}")
		};
		set_attr_first(xpath, &path, "value", &email);
		remove_attr_first(xpath, &path, "nullFlavor");
	} else if let Some(null_flavor) = value.email_null_flavor.as_deref() {
		set_attr_first(xpath, &path, "value", "mailto");
		set_attr_first(xpath, &path, "nullFlavor", null_flavor);
	}
}

/// e2b:C.2.r.3
fn write_c_2_r_3(xpath: &mut Context, base: &str, value: &PrimarySource) {
	let path = format!(
		"{base}/hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code"
	);
	if let Some(code) = value.country_code.as_deref() {
		set_attr_first(xpath, &path, "code", code);
	} else {
		if let Ok(nodes) = xpath.findnodes(&path, None) {
			for mut node in nodes.into_iter().take(1) {
				node.unlink_node();
			}
		}
	}
}

/// e2b:C.2.r.4
fn write_c_2_r_4(xpath: &mut Context, base: &str, value: &PrimarySource) {
	let path = format!("{base}/hl7:assignedPerson/hl7:asQualifiedEntity/hl7:code");
	match (
		value.qualification.as_deref(),
		value.qualification_null_flavor.as_deref(),
	) {
		(Some(code), _) => {
			set_attr_first(xpath, &path, "code", code);
			remove_attr_first(xpath, &path, "nullFlavor");
		}
		(None, Some(null_flavor)) => {
			set_attr_first(xpath, &path, "nullFlavor", null_flavor);
			remove_attr_first(xpath, &path, "code");
		}
		(None, None) => {}
	}
}

/// e2b:C.2.r.4.KR.1
fn write_c_2_r_4_kr_1(_xpath: &mut Context, _base: &str, _value: &PrimarySource) {
	// No XML mapping exists in the current MFDS profile.
}

/// e2b:C.2.r.5
fn write_c_2_r_5(xpath: &mut Context, value: &PrimarySource) {
	if let Some(priority) = value.primary_source_regulatory.as_deref() {
		set_attr_first(xpath, "//hl7:investigationEvent/hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:priorityNumber", "value", priority);
	}
}

fn ensure_primary_source_author_nodes(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
) -> Result<()> {
	let base = "//hl7:investigationEvent/hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:author/hl7:assignedEntity";
	if xpath
		.findnodes(base, None)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false)
	{
		return Ok(());
	}

	append_fragment_child(
		doc,
		parser,
		xpath,
		"//hl7:investigationEvent/hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:relatedInvestigation",
		"<subjectOf2 typeCode=\"SUBJ\"><controlActEvent classCode=\"CACT\" moodCode=\"EVN\"><author typeCode=\"AUT\"><assignedEntity classCode=\"ASSIGNED\"><assignedPerson classCode=\"PSN\" determinerCode=\"INSTANCE\"><name><prefix/><given/><family/></name></assignedPerson><representedOrganization classCode=\"ORG\" determinerCode=\"INSTANCE\"><name/></representedOrganization></assignedEntity></author></controlActEvent></subjectOf2>",
	)
}

pub(crate) async fn apply_report_relationships_section(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let documents = mm.dbx().fetch_all(sqlx::query_as::<_, DocumentsHeldBySender>("SELECT * FROM documents_held_by_sender WHERE case_id = $1 AND deleted = false ORDER BY sequence_number").bind(case_id)).await.map_err(model::Error::from)?;
	let identifiers = mm.dbx().fetch_all(sqlx::query_as::<_, OtherCaseIdentifier>("SELECT * FROM other_case_identifiers WHERE case_id = $1 AND deleted = false ORDER BY sequence_number").bind(case_id)).await.map_err(model::Error::from)?;
	let linked_reports = mm.dbx().fetch_all(sqlx::query_as::<_, LinkedReportNumber>("SELECT * FROM linked_report_numbers WHERE case_id = $1 AND deleted = false ORDER BY sequence_number").bind(case_id)).await.map_err(model::Error::from)?;

	remove_nodes(
		xpath,
		"//hl7:investigationEvent/hl7:reference[hl7:document/hl7:code[@code='1']]",
	);
	remove_nodes(xpath, "//hl7:investigationEvent/hl7:subjectOf1[hl7:controlActEvent/hl7:id[@root='2.16.840.1.113883.3.989.2.1.3.3']]");
	remove_nodes(xpath, "//hl7:investigationEvent/hl7:outboundRelationship[hl7:relatedInvestigation/hl7:code/@nullFlavor='NA']");

	let mut fragment = String::new();
	for value in identifiers {
		fragment.push_str(&format!("<subjectOf1 typeCode=\"SUBJ\"><controlActEvent classCode=\"CACT\" moodCode=\"EVN\"><id root=\"2.16.840.1.113883.3.989.2.1.3.3\" assigningAuthorityName=\"{}\" extension=\"{}\"/></controlActEvent></subjectOf1>", write_c_1_9_1_r_1(&value), write_c_1_9_1_r_2(&value)));
	}
	for value in linked_reports {
		fragment.push_str(&write_c_1_10_r(&value));
	}
	for value in documents {
		fragment.push_str(&format!("<reference typeCode=\"REFR\"><document classCode=\"DOC\" moodCode=\"EVN\"><code code=\"1\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.27\"/>{}{}</document></reference>", write_c_1_6_1_r_1(&value), write_c_1_6_1_r_2(&value, authority)?));
	}
	if fragment.is_empty() {
		return Ok(());
	}
	let Some(xml) =
		inject_fragment_in_investigation_event(&doc.to_string(), &fragment)
	else {
		return Ok(());
	};
	*doc = parser.parse_string(&xml).map_err(|err| Error::InvalidXml {
		message: format!("XML parse error after C relationship injection: {err}"),
		line: None,
		column: None,
	})?;
	*xpath = Context::new(doc).map_err(|_| Error::InvalidXml {
		message: "Failed to initialize XPath after C relationship injection"
			.to_string(),
		line: None,
		column: None,
	})?;
	let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
	let _ =
		xpath.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");
	Ok(())
}

/// e2b:C.1.6.1.r.1
fn write_c_1_6_1_r_1(value: &DocumentsHeldBySender) -> String {
	value
		.title
		.as_deref()
		.filter(|v| !v.trim().is_empty())
		.map(|v| format!("<title>{}</title>", xml_escape(v)))
		.unwrap_or_default()
}

/// e2b:C.1.6.1.r.2
fn write_c_1_6_1_r_2(
	value: &DocumentsHeldBySender,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	write_attachment_text(
		value.document_base64.as_deref(),
		value.file_name.as_deref(),
		value.media_type.as_deref(),
		value.representation.as_deref(),
		value.compression.as_deref(),
		authority,
		"C.1.6.1.r.2",
	)
}

fn write_attachment_text(
	document: Option<&str>,
	file_name: Option<&str>,
	media_type: Option<&str>,
	representation: Option<&str>,
	compression: Option<&str>,
	authority: lib_core::regulatory::RegulatoryAuthority,
	field_code: &str,
) -> Result<String> {
	let Some(document) = document.filter(|value| !value.trim().is_empty()) else {
		return Ok(String::new());
	};
	let file_name = file_name.map(str::trim).filter(|value| !value.is_empty());
	let media_type = media_type
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.unwrap_or("application/octet-stream");
	if authority == lib_core::regulatory::RegulatoryAuthority::Fda {
		let file_name = file_name.ok_or_else(|| Error::InvalidXml {
			message: format!("FDA {field_code} attachment file name is required"),
			line: None,
			column: None,
		})?;
		let expected = lib_core::regulatory::fda_attachment_media_type(file_name)
			.ok_or_else(|| Error::InvalidXml {
				message: format!(
					"FDA {field_code} attachment file type is not supported: {file_name}"
				),
				line: None,
				column: None,
			})?;
		if !media_type.eq_ignore_ascii_case(expected) {
			return Err(Error::InvalidXml {
				message: format!("FDA {field_code} attachment media type '{media_type}' does not match file name '{file_name}'"),
				line: None,
				column: None,
			});
		}
	}
	let reference = file_name
		.map(|value| format!("<reference value=\"{}\"/>", xml_escape(value)))
		.unwrap_or_default();
	let representation = representation
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.unwrap_or("B64");
	let compression = compression
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.map(|value| format!(" compression=\"{}\"", xml_escape(value)))
		.unwrap_or_default();
	Ok(format!(
		"<text mediaType=\"{}\" representation=\"{}\"{}>{}{}</text>",
		xml_escape(media_type),
		xml_escape(representation),
		compression,
		reference,
		xml_escape(document)
	))
}

/// e2b:C.1.9.1.r.1
fn write_c_1_9_1_r_1(value: &OtherCaseIdentifier) -> String {
	xml_escape(&value.source_of_identifier)
}

/// e2b:C.1.9.1.r.2
fn write_c_1_9_1_r_2(value: &OtherCaseIdentifier) -> String {
	xml_escape(&value.case_identifier)
}

/// e2b:C.1.10.r
fn write_c_1_10_r(value: &LinkedReportNumber) -> String {
	format!("<outboundRelationship typeCode=\"SPRT\"><relatedInvestigation classCode=\"INVSTG\" moodCode=\"EVN\"><code nullFlavor=\"NA\"/><subjectOf2 typeCode=\"SUBJ\"><controlActEvent classCode=\"CACT\" moodCode=\"EVN\"><id root=\"2.16.840.1.113883.3.989.2.1.3.2\" extension=\"{}\"/></controlActEvent></subjectOf2></relatedInvestigation></outboundRelationship>", xml_escape(&value.linked_report_number))
}

pub fn export_c_safety_report_patch(
	raw_xml: &[u8],
	_case: &Case,
	report: &SafetyReportIdentification,
	header: Option<&MessageHeader>,
	sender: Option<&SenderInformation>,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	if report.fulfil_expedited_criteria_null_flavor.is_some() {
		return Err(Error::InvalidXml {
			message:
				"ICH.C.1.7 NI export requires verified E2B(R2)-origin provenance"
					.to_string(),
			line: None,
			column: None,
		});
	}
	let combination_true = report
		.combination_product_report_indicator
		.as_deref()
		.map(is_true_like)
		.unwrap_or(false);
	let local_criteria_report_type =
		if !report.fulfil_expedited_criteria.unwrap_or(false) && !combination_true {
			Some("2")
		} else {
			report.local_criteria_report_type.as_deref()
		};

	let patch = CSafetyReportPatch {
		report_unique_id: report.safety_report_id.as_deref().unwrap_or(""),
		transmission_date: report.transmission_date.as_deref(),
		transmission_date_value: header.map(|h| h.message_date.as_str()),
		transmission_date_time: header.and_then(|h| h.batch_transmission_date),
		report_type: report.report_type.as_deref().unwrap_or(""),
		date_first_received: report.date_first_received_from_source,
		date_most_recent: report.date_of_most_recent_information,
		fulfil_expedited: report.fulfil_expedited_criteria.unwrap_or(false),
		additional_documents_available: report.additional_documents_available,
		other_case_identifiers_exist: report.other_case_identifiers_exist,
		other_case_identifiers_exist_null_flavor: report
			.other_case_identifiers_exist_null_flavor
			.as_deref(),
		worldwide_unique_id: report.worldwide_unique_id.as_deref(),
		first_sender_type: report.first_sender_type.as_deref(),
		local_criteria_report_type: matches!(
			authority,
			lib_core::regulatory::RegulatoryAuthority::Fda
		)
		.then_some(local_criteria_report_type)
		.flatten(),
		combination_product_indicator: matches!(
			authority,
			lib_core::regulatory::RegulatoryAuthority::Fda
		)
		.then_some(report.combination_product_report_indicator.as_deref())
		.flatten(),
		combination_product_indicator_null_flavor: matches!(
			authority,
			lib_core::regulatory::RegulatoryAuthority::Fda
		)
		.then_some(
			report
				.combination_product_report_indicator_null_flavor
				.as_deref(),
		)
		.flatten(),
		nullification_code: report.nullification_code.as_deref(),
		nullification_reason: report.nullification_reason.as_deref(),
		sender_type: sender.and_then(|s| s.sender_type.as_deref()),
		sender_health_professional_type_kr1: matches!(
			authority,
			lib_core::regulatory::RegulatoryAuthority::Mfds
		)
		.then(|| sender.and_then(|s| s.health_professional_type_kr1.as_deref()))
		.flatten(),
		sender_org_name: sender.and_then(|s| s.organization_name.as_deref()),
		sender_department: sender.and_then(|s| s.department.as_deref()),
		sender_street_address: sender.and_then(|s| s.street_address.as_deref()),
		sender_city: sender.and_then(|s| s.city.as_deref()),
		sender_state: sender.and_then(|s| s.state.as_deref()),
		sender_postcode: sender.and_then(|s| s.postcode.as_deref()),
		sender_country_code: sender.and_then(|s| s.country_code.as_deref()),
		sender_person_title: sender.and_then(|s| s.person_title.as_deref()),
		sender_person_given_name: sender
			.and_then(|s| s.person_given_name.as_deref()),
		sender_person_middle_name: sender
			.and_then(|s| s.person_middle_name.as_deref()),
		sender_person_family_name: sender
			.and_then(|s| s.person_family_name.as_deref()),
		sender_telephone: sender.and_then(|s| s.telephone.as_deref()),
		sender_fax: sender.and_then(|s| s.fax.as_deref()),
		sender_email: sender.and_then(|s| s.email.as_deref()),
	};

	patch_c_safety_report(raw_xml, &patch)
}

fn is_true_like(value: &str) -> bool {
	matches!(
		value.trim().to_ascii_lowercase().as_str(),
		"true" | "1" | "y" | "yes"
	)
}

#[cfg(test)]
mod primary_source_null_flavor_tests {
	use super::*;
	use sqlx::types::time::OffsetDateTime;
	use sqlx::types::Uuid;
	use std::collections::BTreeSet;

	fn source() -> PrimarySource {
		PrimarySource {
			id: Uuid::nil(),
			case_id: Uuid::nil(),
			source_reporter_presave_id: None,
			sequence_number: 1,
			reporter_title: None,
			reporter_title_null_flavor: None,
			reporter_given_name: None,
			reporter_given_name_null_flavor: Some("ASKU".to_string()),
			reporter_middle_name: None,
			reporter_middle_name_null_flavor: None,
			reporter_family_name: None,
			reporter_family_name_null_flavor: None,
			organization: None,
			organization_null_flavor: None,
			department: None,
			department_null_flavor: None,
			street: None,
			street_null_flavor: None,
			city: None,
			city_null_flavor: Some("NASK".to_string()),
			state: None,
			state_null_flavor: None,
			postcode: None,
			postcode_null_flavor: None,
			telephone: None,
			telephone_null_flavor: None,
			country_code: None,
			email: None,
			email_null_flavor: None,
			qualification: None,
			qualification_null_flavor: None,
			qualification_kr1: None,
			primary_source_regulatory: None,
			deleted: false,
			created_at: OffsetDateTime::UNIX_EPOCH,
			updated_at: OffsetDateTime::UNIX_EPOCH,
			created_by: Uuid::nil(),
			updated_by: None,
		}
	}

	#[test]
	fn primary_source_export_isolates_element_null_flavors() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><relatedInvestigation><code code="2"/><subjectOf2><controlActEvent><author><assignedEntity><assignedPerson><name><prefix/><given/><family/></name></assignedPerson><representedOrganization><name/></representedOrganization><addr><streetAddressLine/><city/><state/><postalCode/></addr><telecom value="tel:"/></assignedEntity></author></controlActEvent></subjectOf2></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let parser = Parser::default();
		let mut doc = parser.parse_string(xml).expect("parse");
		let mut xpath = Context::new(&doc).expect("xpath");
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();

		apply_primary_source_values(
			&mut doc,
			&parser,
			&mut xpath,
			&source(),
			lib_core::regulatory::RegulatoryAuthority::Fda,
		)
		.expect("apply primary source");

		assert_eq!(
			xpath
				.findvalue(
					"//hl7:assignedPerson/hl7:name/hl7:given[1]/@nullFlavor",
					None
				)
				.unwrap(),
			"ASKU"
		);
		assert_eq!(
			xpath
				.findvalue(
					"//hl7:assignedEntity/hl7:addr/hl7:city/@nullFlavor",
					None
				)
				.unwrap(),
			"NASK"
		);
		assert_eq!(
			xpath
				.findvalue(
					"//hl7:assignedPerson/hl7:name/hl7:prefix/@nullFlavor",
					None
				)
				.unwrap(),
			""
		);
		assert_eq!(
			xpath
				.findvalue(
					"//hl7:assignedEntity/hl7:addr/hl7:state/@nullFlavor",
					None
				)
				.unwrap(),
			""
		);
	}

	#[test]
	fn primary_source_export_switches_qualification_value_and_null_flavor() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><relatedInvestigation><code code="2"/><subjectOf2><controlActEvent><author><assignedEntity><assignedPerson><name/></assignedPerson><representedOrganization><name/></representedOrganization><addr/></assignedEntity></author></controlActEvent></subjectOf2></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let parser = Parser::default();
		let mut doc = parser.parse_string(xml).expect("parse");
		let mut xpath = Context::new(&doc).expect("xpath");
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut primary = source();
		primary.qualification_null_flavor = Some("UNK".to_string());

		apply_primary_source_values(
			&mut doc,
			&parser,
			&mut xpath,
			&primary,
			lib_core::regulatory::RegulatoryAuthority::Ich,
		)
		.expect("apply null flavor");
		let path = "//hl7:asQualifiedEntity/hl7:code";
		assert_eq!(
			xpath
				.findvalue(&format!("{path}/@nullFlavor"), None)
				.unwrap(),
			"UNK"
		);
		assert_eq!(xpath.findvalue(&format!("{path}/@code"), None).unwrap(), "");

		primary.qualification = Some("3".to_string());
		primary.qualification_null_flavor = None;
		apply_primary_source_values(
			&mut doc,
			&parser,
			&mut xpath,
			&primary,
			lib_core::regulatory::RegulatoryAuthority::Ich,
		)
		.expect("apply value");
		assert_eq!(
			xpath.findvalue(&format!("{path}/@code"), None).unwrap(),
			"3"
		);
		assert_eq!(
			xpath
				.findvalue(&format!("{path}/@nullFlavor"), None)
				.unwrap(),
			""
		);
	}

	#[test]
	fn primary_source_email_is_fda_only() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><relatedInvestigation><code code="2"/><subjectOf2><controlActEvent><author><assignedEntity><assignedPerson><name/></assignedPerson><representedOrganization><name/></representedOrganization><addr/><telecom value="mailto:old@example.com"/></assignedEntity></author></controlActEvent></subjectOf2></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let parser = Parser::default();
		let mut doc = parser.parse_string(xml).expect("parse");
		let mut xpath = Context::new(&doc).expect("xpath");
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut primary = source();
		primary.email = Some("new@example.com".to_string());

		apply_primary_source_values(
			&mut doc,
			&parser,
			&mut xpath,
			&primary,
			lib_core::regulatory::RegulatoryAuthority::Mfds,
		)
		.expect("apply primary source");

		assert!(!doc.to_string().contains("mailto:"));
	}

	#[test]
	fn fda_telecom_null_flavors_keep_required_discriminators() {
		let xml = br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><PORR_IN049016UV><controlActProcess><subject><investigationEvent><outboundRelationship><relatedInvestigation><code code="2"/><subjectOf2><controlActEvent><author><assignedEntity><assignedPerson><name/></assignedPerson><representedOrganization><name/></representedOrganization><addr/><telecom value="tel:"/><telecom value="mailto:"/></assignedEntity></author></controlActEvent></subjectOf2></relatedInvestigation></outboundRelationship></investigationEvent></subject></controlActProcess></PORR_IN049016UV></MCCI_IN200100UV01>"#;
		let parser = Parser::default();
		let mut doc = parser.parse_string(xml).expect("parse");
		let mut xpath = Context::new(&doc).expect("xpath");
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut primary = source();
		primary.telephone = None;
		primary.telephone_null_flavor = Some("NASK".to_string());
		primary.email = None;
		primary.email_null_flavor = Some("ASKU".to_string());

		apply_primary_source_values(
			&mut doc,
			&parser,
			&mut xpath,
			&primary,
			lib_core::regulatory::RegulatoryAuthority::Fda,
		)
		.expect("apply primary source");

		let output = doc.to_string();
		assert!(
			output.contains("value=\"tel\" nullFlavor=\"NASK\"")
				|| output.contains("nullFlavor=\"NASK\" value=\"tel\"")
		);
		assert!(
			output.contains("value=\"mailto\" nullFlavor=\"ASKU\"")
				|| output.contains("nullFlavor=\"ASKU\" value=\"mailto\"")
		);
	}

	#[test]
	fn fda_attachment_keeps_file_name_and_checks_media_type() {
		let authority = lib_core::regulatory::RegulatoryAuthority::Fda;
		let xml = write_attachment_text(
			Some("QUJD"),
			Some("report.pdf"),
			Some("application/pdf"),
			Some("B64"),
			None,
			authority,
			"C.4.r.2",
		)
		.expect("valid FDA attachment");
		assert!(xml.contains("<reference value=\"report.pdf\"/>QUJD"));
		assert!(write_attachment_text(
			Some("QUJD"),
			Some("report.pdf"),
			Some("text/plain"),
			Some("B64"),
			None,
			authority,
			"C.4.r.2",
		)
		.is_err());
	}

	#[test]
	fn section_c_writers_cover_registry_fields() {
		let registry: serde_json::Value = serde_json::from_str(include_str!(
			"../../../../../../registry/sections/c-safety-report.json"
		))
		.expect("section C registry");
		let expected = registry
			.as_array()
			.expect("registry array")
			.iter()
			.filter(|entry| entry["local_only"] != true)
			.filter_map(|entry| entry["e2br3_code"].as_str())
			.collect::<BTreeSet<_>>();
		let source = format!(
			"{}\n{}",
			include_str!("c.rs"),
			include_str!("../roundtrip/c_safety_report.rs")
		);
		let implemented = source
			.lines()
			.filter_map(|line| line.trim().strip_prefix("/// e2b:"))
			.collect::<BTreeSet<_>>();

		assert_eq!(implemented, expected);
	}
}

pub(crate) async fn apply_literature_section(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let references = fetch_literature_references(mm, case_id).await?;
	if references.is_empty() {
		return Ok(());
	}

	remove_nodes(
		xpath,
		"//hl7:investigationEvent/hl7:reference[hl7:document/hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.27']]",
	);

	let mut fragment = String::new();
	for item in references {
		let bibliographic = write_c_4_r_1(&item);
		let attachment = write_c_4_r_2(&item, authority)?;
		fragment.push_str(&format!(
			"<reference typeCode=\"REFR\"><document classCode=\"DOC\" moodCode=\"EVN\"><code code=\"2\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.27\"/>{}{}</document></reference>",
			bibliographic,
			attachment
		));
	}

	let xml = doc.to_string();
	if let Some(injected) = inject_fragment_in_investigation_event(&xml, &fragment) {
		let new_doc =
			parser
				.parse_string(&injected)
				.map_err(|err| Error::InvalidXml {
					message: format!(
						"XML parse error after literature injection: {err}"
					),
					line: None,
					column: None,
				})?;
		*doc = new_doc;
		*xpath = Context::new(doc).map_err(|_| Error::InvalidXml {
			message: "Failed to initialize XPath context after literature injection"
				.to_string(),
			line: None,
			column: None,
		})?;
		let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
		let _ = xpath
			.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");
	}
	Ok(())
}

/// e2b:C.4.r.1
fn write_c_4_r_1(item: &LiteratureReference) -> String {
	let text = item.reference_text.as_deref().unwrap_or("").trim();
	if !text.is_empty() {
		return format!(
			"<bibliographicDesignationText>{}</bibliographicDesignationText>",
			xml_escape(text)
		);
	}
	item.reference_text_null_flavor
		.as_deref()
		.map(|value| {
			format!(
				"<bibliographicDesignationText nullFlavor=\"{}\"/>",
				xml_escape(value)
			)
		})
		.unwrap_or_else(|| "<bibliographicDesignationText/>".to_string())
}

/// e2b:C.4.r.2
fn write_c_4_r_2(
	item: &LiteratureReference,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<String> {
	write_attachment_text(
		item.document_base64.as_deref(),
		item.file_name.as_deref(),
		item.media_type.as_deref(),
		item.representation.as_deref(),
		item.compression.as_deref(),
		authority,
		"C.4.r.2",
	)
}

pub(crate) async fn apply_study_section(
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
	authority: lib_core::regulatory::RegulatoryAuthority,
) -> Result<()> {
	let study = fetch_study_information(mm, case_id).await?;
	let Some(study) = study else {
		return Ok(());
	};
	let registrations = fetch_study_registrations(mm, study.id).await?;
	let cross_reported_inds =
		if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Fda) {
			fetch_study_fda_cross_reported_inds(mm, study.id).await?
		} else {
			Vec::new()
		};

	remove_nodes(xpath, "//hl7:primaryRole/hl7:subjectOf1[hl7:researchStudy]");
	remove_nodes(xpath, "//hl7:primaryRole/hl7:subjectOf2[hl7:researchStudy]");

	let mut auth_xml = String::new();
	for reg in &registrations {
		if reg.registration_number.trim().is_empty()
			&& reg.registration_number_null_flavor.is_none()
		{
			continue;
		}
		let country_xml = write_c_5_1_r_2(reg);
		let id_xml = write_c_5_1_r_1(reg);
		auth_xml.push_str(&format!(
			"<authorization typeCode=\"AUTH\"><studyRegistration classCode=\"ACT\" moodCode=\"EVN\">{}{}</studyRegistration></authorization>",
			id_xml,
			country_xml
		));
	}

	let fda_ids =
		if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Fda) {
			format!(
				"{}{}{}",
				write_fda_c_5_5a(&study),
				write_fda_c_5_5b(&study),
				write_fda_c_5_6_r(&cross_reported_inds)
			)
		} else {
			String::new()
		};
	if !fda_ids.is_empty() {
		auth_xml.push_str(&format!("<authorization typeCode=\"AUTH\"><studyRegistration classCode=\"ACT\" moodCode=\"EVN\">{fda_ids}</studyRegistration></authorization>"));
	}

	let sponsor_id_xml = write_c_5_3(&study);
	let title_xml = write_c_5_2(&study);
	let study_type = write_c_5_4(&study);
	let regional_study_type =
		if matches!(authority, lib_core::regulatory::RegulatoryAuthority::Mfds) {
			write_c_5_4_kr_1(&study)
		} else {
			String::new()
		};
	let fragment = format!(
		"<subjectOf1 typeCode=\"SBJ\"><researchStudy classCode=\"CLNTRL\" moodCode=\"EVN\">{}<code code=\"{}\" codeSystem=\"2.16.840.1.113883.3.989.2.1.1.8\" codeSystemVersion=\"1.0\"/>{}{}{}</researchStudy></subjectOf1>",
		sponsor_id_xml,
		study_type,
		title_xml,
		auth_xml,
		regional_study_type
	);
	let xml = doc.to_string();
	if let Some(injected) = inject_study_fragment_in_primary_role(&xml, &fragment) {
		let new_doc =
			parser
				.parse_string(&injected)
				.map_err(|err| Error::InvalidXml {
					message: format!("XML parse error after study injection: {err}"),
					line: None,
					column: None,
				})?;
		*doc = new_doc;
		*xpath = Context::new(doc).map_err(|_| Error::InvalidXml {
			message: "Failed to initialize XPath context after study injection"
				.to_string(),
			line: None,
			column: None,
		})?;
		let _ = xpath.register_namespace("hl7", "urn:hl7-org:v3");
		let _ = xpath
			.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance");
	}
	Ok(())
}

/// e2b:C.5.1.r.1
fn write_c_5_1_r_1(value: &StudyRegistrationNumber) -> String {
	if value.registration_number.trim().is_empty() {
		return format!(
			"<id nullFlavor=\"{}\" root=\"2.16.840.1.113883.3.989.2.1.3.6\"/>",
			xml_escape(
				value
					.registration_number_null_flavor
					.as_deref()
					.unwrap_or("ASKU")
			)
		);
	}
	format!(
		"<id extension=\"{}\" root=\"2.16.840.1.113883.3.989.2.1.3.6\"/>",
		xml_escape(&value.registration_number)
	)
}

/// e2b:C.5.1.r.2
fn write_c_5_1_r_2(value: &StudyRegistrationNumber) -> String {
	match (value.country_code.as_deref().filter(|v| !v.trim().is_empty()), value.country_code_null_flavor.as_deref()) {
		(Some(code), _) => format!("<author typeCode=\"AUT\"><territorialAuthority classCode=\"TERR\"><governingPlace classCode=\"COUNTRY\" determinerCode=\"INSTANCE\"><code code=\"{}\" codeSystem=\"1.0.3166.1.2.2\"/></governingPlace></territorialAuthority></author>", xml_escape(code)),
		(None, Some(null_flavor)) => format!("<author typeCode=\"AUT\"><territorialAuthority classCode=\"TERR\"><governingPlace classCode=\"COUNTRY\" determinerCode=\"INSTANCE\"><code nullFlavor=\"{}\" codeSystem=\"1.0.3166.1.2.2\"/></governingPlace></territorialAuthority></author>", xml_escape(null_flavor)),
		(None, None) => String::new(),
	}
}

/// e2b:C.5.2
fn write_c_5_2(value: &StudyInformation) -> String {
	let name = value.study_name.as_deref().unwrap_or("").trim();
	if !name.is_empty() {
		return format!("<title>{}</title>", xml_escape(name));
	}
	value
		.study_name_null_flavor
		.as_deref()
		.map(|v| format!("<title nullFlavor=\"{}\"/>", xml_escape(v)))
		.unwrap_or_else(|| "<title/>".to_string())
}

/// e2b:C.5.3
fn write_c_5_3(value: &StudyInformation) -> String {
	let number = value.sponsor_study_number.as_deref().unwrap_or("").trim();
	if !number.is_empty() {
		return format!(
			"<id extension=\"{}\" root=\"2.16.840.1.113883.3.989.2.1.3.5\"/>",
			xml_escape(number)
		);
	}
	format!(
		"<id nullFlavor=\"{}\" root=\"2.16.840.1.113883.3.989.2.1.3.5\"/>",
		xml_escape(
			value
				.sponsor_study_number_null_flavor
				.as_deref()
				.unwrap_or("ASKU")
		)
	)
}

/// e2b:C.5.4
fn write_c_5_4(value: &StudyInformation) -> String {
	xml_escape(
		value
			.study_type_reaction
			.as_deref()
			.filter(|v| !v.trim().is_empty())
			.unwrap_or("1"),
	)
}

/// e2b:C.5.4.KR.1
fn write_c_5_4_kr_1(value: &StudyInformation) -> String {
	value.study_type_reaction_kr1.as_deref().filter(|v| !v.trim().is_empty()).map(|v| format!("<subjectOf2 typeCode=\"SUBJ\"><observation classCode=\"OBS\" moodCode=\"EVN\"><code code=\"{KR_C_5_4_1}\"/><value xsi:type=\"CE\" code=\"{}\"/></observation></subjectOf2>", xml_escape(v))).unwrap_or_default()
}

/// e2b:FDA.C.5.5a
fn write_fda_c_5_5a(value: &StudyInformation) -> String {
	write_fda_study_id(
		value.fda_ind_number_occurred.as_deref(),
		"2.16.840.1.113883.3.989.5.1.2.2.1.2.1",
	)
}

/// e2b:FDA.C.5.5b
fn write_fda_c_5_5b(value: &StudyInformation) -> String {
	write_fda_study_id(
		value.fda_pre_anda_number_occurred.as_deref(),
		"2.16.840.1.113883.3.989.5.1.2.2.1.2.2",
	)
}

/// e2b:FDA.C.5.6.r
fn write_fda_c_5_6_r(values: &[StudyFdaCrossReportedInd]) -> String {
	values.iter().map(|value| {
		if let Some(number) = value.ind_number.as_deref().filter(|v| !v.trim().is_empty()) {
			format!("<id extension=\"{}\" root=\"2.16.840.1.113883.3.989.5.1.2.2.1.2.3\"/>", xml_escape(number))
		} else if let Some(null_flavor) = value.ind_number_null_flavor.as_deref() {
			format!("<id nullFlavor=\"{}\" root=\"2.16.840.1.113883.3.989.5.1.2.2.1.2.3\"/>", xml_escape(null_flavor))
		} else { String::new() }
	}).collect()
}

fn write_fda_study_id(value: Option<&str>, root: &str) -> String {
	value
		.filter(|v| !v.trim().is_empty())
		.map(|v| format!("<id extension=\"{}\" root=\"{root}\"/>", xml_escape(v)))
		.unwrap_or_default()
}

async fn fetch_study_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<StudyInformation>> {
	let sql = "SELECT * FROM study_information WHERE case_id = $1 ORDER BY created_at ASC LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, StudyInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

async fn fetch_literature_references(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Vec<LiteratureReference>> {
	let sql = "SELECT * FROM literature_references WHERE case_id = $1 AND deleted = false ORDER BY sequence_number";
	mm.dbx()
		.fetch_all(sqlx::query_as::<_, LiteratureReference>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

async fn fetch_study_registrations(
	mm: &ModelManager,
	study_information_id: sqlx::types::Uuid,
) -> Result<Vec<StudyRegistrationNumber>> {
	let sql = "SELECT * FROM study_registration_numbers WHERE study_information_id = $1 AND deleted = false ORDER BY sequence_number";
	mm.dbx()
		.fetch_all(
			sqlx::query_as::<_, StudyRegistrationNumber>(sql)
				.bind(study_information_id),
		)
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

async fn fetch_study_fda_cross_reported_inds(
	mm: &ModelManager,
	study_information_id: sqlx::types::Uuid,
) -> Result<Vec<StudyFdaCrossReportedInd>> {
	mm.dbx().fetch_all(sqlx::query_as::<_, StudyFdaCrossReportedInd>("SELECT * FROM study_fda_cross_reported_inds WHERE study_information_id = $1 AND deleted = false ORDER BY sequence_number").bind(study_information_id)).await.map_err(model::Error::from).map_err(Error::from)
}

fn inject_study_fragment_in_primary_role(
	xml: &str,
	fragment: &str,
) -> Option<String> {
	let primary_start = xml.find("<primaryRole")?;
	let primary_end = xml[primary_start..].find("</primaryRole>")? + primary_start;
	let body_start = xml[primary_start..].find('>')? + primary_start + 1;
	let body = &xml[body_start..primary_end];
	let insert_at = body
		.find("<subjectOf2")
		.map(|idx| body_start + idx)
		.unwrap_or(primary_end);
	let mut out = String::with_capacity(xml.len() + fragment.len() + 8);
	out.push_str(&xml[..insert_at]);
	out.push_str(fragment);
	out.push_str(&xml[insert_at..]);
	Some(out)
}

fn inject_fragment_in_investigation_event(
	xml: &str,
	fragment: &str,
) -> Option<String> {
	let start = xml.find("<investigationEvent")?;
	let end = xml[start..].find("</investigationEvent>")? + start;
	let body_start = xml[start..].find('>')? + start + 1;
	let body = &xml[body_start..end];
	let insert_at = body
		.find("<component")
		.map(|idx| body_start + idx)
		.unwrap_or(end);
	let mut out = String::with_capacity(xml.len() + fragment.len() + 8);
	out.push_str(&xml[..insert_at]);
	out.push_str(fragment);
	out.push_str(&xml[insert_at..]);
	Some(out)
}
