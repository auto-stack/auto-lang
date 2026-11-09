"""Independent text reference contracts and original two-leg certification."""
from pathlib import PurePosixPath

from ace_m1_arch_acceptance import exact_keys
from ace_m1_case_evidence import require
from ace_m1_cert_evidence import text_certification
from ace_m1_legacy import typed_equal
from ace_m1_obligation_catalog import BASE, COMPOSITE_SOURCES, read_tsv

CONTRACT = BASE + 'text-reference-contracts.tsv'
CONTRACT_SHA = 'dcd2744a0dc827e555dff7474a5c335425d393b813d6862584b453886cf04752'
FIELDS = 'case source_file source_sha256 profile caps oracle_kind oracle_value c_reference c_reference_sha256 rust_reference rust_reference_sha256 execution'.split()
ORACLES = {'v03_len_scalars_bytes':('value', '506'), 'v03_nul_data':('value', '33'),
           'v03_eq_full_bytes':('value', '1'), 'v03_char_at':('value', '233'),
           'v03_char_at_negative':('trap', 'TextIndex'), 'v03_slice_oob':('trap', 'TextIndex'),
           'v03_slice_scalars':('value', '3'), 'v03_codepoint_ok':('value', '20013'),
           'v03_codepoint_surrogate':('trap', 'InvalidScalar'), 'v03_concat_clone':('value', '112')}


def text_contracts(inputs):
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    for path, digest in COMPOSITE_SOURCES.items():
        inputs.sha256(path, digest)
    rows = read_tsv(inputs, CONTRACT, FIELDS)
    seen = {}
    for row in rows:
        case = row['case']
        require(case in ORACLES and case not in seen, 'foreign/duplicate independent text reference')
        typed_equal((row['oracle_kind'], row['oracle_value']), ORACLES[case], 'independent text oracle')
        typed_equal(row['source_file'], BASE + 'composite-src/text-char-925--' + case + '.at', 'original text source')
        typed_equal(row['profile'], 'text', 'original text profile')
        typed_equal(row['caps'], '[]', 'original text capabilities')
        typed_equal(row['execution'], 'UNOBSERVED', 'reference registration is not execution')
        inputs.sha256(row['source_file'], row['source_sha256'])
        for target, extension in (('rust', 'rs'), ('c', 'c')):
            typed_equal(row[target + '_reference'], BASE + 'text-reference/' + case + '.' + extension,
                        'independent text reference path')
            inputs.sha256(row[target + '_reference'], row[target + '_reference_sha256'])
        seen[case] = row
    typed_equal(set(seen), set(ORACLES), 'complete independent text reference set')
    return seen


def admit_text_certifications(inputs, evidence, binding_path):
    from ace_m1_first_slice import _compact_admission, _repo_relative
    from ace_m1_daily_evidence import certified_daily_emission
    contracts = text_contracts(inputs)
    document, digest = evidence.read_json(binding_path)
    exact_keys(document, ('producer', 'rows'), 'original text certification bindings')
    producer = document['producer']
    exact_keys(producer, ('producer_commit', 'worker_path', 'worker_sha256', 'cwd'), 'text cert producer')
    typed_equal(_repo_relative(inputs, producer['worker_path']),
                'experimental/ace/target/release/a2x-worker.exe', 'original text certification worker')
    typed_equal(producer['cwd'], str(inputs.root), 'original text certification working directory')
    require(type(document['rows']) is list, 'text certification rows required')
    seen, observations = set(), {}
    for row in document['rows']:
        exact_keys(row, ('case', 'prefix', 'pin'), 'text certification row')
        case, pin = row['case'], row['pin']
        require(case in contracts and type(pin) is dict, 'foreign text certification case')
        exact_keys(pin, ('target', 'identity_request', 'request', 'toolchain', 'source_sha256',
                         'reference_sha256', 'actual_files', 'expected_stdout_hex', 'expected_stderr_hex'), 'text certification pins')
        target = pin['target']
        require(target in ('rust', 'c') and (case, target) not in seen, 'foreign/duplicate text certification target')
        seen.add((case, target))
        typed_equal(pin['request']['case_id'], '941/text-char/' + case, 'independent text case identity')
        typed_equal(row['prefix'], str(PurePosixPath(binding_path).parent / case / target), 'text certification original output')
        typed_equal(pin['identity_request'], {'op':'identity', 'certification_target':target}, 'text toolchain target')
        observation = _compact_admission(inputs, lambda _: certified_daily_emission(
            evidence, inputs, row['prefix'], pin, producer, text_certification(
                evidence, inputs, row['prefix'], pin, producer, contracts[case])), 128 << 10)
        key = 'q4-source/text-char-925/' + case + '/' + ('ACE-A2R-HIR' if target == 'rust' else 'ACE-A2C-HIR')
        observations[key] = observation
    typed_equal(seen, {(case, target) for case in ORACLES for target in ('rust', 'c')}, 'complete two-leg text certification set')
    return {'schema':'ace.m1.text-certification-observations/1', 'contract_sha256':CONTRACT_SHA,
            'bindings_sha256':digest, 'required':20, 'observations':observations,
            'certification_complete':all(row['judgement'] == 'PASS' for row in observations.values()),
            'daily_admission':'PASS' if all(value['daily_judgement'] == 'PASS' for value in observations.values()) else 'FAIL',
            'av_native_evidence':'UNOBSERVED',
            'can_claim_text_q4_complete':False}
