import json
import tempfile
import unittest
from pathlib import Path

from registry.tools.sync_editor_constraint_invalid_values import (
    invalid_value_for,
    isolated_invalid_value_for,
    synchronize_contracts,
)


class EditorConstraintInvalidValueTests(unittest.TestCase):
    def test_derives_a_rejected_fixture_for_every_portable_constraint_kind(self):
        cases = [
            (
                {
                    "kind": "max_length",
                    "maxLength": 2,
                    "values": [],
                    "numericShape": None,
                    "formatName": None,
                },
                "XXX",
            ),
            (
                {
                    "kind": "inline_allowed_values",
                    "maxLength": None,
                    "values": ["1", "2"],
                    "numericShape": None,
                    "formatName": None,
                },
                "__INVALID__",
            ),
            (
                {
                    "kind": "inline_allowed_values",
                    "maxLength": None,
                    "values": ["true"],
                    "numericShape": None,
                    "formatName": None,
                },
                False,
            ),
            (
                {
                    "kind": "numeric",
                    "maxLength": None,
                    "values": [],
                    "numericShape": "decimal",
                    "formatName": None,
                },
                "not-a-number",
            ),
            (
                {
                    "kind": "format",
                    "maxLength": None,
                    "values": [],
                    "numericShape": None,
                    "formatName": "e2b_datetime",
                },
                "not-a-date",
            ),
            (
                {
                    "kind": "format",
                    "maxLength": None,
                    "values": [],
                    "numericShape": None,
                    "formatName": "base64",
                },
                "***",
            ),
            (
                {
                    "kind": "null_flavor",
                    "maxLength": None,
                    "values": ["NI"],
                    "numericShape": None,
                    "formatName": None,
                },
                "__INVALID_NULL_FLAVOR__",
            ),
        ]

        for constraint, expected in cases:
            with self.subTest(kind=constraint["kind"]):
                self.assertEqual(expected, invalid_value_for(constraint))

    def test_allowed_value_fixture_satisfies_a_sibling_max_length_rule(self):
        allowed = {
            "code": "ICH.C.1.3.ALLOWED.VALUE",
            "kind": "inline_allowed_values",
            "maxLength": None,
            "values": ["1", "2", "3", "4"],
            "numericShape": None,
            "formatName": None,
        }
        maximum = {
            "code": "ICH.C.1.3.LENGTH.MAX",
            "kind": "max_length",
            "maxLength": 1,
            "values": [],
            "numericShape": None,
            "formatName": None,
        }
        binding = {
            "valueType": "string",
            "ruleCodes": [maximum["code"], allowed["code"]],
        }

        invalid = isolated_invalid_value_for(
            allowed,
            binding,
            {allowed["code"]: allowed, maximum["code"]: maximum},
        )

        self.assertEqual("0", invalid)

    def test_max_length_fixture_satisfies_a_preceding_numeric_rule(self):
        numeric = {
            "code": "ICH.D.7.1.r.1a.ALLOWED.VALUE",
            "kind": "numeric",
            "maxLength": None,
            "values": [],
            "numericShape": "dotted_version",
            "formatName": None,
        }
        maximum = {
            "code": "ICH.D.7.1.r.1a.LENGTH.MAX",
            "kind": "max_length",
            "maxLength": 4,
            "values": [],
            "numericShape": None,
            "formatName": None,
        }
        binding = {
            "valueType": "string",
            "ruleCodes": [numeric["code"], maximum["code"]],
        }

        invalid = isolated_invalid_value_for(
            maximum,
            binding,
            {numeric["code"]: numeric, maximum["code"]: maximum},
        )

        self.assertEqual("1.000", invalid)

    def test_null_flavor_fixture_uses_a_known_but_disallowed_catalog_token(self):
        target = {
            "code": "ICH.G.k.4.r.4.NULLFLAVOR.ALLOWED",
            "kind": "null_flavor",
            "maxLength": None,
            "values": ["MSK", "ASKU", "NASK"],
            "numericShape": None,
            "formatName": None,
        }
        other = {
            "code": "ICH.C.1.7.NULLFLAVOR.ALLOWED",
            "kind": "null_flavor",
            "maxLength": None,
            "values": ["NI"],
            "numericShape": None,
            "formatName": None,
        }
        binding = {
            "valueType": "string",
            "ruleCodes": [target["code"]],
        }

        invalid = isolated_invalid_value_for(
            target,
            binding,
            {target["code"]: target, other["code"]: other},
        )

        self.assertEqual("NI", invalid)

    def test_synchronizes_explicit_values_from_the_generated_catalog(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            contracts = root / "editor-contracts"
            contracts.mkdir()
            contract_path = contracts / "ci.json"
            contract_path.write_text(
                json.dumps(
                    {
                        "pageId": "CI",
                        "fields": [
                            {
                                "code": "C.1.1",
                                "frontendPath": "safetyReportIdentification.safetyReportId",
                                "constraint": {
                                    "status": "verified",
                                    "ruleCode": "ICH.C.1.1.LENGTH.MAX",
                                },
                            },
                            {
                                "code": "C.local",
                                "constraint": {
                                    "status": "not_applicable",
                                    "reason": "No portable constraint.",
                                },
                            },
                        ],
                    },
                    indent=2,
                )
                + "\n",
                encoding="utf-8",
            )
            catalog_path = root / "catalogConstraints.ts"
            catalog_path.write_text(
                "export const catalogConstraints = "
                + json.dumps(
                    [
                        {
                            "code": "ICH.C.1.1.LENGTH.MAX",
                            "kind": "max_length",
                            "maxLength": 2,
                            "values": [],
                            "numericShape": None,
                            "formatName": None,
                        }
                    ]
                )
                + " as const;\n",
                encoding="utf-8",
            )
            bindings_path = root / "catalogBindings.ts"
            bindings_path.write_text(
                "export const catalogBindings: readonly GeneratedCatalogBinding[] = "
                + json.dumps(
                    [
                        {
                            "frontendPath": "safetyReportIdentification.safetyReportId",
                            "ruleCodes": ["ICH.C.1.1.LENGTH.MAX"],
                        }
                    ]
                )
                + ";\n",
                encoding="utf-8",
            )

            self.assertEqual(
                [contract_path],
                synchronize_contracts(root, catalog_path, bindings_path, write=False),
            )
            self.assertEqual(
                [], synchronize_contracts(root, catalog_path, bindings_path, write=True)
            )
            self.assertEqual(
                [], synchronize_contracts(root, catalog_path, bindings_path, write=False)
            )
            updated = json.loads(contract_path.read_text(encoding="utf-8"))

        self.assertEqual(
            "XXX", updated["fields"][0]["constraint"]["invalidValue"]
        )
        self.assertNotIn("invalidValue", updated["fields"][1]["constraint"])

    def test_wraps_invalid_value_for_a_binding_to_array_elements(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            contracts = root / "editor-contracts"
            contracts.mkdir()
            contract_path = contracts / "dg.json"
            contract_path.write_text(
                json.dumps(
                    {
                        "pageId": "DG",
                        "fields": [
                            {
                                "code": "FDA.G.k.local.additionalInfoCodesJson",
                                "frontendPath": "drugs[].drugAdditionalInformationCodes",
                                "constraint": {
                                    "status": "verified",
                                    "ruleCode": "ICH.G.k.10.r.ALLOWED.VALUE",
                                },
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            catalog_path = root / "catalogConstraints.ts"
            catalog_path.write_text(
                "export const catalogConstraints = "
                + json.dumps(
                    [
                        {
                            "code": "ICH.G.k.10.r.ALLOWED.VALUE",
                            "kind": "inline_allowed_values",
                            "maxLength": None,
                            "values": ["1", "2"],
                            "numericShape": None,
                            "formatName": None,
                        }
                    ]
                )
                + " as const;",
                encoding="utf-8",
            )
            bindings_path = root / "catalogBindings.ts"
            bindings_path.write_text(
                "export const catalogBindings: readonly GeneratedCatalogBinding[] = "
                + json.dumps(
                    [
                        {
                            "frontendPath": "drugs[].drugAdditionalInformationCodes[]",
                            "ruleCodes": ["ICH.G.k.10.r.ALLOWED.VALUE"],
                        }
                    ]
                )
                + ";",
                encoding="utf-8",
            )

            synchronize_contracts(root, catalog_path, bindings_path, write=True)
            updated = json.loads(contract_path.read_text(encoding="utf-8"))

        self.assertEqual(
            ["__INVALID__"],
            updated["fields"][0]["constraint"]["invalidValue"],
        )


if __name__ == "__main__":
    unittest.main()
