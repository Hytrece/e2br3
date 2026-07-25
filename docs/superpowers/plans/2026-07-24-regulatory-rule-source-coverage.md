# Regulatory Rule Source Coverage Implementation Plan

> This is the infrastructure and LR-pilot companion plan. The required
> repository-wide rollout continues in
> `docs/superpowers/plans/2026-07-24-all-editor-regulatory-coverage.md`; LR
> completion alone does not satisfy the overall goal.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a non-runtime coverage gate proving that regulatory prose for an audited surface is reviewed and mapped to an existing business rule, portable constraint, guidance decision, or explicit deferred gap.

**Architecture:** Keep the canonical catalog, portable constraints, section rule tables, and evaluators unchanged. Add a reviewed JSON crosswalk, validate its source/hash/editor-completion semantics in Python, and validate referenced executable codes against the compiled Rust catalogs in a focused unit test. Use LR as the first real fixture; the master rollout plan applies the infrastructure to every remaining surface.

**Tech Stack:** Python 3 `unittest`, JSON, Rust 2021, `serde`/`serde_json`, existing validator catalog and portable binding APIs.

## Global Constraints

- Do not parse regulatory prose into executable validation automatically.
- Do not replace or redesign `VALIDATION_RULES`, condition bindings, value policies, portable bindings, section rule tables, or shared evaluators.
- The crosswalk is build/test metadata only and must not be loaded by the runtime validation path.
- `business_rule` references must exist in the compiled `CaseValidate` catalog.
- `constraint` references must exist in both the portable catalog and a portable field binding.
- `guidance` and `deferred` require a non-empty reason.
- A deferred requirement for an audited editor element prevents that registry row from being `complete`.
- Preserve unrelated dirty work and stage only files named by each task.

---

### Task 1: Finalize the Regional Audit Baseline

**Files:**
- Modify: `registry/sections/c-safety-report.json`
- Modify: `registry/sections/n-message-header.json`
- Test: `registry/tools/test_validate.py`

**Interfaces:**
- Consumes: Current RP/SD/LR regional audit findings.
- Produces: Honest `incomplete` registry states that the new deferred gate will preserve.

- [ ] **Step 1: Run the focused baseline tests**

Run:

```bash
python3 -m unittest \
  registry.tools.test_validate.RegistryValidatorTests.test_certified_editor_sections_track_regional_business_rule_gaps \
  registry.tools.test_validate.RegistryValidatorTests.test_lr_editor_contract_tracks_regional_business_validation_gap
```

Expected: `Ran 2 tests` and `OK`.

- [ ] **Step 2: Run the affected strict contracts**

Run:

```bash
python3 registry/tools/validate.py --strict-editor-contract RP
python3 registry/tools/validate.py --strict-editor-contract SD
python3 registry/tools/validate.py --strict-editor-contract LR
```

Expected: all three commands print `registry validation passed`.

- [ ] **Step 3: Commit only the audit baseline**

```bash
git add \
  registry/sections/c-safety-report.json \
  registry/sections/n-message-header.json \
  registry/tools/test_validate.py
git commit -m "fix: classify regional editor rule gaps"
```

---

### Task 2: Add the Crosswalk Model and Source Integrity Validator

**Files:**
- Create: `registry/rule-source-coverage.schema.json`
- Create: `registry/tools/rule_source_coverage.py`
- Create: `registry/tools/test_rule_source_coverage.py`

**Interfaces:**
- Consumes:
  - `load_coverage(root: Path) -> dict[str, Any]`
  - `load_rule_prose(root: Path) -> dict[tuple[str, str], str]`
- Produces:
  - `source_hash(text: str) -> str`
  - `validate_coverage_structure(root: Path, result: Any) -> dict[tuple[str, str], dict[str, Any]]`
- Later tasks use the returned `(authority, element)` index for editor completion and Rust reference checks.

- [ ] **Step 1: Write failing tests for hashes, stale sources, excerpts, and dispositions**

Create `registry/tools/test_rule_source_coverage.py` with temporary registry fixtures and these concrete cases:

```python
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

import sys
sys.path.insert(0, str(Path(__file__).resolve().parent))

import rule_source_coverage
import validate


class RuleSourceCoverageTests(unittest.TestCase):
    def write_rules(self, root: Path, authority: str, rules: dict[str, str]) -> None:
        rules_dir = root / "dictionary" / "rules"
        rules_dir.mkdir(parents=True, exist_ok=True)
        (rules_dir / f"{authority.lower()}.json").write_text(
            json.dumps({"authority": authority, "source": "fixture", "rules": rules}),
            encoding="utf-8",
        )

    def write_coverage(self, root: Path, sources: list[dict]) -> None:
        (root / "rule-source-coverage.json").write_text(
            json.dumps({"version": 1, "auditedPages": ["LR"], "sources": sources}),
            encoding="utf-8",
        )

    def requirement(self, disposition: str, **extra: object) -> dict:
        return {
            "id": "filename-media-type-match",
            "sourceExcerpt": "file extension",
            "disposition": disposition,
            **extra,
        }

    def test_source_hash_is_stable_fnv1a64(self) -> None:
        self.assertEqual(
            "fnv1a64:4a5f58db9609295e",
            rule_source_coverage.source_hash(
                'VAERS: The location of the attachment file name must follow the '
                '<text mediaType> attribute based upon the following example:  '
                'text mediaType="text/plain" representation="B64">\\n'
                '        <reference value="SUMMARY OF CLINICAL HISTORY.txt"/>. '
                'If the file extension in the filename does not match the media '
                'type, the ICSR file will be rejected.'
            ),
        )

    def test_rejects_stale_source_hash(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_rules(root, "FDA", {"C.4.r.2": "file extension mismatch"})
            self.write_coverage(root, [{
                "authority": "FDA",
                "element": "C.4.r.2",
                "sourceHash": "fnv1a64:0000000000000000",
                "requirements": [self.requirement("deferred", reason="filename missing")],
            }])
            result = validate.ValidationResult()
            rule_source_coverage.validate_coverage_structure(root, result)
            self.assertIn("stale sourceHash", "\\n".join(result.errors))

    def test_rejects_excerpt_absent_from_prose(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prose = "attachment MIME must match"
            self.write_rules(root, "FDA", {"C.4.r.2": prose})
            self.write_coverage(root, [{
                "authority": "FDA",
                "element": "C.4.r.2",
                "sourceHash": rule_source_coverage.source_hash(prose),
                "requirements": [self.requirement("deferred", reason="filename missing")],
            }])
            result = validate.ValidationResult()
            rule_source_coverage.validate_coverage_structure(root, result)
            self.assertIn("sourceExcerpt is not present", "\\n".join(result.errors))

    def test_disposition_requires_codes_or_reason(self) -> None:
        cases = [
            ("business_rule", "ruleCodes"),
            ("constraint", "ruleCodes"),
            ("guidance", "reason"),
            ("deferred", "reason"),
        ]
        for disposition, missing in cases:
            with self.subTest(disposition=disposition), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                prose = "file extension"
                self.write_rules(root, "FDA", {"C.4.r.2": prose})
                self.write_coverage(root, [{
                    "authority": "FDA",
                    "element": "C.4.r.2",
                    "sourceHash": rule_source_coverage.source_hash(prose),
                    "requirements": [self.requirement(disposition)],
                }])
                result = validate.ValidationResult()
                rule_source_coverage.validate_coverage_structure(root, result)
                self.assertIn(f"requires {missing}", "\\n".join(result.errors))
```

- [ ] **Step 2: Run the new tests and verify RED**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
```

Expected: import failure for missing `rule_source_coverage`.

- [ ] **Step 3: Add the JSON schema**

Create `registry/rule-source-coverage.schema.json` with:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "type": "object",
  "required": ["version", "auditedPages", "sources"],
  "additionalProperties": false,
  "properties": {
    "version": { "const": 1 },
    "auditedPages": {
      "type": "array",
      "uniqueItems": true,
      "items": { "type": "string", "minLength": 1 }
    },
    "sources": {
      "type": "array",
      "items": {
        "type": "object",
        "required": ["authority", "element", "sourceHash", "requirements"],
        "additionalProperties": false,
        "properties": {
          "authority": { "enum": ["ICH", "FDA", "MFDS"] },
          "element": { "type": "string", "minLength": 1 },
          "sourceHash": { "pattern": "^fnv1a64:[0-9a-f]{16}$" },
          "requirements": {
            "type": "array",
            "minItems": 1,
            "items": {
              "type": "object",
              "required": ["id", "sourceExcerpt", "disposition"],
              "additionalProperties": false,
              "properties": {
                "id": { "type": "string", "minLength": 1 },
                "sourceExcerpt": { "type": "string", "minLength": 1 },
                "disposition": {
                  "enum": ["business_rule", "constraint", "guidance", "deferred"]
                },
                "ruleCodes": {
                  "type": "array",
                  "minItems": 1,
                  "uniqueItems": true,
                  "items": { "type": "string", "minLength": 1 }
                },
                "reason": { "type": "string", "minLength": 1 }
              }
            }
          }
        }
      }
    }
  }
}
```

- [ ] **Step 4: Implement the Python validator**

Create `registry/tools/rule_source_coverage.py` with these exact public functions and constants:

```python
from __future__ import annotations

import json
from pathlib import Path
from typing import Any

AUTHORITIES = {"ICH", "FDA", "MFDS"}
DISPOSITIONS = {"business_rule", "constraint", "guidance", "deferred"}


def source_hash(text: str) -> str:
    value = 14695981039346656037
    for byte in text.strip().encode("utf-8"):
        value ^= byte
        value = (value * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return f"fnv1a64:{value:016x}"


def load_rule_prose(root: Path) -> dict[tuple[str, str], str]:
    prose: dict[tuple[str, str], str] = {}
    for path in sorted((root / "dictionary" / "rules").glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        authority = payload["authority"]
        for element, text in payload["rules"].items():
            prose[(authority, element)] = text
    return prose


def load_coverage(root: Path) -> dict[str, Any]:
    return json.loads(
        (root / "rule-source-coverage.json").read_text(encoding="utf-8")
    )


def validate_coverage_structure(
    root: Path,
    result: Any,
) -> dict[tuple[str, str], dict[str, Any]]:
    prose = load_rule_prose(root)
    try:
        payload = load_coverage(root)
    except (OSError, json.JSONDecodeError, KeyError) as error:
        result.add(f"rule source coverage could not be loaded: {error}")
        return {}

    if payload.get("version") != 1:
        result.add("rule source coverage version must be 1")
    if not isinstance(payload.get("auditedPages"), list):
        result.add("rule source coverage auditedPages must be an array")
    sources = payload.get("sources")
    if not isinstance(sources, list):
        result.add("rule source coverage sources must be an array")
        return {}

    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for source in sources:
        if not isinstance(source, dict):
            result.add("rule source coverage entry must be an object")
            continue
        key = (source.get("authority"), source.get("element"))
        if key in indexed:
            result.add(f"duplicate rule source coverage entry {key}")
            continue
        indexed[key] = source
        text = prose.get(key)
        if text is None:
            result.add(f"orphaned rule source coverage entry {key}")
            continue
        if source.get("sourceHash") != source_hash(text):
            result.add(f"{key} has stale sourceHash")
        requirements = source.get("requirements")
        if not isinstance(requirements, list) or not requirements:
            result.add(f"{key} requirements must be a non-empty array")
            continue
        seen_ids: set[str] = set()
        for requirement in requirements:
            requirement_id = requirement.get("id")
            if requirement_id in seen_ids:
                result.add(f"{key} duplicate requirement id {requirement_id}")
            seen_ids.add(requirement_id)
            excerpt = requirement.get("sourceExcerpt")
            if not isinstance(excerpt, str) or excerpt not in text:
                result.add(f"{key}/{requirement_id} sourceExcerpt is not present")
            disposition = requirement.get("disposition")
            if disposition not in DISPOSITIONS:
                result.add(f"{key}/{requirement_id} has invalid disposition")
            if disposition in {"business_rule", "constraint"}:
                codes = requirement.get("ruleCodes")
                if not isinstance(codes, list) or not codes:
                    result.add(f"{key}/{requirement_id} requires ruleCodes")
            if disposition in {"guidance", "deferred"}:
                reason = requirement.get("reason")
                if not isinstance(reason, str) or not reason.strip():
                    result.add(f"{key}/{requirement_id} requires reason")
    return indexed
```

- [ ] **Step 5: Run the tests and verify GREEN**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
```

Expected: four tests pass.

- [ ] **Step 6: Commit the source integrity unit**

```bash
git add \
  registry/rule-source-coverage.schema.json \
  registry/tools/rule_source_coverage.py \
  registry/tools/test_rule_source_coverage.py
git commit -m "feat: validate regulatory rule source coverage"
```

---

### Task 3: Gate Audited Editor Completion in Registry Validation

**Files:**
- Create: `registry/rule-source-coverage.json`
- Modify: `registry/tools/rule_source_coverage.py`
- Modify: `registry/tools/validate.py`
- Modify: `registry/tools/test_rule_source_coverage.py`

**Interfaces:**
- Consumes:
  - `validate_coverage_structure(root, result)`
  - editor contracts from `registry/editor-contracts/*.json`
  - flattened registry rows from `validate_registry`
- Produces:
  - `validate_editor_coverage(root, registry_rows, coverage, result) -> None`

- [ ] **Step 1: Add an empty production crosswalk**

Create:

```json
{
  "version": 1,
  "auditedPages": [],
  "sources": []
}
```

- [ ] **Step 2: Write failing tests for missing coverage and deferred completion**

Add tests using real-shaped contract and registry fixtures:

```python
    def write_contract(self, root: Path, page: str, code: str) -> None:
        contracts = root / "editor-contracts"
        contracts.mkdir(parents=True, exist_ok=True)
        (contracts / f"{page.lower()}.json").write_text(
            json.dumps({
                "pageId": page,
                "registryFile": "sections/c-safety-report.json",
                "fields": [{"code": code, "authority": "ICH"}],
            }),
            encoding="utf-8",
        )

    def test_audited_page_requires_every_authority_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_rules(root, "ICH", {"C.4.r.2": "attach documents"})
            self.write_rules(root, "FDA", {"C.4.r.2": "file extension"})
            self.write_coverage(root, [])
            self.write_contract(root, "LR", "C.4.r.2")
            result = validate.ValidationResult()
            coverage = rule_source_coverage.validate_coverage_structure(root, result)
            rule_source_coverage.validate_editor_coverage(
                root,
                [{"id": "C.4.r.2", "e2br3_code": "C.4.r.2",
                  "editor_page": "LR", "status": "incomplete"}],
                coverage,
                result,
            )
            errors = "\\n".join(result.errors)
            self.assertIn("missing ICH/C.4.r.2 source coverage", errors)
            self.assertIn("missing FDA/C.4.r.2 source coverage", errors)

    def test_deferred_requirement_rejects_complete_row(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prose = "file extension"
            self.write_rules(root, "FDA", {"C.4.r.2": prose})
            self.write_coverage(root, [{
                "authority": "FDA",
                "element": "C.4.r.2",
                "sourceHash": rule_source_coverage.source_hash(prose),
                "requirements": [self.requirement("deferred", reason="filename missing")],
            }])
            self.write_contract(root, "LR", "C.4.r.2")
            result = validate.ValidationResult()
            coverage = rule_source_coverage.validate_coverage_structure(root, result)
            rule_source_coverage.validate_editor_coverage(
                root,
                [{"id": "C.4.r.2", "e2br3_code": "C.4.r.2",
                  "editor_page": "LR", "status": "complete"}],
                coverage,
                result,
            )
            self.assertIn(
                "C.4.r.2 cannot be complete while FDA/C.4.r.2 is deferred",
                "\\n".join(result.errors),
            )
```

- [ ] **Step 3: Run the two tests and verify RED**

Run:

```bash
python3 -m unittest \
  registry.tools.test_rule_source_coverage.RuleSourceCoverageTests.test_audited_page_requires_every_authority_source \
  registry.tools.test_rule_source_coverage.RuleSourceCoverageTests.test_deferred_requirement_rejects_complete_row
```

Expected: failure because `validate_editor_coverage` does not exist.

- [ ] **Step 4: Implement the editor gate**

Add to `rule_source_coverage.py`:

```python
def validate_editor_coverage(
    root: Path,
    registry_rows: list[dict[str, Any]],
    coverage: dict[tuple[str, str], dict[str, Any]],
    result: Any,
) -> None:
    payload = load_coverage(root)
    audited_pages = set(payload.get("auditedPages", []))
    prose = load_rule_prose(root)
    rows_by_code = {
        row.get("e2br3_code"): row
        for row in registry_rows
        if isinstance(row.get("e2br3_code"), str)
    }

    for page in sorted(audited_pages):
        contract_path = root / "editor-contracts" / f"{page.lower()}.json"
        contract = json.loads(contract_path.read_text(encoding="utf-8"))
        for field in contract.get("fields", []):
            code = field.get("code")
            row = rows_by_code.get(code)
            if row is None or row.get("local_only") is True:
                continue
            for authority in sorted(AUTHORITIES):
                key = (authority, code)
                if key not in prose:
                    continue
                source = coverage.get(key)
                if source is None:
                    result.add(f"missing {authority}/{code} source coverage")
                    continue
                deferred = any(
                    requirement.get("disposition") == "deferred"
                    for requirement in source.get("requirements", [])
                )
                if deferred and row.get("status") == "complete":
                    result.add(
                        f"{code} cannot be complete while "
                        f"{authority}/{code} is deferred"
                    )
```

- [ ] **Step 5: Integrate it into `validate_registry`**

Import `rule_source_coverage` beside `editor_contract`. After registry rows are
loaded, call:

```python
    coverage = rule_source_coverage.validate_coverage_structure(root, result)
    rule_source_coverage.validate_editor_coverage(
        root,
        registry_rows,
        coverage,
        result,
    )
```

Do not put the call behind `--strict-editor-contract`; default registry
validation must enforce every page named in `auditedPages`.

- [ ] **Step 6: Run focused tests and verify GREEN**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
```

Expected: six tests pass.

- [ ] **Step 7: Verify default registry validation remains green**

Run:

```bash
python3 registry/tools/validate.py
```

Expected: `registry validation passed`; the empty `auditedPages` list does not
make an editor certification claim.

- [ ] **Step 8: Commit the editor gate**

```bash
git add \
  registry/rule-source-coverage.json \
  registry/tools/rule_source_coverage.py \
  registry/tools/test_rule_source_coverage.py \
  registry/tools/validate.py
git commit -m "feat: gate editor completion on source coverage"
```

---

### Task 4: Validate Executable References Against the Existing Rust Catalogs

**Files:**
- Create: `crates/libs/validator/src/rule_source_coverage_tests.rs`
- Modify: `crates/libs/validator/src/lib.rs`
- Modify temporarily, then restore: `registry/rule-source-coverage.json`

**Interfaces:**
- Consumes:
  - `canonical_rules_for_phase(ValidationPhase::CaseValidate)`
  - `portable_constraints()`
  - `portable_field_bindings()`
- Produces: test-only proof that every executable crosswalk reference exists in the current compiled sources of truth.

- [ ] **Step 1: Write the failing Rust test**

Create `rule_source_coverage_tests.rs`:

```rust
use crate::{
    canonical_rules_for_phase, portable_constraints, portable_field_bindings,
    ValidationPhase,
};
use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Coverage {
    sources: Vec<SourceCoverage>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceCoverage {
    requirements: Vec<Requirement>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Requirement {
    id: String,
    disposition: String,
    #[serde(default)]
    rule_codes: Vec<String>,
}

#[test]
fn executable_rule_source_references_exist_in_compiled_catalogs() {
    let coverage: Coverage = serde_json::from_str(include_str!(
        "../../../../registry/rule-source-coverage.json"
    ))
    .expect("rule source coverage JSON parses");
    let business = canonical_rules_for_phase(ValidationPhase::CaseValidate)
        .into_iter()
        .map(|rule| rule.code.to_string())
        .collect::<BTreeSet<_>>();
    let portable = portable_constraints()
        .into_iter()
        .map(|rule| rule.code)
        .collect::<BTreeSet<_>>();
    let bound = portable_field_bindings()
        .into_iter()
        .flat_map(|binding| binding.rule_codes.iter().copied())
        .collect::<BTreeSet<_>>();

    for source in coverage.sources {
        for requirement in source.requirements {
            match requirement.disposition.as_str() {
                "business_rule" => {
                    for code in requirement.rule_codes {
                        assert!(
                            business.contains(&code),
                            "{} references missing business rule {}",
                            requirement.id,
                            code
                        );
                    }
                }
                "constraint" => {
                    for code in requirement.rule_codes {
                        assert!(
                            portable.contains(&code),
                            "{} references missing portable constraint {}",
                            requirement.id,
                            code
                        );
                        assert!(
                            bound.contains(code.as_str()),
                            "{} references unbound portable constraint {}",
                            requirement.id,
                            code
                        );
                    }
                }
                "guidance" | "deferred" => {}
                other => panic!("unsupported source disposition {other}"),
            }
        }
    }
}
```

Add to `lib.rs`:

```rust
#[cfg(test)]
mod rule_source_coverage_tests;
```

Temporarily change the empty crosswalk to reference
`FDA.C.4.r.2.NOT.REGISTERED` as a `business_rule`.

- [ ] **Step 2: Run the Rust test and verify RED**

Run:

```bash
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
```

Expected: failure containing
`references missing business rule FDA.C.4.r.2.NOT.REGISTERED`.

- [ ] **Step 3: Restore the empty crosswalk and verify GREEN**

Restore:

```json
{
  "version": 1,
  "auditedPages": [],
  "sources": []
}
```

Run the same command. Expected: one test passes.

- [ ] **Step 4: Commit the compiled-reference gate**

```bash
git add \
  crates/libs/validator/src/lib.rs \
  crates/libs/validator/src/rule_source_coverage_tests.rs
git commit -m "test: verify executable rule source references"
```

---

### Task 5: Seed and Enforce LR Source Coverage

**Files:**
- Modify: `registry/rule-source-coverage.json`
- Test: `registry/tools/test_rule_source_coverage.py`
- Test: `crates/libs/validator/src/rule_source_coverage_tests.rs`

**Interfaces:**
- Consumes: The Python structural/editor gate and Rust executable-reference gate.
- Produces: The first real audited page, covering every ICH/FDA/MFDS prose source for LR.

- [ ] **Step 1: Replace the empty crosswalk with the reviewed LR entries**

Use exactly:

```json
{
  "version": 1,
  "auditedPages": ["LR"],
  "sources": [
    {
      "authority": "ICH",
      "element": "C.4.r.1",
      "sourceHash": "fnv1a64:915141c7b36208f9",
      "requirements": [
        {
          "id": "null-flavor-guide-reference",
          "sourceExcerpt": "further guidance on the use of nullFlavor",
          "disposition": "guidance",
          "reason": "The prose points to implementation guidance; allowed nullFlavors are structured dictionary constraints."
        }
      ]
    },
    {
      "authority": "MFDS",
      "element": "C.4.r.1",
      "sourceHash": "fnv1a64:8c29e91d7e740d0b",
      "requirements": [
        {
          "id": "reference-length-and-null-flavor",
          "sourceExcerpt": "입력값이 500자 이하인지 검증",
          "disposition": "constraint",
          "ruleCodes": [
            "ICH.C.4.r.1.LENGTH.MAX",
            "ICH.C.4.r.1.NULLFLAVOR.ALLOWED"
          ]
        }
      ]
    },
    {
      "authority": "ICH",
      "element": "C.4.r.2",
      "sourceHash": "fnv1a64:1be37ffe28af13c2",
      "requirements": [
        {
          "id": "regional-attachment-policy",
          "sourceExcerpt": "'Value Allowed' is defined by each region",
          "disposition": "guidance",
          "reason": "ICH delegates attachment value policy to each receiving region."
        }
      ]
    },
    {
      "authority": "FDA",
      "element": "C.4.r.2",
      "sourceHash": "fnv1a64:4a5f58db9609295e",
      "requirements": [
        {
          "id": "filename-media-type-match",
          "sourceExcerpt": "file extension in the filename does not match the media type",
          "disposition": "deferred",
          "reason": "Literature attachment filename is not preserved through UI, API, DB, or XML roundtrip."
        }
      ]
    },
    {
      "authority": "MFDS",
      "element": "C.4.r.2",
      "sourceHash": "fnv1a64:70bb2be38a12cb39",
      "requirements": [
        {
          "id": "attachment-format",
          "sourceExcerpt": "첨부된 파일이 허용하는 형식인지 검증",
          "disposition": "constraint",
          "ruleCodes": ["ICH.C.4.r.2.ALLOWED.VALUE"]
        }
      ]
    }
  ]
}
```

- [ ] **Step 2: Add a real-repository LR coverage test**

Add:

```python
    def test_real_lr_sources_are_fully_covered(self) -> None:
        root = Path(__file__).resolve().parents[1]
        result = validate.ValidationResult()
        coverage = rule_source_coverage.validate_coverage_structure(root, result)
        registry_rows = []
        for path in (root / "sections").glob("*.json"):
            registry_rows.extend(json.loads(path.read_text(encoding="utf-8")))
        rule_source_coverage.validate_editor_coverage(
            root, registry_rows, coverage, result
        )
        self.assertEqual([], result.errors)
```

- [ ] **Step 3: Run Python and Rust coverage tests**

Run:

```bash
python3 -m unittest registry.tools.test_rule_source_coverage -v
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
```

Expected: all Python tests pass and the one Rust test passes.

- [ ] **Step 4: Run LR strict validation**

Run:

```bash
python3 registry/tools/validate.py --strict-editor-contract LR
```

Expected: `registry validation passed`; `C.4.r.2` remains `incomplete`, so its
FDA deferred requirement is honest and does not create a false completion.

- [ ] **Step 5: Commit the LR rollout**

```bash
git add \
  registry/rule-source-coverage.json \
  registry/tools/test_rule_source_coverage.py
git commit -m "feat: audit LR regulatory rule sources"
```

---

### Task 6: Final Focused Verification

**Files:**
- Verify only; no production edits expected.

**Interfaces:**
- Consumes: All previous tasks.
- Produces: Evidence that the new gate works without changing runtime validation behavior.

- [ ] **Step 1: Run all coverage and registry tests**

Run:

```bash
python3 -m unittest \
  registry.tools.test_rule_source_coverage \
  registry.tools.test_validate.RegistryValidatorTests.test_certified_editor_sections_track_regional_business_rule_gaps \
  registry.tools.test_validate.RegistryValidatorTests.test_lr_editor_contract_tracks_regional_business_validation_gap
python3 registry/tools/validate.py
python3 registry/tools/validate.py --strict-editor-contract LR
```

Expected: all unit tests pass and both registry validations print
`registry validation passed`.

- [ ] **Step 2: Run only the affected Rust parity tests**

Run:

```bash
cargo test -p validator executable_rule_source_references_exist_in_compiled_catalogs --lib
cargo test -p validator implemented_case_registry_matches_case_validate_catalog --lib
cargo test -p validator every_portable_rule_is_bound_or_explicitly_excluded_once --lib
```

Expected: one matching test passes for each command.

- [ ] **Step 3: Check repository hygiene**

Run:

```bash
git diff --check
git status --short
```

Expected: no whitespace errors and no uncommitted files from this plan.
