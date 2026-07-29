# Remaining CRUD and Constraint Completion Design

## Scope

Complete the fields that the section audit left `incomplete`, using only:

- frontend/API/DB CRUD and reload roundtrip;
- portable catalog-based constraint rejection;
- field-level registry status and strict editor contracts.

Business validation and prose-rule coverage are explicitly out of scope.

The implementation runs in this order:

1. AE representation mismatches;
2. DG nested row owners;
3. NR nested row owners;
4. N.1.5 message-header timestamp.

## Completion rule

A registry field may return to `complete` only when:

1. the canonical frontend value is accepted by the intended write endpoint;
2. the same value is persisted without information loss;
3. reload returns the same canonical meaning;
4. an applicable portable catalog constraint rejects an invalid API value with
   `CONSTRAINT_VIOLATION` and HTTP 422;
5. its strict contract records the roundtrip and constraint evidence.

Fields without an authoritative executable constraint remain `incomplete`.

## AE

### E.i.3.1 Term Highlighted

Use the ICH code as the canonical representation throughout the system:
`"1"`, `"2"`, `"3"`, or `"4"`.

- Change the reaction DB column and Rust model/DTO from boolean to coded text.
- Keep the AE frontend model and control coded, but correct the UI to expose all
  four official meanings.
- XML import stores the source code directly.
- XML export emits the stored code directly instead of reconstructing it from
  `term_highlighted + serious`.
- Remove boolean compatibility paths rather than retaining a second canonical
  representation.
- Verify all four values through API/DB/reload/XML roundtrip and reject other
  values through `ICH.E.i.3.1.ALLOWED.VALUE`.

The separate local `serious` field remains independent. Consistency between it
and E.i.3.1 is a business rule and is outside this work.

### FDA.E.i.3.2h Required Intervention

Represent the FDA BL value as `true` with an explicit `NI` null flavor.

- Replace the free-text frontend control with a typed true/NI control.
- Store the value and null flavor in their existing dedicated columns using
  typed API fields.
- Generate executable FDA regional allowed-value and null-flavor constraints
  from the regional dictionary, and make the portable catalog loader consume
  regional structured constraints without duplicating rule constants.
- Verify `true` and `NI` roundtrip and reject arbitrary strings before write.

### MFDS device fields

Keep the 17 device fields `incomplete`. CRUD already works, but no authoritative
MFDS device guideline is present in the registry. This work must not invent
allowed values or mark the fields complete without a source-backed executable
constraint.

## DG nested owners

The DG row projection already reads these owners:

- `activeSubstances`;
- `dosageInformation`;
- `indications`;
- `drugReactionAssessments` and their relatedness values.

Extend the DG row write path to persist the same nested arrays.

- Prevalidate the complete normalized DG payload before the first write.
- For each nested row, create when no id exists, update when an id belongs to
  the current drug, and soft-delete only when explicitly requested.
- Reject cross-drug ids.
- Preserve sequence numbers and return the existing composite DG row
  projection after every write.
- Use the current BMC ownership boundaries; do not create an additional legacy
  mapper or alternate payload shape.

Focused tests cover create, update, delete, reload, and one representative
constraint rejection per nested owner. Registry rows return to `complete` only
for owners whose full test set passes.

## NR nested owners

Extend `apply_nr_page_rows_patch` to persist:

- `senderDiagnoses`;
- `caseSummaryInformation`.

Use the same id ownership, upsert, explicit soft-delete, sequence, prevalidation,
and composite reload rules as DG. The existing narrative row behavior remains
unchanged.

## N.1.5 Message Header Timestamp

Use the 14-digit E2B timestamp string as the transport-canonical value.

- The frontend submission payload sends the E2B string instead of the legacy
  `OffsetDateTime` tuple.
- The message-header REST boundary validates raw input with
  `ICH.N.1.5.ALLOWED.VALUE` before typed conversion.
- After validation, convert the canonical string once for DB storage.
- Responses continue to normalize back to the E2B string on the submission
  model boundary.
- Remove tuple compatibility from the frontend path.

An invalid timestamp must produce structured HTTP 422, not a JSON extractor
error.

## Error handling and atomicity

All portable constraints are evaluated before any section write. Ownership and
shape errors return 400; catalog constraint failures return structured 422.
Existing per-model transaction behavior is retained. No unrelated transaction
framework refactor is included.

## Verification

For each section:

- run only the new or previously failing focused test during development;
- run its strict registry contract;
- run format and diff checks;
- at completion, rerun the focused CRUD/reload and constraint tests for changed
  sections.

Do not run the full workspace suite as part of this work.
