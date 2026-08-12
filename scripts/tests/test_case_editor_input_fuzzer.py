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
        self.assertEqual(fuzzer.candidate_expectation(field, 14), ("length_boundary", None))
        self.assertEqual(fuzzer.candidate_expectation(field, 16), ("reject", None))

    def test_generated_max_lengths_and_candidate_caps(self) -> None:
        root = Path(__file__).resolve().parents[2]
        limits = fuzzer.load_max_lengths(root)
        self.assertEqual(limits["ICH.C.1.1.LENGTH.MAX"], 100)
        contract = [{"fields": [{
            "authority": "ICH",
            "code": "C.1.11.1",
            "roundTripValue": "base",
            "constraint": {"ruleCode": "ICH.C.1.11.1.ALLOWED.VALUE"},
        }]}]
        self.assertEqual(fuzzer.apply_max_lengths(contract, limits), 1)
        self.assertEqual(contract[0]["fields"][0]["_maxLength"], 1)
        self.assertEqual(fuzzer.candidate_count({"roundTripValue": 1}, 17), 8)
        self.assertEqual(fuzzer.candidate_count({"roundTripValue": ["x"]}, 17), 14)

    def test_generated_identifier_boolean_rules_and_expectations(self) -> None:
        root = Path(__file__).resolve().parents[2]
        identifiers = fuzzer.load_generated_rules(root, fuzzer.IDENTIFIER_RE)
        booleans = fuzzer.load_generated_rules(root, fuzzer.BOOLEAN_RE)
        field = {
            "authority": "ICH",
            "code": "D.10.8.r.2b",
            "roundTripValue": "base",
            "constraint": {"invalidValue": "bad", "ruleCode": "ICH.D.10.8.r.2b.ALLOWED.VALUE"},
        }
        boolean = {
            "authority": "ICH",
            "code": "C.1.9.1",
            "roundTripValue": True,
            "constraint": {"invalidValue": "not-a-boolean", "ruleCode": "ICH.C.1.9.1.ALLOWED.VALUE"},
        }
        contract = [{"fields": [field, boolean]}]
        self.assertEqual(fuzzer.apply_generated_rules(contract, identifiers, booleans), (1, 1))
        self.assertEqual(fuzzer.candidate_count(field, 99), 18)
        self.assertEqual(fuzzer.field_value(field, random.Random(1), 17), "\t\n")
        self.assertEqual(
            fuzzer.candidate_expectation(field, 17),
            ("reject", "ICH.D.10.8.r.2b.ALLOWED.VALUE"),
        )
        self.assertFalse(fuzzer.field_value(boolean, random.Random(1), 3))
        self.assertTrue(fuzzer.field_value(boolean, random.Random(1), 5))
        self.assertEqual(fuzzer.candidate_expectation(boolean, 3), ("accept", None))
        self.assertEqual(
            fuzzer.candidate_expectation(boolean, 2),
            ("reject", "ICH.C.1.9.1.ALLOWED.VALUE"),
        )

    def test_camel_case_rule_code_and_exact_max_expectation(self) -> None:
        self.assertEqual(
            fuzzer.summarize_error_detail({"ruleCode": "ICH.C.1.LENGTH.MAX", "message": "bad"})["error_rule_code"],
            "ICH.C.1.LENGTH.MAX",
        )
        self.assertIsNone(fuzzer.expectation_error(("reject", "RULE"), 422, "RULE"))
        self.assertIsNotNone(fuzzer.expectation_error(("reject", "RULE"), 422, "OTHER"))
        self.assertIsNone(fuzzer.expectation_error(("length_boundary", "LENGTH"), 422, "FORMAT"))
        self.assertIsNotNone(fuzzer.expectation_error(("length_boundary", "LENGTH"), 422, "LENGTH"))

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
