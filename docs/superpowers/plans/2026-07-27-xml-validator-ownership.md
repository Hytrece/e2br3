# XML Validator Ownership Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Delete `validator::xml` and expose all XML-document validation from the `xml` crate.

**Architecture:** Move XML traversal/profile modules into `xml::validation`. Move only the pure rule catalog/evaluation surface required by those modules into `xml`, then make case-edit validator code consume that surface. Preserve the acyclic `validator -> xml -> lib-core` dependency.

**Tech Stack:** Rust 2021, Cargo workspace, libxml, quick-xml.

## Global Constraints

- Preserve XML validation messages, codes, blocking flags, profiles, and ordering.
- Do not introduce `xml -> validator`.
- Delete `validator/src/xml` and every `validator::xml` reference.

---

### Task 1: Establish XML Business Validation API Ownership

**Files:**
- Modify: `crates/libs/xml/tests/validation_api.rs`
- Modify: `crates/libs/xml/src/validation.rs`

**Interfaces:**
- Produces: `xml::validation::validate_e2b_xml_business(&[u8], Option<XmlValidatorConfig>) -> xml::Result<XmlValidationReport>`

- [ ] Add a compile-time API test importing `validate_e2b_xml_business` from `xml::validation`.
- [ ] Run `cargo test -p xml --test validation_api` and verify the unresolved import failure.
- [ ] Move the XML traversal/profile modules under `crates/libs/xml/src/validation/` and expose the function from `validation.rs`.
- [ ] Run the API test and XML integration tests.

### Task 2: Move Required Rule Evaluation Ownership

**Files:**
- Modify: `crates/libs/validator/src/lib.rs`
- Modify: validator rule catalog/evaluation modules identified by compiler diagnostics
- Create or modify: `crates/libs/xml/src/rules.rs`
- Modify: moved XML validation modules

**Interfaces:**
- Produces pure catalog lookup and rule presence/value/condition evaluation APIs used by both XML validation and case-edit validation.

- [ ] Inventory the exact rule types/functions imported by the old XML module.
- [ ] Move their ownership to `xml::rules` without database or validator dependencies.
- [ ] Re-export or import those APIs in validator case-edit modules.
- [ ] Run `cargo check -p xml -p validator --all-targets` and fix ownership paths without compatibility `validator::xml` exports.

### Task 3: Migrate Consumers and Delete Legacy Module

**Files:**
- Modify: `crates/services/web-server/src/submission.rs`
- Modify: `crates/services/web-server/src/web/rest/case_export_rest.rs`
- Modify: `crates/services/web-server/tests/xml/roundtrip_profiles_web.rs`
- Modify: `crates/libs/validator/tests/xml/*.rs`
- Delete: `crates/libs/validator/src/xml/**`

**Interfaces:**
- Consumes: `xml::validation::validate_e2b_xml_business`.

- [ ] Update production and test imports to `xml::validation`.
- [ ] Remove `pub mod xml` from validator and delete the legacy directory.
- [ ] Run XML and validator integration tests.
- [ ] Commit the ownership migration.

### Task 4: Verify Boundaries and Behavior

**Files:**
- Verify only; no compatibility layer.

- [ ] Run `cargo check --workspace --all-targets`.
- [ ] Run `cargo test -p xml` and `cargo test -p validator --test xml`.
- [ ] Run relevant web-server submission/export tests.
- [ ] Confirm `rg 'validator::xml|crate::xml' crates/libs/validator crates/services/web-server` has no legacy hits and `crates/libs/validator/src/xml` is absent.
- [ ] Confirm `cargo tree -p xml` contains no validator dependency and the worktree is clean.
