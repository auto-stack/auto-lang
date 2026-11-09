"""Bounded T-02 preview using original raw evidence, never a milestone claim.

Parent bindings are explicitly prepared by the formal identity exporter. Each
source/contract/oracle file and current binary is rehashed at admission. This
preview precedes the complete T-03 registry and cannot deliver that registry.
"""
import hashlib
import json
from pathlib import PureWindowsPath

from ace_m1_arch_acceptance import EvidenceScope, LiveBudget, exact_keys, reduce_acceptance
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, admit_exchange, require
from ace_m1_legacy import admit_accounting, typed_equal
from ace_m1_scalar_evidence import scalar_observation
from ace_m1_cert_evidence import scalar_certification
from ace_m1_daily_evidence import certified_daily_emission
from ace_m1_syntax_evidence import syntax_parse_observation
from ace_m1_abi_evidence import bridge_store_query_observations

REGISTRY = 'crates/auto-lang/test/cases/ace-m1-expanded/registry.json'
# PLAN-941 T-03/T-09: content pins of the registry's awaiting_d section (the
# seven still-awaiting PLAN-939 promises) and the delivered_d section (the
# seven PLAN-938 promises delivered by archived D d88b0bc0e and received into
# the 941 tree in round 94; see ace_m1_registry).
AWAITING_D_SHA256 = '4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945'
DELIVERED_D_SHA256 = '2088410f5d2a75bb58de94711bbdf3d57ffb91f1834f9fbc583d1f9aa1a808d4'
TARGETS = ('AC-native', 'AV', 'ACE-A2R-HIR', 'ACE-A2C-HIR')
SYNTAX = ('parse-core', 'parse-scalar', 'parse-text', 'parse-record', 'parse-text-record',
          'parse-entry-unit', 'parse-host-console', 'parse-closures', 'parse-collections',
          'parse-record-under-scalar-grammar-refused')
BRIDGE = ('source-export', 'source-body-edit', 'typed-self', 'typed-n2c', 'typed-c2n', 'bad-leg')
REQUIRED = tuple('q4/' + name for name in TARGETS) + tuple('syntax/' + name for name in SYNTAX) + tuple('bridge/' + name for name in BRIDGE)
SUITES = {
    'q4': 'crates/auto-lang/test/cases/ace-m1-expanded/ace-m1-expanded.suite.ad',
    'syntax': 'crates/auto-lang/test/cases/ace-m1/unified-source-syntax-929/unified-source-syntax-929.suite.ad',
    'bridge': 'crates/auto-lang/test/cases/ace-ca-d-scalar-937/ace-ca-d-scalar-937.suite.ad',
    'legacy': 'crates/auto-lang/test/cases/ace-m1-expanded/legacy-coverage.suite.ad',
    'typed_av': 'crates/auto-lang/test/cases/ac-av/ac-av.suite.ad',
    'typed_native': 'crates/auto-lang/test/cases/ace-m1/typed-native-891/typed-native-891.suite.ad',
    'source_cte': 'crates/auto-lang/test/cases/ace-m1/source-cte-936/source-cte-936.suite.ad',
}
WORKERS = {'av': 'av-worker.exe', 'native': 'mn-worker.exe',
           'syntax': 'ace-case-worker.exe', 'bridge': 'abi-worker.exe'}


def _compact_admission(inputs, reader, retained=512 << 10):
    """Owned bounded readers return compact observations/IDs, never raw documents.

    Their raw buffers, decoded tables and indexes die on return. Reserve the
    retained summary before discovery; release only those transient allocations.
    Callers are internal, closed-schema readers, not arbitrary plugins.
    """
    before = inputs.budget.reserved
    inputs.reserve(retained)
    result = reader(inputs)
    raw = json.dumps(result, ensure_ascii=False, allow_nan=False, separators=(',', ':')).encode()
    nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
    summary_bound = 7 * len(raw) + 256 * nodes + 4096
    require(summary_bound <= retained, 'discovery summary exceeds retained reservation')
    # `reader` has returned: no raw/decoded source table escapes either of
    # these owned admission APIs. Keep the same conservative strings/nodes
    # bound used above, including the encoded summary still live here. Unused
    # reservation padding is no longer live; releasing it never resets peak.
    inputs.budget.release_transient(inputs.budget.reserved - before - summary_bound)
    return result


def _repo_relative(inputs, name):
    require(type(name) is str, 'current input name must be text')
    if name.startswith('\\\\?\\'):
        name = name[4:]
    relative = PureWindowsPath(name).relative_to(PureWindowsPath(str(inputs.root))).as_posix()
    inputs.path(relative)
    return relative


def inventory_contract(inputs, inventory):
    """Portable typed contract snapshot, prepared by the formal exporter."""
    rows = []
    for row in inventory['checks']:
        snapshot = {key:value for key, value in row.items()
                    if key not in ('case_file', 'input_paths', 'span', 'defaults')}
        snapshot['case_file'] = _repo_relative(inputs, row['case_file'])
        snapshot['input_paths'] = [_repo_relative(inputs, name) for name in row['input_paths']]
        rows.append(snapshot)
    return {'schema': inventory['schema'], 'suite': inventory['suite'],
            'suite_sha256': inventory['suite_sha256'], 'checks': rows}


def _inventory(inputs, evidence, ref, key, frozen):
    return _inventory_for_suite(inputs, evidence, ref, SUITES[key], frozen)


def _inventory_for_suite(inputs, evidence, ref, suite, frozen):
    exact_keys(ref, ('path', 'sha256'), 'current parent inventory reference')
    evidence.sha256(ref['path'], ref['sha256'])
    inventory, _ = evidence.read_json(ref['path'])
    require(inventory.get('schema') == 'test-case-inventory/2', 'formal identity inventory required')
    typed_equal(_repo_relative(inputs, inventory['suite_path']), suite, 'frozen suite identity')
    typed_equal(inventory_contract(inputs, inventory), frozen, 'tracked independent typed contract')
    inputs.sha256(_repo_relative(inputs, inventory['suite_path']), inventory['suite_sha256'])
    for check in inventory['checks']:
        inputs.sha256(_repo_relative(inputs, check['case_file']), check['case_file_sha256'])
        require(len(check['source_files']) == 1, 'one current declared source required')
        exact_keys(check['source_origin'], ('kind',), 'declared source origin')
        origin = check['source_origin']['kind']
        require(origin in ('file', 'text', 'block'), 'unknown declared source origin')
        if origin == 'file':
            require(bool(check['input_paths']), 'file-backed first slice required')
            inputs.sha256(_repo_relative(inputs, check['input_paths'][0]), check['source_files'][0]['sha256'])
        # Text/Block bytes belong to the current Case document whose content
        # hash and resolved typed contract were checked above. No Debug parse.
        for artifact in check['oracle_artifacts']:
            name = str(PureWindowsPath(inventory['fixture_base']) / artifact['file'])
            inputs.sha256(_repo_relative(inputs, name), artifact['sha256'])
    return inventory


def _wires(copy, check, protocol, recipe, worker):
    names = [name for name in copy.files if name.endswith('-start.json')
             and '/' + check['case'] + '/' + check['check'] + '/worker-exchanges/' in name]
    names.sort(key=lambda name: int(name.rsplit('/', 1)[-1].split('-')[1]))
    require(bool(names), 'required original exchange missing')
    return [admit_exchange(copy, name, protocol=protocol, recipe=recipe, worker_sha256=worker) for name in names]


def reduce_preview(registry_sha, observations, legacy):
    """Pure AND with a fixed denominator; isolated controls may call this."""
    require(type(observations) is dict and observations.keys() <= set(REQUIRED), 'extra preview obligation')
    rows = [{'id': key, 'verdict': observations.get(key, {}).get('judgement', 'UNOBSERVED')}
            for key in REQUIRED]
    require(all(row['verdict'] in ('PASS', 'FAIL', 'UNOBSERVED') for row in rows), 'unknown preview judgement')
    closed = all(row['verdict'] == 'PASS' for row in rows) and legacy['accounting_valid'] is True
    gates = {'first_slice_evidence': closed,
             'legacy_current_gate': legacy['can_claim_m1'] is True,
             'legacy_required_execution': legacy['execution'] == 'PASS',
             'complete_expanded_registry': False}
    return {'schema': 'ace.m1.acceptance/3', 'mode': 'preview', 'registry_sha256': registry_sha,
            'first_slice_required': len(REQUIRED), 'first_slice_rows': rows,
            'first_slice_evidence_complete': closed, 'domain_observations': observations,
            'legacy': legacy, 'gates': gates,
            'accounting_valid': legacy['accounting_valid'],
            'evidence_complete': False, 'acceptance_system_delivered': False,
            'can_claim_expanded_m1': False,
            'blockers': [key for key, passed in gates.items() if not passed],
            'note': 'Bounded T-02 preview; T-01 mapping/T-03 full registry and remaining mechanisms are incomplete.'}


def retain_parent_status(observation, status):
    require(status in ('PASS', 'FAIL', 'ERROR', 'SKIP'), 'unknown original Case status')
    result = observation | {'parent_status':status}
    if status != 'PASS':
        result['judgement'] = 'FAIL'
    return result


def retain_case_reports(report, parents):
    """Retain every declared parent rejection, including daily-only checks."""
    parent_pass = all(not parent['non_pass_rows'] for parent in parents.values())
    report['source_case_reports'] = parents
    report['gates']['source_case_reports'] = parent_pass
    if not parent_pass:
        report['first_slice_evidence_complete'] = False
        report['gates']['first_slice_evidence'] = False
    report['blockers'] = [key for key, passed in report['gates'].items() if not passed]
    return report




def _slim_obs(observations):
    """Reduce any observation set to the fields the gate and report need.

    The full per-suite documents stay in their own bounded sections and the
    archived copies; the merged denominator only consumes judgements."""
    return {k: {'judgement': v.get('judgement'), 'oracle_source': v.get('oracle_source'),
                'parent_status': v.get('parent_status')} if isinstance(v, dict) else {'judgement': v}
            for k, v in observations.items()}

def _admit_base_slice(inputs, evidence, bindings, registry, workers):
    """Raw base inventories/copies die before later domain admission."""
    budget = inputs.budget
    inventories = {key: _inventory(inputs, evidence, ref, key, registry['inventory_contracts'][key])
                   for key, ref in bindings['inventories'].items()}
    copies = {key: CaseCopy(EvidenceScope(evidence.path(name + "/artifact-copy.json").parent, budget)) for key, name in bindings['copies'].items()}
    observations, parents = {}, {}
    for key, inventory in inventories.items():
        copy = copies[key]
        admitted = admit_case_bindings(copy, inventory)
        parents[key] = {'report_sha256':admitted['report_sha256'], 'non_pass_rows':[
            {'case':check['case'], 'check':check['check'], 'status':row['status']}
            for check, row in admitted['checks'] if row['status'] != 'PASS']}
        for check, row in admitted['checks']:
            obligation = None
            if key == 'q4' and check['check'] in ('av', 'ac-native'):
                av = check['check'] == 'av'
                wire = _wires(copy, check, 'avworker/1' if av else 'mnworker/1', check['recipe'], workers['av' if av else 'native'])
                require(len(wire) == 1, 'one scalar exchange required')
                obligation = 'q4/' + ('AV' if av else 'AC-native')
                observation = scalar_observation(copy.scope, wire[0], check)
            elif key == 'syntax' and check['adapter'] == 'syntax':
                wire = _wires(copy, check, 'acworker/1', 'syntax/' + check['ac_options']['grammar'], workers['syntax'])
                require(len(wire) == 1, 'one syntax exchange required')
                obligation = 'syntax/' + check['check']
                observation = syntax_parse_observation(copy, wire[0], check)
            elif key == 'bridge':
                # D937-final store seam: the worker-exchange frames are gone;
                # the observation consumes the parent Job samples plus the
                # store's product surface (ace_m1_abi_evidence).
                source = inputs.read_bytes(_repo_relative(inputs, check['input_paths'][0]))
                obligation = 'bridge/' + check['check']
                observation = bridge_store_query_observations(copy, check, source)
            if obligation is not None:
                observations[obligation] = retain_parent_status(observation, row['status'])
    exact_keys(bindings['certification'], ('prefix', 'bindings'), 'explicit certification references')
    pins, _ = evidence.read_json(bindings['certification']['bindings'])
    exact_keys(pins, ('producer_commit', 'worker_path', 'worker_sha256', 'cwd', 'targets'), 'current certification pins')
    exact_keys(pins['targets'], ('rust', 'c'), 'required explicit certification targets')
    producer = {key:value for key, value in pins.items() if key != 'targets'}
    typed_equal(_repo_relative(inputs, producer['worker_path']),
                'experimental/ace/target/release/a2x-worker.exe', 'formal certification worker path')
    for target, pin in pins['targets'].items():
        prefix = bindings['certification']['prefix'] + '/' + target
        result = certified_daily_emission(evidence, inputs, prefix, pin, producer,
                                            scalar_certification(evidence, inputs, prefix, pin, producer))
        q4 = inventories['q4']['checks'][0]
        typed_equal(pin['source_sha256'], q4['source_files'][0]['sha256'], 'Q4 certificate source')
        typed_equal(pin['request']['oracle_value'], q4['expected_exit'], 'Q4 certificate independent oracle')
        observations['q4/' + ('ACE-A2R-HIR' if target == 'rust' else 'ACE-A2C-HIR')] = result
    legacy = admit_accounting(evidence, bindings['legacy'], inputs)
    if 'legacy_execution' in bindings:
        from ace_m1_legacy_execution import admit_scalar_slice
        leg = bindings['legacy_execution']
        exact_keys(leg, ('inventory', 'copy'), 'original execution bindings')
        expected = _inventory(inputs, evidence, leg['inventory'], 'legacy', registry['inventory_contracts']['legacy'])
        copy = CaseCopy(EvidenceScope(evidence.path(leg['copy'] + '/artifact-copy.json').parent, budget))
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: admit_scalar_slice(copy, expected, inputs, workers))
        legacy['execution'] = legacy['execution_summary']['verdict']
    return {'observations':observations, 'parents':parents, 'legacy':legacy}


def compose_first_slice(repository_root, evidence_root, bindings_path):
    budget = LiveBudget()
    inputs = EvidenceScope(repository_root, budget)
    evidence = EvidenceScope(evidence_root, budget)
    bindings, bindings_sha = evidence.read_json(bindings_path)
    mandatory = {'inventories', 'copies', 'workers', 'certification', 'legacy'}
    require(type(bindings) is dict and mandatory <= bindings.keys() <= mandatory | {'legacy_base_execution', 'legacy_execution', 'legacy_a2r_execution', 'legacy_probe_families', 'legacy_rust_tests', 'legacy_governance', 'parent_contracts', 'legacy_typed_av', 'legacy_typed_native', 'source_cte_execution', 'text_certifications', 'source_cte_certifications', 'text_q4_execution', 'scalar_q4_execution', 'scalar_certifications', 'scalar_release_execution', 'record_text_execution', 'record_text_certifications', 'record_text_release_execution', 'collections_execution', 'collections_certifications', 'query_instances_execution', 'query_certifications', 'query_frontdoor_refusals', 'query_release_execution', 'source_supplement_execution', 'source_supplement_certifications', 'original_data_execution', 'original_data_certifications', 'original_data_diagnostics', 'ct_source_mechanisms', 'ct_child_mechanisms', 'ct_meta_mechanisms', 'ct_restore_mechanisms', 'sq_fact_mechanisms', 'syntax_mechanisms', 'legacy_architecture', 'incremental_restore_gates', 'incremental_restore_semantics', 'incremental_restore_comptime', 'incremental_restore_abi', 'incremental_restore_av', 'budget_time_gates', 'budget_time_optimized', 'acceptance_system_controls', 'policy_runtime_samples'},
            'unknown/missing preview parent bindings')
    exact_keys(bindings['workers'], ('av', 'native', 'syntax', 'bridge'), 'current preview workers')
    require('scalar_release_execution' not in bindings or 'scalar_certifications' in bindings,
            'release observations require original scalar certification bindings')
    require('record_text_release_execution' not in bindings or 'record_text_certifications' in bindings,
            'release observations require original record/text certification bindings')
    require('query_release_execution' not in bindings or 'query_certifications' in bindings,
            'query optimization observations require original default certifications')
    workers = {}
    for key, ref in bindings['workers'].items():
        exact_keys(ref, ('path', 'sha256'), 'current binary reference')
        typed_equal(ref['path'], 'experimental/ace/target/release/' + WORKERS[key], 'formal current worker path')
        inputs.sha256(ref['path'], ref['sha256'])
        workers[key] = ref['sha256']
    sq_facts = None
    if 'sq_fact_mechanisms' in bindings:
        from ace_m1_sq_facts import admit as admit_sq_facts
        sq_facts = _compact_admission(inputs, lambda _: admit_sq_facts(
            inputs, evidence, bindings['sq_fact_mechanisms']), 128 << 10)
    if 'ct_restore_mechanisms' in bindings:
        from ace_m1_ct_restore import admit as admit_ct_restore
        result_restore = _compact_admission(inputs, lambda _: admit_ct_restore(
            inputs, evidence, bindings['ct_restore_mechanisms']), 128 << 10)
    else:
        result_restore = None
    ct_meta = None
    if 'ct_meta_mechanisms' in bindings:
        from ace_m1_ct_meta import admit as admit_ct_meta
        ct_meta = _compact_admission(inputs, lambda _: admit_ct_meta(
            inputs, evidence, bindings['ct_meta_mechanisms']), 128 << 10)
    ct_child = None
    if 'ct_child_mechanisms' in bindings:
        from ace_m1_ct_child import admit as admit_ct_child
        ct_child = _compact_admission(inputs, lambda _: admit_ct_child(
            inputs, evidence, bindings['ct_child_mechanisms']), 128 << 10)
    ct_mechanisms = None
    if 'ct_source_mechanisms' in bindings:
        from ace_m1_ct_mechanisms import admit as admit_ct_mechanisms
        ct_mechanisms = _compact_admission(inputs, lambda _: admit_ct_mechanisms(
            inputs, evidence, bindings['ct_source_mechanisms']), 256 << 10)
    # The complete old BASE CaseCopy is admitted before other summaries grow.
    # Its transient peak remains in this SAME budget; only its bounded summary
    # stays live during later source/certification reads.
    legacy_base = None
    if 'legacy_base_execution' in bindings:
        from ace_m1_legacy_base import admit as admit_legacy_base
        legacy_base = _compact_admission(inputs, lambda _: admit_legacy_base(
            inputs, evidence, bindings['legacy_base_execution'], workers), 1 << 20)
    original_data_diagnostics = None
    if 'original_data_diagnostics' in bindings:
        from ace_m1_data_diagnostics import admit_diagnostics
        original_data_diagnostics = _compact_admission(inputs, lambda _: admit_diagnostics(
            inputs, evidence, bindings['original_data_diagnostics'], workers), 512 << 10)
    original_data_certifications = None
    if 'original_data_certifications' in bindings:
        from ace_m1_original_data_sources import admit_certifications
        original_data_certifications = _compact_admission(inputs, lambda _: admit_certifications(
            inputs, evidence, bindings['original_data_certifications']), 512 << 10)
    original_data = None
    if 'original_data_execution' in bindings:
        from ace_m1_original_data_sources import admit_source
        original_data = _compact_admission(inputs, lambda _: admit_source(
            inputs, evidence, bindings['original_data_execution'], workers), 512 << 10)
    legacy_data = None
    if original_data is not None or original_data_diagnostics is not None:
        from ace_m1_legacy_data import admit as admit_legacy_data
        legacy_data = _compact_admission(inputs, lambda _: admit_legacy_data(
            inputs, original_data, original_data_diagnostics), 1 << 20)
    supplement_certifications = None
    if 'source_supplement_certifications' in bindings:
        from ace_m1_source_supplement import admit_certifications
        supplement_certifications = _compact_admission(inputs, lambda _: admit_certifications(
            inputs, evidence, bindings['source_supplement_certifications']), 512 << 10)
    supplement_result = None
    if 'source_supplement_execution' in bindings:
        from ace_m1_source_supplement import admit_source
        supplement_result = _compact_admission(inputs, lambda _: admit_source(
            inputs, evidence, bindings['source_supplement_execution'], workers), 512 << 10)
    scalar_result = None
    if 'scalar_q4_execution' in bindings:
        from ace_m1_scalar_q4_evidence import admit_scalar_q4
        scalar_result = _compact_admission(inputs, lambda _: admit_scalar_q4(
            inputs, evidence, bindings['scalar_q4_execution'], workers), 3 << 20)
    scalar_certifications = None
    if 'scalar_certifications' in bindings:
        from ace_m1_scalar_cert_evidence import admit_scalar_certifications
        def _scalar_certs(_):
            value = admit_scalar_certifications(
                inputs, evidence, bindings['scalar_certifications'], bindings.get('scalar_release_execution'))
            value['observations'] = _slim_obs(value.get('observations', {}))
            return value
        scalar_certifications = _compact_admission(inputs, _scalar_certs, 1 << 20)
    rt_source = rt_certifications = None
    if 'record_text_execution' in bindings:
        from ace_m1_record_text_evidence import admit_source
        rt_source = _compact_admission(inputs, lambda _: admit_source(
            inputs, evidence, bindings['record_text_execution'], workers), 1 << 20)
    if 'record_text_certifications' in bindings:
        from ace_m1_record_text_evidence import admit_certifications
        rt_certifications = _compact_admission(inputs, lambda _: admit_certifications(
            inputs, evidence, bindings['record_text_certifications'], bindings.get('record_text_release_execution')), 896 << 10)
    collections_source = collections_certifications = None
    if 'collections_execution' in bindings:
        from ace_m1_collections_evidence import admit_source
        collections_source = _compact_admission(inputs, lambda _: admit_source(
            inputs, evidence, bindings['collections_execution'], workers), 1 << 20)
    if 'collections_certifications' in bindings:
        from ace_m1_collections_evidence import admit_certifications
        collections_certifications = _compact_admission(inputs, lambda _: admit_certifications(
            inputs, evidence, bindings['collections_certifications']), 1 << 20)
    query_instances_source = None
    if 'query_instances_execution' in bindings:
        from ace_m1_query_evidence import admit_source
        query_instances_source = _compact_admission(inputs, lambda _: admit_source(
            inputs, evidence, bindings['query_instances_execution'], workers), 1 << 20)
    query_certifications = query_frontdoor_refusals = None
    if 'query_certifications' in bindings:
        from ace_m1_query_evidence import admit_certifications
        query_certifications = _compact_admission(inputs, lambda _: admit_certifications(
            inputs, evidence, bindings['query_certifications'], bindings.get('query_release_execution')), 896 << 10)
    if 'query_frontdoor_refusals' in bindings:
        from ace_m1_query_evidence import admit_frontdoor_refusals
        query_frontdoor_refusals = _compact_admission(inputs, lambda _: admit_frontdoor_refusals(
            inputs, evidence, bindings['query_frontdoor_refusals']), 128 << 10)
    registry, registry_sha = inputs.read_json(REGISTRY)
    exact_keys(registry, ('schema', 'scope', 'full_registry_closed', 'required', 'awaiting_d', 'delivered_d', 'inventory_contracts', 'parent_contracts', 'expanded_contracts'), 'expanded registry')
    typed_equal({key:value for key, value in registry.items() if key not in ('awaiting_d', 'delivered_d', 'inventory_contracts', 'parent_contracts', 'expanded_contracts')},
                {'schema': 'ace.m1.expanded-registry/2', 'scope': 'universe-closed',
                 'full_registry_closed': True, 'required': list(REQUIRED)}, 'closed-universe registry')
    # PLAN-941 T-03: the awaiting-D promise rows (PLAN-938/939 work-stage
    # acceptance criteria) are pinned by content hash — any edit to an owner,
    # promise, direction or close condition is rejected before admission.
    typed_equal(hashlib.sha256(json.dumps(registry['awaiting_d'], ensure_ascii=False,
                                           sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest(),
                AWAITING_D_SHA256, 'awaiting-D universe rows')
    # T-09: the delivered PLAN-938 rows are pinned the same way — any edit to
    # a promise, close condition or the named delivery commit is refused.
    typed_equal(hashlib.sha256(json.dumps(registry['delivered_d'], ensure_ascii=False,
                                           sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest(),
                DELIVERED_D_SHA256, 'delivered-D universe rows')
    exact_keys(registry['inventory_contracts'], SUITES, 'tracked typed inventories')
    exact_keys(bindings['inventories'], ('q4', 'syntax', 'bridge'), 'preview inventories')
    exact_keys(bindings['copies'], ('q4', 'syntax', 'bridge'), 'preview copies')
    base = _compact_admission(inputs, lambda _: _admit_base_slice(inputs, evidence, bindings, registry, workers), 2 << 20)
    observations, parents, legacy = base['observations'], base['parents'], base['legacy']
    if legacy_base is not None:
        from ace_m1_legacy_execution import execution_summary
        earlier = legacy.get('execution_summary', {}).get('observations', {})
        require(not earlier.keys() & legacy_base['observations'].keys(), 'duplicate original BASE execution')
        required = sorted(set(legacy_base['unobserved_cells']) | legacy_base['observations'].keys())
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: execution_summary(
            required, earlier | legacy_base['observations']), 1 << 20)
        legacy['execution'] = legacy['execution_summary']['verdict']
        parents['legacy_base'] = legacy_base['source_case_report']
        legacy['base_pending_observations'] = legacy_base['pending_observations']
    if legacy_data is not None:
        from ace_m1_legacy_execution import execution_summary
        earlier = legacy.get('execution_summary', {}).get('observations', {})
        require(not earlier.keys() & legacy_data['observations'].keys(), 'duplicate original data execution')
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: execution_summary(
            legacy_data['required'], earlier | legacy_data['observations']), 2 << 20)
        legacy['execution'] = legacy['execution_summary']['verdict']
        legacy['data_pending_observations'] = legacy_data['pending_observations']
    if 'legacy_a2r_execution' in bindings:
        from ace_m1_legacy_a2r import admit_a2r_execution
        def _a2r_exec(_):
            value = admit_a2r_execution(inputs, evidence, bindings['legacy_a2r_execution'], registry)
            value['observations'] = _slim_obs(value.get('observations', {}))
            value.pop('unobserved_cells', None)
            return value
        a2r = _compact_admission(inputs, _a2r_exec, 1 << 20)
        from ace_m1_legacy_execution import execution_summary as _es
        from ace_m1_legacy import frozen_ledger
        earlier = legacy.get('execution_summary', {}).get('observations', {})
        require(not earlier.keys() & a2r['observations'].keys(), 'duplicate original a2r execution')
        legacy.setdefault('_merged_obs', {}).update(_slim_obs(a2r['observations']))
        legacy['execution'] = legacy['execution_summary']['verdict']
        legacy['a2r_claimed_cells'] = a2r['claimed_cells']
    if 'legacy_probe_families' in bindings:
        from ace_m1_legacy_a2r import admit_probe_families
        def _probe_families(_):
            value = admit_probe_families(inputs, evidence, bindings['legacy_probe_families'], registry)
            value['observations'] = _slim_obs(value.get('observations', {}))
            return value
        families = _compact_admission(inputs, _probe_families, 1 << 20)
        from ace_m1_legacy_execution import execution_summary as _es2
        from ace_m1_legacy import frozen_ledger as _fl
        earlier2 = legacy.get('execution_summary', {}).get('observations', {})
        # The pre-existing chains (scalar/base/typed/data/a2r) are the primary
        # owners of their cells; probe families only supply the remainder.
        fresh_family_cells = {k: {'judgement': v['judgement'],
                                  'supporting': [r['suite'] + ':' + r['check'] for r in v.get('supporting_checks', [])][:3]}
                              for k, v in families['observations'].items() if k not in earlier2}
        families['claimed_cells'] = len(fresh_family_cells)
        required2 = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in _fl(inputs) if row['required'])
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: _es2(
            required2, _slim_obs(earlier2) | _slim_obs(fresh_family_cells)), 896 << 10)
        legacy['execution'] = legacy['execution_summary']['verdict']
        legacy['probe_family_claimed_cells'] = families['claimed_cells']
    if 'legacy_rust_tests' in bindings:
        from ace_m1_legacy_a2r import admit_rust_test_cells
        rust = _compact_admission(inputs, lambda _: admit_rust_test_cells(
            inputs, evidence, bindings['legacy_rust_tests'], registry), 512 << 10)
        from ace_m1_legacy_execution import execution_summary as _es3
        from ace_m1_legacy import frozen_ledger as _fl3
        earlier3 = legacy.get('execution_summary', {}).get('observations', {})
        rust['observations'] = {k: {'judgement': v['judgement'], 'oracle_source': v.get('oracle_source'),
                                    'test': v.get('test')}
                                for k, v in rust['observations'].items() if k not in earlier3}
        rust['claimed_cells'] = len(rust['observations'])
        required3 = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in _fl3(inputs) if row['required'])
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: _es3(
            required3, _slim_obs(earlier3) | _slim_obs(rust['observations'])), 896 << 10)
        legacy['execution'] = legacy['execution_summary']['verdict']
        legacy['rust_test_claimed_cells'] = rust['claimed_cells']
    if 'legacy_governance' in bindings:
        from ace_m1_legacy_a2r import admit_governance_cells
        gov = _compact_admission(inputs, lambda _: admit_governance_cells(
            inputs, evidence, bindings['legacy_governance'], registry), 1 << 20)
        from ace_m1_legacy_execution import execution_summary as _es4
        from ace_m1_legacy import frozen_ledger as _fl4
        earlier4 = legacy.get('execution_summary', {}).get('observations', {})
        gov['observations'] = {k: {'judgement': v['judgement'], 'oracle_source': v.get('oracle_source')}
                               for k, v in gov['observations'].items() if k not in earlier4}
        gov['claimed_cells'] = len(gov['observations'])
        required4 = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in _fl4(inputs) if row['required'])
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: _es4(
            required4, _slim_obs(earlier4) | _slim_obs(gov['observations'])), 896 << 10)
        legacy['execution'] = legacy['execution_summary']['verdict']
        legacy['governance_claimed_cells'] = gov['claimed_cells']
        for key, parent in families['source_case_reports'].items():
            parents['probe:' + key] = parent
    result = retain_case_reports(reduce_preview(registry_sha, observations, legacy), parents)
    if original_data_diagnostics is not None:
        result['original_data_diagnostics'] = original_data_diagnostics
        parents['original_data_diagnostics'] = original_data_diagnostics['source_case_report']
        retain_case_reports(result, parents)
    if original_data_certifications is not None:
        original_data_certifications['observations'] = _slim_obs(original_data_certifications.get('observations', {}))
        result['original_data_certifications'] = original_data_certifications
    if original_data is not None:
        result['original_data_execution'] = original_data
        parents['original_data'] = original_data['source_case_report']
        retain_case_reports(result, parents)
    if supplement_certifications is not None:
        supplement_certifications['observations'] = _slim_obs(supplement_certifications.get('observations', {}))
        result['source_supplement_certifications'] = supplement_certifications
    if supplement_result is not None:
        result['source_supplement_execution'] = supplement_result
        parents['source_supplement'] = supplement_result['source_case_report']
        retain_case_reports(result, parents)
    if scalar_certifications is not None:
        scalar_certifications['observations'] = _slim_obs(scalar_certifications.get('observations', {}))
        result['scalar_certifications'] = scalar_certifications
    if rt_certifications is not None:
        rt_certifications['observations'] = _slim_obs(rt_certifications.get('observations', {}))
        result['record_text_certifications'] = rt_certifications
    if rt_source is not None:
        result['record_text_execution'] = rt_source
        parents['record_text'] = rt_source['source_case_report']
        retain_case_reports(result, parents)
    if collections_certifications is not None:
        collections_certifications['observations'] = _slim_obs(collections_certifications.get('observations', {}))
        result['collections_certifications'] = collections_certifications
    if query_instances_source is not None:
        result['query_instances_execution'] = query_instances_source
        for family,parent in query_instances_source['source_case_reports'].items():parents[family] = parent
        retain_case_reports(result, parents)
    if query_certifications is not None:
        query_certifications['observations'] = _slim_obs(query_certifications.get('observations', {}))
        result['query_certifications'] = query_certifications
    if query_frontdoor_refusals is not None:result['query_frontdoor_refusals'] = query_frontdoor_refusals
    if collections_source is not None:
        result['collections_execution'] = collections_source
        parents['collections'] = collections_source['source_case_report']
        retain_case_reports(result, parents)
    if scalar_result is not None:
        result['scalar_q4_execution'] = scalar_result
        parents['scalar_q4'] = scalar_result['source_case_report']
        retain_case_reports(result, parents)
    if 'parent_contracts' in bindings:
        from ace_m1_parent_contracts import admit_parent_contracts
        result['parent_contracts'] = admit_parent_contracts(inputs, evidence, bindings['parent_contracts'], registry['parent_contracts'])
    else:
        result['parent_contracts'] = {'status':'UNOBSERVED', 'executed':0}
    if 'legacy_typed_av' in bindings:
        require(result['parent_contracts']['status'] == 'DISCOVERY', 'AV typed admission requires current original parent tables')
        from ace_m1_legacy_execution import admit_av_typed_slice, execution_summary
        from ace_m1_legacy import frozen_ledger
        leg = bindings['legacy_typed_av']
        exact_keys(leg, ('inventory', 'copy'), 'original AV typed bindings')
        expected = _inventory(inputs, evidence, leg['inventory'], 'typed_av', registry['inventory_contracts']['typed_av'])
        copy = CaseCopy(EvidenceScope(evidence.path(leg['copy'] + '/artifact-copy.json').parent, budget))
        typed = _compact_admission(inputs, lambda _: admit_av_typed_slice(copy, expected, inputs, workers, registry['parent_contracts']['snapshot']))
        existing_legacy = legacy.get('execution_summary', {}).get('observations', {})
        require(not existing_legacy.keys() & typed['observations'].keys(), 'duplicate original required execution')
        required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in frozen_ledger(inputs) if row['required'])
        legacy['execution_summary'] = execution_summary(required, existing_legacy | typed['observations'])
        legacy['execution_summary']['av_typed_controls'] = typed['typed_controls']
        legacy['execution'] = legacy['execution_summary']['verdict']
        result['gates']['legacy_required_execution'] = legacy['execution'] == 'PASS'
        parents['typed_av'] = typed['source_case_report']
        retain_case_reports(result, parents)
    if 'legacy_typed_native' in bindings:
        from ace_m1_legacy_execution import admit_native_typed_slice, execution_summary
        from ace_m1_legacy import frozen_ledger
        leg = bindings['legacy_typed_native']
        exact_keys(leg, ('inventory', 'copy'), 'original native typed bindings')
        expected = _inventory(inputs, evidence, leg['inventory'], 'typed_native', registry['inventory_contracts']['typed_native'])
        copy = CaseCopy(EvidenceScope(evidence.path(leg['copy'] + '/artifact-copy.json').parent, budget))
        typed = _compact_admission(inputs, lambda _: admit_native_typed_slice(copy, expected, inputs, workers))
        previous = legacy.get('execution_summary', {})
        observations = previous.get('observations', {})
        require(not observations.keys() & typed['observations'].keys(), 'duplicate original native execution')
        required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in frozen_ledger(inputs) if row['required'])
        legacy['execution_summary'] = execution_summary(required, observations | typed['observations'])
        if 'av_typed_controls' in previous:
            legacy['execution_summary']['av_typed_controls'] = previous['av_typed_controls']
        legacy['execution'] = legacy['execution_summary']['verdict']
        result['gates']['legacy_required_execution'] = legacy['execution'] == 'PASS'
        parents['typed_native'] = typed['source_case_report']
        retain_case_reports(result, parents)
    if 'source_cte_execution' in bindings:
        from ace_m1_cte_evidence import admit_cte_source_slice
        result['source_cte_execution'] = _compact_admission(inputs, lambda _: admit_cte_source_slice(
            inputs, evidence, bindings['source_cte_execution'], registry['inventory_contracts']['source_cte'], workers))
        parents['source_cte'] = result['source_cte_execution']['source_case_report']
        retain_case_reports(result, parents)
    if 'text_q4_execution' in bindings:
        from ace_m1_text_q4_evidence import admit_text_q4
        result['text_q4_execution'] = _compact_admission(inputs, lambda _: admit_text_q4(
            inputs, evidence, bindings['text_q4_execution'], workers))
        parents['text_q4'] = result['text_q4_execution']['source_case_report']
        retain_case_reports(result, parents)
    if 'text_certifications' in bindings:
        from ace_m1_text_evidence import admit_text_certifications
        _text = _compact_admission(inputs, lambda _: admit_text_certifications(
            inputs, evidence, bindings['text_certifications']))
        _text['observations'] = _slim_obs(_text.get('observations', {}))
        result['text_certifications'] = _text
    if 'source_cte_certifications' in bindings:
        from ace_m1_cte_evidence import admit_cte_certifications
        result['source_cte_certifications'] = _compact_admission(inputs, lambda _: admit_cte_certifications(
            inputs, evidence, bindings['source_cte_certifications']))
    # PLAN-1000 T-05: certification observations are slimmed to what the
    # registered gate and report consume (judgement + source); full documents
    # stay in their bounded per-family sections and archived captures.
    for _fam in ('text_certifications', 'scalar_certifications', 'record_text_certifications',
                 'collections_certifications', 'query_certifications', 'source_supplement_certifications',
                 'original_data_certifications'):
        _section = result.get(_fam)
        if isinstance(_section, dict):
            _obs = _section.get('observations')
            if isinstance(_obs, dict):
                _section['observations'] = _slim_obs(_obs)
    if '_merged_obs' in legacy:
        from ace_m1_legacy_execution import execution_summary as _esf
        from ace_m1_legacy import frozen_ledger as _flf
        _merged = _slim_obs(legacy.pop('_merged_obs'))
        _earlier = legacy.get('execution_summary', {}).get('observations', {})
        _required = sorted(':'.join((row['ns'], row['id'], row['path'])) for row in _flf(inputs) if row['required'])
        legacy['execution_summary'] = _compact_admission(inputs, lambda _: _esf(
            _required, _slim_obs(_earlier) | _merged), 1 << 20)
        legacy['execution'] = legacy['execution_summary']['verdict']
    from ace_m1_obligation_catalog import admit_catalogs
    from ace_m1_legacy_obligations import admit_legacy_obligations
    result['coverage_inventory'] = _compact_admission(inputs, admit_catalogs, 128 << 10)
    result['legacy_obligation_inventory'] = _compact_admission(inputs, admit_legacy_obligations, 128 << 10)
    from ace_m1_budget_contracts import admit_budget_contracts
    result['original_budget_contracts'] = _compact_admission(
        inputs, lambda _: admit_budget_contracts(inputs, registry['parent_contracts']))
    from ace_m1_arch_obligations import admit_architecture_contract
    result['architecture_obligations'] = _compact_admission(inputs, admit_architecture_contract)
    syntax_mechanisms = None
    if 'syntax_mechanisms' in bindings:
        from ace_m1_syntax_arch import admit as admit_syntax_arch
        syntax_mechanisms = _compact_admission(inputs, lambda _: admit_syntax_arch(
            inputs, result['domain_observations'], parents['syntax'], bindings['syntax_mechanisms'], workers['syntax']), 128 << 10)
    legacy_architecture = None
    if 'legacy_architecture' in bindings:
        from ace_m1_legacy_arch import admit as admit_legacy_arch
        legacy_architecture = _compact_admission(inputs, lambda _: admit_legacy_arch(
            inputs, evidence, bindings['legacy_architecture'], legacy), 128 << 10)
    if ct_mechanisms is not None or ct_child is not None or ct_meta is not None or sq_facts is not None or syntax_mechanisms is not None or legacy_architecture is not None:
        from ace_m1_arch_obligations import reduce_architecture
        arch_observations, arch_sources = {}, {}
        for label, admitted in (('ct_source_mechanisms', ct_mechanisms), ('ct_child_mechanisms', ct_child), ('ct_meta_mechanisms', ct_meta), ('sq_fact_mechanisms', sq_facts), ('syntax_mechanisms', syntax_mechanisms), ('legacy_architecture', legacy_architecture)):
            if admitted is None:
                continue
            require(not arch_observations.keys() & admitted['observations'].keys(), 'duplicate actual architecture observation')
            arch_observations.update(admitted['observations'])
            arch_sources[label] = admitted['source_evidence']
        result['architecture_execution'] = {
            'schema':'ace.m1.architecture-execution/1', 'observations':arch_observations,
            'sources':arch_sources, 'gate':reduce_architecture(arch_observations),
            'can_claim_architecture_pass':False}
    if result_restore is not None:
        result['ct_restore_execution'] = result_restore
    result['gates']['architecture_required'] = result.get('architecture_execution', result['architecture_obligations'])['gate']['complete']
    # PLAN-941 T-06: the incremental/restore measurement section — ARCH-05
    # row coverage composed from the admitted full-group gate document plus
    # the optional ace-semantics (934/935 page families) and ace-comptime
    # (meta dependency-edit family) companion documents and the composed
    # CT-restore/architecture observations. Composed before the registered
    # inventory so its PASS rows can claim the ARCH-05 obligations they are
    # the execution-side source for (see compose_registered_inventory).
    if 'incremental_restore_gates' in bindings:
        from ace_m1_incremental_restore import admit_incremental_restore_measurements
        result['incremental_restore_measurements'] = _compact_admission(
            inputs, lambda _: admit_incremental_restore_measurements(
                inputs, evidence, bindings['incremental_restore_gates'],
                result.get('ct_restore_execution'),
                result.get('architecture_execution', result.get('architecture_obligations', {}))
                    .get('gate', {}).get('rows'),
                bindings.get('incremental_restore_semantics'),
                bindings.get('incremental_restore_comptime'),
                bindings.get('incremental_restore_abi'),
                bindings.get('incremental_restore_av')), 256 << 10)
    from ace_m1_registry import compose_registered_inventory
    result['registered_obligations'] = _compact_admission(
        inputs, lambda _: compose_registered_inventory(inputs, registry, result, REQUIRED), 3 << 20)
    result['gates']['registered_required_execution'] = result['registered_obligations']['registered_observations_complete']
    result['gates']['complete_expanded_registry'] = result['registered_obligations']['required_universe_closed']
    # PLAN-941 T-07: the budget/time measurement section — real meters from
    # the admitted full-group gate and optimized-observer documents plus the
    # composer's own live reservation; policy declarations stay runtime-
    # UNOBSERVED (declarations never prove runtime).
    if 'budget_time_gates' in bindings:
        from ace_m1_budget_time import admit_budget_time_measurements
        result['budget_time_measurements'] = _compact_admission(
            inputs, lambda _: admit_budget_time_measurements(
                inputs, evidence, bindings['budget_time_gates'],
                bindings['budget_time_optimized'],
                result.get('original_budget_contracts'),
                budget.peak_reserved,
                bindings.get('policy_runtime_samples')), 256 << 10)
    # PLAN-941 T-10: the acceptance-system control family — the named
    # V-14/V-15 rejection controls must really execute green inside an
    # admitted governance gate document before the system counts as
    # delivered; controls never supply milestone evidence by themselves.
    system_controls = None
    if 'acceptance_system_controls' in bindings:
        from ace_m1_system_controls import admit_system_controls
        system_controls = _compact_admission(
            inputs, lambda _: admit_system_controls(
                inputs, evidence, bindings['acceptance_system_controls']), 128 << 10)
        result['acceptance_system_controls'] = system_controls
    # PLAN-941 T-04: the §5.2 total predicate. Each domain gate maps to the
    # sections actually composed above; compositions that do not exist yet are
    # named missing inputs and never silently count as green.
    def _section_all_pass(*names):
        for name in names:
            section = result.get(name)
            if not isinstance(section, dict):
                return False
            rows = section.get('observations', {}).values()
            # Section schemas differ: q4 families carry observation documents
            # with a `judgement` field; the CT-restore reader emits plain
            # verdict strings.
            judgements = [row.get('judgement') if isinstance(row, dict) else row for row in rows]
            if not judgements or any(value != 'PASS' for value in judgements):
                return False
        return True
    acceptance_gates = {
        'legacy_current_gate': result['legacy']['can_claim_m1'] is True,
        'legacy_required_execution': result['legacy']['execution'] == 'PASS',
        'c_core_and_resource': _section_all_pass('scalar_q4_execution', 'original_data_execution',
                                                 'original_data_diagnostics'),
        'q4_oracle_certification': _section_all_pass(
            'text_certifications', 'scalar_certifications', 'record_text_certifications',
            'collections_certifications', 'query_certifications', 'source_supplement_certifications',
            'original_data_certifications', 'source_cte_certifications',
            'text_q4_execution', 'record_text_execution', 'collections_execution',
            'query_instances_execution', 'source_supplement_execution', 'source_cte_execution'),
        'syntax_architecture': (result['gates']['architecture_required'] is True
                                and all(row['verdict'] == 'PASS' for row in result['first_slice_rows']
                                        if row['id'].startswith('syntax/'))),
        'precise_incremental_restore': (_section_all_pass('ct_restore_execution')
                                        and isinstance(result.get('incremental_restore_measurements'), dict)
                                        and result['incremental_restore_measurements']['gate']['measurements_complete']),
        'budgets_time': (isinstance(result.get('original_budget_contracts'), dict)
                         and isinstance(result.get('budget_time_measurements'), dict)
                         and result['budget_time_measurements']['gate']['budgets_time_complete']),
        'expanded_registry_valid': result['registered_obligations']['required_universe_closed'],
    }
    missing_inputs = [] if isinstance(result.get('budget_time_measurements'), dict) else ['budget_time_measurements']
    if not isinstance(result.get('incremental_restore_measurements'), dict):
        missing_inputs.append('incremental_restore_measurements')
    system_delivered = (isinstance(system_controls, dict)
                        and system_controls['gate']['controls_complete'] is True)
    if not system_delivered:
        missing_inputs.append('acceptance_system_controls(T-10 governance)')
    result['acceptance'] = reduce_acceptance(
        acceptance_gates,
        accounting_valid=result['legacy']['accounting_valid'] is True,
        evidence_complete=result['registered_obligations']['registered_observations_complete'],
        cert_gaps=len(result['legacy']['cert_gaps']),
        awaiting_d=result['registered_obligations']['required_universe']['awaiting_d'],
        system_delivered=system_delivered,
        missing_inputs=missing_inputs)
    result['evidence_complete'] = result['acceptance']['evidence_complete']
    result['acceptance_system_delivered'] = result['acceptance']['acceptance_system_delivered']
    result['can_claim_expanded_m1'] = result['acceptance']['can_claim_expanded_m1']
    result['blockers'] = [key for key, passed in result['gates'].items() if not passed]
    result['parent_bindings_sha256'] = bindings_sha
    result['admission_sources'] = {
        'scripts/test-governance/' + name + '.py': hashlib.sha256(
            inputs.read_bytes('scripts/test-governance/' + name + '.py')).hexdigest()
        for name in ('ace_m1_arch_acceptance', 'ace_m1_first_slice', 'ace_m1_case_evidence',
                     'ace_m1_legacy', 'ace_m1_legacy_arch', 'ace_m1_scalar_evidence', 'ace_m1_cert_evidence',
                     'ace_m1_syntax_evidence', 'ace_m1_syntax_arch', 'ace_m1_abi_evidence', 'ace_m1_obligation_catalog',
                     'ace_m1_legacy_base', 'ace_m1_legacy_data', 'ace_m1_legacy_execution', 'ace_m1_parent_contracts', 'ace_m1_av_typed_evidence',
                     'ace_m1_native_typed_evidence', 'ace_m1_legacy_obligations', 'ace_m1_budget_contracts',
                     'ace_m1_arch_obligations', 'ace_m1_ct_mechanisms', 'ace_m1_ct_child', 'ace_m1_ct_meta', 'ace_m1_ct_restore', 'ace_m1_sq_facts', 'ace_m1_registry', 'ace_m1_cte_evidence', 'ace_m1_text_evidence',
                     'ace_m1_data_diagnostics', 'ace_m1_original_data_sources', 'ace_m1_source_supplement', 'ace_m1_daily_evidence', 'ace_m1_text_q4_evidence', 'ace_m1_scalar_q4_evidence', 'ace_m1_scalar_cert_evidence', 'ace_m1_scalar_release_evidence', 'ace_m1_record_text_evidence', 'ace_m1_collections_evidence', 'ace_m1_query_evidence', 'ace_m1_incremental_restore', 'ace_m1_budget_time', 'ace_m1_system_controls')}
    result['admission_reserved_bytes'] = budget.reserved
    result['admission_peak_reserved_bytes'] = budget.peak_reserved
    return result
