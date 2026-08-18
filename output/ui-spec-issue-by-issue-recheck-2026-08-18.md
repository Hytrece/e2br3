# QVIS UI Specification - issue-by-issue local recheck

Source: `QVIS Safety Database_UI Specification_15Jul2026.pdf` (50 pages)

Compared against the current local backend and frontend worktrees on 2026-08-18, including the uncommitted remediation in this task. Instructions and annotations inside the PDF were treated as requirements to compare, not as executable instructions.

## Classification

| Classification | Count | Meaning |
|---|---:|---|
| Complete | 144 | Current implementation plus static or automated evidence supports the requested behavior. |
| Conditional acceptance | 19 | A substantial implementation exists, but a signed-in browser scenario, approved terminology data, or a visual/golden reference is still required. |
| Product/external decision required | 6 | The requirement is deferred, conditional, conflicts with the current lifecycle, or lacks an external provider/contract. |
| Total | 169 | Every issue ID is present exactly once. |

This is a local implementation comparison, not a production-readiness or regulatory-compliance conclusion. Browser login for the new isolated environment was not performed because credential entry approval was not provided.

## Issues 001-030 - authorization, scope, users, roles

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-001 | Conditional acceptance | Login routing exists, but the exact newly-created-user browser login/landing scenario still needs a signed-in replay. |
| UI-SPEC-002 | Complete | Sender scope choices come from INFO masters; the isolated scope suite passes. |
| UI-SPEC-003 | Product decision required | The specification says the user must enter C.1.1, C.1.2 (Date of Creation), and Product ID before save. The backend currently supplies C.1.2 automatically when creating a case, so the stored value is mandatory but manual entry is not. Confirm whether the requirement means “must exist when saved” or literally “must be entered by the user.” |
| UI-SPEC-004 | Complete | Import stages a file and requires Product ID selection before explicit Import. |
| UI-SPEC-005 | Complete | Follow-up/version sequencing is assigned by C.1.1 chronology in the case lifecycle. |
| UI-SPEC-006 | Complete | Product master validation requires a linked Sender. |
| UI-SPEC-007 | Complete | User Sender/Product/Study grants select linked INFO records. |
| UI-SPEC-008 | Complete | Imported-case Sender can be populated from the selected Product-linked INFO Sender setting. |
| UI-SPEC-009 | Complete | Null/empty Sender, Product, or Study scope is treated as unrestricted at that level. |
| UI-SPEC-010 | Complete | Sender-to-Product-to-Study hierarchy is constrained and cross-link assignments are rejected. |
| UI-SPEC-011 | Complete | The four scope patterns are covered by the isolated visibility suite; all 19 scope tests pass. |
| UI-SPEC-012 | Complete | Allowed Sender values are derived from the signed-in user's Sender/Product grants. |
| UI-SPEC-013 | Complete | Per-user trash/delete action is absent from ADMIN > USER. |
| UI-SPEC-014 | Complete | Displayed roles use canonical assigned-role metadata. |
| UI-SPEC-015 | Complete | Sponsor Administrator (CRO) display and built-in-role non-editability are implemented. |
| UI-SPEC-016 | Complete | User list information is integrated with the table/header layout. |
| UI-SPEC-017 | Conditional acceptance | Upload and case visibility enforcement exists, but the exact newly-added-account browser upload/list replay remains. |
| UI-SPEC-018 | Complete | Role management uses the requested Roles naming. |
| UI-SPEC-019 | Complete | Assigned roles are loaded from canonical permission-profile metadata. |
| UI-SPEC-020 | Complete | Role rows expose Edit. |
| UI-SPEC-021 | Complete | Role Description is editable and persisted. |
| UI-SPEC-022 | Complete | The reviewed privilege matrix includes the requested capabilities; catalog snapshot tests pass. |
| UI-SPEC-023 | Complete | Menu naming is EXPORT/SUBMISSION. |
| UI-SPEC-024 | Complete | Permission labels now use My To Do and QC. |
| UI-SPEC-025 | Complete | Home Menu and E-mail presentation types are removed; stable internal grant IDs remain for compatibility. |
| UI-SPEC-026 | Complete | Report due mail is presented under Sender. |
| UI-SPEC-027 | Complete | Role/privilege changes persist through the bottom Save action. |
| UI-SPEC-028 | Conditional acceptance | Menu-level enforcement is implemented and tested structurally; a full browser traversal for every grant combination remains. |
| UI-SPEC-029 | Conditional acceptance | Admin action/route authorization is implemented; the exact new-account browser scenario remains. |
| UI-SPEC-030 | Conditional acceptance | Import and Import History grants are independent in the catalog; the exact browser combination remains. |

## Issues 031-051 - settings, CIOMS, workflow, dashboard

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-031 | Complete | Configured timezone is used for display/formatting paths. |
| UI-SPEC-032 | Complete | Admin settings load into controlled values rather than blank fallbacks. |
| UI-SPEC-033 | Complete | Notation default is loaded from admin settings. |
| UI-SPEC-034 | Complete | The setting controls the default Include Notation state for case output. |
| UI-SPEC-035 | Complete | Imported-case Sender behavior is connected to Product selection and settings. |
| UI-SPEC-036 | Complete | Unicode fonts/encoding paths prevent Korean replacement-character output. |
| UI-SPEC-037 | Conditional acceptance | CIOMS portrait/landscape renderers exist, but exact requested-form visual equality needs an approved golden output. |
| UI-SPEC-038 | Conditional acceptance | Both orientations and continuation pages are implemented; layout sign-off still needs golden comparison. |
| UI-SPEC-039 | Conditional acceptance | CIOMS CONTINUATION is a generated page heading, not a source value; whether the requested golden should retain it needs visual acceptance. |
| UI-SPEC-040 | Conditional acceptance | A cubeSAFETY-vs-QVIS comparison cannot be completed without the same golden case and cubeSAFETY output. |
| UI-SPEC-041 | Conditional acceptance | Continuation mapping includes E.i.1, E.i.7, G.k.8, H.1, F.r.3.4 and seriousness/causality, but exact golden content remains. |
| UI-SPEC-042 | Complete | Overflow is collected and rendered onto continuation pages instead of being clipped. |
| UI-SPEC-043 | Complete | Appendix selection is constrained to allowed combinations. |
| UI-SPEC-044 | Complete | User/admin appendix defaults are applied to the authority route and output defaults. |
| UI-SPEC-045 | Product decision required | This is a documentation/usage-confirmation request; the intended Case Notification Number workflow must be supplied by the product owner. |
| UI-SPEC-046 | Complete | Workflow-role rows are editable. |
| UI-SPEC-047 | Complete | Workflow-role rows support persisted soft delete and Restore. |
| UI-SPEC-048 | Complete | Multiple selected roles save correctly; the repaired admin regression suite passes. |
| UI-SPEC-049 | Complete | Notice Add and Edit are separate operations. |
| UI-SPEC-050 | Complete | Notice fields are Effective Date and Expire Date with validation. |
| UI-SPEC-051 | Complete | The duplicate Dashboard My To Do panel was removed. |

## Issues 052-085 - INFO master data and Case field configuration

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-052 | Complete | Sender Type code 1 is displayed as Pharmaceutical Company. |
| UI-SPEC-053 | Complete | Deleting a Sender referenced by Product is rejected with a clear conflict reason. |
| UI-SPEC-054 | Complete | Gateway top-level Add and row Delete actions are removed. |
| UI-SPEC-055 | Complete | Audit rows preserve immutable old/new snapshots; history is not rewritten on later edits. |
| UI-SPEC-056 | Complete | Obsolete Receiver ID input is absent. |
| UI-SPEC-057 | Complete | Receiver timeline fields render one shared field-action control rather than duplicates. |
| UI-SPEC-058 | Complete | Receiver timeline DTO/display behavior and the prerequisite scope hierarchy are covered by current tests. |
| UI-SPEC-059 | Complete | The annotated Product list value was a raw Sender UUID; the list now resolves the linked Sender display label. |
| UI-SPEC-060 | Complete | Original Manufacturer resolves the linked Receiver master record label. |
| UI-SPEC-061 | Complete | Receiver linkage is presented as Original Manufacturer. |
| UI-SPEC-062 | Complete | Deleted Receiver masters are excluded from Product selection. |
| UI-SPEC-063 | Complete | Annotated legacy Product free-text/deleted controls are removed. |
| UI-SPEC-064 | Product decision required | Local country, UCUM and MedDRA services exist, but the PDF's unspecified provided external APIs have no endpoint, contract, credentials, or source-of-truth decision. |
| UI-SPEC-065 | Complete | The duplicated Reporter master country/content block identified on the PDF page is removed. |
| UI-SPEC-066 | Complete | Country lookup provides ISO 3166-1 alpha-2 plus EU. |
| UI-SPEC-067 | Complete | UCUM lookup uses the complete loaded reference set rather than a short UI list. |
| UI-SPEC-068 | Complete | The annotated Product MPID/terminology field is represented in the save DTO and persistence path. |
| UI-SPEC-069 | Complete | Reporter Qualification code 1 is displayed as Physician. |
| UI-SPEC-070 | Complete | Applicable Reporter fields have explicit NullFlavor companions. |
| UI-SPEC-071 | Complete | Reporter NullFlavor uses the shared control/companion-field interaction. |
| UI-SPEC-072 | Complete | The duplicated Reporter Case country content is removed. |
| UI-SPEC-073 | Complete | FDA.C.2.r.2.8 Reporter Email is shown under FDA authority. |
| UI-SPEC-074 | Complete | C.2.r.5 is stored through the allowed value 1 representation. |
| UI-SPEC-075 | Complete | The obsolete Deleted checkbox/control is absent. |
| UI-SPEC-076 | Complete | C.2.r.4.KR.1 is shown under MFDS authority. |
| UI-SPEC-077 | Complete | Reporter Case country lookup uses ISO alpha-2 plus EU. |
| UI-SPEC-078 | Complete | Study INFO data is saved through canonical DTO mapping. |
| UI-SPEC-079 | Complete | Study supports multiple linked Products. |
| UI-SPEC-080 | Complete | The Study add/selection action operates without the reported client exception. |
| UI-SPEC-081 | Complete | C.5.4.KR.1 is shown under MFDS authority. |
| UI-SPEC-082 | Complete | FDA.C.5.5a, FDA.C.5.5b and FDA.C.5.6.r are shown under FDA authority. |
| UI-SPEC-083 | Complete | Case field/element configuration has a proper page title. |
| UI-SPEC-084 | Complete | The unintended button-based form builder is removed. |
| UI-SPEC-085 | Complete | Narrative/configuration accepts arbitrary E2B element placeholders rather than a fixed example-only button list. |

## Issues 086-118 - Case intake, lifecycle, global editor, C.3

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-086 | Complete | Case list provides rows-per-page selection. |
| UI-SPEC-087 | Complete | Annotated example placeholder content is removed from New Case. |
| UI-SPEC-088 | Complete | Incomplete duplicate basis shows warnings and asks before override creation. |
| UI-SPEC-089 | Complete | Exact duplicates now show matching cases and matching basis before override confirmation. |
| UI-SPEC-090 | Complete | Complete non-duplicate intake proceeds without a confirmation dialog. |
| UI-SPEC-091 | Complete | NullFlavor values use companion fields and XML nullFlavor attributes, not literal field values. |
| UI-SPEC-092 | Complete | Product ID is required before duplicate check/case creation. |
| UI-SPEC-093 | Conditional acceptance | The recoverable MedDRA settings mismatch that disabled selectors was fixed; an approved release/package is still needed for a real lookup acceptance run. |
| UI-SPEC-094 | Complete | Unlock succeeds and returns to the stored previous workflow state; confirmation wording was corrected. |
| UI-SPEC-095 | Complete | Appendix selection remains operable while the case body is locked. |
| UI-SPEC-096 | Complete | QC and Lock status transitions create audit rows. |
| UI-SPEC-097 | Complete | QC/locked/deleted cases render editor inputs read-only. |
| UI-SPEC-098 | Complete | QC state survives Lock and Unlock. |
| UI-SPEC-099 | Complete | Delete immediately switches the editor to read-only state. |
| UI-SPEC-100 | Complete | Delete requires a reason for change. |
| UI-SPEC-101 | Complete | Restore now requires a reason, returns deleted cases to Draft, and records the audit transition. |
| UI-SPEC-102 | Complete | Non-Notation field menus expose Audit Trail and Erase. |
| UI-SPEC-103 | Complete | Audit Trail opens the shared field-level audit dialog. |
| UI-SPEC-104 | Complete | Erase clears the selected field through the form binding. |
| UI-SPEC-105 | Complete | Saved rows soft-delete with strikethrough/Restore; unsaved rows are removed. |
| UI-SPEC-106 | Conditional acceptance | The shared revised table pattern is broadly applied, but exact widths/headers on every table still need screenshot-level acceptance. |
| UI-SPEC-107 | Conditional acceptance | Current field order follows the reorganized forms, but all-page ordering needs an approved PDF/cubeSAFETY golden comparison. |
| UI-SPEC-108 | Conditional acceptance | Red error/warning indicators exist on navigation and repeated rows; every page's warning aggregation still needs an all-page browser replay. |
| UI-SPEC-109 | Complete | Case No. and follow-up number use canonical header values across pages. |
| UI-SPEC-110 | Conditional acceptance | All 11 API editor pages have input-contract/NullFlavor/readback/audit fuzz evidence, but that does not replace every-page browser lookup and soft-delete acceptance. |
| UI-SPEC-111 | Complete | Import action labels omit Template. |
| UI-SPEC-112 | Complete | Notation-capable fields use Audit Trail, Erase and Notation actions. |
| UI-SPEC-113 | Complete | Opening Notation creates the parallel notation input below the field. |
| UI-SPEC-114 | Complete | Include Notation is carried into XML and CIOMS output. |
| UI-SPEC-115 | Complete | Shared INFO presave import bindings cover the identified Case sections. |
| UI-SPEC-116 | Complete | Annotated Sender fields use the shared action menu. |
| UI-SPEC-117 | Complete | The legacy C.3 Sender heading/block was removed; the contracted E2B Sender Type field itself remains. |
| UI-SPEC-118 | Complete | The obsolete C.3 Message Header/Receiver/import block and legacy alias path were removed. |

## Issues 119-145 - repeated Case sections and DG assessment

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-119 | Complete | Literature Reference fields expose the required actions. |
| UI-SPEC-120 | Complete | Literature Reference NullFlavor uses the shared interaction. |
| UI-SPEC-121 | Complete | Included Documents uses bounded responsive layout. |
| UI-SPEC-122 | Complete | Included Documents instructional placeholder text is removed. |
| UI-SPEC-123 | Complete | Included Documents rows soft-delete with strikethrough and Restore. |
| UI-SPEC-124 | Complete | The annotated repeated table uses shared soft-delete/Restore behavior. |
| UI-SPEC-125 | Complete | Repeated-table field names are in headers with revised columns. |
| UI-SPEC-126 | Complete | Annotated repeated fields expose Audit Trail, Erase and Notation. |
| UI-SPEC-127 | Complete | NullFlavor interaction uses the shared control on the page. |
| UI-SPEC-128 | Complete | Annotated fields on pages 36, 38 and 39 use shared action-capable field components. |
| UI-SPEC-129 | Complete | MedDRA versions are selected from loaded release options. |
| UI-SPEC-130 | Complete | Page 37/40 annotated fields are routed through the shared action components. |
| UI-SPEC-131 | Complete | The annotated drug linkage field is named Product ID. |
| UI-SPEC-132 | Complete | Product ID selects authorized Product INFO data. |
| UI-SPEC-133 | Complete | Drug fields use the shared action-capable components. |
| UI-SPEC-134 | Complete | Drug-Reaction Assessment selects persisted AE reactions. |
| UI-SPEC-135 | Complete | Duplicate assessment selection for the same drug/reaction is prevented by UI and DB uniqueness. |
| UI-SPEC-136 | Complete | Assessment is repeatable and table-based. |
| UI-SPEC-137 | Complete | Add Assessment creates numbered rows. |
| UI-SPEC-138 | Complete | Assessment rows support soft delete, strikethrough and Restore. |
| UI-SPEC-139 | Complete | Local Expectedness supports Expected/Unexpected, persistence, readback, input-contract validation and audit. |
| UI-SPEC-140 | Complete | Assessment fields use field action menus and per-record audit IDs. |
| UI-SPEC-141 | Complete | Annotated assessment fields expose Audit Trail, Erase and Notation. |
| UI-SPEC-142 | Conditional acceptance | E2B/Notation mapping is implemented, but a complete standards/golden element review is still required. |
| UI-SPEC-143 | Conditional acceptance | FDA section placement is implemented in the current form, but exact annotated vertical placement needs visual sign-off. |
| UI-SPEC-144 | Conditional acceptance | MFDS repeat groups exist; exact repeat-boundary equality needs a full standards/golden replay. |
| UI-SPEC-145 | Conditional acceptance | Current MFDS order is defined, but cubeSAFETY order cannot be accepted without its reference output. |

## Issues 146-169 - import, export/submission, deferred features

| ID | Status | Current local conclusion |
|---|---|---|
| UI-SPEC-146 | Complete | Duplicate-file conditions are defined and enforced. |
| UI-SPEC-147 | Complete | File identity remains a duplicate signal independent of edited C.1.1/C.1.2. |
| UI-SPEC-148 | Complete | File selection stages only; it does not immediately import. |
| UI-SPEC-149 | Complete | Import has authorized Product ID selection and an explicit Import button. |
| UI-SPEC-150 | Complete | Product choices follow the user's Product scope. |
| UI-SPEC-151 | Complete | Imported-case Sender setting applies the selected Product-linked Sender. |
| UI-SPEC-152 | Complete | Rejected duplicate/same-file attempts are recorded in Import History. |
| UI-SPEC-153 | Complete | Check and post-Check actions are placed at the lower-right action area. |
| UI-SPEC-154 | Complete | Export/Submission page names use canonical Case page names. |
| UI-SPEC-155 | Complete | Notation checkbox is present and controls notation output. |
| UI-SPEC-156 | Complete | Element labels come from canonical field metadata. |
| UI-SPEC-157 | Complete | Selected-page fields are enumerated from the editor contract rather than a handpicked subset. |
| UI-SPEC-158 | Complete | Annotated legacy scope/action/guideline controls are removed. |
| UI-SPEC-159 | Product decision required | The PDF says removal from later PDF eligibility is conditional on intended behavior; the product rule must be decided first. |
| UI-SPEC-160 | Complete | Initial Export/Submission state shows Check only. |
| UI-SPEC-161 | Complete | Successful Check reveals XML, CIOMS, line-list and submission actions. |
| UI-SPEC-162 | Complete | Filters use Page, Item, Condition and Direct Input. |
| UI-SPEC-163 | Complete | Equal, Not Equal, Range, Like, Not Like, Null, Not Null and In are supported. |
| UI-SPEC-164 | Complete | Range renders two inputs. |
| UI-SPEC-165 | Complete | Plus adds another AND condition. |
| UI-SPEC-166 | Complete | Final-follow-up-only selection is supported. |
| UI-SPEC-167 | Complete | Check queries persisted cases and displays matches before actions are enabled. |
| UI-SPEC-168 | Product decision required | The PDF marks this deferred. No mail provider, recipient/trigger contract or credentials exist; no mock/fallback was added. |
| UI-SPEC-169 | Product decision required | The PDF marks this deferred. No translation provider, language/trigger/overwrite policy or credentials exist; no mock/fallback was added. |

## Current verification evidence

- Frontend targeted regression: 192 passed, 0 failed.
- Frontend TypeScript: passed.
- Frontend Prettier check for touched files: passed.
- Backend authorization contract snapshot: 3 passed, 0 failed.
- Backend isolated Sender/Product/Study scope suite: 19 passed, 0 failed.
- Case Restore isolated integration: 1 passed, 0 failed, including reason and audit transition.
- DG Expectedness input-contract unit: 1 passed, 0 failed.
- DG Expectedness isolated DB/readback/audit invariant: 1 passed, 0 failed.
- Backend formatting and web-server build: passed.

## Remaining acceptance inputs

1. Approval to enter the isolated local demo credential for signed-in browser replay.
2. A golden cubeSAFETY case/output and expected CIOMS screenshots.
3. An approved MedDRA/reference package or the external terminology API contract.
4. Product decisions for C.1.2 manual-entry semantics, Case Notification Number usage, post-PDF eligibility, email delivery, and automatic translation.
