use super::support::field_contract_test;
const PAGE_ID: &str = "AE";
include!("generated/ae_fields.rs");

#[serial_test::serial]
#[tokio::test]
async fn split_null_flavor_http_contract() -> crate::common::Result<()> {
	super::support::verify_split_null_flavor_http_contract().await
}
