#!/usr/bin/env python3
"""Minimal public-API RBAC/RLS smoke runner.

It deliberately records only sanitized response summaries. Mutating scenarios
belong in a later phase after the target and credentials are explicitly set.
"""

from __future__ import annotations

import argparse
import functools
import hashlib
import http.cookiejar
import json
import os
import re
import subprocess
import sys
import time
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass, asdict
from pathlib import Path
from typing import Any


SAFE_LOCAL_HOSTS = {"localhost", "127.0.0.1", "::1"}
UUID_RE = re.compile(
	r"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}"
)


def fingerprint(value: Any) -> str:
	return hashlib.sha256(str(value).encode()).hexdigest()[:12]


def sanitize_path(path: str) -> str:
	return UUID_RE.sub(lambda match: f"<uuid:{fingerprint(match.group(0))}>", path)


def response_summary(status: int | None, body: bytes) -> dict[str, Any]:
	result: dict[str, Any] = {"status": status, "bytes": len(body)}
	try:
		value = json.loads(body)
	except (UnicodeDecodeError, json.JSONDecodeError):
		return result

	if not isinstance(value, dict):
		result["json_type"] = type(value).__name__
		return result

	if isinstance(value.get("error"), dict):
		error = value["error"]
		message = error.get("message")
		if isinstance(message, str):
			# Error text can contain emails, UUIDs, SQL, or secrets; retain only
			# a stable fingerprint for triage.
			result["message_fingerprint"] = fingerprint(message)
			result["message_length"] = len(message)
		data = error.get("data")
		if isinstance(data, dict):
			result["detail_type"] = type(data.get("detail")).__name__
		return result

	data = value.get("data")
	if isinstance(data, list):
		result["items"] = len(data)
	elif isinstance(data, dict):
		result["fields"] = sorted(data.keys())[:20]
	return result


@dataclass
class Action:
	path: str
	method: str
	status: str
	response: dict[str, Any]
	actor: str
	seed: int
	commit_sha: str


class ApiClient:
	def __init__(self, base_url: str, timeout: float) -> None:
		self.base_url = base_url.rstrip("/")
		self.timeout = timeout
		self.jar = http.cookiejar.CookieJar()
		self.opener = urllib.request.build_opener(
			urllib.request.HTTPCookieProcessor(self.jar)
		)

	def request(
		self,
		method: str,
		path: str,
		payload: dict[str, Any] | bytes | None = None,
		content_type: str | None = None,
	) -> tuple[int | None, bytes, str | None]:
		body = (
			payload
			if isinstance(payload, bytes)
			else json.dumps(payload).encode() if payload is not None else None
		)
		headers = {"Accept": "application/json"}
		if payload is not None:
			headers["Content-Type"] = content_type or "application/json"
		request = urllib.request.Request(
			f"{self.base_url}{path}",
			data=body,
			headers=headers,
			method=method,
		)
		try:
			with self.opener.open(request, timeout=self.timeout) as response:
				return response.status, response.read(), None
		except urllib.error.HTTPError as error:
			return error.code, error.read(), None
		except (OSError, urllib.error.URLError) as error:
			return None, b"", type(error).__name__


def classify(status: int | None, expected: set[int]) -> str:
	if status is None:
		return "INCONCLUSIVE"
	if status in expected:
		return "PASS"
	if status in {401, 403, 404}:
		return "BLOCKED"
	return "FAIL"


@functools.cache
def commit_sha() -> str:
	try:
		return subprocess.check_output(
			["git", "rev-parse", "HEAD"], text=True, stderr=subprocess.DEVNULL
		).strip()
	except (OSError, subprocess.CalledProcessError):
		return "unknown"


def guard_target(base_url: str, allow_remote: bool) -> None:
	parsed = urllib.parse.urlparse(base_url)
	if parsed.scheme not in {"http", "https"} or not parsed.hostname:
		raise SystemExit("base URL must be an http(s) URL")
	if parsed.hostname not in SAFE_LOCAL_HOSTS and not allow_remote:
		raise SystemExit(
		"refusing non-local target; pass --allow-remote only for an authorized dev/staging service"
	)


def stop_reason(args: argparse.Namespace, started: float, count: int) -> str | None:
	if count >= args.max_actions:
		return "max_actions"
	if (
		args.deadline_seconds is not None
		and time.monotonic() - started >= args.deadline_seconds
	):
		return "deadline"
	return None


def run_smoke(args: argparse.Namespace) -> int:
	guard_target(args.base_url, args.allow_remote)
	if not args.email or not args.password:
		raise SystemExit("set E2BR3_ADMIN_EMAIL and E2BR3_ADMIN_PASSWORD")

	actor = fingerprint(args.email)
	sha = commit_sha()
	client = ApiClient(args.base_url, args.timeout)
	actions: list[Action] = []
	run_started = time.monotonic()
	interrupted: str | None = None

	def run(
		method: str,
		path: str,
		payload: dict[str, Any] | None = None,
		expected: set[int] = {200},
	) -> None:
		nonlocal interrupted
		interrupted = interrupted or stop_reason(args, run_started, len(actions))
		if interrupted:
			return
		request_started = time.monotonic()
		status, body, transport_error = client.request(method, path, payload)
		response = response_summary(status, body)
		if transport_error:
			response["transport_error"] = transport_error
			interrupted = interrupted or "transport_error"
		elif status == 429:
			interrupted = interrupted or "rate_limited"
		elif status is not None and status >= 500:
			interrupted = interrupted or "server_error"
		response["duration_ms"] = round((time.monotonic() - request_started) * 1000, 2)
		actions.append(
			Action(
				path=sanitize_path(path),
				method=method,
				status=classify(status, expected),
				response=response,
				actor=actor,
				seed=args.seed,
				commit_sha=sha,
			)
		)
	run("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
	if not actions or actions[-1].status != "PASS":
		return write_results(args, actions, interrupted=interrupted or "login_failed")

	for path in (
		"/api/users/me",
		"/api/users/me/profile",
		"/api/users",
		"/api/organizations",
		"/api/cases?list_options.limit=20",
		"/api/submissions/history?list_options.limit=20",
	):
		run("GET", path, expected={200, 403})

	return write_results(args, actions, interrupted=interrupted)


def run_matrix(args: argparse.Namespace) -> int:
	guard_target(args.base_url, args.allow_remote)
	actors = args.actor or ([args.email] if args.email else [])
	if not actors or not args.password:
		raise SystemExit("set E2BR3_ADMIN_EMAIL/E2BR3_ACTORS and E2BR3_ADMIN_PASSWORD")

	sha = commit_sha()
	actions: list[Action] = []
	run_started = time.monotonic()
	interrupted: str | None = None
	sessions: list[
		tuple[str, ApiClient, str, str | None, list[dict[str, Any]], list[dict[str, Any]]]
	] = []

	def call(
		actor_email: str,
		client: ApiClient,
		method: str,
		path: str,
		payload: dict[str, Any] | None = None,
		expected: set[int] = {200},
	) -> tuple[int | None, bytes]:
		nonlocal interrupted
		interrupted = interrupted or stop_reason(args, run_started, len(actions))
		if interrupted:
			return None, b""
		request_started = time.monotonic()
		status, body, transport_error = client.request(method, path, payload)
		response = response_summary(status, body)
		if transport_error:
			response["transport_error"] = transport_error
			interrupted = interrupted or "transport_error"
		elif status == 429:
			interrupted = interrupted or "rate_limited"
		elif status is not None and status >= 500:
			interrupted = interrupted or "server_error"
		response["duration_ms"] = round((time.monotonic() - request_started) * 1000, 2)
		actions.append(
			Action(
				path=sanitize_path(path),
				method=method,
				status=classify(status, expected),
				response=response,
				actor=fingerprint(actor_email),
				seed=args.seed,
				commit_sha=sha,
			)
		)
		return status, body

	for email in actors:
		client = ApiClient(args.base_url, args.timeout)
		status, _ = call(email, client, "POST", "/auth/v1/login", {"email": email, "pwd": args.password})
		if status != 200:
			interrupted = interrupted or "login_failed"
			break
		status, body = call(email, client, "GET", "/api/users/me")
		try:
			me = json.loads(body).get("data", {})
		except (UnicodeDecodeError, json.JSONDecodeError, AttributeError):
			me = {}
		organization_id = me.get("organizationId")
		role = me.get("role")
		status, body = call(email, client, "GET", "/api/users", expected={200, 403})
		try:
			users = json.loads(body).get("data", []) if status == 200 else []
		except (UnicodeDecodeError, json.JSONDecodeError, AttributeError):
			users = []
		status, body = call(email, client, "GET", "/api/cases?list_options.limit=20", expected={200, 403})
		try:
			cases = json.loads(body).get("data", []) if status == 200 else []
		except (UnicodeDecodeError, json.JSONDecodeError, AttributeError):
			cases = []
		sessions.append((email, client, organization_id, role, users, cases))

	for email, client, organization_id, role, _, _ in sessions:
		if role == "system_admin":
			continue
		for other_email, _, _, _, users, cases in sessions:
			if other_email == email:
				continue
			for user in users:
				user_id = user.get("id") if isinstance(user, dict) else None
				user_org = user.get("organizationId") if isinstance(user, dict) else None
				if user_id and user_org != organization_id:
					call(email, client, "GET", f"/api/users/{user_id}", expected={403, 404})
					role_value = user.get("role") if isinstance(user, dict) else None
					if role_value is not None:
						call(
							email,
							client,
							"PUT",
							f"/api/users/{user_id}",
							{"data": {"role": role_value}},
							expected={403, 404},
						)
			for case in (cases if isinstance(cases, list) else []):
				case_id = case.get("id") if isinstance(case, dict) else None
				case_org = case.get("organizationId") if isinstance(case, dict) else None
				if case_id and case_org != organization_id:
					call(email, client, "GET", f"/api/cases/{case_id}", expected={403, 404})
					status_value = case.get("status") if isinstance(case, dict) else None
					if status_value is not None:
						call(
							email,
							client,
							"PUT",
							f"/api/cases/{case_id}",
							{"data": {"status": status_value}},
							expected={403, 404},
						)

	return write_results(args, actions, "matrix", interrupted=interrupted)


def write_results(
	args: argparse.Namespace,
	actions: list[Action],
	name: str = "smoke",
	interrupted: str | None = None,
) -> int:
	path = Path(args.artifact_dir)
	path.mkdir(parents=True, exist_ok=True)
	artifact = path / f"{name}-{args.seed}.jsonl"
	with artifact.open("w", encoding="utf-8") as output:
		for action in actions:
			output.write(json.dumps(asdict(action), sort_keys=True) + "\n")
		if interrupted:
			output.write(
				json.dumps(
					{"kind": "run", "status": "INCONCLUSIVE", "reason": interrupted},
					sort_keys=True,
				)
				+ "\n"
			)

	for action in actions:
		print(
			f"{action.status:12} {action.method:4} {action.path:45} "
			f"{action.response.get('status')}"
		)
	print(f"artifact={artifact}")
	return 2 if interrupted else (1 if any(action.status == "FAIL" for action in actions) else 0)


def parser() -> argparse.ArgumentParser:
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8080"))
	parser.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL"))
	parser.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD"))
	parser.add_argument(
		"--actor",
		action="append",
		default=[email for email in os.getenv("E2BR3_ACTORS", "").split(",") if email],
	)
	parser.add_argument("--seed", type=int, default=1)
	parser.add_argument("--max-actions", type=int, default=8)
	parser.add_argument("--timeout", type=float, default=5.0)
	parser.add_argument("--artifact-dir", default="tmp/rbac-rls-fuzz")
	parser.add_argument("--deadline-seconds", type=float)
	parser.add_argument("--allow-remote", action="store_true")
	parser.add_argument("command", choices=("smoke", "matrix"))
	return parser


if __name__ == "__main__":
	try:
		args = parser().parse_args()
		sys.exit(run_smoke(args) if args.command == "smoke" else run_matrix(args))
	except KeyboardInterrupt:
		print("INCONCLUSIVE interrupted", file=sys.stderr)
		sys.exit(2)
