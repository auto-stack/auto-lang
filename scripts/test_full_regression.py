"""Focused orchestration controls, never invokes a Cargo test tier."""
import importlib.util
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest import mock
import json
import mmap
import contextlib
import os

spec = importlib.util.spec_from_file_location("full", Path(__file__).with_name("run-full-regression.py"))
full = importlib.util.module_from_spec(spec)
spec.loader.exec_module(full)


class Controls(unittest.TestCase):
    def test_feature_only_ids_are_guarded_in_every_formal_child(self):
        common, _ = full.baseline_for("tf", "language")
        for tier in ("tt", "tb", "taa", "ta", "t3"):
            formal = "tf" if tier in ("tt", "tb") else tier
            target = tier if formal == "tf" else "language"
            expected, required = full.baseline_for(formal, target)
            victim = sorted(required - common)[0]
            for mutation in ("missing", "ignored", "profile"):
                with self.subTest(tier=tier, mutation=mutation), tempfile.TemporaryDirectory() as tmp:
                    directory = Path(tmp)
                    class Listed:
                        def __init__(self, cmd, **kwargs):
                            label = "ui-full" if "auto-ui-compiler" in cmd else "language"
                            if label == "language" and formal == "tf" and "--features" in cmd:
                                label = "tt" if "auto-lang/test-trans,auto-lang/test-cases" in cmd else "tb"
                            ids, selected = full.baseline_for(formal, label)
                            suites = {}
                            for identity in ids:
                                suite, name = identity.split("\t", 1)
                                suites.setdefault(suite, {"testcases": {}})["testcases"][name] = {
                                    "ignored": False,
                                    "filter-match": {"status": "matches" if identity in selected else "mismatch"}}
                            if label == target:
                                suite, name = victim.split("\t", 1)
                                if mutation == "missing":
                                    del suites[suite]["testcases"][name]
                                elif mutation == "ignored":
                                    suites[suite]["testcases"][name]["ignored"] = True
                                else:
                                    suites[suite]["testcases"][name]["filter-match"]["status"] = "mismatch"
                            json.dump({"rust-suites": suites}, kwargs["stdout"])
                        def wait(self, **kwargs):
                            return 0
                    with mock.patch.object(full, "root", return_value=directory), \
                         mock.patch.object(full, "machine_gate", contextlib.nullcontext), \
                         mock.patch.object(full, "validate_layout"), \
                         mock.patch.object(full, "require_main"), \
                         mock.patch.object(full, "identity", return_value={"head": "frozen"}), \
                         mock.patch.object(full.subprocess, "Popen", Listed), \
                         mock.patch.object(full, "command", return_value=0):
                        result = full.main([formal, *(["--batch"] if formal == "tf" else []), "--output", str(directory / "out")])
                    receipt = json.loads((directory / "out/receipt.json").read_text())
                    self.assertEqual(result, 1)
                    self.assertEqual(receipt["results"][-1]["leg"], target)
                    self.assertIn("baseline coverage lost", receipt["error"])

    def test_full_execution_rejects_worktree_and_main_code_wip(self):
        main = Path("main").resolve()
        with mock.patch.object(full.subprocess, "check_output", return_value="worktree " + str(main)):
            with self.assertRaisesRegex(ValueError, "main-checkout only"):
                full.require_main(main.parent / "worktree")
        with mock.patch.object(full.subprocess, "check_output", side_effect=["worktree " + str(main), "master", b"crates/wip.rs"]):
            with self.assertRaisesRegex(ValueError, "code/configuration WIP"):
                full.require_main(main)
        with mock.patch.object(full.subprocess, "check_output", side_effect=["worktree " + str(main), "master", b""]):
            full.require_main(main)
    def test_invalid_explicit_root_does_not_fall_back(self):
        with mock.patch.dict(full.os.environ, {"AUTO_UI_ROOT": "Z:/missing-plan850-owner"}):
            with self.assertRaises(ValueError):
                full.root("auto-ui")

    def test_reverse_sdk_owner_mismatch_is_rejected(self):
        lang = Path("language").resolve()
        ui = Path("ui").resolve()
        language = {"packages": [
            {"name": "auto-lang", "manifest_path": str(lang / "crates/auto-lang/Cargo.toml"),
             "dependencies": [{"name": "auto-ui-compiler", "path": str(ui / "crates/auto-ui-compiler")}]},
            {"name": "auto-lang-core", "manifest_path": str(lang / "crates/auto-lang-core/Cargo.toml")}
        ]}
        ui_data = {"packages": [{"name": "auto-ui-compiler", "dependencies": [
            {"name": "auto-lang-core", "path": str(lang / "crates/auto-lang-core")}
        ]}]}
        with mock.patch.object(full.subprocess, "check_output", side_effect=[json.dumps(language), json.dumps(ui_data)]):
            full.validate_layout(lang, ui)
        ui_data["packages"][0]["dependencies"][0]["path"] = "different-language/crates/auto-lang-core"
        with mock.patch.object(full.subprocess, "check_output", side_effect=[json.dumps(language), json.dumps(ui_data)]):
            with self.assertRaisesRegex(ValueError, "language SDK owner mismatch"):
                full.validate_layout(lang, ui)

    def run_queue_control(self, child_code, scope_empty=False):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            # Reuse real inventories while substituting only process execution.
            data = {"rust-suites": {"owner": {"testcases": {
                "x": {"ignored": False, "filter-match": {"status": "mismatch" if scope_empty else "matches"}}
            }}}}
            class Listed:
                def __init__(self, cmd, **kwargs):
                    json.dump(data, kwargs["stdout"])
                def wait(self, **kwargs):
                    return 0
            with mock.patch.object(full, "root", return_value=directory), \
                 mock.patch.object(full, "machine_gate", contextlib.nullcontext), \
                 mock.patch.object(full, "validate_layout"), \
                 mock.patch.object(full, "require_main"), \
                 mock.patch.object(full, "identity", return_value={"head": "frozen"}), \
                 mock.patch.object(full.subprocess, "Popen", Listed), \
                 mock.patch.object(full, "validate_ids", return_value=[]), \
                 mock.patch.object(full, "command", return_value=child_code) as child:
                result = full.main(["tf", "never-matches" if scope_empty else "--batch", "--output", str(directory / "out")])
            receipt = json.loads((directory / "out/receipt.json").read_text())
            if scope_empty:
                self.assertEqual(result, 1)
                self.assertEqual(receipt["outcome"], "fail")
                self.assertEqual(len(receipt["results"]), 2)
                self.assertIn("zero selected", receipt["error"])
                child.assert_not_called()
                return
            self.assertEqual(result, 1 if child_code else 0)
            self.assertEqual(receipt["outcome"], "fail" if child_code else "pass")
            self.assertEqual(len(receipt["results"]), 1 if child_code else 4)
            self.assertEqual(receipt["results"][0]["run_exit"], child_code)
            self.assertEqual(child.call_count, 1 if child_code else 4)

    def test_first_child_failure_is_receipted_and_stops_queue(self):
        self.run_queue_control(7)

    def test_all_four_children_success_is_green(self):
        self.run_queue_control(0)

    def test_zero_selection_across_scoped_owners_is_not_green(self):
        self.run_queue_control(0, scope_empty=True)

    def test_report_wip_survives_failure_and_locked_report_fails_closed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            report = root / "docs/reports/plan796/turnaround-ipc.json"
            report.parent.mkdir(parents=True)
            report.write_bytes(b"user WIP original")
            out = root / "receipt"
            out.mkdir()
            with report.open("r+b") as handle, mmap.mmap(handle.fileno(), 0) as mapped:
                if sys.platform == "win32":
                    with self.assertRaises(OSError):
                        with full.preserve_reports(root, out):
                            self.fail("locked report must reject before executing tests")
                self.assertEqual(report.read_bytes(), b"user WIP original")
                self.assertEqual(mapped[:], b"user WIP original")
            with self.assertRaises(RuntimeError):
                with full.preserve_reports(root, out):
                    report.write_bytes(b"generated")
                    raise RuntimeError("test child failed")
            self.assertEqual(report.read_bytes(), b"user WIP original")
            self.assertEqual((out / "report-0-generated").read_bytes(), b"generated")

    def test_matrix_and_inventory_reports_restore_even_after_partial_prepare(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            names = ["docs/reports/plan796/turnaround-ipc.json",
                     "docs/plans/reports/738-stdlib-matrix.json",
                     "docs/plans/reports/738-stdlib-matrix.md",
                     "docs/reports/plan827/inventory.json"]
            paths = [root / name for name in names]
            for index, path in enumerate(paths):
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(f"user WIP {index}".encode())
            originals = [path.read_bytes() for path in paths]
            out = root / "receipt"
            out.mkdir()
            with paths[1].open("r+b") as handle, mmap.mmap(handle.fileno(), 0) as mapped:
                if sys.platform == "win32":
                    with self.assertRaises(OSError):
                        with full.preserve_reports(root, out):
                            self.fail("mapped matrix must reject before executing tests")
                self.assertEqual([path.read_bytes() for path in paths], originals)
                self.assertEqual(mapped[:], originals[1])
            with self.assertRaises(RuntimeError):
                with full.preserve_reports(root, out):
                    for index, path in enumerate(paths):
                        path.write_bytes(f"generated {index}".encode())
                    raise RuntimeError("test child failed")
            self.assertEqual([path.read_bytes() for path in paths], originals)
            generated = {path.read_bytes() for path in out.glob("*-generated")}
            self.assertEqual(generated, {f"generated {i}".encode() for i in range(4)})

    def test_selection_zero_and_ignored_are_not_green(self):
        for case in ({}, {"x": {"ignored": True, "filter-match": {"status": "matches"}}},
                     {"x": {"ignored": False, "filter-match": {"status": "mismatch"}}}):
            with self.assertRaises(ValueError):
                full.selection({"rust-suites": {"owner": {"testcases": case}}})

    def test_missing_owner_and_filtered_baseline_are_rejected(self):
        expected = {"language\tx", "ui\ty"}
        self.assertEqual(full.validate_ids(expected, expected, expected, expected), [])
        with self.assertRaises(ValueError):
            full.validate_ids({"language\tx"}, {"language\tx"}, expected, expected)
        with self.assertRaises(ValueError):
            full.validate_ids(expected, {"language\tx"}, expected, expected)

    def test_real_child_exit_and_timeout(self):
        with tempfile.TemporaryDirectory() as tmp:
            log = Path(tmp) / "log"
            self.assertEqual(full.command([sys.executable, "-c", "pass"], tmp, log, 5), 0)
            self.assertEqual(full.command([sys.executable, "-c", "raise SystemExit(7)"], tmp, log, 5), 7)
            with self.assertRaises(subprocess.TimeoutExpired):
                full.command([sys.executable, "-c", "import time; time.sleep(30)"], tmp, log, .1)

    def test_interruption_reaps_child_and_propagates(self):
        children = []
        real_popen = subprocess.Popen
        class Interrupted(real_popen):
            interrupted = False
            def wait(self, *args, **kwargs):
                if self.args[0] == sys.executable and not self.interrupted:
                    self.interrupted = True
                    children.append(self)
                    raise KeyboardInterrupt()
                return super().wait(*args, **kwargs)
        with tempfile.TemporaryDirectory() as tmp, mock.patch.object(full.subprocess, "Popen", Interrupted):
            with self.assertRaises(KeyboardInterrupt):
                full.command([sys.executable, "-c", "import time; time.sleep(30)"], tmp, Path(tmp) / "log", 5)
        self.assertEqual(len(children), 1)
        self.assertIsNotNone(children[0].poll())

    @unittest.skipUnless(sys.platform == "win32", "Windows task-tree cleanup control")
    def test_timeout_reaps_real_grandchild(self):
        with tempfile.TemporaryDirectory() as tmp:
            pidfile = Path(tmp) / "pids"
            code = ("import os,subprocess,sys,time,pathlib; "
                    "p=subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); "
                    "pathlib.Path(" + repr(str(pidfile)) + ").write_text(str(os.getpid())+','+str(p.pid)); time.sleep(30)")
            with self.assertRaises(subprocess.TimeoutExpired):
                full.command([sys.executable, "-c", code], tmp, Path(tmp) / "log", 2)
            for pid in pidfile.read_text().split(','):
                listing = subprocess.check_output(["tasklist", "/FI", f"PID eq {pid}", "/FO", "CSV", "/NH"])
                self.assertNotIn(('"' + pid + '"').encode(), listing)

    def test_machine_gate_rejects_competing_process(self):
        name = "AutoLangBatchRegressionTest-" + str(os.getpid())
        code = ("import importlib.util; s=importlib.util.spec_from_file_location('f', "
                + repr(str(Path(full.__file__))) + "); m=importlib.util.module_from_spec(s);"
                "s.loader.exec_module(m);\nwith m.machine_gate(" + repr(name) + "): print('unexpected admission')")
        with full.machine_gate(name):
            child = subprocess.run([sys.executable, "-c", code], capture_output=True, timeout=5)
            self.assertNotEqual(child.returncode, 0)
            self.assertNotIn(b"unexpected admission", child.stdout)

    def test_full_owner_commands_and_batch_order(self):
        rows = full.legs("tf", Path("language"), Path("ui"), True)
        self.assertEqual([row[0] for row in rows], ["language", "ui-full", "tt", "tb"])
        ui = rows[1][2]
        for owner in full.UI_PACKAGES:
            self.assertIn(owner, ui)
        self.assertIn("auto-ui-compiler/ui", ui)
        self.assertIn("language/.config/nextest-full.toml", [str(v).replace('\\', '/') for v in ui])


if __name__ == "__main__":
    unittest.main()
