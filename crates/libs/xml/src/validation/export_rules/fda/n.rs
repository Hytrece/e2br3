use crate::XmlValidationError;
use libxml::xpath::Context;

pub(super) fn run(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	r0006(xpath, errors);
	r0007(xpath, errors);
	r0100(xpath, errors);
	let now = time::OffsetDateTime::now_utc();
	n_1_5(xpath, errors, now);
	n_2_r_4(xpath, errors, now);
}

fn r0006(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0006", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA']/hl7:PORR_IN049016UV[not(hl7:receiver/hl7:device/hl7:id/@extension='CDER')]", true);
}

fn r0007(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0007", "/hl7:MCCI_IN200100UV01[hl7:receiver/hl7:device/hl7:id/@extension='ZZFDA_PREMKT']/hl7:PORR_IN049016UV[not(hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CBER_IND' or hl7:receiver/hl7:device/hl7:id/@extension='CDER_IND_EXEMPT_BA_BE')]", true);
}

fn r0100(xpath: &mut Context, errors: &mut Vec<XmlValidationError>) {
	reject(xpath, errors, "R0100", "/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV[hl7:sender/hl7:device/hl7:id/@extension != /hl7:MCCI_IN200100UV01/hl7:sender/hl7:device/hl7:id/@extension]", true);
}

fn n_1_5(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	now: time::OffsetDateTime,
) {
	super::reject_future_datetime(
		xpath,
		errors,
		"N.1.5",
		"/hl7:MCCI_IN200100UV01/hl7:creationTime/@value",
		now,
	);
}

fn n_2_r_4(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	now: time::OffsetDateTime,
) {
	super::reject_future_datetime(
		xpath,
		errors,
		"N.2.r.4",
		"/hl7:MCCI_IN200100UV01/hl7:PORR_IN049016UV/hl7:creationTime/@value",
		now,
	);
}

fn reject(
	xpath: &mut Context,
	errors: &mut Vec<XmlValidationError>,
	code: &str,
	expression: &str,
	blocking: bool,
) {
	if super::super::matches(xpath, expression) {
		errors.push(XmlValidationError {
			message: format!("[{code}] FDA business rule failed."),
			code: Some(code.to_string()),
			section: Some("xml".to_string()),
			field_path: None,
			blocking: Some(blocking),
			line: None,
			column: None,
		});
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use libxml::parser::Parser;
	#[test]
	fn n_rule_uses_exact_code() {
		let doc = Parser::default().parse_string(br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><receiver><device><id extension="ZZFDA"/></device></receiver><PORR_IN049016UV><receiver><device><id extension="wrong"/></device></receiver></PORR_IN049016UV></MCCI_IN200100UV01>"#).unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		run(&mut xpath, &mut errors);
		assert!(errors
			.iter()
			.any(|error| error.code.as_deref() == Some("R0006")
				&& error.blocking == Some(true)));
	}

	#[test]
	fn same_day_future_time_is_rejected() {
		let doc = Parser::default()
			.parse_string(
				br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><creationTime value="20260807120001"/></MCCI_IN200100UV01>"#,
			)
			.unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		let now = super::super::parse_hl7_datetime("20260807120000").unwrap();
		n_1_5(&mut xpath, &mut errors, now);
		assert_eq!(errors[0].code.as_deref(), Some("N.1.5"));
	}

	#[test]
	fn offset_timestamp_is_compared_as_instant() {
		let doc = Parser::default()
			.parse_string(
				br#"<MCCI_IN200100UV01 xmlns="urn:hl7-org:v3"><creationTime value="20260810174219+0900"/><PORR_IN049016UV><creationTime value="20260810172527+0900"/></PORR_IN049016UV></MCCI_IN200100UV01>"#,
			)
			.unwrap();
		let mut xpath = Context::new(&doc).unwrap();
		xpath.register_namespace("hl7", "urn:hl7-org:v3").unwrap();
		let mut errors = vec![];
		let now = super::super::parse_hl7_datetime("20260810113146").unwrap();
		n_1_5(&mut xpath, &mut errors, now);
		n_2_r_4(&mut xpath, &mut errors, now);
		assert!(errors.is_empty());
	}
}
