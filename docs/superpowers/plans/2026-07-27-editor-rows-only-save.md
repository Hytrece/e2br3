# Editor Rows-Only Save Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the case-editor `changes` request contract and make `rows` the only validation and persistence path for every editor page patch.

**Architecture:** `CaseEditorPagePatchRequest` becomes a strict `{ authorities?, rows? }` DTO. Every page handler validates the supplied row object and sends it directly to its existing row persistence function; all field-patch DTOs, aliases, conversion helpers, and handler branches are deleted.

**Tech Stack:** Rust 2021, Axum, Serde/serde_json, SQLx, Tokio integration tests, utoipa OpenAPI.

## Global Constraints

- `changes` is removed immediately without an adapter, feature flag, or deprecation period.
- A request containing `changes` must return HTTP 400; it must never be ignored.
- Read/projection response shapes and row-specific CRUD endpoints remain unchanged.
- D.7.2 value and null flavor persist through `rows.patientInformation` only.
- Preserve all unrelated dirty-worktree changes.

---

### Task 1: Replace the Editor Patch Contract and Production Paths

**Files:**
- Modify: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_dto.rs`
- Modify: `crates/services/web-server/src/openapi.rs`

**Interfaces:**
- Produces: `CaseEditorPagePatchRequest { authorities: Option<Vec<String>>, rows: BTreeMap<String, Value> }`
- Produces: strict Serde rejection of the removed top-level `changes` property.
- Consumes: existing `patch_json`, `get_json`, and `create_case_for_editor` integration-test helpers.

- [ ] **Step 1: Write the failing removed-contract test**

Add an HTTP contract test beside the page-patch request tests:

```rust
#[serial]
#[tokio::test]
async fn editor_page_patch_rejects_removed_changes_contract() -> Result<()> {
	let mm = init_test_mm().await?;
	let seed = seed_org_with_users(&mm, "adminpwd", "viewpwd").await?;
	let token = generate_web_token(&seed.admin.email, seed.admin.token_salt)?;
	let cookie = cookie_header(&token.to_string());
	let app = web_server::app(mm);
	let case_id = create_case_for_editor(
		&app,
		&cookie,
		"EDITOR-ROWS-ONLY",
		&["ich"],
	)
	.await?;

	let (status, body) = patch_json(
		&app,
		&cookie,
		&format!("/api/cases/{case_id}/editor/pages/DM"),
		json!({
			"changes": {
				"medicalHistoryText": {"value": "must be rejected"}
			}
		}),
	)
	.await?;

	assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
	Ok(())
}
```

- [ ] **Step 2: Run the test and verify RED**

Run:

```bash
cargo test -p web-server --test api case_editor_contract_web::editor_page_patch_rejects_removed_changes_contract -- --exact --nocapture
```

Expected: FAIL because `changes` currently deserializes and reaches the page handler instead of being rejected at HTTP 400.

- [ ] **Step 3: Make the request DTO rows-only and strict**

Change the DTO to:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CaseEditorPagePatchRequest {
	pub authorities: Option<Vec<String>>,
	#[serde(default)]
	pub rows: BTreeMap<String, Value>,
}
```

Delete `CaseEditorFieldPatch`, its custom deserializer, and imports used only by that deserializer. Remove `changes` and `CaseEditorFieldPatchDoc` from the OpenAPI component list and request schema so generated documentation exposes only `authorities` and `rows`.

- [ ] **Step 4: Continue directly with the production-path removal below**

Removing the request field intentionally makes the crate fail to compile until
all production references are removed. Do not run or commit an intermediate
state; complete Steps 5-11 as the GREEN implementation for the failing test.

- [ ] **Step 5: Delete all change-to-row production paths**

**Files:**
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/common.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/portable_save.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/direct.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/dg.rs`
- Modify: macro call sites under `crates/services/web-server/src/web/rest/case_editor_rest/*.rs` that still pass a `changes:` argument.

**Interfaces:**
- Consumes: `CaseEditorPagePatchRequest.rows`.
- Produces: every page patch handler calling its existing row validator and row persistence function exactly once.
- Preserves: `validate_direct_rows`, `validate_row_payload`, page-specific `apply_*_rows_patch`, `parse_row_model`, and validation-cache refresh functions.

- [ ] **Step 6: Use compiler failures as the removal checklist**

Run:

```bash
cargo check -p web-server --tests
```

Expected: FAIL at every production reference to the deleted `CaseEditorFieldPatch` or `request.changes`. Record those references and remove them in the following steps; do not reintroduce a compatibility type.

- [ ] **Step 7: Remove shared field-patch helpers**

Delete from `common.rs` only the helpers whose inputs are `CaseEditorFieldPatch`:

```text
patch_string_value
patch_bool_value
patch_optional_string_value
patch_optional_bool_value
patch_json_value
changes_to_object
row_payload_from_changes
row_array_payload_from_changes
```

Delete `validate_direct_changes` and its change-only unit-test helper/cases from `portable_save.rs`. Keep `validate_direct_rows` and row-payload constraint tests.

- [ ] **Step 8: Simplify shared repeatable-row patch macros**

Remove the `changes: $changes:expr` macro parameter and all synthesized-row branches. Both macro variants must use:

```rust
let row = required_row_object($section, &request.rows, $row_key)?;
validate_row_payload($section, $row_key, row, None)?;
```

Update every invocation to remove its now-unused `changes: ...` alias table. Do not change its model aliases because those still normalize canonical row fields into database model fields.

- [ ] **Step 9: Simplify DG row updates**

Replace the DG synthesized-row block with:

```rust
let row = required_row_object("DG", &request.rows, "drug")?;
validate_row_payload("DG", "drug", row, None)?;
```

Preserve active-substance, dosage, indication, reaction-assessment, cache-refresh, and response-reload calls.

- [ ] **Step 10: Simplify direct page patches, including CI**

For RP, SD, LR, SI, DM, and NR:

```rust
validate_direct_rows(page_id, &request.rows)?;
if !request.rows.is_empty() {
	apply_direct_page_rows_patch(ctx, mm, case_id, page_id, &request.rows).await?;
	refresh_editor_validation_cache(
		ctx,
		mm,
		case_id,
		requested_authorities.clone(),
	)
	.await?;
}
```

Delete `apply_direct_page_changes_patch`, `direct_sd_rows_from_changes`, and all per-page change alias arrays.

For CI, delete the `SafetyReportIdentificationForUpdate` built from `changes` and its field loop. Retain `apply_ci_rows_patch`; validate and apply `request.rows`, refresh only when non-empty, then reload the existing projection.

- [ ] **Step 11: Verify GREEN, compilation, and source removal**

Run:

```bash
cargo test -p web-server --test api case_editor_contract_web::editor_page_patch_rejects_removed_changes_contract -- --exact --nocapture
cargo check -p web-server --tests
rg -n 'request\.changes|validate_direct_changes|apply_direct_page_changes_patch|row_payload_from_changes|row_array_payload_from_changes|changes_to_object|CaseEditorFieldPatch' crates/services/web-server/src -g '*.rs'
```

Expected: the rejection test passes with HTTP 400, `cargo check` exits 0 because JSON test fixtures are runtime data, and `rg` returns no production matches.

- [ ] **Step 12: Commit the complete contract and production-path removal**

```bash
git add crates/services/web-server/src/web/rest/case_editor_dto.rs \
  crates/services/web-server/src/openapi.rs \
  crates/services/web-server/src/web/rest/case_editor_rest \
  crates/services/web-server/tests/api/case_editor_contract_web.rs
git commit -m "refactor(editor): remove changes save contract"
```

---

### Task 2: Migrate All Editor API Tests to Canonical Rows

**Files:**
- Modify: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Modify: `crates/services/web-server/tests/api/case_validation_web.rs`

**Interfaces:**
- Consumes: section-specific row names and canonical field names already accepted by each production row handler.
- Produces: no `changes` request fixtures except the one intentional HTTP 400 contract test.

- [ ] **Step 1: Convert direct-page fixtures**

Convert field envelopes such as:

```json
{"changes":{"medicalHistoryText":{"value":"Relevant history"}}}
```

to:

```json
{"rows":{"patientInformation":{"medicalHistoryText":"Relevant history"}}}
```

Use these row owners for direct pages:

```text
CI -> messageHeader or safetyReportIdentification
RP -> primarySources (array)
SD -> senderInformation or receiverInformation
LR -> literatureReferences (array)
SI -> studyInformation
DM -> patientInformation and the existing DM child-row keys
NR -> narrative
```

Represent null flavors with the existing canonical companion property, for example `medicalHistoryTextNullFlavor: "UNK"`; do not use `{ nullFlavor: ... }` envelopes.

- [ ] **Step 2: Convert repeatable-row fixtures**

For row-specific PATCH routes, wrap the canonical row under the route's row key:

```json
{"rows":{"reaction":{"reactionTerm":"Headache"}}}
{"rows":{"testResult":{"testName":"ALT"}}}
{"rows":{"drug":{"medicinalProduct":"Product"}}}
```

Preserve IDs in URL paths and keep all existing response assertions. Remove empty `"changes": {}` properties from mixed and read-only fixtures.

- [ ] **Step 3: Make D.7.2 regression coverage rows-only**

Rename `editor_dm_page_changes_round_trip_d_7_2_value_and_null_flavor` to `editor_dm_page_rows_round_trip_d_7_2_value_and_null_flavor`. First save `medicalHistoryText`, then save `medicalHistoryTextNullFlavor`, assert mutual exclusion in both responses, and GET the page to assert the stored rows equal the last response.

- [ ] **Step 4: Verify only the rejection test mentions the removed property**

Run:

```bash
rg -n '"changes"\s*:' crates/services/web-server/tests -g '*.rs'
```

Expected: exactly one match, inside `editor_page_patch_rejects_removed_changes_contract`.

- [ ] **Step 5: Run focused contract and validation tests**

Run:

```bash
cargo test -p web-server --test api case_editor_contract_web:: -- --nocapture
cargo test -p web-server --test api case_validation_web:: -- --nocapture
```

Expected: all selected tests pass with zero failures.

- [ ] **Step 6: Commit the migrated tests**

```bash
git add crates/services/web-server/tests/api/case_editor_contract_web.rs \
  crates/services/web-server/tests/api/case_validation_web.rs
git commit -m "test(editor): migrate page saves to rows contract"
```

---

### Task 3: Verify the Rows-Only Editor End to End

**Files:**
- Modify only files requiring formatting corrections introduced by Tasks 1-2.

**Interfaces:**
- Verifies the complete rows-only request contract and all affected API behavior.

- [ ] **Step 1: Format only modified Rust files**

Run `rustfmt --edition 2021` with the explicit modified file paths from Tasks 1-2. Do not run a bulk rewrite over unrelated dirty files.

- [ ] **Step 2: Check formatting and diff integrity**

Run:

```bash
git diff --check
git status --short
git diff --stat
```

Expected: no whitespace errors; unrelated pre-existing changes remain present and unmodified.

- [ ] **Step 3: Run the full API target**

Run:

```bash
cargo test -p web-server --test api -- --nocapture
```

Expected: all API tests pass with zero failures.

- [ ] **Step 4: Run the web-server test build and final source audit**

Run:

```bash
cargo check -p web-server --tests
rg -n 'request\.changes|validate_direct_changes|apply_direct_page_changes_patch|row_payload_from_changes|row_array_payload_from_changes|changes_to_object|CaseEditorFieldPatch|CaseEditorFieldPatchDoc' crates/services/web-server/src -g '*.rs'
```

Expected: check exits 0 and the source audit prints no matches.

- [ ] **Step 5: Commit any final formatting-only adjustments**

If Step 1 changed files after the previous commits:

```bash
git add crates/services/web-server/src/openapi.rs \
  crates/services/web-server/src/web/rest/case_editor_dto.rs \
  crates/services/web-server/src/web/rest/case_editor_rest \
  crates/services/web-server/tests/api/case_editor_contract_web.rs \
  crates/services/web-server/tests/api/case_validation_web.rs
git commit -m "style(editor): format rows-only save changes"
```

If formatting produced no changes, do not create an empty commit.
