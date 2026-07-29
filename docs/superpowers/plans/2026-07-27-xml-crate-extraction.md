# XML Crate Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extract the complete XML engine from `lib-core` into a workspace crate named `xml` while preserving database-backed import/export and existing validation behavior.

**Architecture:** `xml` depends one-way on `lib-core`; `lib-core` never depends on `xml`. `validator` keeps case validation and business-rule projection over XML, but consumes XML-owned types, basic/XSD validation, and pure export-normalization policy from `xml`. `web-server` and `lib-rest-core` switch directly from `lib_core::xml` to `xml`.

**Tech Stack:** Rust 2021 workspace, Cargo, Tokio, SQLx, quick-xml 0.31, libxml 0.3, existing integration tests.

## Global Constraints

- The package and Rust crate name is exactly `xml` at `crates/libs/xml`.
- Preserve the current `xml -> lib-core` dependency for `Ctx`, `ModelManager`, BMCs, and transactions.
- Do not add any `lib-core -> xml` dependency or compatibility re-export.
- Keep case-edit and case business-rule validation in `validator`.
- Move basic, XSD, namespace, and document-format validation ownership to `xml`.
- Keep `validator::xml::validate_e2b_xml_business` and its authority/section profiles in `validator` because they consume the validator rule catalog.
- Permit `validator -> xml` only for XML types, basic validation helpers, and pure export-normalization policy.
- Keep `xml_export_history` in `lib-core`.
- Do not extract submission or redesign import/export persistence in this change.
- Preserve public behavior, error messages, XML output, and transaction semantics.
- Do not retain any `lib_core::xml` source or test imports at completion.

---

## File Structure Map

- Create `crates/libs/xml/Cargo.toml`: package manifest and `lib-core` dependency.
- Create `crates/libs/xml/src/lib.rs`: former XML module root and public re-exports.
- Move `crates/libs/lib-core/src/xml/**` to `crates/libs/xml/src/**`: XML engine implementation, fixtures, mappings, and policies.
- Move `crates/libs/lib-core/tests/xml.rs`, `crates/libs/lib-core/tests/xml/**`, `crates/libs/lib-core/tests/import.rs`, and `crates/libs/lib-core/tests/import/**` to `crates/libs/xml/tests/**`: XML-focused tests.
- Create `crates/libs/xml/src/validation.rs`: basic parsing/root checks, XSD validation, validation config, default schema resolution, and environment gating extracted from `validator/src/xml/mod.rs`.
- Modify `crates/libs/validator/src/xml/mod.rs`: retain business XML validation orchestration and consume XML-owned validation primitives.
- Keep `crates/libs/validator/src/xml/{fda_profile.rs,ich_profile.rs,sections/**,shared_specs.rs}`: validator catalog projection over XML.
- Modify `crates/libs/validator/src/{c_safety_report_policy.rs,e_reaction_policy.rs,g_drug_policy.rs}`: import pure normalization policy from `xml`.
- Modify manifests for the workspace, `validator`, `web-server`, and `lib-rest-core`.
- Modify XML call sites in `web-server` and error conversion in `lib-rest-core`.

---

### Task 1: Extract the Existing XML Engine and Its Tests

**Files:**
- Create: `crates/libs/xml/Cargo.toml`
- Create/move: `crates/libs/xml/src/**`
- Move: `crates/libs/lib-core/tests/xml.rs` to `crates/libs/xml/tests/xml.rs`
- Move: `crates/libs/lib-core/tests/xml/**` to `crates/libs/xml/tests/xml/**`
- Move: `crates/libs/lib-core/tests/import.rs` to `crates/libs/xml/tests/import.rs`
- Move: `crates/libs/lib-core/tests/import/**` to `crates/libs/xml/tests/import/**`
- Modify: `Cargo.toml`
- Modify: `crates/libs/lib-core/Cargo.toml`
- Modify: `crates/libs/lib-core/src/lib.rs`
- Modify: all moved Rust sources and tests that reference `crate::model`, `crate::ctx`, `crate::e2b`, `crate::regulatory`, `crate::xml`, or `lib_core::xml`

**Interfaces:**
- Consumes: `lib_core::{ctx, e2b, model, regulatory}` and SQLx UUID types.
- Produces: crate root `xml::{Error, Result, parse_e2b_xml, XmlImportResult, XmlValidationError, XmlValidationReport}` plus existing `xml::export` and `xml::import` APIs.

- [ ] **Step 1: Record the pre-move XML test inventory and baseline compilation**

Run:

```bash
find crates/libs/lib-core/tests -maxdepth 2 -type f | sort | rg '/(xml|import)'
cargo test -p lib-core --test xml --test import --no-run
```

Expected: both test targets compile, and the inventory includes the XML export/import/patch suites and section import suites.

- [ ] **Step 2: Create the package manifest and register the workspace member**

Create `crates/libs/xml/Cargo.toml` using the XML-relevant dependencies currently declared by `lib-core`:

```toml
[package]
name = "xml"
version = "0.1.0"
edition = "2021"

[lib]
doctest = false

[lints]
workspace = true

[dependencies]
lib-core = { path = "../lib-core" }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_with = { workspace = true }
sqlx = { workspace = true }
modql = { workspace = true }
uuid = { version = "1", features = ["v4", "fast-rng", "serde"] }
time = { workspace = true }
rust_decimal = { workspace = true }
derive_more = { workspace = true }
quick-xml = "0.31"
libxml = "0.3"
tracing = "0.1"

[dev-dependencies]
serial_test = "3"
```

Add this workspace member next to the other application libraries:

```toml
"crates/libs/xml",            # E2B XML import/export and document validation
```

- [ ] **Step 3: Move the full XML implementation and focused tests without reorganizing internals**

Use history-preserving moves:

```bash
mkdir -p crates/libs/xml/src crates/libs/xml/tests
git mv crates/libs/lib-core/src/xml/* crates/libs/xml/src/
git mv crates/libs/lib-core/tests/xml.rs crates/libs/xml/tests/xml.rs
git mv crates/libs/lib-core/tests/xml crates/libs/xml/tests/xml
git mv crates/libs/lib-core/tests/import.rs crates/libs/xml/tests/import.rs
git mv crates/libs/lib-core/tests/import crates/libs/xml/tests/import
```

Rename the moved `src/mod.rs` to the crate root:

```bash
git mv crates/libs/xml/src/mod.rs crates/libs/xml/src/lib.rs
```

- [ ] **Step 4: Convert only cross-crate paths in the moved implementation**

Apply these path rules throughout `crates/libs/xml/src`:

```rust
// Before, when the symbol belongs to the domain/persistence crate
use crate::ctx::Ctx;
use crate::model::ModelManager;

// After
use lib_core::ctx::Ctx;
use lib_core::model::ModelManager;
```

Keep XML-internal paths crate-local:

```rust
use crate::error::Error;
use crate::export::policy::should_clear_null_flavor_on_value;
use crate::Result;
```

Convert `crate::e2b` and `crate::regulatory` to `lib_core::e2b` and `lib_core::regulatory`. Convert moved test imports from `lib_core::xml` to `xml`.

- [ ] **Step 5: Remove the old module declaration and XML-only dependencies from lib-core**

Delete this declaration from `crates/libs/lib-core/src/lib.rs`:

```rust
pub mod xml;
```

Remove `quick-xml` and `libxml` from `crates/libs/lib-core/Cargo.toml`. Remove any other dependency only after this command proves it has no non-XML consumer:

```bash
rg -n 'quick_xml|libxml' crates/libs/lib-core/src crates/libs/lib-core/tests
```

Expected: no matches.

- [ ] **Step 6: Compile the extracted boundary and moved tests**

Run:

```bash
cargo check -p lib-core -p xml
cargo test -p xml --test xml --test import --no-run
```

Expected: both packages and both moved test targets compile. Do not require the full workspace to compile until downstream consumers are migrated.

- [ ] **Step 7: Commit the engine extraction**

```bash
git add Cargo.toml Cargo.lock crates/libs/lib-core crates/libs/xml
git commit -m "refactor: extract xml engine crate"
```

---

### Task 2: Split Generic XML Validation from Validator Business Projection

**Files:**
- Create: `crates/libs/xml/src/validation.rs`
- Modify: `crates/libs/xml/src/lib.rs`
- Modify: `crates/libs/validator/src/xml/mod.rs`
- Modify: `crates/libs/validator/src/xml/ich_profile.rs`
- Modify: `crates/libs/validator/src/{c_safety_report_policy.rs,e_reaction_policy.rs,g_drug_policy.rs}`
- Modify: `crates/libs/validator/Cargo.toml`
- Move/split as needed: `crates/libs/validator/tests/xml/xml_schema_business_ci.rs`
- Modify: `crates/libs/validator/tests/xml/xml_validation.rs`

**Interfaces:**
- Consumes: `xml::{Error, Result, XmlValidationError, XmlValidationReport}` and `xml::export::policy`.
- Produces: `xml::validation::{XmlValidatorConfig, default_xsd_path, should_skip_xml_validation, validate_e2b_xml, validate_e2b_xml_basic, validate_e2b_xml_xsd}`; retains `validator::xml::validate_e2b_xml_business`.

- [ ] **Step 1: Add a compile-time ownership test before moving functions**

Add `crates/libs/xml/tests/validation_api.rs`:

```rust
use xml::validation::{
    default_xsd_path, should_skip_xml_validation, validate_e2b_xml,
    validate_e2b_xml_basic, XmlValidatorConfig,
};

#[test]
fn generic_validation_api_is_owned_by_xml() {
    let config = XmlValidatorConfig { xsd_path: None, ..Default::default() };
    let report = validate_e2b_xml_basic(b"<MCCI_IN200100UV01/>", Some(config))
        .expect("basic validation");
    assert!(report.ok);
    let _ = default_xsd_path();
    let _ = should_skip_xml_validation();
    let _ = validate_e2b_xml;
}
```

- [ ] **Step 2: Run the ownership test and verify the API is missing**

Run:

```bash
cargo test -p xml --test validation_api
```

Expected: FAIL because `xml::validation` does not exist.

- [ ] **Step 3: Extract generic validation and keep catalog-driven validation in validator**

Move these definitions from `validator/src/xml/mod.rs` into `xml/src/validation.rs`:

```rust
pub struct XmlValidatorConfig { /* existing fields unchanged */ }
pub fn default_xsd_path() -> Option<PathBuf>
pub fn validate_e2b_xml(xml: &[u8], config: Option<XmlValidatorConfig>) -> Result<XmlValidationReport>
pub fn validate_e2b_xml_basic(xml: &[u8], config: Option<XmlValidatorConfig>) -> Result<XmlValidationReport>
pub fn should_skip_xml_validation() -> bool
pub fn validate_e2b_xml_xsd(xml: &[u8], xsd_path: &Path) -> Result<Vec<XmlValidationError>>
```

Expose the module and convenience re-exports in `xml/src/lib.rs`:

```rust
pub mod validation;
pub use validation::{
    default_xsd_path, should_skip_xml_validation, validate_e2b_xml,
    validate_e2b_xml_basic, XmlValidatorConfig,
};
```

Keep `validate_e2b_xml_business`, `validate_e2b_xml_rules`, the authority profiles, section collectors, and `shared_specs` in `validator/src/xml`. Replace its generic implementation with imports:

```rust
use xml::validation::{validate_e2b_xml_basic, XmlValidatorConfig};
use xml::{Result, XmlValidationError, XmlValidationReport};
```

- [ ] **Step 4: Add the one-way validator dependency and update policy imports**

Add to `crates/libs/validator/Cargo.toml`:

```toml
xml = { path = "../xml" }
```

Change the three case-policy modules and `validator/src/xml/ich_profile.rs` from `lib_core::xml::export::policy` to:

```rust
use xml::export::policy::should_clear_null_flavor_on_value;
```

Preserve each file's existing imported symbols and `pub use` visibility.

- [ ] **Step 5: Split test ownership without losing mixed schema/business coverage**

Move pure schema/basic test cases to `crates/libs/xml/tests/xml_validation.rs` and import from `xml::validation`. Keep catalog-driven business cases in validator tests and use both owners explicitly:

```rust
use validator::xml::validate_e2b_xml_business;
use xml::validation::{validate_e2b_xml, XmlValidatorConfig};
```

Do not change fixture paths or expected validation messages.

- [ ] **Step 6: Run focused validation and policy tests**

Run:

```bash
cargo test -p xml --test validation_api --test xml_validation
cargo test -p validator --lib
cargo test -p validator --test xml_schema_business_ci --test xml_validation
```

Expected: all tests pass; `validate_e2b_xml_business` is still exported by validator, and generic schema/basic validation is exported by `xml`.

- [ ] **Step 7: Verify the dependency direction and commit**

Run:

```bash
cargo tree -p xml | rg '^validator ' && exit 1 || true
cargo tree -p validator | rg '^xml |lib-core'
```

Expected: the first command finds no `validator` dependency below `xml`; the second shows validator consuming both `xml` and `lib-core`.

```bash
git add Cargo.lock crates/libs/xml crates/libs/validator
git commit -m "refactor: move document validation into xml"
```

---

### Task 3: Migrate REST Error Conversion and Web-Server Call Sites

**Files:**
- Modify: `crates/libs/lib-rest-core/Cargo.toml`
- Modify: `crates/libs/lib-rest-core/src/error.rs`
- Modify: `crates/services/web-server/Cargo.toml`
- Modify: `crates/services/web-server/src/submission.rs`
- Modify: `crates/services/web-server/src/web/rest/case_export_rest.rs`
- Modify: `crates/services/web-server/src/web/rest/import_rest.rs`
- Modify: web-server XML integration tests importing `lib_core::xml` or `validator::xml` generic functions

**Interfaces:**
- Consumes: XML import/export and schema/basic validation from `xml`; business XML validation from `validator`.
- Produces: unchanged HTTP endpoints, submission workflow, and `lib_rest_core::Error::Xml(xml::Error)` conversion.

- [ ] **Step 1: Add direct crate dependencies**

Add to both `crates/libs/lib-rest-core/Cargo.toml` and `crates/services/web-server/Cargo.toml` with the correct relative path for each manifest:

```toml
xml = { path = "../../libs/xml" }
```

For `lib-rest-core`, whose manifest is already under `crates/libs`, use:

```toml
xml = { path = "../../libs/xml" }
```

Before editing, resolve the path from each manifest directory and correct it if necessary with:

```bash
test -f crates/libs/lib-rest-core/../../libs/xml/Cargo.toml
test -f crates/services/web-server/../../libs/xml/Cargo.toml
```

- [ ] **Step 2: Migrate error ownership**

Change `crates/libs/lib-rest-core/src/error.rs`:

```rust
#[from]
Xml(xml::Error),
```

Run:

```bash
cargo check -p lib-rest-core
```

Expected: PASS and `Error::from(xml::Error)` remains available.

- [ ] **Step 3: Update import/export call sites by owner**

Use these ownership rules:

```rust
use xml::export::{export_case_xml, export_case_xml_with_options, ExportXmlOptions};
use xml::import::{import_e2b_xml, import_e2b_xml_unvalidated, CImportSettings, XmlImportRequest};
use xml::validation::{
    should_skip_xml_validation, validate_e2b_xml, validate_e2b_xml_basic,
};
use validator::xml::validate_e2b_xml_business;
```

Preserve existing function calls, blocking-task boundaries, error mapping, and submission ordering.

- [ ] **Step 4: Update web integration-test imports**

Replace `lib_core::xml` imports with `xml`. In tests that use both schema and business checks, import them from their distinct owners:

```rust
use validator::xml::validate_e2b_xml_business;
use xml::validation::{validate_e2b_xml, XmlValidatorConfig};
```

Do not modify assertions or fixture paths.

- [ ] **Step 5: Compile all migrated consumers**

Run:

```bash
cargo check -p lib-rest-core -p web-server --all-targets
```

Expected: PASS with no unresolved `lib_core::xml` or misplaced `validator::xml` generic-validation imports.

- [ ] **Step 6: Run the focused API and submission regressions**

Run:

```bash
cargo test -p web-server --tests --no-run
cargo test -p web-server --lib submission::tests
```

Expected: all registered integration targets compile and submission unit tests pass. The files under `tests/xml/` are nested support modules rather than standalone Cargo test target names, so do not pass their filenames to `--test` unless a top-level target is added.

- [ ] **Step 7: Commit consumer migration**

```bash
git add Cargo.lock crates/libs/lib-rest-core crates/services/web-server
git commit -m "refactor: migrate xml crate consumers"
```

---

### Task 4: Remove Legacy References and Verify the Workspace

**Files:**
- Modify: any remaining source/test file reported by the ownership scans
- Modify: `Cargo.lock`

**Interfaces:**
- Consumes: completed crate boundaries from Tasks 1–3.
- Produces: a cycle-free workspace with no `lib_core::xml` compatibility surface.

- [ ] **Step 1: Run strict ownership scans**

Run:

```bash
rg -n 'lib_core::xml|crate::xml' crates/libs/lib-core crates/libs/validator crates/libs/lib-rest-core crates/services/web-server crates/libs/xml
rg -n 'quick_xml|libxml' crates/libs/lib-core
test ! -e crates/libs/lib-core/src/xml
```

Expected: no `lib_core::xml` matches, no XML-library matches inside `lib-core`, and no old XML directory. `crate::xml` matches are allowed only inside validator for its retained local business-validation module; inspect each match rather than replacing it mechanically.

- [ ] **Step 2: Check formatting and every package**

Run:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
```

Expected: both commands pass. If formatting fails, run `cargo fmt --all`, inspect the diff, then rerun the check.

- [ ] **Step 3: Run library and XML-focused tests**

Run:

```bash
cargo test -p xml
cargo test -p validator
cargo test -p lib-core
cargo test -p lib-rest-core
```

Expected: all tests pass with XML/import tests now owned by `xml` and case/business validation tests still owned by `validator`.

- [ ] **Step 4: Run web-server integration coverage**

Run:

```bash
cargo test -p web-server --lib
cargo test -p web-server --tests
```

Expected: all web API, import/export round-trip, submission, and validation integration tests pass. Tests requiring configured PostgreSQL must use the repository's documented isolated test database runner; do not point them at a shared or production database.

- [ ] **Step 5: Verify the final dependency graph**

Run:

```bash
cargo tree -p xml | rg 'xml v|lib-core'
cargo tree -p validator | rg 'validator v|xml v|lib-core'
cargo tree -p lib-core | rg '^xml ' && exit 1 || true
```

Expected: `xml` includes `lib-core`; validator includes both; the final command proves `lib-core` does not consume `xml`.

- [ ] **Step 6: Review the final diff for scope discipline**

Run:

```bash
git status --short
git diff --stat HEAD~3
git diff --check
```

Expected: changes are limited to the XML extraction, validation ownership split, manifests, call sites, and moved tests. Submission implementation and database-backed import/export behavior are unchanged.

- [ ] **Step 7: Commit any final cleanup**

If Step 1–6 required tracked cleanup changes:

```bash
git add Cargo.lock Cargo.toml crates
git commit -m "test: verify xml crate extraction"
```

If no tracked cleanup changes remain, do not create an empty commit.
