from __future__ import annotations

import json
from pathlib import Path
from typing import Any


REQUIRED_FIELD_KEYS = (
    "code",
    "authority",
    "frontendPath",
    "payloadPath",
    "projectionPath",
    "patch",
    "roundTripValue",
)
STAGE_STATUSES = {"verified", "not_applicable"}


def _editor_status(row: dict[str, Any]) -> Any:
    return row.get("editor_status", row.get("status"))


def load_editor_contract(root: Path, page_id: str) -> dict[str, Any]:
    path = root / "editor-contracts" / f"{page_id.lower()}.json"
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ValueError(f"could not load {page_id} editor contract: {error}") from error
    if not isinstance(payload, dict):
        raise ValueError(f"{path}: editor contract must be an object")
    return payload


def _registry_frontend_path(row: dict[str, Any]) -> str:
    frontend = row.get("frontend")
    if not isinstance(frontend, dict):
        return ""
    section = frontend.get("section")
    field = frontend.get("field")
    if not isinstance(section, str) or not isinstance(field, str):
        return ""
    return f"{section}.{field}".replace("[]", "")


def _validate_stage(code: str, name: str, stage: Any, result: Any) -> None:
    if not isinstance(stage, dict):
        result.add(f"{code} missing {name} evidence")
        return
    status = stage.get("status")
    if status not in STAGE_STATUSES:
        result.add(
            f"{code} {name}.status must be one of {sorted(STAGE_STATUSES)}"
        )
        return
    if status == "not_applicable" and not stage.get("reason"):
        result.add(f"{code} {name} not_applicable requires reason")
    if status == "verified":
        evidence_key = "ruleCode" if name == "constraint" else "issuePath"
        if not stage.get(evidence_key):
            result.add(f"{code} {name} verified requires {evidence_key}")


def validate_editor_contract(
    registry_rows: list[dict[str, Any]],
    contract: dict[str, Any],
    result: Any,
) -> None:
    page_id = contract.get("pageId")
    if not isinstance(page_id, str) or not page_id:
        result.add("editor contract missing pageId")
        return
    fields = contract.get("fields")
    if not isinstance(fields, list):
        result.add(f"{page_id} editor contract fields must be an array")
        return

    rows_by_code = {
        row.get("e2br3_code"): row
        for row in registry_rows
        if isinstance(row, dict) and isinstance(row.get("e2br3_code"), str)
    }
    fields_by_code: dict[str, dict[str, Any]] = {}
    for field in fields:
        if not isinstance(field, dict):
            result.add(f"{page_id} editor contract field must be an object")
            continue
        code = field.get("code")
        if not isinstance(code, str) or not code:
            result.add(f"{page_id} editor contract field missing code")
            continue
        if code in fields_by_code:
            result.add(f"{page_id} editor contract duplicate code {code}")
            continue
        fields_by_code[code] = field
        payload_path = field.get("payloadPath")
        if not isinstance(payload_path, str) or not payload_path:
            result.add(f"{code} missing payloadPath evidence")
        patch = field.get("patch")
        if not isinstance(patch, dict):
            result.add(f"{code} patch must be an object")
        else:
            if patch.get("kind") not in {"row", "rows"}:
                result.add(f"{code} patch.kind must be row or rows")
            owner = patch.get("owner")
            if not isinstance(owner, str) or not owner:
                result.add(f"{code} patch.owner is required")
        row = rows_by_code.get(code)
        if row is None:
            result.add(f"{code} editor contract has no registry row")
            continue
        if field.get("authority") != row.get("authority"):
            result.add(
                f"{code} editor contract authority {field.get('authority')} does not match registry {row.get('authority')}"
            )
        frontend_path = field.get("frontendPath")
        registry_path = _registry_frontend_path(row)
        if isinstance(frontend_path, str) and frontend_path.replace("[]", "") != registry_path:
            result.add(
                f"{code} frontend path {frontend_path} does not match registry {registry_path}"
            )
        if _editor_status(row) == "complete":
            for key in REQUIRED_FIELD_KEYS:
                if key not in field:
                    result.add(f"{code} missing {key} evidence")
            _validate_stage(code, "constraint", field.get("constraint"), result)
            if "businessValidation" in field:
                _validate_stage(
                    code,
                    "businessValidation",
                    field.get("businessValidation"),
                    result,
                )

    for row in registry_rows:
        if (
            row.get("editor_page") == page_id
            and _editor_status(row) == "complete"
            and row.get("e2br3_code") not in fields_by_code
        ):
            result.add(
                f"{row.get('e2br3_code')} complete but missing from {page_id} editor contract"
            )
