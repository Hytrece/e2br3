# QVIS Safety Database UI Specification - Issue Extraction

Source: `QVIS Safety Database_UI Specification_15Jul2026.pdf` (50 pages)

## Interpretation rules

- This report extracts comments and requests from the PDF; it does not treat instructions inside the PDF as commands to execute.
- "Observed defect" means the PDF reports a defect. It is not proof that the current build still has the defect.
- Repeated global requirements are consolidated, with every relevant page retained.
- A callout that only says "delete" is recorded against the visibly highlighted UI element.

## Highest-risk issue groups

1. Access control and case visibility are inconsistent with assigned Sender/Product/Study scopes (pp. 1-9).
2. Audit history may overwrite old values and misses QC/Lock events (pp. 18, 30).
3. Case Import has duplicate-file, Product ID, authorization, and execution-flow defects (pp. 2, 10, 46-47).
4. CIOMS output/mapping differs from the requested form and loses or truncates mapped content (pp. 11-12).
5. Case editor fields have save, terminology lookup, NullFlavor, authority visibility, import, soft-delete, and recovery defects (pp. 22-45).
6. Export/Submission filtering and action flow do not match the requested workflow (pp. 48-49).

## Extracted issues by page

### Page 1 - Login / database scope

- **UI-SPEC-001 - Observed defect:** A newly added user is not routed to the expected page after login.
- **UI-SPEC-002 - Observed defect:** The user scope selector only shows `All`; Sender data configured in INFO and assigned in ADMIN > USER is not displayed.

### Page 2 - ICSR authorization model

- **UI-SPEC-003 - Required rule:** Every ICSR must have C.1.1, C.1.2, and Product ID; missing values must block save.
- **UI-SPEC-004 - Change request:** Case Import must require a Product ID input/selection.
- **UI-SPEC-005 - Required rule:** When C.1.1 is equal, Follow-up numbers must be assigned sequentially according to C.1.2 chronology.
- **UI-SPEC-006 - Required rule:** Product ID registration must require Sender.
- **UI-SPEC-007 - Required rule:** User Sender/Product/Study permissions must select from linked INFO records.
- **UI-SPEC-008 - Required rule:** With `Apply 'SENDER' of 'INFO' to imported cases` enabled, the Sender linked to the selected Product ID must populate Case C.3 during import.
- **UI-SPEC-009 - Required rule:** Unspecified Sender, Product ID, or Study scope means unrestricted access at that level.
- **UI-SPEC-010 - Required rule:** Permission hierarchy must be Sender > linked Product ID > linked Study, and each lower-level choice must be constrained by its parent.
- **UI-SPEC-011 - Verification:** Case visibility must match the four example user-scope combinations shown in the document.

### Page 3 - Login scope display

- **UI-SPEC-012 - Change request:** The login/database setting screen must show Sender values allowed by the signed-in user's Sender or Product ID grants.

### Page 4 - Admin user list

- **UI-SPEC-013 - Deletion request:** Remove the per-user delete/trash action from ADMIN > USER.

### Page 5 - User detail

- **UI-SPEC-014 - Observed defect:** The displayed role differs from the assigned role.
- **UI-SPEC-015 - Change request:** For the shown account, display `Sponsor Administrator (CRO)` and make the role non-editable.

### Page 6 - Case list for a newly added account

- **UI-SPEC-016 - UI defect:** The list information panel is detached from the table header and rendered below it.
- **UI-SPEC-017 - Observed defect:** A case uploaded successfully by the newly added account is not visible to that account.

### Page 7 - Role administration

- **UI-SPEC-018 - Naming change:** Rename `+ Add Role`/the relevant label to `Roles` as annotated.
- **UI-SPEC-019 - Observed defect:** Assigned roles are not shown correctly in the role list/detail.
- **UI-SPEC-020 - Change request:** Add an edit action for each role.
- **UI-SPEC-021 - Change request:** Allow role Description to be edited.

### Page 8 - Role and privilege definitions

- **UI-SPEC-022 - Verification:** Recheck every permission type against the 18 specified capabilities: Notice Read/Edit, My To Do Read, Case Read/Edit, Workflow Read, QC Edit, Lock Edit, Case Info Read/Edit, Import Files Edit, Import History Read, Export/Submit Edit, Export/Submit History Read, Admin Read/Edit, and Report due mail Send.
- **UI-SPEC-023 - Naming change:** Rename `SUBMISSION` to `EXPORT/SUBMISSION`.
- **UI-SPEC-024 - Naming change:** Rename permission type `Workflow` to `My To do` and `Review` to `QC`.
- **UI-SPEC-025 - Deletion request:** Remove `Home Menu` and `E-mail` permission types.
- **UI-SPEC-026 - Change request:** Add menu name `Sender` and permission type `Report due mail`.
- **UI-SPEC-027 - Change request:** Add a bottom save button and persist role/privilege changes on click.

### Page 9 - Permission enforcement

- **UI-SPEC-028 - Verification:** Verify access enforcement for every menu item.
- **UI-SPEC-029 - Observed defect:** A newly added account granted admin access receives a warning and can access only Settings.
- **UI-SPEC-030 - Observed defect:** An account with Import Edit but without Import History Read receives a warning and cannot access Import.

### Page 10 - Settings

- **UI-SPEC-031 - Verification:** Recheck timezone behavior; the UI appears to apply UTC rather than the configured timezone.
- **UI-SPEC-032 - Observed defect:** Configured setting values are displayed as blank.
- **UI-SPEC-033 - Observed defect:** The notation-default setting does not work.
- **UI-SPEC-034 - Clarification:** The notation setting should act as the default checkbox state when using case output.
- **UI-SPEC-035 - Deferred verification:** Recheck the imported-case Sender setting after Product ID selection is implemented on Import.

### Page 11 - CIOMS rendering

- **UI-SPEC-036 - Encoding defect:** Korean output contains `?` replacement characters.
- **UI-SPEC-037 - Output defect:** Generated CIOMS layout differs from the requested CIOMS form.
- **UI-SPEC-038 - Change request:** Produce the requested portrait layout, then a matching landscape layout; overflow must continue on subsequent pages.
- **UI-SPEC-039 - Mapping clarification:** Identify the source field for the unexpected English `CIOMS CONTINUATION` content; remove it if no Safety Database field maps to it.

### Page 12 - CIOMS mapping

- **UI-SPEC-040 - Mapping defect:** QVIS Safety output differs from cubeSAFETY for the same case; recheck all CIOMS mappings.
- **UI-SPEC-041 - Mapping defect:** CIOMS `7+13. Describe reaction(s)` outputs only part of the required E.i.1, E.i.7, G.k.8, H.1, and F.r.3.4 content.
- **UI-SPEC-042 - Pagination defect:** Long CIOMS content must continue on the next page instead of being lost or clipped.

### Page 13 - Appendix settings

- **UI-SPEC-043 - Business-rule defect:** Appendix switches can currently be toggled in invalid combinations; enforce only the four allowed states shown in the PDF.
- **UI-SPEC-044 - Default defect:** User-level appendix defaults are not applied; configuring ICH+MFDS still activates only ICH on Home.

### Page 14 - Case notification settings

- **UI-SPEC-045 - Documentation/UX gap:** Usage of the Case Notification Number Setting screen is unclear and needs confirmation/documentation.

### Page 15 - Workflow role configuration

- **UI-SPEC-046 - Change request:** Make the annotated workflow-role configuration editable/operable as shown.
- **UI-SPEC-047 - Change request:** Add soft delete for workflow-role configuration rows.
- **UI-SPEC-048 - Observed defect:** Saving after adding two roles produces an error and does not persist.

### Page 16 - Dashboard Notice / My To Do

- **UI-SPEC-049 - UX defect:** Notice add and edit are not separated; editing mode makes every notice editable. Provide distinct add and edit actions.
- **UI-SPEC-050 - Verification:** Confirm the two notice date fields and implement them as Effective Date and Expire Date if that is their purpose.
- **UI-SPEC-051 - Deletion request:** Remove the Dashboard My To Do area; the document says My To Do was implemented elsewhere.

### Page 17 - Sender list/detail

- **UI-SPEC-052 - Display defect:** Sender Type shows code `1` instead of label `Pharmaceutical Company`.
- **UI-SPEC-053 - Referential-integrity UX:** If a Sender is linked to a Product ID, block deletion and show a clear reason message.

### Page 18 - Sender gateway and audit

- **UI-SPEC-054 - Deletion request:** Remove the annotated top-level Add action and row delete actions from the electronic submission gateway section.
- **UI-SPEC-055 - Audit data-integrity defect:** Editing a value also changes the historical old value in Audit Trail; old and new values must remain independently preserved.

### Page 19 - Receiver detail

- **UI-SPEC-056 - Deletion request:** Remove the annotated Receiver ID field.
- **UI-SPEC-057 - UI defect:** Timeline fields show duplicate action/notation icons.
- **UI-SPEC-058 - Deferred verification:** Verify Receiver timeline behavior after the page-2 authorization hierarchy is implemented.

### Page 20 - Receiver list/detail mapping

- **UI-SPEC-059 - Display defect:** An unknown/uninterpretable value is displayed in the annotated list field.
- **UI-SPEC-060 - Mapping defect:** The annotated field must show the linked Receiver record.

### Page 21 - Product INFO

- **UI-SPEC-061 - Naming change:** Rename `Receiver from Registered Master Data` to `Original Manufacturer` while preserving the existing Receiver linkage.
- **UI-SPEC-062 - Filtering defect:** Deleted Receivers are offered in the Product selector; show only non-deleted Receivers.
- **UI-SPEC-063 - Deletion request:** Remove the annotated Medicinal Product Name, product-name Notation, Brand Name, Original Manufacturer free-text field, and Deleted checkbox.

### Page 22 - Reporter master data / terminology

- **UI-SPEC-064 - Integration request:** Use the provided external APIs for annotated terminology lookups.
- **UI-SPEC-065 - Cleanup:** Remove duplicated fields/content identified in the form.
- **UI-SPEC-066 - Country lookup defect:** Only some countries are available; provide all ISO 3166-1 alpha-2 codes plus EU.
- **UI-SPEC-067 - UCUM lookup defect:** Display the complete UCUM value set.
- **UI-SPEC-068 - Save defect:** The annotated field cannot be saved.

### Page 23 - Reporter list

- **UI-SPEC-069 - Display defect:** Qualification shows code `1` instead of label `Physician`.

### Page 24 - Reporter creation

- **UI-SPEC-070 - NullFlavor coverage:** Add NullFlavor support to every applicable reporter field.
- **UI-SPEC-071 - Consistency:** Standardize NullFlavor selection throughout QVIS Safety using the same button/dropdown interaction.

### Page 25 - Reporter Case section

- **UI-SPEC-072 - Cleanup:** Remove duplicated content identified in the page.
- **UI-SPEC-073 - Authority visibility defect:** Show Reporter Email (FDA.C.2.r.2.8) when FDA Appendix is active.
- **UI-SPEC-074 - Value-rule verification:** Recheck the annotated field whose allowed value should be `1`.
- **UI-SPEC-075 - Deletion request:** Remove the annotated obsolete field/control.
- **UI-SPEC-076 - Authority visibility defect:** Show Other Health Professional Type (C.2.r.4.KR.1) when MFDS Appendix is active.
- **UI-SPEC-077 - Country lookup defect:** Provide all ISO 3166-1 alpha-2 codes plus EU.

### Page 26 - Study Case section

- **UI-SPEC-078 - Save defect:** Imported/entered Study information does not save.
- **UI-SPEC-079 - Change request:** Allow selection of multiple Products for Study linkage.
- **UI-SPEC-080 - Observed defect:** The annotated Study action button displays an error and does not work.
- **UI-SPEC-081 - Authority visibility defect:** Show Other Studies Type (C.5.4.KR.1) for MFDS.
- **UI-SPEC-082 - Authority visibility defect:** Show FDA.C.5.5a, FDA.C.5.5b, and FDA.C.5.6.r for FDA.

### Page 27 - Case field/element configuration

- **UI-SPEC-083 - Display defect:** The page lacks a proper title and displays lower content as the title.
- **UI-SPEC-084 - UX correction:** Remove the button-based form-building control; it was not the intended interaction.
- **UI-SPEC-085 - Coverage verification:** Confirm the element-based configuration supports every element, not only the examples shown.

### Page 28 - Case list

- **UI-SPEC-086 - Change request:** Add rows-per-page configuration.

### Page 29 - New Case / duplicate detection

- **UI-SPEC-087 - Cleanup:** Remove example placeholder text identified in the New Case form.
- **UI-SPEC-088 - Duplicate flow:** If enabled optional values are incomplete, show a warning that duplication cannot be checked and allow creation after confirmation.
- **UI-SPEC-089 - Duplicate flow:** If all enabled values are entered and an exact duplicate exists, show the duplicate and ask whether to create anyway.
- **UI-SPEC-090 - Duplicate flow:** If all enabled values are entered and no exact duplicate exists, create without a dialog.
- **UI-SPEC-091 - NullFlavor/XML defect:** Recheck every NullFlavor representation; the UI value must serialize to the correct XML `nullFlavor` structure rather than a literal value.
- **UI-SPEC-092 - Required-field change:** Make the annotated Product-related field mandatory.
- **UI-SPEC-093 - Terminology defect:** MedDRA search fails and displays an error.

### Page 30 - Case status, QC, Lock, delete/restore

- **UI-SPEC-094 - Lock defect:** Unlock displays an error and does not unlock the case.
- **UI-SPEC-095 - UX defect:** Locking a case also locks Appendix selection; Appendix selection must remain available.
- **UI-SPEC-096 - Audit coverage defect:** QC and Lock actions are absent from Audit Trail.
- **UI-SPEC-097 - UX defect:** Fields remain clickable after QC/Lock even though save is blocked; they should become non-editable.
- **UI-SPEC-098 - State defect:** Lock changes QC from Yes to No; QC must remain Yes.
- **UI-SPEC-099 - Delete-state defect:** A deleted case still permits editing until save fails; disable editing immediately.
- **UI-SPEC-100 - Auditability:** Require Comments/reason when deleting a case.
- **UI-SPEC-101 - Recovery defect:** Add Restore after deletion and require Comments/reason for restoration.

### Page 31 - Case editor global table/actions

- **UI-SPEC-102 - Global action requirement:** Every ellipsis without Notation must offer `Audit Trail` and `Erase` (pp. 31-43).
- **UI-SPEC-103 - Audit UX:** `Audit Trail` must open a field-level audit dialog.
- **UI-SPEC-104 - Erase UX:** `Erase` must clear the selected field.
- **UI-SPEC-105 - Soft-delete semantics:** Persisted rows must show strikethrough when soft-deleted; unsaved rows should disappear without strikethrough.
- **UI-SPEC-106 - Global table layout:** Apply the revised C.1.6.1.r table pattern to all later tables: field names in headers and adjusted column widths.
- **UI-SPEC-107 - Global ordering:** Recheck and correct field ordering across every Case page.

### Page 32 - Case editor global warnings/notation/import

- **UI-SPEC-108 - Global warning indicator:** Show a red dot on navigation items/pages containing warnings.
- **UI-SPEC-109 - Header defect:** Case No. and Follow-up number are displayed incorrectly on this and other pages; verify all pages.
- **UI-SPEC-110 - Global validation/lookup follow-up:** Recheck NullFlavor, country codes, UCUM, MedDRA, and soft delete across every page.
- **UI-SPEC-111 - Naming cleanup:** Remove `Template` from every Import button label.
- **UI-SPEC-112 - Global action requirement:** Fields supporting Notation must offer `Audit Trail`, `Erase`, and `Notation` (pp. 32-43).
- **UI-SPEC-113 - Notation UX:** Selecting Notation must create a parallel input below the field for English/Korean translation.
- **UI-SPEC-114 - Output behavior:** When Notation output is selected, export the Notation value.

### Page 33 - Sender Case page

- **UI-SPEC-115 - Import defect:** Sender and other identified pages fail to import INFO data; recheck all Case section imports.
- **UI-SPEC-116 - Global actions:** Add `Audit Trail`, `Erase`, and `Notation` to the annotated Sender fields.
- **UI-SPEC-117 - Deletion request:** Remove the annotated `Sender Type (C.3.1)` block.
- **UI-SPEC-118 - Regression:** Remove the obsolete Message Header/Receiver block that had previously been deleted but reappeared.

### Page 34 - Literature References

- **UI-SPEC-119 - Global actions:** Add the required field action menus to annotated Literature Reference fields.
- **UI-SPEC-120 - NullFlavor consistency:** Standardize button/dropdown NullFlavor input.
- **UI-SPEC-121 - Layout defect:** Included Documents UI overflows the viewport; fix width/alignment.
- **UI-SPEC-122 - Deletion request:** Remove the instructional placeholder text in Included Documents.
- **UI-SPEC-123 - Soft-delete UX:** Show strikethrough on delete, replace Delete with Restore, and allow restoration.

### Page 35 - Repeated Case tables

- **UI-SPEC-124 - Soft-delete UX:** Apply strikethrough/Delete-to-Restore behavior to the annotated repeated table.
- **UI-SPEC-125 - Table layout:** Put field names in table headers and resize columns.
- **UI-SPEC-126 - Global actions:** Add `Audit Trail`, `Erase`, and `Notation` to all annotated fields.
- **UI-SPEC-127 - NullFlavor consistency:** Standardize NullFlavor interaction on the page.

### Pages 36-40 - Case editor field actions and MedDRA

- **UI-SPEC-128 - Global actions:** Add `Audit Trail`, `Erase`, and `Notation` to every annotated field on pages 36, 38, and 39.
- **UI-SPEC-129 - MedDRA UX:** Change all MedDRA version inputs to dropdown selection (p. 37).
- **UI-SPEC-130 - Coverage:** Page 37 and page 40 contain multiple annotated fields; ensure no field action menu is omitted.

### Page 41 - Drug/Product Case section

- **UI-SPEC-131 - Naming change:** Rename the annotated field to `Product ID`.
- **UI-SPEC-132 - Data linkage:** Product ID must be linked to Product data from INFO.
- **UI-SPEC-133 - Coverage:** Add `Audit Trail`, `Erase`, and `Notation` to every annotated drug field.

### Page 42 - Drug-reaction assessment

- **UI-SPEC-134 - Data linkage:** Select adverse events from AE (E.i) data entered in the case.
- **UI-SPEC-135 - Constraint:** Prevent the same AE from being selected twice in the assessment.
- **UI-SPEC-136 - Data model/UI change:** Make the assessment group repeatable and table-based.
- **UI-SPEC-137 - UX change:** Add numbered repeated rows using the Add Assessment pattern.
- **UI-SPEC-138 - Soft-delete UX:** Add delete/strikethrough/Restore behavior to assessment rows.
- **UI-SPEC-139 - Non-E2B field:** Add Expectedness.
- **UI-SPEC-140 - Coverage:** Add field action menus to all annotated assessment fields.

### Page 43 - Narrative / E2B review

- **UI-SPEC-141 - Global actions:** Add `Audit Trail`, `Erase`, and `Notation` to annotated fields.
- **UI-SPEC-142 - Standards verification:** Recheck E2B R3 fields and update Notation behavior/mapping accordingly.

### Page 44 - FDA Appendix

- **UI-SPEC-143 - Layout change:** Move the annotated FDA Appendix section upward.

### Page 45 - MFDS Appendix

- **UI-SPEC-144 - Repetition defect:** Recheck the repeat boundary; every field in the annotated MFDS area belongs to the repeated group.
- **UI-SPEC-145 - Ordering defect:** Reorder MFDS fields to match cubeSAFETY.

### Page 46 - Import E2B(R3) Files

- **UI-SPEC-146 - Duplicate import defect:** The same file can be imported more than once; define and enforce duplicate-file conditions.
- **UI-SPEC-147 - Import identity defect:** Even after changing C.1.1/C.1.2 to avoid duplication, re-uploading the same file is blocked based on file identity.
- **UI-SPEC-148 - Workflow defect:** Selecting a file starts import immediately; selection must stage the file only.
- **UI-SPEC-149 - Change request:** Add Product ID selection and an explicit Import button.
- **UI-SPEC-150 - Authorization:** Product ID choices must be restricted to INFO Products granted to the user.
- **UI-SPEC-151 - Settings integration:** Apply the Product-linked Sender according to the ADMIN > SETTINGS imported-case option.

### Page 47 - Import History

- **UI-SPEC-152 - Observability defect:** Import History must also show files rejected as duplicate/same-file uploads.

### Page 48 - Export / Submission setup

- **UI-SPEC-153 - Layout:** Place Check and post-Check action buttons at the bottom-right.
- **UI-SPEC-154 - Naming defect:** Page names must match actual Case page names.
- **UI-SPEC-155 - Change request:** Add a Notation checkbox; when selected, output Notation content.
- **UI-SPEC-156 - Metadata defect:** Displayed element names do not match actual element names.
- **UI-SPEC-157 - Coverage defect:** Only some elements from the selected page are shown; list every element.

### Page 49 - Export / Submission flow and filters

- **UI-SPEC-158 - Deletion request:** Remove the annotated Export Scope/Submission Scope controls, existing action row, and Submission Guidelines area.
- **UI-SPEC-159 - Verification:** After PDF generation, confirm the case is removed from the set eligible for another PDF output if that is the intended rule.
- **UI-SPEC-160 - Initial state:** Initially show only `Check`.
- **UI-SPEC-161 - Action transition:** After Check, replace it with `Export XML`, `Export CIOMS`, `Excel Line List`, and `Submit Cases`.
- **UI-SPEC-162 - Filter redesign:** Use Select Page, Select Item, Condition, and Direct Input fields.
- **UI-SPEC-163 - Filter operators:** Support Equal, Not Equal, Range, Like, Not Like, Null, Not Null, and In.
- **UI-SPEC-164 - Range UX:** Range conditions require two input boxes.
- **UI-SPEC-165 - Compound filters:** `+` must add another AND condition row.
- **UI-SPEC-166 - Follow-up selection:** Add a checkbox that selects only the final follow-up report; unchecked means all reports.
- **UI-SPEC-167 - Result flow:** Check must query the DB and display matching cases before export/submission actions become available.

### Page 50 - Deferred follow-up

- **UI-SPEC-168 - Deferred feature:** Add email notifications based on account configuration and workflow stage.
- **UI-SPEC-169 - Deferred feature:** Add automatic translation for values entered through Notation.

## Consolidated global acceptance themes

- Authorization must be enforced consistently at login, navigation, list-query, import, and case-query levels—not only by hiding UI controls.
- Every applicable field needs consistent NullFlavor handling, terminology sources, field actions, and authority visibility.
- Soft delete must preserve stored data and audit reason, prevent editing while deleted, and support reasoned restoration.
- Audit must preserve immutable old/new values and include field edits, QC, Lock, delete, and restore actions.
- Imports and exports need explicit staging/check steps, deterministic duplicate handling, and user-scope-aware Product/Sender selection.
