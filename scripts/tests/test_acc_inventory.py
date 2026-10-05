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

    def test_head_commit_audit_field_does_not_trigger_drift(self):
        # 计划 §5.1：提交报告本身推进 HEAD 不得造成 --check 误报
        run_tool(self.repo, self.out, "--write")
        man = self.out / acc_inventory.MANIFEST_NAME
        data = json.loads(man.read_text(encoding="utf-8"))
        data["source_identity"]["head_commit_audit_only"] = "0" * 40  # 模拟 HEAD 前进
        man.write_text(json.dumps(data, ensure_ascii=False, indent=2, sort_keys=True) + "\n",
                       encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 0, err)

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
        # 可证明宿主 namespace → native-runtime
        self.assertEqual(aavm[("process", "args")], "native-runtime")
        self.assertEqual(aavm[("IO", "read_line")], "native-runtime")
        self.assertEqual(aavm[(None, "print")], "native-runtime")
        # QA-02：变量接收者不因方法名像内建就标 native（P743-QA-02）
        self.assertEqual(aavm[("head", "parse_int")], "unknown-receiver")
        self.assertEqual(aavm[("argv", "len")], "unknown-receiver")
        self.assertEqual(aavm[(None, "ev_run")], "imported-symbol")
        self.assertEqual(aavm[(None, "ev_run_files")], "imported-symbol")
        eng = kinds("auto/lib/engine.at")
        self.assertEqual(eng[(None, "ev_run")], "local-fn")
        # QA-02：frames 是变量，push 名不赋予 native 身份
        self.assertEqual(eng[("frames", "push")], "unknown-receiver")
        # 本地类型接收者 → type-qualified（不因 new 名误标 native）
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


class Qa01MultilineMasking(unittest.TestCase):
    """P743-QA-01：跨行字面量内容零污染；未闭合 EOF 标记词法不确定。"""

    @classmethod
    def setUpClass(cls):
        _spec2 = importlib.util.spec_from_file_location("ai_qa01", _TOOL_PATH)
        cls.ai = importlib.util.module_from_spec(_spec2)
        _spec2.loader.exec_module(cls.ai)

    def scan(self, text):
        return self.ai.scan_module("inline.at", text)

    def test_multiline_string_pseudocode_not_scanned(self):
        # 复审反例原文形态：pretend() 完全在跨行字符串内
        s = self.scan('fn host() {\n var text = "\n  pretend()\n  fn fake(str q) int\n "\n return 0\n}\n')
        names = [c["name"] for c in s.calls]
        self.assertNotIn("pretend", names)
        self.assertNotIn("fake", names)
        self.assertFalse(s.lexically_indeterminate)
        self.assertEqual(s.anomalies, [])
        self.assertEqual(s.capabilities.get("multiline-string-literal"), [2])

    def test_real_code_after_closing_quote_is_scanned(self):
        s = self.scan('var t = "\n fake_in_string()\n"\nreal_call()\n')
        names = [c["name"] for c in s.calls]
        self.assertNotIn("fake_in_string", names)
        self.assertIn("real_call", names)

    def test_escaped_newline_stays_in_literal(self):
        s = self.scan('var t = "a\\\n pretend_esc()\nb"\nreal_after()\n')
        names = [c["name"] for c in s.calls]
        self.assertNotIn("pretend_esc", names)
        self.assertIn("real_after", names)
        # 行号不因转义续行漂移：real_after 在物理第 4 行
        rc = [c for c in s.calls if c["name"] == "real_after"][0]
        self.assertEqual(rc["first_line"], 4)

    def test_unterminated_string_marks_indeterminate(self):
        s = self.scan('var t = "abc\ntail_call()\n')
        self.assertTrue(s.lexically_indeterminate)
        self.assertTrue(any(a["kind"] == "unterminated-string-at-eof" for a in s.anomalies))
        # EOF 兜底：尾部真实形态代码也被屏蔽，不生成候选
        self.assertNotIn("tail_call", [c["name"] for c in s.calls])

    def test_block_comment_with_string_like_content_stays_masked(self):
        s = self.scan('/* "unterminated note\n fn in_comment() */\nfn ok_fn() {}\n')
        self.assertIn("ok_fn", [d["name"] for d in s.declarations])
        self.assertNotIn("in_comment", [d["name"] for d in s.declarations])
        self.assertFalse(s.lexically_indeterminate)


class Qa02CallClassification(unittest.TestCase):
    """P743-QA-02：调用分类只认可证明证据，不按方法名猜 native。"""

    @classmethod
    def setUpClass(cls):
        _spec3 = importlib.util.spec_from_file_location("ai_qa02", _TOOL_PATH)
        cls.ai = importlib.util.module_from_spec(_spec3)
        _spec3.loader.exec_module(cls.ai)

    def kinds(self, text, path="inline.at"):
        s = self.ai.scan_module(path, text)
        return s, {(c["receiver"], c["name"]): c["kind"] for c in s.calls}

    def test_review_fixture_same_name_method(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "same-name-method.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        self.assertEqual(k[("Meter", "new")], "type-qualified")   # 本地类型，非宿主
        self.assertEqual(k[("x", "len")], "unknown-receiver")     # 变量接收者，不猜
        self.assertEqual(k[(None, "Meter")], "type-construction")

    def test_real_repo_cg_new_ar_new_not_native(self):
        root = _HERE.resolve().parents[1]
        for rel, ty in (("auto/lib/codegen.at", "CG"), ("auto/lib/a2r.at", "Ar")):
            s = self.ai.scan_module(rel, (root / rel).read_text(encoding="utf-8"))
            hits = [c for c in s.calls if c["name"] == "new" and c.get("receiver") == ty]
            self.assertTrue(hits, "%s.%s missing" % (ty, "new"))
            for c in hits:
                self.assertEqual(c["kind"], "type-qualified", (rel, c))

    def test_namespace_receivers_stay_native(self):
        s, k = self.kinds("fn f() {\n var a = List.new()\n var l = IO.read_line()\n}\n")
        self.assertEqual(k[("List", "new")], "native-runtime")
        self.assertEqual(k[("IO", "read_line")], "native-runtime")

    def test_implicit_self_dot_and_pipe_forms(self):
        s, k = self.kinds(
            "type P {\n fn next() int { return 1 }\n}\n"
            "fn walk() int {\n .next()\n return 2 |> .next()\n}\n")
        self.assertEqual(k[(None, "next")], "implicit-self-method")

    def test_unknown_dot_and_pipe_stay_unknown(self):
        s, k = self.kinds("fn walk() int {\n .mystery()\n return 1 |> .mystery()\n}\n")
        # 未升级的 dot/pipe 形态保留形态标记（"."/"|>"），类别保持 unknown
        self.assertEqual(k[(".", "mystery")], "unknown-receiver")
        self.assertEqual(k[("|>", "mystery")], "unknown-receiver")


if __name__ == "__main__":
    unittest.main()
