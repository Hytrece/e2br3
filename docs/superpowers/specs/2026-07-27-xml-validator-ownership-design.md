# XML Validator Ownership Design

## Goal

Remove `validator::xml` and make the standalone `xml` crate own every XML-document validation stage, including catalog-driven ICH/FDA/MFDS business validation.

## Architecture

- `xml` owns XML parsing, basic checks, XSD validation, XML rule traversal, and XML business validation.
- The rule catalog and pure rule-evaluation APIs required by XML validation move to `xml` so `xml` never depends on `validator`.
- `validator` retains case-edit validation and depends on `xml` for shared rule metadata/evaluation and export-normalization policy.
- `web-server` calls XML validation only through `xml`; it no longer imports an XML namespace from `validator`.

The dependency direction remains acyclic: `web-server -> validator -> xml -> lib-core`.

## API and Migration

- Preserve `validate_e2b_xml_business` under `xml::validation` to minimize behavior changes.
- Delete `validator/src/xml` and `pub mod xml`.
- Update validator internals, web-server call sites, and XML integration tests to the new owner.
- Preserve validation messages, codes, blocking flags, profiles, and ordering.

## Verification

- Add/adjust an API ownership test proving business validation is callable from `xml`.
- Run the `xml` suite, validator XML/case-policy tests, consumer compilation, and legacy-reference scans.
- Confirm neither `validator::xml` nor `validator/src/xml` remains.
