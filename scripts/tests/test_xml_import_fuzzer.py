import io
import json
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

import xml_import_fuzzer as fuzzer


FIXTURE = ROOT / "docs/exporter/fda/FAERS2022Scenario1.xml"


class XmlImportFuzzerTests(unittest.TestCase):
    def test_unique_xml_keeps_wrapper_and_case_identifiers_equal(self) -> None:
        xml = fuzzer.unique_xml(FIXTURE.read_bytes(), "ABC")
        self.assertIn(b"<MCCI_IN200100UV01 xmlns=", xml)
        self.assertNotIn(b"<ns0:MCCI_IN200100UV01", xml)
        root = fuzzer.ET.fromstring(xml)
        porr = next(node for node in root.iter() if fuzzer.local_name(node.tag) == "PORR_IN049016UV")
        investigation = next(node for node in root.iter() if fuzzer.local_name(node.tag) == "investigationEvent")
        self.assertEqual(fuzzer.direct_child(porr, "id").get("extension"), "ZZ-FUZZ-ABC")
        self.assertEqual(fuzzer.direct_child(investigation, "id").get("extension"), "ZZ-FUZZ-ABC")

    def test_mutations_cover_accept_and_reject_boundaries(self) -> None:
        xml = fuzzer.unique_xml(FIXTURE.read_bytes(), "MUT")
        self.assertRaises(fuzzer.ET.ParseError, fuzzer.ET.fromstring, fuzzer.mutate(xml, "malformed_xml"))
        self.assertTrue(fuzzer.mutate(xml, "invalid_utf8").startswith(b"\xff\xfe"))
        mismatch = fuzzer.mutate(xml, "wrapper_mismatch")
        root = fuzzer.ET.fromstring(mismatch)
        porr = next(node for node in root.iter() if fuzzer.local_name(node.tag) == "PORR_IN049016UV")
        self.assertEqual(fuzzer.direct_child(porr, "id").get("extension"), "ZZ-FUZZ-MISMATCH")
        self.assertIsNotNone(fuzzer.mutate(xml, "c17_ni"))
        self.assertIsNotNone(fuzzer.mutate(xml, "invalid_date"))
        first = fuzzer.unique_xml(FIXTURE.read_bytes(), "ONE")
        second = fuzzer.unique_xml(FIXTURE.read_bytes(), "TWO")
        self.assertNotEqual(fuzzer.identity_signature(first), fuzzer.identity_signature(second))

    def test_zip_and_result_classification(self) -> None:
        sample = fuzzer.Sample("one.xml", "fda", b"<x/>", "healthy", "success")
        archive = fuzzer.zip_samples([sample])
        with zipfile.ZipFile(io.BytesIO(archive)) as value:
            self.assertEqual(value.namelist(), ["one.xml"])
        body = json.dumps({"data": {"importedCases": [{"sourceFileName": "one.xml"}]}}).encode()
        self.assertEqual(fuzzer.imported_rows(body)[0]["sourceFileName"], "one.xml")
        self.assertEqual(fuzzer.classify_error("PgDatabaseError SQLx(", 200), "server_or_raw_db")
        self.assertEqual(fuzzer.classify_error("includedDocument bad", 200), "base64")

    def test_batch_reconciliation_reports_missing_and_unexpected_rows(self) -> None:
        one = fuzzer.Sample("one.xml", "ich", b"<x/>", "healthy", "success")
        two = fuzzer.Sample("two.xml", "ich", b"<x/>", "healthy", "success")
        matched, missing, unexpected = fuzzer.reconcile_batch(
            [one, two],
            [
                {"sourceFileName": "one.xml", "status": "success"},
                {"sourceFileName": "other.xml", "status": "success"},
            ],
        )
        self.assertEqual([sample.name for sample, _ in matched], ["one.xml"])
        self.assertEqual([sample.name for sample in missing], ["two.xml"])
        self.assertEqual(unexpected[0]["sourceFileName"], "other.xml")

    def test_transport_failure_is_not_an_archive_rejection(self) -> None:
        self.assertEqual(fuzzer.archive_probe_status(None, "TimeoutError", []), "request_error")
        self.assertEqual(fuzzer.archive_probe_status(413, None, []), "error")
        self.assertEqual(
            fuzzer.archive_probe_status(200, None, [{"status": "error"}]),
            "error",
        )

    def test_environment_guard_requires_isolated_non_8080_target(self) -> None:
        fuzzer.guard_environment(
            "http://127.0.0.1:8098",
            "postgres://user:secret@localhost/e2br3_ui_xml_fuzz",
        )
        with self.assertRaises(SystemExit):
            fuzzer.guard_environment(
                "http://127.0.0.1:8080",
                "postgres://user:secret@localhost/e2br3_ui_xml_fuzz",
            )
        with self.assertRaises(SystemExit):
            fuzzer.guard_environment(
                "http://127.0.0.1:8098",
                "postgres://user:secret@localhost/app_db",
            )

    def test_modes_choose_fast_and_full_defaults(self) -> None:
        with mock.patch.object(sys, "argv", ["xml_import_fuzzer.py", "--config", "x.json"]), mock.patch.object(fuzzer, "run", return_value=0) as run:
            self.assertEqual(fuzzer.main(), 0)
            smoke = run.call_args.args[0]
        self.assertEqual(smoke.copies, 1)
        self.assertFalse(smoke.archive_probes)

        with mock.patch.object(sys, "argv", ["xml_import_fuzzer.py", "--config", "x.json", "--mode", "full"]), mock.patch.object(fuzzer, "run", return_value=0) as run:
            self.assertEqual(fuzzer.main(), 0)
            full = run.call_args.args[0]
        self.assertEqual(full.copies, 10)
        self.assertTrue(full.archive_probes)

    def test_corpus_documents_reads_selected_archive_members(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            archive_path = Path(directory) / "official.zip"
            with zipfile.ZipFile(archive_path, "w") as archive:
                archive.writestr("examples/one.xml", b"<one/>")
                archive.writestr("references/two.xml", b"<two/>")
            documents = fuzzer.corpus_documents([
                {"path": str(archive_path), "members": ["examples/*.xml"]}
            ])
        self.assertEqual(documents, [("one", b"<one/>", "success")])

    def test_structural_signature_is_stable_after_identifier_change(self) -> None:
        original = FIXTURE.read_bytes()
        changed = fuzzer.unique_xml(original, "SIG")
        self.assertEqual(fuzzer.structural_signature(original), fuzzer.structural_signature(changed))
        self.assertEqual(fuzzer.identity_signature(changed)[0], "ZZ-FUZZ-SIG")
        self.assertEqual(fuzzer.identity_signature(changed)[1], fuzzer.identity_signature(original)[1])


if __name__ == "__main__":
    unittest.main()
