#!/usr/bin/env python3
"""Seeded black-box fuzzer for persisted Case business-validator rules.

Each scenario proves both edges of one rule: a valid persisted baseline does
not emit the rule, one field mutation emits it, and restoring that field
removes it. Save readback and audit evidence are checked on both mutations.
"""

from __future__ import annotations

import argparse
import json
import os
import random
import re
import sys
import time
import uuid
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from case_editor_input_fuzzer import (
    AUDIT_TABLES,
    audit_key_matches,
    audit_log_complete,
    commit_sha,
    created_row_id,
    get_path,
    object_id,
    redacted,
    response_summary,
    unwrap,
    values_equal,
)
from rbac_rls_blackbox import ApiClient, guard_target


ROOT = Path(__file__).resolve().parents[1]
RULE_RE = re.compile(r'"((?:ICH|FDA|MFDS)\.[A-Za-z0-9_.-]+)"')
INPUT_ONLY_SUFFIXES = (".LENGTH.MAX", ".ALLOWED.VALUE", ".NULLFLAVOR.ALLOWED")
RETAINED_ALLOWED_VALUE_RULES = {
    "ICH.C.1.6.1.r.2.ALLOWED.VALUE",
    "ICH.C.4.r.2.ALLOWED.VALUE",
    "ICH.D.6.NULLFLAVOR.ALLOWED",
    "ICH.D.7.1.r.1a.ALLOWED.VALUE",
    "ICH.D.10.7.1.r.1a.ALLOWED.VALUE",
    "ICH.E.i.2.1a.ALLOWED.VALUE",
    "ICH.F.r.2.2a.ALLOWED.VALUE",
    "ICH.G.k.7.r.2a.ALLOWED.VALUE",
    "ICH.H.3.r.1a.ALLOWED.VALUE",
}


@dataclass(frozen=True)
class Scenario:
    ordinal: int
    scenario_id: str
    authority: str
    page: str
    owner: str
    field: str
    projection_field: str
    expected_code: str
    invalid_value: Any
    valid_value: Any


@dataclass
class Event:
    kind: str
    scenario_id: str | None
    scenario_ordinal: int | None
    classification: str
    http_status: int | None
    response: dict[str, Any]


def discover_business_rule_codes(root: Path = ROOT) -> set[str]:
    codes: set[str] = set()
    section_root = root / "crates/libs/validator/src/case/sections"
    for name in "cdefghn":
        path = section_root / f"{name}.rs"
        codes.update(RULE_RE.findall(path.read_text()))
    return {
        code
        for code in codes
        if code in RETAINED_ALLOWED_VALUE_RULES
        or not code.endswith(INPUT_ONLY_SUFFIXES)
    }


def scenario_catalog(seed: int) -> list[Scenario]:
    rng = random.Random(seed)
    year = rng.randint(2020, 2024)
    suffix = f"{rng.getrandbits(32):08x}"
    return [
        Scenario(
            0,
            "c1-received-date-order",
            "ich",
            "CI",
            "safetyReportIdentification",
            "dateOfMostRecentInformation",
            "dateOfMostRecentInformation",
            "ICH.C.1.4.AFTER_C.1.5.FORBIDDEN",
            f"{year}0302",
            f"{year}0304",
        ),
        Scenario(
            1,
            "g-mpid-phpid-exclusive",
            "ich",
            "DG",
            "drug",
            "phpid",
            "phpid",
            "ICH.G.k.2.1.MPID_PHPID.EXCLUSIVE",
            f"PHPID-{suffix}",
            None,
        ),
        Scenario(
            2,
            "g-cumulative-dose-unit-required",
            "ich",
            "DG",
            "drug",
            "cumulativeDoseUnit",
            "cumulative_dose_first_reaction_unit",
            "ICH.G.k.5b.REQUIRED",
            None,
            "mg",
        ),
        Scenario(
            3,
            "g-cumulative-dose-value-required",
            "ich",
            "DG",
            "drug",
            "cumulativeDoseValue",
            "cumulative_dose_first_reaction_value",
            "ICH.G.k.5a.REQUIRED",
            None,
            12.5,
        ),
        Scenario(
            4,
            "g-suspect-drug-aggregate-required",
            "ich",
            "DG",
            "drug",
            "drugCharacterization",
            "drugCharacterization",
            "ICH.G.k.1.AGGREGATE.REQUIRED",
            "2",
            "1",
        ),
        Scenario(
            5,
            "g-authorization-country-required",
            "ich",
            "DG",
            "drug",
            "drugAuthorizationCountry",
            "manufacturer_country",
            "ICH.G.k.3.2.REQUIRED",
            None,
            "KR",
        ),
    ]


def issue_codes(value: Any) -> set[str]:
    issues = value.get("issues", []) if isinstance(value, dict) else []
    return {
        issue["code"]
        for issue in issues
        if isinstance(issue, dict) and isinstance(issue.get("code"), str)
    }


def issue_complete(value: Any, code: str) -> bool:
    issues = value.get("issues", []) if isinstance(value, dict) else []
    return any(
        isinstance(issue, dict)
        and issue.get("code") == code
        and isinstance(issue.get("message"), str)
        and bool(issue.get("message"))
        and isinstance(issue.get("path"), str)
        and bool(issue.get("path"))
        and isinstance(issue.get("section"), str)
        and isinstance(issue.get("subsection"), str)
        and isinstance(issue.get("blocking"), bool)
        for issue in issues
    )


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser()
    result.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
    result.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "demo.cro.admin@example.com"))
    result.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
    result.add_argument("--seed", type=int, default=int(time.time()))
    result.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz/business-validator")
    result.add_argument("--timeout", type=float, default=20)
    result.add_argument("--max-actions", type=int, default=500)
    result.add_argument("--deadline-seconds", type=float, default=300)
    result.add_argument("--allow-remote", action="store_true")
    result.add_argument("--dry-run", action="store_true")
    return result


def main(args: argparse.Namespace) -> int:
    guard_target(args.base_url, args.allow_remote)
    scenarios = scenario_catalog(args.seed)
    inventory = discover_business_rule_codes()
    covered = {scenario.expected_code for scenario in scenarios}
    if args.dry_run:
        print(json.dumps({
            "seed": args.seed,
            "scenarios": len(scenarios),
            "covered_rules": len(covered),
            "inventory_rules": len(inventory),
            "uncovered_rules": sorted(inventory - covered),
        }, sort_keys=True))
        return 0
    if not args.password:
        raise SystemExit("set E2BR3_ADMIN_PASSWORD")

    client = ApiClient(args.base_url, args.timeout)
    events: list[Event] = []
    started = time.monotonic()
    requests = 0
    interrupted: str | None = None

    def add(kind: str, scenario: Scenario | None, classification: str, status: int | None, detail: dict[str, Any]) -> None:
        events.append(Event(
            kind,
            scenario.scenario_id if scenario else None,
            scenario.ordinal if scenario else None,
            classification,
            status,
            detail,
        ))

    def request(method: str, path: str, payload: dict[str, Any] | None = None) -> tuple[int | None, Any, dict[str, Any]]:
        nonlocal interrupted, requests
        if requests >= args.max_actions:
            interrupted = interrupted or "max_actions"
            return None, None, {}
        if time.monotonic() - started >= args.deadline_seconds:
            interrupted = interrupted or "deadline"
            return None, None, {}
        requests += 1
        status, body, transport = client.request(method, path, payload)
        summary = response_summary(status, body)
        if transport:
            summary["transport_error"] = transport
            interrupted = interrupted or "transport_error"
        if status is not None and status >= 500:
            interrupted = interrupted or "server_error"
        try:
            value = unwrap(json.loads(body))
        except (UnicodeDecodeError, json.JSONDecodeError):
            value = None
        return status, value, summary

    def page_current(case_id: str, page: str, owner: str, row_id: str | None = None) -> tuple[int | None, Any]:
        route = f"/api/cases/{case_id}/editor/pages/{page}"
        status, value, _ = request("GET", f"{route}/rows/{row_id}" if row_id else route)
        if row_id:
            return status, value.get(owner, value) if isinstance(value, dict) else value
        rows = value.get("rows", {}) if isinstance(value, dict) else {}
        current = rows.get(owner) if isinstance(rows, dict) else None
        if isinstance(current, list):
            current = current[0] if current else None
        return status, current

    def audit_logs(case_id: str, owner: str, row_id: str) -> list[dict[str, Any]]:
        table = AUDIT_TABLES.get(owner)
        target = f"{table}/{row_id}" if table else f"cases/{case_id}"
        status, value, _ = request("GET", f"/api/audit-logs/by-record/{target}")
        if status != 200:
            return []
        if isinstance(value, list):
            return [item for item in value if isinstance(item, dict)]
        items = value.get("items", value.get("data", [])) if isinstance(value, dict) else []
        return [item for item in items if isinstance(item, dict)] if isinstance(items, list) else []

    def validation(case_id: str, authority: str) -> tuple[int | None, Any, dict[str, Any]]:
        return request("GET", f"/api/cases/{case_id}/validation?authority={authority}")

    status, _, summary = request("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
    add("login", None, "PASS" if status == 200 else "FAIL", status, summary)
    if status != 200:
        interrupted = interrupted or "login_failed"

    year = scenario_catalog(args.seed)[0].invalid_value[:4]

    def create_case() -> tuple[int | None, str | None, dict[str, Any]]:
        status, value, summary = request("POST", "/api/cases", {
            "data": {
                "safetyReportIdentification": {"safetyReportId": f"BUSINESS-FUZZ-{uuid.uuid4()}"},
                "status": "draft",
            }
        })
        return status, object_id(value), summary

    def ci_payload(field: str | None = None, value: Any = None) -> dict[str, Any]:
        payload = {
            "transmissionDate": f"{year}0305120000+0900",
            "reportType": "1",
            "dateFirstReceivedFromSource": f"{year}0303",
            "dateOfMostRecentInformation": f"{year}0304",
        }
        if field:
            if value is None:
                payload.pop(field, None)
            else:
                payload[field] = value
        return payload

    def drug_payload(scenario: Scenario, value: Any) -> dict[str, Any]:
        payload: dict[str, Any] = {
            "drugCharacterization": "1",
            "medicinalProduct": "Business fuzz product",
            "mpid": f"MPID-{args.seed}",
            "cumulativeDoseValue": 12.5,
            "cumulativeDoseUnit": "mg",
            "drugAuthorizationNumber": f"AUTH-{args.seed}",
            "drugAuthorizationCountry": "KR",
        }
        if value is None:
            payload.pop(scenario.field, None)
        else:
            payload[scenario.field] = value
        return payload

    def run_edge(scenario: Scenario, edge: str, value: Any) -> None:
        nonlocal interrupted
        status, case_id, summary = create_case()
        if status != 201 or not case_id:
            add(edge, scenario, "FAIL", status, {**summary, "reason": "case_create_failed"})
            interrupted = interrupted or "case_create_failed"
            return

        status, current = page_current(case_id, "CI", "safetyReportIdentification")
        ci_id = object_id(current)
        ci = ci_payload(scenario.field, value) if scenario.page == "CI" else ci_payload()
        ci["id"] = ci_id
        status, _, save_summary = request("PATCH", f"/api/cases/{case_id}/editor/pages/CI", {
            "authorities": ["ich"],
            "rows": {"safetyReportIdentification": ci},
        })
        if status != 200 or not ci_id:
            add(edge, scenario, "FAIL", status, {**save_summary, "reason": "ci_fixture_failed"})
            return

        if scenario.page == "CI":
            owner_id = ci_id
            read_status, current = page_current(case_id, "CI", scenario.owner)
        else:
            status, created, save_summary = request("POST", f"/api/cases/{case_id}/editor/pages/DG/rows", {
                "authorities": [scenario.authority],
                "rows": {"drug": drug_payload(scenario, value)},
            })
            owner_id = created_row_id(created)
            if status != 201 or not owner_id:
                add(edge, scenario, "FAIL", status, {**save_summary, "reason": "dg_fixture_failed"})
                return
            read_status, current = page_current(case_id, "DG", scenario.owner, owner_id)

        actual = get_path(current, scenario.projection_field)
        logs = audit_logs(case_id, scenario.owner, owner_id)
        complete_logs = [log for log in logs if audit_log_complete(log)]
        field_match = any(
            audit_key_matches(log.get("changedFields", log.get("changed_fields", {})), scenario.projection_field)
            for log in complete_logs
        )
        validation_status, report, validation_summary = validation(case_id, scenario.authority)
        present = scenario.expected_code in issue_codes(report)
        expected_present = edge == "invalid_edge"
        passed = (
            read_status == 200
            and values_equal(value, actual)
            and bool(complete_logs)
            and (value is None or field_match)
            and validation_status == 200
            and present == expected_present
            and (not expected_present or issue_complete(report, scenario.expected_code))
        )
        add(edge, scenario, "PASS" if passed else "FAIL", status, {
            **save_summary,
            "validation": validation_summary,
            "expected_code": scenario.expected_code,
            "expected_code_present": expected_present,
            "actual_code_present": present,
            "readback": redacted(actual),
            "audit_logs": len(logs),
            "audit_complete": bool(complete_logs),
            "audit_field_match": field_match,
        })

    ordered = scenarios[:]
    random.Random(args.seed).shuffle(ordered)
    for scenario in ordered:
        if interrupted:
            break
        run_edge(scenario, "invalid_edge", scenario.invalid_value)
        if not interrupted:
            run_edge(scenario, "valid_edge", scenario.valid_value)

    if not interrupted:
        status, value, summary = request("GET", "/api/audit-logs/verify-integrity")
        broken = value.get("broken_rows", value.get("brokenRows")) if isinstance(value, dict) else None
        add("audit_chain", None, "PASS" if status == 200 and broken == 0 else "FAIL", status, {
            **summary,
            "broken_rows": broken,
        })

    out_dir = Path(args.artifact_dir)
    out_dir.mkdir(parents=True, exist_ok=True)
    artifact = out_dir / f"case-business-validator-{args.seed}.jsonl"
    with artifact.open("w", encoding="utf-8") as handle:
        for event in events:
            handle.write(json.dumps({"seed": args.seed, "commit": commit_sha(), **asdict(event)}, sort_keys=True) + "\n")
        handle.write(json.dumps({
            "kind": "run",
            "seed": args.seed,
            "requests": requests,
            "elapsed_seconds": round(time.monotonic() - started, 3),
            "interrupted": interrupted,
            "scenario_count": len(scenarios),
            "covered_rules": sorted(covered),
            "inventory_rule_count": len(inventory),
            "uncovered_rules": sorted(inventory - covered),
            "artifact": str(artifact),
            "surface": "case-validation-api",
        }, sort_keys=True) + "\n")
    counts: dict[str, int] = {}
    for event in events:
        counts[event.classification] = counts.get(event.classification, 0) + 1
    print(f"events={len(events)} counts={json.dumps(counts, sort_keys=True)} artifact={artifact}")
    return 2 if interrupted else 1 if any(event.classification != "PASS" for event in events) else 0


if __name__ == "__main__":
    sys.exit(main(parser().parse_args()))
