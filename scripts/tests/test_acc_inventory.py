#!/usr/bin/env python3
"""PLAN-743 盘点工具反例与失败路径测试。

验证的是清单可信度与失败路径（缺文件、漂移、伪代码污染、同名方法、
unknown 保留、人工结论过期、制品篡改、输出不安全、重跑确定性），
不是镜像扫描实现的逐字段断言。
"""
from __future__ import annotations

import importlib.util
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

_HERE = Path(__file__).resolve().parent
_TOOL_PATH = _HERE.parent / "acc_inventory.py"
_FIXTURE_REPO = _HERE / "fixtures" / "acc-inventory" / "repo"

_spec = importlib.util.spec_from_file_location("acc_inventory", _TOOL_PATH)
acc_inventory = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(acc_inventory)


def run_tool(root: Path, out: Path, *flags: str):
    r = subprocess.run(
        [sys.executable, str(_TOOL_PATH), "--root", str(root), "--output", str(out), *flags],
        capture_output=True, text=True, timeout=120,
    )
    return r.returncode, r.stdout, r.stderr


class FixtureCase(unittest.TestCase):
    """每个用例一份 fixture 仓库副本，互不污染。"""

    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="acc-inv-test-"))
        self.repo = self.tmp / "repo"
        shutil.copytree(_FIXTURE_REPO, self.repo)
        self.out = self.tmp / "out"
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)

    def write_decisions(self, bindings: dict, out: Path | None = None):
        out = out or self.out
        out.mkdir(parents=True, exist_ok=True)
        decisions = {
            "format_version": 1,
            "decisions": [
                {
                    "id": "MD-T-001",
                    "subject": "auto/lib/token.at",
                    "kind": "module-role",
                    "conclusion": "adapt",
                    "evidence": ["auto/lib/token.at:2"],
                    "bound_input_hashes": bindings,
                    "note": "test fixture decision",
                }
            ],
        }
        (out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(decisions, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")

    def current_bindings(self) -> dict:
        return {
            rel: acc_inventory.sha256_of(self.repo / rel)
            for rel in acc_inventory.MANAGED_INPUTS
        }

    def manifest(self) -> dict:
        return json.loads(
            (self.out / acc_inventory.MANIFEST_NAME).read_text(encoding="utf-8"))

    def module(self, rel: str) -> dict:
        for m in self.manifest()["observations"]["modules"]:
            if m["path"] == rel:
                return m
        self.fail("module %s not in manifest" % rel)


class WriteAndCheck(FixtureCase):
    def test_write_then_check_green_without_decisions(self):
        code, out, err = run_tool(self.repo, self.out, "--write")
        self.assertEqual(code, 0, err)
        self.assertTrue((self.out / acc_inventory.MANIFEST_NAME).is_file())
        self.assertTrue((self.out / acc_inventory.SUMMARY_NAME).is_file())
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 0, err)
        self.assertIn("decisions-absent", out)  # T-03 之前人工结论缺失=已知警告
        self.assertIn("CHECK-OK", out)

    def test_write_is_byte_deterministic(self):
        out2 = self.tmp / "out2"
        code1, _, err1 = run_tool(self.repo, self.out, "--write")
        code2, _, err2 = run_tool(self.repo, out2, "--write")
        self.assertEqual((code1, code2), (0, 0), err1 + err2)
        for name in (acc_inventory.MANIFEST_NAME, acc_inventory.SUMMARY_NAME):
            self.assertEqual(
                (self.out / name).read_bytes(), (out2 / name).read_bytes(),
                "%s not byte-stable across runs" % name)

    def test_manifest_covers_all_managed_inputs(self):
        code, _, err = run_tool(self.repo, self.out, "--write")
        self.assertEqual(code, 0, err)
        man = self.manifest()
        covered = {i["path"] for i in man["inputs"]}
        self.assertEqual(covered, set(acc_inventory.MANAGED_INPUTS))
        for i in man["inputs"]:
            self.assertEqual(i["sha256"], acc_inventory.sha256_of(self.repo / i["path"]))
        reg = man["registry"]
        self.assertTrue(reg["matches_tool_contract"])
        self.assertEqual(reg["module_order"],
                         ["token", "lexer", "parser", "typeinfo", "codegen", "engine", "a2r"])
        self.assertIn("head_commit_audit_only", man["source_identity"])

    def test_check_green_with_fresh_decisions(self):
        run_tool(self.repo, self.out, "--write")
        self.write_decisions(self.current_bindings())
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 0, err)
        self.assertIn("OK[decisions]", out)


class DriftAndFailurePaths(FixtureCase):
    def test_source_drift_fails_check(self):
        run_tool(self.repo, self.out, "--write")
        tok = self.repo / "auto/lib/token.at"
        tok.write_text(tok.read_text(encoding="utf-8") + "\n// drift\n", encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1, out)
        self.assertIn("manifest-stale", err)

    def test_registry_drift_blocks_write_and_check(self):
        lib_rs = self.repo / "crates/auto-lang/src/lib.rs"
        lib_rs.write_text(
            lib_rs.read_text(encoding="utf-8").replace(
                '"auto/lib/a2r.at",', '"auto/lib/a2r.at",\n    "auto/lib/extra.at",'),
            encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--write")
        self.assertEqual(code, 1, out)
        self.assertIn("registry-drift", err)
        self.assertFalse((self.out / acc_inventory.MANIFEST_NAME).is_file())
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1)
        self.assertIn("registry-drift", err)

    def test_missing_input_file_is_exit2_and_writes_nothing(self):
        (self.repo / "auto/lib/parser.at").unlink()
        code, out, err = run_tool(self.repo, self.out, "--write")
        self.assertEqual(code, 2, out)
        self.assertIn("missing-input", err)
        self.assertFalse((self.out / acc_inventory.MANIFEST_NAME).is_file())
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 2)

    def test_tampered_manifest_fails_check(self):
        run_tool(self.repo, self.out, "--write")
        man = self.out / acc_inventory.MANIFEST_NAME
        man.write_bytes(man.read_bytes().replace(b'"format_version": 1',
                                                 b'"format_version": 2'))
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1, out)
        self.assertIn("manifest-stale", err)

    def test_output_inside_source_dir_refused(self):
        for bad in ("auto", "auto/lib", "crates/auto-lang/src", "."):
            code, out, err = run_tool(self.repo, Path(bad), "--write")
            self.assertEqual(code, 2, "%s should be refused" % bad)
            self.assertIn("unsafe-output", err)

    def test_stale_decisions_fail_check(self):
        run_tool(self.repo, self.out, "--write")
        bindings = self.current_bindings()
        bindings["auto/lib/token.at"] = "0" * 64  # 过期绑定
        self.write_decisions(bindings)
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1, out)
        self.assertIn("decision-stale", err)

    def test_decisions_bound_to_removed_input_fail_check(self):
        run_tool(self.repo, self.out, "--write")
        bindings = self.current_bindings()
        bindings["auto/lib/retired.at"] = "1" * 64
        self.write_decisions(bindings)
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1, out)
        self.assertIn("decision-stale", err)

    def test_malformed_decisions_fail_check(self):
        run_tool(self.repo, self.out, "--write")
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text("{not json", encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 1, out)
        self.assertIn("decisions-malformed", err)


class ScanSemantics(FixtureCase):
    def test_comment_and_string_pseudocode_not_scanned(self):
        code, _, err = run_tool(self.repo, self.out, "--write")
        self.assertEqual(code, 0, err)
        raw = (self.out / acc_inventory.MANIFEST_NAME).read_text(encoding="utf-8")
        self.assertNotIn("fake_decl_in_block_comment", raw)
        self.assertNotIn("fake_in_string", raw)
        lexer = self.module("auto/lib/lexer.at")
        decl_names = [d["name"] for d in lexer["declarations"]]
        self.assertNotIn("fake_decl_in_block_comment", decl_names)
        for mod in self.manifest()["observations"]["modules"]:
            self.assertEqual(mod["brace_balance"], 0, mod["path"])
            self.assertEqual(mod["anomalies"], [], mod["path"])

    def test_same_name_methods_recorded_per_owner(self):
        run_tool(self.repo, self.out, "--write")
        eng = self.module("auto/lib/engine.at")
        lens = sorted(m["owner"] for m in eng["methods"] if m["name"] == "len")
        self.assertEqual(lens, ["Frame", "Scope"])

    def test_unknown_receiver_kept_as_candidate(self):
        run_tool(self.repo, self.out, "--write")
        cg = self.module("auto/lib/codegen.at")
        foo = [c for c in cg["call_candidates"]
               if c["kind"] == "unknown-receiver" and c["name"] == "foo"]
        self.assertEqual(len(foo), 1)
        self.assertGreater(foo[0]["first_line"], 0)

    def test_native_imported_and_local_call_classification(self):
        run_tool(self.repo, self.out, "--write")

        def kinds(rel):
            return {(c["receiver"], c["name"]): c["kind"]
                    for c in self.module(rel)["call_candidates"]}

        aavm = kinds("auto/aavm.at")
        self.assertEqual(aavm[("process", "args")], "native-runtime")
        self.assertEqual(aavm[("IO", "read_line")], "native-runtime")
        self.assertEqual(aavm[(None, "print")], "native-runtime")
        self.assertEqual(aavm[("head", "parse_int")], "native-runtime")
        self.assertEqual(aavm[(None, "ev_run")], "imported-symbol")
        self.assertEqual(aavm[(None, "ev_run_files")], "imported-symbol")
        eng = kinds("auto/lib/engine.at")
        self.assertEqual(eng[(None, "ev_run")], "local-fn")
        self.assertEqual(eng[("frames", "push")], "native-runtime")
        self.assertEqual(eng[("Frame", "greet")], "type-qualified")

    def test_capability_evidence_present(self):
        run_tool(self.repo, self.out, "--write")
        tok = self.module("auto/lib/token.at")
        self.assertIn("enum-declaration", tok["capability_evidence"])
        self.assertIn("is-match-statement", tok["capability_evidence"])
        lex = self.module("auto/lib/lexer.at")
        for key in ("record-type-declaration", "generic-list-type", "char-literal",
                    "logical-operator", "string-literal", "string-type-annotation"):
            self.assertIn(key, lex["capability_evidence"], key)
        par = self.module("auto/lib/parser.at")
        self.assertIn("while-loop", par["capability_evidence"])
        self.assertIn("mut-parameter", par["capability_evidence"])
        self.assertIn("comparison-operator", par["capability_evidence"])


if __name__ == "__main__":
    unittest.main()
