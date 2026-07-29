#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any


CATALOG_PATTERN = re.compile(
    r"export const catalogConstraints = (\[.*\]) as const;",
    re.DOTALL,
)
BINDINGS_PATTERN = re.compile(
    r"export const catalogBindings: readonly GeneratedCatalogBinding\[\] = (\[.*\]);",
    re.DOTALL,
)


def invalid_value_for(constraint: dict[str, Any]) -> Any:
    kind = constraint.get("kind")
    if kind == "max_length":
        max_length = constraint.get("maxLength")
        if not isinstance(max_length, int) or max_length < 0:
            raise ValueError("max_length constraint requires a non-negative maxLength")
        return "X" * (max_length + 1)
    if kind == "inline_allowed_values":
        values = constraint.get("values")
        if values == ["true"]:
            return False
        if set(values or []) == {"false", "true"}:
            return "not-a-boolean"
        return "__INVALID__"
    if kind == "numeric":
        if constraint.get("numericShape") not in {"decimal", "dotted_version"}:
            raise ValueError(
                f"unsupported numeric shape {constraint.get('numericShape')!r}"
            )
        return "not-a-number"
    if kind == "format":
        format_name = constraint.get("formatName")
        if format_name == "e2b_datetime":
            return "not-a-date"
        if format_name == "base64":
            return "***"
        raise ValueError(f"unsupported format {format_name!r}")
    if kind == "null_flavor":
        return "__INVALID_NULL_FLAVOR__"
    raise ValueError(f"unsupported portable constraint kind {kind!r}")


def isolated_invalid_value_for(
    constraint: dict[str, Any],
    binding: dict[str, Any],
    catalog: dict[str, dict[str, Any]],
) -> Any:
    invalid_value = invalid_value_for(constraint)
    if constraint.get("kind") == "null_flavor":
        allowed = set(constraint.get("values") or [])
        for candidate_rule in catalog.values():
            if candidate_rule.get("kind") != "null_flavor":
                continue
            for candidate in candidate_rule.get("values") or []:
                if candidate not in allowed:
                    return candidate
        raise ValueError(
            f"cannot find a known disallowed nullFlavor for {constraint.get('code')}"
        )
    if constraint.get("kind") == "max_length":
        max_length = constraint["maxLength"]
        rule_codes = binding.get("ruleCodes", [])
        target_index = rule_codes.index(constraint["code"])
        for rule_code in rule_codes[:target_index]:
            sibling = catalog.get(rule_code)
            if sibling is None or sibling.get("kind") != "numeric":
                continue
            if sibling.get("numericShape") == "decimal":
                return "1" * (max_length + 1)
            if sibling.get("numericShape") == "dotted_version":
                if max_length < 2:
                    raise ValueError(
                        f"cannot exceed max length {max_length} with a dotted version"
                    )
                return "1." + ("0" * (max_length - 1))
            raise ValueError(
                f"unsupported numeric shape {sibling.get('numericShape')!r}"
            )
        return invalid_value
    if (
        constraint.get("kind") != "inline_allowed_values"
        or binding.get("valueType") != "string"
    ):
        return invalid_value

    sibling_max_lengths = [
        sibling["maxLength"]
        for rule_code in binding.get("ruleCodes", [])
        if (sibling := catalog.get(rule_code)) is not None
        and sibling.get("kind") == "max_length"
    ]
    if not sibling_max_lengths:
        return invalid_value
    max_length = min(sibling_max_lengths)
    if max_length < 1:
        raise ValueError(
            f"cannot isolate {constraint.get('code')}: sibling max length is zero"
        )
    allowed = set(constraint.get("values") or [])
    for candidate in "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_~☃":
        if candidate not in allowed:
            return candidate
    for length in range(2, max_length + 1):
        candidate = "X" * length
        if candidate not in allowed:
            return candidate
    raise ValueError(
        f"cannot isolate {constraint.get('code')} from its sibling max length"
    )


def load_catalog(path: Path) -> dict[str, dict[str, Any]]:
    match = CATALOG_PATTERN.search(path.read_text(encoding="utf-8"))
    if match is None:
        raise ValueError(f"could not read catalogConstraints from {path}")
    constraints = json.loads(match.group(1))
    by_code = {constraint["code"]: constraint for constraint in constraints}
    if len(by_code) != len(constraints):
        raise ValueError(f"duplicate portable constraint code in {path}")
    return by_code


def load_bindings(path: Path) -> dict[str, dict[str, Any]]:
    match = BINDINGS_PATTERN.search(path.read_text(encoding="utf-8"))
    if match is None:
        raise ValueError(f"could not read catalogBindings from {path}")
    bindings = json.loads(match.group(1))
    by_rule_code: dict[str, dict[str, Any]] = {}
    for binding in bindings:
        for rule_code in binding["ruleCodes"]:
            if rule_code in by_rule_code:
                raise ValueError(f"duplicate portable binding for {rule_code} in {path}")
            by_rule_code[rule_code] = binding
    return by_rule_code


def synchronize_contracts(
    registry_root: Path,
    catalog_path: Path,
    bindings_path: Path,
    *,
    write: bool,
) -> list[Path]:
    catalog = load_catalog(catalog_path)
    bindings = load_bindings(bindings_path)
    stale: list[Path] = []
    for path in sorted((registry_root / "editor-contracts").glob("*.json")):
        if path.name == "schema.json":
            continue
        contract = json.loads(path.read_text(encoding="utf-8"))
        changed = False
        for field in contract.get("fields", []):
            stage = field.get("constraint")
            if not isinstance(stage, dict) or stage.get("status") != "verified":
                continue
            rule_code = stage.get("ruleCode")
            rule = catalog.get(rule_code)
            if rule is None:
                raise ValueError(
                    f"{contract.get('pageId')}/{field.get('code')}: "
                    f"portable constraint {rule_code!r} is missing from {catalog_path}"
                )
            binding = bindings.get(rule_code)
            if binding is None:
                raise ValueError(
                    f"{contract.get('pageId')}/{field.get('code')}: "
                    f"portable binding {rule_code!r} is missing from {bindings_path}"
                )
            invalid_value = isolated_invalid_value_for(rule, binding, catalog)
            binding_path = binding.get("frontendPath")
            field_path = field.get("frontendPath")
            if binding_path == f"{field_path}[]":
                invalid_value = [invalid_value]
            elif binding_path != field_path:
                raise ValueError(
                    f"{contract.get('pageId')}/{field.get('code')}: frontend path "
                    f"{field_path!r} does not match portable binding {binding_path!r}"
                )
            if stage.get("invalidValue") != invalid_value:
                stage["invalidValue"] = invalid_value
                changed = True
        if not changed:
            continue
        if write:
            path.write_text(
                json.dumps(contract, indent=2, ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
        else:
            stale.append(path)
    return stale


def main() -> int:
    parser = argparse.ArgumentParser()
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--write", action="store_true")
    mode.add_argument("--check", action="store_true")
    parser.add_argument("--catalog-constraints", type=Path, required=True)
    parser.add_argument("--catalog-bindings", type=Path, required=True)
    parser.add_argument(
        "--registry-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
    )
    args = parser.parse_args()

    stale = synchronize_contracts(
        args.registry_root,
        args.catalog_constraints,
        args.catalog_bindings,
        write=args.write,
    )
    if stale:
        print("editor constraint invalid values are stale; run with --write")
        for path in stale:
            print(path)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
