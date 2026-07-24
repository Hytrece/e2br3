# Regulatory Rule Source Coverage Design

## Goal

Prevent an ICH, FDA, or MFDS business-rule prose entry applicable to a
certified editor page from being absent from both the executable canonical
catalog and the section rule tables while all existing parity tests still pass.

This change does not replace or redesign the canonical validator catalog,
portable constraints, section rule tables, or shared evaluators.

## Existing Boundaries

- `registry/dictionary/rules/{ich,fda,mfds}.json` remains the generated,
  authoritative prose inventory.
- Portable representation constraints remain in the existing constraint
  catalog and portable bindings.
- Business validation remains in the existing canonical catalog and section
  rule tables.
- Existing canonical-catalog-to-rule-table exact-set tests remain authoritative
  for executable business-rule wiring.

The missing boundary is prose-source coverage: nothing currently proves that a
prose requirement was reviewed and either connected to an existing executable
rule or explicitly classified as non-executable or deferred.

## Coverage Crosswalk

Add one reviewed artifact at `registry/rule-source-coverage.json`.

Each source block is identified by authority and data-element code and records
the hash of the exact trimmed prose in the generated rules file:

```json
{
  "version": 1,
  "sources": [
    {
      "authority": "FDA",
      "element": "C.4.r.2",
      "sourceHash": "fnv1a64:...",
      "requirements": [
        {
          "id": "filename-media-type-match",
          "sourceExcerpt": "If the file extension in the filename does not match the media type",
          "disposition": "deferred",
          "reason": "The literature attachment filename is not persisted."
        }
      ]
    }
  ]
}
```

Requirement IDs are stable within an authority and element. `sourceExcerpt`
must be a literal substring of the current prose so a reviewer can trace the
classification to its evidence. The source hash makes any upstream prose
change stale and forces re-review.

Allowed dispositions are:

- `business_rule`: requires one or more canonical business-rule codes.
- `constraint`: requires one or more portable constraint codes.
- `guidance`: requires a reason explaining why no executable check applies.
- `deferred`: requires an implementation blocker or reason and prevents a
  corresponding editor field from being certified complete.

The crosswalk is coverage metadata only. It is not loaded at runtime and cannot
change validation behavior.

## Validation Gates

Extend `registry/tools/validate.py` with source-coverage validation:

1. Every prose source block applicable to a field in a strict editor contract
   must have exactly one crosswalk source entry for each authority that defines
   prose for that element.
2. Crosswalk entries must not reference absent prose blocks.
3. `sourceHash` must match the current trimmed prose.
4. Every source entry must contain at least one requirement.
5. Requirement IDs must be unique within their source entry.
6. `sourceExcerpt` must occur in the current prose.
7. `business_rule` requirements must reference canonical case-validation rule
   codes.
8. `constraint` requirements must reference existing portable constraint
   codes and bindings.
9. `guidance` and `deferred` requirements must provide a non-empty reason.
10. A registry field marked `complete` must not have a matching `deferred`
    requirement under any applicable ICH, FDA, or MFDS source entry.

Python registry validation owns checks 1 through 6, 9, and 10. A focused Rust
unit test loads the same crosswalk with `include_str!` and owns checks 7 and 8
against the actual compiled canonical catalog and portable bindings. This
avoids a generated executable-rule inventory and does not introduce a second
catalog. The existing exact-set test continues to prove that a canonical
case-validation rule is backed by a section rule table.

## Editor Certification

Editor certification is authority-complete rather than representative-authority
only. For a shared ICH element such as `C.4.r.2`, the gate evaluates:

- the ICH prose source entry;
- any FDA prose source entry keyed by `C.4.r.2`;
- any MFDS prose source entry keyed by `C.4.r.2`.

Local-only fields with no regulatory element are exempt. A `guidance`
classification does not block completion. A `deferred` classification does.

The first rollout covers every field referenced by the existing CI, RP, SD, and
LR editor contracts. The validator also reports prose entries outside those
pages as unaudited inventory, but they do not become completion claims until
their editor section is certified. New strict editor contracts must have source
coverage from their first introduction. Expanding coverage to a new page is
therefore part of that page's certification, not a repository-wide flag day.

## Workflow for Implementing a Deferred Rule

1. Add or correct the rule in the existing canonical business catalog, or map
   it to an existing portable constraint.
2. Bind business rules through the existing section rule table and shared
   evaluators.
3. Add focused condition, issue-path, and authority tests.
4. Add missing UI/API/DB/XML data plumbing when the rule input is not currently
   preserved.
5. Change the crosswalk disposition from `deferred` to `business_rule` or
   `constraint`.
6. Restore the registry field to `complete` only after its editor contract,
   save/reload roundtrip, and applicable validation tests pass.

## Testing

- Unit tests for missing, duplicate, stale, and orphaned source entries.
- Unit tests for every disposition and its required fields.
- Tests rejecting nonexistent canonical or portable rule codes.
- Tests proving a deferred regional rule prevents `complete`.
- Tests proving guidance does not prevent `complete`.
- Strict CI/RP/SD/LR contract validation using the reviewed crosswalk.
- Existing canonical-catalog-to-rule-table and portable-binding parity tests
  remain unchanged and must continue to pass.

## Non-Goals

- Parsing regulatory prose into executable rules automatically.
- Replacing `VALIDATION_RULES`, condition bindings, value policies, or section
  rule tables.
- Executing validation from the crosswalk at runtime.
- Marking a rule implemented merely because its element code appears in the
  canonical catalog; each distinct requirement must name its actual rule code.
