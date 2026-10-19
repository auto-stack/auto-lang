#!/usr/bin/env python3
"""Verify every TAPL execution descriptor through the local Playground API."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


def find_book_root(explicit: str | None) -> Path:
    candidates = [explicit, os.environ.get("AUTO_BOOK_ROOT")]
    candidates.extend(
        [
            str(REPO_ROOT.parent / "book"),
            str(REPO_ROOT.parent.parent.parent / "book"),
        ]
    )
    for candidate in candidates:
        if candidate and (Path(candidate) / "tapl" / "examples.json").is_file():
            return Path(candidate).resolve()
    raise SystemExit("TAPL author repository not found; pass --book-root or set AUTO_BOOK_ROOT")


def display_code(tapl_root: Path, entry: dict) -> str:
    source = tapl_root.parent / entry["display"]["source"]
    if entry["display"]["kind"] != "fence":
        return source.read_text(encoding="utf-8").rstrip()
    lines = source.read_text(encoding="utf-8-sig").splitlines()
    index = int(entry["line"]) - 1
    if index < 0 or index >= len(lines):
        raise ValueError(f"{entry['id']}: line is outside {source}")
    fence = chr(96) * 3
    if not re.match(r"^\s*" + re.escape(fence) + r"auto(?:\s|$)", lines[index]):
        raise ValueError(f"{entry['id']}: expected auto fence at {source}:{entry['line']}")
    code = []
    for line in lines[index + 1 :]:
        if re.match(r"^\s*" + re.escape(fence) + r"\s*$", line):
            return "\n".join(code).rstrip()
        code.append(line)
    raise ValueError(f"{entry['id']}: unterminated auto fence in {source}")


def display_source_bytes(tapl_root: Path, entry: dict) -> bytes:
    source = tapl_root.parent / entry["display"]["source"]
    raw = source.read_bytes()
    if entry["display"]["kind"] != "fence":
        return raw
    with source.open("r", encoding="utf-8-sig", newline="") as stream:
        lines = stream.readlines()
    start = int(entry["line"])
    fence = chr(96) * 3
    code = []
    for line in lines[start:]:
        if re.match(r"^\s*" + re.escape(fence) + r"\s*$", line.strip()):
            return "".join(code).encode("utf-8")
        code.append(line)
    raise ValueError(f"{entry['id']}: unterminated auto fence in {source}")


def audit_bindings(tapl_root: Path, entries: list[dict]) -> list[str]:
    errors: list[str] = []
    by_id = {entry["id"]: entry for entry in entries}
    if len(by_id) != len(entries):
        errors.append("catalog contains duplicate IDs")
    bindings: dict[str, int] = {}
    for page in sorted(tapl_root.glob("*.md")):
        text = page.read_text(encoding="utf-8-sig")
        for found in re.findall(r'example-id=["\']([^"\']+)["\']', text):
            bindings[found] = bindings.get(found, 0) + 1
    for example_id in sorted(by_id):
        count = bindings.get(example_id, 0)
        if count != 1:
            errors.append(f"{example_id}: expected one page binding, found {count}")
    for example_id in sorted(set(bindings) - set(by_id)):
        errors.append(f"orphan page binding: {example_id}")
    for entry in entries:
        example_id = entry["id"]
        try:
            page_lines = (tapl_root / entry["page"]).read_text(encoding="utf-8-sig").splitlines()
            display_kind = entry["display"].get("kind")
            if display_kind == "listing":
                anchor = re.compile(r"<Listing\b.*?example-id=[\"']" + re.escape(example_id) + r"[\"']")
            elif display_kind == "fence":
                fence = re.escape(chr(96) * 3)
                anchor = re.compile(r"^\s*" + fence + r"auto(?:\s|$).*?example-id=[\"']" + re.escape(example_id) + r"[\"']")
            else:
                errors.append(f"{example_id}: unsupported display anchor kind {display_kind!r}")
                anchor = None
            if anchor is not None:
                anchor_lines = [index for index, line in enumerate(page_lines, 1) if anchor.search(line)]
                if len(anchor_lines) != 1:
                    errors.append(f"{example_id}: expected one display anchor, found {len(anchor_lines)}")
                elif entry.get("line") != anchor_lines[0]:
                    errors.append(
                        f"{example_id}: catalog line {entry.get('line')} does not match display anchor line {anchor_lines[0]}"
                    )
            if entry.get("mode") == "display-only":
                reason = str(entry.get("display_only_reason", "")).strip()
                if len(reason) < 12:
                    errors.append(f"{example_id}: display-only entry requires a specific reason")
            digest = hashlib.sha256(display_source_bytes(tapl_root, entry)).hexdigest()
            if digest != entry.get("source_sha256"):
                errors.append(f"{example_id}: display source hash changed; update the catalog")
        except Exception as error:
            errors.append(f"{example_id}: {error}")
    return errors


def request_json(base_url: str, endpoint: str, body: dict, timeout: int):
    request = urllib.request.Request(
        f"{base_url.rstrip('/')}/api/{endpoint}",
        data=json.dumps(body, ensure_ascii=False).encode("utf-8"),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(request, timeout=timeout + 5) as response:
            return response.status, json.loads(response.read().decode("utf-8"))
    except urllib.error.HTTPError as error:
        try:
            payload = json.loads(error.read().decode("utf-8"))
        except Exception:
            payload = {"message": error.reason}
        return error.code, payload
    except (TimeoutError, urllib.error.URLError) as error:
        return 598, {"transport_error": str(error)}


def check_response(entry: dict, status: int, response: dict) -> str | None:
    expected = entry.get("expected") or {}
    kind = expected.get("kind")
    mode = entry.get("mode")
    if mode == "display-only":
        return None
    if response.get("transport_error"):
        return f"API request transport failed: {response['transport_error']}"
    if mode == "expected-error":
        diagnostic = str(expected.get("diagnostic", ""))
        actual = str(response.get("message", response.get("error", response.get("stdout", ""))))
        if status < 400:
            return "expected an error, but the request succeeded"
        if diagnostic and diagnostic not in actual:
            return f"diagnostic mismatch: expected {diagnostic!r}, got {actual!r}"
        return None
    if status < 200 or status >= 300:
        return f"unexpected HTTP status {status}: {response}"
    if mode == "test":
        if kind != "tests":
            return f"test mode requires a tests expectation, got {kind!r}"
        for field in ("executed", "passed", "failed"):
            if field in expected and response.get(field) != expected[field]:
                return f"{field}: expected {expected[field]!r}, got {response.get(field)!r}"
        reports = response.get("reports", [])
        failed = [report for report in reports if not report.get("passed")]
        if failed:
            return f"{len(failed)} test(s) failed: {failed}"
        return None
    stdout = str(response.get("stdout", ""))
    if kind == "stdout":
        if stdout != expected.get("stdout", ""):
            return f"stdout mismatch: expected {expected.get('stdout')!r}, got {stdout!r}"
    elif kind == "stdout_contains":
        missing = [item for item in expected.get("contains", []) if str(item) not in stdout]
        if missing:
            return f"stdout is missing expected text {missing!r}: got {stdout!r}"
    elif kind == "stdout_regex":
        pattern = str(expected.get("pattern", ""))
        if re.search(pattern, stdout) is None:
            return f"stdout does not match {pattern!r}: got {stdout!r}"
    elif kind == "result":
        if str(response.get("result", "")) != str(expected.get("value", "")):
            return f"result mismatch: expected {expected.get('value')!r}, got {response.get('result')!r}"
    else:
        return f"unsupported expectation kind {kind!r}"
    return None


def compact_response(mode: str, response: dict) -> dict:
    if mode == "test":
        reports = [
            {
                key: report[key]
                for key in ("qualified_name", "passed", "error", "duration_ms", "stdout")
                if key in report
            }
            for report in response.get("reports", [])
        ]
        return {
            key: response[key]
            for key in ("executed", "passed", "failed")
            if key in response
        } | {"reports": reports}
    return {
        key: response[key]
        for key in ("stdout", "result", "time_ms", "message", "error", "transport_error")
        if key in response
    }


def negative_control() -> None:
    sample = {"stdout": "right\n", "result": "", "time_ms": 1}
    expected = {"kind": "stdout", "stdout": "wrong\n"}
    if check_response({"mode": "run", "expected": expected}, 200, sample) is None:
        raise SystemExit("negative control failed: deliberately incorrect expected output was accepted")
    regex_expected = {"kind": "stdout_regex", "pattern": r"wrong\\n"}
    if check_response({"mode": "run", "expected": regex_expected}, 200, sample) is None:
        raise SystemExit("negative control failed: deliberately incorrect regex was accepted")
    print("negative control: deliberately incorrect stdout is rejected")


def negative_catalog_control(tapl_root: Path, entries: list[dict]) -> None:
    if not entries:
        raise SystemExit("negative catalog control requires at least one entry")
    mutated = [dict(entry) for entry in entries]
    mutated[0]["line"] = -1
    errors = audit_bindings(tapl_root, mutated)
    if not any("does not match display anchor line" in error for error in errors):
        raise SystemExit("negative catalog control failed: a stale display anchor line was accepted")
    print("negative control: stale page anchor line is rejected")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--book-root", help="book repository root (contains tapl/examples.json)")
    parser.add_argument("--base-url", default="http://127.0.0.1:3030")
    parser.add_argument("--report", help="write detailed execution results as JSON")
    parser.add_argument("--audit-only", action="store_true", help="check bindings and hashes without calling the API")
    parser.add_argument("--negative-control", action="store_true", help="prove wrong output and stale page anchors are rejected")
    parser.add_argument("--select", help="execute entries whose stable ID contains this text; catalog audit remains full")
    args = parser.parse_args()

    book_root = find_book_root(args.book_root)
    tapl_root = book_root / "tapl"
    author_manifest = json.loads((tapl_root / "examples.json").read_text(encoding="utf-8-sig"))
    materialized_path = REPO_ROOT / "website" / "public" / "tapl-examples.json"
    manifest = (
        json.loads(materialized_path.read_text(encoding="utf-8-sig"))
        if materialized_path.is_file()
        else author_manifest
    )
    entries = manifest.get("entries", [])
    errors = audit_bindings(tapl_root, entries)
    if args.negative_control:
        negative_control()
        negative_catalog_control(tapl_root, entries)
    if errors:
        print("catalog audit failed:", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print(f"catalog audit: {len(entries)} descriptors, all uniquely bound, correctly located and source-hash matched")
    if args.audit_only:
        return 0

    run_entries = entries if not args.select else [entry for entry in entries if args.select in entry["id"]]
    if not run_entries:
        raise SystemExit(f"no catalog entries match --select {args.select!r}")
    results = []
    failures = []
    started = time.monotonic()
    for index, entry in enumerate(run_entries, 1):
        mode = entry.get("mode")
        if mode == "display-only":
            results.append({"id": entry["id"], "mode": mode, "status": "display-only"})
            continue
        execution = entry.get("execution") or {}
        if not execution:
            source = display_code(tapl_root, entry)
            files = entry.get("files", [])
        else:
            source = execution.get("source", "")
            files = execution.get("files", entry.get("files", []))
        body = {
            "source": source,
            "files": files,
            "entry": execution.get("entry", entry.get("entry", "main.at")),
            "input": execution.get("input", entry.get("input", "")),
            "timeout_secs": execution.get("timeout_secs", entry.get("timeout_secs", 10)),
        }
        endpoint = "test" if mode == "test" else "run"
        status, response = request_json(args.base_url, endpoint, body, int(body["timeout_secs"]))
        error = check_response(entry, status, response)
        results.append(
            {
                "id": entry["id"],
                "mode": mode,
                "endpoint": endpoint,
                "http_status": status,
                "response": compact_response(mode, response),
                "error": error,
            }
        )
        if error:
            failures.append(f"{entry['id']}: {error}")
        if index % 25 == 0 or index == len(run_entries):
            print(f"checked {index}/{len(run_entries)} entries")

    report = {
        "schema_version": manifest.get("schema_version"),
        "book": "tapl",
        "entry_count": len(run_entries),
        "elapsed_seconds": round(time.monotonic() - started, 3),
        "passed": len(results) - len(failures),
        "failed": len(failures),
        "results": results,
    }
    if args.report:
        report_path = Path(args.report)
        report_path.parent.mkdir(parents=True, exist_ok=True)
        report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(f"execution: {report['passed']} passed, {report['failed']} failed in {report['elapsed_seconds']}s")
    for failure in failures[:60]:
        print(f"  - {failure}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
