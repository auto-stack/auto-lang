#!/usr/bin/env python3
"""PLAN-893 V-01/V-02: the committed ACE slow-test policy is conservation-safe.

Checks (static, no build): the 13 original obligations map one-to-one onto
current native identities, migration renames are recorded with evidence,
replay commands reference registered fixed groups and the frozen native leaf
names, and mutated policies (missing/extra/duplicate items, old leaf names
reused as current, nonexistent functions, wrong formal-axis membership,
unregistered groups) are rejected. select-only style knowledge alone cannot
pass: every current fn must really be declared in its recorded source file.
"""
from pathlib import Path
import json
import re
import unittest

ROOT = Path(__file__).resolve().parent.parent.parent
POLICY = ROOT / 'scripts/test-governance/ace-test-policy.json'
GROUPS = ROOT / 'scripts/test-groups.toml'

EXPECTED_COUNT = 13
KNOWN_T07_NAMES = {'t07_v07_512_revisions_bounded_history_stable_watermark',
                   't07_v07_full_storage_axis_512_edits_gc_restart_sentinel'}


def group_names():
    import tomllib
    document = tomllib.loads(GROUPS.read_text(encoding='utf-8'))
    return {g['name'] for g in document['group']}


def source_text(item):
    return (ROOT / item['native']['source_file']).read_text(encoding='utf-8', errors='replace')


def fn_is_declared(item):
    """The recorded fn must appear as a function declaration inside the file's
    in-file module chain. The file module itself (the source_file stem, mounted
    by the target's conformance.rs) and a leading `tests` namespace are not
    re-declared inside the file and are skipped."""
    text = source_text(item)
    if not re.search(r'\bfn\s+' + re.escape(item['native']['fn']) + r'\s*\(', text):
        return False
    chain = item['native']['test'].split('::')[:-1]
    stem = Path(item['native']['source_file']).stem
    if stem in chain:
        chain = chain[chain.index(stem) + 1:]
    elif chain and chain[0] == 'tests':
        chain = chain[1:]
    body = text
    for module in chain:
        match = re.search(r'\bmod\s+' + re.escape(module) + r'\s*\{', body)
        if not match:
            return False
        body = body[match.end():]
    return True


def windows_gated(item):
    text = source_text(item)
    match = re.search(r'\bfn\s+' + re.escape(item['native']['fn']) + r'\s*\(', text)
    if not match:
        return False
    window = text[:match.start()].splitlines()[-4:]
    return any('#[cfg(windows)]' in line for line in window)


def validate(document, groups=None):
    groups = groups if groups is not None else group_names()
    problems = []
    if document.get('schema') != 'ace-test-policy/1':
        problems.append('schema')
    items = document.get('items')
    if not isinstance(items, list) or len(items) != EXPECTED_COUNT:
        problems.append('item count %s != %d' % (len(items) if isinstance(items, list) else type(items), EXPECTED_COUNT))
        return problems
    ids, natives, leaves = [], [], []
    for item in items:
        kid = item.get('id')
        ids.append(kid)
        native = item.get('native') or {}
        key = (native.get('binary'), native.get('test'))
        natives.append(key)
        leaves.append(item.get('historical_leaf_id'))
        if native.get('fn') != (native.get('test') or '').split('::')[-1]:
            problems.append('%s: fn does not match native test tail' % kid)
        if not (ROOT / native.get('source_file', 'missing')).is_file():
            problems.append('%s: source file missing: %s' % (kid, native.get('source_file')))
            continue
        if not fn_is_declared(item):
            problems.append('%s: fn %s not declared in %s' % (kid, native.get('fn'), native.get('source_file')))
        if item.get('conditions', {}).get('windows_only') != windows_gated(item):
            problems.append('%s: windows_only flag disagrees with source #[cfg(windows)]' % kid)
        if item.get('conditions', {}).get('ignored'):
            problems.append('%s: policy must not carry ignored tests' % kid)
        groups_field = item.get('groups') or {}
        for role in ('responsibility',):
            if groups_field.get(role) not in groups:
                problems.append('%s: %s group not registered: %s' % (kid, role, groups_field.get(role)))
        formal = groups_field.get('formal')
        if formal:
            if formal not in groups:
                problems.append('%s: formal group not registered: %s' % (kid, formal))
            if formal == 'ace-perf' and '::performance::' not in (native.get('test') or ''):
                problems.append('%s: formal=ace-perf but native test is outside ::performance::' % kid)
        elif '::performance::' in (native.get('test') or ''):
            problems.append('%s: native test in ::performance:: but no formal group recorded' % kid)
        replay = item.get('replay', '')
        if replay.split()[0:3] != ['cargo', 't', 'ace-perf']:
            problems.append('%s: replay must explicitly use ace-perf' % kid)
        if native.get('fn') and native['fn'] not in replay:
            problems.append('%s: replay does not navigate to the frozen fn' % kid)
        if native.get('fn') in KNOWN_T07_NAMES:
            problems.append('%s: retired t07_ leaf name used as current fn' % kid)
        old = item.get('historical_name')
        if old and old != native.get('fn') and not item.get('migration'):
            problems.append('%s: renamed without migration evidence' % kid)
    if len(set(ids)) != len(ids):
        problems.append('duplicate item ids')
    if len(set(natives)) != len(natives):
        problems.append('duplicate native identities')
    if len(set(leaves)) != len(leaves):
        problems.append('duplicate historical leaf ids')
    return problems


def load():
    return json.loads(POLICY.read_text(encoding='utf-8'))


class AceTestPolicyTests(unittest.TestCase):

    def test_committed_policy_is_conservation_safe(self):
        problems = validate(load())
        self.assertEqual([], problems)

    def test_thirteen_obligations_conserved(self):
        document = load()
        ids = [item['id'] for item in document['items']]
        self.assertEqual(['K-%02d' % i for i in range(1, 14)], sorted(ids))
        families = {}
        for item in document['items']:
            families[item['family_owner']] = families.get(item['family_owner'], 0) + 1
        self.assertEqual({'av': 4, 'native': 3, 'cte': 4, 'semantics': 1, 'syntax': 1}, families)

    def test_k01_k02_migration_recorded(self):
        document = load()
        by_id = {item['id']: item for item in document['items']}
        for kid in ('K-01', 'K-02'):
            item = by_id[kid]
            self.assertTrue(item['historical_name'].startswith('t07_'))
            self.assertNotEqual(item['historical_name'], item['native']['fn'])
            self.assertIn('5d7afcf78', item['migration'])

    def test_missing_item_rejected(self):
        document = load()
        document['items'] = document['items'][:-1]
        self.assertTrue(validate(document))

    def test_extra_item_rejected(self):
        document = load()
        clone = json.loads(json.dumps(document['items'][0]))
        clone['id'] = 'K-14'
        document['items'].append(clone)
        problems = validate(document)
        self.assertTrue(any('item count' in p for p in problems))

    def test_duplicate_native_rejected(self):
        document = load()
        document['items'][1]['native'] = dict(document['items'][0]['native'])
        self.assertTrue(any('duplicate native' in p for p in validate(document)))

    def test_retired_t07_name_as_current_rejected(self):
        document = load()
        document['items'][0]['native']['fn'] = 't07_v07_512_revisions_bounded_history_stable_watermark'
        document['items'][0]['native']['test'] = (
            'function_incremental::performance::t07_v07_512_revisions_bounded_history_stable_watermark')
        document['items'][0]['replay'] = document['items'][0]['replay'].replace(
            'v07_512_revisions_bounded_history_stable_watermark',
            't07_v07_512_revisions_bounded_history_stable_watermark')
        self.assertTrue(any('t07_' in p for p in validate(document)))

    def test_nonexistent_function_rejected(self):
        document = load()
        document['items'][2]['native']['fn'] = 'resources_ghost_function_missing'
        document['items'][2]['native']['test'] = 'performance::resources_ghost_function_missing'
        document['items'][2]['replay'] = 'cargo t ace-comptime --test-name resources_ghost_function_missing --out <fresh-dir>'
        self.assertTrue(any('not declared' in p for p in validate(document)))

    def test_unregistered_group_rejected(self):
        document = load()
        document['items'][0]['groups']['responsibility'] = 'ace-nonexistent-group'
        self.assertTrue(any('responsibility group not registered' in p for p in validate(document)))

    def test_missing_formal_axis_rejected(self):
        document = load()
        for item in document['items']:
            if '::performance::' in item['native']['test']:
                item['groups']['formal'] = None
                break
        self.assertTrue(any('no formal group recorded' in p for p in validate(document)))

    def test_ordinary_group_cannot_replay_formal_case(self):
        document = load()
        item = document['items'][2]
        item['replay'] = item['replay'].replace('ace-perf', item['groups']['responsibility'])
        self.assertTrue(any('replay must explicitly use ace-perf' in p for p in validate(document)))

    def test_replay_command_matches_frozen_name(self):
        document = load()
        for item in document['items']:
            self.assertIn(item['native']['fn'], item['replay'])
            self.assertEqual(['cargo', 't', 'ace-perf'], item['replay'].split()[:3])


if __name__ == '__main__':
    unittest.main()
