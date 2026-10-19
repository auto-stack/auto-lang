#!/usr/bin/env python3
"""Audit the TAPL execution catalog and optionally run each runnable example."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import tempfile
import time
from collections import Counter
from pathlib import Path, PurePosixPath
from typing import Any


RUNNABLE_MODES = {"run", "test", "expected-error"}


class AuditError(RuntimeError):
    pass


def book_path(book_root: Path, reference: str) -> Path:
    normalized = reference.replace("\\", "/")
    candidate = Path(normalized)
    if candidate.is_absolute():
        resolved = candidate.resolve()
        if not resolved.is_relative_to(book_root.resolve()):
            raise AuditError(f"reference escapes book root: {reference}")
        return resolved
    if normalized.startswith("tapl/"):
        resolved = (book_root / normalized).resolve()
    else:
        resolved = (book_root / "tapl" / normalized).resolve()
    if not resolved.is_relative_to(book_root.resolve()):
        raise AuditError(f"reference escapes book root: {reference}")
    return resolved


def safe_project_path(root: Path, relative: str) -> Path:
    path = PurePosixPath(relative.replace("\\", "/"))
    if path.is_absolute() or any(part in ("", ".", "..") for part in path.parts):
        raise AuditError(f"unsafe project file path: {relative}")
    target = (root / Path(*path.parts)).resolve()
    if not target.is_relative_to(root.resolve()):
        raise AuditError(f"project file escapes temporary root: {relative}")
    return target


def extract_fence(book_root: Path, entry: dict[str, Any]) -> tuple[str, bytes]:
    page = book_path(book_root, entry["page"])
    lines = page.read_text(encoding="utf-8").splitlines()
    line_no = int(entry["line"])
    if not 1 <= line_no <= len(lines):
        raise AuditError(f"{entry['id']}: line {line_no} is outside {page}")
    opening = lines[line_no - 1]
    marker = f'example-id="{entry["id"]}"'
    if marker not in opening:
        raise AuditError(f"{entry['id']}: line {line_no} no longer contains its example marker")
    match = re.match(r"^\s*(`{3,}|~{3,})", opening)
    if not match:
        raise AuditError(f"{entry['id']}: example marker is not on a fenced code opener")
    fence = match.group(1)
    closing_index = next(
        (index for index in range(line_no, len(lines)) if lines[index].strip().startswith(fence)),
        None,
    )
    if closing_index is None:
        raise AuditError(f"{entry['id']}: code fence is not closed")
    source = "\n".join(lines[line_no:closing_index]).rstrip("\r\n")
    source_bytes = (source + "\n").encode("utf-8") if source else b""
    return source, source_bytes


def read_source(book_root: Path, entry: dict[str, Any]) -> tuple[str, bytes, str]:
    if isinstance(entry.get("run_source"), str):
        source = entry["run_source"]
        return source, source.encode("utf-8"), "run_source"
    source_ref = entry.get("source_file") or entry.get("display", {}).get("source")
    if entry.get("display", {}).get("kind") == "fence" and not entry.get("source_file"):
        source, digest_bytes = extract_fence(book_root, entry)
        return source, digest_bytes, entry["page"]
    if not isinstance(source_ref, str):
        raise AuditError(f"{entry['id']}: no source reference")
    path = book_path(book_root, source_ref)
    raw = path.read_bytes()
    try:
        source = raw.decode("utf-8-sig")
    except UnicodeDecodeError as error:
        raise AuditError(f"{entry['id']}: source is not UTF-8: {path}") from error
    return source, raw, source_ref


def static_audit(book_root: Path, entries: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], dict[str, int]]:
    errors: list[dict[str, Any]] = []
    ids: set[str] = set()
    modes: Counter[str] = Counter()
    catalog_ids = {entry.get("id") for entry in entries if isinstance(entry.get("id"), str)}
    page_markers: dict[str, list[str]] = {}

    for page in sorted((book_root / "tapl").glob("*.md")):
        page_text = page.read_text(encoding="utf-8")
        for marker_id in re.findall(r'example-id\s*=\s*["\']([^"\']+)["\']', page_text):
            page_markers.setdefault(marker_id, []).append(page.relative_to(book_root).as_posix())

    for marker_id, marker_pages in sorted(page_markers.items()):
        if marker_id not in catalog_ids:
            errors.append({"id": marker_id, "stage": "catalog", "error": "page marker has no catalog entry"})
        if len(marker_pages) > 1:
            errors.append({"id": marker_id, "stage": "catalog", "error": f"page marker occurs {len(marker_pages)} times: {marker_pages}"})

    for catalog_id in sorted(catalog_ids):
        if catalog_id not in page_markers:
            errors.append({"id": catalog_id, "stage": "catalog", "error": "catalog entry has no page marker"})

    for entry in entries:
        example_id = entry.get("id", "<missing-id>")
        mode = entry.get("mode", "")
        modes[mode] += 1
        if example_id in ids:
            errors.append({"id": example_id, "stage": "catalog", "error": "duplicate id"})
        ids.add(example_id)
        try:
            page = book_path(book_root, entry["page"])
            page_text = page.read_text(encoding="utf-8")
            marker = f'example-id="{example_id}"'
            if page_text.count(marker) != 1:
                raise AuditError(f"expected one page marker, found {page_text.count(marker)}")
            marker_line = next(i for i, line in enumerate(page_text.splitlines(), 1) if marker in line)
            if int(entry.get("line", -1)) != marker_line:
                raise AuditError(f"line map is {entry.get('line')}, actual marker is {marker_line}")
            source, source_bytes, source_origin = read_source(book_root, entry)
            actual_hash = hashlib.sha256(source_bytes).hexdigest()
            expected_hash = str(entry.get("source_sha256", "")).lower()
            if actual_hash != expected_hash:
                raise AuditError(f"source hash mismatch for {source_origin}: {actual_hash}")
            if mode in RUNNABLE_MODES and not source.strip():
                raise AuditError("runnable source is empty")
            if mode == "display-only":
                if entry.get("expected", {}).get("kind") != "display-only":
                    raise AuditError("display-only mode must have expected.kind=display-only")
                if not entry.get("display_only_reason", "").strip():
                    raise AuditError("display-only example has no display_only_reason")
            elif mode == "local-only":
                if entry.get("expected", {}).get("kind") != "local-only":
                    raise AuditError("local-only mode must have expected.kind=local-only")
                if not entry.get("local_only_reason", "").strip():
                    raise AuditError("local-only example has no local_only_reason")
            elif mode not in RUNNABLE_MODES:
                raise AuditError(f"unsupported execution mode: {mode}")

            if mode in RUNNABLE_MODES and entry.get("expected", {}).get("kind") in ("display-only", "local-only"):
                raise AuditError("runnable mode has a non-runnable expected kind")
        except (AuditError, KeyError, OSError, ValueError) as error:
            errors.append({"id": example_id, "stage": "static", "error": str(error)})

    return errors, dict(modes)


def refresh_source_metadata(book_root: Path, manifest_path: Path, manifest: dict[str, Any]) -> None:
    """Explicitly refresh line anchors and hashes after an intentional source edit."""
    for entry in manifest.get("entries", []):
        page = book_path(book_root, entry["page"])
        page_text = page.read_text(encoding="utf-8")
        marker = f'example-id="{entry["id"]}"'
        matching_lines = [index for index, line in enumerate(page_text.splitlines(), 1) if marker in line]
        if len(matching_lines) != 1:
            raise AuditError(f"{entry['id']}: expected one page marker, found {len(matching_lines)}")
        entry["line"] = matching_lines[0]
        _, source_bytes, _ = read_source(book_root, entry)
        entry["source_sha256"] = hashlib.sha256(source_bytes).hexdigest()
    manifest_path.write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def materialize_files(
    book_root: Path,
    temp_root: Path,
    entry: dict[str, Any],
    source: str,
) -> Path:
    entry_name = entry.get("entry") or "main.at"
    entry_path = safe_project_path(temp_root, entry_name)
    entry_path.parent.mkdir(parents=True, exist_ok=True)
    entry_path.write_text(source, encoding="utf-8", newline="")

    for item in entry.get("files", []):
        if isinstance(item, str):
            file_ref = item
            target_name = PurePosixPath(item.replace("\\", "/")).name
            content = book_path(book_root, file_ref).read_text(encoding="utf-8")
        elif isinstance(item, dict):
            target_name = item.get("path")
            if not isinstance(target_name, str):
                raise AuditError(f"{entry['id']}: project file has no path")
            if isinstance(item.get("content"), str):
                content = item["content"]
            elif isinstance(item.get("source_code"), str):
                content = item["source_code"]
            elif isinstance(item.get("file"), str):
                content = book_path(book_root, item["file"]).read_text(encoding="utf-8")
            else:
                raise AuditError(f"{entry['id']}: project file {target_name} has no content reference")
        else:
            raise AuditError(f"{entry['id']}: unsupported files entry {item!r}")

        target = safe_project_path(temp_root, target_name)
        if target == entry_path:
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(content, encoding="utf-8", newline="")
    return entry_path


def execute_entry(
    book_root: Path,
    entry: dict[str, Any],
    auto: Path,
    run_cwd: Path,
) -> dict[str, Any]:
    started = time.monotonic()
    mode = entry["mode"]
    expected = entry.get("expected", {})
    input_text = entry.get("input") or ""
    timeout_secs = min(60, max(1, int(entry.get("timeout_secs", 10))))
    source, _, _ = read_source(book_root, entry)

    with tempfile.TemporaryDirectory(prefix="tapl-audit-") as temp_name:
        temp_root = Path(temp_name)
        entry_path = materialize_files(book_root, temp_root, entry, source)
        if mode == "test":
            command = [str(auto), "test", "--dir", str(entry_path)]
        else:
            command = [str(auto), "--format", "json", str(entry_path)]
        env = os.environ.copy()
        env["AUTO_LANG_ROOT"] = str(run_cwd)
        try:
            result = subprocess.run(
                command,
                cwd=temp_root,
                env=env,
                input=input_text,
                text=True,
                encoding="utf-8",
                errors="replace",
                capture_output=True,
                timeout=timeout_secs + 5,
                check=False,
            )
        except subprocess.TimeoutExpired as error:
            return {
                "id": entry["id"],
                "mode": mode,
                "status": "fail",
                "duration_ms": round((time.monotonic() - started) * 1000),
                "error": f"CLI timed out after {timeout_secs + 5}s",
                "stdout": (error.stdout or b"").decode("utf-8", "replace")[-2000:] if isinstance(error.stdout, bytes) else (error.stdout or "")[-2000:],
                "stderr": (error.stderr or b"").decode("utf-8", "replace")[-2000:] if isinstance(error.stderr, bytes) else (error.stderr or "")[-2000:],
            }

    stdout = result.stdout
    if mode in {"run", "expected-error"}:
        status_start = stdout.rfind('\n{"status":')
        status_text = stdout[status_start + 1 :].strip() if status_start >= 0 else stdout.strip()
        try:
            status = json.loads(status_text)
        except json.JSONDecodeError:
            status = None
        if isinstance(status, dict) and "status" in status and "result" in status:
            stdout = stdout[:status_start] if status_start >= 0 else ""
    combined = stdout + result.stderr
    failure: str | None = None
    kind = expected.get("kind")
    if mode == "run":
        if result.returncode != 0:
            failure = f"CLI exited {result.returncode}"
        elif kind == "stdout" and stdout != expected.get("stdout", ""):
            failure = "stdout differs from expected output"
        elif kind == "stdout_regex" and not re.fullmatch(expected.get("pattern", ""), stdout, flags=re.DOTALL):
            failure = "stdout does not match expected pattern"
        elif kind == "stdout_contains":
            missing = [text for text in expected.get("contains", []) if text not in stdout]
            if missing:
                failure = f"stdout is missing expected text: {missing!r}"
        elif kind not in {"stdout", "stdout_regex", "stdout_contains"}:
            failure = f"unsupported run expectation: {kind}"
    elif mode == "test":
        summary = re.search(r"test result:\s*(\d+) passed,\s*(\d+) failed", combined)
        if result.returncode != 0:
            failure = f"test CLI exited {result.returncode}"
        elif kind != "tests" or not summary:
            failure = "test result summary was not found"
        elif int(summary.group(1)) < 1 or int(summary.group(2)) != 0:
            failure = f"test summary reports {summary.group(1)} passed, {summary.group(2)} failed"
    elif mode == "expected-error":
        diagnostic = expected.get("diagnostic")
        if result.returncode == 0:
            failure = "expected an error but CLI exited successfully"
        elif not isinstance(diagnostic, str) or diagnostic not in combined:
            failure = f"expected diagnostic not found: {diagnostic!r}"
    else:
        failure = f"unsupported executable mode: {mode}"

    return {
        "id": entry["id"],
        "mode": mode,
        "status": "fail" if failure else "pass",
        "duration_ms": round((time.monotonic() - started) * 1000),
        "returncode": result.returncode,
        "stdout_sha256": hashlib.sha256(stdout.encode("utf-8")).hexdigest(),
        "stdout": stdout[-2000:],
        "stderr": result.stderr[-2000:],
        **({"error": failure} if failure else {}),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--book-root", type=Path, required=True, help="book repository root containing tapl/examples.json")
    parser.add_argument("--auto", type=Path, help="built Auto CLI executable")
    parser.add_argument("--run-cwd", type=Path, help="AutoLang repository root used for stdlib/dependency resolution")
    parser.add_argument("--audit-only", action="store_true", help="validate metadata and source maps without executing programs")
    parser.add_argument("--refresh-source-metadata", action="store_true", help="rewrite catalog line anchors and SHA-256 values from current sources")
    parser.add_argument("--report", type=Path, help="write the complete JSON receipt to this path")
    parser.add_argument("--keep-going", action="store_true", help="continue after execution failures")
    options = parser.parse_args()

    book_root = options.book_root.resolve()
    manifest_path = book_root / "tapl" / "examples.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    entries = manifest.get("entries", [])
    if options.refresh_source_metadata:
        refresh_source_metadata(book_root, manifest_path, manifest)
    static_errors, modes = static_audit(book_root, entries)
    results = list(static_errors)
    runnable = [entry for entry in entries if entry.get("mode") in RUNNABLE_MODES]
    if not options.audit_only and (not options.auto or not options.run_cwd):
        parser.error("--auto and --run-cwd are required unless --audit-only is used")

    if not options.audit_only and not static_errors:
        auto = options.auto.resolve()
        run_cwd = options.run_cwd.resolve()
        for index, entry in enumerate(runnable, 1):
            result = execute_entry(book_root, entry, auto, run_cwd)
            results.append(result)
            if result["status"] == "fail":
                print(f"FAIL {index}/{len(runnable)} {result['id']}: {result.get('error')}")
                if not options.keep_going:
                    break
            elif index % 25 == 0 or index == len(runnable):
                print(f"checked {index}/{len(runnable)} runnable examples")

    pass_count = sum(result.get("status") == "pass" for result in results)
    fail_count = sum(result.get("status") == "fail" or "error" in result for result in results)
    skipped_count = sum(entry.get("mode") not in RUNNABLE_MODES for entry in entries)
    receipt = {
        "schema_version": 1,
        "book": manifest.get("book"),
        "manifest_entries": len(entries),
        "mode_counts": modes,
        "runnable_entries": len(runnable),
        "executed_entries": sum("duration_ms" in result for result in results),
        "skipped_entries": skipped_count,
        "passed": pass_count,
        "failed": fail_count,
        "audit_only": options.audit_only,
        "results": results,
    }
    if options.report:
        options.report.parent.mkdir(parents=True, exist_ok=True)
        options.report.write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({key: value for key, value in receipt.items() if key != "results"}, ensure_ascii=False))
    return 1 if fail_count else 0


if __name__ == "__main__":
    raise SystemExit(main())
