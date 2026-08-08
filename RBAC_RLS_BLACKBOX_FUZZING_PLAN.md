# RBAC/RLS Black-box Fuzzing Plan

## Goal

Run stateful fuzzing against the running backend through public HTTP APIs while
following actions a real user can perform. Find and fix RBAC, RLS, multi-org,
Database/Sender scope, and pre-save authorization defects without mocks,
fallbacks, skips, or test-only product behavior.

## Non-negotiable rules

- Do not reuse the existing authz/API test harness or its helpers.
- Do not call internal services or repositories and do not insert test state
  directly into the database.
- Create organizations, users, memberships, roles, permission profiles, and
  scopes through the same public APIs used by the product.
- Authenticate every actor through the real login API.
- Preserve user action order. Change one state, save it, read it back, then
  continue.
- A blocked action is recorded as `BLOCKED`; it is never skipped or bypassed.
- Runtime fixes may not add mocks, silent fallback values, test endpoints, or
  UI-only authorization.
- Fix the shared production authorization/RLS path that owns the defect. Do not
  add symptom-specific helpers or structures that conflict with repository
  conventions.
- Subagents may investigate and edit concurrently in separate worktrees.
  Integration, build, deployment, and full regression run through one queue.

## Orchestration contract

- Run overnight only in an authorized staging/dev environment with synthetic data.
- Stop at the deadline or request budget, or immediately on approval/policy block,
  auth anomaly, rate limit, health degradation, error threshold, or suspected leak.
- Persist redacted artifacts and mark interrupted results `INCONCLUSIVE`.
- Fuzz workers may run unattended; delete, deploy, commit, and push require
  explicit human approval and are never automatic.
- Resume after a stop requires explicit approval.

- The main orchestrator remains active for the full overnight run. Intermediate
  findings produce progress updates, not a final response or a stopped run.
- The main orchestrator owns fuzz execution, failure deduplication, triage,
  subagent assignment, integration review, serialized deployment, and
  regression verification.
- Production-code investigation and fixes are delegated to subagents. The main
  orchestrator may change only the black-box runner and run artifacts directly.
- Spawn every production-code subagent with `gpt-5.6-luna`, `xhigh` reasoning,
  and Fast service tier through the enabled `multi_agent_v2` workflow.
- Each subagent receives one minimized reproducer, the expected invariant, and
  the no-mock/no-fallback/repository-consistency rules.
- Subagents work concurrently in isolated worktrees and report root cause,
  affected shared paths, changed files, and focused verification. They do not
  deploy independently.
- The main orchestrator accepts fixes through one integration queue. It reviews
  and applies one fix at a time, then performs one build and deployment before
  resuming affected fuzz scenarios.
- A failing scenario does not stop unrelated workers. A deployment pauses result
  acceptance, starts a new epoch, and resumes fuzzing after health verification.
- The run stops only when the completion criteria are met or an external or
  policy blocker prevents all meaningful progress. Ambiguous large changes are
  recorded while unaffected fuzzing continues.

## Test model

### Organization states

- Independent organizations A, B, and C
- Parent and child organizations where the product supports that relationship
- User with no organization
- User with one organization
- User belonging to two or more organizations
- Active organization switches during an authenticated session
- Membership addition and removal during an authenticated session

### Authorization states

- No role, each built-in role, and configured custom roles
- Different roles in different organizations
- Multiple roles in one organization
- Permission grant and revocation during an authenticated session
- Database scope: none, one, several, and all permitted values
- Sender scope: none, one, several, and all permitted UUIDs
- Invalid and foreign UUIDs submitted at the API trust boundary

### Operations

- List, count, search, read, create, update, and delete
- Direct access by known foreign UUID
- Organization and routing changes
- Case creation and field-by-field saves
- Import, QC, export, and submission where the actor can reach the workflow
- Attachment and nested-resource access
- Pre-save Sender, Product, Study, Reporter, Receiver, Narrative, and every
  exposed child resource

## Required invariants

1. A response never contains rows outside the actor's effective organization
   and resource scope.
2. A known foreign UUID cannot be read, changed, deleted, exported, or used as
   a parent reference.
3. Organization A privileges never become organization B privileges merely
   because the same user belongs to both.
4. Switching the active organization changes effective visibility and
   permissions immediately and does not leak cached data.
5. Revoked membership, role, Database scope, and Sender scope stop working for
   the existing session according to product policy.
6. API writes reject unauthorized scope values instead of converting them to
   `All`, ignoring them, or retaining an earlier value.
7. List, detail, count, search, export, attachment, and nested routes enforce
   the same scope.
8. Pre-save list options and direct UUID writes enforce the same scope.
9. Failure responses follow the product's established `403`/`404` policy and
   never expose foreign record contents.
10. Successful writes are read back as the same actor before the next action.

## Runner behavior

- Run outside the backend process against its public base URL.
- Use a fixed, reported seed so every failure is reproducible.
- Give each worker a unique organization/name prefix to prevent collisions.
- Record commit SHA, seed, actor, active organization, memberships, roles,
  scopes, request sequence, expected result, status, and response summary.
- Classify every action as only `PASS`, `FAIL`, or `BLOCKED`.
- Fingerprint equivalent failures and retain the shortest reproducing sequence.
- Discard results produced while a deployment is in progress and begin a new
  epoch after the server is healthy.

## Execution phases

1. **Oracle definition**
   - Read the public permission catalog and product behavior.
   - Define expected allow/deny outcomes for every generated role and scope.
   - Record policy ambiguity instead of guessing.
2. **Black-box runner**
   - Add the smallest external runner needed to execute real login and public
     API requests. Use existing runtime dependencies or the standard library.
   - Do not import existing test helpers or production internals.
3. **Control-plane setup**
   - Log in as an authorized administrator.
   - Create organizations, roles, users, memberships, and scopes through APIs.
   - Log in as each generated user and complete required first-login actions.
4. **Stateful fuzzing**
   - Generate valid user-action sequences across organization, membership,
     role, Database/Sender scope, case, and pre-save states.
   - Verify each mutation with a read before continuing.
5. **RLS attack paths**
   - Reuse discovered foreign UUIDs only through HTTP requests.
   - Probe list filters, direct routes, nested routes, exports, attachments,
     counts, and searches for cross-organization leakage.
6. **Triage and delegation**
   - Minimize each unique failure.
   - Delegate bounded root-cause investigations and fixes to subagents in
     separate worktrees.
   - Record ambiguous or policy-changing findings without modifying behavior.
7. **Single integration queue**
   - Review each fix for repository consistency and fail-closed behavior.
   - Integrate one commit, build once, deploy once, and wait for health.
   - Re-run the minimal reproducer, related scenarios, then all fixed seeds.
8. **Browser verification**
   - Replay high-risk paths and API failures through the real frontend.
   - Verify menus, direct URLs, organization switching, available options, and
     absence of foreign data in the rendered UI.
9. **Overnight loop**
   - Continue fuzzing from a clean epoch after every deployment.
   - Keep API workers parallel; keep browser replay and integration serialized.
   - Continue orchestrating after progress updates and individual fixes; do not
     end the run merely because a failure was reported or resolved.

## Fix acceptance checklist

- The failure reproduces before the change and passes after it.
- The change is inside the existing shared production path responsible for the
  decision or query scope.
- Sibling endpoints using that path are inspected and regression-tested.
- No mock, fallback, silent coercion, test-only endpoint, or UI-only guard was
  added.
- Existing naming, module boundaries, error types, and data-access patterns are
  preserved.
- The minimal reproducer, related scenario set, and fixed-seed suite pass.

## Completion criteria

- Zero reproducible cross-organization RLS leaks
- Zero successful unauthorized create, update, delete, import, QC, export, or
  submission actions
- Zero pre-save scope bypasses through list, direct UUID, or nested routes
- All fixed seeds pass after the final deployment
- High-risk browser scenarios pass
- Every remaining `BLOCKED` result and policy ambiguity is documented with a
  reproducible sequence
- Backend and frontend fixes are committed and pushed to `dev`
