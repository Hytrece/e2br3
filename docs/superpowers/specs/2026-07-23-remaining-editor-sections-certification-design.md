# Remaining Editor Sections Certification Design

## Goal

Certify every registry-backed case-editor field after CI against the current
`dev` baseline. A field remains `complete` only when its registry mapping,
portable constraint coverage, business-validation coverage, editor projection,
PATCH persistence, and database reload roundtrip are all proven.

## Scope

The certification proceeds in this fixed order:

1. `CI` — Case identification
2. `RP` — Reporter / primary sources
3. `SD` — Sender
4. `LR` — Literature references
5. `SI` — Study
6. `DM` — Patient
7. `DH` — Patient drug history
8. `RE` — Reactions
9. `LB` — Test results
10. `DG` — Drugs
11. `NR` — Narrative

`CI`, `RP`, `SD`, and `LR` already have strict contracts and are the first
regulatory source-coverage rollout. `AE` and `AT` are audit screens. `WF` is a
workflow screen and does not own E2B data fields. They are outside this
registry/validation/roundtrip certification.

N Message Header is no longer an editor section and must not be assigned to
`CI` or `SD`. It is certified separately at the submission/export boundary.

## Certification Unit

The unit of completion is one registry field, not an entire file or page. Each
field contract records and proves all of the following stages:

- canonical registry row and authority
- current frontend field path
- backend model/storage mapping
- editor projection path
- PATCH key or repeating-row owner
- reload roundtrip value
- portable constraint rule, or a concrete `not_applicable` reason
- business-validation issue path, or a concrete `not_applicable` reason

A page-level strict gate aggregates these field contracts but may not infer
field completion from a page-level smoke test.

## Status Rules

A field may remain `complete` only when every applicable stage has executable
evidence and passes. A stage is `not_applicable` only when the catalog or
business-rule engine genuinely has no rule for that field; absence of an
implementation is not a valid reason.

If a stage is missing or fails because the product implementation is absent or
incorrect, the field becomes `incomplete`. Its registry row must record the
specific failed stage and required action. Environmental failures are repaired
or isolated before status is evaluated; they do not by themselves make a field
incomplete.

## Architecture

Each page receives an editor contract under `registry/editor-contracts/`, using
the CI contract schema and strict gate as the common format. The validator tool
loads the page contract, resolves every contract row back to the canonical
section registry, and rejects missing, duplicate, stale, or unsupported
evidence.

Backend integration tests exercise only the page being certified. Direct pages
prove projection, PATCH, and reload behavior; repeating-row pages additionally
prove create, update, soft delete, visibility with `include_deleted`, and
restore. Constraint tests submit one intentionally invalid field at a time and
assert the structured `ConstraintViolation` rule code and path. Business-rule
tests assert the validation issue path without treating business validation as
a save rejection.

Frontend tests verify that the page consumes the canonical projection paths,
uses catalog-derived portable constraints, maps structured save-rejection paths
to the correct inline field, and does not reintroduce hand-written legacy
regex/Zod rules.

## Data Flow

```text
canonical registry row
  -> editor field contract
  -> frontend projection path and catalog constraint binding
  -> page PATCH or row request
  -> backend portable save constraint
  -> database persistence
  -> editor projection reload
  -> frontend field value and inline error path
```

The stored value must return through the same canonical frontend path used by
the input. Compatibility aliases and legacy field names do not satisfy the
roundtrip contract.

## Execution Strategy

Certification is sequential and fail-fast by page. For each page:

1. Inventory registry rows and current frontend/backend paths.
2. Add the page editor contract and strict field-level gate.
3. Add focused failing tests for missing projection, PATCH, constraint,
   business-validation, and roundtrip evidence.
4. Implement only the missing mappings or behavior exposed by those tests.
5. Run only that page's focused tests until they pass.
6. Update field statuses from the resulting evidence and commit the page.
7. Proceed to the next page only after the current page gate passes.

Broad suites are not restarted after every failure. A failed test is rerun by
its exact name after the cause is corrected.

## Test Environment

Backend integration tests must use a disposable PostgreSQL instance initialized
from the current worktree. The shared Homebrew PostgreSQL service and persistent
development Docker volume must not be used for certification because their
schema ownership or branch state may differ. The disposable instance uses an
explicit container name, volume, and connection port; cleanup restores the
developer's original services and removes only the disposable test data.

## Final Verification

After all eleven page gates and the N submission/export gate pass:

- run the strict editor-contract gate for every certified page
- run each page's focused backend projection/PATCH/roundtrip tests
- run focused frontend catalog, inline-error, and save-state tests
- run frontend type checking
- run one live browser smoke flow per page: edit, save, reload, verify value
- confirm a catalog constraint disables or blocks invalid frontend input and
  that forced API injection returns a structured HTTP save failure on the same
  field path
- confirm submission/export generates N Message Header values without exposing
  them as CI or SD editor fields and XML export emits the generated values

The work is complete only when all field contracts are certified, the final
live frontend-to-backend roundtrip succeeds for every in-scope page, and the N
submission/export roundtrip succeeds.
