#!/usr/bin/env python3
"""Seeded black-box XML import fuzzer using only the backend HTTP API."""

from __future__ import annotations

import argparse
import collections
import fnmatch
import hashlib
import http.cookiejar
import io
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
import uuid
import zipfile
from dataclasses import dataclass
from pathlib import Path
from typing import Any
from xml.etree import ElementTree as ET


PAGES = ("CI", "RP", "SD", "LR", "SI", "DM", "NR", "DH", "AE", "LB", "DG")
ET.register_namespace("", "urn:hl7-org:v3")
ET.register_namespace("xsi", "http://www.w3.org/2001/XMLSchema-instance")


@dataclass(frozen=True)
class Actor:
	label: str
	email: str
	password_env: str
	product_presave_id: str
	organization_id: str | None = None


@dataclass(frozen=True)
class Sample:
	name: str
	authority: str
	xml: bytes
	kind: str
	expected: str


class Client:
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
		body: bytes | dict[str, Any] | None = None,
		content_type: str | None = None,
	) -> tuple[int | None, bytes, str | None]:
		data = body if isinstance(body, bytes) else json.dumps(body).encode() if body is not None else None
		headers = {"Accept": "application/json"}
		if data is not None:
			headers["Content-Type"] = content_type or "application/json"
		req = urllib.request.Request(self.base_url + path, data=data, headers=headers, method=method)
		try:
			with self.opener.open(req, timeout=self.timeout) as response:
				return response.status, response.read(), None
		except urllib.error.HTTPError as error:
			return error.code, error.read(), None
		except (OSError, urllib.error.URLError) as error:
			return None, b"", type(error).__name__


def local_name(tag: str) -> str:
	return tag.rsplit("}", 1)[-1]


def direct_child(element: ET.Element, name: str) -> ET.Element | None:
	return next((child for child in element if local_name(child.tag) == name), None)


def unique_xml(xml: bytes, token: str) -> bytes:
	root = ET.fromstring(xml)
	report_id = f"ZZ-FUZZ-{token}"[:100]
	porr = next((node for node in root.iter() if local_name(node.tag) == "PORR_IN049016UV"), None)
	if porr is None:
		raise ValueError("PORR_IN049016UV missing")
	message_id = direct_child(porr, "id")
	if message_id is not None:
		message_id.set("extension", report_id)
	investigation = next((node for node in root.iter() if local_name(node.tag) == "investigationEvent"), None)
	if investigation is None:
		raise ValueError("investigationEvent missing")
	case_id = direct_child(investigation, "id")
	if case_id is None:
		raise ValueError("C.1.1 id missing")
	case_id.set("extension", report_id)
	return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def element_with_code(root: ET.Element, code: str) -> ET.Element | None:
	for node in root.iter():
		child = direct_child(node, "code")
		if child is not None and child.get("code") == code:
			return node
	return None


def mutate(xml: bytes, kind: str) -> bytes | None:
	if kind == "malformed_xml":
		return xml[: max(1, len(xml) // 2)]
	if kind == "invalid_utf8":
		return b"\xff\xfe" + xml
	root = ET.fromstring(xml)
	if kind == "wrapper_mismatch":
		porr = next(node for node in root.iter() if local_name(node.tag) == "PORR_IN049016UV")
		message_id = direct_child(porr, "id")
		if message_id is None:
			return None
		message_id.set("extension", "ZZ-FUZZ-MISMATCH")
	elif kind == "c17_ni":
		node = element_with_code(root, "23")
		value = direct_child(node, "value") if node is not None else None
		if value is None:
			return None
		value.attrib.pop("value", None)
		value.set("nullFlavor", "NI")
	elif kind == "value_and_nullflavor":
		node = element_with_code(root, "23")
		value = direct_child(node, "value") if node is not None else None
		if value is None:
			return None
		value.set("value", value.get("value", "true"))
		value.set("nullFlavor", "NI")
	elif kind == "invalid_date":
		control = next((node for node in root.iter() if local_name(node.tag) == "controlActProcess"), None)
		value = direct_child(control, "effectiveTime") if control is not None else None
		if value is None:
			return None
		value.set("value", "not-a-date")
	elif kind == "invalid_base64":
		text = next((node for node in root.iter() if local_name(node.tag) == "text" and node.get("representation") == "B64"), None)
		if text is None:
			return None
		for child in list(text):
			child.tail = "%%%not-base64%%%"
		if not list(text):
			text.text = "%%%not-base64%%%"
	else:
		raise ValueError(f"unknown mutation: {kind}")
	return ET.tostring(root, encoding="utf-8", xml_declaration=True)


def structural_signature(xml: bytes) -> dict[str, int]:
	root = ET.fromstring(xml)
	counts = collections.Counter()
	for node in root.iter():
		code = direct_child(node, "code")
		if code is None:
			continue
		if local_name(node.tag) == "document" and code.get("code") == "1":
			counts["documents"] += 1
		elif local_name(node.tag) == "observation" and code.get("code") == "29":
			counts["reactions"] += 1
		elif local_name(node.tag) == "organizer" and code.get("code") == "4":
			counts["drugs"] += 1
		elif local_name(node.tag) == "organizer" and code.get("code") == "3":
			counts["tests"] += 1
	return {key: counts[key] for key in ("documents", "tests", "drugs", "reactions")}


def identity_signature(xml: bytes) -> tuple[str | None, str | None]:
	root = ET.fromstring(xml)
	investigation = next((node for node in root.iter() if local_name(node.tag) == "investigationEvent"), None)
	control = next((node for node in root.iter() if local_name(node.tag) == "controlActProcess"), None)
	case_id = direct_child(investigation, "id") if investigation is not None else None
	creation = direct_child(control, "effectiveTime") if control is not None else None
	raw_date = creation.get("value") if creation is not None else None
	normalized_date = "".join(character for character in raw_date or "" if character.isdigit())[:14] or None
	return (
		case_id.get("extension") if case_id is not None else None,
		normalized_date,
	)


def multipart(authority: str, product_id: str, filename: str, payload: bytes) -> tuple[bytes, str]:
	boundary = f"XML-FUZZ-{uuid.uuid4().hex}"
	parts = [
		f'--{boundary}\r\nContent-Disposition: form-data; name="format"\r\n\r\n{authority}\r\n',
		f'--{boundary}\r\nContent-Disposition: form-data; name="productPresaveId"\r\n\r\n{product_id}\r\n',
		f'--{boundary}\r\nContent-Disposition: form-data; name="file"; filename="{filename}"\r\nContent-Type: application/zip\r\n\r\n',
	]
	body = "".join(parts).encode() + payload + f"\r\n--{boundary}--\r\n".encode()
	return body, f"multipart/form-data; boundary={boundary}"


def zip_samples(samples: list[Sample]) -> bytes:
	buffer = io.BytesIO()
	with zipfile.ZipFile(buffer, "w", zipfile.ZIP_DEFLATED) as archive:
		for sample in samples:
			archive.writestr(sample.name, sample.xml)
	return buffer.getvalue()


def json_value(body: bytes) -> Any:
	try:
		return json.loads(body)
	except (UnicodeDecodeError, json.JSONDecodeError):
		return None


def imported_rows(body: bytes) -> list[dict[str, Any]]:
	value = json_value(body)
	data = value.get("data", {}) if isinstance(value, dict) else {}
	rows = data.get("importedCases", data.get("imported_cases", [])) if isinstance(data, dict) else []
	return [row for row in rows if isinstance(row, dict)]


def classify_error(message: str | None, http_status: int | None) -> str:
	text = (message or "").lower()
	if http_status is None:
		return "transport"
	if http_status >= 500 or "panic" in text or "pgdatabaseerror" in text or "sqlx(" in text:
		return "server_or_raw_db"
	for label, needles in (
		("xml_syntax", ("xml parse", "not valid utf-8", "invalid xml")),
		("base64", ("base64", "includeddocument")),
		("input_contract", ("allowed.value", "length.max", "expected format")),
		("duplicate", ("same c.1.1", "import skipped")),
		("database", ("constraint", "database")),
	):
		if any(needle in text for needle in needles):
			return label
	return "other"


def load_config(path: Path) -> tuple[str, list[Actor], dict[str, list[Any]]]:
	config = json.loads(path.read_text())
	actors = [Actor(**item) for item in config["actors"]]
	corpora = {
		authority.lower(): values
		for authority, values in config["corpora"].items()
	}
	return config.get("base_url", "http://localhost:8080"), actors, corpora


def corpus_documents(sources: list[Any]) -> list[tuple[str, bytes, str]]:
	documents: list[tuple[str, bytes, str]] = []
	for source in sources:
		config = {"path": source} if isinstance(source, str) else source
		expected = config.get("expected", "success")
		location = config.get("path") or config.get("url")
		if not location:
			raise ValueError("corpus source requires path or url")
		if config.get("url"):
			with urllib.request.urlopen(location, timeout=120) as response:
				data = response.read()
			path = Path(urllib.parse.urlparse(location).path)
		else:
			path = Path(location)
			if path.is_dir():
				for xml_path in sorted(path.glob("*.xml")):
					documents.append((xml_path.stem, xml_path.read_bytes(), expected))
				continue
			data = path.read_bytes()
		if path.suffix.lower() != ".zip":
			documents.append((path.stem, data, expected))
			continue
		patterns = config.get("members", ["*.xml"])
		with zipfile.ZipFile(io.BytesIO(data)) as archive:
			for member in sorted(archive.namelist()):
				if not member.endswith("/") and any(fnmatch.fnmatch(member, pattern) for pattern in patterns):
					documents.append((Path(member).stem, archive.read(member), expected))
	return documents


def login(client: Client, actor: Actor) -> tuple[int | None, str | None]:
	password = os.environ.get(actor.password_env)
	if password is None:
		return None, f"missing environment variable {actor.password_env}"
	payload: dict[str, Any] = {"email": actor.email, "pwd": password}
	if actor.organization_id:
		payload["organizationId"] = actor.organization_id
	status, body, transport = client.request("POST", "/auth/v1/login", payload)
	return status, transport or (None if status == 200 else body.decode(errors="replace")[:300])


def verify_roundtrip(client: Client, case_id: str, authority: str, source: bytes) -> list[str]:
	failures: list[str] = []
	status, _, transport = client.request("GET", f"/api/cases/{case_id}")
	if status != 200:
		failures.append(f"case_get:{status or transport}")
	for page in PAGES:
		status, _, transport = client.request(
			"GET", f"/api/cases/{case_id}/editor/pages/{page}?authorities={authority}"
		)
		if status != 200:
			failures.append(f"page_{page}:{status or transport}")
	status, exported, transport = client.request("GET", f"/api/cases/{case_id}/export/xml?authority={authority}")
	if status != 200:
		value = json_value(exported)
		detail = value.get("error", {}).get("data", {}).get("detail") if isinstance(value, dict) else None
		failures.append(f"export:{status or transport}:{str(detail)[:500] if detail else ''}")
	else:
		try:
			expected_identity = identity_signature(source)
			actual_identity = identity_signature(exported)
			if actual_identity != expected_identity:
				failures.append(f"roundtrip_identity:{expected_identity}->{actual_identity}")
			expected = structural_signature(source)
			actual = structural_signature(exported)
			for key, count in expected.items():
				if count and actual[key] < count:
					failures.append(f"roundtrip_{key}:{count}->{actual[key]}")
		except ET.ParseError as error:
			failures.append(f"export_xml:{error}")
	return failures


def batches(items: list[Sample], size: int) -> list[list[Sample]]:
	return [items[index:index + size] for index in range(0, len(items), size)]


def run(args: argparse.Namespace) -> int:
	base_url, actors, corpora = load_config(args.config)
	if not actors:
		raise ValueError("at least one actor is required")
	clients = {actor.label: Client(base_url, args.timeout) for actor in actors}
	results: list[dict[str, Any]] = []
	for actor in actors:
		status, error = login(clients[actor.label], actor)
		if status != 200:
			raise RuntimeError(f"login failed for {actor.label}: {status} {error}")

	primary = actors[0]
	all_samples: dict[str, list[Sample]] = collections.defaultdict(list)
	for authority, sources in corpora.items():
		for source_name, raw, source_expected in corpus_documents(sources):
			for copy in range(args.copies):
				token = hashlib.sha256(f"{args.seed}:{authority}:{source_name}:{copy}".encode()).hexdigest()[:20].upper()
				try:
					healthy = unique_xml(raw, token)
				except (ET.ParseError, ValueError) as error:
					results.append({"actor": primary.label, "authority": authority, "file": source_name, "kind": "corpus_setup", "status": "error", "message": str(error)})
					continue
				base = Sample(f"healthy-{source_name}-{copy}.xml", authority, healthy, "healthy", source_expected)
				all_samples[authority].append(base)
				if copy < args.mutations_per_seed:
					for kind, mutation_expected in (
						("malformed_xml", "error"), ("invalid_utf8", "error"),
						("invalid_base64", "error"), ("value_and_nullflavor", "error"),
						("invalid_date", "error"), ("wrapper_mismatch", "success"),
						("c17_ni", "success"),
					):
						mutation_token = hashlib.sha256(
							f"{args.seed}:{authority}:{source_name}:{copy}:{kind}".encode()
						).hexdigest()[:20].upper()
						changed = mutate(unique_xml(raw, mutation_token), kind)
						if changed is not None:
							all_samples[authority].append(Sample(f"mut-{kind}-{source_name}-{copy}.xml", authority, changed, kind, mutation_expected))

	for authority, samples in all_samples.items():
		by_name = {sample.name: sample for sample in samples}
		for batch in batches(samples, args.batch_size):
			payload, content_type = multipart(authority, primary.product_presave_id, f"xml-fuzz-{authority}.zip", zip_samples(batch))
			status, body, transport = clients[primary.label].request("POST", "/api/import/xml", payload, content_type)
			rows = imported_rows(body)
			if not rows:
				for sample in batch:
					results.append({"actor": primary.label, "authority": authority, "file": sample.name, "kind": sample.kind, "expected": sample.expected, "status": "request_error", "http": status, "category": classify_error(transport or body.decode(errors="replace")[:500], status)})
				continue
			for row in rows:
				name = row.get("sourceFileName") or row.get("source_file_name")
				sample = by_name.get(name)
				if sample is None:
					continue
				row_status = row.get("status")
				message = row.get("message")
				case_id = row.get("caseId") or row.get("case_id")
				roundtrip = verify_roundtrip(clients[primary.label], case_id, authority, sample.xml) if row_status in {"success", "warning"} and case_id else []
				results.append({"actor": primary.label, "authority": authority, "file": name, "kind": sample.kind, "expected": sample.expected, "status": row_status, "case_id": case_id, "category": classify_error(message, status), "message": message, "roundtrip_failures": roundtrip})

	# Same C.1.1/version must be accepted independently by every configured organization.
	if len(actors) > 1:
		probe_authority = next((key for key, value in all_samples.items() if value), None)
		if probe_authority:
			base_probe = next(sample for sample in all_samples[probe_authority] if sample.kind == "healthy")
			probe = Sample(
				"multi-org-probe.xml",
				probe_authority,
				unique_xml(base_probe.xml, hashlib.sha256(f"{args.seed}:multi-org".encode()).hexdigest()[:20].upper()),
				"multi_org_same_identifier",
				"success",
			)
			for actor in actors:
				payload, content_type = multipart(probe_authority, actor.product_presave_id, f"multi-org-{probe.name}.zip", zip_samples([probe]))
				status, body, transport = clients[actor.label].request("POST", "/api/import/xml", payload, content_type)
				row = (imported_rows(body) or [{}])[0]
				row_status = row.get("status", "request_error")
				case_id = row.get("caseId") or row.get("case_id")
				roundtrip = verify_roundtrip(clients[actor.label], case_id, probe_authority, probe.xml) if row_status in {"success", "warning"} and case_id else []
				results.append({"actor": actor.label, "authority": probe_authority, "file": probe.name, "kind": "multi_org_same_identifier", "expected": "success", "status": row_status, "case_id": case_id, "http": status, "message": row.get("message") or transport, "roundtrip_failures": roundtrip})

	args.output.parent.mkdir(parents=True, exist_ok=True)
	with args.output.open("w") as handle:
		for result in results:
			handle.write(json.dumps(result, ensure_ascii=False) + "\n")
	summary = collections.Counter((item.get("kind"), item.get("status")) for item in results)
	errors = [item for item in results if item.get("category") == "server_or_raw_db" or item.get("roundtrip_failures") or (item.get("expected") == "success" and item.get("status") not in {"success", "warning"}) or (item.get("expected") == "error" and item.get("status") != "error")]
	print(json.dumps({"total": len(results), "failures": len(errors), "by_kind_status": {f"{kind}:{status}": count for (kind, status), count in sorted(summary.items())}, "output": str(args.output)}, ensure_ascii=False, indent=2))
	return 1 if errors else 0


def main() -> int:
	parser = argparse.ArgumentParser()
	parser.add_argument("--config", type=Path, required=True)
	parser.add_argument("--copies", type=int, default=10)
	parser.add_argument("--mutations-per-seed", type=int, default=1)
	parser.add_argument("--batch-size", type=int, default=10)
	parser.add_argument("--seed", type=int, default=20260812)
	parser.add_argument("--timeout", type=float, default=300)
	parser.add_argument("--output", type=Path, default=Path("tmp/xml-import-fuzz/results.jsonl"))
	args = parser.parse_args()
	if args.copies < 1 or args.batch_size < 1 or args.mutations_per_seed < 0:
		parser.error("copies/batch-size must be positive and mutations-per-seed non-negative")
	return run(args)


if __name__ == "__main__":
	sys.exit(main())
