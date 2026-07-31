import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]


class ExportWriterCoverageTest(unittest.TestCase):
    def test_dictionary_elements_match_export_writers(self):
        dictionary_codes = set()
        for path in (ROOT / "registry/dictionary").glob("*.json"):
            entries = json.loads(path.read_text())["entries"]
            dictionary_codes.update(
                entry["code"] for entry in entries if entry.get("kind") == "element"
            )

        writer_codes = set()
        for path in (ROOT / "crates/libs/xml/src/export").rglob("*.rs"):
            writer_codes.update(
                code
                for code in re.findall(
                    r"^\s*///?\s*e2b:([A-Za-z0-9.]+)\s*$", path.read_text(), re.M
                )
                if not code.startswith("ICH.XML.")
            )

        self.assertEqual(dictionary_codes, writer_codes)


if __name__ == "__main__":
    unittest.main()
