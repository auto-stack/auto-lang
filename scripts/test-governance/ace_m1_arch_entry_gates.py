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
import subprocess
import tomllib

from ace_m1_arch_acceptance import AdmissionError, counter, exact_keys, hash_value
from ace_m1_arch_obligations import REQUIRED
from ace_m1_case_evidence import require
from ace_m1_legacy import typed_equal

CONTRACT = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-obligations.json'
RECIPES = 'crates/auto-lang/test/cases/ace-m1-expanded/architecture-entry-recipes.json'
RECIPES_SHA = '78e44b2e06cc76ab81aa61f6c6bbcce17e1ca71fbecd58780ce78aeadb467baa'
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


def _verify_commit_object(inputs, commit, frozen_anchors=()):
    """R-02: the frozen producer commit must resolve to a real commit object
    in the audited repository and belong to the frozen anchor closure.

    Read-only metadata query (git rev-parse --verify), never a language or
    test execution tool. A well-formed 40-hex string that names no object,
    a commit from another repository, or a commit outside the frozen anchor
    closure is refused before any row can be admitted (F-1013-01)."""
    require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None,
            'frozen producer commit')
    if frozen_anchors:
        require(commit in frozen_anchors, 'frozen producer commit outside the frozen anchor closure')
    try:
        subprocess.run(['git', '-C', str(inputs.root), 'rev-parse', '--verify', commit + '^{commit}'],
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except (subprocess.CalledProcessError, OSError):
        raise AdmissionError(
            f'commit is not a real commit object in the audited repository: {commit}')


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


def _read_json_tracked(evidence, relative, transients):
    """Read one envelope document whose bytes/decoded form die with the gate."""
    raw = evidence.read_bytes(relative)
    value, sha = evidence.decode_json(raw)
    transients.append(raw)
    return value, sha


def _release_gate_transients(evidence, transients):
    """PLAN-1013 T-07: per-entry whole-path release. Every envelope document
    (gate envelope, window pins, command receipts, phased facts projection)
    served its verification inside this gate and is dead by return; only the
    verified compact projection stays live for row evaluation."""
    for raw in transients:
        nodes = sum(raw.count(bytes([c])) for c in b'{}[],:')
        evidence.budget.release_transient((len(raw) + 1) + (7 * len(raw) + 256 * nodes + 4096))


def _admit_gate_root(inputs, evidence, root, frozen_anchors=(), transients=None):
    """Admit one entry gate directory: identity + every pinned original byte."""
    from ace_m1_first_slice import _repo_relative
    if transients is None:
        document, _ = evidence.read_json(root + '/gate.json')
    else:
        document, _ = _read_json_tracked(evidence, root + '/gate.json', transients)
    exact_keys(document, _GATE_FIELDS, 'architecture entry gate envelope')
    typed_equal(document['schema'], 'ace.m1.arch-entry-gate/1', 'entry gate schema')
    require(document['kind'] in ('native', 'case'), 'entry gate kind')
    require(type(document['group']) is str and document['group'], 'entry gate group')
    commit = document['commit']
    require(type(commit) is str and re.fullmatch('[0-9a-f]{40}', commit) is not None, 'frozen producer commit')
    if frozen_anchors:
        require(commit in frozen_anchors, 'frozen producer commit outside the frozen anchor closure')
    _verify_commit_object(inputs, commit, frozen_anchors)
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


def _window_pins(evidence, root, closure, transients=None):
    """Frozen producer window: identical before/after snapshots inside the closure."""
    if transients is None:
        before, _ = evidence.read_json(root + '/source-pins-before.json')
        after, _ = evidence.read_json(root + '/source-pins-after.json')
    else:
        before, _ = _read_json_tracked(evidence, root + '/source-pins-before.json', transients)
        after, _ = _read_json_tracked(evidence, root + '/source-pins-after.json', transients)
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


def _verified_compact(evidence, derived):
    """PLAN-1013 T-07: retain the VERIFIED compact projection per gate.

    The identity-pinned derived rows served their whole-path role inside the
    per-gate admission (re-derivation from pinned originals, item-by-item
    facts comparison, commit authenticity gate) and die here — exactly the
    plan §2 default of reading originals per entry and releasing transients.
    Row evaluation only consumes commit/green/executed and per-fact verdicts,
    so the projection carries the same verdicts raw admission would; it is
    derived, never read back from a self-reported summary (F-1013-01)."""
    rows = derived.get('terminals') if 'terminals' in derived else derived.get('checks') or {}
    evidence.budget.release_transient(_doc_transient(repr(rows).encode('utf-8')))
    compact = {'commit': derived['commit'], 'green': derived['green'],
               'executed': derived['executed']}
    if 'terminals' in derived:
        compact['terminals'] = {name: {'event': row.get('event')}
                                for name, row in derived['terminals'].items()}
    else:
        compact['checks'] = {key: {'status': row.get('status')}
                             for key, row in derived['checks'].items()}
    _retain(evidence, compact)
    return compact


def admit_native_gate(inputs, evidence, root, group_name, closure, keep_events=None,
                      frozen_anchors=()):
    """Full-group native admission: selection, terminals, receipts, identity.

    Returns {'commit', 'green', 'terminals', 'excluded', 'executed'}; every
    fact is re-derived from the pinned original bytes, never from a summary.
    """
    from ace_m1_first_slice import _repo_relative
    transients = []
    gate = _admit_gate_root(inputs, evidence, root, frozen_anchors, transients)
    typed_equal(gate['kind'], 'native', 'native entry gate kind')
    group = _group_config(inputs, group_name)
    require(group['runner'] == 'nextest', 'native fixed group runner')
    window = _window_pins(evidence, root, closure, transients)
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
    # Component bound for the listing parse (PLAN-1013 T-07): decoded text
    # (<=4x), json.loads tree + the parent's selected/excluded indexes built
    # from it (<=5x for this dense per-test shape), iteration slack. Same
    # never-understate rule as the run-stream bound above.
    listing_transient = 10 * len(listing_raw) + (1 << 16)
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
    listing_command, _ = _read_json_tracked(evidence, root + '/discovery.command.json', transients)
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
    # Per-component transient bound for the raw stream parse (PLAN-1013 T-07):
    # the decoded text (<=4x raw), the parent parser's internal re-decode plus
    # splitlines list (<=5x raw), and its per-test result index (512x selected,
    # the unchanged component bound). Per-line strict decodes in the loop below
    # reserve and release their own exact bound, so they need no headroom here.
    # This never understates the live set while removing the whole-document
    # node-count overestimate that priced a ~1 MiB libtest stream at ~26 MiB.
    raw_transient = 10 * len(raw) + 512 * len(selected) + (1 << 16)
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
    command, _ = _read_json_tracked(evidence, root + '/run.command.json', transients)
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
    _release_gate_transients(evidence, transients)
    return {'commit': gate['commit'], 'green': outcome == 'PASS',
            'terminals': rows, 'executed': executed}


def admit_case_gate(inputs, evidence, root, group_name, closure, keep_checks=None,
                    frozen_anchors=()):
    """Full-group Case admission: declared inventory, parent report, receipts.

    Returns {'commit', 'green', 'checks'} where checks maps (case, check) to
    the original parent report row with its identity pins."""
    from ace_m1_first_slice import _repo_relative
    transients = []
    gate = _admit_gate_root(inputs, evidence, root, frozen_anchors, transients)
    typed_equal(gate['kind'], 'case', 'case entry gate kind')
    group = _group_config(inputs, group_name)
    require(group['runner'] == 'case', 'case fixed group runner')
    window = _window_pins(evidence, root, closure, transients)
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
    inventory_command, _ = _read_json_tracked(evidence, root + '/inventory.command.json', transients)
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
    run_command, _ = _read_json_tracked(evidence, root + '/run.command.json', transients)
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
    _release_gate_transients(evidence, transients)
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


def _entry_closure(inputs, name, declared, config, oracles_sha=None):
    """The frozen producer-window closure for one entry: the declared source/
    config/oracle pins the raw reader re-verifies against the originals.
    `oracles_sha` (A-1013-01 knife 2) lets one admission read the frozen
    oracle table once instead of once per case entry; content identity is
    unchanged (same bytes, same hash)."""
    if declared['kind'] == 'case-suite':
        inputs.sha256(declared['suite_file'], declared['source_sha256'])
        if oracles_sha is None:
            oracles_sha = hashlib.sha256(
                inputs.read_bytes('scripts/test-governance/case-control-oracles.json')).hexdigest()
        return {'scripts/test-groups.toml': config,
                declared['suite_file']: declared['source_sha256'],
                'scripts/test-governance/case-control-oracles.json': oracles_sha}
    if declared['kind'] == 'native-module':
        inputs.sha256(declared['source_file'], declared['source_sha256'])
        return {'scripts/test-groups.toml': config,
                declared['source_file']: declared['source_sha256']}
    owner, _resident, _modules = DELIVERED_PROVIDERS[name]
    require(declared.get('owner') == owner, f'delivered provider owner {name}')
    return {'scripts/test-groups.toml': config}


def admit_arch_entry_gates(inputs, evidence, binding, frozen_anchors=()):
    """binding: {'entries': {entry-name: {'root': <gate dir>} | {'facts': {...}, 'root': <gate dir>}}} —
    strict envelope + per-row recipe admission; never emits the review row.

    `frozen_anchors` (optional binding-level list of 40-hex commits) names the
    audited frozen producer identity; every admitted gate commit must belong to
    it (TAG-01/03 commitment). A phased facts reference must ALSO carry the
    original gate root: the final composition re-derives every entry from the
    pinned originals through the same strict raw reader and compares the facts
    projection item by item — a self-reported facts document alone proves
    nothing (F-1013-01)."""
    require(type(binding) is dict and 'entries' in binding and set(binding) <= {'entries', 'frozen_anchors'},
            'architecture entry-gate binding')
    require(type(binding['entries']) is dict and binding['entries'], 'entry gate map required')
    if not frozen_anchors and 'frozen_anchors' in binding:
        frozen_anchors = binding['frozen_anchors']
    if frozen_anchors:
        require(type(frozen_anchors) in (list, tuple) and all(
            type(c) is str and re.fullmatch('[0-9a-f]{40}', c) for c in frozen_anchors),
            'frozen anchors must be a list of 40-hex commits')
        frozen_anchors = frozenset(frozen_anchors)
    # A-1013-01 knife 2: the contract document is also a function-level
    # transient — validate, copy the per-entry declarations and the small
    # obligations list, release the whole-document reservation.
    contract_raw = inputs.read_bytes(CONTRACT)
    document, _ = inputs.decode_json(contract_raw)
    require(document.get('schema') == 'ace.m1.arch-obligation-contract/1', 'architecture contract schema')
    entries = {name: dict(declared) for name, declared in document['entries'].items()}
    obligations = [dict(obligation) for obligation in document['obligations']]
    _release_gate_transients(inputs, [contract_raw])
    del document, contract_raw
    # PLAN-1013 A-1013-01: the recipe document is a FUNCTION-LEVEL transient.
    # After validation, keep-building and the per-row fact projection below,
    # its full-document reservation is released; row evaluation consumes the
    # compact projection (copied per-row check lists — rows that share an
    # entry carry different lists, so the keep union can never replace it).
    # Same data, same verification path, same verdicts — only the retention
    # policy follows the plan §2 read-per-entry/release-transients default.
    recipes_raw = inputs.read_bytes(RECIPES)
    recipes_doc, recipes_digest = inputs.decode_json(recipes_raw)
    typed_equal(recipes_digest, RECIPES_SHA, 'frozen recipe contract identity')
    validate_recipes(recipes_doc, entries)

    oracles_sha = None
    if any(e['kind'] == 'case-suite' for e in entries.values()):
        oracles_sha = hashlib.sha256(
            inputs.read_bytes('scripts/test-governance/case-control-oracles.json')).hexdigest()
    keep = {}
    row_facts = {}
    for recipe in recipes_doc['recipes']:
        facts = []
        for fact in recipe['facts']:
            entry = fact['entry']
            if 'case_checks' in fact:
                pairs = [tuple(pair) for pair in fact['case_checks']]
                keep.setdefault(('case', entry), set()).update(pairs)
                facts.append({'entry': entry, 'case_checks': pairs})
            else:
                events = list(fact['native_tests'])
                keep.setdefault(('native', entry), set()).update(events)
                facts.append({'entry': entry, 'native_tests': events})
        row_facts[recipe['row']] = facts
    _release_gate_transients(inputs, [recipes_raw])
    del recipes_doc, recipes_raw
    _retain(inputs, row_facts)
    gates = {}
    for name in sorted(binding['entries']):
        ref = binding['entries'][name]
        require(name in entries, f'foreign entry family {name}')
        declared = entries[name]
        config = config_digest(inputs)
        if 'facts' in ref:
            # PLAN-1013 T-03: the phased facts document is a checked projection
            # of a real gate directory, never a trusted summary. The reference
            # must carry the original root; the final composition re-derives
            # every fact from the pinned originals and compares item by item.
            exact_keys(ref, ('facts', 'root'), f'phased entry facts reference {name}')
            facts_ref = ref['facts']
            exact_keys(facts_ref, ('path', 'sha256'), f'phased entry facts {name}')
            phased_transients = []
            facts_doc, digest = _read_json_tracked(evidence, facts_ref['path'], phased_transients)
            if digest != facts_ref['sha256']:
                raise ValueError(f'STALE artifact content: {facts_ref["path"]}')
            exact_keys(facts_doc, ('schema', 'entry', 'group', 'kind', 'commit', 'green',
                                   'executed', 'kept', 'gate_originals_sha256'),
                       f'phased entry facts {name}')
            typed_equal(facts_doc['schema'], 'ace.m1.arch-entry-facts/1', 'phased facts schema')
            typed_equal(facts_doc['entry'], name, 'phased facts entry identity')
            require(type(facts_doc['group']) is str and facts_doc['group'], 'phased facts group')
            require(type(facts_doc['green']) is bool, 'phased facts gate status')
            require(type(facts_doc['executed']) is int and facts_doc['executed'] > 0,
                    'phased facts executed count')
            require(type(facts_doc['kept']) is dict, 'phased facts rows required')
            require(type(facts_doc['gate_originals_sha256']) is dict
                    and facts_doc['gate_originals_sha256'], 'phased facts gate identity')
            expected_group = declared['group'] if declared['kind'] != 'provider-pending'                 else DELIVERED_PROVIDERS[name][1]
            typed_equal(facts_doc['group'], expected_group, 'phased facts group identity')
            expected_kind = 'case' if declared['kind'] == 'case-suite' else 'native'
            typed_equal(facts_doc['kind'], expected_kind, 'phased facts gate kind')
            closure = _entry_closure(inputs, name, declared, config, oracles_sha)
            if expected_kind == 'native':
                resident = declared['group'] if declared['kind'] == 'native-module' else DELIVERED_PROVIDERS[name][1]
                derived = admit_native_gate(inputs, evidence, ref['root'], resident, closure,
                                            keep_events=keep.get(('native', name)),
                                            frozen_anchors=frozen_anchors)
                typed_equal(derived['commit'], facts_doc['commit'], 'phased facts commit drift')
                typed_equal(derived['green'], facts_doc['green'], 'phased facts green drift')
                typed_equal(derived['executed'], facts_doc['executed'], 'phased facts executed drift')
                require(set(facts_doc['kept']) == set(derived['terminals']),
                        'phased facts kept identity drift')
                for key, row in facts_doc['kept'].items():
                    event = row.get('event') if isinstance(row, dict) else row
                    terminal = derived['terminals'][key]
                    require(terminal.get('event') == event, f'phased facts kept drift {name}:{key}')
                gates[name] = _verified_compact(evidence, derived)
            else:
                derived = admit_case_gate(inputs, evidence, ref['root'], expected_group, closure,
                                          keep_checks=keep.get(('case', name)),
                                          frozen_anchors=frozen_anchors)
                typed_equal(derived['commit'], facts_doc['commit'], 'phased facts commit drift')
                typed_equal(derived['green'], facts_doc['green'], 'phased facts green drift')
                typed_equal(derived['executed'], facts_doc['executed'], 'phased facts executed drift')
                require(set(facts_doc['kept']) == set('|'.join(key) for key in derived['checks']),
                        'phased facts kept identity drift')
                for key, row in facts_doc['kept'].items():
                    status = row.get('status') if isinstance(row, dict) else row
                    case, check = key.split('|')
                    check_row = derived['checks'][(case, check)]
                    require(check_row.get('status') == status, f'phased facts kept drift {name}:{key}')
                gates[name] = _verified_compact(evidence, derived)
            gate_env, _ = _read_json_tracked(evidence, ref['root'] + '/gate.json', phased_transients)
            require(gate_env.get('originals') == facts_doc['gate_originals_sha256'],
                    'phased facts gate identity drift')
            _release_gate_transients(evidence, phased_transients)
            continue
        exact_keys(ref, ('root',), f'entry gate reference {name}')
        if declared['kind'] == 'case-suite':
            closure = _entry_closure(inputs, name, declared, config, oracles_sha)
            gates[name] = _verified_compact(evidence, admit_case_gate(
                inputs, evidence, ref['root'], declared['group'], closure,
                keep_checks=keep.get(('case', name)), frozen_anchors=frozen_anchors))
        elif declared['kind'] == 'native-module':
            closure = _entry_closure(inputs, name, declared, config, oracles_sha)
            gates[name] = _verified_compact(evidence, admit_native_gate(
                inputs, evidence, ref['root'], declared['group'], closure,
                keep_events=keep.get(('native', name)), frozen_anchors=frozen_anchors))
        else:
            owner, resident, _modules = DELIVERED_PROVIDERS[name]
            require(declared.get('owner') == owner, f'delivered provider owner {name}')
            closure = _entry_closure(inputs, name, declared, config, oracles_sha)
            gates[name] = _verified_compact(evidence, admit_native_gate(
                inputs, evidence, ref['root'], resident, closure,
                keep_events=keep.get(('native', name)), frozen_anchors=frozen_anchors))

    observations = {}
    admission_facts = {}
    for obligation in obligations:
        row = obligation['id']
        if row == REVIEW_ROW:
            continue  # only a verified pre-signature credential supplies it
        needed = obligation['entries']
        if [n for n in needed if n not in gates]:
            continue  # row stays UNOBSERVED; never partially claimed
        facts = row_facts[row]
        red = [fact['entry'] for fact in facts if not gates[fact['entry']]['green']]
        if red:
            observations[row] = 'FAIL'  # a really red referenced entry is a real failure
            continue
        facts_green = True
        for fact in facts:
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
                                       for f in facts)
            observations[row] = 'PASS'
        # else: required named facts absent while the group is green -> UNOBSERVED
    return {'schema': 'ace.m1.arch-entry-gates/2', 'observations': observations,
            'admitted_entries': sorted(gates),
            'admission_facts': admission_facts,
            'gate_commits': sorted({g['commit'] for g in gates.values()}),
            'review_row': REVIEW_ROW,
            'source_evidence': {name: ('green' if g['green'] else 'red') for name, g in gates.items()}}
