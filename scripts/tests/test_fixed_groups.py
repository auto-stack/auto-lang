"""Controls for fixed native scope, truthful results, and owned cache cleanup."""
import contextlib
import importlib.util
import io
import json
import os
import shutil
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
    def test_cleanup_foreign_compiler_requires_observed_cwd_and_explicit_target(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                scope = build_storage.foreign_compiler_scope(foreign, ['cargo', 'test'], {'CARGO_TARGET_DIR': str(foreign / 'target')})
                self.assertEqual(scope['target_dir'], str(foreign / 'target'))
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo', 'test'], {})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(root, ['cargo', 'test'], {'CARGO_TARGET_DIR': str(foreign / 'target')})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', str(root / 'source.rs')], {'CARGO_TARGET_DIR': str(foreign / 'target')})

    def test_cleanup_cli_target_override_cannot_hide_an_owned_cache(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', '--target-dir', str(root / 'target')], {'CARGO_TARGET_DIR': str(foreign / 'target')})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', '--target-dir=../ours/target'], {})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', '--target-dir', str(foreign / 'explicit')], {'CARGO_TARGET_DIR': str(root / 'target')})
                scope = build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', '--target-dir', str(foreign / 'explicit')], {'CARGO_TARGET_DIR': str(foreign / 'inherited')})
                self.assertEqual(scope['target_dir'], str(foreign / 'explicit'))

    def test_cleanup_resolves_relative_manifest_source_and_nested_tool_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                for argv in (['cargo','test','--manifest-path','../ours/Cargo.toml'],
                             ['cargo','test','--manifest-path=../ours/Cargo.toml'],
                             ['rustc','../ours/target/generated.rs']):
                    with self.assertRaises(ValueError):
                        build_storage.foreign_compiler_scope(foreign, argv, {'CARGO_TARGET_DIR': str(foreign / 'target')})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','build'], {'CARGO_TARGET_DIR': str(foreign / 'target'), 'RUSTC_WRAPPER':'../ours/target/wrapper.exe'})

    def test_cleanup_does_not_treat_program_arguments_as_cargo_target_overrides(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','test','--','--target-dir',str(foreign / 'fake')], {'CARGO_TARGET_DIR':str(root / 'target')})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','test','--','--target-dir',str(foreign / 'fake')], {})
                scope = build_storage.foreign_compiler_scope(foreign, ['cargo','test','--','--target-dir',str(foreign / 'fake')], {'CARGO_TARGET_DIR':str(foreign / 'actual')})
                self.assertEqual(scope['target_dir'], str(foreign / 'actual'))

    def test_cleanup_rejects_opaque_response_files_without_reading_them(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                for value in ('@../ours/build.rsp', '@foreign.rsp', 'link-arg=@foreign.rsp'):
                    with self.assertRaises(ValueError):
                        build_storage.foreign_compiler_scope(foreign, ['rustc',value], {'CARGO_TARGET_DIR':str(foreign / 'target')})

    def test_cleanup_rejects_opaque_inherited_flags_and_linker_references(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                for name in ('RUSTFLAGS','CARGO_ENCODED_RUSTFLAGS','RUSTDOCFLAGS','CARGO_BUILD_RUSTFLAGS','CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS','CFLAGS','LDFLAGS','CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUNNER'):
                    with self.assertRaises(ValueError):
                        build_storage.foreign_compiler_scope(foreign, ['cargo','build'], {'CARGO_TARGET_DIR':str(foreign / 'target'),name:'-L ../ours/target'})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','build'], {'CARGO_TARGET_DIR':str(foreign / 'target'),'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER':'../ours/target/linker.exe'})
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','build'], {'CARGO_TARGET_DIR':str(foreign / 'target'),'CARGO_BUILD_RUSTC_WRAPPER':'../ours/target/wrapper.exe'})

    @unittest.skipUnless(sys.platform == 'win32', 'Windows drive-relative path contract')
    def test_cleanup_rejects_drive_relative_paths(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                with self.assertRaises(ValueError):
                    build_storage.foreign_compiler_scope(foreign, ['cargo','build'], {'CARGO_TARGET_DIR':foreign.drive + 'relative-cache'})

    @unittest.skipUnless(sys.platform == 'win32', 'Windows drive-relative tool contract')
    def test_cleanup_rejects_drive_relative_tool_environment(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                for name in ('RUSTC', 'RUSTDOC', 'RUSTC_WRAPPER', 'RUSTC_WORKSPACE_WRAPPER',
                             'CC', 'CXX', 'AR', 'LD', 'CARGO_BUILD_RUSTC_WRAPPER',
                             'CARGO_BUILD_RUSTC', 'CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_LINKER'):
                    with self.subTest(name=name), self.assertRaisesRegex(ValueError, 'drive-relative'):
                        build_storage.foreign_compiler_scope(foreign, ['cargo', 'build'],
                            {'CARGO_TARGET_DIR': str(foreign / 'target'), name: foreign.drive + 'compiler.exe'})

    @unittest.skipUnless(sys.platform == 'win32', 'Windows drive-relative argv contract')
    def test_cleanup_rejects_drive_relative_extensionless_argv(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp) / 'ours'; root.mkdir()
            foreign = Path(temp) / 'foreign'; foreign.mkdir()
            with mock.patch.object(build_storage, 'ROOT', root):
                for value in (foreign.drive + 'compiler', 'key=' + foreign.drive + 'resource'):
                    with self.subTest(value=value), self.assertRaisesRegex(ValueError, 'drive-relative'):
                        build_storage.foreign_compiler_scope(foreign, ['cargo', 'build', value],
                            {'CARGO_TARGET_DIR': str(foreign / 'target')})

    def setUp(self):
        self.value, self.groups = fixed.configuration()

    def test_static_list_never_launches_a_process(self):
        with mock.patch.object(fixed, 'capture', side_effect=AssertionError('must not build')), contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(fixed.main(['--list']), 0)
        self.assertIn('parser:', out.getvalue())

    def test_case_owner_rejection_precedes_all_final_output_writes(self):
        with tempfile.TemporaryDirectory() as temp,                 mock.patch.object(fixed, 'execution_lease', lambda: contextlib.nullcontext()):
            output = Path(temp) / 'protected-output'
            with mock.patch.object(fixed, 'audit_suite_outputs', side_effect=ValueError('protected source domain')) as audit, \
                    mock.patch.object(fixed.tempfile, 'mkdtemp', return_value=str(Path(temp) / 'diagnostics')) as admission_scope:
                with self.assertRaisesRegex(ValueError, 'protected source domain'):
                    fixed.main(['case-pilot', '--out', str(output)])
                self.assertEqual(audit.call_count, 1)
                self.assertFalse(output.exists())
                requested_home = admission_scope.call_args.kwargs.get('dir') or tempfile.gettempdir()
                self.assertEqual(Path(requested_home).resolve(), Path(tempfile.gettempdir()).resolve())

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

    def test_ace_set_is_explicit_and_separate_from_daily(self):
        selected = fixed.select(['ace'], self.value, self.groups)
        self.assertEqual([g['name'] for g in selected], ['ace-workspace', 'ace-adapters', 'ace-batch'])
        self.assertEqual(selected[0]['workspace'], 'experimental/ace')
        batch = fixed.invocations([selected[2]])[0]
        argv = fixed.nextest_args(batch)
        self.assertTrue(argv[argv.index('--target-dir') + 1].endswith('target-batch'))
        self.assertIn('test-case-batch', batch['args'])
        self.assertIn('batch-av', batch['tools'])
        self.assertNotIn('daily', [g['name'] for g in selected])
        self.assertNotIn('ace-workspace', [g['name'] for g in fixed.select([], self.value, self.groups)])

    def test_incompatible_packages_remain_separate(self):
        self.assertEqual(len(fixed.invocations(fixed.select(['lexer', 'parser'], self.value, self.groups))), 2)

    def test_all_is_explicit_language_only(self):
        names = {r['name'] for r in fixed.select(['all'], self.value, self.groups)}
        self.assertIn('auto-lang-core', names)
        self.assertIn('auto-lex', names)
        # PLAN-898 user ruling: `all` is the language-side union only; ACE
        # domain groups (incl. ace-host) run explicitly via `cargo t ace`.
        self.assertNotIn('ace-host', names)
        self.assertFalse(any(n.startswith('ace-') or n.startswith('case-ace-')
                             for n in names) or any(
            self.groups[n].get('domain') == 'ace' for n in names))
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

    def test_outer_target_override_does_not_leak_to_private_build_consumers(self):
        import os
        target = ROOT / 'target/isolated-outer-control'
        group = fixed.invocations([self.groups['parser']])[0]
        listing = subprocess.CompletedProcess([], 0, json.dumps(self.enumeration()).encode(), b'')
        terminal = {'type': 'test', 'event': 'ok', 'name': 'p::p$parser::decls::a'}
        execution = subprocess.CompletedProcess([], 0, (json.dumps(terminal) + '\n').encode(), b'')
        with mock.patch.dict(os.environ, {'CARGO_TARGET_DIR': str(target)}), tempfile.TemporaryDirectory() as temp:
            with mock.patch.object(fixed, 'invoke', side_effect=[(listing, {}), (execution, {})]) as invoke:
                self.assertEqual(fixed.run_rust(group, Path(temp), dict(os.environ))['outcome'], 'PASS')
                for call in invoke.call_args_list:
                    argv, _, env = call.args[:3]
                    self.assertEqual(argv[argv.index('--target-dir') + 1], str(target.resolve()))
                    self.assertNotIn('CARGO_TARGET_DIR', env)
            for name, directory in [('ace-native', 'target'), ('ace-batch', 'target-batch')]:
                argv = fixed.nextest_args(fixed.invocations([self.groups[name]])[0])
                self.assertEqual(argv.count('--target-dir'), 1)
                self.assertEqual(argv[argv.index('--target-dir') + 1], str(ROOT / 'experimental/ace' / directory))

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

    def test_legacy_compat_groups_are_retired(self):
        # PLAN-898: tm/tier compat groups and aliases are deleted entirely.
        self.assertFalse({'legacy-tv', 'legacy-tt', 'legacy-tb', 'legacy-th'} & set(self.groups))

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


class IncrementalGovernance(unittest.TestCase):
    """PLAN-1001 V-01..V-05: inventory / preview / apply-manifest controls."""

    def setUp(self):
        self._temp = tempfile.TemporaryDirectory()
        self.base = Path(self._temp.name)
        self.root = self.base / 'ours'
        (self.root / 'target').mkdir(parents=True)
        # Fail-closed retention registry for the mocked ROOT.
        registry = self.root / '.retention'
        registry.mkdir()
        (registry / 'policy.json').write_text(json.dumps({'policy': 'test', 'scopes': [], 'holds': []}), encoding='utf-8')

    def tearDown(self):
        self._temp.cleanup()

    # -- fixtures ----------------------------------------------------

    @staticmethod
    def make_unit(path, files=3, content=b'x' * 4096):
        path.mkdir(parents=True)
        for index in range(files):
            (path / ('sess%d-1a2b3c4d5e6f' % index)).write_bytes(content)
        (path.parent / '.cargo-lock').write_bytes(b'')
        return path

    def write_policy(self, owners, watermarks=None):
        path = self.base / 'policy.json'
        path.write_text(json.dumps({
            'schema': 'incremental-cache-policy/v1',
            'owners': [{'owner': owner, 'root': str(root), 'rationale': 'test',
                        'foreign': foreign} for owner, root, foreign in owners],
            'watermarks': watermarks or {},
        }, indent=1), encoding='utf-8')
        return path

    def preview(self, policy, output=None):
        output = output or (self.base / 'preview.json')
        code = build_storage.main(['--incremental', '--preview', '--policy', str(policy),
                                   '--output', str(output)])
        self.assertEqual(code, 0)
        return json.loads(output.read_text(encoding='utf-8')), output

    def apply(self, manifest, policy, output=None):
        output = output or (self.base / 'apply.json')
        code = build_storage.main(['--incremental', '--apply-manifest', str(manifest),
                                   '--policy', str(policy), '--idle-confirmed',
                                   '--output', str(output)])
        return code, json.loads(output.read_text(encoding='utf-8'))

    @staticmethod
    def tree_state(path):
        state = {}
        for folder, _dirs, names in os.walk(path):
            for name in names:
                child = Path(folder) / name
                state[str(child)] = child.stat().st_size
        return state

    # -- V-01 preview is read-only; admission and manifest honesty ----

    def test_incremental_preview_is_read_only_with_positive_control(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        before = self.tree_state(self.root / 'target')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            manifest, _ = self.preview(policy)
        self.assertEqual(len(manifest['units']), 1)
        self.assertEqual(manifest['units'][0]['status'], 'eligible-idle')
        self.assertEqual(manifest['units'][0]['files'], 3)
        self.assertTrue(manifest['zero_deletion'])
        self.assertEqual(self.tree_state(self.root / 'target'), before)
        self.assertTrue(unit.is_dir())

    def test_incremental_refuses_unregistered_foreign_root(self):
        foreign = self.base / 'foreign'
        foreign.mkdir()
        (foreign / 'Cargo.toml').write_text('[workspace]\n', encoding='utf-8')
        self.make_unit(foreign / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            with self.assertRaises(ValueError) as ctx:
                build_storage.main(['--incremental', '--inventory', str(foreign / 'target'),
                                    '--output', str(self.base / 'inv.json')])
        self.assertIn('not a registered incremental-cache owner root', str(ctx.exception))

    def test_incremental_foreign_root_needs_dual_proof(self):
        foreign = self.base / 'foreign'
        foreign.mkdir()
        (foreign / 'Cargo.toml').write_text('[workspace]\n', encoding='utf-8')
        self.make_unit(foreign / 'target' / 'debug' / 'incremental')
        with_repo = self.write_policy([('man', foreign / 'target', True)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(with_repo)
        self.assertEqual([u['status'] for u in manifest['units']], ['eligible-idle'])
        self.assertEqual(manifest['units'][0]['owner'], 'man')
        # Registration without a workspace manifest is refused.
        (foreign / 'Cargo.toml').unlink()
        policy = self.write_policy([('man', foreign / 'target', True)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        self.assertEqual(manifest['units'], [])
        self.assertIn('not a registered incremental-cache owner root',
                      manifest['refused'][0]['reason'])

    def test_incremental_apply_rejects_hand_written_manifest_paths(self):
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        victim = self.make_unit(self.base / 'victim' / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        manifest['units'].append({'path': str(victim), 'status': 'eligible-idle',
                                  'tree_digest': '0' * 64, 'files': 3})
        tampered = self.base / 'tampered.json'
        tampered.write_text(json.dumps(manifest), encoding='utf-8')
        before = self.tree_state(self.root / 'target')
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(tampered, policy)
        self.assertIn('no longer an eligible unit', str(ctx.exception))
        self.assertEqual(self.tree_state(self.root / 'target'), before)
        self.assertTrue(victim.is_dir())

    def test_incremental_policy_digest_binding(self):
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        policy2 = self.write_policy([('ours-renamed', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(path, policy2)
        self.assertIn('policy does not match the snapshot policy', str(ctx.exception))

    # -- V-02 protections refuse; protected trees unchanged ----------

    def test_incremental_live_lease_refused_dead_lease_stale(self):
        ctx_root = self.root / 'target' / 'generated-consumers' / 'abcd1234'
        ctx_root.mkdir(parents=True)
        (ctx_root / 'context.json').write_text(json.dumps({
            'schema': 'generated-build-context/v1', 'context_id': 'abcd1234',
            'producer_root': str(self.root.resolve())}), encoding='utf-8')
        unit = self.make_unit(ctx_root / 'debug' / 'incremental')
        lease = ctx_root / '.lease'
        lease.mkdir()
        (lease / 'run.json').write_text(json.dumps({'pid': 424242, 'purpose': 'test'}), encoding='utf-8')
        policy = self.write_policy([('gen', ctx_root, False)])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'process_alive', return_value=True):
            manifest, _ = self.preview(policy)
        self.assertEqual(manifest['units'], [])
        self.assertIn('live build/run lease', manifest['refused'][0]['reason'])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'process_alive', return_value=False):
            manifest, _ = self.preview(policy)
        self.assertEqual(manifest['units'][0]['status'], 'eligible-idle')
        self.assertEqual(manifest['units'][0]['stale_leases_ignored'], [str(lease / 'run.json')])
        self.assertTrue(unit.is_dir())

    def test_incremental_evidence_and_reparse_refused(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        (unit / 'results.json').write_text('{}', encoding='utf-8')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        self.assertEqual(manifest['units'][0]['status'], 'protected')
        self.assertIn('evidence mixed into incremental unit', manifest['units'][0]['refusal_reason'])
        (unit / 'results.json').unlink()
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'reparse', side_effect=lambda p: p.name == 'sess0-1a2b3c4d5e6f'):
            manifest, _ = self.preview(policy)
        self.assertIn('reparse point inside incremental unit', manifest['units'][0]['refusal_reason'])
        self.assertTrue(unit.is_dir())

    def test_incremental_retention_hold_blocks_apply(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        registry = self.root / '.retention' / 'policy.json'
        registry.write_text(json.dumps({'policy': 'test', 'scopes': [], 'holds': [
            {'path': str(unit), 'owner': 'user', 'reason': 'audit'}]}), encoding='utf-8')
        before = self.tree_state(unit)
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(path, policy)
        self.assertIn('retention hold covers this unit', str(ctx.exception))
        self.assertEqual(self.tree_state(unit), before)
        registry.unlink()
        with mock.patch.object(build_storage, 'ROOT', self.root):
            with self.assertRaises(ValueError) as ctx:
                build_storage.retention_hold_paths()
        self.assertIn('retention registry missing', str(ctx.exception))

    def test_incremental_missing_registry_fails_closed(self):
        (self.root / '.retention' / 'policy.json').unlink()
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(path, policy)
        self.assertIn('retention registry missing', str(ctx.exception))

    def test_incremental_apply_refuses_without_idle_confirmation(self):
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        with mock.patch.object(build_storage, 'ROOT', self.root):
            with self.assertRaises(SystemExit):
                build_storage.main(['--incremental', '--apply-manifest', str(path),
                                    '--policy', str(policy), '--output', str(self.base / 'a.json')])

    # -- V-03 real mutex, identity change, failure receipts ----------

    def test_incremental_apply_blocks_when_cargo_lock_held(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        lock = self.root / 'target' / 'debug' / '.cargo-lock'
        if os.name == 'nt':
            import msvcrt
            held = os.open(str(lock), os.O_RDWR)
            os.lseek(held, 0, os.SEEK_SET)
            msvcrt.locking(held, msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            held = os.open(str(lock), os.O_RDWR)
            fcntl.flock(held, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            with mock.patch.object(build_storage, 'ROOT', self.root), \
                    mock.patch.object(build_storage, 'refuse_active_processes'):
                with self.assertRaises(ValueError) as ctx:
                    self.apply(path, policy)
            self.assertIn('cargo lock held by another producer', str(ctx.exception))
        finally:
            if os.name == 'nt':
                import msvcrt
                os.lseek(held, 0, os.SEEK_SET)
                msvcrt.locking(held, msvcrt.LK_UNLCK, 1)
            else:
                import fcntl
                fcntl.flock(held, fcntl.LOCK_UN)
            os.close(held)
        self.assertTrue(unit.is_dir())

    def test_incremental_apply_identity_change_refused(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        (unit / 'late-arrival').write_bytes(b'new')
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(path, policy)
        self.assertIn('identity changed since the preview', str(ctx.exception))
        self.assertTrue(unit.is_dir())

    def test_incremental_apply_partial_failure_receipts(self):
        first = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        second = self.make_unit(self.root / 'target' / 'release' / 'incremental', files=2)
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        self.assertEqual(len(manifest['units']), 2)
        real_rmtree = shutil.rmtree

        def flaky_rmtree(target, *args, **kwargs):
            if str(second) in str(target):
                raise OSError('simulated mid-delete failure')
            return real_rmtree(target, *args, **kwargs)

        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'), \
                mock.patch.object(build_storage.shutil, 'rmtree', side_effect=flaky_rmtree):
            code, receipt = self.apply(path, policy)
        self.assertEqual(code, 3)
        self.assertEqual([d['path'] for d in receipt['deleted']], [str(first)])
        self.assertEqual([f['path'] for f in receipt['failed']], [str(second)])
        self.assertFalse(first.exists())
        self.assertTrue(second.is_dir())

    def test_incremental_apply_happy_path_deletes_only_units(self):
        unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        deps = self.root / 'target' / 'debug' / 'deps'
        deps.mkdir()
        (deps / 'libkeep.rlib').write_bytes(b'keep me')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            code, receipt = self.apply(path, policy)
        self.assertEqual(code, 0)
        self.assertFalse(unit.exists())
        self.assertTrue(deps.is_dir())
        self.assertEqual((deps / 'libkeep.rlib').read_bytes(), b'keep me')
        self.assertTrue((self.root / 'target' / 'debug' / '.cargo-lock').is_file())
        # Lock released: a second exclusive lock can be taken again.
        fd = os.open(str(self.root / 'target' / 'debug' / '.cargo-lock'), os.O_RDWR)
        build_storage._lock_byte(fd)
        build_storage._unlock_byte(fd)
        os.close(fd)

    # -- V-04 nesting, hard links, unknowns, shapes -------------------

    def test_incremental_nested_roots_and_hardlink_dedup(self):
        main_unit = self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        ctx_root = self.root / 'target' / 'generated-consumers' / 'abcd1234'
        ctx_root.mkdir(parents=True)
        (ctx_root / 'context.json').write_text(json.dumps({
            'schema': 'generated-build-context/v1', 'context_id': 'abcd1234',
            'producer_root': str(self.root.resolve())}), encoding='utf-8')
        gen_unit = self.make_unit(ctx_root / 'debug' / 'incremental', files=2)
        os.link(main_unit / 'sess0-1a2b3c4d5e6f', gen_unit / 'shared-with-main')
        (self.root / 'target' / 'test-tools' / 'regular').mkdir(parents=True)
        (self.root / 'target' / 'test-tools' / 'regular' / 'marker').write_bytes(b't')
        policy = self.write_policy([('ours', self.root / 'target', False), ('gen', ctx_root, False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        by_root = {u['root']: u for u in manifest['units']}
        self.assertEqual({str(self.root / 'target'), str(ctx_root)}, set(by_root))
        # The nested generated-consumers root is not double-counted under
        # the main root, and test-tools is pruned from the walk.
        self.assertEqual(sum(1 for u in manifest['units'] if u['root'] == str(self.root / 'target')), 1)
        self.assertEqual(manifest['shared_file_ids_across_units'], 1)
        gen_row = by_root[str(ctx_root)]
        # The shared file is not reclaimable by deleting the generated unit.
        self.assertLess(gen_row['reclaimable_allocated_upper_bound'], gen_row['allocated_bytes'])

    def test_incremental_unknown_allocation_demotes_unit(self):
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'allocated_bytes', return_value=None):
            manifest, _ = self.preview(policy)
        self.assertEqual(manifest['units'][0]['status'], 'unknown')
        self.assertGreater(manifest['units'][0]['unknown_alloc_files'], 0)
        self.assertNotEqual(manifest['units'][0]['logical_bytes'], manifest['units'][0]['allocated_bytes'])

    def test_incremental_foreign_generated_context_admission(self):
        # A registered foreign workspace's generated-consumer context is
        # admitted through its context.json producer proof (dual proof),
        # from a checkout other than the producer's own.
        foreign_repo = self.base / 'langmain'
        foreign_repo.mkdir()
        (foreign_repo / 'Cargo.toml').write_text('[workspace]\n', encoding='utf-8')
        ctx_root = foreign_repo / 'target' / 'generated-consumers' / 'abcd1234'
        ctx_root.mkdir(parents=True)
        (ctx_root / 'context.json').write_text(json.dumps({
            'schema': 'generated-build-context/v1', 'context_id': 'abcd1234',
            'producer_root': str(foreign_repo.resolve())}), encoding='utf-8')
        self.make_unit(foreign_repo / 'target' / 'debug' / 'incremental')
        self.make_unit(ctx_root / 'debug' / 'incremental')
        policy = self.write_policy([('lang-main', foreign_repo / 'target', True),
                                    ('gen-ctx', ctx_root, True)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        by_root = {u['root']: u for u in manifest['units']}
        self.assertIn(str(foreign_repo / 'target'), by_root)
        self.assertIn(str(ctx_root), by_root)
        self.assertEqual(by_root[str(ctx_root)]['kind'], 'foreign-generated:lang-main')
        # A context.json lying about its producer is refused.
        (ctx_root / 'context.json').write_text(json.dumps({
            'schema': 'generated-build-context/v1', 'context_id': 'abcd1234',
            'producer_root': str(self.base / 'someone-else')}), encoding='utf-8')
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        self.assertNotIn(str(ctx_root), {u['root'] for u in manifest['units']})
        self.assertIn('does not prove its producer',
                      next(r['reason'] for r in manifest['refused'] if r['root'] == str(ctx_root)))

    def test_incremental_triple_profile_unit_shape(self):
        self.make_unit(self.root / 'target' / 'x86_64-pc-windows-msvc' / 'runtime-release' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        unit = manifest['units'][0]
        self.assertEqual(unit['profile'], 'runtime-release')
        self.assertEqual(unit['triple'], 'x86_64-pc-windows-msvc')

    # -- V-05 idempotence, watermarks, legacy CLI ---------------------

    def test_incremental_consumed_manifest_cannot_reapply(self):
        self.make_unit(self.root / 'target' / 'debug' / 'incremental')
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            code, _ = self.apply(path, policy)
        self.assertEqual(code, 0)
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            with self.assertRaises(ValueError) as ctx:
                self.apply(path, policy)
        self.assertIn('no longer an eligible unit', str(ctx.exception))

    def test_incremental_watermark_shortfall_reported(self):
        # Shortfall is only real when protected bytes alone already exceed
        # the ceiling: eligible reclamation alone cannot cross protection.
        eligible_row = self.make_unit(self.root / 'target' / 'debug' / 'incremental', files=2, content=b'y' * 8192)
        protected_unit = self.make_unit(self.root / 'target' / 'release' / 'incremental', files=2, content=b'y' * 8192)
        (protected_unit / 'results.json').write_text('{}', encoding='utf-8')
        policy = self.write_policy([('ours', self.root / 'target', False)],
                                   {'ours': {'high_bytes': 1, 'low_bytes': None,
                                             'rationale': 'test ceiling'}})
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, _ = self.preview(policy)
        mark = manifest['watermarks']['ours']
        self.assertTrue(mark['above_high_watermark'])
        self.assertGreater(mark['shortfall_bytes'], 0)
        self.assertEqual(mark['shortfall_bytes'],
                         mark['non_eligible_allocated_bytes'] - mark['high_bytes'])
        self.assertEqual(mark['eligible_idle_allocated_bytes'],
                         sum(u['allocated_bytes'] for u in manifest['units']
                             if u['status'] == 'eligible-idle'))
        self.assertTrue(protected_unit.is_dir())

    def test_incremental_zero_candidates_are_legal(self):
        (self.root / 'target' / 'debug').mkdir(parents=True)
        policy = self.write_policy([('ours', self.root / 'target', False)])
        with mock.patch.object(build_storage, 'ROOT', self.root):
            manifest, path = self.preview(policy)
        self.assertEqual(manifest['units'], [])
        with mock.patch.object(build_storage, 'ROOT', self.root), \
                mock.patch.object(build_storage, 'refuse_active_processes'):
            code, receipt = self.apply(path, policy)
        self.assertEqual(code, 0)
        self.assertTrue(receipt['zero_delete'])

    def test_legacy_cli_unchanged(self):
        cache = self.root / 'target'
        (cache / 'debug' / 'incremental').mkdir(parents=True)
        (cache / 'debug' / 'incremental' / 's').write_bytes(b'z')
        with mock.patch.object(build_storage, 'ROOT', self.root):
            with contextlib.redirect_stdout(io.StringIO()) as captured:
                self.assertEqual(build_storage.main([str(cache)]), 0)
        payload = json.loads(captured.getvalue())
        self.assertEqual(payload['mode'], 'preview')
        self.assertEqual(payload['caches'][0]['files'], 1)


if __name__ == '__main__':
    unittest.main()
