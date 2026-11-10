"""PLAN-941: independently reconcile the frozen /2 accounting, not execution.

Only the explicit zero-execution accounting mode is accepted by this reader.
Actual legacy execution coverage is a separate obligation, never inferred from
tokens, an old parent-admission boolean, the script exit, or a verdict string.
"""
from collections import Counter
import hashlib
from pathlib import Path

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys

CORPUS = 'crates/auto-lang/test/cases/ace-m1/'
DECISION = 'docs/plans/reports/ace-m1-scope-decision-2026-10-28.md'
INPUTS = {
    CORPUS + 'binding.tsv': 'e12b4e782d17eeee1aea71e02f505977b8f2b47893dfe6e0bb765af8ad79b16a',
    CORPUS + 'applicability.json': '74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0',
    CORPUS + 'source-manifest.json': '497ca6d1519ae992a24515f731f275c9593365f4d9bbb80069453257aa9922d6',
    DECISION: '874ccc1198bf23c284fa588a6dfcdee80eecc8f5a0e392b88b34ddd74db0875b',
}
REQUIRED_SHA256 = '35ad43aae27ffb2d2c9a162c3dd5b99f971e4cd7b2ed36f4fe747a5634bd770e'
PATHS = ('ac', 'av', 'avm', 'a2r')
LAYERS = {
    'core_rows': 103, 'core_cells': 412, 'core_na_cells': 95,
    'core_ruled_accepted_cells': 32, 'core_required': 285,
    'extension_rows': 28, 'extension_cells': 112, 'extension_committed_cells': 28,
    'extension_na_cells': 39, 'future_cells': 45, 'required_cells': 313,
}


def typed_equal(actual, expected, where):
    """JSON equality including types: True must not compare equal to 1."""
    if type(actual) is not type(expected):
        raise AdmissionError(f'{where}: wrong JSON type')
    if isinstance(expected, dict):
        exact_keys(actual, expected.keys(), where)
        for key in expected:
            typed_equal(actual[key], expected[key], f'{where}.{key}')
    elif isinstance(expected, list):
        if len(actual) != len(expected):
            raise AdmissionError(f'{where}: incomplete/extra entries')
        for i, (left, right) in enumerate(zip(actual, expected)):
            typed_equal(left, right, f'{where}[{i}]')
    elif actual != expected:
        raise AdmissionError(f'{where}: stale or inconsistent value')


def _state(token):
    if token in ('emit-ok-run-unverified', 'snapshot-only-cert-gap', 'cert-diff'):
        return 'green-cert-gap', 'cert-gap'
    if token.startswith('ok'):
        return 'green', 'none'
    if token.startswith('diff:accepted'):
        return 'accepted-diff', 'none'
    if token.startswith(('diff', 'emit-')):
        return 'pending-diff', 'none'
    if token.startswith(('gap:', 'wait:')):
        return 'gap', 'none'
    if token.startswith('na:m1(NA-'):
        return 'na', 'none'
    raise AdmissionError(f'unknown binding token: {token}')


def frozen_ledger(repo):
    # Paths and approval hashes are parent-owned, never taken from a report.
    for relative, expected in INPUTS.items():
        repo.sha256(relative, expected)
    app, _ = repo.read_json(CORPUS + 'applicability.json')
    rows = [line.split('\t') for line in repo.read_bytes(CORPUS + 'binding.tsv').decode('utf-8').splitlines()
            if line.strip() and not line.startswith('#')]
    if len(rows) != 131 or any(len(row) != 12 for row in rows):
        raise AdmissionError('frozen binding shape changed')
    claims = {}
    for rule in app['rules']:
        for cell in rule['cells']:
            key = (cell['ns'], cell['id'], cell['path'])
            if key in claims:
                raise AdmissionError('duplicate applicability claim')
            claims[key] = rule['rule']
    extensions = {name: item['committed_path'] for item in app['extension_rows'] for name in item['ids']}
    ledger = []
    for row in rows:
        case_id, ns = row[:2]
        extension = ns == 'corpus' and case_id in extensions
        for target, token in zip(PATHS, row[5:9]):
            rule = claims.get((ns, case_id, target))
            if rule == 'FUTURE-O':
                layer = 'future'
            elif rule == 'R-ACCEPTED':
                layer = 'core_ruled_accepted'
            elif extension:
                layer = 'extension_committed' if extensions[case_id] == target else 'extension_na' if rule else 'extension_other'
            else:
                layer = 'core_na' if rule else 'core'
            state, cert = _state(token)
            ledger.append({'ns': ns, 'id': case_id, 'path': target, 'layer': layer,
                           'required': layer in ('core', 'extension_committed'),
                           'token': token, 'state': state, 'cert_axis': cert,
                           'producer': token, 'rule': rule})
    required = sorted(f"{r['ns']}:{r['id']}:{r['path']}" for r in ledger if r['required'])
    if len(required) != 313 or hashlib.sha256('\n'.join(required).encode()).hexdigest() != REQUIRED_SHA256:
        raise AdmissionError('approved required set changed')
    return ledger


def admit_accounting(scope, relative, repo):
    ledger = frozen_ledger(repo)
    record, receipt_sha = scope.read_json(relative)
    exact_keys(record, ('schema', 'generated_by', 'repo_root', 'identity', 'registered', 'layers',
                        'required_set_sha256', 'cell_states', 'executed', 'ledger', 'verdicts',
                        'core_blockers', 'extension_blockers', 'ruled_accepted', 'future_gaps'), 'legacy')
    if type(record['repo_root']) is not str or Path(record['repo_root']) != repo.root:
        raise AdmissionError('STALE legacy repository identity')
    typed_equal(record['schema'], 'ace.m1.acceptance/2', 'legacy.schema')
    typed_equal(record['generated_by'], 'scripts/verify-ace-m1-852.ps1 -Scope acceptance (PLAN-908 T-06)', 'legacy.producer')
    expected_identity = {
        'binding_sha256': INPUTS[CORPUS + 'binding.tsv'],
        'applicability_sha256': INPUTS[CORPUS + 'applicability.json'],
        'source_manifest_sha256': INPUTS[CORPUS + 'source-manifest.json'],
        'approval_doc': DECISION, 'approval_doc_sha256': INPUTS[DECISION], 'ruling_date': '2026-11-01',
    }
    typed_equal(record['identity'], expected_identity, 'legacy.identity')
    typed_equal(record['registered'], {'rows': 131, 'corpus_rows': 91, 'scenario_rows': 40, 'cells': 524}, 'legacy.registered')
    typed_equal(record['layers'], LAYERS, 'legacy.layers')
    typed_equal(record['required_set_sha256'], REQUIRED_SHA256, 'legacy.required_set')
    typed_equal(record['ledger'], ledger, 'legacy.ledger')
    typed_equal(record['cell_states'], dict(Counter(r['state'] for r in ledger)), 'legacy.cell_states')
    exact_keys(record['executed'], ('selected', 'executed', 'conditionally_excluded', 'not_executed',
                                  'parent_admission', 'selection', 'note'), 'legacy.executed')
    for field in ('selected', 'executed', 'conditionally_excluded', 'not_executed'):
        if counter(record['executed'][field], f'legacy.{field}') != 0:
            raise AdmissionError('legacy reader consumes accounting-only mode; actual execution needs separate evidence')
    if record['executed']['parent_admission'] is not None or record['executed']['selection'] is not None:
        raise AdmissionError('legacy accounting-only mode cannot authorize parent/selection echoes')
    if type(record['executed']['note']) is not str:
        raise AdmissionError('legacy execution note must be text')
    gaps = [f"{r['ns']}:{r['id']}:{r['path']}" for r in ledger
            if r['required'] and r['path'] == 'a2r' and r['cert_axis'] == 'cert-gap']
    typed_equal(record['core_blockers'], [], 'legacy.core_blockers')
    typed_equal(record['extension_blockers'], [], 'legacy.extension_blockers')
    def entries(layer):
        return [f"{r['ns']}:{r['id']}:{r['path']}={r['token']}" for r in ledger if r['layer'] == layer]
    typed_equal(record['ruled_accepted'], entries('core_ruled_accepted'), 'legacy.accepted_differences')
    typed_equal(record['future_gaps'], entries('future'), 'legacy.future')
    expected_verdicts = {
        'base': 'not-claimed(this scope never executes)',
        'core': 'pass (285/285 required cells green; 32 ruled-accepted shown, never green)',
        'extension': 'pass (28/28 committed cells green)',
        'future': 'reported-not-required (45/45 gap cells shown)',
        'can_claim_m1': not gaps,
    }
    for field, value in expected_verdicts.items():
        typed_equal(record['verdicts'][field], value, f'legacy.verdicts.{field}')
    # PLAN-997/998 reception (941 round 120): every required a2r cell is now
    # certified (value 38 via 992..996, trap 4 via 997, resource-scene 6 via
    # 998) — the gap() clause is gone. The cert verdict carries the strictly
    # validated trap + resource-scenario receipt clauses; the scenario verdict
    # carries the exact ok-emit-scene cell count (both counts re-pinned at
    # each domain reception; derived from the ledger where it is derivable).
    import re as _re
    from ace_m1_case_evidence import require as _require
    _require(set(record['verdicts']) <= set(expected_verdicts) | {'cert', 'scenario'}, 'extra legacy verdict')
    scene = sum(1 for r in ledger
                if r['required'] and r['ns'] == 'scenario' and r['path'] == 'a2r'
                and r['token'] == 'ok-emit-scene')
    _require(not gaps and scene == 6, '998 reception: six resource-scenario cells close the cert-gap axis')
    _require(_re.fullmatch(
        r'trap\+scene-validated\(4 trap \+ ' + str(scene) + r' resource-scenario receipt\(s\)\); value axis: supply -BuildIdentity or run snapshot/receipts',
        record['verdicts']['cert']), 'legacy.verdicts.cert: stale or inconsistent value')
    typed_equal(record['verdicts']['scenario'],
                f'{scene}/{scene} required a2r cells on the ok-emit-scene token; {scene} '
                f'resource-scenario receipt(s) strictly validated (ace.m1.a2r-resource-scenario/1)',
                'legacy.verdicts.scenario')
    return {'accounting_valid': True, 'required': 313, 'na': 134, 'accepted_diff': 32,
            'future': 45, 'required_sha256': REQUIRED_SHA256, 'cert_gaps': gaps,
            'execution': 'UNOBSERVED', 'can_claim_m1': not gaps, 'receipt_sha256': receipt_sha}
