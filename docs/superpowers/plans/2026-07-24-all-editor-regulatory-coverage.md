# All Editor Regulatory Coverage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Certify every real case-editor field and the submission/export-owned N Message Header against registry mapping, portable constraints, business validation, ICH/FDA/MFDS prose, persistence, and UI roundtrip evidence.

**Architecture:** Complete the source-coverage infrastructure from the companion plan, then expand certification one contract at a time. Existing `CI`, `RP`, `SD`, and `LR` contracts are audited first. `SI`, `DM`, `DH`, `RE`, `LB`, `DG`, and `NR` each receive a strict field contract before their prose coverage can be claimed. N uses a separate submission/export contract and never re-enters the case editor.

**Tech Stack:** Python 3 `unittest`, JSON Schema, Rust 2021, Axum integration tests, PostgreSQL, Next.js/TypeScript, Vitest, Playwright.

## Global Constraints

- The editor scope is exactly `CI`, `RP`, `SD`, `LR`, `SI`, `DM`, `DH`, `RE`, `LB`, `DG`, and `NR`.
- `AE` and `AT` are audit screens and `WF` is workflow state; all three are excluded.
- N Message Header is generated at submission/export time and must not appear in `CI` or `SD`.
- ICH, FDA, and MFDS prose inventories remain generated evidence; they do not execute at runtime.
- Business validation continues through the canonical rule catalog and section rule tables.
- Save constraints continue through the portable constraint catalog and bindings.
- A `deferred` prose requirement forces every affected `complete` field to `incomplete`.
- A missing implementation is never classified as `guidance` or `not_applicable`.
- Run only the failing focused test while iterating; run broader gates once at the final task.
- Use the disposable current-worktree PostgreSQL test instance for DB/API verification.
- Preserve `registry/tools/test_rule_source_coverage.py` if it already contains uncommitted infrastructure work.

---

### Task 1: Finish the Shared Source-Coverage Infrastructure

**Files:**
- Follow and complete Tasks 2–4 in `docs/superpowers/plans/2026-07-24-regulatory-rule-source-coverage.md`
- Create: `registry/rule-source-coverage.schema.json`
- Create: `registry/rule-source-coverage.json`
- Create: `registry/tools/rule_source_coverage.py`
- Modify: `registry/tools/validate.py`
- Test: `registry/tools/test_rule_source_coverage.py`
- Test: `crates/libs/validator/src/rule_source_coverage_tests.rs`

**Interfaces:**
- Produces `validate_coverage_structure(root, result)`.
- Produces `validate_editor_coverage(root, registry_rows, coverage, result)`.
- Enforces compiled business-rule and portable-constraint references.

- [ ] **Step 1: Prove the unfinished infrastructure is red**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
```

Expected: failure because the implementation module or production coverage file is absent.

- [ ] **Step 2: Complete the Python and Rust infrastructure from the companion plan**

Implement the schema, FNV-1a source hash, structural checks, audited-contract checks, deferred-completion gate, and compiled-reference test exactly as specified by Tasks 2–4 of the companion plan.

- [ ] **Step 3: Prove the infrastructure is green**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
python3 registry/tools/validate.py
```

Expected: all unit tests pass and registry validation prints `registry validation passed`.

- [ ] **Step 4: Commit the infrastructure**

```bash
git add \
  registry/rule-source-coverage.schema.json \
  registry/rule-source-coverage.json \
  registry/tools/rule_source_coverage.py \
  registry/tools/test_rule_source_coverage.py \
  registry/tools/validate.py \
  crates/libs/validator/src/lib.rs \
  crates/libs/validator/src/rule_source_coverage_tests.rs
git commit -m "feat: gate field completion on regulatory source coverage"
```

---

### Task 2: Add a Deterministic Contract Audit Report

**Files:**
- Modify: `registry/tools/rule_source_coverage.py`
- Modify: `registry/tools/validate.py`
- Test: `registry/tools/test_rule_source_coverage.py`

**Interfaces:**
- Produces `audit_contract_sources(root: Path, page: str) -> list[dict[str, str]]`.
- CLI: `python3 registry/tools/validate.py --report-rule-source-coverage PAGE`.
- Each row contains `page`, `fieldId`, `element`, `authority`, `coverage`, and `disposition`.

- [ ] **Step 1: Add a failing deterministic-report test**

Create a fixture where `ICH/C.4.r.2` is covered and `FDA/C.4.r.2` is absent. Assert the returned rows are authority-sorted and explicitly report `covered` and `missing`.

- [ ] **Step 2: Run the focused test**

```bash
python3 -m unittest registry.tools.test_rule_source_coverage.RuleSourceCoverageTests.test_contract_audit_report_is_deterministic -v
```

Expected: failure because `audit_contract_sources` does not exist.

- [ ] **Step 3: Implement the report**

Derive rows only from the strict contract, registry rows, generated prose dictionaries, and the reviewed crosswalk. Do not add a committed inventory or page-to-element mapping.

- [ ] **Step 4: Verify and commit**

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
git add registry/tools/rule_source_coverage.py registry/tools/validate.py registry/tools/test_rule_source_coverage.py
git commit -m "feat: report editor rule source coverage"
```

---

### Task 3: Audit Existing Contracts CI, RP, SD, and LR

**Files:**
- Modify: `registry/rule-source-coverage.json`
- Modify when evidence fails: `registry/editor-contracts/ci.json`
- Modify when evidence fails: `registry/editor-contracts/rp.json`
- Modify when evidence fails: `registry/editor-contracts/sd.json`
- Modify when evidence fails: `registry/editor-contracts/lr.json`
- Modify when completion is false: `registry/sections/c-safety-report.json`

**Interfaces:**
- Consumes the deterministic report from Task 2.
- Produces `auditedPages: ["CI", "RP", "SD", "LR"]`.

- [ ] **Step 1: Generate the four audits**

```bash
python3 registry/tools/validate.py --report-rule-source-coverage CI
python3 registry/tools/validate.py --report-rule-source-coverage RP
python3 registry/tools/validate.py --report-rule-source-coverage SD
python3 registry/tools/validate.py --report-rule-source-coverage LR
```

Expected: a finite authority-by-element list for each page; missing entries are visible.

- [ ] **Step 2: Classify every reported prose requirement**

For each missing entry, add one or more requirements with exactly one of:

- `business_rule` plus compiled canonical rule codes;
- `constraint` plus compiled and bound portable constraint codes;
- `guidance` plus a non-executable reason grounded in the prose;
- `deferred` plus the concrete missing implementation.

If any requirement is `deferred`, change the affected registry field from `complete` to `incomplete` and record the failed stage and action.

- [ ] **Step 3: Run all four strict gates**

```bash
python3 registry/tools/validate.py --strict-editor-contract CI
python3 registry/tools/validate.py --strict-editor-contract RP
python3 registry/tools/validate.py --strict-editor-contract SD
python3 registry/tools/validate.py --strict-editor-contract LR
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
```

Expected: all five commands pass.

- [ ] **Step 4: Commit**

```bash
git add registry/rule-source-coverage.json registry/editor-contracts registry/sections/c-safety-report.json
git commit -m "feat: audit existing editor regulatory coverage"
```

---

### Task 4: Certify SI

**Files:**
- Create: `registry/editor-contracts/si.json`
- Modify: `registry/sections/c-safety-report.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/si-editor-contract.test.ts`

**Interfaces:**
- Page ID `SI`.
- Covers C.5 study fields only.

- [ ] **Step 1: Add failing strict-contract and roundtrip tests**

Require every SI registry field to have projection, frontend path, PATCH/readback, constraint, and business-validation evidence. Add one API test that patches each scalar SI field, reloads `/editor/pages/SI`, and compares canonical paths.

- [ ] **Step 2: Run only SI tests and confirm red**

```bash
python3 registry/tools/validate.py --strict-editor-contract SI
cargo test -p web-server --test api case_editor_contract_web::editor_si_page_patch_roundtrips_all_contract_fields -- --nocapture
```

Expected: failure for the absent contract or missing runtime mapping.

- [ ] **Step 3: Add the contract and repair only proven gaps**

Create `si.json` using the existing editor-contract schema. Add every ICH/FDA/MFDS prose entry reported for SI. Downgrade any field whose executable or roundtrip evidence remains absent.

- [ ] **Step 4: Verify and commit**

```bash
python3 registry/tools/validate.py --strict-editor-contract SI
cargo test -p web-server --test api case_editor_contract_web::editor_si_page_patch_roundtrips_all_contract_fields -- --nocapture
npx vitest run tests/registry/si-editor-contract.test.ts
git add registry/editor-contracts/si.json registry/sections/c-safety-report.json registry/rule-source-coverage.json crates/services/web-server/tests/api/case_editor_contract_web.rs
git commit -m "feat: certify SI editor coverage"
```

---

### Task 5: Certify DM

**Files:**
- Create: `registry/editor-contracts/dm.json`
- Modify: `registry/sections/d-patient.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/dm-editor-contract.test.ts`

**Interfaces:** Page ID `DM`; covers patient demographic fields, excluding patient drug history.

- [ ] Add a failing field-by-field DM projection/PATCH/reload test and strict-contract test.
- [ ] Run `python3 registry/tools/validate.py --strict-editor-contract DM` and the exact `editor_dm_page_patch_roundtrips_all_contract_fields` API test; confirm red.
- [ ] Create `dm.json`, classify every reported ICH/FDA/MFDS source, repair proven mappings, and mark unsupported fields `incomplete`.
- [ ] Run the DM strict gate, exact API test, and `npx vitest run tests/registry/dm-editor-contract.test.ts`.
- [ ] Commit as `feat: certify DM editor coverage`.

---

### Task 6: Certify DH

**Files:**
- Create: `registry/editor-contracts/dh.json`
- Modify: `registry/sections/d-patient.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/dh-editor-contract.test.ts`

**Interfaces:** Page ID `DH`; covers repeating patient drug-history rows and their create/update/delete/restore lifecycle.

- [ ] Add failing tests for row creation, update, soft delete, `include_deleted`, restore, and reload.
- [ ] Run the DH strict gate and exact failing API lifecycle test.
- [ ] Create `dh.json`, classify all authority prose, repair only evidenced gaps, and downgrade false completion.
- [ ] Run the DH strict gate, exact API test, and frontend contract test.
- [ ] Commit as `feat: certify DH editor coverage`.

---

### Task 7: Certify RE

**Files:**
- Create: `registry/editor-contracts/re.json`
- Modify: `registry/sections/e-reaction.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/re-editor-contract.test.ts`

**Interfaces:** Page ID `RE`; covers E reaction data fields. It does not cover the `AE` audit screen.

- [ ] Add failing repeating-row projection/PATCH/delete/restore/reload tests for RE.
- [ ] Prove the RE strict gate and exact API test are red.
- [ ] Create `re.json`, classify all authority prose, and correct only demonstrated gaps.
- [ ] Verify the RE strict gate, exact API test, and frontend contract test.
- [ ] Commit as `feat: certify RE editor coverage`.

---

### Task 8: Certify LB

**Files:**
- Create: `registry/editor-contracts/lb.json`
- Modify: `registry/sections/f-test.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/lb-editor-contract.test.ts`

**Interfaces:** Page ID `LB`; covers repeating laboratory/test-result rows.

- [ ] Add failing LB row lifecycle and canonical reload tests.
- [ ] Prove the LB strict gate and exact API test are red.
- [ ] Create `lb.json`, classify all authority prose, repair proven gaps, and downgrade unsupported fields.
- [ ] Verify the LB strict gate, exact API test, and frontend contract test.
- [ ] Commit as `feat: certify LB editor coverage`.

---

### Task 9: Certify DG

**Files:**
- Create: `registry/editor-contracts/dg.json`
- Modify: `registry/sections/g-drug.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/dg-editor-contract.test.ts`

**Interfaces:** Page ID `DG`; covers drug, substance, dosage, indication, and reaction-relatedness child rows.

- [ ] Add failing DG nested-row lifecycle and canonical reload tests.
- [ ] Prove the DG strict gate and exact API test are red.
- [ ] Create `dg.json`, classify all authority prose, correct proven nested mappings, and downgrade unsupported fields.
- [ ] Verify the DG strict gate, exact API test, and frontend contract test.
- [ ] Commit as `feat: certify DG editor coverage`.

---

### Task 10: Certify NR

**Files:**
- Create: `registry/editor-contracts/nr.json`
- Modify: `registry/sections/h-narrative.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Test in frontend: `tests/registry/nr-editor-contract.test.ts`

**Interfaces:** Page ID `NR`; covers narrative and reporter-comment fields.

- [ ] Add failing NR projection/PATCH/reload tests.
- [ ] Prove the NR strict gate and exact API test are red.
- [ ] Create `nr.json`, classify all authority prose, repair proven gaps, and downgrade unsupported fields.
- [ ] Verify the NR strict gate, exact API test, and frontend contract test.
- [ ] Commit as `feat: certify NR editor coverage`.

---

### Task 11: Certify N at the Submission/Export Boundary

**Files:**
- Create: `registry/submission-contracts/schema.json`
- Create: `registry/submission-contracts/message-header.json`
- Create: `registry/tools/submission_contract.py`
- Modify: `registry/tools/validate.py`
- Modify: `registry/sections/n-message-header.json`
- Modify: `registry/rule-source-coverage.json`
- Test: `registry/tools/test_submission_contract.py`
- Test: `crates/services/web-server/tests/api/export_contract_web.rs`
- Test in frontend: `tests/submission/message-header.test.ts`

**Interfaces:**
- Contract ID `MESSAGE_HEADER`.
- Proves frontend builder output → API/export request → `MessageHeaderBmc`/export model → XML N nodes.
- N fields are forbidden from all editor contracts.

- [ ] **Step 1: Add failing boundary tests**

Assert that no editor contract contains an N registry row, the frontend builder emits fresh identifiers and timestamps, and XML export emits N.1/N.2 values from the generated header.

- [ ] **Step 2: Run focused tests and confirm red**

```bash
python3 -m unittest registry.tools.test_submission_contract -v
cargo test -p web-server --test api export_contract_web::submission_generated_message_header_roundtrips_to_xml -- --nocapture
npx vitest run tests/submission/message-header.test.ts
```

- [ ] **Step 3: Implement the strict submission contract**

The contract schema requires registry ID, frontend builder field, backend field, export XPath, validation evidence, and roundtrip evidence for every N row. Add authority-complete prose classifications without assigning `editor_page`.

- [ ] **Step 4: Verify and commit**

Run the three focused commands from Step 2 plus default registry validation. Commit as `feat: certify submission message header coverage`.

---

### Task 12: Final Cross-Layer Verification

**Files:** Verification only.

- [ ] **Step 1: Run every strict registry contract**

```bash
for page in CI RP SD LR SI DM DH RE LB DG NR; do
  python3 registry/tools/validate.py --strict-editor-contract "$page" || exit 1
done
python3 registry/tools/validate.py --strict-submission-contract MESSAGE_HEADER
python3 registry/tools/validate.py
```

Expected: every command prints `registry validation passed`.

- [ ] **Step 2: Run source and runtime parity gates**

```bash
python3 -m unittest registry.tools.test_rule_source_coverage registry.tools.test_submission_contract -v
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
cargo test -p validator implemented_case_registry_matches_case_validate_catalog --lib
cargo test -p validator every_portable_rule_is_bound_or_explicitly_excluded_once --lib
```

Expected: zero failures.

- [ ] **Step 3: Run focused backend editor and export tests**

```bash
cargo test -p web-server --test api case_editor_contract_web -- --nocapture
cargo test -p web-server --test api export_contract_web -- --nocapture
```

Expected: zero failures.

- [ ] **Step 4: Run focused frontend contract tests and type checking**

```bash
npx vitest run tests/registry tests/submission/message-header.test.ts
npx tsc --noEmit
```

Expected: zero failures and no TypeScript diagnostics.

- [ ] **Step 5: Run one live browser roundtrip per editor page**

For each of `CI`, `RP`, `SD`, `LR`, `SI`, `DM`, `DH`, `RE`, `LB`, `DG`, and `NR`: edit one contract field, save, reload, and verify the same canonical value. Force one invalid portable-constraint payload through the API and verify a non-2xx structured `ConstraintViolation` whose path maps to the same frontend field.

- [ ] **Step 6: Check completion truthfulness**

```bash
python3 registry/tools/validate.py --report-rule-source-coverage ALL
git diff --check
git status --short
```

Expected: no `missing` coverage on certified surfaces, no `complete` field backed by `deferred`, no whitespace errors, and only intentionally preserved user files remain untracked.
