"""Adversarial admission controls for the public fixed four-owner gate."""
import copy
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent / 'test-governance'))
import four_repository as gate


class FourOwnerControls(unittest.TestCase):
    def test_distribution_consumer_source_and_closure_controls(self):
        # Run the existing real distribution-consumer oracle, including its
        # current-cohort source/closure mutations, without legacy tm fixtures.
        suite = gate.module('four_distribution_controls', Path(__file__).parent / 'tests/test_web_test_ownership.py')
        suite.WebOwnershipTests('test_distribution_rejects_partial_missing_steps_changed_sources_and_unexecuted_commands').debug()

    def test_web_original_required_cannot_be_consistently_dropped(self):
        rows = gate.load(gate.ROOT / 'scripts/test-governance/web-test-migrations.json')['migrations']
        selected = {r['new']['binary'] + '\t' + r['new']['test'] for r in rows if not r['original_ignored']}
        gate.owner_selection('web', selected, selected)
        selected.pop()
        with self.assertRaisesRegex(ValueError, 'migrated required'):
            gate.owner_selection('web', selected, selected)

    def test_ui_required_and_actual_media_credit_are_both_required(self):
        baseline, required = gate.full.baseline_for('tf', 'ui-full')
        rows = gate.load(gate.ROOT / 'scripts/test-governance/media-test-migrations.json')['migrations']
        moved = {r['old'] for r in rows}
        credit = {r['new'] for r in rows if r['old'] in gate.full.raw_baseline('ui')[1]}
        ids, selected = baseline - moved, required - moved
        extra = {'current\textra' + str(i) for i in range(100)}
        ids, selected = ids | extra, selected | extra
        gate.owner_selection('ui-full', ids, selected, credit)
        with self.assertRaisesRegex(ValueError, 'actual media PASS'):
            gate.owner_selection('ui-full', ids, selected, set())
        gate.owner_selection('ui-full', ids - {'current\textra0'}, selected - {'current\textra0'}, credit)
        victim = sorted(required - moved)[0]
        with self.assertRaisesRegex(ValueError, 'baseline coverage lost'):
            gate.owner_selection('ui-full', ids - {victim}, selected - {victim}, credit)

    def test_man_floor_is_shared_with_receipt_consumer(self):
        with self.assertRaisesRegex(ValueError, 'minimum'):
            gate.owner_selection('man-t', {'p\tt'}, {'p\tt'})

    def test_libtest_summary_cannot_hide_failure_duplicate_or_extra(self):
        raw = b'test a ... ok\ntest result: ok. 1 passed; 0 failed;\n'
        self.assertEqual(gate.libtest_passes(raw, 'p'), {'p\ta'})
        for bad in (raw + b'test a ... ok\n', raw.replace(b'0 failed', b'1 failed'), raw.replace(b'1 passed', b'2 passed')):
            with self.assertRaises(ValueError):
                gate.libtest_passes(bad, 'p')

    def test_public_validator_rejects_missing_leg_and_wrong_checkout(self):
        identities = {k: {'root': str(gate.ROOT if k == 'language' else gate.ROOT.parent / ('auto-' + k))}
                      for k in gate.OWNERS}
        receipt = {'schema': 'four-repository-fixed/1', 'outcome': 'pass',
                   'identities': identities, 'identities_after': identities,
                   'language_groups': gate.LANGUAGE_GROUPS, 'language': {'path': 'mock'}, 'results': []}
        with tempfile.TemporaryDirectory() as td, patch.object(gate, 'snapshot', side_effect=lambda p: {'root': str(p)}), \
                patch.object(gate, 'read', return_value=b''), patch.object(gate, 'fixed_proof', return_value=set()), \
                patch.object(gate, 'require_clean'), patch.object(gate, 'validate_layout'):
            with self.assertRaisesRegex(ValueError, 'external owner leg'):
                gate.validate(receipt, Path(td))
            bad = copy.deepcopy(receipt)
            bad['language_groups'] = gate.LANGUAGE_GROUPS[:-1]
            with self.assertRaisesRegex(ValueError, 'groups omitted'):
                gate.validate(bad, Path(td))

    def test_native_admission_rejects_self_consistent_listing_omission(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            spec = {'leg': 'web', 'cwd': root, 'kind': 'nextest', 'options': ['-p', 'p', '--lib']}
            def listing(names):
                return json.dumps({'rust-suites': {'p': {'package-name': 'p', 'binary-name': 'p', 'cwd': str(root),
                    'testcases': {n: {'ignored': False, 'filter-match': {'status': 'matches'}} for n in names}}}}).encode()
            stored = root / 'list.json'; stored.write_bytes(listing(['a']))
            run = root / 'run.json'; run.write_bytes(b'{"type":"suite","event":"started"}\n'
                b'{"type":"test","event":"started","name":"p::p$a"}\n'
                b'{"type":"test","event":"ok","name":"p::p$a"}\n{"type":"suite","event":"ok"}\n')
            lc, rc = gate.native_commands(spec)
            command = lambda argv: {'argv': argv, 'cwd': str(root), 'exit': 0, 'meter': {'error': None}}
            row = {'listing': gate.proof(stored), 'run_output': gate.proof(run), 'list_command': lc, 'run_command': rc,
                   'cwd': str(root), 'selected_ids': ['p\ta'], 'condition_excluded': [], 'run_exit': 0,
                   'preparation': command(lc), 'execution': command(rc)}
            with patch.object(gate, 'source_listing', return_value=listing(['a'])):
                self.assertEqual(gate.native_proof(row, spec, {}, root / 'check'), ({'p\ta'}, {'p\ta'}))
            with patch.object(gate, 'source_listing', return_value=listing(['a', 'required'])):
                with self.assertRaisesRegex(ValueError, 'denominator'):
                    gate.native_proof(row, spec, {}, root / 'check')
            row['execution']['meter']['error'] = 'descendant leak'
            with self.assertRaisesRegex(ValueError, 'process tree'):
                gate.native_proof(row, spec, {}, root / 'check')

    def test_output_path_cannot_escape_invocation(self):
        with tempfile.TemporaryDirectory() as td:
            with self.assertRaisesRegex(ValueError, 'escapes'):
                gate.local_output(Path(td), '../another.stdout')

    def test_bare_ok_terminal_cannot_certify_execution(self):
        with self.assertRaisesRegex(ValueError, 'without actual start'):
            gate.native_terminals(b'{"type":"test","event":"ok","name":"p::p$a"}\n', {'p::p$a': {}}, set())

    def test_only_source_pinned_reaped_preparation_auxiliary_is_admitted(self):
        import hashlib
        import process_capture
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            auxiliary = root / 'auxiliary.exe'; auxiliary.write_bytes(b'known compiler helper')
            policy = root / 'policy.json'
            policy.write_text(json.dumps({'schema': 'test-compiler-auxiliaries/1', 'images': [
                {'path': str(auxiliary), 'sha256': hashlib.sha256(auxiliary.read_bytes()).hexdigest()}]}))
            meter = {'exit': 0, 'error': 'descendants live after workload exit',
                     'live_after_exit': {'cleanup_complete': True, 'active': 1, 'remaining_active': 0,
                                        'processes': [{'image': str(auxiliary)}]}}
            record = {'argv': ['cargo', 'nextest', 'list'], 'cwd': str(root), 'exit': 0, 'meter': meter}
            with patch.object(process_capture, 'COMPILER_AUXILIARIES', policy):
                gate.command_proof(record, record['argv'], root, preparation=True)
                with self.assertRaises(ValueError): gate.command_proof(record, record['argv'], root)
                meter['live_after_exit']['remaining_active'] = 1
                with self.assertRaises(ValueError): gate.command_proof(record, record['argv'], root, preparation=True)
                meter['live_after_exit']['remaining_active'] = 0
                meter['accepted_compiler_auxiliary_cleanup'] = True
                auxiliary.write_bytes(b'changed helper')
                with self.assertRaises(ValueError): gate.command_proof(record, record['argv'], root, preparation=True)

    def test_frozen_sources_reject_new_untracked_runtime(self):
        with patch.object(gate.full, 'require_clean_tree'), \
                patch.object(gate.subprocess, 'check_output', side_effect=['', 'scripts/new_runtime.py\n']):
            with self.assertRaisesRegex(ValueError, 'untracked WIP'):
                gate.require_clean('language', gate.ROOT)

    def test_actual_path_dependency_rejects_another_clean_owner(self):
        with tempfile.TemporaryDirectory() as td:
            root = Path(td)
            documents = [
                {'packages': [{'name': 'lang', 'manifest_path': str(root/'lang/Cargo.toml'),
                    'dependencies': [{'name': 'web', 'path': str(root/'ambient-web')}]}]},
                {'packages': [{'name': 'web', 'manifest_path': str(root/'selected-web/Cargo.toml'), 'dependencies': []}]}]
            with patch.object(gate.subprocess, 'check_output', side_effect=[json.dumps(d) for d in documents]):
                with self.assertRaisesRegex(ValueError, 'owner mismatch'):
                    gate.validate_layout({'language': root/'lang', 'web': root/'selected-web'})


if __name__ == '__main__':
    unittest.main()
