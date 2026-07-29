use super::support::field_contract_test;
const PAGE_ID: &str = "DM";
include!("generated/dm_fields.rs");

#[serial_test::serial]
#[tokio::test]
async fn d_10_1_value_nullflavor_value() -> crate::common::Result<()> {
	super::support::verify_dm_parent_transition(
		super::support::DmParentField::Identification,
	)
	.await
}

#[serial_test::serial]
#[tokio::test]
async fn d_10_6_value_nullflavor_value() -> crate::common::Result<()> {
	super::support::verify_dm_parent_transition(super::support::DmParentField::Sex)
		.await
}
