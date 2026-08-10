#!/usr/bin/env python3

import random
import sys
import unicodedata
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

import case_editor_input_fuzzer as fuzzer


class CaseEditorInputFuzzerTests(unittest.TestCase):
    def test_nested_rows_and_list_readback(self) -> None:
        self.assertEqual(
            fuzzer.nested_root("fdaCrossReportedIndNumbers[].indNumber"),
            "fdaCrossReportedIndNumbers[]",
        )
        self.assertEqual(
            fuzzer.nested_root("drugReactionAssessments[].sourceOfAssessment"),
            "drugReactionAssessments[]",
        )
        self.assertIsNone(fuzzer.nested_root("raceCodes[]"))
        self.assertIsNone(fuzzer.nested_root("[].registrationNumber"))
        self.assertTrue(fuzzer.values_equal(["\t\n"], ["\t\n"]))
        self.assertTrue(fuzzer.values_equal(64.5, ["64.50"]))
        self.assertFalse(fuzzer.values_equal(["x"], ["y"]))

    def test_unicode_and_exact_length_candidates(self) -> None:
        field = {"roundTripValue": "base", "_maxLength": 4}
        rng = random.Random(1)
        self.assertEqual(fuzzer.candidate_count(field, 99), 17)
        self.assertNotEqual(fuzzer.field_value(field, rng, 8), "한글")
        self.assertEqual(unicodedata.normalize("NFC", fuzzer.field_value(field, rng, 8)), "한글")
        self.assertIn("\u202b", fuzzer.field_value(field, rng, 9))
        self.assertIn("\u200b", fuzzer.field_value(field, rng, 10))
        self.assertGreater(len(fuzzer.field_value(field, rng, 11)), 64)
        self.assertEqual(ord(fuzzer.field_value(field, rng, 12)), 0xD800)
        self.assertIn("\U0010ffff", fuzzer.field_value(field, rng, 13))
        self.assertEqual(len(fuzzer.field_value(field, rng, 14)), 4)
        self.assertEqual(len(fuzzer.field_value(field, rng, 15)), 5)
        self.assertGreater(len(fuzzer.field_value(field, rng, 16)), 5)
        self.assertEqual(fuzzer.length_expectation(field, 14), "accept")
        self.assertEqual(fuzzer.length_expectation(field, 16), "reject")

    def test_generated_max_lengths_and_candidate_caps(self) -> None:
        root = Path(__file__).resolve().parents[2]
        limits = fuzzer.load_max_lengths(root)
        self.assertEqual(limits["ICH.C.1.1.LENGTH.MAX"], 100)
        self.assertEqual(fuzzer.candidate_count({"roundTripValue": 1}, 17), 8)
        self.assertEqual(fuzzer.candidate_count({"roundTripValue": ["x"]}, 17), 14)

    def test_nullflavor_error_candidates_and_value_conflict(self) -> None:
        field = {
            "code": "D.1.nullFlavor",
            "payloadPath": "patientBirthDateNullFlavor",
            "roundTripValue": "UNK",
            "constraint": {"status": "verified", "invalidValue": "ZZZ"},
            "_allowedNullFlavors": ["UNK"],
            "_nullFlavorPartnerPath": "patientBirthDate",
            "_nullFlavorPartnerValue": "20000101",
        }
        rng = random.Random(1)
        self.assertEqual(fuzzer.candidate_count(field, 17), 14)
        for ordinal in (0, 5, 8, 9, 10, 11, 12):
            self.assertTrue(fuzzer.nullflavor_invalid_candidate(field, fuzzer.field_value(field, rng, ordinal)))
        mutation = {"patientBirthDateNullFlavor": "UNK"}
        self.assertTrue(fuzzer.add_nullflavor_partner(field, mutation, 13))
        self.assertEqual(mutation["patientBirthDate"], "20000101")
        self.assertEqual(fuzzer.candidate_kind(field, 13), "nullflavor_with_value")


if __name__ == "__main__":
    unittest.main()
