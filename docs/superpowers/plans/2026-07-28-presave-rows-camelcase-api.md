# Presave Rows and CamelCase API Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every Info presave API use Case Editor-style `data.rows`, camelCase JSON, and `id`/`sequenceNumber`/`deleted` row lifecycle semantics.

**Architecture:** Add strict REST-only rows DTOs at the presave boundary and convert them into the existing snake_case domain models inside the current authorization and transaction wrappers. Migrate backend sections in dependency order, then simplify the frontend canonical hooks to send and consume the same camelCase rows contract. Keep Rust models and PostgreSQL unchanged.

**Tech Stack:** Rust, Axum, Serde, SQLx, PostgreSQL, Next.js, TypeScript, React Query, Jest.

## Global Constraints

- Public Presave JSON uses camelCase; Rust model fields and database columns remain snake_case.
- Aggregate writes use `data.rows`; no Presave endpoint accepts `changes`.
- Repeatable rows use `id`, `sequenceNumber`, and `deleted`; `_delete` is rejected.
- Requests use strict unknown-field deserialization and removed legacy keys return HTTP 422.
- Primary archival continues through `PresaveLifecycleService`.
- Aggregate preflight occurs before mutation and the existing atomic transaction boundary is preserved.
- Do not modify or overwrite unrelated dirty worktree changes.

---

### Task 1: Shared Presave Rows Boundary

**Files:**
- Create: `crates/services/web-server/src/web/rest/section_presave_rest/rows.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/shared.rs`
- Test: `crates/services/web-server/tests/api/presave/product_web.rs`

**Interfaces:**
- Produces: `PresaveRowsRequest<R> { rows: R }`, `PresaveRowsResponse<R> { rows: R }`, and `ChildRowMeta { id, sequence_number, deleted }` REST boundary types.
- Consumes: existing `ParamsForCreate<T>`, `ParamsForUpdate<T>`, authorization wrappers, and atomic model-manager transaction behavior.

- [ ] **Step 1: Write a failing strict-envelope test**

Add a Product details HTTP test sending:

```rust
json!({
    "data": {
        "rows": {
            "product": { "medicinalProduct": "Rows Product" },
            "activeSubstances": []
        }
    }
})
```

Assert HTTP 200, and separately assert that `data.parent`, `data.active_substances`, `data.substances`, and `data.changes` each return HTTP 422.

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test -p web-server --test api api::presave::product_web -- --nocapture`

Expected: the `data.rows` request fails because the current handler expects `parent` and `active_substances`.

- [ ] **Step 3: Add the shared REST envelope**

Implement strict camelCase DTOs:

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PresaveRowsRequest<R> {
    pub rows: R,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresaveRowsResponse<R> {
    pub rows: R,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChildRowMeta {
    pub id: Option<Uuid>,
    pub sequence_number: Option<i32>,
    #[serde(default)]
    pub deleted: bool,
}
```

Keep section field DTOs explicit; do not deserialize API JSON directly into lib-core models.

- [ ] **Step 4: Run formatting and compile checks**

Run: `cargo fmt --check -- crates/services/web-server/src/web/rest/section_presave_rest.rs crates/services/web-server/src/web/rest/section_presave_rest/rows.rs crates/services/web-server/src/web/rest/section_presave_rest/shared.rs`

Run: `cargo check -p web-server`

Expected: PASS.

- [ ] **Step 5: Commit the shared boundary**

```bash
git add crates/services/web-server/src/web/rest/section_presave_rest.rs crates/services/web-server/src/web/rest/section_presave_rest/rows.rs crates/services/web-server/src/web/rest/section_presave_rest/shared.rs crates/services/web-server/tests/api/presave/product_web.rs
git commit -m "refactor: add presave rows API boundary"
```

### Task 2: Product Rows Contract

**Files:**
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/product.rs`
- Modify: `crates/services/web-server/tests/api/presave/product_web.rs`
- Modify: `crates/services/web-server/tests/api/presave/helpers.rs`
- Modify: `crates/services/web-server/src/openapi.rs`

**Interfaces:**
- Consumes: `PresaveRowsRequest<ProductPresaveRowsForUpdate>` and `PresaveRowsResponse<ProductPresaveRows>` from Task 1.
- Produces: Product rows named `product` and `activeSubstances` with Case Editor field names.

- [ ] **Step 1: Convert Product tests to the literal canonical contract**

Use this payload in create/update tests:

```json
{
  "data": {
    "rows": {
      "product": {
        "productId": "P-001",
        "medicinalProduct": "Product A",
        "senderPresaveId": "<uuid>"
      },
      "activeSubstances": [{
        "sequenceNumber": 1,
        "substanceName": "Caffeine",
        "substanceStrengthValue": 10,
        "substanceStrengthUnit": "mg",
        "deleted": false
      }]
    }
  }
}
```

Assert readback uses the same names. Add rejection assertions for `_delete`, `parent`, `active_substances`, `substances`, `sender`, and `receiver`.

- [ ] **Step 2: Run Product tests and verify RED**

Run: `cargo test -p web-server --test api api::presave::product_web -- --nocapture`

Expected: FAIL on camelCase rows deserialization/readback.

- [ ] **Step 3: Implement Product REST rows DTO conversion**

Define `ProductPresaveRow`, `ProductActiveSubstanceRow`, `ProductPresaveRows`, and update variants with `#[serde(rename_all = "camelCase", deny_unknown_fields)]`. Map `substanceStrengthValue`/`substanceStrengthUnit` to existing `strength_value`/`strength_unit` domain fields. Reject a new child with `deleted: true`; verify an existing child ID belongs to the route Product before mutation.

- [ ] **Step 4: Make Product create atomic**

Accept `data.rows` on collection POST, create the primary Product, then create supplied `activeSubstances` inside the same authorized atomic transaction. Return the full rows aggregate so the frontend no longer performs POST then PUT.

- [ ] **Step 5: Verify Product behavior**

Run: `cargo test -p web-server --test api api::presave::product_web -- --nocapture`

Run: `cargo test -p web-server --test api api::presave::delete_constraints_web -- --nocapture`

Expected: PASS.

- [ ] **Step 6: Commit Product migration**

```bash
git add crates/services/web-server/src/web/rest/section_presave_rest/product.rs crates/services/web-server/tests/api/presave/product_web.rs crates/services/web-server/tests/api/presave/helpers.rs crates/services/web-server/src/openapi.rs
git commit -m "feat: migrate product presaves to camelCase rows"
```

### Task 3: Sender and Receiver Rows Contracts

**Files:**
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/sender.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/receiver.rs`
- Test: `crates/services/web-server/tests/api/presave/sender_web.rs`
- Test: `crates/services/web-server/tests/api/presave/receiver_web.rs`
- Modify: `crates/services/web-server/tests/api/presave/helpers.rs`

**Interfaces:**
- Produces Sender rows: `sender`, `gateways`, `responsiblePersons`.
- Produces Receiver rows: `receiver`, `consignees`, `routes`; removes duplicate response-only `children`.

- [ ] **Step 1: Rewrite Sender and Receiver contract tests first**

Assert exact response keys and lifecycle semantics using `deleted`. Include rollback tests where a valid primary edit accompanies an invalid child ID.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p web-server --test api api::presave::sender_web api::presave::receiver_web -- --nocapture`

Expected: FAIL because current DTOs use `parent`, `responsible_persons`, and the duplicate Receiver `children` wrapper.

- [ ] **Step 3: Implement strict section DTOs and atomic create/update**

Convert gateway fields to `gatewayAuthority`, `senderIdentifier`, `routingIdentifier`, `cdeSenderIdentifier`, `cdrSenderIdentifier`, and `isDefaultForAuthority`. Convert responsible persons and Receiver child fields with automatic camelCase naming. Preserve current soft/hard child deletion behavior behind `deleted: true`.

- [ ] **Step 4: Verify both sections**

Run: `cargo test -p web-server --test api api::presave::sender_web api::presave::receiver_web -- --nocapture`

Expected: PASS, including atomic rollback and cross-parent rejection.

- [ ] **Step 5: Commit Sender and Receiver migration**

```bash
git add crates/services/web-server/src/web/rest/section_presave_rest/sender.rs crates/services/web-server/src/web/rest/section_presave_rest/receiver.rs crates/services/web-server/tests/api/presave/sender_web.rs crates/services/web-server/tests/api/presave/receiver_web.rs crates/services/web-server/tests/api/presave/helpers.rs
git commit -m "feat: migrate sender and receiver presaves to rows"
```

### Task 4: Reporter, Study, and Narrative Rows Contracts

**Files:**
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/reporter.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/study.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/narrative.rs`
- Test: `crates/services/web-server/tests/api/presave/reporter_web.rs`
- Test: `crates/services/web-server/tests/api/presave/study_web.rs`
- Test: `crates/services/web-server/tests/api/presave/narrative_web.rs`
- Modify: `crates/services/web-server/tests/api/presave/helpers.rs`

**Interfaces:**
- Produces Reporter row: `reporter`.
- Produces Study rows: `study`, `products`, `reporters`, `registrationNumbers`, `fdaCrossReportedInds`.
- Produces Narrative row: `narrative`.

- [ ] **Step 1: Convert section tests to rows/camelCase**

For Study, explicitly cover all four repeatable child families, `sequenceNumber`, `deleted`, cross-parent IDs, and aggregate rollback. For Reporter and Narrative, assert primary row create/read/update and strict legacy-key rejection.

- [ ] **Step 2: Verify RED**

Run: `cargo test -p web-server --test api api::presave::reporter_web api::presave::study_web api::presave::narrative_web -- --nocapture`

Expected: FAIL on the new rows envelope and camelCase names.

- [ ] **Step 3: Implement Reporter and Narrative primary-row DTOs**

Wrap their fields under `rows.reporter` and `rows.narrative`; use strict camelCase request DTOs and camelCase response serialization. Route `deleted: true` through the lifecycle service.

- [ ] **Step 4: Implement Study aggregate DTOs**

Map `registrationNumbers` to existing registration models and `fdaCrossReportedInds` to existing FDA cross-reported IND models. Apply the same preflight ownership and lifecycle rules to every child family before writing.

- [ ] **Step 5: Verify all three sections**

Run the command from Step 2 and expect PASS.

- [ ] **Step 6: Commit remaining backend sections**

```bash
git add crates/services/web-server/src/web/rest/section_presave_rest/reporter.rs crates/services/web-server/src/web/rest/section_presave_rest/study.rs crates/services/web-server/src/web/rest/section_presave_rest/narrative.rs crates/services/web-server/tests/api/presave/reporter_web.rs crates/services/web-server/tests/api/presave/study_web.rs crates/services/web-server/tests/api/presave/narrative_web.rs crates/services/web-server/tests/api/presave/helpers.rs
git commit -m "feat: migrate remaining presaves to rows"
```

### Task 5: Authorization, Import, and OpenAPI Contract Sweep

**Files:**
- Modify: `crates/services/web-server/tests/authz/authorization_test_support.rs`
- Modify: `crates/services/web-server/tests/api/scope_visibility_web.rs`
- Modify: `crates/services/web-server/tests/api/import_contract_web.rs`
- Modify: `crates/services/web-server/src/openapi.rs`
- Modify: affected request fixtures under `crates/services/web-server/tests/`

**Interfaces:**
- Consumes all canonical section DTOs from Tasks 2–4.
- Produces a repository-wide backend with no snake_case Presave HTTP fixtures.

- [ ] **Step 1: Add OpenAPI and legacy-rejection tests**

Assert schemas contain `rows`, camelCase section fields, and `deleted`, and omit `changes`, `parent`, `_delete`, and snake_case child names.

- [ ] **Step 2: Verify RED, then migrate callers and fixtures**

Run: `rg -n '"(parent|active_substances|responsible_persons|study_registration_numbers|fda_cross_reported_ind_numbers|_delete)"' crates/services/web-server/tests crates/services/web-server/src/openapi.rs`

Convert every HTTP Presave fixture to the canonical rows contract. Do not modify SQL identifiers or internal model assertions returned by this search.

- [ ] **Step 3: Verify backend integration**

Run: `cargo test -p web-server --test api -- --nocapture`

Run: `cargo test -p web-server --test authz -- --nocapture`

Expected: PASS with no Presave contract regressions.

- [ ] **Step 4: Commit backend sweep**

```bash
git add crates/services/web-server/src/openapi.rs crates/services/web-server/tests
git commit -m "test: enforce canonical presave rows contracts"
```

### Task 6: Frontend Canonical Rows Client

**Files:**
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/lib/presave/canonicalMappers.ts`
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/lib/presave/canonicalWriteMappers.ts`
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/lib/hooks/useCanonicalPresaveBase.ts`
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/lib/hooks/use*Presaves.ts`
- Test: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/__tests__/dashboard/useCanonicalPresaves.test.ts`
- Test: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/__tests__/dashboard/canonical-presave-mappers.test.ts`

**Interfaces:**
- Consumes backend `data.rows` responses.
- Produces literal `data.rows` writes without casing or lifecycle translation.

- [ ] **Step 1: Change frontend expectations first**

For every section, assert that create and save send `data: { rows: ... }`. Product must send `activeSubstances`, Case Editor field names, and `deleted`; assert resolved `sender` and `receiver` labels are absent.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --runInBand __tests__/dashboard/useCanonicalPresaves.test.ts __tests__/dashboard/canonical-presave-mappers.test.ts`

Expected: FAIL because current mappers produce flat create payloads and snake_case detail graphs.

- [ ] **Step 3: Simplify read/write mapping**

Read `response.data.rows` directly into form data. Retain only semantic mappings where Presave and Case Edit concepts genuinely differ; delete casing-only mappings, `parent` construction, `_delete` conversion, and fallback reads of removed snake_case keys.

- [ ] **Step 4: Make create a single aggregate request**

Update the shared hook so create posts the complete rows aggregate once and does not follow successful POST with a details PUT.

- [ ] **Step 5: Verify frontend contracts**

Run the Step 2 command and expect PASS.

- [ ] **Step 6: Commit frontend migration**

```bash
git add lib/presave lib/hooks __tests__/dashboard/useCanonicalPresaves.test.ts __tests__/dashboard/canonical-presave-mappers.test.ts
git commit -m "feat: use camelCase rows for info presaves"
```

### Task 7: Info UI and Presave-to-Case Round Trip

**Files:**
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/components/presave/InfoPresaveDetailRoute.tsx`
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/components/presave/ProductForm.tsx`
- Modify: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/lib/hooks/usePresaveTemplates.ts`
- Test: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/__tests__/dashboard/info-presave-detail-route.test.tsx`
- Test: `/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/__tests__/case-form/presave-import-picker-sections.test.tsx`
- Test: `crates/services/web-server/tests/api/import_contract_web.rs`

**Interfaces:**
- Consumes canonical Product `activeSubstances` rows.
- Produces an Info Product that saves, reloads, and imports into Case Edit without casing/lifecycle aliases.

- [ ] **Step 1: Add failing round-trip tests**

Create a Product with one active substance, reload the details route, apply it through the Presave import picker, and assert the Case Edit form receives `activeSubstances[0].substanceName`, `substanceStrengthValue`, and `substanceStrengthUnit` unchanged.

- [ ] **Step 2: Verify RED**

Run: `npm test -- --runInBand __tests__/dashboard/info-presave-detail-route.test.tsx __tests__/case-form/presave-import-picker-sections.test.tsx`

Expected: FAIL wherever the UI still expects `substances` or legacy response shapes.

- [ ] **Step 3: Align form state and import mapping**

Use `activeSubstances` as the Product form array and shared row type. Keep sender/receiver labels as derived display state only. Remove legacy snake_case/camelCase fallback reads from the import mapping once backend tests prove the canonical response.

- [ ] **Step 4: Verify frontend and backend round trips**

Run the Step 2 command and expect PASS.

Run: `cargo test -p web-server --test api api::import_contract_web -- --nocapture`

Expected: PASS.

- [ ] **Step 5: Commit round-trip migration**

Commit backend and frontend files separately with `test: prove presave case rows round trip`.

### Task 8: Final Verification and Legacy Removal Audit

**Files:**
- Modify only files identified by the legacy audit when the occurrence is an HTTP contract rather than a database/internal identifier.

**Interfaces:**
- Verifies the complete implementation; produces no new API behavior.

- [ ] **Step 1: Audit removed HTTP contract names**

Run targeted searches in both repositories for `changes`, `parent`, `_delete`, `active_substances`, `responsible_persons`, and other legacy Presave JSON keys. Classify every hit; retain DB/Rust internal names and remove only public-contract compatibility code.

- [ ] **Step 2: Run backend verification**

Run: `cargo fmt --check`

Run: `cargo check -p web-server`

Run: `cargo test -p web-server --test api -- --nocapture`

Run: `cargo test -p web-server --test authz -- --nocapture`

- [ ] **Step 3: Run frontend verification**

Run: `npm test -- --runInBand __tests__/dashboard __tests__/case-form/presave-import-picker-sections.test.tsx`

Run: `npx tsc --noEmit`

- [ ] **Step 4: Run the browser acceptance flow**

Create an Info Product with sender, receiver, Product ID, medicinal product, and one active substance. Save, reload, edit the substance, save again, import it into a Case Edit Drug, and verify there is no 422 and all values persist.

- [ ] **Step 5: Review diffs and commit remaining mechanical migration**

Stage only files belonging to this plan, inspect both repository diffs, and commit remaining contract cleanup independently in backend and frontend.
