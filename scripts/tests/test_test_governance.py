"""PLAN-887 T-03 Python control tests — conservation guards (V-02) and
admission controls 1-2 (V-04), runner-side.

Run: python -m unittest discover -s scripts/tests -p test_test_governance.py
"""
from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
SCRIPTS = HERE.parent
ROOT = SCRIPTS.parent
sys.path.insert(0, str(SCRIPTS))


def _load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


rfr = _load("run_full_regression", SCRIPTS / "run-full-regression.py")
ci = _load("check_inventory", SCRIPTS / "test-governance" / "check_inventory.py")
inv = _load("inventory", SCRIPTS / "test-governance" / "inventory.py")


class RetirementLoadTests(unittest.TestCase):
    def test_both_844_and_869_loaded(self):
        rows = rfr.retirements()
        ids = {r["id"] for r in rows}
        self.assertIn("auto-lang-core\tdatabase::tests::test_scope_management", ids)  # 844
        self.assertIn("auto-lang-core\timplicit_union::tests::test_implicit_union_new", ids)  # 869
        self.assertEqual(len(ids), len(rows), "duplicate identity inside merged manifests")

    def test_retired_ids_present_in_baseline(self):
        baseline = {l for l in (ROOT / "scripts/full-regression-language-ids.txt").read_text(encoding="utf-8").splitlines() if l}
        for row in rfr.retirements():
            self.assertIn(row["id"], baseline, f"ghost retirement: {row['id']}")


class ConservationTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "scripts").mkdir(parents=True)
        self._write("full-regression-language-ids.txt", "c1\talpha\nc1\tbeta\n")
        self._write("full-regression-language-selected.txt", "c1\talpha\n")
        self._write("full-regression-retirements-844.json.txt", json.dumps({
            "schema": "test-id-retirements/1", "plan": 844,
            "retirements": [{"id": "c1\tbeta", "evidence": "test"}],
        }))
        self._write("full-regression-retirements-869.json.txt", json.dumps({
            "schema": "test-id-retirements/1", "plan": 869, "retirements": [],
        }))
        self._write("full-regression-migrations-841.json.txt", json.dumps({
            "schema": "test-id-migrations/1", "plan": 841, "migrations": [],
        }))

    def _write(self, name, text):
        (self.root / "scripts" / name).write_text(text, encoding="utf-8")

    def test_legal_retirement_conserves(self):
        actual = {"c1\talpha"}
        verdict = ci.run_checks(self.root, actual=actual, selected={"c1\talpha"})
        self.assertEqual(verdict["errors"], [], f"legal conservation must pass: {verdict['errors']}")

    def test_ghost_retirement_fails(self):
        self._write("full-regression-retirements-844.json.txt", json.dumps({
            "schema": "test-id-retirements/1", "plan": 844,
            "retirements": [{"id": "c9\tghost", "evidence": "no baseline row"}],
        }))
        verdict = ci.run_checks(self.root, actual={"c1\talpha"}, selected={"c1\talpha"})
        self.assertTrue(any("ghost retirement" in e for e in verdict["errors"]), verdict["errors"])

    def test_missing_required_id_fails(self):
        self._write("full-regression-language-ids.txt", "c1\talpha\nc1\tbeta\nc1\tgamma\n")
        verdict = ci.run_checks(self.root, actual={"c1\talpha"}, selected={"c1\talpha"})
        self.assertTrue(any("missing required" in e for e in verdict["errors"]), verdict["errors"])

    def test_zero_selection_baseline_fails(self):
        self._write("full-regression-language-selected.txt", "")
        verdict = ci.run_checks(self.root, actual={"c1\talpha"}, selected={"c1\talpha"})
        self.assertTrue(any("zero selection" in e for e in verdict["errors"]), verdict["errors"])

    def test_suppressed_required_selected_fails(self):
        verdict = ci.run_checks(self.root, actual={"c1\talpha"}, selected=set())
        self.assertTrue(any("suppressed required-selected" in e for e in verdict["errors"]), verdict["errors"])


class MasqueradeTests(unittest.TestCase):
    def test_aggregate_wrapper_flagged(self):
        inv_json = {"entries": [{"entry": "vm_file_tests", "disposition": "managed"}]}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "scripts").mkdir(parents=True)
            for name in ("full-regression-language-ids.txt", "full-regression-language-selected.txt"):
                (root / "scripts" / name).write_text("", encoding="utf-8")
            for name in ("full-regression-retirements-844.json.txt", "full-regression-retirements-869.json.txt"):
                (root / "scripts" / name).write_text(json.dumps(
                    {"schema": "test-id-retirements/1", "plan": 869 if "869" in name else 844, "retirements": []}),
                    encoding="utf-8")
            (root / "scripts" / "full-regression-migrations-841.json.txt").write_text(
                json.dumps({"schema": "test-id-migrations/1", "plan": 841, "migrations": []}), encoding="utf-8")
            verdict = ci.run_checks(root, actual=None, selected=None, inventory=inv_json)
        self.assertTrue(any("aggregate wrapper" in e for e in verdict["errors"]), verdict["errors"])


class InventoryParseTests(unittest.TestCase):
    def test_suite_cases_parsed(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cases = root / "cases" / "gov"
            cases.mkdir(parents=True)
            (cases / "x.case.ad").write_text(
                '```test-case\ncase { schema: "autolang.case/2" id: "gov.d.one" '
                'source: { file: "a.at", syntax: "auto", path: "main.at" } '
                'checks: [{ id: "av", adapter: "probe", action: "run" }] }\n```',
                encoding="utf-8")
            (cases / "s.suite.ad").write_text(
                '```test-suite\nsuite { schema: "autolang.suite/2", id: "s", include: ["x.case.ad"] }\n```',
                encoding="utf-8")
            entries = inv.parse_suite_files(cases)
            self.assertEqual(len(entries), 1)
            self.assertEqual(entries[0]["identity"], "suite-case\tgov.d.one")
            self.assertEqual(entries[0]["source"], "a.at")


if __name__ == "__main__":
    unittest.main()
