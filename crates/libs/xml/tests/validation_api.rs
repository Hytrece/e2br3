use xml::validation::{
	default_xsd_path, should_skip_xml_validation, validate_e2b_xml,
	validate_e2b_xml_basic, validate_e2b_xml_business, XmlValidatorConfig,
};

#[test]
fn generic_validation_api_is_owned_by_xml() {
	let config = XmlValidatorConfig {
		xsd_path: None,
		..Default::default()
	};
	let report = validate_e2b_xml_basic(
		b"<MCCI_IN200100UV01></MCCI_IN200100UV01>",
		Some(config),
	)
	.expect("basic validation");
	assert!(report.ok);
	let _ = default_xsd_path();
	let _ = should_skip_xml_validation();
	let _ = validate_e2b_xml;
	let _ = validate_e2b_xml_business;
}
