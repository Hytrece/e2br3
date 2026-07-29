# XML Crate Extraction Design

## Goal

Move the complete XML engine out of `lib-core` into a workspace crate named
`xml`. Keep the existing behavior and database integration while establishing
one-way dependencies and a clear ownership boundary.

This change is an extraction, not a redesign of import/export persistence.

## Scope

The new crate owns all code currently under `crates/libs/lib-core/src/xml`,
including:

- XML import and export entry points;
- parsers, XML-specific models, result types, and errors;
- FDA, ICH, and MFDS mappings and codes;
- DOM and round-trip patching;
- import/export section implementations and shared utilities;
- XML schema, basic document-format, and export-normalization validation that
  is currently housed in `validator` but does not evaluate its case-rule
  catalog;
- XML fixtures and XML-focused tests.

The following remain outside the new crate:

- case-edit and business-rule validation in `validator`;
- domain models, database access, `Ctx`, and `ModelManager` in `lib-core`;
- `xml_export_history`, because it is an application persistence model;
- submission transport, ACK handling, retries, and reconciliation;
- HTTP handlers and HTTP DTOs in `web-server`.

Submission extraction and removal of the XML engine's database dependency are
explicitly deferred.

## Architecture

The relevant dependency graph after extraction is:

```text
web-server   ──▶ xml ───────▶ lib-core
web-server   ──▶ validator ─▶ lib-core
web-server   ──▶ lib-core
validator    ──▶ xml (pure export-normalization policy only)
lib-rest-core──▶ xml
lib-rest-core──▶ lib-core
```

`lib-core` must not depend on `xml`. `validator` remains a case-rule validation
engine that depends on `lib-core`; it may depend on `xml` for the pure,
database-independent XML types, basic document validation, and
export-normalization policy described below. `web-server` may depend directly
on all three crates. `lib-rest-core` depends on `xml` to convert `xml::Error`
into API errors and retains its existing `lib-core` dependency.

The `xml` crate is allowed to depend on `lib-core` in this phase. Import and
export retain their existing use of `Ctx`, `ModelManager`, domain BMCs, and
database transactions.

## Crate Layout

Create the package at `crates/libs/xml` and add it to the workspace. Its module
layout follows the existing XML tree to minimize behavioral changes:

```text
crates/libs/xml/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── error.rs
│   ├── types.rs
│   ├── parser.rs
│   ├── export_data.rs
│   ├── export_utils.rs
│   ├── import.rs
│   ├── import_runtime/
│   ├── import_sections/
│   ├── export.rs
│   ├── export/
│   ├── mapping/
│   ├── raw/
│   ├── fda/
│   ├── ich/
│   ├── mfds/
│   ├── model/
│   ├── xml_validation/
│   ├── fda_optional_paths.txt
│   └── fixtures/
└── tests/
```

Internal references such as `crate::model` become `lib_core::model`, while
references between XML modules remain crate-local.

## Public API

The extraction preserves the behavior of the currently consumed APIs, but all
callers move from `lib_core::xml` to `xml`. No compatibility re-export remains
in `lib-core`.

Primary entry points include:

```rust
xml::export::{
    export_case_xml,
    export_case_xml_with_options,
    ExportXmlOptions,
}

xml::import::{
    import_e2b_xml,
    import_e2b_xml_unvalidated,
    XmlImportRequest,
    CImportSettings,
}

xml::{
    parse_e2b_xml,
    Error,
    Result,
    XmlImportResult,
    XmlValidationError,
    XmlValidationReport,
}
```

Additional APIs used by existing tests or validation code may remain public
where required for a behavior-preserving migration. The extraction must not
expand visibility without an existing consumer or a documented need.

## Validation Boundary

Validation is divided by responsibility:

- `validator` retains validation of case-edit data and domain business rules;
- `xml` owns XSD validation, XML structure and namespace checks, parsing errors,
  XML normalization, and generated-document format checks.

The existing `validator/src/xml` code must be classified by responsibility
rather than moved wholesale. `validate_e2b_xml`, `validate_e2b_xml_basic`, XSD
helpers, configuration, and environment gating move to `xml`. The
`validate_e2b_xml_business` entry point and its authority/section profiles stay
in `validator` because they evaluate the validator rule catalog against an XML
representation. Moving that code to `xml` would introduce an `xml -> validator`
edge and a dependency cycle.

The existing `xml::export::policy` module is pure and database-independent. The
case-edit policies in `validator` may continue to import only this policy API
from `xml`, including null-flavor normalization, outcome display defaults, and
normalization specifications. This limited `validator -> xml -> lib-core`
dependency is acyclic and preserves the current rule behavior. The business XML
validator also consumes `xml::XmlValidatorConfig`, `xml::XmlValidationError`,
`xml::XmlValidationReport`, `xml::Result`, and the basic validation entry point.
Schema/XSD implementation and XML error/type ownership do not remain in
`validator`.

Submission continues to require a validated case status. It uses the XML crate
for document generation and schema validation and preserves its existing call
to `validator::xml::validate_e2b_xml_business`. This extraction does not change
submission validation semantics.

## Error Handling

The new crate defines `xml::Error` and `xml::Result`. The error may continue to
wrap `lib_core::model::Error`, reflecting the retained database dependency.

HTTP-specific errors must not enter `xml`. `web-server` and `lib-rest-core`
convert `xml::Error` into API responses. Error messages and failure behavior
must remain compatible unless a change is required to remove an invalid layer
dependency.

## Migration Strategy

Perform the extraction as one workspace change with behavior-preserving steps:

1. Create `crates/libs/xml` and declare `xml -> lib-core` dependencies.
2. Move the complete XML module tree, fixtures, and focused tests.
3. Convert internal paths from `crate::...` to the new crate boundary.
4. Move basic/schema XML validation out of `validator`, retain business XML and
   case validation, and update their XML type, helper, and normalization-policy
   imports to `xml`.
5. Update `web-server`, `validator`, and `lib-rest-core` call sites and manifests.
6. Remove `pub mod xml` and XML-only dependencies from `lib-core`.
7. Verify the whole workspace and confirm no `lib_core::xml` references remain.

Tests should move with the code they exercise. Web-server integration and API
tests remain in place and provide end-to-end regression coverage.

## Verification and Completion Criteria

The extraction is complete when:

- the entire workspace builds;
- XML unit, integration, import/export round-trip, and web API tests pass;
- case-edit and business XML validator tests pass with `validator -> xml`
  limited to public XML types, basic validation helpers, and the pure
  export-normalization policy API;
- `lib-core` has no XML module or XML-only dependencies;
- no source or test imports `lib_core::xml`;
- the Cargo dependency graph has no cycle;
- import/export output and database transaction behavior remain unchanged.

## Deferred Work

The following are intentionally not part of this change:

- making `xml` a pure, database-independent codec;
- introducing import/export DTO assembly services;
- extracting submission into its own crate;
- changing submission workflow or case validation semantics;
- renaming or redesigning existing public XML types beyond what compilation
  requires.
