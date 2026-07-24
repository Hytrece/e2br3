# RBAC Legacy Removal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove the backend and frontend legacy Permission runtime so every authorization decision uses normalized grants, the policy registry, a request snapshot, the kernel, and typed permit evidence.

**Architecture:** Reuse the existing 51-action vocabulary and add actions only for demonstrated semantic gaps. A sealed fact loader reads only authorization facts before contextual authorization; protected projections and mutations require subject or contextual permit evidence. The frontend receives generated action identifiers and request-derived `eligibleActions`, while the PDF `privileges` matrix remains the editable role contract.

**Tech Stack:** Rust 1.88, Axum, SQLx/PostgreSQL RLS, `lib-core` authorization registry/kernel, Next.js 15, TypeScript, Jest.

## Global Constraints

- The approved design is `docs/superpowers/specs/2026-07-24-rbac-legacy-removal-design.md`.
- The PDF at `/Users/hyundonghoon/Downloads/QVIS Safety Database_UI Specification_18JUN2026_Updated.pdf` remains the visible contract.
- Preserve exactly 18 PDF rows: 16 implemented and two reserved e-mail rows.
- Do not introduce Permission-to-Action translation, HTTP-method inference, a new cache, a new policy engine, a salt, or dual response fields.
- Reuse existing semantic actions; add an action only when no existing action represents the operation.
- A protected business query or mutation requires permit evidence. Before a contextual permit, only the sealed authorization-fact loader may query the database.
- `privileges` remains the role editor matrix; `eligibleActions` replaces runtime `permissions`.
- Preserve the untracked `tmp/pdfs/qvis-ui-spec.txt`.
- Keep backend and frontend cutover changes isolated until both repositories pass verification.

## Repository Map

Backend:
`/Users/hyundonghoon/projects/rust/e2br3/e2br3`

Frontend:
`/Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/.worktrees/merge-local-dev`

Backend authorization responsibilities:

- `crates/libs/lib-core/src/authorization/registry.rs`: policy data.
- `crates/libs/lib-core/src/authorization/kernel.rs`: decisions.
- `crates/libs/lib-core/src/authorization/permit.rs`: unforgeable evidence.
- `crates/libs/lib-core/src/authorization/context.rs`: typed context shapes.
- `crates/libs/lib-core/src/model/authorization/`: snapshot, normalized storage,
  fact loaders, and migrations.
- `crates/libs/lib-rest-core/src/authorization.rs`: permit-to-DB-context adapter.
- `crates/libs/lib-rest-core/src/utils/macro_utils.rs`: generated CRUD handlers.
- `crates/services/web-server/src/web/rest/`: route orchestration.

Frontend authorization responsibilities:

- `lib/auth/generated-authorization.ts`: generated action and PDF-row contract.
- `lib/contexts/AuthContext.tsx`: current snapshot projection.
- `lib/auth/routeAccess.ts` and domain access helpers: UI-only action checks.
- `lib/types/api.ts` and `lib/api/endpoints/auth.ts`: profile contract.

---

### Task 1: Add Subject Permit and Sealed Authorization Fact Loading

**Files:**
- Modify: `crates/libs/lib-core/src/authorization/permit.rs`
- Modify: `crates/libs/lib-core/src/authorization/kernel.rs`
- Modify: `crates/libs/lib-core/src/authorization/context.rs`
- Create: `crates/libs/lib-core/src/model/authorization/context_loader.rs`
- Modify: `crates/libs/lib-core/src/model/authorization/mod.rs`
- Modify: `crates/libs/lib-rest-core/src/authorization.rs`
- Test: `crates/libs/lib-core/src/authorization/kernel.rs`
- Test: `crates/services/web-server/tests/authz/authorization_legacy_gate.rs`

**Interfaces:**
- Produces: `AuthorizedSubject`, bound to action, principal, organization, and
  `PolicySnapshotVersion`.
- Produces: `AuthorizationFactLoader` methods returning only typed
  `ContextSnapshot` or `LockedMutationContext`.
- Produces: `rls_ctx_for_authorized_subject(&Ctx, &RequestAuthorizationSnapshot, &AuthorizedSubject) -> Result<Ctx>`.

- [ ] **Step 1: Write failing subject-permit evidence tests**

Add kernel tests proving that:

```rust
let action = policy_registry().subject_action("user.profile.read").unwrap();
let permit = authorize_subject(action, &snapshot).unwrap();
assert_eq!(permit.principal_id(), snapshot.principal_id());
assert_eq!(permit.organization_id(), snapshot.organization_id());
assert_eq!(permit.snapshot_version(), snapshot.version());
```

Add a REST-core structural test requiring subject permit evidence and rejecting
a permit from another principal, organization, or policy version.

- [ ] **Step 2: Run the tests and verify RED**

Run:

```bash
cargo test -p lib-core subject_permit -- --nocapture
cargo test -p web-server --test authz user_admin_rls_context_requires_authorization_permit_evidence -- --nocapture
```

Expected: FAIL because `authorize_subject` returns only
`AuthorizationDecision` and no subject permit exists.

- [ ] **Step 3: Implement the minimal subject permit**

Add `AuthorizedSubject` with read-only accessors. Change:

```rust
pub fn authorize_subject(
    action: SubjectActionId,
    snapshot: &RequestAuthorizationSnapshot,
) -> Result<AuthorizedSubject, AuthorizationDenial>
```

Use `check_eligibility` exactly once. Do not add role or permission logic.

- [ ] **Step 4: Implement the subject DB-context adapter**

`rls_ctx_for_authorized_subject` must compare request, snapshot, and permit
principal/organization/version and return only the request organization
context. It must not accept a target organization.

- [ ] **Step 5: Write failing context-loader boundary tests**

Add tests proving:

- the loader can return organization, parent Case, lifecycle, target set, and
  scope facts;
- it cannot return a business DTO;
- a mutation loader compares locked policy revisions with the request
  snapshot and returns a stale-snapshot error before writing.

- [ ] **Step 6: Implement the sealed fact loader**

Expose domain-specific methods, not a generic SQL escape hatch:

```rust
AuthorizationFactLoader::case_collection(...)
AuthorizationFactLoader::case_existing(...)
AuthorizationFactLoader::case_child(...)
AuthorizationFactLoader::presave_existing(...)
AuthorizationFactLoader::import_history_existing(...)
AuthorizationFactLoader::submission_existing(...)
```

Read loaders share the transaction used for the protected projection. Mutation
loaders lock the target and authorization revision rows and create
`LockedMutationContext`.

- [ ] **Step 7: Verify and commit**

Run:

```bash
cargo test -p lib-core authorization -- --nocapture
cargo test -p web-server --test authz authorization_isolation -- --nocapture
cargo check -p web-server
```

Commit:

```bash
git add crates/libs/lib-core/src/authorization \
  crates/libs/lib-core/src/model/authorization \
  crates/libs/lib-rest-core/src/authorization.rs \
  crates/services/web-server/tests/authz
git commit -m "refactor: require permit evidence for authorization contexts"
```

---

### Task 2: Move PDF Menu Privileges Out of the Legacy ACS Module

**Files:**
- Create: `crates/libs/lib-core/src/authorization/menu_privileges.rs`
- Modify: `crates/libs/lib-core/src/authorization/mod.rs`
- Modify: `crates/libs/lib-core/src/model/permission_profile.rs`
- Modify: `crates/libs/lib-core/src/model/authorization/assignment_repo.rs`
- Modify: `crates/services/web-server/src/web/rest/permission_profile_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/user_rest.rs`
- Test: `crates/libs/lib-core/tests/rbac_dynamic_roles.rs`
- Test: `crates/services/web-server/tests/api/role_admin/`

**Interfaces:**
- Produces: `AdminMenuPrivilege`, `PrivilegeAdapterError`,
  `normalize_current_menu_privileges`, `normalize_menu_privileges`, and
  `built_in_menu_privileges` under `lib_core::authorization`.
- Does not produce Permission arrays.

- [ ] **Step 1: Write a failing import-boundary test**

Add a structural assertion that production code imports menu privilege types
from `lib_core::authorization` and that the new module contains no
`Permission`, `Resource`, `has_permission`, or dynamic cache reference.

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p web-server --test authz authorization_legacy_gate -- --nocapture
```

Expected: FAIL because the DTO and normalizer still live under `model::acs`.

- [ ] **Step 3: Move only matrix normalization**

Move `registry_adapter.rs` behavior into `authorization/menu_privileges.rs`.
Replace Permission expansion with direct grant selection:

```rust
pub fn grant_ids_for_menu_privileges(
    privileges: &[AdminMenuPrivilege],
    allow_legacy_aliases: bool,
) -> Result<BTreeSet<GrantId>, PrivilegeAdapterError>
```

Runtime writes call it with `false`; migration calls it with `true`.

- [ ] **Step 4: Remove Permission expansion from role persistence**

`PermissionProfileBmc` and `RoleAssignmentRepository` normalize the matrix
directly to grant IDs. They must not refresh or update process-local
permissions.

- [ ] **Step 5: Verify PDF contract behavior**

Run:

```bash
cargo test -p lib-core --test authorization_contract_snapshot -- --nocapture
cargo test -p web-server --test api 'role_admin::' -- --nocapture --test-threads=1
```

Expected: all PDF row, alias rejection, Review/Lock, and role CRUD tests PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/libs/lib-core/src/authorization \
  crates/libs/lib-core/src/model/permission_profile.rs \
  crates/libs/lib-core/src/model/authorization/assignment_repo.rs \
  crates/services/web-server/src/web/rest/permission_profile_rest.rs \
  crates/services/web-server/src/web/rest/user_rest.rs \
  crates/libs/lib-core/tests crates/services/web-server/tests
git commit -m "refactor: move role privileges into authorization registry"
```

---

### Task 3: Convert Test Identity Setup to Normalized Grants

**Files:**
- Modify: `crates/services/web-server/tests/common/mod.rs`
- Modify: `crates/services/web-server/tests/api/role_admin/helpers.rs`
- Modify: `crates/services/web-server/tests/api/case_editor_contract_web.rs`
- Move: `crates/libs/lib-core/tests/rbac_dynamic_roles.rs` to
  `crates/libs/lib-core/tests/rbac_grant_profiles.rs`
- Move: `crates/libs/lib-core/tests/rbac_dynamic_roles/` to
  `crates/libs/lib-core/tests/rbac_grant_profiles/`
- Test: `crates/services/web-server/tests/authz/authorization_snapshot.rs`
- Test: `crates/services/web-server/tests/authz/authorization_storage.rs`

**Interfaces:**
- Produces: test helpers that create a `permission_profile`, reconcile
  `authorization_roles`/`role_grants`, assign it through
  `user_role_assignments`, and authenticate a new snapshot.
- Removes: `upsert_dynamic_role_permissions`, `replace_dynamic_roles`, and
  arbitrary `Vec<Permission>` test setup.

- [ ] **Step 1: Write a failing normalized-test-path assertion**

Add a structural test rejecting these names outside the legacy module:

```text
upsert_dynamic_role_permissions
replace_dynamic_roles
permissions_for_menu_privileges
Vec<Permission>
```

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p web-server --test authz production_tests_use_normalized_role_grants -- --nocapture
```

- [ ] **Step 3: Add normalized role helpers**

Provide helpers with PDF matrix input:

```rust
create_custom_role_with_privileges(&mm, organization_id, privileges)
assign_custom_role(&mm, user_id, organization_id, role_id)
authenticate_assigned_user(&mm, user)
```

The helper must use the same repository and reconciliation path as production.

- [ ] **Step 4: Replace partial child Permission fixtures**

Legacy fixtures that grant individual child permissions are not expressible in
the PDF matrix. Replace them with:

- Case Read role for all Case child reads;
- Case Edit role for Case child writes;
- no Case grant for denial cases.

Keep dedicated Review and Lock roles separate.

Rename the suite and module directory from `rbac_dynamic_roles` to
`rbac_grant_profiles`; no test name may preserve the deleted cache model.

- [ ] **Step 5: Verify normalized snapshot behavior**

Run:

```bash
cargo test -p web-server --test authz authorization_snapshot -- --nocapture
cargo test -p web-server --test authz authorization_storage -- --nocapture
cargo test -p web-server --test api case_editor_contract_web -- --nocapture --test-threads=1
```

- [ ] **Step 6: Commit**

```bash
git add crates/services/web-server/tests crates/libs/lib-core/tests
git commit -m "test: use normalized grants for authorization fixtures"
```

---

### Task 4: Convert Case and Case-Child Routes to Contextual Actions

**Files:**
- Modify: `crates/libs/lib-rest-core/src/utils/macro_utils.rs`
- Modify: `crates/services/web-server/src/web/rest/case_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/ae.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/common.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/dg.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/dh.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/direct.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/lb.rs`
- Modify: `crates/services/web-server/src/web/rest/case_editor_rest/shell.rs`
- Modify: `crates/services/web-server/src/web/rest/case_identifiers_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/case_intake_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/case_validation_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/case_workflow_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/drug_reaction_assessment_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/drug_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/message_header_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/narrative_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/narrative_sub_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/parent_history_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/patient_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/patient_sub_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/reaction_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/relatedness_assessment_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/safety_report_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/safety_report_sub_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/test_result_rest.rs`
- Test: `crates/services/web-server/tests/api/role_admin/effective_access/case_web.rs`
- Test: `crates/services/web-server/tests/api/role_admin/review_lock_web.rs`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`

**Interfaces:**
- Uses: `case.read/list/create/update/delete`, `case.child.read/update`,
  `case.review.toggle`, `case.lock.toggle`, `case.validate`,
  `case.workflow.read/transition`.
- Produces: permit-bound `Ctx` used by every protected Case repository call.

- [ ] **Step 1: Add failing no-legacy Case route checks**

Assert that Case REST files and generated macros contain no
`require_permission`, `RequirePermission`, `check_permission`,
`legacy_permission_allowed`, or Permission constant.

- [ ] **Step 2: Convert the shared CRUD macros first**

Change generated handlers to accept `AuthorizationSnapshotW`, load the
collection/existing/parent facts, authorize the appropriate action, derive the
DB context, and execute the repository call in the same transaction.

Do not pass a Permission argument into a macro. Pass an explicit action ID and
typed context factory.

- [ ] **Step 3: Convert Case shell and collection operations**

Map list/read/create/update/delete to their existing Case actions. Preserve
principal scope and platform cross-organization semantics.

- [ ] **Step 4: Convert Case child domains**

Use `case.child.read` for child list/detail and `case.child.update` for child
create/update/delete/reorder. The parent Case must be loaded and authorized;
the child fingerprint includes the concrete resource and ID for audit.

- [ ] **Step 5: Keep lifecycle actions independent**

Review, validation, workflow transition, and lock/unlock must use only their
dedicated contextual actions. A Case Edit permit cannot substitute for Review
or Lock.

- [ ] **Step 6: Run Case integration tests**

```bash
cargo test -p web-server --test api 'role_admin::effective_access::case_web' -- --nocapture --test-threads=1
cargo test -p web-server --test api 'role_admin::review_lock_web' -- --nocapture --test-threads=1
cargo test -p web-server --test api case_editor_contract_web -- --nocapture --test-threads=1
```

- [ ] **Step 7: Commit**

```bash
git add crates/libs/lib-rest-core/src/utils/macro_utils.rs \
  crates/services/web-server/src/web/rest \
  crates/services/web-server/tests/api
git commit -m "refactor: authorize case operations with contextual permits"
```

---

### Task 5: Convert INFO, Transfer, Administration, and Identity Branches

**Files:**
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/product.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/receiver.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/sender.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/shared.rs`
- Modify: `crates/services/web-server/src/web/rest/section_presave_rest/study.rs`
- Modify: `crates/services/web-server/src/web/rest/import_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/submission_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/case_export_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/cioms_export_rest/build.rs`
- Modify: `crates/services/web-server/src/web/rest/terminology_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/admin_settings_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/audit_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/organization_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/user_rest/handlers.rs`
- Modify: `crates/services/web-server/src/web/rest/user_rest/validation.rs`
- Modify: `crates/libs/lib-rest-core/src/lib.rs`
- Modify: `crates/libs/lib-web/src/middleware/mw_permission.rs`
- Test: `crates/services/web-server/tests/api/role_admin/effective_access/`
- Test: `crates/services/web-server/tests/authz/rbac_users/`

**Interfaces:**
- Uses existing INFO, Import, Submission, Terminology, Settings, Notice, Audit,
  Organization, User, and Role actions.
- Removes authorization decisions using `Ctx::is_system_admin`,
  `Ctx::is_sponsor_admin`, role labels, and `check_organization_access`.

- [ ] **Step 1: Add failing domain source guards**

Require every listed REST module to contain a snapshot/action authorization
path and reject legacy Permission or `Ctx::is_*admin` decisions.

- [ ] **Step 2: Convert INFO and transfer routes**

Use typed Presave, ImportHistory, Submission, Case, proposed import, and
resource-set contexts. Empty export/import sets still require their declared
destination context.

- [ ] **Step 3: Convert terminology, notice, settings, and audit**

Remove duplicate extractor-plus-handler checks. Each request receives exactly
one final registered action decision.

- [ ] **Step 4: Convert organization and user identity branches**

Platform and sponsor identity constraints live in registry actions and kernel
context evaluation. Validation functions may validate payload shape and
organization type but cannot decide authorization from `Ctx` role helpers.

- [ ] **Step 5: Remove permission middleware**

Delete `mw_permission.rs` after all consumers are gone. Do not replace it with
a generic role or grant middleware. Subject-only actions may use a thin typed
action adapter only when no contextual facts are required.

- [ ] **Step 6: Run effective-access matrices**

```bash
cargo test -p web-server --test api 'role_admin::' -- --nocapture --test-threads=1
cargo test -p web-server --test authz 'rbac_users::' -- --nocapture --test-threads=1
cargo test -p web-server --test authz authorization_isolation -- --nocapture
```

- [ ] **Step 7: Commit**

```bash
git add crates/services/web-server/src/web/rest \
  crates/libs/lib-rest-core/src \
  crates/libs/lib-web/src \
  crates/services/web-server/tests
git commit -m "refactor: authorize operational routes through the policy kernel"
```

---

### Task 6: Replace Profile Permissions With Eligible Actions

**Files:**
- Modify: `crates/services/web-server/src/web/rest/user_rest/dto.rs`
- Modify: `crates/services/web-server/src/web/rest/user_rest/handlers.rs`
- Modify: `crates/services/web-server/src/openapi.rs`
- Modify: `crates/libs/lib-core/src/authorization/kernel.rs`
- Modify: `crates/libs/lib-core/src/authorization/contract.rs`
- Test: `crates/services/web-server/tests/api/role_admin/helpers.rs`
- Test: `crates/services/web-server/tests/api/role_admin/effective_access/*.rs`

**Interfaces:**
- Produces: `eligible_action_ids(&RequestAuthorizationSnapshot) -> Vec<ActionId>`.
- API field: `eligibleActions: Vec<String>`.
- Removes API field: `permissions`.

- [ ] **Step 1: Write a failing profile contract test**

Require:

```rust
assert!(profile["data"]["eligibleActions"].is_array());
assert!(profile["data"].get("permissions").is_none());
```

Verify Case Read includes Case read/list/child-read eligibility but excludes
export, user, mutation, Review, and Lock actions.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p web-server --test api 'role_admin::effective_access' -- --nocapture --test-threads=1
```

- [ ] **Step 3: Implement request-derived action projection**

Iterate registry actions in canonical order and include actions for which
`check_eligibility` succeeds. Do not evaluate contextual facts, store the
result, or cache it separately.

- [ ] **Step 4: Update DTO and OpenAPI**

Replace only the field name and generated action type description. Keep
`privileges` unchanged.

- [ ] **Step 5: Verify and commit**

```bash
cargo test -p web-server --test api 'role_admin::' -- --nocapture --test-threads=1
cargo test -p lib-core --test authorization_contract_snapshot -- --nocapture
```

```bash
git add crates/services/web-server/src/web/rest/user_rest \
  crates/services/web-server/src/openapi.rs \
  crates/libs/lib-core/src/authorization \
  crates/services/web-server/tests/api/role_admin
git commit -m "feat: expose canonical eligible actions"
```

---

### Task 7: Delete the Backend Legacy Permission Runtime

**Files:**
- Delete: `crates/libs/lib-core/src/model/acs/types.rs`
- Delete: `crates/libs/lib-core/src/model/acs/catalog.rs`
- Delete: `crates/libs/lib-core/src/model/acs/builtin_roles.rs`
- Delete: `crates/libs/lib-core/src/model/acs/dynamic_roles.rs`
- Delete: `crates/libs/lib-core/src/model/acs/check.rs`
- Delete: `crates/libs/lib-core/src/model/acs/registry_adapter.rs`
- Delete: `crates/libs/lib-core/src/model/acs/tests.rs`
- Delete: `crates/libs/lib-core/src/model/acs/mod.rs`
- Modify: `crates/libs/lib-core/src/model/mod.rs`
- Modify: `crates/services/web-server/src/main.rs`
- Modify: `crates/services/web-server/src/web/rest/permission_profile_rest.rs`
- Delete: `scripts/generate_frontend_permissions.sh`
- Delete: `scripts/generate_frontend_endpoint_permissions.sh`
- Test: `crates/services/web-server/tests/authz/authorization_legacy_gate.rs`

**Interfaces:**
- Removes the complete runtime Permission representation and startup dynamic
  role refresh.
- Keeps only authorization-owned menu privilege normalization.

- [ ] **Step 1: Strengthen the failing zero-legacy test**

Scan production Rust and scripts, excluding tests and historical design
documents, for:

```text
legacy_permission_allowed
has_permission
require_permission
RequirePermission
model::acs
use .*Permission
Vec<Permission>
dynamic_role_permissions
refresh_dynamic_roles
generate_frontend_permissions
generate_frontend_endpoint_permissions
```

Exclude historical design documents and migration-only menu aliases.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p web-server --test authz authorization_legacy_gate -- --nocapture
```

- [ ] **Step 3: Delete legacy files and startup refresh**

Remove the entire ACS module and scripts. Fix imports to use
`lib_core::authorization`. Do not leave deprecated re-exports.

- [ ] **Step 4: Verify backend**

```bash
cargo fmt --all -- --check
cargo check -p web-server
cargo test -p lib-core authorization -- --nocapture
cargo test -p web-server --test authz authorization_legacy_gate -- --nocapture
```

- [ ] **Step 5: Commit**

```bash
git add -A crates scripts
git commit -m "refactor: remove legacy permission runtime"
```

---

### Task 8: Cut the Frontend Over to Generated Actions

**Files:**
- Modify: `lib/contexts/AuthContext.tsx`
- Modify: `lib/api/endpoints/auth.ts`
- Modify: `lib/types/api.ts`
- Modify: `lib/auth/routeAccess.ts`
- Modify: `lib/auth/access-rules.ts`
- Modify: `lib/auth/case-permissions.ts`
- Modify: `lib/auth/admin-permissions.ts`
- Modify: `lib/info/section-contracts.ts`
- Modify: `components/case-form/CaseEditor.tsx`
- Modify: `components/dashboard/NoticePanel.tsx`
- Modify: `components/dashboard/SystemAdminDashboard.tsx`
- Modify: `app/(protected)/admin/AdminWorkspace.tsx`
- Modify: `app/(protected)/cases/page.tsx`
- Modify: `app/(protected)/import/page.tsx`
- Modify: `app/(protected)/submission/page.tsx`
- Delete: `lib/auth/generated-permissions.ts`
- Delete: `lib/auth/generated-endpoint-permissions.ts`
- Delete: `lib/auth/permissions.ts`
- Delete: `lib/auth/PermissionGate.tsx`
- Delete: `lib/auth/endpoint-contract.ts`
- Delete: `scripts/check_generated_permissions.mjs`
- Delete: `scripts/check_generated_endpoint_permissions.mjs`
- Modify: `package.json`
- Test: `__tests__/auth/generated-authorization.test.ts`
- Test: `__tests__/auth/generated-role-privilege-cutover.test.ts`
- Test: `__tests__/auth/menu-privileges.test.ts`
- Test: `__tests__/role-privilege-rows.test.ts`
- Test: `__tests__/integration/role-privilege-effective-access.contract.test.ts`

**Interfaces:**
- `AuthContext` exposes `eligibleActions: ReadonlySet<ActionId>`.
- UI helper functions accept generated `ActionId`, never Permission values.
- `privileges` remains available for role editor projection only.

- [ ] **Step 1: Write failing frontend contract tests**

Require the profile adapter to consume `eligibleActions` and reject
`permissions`. Add a source test requiring zero imports from
`generated-permissions`, `permissions.ts`, or `PermissionGate`.

- [ ] **Step 2: Verify RED**

```bash
npm test -- __tests__/auth/generated-role-privilege-cutover.test.ts __tests__/integration/role-privilege-effective-access.contract.test.ts --runInBand
```

- [ ] **Step 3: Replace AuthContext and UI gates**

Use:

```ts
const eligibleActions = useMemo(
  () => new Set<ActionId>(user?.eligibleActions ?? []),
  [user?.eligibleActions],
);

const can = (action: ActionId) => eligibleActions.has(action);
```

Do not implement implication, any permission expansion, or resource-scope
decisions in the client.

- [ ] **Step 4: Replace domain gates with canonical actions**

Case editor uses Case/CaseChild/Review/Lock actions. Admin workspace uses
User/Organization/Settings/Role actions. INFO, Import, Submission, and Notice
use their domain actions.

- [ ] **Step 5: Delete frontend permission artifacts and scripts**

Remove the files and package scripts rather than leaving forwarding exports.

- [ ] **Step 6: Verify and commit frontend**

```bash
npm test -- __tests__/auth/generated-authorization.test.ts __tests__/auth/generated-role-privilege-cutover.test.ts __tests__/auth/menu-privileges.test.ts __tests__/role-privilege-rows.test.ts __tests__/integration/role-privilege-effective-access.contract.test.ts --runInBand
npm run build
```

```bash
git add -A
git commit -m "refactor: replace frontend permissions with actions"
```

---

### Task 9: Catalog Migration, Generated Contract, and Coordinated Verification

**Files:**
- Modify: `crates/libs/lib-core/tests/snapshots/authorization_catalog.sha256`
- Create: `db/migrations/20260724_authorization_legacy_permission_removal.sql`
- Modify: `crates/services/web-server/tests/common/mod.rs`
- Modify: `crates/services/web-server/tests/authz/authorization_storage.rs`
- Regenerate: frontend `lib/auth/generated-authorization.ts`

**Interfaces:**
- The migration advances only the reviewed current catalog hash.
- Backend and frontend expose the same schema version and catalog hash.

- [ ] **Step 1: Write failing catalog predecessor tests**

Require:

- clean bootstrap equals deployed registry;
- the current reviewed predecessor advances to the new hash;
- an unknown hash fails closed;
- schema version and generated frontend hash match.

- [ ] **Step 2: Verify RED**

```bash
cargo test -p web-server --test authz 'authorization_storage::' -- --nocapture --test-threads=1
cargo test -p lib-core --test authorization_contract_snapshot -- --nocapture
```

- [ ] **Step 3: Add the explicit migration and regenerate**

Create a one-way migration from the exact deployed predecessor hash. Do not
edit an already committed migration. Run:

```bash
./scripts/generate_frontend_authorization.sh /Users/hyundonghoon/projects/rust/e2br3/frontend/E2BR3-frontend/.worktrees/merge-local-dev
```

- [ ] **Step 4: Run fresh backend verification**

```bash
cargo fmt --all -- --check
cargo check -p web-server
cargo test -p lib-core authorization -- --nocapture
cargo test -p lib-core --test authorization_contract_snapshot -- --nocapture
cargo test -p web-server --test authz authorization_legacy_gate -- --nocapture
cargo test -p web-server --test authz 'authorization_storage::' -- --nocapture --test-threads=1
cargo test -p web-server --test api 'role_admin::' -- --nocapture --test-threads=1
```

- [ ] **Step 5: Run fresh frontend verification**

```bash
npm test -- __tests__/auth/generated-authorization.test.ts __tests__/auth/generated-role-privilege-cutover.test.ts __tests__/auth/menu-privileges.test.ts __tests__/role-privilege-rows.test.ts __tests__/integration/role-privilege-effective-access.contract.test.ts --runInBand
npm run build
```

- [ ] **Step 6: Run live roundtrip and browser E2E**

Start the tested backend and frontend builds. Verify:

- all 18 PDF rows save and reload;
- reserved rows remain disabled and absent from payloads;
- a fresh login receives updated `eligibleActions`;
- minimal Case Read/Edit/Review/Lock and Admin Read/Edit users see matching UI;
- API outcomes match the UI;
- role escalation fails;
- backend restart reconciles the new catalog without fallback.

- [ ] **Step 7: Request final code review**

Review the complete backend and frontend diffs against the design. Resolve
only demonstrated defects and rerun the affected RED/GREEN tests.

- [ ] **Step 8: Commit generated migration/contract**

Backend:

```bash
git add db crates docs
git commit -m "chore: finalize action-only authorization catalog"
```

Frontend:

```bash
git add lib/auth/generated-authorization.ts
git commit -m "chore: sync action-only authorization contract"
```

- [ ] **Step 9: Merge and push together**

Fetch both `origin/dev` branches and confirm no divergence. Merge the reviewed
work into backend and frontend `dev`, push both, then verify local HEAD equals
`origin/dev`. Do not stage or delete `tmp/pdfs/qvis-ui-spec.txt`.
