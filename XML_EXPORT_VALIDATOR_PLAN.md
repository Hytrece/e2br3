# XML Export Validator Implementation Plan

## 1. Goal

Generated E2B(R3) XML must fail locally for the same deterministic XML business-rule violations that the selected authority rejects.

- Apply ICH common rules to every export.
- Apply exactly one regional overlay: FDA or MFDS.
- Keep XSD validation responsible for XML structure, order, cardinality, and XML types.
- Implement rules as explicit Rust functions named after their official identifiers.
- Use the same validation path for export/download and submission.

This work does not replace case-data validation in `crates/libs/validator`, and it does not add a generic rule engine.

## 2. Fixed design decisions

- No runtime rule tables, JSON-driven execution, mocks, or fallback behavior.
- Official spreadsheets, CSV files, PDFs, and registry JSON are implementation evidence only.
- Parse the XML once in `export_rules.rs`; build the XPath context once and pass it to section modules.
- Reuse the current public validation entry points instead of adding a parallel validator API.
- XSD-owned checks are not duplicated as business-rule functions.
- ICH rules are implemented once. FDA and MFDS modules contain only regional additions or overrides.
- Do not create empty section files for hypothetical future rules. Add a file only when at least one real rule belongs there.
- A rule function uses its official identifier without a `validate_` prefix:
  - `D.2.2b` -> `fn d_2_2b(...)`
  - `G.k.4.r.6b` -> `fn g_k_4_r_6b(...)`
  - `FDA.C.1.7.1` -> `fn fda_c_1_7_1(...)`
  - FDA rejection rule `R0012` -> `fn r0012(...)`
- Each section module exposes only `pub(super) fn run(...)`; individual rule functions remain private.

## 3. Target structure

```text
crates/libs/xml/src/
├── validation.rs
└── validation/
    ├── export_rules.rs
    └── export_rules/
        ├── mod.rs
        ├── authority.rs
        ├── ich/
        │   ├── mod.rs
        │   ├── common.rs
        │   ├── n.rs
        │   ├── c.rs
        │   ├── d.rs
        │   ├── e.rs
        │   ├── f.rs
        │   ├── g.rs
        │   └── h.rs
        ├── fda/
        │   ├── mod.rs
        │   ├── n.rs
        │   ├── c.rs
        │   ├── d.rs
        │   ├── e.rs
        │   └── g.rs
        └── mfds/
            ├── mod.rs
            ├── n.rs
            ├── c.rs
            ├── d.rs
            ├── e.rs
            ├── f.rs
            ├── g.rs
            └── h.rs
```

The tree is the ownership target, not mandatory scaffolding. Files with no implemented rule are omitted.

## 4. Rule ownership

### `validation.rs`

- Preserve the existing public XML-validation API.
- Run normal XML parsing and XSD validation.
- Remove the PORR XSD-error suppression; schema failures must not be hidden.

### `validation/export_rules.rs`

- Parse once and create the shared XPath context.
- Call ICH common validation first.
- Resolve the requested authority once.
- Call exactly one FDA or MFDS overlay.
- Return errors in the existing error representation.

### `validation/export_rules/authority.rs`

- Reject regional elements that do not belong to the selected authority.
- Do not infer an authority from document contents and do not fall back to another authority.

### Authority section files

- Hold explicit, deterministic rules grouped by the official E2B section.
- Preserve the official rule/error code in every emitted error.
- Keep warnings distinct from rejection errors where the official source distinguishes them.

## 5. Sources of truth

Implementation must be checked against the repository-owned official material:

- `docs/refs/fda_e2b_core_regional.xlsx`
- `registry/sources/fda-core-regional-data-elements-v1.csv`
- `registry/sources/fda-rejection-warning-rules.csv`
- `docs/refs/fda_regional_implementation_guide_aug_2024.pdf`
- `registry/sources/mfds-safety-r3-business-rules.xlsx`
- `registry/dictionary/rules/ich.json`
- `registry/dictionary/rules/fda.json`
- `registry/dictionary/rules/mfds.json`
- `registry/dictionary/ich-e2br3.json`

Registry JSON must not become a runtime rules engine. It may be used to verify XPath, conformance, length, allowed values, null flavors, severity, and official identifiers while writing Rust.

## 6. Implementation phases

### Phase 0: Inventory and correction

1. Inventory the currently implemented ICH, FDA, and MFDS checks by official identifier.
2. Map each check to its official source and section.
3. Identify missing, duplicated, incorrectly scoped, or XSD-owned checks.
4. Audit the current local draft before moving it. In particular, correct known discrepancies:
   - `D.10.2.2b` permits only `a` and `10.a`.
   - Trimester units use the official case: `{Trimester}`.
   - Unit and boolean value sets must match the source exactly.

### Phase 1: Split without semantic expansion

1. Keep the existing public entry points.
2. Move current valid rules into the authority/section ownership structure.
3. Parse XML and create XPath context only once.
4. Run focused XML-library tests before adding more rules.

### Phase 2: Close observed FDA gaps

Implement and test the deterministic rules behind the failures already observed during FDA validation, including:

- Future-date restrictions for `N.1.5`, `N.2.r.4`, and `C.1.2`.
- Required FDA fields under `C.1`, `C.3`, `D.7`, `D.11`, `D.12`, `E.i`, and `G.k`.
- Official unit and boolean value constraints.
- Element length, decimal, country, and coded-value constraints.
- Telephone and fax URI formatting where it is an XML/export rule.

Every added rule must cite its official identifier in code and produce that identifier in the error.

### Phase 3: Complete deterministic authority coverage

1. Add remaining machine-verifiable ICH rejection rules by section.
2. Add remaining machine-verifiable FDA regional rejection rules by section.
3. Add machine-verifiable MFDS regional rules by section.
4. Record rules requiring unavailable external knowledge instead of approximating them.
5. Use existing local terminology/code-list facilities when an official rule requires them; never accept unknown data through a fallback.

Guidance prose, manual-review guidance, and rules requiring an unavailable external system are not converted into guessed validators.

### Phase 4: Unify enforcement

1. Ensure generated XML and submitted XML call the same XSD plus business-rule path.
2. Submission remains fail-closed.
3. FDA/MFDS production export must not bypass authority validation through an environment switch.
4. Do not add a second validation facade or authority inference helper.

### Phase 5: Regression and cleanup

1. Remove duplicated checks and obsolete suppression code.
2. Confirm authority isolation: FDA XML rejects MFDS-only elements and vice versa.
3. Recreate rich cases through the exporter for regression; do not commit files from a personal Downloads directory.
4. Confirm previously valid rich exports remain valid and each known invalid mutation fails locally with the expected official code.

## 7. Luna subagent execution

Requested worker configuration:

- Model: Luna
- Fast mode: enabled
- Reasoning: extra-high/max
- Maximum concurrency: main orchestrator plus three subagents
- If Luna is unavailable in the active runtime, do not silently substitute another model. Stop before dispatch and report the limitation.

### Wave 0: read-only inventories

Run three agents in parallel:

1. ICH inventory: official rule list, current implementation, missing/duplicate/XSD-owned classification.
2. FDA inventory: regional elements and rejection/warning rules with exact identifiers and sources.
3. MFDS inventory: regional rules with exact identifiers and sources.

Agents return findings only. They do not edit or commit.

### Main integration boundary

The main orchestrator owns shared files:

- `validation.rs`
- `validation/export_rules.rs`
- `validation/export_rules/mod.rs`
- `validation/export_rules/authority.rs`
- authority `mod.rs` files
- final integration, test runs, commits, and push

### Wave 1: disjoint builders

After the main orchestrator creates the minimum module skeleton:

1. Agent A: ICH `common`, `n`, and `c` section files.
2. Agent B: ICH `d`, `e`, `f`, `g`, and `h` section files.
3. Agent C: FDA regional section files.

Each builder edits only its assigned section files, follows existing XML error/XPath patterns, runs focused tests, and does not commit.

### Wave 2: completion and review

1. One builder implements MFDS regional section files.
2. One read-only reviewer compares implemented official identifiers with the inventories and flags omissions, duplicates, wrong scopes, and guessed rules.
3. One read-only reviewer checks for unnecessary abstractions, runtime tables, fallbacks, mocks, duplicate parsing, or divergence from existing code patterns.

The main orchestrator resolves all findings and alone edits shared integration files.

## 8. Tests and verification

Run builds and tests sequentially to avoid the prior resource contention.

For each rule file:

- Add the smallest valid/invalid test pair that proves the branch.
- Assert the exact official rule/error identifier.
- Do not create a generic fixture framework or mock authority service.

Integration coverage:

- XML is parsed once and all applicable modules run.
- XSD failures, including PORR child errors, are reported.
- ICH rules apply to FDA and MFDS.
- Only the selected regional overlay applies.
- Every previously observed FDA rejection has a local negative regression.
- A rich valid export passes local XSD and authority validation.
- The export and submission paths produce identical validation results for the same XML.

Verification order:

1. Focused tests for the edited section.
2. `cargo test -p xml --lib`.
3. Focused web-server export/submission tests.
4. Relevant workspace tests only if the focused tests pass.
5. `git diff --check` and diff review.
6. Final browser export plus external FDA Validator confirmation for the rich-case regression set.

## 9. Acceptance criteria

- Explicit Rust functions cover the agreed deterministic ICH, FDA, and MFDS rules.
- Function names and emitted errors retain official rule identifiers.
- No runtime rule tables, mocks, fallbacks, duplicate XML parsing, or hidden XSD errors remain.
- ICH/common and regional ownership is unambiguous and non-duplicated.
- Export and submission share one fail-closed validation flow.
- Known invalid XML fails locally before external submission.
- Known valid rich exports still pass locally and the FDA Validator.
- No unrelated working-tree changes are modified, staged, or committed.

## 10. Commit boundary

After all checks pass, commit only validator-related files with one focused commit, then push `dev`. Existing unrelated changes in submission, user views, RBAC scripts, temporary fuzzing output, or other worktree files remain untouched.
