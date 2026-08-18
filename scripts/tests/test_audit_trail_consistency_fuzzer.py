#!/usr/bin/env python3

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import audit_trail_consistency_fuzzer as fuzzer


class AuditTrailConsistencyFuzzerTests(unittest.TestCase):
    def test_update_row_requires_part11_evidence(self) -> None:
        log = {
            "id": 7,
            "organization_id": "00000000-0000-0000-0000-000000000001",
            "user_id": "00000000-0000-0000-0000-000000000002",
            "created_at": "2026-08-18T00:00:00Z",
            "prev_hash": "a",
            "entry_hash": "b",
            "action": "UPDATE",
            "changed_fields": {"report_year": {"old": None, "new": "x"}},
        }
        self.assertTrue(fuzzer.audit_complete(log))
        self.assertTrue(fuzzer.field_matches(log, "report_year"))
        log["changed_fields"]["report_year"] = {"old": None}
        self.assertFalse(fuzzer.audit_complete(log))

    def test_parser_defaults_to_all_cross_cutting_surfaces(self) -> None:
        args = fuzzer.parser().parse_args(["--dry-run", "--seed", "17"])
        self.assertEqual(args.surfaces, "case,case-editor,presave,settings,notices,user")
        self.assertTrue(args.all_operations)
        self.assertTrue(args.dry_run)
        self.assertFalse(args.synthetic_user)

    def test_trigger_inventory_is_read_from_bootstrap(self) -> None:
        self.assertIn("cases", fuzzer.AUDIT_TRIGGER_TABLES)
        self.assertIn("users", fuzzer.AUDIT_TRIGGER_TABLES)
        self.assertGreater(len(fuzzer.AUDIT_TRIGGER_TABLES), 50)

    def test_multipart_file_has_one_upload_part(self) -> None:
        body, content_type = fuzzer.multipart_file("file", "probe.txt", b"probe")
        self.assertIn(b'filename="probe.txt"', body)
        self.assertIn(b"probe", body)
        self.assertIn("multipart/form-data; boundary=", content_type)

    def test_operation_coverage_is_incomplete_when_trigger_table_is_unseen(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "audit.jsonl"
            artifact.write_text("{}\n", encoding="utf-8")
            events = [
                {"surface": surface, "classification": "PASS", "audit_rows": 1,
                 "audit_rows_complete": True, "audit_actions": ["UPDATE"],
                 "audit_row_ids": [7], "table": "cases"}
                for surface in fuzzer.SURFACES
            ]
            events.extend([
                {"surface": "audit_chain", "classification": "PASS"},
                {"surface": "audit_mutation_guard", "classification": "PASS"},
            ])
            report = fuzzer.build_part11_report(
                events, list(fuzzer.SURFACES), artifact,
                expected_tables=("cases", "users"),
            )
            self.assertEqual(report["status"], "INCOMPLETE")
            coverage = next(item for item in report["controls"] if item["id"] == "11.10(e).operation-coverage")
            self.assertEqual(coverage["status"], "NOT_TESTED")
            self.assertIn("users", coverage["evidence"]["uncovered_tables"])

    def test_shared_oracle_handles_camel_case_and_storage_aliases(self) -> None:
        log = {
            "id": 8,
            "organizationId": "00000000-0000-0000-0000-000000000001",
            "userId": "00000000-0000-0000-0000-000000000002",
            "createdAt": "2026-08-18T00:00:00Z",
            "prevHash": "a",
            "entryHash": "b",
            "action": "UPDATE",
            "changedFields": {"country_code": {"old": "A", "new": "B"}},
        }
        self.assertTrue(fuzzer.audit_log_complete(log))
        self.assertTrue(fuzzer.audit_key_matches(log["changedFields"], "reporterCountry"))
        self.assertEqual(fuzzer.audit_field_key("reporterCountry"), "country_code")

    def test_part11_report_separates_technical_pass_from_unmeasured_procedure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "audit.jsonl"
            artifact.write_text("{}\n", encoding="utf-8")
            events = [
                {
                    "surface": surface,
                    "classification": "PASS",
                    "audit_rows": 1,
                    "audit_rows_complete": True,
                    "audit_actions": ["UPDATE"],
                    "audit_row_ids": [7],
                }
                for surface in fuzzer.SURFACES
            ]
            events.extend([
                {"surface": "audit_chain", "classification": "PASS"},
                {"surface": "audit_mutation_guard", "classification": "PASS"},
            ])
            report = fuzzer.build_part11_report(events, list(fuzzer.SURFACES), artifact)
            self.assertEqual(report["status"], "INCOMPLETE")
            self.assertEqual(report["claim"], "NOT_ESTABLISHED")
            self.assertIn("11.10(e).create-modify-delete", report["limitations"])
            self.assertIn("11.10(a).validation", report["limitations"])
            self.assertIn("11.10(e).retention", report["limitations"])
            events.extend([
                {
                    "surface": "presave-create",
                    "classification": "PASS",
                    "audit_rows": 1,
                    "audit_rows_complete": True,
                    "audit_actions": ["CREATE"],
                },
                {
                    "surface": "presave-delete",
                    "classification": "PASS",
                    "audit_rows": 1,
                    "audit_rows_complete": True,
                    "audit_actions": ["DELETE"],
                },
            ])
            complete = fuzzer.build_part11_report(events, list(fuzzer.SURFACES), artifact)
            self.assertEqual(complete["status"], "PASS")
            self.assertEqual(complete["claim"], "AUDIT_TRAIL_TECHNICAL_CONTROLS_DEMONSTRATED")

    def test_audit_timestamp_must_include_timezone(self) -> None:
        self.assertTrue(fuzzer.audit_timestamp_valid("2026-08-18T00:00:00Z"))
        self.assertFalse(fuzzer.audit_timestamp_valid("2026-08-18T00:00:00"))

    def test_delete_evidence_can_use_old_values_without_field_delta(self) -> None:
        log = {
            "organization_id": "org",
            "user_id": "user",
            "created_at": "2026-08-18T00:00:00Z",
            "prev_hash": "a",
            "entry_hash": "b",
            "action": "DELETE",
            "old_values": {"id": "record"},
            "changed_fields": None,
        }
        self.assertTrue(fuzzer.audit_log_complete(log, "DELETE"))


if __name__ == "__main__":
    unittest.main()
