# Catalog Implementation Inventory

## Case Validator Coverage

The validator executes explicit field-code functions in
`case/sections/{c,d,e,f,g,h,n}.rs`. Each section exposes its implemented rule
codes for the exact-set regression, while shared primitives such as length,
allowed-value, date, MedDRA, and vocabulary checks live in
`case/sections/helpers.rs`.

| Catalog scope | Catalog rules | Field validators | Missing | Unexpected |
|---|---:|---:|---:|---:|
| `CaseValidate`, sections C/D/E/F/G/H/N, ICH/FDA/MFDS | 462 | 462 | 0 | 0 |

The exact-set regression is
`case::sections::tests::implemented_case_registry_matches_case_catalog`.
Run it with:

```bash
cargo test -p validator implemented_case_registry_matches_case_catalog --lib
```

The 462 field validators cover required/presence, companion, allowed-value,
vocabulary, MedDRA, maximum-length, future-date, and algorithmic violations.
They resolve concrete paths and conditional facts in the owning field function
and reuse the shared helpers. The count is enforced by
`case_catalog_is_fully_field_validator_backed`; a catalog rule added to this
scope fails the exact-set test until its field validator is registered.

## Input Contract Coverage

The portable runtime projection and cross-layer bindings have been removed.
The existing dictionaries were converted once into 262 explicit field-code
functions for Rust and matching Zod schemas for the frontend. Their comments
cover the same 399 input rule codes with no missing or unexpected rules.

Backend editor saves, XML import, and case intake call the Rust field functions
directly. The frontend calls the corresponding Zod schemas from explicit field
functions using frontend-local paths. Neither side performs runtime dictionary
lookup or maps frontend paths to backend request paths.

## Release-Backed Terminology

| Vocabulary | Source owner | Storage / release | Validator path | Operational input status |
|---|---|---|---|---|
| ISO 3166 | Approved ISO release import | `controlled_terminology_terms` / `iso3166` | `VocabularyContext` active membership; no fallback | Release import required |
| ICH constrained UCUM | Approved ICH constrained lists | `controlled_terminology_terms` / `ich_constrained_ucum` | Scoped exact membership; general UCUM remains parser-based | Release import required |
| EDQM | Approved EDQM export | `controlled_terminology_terms` / `edqm` | Scoped exact membership and active release version | Authenticated export required |
| MFDS domestic products | MFDS Service07 product and ingredient APIs (`ITEM_SEQ`, `MTRAL_CODE`) | `mfds_products`, `mfds_product_substances` / `mfds_product` | KR receiver selects product and linked ingredients | Collector, staged loader, release activation, active-only search, and explicit product-ingredient linking implemented |
| WHODrug foreign products | Licensed WHODrug release | `whodrug_products` / `whodrug` | FR receiver selects `WHODrug/all` | Licensed release import required |

MFDS product collection uses `registry/tools/import_mfds_products.py`. Raw pages and the normalized artifact are written below ignored `tmp/mfds-products/`. The service key is read only from `DATA_GO_KR_SERVICE_KEY` and is not persisted. Product-ingredient links come directly from the Service07 ingredient response, never from name matching. Loading creates a `validated`, inactive release; approval and activation use the existing terminology release endpoints. Runtime search reads only the active release through `GET /api/terminology/mfds-products`.
