use crate::error::Error;
use crate::import_sections::shared::{
	first_attr, first_text, first_text_root, first_value_root, normalize_code,
	normalize_iso2, telecom_first, telecom_first_in_node, MessageHeaderExtract,
};
use crate::mfds::codes::{KR_C_3_1_1, KR_C_5_4_1};
use crate::Result;
use lib_core::model::receiver::ReceiverInformationForUpdate;
use libxml::parser::Parser;
use libxml::xpath::Context;

pub(crate) struct SenderImport {
	pub(crate) sender_type: String,
	pub(crate) health_professional_type_kr1: Option<String>,
	pub(crate) organization_name: String,
	pub(crate) department: Option<String>,
	pub(crate) street_address: Option<String>,
	pub(crate) city: Option<String>,
	pub(crate) state: Option<String>,
	pub(crate) postcode: Option<String>,
	pub(crate) country_code: Option<String>,
	pub(crate) person_title: Option<String>,
	pub(crate) person_given_name: Option<String>,
	pub(crate) person_middle_name: Option<String>,
	pub(crate) person_family_name: Option<String>,
	pub(crate) telephone: Option<String>,
	pub(crate) fax: Option<String>,
	pub(crate) email: Option<String>,
}

pub(crate) struct PrimarySourceImport {
	pub(crate) reporter_title: Option<String>,
	pub(crate) reporter_title_null_flavor: Option<String>,
	pub(crate) reporter_given_name: Option<String>,
	pub(crate) reporter_given_name_null_flavor: Option<String>,
	pub(crate) reporter_middle_name: Option<String>,
	pub(crate) reporter_middle_name_null_flavor: Option<String>,
	pub(crate) reporter_family_name: Option<String>,
	pub(crate) reporter_family_name_null_flavor: Option<String>,
	pub(crate) organization: Option<String>,
	pub(crate) organization_null_flavor: Option<String>,
	pub(crate) department: Option<String>,
	pub(crate) department_null_flavor: Option<String>,
	pub(crate) street: Option<String>,
	pub(crate) street_null_flavor: Option<String>,
	pub(crate) city: Option<String>,
	pub(crate) city_null_flavor: Option<String>,
	pub(crate) state: Option<String>,
	pub(crate) state_null_flavor: Option<String>,
	pub(crate) postcode: Option<String>,
	pub(crate) postcode_null_flavor: Option<String>,
	pub(crate) telephone: Option<String>,
	pub(crate) telephone_null_flavor: Option<String>,
	pub(crate) country_code: Option<String>,
	pub(crate) country_code_null_flavor: Option<String>,
	pub(crate) email: Option<String>,
	pub(crate) email_null_flavor: Option<String>,
	pub(crate) qualification: Option<String>,
	pub(crate) qualification_null_flavor: Option<String>,
	pub(crate) primary_source_regulatory: Option<String>,
}

#[derive(Debug)]
pub(crate) struct OtherCaseIdentifierImport {
	pub(crate) source_of_identifier: String,
	pub(crate) case_identifier: String,
}

#[derive(Debug)]
pub(crate) struct LinkedReportImport {
	pub(crate) linked_report_number: String,
}

#[derive(Debug)]
pub(crate) struct LiteratureImport {
	pub(crate) reference_text: String,
	pub(crate) reference_text_null_flavor: Option<String>,
	pub(crate) document_base64: Option<String>,
	pub(crate) media_type: Option<String>,
	pub(crate) representation: Option<String>,
	pub(crate) compression: Option<String>,
}

#[derive(Debug)]
pub(crate) struct DocumentHeldImport {
	pub(crate) title: Option<String>,
	pub(crate) document_base64: Option<String>,
	pub(crate) media_type: Option<String>,
	pub(crate) representation: Option<String>,
	pub(crate) compression: Option<String>,
}

#[derive(Debug)]
pub(crate) struct StudyImport {
	pub(crate) study_name: Option<String>,
	pub(crate) study_name_null_flavor: Option<String>,
	pub(crate) sponsor_study_number: Option<String>,
	pub(crate) sponsor_study_number_null_flavor: Option<String>,
	pub(crate) study_type_reaction: Option<String>,
	pub(crate) study_type_reaction_kr1: Option<String>,
	pub(crate) fda_ind_number_occurred: Option<String>,
	pub(crate) fda_pre_anda_number_occurred: Option<String>,
	pub(crate) registrations: Vec<StudyRegistrationImport>,
	pub(crate) cross_reported_inds: Vec<(Option<String>, Option<String>)>,
}

#[derive(Debug)]
pub(crate) struct StudyRegistrationImport {
	pub(crate) registration_number: String,
	pub(crate) registration_number_null_flavor: Option<String>,
	pub(crate) country_code: Option<String>,
	pub(crate) country_code_null_flavor: Option<String>,
}

fn sender_text(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
	relative: &str,
	global: &str,
) -> Option<String> {
	node.and_then(|node| first_text(xpath, node, relative))
		.or_else(|| first_text_root(xpath, global))
}

/// e2b:C.3.1
fn read_c_3_1(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Result<String> {
	let raw = node
		.and_then(|node| first_attr(xpath, node, "./hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7']", "code"))
		.or_else(|| first_value_root(xpath, "//hl7:sender/hl7:device/hl7:asAgent/hl7:representedOrganization/hl7:code/@code"))
		.or_else(|| first_value_root(xpath, "//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:code/@code"));
	normalize_code(
		raw,
		&["1", "2", "3", "4", "5", "6", "7"],
		"sender_information.sender_type",
	)
	.ok_or_else(|| Error::InvalidXml {
		message: "ICH.C.3.1.REQUIRED: sender type missing".to_string(),
		line: None,
		column: None,
	})
}

/// e2b:C.3.1.KR.1
fn read_c_3_1_kr_1(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	let raw = node
		.and_then(|node| first_attr(xpath, node, &format!("./hl7:subjectOf2/hl7:observation[hl7:code[@code='{KR_C_3_1_1}']]/hl7:value"), "code"))
		.or_else(|| first_value_root(xpath, &format!("//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:author/hl7:assignedEntity/hl7:subjectOf2/hl7:observation[hl7:code[@code='{KR_C_3_1_1}']]/hl7:value/@code")));
	normalize_code(
		raw,
		&["1", "2", "3", "4"],
		"sender_information.health_professional_type_kr1",
	)
}

/// e2b:C.3.2
fn read_c_3_2(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
	header: Option<&MessageHeaderExtract>,
) -> Result<String> {
	node.and_then(|node| first_text(xpath, node, "./hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name"))
		.or_else(|| node.and_then(|node| first_text(xpath, node, "./hl7:representedOrganization/hl7:name")))
		.or_else(|| first_text_root(xpath, "//hl7:sender/hl7:device/hl7:asAgent/hl7:representedOrganization/hl7:name"))
		.or_else(|| first_text_root(xpath, "//hl7:assignedEntity/hl7:representedOrganization/hl7:name"))
		.or_else(|| header.and_then(|header| header.message_sender.clone()))
		.ok_or_else(|| Error::InvalidXml { message: "ICH.C.3.2.REQUIRED: sender organization missing".to_string(), line: None, column: None })
}

/// e2b:C.3.3.1
fn read_c_3_3_1(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		"./hl7:representedOrganization/hl7:name",
		"//hl7:assignedEntity/hl7:representedOrganization/hl7:desc",
	)
}

/// e2b:C.3.3.2
fn read_c_3_3_2(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		"./hl7:assignedPerson/hl7:name/hl7:prefix",
		"//hl7:assignedEntity/hl7:assignedPerson/hl7:name/hl7:prefix",
	)
}

/// e2b:C.3.3.3
fn read_c_3_3_3(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		"./hl7:assignedPerson/hl7:name/hl7:given[1]",
		"//hl7:assignedEntity/hl7:assignedPerson/hl7:name/hl7:given[1]",
	)
}

/// e2b:C.3.3.4
fn read_c_3_3_4(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		"./hl7:assignedPerson/hl7:name/hl7:given[2]",
		"//hl7:assignedEntity/hl7:assignedPerson/hl7:name/hl7:given[2]",
	)
}

/// e2b:C.3.3.5
fn read_c_3_3_5(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		"./hl7:assignedPerson/hl7:name/hl7:family",
		"//hl7:assignedEntity/hl7:assignedPerson/hl7:name/hl7:family",
	)
}

fn read_sender_address(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
	element: &str,
) -> Option<String> {
	sender_text(
		xpath,
		node,
		&format!("./hl7:addr/hl7:{element}"),
		&format!("//hl7:assignedEntity/hl7:addr/hl7:{element}"),
	)
}

/// e2b:C.3.4.1
fn read_c_3_4_1(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_address(xpath, node, "streetAddressLine")
}
/// e2b:C.3.4.2
fn read_c_3_4_2(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_address(xpath, node, "city")
}
/// e2b:C.3.4.3
fn read_c_3_4_3(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_address(xpath, node, "state")
}
/// e2b:C.3.4.4
fn read_c_3_4_4(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_address(xpath, node, "postalCode")
}

/// e2b:C.3.4.5
fn read_c_3_4_5(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	let raw = node
		.and_then(|node| {
			first_attr(
				xpath,
				node,
				"./hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code",
				"code",
			)
		})
		.or_else(|| {
			node.and_then(|node| {
				first_attr(xpath, node, "./hl7:addr/hl7:country", "code")
			})
		})
		.or_else(|| {
			first_value_root(
				xpath,
				"//hl7:assignedEntity/hl7:addr/hl7:country/@code",
			)
		});
	normalize_iso2(raw, "sender_information.country_code")
}

fn read_sender_telecom(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
	prefix: &str,
) -> Option<String> {
	node.and_then(|node| telecom_first_in_node(xpath, node, prefix))
		.or_else(|| telecom_first(xpath, prefix))
}

/// e2b:C.3.4.6
fn read_c_3_4_6(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_telecom(xpath, node, "tel:")
}
/// e2b:C.3.4.7
fn read_c_3_4_7(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_telecom(xpath, node, "fax:")
}
/// e2b:C.3.4.8
fn read_c_3_4_8(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	read_sender_telecom(xpath, node, "mailto:")
}

pub(crate) fn parse_sender_information(
	xml: &[u8],
	header: Option<&MessageHeaderExtract>,
) -> Result<Option<SenderImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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
	let sender_node = xpath
		.findnodes(
			"//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:author/hl7:assignedEntity[hl7:code[@codeSystem='2.16.840.1.113883.3.989.2.1.1.7']]",
			None,
		)
		.ok()
		.and_then(|nodes| nodes.into_iter().next());

	let sender_type = read_c_3_1(&mut xpath, sender_node.as_ref())?;
	let organization_name = read_c_3_2(&mut xpath, sender_node.as_ref(), header)?;

	Ok(Some(SenderImport {
		sender_type,
		health_professional_type_kr1: read_c_3_1_kr_1(
			&mut xpath,
			sender_node.as_ref(),
		),
		organization_name,
		department: read_c_3_3_1(&mut xpath, sender_node.as_ref()),
		person_title: read_c_3_3_2(&mut xpath, sender_node.as_ref()),
		person_given_name: read_c_3_3_3(&mut xpath, sender_node.as_ref()),
		person_middle_name: read_c_3_3_4(&mut xpath, sender_node.as_ref()),
		person_family_name: read_c_3_3_5(&mut xpath, sender_node.as_ref()),
		street_address: read_c_3_4_1(&mut xpath, sender_node.as_ref()),
		city: read_c_3_4_2(&mut xpath, sender_node.as_ref()),
		state: read_c_3_4_3(&mut xpath, sender_node.as_ref()),
		postcode: read_c_3_4_4(&mut xpath, sender_node.as_ref()),
		country_code: read_c_3_4_5(&mut xpath, sender_node.as_ref()),
		telephone: read_c_3_4_6(&mut xpath, sender_node.as_ref()),
		fax: read_c_3_4_7(&mut xpath, sender_node.as_ref()),
		email: read_c_3_4_8(&mut xpath, sender_node.as_ref()),
	}))
}

fn read_text_with_null_flavor(
	xpath: &mut Context,
	node: &libxml::tree::Node,
	path: &str,
) -> (Option<String>, Option<String>) {
	(
		first_text(xpath, node, path),
		first_attr(xpath, node, path, "nullFlavor"),
	)
}

/// e2b:C.2.r.1.1
/// e2b:C.2.r.local.reporterTitleNullFlavor
fn read_c_2_r_1_1(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedPerson/hl7:name/hl7:prefix",
	)
}

/// e2b:C.2.r.1.2
/// e2b:C.2.r.local.reporterGivenNameNullFlavor
fn read_c_2_r_1_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedPerson/hl7:name/hl7:given[1]",
	)
}

/// e2b:C.2.r.1.3
/// e2b:C.2.r.local.reporterMiddleNameNullFlavor
fn read_c_2_r_1_3(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedPerson/hl7:name/hl7:given[2]",
	)
}

/// e2b:C.2.r.1.4
/// e2b:C.2.r.local.reporterFamilyNameNullFlavor
fn read_c_2_r_1_4(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedPerson/hl7:name/hl7:family",
	)
}

/// e2b:C.2.r.2.1
/// e2b:C.2.r.2.2
/// e2b:C.2.r.local.reporterOrganizationNullFlavor
/// e2b:C.2.r.local.reporterDepartmentNullFlavor
fn read_c_2_r_2_1_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (
	Option<String>,
	Option<String>,
	Option<String>,
	Option<String>,
) {
	let nested_path = ".//hl7:representedOrganization/hl7:assignedEntity/hl7:representedOrganization/hl7:name";
	let direct_path = ".//hl7:representedOrganization/hl7:name";
	let (nested, nested_null_flavor) =
		read_text_with_null_flavor(xpath, node, nested_path);
	let (direct, direct_null_flavor) =
		read_text_with_null_flavor(xpath, node, direct_path);
	let has_nested = nested.is_some() || nested_null_flavor.is_some();
	(
		nested.or_else(|| direct.clone()),
		nested_null_flavor.or_else(|| {
			(!has_nested)
				.then_some(direct_null_flavor.clone())
				.flatten()
		}),
		has_nested.then_some(direct).flatten(),
		has_nested.then_some(direct_null_flavor).flatten(),
	)
}

/// e2b:C.2.r.2.3
/// e2b:C.2.r.local.reporterStreetNullFlavor
fn read_c_2_r_2_3(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedEntity/hl7:addr/hl7:streetAddressLine",
	)
}

/// e2b:C.2.r.2.4
/// e2b:C.2.r.local.reporterCityNullFlavor
fn read_c_2_r_2_4(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedEntity/hl7:addr/hl7:city",
	)
}

/// e2b:C.2.r.2.5
/// e2b:C.2.r.local.reporterStateNullFlavor
fn read_c_2_r_2_5(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedEntity/hl7:addr/hl7:state",
	)
}

/// e2b:C.2.r.2.6
/// e2b:C.2.r.local.reporterPostcodeNullFlavor
fn read_c_2_r_2_6(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(
		xpath,
		node,
		".//hl7:assignedEntity/hl7:addr/hl7:postalCode",
	)
}

/// e2b:C.2.r.2.7
/// e2b:C.2.r.local.reporterTelephoneNullFlavor
fn read_c_2_r_2_7(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	(
		telecom_first_in_node(xpath, node, "tel:"),
		first_attr(xpath, node, ".//hl7:assignedEntity/hl7:telecom[not(starts-with(@value,'mailto:'))][1]", "nullFlavor"),
	)
}

/// e2b:FDA.C.2.r.2.8
/// e2b:C.2.r.local.reporterEmailNullFlavor
fn read_fda_c_2_r_2_8(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	(
		telecom_first_in_node(xpath, node, "mailto:"),
		first_attr(
			xpath,
			node,
			".//hl7:assignedEntity/hl7:telecom[starts-with(@value,'mailto:')][1]",
			"nullFlavor",
		),
	)
}

/// e2b:C.2.r.3
fn read_c_2_r_3(xpath: &mut Context, node: &libxml::tree::Node) -> Option<String> {
	first_attr(xpath, node, "../hl7:priorityNumber", "value")
		.filter(|value| !value.trim().is_empty())
}

/// e2b:C.2.r.4
/// e2b:C.2.r.local.qualificationNullFlavor
fn read_c_2_r_4(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>, Option<String>) {
	let path = ".//hl7:assignedPerson/hl7:asQualifiedEntity/hl7:code";
	let raw = first_attr(xpath, node, path, "code");
	(
		raw.clone(),
		normalize_code(
			raw,
			&["1", "2", "3", "4", "5"],
			"primary_sources.qualification",
		)
		.or(Some("1".to_string())),
		first_attr(xpath, node, path, "nullFlavor"),
	)
}

/// e2b:C.2.r.5
/// e2b:C.2.r.local.reporterCountryNullFlavor
fn read_c_2_r_5(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	let path = ".//hl7:assignedPerson/hl7:asLocatedEntity/hl7:location/hl7:code";
	(
		normalize_iso2(
			first_attr(xpath, node, path, "code"),
			"primary_sources.country_code",
		),
		first_attr(xpath, node, path, "nullFlavor"),
	)
}

pub(crate) fn parse_primary_sources(xml: &[u8]) -> Result<Vec<PrimarySourceImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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
	let nodes = xpath
		.findnodes(
			"//hl7:outboundRelationship[@typeCode='SPRT'][hl7:relatedInvestigation/hl7:code[@code='2']]/hl7:relatedInvestigation",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query primary sources".to_string(),
			line: None,
			column: None,
		})?;
	let mut items = Vec::new();
	for node in nodes {
		let (reporter_title, reporter_title_null_flavor) =
			read_c_2_r_1_1(&mut xpath, &node);
		let (reporter_given_name, reporter_given_name_null_flavor) =
			read_c_2_r_1_2(&mut xpath, &node);
		let (reporter_middle_name, reporter_middle_name_null_flavor) =
			read_c_2_r_1_3(&mut xpath, &node);
		let (reporter_family_name, reporter_family_name_null_flavor) =
			read_c_2_r_1_4(&mut xpath, &node);
		let (
			organization,
			organization_null_flavor,
			department,
			department_null_flavor,
		) = read_c_2_r_2_1_2(&mut xpath, &node);
		let (street, street_null_flavor) = read_c_2_r_2_3(&mut xpath, &node);
		let (city, city_null_flavor) = read_c_2_r_2_4(&mut xpath, &node);
		let (state, state_null_flavor) = read_c_2_r_2_5(&mut xpath, &node);
		let (postcode, postcode_null_flavor) = read_c_2_r_2_6(&mut xpath, &node);
		let (telephone, telephone_null_flavor) = read_c_2_r_2_7(&mut xpath, &node);
		let (email, email_null_flavor) = read_fda_c_2_r_2_8(&mut xpath, &node);
		let (country_code, country_code_null_flavor) =
			read_c_2_r_5(&mut xpath, &node);
		let (qualification_raw, qualification, qualification_null_flavor) =
			read_c_2_r_4(&mut xpath, &node);
		let primary_source_regulatory_raw = read_c_2_r_3(&mut xpath, &node);
		let primary_source_regulatory = primary_source_regulatory_raw
			.clone()
			.or(Some("2".to_string()));

		let has_importable_content = [
			reporter_title.as_ref(),
			reporter_title_null_flavor.as_ref(),
			reporter_given_name.as_ref(),
			reporter_given_name_null_flavor.as_ref(),
			reporter_middle_name.as_ref(),
			reporter_middle_name_null_flavor.as_ref(),
			reporter_family_name.as_ref(),
			reporter_family_name_null_flavor.as_ref(),
			organization.as_ref(),
			organization_null_flavor.as_ref(),
			department.as_ref(),
			department_null_flavor.as_ref(),
			street.as_ref(),
			street_null_flavor.as_ref(),
			city.as_ref(),
			city_null_flavor.as_ref(),
			state.as_ref(),
			state_null_flavor.as_ref(),
			postcode.as_ref(),
			postcode_null_flavor.as_ref(),
			telephone.as_ref(),
			telephone_null_flavor.as_ref(),
			country_code.as_ref(),
			country_code_null_flavor.as_ref(),
			email.as_ref(),
			email_null_flavor.as_ref(),
			qualification_raw.as_ref(),
			qualification_null_flavor.as_ref(),
			primary_source_regulatory_raw.as_ref(),
		]
		.into_iter()
		.any(|value| value.is_some());

		if !has_importable_content {
			continue;
		}

		items.push(PrimarySourceImport {
			reporter_title,
			reporter_title_null_flavor,
			reporter_given_name,
			reporter_given_name_null_flavor,
			reporter_middle_name,
			reporter_middle_name_null_flavor,
			reporter_family_name,
			reporter_family_name_null_flavor,
			organization,
			organization_null_flavor,
			department,
			department_null_flavor,
			street,
			street_null_flavor,
			city,
			city_null_flavor,
			state,
			state_null_flavor,
			postcode,
			postcode_null_flavor,
			telephone,
			telephone_null_flavor,
			country_code,
			country_code_null_flavor,
			email,
			email_null_flavor,
			qualification,
			qualification_null_flavor,
			primary_source_regulatory,
		});
	}

	Ok(items)
}

#[cfg(test)]
mod tests {
	use super::parse_primary_sources;

	fn primary_source_xml(body: &str) -> String {
		format!(
			r#"<?xml version="1.0" encoding="utf-8"?>
<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3">
  <PORR_IN049016UV>
    <controlActProcess>
      <subject>
        <investigationEvent>
          <outboundRelationship typeCode="SPRT">
            <priorityNumber value="1"/>
            <relatedInvestigation>
              <code code="2"/>
              <subjectOf2>
                <controlActEvent>
                  <author>
                    <assignedEntity>
                      {body}
                    </assignedEntity>
                  </author>
                </controlActEvent>
              </subjectOf2>
            </relatedInvestigation>
          </outboundRelationship>
        </investigationEvent>
      </subject>
    </controlActProcess>
  </PORR_IN049016UV>
</MCCI_IN200100UV01>"#
		)
	}

	#[test]
	fn primary_source_import_reads_direct_represented_organization_name() {
		let xml = primary_source_xml(
			r#"<representedOrganization>
  <name>Direct Reporter Org</name>
</representedOrganization>"#,
		);

		let primary_sources = parse_primary_sources(xml.as_bytes()).expect("parse");

		assert_eq!(primary_sources.len(), 1);
		assert_eq!(
			primary_sources[0].organization.as_deref(),
			Some("Direct Reporter Org")
		);
	}

	#[test]
	fn primary_source_import_keeps_rows_with_contact_data_only() {
		let xml = primary_source_xml(
			r#"<addr>
  <streetAddressLine>13 Elm St.</streetAddressLine>
  <city>Metropolis</city>
</addr>
<telecom value="mailto:reporter@example.test"/>"#,
		);

		let primary_sources = parse_primary_sources(xml.as_bytes()).expect("parse");

		assert_eq!(primary_sources.len(), 1);
		assert_eq!(primary_sources[0].street.as_deref(), Some("13 Elm St."));
		assert_eq!(
			primary_sources[0].email.as_deref(),
			Some("reporter@example.test")
		);
	}

	#[test]
	fn primary_source_import_isolates_element_null_flavors() {
		let xml = primary_source_xml(
			r#"<assignedPerson><name>
  <prefix/>
  <given nullFlavor="ASKU"/>
  <family/>
</name></assignedPerson>
<addr><city nullFlavor="NASK"/><state/></addr>"#,
		);

		let primary_sources = parse_primary_sources(xml.as_bytes()).expect("parse");

		assert_eq!(primary_sources.len(), 1);
		assert_eq!(
			primary_sources[0]
				.reporter_given_name_null_flavor
				.as_deref(),
			Some("ASKU")
		);
		assert_eq!(primary_sources[0].city_null_flavor.as_deref(), Some("NASK"));
		assert!(primary_sources[0].reporter_title_null_flavor.is_none());
		assert!(primary_sources[0].state_null_flavor.is_none());
	}

	#[test]
	fn primary_source_import_keeps_contact_null_flavors() {
		let xml = primary_source_xml(
			r#"<assignedPerson>
  <asQualifiedEntity><code nullFlavor="ASKU"/></asQualifiedEntity>
  <asLocatedEntity><location><code nullFlavor="NASK"/></location></asLocatedEntity>
</assignedPerson>
<telecom nullFlavor="NI"/>"#,
		);
		let sources = parse_primary_sources(xml.as_bytes()).expect("parse");
		assert_eq!(sources[0].telephone_null_flavor.as_deref(), Some("NI"));
		assert_eq!(sources[0].country_code_null_flavor.as_deref(), Some("NASK"));
		assert_eq!(
			sources[0].qualification_null_flavor.as_deref(),
			Some("ASKU")
		);
	}
}

pub(crate) fn parse_other_case_identifiers(
	xml: &[u8],
) -> Result<Vec<OtherCaseIdentifierImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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

	let nodes = xpath
		.findnodes(
			"//hl7:investigationEvent/hl7:subjectOf1/hl7:controlActEvent/hl7:id",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query other case identifiers".to_string(),
			line: None,
			column: None,
		})?;

	let mut items = Vec::new();
	for node in nodes {
		let source = read_c_1_9_1_r_1(&node);
		let extension = read_c_1_9_1_r_2(&node);
		let Some(source) = source else {
			continue;
		};
		let Some(case_identifier) = extension else {
			continue;
		};
		if source.trim().is_empty() || case_identifier.trim().is_empty() {
			continue;
		}
		items.push(OtherCaseIdentifierImport {
			source_of_identifier: source,
			case_identifier,
		});
	}
	Ok(items)
}

/// e2b:C.1.9.1.r.1
fn read_c_1_9_1_r_1(node: &libxml::tree::Node) -> Option<String> {
	node.get_attribute("assigningAuthorityName")
}

/// e2b:C.1.9.1.r.2
fn read_c_1_9_1_r_2(node: &libxml::tree::Node) -> Option<String> {
	node.get_attribute("extension")
}

pub(crate) fn parse_linked_reports(xml: &[u8]) -> Result<Vec<LinkedReportImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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

	let nodes = xpath
		.findnodes(
			"//hl7:investigationEvent/hl7:outboundRelationship[@typeCode='SPRT']/hl7:relatedInvestigation/hl7:subjectOf2/hl7:controlActEvent/hl7:id",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query linked reports".to_string(),
			line: None,
			column: None,
		})?;

	let mut items = Vec::new();
	for node in nodes {
		let extension = read_c_1_10_r(&node);
		let Some(linked_report_number) = extension else {
			continue;
		};
		if linked_report_number.trim().is_empty() {
			continue;
		}
		items.push(LinkedReportImport {
			linked_report_number,
		});
	}
	Ok(items)
}

/// e2b:C.1.10.r
fn read_c_1_10_r(node: &libxml::tree::Node) -> Option<String> {
	node.get_attribute("extension")
}

pub(crate) fn parse_documents_held_by_sender(
	xml: &[u8],
) -> Result<Vec<DocumentHeldImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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

	let nodes = xpath
		.findnodes(
			"//hl7:reference/hl7:document[hl7:code[@code='1' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.27']]",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query documents held by sender".to_string(),
			line: None,
			column: None,
		})?;

	let mut items = Vec::new();
	for node in nodes {
		let title = read_c_1_6_1_r_1(&mut xpath, &node);
		let (document_base64, media_type, representation, compression) =
			read_c_1_6_1_r_2(&mut xpath, &node);
		items.push(DocumentHeldImport {
			title,
			document_base64,
			media_type,
			representation,
			compression,
		});
	}
	Ok(items)
}

/// e2b:C.1.6.1.r.1
fn read_c_1_6_1_r_1(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> Option<String> {
	first_text(xpath, node, "hl7:title")
}

/// e2b:C.1.6.1.r.2
/// e2b:C.1.6.1.r.2.local.mediaType
/// e2b:C.1.6.1.r.2.local.representation
/// e2b:C.1.6.1.r.2.local.compression
fn read_c_1_6_1_r_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (
	Option<String>,
	Option<String>,
	Option<String>,
	Option<String>,
) {
	(
		first_text(xpath, node, "hl7:text"),
		first_attr(xpath, node, "hl7:text", "mediaType"),
		first_attr(xpath, node, "hl7:text", "representation"),
		first_attr(xpath, node, "hl7:text", "compression"),
	)
}

pub(crate) fn parse_literature_references(
	xml: &[u8],
) -> Result<Vec<LiteratureImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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

	let nodes = xpath
		.findnodes(
			"//hl7:reference/hl7:document[hl7:code[@code='2' and @codeSystem='2.16.840.1.113883.3.989.2.1.1.27']]",
			None,
		)
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query literature references".to_string(),
			line: None,
			column: None,
		})?;

	let mut items = Vec::new();
	for (idx, node) in nodes.into_iter().enumerate() {
		let (reference_text, reference_text_null_flavor) =
			read_c_4_r_1(&mut xpath, &node, idx)?;
		let (document_base64, media_type, representation, compression) =
			read_c_4_r_2(&mut xpath, &node);
		items.push(LiteratureImport {
			reference_text,
			reference_text_null_flavor,
			document_base64,
			media_type,
			representation,
			compression,
		});
	}
	Ok(items)
}

/// e2b:C.4.r.1
/// e2b:C.4.r.local.referenceTextNullFlavor
fn read_c_4_r_1(
	xpath: &mut Context,
	node: &libxml::tree::Node,
	index: usize,
) -> Result<(String, Option<String>)> {
	let null_flavor = first_attr(
		xpath,
		node,
		"hl7:bibliographicDesignationText",
		"nullFlavor",
	)
	.or_else(|| first_attr(xpath, node, "hl7:title", "nullFlavor"));
	let text = first_text(xpath, node, "hl7:bibliographicDesignationText")
		.or_else(|| first_text(xpath, node, "hl7:title"))
		.or_else(|| null_flavor.as_ref().map(|_| String::new()))
		.ok_or_else(|| Error::InvalidXml {
			message: format!("ICH.C.4.r.REQUIRED: literature reference text missing for sequence {}", index + 1),
			line: None,
			column: None,
		})?;
	Ok((text, null_flavor))
}

/// e2b:C.4.r.2
/// e2b:C.4.r.2.local.mediaType
/// e2b:C.4.r.2.local.representation
/// e2b:C.4.r.2.local.compression
fn read_c_4_r_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (
	Option<String>,
	Option<String>,
	Option<String>,
	Option<String>,
) {
	read_c_1_6_1_r_2(xpath, node)
}

pub(crate) fn parse_study_information(xml: &[u8]) -> Result<Option<StudyImport>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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

	let nodes = xpath.findnodes("//hl7:researchStudy", None).map_err(|_| {
		Error::InvalidXml {
			message: "Failed to query study information".to_string(),
			line: None,
			column: None,
		}
	})?;
	let Some(node) = nodes.get(0) else {
		return Ok(None);
	};

	let (study_name, study_name_null_flavor) = read_c_5_2(&mut xpath, node);
	let (sponsor_study_number, sponsor_study_number_null_flavor) =
		read_c_5_3(&mut xpath, node);
	let study_type_reaction = read_c_5_4(&mut xpath, node);
	let study_type_reaction_kr1 = read_c_5_4_kr_1(&mut xpath, node);
	let fda_ind_number_occurred = read_fda_c_5_5a(&mut xpath, node);
	let fda_pre_anda_number_occurred = read_fda_c_5_5b(&mut xpath, node);
	let cross_reported_inds = read_fda_c_5_6_r(&mut xpath, node);

	let reg_nodes = xpath
		.findnodes(".//hl7:studyRegistration", Some(node))
		.map_err(|_| Error::InvalidXml {
			message: "Failed to query study registrations".to_string(),
			line: None,
			column: None,
		})?;
	let mut registrations = Vec::new();
	for reg in reg_nodes {
		let (registration_number, registration_number_null_flavor) =
			read_c_5_1_r_1(&mut xpath, &reg);
		let Some(registration_number) = registration_number.or_else(|| {
			registration_number_null_flavor
				.as_ref()
				.map(|_| String::new())
		}) else {
			continue;
		};
		let (country_code, country_code_null_flavor) =
			read_c_5_1_r_2(&mut xpath, &reg);
		registrations.push(StudyRegistrationImport {
			registration_number,
			registration_number_null_flavor,
			country_code: normalize_iso2(
				country_code,
				"study_registration.country_code",
			),
			country_code_null_flavor,
		});
	}

	Ok(Some(StudyImport {
		study_name,
		study_name_null_flavor,
		sponsor_study_number,
		sponsor_study_number_null_flavor,
		study_type_reaction,
		study_type_reaction_kr1,
		fda_ind_number_occurred,
		fda_pre_anda_number_occurred,
		registrations,
		cross_reported_inds,
	}))
}

/// e2b:C.5.1.r.1
fn read_c_5_1_r_1(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	(
		first_attr(xpath, node, "hl7:id", "extension"),
		first_attr(xpath, node, "hl7:id", "nullFlavor"),
	)
}

/// e2b:C.5.1.r.2
fn read_c_5_1_r_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	let path = "hl7:author/hl7:territorialAuthority/hl7:governingPlace/hl7:code";
	(
		first_attr(xpath, node, path, "code"),
		first_attr(xpath, node, path, "nullFlavor"),
	)
}

/// e2b:C.5.2
fn read_c_5_2(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	read_text_with_null_flavor(xpath, node, "hl7:title")
}

/// e2b:C.5.3
fn read_c_5_3(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> (Option<String>, Option<String>) {
	(
		first_attr(xpath, node, "hl7:id", "extension"),
		first_attr(xpath, node, "hl7:id", "nullFlavor"),
	)
}

/// e2b:C.5.4
fn read_c_5_4(xpath: &mut Context, node: &libxml::tree::Node) -> Option<String> {
	first_attr(xpath, node, "hl7:code", "code")
}

/// e2b:C.5.4.KR.1
fn read_c_5_4_kr_1(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> Option<String> {
	first_attr(xpath, node, &format!("hl7:subjectOf2/hl7:observation[hl7:code/@code='{KR_C_5_4_1}']/hl7:value"), "code")
}

/// e2b:FDA.C.5.5a
fn read_fda_c_5_5a(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> Option<String> {
	first_attr(
		xpath,
		node,
		"hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.1']",
		"extension",
	)
}

/// e2b:FDA.C.5.5b
fn read_fda_c_5_5b(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> Option<String> {
	first_attr(
		xpath,
		node,
		"hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.2']",
		"extension",
	)
}

/// e2b:FDA.C.5.6.r
/// e2b:FDA.C.5.6.r.local.indNumberNullFlavor
fn read_fda_c_5_6_r(
	xpath: &mut Context,
	node: &libxml::tree::Node,
) -> Vec<(Option<String>, Option<String>)> {
	xpath
		.findnodes(
			"hl7:id[@root='2.16.840.1.113883.3.989.5.1.2.2.1.2.3']",
			Some(node),
		)
		.unwrap_or_default()
		.into_iter()
		.map(|id| (id.get_property("extension"), id.get_property("nullFlavor")))
		.collect()
}

pub(crate) fn parse_receiver_information(
	xml: &[u8],
) -> Result<Option<ReceiverInformationForUpdate>> {
	let xml_str = std::str::from_utf8(xml).map_err(|err| Error::InvalidXml {
		message: format!("XML not valid UTF-8: {err}"),
		line: None,
		column: None,
	})?;
	let parser = Parser::default();
	let doc = parser
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
	let receiver_node = xpath
		.findnodes("//hl7:receiver/hl7:device", None)
		.ok()
		.and_then(|nodes| nodes.into_iter().next());

	let organization_name = read_receiver_organization_name(&mut xpath);

	if organization_name.is_none() {
		return Ok(None);
	}

	Ok(Some(ReceiverInformationForUpdate {
		receiver_type: normalize_code(
			read_receiver_type(&mut xpath),
			&["1", "2", "3", "4", "5", "6"],
			"receiver_information.receiver_type",
		),
		organization_name,
		department: read_receiver_department(&mut xpath),
		street_address: read_receiver_street_address(&mut xpath),
		city: read_receiver_city(&mut xpath),
		state_province: read_receiver_state_province(&mut xpath),
		postcode: read_receiver_postcode(&mut xpath),
		country_code: normalize_iso2(
			read_receiver_country(&mut xpath),
			"receiver_information.country_code",
		),
		telephone: read_receiver_telephone(&mut xpath, receiver_node.as_ref()),
		fax: read_receiver_fax(&mut xpath, receiver_node.as_ref()),
		email: read_receiver_email(&mut xpath, receiver_node.as_ref()),
	}))
}

/// e2b:local.receiver.1
fn read_receiver_type(xpath: &mut Context) -> Option<String> {
	first_value_root(xpath, "//hl7:receiver/hl7:device/hl7:asAgent/hl7:representedOrganization/hl7:code/@code")
}

/// e2b:local.receiver.2
fn read_receiver_organization_name(xpath: &mut Context) -> Option<String> {
	first_value_root(xpath, "//hl7:receiver/hl7:device/hl7:id/@extension").or_else(|| first_text_root(xpath, "//hl7:receiver/hl7:device/hl7:asAgent/hl7:representedOrganization/hl7:name"))
}

/// e2b:local.receiver.3
fn read_receiver_department(xpath: &mut Context) -> Option<String> {
	first_text_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:representedOrganization/hl7:desc",
	)
}

/// e2b:local.receiver.4
fn read_receiver_street_address(xpath: &mut Context) -> Option<String> {
	first_text_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:addr/hl7:streetAddressLine",
	)
}

/// e2b:local.receiver.5
fn read_receiver_city(xpath: &mut Context) -> Option<String> {
	first_text_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:addr/hl7:city",
	)
}

/// e2b:local.receiver.6
fn read_receiver_state_province(xpath: &mut Context) -> Option<String> {
	first_text_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:addr/hl7:state",
	)
}

/// e2b:local.receiver.7
fn read_receiver_postcode(xpath: &mut Context) -> Option<String> {
	first_text_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:addr/hl7:postalCode",
	)
}

/// e2b:local.receiver.country
fn read_receiver_country(xpath: &mut Context) -> Option<String> {
	first_value_root(
		xpath,
		"//hl7:receiver/hl7:device/hl7:asAgent/hl7:addr/hl7:country/@code",
	)
}

/// e2b:local.receiver.8
fn read_receiver_telephone(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	node.and_then(|node| telecom_first_in_node(xpath, node, "tel:"))
}

/// e2b:local.receiver.9
fn read_receiver_fax(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	node.and_then(|node| telecom_first_in_node(xpath, node, "fax:"))
}

/// e2b:local.receiver.10
fn read_receiver_email(
	xpath: &mut Context,
	node: Option<&libxml::tree::Node>,
) -> Option<String> {
	node.and_then(|node| telecom_first_in_node(xpath, node, "mailto:"))
}
