# RBAC Legacy Removal Design

## Objective

Remove the legacy permission runtime completely and make the normalized
authorization registry, request snapshot, kernel, and permit the only runtime
authorization path.

The reviewed QVIS UI specification remains the visible Role & Privilege
contract. The existing 16 implemented PDF grants and two reserved e-mail
grants remain unchanged. This cutover changes how applications consume those
grants; it does not add role capabilities or broaden access.

## Root Cause

Authentication already loads the active normalized role assignment and
`role_grants` into `RequestAuthorizationSnapshot`. However, many routes discard
that canonical input for their decision and evaluate a second projection:

```text
Ctx role label
  -> process-local dynamic permission cache or built-in permission array
  -> Permission(Resource, Action)
  -> has_permission / require_permission / RequirePermission
```

The frontend consumes the same projection through
`CurrentUserProfile.permissions` and generated permission constants. Tests can
also inject arbitrary permission arrays without creating a normalized role or
assignment.

Centralizing `has_permission` behind `legacy_permission_allowed` prevents
direct callers but does not remove the second runtime. The root fix is to
remove every consumer and producer of the permission projection.

## Canonical Runtime

The only authorization flow after cutover is:

```text
authenticated request
  -> RequestAuthorizationSnapshot
       (principal, organization, identity traits, effective grants,
        scope, policy version)
  -> explicit ActionId from policy_registry
  -> authorization kernel
       (grant, identity, organization, resource, lifecycle, scope)
  -> typed permit
  -> permit-bound database context
  -> operation
```

The policy registry is the only policy data source. It owns:

- canonical action identifiers;
- required PDF grants;
- built-in identity restrictions;
- decision stage and context kind;
- organization, principal-scope, parent, target-set, and lifecycle conditions;
- read or mutation audit classification.

No code may infer an action from an HTTP method, a Rust model name, a legacy
permission constant, or a role label.

## Action Vocabulary

The cutover reuses the current semantic action vocabulary instead of creating
one action per legacy permission occurrence.

Examples:

- Case shell and collection operations use `case.read`, `case.list`,
  `case.create`, `case.update`, or `case.delete`.
- Patient, drug, reaction, narrative, and other Case-owned resources use
  `case.child.read` or `case.child.update`. The evaluated context carries the
  parent Case, target organization, scope result, and a target fingerprint.
- Review and validation use their dedicated actions.
- Lock and unlock use their dedicated actions.
- INFO, Import, Submission, User, Role, Organization, Settings, Notice, Audit,
  and Terminology use their existing domain actions.

A new action is added only when an operation has policy semantics not
represented by an existing action. Registry validation rejects duplicate IDs,
unknown grants, incompatible decision stages, and unbound protected route
actions.

## Subject and Contextual Authorization

Subject-only authorization is limited to operations that need no
target-specific facts, such as the authenticated principal's own profile or
application branding. The kernel returns a subject permit bound to the
principal, snapshot organization, and policy version. A DB context derived
from that permit is restricted to the snapshot organization and cannot target
another organization or arbitrary resource.

Every protected operation that reads or mutates scoped database data uses a
contextual permit:

- collection reads use `Collection<Resource>`;
- proposed writes use `Proposed<Proposal>`;
- existing resource access uses `Existing<Resource>`;
- Case-owned rows use `Parent<Case, CaseChild>`;
- batch operations use an explicit resource-set context.

Handlers evaluate the target organization and resource facts before issuing a
permit. A handler uses either a static action adapter or an in-handler
contextual authorization call, never both for the same operation.

A sealed context loader is the only database path allowed before a contextual
permit exists. It consumes the request snapshot and returns only the minimum
organization, parent, lifecycle, target-set, and principal-scope facts needed
to construct the typed context. It cannot return a protected business
projection or perform a mutation.

Contextual reads load facts, authorize, and read the protected projection from
one consistent transaction. Contextual mutations lock the relevant target and
authorization revision rows, reject a stale request snapshot, build
`LockedMutationContext`, authorize, and write in that same transaction.

The database context builder accepts permit evidence and verifies:

- request principal equals snapshot and permit principal;
- request organization equals snapshot organization;
- permit policy version equals snapshot policy version;
- target organization matches the evaluated context;
- platform cross-organization access is explicit;
- compliance reason, category, and signature metadata are preserved.

A protected business operation cannot construct an authorization-aware DB
context directly from a plain `Ctx`; it requires either a subject permit for
snapshot-organization work or a contextual permit for target-specific work.
The sealed context loader is restricted to authorization facts and cannot be
used as a general preauthorization repository.

## Thin Route Adapter

Static routes may use a typed `AuthorizedAction<A>` extractor. The marker
contains only a compile-time canonical action ID. The extractor:

1. loads the request snapshot;
2. resolves the action in the registry;
3. calls the kernel;
4. returns permit evidence or the existing normalized HTTP denial.

The extractor contains no role, grant, organization, or permission logic.
Body-dependent and resource-dependent operations authorize inside the handler
after parsing and loading the required context.

## Frontend Contract

`CurrentUserProfile.permissions` is removed. It is replaced by
`eligibleActions`, calculated for the current request snapshot by the backend
registry and kernel eligibility check.

`eligibleActions` is:

- not stored in the database;
- not cached separately;
- not expanded or inferred by the frontend;
- a coarse UI projection, not a resource-level permit.

The frontend uses generated `Action` constants to hide or disable navigation,
forms, and commands. Resource organization, scope, parent, and lifecycle
conditions remain server decisions, so the frontend must handle a server
denial even when an action is eligible.

The Role & Privilege matrix continues to use `privileges`, because those are
the editable PDF checkbox values. `privileges` and `eligibleActions` have
different responsibilities and are not reverse-mapped in the client.

The following frontend artifacts are removed:

- generated legacy permission constants and their generation/check scripts;
- permission sets, gates, and `can`/`canAny`/`canAll` helpers;
- handwritten endpoint-to-permission contracts;
- tests that treat client permission arrays as authoritative.

## Legacy Backend Removal

The legacy `model::acs` execution model is deleted:

- `Permission`, `Resource`, and legacy `Action` types;
- permission constants and resource permission catalogs;
- built-in permission arrays;
- dynamic role permission cache;
- `has_permission`, `has_any_permission`, and `has_all_permissions`;
- `legacy_permission_allowed`;
- `require_permission`;
- `RequirePermission`, `RequireAnyPermission`, and permission marker types;
- permission expansion and permission injection test helpers.

The PDF privilege DTO and normalization functions are not authorization
decisions. They move from `model::acs` to an authorization-owned
`menu_privileges` module before `model::acs` is removed.

Startup and permission-profile writes stop refreshing process-local dynamic
permissions. Runtime authorization continues to read normalized role
assignments and grants through `SnapshotRepository`.

## Identity and Organization Shortcuts

Removing Permission is insufficient if REST code can still authorize through
role or identity shortcuts. Production authorization code must not make an
allow/deny decision using:

- `ctx.is_system_admin()`;
- `ctx.is_sponsor_admin()`;
- a role string comparison;
- `check_organization_access`;
- `USER_CREATE` or another operational grant as administrator identity.

Built-in identity traits remain in the request snapshot and may be evaluated
only by the kernel for authorization. Non-authorizing display metadata may
show the assigned built-in identity, but it cannot decide access.

Database functions that protect normalized role assignment and role
administration continue to use normalized immutable identity kinds. They do
not interpret legacy Permission values.

## Persistence and Migration

Existing normalized data remains authoritative:

- `authorization_roles`;
- `role_grants`;
- `user_role_assignments`;
- authorization revision and catalog state.

No user or role reassignment is required. The action catalog change produces a
new canonical catalog hash and an explicit reviewed predecessor migration.
Startup reconciliation remains fail-closed.

`users.role` and `permission_profiles.privileges_json` are not deleted in this
cutover. The former remains compatibility/display data outside runtime
authorization; the latter remains the source representation of the editable
PDF matrix and is normalized into `role_grants`. Destructive schema cleanup is
a separate migration and cannot reintroduce runtime permission decisions.

Backend and frontend API changes deploy together. No compatibility response
containing both `permissions` and `eligibleActions` is introduced.

## Error Handling

- Missing authentication or snapshot: `401`.
- Unknown or unregistered action: fail closed; registry/startup tests treat
  this as a deployment defect.
- Authenticated principal missing a grant or identity condition: `403`.
- Organization, principal scope, parent, target-set, or resource constraint
  failure: `403`.
- Valid in-scope resource absent: `404`.
- Lifecycle conflict or concurrently invalidated transition: `409`.
- Unknown current matrix key or migration-only alias in a runtime write: `400`.

The existing normalized error envelope remains unchanged.

## Cutover Sequence

Work remains isolated until both repositories are ready. No intermediate
legacy/action mixed runtime is pushed to `dev`.

1. Freeze the current PDF-grant behavior in canonical action and API tests.
2. Complete the registry action vocabulary and context factories.
3. Add typed subject and contextual route adapters with permit-bound DB
   contexts.
4. Convert backend routes domain by domain, removing duplicate checks as each
   route moves.
5. Replace dynamic permission test setup with normalized roles, grants, and
   assignments.
6. Replace the profile response and frontend gates with generated actions.
7. Delete the legacy backend and frontend permission artifacts.
8. Update the catalog hash migration and generated frontend contract.
9. Run structural, contract, integration, roundtrip, browser, and restart
   verification.
10. Review the complete diff, then merge and push backend and frontend `dev`
    together.

## Verification

### Compile-Time and Structural

- Every protected route declares exactly one canonical action.
- Typed context actions cannot be called with an incompatible context kind.
- Read permits cannot authorize mutations.
- Permit evidence cannot escape its transaction lifetime where a transaction
  brand is required.
- Production Rust contains no legacy Permission type or checker.
- REST authorization contains no role-label or `Ctx::is_*admin` decision.
- Frontend production code contains no generated permission import or
  permission helper.
- The process-local dynamic permission cache and startup refresh are absent.

### Registry and Policy

- Every route action exists in the registry.
- Every implemented action requires only registered implemented grants.
- Reserved e-mail grants authorize no action.
- Case Read excludes export, user administration, mutation, review, and lock.
- Admin Edit authorizes baseline user operations but not role assignment or
  permission-profile management.
- Review and Lock remain independent.
- Every generated action identifier is stable and collision-free.

### API and Database Integration

- Each of the 16 implemented PDF grants has positive and negative endpoint
  coverage.
- Case child reads and writes enforce parent Case organization and principal
  scope.
- User role assignment and role management require the registered built-in
  identity condition.
- Cross-organization access works only with an explicit platform permit.
- Permit principal, organization, target, and policy-version mismatches fail.
- Role & Privilege save, logout/login, snapshot reload, and effective action
  projection roundtrip successfully.
- Restart reconciliation accepts only the reviewed catalog predecessor.

### Frontend and Browser

- All 18 PDF rows render in reviewed order.
- Reserved rows remain disabled and are never sent.
- Navigation and commands use generated actions.
- Minimal Case Read, Case Edit, Review, Lock, INFO, Import, Submission, Admin
  Read, and Admin Edit roles show the expected UI and receive matching API
  outcomes.
- Role escalation attempts remain denied.

## Definition of Done

- `policy_registry`, `RequestAuthorizationSnapshot`, the kernel, and typed
  permits are the only runtime authorization system.
- No backend or frontend runtime Permission representation remains.
- No process-local role-to-permission cache remains.
- No protected DB operation depends on a plain role-bearing `Ctx` for its
  authorization decision.
- Every protected route has one explicit registered action.
- The PDF matrix, frontend projection, API result, RLS scope, and audit action
  agree.
- Backend and frontend verification pass from clean builds against the same
  catalog hash.
