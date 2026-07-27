#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd -P)"
runner="$repo_root/scripts/test-isolated-db.sh"
test_root="$(mktemp -d "${TMPDIR:-/tmp}/e2br3-isolated-contract.XXXXXX")"
fake_bin="$test_root/bin"
mkdir -p "$fake_bin"

cleanup_test_root() {
	rm -rf "$test_root"
}
trap cleanup_test_root EXIT

write_fake_commands() {
	cat >"$fake_bin/createdb" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_LOG_DIR/createdb.args"
EOF
	cat >"$fake_bin/psql" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_LOG_DIR/psql.args"
exit "${FAKE_PSQL_EXIT:-0}"
EOF
	cat >"$fake_bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >"$FAKE_LOG_DIR/cargo.args"
{
	printf 'SERVICE_DB_URL=%s\n' "$SERVICE_DB_URL"
	printf 'SERVICE_MIGRATION_DB_URL=%s\n' "$SERVICE_MIGRATION_DB_URL"
	printf 'SKIP_DEV_INIT=%s\n' "$SKIP_DEV_INIT"
	printf 'E2BR3_TEST_DATABASE_NAME=%s\n' "$E2BR3_TEST_DATABASE_NAME"
} >"$FAKE_LOG_DIR/cargo.env"
exit "${FAKE_CARGO_EXIT:-0}"
EOF
	cat >"$fake_bin/dropdb" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '%s\n' "$*" >>"$FAKE_LOG_DIR/dropdb.args"
exit "${FAKE_DROPDB_EXIT:-0}"
EOF
	chmod +x "$fake_bin/createdb" "$fake_bin/psql" "$fake_bin/cargo" "$fake_bin/dropdb"
}

new_case_dir() {
	local name="$1"
	local case_dir="$test_root/$name"
	mkdir -p "$case_dir"
	printf '%s\n' "$case_dir"
}

assert_success_contract() {
	local case_dir
	case_dir="$(new_case_dir success)"
	SERVICE_DB_URL='postgres://user:secret@db:5432/app_db?sslmode=disable' \
		FAKE_LOG_DIR="$case_dir" \
		PATH="$fake_bin:$PATH" \
		"$runner" -p lib-core --test rbac_grant_profiles \
		>"$case_dir/stdout" 2>"$case_dir/stderr"

	local test_url migration_url database_name
	test_url="$(sed -n 's/^SERVICE_DB_URL=//p' "$case_dir/cargo.env")"
	migration_url="$(sed -n 's/^SERVICE_MIGRATION_DB_URL=//p' "$case_dir/cargo.env")"
	database_name="$(sed -n 's/^E2BR3_TEST_DATABASE_NAME=//p' "$case_dir/cargo.env")"

	[[ "$database_name" =~ ^e2br3_test_[a-z0-9_]+$ ]]
	[[ "$test_url" == "postgres://user:secret@db:5432/$database_name?sslmode=disable" ]]
	[[ "$migration_url" == "$test_url" ]]
	grep -Fx -- 'test -p lib-core --test rbac_grant_profiles' "$case_dir/cargo.args" >/dev/null
	grep -Fx -- 'SKIP_DEV_INIT=1' "$case_dir/cargo.env" >/dev/null
	grep -F -- "$database_name" "$case_dir/createdb.args" >/dev/null
	grep -F -- "$database_name" "$case_dir/dropdb.args" >/dev/null
	if grep -F -- 'user:secret' "$case_dir/stdout" "$case_dir/stderr" >/dev/null; then
		printf 'runner leaked database credentials\n' >&2
		return 1
	fi
}

assert_test_failure_preserves_status_and_cleans_up() {
	local case_dir
	case_dir="$(new_case_dir cargo-failure)"
	set +e
	SERVICE_DB_URL='postgres://user:secret@db:5432/app_db?sslmode=disable' \
		FAKE_LOG_DIR="$case_dir" \
		FAKE_CARGO_EXIT=23 \
		PATH="$fake_bin:$PATH" \
		"$runner" -p lib-core >"$case_dir/stdout" 2>"$case_dir/stderr"
	local status=$?
	set -e
	[[ "$status" -eq 23 ]]
	test -s "$case_dir/dropdb.args"
}

assert_cleanup_failure_fails_successful_run() {
	local case_dir
	case_dir="$(new_case_dir cleanup-failure)"
	set +e
	SERVICE_DB_URL='postgres://user:secret@db:5432/app_db?sslmode=disable' \
		FAKE_LOG_DIR="$case_dir" \
		FAKE_DROPDB_EXIT=9 \
		PATH="$fake_bin:$PATH" \
		"$runner" -p lib-core >"$case_dir/stdout" 2>"$case_dir/stderr"
	local status=$?
	set -e
	[[ "$status" -ne 0 ]]
	test -s "$case_dir/dropdb.args"
}

assert_initialization_failure_preserves_status_and_cleans_up() {
	local case_dir
	case_dir="$(new_case_dir psql-failure)"
	set +e
	SERVICE_DB_URL='postgres://user:secret@db:5432/app_db?sslmode=disable' \
		FAKE_LOG_DIR="$case_dir" \
		FAKE_PSQL_EXIT=31 \
		PATH="$fake_bin:$PATH" \
		"$runner" -p lib-core >"$case_dir/stdout" 2>"$case_dir/stderr"
	local status=$?
	set -e
	[[ "$status" -eq 31 ]]
	test -s "$case_dir/dropdb.args"
}

assert_repository_wiring() {
	[[ "$(grep -c 'scripts/test-isolated-db.sh' \
		"$repo_root/.github/workflows/ci.yml")" -eq 2 ]]
}

write_fake_commands
assert_success_contract
assert_test_failure_preserves_status_and_cleans_up
assert_cleanup_failure_fails_successful_run
assert_initialization_failure_preserves_status_and_cleans_up
assert_repository_wiring
printf 'isolated database runner contract: PASS\n'
