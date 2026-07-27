#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
maintenance_url="${SERVICE_DB_URL:?SERVICE_DB_URL is required}"

for required_command in createdb psql dropdb cargo; do
	if ! command -v "$required_command" >/dev/null 2>&1; then
		printf 'required command not found: %s\n' "$required_command" >&2
		exit 127
	fi
done

database_name="e2br3_test_$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')"
if [[ ! "$database_name" =~ ^e2br3_test_[a-z0-9_]+$ ]]; then
	printf 'refusing unsafe test database name\n' >&2
	exit 2
fi

url_without_query="${maintenance_url%%\?*}"
if [[ "$url_without_query" != *://*/* ]]; then
	printf 'SERVICE_DB_URL must include a database path\n' >&2
	exit 2
fi
query_suffix=""
if [[ "$maintenance_url" == *\?* ]]; then
	query_suffix="?${maintenance_url#*\?}"
fi
test_url="${url_without_query%/*}/${database_name}${query_suffix}"
created=0

cleanup() {
	local status=$?
	trap - EXIT INT TERM
	if [[ "$created" -eq 1 ]]; then
		printf 'Dropping isolated test database %s\n' "$database_name"
		if ! dropdb --maintenance-db="$maintenance_url" --force "$database_name"; then
			printf 'failed to drop isolated test database %s\n' "$database_name" >&2
			if [[ "$status" -eq 0 ]]; then
				status=1
			fi
		fi
	fi
	exit "$status"
}

trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

printf 'Creating isolated test database %s\n' "$database_name"
createdb --maintenance-db="$maintenance_url" "$database_name"
created=1

sql_list="$("$repo_root/scripts/db/list_init_sql.sh" "$repo_root/db" 1)"
while IFS= read -r sql_file; do
	if [[ -n "$sql_file" ]]; then
		psql "$test_url" -v ON_ERROR_STOP=1 -f "$repo_root/db/$sql_file"
	fi
done <<<"$sql_list"

export SERVICE_DB_URL="$test_url"
export SERVICE_MIGRATION_DB_URL="$test_url"
export SKIP_DEV_INIT=1
export E2BR3_TEST_DATABASE_NAME="$database_name"

cargo test "$@"
