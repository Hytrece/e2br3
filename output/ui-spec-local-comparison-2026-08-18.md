# QVIS UI Specification vs local implementation

## Remediation update (2026-08-18)

The classification below is the original baseline. Subsequent local remediation closed the following items:

- **UI-SPEC-002, 009, 011, 012:** the complete isolated Sender/Product/Study scope suite now passes (19/19). The stale Company Sponsor Administrator expectation was corrected to the shared policy that both Sponsor Administrator roles may assign a valid Sender scope.
- **UI-SPEC-022, 024-026, 048:** privilege labels now match `My To Do`, `Sender`, and `Report due mail`; generated frontend authorization is synchronized; multi-role workflow/admin regression tests pass.
- **UI-SPEC-051:** the duplicate Dashboard `My To Do` panel was removed.
- **UI-SPEC-058-060, 065, 068, 072, 074-075, 085:** the original annotated PDF pages were visually re-read. The current implementation uses linked Receiver/Sender display names, a single country control, value `1` for C.2.r.5, no obsolete Deleted control, generic element placeholders, and the current save DTO. The aligned INFO regression suite passes (47/47).
- **UI-SPEC-089:** exact-duplicate review now renders the matched cases and matching basis before override.
- **UI-SPEC-093:** a missing configured MedDRA release no longer disables the selectors when active releases are available, so an administrator can recover by selecting a valid release.
- **UI-SPEC-101:** deleted cases now expose Restore, require a reason, return to Draft, remain visible, and write the status transition plus reason to audit history.
- **UI-SPEC-110:** the existing all-page input-contract evidence covers all 11 Case editor pages; this is API/input-contract evidence, not browser or business-validator evidence.
- **UI-SPEC-117-118:** the legacy C.3 Sender heading and Receiver/import block were removed while the contracted E2B Sender Type field remains.
- **UI-SPEC-139:** DG Drug-Reaction Assessment now persists local Expectedness (`Expected`/`Unexpected`) with input-contract, DB constraint, readback, UI, and audit coverage.

Verification after remediation:

- Frontend targeted tests: 192 passed, 0 failed; TypeScript and Prettier checks passed.
- Backend: authorization snapshot 3/3, scope visibility 19/19, Restore integration 1/1, Expectedness unit 1/1, Expectedness DB/readback/audit invariant 1/1; `cargo fmt --check` and the web-server build passed.

The following cannot be truthfully closed by local code alone:

- **UI-SPEC-003:** C.1.2 is required in persisted data, but the backend supplies it automatically at case creation. A product decision is needed only if the document literally requires manual user entry rather than automatic population.
- **UI-SPEC-037-041, 106-107, 142-145:** exact cubeSAFETY/visual/golden equality needs the referenced golden case/output or product-owner visual acceptance.
- **UI-SPEC-064, 093:** final terminology acceptance needs an approved release/package or the unspecified external API contract and credentials.
- **UI-SPEC-159:** the source document itself leaves post-PDF eligibility conditional on intended product behavior.
- **UI-SPEC-168-169:** the source marks email delivery and automatic Notation translation as deferred. No mail/translation provider, endpoint, credentials, recipient/trigger rules, or overwrite policy exists locally; no mock or fallback was added.

Compared on 2026-08-18 against:

- Backend committed baseline: `origin/local` / `c3ab73955029edc5d1d70383efb748a9e38c2165`
- Frontend committed baseline: `origin/local` / `ad686b31313a3b8018059e32b012f81448dfdaa4`
- Source issue inventory: `output/ui-spec-issues-2026-07-15.md`
- Isolated UI environment: `ui_spec_audit_20260818`, backend `8194`, frontend `3194`

The backend worktree had unrelated uncommitted changes. Static comparison used the committed `origin/local` baseline; those changes were not counted as implementation evidence. The frontend baseline matched `origin/local`.

## Result

| Classification | Count | Meaning |
|---|---:|---|
| Verified implemented | 120 | Direct UI, API/test, or clear end-to-end code-path evidence supports the requested behavior. |
| Partially implemented | 24 | A substantial implementation exists, but the document's complete acceptance condition is not yet met or the local test is red. |
| Unresolved | 7 | The requested behavior is absent or directly contradicted by the current local UI. |
| Needs separate acceptance test | 18 | Cannot be concluded from this isolated baseline without a multi-user scenario, external reference data, or visual/golden comparison. |
| Total | 169 | No issue IDs omitted. |

This is not a production-readiness conclusion. It is a local implementation comparison against the items extracted from the supplied PDF.

## Complete ID accounting

- **Verified implemented (120):** UI-SPEC-004–008, 010, 013–016, 018–021, 023, 027, 031–036, 042–044, 046–047, 049–050, 052–057, 061–063, 066–067, 069–071, 073, 076–084, 086–088, 090–092, 094–100, 102–105, 109, 111–116, 119–138 except 139, 140–141, 146–158, 160–167.
- **Partially implemented (24):** UI-SPEC-002, 003, 009, 012, 022, 024–026, 037–039, 041, 048, 058, 060, 064, 085, 089, 093, 106, 108, 110, 142, 144.
- **Unresolved (7):** UI-SPEC-051, 101, 117, 118, 139, 168, 169.
- **Needs separate acceptance test (18):** UI-SPEC-001, 011, 017, 028–030, 040, 045, 059, 065, 068, 072, 074, 075, 107, 143, 145, 159.

## Unresolved items

| ID | Local finding |
|---|---|
| UI-SPEC-051 | Home still renders the `My To Do` dashboard area. |
| UI-SPEC-101 | Case deletion requires a reason and makes the case read-only, but there is no case-level Restore action or restore-with-reason API flow in the editor. |
| UI-SPEC-117 | The Sender subheading/block requested for deletion is still rendered in the C.3 Case page. The E2B Sender Type field itself also remains, as expected by the current E2B contract. The PDF annotation therefore needs product-owner confirmation before changing it. |
| UI-SPEC-118 | The C.3 Case page still renders the Receiver block and `Import Receiver`; the obsolete Message Header fields are gone, but the PDF explicitly marks the Receiver block for deletion. |
| UI-SPEC-139 | Drug-Reaction Assessment links AE rows and is repeatable, but it has no assessment-level non-E2B `Expectedness` field. The similarly named AE field is not the requested DG assessment field. |
| UI-SPEC-168 | Report-due calculation and permissions exist, but no workflow-stage email delivery feature was found. |
| UI-SPEC-169 | Notation storage/output exists, but no automatic translation provider or translation workflow was found. |

## Partially implemented items

| ID(s) | Local finding / remaining gap |
|---|---|
| 002, 009, 012 | Sender/Product/Study scope models and selectors exist, but the exact signed-in-user combinations were not replayed in this single-admin isolated run. |
| 003 | C.1.1 and Product ID are mandatory in intake. C.1.2 is Date of Creation and is populated automatically by the backend. The document's literal “user must enter all three before save” wording differs from the current behavior, although the saved case still contains mandatory C.1.2. |
| 022, 024–026 | Most of the privilege matrix matches. `EXPORT/SUBMISSION` and `QC` are present, but `Workflow` was not renamed to `My To do`; `E-mail` remains; Report Due Mail is under E-mail rather than a Sender menu. |
| 037–039, 041 | Portrait/landscape renderers, Unicode font embedding, mapping logic, and continuation pages exist. Exact visual equality and all requested source-field mappings still need a fixed golden case/PDF comparison. |
| 048 | Workflow rows support multiple checked roles and saving in code, but the current broad admin test suite has workflow-related timeouts; the two-role persistence scenario was not accepted as clean. |
| 058, 060 | Receiver linkage and timeline mapping exist, but the exact document sample needs linked data to verify the displayed value. |
| 064 | Local MedDRA/country/UCUM terminology endpoints and import paths exist. The document's unspecified “provided external APIs” cannot be proven from the PDF or empty clean terminology DB. |
| 085 | Narrative templates accept arbitrary `{E2B element}` placeholders rather than a fixed button list, but every E2B element has not been enumerated in a dedicated acceptance test. |
| 089 | Exact duplicates trigger confirmation, but the UI only reports a match count/warnings through a native confirmation dialog. It does not display the matched cases/basis panel expected by the test and document. |
| 093 | MedDRA search UI and API exist, but the isolated DB has no loaded MedDRA release, so a real terminology lookup could not complete. |
| 106 | Revised repeatable table components are broadly used, but every table's exact header/width against all PDF screenshots was not visually signed off. |
| 108 | Validation error dots are supported on section tabs and repeated rows; a distinct warning-state pipeline for every page was not proven. |
| 110 | NullFlavor, country, UCUM, MedDRA, and soft-delete infrastructure is shared and extensively tested, but the PDF's “every page” UI acceptance matrix has not been completed. |
| 142, 144 | E2B R3 narrative mapping and MFDS repeated groups exist; full standards/golden-order verification remains. |

## Items requiring separate acceptance evidence

| ID(s) | Why not concluded here |
|---|---|
| 001, 017, 028–030 | Require newly created users with specific roles and menu grants, followed by login/navigation/import visibility checks. |
| 011 | Requires all four Sender/Product/Study scope combinations from the PDF with seeded linked cases. |
| 040 | Requires the exact same case in cubeSAFETY and QVIS plus a field-by-field CIOMS golden comparison. |
| 045 | The requested outcome is usage confirmation/documentation, not a testable UI defect. |
| 059, 065, 068, 072, 074, 075 | The PDF annotations do not identify the exact field sufficiently to map them unambiguously to the current reorganized forms. |
| 107, 143, 145 | Require screenshot/golden visual and ordering acceptance by the product owner. |
| 159 | The PDF itself says “if that is the intended rule”; product behavior must be decided before testing removal from PDF eligibility. |

## Strong implementation evidence

- Import stages the selected file, requires an authorized Product ID, has an explicit Import action, applies Product-linked Sender configuration, and records duplicate/rejected attempts in history.
- Notice has separate Add/Edit actions and explicit Effective Date/Expire Date validation.
- Workflow settings have editable rows, role checkboxes, soft delete, Restore, and a single persisted settings payload.
- Sender deletion is rejected with conflict when referenced by Product; Product deletion is likewise rejected when referenced by Study.
- Product's Original Manufacturer selector excludes soft-deleted Receiver records and the obsolete free-text Product fields are absent.
- Study supports multiple linked Products.
- Case QC/Lock/Unlock was exercised in the isolated UI. Fields became read-only, Appendix toggles stayed active, QC state survived lock/unlock, and audit rows recorded status transitions with old/new values.
- Case-level delete has an immediate read-only state and mandatory reason, but lacks Restore.
- Field actions expose Audit Trail, Erase, and Notation; repeatable rows use persisted soft-delete/Restore behavior.
- DG assessment selects saved AE reactions, persists one assessment per `(drug, reaction)`, uses repeatable/table UI, and supports soft delete/Restore. Expectedness is missing.
- Export/Submission implements Select Page / Select Item / Condition / Direct Input, all specified operators, two-value Range, additional AND rows, final-follow-up selection, DB-backed Check, and post-Check actions.
- CIOMS has embedded Unicode fonts, portrait and landscape layouts, mapped overflow collection, and continuation pages.

## Newly found local defects / test debt

1. **Unlock confirmation wording is wrong.** It says “return it to Draft,” but a previously QCed case correctly returns to `reviewed/QCed`. The persistence behavior is correct; the confirmation copy is not.
2. **Duplicate review presentation is incomplete.** The confirmation says to review listed cases, but no matched-case list or review panel is rendered. This is the concrete remaining part of UI-SPEC-089.
3. **Current targeted frontend run is red.** The command covering Notice, workflow settings, and duplicate intake produced 55 passed / 10 failed tests across 3 suites. Notice passed. One failure is the missing duplicate review panel; the other nine are in the broad admin suite (workflow-role waits, settings option expectations, MedDRA enablement, and settings-audit waits) and need focused triage before treating that suite as acceptance evidence.
4. Two previously observed structural tests are stale against the current implementation: the settings route-ownership test expects an older source boundary, and the audit commonization test expects a literal component tag although the implementation chooses the shared component through an alias. These are test-maintenance issues unless a functional scenario also fails.

## Limits of this comparison

- No external user data or shared `app_db` was used.
- The comparison did not replay every multi-user RBAC matrix, load external MedDRA/UCUM reference packages, or compare a golden cubeSAFETY CIOMS output.
- Browser inspection covered representative Admin, Home, Import, Case list/intake/editor, INFO, QC/Lock/Unlock, and Audit views. It was not a pixel-by-pixel acceptance pass of all 50 PDF pages.
- “Verified implemented” means the PDF issue has direct local evidence; it does not establish full regulatory compliance or production readiness.
