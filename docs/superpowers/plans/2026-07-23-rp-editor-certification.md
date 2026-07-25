# RP Editor Certification Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Certify every Reporter (`RP`, E2B C.2.r) editor field against the current `dev` baseline from registry mapping through frontend save, backend constraints and business validation, database persistence, and reload roundtrip.

**Architecture:** Extend the CI field-level editor-contract pattern to RP without changing the distinction between portable save constraints and business validation. RP remains a repeating `primarySources[]` owner: frontend persistence may continue using the existing collection endpoint, while the canonical editor projection/PATCH endpoint provides an independently tested field contract. A disposable PostgreSQL instance initialized from the current backend worktree is used for integration and browser verification.

**Tech Stack:** Rust/Axum/sqlx, Python registry gates, Next.js/React Hook Form/TypeScript, Jest, Playwright, PostgreSQL 16.

## Global Constraints

- Work from the current backend branch after `origin/dev` merge commit `ee118f1d` and merge current frontend `origin/dev` before RP code changes.
- Treat `validation` as business-rule validation only; portable save rejection remains `ConstraintViolation`.
- Keep the field-level registry status `complete` only when every applicable evidence stage passes.
- Use `not_applicable` only for a genuine absence of a catalog or business rule, never for missing implementation.
- Run failed tests by exact name; do not restart broad suites during diagnosis.
- Do not use the shared Homebrew PostgreSQL database or persistent development Docker volume for certification.

---

### Task 1: Synchronize the frontend with current dev

**Files:**
- Merge target: frontend branch `codex/unify-presave-canonical-names`
- Preserve: `lib/validation/syntax.ts`
- Preserve: `lib/case-save/pages/CI/save.ts`
- Preserve: `app/(protected)/[authority]/case/[id]/detail/CI/**`
- Preserve: `__tests__/case-form/CaseEditor.validation-errors.integration.test.ts`

**Interfaces:**
- Consumes: frontend commit `92da978` and `origin/dev`
- Produces: a clean frontend worktree containing both generated RBAC changes and the canonical CI validation/save changes

- [ ] **Step 1: Fetch and inspect the frontend divergence**

```sh
git fetch origin dev
git rev-list --left-right --count HEAD...origin/dev
git status --short --branch
```

Expected: the current feature branch is ahead of and behind `origin/dev`, with no uncommitted files.

- [ ] **Step 2: Merge current dev**

```sh
git merge --no-edit origin/dev
```

Expected: merge succeeds. If a conflict touches generated authorization files, keep `origin/dev`; if it touches CI validation/save files, retain the feature-branch behavior and adapt imports to the merged API.

- [ ] **Step 3: Verify the preserved frontend baseline**

```sh
npx jest --runInBand \
  __tests__/architecture.no-legacy-case-editor-validation.test.ts \
  __tests__/case-form/CaseEditor.validation-errors.integration.test.ts \
  __tests__/case-save/reporter.coordinator.test.ts \
  __tests__/validation.catalog-generated.test.ts
npx tsc --noEmit
```

Expected: all selected tests and TypeScript compilation pass.

- [ ] **Step 4: Commit only conflict-resolution changes when present**

```sh
git status --short
git add -u
git commit -m "merge: align editor validation with current dev"
```

Expected: no extra commit when the merge was conflict-free; otherwise one intentional conflict-resolution commit.

---

### Task 2: Add the RP field-level registry contract

**Files:**
- Create: `registry/editor-contracts/rp.json`
- Modify: `registry/sections/c-safety-report.json`
- Modify: `registry/tools/test_validate.py`
- Modify: `registry/README.md`

**Interfaces:**
- Consumes: `registry/tools/editor_contract.py::validate_editor_contract`
- Produces: `python3 registry/tools/validate.py --strict-editor-contract RP`

The RP contract contains exactly these 30 canonical paths:

```text
C.2.r.1.1                                      primarySources[].reporterTitle
C.2.r.1.2                                      primarySources[].reporterGivenName
C.2.r.1.3                                      primarySources[].reporterMiddleName
C.2.r.1.4                                      primarySources[].reporterFamilyName
C.2.r.2.1                                      primarySources[].reporterOrganization
C.2.r.2.2                                      primarySources[].reporterDepartment
C.2.r.2.3                                      primarySources[].reporterStreet
C.2.r.2.4                                      primarySources[].reporterCity
C.2.r.2.5                                      primarySources[].reporterState
C.2.r.2.6                                      primarySources[].reporterPostcode
C.2.r.2.7                                      primarySources[].reporterTelephone
C.2.r.3                                        primarySources[].reporterCountry
FDA.C.2.r.2.8                                  primarySources[].reporterEmail
C.2.r.4                                        primarySources[].qualification
C.2.r.4.KR.1                                   primarySources[].qualificationKr1
C.2.r.5                                        primarySources[].primarySourceForRegulatoryPurposes
C.2.r.local.reporterTitleNullFlavor            primarySources[].reporterTitleNullFlavor
C.2.r.local.reporterGivenNameNullFlavor        primarySources[].reporterGivenNameNullFlavor
C.2.r.local.reporterMiddleNameNullFlavor       primarySources[].reporterMiddleNameNullFlavor
C.2.r.local.reporterFamilyNameNullFlavor       primarySources[].reporterFamilyNameNullFlavor
C.2.r.local.reporterOrganizationNullFlavor     primarySources[].reporterOrganizationNullFlavor
C.2.r.local.reporterDepartmentNullFlavor       primarySources[].reporterDepartmentNullFlavor
C.2.r.local.reporterStreetNullFlavor           primarySources[].reporterStreetNullFlavor
C.2.r.local.reporterCityNullFlavor             primarySources[].reporterCityNullFlavor
C.2.r.local.reporterStateNullFlavor            primarySources[].reporterStateNullFlavor
C.2.r.local.reporterPostcodeNullFlavor         primarySources[].reporterPostcodeNullFlavor
C.2.r.local.reporterTelephoneNullFlavor        primarySources[].reporterTelephoneNullFlavor
C.2.r.local.reporterCountryNullFlavor          primarySources[].reporterCountryNullFlavor
C.2.r.local.reporterEmailNullFlavor            primarySources[].reporterEmailNullFlavor
C.2.r.local.qualificationNullFlavor            primarySources[].qualificationNullFlavor
```

All entries use this repeating-row PATCH owner:

```json
"patch": { "kind": "rows", "owner": "primarySources" }
```

Concrete roundtrip values use one populated row:

```json
{
  "reporterTitle": "Dr",
  "reporterGivenName": "Mina",
  "reporterMiddleName": "J",
  "reporterFamilyName": "Kim",
  "reporterOrganization": "QVIS Safety",
  "reporterDepartment": "Pharmacovigilance",
  "reporterStreet": "1 Main Street",
  "reporterCity": "Seoul",
  "reporterState": "Seoul",
  "reporterPostcode": "04524",
  "reporterTelephone": "+821012345678",
  "reporterCountry": "KR",
  "reporterEmail": "reporter@example.test",
  "qualification": "1",
  "qualificationKr1": "2",
  "primarySourceForRegulatoryPurposes": "1"
}
```

Null-flavor values use a separate non-primary row with blank paired values and
`MSK` for identity/address fields, `UNK` for country and qualification, and
`NASK` for telephone/email where allowed by the catalog.

- [ ] **Step 1: Add a failing RP strict-gate test**

Add to `registry/tools/test_validate.py`:

```python
def test_complete_rp_row_requires_field_contract(self) -> None:
    row = self.editor_row(code="C.2.r.1.1")
    row["editor_page"] = "RP"
    row["frontend"]["section"] = "primarySources"
    row["frontend"]["field"] = "reporterTitle"
    result = validate.ValidationResult()
    editor_contract.validate_editor_contract(
        [row], {"pageId": "RP", "fields": []}, result
    )
    self.assertIn("C.2.r.1.1 complete but missing from RP editor contract", result.errors)
```

- [ ] **Step 2: Run the strict-gate test and confirm red**

```sh
python3 -m unittest registry.tools.test_validate.RegistryValidatorTests.test_complete_rp_row_requires_field_contract
```

Expected: FAIL because the RP contract is missing the complete field.

- [ ] **Step 3: Add `editor_page: "RP"` to the 30 registry rows and create `rp.json`**

For each contract entry, set `frontendPath` and `projectionPath` to the canonical
path above, use the `primarySources` row owner, and record one portable rule from
`crates/libs/validator/src/portable_bindings/c.rs`. Value fields use their
`*.LENGTH.MAX` rule except `qualificationKr1`, which uses
`MFDS.C.2.r.4.KR.1.LENGTH.MAX`. Null-flavor companions use the matching
`*.NULLFLAVOR.ALLOWED` rule. `reporterEmailNullFlavor` is
`constraint.not_applicable` only if the catalog still has no binding after the
inventory test. Business-validation evidence uses the exact matching
`primarySources.0` field issue path when `case/sections/c.rs` emits a semantic
rule; otherwise it records a field-specific `not_applicable` reason.

- [ ] **Step 4: Make the RP strict gate green**

```sh
python3 registry/tools/validate.py --strict-editor-contract RP
python3 -m unittest registry.tools.test_validate
```

Expected: `registry validation passed`; all registry validator tests pass.

- [ ] **Step 5: Commit the registry contract**

```sh
git add registry/editor-contracts/rp.json registry/sections/c-safety-report.json registry/tools/test_validate.py registry/README.md
git commit -m "test: certify RP editor field contract"
```

---

### Task 3: Prove backend RP projection, constraints, and roundtrip

**Files:**
- Modify: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Modify only if a test exposes a defect: `crates/services/web-server/src/web/rest/case_editor_rest/direct.rs`
- Modify only if a test exposes a defect: `crates/libs/validator/src/portable_bindings/c.rs`
- Modify only if a test exposes a defect: `crates/libs/validator/src/case/sections/c.rs`

**Interfaces:**
- Consumes: `PATCH /api/cases/{case_id}/editor/pages/RP`
- Produces: canonical `rows.primarySources[]`, structured `ConstraintViolation`, and business-validation issue paths

- [ ] **Step 1: Add a failing full-field roundtrip test**

Add `editor_rp_complete_fields_round_trip`. Create a case, PATCH two
`primarySources` rows using the concrete and null-flavor payloads from Task 2,
GET `/editor/pages/RP?authorities=ich,fda,mfds`, and assert every canonical
camelCase field. Also query `primary_sources` under full DB context and assert
the corresponding snake_case columns.

The core assertions are:

```rust
assert_eq!(body["rows"]["primarySources"][0]["reporterGivenName"], "Mina");
assert_eq!(body["rows"]["primarySources"][0]["qualificationKr1"], "2");
assert_eq!(body["rows"]["primarySources"][1]["reporterGivenNameNullFlavor"], "MSK");
assert_eq!(body["rows"]["primarySources"][1]["qualificationNullFlavor"], "UNK");
```

- [ ] **Step 2: Run the roundtrip test and confirm red or green**

```sh
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_complete_fields_round_trip' \
  -- --exact --nocapture
```

Expected before implementation: FAIL on the first missing or wrongly named field; if it already passes, retain the test as evidence and do not alter production code.

- [ ] **Step 3: Add a table-driven portable constraint rejection test**

Add `editor_rp_portable_constraints_return_structured_paths`. For each portable
binding, submit exactly one invalid value through RP PATCH and assert:

```rust
assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
assert_eq!(body["error"]["message"], "ConstraintViolation");
assert_eq!(body["error"]["data"]["details"][0]["ruleCode"], expected_rule);
assert_eq!(body["error"]["data"]["details"][0]["path"], expected_path);
```

Use the exact rule/path pairs from `portable_bindings/c.rs`; run each case as a
separate request so one invalid field cannot hide another.

- [ ] **Step 4: Add focused business-validation path tests**

Add `editor_rp_business_validation_paths_are_canonical` and assert at least:

```text
ICH.C.2.r.4.REQUIRED           primarySources.0.qualification
ICH.C.2.r.5.REQUIRED           primarySources.0.primarySourceForRegulatoryPurposes
FDA.C.2.r.2.8.REQUIRED         primarySources.0.reporterEmail
MFDS.C.2.r.4.KR.1.REQUIRED     primarySources.0.qualificationKr1
```

These issues must come from the validation report/cache response, not from the
save-rejection HTTP response.

- [ ] **Step 5: Implement only defects demonstrated by Steps 2–4**

Preserve canonical camelCase at the REST boundary and snake_case only in DB
models. Do not add compatibility aliases. If a portable binding is missing,
add it to the RP entries in `portable_bindings/c.rs`; if a projection field is
missing, add it to the RP row serialization/deserialization in `direct.rs`.

- [ ] **Step 6: Re-run only the exact RP backend tests**

```sh
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_complete_fields_round_trip' \
  -- --exact --nocapture
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_portable_constraints_return_structured_paths' \
  -- --exact --nocapture
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_business_validation_paths_are_canonical' \
  -- --exact --nocapture
```

Expected: each command reports one passed test and zero failures.

- [ ] **Step 7: Commit backend RP evidence and fixes**

```sh
git add crates/services/web-server/tests/api/case_editor_contract_web.rs \
  crates/services/web-server/src/web/rest/case_editor_rest/direct.rs \
  crates/libs/validator/src/portable_bindings/c.rs \
  crates/libs/validator/src/case/sections/c.rs
git commit -m "feat: complete RP editor roundtrip"
```

Stage only files that actually changed.

---

### Task 4: Prove frontend RP save, catalog validation, and inline errors

**Files:**
- Modify: `__tests__/case-save/reporter.coordinator.test.ts`
- Modify: `__tests__/field-error-banners/reporter.test.ts`
- Modify: `__tests__/validation.catalog-generated.test.ts`
- Modify only if a test exposes a defect: `lib/case-save/pages/RP/save.ts`
- Modify only if a test exposes a defect: `lib/api/endpoints/cases/core/detail.reporter.ts`
- Modify only if a test exposes a defect: `lib/validation/backendFieldBanners.ts`
- Modify only if a test exposes a defect: `app/(protected)/[authority]/case/[id]/detail/RP/components/ReporterEditorPanel.tsx`

**Interfaces:**
- Consumes: `primarySources[]` projection, generated catalog bindings, and structured backend error details
- Produces: owned dirty-path save tasks, canonical reload values, disabled invalid inputs, and inline field messages

- [ ] **Step 1: Extend the reporter coordinator roundtrip fixture**

Add one test that passes all 30 fields through `reporterPageSave.prepare`, awaits
the save task, feeds the backend snake_case readback through
`buildReporterDetail`, and asserts the complete canonical object. The save
payload must contain only camelCase fields and must not contain
`qualification_kr1`, `email`, or other backend aliases.

- [ ] **Step 2: Extend inline-error coverage to all RP companions**

Add these paths to the existing `it.each` matrix:

```ts
"primarySources.0.reporterTitleNullFlavor",
"primarySources.0.reporterGivenNameNullFlavor",
"primarySources.0.reporterMiddleNameNullFlavor",
"primarySources.0.reporterFamilyNameNullFlavor",
"primarySources.0.reporterOrganizationNullFlavor",
"primarySources.0.reporterDepartmentNullFlavor",
"primarySources.0.reporterStreetNullFlavor",
"primarySources.0.reporterCityNullFlavor",
"primarySources.0.reporterStateNullFlavor",
"primarySources.0.reporterPostcodeNullFlavor",
"primarySources.0.reporterTelephoneNullFlavor",
"primarySources.0.reporterCountryNullFlavor",
"primarySources.0.reporterEmailNullFlavor",
"primarySources.0.qualificationNullFlavor",
"primarySources.0.qualificationKr1",
```

- [ ] **Step 3: Add an RP generated-catalog parity test**

Filter generated bindings where `section === "RP"` and assert every verified
`constraint.ruleCode` from `registry/editor-contracts/rp.json` occurs in the
generated frontend catalog. Assert there is no RP rule source outside generated
catalog files.

- [ ] **Step 4: Run the focused frontend tests and confirm red or green**

```sh
npx jest --runInBand \
  __tests__/case-save/reporter.coordinator.test.ts \
  __tests__/field-error-banners/reporter.test.ts \
  __tests__/validation.catalog-generated.test.ts \
  __tests__/architecture.no-legacy-case-editor-validation.test.ts
```

Expected before implementation: failures identify missing readback, path, or
catalog coverage. Already passing assertions remain evidence and require no
production change.

- [ ] **Step 5: Implement only demonstrated frontend defects**

Keep `primarySources[]` as the canonical owner. Map backend snake_case only in
`detail.reporter.ts`; do not add snake_case form fields. Preserve the existing
generated `lib/validation/syntax.ts` engine and structured error mapping.

- [ ] **Step 6: Verify focused frontend tests and type checking**

```sh
npx jest --runInBand \
  __tests__/case-save/reporter.coordinator.test.ts \
  __tests__/field-error-banners/reporter.test.ts \
  __tests__/validation.catalog-generated.test.ts \
  __tests__/architecture.no-legacy-case-editor-validation.test.ts
npx tsc --noEmit
```

Expected: all selected tests pass; TypeScript exits zero.

- [ ] **Step 7: Commit frontend RP evidence and fixes**

```sh
git add __tests__/case-save/reporter.coordinator.test.ts \
  __tests__/field-error-banners/reporter.test.ts \
  __tests__/validation.catalog-generated.test.ts \
  lib/case-save/pages/RP/save.ts \
  lib/api/endpoints/cases/core/detail.reporter.ts \
  lib/validation/backendFieldBanners.ts \
  'app/(protected)/[authority]/case/[id]/detail/RP/components/ReporterEditorPanel.tsx'
git commit -m "test: prove RP catalog and inline errors"
```

Stage only files that actually changed.

---

### Task 5: Perform RP live smoke verification and finalize statuses

**Files:**
- Modify only if evidence changes status: `registry/sections/c-safety-report.json`
- Modify only if evidence changes status: `registry/editor-contracts/rp.json`

**Interfaces:**
- Consumes: running frontend, current backend, disposable PostgreSQL, certified RP contract
- Produces: live edit/save/reload evidence and final field statuses

- [ ] **Step 1: Start a disposable PostgreSQL instance**

Use explicit RP-only names and a non-shared volume. Temporarily stop any local
service that intercepts the configured test port, and verify the connected
`authorization_roles` owner is `app_user` before running the backend.

```sh
psql "$SERVICE_DB_URL" -Atc \
  "select current_user, pg_get_userbyid(relowner) from pg_class where relname='authorization_roles'"
```

Expected: `app_user|app_user`.

- [ ] **Step 2: Run one browser RP roundtrip**

Create/open a case, add two reporters using the concrete and null-flavor
fixtures, save, reload, and assert every visible value. Confirm the header is
not `Unsaved` after reload and the Save button remains disabled.

- [ ] **Step 3: Verify frontend constraint and forced API rejection**

Enter an over-length reporter title in the UI and confirm Save is disabled with
the catalog message. Send the same invalid payload directly to the RP API and
assert a non-2xx `ConstraintViolation` response with
`ICH.C.2.r.1.1.LENGTH.MAX` and
`primarySources.0.reporterTitle`; verify the inline RP field displays that
message when the structured response is supplied to the form.

- [ ] **Step 4: Run final focused gates**

```sh
python3 registry/tools/validate.py --strict-editor-contract RP
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_complete_fields_round_trip' \
  -- --exact
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_portable_constraints_return_structured_paths' \
  -- --exact
cargo test -p web-server --test api \
  'case_editor_contract_web::editor_rp_business_validation_paths_are_canonical' \
  -- --exact
npx jest --runInBand \
  __tests__/case-save/reporter.coordinator.test.ts \
  __tests__/field-error-banners/reporter.test.ts \
  __tests__/validation.catalog-generated.test.ts \
  __tests__/architecture.no-legacy-case-editor-validation.test.ts
npx tsc --noEmit
```

Expected: every command exits zero.

- [ ] **Step 5: Apply field statuses from evidence**

Keep `complete` for each field whose stages passed. Change only a demonstrably
missing or incorrect field to `incomplete`, recording the exact failed stage in
`action` and the observed evidence in `notes`. Environmental failures do not
change registry status.

- [ ] **Step 6: Commit any final status correction**

```sh
git add registry/sections/c-safety-report.json registry/editor-contracts/rp.json
git commit -m "chore: finalize RP certification status"
```

Skip this commit when no status changed.
