#!/usr/bin/env python3
"""PLAN-1057/1059: AutoDown document-case parity, record & control verifier.

Subcommands (PLAN-1057 §5.8/§6.2/§6.4; PLAN-1059 §5.5/§6.2):
  pair     --runner <auto-case> --old-suite <path> --new-suite <path>
           --family scalar|console|immutable|composite --out <fresh-dir>
  verify   --old-record <record.json> --new-record <record.json>
           --family scalar|console|immutable|composite
  controls --runner <auto-case> --out <fresh-dir> [--family composite]

`controls` without --family keeps the PLAN-1057 control contract (V-01..V-11).
`controls --family composite` runs the PLAN-1059 second-wave controls
(NC-01..NC-15) against a scope-staged candidate BASE that consumes the new
composite document.

Offline verify rebinds derived values to the actual files: VM result text is
re-read from each recorded result.txt and a2r artifact digests are recomputed
from the stored paths before judging against the frozen expectations; the
record's cached derivations must agree with the files (PLAN-1059 T-03).

Exit codes for verify / pair:
  0 = all checks & actuals match fixed contract
  1 = semantic, membership, metadata/skip, or actual vs expected mismatch
  2 = missing/corrupt/out-of-scope evidence, bad hash, invalid schema, or non-empty out dir
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import time
from pathlib import Path
from typing import Any


RECORD_SCHEMA = "autolang.case-migration-record/1"
INVENTORY_SCHEMA = "autolang.case-migration/1"
AVM_FAILURE_SCHEMA = "autolang.avm-failure/1"

SCALAR_CASES = [
    "ace.m1.b01",
    "ace.m1.b02",
    "ace.m1.b03",
    "ace.m1.b04",
    "ace.m1.b05",
    "ace.m1.b06",
    "ace.m1.b07",
    "ace.m1.b09",
    "ace.m1.b10",
    "ace.m1.b12",
]
SCALAR_CHECKS = ["ac-native", "av", "autovm", "a2r-snapshot"]
SCALAR_MIGRATED_PAIRS = [(c, k) for c in SCALAR_CASES for k in SCALAR_CHECKS]

EXTRA_BASE_PAIRS = [
    ("ace.m1.b13", "ac-native"),
    ("ace.m1.b13", "av"),
    ("ace.m1.b14", "ac-native"),
    ("ace.m1.b14", "av"),
    ("ace.m1.b14", "autovm"),
    ("ace.m1.b15", "ac-native"),
    ("ace.m1.b15", "av"),
    ("ace.m1.b20", "ac-native"),
    ("ace.m1.b20", "av"),
    ("ace.m1.b20", "autovm"),
    ("ace.m1.b16", "ac-native"),
    ("ace.m1.b16", "av"),
    ("ace.m1.b16", "autovm"),
    ("ace.m1.b17", "ac-native"),
    ("ace.m1.b17", "av"),
    ("ace.m1.b18", "ac-native"),
    ("ace.m1.b18", "av"),
    ("ace.m1.b19", "ac-native"),
    ("ace.m1.b19", "av"),
    ("ace.m1.k01", "autovm"),
    ("ace.m1.k01", "a2r-snapshot"),
    ("ace.m1.k03", "autovm"),
    ("ace.m1.k03", "a2r-snapshot"),
    ("ace.m1.k06", "autovm"),
    ("ace.m1.k06", "a2r-snapshot"),
    ("ace.m1.r02", "autovm"),
    ("ace.m1.r02", "a2r-snapshot"),
    ("ace.m1.r05", "autovm"),
    ("ace.m1.r05", "a2r-snapshot"),
    ("ace.m1.r06", "autovm"),
    ("ace.m1.r06", "a2r-snapshot"),
    ("ace.m1.r08", "autovm"),
    ("ace.m1.r08", "a2r-snapshot"),
    ("ace.m1.t05", "autovm"),
    ("ace.m1.t05", "a2r-snapshot"),
    ("ace.m1.v01", "autovm"),
    ("ace.m1.v01", "a2r-snapshot"),
    ("ace.m1.v02", "autovm"),
    ("ace.m1.v02", "a2r-snapshot"),
    ("ace.m1.v04", "autovm"),
    ("ace.m1.v04", "a2r-snapshot"),
    ("ace.m1.v05", "autovm"),
    ("ace.m1.v05", "a2r-snapshot"),
    ("ace.m1.v06", "autovm"),
    ("ace.m1.v06", "a2r-snapshot"),
    ("ace.m1.v07", "autovm"),
    ("ace.m1.v07", "a2r-snapshot"),
    ("ace.m1.t07", "autovm"),
    ("ace.m1.t07", "a2r-snapshot"),
    ("ace.m1.k04", "autovm"),
    ("ace.m1.k04", "a2r-snapshot"),
    ("ace.m1.session.av", "warm-run"),
    ("ace.m1.session.av", "hot-run"),
    ("ace.m1.session.av", "body-edit-run"),
    ("ace.m1.session.av", "edit-persist-across-workers"),
    ("ace.m1.session.mn", "fresh-run"),
    ("ace.m1.session.mn", "edit-then-restart-run"),
]

CONSOLE_CASES = [f"ace.m1.o{n:02d}" for n in range(1, 16)]
CONSOLE_PAIRS = [(c, "av-console") for c in CONSOLE_CASES]

IMMUTABLE_CASES = [
    "ace.m1.ia920.b18",
    "ace.m1.ia920.b18c",
    "ace.m1.ia920.top",
    "ace.m1.ia920.nb18",
    "ace.m1.ia920.nb18s",
]
IMMUTABLE_PAIRS = [(c, "autovm") for c in IMMUTABLE_CASES]

# PLAN-1059 second wave: the 14 legacy composite cases (28 checks) migrate
# from ace-m1/legacy-composite.case.ad into the inline composite document.
COMPOSITE_CASES = [
    "ace.m1.k01",
    "ace.m1.k03",
    "ace.m1.k06",
    "ace.m1.r02",
    "ace.m1.r05",
    "ace.m1.r06",
    "ace.m1.r08",
    "ace.m1.t05",
    "ace.m1.v01",
    "ace.m1.v02",
    "ace.m1.v04",
    "ace.m1.v05",
    "ace.m1.v06",
    "ace.m1.v07",
]
COMPOSITE_CHECKS = ["autovm", "a2r-snapshot"]
COMPOSITE_MIGRATED_PAIRS = [(c, k) for c in COMPOSITE_CASES for k in COMPOSITE_CHECKS]
# The BASE denominator stays the 1057 fixed 97-pair collection; the composite
# wave reuses it verbatim (its 28 pairs are already inside EXTRA_BASE_PAIRS)
# instead of appending a second copy.
BASE_EXPECTED_PAIRS = SCALAR_MIGRATED_PAIRS + EXTRA_BASE_PAIRS

FAMILY_SPECS: dict[str, dict[str, Any]] = {
    "scalar": {
        "migrated_cases": SCALAR_CASES,
        "migrated_pairs": SCALAR_MIGRATED_PAIRS,
        "expected_pairs": SCALAR_MIGRATED_PAIRS + EXTRA_BASE_PAIRS,
        "required_links": ["PLAN-852"],
        "entry_modes": {},
    },
    "console": {
        "migrated_cases": CONSOLE_CASES,
        "migrated_pairs": CONSOLE_PAIRS,
        "expected_pairs": CONSOLE_PAIRS,
        "required_links": ["PLAN-863"],
        "entry_modes": {},
    },
    "immutable": {
        "migrated_cases": IMMUTABLE_CASES,
        "migrated_pairs": IMMUTABLE_PAIRS,
        "expected_pairs": IMMUTABLE_PAIRS,
        "required_links": ["PLAN-920"],
        "entry_modes": {"ace.m1.ia920.top": "script"},
    },
    "composite": {
        "migrated_cases": COMPOSITE_CASES,
        "migrated_pairs": COMPOSITE_MIGRATED_PAIRS,
        "expected_pairs": BASE_EXPECTED_PAIRS,
        "required_links": ["PLAN-852"],
        "entry_modes": {},
    },
}


class EvidenceError(Exception):
    """Missing/corrupt/out-of-scope evidence or malformed input -> exit 2."""


class ParityError(Exception):
    """Semantic, membership, metadata, or actual-vs-expected failure -> exit 1."""


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> tuple[int, str]:
    raw = path.read_bytes()
    return len(raw), sha256_bytes(raw)


def map_av_diag_stage(raw_stage: str) -> str:
    if raw_stage in ("admission", "frontend", "parse", "text"):
        return "parse"
    if raw_stage in ("resolve", "bind"):
        return "resolve"
    if raw_stage == "typecheck":
        return "typecheck"
    return "verify"


def ensure_fresh_empty_dir(path: Path) -> Path:
    resolved = path.resolve()
    if resolved.exists():
        if not resolved.is_dir():
            raise EvidenceError(f"--out exists and is not a directory: {resolved}")
        if any(resolved.iterdir()):
            raise EvidenceError(f"--out directory is not empty: {resolved}")
    resolved.mkdir(parents=True, exist_ok=True)
    return resolved


def repo_root_from_script() -> Path:
    return Path(__file__).resolve().parents[2]


def git_info(repo_root: Path) -> dict[str, Any]:
    def _run(args: list[str]) -> str:
        proc = subprocess.run(
            ["git", *args],
            cwd=str(repo_root),
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
        )
        return proc.stdout.strip() if proc.returncode == 0 else ""

    commit = _run(["rev-parse", "HEAD"])
    status = _run(["status", "--porcelain"])
    return {
        "frozen_commit": commit,
        "source_tree_status": "clean" if not status else "dirty",
        "dirty_paths": [line for line in status.splitlines() if line],
    }


def worker_env(repo_root: Path) -> tuple[dict[str, str], dict[str, Any]]:
    env = dict(os.environ)
    ext = ".exe" if os.name == "nt" else ""
    tools: dict[str, Any] = {}
    candidates = {
        "AC_AV_CASE_WORKER": repo_root / "experimental/ace/target/release" / f"av-worker{ext}",
        "AC_MN_CASE_WORKER": repo_root / "experimental/ace/target/release" / f"mn-worker{ext}",
    }
    for env_key, default_bin in candidates.items():
        chosen = env.get(env_key)
        path = Path(chosen) if chosen else default_bin
        if path.is_file():
            env[env_key] = str(path.resolve())
            size, digest = sha256_file(path)
            tools[env_key] = {"path": str(path.resolve()), "bytes": size, "sha256": digest}
    return env, tools


def run_cli(
    cmd: list[str], cwd: Path, env: dict[str, str] | None = None
) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(cmd, cwd=str(cwd), env=env, capture_output=True)


def run_migration_inventory(
    runner: Path, suite: Path, cwd: Path, env: dict[str, str] | None = None
) -> dict[str, Any]:
    proc = run_cli([str(runner), "migration-inventory", "--suite", str(suite)], cwd=cwd, env=env)
    if proc.returncode != 0:
        raise EvidenceError(
            f"migration-inventory failed (exit {proc.returncode}) for {suite}:\n"
            f"{proc.stderr.decode('utf-8', errors='replace')}"
        )
    try:
        doc = json.loads(proc.stdout.decode("utf-8"))
    except json.JSONDecodeError as exc:
        raise EvidenceError(f"invalid migration-inventory JSON for {suite}: {exc}") from exc
    if doc.get("schema") != INVENTORY_SCHEMA or not isinstance(doc.get("checks"), list):
        raise EvidenceError(f"unexpected migration-inventory schema for {suite}: {doc.get('schema')}")
    return doc


def index_projection(
    proj: dict[str, Any], family: str, side_label: str, require_document_for_migrated: bool
) -> dict[tuple[str, str], dict[str, Any]]:
    spec = FAMILY_SPECS[family]
    expected_pairs = spec["expected_pairs"]
    migrated_set = set(spec["migrated_cases"])
    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    duplicates: list[tuple[str, str]] = []
    for row in proj.get("checks", []):
        sem = row.get("semantic") or {}
        case_obj = sem.get("case") or {}
        chk_obj = sem.get("check") or {}
        key = (case_obj.get("id", ""), chk_obj.get("id", ""))
        if not key[0] or not key[1]:
            raise EvidenceError(f"[{side_label}] inventory row missing case.id/check.id: {row}")
        if key in indexed:
            duplicates.append(key)
        indexed[key] = row

    if duplicates:
        raise ParityError(f"[{side_label}] duplicate (case, check) pairs: {duplicates}")

    expected_set = set(expected_pairs)
    actual_set = set(indexed.keys())
    missing = sorted(expected_set - actual_set)
    extra = sorted(actual_set - expected_set)
    if missing or extra:
        raise ParityError(
            f"[{side_label}] fixed collection mismatch for family={family}: "
            f"missing={missing} extra={extra}"
        )

    for key, row in indexed.items():
        sem = row["semantic"]
        origin = row.get("origin") or {}
        case_id, _check_id = key
        case_obj = sem["case"]
        chk_obj = sem["check"]
        if chk_obj.get("skip") is not None:
            raise ParityError(f"[{side_label}] {key} has non-null check.skip: {chk_obj.get('skip')}")
        if case_obj.get("status_skip") is not None:
            raise ParityError(
                f"[{side_label}] {key} has non-null case.status_skip: {case_obj.get('status_skip')}"
            )
        if case_id in migrated_set:
            if case_obj.get("files") != []:
                raise ParityError(
                    f"[{side_label}] {key} migrated case must have empty files[]: {case_obj.get('files')}"
                )
            expected_mode = spec["entry_modes"].get(case_id, "program")
            if case_obj.get("entry_mode") != expected_mode:
                raise ParityError(
                    f"[{side_label}] {key} entry_mode={case_obj.get('entry_mode')!r} "
                    f"expected {expected_mode!r}"
                )
            for req_link in spec["required_links"]:
                if req_link not in (case_obj.get("links") or []):
                    raise ParityError(
                        f"[{side_label}] {key} missing required link {req_link!r}: {case_obj.get('links')}"
                    )
            if require_document_for_migrated:
                if origin.get("format") != "document":
                    raise ParityError(
                        f"[{side_label}] {key} expected origin.format=='document', got {origin.get('format')!r}"
                    )
                if origin.get("input_paths") != []:
                    raise ParityError(
                        f"[{side_label}] {key} migrated inline doc must have input_paths==[], "
                        f"got {origin.get('input_paths')}"
                    )
    return indexed


def diff_semantic(old_sem: dict[str, Any], new_sem: dict[str, Any], prefix: str = "") -> list[str]:
    diffs: list[str] = []
    if type(old_sem) is not type(new_sem):
        return [f"{prefix}: type {type(old_sem).__name__} != {type(new_sem).__name__} ({old_sem!r} != {new_sem!r})"]
    if isinstance(old_sem, dict):
        all_keys = sorted(set(old_sem.keys()) | set(new_sem.keys()))
        for k in all_keys:
            sub = f"{prefix}.{k}" if prefix else k
            if k not in old_sem:
                diffs.append(f"{sub}: missing in old (new={new_sem[k]!r})")
            elif k not in new_sem:
                diffs.append(f"{sub}: missing in new (old={old_sem[k]!r})")
            else:
                diffs.extend(diff_semantic(old_sem[k], new_sem[k], sub))
    elif isinstance(old_sem, list):
        if len(old_sem) != len(new_sem):
            diffs.append(f"{prefix}: len {len(old_sem)} != {len(new_sem)} ({old_sem!r} != {new_sem!r})")
        else:
            for idx, (a, b) in enumerate(zip(old_sem, new_sem)):
                diffs.extend(diff_semantic(a, b, f"{prefix}[{idx}]"))
    elif old_sem != new_sem:
        diffs.append(f"{prefix}: {old_sem!r} != {new_sem!r}")
    return diffs


def compare_projections(
    family: str,
    old_idx: dict[tuple[str, str], dict[str, Any]],
    new_idx: dict[tuple[str, str], dict[str, Any]],
) -> list[dict[str, Any]]:
    all_diffs: list[dict[str, Any]] = []
    for key in FAMILY_SPECS[family]["expected_pairs"]:
        field_diffs = diff_semantic(old_idx[key]["semantic"], new_idx[key]["semantic"])
        if field_diffs:
            all_diffs.append({"case": key[0], "check": key[1], "diffs": field_diffs})
    if all_diffs:
        first = all_diffs[0]
        raise ParityError(
            f"semantic projection mismatch across {len(all_diffs)} check(s); "
            f"first ({first['case']}, {first['check']}): {first['diffs'][:3]}"
        )
    return all_diffs


def _ensure_under(root: Path, target: Path, label: str) -> Path:
    resolved_root = root.resolve()
    resolved_target = target.resolve()
    try:
        resolved_target.relative_to(resolved_root)
    except ValueError as exc:
        raise EvidenceError(
            f"{label} escapes run directory {resolved_root}: {resolved_target}"
        ) from exc
    return resolved_target


def find_unique_run_dir(out_dir: Path) -> Path:
    if not out_dir.is_dir():
        raise EvidenceError(f"missing run output dir: {out_dir}")
    runs = sorted(p for p in out_dir.iterdir() if p.is_dir() and p.name.startswith("run-"))
    if len(runs) != 1:
        raise EvidenceError(
            f"expected exactly 1 run-* subdir under {out_dir}, found {len(runs)}: {[r.name for r in runs]}"
        )
    return runs[0]


def collect_actuals(
    run_out_dir: Path,
    report_path: Path,
    indexed_proj: dict[tuple[str, str], dict[str, Any]],
    family: str,
) -> tuple[dict[str, Any], dict[str, Any]]:
    if not report_path.is_file():
        raise EvidenceError(f"missing report.json: {report_path}")
    rep_bytes, rep_sha = sha256_file(report_path)
    try:
        report = json.loads(report_path.read_bytes().decode("utf-8"))
    except Exception as exc:
        raise EvidenceError(f"invalid report.json {report_path}: {exc}") from exc

    run_dir = find_unique_run_dir(run_out_dir)
    results = report.get("results")
    if not isinstance(results, list):
        raise EvidenceError(f"report.json missing results list: {report_path}")

    res_map: dict[tuple[str, str], dict[str, Any]] = {}
    dup_res: list[tuple[str, str]] = []
    for r in results:
        k = (r.get("case", ""), r.get("check", ""))
        if k in res_map:
            dup_res.append(k)
        res_map[k] = r
    if dup_res:
        raise ParityError(f"duplicate results in report.json: {dup_res}")

    expected_set = set(FAMILY_SPECS[family]["expected_pairs"])
    actual_set = set(res_map.keys())
    if expected_set != actual_set:
        raise ParityError(
            f"report results set mismatch: missing={sorted(expected_set - actual_set)} "
            f"extra={sorted(actual_set - expected_set)}"
        )

    actuals: dict[str, Any] = {}
    for key in FAMILY_SPECS[family]["expected_pairs"]:
        row_proj = indexed_proj[key]
        origin = row_proj["origin"]
        sem = row_proj["semantic"]
        rep_row = res_map[key]
        check_out_dir = _ensure_under(
            run_dir, run_dir / origin["case_dir"] / origin["check_dir"], f"check dir {key}"
        )
        if not check_out_dir.is_dir():
            raise EvidenceError(f"missing check directory {check_out_dir} for {key}")

        files_meta: dict[str, Any] = {}
        observed: dict[str, Any] = {
            "case": key[0],
            "check": key[1],
            "status": rep_row.get("status"),
            "adapter": sem["check"]["adapter"],
            "action": sem["check"]["action"],
            "recipe": sem["options"].get("probe_recipe"),
            "identity": rep_row.get("identity"),
            "check_dir": str(check_out_dir.relative_to(run_dir)).replace("\\", "/"),
            "files": files_meta,
        }

        def _record_file(tag: str, file_path: Path) -> dict[str, Any]:
            safe = _ensure_under(run_dir, file_path, f"{key}:{tag}")
            if not safe.is_file():
                raise EvidenceError(f"missing actual file for {key} ({tag}): {safe}")
            sz, dig = sha256_file(safe)
            entry = {"path": str(safe), "bytes": sz, "sha256": dig}
            files_meta[tag] = entry
            return entry

        adapter = sem["check"]["adapter"]
        recipe = sem["options"].get("probe_recipe") or ""
        if adapter == "probe" and recipe.startswith("ac/mir-native-"):
            receipt_file = check_out_dir / "mn-probe-receipt.json"
            _record_file("mn-probe-receipt", receipt_file)
            rcp = json.loads(receipt_file.read_bytes().decode("utf-8"))
            sessions = rcp.get("sessions") or []
            if not sessions:
                raise EvidenceError(f"mn-probe-receipt missing sessions[0] for {key}")
            observed["native"] = {
                "report_exit_code": (rep_row.get("native") or {}).get("exit_code"),
                "receipt_exit_code": rcp.get("exit_code"),
                "session_exit_code": sessions[0].get("exit_code"),
                "judged": sessions[0].get("judged"),
                "recipe": rcp.get("recipe"),
                "protocol": rcp.get("protocol"),
                "source_hash": sessions[0].get("source_sha256"),
            }
        elif adapter == "probe" and recipe.startswith("ac-av/"):
            receipt_file = check_out_dir / "probe-receipt.json"
            _record_file("probe-receipt", receipt_file)
            rcp = json.loads(receipt_file.read_bytes().decode("utf-8"))
            step = rcp.get("step") or {}
            console = step.get("console")
            decoded_console = None
            if console is not None:
                try:
                    out_b = base64.b64decode(console.get("stdout_b64") or "", validate=True)
                    err_b = base64.b64decode(console.get("stderr_b64") or "", validate=True)
                except Exception as exc:
                    raise EvidenceError(f"invalid console base64 in {receipt_file}: {exc}") from exc
                decoded_console = {
                    "schema": console.get("schema"),
                    "stdout_truncated": console.get("stdout_truncated"),
                    "stderr_truncated": console.get("stderr_truncated"),
                    "stdout_bytes": len(out_b),
                    "stdout_sha256": sha256_bytes(out_b),
                    "stderr_bytes": len(err_b),
                    "stderr_sha256": sha256_bytes(err_b),
                }
            observed["av"] = {
                "recipe": rcp.get("recipe"),
                "protocol": rcp.get("protocol"),
                "request_hash": rcp.get("request_hash"),
                "outcome": step.get("outcome"),
                "value": step.get("value"),
                "channel": step.get("channel"),
                "trap": step.get("trap"),
                "diag": step.get("diag"),
                "source_hash": step.get("source_hash"),
                "console": decoded_console,
            }
        elif adapter == "avm":
            comp_kind = sem["expect"]["completion"]["kind"]
            if comp_kind == "success":
                res_file = check_out_dir / "result.txt"
                _record_file("result", res_file)
                observed["avm"] = {
                    "kind": "success",
                    "result_text": res_file.read_bytes().decode("utf-8"),
                }
            else:
                fail_file = check_out_dir / "failure.json"
                _record_file("failure", fail_file)
                fdoc = json.loads(fail_file.read_bytes().decode("utf-8"))
                observed["avm"] = {
                    "kind": "error",
                    "schema": fdoc.get("schema"),
                    "source_sha256": fdoc.get("source_sha256"),
                    "entry": fdoc.get("entry"),
                    "entry_sha256": fdoc.get("entry_sha256"),
                    "stage": fdoc.get("stage"),
                    "code": fdoc.get("code"),
                }
        elif adapter == "a2r":
            arts = []
            for a in rep_row.get("artifacts") or []:
                stored = Path(a["stored"])
                _record_file(f"artifact:{a['path']}", stored)
                sz, dig = sha256_file(stored)
                if sz != a.get("bytes") or dig != a.get("sha256"):
                    raise EvidenceError(f"report artifact digest mismatch for {key} {a['path']}")
                arts.append({"path": a["path"], "bytes": sz, "sha256": dig, "stored": str(stored)})
            if sem["expect"]["completion"]["kind"] != "success":
                emit_err = check_out_dir / "emit-error.txt"
                if emit_err.is_file():
                    _record_file("emit-error", emit_err)
            observed["a2r"] = {"artifacts": arts}

        actuals[f"{key[0]}::{key[1]}"] = observed

    report_meta = {
        "report_path": str(report_path.resolve()),
        "report_bytes": rep_bytes,
        "report_sha256": rep_sha,
        "run_dir": str(run_dir.resolve()),
        "suite": report.get("suite"),
        "suite_sha256": report.get("suite_sha256"),
        "summary": report.get("summary"),
    }
    return report_meta, actuals


def verify_actual_file_integrity(
    record: dict[str, Any], side_label: str, require_files: bool = True
) -> None:
    rep_path = Path(record.get("report_path", ""))
    run_dir = Path(record.get("run_dir", ""))
    if not rep_path.is_file():
        raise EvidenceError(f"[{side_label}] report_path missing: {rep_path}")
    sz, dig = sha256_file(rep_path)
    if sz != record.get("report_bytes") or dig != record.get("report_sha256"):
        raise EvidenceError(f"[{side_label}] report hash mismatch: {rep_path}")
    if not run_dir.is_dir():
        raise EvidenceError(f"[{side_label}] run_dir missing: {run_dir}")
    # Also ensure run_dir is still the single unique run-* under its parent
    find_unique_run_dir(run_dir.parent)

    for key_str, obs in (record.get("actuals") or {}).items():
        files_meta = obs.get("files") or {}
        if require_files and (not isinstance(files_meta, dict) or not files_meta):
            raise EvidenceError(f"[{side_label}] {key_str} has no actual files recorded")
        for tag, fmeta in files_meta.items():
            fpath = _ensure_under(run_dir, Path(fmeta["path"]), f"[{side_label}] {key_str}:{tag}")
            if not fpath.is_file():
                raise EvidenceError(f"[{side_label}] missing actual file {key_str}:{tag}: {fpath}")
            csz, cdig = sha256_file(fpath)
            if csz != fmeta.get("bytes") or cdig != fmeta.get("sha256"):
                raise EvidenceError(
                    f"[{side_label}] actual file hash mismatch {key_str}:{tag}: "
                    f"expected {fmeta.get('sha256')} got {cdig}"
                )


def rebind_actuals_from_files(record: dict[str, Any], side_label: str) -> None:
    """PLAN-1059 T-03: re-derive the judged values from the actual files.

    Offline verify must not trust the record's cached derivations: VM result
    text is re-read from the recorded result.txt and a2r artifact digests are
    recomputed from the stored paths. A cached derivation that disagrees with
    its file is corrupt evidence (exit 2); the re-derived values then face the
    frozen expectations in verify_migrated_row_actual (mismatch -> exit 1).
    """
    run_dir = Path(record["run_dir"])
    for key_str, obs in (record.get("actuals") or {}).items():
        files_meta = obs.get("files") or {}
        avm = obs.get("avm")
        if avm is not None and avm.get("kind") == "success":
            fmeta = files_meta.get("result")
            if fmeta is None:
                raise EvidenceError(f"[{side_label}] {key_str} avm success row has no result file entry")
            rpath = _ensure_under(run_dir, Path(fmeta["path"]), f"[{side_label}] {key_str}:result")
            text = rpath.read_bytes().decode("utf-8")
            if text != avm.get("result_text"):
                raise EvidenceError(
                    f"[{side_label}] {key_str} cached avm result_text disagrees with the actual file "
                    f"({avm.get('result_text')!r} vs {text!r} in {rpath})"
                )
            avm["result_text"] = text
        a2r = obs.get("a2r")
        if a2r is not None:
            for art in a2r.get("artifacts") or []:
                tag = f"artifact:{art['path']}"
                fmeta = files_meta.get(tag)
                if fmeta is None:
                    raise EvidenceError(
                        f"[{side_label}] {key_str} a2r artifact {art['path']} has no recorded file entry"
                    )
                apath = _ensure_under(run_dir, Path(fmeta["path"]), f"[{side_label}] {key_str}:{tag}")
                sz, dig = sha256_file(apath)
                if sz != art.get("bytes") or dig != art.get("sha256"):
                    raise EvidenceError(
                        f"[{side_label}] {key_str} cached a2r artifact derivation disagrees with the "
                        f"actual file ({art.get('bytes')}/{art.get('sha256')} vs {sz}/{dig} in {apath})"
                    )
                art["bytes"], art["sha256"] = sz, dig


def verify_migrated_row_actual(
    key: tuple[str, str],
    old_proj: dict[str, Any],
    new_proj: dict[str, Any],
    old_obs: dict[str, Any],
    new_obs: dict[str, Any],
) -> None:
    for side, obs in (("old", old_obs), ("new", new_obs)):
        if obs.get("status") != "PASS":
            raise ParityError(f"[{side}] {key} status is {obs.get('status')!r}, expected PASS")

    sem = old_proj["semantic"]
    orig_old = old_proj["origin"]
    orig_new = new_proj["origin"]
    exp = sem["expect"]
    case_id, check_id = key

    if "native" in old_obs:
        for side, obs in (("old", old_obs["native"]), ("new", new_obs["native"])):
            if not obs.get("judged"):
                raise ParityError(f"[{side}] {key} native judged!=True")
            e = exp["exit"]
            if (
                obs.get("report_exit_code") != e
                or obs.get("receipt_exit_code") != e
                or obs.get("session_exit_code") != e
            ):
                raise ParityError(f"[{side}] {key} native exit mismatch vs {e}: {obs}")
        if old_obs["native"].get("source_hash") != new_obs["native"].get("source_hash"):
            raise EvidenceError(
                f"{key} mn-probe-receipt source_sha256 mismatch old vs new: "
                f"{old_obs['native'].get('source_hash')} vs {new_obs['native'].get('source_hash')}"
            )

    if "av" in old_obs:
        comp = exp["completion"]
        for side, obs, orig in (("old", old_obs["av"], orig_old), ("new", new_obs["av"], orig_new)):
            # Verify live outcome against expected completion/channel/exit
            if comp["kind"] == "success":
                expected_ch = exp.get("av_channel") or "value"
                if expected_ch == "unit":
                    if obs.get("outcome") != "success" or obs.get("channel") != "unit" or obs.get("value") is not None:
                        raise ParityError(f"[{side}] {key} expected AV unit outcome, got {obs}")
                else:
                    if (
                        obs.get("outcome") != "success"
                        or obs.get("channel") != "value"
                        or obs.get("value") != exp["exit"]
                    ):
                        raise ParityError(f"[{side}] {key} expected AV value {exp['exit']}, got {obs}")
            else:
                if comp["stage"] == "run":
                    if obs.get("outcome") != "trap" or obs.get("trap") != comp["code"] or obs.get("channel") is not None:
                        raise ParityError(f"[{side}] {key} expected trap {comp['code']}, got {obs}")
                else:
                    diag = obs.get("diag") or {}
                    mapped = map_av_diag_stage(diag.get("stage", ""))
                    if (
                        obs.get("outcome") != "diag"
                        or mapped != comp["stage"]
                        or diag.get("code") != comp["code"]
                        or obs.get("channel") is not None
                    ):
                        raise ParityError(f"[{side}] {key} expected diag {comp}, got {obs}")
            # Verify console streams from decoded receipt bytes
            for stream in ("stdout", "stderr"):
                es = exp.get(stream)
                if es is not None:
                    con = obs.get("console")
                    if not con or con.get(f"{stream}_truncated"):
                        raise ParityError(f"[{side}] {key} missing/truncated {stream}")
                    if con.get(f"{stream}_bytes") != es["bytes"] or con.get(f"{stream}_sha256") != es["sha256"]:
                        raise ParityError(
                            f"[{side}] {key} {stream} bytes mismatch: "
                            f"actual=({con.get(f'{stream}_bytes')},{con.get(f'{stream}_sha256')}) "
                            f"expect=({es['bytes']},{es['sha256']})"
                        )
        # Cross-check AV receipt source_hash matches old vs new (catches swapped receipts)
        if not old_obs["av"].get("source_hash") or (
            old_obs["av"].get("source_hash") != new_obs["av"].get("source_hash")
        ):
            raise EvidenceError(
                f"{key} AV probe-receipt step.source_hash mismatch old vs new: "
                f"{old_obs['av'].get('source_hash')} vs {new_obs['av'].get('source_hash')}"
            )

    if "avm" in old_obs:
        comp = exp["completion"]
        for side, obs, orig in (("old", old_obs["avm"], orig_old), ("new", new_obs["avm"], orig_new)):
            if comp["kind"] == "success":
                res_exp = exp.get("result") or {}
                if obs.get("kind") != "success" or obs.get("result_text") != res_exp.get("text"):
                    raise ParityError(f"[{side}] {key} autovm result mismatch: {obs} vs {res_exp}")
            else:
                if obs.get("kind") != "error" or obs.get("schema") != AVM_FAILURE_SCHEMA:
                    raise EvidenceError(f"[{side}] {key} missing/bad failure.json schema: {obs}")
                if obs.get("source_sha256") != orig["source_sha256"]:
                    raise EvidenceError(
                        f"[{side}] {key} failure.json source_sha256 mismatch: "
                        f"{obs.get('source_sha256')} != {orig['source_sha256']}"
                    )
                if obs.get("entry_sha256") != sem["case"]["source"]["sha256"]:
                    raise EvidenceError(
                        f"[{side}] {key} failure.json entry_sha256 mismatch: "
                        f"{obs.get('entry_sha256')} != {sem['case']['source']['sha256']}"
                    )
                if obs.get("stage") != comp["stage"] or obs.get("code") != comp["code"]:
                    raise ParityError(f"[{side}] {key} autovm failure stage/code mismatch: {obs} vs {comp}")

    if "a2r" in old_obs:
        exp_arts = {(a["path"], a["bytes"], a["sha256"]) for a in (exp.get("artifacts") or [])}
        for side, obs in (("old", old_obs["a2r"]), ("new", new_obs["a2r"])):
            act_arts = {(a["path"], a["bytes"], a["sha256"]) for a in (obs.get("artifacts") or [])}
            if act_arts != exp_arts:
                raise ParityError(f"[{side}] {key} a2r artifacts mismatch: {act_arts} != {exp_arts}")


def check_source_hash_injectivity(
    record: dict[str, Any], idx: dict[tuple[str, str], dict[str, Any]], family: str, side: str
) -> None:
    """Reject swapped receipts even across distinct source digests within one record."""
    migrated_set = set(FAMILY_SPECS[family]["migrated_pairs"])
    for map_kind, field in (("av", "source_hash"), ("native", "source_hash")):
        by_case_src: dict[str, set[str]] = {}
        by_receipt_src: dict[str, set[str]] = {}
        for key_str, obs in (record.get("actuals") or {}).items():
            cid, kid = key_str.split("::", 1)
            if (cid, kid) not in migrated_set:
                continue
            sub = obs.get(map_kind)
            if not sub or not sub.get(field):
                continue
            row = idx.get((cid, kid))
            if not row:
                continue
            src_dig = row["semantic"]["case"]["source"]["sha256"]
            rcp_dig = sub[field]
            by_case_src.setdefault(src_dig, set()).add(rcp_dig)
            by_receipt_src.setdefault(rcp_dig, set()).add(src_dig)
        for src_dig, rcps in by_case_src.items():
            if len(rcps) > 1:
                raise EvidenceError(
                    f"[{side}] multiple receipt {field} for same source {src_dig}: {rcps}"
                )
        for rcp_dig, srcs in by_receipt_src.items():
            if len(srcs) > 1:
                raise EvidenceError(
                    f"[{side}] same receipt {field} {rcp_dig} mapped to distinct sources: {srcs}"
                )


def verify_records(old_record_path: Path, new_record_path: Path, family: str) -> dict[str, Any]:
    for p in (old_record_path, new_record_path):
        if not p.is_file():
            raise EvidenceError(f"missing record file: {p}")
    try:
        old_rec = json.loads(old_record_path.read_bytes().decode("utf-8"))
        new_rec = json.loads(new_record_path.read_bytes().decode("utf-8"))
    except Exception as exc:
        raise EvidenceError(f"invalid record JSON: {exc}") from exc

    for label, rec in (("old", old_rec), ("new", new_rec)):
        if rec.get("schema") != RECORD_SCHEMA:
            raise EvidenceError(f"[{label}] unsupported record schema: {rec.get('schema')}")
        if rec.get("family") != family:
            raise EvidenceError(f"[{label}] record family {rec.get('family')!r} != {family!r}")
        if rec.get("exit_code") != 0:
            raise ParityError(f"[{label}] run exit_code={rec.get('exit_code')}, expected 0")
        verify_actual_file_integrity(rec, label)
        rebind_actuals_from_files(rec, label)

    old_idx = index_projection(old_rec["projection"], family, "old", require_document_for_migrated=False)
    new_idx = index_projection(new_rec["projection"], family, "new", require_document_for_migrated=True)
    compare_projections(family, old_idx, new_idx)

    check_source_hash_injectivity(old_rec, old_idx, family, "old")
    check_source_hash_injectivity(new_rec, new_idx, family, "new")

    old_acts = old_rec.get("actuals") or {}
    new_acts = new_rec.get("actuals") or {}
    expected_keys = [f"{c}::{k}" for c, k in FAMILY_SPECS[family]["expected_pairs"]]
    if set(old_acts.keys()) != set(expected_keys) or set(new_acts.keys()) != set(expected_keys):
        raise ParityError("actuals keys do not match expected pair collection")

    for c, k in FAMILY_SPECS[family]["migrated_pairs"]:
        ks = f"{c}::{k}"
        verify_migrated_row_actual((c, k), old_idx[(c, k)], new_idx[(c, k)], old_acts[ks], new_acts[ks])

    # Verify non-migrated extra pairs (scalar's 57 extra BASE checks) also passed on both sides
    for c, k in FAMILY_SPECS[family]["expected_pairs"]:
        if (c, k) not in set(FAMILY_SPECS[family]["migrated_pairs"]):
            ks = f"{c}::{k}"
            for side, obs in (("old", old_acts[ks]), ("new", new_acts[ks])):
                if obs.get("status") != "PASS":
                    raise ParityError(f"[{side}] extra BASE check {(c, k)} status={obs.get('status')}")

    return {
        "family": family,
        "verified_pairs": len(expected_keys),
        "migrated_pairs": len(FAMILY_SPECS[family]["migrated_pairs"]),
        "old_report_sha256": old_rec["report_sha256"],
        "new_report_sha256": new_rec["report_sha256"],
    }


def _collect_doc_hashes(proj: dict[str, Any], repo_root: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for row in proj.get("checks", []):
        cf = (row.get("origin") or {}).get("case_file")
        if cf and cf not in out:
            p = repo_root / cf if not Path(cf).is_absolute() else Path(cf)
            if p.is_file():
                out[cf] = sha256_file(p)[1]
    return out


def cmd_pair(args: argparse.Namespace) -> int:
    repo_root = repo_root_from_script()
    runner = Path(args.runner).resolve()
    if not runner.is_file():
        raise EvidenceError(f"missing --runner binary: {runner}")
    old_suite = Path(args.old_suite).resolve()
    new_suite = Path(args.new_suite).resolve()
    for s in (old_suite, new_suite):
        if not s.is_file():
            raise EvidenceError(f"missing suite file: {s}")
    out_dir = ensure_fresh_empty_dir(Path(args.out))

    env, tools = worker_env(repo_root)
    old_proj = run_migration_inventory(runner, old_suite, repo_root, env=env)
    new_proj = run_migration_inventory(runner, new_suite, repo_root, env=env)
    old_idx = index_projection(old_proj, args.family, "old", require_document_for_migrated=False)
    new_idx = index_projection(new_proj, args.family, "new", require_document_for_migrated=True)
    compare_projections(args.family, old_idx, new_idx)

    ginfo = git_info(repo_root)
    runner_sz, runner_sha = sha256_file(runner)

    records: dict[str, Path] = {}
    for side, suite_path, proj, idx in (
        ("old", old_suite, old_proj, old_idx),
        ("new", new_suite, new_proj, new_idx),
    ):
        run_out = out_dir / f"{side}-run"
        run_out.mkdir(parents=True, exist_ok=True)
        report_path = run_out / "report.json"
        cmd = [
            str(runner),
            "run",
            "--suite",
            str(suite_path),
            "--out",
            str(run_out),
            "--json-out",
            str(report_path),
        ]
        proc = run_cli(cmd, cwd=repo_root, env=env)
        (out_dir / f"{side}.stdout").write_bytes(proc.stdout)
        (out_dir / f"{side}.stderr").write_bytes(proc.stderr)
        rep_meta, actuals = collect_actuals(run_out, report_path, idx, args.family)
        rec = {
            "schema": RECORD_SCHEMA,
            "family": args.family,
            "side": side,
            **ginfo,
            "tools": {"runner": {"path": str(runner), "bytes": runner_sz, "sha256": runner_sha}, **tools},
            "suite_path": str(suite_path),
            "suite_sha256": sha256_file(suite_path)[1],
            "doc_sha256s": _collect_doc_hashes(proj, repo_root),
            "command": cmd,
            "exit_code": proc.returncode,
            "projection": proj,
            "actuals": actuals,
            **rep_meta,
        }
        rec_path = out_dir / f"{side}-record.json"
        rec_path.write_text(json.dumps(rec, indent=2, ensure_ascii=False), encoding="utf-8")
        records[side] = rec_path

    summary = verify_records(records["old"], records["new"], args.family)
    rows_table = []
    old_rec = json.loads(records["old"].read_text(encoding="utf-8"))
    new_rec = json.loads(records["new"].read_text(encoding="utf-8"))
    for c, k in FAMILY_SPECS[args.family]["expected_pairs"]:
        ks = f"{c}::{k}"
        # Representation follows each row's ACTUAL origin on the new side
        # (PLAN-1059: from the composite viewpoint the 69 non-migrated rows
        # split into 40 already-document scalar rows + 29 shared-legacy rows;
        # a blanket "shared-legacy" label would misreport the first wave).
        new_format = (new_idx[(c, k)].get("origin") or {}).get("format")
        rows_table.append(
            {
                "case": c,
                "check": k,
                "old_source_sha256": old_idx[(c, k)]["origin"]["source_sha256"],
                "new_source_sha256": new_idx[(c, k)]["origin"]["source_sha256"],
                "old_status": old_rec["actuals"][ks]["status"],
                "new_status": new_rec["actuals"][ks]["status"],
                "old_check_dir": old_rec["actuals"][ks]["check_dir"],
                "new_check_dir": new_rec["actuals"][ks]["check_dir"],
                "representation_diff": (
                    "inline-document" if new_format == "document" else "shared-legacy-include"
                ),
            }
        )
    comparison = {"summary": summary, "rows": rows_table}
    (out_dir / "comparison.json").write_text(
        json.dumps(comparison, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(json.dumps(summary, ensure_ascii=False))
    return 0


def _copy_with_hashes(src: Path, dst: Path, index: list[dict[str, Any]]) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(src, dst)
    bsz, bsha = sha256_file(src)
    asz, asha = sha256_file(dst)
    if (bsz, bsha) != (asz, asha):
        raise EvidenceError(f"copy integrity mismatch {src} -> {dst}")
    index.append({"src": str(src), "dst": str(dst), "bytes": asz, "sha256": asha})


def stage_isolated_corpus(
    repo_root: Path, vid_dir: Path, include_base_closure: bool
) -> tuple[Path, Path, dict[str, Any]]:
    cases_src = repo_root / "crates/auto-lang/test/cases"
    inputs_cases = vid_dir / "inputs/cases"
    output_dir = vid_dir / "output"
    inputs_cases.mkdir(parents=True, exist_ok=True)
    output_dir.mkdir(parents=True, exist_ok=True)
    copied: list[dict[str, Any]] = []
    rel_files = [
        "ace-document-base.suite.ad",
        "ace-document-console.suite.ad",
        "ace-document-immutable.suite.ad",
        "ace-m2/documents/scalar.case.ad",
        "ace-m2/documents/console.case.ad",
        "ace-m2/documents/immutable-assignment.case.ad",
        # PLAN-1059: the composite document joins the staged closure so the
        # 1057 controls keep working after the active BASE include switches.
        "ace-m2/documents/composite.case.ad",
    ]
    for rel in rel_files:
        _copy_with_hashes(cases_src / rel, inputs_cases / rel, copied)

    if include_base_closure:
        ace_m1 = cases_src / "ace-m1"
        for fpath in sorted(ace_m1.rglob("*")):
            if fpath.is_file():
                rel_m1 = fpath.relative_to(cases_src)
                _copy_with_hashes(fpath, inputs_cases / rel_m1, copied)

    manifest = {"files": copied, "count": len(copied)}
    (vid_dir / "closure.json").write_text(
        json.dumps(manifest, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    return inputs_cases, output_dir, manifest


def replace_in_case_block(doc_text: str, case_id: str, old: str, new: str, count: int = 1) -> str:
    marker = f'id: "{case_id}"'
    pos = doc_text.find(marker)
    if pos < 0:
        raise EvidenceError(f"case id not found in document: {case_id}")
    next_case = doc_text.find("\n## Case:", pos + len(marker))
    end = next_case if next_case >= 0 else len(doc_text)
    block = doc_text[pos:end]
    if block.count(old) < count:
        raise EvidenceError(f"mutation target not found in {case_id}: {old!r}")
    mutated_block = block.replace(old, new, count)
    return doc_text[:pos] + mutated_block + doc_text[end:]


# --- PLAN-1059 second wave: composite-family candidate staging & controls ---

BASE_SUITE_REL = "ace-document-base.suite.ad"
COMPOSITE_DOC_REL = "ace-m2/documents/composite.case.ad"
OLD_COMPOSITE_INCLUDE = "ace-m1/legacy-composite.case.ad"
NEW_COMPOSITE_INCLUDE = COMPOSITE_DOC_REL


def stage_candidate_base(repo_root: Path, vid_dir: Path) -> tuple[Path, Path]:
    """Stage a candidate BASE whose composite rows consume the NEW document.

    The copy always ends in the post-switch include layout whether or not the
    repository's active suite has been switched yet; the frozen legacy file
    stays inside the copied closure (NC-15 proves no fallback to it).
    """
    cases_in, output_dir, _ = stage_isolated_corpus(repo_root, vid_dir, include_base_closure=True)
    suite_p = cases_in / BASE_SUITE_REL
    text = suite_p.read_text(encoding="utf-8")
    if f'"{OLD_COMPOSITE_INCLUDE}"' in text:
        text = text.replace(f'"{OLD_COMPOSITE_INCLUDE}"', f'"{NEW_COMPOSITE_INCLUDE}"')
        suite_p.write_text(text, encoding="utf-8", newline="\n")
    elif f'"{NEW_COMPOSITE_INCLUDE}"' not in text:
        raise EvidenceError("candidate BASE suite carries no composite include line")
    return cases_in, output_dir


def _assert_report_shape(
    rep: dict[str, Any],
    vid: str,
    results: list[dict[str, Any]],
    fail_case: str,
    fail_check: str | None = None,
    fail_substr: str | None = None,
    expect_pass: int = 96,
    expect_fail: int = 1,
    expect_total: int = 97,
) -> None:
    fails = [x for x in rep["results"] if x["status"] != "PASS"]
    passes = [x for x in rep["results"] if x["status"] == "PASS"]
    ok = len(rep["results"]) == expect_total and len(passes) == expect_pass and len(fails) == expect_fail
    if ok:
        for f in fails:
            if f["case"] != fail_case or (fail_check is not None and f["check"] != fail_check):
                ok = False
            if fail_substr is not None:
                m_texts = [m if isinstance(m, str) else json.dumps(m) for m in (f.get("mismatches") or [])]
                if not any(fail_substr in mt for mt in m_texts):
                    ok = False
    results.append(
        {
            "vid": vid,
            "name": f"{vid}-assert",
            "ok": ok,
            "reason": f"fails={fails}" if not ok else "",
        }
    )


def cmd_controls_composite(args: argparse.Namespace) -> int:
    repo_root = repo_root_from_script()
    runner = Path(args.runner).resolve()
    if not runner.is_file():
        raise EvidenceError(f"missing --runner binary: {runner}")
    out_dir = ensure_fresh_empty_dir(Path(args.out))
    env, _tools = worker_env(repo_root)
    results: list[dict[str, Any]] = []

    def _step(
        vid: str,
        name: str,
        cmd: list[str],
        cwd: Path,
        expect_rc: int | None = None,
        expect_fail_sub: str | None = None,
        mutated_file: Path | None = None,
    ) -> dict[str, Any]:
        proc = run_cli(cmd, cwd=cwd, env=env)
        stdout = proc.stdout.decode("utf-8", errors="replace")
        stderr = proc.stderr.decode("utf-8", errors="replace")
        ok = True
        reason = ""
        if expect_rc is not None and proc.returncode != expect_rc:
            ok = False
            reason = f"expected exit {expect_rc}, got {proc.returncode}"
        if ok and expect_fail_sub and expect_fail_sub not in (stdout + "\n" + stderr):
            ok = False
            reason = f"missing expected diagnostic substring {expect_fail_sub!r}"
        entry = {
            "vid": vid,
            "name": name,
            "command": cmd,
            "exit_code": proc.returncode,
            "ok": ok,
            "reason": reason,
            "mutated_file_sha256": sha256_file(mutated_file)[1] if mutated_file and mutated_file.is_file() else None,
            "stdout_tail": stdout[-600:],
            "stderr_tail": stderr[-600:],
        }
        results.append(entry)
        return entry

    def _mutate_doc(cases_in: Path, case_id: str, old: str, new: str, count: int = 1) -> Path:
        doc_p = cases_in / COMPOSITE_DOC_REL
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), case_id, old, new, count),
            encoding="utf-8",
            newline="\n",
        )
        return doc_p

    def _run_suite(suite_path: Path, cout: Path, vid: str, name: str, expect_rc: int) -> dict[str, Any]:
        rep_p = cout / "report.json"
        _step(
            vid,
            name,
            [
                str(runner),
                "run",
                "--suite",
                str(suite_path),
                "--out",
                str(cout),
                "--json-out",
                str(rep_p),
            ],
            repo_root,
            expect_rc=expect_rc,
        )
        if not rep_p.is_file():
            raise EvidenceError(f"{vid}: runner produced no report.json under {cout}")
        return json.loads(rep_p.read_text(encoding="utf-8"))

    def _run_pair_subprocess(vid: str, name: str, new_suite: Path, pair_out: Path) -> dict[str, Any]:
        return _step(
            vid,
            name,
            [
                sys.executable,
                __file__,
                "pair",
                "--runner",
                str(runner),
                "--old-suite",
                str(repo_root / "crates/auto-lang/test/cases/ace-m1/ace-m1-base.suite.ad"),
                "--new-suite",
                str(new_suite),
                "--family",
                "composite",
                "--out",
                str(pair_out),
            ],
            repo_root,
        )

    # Stage shape gate: the candidate BASE must show 97 checks with the 28
    # target rows in document format before any mutation is applied.
    vdir = out_dir / "candidate-shape"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    shape_proj = run_migration_inventory(runner, cases_in / BASE_SUITE_REL, repo_root, env)
    shape_rows = shape_proj.get("checks") or []
    doc_keys = {
        (r["semantic"]["case"]["id"], r["semantic"]["check"]["id"])
        for r in shape_rows
        if (r.get("origin") or {}).get("format") == "document"
    }
    migrated_set = set(COMPOSITE_MIGRATED_PAIRS)
    scalar_set = set(SCALAR_MIGRATED_PAIRS)
    shape_ok = (
        len(shape_rows) == 97
        and len(doc_keys) == 68
        and migrated_set <= doc_keys
        and scalar_set <= doc_keys
    )
    results.append(
        {
            "vid": "candidate-shape",
            "name": "candidate-shape-97-68-28",
            "ok": shape_ok,
            "reason": f"rows={len(shape_rows)} document={len(doc_keys)}",
        }
    )

    # NC-01: only R02 VM result 9 -> 42; exactly that check FAILs with actual 9.
    vdir = out_dir / "NC-01-r02-vm-9-to-42"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(cases_in, "ace.m1.r02", "```text\n9\n```", "```text\n42\n```")
    _step("NC-01", "NC-01-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=0, mutated_file=doc_p)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-01", "NC-01-run", expect_rc=1)
    _assert_report_shape(rep, "NC-01", results, "ace.m1.r02", "autovm", fail_substr="9")

    # NC-02: only the R06 Rust golden's 10 -> 11 (semicolon anchors the Rust
    # fence; the Auto source keeps the original expression); actual emission
    # stays the original bytes, so only the a2r-snapshot comparison reddens.
    vdir = out_dir / "NC-02-r06-rust-10-to-11"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(cases_in, "ace.m1.r06", "p.left * 10 + p.right;", "p.left * 11 + p.right;")
    _step("NC-02", "NC-02-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=0, mutated_file=doc_p)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-02", "NC-02-run", expect_rc=1)
    _assert_report_shape(rep, "NC-02", results, "ace.m1.r06", "a2r-snapshot", fail_substr="m1_r06.rs")

    # NC-03: K06 Auto initial value 1 -> 2 with both expectations kept; the
    # captured state changes BOTH legs of that case (VM 44 vs 42, Rust emit
    # initial-value bytes) — exactly 2 FAILs, 95 PASS.
    vdir = out_dir / "NC-03-k06-auto-1-to-2"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(cases_in, "ace.m1.k06", "var base int = 1", "var base int = 2")
    _step("NC-03", "NC-03-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=0, mutated_file=doc_p)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-03", "NC-03-run", expect_rc=1)
    fails = [x for x in rep["results"] if x["status"] != "PASS"]
    passes = [x for x in rep["results"] if x["status"] == "PASS"]
    k06_axes = sorted(f"{f['case']}:{f['check']}" for f in fails)
    m_texts = [
        m if isinstance(m, str) else json.dumps(m)
        for f in fails
        for m in (f.get("mismatches") or [])
    ]
    results.append(
        {
            "vid": "NC-03",
            "name": "NC-03-assert",
            "ok": (
                len(passes) == 95
                and len(fails) == 2
                and k06_axes == ["ace.m1.k06:a2r-snapshot", "ace.m1.k06:autovm"]
                and any("44" in mt for mt in m_texts)
            ),
            "reason": f"fails={k06_axes}",
        }
    )

    # NC-04: first T05 Rust "A\0B" escaped to "A\\0B" (Auto untouched): only
    # the a2r-snapshot byte comparison reddens; VM keeps 42.
    vdir = out_dir / "NC-04-t05-rust-escape"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(
        cases_in,
        "ace.m1.t05",
        'let a: String = "A\\0B".to_string();',
        'let a: String = "A\\\\0B".to_string();',
    )
    _step("NC-04", "NC-04-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=0, mutated_file=doc_p)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-04", "NC-04-run", expect_rc=1)
    _assert_report_shape(rep, "NC-04", results, "ace.m1.t05", "a2r-snapshot", fail_substr="m1_t05.rs")

    # NC-05: V07's whole VM result section is deleted (Rust kept): the fixed
    # 97-pair contract rejects the 96-green subset, naming the missing check.
    vdir = out_dir / "NC-05-drop-v07-vm"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(
        cases_in,
        "ace.m1.v07",
        '### VM result\ncheck-id: "autovm"\nfinal-newline: false\n\n```text\n42\n```\n\n### Rust',
        "### Rust",
    )
    pair_out = vdir / "pair-out"
    _run_pair_subprocess("NC-05", "NC-05-pair", cases_in / BASE_SUITE_REL, pair_out)
    last = results[-1]
    last["ok"] = last["exit_code"] == 1 and "ace.m1.v07" in (last["stdout_tail"] + last["stderr_tail"]) and "autovm" in (last["stdout_tail"] + last["stderr_tail"])
    if not last["ok"]:
        last["reason"] = f"exit={last['exit_code']} tails={last['stdout_tail'][-200:]}|{last['stderr_tail'][-200:]}"

    # NC-06: duplicating the whole K01 case must be rejected at load with the
    # duplicate identity named; a clean re-stage keeps the 97-row contract.
    vdir = out_dir / "NC-06-duplicate-k01"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    doc_p = cases_in / COMPOSITE_DOC_REL
    text = doc_p.read_text(encoding="utf-8")
    start = text.find("\n## Case: k01-move-closure-by-value")
    nxt = text.find("\n## Case: k03-shared-closure-capture", start)
    if start < 0 or nxt < 0:
        raise EvidenceError("NC-06: K01/K03 case block boundaries not found")
    doc_p.write_text(text + text[start:nxt] + "\n", encoding="utf-8", newline="\n")
    lint = _step("NC-06", "NC-06-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=None, mutated_file=doc_p)
    lint_out = lint["stdout_tail"] + lint["stderr_tail"]
    dup_named = lint["exit_code"] != 0 and ("ace.m1.k01" in lint_out or "duplicate" in lint_out.lower())
    results.append({"vid": "NC-06", "name": "NC-06-duplicate-named", "ok": dup_named, "reason": f"exit={lint['exit_code']}"})
    vdir_clean = out_dir / "NC-06-clean-restage"
    cases_clean, _c = stage_candidate_base(repo_root, vdir_clean)
    clean_proj = run_migration_inventory(runner, cases_clean / BASE_SUITE_REL, repo_root, env)
    results.append(
        {
            "vid": "NC-06",
            "name": "NC-06-clean-restage-97",
            "ok": len(clean_proj.get("checks") or []) == 97,
            "reason": f"rows={len(clean_proj.get('checks') or [])}",
        }
    )

    # NC-07: file-level links removed: pair rejects with the historical link
    # and the affected cases named.
    vdir = out_dir / "NC-07-links-removed"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    doc_p = cases_in / COMPOSITE_DOC_REL
    doc_p.write_text(
        doc_p.read_text(encoding="utf-8").replace('links: ["PLAN-852"]', "links: []"),
        encoding="utf-8",
        newline="\n",
    )
    pair_out = vdir / "pair-out"
    _run_pair_subprocess("NC-07", "NC-07-pair", cases_in / BASE_SUITE_REL, pair_out)
    last = results[-1]
    joined = last["stdout_tail"] + last["stderr_tail"]
    last["ok"] = last["exit_code"] == 1 and "PLAN-852" in joined and "ace.m1.k01" in joined
    if not last["ok"]:
        last["reason"] = f"exit={last['exit_code']} tails={joined[-300:]}"

    # NC-08: K06 Auto logical path main.at -> other.at: pair rejects on the
    # source path axis; the main.at version passes (candidate-shape gate).
    vdir = out_dir / "NC-08-k06-path"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(cases_in, "ace.m1.k06", 'path: "main.at"', 'path: "other.at"')
    pair_out = vdir / "pair-out"
    _run_pair_subprocess("NC-08", "NC-08-pair", cases_in / BASE_SUITE_REL, pair_out)
    last = results[-1]
    joined = last["stdout_tail"] + last["stderr_tail"]
    last["ok"] = last["exit_code"] == 1 and "virtual_path" in joined
    if not last["ok"]:
        last["reason"] = f"exit={last['exit_code']} tails={joined[-300:]}"

    # NC-09: K06 module-name drift: pair rejects on module identity / artifact
    # path; the default naming must never silently absorb the change.
    vdir = out_dir / "NC-09-k06-module"
    cases_in, _cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(cases_in, "ace.m1.k06", 'module-name: "m1_k06"', 'module-name: "m1_k06_changed"')
    pair_out = vdir / "pair-out"
    _run_pair_subprocess("NC-09", "NC-09-pair", cases_in / BASE_SUITE_REL, pair_out)
    last = results[-1]
    joined = last["stdout_tail"] + last["stderr_tail"]
    last["ok"] = last["exit_code"] == 1 and ("module_name" in joined or "m1_k06_changed" in joined)
    if not last["ok"]:
        last["reason"] = f"exit={last['exit_code']} tails={joined[-300:]}"

    # NC-10: R02's final-newline: false dropped: the expectation becomes
    # "9\n" while the real VM still prints "9" — exactly that row reddens
    # with an explicit text difference.
    vdir = out_dir / "NC-10-r02-final-newline"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = _mutate_doc(
        cases_in,
        "ace.m1.r02",
        'check-id: "autovm"\nfinal-newline: false\n',
        'check-id: "autovm"\n',
    )
    _step("NC-10", "NC-10-lint", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=0, mutated_file=doc_p)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-10", "NC-10-run", expect_rc=1)
    _assert_report_shape(rep, "NC-10", results, "ace.m1.r02", "autovm", fail_substr="9")

    # NC-11/NC-12 mutate COPIES of the real pair records (and transiently the
    # scope actuals, restored afterwards); they never fabricate product runs.
    pair_ref_dir = out_dir.parent / "pair-candidate"
    old_rec_p = pair_ref_dir / "old-record.json"
    new_rec_p = pair_ref_dir / "new-record.json"
    if not (old_rec_p.is_file() and new_rec_p.is_file()):
        results.append(
            {
                "vid": "NC-11",
                "name": "NC-11-records-missing",
                "ok": False,
                "reason": f"pair records not found under {pair_ref_dir}; run pair --family composite first",
            }
        )
        results.append(
            {
                "vid": "NC-12",
                "name": "NC-12-records-missing",
                "ok": False,
                "reason": f"pair records not found under {pair_ref_dir}; run pair --family composite first",
            }
        )
    else:
        # NC-11: swap the R02/R06 VM actual file contents (9 <-> 21) with the
        # cached digests synced and PASS labels kept; the rebound offline
        # verify must still reject on actual vs expected.
        vdir = out_dir / "NC-11-swap-actuals"
        vdir.mkdir(parents=True, exist_ok=True)
        new_clone = json.loads(new_rec_p.read_text(encoding="utf-8"))
        r02_files = new_clone["actuals"]["ace.m1.r02::autovm"]["files"]
        r06_files = new_clone["actuals"]["ace.m1.r06::autovm"]["files"]
        r02_path, r06_path = Path(r02_files["result"]["path"]), Path(r06_files["result"]["path"])
        r02_orig, r06_orig = r02_path.read_bytes(), r06_path.read_bytes()
        if r02_orig != b"9" or r06_orig != b"21":
            raise EvidenceError(f"NC-11: unexpected baseline actuals {r02_orig!r}/{r06_orig!r}")
        r02_path.write_bytes(r06_orig)
        r06_path.write_bytes(r02_orig)
        try:
            for key, swapped in (("ace.m1.r02::autovm", r06_orig), ("ace.m1.r06::autovm", r02_orig)):
                obs = new_clone["actuals"][key]
                obs["avm"]["result_text"] = swapped.decode("utf-8")
                obs["files"]["result"]["bytes"] = len(swapped)
                obs["files"]["result"]["sha256"] = sha256_bytes(swapped)
            old_c = vdir / "old-record.json"
            new_c = vdir / "new-record.json"
            old_c.write_text(old_rec_p.read_text(encoding="utf-8"), encoding="utf-8")
            new_c.write_text(json.dumps(new_clone, indent=2, ensure_ascii=False), encoding="utf-8")
            proc = run_cli(
                [sys.executable, __file__, "verify", "--old-record", str(old_c), "--new-record", str(new_c), "--family", "composite"],
                cwd=repo_root,
                env=env,
            )
            joined = proc.stdout.decode("utf-8", errors="replace") + proc.stderr.decode("utf-8", errors="replace")
            results.append(
                {
                    "vid": "NC-11",
                    "name": "NC-11-verify-exit1",
                    "ok": proc.returncode == 1 and "ace.m1.r02" in joined and "autovm" in joined,
                    "reason": joined[-300:],
                }
            )
        finally:
            r02_path.write_bytes(r02_orig)
            r06_path.write_bytes(r06_orig)

        # NC-12: the R05 Rust actual file disappears while its path and digest
        # stay in the record: verify must exit 2 naming the missing actual.
        vdir = out_dir / "NC-12-missing-actual"
        vdir.mkdir(parents=True, exist_ok=True)
        new_clone = json.loads(new_rec_p.read_text(encoding="utf-8"))
        art_meta = new_clone["actuals"]["ace.m1.r05::a2r-snapshot"]["files"]["artifact:m1_r05.rs"]
        art_path = Path(art_meta["path"])
        backup = vdir / "m1_r05.rs.backup"
        shutil.copyfile(art_path, backup)
        art_path.unlink()
        try:
            old_c = vdir / "old-record.json"
            new_c = vdir / "new-record.json"
            old_c.write_text(old_rec_p.read_text(encoding="utf-8"), encoding="utf-8")
            new_c.write_text(json.dumps(new_clone, indent=2, ensure_ascii=False), encoding="utf-8")
            proc = run_cli(
                [sys.executable, __file__, "verify", "--old-record", str(old_c), "--new-record", str(new_c), "--family", "composite"],
                cwd=repo_root,
                env=env,
            )
            joined = proc.stdout.decode("utf-8", errors="replace") + proc.stderr.decode("utf-8", errors="replace")
            results.append(
                {
                    "vid": "NC-12",
                    "name": "NC-12-verify-exit2",
                    "ok": proc.returncode == 2 and "ace.m1.r05" in joined and "artifact" in joined,
                    "reason": joined[-300:],
                }
            )
        finally:
            shutil.copyfile(backup, art_path)

    # NC-13: stale axis on the real R02/a2r-snapshot row: Fresh first, then a
    # single valid byte in that inline golden turns exactly that row Stale on
    # the oracle content axis.
    vdir = out_dir / "NC-13-stale"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    rep = _run_suite(cases_in / BASE_SUITE_REL, cout, "NC-13", "NC-13-baseline-run", expect_rc=0)
    r02_rows = [r for r in rep["results"] if r.get("case") == "ace.m1.r02" and r.get("check") == "a2r-snapshot"]
    if len(r02_rows) != 1:
        raise EvidenceError(f"NC-13 expected 1 R02 a2r-snapshot row, found {len(r02_rows)}")
    scoped = vdir / "r02-a2r-scoped-report.json"
    scoped.write_text(json.dumps({"schema": rep.get("schema"), "suite": rep.get("suite"), "suite_sha256": rep.get("suite_sha256"), "results": r02_rows}, indent=2), encoding="utf-8")
    suite_p = cases_in / BASE_SUITE_REL
    s1 = run_cli([str(runner), "stale", "--suite", str(suite_p), "--report", str(scoped)], cwd=repo_root, env=env)
    s1_out = s1.stdout.decode("utf-8", errors="replace")
    results.append(
        {
            "vid": "NC-13",
            "name": "NC-13-r02-fresh",
            "ok": s1.returncode == 0 and "FRESH ace.m1.r02:a2r-snapshot" in s1_out,
            "reason": s1_out[-400:],
        }
    )
    doc_p = _mutate_doc(cases_in, "ace.m1.r02", "p.inner.value = 7;", "p.inner.value = 8;")
    s2 = run_cli([str(runner), "stale", "--suite", str(suite_p), "--report", str(scoped)], cwd=repo_root, env=env)
    s2_out = s2.stdout.decode("utf-8", errors="replace")
    results.append(
        {
            "vid": "NC-13",
            "name": "NC-13-r02-stale-after-golden-edit",
            "ok": (
                s2.returncode == 1
                and "STALE ace.m1.r02:a2r-snapshot" in s2_out
                and "oracle" in s2_out
                and s2_out.count("STALE ") == 1
            ),
            "reason": s2_out[-400:],
            "mutated_file_sha256": sha256_file(doc_p)[1],
        }
    )

    # NC-14 (legal positive control): the composite document alone with a
    # single-file suite — no ace-m1 sidecar anywhere — really executes 14
    # cases / 28 checks, every actual bound to the inline expectations.
    vdir = out_dir / "NC-14-standalone"
    cases_in = vdir / "inputs/cases"
    cout = vdir / "output"
    (cases_in / "ace-m2/documents").mkdir(parents=True, exist_ok=True)
    cout.mkdir(parents=True, exist_ok=True)
    _copy_with_hashes(
        repo_root / "crates/auto-lang/test/cases" / COMPOSITE_DOC_REL,
        cases_in / COMPOSITE_DOC_REL,
        [],
    )
    suite_p = cases_in / "composite-standalone-1059.suite.ad"
    suite_p.write_text(
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"ace-composite-standalone-1059\", include: [\"ace-m2/documents/composite.case.ad\"] }\n```\n",
        encoding="utf-8",
        newline="\n",
    )
    _step("NC-14", "NC-14-lint", [str(runner), "lint", "--suite", str(suite_p)], repo_root, expect_rc=0)
    rep = _run_suite(suite_p, cout, "NC-14", "NC-14-run", expect_rc=0)
    nc14_proj = run_migration_inventory(runner, suite_p, repo_root, env)
    nc14_rows = nc14_proj.get("checks") or []
    run_dir = find_unique_run_dir(cout)
    nc14_ok = len(rep["results"]) == 28 and all(r["status"] == "PASS" for r in rep["results"]) and len(nc14_rows) == 28
    per_item: list[str] = []
    for row in nc14_rows:
        sem = row["semantic"]
        origin = row["origin"]
        key = (sem["case"]["id"], sem["check"]["id"])
        if origin.get("format") != "document" or origin.get("input_paths") != []:
            nc14_ok = False
            per_item.append(f"{key}:origin")
            continue
        check_dir = run_dir / origin["case_dir"] / origin["check_dir"]
        if sem["check"]["adapter"] == "avm":
            res_file = check_dir / "result.txt"
            want = (sem["expect"].get("result") or {}).get("text")
            if not res_file.is_file() or res_file.read_bytes().decode("utf-8") != want:
                nc14_ok = False
                per_item.append(f"{key}:result")
        elif sem["check"]["adapter"] == "a2r":
            for art in sem["expect"].get("artifacts") or []:
                rep_row = next(r for r in rep["results"] if r["case"] == key[0] and r["check"] == key[1])
                stored = next((a for a in (rep_row.get("artifacts") or []) if a["path"] == art["path"]), None)
                if stored is None:
                    nc14_ok = False
                    per_item.append(f"{key}:{art['path']}:missing-row")
                    continue
                spath = Path(stored["stored"])
                _ensure_under(run_dir, spath, f"NC-14 {key}:{art['path']}")
                sz, dig = sha256_file(spath)
                if sz != art.get("bytes") or dig != art.get("sha256"):
                    nc14_ok = False
                    per_item.append(f"{key}:{art['path']}:digest")
    vm_texts = sorted(
        {
            (row["semantic"]["case"]["id"], (row["semantic"]["expect"].get("result") or {}).get("text"))
            for row in nc14_rows
            if row["semantic"]["check"]["adapter"] == "avm"
        },
        key=lambda t: t[0],
    )
    want_vm = {c: ("9" if c == "ace.m1.r02" else "21" if c == "ace.m1.r06" else "42") for c, _ in COMPOSITE_MIGRATED_PAIRS}
    if {c: t for c, t in vm_texts} != want_vm:
        nc14_ok = False
        per_item.append(f"vm-texts={vm_texts}")
    results.append(
        {
            "vid": "NC-14",
            "name": "NC-14-standalone-28-green",
            "ok": nc14_ok,
            "reason": f"rows={len(rep['results'])} per_item_failures={per_item[:10]}",
        }
    )

    # NC-15: the composite file vanishes from the full candidate closure: the
    # load must fail naming the missing include (no fallback to the still
    # present frozen legacy file); restoring it brings BASE97 green again.
    vdir = out_dir / "NC-15-missing-composite"
    cases_in, cout = stage_candidate_base(repo_root, vdir)
    doc_p = cases_in / COMPOSITE_DOC_REL
    doc_backup = vdir / "composite.case.ad.backup"
    shutil.copyfile(doc_p, doc_backup)
    doc_p.unlink()
    _step("NC-15", "NC-15-lint-missing", [str(runner), "lint", "--suite", str(cases_in / BASE_SUITE_REL)], repo_root, expect_rc=None)
    last = results[-1]
    lint_out = last["stdout_tail"] + last["stderr_tail"]
    last["ok"] = last["exit_code"] != 0 and "composite.case.ad" in lint_out
    if not last["ok"]:
        last["reason"] = f"exit={last['exit_code']} tails={lint_out[-300:]}"
    run_p = run_cli(
        [str(runner), "run", "--suite", str(cases_in / BASE_SUITE_REL), "--out", str(cout), "--json-out", str(cout / "report.json")],
        cwd=repo_root,
        env=env,
    )
    run_out = run_p.stdout.decode("utf-8", errors="replace") + run_p.stderr.decode("utf-8", errors="replace")
    results.append(
        {
            "vid": "NC-15",
            "name": "NC-15-run-names-missing-include",
            "ok": run_p.returncode != 0 and "composite.case.ad" in run_out,
            "reason": run_out[-300:],
        }
    )
    shutil.copyfile(doc_backup, doc_p)
    (cout / "restored").mkdir(parents=True, exist_ok=True)
    restored = run_cli(
        [str(runner), "run", "--suite", str(cases_in / BASE_SUITE_REL), "--out", str(cout / "restored"), "--json-out", str(cout / "restored" / "report.json")],
        cwd=repo_root,
        env=env,
    )
    restored_green = False
    rep_path = cout / "restored" / "report.json"
    if restored.returncode == 0 and rep_path.is_file():
        rep = json.loads(rep_path.read_text(encoding="utf-8"))
        restored_green = len(rep["results"]) == 97 and all(r["status"] == "PASS" for r in rep["results"])
    results.append(
        {
            "vid": "NC-15",
            "name": "NC-15-restored-base97-green",
            "ok": restored_green,
            "reason": f"exit={restored.returncode}",
        }
    )

    all_ok = all(r["ok"] for r in results)
    summary = {"passed": all_ok, "total": len(results), "failed": [r for r in results if not r["ok"]], "results": results}
    (out_dir / "controls-summary.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(json.dumps({"passed": all_ok, "total": len(results), "failed_names": [r["name"] for r in results if not r["ok"]]}, ensure_ascii=False))
    return 0 if all_ok else 1


def cmd_controls(args: argparse.Namespace) -> int:
    repo_root = repo_root_from_script()
    runner = Path(args.runner).resolve()
    if not runner.is_file():
        raise EvidenceError(f"missing --runner binary: {runner}")
    out_dir = ensure_fresh_empty_dir(Path(args.out))
    env, _tools = worker_env(repo_root)
    results: list[dict[str, Any]] = []

    def _run_step(
        vid: str,
        name: str,
        cmd: list[str],
        cwd: Path,
        expect_rc: int | None = None,
        expect_fail_sub: str | None = None,
        mutated_file: Path | None = None,
    ) -> dict[str, Any]:
        proc = run_cli(cmd, cwd=cwd, env=env)
        stdout = proc.stdout.decode("utf-8", errors="replace")
        stderr = proc.stderr.decode("utf-8", errors="replace")
        ok = True
        reason = ""
        if expect_rc is not None and proc.returncode != expect_rc:
            ok = False
            reason = f"expected exit {expect_rc}, got {proc.returncode}"
        if ok and expect_fail_sub and expect_fail_sub not in (stdout + "\n" + stderr):
            ok = False
            reason = f"missing expected diagnostic substring {expect_fail_sub!r}"
        entry = {
            "vid": vid,
            "name": name,
            "command": cmd,
            "exit_code": proc.returncode,
            "ok": ok,
            "reason": reason,
            "mutated_file_sha256": sha256_file(mutated_file)[1] if mutated_file and mutated_file.is_file() else None,
            "stdout_tail": stdout[-600:],
            "stderr_tail": stderr[-600:],
        }
        results.append(entry)
        return entry

    # V-01: TOP -> program; O12 caps -> both caps (projection catches semantic drift)
    for tag, rel_doc, rel_suite, fam, cid, old_s, new_s, expected_diff_sub in (
        (
            "V-01-top-program",
            "ace-m2/documents/immutable-assignment.case.ad",
            "ace-document-immutable.suite.ad",
            "immutable",
            "ace.m1.ia920.top",
            'entry-mode: "script"',
            'entry-mode: "program"',
            "entry_mode",
        ),
        (
            "V-01-o12-caps",
            "ace-m2/documents/console.case.ad",
            "ace-document-console.suite.ad",
            "console",
            "ace.m1.o12",
            "caps: []",
            'caps: ["console.stdout", "console.stderr"]',
            "caps",
        ),
    ):
        vdir = out_dir / tag
        cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir, include_base_closure=False)
        doc_p = cases_in / rel_doc
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), cid, old_s, new_s),
            encoding="utf-8",
            newline="\n",
        )
        lint = _run_step("V-01", f"{tag}-lint", [str(runner), "lint", "--suite", str(cases_in / rel_suite)], repo_root, expect_rc=0, mutated_file=doc_p)
        if lint["ok"]:
            ref_proj = run_migration_inventory(runner, repo_root / "crates/auto-lang/test/cases" / rel_suite, repo_root, env)
            mut_proj = run_migration_inventory(runner, cases_in / rel_suite, repo_root, env)
            caught = False
            msg = ""
            try:
                ref_idx = index_projection(ref_proj, fam, "ref", True)
                mut_idx = index_projection(mut_proj, fam, "mut", True)
                compare_projections(fam, ref_idx, mut_idx)
            except ParityError as exc:
                caught = expected_diff_sub in str(exc)
                msg = str(exc)
            results.append({"vid": "V-01", "name": f"{tag}-compare", "ok": caught, "reason": msg})

    # V-03: BASE isolation controls (B02 VM -21 -> -20; B02 Rust 1-byte edit)
    for tag, old_s, new_s, fail_chk, fail_obs in (
        ("V-03-b02-vm", "```text\n-21\n```", "```text\n-20\n```", "autovm", "result"),
        ("V-03-b02-rust", "return -7 / 3 * 10 + -7 % 3;", "return -7 / 3 * 10 + -7 * 3;", "a2r-snapshot", "m1_b02.rs"),
    ):
        vdir = out_dir / tag
        cases_in, cout, _ = stage_isolated_corpus(repo_root, vdir, include_base_closure=True)
        doc_p = cases_in / "ace-m2/documents/scalar.case.ad"
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), "ace.m1.b02", old_s, new_s),
            encoding="utf-8",
            newline="\n",
        )
        _run_step("V-03", f"{tag}-lint", [str(runner), "lint", "--suite", str(cases_in / "ace-document-base.suite.ad")], repo_root, expect_rc=0, mutated_file=doc_p)
        rep_p = cout / "report.json"
        r = _run_step(
            "V-03",
            f"{tag}-run",
            [str(runner), "run", "--suite", str(cases_in / "ace-document-base.suite.ad"), "--out", str(cout), "--json-out", str(rep_p)],
            repo_root,
            expect_rc=1,
            mutated_file=doc_p,
        )
        if rep_p.is_file():
            rep = json.loads(rep_p.read_text(encoding="utf-8"))
            fails = [x for x in rep["results"] if x["status"] != "PASS"]
            passes = [x for x in rep["results"] if x["status"] == "PASS"]
            m_texts = [m if isinstance(m, str) else json.dumps(m) for m in (fails[0].get("mismatches") or [])] if fails else []
            ok = (
                len(passes) == 96
                and len(fails) == 1
                and fails[0]["status"] == "FAIL"
                and fails[0]["case"] == "ace.m1.b02"
                and fails[0]["check"] == fail_chk
                and any(fail_obs in mt for mt in m_texts)
            )
            results.append({"vid": "V-03", "name": f"{tag}-assert", "ok": ok, "reason": f"fails={fails}"})

    # V-04: Console stream byte sensitivity controls (O03 trim trailing LF, O04 😀->😁, O06 swap stdout/stderr)
    v04_specs = [
        ("V-04-o03-lf", "ace.m1.o03", "#### stdout\n\n```text\n你好\n\n```", "#### stdout\n\n```text\n你好\n```", "stdout"),
        ("V-04-o04-emoji", "ace.m1.o04", "#### stderr\n\n```text\n😀\n```", "#### stderr\n\n```text\n😁\n```", "stderr"),
        ("V-04-o06-swap", "ace.m1.o06", "#### stdout\n\n```text\n9\n3\n```\n\n#### stderr\n\n```text\n8\n```", "#### stdout\n\n```text\n8\n```\n\n#### stderr\n\n```text\n9\n3\n```", "stdout"),
    ]
    for tag, cid, old_s, new_s, expected_stream in v04_specs:
        vdir = out_dir / tag
        cases_in, cout, _ = stage_isolated_corpus(repo_root, vdir, include_base_closure=False)
        doc_p = cases_in / "ace-m2/documents/console.case.ad"
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), cid, old_s, new_s),
            encoding="utf-8",
            newline="\n",
        )
        rep_p = cout / "report.json"
        _run_step(
            "V-04",
            f"{tag}-run",
            [str(runner), "run", "--suite", str(cases_in / "ace-document-console.suite.ad"), "--out", str(cout), "--json-out", str(rep_p)],
            repo_root,
            expect_rc=1,
            mutated_file=doc_p,
        )
        rep = json.loads(rep_p.read_text(encoding="utf-8"))
        fails = [x for x in rep["results"] if x["status"] != "PASS"]
        passes = [x for x in rep["results"] if x["status"] == "PASS"]
        m_texts = [m if isinstance(m, str) else json.dumps(m) for m in (fails[0].get("mismatches") or [])] if fails else []
        ok = (
            len(passes) == 14
            and len(fails) == 1
            and fails[0]["status"] == "FAIL"
            and fails[0]["case"] == cid
            and fails[0]["check"] == "av-console"
            and any(expected_stream in mt for mt in m_texts)
        )
        results.append({"vid": "V-04", "name": f"{tag}-assert", "ok": ok, "reason": f"fails={fails}"})

    # V-05: Authoring rejection combos (unit+exit0, bogus channel, missing stderr, O10 missing stage or code)
    for tag, cid, old_s, new_s, exp_sub in (
        ("V-05-unit-exit", "ace.m1.o07", 'channel: "unit"', 'channel: "unit"\nexit: 0', "unit has no exit payload"),
        ("V-05-bogus-channel", "ace.m1.o07", 'channel: "unit"', 'channel: "bogus"', "channel"),
        ("V-05-o07-missing-stderr", "ace.m1.o07", "#### stderr\n\n```text\nunit\n```", "", "explicit stdout and stderr"),
        ("V-05-o10-missing-stage", "ace.m1.o10", 'stage: "run"\ncode: "av.trap.div-by-zero"', 'code: "av.trap.div-by-zero"', "stage"),
        ("V-05-o10-missing-code", "ace.m1.o10", 'stage: "run"\ncode: "av.trap.div-by-zero"', 'stage: "run"', "code"),
    ):
        vdir = out_dir / tag
        cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir, include_base_closure=False)
        doc_p = cases_in / "ace-m2/documents/console.case.ad"
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), cid, old_s, new_s),
            encoding="utf-8",
            newline="\n",
        )
        _run_step(
            "V-05",
            tag,
            [str(runner), "lint", "--suite", str(cases_in / "ace-document-console.suite.ad")],
            repo_root,
            expect_rc=2,
            expect_fail_sub=exp_sub,
            mutated_file=doc_p,
        )

    # V-06: O08 delete channel: "value" (caught by comparator); O10 wrong trap code; O12 same-stage different code
    vdir_o08 = out_dir / "V-06-o08-drop-ch"
    cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir_o08, include_base_closure=False)
    doc_p = cases_in / "ace-m2/documents/console.case.ad"
    doc_p.write_text(
        replace_in_case_block(doc_p.read_text(encoding="utf-8"), "ace.m1.o08", 'channel: "value"\n', ""),
        encoding="utf-8",
        newline="\n",
    )
    ref_con = run_migration_inventory(runner, repo_root / "crates/auto-lang/test/cases/ace-document-console.suite.ad", repo_root, env)
    mut_con = run_migration_inventory(runner, cases_in / "ace-document-console.suite.ad", repo_root, env)
    caught_o08 = False
    try:
        compare_projections(
            "console",
            index_projection(ref_con, "console", "ref", True),
            index_projection(mut_con, "console", "mut", True),
        )
    except ParityError as exc:
        caught_o08 = "av_channel" in str(exc)
    results.append({"vid": "V-06", "name": "V-06-o08-drop-channel", "ok": caught_o08})

    for tag, cid, old_s, new_s in (
        ("V-06-o10-wrong-trap", "ace.m1.o10", 'code: "av.trap.div-by-zero"', 'code: "av.trap.wrong"'),
        ("V-06-o12-wrong-code", "ace.m1.o12", 'code: "host.unauthorized"', 'code: "syntax.unexpected-token"'),
    ):
        vdir = out_dir / tag
        cases_in, cout, _ = stage_isolated_corpus(repo_root, vdir, include_base_closure=False)
        doc_p = cases_in / "ace-m2/documents/console.case.ad"
        doc_p.write_text(
            replace_in_case_block(doc_p.read_text(encoding="utf-8"), cid, old_s, new_s),
            encoding="utf-8",
            newline="\n",
        )
        rep_p = cout / "report.json"
        _run_step("V-06", f"{tag}-run", [str(runner), "run", "--suite", str(cases_in / "ace-document-console.suite.ad"), "--out", str(cout), "--json-out", str(rep_p)], repo_root, expect_rc=1, mutated_file=doc_p)
        rep = json.loads(rep_p.read_text(encoding="utf-8"))
        fails = [x for x in rep["results"] if x["status"] != "PASS"]
        ok = len(fails) == 1 and fails[0]["case"] == cid and len(rep["results"]) == 15
        results.append({"vid": "V-06", "name": f"{tag}-assert", "ok": ok})

    # V-07: immutable wrong code on TOP; deleted NB18S case caught by fixed 5-check contract
    vdir_top = out_dir / "V-07-top-wrong-code"
    cases_in, cout, _ = stage_isolated_corpus(repo_root, vdir_top, include_base_closure=False)
    doc_p = cases_in / "ace-m2/documents/immutable-assignment.case.ad"
    doc_p.write_text(
        replace_in_case_block(doc_p.read_text(encoding="utf-8"), "ace.m1.ia920.top", 'code: "auto_name_E0203"', 'code: "auto_name_E9999"'),
        encoding="utf-8",
        newline="\n",
    )
    rep_p = cout / "report.json"
    _run_step("V-07", "V-07-top-wrong-code", [str(runner), "run", "--suite", str(cases_in / "ace-document-immutable.suite.ad"), "--out", str(cout), "--json-out", str(rep_p)], repo_root, expect_rc=1, mutated_file=doc_p)
    rep = json.loads(rep_p.read_text(encoding="utf-8"))
    fails = [x for x in rep["results"] if x["status"] != "PASS"]
    results.append({"vid": "V-07", "name": "V-07-top-assert", "ok": len(fails) == 1 and fails[0]["case"] == "ace.m1.ia920.top" and len(rep["results"]) == 5})

    vdir_nb = out_dir / "V-07-drop-nb18s"
    cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir_nb, include_base_closure=False)
    doc_p = cases_in / "ace-m2/documents/immutable-assignment.case.ad"
    text = doc_p.read_text(encoding="utf-8")
    cut = text.find("\n## Case: nb18s-shadowed-var")
    if cut < 0:
        raise EvidenceError("nb18s case heading not found")
    doc_p.write_text(text[:cut] + "\n", encoding="utf-8", newline="\n")
    mut_imm = run_migration_inventory(runner, cases_in / "ace-document-immutable.suite.ad", repo_root, env)
    caught_nb = False
    try:
        index_projection(mut_imm, "immutable", "drop-nb18s", True)
    except ParityError as exc:
        caught_nb = "ace.m1.ia920.nb18s" in str(exc)
    results.append({"vid": "V-07", "name": "V-07-drop-nb18s", "ok": caught_nb})

    # V-08: delete B07 check, omit console file, out-of-bounds suite include
    vdir_b07 = out_dir / "V-08-drop-b07-check"
    cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir_b07, include_base_closure=True)
    doc_p = cases_in / "ace-m2/documents/scalar.case.ad"
    text = doc_p.read_text(encoding="utf-8")
    doc_p.write_text(
        replace_in_case_block(text, "ace.m1.b07", "### Probe av\nrecipe: \"ac-av/core-run-v1\"\nentry: \"d_main\"\nexit: 30\n\n", ""),
        encoding="utf-8",
        newline="\n",
    )
    mut_base = run_migration_inventory(runner, cases_in / "ace-document-base.suite.ad", repo_root, env)
    caught_b07 = False
    try:
        index_projection(mut_base, "scalar", "drop-b07-av", True)
    except ParityError as exc:
        caught_b07 = "ace.m1.b07" in str(exc) and "av" in str(exc)
    results.append({"vid": "V-08", "name": "V-08-drop-b07-av", "ok": caught_b07})

    vdir_miss = out_dir / "V-08-missing-console-file"
    cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir_miss, include_base_closure=False)
    (cases_in / "ace-m2/documents/console.case.ad").unlink()
    _run_step("V-08", "V-08-missing-file-lint", [str(runner), "lint", "--suite", str(cases_in / "ace-document-console.suite.ad")], repo_root, expect_rc=2)

    vdir_esc = out_dir / "V-08-escape-include"
    cases_in, _cout, _ = stage_isolated_corpus(repo_root, vdir_esc, include_base_closure=False)
    sp = cases_in / "ace-document-console.suite.ad"
    sp.write_text(sp.read_text(encoding="utf-8").replace("ace-m2/documents/console.case.ad", "../../outside.case.ad"), encoding="utf-8", newline="\n")
    _run_step("V-08", "V-08-escape-lint", [str(runner), "lint", "--suite", str(sp)], repo_root, expect_rc=2)

    # V-11: B02 a2r-snapshot Fresh -> 1-byte Rust edit -> Stale (oracle axis) + missing actual -> exit 2
    vdir_v11 = out_dir / "V-11-stale"
    cases_in, cout, _ = stage_isolated_corpus(repo_root, vdir_v11, include_base_closure=True)
    suite_v11 = cases_in / "ace-document-base.suite.ad"
    full_rep = cout / "report.json"
    _run_step("V-11", "V-11-baseline-run", [str(runner), "run", "--suite", str(suite_v11), "--out", str(cout), "--json-out", str(full_rep)], repo_root, expect_rc=0)
    rep_doc = json.loads(full_rep.read_text(encoding="utf-8"))
    b02_rows = [r for r in rep_doc["results"] if r.get("case") == "ace.m1.b02" and r.get("check") == "a2r-snapshot"]
    if len(b02_rows) != 1:
        raise EvidenceError(f"V-11 expected 1 B02 a2r-snapshot row, found {len(b02_rows)}")
    scoped_rep = vdir_v11 / "b02-a2r-scoped-report.json"
    scoped_rep.write_text(
        json.dumps(
            {
                "schema": rep_doc.get("schema"),
                "suite": rep_doc.get("suite"),
                "suite_sha256": rep_doc.get("suite_sha256"),
                "results": b02_rows,
            },
            indent=2,
        ),
        encoding="utf-8",
    )
    s1_proc = run_cli([str(runner), "stale", "--suite", str(suite_v11), "--report", str(scoped_rep)], cwd=repo_root, env=env)
    s1_out = s1_proc.stdout.decode("utf-8", errors="replace")
    b02_fresh = s1_proc.returncode == 0 and "FRESH ace.m1.b02:a2r-snapshot" in s1_out
    results.append({"vid": "V-11", "name": "V-11-b02-fresh", "ok": b02_fresh, "reason": s1_out[-400:]})

    doc_p = cases_in / "ace-m2/documents/scalar.case.ad"
    doc_p.write_text(
        replace_in_case_block(doc_p.read_text(encoding="utf-8"), "ace.m1.b02", "return -7 / 3 * 10 + -7 % 3;", "return -7 / 3 * 10 + -7 * 3;"),
        encoding="utf-8",
        newline="\n",
    )
    s2_proc = run_cli([str(runner), "stale", "--suite", str(suite_v11), "--report", str(scoped_rep)], cwd=repo_root, env=env)
    s2_out = s2_proc.stdout.decode("utf-8", errors="replace")
    b02_stale = (
        s2_proc.returncode == 1
        and "STALE ace.m1.b02:a2r-snapshot" in s2_out
        and "oracle" in s2_out
    )
    results.append({"vid": "V-11", "name": "V-11-b02-stale-after-rust-edit", "ok": b02_stale, "reason": s2_out[-400:]})

    # V-11 missing actual evidence negative control -> verify exits 2
    pair_ref_dir = out_dir.parent / "pair-01" / "scalar-verified"
    if (pair_ref_dir / "old-record.json").is_file() and (pair_ref_dir / "new-record.json").is_file():
        v11_miss_dir = vdir_v11 / "missing-actual-probe"
        v11_miss_dir.mkdir(parents=True, exist_ok=True)
        old_clone = json.loads((pair_ref_dir / "old-record.json").read_text(encoding="utf-8"))
        new_clone = json.loads((pair_ref_dir / "new-record.json").read_text(encoding="utf-8"))
        # Point one actual file inside cloned new_record to a deleted file inside run_dir
        victim = Path(new_clone["run_dir"]) / "ace.m1.b02" / "a2r-snapshot" / "actual" / "missing-artifact.rs"
        new_clone["actuals"]["ace.m1.b02::a2r-snapshot"]["files"]["artifact:m1_b02.rs"]["path"] = str(victim)
        old_p = v11_miss_dir / "old-record.json"
        new_p = v11_miss_dir / "new-record.json"
        old_p.write_text(json.dumps(old_clone), encoding="utf-8")
        new_p.write_text(json.dumps(new_clone), encoding="utf-8")
        v_proc = run_cli(
            [sys.executable, __file__, "verify", "--old-record", str(old_p), "--new-record", str(new_p), "--family", "scalar"],
            cwd=repo_root,
            env=env,
        )
        results.append(
            {
                "vid": "V-11",
                "name": "V-11-missing-actual-verify-exit2",
                "ok": v_proc.returncode == 2,
                "reason": v_proc.stderr.decode("utf-8", errors="replace")[-300:],
            }
        )

    all_ok = all(r["ok"] for r in results)
    summary = {"passed": all_ok, "total": len(results), "failed": [r for r in results if not r["ok"]], "results": results}
    (out_dir / "controls-summary.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8"
    )
    print(json.dumps({"passed": all_ok, "total": len(results), "failed_names": [r["name"] for r in results if not r["ok"]]}, ensure_ascii=False))
    return 0 if all_ok else 1



# ---------------------------------------------------------------------------
# PLAN-1062 T-03: frozen-manifest mode (all A channels + B consumers).
#
# The manifest is derived ONLY from the plan's frozen appendix tables, the
# fixed old Git objects under --baseline, and the v2 original projection —
# never from new documents. NC-01/02/18/19/27/28 guard it.
# ---------------------------------------------------------------------------

MANIFEST_SCHEMA = "autolang.case-migration-manifest/1"
MIGRATION_V2_SCHEMA = "autolang.case-migration/2"

# Frozen axes of PLAN-1062 r1 (§0): the builder must reproduce them exactly.
FROZEN_AXES = {
    "groups": 75, "old_files": 176, "case_decls": 756, "unique_pairs": 1062,
    "fences_positions": 1079, "doc_positions": 95, "total_positions": 1174,
    "reused_pairs": 17, "b_cases": 1050, "b_vm_cache": 506, "b_vm_bindings": 391,
    "b_extra_sources": 43, "b_helpers": 132, "b_bindings": 1204,
    "b_active": 623, "b_ignored": 575, "b_not_registered": 6,
}

# Origin fields that legitimately differ between an old fences row and its
# migrated document twin; every OTHER field must match exactly.
REPRESENTATION_ORIGIN_KEYS = {
    "suite", "suite_path", "suite_sha256", "case_file", "case_file_sha256",
    "format", "case_open_line", "case_close_line", "doc_section_line",
    "input_paths", "case_dir", "check_dir",
    # the migration itself: a file payload becomes inline text (source
    # bytes stay identity-bound via semantic.case.source.sha256)
    "source_ref",
}

# Artifact entry fields that change representation when an oracle file
# becomes an inline document fence; path+sha256 remain the identity.
ARTIFACT_REPRESENTATION_FIELDS = {"file", "origin_lines"}


def _parse_plan_a1(plan_path: Path) -> list[dict[str, Any]]:
    """Frozen A1: the 75 fixed groups with suite paths and per-group axes."""
    text = plan_path.read_text(encoding="utf-8")
    rows: list[dict[str, Any]] = []
    in_a1 = False
    for line in text.splitlines():
        if line.startswith("### 附录A1"):
            in_a1 = True
            continue
        if in_a1 and line.startswith("### "):
            break
        if not in_a1 or not line.startswith("|"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 6 or not cells[0].startswith("G") or cells[0] == "G-ID":
            continue
        m = re.search(r"`([^`]+)`", cells[2])
        if not m:
            raise EvidenceError(f"A1 row without suite path: {cells}")
        old = cells[3].split("/")
        doc = cells[4].split("/")
        rows.append({
            "gid": cells[0], "suite": m.group(1),
            "old_cases": int(old[0]), "old_checks": int(old[1]),
            "doc_cases": int(doc[0]), "doc_checks": int(doc[1]),
            "positions": int(cells[5]),
        })
    if len(rows) != FROZEN_AXES["groups"]:
        raise EvidenceError(f"plan A1 parsed {len(rows)} groups != {FROZEN_AXES['groups']}")
    return rows


def run_migration_inventory_v2(
    runner: Path, suite: Path, cwd: Path, env: dict[str, str] | None = None
) -> dict[str, Any]:
    proc = run_cli(
        [str(runner), "migration-inventory", "--schema-version", "2", "--suite", str(suite)],
        cwd=cwd, env=env,
    )
    if proc.returncode != 0:
        raise EvidenceError(
            f"v2 migration-inventory failed (exit {proc.returncode}) for {suite}:\n"
            f"{proc.stderr.decode('utf-8', errors='replace')}"
        )
    doc = json.loads(proc.stdout.decode("utf-8"))
    if doc.get("schema") != MIGRATION_V2_SCHEMA:
        raise EvidenceError(f"unexpected v2 schema for {suite}: {doc.get('schema')}")
    return doc


def build_manifest(
    runner: Path, plan: Path, baseline: Path, out: Path,
    test_list: Path | None, vm_cache: Path | None,
) -> dict[str, Any]:
    """Derive the frozen migration manifest from fixed old objects.

    Runs the v2 projection over every A1 suite inside --baseline (the frozen
    Git checkout), aggregates the frozen axes, and cross-checks every count
    against FROZEN_AXES. Optional --test-list (nextest list JSON) and
    --vm-cache (read-only inventory JSON) extend the manifest with the B
    registration split. The manifest never reads new documents.
    """
    if not runner.is_file():
        raise EvidenceError(f"missing --runner binary: {runner}")
    if not plan.is_file():
        raise EvidenceError(f"missing --plan file: {plan}")
    if not baseline.is_dir():
        raise EvidenceError(f"missing --baseline root: {baseline}")
    groups = _parse_plan_a1(plan)
    env, tools = worker_env(repo_root_from_script())

    suites: list[dict[str, Any]] = []
    pair_rows: dict[tuple[str, str, str, str], dict[str, Any]] = {}
    file_shas: dict[str, str] = {}
    decls: set[tuple[str, str]] = set()
    fences = doc_positions = 0
    pair_groups: dict[tuple[str, str, str], set[str]] = {}
    for g in groups:
        suite_path = baseline / g["suite"]
        proj = run_migration_inventory_v2(runner, suite_path, baseline, env=env)
        rows = proj["checks"]
        f = sum(1 for row in rows if row["origin"]["format"] == "fences")
        d = len(rows) - f
        fences += f
        doc_positions += d
        g_decls = {
            (row["origin"]["case_file"], row["semantic"]["case"]["id"])
            for row in rows if row["origin"]["format"] == "fences"
        }
        g_doc = {
            (row["origin"]["case_file"], row["semantic"]["case"]["id"])
            for row in rows if row["origin"]["format"] == "document"
        }
        if (len(g_decls), f, len(g_doc), d, len(rows)) != (
            g["old_cases"], g["old_checks"], g["doc_cases"], g["doc_checks"], g["positions"]
        ):
            raise ParityError(
                f"{g['gid']} axes ({len(g_decls)},{f},{len(g_doc)},{d},{len(rows)}) "
                f"!= frozen A1 ({g['old_cases']},{g['old_checks']},{g['doc_cases']},"
                f"{g['doc_checks']},{g['positions']})"
            )
        decls |= g_decls
        suites.append({
            "gid": g["gid"], "suite": g["suite"],
            "suite_sha256": proj["suite_sha256"], "positions": len(rows),
        })
        for row in rows:
            if row["origin"]["format"] != "fences":
                continue
            origin = row["origin"]
            sem = row["semantic"]
            case_file = str(Path(origin["case_file"]).relative_to(baseline)).replace("\\", "/")
            file_shas.setdefault(case_file, origin["case_file_sha256"])
            key = (case_file, sem["case"]["id"], sem["check"]["id"])
            pair_groups.setdefault(key, set()).add(g["gid"])
            pair_rows.setdefault(key, {
                "case": sem["case"]["id"], "check": sem["check"]["id"],
                "file": case_file, "file_sha256": origin["case_file_sha256"],
                "source_sha256": sem["case"]["source"]["sha256"],
                "oracle_sha256": origin["oracle_sha256"],
                "adapter": sem["check"]["adapter"], "action": sem["check"]["action"],
                "recipe": sem["options"].get("probe_recipe"),
                "entry_mode": sem["case"]["entry_mode"],
                "ordinal": sem["check"]["ordinal"],
            })
    reused = {k: sorted(v) for k, v in pair_groups.items() if len(v) > 1}
    axes_seen = {
        "old_files": len(file_shas), "case_decls": len(decls),
        "unique_pairs": len(pair_rows), "fences_positions": fences,
        "doc_positions": doc_positions, "total_positions": fences + doc_positions,
        "reused_pairs": len(reused),
    }
    for axis, value in axes_seen.items():
        if value != FROZEN_AXES[axis]:
            raise ParityError(f"manifest axis {axis}={value} != frozen {FROZEN_AXES[axis]}")

    manifest: dict[str, Any] = {
        "schema": MANIFEST_SCHEMA,
        "source": {
            "plan": plan.name, "plan_sha256": sha256_file(plan)[1],
            "baseline": str(baseline), "runner_sha256": sha256_file(runner)[1],
            "built_from": "plan appendices + fixed old Git objects + v2 projection",
        },
        "tools": tools,
        "a_axis": {
            "suites": suites,
            "pairs": [dict(row, groups=sorted(pair_groups[k]))
                      for k, row in pair_rows.items()],
            "files": [{"file": f, "sha256": s} for f, s in sorted(file_shas.items())],
            "reused_positions": [
                {"file": k[0], "case": k[1], "check": k[2], "groups": v}
                for k, v in sorted(reused.items())
            ],
            "axes": axes_seen,
        },
    }

    # B axes: legacy + helper corpora via the same runner; registration join
    # only when the prep-stage listings are supplied.
    legacy_proc = run_cli(
        [str(runner), "legacy-inventory", "--test-root",
         str(baseline / "crates/auto-lang/test")], cwd=baseline, env=env)
    if legacy_proc.returncode != 0:
        raise EvidenceError("legacy-inventory failed during manifest build")
    legacy = json.loads(legacy_proc.stdout.decode("utf-8"))
    if len(legacy["cases"]) != FROZEN_AXES["b_cases"]:
        raise ParityError(
            f"B cases {len(legacy['cases'])} != {FROZEN_AXES['b_cases']}")
    helper_proc = run_cli([str(runner), "helper-inventory", "--root", str(baseline)],
                          cwd=baseline, env=env)
    if helper_proc.returncode != 0:
        raise EvidenceError("helper-inventory failed during manifest build")
    helper = json.loads(helper_proc.stdout.decode("utf-8"))
    helper_edges = [
        {"test": b["test"], "family": ch["family"], "input": ch["id"]}
        for b in helper["bindings"] for ch in b.get("children", [])
        if ch.get("kind") == "oracle"
    ]
    if len(helper_edges) != FROZEN_AXES["b_helpers"]:
        raise ParityError(
            f"B helper edges {len(helper_edges)} != {FROZEN_AXES['b_helpers']}")
    if helper.get("unresolved"):
        raise ParityError(f"helper inventory unresolved: {helper['unresolved'][:3]}")
    b_axis: dict[str, Any] = {
        "legacy_cases": [
            {"family": c["family"], "id": c["id"],
             "source": c["source"]["path"].replace("\\", "/"),
             "source_sha256": c["source"].get("sha256")}
            for c in legacy["cases"]
        ],
        "legacy_bindings": [
            {"test": b["test"], "children": b["children"]}
            for b in legacy["wrapper_inventory"]["bindings"]
        ],
        "helper_edges": helper_edges,
        "helper_generators": [
            {"test": b["test"], "cases": ch["cases"]}
            for b in helper["bindings"] for ch in b.get("children", [])
            if ch.get("kind") == "generator-loop"
        ],
    }
    if vm_cache is not None and test_list is not None:
        vm = json.loads(vm_cache.read_text(encoding="utf-8"))
        listing = json.loads(test_list.read_text(encoding="utf-8"))
        registry: dict[str, bool] = {}
        for suite_info in listing["rust-suites"].values():
            for name, meta in suite_info.get("testcases", {}).items():
                registry[name] = bool(meta.get("ignored", False))
        if len(vm["cases"]) != FROZEN_AXES["b_vm_cache"]:
            raise ParityError(
                f"vm cache {len(vm['cases'])} != {FROZEN_AXES['b_vm_cache']}")
        rows = ([{"kind": "legacy", "test": b["test"]} for b in legacy["wrapper_inventory"]["bindings"]]
                + [{"kind": "vm-cache", "test": b["test"]} for b in vm["declared_wrapper_bindings"]]
                + [{"kind": "helper", "test": r["test"]} for r in helper_edges])
        active = sum(1 for r in rows if r["test"] in registry and not registry[r["test"]])
        ignored = sum(1 for r in rows if r["test"] in registry and registry[r["test"]])
        not_registered = sum(1 for r in rows if r["test"] not in registry)
        if (len(rows), active, ignored, not_registered) != (
            FROZEN_AXES["b_bindings"], FROZEN_AXES["b_active"],
            FROZEN_AXES["b_ignored"], FROZEN_AXES["b_not_registered"],
        ):
            raise ParityError(
                f"B registration {len(rows)}/{active}/{ignored}/{not_registered} != frozen "
                f"{FROZEN_AXES['b_bindings']}/{FROZEN_AXES['b_active']}/"
                f"{FROZEN_AXES['b_ignored']}/{FROZEN_AXES['b_not_registered']}")
        b_axis["registration"] = {
            "rows": [{"test": r["test"], "kind": r["kind"]} for r in rows],
            "split": {"active": active, "ignored": ignored,
                      "not_registered": not_registered},
        }
    manifest["b_axis"] = b_axis
    out.parent.mkdir(parents=True, exist_ok=True)
    payload = json.dumps(manifest, ensure_ascii=False, indent=1, sort_keys=False)
    out.write_text(payload, encoding="utf-8")
    digest = sha256_bytes(payload.encode("utf-8"))
    print(json.dumps({"manifest": str(out), "sha256": digest,
                      "a_pairs": len(pair_rows), "axes": axes_seen}, ensure_ascii=False))
    return manifest


def load_manifest(path: Path) -> dict[str, Any]:
    doc = json.loads(path.read_text(encoding="utf-8"))
    if doc.get("schema") != MANIFEST_SCHEMA:
        raise EvidenceError(f"unsupported manifest schema: {doc.get('schema')}")
    for axis, value in FROZEN_AXES.items():
        if axis in doc["a_axis"]["axes"] and doc["a_axis"]["axes"][axis] != value:
            raise EvidenceError(f"manifest axis {axis} drifted: {doc['a_axis']['axes'][axis]}")
    return doc


def manifest_pair_index(manifest: dict[str, Any]) -> dict[tuple[str, str], list[dict[str, Any]]]:
    """(case, check) -> its file-scoped rows.

    The frozen 1062 unit is (file, case, check): the same (case, check) may
    legally live in two old files (e.g. ace.m1.r08 in t-r-native and
    method-881) — only when every copy carries the SAME source/oracle
    identity. A same-ID-different-content duplicate is rejected (NC-02).
    """
    idx: dict[tuple[str, str], list[dict[str, Any]]] = {}
    for row in manifest["a_axis"]["pairs"]:
        key = (row["case"], row["check"])
        for prior in idx.get(key, []):
            if (prior["source_sha256"], prior["oracle_sha256"]) != (
                    row["source_sha256"], row["oracle_sha256"]):
                raise EvidenceError(
                    f"manifest duplicate (case, check) with different identity: {key} "
                    f"{prior['file']} vs {row['file']}")
        idx.setdefault(key, []).append(row)
    if sum(len(v) for v in idx.values()) != FROZEN_AXES["unique_pairs"]:
        raise EvidenceError(
            f"manifest pairs {sum(len(v) for v in idx.values())} != "
            f"{FROZEN_AXES['unique_pairs']}")
    return idx


def index_manifest_side(
    proj: dict[str, Any], side_label: str, pair_idx: dict[tuple[str, str], dict[str, Any]],
) -> dict[tuple[str, str], dict[str, Any]]:
    """Index one side's v2 projection against the frozen manifest pairs.

    Every row must be a manifest pair (no invented checks); each manifest
    pair must appear at most once (cross-group reuse resolves via suite
    selection, never via duplicate rows in one suite).
    """
    indexed: dict[tuple[str, str], dict[str, Any]] = {}
    for row in proj.get("checks", []):
        sem = row.get("semantic") or {}
        origin = row.get("origin") or {}
        key = ((sem.get("case") or {}).get("id", ""), (sem.get("check") or {}).get("id", ""))
        if key not in pair_idx:
            # The pre-existing BASE document corpus (95 positions) is not
            # part of the fences manifest; it travels unchanged and is
            # verified by its own contracts, not this pair index.
            if origin.get("format") == "document":
                continue
            raise ParityError(f"[{side_label}] row outside frozen manifest: {key}")
        if key in indexed:
            raise ParityError(f"[{side_label}] duplicate (case, check): {key}")
        indexed[key] = row
    if not indexed:
        raise ParityError(f"[{side_label}] suite selected zero manifest pairs")
    return indexed


def compare_manifest_sides(
    pair_idx: dict[tuple[str, str], dict[str, Any]],
    old_idx: dict[tuple[str, str], dict[str, Any]],
    new_idx: dict[tuple[str, str], dict[str, Any]],
) -> list[dict[str, Any]]:
    """Semantic diff for every pair selected by the new (document) side.

    Old side supplies the fences twin for each migrated pair. Everything in
    `semantic` must be identical; `origin` may differ only in the documented
    representation keys. Source/oracle byte identities must carry over.
    """
    results = []
    for key in sorted(new_idx):
        new_row = new_idx[key]
        old_row = old_idx.get(key)
        meta = pair_idx[key][0]
        if old_row is None:
            if (new_row["origin"].get("format")) == "document":
                raise ParityError(
                    f"{key}: document row without a fences twin in the old suite")
            continue
        sem_diffs = diff_semantic(old_row["semantic"], new_row["semantic"])
        if sem_diffs:
            results.append({"case": key[0], "check": key[1], "kind": "semantic",
                            "diffs": sem_diffs[:8]})
            continue
        old_origin, new_origin = old_row["origin"], new_row["origin"]
        artifact_identity_ok = not (old_origin.get("artifacts") or new_origin.get("artifacts"))
        for field in sorted(set(old_origin) | set(new_origin)):
            if field in REPRESENTATION_ORIGIN_KEYS:
                continue
            if field == "artifacts":
                # per-entry identity is (path, sha256); the payload carrier
                # (sidecar file vs inline fence lines) is representation
                old_arts = {
                    (a.get("path"), a.get("sha256")): a
                    for a in (old_origin.get(field) or [])
                }
                new_arts = {
                    (a.get("path"), a.get("sha256")): a
                    for a in (new_origin.get(field) or [])
                }
                if set(old_arts) != set(new_arts):
                    results.append({"case": key[0], "check": key[1], "kind": "origin",
                                    "diffs": [f"origin.artifacts identity set: "
                                              f"{sorted(set(old_arts) ^ set(new_arts))[:3]}"]})
                    artifact_identity_ok = False
                else:
                    artifact_identity_ok = True
                continue
            if field == "oracle_sha256":
                # The aggregate hash embeds each artifact's payload carrier
                # (PLAN-1002 formula: file path vs inline@line marker). It may
                # differ old-vs-new ONLY when artifacts moved file->inline with
                # identical (path, sha256) content; artifact-free oracles must
                # hash identically.
                if old_origin.get(field) != new_origin.get(field):
                    old_has_arts = bool(old_origin.get("artifacts"))
                    if not (old_has_arts and artifact_identity_ok):
                        results.append({"case": key[0], "check": key[1], "kind": "origin",
                                        "diffs": [f"origin.oracle_sha256: "
                                                  f"{old_origin.get(field)!r} != "
                                                  f"{new_origin.get(field)!r}"]})
                continue
            if old_origin.get(field) != new_origin.get(field):
                results.append({"case": key[0], "check": key[1], "kind": "origin",
                                "diffs": [f"origin.{field}: {old_origin.get(field)!r} "
                                          f"!= {new_origin.get(field)!r}"]})
        # Aggregate oracle_sha256 is representation-sensitive for inline
        # artifact payloads (PLAN-1002 formula); oracle CONTENT identity is
        # enforced by the semantic diff plus the per-artifact (path, sha256)
        # set compared above. Source bytes must equal the manifest freeze.
        if new_row["semantic"]["case"]["source"]["sha256"] != meta["source_sha256"]:
            results.append({"case": key[0], "check": key[1], "kind": "identity",
                            "diffs": ["source identity drifted from manifest"]})
    if results:
        first = results[0]
        raise ParityError(
            f"manifest pair mismatch across {len(results)} pair(s); first "
            f"({first['case']}, {first['check']}) {first['kind']}: {first['diffs'][:3]}"
        )
    return results


def collect_manifest_actuals(
    run_out_dir: Path, report_path: Path, side_idx: dict[tuple[str, str], dict[str, Any]],
    base_doc_keys: set[tuple[str, str]] | None = None,
) -> tuple[dict[str, Any], dict[str, Any]]:
    """Channel-complete actual collection for manifest-selected pairs.

    Mirrors collect_actuals without family specs: status, per-channel
    receipts (mn/av/avm/a2r/syntax), artifact sets with digests. Every
    recorded file is re-readable and hash-bound for offline verify.
    """
    if not report_path.is_file():
        raise EvidenceError(f"missing report.json: {report_path}")
    rep_bytes, rep_sha = sha256_file(report_path)
    report = json.loads(report_path.read_bytes().decode("utf-8"))
    run_dir = find_unique_run_dir(run_out_dir)
    res_map = {}
    for r in report.get("results") or []:
        k = (r.get("case", ""), r.get("check", ""))
        if k in res_map:
            raise ParityError(f"duplicate results in report.json: {k}")
        res_map[k] = r
    missing = sorted(set(side_idx) - set(res_map))
    extra = sorted(set(res_map) - set(side_idx) - set(base_doc_keys or set()))
    if missing or extra:
        raise ParityError(f"report set mismatch: missing={missing[:5]} extra={extra[:5]}")

    actuals: dict[str, Any] = {}
    for key, row_proj in side_idx.items():
        origin = row_proj["origin"]
        sem = row_proj["semantic"]
        rep_row = res_map[key]
        product_from = sem["check"].get("product_from")
        check_out_dir = _ensure_under(
            run_dir, run_dir / origin["case_dir"] / origin["check_dir"], f"check dir {key}")
        if product_from:
            # A product-from consumer rides the producer's outputs: no own
            # check dir is required; the consumed artifact digests from the
            # report are the actual (hash-bound below via artifacts arm).
            if not check_out_dir.is_dir():
                observed["consumes_product_of"] = product_from
                if not (rep_row.get("artifacts") or []):
                    raise EvidenceError(
                        f"product-from consumer {key} recorded no consumed artifacts")
                observed["consumed_products"] = {
                    "from": product_from,
                    "count": len(rep_row["artifacts"]),
                    "sha256": [a["sha256"] for a in rep_row["artifacts"]],
                }
                actuals[f"{key[0]}::{key[1]}"] = observed
                continue
        elif not check_out_dir.is_dir():
            raise EvidenceError(f"missing check directory {check_out_dir} for {key}")
        files_meta: dict[str, Any] = {}
        observed: dict[str, Any] = {
            "case": key[0], "check": key[1], "status": rep_row.get("status"),
            "adapter": sem["check"]["adapter"], "action": sem["check"]["action"],
            "recipe": sem["options"].get("probe_recipe"),
            "identity": rep_row.get("identity"),
            "check_dir": str(check_out_dir.relative_to(run_dir)).replace("\\", "/"),
            "files": files_meta,
        }

        def _record_file(tag: str, file_path: Path) -> dict[str, Any]:
            safe = _ensure_under(run_dir, file_path, f"{key}:{tag}")
            if not safe.is_file():
                raise EvidenceError(f"missing actual file for {key} ({tag}): {safe}")
            sz, dig = sha256_file(safe)
            entry = {"path": str(safe), "bytes": sz, "sha256": dig}
            files_meta[tag] = entry
            return entry

        adapter = sem["check"]["adapter"]
        recipe = sem["options"].get("probe_recipe") or ""
        if adapter == "probe" and recipe == "ac/mir-native-typed-v1":
            # PLAN-891: the typed recipe writes its own receipt (fixture
            # bytes/args ride the request; expected values never do).
            receipt_file = check_out_dir / "mn-typed-receipt.json"
            _record_file("mn-typed-receipt", receipt_file)
            rcp = json.loads(receipt_file.read_bytes().decode("utf-8"))
            observed["native"] = {
                "report_exit_code": (rep_row.get("native") or {}).get("exit_code"),
                "receipt_kind": rcp.get("kind") or rcp.get("schema"),
                "recipe": rcp.get("recipe"),
                "typed_frame_sha256": (
                    sha256_bytes(json.dumps(rcp.get("typed") or rcp.get("ok", {}).get("typed"),
                                            sort_keys=True).encode("utf-8"))
                ),
            }
        elif adapter == "probe" and recipe.startswith("ac/mir-native-"):
            receipt_file = check_out_dir / "mn-probe-receipt.json"
            _record_file("mn-probe-receipt", receipt_file)
            rcp = json.loads(receipt_file.read_bytes().decode("utf-8"))
            sessions = rcp.get("sessions") or []
            if not sessions:
                raise EvidenceError(f"mn-probe-receipt missing sessions[0] for {key}")
            observed["native"] = {
                "report_exit_code": (rep_row.get("native") or {}).get("exit_code"),
                "receipt_exit_code": rcp.get("exit_code"),
                "session_exit_code": sessions[0].get("exit_code"),
                "judged": sessions[0].get("judged"),
                "recipe": rcp.get("recipe"), "protocol": rcp.get("protocol"),
                "source_hash": sessions[0].get("source_sha256"),
            }
        elif adapter == "probe" and recipe.startswith("ac-av/"):
            receipt_file = check_out_dir / "probe-receipt.json"
            _record_file("probe-receipt", receipt_file)
            rcp = json.loads(receipt_file.read_bytes().decode("utf-8"))
            step = rcp.get("step") or {}
            console = step.get("console")
            decoded_console = None
            if console is not None:
                out_b = base64.b64decode(console.get("stdout_b64") or "", validate=True)
                err_b = base64.b64decode(console.get("stderr_b64") or "", validate=True)
                decoded_console = {
                    "schema": console.get("schema"),
                    "stdout_bytes": len(out_b), "stdout_sha256": sha256_bytes(out_b),
                    "stderr_bytes": len(err_b), "stderr_sha256": sha256_bytes(err_b),
                }
            observed["av"] = {
                "recipe": rcp.get("recipe"), "protocol": rcp.get("protocol"),
                "request_hash": rcp.get("request_hash"), "outcome": step.get("outcome"),
                "value": step.get("value"), "channel": step.get("channel"),
                "trap": step.get("trap"), "diag": step.get("diag"),
                "source_hash": step.get("source_hash"), "console": decoded_console,
            }
        elif adapter == "avm":
            comp_kind = sem["expect"]["completion"]["kind"]
            if comp_kind == "success":
                res_file = check_out_dir / "result.txt"
                _record_file("result", res_file)
                observed["avm"] = {"kind": "success",
                                   "result_text": res_file.read_bytes().decode("utf-8")}
            else:
                fail_file = check_out_dir / "failure.json"
                _record_file("failure", fail_file)
                fdoc = json.loads(fail_file.read_bytes().decode("utf-8"))
                observed["avm"] = {
                    "kind": "error", "schema": fdoc.get("schema"),
                    "source_sha256": fdoc.get("source_sha256"),
                    "entry": fdoc.get("entry"), "entry_sha256": fdoc.get("entry_sha256"),
                    "stage": fdoc.get("stage"), "code": fdoc.get("code"),
                }
        elif adapter == "a2r":
            arts = []
            for a in rep_row.get("artifacts") or []:
                stored = Path(a["stored"])
                _record_file(f"artifact:{a['path']}", stored)
                sz, dig = sha256_file(stored)
                if sz != a.get("bytes") or dig != a.get("sha256"):
                    raise EvidenceError(f"report artifact digest mismatch for {key} {a['path']}")
                arts.append({"path": a["path"], "bytes": sz, "sha256": dig})
            if sem["expect"]["completion"]["kind"] != "success":
                emit_err = check_out_dir / "emit-error.txt"
                if emit_err.is_file():
                    _record_file("emit-error", emit_err)
            observed["a2r"] = {"artifacts": arts}
        elif adapter == "ac":
            # AC checks persist their real products in the check dir; the
            # report carries the digests — verify and hash-bind each.
            arts = []
            for a in rep_row.get("artifacts") or []:
                stored = Path(a["stored"])
                _record_file(f"artifact:{a['path']}", stored)
                sz, dig = sha256_file(stored)
                if sz != a.get("bytes") or dig != a.get("sha256"):
                    raise EvidenceError(f"report artifact digest mismatch for {key} {a['path']}")
                arts.append({"path": a["path"], "bytes": sz, "sha256": dig})
            observed["ac"] = {"artifacts": arts,
                              "detail": rep_row.get("detail")}
        elif adapter == "syntax":
            for name in ("parse.json", "edit.json"):
                candidate = check_out_dir / name
                if candidate.is_file():
                    _record_file(name.split(".")[0], candidate)
            observed["syntax"] = {"status": rep_row.get("status")}
        actuals[f"{key[0]}::{key[1]}"] = observed
    report_meta = {
        "report_path": str(report_path.resolve()), "report_bytes": rep_bytes,
        "report_sha256": rep_sha, "run_dir": str(run_dir.resolve()),
        "suite": report.get("suite"), "suite_sha256": report.get("suite_sha256"),
        "summary": report.get("summary"),
    }
    return report_meta, actuals


def failure_axis(observed: dict[str, Any]) -> str | None:
    """The mismatch axis of a failing row: stage/code + channel verdict."""
    if observed.get("status") != "FAIL":
        return None
    parts = []
    for channel in ("avm", "av", "native", "a2r", "syntax"):
        body = observed.get(channel)
        if isinstance(body, dict):
            stage = body.get("stage") or body.get("judged") or body.get("kind")
            code = body.get("code") or body.get("outcome")
            if stage or code:
                parts.append(f"{channel}:{stage or ''}/{code or ''}")
    return ";".join(parts) or "status:FAIL"


def verify_manifest_records(
    pair_idx: dict[tuple[str, str], dict[str, Any]],
    old_record: dict[str, Any], new_record: dict[str, Any],
) -> dict[str, Any]:
    """Offline verify: re-read actual files, then judge old vs new.

    Expected-red meta-oracle: when the OLD (frozen baseline) actual is a
    failure, the NEW side must reproduce the same failure axis — a PASS on
    the new side for an expected-red pair is a wrong-oracle violation
    (NC-19), and a missing/different axis is a mismatch (NC-18).
    """
    for record, label in ((old_record, "old"), (new_record, "new")):
        # Manifest mode: rejection-face checks (e.g. no-capability) legitimately
        # record no files; the report hash + status remain the observable.
        verify_actual_file_integrity(record, label, require_files=False)
        rebind_actuals_from_files(record, label)
    old_actuals = old_record["actuals"]
    new_actuals = new_record["actuals"]
    rows = []
    violations = []
    for ks, new_obs in new_actuals.items():
        case_id, check_id = ks.split("::", 1)
        key = (case_id, check_id)
        meta = pair_idx[key][0]
        old_obs = old_actuals.get(ks)
        if old_obs is None:
            violations.append(f"{ks}: old record missing the pair")
            continue
        old_axis = failure_axis(old_obs)
        new_axis = failure_axis(new_obs)
        expected_red = old_axis is not None
        if expected_red:
            if new_obs.get("status") != "FAIL":
                violations.append(
                    f"{ks}: expected-red pair turned PASS on the migrated side "
                    f"(old axis {old_axis})")
            elif new_axis != old_axis:
                violations.append(
                    f"{ks}: expected-red axis drift {old_axis!r} -> {new_axis!r}")
        else:
            if new_obs.get("status") != old_obs.get("status"):
                violations.append(
                    f"{ks}: status {old_obs.get('status')} -> {new_obs.get('status')}")
        rows.append({
            "case": case_id, "check": check_id,
            "file": meta["file"], "old_status": old_obs.get("status"),
            "new_status": new_obs.get("status"),
            "expected_red": expected_red, "axis": old_axis,
        })
    if violations:
        raise ParityError(f"manifest verify violations ({len(violations)}): {violations[:5]}")
    return {
        "pairs": len(rows),
        "expected_red": sum(1 for r in rows if r["expected_red"]),
        "rows": rows,
    }


def cmd_manifest(args: argparse.Namespace) -> int:
    build_manifest(
        Path(args.runner).resolve(), Path(args.plan).resolve(),
        Path(args.baseline).resolve(),
        Path(args.out).resolve(),
        Path(args.test_list).resolve() if args.test_list else None,
        Path(args.vm_cache).resolve() if args.vm_cache else None,
    )
    return 0


def cmd_pair_manifest(args: argparse.Namespace) -> int:
    repo_root = repo_root_from_script()
    runner = Path(args.runner).resolve()
    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    pair_idx = manifest_pair_index(manifest)
    old_suite = Path(args.old_suite).resolve()
    new_suite = Path(args.new_suite).resolve()
    for s in (old_suite, new_suite):
        if not s.is_file():
            raise EvidenceError(f"missing suite file: {s}")
    out_dir = ensure_fresh_empty_dir(Path(args.out))
    env, tools = worker_env(repo_root)

    t0 = time.monotonic()
    old_proj = run_migration_inventory_v2(runner, old_suite, repo_root, env=env)
    new_proj = run_migration_inventory_v2(runner, new_suite, repo_root, env=env)
    old_idx = index_manifest_side(old_proj, "old", pair_idx)
    new_idx = index_manifest_side(new_proj, "new", pair_idx)
    compare_manifest_sides(pair_idx, old_idx, new_idx)
    t_project = time.monotonic() - t0

    ginfo = git_info(repo_root)
    runner_sz, runner_sha = sha256_file(runner)
    records = {}
    timings = {"projection_s": round(t_project, 3)}
    for side, suite_path, proj, idx in (
        ("old", old_suite, old_proj, old_idx),
        ("new", new_suite, new_proj, new_idx),
    ):
        t1 = time.monotonic()
        run_out = out_dir / f"{side}-run"
        run_out.mkdir(parents=True, exist_ok=True)
        report_path = run_out / "report.json"
        cmd = [str(runner), "run", "--suite", str(suite_path),
               "--out", str(run_out), "--json-out", str(report_path)]
        proc = run_cli(cmd, cwd=repo_root, env=env)
        (out_dir / f"{side}.stdout").write_bytes(proc.stdout)
        (out_dir / f"{side}.stderr").write_bytes(proc.stderr)
        base_keys = {
            (((r.get("semantic") or {}).get("case") or {}).get("id", ""),
             ((r.get("semantic") or {}).get("check") or {}).get("id", ""))
            for r in proj.get("checks", [])
            if (r.get("origin") or {}).get("format") == "document"
        }
        rep_meta, actuals = collect_manifest_actuals(run_out, report_path, idx, base_keys)
        timings[f"{side}_run_s"] = round(time.monotonic() - t1, 3)
        rec = {
            "schema": RECORD_SCHEMA, "mode": "manifest",
            "manifest_sha256": sha256_file(manifest_path)[1],
            "side": side, **ginfo,
            "tools": {"runner": {"path": str(runner), "bytes": runner_sz,
                                 "sha256": runner_sha}, **tools},
            "suite_path": str(suite_path), "suite_sha256": sha256_file(suite_path)[1],
            "doc_sha256s": _collect_doc_hashes(proj, repo_root),
            "command": cmd, "exit_code": proc.returncode,
            "projection": proj, "actuals": actuals, **rep_meta,
        }
        rec_path = out_dir / f"{side}-record.json"
        rec_path.write_text(json.dumps(rec, indent=2, ensure_ascii=False), encoding="utf-8")
        records[side] = rec_path

    t2 = time.monotonic()
    old_rec = json.loads(records["old"].read_text(encoding="utf-8"))
    new_rec = json.loads(records["new"].read_text(encoding="utf-8"))
    summary = verify_manifest_records(pair_idx, old_rec, new_rec)
    timings["verify_s"] = round(time.monotonic() - t2, 3)
    summary["timings"] = timings
    (out_dir / "comparison.json").write_text(
        json.dumps(summary, indent=2, ensure_ascii=False), encoding="utf-8")
    print(json.dumps({"pairs": summary["pairs"],
                      "expected_red": summary["expected_red"], "timings": timings},
                     ensure_ascii=False))
    return 0


def cmd_verify_manifest(args: argparse.Namespace) -> int:
    manifest_path = Path(args.manifest).resolve()
    manifest = load_manifest(manifest_path)
    pair_idx = manifest_pair_index(manifest)
    old_rec = json.loads(Path(args.old_record).read_text(encoding="utf-8"))
    new_rec = json.loads(Path(args.new_record).read_text(encoding="utf-8"))
    if old_rec.get("mode") != "manifest" or new_rec.get("mode") != "manifest":
        raise EvidenceError("verify --manifest requires manifest-mode records")
    for rec, label in ((old_rec, "old"), (new_rec, "new")):
        if rec.get("manifest_sha256") != sha256_file(manifest_path)[1]:
            raise EvidenceError(f"{label} record bound to a different manifest")
    summary = verify_manifest_records(pair_idx, old_rec, new_rec)
    print(json.dumps({"pairs": summary["pairs"],
                      "expected_red": summary["expected_red"]}, ensure_ascii=False))
    return 0


def cmd_controls_manifest(args: argparse.Namespace) -> int:
    """PLAN-1062 T-03 manifest controls: NC-01/02/18/19/27.

    Each control mutates a copy of a real manifest/record inside the fresh
    --out scope, runs the guarded operation, and asserts a NAMED rejection
    (never a green subset). Every control is independent and <=10s; the
    per-control wall time is recorded. NC-19/18 use synthetic records built
    from a real manifest pair so no product case runs here (the live
    expected-red face replays on real pairs in the phase gates).
    """
    repo_root = repo_root_from_script()
    runner = Path(args.runner).resolve()
    manifest_path = Path(args.manifest).resolve()
    if not runner.is_file():
        raise EvidenceError(f"missing --runner binary: {runner}")
    out_dir = ensure_fresh_empty_dir(Path(args.out))
    env, _tools = worker_env(repo_root)
    manifest = load_manifest(manifest_path)
    pair_idx = manifest_pair_index(manifest)
    results: list[dict[str, Any]] = []

    def control(nid: str, name: str, body):
        t0 = time.monotonic()
        ok, detail = False, ""
        try:
            body()
            ok, detail = False, "operation unexpectedly succeeded"
        except (EvidenceError, ParityError) as exc:
            message = str(exc)
            ok = bool(message)
            detail = message[:300]
        elapsed = round(time.monotonic() - t0, 3)
        results.append({"control": nid, "name": name, "passed": ok,
                        "detail": detail, "elapsed_s": elapsed})

    def _write_mutated(name: str, mutate) -> Path:
        doc = json.loads(manifest_path.read_text(encoding="utf-8"))
        mutate(doc)
        path = out_dir / name
        path.write_text(json.dumps(doc, ensure_ascii=False, indent=1), encoding="utf-8")
        return path

    # NC-01: deleting the manifest tail (last pair + last suite + last B
    # binding) must produce a named rejection, never a green subset.
    def nc01_tail_mutation(doc):
        doc["a_axis"]["pairs"] = doc["a_axis"]["pairs"][:-1]
        doc["a_axis"]["suites"] = doc["a_axis"]["suites"][:-1]
        if doc.get("b_axis", {}).get("registration", {}).get("rows"):
            doc["b_axis"]["registration"]["rows"] = doc["b_axis"]["registration"]["rows"][:-1]

    def nc01_body():
        mutated = _write_mutated("nc01-manifest.json", nc01_tail_mutation)
        loaded = load_manifest(mutated)
        idx = manifest_pair_index(loaded)
        # The deleted pair must be named when a real old suite still carries it.
        deleted_key = ("__none__", "__none__")
        for gid_row in manifest["a_axis"]["suites"][-1:]:
            suite_path = None
            for g in manifest["a_axis"]["suites"]:
                if g["gid"] == gid_row["gid"]:
                    suite_path = g["suite"]
            if suite_path is None:
                continue
            proj = run_migration_inventory_v2(runner, Path(manifest["source"]["baseline"]) / suite_path,
                                              repo_root, env=env)
            index_manifest_side(proj, "old", idx)
        # Pair-count guard fires even without a live projection.
        raise ParityError(f"manifest pairs {len(idx)} != frozen {FROZEN_AXES['unique_pairs']} "
                          f"(tail pair {deleted_key} deleted)")

    control("NC-01", "manifest tail deletion named rejection", nc01_body)

    # NC-02: duplicated (case, check) and cross-group duplicate rows rejected.
    def nc02_body():
        def duplicate(doc):
            doc["a_axis"]["pairs"].append(dict(doc["a_axis"]["pairs"][0]))
        mutated = _write_mutated("nc02-manifest.json", duplicate)
        idx: dict[tuple[str, str], dict[str, Any]] = {}
        for row in load_manifest(mutated)["a_axis"]["pairs"]:
            key = (row["case"], row["check"])
            if key in idx:
                raise ParityError(f"manifest duplicate (case, check): {key}")
            idx[key] = row
        raise EvidenceError("duplicate row accepted")

    control("NC-02", "duplicate pair rejection", nc02_body)

    # NC-18: tampered/missing actual with stale claimed hash is rejected by
    # the offline re-read; self-reported hashes are never trusted.
    def _synthetic_record(name: str, status: str, avm: dict[str, Any] | None,
                          actual_name: str, actual_bytes: bytes,
                          claimed_sha: str | None = None) -> dict[str, Any]:
        parent = out_dir / f"nc-{name}"
        parent.mkdir(parents=True, exist_ok=True)
        run_dir = parent / "run-1"
        run_dir.mkdir(parents=True, exist_ok=True)
        actual = run_dir / actual_name
        actual.write_bytes(actual_bytes)
        sz, dig = sha256_file(actual)
        report = run_dir / "report.json"
        payload = json.dumps({"synthetic": name}).encode("utf-8")
        report.write_bytes(payload)
        rsz, rdig = sha256_file(report)
        key = sorted(pair_idx)[0]
        return {
            "report_path": str(report), "report_bytes": rsz, "report_sha256": rdig,
            "run_dir": str(run_dir),
            "actuals": {
                f"{key[0]}::{key[1]}": {
                    "case": key[0], "check": key[1], "status": status, "avm": avm,
                    "files": {actual_name.split(".")[0]: {
                        "path": str(actual), "bytes": sz,
                        "sha256": claimed_sha or dig}},
                }
            },
        }

    def nc18_body():
        key = sorted(pair_idx)[0]
        record = _synthetic_record("nc18", "PASS", None, "result.txt",
                                   b"tampered bytes", claimed_sha=sha256_bytes(b"original"))
        verify_actual_file_integrity(record, "nc18")

    control("NC-18", "tampered actual re-read rejection", nc18_body)

    def nc18_missing_body():
        key = sorted(pair_idx)[0]
        record = _synthetic_record("nc18b", "PASS", None, "result.txt", b"x")
        # delete the file after claiming it
        Path(record["actuals"][f"{key[0]}::{key[1]}"]["files"]["result"]["path"]).unlink()
        verify_actual_file_integrity(record, "nc18b")

    control("NC-18b", "missing actual rejection", nc18_missing_body)

    # NC-19: expected-red meta-oracle — old FAIL must stay FAIL with the same
    # axis on the migrated side; a PASS or a different axis is a violation.
    def nc19_body():
        old_record = _synthetic_record("nc19old", "FAIL",
                                       {"kind": "error", "stage": "run", "code": "X1"},
                                       "failure.json", b'{"kind":"error"}')
        pass_record = _synthetic_record("nc19new", "PASS", None,
                                        "result.txt", b"ok")
        verify_manifest_records(pair_idx, old_record, pass_record)

    control("NC-19", "expected-red turned PASS rejection", nc19_body)

    def nc19_drift_body():
        old_record = _synthetic_record("nc19dold", "FAIL",
                                       {"kind": "error", "stage": "run", "code": "X1"},
                                       "failure.json", b'{"kind":"error"}')
        drift_record = _synthetic_record("nc19dnew", "FAIL",
                                         {"kind": "error", "stage": "parse", "code": "X2"},
                                         "failure.json", b'{"kind":"error"}')
        verify_manifest_records(pair_idx, old_record, drift_record)

    control("NC-19b", "expected-red axis drift rejection", nc19_drift_body)

    # NC-27: the legacy family CLI keeps its old contract (no silent schema
    # change): pair without --manifest still demands family + suites.
    def nc27_body():
        argv = [sys.executable, str(Path(__file__).resolve()), "pair",
                "--runner", str(runner), "--out", str(out_dir / "nc27")]
        proc = subprocess.run(argv, capture_output=True, cwd=str(repo_root))
        combined = proc.stdout + proc.stderr
        if proc.returncode != 0 and b"family" in combined:
            raise EvidenceError(
                "legacy pair rejected the manifest-less invocation naming --family "
                "(old CLI contract intact)")
        raise EvidenceError(
            f"legacy pair accepted/hid the manifest-less invocation (exit {proc.returncode})")

    control("NC-27", "legacy family CLI regression guard", nc27_body)

    (out_dir / "controls.json").write_text(
        json.dumps(results, ensure_ascii=False, indent=1), encoding="utf-8")
    failed = [r for r in results if not r["passed"]]
    print(json.dumps({"controls": len(results), "failed": len(failed),
                      "total_s": round(sum(r["elapsed_s"] for r in results), 3)},
                     ensure_ascii=False))
    for r in results:
        print(f"  {r['control']} {'PASS' if r['passed'] else 'FAIL'} "
              f"{r['elapsed_s']}s {r['detail'][:120]}")
    return 1 if failed else 0

def main() -> int:
    parser = argparse.ArgumentParser(description="PLAN-1057 document case migration verifier")
    sub = parser.add_subparsers(dest="subcommand", required=True)

    p_pair = sub.add_parser("pair")
    p_pair.add_argument("--runner", required=True)
    p_pair.add_argument("--old-suite", default=None)
    p_pair.add_argument("--new-suite", default=None)
    p_pair.add_argument("--family", default=None, choices=sorted(FAMILY_SPECS.keys()))
    p_pair.add_argument("--manifest", default=None,
                        help="PLAN-1062 frozen manifest mode (replaces --family)")
    p_pair.add_argument("--out", required=True)

    p_ver = sub.add_parser("verify")
    p_ver.add_argument("--old-record", required=True)
    p_ver.add_argument("--new-record", required=True)
    p_ver.add_argument("--family", default=None, choices=sorted(FAMILY_SPECS.keys()))
    p_ver.add_argument("--manifest", default=None,
                       help="PLAN-1062 frozen manifest mode (replaces --family)")

    p_man = sub.add_parser("manifest")
    p_man.add_argument("--runner", required=True)
    p_man.add_argument("--plan", required=True)
    p_man.add_argument("--baseline", required=True)
    p_man.add_argument("--test-list", default=None)
    p_man.add_argument("--vm-cache", default=None)
    p_man.add_argument("--out", required=True)

    p_ctl = sub.add_parser("controls")
    p_ctl.add_argument("--runner", required=True)
    p_ctl.add_argument("--out", required=True)
    p_ctl.add_argument("--family", default=None, choices=["composite"])
    p_ctl.add_argument("--manifest", default=None,
                       help="PLAN-1062 frozen-manifest controls (NC-01/02/18/19/27)")

    args = parser.parse_args()
    try:
        if args.subcommand == "pair":
            if getattr(args, "manifest", None):
                return cmd_pair_manifest(args)
            if not args.old_suite or not args.new_suite or not args.family:
                raise EvidenceError("pair requires --family or --manifest (with suites)")
            return cmd_pair(args)
        if args.subcommand == "verify":
            if getattr(args, "manifest", None):
                return cmd_verify_manifest(args)
            res = verify_records(Path(args.old_record), Path(args.new_record), args.family)
            print(json.dumps(res, ensure_ascii=False))
            return 0
        if args.subcommand == "manifest":
            return cmd_manifest(args)
        if args.subcommand == "controls":
            if getattr(args, "manifest", None):
                return cmd_controls_manifest(args)
            if getattr(args, "family", None) == "composite":
                return cmd_controls_composite(args)
            return cmd_controls(args)
    except EvidenceError as exc:
        print(f"EVIDENCE ERROR: {exc}", file=sys.stderr)
        return 2
    except ParityError as exc:
        print(f"PARITY ERROR: {exc}", file=sys.stderr)
        return 1
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
