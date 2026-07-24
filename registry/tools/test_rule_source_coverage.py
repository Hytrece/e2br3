from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import rule_source_coverage
import validate


class RuleSourceCoverageTests(unittest.TestCase):
    def write_rules(
        self,
        root: Path,
        authority: str,
        rules: dict[str, str],
    ) -> None:
        rules_dir = root / "dictionary" / "rules"
        rules_dir.mkdir(parents=True, exist_ok=True)
        (rules_dir / f"{authority.lower()}.json").write_text(
            json.dumps(
                {
                    "authority": authority,
                    "source": "fixture",
                    "rules": rules,
                }
            ),
            encoding="utf-8",
        )

    def write_coverage(self, root: Path, sources: list[dict]) -> None:
        (root / "rule-source-coverage.json").write_text(
            json.dumps(
                {
                    "version": 1,
                    "auditedPages": ["LR"],
                    "sources": sources,
                }
            ),
            encoding="utf-8",
        )

    def requirement(self, disposition: str, **extra: object) -> dict:
        return {
            "id": "filename-media-type-match",
            "sourceExcerpt": "file extension",
            "disposition": disposition,
            **extra,
        }

    def write_contract(self, root: Path, page: str, code: str) -> None:
        contracts = root / "editor-contracts"
        contracts.mkdir(parents=True, exist_ok=True)
        (contracts / f"{page.lower()}.json").write_text(
            json.dumps(
                {
                    "pageId": page,
                    "registryFile": "sections/c-safety-report.json",
                    "fields": [{"code": code, "authority": "ICH"}],
                }
            ),
            encoding="utf-8",
        )

    def test_source_hash_is_stable_fnv1a64(self) -> None:
        self.assertEqual(
            "fnv1a64:4a5f58db9609295e",
            rule_source_coverage.source_hash(
                'VAERS: The location of the attachment file name must follow the '
                "<text mediaType> attribute based upon the following example:  "
                'text mediaType="text/plain" representation="B64">\n'
                '        <reference value="SUMMARY OF CLINICAL HISTORY.txt"/>. '
                "If the file extension in the filename does not match the media "
                "type, the ICSR file will be rejected."
            ),
        )

    def test_rejects_stale_source_hash(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_rules(
                root,
                "FDA",
                {"C.4.r.2": "file extension mismatch"},
            )
            self.write_coverage(
                root,
                [
                    {
                        "authority": "FDA",
                        "element": "C.4.r.2",
                        "sourceHash": "fnv1a64:0000000000000000",
                        "requirements": [
                            self.requirement(
                                "deferred",
                                reason="filename missing",
                            )
                        ],
                    }
                ],
            )
            result = validate.ValidationResult()

            rule_source_coverage.validate_coverage_structure(root, result)

            self.assertIn("stale sourceHash", "\n".join(result.errors))

    def test_rejects_excerpt_absent_from_prose(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prose = "attachment MIME must match"
            self.write_rules(root, "FDA", {"C.4.r.2": prose})
            self.write_coverage(
                root,
                [
                    {
                        "authority": "FDA",
                        "element": "C.4.r.2",
                        "sourceHash": rule_source_coverage.source_hash(prose),
                        "requirements": [
                            self.requirement(
                                "deferred",
                                reason="filename missing",
                            )
                        ],
                    }
                ],
            )
            result = validate.ValidationResult()

            rule_source_coverage.validate_coverage_structure(root, result)

            self.assertIn(
                "sourceExcerpt is not present",
                "\n".join(result.errors),
            )

    def test_disposition_requires_codes_or_reason(self) -> None:
        cases = [
            ("business_rule", "ruleCodes"),
            ("constraint", "ruleCodes"),
            ("guidance", "reason"),
            ("deferred", "reason"),
        ]
        for disposition, missing in cases:
            with (
                self.subTest(disposition=disposition),
                tempfile.TemporaryDirectory() as tmp,
            ):
                root = Path(tmp)
                prose = "file extension"
                self.write_rules(root, "FDA", {"C.4.r.2": prose})
                self.write_coverage(
                    root,
                    [
                        {
                            "authority": "FDA",
                            "element": "C.4.r.2",
                            "sourceHash": (
                                rule_source_coverage.source_hash(prose)
                            ),
                            "requirements": [
                                self.requirement(disposition)
                            ],
                        }
                    ],
                )
                result = validate.ValidationResult()

                rule_source_coverage.validate_coverage_structure(root, result)

                self.assertIn(
                    f"requires {missing}",
                    "\n".join(result.errors),
                )

    def test_audited_page_requires_every_authority_source(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            self.write_rules(root, "ICH", {"C.4.r.2": "attach documents"})
            self.write_rules(root, "FDA", {"C.4.r.2": "file extension"})
            self.write_coverage(root, [])
            self.write_contract(root, "LR", "C.4.r.2")
            result = validate.ValidationResult()
            coverage = rule_source_coverage.validate_coverage_structure(
                root, result
            )

            rule_source_coverage.validate_editor_coverage(
                root,
                [
                    {
                        "id": "C.4.r.2",
                        "e2br3_code": "C.4.r.2",
                        "editor_page": "LR",
                        "status": "incomplete",
                    }
                ],
                coverage,
                result,
            )

            errors = "\n".join(result.errors)
            self.assertIn("missing ICH/C.4.r.2 source coverage", errors)
            self.assertIn("missing FDA/C.4.r.2 source coverage", errors)

    def test_deferred_requirement_rejects_complete_row(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            prose = "file extension"
            self.write_rules(root, "FDA", {"C.4.r.2": prose})
            self.write_coverage(
                root,
                [
                    {
                        "authority": "FDA",
                        "element": "C.4.r.2",
                        "sourceHash": rule_source_coverage.source_hash(prose),
                        "requirements": [
                            self.requirement(
                                "deferred",
                                reason="filename missing",
                            )
                        ],
                    }
                ],
            )
            self.write_contract(root, "LR", "C.4.r.2")
            result = validate.ValidationResult()
            coverage = rule_source_coverage.validate_coverage_structure(
                root, result
            )

            rule_source_coverage.validate_editor_coverage(
                root,
                [
                    {
                        "id": "C.4.r.2",
                        "e2br3_code": "C.4.r.2",
                        "editor_page": "LR",
                        "status": "complete",
                    }
                ],
                coverage,
                result,
            )

            self.assertIn(
                "C.4.r.2 cannot be complete while "
                "FDA/C.4.r.2 is deferred",
                "\n".join(result.errors),
            )

    def test_contract_audit_report_is_deterministic(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            ich_prose = "attachment file extension guidance"
            self.write_rules(root, "ICH", {"C.4.r.2": ich_prose})
            self.write_rules(
                root,
                "FDA",
                {"C.4.r.2": "file extension mismatch"},
            )
            self.write_coverage(
                root,
                [
                    {
                        "authority": "ICH",
                        "element": "C.4.r.2",
                        "sourceHash": rule_source_coverage.source_hash(
                            ich_prose
                        ),
                        "requirements": [
                            self.requirement(
                                "guidance",
                                reason="non-executable prose",
                            )
                        ],
                    }
                ],
            )
            self.write_contract(root, "LR", "C.4.r.2")

            report = rule_source_coverage.audit_contract_sources(root, "LR")

            self.assertEqual(
                [
                    {
                        "page": "LR",
                        "fieldId": "C.4.r.2",
                        "element": "C.4.r.2",
                        "authority": "FDA",
                        "coverage": "missing",
                        "disposition": "",
                    },
                    {
                        "page": "LR",
                        "fieldId": "C.4.r.2",
                        "element": "C.4.r.2",
                        "authority": "ICH",
                        "coverage": "covered",
                        "disposition": "guidance",
                    },
                ],
                report,
            )
