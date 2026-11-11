"""PLAN-941 strict evidence I/O controls; no synthetic milestone PASS."""
import hashlib
import json
import os
from pathlib import Path
import tempfile
import unittest
import sys
import copy
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/test-governance'))
import ace_m1_arch_acceptance as m
import ace_m1_legacy as legacy
import ace_m1_case_evidence as case_evidence
import ace_m1_scalar_evidence as scalar_evidence
import ace_m1_cert_evidence as cert_evidence
import ace_m1_syntax_evidence as syntax_evidence
import ace_m1_abi_evidence as abi_evidence
import ace_m1_first_slice as first_slice
import ace_m1_obligation_catalog as obligation_catalog
import ace_m1_legacy_execution as legacy_execution
import ace_m1_legacy_base as legacy_base
import ace_m1_parent_contracts as parent_contracts
import ace_m1_av_typed_evidence as av_typed_evidence
import ace_m1_native_typed_evidence as native_typed_evidence
import ace_m1_legacy_obligations as legacy_obligations
import ace_m1_budget_contracts as budget_contracts
import ace_m1_incremental_restore as incremental_restore
import ace_m1_budget_time as budget_time_module
import ace_m1_system_controls as system_controls
import ace_m1_arch_obligations as arch_obligations
import ace_m1_registry as expanded_registry
import ace_m1_text_evidence as text_evidence
import ace_m1_cte_evidence as cte_evidence
import ace_m1_daily_evidence as daily_evidence
import ace_m1_text_q4_evidence as text_q4
import ace_m1_scalar_q4_evidence as scalar_q4
import ace_m1_scalar_cert_evidence as scalar_cert
import ace_m1_scalar_release_evidence as scalar_release
import ace_m1_record_text_evidence as record_text
import ace_m1_collections_evidence as collections_evidence
import ace_m1_query_evidence as query_evidence
import ace_m1_source_supplement as source_supplement
import ace_m1_original_data_sources as original_data
import ace_m1_data_diagnostics as data_diagnostics
from unittest.mock import patch
import ace_m1_legacy_arch as legacy_arch


class LegacyArchitectureControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        cls.temp = tempfile.TemporaryDirectory(prefix='legacy-architecture-controls-', dir=scratch)
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        cls.contract = json.loads((ROOT / legacy_arch.CONTRACT).read_text(encoding='utf-8'))
        before = dict(commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                      source_pins=cls.contract['source_pins'])
        from process_capture import capture
        argv = [shutil.which('pwsh') or shutil.which('powershell'), '-NoProfile', '-File',
                str(ROOT / legacy_arch.SCRIPT), '-Scope', 'acceptance', '-RepoRoot', str(ROOT),
                '-OutDir', str(cls.root)]
        actual = capture(argv, cwd=ROOT, timeout=30, output_cap=1 << 20, error_cap=1 << 20)
        if actual.returncode:
            raise RuntimeError('real old architecture accounting failed')
        after = dict(commit=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                     source_pins={p:hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in before['source_pins']})
        if before != after:
            raise RuntimeError('real old accounting input changed')
        command = dict(schema='ace.m1.legacy-accounting-command/1', argv=argv, cwd=str(ROOT),
                       exit=actual.returncode, meter=actual.measurement,
                       stdout='producer.stdout', stderr='producer.stderr')
        cls.original = {'source-pins-before.json':json.dumps(before).encode(),
                        'source-pins-after.json':json.dumps(after).encode(),
                        'command.json':json.dumps(command).encode(),
                        'producer.stdout':actual.stdout, 'producer.stderr':actual.stderr,
                        'ace-m1-acceptance.json':(cls.root / 'ace-m1-acceptance.json').read_bytes()}

    def setUp(self):
        for name, content in self.original.items():
            (self.root / name).write_bytes(content)

    def admit(self, transform=None):
        budget = m.LiveBudget()
        inputs = m.EvidenceScope(ROOT, budget)
        evidence = m.EvidenceScope(self.root.parent, budget)
        parent = legacy.admit_accounting(evidence, self.root.name + '/ace-m1-acceptance.json', inputs)
        if transform:
            transform(parent)
        with patch.object(subprocess, 'Popen', side_effect=AssertionError('reader spawned a producer')):
            return legacy_arch.admit(inputs, evidence, {'root':self.root.name}, parent)

    def test_actual_guard_and_all_original_counts_admit_without_execution(self):
        admitted = self.admit()
        self.assertEqual(admitted['observations'], dict.fromkeys(legacy_arch.ARCH, 'PASS'))
        # PLAN-992..998 reception: the old gate's own accounting claims after
        # the three-domain closure; the expanded-M1 claim stays separate.
        self.assertTrue(admitted['source_evidence']['original_claim'])
        self.assertEqual(admitted['source_evidence']['old_certification_gaps'], 0)
        self.assertEqual(admitted['legacy_execution'], 'UNOBSERVED')

    def test_exit_zero_with_forged_or_incomplete_raw_terminal_refuses(self):
        for text in (b'PASS', self.original['producer.stdout'].replace(b'can_claim_m1: True', b'can_claim_m1: False'),
                     self.original['producer.stdout'].replace(b'trap+scene-validated(4 trap + 6 ', b'trap+scene-validated(4 trap + 5 ')):
            with self.subTest(text=text[:30]):
                (self.root / 'producer.stdout').write_bytes(text)
                with self.assertRaises(m.AdmissionError):
                    self.admit()

    def test_cancelled_and_wrong_type_meter_cannot_pass(self):
        for change in (lambda d:d.update(exit=True), lambda d:d['meter'].update(error='deadline'),
                       lambda d:d['meter'].update(job_processes=0), lambda d:d['meter'].update(unknown=0),
                       lambda d:d['meter'].update(total_cpu_ms=999999)):
            command = json.loads(self.original['command.json'])
            change(command)
            (self.root / 'command.json').write_text(json.dumps(command), encoding='utf-8')
            with self.subTest(command=command), self.assertRaises(m.AdmissionError):
                self.admit()

    def test_original_source_window_cannot_be_replaced_by_summary(self):
        before = json.loads(self.original['source-pins-before.json'])
        before['source_pins'][legacy_arch.SCRIPT] = '0' * 64
        for name in ('source-pins-before.json', 'source-pins-after.json'):
            (self.root / name).write_text(json.dumps(before), encoding='utf-8')
        with self.assertRaises(m.AdmissionError):
            self.admit()

    def test_rehashed_cert_gap_forgery_and_alternate_ledger_refuse(self):
        record = json.loads(self.original['ace-m1-acceptance.json'])
        record['verdicts']['can_claim_m1'] = False
        (self.root / 'ace-m1-acceptance.json').write_text(json.dumps(record), encoding='utf-8')
        with self.assertRaises(m.AdmissionError):
            self.admit()
        (self.root / 'ace-m1-acceptance.json').write_bytes(self.original['ace-m1-acceptance.json'])
        with self.assertRaises(m.AdmissionError):
            self.admit(lambda d:d.update(receipt_sha256='0' * 64))

    def test_new_hir_certificates_never_discharge_old38_guard(self):
        admitted = self.admit(lambda d:d.update(new_hir_certificates={'PASS':292}, new_target='ACE-A2R-HIR'))
        # PLAN-992..998 reception: the old gate claims on its own certified
        # closure; injected HIR certificates change none of its accounting.
        self.assertTrue(admitted['source_evidence']['original_claim'])
        self.assertEqual(admitted['source_evidence']['old_certification_gaps'], 0)


class EvidenceAdmissionTests(unittest.TestCase):
    def setUp(self):
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(prefix='admission-', dir=scratch)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.scope = m.EvidenceScope(self.root)

    def put(self, raw, name='child.json'):
        (self.root / name).write_bytes(raw)
        return name

    def test_legitimate_nonzero_and_utf8_control(self):
        raw = '{"counter":17,"text":"恢复"}'.encode()
        value, sha = self.scope.read_json(self.put(raw))
        self.assertEqual(value, {'counter': 17, 'text': '恢复'})
        self.assertEqual(sha, hashlib.sha256(raw).hexdigest())
        self.assertEqual(m.counter(value['counter'], 'counter'), 17)
        m.exact_keys(value, ('counter', 'text'), 'child')

    def test_duplicate_keys_at_any_depth(self):
        for raw in (b'{"schema":1,"schema":2}', b'{"a":{"n":1,"n":2}}'):
            with self.subTest(raw=raw), self.assertRaisesRegex(m.AdmissionError, 'duplicate'):
                self.scope.read_json(self.put(raw))

    def test_truncation_trailing_invalid_utf8_and_nonfinite(self):
        for raw in (b'{"a":', b'{} {}', b'{"a":"\xff"}', b'NaN', b'Infinity', b'1e999'):
            with self.subTest(raw=raw), self.assertRaises(m.AdmissionError):
                self.scope.read_json(self.put(raw))

    def test_counter_type_and_overflow_controls(self):
        for bad in (True, False, -1, 1 << 64, '0', 0.0, None):
            with self.subTest(bad=bad), self.assertRaises(m.AdmissionError):
                m.counter(bad, 'attempted')
        self.assertEqual(m.counter((1 << 64) - 1, 'attempted'), (1 << 64) - 1)

    def test_integer_decode_is_bounded_before_conversion(self):
        for raw in (b'18446744073709551616', b'-9223372036854775809', b'9' * 10000):
            with self.subTest(length=len(raw)), self.assertRaises(m.AdmissionError):
                self.scope.read_json(self.put(raw))

    def test_unknown_missing_and_wrong_object_shape(self):
        for bad in ({'a': 1, 'extra': 2}, {}, [], None):
            with self.subTest(bad=bad), self.assertRaises(m.AdmissionError):
                m.exact_keys(bad, ('a',), 'schema')
        self.assertEqual(m.exact_keys({'a': 1}, ('a',), 'schema'), {'a': 1})

    def test_byte_cap_not_character_cap(self):
        raw = json.dumps('界' * (m.DOCUMENT_BYTES // 3), ensure_ascii=False).encode()
        self.assertGreater(len(raw), m.DOCUMENT_BYTES)
        self.assertLess(len(raw.decode()), m.DOCUMENT_BYTES)
        with self.assertRaises(m.ResourceError):
            self.scope.read_json(self.put(raw))

    def test_document_exact_byte_limit_accepted(self):
        raw = b'"' + b'x' * (m.DOCUMENT_BYTES - 2) + b'"'
        value, _ = self.scope.read_json(self.put(raw))
        self.assertEqual(len(value), m.DOCUMENT_BYTES - 2)

    def test_predecode_live_data_reservation(self):
        # Small input with many tiny nodes consumes real decoder/index space;
        # file-size admission alone would undercount it.
        raw = b'[' + b'0,' * 220000 + b'0]'
        self.assertLess(len(raw), m.DOCUMENT_BYTES)
        with self.assertRaises(m.ResourceError):
            self.scope.read_json(self.put(raw))

    def test_aggregate_reservations_cannot_reset_per_document(self):
        raw = b'"' + b'x' * (3 << 20) + b'"'
        name = self.put(raw)
        first, _ = self.scope.read_json(name)
        self.assertEqual(len(first), 3 << 20)
        with self.assertRaises(m.ResourceError):
            self.scope.read_json(name)

    def test_live_budget_is_shared_across_authorized_roots(self):
        raw = b'"' + b'x' * (3 << 20) + b'"'
        self.put(raw)
        other = self.root / 'other'
        other.mkdir()
        (other / 'child.json').write_bytes(raw)
        second = m.EvidenceScope(other, self.scope.budget)
        first, _ = self.scope.read_json('child.json')
        self.assertEqual(len(first), 3 << 20)
        with self.assertRaises(m.ResourceError):
            second.read_json('child.json')

    def test_path_escape_alias_device_and_absolute_controls(self):
        for bad in ('../child.json', '/child.json', 'D:/child.json', 'a\\b',
                    'a//b', './child.json', 'a/../child.json', 'a./b', 'a /b',
                    'CON', 'nul.txt', 'a:stream', 'a\0b', 'a?b', '\ud800', '', None):
            with self.subTest(bad=bad), self.assertRaises(m.AdmissionError):
                self.scope.path(bad)

    def test_missing_or_directory_is_not_a_file(self):
        (self.root / 'directory').mkdir()
        for name in ('absent', 'directory'):
            with self.subTest(name=name), self.assertRaises(m.AdmissionError):
                self.scope.path(name)

    def test_streamed_hash_rejects_stale_and_bad_digest(self):
        raw = b'actual payload' * 100000
        name = self.put(raw, 'actual.bin')
        sha = hashlib.sha256(raw).hexdigest()
        self.assertEqual(self.scope.sha256(name, sha), self.root / name)
        for bad in ('0' * 64, sha.upper(), '', True):
            with self.subTest(bad=bad), self.assertRaises(m.AdmissionError):
                self.scope.sha256(name, bad)

    def test_reparse_rejected_before_resolve(self):
        # Simulate the Windows reparse attribute on an actual lexical
        # ancestor: never create a junction/symlink inside any worktree.
        from unittest.mock import patch
        original = Path.lstat
        self.put(b'{}')
        def marked(path):
            info = original(path)
            if path == self.root:
                class Reparse:
                    st_mode = info.st_mode
                    st_file_attributes = 0x400
                return Reparse()
            return info
        with patch.object(Path, 'lstat', marked), self.assertRaisesRegex(m.AdmissionError, 'reparse'):
            self.scope.path('child.json')


class LegacyAccountingTests(unittest.TestCase):
    """Mutate the real /2 producer's fresh output, not a reimplemented fixture."""

    @classmethod
    def setUpClass(cls):
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        scratch.mkdir(parents=True, exist_ok=True)
        cls.temp = tempfile.TemporaryDirectory(prefix='legacy-controls-', dir=scratch)
        cls.addClassCleanup(cls.temp.cleanup)
        cls.root = Path(cls.temp.name)
        ps = shutil.which('pwsh') or shutil.which('powershell')
        if not ps:
            raise RuntimeError('PowerShell required for real legacy accounting producer')
        from process_capture import capture
        argv = [ps, '-NoProfile', '-File', str(ROOT / 'scripts/verify-ace-m1-852.ps1'),
                '-Scope', 'acceptance', '-RepoRoot', str(ROOT), '-OutDir', str(cls.root)]
        result = capture(argv, cwd=ROOT, timeout=30, output_cap=1 << 20, error_cap=1 << 20)
        (cls.root / 'producer.stdout').write_bytes(result.stdout)
        (cls.root / 'producer.stderr').write_bytes(result.stderr)
        if result.returncode:
            raise RuntimeError('legacy producer failed: ' + result.stderr.decode(errors='replace'))
        cls.base = json.loads((cls.root / 'ace-m1-acceptance.json').read_text(encoding='utf-8'))

    def setUp(self):
        self.record = json.loads(json.dumps(self.base))

    def admit(self, record=None, repo=None):
        name = 'mutation-' + self._testMethodName + '.json'
        (self.root / name).write_text(json.dumps(self.record if record is None else record), encoding='utf-8')
        budget = m.LiveBudget()
        return legacy.admit_accounting(m.EvidenceScope(self.root, budget), name,
                                       m.EvidenceScope(repo or ROOT, budget))

    def test_real_accounting_preserves_all_counts_and_current_cert_gaps(self):
        from unittest.mock import patch
        # Discovery/preparation above is explicit; admission itself is read-only.
        # PLAN-992/993 reception (941 round 108): seventeen value cells plus
        # K06 are certified (ok-emit, cert_axis none) — the structural
        # counts are unchanged and the cert-gap axis is down to 20.
        with patch.object(subprocess, 'Popen', side_effect=AssertionError('reader spawned a tool')):
            facts = self.admit()
        self.assertEqual((facts['required'], facts['na'], facts['accepted_diff'], facts['future']),
                         (313, 134, 32, 45))
        # PLAN-992..998 reception: 38 -> 0 open gaps (value+trap+scene all
        # certified); the old accounting honestly claims.
        self.assertEqual(facts['cert_gaps'], [])
        self.assertTrue(facts['can_claim_m1'])
        self.assertEqual(facts['execution'], 'UNOBSERVED')

    def test_cannot_promote_script_exit_or_boolean_to_claim(self):
        # PLAN-998 reception: the ledger-derived honest value is now True, so
        # the disagreeing boolean (a demotion forgery) is what gets refused.
        self.record['verdicts']['can_claim_m1'] = False
        with self.assertRaises(m.AdmissionError):
            self.admit()

    def test_missing_and_duplicate_original_cells_rejected(self):
        for kind in ('missing', 'duplicate'):
            record = json.loads(json.dumps(self.base))
            if kind == 'missing':
                record['ledger'].pop()
            else:
                record['ledger'][-1] = record['ledger'][0]
            with self.subTest(kind=kind), self.assertRaises(m.AdmissionError):
                self.admit(record)

    def test_cannot_reclassify_required_or_cert_axis(self):
        # PLAN-992 reception: B08 is certified (cert_axis none) — the cert
        # axis tamper needs a cell that still carries one (B13).
        # Round 113: every corpus cell is certified (cert_axis none) — the tamper
        # reclassifies a certified corpus cell as gapped instead.
        for field, value, cid in (('required', False, 'B08'), ('cert_axis', 'cert-gap', 'B08'), ('token', 'ok', 'B08')):
            record = json.loads(json.dumps(self.base))
            cell = next(r for r in record['ledger'] if r['ns'] == 'corpus' and r['id'] == cid and r['path'] == 'a2r')
            cell[field] = value
            with self.subTest(field=field), self.assertRaises(m.AdmissionError):
                self.admit(record)

    def test_wrong_json_types_not_equal_to_counts_or_booleans(self):
        for location in ('rows', 'required', 'executed'):
            record = json.loads(json.dumps(self.base))
            if location == 'rows':
                record['registered']['rows'] = 131.0
            elif location == 'required':
                record['ledger'][0]['required'] = 1
            else:
                record['executed']['executed'] = False
            with self.subTest(location=location), self.assertRaises(m.AdmissionError):
                self.admit(record)

    def test_summary_drift_unknown_and_old_schema_rejected(self):
        for location in ('summary', 'unknown', 'schema', 'future', 'accepted'):
            record = json.loads(json.dumps(self.base))
            if location == 'summary':
                record['layers']['required_cells'] = 312
            elif location == 'unknown':
                record['trusted'] = True
            elif location == 'schema':
                record['schema'] = 'ace.m1.acceptance/1'
            elif location == 'future':
                record['future_gaps'].pop()
            else:
                record['ruled_accepted'].pop()
            with self.subTest(location=location), self.assertRaises(m.AdmissionError):
                self.admit(record)

    def test_parent_echo_and_selected_count_cannot_supply_execution(self):
        for field, value in (('executed', 313), ('parent_admission', {'PASS': True}),
                             ('selection', 'all')):
            record = json.loads(json.dumps(self.base))
            record['executed'][field] = value
            with self.subTest(field=field), self.assertRaises(m.AdmissionError):
                self.admit(record)

    def test_frozen_approval_or_input_drift_is_stale(self):
        with tempfile.TemporaryDirectory(prefix='repo-', dir=self.root) as scratch:
            clone = Path(scratch)
            for relative in legacy.INPUTS:
                destination = clone / relative
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / relative, destination)
            decision = clone / legacy.DECISION
            decision.write_bytes(decision.read_bytes() + b'\nchanged ruling\n')
            with self.assertRaisesRegex(m.AdmissionError, 'STALE'):
                self.admit(repo=clone)


class CaseCopyBindingTests(unittest.TestCase):
    """Schema/linkage unit controls; generated ERROR data supplies no M1 PASS."""
    def setUp(self):
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        self.temp = tempfile.TemporaryDirectory(prefix='case-binding-', dir=scratch)
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.old = 'C:\\old-runtime'
        h = '1' * 64
        self.expected = {'schema': 'test-case-inventory/2', 'executed': 0,
                         'suite': 'unit', 'suite_sha256': h, 'checks': [{
                             'case': 'one', 'check': 'check', 'adapter': 'probe',
                             'action': 'run', 'tier': None, 'case_file_sha256': h,
                             'source_sha256': h, 'oracle_sha256': h, 'contract_sha256': h}]}
        self.report = {'schema': 'autolang.test-report/2', 'suite': 'unit',
                       'suite_sha256': h, 'required_adapters': [], 'toolchain': {},
                       'coverage_gaps': [], 'load_errors': [],
                       'summary': {'total': 1, 'ERROR': 1}, 'results': [{
                           'execution_key': f'unit:{h}:one:check', 'case': 'one',
                           'check': 'check', 'adapter': 'probe', 'action': 'run',
                           'tier': None, 'status': 'ERROR', 'duration_ms': 1,
                           'identity': dict.fromkeys(('suite', 'case_file', 'source', 'oracle', 'contract'), h)}]}

    def write_copy(self, extra=None):
        files = {'report.json': json.dumps(self.report).encode(), **(extra or {})}
        refs = []
        for name, raw in files.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(raw)
            refs.append({'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
        self.manifest = {'schema': 'test-case-artifact-copy/1',
                         'original_runtime_root': self.old, 'files': refs,
                         'runtime_cleanup_complete': True, 'note': 'schema unit control only'}
        self.save_manifest()
        return case_evidence.CaseCopy(m.EvidenceScope(self.root))

    def save_manifest(self):
        (self.root / 'artifact-copy.json').write_text(json.dumps(self.manifest))

    def test_binding_retains_error_without_execution_verdict(self):
        result = case_evidence.admit_case_bindings(self.write_copy(), self.expected)
        self.assertEqual(result['execution_judgement'], 'UNOBSERVED')
        self.assertEqual(result['checks'][0][1]['status'], 'ERROR')

    def test_pass_echo_still_supplies_no_judgement(self):
        self.report['results'][0]['status'] = 'PASS'
        self.report['summary'] = {'total': 1, 'PASS': 1}
        self.assertEqual(case_evidence.admit_case_bindings(self.write_copy(), self.expected)
                         ['execution_judgement'], 'UNOBSERVED')

    def test_missing_duplicate_extra_and_wrong_typed_summary_refused(self):
        original = self.report['results']
        for rows in ([], original * 2, [dict(original[0], check='foreign')]):
            self.report['results'] = rows
            with self.subTest(rows=len(rows)), self.assertRaises(m.AdmissionError):
                case_evidence.admit_case_bindings(self.write_copy(), self.expected)
        self.report['results'] = original
        self.report['summary']['total'] = True
        with self.assertRaises(m.AdmissionError):
            case_evidence.admit_case_bindings(self.write_copy(), self.expected)

    def test_current_identity_source_oracle_contract_and_key_controls(self):
        for field in self.report['results'][0]['identity']:
            self.report['results'][0]['identity'][field] = '2' * 64
            with self.subTest(field=field), self.assertRaises(m.AdmissionError):
                case_evidence.admit_case_bindings(self.write_copy(), self.expected)
            self.report['results'][0]['identity'][field] = '1' * 64
        self.report['results'][0]['execution_key'] = 'old:key'
        with self.assertRaises(m.AdmissionError):
            case_evidence.admit_case_bindings(self.write_copy(), self.expected)

    def test_copy_drift_duplicate_and_unfinished_cleanup_refused(self):
        copy = self.write_copy()
        (self.root / 'report.json').write_bytes(b'{}')
        with self.assertRaises(m.AdmissionError):
            copy.json('report.json')
        self.write_copy()
        self.manifest['files'] *= 2
        self.save_manifest()
        with self.assertRaises(m.AdmissionError):
            case_evidence.CaseCopy(m.EvidenceScope(self.root))
        self.write_copy()
        self.manifest['runtime_cleanup_complete'] = False
        self.save_manifest()
        with self.assertRaises(m.AdmissionError):
            case_evidence.CaseCopy(m.EvidenceScope(self.root))

    def test_old_absolute_names_are_never_opened(self):
        copy = self.write_copy()
        self.assertEqual(copy.path_from_original(self.old + '\\report.json'), 'report.json')
        for name in ('C:\\other\\report.json', self.old + '\\..\\outside', '//server/share/report.json'):
            with self.subTest(name=name), self.assertRaises(m.AdmissionError):
                copy.path_from_original(name)

    def wire_copy(self, **terminal_changes):
        directory, identity = 'run/one/check/worker-exchanges', '1-0'
        files = {}
        def ref(role, raw):
            name = identity + '-' + role
            files[directory + '/' + name] = raw
            return {'path': name, 'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()}
        start = {'schema': 'case-worker-exchange/1', 'exchange': identity, 'phase': 'start',
                 'protocol': 'unit/1', 'recipe': 'unit/error', 'worker_sha256': '1' * 64,
                 'request': ref('request', b'{"source":"unit"}')}
        start_bytes = json.dumps(start).encode()
        start_path = directory + '/' + identity + '-start.json'
        files[start_path] = start_bytes
        terminal = {'schema': 'case-worker-exchange/1', 'exchange': identity, 'phase': 'terminal',
                    'start_sha256': hashlib.sha256(start_bytes).hexdigest(), 'exit_code': 7,
                    'stdout': ref('stdout', b'\xff\x00'), 'stderr': ref('stderr', b'error'),
                    'transport_error': 'unit failure', **terminal_changes}
        files[directory + '/' + identity + '-terminal.json'] = json.dumps(terminal).encode()
        return self.write_copy(files), start_path

    def read_wire(self, copy, path, **changes):
        return case_evidence.admit_exchange(copy, path, **{
            'protocol': 'unit/1', 'recipe': 'unit/error', 'worker_sha256': '1' * 64, **changes})

    def test_wire_preserves_bytes_nonzero_and_error(self):
        wire = self.read_wire(*self.wire_copy())
        self.assertEqual((wire['stdout'], wire['exit_code'], wire['transport_error']),
                         (b'\xff\x00', 7, 'unit failure'))
        self.assertEqual(wire['execution_judgement'], 'UNOBSERVED')

    def test_wire_terminal_and_current_tool_binding_controls(self):
        for changes in ({'start_sha256': '2' * 64}, {'exit_code': True},
                        {'stdout': []}, {'transport_error': True}):
            with self.subTest(changes=changes), self.assertRaises(m.AdmissionError):
                self.read_wire(*self.wire_copy(**changes))
        for changes in ({'protocol': 'old/1'}, {'recipe': 'foreign'}, {'worker_sha256': '2' * 64}):
            with self.subTest(changes=changes), self.assertRaises(m.AdmissionError):
                self.read_wire(*self.wire_copy(), **changes)


class ScalarObservationTests(unittest.TestCase):
    """Domain admission units. Real compiled Q4 controls are recorded separately."""
    setUp = EvidenceAdmissionTests.setUp

    def wire(self, target='av', value=42, caps=()):
        import base64
        source = b'fn main() int { return 42 }\n'
        recipe = 'ac-av/core-run-v1' if target == 'av' else 'ac/mir-native-scalar-v1'
        protocol = 'avworker/1' if target == 'av' else 'mnworker/1'
        request = {'recipe': recipe, 'nonce': 'unit-nonce',
                   'source_b64': base64.b64encode(source).decode(),
                   'entry': 'd_main', 'caps': list(caps), 'store_root': 'unit-store'}
        if target == 'av':
            request['steps'] = [{'op': 'run'}]
            ok = {'recipe': recipe, 'worker_version': 'avworker/1.0',
                  'av_core_version': 'av.core-i32/1', 'steps': [{
                      'op': 'run', 'outcome': 'success', 'value': value, 'channel': 'value',
                      'trap': None, 'diag': None, 'counters': {'av_execute': 1},
                      'spans': [['av_execute', 0.1]], 'source_hash': scalar_evidence._fnv1a64(source),
                      'wall_ms': 1.0, 'trace': []}]}
        else:
            request['workdir'] = 'unit-work'
            ok = {'recipe': recipe, 'worker_version': 'mnworker/1.0',
                  'mn_abi': 'msvc-x64/priv-i32-bool-v1', 'mn_target': 'x86_64-pc-windows-msvc',
                  'counters': {'mn_runs': 1}, 'spans': {'run': 1}, 'manifest_fp': '1'*16,
                  'exit_code': value, 'stderr_hex': '', 'stdout_hex': ''}
        request['request_hash'] = hashlib.sha256(json.dumps(request, ensure_ascii=False,
                                                            separators=(',', ':')).encode()).hexdigest()
        response = {'schema': protocol, 'nonce': request['nonce'],
                    'request_hash': request['request_hash'], 'ok': ok}
        wire = {'protocol': protocol, 'recipe': recipe, 'exit_code': 0, 'transport_error': None,
                'stderr': b'', 'request': json.dumps(request).encode(),
                'stdout': json.dumps(response).encode()}
        expected = {'recipe': recipe, 'source_files': [{'sha256': hashlib.sha256(source).hexdigest()}],
                    'expected_exit': 42}
        expected['probe_bindings'] = {'av' if target == 'av' else 'native':{'entry':'d_main', 'caps':list(caps)}}
        return wire, expected

    def judge(self, wire, expected):
        return scalar_evidence.scalar_observation(self.scope, wire, expected)

    def test_independent_expected_rejects_same_wrong_on_both_targets(self):
        for target in ('av', 'native'):
            with self.subTest(target=target):
                self.assertEqual(self.judge(*self.wire(target))['judgement'], 'PASS')
                self.assertEqual(self.judge(*self.wire(target, 43))['judgement'], 'FAIL')

    def test_ct_capabilities_are_bound_to_independent_current_contract_and_value(self):
        caps = ('ac.source.cte.scalar/v1', 'cte.meta.fields/v1')
        for target in ('av', 'native'):
            with self.subTest(target=target):
                self.assertEqual(self.judge(*self.wire(target, caps=caps))['judgement'], 'PASS')
                self.assertEqual(self.judge(*self.wire(target, 43, caps=caps))['judgement'], 'FAIL')
                wire, expected = self.wire(target, caps=caps)
                expected['probe_bindings']['av' if target == 'av' else 'native']['caps'] = []
                with self.assertRaises(m.AdmissionError): self.judge(wire, expected)
        with self.assertRaises(m.AdmissionError): self.judge(*self.wire(caps=('ungranted/future',)))

    def test_echoed_source_hash_cannot_replace_current_source(self):
        import base64
        wire, expected = self.wire()
        request = json.loads(wire['request'])
        del request['request_hash']
        request['source_b64'] = base64.b64encode(b'wrong source').decode()
        request['request_hash'] = hashlib.sha256(json.dumps(request, ensure_ascii=False,
                                                            separators=(',', ':')).encode()).hexdigest()
        response = json.loads(wire['stdout'])
        response['request_hash'] = request['request_hash']
        response['ok']['steps'][0]['source_hash'] = scalar_evidence._fnv1a64(b'wrong source')
        wire['request'], wire['stdout'] = json.dumps(request).encode(), json.dumps(response).encode()
        with self.assertRaises(m.AdmissionError):
            self.judge(wire, expected)

    def test_scalar_envelope_nonce_recipe_type_and_unknown_fields(self):
        for change in ('nonce', 'recipe', 'bool-value', 'unknown', 'duplicate-span', 'nonfinite'):
            wire, expected = self.wire()
            response = json.loads(wire['stdout'])
            if change == 'nonce': response['nonce'] = 'foreign'
            if change == 'recipe': response['ok']['recipe'] = 'foreign'
            if change == 'bool-value': response['ok']['steps'][0]['value'] = True
            if change == 'unknown': response['ok']['steps'][0]['unapproved'] = 0
            if change == 'duplicate-span': response['ok']['steps'][0]['spans'] *= 2
            if change == 'nonfinite': response['ok']['steps'][0]['wall_ms'] = float('nan')
            wire['stdout'] = json.dumps(response).encode()
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                self.judge(wire, expected)

    def test_request_hash_transport_and_oracle_are_not_child_authority(self):
        for change in ('request-hash', 'bool-oracle', 'bad-source-encoding', 'nonzero', 'transport-error', 'stderr'):
            wire, expected = self.wire('native')
            if change in ('request-hash', 'bad-source-encoding'):
                request = json.loads(wire['request'])
                request['request_hash' if change == 'request-hash' else 'source_b64'] = 'bad'
                wire['request'] = json.dumps(request).encode()
            if change == 'bool-oracle': expected['expected_exit'] = True
            if change == 'nonzero': wire['exit_code'] = 7
            if change == 'transport-error': wire['transport_error'] = 'timeout'
            if change == 'stderr': wire['stderr'] = b'worker error'
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                self.judge(wire, expected)


class CertificationReaderPrimitiveTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def command(self):
        request = {'op': 'unit-only-identity'}
        producer = {'cwd': 'unit-cwd', 'worker_sha256': '1' * 64,
                    'producer_commit': '2' * 40, 'worker_path': 'unit-worker'}
        argv = [producer['worker_path'], json.dumps(request)]
        refs = {}
        for role, raw in (('stdout', b'{"schema":"unit-only"}'), ('stderr', b'')):
            path = self.root / ('unit.' + role)
            path.write_bytes(raw)
            refs[role] = {'path': path.name, 'sha256': hashlib.sha256(raw).hexdigest(), 'bytes': len(raw)}
        record = {'schema': 'ace.m1.command/1', 'argv': argv, 'cwd': producer['cwd'],
                  'exit': 0, 'worker_sha256': producer['worker_sha256'],
                  'producer_commit': producer['producer_commit'], **refs,
                  'meter': {'argv': argv, 'exit': 0, 'error': None, 'wall_ms': 1.0,
                            'workload_cpu_ms': 1.0, 'meter_cpu_ms': 0.0, 'total_cpu_ms': 1.0,
                            'peak_process_bytes': 1, 'peak_job_bytes': 1, 'job_processes': 1}}
        return record, request, producer

    def admit(self, record, request, producer):
        (self.root / 'unit.command.json').write_text(json.dumps(record))
        return cert_evidence.admit_command(self.scope, 'unit.command.json', request, producer)

    def test_command_linkage_supplies_no_certification_verdict(self):
        response, receipt = self.admit(*self.command())
        self.assertEqual(response, {'schema': 'unit-only'})
        self.assertNotIn('judgement', receipt)
        self.assertEqual(receipt['wall_ms'], 1.0)

    def test_failed_or_inconsistent_original_meter_cannot_pass(self):
        for changes in ({'exit': True}, {'exit': 7}, {'error': 'deadline'},
                        {'wall_ms': 360001}, {'job_processes': False},
                        {'job_processes': 0}, {'peak_job_bytes': -1}, {'peak_job_bytes': 0},
                        {'peak_process_bytes': 0}, {'peak_process_bytes': 2}, {'total_cpu_ms': 2.0}):
            record, request, producer = self.command()
            record['meter'].update(changes)
            with self.subTest(changes=changes), self.assertRaises(m.AdmissionError):
                self.admit(record, request, producer)

    def test_command_producer_request_raw_bytes_and_extra_fields(self):
        for change in ('producer', 'request', 'raw', 'extra', 'exit-bool'):
            record, request, producer = self.command()
            if change == 'producer': record['producer_commit'] = '3' * 40
            if change == 'request': record['argv'][1] = '{"op":"foreign"}'
            if change == 'raw': (self.root / 'unit.stdout').write_bytes(b'{}')
            if change == 'extra': record['echoed_pass'] = True
            if change == 'exit-bool': record['exit'] = False
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                self.admit(record, request, producer)

    def test_terminal_nonzero_bool_lossy_shape_and_hex_controls(self):
        good = {'exit_code': 0, 'stdout_hex': 'ff00200a', 'stderr_hex': '0afe'}
        cert_evidence._terminal(good, 'unit preserved raw bytes')
        for changes in ({'exit_code': True}, {'exit_code': 7}, {'stdout_hex': '0'},
                        {'stdout_hex': 'GG'}, {'stderr_hex': False}, {'verdict': 'PASS'}):
            with self.subTest(changes=changes), self.assertRaises(m.AdmissionError):
                cert_evidence._terminal(good | changes, 'unit')


class SyntaxReaderTests(unittest.TestCase):
    """Admitted-frame domain units, not compiled language/M1 positive evidence."""
    setUp = CaseCopyBindingTests.setUp
    write_copy = CaseCopyBindingTests.write_copy
    save_manifest = CaseCopyBindingTests.save_manifest

    def sample(self, *, diagnostic=False, reconstructed=None):
        import base64
        source = b'fn main() int { return 42 }\n'
        prefix = 'run-unit/one/check'
        scratch = self.old + '\\' + prefix.replace('/', '\\') + '\\work'
        options = {'profile': 'core', 'entry': 'main', 'cache': 'reuse',
                   'grammar': 'auto-source@core', 'capabilities': [],
                   'budget': dict.fromkeys(('build_ms', 'run_ms', 'total_ms'))}
        expected = {'adapter': 'syntax', 'action': 'parse', 'case': 'one', 'check': 'check',
                    'ac_options': options, 'source_files': [{'sha256': hashlib.sha256(source).hexdigest()}],
                    'completion': {'kind': 'error', 'stage': 'parse', 'code': 'syntax.parse', 'message': None}
                    if diagnostic else {'kind': 'success'},
                    'oracle_artifacts': [] if diagnostic else [{'path': 'reconstructed.at',
                        'file': 'oracle.at', 'sha256': hashlib.sha256(source).hexdigest()}],
                    'artifact_modes': [] if diagnostic else [{'path': 'reconstructed.at', 'compare': 'bytes'}]}
        request = {'schema': 'acworker/1', 'kind': 'parse', 'nonce': 'unit-nonce',
                   'request_hash': None, 'input': {'kind': 'source', 'source_b64': base64.b64encode(source).decode()},
                   'profile': 'core', 'entry': 'main', 'budget': {'build_ms': 60000, 'run_ms': 5000, 'total_ms': 70000},
                   'store_root': None, 'grammar': options['grammar'], 'capabilities': [], 'support_lib': None,
                   'edits': None, 'cache': 'reuse', 'project': 'ac-case', 'include_hir': not diagnostic,
                   'scratch': scratch}
        identity = '|'.join(('acworker/1', 'parse', hashlib.sha256(source).hexdigest(), 'core', 'main',
                             '60000', '5000', '70000', scratch, 'reuse', self.old + '\\run-unit\\cache-store'))
        request['request_hash'] = hashlib.sha256(identity.encode()).hexdigest()
        artifacts, files = [], {}
        for name, raw in (('reconstructed.at', source if reconstructed is None else reconstructed),
                          ('outline.json', b'[]\n'), ('cst.canonical', b'unit opaque artifact')):
            path = prefix + '/work/' + name
            files[path] = raw
            artifacts.append({'kind': 'syntax', 'logical': name, 'path': self.old + '\\' + path.replace('/', '\\'),
                              'bytes': len(raw), 'sha256': hashlib.sha256(raw).hexdigest()})
        tools = dict.fromkeys(('lld_path', 'lld_sha256', 'sdk_um', 'sdk_um_sha256', 'ac_core_version',
                               'worker_version', 'backend'), '')
        tools['worker_exe_sha256'] = '1' * 64
        response = {'schema': 'acworker/1', 'nonce': request['nonce'], 'request_hash': request['request_hash'],
                    'ok': {'diagnostics': [{'stage': 'parse', 'code': 'syntax.parse', 'message': 'unit diagnostic',
                                          'span_start': 0, 'span_end': 1}] if diagnostic else [],
                           'hir_b64': None, 'run': None, 'artifacts': artifacts, 'toolchain': tools,
                           'trace': [{'stage': 'parse', 'wall_ms': 1}], 'metric_gaps': ['cpu_ms', 'io_bytes'],
                           'cache': {'mode': 'syntax:auto-source@core:revision-0', 'hits': [['grammar', 'auto-source@core']],
                                     'counters': [['reparsed_decls', 17]]}}}
        wire = {'protocol': 'acworker/1', 'recipe': 'syntax/auto-source@core', 'worker_sha256': '1'*64,
                'exit_code': 0, 'transport_error': None, 'stderr': b'',
                'start_path': prefix + '/worker-exchanges/1-0-start.json'}
        self.encode(wire, request, response)
        return self.write_copy(files), wire, expected, request, response

    def encode(self, wire, request, response):
        frames = [{'op': 'begin', 'schema': 'acworker/1'}, {'op': 'request', 'request': request}, {'op': 'end'}]
        wire['request'] = ('\n'.join(json.dumps(frame) for frame in frames) + '\n').encode()
        wire['stdout'] = json.dumps(response).encode()

    def test_byte_reconstruction_and_gaps_do_not_imply_incremental_claim(self):
        copy, wire, expected, _, _ = self.sample()
        result = syntax_evidence.syntax_parse_observation(copy, wire, expected)
        self.assertEqual(result['judgement'], 'PASS')
        self.assertEqual(result['counters'], [['reparsed_decls', 17]])
        self.assertEqual(result['metric_gaps'], ['cpu_ms', 'io_bytes'])
        self.assertIn('no incremental/performance claim', result['scope'])

    def test_expected_diagnostic_and_message_are_independent_oracles(self):
        copy, wire, expected, request, response = self.sample(diagnostic=True)
        self.assertEqual(syntax_evidence.syntax_parse_observation(copy, wire, expected)['judgement'], 'PASS')
        response['ok']['diagnostics'][0]['code'] = 'different.valid.code'
        self.encode(wire, request, response)
        self.assertEqual(syntax_evidence.syntax_parse_observation(copy, wire, expected)['judgement'], 'FAIL')

    def test_consistent_wrong_reconstruction_is_fail(self):
        copy, wire, expected, _, _ = self.sample(reconstructed=b'wrong bytes')
        self.assertEqual(syntax_evidence.syntax_parse_observation(copy, wire, expected)['judgement'], 'FAIL')

    def test_source_nonce_grammar_budget_and_frame_controls(self):
        for change in ('source', 'nonce', 'grammar', 'budget', 'frame'):
            copy, wire, expected, request, response = self.sample()
            if change == 'source': request['input']['source_b64'] = 'eA=='
            if change == 'nonce': response['nonce'] = 'foreign'
            if change == 'grammar': request['grammar'] = 'auto-source@scalar'
            if change == 'budget': request['budget']['run_ms'] = True
            self.encode(wire, request, response)
            if change == 'frame': wire['request'] = wire['request'].splitlines()[1]
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                syntax_evidence.syntax_parse_observation(copy, wire, expected)

    def test_diagnostic_span_unknown_counter_and_worker_identity_controls(self):
        for change in ('span', 'extra', 'counter-bool', 'worker'):
            copy, wire, expected, request, response = self.sample(diagnostic=True)
            if change == 'span': response['ok']['diagnostics'][0]['span_end'] = 1000
            if change == 'extra': response['ok']['unapproved'] = 0
            if change == 'counter-bool': response['ok']['cache']['counters'][0][1] = True
            if change == 'worker': response['ok']['toolchain']['worker_exe_sha256'] = '2'*64
            self.encode(wire, request, response)
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                syntax_evidence.syntax_parse_observation(copy, wire, expected)

    def test_artifact_namespace_and_content_integrity_controls(self):
        for change in ('owner', 'bytes'):
            copy, wire, expected, request, response = self.sample()
            if change == 'owner': response['ok']['artifacts'][0]['path'] = 'C:\\foreign\\reconstructed.at'
            else: (self.root / 'run-unit/one/check/work/reconstructed.at').write_bytes(b'changed after copy')
            self.encode(wire, request, response)
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                syntax_evidence.syntax_parse_observation(copy, wire, expected)



class BridgeEvidenceTests(unittest.TestCase):
    setUp = CaseCopyBindingTests.setUp
    write_copy = CaseCopyBindingTests.write_copy
    save_manifest = CaseCopyBindingTests.save_manifest

    def sample(self, diagnostic=None):
        import base64
        source = b'deck opaque-unit\n'
        expected = {'case':'one', 'check':'check', 'adapter':'probe', 'action':'run',
                    'recipe':'abi/ca-d1-bridge-v1', 'source_files':[{'sha256':hashlib.sha256(source).hexdigest()}],
                    'abi_options':{'leg':'source'}, 'abi_steps':[{'op':'query'}],
                    'probe_oracles':[{'cache':'published', 'compute_calls':5,
                                      'bindings_sha256':None, 'stages_include':['bridge']}],
                    'completion':{'kind':'success'}}
        request = {'recipe':expected['recipe'], 'nonce':'unit-nonce',
                   'deck_b64':base64.b64encode(source).decode(), 'leg':'source',
                   'store_root':self.old+'\\run-unit\\one\\check\\store', 'steps':[{'op':'query'}]}
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        meter={'hot_bytes':123,'scratch_peak_disk_bytes':456,'rejections':0,
               'gc_runs':0,'gc_deleted_chunks':0,'gc_freed_bytes':0}
        step={'op':'query','outcome':'ok','cache':'published','compute_calls':5,
              'stages':[{'name':'bridge','wall_ns':1299999}], 'meter':meter, 'pe_exit':0}
        if diagnostic:
            expected['completion']={'kind':'error','stage':'run','code':'pe-exit','message':None}
            step.update(outcome='diags',diagnostics=[{'code':diagnostic,'message':'unit failure'}],pe_exit=None)
        response={'schema':'abiworker/1','nonce':request['nonce'],'request_hash':request['request_hash'],
                  'ok':{'recipe':expected['recipe'],'steps':[step], 'restart_pending':False,'worker_version':'abi-worker/0.1.0'}}
        sample={'seq':0,'op':'query','ok':True,'edit_ms':0,'request_ms':7,
                'outcome':step['outcome'],'pe_exit':step['pe_exit'],'cache':'published','compute_calls':5,
                'stages':{'bridge':1}, 'meter':{('scratch_disk_bytes' if k=='scratch_peak_disk_bytes' else k):v for k,v in meter.items()},
                'peak_mem_bytes':1024,'peak_job_mem_bytes':2048,'io_read_bytes':32,'io_write_bytes':64,
                'error':None,'request_nonce':request['nonce']}
        wire={'protocol':'abiworker/1','recipe':expected['recipe'], 'exit_code':0,'transport_error':None,
              'stderr':b'', 'start_path':'run-unit/one/check/worker-exchanges/1-0-start.json'}
        return expected,request,response,sample,wire,source

    def admit(self, data):
        expected,request,response,sample,wire,source=data
        wire['request']=json.dumps(request,separators=(',',':')).encode()
        wire['stdout']=json.dumps(response,separators=(',',':')).encode()
        copy=self.write_copy({'run-unit/one/check/abi-request-samples.jsonl':json.dumps(sample).encode()+b'\n'})
        return abi_evidence.bridge_query_observations(copy,[wire],expected,source)

    def test_actual_query_and_parent_job_observations(self):
        result=self.admit(self.sample())
        self.assertEqual(result['judgement'],'PASS')
        self.assertEqual(result['queries'][0]['parent_job']['io_read_bytes'],32)
        self.assertIn('no restart',result['scope'])

    def test_independent_counter_oracle_rejects_consistent_wrong_echo(self):
        data=self.sample();data[2]['ok']['steps'][0]['compute_calls']=0;data[3]['compute_calls']=0
        self.assertEqual(self.admit(data)['judgement'],'FAIL')

    def test_expected_language_diagnostic_is_distinct_from_infra(self):
        self.assertEqual(self.admit(self.sample('pe-exit'))['judgement'],'PASS')
        self.assertEqual(self.admit(self.sample('tool-missing'))['judgement'],'FAIL')
        data=self.sample('pe-exit');data[2]['ok']['steps'][0]['outcome']='infra';data[3]['outcome']='infra'
        self.assertEqual(self.admit(data)['judgement'],'FAIL')

    def test_parent_samples_are_bound_to_nonce_sequence_and_raw_result(self):
        for key,value in [('request_nonce','foreign'),('seq',1),('compute_calls',99),('stages',{})]:
            data=self.sample();data[3][key]=value
            with self.subTest(key=key), self.assertRaises(m.AdmissionError):self.admit(data)

    def test_request_input_recipe_and_store_owner_cannot_be_foreign(self):
        for key,value in [('deck_b64','eA=='),('leg','typed-self'),('store_root','C:\\foreign'),('recipe','abi/interop-v1')]:
            data=self.sample();data[1][key]=value
            with self.subTest(key=key), self.assertRaises(m.AdmissionError):self.admit(data)

    def test_missing_job_counters_and_bool_numeric_values_are_refused(self):
        for key in ('peak_job_mem_bytes','io_read_bytes','request_ms'):
            for value in (None,True,-1):
                data=self.sample();data[3][key]=value
                with self.subTest(key=key,value=value),self.assertRaises(m.AdmissionError):self.admit(data)
        data=self.sample();data[2]['ok']['steps'][0]['compute_calls']=1<<32
        with self.assertRaises(m.AdmissionError):self.admit(data)

    def test_empty_queries_restart_and_unmatched_edit_are_not_admitted(self):
        for steps in ([],[{'op':'restart'}],[{'op':'edit-source','from':'absent','to':'x'},{'op':'query'}]):
            data=self.sample();data[0]['abi_steps']=steps
            with self.subTest(steps=steps),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_parent_rejection_and_incomplete_child_never_promoted(self):
        data=self.sample();data[3].update(ok=False,error='parent rejection')
        self.assertEqual(self.admit(data)['judgement'],'FAIL')
        for change in ('empty','extra','duplicate-stage','both-envelope'):
            data=self.sample()
            if change=='empty':data[2]['ok']['steps']=[]
            if change=='extra':data[2]['ok']['steps'][0]['unobserved']=0
            if change=='duplicate-stage':data[2]['ok']['steps'][0]['stages']*=2
            if change=='both-envelope':data[2]['error']={'code':'infra','message':'fault'}
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)


class BridgeStoreEvidenceTests(unittest.TestCase):
    """D937-final store seam (round 120): parent Job samples + store products."""
    setUp = CaseCopyBindingTests.setUp
    write_copy = CaseCopyBindingTests.write_copy
    save_manifest = CaseCopyBindingTests.save_manifest

    def sample(self, leg='source', outcome='ok'):
        source = b'deck opaque-unit\n'
        expected = {'case':'one', 'check':'check', 'adapter':'probe', 'action':'run',
                    'recipe':'abi/ca-d1-bridge-v1', 'source_files':[{'sha256':hashlib.sha256(source).hexdigest()}],
                    'abi_options':{'leg':leg}, 'abi_steps':[{'op':'query'}, {'op':'query'}],
                    'probe_oracles':[{'cache':'published', 'compute_calls':5,
                                      'bindings_sha256':None, 'stages_include':['bridge']},
                                     {'cache':'hit', 'compute_calls':0,
                                      'bindings_sha256':None, 'stages_include':[]}],
                    'completion':{'kind':'success'}}
        if outcome == 'diags':
            expected['completion'] = {'kind':'error','stage':'parse','code':'deck','message':None}
            expected['probe_oracles'] = [{'cache':None, 'compute_calls':None,
                                          'bindings_sha256':None, 'stages_include':[]}]
            expected['abi_steps'] = [{'op':'query'}]
        def row(seq, cache, calls, out):
            return {'seq':seq,'op':'query','ok':True,'edit_ms':0,'request_ms':9,
                    'outcome':out,'pe_exit':0 if out=='ok' else None,'cache':cache,'compute_calls':calls,
                    'stages':{'bridge':1} if out=='ok' else {}, 'meter':{'hot_bytes':123,'scratch_disk_bytes':456,
                    'rejections':0,'gc_runs':0,'gc_deleted_chunks':0,'gc_freed_bytes':0},
                    'peak_mem_bytes':1024,'peak_job_mem_bytes':2048,'io_read_bytes':32,'io_write_bytes':64,
                    'error':None}
        samples = ([row(0,'published',5,'ok'), row(1,'hit',0,'ok')] if outcome=='ok'
                   else [row(0,'-',0,'diags')])
        files = {f'run-unit/one/check/abi-request-samples.jsonl':
                 ''.join(json.dumps(s,separators=(',',':'))+'\n' for s in samples).encode(),
                 'run-unit/one/check/store/journal.db': b'unit-journal'}
        if leg == 'source':
            for name in ('generated.c','a2x_c_runtime.h','bridge_source_caller.c','generated.obj'):
                files[f'run-unit/one/check/store/bridge-scratch/bridge-937-1-run/{name}'] = b'unit-' + name.encode()
        else:
            lane = {'typed-self':'self','typed-n2c':'n2c','typed-c2n':'c2n'}[leg]
            files[f'run-unit/one/check/store/bridge-scratch/bridge-937-1-typed/{lane}/caller.obj'] = b'unit-obj'
        return expected, samples, files, source

    def admit(self, expected, files, source):
        return abi_evidence.bridge_store_query_observations(self.write_copy(files), expected, source)

    def test_store_pass_observation(self):
        expected, samples, files, source = self.sample()
        result = self.admit(expected, files, source)
        self.assertEqual(result['judgement'],'PASS')
        self.assertEqual(result['queries'][0]['parent_job']['io_read_bytes'],32)
        self.assertIn('store seam', result['scope'])
        self.assertEqual(result['schema'],'ace.m1.bridge-store-query-observation/1')

    def test_typed_lane_products(self):
        for leg in ('typed-self','typed-n2c','typed-c2n'):
            expected, _, files, source = self.sample(leg=leg)
            with self.subTest(leg=leg):
                self.assertEqual(self.admit(expected, files, source)['judgement'],'PASS')

    def test_counter_oracle_and_semantic_mismatches_are_fail(self):
        expected, samples, files, source = self.sample()
        samples[0]['compute_calls'] = 4; files = self.reserialize(expected, files, samples)
        self.assertEqual(self.admit(expected, files, source)['judgement'],'FAIL')
        expected, samples, files, source = self.sample()
        samples[0]['outcome'] = 'diags'; files = self.reserialize(expected, files, samples)
        self.assertEqual(self.admit(expected, files, source)['judgement'],'FAIL')

    def reserialize(self, expected, files, samples):
        files = dict(files)
        files['run-unit/one/check/abi-request-samples.jsonl'] = \
            ''.join(json.dumps(s,separators=(',',':'))+'\n' for s in samples).encode()
        return files

    def test_missing_store_surfaces_refused(self):
        for change in ('scratch','journal','both-runs'):
            expected, _, files, source = self.sample()
            if change == 'scratch':
                files = {k:v for k,v in files.items() if '/bridge-scratch/' not in k}
            if change == 'journal':
                files = {k:v for k,v in files.items() if not k.endswith('/store/journal.db')}
            if change == 'both-runs':
                files['run-two/one/check/abi-request-samples.jsonl'] = files['run-unit/one/check/abi-request-samples.jsonl']
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):
                self.admit(expected, files, source)

    def test_wrong_leg_lane_and_sample_shape_refused(self):
        expected, _, files, source = self.sample(leg='typed-self')
        bad = {k:v for k,v in files.items() if '/bridge-scratch/' not in k}
        bad['run-unit/one/check/store/bridge-scratch/bridge-937-1-typed/n2c/caller.obj'] = b'unit-obj'
        with self.assertRaises(m.AdmissionError):  # self lane products missing
            self.admit(expected, bad, source)
        expected, samples, files, source = self.sample()
        samples[0]['request_nonce'] = 'unobservable-in-store-seam'
        with self.assertRaises(m.AdmissionError):  # exchange-frame field is not a store-sample field
            self.admit(expected, self.reserialize(expected, files, samples), source)
        expected, samples, files, source = self.sample()
        samples = samples[:1]  # denominator mismatch: two declared queries, one sample
        with self.assertRaises(m.AdmissionError):
            self.admit(expected, self.reserialize(expected, files, samples), source)


class FirstSliceReductionTests(unittest.TestCase):
    def test_daily_parent_error_is_not_hidden_by_fresh_explicit_certification(self):
        observations={key:{'judgement':'PASS'} for key in first_slice.REQUIRED}
        report=first_slice.reduce_preview('1'*64,observations,{'accounting_valid':True,'can_claim_m1':False,'execution':'UNOBSERVED'})
        parents={'q4':{'report_sha256':'2'*64,'non_pass_rows':[{'case':'one','check':'a2c-emit','status':'ERROR'}]}}
        result=first_slice.retain_case_reports(report,parents)
        self.assertFalse(result['first_slice_evidence_complete'])
        self.assertFalse(result['gates']['source_case_reports'])
        self.assertIn('source_case_reports',result['blockers'])
        self.assertEqual(result['source_case_reports']['q4']['non_pass_rows'][0]['status'],'ERROR')

    def test_parent_rejection_is_never_promoted_by_matching_raw_value(self):
        for parent in ('FAIL','ERROR','SKIP'):
            result=first_slice.retain_parent_status({'judgement':'PASS'},parent)
            self.assertEqual(result['judgement'],'FAIL')
            self.assertEqual(result['parent_status'],parent)
        self.assertEqual(first_slice.retain_parent_status({'judgement':'FAIL'},'PASS')['judgement'],'FAIL')
        self.assertEqual(first_slice.retain_parent_status({'judgement':'PASS'},'PASS')['judgement'],'PASS')

    def records(self):
        observations={key:{'judgement':'PASS'} for key in first_slice.REQUIRED}
        legacy={'accounting_valid':True,'can_claim_m1':False,'execution':'UNOBSERVED'}
        return observations,legacy

    def test_complete_bounded_slice_still_cannot_claim_milestone(self):
        observations,legacy=self.records();v=first_slice.reduce_preview('1'*64,observations,legacy)
        self.assertTrue(v['first_slice_evidence_complete'])
        self.assertFalse(v['can_claim_expanded_m1'])
        self.assertFalse(v['acceptance_system_delivered'])
        self.assertIn('legacy_current_gate',v['blockers'])
        self.assertIn('legacy_required_execution',v['blockers'])

    def test_deleting_c_or_any_required_observation_breaks_slice_and(self):
        for key in first_slice.REQUIRED:
            observations,legacy=self.records();del observations[key]
            with self.subTest(key=key):
                v=first_slice.reduce_preview('1'*64,observations,legacy)
                self.assertFalse(v['first_slice_evidence_complete'])
                self.assertIn('first_slice_evidence',v['blockers'])

    def test_fail_unobserved_and_foreign_obligations_do_not_pass(self):
        for state in ('FAIL','UNOBSERVED'):
            observations,legacy=self.records();observations['q4/AV']['judgement']=state
            self.assertFalse(first_slice.reduce_preview('1'*64,observations,legacy)['first_slice_evidence_complete'])
        observations,legacy=self.records();observations['foreign']={'judgement':'PASS'}
        with self.assertRaises(m.AdmissionError):first_slice.reduce_preview('1'*64,observations,legacy)

    def test_forged_legacy_green_never_closes_full_registry_or_preview(self):
        observations,legacy=self.records();legacy.update(can_claim_m1=True,execution='PASS')
        v=first_slice.reduce_preview('1'*64,observations,legacy)
        self.assertFalse(v['gates']['complete_expanded_registry'])
        self.assertFalse(v['can_claim_expanded_m1'])


class TrackedOracleBindingTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self,key='q4',base=None):
        base=base or ROOT
        registry=json.loads((ROOT/first_slice.REGISTRY).read_bytes())
        frozen=registry['inventory_contracts'][key]
        inventory=json.loads(json.dumps(frozen))
        inventory['suite_path']=str(base/first_slice.SUITES[key])
        inventory['fixture_base']=str((base/first_slice.SUITES[key]).parent)
        for row in inventory['checks']:
            row['case_file']=str(base/row['case_file'])
            row['input_paths']=[str(base/path) for path in row['input_paths']]
        return frozen,inventory

    def admit(self,frozen,inventory,key='q4',inputs_root=None):
        path=self.root/'inventory.json';path.write_text(json.dumps(inventory))
        ref={'path':path.name,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}
        return first_slice._inventory(m.EvidenceScope(inputs_root or ROOT),m.EvidenceScope(self.root),ref,key,frozen)

    def test_inline_source_is_bound_to_the_actual_case_document(self):
        base=self.root/'repo';base.mkdir()
        registry=json.loads((ROOT/first_slice.REGISTRY).read_bytes())
        frozen=registry['inventory_contracts']['typed_av']
        names={first_slice.SUITES['typed_av']}
        for row in frozen['checks']:names.update([row['case_file'],*row['input_paths']])
        for name in names:
            dest=base/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
        frozen,inventory=self.sample('typed_av',base)
        self.assertEqual(len(self.admit(frozen,inventory,'typed_av',base)['checks']),18)
        inline=next(row for row in frozen['checks'] if row['source_origin']['kind']=='text')
        path=base/inline['case_file'];path.write_bytes(path.read_bytes()+b'\nchanged inline source document\n')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):
            self.admit(frozen,inventory,'typed_av',base)

    def test_portable_contract_snapshot_preserves_formal_current_files(self):
        frozen,inventory=self.sample()
        admitted=self.admit(frozen,inventory)
        self.assertEqual(len(admitted['checks']),4)
        self.assertEqual(first_slice.inventory_contract(m.EvidenceScope(ROOT),admitted),frozen)

    def test_rehashed_parent_oracle_and_recipe_cannot_override_tracked_contract(self):
        for field,value in [('expected_exit',43),('recipe','foreign/recipe'),('completion',{'kind':'error'})]:
            frozen,inventory=self.sample();inventory['checks'][0][field]=value
            with self.subTest(field=field),self.assertRaisesRegex(m.AdmissionError,'tracked independent typed contract'):
                self.admit(frozen,inventory)

    def test_source_relocation_and_missing_check_cannot_shrink_denominator(self):
        for change in ('source','missing'):
            frozen,inventory=self.sample()
            if change=='source':inventory['checks'][0]['input_paths'][0]=str(ROOT/'Cargo.toml')
            else:inventory['checks'].pop()
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(frozen,inventory)


class ObligationCatalogTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def case_rows(self):
        inputs=m.EvidenceScope(ROOT)
        return inputs,obligation_catalog.read_tsv(inputs,obligation_catalog.CASE_CATALOG,obligation_catalog.CASE_FIELDS)

    def test_catalogs_remain_discovery_and_preserve_all_configurations(self):
        result=obligation_catalog.admit_catalogs(m.EvidenceScope(ROOT))
        self.assertEqual(result['formal_cases']['checks'],511)
        self.assertEqual(len(result['formal_cases']['manual_suites']),1)
        self.assertEqual(result['scalar']['cases'],61)
        self.assertEqual(result['scalar']['declared_rust_runs'],69)
        self.assertEqual(result['scalar']['declared_c_runs'],61)
        self.assertEqual(result['scalar']['execution'],'UNOBSERVED')
        self.assertFalse(result['mapping_complete'])

    def test_missing_duplicate_fake_execution_and_unknown_group_are_refused(self):
        for change in ('missing','duplicate','execution','unknown-group','source','snapshot'):
            inputs,rows=self.case_rows()
            if change=='missing':rows.pop()
            if change=='duplicate':rows[1]=rows[0].copy()
            if change=='execution':rows[0]['execution']='PASS'
            if change=='unknown-group':rows[0]['group']='unregistered-other'
            if change=='source':rows[0]['case_file_sha256']='0'*64
            if change=='snapshot':
                refs=json.loads(rows[0]['snapshot_inputs']);refs[0]['sha256']='0'*64
                rows[0]['snapshot_inputs']=json.dumps(refs)
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):obligation_catalog.admit_case_catalog(inputs,rows)

    def test_release_and_trap_denominators_cannot_be_reduced(self):
        for change in ('release','trap','missing'):
            inputs=m.EvidenceScope(ROOT)
            rows=obligation_catalog.read_tsv(inputs,obligation_catalog.SCALAR_CATALOG,obligation_catalog.SCALAR_FIELDS)
            if change=='release':next(row for row in rows if row['rust_release_runs']=='1')['rust_release_runs']='0'
            if change=='trap':next(row for row in rows if row['oracle_kind']=='trap').update(oracle_kind='value',oracle_value='0')
            if change=='missing':rows.pop()
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):obligation_catalog.admit_scalar_catalog(inputs,rows)

    def test_rewritten_tracked_catalog_is_stale_before_admission(self):
        path=self.root/obligation_catalog.CASE_CATALOG;path.parent.mkdir(parents=True)
        path.write_bytes((ROOT/obligation_catalog.CASE_CATALOG).read_bytes()+b'\n')
        with self.assertRaisesRegex(m.AdmissionError,'STALE artifact content'):
            obligation_catalog.admit_catalogs(m.EvidenceScope(self.root))


class LegacyExecutionReductionTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.required=sorted(':'.join((row['ns'],row['id'],row['path']))
                            for row in legacy.frozen_ledger(m.EvidenceScope(ROOT)) if row['required'])

    def test_four_observations_do_not_promote_all_313(self):
        cells={key:{'judgement':'PASS'} for key in ('corpus:B08:ac','corpus:B08:av','corpus:B11:ac','corpus:B11:av')}
        result=legacy_execution.execution_summary(self.required,cells)
        self.assertEqual((result['observed'],result['passed'],len(result['unobserved_cells'])),(4,4,309))
        self.assertFalse(result['complete'])
        self.assertEqual(result['verdict'],'UNOBSERVED')

    def test_explicit_failure_remains_failed_with_missing_cells(self):
        result=legacy_execution.execution_summary(self.required,{'corpus:B08:av':{'judgement':'FAIL'}})
        self.assertEqual(result['verdict'],'FAIL')
        self.assertEqual(result['failed_cells'],['corpus:B08:av'])
        self.assertEqual(len(result['unobserved_cells']),312)

    def test_synthetic_complete_and_each_missing_cell_controls_are_isolated(self):
        cells={key:{'judgement':'PASS'} for key in self.required}
        self.assertTrue(legacy_execution.execution_summary(self.required,cells)['complete'])
        for missing in self.required:
            with self.subTest(missing=missing):
                result=legacy_execution.execution_summary(self.required,{k:v for k,v in cells.items() if k!=missing})
                self.assertFalse(result['complete'])

    def test_foreign_cells_changed_denominators_and_bool_verdicts_are_refused(self):
        for required,cells in [(self.required[:-1],{}),(self.required[:-1]+[self.required[0]],{}),
                               (self.required,{'other':{'judgement':'PASS'}}),
                               (self.required,{'corpus:B08:ac':{'judgement':True}})]:
            with self.assertRaises(m.AdmissionError):legacy_execution.execution_summary(required,cells)

    def test_new_fixture_sources_are_exact_original_approved_bytes(self):
        manifest,_=m.EvidenceScope(ROOT).read_json(legacy.CORPUS+'source-manifest.json')
        for name in ('B08','B11'):
            relative='crates/auto-lang/test/cases/ace-m1-expanded/src/legacy-'+name+'.at'
            m.EvidenceScope(ROOT).sha256(relative,manifest['corpus_src']['files'][name+'.at'])


class ParentContractAdmissionTests(unittest.TestCase):
    """Capture/schema units use inert producer bytes, never execution evidence."""
    setUp = EvidenceAdmissionTests.setUp

    def sample(self):
        self.inputs=self.root/'repository';self.evidence=self.root/'evidence'
        self.inputs.mkdir();self.evidence.mkdir()
        frozen=json.loads((ROOT/first_slice.REGISTRY).read_bytes())['parent_contracts']
        for path in parent_contracts.SOURCES:
            dest=self.inputs/path;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/path,dest)
        binary=self.inputs/parent_contracts.PRODUCER;binary.parent.mkdir(parents=True)
        binary.write_bytes(b'inert unit producer; not an executable')
        argv=[str(binary),'parent-inventory']
        record={'schema':'test-parent-contract-capture/1','argv':argv,'cwd':str(self.inputs),'exit':0,
                'producer_sha256':hashlib.sha256(binary.read_bytes()).hexdigest(),
                'meter':{'argv':argv,'exit':0,'error':None,'wall_ms':1.0,'workload_cpu_ms':1.0,
                         'meter_cpu_ms':1.0,'total_cpu_ms':2.0,'peak_process_bytes':4096,
                         'peak_job_bytes':8192,'job_processes':2}}
        return frozen,json.loads(json.dumps(frozen['snapshot'])),record

    def admit(self,data):
        frozen,snapshot,record=data
        for role,raw in [('stdout',json.dumps(snapshot).encode()),('stderr',b'')]:
            name=role+'.raw';(self.evidence/name).write_bytes(raw)
            record[role]={'path':name,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest()}
        (self.evidence/'capture.json').write_text(json.dumps(record))
        return parent_contracts.admit_parent_contracts(m.EvidenceScope(self.inputs),m.EvidenceScope(self.evidence),'capture.json',frozen)

    def test_original_declared_tables_and_limits_are_not_execution(self):
        result=self.admit(self.sample())
        self.assertEqual(result['status'],'DISCOVERY');self.assertEqual(result['executed'],0)
        self.assertEqual(len(result['tables']),11);self.assertEqual(sum(result['tables'].values()),43)
        self.assertEqual(result['limits']['av']['default_session_wall_ms'],60000)
        self.assertEqual(result['limits']['native']['default_session_wall_ms'],120000)
        self.assertEqual(result['limits']['abi']['session_wall_ms'],60000)

    def test_changed_binary_is_stale_without_implicit_preparation(self):
        data=self.sample();(self.inputs/parent_contracts.PRODUCER).write_bytes(b'changed producer')
        from unittest.mock import patch
        with patch('subprocess.Popen',side_effect=AssertionError('no implicit preparation')):
            with self.assertRaisesRegex(m.AdmissionError,'STALE'):self.admit(data)

    def test_matching_payload_does_not_rescue_changed_parent_source(self):
        data=self.sample();path=self.inputs/parent_contracts.SOURCES[0];path.write_bytes(path.read_bytes()+b'\n')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):self.admit(data)

    def test_rehashed_forged_limits_and_reduced_oracles_are_refused(self):
        data=self.sample()
        original=json.loads(json.dumps(data[1]))
        for change in ('budget-bool','table-row','executed','protocol'):
            snapshot=json.loads(json.dumps(original))
            if change=='budget-bool':snapshot['parents']['av']['limits']['default_session_wall_ms']=True
            if change=='table-row':snapshot['parents']['av']['tables']['resource_sweep']['rows'].pop()
            if change=='executed':snapshot['executed']=1
            if change=='protocol':snapshot['parents']['av']['protocol']='foreign/1'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit((data[0],snapshot,data[2].copy()))

    def test_non_discovery_invocation_and_corrupt_measurement_are_refused(self):
        frozen,snapshot,original=self.sample()
        for change in ('invocation','bool-exit','meter-error','cpu-sum','bool-memory'):
            record=json.loads(json.dumps(original))
            if change=='invocation':record['argv'][-1]='run'
            if change=='bool-exit':record['exit']=False
            if change=='meter-error':record['meter']['error']='deadline'
            if change=='cpu-sum':record['meter']['total_cpu_ms']=0
            if change=='bool-memory':record['meter']['peak_job_bytes']=True
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit((frozen,snapshot,record))


class AVTypedEvidenceTests(unittest.TestCase):
    """Typed schema/oracle units; actual AV execution is recorded separately."""
    setUp = CaseCopyBindingTests.setUp
    write_copy = CaseCopyBindingTests.write_copy
    save_manifest = CaseCopyBindingTests.save_manifest

    def sample(self,fixture='i64-min',kind='i64',bits='8000000000000000'):
        import base64
        source=b'opaque typed request carrier\n';entry='typed:'+fixture
        oracle={'fixture':fixture,'kind':kind,'bits':bits}
        expected={'case':'one','check':'check','state':'RUN','adapter':'probe','action':'run',
                  'recipe':'ac-av/data-run-v1','completion':{'kind':'success'},
                  'probe_bindings':{'av':{'entry':entry,'caps':[],'session':None,'profile':None,'resource':None,'invalidation':None}},
                  'declared_probe_steps':[{'op':'run'}], 'source_files':[{'sha256':hashlib.sha256(source).hexdigest()}]}
        request={'recipe':expected['recipe'],'nonce':'typed-unit','source_b64':base64.b64encode(source).decode(),
                 'entry':entry,'caps':[],'store_root':self.old+'\\run-unit\\one\\check\\store',
                 'steps':[{'op':'run'}],'typed_fixture':fixture}
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        step={'op':'run','outcome':'success','value':None,'typed':{'kind':kind,'bits':bits},'trap':None,'diag':None,
              'source_hash':scalar_evidence._fnv1a64(entry.encode()),'wall_ms':0.5}
        response={'schema':'avworker/1','nonce':request['nonce'],'request_hash':request['request_hash'],
                  'ok':{'recipe':expected['recipe'],'steps':[step],'worker_version':'avworker/1.0','av_core_version':'av.data/1'}}
        wire={'protocol':'avworker/1','recipe':expected['recipe'],'exit_code':0,'transport_error':None,'stderr':b'',
              'start_path':'run-unit/one/check/worker-exchanges/1-0-start.json'}
        return expected,request,response,wire,oracle,source

    def admit(self,data):
        expected,request,response,wire,oracle,_=data
        wire['request']=json.dumps(request,separators=(',',':')).encode();wire['stdout']=json.dumps(response,separators=(',',':')).encode()
        return av_typed_evidence.av_typed_observation(self.write_copy(),wire,expected,oracle)

    def test_exact_bits_and_tag_are_preserved_without_source_claim(self):
        result=self.admit(self.sample())
        self.assertEqual(result['judgement'],'PASS');self.assertEqual(result['input_kind'],'typed-fixture')
        self.assertEqual(result['observed'],{'kind':'i64','bits':'8000000000000000'})
        self.assertIn('no wide source',result['scope']);self.assertIn('producer-counters',result['metric_gaps'])

    def test_equal_bits_do_not_authorize_a_different_type(self):
        data=self.sample();data[2]['ok']['steps'][0]['typed']['kind']='f64'
        self.assertEqual(self.admit(data)['judgement'],'FAIL')

    def test_nan_payload_is_compared_as_bits_including_sign(self):
        for fixture,kind,bits in [('fneg-nan','f64','fff80000deadbeef'),('f32-nan-payload','f32','7f80ad00')]:
            data=self.sample(fixture,kind,bits);self.assertEqual(self.admit(data)['judgement'],'PASS')
            data[2]['ok']['steps'][0]['typed']['bits']=bits[:-1]+'1'
            self.assertEqual(self.admit(data)['judgement'],'FAIL')

    def test_numeric_bool_and_noncanonical_bit_encodings_are_refused(self):
        for bits in (0,True,-0.0,'0x8000','ABC','f'*17,''):
            data=self.sample();data[2]['ok']['steps'][0]['typed']['bits']=bits
            with self.subTest(bits=bits),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_request_carrier_entry_fixture_nonce_and_namespace_are_bound(self):
        for change in ('carrier','entry','fixture','nonce','owner','source-hash'):
            data=self.sample()
            if change=='carrier':data[1]['source_b64']='eA=='
            if change=='entry':data[1]['entry']='d_main'
            if change=='fixture':data[1]['typed_fixture']='u64-max'
            if change=='nonce':data[2]['nonce']='other'
            if change=='owner':data[1]['store_root']='C:\\foreign'
            if change=='source-hash':data[2]['ok']['steps'][0]['source_hash']=scalar_evidence._fnv1a64(data[5])
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_value_fallback_unknown_fields_and_failed_transport_do_not_pass(self):
        for change in ('value','unknown','extra-step','stderr','infra','bool-wall'):
            data=self.sample();step=data[2]['ok']['steps'][0]
            if change=='value':step.update(value=42,typed=None)
            if change=='unknown':step['unobserved_counter']=0
            if change=='extra-step':data[2]['ok']['steps']*=2
            if change=='stderr':data[3]['stderr']=b'worker panic'
            if change=='infra':step['outcome']='infra'
            if change=='bool-wall':step['wall_ms']=False
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)



class NativeTypedEvidenceTests(unittest.TestCase):
    setUp = CaseCopyBindingTests.setUp
    write_copy = CaseCopyBindingTests.write_copy
    save_manifest = CaseCopyBindingTests.save_manifest

    def sample(self):
        import base64
        check=json.loads((ROOT/first_slice.REGISTRY).read_bytes())['inventory_contracts']['typed_native']['checks'][0]
        check=json.loads(json.dumps(check));check['case']='one';check['check']='check'
        source=(ROOT/check['input_paths'][0]).read_bytes()
        request={'recipe':check['recipe'],'nonce':'native-typed-unit',
                 'typed_fixture_b64':base64.b64encode(source).decode(),'entry':'main','args':[],
                 'store_root':self.old+'\\run-unit\\one\\check\\mn-typed-store',
                 'workdir':self.old+'\\run-unit\\one\\check\\mn-typed-work'}
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        sig=int(scalar_evidence._fnv1a64(b'()->i64'),16)&0xffffffff
        ok={'recipe':check['recipe'],'worker_version':'mnworker/1.0','mn_abi':'msvc-x64/priv-typed-data-v1',
            'mn_target':'x86_64-pc-windows-msvc','counters':dict.fromkeys(native_typed_evidence.COUNTERS,0),
            'manifest_fp':'a'*16,'exit_code':0,'stderr_hex':'',
            'typed':{'status':'value','nonce':0,'sig_id':sig,'kind':'i64','bits':'8000000000000000'}}
        ok['counters']['run_invocations']=1
        response={'schema':'mnworker/1','nonce':request['nonce'],'request_hash':request['request_hash'],'ok':ok}
        wire={'protocol':'mnworker/1','recipe':check['recipe'],'exit_code':0,'transport_error':None,'stderr':b'',
              'start_path':'run-unit/one/check/worker-exchanges/1-0-start.json'}
        data=check,request,response,wire
        self.frame(data);return data

    def frame(self,data,**changes):
        import struct
        t=data[2]['ok']['typed'];fields={'magic':0x4d4e5231,'version':1,'sig':t['sig_id'],'status':0,
             'nonce':t['nonce'],'kind':next(k for k,v in native_typed_evidence.KINDS.items() if v==t['kind']),
             'bits':int(t['bits'],16)};fields.update(changes)
        raw=struct.pack('<IIIIQIQ',*fields.values());raw+=int(scalar_evidence._fnv1a64(raw),16).to_bytes(8,'little')
        data[2]['ok']['stdout_hex']=raw.hex()

    def admit(self,data):
        check,request,response,wire=data
        wire['request']=json.dumps(request,separators=(',',':')).encode()
        wire['stdout']=json.dumps(response,separators=(',',':')).encode()
        return native_typed_evidence.native_typed_observation(self.write_copy(),wire,check)

    def test_original_frame_and_private_abi_do_not_claim_source_or_freshness(self):
        result=self.admit(self.sample());self.assertEqual(result['judgement'],'PASS')
        self.assertEqual(result['observed'],{'kind':'i64','bits':'8000000000000000'})
        self.assertEqual(result['frame_nonce'],0);self.assertIn('no wide source',result['scope'])

    def test_consistently_wrong_frame_and_echo_still_fail_independent_oracle(self):
        for kind,bits in [('f64','8000000000000000'),('i64','8000000000000001')]:
            data=self.sample();data[2]['ok']['typed'].update(kind=kind,bits=bits);self.frame(data)
            self.assertEqual(self.admit(data)['judgement'],'FAIL')

    def test_machine_bytes_cannot_be_replaced_by_a_pass_looking_json_value(self):
        for change in ('echo','empty','short','extra','checksum','uppercase'):
            data=self.sample();ok=data[2]['ok']
            if change=='echo':ok['typed']['bits']='0'
            if change=='empty':ok['stdout_hex']=''
            if change=='short':ok['stdout_hex']=ok['stdout_hex'][:-2]
            if change=='extra':ok['stdout_hex']+='00'
            if change=='checksum':ok['stdout_hex']=ok['stdout_hex'][:-2]+'00'
            if change=='uppercase':ok['stdout_hex']=ok['stdout_hex'].upper()
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_rehashed_wrong_frame_identity_and_outcome_are_refused(self):
        for change in ({'magic':0},{'version':2},{'sig':0},{'nonce':1},{'status':1},{'kind':99}):
            data=self.sample();self.frame(data,**change)
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_fixture_args_directories_nonce_and_protocol_are_bound(self):
        for change in ('fixture','args','workdir','store','nonce','protocol','abi'):
            data=self.sample()
            if change=='fixture':data[1]['typed_fixture_b64']='eA=='
            if change=='args':data[1]['args']=[['i64','1']]
            if change=='workdir':data[1]['workdir']='C:\\other'
            if change=='store':data[1]['store_root']='C:\\other'
            if change=='nonce':data[2]['nonce']='other'
            if change=='protocol':data[3]['protocol']='avworker/1'
            if change=='abi':data[2]['ok']['mn_abi']='public-c-abi'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_missing_extra_bool_overflow_or_no_execution_counters_are_refused(self):
        for change in ('missing','extra','bool','overflow','zero','two'):
            data=self.sample();c=data[2]['ok']['counters']
            if change=='missing':c.pop('unit_prepare')
            if change=='extra':c['invented']=0
            if change=='bool':c['cache_hits']=False
            if change=='overflow':c['cache_hits']=1<<64
            if change=='zero':c['run_invocations']=0
            if change=='two':c['run_invocations']=2
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_nonzero_exit_stderr_infrastructure_and_unknown_fields_refuse(self):
        for change in ('worker-exit','child-exit','stderr','infra','unknown'):
            data=self.sample()
            if change=='worker-exit':data[3]['exit_code']=1
            if change=='child-exit':data[2]['ok']['exit_code']=70
            if change=='stderr':data[2]['ok']['stderr_hex']='00'
            if change=='infra':data[3]['transport_error']='deadline'
            if change=='unknown':data[2]['ok']['unobserved_metric']=0
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)



class LegacyBaseValueTests(unittest.TestCase):
    """Synthetic packet controls; original 97-check execution is separate."""
    setUp = EvidenceAdmissionTests.setUp

    def sample(self,name='B01',target='av',value=None):
        import base64,copy
        required,contracts=legacy_base.value_contracts(m.EvidenceScope(ROOT))
        contract=contracts[name]
        raw=json.loads((ROOT/legacy_base.CONTRACT).read_bytes())
        check=copy.deepcopy(next(x for x in raw['checks'] if x['case']=='ace.m1.'+name.lower()
                                and x['check']==('av' if target=='av' else 'ac-native')))
        check['input_paths']=[str(ROOT/p) for p in check['input_paths']]
        wire,_=ScalarObservationTests.wire(self,target,contract['value'] if value is None else value)
        request=json.loads(wire['request']);response=json.loads(wire['stdout']);source=(ROOT/contract['source_file']).read_bytes()
        request['source_b64']=base64.b64encode(source).decode();request.pop('request_hash')
        request['request_hash']=hashlib.sha256(json.dumps(request,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
        response['request_hash']=request['request_hash']
        if target=='av':response['ok']['steps'][0]['source_hash']=scalar_evidence._fnv1a64(source)
        wire.update(request=json.dumps(request).encode(),stdout=json.dumps(response).encode())
        return wire,check,contract

    def judge(self,wire,check,contract,*,name='B01',target='av',parent='PASS'):
        return legacy_base.value_observation(m.EvidenceScope(ROOT),self.scope,wire,check,parent,
                                            name,'av' if target=='av' else 'ac',contract)['judgement']

    def test_parent_pass_and_same_wrong_values_cannot_supply_original_oracle(self):
        for name in ('B01','B12'):
            for target in ('av','native'):
                wire,check,contract=self.sample(name,target)
                self.assertEqual(self.judge(wire,check,contract,name=name,target=target),'PASS')
                self.assertEqual(self.judge(wire,check,contract,name=name,target=target,parent='FAIL'),'FAIL')
                wire,check,contract=self.sample(name,target,contract['value']+1)
                self.assertEqual(self.judge(wire,check,contract,name=name,target=target),'FAIL')
                check['expected_exit']+=1
                with self.assertRaises(m.AdmissionError):self.judge(wire,check,contract,name=name,target=target)

    def test_original_source_target_entry_caps_and_run_contract_are_bound(self):
        for change in ('source','path','recipe','entry','caps','steps','skip','emit'):
            wire,check,contract=self.sample()
            if change=='source':check['source_files'][0]['sha256']='0'*64
            if change=='path':check['input_paths']=[str(ROOT/contract['source_file']).replace('B01.at','B02.at')]
            if change=='recipe':check['recipe']='ac-a2r/hir-scalar-emit-v1'
            if change=='entry':check['probe_bindings']['av']['entry']='other'
            if change=='caps':check['probe_bindings']['av']['caps']=['hir.test.trace.v1']
            if change=='steps':check['declared_probe_steps']=[]
            if change=='skip':check['state']='SKIP'
            if change=='emit':check['action']='emit'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(wire,check,contract)

    def test_original_required_set_and_scenario_cert_gaps_closed_by_domain_reception(self):
        required,contracts=legacy_base.value_contracts(m.EvidenceScope(ROOT))
        self.assertEqual(len(required),313);self.assertEqual(len(contracts),10)
        self.assertEqual(contracts['B01']['value'],14);self.assertEqual(contracts['B12']['value'],-2)
        for name in ('B08','B11','B13','B16','I01','W01'):self.assertNotIn(name,contracts)
        data=json.loads((ROOT/legacy_obligations.MAP).read_bytes());cells={r['cell']:r for r in data['cells']}
        self.assertEqual(sum(c['reader_status']=='supported-slice' for c in cells.values()),102)
        for key in ('corpus:B13:ac','scenario:I01:av','corpus:B01:a2r'):
            self.assertEqual(cells[key]['reader_status'],'pending');self.assertEqual(cells[key]['execution'],'UNOBSERVED')
        # PLAN-992..998 reception (941 round 120): 38 -> 0 — the six
        # resource-scenario cells carry ok-emit-scene, no cert-gap remains.
        self.assertEqual(sum('legacy-a2r-cert-gap' in c['gaps'] for c in cells.values()),0)
        self.assertEqual(sum(c['token']=='ok-emit-scene' for c in cells.values()),6)
        self.assertFalse(data['mapping_complete'])


class LegacyBaseCompletionTests(unittest.TestCase):
    """Typed completion controls; synthetic packets are never execution receipts."""
    setUp=EvidenceAdmissionTests.setUp

    def sample(self,name,target='av'):
        import base64,copy
        contract=legacy_base.other_contracts(m.EvidenceScope(ROOT))[name]
        inv=json.loads((ROOT/legacy_base.CONTRACT).read_bytes())
        check=copy.deepcopy(next(c for c in inv['checks'] if c['case']=='ace.m1.'+name.lower() and c['check']==('av' if target=='av' else 'ac-native')))
        check['input_paths']=[str(ROOT/p) for p in check['input_paths']]
        wire,_=ScalarObservationTests.wire(self,target,70)
        request=json.loads(wire['request']);response=json.loads(wire['stdout']);source=(ROOT/contract['source_file']).read_bytes();digest=hashlib.sha256(source).hexdigest()
        request['source_b64']=base64.b64encode(source).decode();request.pop('request_hash')
        trap=name in legacy_base.TRAP_CASES
        if target=='av':
            step=response['ok']['steps'][0];step['value']=step['channel']=None;step['source_hash']=scalar_evidence._fnv1a64(source)
            step['outcome']='trap' if trap else 'diag'
            step['trap']=scalar_q4.TRAPS[contract['oracle_value']] if trap else None
            step['diag']=None if trap else dict(stage='frontend',code=contract['oracle_value'],message='unit rejection')
            if not trap:step['counters']={'frontend_reject':1,'requests':1};step['spans']=[['frontend',1]]
        elif trap:
            response['ok']['counters']={k:0 for k in text_q4.NATIVE_COUNTERS}
            response['ok']['counters'].update(link_invocations=1,run_invocations=1)
            response['ok']['spans']={k:0 for k in text_q4.NATIVE_SPANS}
        else:
            ok=response['ok'];response['ok']={k:ok[k] for k in ('recipe','worker_version','mn_abi','mn_target')}
            response['ok'].update(counters={'frontend_reject':1},spans={'frontend':1})
            response['diagnostic']=dict(code='mn.unsupported.frontend',message='untyped message is not an oracle')
            if name!='B16':
                request['diagnostics']='source-v1';response['ok'].update(diagnostics_mode='source-v1',diagnostics_schema='mn.source-diagnostic/1')
                stage,code=legacy_base.DIAGNOSTICS[name][:2]
                response['source_error']=dict(schema='mn.source-diagnostic/1',mode='source-v1',producer='ace-core.source',profile='scalar',attempted=['scalar'],surface_rejections=[],path='mn.at',source_sha256=digest,source_len=len(source),span_basis='utf8-byte',primary=0,causes=[dict(stage=stage,code=code,message='unit rejection',span=legacy_base.diagnostic_span(source,name))],operation=dict(kind='cold',step=None,base_sha256=None,candidate_sha256=digest,session_revision=None,edit=None))
                response['diagnostic']['message']=stage.capitalize()+'/'+code+': unit rejection'
        request['request_hash']=hashlib.sha256(json.dumps(request,ensure_ascii=False,separators=(',',':')).encode()).hexdigest();response['request_hash']=request['request_hash']
        wire.update(request=json.dumps(request).encode(),stdout=json.dumps(response).encode())
        return wire,check,contract

    def judge(self,name,wire,check,contract,target='av'):
        return legacy_base.nonvalue_observation(m.EvidenceScope(ROOT),self.scope,wire,check,'PASS',name,'av' if target=='av' else 'ac',contract)['judgement']

    def test_named_av_traps_and_codes_are_not_interchangeable(self):
        for name in legacy_base.TRAP_CASES:
            wire,check,contract=self.sample(name);self.assertEqual(self.judge(name,wire,check,contract),'PASS')
            response=json.loads(wire['stdout']);response['ok']['steps'][0]['trap']='av.trap.rem-overflow' if name!='B15' else 'av.trap.int-overflow';wire['stdout']=json.dumps(response).encode()
            self.assertEqual(self.judge(name,wire,check,contract),'FAIL')
        for name in legacy_base.DIAGNOSTICS:
            wire,check,contract=self.sample(name);self.assertEqual(self.judge(name,wire,check,contract),'PASS')
            response=json.loads(wire['stdout']);response['ok']['steps'][0]['diag']['code']='wrong-code';wire['stdout']=json.dumps(response).encode()
            self.assertEqual(self.judge(name,wire,check,contract),'FAIL')

    def test_native_generic_exit_and_rendered_refusal_never_supply_named_authority(self):
        for name in legacy_base.TRAP_CASES:
            wire,check,contract=self.sample(name,'native');self.assertEqual(self.judge(name,wire,check,contract,'native'),'UNOBSERVED')
        wire,check,contract=self.sample('B16','native')
        self.assertEqual(self.judge('B16',wire,check,contract,'native'),'UNOBSERVED')
        response=json.loads(wire['stdout']);response['diagnostic']['message']='Bind/source.undefined: exact expected words';wire['stdout']=json.dumps(response).encode()
        self.assertEqual(self.judge('B16',wire,check,contract,'native'),'UNOBSERVED')

    def test_native_typed_primary_stage_span_and_operation_are_independently_bound(self):
        positions={'B17':dict(start=23,end=27),'B18':dict(start=38,end=43),'B19':dict(start=3,end=7)}
        for name in ('B17','B18','B19'):
            source=(ROOT/legacy_base.other_contracts(m.EvidenceScope(ROOT))[name]['source_file']).read_bytes()
            self.assertEqual(legacy_base.diagnostic_span(source,name),positions[name])
            wire,check,contract=self.sample(name,'native');self.assertEqual(self.judge(name,wire,check,contract,'native'),'PASS')
            with patch.object(legacy_base,'diagnostic_span',return_value=None),self.assertRaises(m.AdmissionError):
                self.judge(name,wire,check,contract,'native')
            response=json.loads(wire['stdout']);response['source_error']['causes'][0]['span']['end']+=1;wire['stdout']=json.dumps(response).encode()
            self.assertEqual(self.judge(name,wire,check,contract,'native'),'FAIL')
            for mutation in ('primary','operation','profile'):
                wire,check,contract=self.sample(name,'native');response=json.loads(wire['stdout'])
                if mutation=='primary':response['source_error']['primary']=False
                if mutation=='operation':response['source_error']['operation']['base_sha256']='0'*64
                if mutation=='profile':response['source_error']['profile']='record'
                wire['stdout']=json.dumps(response).encode()
                with self.subTest(case=name,mutation=mutation),self.assertRaises(m.AdmissionError):self.judge(name,wire,check,contract,'native')

    def test_legacy_vm_value_text_is_numeric_evidence_not_type_or_trap_authority(self):
        for text,expected in [(b'14',14),(b'-2',-2),(b'70',70)]:
            row=legacy_base.vm_result_observation(text,b'',expected);self.assertEqual(row['judgement'],'PASS');self.assertFalse(row['type_or_bits_authority'])
            self.assertEqual(legacy_base.vm_result_observation(text,b'',expected+1)['judgement'],'FAIL')
        for text in (b'14\n',b'+14',b'014',b'True',b'1.0',b'NaN',b'\xff',b'2147483648'):
            with self.subTest(text=text),self.assertRaises(m.AdmissionError):legacy_base.vm_result_observation(text,b'',14)

    def test_legacy_completion_switches_are_finite_and_exclusive(self):
        wire,check,contract=self.sample('B17');response=json.loads(wire['stdout'])
        for bad in (1,None,'yes'):
            with self.subTest(flag=bad),self.assertRaises(m.AdmissionError):scalar_q4.scalar_q4_observation(self.scope,wire,check,contract,legacy_base=bad)
        with self.assertRaises(m.AdmissionError):scalar_q4.scalar_q4_observation(self.scope,wire,check,contract,query_family=True,legacy_base=True)
        with self.assertRaises(m.AdmissionError):query_evidence.native_source_diagnostic(response,b'',contract,data_family=True,legacy_base=True)


class LegacyVmFailureTests(unittest.TestCase):
    """Synthetic persisted products; no execution credit from these controls."""
    setUp=EvidenceAdmissionTests.setUp

    def sample(self,name='B14',actual_change=None,raw=None,owner=None):
        import copy
        contract=legacy_base.vm_failure_contracts(m.EvidenceScope(ROOT))[name]
        inv=json.loads((ROOT/legacy_base.CONTRACT).read_bytes())
        check=copy.deepcopy(next(c for c in inv['checks'] if c['case']=='ace.m1.'+name.lower() and c['check']=='autovm'))
        check['input_paths']=[str(ROOT/p) for p in check['input_paths']]
        actual=dict(schema='autolang.avm-failure/1',adapter='avm',action='run',outcome='language-error',
                    source_sha256=check['source_sha256'],entry='main.at',entry_sha256=contract['source_sha256'],
                    stage=contract['stage'],code=contract['code'])
        if actual_change:actual.update(actual_change)
        raw=json.dumps(actual).encode() if raw is None else raw
        relative='run-unit/'+(owner or check['case']+'/autovm')+'/failure.json'
        path=self.root/relative;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
        ref=dict(path=relative,bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
        manifest=dict(schema='test-case-artifact-copy/1',original_runtime_root='D:/unit-runtime',
                      files=[ref],runtime_cleanup_complete=True,note='unit control, not a run')
        (self.root/'artifact-copy.json').write_text(json.dumps(manifest))
        product=dict(path='failure.json',stored='D:/unit-runtime/'+relative,sha256=ref['sha256'],bytes=len(raw))
        row=dict(status='PASS',artifacts=[product])
        return case_evidence.CaseCopy(m.EvidenceScope(self.root)),check,row,contract

    def judge(self,copy,check,row,contract,name='B14'):
        return legacy_base.vm_failure_observation(m.EvidenceScope(ROOT),copy,check,row,name,contract)

    def test_all_five_bind_original_codes_and_preserve_parent_rejection(self):
        for name in legacy_base.VM_FAILURES:
            args=self.sample(name);item=self.judge(*args,name=name)
            self.assertEqual(item['judgement'],'PASS')
            self.assertFalse(item['semantic_borrow_rule_authority'])
            # PLAN-998 reception: K04 rejects at the designed typed channel too.
            self.assertEqual(item['authority'],'typed-language-failure')
            for status in ('FAIL','ERROR','SKIP'):
                args[2]['status']=status
                self.assertEqual(self.judge(*args,name=name)['judgement'],'FAIL')

    def test_actual_stage_and_code_are_compared_without_message_fallback(self):
        for name in legacy_base.VM_FAILURES:
            for change in ({'code':'wrong-code'},{'stage':'run' if legacy_base.VM_FAILURES[name][2]!='run' else 'parse'}):
                with self.subTest(name=name,change=change):self.assertEqual(self.judge(*self.sample(name,change),name=name)['judgement'],'FAIL')
            item=self.judge(*self.sample(name,{'code':None}),name=name)
            self.assertEqual(item['judgement'],'UNOBSERVED');self.assertIn('no stable code',item['gap'])
            wrong_stage='run' if legacy_base.VM_FAILURES[name][2]!='run' else 'parse'
            self.assertEqual(self.judge(*self.sample(name,{'code':None,'stage':wrong_stage}),name=name)['judgement'],'FAIL')

    def test_foreign_source_target_infrastructure_and_unknown_fields_are_refused(self):
        changes=[{'schema':'other/1'},{'adapter':'a2r'},{'action':'emit'},{'outcome':'infrastructure'},
                 {'source_sha256':'0'*64},{'entry':'other.at'},{'entry_sha256':'0'*64},
                 {'stage':'typecheck'},{'stage':None},{'code':''},{'code':False},{'code':'x'*129},
                 {'passed':True},{'message':'run: DivisionByZero'}]
        for change in changes:
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(*self.sample(actual_change=change))

    def test_current_check_and_expected_identity_cannot_be_replaced(self):
        for change in ('source','path','adapter','action','state','case','completion','entry-bytes'):
            args=self.sample();check=args[1]
            if change=='source':check['source_sha256']='0'*64
            if change=='path':check['input_paths']=[str(ROOT/'other.at')]
            if change=='adapter':check['adapter']='probe'
            if change=='action':check['action']='emit'
            if change=='state':check['state']='SKIP'
            if change=='case':check['case']='ace.m1.b20'
            if change=='completion':check['completion']['code']='wrong-code'
            if change=='entry-bytes':check['source_files'][0]['sha256']='0'*64
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(*args)
        with self.assertRaises(m.AdmissionError):self.judge(*self.sample(),name='B13')

    def test_product_owner_digest_size_and_presence_are_required(self):
        for change in ('missing','duplicate','name','hash','size','stored-outside','disk'):
            args=self.sample();ref=args[2]['artifacts'][0]
            if change=='missing':args[2]['artifacts']=[]
            if change=='duplicate':args[2]['artifacts']*=2
            if change=='name':ref['path']='error.txt'
            if change=='hash':ref['sha256']='0'*64
            if change=='size':ref['bytes']+=1
            if change=='stored-outside':ref['stored']='D:/other-runtime/failure.json'
            if change=='disk':(self.root/args[0].path_from_original(ref['stored'])).write_bytes(b'{}')
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(*args)
        for owner in ('ace.m1.b20/autovm','ace.m1.b14/a2r','ace.m1.b14/autovm/nested'):
            with self.subTest(owner=owner),self.assertRaises(m.AdmissionError):self.judge(*self.sample(owner=owner))

    def test_invalid_duplicate_or_oversized_json_cannot_supply_failure_facts(self):
        for raw in (b'not-json',b'\xff',b'{"stage":"run","stage":"parse"}',b'{}',b'[]'):
            with self.subTest(raw=raw),self.assertRaises(m.AdmissionError):self.judge(*self.sample(raw=raw))
        args=self.sample(raw=b' '*(16*1024+1))
        with patch.object(args[0],'json',side_effect=AssertionError('oversize must fail before decoding')):
            with self.assertRaises(m.AdmissionError):self.judge(*args)


class LegacyObligationMapTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self):
        return json.loads((ROOT/legacy_obligations.MAP).read_bytes())

    def admit(self,data):
        return legacy_obligations.validate_map(m.EvidenceScope(ROOT),data)

    def test_all_original_cells_have_explicit_locators_or_named_gaps_without_execution(self):
        result=legacy_obligations.admit_legacy_obligations(m.EvidenceScope(ROOT))
        self.assertEqual((result['required'],result['linked_cells'],result['formal_entries']),(313,251,332))
        self.assertEqual(len(result['unbound_cells']),62)
        # PLAN-992..998 reception (941 round 120): 38 -> 0 — the map's
        # cert-gap axis is fully discharged by the three domain receptions.
        self.assertNotIn('legacy-a2r-cert-gap', result['gaps'])
        self.assertEqual(result['executed'],0);self.assertEqual(result['execution'],'UNOBSERVED')
        self.assertFalse(result['mapping_complete']);self.assertFalse(result['can_claim_expanded_m1'])
        cells={x['cell'] for x in self.sample()['cells']}
        self.assertIn('corpus:R01:ac',cells);self.assertIn('scenario:R01:ac',cells)

    def test_missing_duplicate_and_unapproved_required_cells_are_refused(self):
        for change in ('missing','duplicate','foreign'):
            data=self.sample()
            if change=='missing':data['cells'].pop()
            if change=='duplicate':data['cells'][1]=data['cells'][0]
            if change=='foreign':data['cells'][0]['cell']='corpus:O01:ac'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_same_source_old_locator_cannot_replace_exact_admitted_data_entry(self):
        data = self.sample()
        cell = next(c for c in data['cells'] if c['cell'] == 'corpus:R01:av')
        locator = 'ace-m1-original-data-q4-941/ace.m1.original-data.r01/av'
        self.assertIn(locator, cell['entries'])
        cell['entries'].remove(locator)
        del data['entries'][locator]
        # The historical owned recipe locator is still present and same-source.
        self.assertTrue(cell['entries'])
        with self.assertRaises(m.AdmissionError): self.admit(data)

    def test_new_hir_targets_and_typed_frames_cannot_impersonate_old_source_cells(self):
        for change in ('target','source-kind','source-bytes','hir-recipe'):
            data=self.sample();cell=next(c for c in data['cells'] if c['cell']=='corpus:B01:ac')
            if change=='target':cell['target']='ACE-A2R-HIR'
            if change=='source-kind':cell['input_kind']='typed-fixture'
            if change=='source-bytes':cell['frozen_source']['sha256']='0'*64
            if change=='hir-recipe':data['entries'][cell['entries'][0]]['recipe']='ac-a2r/hir-scalar-emit-v1'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_equal_wide_bits_do_not_allow_wrong_kind_entry_alias(self):
        data=self.sample();cells={x['cell']:x for x in data['cells']}
        cells['corpus:W01:ac']['entries']=cells['corpus:W03:ac']['entries']
        with self.assertRaisesRegex(m.AdmissionError,'kind-specific'):self.admit(data)

    def test_original_samples_current_p90_and_historical_p95_are_distinct(self):
        for change in ('revisions','churn','samples','percentile','threshold','historical'):
            data=self.sample();samples=data['original_samples']
            if change=='revisions':samples['I01']['body_revisions']=1
            if change=='churn':samples['I10']['distinct_revisions']=32
            if change=='samples':samples['I11']['min_samples']=1
            if change=='percentile':samples['I11']['gate_percentile']=95
            if change=='threshold':samples['I11']['gate_ms']=5000
            if change=='historical':samples['I11'].pop('historical_diagnostic_ms')
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_removed_cert_gaps_parent_pass_or_accounting_cannot_become_execution(self):
        for change in ('gap','parent-pass','executed','complete','bool-exit'):
            data=self.sample()
            # PLAN-998 reception: the cert-gap axis is fully discharged, so the
            # removable unresolved item is a still-open named gap.
            if change=='gap':next(c for c in data['cells'] if 'formal-entry-not-yet-bound' in c['gaps'])['gaps'].remove('formal-entry-not-yet-bound')
            if change=='parent-pass':data['cells'][0]['execution']='PASS'
            if change=='executed':data['executed']=313
            if change=='complete':data['mapping_complete']=True
            if change=='bool-exit':data['execution']=True
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.admit(data)

    def test_tracked_map_tampering_and_current_case_bytes_are_checked(self):
        data=self.sample();names=set(data['source_pins'])|{'scripts/test-groups.toml',legacy_obligations.MAP}
        for e in data['entries'].values():names.add(e['case_file']);names.update(x['path'] for x in e['inputs'])
        for c in data['cells']:
            if c['frozen_source']:names.add(c['frozen_source']['path'])
        for name in names:
            dest=self.root/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,dest)
        scope=m.EvidenceScope(self.root)
        self.assertEqual(legacy_obligations.admit_legacy_obligations(scope)['required'],313)
        case=self.root/next(iter(data['entries'].values()))['case_file'];case.write_bytes(case.read_bytes()+b'changed')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):legacy_obligations.admit_legacy_obligations(m.EvidenceScope(self.root))
        path=self.root/legacy_obligations.MAP;path.write_bytes(path.read_bytes()+b' ')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):legacy_obligations.admit_legacy_obligations(m.EvidenceScope(self.root))


class CompactAdmissionLifetimeTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def test_transient_input_release_preserves_the_live_cap_and_peak(self):
        budget=m.LiveBudget();scope=m.EvidenceScope(self.root,budget);base=budget.reserved;budget.reserve(8<<20)
        def reader(inputs):
            inputs.reserve(32<<20)
            return {'executed':0,'state':'DISCOVERY'}
        result=first_slice._compact_admission(scope,reader)
        self.assertEqual(result['executed'],0)
        self.assertGreater(budget.reserved,base+(8<<20))
        self.assertLess(budget.reserved,base+(8<<20)+(512<<10))
        self.assertEqual(budget.peak_reserved,base+(40<<20)+(512<<10))
        retained_first=budget.reserved
        first_slice._compact_admission(scope,lambda _: {'second':'summary'})
        self.assertGreater(budget.reserved,retained_first)
        self.assertEqual(budget.peak_reserved,base+(40<<20)+(512<<10))
        def oversized(inputs):
            inputs.reserve(40<<20)
            return {'executed':0}
        with self.assertRaises(m.ResourceError):first_slice._compact_admission(scope,oversized)
        self.assertEqual(budget.peak_reserved,base+(40<<20)+(512<<10))

    def test_an_unbounded_result_cannot_release_its_live_documents(self):
        scope=m.EvidenceScope(self.root);before=scope.budget.reserved
        def retained_document(inputs):
            inputs.reserve(2<<20)
            return {'source':'x'*(256<<10)}
        with self.assertRaisesRegex(m.AdmissionError,'summary exceeds'):first_slice._compact_admission(scope,retained_document)
        self.assertEqual(scope.budget.reserved,before+(512<<10)+(2<<20))

    def test_invalid_releases_cannot_hide_live_allocations(self):
        budget=m.LiveBudget();budget.reserve(100)
        for amount in (-1,True,101):
            with self.assertRaises(m.AdmissionError):budget.release_transient(amount)
        self.assertEqual((budget.reserved,budget.peak_reserved),(100,100))



class CompositeObligationCatalogTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def rows(self):
        return obligation_catalog.read_tsv(m.EvidenceScope(ROOT),obligation_catalog.COMPOSITE_CATALOG,obligation_catalog.COMPOSITE_FIELDS)

    def test_all_supplements_keep_original_oracles_and_reference_gaps(self):
        result=obligation_catalog.admit_composite_catalog(m.EvidenceScope(ROOT),self.rows())
        self.assertEqual(result['cases'],41)
        self.assertEqual(result['families'],{'record-text-cert-925':11,'text-char-925':10,'collections-926':20})
        self.assertEqual(result['oracles'],{'value':34,'trap':7})
        self.assertEqual(result['independent_c_references'],31);self.assertEqual(len(result['reference_gaps']),10)
        self.assertEqual(result['executed'],0);self.assertEqual(result['execution'],'UNOBSERVED')

    def test_scalar_sources_are_physical_original_bytes_without_a_tool_export(self):
        rows=obligation_catalog.read_tsv(m.EvidenceScope(ROOT),obligation_catalog.SCALAR_CATALOG,obligation_catalog.SCALAR_FIELDS)
        self.assertEqual(len(rows),61)
        for row in rows:
            self.assertEqual(hashlib.sha256((ROOT/row['source_file']).read_bytes()).hexdigest(),row['source_sha256'])
        self.assertEqual(obligation_catalog.admit_scalar_catalog(m.EvidenceScope(ROOT),rows)['declared_rust_runs'],69)

    def test_source_oracles_caps_and_profiles_cannot_be_silently_reclassified(self):
        for change in ('missing','duplicate','family','profile','caps','trap','execution','group'):
            rows=self.rows()
            if change=='missing':rows.pop()
            if change=='duplicate':rows[1]=rows[0].copy()
            if change=='family':rows[0]['family']='closures'
            if change=='profile':rows[0]['profile']='core'
            if change=='caps':rows[0]['caps']='[true]'
            if change=='trap':next(x for x in rows if x['oracle_kind']=='trap')['oracle_value']='panic'
            if change=='execution':rows[0]['execution']='PASS'
            if change=='group':rows[0]['formal_group']='ace-av'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):obligation_catalog.admit_composite_catalog(m.EvidenceScope(ROOT),rows)

    def test_missing_reference_is_visible_and_cannot_be_moved_to_another_family(self):
        rows=self.rows();row=next(x for x in rows if x['family']=='collections-926')
        row.update(reference_file='',reference_sha256='',reference_status='original-test-declares-no-reference')
        with self.assertRaises(m.AdmissionError):obligation_catalog.admit_composite_catalog(m.EvidenceScope(ROOT),rows)

    def test_current_fixture_and_independent_reference_are_both_rehashed(self):
        rows=self.rows();names=set(obligation_catalog.COMPOSITE_SOURCES)
        for row in rows:
            names.add(row['source_file'])
            if row['reference_file']:names.add(row['reference_file'])
        for name in names:
            p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,p)
        self.assertEqual(obligation_catalog.admit_composite_catalog(m.EvidenceScope(self.root),rows)['cases'],41)
        for role in ('source_file','reference_file'):
            row=next(x for x in rows if x[role]);p=self.root/row[role];old=p.read_bytes();p.write_bytes(old+b'changed')
            with self.subTest(role=role),self.assertRaisesRegex(m.AdmissionError,'STALE'):
                obligation_catalog.admit_composite_catalog(m.EvidenceScope(self.root),rows)
            p.write_bytes(old)

    def test_renamed_source_paths_cannot_escape_the_registered_fixture(self):
        rows=self.rows();rows[0]['source_file']='../foreign.at'
        with self.assertRaises(m.AdmissionError):obligation_catalog.admit_composite_catalog(m.EvidenceScope(ROOT),rows)



class OriginalBudgetContractTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self):
        data=json.loads((ROOT/budget_contracts.CONTRACT).read_bytes())
        parents=json.loads((ROOT/first_slice.REGISTRY).read_bytes())['parent_contracts']
        return data,parents

    def test_original_domains_remain_distinct_and_declarations_do_not_prove_runtime(self):
        data,parents=self.sample();result=budget_contracts.admit_budget_contracts(m.EvidenceScope(ROOT),parents)
        self.assertEqual(len(result['policies']),16);self.assertFalse(result['can_claim_budget_pass'])
        self.assertEqual(result['executed'],0)
        self.assertEqual({x['runtime_verdict'] for x in result['policies'].values()},{'UNOBSERVED'})
        self.assertIsNone(result['policies']['legacy-inprocess']['limits']['inner_deadline_ms'])
        self.assertEqual(result['policies']['fixed-case-suite']['limits']['wall_ms'],1800000)
        self.assertEqual(result['policies']['abi-interop-runtime']['limits']['run_ms'],5000)
        self.assertEqual(result['policies']['native-worker']['limits']['default_session_wall_ms'],120000)
        self.assertEqual(result['policies']['a2x-scalar-formal']['limits']['source_functions'],115)
        self.assertEqual(result['policies']['a2x-collections-formal']['limits']['source_functions_min'],40)

    def test_public_abi_and_composer_caps_cannot_be_replaced_by_outer_defaults(self):
        for change in ('abi-hot','abi-time','composer','parent'):
            data,parents=self.sample()
            if change=='abi-hot':data['policies']['abi-interop-runtime']['limits']['hot_bytes']=64<<20
            if change=='abi-time':data['policies']['abi-interop-runtime']['limits']['run_ms']=60000
            if change=='composer':data['policies']['composer']['limits']['live_bytes']=64<<20
            if change=='parent':data['policies']['abi-worker-ipc']['limits']=data['policies']['native-worker']['limits']
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):budget_contracts.validate_contract(m.EvidenceScope(ROOT),data,parents)

    def test_zero_bool_unknown_none_and_pass_echoes_cannot_supply_limits(self):
        for change in ('zero','bool','none','pass','execution','missing'):
            data,parents=self.sample()
            if change=='zero':data['policies']['fixed-case-suite']['limits']['wall_ms']=0
            if change=='bool':data['policies']['fixed-case-suite']['limits']['wall_ms']=True
            if change=='none':data['policies']['fixed-case-suite']['limits']['wall_ms']=None
            if change=='pass':data['policies']['fixed-case-suite']['runtime_verdict']='PASS'
            if change=='execution':data['executed']=1
            if change=='missing':data['policies'].pop('a2x-record-text-formal')
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):budget_contracts.validate_contract(m.EvidenceScope(ROOT),data,parents)

    def test_contract_rewrite_and_changed_source_are_stale_before_consumption(self):
        data,parents=self.sample();names=set(data['source_pins'])|{budget_contracts.CONTRACT}
        for name in names:
            p=self.root/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,p)
        self.assertEqual(len(budget_contracts.admit_budget_contracts(m.EvidenceScope(self.root),parents)['policies']),16)
        p=self.root/'scripts/fixed-tests.py';raw=p.read_bytes();p.write_bytes(raw+b'changed')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):budget_contracts.admit_budget_contracts(m.EvidenceScope(self.root),parents)
        p.write_bytes(raw)
        data['policies']['a2x-scalar-formal']['limits']['samples_per_state']=1
        (self.root/budget_contracts.CONTRACT).write_text(json.dumps(data),newline='\n')
        with self.assertRaisesRegex(m.AdmissionError,'STALE'):budget_contracts.admit_budget_contracts(m.EvidenceScope(self.root),parents)



class Q4SourceContractCatalogTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def rows(self):
        return obligation_catalog.read_tsv(m.EvidenceScope(ROOT),obligation_catalog.Q4_CATALOG,obligation_catalog.Q4_FIELDS)

    def test_delivered_ct_sources_keep_all_four_targets_without_implying_execution(self):
        result=obligation_catalog.admit_q4_catalog(m.EvidenceScope(ROOT),self.rows())
        self.assertEqual((result['sources'],result['source_bound'],result['registered_required_target_cells']),(126,126,504))
        self.assertEqual(len(result['pending_provider_sources']),0);self.assertEqual(result['executed'],0)
        self.assertFalse(result['scope_closed']);self.assertEqual(result['execution'],'UNOBSERVED')
        self.assertEqual(result['targets'],['AC-native','AV','ACE-A2R-HIR','ACE-A2C-HIR'])

    def test_missing_source_duplicate_or_target_intersection_cannot_reduce_declared_scope(self):
        for change in ('missing','duplicate','no-c','legacy','execution'):
            rows=self.rows()
            if change=='missing':rows.pop()
            if change=='duplicate':rows[1]=rows[0].copy()
            if change=='no-c':rows[0]['required_targets']='["AC-native","AV","ACE-A2R-HIR"]'
            if change=='legacy':rows[0]['required_targets']='["AC-native","AV","legacy-A2R","ACE-A2C-HIR"]'
            if change=='execution':rows[0]['execution']='PASS'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):obligation_catalog.admit_q4_catalog(m.EvidenceScope(ROOT),rows)

    def test_delivered_ct_source_profile_owner_and_bool_oracle_cannot_be_rewritten(self):
        for change in ('source','profile','oracle','owner'):
            rows=self.rows();row=next(x for x in rows if x['family']=='source-cte' and x['case']=='bool')
            if change=='source':row['source_file']='imaginary.at'
            if change=='profile':row['profile']='core'
            if change=='oracle':row.update(oracle_kind='value',oracle_value='42')
            if change=='owner':row['owner']='PLAN-941'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):obligation_catalog.admit_q4_catalog(m.EvidenceScope(ROOT),rows)

    def test_empty_trace_is_distinct_from_no_trace_and_deliberate_crlf_is_preserved(self):
        rows=self.rows();indexed={r['case']:r for r in rows if r['family']=='scalar'}
        self.assertEqual(json.loads(indexed['bd-or-skips-rhs']['marks']),[])
        self.assertIn('hir.test.trace.v1',json.loads(indexed['bd-or-skips-rhs']['caps']))
        self.assertEqual(json.loads(indexed['sc-crlf']['marks']),[])
        self.assertNotIn('hir.test.trace.v1',json.loads(indexed['sc-crlf']['caps']))
        for name in ('sc-crlf','core-p20-utf8-crlf-min'):
            row=indexed[name];raw=(ROOT/row['source_file']).read_bytes();self.assertIn(b'\r\n',raw)
            self.assertEqual(hashlib.sha256(raw).hexdigest(),row['source_sha256'])

class ArchitectureObligationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self):
        return json.loads((ROOT / arch_obligations.CONTRACT).read_bytes())

    def test_current_entry_locators_leave_every_atomic_obligation_unobserved(self):
        result = arch_obligations.admit_architecture_contract(m.EvidenceScope(ROOT))
        self.assertEqual((result['entry_count'], result['gate']['required']), (30, 49))
        self.assertEqual(result['pending_providers'], arch_obligations.PENDING)
        self.assertEqual(result['executed'], 0)
        self.assertFalse(result['can_claim_architecture_pass'])
        self.assertFalse(result['gate']['complete'])
        self.assertEqual({r['verdict'] for r in result['gate']['rows']}, {'UNOBSERVED'})

    def test_every_deleted_or_failed_architecture_obligation_keeps_and_red(self):
        all_pass = dict.fromkeys(arch_obligations.REQUIRED, 'PASS')
        self.assertTrue(arch_obligations.reduce_architecture(all_pass)['complete'])
        for key in all_pass:
            for state in ('missing', 'FAIL', 'UNOBSERVED'):
                changed = all_pass.copy()
                if state == 'missing': changed.pop(key)
                else: changed[key] = state
                with self.subTest(key=key, state=state):
                    gate = arch_obligations.reduce_architecture(changed)
                    self.assertFalse(gate['complete'])
                    self.assertEqual(gate['required'], 49)
        for bad in ({'ARCH-09/fake':'PASS'}, {arch_obligations.REQUIRED[0]:True},
                    {arch_obligations.REQUIRED[0]:{'pass':True}}):
            with self.assertRaises(m.AdmissionError): arch_obligations.reduce_architecture(bad)

    def test_delete_duplicate_wrong_budget_provider_or_pass_echo_is_rejected(self):
        for change in ('delete', 'duplicate', 'budget', 'provider', 'pass', 'oracle', 'observations', 'verification'):
            data = self.sample()
            if change == 'delete': data['obligations'].pop()
            if change == 'duplicate': data['obligations'][1] = data['obligations'][0].copy()
            if change == 'budget': data['obligations'][0]['budget_domains'] = ['generic-default']
            if change == 'provider': data['entries']['ct']['owner'] = 'PLAN-941'
            if change == 'pass': data['obligations'][0]['execution'] = 'PASS'
            if change == 'oracle': data['obligations'][0]['closing_oracle'] = ''
            if change == 'observations': data['obligations'][0]['required_observations'] = ['summary-pass']
            if change == 'verification': data['obligations'][-1]['verification_ids'] = ['V-17']
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                arch_obligations.validate_contract(m.EvidenceScope(ROOT), data)

    def test_case_or_package_locator_cannot_be_replaced_by_unrelated_group(self):
        for key, group in (('syn', 'case-ace-m1-q4-941'), ('sq-core', 'ace-syntax')):
            data = self.sample(); data['entries'][key]['group'] = group
            with self.subTest(key=key), self.assertRaises(m.AdmissionError):
                arch_obligations.validate_contract(m.EvidenceScope(ROOT), data)

    def test_current_source_and_contract_bytes_are_rechecked_before_admission(self):
        data = self.sample()
        names = {'scripts/test-groups.toml', arch_obligations.CONTRACT}
        for entry in data['entries'].values():
            if entry['kind'] != 'provider-pending': names.add(entry.get('source_file', entry.get('suite_file')))
        for name in names:
            path = self.root / name; path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        self.assertEqual(arch_obligations.admit_architecture_contract(m.EvidenceScope(self.root))['gate']['required'], 49)
        path = self.root / data['entries']['syn-local']['source_file']
        raw = path.read_bytes(); path.write_bytes(raw + b'changed')
        with self.assertRaisesRegex(m.AdmissionError, 'STALE'):
            arch_obligations.admit_architecture_contract(m.EvidenceScope(self.root))
        path.write_bytes(raw)
        data['obligations'][0]['closing_oracle'] = 'All green'
        (self.root / arch_obligations.CONTRACT).write_text(json.dumps(data), newline='\n')
        with self.assertRaisesRegex(m.AdmissionError, 'STALE'):
            arch_obligations.admit_architecture_contract(m.EvidenceScope(self.root))

    def test_registered_index_keeps_targets_old_cells_and_architecture_distinct(self):
        keys = expanded_registry.registered_keys(m.EvidenceScope(ROOT), first_slice.REQUIRED)
        self.assertEqual(len(keys), 1006)
        for prefix, count in (('legacy/', 313), ('q4-source/', 624), ('architecture/', 49), ('preview/', 20)):
            self.assertEqual(sum(key.startswith(prefix) for key in keys), count)
        gate = expanded_registry.registered_gate(keys, {})
        self.assertEqual(gate['counts']['UNOBSERVED'], 1006)
        for family, cases in [('original-method-881', ['M%02d' % i for i in range(1, 13)]),
                              ('original-record-text', ['R01','R02','R03','R04','R05','R06','R09','R10','T01','T02','T04','T06','T07','T08'])]:
            expected = {'q4-source/' + family + '/' + case + '/' + target
                        for case in cases for target in ('AV','AC-native','ACE-A2R-HIR','ACE-A2C-HIR')}
            self.assertEqual({key for key in keys if key.startswith('q4-source/' + family + '/')}, expected)
            self.assertEqual({row['verdict'] for row in gate['registered_rows'] if row['id'] in expected}, {'UNOBSERVED'})
        supplemental = {key for key in keys if key.startswith('q4-source/source-original-scalar/')}
        self.assertEqual(supplemental, {'q4-source/source-original-scalar/' + case + '/' + target
                                      for case in ('c01', 'c02', 'c03', 'named-01')
                                      for target in ('AV', 'AC-native', 'ACE-A2R-HIR', 'ACE-A2C-HIR')})
        self.assertEqual({row['verdict'] for row in gate['registered_rows'] if row['id'] in supplemental}, {'UNOBSERVED'})
        self.assertFalse(gate['complete'])
        # Synthetic reducer success cannot close the unreviewed universe.
        gate = expanded_registry.registered_gate(keys, dict.fromkeys(keys, 'PASS'))
        self.assertTrue(gate['registered_observations_complete'])
        self.assertFalse(gate['required_universe_closed'])
        self.assertFalse(gate['complete'])
        for bad in ({'legacy/ARCH-01/fake':'PASS'}, {keys[0]:True}):
            with self.assertRaises(m.AdmissionError): expanded_registry.registered_gate(keys, bad)

    def test_changed_registered_contract_reference_is_rejected_before_observations(self):
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        registry['expanded_contracts']['architecture']['sha256'] = '0' * 64
        with self.assertRaises(m.AdmissionError):
            expanded_registry.compose_registered_inventory(m.EvidenceScope(ROOT), registry, {}, first_slice.REQUIRED)

    def test_required_universe_closes_with_owned_awaiting_d_rows_only(self):
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        report = {'first_slice_rows': [], 'legacy': {'execution_summary': {'observations': {}}},
                  'architecture_obligations': {'gate': {'rows': []}},
                  'architecture_execution': {'gate': {'rows': []}}}
        composed = expanded_registry.compose_registered_inventory(
            m.EvidenceScope(ROOT), registry, report, first_slice.REQUIRED)
        universe = composed['required_universe']
        self.assertTrue(composed['required_universe_closed'])
        # T-09: the seven PLAN-938 promises moved to delivered-d (archived D
        # received into the tree); only PLAN-939 remains awaiting.
        # Round 113: PLAN-939 delivered (dfe5a1002) — every promise is now
        # delivered; the awaiting section is empty.
        self.assertEqual((universe['registered'], universe['awaiting_d'], universe['delivered_d']),
                         (1006, 0, 14))
        self.assertEqual(universe['awaiting_owners'], {})
        self.assertEqual(universe['delivered_owners'], {'PLAN-938': 7, 'PLAN-939': 7})
        # Closure enumerates ownership only: evidence and the M1 claim stay open.
        self.assertFalse(composed['registered_observations_complete'])
        self.assertFalse(composed['complete'])

    def _registered_report(self, measurements, arch_gate_rows=()):
        return {'first_slice_rows': [], 'legacy': {'execution_summary': {'observations': {}}},
                'architecture_obligations': {'gate': {'rows': []}},
                'architecture_execution': {'gate': {'rows': list(arch_gate_rows)}},
                'incremental_restore_measurements': measurements}

    def test_registered_index_admits_arch05_measurement_pass_rows_only(self):
        # T-06/T-04: the measurement section is the execution-side source for
        # the ARCH-05 registered obligations — a fully observed PASS row claims
        # the obligation; PARTIAL or UNOBSERVED rows keep UNOBSERVED (real but
        # incomplete coverage never claims it).
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        report = self._registered_report({'rows': [
            {'id': 'ARCH-05/restore-immediate-good-edit', 'verdict': 'PASS'},
            {'id': 'ARCH-05/all-required-pages', 'verdict': 'PARTIAL'},
            {'id': 'ARCH-05/historical-cross-project-ownership', 'verdict': 'UNOBSERVED'}]})
        composed = expanded_registry.compose_registered_inventory(
            m.EvidenceScope(ROOT), registry, report, first_slice.REQUIRED)
        rows = {row['id']: row['verdict'] for row in composed['registered_rows']}
        self.assertEqual(rows['architecture/ARCH-05/restore-immediate-good-edit'], 'PASS')
        self.assertEqual(rows['architecture/ARCH-05/all-required-pages'], 'UNOBSERVED')
        self.assertEqual(rows['architecture/ARCH-05/historical-cross-project-ownership'], 'UNOBSERVED')
        self.assertEqual(composed['counts']['PASS'], 1)
        self.assertFalse(composed['registered_observations_complete'])

    def test_registered_index_rejects_measurement_conflict_with_mechanism_verdict(self):
        # The one genuine conflict — a mechanism-side FAIL against a
        # measurement claiming full PASS coverage — is rejected, never
        # silently resolved.
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        report = self._registered_report(
            {'rows': [{'id': 'ARCH-05/restore-immediate-good-edit', 'verdict': 'PASS'}]},
            arch_gate_rows=[{'id': 'ARCH-05/restore-immediate-good-edit', 'verdict': 'FAIL'}])
        with self.assertRaisesRegex(m.AdmissionError, 'measurement verdict conflicts'):
            expanded_registry.compose_registered_inventory(
                m.EvidenceScope(ROOT), registry, report, first_slice.REQUIRED)
        # A non-claiming (PARTIAL/UNOBSERVED) measurement neither conflicts
        # with nor masks an admitted mechanism verdict.
        for mechanism in ('FAIL', 'PASS'):
            with self.subTest(mechanism=mechanism):
                report = self._registered_report(
                    {'rows': [{'id': 'ARCH-05/restore-immediate-good-edit', 'verdict': 'PARTIAL'}]},
                    arch_gate_rows=[{'id': 'ARCH-05/restore-immediate-good-edit', 'verdict': mechanism}])
                composed = expanded_registry.compose_registered_inventory(
                    m.EvidenceScope(ROOT), registry, report, first_slice.REQUIRED)
                rows = {row['id']: row['verdict'] for row in composed['registered_rows']}
                self.assertEqual(rows['architecture/ARCH-05/restore-immediate-good-edit'], mechanism)

    def test_registered_index_rejects_foreign_measurement_row(self):
        # A measurement id outside the registered architecture keys reaches
        # the registered gate's foreign-observation rejection.
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        report = self._registered_report(
            {'rows': [{'id': 'ARCH-05/not-a-registered-obligation', 'verdict': 'PASS'}]})
        with self.assertRaisesRegex(m.AdmissionError, 'foreign registered observation'):
            expanded_registry.compose_registered_inventory(
                m.EvidenceScope(ROOT), registry, report, first_slice.REQUIRED)

    def test_universe_rejects_missing_tampered_or_foreign_awaiting_d_rows(self):
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        base = [dict(row) for row in registry['awaiting_d']]
        delivered = [dict(row) for row in registry['delivered_d']]
        scope = m.EvidenceScope(ROOT)
        def universe(awaiting, delivered_rows=None):
            expanded_registry.universe_rows(scope, first_slice.REQUIRED, awaiting,
                                            delivered if delivered_rows is None else delivered_rows)
        # Round 113: awaiting is empty; dropping a delivered row or blanking
        # its named delivery commit refuses the universe.
        with self.assertRaisesRegex(m.AdmissionError, 'delivered-D surface'):
            universe([], delivered[:-1])
        bad_commit = dict(delivered[0], delivered_commit=' ')
        with self.assertRaisesRegex(m.AdmissionError, 'delivered-D'):
            universe(base, [bad_commit] + delivered[1:])
        # Round 113: the tamper surface rides on delivered rows — tampered
        # promise text, missing close condition, foreign owner and a foreign
        # namespace key are all rejected.
        for change in ('promise', 'close', 'owner', 'namespace'):
            row = dict(delivered[0])
            if change == 'promise': row['promise'] = ''
            if change == 'close': row['close'] = ' '
            if change == 'owner': row['owner'] = 'PLAN-940'
            if change == 'namespace': row['key'] = 'awaited-d/x'
            with self.subTest(change=change), self.assertRaisesRegex(m.AdmissionError, 'delivered-D'):
                universe([], [row] + delivered[1:])
        # A foreign registered observation is still refused after closure.
        with self.assertRaises(m.AdmissionError):
            expanded_registry.registered_gate(
                expanded_registry.registered_keys(scope, first_slice.REQUIRED),
                {'awaiting-d/plan-938/ac-01-checked-source-env': 'PASS'})
        # The reducer alone still never closes the universe.
        gate = expanded_registry.registered_gate(
            expanded_registry.registered_keys(scope, first_slice.REQUIRED), {})
        self.assertFalse(gate['required_universe_closed'])

    def test_registry_awaiting_d_bytes_are_pinned_by_the_reader(self):
        registry = json.loads((ROOT / first_slice.REGISTRY).read_bytes())
        digest = hashlib.sha256(json.dumps(registry['awaiting_d'], ensure_ascii=False,
                                           sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()
        self.assertEqual(digest, first_slice.AWAITING_D_SHA256)
        # T-09: the delivered section is byte-pinned the same way.
        delivered_digest = hashlib.sha256(json.dumps(registry['delivered_d'], ensure_ascii=False,
                                                     sort_keys=True, separators=(',', ':')).encode('utf-8')).hexdigest()
        self.assertEqual(delivered_digest, first_slice.DELIVERED_D_SHA256)
        tampered_d = [dict(row) for row in registry['delivered_d']]
        tampered_d[0]['delivered_commit'] = '0' * 40
        self.assertNotEqual(
            hashlib.sha256(json.dumps(tampered_d, ensure_ascii=False, sort_keys=True,
                                      separators=(',', ':')).encode('utf-8')).hexdigest(),
            first_slice.DELIVERED_D_SHA256)
        # Any byte-level edit breaks the pinned digest before admission.
        tampered = [dict(row) for row in registry['awaiting_d']]
        # Round 113: awaiting is empty — the delivered section carries the
        # same byte-pinned tamper surface.
        tampered = json.loads(json.dumps(registry['delivered_d']))
        delivered_digest = hashlib.sha256(json.dumps(registry['delivered_d'], ensure_ascii=False,
                                                     sort_keys=True,
                                                     separators=(',', ':')).encode('utf-8')).hexdigest()
        self.assertEqual(delivered_digest, first_slice.DELIVERED_D_SHA256)
        # The awaiting section is empty and pinned as the empty list.
        self.assertEqual(first_slice.AWAITING_D_SHA256,
                         hashlib.sha256(json.dumps([], ensure_ascii=False, sort_keys=True,
                                                   separators=(',', ':')).encode('utf-8')).hexdigest())
        # A tampered delivered row is refused by the delivered surface checks.
        tampered[0]['delivered_commit'] = ' '
        with self.assertRaisesRegex(m.AdmissionError, 'delivered-D'):
            expanded_registry.universe_rows(m.EvidenceScope(ROOT), first_slice.REQUIRED, [],
                                            tampered)


class AcceptanceReducerTests(unittest.TestCase):
    """PLAN-941 T-04: the §5.2 total predicate as a pure reducer."""

    def gates(self, **overrides):
        gates = dict.fromkeys(m.ACCEPTANCE_GATES, True)
        gates.update(overrides)
        return gates

    def reduce(self, **kwargs):
        args = dict(gates=self.gates(), accounting_valid=True, evidence_complete=True,
                    cert_gaps=0, awaiting_d=0, system_delivered=True, missing_inputs=[])
        args.update(kwargs)
        return m.reduce_acceptance(**args)

    def test_isolated_fully_green_control_fires_and_names_every_blocker_class(self):
        # Isolated reducer control only — never production evidence.
        verdict = self.reduce()
        self.assertTrue(verdict['can_claim_expanded_m1'])
        self.assertEqual(verdict['blockers'], [])
        for kwargs, blocker in (
                (dict(gates=self.gates(legacy_current_gate=False)), 'legacy_current_gate'),
                (dict(accounting_valid=False), 'accounting_invalid'),
                (dict(evidence_complete=False), 'evidence_incomplete'),
                (dict(cert_gaps=38), 'legacy_cert_gaps:38'),
                (dict(awaiting_d=14), 'awaiting_d:14'),
                (dict(system_delivered=False), 'acceptance_system_not_delivered'),
                (dict(missing_inputs=['budget_time_measurements']), 'missing_input:budget_time_measurements')):
            with self.subTest(blocker=blocker):
                verdict = self.reduce(**kwargs)
                self.assertFalse(verdict['can_claim_expanded_m1'])
                self.assertIn(blocker, verdict['blockers'])
        # The four input classes never derive the claim on their own: all
        # domain gates green plus accounting still cannot outvote an open
        # evidence/universe/system blocker.
        self.assertFalse(self.reduce(evidence_complete=False, cert_gaps=38, awaiting_d=14,
                                     system_delivered=False)['can_claim_expanded_m1'])

    def test_reducer_rejects_unknown_missing_or_mistyped_inputs(self):
        with self.assertRaisesRegex(m.AdmissionError, 'acceptance domain gates'):
            self.reduce(gates=dict.fromkeys(m.ACCEPTANCE_GATES[:-1], True))
        forged = dict.fromkeys(m.ACCEPTANCE_GATES, True); forged['extra'] = True
        with self.assertRaisesRegex(m.AdmissionError, 'acceptance domain gates'):
            self.reduce(gates=forged)
        with self.assertRaisesRegex(m.AdmissionError, 'must be a bool'):
            self.reduce(gates=self.gates(c_core_and_resource=1))
        for kwargs in (dict(accounting_valid=None), dict(cert_gaps=-1), dict(awaiting_d='14'),
                       dict(missing_inputs='budgets'), dict(missing_inputs=[''])):
            with self.subTest(kwargs=kwargs):
                with self.assertRaises(m.AdmissionError):
                    self.reduce(**kwargs)


class IncrementalRestoreMeasurementTests(unittest.TestCase):
    """PLAN-941 T-06: ARCH-05 row coverage from admitted gate events."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        events = {}
        for spec in incremental_restore.ROWS:
            for needle in spec['tests']:
                name = 'ace-core::' + needle + '::synthetic'
                events[name] = {'type': 'test', 'event': 'ok', 'name': name}
        self.document = {'schema': 'fixed-test-run/1', 'gate_passed': True,
                         'results': [{'groups': [group], 'outcome': 'PASS', 'executed': 1,
                                      'results': events if index == 0 else {}}
                                     for index, group in enumerate(incremental_restore.GROUPS)]}
        self.ct_restore = {'observations': {key.split('/', 1)[1]: 'PASS'
                                            for spec in incremental_restore.ROWS
                                            for observation in spec['observations']
                                            for key in [observation] if observation.startswith('ct_restore/')}}
        self.arch_rows = [{'id': observation.split('/', 1)[1], 'verdict': 'PASS'}
                          for spec in incremental_restore.ROWS
                          for observation in spec['observations']
                          if observation.startswith('architecture/')]

    def tearDown(self):
        shutil.rmtree(self.root, ignore_errors=True)

    def admit(self, document=None):
        path = self.root / 'gate.json'
        path.write_text(json.dumps(document or self.document), newline='\n')
        ref = {'path': 'gate.json', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
        return incremental_restore.admit_incremental_restore_measurements(
            m.EvidenceScope(self.root), m.EvidenceScope(self.root), ref, self.ct_restore, self.arch_rows)

    def test_companion_bindings_merge_only_their_own_group_events(self):
        # A companion document (ace-semantics / ace-comptime) contributes its
        # own events to the pool; wrong group, non-green run, malformed event
        # or wrong hash is rejected, and a row whose needles live only in the
        # companion stays UNOBSERVED without it.
        meta = next(row for row in incremental_restore.ROWS if row['id'] == 'ARCH-05/signature-layout-meta')
        meta_needles = [n for n in meta['tests'] if n.startswith('meta_capability_941$')]
        needle = meta_needles[0]
        stripped = json.loads(json.dumps(self.document))
        stripped['results'][0]['results'] = {
            name: event for name, event in stripped['results'][0]['results'].items()
            if not any(needle in name for needle in meta_needles)}
        gate_path = self.root / 'gate.json'
        gate_path.write_text(json.dumps(stripped), newline='\n')
        companion_events = {'ace-source-cte::' + n: {'type': 'test', 'event': 'ok'}
                            for n in meta_needles}
        companion = {'schema': 'fixed-test-run/1', 'gate_passed': True,
                     'results': [{'groups': ['ace-comptime'], 'outcome': 'PASS',
                                  'executed': len(companion_events),
                                  'results': companion_events}]}
        companion_path = self.root / 'companion.json'
        companion_path.write_text(json.dumps(companion), newline='\n')
        good_ref = {'path': 'companion.json',
                    'sha256': hashlib.sha256(companion_path.read_bytes()).hexdigest()}
        def admit_with(ref):
            return incremental_restore.admit_incremental_restore_measurements(
                m.EvidenceScope(self.root), m.EvidenceScope(self.root),
                {'path': 'gate.json', 'sha256': hashlib.sha256(gate_path.read_bytes()).hexdigest()},
                self.ct_restore, self.arch_rows, None, ref)
        # Without the companion the meta needles are missing from the pool.
        section = admit_with(None)
        row = next(row for row in section['rows'] if row['id'] == 'ARCH-05/signature-layout-meta')
        self.assertEqual(row['verdict'], 'PARTIAL')
        self.assertIn(needle, row['missing'])
        # With the admitted companion its events cover the row.
        section = admit_with(good_ref)
        row = next(row for row in section['rows'] if row['id'] == 'ARCH-05/signature-layout-meta')
        self.assertEqual(row['verdict'], 'PASS')
        self.assertIn('ace-source-cte::' + needle, row['evidence']['tests'])
        # Wrong group / not green / stale hash / malformed event are refused.
        for change in ('groups', 'outcome', 'sha', 'event'):
            bad = json.loads(json.dumps(companion))
            if change == 'groups': bad['results'][0]['groups'] = ['ace-semantics']
            if change == 'outcome': bad['results'][0]['outcome'] = 'FAIL'
            if change == 'event': bad['results'][0]['results'] = {'x': {'event': 'fail'}}
            bad_path = self.root / 'companion-bad.json'
            bad_path.write_text(json.dumps(bad), newline='\n')
            ref = {'path': 'companion-bad.json',
                   'sha256': hashlib.sha256(bad_path.read_bytes()).hexdigest()}
            if change == 'sha': ref['sha256'] = '0' * 64
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                admit_with(ref)

    def test_av_companion_covers_the_historical_row_av_needles(self):
        # The AV-side f94109 consumption suite enters only through the
        # ace-av companion binding: without it the three AV needles are
        # missing from the pool; with an admitted companion they cover the
        # row while the remaining CT/foreign partial gap keeps it PARTIAL;
        # a wrong-group or non-green AV document is rejected.
        historical = next(row for row in incremental_restore.ROWS
                          if row['id'] == 'ARCH-05/historical-cross-project-ownership')
        av_needles = [n for n in historical['tests'] if 'f94109_av_' in n]
        self.assertEqual(len(av_needles), 3)
        stripped = json.loads(json.dumps(self.document))
        stripped['results'][0]['results'] = {
            name: event for name, event in stripped['results'][0]['results'].items()
            if not any(needle in name for needle in av_needles)}
        gate_path = self.root / 'gate.json'
        gate_path.write_text(json.dumps(stripped), newline='\n')
        av_events = {'ace-av::conformance$sq_source_facts_934::' + n.rsplit('::', 1)[1]
                     : {'type': 'test', 'event': 'ok'} for n in av_needles}
        av_doc = {'schema': 'fixed-test-run/1', 'gate_passed': True,
                  'results': [{'groups': ['ace-av'], 'outcome': 'PASS',
                               'executed': len(av_events), 'results': av_events}]}
        av_path = self.root / 'av.json'
        av_path.write_text(json.dumps(av_doc), newline='\n')
        good_ref = {'path': 'av.json', 'sha256': hashlib.sha256(av_path.read_bytes()).hexdigest()}
        def admit_with(ref):
            return incremental_restore.admit_incremental_restore_measurements(
                m.EvidenceScope(self.root), m.EvidenceScope(self.root),
                {'path': 'gate.json', 'sha256': hashlib.sha256(gate_path.read_bytes()).hexdigest()},
                self.ct_restore, self.arch_rows, None, None, None, ref)
        section = admit_with(None)
        row = next(row for row in section['rows']
                   if row['id'] == 'ARCH-05/historical-cross-project-ownership')
        self.assertEqual(row['verdict'], 'PARTIAL')
        for needle in av_needles:
            self.assertIn(needle, row['missing'])
        section = admit_with(good_ref)
        row = next(row for row in section['rows']
                   if row['id'] == 'ARCH-05/historical-cross-project-ownership')
        # With the CT and foreign-family needles satisfied by the synthetic
        # main-document events (setUp) and the AV needles by the admitted
        # companion, the row now reaches PASS.
        self.assertEqual(row['verdict'], 'PASS')
        self.assertEqual(row['missing'], [])
        # PLAN-1000: per-needle coverage, not exact matched-list length —
        # the corrected cache_turnaround:: needle legitimately substring-
        # matches the all-required-pages needle's event too.
        for needle in historical['tests']:
            self.assertTrue(any(needle in t for t in row['evidence']['tests']),
                            'uncovered historical needle: ' + needle)
        for change in ('groups', 'outcome'):
            bad = json.loads(json.dumps(av_doc))
            if change == 'groups': bad['results'][0]['groups'] = ['ace-semantics']
            if change == 'outcome': bad['results'][0]['outcome'] = 'FAIL'
            bad_path = self.root / 'av-bad.json'
            bad_path.write_text(json.dumps(bad), newline='\n')
            bad_ref = {'path': 'av-bad.json',
                       'sha256': hashlib.sha256(bad_path.read_bytes()).hexdigest()}
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                admit_with(bad_ref)

    def test_full_synthetic_coverage_closes_the_measurement_gate(self):
        section = self.admit()
        # Isolated synthetic control — never production evidence: every row
        # that has no partial_missing list reaches PASS, partial rows stay
        # PARTIAL, and the gate only closes when ALL rows are PASS.
        full = [row for row in incremental_restore.ROWS if not row['partial_missing']]
        self.assertEqual(sum(row['verdict'] == 'PASS' for row in section['rows']), len(full))
        # PLAN-1000 (delivered D939 mapping): every row's partial_missing is
        # now empty, so full synthetic coverage closes the measurement gate;
        # the isolated-control caveat still stands (this document is synthetic
        # and never production evidence).
        self.assertTrue(section['gate']['measurements_complete'])
        self.assertTrue(all(row['verdict'] == 'PASS' for row in section['rows']))

    def test_dropping_a_mapped_test_degrades_only_its_row(self):
        needle = incremental_restore.ROWS[0]['tests'][0]
        self.document['results'][0]['results'] = {
            name: event for name, event in self.document['results'][0]['results'].items()
            if needle not in name}
        section = self.admit()
        first = section['rows'][0]
        self.assertEqual(first['verdict'], 'PARTIAL')
        self.assertIn(needle, first['missing'])
        self.assertEqual(section['rows'][1]['verdict'], 'PASS')
        # Removing the row's whole test family degrades it to UNOBSERVED
        # without touching the other rows.
        for needle in incremental_restore.ROWS[0]['tests']:
            self.document['results'][0]['results'] = {
                name: event for name, event in self.document['results'][0]['results'].items()
                if needle not in name}
        first = self.admit()['rows'][0]
        self.assertEqual(first['verdict'], 'UNOBSERVED')
        self.assertEqual(len(first['evidence']['tests']), 0)


    def test_stale_foreign_or_nongreen_documents_are_rejected(self):
        for change, pattern in (('sha', 'STALE'), ('schema', 'schema'), ('gate', 'did not pass'),
                                ('groups', 'groups'), ('outcome', 'execute green')):
            document = json.loads(json.dumps(self.document))
            if change == 'sha':
                path = self.root / 'gate.json'
                path.write_text(json.dumps(document), newline='\n')
                ref = {'path': 'gate.json', 'sha256': '0' * 64}
                with self.assertRaisesRegex(m.AdmissionError, pattern):
                    incremental_restore.admit_incremental_restore_measurements(
                        m.EvidenceScope(self.root), m.EvidenceScope(self.root), ref, self.ct_restore, self.arch_rows)
                continue
            if change == 'schema': document['schema'] = 'other/1'
            if change == 'gate': document['gate_passed'] = False
            if change == 'groups': document['results'][1]['groups'] = ['ace-syntax-x']
            if change == 'outcome': document['results'][2]['outcome'] = 'FAIL'
            with self.subTest(change=change), self.assertRaisesRegex(m.AdmissionError, pattern):
                self.admit(document)


class BudgetTimeMeasurementTests(unittest.TestCase):
    """PLAN-941 T-07: budget/time measurements from admitted meters."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        self.gate = {'schema': 'fixed-test-run/1', 'gate_passed': True,
                     'results': [{'groups': [group], 'outcome': 'PASS', 'executed': 1,
                                  'execution': {'meter': {'wall_ms': 1000.5, 'workload_cpu_ms': 900.0,
                                                          'peak_job_bytes': 1 << 20, 'job_processes': 4}}}
                                 for group in budget_time_module.GROUPS]}
        self.optimized = {'exit': 0, 'meter': {'wall_ms': 7175.5, 'workload_cpu_ms': 7000.0,
                                               'peak_job_bytes': 618_774_528, 'job_processes': 12}}
        self.contracts = {'policies': {name: {'runtime_verdict': 'UNOBSERVED'}
                                       for name in ('av-worker', 'native-worker', 'a2x-scalar-formal')}}

    def tearDown(self):
        shutil.rmtree(self.root, ignore_errors=True)

    def admit(self, gate=None, optimized=None, peak=(32 << 20)):
        gate_path = self.root / 'gate.json'
        gate_path.write_text(json.dumps(gate or self.gate), newline='\n')
        optimized_path = self.root / 'optimized.json'
        optimized_path.write_text(json.dumps(optimized or self.optimized), newline='\n')
        return budget_time_module.admit_budget_time_measurements(
            m.EvidenceScope(self.root), m.EvidenceScope(self.root),
            {'path': 'gate.json', 'sha256': hashlib.sha256(gate_path.read_bytes()).hexdigest()},
            {'path': 'optimized.json', 'sha256': hashlib.sha256(optimized_path.read_bytes()).hexdigest()},
            self.contracts, peak)

    def test_real_meters_pass_but_declarations_keep_the_gate_open(self):
        section = self.admit()
        by_id = {row['id']: row for row in section['rows']}
        self.assertEqual(by_id['gate-meter/ace-source']['verdict'], 'PASS')
        self.assertEqual(by_id['optimized-original-observer-meter']['verdict'], 'PASS')
        self.assertEqual(by_id['composer-live-cap-48mib']['verdict'], 'PASS')
        self.assertEqual(by_id['policy-runtime-samples']['verdict'], 'PARTIAL')
        self.assertIn('declarations never prove runtime',
                      ' '.join(by_id['policy-runtime-samples']['missing']))
        self.assertFalse(section['gate']['budgets_time_complete'])
        # An over-cap composer reservation is a real FAIL, not a missing row.
        section = self.admit(peak=(64 << 20))
        self.assertEqual(section['rows'][-2]['verdict'], 'FAIL')

    def test_stale_or_malformed_documents_are_rejected(self):
        gate_path = self.root / 'gate.json'
        gate_path.write_text(json.dumps(self.gate), newline='\n')
        with self.assertRaisesRegex(m.AdmissionError, 'STALE'):
            budget_time_module.admit_budget_time_measurements(
                m.EvidenceScope(self.root), m.EvidenceScope(self.root),
                {'path': 'gate.json', 'sha256': '0' * 64},
                {'path': 'optimized.json', 'sha256': '0' * 64},
                self.contracts, 1 << 20)
        for kwargs in (dict(gate={'schema': 'other/1'}), dict(optimized={'exit': 3}),
                       dict(optimized={'meter': {'wall_ms': 0, 'workload_cpu_ms': 1,
                                                 'peak_job_bytes': 1, 'job_processes': 1}})):
            with self.subTest(kwargs=list(kwargs)), self.assertRaises(m.AdmissionError):
                self.admit(**kwargs)
        optimized_path = self.root / 'optimized.json'
        optimized_path.write_text(json.dumps(self.optimized), newline='\n')
        with self.assertRaisesRegex(m.AdmissionError, 'policy'):
            budget_time_module.admit_budget_time_measurements(
                m.EvidenceScope(self.root), m.EvidenceScope(self.root),
                {'path': 'gate.json', 'sha256': hashlib.sha256(gate_path.read_bytes()).hexdigest()},
                {'path': 'optimized.json', 'sha256': hashlib.sha256(optimized_path.read_bytes()).hexdigest()},
                {'policies': {}}, 1 << 20)

    def _samples_binding(self, n=11):
        runs = self.root / 'runs'; runs.mkdir(exist_ok=True)
        sources = []
        for i in range(n):
            path = runs / f'run-{i}.command.json'
            path.write_text(json.dumps({'schema': 'plan941.test/1',
                                        'meter': {'wall_ms': 100.0 + i, 'peak_job_bytes': 1 << 20}}), newline='\n')
            sources.append({'path': str(path.relative_to(self.root)).replace('\\', '/'),
                            'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})
        binding = self.root / 'samples.json'
        binding.write_text(json.dumps({'schema': 'ace.m1.policy-runtime-samples-binding/1',
                                       'policies': {'ac-v2-check': sources}}), newline='\n')
        return {'path': 'samples.json', 'sha256': hashlib.sha256(binding.read_bytes()).hexdigest()}

    def _admit_with_samples(self, samples_ref):
        gate_path = self.root / 'gate.json'
        gate_path.write_text(json.dumps(self.gate), newline='\n')
        optimized_path = self.root / 'optimized.json'
        optimized_path.write_text(json.dumps(self.optimized), newline='\n')
        return budget_time_module.admit_budget_time_measurements(
            m.EvidenceScope(self.root), m.EvidenceScope(self.root),
            {'path': 'gate.json', 'sha256': hashlib.sha256(gate_path.read_bytes()).hexdigest()},
            {'path': 'optimized.json', 'sha256': hashlib.sha256(optimized_path.read_bytes()).hexdigest()},
            self.full_contracts(), 1 << 20, samples_ref)

    def full_contracts(self):
        return {'policies': {'ac-v2-check': {'limits': {'build_ms': 60000}, 'runtime_verdict': 'UNOBSERVED'},
                             'av-worker': {'limits': {'response_bytes': 1}, 'runtime_verdict': 'UNOBSERVED'}}}

    def test_policy_samples_count_only_really_measured_windows(self):
        # Eleven real command meters observe the policy; the other declared
        # policy stays named-missing, so the row keeps PARTIAL honestly.
        section = self._admit_with_samples(self._samples_binding(11))
        row = next(r for r in section['rows'] if r['id'] == 'policy-runtime-samples')
        self.assertEqual(row['verdict'], 'PARTIAL')
        self.assertEqual(row['measured']['runtime_observed'], 1)
        self.assertIn('av-worker lacks a measured runtime window', row['missing'])
        self.assertEqual(row['measured']['per_policy']['ac-v2-check']['n'], 11)
        self.assertEqual(row['measured']['per_policy']['ac-v2-check']['p90_ms'], 109.0)
        # Nine windows are below the ten-window floor: the policy is NOT
        # observed even though the samples really ran.
        section = self._admit_with_samples(self._samples_binding(9))
        row = next(r for r in section['rows'] if r['id'] == 'policy-runtime-samples')
        self.assertEqual(row['verdict'], 'PARTIAL')
        self.assertEqual(row['measured']['runtime_observed'], 0)
        self.assertIn('ac-v2-check lacks a measured runtime window', row['missing'])

    def test_policy_sample_documents_are_reread_and_strict(self):
        ref = self._samples_binding(11)
        # A stale source hash, a zero wall, a foreign policy name and a
        # budget-exceeding window are all refused.
        good = json.loads((self.root / 'samples.json').read_bytes())
        def write_samples(mutated):
            binding = self.root / 'samples.json'
            binding.write_text(json.dumps(mutated), newline='\n')
            return {'path': 'samples.json', 'sha256': hashlib.sha256(binding.read_bytes()).hexdigest()}
        with self.assertRaises(m.AdmissionError):  # stale source sha
            self._admit_with_samples({'path': 'samples.json', 'sha256': '0' * 64})
        bad = json.loads(json.dumps(good)); bad['policies']['ac-v2-check'][0]['sha256'] = '0' * 64
        with self.assertRaises(m.AdmissionError):
            self._admit_with_samples(write_samples(bad))
        zero = self.root / 'runs/run-0.command.json'
        zero.write_text(json.dumps({'schema': 'plan941.test/1', 'meter': {'wall_ms': 0}}), newline='\n')
        with self.assertRaises(m.AdmissionError):
            self._admit_with_samples(ref)
        zero.write_text(json.dumps({'schema': 'plan941.test/1', 'meter': {'wall_ms': 500000.0}}), newline='\n')
        bad = json.loads(json.dumps(good))
        for row in bad['policies']['ac-v2-check']:
            row['sha256'] = hashlib.sha256((self.root / row['path']).read_bytes()).hexdigest()
        with self.assertRaises(m.AdmissionError):  # exceeds build_ms budget
            self._admit_with_samples(write_samples(bad))
        foreign = json.loads(json.dumps(good)); foreign['policies']['not-a-policy'] = foreign['policies'].pop('ac-v2-check')
        with self.assertRaises(m.AdmissionError):
            self._admit_with_samples(write_samples(foreign))

class AcceptanceSystemControlsTests(unittest.TestCase):
    """PLAN-941 T-10: the named V-14/V-15 control family gates delivery."""

    def setUp(self):
        self.root = Path(tempfile.mkdtemp())
        selected = ['test_ace_m1_arch_acceptance.Suite.test_' + needle
                    for _, needle in system_controls.CONTROL_MAP]
        self.document = {'schema': 'fixed-test-run/1', 'gate_passed': True,
                         'results': [{'groups': [system_controls.GROUP], 'outcome': 'PASS',
                                      'executed': len(selected),
                                      'report': {'schema': 'python-tests/1', 'selected': selected,
                                                 'executed': len(selected), 'skipped': [],
                                                 'failures': [], 'errors': []}}]}

    def tearDown(self):
        shutil.rmtree(self.root, ignore_errors=True)

    def admit(self, document=None):
        path = self.root / 'controls-gate.json'
        path.write_text(json.dumps(document or self.document), newline='\n')
        return system_controls.admit_system_controls(
            m.EvidenceScope(self.root), m.EvidenceScope(self.root),
            {'path': 'controls-gate.json', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()})

    def test_full_control_family_closes_delivery_and_a_missing_class_keeps_it_open(self):
        section = self.admit()
        # Isolated synthetic control — never production evidence.
        self.assertTrue(section['gate']['controls_complete'])
        self.assertEqual(len(section['controls']), len(system_controls.CONTROL_MAP))
        self.assertEqual(section['missing'], [])
        # Dropping one executed control keeps the family honestly incomplete
        # and names exactly the missing class.
        document = json.loads(json.dumps(self.document))
        document['results'][0]['report']['selected'] = [
            name for name in document['results'][0]['report']['selected']
            if system_controls.CONTROL_MAP[0][1] not in name]
        document['results'][0]['executed'] = len(document['results'][0]['report']['selected'])
        document['results'][0]['report']['executed'] = len(document['results'][0]['report']['selected'])
        section = self.admit(document)
        self.assertFalse(section['gate']['controls_complete'])
        self.assertEqual(section['missing'], [system_controls.CONTROL_MAP[0][0]])

    def test_foreign_group_stale_hash_failures_or_skips_are_refused(self):
        for change in ('group', 'outcome', 'failures', 'sha', 'count'):
            document = json.loads(json.dumps(self.document))
            if change == 'group': document['results'][0]['groups'] = ['ace-source']
            if change == 'outcome': document['results'][0]['outcome'] = 'FAIL'
            if change == 'failures': document['results'][0]['report']['failures'] = ['x']
            if change == 'count': document['results'][0]['executed'] = 3
            path = self.root / 'controls-gate.json'
            path.write_text(json.dumps(document), newline='\n')
            ref = {'path': 'controls-gate.json', 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
            if change == 'sha': ref['sha256'] = '0' * 64
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                system_controls.admit_system_controls(
                    m.EvidenceScope(self.root), m.EvidenceScope(self.root), ref)

class OriginalSourceSupplementTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def test_missing_original_data_certifications_stay_required_and_unobserved(self):
        contracts=original_data.reference_contracts(m.EvidenceScope(ROOT))
        expected={'q4-source/'+row['family']+'/'+row['case']+'/'+target
                  for row in contracts.values() for target in ('ACE-A2R-HIR','ACE-A2C-HIR')}
        absent={key for key in expected if key.endswith('/ACE-A2R-HIR')
                and key.split('/')[-2] in ('M01','R01','R02')}
        # Isolated coverage control, never production certification evidence.
        supplied={key:{'judgement':'PASS'} for key in expected-absent}
        rows,missing=original_data.certification_coverage(contracts,supplied)
        self.assertEqual(set(rows),expected)
        self.assertEqual(set(missing),absent)
        self.assertEqual({rows[key]['judgement'] for key in absent},{'UNOBSERVED'})
        rows,missing=original_data.certification_coverage(contracts,{})
        self.assertEqual(set(missing),expected)
        self.assertTrue(all(row['judgement']=='UNOBSERVED' for row in rows.values()))
        for bad in ({'q4-source/foreign/M01/ACE-A2R-HIR':{'judgement':'PASS'}},
                    {next(iter(expected)):{'judgement':'UNKNOWN'}}):
            with self.assertRaises(m.AdmissionError):original_data.certification_coverage(contracts,bad)

    def test_references_cannot_hide_changed_original_source_or_reference_bytes(self):
        contracts = source_supplement.reference_contracts(m.EvidenceScope(ROOT))
        paths = {source_supplement.REFERENCE, source_supplement.CATALOG, obligation_catalog.Q4_CATALOG}
        for row in source_supplement.source_contracts(m.EvidenceScope(ROOT)).values():
            paths.update((row['source_file'], row['origin_case_file']))
        for row in contracts.values():
            paths.update((row['rust_reference'], row['c_reference']))
        for relative in paths:
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes((ROOT / relative).read_bytes())
        self.assertEqual(len(source_supplement.reference_contracts(m.EvidenceScope(self.root))), 4)
        for field in ('source_file', 'rust_reference', 'c_reference'):
            path = self.root / contracts['named-01'][field]
            original = path.read_bytes()
            with self.subTest(field=field):
                path.write_bytes(original + b'\n// changed source or independent reference\n')
                with self.assertRaisesRegex(m.AdmissionError, 'STALE'):
                    source_supplement.reference_contracts(m.EvidenceScope(self.root))
                path.write_bytes(original)


class TextCertificationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def observations(self, line):
        pair = {'compile':{'exit_code':0, 'stdout_hex':'', 'stderr_hex':''},
                'run':{'exit_code':0, 'stdout_hex':line.encode().hex(), 'stderr_hex':''}}
        return {'schema':'ace.a2x.cert-execution/1', 'actual':json.loads(json.dumps(pair)),
                'reference':json.loads(json.dumps(pair))}

    def test_same_wrong_value_or_trap_in_both_legs_never_matches_independent_oracle(self):
        for expected, wrong in (('value:506\n', 'value:507\n'),
                                ('trap:TextIndex\n', 'trap:InvalidScalar\n')):
            with self.subTest(expected=expected):
                self.assertEqual(cert_evidence.process_matches(self.observations(expected), expected.encode().hex(), ''), [True, True])
                self.assertEqual(cert_evidence.process_matches(self.observations(wrong), expected.encode().hex(), ''), [False, False])
        observed = self.observations('value:506\n')
        observed['actual']['run']['stderr_hex'] = b'unexpected side effect'.hex()
        self.assertEqual(cert_evidence.process_matches(observed, b'value:506\n'.hex(), ''), [False, True])

    def test_compile_run_terminal_or_schema_cannot_be_missing_or_echoed(self):
        for change in ('missing-compile', 'missing-reference', 'exit-bool', 'nonzero', 'pass-echo', 'wrong-schema'):
            observed = self.observations('value:506\n')
            if change == 'missing-compile': observed['actual'].pop('compile')
            if change == 'missing-reference': observed.pop('reference')
            if change == 'exit-bool': observed['reference']['compile']['exit_code'] = False
            if change == 'nonzero': observed['actual']['compile']['exit_code'] = 1
            if change == 'pass-echo': observed['actual']['run']['verdict'] = 'PASS'
            if change == 'wrong-schema': observed['schema'] = 'ace.a2x.cert-execution/0'
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                cert_evidence.process_matches(observed, b'value:506\n'.hex(), '')

    def test_independent_references_preserve_nul_unicode_and_trap_contracts(self):
        rows = text_evidence.text_contracts(m.EvidenceScope(ROOT))
        self.assertEqual(len(rows), 10)
        self.assertEqual(rows['v03_nul_data']['oracle_value'], '33')
        self.assertEqual(rows['v03_len_scalars_bytes']['oracle_value'], '506')
        self.assertEqual(rows['v03_char_at_negative']['oracle_value'], 'TextIndex')
        self.assertEqual(rows['v03_codepoint_surrogate']['oracle_value'], 'InvalidScalar')
        self.assertEqual({row['execution'] for row in rows.values()}, {'UNOBSERVED'})

    def test_changed_reference_bytes_or_rewritten_oracle_are_stale(self):
        rows = text_evidence.text_contracts(m.EvidenceScope(ROOT))
        names = {text_evidence.CONTRACT} | set(text_evidence.COMPOSITE_SOURCES)
        for row in rows.values(): names.update(row[k] for k in ('source_file', 'rust_reference', 'c_reference'))
        for name in names:
            path = self.root / name; path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        self.assertEqual(len(text_evidence.text_contracts(m.EvidenceScope(self.root))), 10)
        for key in ('source_file', 'rust_reference', 'c_reference'):
            path = self.root / rows['v03_nul_data'][key]; raw = path.read_bytes(); path.write_bytes(raw + b'changed')
            with self.subTest(role=key), self.assertRaisesRegex(m.AdmissionError, 'STALE'):
                text_evidence.text_contracts(m.EvidenceScope(self.root))
            path.write_bytes(raw)
        path = self.root / text_evidence.CONTRACT
        path.write_bytes(path.read_bytes().replace(b'\t506\t', b'\t507\t'))
        with self.assertRaisesRegex(m.AdmissionError, 'STALE'): text_evidence.text_contracts(m.EvidenceScope(self.root))

    def test_compiler_target_flags_and_observed_version_identity_are_strict(self):
        rust = {'kind':'rustc', 'version_vv':'unit compiler\n',
                'version_vv_sha256':hashlib.sha256(b'unit compiler\n').hexdigest(), 'flags':['--edition', '2021']}
        c = {'kind':'cl', 'version':'unit C compiler', 'version_sha256':'1' * 64,
             'vcvars_sha256':'2' * 64, 'flags':['-nologo', '-std:c11', '-TC', '-W4', '-WX', '-O2', '-MT']}
        for target, value in (('rust', rust), ('c', c)):
            cert_evidence._toolchain(value, target)
            for change in ('kind', 'flags', 'unknown', 'version-hash'):
                bad = json.loads(json.dumps(value))
                if change == 'kind': bad['kind'] = 'foreign'
                if change == 'flags': bad['flags'] = []
                if change == 'unknown': bad['echoed_pass'] = True
                if change == 'version-hash': bad['version_vv_sha256' if target == 'rust' else 'version_sha256'] = True
                with self.subTest(target=target, change=change), self.assertRaises(m.AdmissionError):
                    cert_evidence._toolchain(bad, target)



class CtCertificationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def test_seven_independent_references_bind_bool_meta_and_instance_oracles(self):
        rows = cte_evidence.cte_reference_contracts(m.EvidenceScope(ROOT))
        self.assertEqual(len(rows), 7)
        self.assertEqual(rows['bool']['oracle_value'], '7')
        self.assertEqual(rows['meta-fields3']['oracle_value'], '43')
        self.assertEqual(rows['child-ct-chain40']['oracle_value'], '40')
        self.assertEqual(json.loads(rows['meta-fields2']['caps']),
                         ['ac.source.cte.scalar/v1', 'cte.meta.fields/v1'])
        self.assertEqual(json.loads(rows['choose-instance42']['caps']),
                         ['ac.source.cte.scalar/v1', 'ac.source.instances.scalar/v1'])
        self.assertEqual({row['execution'] for row in rows.values()}, {'UNOBSERVED'})

    def test_changed_ct_source_or_either_independent_reference_is_stale(self):
        rows = cte_evidence.cte_reference_contracts(m.EvidenceScope(ROOT))
        names = {cte_evidence.REFERENCE_CONTRACT}
        for row in rows.values(): names.update(row[k] for k in ('source_file', 'rust_reference', 'c_reference'))
        for name in names:
            path = self.root / name; path.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / name, path)
        self.assertEqual(len(cte_evidence.cte_reference_contracts(m.EvidenceScope(self.root))), 7)
        for key in ('source_file', 'rust_reference', 'c_reference'):
            path = self.root / rows['bool'][key]; raw = path.read_bytes(); path.write_bytes(raw + b'changed')
            with self.subTest(role=key), self.assertRaisesRegex(m.AdmissionError, 'STALE'):
                cte_evidence.cte_reference_contracts(m.EvidenceScope(self.root))
            path.write_bytes(raw)

    def test_missing_duplicate_foreign_or_wrong_version_cap_is_rejected_before_io(self):
        row = cte_evidence.cte_reference_contracts(m.EvidenceScope(ROOT))['sum42']
        for caps in ([], ['ac.source.cte.scalar/v10'], ['ac.source.instances.scalar/v1'],
                     ['ac.source.cte.scalar/v1'] * 2, ['ac.source.cte.scalar/v1', True],
                     ['ac.source.cte.scalar/v1', 'foreign']):
            contract = dict(row, caps=json.dumps(caps))
            with self.subTest(caps=caps), self.assertRaises(m.AdmissionError):
                cert_evidence.cte_certification(None, m.EvidenceScope(ROOT), '', {}, {}, contract)
        with patch.object(cert_evidence, '_certification', return_value='admitted') as reader:
            self.assertEqual(cert_evidence.cte_certification(None, m.EvidenceScope(ROOT), '', {}, {}, row), 'admitted')
            self.assertEqual(reader.call_count, 1)

    def test_duplicate_or_missing_ct_certification_rows_cannot_close_the_set(self):
        row = cte_evidence.cte_reference_contracts(m.EvidenceScope(ROOT))['sum42']
        worker = ROOT / 'experimental/ace/target/release/a2x-worker.exe'
        producer = dict(producer_commit='1'*40, worker_path=str(worker), worker_sha256='2'*64, cwd=str(ROOT))
        pin = dict(target='rust', identity_request=dict(op='identity', certification_target='rust'),
                   request=dict(case_id='941/source-cte/sum42'), toolchain={}, source_sha256=row['source_sha256'],
                   reference_sha256=row['rust_reference_sha256'], actual_files={}, expected_stdout_hex='', expected_stderr_hex='')
        entry = dict(case='sum42', prefix='sum42/rust', pin=pin)
        for rows in ([], [entry], [entry, entry]):
            (self.root / 'bindings.json').write_text(json.dumps(dict(producer=producer, rows=rows)), newline='\n')
            with self.subTest(count=len(rows)), patch.object(cert_evidence, 'cte_certification', return_value={'judgement':'PASS'}), patch.object(daily_evidence, 'certified_daily_emission', return_value={'judgement':'PASS', 'daily_judgement':'PASS'}), self.assertRaises(m.AdmissionError):
                cte_evidence.admit_cte_certifications(m.EvidenceScope(ROOT), m.EvidenceScope(self.root), 'bindings.json')


class DailyEmissionTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self, target='rust', profile='scalar', change=None):
        fixture = Path(tempfile.mkdtemp(prefix='daily-', dir=self.root))
        inputs = fixture / 'inputs'; evidence = fixture / 'evidence'
        inputs.mkdir(exist_ok=True); evidence.mkdir(exist_ok=True)
        source = b'main() { return 42 }'
        (inputs / 'source.at').write_bytes(source); (inputs / 'worker.exe').write_bytes(b'unit worker')
        sha = lambda b: hashlib.sha256(b).hexdigest()
        producer = dict(producer_commit='1'*40, worker_path=str(inputs/'worker.exe'),
                        worker_sha256=sha(b'unit worker'), cwd=str(inputs))
        nonce = '2'*48; prefix = 'case/daily-' + target
        out = evidence / prefix; out.mkdir(parents=True, exist_ok=True)
        original = dict(source_path=str(inputs/'source.at'), profile=profile, entry='main', caps=[], nonce='3'*48)
        req = {k:original[k] for k in ('source_path','profile','entry','caps')}
        req.update(op='daily-emit' if target=='rust' else 'daily-emit-c', nonce=nonce, out_dir=str(out))
        rt = profile in ('record','text','text-record');coll=profile=='collections'
        if coll:original['mode']=req['mode']='debug'
        names = ['main.rs','a2x_runtime.rs','generated.rs'] if target=='rust' else ['main.c','a2x_c_coll_runtime.h' if coll else 'a2x_c_rt_runtime.h' if rt else 'a2x_c_runtime.h','generated.c']
        files=[]
        for name in names:
            raw=('unit bytes: '+name).encode(); (out/name).write_bytes(raw)
            files.append(dict(path=name,sha256=sha(raw)))
        emitted_profile = ('auto.source.'+profile+'.draft' if rt or coll else 'auto.source.'+profile+'-i32.draft') if target=='rust' else profile
        manifest=dict(schema='auto.a2x.'+('rust' if target=='rust' else 'c')+('.collections' if coll else '.record-text' if rt else '')+'.fileset/1',
                      input_hash=scalar_evidence._fnv1a64(source), entry='main', files=files)
        if coll:manifest['source_sha256']=sha(source)
        if target=='rust':
            backend=dict(lang='rust',edition='2021',backend='auto-a2x-rust',backend_version='0.1.0')
            if rt or coll:backend['resource_runtime']='a2x-coll/1' if coll else 'a2x-rt/1'
            if coll:manifest['execution_config']=cert_evidence.collections_execution_config('debug')
            manifest.update(source_profile=emitted_profile,hir_domain='collections-draft' if coll else profile+'-i32-draft',entry_fn='fn_main',target=backend)
        else: manifest['functions']=[dict(decl='main',sem_fp='4'*64,origin=[0,len(source)])]
        if change=='wrong-coll-config':
            if target=='rust':manifest['execution_config']['mode']='release'
            else:manifest['source_sha256']='0'*64
        if change=='wrong-manifest-family': manifest['schema']='auto.a2x.foreign.fileset/1'
        (out/'manifest.json').write_text(json.dumps(manifest),newline='\n')
        golden={name:sha((out/name).read_bytes()) for name in names+['manifest.json']}
        response=dict(status='ok',tool='a2x-worker',protocol='a2xworker/1',
                      recipe=('ac-a2r/' if target=='rust' else 'ac-a2c/')+('hir-collections-emit-v1' if coll else 'hir-record-text-emit-v1' if rt else 'hir-scalar-emit-v1'),
                      mode=req['op'],nonce=nonce,profile=emitted_profile,input_hash=scalar_evidence._fnv1a64(source),
                      source_sha256=sha(source),entry_fn='fn_main',manifest_sha256=golden['manifest.json'],
                      files=[dict(f,bytes=(out/f['path']).stat().st_size) for f in files],spawned_rustc=0)
        if target=='c': response.update(target='c',spawned_cl=0)
        if change=='spawn-bool': response['spawned_rustc']=False
        if change=='spawn-compiler': response['spawned_rustc']=1
        if change=='wrong-nonce': response['nonce']='5'*48
        if change=='wrong-source': response['source_sha256']='6'*64
        if change=='duplicate-file': response['files'][1]=dict(response['files'][0])
        if change=='missing-file': response['files'].pop()
        if change=='bytes-bool': response['files'][0]['bytes']=True
        if change=='wrong-cap': req['caps']=['ac.source.cte.scalar/v1']
        if change=='wrong-output-root': req['out_dir']=str(evidence/'foreign')
        if change=='wrong-command-target': req['op']='daily-emit-c' if target=='rust' else 'daily-emit'
        if change=='reused-cert-nonce': original['nonce']=nonce
        argv=[producer['worker_path'],json.dumps(req,separators=(',',':'))]
        meter=dict(argv=argv,exit=0,error=None,wall_ms=1,workload_cpu_ms=1,meter_cpu_ms=0,total_cpu_ms=1,
                   peak_process_bytes=1024,peak_job_bytes=2048,job_processes=3 if change=='extra-child' else 2)
        refs={}
        for name,raw in [('stdout',json.dumps(response).encode()),('stderr',b'')]:
            path=evidence/(prefix+'.'+name);path.write_bytes(raw)
            refs[name]=dict(path=prefix+'.'+name,bytes=len(raw),sha256=sha(raw))
        record=dict(schema='ace.m1.command/1',argv=argv,cwd=producer['cwd'],exit=0,meter=meter,
                    worker_sha256=producer['worker_sha256'],producer_commit=producer['producer_commit'],**refs)
        (evidence/(prefix+'.command.json')).write_text(json.dumps(record),newline='\n')
        pin=dict(target=target,request=original,source_sha256=sha(source),actual_files=golden)
        return m.EvidenceScope(evidence),m.EvidenceScope(inputs),pin,producer,out

    def test_all_six_profiles_both_targets_bind_current_complete_daily_files(self):
        for target in ('rust','c'):
            for profile in ('core','scalar','record','text','text-record','collections'):
                scope,inputs,pin,producer,_=self.sample(target,profile)
                with self.subTest(target=target,profile=profile), patch('subprocess.Popen',side_effect=AssertionError('reader is read-only')):
                    result=daily_evidence.daily_emission(scope,inputs,'case/'+target,pin,producer)
                    self.assertEqual(result['judgement'],'PASS')
                    self.assertTrue(result['compiler_spawn_zero'])

    def test_rehashed_wrong_response_or_request_cannot_supply_daily_authority(self):
        changes=('spawn-bool','spawn-compiler','wrong-nonce','wrong-source','duplicate-file','missing-file',
                 'bytes-bool','wrong-cap','wrong-output-root','wrong-command-target','reused-cert-nonce',
                 'extra-child','wrong-manifest-family')
        for target in ('rust','c'):
            for change in changes:
                scope,inputs,pin,producer,_=self.sample(target,'text',change)
                with self.subTest(target=target,change=change),self.assertRaises(m.AdmissionError):
                    daily_evidence.daily_emission(scope,inputs,'case/'+target,pin,producer)

    def test_emitted_bytes_must_equal_files_frozen_before_certification(self):
        for target in ('rust','c'):
            scope,inputs,pin,producer,out=self.sample(target)
            (out/('main.rs' if target=='rust' else 'main.c')).write_bytes(b'changed emission')
            with self.subTest(target=target),self.assertRaisesRegex(m.AdmissionError,'STALE'):
                daily_evidence.daily_emission(scope,inputs,'case/'+target,pin,producer)

    def test_unlisted_executable_or_directory_is_refused(self):
        for member in ('extra.exe','subdirectory'):
            scope,inputs,pin,producer,out=self.sample()
            path=out/member
            if member.endswith('.exe'):path.write_bytes(b'unlisted compiler output')
            else:path.mkdir()
            with self.subTest(member=member),self.assertRaisesRegex(m.AdmissionError,'undeclared daily output'):
                daily_evidence.daily_emission(scope,inputs,'case/rust',pin,producer)
            if path.is_dir():path.rmdir()
            else:path.unlink()

    def test_certification_pass_never_masks_missing_daily_command(self):
        scope,inputs,pin,producer,_=self.sample()
        (scope.root/'case/daily-rust.command.json').unlink()
        with self.assertRaises(m.AdmissionError):
            daily_evidence.certified_daily_emission(scope,inputs,'case/rust',pin,producer,{'judgement':'PASS'})


    def test_collections_daily_mode_and_full_source_cannot_be_rehashed_away(self):
        for target in ('rust','c'):
            scope,inputs,pin,producer,_=self.sample(target,'collections','wrong-coll-config')
            with self.subTest(target=target),self.assertRaises(m.AdmissionError):
                daily_evidence.daily_emission(scope,inputs,'case/'+target,pin,producer)


class TextSourceObservationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def fixture(self, av=True, trap=False):
        case='v03_char_at_negative' if trap else 'v03_nul_data'
        contract=text_evidence.text_contracts(m.EvidenceScope(ROOT))[case]
        source=(ROOT/contract['source_file']).read_bytes()
        import base64
        recipe='ac-av/owned-run-v1' if av else 'ac/mir-native-scalar-v1'
        request=dict(recipe=recipe,nonce='a'*64,source_b64=base64.b64encode(source).decode(),entry='d_main',caps=[],store_root='owned-store')
        if av:request['steps']=[dict(op='run')]
        if not av:request['workdir']='native-work'
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        protocol='avworker/1' if av else 'mnworker/1'
        if av:
            owned=dict(result=dict(kind='i32',bits='21'),live_after_release=0,allocs=[dict(id=0,bytes=3)],frees=[dict(id=0)])
            step=dict(op='run',outcome='trap' if trap else 'success',value=None if trap else 33,owned=None if trap else owned,
                      trap='av.trap.str-index' if trap else None,diag=None,source_hash=scalar_evidence._fnv1a64(source),wall_ms=1)
            ok=dict(recipe=recipe,steps=[step],worker_version='avworker/1.0',av_core_version='av.owned/1')
        else:
            events=text_q4.TRAP_TRACES[case] if trap else b'LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\n'
            ok=dict(recipe=recipe,worker_version='mnworker/1.0',mn_abi='msvc-x64/priv-i32-bool-v1',mn_target='x86_64-pc-windows-msvc',
                    counters=dict.fromkeys(text_q4.NATIVE_COUNTERS,1),spans=dict.fromkeys(text_q4.NATIVE_SPANS,0),
                    manifest_fp='a'*16,exit_code=70 if trap else 33,stdout_hex='',stderr_hex=events.hex())
        response=dict(schema=protocol,nonce=request['nonce'],request_hash=request['request_hash'],ok=ok)
        wire=dict(protocol=protocol,recipe=recipe,exit_code=0,transport_error=None,stderr=b'',request=json.dumps(request,separators=(',',':')).encode())
        return wire,response,dict(recipe=recipe),contract

    def admit(self,wire,response,check,contract):
        return text_q4.text_observation(self.scope,dict(wire,stdout=json.dumps(response).encode()),check,contract)

    def test_source_bound_value_and_named_trap_pass_on_both_recipes(self):
        for av in (True,False):
            for trap in (True,False):
                args=self.fixture(av,trap)
                with self.subTest(av=av,trap=trap),patch('subprocess.Popen',side_effect=AssertionError('read-only')):
                    self.assertEqual(self.admit(*args)['judgement'],'PASS')

    def test_both_backends_same_wrong_value_or_trap_stays_red(self):
        for av in (True,False):
            for trap in (True,False):
                wire,response,check,contract=self.fixture(av,trap)
                if av:
                    step=response['ok']['steps'][0]
                    if trap:step['trap']='av.trap.char-invalid'
                    else:step['value']=34;step['owned']['result']['bits']='22'
                else:
                    if trap:response['ok']['stderr_hex']=bytes.fromhex(response['ok']['stderr_hex']).replace(b'code=-10',b'code=-8').hex()
                    else:response['ok']['exit_code']=34
                with self.subTest(av=av,trap=trap):self.assertEqual(self.admit(wire,response,check,contract)['judgement'],'FAIL')

    def test_native_exit_70_without_named_runtime_failure_never_proves_trap(self):
        wire,response,check,contract=self.fixture(False,True)
        response['ok']['stderr_hex']=b'LIT len=3\nLIT id=1\nFREE id=1\n'.hex()
        self.assertEqual(self.admit(wire,response,check,contract)['judgement'],'FAIL')

    def test_borrowed_static_literal_trap_does_not_claim_owned_allocation(self):
        import base64
        wire,response,check,_=self.fixture(False,True)
        contract=original_data.source_contracts(m.EvidenceScope(ROOT))['T08']
        request=json.loads(wire['request'])
        request.update(source_b64=base64.b64encode((ROOT/contract['source_file']).read_bytes()).decode(),
                       caps=json.loads(contract['caps']))
        request.pop('request_hash')
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        response['request_hash']=request['request_hash'];wire['request']=json.dumps(request).encode()
        for events,expected in [(b'RT-FAIL code=-10\n','PASS'),(b'','FAIL'),
                                (b'RT-FAIL code=-8\n','FAIL'),
                                (b'LIT len=3\nLIT id=1\nRT-FAIL code=-10\nFREE id=1\n','FAIL')]:
            response['ok']['stderr_hex']=events.hex()
            with self.subTest(events=events),patch('subprocess.Popen',side_effect=AssertionError('read-only')):
                result=text_q4.text_observation(self.scope,dict(wire,stdout=json.dumps(response).encode()),check,contract,
                                                record_family=True,trap_trace=original_data.T08_NATIVE_TRACE)
                self.assertEqual(result['judgement'],expected)

    def test_native_text_events_and_av_ledger_refuse_missing_or_foreign_cleanup(self):
        for raw in (b'LIT id=1\n',b'FREE id=1\n',b'LIT id=1\nFREE id=1\nFREE id=1\n',
                    b'RT-FAIL code=-10',b'child PASS\n',b'LIT id='+b'1'*200+b'\n'):
            with self.subTest(raw=raw[:30]),self.assertRaises(m.AdmissionError):text_q4.native_events(raw)
        for change in ('missing-free','wrong-free','live','bool-id','wrong-kind'):
            _,response,_,_=self.fixture();value=response['ok']['steps'][0]['owned']
            if change=='missing-free':value['frees']=[]
            if change=='wrong-free':value['frees'][0]['id']=2
            if change=='live':value['live_after_release']=1
            if change=='bool-id':value['allocs'][0]['id']=False
            if change=='wrong-kind':value['result']['kind']='bool'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):text_q4.owned_i32(value)

    def test_real_frontend_diagnostic_is_semantic_fail_not_tool_failure_or_pass(self):
        for av in (True,False):
            wire,response,check,contract=self.fixture(av)
            if av:
                step=response['ok']['steps'][0];step.update(outcome='diag',value=None,owned=None,trap=None,
                    diag=dict(stage='frontend',code='source.syntax',message='expected expression, found return'))
            else:
                ok=response['ok']
                for key in ('manifest_fp','exit_code','stderr_hex','stdout_hex'):ok.pop(key)
                ok['counters']={'frontend_reject':1};ok['spans']={'frontend':1}
                response['diagnostic']=dict(code='mn.unsupported.frontend',message='Text/source.syntax: expected expression')
            with self.subTest(av=av):
                result=self.admit(wire,response,check,contract)
                self.assertEqual(result['judgement'],'FAIL');self.assertEqual(result['observed']['kind'],'diagnostic')

class ScalarQ4ObservationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp
    wire = ScalarObservationTests.wire

    def sample(self, target='av', value=42, *, trace=False):
        wire, check = self.wire(target, value, caps=(scalar_q4.TRACE_CAP,) if trace else ())
        response = json.loads(wire['stdout'])
        if target == 'native':
            response['ok']['counters'] = {name: 0 for name in text_q4.NATIVE_COUNTERS}
            response['ok']['counters'].update(link_invocations=1, run_invocations=1)
            response['ok']['spans'] = {name: 0 for name in text_q4.NATIVE_SPANS}
        wire['stdout'] = json.dumps(response).encode()
        contract = dict(source_sha256=check['source_files'][0]['sha256'],
                        oracle_kind='value', oracle_value='42', marks='',
                        trace='true' if trace else 'false')
        return wire, check, contract

    def judge(self, wire, check, contract):
        return scalar_q4.scalar_q4_observation(self.scope, wire, check, contract)

    def test_same_wrong_values_and_value70_are_independent(self):
        for target in ('av', 'native'):
            with self.subTest(target=target):
                self.assertEqual(self.judge(*self.sample(target))['judgement'], 'PASS')
                self.assertEqual(self.judge(*self.sample(target, 43))['judgement'], 'FAIL')
                wire, check, contract = self.sample(target, 70)
                self.assertEqual(self.judge(wire, check, contract)['judgement'], 'FAIL')
                contract['oracle_value'] = '70'
                self.assertEqual(self.judge(wire, check, contract)['judgement'], 'PASS')

    def test_native_generic70_never_proves_any_named_trap(self):
        for trap in scalar_q4.TRAPS:
            wire, check, contract = self.sample('native', 70)
            contract.update(oracle_kind='trap', oracle_value=trap)
            self.assertEqual(self.judge(wire, check, contract)['judgement'], 'UNOBSERVED')
        wire, check, contract = self.sample()
        contract.update(oracle_kind='trap', oracle_value='Overflow')
        response = json.loads(wire['stdout']);step = response['ok']['steps'][0]
        step.update(outcome='trap', channel=None, value=None, trap='av.trap.int-overflow')
        wire['stdout'] = json.dumps(response).encode()
        self.assertEqual(self.judge(wire, check, contract)['judgement'], 'PASS')
        step['trap'] = 'av.trap.div-by-zero';wire['stdout'] = json.dumps(response).encode()
        self.assertEqual(self.judge(wire, check, contract)['judgement'], 'FAIL')

    def test_native_trap_probe_names_close_exactly_the_expected_cell(self):
        # PLAN-1000 F-1000-01 (owner option a): the response-side named trap
        # (trap-probe observation build) closes ONLY the matching oracle.
        closing = {'Overflow':'mn.trap.add-overflow','DivByZero':'mn.trap.div-zero',
                   'DivOverflow':'mn.trap.div-overflow','RemOverflow':'mn.trap.div-overflow',
                   'RemByZero':'mn.trap.div-zero'}
        for oracle, native in closing.items():
            with self.subTest(oracle=oracle):
                wire, check, contract = self.sample('native', 70)
                contract.update(oracle_kind='trap', oracle_value=oracle)
                response = json.loads(wire['stdout'])
                response['ok'].update(trap=native, trap_reason=1, trap_observation='trap-probe/1')
                wire['stdout'] = json.dumps(response).encode()
                self.assertEqual(self.judge(wire, check, contract)['judgement'], 'PASS')
        # a machine-valid name that contradicts the oracle FAILS
        wire, check, contract = self.sample('native', 70)
        contract.update(oracle_kind='trap', oracle_value='DivByZero')
        response = json.loads(wire['stdout'])
        response['ok'].update(trap='mn.trap.div-overflow', trap_reason=2, trap_observation='trap-probe/1')
        wire['stdout'] = json.dumps(response).encode()
        self.assertEqual(self.judge(wire, check, contract)['judgement'], 'FAIL')
        # the whole int-overflow family closes the shared Overflow oracle
        for native in ('mn.trap.sub-overflow','mn.trap.mul-overflow','mn.trap.neg-overflow'):
            with self.subTest(family=native):
                wire, check, contract = self.sample('native', 70)
                contract.update(oracle_kind='trap', oracle_value='Overflow')
                response = json.loads(wire['stdout'])
                response['ok'].update(trap=native, trap_reason=4, trap_observation='trap-probe/1')
                wire['stdout'] = json.dumps(response).encode()
                self.assertEqual(self.judge(wire, check, contract)['judgement'], 'PASS')

    def test_native_trap_probe_fields_are_refused_when_malformed(self):
        refusals = {
            'unknown-name': dict(trap='mn.trap.sudden-death', trap_reason=1, trap_observation='trap-probe/1'),
            'foreign-name': dict(trap='av.trap.int-overflow', trap_reason=3, trap_observation='trap-probe/1'),
            'missing-mode': dict(trap='mn.trap.add-overflow', trap_reason=3),
            'missing-reason': dict(trap='mn.trap.add-overflow', trap_observation='trap-probe/1'),
            'null-trap': dict(trap=None, trap_reason=3, trap_observation='trap-probe/1'),
            'bad-reason': dict(trap='mn.trap.add-overflow', trap_reason=9, trap_observation='trap-probe/1'),
            'wrong-mode': dict(trap='mn.trap.add-overflow', trap_reason=3, trap_observation='guess/0'),
        }
        for label, fields in refusals.items():
            with self.subTest(label=label), self.assertRaises(m.AdmissionError):
                wire, check, contract = self.sample('native', 70)
                contract.update(oracle_kind='trap', oracle_value='Overflow')
                response = json.loads(wire['stdout'])
                response['ok'].update(fields)
                wire['stdout'] = json.dumps(response).encode()
                self.judge(wire, check, contract)
        # trap fields never rescue a VALUE oracle whose exit disagrees
        wire, check, contract = self.sample('native', 70)
        response = json.loads(wire['stdout'])
        response['ok'].update(trap='mn.trap.add-overflow', trap_reason=3, trap_observation='trap-probe/1')
        wire['stdout'] = json.dumps(response).encode()
        self.assertEqual(self.judge(wire, check, contract)['judgement'], 'FAIL')

    def test_original_mark_order_and_short_circuit_are_required(self):
        for target in ('av', 'native'):
            wire, check, contract = self.sample(target, trace=True)
            contract['marks'] = 'mark_b,mark_a'
            response = json.loads(wire['stdout'])
            def observed(marks):
                if target == 'av': response['ok']['steps'][0]['trace'] = list(marks)
                else: response['ok']['stderr_hex'] = marks.encode().hex()
                wire['stdout'] = json.dumps(response).encode()
                return self.judge(wire, check, contract)['judgement']
            self.assertEqual(observed('ba'), 'PASS')
            self.assertEqual(observed('ab'), 'FAIL')
            contract['marks'] = ''
            self.assertEqual(observed('b'), 'FAIL')
            self.assertEqual(observed(''), 'PASS')

    def test_real_diagnostics_remain_fail_even_when_parent_claims_pass(self):
        for target in ('av', 'native'):
            wire, check, contract = self.sample(target)
            response = json.loads(wire['stdout'])
            if target == 'av':
                response['ok']['steps'][0].update(outcome='diag', value=None, channel=None,
                    diag=dict(stage='frontend', code='av.resource.frontend-nesting', message='original budget refusal'))
            else:
                for key in ('manifest_fp','exit_code','stderr_hex','stdout_hex'): response['ok'].pop(key)
                response['diagnostic'] = dict(code='mn.unsupported.frontend', message='while is outside this profile')
            wire['stdout'] = json.dumps(response).encode()
            value = first_slice.retain_parent_status(self.judge(wire, check, contract), 'PASS')
            self.assertEqual(value['judgement'], 'FAIL')
            self.assertEqual(value['observed']['kind'], 'diagnostic')

    def test_nonce_source_extra_fields_and_link_counter_are_rejected(self):
        for change in ('nonce','source','extra','link'):
            wire, check, contract = self.sample('native')
            response = json.loads(wire['stdout'])
            if change == 'nonce': response['nonce'] = 'other'
            elif change == 'source': contract['source_sha256'] = '0'*64
            elif change == 'extra': response['ok']['unknown'] = True
            else: response['ok']['counters']['link_invocations'] = 0
            wire['stdout'] = json.dumps(response).encode()
            with self.subTest(change=change), self.assertRaises(m.AdmissionError):
                self.judge(wire, check, contract)


class ScalarCertificationVariantTests(unittest.TestCase):
    def test_missing_release_is_never_filled_from_default_certificate(self):
        for target, count, expected in [('rust','1','UNOBSERVED'),('rust','0','PASS'),('c','1','PASS')]:
            value=scalar_cert.required_variants({'judgement':'PASS'},{'rust_release_runs':count},target)
            self.assertEqual(value['judgement'],expected)
        value=scalar_cert.required_variants({'judgement':'FAIL'},{'rust_release_runs':'1'},'rust')
        self.assertEqual(value['judgement'],'FAIL')

    def test_unknown_configuration_and_child_verdict_types_are_rejected(self):
        for target,count,verdict in [('other','1','PASS'),('rust',True,'PASS'),('rust','2','PASS'),('rust','1',True)]:
            with self.subTest(target=target,count=count,verdict=verdict),self.assertRaises(m.AdmissionError):
                scalar_cert.required_variants({'judgement':verdict},{'rust_release_runs':count},target)

    def test_release_does_not_mask_failed_default_and_failed_release_stays_red(self):
        for default,release,expected in [('PASS','PASS','PASS'),('PASS','FAIL','FAIL'),('FAIL','PASS','FAIL')]:
            value=scalar_cert.required_variants({'judgement':default},{'rust_release_runs':'1'},'rust',
                                               dict(judgement=release,receipt_sha256='1'*64))
            self.assertEqual(value['judgement'],expected)


class ScalarReleaseProcessTests(unittest.TestCase):
    def sample(self):
        argv=['rustc',*scalar_release.FLAGS,'-o','D:/scope/release.exe','main.rs']
        phase=dict(argv=argv,cwd='D:/scope',deadline_ms=120000,wall_ms=1,
                   terminal=dict(exit_code=0,stdout_hex='',stderr_hex=''))
        return phase,argv

    def test_exact_original_release_flags_and_full_raw_terminal_are_required(self):
        phase,argv=self.sample()
        self.assertEqual(scalar_release.release_phase(phase,argv,'D:/scope',120000)['exit_code'],0)
        for change in ('flags','mode','exit','deadline','wall','extra','lossy'):
            phase,argv=self.sample();phase['argv']=list(argv)
            if change=='flags':phase['argv'].remove('-O')
            elif change=='mode':phase['argv'][5]='debug-assertions=on'
            elif change=='exit':phase['terminal']['exit_code']=True
            elif change=='deadline':phase['deadline_ms']=120001
            elif change=='wall':phase['wall_ms']=120001
            elif change=='extra':phase['echoed_pass']=True
            else:phase['terminal']['stdout_hex']='GG'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):
                scalar_release.release_phase(phase,argv,'D:/scope',120000)

    def test_record_text_optimization_keeps_902_flags_distinct_from_scalar(self):
        argv=['rustc',*scalar_release.RT_FLAGS,'-o','D:/scope/release.exe','main.rs']
        phase=dict(argv=argv,cwd='D:/scope',deadline_ms=120000,wall_ms=1,
                   terminal=dict(exit_code=0,stdout_hex='',stderr_hex=''))
        self.assertEqual(scalar_release.release_phase(phase,argv,'D:/scope',120000)['exit_code'],0)
        for wrong in (['--edition','2021'],scalar_release.FLAGS,
                      ['--edition','2021','-C','opt-level=3']):
            candidate=dict(phase,argv=['rustc',*wrong,'-o','D:/scope/release.exe','main.rs'])
            with self.subTest(wrong=wrong),self.assertRaises(m.AdmissionError):
                scalar_release.release_phase(candidate,argv,'D:/scope',120000)


class RecordTextSourceTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def data(self):
        import base64
        contract=record_text.reference_contracts(m.EvidenceScope(ROOT))[('record-text-cert-925','M_COPY_SNAP')]
        source=(ROOT/contract['source_file']).read_bytes();recipe='ac-av/data-run-v1'
        request=dict(recipe=recipe,nonce='b'*64,source_b64=base64.b64encode(source).decode(),entry='d_main',caps=[],store_root='unit-store',steps=[dict(op='run')])
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        step=dict(op='run',outcome='success',value=221,typed=dict(kind='i32',bits='dd'),trap=None,diag=None,source_hash=scalar_evidence._fnv1a64(source),wall_ms=1,trace=[])
        response=dict(schema='avworker/1',nonce=request['nonce'],request_hash=request['request_hash'],ok=dict(recipe=recipe,steps=[step],worker_version='avworker/1.0',av_core_version='av.data/1'))
        wire=dict(protocol='avworker/1',recipe=recipe,exit_code=0,transport_error=None,stderr=b'',request=json.dumps(request).encode())
        return wire,response,dict(recipe=recipe),contract

    def judge(self,wire,response,check,contract):
        return text_q4.text_observation(self.scope,dict(wire,stdout=json.dumps(response).encode()),check,contract,
                                        record_family=True,trap_trace=record_text.TRACES.get(contract['case']))

    def test_record_typed_i32_is_not_an_invented_owned_ledger(self):
        self.assertEqual(self.judge(*self.data())['judgement'],'PASS')
        for change in ('wrong-value','wide','noncanonical','inconsistent'):
            wire,response,check,contract=self.data();step=response['ok']['steps'][0]
            if change=='wrong-value':step['value']=222;step['typed']['bits']='de'
            elif change=='wide':step['typed']['kind']='i64'
            elif change=='noncanonical':step['typed']['bits']='0dd'
            else:step['value']=222
            with self.subTest(change=change):
                if change=='wrong-value':self.assertEqual(self.judge(wire,response,check,contract)['judgement'],'FAIL')
                else:
                    with self.assertRaises(m.AdmissionError):self.judge(wire,response,check,contract)

    def test_text_capability_and_named_trap_bind_both_consumers(self):
        for av in (True,False):
            wire,response,check,_=TextSourceObservationTests.fixture(self,av,True)
            contract=record_text.reference_contracts(m.EvidenceScope(ROOT))[('record-text-cert-925','T_CHAR_AT_NEG')]
            request=json.loads(wire['request']);request.pop('request_hash');request['caps']=['ac.runtime.text.v1']
            request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
            response['request_hash']=request['request_hash'];wire['request']=json.dumps(request).encode()
            self.assertEqual(self.judge(wire,response,check,contract)['judgement'],'PASS')
            request['caps']=[];wire['request']=json.dumps(request).encode()
            with self.assertRaises(m.AdmissionError):self.judge(wire,response,check,contract)

    def test_record_domain_cannot_be_replaced_with_owned_or_unknown_profile(self):
        wire,response,check,contract=self.data();check['recipe']='ac-av/owned-run-v1'
        with self.assertRaises(m.AdmissionError):self.judge(wire,response,check,contract)
        wire,response,check,contract=self.data();contract['profile']='unknown'
        with self.assertRaises(m.AdmissionError):self.judge(wire,response,check,contract)


class CollectionsConfigurationTests(unittest.TestCase):
    def test_original_target_specific_configuration_is_closed_and_canonical(self):
        for target in ('rust','c'):
            for mode in ('debug','release'):
                config=cert_evidence.collections_execution_config(mode)
                render=lambda value:json.dumps(value,separators=(',',':')) if target=='rust' else value
                cert_evidence.collections_certificate_config(dict(execution_config=render(config)),target,mode)
                for change in ('opposite-mode','budget','boolean-type','fault','extra','representation'):
                    wrong=dict(config)
                    if change=='opposite-mode':wrong['mode']='release' if mode=='debug' else 'debug'
                    elif change=='budget':wrong['budget_work_units']=16777215
                    elif change=='boolean-type':wrong['events']=0
                    elif change=='fault':wrong['fault_alloc']=1
                    elif change=='extra':wrong['passed']=True
                    value=render(wrong)
                    if change=='representation':value=wrong if target=='rust' else json.dumps(wrong,separators=(',',':'))
                    with self.subTest(target=target,mode=mode,change=change),self.assertRaises(m.AdmissionError):
                        cert_evidence.collections_certificate_config(dict(execution_config=value),target,mode)


class CollectionsSourceTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def fixture(self,case='L01'):
        import base64
        row=collections_evidence.source_contracts(m.EvidenceScope(ROOT))[case]
        source=(ROOT/row['source_file']).read_bytes();recipe=collections_evidence.AV_RECIPE
        request=dict(recipe=recipe,nonce='a'*64,source_b64=base64.b64encode(source).decode(),entry='d_main',caps=[],store_root='unit-store',steps=[dict(op='run')])
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        trap=case=='L06'
        step=dict(op='run',outcome='trap' if trap else 'success',value=None if trap else int(row['oracle_value']),collections=None,
                  trap='av.trap.list-index' if trap else None,diag=None,source_hash=scalar_evidence._fnv1a64(source),wall_ms=1)
        response=dict(schema='avworker/1',nonce=request['nonce'],request_hash=request['request_hash'],ok=dict(recipe=recipe,steps=[step],worker_version='avworker/1.0',av_core_version='av.collections/1'))
        wire=dict(protocol='avworker/1',recipe=recipe,exit_code=0,transport_error=None,stderr=b'',request=json.dumps(request).encode())
        return wire,response,dict(recipe=recipe),row

    def judge(self,wire,response,check,row):
        return collections_evidence.collections_observation(self.scope,dict(wire,stdout=json.dumps(response).encode()),check,row,{})

    def test_original_source_value_and_named_trap_each_have_independent_oracle(self):
        for case in ('L01','L06'):
            self.assertEqual(self.judge(*self.fixture(case))['judgement'],'PASS')
            wire,response,check,row=self.fixture(case);step=response['ok']['steps'][0]
            if case=='L01':step['value']=43
            else:step['trap']='av.trap.div-by-zero'
            self.assertEqual(self.judge(wire,response,check,row)['judgement'],'FAIL')

    def test_source_recipe_does_not_accept_fixture_ledger_or_foreign_frame(self):
        for change in ('fixture','nonce','caps','source','unknown'):
            wire,response,check,row=self.fixture()
            if change=='fixture':response['ok']['steps'][0]['collections']={'result':42}
            elif change=='nonce':response['nonce']='b'*64
            elif change=='unknown':response['ok']['steps'][0]['passed']=True
            else:
                request=json.loads(wire['request']);request.pop('request_hash')
                if change=='caps':request['caps']=['ac.runtime.text.v1']
                else:request['source_b64']='Zm4gbWFpbigpIGludCB7IHJldHVybiA0MiB9'
                request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
                wire['request']=json.dumps(request).encode();response['request_hash']=request['request_hash']
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(wire,response,check,row)

    def summary(self):
        values={key:0 for key in collections_evidence.SUMMARY_KEYS}
        values.update(process_sample_ok=1,allocs=1,frees=1)
        prefix='LIT len=3\nLIT id=1\nTEXT-SCALARS n=3\nFREE id=1\nMN-COLL-TRAP code=21\n'
        return (prefix+''.join('MN-COLL '+key+'='+str(values[key])+'\n' for key in collections_evidence.SUMMARY_KEYS)
                +'SUMMARY attempts=1 allocs=1 frees=1 live=0 live_bytes=0 peak=3\nFREESTAT free_attempts=1 frees_ok=1 invalid=0\n').encode()

    def test_native_summary_distinguishes_typed_trap_and_balanced_actual_text_events(self):
        raw=self.summary();self.assertEqual(collections_evidence.native_summary(raw)[0],21)
        self.assertIsNone(collections_evidence.native_summary(raw.replace(b'MN-COLL-TRAP code=21\n',b''))[0])
        for old,new in [(b'MN-COLL live=0',b'MN-COLL live=1'),(b'FREE id=1\n',b''),
                        (b'SUMMARY attempts=1 allocs=1',b'SUMMARY attempts=1 allocs=2'),
                        (b'MN-COLL-TRAP code=21',b'MN-COLL-TRAP code=ListIndex'),
                        (b'MN-COLL bytes=0',b'MN-COLL bytes=0 extra=1')]:
            with self.subTest(old=old),self.assertRaises(m.AdmissionError):collections_evidence.native_summary(raw.replace(old,new))


class QuerySourceObservationTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def sample(self,family='source-query',case='neg-bool-plus',target='av'):
        import base64
        row=query_evidence.source_contracts(m.EvidenceScope(ROOT))[(family,case)]
        source=(ROOT/row['source_file']).read_bytes();av=target=='av';negative=row['oracle_kind']=='diagnostic'
        recipe='ac-av/core-run-v1' if av else 'ac/mir-native-scalar-v1'
        req=dict(recipe=recipe,nonce='a'*64,source_b64=base64.b64encode(source).decode(),entry='d_main',caps=json.loads(row['caps']),store_root='unit-store')
        if av:req['steps']=[dict(op='run')]
        else:
            req['workdir']='unit-work'
            if negative:req['diagnostics']='source-v1'
        req['request_hash']=hashlib.sha256(json.dumps(req,separators=(',',':')).encode()).hexdigest()
        protocol='avworker/1' if av else 'mnworker/1';response=dict(schema=protocol,nonce=req['nonce'],request_hash=req['request_hash'])
        if av:
            marks=['a','b'] if case=='trace-ab7' else []
            step=dict(op='run',outcome='diag' if negative else 'success',value=None if negative else int(row['oracle_value']),channel=None if negative else 'value',trap=None,
                      diag=dict(stage='frontend',code=row['oracle_value'],message='expected type mismatch') if negative else None,
                      counters=dict(frontend_reject=1,requests=1) if negative else {},spans=[],source_hash=scalar_evidence._fnv1a64(source),wall_ms=1,trace=marks)
            response['ok']=dict(recipe=recipe,steps=[step],worker_version='avworker/1.0',av_core_version='av.core-i32/1')
        else:
            ok=dict(recipe=recipe,worker_version='mnworker/1.0',mn_abi='msvc-x64/priv-i32-bool-v1',mn_target='x86_64-pc-windows-msvc',counters=dict(frontend_reject=1) if negative else dict.fromkeys(text_q4.NATIVE_COUNTERS,1),spans=dict(frontend=1) if negative else dict.fromkeys(text_q4.NATIVE_SPANS,0))
            response['ok']=ok
            if not negative:ok.update(manifest_fp='a'*16,exit_code=int(row['oracle_value']),stdout_hex='',stderr_hex=b'ab'.hex() if case=='trace-ab7' else '')
            else:
                ok.update(diagnostics_mode='source-v1',diagnostics_schema='mn.source-diagnostic/1')
                digest=hashlib.sha256(source).hexdigest();stage='verify' if family=='source-query' else 'bind';start=source.index(b'true')
                response['source_error']=dict(schema='mn.source-diagnostic/1',mode='source-v1',producer='ace-core.source',profile='scalar',attempted=['scalar'],surface_rejections=[],path='mn.at',source_sha256=digest,source_len=len(source),span_basis='utf8-byte',
                    causes=[dict(stage=stage,code=row['oracle_value'],message='expected type mismatch',span=dict(start=start,end=start+4))],primary=0,
                    operation=dict(kind='cold',step=None,base_sha256=None,candidate_sha256=digest,session_revision=None,edit=None))
                response['diagnostic']=dict(code='mn.unsupported.frontend',message=stage.capitalize()+'/'+row['oracle_value']+': expected type mismatch')
        wire=dict(protocol=protocol,recipe=recipe,exit_code=0,transport_error=None,stderr=b'',request=json.dumps(req).encode(),stdout=json.dumps(response).encode())
        return wire,dict(recipe=recipe),row

    def judge(self,wire,check,row):
        return scalar_q4.scalar_q4_observation(self.scope,wire,check,row,query_family=True)

    def test_instance_cap_and_query_mark_order_bind_both_actual_consumers(self):
        for target in ('av','native'):
            for family,case in [('source-query','trace-ab7'),('source-instances','choose42')]:
                self.assertEqual(self.judge(*self.sample(family,case,target))['judgement'],'PASS')
                wire,check,row=self.sample(family,case,target);request=json.loads(wire['request']);request.pop('request_hash');request['caps']=[]
                request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest();response=json.loads(wire['stdout']);response['request_hash']=request['request_hash'];wire.update(request=json.dumps(request).encode(),stdout=json.dumps(response).encode())
                with self.subTest(target=target,case=case),self.assertRaises(m.AdmissionError):self.judge(wire,check,row)

    def test_source_diagnostics_require_independent_code_and_raw_stage(self):
        for target in ('av','native'):
            for family,case in [('source-query','neg-bool-plus'),('source-instances','neg-hetero')]:
                self.assertEqual(self.judge(*self.sample(family,case,target))['judgement'],'PASS')
                wire,check,row=self.sample(family,case,target);response=json.loads(wire['stdout'])
                if target=='av':response['ok']['steps'][0]['diag']['code']='wrong-code'
                else:
                    cause=response['source_error']['causes'][0];cause['code']='wrong-code';response['diagnostic']['message']=cause['stage'].capitalize()+'/wrong-code: '+cause['message']
                wire['stdout']=json.dumps(response).encode();self.assertEqual(self.judge(wire,check,row)['judgement'],'FAIL')

    def test_native_snapshot_cannot_change_owner_operation_or_primary_operand_span(self):
        for change in ('hash','operation','primary-type','unknown','span'):
            wire,check,row=self.sample(target='native');response=json.loads(wire['stdout']);fact=response['source_error']
            if change=='hash':fact['source_sha256']='0'*64
            elif change=='operation':fact['operation']['base_sha256']='0'*64
            elif change=='primary-type':fact['primary']=False
            elif change=='unknown':fact['causes'][0]['echoed_pass']=True
            else:fact['causes'][0]['span']=dict(start=0,end=1)
            wire['stdout']=json.dumps(response).encode()
            with self.subTest(change=change):
                if change=='span':self.assertEqual(self.judge(wire,check,row)['judgement'],'FAIL')
                else:
                    with self.assertRaises(m.AdmissionError):self.judge(wire,check,row)


class OriginalDataDiagnosticTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp

    def test_rendered_av_message_cannot_supply_missing_diagnostic_code(self):
        import base64
        row=original_data.source_contracts(m.EvidenceScope(ROOT))['M09']
        source=(ROOT/row['source_file']).read_bytes();recipe='ac-av/data-run-v1'
        request=dict(recipe=recipe,nonce='a'*64,source_b64=base64.b64encode(source).decode(),entry='d_main',caps=[],store_root='unit-store',steps=[{'op':'run'}])
        request['request_hash']=hashlib.sha256(json.dumps(request,separators=(',',':')).encode()).hexdigest()
        step=dict(op='run',outcome='diag',value=None,trap=None,diag=dict(stage='frontend',code='',message='frontend: source.not-callable'),source_hash=scalar_evidence._fnv1a64(source),wall_ms=0)
        response=dict(schema='avworker/1',nonce=request['nonce'],request_hash=request['request_hash'],ok=dict(recipe=recipe,steps=[step],worker_version='avworker/1.0',av_core_version='av.data/1'))
        wire=dict(protocol='avworker/1',recipe=recipe,exit_code=0,transport_error=None,stderr=b'',request=json.dumps(request).encode())
        check=dict(check='av',recipe=recipe)
        def judge():return data_diagnostics.diagnostic_observation(self.scope,dict(wire,stdout=json.dumps(response).encode()),check,row)
        self.assertEqual(judge()['judgement'],'UNOBSERVED')
        self.assertEqual(first_slice.retain_parent_status(judge(),'FAIL')['judgement'],'FAIL')
        step['diag']['code']='wrong-code';self.assertEqual(judge()['judgement'],'FAIL')
        step['diag']['code']='source.not-callable';self.assertEqual(judge()['judgement'],'PASS')
        step['echoed_pass']=True
        with self.assertRaises(m.AdmissionError):judge()

    def test_t06_block_span_uses_live_statement_and_conversion_type_word(self):
        row=original_data.source_contracts(m.EvidenceScope(ROOT))['T06']
        source=(ROOT/row['source_file']).read_bytes()
        # Independently hand-counted UTF-8 positions in the original source:
        # block begins at the live let; .to ends at its type word, before ')'.
        expected=dict(start=19,end=62)
        self.assertEqual(data_diagnostics.diagnostic_span(source,'T06'),expected)
        self.assertEqual(source[expected['start']:expected['start']+4],b'let ')
        self.assertEqual(source[expected['end']-3:expected['end']],b'str')
        self.assertEqual(source[expected['end']:expected['end']+1],b')')
        prefix='// 雪\n'.encode()
        self.assertEqual(data_diagnostics.diagnostic_span(prefix+source,'T06'),
                         dict(start=19+len(prefix),end=62+len(prefix)))

    def test_t06_span_anchor_refuses_missing_or_ambiguous_source(self):
        row=original_data.source_contracts(m.EvidenceScope(ROOT))['T06']
        source=(ROOT/row['source_file']).read_bytes()
        for changed in (source+source,source.replace(b'return s.to(str)',b'return s.to(Str)')):
            with self.subTest(source=changed),self.assertRaises(m.AdmissionError):
                data_diagnostics.diagnostic_span(changed,'T06')

    def test_native_data_facts_bind_profile_operation_and_independent_span(self):
        for case in ('M09','T06'):
            row=original_data.source_contracts(m.EvidenceScope(ROOT))[case]
            source=(ROOT/row['source_file']).read_bytes();digest=hashlib.sha256(source).hexdigest()
            stage,code,profile=data_diagnostics.ORACLES[case]
            # PLAN-999 reception: Core side face between scalar and the rungs.
            attempted=['scalar','core','record']+(['text-record'] if profile=='text-record' else [])
            span=data_diagnostics.diagnostic_span(source,case) or dict(start=0,end=1)
            fact=dict(schema='mn.source-diagnostic/1',mode='source-v1',producer='ace-core.source',profile=profile,attempted=attempted,
                      surface_rejections=[dict(profile=p,codes=['source.unsupported']) for p in attempted[:-1] if p!='core'],path='mn.at',source_sha256=digest,source_len=len(source),span_basis='utf8-byte',
                      causes=[dict(stage=stage,code=code,message='rejected',span=span)],primary=0,
                      operation=dict(kind='cold',step=None,base_sha256=None,candidate_sha256=digest,session_revision=None,edit=None))
            response=dict(source_error=fact,diagnostic=dict(code='mn.unsupported.frontend',message=stage.capitalize()+'/'+code+': rejected'))
            def judge():return query_evidence.native_source_diagnostic(response,source,row,data_family=True)[1]
            self.assertEqual(judge(),'PASS')
            if case=='T06':
                fact['causes'][0]['span']['end']+=1
                self.assertEqual(judge(),'FAIL','closing parenthesis is outside the conversion span')
                fact['causes'][0]['span']['end']-=1
            fact['primary']=False
            with self.assertRaises(m.AdmissionError):judge()
            fact['primary']=0;fact['causes'][0]['span']['start']+=1
            self.assertEqual(judge(),'FAIL')
            fact['operation']['base_sha256']='0'*64
            with self.assertRaises(m.AdmissionError):judge()


class QueryRefusalCommandTests(unittest.TestCase):
    setUp = EvidenceAdmissionTests.setUp
    command = CertificationReaderPrimitiveTests.command

    def test_controlled_refusal_terminal_never_becomes_a_successful_command(self):
        record,request,producer=self.command();record['exit']=record['meter']['exit']=2
        (self.root/'unit.command.json').write_text(json.dumps(record))
        with self.assertRaises(m.AdmissionError):cert_evidence.admit_command(self.scope,'unit.command.json',request,producer)
        response,receipt=cert_evidence.admit_command(self.scope,'unit.command.json',request,producer,expected_exit=2)
        self.assertEqual(response,{'schema':'unit-only'});self.assertNotIn('judgement',receipt)
        for terminal in (False,True,1,7):
            with self.subTest(terminal=terminal),self.assertRaises(m.AdmissionError):
                cert_evidence.admit_command(self.scope,'unit.command.json',request,producer,expected_exit=terminal)

    def test_typed_frontdoor_channel_certifies_only_the_matching_oracle(self):
        # PLAN-1000 F-1000-02 (GAP-941-A2X-DIAG-01 closure): the structured
        # diagnostics carry the semantic authority; the rendered detail must
        # re-derive from the causes verbatim.
        source=b'fn main() int { return true + 1 }'
        contract=dict(family='source-query',source_sha256=hashlib.sha256(source).hexdigest(),
                      oracle_value='source.type-mismatch')
        request=dict(op='daily-emit',profile='scalar',source_path='X:/src/neg.at')
        def response(stage,code,*,detail=None,causes=1,span=(4,8)):
            snap=dict(schema='a2x.source-diagnostic/1',mode='typed-negative/1',producer='a2x-worker',
                      profile='scalar',path='X:/src/neg.at',
                      source_sha256=contract['source_sha256'],source_len=len(source),
                      causes=[dict(stage=stage if i==0 else 'bind',code=code if i==0 else 'other',
                                   message=f'expected i32 {i}',span=dict(start=span[0],end=span[1]))
                              for i in range(causes)],primary=0)
            rendered='frontend diagnostics: '+'; '.join(
                f"{c['code']}: {c['message']}" for c in snap['causes'])
            return dict(status='error',detail=detail if detail is not None else rendered,diagnostics=snap)
        # the matching oracle closes; extra causes keep the FIRST primary
        value=query_evidence.typed_frontdoor_observation(response('verify','source.type-mismatch'),request,contract,source)
        self.assertEqual(value['judgement'],'PASS');self.assertEqual(value['observed']['kind'],'typed-frontdoor-refusal')
        self.assertEqual(query_evidence.typed_frontdoor_observation(
            response('verify','source.type-mismatch',causes=3),request,contract,source)['judgement'],'PASS')
        # wrong stage or code FAILs (never UNOBSERVED: the typed facts disagree)
        self.assertEqual(query_evidence.typed_frontdoor_observation(
            response('bind','source.type-mismatch'),request,contract,source)['judgement'],'FAIL')
        self.assertEqual(query_evidence.typed_frontdoor_observation(
            response('verify','type-mismatch'),request,contract,source)['judgement'],'FAIL')
        # an instances-family oracle expects the bind stage
        contract2=dict(family='source-instances',source_sha256=contract['source_sha256'],oracle_value='type-mismatch')
        self.assertEqual(query_evidence.typed_frontdoor_observation(
            response('bind','type-mismatch'),request,contract2,source)['judgement'],'PASS')
        # free-text divergence between detail and causes refuses
        for bad in (response('verify','source.type-mismatch',detail='frontend diagnostics: something else: x'),
                    response('verify','source.type-mismatch',detail='a2x.compile: source.type-mismatch: expected i32 0')):
            with self.assertRaises(m.AdmissionError):
                query_evidence.typed_frontdoor_observation(bad,request,contract,source)
        # malformed typed shapes refuse: bad span bounds, wrong source hash,
        # non-zero primary, foreign schema, non-UTF-8 span boundary
        bad_span=response('verify','source.type-mismatch',span=(len(source)+1,len(source)+2))
        with self.assertRaises(m.AdmissionError):
            query_evidence.typed_frontdoor_observation(bad_span,request,contract,source)
        mutated=response('verify','source.type-mismatch')
        mutated['diagnostics']['source_sha256']='0'*64
        with self.assertRaises(m.AdmissionError):
            query_evidence.typed_frontdoor_observation(mutated,request,contract,source)
        mutated=response('verify','source.type-mismatch');mutated['diagnostics']['primary']=1
        with self.assertRaises(m.AdmissionError):
            query_evidence.typed_frontdoor_observation(mutated,request,contract,source)
        mutated=response('verify','source.type-mismatch');mutated['diagnostics']['schema']='a2x.diagnostic/2'
        with self.assertRaises(m.AdmissionError):
            query_evidence.typed_frontdoor_observation(mutated,request,contract,source)

    def test_rendered_frontdoor_errors_never_supply_typed_semantic_authority(self):
        for detail in ('frontend diagnostics: source.type-mismatch: expected i32, found bool',
                       'a2x.compile: type-mismatch: expected bool, found i32','both wrong'):
            value=query_evidence.untyped_frontdoor_observation(dict(status='error',detail=detail))
            self.assertEqual(value['judgement'],'UNOBSERVED')
        for response in ({'status':'ok','detail':'looks like a diagnostic'}, {'status':'error','detail':''},
                         {'status':'error','detail':False},{'status':'error','detail':'code','echoed_pass':True}):
            with self.subTest(response=response),self.assertRaises(m.AdmissionError):
                query_evidence.untyped_frontdoor_observation(response)


class OriginalDataNegativeCertificationTests(unittest.TestCase):
    """PLAN-1000 16 diagnostic legs: negative certification coverage."""

    def test_diagnostic_slots_join_coverage_and_missing_stay_unobserved(self):
        import ace_m1_original_data_sources as ods
        contracts={'M01':dict(family='original-method-881',case='M01')}
        diagnostics=[dict(family='original-method-881',case='M09'),
                     dict(family='original-record-text',case='R09')]
        observations={'q4-source/original-method-881/M01/ACE-A2R-HIR':dict(judgement='PASS'),
                      'q4-source/original-method-881/M01/ACE-A2C-HIR':dict(judgement='PASS')}
        complete,missing=ods.certification_coverage(contracts,observations,diagnostics)
        expected_diag={'q4-source/original-method-881/M09/ACE-A2R-HIR','q4-source/original-method-881/M09/ACE-A2C-HIR',
                       'q4-source/original-record-text/R09/ACE-A2R-HIR','q4-source/original-record-text/R09/ACE-A2C-HIR'}
        self.assertEqual(set(complete),set(observations)|expected_diag)
        self.assertEqual(set(missing),expected_diag)
        for key in expected_diag:
            self.assertEqual(complete[key]['judgement'],'UNOBSERVED')
        # supplied negative rows must carry completed verdicts
        with self.assertRaises(m.AdmissionError):
            ods.certification_coverage(contracts,{**observations,
                'q4-source/original-method-881/M09/ACE-A2R-HIR':dict(judgement='UNOBSERVED')},diagnostics)


class LegacyDataJoinTests(unittest.TestCase):
    """Synthetic admitted facts test the join; they are not execution evidence."""

    @classmethod
    def setUpClass(cls):
        import ace_m1_legacy_data as data
        cls.data = data
        cls.required, cls.rules = data.contracts(m.EvidenceScope(ROOT))

    def sample(self, name='R01', target='av'):
        rule = self.rules[name]
        diagnostic = name in self.data.DIAGNOSTICS
        expected = dict(kind=rule['kind'], value=rule['value'])
        if diagnostic:
            _, stage, code = self.data.DIAGNOSTICS[name]
            expected = dict(kind='diagnostic', stage=stage, code=code)
            observed = dict(kind='diagnostic', stage='frontend' if target == 'av' else stage, code=code)
        elif name == 'T08':
            observed = dict(kind='trap', code='av.trap.str-index') if target == 'av' else dict(kind='trap', runtime_code=-10, exit=70)
        else:
            observed = dict(kind='value', value=int(rule['old_value']))
        result = dict(schema='ace.m1.original-data-diagnostic/1' if diagnostic else 'ace.m1.text-source-observation/1',
                      source_sha256=rule['source_sha256'], request_sha256='1'*64, response_sha256='2'*64,
                      expected=expected, observed=observed, judgement='PASS', parent_status='PASS')
        if diagnostic: result['diagnostic_authority'] = 'OBSERVED'
        return result

    def judge(self, source, name='R01', target='av'):
        return self.data.observation(name, target, self.rules[name], source)['judgement']

    def test_exact_corpus_names_targets_and_unchanged_original_denominator(self):
        self.assertEqual(len(self.required), 313)
        self.assertEqual(len(self.rules), 14)
        self.assertEqual(self.rules['R01']['old_value'], '19')
        self.assertIn('scenario:R01:av', self.required)
        for name in ('M01', 'R07', 'R08', 'T03', 'T05', 'I01'): self.assertNotIn(name, self.rules)
        for name in self.rules:
            for target in ('ac', 'av'):
                self.assertIn('corpus:'+name+':'+target, self.required)
                self.assertEqual(self.judge(self.sample(name, target), name, target), 'PASS')
        with self.assertRaises(m.AdmissionError): self.data.observation('M01', 'av', self.rules['R01'], self.sample())
        with self.assertRaises(m.AdmissionError): self.data.observation('R01', 'avm', self.rules['R01'], self.sample())

    def test_pass_echo_cannot_override_wrong_value_or_same_value_other_source(self):
        source = self.sample(); source['observed']['value'] = 20
        self.assertEqual(self.judge(source), 'FAIL')
        source['observed']['value'] = True
        self.assertEqual(self.judge(source), 'FAIL')
        for field, value in (('source_sha256', self.rules['R02']['source_sha256']),
                             ('schema', 'ace.m1.original-data-diagnostic/1'), ('request_sha256', 'bad')):
            source = self.sample(); source[field] = value
            with self.subTest(field=field), self.assertRaises(m.AdmissionError): self.judge(source)
        source = self.sample(); source['expected']['value'] = '20'
        with self.assertRaises(m.AdmissionError): self.judge(source)

    def test_parent_and_source_failure_or_missing_authority_are_preserved(self):
        for name in ('R01', 'R09', 'T08'):
            for field in ('parent_status', 'judgement'):
                source = self.sample(name); source[field] = 'FAIL'
                self.assertEqual(self.judge(source, name), 'FAIL')
            source = self.sample(name); source['judgement'] = 'UNOBSERVED'
            self.assertEqual(self.judge(source, name), 'UNOBSERVED')
        source = self.sample('R09'); source['diagnostic_authority'] = 'UNOBSERVED'
        self.assertEqual(self.judge(source, 'R09'), 'UNOBSERVED')

    def test_diagnostic_code_and_target_stage_are_facts_not_message_text(self):
        for name in self.data.DIAGNOSTICS:
            for target in ('ac', 'av'):
                for field in ('code', 'stage'):
                    source = self.sample(name, target)
                    source['observed']['message'] = str(source['expected'])
                    source['observed'][field] = 'wrong'
                    self.assertEqual(self.judge(source, name, target), 'FAIL')
        source = self.sample('R09', 'av')
        self.assertEqual(self.judge(source, 'R09', 'ac'), 'FAIL')

    def test_t08_native_requires_actual_named_runtime_failure(self):
        for fact in (dict(kind='trap', exit=70), dict(kind='trap', exit=70, runtime_code=-9)):
            source = self.sample('T08', 'ac'); source['observed'] = fact
            self.assertEqual(self.judge(source, 'T08', 'ac'), 'UNOBSERVED')
        source = self.sample('T08', 'ac'); source['observed'] = dict(kind='value', value=70)
        self.assertEqual(self.judge(source, 'T08', 'ac'), 'FAIL')
        source = self.sample('T08'); source['observed']['code'] = 'av.trap.div-by-zero'
        self.assertEqual(self.judge(source, 'T08'), 'FAIL')
        source = self.sample('R01'); source['observed']['value'] = 70
        self.assertEqual(self.judge(source), 'FAIL')

    def test_only_exact_admitted_family_slots_are_joined_and_pending_is_separate(self):
        from unittest.mock import patch
        reports = [dict(schema='ace.m1.original-data-source-observations/1', observations={}),
                   dict(schema='ace.m1.original-data-diagnostics/1', observations={})]
        for name in self.rules:
            report = reports[int(name in self.data.DIAGNOSTICS)]
            for target, label in (('ac', 'AC-native'), ('av', 'AV')):
                report['observations']['q4-source/original-record-text/'+name+'/'+label] = self.sample(name, target)
        reports[0]['observations']['q4-source/original-method-881/M01/AV'] = self.sample()
        with patch.object(self.data, 'contracts', return_value=(self.required, self.rules)):
            joined = self.data.admit(None, *reports)
            self.assertEqual(len(joined['observations']), 28)
            self.assertEqual(set(joined['observations']), {'corpus:'+n+':'+t for n in self.rules for t in ('ac','av')})
            source = reports[1]['observations']['q4-source/original-record-text/R09/AV']
            source['diagnostic_authority'] = 'UNOBSERVED'
            joined = self.data.admit(None, *reports)
            self.assertEqual(len(joined['observations']), 27)
            self.assertEqual(set(joined['pending_observations']), {'corpus:R09:av'})
            source['parent_status'] = 'FAIL'
            self.assertEqual(self.data.admit(None, *reports)['observations']['corpus:R09:av']['judgement'], 'FAIL')
            del reports[0]['observations']['q4-source/original-record-text/R01/AV']
            with self.assertRaises(m.AdmissionError): self.data.admit(None, *reports)


class CtSourceMechanismTests(unittest.TestCase):
    """Adversarial synthetic packets test readers, never count as execution."""
    setUp = EvidenceAdmissionTests.setUp

    def packet(self):
        import ace_m1_ct_mechanisms as ct
        self.ct = ct
        first = dict(key_hash='a'*64, attempt=1, outcome_hex='6f6b', steps=6, wall_ms=0, from_cache=False)
        hit = dict(first, attempt=0, outcome_hex='686974', from_cache=True)
        second = dict(first, key_hash='b'*64, attempt=2, steps=8)
        def snapshot(counts, log): return dict(zip(ct.COUNTERS, counts)) | dict(exec_log=copy.deepcopy(log))
        source = ROOT/'crates/auto-lang/test/cases/ace-m1/source-cte-936/src'
        self.sources = [(source/(name+'.at')).read_bytes() for name in ('sum42','sum43')]
        return dict(schema='ace.m1.ct-source-mechanisms/1', sink='fresh-memory-envelope',
                    source42_hex=self.sources[0].hex(), source43_hex=self.sources[1].hex(),
                    cold_control_code_hex='cte.infra/av-forbidden'.encode().hex(),
                    edit_control_code_hex='cte.infra/av-forbidden'.encode().hex(), owner_values=[42,42,43],
                    cold_control=snapshot((0,0,0,1), []), cold=snapshot((1,0,1,0), [first]),
                    hit=snapshot((1,1,1,0), [first,hit]), edit_control=snapshot((1,1,1,1), [first,hit]),
                    edited=snapshot((2,1,2,1), [first,hit,second]))

    def test_counter_echoes_wrong_values_and_missing_logs_never_prove_execution(self):
        packet = self.packet(); expected = dict.fromkeys(self.ct.ARCH, 'PASS')
        self.assertEqual(self.ct.validate_payload(packet, *self.sources), expected)
        for field in self.ct.COUNTERS:
            bad = copy.deepcopy(packet); bad['cold'][field] += 1
            self.assertEqual(set(self.ct.validate_payload(bad, *self.sources).values()), {'FAIL'})
        for stage in ('cold','hit','edited'):
            bad = copy.deepcopy(packet); bad[stage]['exec_log'] = []
            self.assertEqual(self.ct.validate_payload(bad, *self.sources)[self.ct.ARCH[1]], 'FAIL')
        bad = copy.deepcopy(packet); bad['owner_values'] = [41,41,43]
        self.assertEqual(set(self.ct.validate_payload(bad, *self.sources).values()), {'FAIL'})

    def test_hit_preserves_frozen_steps_but_never_counts_as_new_execution(self):
        packet = self.packet()
        for field, value in (('from_cache',False),('attempt',1),('key_hash','c'*64),('steps',0),('wall_ms',1)):
            bad = copy.deepcopy(packet); bad['hit']['exec_log'][1][field] = value
            self.assertEqual(set(self.ct.validate_payload(bad, *self.sources).values()), {'FAIL'})
        for field, value in (('from_cache',True),('attempt',1),('key_hash','a'*64),('steps',0)):
            bad = copy.deepcopy(packet); bad['edited']['exec_log'][2][field] = value
            self.assertEqual(self.ct.validate_payload(bad, *self.sources), {self.ct.ARCH[0]:'PASS',self.ct.ARCH[1]:'FAIL'})

    def test_original_source_types_schema_sink_and_forbidden_controls_are_independent(self):
        packet = self.packet()
        for mutate in (lambda p:p.update(source43_hex=p['source42_hex']), lambda p:p.update(sink='legacy-deck'),
                       lambda p:p['cold'].update(ct_execute=True), lambda p:p.update(owner_values=[True,42,43]),
                       lambda p:p.update(passed=True), lambda p:p.pop('edited')):
            bad = copy.deepcopy(packet); mutate(bad)
            with self.assertRaises(m.AdmissionError): self.ct.validate_payload(bad, *self.sources)
        for key in ('cold_control_code_hex','edit_control_code_hex'):
            bad = copy.deepcopy(packet); bad[key] = 'wrong'
            self.assertEqual(self.ct.validate_payload(bad, *self.sources)[self.ct.ARCH[1]], 'FAIL')

    def native_sample(self):
        packet = self.packet()
        repo, evidence = self.root/'repo', self.root/'evidence'
        repo.mkdir(); evidence.mkdir(); (evidence/'gate/0').mkdir(parents=True)
        def put(root, name, value):
            path = root/name; path.parent.mkdir(parents=True,exist_ok=True)
            path.write_bytes(value if isinstance(value,bytes) else json.dumps(value).encode())
        self.put_native = lambda name,value: put(evidence,name,value)
        for name, data in (('scripts/fixed-tests.py',b'synthetic parser import is mocked'),
                           ('scripts/test-groups.toml',(ROOT/'scripts/test-groups.toml').read_bytes()),
                           (self.ct.SOURCE_DIR+'sum42.at',self.sources[0]),(self.ct.SOURCE_DIR+'sum43.at',self.sources[1])):
            put(repo,name,data)
        binary = repo/'experimental/ace/target/debug/deps/architecture_941-0123456789abcdef.exe'
        put(repo,binary.relative_to(repo),b'synthetic-unit-control-not-an-executable')
        pin = dict(path=str(binary),sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
        pins = {p.relative_to(repo).as_posix():hashlib.sha256(p.read_bytes()).hexdigest() for p in repo.rglob('*') if p.is_file() and p != binary}
        before = dict(commit='1'*40,source_pins=pins)
        for name,value in [('source-pins-before.json',before),('source-pins-after.json',before),
                           ('binary-before.json',pin),('binary-after.json',pin),('observations.json',packet)]: self.put_native(name,value)
        listing = {'rust-suites':{self.ct.BINARY:{'package-name':'ace-source-cte','binary-name':'architecture_941',
                   'binary-path':str(binary),'testcases':{self.ct.TEST:dict(kind='test',ignored=False,**{'filter-match':{'status':'matches'}})}}}}
        import tomllib
        group = next(g for g in tomllib.loads((repo/'scripts/test-groups.toml').read_text())['group'] if g['name']=='ace-comptime')
        expression = '(((all())) and not test(=tests::aavm2_t3::test_aavm2_t3_tower_milestone)) and test('+self.ct.TEST+')'
        command = dict(argv=['cargo','nextest','run',*group['args'],'--ignore-default-filter','-E',expression,
                       '--target-dir',str(repo/'experimental/ace/target'),'--no-fail-fast','--no-tests','fail',
                       '--message-format','libtest-json-plus','--message-format-version','0.1'],
                       cwd=str(repo/'experimental/ace'),exit=0,meter=dict(exit=0,error=None),stdout='run.stdout',stderr='run.stderr')
        terminal = dict(type='test',event='ok',name=self.ct.EVENT,exec_time=0.01)
        result = dict(selected=[dict(binary=self.ct.BINARY,test=self.ct.TEST,ignored=False)],excluded=[],executed=1,
                      results={self.ct.EVENT:terminal},not_run=[],errors=[],outcome='PASS',execution=command)
        gate = dict(selected_groups=['ace-comptime'],config_sha256=pins['scripts/test-groups.toml'],
                    source_state=hashlib.sha256(b'1'*40+b'[]').hexdigest(),results=[result],gate_passed=True)
        raw = (json.dumps(dict(type='test',event='started',name=self.ct.EVENT))+'\n'+json.dumps(terminal)+'\n').encode()
        for name,value in [('gate/results.json',gate),('gate/0/discovery.stdout',listing),('gate/0/run.command.json',command),
                           ('gate/0/run.stdout',raw),('gate/0/run.stderr',b'')]:self.put_native(name,value)
        # Use the real parent's pure parsers without copying or running a runner.
        import importlib.util, importlib.abc
        spec = importlib.util.spec_from_file_location('ct_parent_test',ROOT/'scripts/fixed-tests.py')
        parent = importlib.util.module_from_spec(spec); spec.loader.exec_module(parent)
        class Loader(importlib.abc.Loader):
            def create_module(self,spec):return None
            def exec_module(self,module):
                module.enumerate_rust=parent.enumerate_rust;module.terminal_results=parent.terminal_results
        self.native_loader = importlib.util.spec_from_loader('ct_fixed_parent',Loader())
        self.native_args = (m.EvidenceScope(repo),m.EvidenceScope(evidence),pins,self.ct.validate_payload)
        return gate, listing, raw

    def native_judge(self):
        with patch.object(self.ct.importlib.util,'spec_from_file_location',return_value=self.native_loader):
            return self.ct.native_facts(*self.native_args)

    def test_raw_terminal_and_actual_binary_identity_are_required_even_for_parent_pass(self):
        gate, listing, raw = self.native_sample()
        self.assertEqual(set(self.native_judge()['observations'].values()), {'PASS'})
        self.put_native('gate/0/run.stdout',b'')
        with self.assertRaises(m.AdmissionError):self.native_judge()
        self.put_native('gate/0/run.stdout',raw)
        listing['rust-suites'][self.ct.BINARY]['binary-path'] += '.wrong'
        self.put_native('gate/0/discovery.stdout',listing)
        with self.assertRaises(m.AdmissionError):self.native_judge()

    def test_failed_real_native_terminal_cannot_be_overridden_by_good_ct_payload(self):
        gate, listing, raw = self.native_sample()
        result = gate['results'][0]; result['results'][self.ct.EVENT]['event']='failed'
        result['errors']=['failed or unexpectedly skipped test'];result['outcome']='FAIL'
        result['execution']['exit']=100;result['execution']['meter']['exit']=100;gate['gate_passed']=False
        self.put_native('gate/results.json',gate);self.put_native('gate/0/run.command.json',result['execution'])
        self.put_native('gate/0/run.stdout',raw.replace(b'"ok"',b'"failed"'))
        self.assertEqual(set(self.native_judge()['observations'].values()), {'FAIL'})

    def test_same_named_current_native_binary_with_different_bytes_is_stale(self):
        self.native_sample()
        self.assertEqual(set(self.native_judge()['observations'].values()), {'PASS'})
        binary = self.root/'repo/experimental/ace/target/debug/deps/architecture_941-0123456789abcdef.exe'
        binary.write_bytes(b'different synthetic producer bytes')
        with self.assertRaises(m.AdmissionError): self.native_judge()

    def test_native_false_current_inputs_unknown_selection_and_duplicate_raw_keys_are_refused(self):
        gate, listing, raw = self.native_sample()
        for key,value in (('selected_groups',['other']),('source_state','0'*64),('gate_passed',False)):
            bad=copy.deepcopy(gate);bad[key]=value;self.put_native('gate/results.json',bad)
            with self.assertRaises(m.AdmissionError):self.native_judge()
        self.put_native('gate/results.json',gate)
        self.put_native('gate/0/run.stdout',b'{"type":"test","type":"suite"}\n')
        with self.assertRaises(m.AdmissionError):self.native_judge()
        self.put_native('gate/0/run.stdout',raw)
        self.put_native('observations.json',b' '*(16*1024+1))
        with self.assertRaises(m.AdmissionError):self.native_judge()


class CtChildVersionTests(unittest.TestCase):
    """Synthetic version/counter faults are controls, not execution receipts."""
    setUp = EvidenceAdmissionTests.setUp

    def sample(self):
        import ace_m1_ct_child as child
        self.child=child;self.sources=[(ROOT/p).read_bytes() for p in (child.TRUE,child.FALSE)]
        keys=[];events=[]
        for index,(callee,logical,signature,body,args) in enumerate((
            ('flag','a','1','3',''),('pick_i32','b','2','4','i32:40,i32:42'),
            ('flag','a','1','5',''),('pick_i32','b','2','6','i32:40,i32:42'))):
            digest=hex(index+10)[2:]*64
            keys.append(dict(key_hash=digest,callee_hex=callee.encode().hex(),logical=logical*64,
                             signature=signature*64,body=body*64,args_hex=args.encode().hex()))
            events.append(dict(key_hash=digest,attempt=index+1,outcome_hex=b'ok'.hex(),steps=index+6,wall_ms=0,from_cache=False))
        hits=[dict(e,attempt=0,outcome_hex=b'hit'.hex(),from_cache=True) for e in events[2:]]
        def state(counts,log):return dict(zip(('ct_execute','ct_hits','ct_freeze_publishes','ct_rejections'),counts))|dict(exec_log=copy.deepcopy(log))
        instance=dict(logical='7'*64,key='8'*64,template_hex=b'pick'.hex(),concrete_hex=b'pick_i32'.hex(),signature='9'*64,body='a'*64,demands=1)
        return dict(schema='ace.m1.ct-child-versions/1',source_true_hex=self.sources[0].hex(),source_false_hex=self.sources[1].hex(),
                    owner_values=[40,42,42],instance_before=instance,instance_after=copy.deepcopy(instance),
                    cold=state((2,0,2,0),events[:2]),edited=state((4,0,4,0),events),hit=state((4,2,4,0),events+hits),published_keys=keys)

    def judge(self,value):return self.child.validate_payload(value,*self.sources)[self.child.ARCH]

    def test_same_template_key_and_changed_effective_checked_keys_are_both_required(self):
        value=self.sample();self.assertEqual(self.judge(value),'PASS')
        for field in ('logical','key','signature','body'):
            bad=copy.deepcopy(value);bad['instance_after'][field]='c'*64
            self.assertEqual(self.judge(bad),'FAIL')
        for index,field,new in ((3,'body','4'*64),(2,'body','3'*64),(3,'logical','c'*64),(3,'signature','c'*64),
                                (3,'args_hex',b'i32:42,i32:40'.hex()),(3,'callee_hex',b'flag'.hex())):
            bad=copy.deepcopy(value);bad['published_keys'][index][field]=new
            self.assertEqual(self.judge(bad),'FAIL')

    def test_same_wrong_values_and_counter_only_claims_never_prove_child_execution(self):
        value=self.sample();value['owner_values']=[42,42,42];self.assertEqual(self.judge(value),'FAIL')
        for stage in ('cold','edited','hit'):
            value=self.sample();value[stage]['exec_log']=[];self.assertEqual(self.judge(value),'FAIL')
        value=self.sample();value['edited']['ct_execute']=2;self.assertEqual(self.judge(value),'FAIL')
        value=self.sample();value['published_keys'][3]['key_hash']='c'*64;self.assertEqual(self.judge(value),'FAIL')

    def test_frozen_hit_does_not_become_new_work_or_lose_its_boundary_provenance(self):
        for field,new in (('attempt',5),('from_cache',False),('steps',0),('key_hash','f'*64),('wall_ms',1)):
            value=self.sample();value['hit']['exec_log'][4][field]=new
            self.assertEqual(self.judge(value),'FAIL')
        value=self.sample();value['hit']['ct_execute']=6;self.assertEqual(self.judge(value),'FAIL')
        value=self.sample();value['edited']['exec_log'][0]['attempt']=2;self.assertEqual(self.judge(value),'FAIL')

    def test_foreign_sources_bool_counts_and_unknown_parent_pass_fields_are_refused(self):
        for mutate in (lambda v:v.update(source_false_hex=v['source_true_hex']),lambda v:v.update(passed=True),
                       lambda v:v['cold'].update(ct_execute=True),lambda v:v.update(owner_values=[True,42,42]),
                       lambda v:v['published_keys'][0].update(logical='not-a-key')):
            value=self.sample();mutate(value)
            with self.assertRaises(m.AdmissionError):self.judge(value)

    def test_native_child_producer_cannot_be_replaced_with_parent_or_foreign_test(self):
        import ace_m1_ct_mechanisms as parent
        with self.assertRaises(m.AdmissionError):
            parent.native_facts(m.EvidenceScope(ROOT),self.scope,{},lambda *_:None,
                                binary_name=parent.CHILD_BINARY,test_name=parent.TEST)


class CtMetaArgumentTests(unittest.TestCase):
    """Controlled bad facts distinguish real meta/AV work; never milestone evidence."""
    def sample(self):
        import ace_m1_ct_meta as meta
        self.meta=meta;self.sources=[(ROOT/p).read_bytes() for p in meta.SOURCES]
        def state(counts,probes,log):
            return dict(zip(meta.COUNTERS,counts))|dict(probes=probes,exec_log=copy.deepcopy(log))
        def axis(values,callee,args):
            keys=[dict(key_hash=h*64,callee_hex=callee.encode().hex(),logical='a'*64,signature='b'*64,body='c'*64,args_hex=arg.encode().hex()) for h,arg in zip(('1','2'),args)]
            cold=dict(key_hash=keys[0]['key_hash'],attempt=1,outcome_hex=b'ok'.hex(),steps=7,wall_ms=0,from_cache=False)
            hit=dict(cold,attempt=0,outcome_hex=b'hit'.hex(),from_cache=True)
            edit=dict(cold,key_hash=keys[1]['key_hash'],attempt=2)
            return dict(values=values,meta_queries=[1,1,1],control_code_hex=b'cte.infra/av-forbidden'.hex(),
                cold=state((1,0,1,0),1,[cold]),hit=state((1,1,1,0),2,[cold,hit]),
                control=state((1,1,1,1),3,[cold,hit]),edited=state((2,1,2,1),4,[cold,hit,edit]),published_keys=keys)
        closed=dict(stage='bind',code_hex=b'source.cte.bind'.hex(),state=state((0,0,0,0),0,[]))
        return dict(schema='ace.m1.ct-meta-arguments/1',sources_hex=[s.hex() for s in self.sources],
            field=axis([42,42,43],'sum',['i32:2,i32:40','i32:3,i32:40']),
            has=axis([42,42,40],'pick',['bool:false','bool:true']),field_closed=copy.deepcopy(closed),has_closed=closed,
            local=dict(value=2,meta_queries=1,state=state((0,0,0,0),0,[])))

    def judge(self,value):return self.meta.validate_payload(value,self.sources)[self.meta.ARCH]

    def test_legitimate_meta_arguments_and_absence_are_accepted(self):
        self.assertEqual(self.judge(self.sample()),'PASS')

    def test_host_fold_or_meta_only_counted_as_av_is_red(self):
        for change in ('folded','local-execute','local-probe','no-meta'):
            v=self.sample()
            if change=='folded':v['field']['cold']['ct_execute']=0
            if change=='local-execute':v['local']['state']['ct_execute']=1
            if change=='local-probe':v['local']['state']['probes']=1
            if change=='no-meta':v['has']['meta_queries'][2]=0
            with self.subTest(change=change):self.assertEqual(self.judge(v),'FAIL')

    def test_missing_negative_dependency_or_wrong_av_argument_is_red(self):
        for change in ('old-arg','old-result','same-key','unrelated-body'):
            v=self.sample()
            if change=='old-arg':v['has']['published_keys'][1]['args_hex']=b'bool:false'.hex()
            if change=='old-result':v['has']['values'][2]=42
            if change=='same-key':v['field']['published_keys'][1]['key_hash']=v['field']['published_keys'][0]['key_hash']
            if change=='unrelated-body':v['field']['published_keys'][1]['body']='d'*64
            with self.subTest(change=change):self.assertEqual(self.judge(v),'FAIL')

    def test_no_cap_must_refuse_before_probe_hit_or_publish(self):
        for name in ('field_closed','has_closed'):
            for field in ('probes','ct_execute','ct_hits','ct_freeze_publishes','ct_rejections'):
                v=self.sample();v[name]['state'][field]=1
                with self.subTest(name=name,field=field):self.assertEqual(self.judge(v),'FAIL')
        v=self.sample();v['field_closed']['code_hex']=b'cte.infra/av-forbidden'.hex();self.assertEqual(self.judge(v),'FAIL')

    def test_hit_cannot_hide_an_execution_or_rewrite_frozen_work(self):
        for field,value in [('attempt',1),('from_cache',False),('steps',0),('key_hash','f'*64),('wall_ms',4)]:
            v=self.sample();v['field']['hit']['exec_log'][1][field]=value
            with self.subTest(field=field):self.assertEqual(self.judge(v),'FAIL')

    def test_wrong_source_unknown_shape_and_boolean_counters_are_rejected(self):
        for change in ('source','extra','bool-count','missing-key','code-type','args-type','stage-type','i32-overflow','invalid-utf8'):
            v=self.sample()
            if change=='source':v['sources_hex'][0]=b'wrong-source'.hex()
            if change=='extra':v['local']['claimed_pass']=True
            if change=='bool-count':v['field']['cold']['ct_execute']=True
            if change=='missing-key':v['has']['published_keys'].pop()
            if change=='code-type':v['field']['control_code_hex']=True
            if change=='args-type':v['has']['published_keys'][0]['args_hex']=False
            if change=='stage-type':v['field_closed']['stage']=1
            if change=='i32-overflow':v['field']['values'][0]=1<<40
            if change=='invalid-utf8':v['has_closed']['code_hex']='ff'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(v)


class CtDiskRestoreTests(unittest.TestCase):
    """Controlled restore facts check rejection; these are not actual execution evidence."""
    def sample(self):
        import ace_m1_ct_restore as restore
        self.restore=restore;self.sources=[(ROOT/p).read_bytes() for p in restore.SOURCES]
        def event(key,attempt=1,outcome='ok'):
            return dict(key_hash=key*64,attempt=attempt,outcome_hex=outcome.encode().hex(),steps=0 if outcome.startswith('cte.trap/') else 6,wall_ms=0,from_cache=outcome=='hit')
        def state(counts,reads,log):
            return dict(zip(restore.COUNTERS,counts))|dict(read_probes=reads,read_bytes=reads*100,read_decodes=reads,exec_log=copy.deepcopy(log))
        good,args,bad,repair,second=(event(k) for k in '12345')
        bad=event('3',outcome='cte.trap/av.trap.div-by-zero');repair['attempt']=2
        hit=event('1',0,'hit')
        return dict(schema='ace.m1.ct-disk-restore/1',sink='ManagedEnvelopeSink/S936',owner_values=[42,42,42,42,42,42,43,44,43,44,42],
            sources_hex=[s.hex() for s in self.sources],store_root_hex=b'D:/autostack/plan-artifacts/plan-941/ct-restore-store-1-2'.hex(),
            trap_code_hex=b'cte.trap/av.trap.div-by-zero'.hex(),missing_code_hex=b'cte.infra/av-forbidden'.hex(),
            cold=state((1,0,1,0),0,[good]),first_edit=state((1,0,1,0),0,[args]),
            restored=state((0,2,0,0),2,[hit,event('2',0,'hit')]),failed=state((1,0,0,1),0,[bad]),
            preserved=state((1,1,0,1),1,[bad,hit]),repaired=state((2,1,1,1),1,[bad,hit,repair]),
            second_edit=state((1,0,1,0),0,[second]),repair_restore=state((0,2,0,0),2,[event('4',0,'hit'),event('5',0,'hit')]),
            missing=state((0,0,0,1),0,[]),fallback=state((1,0,1,1),0,[good]))

    def judge(self,value):return self.restore.validate_payload(value,self.sources)

    def test_legitimate_disk_ct_slice_is_accepted_without_architecture_pass(self):
        v=self.sample();self.assertEqual(self.judge(v),dict.fromkeys(self.restore.SLICE,'PASS'))
        self.assertFalse(any(k.startswith('ARCH-') for k in self.judge(v)))

    def test_prewarm_same_key_or_executed_restore_cannot_pass(self):
        for change in ('prewarm','same-key','executed','no-read','no-decode','rewritten-work'):
            v=self.sample()
            if change=='prewarm':v['first_edit']['ct_hits']=1
            if change=='same-key':v['first_edit']['exec_log'][0]['key_hash']=v['cold']['exec_log'][0]['key_hash']
            if change=='executed':v['restored']['ct_execute']=1
            if change=='no-read':v['restored']['read_bytes']=0
            if change=='no-decode':v['restored']['read_decodes']=0
            if change=='rewritten-work':v['restored']['exec_log'][0]['wall_ms']=1
            with self.subTest(change=change):self.assertEqual(self.judge(v)[self.restore.SLICE[0]],'FAIL')

    def test_failed_publication_stale_repair_or_second_edit_is_red(self):
        for change in ('publish','lost-old','repair-value','second-prewarm','second-value','bad-key','repair-decode'):
            v=self.sample()
            if change=='publish':v['failed']['ct_freeze_publishes']=1
            if change=='lost-old':v['preserved']['ct_hits']=0
            if change=='repair-value':v['owner_values'][6]=42
            if change=='second-prewarm':v['second_edit']['ct_hits']=1
            if change=='second-value':v['owner_values'][7]=43
            if change=='bad-key':v['repair_restore']['exec_log'][0]['key_hash']=v['failed']['exec_log'][0]['key_hash']
            if change=='repair-decode':v['repair_restore']['read_decodes']=0
            with self.subTest(change=change):self.assertEqual(self.judge(v)[self.restore.SLICE[1]],'FAIL')

    def test_missing_page_cannot_masquerade_as_restore(self):
        for change in ('hit','not-executed','published-failure','wrong-code'):
            v=self.sample()
            if change=='hit':v['missing']['ct_hits']=1
            if change=='not-executed':v['fallback']['ct_execute']=0
            if change=='published-failure':v['missing']['ct_freeze_publishes']=1
            if change=='wrong-code':v['missing_code_hex']=b'cte.trap/av.trap.div-by-zero'.hex()
            with self.subTest(change=change):self.assertEqual(self.judge(v)[self.restore.SLICE[2]],'FAIL')

    def test_sources_schema_counter_types_and_store_escape_are_rejected(self):
        for change in ('source','bool','extra','escape','unknown-trap','extra-log'):
            v=self.sample()
            if change=='source':v['sources_hex'][0]=b'wrong'.hex()
            if change=='bool':v['restored']['read_decodes']=True
            if change=='extra':v['claimed_pass']=True
            if change=='escape':v['store_root_hex']=b'D:/autostack/plan-artifacts/plan-941/ct-restore-store-1-2/../../other'.hex()
            if change=='unknown-trap':v['failed']['exec_log'][0]['outcome_hex']=b'generic-failure'.hex()
            if change=='extra-log':v['cold']['exec_log'][0]['claimed_pass']=True
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(v)

    def test_native_ct_restore_cannot_use_another_test_identity(self):
        import ace_m1_ct_mechanisms as parent
        v=self.sample()
        with self.assertRaises(m.AdmissionError):
            parent.native_facts(m.EvidenceScope(ROOT),m.EvidenceScope(ROOT),{},lambda *_:None,
                                binary_name=self.restore.BINARY,test_name=parent.TEST)


class SqFactAuthorityTests(unittest.TestCase):
    """Bad-fact controls for the parent; synthetic fixtures are never native evidence."""
    def sample(self):
        import ace_m1_sq_facts as sq
        self.sq=sq;self.source=(ROOT/sq.SOURCE).read_bytes()
        def state(v):return dict(zip(sq.COUNTERS,v))
        def applied(hit):
            return [dict(name_hex=n.encode().hex(),sig_hit=hit,body_hit=hit,body_version=k*64,exprs=e,bindings=b)
                    for n,k,e,b in [('leaf','1',4,1),('twin','1',4,1),('main','2',2,0)]]
        return dict(schema='ace.m1.sq-fact-authority/1',source_hex=self.source.hex(),own_logical='a'*64,
            denied=dict(outcome=dict(phase='query-disabled',query_hex=b'infer-body'.hex(),owner_hex=b'leaf'.hex()),counters=state((0,0,1,0,1,1,0))),
            cold=dict(hir_blake3='9'*64,counters=state((3,0,3,0,3,0,0)),applied=applied(False)),
            hit=dict(hir_blake3='9'*64,counters=state((3,3,3,3,3,0,0)),applied=applied(True)),legacy_hir_blake3='9'*64,
            poisoned=dict(outcome=dict(phase='bind-refused',diagnostics=[dict(stage='bind',code_hex=b'source.fact-mismatch'.hex())]),counters=state((0,0,3,3,0,0,0))),
            own_restore=dict(verify_admitted=True,phase='verified',admitted_bodies=3,hir_blake3='9'*64,leaf_logical='a'*64),
            foreign=[dict(axis=axis,seed_refused=True,restore=dict(verify_admitted=False,phase='restore-refused',admitted_bodies=0,hir_blake3='',leaf_logical=k*64))
                     for axis,k in zip(('package','module','owner'),'bcd')])

    def judge(self,value):return self.sq.validate_payload(value,self.source)

    def test_legitimate_solver_control_facts_and_identity_results_are_accepted(self):
        v=self.sample();self.assertEqual(self.judge(v),dict.fromkeys(self.sq.ARCH,'PASS'))

    def test_actual_foreign_restore_acceptance_cannot_be_hidden_by_green_hir(self):
        for axis in range(3):
            v=self.sample();v['foreign'][axis]['restore'].update(verify_admitted=True,phase='verified',admitted_bodies=3,hir_blake3=v['legacy_hir_blake3'])
            with self.subTest(axis=axis):
                self.assertEqual(self.judge(v)[self.sq.ARCH[0]],'PASS')
                self.assertEqual(self.judge(v)[self.sq.ARCH[1]],'FAIL')

    def test_solver_fallback_fake_hits_or_unused_fact_supply_are_red(self):
        for change in ('cold-accept','warm-solve','diagnostic-fallback','fake-hit','missing-exprs','wrong-hir'):
            v=self.sample()
            if change=='cold-accept':v['denied']['outcome']['phase']='accepted'
            if change=='warm-solve':v['hit']['counters']['solver_runs']=4
            if change=='diagnostic-fallback':v['hit']['counters']['diagnostic_verdicts']=1
            if change=='fake-hit':v['hit']['applied'][0]['body_hit']=False
            if change=='missing-exprs':v['hit']['applied'][0]['exprs']=0
            if change=='wrong-hir':v['hit']['hir_blake3']='0'*64
            with self.subTest(change=change):self.assertEqual(self.judge(v)[self.sq.ARCH[0]],'FAIL')

    def test_poisoned_types_must_refuse_at_bind_before_hir(self):
        for change in ('late','generic-code','hidden-solve'):
            v=self.sample()
            if change=='late':v['poisoned']['outcome']['phase']='verified'
            if change=='generic-code':v['poisoned']['outcome']['diagnostics'][0]['code_hex']=b'generic-error'.hex()
            if change=='hidden-solve':v['poisoned']['counters']['solver_runs']=1
            with self.subTest(change=change):self.assertEqual(set(self.judge(v).values()),{'FAIL'})

    def test_foreign_identity_source_shape_and_counter_types_are_strict(self):
        for change in ('bool-count','extra','same-owner','wrong-source','wrong-key','duplicate-domain'):
            v=self.sample()
            if change=='bool-count':v['cold']['counters']['solver_runs']=True
            if change=='extra':v['foreign'][0]['passed']=True
            if change=='same-owner':v['foreign'][0]['restore']['leaf_logical']=v['own_logical']
            if change=='wrong-source':v['source_hex']='00'
            if change=='wrong-key':v['hit']['applied'][0]['body_version']='bad'
            if change=='duplicate-domain':v['foreign'][1]['axis']='package'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(v)

    def test_sq_native_test_requires_its_exact_original_fixed_group(self):
        import ace_m1_ct_mechanisms as native
        self.sample()
        with self.assertRaises(m.AdmissionError):
            native.native_facts(m.EvidenceScope(ROOT),m.EvidenceScope(ROOT),{},lambda *_:None,
                                binary_name=self.sq.BINARY,test_name=self.sq.TEST,group_name='ace-comptime')


class SyntaxArchitectureTests(unittest.TestCase):
    """Synthetic joins test sensitivity; actual grammar evidence comes from Case wire."""
    def sample(self):
        import ace_m1_syntax_arch as syntax_arch
        self.arch=syntax_arch;self.worker='e'*64
        self.rows=json.loads((ROOT/syntax_arch.CONTRACT).read_bytes())['checks']
        result={}
        for number,row in enumerate(self.rows):
            positive=row['completion']=='success'
            result['syntax/'+row['check']]=dict(schema='ace.m1.syntax-parse-observation/2',source_sha256=row['source_sha256'],response_sha256=f'{number:064x}',grammar=row['grammar'],case_id=row['case'],check_id=row['check'],request_profile=row['request_profile'],entry='main',capabilities=[],worker_sha256=self.worker,primary_diagnostic=None if positive else dict(stage='parse',code='syntax.parse',span_start=0,span_end=1),reconstructed_sha256=row['source_sha256'],diagnostics=0 if positive else 114,artifact_oracles=1 if positive else 0,metric_gaps=['io_bytes'],counters=[],judgement='PASS',scope='unit parse only',parent_status='PASS')
        return result

    def judge(self,value):return self.arch.evaluate(value,self.rows,self.worker)[self.arch.ARCH]

    def test_all_ten_original_checks_join_to_one_grammar_obligation(self):
        self.assertEqual(self.judge(self.sample()),'PASS')

    def test_missing_extra_or_legacy_summary_cannot_authorize_the_row(self):
        for change in ('missing','extra','legacy'):
            v=self.sample()
            if change=='missing':v.pop('syntax/parse-record')
            if change=='extra':v['syntax/future']=v['syntax/parse-record']
            if change=='legacy':v['syntax/parse-core']['schema']='ace.m1.syntax-parse-observation/1'
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(v)

    def test_core_grammar_or_source_cannot_impersonate_record(self):
        for field,value in [('grammar','auto-source@core'),('source_sha256','1'*64),('case_id','ace.m1.unified-source-929.parse-core'),('check_id','parse-core')]:
            v=self.sample();v['syntax/parse-record'][field]=value
            with self.subTest(field=field):self.assertEqual(self.judge(v),'FAIL')

    def test_profile_entry_capabilities_and_worker_identity_remain_separate(self):
        for field,value in [('request_profile','record'),('entry','other'),('capabilities',['unexpected']),('worker_sha256','2'*64)]:
            v=self.sample();v['syntax/parse-record'][field]=value
            with self.subTest(field=field):self.assertEqual(self.judge(v),'FAIL')

    def test_explicit_negative_cannot_be_a_success_or_untyped_late_failure(self):
        key='syntax/parse-record-under-scalar-grammar-refused'
        for change in ('success','late','wrong-code','wrong-source'):
            v=self.sample()
            if change=='success':v[key].update(diagnostics=0,primary_diagnostic=None)
            if change=='late':v[key]['primary_diagnostic']['stage']='backend'
            if change=='wrong-code':v[key]['primary_diagnostic']['code']='generic-error'
            if change=='wrong-source':v[key]['source_sha256']=v['syntax/parse-core']['source_sha256']
            with self.subTest(change=change):self.assertEqual(self.judge(v),'FAIL')

    def test_reconstruction_or_parent_failure_cannot_be_hidden_by_child_pass(self):
        for field,value in [('reconstructed_sha256','3'*64),('artifact_oracles',0),('parent_status','FAIL'),('judgement','FAIL')]:
            v=self.sample();v['syntax/parse-collections'][field]=value
            with self.subTest(field=field):self.assertEqual(self.judge(v),'FAIL')

    def test_unknown_fields_bad_hash_bool_counter_and_outside_span_are_rejected(self):
        for change in ('extra','hash','counter','span'):
            v=self.sample()
            if change=='extra':v['syntax/parse-core']['claimed_arch_pass']=True
            if change=='hash':v['syntax/parse-core']['worker_sha256']='invalid'
            if change=='counter':v['syntax/parse-core']['diagnostics']=True
            if change=='span':v['syntax/parse-record-under-scalar-grammar-refused']['primary_diagnostic']['span_end']=1<<40
            with self.subTest(change=change),self.assertRaises(m.AdmissionError):self.judge(v)

    def test_binding_cannot_switch_to_another_admitted_domain(self):
        self.sample()
        with self.assertRaises(m.AdmissionError):
            self.arch.admit(m.EvidenceScope(ROOT),{}, {},dict(source='bridge'),self.worker)


class FirstSliceFourTargetOracleTests(unittest.TestCase):
    """One shared wrong value across four real reader primitives, not execution evidence."""
    setUp = EvidenceAdmissionTests.setUp
    wire = ScalarObservationTests.wire
    judge_scalar = ScalarObservationTests.judge

    def four_target_facts(self, value):
        observations={key:{'judgement':'PASS'} for key in first_slice.REQUIRED}
        for target,label in [('av','AV'),('native','AC-native')]:
            observations['q4/'+label]=self.judge_scalar(*self.wire(target,value))
        # Both certificate targets consume this original process-pair primitive.
        # Their actual AND reference can agree and still violate the source's
        # independent arithmetic oracle42. No child-verdict or target equality
        # is used to manufacture a pass.
        pair=dict(compile=dict(exit_code=0,stdout_hex='',stderr_hex=''),
                  run=dict(exit_code=0,stdout_hex=f'value:{value}\n'.encode().hex(),stderr_hex=''))
        for label in ('ACE-A2R-HIR','ACE-A2C-HIR'):
            facts=dict(schema='ace.a2x.cert-execution/1',actual=copy.deepcopy(pair),reference=copy.deepcopy(pair))
            matches=cert_evidence.process_matches(facts,b'value:42\n'.hex(),'')
            observations['q4/'+label]=dict(judgement='PASS' if all(matches) else 'FAIL')
        return observations

    def test_independent_42_accepts_four_targets_but_never_promotes_preview(self):
        facts=self.four_target_facts(42)
        self.assertEqual([facts['q4/'+target]['judgement'] for target in first_slice.TARGETS],['PASS']*4)
        report=first_slice.reduce_preview('1'*64,facts,dict(accounting_valid=True,can_claim_m1=False,execution='UNOBSERVED'))
        self.assertTrue(report['first_slice_evidence_complete'])
        self.assertFalse(report['acceptance_system_delivered'])
        self.assertFalse(report['can_claim_expanded_m1'])

    def test_same_wrong_43_on_all_four_targets_and_both_references_remains_red(self):
        facts=self.four_target_facts(43)
        self.assertEqual([facts['q4/'+target]['judgement'] for target in first_slice.TARGETS],['FAIL']*4)
        report=first_slice.reduce_preview('1'*64,facts,dict(accounting_valid=True,can_claim_m1=False,execution='UNOBSERVED'))
        self.assertFalse(report['first_slice_evidence_complete'])
        self.assertFalse(report['gates']['first_slice_evidence'])
        self.assertFalse(report['can_claim_expanded_m1'])


if __name__ == '__main__':
    unittest.main()


class ArchitectureEntryGateStrictControls(unittest.TestCase):
    """PLAN-1005 F-1005-01: the frozen counterexample and the strict envelope.

    The three rows that passed on a green boolean with executed=0 and empty
    results must now be refused or stay UNOBSERVED. Only a real single-group
    run whose complete selection, raw terminal events, command receipts,
    frozen producer window, current configuration and pinned source identity
    all verify can admit entries — and a row then still needs every named
    recipe fact actually executed green inside that gate (V-01..V-06)."""

    @classmethod
    def setUpClass(cls):
        import ace_m1_arch_entry_gates as entry_gates
        cls.entry_gates = entry_gates
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        cls.temp = tempfile.TemporaryDirectory(prefix='arch-strict-gates-', dir=scratch)
        cls.addClassCleanup(cls.temp.cleanup)
        cls.home = Path(cls.temp.name)
        cls.repo = cls.home / 'repo'
        cls.ev = cls.home / 'evidence'
        cls._build_synthetic_repo()
        cls.contract = json.loads((cls.repo / entry_gates.CONTRACT).read_text(encoding='utf-8'))

    @classmethod
    def _put(cls, root, name, value):
        path = root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(value if isinstance(value, bytes) else json.dumps(value).encode())

    @classmethod
    def _build_synthetic_repo(cls):
        import ace_m1_arch_entry_gates as entry_gates
        repo = cls.repo
        # A closed synthetic repository copy: only the tracked inputs the
        # reader actually verifies, plus synthetic producer binaries. The repo
        # is a REAL git repository so the F-1013-01 commit-authenticity gate
        # (R-02) can distinguish a genuine frozen object from a well-formed
        # 40-hex fiction (V-04/V-05).
        for rel in ('scripts/test-groups.toml',
                    'scripts/fixed-tests.py',
                    'scripts/test-governance/case-control-oracles.json',
                    entry_gates.CONTRACT,
                    entry_gates.RECIPES,
                    'crates/auto-lang/test/cases/ace-m1/resident-online-866.suite.ad',
                    'crates/auto-lang/test/cases/ace-m1/resident-online-866.case.ad',
                    'experimental/ace/crates/av/tests/cases/av_resident.rs'):
            cls._put(repo, rel, (ROOT / rel).read_bytes())
        cls._put(repo, 'target/debug/auto-case.exe', b'synthetic-case-parent-control')
        binary = repo / 'experimental/ace/target/debug/deps/conformance-0123456789abcdef.exe'
        cls._put(repo, binary.relative_to(repo), b'synthetic-av-test-binary-control')
        cls.native_binary = binary
        subprocess.run(['git', '-C', str(repo), 'init', '-q'], check=True)
        subprocess.run(['git', '-C', str(repo), 'add', '-A'], check=True)
        subprocess.run(['git', '-C', str(repo), '-c', 'user.name=probe',
                        '-c', 'user.email=probe@local',
                        'commit', '-q', '-m', 'synthetic audit base'], check=True)
        cls.synthetic_commit = subprocess.check_output(
            ['git', '-C', str(repo), 'rev-parse', 'HEAD'], text=True).strip()

    def _digest(self, root, rel):
        return hashlib.sha256((Path(root) / rel).read_bytes()).hexdigest()

    # ---- gate builders ----------------------------------------------------
    def build_case_gate(self, *, commit=None, report_rows=None,
                        executed=None, outcome='PASS', inventory_checks=None):
        """A complete, internally consistent Case gate for the resident entry."""
        commit = self.synthetic_commit if commit is None else commit
        root = self.ev / 'gate-resident'
        if root.exists():
            shutil.rmtree(root)
        repo = self.repo
        group = 'case-ace-m1-resident-866'
        suite = 'crates/auto-lang/test/cases/ace-m1/resident-online-866.suite.ad'
        binary = str(repo / 'target/debug/auto-case.exe')
        inventory_checks = inventory_checks if inventory_checks is not None else [
            {'case': 'ace.m1.res-r01-av866', 'check': 'av-res-r01-continuity', 'suite': 'resident-online-866.suite.ad',
             'state': 'RUN', 'adapter': 'probe', 'recipe': 'ac-av/resident-run-v1',
             'source_sha256': 'a' * 64, 'oracle_sha256': 'b' * 64},
            {'case': 'ace.m1.res-r02-av866', 'check': 'av-res-r02-online-swap', 'suite': 'resident-online-866.suite.ad',
             'state': 'RUN', 'adapter': 'probe', 'recipe': 'ac-av/resident-run-v1',
             'source_sha256': 'a' * 64, 'oracle_sha256': 'b' * 64},
            {'case': 'ace.m1.res-r03-av866', 'check': 'av-res-r03-churn-two-swaps', 'suite': 'resident-online-866.suite.ad',
             'state': 'RUN', 'adapter': 'probe', 'recipe': 'ac-av/resident-run-v1',
             'source_sha256': 'a' * 64, 'oracle_sha256': 'b' * 64}]
        inventory = {'schema': 'test-case-inventory/1', 'checks': inventory_checks}
        rows = report_rows if report_rows is not None else [
            {'case': c['case'], 'check': c['check'], 'status': 'PASS',
             'identity': {'source': c['source_sha256'], 'oracle': c['oracle_sha256']}}
            for c in inventory_checks if c['state'] == 'RUN']
        report = {'schema': 'test-case-report/1', 'results': rows, 'load_errors': [], 'coverage_gaps': []}
        executed = len(rows) if executed is None else executed
        run_command = {'argv': [binary, 'run', '--suite', str(repo / suite), '--out', 'tmcase-control'],
                       'cwd': str(repo), 'exit': 0, 'meter': {'exit': 0, 'error': None},
                       'stdout': 'run.stdout', 'stderr': 'run.stderr'}
        inventory_command = {'argv': [binary, 'inventory', '--suite', str(repo / suite)],
                             'cwd': str(repo), 'exit': 0, 'meter': {'exit': 0, 'error': None},
                             'stdout': 'inventory.stdout', 'stderr': 'inventory.stderr'}
        summary = {'schema': 'fixed-test-run/1', 'selected_groups': [group],
                   'config_sha256': self._digest(repo, 'scripts/test-groups.toml'),
                   'source_state': hashlib.sha256(commit.encode() + b'[]').hexdigest(),
                   'results': [{'outcome': outcome, 'executed': executed}], 'gate_passed': outcome == 'PASS'}
        closure = {'scripts/test-groups.toml': summary['config_sha256'],
                   suite: self._digest(repo, suite),
                   'scripts/test-governance/case-control-oracles.json':
                       self._digest(repo, 'scripts/test-governance/case-control-oracles.json')}
        pins = {'commit': commit, 'source_pins': closure}
        originals = {'results.json': summary, 'inventory.stdout': inventory,
                     'inventory.command.json': inventory_command,
                     'run.command.json': run_command, 'report.json': report}
        gate = {'schema': 'ace.m1.arch-entry-gate/1', 'kind': 'case', 'group': group,
                'commit': commit,
                'binaries': {'target/debug/auto-case.exe': self._digest(repo, 'target/debug/auto-case.exe')},
                'originals': {name: hashlib.sha256(json.dumps(value).encode()).hexdigest()
                              for name, value in sorted(originals.items())}}
        for name, value in sorted(originals.items()):
            self._put(root, name, value)
        self._put(root, 'gate.json', gate)
        self._put(root, 'source-pins-before.json', pins)
        self._put(root, 'source-pins-after.json', pins)
        return root

    def av_tests(self, drop=()):
        document = json.loads((self.repo / self.entry_gates.RECIPES).read_text(encoding='utf-8'))
        recipe = next(r for r in document['recipes'] if r['row'] == 'ARCH-07/one-pid-one-entry')
        events = [t for fact in recipe['facts'] if fact['entry'] == 'av-resident' for t in fact['native_tests']]
        return [e for e in events if e.split('::')[-1] not in drop]

    def build_native_gate(self, *, commit=None, events=None, failed=(), ignored=(),
                          drop=(), result_override=None):
        """A complete, internally consistent native gate for the av-resident entry."""
        commit = self.synthetic_commit if commit is None else commit
        import importlib.util
        import tomllib
        root = self.ev / 'gate-av-resident'
        if root.exists():
            shutil.rmtree(root)
        repo = self.repo
        group_name = 'ace-av'
        spec = importlib.util.spec_from_file_location('strict_parent', repo / 'scripts/fixed-tests.py')
        parent = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(parent)
        config = tomllib.loads((repo / 'scripts/test-groups.toml').read_text(encoding='utf-8'))
        group = next(g for g in config['group'] if g['name'] == group_name)
        events = self.av_tests(drop) if events is None else events
        testcases = {}
        selected, excluded = [], []
        for event in events:
            name = event.split('$', 1)[1]
            bare = name.rsplit('::', 1)[-1]
            if bare in ignored:
                # the real listing keeps conditionally-ignored tests visible
                testcases[name] = dict(kind='test', ignored=True, **{'filter-match': {'status': 'matches'}})
                excluded.append({'binary': 'ace-av::conformance', 'test': name, 'ignored': True})
            else:
                selected.append({'binary': 'ace-av::conformance', 'test': name, 'ignored': False})
                testcases[name] = dict(kind='test', ignored=False, **{'filter-match': {'status': 'matches'}})
        listing = {'rust-suites': {'ace-av::conformance': {
            'package-name': 'ace-av', 'binary-name': 'conformance',
            'binary-path': str(self.native_binary), 'testcases': testcases}}}
        terminals = {}
        raw_lines = []
        for row in selected:
            name = row['test']
            event_name = 'ace-av::conformance$' + name
            terminal_event = 'failed' if name.rsplit('::', 1)[-1] in failed else 'ok'
            terminals[event_name] = {'type': 'test', 'event': terminal_event, 'name': event_name, 'exec_time': 0.01}
            raw_lines.append({'type': 'test', 'event': 'started', 'name': event_name})
            raw_lines.append(terminals[event_name])
        raw = ('\n'.join(json.dumps(line) for line in raw_lines) + '\n').encode()
        errors = ['failed or unexpectedly skipped test'] if failed else []
        outcome = 'FAIL' if failed else 'PASS'
        exit_code = 1 if failed else 0
        plan_group = parent.invocations([dict(group)])[0]
        expected_argv = ['cargo', 'nextest', 'run', *parent.nextest_args(plan_group),
                         '--no-fail-fast', '--no-tests', 'fail',
                         '--message-format', 'libtest-json-plus', '--message-format-version', '0.1']
        listing_command = {'argv': ['cargo', 'nextest', 'list', *parent.nextest_args(plan_group),
                                    '--run-ignored', 'all', '--message-format', 'json'],
                           'cwd': str(repo / 'experimental/ace'), 'exit': 0,
                           'meter': {'exit': 0, 'error': None},
                           'stdout': 'discovery.stdout', 'stderr': 'discovery.stderr'}
        command = {'argv': expected_argv, 'cwd': str(repo / 'experimental/ace'), 'exit': exit_code,
                   'meter': {'exit': exit_code, 'error': None}, 'stdout': 'run.stdout', 'stderr': 'run.stderr'}
        result = {'outcome': outcome, 'executed': len(terminals), 'selected': selected,
                  'excluded': excluded, 'results': terminals, 'not_run': [], 'errors': errors}
        if result_override:
            result = result | result_override
        summary = {'schema': 'fixed-test-run/1', 'selected_groups': [group_name],
                   'config_sha256': self._digest(repo, 'scripts/test-groups.toml'),
                   'source_state': hashlib.sha256(commit.encode() + b'[]').hexdigest(),
                   'results': [result], 'gate_passed': outcome == 'PASS'}
        source_pin = 'experimental/ace/crates/av/tests/cases/av_resident.rs'
        closure = {'scripts/test-groups.toml': summary['config_sha256'],
                   source_pin: self._digest(repo, source_pin)}
        pins = {'commit': commit, 'source_pins': closure}
        binaries = {'experimental/ace/target/debug/deps/conformance-0123456789abcdef.exe':
                    self._digest(repo, self.native_binary.relative_to(repo))}
        originals = {'results.json': summary, 'discovery.stdout': listing,
                     'run.command.json': command, 'discovery.command.json': listing_command}
        gate = {'schema': 'ace.m1.arch-entry-gate/1', 'kind': 'native', 'group': group_name,
                'commit': commit, 'binaries': binaries,
                'originals': {name: hashlib.sha256(json.dumps(value).encode()).hexdigest()
                              for name, value in sorted(originals.items())}}
        for name, value in sorted(originals.items()):
            self._put(root, name, value)
        self._put(root, 'run.stdout', raw)
        self._put(root, 'run.stderr', b'')
        gate['originals']['run.stdout'] = self._digest(root, 'run.stdout')
        gate['originals']['run.stderr'] = self._digest(root, 'run.stderr')
        self._put(root, 'gate.json', gate)
        self._put(root, 'source-pins-before.json', pins)
        self._put(root, 'source-pins-after.json', pins)
        return root

    def admit(self, entries, frozen_anchors=()):
        inputs = m.EvidenceScope(self.repo)
        evidence = m.EvidenceScope(self.ev)
        return self.entry_gates.admit_arch_entry_gates(
            inputs, evidence, {'entries': entries}, frozen_anchors=frozen_anchors)

    def both_entries(self):
        return {'resident': {'root': 'gate-resident'},
                'av-resident': {'root': 'gate-av-resident'}}

    # ---- PLAN-1013 T-03 facts re-derivation helpers ------------------------
    def build_facts_doc(self, gate_root, entry, kind, group):
        """Project a gate's admitted result into the phased facts shape."""
        gate = json.loads((gate_root / 'gate.json').read_text(encoding='utf-8'))
        if kind == 'native':
            root = self.ev / gate_root
            kept = {}
            for line in (root / 'run.stdout').read_bytes().decode('utf-8').splitlines():
                if line.startswith('{'):
                    row = json.loads(line)
                    if row.get('type') == 'test' and row.get('event') == 'ok':
                        kept[row['name']] = 'ok'
        else:
            report = json.loads((gate_root / 'report.json').read_text(encoding='utf-8'))
            kept = {f"{row['case']}|{row['check']}": 'PASS'
                    for row in report['results'] if row['status'] == 'PASS'}
        return {'schema': 'ace.m1.arch-entry-facts/1', 'entry': entry, 'group': group,
                'kind': kind, 'commit': gate['commit'], 'green': True,
                'executed': len(kept), 'kept': kept,
                'gate_originals_sha256': gate['originals']}

    def put_facts(self, doc, name):
        root = self.ev / name
        root.parent.mkdir(parents=True, exist_ok=True)
        root.write_bytes(json.dumps(doc, ensure_ascii=False, separators=(',', ':')).encode('utf-8'))
        return name, hashlib.sha256(root.read_bytes()).hexdigest()

    def test_v04_fake_commit_with_complete_originals_refused(self):
        # F-1013-01 R-02: a well-formed 40-hex commit that resolves to NO
        # object in the audited repository is refused even when every
        # original, receipt and window is complete and rehashed — the
        # authenticity gate must be what trips, not a missing raw file.
        root = self.build_case_gate(commit='f' * 40)
        pins = {'commit': 'f' * 40, 'source_pins': json.loads(
            (root / 'source-pins-before.json').read_text(encoding='utf-8'))['source_pins']}
        for name in ('source-pins-before.json', 'source-pins-after.json'):
            self._put(root, name, pins)
        summary = json.loads((root / 'results.json').read_text(encoding='utf-8'))
        summary['source_state'] = hashlib.sha256(b'f' * 40 + b'[]').hexdigest()
        self._put(root, 'results.json', summary)
        gate = json.loads((root / 'gate.json').read_text(encoding='utf-8'))
        gate['originals']['results.json'] = self._digest(root, 'results.json')
        self._put(root, 'gate.json', gate)
        with self.assertRaisesRegex(m.AdmissionError, 'not a real commit object'):
            self.admit({'resident': {'root': 'gate-resident'}})

    def test_v05_real_but_inapplicable_commit_and_anchor_gap_refused(self):
        # R-02: a real commit from the WRONG tree (another repository
        # identity) fails the audited-repo object check; a real commit inside
        # this repository but OUTSIDE the frozen anchor closure fails the
        # anchor membership gate — neither can unlock rows.
        self.build_case_gate()
        # A second real commit in the audited synthetic repo: it exists as an
        # object, but it is not the audited frozen producer identity.
        (self.repo / 'second-marker').write_bytes(b'second producer state')
        subprocess.run(['git', '-C', str(self.repo), 'add', '-A'], check=True)
        subprocess.run(['git', '-C', str(self.repo), '-c', 'user.name=probe',
                        '-c', 'user.email=probe@local', 'commit', '-q', '-m', 'second'], check=True)
        second = subprocess.check_output(['git', '-C', str(self.repo), 'rev-parse', 'HEAD'],
                                         text=True).strip()
        root = self.ev / 'gate-resident'
        pins = {'commit': second, 'source_pins': json.loads(
            (root / 'source-pins-before.json').read_text(encoding='utf-8'))['source_pins']}
        for name in ('source-pins-before.json', 'source-pins-after.json'):
            self._put(root, name, pins)
        summary = json.loads((root / 'results.json').read_text(encoding='utf-8'))
        summary['source_state'] = hashlib.sha256(second.encode() + b'[]').hexdigest()
        self._put(root, 'results.json', summary)
        gate = json.loads((root / 'gate.json').read_text(encoding='utf-8'))
        gate['commit'] = second
        gate['originals']['results.json'] = self._digest(root, 'results.json')
        self._put(root, 'gate.json', gate)
        # Real object in the correct repo, but outside the frozen anchors.
        with self.assertRaisesRegex(m.AdmissionError, 'outside the frozen anchor closure'):
            self.admit({'resident': {'root': 'gate-resident'}},
                       frozen_anchors=(self.synthetic_commit,))
        # With the anchor closed to the actual producer commit the same gate
        # admits (positive control for the same authentic material).
        out = self.admit({'resident': {'root': 'gate-resident'}},
                         frozen_anchors=(second,))
        self.assertEqual(out['admitted_entries'], ['resident'])
        self.assertEqual(out['gate_commits'], [second])

    def test_v02_missing_each_required_original_refused(self):
        # R-01: deleting ANY pinned original (run/discovery output, receipts,
        # inventory/report, window) must refuse — a non-empty hash table never
        # proves the file exists (F-1013-01). Each deletion is re-admitted
        # against the same gate with the facts reference present.
        self.build_case_gate()
        self.build_native_gate()
        for root_name, rel in (('gate-resident', 'results.json'),
                               ('gate-resident', 'inventory.stdout'),
                               ('gate-resident', 'inventory.command.json'),
                               ('gate-resident', 'run.command.json'),
                               ('gate-resident', 'report.json'),
                               ('gate-resident', 'source-pins-before.json'),
                               ('gate-resident', 'source-pins-after.json'),
                               ('gate-av-resident', 'results.json'),
                               ('gate-av-resident', 'discovery.stdout'),
                               ('gate-av-resident', 'discovery.command.json'),
                               ('gate-av-resident', 'run.command.json'),
                               ('gate-av-resident', 'run.stdout')):
            with self.subTest(root=root_name, original=rel):
                gate_root = self.ev / root_name
                missing = gate_root / rel
                saved = missing.read_bytes()
                missing.unlink()
                try:
                    with self.assertRaises(m.AdmissionError):
                        self.admit(self.both_entries())
                finally:
                    missing.write_bytes(saved)

    def test_v01_phased_facts_requires_root_and_rederives_originals(self):
        # F-1013-01: a facts document WITHOUT the original root (old schema)
        # is refused — no fallback to the self-reported branch.
        self.build_case_gate()
        self.build_native_gate()
        facts = self.build_facts_doc(self.ev / 'gate-av-resident', 'av-resident', 'native', 'ace-av')
        name, sha = self.put_facts(facts, 'av-resident.facts.json')
        with self.assertRaises(m.AdmissionError):
            self.admit({'av-resident': {'facts': {'path': name, 'sha256': sha}}})
        # With the root present, the reader re-derives and compares.
        out = self.admit({'av-resident': {'facts': {'path': name, 'sha256': sha},
                                          'root': 'gate-av-resident'}})
        self.assertEqual(out['admitted_entries'], ['av-resident'])

    def test_v03_phased_facts_tampered_projection_rehashed_refused(self):
        # R-01/R-03: flipping one kept row and rehashing every local hash must
        # not change the verdict — the re-derived original decides.
        self.build_case_gate()
        self.build_native_gate()
        facts = self.build_facts_doc(self.ev / 'gate-av-resident', 'av-resident', 'native', 'ace-av')
        facts['kept'] = dict(facts['kept'])
        key = sorted(facts['kept'])[0]
        facts['kept'][key] = 'failed' if facts['kept'][key] == 'ok' else 'ok'
        name, sha = self.put_facts(facts, 'av-resident.facts-tampered.json')
        with self.assertRaisesRegex(m.AdmissionError, 'kept drift'):
            self.admit({'av-resident': {'facts': {'path': name, 'sha256': sha},
                                        'root': 'gate-av-resident'}})

    def test_v04_phased_facts_fake_commit_refused(self):
        # R-02: the facts projection may name a fake commit; the original root
        # carries a REAL commit — the drift is caught at re-derivation.
        self.build_case_gate()
        self.build_native_gate()
        facts = self.build_facts_doc(self.ev / 'gate-av-resident', 'av-resident', 'native', 'ace-av')
        facts['commit'] = 'f' * 40
        name, sha = self.put_facts(facts, 'av-resident.facts-fake-commit.json')
        with self.assertRaisesRegex(m.AdmissionError, 'commit drift'):
            self.admit({'av-resident': {'facts': {'path': name, 'sha256': sha},
                                        'root': 'gate-av-resident'}})
        # The facts document may also omit the originals identity entirely.
        facts2 = dict(facts)
        facts2['commit'] = self.synthetic_commit
        facts2['gate_originals_sha256'] = {'missing': '0' * 64}
        name2, sha2 = self.put_facts(facts2, 'av-resident.facts-bad-originals.json')
        with self.assertRaises(m.AdmissionError):
            self.admit({'av-resident': {'facts': {'path': name2, 'sha256': sha2},
                                        'root': 'gate-av-resident'}})

    # ---- V-01: the frozen counterexample ----------------------------------
    def test_v01_green_boolean_with_zero_execution_is_refused(self):
        # The exact pre-fix counterexample payload: correct group, green
        # boolean, executed=0, empty results. The strict envelope refuses the
        # shape entirely (no such gate directory can verify).
        fake_root = self.ev / 'gate-fake'
        if fake_root.exists():
            shutil.rmtree(fake_root)
        fake_root.mkdir(parents=True)
        for name, doc in (('resident.json', {'selected_groups': ['case-ace-m1-resident-866'],
                                             'gate_passed': True, 'results': {}, 'executed': 0}),
                          ('av-resident.json', {'selected_groups': ['ace-av'],
                                                'gate_passed': True, 'results': {}, 'executed': 0})):
            self._put(fake_root, name, doc)
        with self.assertRaises(m.AdmissionError):
            self.admit({'resident': {'root': 'gate-fake/resident'},
                        'av-resident': {'root': 'gate-fake/av-resident'}})

    def test_v01_wellformed_zero_selection_is_refused(self):
        # Same-type well-formed control: a native gate selecting zero tests is
        # refused outright; it can never be green.
        self.build_case_gate()
        self.build_native_gate(events=[])
        with self.assertRaises(m.AdmissionError):
            self.admit(self.both_entries())

    def test_legitimate_consistent_gates_admit_the_rows(self):
        # The old legitimate behaviour is preserved and strengthened: a fully
        # consistent real-shape gate pair admits the two ARCH-07 rows.
        self.build_case_gate()
        self.build_native_gate()
        out = self.admit(self.both_entries())
        self.assertEqual(out['observations'].get('ARCH-07/one-pid-one-entry'), 'PASS')
        self.assertEqual(out['observations'].get('ARCH-07/last-owner-gc'), 'PASS')
        self.assertNotIn('ARCH-08/fresh-union-spec-review', out['observations'])
        self.assertNotIn('ARCH-07/original-churn-and-samples', out['observations'])

    # ---- V-02..V-06 differential controls ---------------------------------
    def test_v02_green_summary_with_failed_terminal_refused(self):
        # gate_passed=true / outcome=PASS painted over a failed terminal event
        # and a nonzero exit cannot verify: terminals are re-derived from the
        # raw output and must equal the recorded selection.
        self.build_case_gate()
        root = self.build_native_gate(failed={'av_resident_restart_variant_is_red'})
        summary = json.loads((root / 'results.json').read_text(encoding='utf-8'))
        result = summary['results'][0]
        result['outcome'] = 'PASS'
        result['errors'] = []
        result['results']['ace-av::conformance$av_resident::av_resident_restart_variant_is_red']['event'] = 'ok'
        summary['gate_passed'] = True
        self._put(root, 'results.json', summary)
        gate = json.loads((root / 'gate.json').read_text(encoding='utf-8'))
        gate['originals']['results.json'] = self._digest(root, 'results.json')
        self._put(root, 'gate.json', gate)
        with self.assertRaises(m.AdmissionError):
            self.admit(self.both_entries())

    def test_v02_executed_count_mismatch_refused(self):
        self.build_case_gate()
        self.build_native_gate(result_override={'executed': 999})
        with self.assertRaises(m.AdmissionError):
            self.admit(self.both_entries())

    def test_v03_named_recipe_fact_deleted_row_not_pass(self):
        # Group stays fully green; deleting ONE named semantic test (the
        # restart negative control) from the consistent gate must drop the
        # row to UNOBSERVED — the core F-1005-01 fix.
        self.build_case_gate()
        self.build_native_gate(drop={'av_resident_restart_variant_is_red'})
        out = self.admit(self.both_entries())
        self.assertNotIn('ARCH-07/one-pid-one-entry', out['observations'])
        self.assertNotIn('ARCH-07/last-owner-gc', out['observations'])

    def test_v03_named_recipe_fact_conditionally_excluded_row_not_pass(self):
        self.build_case_gate()
        self.build_native_gate(ignored={'av_resident_restart_variant_is_red'})
        out = self.admit(self.both_entries())
        self.assertNotIn('ARCH-07/one-pid-one-entry', out['observations'])

    def test_v04_red_gate_marks_row_fail_not_unobserved(self):
        self.build_case_gate()
        self.build_native_gate(failed={'av_resident_slice_invariance_and_chain_continuity'})
        out = self.admit(self.both_entries())
        self.assertEqual(out['observations'].get('ARCH-07/one-pid-one-entry'), 'FAIL')

    def test_v06_tampered_raw_output_with_rehashed_summary_refused(self):
        # Re-hash every local file after mutating the raw terminal stream: the
        # re-derived terminals no longer equal the recorded selection, so the
        # internally-rehashed forgery is still refused.
        self.build_case_gate()
        root = self.build_native_gate()
        raw = (root / 'run.stdout').read_bytes()
        (root / 'run.stdout').write_bytes(raw.replace(b'"ok"', b'"failed"', 1))
        gate = json.loads((root / 'gate.json').read_text(encoding='utf-8'))
        gate['originals']['run.stdout'] = self._digest(root, 'run.stdout')
        self._put(root, 'gate.json', gate)
        with self.assertRaises(m.AdmissionError):
            self.admit(self.both_entries())

    def test_v06_stale_binary_bytes_refused(self):
        self.build_case_gate()
        self.build_native_gate()
        self.native_binary.write_bytes(b'different synthetic producer bytes')
        try:
            with self.assertRaises(m.AdmissionError):
                self.admit(self.both_entries())
        finally:
            self.native_binary.write_bytes(b'synthetic-av-test-binary-control')

    def test_missing_entry_binding_leaves_row_unobserved(self):
        self.build_case_gate()
        self.build_native_gate()
        out = self.admit({'resident': {'root': 'gate-resident'}})
        self.assertNotIn('ARCH-07/one-pid-one-entry', out['observations'])

    def test_wrong_kind_or_group_and_foreign_entry_refused(self):
        self.build_case_gate()
        self.build_native_gate()
        with self.assertRaises(m.AdmissionError):
            self.admit({'resident': {'root': 'gate-resident'},
                        'av-resident': {'root': 'gate-resident'}})  # case kind, not native
        with self.assertRaises(m.AdmissionError):
            self.admit({'not-in-the-contract': {'root': 'gate-resident'}})


class ReviewCredentialControls(unittest.TestCase):
    """PLAN-1005 F-1005-02 (V-07..V-09): the review row is a credential input
    gate. Group PASS, self-checks and reviewer strings never close it; a
    structurally valid credential binds the frozen code, the pre-signature
    candidate and the composition's own identities (acyclic: the final report
    references the credential, never the reverse). These are SYNTHETIC
    CONTROLS of the consumer's accept/reject rules, never production audit
    evidence."""

    @classmethod
    def setUpClass(cls):
        import ace_m1_review_evidence as review
        import uuid
        cls.uuid = uuid
        cls.review = review
        scratch = Path(os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir()))
        cls.temp = tempfile.TemporaryDirectory(prefix='arch-review-credential-', dir=scratch)
        cls.addClassCleanup(cls.temp.cleanup)
        cls.home = Path(cls.temp.name)

    def _put(self, root, name, value):
        (root / name).write_bytes(value if isinstance(value, bytes) else json.dumps(value).encode())

    def _digest(self, root, name):
        return hashlib.sha256((root / name).read_bytes()).hexdigest()

    def sample(self, *, candidate=None, credential=None):
        """A consistent credential + candidate + composition triple."""
        from ace_m1_arch_obligations import REQUIRED
        from ace_m1_arch_entry_gates import RECIPES_SHA
        ev = self.home / ('ev-' + self.uuid.uuid4().hex[:8])
        ev.mkdir(parents=True)
        rows = [{'id': key, 'verdict': 'PASS' if key != self.review.REVIEW_ROW else 'UNOBSERVED'}
                for key in REQUIRED]
        composition = {'architecture_execution': {'gate': {'rows': [copy.deepcopy(r) for r in rows]}},
                       'parent_bindings_sha256': 'c' * 64,
                       'admission_sources': {'scripts/test-governance/ace_m1_first_slice.py': 'd' * 64}}
        if candidate is None:
            candidate = {'architecture_execution': {'gate': {'rows': [copy.deepcopy(r) for r in rows]}},
                         'can_claim_expanded_m1': False,
                         'parent_bindings_sha256': composition['parent_bindings_sha256'],
                         'admission_sources': dict(composition['admission_sources'])}
        base = {'schema': 'ace.m1.pre-signature-review-credential/1', 'plan': self.review.PLAN,
                'plan_revision': self.review.REVISION, 'frozen_commit': '1' * 40,
                'candidate': {'bindings_sha256': composition['parent_bindings_sha256'],
                              'pre_signature_report_sha256': None},
                'row_rules_sha256': RECIPES_SHA,
                'registry_sha256': hashlib.sha256(
                    (ROOT / 'crates/auto-lang/test/cases/ace-m1-expanded/registry.json').read_bytes()).hexdigest(),
                'audited': {'arch_rows': 48, 'registered_keys': 1006,
                            'excluded_row': self.review.REVIEW_ROW},
                'reviewer': {'role': 'independent-final-review',
                             'context': 'synthetic-unit-control', 'method': 'counterexample replay'},
                'coverage': {'acceptance_criteria': self.review.AC_IDS,
                             'verifications': self.review.V_IDS,
                             'spec_deltas': self.review.SD_IDS},
                'findings': [], 'verdict': 'pass'}
        self._put(ev, 'candidate.json', candidate)
        base['candidate']['pre_signature_report_sha256'] = self._digest(ev, 'candidate.json')
        if credential is not None:
            base = credential
        self._put(ev, 'credential.json', base)
        binding = {'credential': {'path': 'credential.json', 'sha256': self._digest(ev, 'credential.json')},
                   'pre_signature_candidate': {'path': 'candidate.json',
                                               'sha256': self._digest(ev, 'candidate.json')}}
        return ev, binding, composition, frozenset({'1' * 40})

    def admit(self, prepared):
        ev, binding, composition, commits = prepared
        return self.review.admit_review_credential(
            m.EvidenceScope(ROOT), m.EvidenceScope(ev), binding, composition, commits)

    def test_v09_valid_control_credential_supplies_exactly_the_review_row(self):
        out = self.admit(self.sample())
        self.assertEqual(out['observations'], {self.review.REVIEW_ROW: 'PASS'})
        self.assertIn('credential_sha256', out)

    def test_v08_wrong_bindings_sha_refused(self):
        ev, binding, composition, commits = self.sample()
        credential = json.loads((ev / 'credential.json').read_text(encoding='utf-8'))
        credential['candidate']['bindings_sha256'] = 'e' * 64
        self._put(ev, 'credential.json', credential)
        binding['credential']['sha256'] = self._digest(ev, 'credential.json')
        with self.assertRaises(m.AdmissionError):
            self.review.admit_review_credential(m.EvidenceScope(ROOT), m.EvidenceScope(ev),
                                                binding, composition, commits)

    def test_v08_drifted_candidate_refused(self):
        ev, binding, composition, commits = self.sample()
        candidate = json.loads((ev / 'candidate.json').read_text(encoding='utf-8'))
        candidate['architecture_execution']['gate']['rows'][0]['verdict'] = 'UNOBSERVED'
        self._put(ev, 'candidate.json', candidate)
        binding['pre_signature_candidate']['sha256'] = self._digest(ev, 'candidate.json')
        with self.assertRaises(m.AdmissionError):
            self.review.admit_review_credential(m.EvidenceScope(ROOT), m.EvidenceScope(ev),
                                                binding, composition, commits)

    def test_v08_candidate_must_leave_the_review_row_unobserved(self):
        from ace_m1_arch_obligations import REQUIRED
        rows = [{'id': key, 'verdict': 'PASS'} for key in REQUIRED]
        candidate = {'architecture_execution': {'gate': {'rows': rows}},
                     'can_claim_expanded_m1': False,
                     'parent_bindings_sha256': 'c' * 64,
                     'admission_sources': {'scripts/test-governance/ace_m1_first_slice.py': 'd' * 64}}
        with self.assertRaises(m.AdmissionError):
            self.admit(self.sample(candidate=candidate))

    def test_v08_unresolved_finding_wrong_coverage_verdict_and_cycles_refused(self):
        mutations = (
            lambda c: c.__setitem__('findings', [{'id': 'F-1', 'disposition': 'open', 'note': 'x'}]),
            lambda c: c['coverage'].__setitem__('spec_deltas', ['SD-01']),
            lambda c: c.__setitem__('verdict', 'needs_fix'),
            lambda c: c.__setitem__('frozen_commit', '2' * 40),
            lambda c: c.__setitem__('final_report_sha256', 'f' * 64),
            lambda c: c['audited'].__setitem__('excluded_row', 'ARCH-07/one-pid-one-entry'),
        )
        for mutate in mutations:
            with self.subTest(name=getattr(mutate, '__name__', 'mutation')):
                ev, binding, composition, commits = self.sample()
                credential = json.loads((ev / 'credential.json').read_text(encoding='utf-8'))
                mutate(credential)
                self._put(ev, 'credential.json', credential)
                binding['credential']['sha256'] = self._digest(ev, 'credential.json')
                with self.assertRaises(m.AdmissionError):
                    self.review.admit_review_credential(m.EvidenceScope(ROOT), m.EvidenceScope(ev),
                                                        binding, composition, commits)

    def test_v08_old_plan1005_credential_or_wrong_revision_or_coverage_refused(self):
        # F-1013-01 (plan 5.3): a legacy PLAN-1005 credential, a wrong
        # plan_revision, or a partial/foreign coverage set can never claim
        # this fix's fresh review — even with correct frozen_commit/candidate.
        for mutate in (
            lambda c: c.__setitem__('plan', 'PLAN-1005'),
            lambda c: c.__setitem__('plan_revision', 2),
            lambda c: c['coverage'].__setitem__('acceptance_criteria',
                                                [f'AC-{i:02}' for i in range(1, 9)]),
            lambda c: c['coverage'].__setitem__('verifications',
                                                [f'V-{i:02}' for i in range(1, 14)]),
            lambda c: c['audited'].__setitem__('registered_keys', 1005),
        ):
            with self.subTest(name=getattr(mutate, '__name__', 'mutation')):
                ev, binding, composition, commits = self.sample()
                credential = json.loads((ev / 'credential.json').read_text(encoding='utf-8'))
                mutate(credential)
                self._put(ev, 'credential.json', credential)
                binding['credential']['sha256'] = self._digest(ev, 'credential.json')
                with self.assertRaises(m.AdmissionError):
                    self.review.admit_review_credential(m.EvidenceScope(ROOT), m.EvidenceScope(ev),
                                                        binding, composition, commits)

    def test_v08_wrong_row_rules_sha_refused(self):
        ev, binding, composition, commits = self.sample()
        credential = json.loads((ev / 'credential.json').read_text(encoding='utf-8'))
        credential['row_rules_sha256'] = '0' * 64
        self._put(ev, 'credential.json', credential)
        binding['credential']['sha256'] = self._digest(ev, 'credential.json')
        with self.assertRaises(m.AdmissionError):
            self.review.admit_review_credential(m.EvidenceScope(ROOT), m.EvidenceScope(ev),
                                                binding, composition, commits)


class ArchitectureMergeBoundaryControls(unittest.TestCase):
    """PLAN-1005 T-06 (V-07/V-10): every architecture merge path shares one
    trust boundary — the review row never rides entries, mechanisms, or
    phased section injection, and conflicting verdicts are refused."""

    ROW = 'ARCH-08/fresh-union-spec-review'

    def test_v07_merge_refuses_the_review_row_from_any_family(self):
        for label, incoming in (('entry gates', {'ARCH-07/one-pid-one-entry': 'PASS', self.ROW: 'PASS'}),
                                ('mechanism families', {self.ROW: 'PASS'}),
                                ('phased section', {self.ROW: 'UNOBSERVED'})):
            with self.subTest(label=label):
                with self.assertRaises(m.AdmissionError):
                    first_slice.merge_architecture_observations({}, incoming, label)

    def test_conflicting_verdicts_are_refused_not_overridden(self):
        with self.assertRaises(m.AdmissionError):
            first_slice.merge_architecture_observations(
                {'ARCH-07/one-pid-one-entry': 'PASS'}, {'ARCH-07/one-pid-one-entry': 'FAIL'}, 'mechanism families')
        merged = first_slice.merge_architecture_observations(
            {'ARCH-07/one-pid-one-entry': 'PASS'}, {'ARCH-07/one-pid-one-entry': 'PASS'}, 'entry gates')
        self.assertEqual(merged['ARCH-07/one-pid-one-entry'], 'PASS')

    def test_v10_phased_sections_are_whitelisted_and_architecture_free(self):
        home = Path(tempfile.mkdtemp(prefix='phased-boundary-',
                                     dir=os.environ.get('PLAN941_SCRATCH', tempfile.gettempdir())))
        self.addCleanup(shutil.rmtree, home, True)
        evidence = m.EvidenceScope(home)
        good = {'schema': 'ace.m1.phased-observations/1',
                'observations': {'text/src-01': {'judgement': 'PASS'}}}
        (home / 'good.json').write_text(json.dumps(good), encoding='utf-8')
        result = {}
        first_slice.admit_phased_observations(
            evidence, {'text_certifications': {'path': 'good.json', 'sha256': self._digest(home, 'good.json')}}, result)
        self.assertIn('text_certifications', result)
        arch = {'schema': 'ace.m1.phased-observations/1',
                'observations': {self.ROW: 'PASS'}}
        (home / 'arch.json').write_text(json.dumps(arch), encoding='utf-8')
        with self.assertRaises(m.AdmissionError):
            first_slice.admit_phased_observations(
                evidence, {'text_certifications': {'path': 'arch.json', 'sha256': self._digest(home, 'arch.json')}}, {})
        section = {'schema': 'ace.m1.phased-observations/1',
                   'section': {'schema': 'ace.m1.architecture-execution/1',
                               'observations': {self.ROW: 'PASS'}, 'gate': {'complete': True}}}
        (home / 'section.json').write_text(json.dumps(section), encoding='utf-8')
        with self.assertRaises(m.AdmissionError):
            first_slice.admit_phased_observations(
                evidence, {'incremental_restore_measurements': {'path': 'section.json', 'sha256': self._digest(home, 'section.json')}}, {})

    def _digest(self, root, name):
        return hashlib.sha256((root / name).read_bytes()).hexdigest()


if __name__ == '__main__':
    unittest.main()
