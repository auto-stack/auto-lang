"""PLAN-895 purpose-policy guards: conservation, completeness, negative controls.

Real checks load the frozen catalog, the top-level responsibility layer, and
the purpose policy. Negative controls mutate in-memory copies and must be
rejected. V-01/V-02 of docs/plans/895-test-purpose-audit-and-daily-admission.md.
"""
import copy
import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))

import audit_test_purposes as audit  # noqa: E402
from pathlib import Path as _Path
ROOT = _Path(__file__).resolve().parents[2]


class RealPolicyTest(unittest.TestCase):
    """The shipped artifacts must satisfy every conservation rule."""

    @classmethod
    def setUpClass(cls):
        cls.leaves, cls.obligations = audit.load_catalog()
        cls.policy = audit.load_policy()
        cls.top = audit.load_top()
        audit.validate(cls.policy, cls.top, cls.leaves, cls.obligations)

    def test_exact_partition_of_leaf_catalog(self):
        seen = {}
        for module in self.top["modules"]:
            for leaf in module["member_leaves"]:
                self.assertNotIn(leaf, seen, f"leaf {leaf} in two top modules")
                seen[leaf] = module["id"]
        self.assertEqual(set(seen), {m["id"] for m in self.leaves})

    def test_two_rust_targets_fully_mapped(self):
        auto_lang = sum(1 for o in self.obligations if o["recipe"] == "language"
                        and o["id"].startswith("language|auto-lang|"))
        core = sum(1 for o in self.obligations if o["recipe"] == "language"
                   and o["id"].startswith("language|auto-lang-core|"))
        self.assertGreaterEqual(auto_lang, 3000)
        self.assertGreaterEqual(core, 1600)
        leaf_top = {leaf: m["id"] for m in self.top["modules"] for leaf in m["member_leaves"]}
        # every language obligation resolves through its module into a top module
        for module in self.leaves:
            if module["recipe"] == "language":
                self.assertIn(module["id"], leaf_top)

    def test_per_item_rows_cover_every_obligation_exactly_once(self):
        rows = audit.per_item_rows(self.policy, self.top, self.leaves, self.obligations)
        self.assertEqual(len(rows), len(self.obligations))
        ids = [row["obligation"] for row in rows]
        self.assertEqual(len(set(ids)), len(ids))

    def test_purpose_stats_reconstructable(self):
        stats = audit.purpose_stats(self.top, self.policy["purpose_groups"])
        self.assertEqual(stats["top_modules"], len(self.top["modules"]))
        self.assertEqual(sum(stats[c] for c in audit.PURPOSE_CLASSES), len(self.top["modules"]))
        self.assertGreater(stats["daily"], 30)  # 数十个职责模块中的日常主体
        self.assertLess(stats["top_modules"], 100)  # 不是1181分片冒充

    def test_source_ledger_axes_complete(self):
        for source in self.policy["sources"]:
            self.assertIn(source["coverage"], audit.COVERAGE_AXES)
            self.assertTrue(source["evidence"])

    def test_long_atoms_and_reds_reference_real_leaves(self):
        leaf_ids = {m["id"] for m in self.leaves}
        for atom in self.policy["long_atoms"]:
            self.assertIn(atom["module"], leaf_ids)
        for red in self.policy["known_reds"]:
            self.assertIn(red["module"], leaf_ids)
        self.assertEqual(len(self.policy["long_atoms"]), 13)


class NegativeControlTest(unittest.TestCase):
    """Mutated copies must be rejected — a green banner cannot hide lost duties."""

    @classmethod
    def setUpClass(cls):
        cls.leaves, cls.obligations = audit.load_catalog()
        cls.policy = audit.load_policy()
        cls.top = audit.load_top()

    def test_missing_leaf_breaks_partition(self):
        top = copy.deepcopy(self.top)
        top["modules"][0]["member_leaves"] = top["modules"][0]["member_leaves"][:-1]
        with self.assertRaises(audit.PolicyError):
            audit.validate(self.policy, top, self.leaves, self.obligations)

    def test_duplicated_leaf_breaks_partition(self):
        top = copy.deepcopy(self.top)
        stolen = top["modules"][0]["member_leaves"][0]
        top["modules"][1]["member_leaves"] = [*top["modules"][1]["member_leaves"], stolen]
        with self.assertRaises(audit.PolicyError):
            audit.validate(self.policy, top, self.leaves, self.obligations)

    def test_unknown_leaf_reference_rejected(self):
        top = copy.deepcopy(self.top)
        top["modules"][0]["member_leaves"] = [*top["modules"][0]["member_leaves"], "language/auto-lang/ghost/001"]
        with self.assertRaises(audit.PolicyError):
            audit.validate(self.policy, top, self.leaves, self.obligations)

    def test_purpose_row_drift_rejected(self):
        policy = copy.deepcopy(self.policy)
        policy["purpose_groups"][0]["purpose"] = "research"  # disagree with top layer
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_invalid_purpose_class_rejected(self):
        policy = copy.deepcopy(self.policy)
        policy["purpose_groups"][0]["purpose"] = "sometimes"
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_missing_evidence_field_rejected(self):
        policy = copy.deepcopy(self.policy)
        del policy["purpose_groups"][0]["oracle"]
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_incomplete_pending_rejected(self):
        policy = copy.deepcopy(self.policy)
        row = next(r for r in policy["purpose_groups"] if r["top_module"] == "lang/corpus-vm-file")
        row["pending"] = {"question": "x"}  # missing checked/missing/next_fact/decider
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_source_without_coverage_rejected(self):
        policy = copy.deepcopy(self.policy)
        del policy["sources"][0]["coverage"]
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_broken_long_atom_reference_rejected(self):
        policy = copy.deepcopy(self.policy)
        policy["long_atoms"][0]["module"] = "language/auto-lang/ghost/001"
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_duplicate_purpose_row_rejected(self):
        policy = copy.deepcopy(self.policy)
        policy["purpose_groups"].append(copy.deepcopy(policy["purpose_groups"][0]))
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_register_proposal_unknown_source_rejected(self):
        policy = copy.deepcopy(self.policy)
        policy["register_proposals"][0]["source"] = "S-999"
        with self.assertRaises(audit.PolicyError):
            audit.validate(policy, self.top, self.leaves, self.obligations)

    def test_legacy_tier_with_filter_keeps_filter_semantics(self):
        """F-3 regression control: positional filters must reach the legacy
        navigation as substring selectors, never as module IDs (frozen
        compatibility contract until 2026-11-30)."""
        import tempfile
        from unittest import mock
        import module_command
        import module_catalog as mc
        catalog_path = ROOT / "scripts/test-governance/modules.json"
        catalog = mc.load(catalog_path)
        with mock.patch.object(module_command, "source_identity",
                               return_value=catalog["source_identity"]):
            with tempfile.TemporaryDirectory() as tmp:
                code = module_command.main(["--legacy-tier", "t", "vm", "--select-only",
                                            "--out", str(Path(tmp) / "sel")])
        self.assertEqual(code, 0)  # master semantics: filters never error as unknown modules

    def test_admission_is_suggestion_only(self):
        import tempfile
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "proposal.json"
            document = audit.admission(self.policy, self.top, self.leaves, out)
            self.assertEqual(document["schema"], "daily-admission-proposal/1")
            self.assertIn("NOT approved", document["note"])
            # a research module must never be recommended for daily
            for proposal in document["proposals"]:
                if proposal["purpose"] == "research":
                    self.assertNotEqual(proposal["daily_admission"], "recommend")


if __name__ == "__main__":
    unittest.main()
