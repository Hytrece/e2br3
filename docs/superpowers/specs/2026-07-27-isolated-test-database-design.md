# Isolated Test Database Design

## Goal

Make one Rust test invocation independent from every other local worktree,
terminal, or CI test step. A test invocation must never read from, mutate, drop,
or rebuild the shared development `app_db`.

The isolation unit is one invocation, not one test function or one Cargo test
binary. This keeps bootstrap cost bounded while allowing separate invocations to
run concurrently.

## Root Cause

The repository-level Cargo environment points every process at `app_db` and
sets `SKIP_DEV_INIT=1`. Consequently, concurrent test invocations share mutable
users, role assignments, RLS context prerequisites, and migration state. A
different test or worktree can invalidate a running test's fixed platform
assignment, producing failures such as `platform isolation bypass requires the
fixed platform assignment`.

CI reduces the probability by serializing tests, but serialization does not
provide ownership or isolation and does not protect local concurrent work.

## Chosen Architecture

Add a repository-owned `scripts/test-isolated-db.sh` runner and expose it as the
Cargo alias `cargo test-isolated`.

For each invocation the runner will:

1. Read the existing `SERVICE_DB_URL` as the maintenance connection.
2. Generate a PostgreSQL-safe, collision-resistant temporary database name.
3. Create that database without changing or rebuilding `app_db`.
4. Apply the authoritative SQL list from `scripts/db/list_init_sql.sh`, in the
   same bootstrap, migration, and seed order used by CI.
5. Export `SERVICE_DB_URL` and `SERVICE_MIGRATION_DB_URL` for the temporary
   database and force `SKIP_DEV_INIT=1`.
6. Execute `cargo test` with all caller arguments unchanged.
7. Force-drop the temporary database from an exit trap on success, test
   failure, initialization failure, or interruption.

The runner will use standard PostgreSQL client commands already required by
the project. It will not embed database credentials, introduce a database
library, alter production configuration, or modify the development database.

## URL and Safety Rules

- Preserve the scheme, authority, credentials, port, and query string from
  `SERVICE_DB_URL`; replace only the final database path segment.
- Accept only generated database names with the fixed `e2br3_test_` prefix and
  lowercase ASCII letters, digits, and underscores.
- Refuse cleanup if the resolved name lacks that prefix.
- Pass database names as explicit command arguments; do not use shell globs or
  unresolved variables as destructive targets.
- Use `dropdb --force` so leaked test connections cannot prevent cleanup.
- Print the temporary database name, but never print credentials or the full
  connection URL.

## CI Integration

The main lib-core and remaining-workspace Rust test steps will invoke the
isolated runner. The existing CI `app_db` initialization remains available for
release validation commands that are intentionally executed afterward, but
earlier test suites will no longer mutate it.

CI and local execution therefore use the same database bootstrap ordering and
the same isolation runner.

## Failure Behavior

- Missing `SERVICE_DB_URL` or PostgreSQL client commands: fail before creating
  anything with a direct diagnostic.
- Database creation failure: fail without running tests.
- SQL initialization failure: stop, run cleanup, and return failure.
- Test failure: preserve the test exit status after cleanup.
- Cleanup failure: report it and return failure when the tests otherwise
  succeeded; never hide an existing test failure.

## Testing

Implementation follows test-driven development:

1. A shell contract test supplies fake `createdb`, `psql`, `dropdb`, and Cargo
   commands. It verifies unique prefixed naming, URL rewriting, environment
   propagation, argument forwarding, cleanup, and exit-status preservation.
2. The contract test is run before implementation and must fail because the
   runner does not exist.
3. After implementation, run the contract test to green.
4. Run a real isolated RBAC grant-profile suite and confirm the temporary
   database is absent afterward.
5. Run two isolated lightweight invocations concurrently and confirm they use
   different database names and both clean up.

## Non-Goals

- One database per test function or Cargo test binary.
- Replacing PostgreSQL with mocks or an in-memory database.
- Refactoring production connection management.
- Repairing unrelated tests or changing RBAC behavior.
- Automatically intercepting plain `cargo test`; isolated database tests use
  the explicit `cargo test-isolated` repository command.
