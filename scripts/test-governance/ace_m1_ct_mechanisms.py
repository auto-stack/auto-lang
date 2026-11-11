"""Strict ordinary-source CT facts; no tool execution or restore inference."""
import hashlib
import importlib.util
import re
import tomllib
from ace_m1_arch_acceptance import EvidenceScope, counter, exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

ARCH = ('ARCH-04/cold-real-av-and-hit', 'ARCH-04/callee-body-change')
STAGES = ('cold_control', 'cold', 'hit', 'edit_control', 'edited')
COUNTERS = ('ct_execute', 'ct_hits', 'ct_freeze_publishes', 'ct_rejections')
LOG_FIELDS = ('key_hash', 'attempt', 'outcome_hex', 'steps', 'wall_ms', 'from_cache')


def validate_payload(document, source42, source43):
    exact_keys(document, ('schema', 'sink', 'source42_hex', 'source43_hex',
                         'cold_control_code_hex', 'edit_control_code_hex', 'owner_values', *STAGES),
               'ordinary source CT mechanism observation')
    typed_equal(document['schema'], 'ace.m1.ct-source-mechanisms/1', 'ordinary source CT observation schema')
    typed_equal(document['sink'], 'fresh-memory-envelope', 'CT sink scope; no managed restore claim')
    require(type(source42) is bytes and type(source43) is bytes, 'actual approved CT source bytes')
    typed_equal(document['source42_hex'], source42.hex(), 'original sum42 input bytes')
    typed_equal(document['source43_hex'], source43.hex(), 'original sum43 input bytes')
    require(source42.count(b'fn main') == source43.count(b'fn main') == 1, 'unique original CT owner')
    typed_equal(source42.split(b'fn main', 1)[1], source43.split(b'fn main', 1)[1], 'unchanged original call site')
    values = document['owner_values']
    require(type(values) is list and len(values) == 3 and all(type(v) is int for v in values),
            'three real checked-owner numeric observations')
    for stage in STAGES:
        snapshot = document[stage]
        exact_keys(snapshot, (*COUNTERS, 'exec_log'), 'original CT snapshot ' + stage)
        for name in COUNTERS: counter(snapshot[name], 'original CT ' + name)
        require(type(snapshot['exec_log']) is list and len(snapshot['exec_log']) <= 3, 'bounded actual CT execute log')
        for event in snapshot['exec_log']:
            exact_keys(event, LOG_FIELDS, 'original CT boundary event')
            hash_value(event['key_hash'], 'actual CT execution key')
            for name in ('attempt', 'steps', 'wall_ms'): counter(event[name], 'CT log ' + name)
            require(type(event['from_cache']) is bool, 'CT cache indicator must be boolean')
            require(event['outcome_hex'] in ('6f6b', '686974'), 'typed successful CT boundary outcome')
    def counts(stage, expected):
        return tuple(document[stage][name] for name in COUNTERS) == expected
    cold_control, cold, hit, edit_control, edited = (document[s] for s in STAGES)
    denied = 'cte.infra/av-forbidden'.encode().hex()
    cold_log, hit_log, edit_log = (d['exec_log'] for d in (cold, hit, edited))
    cold_ok = (document['cold_control_code_hex'] == denied
               and counts('cold_control', (0, 0, 0, 1)) and not cold_control['exec_log']
               and counts('cold', (1, 0, 1, 0)) and len(cold_log) == 1
               and cold_log[0]['attempt'] == 1 and cold_log[0]['outcome_hex'] == '6f6b'
               and cold_log[0]['from_cache'] is False and cold_log[0]['steps'] > 0
               and counts('hit', (1, 1, 1, 0)) and len(hit_log) == 2
               and hit_log[:1] == cold_log
               and hit_log[1]['attempt'] == 0 and hit_log[1]['outcome_hex'] == '686974'
               and hit_log[1]['from_cache'] is True
               and hit_log[1]['key_hash'] == cold_log[0]['key_hash']
               # A hit preserves the frozen execution's measured work fields.
               # They are not new work, and must not be read as another attempt.
               and hit_log[1]['steps'] == cold_log[0]['steps']
               and hit_log[1]['wall_ms'] == cold_log[0]['wall_ms']
               and values[:2] == [42, 42])
    edit_ok = (cold_ok and document['edit_control_code_hex'] == denied
               and counts('edit_control', (1, 1, 1, 1)) and edit_control['exec_log'] == hit_log
               and counts('edited', (2, 1, 2, 1)) and len(edit_log) == 3
               and edit_log[:2] == hit_log
               and edit_log[2]['attempt'] == 2 and edit_log[2]['from_cache'] is False
               and edit_log[2]['outcome_hex'] == '6f6b' and edit_log[2]['steps'] > 0
               and edit_log[2]['key_hash'] != cold_log[0]['key_hash'] and values[2] == 43)
    return {ARCH[0]: 'PASS' if cold_ok else 'FAIL', ARCH[1]: 'PASS' if edit_ok else 'FAIL'}


TEST = 'ordinary_source_cold_hit_and_callee_edit_observations_941'
BINARY = 'ace-source-cte::architecture_941'
EVENT = BINARY + '$' + TEST
CHILD_BINARY = 'ace-source-cte::architecture_child_941'
CHILD_TEST = 'ordinary_source_child_effective_versions_941'
META_BINARY = 'ace-source-cte::architecture_meta_941'
META_TEST = 'ordinary_source_meta_arguments_and_absence_941'
SOURCE_DIR = 'crates/auto-lang/test/cases/ace-m1/source-cte-936/src/'


def native_facts(inputs, scope, source_pins, validate_payload, *, binary_name=BINARY, test_name=TEST, group_name='ace-comptime'):
    from ace_m1_first_slice import _repo_relative
    producers = {(BINARY, TEST):'ace-comptime', (CHILD_BINARY, CHILD_TEST):'ace-comptime',
                 (META_BINARY, META_TEST):'ace-comptime',
                 ('ace-source-cte::architecture_restore_941', 'ordinary_source_disk_restore_first_edits_and_repair_941'):'ace-comptime',
                 ('ace-source-query::architecture_facts_941', 'ordinary_source_fact_authority_and_restore_identity_941'):'ace-semantics'}
    require(producers.get((binary_name,test_name)) == group_name, 'finite native producer and original group')
    event_name = binary_name + '$' + test_name
    for path, digest in source_pins.items():
        inputs.sha256(path, digest)
    before, before_sha = scope.read_json('source-pins-before.json')
    after, after_sha = scope.read_json('source-pins-after.json')
    exact_keys(before, ('commit', 'source_pins'), 'CT original before input snapshot')
    exact_keys(after, ('commit', 'source_pins'), 'CT original after input snapshot')
    typed_equal(before, after, 'CT original window source input stability')
    commit = before['commit']
    require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None, 'CT frozen producer commit')
    require(type(before['source_pins']) is dict and bool(before['source_pins']), 'actual CT before input set')
    require(before['source_pins'].keys() <= source_pins.keys(), 'CT before snapshot belongs to the declared producer closure')
    for path, digest in before['source_pins'].items():
        typed_equal(digest, source_pins[path], 'CT before input agrees with frozen producer declaration')
    binary_before, _ = scope.read_json('binary-before.json')
    binary_after, _ = scope.read_json('binary-after.json')
    exact_keys(binary_before, ('path', 'sha256'), 'original CT prepared binary')
    typed_equal(binary_before, binary_after, 'CT binary stable over actual gate')
    binary = _repo_relative(inputs, binary_before['path'])
    require(re.fullmatch('experimental/ace/target/debug/deps/' + re.escape(binary_name.split('::')[1]) + '-[0-9a-f]{16}\\.exe', binary) is not None,
            'CT original native test target path')
    inputs.sha256(binary, binary_before['sha256'])

    gate, gate_sha = scope.read_json('gate/results.json')
    typed_equal(gate['selected_groups'], [group_name], 'original complete native fixed group')
    typed_equal(gate['config_sha256'], source_pins['scripts/test-groups.toml'], 'original CT group configuration')
    require('error' not in gate, 'CT input changed or fixed driver failed during execution')
    # The native driver guards HEAD + code/config/input WIP for the entire
    # measurement window. Reader-only commits do not invalidate identical
    # declared producer bytes or this already terminal source-state guard.
    clean_state = hashlib.sha256(commit.encode() + b'[]').hexdigest()
    typed_equal(gate['source_state'], clean_state, 'clean frozen producer window')
    require(type(gate['results']) is list and len(gate['results']) == 1, 'one original native invocation')
    result = gate['results'][0]
    listing, listing_sha = scope.read_json('gate/0/discovery.stdout')
    suite = listing['rust-suites'][binary_name]
    typed_equal((suite['package-name'], suite['binary-name']), tuple(binary_name.split('::')), 'original CT native listing owner')
    typed_equal(_repo_relative(inputs, suite['binary-path']), binary, 'CT actually listed prepared binary')
    typed_equal(set(suite['testcases']), {test_name}, 'one CT observer test in its actual target')
    typed_equal(suite['testcases'][test_name], dict(kind='test', ignored=False, **{'filter-match':{'status':'matches'}}),
                'CT observer actually selected and non-ignored')

    # Reuse the fixed parent's pure selection and terminal parsers, not its
    # runner. A strict UTF-8/duplicate-key pass happens before its parser.
    spec = importlib.util.spec_from_file_location('ct_fixed_parent', inputs.path('scripts/fixed-tests.py'))
    parent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parent)
    config = tomllib.loads(inputs.read_bytes('scripts/test-groups.toml').decode('utf-8'))
    group = next(g for g in config['group'] if g['name'] == group_name)
    expression = '((' + group['expression'] + ')) and not test(=tests::aavm2_t3::test_aavm2_t3_tower_milestone)'
    expression = '(' + expression + ') and test(' + test_name + ')'
    arguments = [*group['args'], '--ignore-default-filter', '-E', expression,
                 '--target-dir', str(inputs.root/'experimental/ace/target')]
    selected, excluded = parent.enumerate_rust(listing, root=inputs.root)
    typed_equal(result['selected'], list(selected.values()), 'CT complete actual native selection')
    typed_equal(result['excluded'], excluded, 'CT original conditional exclusions')
    typed_equal(set(selected), {event_name}, 'exact named CT observer selection')
    raw = scope.read_bytes('gate/0/run.stdout')
    # Account the raw->Unicode->line copies and the parent's compact index.
    scope.reserve(6 * len(raw) + 512 * len(selected) + 4096)
    text = raw.decode('utf-8', errors='strict')
    for line in text.splitlines():
        if line.startswith('{'):
            scope.decode_json(line.encode('utf-8'))
    known = {s['package-name'] + '::' + s['binary-name'] + '$' + n
             for s in listing['rust-suites'].values() for n in s.get('testcases', {})}
    actual = parent.terminal_results(raw, selected, known - selected.keys())
    typed_equal(result['results'], actual['results'], 'CT actual native terminal rows cannot be replaced by a summary')
    typed_equal(result['executed'], len(actual['results']), 'CT native executed count')
    typed_equal(result['not_run'], actual['not_run'], 'CT missing test terminal status')
    typed_equal(result['errors'], actual['errors'], 'CT parent errors must preserve original terminal failures')
    require(not actual['not_run'], 'CT complete group did not finish every selected native test')
    command, _ = scope.read_json('gate/0/run.command.json')
    typed_equal(result['execution'], command, 'CT formal native command receipt')
    require(command['meter']['error'] is None and command['meter']['exit'] == command['exit'], 'CT actual native process terminal')
    typed_equal(command['stdout'], 'run.stdout', 'CT original raw stdout reference')
    typed_equal(command['stderr'], 'run.stderr', 'CT original stderr reference')
    typed_equal(command['argv'], ['cargo', 'nextest', 'run', *arguments,
                '--no-fail-fast', '--no-tests', 'fail', '--message-format', 'libtest-json-plus',
                '--message-format-version', '0.1'], 'CT original complete native runner invocation')
    typed_equal(command['cwd'], str(inputs.root / 'experimental/ace'), 'CT actual producer working directory')
    stderr = scope.read_bytes('gate/0/run.stderr')
    typed_equal(gate['gate_passed'], result['outcome'] == 'PASS', 'CT parent gate status')
    typed_equal(result['outcome'], 'PASS' if command['exit'] == 0 and not actual['errors'] else 'FAIL', 'CT real native parent outcome')

    require(scope.path('observations.json').stat().st_size <= 16 << 10, 'bounded CT observation payload')
    document, raw_sha = scope.read_json('observations.json')
    verdicts = validate_payload(document, inputs.read_bytes(SOURCE_DIR+'sum42.at'), inputs.read_bytes(SOURCE_DIR+'sum43.at'))
    terminal = actual['results'][event_name]
    require(terminal['event'] in ('ok', 'failed'), 'CT observer is not an actual completed native execution')
    if terminal['event'] != 'ok':
        verdicts = {key:'FAIL' for key in verdicts}
    return dict(schema='ace.m1.ct-source-mechanism-admission/1', observations=verdicts,
                source_evidence=dict(producer_commit=commit, current_binary_sha256=binary_before['sha256'],
                original_before_sha256=before_sha, original_after_sha256=after_sha,
                gate_sha256=gate_sha, native_listing_sha256=listing_sha, raw_observation_sha256=raw_sha,
                native_stdout_sha256=hashlib.sha256(raw).hexdigest(), native_test=event_name,
                native_stderr_sha256=hashlib.sha256(stderr).hexdigest(),
                parent_status=terminal['event'], observer_gate=gate['gate_passed'],
                selected_native_executed=result['executed']),
                managed_restore='UNOBSERVED', meta_instances='UNOBSERVED', other_architecture='UNOBSERVED')


CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/ct-source-mechanisms-contract.json'
CONTRACT_SHA = 'e832c4274e836c2e4f48c753682cfcc8cbb69d05bdbf93994d554ff9b59aab29'


def admit(inputs, evidence, binding):
    exact_keys(binding, ('root',), 'ordinary source CT mechanism bindings')
    inputs.sha256(CONTRACT, CONTRACT_SHA)
    declaration, _ = inputs.read_json(CONTRACT)
    typed_equal(declaration['schema'], 'ace.m1.ct-source-mechanism-contract/1', 'CT mechanism declaration schema')
    typed_equal(declaration['mechanisms'], list(ARCH), 'finite CT architecture obligation slice')
    typed_equal(declaration['execution'], 'UNOBSERVED', 'CT declaration is not execution')
    scope = EvidenceScope(evidence.path(binding['root'] + '/observations.json').parent, inputs.budget)
    result = native_facts(inputs, scope, declaration['source_pins'], validate_payload)
    from ace_m1_arch_obligations import reduce_architecture
    result['gate'] = reduce_architecture(result['observations'])
    result['can_claim_architecture_pass'] = False
    return result
