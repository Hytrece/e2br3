#!/usr/bin/env sh
set -eu

PROJECT_DIR="${PROJECT_DIR:-$(pwd)}"
DATABASE_URL="${DATABASE_URL:-${SERVICE_MIGRATION_DB_URL:-}}"
BASELINE="${BASELINE:-0}"
BASELINE_IF_EMPTY="${BASELINE_IF_EMPTY:-0}"
MIGRATIONS_DIR="${PROJECT_DIR}/db/migrations"

if [ -z "${DATABASE_URL}" ]; then
	echo "DATABASE_URL or SERVICE_MIGRATION_DB_URL is required" >&2
	exit 1
fi

command -v psql >/dev/null 2>&1 || { echo "psql is required" >&2; exit 1; }
command -v sha256sum >/dev/null 2>&1 || { echo "sha256sum is required" >&2; exit 1; }
test -d "${MIGRATIONS_DIR}" || { echo "Missing ${MIGRATIONS_DIR}" >&2; exit 1; }

psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS public.schema_migrations (
    filename text PRIMARY KEY,
    checksum text NOT NULL,
    applied_at timestamptz NOT NULL DEFAULT now()
);
SQL

if [ "${BASELINE_IF_EMPTY}" = "1" ] && [ "$(psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -At -c 'SELECT count(*) FROM public.schema_migrations')" = "0" ]; then
	BASELINE=1
fi

if [ "${BASELINE}" = "1" ]; then
	echo "Baselining existing migrations (no SQL will be executed)"
fi

LC_ALL=C find "${MIGRATIONS_DIR}" -maxdepth 1 -type f -name '*.sql' | sort |
while IFS= read -r path; do
	filename=$(basename "${path}")
	checksum=$(sha256sum "${path}" | awk '{print $1}')
	applied_checksum=$(psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -At \
		-v filename="${filename}" <<'SQL'
SELECT checksum FROM public.schema_migrations WHERE filename = :'filename';
SQL
)

	if [ -n "${applied_checksum}" ]; then
		if [ "${applied_checksum}" != "${checksum}" ]; then
			echo "Applied migration was modified: ${filename}" >&2
			exit 1
		fi
		continue
	fi

	if [ "${BASELINE}" = "1" ]; then
		psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 \
			-v filename="${filename}" -v checksum="${checksum}" <<'SQL'
INSERT INTO public.schema_migrations (filename, checksum) VALUES (:'filename', :'checksum');
SQL
	else
		echo "==> migrations/${filename}"
		psql "${DATABASE_URL}" -v ON_ERROR_STOP=1 -1 \
			-v filename="${filename}" -v checksum="${checksum}" <<SQL
\i ${path}
INSERT INTO public.schema_migrations (filename, checksum) VALUES (:'filename', :'checksum');
SQL
	fi
done

echo "RDS migrations complete."
