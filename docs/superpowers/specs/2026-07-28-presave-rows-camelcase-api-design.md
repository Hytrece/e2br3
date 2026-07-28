# Presave Rows and CamelCase API Design

## Goal

Align every Info presave write contract with the canonical Case Editor API
conventions. Presave requests and responses use camelCase JSON, aggregate writes
use a `rows` object instead of `parent` plus section-specific snake_case child
keys, and repeatable rows use the Case Editor lifecycle fields `id`,
`sequenceNumber`, and `deleted`.

Rust model fields and PostgreSQL columns remain snake_case. Serde and explicit
REST DTOs own the JSON naming boundary.

## Scope

The change covers the Sender, Receiver, Product, Reporter, Study, and Narrative
presave REST APIs and their frontend callers. It includes collection create,
single-record read/update, aggregate detail read/update, OpenAPI schemas, and the
presave-to-case import boundary where the shared row shape removes unnecessary
translation.

It does not rename database columns, Rust domain-model fields, audit table
columns, or E2B registry storage fields. It does not make a presave aggregate
identical to an entire Case Editor page; only their JSON naming and row lifecycle
conventions are shared.

## Canonical Envelope

Aggregate reads and writes use the existing REST `data` wrapper with one `rows`
member. Each section gives its primary row a semantic name instead of `parent`.

```json
{
  "data": {
    "rows": {
      "product": {
        "productId": "P-001",
        "medicinalProduct": "Product A",
        "senderPresaveId": "00000000-0000-0000-0000-000000000001",
        "receiverPresaveId": "00000000-0000-0000-0000-000000000002"
      },
      "activeSubstances": [
        {
          "id": "00000000-0000-0000-0000-000000000003",
          "sequenceNumber": 1,
          "substanceName": "Caffeine",
          "substanceTermIdVersion": "v1",
          "substanceTermId": "TERM-1",
          "substanceStrengthValue": 10,
          "substanceStrengthUnit": "mg",
          "deleted": false
        }
      ]
    }
  }
}
```

The request wrapper remains `data` because it is the repository-wide REST
mutation envelope. The payload does not introduce a `changes` field.

## Section Row Names

- Sender: `sender`, `gateways`, `responsiblePersons`
- Receiver: `receiver`, `consignees`, `routes`
- Product: `product`, `activeSubstances`
- Reporter: `reporter`
- Study: `study`, `products`, `reporters`, `registrationNumbers`,
  `fdaCrossReportedInds`
- Narrative: `narrative`

These names cover every child family exposed by the current aggregate DTOs. The
obsolete Receiver response-only `children` duplicate is removed; `consignees`
and `routes` appear once directly under `rows`.

## Row Lifecycle

All primary and repeatable rows follow the Case Editor rules:

- a persisted row carries `id`;
- a new child row omits `id`;
- `sequenceNumber` defines repeatable-row order;
- `deleted: true` requests deletion or archival;
- `deleted: false` represents an active row when returned;
- `_delete` is not part of the canonical contract;
- a new row with `deleted: true` is rejected;
- an existing child `id` must belong to the aggregate in the route path;
- aggregate changes are validated before any write and committed atomically.

Primary presave deletion continues to use the presave lifecycle service so
references and authorization grants are protected. Child deletion retains the
current domain behavior, whether that is soft deletion or physical deletion,
but exposes the single JSON instruction `deleted: true`.

## Field Naming

Public JSON uses the Case Editor camelCase names. Where Presave and Case Editor
currently use different concepts for the same value, Case Editor naming wins.
For Product active substances this includes:

| Meaning | Canonical JSON |
|---|---|
| collection | `activeSubstances` |
| row order | `sequenceNumber` |
| substance name | `substanceName` |
| terminology version | `substanceTermIdVersion` |
| terminology identifier | `substanceTermId` |
| MFDS version | `mfdsVersion` |
| MFDS identifier | `mfdsId` |
| strength value | `substanceStrengthValue` |
| strength unit | `substanceStrengthUnit` |
| deletion | `deleted` |

Display-only frontend fields such as resolved sender and receiver labels are
not accepted as writable Product fields. Relationships are written by their
presave identifiers.

## Backend Architecture

REST-specific request and response DTOs define the rows contract and use
`#[serde(rename_all = "camelCase", deny_unknown_fields)]` for request objects.
DTO conversion maps camelCase JSON into existing snake_case domain create/update
models. Domain models and database access remain independent of API casing.

Each aggregate update follows one pipeline:

1. deserialize and reject unknown properties;
2. authorize the presave aggregate;
3. preflight primary and child rows, including ownership and lifecycle checks;
4. convert the REST rows into domain mutations;
5. apply all mutations in the existing transaction boundary;
6. reload and serialize the complete canonical rows response.

OpenAPI documents only the canonical camelCase rows contract after migration.

The existing route families remain stable while their bodies converge:

- collection `POST` accepts `data.rows` and creates the primary row plus any
  supplied children atomically;
- collection `GET` returns camelCase summary rows;
- `GET /{id}/details` returns the complete `data.rows` aggregate;
- `PUT /{id}/details` accepts the complete or partial `data.rows` aggregate;
- primary-only `PATCH /{id}` remains available for narrow integrations but
  accepts `data.rows.<primaryRowName>` instead of a flat snake_case object;
- `DELETE /{id}` retains its existing lifecycle behavior;
- child-specific routes retain their URLs but use camelCase row bodies and
  responses.

The aggregate detail route is the canonical Info UI save path. Create no longer
requires a primary POST followed by a second child PUT.

## Frontend Architecture

Info forms use camelCase values matching the API. Canonical read/write mappers
that only translate casing, `parent`, child collection names, or `_delete` are
removed. UI-only display labels remain local view state and never enter request
DTOs.

Shared row types and lifecycle helpers are reused with Case Edit where the
semantics are genuinely identical. A complete Case Editor Drug type is not
reused for Product presaves because case-only fields and presave relationship
fields differ. Shared types are factored at the smallest common row boundary,
such as `ActiveSubstanceRow`.

## Migration

This is a coordinated breaking contract change, consistent with the existing
Case Editor rows-only migration. Backend and frontend changes ship together.
There is no long-lived dual contract and no silent acceptance of unknown legacy
keys.

During implementation, tests are converted first and provide the migration
inventory. Once the frontend caller and backend route for a section both use the
new contract, the corresponding legacy keys (`parent`, snake_case child names,
and `_delete`) are removed. A request using a removed key returns HTTP 422.

Because backend and frontend are separate repositories, deployment must either
be atomic or deploy a short-lived compatibility build explicitly scheduled for
removal in the same release. Compatibility aliases must not remain in the final
state.

## Error Handling

- unknown top-level, `rows`, primary-row, or child-row properties return 422;
- malformed UUIDs and row value types return 422 with the affected camelCase
  path;
- domain validation errors preserve their existing rule codes and use camelCase
  field paths at the HTTP boundary;
- unauthorized relationships and cross-aggregate child IDs are rejected before
  mutation;
- any preflight or persistence failure rolls back the aggregate write;
- the frontend displays the backend error detail instead of replacing it with a
  generic save failure.

## Testing

Tests are changed before production code and must demonstrate the old contract
failing against the new expectation.

- Backend contract tests cover create, read, update, delete, child create,
  child update, and child delete for each section.
- Product tests explicitly cover `activeSubstances`, strength fields, resolved
  label exclusion, and rejection of `active_substances`, `substances`,
  `parent`, and `_delete`.
- Atomicity tests prove that one invalid child prevents every primary and child
  mutation.
- Authorization tests prove referenced Sender, Receiver, Product, Reporter, and
  Study IDs remain organization- and scope-safe.
- Frontend hook/form tests assert the literal camelCase rows payload.
- Presave-to-case tests prove shared row values survive import without casing or
  lifecycle translation errors.
- OpenAPI tests assert camelCase schemas and absence of legacy request fields.
- A browser test creates and edits an Info Product with an active substance,
  reloads it, and applies it to a Case Edit drug.

## Success Criteria

- Every Presave request and response uses camelCase JSON.
- Every aggregate Presave detail write uses `data.rows` and never `changes`.
- `parent`, snake_case child collection names, and `_delete` are absent from the
  final public contract.
- Case Edit and Presave repeatable rows share `id`, `sequenceNumber`, and
  `deleted` lifecycle semantics.
- Frontend casing-only Presave mappers are removed.
- Rust domain models and database columns remain snake_case.
- Focused backend, frontend, OpenAPI, atomicity, authorization, import, and
  browser round-trip tests pass.
