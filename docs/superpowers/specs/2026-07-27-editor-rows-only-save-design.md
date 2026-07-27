# Editor Rows-Only Save Design

## Goal

Remove the case-editor `changes` request format completely and make `rows` the
only accepted representation for page saves. This eliminates two independently
maintained mappings for the same fields and prevents fields such as D.7.2 from
being accepted by one save path but omitted by the other.

## Scope

This change applies to every endpoint that consumes
`CaseEditorPagePatchRequest`, including direct pages and macro-generated editor
pages. It removes the `changes` API without a compatibility or deprecation
period. Existing clients must send `rows` payloads.

The page read/projection response remains unchanged. Row-specific create,
update, delete, and restore endpoints outside `CaseEditorPagePatchRequest` are
also unchanged.

## Request Contract

`CaseEditorPagePatchRequest` contains:

- optional `authorities`;
- `rows`, defaulting to an empty map.

It no longer contains `changes`. The request type uses strict unknown-field
deserialization so a payload containing `changes` returns HTTP 400 instead of
being silently ignored. The OpenAPI request schema exposes only `authorities`
and `rows`.

## Architecture and Data Flow

All page patch handlers follow one pipeline:

1. Deserialize the request and reject unknown top-level properties.
2. Resolve and validate the requested authority context.
3. Validate `rows` through the portable row bindings and constraints.
4. Apply `rows` through the page's canonical row patch function.
5. Refresh validation caches when `rows` is non-empty.
6. Reload and return the existing page projection.

No handler synthesizes rows from field patches. The following compatibility
code is deleted:

- `CaseEditorFieldPatch` and its custom deserializer;
- `validate_direct_changes`;
- `patch_*_value` helpers used only by change processing;
- `changes_to_object`, `row_payload_from_changes`, and
  `row_array_payload_from_changes`;
- per-page change aliases and `apply_direct_page_changes_patch`;
- change-specific branches in the shared editor macros and DG/direct handlers;
- the OpenAPI `CaseEditorFieldPatchDoc` schema.

Helpers that still serve non-`changes` endpoints remain in place even if their
names include `patch`.

## D.7.2 Behavior

D.7.2 is saved only through the canonical DM row object:

```json
{
  "rows": {
    "patientInformation": {
      "medicalHistoryText": "Relevant history"
    }
  }
}
```

Its null flavor is saved through the companion row property:

```json
{
  "rows": {
    "patientInformation": {
      "medicalHistoryTextNullFlavor": "UNK"
    }
  }
}
```

The existing model rule continues to enforce mutual exclusivity: saving a text
value clears its null flavor, and saving a null flavor clears its text value.

## Error Handling

- A top-level `changes` property is an invalid request and returns HTTP 400.
- Unknown row names and unknown fields inside supported row objects continue to
  use the existing page-specific bad-request errors.
- Portable constraint violations continue to return HTTP 422 with the existing
  rule code and field path.
- Empty `rows` remains a valid no-op and does not refresh validation caches.

## Testing

Tests exercise the real HTTP routes and storage layer.

- Add a contract test proving `changes` is rejected with HTTP 400.
- Convert every existing `changes` request fixture to its canonical `rows`
  equivalent and preserve its behavioral assertions.
- Keep D.7.2 value and null-flavor round-trip coverage using only `rows`.
- Run focused tests for direct editor pages and macro-generated pages.
- Run the complete web-server API test target after focused tests pass.
- Confirm formatting for all modified Rust files. Existing unrelated formatting
  failures elsewhere in the worktree are reported separately and are not
  rewritten as part of this change.

## Migration and Compatibility

This is an intentional breaking API change. There is no server-side adapter,
feature flag, fallback, or dual-write period. Any frontend or external client
that still sends `changes` must migrate to `rows` in the same release. The
repository is searched for all request fixtures and callers; any caller outside
this repository must be coordinated separately by the deployment owner.

## Success Criteria

- No production type, handler, validator, OpenAPI schema, or test helper for the
  `changes` request format remains.
- Every case-editor page patch uses the `rows` validation and persistence path.
- A request containing `changes` fails visibly with HTTP 400.
- D.7.2 text and null flavor both persist and reload through `rows`.
- Focused and full API verification complete without regressions caused by this
  change.
