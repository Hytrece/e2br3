use super::*;
use lib_core::model::message_header::MessageHeaderBmc;

pub(crate) async fn apply_section_n(
	ctx: &Ctx,
	doc: &mut Document,
	parser: &Parser,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
	xpath: &mut Context,
) -> Result<()> {
	let header = fetch_message_header(ctx, mm, case_id).await?;
	let Some(header) = header else {
		return Ok(());
	};
	let report = fetch_safety_report_identification(mm, case_id).await?;

	write_n_1_1(xpath, &header.message_type);
	write_n_1_2(xpath, header.batch_number.as_deref());
	let batch_sender = header
		.batch_sender_identifier
		.as_deref()
		.filter(|val| !val.trim().is_empty())
		.unwrap_or(&header.message_sender_identifier);
	write_n_1_3(xpath, batch_sender);
	let batch_receiver = header
		.batch_receiver_identifier
		.as_deref()
		.filter(|val| !val.trim().is_empty())
		.ok_or_else(|| Error::InvalidXml {
			message: "message_headers.batch_receiver_identifier is required for section N export".to_string(),
			line: None,
			column: None,
		})?;
	write_n_1_4(doc, parser, xpath, batch_receiver)?;
	write_n_1_5(xpath, header.batch_transmission_date, &header.message_date);
	write_n_2_r_1(xpath, &header.message_number);
	write_n_2_r_2(xpath, &header.message_sender_identifier);
	write_n_2_r_3(xpath, &header.message_receiver_identifier);
	write_n_2_r_4(xpath, &header.message_date);

	if let Some(receiver) = fetch_receiver_information(mm, case_id).await? {
		ensure_top_level_receiver_agent_nodes(
			doc,
			parser,
			xpath,
			&header.message_receiver_identifier,
		)?;
		ensure_receiver_agent_nodes(
			doc,
			parser,
			xpath,
			&header.message_receiver_identifier,
		)?;
		apply_receiver_organization(
			doc,
			parser,
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:asAgent",
			&receiver,
			report
				.as_ref()
				.and_then(|r| r.receiver_organization.as_deref()),
		);
		apply_receiver_organization(
			doc,
			parser,
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:asAgent",
			&receiver,
			report.as_ref().and_then(|r| r.receiver_organization.as_deref()),
		);
	}
	Ok(())
}

/// e2b:N.1.1
fn write_n_1_1(xpath: &mut Context, message_type: &str) {
	if !message_type.trim().is_empty() {
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:name",
			"displayName",
			message_type,
		);
	}
}

/// e2b:N.1.2
fn write_n_1_2(xpath: &mut Context, batch_number: Option<&str>) {
	if let Some(batch_number) = batch_number {
		set_attr_first(
			xpath,
			"/hl7:MCCI_IN200100UV01/hl7:id",
			"extension",
			batch_number,
		);
	}
}

/// e2b:N.1.3
fn write_n_1_3(xpath: &mut Context, batch_sender: &str) {
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id",
		"extension",
		batch_sender,
	);
}

/// e2b:N.1.4
fn write_n_1_4(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	batch_receiver: &str,
) -> Result<()> {
	ensure_batch_receiver_nodes(doc, parser, xpath, batch_receiver)?;
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:id",
		"extension",
		batch_receiver,
	);
	Ok(())
}

/// e2b:N.1.5
fn write_n_1_5(
	xpath: &mut Context,
	batch_transmission_date: Option<sqlx::types::time::OffsetDateTime>,
	message_date: &str,
) {
	let value = batch_transmission_date
		.map(fmt_datetime)
		.unwrap_or_else(|| message_date.to_string());
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:creationTime",
		"value",
		&value,
	);
}

/// e2b:N.2.r.1
fn write_n_2_r_1(xpath: &mut Context, message_number: &str) {
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:id",
		"extension",
		message_number,
	);
}

/// e2b:N.2.r.2
fn write_n_2_r_2(xpath: &mut Context, message_sender: &str) {
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:sender/hl7:device/hl7:id",
		"extension",
		message_sender,
	);
}

/// e2b:N.2.r.3
fn write_n_2_r_3(xpath: &mut Context, message_receiver: &str) {
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:id",
		"extension",
		message_receiver,
	);
}

/// e2b:N.2.r.4
fn write_n_2_r_4(xpath: &mut Context, message_date: &str) {
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:creationTime",
		"value",
		message_date,
	);
	set_attr_first(
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:controlActProcess/hl7:effectiveTime",
		"value",
		message_date,
	);
}

pub(crate) async fn fetch_message_header(
	ctx: &Ctx,
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<MessageHeader>> {
	match MessageHeaderBmc::get_by_case(ctx, mm, case_id).await {
		Ok(header) => Ok(Some(header)),
		Err(lib_core::model::Error::EntityUuidNotFound { .. }) => Ok(None),
		Err(err) => Err(Error::Model(err)),
	}
}

pub(crate) async fn fetch_primary_source(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<PrimarySource>> {
	let sql = "SELECT * FROM primary_sources WHERE case_id = $1 AND deleted = false ORDER BY sequence_number LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, PrimarySource>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

pub(crate) fn ensure_receiver_agent_nodes(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	receiver_id: &str,
) -> Result<()> {
	let base = "/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device/hl7:asAgent/hl7:representedOrganization";
	if xpath
		.findnodes(base, None)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false)
	{
		return Ok(());
	}
	let escaped = xml_escape(receiver_id);
	let fragment = format!(
		"<asAgent classCode=\"AGNT\">\
			<representedOrganization classCode=\"ORG\" determinerCode=\"INSTANCE\">\
				<id root=\"2.16.840.1.113883.3.989.2.1.3.14\" extension=\"{escaped}\"/>\
				<name/>\
			</representedOrganization>\
		</asAgent>"
	);
	append_fragment_child(
		doc,
		parser,
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:receiver/hl7:device",
		&fragment,
	)
}

fn ensure_top_level_receiver_agent_nodes(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	receiver_id: &str,
) -> Result<()> {
	let base = "/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device/hl7:asAgent/hl7:representedOrganization";
	if xpath
		.findnodes(base, None)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false)
	{
		return Ok(());
	}
	let escaped = xml_escape(receiver_id);
	let fragment = format!(
		"<asAgent classCode=\"AGNT\">\
			<representedOrganization classCode=\"ORG\" determinerCode=\"INSTANCE\">\
				<id root=\"2.16.840.1.113883.3.989.2.1.3.14\" extension=\"{escaped}\"/>\
				<name/>\
			</representedOrganization>\
		</asAgent>"
	);
	append_fragment_child(
		doc,
		parser,
		xpath,
		"/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device",
		&fragment,
	)
}

fn apply_receiver_organization(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	agent_base: &str,
	receiver: &ReceiverInformation,
	report_receiver_organization: Option<&str>,
) {
	let org_base = format!("{agent_base}/hl7:representedOrganization");
	remove_nodes(xpath, &format!("{org_base}/hl7:code"));
	remove_nodes(xpath, &format!("{org_base}/hl7:desc"));
	remove_nodes(xpath, &format!("{org_base}/hl7:addr"));
	if let Some(value) = receiver
		.organization_name
		.as_deref()
		.or(report_receiver_organization)
	{
		set_text_first(xpath, &format!("{org_base}/hl7:name"), value);
	}
	if let Some(value) = receiver.telephone.as_deref() {
		append_fragment_child_text_telecom(
			doc,
			parser,
			xpath,
			&org_base,
			&format!("tel:{value}"),
		);
	}
	if let Some(value) = receiver.fax.as_deref() {
		append_fragment_child_text_telecom(
			doc,
			parser,
			xpath,
			&org_base,
			&format!("fax:{value}"),
		);
	}
	if let Some(value) = receiver.email.as_deref() {
		append_fragment_child_text_telecom(
			doc,
			parser,
			xpath,
			&org_base,
			&format!("mailto:{value}"),
		);
	}
}

fn append_fragment_child_text_telecom(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	org_base: &str,
	value: &str,
) {
	let telecom_xpath = format!("{org_base}/hl7:telecom[@value='{value}']");
	if xpath
		.findnodes(&telecom_xpath, None)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false)
	{
		return;
	}
	let _ = append_fragment_child(
		doc,
		parser,
		xpath,
		org_base,
		&format!("<telecom value=\"{}\"/>", xml_escape(value)),
	);
}

fn ensure_batch_receiver_nodes(
	doc: &mut Document,
	parser: &Parser,
	xpath: &mut Context,
	receiver_id: &str,
) -> Result<()> {
	if xpath
		.findnodes("/hl7:MCCI_IN200100UV01/hl7:receiver/hl7:device", None)
		.map(|nodes| !nodes.is_empty())
		.unwrap_or(false)
	{
		return Ok(());
	}

	let escaped = xml_escape(receiver_id);
	let fragment = format!(
		"<receiver typeCode=\"RCV\">\
			<device classCode=\"DEV\" determinerCode=\"INSTANCE\">\
				<id root=\"2.16.840.1.113883.3.989.2.1.3.14\" extension=\"{escaped}\"/>\
			</device>\
		</receiver>"
	);
	append_fragment_child(doc, parser, xpath, "/hl7:MCCI_IN200100UV01", &fragment)
}

pub(super) async fn fetch_receiver_information(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<ReceiverInformation>> {
	let sql = "SELECT * FROM receiver_information WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(sqlx::query_as::<_, ReceiverInformation>(sql).bind(case_id))
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

async fn fetch_safety_report_identification(
	mm: &ModelManager,
	case_id: sqlx::types::Uuid,
) -> Result<Option<lib_core::model::safety_report::SafetyReportIdentification>> {
	let sql =
		"SELECT * FROM safety_report_identification WHERE case_id = $1 LIMIT 1";
	mm.dbx()
		.fetch_optional(
			sqlx::query_as::<
				_,
				lib_core::model::safety_report::SafetyReportIdentification,
			>(sql)
			.bind(case_id),
		)
		.await
		.map_err(|e| Error::Model(lib_core::model::Error::Store(format!("{e}"))))
}

#[cfg(test)]
mod tests {
	use std::collections::BTreeSet;

	#[test]
	fn section_n_writers_cover_registry_fields() {
		let registry: serde_json::Value = serde_json::from_str(include_str!(
			"../../../../../../registry/sections/n-message-header.json"
		))
		.expect("section N registry");
		let expected = registry
			.as_array()
			.expect("registry array")
			.iter()
			.filter_map(|entry| entry["e2br3_code"].as_str())
			.collect::<BTreeSet<_>>();
		let implemented = include_str!("n.rs")
			.lines()
			.filter_map(|line| line.trim().strip_prefix("/// e2b:"))
			.collect::<BTreeSet<_>>();

		assert_eq!(implemented, expected);
	}
}
