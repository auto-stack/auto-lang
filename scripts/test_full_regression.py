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
                         mock.patch.object(full, "tier_minimums", return_value={}), \
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

    def run_queue_control(self, child_code, scope_empty=False, man_zero=False, minimums=None):
        with tempfile.TemporaryDirectory() as tmp:
            directory = Path(tmp)
            # Reuse real inventories while substituting only process execution.
            data = {"rust-suites": {"owner": {"testcases": {
                "x": {"ignored": False, "filter-match": {"status": "mismatch" if scope_empty else "matches"}}
            }}}}
            class Listed:
                def __init__(self, cmd, **kwargs):
                    if "--list" in cmd and cmd[1:2] == ["test"]:  # libtest listing (ui-tu/ui-tn)
                        kwargs["stdout"].write("ui_gen::sample: test\nui_gen::ignored_one: ignore\n")
                        return
                    if man_zero and "--workspace" in cmd:  # man-t face emptied
                        json.dump({"rust-suites": {}}, kwargs["stdout"])
                        return
                    json.dump(data, kwargs["stdout"])
                def wait(self, **kwargs):
                    return 0
            with mock.patch.object(full, "root", return_value=directory), \
                 mock.patch.object(full, "machine_gate", contextlib.nullcontext), \
                 mock.patch.object(full, "validate_layout"), \
                 mock.patch.object(full, "require_main"), \
                 mock.patch.object(full, "identity", return_value={"head": "frozen"}), \
                 mock.patch.object(full, "tier_minimums", return_value=minimums or {}), \
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
                return receipt
            if man_zero or minimums:
                return receipt
            self.assertEqual(result, 1 if child_code else 0)
            self.assertEqual(receipt["outcome"], "fail" if child_code else "pass")
            if not child_code:
                # language, ui-full, tt, tb + man-t/tcore/tc/ti + ui-tu/ti/tn (PLAN-867)
                self.assertEqual([row["leg"] for row in receipt["results"]],
                                 ["language", "ui-full", "tt", "tb", "man-t", "man-tcore",
                                  "man-tc", "man-ti", "ui-tu", "ui-ti", "ui-tn"])
                self.assertIn("manual native gate", receipt["results"][-1].get("skip", ""))
            self.assertEqual(receipt["results"][0]["run_exit"], child_code)
            # bootstrap 2 + language/ui-full/tt/tb 4 + man 4 + ui-tu 1 + ui-ti 1 (ui-tn manual pending)
            self.assertEqual(child.call_count, 1 if child_code else 12)
            return receipt

    def test_first_child_failure_is_receipted_and_stops_queue(self):
        receipt = self.run_queue_control(7)
        self.assertEqual(len(receipt["results"]), 1)
        self.assertIn("tests failed", receipt["error"])

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

    def test_plan841_migration_preserves_selection_and_rejects_lost_targets(self):
        migrations = full.migrations()
        self.assertEqual(len(migrations), 21)
        self.assertEqual(len({m["old"] for m in migrations}), 21)
        self.assertEqual(len({m["new"] for m in migrations}), 21)
        for migration in migrations:
            owner = "language" if migration["source"] == "language" else "ui"
            original_ids, original_required = full.raw_baseline(owner)
            self.assertIn(migration["old"], original_ids)
            self.assertIn(migration["old"], original_required)
        for tier in ("tf", "taa", "ta", "t3"):
            for label in ("language", "ui-full"):
                expected, required = full.baseline_for(tier, label)
                for migration in migrations:
                    if migration["target"] != label:
                        continue
                    target = migration["new"]
                    self.assertIn(target, expected, (tier, label, target))
                    self.assertIn(target, required, (tier, label, target))
                    with self.assertRaisesRegex(ValueError, "baseline coverage lost"):
                        full.validate_ids(expected - {target}, required, expected, required)
                    with self.assertRaisesRegex(ValueError, "baseline coverage lost"):
                        full.validate_ids(expected, required - {target}, expected, required)
        for label in ("tt", "tb"):
            expected, required = full.baseline_for("tf", label)
            for migration in migrations:
                if migration["source"] == "language":
                    self.assertNotIn(migration["old"], expected)
                    if migration["target"] == "language":
                        self.assertIn(migration["new"], required)

    def test_migrated_recipe_target_is_in_every_full_owner_selection(self):
        for tier in ("tf", "taa", "ta", "t3"):
            ui = full.legs(tier, Path("language"), Path("ui"))[1][2]
            self.assertIn("--lib", ui)
            self.assertIn("--test", ui)
            self.assertIn("design_tokens_recipe", ui)

    def test_full_owner_commands_and_batch_order(self):
        rows = full.legs("tf", Path("language"), Path("ui"), True)
        self.assertEqual([row[0] for row in rows], ["language", "ui-full", "tt", "tb"])
        ui = rows[1][2]
        for owner in full.UI_PACKAGES:
            self.assertIn(owner, ui)
        self.assertIn("auto-ui-compiler/ui", ui)
        self.assertIn("language/.config/nextest-full.toml", [str(v).replace('\\', '/') for v in ui])

    def test_batch_extra_legs_fixed_order_and_kinds(self):
        rows = full.batch_extra_legs(Path("language"), Path("ui"), Path("man"))
        self.assertEqual([row["leg"] for row in rows], ["man-t", "man-tcore", "man-tc", "man-ti", "ui-tu", "ui-ti", "ui-tn"])
        self.assertEqual([row["kind"] for row in rows], ["nextest", "nextest", "nextest", "nextest", "libtest", "nextest", "libtest"])
        self.assertTrue(rows[-1]["manual"])
        self.assertIn("--manifest-path", " ".join(rows[1]["options"]))
        self.assertIn("core/Cargo.toml", rows[1]["options"])
        self.assertEqual(rows[4]["package"], "auto-ui-compiler")
        self.assertEqual(rows[6]["package"], "auto-ui-runtime")

    def test_zero_selection_in_man_leg_is_not_green(self):
        receipt = self.run_queue_control(0, man_zero=True)
        self.assertEqual(receipt["outcome"], "fail")
        self.assertIn("zero selected", receipt["error"])
        failed = [row["leg"] for row in receipt["results"] if row["leg"] == "man-t"]
        self.assertEqual(failed, ["man-t"])

    def test_minimum_selection_floor_is_enforced_in_batch(self):
        receipt = self.run_queue_control(0, minimums={"man-t": {"minimum": 281}})
        self.assertEqual(receipt["outcome"], "fail")
        self.assertIn("selection below frozen minimum", receipt["error"])

    def test_frozen_trees_rejects_code_wip_and_accepts_clean(self):
        with tempfile.TemporaryDirectory() as tmp:
            tree = Path(tmp)
            (tree / "Cargo.toml").write_text("")
            git = mock.patch.object(full.subprocess, "check_output")
            with git as check:
                check.return_value = b"crates/wip.rs"
                with self.assertRaisesRegex(ValueError, "code/configuration WIP"):
                    full.require_clean_tree("language", tree)
                check.return_value = b""
                full.require_clean_tree("language", tree)

    def test_libtest_selection_counts_ignored_not_selected(self):
        text = "ui_gen::a: test\nui_gen::b: test\nui_gen::c: ignore\n"
        ids, selected = full.libtest_selection(text, "auto-ui-compiler")
        self.assertEqual(ids, {"auto-ui-compiler\tui_gen::a", "auto-ui-compiler\tui_gen::b", "auto-ui-compiler\tui_gen::c"})
        self.assertEqual(selected, {"auto-ui-compiler\tui_gen::a", "auto-ui-compiler\tui_gen::b"})
        with self.assertRaises(ValueError):
            full.libtest_selection("", "auto-ui-compiler")

    def test_failed_ids_extracted_from_nextest_and_libtest_logs(self):
        with tempfile.TemporaryDirectory() as tmp:
            nextest_log = Path(tmp) / "nextest.log"
            nextest_log.write_text("        FAIL [ 1.23s] auto-lang ui::tests::broken\n   Abort", encoding="utf-8")
            self.assertEqual(full.failed_from_log(nextest_log), ["auto-lang\tui::tests::broken"])
            libtest_log = Path(tmp) / "libtest.log"
            libtest_log.write_text("test ui_gen::vue::tests::bad ... FAILED\n1 passed", encoding="utf-8")
            self.assertEqual(full.failed_from_log(libtest_log, "auto-ui-compiler"),
                             ["auto-ui-compiler\tui_gen::vue::tests::bad"])
            self.assertEqual(full.failed_from_log(Path(tmp) / "missing.log"), [])

    def test_frozen_trees_flag_requires_batch(self):
        with tempfile.TemporaryDirectory() as tmp:
            with mock.patch.object(full, "root", return_value=Path(tmp)), \
                 mock.patch.object(full, "machine_gate", contextlib.nullcontext), \
                 mock.patch.object(full, "validate_layout"):
                with self.assertRaises(SystemExit):
                    full.main(["tf", "--frozen-trees", "--list-only", "--output", str(Path(tmp) / "out")])


if __name__ == "__main__":
    unittest.main()
