mod common;

#[tokio::test]
async fn database_test_without_isolation_marker_is_rejected() {
	std::env::remove_var("E2BR3_TEST_DATABASE_NAME");
	std::env::set_var("SERVICE_DB_URL", "postgres://sentinel/must_not_change");

	let error = common::init_test_env().await.unwrap_err();

	assert_eq!(
		error.to_string(),
		"database-backed web-server tests require scripts/test-isolated-db.sh"
	);
	assert_eq!(
		std::env::var("SERVICE_DB_URL").unwrap(),
		"postgres://sentinel/must_not_change"
	);
}
