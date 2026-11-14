#!/usr/bin/env python3
"""PLAN-1057: AutoDown document-case first-wave parity, record & control verifier.

Subcommands (see PLAN-1057 §5.8, §6.2, §6.4):
  pair     --runner <auto-case> --old-suite <path> --new-suite <path>
           --family scalar|console|immutable --out <fresh-dir>
  verify   --old-record <record.json> --new-record <record.json>
           --family scalar|console|immutable
  controls --runner <auto-case> --out <fresh-dir>

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


def verify_actual_file_integrity(record: dict[str, Any], side_label: str) -> None:
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
        if not isinstance(files_meta, dict) or not files_meta:
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
                    "inline-document"
                    if c in set(FAMILY_SPECS[args.family]["migrated_cases"])
                    else "shared-legacy-include"
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


def main() -> int:
    parser = argparse.ArgumentParser(description="PLAN-1057 document case migration verifier")
    sub = parser.add_subparsers(dest="subcommand", required=True)

    p_pair = sub.add_parser("pair")
    p_pair.add_argument("--runner", required=True)
    p_pair.add_argument("--old-suite", required=True)
    p_pair.add_argument("--new-suite", required=True)
    p_pair.add_argument("--family", required=True, choices=sorted(FAMILY_SPECS.keys()))
    p_pair.add_argument("--out", required=True)

    p_ver = sub.add_parser("verify")
    p_ver.add_argument("--old-record", required=True)
    p_ver.add_argument("--new-record", required=True)
    p_ver.add_argument("--family", required=True, choices=sorted(FAMILY_SPECS.keys()))

    p_ctl = sub.add_parser("controls")
    p_ctl.add_argument("--runner", required=True)
    p_ctl.add_argument("--out", required=True)

    args = parser.parse_args()
    try:
        if args.subcommand == "pair":
            return cmd_pair(args)
        if args.subcommand == "verify":
            res = verify_records(Path(args.old_record), Path(args.new_record), args.family)
            print(json.dumps(res, ensure_ascii=False))
            return 0
        if args.subcommand == "controls":
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
