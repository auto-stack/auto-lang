#!/usr/bin/env python3
"""PLAN-1057 §6.1 & V-13: unit & negative controls for document_case_migration.py."""

from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "test-governance"))

import document_case_migration as dcm  # noqa: E402


def _sha(data: bytes) -> tuple[int, str]:
    return len(data), hashlib.sha256(data).hexdigest()


def _make_record_pair(tmp: Path, family: str) -> tuple[Path, Path]:
    spec = dcm.FAMILY_SPECS[family]
    pair_list = spec["expected_pairs"]
    records: dict[str, Path] = {}

    for side in ("old", "new"):
        run_root = tmp / f"{side}-out"
        run_dir = run_root / "run-1"
        run_dir.mkdir(parents=True, exist_ok=True)
        proj_checks = []
        actuals = {}
        for idx, (cid, kid) in enumerate(pair_list):
            src_bytes = f"// {cid}\nfn main() int {{ return 0 }}\n".encode("utf-8")
            src_sz, src_sha = _sha(src_bytes)
            combined_sha = hashlib.sha256(b"main.at\x00" + src_sha.encode() + b"\x00").hexdigest()
            is_console = family == "console"
            is_imm = family == "immutable"
            entry_mode = "script" if (is_imm and cid == "ace.m1.ia920.top") else "program"

            if is_console:
                ch_dir = run_dir / cid / kid
                ch_dir.mkdir(parents=True, exist_ok=True)
                rcp_file = ch_dir / "probe-receipt.json"
                if cid == "ace.m1.o07":
                    step = {
                        "op": "run",
                        "outcome": "success",
                        "value": None,
                        "channel": "unit",
                        "trap": None,
                        "diag": None,
                        "source_hash": src_sha[:16],
                        "console": {
                            "schema": "av.console-capture/1",
                            "stdout_b64": "",
                            "stderr_b64": "",
                            "stdout_truncated": False,
                            "stderr_truncated": False,
                        },
                    }
                    expect = {
                        "completion": {"kind": "success", "stage": None, "code": None, "message": None},
                        "exit": None,
                        "av_channel": "unit",
                        "result": None,
                        "stdout": {"compare": "bytes", "bytes": 0, "sha256": _sha(b"")[1], "text": ""},
                        "stderr": {"compare": "bytes", "bytes": 0, "sha256": _sha(b"")[1], "text": ""},
                        "trace": None,
                        "artifacts": [],
                        "perf": None,
                        "typed": None,
                        "probe": None,
                    }
                else:
                    av_ch = "value" if cid == "ace.m1.o08" else None
                    step = {
                        "op": "run",
                        "outcome": "success",
                        "value": 0,
                        "channel": "value",
                        "trap": None,
                        "diag": None,
                        "source_hash": src_sha[:16],
                        "console": {
                            "schema": "av.console-capture/1",
                            "stdout_b64": "",
                            "stderr_b64": "",
                            "stdout_truncated": False,
                            "stderr_truncated": False,
                        },
                    }
                    expect = {
                        "completion": {"kind": "success", "stage": None, "code": None, "message": None},
                        "exit": 0,
                        "av_channel": av_ch,
                        "result": None,
                        "stdout": {"compare": "bytes", "bytes": 0, "sha256": _sha(b"")[1], "text": ""},
                        "stderr": {"compare": "bytes", "bytes": 0, "sha256": _sha(b"")[1], "text": ""},
                        "trace": None,
                        "artifacts": [],
                        "perf": None,
                        "typed": None,
                        "probe": None,
                    }
                rcp_bytes = json.dumps({"recipe": "ac-av/console-run-v1", "protocol": "avworker/1", "step": step}).encode()
                rcp_file.write_bytes(rcp_bytes)
                rsz, rdig = _sha(rcp_bytes)
                adapter, recipe = "probe", "ac-av/console-run-v1"
                actuals[f"{cid}::{kid}"] = {
                    "case": cid,
                    "check": kid,
                    "status": "PASS",
                    "adapter": adapter,
                    "files": {"probe-receipt": {"path": str(rcp_file), "bytes": rsz, "sha256": rdig}},
                    "av": {
                        "outcome": step["outcome"],
                        "value": step["value"],
                        "channel": step["channel"],
                        "trap": step["trap"],
                        "diag": step["diag"],
                        "source_hash": step["source_hash"],
                        "console": {
                            "schema": "av.console-capture/1",
                            "stdout_truncated": False,
                            "stderr_truncated": False,
                            "stdout_bytes": 0,
                            "stdout_sha256": _sha(b"")[1],
                            "stderr_bytes": 0,
                            "stderr_sha256": _sha(b"")[1],
                        },
                    },
                }
            else:
                # Minimal scalar, composite or immutable fixture
                ch_dir = run_dir / cid / kid
                ch_dir.mkdir(parents=True, exist_ok=True)
                res_file = ch_dir / "result.txt"
                res_file.write_bytes(b"0")
                rsz, rdig = _sha(b"0")
                adapter, recipe = "avm", None
                if family in ("scalar", "composite"):
                    # Mirror the exact 4-adapter structure only minimally when testing scalar metadata
                    actuals[f"{cid}::{kid}"] = {
                        "case": cid,
                        "check": kid,
                        "status": "PASS",
                        "adapter": "avm",
                        "files": {"result": {"path": str(res_file), "bytes": rsz, "sha256": rdig}},
                        "avm": {"kind": "success", "result_text": "0"},
                    } if kid == "autovm" else (
                        {
                            "case": cid,
                            "check": kid,
                            "status": "PASS",
                            "adapter": "probe",
                            "files": {"mn-probe-receipt": {"path": str(res_file), "bytes": rsz, "sha256": rdig}},
                            "native": {
                                "report_exit_code": 0,
                                "receipt_exit_code": 0,
                                "session_exit_code": 0,
                                "judged": True,
                                "source_hash": src_sha,
                            },
                        } if kid == "ac-native" else (
                            {
                                "case": cid,
                                "check": kid,
                                "status": "PASS",
                                "adapter": "probe",
                                "files": {"probe-receipt": {"path": str(res_file), "bytes": rsz, "sha256": rdig}},
                                "av": {"outcome": "success", "value": 0, "channel": "value", "source_hash": src_sha[:16]},
                            } if kid == "av" else {
                                "case": cid,
                                "check": kid,
                                "status": "PASS",
                                "adapter": "a2r",
                                "files": {"artifact": {"path": str(res_file), "bytes": rsz, "sha256": rdig}},
                                "a2r": {"artifacts": []},
                            }
                        )
                    )
                else:
                    actuals[f"{cid}::{kid}"] = {
                        "case": cid,
                        "check": kid,
                        "status": "PASS",
                        "adapter": "avm",
                        "files": {"result": {"path": str(res_file), "bytes": rsz, "sha256": rdig}},
                        "avm": {"kind": "success", "result_text": "0"},
                    }
                expect = {
                    "completion": {"kind": "success", "stage": None, "code": None, "message": None},
                    "exit": 0 if kid in ("ac-native", "av") else None,
                    "av_channel": None,
                    "result": {"format": "text", "text": "0"} if kid == "autovm" else None,
                    "stdout": None,
                    "stderr": None,
                    "trace": None,
                    "artifacts": [],
                    "perf": None,
                    "typed": None,
                    "probe": None,
                }

            proj_checks.append(
                {
                    "semantic": {
                        "case": {
                            "id": cid,
                            "tags": ["ace-m1"],
                            "links": list(spec["required_links"]),
                            "entry_mode": entry_mode,
                            "status_skip": None,
                            "source": {
                                "syntax": "auto",
                                "virtual_path": "main.at",
                                "bytes": src_sz,
                                "sha256": src_sha,
                            },
                            "files": [],
                        },
                        "check": {
                            "id": kid,
                            "ordinal": idx,
                            "adapter": adapter,
                            "action": "run",
                            "tier": None,
                            "product_from": None,
                            "skip": None,
                        },
                        "options": {"probe_recipe": recipe},
                        "expect": expect,
                    },
                    "origin": {
                        "format": "document" if side == "new" else "fence",
                        "input_paths": [] if side == "new" else ["main.at"],
                        "source_sha256": combined_sha,
                        "oracle_sha256": "0" * 64,
                        "case_dir": cid,
                        "check_dir": kid,
                    },
                }
            )

        rep_file = run_root / "report.json"
        rep_bytes = b"{}"
        rep_file.write_bytes(rep_bytes)
        rep_sz, rep_dig = _sha(rep_bytes)
        rec = {
            "schema": dcm.RECORD_SCHEMA,
            "family": family,
            "side": side,
            "exit_code": 0,
            "report_path": str(rep_file),
            "report_bytes": rep_sz,
            "report_sha256": rep_dig,
            "run_dir": str(run_dir),
            "projection": {"schema": dcm.INVENTORY_SCHEMA, "checks": proj_checks},
            "actuals": actuals,
        }
        rec_path = tmp / f"{side}-{family}.record.json"
        rec_path.write_text(json.dumps(rec, indent=2), encoding="utf-8")
        records[side] = rec_path

    return records["old"], records["new"]


class DocumentCaseMigrationTests(unittest.TestCase):
    def test_valid_console_and_immutable_records_pass(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "console")
            res = dcm.verify_records(old_p, new_p, "console")
            self.assertEqual(res["verified_pairs"], 15)

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "immutable")
            res = dcm.verify_records(old_p, new_p, "immutable")
            self.assertEqual(res["verified_pairs"], 5)

    def test_v13_missing_and_duplicate_rows_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "immutable")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            # Drop NB18S
            mut["projection"]["checks"] = mut["projection"]["checks"][:-1]
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "immutable")

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "immutable")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            mut["projection"]["checks"].append(copy.deepcopy(mut["projection"]["checks"][0]))
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "immutable")

    def test_v13_top_entry_mode_and_o08_channel_loss_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "immutable")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            for row in mut["projection"]["checks"]:
                if row["semantic"]["case"]["id"] == "ace.m1.ia920.top":
                    row["semantic"]["case"]["entry_mode"] = "program"
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "immutable")

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "console")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            for row in mut["projection"]["checks"]:
                if row["semantic"]["case"]["id"] == "ace.m1.o08":
                    row["semantic"]["expect"]["av_channel"] = None
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "console")

    def test_v13_links_removal_and_skip_drift_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "scalar")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            for row in mut["projection"]["checks"]:
                if row["semantic"]["case"]["id"] == "ace.m1.b01":
                    row["semantic"]["case"]["links"] = []
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "scalar")

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "console")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            for row in mut["projection"]["checks"]:
                if row["semantic"]["case"]["id"] == "ace.m1.o08":
                    row["semantic"]["check"]["skip"] = "temporarily-disabled"
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "console")

    def test_v13_swapped_receipts_and_tampered_actual_rejected_exit2(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "console")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            a1 = mut["actuals"]["ace.m1.o01::av-console"]
            a2 = mut["actuals"]["ace.m1.o02::av-console"]
            a1["av"]["source_hash"], a2["av"]["source_hash"] = (
                a2["av"]["source_hash"],
                a1["av"]["source_hash"],
            )
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.EvidenceError):
                dcm.verify_records(old_p, new_p, "console")

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "console")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            rcp_path = Path(mut["actuals"]["ace.m1.o01::av-console"]["files"]["probe-receipt"]["path"])
            rcp_path.write_bytes(b"corrupted")
            with self.assertRaises(dcm.EvidenceError):
                dcm.verify_records(old_p, new_p, "console")

        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "immutable")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            Path(mut["run_dir"]).parent.joinpath("run-duplicate").mkdir()
            with self.assertRaises(dcm.EvidenceError):
                dcm.verify_records(old_p, new_p, "immutable")


# --- PLAN-1059 composite family: independent literals & rebound-actual tests ---

_LITERAL_COMPOSITE_CASES = [
    "ace.m1.k01", "ace.m1.k03", "ace.m1.k06",
    "ace.m1.r02", "ace.m1.r05", "ace.m1.r06", "ace.m1.r08",
    "ace.m1.t05",
    "ace.m1.v01", "ace.m1.v02", "ace.m1.v04", "ace.m1.v05", "ace.m1.v06", "ace.m1.v07",
]

_LITERAL_COMPOSITE_PAIRS = sorted(
    (case, check)
    for case in _LITERAL_COMPOSITE_CASES
    for check in ("autovm", "a2r-snapshot")
)

# The BASE-97 denominator as an independent literal: the 40 first-wave scalar
# document checks plus the 57 shared rows (8 extra scalar rows, this wave's 28
# composite rows, T07/K04, and the 6 session lifecycle rows).
_LITERAL_BASE_PAIRS = sorted(
    [(f"ace.m1.b{n:02d}", chk) for n in (1, 2, 3, 4, 5, 6, 7, 9, 10, 12) for chk in ("ac-native", "av", "autovm", "a2r-snapshot")]
    + [
        ("ace.m1.b13", "ac-native"), ("ace.m1.b13", "av"),
        ("ace.m1.b14", "ac-native"), ("ace.m1.b14", "av"), ("ace.m1.b14", "autovm"),
        ("ace.m1.b15", "ac-native"), ("ace.m1.b15", "av"),
        ("ace.m1.b20", "ac-native"), ("ace.m1.b20", "av"), ("ace.m1.b20", "autovm"),
        ("ace.m1.b16", "ac-native"), ("ace.m1.b16", "av"), ("ace.m1.b16", "autovm"),
        ("ace.m1.b17", "ac-native"), ("ace.m1.b17", "av"),
        ("ace.m1.b18", "ac-native"), ("ace.m1.b18", "av"),
        ("ace.m1.b19", "ac-native"), ("ace.m1.b19", "av"),
    ]
    + _LITERAL_COMPOSITE_PAIRS
    + [("ace.m1.t07", "autovm"), ("ace.m1.t07", "a2r-snapshot"),
       ("ace.m1.k04", "autovm"), ("ace.m1.k04", "a2r-snapshot")]
    + [("ace.m1.session.av", chk) for chk in ("warm-run", "hot-run", "body-edit-run", "edit-persist-across-workers")]
    + [("ace.m1.session.mn", chk) for chk in ("fresh-run", "edit-then-restart-run")]
)


class CompositeFamilyTests(unittest.TestCase):
    def test_fixed_collections_match_independent_literals(self):
        self.assertEqual(dcm.COMPOSITE_CASES, _LITERAL_COMPOSITE_CASES)
        self.assertEqual(len(_LITERAL_COMPOSITE_CASES), 14)
        self.assertEqual(sorted(dcm.COMPOSITE_MIGRATED_PAIRS), _LITERAL_COMPOSITE_PAIRS)
        self.assertEqual(len(_LITERAL_COMPOSITE_PAIRS), 28)
        spec = dcm.FAMILY_SPECS["composite"]
        self.assertEqual(sorted(spec["expected_pairs"]), _LITERAL_BASE_PAIRS)
        self.assertEqual(len(_LITERAL_BASE_PAIRS), 97)
        self.assertEqual(len(set(_LITERAL_BASE_PAIRS)), 97, "BASE denominator must not duplicate the 28")
        self.assertEqual(sorted(set(spec["migrated_pairs"]) - set(spec["expected_pairs"])), [])
        self.assertEqual(spec["required_links"], ["PLAN-852"])

    def test_composite_record_pair_verifies_97_28(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "composite")
            res = dcm.verify_records(old_p, new_p, "composite")
            self.assertEqual(res["verified_pairs"], 97)
            self.assertEqual(res["migrated_pairs"], 28)

    def test_composite_missing_v07_autovm_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = _make_record_pair(Path(td), "composite")
            mut = json.loads(new_p.read_text(encoding="utf-8"))
            mut["projection"]["checks"] = [
                row
                for row in mut["projection"]["checks"]
                if not (row["semantic"]["case"]["id"] == "ace.m1.v07" and row["semantic"]["check"]["id"] == "autovm")
            ]
            mut["actuals"].pop("ace.m1.v07::autovm")
            new_p.write_text(json.dumps(mut), encoding="utf-8")
            with self.assertRaises(dcm.ParityError) as ctx:
                dcm.verify_records(old_p, new_p, "composite")
            self.assertIn("ace.m1.v07", str(ctx.exception))

    def _swap_avm_actuals(self, td: Path, sync_cached_text: bool) -> tuple[Path, Path]:
        """Swap the R02/R06 VM actual file contents (0 <-> placeholder) with
        the recorded digests synced; the rebind judge must reject on content."""
        old_p, new_p = _make_record_pair(td, "composite")
        new_rec = json.loads(new_p.read_text(encoding="utf-8"))
        run_dir = Path(new_rec["run_dir"])
        r02 = new_rec["actuals"]["ace.m1.r02::autovm"]
        r06 = new_rec["actuals"]["ace.m1.r06::autovm"]
        r02_file = run_dir / "ace.m1.r02" / "autovm" / "result.txt"
        r06_file = run_dir / "ace.m1.r06" / "autovm" / "result.txt"
        self.assertEqual(r02_file.read_bytes(), b"0")
        r02_file.write_bytes(b"21")
        r06_file.write_bytes(b"9")
        for obs, blob in ((r02, b"21"), (r06, b"9")):
            obs["files"]["result"]["bytes"] = len(blob)
            obs["files"]["result"]["sha256"] = _sha(blob)[1]
            if sync_cached_text:
                obs["avm"]["result_text"] = blob.decode("utf-8")
        new_p.write_text(json.dumps(new_rec, indent=2), encoding="utf-8")
        return old_p, new_p

    def test_composite_avm_actual_swap_rejected_after_rebind(self):
        # NC-11 shape: files + digests + cached text all swapped consistently
        # (PASS labels kept): the re-read text no longer matches the frozen
        # expectation -> ParityError (exit 1), not a digest-pass stamp.
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = self._swap_avm_actuals(Path(td), sync_cached_text=True)
            with self.assertRaises(dcm.ParityError) as ctx:
                dcm.verify_records(old_p, new_p, "composite")
            self.assertIn("ace.m1.r02", str(ctx.exception))

    def test_composite_stale_cached_text_rejected_as_corrupt_binding(self):
        # Files + digests swapped but the cached result_text left stale: the
        # record derivation disagrees with its own file -> EvidenceError (2).
        with tempfile.TemporaryDirectory() as td:
            old_p, new_p = self._swap_avm_actuals(Path(td), sync_cached_text=False)
            with self.assertRaises(dcm.EvidenceError):
                dcm.verify_records(old_p, new_p, "composite")

    def test_composite_a2r_artifact_tampering_rejected(self):
        with tempfile.TemporaryDirectory() as td:
            td = Path(td)
            old_p, new_p = _make_record_pair(td, "composite")
            # Give the R05 a2r row a real artifact on both sides.
            art = b"// golden\nfn main() -> i64 {\n    return 42;\n}\n"
            art_r05 = None
            for side, path in (("old", old_p), ("new", new_p)):
                rec = json.loads(path.read_text(encoding="utf-8"))
                run_dir = Path(rec["run_dir"])
                stored = run_dir / "ace.m1.r05" / "a2r-snapshot" / "actual" / "m1_r05.rs"
                stored.parent.mkdir(parents=True, exist_ok=True)
                stored.write_bytes(art)
                asz, adig = _sha(art)
                for row in rec["projection"]["checks"]:
                    sem = row["semantic"]
                    if sem["case"]["id"] == "ace.m1.r05" and sem["check"]["id"] == "a2r-snapshot":
                        sem["expect"]["artifacts"] = [{"path": "m1_r05.rs", "bytes": asz, "sha256": adig, "compare": "bytes"}]
                obs = rec["actuals"]["ace.m1.r05::a2r-snapshot"]
                obs["a2r"]["artifacts"] = [{"path": "m1_r05.rs", "bytes": asz, "sha256": adig, "stored": str(stored)}]
                obs["files"]["artifact:m1_r05.rs"] = {"path": str(stored), "bytes": asz, "sha256": adig}
                if side == "new":
                    art_r05 = stored
                path.write_text(json.dumps(rec, indent=2), encoding="utf-8")
            # Baseline: the augmented pair still verifies.
            dcm.verify_records(old_p, new_p, "composite")
            # NC-12 shape: the actual file disappears -> exit-2 evidence error.
            art_r05.unlink()
            with self.assertRaises(dcm.EvidenceError) as ctx:
                dcm.verify_records(old_p, new_p, "composite")
            self.assertIn("ace.m1.r05", str(ctx.exception))
            # Tampered content with the cached digest synced -> exit-1 parity
            # error after digest recomputation (rebind catches the swap).
            art_r05.write_bytes(b"// tampered\n")
            new_rec = json.loads(new_p.read_text(encoding="utf-8"))
            obs = new_rec["actuals"]["ace.m1.r05::a2r-snapshot"]
            tsz, tdig = _sha(b"// tampered\n")
            obs["a2r"]["artifacts"][0]["bytes"] = tsz
            obs["a2r"]["artifacts"][0]["sha256"] = tdig
            obs["files"]["artifact:m1_r05.rs"]["bytes"] = tsz
            obs["files"]["artifact:m1_r05.rs"]["sha256"] = tdig
            new_p.write_text(json.dumps(new_rec, indent=2), encoding="utf-8")
            with self.assertRaises(dcm.ParityError):
                dcm.verify_records(old_p, new_p, "composite")


if __name__ == "__main__":
    unittest.main()
