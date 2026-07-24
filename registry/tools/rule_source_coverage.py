from __future__ import annotations

import json
from pathlib import Path
from typing import Any


AUTHORITIES = {"ICH", "FDA", "MFDS"}
DISPOSITIONS = {"business_rule", "constraint", "guidance", "deferred"}


def source_hash(text: str) -> str:
    value = 14695981039346656037
    for byte in text.strip().encode("utf-8"):
        value ^= byte
        value = (value * 1099511628211) & 0xFFFFFFFFFFFFFFFF
    return f"fnv1a64:{value:016x}"


def load_rule_prose(root: Path) -> dict[tuple[str, str], str]:
    prose: dict[tuple[str, str], str] = {}
    for path in sorted((root / "dictionary" / "rules").glob("*.json")):
        payload = json.loads(path.read_text(encoding="utf-8"))
        authority = payload["authority"]
        for element, text in payload["rules"].items():
            prose[(authority, element)] = text
    return prose


def load_coverage(root: Path) -> dict[str, Any]:
    return json.loads(
        (root / "rule-source-coverage.json").read_text(encoding="utf-8")
    )


def validate_coverage_structure(
    root: Path,
    result: Any,
) -> dict[tuple[str, str], dict[str, Any]]:
    prose = load_rule_prose(root)
    try:
        payload = load_coverage(root)
    except (OSError, json.JSONDecodeError, KeyError) as error:
        result.add(f"rule source coverage could not be loaded: {error}")
        return {}

    if payload.get("version") != 1:
        result.add("rule source coverage version must be 1")
    if not isinstance(payload.get("auditedPages"), list):
        result.add("rule source coverage auditedPages must be an array")
    sources = payload.get("sources")
    if not isinstance(sources, list):
        result.add("rule source coverage sources must be an array")
        return {}

    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for source in sources:
        if not isinstance(source, dict):
            result.add("rule source coverage entry must be an object")
            continue
        key = (source.get("authority"), source.get("element"))
        if key in indexed:
            result.add(f"duplicate rule source coverage entry {key}")
            continue
        indexed[key] = source
        text = prose.get(key)
        if text is None:
            result.add(f"orphaned rule source coverage entry {key}")
            continue
        if source.get("sourceHash") != source_hash(text):
            result.add(f"{key} has stale sourceHash")
        requirements = source.get("requirements")
        if not isinstance(requirements, list) or not requirements:
            result.add(f"{key} requirements must be a non-empty array")
            continue
        seen_ids: set[str] = set()
        for requirement in requirements:
            if not isinstance(requirement, dict):
                result.add(f"{key} requirement must be an object")
                continue
            requirement_id = requirement.get("id")
            if requirement_id in seen_ids:
                result.add(f"{key} duplicate requirement id {requirement_id}")
            if isinstance(requirement_id, str):
                seen_ids.add(requirement_id)
            excerpt = requirement.get("sourceExcerpt")
            if not isinstance(excerpt, str) or excerpt not in text:
                result.add(
                    f"{key}/{requirement_id} sourceExcerpt is not present"
                )
            disposition = requirement.get("disposition")
            if disposition not in DISPOSITIONS:
                result.add(f"{key}/{requirement_id} has invalid disposition")
            if disposition in {"business_rule", "constraint"}:
                codes = requirement.get("ruleCodes")
                if not isinstance(codes, list) or not codes:
                    result.add(f"{key}/{requirement_id} requires ruleCodes")
            if disposition in {"guidance", "deferred"}:
                reason = requirement.get("reason")
                if not isinstance(reason, str) or not reason.strip():
                    result.add(f"{key}/{requirement_id} requires reason")
    return indexed


def validate_editor_coverage(
    root: Path,
    registry_rows: list[dict[str, Any]],
    coverage: dict[tuple[str, str], dict[str, Any]],
    result: Any,
) -> None:
    payload = load_coverage(root)
    audited_pages = set(payload.get("auditedPages", []))
    prose = load_rule_prose(root)
    rows_by_code = {
        row.get("e2br3_code"): row
        for row in registry_rows
        if isinstance(row.get("e2br3_code"), str)
    }

    for page in sorted(audited_pages):
        contract_path = root / "editor-contracts" / f"{page.lower()}.json"
        try:
            contract = json.loads(contract_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            result.add(f"could not load {page} source coverage contract: {error}")
            continue
        for field in contract.get("fields", []):
            code = field.get("code")
            row = rows_by_code.get(code)
            if row is None or row.get("local_only") is True:
                continue
            element = code.split("@", 1)[0]
            for authority in sorted(AUTHORITIES):
                key = (authority, element)
                if key not in prose:
                    continue
                source = coverage.get(key)
                if source is None:
                    result.add(
                        f"missing {authority}/{element} source coverage"
                    )
                    continue
                deferred = any(
                    requirement.get("disposition") == "deferred"
                    for requirement in source.get("requirements", [])
                )
                if deferred and row.get("status") == "complete":
                    result.add(
                        f"{code} cannot be complete while "
                        f"{authority}/{element} is deferred"
                    )
