import io
import json
import sys
import unittest
import zipfile
from pathlib import Path

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

    def test_structural_signature_is_stable_after_identifier_change(self) -> None:
        original = FIXTURE.read_bytes()
        changed = fuzzer.unique_xml(original, "SIG")
        self.assertEqual(fuzzer.structural_signature(original), fuzzer.structural_signature(changed))
        self.assertEqual(fuzzer.identity_signature(changed)[0], "ZZ-FUZZ-SIG")
        self.assertEqual(fuzzer.identity_signature(changed)[1], fuzzer.identity_signature(original)[1])


if __name__ == "__main__":
    unittest.main()
