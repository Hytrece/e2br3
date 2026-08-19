#!/usr/bin/env python3
"""Create a rich Case, export it per authority, import it, and re-export it."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
import time
import urllib.error
import uuid
from pathlib import Path
from typing import Any

from case_editor_input_fuzzer import object_id
from rbac_rls_blackbox import ApiClient, guard_target, response_summary
from xml_import_fuzzer import (
	identity_signature,
	multipart,
	preservation_signature,
	structural_signature,
	unique_xml,
)


ROOT = Path(__file__).resolve().parents[1]
PAGES = ("CI", "RP", "SD", "LR", "SI", "DM", "DH", "NR", "AE", "LB", "DG")


def json_value(body: bytes) -> Any:
	try:
		return json.loads(body)
	except (UnicodeDecodeError, json.JSONDecodeError):
		return None


def imported_case_id(body: bytes) -> str | None:
	value = json_value(body)
	data = value.get("data", {}) if isinstance(value, dict) else {}
	rows = data.get("importedCases", data.get("imported_cases", [])) if isinstance(data, dict) else []
	if isinstance(rows, list):
		for row in rows:
			if isinstance(row, dict):
				case_id = row.get("caseId") or row.get("case_id")
				if isinstance(case_id, str):
					return case_id
	case_id = data.get("caseId") or data.get("case_id") if isinstance(data, dict) else None
	return case_id if isinstance(case_id, str) else None


def fingerprint(value: str) -> str:
	return hashlib.sha256(value.encode()).hexdigest()[:12]


def summary(status: int | None, body: bytes, transport: str | None = None) -> dict[str, Any]:
	value = response_summary(status, body)
	if transport:
		value["transport_error"] = transport
	return value


def setup_rich_case(args: argparse.Namespace, seed: int, output_dir: Path) -> tuple[str | None, dict[str, Any]]:
	setup_dir = output_dir / f"round-{seed}" / "setup"
	setup_dir.mkdir(parents=True, exist_ok=True)
	command = [
		sys.executable,
		str(ROOT / "scripts/case_editor_input_fuzzer.py"),
		"--base-url", args.base_url,
		"--email", args.email,
		"--password", args.password,
		"--seed", str(seed),
		"--pages", ",".join(PAGES),
		"--complete-baseline",
		"--values-per-field", "0",
		"--samples-per-category", "1",
		"--artifact-dir", str(setup_dir),
		"--no-run-gates",
	]
	try:
		process = subprocess.run(
			command,
			cwd=ROOT,
			capture_output=True,
			text=True,
			timeout=args.setup_timeout,
			check=False,
		)
	except (OSError, subprocess.TimeoutExpired) as error:
		return None, {"status": "FAIL", "reason": type(error).__name__}
	(setup_dir / "stdout.log").write_text(process.stdout, encoding="utf-8")
	(setup_dir / "stderr.log").write_text(process.stderr, encoding="utf-8")
	artifact = setup_dir / f"case-editor-{seed}.jsonl"
	case_id = None
	if artifact.exists():
		try:
			case_id = json.loads(artifact.read_text(encoding="utf-8").splitlines()[-1]).get("case_id")
		except (IndexError, json.JSONDecodeError):
			pass
	return case_id, {
		"status": "PASS" if process.returncode == 0 and case_id else "FAIL",
		"exit_code": process.returncode,
		"case_id": case_id,
		"stdout_fingerprint": fingerprint(process.stdout),
		"stderr_fingerprint": fingerprint(process.stderr),
		"artifact": str(artifact),
	}


def create_import_product(client: ApiClient, token: str) -> tuple[str | None, list[dict[str, Any]]]:
	results: list[dict[str, Any]] = []
	sender_payload = {
		"data": {
			"rows": {
				"sender": {"senderType": "1", "organizationName": f"RICH-IMPORT-{token}"},
				"gateways": [],
				"responsiblePersons": [],
			}
		}
	}
	status, body, transport = client.request("POST", "/api/presaves/senders", sender_payload)
	results.append({"kind": "support_sender", "status": status, "response": summary(status, body, transport)})
	sender_id = object_id(json_value(body))
	if status != 201 or not sender_id:
		return None, results
	product_payload = {
		"data": {
			"rows": {
				"product": {
					"senderPresaveId": sender_id,
					"productId": f"RICH-IMPORT-PRODUCT-{token}",
					"medicinalProduct": f"Rich Import Product {token}",
				},
				"activeSubstances": [],
			}
		}
	}
	status, body, transport = client.request("POST", "/api/presaves/products", product_payload)
	results.append({"kind": "support_product", "status": status, "response": summary(status, body, transport)})
	return (object_id(json_value(body)) if status == 201 else None), results


def validation(client: ApiClient, case_id: str, authority: str) -> dict[str, Any]:
	status, body, transport = client.request("GET", f"/api/cases/{case_id}/validation?authority={authority}")
	value = json_value(body)
	data = value.get("data", {}) if isinstance(value, dict) else {}
	return {"status": status, "ok": data.get("ok") if isinstance(data, dict) else None, "response": summary(status, body, transport)}


def mark_validated(client: ApiClient, case_id: str) -> dict[str, Any]:
	status, body, transport = client.request("POST", f"/api/cases/{case_id}/validator/mark-validated")
	return {"status": status, "response": summary(status, body, transport)}


def export_xml(client: ApiClient, case_id: str, authority: str) -> tuple[int | None, bytes, dict[str, Any]]:
	status, body, transport = client.request("GET", f"/api/cases/{case_id}/export/xml?authority={authority}")
	return status, body, summary(status, body, transport)


def compare_xml(source: bytes, exported: bytes) -> list[str]:
	failures: list[str] = []
	try:
		if identity_signature(source) != identity_signature(exported):
			failures.append("identity")
		before = structural_signature(source)
		after = structural_signature(exported)
		for key, count in before.items():
			if after[key] < count:
				failures.append(f"structure:{key}:{count}->{after[key]}")
		before = preservation_signature(source)
		after = preservation_signature(exported)
		for key, values in before.items():
			if values - after[key]:
				failures.append(f"preservation:{key}")
	except Exception as error:  # noqa: BLE001 - report malformed export as a test failure
		failures.append(f"compare:{type(error).__name__}")
	return failures


def delete_case(client: ApiClient, case_id: str) -> dict[str, Any]:
	status, body, transport = client.request("DELETE", f"/api/cases/{case_id}", {"reason_for_change": "rich export/import fuzzer cleanup"})
	return {"status": status, "case_id": case_id, "response": summary(status, body, transport)}


def run(args: argparse.Namespace) -> int:
	guard_target(args.base_url, args.allow_remote)
	authorities = [item.strip().lower() for item in args.authorities.split(",") if item.strip()]
	unknown = set(authorities) - {"ich", "fda", "mfds"}
	if unknown or not authorities:
		raise SystemExit(f"invalid authorities: {', '.join(sorted(unknown)) or 'none'}")
	output_dir = Path(args.artifact_dir)
	output_dir.mkdir(parents=True, exist_ok=True)
	client = ApiClient(args.base_url, args.timeout)
	results: list[dict[str, Any]] = []
	case_ids: list[str] = []
	status, body, transport = client.request("POST", "/auth/v1/login", {"email": args.email, "pwd": args.password})
	results.append({"kind": "login", "status": status, "response": summary(status, body, transport)})
	if status != 200:
		return write_results(args, results)
	product_id, product_results = create_import_product(client, f"{args.seed}-{uuid.uuid4().hex[:8]}")
	results.extend(product_results)
	if not product_id:
		return write_results(args, results)
	for round_no in range(args.rounds):
		seed = args.seed + round_no
		source_id, setup = setup_rich_case(args, seed, output_dir)
		results.append({"kind": "rich_case_setup", "round": round_no, **setup})
		if not source_id:
			continue
		case_ids.append(source_id)
		for authority in authorities:
			check = validation(client, source_id, authority)
			results.append({"kind": "source_validation", "round": round_no, "authority": authority, "case_id": source_id, **check})
			if check["status"] == 200 and check["ok"] is True:
				results.append({"kind": "source_mark_validated", "round": round_no, "authority": authority, "case_id": source_id, **mark_validated(client, source_id)})
			status, source_xml, export_summary = export_xml(client, source_id, authority)
			results.append({"kind": "source_export", "round": round_no, "authority": authority, "case_id": source_id, "status": status, "response": export_summary})
			if status != 200:
				continue
			authority_dir = output_dir / f"round-{seed}" / authority
			authority_dir.mkdir(parents=True, exist_ok=True)
			(source_path := authority_dir / "source.xml").write_bytes(source_xml)
			try:
				import_xml = unique_xml(source_xml, f"{seed}-{authority}-{uuid.uuid4().hex[:10]}")
			except (ValueError, UnicodeError) as error:
				results.append({"kind": "import_prepare", "round": round_no, "authority": authority, "status": "FAIL", "reason": type(error).__name__})
				continue
			(import_path := authority_dir / "import.xml").write_bytes(import_xml)
			payload, content_type = multipart(authority, product_id, f"rich-{authority}-{seed}.xml", import_xml)
			status, body, transport = client.request("POST", "/api/import/xml", payload, content_type)
			import_id = imported_case_id(body)
			if import_id:
				case_ids.append(import_id)
			results.append({"kind": "import", "round": round_no, "authority": authority, "status": status, "case_id": import_id, "response": summary(status, body, transport)})
			if status != 200 or not import_id:
				continue
			import_check = validation(client, import_id, authority)
			results.append({"kind": "import_validation", "round": round_no, "authority": authority, "case_id": import_id, **import_check})
			if import_check["status"] == 200 and import_check["ok"] is True:
				results.append({"kind": "import_mark_validated", "round": round_no, "authority": authority, "case_id": import_id, **mark_validated(client, import_id)})
			status, exported, export_summary = export_xml(client, import_id, authority)
			failures = compare_xml(import_xml, exported) if status == 200 else [f"reexport:{status}"]
			results.append({"kind": "reexport_compare", "round": round_no, "authority": authority, "case_id": import_id, "status": "PASS" if status == 200 and not failures else "FAIL", "http_status": status, "failures": failures, "response": export_summary})
			if status == 200:
				(authority_dir / "reexport.xml").write_bytes(exported)
	if not args.keep_cases:
		for case_id in sorted(set(case_ids)):
			results.append({"kind": "cleanup", **delete_case(client, case_id)})
	return write_results(args, results)


def write_results(args: argparse.Namespace, results: list[dict[str, Any]]) -> int:
	path = Path(args.artifact_dir) / f"rich-case-export-import-{args.seed}.jsonl"
	path.parent.mkdir(parents=True, exist_ok=True)
	with path.open("w", encoding="utf-8") as handle:
		for result in results:
			handle.write(json.dumps(result, ensure_ascii=True, sort_keys=True) + "\n")
	failed = [item for item in results if item.get("status") in {"FAIL", "error"} or item.get("kind") == "reexport_compare" and item.get("status") != "PASS"]
	counts = {kind: sum(1 for item in results if item.get("kind") == kind) for kind in sorted({item.get("kind") for item in results})}
	print(json.dumps({"results": len(results), "failed": len(failed), "counts": counts, "artifact": str(path)}, ensure_ascii=True))
	return 1 if failed else 0


def parser() -> argparse.ArgumentParser:
	parser = argparse.ArgumentParser()
	parser.add_argument("--base-url", default=os.getenv("E2BR3_BASE_URL", "http://127.0.0.1:8216"))
	parser.add_argument("--email", default=os.getenv("E2BR3_ADMIN_EMAIL", "demo.cro.admin@example.com"))
	parser.add_argument("--password", default=os.getenv("E2BR3_ADMIN_PASSWORD", "welcome"))
	parser.add_argument("--authorities", default="ich,fda,mfds")
	parser.add_argument("--rounds", type=int, default=1)
	parser.add_argument("--seed", type=int, default=20260819)
	parser.add_argument("--timeout", type=float, default=30)
	parser.add_argument("--setup-timeout", type=float, default=600)
	parser.add_argument("--artifact-dir", default="tmp/rich-case-export-import")
	parser.add_argument("--keep-cases", action="store_true")
	parser.add_argument("--allow-remote", action="store_true")
	return parser


if __name__ == "__main__":
	try:
		sys.exit(run(parser().parse_args()))
	except KeyboardInterrupt:
		sys.exit(2)
