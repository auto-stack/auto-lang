"""Original policy declarations, kept separate from observed budget compliance."""
from ace_m1_arch_acceptance import counter, exact_keys
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/original-budget-contracts.json'
CONTRACT_SHA = '6c47f857f3b3838ca2e087102ea89f6f250b88ff351528114c67bb02abe6bdac'
POLICIES = ('fixed-case-suite', 'nextest-default', 'ac-v2-check', 'legacy-inprocess',
            'a2x-daily-parent', 'abi-interop-runtime', 'cte-original-service',
            'a2x-warm-management', 'explicit-cert-941-capture', 'composer',
            'av-worker', 'native-worker', 'abi-worker-ipc', 'a2x-scalar-formal',
            'a2x-record-text-formal', 'a2x-collections-formal')


def _limits(value, path):
    require(type(value) is dict and bool(value), 'budget limit map required')
    for name, limit in value.items():
        require(type(name) is str and bool(name), 'budget limit name required')
        if type(limit) is dict:
            _limits(limit, path + '/' + name)
        elif limit is None:
            require('override' in path or 'override' in name
                    or name in ('inner_deadline_ms', 'inner_stream_cap_bytes'), 'unknown absence of budget')
        else:
            require(counter(limit, path + '/' + name) > 0, 'zero budget declaration')


def validate_contract(inputs, document, parents):
    exact_keys(document, ('schema', 'policies', 'source_pins', 'execution', 'executed', 'can_claim_budget_pass'), 'budget contracts')
    typed_equal(document['schema'], 'ace.m1.original-budget-contracts/1', 'budget schema')
    typed_equal(document['execution'], 'UNOBSERVED', 'declarations are not execution')
    typed_equal(document['executed'], 0, 'budget discovery execution')
    typed_equal(document['can_claim_budget_pass'], False, 'budget discovery cannot claim compliance')
    exact_keys(document['policies'], POLICIES, 'original policy domains')
    source_names = set()
    summaries = {}
    for name, policy in document['policies'].items():
        exact_keys(policy, ('scope', 'owner', 'limits', 'sources', 'runtime_verdict'), 'budget policy')
        require(type(policy['scope']) is str and bool(policy['scope']) and type(policy['owner']) is str and bool(policy['owner']), 'budget scope/owner required')
        typed_equal(policy['runtime_verdict'], 'UNOBSERVED', 'declaration cannot supply runtime observations')
        _limits(policy['limits'], name)
        require(type(policy['sources']) is list and bool(policy['sources'])
                and len(set(policy['sources'])) == len(policy['sources']), 'budget policy source list')
        source_names.update(policy['sources'])
        summaries[name] = {key:policy[key] for key in ('scope', 'limits', 'runtime_verdict')}
    exact_keys(document['source_pins'], source_names, 'complete original budget source closure')
    for name, digest in document['source_pins'].items():
        inputs.sha256(name, digest)
    for policy, parent in (('av-worker', 'av'), ('native-worker', 'native'), ('abi-worker-ipc', 'abi')):
        typed_equal(document['policies'][policy]['limits'], parents['snapshot']['parents'][parent]['limits'], 'separate original parent policy')
    typed_equal(document['policies']['abi-interop-runtime']['limits'], {
        'build_ms':60000, 'run_ms':5000, 'total_ms':70000, 'stdout_bytes':1 << 20,
        'stderr_bytes':1 << 20, 'ipc_bytes':4 << 20, 'single_artifact_bytes':64 << 20,
        'request_artifact_bytes':128 << 20, 'hot_bytes':8 << 20, 'staging_and_index_bytes':8 << 20}, 'public ABI original caps')
    typed_equal(document['policies']['composer']['limits'], {
        'document_bytes':4 << 20, 'live_bytes':48 << 20,
        'parent_private_peak_increment_bytes':128 << 20, 'hash_block_bytes':64 << 10}, 'composer own caps')
    return {'schema':'ace.m1.budget-discovery/1', 'policies':summaries, 'source_sha256':document['source_pins'],
            'execution':'UNOBSERVED', 'executed':0, 'can_claim_budget_pass':False}


def admit_budget_contracts(inputs, parents):
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    document, digest = inputs.read_json(CONTRACT)
    return validate_contract(inputs, document, parents) | {'contract_sha256':digest}
