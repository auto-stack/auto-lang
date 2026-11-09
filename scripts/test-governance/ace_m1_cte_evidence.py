"""Read original CT source value observations; do not infer CT cache mechanics."""
from ace_m1_arch_acceptance import EvidenceScope, exact_keys
from ace_m1_case_evidence import CaseCopy, admit_case_bindings, require
from ace_m1_legacy import typed_equal
from ace_m1_scalar_evidence import scalar_observation
from pathlib import PurePosixPath

REFERENCE_CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/cte-reference-contracts.tsv'
REFERENCE_SHA = '4fa162f7fe6722f540fb9c90a45fd89ed13b7a938d2ebc852c303ed8733dda69'
REFERENCE_FIELDS = 'case source_file source_sha256 profile caps oracle_kind oracle_value c_reference c_reference_sha256 rust_reference rust_reference_sha256 execution'.split()

# Hand-derived original 936 Case oracles, including Bool demand -> main exit 7.
CT_CONTRACTS = {
    'sum42':('sum42', 42, []), 'callee-body43':('sum43', 43, []),
    'bool':('boollit', 7, []),
    'meta-fields2':('meta2', 42, ['cte.meta.fields/v1']),
    'meta-fields3':('meta3', 43, ['cte.meta.fields/v1']),
    'choose-instance42':('choose42', 42, ['ac.source.instances.scalar/v1']),
    'child-ct-chain40':('pick40', 40, ['ac.source.instances.scalar/v1'])}


def cte_reference_contracts(inputs):
    from ace_m1_obligation_catalog import BASE, read_tsv
    inputs.sha256(REFERENCE_CONTRACT, REFERENCE_SHA)
    rows = read_tsv(inputs, REFERENCE_CONTRACT, REFERENCE_FIELDS)
    contracts = {}
    for row in rows:
        case = row['case']
        require(case in CT_CONTRACTS and case not in contracts, 'foreign/duplicate CT reference case')
        name, value, extra = CT_CONTRACTS[case]
        typed_equal(row['profile'], 'scalar', 'original CT profile')
        caps, _ = inputs.decode_json(row['caps'].encode())
        typed_equal(caps, ['ac.source.cte.scalar/v1'] + extra, 'original CT capabilities')
        typed_equal((row['oracle_kind'], row['oracle_value']), ('value', str(value)), 'independent CT oracle')
        typed_equal(row['source_file'], 'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/' + name + '.at', 'original CT source')
        typed_equal(row['execution'], 'UNOBSERVED', 'CT reference registration is not execution')
        inputs.sha256(row['source_file'], row['source_sha256'])
        for target, extension in (('rust', 'rs'), ('c', 'c')):
            typed_equal(row[target + '_reference'], BASE + 'cte-reference/' + case + '.' + extension,
                        'independent CT reference path')
            inputs.sha256(row[target + '_reference'], row[target + '_reference_sha256'])
        contracts[case] = row
    typed_equal(set(contracts), set(CT_CONTRACTS), 'complete seven-source CT reference set')
    return contracts


def admit_cte_certifications(inputs, evidence, binding_path):
    from ace_m1_first_slice import _compact_admission, _repo_relative
    from ace_m1_daily_evidence import certified_daily_emission
    from ace_m1_cert_evidence import cte_certification
    contracts = cte_reference_contracts(inputs)
    document, digest = evidence.read_json(binding_path)
    exact_keys(document, ('producer', 'rows'), 'original CT certification bindings')
    producer = document['producer']
    exact_keys(producer, ('producer_commit', 'worker_path', 'worker_sha256', 'cwd'), 'CT certification producer')
    typed_equal(_repo_relative(inputs, producer['worker_path']), 'experimental/ace/target/release/a2x-worker.exe',
                'current CT certification worker')
    typed_equal(producer['cwd'], str(inputs.root), 'current CT producer working directory')
    require(type(document['rows']) is list, 'CT certification rows required')
    seen, observations = set(), {}
    for row in document['rows']:
        exact_keys(row, ('case', 'prefix', 'pin'), 'CT certification row')
        case, pin = row['case'], row['pin']
        require(case in contracts and type(pin) is dict, 'foreign CT certification case')
        exact_keys(pin, ('target', 'identity_request', 'request', 'toolchain', 'source_sha256',
                         'reference_sha256', 'actual_files', 'expected_stdout_hex', 'expected_stderr_hex'), 'CT certification pins')
        target = pin['target']
        require(target in ('rust', 'c') and (case, target) not in seen, 'foreign/duplicate CT certification target')
        seen.add((case, target))
        typed_equal(pin['request']['case_id'], '941/source-cte/' + case, 'independent CT case identity')
        typed_equal(row['prefix'], str(PurePosixPath(binding_path).parent / case / target), 'CT original certification output')
        typed_equal(pin['identity_request'], {'op':'identity', 'certification_target':target}, 'CT toolchain target')
        value = _compact_admission(inputs, lambda _: certified_daily_emission(
            evidence, inputs, row['prefix'], pin, producer, cte_certification(
                evidence, inputs, row['prefix'], pin, producer, contracts[case])), 128 << 10)
        key = 'q4-source/source-cte/' + case + '/' + ('ACE-A2R-HIR' if target == 'rust' else 'ACE-A2C-HIR')
        observations[key] = value
    typed_equal(seen, {(case, target) for case in CT_CONTRACTS for target in ('rust', 'c')}, 'complete CT certification set')
    return {'schema':'ace.m1.cte-certification-observations/1', 'required':14,
            'contract_sha256':REFERENCE_SHA, 'bindings_sha256':digest, 'observations':observations,
            'certification_complete':all(value['judgement'] == 'PASS' for value in observations.values()),
            'daily_admission':'PASS' if all(value['daily_judgement'] == 'PASS' for value in observations.values()) else 'FAIL',
            'ct_execution_hit_restore_mechanisms':'UNOBSERVED',
            'can_claim_ct_architecture':False}


def admit_cte_source_slice(inputs, evidence, binding, frozen, workers):
    from ace_m1_first_slice import _inventory, _repo_relative, _wires, retain_parent_status
    exact_keys(binding, ('inventory', 'copy'), 'CT source original execution binding')
    inventory = _inventory(inputs, evidence, binding['inventory'], 'source_cte', frozen)
    copy = CaseCopy(EvidenceScope(evidence.path(binding['copy'] + '/artifact-copy.json').parent, inputs.budget))
    admitted = admit_case_bindings(copy, inventory)
    by_case = {'ace.m1.cte936.' + name:(key, value, extra)
               for key, (name, value, extra) in CT_CONTRACTS.items()}
    seen, observations, rejected = set(), {}, []
    for check, parent in admitted['checks']:
        require(check['case'] in by_case and check['check'] in ('av', 'ac-native'), 'foreign CT source execution')
        key, value, extra = by_case[check['case']]
        target = 'AV' if check['check'] == 'av' else 'AC-native'
        slot = 'q4-source/source-cte/' + key + '/' + target
        require(slot not in seen, 'duplicate CT source execution')
        seen.add(slot)
        typed_equal(check['expected_exit'], value, 'original CT independent oracle')
        typed_equal(check['completion'], {'kind':'success'}, 'CT source success oracle')
        av = target == 'AV'
        declared = check['probe_bindings']['av' if av else 'native']
        typed_equal(declared['entry'], 'd_main', 'CT original entry')
        typed_equal(declared['caps'], ['ac.source.cte.scalar/v1'] + extra, 'CT original capabilities')
        source = _repo_relative(inputs, check['input_paths'][0])
        typed_equal(source, 'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/'
                    + CT_CONTRACTS[key][0] + '.at', 'CT original source')
        wires = _wires(copy, check, 'avworker/1' if av else 'mnworker/1', check['recipe'], workers['av' if av else 'native'])
        require(len(wires) == 1, 'one original CT scalar run required')
        observations[slot] = retain_parent_status(scalar_observation(copy.scope, wires[0], check), parent['status'])
        if parent['status'] != 'PASS':
            rejected.append({'case':check['case'], 'check':check['check'], 'status':parent['status']})
    required = {'q4-source/source-cte/' + key + '/' + target for key in CT_CONTRACTS for target in ('AV', 'AC-native')}
    typed_equal(seen, required, 'all original CT source checks required')
    return {'schema':'ace.m1.cte-source-value-observations/1', 'required':14,
            'observations':observations, 'value_evidence_complete':all(
                row['judgement'] == 'PASS' for row in observations.values()),
            'source_case_report':{'report_sha256':admitted['report_sha256'], 'non_pass_rows':rejected},
            'ct_execution_hit_restore_mechanisms':'UNOBSERVED',
            'certification_targets':'UNOBSERVED', 'can_claim_ct_architecture':False}
