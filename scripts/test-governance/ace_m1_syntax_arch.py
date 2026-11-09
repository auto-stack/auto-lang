"""Join the ten original admitted syntax facts; no parser or runner here."""
from ace_m1_arch_acceptance import counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

ARCH = 'ARCH-01/all-grammars'
CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/syntax-architecture-contract.json'
CONTRACT_SHA = '72a5b690432cd9b8da5eb3c5666924c363798b08c656ab53873585170a33409d'
SLUGS = ('core','scalar','text','record','text-record','entry-unit','host-console','closures','collections')
CHECKS = tuple('parse-'+slug for slug in SLUGS) + ('parse-record-under-scalar-grammar-refused',)


def evaluate(observations, rows, worker):
    """Only accepts facts from syntax_parse_observation/2 after original admission."""
    selected = {k:v for k,v in observations.items() if k.startswith('syntax/')}
    typed_equal(set(selected), {'syntax/'+name for name in CHECKS}, 'all ten original syntax checks, no subset')
    require(type(rows) is list and len(rows) == 10, 'finite original syntax contract rows')
    hash_value(worker, 'current actual syntax worker')
    ok = True
    for row, name in zip(rows, CHECKS):
        exact_keys(row, ('check','case','source','source_sha256','source_bytes','grammar','request_profile','completion'), 'original syntax architecture row')
        typed_equal(row['check'], name, 'frozen syntax check order')
        value = selected['syntax/'+name]
        exact_keys(value, ('schema','source_sha256','response_sha256','grammar','case_id','check_id','request_profile','entry','capabilities','worker_sha256','primary_diagnostic','reconstructed_sha256','diagnostics','artifact_oracles','metric_gaps','counters','judgement','scope','parent_status'), 'admitted original syntax observation')
        typed_equal(value['schema'], 'ace.m1.syntax-parse-observation/2', 'admitted syntax identity schema')
        for field in ('source_sha256','response_sha256','worker_sha256','reconstructed_sha256'): hash_value(value[field], 'syntax '+field)
        for field in ('diagnostics','artifact_oracles'): counter(value[field], 'actual syntax '+field)
        require(value['judgement'] in ('PASS','FAIL'), 'actual syntax verdict')
        require(value['parent_status'] in ('PASS','FAIL','SKIP','ERROR'), 'actual syntax parent status')
        require(type(value['capabilities']) is list and all(type(c) is str for c in value['capabilities']), 'syntax actual capabilities')
        ok &= (value['case_id'] == row['case'] and value['check_id'] == row['check']
               and value['source_sha256'] == row['source_sha256'] and value['grammar'] == row['grammar']
               and value['request_profile'] == row['request_profile'] and value['entry'] == 'main'
               and value['capabilities'] == [] and value['worker_sha256'] == worker
               and value['judgement'] == value['parent_status'] == 'PASS')
        primary = value['primary_diagnostic']
        if row['completion'] == 'success':
            ok &= (value['diagnostics'] == 0 and primary is None and value['artifact_oracles'] == 1
                   and value['reconstructed_sha256'] == row['source_sha256'])
        else:
            exact_keys(row['completion'], ('stage','code'), 'original negative syntax oracle')
            if primary is None:
                ok = False
            else:
                exact_keys(primary, ('stage','code','span_start','span_end'), 'actual primary syntax refusal')
                a = counter(primary['span_start'], 'syntax refusal start')
                b = counter(primary['span_end'], 'syntax refusal end')
                require(a <= b <= row['source_bytes'], 'syntax primary span within original source')
                ok &= (value['diagnostics'] > 0 and value['artifact_oracles'] == 0
                       and primary['stage'] == row['completion']['stage'] and primary['code'] == row['completion']['code'])
    # The negative uses the exact record bytes accepted by the record grammar.
    # It is the grammar, not a weakened or unrelated source, that changes.
    ok &= (rows[-1]['source_sha256'] == rows[3]['source_sha256']
           and rows[-1]['grammar'] == 'auto-source@scalar' and rows[3]['grammar'] == 'auto-source@record')
    return {ARCH: 'PASS' if ok else 'FAIL'}


def admit(inputs, observations, parent, binding, worker):
    exact_keys(binding, ('source',), 'syntax architecture binding')
    typed_equal(binding['source'], 'syntax', 'already admitted original syntax source')
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    declaration,_ = inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'], 'ace.m1.syntax-architecture-contract/1', 'syntax architecture declaration')
    typed_equal(declaration['mechanisms'], [ARCH], 'one original grammar obligation')
    typed_equal(declaration['execution'], 'UNOBSERVED', 'declaration is not execution')
    for path,digest in declaration['source_pins'].items(): inputs.sha256(path,digest)
    for row in declaration['checks']:
        source = inputs.read_bytes(row['source'])
        typed_equal(len(source),row['source_bytes'],'original syntax source length')
        inputs.sha256(row['source'],row['source_sha256'])
    verdicts = evaluate(observations,declaration['checks'],worker)
    exact_keys(parent, ('report_sha256','non_pass_rows'), 'original whole syntax parent report')
    hash_value(parent['report_sha256'], 'original syntax case report')
    return dict(schema='ace.m1.syntax-architecture-admission/1', observations=verdicts,
                source_evidence=dict(contract_sha256=CONTRACT_SHA, original_case_report_sha256=parent['report_sha256'],
                    current_worker_sha256=worker, original_checks=10,
                    responses={k:v['response_sha256'] for k,v in observations.items() if k.startswith('syntax/')}),
                other_architecture='UNOBSERVED; parse is not incremental, semantic typing or restore evidence',
                can_claim_architecture_pass=False)
