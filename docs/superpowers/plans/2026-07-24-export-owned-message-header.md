# Export-Owned Message Header Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove N.1/N.2 message-header editing and persistence from CI/SD, and make export/submission preparation the sole frontend writer.

**Architecture:** The existing standalone backend `MessageHeaderBmc` and endpoint remain unchanged. Frontend CI/SD page coordinators stop owning the subresource; export/submission builds the complete outbound envelope immediately before XML generation or dispatch.

**Tech Stack:** Next.js, TypeScript, React Hook Form, Vitest, Rust, Axum, SQLx

## Global Constraints

- Do not create a second Message Header BMC.
- Do not move Message Header columns into sender or case-identification tables.
- Preserve XML import/export support for the standalone Message Header model.
- Run only focused tests during red-green iterations.

---

### Task 1: Remove CI and SD editor ownership

**Files:**
- Delete: `frontend/E2BR3-frontend/app/(protected)/[authority]/case/[id]/detail/SD/components/MessageHeaderFields.tsx`
- Modify: `frontend/E2BR3-frontend/app/(protected)/[authority]/case/[id]/detail/SD/SDPage.tsx`
- Modify: `frontend/E2BR3-frontend/app/(protected)/[authority]/case/[id]/detail/SD/hooks/useSenderPresaveImport.ts`
- Modify: `frontend/E2BR3-frontend/app/(protected)/[authority]/case/[id]/detail/SD/hooks/useReceiverPresaveImport.ts`
- Modify: `frontend/E2BR3-frontend/lib/case-save/pages/CI/save.ts`
- Modify: `frontend/E2BR3-frontend/lib/case-save/pages/SD/save.ts`
- Modify: `frontend/E2BR3-frontend/lib/case-save/contracts.ts`
- Modify: `frontend/E2BR3-frontend/lib/case-save/pathOwnership.ts`
- Test: `frontend/E2BR3-frontend/__tests__/case-save/caseIdentification.coordinator.test.ts`
- Test: `frontend/E2BR3-frontend/__tests__/case-save/sender.coordinator.test.ts`
- Test: `frontend/E2BR3-frontend/__tests__/case-save/pathOwnership.test.ts`
- Test: `frontend/E2BR3-frontend/__tests__/case-form/case-editor-alignment.test.ts`

**Interfaces:**
- Consumes: existing `CaseFormPageSave` and standalone message-header API.
- Produces: CI/SD save coordinators that never return a message-header save task.

- [ ] **Step 1: Write failing ownership tests**

Add assertions equivalent to:

```ts
expect(sourceOfSdPage).not.toContain("MessageHeaderFields");
expect(resolveCasePathOwner("messageHeader.messageNumber")).toBeNull();
expect(api.cases.upsertMessageHeader).not.toHaveBeenCalled();
```

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```bash
npm test -- __tests__/case-save/caseIdentification.coordinator.test.ts __tests__/case-save/sender.coordinator.test.ts __tests__/case-save/pathOwnership.test.ts __tests__/case-form/case-editor-alignment.test.ts
```

Expected: failures show current CI/SD message-header ownership.

- [ ] **Step 3: Remove the UI, imports, ownership rules, contracts, snapshots, and save tasks**

Delete the SD component and remove all CI/SD calls equivalent to:

```ts
api.cases.upsertMessageHeader(currentCaseId, messageHeader)
```

Sender and receiver presave import hooks must no longer call:

```ts
setValue("messageHeader.messageSenderIdentifier", value);
setValue("messageHeader.messageReceiverIdentifier", value);
setValue("messageHeader.batchReceiverIdentifier", value);
```

- [ ] **Step 4: Re-run focused tests and verify GREEN**

Run the same focused command and expect zero failures.

- [ ] **Step 5: Commit the frontend ownership change**

Commit only Task 1 frontend files with:

```bash
git commit -m "fix: remove message header from case editor"
```

### Task 2: Complete export/submission envelope generation

**Files:**
- Create: `frontend/E2BR3-frontend/app/(protected)/submission/message-header.ts`
- Modify: `frontend/E2BR3-frontend/app/(protected)/submission/page.tsx`
- Test: `frontend/E2BR3-frontend/__tests__/dashboard/submission-message-header.test.ts`

**Interfaces:**
- Consumes: existing header, case number, authority, selected sender route, selected receiver route, and a supplied timestamp.
- Produces:

```ts
export function buildOutboundMessageHeader(input: BuildOutboundMessageHeaderInput): Record<string, unknown>
```

- [ ] **Step 1: Write a failing envelope builder test**

Test that a fixed `now = new Date("2026-07-24T06:30:45.000Z")` produces:

```ts
{
  messageNumber: expect.any(String),
  batchNumber: expect.any(String),
  messageDate: "20260724063045",
  batchTransmissionDate: "20260724063045",
  messageSenderIdentifier: "SENDER",
  batchSenderIdentifier: "SENDER",
  messageReceiverIdentifier: "RECEIVER",
  batchReceiverIdentifier: "BATCH-RECEIVER"
}
```

- [ ] **Step 2: Run the new test and verify RED**

Run:

```bash
npm test -- __tests__/dashboard/submission-message-header.test.ts
```

Expected: module or export is missing.

- [ ] **Step 3: Implement the pure builder and call it from export/submission preparation**

The builder must format E2B timestamps as `YYYYMMDDHHmmss` in UTC and overwrite outbound routing/timestamp values instead of relying on CI/SD input.

- [ ] **Step 4: Re-run focused submission tests and verify GREEN**

Run:

```bash
npm test -- __tests__/dashboard/submission-message-header.test.ts __tests__/dashboard/submission-history-details.test.ts
```

Expected: zero failures.

- [ ] **Step 5: Commit the export/submission change**

```bash
git commit -m "fix: generate message header during export"
```

### Task 3: Remove SD registry certification

**Files:**
- Modify: `registry/editor-contracts/sd.json`
- Test: `registry/tools/test_editor_contracts.py`
- Test: `crates/services/web-server/tests/api/case_editor_contract_web.rs`

**Interfaces:**
- Consumes: SD editor contract rows.
- Produces: an SD contract containing only actual sender/receiver editor fields.

- [ ] **Step 1: Add or update the registry test expectation**

Assert that SD contains none of:

```text
N.1.5
N.2.r.1
N.2.r.2
N.2.r.3
```

- [ ] **Step 2: Run the focused registry test and verify RED**

Run:

```bash
python3 -m unittest registry.tools.test_editor_contracts
```

Expected: SD still contains N rows.

- [ ] **Step 3: Remove the four N rows from the SD contract**

Do not remove the standalone `registry/sections/n-message-header.json` regulatory catalog.

- [ ] **Step 4: Run registry and case-editor contract tests**

Run:

```bash
python3 -m unittest registry.tools.test_editor_contracts
cargo test -p web-server case_editor_contract -- --nocapture
```

Expected: zero failures; if the cargo filter selects no tests, run the exact affected test name reported by source inspection.

- [ ] **Step 5: Commit the registry change**

```bash
git commit -m "fix: remove message header from sd registry"
```

### Task 4: Final verification

**Files:**
- Verify all modified files from Tasks 1-3.

- [ ] **Step 1: Run frontend type checking**

```bash
npm run typecheck
```

- [ ] **Step 2: Run all affected frontend tests once**

Run the union of the focused tests from Tasks 1 and 2; do not restart already-passing unrelated suites.

- [ ] **Step 3: Run backend registry and Message Header focused tests**

```bash
python3 -m unittest registry.tools.test_editor_contracts
cargo test -p lib-core message_header -- --nocapture
```

- [ ] **Step 4: Inspect diffs and verify no Message Header writer remains in CI/SD**

```bash
rg -n "upsertMessageHeader|MessageHeaderFields|messageHeader\\." \
  lib/case-save/pages/CI \
  lib/case-save/pages/SD \
  'app/(protected)/[authority]/case/[id]/detail/SD'
```

Expected: no CI/SD editor ownership references.

