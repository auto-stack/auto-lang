"""Controls for fixed native scope, truthful results, and owned cache cleanup."""
import contextlib
import importlib.util
import io
import json
import subprocess
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('fixed_tests', ROOT / 'scripts/fixed-tests.py')
fixed = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixed)
sys.path.insert(0, str(ROOT / 'scripts/test-governance'))
import build_storage


class FixedGroups(unittest.TestCase):
    def setUp(self):
        self.value, self.groups = fixed.configuration()

    def test_static_list_never_launches_a_process(self):
        with mock.patch.object(fixed, 'capture', side_effect=AssertionError('must not build')), contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(fixed.main(['--list']), 0)
        self.assertIn('parser:', out.getvalue())

    def test_case_owner_rejection_precedes_all_final_output_writes(self):
        with tempfile.TemporaryDirectory() as temp:
            output = Path(temp) / 'protected-output'
            with mock.patch.object(fixed, 'audit_suite_outputs', side_effect=ValueError('protected source domain')) as audit:
                with self.assertRaisesRegex(ValueError, 'protected source domain'):
                    fixed.main(['case-pilot', '--out', str(output)])
                self.assertEqual(audit.call_count, 1)
                self.assertFalse(output.exists())

    def test_case_static_preview_never_writes_without_owner_admission(self):
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(fixed, 'capture', side_effect=AssertionError('static preview must not build')):
            output = Path(temp) / 'output'
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit):
                fixed.main(['case-pilot', '--select-only', '--out', str(output)])
            self.assertFalse(output.exists())

    def test_receipt_write_failure_preserves_previous_committed_result(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / 'results.json'
            fixed.write(path, {'first_attempt': 'FAIL'})
            with mock.patch.object(fixed.os, 'replace', side_effect=OSError('disk full')):
                with self.assertRaises(OSError):
                    fixed.write(path, {'later_attempt': 'PASS'})
            self.assertEqual(json.loads(path.read_text()), {'first_attempt': 'FAIL'})
            self.assertEqual(list(Path(tmp).glob('*.tmp-*')), [])

    def test_selection_does_not_use_a_catalog_or_auto_consumers(self):
        with mock.patch.object(fixed, 'capture', side_effect=AssertionError('selection builds nothing')):
            selected = fixed.select(['parser'], self.value, self.groups)
            self.assertEqual([r['name'] for r in selected], ['parser'])
            self.assertEqual(fixed.select([], self.value, self.groups)[0]['name'], 'daily')
        with self.assertRaises(ValueError):
            fixed.select(['parse_typo'], self.value, self.groups)

    def test_compatible_native_modules_merge_once_not_by_leaf(self):
        groups = fixed.select(['parser', 'vm', 'parser'], self.value, self.groups)
        calls = fixed.invocations(groups)
        self.assertEqual(len(calls), 1)
        self.assertEqual(calls[0]['groups'], ['parser', 'vm'])
        self.assertIn('parser::', calls[0]['expression'])
        self.assertIn('vm::', calls[0]['expression'])

    def test_parser_http_union_keeps_parallel_scheduler_and_http_resource_limit(self):
        group = fixed.invocations([self.groups['parser'], self.groups['http']])[0]
        self.assertEqual(group['groups'], ['parser', 'http'])
        listing = subprocess.CompletedProcess([], 0, json.dumps(self.enumeration()).encode(), b'')
        terminal = {'type': 'test', 'event': 'ok', 'name': 'p::p$parser::decls::a'}
        execution = subprocess.CompletedProcess([], 0, (json.dumps(terminal) + '\n').encode(), b'')
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(fixed, 'invoke', side_effect=[(listing, {}), (execution, {})]) as invoke:
            self.assertEqual(fixed.run_rust(group, Path(temp), {})['outcome'], 'PASS')
            command = invoke.call_args_list[1].args[0]
            self.assertNotIn('--test-threads', command)
        import tomllib
        for config in ('nextest.toml', 'nextest-full.toml', 'nextest-t3.toml'):
            value = tomllib.loads((ROOT / '.config' / config).read_text())
            self.assertEqual(value['test-groups']['fixed-http']['max-threads'], 1)
            self.assertTrue(any(row.get('test-group') == 'fixed-http' and 'http' in row['filter']
                                for row in value['profile']['default']['overrides']))
        self.assertEqual(self.groups['legacy-th']['args'], ['-p', 'auto-lang', '--lib', '--features', 'test-http-e2e'])

    def test_ace_set_is_explicit_and_separate_from_daily(self):
        selected = fixed.select(['ace'], self.value, self.groups)
        self.assertEqual([g['name'] for g in selected], ['ace-workspace', 'ace-adapters'])
        self.assertEqual(selected[0]['workspace'], 'experimental/ace')
        self.assertNotIn('daily', [g['name'] for g in selected])
        self.assertNotIn('ace-workspace', [g['name'] for g in fixed.select([], self.value, self.groups)])

    def test_incompatible_packages_remain_separate(self):
        self.assertEqual(len(fixed.invocations(fixed.select(['lexer', 'parser'], self.value, self.groups))), 2)

    def test_all_is_explicit_and_contains_full_packages_and_ace(self):
        names = {r['name'] for r in fixed.select(['all'], self.value, self.groups)}
        self.assertIn('auto-lang-core', names)
        self.assertIn('auto-lex', names)
        self.assertIn('ace-host', names)
        self.assertNotIn('daily', names)
        self.assertNotIn('parser', names)  # covered by full package; avoid repeated execution
        with self.assertRaises(ValueError):
            fixed.select(['all', 'parser'], self.value, self.groups)

    def test_namespace_descendants_not_only_tests_submodule(self):
        expression = self.groups['parser']['expression']
        self.assertIn('^parser::', expression)
        self.assertNotIn('^parser::tests::', expression)

    def test_shared_target_does_not_redirect_nested_shim_builds(self):
        group = fixed.invocations([self.groups['ace-native']])[0]
        args = fixed.nextest_args(group)
        self.assertEqual(args[args.index('--target-dir') + 1], str(ROOT / 'experimental/ace/target'))
        self.assertFalse((ROOT / 'experimental/ace/crates/native/.cargo/config.toml').exists())

    def test_ace_declaration_in_legacy_group_fails_before_execution(self):
        doc = self.enumeration()
        suite = doc['rust-suites']['p']
        suite['testcases'] = {'test_cases::ac_mn::collections::controls::unexpected': {
            'ignored': False, 'filter-match': {'status': 'matches'}}}
        listing = subprocess.CompletedProcess([], 0, json.dumps(doc).encode(), b'')
        group = fixed.invocations([self.groups['daily']])[0]
        with tempfile.TemporaryDirectory() as temp, mock.patch.object(fixed, 'invoke', return_value=(listing, {})) as invoke:
            with self.assertRaisesRegex(ValueError, 'leaked into a legacy group'):
                fixed.run_rust(group, Path(temp), {})
            self.assertEqual(invoke.call_count, 1)  # native list only, no run/tool preparation

    def test_external_native_cwd_is_rejected(self):
        doc = self.enumeration()
        doc['rust-suites']['p']['cwd'] = str(ROOT.parent / 'foreign')
        with self.assertRaisesRegex(ValueError, 'cwd escapes'):
            fixed.enumerate_rust(doc)

    def test_daily_keeps_original_filter_but_named_modules_are_complete(self):
        daily = fixed.invocations([self.groups['daily']])[0]
        parser = fixed.invocations([self.groups['parser']])[0]
        self.assertNotIn('--ignore-default-filter', fixed.nextest_args(daily))
        self.assertIn('--ignore-default-filter', fixed.nextest_args(parser))

    def test_non_ace_scope_has_no_experimental_target_or_tools(self):
        for name in ('daily', 'parser', 'vm', 'a2r', 'lexer'):
            group = self.groups[name]
            self.assertEqual(group['domain'], 'legacy')
            self.assertEqual(group['tools'], [])
            self.assertNotIn('test-ace', ','.join(group['args']))

    @staticmethod
    def enumeration():
        return {'rust-suites': {'p': {'package-name': 'p', 'binary-name': 'p', 'status': 'listed', 'testcases': {
            'parser::decls::a': {'ignored': False, 'filter-match': {'status': 'matches'}},
            'parser::tests::manual': {'ignored': True, 'filter-match': {'status': 'matches'}},
            'vm::b': {'ignored': False, 'filter-match': {'status': 'mismatch'}},
        }}}}

    def test_native_identity_and_ignore_are_not_confused_with_passing(self):
        active, excluded = fixed.enumerate_rust(self.enumeration())
        self.assertEqual(list(active), ['p::p$parser::decls::a'])
        self.assertEqual(len(excluded), 1)
        active, excluded = fixed.enumerate_rust(self.enumeration(), ignored=True)
        self.assertEqual(list(active), ['p::p$parser::tests::manual'])
        self.assertEqual(len(excluded), 1)

    def test_results_require_real_unique_terminal_per_selected_identity(self):
        expected = {'p::p$a': {}, 'p::p$b': {}}
        def raw(*rows):
            return ('\n'.join(json.dumps(r) for r in rows)).encode()
        good = {'type': 'test', 'event': 'ok', 'name': 'p::p$a'}
        self.assertTrue(fixed.terminal_results(raw(good), expected)['errors'])
        with self.assertRaises(ValueError):
            fixed.terminal_results(raw(good, good), expected)
        with self.assertRaises(ValueError):
            fixed.terminal_results(raw(dict(good, name='extra')), expected)
        red = dict(good, name='p::p$b', event='failed')
        self.assertTrue(fixed.terminal_results(raw(good, red), expected)['errors'])
        self.assertFalse(fixed.terminal_results(raw(good, dict(red, event='ok')), expected)['errors'])
        excluded = dict(good, name='p::p$filtered', event='ignored')
        complete = raw(dict(excluded, event='started'), good, dict(red, event='ok'), excluded)
        self.assertFalse(fixed.terminal_results(complete, expected, {'p::p$filtered'})['errors'])
        with self.assertRaises(ValueError):
            fixed.terminal_results(raw(dict(excluded, event='ok')), expected, {'p::p$filtered'})

    def test_legacy_trans_and_book_keep_the_entire_original_native_configuration(self):
        for name, feature in [('legacy-tt', 'test-trans'), ('legacy-tb', 'test-book')]:
            group = self.groups[name]
            self.assertEqual(group['expression'], 'all()')
            self.assertIn(feature, ','.join(group['args']))
            self.assertTrue(group['respect_default_filter'])
            self.assertNotIn('--ignore-default-filter', fixed.nextest_args(fixed.invocations([group])[0]))

    def test_invalid_navigation_cannot_inject_a_filter(self):
        with self.assertRaises(ValueError):
            fixed.invocations([self.groups['parser']], 'x) or all()')

    def test_milestone_not_implicitly_enabled_even_for_ignored_replay(self):
        with mock.patch.dict('os.environ', {}, clear=True):
            expression = fixed.invocations([self.groups['aavm']])[0]['expression']
        self.assertIn('not test(=tests::aavm2_t3::', expression)

    def test_explicit_ignored_replay_cannot_disable_unrelated_defaults(self):
        with self.assertRaises(SystemExit), contextlib.redirect_stderr(io.StringIO()):
            fixed.main(['parser', '--ignored', '--select-only'])

    def test_self_hosting_declarations_keep_code_but_all_are_ignored(self):
        import re
        for path in (ROOT / 'crates/auto-lang/src/tests').glob('aavm*.rs'):
            text = path.read_text(encoding='utf-8')
            for match in re.finditer(r'#\[test\][\s\S]*?(?=\bfn\s)', text):
                before = text[max(0, match.start()-160):match.start()]
                self.assertIn('ignore', before + match[0], str(path))


class CacheOwnership(unittest.TestCase):
    def test_process_and_partial_delete_failures_never_report_success(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            cache = root / 'experimental/fixture/target'
            cache.mkdir(parents=True)
            (cache.parent / 'Cargo.toml').write_text('[workspace]\n')
            item = cache / 'object.o'
            item.write_bytes(b'cache')
            with mock.patch.object(build_storage, 'ROOT', root):
                with mock.patch.object(build_storage, 'refuse_active_processes', side_effect=ValueError('live process')):
                    with contextlib.redirect_stdout(io.StringIO()), self.assertRaisesRegex(ValueError, 'live process'):
                        build_storage.main([str(cache), '--apply', '--idle-confirmed'])
                self.assertTrue(item.exists())
                def partial_failure(path):
                    item.unlink()
                    raise OSError('injected mid-delete failure')
                with mock.patch.object(build_storage, 'refuse_active_processes'), mock.patch.object(build_storage.shutil, 'rmtree', side_effect=partial_failure):
                    with contextlib.redirect_stdout(io.StringIO()) as output, self.assertRaises(OSError):
                        build_storage.main([str(cache), '--apply', '--idle-confirmed'])
                    self.assertNotIn('file_data_bytes_reclaimed', output.getvalue())
                self.assertTrue(cache.exists())

    def test_shared_inode_bytes_are_not_claimed_as_reclaimed(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            cache = root / 'target/test-tools/regular'
            cache.mkdir(parents=True)
            original = root / 'protected-original'
            original.write_bytes(b'protected')
            import os
            os.link(original, cache / 'one')
            os.link(original, cache / 'two')
            with mock.patch.object(build_storage, 'ROOT', root):
                summary = build_storage.inspect(cache)
                self.assertEqual(summary['unique_file_bytes'], 9)
                self.assertEqual(summary['reclaimable_file_data_bytes'], 0)
                with mock.patch.object(build_storage, 'reparse', side_effect=lambda p: p == cache.parent):
                    with self.assertRaisesRegex(ValueError, 'reparse point/ancestor'):
                        build_storage.inspect(cache)
            self.assertEqual(original.read_bytes(), b'protected')

    def test_owned_preview_apply_and_protected_states(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            cache = root / 'experimental/fixture/target'
            cache.mkdir(parents=True)
            (cache.parent / 'Cargo.toml').write_text('[workspace]\n')
            (cache / 'object.o').write_bytes(b'abcd')
            with mock.patch.object(build_storage, 'ROOT', root):
                self.assertEqual(build_storage.inspect(cache)['unique_file_bytes'], 4)
                with self.assertRaises(ValueError):
                    build_storage.inspect(root)
                (cache / 'report.json').write_text('{}')
                with self.assertRaises(ValueError):
                    build_storage.inspect(cache)
                (cache / 'report.json').unlink()
                (root / 'target/test-groups/active').mkdir(parents=True)
                with self.assertRaises(ValueError):
                    build_storage.inspect(cache)
                (root / 'target/test-groups/active').rmdir()
                with mock.patch.object(build_storage, 'refuse_active_processes'), contextlib.redirect_stdout(io.StringIO()):
                    self.assertEqual(build_storage.main([str(cache), '--apply', '--idle-confirmed']), 0)
                self.assertFalse(cache.exists())

    def test_retired_cache_requires_ancestor_manifest_ownership(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            def git(*argv, input=None):
                return subprocess.check_output(
                    ['git', '-C', str(root), '-c', 'user.name=Cache control',
                     '-c', 'user.email=cache@example.invalid', *argv],
                    input=input, text=True, stderr=subprocess.PIPE).strip()
            git('init', '-q')
            member = root / 'experimental/retired'
            member.mkdir(parents=True)
            manifest = member / 'Cargo.toml'
            manifest.write_text('[workspace]\n')
            git('add', 'experimental/retired/Cargo.toml')
            git('commit', '-qm', 'owned historical workspace')
            owner = git('rev-parse', 'HEAD')
            manifest.unlink()
            cache = member / 'target'
            cache.mkdir()
            (cache / 'rebuildable.o').write_bytes(b'owned')
            unrelated = git('commit-tree', git('mktree', input=''), '-m', 'unrelated root')
            with mock.patch.object(build_storage, 'ROOT', root):
                with self.assertRaises(ValueError):
                    build_storage.inspect(cache)
                self.assertEqual(build_storage.inspect(cache, owner)['unique_file_bytes'], 5)
                with self.assertRaises(subprocess.CalledProcessError):
                    build_storage.inspect(cache, unrelated)
            self.assertEqual((cache / 'rebuildable.o').read_bytes(), b'owned')


if __name__ == '__main__':
    unittest.main()
