# Isolated Test Database Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a repository command that gives each Rust test invocation its own initialized PostgreSQL database and always deletes it afterward.

**Architecture:** A Bash runner creates one prefixed temporary database from the configured maintenance URL, applies the authoritative SQL list, exports both service database URLs, runs Cargo with unchanged arguments, and force-drops the database from an exit trap. A fake-command contract suite verifies orchestration without touching PostgreSQL; a real smoke test verifies bootstrap and cleanup.

**Tech Stack:** Bash 3.2+, PostgreSQL client commands (`createdb`, `psql`, `dropdb`), Cargo aliases, GitHub Actions, existing `scripts/db/list_init_sql.sh`.

## Global Constraints

- Isolation unit: exactly one database per `scripts/test-isolated-db.sh` invocation.
- Never read from, mutate, drop, or rebuild the shared development `app_db`.
- Temporary names must match `^e2br3_test_[a-z0-9_]+$`.
- Preserve the configured URL except for its final database path segment.
- Never print the full connection URL or credentials.
- Cleanup must run on success, test failure, initialization failure after creation, and interruption.
- Do not add a Rust database dependency or change production connection management.
- Use `scripts/db/list_init_sql.sh <repo>/db 1` as the only SQL ordering source.

---

### Task 1: Build the isolated database runner through a shell contract test

**Files:**
- Create: `scripts/tests/test_isolated_db.sh`
- Create: `scripts/test-isolated-db.sh`

**Interfaces:**
- Consumes: `SERVICE_DB_URL`, PostgreSQL client commands on `PATH`, and arbitrary Cargo test arguments.
- Produces: `scripts/test-isolated-db.sh [cargo-test-args...]`; exports `SERVICE_DB_URL`, `SERVICE_MIGRATION_DB_URL`, `SKIP_DEV_INIT=1`, and `E2BR3_TEST_DATABASE_NAME` only to the child Cargo process.

- [ ] **Step 1: Write the failing shell contract test**

Create `scripts/tests/test_isolated_db.sh` with a temporary fake `PATH`. The fake `createdb`, `psql`, `cargo`, and `dropdb` commands append their arguments and relevant environment to files under `FAKE_LOG_DIR`. Cover these exact assertions:

```bash
runner="$repo_root/scripts/test-isolated-db.sh"
SERVICE_DB_URL='postgres://user:secret@db:5432/app_db?sslmode=disable' \
FAKE_LOG_DIR="$fake_log_dir" \
PATH="$fake_bin:$PATH" \
"$runner" -p lib-core --test rbac_grant_profiles >"$stdout_file" 2>"$stderr_file"

test_url="$(sed -n 's/^SERVICE_DB_URL=//p' "$fake_log_dir/cargo.env")"
migration_url="$(sed -n 's/^SERVICE_MIGRATION_DB_URL=//p' "$fake_log_dir/cargo.env")"
database_name="$(sed -n 's/^E2BR3_TEST_DATABASE_NAME=//p' "$fake_log_dir/cargo.env")"

[[ "$database_name" =~ ^e2br3_test_[a-z0-9_]+$ ]]
[[ "$test_url" == "postgres://user:secret@db:5432/$database_name?sslmode=disable" ]]
[[ "$migration_url" == "$test_url" ]]
grep -Fx -- '-p lib-core --test rbac_grant_profiles' "$fake_log_dir/cargo.args"
grep -F -- "$database_name" "$fake_log_dir/createdb.args"
grep -F -- "$database_name" "$fake_log_dir/dropdb.args"
! grep -F -- 'user:secret' "$stdout_file" "$stderr_file"
```

Run a second scenario with `FAKE_CARGO_EXIT=23`; assert the runner exits `23` and `dropdb` still runs. Run a third scenario with `FAKE_PSQL_EXIT=31`; assert initialization exits `31` and still cleans up. Run a fourth scenario with successful Cargo and `FAKE_DROPDB_EXIT=9`; assert the runner returns nonzero.

- [ ] **Step 2: Run the contract test and verify RED**

Run: `bash scripts/tests/test_isolated_db.sh`

Expected: FAIL because `scripts/test-isolated-db.sh` does not exist.

- [ ] **Step 3: Implement the minimal safe runner**

Create `scripts/test-isolated-db.sh` with these boundaries:

```bash
#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
maintenance_url="${SERVICE_DB_URL:?SERVICE_DB_URL is required}"
database_name="e2br3_test_$(od -An -N8 -tx1 /dev/urandom | tr -d ' \n')"
created=0

if [[ ! "$database_name" =~ ^e2br3_test_[a-z0-9_]+$ ]]; then
	printf 'refusing unsafe test database name\n' >&2
	exit 2
fi

url_without_query="${maintenance_url%%\?*}"
query_suffix=""
if [[ "$maintenance_url" == *\?* ]]; then
	query_suffix="?${maintenance_url#*\?}"
fi
test_url="${url_without_query%/*}/${database_name}${query_suffix}"
```

Before creation, require `createdb`, `psql`, `dropdb`, and `cargo` with `command -v`. Register an `EXIT`, `INT`, and `TERM` cleanup path that validates the prefix again, executes `dropdb --maintenance-db="$maintenance_url" --force "$database_name"`, preserves an existing nonzero status, and turns cleanup failure into failure when the prior status was zero.

Create the database with:

```bash
createdb --maintenance-db="$maintenance_url" "$database_name"
created=1
```

Apply SQL in canonical order with:

```bash
while IFS= read -r sql_file; do
	psql "$test_url" -v ON_ERROR_STOP=1 -f "$repo_root/db/$sql_file"
done < <("$repo_root/scripts/db/list_init_sql.sh" "$repo_root/db" 1)
```

Then export the four child variables and execute `cargo test "$@"`.

- [ ] **Step 4: Run the contract test and verify GREEN**

Run: `bash scripts/tests/test_isolated_db.sh`

Expected: PASS for URL rewriting, safe naming, argument forwarding, failure-status preservation, and cleanup.

- [ ] **Step 5: Run shell syntax validation**

Run:

```bash
bash -n scripts/test-isolated-db.sh
bash -n scripts/tests/test_isolated_db.sh
```

Expected: both commands exit `0`.

- [ ] **Step 6: Commit the independently working runner**

```bash
git add scripts/test-isolated-db.sh scripts/tests/test_isolated_db.sh
git commit -m "test: run Rust suites in isolated databases"
```

---

### Task 2: Make the isolated runner the CI entry point

**Files:**
- Modify: `.github/workflows/ci.yml`
- Modify: `scripts/tests/test_isolated_db.sh`

**Interfaces:**
- Consumes: the runner from Task 1.
- Produces: CI lib-core and remaining-workspace steps that use the same executable repository runner.

- [ ] **Step 1: Extend the contract test with repository wiring assertions**

Add this exact check:

```bash
test "$(grep -c 'scripts/test-isolated-db.sh' "$repo_root/.github/workflows/ci.yml")" -eq 2
```

- [ ] **Step 2: Run the contract test and verify RED**

Run: `bash scripts/tests/test_isolated_db.sh`

Expected: FAIL because the CI calls do not exist.

- [ ] **Step 3: Route the two main CI test steps through isolation**

Replace the lib-core command with:

```yaml
run: scripts/test-isolated-db.sh -p lib-core -j1 -- --test-threads=1
```

Replace the remaining-workspace command with:

```yaml
run: scripts/test-isolated-db.sh --workspace --exclude lib-core -- --test-threads=1
```

Keep the release validation gates on the preinitialized CI `app_db`; they run after isolated suites and therefore start from unchanged seed state.

- [ ] **Step 4: Run contract and configuration validation**

Run:

```bash
bash scripts/tests/test_isolated_db.sh
bash -n scripts/test-isolated-db.sh
```

Expected: the contract and syntax checks pass.

- [ ] **Step 5: Commit repository wiring**

```bash
git add .github/workflows/ci.yml scripts/tests/test_isolated_db.sh docs/superpowers/specs/2026-07-27-isolated-test-database-design.md docs/superpowers/plans/2026-07-27-isolated-test-database.md
git commit -m "ci: isolate PostgreSQL test invocations"
```

---

### Task 3: Prove real PostgreSQL isolation and cleanup

**Files:**
- Modify only if a verification defect is discovered: `scripts/test-isolated-db.sh`, `scripts/tests/test_isolated_db.sh`

**Interfaces:**
- Consumes: `scripts/test-isolated-db.sh` from Task 2 and the local PostgreSQL configured by `SERVICE_DB_URL`.
- Produces: verification evidence that bootstrap, concurrent naming, tests, and cleanup work against real PostgreSQL.

- [ ] **Step 1: Record the pre-run set of temporary databases**

Run:

```bash
maintenance_url="${SERVICE_DB_URL:-$(sed -n 's/^SERVICE_DB_URL="\(.*\)"/\1/p' .cargo/config.toml)}"
psql "$maintenance_url" -Atc "SELECT datname FROM pg_database WHERE datname LIKE 'e2br3_test_%' ORDER BY datname"
```

Expected: record the exact output; do not delete databases not created by this task.

- [ ] **Step 2: Run a real isolated RBAC suite**

Run:

```bash
SERVICE_DB_URL="$maintenance_url" scripts/test-isolated-db.sh -p lib-core --test rbac_grant_profiles
```

Expected: 5 tests pass and the runner reports cleanup of its generated database.

- [ ] **Step 3: Verify the invocation left no new database**

Run the same `pg_database` query from Step 1.

Expected: output exactly equals the recorded pre-run set.

- [ ] **Step 4: Run two lightweight isolated invocations concurrently**

Run each command in its own background job and wait for both:

```bash
SERVICE_DB_URL="$maintenance_url" scripts/test-isolated-db.sh -p lib-core --test rbac_grant_profiles > /tmp/e2br3-isolated-a.log 2>&1 &
pid_a=$!
SERVICE_DB_URL="$maintenance_url" scripts/test-isolated-db.sh -p lib-core --test authorization_contract_snapshot > /tmp/e2br3-isolated-b.log 2>&1 &
pid_b=$!
wait "$pid_a"
wait "$pid_b"
```

Expected: both exit `0`, logs show different `e2br3_test_` names, and neither log contains a connection URL or credential.

- [ ] **Step 5: Verify concurrent cleanup and repository quality gates**

Run:

```bash
psql "$maintenance_url" -Atc "SELECT datname FROM pg_database WHERE datname LIKE 'e2br3_test_%' ORDER BY datname"
bash scripts/tests/test_isolated_db.sh
cargo fmt --all -- --check
git diff --check
```

Expected: database output still equals the Step 1 baseline; all remaining commands exit `0`.

- [ ] **Step 6: Commit any verification fix, if and only if needed**

If Steps 2–5 required a runner or contract-test correction, commit only those files:

```bash
git add scripts/test-isolated-db.sh scripts/tests/test_isolated_db.sh
git commit -m "fix: harden isolated test database cleanup"
```

If no correction was needed, create no empty commit.

---

### Task 4: Review, merge, and push

**Files:**
- Review the complete diff from the design commit through Task 3.

**Interfaces:**
- Consumes: all prior tasks and their green verification evidence.
- Produces: reviewed changes merged into `dev` and pushed to `origin/dev`.

- [ ] **Step 1: Run the final focused verification**

Run:

```bash
bash scripts/tests/test_isolated_db.sh
maintenance_url="${SERVICE_DB_URL:-$(sed -n 's/^SERVICE_DB_URL="\(.*\)"/\1/p' .cargo/config.toml)}"
SERVICE_DB_URL="$maintenance_url" scripts/test-isolated-db.sh -p lib-core --test rbac_grant_profiles
cargo fmt --all -- --check
git diff --check origin/dev...HEAD
```

Expected: every command exits `0` and the isolated database is absent afterward.

- [ ] **Step 2: Request code review**

Review the design commit through `HEAD` for destructive cleanup safety, credential leakage, URL rewriting, exit-status preservation, CI correctness, and test fidelity. Fix all Critical and Important findings using a new RED/GREEN cycle.

- [ ] **Step 3: Integrate the reviewed branch into `dev`**

Fetch `origin/dev`, merge any new `dev` changes into the implementation branch, repeat Step 1 if the tree changes, then fast-forward local `dev` to the reviewed commit.

- [ ] **Step 4: Push and verify**

Push `dev` to `origin`, fetch it again, and verify `git rev-parse dev` equals `git rev-parse origin/dev`.
