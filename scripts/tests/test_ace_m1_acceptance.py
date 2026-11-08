#!/usr/bin/env python3
"""PLAN-908 governance controls for the M1 applicability registry, the
tracked source manifest and the strict acceptance accounting.

Static checks (hermetic, real tree): binding/applicability/manifest
cross-consistency, the five-state vocabulary, the 61-cell G-DEC migration
integrity, the decision-doc hash binding, and the near-neighbor assets.

Mutation controls (V-01/V-02/V-03/V-05/V-07 class): the tracked corpus plus
its bound documents are copied into a scratch tree, ONE defect is injected,
and the REAL verifier (scripts/verify-ace-m1-852.ps1) is run against the
mutated copy via -RepoRoot. Every injected defect must REDDEN the gate
(exit 1) with a naming reason; the unmutated copy is the accepted control
and must stay green. No repo file is ever modified: mutations live only in
the scratch copy (per-plan evidence policy).
"""
from pathlib import Path
import json
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parent.parent.parent
CORPUS = ROOT / 'crates/auto-lang/test/cases/ace-m1'
VERIFIER = ROOT / 'scripts/verify-ace-m1-852.ps1'
DECISION = ROOT / 'docs/plans/reports/ace-m1-scope-decision-2026-10-28.md'
ORACLES_DOC = ROOT / 'docs/specs/auto-test-case/ace-m1-scenario-oracles.md'

PATHS = ('ac', 'av', 'avm', 'a2r')
PS = 'pwsh' if shutil.which('pwsh') else 'powershell'


def load_binding():
    rows = []
    for ln in (CORPUS / 'binding.tsv').read_text(encoding='utf-8').split('\n'):
        if not ln.strip() or ln.startswith('#'):
            continue
        f = ln.split('\t')
        rows.append(f)
    return rows


def cell_state(token):
    if token.startswith('ok'):
        return 'green'
    if token in ('emit-ok-run-unverified', 'snapshot-only-cert-gap', 'cert-diff'):
        return 'green-cert-gap'
    if token.startswith('diff:accepted'):
        return 'accepted-diff'
    if token.startswith('diff') or token.startswith('emit-'):
        return 'pending-diff'
    if token.startswith(('gap:', 'wait:')):
        return 'gap'
    if token.startswith('na:m1(NA-'):
        return 'na'
    if token == 'n/a':
        return 'bare-na'
    return 'invalid'


class StaticRegistryTests(unittest.TestCase):
    """Hermetic checks on the real tree (V-01/V-02/V-08 class anchors)."""

    def test_binding_shape_and_conservation(self):
        rows = load_binding()
        self.assertEqual(len(rows), 131)
        self.assertTrue(all(len(f) == 12 for f in rows))
        corpus = [f for f in rows if f[1] == 'corpus']
        scen = [f for f in rows if f[1] == 'scenario']
        self.assertEqual((len(corpus), len(scen)), (91, 40))
        keys = [(f[1], f[0]) for f in rows]
        self.assertEqual(len(keys), len(set(keys)))
        self.assertIn(('corpus', 'R03'), keys)
        self.assertIn(('scenario', 'R03'), keys)

    def test_five_state_vocabulary_and_no_bare_na(self):
        for f in load_binding():
            for p in PATHS:
                tok = f[PATHS.index(p) + 5]
                self.assertNotEqual(cell_state(tok), 'bare-na',
                                    f"bare n/a at {f[0]}:{p}")
                self.assertNotEqual(cell_state(tok), 'invalid',
                                    f"non-vocabulary token {tok!r} at {f[0]}:{p}")

    def test_applicability_rule_cells_match_binding(self):
        app = json.loads((CORPUS / 'applicability.json').read_text(encoding='utf-8'))
        self.assertEqual(app['schema'], 'ace.m1.applicability/1')
        rows = {(f[1], f[0]): f for f in load_binding()}
        claimed = {}
        migrated = 0
        for rule in app['rules']:
            for c in rule['cells']:
                key = (c['ns'], c['id'], c['path'])
                self.assertNotIn(key, claimed, f"duplicate claim {key}")
                claimed[key] = rule['rule']
                self.assertIn((c['ns'], c['id']), rows, f"unknown row {key}")
                current = rows[(c['ns'], c['id'])][PATHS.index(c['path']) + 5]
                self.assertEqual(current, c['binding_token'],
                                 f"{key}: binding token drifted from the rule")
                if c['migrated']:
                    migrated += 1
                    self.assertEqual(c['binding_token'], f"na:m1({rule['rule']})")
        self.assertEqual(migrated, 61, "G-DEC migration must be exactly 61 cells")
        # every na:m1 token is claimed by a rule (sourceless NA forbidden)
        for f in load_binding():
            for p in PATHS:
                if cell_state(f[PATHS.index(p) + 5]) == 'na':
                    self.assertIn((f[1], f[0], p), claimed,
                                  f"sourceless NA at {f[0]}:{p}")

    def test_expected_totals(self):
        app = json.loads((CORPUS / 'applicability.json').read_text(encoding='utf-8'))
        t = app['expected_totals']
        self.assertEqual(t['cells'], 524)
        self.assertEqual(t['core_cells'], 412)
        self.assertEqual(t['core_na_cells'], 95)
        self.assertEqual(t['ruled_accepted_cells'], 32)
        self.assertEqual(t['committed_cells'], 28)
        self.assertEqual(t['extension_na_cells'], 39)
        self.assertEqual(t['future_cells'], 45)
        self.assertEqual(t['required_cells'], 313)
        self.assertEqual(285 + 28, 313)
        self.assertEqual(95 + 32 + 285, 412)
        self.assertEqual(313 + 134 + 32 + 45, 524)

    def test_decision_doc_hash_binding(self):
        import hashlib
        app = json.loads((CORPUS / 'applicability.json').read_text(encoding='utf-8'))
        sha = hashlib.sha256(DECISION.read_bytes()).hexdigest()
        self.assertEqual(app['approval_source']['decision_doc_sha256'], sha)
        self.assertEqual(app['approval_source']['ruling_date'], '2026-11-01')

    def test_sidecars_are_git_tracked_and_not_ignored(self):
        """F-908-R1-01 regression lock: the `*.json` parent ignore rule once
        swallowed both registries out of the delivery commit (fresh checkout
        contract red). They must be tracked and must not match any ignore
        rule; skipped only when the tree is not a git checkout."""
        if shutil.which('git') is None or not (ROOT / '.git').exists():
            self.skipTest('not a git checkout')
        for rel in ('crates/auto-lang/test/cases/ace-m1/applicability.json',
                    'crates/auto-lang/test/cases/ace-m1/source-manifest.json'):
            tracked = subprocess.run(['git', 'ls-files', '--error-unmatch', rel],
                                     cwd=ROOT, capture_output=True, text=True)
            self.assertEqual(tracked.returncode, 0, f"{rel} is not git-tracked")
            ignored = subprocess.run(['git', 'check-ignore', '-q', rel],
                                     cwd=ROOT, capture_output=True, text=True)
            self.assertNotEqual(ignored.returncode, 0, f"{rel} matches an ignore rule")

    def test_manifest_counts_and_neighbors(self):
        m = json.loads((CORPUS / 'source-manifest.json').read_text(encoding='utf-8'))
        self.assertEqual(m['schema'], 'ace.m1.source-manifest/1')
        self.assertEqual(len(list((CORPUS / 'src').glob('*.at'))),
                         m['corpus_src']['expected_count'])
        for nb in ('NB17.at', 'NB18.at', 'NB19.at'):
            self.assertIn(nb, m['corpus_src']['files'])
            self.assertIn(nb, m['corpus_src']['provenance']['neighbors-908'])
        self.assertFalse(list((CORPUS / 'src').glob('D??.at')),
                         "scenario D-row sources must never be fabricated")
        case = (CORPUS / 'nb-diags-908.case.ad').read_text(encoding='utf-8')
        for nb, oracle in (('nb17', '2'), ('nb18', '2'), ('nb19', '0')):
            self.assertIn(f'ace.m1.{nb}', case)
        # the frozen 852 base suite stays unexpanded (36 cases / 97 checks)
        base = (CORPUS / 'ace-m1-base.suite.ad').read_text(encoding='utf-8')
        self.assertNotIn('nb-diags-908.case.ad', base)

    def test_scenario_certs(self):
        """PLAN-998 resource-scenario domain: the six H-07 scenario cells
        (ns=scenario R01..R06, outcome=events) carry the ok-emit-scene a2r
        token AND a structurally valid, current
        ace.m1.a2r-resource-scenario/1 receipt; the registry
        (oracle/scenario/scenario-oracle.json) is the authority for leg
        membership and every registry-declared fixture source exists on
        disk (registry -> disk consistency; no fabricated sources)."""
        import hashlib
        wrapper_sha = hashlib.sha256(
            (CORPUS / 'oracle/resource-wrapper-v1.rs').read_bytes()).hexdigest()
        rows = {f[0]: f for f in load_binding() if f[1] == 'scenario'}
        registry = json.loads(
            (CORPUS / 'oracle/scenario/scenario-oracle.json').read_text(encoding='utf-8'))
        self.assertEqual(registry['schema'], 'ace.m1.a2r-resource-oracle/1')
        for rid in ('R01', 'R02', 'R03', 'R04', 'R05', 'R06'):
            f = rows[rid]
            self.assertEqual(f[3], 'events', f"{rid}: outcome must be events")
            self.assertEqual(f[8], 'ok-emit-scene',
                             f"{rid}: a2r token must be ok-emit-scene")
            cert = json.loads(
                (CORPUS / f'oracle/scenario-certs/m1_{rid.lower()}.scenario-cert.json')
                .read_text(encoding='utf-8'))
            self.assertEqual(cert['schema'], 'ace.m1.a2r-resource-scenario/1',
                             f"{rid}: receipt schema")
            self.assertEqual(cert['case'], rid, f"{rid}: receipt case id")
            self.assertEqual(cert['verdict'], 'certified', f"{rid}: verdict")
            self.assertEqual(cert['identity']['wrapper_sha256'], wrapper_sha,
                             f"{rid}: wrapper axis is not the frozen resource wrapper")
            for leg in cert['legs']:
                for run in leg['runs']:
                    self.assertEqual(run['invariants'], 'pass',
                                     f"{rid}/{leg['id']}/{run['run']}: invariants")
            # registry-authoritative leg cross-check (F-998 mirror of
            # F-997-R1-02): non-empty subset of the registry cell's legs.
            self.assertIn(rid, registry['cells'], f"{rid}: registry cell missing")
            reg_leg_ids = {leg['id'] for leg in registry['cells'][rid]['legs']}
            receipt_leg_ids = {leg['id'] for leg in cert['legs']}
            self.assertTrue(receipt_leg_ids, f"{rid}: receipt carries no legs")
            self.assertTrue(receipt_leg_ids <= reg_leg_ids,
                            f"{rid}: receipt legs outside the registry cell")
        # registry -> disk consistency: the declared fixture sources (the 12
        # legacy-a2r-resources-998 byte-copies) all exist; the feature dir
        # never fabricates D*.at corpus-pattern sources.
        sources = set()
        for cell in registry['cells'].values():
            for leg in cell['legs']:
                if 'source' in leg:
                    sources.add(leg['source'].replace('\\', '/'))
        self.assertEqual(len(sources), 12,
                         f"expected 12 distinct fixture sources, got {sorted(sources)}")
        for rel in sorted(sources):
            self.assertTrue(rel.startswith('legacy-a2r-resources-998/src/'),
                            f"fixture source outside the 998 feature dir: {rel}")
            self.assertTrue((CORPUS / rel).is_file(),
                            f"registry fixture source missing on disk: {rel}")
        self.assertFalse(
            list((CORPUS / 'legacy-a2r-resources-998/src').glob('D*.at')),
            "fabricated D*.at scenario source in the 998 feature dir")


class MutationControlTests(unittest.TestCase):
    """V-01/V-02/V-03/V-05/V-07: one injected defect per scratch copy; the
    REAL verifier must reject it. The unmutated copy is the accepted
    control (mutation testing discipline: control green, mutants red).
    Every test refreshes its scratch tree from a pristine snapshot so
    mutations can never leak between tests or into the repository."""

    @classmethod
    def setUpClass(cls):
        cls.tmp = Path(tempfile.mkdtemp(prefix='ace-m1-908-ctrl-'))
        cls.pristine = cls.tmp / 'pristine'
        corpus_dst = cls.pristine / 'crates/auto-lang/test/cases/ace-m1'
        corpus_dst.parent.mkdir(parents=True)
        shutil.copytree(CORPUS, corpus_dst)
        spec_dst = cls.pristine / 'docs/specs/auto-test-case'
        spec_dst.mkdir(parents=True)
        shutil.copy2(ORACLES_DOC, spec_dst / ORACLES_DOC.name)
        rep_dst = cls.pristine / 'docs/plans/reports'
        rep_dst.mkdir(parents=True)
        shutil.copy2(DECISION, rep_dst / DECISION.name)

    @classmethod
    def tearDownClass(cls):
        shutil.rmtree(cls.tmp, ignore_errors=True)

    def setUp(self):
        self.tree = self.tmp / 'tree'
        if self.tree.exists():
            shutil.rmtree(self.tree)
        shutil.copytree(self.pristine, self.tree)
        self.corpus = self.tree / 'crates/auto-lang/test/cases/ace-m1'

    def run_verifier(self, scope, extra=()):
        cmd = [PS, '-NoProfile', '-File', str(VERIFIER), '-Scope', scope,
               '-RepoRoot', str(self.tree), *extra]
        proc = subprocess.run(cmd, capture_output=True, text=True, timeout=180)
        return proc.returncode, (proc.stdout + proc.stderr)

    # ---- accepted control ---------------------------------------------------
    def test_control_unmutated_copy_stays_green(self):
        for scope, extra in (('contract', ()),
                             ('acceptance', ('-OutDir', str(self.tree / "acc-out")))):
            code, out = self.run_verifier(scope, extra)
            self.assertEqual(code, 0, f"control {scope} must stay green:\n{out}")
        report = json.loads((self.tree / 'acc-out' / 'ace-m1-acceptance.json').read_text(encoding='utf-8'))
        self.assertEqual(report['schema'], 'ace.m1.acceptance/2')
        # PLAN-998 closed the last six required a2r cells (ok-emit-scene,
        # cert-gap 6 -> 0): the registry is now 313/313 green with strictly
        # validated receipts, so the honest G-DEC-5 claim is True. Any
        # regression that flips a required cell back to red/gap (or drops a
        # scenario certificate) must flip this False again.
        self.assertTrue(report['verdicts']['can_claim_m1'])
        self.assertEqual(report['layers']['required_cells'], 313)
        self.assertEqual(len(report['future_gaps']), 45)
        self.assertEqual(len(report['ruled_accepted']), 32)

    # ---- V-01 approval/whitelist controls -----------------------------------
    def _with_applicability(self, mutate):
        path = self.corpus / 'applicability.json'
        doc = json.loads(path.read_text(encoding='utf-8'))
        mutate(doc)
        path.write_text(json.dumps(doc, ensure_ascii=False, indent=1), encoding='utf-8')

    def test_v01_missing_approval_source_rejected(self):
        def m(doc):
            del doc['approval_source']['decision_doc_sha256']
        self._with_applicability(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('approval_source', out)

    def test_v01_enlarged_na_whitelist_rejected(self):
        def m(doc):
            na5 = next(r for r in doc['rules'] if r['rule'] == 'NA-5')
            na5['cells'].append({'ns': 'corpus', 'id': 'B01', 'path': 'ac',
                                 'old_token': 'ok', 'binding_token': 'ok',
                                 'migrated': False, 'approval': 'smuggled'})
        self._with_applicability(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('total mismatch', out)

    def test_v01_duplicate_claim_rejected(self):
        def m(doc):
            na6 = next(r for r in doc['rules'] if r['rule'] == 'NA-6')
            na6['cells'].append({'ns': 'corpus', 'id': 'H01', 'path': 'avm',
                                 'old_token': 'n/a', 'binding_token': 'na:m1(NA-6)',
                                 'migrated': True, 'approval': 'G-DEC-1'})
        self._with_applicability(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('two rules', out)

    def test_v01_unknown_rule_row_rejected(self):
        def m(doc):
            doc['rules'][0]['cells'].append({'ns': 'scenario', 'id': 'Z99',
                                             'path': 'ac', 'old_token': 'x',
                                             'binding_token': 'x',
                                             'migrated': False, 'approval': 'x'})
        self._with_applicability(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('unknown row', out)

    # ---- V-02 binding conservation controls ---------------------------------
    def _with_binding(self, mutate):
        path = self.corpus / 'binding.tsv'
        lines = path.read_text(encoding='utf-8').split('\n')
        lines = mutate(lines)
        path.write_text('\n'.join(lines), encoding='utf-8')

    def test_v02_dropped_row_rejected(self):
        def m(lines):
            out = [ln for ln in lines if not ln.startswith('I06\t')]
            self.assertEqual(len(lines) - len(out), 1)
            return out
        self._with_binding(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('!= 131', out)

    def test_v02_column_truncation_rejected(self):
        def m(lines):
            for i, ln in enumerate(lines):
                if ln.startswith('B03\t'):
                    lines[i] = '\t'.join(ln.split('\t')[:8])
            return lines
        self._with_binding(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('columns', out)

    def test_v02_half_migration_bare_na_rejected(self):
        def m(lines):
            for i, ln in enumerate(lines):
                if ln.startswith('C01\t'):
                    f = ln.split('\t')
                    f[7] = 'n/a'  # roll one migrated cell back to a bare n/a
                    lines[i] = '\t'.join(f)
            return lines
        self._with_binding(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        # defense in depth: the rule/token desync check fires before the
        # bare-na scan; either reason proves the half-migration is rejected
        self.assertTrue("bare 'n/a'" in out or 'half-written' in out, out)

    def test_v07_rule_binding_token_desync_rejected(self):
        # binding edited without the ruling registry: a required gap cell is
        # silently flipped to na:m1 — the rule never claimed it, so both the
        # token drift AND the layer totals must catch it.
        def m(lines):
            for i, ln in enumerate(lines):
                if ln.startswith('E01\t'):
                    f = ln.split('\t')
                    f[5] = 'na:m1(NA-5)'  # scenario-tooling gap smuggled to NA
                    lines[i] = '\t'.join(f)
            return lines
        self._with_binding(m)
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        # manifest binding-hash, rule binding_token desync or the sourceless-
        # NA scan — any of the three layers proves the smuggled NA is caught
        self.assertTrue('DRIFTED' in out or 'binding_token' in out or 'NOT covered' in out, out)

    # ---- V-03 source-identity controls --------------------------------------
    def test_v03_source_hash_drift_rejected(self):
        src = self.corpus / 'src/B01.at'
        src.write_text(src.read_text(encoding='utf-8') + '\n', encoding='utf-8')
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('DRIFTED', out)

    def test_v03_fabricated_scenario_source_rejected(self):
        (self.corpus / 'src/D01.at').write_text('fn main() int { return 42 }\n', encoding='utf-8')
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        # the unregistered-source count check fires before the D-pattern
        # scan; either reason proves fabrication is rejected
        self.assertTrue('fabricated' in out or 'src count' in out, out)

    def test_v03_missing_scenario_oracles_doc_rejected(self):
        doc = self.tree / 'docs/specs/auto-test-case/ace-m1-scenario-oracles.md'
        doc.unlink()
        code, out = self.run_verifier('contract')
        self.assertNotEqual(code, 0)
        self.assertIn('scenario oracles', out)

    # ---- V-05 acceptance admission controls ----------------------------------
    def test_v05_fake_parent_report_rejected(self):
        fake = self.tree / 'fake-report.json'
        fake.write_text(json.dumps({'results': [], 'suite': 'made-up'}), encoding='utf-8')
        code, out = self.run_verifier('acceptance', ('-OutDir', str(self.tree / "a1"), '-ParentReport', str(fake)))
        self.assertNotEqual(code, 0)
        self.assertIn('zero results', out)

    def test_v05_failing_parent_report_rejected(self):
        fake = self.tree / 'failing-report.json'
        fake.write_text(json.dumps({'results': [
            {'case': 'x', 'check': 'y', 'status': 'FAIL'}]}), encoding='utf-8')
        code, out = self.run_verifier('acceptance', ('-OutDir', str(self.tree / "a2"), '-ParentReport', str(fake)))
        self.assertNotEqual(code, 0)
        self.assertIn('non-PASS', out)

    def test_v05_unknown_selection_rejected(self):
        code, out = self.run_verifier('acceptance', ('-OutDir', str(self.tree / "a3"), '-Select', 'corpus:NOPE'))
        self.assertNotEqual(code, 0)
        self.assertIn('selection', out)

    def test_v05_full_never_claimed_from_green_subset(self):
        # a passing parent report for a local selection must NOT flip M1:
        # since PLAN-998 completed the registry (313/313 green, zero
        # cert-gap) the claim is honestly True WITHOUT any parent admission
        # (proven by the unmutated control) — so the guard here is that the
        # one-check subset admission neither manufactures nor retracts it:
        # the claim must equal the no-parent state, never the report's own
        # scope.
        good = self.tree / 'good-report.json'
        good.write_text(json.dumps({'results': [
            {'case': 'ace.m1.b01', 'check': 'ac-native', 'status': 'PASS'}],
            'suite': 'ace-m1-base'}), encoding='utf-8')
        code, out = self.run_verifier('acceptance', ('-OutDir', str(self.tree / "a4"), '-ParentReport', str(good)))
        self.assertEqual(code, 0, out)
        report = json.loads((self.tree / 'a4' / 'ace-m1-acceptance.json').read_text(encoding='utf-8'))
        self.assertTrue(report['verdicts']['can_claim_m1'])
        self.assertIn('pass(parent-report-admitted, 1 checks)',
                      report['verdicts']['base'])
        self.assertIn('pass', report['verdicts']['core'])


if __name__ == '__main__':
    unittest.main()
