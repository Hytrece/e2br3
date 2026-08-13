#!/usr/bin/env python3

import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import case_business_validator_fuzzer as fuzzer


class CaseBusinessValidatorFuzzerTests(unittest.TestCase):
    def test_catalog_is_seeded_and_ordinals_are_stable(self) -> None:
        first = fuzzer.scenario_catalog(11)
        repeated = fuzzer.scenario_catalog(11)
        another = fuzzer.scenario_catalog(12)
        self.assertEqual(first, repeated)
        self.assertNotEqual(first[1].invalid_value, another[1].invalid_value)
        self.assertEqual([item.ordinal for item in first], list(range(len(first))))
        self.assertEqual(len({item.expected_code for item in first}), len(first))

    def test_inventory_separates_input_contract_rules(self) -> None:
        codes = fuzzer.discover_business_rule_codes()
        self.assertIn("ICH.G.k.2.1.MPID_PHPID.EXCLUSIVE", codes)
        self.assertIn("ICH.G.k.5a.REQUIRED", codes)
        self.assertNotIn("ICH.G.k.2.2.LENGTH.MAX", codes)
        self.assertNotIn("ICH.G.k.1.ALLOWED.VALUE", codes)
        self.assertIn("ICH.G.k.7.r.2a.ALLOWED.VALUE", codes)

    def test_issue_oracle_requires_complete_evidence(self) -> None:
        report = {"issues": [{
            "code": "RULE",
            "message": "bad relation",
            "path": "drugs.0.value",
            "section": "G",
            "subsection": "G.k",
            "blocking": True,
        }]}
        self.assertEqual(fuzzer.issue_codes(report), {"RULE"})
        self.assertTrue(fuzzer.issue_complete(report, "RULE"))
        report["issues"][0].pop("path")
        self.assertFalse(fuzzer.issue_complete(report, "RULE"))


if __name__ == "__main__":
    unittest.main()
