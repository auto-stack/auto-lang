"""Explicit old required-cell locators; discovery never supplies execution."""
from collections import Counter
import tomllib

from ace_m1_arch_acceptance import exact_keys, hash_value
from ace_m1_case_evidence import require
from ace_m1_legacy_base import VALUE_CASES as BASE_VALUE_CASES,TRAP_CASES as BASE_TRAPS,DIAGNOSTICS as BASE_DIAGS,VM_VALUE_CASES as BASE_VM_VALUES,VM_FAILURES as BASE_VM_FAILURES
from ace_m1_legacy_data import CASES as DATA_CASES
from ace_m1_legacy import CORPUS, INPUTS, REQUIRED_SHA256, frozen_ledger, typed_equal

MAP = 'crates/auto-lang/test/cases/ace-m1-expanded/legacy-required-map.json'
MAP_SHA = '25df3c1ba7b1f9d716eb988ee79c611be0b8301037a16ba07fade25f469a8aa9'
SCENARIOS = 'docs/specs/auto-test-case/ace-m1-scenario-oracles.md'
SCENARIOS_SHA = 'cb1796d84f23a3dcd8514583e9b29bd17b6ed740140eb6338f593233bce558b4'
TARGETS = {'ac':'AC-native', 'av':'AV', 'avm':'legacy-AVM', 'a2r':'legacy-A2R'}
SAMPLES = {'I01':{'body_revisions':30}, 'I10':{'distinct_revisions':512},
           'I11':{'min_samples':10, 'gate_percentile':90, 'gate_ms':2000, 'min_passing_samples':9,
                  'historical_diagnostic_percentile':95, 'historical_diagnostic_ms':1000}}
ENTRY_FIELDS = ('group', 'suite', 'case', 'check', 'suite_file', 'case_file', 'case_file_sha256',
                'adapter', 'recipe', 'action', 'oracle_sha256', 'contract_sha256', 'check_sha256',
                'source_files', 'inputs', 'expected_exit', 'completion', 'typed_oracle', 'state', 'steps')
CELL_FIELDS = ('cell', 'layer', 'token', 'target', 'binding_oracle', 'input_kind', 'link_basis',
               'frozen_source', 'entries', 'gaps', 'evidence_owner', 'capability_owner',
               'reader_status', 'budget_domain', 'execution')


def target_matches(target, entry):
    if target in ('avm', 'a2r'):
        return entry['adapter'] == ('avm' if target == 'avm' else 'a2r')
    recipe = entry['recipe']
    return entry['adapter'] == 'probe' and type(recipe) is str and recipe.startswith(
        'ac-av/' if target == 'av' else 'ac/mir-native')


def validate_map(inputs, data):
    exact_keys(data, ('schema', 'required_set_sha256', 'source_pins', 'entry_semantics', 'original_samples',
                      'budget_contracts', 'entries', 'cells', 'execution', 'mapping_complete'), 'old obligation map')
    typed_equal(data['schema'], 'ace.m1.legacy-obligation-map/1', 'old map schema')
    typed_equal(data['required_set_sha256'], REQUIRED_SHA256, 'approved old set identity')
    typed_equal(data['source_pins'], dict(INPUTS) | {SCENARIOS:SCENARIOS_SHA}, 'old map approved inputs')
    for name, digest in data['source_pins'].items():
        inputs.sha256(name, digest)
    typed_equal(data['original_samples'], SAMPLES, 'original scenario scale/time gates')
    typed_equal(data['execution'], 'UNOBSERVED', 'map is not execution')
    typed_equal(data['mapping_complete'], False, 'map does not close remaining integration')
    require(type(data['entry_semantics']) is str and bool(data['entry_semantics']), 'entry scope required')
    exact_keys(data['budget_contracts'], ('av', 'native', 'legacy-in-process'), 'old budget domains')
    for domain in ('av', 'native'):
        typed_equal(data['budget_contracts'][domain], {
            'source':'registry.json#parent_contracts/snapshot/parents/' + domain,
            'status':'original-declared'}, 'original parent budget reference')
    typed_equal(data['budget_contracts']['legacy-in-process'], {
        'source':'original-budget-contracts.json#policies/legacy-inprocess', 'status':'original-declared', 'owner':'PLAN-941'},
        'legacy outer/inner policy identity must remain explicit')
    ledger = {':'.join((row['ns'], row['id'], row['path'])):row
              for row in frozen_ledger(inputs) if row['required']}
    old_rows = [line.split('\t') for line in inputs.read_bytes(CORPUS + 'binding.tsv').decode().splitlines()
                if line and not line.startswith('#')]
    old = {(row[1], row[0]):row for row in old_rows}
    manifest, _ = inputs.read_json(CORPUS + 'source-manifest.json')
    groups = {row['name']:row for row in tomllib.loads(inputs.read_bytes('scripts/test-groups.toml').decode())['group']}
    require(type(data['entries']) is dict and type(data['cells']) is list, 'old map entries/cells required')
    verified = {}

    def current(path, digest):
        if path in verified:
            typed_equal(digest, verified[path], 'shared current source identity')
        else:
            inputs.sha256(path, digest)
            verified[path] = digest

    for key, entry in data['entries'].items():
        exact_keys(entry, ENTRY_FIELDS, 'formal entry locator')
        typed_equal(key, '/'.join((entry['suite'], entry['case'], entry['check'])), 'formal entry key')
        require(entry['group'] in groups and groups[entry['group']]['runner'] == 'case', 'unknown formal Case group')
        typed_equal(groups[entry['group']]['suite'], entry['suite_file'], 'formal group suite')
        require(entry['state'] in ('RUN', 'SKIP') and entry['action'] in ('run', 'emit'), 'unknown formal entry state/action')
        for field in ('case_file_sha256', 'oracle_sha256', 'contract_sha256', 'check_sha256'):
            hash_value(entry[field], 'entry ' + field)
        current(entry['case_file'], entry['case_file_sha256'])
        require(type(entry['inputs']) is list and type(entry['source_files']) is list
                and bool(entry['source_files']), 'explicit original source/sidecars required')
        names = set()
        for ref in entry['inputs']:
            exact_keys(ref, ('path', 'sha256'), 'explicit formal input')
            require(ref['path'] not in names, 'duplicate entry input')
            names.add(ref['path'])
            current(ref['path'], ref['sha256'])
        source_names = set()
        for source in entry['source_files']:
            exact_keys(source, ('virtual_path', 'sha256'), 'original source file')
            require(source['virtual_path'] not in source_names, 'duplicate source virtual path')
            source_names.add(source['virtual_path'])
            hash_value(source['sha256'], 'original source bytes')
            require(source['sha256'] in {ref['sha256'] for ref in entry['inputs']}, 'source bytes missing from physical inputs')
    seen, used, unbound, gaps = set(), set(), [], Counter()
    for cell in data['cells']:
        exact_keys(cell, CELL_FIELDS, 'old required cell')
        key = cell['cell']
        require(key in ledger and key not in seen, 'foreign/duplicate old required cell')
        seen.add(key)
        original = ledger[key]
        ns, name, target = key.split(':')
        row = old[(ns, name)]
        for field in ('layer', 'token'):
            typed_equal(cell[field], original[field], 'original cell ' + field)
        typed_equal(cell['target'], TARGETS[target], 'separate legacy/current target identity')
        typed_equal(cell['binding_oracle'], {'kind':row[3], 'expected':row[4]}, 'original binding oracle')
        typed_equal(cell['evidence_owner'], 'PLAN-941', 'evidence owner')
        typed_equal(cell['execution'], 'UNOBSERVED', 'locator cannot claim execution')
        require(type(cell['capability_owner']) is str and bool(cell['capability_owner']), 'capability owner missing')
        supported = ns == 'corpus' and ((target in ('ac','av') and name in DATA_CASES + BASE_VALUE_CASES + ('B08','B11','W01','W02','W03','W04','W05')) or (target=='av' and name in BASE_TRAPS+tuple(BASE_DIAGS)) or (target=='ac' and name in ('B17','B18','B19')) or (target=='avm' and name in BASE_VM_VALUES+tuple(BASE_VM_FAILURES)))
        typed_equal(cell['reader_status'], 'supported-slice' if supported else 'pending', 'reader coverage boundary')
        wanted_gaps = [] if supported else ['raw-evidence-adapter-and-full-oracle']
        require(type(cell['entries']) is list and len(set(cell['entries'])) == len(cell['entries']), 'duplicate entry links')
        if not cell['entries']:
            wanted_gaps.append('formal-entry-not-yet-bound')
            unbound.append(key)
        if ns == 'corpus' and name in DATA_CASES and target in ('ac', 'av'):
            from ace_m1_legacy_data import DIAGNOSTICS as DATA_DIAGNOSTICS
            family = 'original-data-diag' if name in DATA_DIAGNOSTICS else 'original-data'
            suite = 'ace-m1-original-data-' + ('diagnostics' if name in DATA_DIAGNOSTICS else 'q4') + '-941'
            locator = '/'.join((suite, 'ace.m1.' + family + '.' + name.lower(), 'av' if target == 'av' else 'ac-native'))
            require(locator in cell['entries'], 'exact original data target locator required')
        if ns == 'scenario':
            wanted_gaps.append('full-scenario-sequence-and-original-samples')
        if original['cert_axis'] == 'cert-gap':
            wanted_gaps.append('legacy-a2r-cert-gap')
        typed_equal(cell['gaps'], wanted_gaps, 'required unresolved items cannot disappear')
        gaps.update(wanted_gaps)
        digest = manifest['corpus_src']['files'].get(name + '.at') if ns == 'corpus' else None
        if digest:
            typed_equal(cell['frozen_source'], {'path':CORPUS + 'src/' + name + '.at', 'sha256':digest}, 'approved corpus source')
            current(cell['frozen_source']['path'], digest)
        else:
            typed_equal(cell['frozen_source'], None, 'no invented Auto source')
        for entry_key in cell['entries']:
            require(entry_key in data['entries'], 'unbound formal entry reference')
            entry = data['entries'][entry_key]
            require(target_matches(target, entry), 'entry belongs to another target/domain')
            if digest:
                typed_equal(entry['source_files'][0]['sha256'], digest, 'same approved source bytes')
            used.add(entry_key)
        typed_equal(cell['input_kind'], 'scenario' if ns == 'scenario' else 'typed-fixture' if name.startswith('W')
                    else 'resident-source' if name.startswith('H') else 'auto-source', 'source vs typed/scenario boundary')
        typed_equal(cell['link_basis'], 'original-scenario-contract' if ns == 'scenario'
                    else 'original-wide-kind-bits' if name.startswith('W')
                    else 'original-resident-round-split' if name.startswith('H')
                    else 'approved-extension-case' if name.startswith(('U', 'O')) else 'exact-frozen-source',
                    'explicit source/typed/scenario mapping basis')
        if ns == 'corpus' and name.startswith('W'):
            fixture = {'W01':'i64-min', 'W02':'u64-max', 'W03':'neg-zero', 'W04':'fneg-nan', 'W05':'f32-nan-payload'}[name]
            wanted = ('av.data.typed', 'typed-' + fixture) if target == 'av' else (
                'ace.m1.' + name.lower() + '-mn891', 'mn-' + name.lower() + '-typed')
            require(len(cell['entries']) == 1, 'one original wide typed locator required')
            entry = data['entries'][cell['entries'][0]]
            typed_equal((entry['case'], entry['check']), wanted, 'original wide kind-specific entry')
        typed_equal(cell['budget_domain'], 'native' if target == 'ac' else 'av' if target == 'av' else 'legacy-in-process', 'original budget domain')
    require(seen == ledger.keys() and used == data['entries'].keys(), 'missing required cell or orphaned entry')
    return {'schema':'ace.m1.legacy-obligation-discovery/1', 'required':len(seen), 'linked_cells':len(seen)-len(unbound),
            'formal_entries':len(used), 'unbound_cells':unbound, 'gaps':dict(gaps), 'current_files':len(verified),
            'original_samples':SAMPLES, 'execution':'UNOBSERVED', 'executed':0, 'mapping_complete':False,
            'can_claim_expanded_m1':False}


def admit_legacy_obligations(inputs):
    inputs.sha256(MAP, MAP_SHA)
    data, digest = inputs.read_json(MAP)
    result = validate_map(inputs, data)
    return result | {'map_sha256':digest}
