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

    def write_layer(self, layer):
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")

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
        # r3 R2-QA-01：正例必须在 enclosing owner（P）方法体内
        s, k = self.kinds(
            "type P {\n fn next() int {\n  .next()\n  return 2 |> .next()\n }\n}\n")
        self.assertEqual(k[(None, "next")], "implicit-self-method")

    def test_review_fixture_shadowed_host(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "shadowed_host.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        # 本地 fn print / 本地 type IO 遮蔽宿主内建（r3 R2-QA-01）
        self.assertEqual(k[(None, "print")], "local-fn")
        self.assertEqual(k[("IO", "read_line")], "type-qualified")

    def test_review_fixture_different_owner(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "different_owner.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        # .next() 在 Q 方法体内，next 声明于 P——非当前 owner 不升级
        self.assertEqual(k[(".", "next")], "unknown-receiver")

    def test_review_fixture_unrelated_dot_pipe(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "unrelated_dot_pipe.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        # 自由函数内的点/管道形态无 enclosing owner → 保持 unknown
        self.assertEqual(k[(".", "next")], "unknown-receiver")
        self.assertEqual(k[("|>", "next")], "unknown-receiver")

    def test_unknown_dot_and_pipe_stay_unknown(self):
        s, k = self.kinds("fn walk() int {\n .mystery()\n return 1 |> .mystery()\n}\n")
        # 未升级的 dot/pipe 形态保留形态标记（"."/"|>"），类别保持 unknown
        self.assertEqual(k[(".", "mystery")], "unknown-receiver")
        self.assertEqual(k[("|>", "mystery")], "unknown-receiver")


class StrictDecisionsGate(FixtureCase):
    """P743-QA-03：人工层完整性校验与 --require-decisions 严格完成态门。"""

    def complete_layer(self, out: Path | None = None):
        """按 fixture 当前扫描构造一份完整合法的人工层（含 unknown_families）。"""
        out = out or self.out
        out.mkdir(parents=True, exist_ok=True)
        import importlib.util as _ilu
        _s = _ilu.spec_from_file_location("ai_strict", _TOOL_PATH)
        _ai = _ilu.module_from_spec(_s)
        _s.loader.exec_module(_ai)
        scans = {rel: _ai.scan_module(rel, (self.repo / rel).read_text(encoding="utf-8"))
                 for rel in _ai.SOURCE_MODULES}
        unknowns = _ai._unknown_candidates_from_scans(scans)
        bindings = {rel: _ai.sha256_of(self.repo / rel)
                    for rel in _ai.MANAGED_INPUTS}
        by_module = {}
        for (p, k, n) in unknowns:
            by_module.setdefault(p, []).append(n)
        decisions = [
            {"id": "MD-S-%03d" % i, "subject": rel, "kind": "module-role",
             "conclusion": "adapt", "evidence": ["%s:1" % rel], "note": "t",
             "bound_input_hashes": {rel: bindings[rel]}}
            for i, rel in enumerate(_ai.MANAGED_INPUTS)
        ]
        mod_paths = sorted(by_module)
        # 族引用的 unknown-resolution 决定：证据覆盖各族路径（适用性规则，r3）
        decisions.append({
            "id": "MD-S-900", "subject": "fixture-unknown-families",
            "kind": "unknown-resolution", "conclusion": "resolved",
            "evidence": ["%s:1" % p for p in mod_paths],
            "note": "fixture unknown disposition",
            "bound_input_hashes": {p: bindings[p] for p in mod_paths},
        })
        fams = [{"path": p, "kind": "unknown-receiver", "names": "*",
                 "disposition": "resolved", "decision": "MD-S-900",
                 "note": "fixture family"}
                for p in mod_paths]
        layer = {"format_version": 1, "decisions": decisions,
                 "unknown_families": fams}
        (out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
        return layer

    def test_strict_gate_green_with_complete_layer(self):
        run_tool(self.repo, self.out, "--write")
        self.complete_layer()
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 0, err)
        self.assertIn("严格门覆盖闭环通过", out)

    def test_strict_gate_fails_when_layer_absent(self):
        run_tool(self.repo, self.out, "--write")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decisions-required", err)

    def test_strict_gate_fails_on_empty_decisions(self):
        run_tool(self.repo, self.out, "--write")
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps({"format_version": 1, "decisions": []}), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("非空数组", err)

    def test_unbound_decision_fails_with_id(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][0]["bound_input_hashes"] = {}
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("bound_input_hashes", err)
        self.assertIn(layer["decisions"][0]["id"], err)

    def test_missing_evidence_file_fails_with_id(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][0]["evidence"] = ["auto/lib/nope.at:1"]
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-evidence-missing", err)
        self.assertIn(layer["decisions"][0]["id"], err)

    def test_duplicate_ids_fail(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][1]["id"] = layer["decisions"][0]["id"]
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-duplicate-id", err)

    def test_illegal_conclusion_fails(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][0]["conclusion"] = "super-done"
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("conclusion", err)

    def test_uncovered_unknown_candidate_fails(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["unknown_families"] = layer["unknown_families"][:-1]
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("coverage-missing-unknown", err)

    def test_orphan_family_fails(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["unknown_families"].append(
            {"path": "auto/lib/token.at", "kind": "unknown-receiver", "names": "*",
             "disposition": "resolved", "decision": "MD-S-000"})
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-orphan", err)

    def test_open_family_requires_owner_probe_workpackage(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["unknown_families"][0]["disposition"] = "open"
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-open-incomplete", err)

    def test_uncovered_managed_input_fails(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"] = [d for d in layer["decisions"]
                              if d["subject"] != "auto/lib/token.at"]
        self.out.mkdir(parents=True, exist_ok=True)
        (self.out / acc_inventory.DECISIONS_NAME).write_text(
            json.dumps(layer, ensure_ascii=False), encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("coverage-missing-input", err)

    def test_scan_only_mode_survives_without_flag_but_not_ok(self):
        run_tool(self.repo, self.out, "--write")
        code, out, err = run_tool(self.repo, self.out, "--check")
        self.assertEqual(code, 0)
        self.assertIn("scan-only", out)
        self.assertNotIn("人工层 ok", out)

    def test_strict_flag_rejected_with_write(self):
        code, out, err = run_tool(self.repo, self.out, "--write", "--require-decisions")
        self.assertEqual(code, 2)
        self.assertIn("usage", err)


class IntegrityHardening(StrictDecisionsGate):
    """P743-R2-QA-02/03/04：证据同条绑定、族引用适用性、字段类型受控拒绝。"""

    def test_evidence_without_binding_rejected_then_fixed_then_stale(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        # 追加仓内非扫描输入证据但不同条绑定（R2-QA-02 反例）
        (self.repo / "review-evidence.txt").write_text("anchor\n", encoding="utf-8")
        layer["decisions"][0]["evidence"].append("review-evidence.txt:1")
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-evidence-unbound", err)
        self.assertIn(layer["decisions"][0]["id"], err)
        # 补上同条绑定 → 通过
        layer["decisions"][0]["bound_input_hashes"]["review-evidence.txt"] = \
            acc_inventory.sha256_of(self.repo / "review-evidence.txt")
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 0, err)
        # 证据内容变化 → stale（绑定在先的证据同样受控）
        (self.repo / "review-evidence.txt").write_text("changed\n", encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-stale", err)

    def test_resolved_family_without_decision_ref_rejected(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        for f in layer["unknown_families"]:
            f.pop("decision", None)
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-ref-missing", err)

    def test_resolved_family_wrong_kind_ref_rejected(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["unknown_families"][0]["decision"] = layer["decisions"][0]["id"]  # module-role
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-ref-inapplicable", err)

    def test_resolved_family_nonexistent_ref_rejected(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["unknown_families"][0]["decision"] = "MD-NOPE"
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-ref-missing", err)

    def test_valid_open_family_passes(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        f = layer["unknown_families"][0]
        f["disposition"] = "open"
        f.pop("decision", None)
        f["owner"] = "fixture-owner"
        f["probe"] = "fixture-probe"
        f["work_package"] = "fixture-work-package"
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 0, err)

    def test_wrong_field_types_controlled_rejection(self):
        run_tool(self.repo, self.out, "--write")
        base = self.complete_layer()
        cases = [
            ("kind", lambda L: L["decisions"][0].__setitem__("kind", [])),
            ("conclusion", lambda L: L["decisions"][0].__setitem__("conclusion", None)),
            ("evidence", lambda L: L["decisions"][0].__setitem__("evidence", "not-a-list")),
            ("bound", lambda L: L["decisions"][0].__setitem__("bound_input_hashes", ["oops"])),
            ("note", lambda L: L["decisions"][0].__setitem__("note", {})),
            ("family.path", lambda L: L["unknown_families"][0].__setitem__("path", 17)),
            ("family.names", lambda L: L["unknown_families"][0].__setitem__("names", [])),
            ("family.disposition", lambda L: L["unknown_families"][0].__setitem__("disposition", "yes")),
            ("family.owner", lambda L: L["unknown_families"][0].__setitem__("owner", 42)),
        ]
        for i, (label, mut) in enumerate(cases):
            with self.subTest(case=i, field=label):
                layer = json.loads(json.dumps(base, ensure_ascii=False))
                mut(layer)
                self.write_layer(layer)
                code, out, err = run_tool(self.repo, self.out,
                                          "--check", "--require-decisions")
                self.assertEqual(code, 1, "case %s should fail" % label)
                self.assertIn("ERROR[", err, "case %s needs located error" % label)
                self.assertNotIn("Traceback", err, "case %s leaked traceback" % label)


class ConsumerIntegrity(StrictDecisionsGate):
    """P743-R3-QA-01/02：消费者索引边界与 evidence-only 适用性。"""

    def test_referenced_invalid_decision_controlled(self):
        # 组合路径：被全部 resolved 族引用的决定类型错误 → 受控拒绝非 traceback
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][-1]["evidence"] = 17  # MD-S-900（所有族的引用目标）
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertNotIn("Traceback", err)
        self.assertIn("ERROR[decision-malformed]", err)
        self.assertIn("MD-S-900", err)

    def test_hash_only_applicability_bypass_rejected(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        ref = layer["decisions"][-1]  # MD-S-900
        ref["evidence"] = ["crates/auto-lang/src/lib.rs:1874"]  # 不含族路径
        ref["bound_input_hashes"]["crates/auto-lang/src/lib.rs"] =             acc_inventory.sha256_of(self.repo / "crates/auto-lang/src/lib.rs")
        for f in layer["unknown_families"]:  # 族路径仍被 bound 覆盖（旁路尝试）
            ref["bound_input_hashes"][f["path"]] =                 acc_inventory.sha256_of(self.repo / f["path"])
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-ref-inapplicable", err)

    def test_subject_only_applicability_bypass_rejected(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        # 专用决定：evidence 不含族路径（仅 lib.rs），subject 命中族路径——不得替代
        fam0 = layer["unknown_families"][0]
        layer["decisions"].append({
            "id": "MD-S-901", "subject": fam0["path"],
            "kind": "unknown-resolution", "conclusion": "resolved",
            "evidence": ["crates/auto-lang/src/lib.rs:1874"],
            "note": "subject-only bypass probe",
            "bound_input_hashes": {
                "crates/auto-lang/src/lib.rs":
                    acc_inventory.sha256_of(self.repo / "crates/auto-lang/src/lib.rs"),
                fam0["path"]: acc_inventory.sha256_of(self.repo / fam0["path"])},
        })
        fam0["decision"] = "MD-S-901"
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("family-ref-inapplicable", err)
        self.assertIn("MD-S-901", err)


class HostAdmissionShadowing(FixtureCase):
    """P743-R3-QA-03：导入/参数/局部绑定遮蔽宿主命名空间。"""

    @classmethod
    def setUpClass(cls):
        _spec4 = importlib.util.spec_from_file_location("ai_qa_r3", _TOOL_PATH)
        cls.ai = importlib.util.module_from_spec(_spec4)
        _spec4.loader.exec_module(cls.ai)

    def kinds(self, text, path="inline.at"):
        s = self.ai.scan_module(path, text)
        return s, {(c.get("receiver"), c["name"]): c["kind"] for c in s.calls}

    def test_review_fixture_import_shadow(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "import_shadow.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        self.assertEqual(k[(None, "print")], "imported-symbol")
        for recv in ("IO", "List", "File", "process"):
            self.assertEqual(k[(recv, k and self._meth(recv))], "unknown-receiver", recv)

    @staticmethod
    def _meth(recv):
        return {"IO": "read_line", "List": "new",
                "File": "read_text", "process": "args"}[recv]

    def test_review_fixture_parameter_shadow(self):
        fixture = _HERE / "fixtures" / "acc-inventory" / "review" / "parameter_shadow.at"
        s, k = self.kinds(fixture.read_text(encoding="utf-8"))
        # IO 是 Meter 参数：身份不可证明，保持 unknown
        self.assertEqual(k[("IO", "read_line")], "unknown-receiver")

    def test_let_var_binding_shadow(self):
        s, k = self.kinds('fn f() {\n var IO = List.new()\n IO.read_line()\n}\n')
        self.assertEqual(k[("IO", "read_line")], "unknown-receiver")

    def test_conflict_free_host_positive_unchanged(self):
        s, k = self.kinds(
            'fn f() {\n var a = List.new()\n var l = IO.read_line()\n'
            ' var t = File.read_text()\n var v = process.args()\n}\n')
        self.assertEqual(k[("List", "new")], "native-runtime")
        self.assertEqual(k[("IO", "read_line")], "native-runtime")
        self.assertEqual(k[("File", "read_text")], "native-runtime")
        self.assertEqual(k[("process", "args")], "native-runtime")


class HostAdmissionPhase5(FixtureCase):
    """P743-R4-QA-01：mut/方法/裸参数遮蔽不升级宿主。"""

    @classmethod
    def setUpClass(cls):
        _spec5 = importlib.util.spec_from_file_location("ai_qa_p5", _TOOL_PATH)
        cls.ai = importlib.util.module_from_spec(_spec5)
        _spec5.loader.exec_module(cls.ai)

    def kinds(self, name):
        path = _HERE / "fixtures" / "acc-inventory" / "review" / name
        s = self.ai.scan_module(name, path.read_text(encoding="utf-8"))
        return s, {(c.get("receiver"), c["name"]): c["kind"] for c in s.calls}

    def test_mut_parameter_shadow(self):
        s, k = self.kinds("mut_parameter.at")
        self.assertEqual(k[("IO", "read_line")], "unknown-receiver")

    def test_method_parameter_shadow(self):
        s, k = self.kinds("method_parameter.at")
        self.assertEqual(k[("IO", "read_line")], "unknown-receiver")

    def test_bare_parameter_shadow(self):
        s, k = self.kinds("bare_parameter.at")
        self.assertEqual(k[(None, "print")], "unknown-bare-call")

    def test_method_bare_parameter_shadow(self):
        s, k = self.kinds("method_bare_parameter.at")
        self.assertEqual(k[(None, "print")], "unknown-bare-call")

    def test_usual_parameter_control(self):
        s, k = self.kinds("usual_parameter.at")
        self.assertEqual(k[("IO", "read_line")], "unknown-receiver")

    def test_conflict_free_control(self):
        s, k = self.kinds("conflict_free.at")
        self.assertEqual(k[(None, "print")], "native-runtime")
        self.assertEqual(k[("IO", "read_line")], "native-runtime")


class TrustedIndexSemantics(StrictDecisionsGate):
    """P743-R4-QA-03：非法语义/过期绑定/重复 ID 不被族消费者读取。"""

    def test_illegal_conclusion_referenced_not_consumed(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][-1]["conclusion"] = "super-done"  # MD-S-900 被族引用
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("ERROR[decision-malformed]", err)
        self.assertIn("family-ref-missing", err)  # 消费者不再读取该记录
        self.assertNotIn("Traceback", err)

    def test_stale_binding_referenced_not_consumed(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        layer["decisions"][-1]["bound_input_hashes"]["auto/lib/token.at"] = "0" * 64
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-stale", err)
        self.assertIn("family-ref-missing", err)

    def test_duplicate_id_whole_group_invalidated(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        dup = dict(layer["decisions"][-1])  # 同 ID 再插一次（后者）
        layer["decisions"].append(dup)
        self.write_layer(layer)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-duplicate-id", err)
        self.assertIn("family-ref-missing", err)  # 先插入者也被整组作废


class ManifestShapeGate(FixtureCase):
    """P743-R4-QA-02：manifest 合法 JSON 错误形状受控拒绝。"""

    def mutate_manifest(self, mutate):
        run_tool(self.repo, self.out, "--write")
        man = self.out / acc_inventory.MANIFEST_NAME
        data = json.loads(man.read_text(encoding="utf-8"))
        mutate(data)
        man.write_text(json.dumps(data, ensure_ascii=False), encoding="utf-8")

    def assert_controlled(self, code, err):
        self.assertEqual(code, 1)
        self.assertIn("ERROR[manifest-malformed]", err)
        self.assertNotIn("Traceback", err)

    def test_manifest_top_level_list(self):
        run_tool(self.repo, self.out, "--write")
        man = self.out / acc_inventory.MANIFEST_NAME
        man.write_text("[]", encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assert_controlled(code, err)

    def test_manifest_top_level_int(self):
        man = self.out / acc_inventory.MANIFEST_NAME
        self.mutate_manifest(lambda d: None)
        man.write_text("1", encoding="utf-8")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assert_controlled(code, err)

    def test_source_identity_null(self):
        self.mutate_manifest(lambda d: d.__setitem__("source_identity", None))
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assert_controlled(code, err)

    def test_source_identity_list(self):
        self.mutate_manifest(lambda d: d.__setitem__("source_identity", []))
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assert_controlled(code, err)

    def test_inputs_and_manual_untouched(self):
        self.mutate_manifest(lambda d: d.__setitem__("source_identity", None))
        before = {f.name: f.read_bytes() for f in (self.repo / "auto/lib").iterdir()}
        run_tool(self.repo, self.out, "--check", "--require-decisions")
        after = {f.name: f.read_bytes() for f in (self.repo / "auto/lib").iterdir()}
        self.assertEqual(before, after)
        self.assertFalse((self.out / acc_inventory.DECISIONS_NAME).exists())


class TrustedIndexPhase6(StrictDecisionsGate):
    """P743-R5-QA-01/02：缺失绑定退出消费者、全局唯一 ID 覆盖错类型同 ID 组。"""

    def test_missing_bound_file_three_stage(self):
        run_tool(self.repo, self.out, "--write")
        layer = self.complete_layer()
        ref = self.repo / "docs/review-bound-reference.txt"
        ref.parent.mkdir(exist_ok=True)
        ref.write_bytes(b"ref\n")
        d9 = layer["decisions"][-1]
        d9["evidence"].append("docs/review-bound-reference.txt:1")
        d9["bound_input_hashes"]["docs/review-bound-reference.txt"] = \
            acc_inventory.sha256_of(ref)
        self.write_layer(layer)
        # 存在：可消费
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 0, err)
        # 删除：stale + 消费者不再读取该记录（family-ref-missing）
        ref.unlink()
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-stale", err)
        self.assertIn("family-ref-missing", err)
        # 恢复：重新可消费
        ref.write_bytes(b"ref\n")
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 0, err)
        shutil.rmtree(self.repo / "docs")

    def _dup_layer(self, bad_first: bool):
        layer = self.complete_layer()
        bad = dict(layer["decisions"][-1])
        bad["evidence"] = 17  # 类型错误
        if bad_first:
            layer["decisions"].insert(-1, bad)
        else:
            layer["decisions"].append(bad)
        self.write_layer(layer)

    def test_type_invalid_twin_bad_first_consumers_zero(self):
        run_tool(self.repo, self.out, "--write")
        self._dup_layer(bad_first=True)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-duplicate-id", err)
        self.assertIn("family-ref-missing", err)  # 合法同 ID 记录整组作废，消费者 0
        self.assertNotIn("Traceback", err)

    def test_type_invalid_twin_bad_last_consumers_zero(self):
        run_tool(self.repo, self.out, "--write")
        self._dup_layer(bad_first=False)
        code, out, err = run_tool(self.repo, self.out, "--check", "--require-decisions")
        self.assertEqual(code, 1)
        self.assertIn("decision-duplicate-id", err)
        self.assertIn("family-ref-missing", err)
        self.assertNotIn("Traceback", err)


class BindingBeforeTypePromotion(FixtureCase):
    """P743-R5-QA-03：可观察绑定先于本地类型/构造/宿主升级。"""

    @classmethod
    def setUpClass(cls):
        _spec6 = importlib.util.spec_from_file_location("ai_qa_p6", _TOOL_PATH)
        cls.ai = importlib.util.module_from_spec(_spec6)
        _spec6.loader.exec_module(cls.ai)

    def kinds(self, text, path="inline.at"):
        s = self.ai.scan_module(path, text)
        return s, {(c.get("receiver"), c["name"]): c["kind"] for c in s.calls}

    def test_param_shadowing_same_named_local_type(self):
        s, k = self.kinds(
            "type Meter {\n fn len() int { return 1 }\n}\n"
            "type Other {\n fn go() int { return 2 }\n}\n"
            "fn demo(Meter Other) int {\n return Meter.len() + Other.go()\n}\n"
            "fn ctor(Meter Other) int {\n return Meter() + Other()\n}\n")
        # Meter 是参数绑定：即使同名本地类型存在也不升级
        self.assertEqual(k[("Meter", "len")], "unknown-receiver")
        self.assertEqual(k[(None, "Meter")], "unknown-bare-call")
        # Other 是本地类型：合法升级保留
        self.assertEqual(k[("Other", "go")], "type-qualified")
        self.assertEqual(k[(None, "Other")], "type-construction")

    def test_no_binding_control_upgrades(self):
        s, k = self.kinds(
            "type Meter {\n fn len() int { return 1 }\n}\n"
            "fn demo2() int {\n return Meter.len()\n}\n"
            "fn ctor2() Meter {\n return Meter()\n}\n")
        self.assertEqual(k[("Meter", "len")], "type-qualified")
        self.assertEqual(k[(None, "Meter")], "type-construction")



if __name__ == "__main__":
    unittest.main()
