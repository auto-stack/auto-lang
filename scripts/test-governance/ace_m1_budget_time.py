#!/usr/bin/env python3
"""PLAN-941 T-07: the budget/time measurement section.

Composes REAL measured resource facts from documents that are already
strictly admitted: the frozen full-group gate meters (per-group wall/CPU/
peak/processes), the original optimized-observer run meter, and the
composer's own live-data reservation. Budget policy DECLARATIONS never prove
runtime (their runtime_verdict stays UNOBSERVED — the original control's
rule); the per-policy row instead counts only really-measured sample
windows: every listed source is sha-verified and its meter re-read here, a
policy is observed only with at least ten real windows, and the row stays
PARTIAL naming exactly the policies that still lack a measured window. The
section never converts declarations or a single green meter into the
budgets_time gate.
"""
from ace_m1_arch_acceptance import AdmissionError, exact_keys

GATE_SCHEMA = 'fixed-test-run/1'
GROUPS = ('ace-source', 'ace-syntax', 'infra-ace-m1-arch')
LIVE_CAP = 48 << 20
SAMPLES_SCHEMA = 'ace.m1.policy-runtime-samples-binding/1'
MIN_SAMPLES = 10


def _meter(row, where):
    if type(row) is not dict:
        raise AdmissionError(f'{where}: expected a meter object')
    meter = row.get('meter')
    if type(meter) is not dict:
        raise AdmissionError(f'{where}: expected a meter object')
    for field in ('wall_ms', 'workload_cpu_ms', 'peak_job_bytes', 'job_processes'):
        value = meter.get(field)
        if type(value) not in (int, float) or value <= 0:
            raise AdmissionError(f'{where}: meter {field} must be a positive number')
    return {field: meter[field] for field in ('wall_ms', 'workload_cpu_ms', 'peak_job_bytes', 'job_processes')}


def _p90(walls):
    """P90 by the §5.6 rank N - floor(N/10) over ascending samples
    (1-based rank → 0-based index N - floor(N/10) - 1)."""
    return walls[len(walls) - (len(walls) // 10) - 1]


def admit_policy_runtime_samples(evidence, ref, gate_doc=None, gate_sha=None):
    """Admit the per-policy runtime sample binding: every source document is
    sha-verified and its meter re-read here (summaries are never trusted).
    Gate documents contribute their per-test exec_time samples (libtest
    seconds converted to ms) plus their group meters' peaks; command
    documents contribute their own process meter. Returns per-policy stats
    keyed by policy name; observation requires >= MIN_SAMPLES windows."""
    exact_keys(ref, ('path', 'sha256'), 'policy runtime samples binding')
    evidence.sha256(ref['path'], ref['sha256'])
    document, _ = evidence.read_json(ref['path'])
    if document.get('schema') != SAMPLES_SCHEMA:
        raise AdmissionError('unexpected policy runtime samples schema')
    policies = document.get('policies')
    if type(policies) is not dict or not policies:
        raise AdmissionError('policy runtime samples missing')
    stats = {}
    for name, sources in policies.items():
        if type(name) is not str or not name or type(sources) is not list or not sources:
            raise AdmissionError('malformed policy sample entry')
        walls, peaks = [], []
        for src in sources:
            exact_keys(src, ('path', 'sha256'), 'policy sample source')
            # Per-document transient admission: read, verify the digest over
            # the read bytes, extract the meter, then release the raw/decoded
            # reservation so a thousand-window sample set stays bounded.
            if (gate_doc is not None and gate_sha == src['sha256']):
                # Same gate document the caller already admitted by content
                # hash — reuse the decoded frame instead of a second
                # multi-megabyte transient read inside the shared budget.
                doc = gate_doc
                transient = 0
            else:
                raw = evidence.read_bytes(src['path'])
                nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
                doc, digest = evidence.decode_json(raw)
                if digest != src['sha256']:
                    raise AdmissionError(f'STALE artifact content: {src["path"]}')
                transient = len(raw) + 1 + 7 * len(raw) + 256 * nodes + 4096
            if src['path'].endswith('/results.json') or src['path'] == 'results.json':
                if doc.get('schema') != GATE_SCHEMA or doc.get('gate_passed') is not True:
                    raise AdmissionError('sample gate document did not pass')
                results = doc.get('results')
                if type(results) is not list:
                    raise AdmissionError('sample gate document malformed')
                for result in results:
                    execution = result.get('execution') if isinstance(result, dict) else None
                    if isinstance(execution, dict):
                        peak = execution.get('meter', {}).get('peak_job_bytes')
                        if type(peak) in (int, float) and peak > 0:
                            peaks.append(peak)
                    for event in (result.get('results', {}) if isinstance(result, dict) else {}).values():
                        t = event.get('exec_time') if isinstance(event, dict) else None
                        if type(t) not in (int, float) or t <= 0:
                            raise AdmissionError('malformed per-test exec_time sample')
                        walls.append(t * 1000.0)
            else:
                meter = doc.get('meter') if isinstance(doc, dict) else None
                if not isinstance(meter, dict):
                    raise AdmissionError('sample command document carries no meter')
                wall = meter.get('wall_ms')
                if type(wall) not in (int, float) or wall <= 0:
                    raise AdmissionError('sample command meter wall must be positive')
                walls.append(wall)
                peak = meter.get('peak_job_bytes') or meter.get('peak_process_bytes')
                if type(peak) in (int, float) and peak > 0:
                    peaks.append(peak)
            del doc
            if transient:
                evidence.budget.release_transient(transient)
                del raw
        walls.sort()
        stats[name] = {'n': len(walls), 'p90_ms': _p90(walls), 'max_ms': walls[-1],
                       'peak_bytes': max(peaks) if peaks else None}
    return stats


def admit_budget_time_measurements(inputs, evidence, gate_ref, optimized_ref, budget_contracts, admission_peak, samples_ref=None):
    """Admit the gate and optimized documents by content hash, then compose."""
    exact_keys(gate_ref, ('path', 'sha256'), 'budget-time gate binding')
    evidence.sha256(gate_ref['path'], gate_ref['sha256'])
    document, _ = evidence.read_json(gate_ref['path'])
    if document.get('schema') != GATE_SCHEMA or document.get('gate_passed') is not True:
        raise AdmissionError('budget-time gate document did not pass')
    results = document.get('results')
    if type(results) is not list or len(results) != len(GROUPS):
        raise AdmissionError('budget-time gate document group count mismatch')
    gate_meters = {}
    for result, group in zip(results, GROUPS):
        if result.get('groups') != [group] or result.get('outcome') != 'PASS':
            raise AdmissionError('budget-time gate document groups mismatch')
        gate_meters[group] = _meter(result.get('execution'), f'gate {group}')
    exact_keys(optimized_ref, ('path', 'sha256'), 'budget-time optimized binding')
    evidence.sha256(optimized_ref['path'], optimized_ref['sha256'])
    optimized, _ = evidence.read_json(optimized_ref['path'])
    if optimized.get('exit') not in (0, 1) or 'meter' not in optimized:
        raise AdmissionError('optimized observer command document malformed')
    optimized_meter = _meter(optimized, 'optimized observer')
    policies = budget_contracts.get('policies') if isinstance(budget_contracts, dict) else None
    if type(policies) is not dict or not policies:
        raise AdmissionError('budget policy declarations missing')
    rows = []
    for group in GROUPS:
        rows.append({'id': f'gate-meter/{group}', 'verdict': 'PASS',
                     'measured': gate_meters[group], 'missing': []})
    rows.append({'id': 'optimized-original-observer-meter', 'verdict': 'PASS',
                 'measured': optimized_meter, 'missing': []})
    if type(admission_peak) is int and 0 < admission_peak <= LIVE_CAP:
        rows.append({'id': 'composer-live-cap-48mib', 'verdict': 'PASS',
                     'measured': {'peak_reserved_bytes': admission_peak}, 'missing': []})
    else:
        rows.append({'id': 'composer-live-cap-48mib', 'verdict': 'FAIL',
                     'measured': {'peak_reserved_bytes': admission_peak},
                     'missing': ['composer live reservation exceeds the 48 MiB cap']})
    declared = sum(policy.get('runtime_verdict') == 'UNOBSERVED' for policy in policies.values())
    if samples_ref is not None:
        stats = admit_policy_runtime_samples(evidence, samples_ref, document, gate_ref['sha256'])
        foreign = sorted(set(stats) - set(policies))
        if foreign:
            raise AdmissionError('sample binding carries foreign policies: ' + ','.join(foreign))
        observed = {name: s for name, s in stats.items() if s['n'] >= MIN_SAMPLES}
        unobserved = sorted(set(policies) - set(observed))
        # Every measured window must respect its policy's wall/total budgets
        # where the declaration names them (single-window budgets, not P90).
        for name, s in observed.items():
            limits = policies[name]['limits']
            budget = limits.get('total_ms') or limits.get('wall_ms') or limits.get('build_ms')
            if budget and s['max_ms'] > budget:
                raise AdmissionError(f'policy {name} sample window exceeds its declared budget')
        rows.append({'id': 'policy-runtime-samples', 'verdict': 'PASS' if not unobserved else 'PARTIAL',
                     'measured': {'declared_policies': len(policies),
                                  'runtime_observed': len(observed),
                                  'per_policy': observed},
                     'missing': [f'{name} lacks a measured runtime window' for name in unobserved]
                                + (['declarations never prove runtime'] if declared else [])})
    else:
        rows.append({'id': 'policy-runtime-samples', 'verdict': 'PARTIAL',
                     'measured': {'declared_policies': len(policies), 'runtime_unobserved': declared},
                     'missing': ['per-policy P90/peak runtime sample windows (declarations never prove runtime)']})
    counts = {state: sum(row['verdict'] == state for row in rows)
              for state in ('PASS', 'PARTIAL', 'FAIL', 'UNOBSERVED')}
    return {'schema': 'ace.m1.budget-time-measurements/1',
            'rows': rows,
            'gate': {'budgets_time_complete': counts['PASS'] == len(rows)},
            'counts': counts,
            'note': 'Real measured meters from admitted documents; policy observations '
                    'count only really-measured sample windows (>=10 each, P90 by the '
                    'N-floor(N/10) rank); declarations never prove runtime.'}
