#!/usr/bin/env python3
"""Small public-HTTP stateful RBAC/RLS and presave-scope probe."""

from __future__ import annotations

import argparse
import json
import os
import random
import sys
import time
import uuid
from collections import Counter
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

from rbac_rls_blackbox import (
    ApiClient,
    commit_sha,
    fingerprint,
    guard_target,
    response_summary,
    sanitize_path,
)


ORG = "00000000-0000-0000-0000-000000000001"
COMPANY_ORG = "00000000-0000-0000-0000-000000000002"
UNKNOWN_ORG = "00000000-0000-0000-0000-000000000003"


@dataclass
class Step:
    name: str
    method: str
    path: str
    expected: list[int] | None
    status: str
    response: dict[str, Any]


def json_value(body: bytes) -> Any:
    try:
        return json.loads(body)
    except (UnicodeDecodeError, json.JSONDecodeError):
        return None


def nested(value: Any, *keys: str) -> Any:
    for key in keys:
        if not isinstance(value, dict):
            return None
        value = value.get(key)
    return value


def multipart_xml_probe() -> tuple[bytes, str]:
    boundary = "----e2br3-rbac-probe"
    body = (
        f"--{boundary}\r\n"
        'Content-Disposition: form-data; name="file"; filename="probe.xml"\r\n'
        "Content-Type: application/xml\r\n\r\n"
        "<probe/>\r\n"
        f"--{boundary}--\r\n"
    ).encode()
    return body, f"multipart/form-data; boundary={boundary}"


def privilege(
    menu_key: str,
    read: bool = False,
    edit: bool = False,
    review: bool = False,
    lock: bool = False,
) -> dict[str, Any]:
    return {
        "menu_key": menu_key,
        "can_read": read,
        "can_edit": edit,
        "can_review": review,
        "can_lock": lock,
    }


def main(args: argparse.Namespace) -> int:
    guard_target(args.base_url, args.allow_remote)
    if not args.password:
        raise SystemExit("set E2BR3_ADMIN_PASSWORD")

    prefix = f"rbac-scope-{uuid.uuid4()}"
    rng = random.Random(args.seed)
    password = f"Syn-{uuid.uuid4().hex}-Aa1!"
    email = f"{prefix}@example.com"
    username = prefix[:120]
    actor_sha = commit_sha()
    started = time.monotonic()
    steps: list[Step] = []
    coverage: Counter[str] = Counter()
    polarity: Counter[str] = Counter()
    interrupted: str | None = None
    current_scope = "unknown"
    system = ApiClient(args.base_url, args.timeout)
    cro = ApiClient(args.base_url, args.timeout)
    company = ApiClient(args.base_url, args.timeout)
    user = ApiClient(args.base_url, args.timeout)
    stale_user = ApiClient(args.base_url, args.timeout)
    fresh_user = ApiClient(args.base_url, args.timeout)
    company_user = ApiClient(args.base_url, args.timeout)
    multi_org_legacy = ApiClient(args.base_url, args.timeout)
    multi_org_one = ApiClient(args.base_url, args.timeout)
    multi_org_one_stable = ApiClient(args.base_url, args.timeout)
    multi_org_two = ApiClient(args.base_url, args.timeout)
    company_user_email = f"{prefix}-company-user@example.com"
    company_user_password = f"Syn-{uuid.uuid4().hex}-Bb2!"
    company_profile_id: str | None = None
    company_user_id: str | None = None
    company_product_id: str | None = None
    multi_org_user_id: str | None = None

    def record_coverage(
        path: str,
        permission: str,
        scope: str,
        lifecycle: str,
        outcome: str,
    ) -> None:
        endpoint = sanitize_path(path.split("?", 1)[0])
        coverage["|".join((endpoint, permission, scope, lifecycle))] += 1
        polarity[outcome] += 1

    def call(
        name: str,
        client: ApiClient,
        actor: str,
        method: str,
        path: str,
        payload: dict[str, Any] | bytes | None = None,
        expected: set[int] = {200},
        contains: str | None = None,
        content_type: str | None = None,
        coverage_tags: tuple[str, str, str] | None = None,
        polarity_tag: str | None = None,
    ) -> tuple[int | None, Any]:
        nonlocal interrupted
        if len(steps) >= args.max_actions:
            interrupted = interrupted or "max_actions"
            return None, None
        if time.monotonic() - started >= args.deadline_seconds:
            interrupted = interrupted or "deadline"
            return None, None
        if interrupted:
            return None, None
        request_started = time.monotonic()
        status, body, transport = client.request(
            method, path, payload, content_type
        )
        response = response_summary(status, body)
        if transport:
            response["transport_error"] = transport
            interrupted = interrupted or "transport_error"
        elif status == 429:
            interrupted = interrupted or "rate_limited"
        elif status is not None and status >= 500:
            interrupted = interrupted or "server_error"
        if contains is not None and status in expected:
            response["contains"] = contains in body.decode(errors="replace")
            if contains not in body.decode(errors="replace"):
                status = -1
        response["duration_ms"] = round((time.monotonic() - request_started) * 1000, 2)
        permission, scope, lifecycle = coverage_tags or (
            "read" if method == "GET" else "edit" if method in {"PATCH", "PUT", "POST", "DELETE"} else "unknown",
            current_scope,
            "delete" if method == "DELETE" else "update" if method in {"PATCH", "PUT", "POST"} else "steady",
        )
        record_coverage(
            path,
            permission,
            scope,
            lifecycle,
            polarity_tag or ("positive" if status in expected else "negative"),
        )
        outcome = "PASS" if status in expected else "FAIL"
        steps.append(
            Step(
                name=name,
                method=method,
                path=sanitize_path(path),
                expected=sorted(expected),
                status=outcome,
                response={**response, "http_status": status},
            )
        )
        return status, json_value(body)

    def refresh_cro(name: str) -> bool:
        nonlocal interrupted
        status, _ = call(
            name,
            cro,
            "cro",
            "POST",
            "/auth/v1/login",
            {"email": "demo.cro.admin@example.com", "pwd": args.password},
            expected={200},
        )
        if status != 200:
            interrupted = interrupted or "cro_relogin_failed"
            return False
        return True

    def refresh_company(name: str) -> bool:
        nonlocal interrupted
        status, _ = call(
            name,
            company,
            "company_admin",
            "POST",
            "/auth/v1/login",
            {"email": "demo.company.admin@example.com", "pwd": args.password},
            expected={200},
        )
        if status != 200:
            return False
        return True

    def assert_ids(name: str, ids: list[str], forbidden: list[str]) -> None:
        leaked = sorted(set(ids).intersection(forbidden))
        steps.append(
            Step(
                name=name,
                method="ASSERT",
                path="<response-data>",
                expected=[],
                status="PASS" if not leaked else "FAIL",
                response={"item_count": len(ids), "forbidden_count": len(leaked)},
            )
        )

    def assert_identity(
        name: str,
        value: Any,
        expected_user_id: str,
        expected_organization_id: str,
    ) -> None:
        observed_user_id = nested(value, "data", "id")
        observed_organization_id = nested(value, "data", "organizationId")
        matches = (
            observed_user_id == expected_user_id
            and observed_organization_id == expected_organization_id
        )
        steps.append(
            Step(
                name=name,
                method="ASSERT",
                path="<response-data>",
                expected=[],
                status="PASS" if matches else "FAIL",
                response={
                    "user_id_match": observed_user_id == expected_user_id,
                    "organization_id_match": observed_organization_id == expected_organization_id,
                },
            )
        )

    def study_sender_scope_observation(
        name: str,
        method: str,
        path: str,
        observed_status: int | None,
        study_b_visible: bool,
    ) -> None:
        # Parent scope is part of the authorization invariant: sender-A must
        # not expose an unrelated study-B descendant.
        invariant_holds = observed_status in {403, 404} or not study_b_visible
        steps.append(
            Step(
                name=name,
                method=method,
                path=sanitize_path(path),
                expected=[403, 404],
                status="PASS" if invariant_holds else "FAIL",
                response={
                    "observed_status": observed_status,
                    "study_b_visible": study_b_visible,
                    "policy_source": "context_loader.presave_within_scope(Study): parent product/sender scope",
                    "reason": None if invariant_holds else "scope inheritance violation: sender scope exposed an unrelated study",
                },
            )
        )

    def sender_product_scope_observation(
        name: str,
        method: str,
        path: str,
        observed_status: int | None,
        sender_b_visible: bool,
    ) -> None:
        invariant_holds = observed_status in {403, 404} or not sender_b_visible
        steps.append(
            Step(
                name=name,
                method=method,
                path=sanitize_path(path),
                expected=[403, 404],
                status="PASS" if invariant_holds else "FAIL",
                response={
                    "observed_status": observed_status,
                    "sender_b_visible": sender_b_visible,
                    "policy_source": "context_loader.presave_within_scope(Sender): parent product/study scope",
                    "reason": None if invariant_holds else "scope inheritance violation: product scope exposed an unrelated sender",
                },
            )
        )

    if not refresh_cro("cro_login"):
        interrupted = interrupted or "login_failed"
    status, _ = call("system_login", system, "system", "POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
    if status != 200:
        interrupted = interrupted or "login_failed"
    if interrupted:
        return write(args, steps, interrupted, coverage, polarity)

    profile_name = f"{prefix}-profile"
    profile_payload = {
        "data": {
            "name": profile_name,
            "description": "synthetic stateful scope probe",
            "privileges": [{"menu_key": "info", "can_read": True, "can_edit": True, "can_review": False, "can_lock": False}],
        }
    }
    status, value = call(
        "profile_create",
        system,
        "system",
        "POST",
        f"/api/admin/permission-profiles?organizationId={ORG}",
        profile_payload,
        {201},
    )
    profile_id = value.get("id") if isinstance(value, dict) and status == 201 else None
    call("profile_list", system, "system", "GET", f"/api/admin/permission-profiles?organizationId={ORG}", expected={200})
    if not profile_id:
        return write(args, steps, "profile_create_failed", coverage, polarity)
    call("profile_read", system, "system", "GET", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", expected={200})

    def set_profile(name: str, *privileges: dict[str, Any]) -> None:
        call(
            name,
            system,
            "system",
            "PUT",
            f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
            {"data": {"privileges": list(privileges)}},
            expected={200},
        )

    status, value = call(
        "user_create_custom_role",
        system,
        "system",
        "POST",
        "/api/users",
        {"data": {"organization_id": ORG, "email": email, "username": username, "pwd_clear": password, "role": profile_id}},
        {201},
    )
    user_id = nested(value, "data", "id") if isinstance(value, dict) else None
    if not user_id:
        return write(args, steps, "user_create_failed", coverage, polarity)

    status, _ = call("synthetic_login", user, email, "POST", "/auth/v1/login", {"email": email, "pwd": password})
    if status == 200:
        call("first_login_password_clear", user, email, "POST", "/api/users/me/password", {"data": {"new_password": password}}, expected={204})
        call("stale_token_login", stale_user, email, "POST", "/auth/v1/login", {"email": email, "pwd": password}, expected={200})
    call("custom_profile_read", user, email, "GET", "/api/presaves/senders", expected={200})
    call("custom_profile_lacks_admin", user, email, "GET", "/api/users", expected={403})
    for name, path in (
        ("route_case_denied", "/api/cases?list_options.limit=1"),
        ("route_admin_denied", f"/api/admin/permission-profiles?organizationId={ORG}"),
        ("route_import_denied", "/api/import/xml/history"),
        ("route_export_denied", "/api/submissions/history?list_options.limit=1"),
        ("route_workflow_denied", "/api/cases/workflow/config"),
    ):
        call(name, user, email, "GET", path, expected={403})

    profile_info_case = {
        "data": {
            "privileges": [
                {"menu_key": "info", "can_read": True, "can_edit": True, "can_review": False, "can_lock": False},
                {"menu_key": "case", "can_read": True, "can_edit": False, "can_review": False, "can_lock": False},
            ]
        }
    }
    call("profile_grant_case", system, "system", "PUT", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", profile_info_case, expected={200})
    call("same_session_profile_grant", user, email, "GET", "/api/cases?list_options.limit=1", expected={200})
    profile_route_read = {
        "data": {
            "privileges": [
                {"menu_key": key, "can_read": True, "can_edit": edit, "can_review": False, "can_lock": False}
                for key, edit in (("info", True), ("case", False), ("case_workflow", False), ("import", False), ("export_submission", False), ("admin", False))
            ]
        }
    }
    call("profile_grant_route_matrix", system, "system", "PUT", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", profile_route_read, expected={200})
    for name, path in (
        ("route_case_allowed", "/api/cases?list_options.limit=1"),
        ("route_admin_identity_gate", f"/api/admin/permission-profiles?organizationId={ORG}"),
        ("route_import_allowed", "/api/import/xml/history"),
        ("route_export_allowed", "/api/submissions/history?list_options.limit=1"),
        ("route_workflow_allowed", "/api/cases/workflow/config"),
    ):
        call(name, user, email, "GET", path, expected={403} if name == "route_admin_identity_gate" else {200})
    status, value = call(
        "case_seed_create",
        cro,
        "cro",
        "POST",
        "/api/cases",
        {"data": {"safetyReportIdentification": {"safetyReportId": f"{prefix}-case"}, "status": "draft"}},
        expected={201},
    )
    case_id = nested(value, "data", "id") if status == 201 else None
    if not case_id:
        return write(args, steps, "case_setup_failed", coverage, polarity)

    import_probe, import_content_type = multipart_xml_probe()
    call(
        "case_update_denied",
        user,
        email,
        "PUT",
        f"/api/cases/{case_id}",
        {"data": {"fda_report_type": "INVALID"}},
        expected={403},
    )
    call(
        "workflow_transition_denied",
        user,
        email,
        "POST",
        f"/api/cases/{case_id}/workflow/transition",
        {"data": {"to_status": "__invalid__"}},
        expected={403},
    )
    call(
        "export_denied",
        user,
        email,
        "GET",
        f"/api/cases/{case_id}/export/xml",
        expected={403},
    )
    call(
        "import_validate_denied",
        user,
        email,
        "POST",
        "/api/import/xml/validate",
        import_probe,
        expected={403},
        content_type=import_content_type,
    )
    call(
        "submission_denied",
        user,
        email,
        "POST",
        f"/api/cases/{case_id}/submissions/fda",
        expected={403},
    )

    profile_route_write = {
        "data": {
            "privileges": [
                privilege("info", read=True, edit=True),
                privilege("case", read=True, edit=True),
                privilege("case_workflow", read=True, edit=True),
                privilege("import", read=True, edit=True),
                privilege("export_submission", read=True, edit=True),
                privilege("admin"),
            ]
        }
    }
    call(
        "profile_grant_route_writes",
        system,
        "system",
        "PUT",
        f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
        profile_route_write,
        expected={200},
    )
    call(
        "case_update_permission_enabled",
        user,
        email,
        "PUT",
        f"/api/cases/{case_id}",
        {"data": {"fda_report_type": "INVALID"}},
        expected={400},
    )
    call(
        "workflow_transition_permission_enabled",
        user,
        email,
        "POST",
        f"/api/cases/{case_id}/workflow/transition",
        {"data": {"to_status": "__invalid__"}},
        expected={400},
    )
    call(
        "export_permission_enabled",
        user,
        email,
        "GET",
        f"/api/cases/{case_id}/export/xml",
        expected={400},
    )
    call(
        "import_validate_permission_enabled",
        user,
        email,
        "POST",
        "/api/import/xml/validate",
        import_probe,
        expected={200, 400},
        content_type=import_content_type,
    )
    call(
        "submission_permission_enabled",
        user,
        email,
        "POST",
        f"/api/cases/{case_id}/submissions/fda",
        expected={400},
    )
    call("profile_restore_info_only", system, "system", "PUT", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", {"data": {"privileges": [{"menu_key": "info", "can_read": True, "can_edit": True, "can_review": False, "can_lock": False}]}}, expected={200})
    call("role_change_to_user", system, "system", "PUT", f"/api/users/{user_id}", {"data": {"role": "user"}}, expected={200})
    call("same_session_role_revoke", user, email, "GET", "/api/presaves/senders", expected={403})
    call("role_restore_custom", system, "system", "PUT", f"/api/users/{user_id}", {"data": {"role": profile_id}}, expected={200})
    call("same_session_role_restore", user, email, "GET", "/api/presaves/senders", expected={200})
    call("same_session_profile_revoke", user, email, "GET", "/api/cases?list_options.limit=1", expected={403})

    sender_ids: list[str] = []
    product_ids: list[str] = []
    study_ids: list[str] = []
    # Refresh the CRO snapshot after role/profile mutations; this avoids
    # carrying a stale authorization session into presave setup.
    refresh_cro("cro_relogin_before_presave_setup")
    for label in ("A", "B"):
        refresh_cro(f"cro_relogin_before_sender_{label}")
        status, value = call(
            f"sender_create_{label}", cro, "cro", "POST", "/api/presaves/senders",
            {"data": {"rows": {"sender": {"senderType": "1", "organizationName": f"{prefix}-{label}", "countryCode": "US", "email": f"{prefix}-{label}@example.com"}, "gateways": [], "responsiblePersons": []}}},
            {201},
        )
        sender_id = nested(value, "data", "rows", "sender", "id") if isinstance(value, dict) else None
        if sender_id:
            sender_ids.append(sender_id)
    if len(sender_ids) == 2:
        for label, sender_id in zip(("A", "B"), sender_ids):
            refresh_cro(f"cro_relogin_before_product_{label}")
            status, value = call(
                f"product_create_{label}", cro, "cro", "POST", "/api/presaves/products",
                {"data": {"rows": {"product": {"senderPresaveId": sender_id, "productId": f"{prefix}-{label}", "medicinalProduct": f"{prefix}-{label}"}, "activeSubstances": []}}},
                {201},
            )
            product_id = nested(value, "data", "rows", "product", "id") if isinstance(value, dict) else None
            if product_id:
                product_ids.append(product_id)
    if len(product_ids) == 2:
        for label, product_id in zip(("A", "B"), product_ids):
            refresh_cro(f"cro_relogin_before_study_{label}")
            study_payload = {"data": {"rows": {"study": {"productPresaveId": product_id, "studyName": f"{prefix}-{label}", "sponsorStudyNumber": f"{prefix}-{label}", "studyTypeReaction": "1"}, "products": [], "reporters": [], "registrationNumbers": [], "fdaCrossReportedInds": []}}}
            status, value = call(f"study_create_{label}", cro, "cro", "POST", "/api/presaves/studies", study_payload, expected={201})
            study_id = nested(value, "data", "rows", "study", "id") if isinstance(value, dict) else None
            if study_id:
                study_ids.append(study_id)

        # Company path: company admin creates its own product, then assigns
        # product scope to an ordinary company user. Sender assignment stays a
        # deliberate negative probe; product-only scope is valid by hierarchy.
        company_profile_payload = {
            "data": {
                "name": f"{prefix}-company-profile",
                "description": "synthetic company product-scope probe",
                "privileges": [privilege("info", read=True, edit=True)],
            }
        }
        status, value = call(
            "company_profile_create",
            system,
            "system",
            "POST",
            f"/api/admin/permission-profiles?organizationId={COMPANY_ORG}",
            company_profile_payload,
            expected={201},
        )
        company_profile_id = value.get("id") if isinstance(value, dict) and status == 201 else None

        # Multi-org account matrix: two real users share one email, but each
        # Login binds to explicit organization. Membership-only and
        # email-only ambiguous paths are rejected.
        if company_profile_id:
            status, value = call(
                "multi_org_account_create",
                system,
                "system",
                "POST",
                "/api/users",
                {"data": {
                    "organization_id": COMPANY_ORG,
                    "email": email,
                    "username": f"{prefix}-same-email-org2",
                    "pwd_clear": password,
                    "role": company_profile_id,
                }},
                expected={201},
                coverage_tags=("organization", "multi_org", "create"),
                polarity_tag="positive",
            )
            multi_org_user_id = nested(value, "data", "id") if isinstance(value, dict) else None
            if multi_org_user_id:
                status, _ = call(
                    "multi_org_login_org1",
                    multi_org_one,
                    email,
                    "POST",
                    "/auth/v1/login",
                    {"email": email, "pwd": password, "organizationId": ORG},
                    expected={200},
                    coverage_tags=("organization", "org1", "login"),
                    polarity_tag="positive",
                )
                if status == 200:
                    status, value = call(
                        "multi_org_org1_identity",
                        multi_org_one,
                        email,
                        "GET",
                        "/api/users/me",
                        expected={200},
                        coverage_tags=("organization", "org1", "read"),
                        polarity_tag="positive",
                    )
                    if status == 200:
                        assert_identity("multi_org_org1_identity_exact", value, user_id, ORG)

                status, _ = call(
                    "multi_org_login_org1_stable",
                    multi_org_one_stable,
                    email,
                    "POST",
                    "/auth/v1/login",
                    {"email": email, "pwd": password, "organizationId": ORG},
                    expected={200},
                    coverage_tags=("organization", "org1", "login"),
                    polarity_tag="positive",
                )
                status, _ = call(
                    "multi_org_login_org2",
                    multi_org_two,
                    email,
                    "POST",
                    "/auth/v1/login",
                    {"email": email, "pwd": password, "organizationId": COMPANY_ORG},
                    expected={200},
                    coverage_tags=("organization", "org2", "login"),
                    polarity_tag="positive",
                )
                if status == 200:
                    call(
                        "multi_org_org2_password_clear",
                        multi_org_two,
                        email,
                        "POST",
                        "/api/users/me/password",
                        {"data": {"new_password": password}},
                        expected={204},
                        coverage_tags=("organization", "org2", "update"),
                        polarity_tag="positive",
                    )
                    status, value = call(
                        "multi_org_org2_identity",
                        multi_org_two,
                        email,
                        "GET",
                        "/api/users/me",
                        expected={200},
                        coverage_tags=("organization", "org2", "read"),
                        polarity_tag="positive",
                    )
                    if status == 200:
                        assert_identity("multi_org_org2_identity_exact", value, multi_org_user_id, COMPANY_ORG)

                call(
                    "multi_org_legacy_login_rejected",
                    multi_org_legacy,
                    email,
                    "POST",
                    "/auth/v1/login",
                    {"email": email, "pwd": password},
                    expected={401, 403},
                    coverage_tags=("organization", "ambiguous", "login"),
                    polarity_tag="negative",
                )
                status, _ = call(
                    "multi_org_switch_org1_to_org2",
                    multi_org_one,
                    email,
                    "PUT",
                    "/api/users/me/organization",
                    {"data": {"organization_id": COMPANY_ORG}},
                    expected={200},
                    coverage_tags=("organization", "switch", "update"),
                    polarity_tag="positive",
                )
                if status == 200:
                    status, value = call(
                        "multi_org_switched_identity",
                        multi_org_one,
                        email,
                        "GET",
                        "/api/users/me",
                        expected={200},
                        coverage_tags=("organization", "org2", "read"),
                        polarity_tag="positive",
                    )
                    if status == 200:
                        assert_identity("multi_org_switched_identity_exact", value, multi_org_user_id, COMPANY_ORG)

                call(
                    "multi_org_role_revoke_org2",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{multi_org_user_id}",
                    {"data": {"role": "user"}},
                    expected={200},
                    coverage_tags=("role", "org2", "revoke"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_org2_role_revoked_denied",
                    multi_org_two,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={403},
                    coverage_tags=("read", "org2", "revoke"),
                    polarity_tag="negative",
                )
                call(
                    "multi_org_org1_survives_org2_role_revoke",
                    multi_org_one_stable,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={200},
                    coverage_tags=("read", "org1", "revoke"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_role_restore_org2",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{multi_org_user_id}",
                    {"data": {"role": company_profile_id}},
                    expected={200},
                    coverage_tags=("role", "org2", "restore"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_org2_role_restored",
                    multi_org_two,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={200},
                    coverage_tags=("read", "org2", "restore"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_deactivate_org2",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{multi_org_user_id}",
                    {"data": {"access_end_at": "2000-01-01T00:00:00Z"}},
                    expected={200},
                    coverage_tags=("account", "org2", "deactivate"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_org2_deactivated_denied",
                    multi_org_two,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={401, 403},
                    coverage_tags=("read", "org2", "deactivate"),
                    polarity_tag="negative",
                )
                call(
                    "multi_org_org1_survives_org2_deactivate",
                    multi_org_one_stable,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={200},
                    coverage_tags=("read", "org1", "deactivate"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_reactivate_org2",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{multi_org_user_id}",
                    {"data": {"access_end_at": "2999-01-01T00:00:00Z"}},
                    expected={200},
                    coverage_tags=("account", "org2", "restore"),
                    polarity_tag="positive",
                )
                call(
                    "multi_org_org2_reactivated",
                    multi_org_two,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected={200},
                    coverage_tags=("read", "org2", "restore"),
                    polarity_tag="positive",
                )
            else:
                steps.append(Step("multi_org_account_setup_failed", "N/A", "<same-email-account>", [], "BLOCKED", {"reason": "same-email organization account creation failed"}))
        else:
            steps.append(Step("multi_org_profile_setup_failed", "N/A", "<company-profile>", [], "BLOCKED", {"reason": "company organization permission profile creation failed"}))
        company_ready = refresh_company("company_admin_login")
        company_sender_id: str | None = None
        if company_ready:
            status, value = call(
                "company_admin_sender_list",
                company,
                "company_admin",
                "GET",
                "/api/presaves/senders",
                expected={200},
                coverage_tags=("read", "company", "steady"),
            )
            if status == 200 and isinstance(value, dict):
                for row in nested(value, "data") or []:
                    if isinstance(row, dict):
                        company_sender_id = row.get("id") or nested(row, "rows", "sender", "id")
                        if company_sender_id:
                            break
            if company_sender_id:
                steps.append(Step("company_admin_sender_reuse_existing", "ASSERT", "<company-org>", [], "PASS", {"reason": "company organization permits one active sender; existing sender reused"}))
            elif not interrupted:
                status, value = call(
                    "company_admin_sender_create",
                    company,
                    "company_admin",
                    "POST",
                    "/api/presaves/senders",
                    {"data": {"rows": {"sender": {"senderType": "1", "organizationName": f"{prefix}-company", "countryCode": "KR", "email": f"{prefix}-company@example.com"}, "gateways": [], "responsiblePersons": []}}},
                    expected={201},
                    coverage_tags=("edit", "company", "create"),
                )
                company_sender_id = nested(value, "data", "rows", "sender", "id") if isinstance(value, dict) else None
            if company_sender_id:
                status, value = call(
                    "company_admin_product_create",
                    company,
                    "company_admin",
                    "POST",
                    "/api/presaves/products",
                    {"data": {"rows": {"product": {"senderPresaveId": company_sender_id, "productId": f"{prefix}-company", "medicinalProduct": f"{prefix}-company"}, "activeSubstances": []}}},
                    expected={201},
                    coverage_tags=("edit", "company", "create"),
                )
                company_product_id = nested(value, "data", "rows", "product", "id") if isinstance(value, dict) else None
        elif company_profile_id:
            steps.append(Step("company_flow_blocked", "N/A", "<company-admin-login>", [], "BLOCKED", {"reason": "company admin login failed"}))

        if company_profile_id and company_sender_id:
            status, value = call(
                "company_user_create",
                system,
                "system",
                "POST",
                "/api/users",
                {"data": {"organization_id": COMPANY_ORG, "email": company_user_email, "username": f"{prefix}-company-user", "pwd_clear": company_user_password, "role": company_profile_id}},
                expected={201},
            )
            company_user_id = nested(value, "data", "id") if isinstance(value, dict) else None
            if company_user_id:
                status, _ = call("company_user_login", company_user, "company_user", "POST", "/auth/v1/login", {"email": company_user_email, "pwd": company_user_password}, expected={200})
                if status == 200:
                    call("company_user_password_clear", company_user, "company_user", "POST", "/api/users/me/password", {"data": {"new_password": company_user_password}}, expected={204})
                call("company_admin_sender_scope_denied", company, "company_admin", "PUT", f"/api/users/{company_user_id}", {"data": {"access_sender_ids": [company_sender_id]}}, expected={403}, coverage_tags=("scope", "company", "update"))
                if company_product_id:
                    call("company_admin_product_scope_assign", company, "company_admin", "PUT", f"/api/users/{company_user_id}", {"data": {"access_product_ids": [company_product_id]}}, expected={200}, coverage_tags=("scope", "company_product", "update"))
                    call("company_admin_product_read", company, "company_admin", "GET", f"/api/presaves/products/{company_product_id}", expected={200}, coverage_tags=("read", "company_product", "steady"))
                    call("company_admin_product_write", company, "company_admin", "PATCH", f"/api/presaves/products/{company_product_id}", {"data": {"medicinalProduct": f"{prefix}-company-admin"}}, expected={200}, coverage_tags=("edit", "company_product", "update"))
                    call("company_user_product_read", company_user, "company_user", "GET", f"/api/presaves/products/{company_product_id}", expected={200}, coverage_tags=("read", "company_product", "steady"))
                    call("company_user_product_write", company_user, "company_user", "PATCH", f"/api/presaves/products/{company_product_id}", {"data": {"medicinalProduct": f"{prefix}-company-user"}}, expected={200}, coverage_tags=("edit", "company_product", "update"))
                    call("company_user_foreign_product_denied", company_user, "company_user", "GET", f"/api/presaves/products/{product_ids[0]}", expected={403, 404}, coverage_tags=("read", "cross_org", "steady"))
                else:
                    call("company_admin_product_scope_cross_org_reject", company, "company_admin", "PUT", f"/api/users/{company_user_id}", {"data": {"access_product_ids": [product_ids[0]]}}, expected={400, 403, 404}, coverage_tags=("scope", "cross_org_product", "update"), polarity_tag="negative")
                    if status == 200:
                        call("company_user_foreign_product_denied", company_user, "company_user", "GET", f"/api/presaves/products/{product_ids[0]}", expected={403, 404}, coverage_tags=("read", "cross_org", "steady"))
                    steps.append(Step("company_positive_product_scope", "N/A", "<company-product-public-create>", [], "BLOCKED", {"reason": "product sender linkage is restricted to CRO sponsor admin; no public API seeds a company-org product"}))
            else:
                steps.append(Step("company_user_flow_blocked", "N/A", "<company-user-create>", [], "BLOCKED", {"reason": "company user creation failed"}))
        elif company_profile_id:
            steps.append(Step("company_user_flow_blocked", "N/A", "<company-product>", [], "BLOCKED", {"reason": "company product setup failed"}))

        cro_scope = {"data": {"access_sender_ids": [sender_ids[0]], "access_product_ids": [], "access_study_ids": [], "access_blind_allowed": False, "active_sender_identifier": sender_ids[0]}}
        refresh_cro("cro_relogin_before_scope_sender_a")
        call("scope_sender_a_only", cro, "cro", "PUT", f"/api/users/{user_id}", cro_scope, expected={200})
        status, value = call("sender_list_a_only", user, email, "GET", "/api/presaves/senders", expected={200})
        ids = [row.get("id") for row in (nested(value, "data") or []) if isinstance(row, dict)] if isinstance(value, dict) else []
        assert_ids("sender_list_no_b", ids, [sender_ids[1]])
        status, value = call("product_list_sender_a_only", user, email, "GET", "/api/presaves/products", expected={200})
        ids = [row.get("id") for row in (nested(value, "data") or []) if isinstance(row, dict)] if isinstance(value, dict) else []
        assert_ids("product_list_no_b", ids, [product_ids[1]])
        if len(study_ids) == 2:
            status, value = call("study_list_sender_a_only", user, email, "GET", "/api/presaves/studies", expected={200})
            ids = [row.get("id") for row in (nested(value, "data") or []) if isinstance(row, dict)] if isinstance(value, dict) else []
            study_sender_scope_observation("study_list_sender_a_policy", "GET", "/api/presaves/studies", status, study_ids[1] in ids)
            status, _ = call("study_direct_sender_b", user, email, "GET", f"/api/presaves/studies/{study_ids[1]}", expected={200, 403, 404})
            study_sender_scope_observation("study_direct_sender_b_policy", "GET", f"/api/presaves/studies/{study_ids[1]}", status, status == 200)
            status, _ = call("study_details_sender_b", user, email, "GET", f"/api/presaves/studies/{study_ids[1]}/details", expected={200, 403, 404, 405})
            study_sender_scope_observation("study_details_sender_b_policy", "GET", f"/api/presaves/studies/{study_ids[1]}/details", status, status == 200)
        call("product_direct_foreign_scope", user, email, "GET", f"/api/presaves/products/{product_ids[1]}", expected={403, 404})
        call("product_update_foreign_scope", user, email, "PATCH", f"/api/presaves/products/{product_ids[1]}", {"data": {"medicinalProduct": f"{prefix}-B"}}, expected={403, 404})
        refresh_cro("cro_relogin_before_scope_sender_product_b")
        call("scope_switch_sender_product_b", cro, "cro", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [sender_ids[1]], "access_product_ids": [product_ids[1]], "access_study_ids": [], "active_sender_identifier": sender_ids[1]}}, expected={200})
        call("scope_b_read_after_switch", user, email, "GET", f"/api/presaves/products/{product_ids[1]}", expected={200})
        call("routing_b_select", user, email, "PUT", "/api/users/me/routing", {"data": {"active_sender_identifier": sender_ids[1]}}, expected={200})
        call("routing_foreign_reject", user, email, "PUT", "/api/users/me/routing", {"data": {"active_sender_identifier": sender_ids[0]}}, expected={403})
        refresh_cro("cro_relogin_before_scope_hierarchy")
        call("scope_hierarchy_reject", cro, "cro", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [sender_ids[0]], "access_product_ids": [product_ids[1]], "active_sender_identifier": sender_ids[0]}}, expected={400})
        call("custom_product_scope_a", system, "system", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [], "access_product_ids": [product_ids[0]], "access_study_ids": []}}, expected={200})
        status, value = call("sender_list_product_scope_a", user, email, "GET", "/api/presaves/senders", expected={200})
        ids = [row.get("id") for row in (nested(value, "data") or []) if isinstance(row, dict)] if isinstance(value, dict) else []
        sender_product_scope_observation("sender_list_product_scope_policy", "GET", "/api/presaves/senders", status, sender_ids[1] in ids)
        status, _ = call("sender_direct_product_scope_b", user, email, "GET", f"/api/presaves/senders/{sender_ids[1]}", expected={200, 403, 404})
        sender_product_scope_observation("sender_direct_product_scope_b_policy", "GET", f"/api/presaves/senders/{sender_ids[1]}", status, status == 200)
        status, _ = call("sender_details_product_scope_b", user, email, "GET", f"/api/presaves/senders/{sender_ids[1]}/details", expected={200, 403, 404, 405})
        sender_product_scope_observation("sender_details_product_scope_b_policy", "GET", f"/api/presaves/senders/{sender_ids[1]}/details", status, status == 200)
        call("custom_product_scope_denies_b", user, email, "GET", f"/api/presaves/products/{product_ids[1]}", expected={403, 404})
        if len(study_ids) == 2:
            call("custom_study_scope_a", system, "system", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [], "access_product_ids": [], "access_study_ids": [study_ids[0]]}}, expected={200})
            status, value = call("study_list_scope_a", user, email, "GET", "/api/presaves/studies", expected={200})
            ids = [row.get("id") for row in (nested(value, "data") or []) if isinstance(row, dict)] if isinstance(value, dict) else []
            assert_ids("study_list_no_b", ids, [study_ids[1]])
            call("study_direct_foreign_scope", user, email, "GET", f"/api/presaves/studies/{study_ids[1]}", expected={403, 404})
        refresh_cro("cro_relogin_before_scope_clear")
        call("scope_clear_all", cro, "cro", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [], "access_product_ids": [], "access_study_ids": [], "active_sender_identifier": None}}, expected={200})
        call("empty_scope_allows_product", user, email, "GET", f"/api/presaves/products/{product_ids[0]}", expected={200})
        call("presave_write_with_edit", user, email, "PATCH", f"/api/presaves/products/{product_ids[0]}", {"data": {"medicinalProduct": f"{prefix}-A"}}, expected={200})
        call("profile_read_only", system, "system", "PUT", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", {"data": {"privileges": [{"menu_key": "info", "can_read": True, "can_edit": False, "can_review": False, "can_lock": False}]}}, expected={200})
        call("presave_read_with_read_only", user, email, "GET", f"/api/presaves/products/{product_ids[0]}", expected={200})
        call("presave_write_denied_read_only", user, email, "PATCH", f"/api/presaves/products/{product_ids[0]}", {"data": {"medicinalProduct": f"{prefix}-A"}}, expected={403})
        call("profile_restore_edit", system, "system", "PUT", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", {"data": {"privileges": [{"menu_key": "info", "can_read": True, "can_edit": True, "can_review": False, "can_lock": False}]}}, expected={200})
        call("presave_write_restored", user, email, "PATCH", f"/api/presaves/products/{product_ids[0]}", {"data": {"medicinalProduct": f"{prefix}-A"}}, expected={200})
        refresh_cro("cro_relogin_before_permission_matrix")
        call("scope_sender_a_permission_matrix", cro, "cro", "PUT", f"/api/users/{user_id}", cro_scope, expected={200})
        set_profile("profile_info_none", privilege("info"))
        call("presave_read_denied_none", user, email, "GET", f"/api/presaves/products/{product_ids[0]}", expected={403})
        call("presave_write_denied_none", user, email, "PATCH", f"/api/presaves/products/{product_ids[0]}", {"data": {"medicinalProduct": f"{prefix}-A"}}, expected={403})
        set_profile("profile_info_edit_only", privilege("info", edit=True))
        call("presave_read_implied_by_edit", user, email, "GET", f"/api/presaves/products/{product_ids[0]}", expected={200})
        call("presave_write_edit_only", user, email, "PATCH", f"/api/presaves/products/{product_ids[0]}", {"data": {"medicinalProduct": f"{prefix}-A"}}, expected={200})
        call("presave_read_rls_denied_edit_only", user, email, "GET", f"/api/presaves/products/{product_ids[1]}", expected={403, 404})
        call("presave_write_rls_denied_edit_only", user, email, "PATCH", f"/api/presaves/products/{product_ids[1]}", {"data": {"medicinalProduct": f"{prefix}-B"}}, expected={403, 404})
        set_profile("profile_info_both", privilege("info", read=True, edit=True))
        refresh_cro("cro_relogin_before_scope_clear_after_matrix")
        call("scope_clear_after_permission_matrix", cro, "cro", "PUT", f"/api/users/{user_id}", {"data": {"access_sender_ids": [], "access_product_ids": [], "access_study_ids": [], "active_sender_identifier": None}}, expected={200})
        call("org_switch_unknown_org_reject", user, email, "PUT", "/api/users/me/organization", {"data": {"organization_id": UNKNOWN_ORG}}, expected={403})

        # Seeded matrix: each round changes both the profile and RLS scope,
        # then probes read/write paths through the same authenticated session.
        # Keep this bounded by --max-actions/--deadline; artifacts stay redacted
        # by call() just like the fixed probes above.
        modes = ("none", "read", "edit", "both")
        scope_specs: dict[str, dict[str, Any]] = {
            "sender_a": {"access_sender_ids": [sender_ids[0]], "access_product_ids": [], "access_study_ids": [], "active_sender_identifier": sender_ids[0]},
            "sender_b": {"access_sender_ids": [sender_ids[1]], "access_product_ids": [], "access_study_ids": [], "active_sender_identifier": sender_ids[1]},
            "product_a": {"access_sender_ids": [], "access_product_ids": [product_ids[0]], "access_study_ids": [], "active_sender_identifier": None},
            "product_b": {"access_sender_ids": [], "access_product_ids": [product_ids[1]], "access_study_ids": [], "active_sender_identifier": None},
            "empty": {"access_sender_ids": [], "access_product_ids": [], "access_study_ids": [], "active_sender_identifier": None},
            # Deliberately invalid hierarchy: sender A cannot own product B.
            "foreign": {"access_sender_ids": [sender_ids[0]], "access_product_ids": [product_ids[1]], "access_study_ids": [], "active_sender_identifier": sender_ids[0]},
        }
        valid_scope_names = tuple(name for name in scope_specs if name != "foreign")
        current_scope = "empty"
        # A round is 12 calls. Leave room for profile cleanup so a caller
        # using --max-actions cannot strand a synthetic role/profile.
        round_budget = 12
        matrix_rounds = min(
            args.matrix_rounds,
            max(0, (args.max_actions - len(steps) - 5) // round_budget),
        )

        def profile_payload(mode: str) -> dict[str, Any]:
            can_read = mode in {"read", "edit", "both"}
            can_edit = mode in {"edit", "both"}
            return {
                "data": {
                    "privileges": [
                        privilege("info", read=can_read, edit=can_edit),
                        privilege("case", read=can_read, edit=can_edit),
                    ]
                }
            }

        def product_allowed(scope_name: str, index: int) -> bool:
            return scope_name in {"empty", f"sender_{'a' if index == 0 else 'b'}", f"product_{'a' if index == 0 else 'b'}"}

        for round_no in range(matrix_rounds):
            if interrupted:
                break
            mode = rng.choice(modes)
            scope_name = rng.choice((*valid_scope_names, "foreign"))
            target_index = rng.randrange(2)
            target_product = product_ids[target_index]
            target_label = "ab"[target_index]

            call(
                f"matrix_{round_no:02d}_profile_{mode}",
                system,
                "system",
                "PUT",
                f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                profile_payload(mode),
                expected={200},
            )
            scope_status, _ = call(
                f"matrix_{round_no:02d}_scope_{scope_name}",
                cro,
                "cro",
                "PUT",
                f"/api/users/{user_id}",
                {"data": scope_specs[scope_name]},
                expected={200} if scope_name != "foreign" else {400, 403, 404},
            )
            if scope_name != "foreign" and scope_status == 200:
                current_scope = scope_name

            readable = mode != "none"
            editable = mode in {"edit", "both"}
            allowed = product_allowed(current_scope, target_index)
            direct_expected = {200} if readable and allowed else {403, 404}
            write_expected = {200} if editable and allowed else {403, 404}
            call(
                f"matrix_{round_no:02d}_product_{target_label}_read",
                user,
                email,
                "GET",
                f"/api/presaves/products/{target_product}",
                expected=direct_expected,
            )
            call(
                f"matrix_{round_no:02d}_product_{target_label}_write",
                user,
                email,
                "PATCH",
                f"/api/presaves/products/{target_product}",
                {"data": {"medicinalProduct": f"{prefix}-{target_label}"}},
                expected=write_expected,
            )
            call(
                f"matrix_{round_no:02d}_sender_list",
                user,
                email,
                "GET",
                "/api/presaves/senders",
                expected={200} if readable else {403},
            )
            call(
                f"matrix_{round_no:02d}_case_read",
                user,
                email,
                "GET",
                "/api/cases?list_options.limit=1",
                expected={200} if readable else {403},
            )
            call(
                f"matrix_{round_no:02d}_case_write",
                user,
                email,
                "PUT",
                f"/api/cases/{case_id}",
                {"data": {"fda_report_type": "INVALID"}},
                expected={400} if editable else {403},
            )
            route_target = sender_ids[target_index]
            # Sender chooser follows the same parent/child scope chain.
            route_allowed = current_scope == "empty" or current_scope.endswith(target_label)
            call(
                f"matrix_{round_no:02d}_routing_{target_label}",
                user,
                email,
                "PUT",
                "/api/users/me/routing",
                {"data": {"active_sender_identifier": route_target}},
                expected={200} if route_allowed else {403},
            )

            # Explicit same-session revoke/restore per round. This catches
            # cached authorization decisions instead of only fresh logins.
            call(
                f"matrix_{round_no:02d}_same_session_revoke",
                system,
                "system",
                "PUT",
                f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                profile_payload("none"),
                expected={200},
            )
            call(
                f"matrix_{round_no:02d}_revoke_denied",
                user,
                email,
                "GET",
                f"/api/presaves/products/{target_product}",
                expected={403, 404},
            )
            call(
                f"matrix_{round_no:02d}_same_session_restore",
                system,
                "system",
                "PUT",
                f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                profile_payload(mode),
                expected={200},
            )
            call(
                f"matrix_{round_no:02d}_restore_probe",
                user,
                email,
                "GET",
                f"/api/presaves/products/{target_product}",
                expected={200} if readable and allowed else {403, 404},
            )

        # Adversarial phase: choose state transitions and endpoint probes from
        # a seeded sequence.  The fixed matrix above is a sanity check; this
        # phase deliberately sends malformed, duplicate, cross-org, stale-token
        # and lifecycle inputs so PASS is not just a repeated happy path.
        def adversarial_call(
            name: str,
            client: ApiClient,
            actor: str,
            method: str,
            path: str,
            payload: dict[str, Any] | bytes | None = None,
            expected: set[int] = {200},
            permission: str = "unknown",
            scope: str | None = None,
            lifecycle: str = "steady",
            polarity_tag: str | None = None,
        ) -> tuple[int | None, Any]:
            return call(
                name,
                client,
                actor,
                method,
                path,
                payload,
                expected=expected,
                coverage_tags=(permission, scope or current_scope, lifecycle),
                polarity_tag=polarity_tag,
            )

        def set_adversarial_profile(mode: str, name: str) -> int | None:
            status, _ = adversarial_call(
                name,
                system,
                "system",
                "PUT",
                f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                profile_payload(mode),
                expected={200},
                permission="admin",
                lifecycle="update",
                polarity_tag="positive",
            )
            return status

        # Exercise a second profile's complete lifecycle.  It is synthetic and
        # the only extra cleanup permitted here is its explicit soft-delete.
        ephemeral_profile_id: str | None = None
        status, value = adversarial_call(
            "adversarial_profile_create",
            system,
            "system",
            "POST",
            f"/api/admin/permission-profiles?organizationId={ORG}",
            {"data": {"name": f"{prefix}-adversarial", "description": "seeded negative lifecycle", "privileges": [privilege("info", read=True)]}},
            expected={201},
            permission="admin",
            lifecycle="create",
            polarity_tag="positive",
        )
        if status == 201 and isinstance(value, dict):
            ephemeral_profile_id = value.get("id")
        if ephemeral_profile_id:
            adversarial_call(
                "adversarial_profile_read",
                system,
                "system",
                "GET",
                f"/api/admin/permission-profiles/{ephemeral_profile_id}?organizationId={ORG}",
                expected={200},
                permission="admin",
                lifecycle="read",
                polarity_tag="positive",
            )
            adversarial_call(
                "adversarial_profile_update",
                system,
                "system",
                "PUT",
                f"/api/admin/permission-profiles/{ephemeral_profile_id}?organizationId={ORG}",
                {"data": {"description": f"updated-{args.seed}", "privileges": [privilege("info", read=True, edit=True)]}},
                expected={200},
                permission="admin",
                lifecycle="update",
                polarity_tag="positive",
            )
            adversarial_call(
                "adversarial_profile_delete",
                system,
                "system",
                "DELETE",
                f"/api/admin/permission-profiles/{ephemeral_profile_id}?organizationId={ORG}",
                expected={204},
                permission="admin",
                lifecycle="delete",
                polarity_tag="positive",
            )

        # Sender scope assignment is intentionally delegated to CRO admins;
        # a platform/system admin must be denied on this route.
        adversarial_call(
            "adversarial_system_sender_scope_denied",
            system,
            "system",
            "PUT",
            f"/api/users/{user_id}",
            {"data": {"access_sender_ids": [sender_ids[0]], "active_sender_identifier": sender_ids[0]}},
            expected={403},
            permission="scope",
            scope="sender_a",
            lifecycle="update",
            polarity_tag="negative",
        )

        adversarial_budget = min(
            args.adversarial_actions,
            # role/profile revoke+restore plus random profile/scope can issue
            # six requests in one action; reserve company + base cleanup calls.
            max(0, (args.max_actions - len(steps) - 5) // 6),
        )
        adversarial_kinds = (
            "invalid_privilege",
            "duplicate_privilege",
            "malformed_role",
            "foreign_scope",
            "cross_org_profile",
            "role_revoke_restore",
            "profile_revoke_restore",
            "presave_probe",
            "route_probe",
        )
        modes = ("none", "read", "edit", "both")
        scope_names = tuple(name for name in scope_specs if name != "foreign")
        entity_specs = (
            ("sender", sender_ids[0], "/api/presaves/senders", 0),
            ("sender", sender_ids[1], "/api/presaves/senders", 1),
            ("product", product_ids[0], "/api/presaves/products", 0),
            ("product", product_ids[1], "/api/presaves/products", 1),
            ("study", study_ids[0] if study_ids else None, "/api/presaves/studies", 0),
            ("study", study_ids[1] if len(study_ids) > 1 else None, "/api/presaves/studies", 1),
        )

        def entity_is_allowed(kind: str, index: int) -> bool:
            if current_scope == "empty":
                return True
            if current_scope.startswith(("sender_", "product_", "study_")):
                return index == (0 if current_scope.endswith("a") else 1)
            return False

        def probe_entity(kind: str, entity_id: str, collection: str, index: int) -> None:
            allowed = entity_is_allowed(kind, index)
            readable = current_mode != "none"
            editable = current_mode in {"edit", "both"}
            direct = f"{collection}/{entity_id}"
            operation = rng.choice(("list", "read", "write", "delete", "details", "children"))
            if operation == "list":
                path = collection
                expected = {200} if readable else {403}
                adversarial_call(f"adv_{kind}_list", user, email, "GET", path, expected=expected, permission="read", lifecycle="steady", polarity_tag="positive" if readable else "negative")
            elif operation == "read":
                expected = {200} if readable and allowed else {403, 404}
                adversarial_call(f"adv_{kind}_read", user, email, "GET", direct, expected=expected, permission="read", scope=current_scope, polarity_tag="positive" if expected == {200} else "negative")
            elif operation == "write":
                expected = {200, 400, 404, 422} if editable and allowed else {403, 404}
                adversarial_call(f"adv_{kind}_write", user, email, "PATCH", direct, {"data": {}}, expected=expected, permission="edit", scope=current_scope, polarity_tag="positive" if editable and allowed else "negative")
            elif operation == "delete":
                # Never delete a real synthetic row during fuzzing.  Denial is
                # checked against the real row; enabled mode probes route
                # existence with a foreign UUID.
                target = entity_id if not (editable and allowed) else str(uuid.uuid4())
                expected = {403, 404} if not (editable and allowed) else {400, 404, 405, 422}
                adversarial_call(f"adv_{kind}_delete", user, email, "DELETE", f"{collection}/{target}", expected=expected, permission="edit", scope=current_scope, lifecycle="delete", polarity_tag="negative" if not (editable and allowed) else "positive")
            else:
                suffix = operation
                expected = {200, 400, 404, 405, 422} if readable and allowed else {403, 404, 405}
                adversarial_call(f"adv_{kind}_{suffix}", user, email, "GET", f"{direct}/{suffix}", expected=expected, permission="read", scope=current_scope, lifecycle="route", polarity_tag="positive" if readable and allowed else "negative")

        current_mode = "both"
        for action_no in range(adversarial_budget):
            if interrupted:
                break
            kind = adversarial_kinds[action_no] if action_no < len(adversarial_kinds) else rng.choice(adversarial_kinds)
            if kind == "invalid_privilege":
                adversarial_call(
                    f"adv_{action_no:03d}_unknown_privilege",
                    system,
                    "system",
                    "PUT",
                    f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                    {"data": {"privileges": [privilege("not-a-real-menu", read=True)]}},
                    expected={400, 422},
                    permission="admin",
                    lifecycle="update",
                    polarity_tag="negative",
                )
                set_adversarial_profile(current_mode, f"adv_{action_no:03d}_restore_profile")
            elif kind == "duplicate_privilege":
                duplicate_status, duplicate_value = adversarial_call(
                    f"adv_{action_no:03d}_duplicate_privilege",
                    system,
                    "system",
                    "PUT",
                    f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}",
                    {"data": {"privileges": [privilege("info", read=True), privilege("info", edit=True)]}},
                    # Some APIs safely normalize duplicate keys.  Accept that
                    # only when the response proves the keys were deduplicated.
                    expected={200, 400, 422},
                    permission="admin",
                    lifecycle="update",
                    polarity_tag="negative",
                )
                if duplicate_status == 200:
                    duplicate_rows = duplicate_value.get("privileges", []) if isinstance(duplicate_value, dict) else []
                    duplicate_keys = [row.get("menu_key") for row in duplicate_rows if isinstance(row, dict)]
                    duplicate_keys = [key for key in duplicate_keys if key]
                    steps.append(Step(
                        f"adv_{action_no:03d}_duplicate_privilege_invariant",
                        "ASSERT",
                        "<profile-privileges>",
                        [],
                        "PASS" if len(duplicate_keys) == len(set(duplicate_keys)) else "FAIL",
                        {"privilege_count": len(duplicate_keys), "duplicate_count": len(duplicate_keys) - len(set(duplicate_keys))},
                    ))
                set_adversarial_profile(current_mode, f"adv_{action_no:03d}_restore_duplicate")
            elif kind == "malformed_role":
                adversarial_call(
                    f"adv_{action_no:03d}_malformed_role",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{user_id}",
                    {"data": {"role": rng.choice(("not-a-uuid", "00000000-0000-0000-0000-bad"))}},
                    expected={400, 422},
                    permission="role",
                    lifecycle="update",
                    polarity_tag="negative",
                )
            elif kind == "foreign_scope":
                foreign = str(uuid.uuid4())
                adversarial_call(
                    f"adv_{action_no:03d}_foreign_scope",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{user_id}",
                    {"data": {"access_sender_ids": [foreign], "access_product_ids": [foreign], "access_study_ids": [foreign], "active_sender_identifier": foreign}},
                    expected={400, 403, 404, 422},
                    permission="scope",
                    lifecycle="update",
                    polarity_tag="negative",
                )
            elif kind == "cross_org_profile":
                adversarial_call(
                    f"adv_{action_no:03d}_cross_org_profile",
                    system,
                    "system",
                    "GET",
                    f"/api/admin/permission-profiles/{profile_id}?organizationId={COMPANY_ORG}",
                    expected={400, 403, 404},
                    permission="admin",
                    scope="cross_org",
                    lifecycle="read",
                    polarity_tag="negative",
                )
                adversarial_call(
                    f"adv_{action_no:03d}_cross_org_switch",
                    user,
                    email,
                    "PUT",
                    "/api/users/me/organization",
                    {"data": {"organization_id": UNKNOWN_ORG}},
                    expected={403, 404},
                    permission="organization",
                    scope="cross_org",
                    lifecycle="switch",
                    polarity_tag="negative",
                )
            elif kind == "role_revoke_restore":
                adversarial_call(
                    f"adv_{action_no:03d}_role_revoke",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{user_id}",
                    {"data": {"role": "user"}},
                    expected={200},
                    permission="role",
                    lifecycle="revoke",
                    polarity_tag="positive",
                )
                adversarial_call(f"adv_{action_no:03d}_role_revoke_old_token", stale_user, email, "GET", "/api/presaves/senders", expected={403}, permission="read", lifecycle="revoke", polarity_tag="negative")
                adversarial_call(
                    f"adv_{action_no:03d}_role_restore",
                    system,
                    "system",
                    "PUT",
                    f"/api/users/{user_id}",
                    {"data": {"role": profile_id}},
                    expected={200},
                    permission="role",
                    lifecycle="restore",
                    polarity_tag="positive",
                )
                role_restore_expected = {200} if current_mode != "none" else {403}
                adversarial_call(
                    f"adv_{action_no:03d}_role_restore_old_token",
                    stale_user,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected=role_restore_expected,
                    permission="read",
                    lifecycle="restore",
                    polarity_tag="positive" if role_restore_expected == {200} else "negative",
                )
                adversarial_call(
                    f"adv_{action_no:03d}_role_restore_fresh_login",
                    fresh_user,
                    email,
                    "POST",
                    "/auth/v1/login",
                    {"email": email, "pwd": password, "organizationId": ORG},
                    expected={200},
                    permission="role",
                    lifecycle="restore",
                    polarity_tag="positive",
                )
                adversarial_call(
                    f"adv_{action_no:03d}_role_restore_fresh_token",
                    fresh_user,
                    email,
                    "GET",
                    "/api/presaves/senders",
                    expected=role_restore_expected,
                    permission="read",
                    lifecycle="restore",
                    polarity_tag="positive" if role_restore_expected == {200} else "negative",
                )
            elif kind == "profile_revoke_restore":
                current_mode = "none"
                set_adversarial_profile("none", f"adv_{action_no:03d}_profile_revoke")
                adversarial_call(f"adv_{action_no:03d}_profile_revoke_old_token", stale_user, email, "GET", "/api/presaves/senders", expected={403}, permission="read", lifecycle="revoke", polarity_tag="negative")
                current_mode = rng.choice(("read", "edit", "both"))
                set_adversarial_profile(current_mode, f"adv_{action_no:03d}_profile_restore")
                adversarial_call(f"adv_{action_no:03d}_profile_restore_old_token", stale_user, email, "GET", "/api/presaves/senders", expected={200}, permission="read", lifecycle="restore", polarity_tag="positive")
            elif kind == "presave_probe":
                target = rng.choice(tuple(spec for spec in entity_specs if spec[1]))
                probe_entity(target[0], target[1], target[2], target[3])
            elif kind == "route_probe":
                route_target = rng.choice(sender_ids)
                # Routing profile is authenticated self-service, independent
                # of menu read privilege. Only scope controls the write.
                adversarial_call(f"adv_{action_no:03d}_route_read", user, email, "GET", "/api/users/me/routing", expected={200}, permission="routing", lifecycle="route", polarity_tag="positive")
                route_scope_allows = current_scope == "empty" or current_scope.endswith("a") and route_target == sender_ids[0] or current_scope.endswith("b") and route_target == sender_ids[1]
                adversarial_call(f"adv_{action_no:03d}_route_write", user, email, "PUT", "/api/users/me/routing", {"data": {"active_sender_identifier": route_target}}, expected={200} if route_scope_allows else {403, 404}, permission="routing", scope=current_scope, lifecycle="route", polarity_tag="positive" if route_scope_allows else "negative")

            # Randomly vary profile and scope between probes.  Invalid scope is
            # kept as an explicit negative action and never becomes state.
            if not interrupted and rng.random() < 0.35:
                current_mode = rng.choice(modes)
                set_adversarial_profile(current_mode, f"adv_{action_no:03d}_random_profile_{current_mode}")
            if not interrupted and rng.random() < 0.35:
                next_scope = rng.choice(scope_names)
                scope_client, scope_actor = (
                    (cro, "cro") if next_scope.startswith("sender_") else (system, "system")
                )
                if next_scope.startswith("sender_"):
                    refresh_cro(f"adv_{action_no:03d}_cro_relogin_before_scope_{next_scope}")
                scope_status, _ = adversarial_call(
                    f"adv_{action_no:03d}_random_scope_{next_scope}",
                    scope_client,
                    scope_actor,
                    "PUT",
                    f"/api/users/{user_id}",
                    {"data": scope_specs[next_scope]},
                    expected={200},
                    permission="scope",
                    scope=next_scope,
                    lifecycle="update",
                    polarity_tag="positive",
                )
                if scope_status == 200:
                    current_scope = next_scope
    else:
        interrupted = interrupted or "presave_setup_failed"

    if company_user_id:
        call("company_user_role_revoke", system, "system", "PUT", f"/api/users/{company_user_id}", {"data": {"role": "user"}}, expected={200})
    if company_profile_id:
        call("company_profile_soft_delete", system, "system", "DELETE", f"/api/admin/permission-profiles/{company_profile_id}?organizationId={COMPANY_ORG}", expected={204})
    call("profile_soft_delete", system, "system", "DELETE", f"/api/admin/permission-profiles/{profile_id}?organizationId={ORG}", expected={204})
    call("same_session_profile_delete_revoke", user, email, "GET", "/api/presaves/senders", expected={403})
    return write(args, steps, interrupted, coverage, polarity)


def write(
    args: argparse.Namespace,
    steps: list[Step],
    interrupted: str | None,
    coverage: Counter[str],
    polarity: Counter[str],
) -> int:
    path = Path(args.artifact_dir)
    path.mkdir(parents=True, exist_ok=True)
    artifact = path / f"stateful-{args.seed}.jsonl"
    with artifact.open("w", encoding="utf-8") as output:
        for step in steps:
            output.write(json.dumps(asdict(step), sort_keys=True) + "\n")
        counts = {status: sum(step.status == status for step in steps) for status in ("PASS", "FAIL", "BLOCKED", "INCONCLUSIVE")}
        output.write(json.dumps({"kind": "run", "status": "INCONCLUSIVE" if interrupted else "COMPLETE", "reason": interrupted, "cases": len(steps), "counts": counts, "coverage": dict(sorted(coverage.items())), "polarity": dict(sorted(polarity.items()))}, sort_keys=True) + "\n")
    for step in steps:
        print(f"{step.status:12} {step.name:36} {step.response.get('http_status', '')}")
    print(f"cases={len(steps)} pass={sum(step.status == 'PASS' for step in steps)} fail={sum(step.status == 'FAIL' for step in steps)} blocked={sum(step.status == 'BLOCKED' for step in steps)} artifact={artifact}")
    return 2 if interrupted else (1 if any(step.status == "FAIL" for step in steps) else 0)


def parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
    parser.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "hdh4063@gmail.com"))
    parser.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
    parser.add_argument("--seed", type=int, default=int(time.time()))
    parser.add_argument("--max-actions", type=int, default=800)
    parser.add_argument("--matrix-rounds", type=int, default=16)
    parser.add_argument("--adversarial-actions", type=int, default=64)
    parser.add_argument("--deadline-seconds", type=float, default=180)
    parser.add_argument("--timeout", type=float, default=8)
    parser.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz/stateful")
    parser.add_argument("--allow-remote", action="store_true")
    return parser


if __name__ == "__main__":
    try:
        sys.exit(main(parser().parse_args()))
    except KeyboardInterrupt:
        sys.exit(2)
