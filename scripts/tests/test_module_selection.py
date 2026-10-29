"""Counterexamples for partition and complete-module selection boundaries."""
import copy
from pathlib import Path
import sys
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
from module_catalog import SCHEMA, discover, select, source_identity, validate, verify_selection
from build_modules import make_catalog, navigation
from run_selected import exact_filter, validate_python_result


def catalog():
    rows = [{"id": f"r|binary|{n}", "recipe": "r", "binary": "binary", "test": n}
            for n in ("lex", "parse", "integration")]
    return {"schema": SCHEMA, "recipes": {"r": {}}, "obligations": rows,
            "source_identity": {"code": "source"}, "modules": [
                {"id": "lexer", "obligations": [rows[0]["id"]],
                 "source_paths": ["src/lexer.rs"], "consumers": ["parser"]},
                {"id": "parser", "obligations": [rows[1]["id"]],
                 "source_paths": ["src/parser"], "consumers": ["integration"]},
                {"id": "integration", "obligations": [rows[2]["id"]],
                 "source_paths": ["tests/integration.rs"], "consumers": []}]}


class ModuleSelection(unittest.TestCase):
    def test_python_exit_zero_cannot_hide_lost_or_skipped_case(self):
        import json
        import tempfile
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/"run.log"
            good = {"schema":"python-unittest-result/1","selected":["one","two"],
                    "executed":2,"skipped":[],"errors":[],"failures":[]}
            path.write_text(json.dumps(good)+"\n")
            validate_python_result(path,["one","two"])
            for field,value in [("selected",["one"]),("executed",1),("skipped",[["two","condition"]])]:
                bad = {**good,field:value}
                path.write_text(json.dumps(bad)+"\n")
                with self.assertRaises(ValueError):
                    validate_python_result(path,["one","two"])

    def test_cargo_runtime_edges_and_module_edges_form_one_checked_closure(self):
        c = catalog()
        for m in c["modules"]:
            m["owner_package"] = m["id"]
        c["impact_graph"] = {"schema":"test-impact-graph/1", "nodes":{
            n:{"source_paths":["pkg/"+n]} for n in ("lexer","parser","integration")},
            "edges":[["parser","integration"]]}
        r = select(c, changes=["pkg/parser/source.rs"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        verify_selection(c,r,{"code":"source"})
        r["modules"].remove("integration")
        r["obligations"] = [c["modules"][1]["obligations"][0]]
        with self.assertRaises(ValueError):
            verify_selection(c,r,{"code":"source"})

    def test_python_enumeration_remains_distinct_and_rejects_zero_or_duplicates(self):
        doc = {"schema":"python-unittest-discovery/1", "tests":[{"id":"module.Case.test_a","ignored":False}]}
        rows = discover(doc,"python")
        self.assertEqual(rows[0]["id"],"python|unittest|module.Case.test_a")
        doc["tests"].append(doc["tests"][0])
        with self.assertRaises(ValueError):
            discover(doc,"python")
        doc["tests"] = []
        with self.assertRaises(ValueError):
            discover(doc,"python")

    def test_reference_document_is_input_but_plan_receipt_is_bookkeeping(self):
        import tempfile
        from unittest.mock import patch
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory) / "auto-lang"
            (root / "docs/plans").mkdir(parents=True)
            reference = root / "docs/reference.txt"
            receipt = root / "docs/plans/progress.md"
            reference.write_text("original")
            receipt.write_text("pending")
            def git(*args, **kwargs):
                return "head" if kwargs.get("text") else b"docs/reference.txt\0docs/plans/progress.md\0"
            with patch("module_catalog.subprocess.check_output", side_effect=git):
                before = source_identity(root)
                receipt.write_text("complete")
                self.assertEqual(source_identity(root)["code"], before["code"])
                reference.write_text("changed fixture")
                self.assertNotEqual(source_identity(root)["code"], before["code"])

    def test_plan_number_is_provenance_not_module_ownership(self):
        self.assertEqual(navigation({"test": "tests::plan781_budget_http_tests::a",
                                     "binary": "auto-lang"}), "budget-http")

    def test_runtime_discovery_partitions_and_preserves_profiles_and_ignored(self):
        doc = {"rust-suites": {"binary": {"status": "listed", "testcases": {
            f"parser::tests::case_{i}": {"ignored": i == 2,
                "filter-match": {"status": "matches"}} for i in range(5)}}}}
        c = make_catalog({"a": {}, "b": {}}, {"a": doc, "b": doc}, max_cases=2)
        validate(c)
        self.assertEqual(len(c["modules"]), 6)
        self.assertEqual(len(c["obligations"]), 10)
        self.assertEqual(sum(r["ignored"] for r in c["obligations"]), 2)
        self.assertEqual({m["timing"]["state"] for m in c["modules"]}, {"unmeasured"})

    def test_runtime_discovery_cannot_accept_zero_or_missing_binary(self):
        for document in ({}, {"rust-suites": {}},
                         {"rust-suites": {"binary": {"status": "skipped"}}}):
            with self.assertRaises(ValueError):
                make_catalog({"r": {}}, {"r": document})

    def test_runtime_subset_may_filter_unselected_binary_but_not_empty_everything(self):
        doc = {"rust-suites": {
            "excluded": {"status": "skipped"},
            "selected": {"status": "listed", "testcases": {"f": {
                "ignored": False, "filter-match": {"status": "matches"}}}}}}
        with self.assertRaises(ValueError):
            discover(doc, "r")
        self.assertEqual([r["id"] for r in discover(doc, "r", allow_filtered_skipped=True)],
                         ["r|selected|f"])
        with self.assertRaises(ValueError):
            discover({"rust-suites": {"excluded": {"status": "skipped"}}}, "r",
                     allow_filtered_skipped=True)

    def test_filter_uses_exact_operator_and_rejects_injection(self):
        self.assertEqual(exact_filter([{"binary": "a::b", "test": "tests::f"}]),
                         "(binary_id(=a::b) and test(=tests::f))")
        with self.assertRaises(ValueError):
            exact_filter([{"binary": "a", "test": "f) or all()"}])

    def test_valid_partition_and_multiple_module_union(self):
        c = catalog()
        validate(c)
        r = select(c, ["parser", "integration"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        self.assertEqual(len(verify_selection(c, r, {"code": "source"})), 2)

    def test_missing_and_duplicate_partition_refused(self):
        for mutation in ("missing", "duplicate", "unknown", "empty"):
            with self.subTest(mutation=mutation):
                c = catalog()
                if mutation == "missing":
                    c["modules"].pop()
                    c["modules"][1]["consumers"] = []
                elif mutation == "duplicate":
                    c["modules"][1]["obligations"] += c["modules"][0]["obligations"]
                elif mutation == "unknown":
                    c["modules"][0]["obligations"] = ["ghost"]
                else:
                    c["modules"][0]["obligations"] = []
                with self.assertRaises(ValueError):
                    validate(c)

    def test_transitive_consumer_and_boundary_path(self):
        c = catalog()
        r = select(c, changes=["src/parser/expr.rs"])
        self.assertEqual(r["modules"], ["integration", "parser"])
        self.assertFalse(r["fallback"])
        r = select(c, changes=["src/parser_extra.rs"])
        self.assertTrue(r["fallback"])
        self.assertEqual(len(r["obligations"]), 3)

    def test_cycles_terminate_and_union_once(self):
        c = catalog()
        c["modules"][2]["consumers"] = ["lexer"]
        r = select(c, ["parser"])
        self.assertEqual(len(r["obligations"]), 3)

    def test_unknown_deleted_and_config_inputs_expand(self):
        for path in ("deleted.rs", "Cargo.lock", "new-test.rs", ".config/nextest.toml"):
            r = select(catalog(), changes=[path])
            self.assertTrue(r["fallback"])
            self.assertEqual(len(r["obligations"]), 3)

    def test_unknown_named_module_is_error_not_empty_success(self):
        with self.assertRaises(ValueError):
            select(catalog(), ["typo"])

    def test_mutated_receipt_cannot_omit_consumer_or_case(self):
        c = catalog()
        good = select(c, ["lexer"])
        for axis in ("case", "module", "duplicate", "catalog", "source"):
            with self.subTest(axis=axis):
                r = copy.deepcopy(good)
                if axis == "case": r["obligations"].pop()
                elif axis == "module": r["modules"].remove("integration")
                elif axis == "duplicate": r["modules"].append("parser")
                elif axis == "catalog": r["catalog_sha256"] = "other"
                else: r["source_identity"]["code"] = "other"
                with self.assertRaises(ValueError):
                    verify_selection(c, r, {"code": "source"})

    def test_stale_catalog_expands_but_forbids_execution(self):
        c = catalog()
        r = select(c, ["parser"], current_source={"code": "new"})
        self.assertEqual(len(r["obligations"]), 3)
        self.assertFalse(r["execution_allowed"])
        with self.assertRaises(ValueError):
            verify_selection(c, r, {"code": "new"})

    def test_empty_changes_explains_only_and_cannot_execute(self):
        c = catalog()
        r = select(c)
        self.assertEqual(r["obligations"], [])
        with self.assertRaises(ValueError):
            verify_selection(c, r, {"code": "source"})


if __name__ == "__main__":
    unittest.main()
