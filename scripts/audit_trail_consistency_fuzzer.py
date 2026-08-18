#!/usr/bin/env python3
"""Seeded black-box checks for the shared audit contract.

The existing case-editor and presave fuzzers stress their large payloads. This
runner probes the cross-cutting write paths, restores every successful
mutation, and emits a control-by-control 21 CFR 11.10(e) evidence report.
"""

from __future__ import annotations

import argparse
import copy
import io
import json
import os
import random
import re
import sys
import time
import zipfile
from datetime import datetime
from pathlib import Path
from typing import Any, Callable

from rbac_rls_blackbox import ApiClient, commit_sha, guard_target, response_summary


UUID_RE = re.compile(
    r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$"
)
SURFACES = ("case", "case-editor", "presave", "settings", "notices", "user")
FAILURES = {"FAIL", "INCONCLUSIVE"}
TRIGGER_SQL = Path(__file__).resolve().parents[1] / "db/bootstrap/10-triggers.sql"

# The trigger list is the source of truth for audit coverage.  Keeping this
# parser next to the black-box oracle makes a newly added auditable table show
# up as uncovered instead of silently disappearing from the report.
def audit_trigger_tables(path: Path = TRIGGER_SQL) -> tuple[str, ...]:
    text = path.read_text(encoding="utf-8")
    tables = re.findall(
        r"CREATE\s+TRIGGER\s+audit_[^\n]+?\s+ON\s+([a-z_][a-z0-9_]*)",
        text,
        flags=re.IGNORECASE,
    )
    return tuple(dict.fromkeys(tables))


AUDIT_TRIGGER_TABLES = audit_trigger_tables()

# Case child CRUD is deliberately data-driven.  The rich demo case contains
# one row for these resources, so one reversible scalar mutation covers the
# endpoint and its database trigger without inventing dozens of payloads.
CASE_MUTATION_PLANS = (
    ("patient", "patient_information", "/api/cases/{case_id}/patient", "/api/cases/{case_id}/patient", ("patient_initials", "sex")),
    ("patient-identifiers", "patient_identifiers", "/api/cases/{case_id}/patient/identifiers", "/api/cases/{case_id}/patient/identifiers/{id}", ("identifier_value", "identifier_type_code")),
    ("medical-history", "medical_history_episodes", "/api/cases/{case_id}/patient/medical-history", "/api/cases/{case_id}/patient/medical-history/{id}", ("comments", "meddra_code")),
    ("past-drugs", "past_drug_history", "/api/cases/{case_id}/patient/past-drugs", "/api/cases/{case_id}/patient/past-drugs/{id}", ("drug_name", "mpid")),
    ("death-info", "patient_death_information", "/api/cases/{case_id}/patient/death-info", "/api/cases/{case_id}/patient/death-info/{id}", ("autopsy_performed",)),
    ("parents", "parent_information", "/api/cases/{case_id}/patient/parents", "/api/cases/{case_id}/patient/parents/{id}", ("parent_identification",)),
    ("reactions", "reactions", "/api/cases/{case_id}/reactions", "/api/cases/{case_id}/reactions/{id}", ("primary_source_reaction", "outcome")),
    ("drugs", "drug_information", "/api/cases/{case_id}/drugs", "/api/cases/{case_id}/drugs/{id}", ("medicinal_product", "manufacturer_name")),
    ("test-results", "test_results", "/api/cases/{case_id}/test-results", "/api/cases/{case_id}/test-results/{id}", ("test_name", "comments")),
    ("narrative", "narrative_information", "/api/cases/{case_id}/narrative", "/api/cases/{case_id}/narrative", ("case_narrative", "template_title")),
    ("sender-diagnoses", "sender_diagnoses", "/api/cases/{case_id}/narrative/sender-diagnoses", "/api/cases/{case_id}/narrative/sender-diagnoses/{id}", ("diagnosis_meddra_code", "diagnosis_meddra_version")),
    ("case-summaries", "case_summary_information", "/api/cases/{case_id}/narrative/summaries", "/api/cases/{case_id}/narrative/summaries/{id}", ("summary_text", "language_code")),
    ("message-header", "message_headers", "/api/cases/{case_id}/message-header", "/api/cases/{case_id}/message-header", ("batch_number", "message_number")),
    ("safety-report", "safety_report_identification", "/api/cases/{case_id}/safety-report", "/api/cases/{case_id}/safety-report", ("case_identifier", "worldwide_unique_id")),
    ("sender-information", "sender_information", "/api/cases/{case_id}/safety-report/senders", "/api/cases/{case_id}/safety-report/senders/{id}", ("organization", "department")),
    ("primary-sources", "primary_sources", "/api/cases/{case_id}/safety-report/primary-sources", "/api/cases/{case_id}/safety-report/primary-sources/{id}", ("organization", "qualification")),
    ("source-documents", "documents_held_by_sender", "/api/cases/{case_id}/safety-report/documents", "/api/cases/{case_id}/safety-report/documents/{id}", ("document_description", "included_document")),
    ("literature", "literature_references", "/api/cases/{case_id}/safety-report/literature", "/api/cases/{case_id}/safety-report/literature/{id}", ("reference_text",)),
    ("studies", "study_information", "/api/cases/{case_id}/safety-report/studies", "/api/cases/{case_id}/safety-report/studies/{id}", ("study_name", "sponsor_study_number")),
    ("receiver", "receiver_information", "/api/cases/{case_id}/receiver", "/api/cases/{case_id}/receiver", ("organization_name", "department")),
    ("other-identifiers", "other_case_identifiers", "/api/cases/{case_id}/other-identifiers", "/api/cases/{case_id}/other-identifiers/{id}", ("case_identifier", "source_of_identifier")),
    ("linked-reports", "linked_report_numbers", "/api/cases/{case_id}/linked-reports", "/api/cases/{case_id}/linked-reports/{id}", ("linked_report_number",)),
)
NESTED_CASE_MUTATION_PLANS = (
    ("reported-causes", "reported_causes_of_death", "death-info", "/api/cases/{case_id}/patient/death-info/{parent_id}/reported-causes", "/api/cases/{case_id}/patient/death-info/{parent_id}/reported-causes/{id}", ("comments", "meddra_code")),
    ("autopsy-causes", "autopsy_causes_of_death", "death-info", "/api/cases/{case_id}/patient/death-info/{parent_id}/autopsy-causes", "/api/cases/{case_id}/patient/death-info/{parent_id}/autopsy-causes/{id}", ("comments", "meddra_code")),
    ("parent-medical-history", "parent_medical_history", "parents", "/api/cases/{case_id}/patient/parent/{parent_id}/medical-history", "/api/cases/{case_id}/patient/parent/{parent_id}/medical-history/{id}", ("comments", "meddra_code")),
    ("parent-past-drugs", "parent_past_drug_history", "parents", "/api/cases/{case_id}/patient/parent/{parent_id}/past-drugs", "/api/cases/{case_id}/patient/parent/{parent_id}/past-drugs/{id}", ("drug_name", "mpid")),
    ("drug-active-substances", "drug_active_substances", "drugs", "/api/cases/{case_id}/drugs/{parent_id}/active-substances", "/api/cases/{case_id}/drugs/{parent_id}/active-substances/{id}", ("substance_name", "substance_termid")),
    ("dosages", "dosage_information", "drugs", "/api/cases/{case_id}/drugs/{parent_id}/dosages", "/api/cases/{case_id}/drugs/{parent_id}/dosages/{id}", ("dose_number", "dose_value")),
    ("drug-indications", "drug_indications", "drugs", "/api/cases/{case_id}/drugs/{parent_id}/indications", "/api/cases/{case_id}/drugs/{parent_id}/indications/{id}", ("indication_meddra_code", "indication_meddra_version")),
    ("drug-reaction-assessments", "drug_reaction_assessments", "drugs", "/api/cases/{case_id}/drugs/{parent_id}/reaction-assessments", "/api/cases/{case_id}/drugs/{parent_id}/reaction-assessments/{id}", ("assessment_result", "source_of_assessment")),
    ("study-registrations", "study_registration_numbers", "studies", "/api/cases/{case_id}/safety-report/studies/{parent_id}/registrations", "/api/cases/{case_id}/safety-report/studies/{parent_id}/registrations/{id}", ("registration_number", "country_code")),
    ("study-cross-reported-inds", "study_fda_cross_reported_inds", "studies", "/api/cases/{case_id}/safety-report/studies/{parent_id}/fda-cross-reported-inds", "/api/cases/{case_id}/safety-report/studies/{parent_id}/fda-cross-reported-inds/{id}", ("ind_number", "ind_number_source")),
)
AUDIT_FIELD_ALIASES = {
    "documentDescription": "title",
    "includedDocument": "document_base64",
    "source": "source_of_identifier",
    "caseIdentifier": "case_identifier",
    "reporterCountry": "country_code",
    "reporterOrganization": "organization",
    "reporterDepartment": "department",
    "reporterStreet": "street",
    "reporterCity": "city",
    "reporterState": "state",
    "reporterPostcode": "postcode",
    "reporterTelephone": "telephone",
    "reporterEmail": "email",
    "reporterOrganizationNullFlavor": "organization_null_flavor",
    "reporterDepartmentNullFlavor": "department_null_flavor",
    "reporterStreetNullFlavor": "street_null_flavor",
    "reporterCityNullFlavor": "city_null_flavor",
    "reporterStateNullFlavor": "state_null_flavor",
    "reporterPostcodeNullFlavor": "postcode_null_flavor",
    "reporterTelephoneNullFlavor": "telephone_null_flavor",
    "reporterEmailNullFlavor": "email_null_flavor",
    "primarySourceForRegulatoryPurposes": "primary_source_regulatory",
    "nullificationAmendmentCode": "nullification_code",
    "worldwideUniqueId": "worldwide_unique_id",
    "additionalDocumentsAvailable": "additional_documents_available",
    "otherCaseIdentifiersExist": "other_case_identifiers_exist",
    "combinationProductReportIndicator": "combination_product_report_indicator",
    "localCriteriaReportType": "local_criteria_report_type",
}


def unwrap(value: Any) -> Any:
    if isinstance(value, dict) and "data" in value:
        return value["data"]
    return value


def uuid_value(value: Any) -> str | None:
    return value if isinstance(value, str) and UUID_RE.fullmatch(value) else None


def record_id(value: Any) -> str | None:
    if isinstance(value, dict):
        for key in ("id", "caseId", "case_id", "organizationId", "organization_id"):
            candidate = value.get(key)
            if isinstance(candidate, str) and UUID_RE.fullmatch(candidate):
                return candidate
        for child in value.values():
            found = record_id(child)
            if found:
                return found
    if isinstance(value, list):
        for child in value:
            found = record_id(child)
            if found:
                return found
    return None


def response_records(value: Any) -> list[dict[str, Any]]:
    """Normalize singleton and collection REST responses for generic probes."""
    if isinstance(value, dict):
        return [value] if uuid_value(value.get("id")) else []
    if isinstance(value, list):
        return [item for item in value if isinstance(item, dict)]
    return []


def multipart_file(field: str, filename: str, content: bytes) -> tuple[bytes, str]:
    """Build one-file multipart payload with stdlib only."""
    boundary = f"auditfuzz{random.randrange(10**12):012d}"
    head = (
        f"--{boundary}\r\n"
        f"Content-Disposition: form-data; name=\"{field}\"; filename=\"{filename}\"\r\n"
        "Content-Type: application/octet-stream\r\n\r\n"
    ).encode()
    body = head + content + f"\r\n--{boundary}--\r\n".encode()
    return body, f"multipart/form-data; boundary={boundary}"


def reversible_value(row: dict[str, Any], fields: tuple[str, ...], seed: int) -> tuple[str, Any, Any] | None:
    for field in fields:
        old = row.get(field)
        if isinstance(old, bool):
            return field, old, not old
        if isinstance(old, (int, float)) and not isinstance(old, bool):
            return field, old, old + 1
        if isinstance(old, str) and old.strip():
            if re.fullmatch(r"-?\d+(?:\.\d+)?", old.strip()):
                try:
                    return field, old, str(float(old) + 1)
                except ValueError:
                    pass
            # Short marker stays inside common initials/code/name limits.
            marker = f"AF{seed % 100000:05d}"
            if marker != old:
                return field, old, marker
    return None


def _snake(value: str) -> str:
    value = re.sub(r"([a-z0-9])([A-Z])", r"\1_\2", value)
    return re.sub(r"[^a-zA-Z0-9]+", "_", value).strip("_").lower()


def audit_key_matches(changed: dict[str, Any], payload_path: str) -> bool:
    raw_leaf = payload_path.removeprefix("[]").lstrip(".").split(".")[-1]
    wanted = {
        _snake(raw_leaf),
        _snake(AUDIT_FIELD_ALIASES.get(raw_leaf, raw_leaf)),
    }
    return any(
        wanted.intersection({_snake(part) for part in str(key).split(".") if part})
        for key in changed
    )


def audit_field_key(payload_path: str) -> str:
    raw_leaf = payload_path.removeprefix("[]").lstrip(".").split(".")[-1]
    return _snake(AUDIT_FIELD_ALIASES.get(raw_leaf, raw_leaf))


def field_matches(log: dict[str, Any], field: str) -> bool:
    changed = log.get("changedFields", log.get("changed_fields", {}))
    if not isinstance(changed, dict):
        return False
    return audit_key_matches(changed, field)


def audit_timestamp_valid(value: Any) -> bool:
    if not isinstance(value, str) or not value:
        return False
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return False
    return parsed.tzinfo is not None


def audit_log_complete(log: dict[str, Any], action: str | None = None) -> bool:
    """Shared Part 11 evidence oracle used by every audit-aware fuzzer."""
    def get(*names: str) -> Any:
        return next((log[name] for name in names if name in log), None)

    if any(get(*names) is None for names in (
        ("organizationId", "organization_id"), ("userId", "user_id"),
        ("createdAt", "created_at"), ("prevHash", "prev_hash"),
        ("entryHash", "entry_hash"), ("action",),
    )):
        return False
    if action is not None and get("action") != action:
        return False
    changed = get("changedFields", "changed_fields")
    if action == "DELETE" and changed is None:
        return get("oldValues", "old_values") is not None
    return isinstance(changed, dict) and all(
        isinstance(delta, dict) and "old" in delta and "new" in delta
        for delta in changed.values()
    )


def audit_complete(log: dict[str, Any]) -> bool:
    """Backward-compatible UPDATE-only alias for this runner."""
    return audit_log_complete(log, "UPDATE") and log.get("id") is not None


def audit_evidence(logs: list[dict[str, Any]], action: str) -> dict[str, Any]:
    if action == "DELETE":
        complete = [
            log for log in logs
            if audit_log_complete(log, "DELETE") or (
                audit_log_complete(log, "UPDATE") and audit_soft_delete(log)
            )
        ]
    else:
        complete = [log for log in logs if audit_log_complete(log, action)]
    timestamped = [
        log for log in complete
        if audit_timestamp_valid(log.get("createdAt", log.get("created_at")))
    ]
    return {
        "audit_rows": len(complete),
        "audit_rows_complete": bool(complete) and len(complete) == len(logs) and len(timestamped) == len(complete),
        "audit_row_ids": [log["id"] for log in complete if log.get("id") is not None],
        "audit_actions": [action] if complete else [],
    }


def audit_soft_delete(log: dict[str, Any]) -> bool:
    changed = log.get("changedFields", log.get("changed_fields", {}))
    if not isinstance(changed, dict):
        return False
    for key, delta in changed.items():
        if _snake(str(key).split(".")[-1]) != "deleted" or not isinstance(delta, dict):
            continue
        if delta.get("old") is False and delta.get("new") is True:
            return True
    return False


def build_part11_report(
    events: list[dict[str, Any]],
    requested_surfaces: list[str],
    artifact: Path,
    expected_tables: tuple[str, ...] | None = None,
) -> dict[str, Any]:
    surface_events = {
        surface: [event for event in events if event.get("surface") == surface]
        for surface in requested_surfaces
    }
    successful_mutations = [
        event for surface in requested_surfaces for event in surface_events[surface]
        if event.get("classification") == "PASS" and event.get("audit_rows", 0) > 0
    ]
    successful_audit_events = [
        event for event in events
        if event.get("classification") == "PASS" and event.get("audit_rows", 0) > 0
    ]
    actions = {
        action
        for event in events
        for action in event.get("audit_actions", [])
    }
    all_surfaces_pass = all(
        surface_events[surface]
        and all(event.get("classification") == "PASS" for event in surface_events[surface])
        for surface in requested_surfaces
    )
    chain_pass = any(
        event.get("surface") == "audit_chain" and event.get("classification") == "PASS"
        for event in events
    )
    mutation_guard_pass = any(
        event.get("surface") == "audit_mutation_guard" and event.get("classification") == "PASS"
        for event in events
    )
    observed_tables = {
        str(event.get("table"))
        for event in events
        if event.get("classification") == "PASS" and event.get("table")
    }
    uncovered_tables = sorted(set(expected_tables or ()) - observed_tables)
    uncovered_reasons = {
        table: next(
            (
                event.get("reason")
                for event in events
                if event.get("table") == table and event.get("classification") == "BLOCKED" and event.get("reason")
            ),
            "no successful mutation observed",
        )
        for table in uncovered_tables
    }
    controls = [
        {
            "id": "11.10(e).capture",
            "status": "PASS" if all_surfaces_pass and successful_mutations else "FAIL",
            "evidence": {"surfaces": requested_surfaces, "successful_mutations": len(successful_mutations)},
        },
        {
            "id": "11.10(e).create-modify-delete",
            "status": "PASS" if {"CREATE", "UPDATE", "DELETE"}.issubset(actions) else "NOT_TESTED",
            "evidence": {"actions_observed": sorted(actions)},
        },
        {
            "id": "11.10(e).actor-time-old-new",
            "status": "PASS" if successful_audit_events and all(
                event.get("audit_rows_complete") for event in successful_audit_events
            ) else "FAIL",
            "evidence": {"rows_checked": sum(event.get("audit_rows", 0) for event in successful_audit_events)},
        },
        {
            "id": "11.10(e).integrity",
            "status": "PASS" if chain_pass else "FAIL",
            "evidence": {"hash_chain_verified": chain_pass},
        },
        {
            "id": "11.10(b).copy",
            "status": "PASS" if artifact.exists() and successful_mutations else "FAIL",
            "evidence": {"artifact": str(artifact), "readable": artifact.exists()},
        },
        {
            "id": "11.10(c).protection",
            "status": "PASS" if mutation_guard_pass else "NOT_TESTED",
            "evidence": {"api_mutation_guard": mutation_guard_pass},
        },
        {
            "id": "11.10(e).operation-coverage",
            "status": "PASS" if expected_tables is None or not uncovered_tables else "NOT_TESTED",
            "evidence": {
                "trigger_tables": len(expected_tables or ()),
                "observed_tables": len(observed_tables),
                "uncovered_tables": uncovered_tables,
                "uncovered_reasons": uncovered_reasons,
            },
        },
        {
            "id": "11.10(a).validation",
            "status": "NOT_TESTED",
            "evidence": {"note": "Fuzzer evidence supports validation; IQ/OQ/PQ and approved protocol remain required."},
        },
        {
            "id": "11.10(e).retention",
            "status": "NOT_TESTED",
            "evidence": {"note": "Retention-period evidence must be established by the records-retention procedure."},
        },
    ]
    required = {
        "11.10(e).capture",
        "11.10(e).create-modify-delete",
        "11.10(e).actor-time-old-new",
        "11.10(e).integrity",
        "11.10(b).copy",
        "11.10(c).protection",
    }
    if expected_tables is not None:
        required.add("11.10(e).operation-coverage")
    failed = [control["id"] for control in controls if control["status"] == "FAIL"]
    failed.extend(
        f"event:{event.get('surface', 'unknown')}"
        for event in events
        if event.get("classification") == "FAIL"
    )
    incomplete = [
        control["id"] for control in controls
        if control["id"] in required and control["status"] != "PASS"
    ]
    limitations = [control["id"] for control in controls if control["status"] == "NOT_TESTED"]
    status = "FAIL" if failed else "INCOMPLETE" if incomplete else "PASS"
    return {
        "kind": "part11-audit-trail-report",
        "regulation": "21 CFR 11.10(e)",
        "claim": "AUDIT_TRAIL_TECHNICAL_CONTROLS_DEMONSTRATED" if status == "PASS" else "NOT_ESTABLISHED",
        "scope": "Technical audit-trail controls only; this is not a certification of full 21 CFR Part 11 compliance.",
        "status": status,
        "controls": controls,
        "failed_controls": failed,
        "incomplete_controls": incomplete,
        "limitations": limitations,
    }


def short_error(status: int | None, body: bytes) -> dict[str, Any]:
    summary = response_summary(status, body)
    try:
        value = json.loads(body)
        error = value.get("error", {}) if isinstance(value, dict) else {}
        if isinstance(error, dict) and isinstance(error.get("code"), str):
            summary["error_code"] = error["code"]
    except (UnicodeDecodeError, json.JSONDecodeError):
        pass
    return summary


def run(args: argparse.Namespace) -> int:
    guard_target(args.base_url, args.allow_remote)
    if args.synthetic_user and args.allow_remote:
        raise SystemExit("--synthetic-user is limited to an isolated local target")
    surfaces = [item.strip().lower() for item in args.surfaces.split(",") if item.strip()]
    unknown = set(surfaces) - set(SURFACES)
    if unknown:
        raise SystemExit(f"unknown surfaces: {', '.join(sorted(unknown))}")
    if args.dry_run:
        print(f"seed={args.seed} surfaces={surfaces}")
        return 0
    if not args.email or not args.password:
        raise SystemExit("set E2BR3_ADMIN_EMAIL and E2BR3_ADMIN_PASSWORD")

    rng = random.Random(args.seed)
    rng.shuffle(surfaces)
    client = ApiClient(args.base_url, args.timeout)
    platform_client: ApiClient | None = None
    started = time.monotonic()
    requests = 0
    interrupted: str | None = None
    events: list[dict[str, Any]] = []

    def call(
        method: str,
        path: str,
        payload: dict[str, Any] | bytes | None = None,
        extra_headers: dict[str, str] | None = None,
        content_type: str | None = None,
        client_override: ApiClient | None = None,
    ) -> tuple[int | None, Any, dict[str, Any]]:
        nonlocal requests, interrupted
        if requests >= args.max_actions:
            interrupted = interrupted or "max_actions"
            return None, None, {"status": None}
        if time.monotonic() - started >= args.deadline_seconds:
            interrupted = interrupted or "deadline"
            return None, None, {"status": None}
        requests += 1
        status, body, transport = (client_override or client).request(method, path, payload, content_type, extra_headers)
        summary = short_error(status, body)
        if transport:
            summary["transport_error"] = transport
            interrupted = interrupted or "transport_error"
        if status == 429:
            interrupted = interrupted or "rate_limited"
        if status is not None and status >= 500:
            interrupted = interrupted or "server_error"
        try:
            value = unwrap(json.loads(body))
        except (UnicodeDecodeError, json.JSONDecodeError):
            value = None
        return status, value, summary

    def logs(table: str, row_id: str, client_override: ApiClient | None = None) -> tuple[int | None, list[dict[str, Any]]]:
        status, value, _ = call("GET", f"/api/audit-logs/by-record/{table}/{row_id}", client_override=client_override)
        return status, value if isinstance(value, list) else []

    def all_logs(client_override: ApiClient | None = None) -> tuple[int | None, list[dict[str, Any]]]:
        status, value, _ = call("GET", "/api/audit-logs?list_options.limit=5000&list_options.order_bys=!created_at", client_override=client_override)
        return status, value if isinstance(value, list) else []

    def event(surface: str, classification: str, status: int | None, **detail: Any) -> None:
        events.append({
            "surface": surface,
            "classification": classification,
            "http_status": status,
            "seed": args.seed,
            "commit": commit_sha(),
            **detail,
        })

    def probe(
        surface: str,
        table: str,
        row_id: str,
        field: str,
        mutate: Callable[[], tuple[int | None, Any, dict[str, Any]]],
        restore: Callable[[Any], tuple[int | None, Any, dict[str, Any]]],
        client_override: ApiClient | None = None,
    ) -> None:
        before_status, before = logs(table, row_id, client_override)
        if before_status != 200:
            event(surface, "BLOCKED" if before_status in {401, 403, 404} else "INCONCLUSIVE", before_status, table=table, record_id=row_id)
            return
        before_ids = {int(log["id"]) for log in before if str(log.get("id", "")).isdigit()}
        status, value, summary = mutate()
        if status not in {200, 201, 204}:
            event(surface, "BLOCKED" if status in {401, 403, 404, 409, 422} else "FAIL", status, response=summary, table=table, record_id=row_id)
            return
        after_status, after = logs(table, row_id, client_override)
        fresh = [log for log in after if str(log.get("id", "")).isdigit() and int(log["id"]) not in before_ids]
        matching = [log for log in fresh if field_matches(log, field) and audit_complete(log)]
        restore_status, _, restore_summary = restore(value)
        classification = "PASS" if matching and restore_status in {200, 201, 204} else "FAIL"
        event(surface, classification, status, table=table, record_id=row_id, field=field,
              after_status=after_status, restore_status=restore_status, response=summary,
              restore_response=restore_summary, **audit_evidence(matching, "UPDATE"))

    status, _, summary = call("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
    event("auth", "PASS" if status == 200 else "FAIL", status, response=summary)
    if status != 200:
        interrupted = interrupted or "login_failed"

    current: dict[str, Any] = {}
    if not interrupted:
        status, value, summary = call("GET", "/api/users/me")
        current = value if isinstance(value, dict) else {}
        if status != 200:
            interrupted = interrupted or "identity_failed"
            event("identity", "FAIL", status, response=summary)

    organization_id = uuid_value(current.get("organizationId") or current.get("organization_id")) or record_id(current)
    current_user_id = record_id(current)
    user_id = args.user_id
    case_id = args.case_id

    if not interrupted and args.platform_email and args.platform_password:
        platform_client = ApiClient(args.base_url, args.timeout)
        platform_status, platform_body, platform_transport = platform_client.request(
            "POST", "/auth/v1/login", {"email": args.platform_email, "pwd": args.platform_password}
        )
        event("platform-auth", "PASS" if platform_status == 200 else "BLOCKED", platform_status, response=short_error(platform_status, platform_body), transport_error=platform_transport)
        if platform_status != 200:
            platform_client = None

    if not interrupted and "case" in surfaces and not case_id:
        status, value, _ = call("GET", "/api/cases?list_options.limit=50")
        records = value if isinstance(value, list) else []
        candidates = [item for item in records if isinstance(item, dict) and str(item.get("status", "")).lower() not in {"locked", "submitted"}]
        case_id = record_id(candidates or records)
    if not interrupted and "case" in surfaces:
        if not case_id:
            event("case", "BLOCKED", None, reason="no editable case discovered")
        else:
            status, value, _ = call("GET", f"/api/cases/{case_id}")
            current_case = value if isinstance(value, dict) else {}
            candidates = (
                ("report_year", "report_year"),
                ("mfds_report_type", "mfds_report_type"),
                ("fda_report_type", "fda_report_type"),
            )
            selected = next(
                ((field, payload, current_case.get(field, current_case.get(payload)))
                 for field, payload in candidates
                 if isinstance(current_case.get(field, current_case.get(payload)), str)
                 and current_case.get(field, current_case.get(payload)).strip()),
                None,
            )
            if status != 200 or selected is None:
                event("case", "BLOCKED", status, reason="case has no reversible scalar field")
            else:
                field, payload, old_value = selected
                if field == "report_year":
                    current_year = int(old_value)
                    marker = str(current_year + 1 if current_year < 9999 else current_year - 1)
                elif field == "fda_report_type":
                    marker = "1" if old_value != "1" else "2"
                else:
                    marker = f"AUDIT-FUZZ-{args.seed}-{rng.randrange(100000):05d}"
                probe("case", "cases", case_id, field,
                      lambda: call("PUT", f"/api/cases/{case_id}", {"data": {payload: marker}, "reason_for_change": "audit consistency fuzzer"}),
                      lambda _: call("PUT", f"/api/cases/{case_id}", {"data": {payload: old_value}, "reason_for_change": "audit consistency fuzzer restore"}))

    if not interrupted and "case-editor" in surfaces:
        if not case_id:
            event("case-editor", "BLOCKED", None, reason="no editable case discovered")
        else:
            status, projection, summary = call("GET", f"/api/cases/{case_id}/editor/pages/CI")
            rows = projection.get("rows", {}) if isinstance(projection, dict) else {}
            ci = rows.get("safetyReportIdentification") if isinstance(rows, dict) else None
            old_value = ci.get("transmissionDate", ci.get("transmission_date")) if isinstance(ci, dict) else None
            if status != 200 or not isinstance(old_value, str) or not old_value:
                event("case-editor", "BLOCKED", status, reason="CI transmissionDate is not reversible", response=summary)
            else:
                marker = f"2099{(args.seed % 12) + 1:02d}01" "120000+0900"
                before_status, before = logs("cases", case_id)
                mutation = {"authorities": ["ICH"], "rows": {"safetyReportIdentification": {"transmissionDate": marker}}}
                update_status, _, update_summary = call("PATCH", f"/api/cases/{case_id}/editor/pages/CI", mutation)
                after_status, after = logs("cases", case_id)
                before_ids = {int(item["id"]) for item in before if str(item.get("id", "")).isdigit()}
                matching = [
                    item for item in after
                    if str(item.get("id", "")).isdigit()
                    and int(item["id"]) not in before_ids
                    and item.get("table_name", item.get("tableName")) == "safety_report_identification"
                    and field_matches(item, "transmission_date")
                    and audit_complete(item)
                ]
                restore_status, _, restore_summary = call(
                    "PATCH",
                    f"/api/cases/{case_id}/editor/pages/CI",
                    {"authorities": ["ICH"], "rows": {"safetyReportIdentification": {"transmissionDate": old_value}}},
                ) if update_status == 200 else (None, None, {})
                classification = "PASS" if before_status == 200 and after_status == 200 and matching and restore_status == 200 else ("BLOCKED" if update_status in {401, 403, 404, 409, 422} else "FAIL")
                event("case-editor", classification, update_status, table="safety_report_identification", field="transmission_date", restore_status=restore_status, response=update_summary, restore_response=restore_summary, **audit_evidence(matching, "UPDATE"))

    if not interrupted and args.all_operations:
        if not case_id:
            event("case-operations", "BLOCKED", None, reason="no editable case discovered")
        else:
            discovered: dict[str, dict[str, Any]] = {}
            for name, table, collection_template, detail_template, fields in CASE_MUTATION_PLANS:
                if interrupted:
                    break
                collection = collection_template.format(case_id=case_id)
                detail_status, value, detail_summary = call("GET", collection)
                candidates = response_records(value)
                if detail_status != 200 or not candidates:
                    event("case-operations", "BLOCKED", detail_status, operation=name, table=table, response=detail_summary)
                    continue
                row = candidates[0]
                discovered[name] = row
                row_id = uuid_value(row.get("id"))
                mutation = reversible_value(row, fields, args.seed)
                if not row_id and "{id}" in detail_template:
                    event("case-operations", "BLOCKED", detail_status, operation=name, table=table, reason="row has no id")
                    continue
                if mutation is None:
                    event("case-operations", "BLOCKED", detail_status, operation=name, table=table, record_id=row_id, reason="no reversible scalar")
                    continue
                field, old_value, new_value = mutation
                endpoint = detail_template.format(case_id=case_id, id=row_id or "")
                probe(
                    "case-operations",
                    table,
                    row_id or case_id,
                    field,
                    lambda endpoint=endpoint, field=field, new_value=new_value: call("PUT", endpoint, {"data": {field: new_value}}),
                    lambda _value, endpoint=endpoint, field=field, old_value=old_value: call("PUT", endpoint, {"data": {field: old_value}}),
                )

            for name, table, parent_name, collection_template, detail_template, fields in NESTED_CASE_MUTATION_PLANS:
                if interrupted:
                    break
                parent_id = uuid_value(discovered.get(parent_name, {}).get("id"))
                if not parent_id:
                    event("case-operations", "BLOCKED", None, operation=name, table=table, reason=f"missing parent: {parent_name}")
                    continue
                collection = collection_template.format(case_id=case_id, parent_id=parent_id)
                detail_status, value, detail_summary = call("GET", collection)
                candidates = response_records(value)
                if detail_status != 200 or not candidates:
                    event("case-operations", "BLOCKED", detail_status, operation=name, table=table, response=detail_summary)
                    continue
                row = candidates[0]
                row_id = uuid_value(row.get("id"))
                mutation = reversible_value(row, fields, args.seed)
                if not row_id or mutation is None:
                    event("case-operations", "BLOCKED", detail_status, operation=name, table=table, record_id=row_id, reason="no reversible row")
                    continue
                field, old_value, new_value = mutation
                endpoint = detail_template.format(case_id=case_id, parent_id=parent_id, id=row_id)
                probe(
                    "case-operations",
                    table,
                    row_id,
                    field,
                    lambda endpoint=endpoint, field=field, new_value=new_value: call("PUT", endpoint, {"data": {field: new_value}}),
                    lambda _value, endpoint=endpoint, field=field, old_value=old_value: call("PUT", endpoint, {"data": {field: old_value}}),
                )

            def create_probe_delete(
                name: str,
                table: str,
                create_path: str,
                create_payload: dict[str, Any],
                detail_path: str,
                field: str,
                delete_path: str,
                after_create: Callable[[str], None] | None = None,
            ) -> None:
                before_status, before = all_logs()
                create_status, created, create_summary = call("POST", create_path, {"data": create_payload})
                row_id = record_id(created)
                after_status, after = all_logs()
                before_ids = {int(item["id"]) for item in before if str(item.get("id", "")).isdigit()}
                created_logs = [
                    item for item in after
                    if str(item.get("id", "")).isdigit()
                    and int(item["id"]) not in before_ids
                    and item.get("table_name", item.get("tableName")) == table
                    and item.get("action") in {"CREATE", "UPDATE"}
                ] if before_status == 200 and after_status == 200 else []
                create_action = "CREATE" if any(item.get("action") == "CREATE" for item in created_logs) else "UPDATE"
                create_evidence = audit_evidence(created_logs, create_action)
                if create_status not in {200, 201} or not row_id:
                    event("case-operations", "BLOCKED" if create_status in {401, 403, 404, 409, 422} else "FAIL", create_status, operation=f"{name}-create", table=table, response=create_summary, **create_evidence)
                    return
                create_classification = (
                    "PASS" if create_evidence["audit_rows_complete"]
                    else "BLOCKED" if create_status in {200, 201} and not created_logs
                    else "FAIL"
                )
                event("case-operations", create_classification, create_status, operation=f"{name}-create", table=table, audit_action=create_action, response=create_summary, **create_evidence)
                if after_create:
                    after_create(row_id)
                status, row, row_summary = call("GET", detail_path.format(id=row_id))
                if status != 200 or not isinstance(row, dict):
                    event("case-operations", "FAIL", status, operation=f"{name}-read", table=table, response=row_summary)
                    return
                mutation = reversible_value(row, (field,), args.seed)
                if mutation is not None:
                    field_name, old_value, new_value = mutation
                    probe(
                        "case-operations",
                        table,
                        row_id,
                        field_name,
                        lambda detail_path=detail_path, field_name=field_name, new_value=new_value: call("PUT", detail_path.format(id=row_id), {"data": {field_name: new_value}}),
                        lambda _value, detail_path=detail_path, field_name=field_name, old_value=old_value: call("PUT", detail_path.format(id=row_id), {"data": {field_name: old_value}}),
                    )
                cleanup_before_status, cleanup_before = logs(table, row_id)
                delete_status, _, delete_summary = call("DELETE", delete_path.format(id=row_id))
                cleanup_after_status, cleanup_after = logs(table, row_id)
                before_delete_ids = {int(item["id"]) for item in cleanup_before if str(item.get("id", "")).isdigit()}
                delete_logs = [item for item in cleanup_after if str(item.get("id", "")).isdigit() and int(item["id"]) not in before_delete_ids and (item.get("action") == "DELETE" or audit_soft_delete(item))]
                delete_evidence = audit_evidence(delete_logs, "DELETE")
                event("case-operations", "PASS" if delete_status in {200, 204} and cleanup_after_status == 200 and delete_evidence["audit_rows_complete"] else "FAIL", delete_status, operation=f"{name}-delete", table=table, response=delete_summary, **delete_evidence)

            if case_id:
                create_probe_delete(
                    "source-documents",
                    "documents_held_by_sender",
                    f"/api/cases/{case_id}/safety-report/documents",
                    {"case_id": case_id, "title": f"Audit evidence {args.seed}", "document_base64": "SGVsbG8=", "file_name": "audit.txt", "media_type": "text/plain", "sequence_number": 99},
                    f"/api/cases/{case_id}/safety-report/documents/{{id}}",
                    "title",
                    f"/api/cases/{case_id}/safety-report/documents/{{id}}",
                )
                drug_id = uuid_value(discovered.get("drugs", {}).get("id"))
                if drug_id:
                    create_probe_delete(
                        "dosages",
                        "dosage_information",
                        f"/api/cases/{case_id}/drugs/{drug_id}/dosages",
                        {"drug_id": drug_id, "sequence_number": 99, "dose_value": 1, "dose_unit": "mg"},
                        f"/api/cases/{case_id}/drugs/{drug_id}/dosages/{{id}}",
                        "dose_value",
                        f"/api/cases/{case_id}/drugs/{drug_id}/dosages/{{id}}",
                    )
                    reaction_id = uuid_value(discovered.get("reactions", {}).get("id"))
                    if reaction_id:
                        create_probe_delete(
                            "drug-reaction-assessments",
                            "drug_reaction_assessments",
                            f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments",
                            {"drug_id": drug_id, "reaction_id": reaction_id},
                            f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{{id}}",
                            "expectedness",
                            f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{{id}}",
                            after_create=lambda assessment_id, drug_id=drug_id: create_probe_delete(
                                "relatedness-assessments",
                                "relatedness_assessments",
                                f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assessment_id}/relatedness",
                                {"drug_reaction_assessment_id": assessment_id, "sequence_number": 99, "source_of_assessment": "AF", "method_of_assessment": "1", "result_of_assessment": "1"},
                                f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assessment_id}/relatedness/{{id}}",
                                "source_of_assessment",
                                f"/api/cases/{case_id}/drugs/{drug_id}/reaction-assessments/{assessment_id}/relatedness/{{id}}",
                            ),
                        )
                study_id = uuid_value(discovered.get("studies", {}).get("id"))
                if study_id:
                    create_probe_delete(
                        "study-cross-reported-inds",
                        "study_fda_cross_reported_inds",
                        f"/api/cases/{case_id}/safety-report/studies/{study_id}/fda-cross-reported-inds",
                        {"study_information_id": study_id, "ind_number": f"AF{args.seed % 100000000:08d}", "sequence_number": 99},
                        f"/api/cases/{case_id}/safety-report/studies/{study_id}/fda-cross-reported-inds/{{id}}",
                        "ind_number",
                        f"/api/cases/{case_id}/safety-report/studies/{study_id}/fda-cross-reported-inds/{{id}}",
                    )

    if not interrupted and args.all_operations:
        # Reuse the existing registry-backed presave generator so every root
        # and child presave table gets one real CREATE/UPDATE/DELETE cycle.
        from presave_case_roundtrip_fuzzer import (
            MODEL_GROUPS,
            SECTIONS,
            baseline as presave_baseline,
            load_fields as load_presave_fields,
        )

        presave_fields = load_presave_fields(list(SECTIONS))
        presave_table_by_group = {
            group_name: table for group_name, table in MODEL_GROUPS.values()
        }
        presave_table_by_group.update({
            "gateways": "sender_presave_gateways",
            "responsiblePersons": "sender_presave_responsible_persons",
            "consignees": "receiver_presave_consignees",
            "routes": "receiver_presave_routes",
            "activeSubstances": "product_presave_active_substances",
            "registrationNumbers": "study_presave_registration_numbers",
            "fdaCrossReportedInds": "study_presave_fda_cross_reported_ind_numbers",
            "products": "study_presave_products",
            "reporters": "study_presave_reporters",
        })
        presave_identity_fields = {
            "sender": ("sender", "organizationName"),
            "receiver": ("receiver", "organizationName"),
            "reporter": ("reporter", "organization"),
            "product": ("product", "medicinalProduct"),
            "study": ("study", "studyName"),
            "narrative": ("narrative", "caseNarrative"),
        }
        presave_records: dict[str, tuple[str, dict[str, Any], str]] = {}
        for section in SECTIONS:
            if interrupted:
                break
            _, plural, root_group, _, _ = SECTIONS[section]
            state = presave_baseline(section, presave_fields, args.seed)
            if section == "sender":
                state["gateways"] = [{"sequenceNumber": 99, "gatewayAuthority": "fda", "senderIdentifier": f"AF{args.seed % 100000:05d}"}]
                state["responsiblePersons"] = [{"sequenceNumber": 99, "department": f"AF{args.seed % 100000:05d}"}]
            elif section == "receiver":
                state["consignees"] = [{"sequenceNumber": 99, "name": f"AF{args.seed % 100000:05d}"}]
                state["routes"] = [{"sequenceNumber": 99, "authority": "fda", "receiverLabel": f"AF{args.seed % 1000:03d}", "messageReceiverIdentifier": "AF", "conditionPage": "CI", "conditionFieldCode": "C.1.1", "conditionOperator": "Equal", "conditionValueCode": "AF", "conditionValueLabel": "AF"}]
            elif section == "study":
                state["products"] = [{"sequenceNumber": 99, "productName": f"AF{args.seed % 100000:05d}"}]
                state["reporters"] = [{"sequenceNumber": 99, "reporterOrganization": f"AF{args.seed % 100000:05d}"}]
            if section == "product" and presave_records.get("sender"):
                state["product"]["senderPresaveId"] = presave_records["sender"][0]
                state["product"]["productId"] = f"AF-PRODUCT-{args.seed}"
            if section == "study" and presave_records.get("product"):
                state["study"]["productPresaveId"] = presave_records["product"][0]
            before_status, before = all_logs()
            create_status, created, create_summary = call("POST", f"/api/presaves/{plural}", {"data": {"rows": state}})
            presave_id = record_id(created)
            after_status, after = all_logs()
            new_logs = [
                item for item in after
                if str(item.get("id", "")).isdigit()
                and int(item["id"]) not in {int(log["id"]) for log in before if str(log.get("id", "")).isdigit()}
            ] if before_status == 200 and after_status == 200 else []
            created_tables = {str(item.get("table_name", item.get("tableName"))) for item in new_logs if item.get("action") == "CREATE"}
            if create_status != 201 or not presave_id:
                event("presave-operations", "BLOCKED" if create_status in {401, 403, 404, 409, 422} else "FAIL", create_status, operation=f"{section}-create", response=create_summary)
                continue
            presave_records[section] = (presave_id, state, plural)
            for table in sorted(created_tables):
                evidence = audit_evidence([item for item in new_logs if item.get("table_name", item.get("tableName")) == table and item.get("action") == "CREATE"], "CREATE")
                event("presave-operations", "PASS" if evidence["audit_rows_complete"] else "FAIL", create_status, operation=f"{section}-create", table=table, **evidence)
            root, identity_field = presave_identity_fields[section]
            old_identity = state[root].get(identity_field)
            if not isinstance(old_identity, str) or not old_identity:
                event("presave-operations", "BLOCKED", create_status, operation=f"{section}-update", table=presave_table_by_group[root], reason="missing identity field")
                continue
            changed_state = copy.deepcopy(state)
            changed_state[root][identity_field] = f"AF{args.seed % 100000:05d}"
            root_endpoint = f"/api/presaves/{plural}/{presave_id}"
            endpoint = root_endpoint + ("" if section in {"reporter", "narrative"} else "/details")
            method = "PATCH" if section in {"reporter", "narrative"} else "PUT"
            update_payload = {"data": {identity_field: changed_state[root][identity_field]}}
            restore_payload = {"data": {identity_field: old_identity}}
            if section in {"sender", "receiver", "product", "study"}:
                endpoint = root_endpoint
                method = "PATCH"
            else:
                update_payload = {"data": {"rows": changed_state}}
                restore_payload = {"data": {"rows": state}}
            update_before_status, update_before = all_logs()
            update_before_ids = {int(item["id"]) for item in update_before if str(item.get("id", "")).isdigit()}
            update_status, _, update_summary = call(method, endpoint, update_payload)
            update_after_status, update_after = all_logs()
            update_fresh = [item for item in update_after if str(item.get("id", "")).isdigit() and int(item["id"]) not in update_before_ids]
            root_update_logs = [item for item in update_fresh if item.get("table_name", item.get("tableName")) == presave_table_by_group[root] and item.get("action") == "UPDATE"]
            update_evidence = audit_evidence(root_update_logs[-1:], "UPDATE")
            restore_status, _, restore_summary = call(method, endpoint, restore_payload) if update_status == 200 else (None, None, {})
            event("presave-operations", "PASS" if update_status == 200 and update_after_status == 200 and update_evidence["audit_rows_complete"] and restore_status == 200 else "FAIL", update_status, operation=f"{section}-update", table=presave_table_by_group[root], response=update_summary, restore_status=restore_status, restore_response=restore_summary, **update_evidence)
        # Keep parent presaves alive until dependent product/study rows exist.
        for section, (presave_id, _state, plural) in reversed(list(presave_records.items())):
            cleanup_before_status, cleanup_before = all_logs()
            cleanup_before_ids = {int(item["id"]) for item in cleanup_before if str(item.get("id", "")).isdigit()}
            cleanup_status, _, cleanup_summary = call("DELETE", f"/api/presaves/{plural}/{presave_id}")
            cleanup_after_status, cleanup_after = all_logs()
            cleanup_fresh = [item for item in cleanup_after if str(item.get("id", "")).isdigit() and int(item["id"]) not in cleanup_before_ids]
            for table in sorted({str(item.get("table_name", item.get("tableName"))) for item in cleanup_fresh if item.get("action") == "DELETE" and item.get("table_name", item.get("tableName")) in AUDIT_TRIGGER_TABLES}):
                delete_logs = [item for item in cleanup_fresh if item.get("table_name", item.get("tableName")) == table and item.get("action") == "DELETE"]
                evidence = audit_evidence(delete_logs[-1:], "DELETE")
                event("presave-operations", "PASS" if cleanup_status in {200, 204} and cleanup_after_status == 200 and evidence["audit_rows_complete"] else "FAIL", cleanup_status, operation=f"{section}-delete", table=table, response=cleanup_summary, **evidence)

    if not interrupted and args.all_operations and case_id:
        # Submission is the one write path that fans out into signatures,
        # idempotency, events, dispatch state, and ACK rows.  It is safe to
        # exercise on an isolated UI database; a real gateway may return a
        # controlled BLOCKED result while still proving the signature path.
        submission_case_status, submission_case, _ = call("GET", f"/api/cases/{case_id}")
        submission_case = submission_case if isinstance(submission_case, dict) else {}
        if submission_case_status == 200 and str(submission_case.get("status", "")).lower() != "submitted":
            sender_id: str | None = None
            sender_created = False
            source_before: Any = None
            setup_changed_report_type = False
            sd_status, sd_value, _ = call("GET", f"/api/cases/{case_id}/editor/pages/SD")
            sd_rows = sd_value.get("rows", {}) if isinstance(sd_value, dict) else {}
            sd_sender = sd_rows.get("senderInformation", {}) if isinstance(sd_rows, dict) else {}
            if isinstance(sd_sender, dict):
                source_before = sd_sender.get("sourceSenderPresaveId", sd_sender.get("source_sender_presave_id"))
            create_status, created_sender, create_summary = call(
                "POST",
                "/api/presaves/senders",
                {"data": {"rows": {
                    "sender": {"senderType": "1", "organizationName": f"AUDIT-SUBMISSION-{args.seed}"},
                    "gateways": [{
                        "sequenceNumber": 99,
                        "gatewayAuthority": "fda",
                        "senderIdentifier": "AF",
                        "isDefaultForAuthority": True,
                    }],
                    "responsiblePersons": [],
                }}},
            )
            sender_id = record_id(created_sender)
            sender_created = bool(sender_id)
            if not sender_id:
                event("submission-operations", "BLOCKED" if create_status in {401, 403, 404, 409, 422} else "FAIL", create_status, operation="sender-setup", response=create_summary)
            else:
                patch_status, _, patch_summary = call(
                    "PATCH",
                    f"/api/cases/{case_id}/editor/pages/SD",
                    {"authorities": ["FDA"], "rows": {"senderInformation": {"sourceSenderPresaveId": sender_id}}},
                )
                if patch_status not in {200, 201}:
                    event("submission-operations", "BLOCKED" if patch_status in {401, 403, 404, 409, 422} else "FAIL", patch_status, operation="sender-link", response=patch_summary)
                    sender_id = None

            old_report_type = submission_case.get("fda_report_type")
            if sender_id and not isinstance(old_report_type, str):
                setup_status, _, setup_summary = call(
                    "PUT",
                    f"/api/cases/{case_id}",
                    {"data": {"fda_report_type": "1"}, "reason_for_change": "audit submission setup"},
                )
                setup_changed_report_type = setup_status == 200
                if not setup_changed_report_type:
                    event("submission-operations", "BLOCKED" if setup_status in {401, 403, 404, 409, 422} else "FAIL", setup_status, operation="submission-setup", response=setup_summary)

            if sender_id and (not isinstance(old_report_type, str) or old_report_type.strip()):
                before_status, before = all_logs()
                submit_status, _, submit_summary = call(
                    "POST",
                    f"/api/cases/{case_id}/submissions/fda",
                    {"reason_for_change": "audit submission probe", "e_signature": {"meaning": "execute submission", "password": args.password}},
                    {"x-idempotency-key": f"audit-fuzz-{args.seed}"},
                )
                after_status, after = all_logs()
                before_ids = {int(item["id"]) for item in before if str(item.get("id", "")).isdigit()}
                fresh = [item for item in after if str(item.get("id", "")).isdigit() and int(item["id"]) not in before_ids]
                submission_tables = {"case_submissions", "e_signatures", "submission_events", "submission_dispatch_state", "submission_idempotency", "submission_acks", "submission_receiver_options"}
                observed_submission_tables = {str(item.get("table_name", item.get("tableName"))) for item in fresh if item.get("table_name", item.get("tableName")) in submission_tables}
                required_submission_tables = {"case_submissions", "e_signatures", "submission_events", "submission_dispatch_state", "submission_idempotency"}
                if submit_status == 201:
                    for table in sorted(observed_submission_tables):
                        table_logs = [item for item in fresh if item.get("table_name", item.get("tableName")) == table]
                        action = "CREATE" if any(item.get("action") == "CREATE" for item in table_logs) else "UPDATE"
                        evidence = audit_evidence(table_logs, action)
                        event("submission-operations", "PASS" if evidence["audit_rows_complete"] else "FAIL", submit_status, operation="submission-create", table=table, response=submit_summary, **evidence)
                    missing = sorted(required_submission_tables - observed_submission_tables)
                    if missing:
                        event("submission-operations", "FAIL", submit_status, operation="submission-create", reason="missing submission audit tables", missing_tables=missing, response=submit_summary)
                else:
                    event("submission-operations", "BLOCKED" if submit_status in {400, 401, 403, 404, 409, 422, 502, 503} else "FAIL", submit_status, operation="submission-create", observed_tables=sorted(observed_submission_tables), response=submit_summary)
                if submit_status != 201:
                    restore_rows = {"senderInformation": {"sourceSenderPresaveId": source_before}}
                    call("PATCH", f"/api/cases/{case_id}/editor/pages/SD", {"authorities": ["FDA"], "rows": restore_rows})
                    if setup_changed_report_type:
                        call("PUT", f"/api/cases/{case_id}", {"data": {"fda_report_type": old_report_type}, "reason_for_change": "audit submission cleanup"})
                    if sender_created and sender_id:
                        call("DELETE", f"/api/presaves/senders/{sender_id}")
                else:
                    event("submission-operations", "PASS", submit_status, operation="submission-create", warning="case is submitted by design of the real submission workflow")
        elif submission_case_status == 200:
            event("submission-operations", "BLOCKED", submission_case_status, operation="submission-create", reason="case already submitted")
        else:
            event("submission-operations", "BLOCKED" if submission_case_status in {401, 403, 404, 409, 422} else "FAIL", submission_case_status, operation="submission-create", reason="case unavailable")

    if not interrupted and args.all_operations:
        # Administrative and reference-data writes are deliberately attempted
        # through their public APIs.  A normal CRO user has read-only/no access
        # here, so the result is evidence of the boundary rather than a fake
        # database mutation.
        def import_reference(table: str, path: str, body: bytes, content_type: str, operation: str) -> None:
            evidence_client = platform_client
            before_status, before = all_logs(evidence_client)
            import_status, _, import_summary = call("POST", path, body, content_type=content_type)
            after_status, after = all_logs(evidence_client)
            before_ids = {int(item["id"]) for item in before if str(item.get("id", "")).isdigit()}
            fresh = [item for item in after if str(item.get("id", "")).isdigit() and int(item["id"]) not in before_ids and item.get("table_name", item.get("tableName")) == table]
            action = "CREATE" if any(item.get("action") == "CREATE" for item in fresh) else "UPDATE"
            evidence = audit_evidence(fresh, action)
            classification = "PASS" if import_status == 200 and evidence["audit_rows_complete"] else "BLOCKED" if import_status in {401, 403, 404, 409, 422} else "FAIL" if import_status not in {200, None} else "BLOCKED"
            event("reference-operations", classification, import_status, operation=operation, table=table, response=import_summary, **evidence)

        terminology_version = f"AF{args.seed % 100000:05d}"
        meddra_zip = io.BytesIO()
        with zipfile.ZipFile(meddra_zip, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.writestr("llt.asc", f"{args.seed % 100000000}$Audit Fuzzer LLT\n")
            archive.writestr("mdhier.asc", f"{args.seed % 100000000 + 1}$2$3$4$Audit Fuzzer PT$Audit Fuzzer HLT$Audit Fuzzer HLGT$Audit Fuzzer SOC\n")
        meddra_body, meddra_type = multipart_file("file", "meddra.zip", meddra_zip.getvalue())
        import_reference("meddra_terms", f"/api/terminology/import/meddra?version={terminology_version}&language=en", meddra_body, meddra_type, "meddra-import")
        whodrug_csv = f"code,drug_name,atc_code\nAF{args.seed % 100000:05d},Audit Fuzzer Drug,AF01\n".encode()
        whodrug_body, whodrug_type = multipart_file("file", "whodrug.csv", whodrug_csv)
        import_reference("whodrug_products", f"/api/terminology/import/whodrug?version={terminology_version}&language=en", whodrug_body, whodrug_type, "whodrug-import")

        terminology_reads = (
            ("meddra_terms", "/api/terminology/meddra?q=AF&limit=1"),
            ("whodrug_products", "/api/terminology/whodrug?q=AF&limit=1"),
            ("mfds_products", "/api/terminology/mfds-products?q=AF&limit=1"),
            ("iso_countries", "/api/terminology/countries"),
            ("e2b_code_lists", "/api/terminology/code-lists?list_name=FDA"),
        )
        for table, path in terminology_reads:
            read_status, value, read_summary = call("GET", path)
            rows = value if isinstance(value, list) else []
            event(
                "reference-operations",
                "BLOCKED" if read_status in {401, 403, 404, 409, 422} or not rows else "PASS",
                read_status,
                operation="read-only-reference",
                table=table,
                reason="no loaded release or no direct mutation endpoint" if not rows else "read-only endpoint; mutation requires licensed import",
                rows=len(rows),
                response=read_summary,
            )
        event("reference-operations", "BLOCKED", 200, operation="read-only-reference", table="controlled_terminology_terms", reason="controlled terms are release/import managed")
        event("reference-operations", "BLOCKED", 200, operation="read-only-reference", table="mfds_product_substances", reason="substances are release/import managed")
        event("reference-operations", "BLOCKED", 200, operation="case-version-create", table="case_versions", reason="system-generated by XML import; no direct REST mutation")

        operator = platform_client
        org_status, org_value, org_summary = call("GET", "/api/organizations?list_options.limit=20", client_override=operator)
        org_rows = org_value if isinstance(org_value, list) else []
        org = next((row for row in org_rows if isinstance(row, dict) and uuid_value(row.get("id"))), None)
        if org is None:
            event("admin-operations", "BLOCKED", org_status, operation="organization-create-update-delete", table="organizations", reason="platform organization permission unavailable", response=org_summary)
        else:
            org_id = org["id"]
            old_name = org.get("name")
            probe("admin-operations", "organizations", org_id, "name",
                  lambda: call("PUT", f"/api/organizations/{org_id}", {"data": {"name": f"{old_name}-AF"}}, client_override=operator),
                  lambda _: call("PUT", f"/api/organizations/{org_id}", {"data": {"name": old_name}}, client_override=operator),
                  client_override=operator)

        role_status, role_value, role_summary = call("GET", "/api/admin/permission-profiles?organizationId=00000000-0000-0000-0000-000000000001", client_override=operator)
        roles = role_value if isinstance(role_value, list) else []
        custom = next((row for row in roles if isinstance(row, dict) and not row.get("built_in") and uuid_value(row.get("id"))), None)
        if custom:
            role_id = custom["id"]
            old_name = custom.get("name")
            probe("admin-operations", "permission_profiles", role_id, "name",
                  lambda: call("PUT", f"/api/admin/permission-profiles/{role_id}?organizationId=00000000-0000-0000-0000-000000000001", {"data": {"name": f"{old_name}-AF"}}, client_override=operator),
                  lambda _: call("PUT", f"/api/admin/permission-profiles/{role_id}?organizationId=00000000-0000-0000-0000-000000000001", {"data": {"name": old_name}}, client_override=operator),
                  client_override=operator)
        else:
            create_role_status, created_role, role_create_summary = call(
                "POST",
                "/api/admin/permission-profiles?organizationId=00000000-0000-0000-0000-000000000001",
                {"data": {"name": f"Audit Fuzzer {args.seed}", "description": "temporary audit evidence role", "privileges": [], "active": True}},
                client_override=operator,
            )
            role_id = record_id(created_role)
            role_logs_status, role_logs = all_logs(operator)
            fresh_role_logs = [log for log in role_logs if log.get("table_name", log.get("tableName")) == "permission_profiles" and log.get("action") == "CREATE"]
            role_evidence = audit_evidence(fresh_role_logs[-1:], "CREATE")
            if role_id and create_role_status == 201:
                event("admin-operations", "PASS" if role_evidence["audit_rows_complete"] else "FAIL", create_role_status, operation="permission-profile-create", table="permission_profiles", response=role_create_summary, **role_evidence)
                call("PUT", f"/api/admin/permission-profiles/{role_id}?organizationId=00000000-0000-0000-0000-000000000001", {"data": {"active": False}}, client_override=operator)
            else:
                event("admin-operations", "BLOCKED" if create_role_status in {401, 403, 404, 409, 422} else "FAIL", create_role_status, operation="permission-profile-create", table="permission_profiles", response=role_create_summary)

    if not interrupted and "presave" in surfaces:
        status, value, summary = call("GET", "/api/presaves/senders?list_options.limit=1")
        records = value if isinstance(value, list) else []
        sender_id = record_id(records)
        synthetic = False
        if status == 200:
            create_status, created, create_summary = call(
                "POST",
                "/api/presaves/senders",
                {"data": {"rows": {"sender": {"senderType": "1", "organizationName": f"AUDIT-FUZZ-SD-{args.seed}"}, "gateways": [], "responsiblePersons": []}}},
            )
            created_id = record_id(created)
            if created_id:
                sender_id, synthetic = created_id, True
                _, lifecycle_after = logs("sender_presaves", sender_id)
                created_logs = [log for log in lifecycle_after if log.get("action") == "CREATE"]
                create_evidence = audit_evidence(created_logs, "CREATE")
                event("presave-create", "PASS" if create_evidence["audit_rows_complete"] else "FAIL", create_status, table="sender_presaves", **create_evidence)
            elif not sender_id:
                event("presave", "BLOCKED" if create_status in {401, 403, 404, 409, 422} else "FAIL", create_status, reason="sender presave setup failed", response=create_summary)
        if not sender_id:
            event("presave", "BLOCKED", status, reason="no sender presave discovered", response=summary)
        else:
            detail_status, detail, detail_summary = call("GET", f"/api/presaves/senders/{sender_id}/details")
            rows = detail.get("rows", {}) if isinstance(detail, dict) else {}
            sender = rows.get("sender") if isinstance(rows, dict) else None
            old_value = sender.get("organizationName", sender.get("organization_name")) if isinstance(sender, dict) else None
            if detail_status != 200 or not isinstance(old_value, str) or not old_value:
                event("presave", "BLOCKED", detail_status, reason="sender organizationName is not reversible", response=detail_summary)
            else:
                marker = f"AUDIT-FUZZ-SD-{args.seed}-{rng.randrange(100000):05d}"
                before_status, before = logs("sender_presaves", sender_id)
                sender_row = rows.get("sender", {}) if isinstance(rows, dict) else {}
                allowed_sender_keys = {
                    "deleted", "isDefault", "senderType", "organizationName",
                    "organizationNameNotation", "streetAddress", "city", "state",
                    "postcode", "countryCode", "telephone", "fax", "email",
                }
                attempted_sender = {
                    key: value for key, value in sender_row.items()
                    if key in allowed_sender_keys
                }
                attempted_sender["organizationName"] = marker
                attempted = {"sender": attempted_sender}
                update_status, _, update_summary = call("PUT", f"/api/presaves/senders/{sender_id}/details", {"data": {"rows": attempted}})
                after_status, after = logs("sender_presaves", sender_id)
                before_ids = {int(item["id"]) for item in before if str(item.get("id", "")).isdigit()}
                matching = [item for item in after if str(item.get("id", "")).isdigit() and int(item["id"]) not in before_ids and field_matches(item, "organization_name") and audit_complete(item)]
                restore_sender = dict(attempted_sender)
                restore_sender["organizationName"] = old_value
                restore = {"sender": restore_sender}
                restore_status, _, restore_summary = call("PUT", f"/api/presaves/senders/{sender_id}/details", {"data": {"rows": restore}}) if update_status == 200 else (None, None, {})
                classification = "PASS" if before_status == 200 and after_status == 200 and matching and restore_status == 200 else ("BLOCKED" if update_status in {401, 403, 404, 409, 422} else "FAIL")
                event("presave", classification, update_status, table="sender_presaves", record_id=sender_id, field="organization_name", restore_status=restore_status, response=update_summary, restore_response=restore_summary, **audit_evidence(matching, "UPDATE"))
            if synthetic:
                cleanup_status, _, cleanup_summary = call("DELETE", f"/api/presaves/senders/{sender_id}")
                lifecycle_after_status, lifecycle_after = logs("sender_presaves", sender_id)
                deleted_logs = [
                    log for log in lifecycle_after
                    if log.get("action") == "DELETE" or audit_soft_delete(log)
                ]
                cleanup_ok = cleanup_status in {200, 204}
                delete_evidence = audit_evidence(deleted_logs, "DELETE")
                event("presave-delete", "PASS" if cleanup_ok and lifecycle_after_status == 200 and delete_evidence["audit_rows_complete"] else "FAIL", cleanup_status, table="sender_presaves", response=cleanup_summary, **delete_evidence)

    if not interrupted and "settings" in surfaces and organization_id:
        status, value, _ = call("GET", "/api/admin/settings")
        settings = value if isinstance(value, dict) else {}
        old_timezone = settings.get("timezone")
        choices = ["UTC", "Asia/Seoul", "America/New_York"]
        candidate = next((item for item in choices if item != old_timezone), None)
        if status != 200 or not isinstance(old_timezone, str) or not candidate:
            event("settings", "BLOCKED", status, reason="settings unavailable or invalid timezone")
        else:
            probe("settings", "app_settings", organization_id, "timezone",
                  lambda: call("PUT", "/api/admin/settings", {"data": {"timezone": candidate}}),
                  lambda _: call("PUT", "/api/admin/settings", {"data": {"timezone": old_timezone}}))

    if not interrupted and "user" in surfaces:
        synthetic_user = False
        if not args.user_id:
            user_status, user_list, _ = call("GET", "/api/users?list_options.limit=50")
            user_records = user_list if isinstance(user_list, list) else []
            candidate = next(
                (
                    item.get("id") for item in user_records
                    if isinstance(item, dict)
                    and item.get("id") != current_user_id
                    and isinstance(item.get("active"), bool)
                ),
                None,
            )
            if candidate:
                user_id = candidate
            elif args.synthetic_user and user_status == 200:
                create_status, created, create_summary = call(
                    "POST",
                    "/api/users",
                    {"data": {
                        "email": f"audit-fuzz-{args.seed}@example.invalid",
                        "username": f"audit_fuzz_{args.seed}",
                        "pwd_clear": "Audit-Fuzz-2026!",
                    }},
                )
                user_id = record_id(created)
                synthetic_user = bool(user_id)
                create_logs_status, create_logs = logs("users", user_id) if user_id else (None, [])
                create_evidence = audit_evidence([log for log in create_logs if log.get("action") == "CREATE"], "CREATE")
                event("user-create", "PASS" if create_status == 201 and create_logs_status == 200 and create_evidence["audit_rows_complete"] else "FAIL", create_status, table="users", response=create_summary, **create_evidence)
            else:
                user_id = current_user_id
        if not user_id:
            event("user", "BLOCKED", None, reason="no user available for mutation")
        else:
            status, value, _ = call("GET", f"/api/users/{user_id}")
            user = value if isinstance(value, dict) else {}
            old_active = user.get("active")
            if status != 200 or not isinstance(old_active, bool):
                event("user", "BLOCKED", status, reason="user unavailable or active flag missing")
            else:
                probe("user", "users", user_id, "active",
                      lambda: call("PUT", f"/api/users/{user_id}", {"data": {"active": not old_active}}),
                      lambda _: call("PUT", f"/api/users/{user_id}", {"data": {"active": old_active}}))
            if synthetic_user:
                cleanup_status, _, cleanup_summary = call("PUT", f"/api/users/{user_id}", {"data": {"active": False}})
                if cleanup_status not in {200, 204}:
                    event("user-cleanup", "FAIL", cleanup_status, response=cleanup_summary)

    if not interrupted and "notices" in surfaces:
        status, runtime, _ = call("GET", "/api/settings/runtime")
        runtime = runtime if isinstance(runtime, dict) else {}
        original_notices = runtime.get("notices")
        revision = runtime.get("notices_revision", runtime.get("noticesRevision"))
        if status != 200 or not isinstance(original_notices, list) or not isinstance(revision, str):
            event("notices", "BLOCKED", status, reason="notices unavailable or empty")
        else:
            active_notices = copy.deepcopy(original_notices)
            synthetic_notice = not active_notices
            setup_ok = not synthetic_notice
            if synthetic_notice:
                active_notices = [{
                    "id": f"audit-fuzz-{args.seed}",
                    "title": f"Audit evidence {args.seed}",
                    "body": "temporary audit-trail evidence notice",
                }]
                create_before_status, create_before = all_logs()
                create_status, create_value, create_summary = call("PUT", "/api/admin/notices", {"data": {"notices": active_notices, "revision": revision}})
                create_value = create_value if isinstance(create_value, dict) else {}
                create_after_status, create_after = all_logs()
                create_before_ids = {int(log["id"]) for log in create_before if str(log.get("id", "")).isdigit()}
                created_logs = [
                    log for log in create_after
                    if str(log.get("id", "")).isdigit()
                    and int(log["id"]) not in create_before_ids
                    and log.get("table_name", log.get("tableName")) == "dashboard_notices"
                    and log.get("action") == "CREATE"
                ]
                create_evidence = audit_evidence(created_logs, "CREATE")
                event("notices-create", "PASS" if create_status == 200 and create_before_status == 200 and create_after_status == 200 and create_evidence["audit_rows_complete"] else "FAIL", create_status, table="dashboard_notices", response=create_summary, **create_evidence)
                if create_status != 200:
                    event("notices", "BLOCKED" if create_status in {401, 403, 404, 409, 422} else "FAIL", create_status, reason="synthetic notice setup failed", response=create_summary)
                else:
                    revision = create_value.get("revision", revision)
                    setup_ok = True

            if setup_ok:
                before_status, before = all_logs()
                changed = copy.deepcopy(active_notices)
                changed[0]["title"] = f"{changed[0].get('title', 'notice')} [audit-{args.seed}]"
                update_status, update_value, update_summary = call("PUT", "/api/admin/notices", {"data": {"notices": changed, "revision": revision}})
                update_value = update_value if isinstance(update_value, dict) else {}
                after_status, after = all_logs()
                before_ids = {int(log["id"]) for log in before if str(log.get("id", "")).isdigit()}
                matching = [log for log in after if str(log.get("id", "")).isdigit() and int(log["id"]) not in before_ids and log.get("table_name", log.get("tableName")) == "dashboard_notices" and field_matches(log, "title") and audit_complete(log)]
                new_revision = update_value.get("revision", revision)
                restore_status, _, restore_summary = call("PUT", "/api/admin/notices", {"data": {"notices": original_notices, "revision": new_revision}}) if update_status == 200 else (None, None, {})
                classification = "PASS" if before_status == 200 and after_status == 200 and matching and restore_status == 200 else ("BLOCKED" if update_status in {401, 403, 404, 409, 422} else "FAIL")
                event("notices", classification, update_status, field="title", restore_status=restore_status, response=update_summary, restore_response=restore_summary, **audit_evidence(matching, "UPDATE"))
                if synthetic_notice and restore_status == 200:
                    _, restored_logs = all_logs()
                    deleted_logs = [log for log in restored_logs if log.get("table_name", log.get("tableName")) == "dashboard_notices" and log.get("action") == "DELETE"]
                    delete_evidence = audit_evidence(deleted_logs, "DELETE")
                    event("notices-delete", "PASS" if delete_evidence["audit_rows_complete"] else "FAIL", restore_status, table="dashboard_notices", **delete_evidence)

    if not interrupted:
        status, value, summary = call("GET", "/api/audit-logs/verify-integrity")
        broken = value.get("brokenRows", value.get("broken_rows")) if isinstance(value, dict) else None
        event("audit_chain", "PASS" if status == 200 and broken == 0 else "FAIL", status, broken_rows=broken, response=summary)

    known_audit_id = next(
        (
            row_id
            for item in events
            for row_id in item.get("audit_row_ids", [])
            if str(row_id).isdigit()
        ),
        None,
    )
    if not interrupted and known_audit_id is not None:
        guard_results = []
        for method in ("PATCH", "DELETE"):
            guard_status, _, guard_summary = call(method, f"/api/audit-logs/{known_audit_id}", {})
            guard_results.append({"method": method, "status": guard_status, "response": guard_summary})
        guard_ok = all(item["status"] in {401, 403, 404, 405} for item in guard_results)
        event("audit_mutation_guard", "PASS" if guard_ok else "FAIL", guard_results[-1]["status"], attempts=guard_results)

    out = Path(args.artifact_dir)
    out.mkdir(parents=True, exist_ok=True)
    artifact = out / f"audit-consistency-{args.seed}.jsonl"
    counts: dict[str, int] = {}
    with artifact.open("w", encoding="utf-8") as handle:
        for item in events:
            counts[item["classification"]] = counts.get(item["classification"], 0) + 1
            handle.write(json.dumps(item, ensure_ascii=True, sort_keys=True) + "\n")
    part11 = build_part11_report(events, surfaces, artifact, expected_tables=AUDIT_TRIGGER_TABLES)
    part11.update({"seed": args.seed, "commit": commit_sha(), "surfaces": surfaces})
    part11_path = out / f"audit-part11-{args.seed}.json"
    part11_path.write_text(json.dumps(part11, ensure_ascii=True, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    with artifact.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps({"kind": "run", "seed": args.seed, "requests": requests, "counts": counts, "interrupted": interrupted, "artifact": str(artifact), "part11_report": str(part11_path), "part11_status": part11["status"], "part11_claim": part11["claim"]}, sort_keys=True) + "\n")
    print(f"events={len(events)} counts={json.dumps(counts, sort_keys=True)} artifact={artifact} part11={part11_path} claim={part11['claim']}")
    if interrupted:
        return 2
    return 1 if part11["status"] != "PASS" or any(item["classification"] in FAILURES for item in events) else 0


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
    value.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "demo.cro.admin@example.com"))
    value.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
    value.add_argument("--platform-email", default=os.getenv("E2BR3_PLATFORM_EMAIL"), help="optional platform operator for global audit/reference tables")
    value.add_argument("--platform-password", default=os.getenv("E2BR3_PLATFORM_PASSWORD"))
    value.add_argument("--seed", type=int, default=int(time.time()))
    value.add_argument("--surfaces", default=",".join(SURFACES))
    value.add_argument("--case-id")
    value.add_argument("--user-id")
    value.add_argument("--synthetic-user", action="store_true", help="create a temporary local-only user when no mutable user exists")
    value.add_argument("--all-operations", action=argparse.BooleanOptionalAction, default=True, help="probe every case CRUD resource listed in the audit trigger inventory")
    value.add_argument("--max-actions", type=int, default=1500)
    value.add_argument("--deadline-seconds", type=float, default=300)
    value.add_argument("--timeout", type=float, default=15)
    value.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz/audit-consistency")
    value.add_argument("--allow-remote", action="store_true")
    value.add_argument("--dry-run", action="store_true")
    return value


if __name__ == "__main__":
    try:
        sys.exit(run(parser().parse_args()))
    except KeyboardInterrupt:
        sys.exit(2)
