#!/usr/bin/env python3
"""Seeded presave save/readback/audit fuzzer with a Case Edit import plan."""

from __future__ import annotations

import argparse
import copy
import json
import os
import subprocess
import sys
import time
from collections import Counter
from pathlib import Path
from typing import Any

import case_editor_input_fuzzer as candidates
from rbac_rls_blackbox import ApiClient, commit_sha, guard_target, response_summary


ROOT = Path(__file__).resolve().parents[1]
REGISTRY = ROOT / "registry/presaves/sections"
SECTIONS = {
    "sender": ("c-sender.json", "senders", "sender", "SD", "Import Sender"),
    "receiver": ("c-receiver.json", "receivers", "receiver", "SD", "Import Receiver"),
    "reporter": ("c-reporter.json", "reporters", "reporter", "RP", "Import Reporter"),
    "product": ("g-product.json", "products", "product", "DG", "Import Product"),
    "study": ("c-study.json", "studies", "study", "SI", "Import"),
    "narrative": ("h-narrative.json", "narratives", "narrative", "NR", "Import Record"),
}
MODEL_GROUPS = {
    "SenderPresave": ("sender", "sender_presaves"),
    "SenderPresaveResponsiblePerson": ("responsiblePersons", "sender_presave_responsible_persons"),
    "ReceiverPresave": ("receiver", "receiver_presaves"),
    "ReporterPresave": ("reporter", "reporter_presaves"),
    "ProductPresave": ("product", "product_presaves"),
    "ProductPresaveActiveSubstance": ("activeSubstances", "product_presave_active_substances"),
    "StudyPresave": ("study", "study_presaves"),
    "StudyPresaveRegistrationNumber": ("registrationNumbers", "study_presave_registration_numbers"),
    "StudyPresaveFdaCrossReportedIndNumber": ("fdaCrossReportedInds", "study_presave_fda_cross_reported_ind_numbers"),
    "NarrativePresave": ("narrative", "narrative_presaves"),
}
GROUPS = {
    "sender": ("sender", "gateways", "responsiblePersons"),
    "receiver": ("receiver", "consignees", "routes"),
    "reporter": ("reporter",),
    "product": ("product", "activeSubstances"),
    "study": ("study", "products", "registrationNumbers", "fdaCrossReportedInds", "reporters"),
    "narrative": ("narrative",),
}
API_FIELD_ALIASES = {
    "substance_termid_version": "substanceTermIdVersion",
    "substance_termid": "substanceTermId",
    "strength_value": "substanceStrengthValue",
    "strength_unit": "substanceStrengthUnit",
}


def camel(value: str) -> str:
    head, *tail = value.split("_")
    return head + "".join(part[:1].upper() + part[1:] for part in tail)


def api_field(value: str) -> str:
    return API_FIELD_ALIASES.get(value, camel(value))


def prepared_contract() -> list[dict[str, Any]]:
    contract = json.loads(candidates.DEFAULT_CONTRACT.read_text(encoding="utf-8"))
    candidates.apply_max_lengths(contract, candidates.load_max_lengths(ROOT))
    candidates.apply_generated_rules(
        contract,
        candidates.load_generated_rules(ROOT, candidates.IDENTIFIER_RE),
        candidates.load_generated_rules(ROOT, candidates.BOOLEAN_RE),
    )
    candidates.expand_null_flavor_contracts(
        contract,
        candidates.load_null_flavor_pairs(candidates.DEFAULT_NULL_FLAVOR_PAIRS),
        candidates.load_dictionary_null_flavors(ROOT),
    )
    return contract


def load_fields(section_names: list[str]) -> list[dict[str, Any]]:
    contract = prepared_contract()
    by_code = {
        field["code"]: {**field, "pageId": page["pageId"]}
        for page in contract
        for field in page.get("fields", [])
        if field.get("code")
    }
    fields: list[dict[str, Any]] = []
    for section in section_names:
        registry_file, _, parent_group, page, _ = SECTIONS[section]
        for row in json.loads((REGISTRY / registry_file).read_text(encoding="utf-8")):
            frontend = row.get("frontend", {})
            backend = row.get("backend", {})
            contract_field = by_code.get(row.get("e2br3_code"))
            model = backend.get("model")
            if not frontend.get("field") or not contract_field or model not in MODEL_GROUPS:
                continue
            group, table = MODEL_GROUPS[model]
            fields.append({
                **copy.deepcopy(contract_field),
                "section": section,
                "pageId": page,
                "frontendField": frontend["field"],
                "backendField": backend["field"],
                "apiField": api_field(backend["field"]),
                "group": group,
                "parentGroup": parent_group,
                "auditTable": table,
            })
    return fields


def mutation_value(field: dict[str, Any], rng: Any, ordinal: int, sample: int) -> Any:
    if ordinal == 0 and "invalidValue" not in field.get("constraint", {}):
        return f"fuzz-{rng.randrange(1_000_000):06d}"
    return candidates.field_value(field, rng, ordinal, sample)


def empty_rows(section: str) -> dict[str, Any]:
    parent = SECTIONS[section][2]
    return {group: {} if group == parent else [] for group in GROUPS[section]}


def group_row(rows: dict[str, Any], field: dict[str, Any]) -> dict[str, Any]:
    value = rows[field["group"]]
    if isinstance(value, list):
        if not value:
            value.append({"sequenceNumber": 1, "deleted": False})
        return value[0]
    return value


def get_value(rows: dict[str, Any], field: dict[str, Any]) -> Any:
    value = rows.get(field["group"])
    if isinstance(value, list):
        value = value[0] if value else {}
    return value.get(field["apiField"]) if isinstance(value, dict) else None


def set_value(rows: dict[str, Any], field: dict[str, Any], value: Any) -> None:
    group_row(rows, field)[field["apiField"]] = value


def parent_record_id(record_id: str, rows: dict[str, Any], field: dict[str, Any]) -> str | None:
    if field["group"] == field["parentGroup"]:
        return record_id
    row = group_row(rows, field)
    value = row.get("id")
    return value if isinstance(value, str) else None


def merge_child_ids(state: dict[str, Any], actual: dict[str, Any]) -> None:
    for group, value in state.items():
        if not isinstance(value, list) or not value:
            continue
        actual_rows = actual.get(group)
        if isinstance(actual_rows, list) and actual_rows and isinstance(actual_rows[0], dict):
            if isinstance(actual_rows[0].get("id"), str):
                value[0]["id"] = actual_rows[0]["id"]


def baseline(section: str, fields: list[dict[str, Any]], seed: int) -> dict[str, Any]:
    rows = empty_rows(section)
    for field in fields:
        if field["section"] != section or candidates.is_nullflavor_field(field):
            continue
        set_value(rows, field, copy.deepcopy(field["roundTripValue"]))
    identity = {
        "sender": ("organizationName", f"FZ-SD-{seed}"),
        "receiver": ("organizationName", f"FZ-SR-{seed}"),
        "reporter": ("organization", f"FZ-RP-{seed}"),
        "study": ("studyName", f"FZ-SI-{seed}"),
        "product": ("medicinalProduct", f"FZ-DG-{seed}"),
        "narrative": ("caseNarrative", f"FZ-NR-{seed}"),
    }[section]
    rows[SECTIONS[section][2]][identity[0]] = identity[1]
    if section == "receiver":
        rows["receiver"]["receiverType"] = "Regulatory Authority"
    if section == "narrative":
        rows["narrative"]["templateTitle"] = identity[1]
    return rows


def transfer_baseline(section: str, fields: list[dict[str, Any]], seed: int) -> dict[str, Any]:
    rows = baseline(section, fields, seed)
    if section == "product":
        # MPID and PhPID are individually valid but mutually exclusive in Case Edit.
        rows["product"]["phpidVersion"] = ""
        rows["product"]["phpid"] = ""
        assert not (rows["product"].get("mpid") and rows["product"].get("phpid"))
    return rows


def unwrap(value: Any) -> Any:
    if isinstance(value, dict) and "data" in value:
        return value["data"]
    return value


def object_id(value: Any) -> str | None:
    return candidates.object_id(value)


def logs_changed(before: list[dict[str, Any]], after: list[dict[str, Any]]) -> list[dict[str, Any]]:
    before_ids = {str(log.get("id")) for log in before}
    return [log for log in after if str(log.get("id")) not in before_ids]


def error_detail(value: Any) -> tuple[str | None, str | None]:
    error = value.get("error", {}) if isinstance(value, dict) else {}
    data = error.get("data", {}) if isinstance(error, dict) else {}
    detail = data.get("detail", {}) if isinstance(data, dict) else {}
    if not isinstance(detail, dict):
        details = error.get("details", {}) if isinstance(error, dict) else {}
        detail = details.get("detail") if isinstance(details, dict) else None
    if not isinstance(detail, dict) or not detail:
        message = error.get("message") if isinstance(error, dict) else None
        return (message, None) if message == "INVALID_REQUEST" else (None, None)
    return detail.get("ruleCode") or detail.get("rule_code"), detail.get("path")


def run(args: argparse.Namespace) -> int:
    guard_target(args.base_url, False)
    requested_sections = [value.strip().lower() for value in args.sections.split(",") if value.strip()]
    unknown = set(requested_sections) - set(SECTIONS)
    if unknown:
        raise SystemExit(f"unknown sections: {', '.join(sorted(unknown))}")
    section_names = [section for section in SECTIONS if section in requested_sections]
    fields = load_fields(section_names)
    if args.dry_run:
        counts = Counter(field["section"] for field in fields)
        mutations = sum(
            candidates.candidate_sample_count(field, ordinal, args.samples_per_category)
            for field in fields
            for ordinal in range(candidates.candidate_count(field, args.values_per_field))
        )
        print(f"fields={len(fields)} mutations={mutations} sections={dict(counts)} seed={args.seed}")
        return 0

    client = ApiClient(args.base_url, args.timeout)
    started = time.monotonic()
    requests = 0
    events: list[dict[str, Any]] = []
    records: dict[str, dict[str, Any]] = {}
    sha = commit_sha()

    def request(method: str, path: str, payload: dict[str, Any] | None = None) -> tuple[int | None, Any, dict[str, Any]]:
        nonlocal requests
        if requests >= args.max_actions or time.monotonic() - started >= args.deadline_seconds:
            return None, None, {"interrupted": "limit"}
        requests += 1
        status, body, transport = client.request(method, path, payload)
        try:
            parsed = json.loads(body)
        except (json.JSONDecodeError, UnicodeDecodeError):
            parsed = None
        summary = response_summary(status, body)
        if transport:
            summary["transport_error"] = transport
        return status, unwrap(parsed), summary

    def audit_logs(table: str, record_id: str | None) -> list[dict[str, Any]]:
        if not record_id:
            return []
        status, value, _ = request("GET", f"/api/audit-logs/by-record/{table}/{record_id}")
        return value if status == 200 and isinstance(value, list) else []

    def event(field: dict[str, Any] | None, classification: str, status: int | None, **detail: Any) -> None:
        events.append({
            "kind": "mutation" if field else detail.pop("kind", "setup"),
            "seed": args.seed,
            "commit": sha,
            "section": field.get("section") if field else detail.pop("section", None),
            "field": field.get("code") if field else None,
            "ordinal": detail.pop("ordinal", None),
            "sample": detail.pop("sample", None),
            "classification": classification,
            "http_status": status,
            **detail,
        })

    status, _, summary = request("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
    event(None, "PASS" if status == 200 else "FAIL", status, kind="login", response=summary)
    if status != 200:
        return write_artifacts(args, events, records, fields, requests, started, None)

    if "product" in section_names:
        status, profile, summary = request("GET", "/api/users/me")
        scope = profile.get("scope", {}) if isinstance(profile, dict) else {}
        blind_allowed = scope.get("accessBlindAllowed") is True
        event(
            None,
            "PASS" if status == 200 and blind_allowed else "FAIL",
            status,
            kind="case_blind_permission",
            section="product",
            blind_allowed=blind_allowed,
            response=summary,
        )

    support_sender_id: str | None = None
    support_product_id: str | None = None

    def ensure_sender_dependency(owner: str) -> str | None:
        nonlocal support_sender_id
        sender_id = records.get("sender", {}).get("id") or support_sender_id
        if sender_id:
            return sender_id
        support = {
            "sender": {
                "senderType": "1",
                "organizationName": f"FZ-SUPPORT-SD-{args.seed}",
            },
            "gateways": [],
            "responsiblePersons": [],
        }
        dep_status, dep_body, dep_summary = request(
            "POST", "/api/presaves/senders", {"data": {"rows": support}}
        )
        support_sender_id = object_id(dep_body)
        event(
            None,
            "PASS" if dep_status == 201 and support_sender_id else "FAIL",
            dep_status,
            kind="dependency",
            section=owner,
            response=dep_summary,
        )
        return support_sender_id

    def ensure_product_dependency() -> str | None:
        nonlocal support_product_id
        product_id = records.get("product", {}).get("id") or support_product_id
        if product_id:
            return product_id
        support = {
            "product": {
                "senderPresaveId": ensure_sender_dependency("study"),
                "productId": f"FZ-SUPPORT-PRODUCT-{args.seed}",
                "medicinalProduct": f"FZ-SUPPORT-DG-{args.seed}",
            },
            "activeSubstances": [],
        }
        dep_status, dep_body, dep_summary = request(
            "POST", "/api/presaves/products", {"data": {"rows": support}}
        )
        support_product_id = object_id(dep_body)
        event(
            None,
            "PASS" if dep_status == 201 and support_product_id else "FAIL",
            dep_status,
            kind="dependency",
            section="study",
            response=dep_summary,
        )
        return support_product_id

    for section in section_names:
        _, plural, _, _, _ = SECTIONS[section]
        state = baseline(section, fields, args.seed)
        if section == "product":
            state["product"]["senderPresaveId"] = ensure_sender_dependency("product")
            state["product"]["productId"] = f"FZ-PRODUCT-{args.seed}"
        if section == "study":
            state["study"]["productPresaveId"] = ensure_product_dependency()
        status, created, summary = request("POST", f"/api/presaves/{plural}", {"data": {"rows": state}})
        record_id = object_id(created)
        event(None, "PASS" if status == 201 and record_id else "FAIL", status, kind="create", section=section, response=summary)
        if not record_id:
            continue
        status, actual, summary = request("GET", f"/api/presaves/{plural}/{record_id}" + ("" if section in {"reporter", "narrative"} else "/details"))
        actual_rows = actual.get("rows", {}) if isinstance(actual, dict) else {}
        if status != 200 or not isinstance(actual_rows, dict):
            event(None, "FAIL", status, kind="readback", section=section, response=summary)
            continue
        merge_child_ids(state, actual_rows)
        records[section] = {"id": record_id, "state": state, "actual": actual_rows}

    by_code = {field["code"]: field for field in fields}
    for field in fields:
        section = field["section"]
        record = records.get(section)
        if not record:
            continue
        _, plural, _, _, _ = SECTIONS[section]
        endpoint = f"/api/presaves/{plural}/{record['id']}" + ("" if section in {"reporter", "narrative"} else "/details")
        method = "PATCH" if section in {"reporter", "narrative"} else "PUT"
        for ordinal in range(candidates.candidate_count(field, args.values_per_field)):
            for sample in range(candidates.candidate_sample_count(field, ordinal, args.samples_per_category)):
                rng = candidates.candidate_rng(args.seed, field, ordinal, sample)
                candidate = mutation_value(field, rng, ordinal, sample)
                before_rows = copy.deepcopy(record["actual"])
                before_value = get_value(before_rows, field)
                attempted = copy.deepcopy(record["state"])
                set_value(attempted, field, candidate)
                partner = by_code.get(field.get("nullFlavorPartnerCode"))
                if partner and partner["section"] == section:
                    if candidates.is_nullflavor_field(field):
                        set_value(attempted, partner, partner["roundTripValue"] if ordinal == 13 else None)
                    else:
                        set_value(attempted, partner, None)
                target_id = parent_record_id(record["id"], record["actual"], field)
                logs_before = audit_logs(field["auditTable"], target_id)
                status, body, summary = request(method, endpoint, {"data": {"rows": attempted}})
                rule, error_path = error_detail(body)
                status_get, actual, _ = request("GET", endpoint)
                actual_rows = actual.get("rows", {}) if isinstance(actual, dict) else {}
                actual_value = get_value(actual_rows, field) if isinstance(actual_rows, dict) else None
                logs_after = audit_logs(field["auditTable"], target_id)
                changed = logs_changed(logs_before, logs_after)
                expectation = candidates.candidate_expectation(field, ordinal)
                if field["code"] == "G.k.2.5" and candidate is False:
                    expectation = ("reject", "ICH.G.k.2.5.ALLOWED.VALUE")
                if candidates.is_nullflavor_field(field):
                    if ordinal == 13:
                        expectation = ("reject", "INPUT_CONTRACT.NULLFLAVOR.PAIR")
                    elif candidates.nullflavor_invalid_candidate(field, candidate):
                        expectation = ("reject", field.get("constraint", {}).get("ruleCode"))
                    elif ordinal in {2, 3}:
                        expectation = ("accept", None)
                expectation_failure = candidates.expectation_error(expectation, status, rule)
                if (
                    expectation_failure
                    and rule == "INPUT.JSON.INVALID"
                    and isinstance(error_path, str)
                    and error_path.endswith(field["apiField"])
                ):
                    expectation_failure = None
                if status == 200 and status_get == 200:
                    changed_value = not candidates.values_equal(actual_value, before_value)
                    matched = [
                        log for log in changed
                        if candidates.audit_key_matches(log.get("changed_fields", log.get("changedFields", {})), field["backendField"])
                    ]
                    complete = any(candidates.audit_log_complete(log) for log in matched)
                    if expectation_failure or (changed_value and not complete) or (not changed_value and changed):
                        classification = "FAIL"
                    elif candidates.values_equal(candidate, actual_value):
                        classification = "SAVE_ACCEPTED"
                    elif candidates.values_equal(actual_value, before_value):
                        classification = "NOOP_ACCEPTED"
                    elif (
                        candidates.is_nullflavor_field(field)
                        and candidate == ""
                        and actual_value is None
                    ):
                        classification = "SAVE_NORMALIZED"
                    else:
                        classification = "FAIL"
                    record["state"] = attempted
                    record["actual"] = actual_rows
                    merge_child_ids(record["state"], actual_rows)
                elif status in {400, 409, 422} and status_get == 200:
                    unchanged = candidates.values_equal(actual_value, before_value)
                    classification = (
                        "CONSTRAINT_REJECTED"
                        if unchanged and not changed and not expectation_failure and rule
                        else "FAIL"
                    )
                else:
                    classification = "FAIL"
                event(
                    field,
                    classification,
                    status,
                    ordinal=ordinal,
                    sample=sample,
                    candidate_kind=candidates.candidate_kind(field, ordinal),
                    candidate=candidates.redacted(candidate),
                    expected=expectation[0] if expectation else None,
                    expected_rule=expectation[1] if expectation else None,
                    actual_rule=rule,
                    error_path=error_path,
                    expectation_failure=expectation_failure,
                    readback_status=status_get,
                    audit_new_logs=len(changed),
                )
                if status is None:
                    break
            if status is None:
                break

    for section, record in records.items():
        _, plural, _, _, _ = SECTIONS[section]
        endpoint = f"/api/presaves/{plural}/{record['id']}" + ("" if section in {"reporter", "narrative"} else "/details")
        method = "PATCH" if section in {"reporter", "narrative"} else "PUT"
        attempted = transfer_baseline(section, fields, args.seed)
        parent = SECTIONS[section][2]
        for key in ("senderPresaveId", "productId", "productPresaveId"):
            value = record["state"].get(parent, {}).get(key)
            if value is not None:
                attempted[parent][key] = value
        merge_child_ids(attempted, record["actual"])
        status, _, _ = request(method, endpoint, {"data": {"rows": attempted}})
        status_get, actual, _ = request("GET", endpoint)
        actual_rows = actual.get("rows", {}) if isinstance(actual, dict) else {}
        section_fields = [field for field in fields if field["section"] == section]
        passed = (
            status == 200
            and status_get == 200
            and all(
                candidates.values_equal(get_value(attempted, field), get_value(actual_rows, field))
                or (get_value(attempted, field) == "" and get_value(actual_rows, field) is None)
                for field in section_fields
            )
        )
        event(
            None,
            "PASS" if passed else "FAIL",
            status,
            kind="transfer_baseline",
            section=section,
            readback_status=status_get,
        )
        if passed:
            record["state"] = attempted
            record["actual"] = actual_rows

    case_id = setup_case(args) if records and not any(event["classification"] == "FAIL" for event in events) else None
    status, integrity, summary = request("GET", "/api/audit-logs/verify-integrity")
    broken = (
        integrity.get("brokenRows", integrity.get("broken_rows"))
        if isinstance(integrity, dict)
        else None
    )
    event(None, "PASS" if status == 200 and broken == 0 else "FAIL", status, kind="audit_chain", response=summary, broken_rows=broken)
    return write_artifacts(args, events, records, fields, requests, started, case_id)


def setup_case(args: argparse.Namespace) -> str | None:
    setup_dir = Path(args.artifact_dir) / "case-setup"
    command = [
        sys.executable,
        str(ROOT / "scripts/case_editor_input_fuzzer.py"),
        "--base-url", args.base_url,
        "--email", args.email,
        "--password", args.password,
        "--seed", str(args.seed),
        "--pages", "DG",
        "--field", "G.k.2.2",
        "--values-per-field", "0",
        "--samples-per-category", "1",
        "--artifact-dir", str(setup_dir),
        "--no-run-gates",
    ]
    subprocess.run(command, cwd=ROOT, check=True)
    artifact = setup_dir / f"case-editor-{args.seed}.jsonl"
    result = json.loads(artifact.read_text(encoding="utf-8").splitlines()[-1])
    return result.get("case_id")


def write_artifacts(
    args: argparse.Namespace,
    events: list[dict[str, Any]],
    records: dict[str, dict[str, Any]],
    fields: list[dict[str, Any]],
    requests: int,
    started: float,
    case_id: str | None,
) -> int:
    out = Path(args.artifact_dir)
    out.mkdir(parents=True, exist_ok=True)
    artifact = out / f"presave-roundtrip-{args.seed}.jsonl"
    counts = Counter(event["classification"] for event in events)
    run_event = {
        "kind": "run",
        "seed": args.seed,
        "case_id": case_id,
        "fields": len(fields),
        "events": len(events),
        "requests": requests,
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "counts": dict(counts),
        "surface": "presave-api+case-edit-ui-plan",
    }
    with artifact.open("w", encoding="utf-8") as handle:
        for event in events:
            handle.write(json.dumps(event, ensure_ascii=True, sort_keys=True) + "\n")
        handle.write(json.dumps(run_event, ensure_ascii=True, sort_keys=True) + "\n")
    plan_records = []
    for section, record in records.items():
        _, _, _, page, import_button = SECTIONS[section]
        expected = []
        for field in fields:
            if field["section"] != section:
                continue
            expected_value = get_value(record["actual"], field)
            if field["code"] == "local.receiver.2":
                expected_value = {
                    "Original Manufacturer": "1",
                    "Regulatory Authority": "2",
                }.get(expected_value, expected_value)
            expected.append({
                "code": field["code"],
                "authority": field.get("authority", "ICH").lower(),
                "projectionPath": field.get("projectionPath"),
                "value": expected_value,
                "auditField": field["backendField"],
            })
        identity_field = {
            "sender": "organizationName",
            "receiver": "organizationName",
            "reporter": "organization",
            "study": "studyName",
            "product": "medicinalProduct",
            "narrative": "templateTitle",
        }[section]
        identity = record["actual"][SECTIONS[section][2]].get(identity_field)
        plan_records.append({
            "section": section,
            "recordId": record["id"],
            "page": page,
            "importButton": import_button,
            "identity": identity,
            "expected": expected,
        })
    plan = {
        "schemaVersion": 2,
        "seed": args.seed,
        "caseId": case_id,
        "coverageMode": "random-presave-fuzz+transfer-safe-case-baseline",
        "records": plan_records,
    }
    plan_path = out / f"presave-ui-plan-{args.seed}.json"
    plan_path.write_text(json.dumps(plan, ensure_ascii=True, indent=2, sort_keys=True), encoding="utf-8")
    print(f"counts={dict(counts)} fields={len(fields)} requests={requests} artifact={artifact} ui_plan={plan_path}")
    return 1 if counts.get("FAIL") or not case_id else 0


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
    value.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "demo.cro.admin@example.com"))
    value.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
    value.add_argument("--seed", type=int, default=int(time.time()))
    value.add_argument("--sections", default=",".join(SECTIONS))
    value.add_argument("--values-per-field", type=int, default=candidates.IDENTIFIER_CANDIDATES)
    value.add_argument("--samples-per-category", type=int, default=1)
    value.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz/presave-case-roundtrip")
    value.add_argument("--max-actions", type=int, default=30000)
    value.add_argument("--deadline-seconds", type=float, default=1800)
    value.add_argument("--timeout", type=float, default=15)
    value.add_argument("--dry-run", action="store_true")
    return value


if __name__ == "__main__":
    sys.exit(run(parser().parse_args()))
