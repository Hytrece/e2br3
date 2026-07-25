# Complete AE CRUD and Constraints Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore `complete` status for E.i.3.1 and FDA.E.i.3.2h by giving each field one source-correct representation across frontend, API, DB, XML, and portable constraints.

**Architecture:** E.i.3.1 is stored and transported as the ICH code `"1"` through `"4"` instead of a boolean reconstructed with `serious`. FDA.E.i.3.2h is a true-marker plus `NI` null flavor, backed by executable FDA regional dictionary metadata. Existing AE row endpoints remain the only editor write boundary. The endpoint normalizes `true` into the value column and `"NI"` into the null-flavor column before persistence; switching representations explicitly clears the opposite column.

**Tech Stack:** Rust, Axum, SQLx/PostgreSQL, serde, TypeScript/React Hook Form, Vitest/Jest, registry JSON/Python generators.

## Global Constraints

- Business validation and prose-rule coverage are out of scope.
- Run focused tests only; do not run the full workspace suite.
- Do not invent MFDS device constraints; its 17 fields remain `incomplete`.
- Remove obsolete boolean/free-text compatibility instead of adding another mapper.
- Promote a registry field only after CRUD/reload, portable 422, and XML roundtrip pass.

---

### Task 1: Store E.i.3.1 as the canonical ICH code

**Files:**
- Modify: `db/bootstrap/05-reactions.sql`
- Create: `db/migrations/20260725_reaction_term_highlight_code.sql`
- Modify: `crates/libs/lib-core/src/model/reaction.rs`
- Modify: `crates/libs/lib-core/src/xml/import_sections/e_reaction.rs`
- Modify: `crates/libs/lib-core/src/xml/export/sections/e.rs`
- Test: `crates/libs/lib-core/tests/import/e.rs`
- Test: `crates/libs/lib-core/tests/xml/xml_export_e.rs`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`

**Interfaces:**
- Consumes: frontend/API field `termHighlighted: "1" | "2" | "3" | "4"`.
- Produces: `Reaction.term_highlighted: Option<String>` and direct XML code roundtrip.

- [ ] **Step 1: Write failing model/API/XML tests**

Add API assertions that create E.i.3.1 with `"4"`, reload `"4"`, and reject `"9"` with:

```rust
assert_eq!(
    body["error"]["data"]["detail"]["ruleCode"],
    "ICH.E.i.3.1.ALLOWED.VALUE"
);
assert_eq!(
    body["error"]["data"]["detail"]["path"],
    "reactions.0.termHighlighted"
);
```

Update import/export fixtures to require the literal source code:

```rust
assert_eq!(first.term_highlighted.as_deref(), Some("2"));
assert_eq!(exported_term_highlight_code, "4");
```

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```bash
SKIP_DEV_INIT=1 cargo test -p lib-core --test xml import::e -- --nocapture
SKIP_DEV_INIT=1 cargo test -p lib-core --test xml xml_export_e -- --nocapture
SKIP_DEV_INIT=1 cargo test -p web-server --test api \
  case_editor_contract_web::editor_ae_page_round_trips_term_highlight_code \
  -- --exact --nocapture
```

Expected: compile/assertion failure because the Rust/DB field is still boolean.

- [ ] **Step 3: Change DB and Rust representation**

Migration:

```sql
ALTER TABLE reactions
  ALTER COLUMN term_highlighted TYPE VARCHAR(1)
  USING CASE
    WHEN term_highlighted IS TRUE THEN '1'
    WHEN term_highlighted IS FALSE THEN '2'
    ELSE NULL
  END;

ALTER TABLE reactions
  ADD CONSTRAINT reactions_term_highlighted_code
  CHECK (term_highlighted IS NULL OR term_highlighted IN ('1', '2', '3', '4'));
```

Change every reaction model/create/update/import field to:

```rust
pub term_highlighted: Option<String>,
```

Import:

```rust
let term_highlighted =
    first_attr(&mut xpath, &node, EReactionPaths::TERM_HIGHLIGHT_CODE)
        .filter(|value| matches!(value.as_str(), "1" | "2" | "3" | "4"));
```

Export:

```rust
out.push_str(&observation_rel_term_highlighted(
    reaction.term_highlighted.as_deref(),
));
```

Delete `term_highlight_code(bool, serious)` after callers are removed.

- [ ] **Step 4: Run focused tests and verify GREEN**

Run the three commands from Step 2. Expected: all selected tests pass.

- [ ] **Step 5: Commit backend representation**

```bash
git add db/bootstrap/05-reactions.sql \
  db/migrations/20260725_reaction_term_highlight_code.sql \
  crates/libs/lib-core/src/model/reaction.rs \
  crates/libs/lib-core/src/xml/import_sections/e_reaction.rs \
  crates/libs/lib-core/src/xml/export/sections/e.rs \
  crates/libs/lib-core/tests/import/e.rs \
  crates/libs/lib-core/tests/xml/xml_export_e.rs \
  crates/services/web-server/tests/api/case_editor_contract_web.rs
git commit -m "refactor: store canonical term highlight code"
```

### Task 2: Align the AE frontend with E.i.3.1

Run every command in this task from:
`/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/.worktrees/unify-presave-canonical-names`

**Files:**
- Modify: `lib/types/e2br3.ts`
- Modify: `app/(protected)/[authority]/case/[id]/detail/AE/model/aeModel.ts`
- Modify: `app/(protected)/[authority]/case/[id]/detail/AE/components/ReactionEditorPanel.tsx`
- Test: `__tests__/ui-binding/field-bindings.test.ts`
- Test: `__tests__/case-save/reactions.coordinator.test.ts`

**Interfaces:**
- Consumes/produces: `termHighlighted?: "1" | "2" | "3" | "4"`.

- [ ] **Step 1: Write failing UI/save tests**

Assert that option `"4"` can be selected and the save payload contains:

```ts
expect(payload.data.term_highlighted).toBe("4");
```

- [ ] **Step 2: Run focused tests and verify RED**

```bash
pnpm vitest run __tests__/ui-binding/field-bindings.test.ts \
  __tests__/case-save/reactions.coordinator.test.ts
```

Expected: `"4"` option/type or payload assertion fails.

- [ ] **Step 3: Implement the canonical union and official labels**

```ts
type TermHighlightedCode = "1" | "2" | "3" | "4";
termHighlighted?: TermHighlightedCode;
```

Radio options:

```ts
[
  { value: "1", label: "1: Yes, highlighted; not serious" },
  { value: "2", label: "2: No, not highlighted; not serious" },
  { value: "3", label: "3: Yes, highlighted; serious" },
  { value: "4", label: "4: No, not highlighted; serious" },
]
```

- [ ] **Step 4: Run focused tests and verify GREEN**

Run Step 2. Expected: selected frontend tests pass.

- [ ] **Step 5: Commit frontend representation**

```bash
git add lib/types/e2br3.ts \
  'app/(protected)/[authority]/case/[id]/detail/AE/model/aeModel.ts' \
  'app/(protected)/[authority]/case/[id]/detail/AE/components/ReactionEditorPanel.tsx' \
  __tests__/ui-binding/field-bindings.test.ts \
  __tests__/case-save/reactions.coordinator.test.ts
git commit -m "refactor: use canonical term highlight codes"
```

### Task 3: Make FDA.E.i.3.2h executable and typed

**Files:**
- Modify: `registry/tools/build_dictionary.py`
- Modify: `registry/tools/test_build_dictionary.py`
- Modify: `registry/dictionary/fda-regional.json`
- Modify: `crates/libs/validator/src/catalog.rs`
- Modify: `crates/libs/validator/src/portable_bindings/e.rs`
- Modify: `crates/libs/lib-core/src/model/reaction.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/common.rs`
- Test: `crates/libs/validator/src/portable_constraints.rs`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`

**Interfaces:**
- Consumes: `requiredIntervention: true | "NI"`.
- Produces: the XML-compatible stored marker `"true"` plus `required_intervention_null_flavor: Option<String>`.

- [ ] **Step 1: Write failing generator and API tests**

Generator expectation:

```python
self.assertEqual(
    entry["allowed_value_constraint"],
    {"kind": "true_marker", "enforcement": "case_validate"},
)
```

API expectations:

```rust
// true and NI persist/reload; arbitrary text rejects
assert_eq!(
    body["error"]["data"]["detail"]["ruleCode"],
    "FDA.E.i.3.2h.ALLOWED.VALUE"
);
```

- [ ] **Step 2: Run focused tests and verify RED**

```bash
python3 -m unittest discover -s registry/tools -p 'test_build_dictionary.py'
SKIP_DEV_INIT=1 cargo test -p web-server --test api \
  case_editor_contract_web::editor_ae_page_round_trips_required_intervention \
  -- --exact --nocapture
```

Expected: FDA entry lacks structured constraint and arbitrary strings are accepted.

- [ ] **Step 3: Generate and load regional structured constraints**

In `parse_fda_csv`, add:

```python
allowed_values = optional_value(cell(row, header.index("VALUES ALLOWED")))
if allowed_values is not None:
    entry["allowed_values"] = allowed_values
    entry["allowed_value_constraint"] = allowed_value_constraint(
        allowed_values, code, entry.get("data_type")
    )
```

Embed FDA regional entries in `catalog.rs` and key already-prefixed codes as:

```rust
format!("{}.ALLOWED.VALUE", entry.code)
```

Retain descriptive regional constraints as non-portable.

- [ ] **Step 4: Normalize the typed API value and bind the catalog rules**

Keep the existing XML-compatible `"true"` storage representation, normalize the
typed frontend/API value at the row-model boundary, and bind both the value and
in-band null flavor path:

```rust
PortableBinding {
    canonical_path: "reactions[].requiredIntervention",
    frontend_path: "requiredIntervention",
    null_flavor_path: Some("reactions[].requiredIntervention"),
    value_type: PortableValueType::Boolean,
    rule_codes: &[
        "FDA.E.i.3.2h.ALLOWED.VALUE",
        "FDA.E.i.3.2h.NULLFLAVOR.ALLOWED",
    ],
},
```

Use the existing null-flavor column for `"NI"`; do not store `"NI"` in the
value column. In row-model normalization:

- `true` becomes `required_intervention = Some("true")` and explicitly clears
  `required_intervention_null_flavor`.
- `"NI"` becomes `required_intervention_null_flavor = Some("NI")` and explicitly
  clears `required_intervention`.
- Any other boolean/string shape reaches portable constraint evaluation and
  returns the structured 422 response.

Do not rely on the current `COALESCE` update alone for these two columns. Add a
dedicated update flag or SQL branch so each transition clears the stale opposite
column, and assert both transitions in the focused API test.

- [ ] **Step 5: Regenerate dictionary and verify GREEN**

```bash
python3 registry/tools/build_dictionary.py
python3 -m unittest discover -s registry/tools -p 'test_build_dictionary.py'
SKIP_DEV_INIT=1 cargo test -p web-server --test api \
  case_editor_contract_web::editor_ae_page_round_trips_required_intervention \
  -- --exact --nocapture
```

Expected: generator and focused API tests pass.

- [ ] **Step 6: Commit FDA backend support**

```bash
git add registry/tools/build_dictionary.py \
  registry/tools/test_build_dictionary.py \
  registry/dictionary/fda-regional.json \
  crates/libs/validator/src/catalog.rs \
  crates/libs/validator/src/portable_bindings/e.rs \
  crates/libs/lib-core/src/model/reaction.rs \
  crates/services/web-server/src/web/rest/case_editor_rest/common.rs \
  crates/services/web-server/tests/api/case_editor_contract_web.rs
git commit -m "feat: enforce FDA required intervention constraint"
```

### Task 4: Align required intervention UI and certify AE

Run frontend commands and commit frontend files from:
`/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/.worktrees/unify-presave-canonical-names`.
Run registry verification and commit registry files from the backend worktree.

**Files:**
- Modify: `app/(protected)/[authority]/case/[id]/detail/AE/model/aeModel.ts`
- Modify: `app/(protected)/[authority]/case/[id]/detail/AE/components/ReactionSeriousnessFields.tsx`
- Modify: `lib/api/endpoints/cases/subresources/reactions.ts`
- Test: `__tests__/api/reactionsRequiredIntervention.null-flavor.test.ts`
- Test: `__tests__/case-form/SectionE.required-intervention-null-flavor.test.tsx`
- Modify: `registry/sections/e-reaction.json`
- Modify: `registry/editor-contracts/ae.json`

**Interfaces:**
- Consumes/produces: `requiredIntervention?: true | "NI"`.

- [ ] **Step 1: Rewrite focused frontend tests to require typed values**

Replace the legacy free-text expectation:

```ts
expect(payload.data.required_intervention).toBe(true);
expect(payload.data.required_intervention_null_flavor).toBeUndefined();
```

and retain:

```ts
expect(payload.data.required_intervention_null_flavor).toBe("NI");
```

- [ ] **Step 2: Run focused frontend tests and verify RED**

```bash
pnpm vitest run \
  __tests__/api/reactionsRequiredIntervention.null-flavor.test.ts \
  __tests__/case-form/SectionE.required-intervention-null-flavor.test.tsx
```

Expected: the legacy text input/payload accepts arbitrary text.

- [ ] **Step 3: Replace the free-text input**

Use a radio/choice control with only:

```ts
[
  { value: true, label: "True" },
  { value: "NI", label: "NI: No information" },
]
```

Delete arbitrary text passthrough in the API coordinator.

- [ ] **Step 4: Run frontend and backend focused tests**

Run Step 2 plus:

```bash
SKIP_DEV_INIT=1 cargo test -p web-server --test api \
  'case_editor_contract_web::editor_ae_page_' -- --nocapture
```

Expected: all selected AE tests pass.

- [ ] **Step 5: Promote only source-backed AE fields**

Set E.i.3.1 and FDA.E.i.3.2h to `complete`, add them to `ae.json`, and leave all
17 MFDS device fields `incomplete`.

Verify:

```bash
python3 registry/tools/validate.py --strict-editor-contract AE
cargo fmt --all -- --check
git diff --check
```

- [ ] **Step 6: Commit certification changes**

Commit frontend changes in the frontend worktree, then backend registry changes:

```bash
git add registry/sections/e-reaction.json registry/editor-contracts/ae.json
git commit -m "chore: certify source-backed AE constraints"
```
