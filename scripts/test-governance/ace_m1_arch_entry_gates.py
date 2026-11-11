"""PLAN-1005 (F-1005-01): strict architecture-row entry-gate admission.

Each obligation row names the evidence entries that own it. An entry is
admitted ONLY from a real single-group fixed-test run directory captured by
the original parent (`scripts/fixed-tests.py --out`): the reader re-derives
the complete selection and every terminal event from the captured original
discovery listing and raw run output, checks the command receipts, the frozen
producer window, the current group configuration, the actually listed binary
bytes, and the entry's pinned source identity. A row passes ONLY when every
declared entry has an admitted green gate AND every named recipe fact (suite
check or native test of the pinned module) actually executed green inside
that gate. Group green never supplies a row by itself, and `executed=0` /
empty results can never pass (F-1005-01).

The review-owned row (ARCH-08/fresh-union-spec-review) is NEVER supplied by
this reader, by mechanism families, or by any section injection: only a
verified pre-signature review credential admits it (F-1005-02, see
ace_m1_review_evidence).
"""
import hashlib
import importlib.util
import json
import re
import tomllib

from ace_m1_arch_acceptance import counter, exact_keys, hash_value
from ace_m1_arch_obligations import REQUIRED
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-obligations.json'
RECIPES = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-entry-recipes.json'
RECIPES_SHA = '0f224ad5e14b1f3f34b946206d93aba6c3c6392256a4fb0025f76dd335af9fc1'
COMPOSER_FACTS = ('admission-peak-within-live-cap', 'document-byte-caps-enforced',
                  'denominator-and-honest-three-state', 'raw-terminal-bound-execution')
REVIEW_ROW = 'ARCH-08/fresh-union-spec-review'
NATIVE_ORIGINALS = ('results.json', 'discovery.stdout', 'run.stdout', 'run.command.json',
                    'discovery.command.json')
CASE_ORIGINALS = ('results.json', 'inventory.stdout', 'inventory.command.json', 'run.command.json', 'report.json')

# delivered provider plans: entry -> (owner plan, resident evidence group, modules)
DELIVERED_PROVIDERS = {
    'foreign': ('PLAN-938', 'ace-abi', ('source_foreign',)),
    'foreign-resource': ('PLAN-939', 'ace-abi', ('resource', 'resource_recovery')),
}

_GATE_FIELDS = ('schema', 'kind', 'group', 'commit', 'binaries', 'originals')


def config_digest(inputs):
    return hashlib.sha256(inputs.read_bytes('scripts/test-groups.toml')).hexdigest()


def _parent_parsers(inputs):
    """Reuse the fixed parent's pure selection/terminal parsers, never its runner."""
    spec = importlib.util.spec_from_file_location('arch_entry_parent', inputs.path('scripts/fixed-tests.py'))
    parent = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(parent)
    return parent


def _group_config(inputs, name):
    config = tomllib.loads(inputs.read_bytes('scripts/test-groups.toml').decode('utf-8'))
    group = next((g for g in config['group'] if g['name'] == name), None)
    require(group is not None, f'unknown fixed group {name}')
    return group


def _admit_gate_root(inputs, evidence, root):
    """Admit one entry gate directory: identity + every pinned original byte."""
    from ace_m1_first_slice import _repo_relative
    document, _ = evidence.read_json(root + '/gate.json')
    exact_keys(document, _GATE_FIELDS, 'architecture entry gate envelope')
    typed_equal(document['schema'], 'ace.m1.arch-entry-gate/1', 'entry gate schema')
    require(document['kind'] in ('native', 'case'), 'entry gate kind')
    require(type(document['group']) is str and document['group'], 'entry gate group')
    commit = document['commit']
    require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None, 'frozen producer commit')
    binaries = document['binaries']
    require(type(binaries) is dict, 'gate binary pins required')
    for name, digest in sorted(binaries.items()):
        hash_value(digest, 'gate binary sha')
        require(type(name) is str and name and '\\' not in name and ':' not in name
                and not name.startswith('/') and '..' not in name.split('/'),
                'gate binary pin must be a normalized relative POSIX path')
    require(type(document['originals']) is dict, 'original run outputs required')
    required_originals = NATIVE_ORIGINALS if document['kind'] == 'native' else CASE_ORIGINALS
    require(set(required_originals) <= set(document['originals']), 'original run outputs incomplete')
    for name, digest in sorted(document['originals'].items()):
        hash_value(digest, 'original output sha')
        require(type(name) is str and name and '/' not in name and '\\' not in name,
                'original output must sit directly in the gate root')
        evidence.sha256(root + '/' + name, digest)
    return document


def _window_pins(evidence, root, closure):
    """Frozen producer window: identical before/after snapshots inside the closure."""
    before, _ = evidence.read_json(root + '/source-pins-before.json')
    after, _ = evidence.read_json(root + '/source-pins-after.json')
    exact_keys(before, ('commit', 'source_pins'), 'entry before input snapshot')
    exact_keys(after, ('commit', 'source_pins'), 'entry after input snapshot')
    typed_equal(before, after, 'entry window source input stability')
    pins = before['source_pins']
    require(type(pins) is dict and bool(pins), 'entry window input set required')
    require(set(pins) <= set(closure), 'entry window snapshot belongs to the declared closure')
    for path, digest in pins.items():
        typed_equal(digest, closure[path], 'entry window input agrees with frozen declaration')
    return before


def _doc_transient(raw):
    nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
    return len(raw) + 1 + 7 * len(raw) + 256 * nodes + 4096


def _retain(evidence, value):
    """Reserve the compact bound of a small derived structure that stays live."""
    raw = repr(value).encode('utf-8')
    evidence.reserve(_doc_transient(raw))


def admit_native_gate(inputs, evidence, root, group_name, closure, keep_events=None):
    """Full-group native admission: selection, terminals, receipts, identity.

    Returns {'commit', 'green', 'terminals', 'excluded', 'executed'}; every
    fact is re-derived from the pinned original bytes, never from a summary.
    """
    from ace_m1_first_slice import _repo_relative
    gate = _admit_gate_root(inputs, evidence, root)
    typed_equal(gate['kind'], 'native', 'native entry gate kind')
    group = _group_config(inputs, group_name)
    require(group['runner'] == 'nextest', 'native fixed group runner')
    window = _window_pins(evidence, root, closure)
    typed_equal(window['commit'], gate['commit'], 'native gate window commit')
    clean_state = hashlib.sha256(gate['commit'].encode() + b'[]').hexdigest()
    summary_raw = evidence.read_bytes(root + '/results.json')
    summary_transient = _doc_transient(summary_raw)
    evidence.reserve(summary_transient - (len(summary_raw) + 1))
    summary = json.loads(summary_raw.decode('utf-8'))
    require(type(summary) is dict, 'original parent summary required')
    typed_equal(summary.get('selected_groups'), [group_name], 'original single-group selection')
    typed_equal(summary.get('config_sha256'), config_digest(inputs), 'original group configuration')
    typed_equal(summary.get('source_state'), clean_state, 'clean frozen producer window')
    require('error' not in summary, 'fixed driver changed inputs or failed during execution')
    require(type(summary.get('results')) is list and len(summary['results']) == 1,
            'one original native invocation')
    result = summary['results'][0]

    listing_raw = evidence.read_bytes(root + '/discovery.stdout')
    listing_transient = _doc_transient(listing_raw)
    evidence.reserve(listing_transient - (len(listing_raw) + 1))
    listing = json.loads(listing_raw.decode('utf-8'))
    binaries = {suite['binary-path'] for suite in listing['rust-suites'].values()
                if suite.get('binary-path')}
    require(binaries, 'native listing carries no real binaries')
    for path in sorted(binaries):
        relative = _repo_relative(inputs, path)
        hash_value(gate['binaries'].get(relative), 'listed binary pin')
        inputs.sha256(relative, gate['binaries'][relative])
    parent = _parent_parsers(inputs)
    # The original complete runner invocation, derived through the parent's
    # own plan/argument builders against this repository's configuration.
    plan_group = parent.invocations([dict(group)])[0]
    listing_command, _ = evidence.read_json(root + '/discovery.command.json')
    typed_equal(listing_command['argv'],
                ['cargo', 'nextest', 'list', *parent.nextest_args(plan_group),
                 '--run-ignored', 'all', '--message-format', 'json'],
                'original native discovery invocation')
    expected_argv = ['cargo', 'nextest', 'run', *parent.nextest_args(plan_group),
                     '--no-fail-fast', '--no-tests', 'fail',
                     '--message-format', 'libtest-json-plus', '--message-format-version', '0.1']
    selected, excluded = parent.enumerate_rust(listing, root=str(inputs.root))
    require(selected, 'native group selected zero actual tests')
    raw = evidence.read_bytes(root + '/run.stdout')
    raw_transient = _doc_transient(raw) + 512 * len(selected)
    evidence.reserve(raw_transient - (len(raw) + 1))
    text = raw.decode('utf-8', errors='strict')
    for line in text.splitlines():
        if line.startswith('{'):
            before = evidence.reserved
            evidence.decode_json(line.encode('utf-8'))
            # strict-JSON validation only: the decoded event dies immediately
            frame = len(line.encode('utf-8')) + 1
            evidence.budget.release_transient(evidence.reserved - before)
    known = {suite['package-name'] + '::' + suite['binary-name'] + '$' + name
             for suite in listing['rust-suites'].values() for name in suite.get('testcases', {})}
    terminals = parent.terminal_results(raw, selected, known - selected.keys())
    errors = terminals['errors']
    executed = len(terminals['results'])
    # The raw listing/stream transients die here: only the compact per-row
    # verdicts the recipes consume stay live for the rest of the composition.
    evidence.budget.release_transient(listing_transient)
    evidence.budget.release_transient(raw_transient)
    del listing, listing_raw, raw, text, known
    command, _ = evidence.read_json(root + '/run.command.json')
    require(command['meter']['error'] is None and command['meter']['exit'] == command['exit'],
            'actual native process terminal')
    typed_equal(command['stdout'], 'run.stdout', 'raw stdout reference')
    typed_equal(command['stderr'], 'run.stderr', 'raw stderr reference')
    typed_equal(command['argv'], expected_argv,
                'original complete native runner invocation')
    typed_equal(command['cwd'], str(inputs.root / group['workspace']), 'actual producer working directory')
    counter(result.get('executed'), 'parent executed count')
    typed_equal(result['executed'], executed, 'native executed count matches raw terminals')
    typed_equal(result['selected'], list(selected.values()), 'complete actual native selection')
    typed_equal(result['excluded'], excluded, 'original conditional exclusions')
    typed_equal(result['results'], terminals['results'], 'native terminals cannot be replaced by a summary')
    require(all(row.get('ignored') is True for row in excluded), 'only ignored:true conditional exclusions')
    outcome = 'PASS' if command['exit'] == 0 and not errors else 'FAIL'
    typed_equal(result['outcome'], outcome, 'real native parent outcome')
    rows = terminals['results']
    if keep_events is not None:
        rows = {name: row for name, row in rows.items() if name in keep_events}
    evidence.budget.release_transient(summary_transient)
    _retain(evidence, rows)
    return {'commit': gate['commit'], 'green': outcome == 'PASS',
            'terminals': rows, 'executed': executed}


def admit_case_gate(inputs, evidence, root, group_name, closure, keep_checks=None):
    """Full-group Case admission: declared inventory, parent report, receipts.

    Returns {'commit', 'green', 'checks'} where checks maps (case, check) to
    the original parent report row with its identity pins."""
    from ace_m1_first_slice import _repo_relative
    gate = _admit_gate_root(inputs, evidence, root)
    typed_equal(gate['kind'], 'case', 'case entry gate kind')
    group = _group_config(inputs, group_name)
    require(group['runner'] == 'case', 'case fixed group runner')
    window = _window_pins(evidence, root, closure)
    typed_equal(window['commit'], gate['commit'], 'case gate window commit')
    clean_state = hashlib.sha256(gate['commit'].encode() + b'[]').hexdigest()
    summary_raw = evidence.read_bytes(root + '/results.json')
    summary_transient = _doc_transient(summary_raw)
    evidence.reserve(summary_transient - (len(summary_raw) + 1))
    summary = json.loads(summary_raw.decode('utf-8'))
    typed_equal(summary.get('selected_groups'), [group_name], 'original single-group selection')
    typed_equal(summary.get('config_sha256'), config_digest(inputs), 'original group configuration')
    typed_equal(summary.get('source_state'), clean_state, 'clean frozen producer window')
    require('error' not in summary, 'fixed driver changed inputs or failed during execution')
    require(type(summary.get('results')) is list and len(summary['results']) == 1,
            'one original case invocation')

    inventory_raw = evidence.read_bytes(root + '/inventory.stdout')
    inventory_transient = _doc_transient(inventory_raw)
    evidence.reserve(inventory_transient - (len(inventory_raw) + 1))
    inventory = json.loads(inventory_raw.decode('utf-8'))
    require(inventory.get('schema') == 'test-case-inventory/1', 'original TestCase inventory')
    checks = inventory['checks']
    require(type(checks) is list and bool(checks), 'inventory checks required')
    binary = None
    inventory_command, _ = evidence.read_json(root + '/inventory.command.json')
    require(inventory_command['argv'][1:] == ['inventory', '--suite', str(inputs.root / group['suite'])],
            'original inventory invocation')
    binary = inventory_command['argv'][0]
    relative = _repo_relative(inputs, binary)
    hash_value(gate['binaries'].get(relative), 'case parent binary pin')
    inputs.sha256(relative, gate['binaries'][relative])
    overrides = json.loads(inputs.read_bytes('scripts/test-governance/case-control-oracles.json'))['overrides']
    suite_names = {row['suite'] for row in checks}
    expectations = {(row['case'], row['check']): 'PASS' for row in checks if row['state'] == 'RUN'}
    for row in overrides:
        if row['suite'] in suite_names and (row['case'], row['check']) in expectations:
            expectations[(row['case'], row['check'])] = row['expected_status']
    report_raw = evidence.read_bytes(root + '/report.json')
    report_transient = _doc_transient(report_raw)
    evidence.reserve(report_transient - (len(report_raw) + 1))
    report = json.loads(report_raw.decode('utf-8'))
    require(type(report.get('results')) is list, 'original parent report rows')
    actual = {}
    for row in report['results']:
        key = (row['case'], row['check'])
        require(key not in actual, 'duplicate parent report row')
        actual[key] = row
    typed_equal(set(actual), set(expectations), 'complete declared suite execution')
    require(not report.get('load_errors') and not report.get('coverage_gaps'),
            'original parent load/coverage gaps')
    declared = {(row['case'], row['check']): row for row in checks}
    valid = True
    for key, status in sorted(expectations.items()):
        row = actual[key]
        original = declared[key]
        valid &= row.get('status') == status and (status != 'FAIL' or bool(row.get('mismatches')))
        valid &= row.get('identity', {}).get('source') == original['source_sha256']
        valid &= row.get('identity', {}).get('oracle') == original['oracle_sha256']
    run_command, _ = evidence.read_json(root + '/run.command.json')
    require(run_command['meter']['error'] is None and run_command['meter']['exit'] == run_command['exit'],
            'actual case process terminal')
    expected_exit = 1 if 'FAIL' in expectations.values() else 0
    typed_equal(run_command['exit'], expected_exit, 'case parent exit matches its expectations')
    suite = str(inputs.root / group['suite'])
    typed_equal(run_command['argv'][:4], [binary, 'run', '--suite', suite],
                'original case runner invocation')
    typed_equal(run_command['cwd'], str(inputs.root), 'actual case producer working directory')
    result = summary['results'][0]
    typed_equal(result['executed'], len(actual), 'case executed count matches the original report')
    outcome = 'PASS' if valid else 'FAIL'
    typed_equal(result['outcome'], outcome, 'real case parent outcome')
    if keep_checks is not None:
        actual = {key: row for key, row in actual.items() if key in keep_checks}
    evidence.budget.release_transient(summary_transient)
    evidence.budget.release_transient(inventory_transient)
    evidence.budget.release_transient(report_transient)
    _retain(evidence, actual)
    return {'commit': gate['commit'], 'green': outcome == 'PASS',
            'checks': actual, 'executed': len(actual)}


def validate_recipes(document, entries):
    """The frozen recipe table: exact coverage, no review row, known shapes."""
    exact_keys(document, ('schema', 'plan', 'review_row', 'composer_facts_allowed', 'note', 'recipes'),
               'architecture recipe contract')
    typed_equal(document['schema'], 'ace.m1.arch-entry-recipe-contract/1', 'recipe schema')
    typed_equal(document['review_row'], REVIEW_ROW, 'the single review-owned row')
    typed_equal(sorted(document['composer_facts_allowed']), sorted(COMPOSER_FACTS), 'composer fact vocabulary')
    require(type(document['recipes']) is list, 'recipe rows required')
    seen = set()
    for recipe in document['recipes']:
        exact_keys(recipe, ('row', 'facts', 'composer_facts'), 'architecture row recipe')
        row = recipe['row']
        require(row in REQUIRED and row != REVIEW_ROW and row not in seen, 'foreign/duplicate recipe row')
        seen.add(row)
        require(type(recipe['facts']) is list and recipe['facts'], 'recipe facts required')
        names = set()
        for fact in recipe['facts']:
            keys = set(fact)
            require(keys == {'entry', 'case_checks'} or keys == {'entry', 'native_tests'}, 'recipe fact shape')
            require(fact['entry'] in entries, 'recipe entry must be declared')
            values = fact.get('case_checks', fact.get('native_tests'))
            require(type(values) is list and values, 'recipe fact values required')
            for value in values:
                require(value and str(value) not in names, 'duplicate/empty recipe fact')
                names.add(str(value))
        require(set(recipe['composer_facts']) <= set(COMPOSER_FACTS), 'unknown composer fact')
    typed_equal(seen, set(REQUIRED) - {REVIEW_ROW}, 'recipe coverage is every non-review row')


def admit_arch_entry_gates(inputs, evidence, binding):
    """binding: {'entries': {entry-name: {'root': <gate dir>}}} — strict
    envelope + per-row recipe admission; never emits the review row."""
    exact_keys(binding, ('entries',), 'architecture entry-gate binding')
    require(type(binding['entries']) is dict and binding['entries'], 'entry gate map required')
    document, _ = inputs.read_json(CONTRACT)
    require(document.get('schema') == 'ace.m1.arch-obligation-contract/1', 'architecture contract schema')
    entries = document['entries']
    obligations = document['obligations']
    recipes_doc, recipes_digest = inputs.read_json(RECIPES)
    typed_equal(recipes_digest, RECIPES_SHA, 'frozen recipe contract identity')
    validate_recipes(recipes_doc, entries)

    keep = {}
    for recipe in recipes_doc['recipes']:
        for fact in recipe['facts']:
            entry = fact['entry']
            if 'case_checks' in fact:
                keep.setdefault(('case', entry), set()).update(
                    (case, check) for case, check in fact['case_checks'])
            else:
                keep.setdefault(('native', entry), set()).update(fact['native_tests'])
    gates = {}
    for name in sorted(binding['entries']):
        ref = binding['entries'][name]
        require(name in entries, f'foreign entry family {name}')
        if 'facts' in ref:
            # PLAN-1005 T-08 phased admission: the full strict gate admission
            # ran in its own bounded phase (fresh live-data budget) and emitted
            # this compact hash-pinned facts document; the final composition
            # consumes only the per-row facts it needs. The phase is re-runnable
            # against the same gate directory (its receipts stay in the scope).
            exact_keys(ref, ('facts',), f'phased entry facts reference {name}')
            facts_ref = ref['facts']
            exact_keys(facts_ref, ('path', 'sha256'), f'phased entry facts {name}')
            document, digest = evidence.read_json(facts_ref['path'])
            if digest != facts_ref['sha256']:
                raise ValueError(f'STALE artifact content: {facts_ref["path"]}')
            exact_keys(document, ('schema', 'entry', 'group', 'kind', 'commit', 'green',
                                  'executed', 'kept', 'gate_originals_sha256'),
                       f'phased entry facts {name}')
            typed_equal(document['schema'], 'ace.m1.arch-entry-facts/1', 'phased facts schema')
            typed_equal(document['entry'], name, 'phased facts entry identity')
            require(type(document['group']) is str and document['group'], 'phased facts group')
            require(type(document['green']) is bool, 'phased facts gate status')
            require(type(document['executed']) is int and document['executed'] > 0,
                    'phased facts executed count')
            require(type(document['kept']) is dict, 'phased facts rows required')
            require(type(document['gate_originals_sha256']) is dict
                    and document['gate_originals_sha256'], 'phased facts gate identity')
            declared = entries[name]
            expected_group = declared['group'] if declared['kind'] != 'provider-pending'                 else DELIVERED_PROVIDERS[name][1]
            typed_equal(document['group'], expected_group, 'phased facts group identity')
            expected_kind = 'case' if declared['kind'] == 'case-suite' else 'native'
            typed_equal(document['kind'], expected_kind, 'phased facts gate kind')
            if expected_kind == 'native':
                kept = {key: {'event': row} if isinstance(row, str) else row
                        for key, row in document['kept'].items()}
            else:
                kept = {tuple(key.split('|')): {'status': row} if isinstance(row, str) else row
                        for key, row in document['kept'].items()}
            gates[name] = {'commit': document['commit'], 'green': document['green'],
                           'executed': document['executed'],
                           'terminals': kept if expected_kind == 'native' else None,
                           'checks': kept if expected_kind == 'case' else None}
            continue
        exact_keys(ref, ('root',), f'entry gate reference {name}')
        declared = entries[name]
        config = config_digest(inputs)
        if declared['kind'] == 'case-suite':
            inputs.sha256(declared['suite_file'], declared['source_sha256'])
            closure = {'scripts/test-groups.toml': config,
                       declared['suite_file']: declared['source_sha256'],
                       'scripts/test-governance/case-control-oracles.json':
                           hashlib.sha256(inputs.read_bytes('scripts/test-governance/case-control-oracles.json')).hexdigest()}
            gates[name] = admit_case_gate(inputs, evidence, ref['root'], declared['group'], closure,
                                          keep_checks=keep.get(('case', name)))
        elif declared['kind'] == 'native-module':
            inputs.sha256(declared['source_file'], declared['source_sha256'])
            closure = {'scripts/test-groups.toml': config,
                       declared['source_file']: declared['source_sha256']}
            gates[name] = admit_native_gate(inputs, evidence, ref['root'], declared['group'], closure,
                                            keep_events=keep.get(('native', name)))
        else:
            owner, resident, _modules = DELIVERED_PROVIDERS[name]
            require(declared.get('owner') == owner, f'delivered provider owner {name}')
            closure = {'scripts/test-groups.toml': config}
            gates[name] = admit_native_gate(inputs, evidence, ref['root'], resident, closure,
                                            keep_events=keep.get(('native', name)))

    recipes = {recipe['row']: recipe for recipe in recipes_doc['recipes']}
    observations = {}
    admission_facts = {}
    for obligation in obligations:
        row = obligation['id']
        if row == REVIEW_ROW:
            continue  # only a verified pre-signature credential supplies it
        needed = obligation['entries']
        if [n for n in needed if n not in gates]:
            continue  # row stays UNOBSERVED; never partially claimed
        recipe = recipes[row]
        red = [fact['entry'] for fact in recipe['facts'] if not gates[fact['entry']]['green']]
        if red:
            observations[row] = 'FAIL'  # a really red referenced entry is a real failure
            continue
        facts_green = True
        for fact in recipe['facts']:
            gate = gates[fact['entry']]
            if 'case_checks' in fact:
                for case, check in fact['case_checks']:
                    row_value = gate['checks'].get((case, check))
                    if row_value is None or row_value.get('status') != 'PASS':
                        facts_green = False
            else:
                for event in fact['native_tests']:
                    terminal = gate['terminals'].get(event)
                    if terminal is None or terminal.get('event') != 'ok':
                        facts_green = False
        if facts_green:
            admission_facts[row] = sum(len(f.get('case_checks', f.get('native_tests', [])))
                                       for f in recipe['facts'])
            observations[row] = 'PASS'
        # else: required named facts absent while the group is green -> UNOBSERVED
    return {'schema': 'ace.m1.arch-entry-gates/2', 'observations': observations,
            'admitted_entries': sorted(gates),
            'admission_facts': admission_facts,
            'gate_commits': sorted({g['commit'] for g in gates.values()}),
            'review_row': REVIEW_ROW,
            'source_evidence': {name: ('green' if g['green'] else 'red') for name, g in gates.items()}}
