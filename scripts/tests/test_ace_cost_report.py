#!/usr/bin/env python3
"""PLAN-893 V-03/V-04/V-05: read-only attribution cannot mint green verdicts.

Synthetic run fixtures (no builds, no product code) drive the same code paths
as real `cargo t --out` artifacts: verdict linkage needs real terminal events,
canceled/timeout exits stay red, phases are attributed to the bucket that
actually spent the time (and the total window responds), intra-run phases stay
UNOBSERVED instead of zero, stale runs are marked, and existing reports are
never overwritten.
"""
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "test-governance"))
import ace_cost_report as report


POLICY = {
    'schema': 'ace-test-policy/1',
    'items': [{
        'id': 'K-99',
        'family_owner': 'syntax',
        'native': {'binary': 'ace-syntax-artifacts::conformance',
                   'test': 'syntax_budget::performance::fixture_atom',
                   'fn': 'fixture_atom',
                   'source_file': 'experimental/ace/nonexistent.rs'},
    }],
}


def command_record(name, *, exit_code=0, wall_ms=1000.0, cpu_ms=500.0, peak=1024, error=None):
    return {'argv': ['fake', name], 'cwd': 'X:\\fake', 'exit': exit_code,
            'meter': {'wall_ms': wall_ms, 'workload_cpu_ms': cpu_ms, 'meter_cpu_ms': 1.0,
                      'peak_job_bytes': peak, 'job_processes': 2, 'error': error},
            'stdout': name + '.stdout', 'stderr': name + '.stderr'}


def event(name, outcome, exec_time=None):
    row = {'type': 'test', 'event': outcome, 'name': name}
    if exec_time is not None:
        row['exec_time'] = exec_time
    return row


class FixtureRun:
    """A synthetic fixed-driver --out directory."""

    def __init__(self, root, *, outcome='PASS', executed=1, run_exit=0, run_error=None,
                 run_wall=20000.0, discovery_wall=8000.0, tool_walls=(), events=None,
                 with_run_command=True, omit_terminals=False):
        self.dir = Path(root) / 'run'
        index = self.dir / '0'
        index.mkdir(parents=True)
        self.summary = {
            'schema': 'fixed-test-run/1', 'selected_groups': ['ace-syntax'],
            'config_sha256': '0' * 64, 'source_state': 'deadbeef',
            'invocations': [], 'wall_seconds': (discovery_wall + run_wall + sum(tool_walls)) / 1000.0,
            'results': [{
                'outcome': outcome, 'executed': executed, 'groups': ['ace-syntax'],
                'selected': [{'binary': 'ace-syntax-artifacts::conformance',
                              'test': 'syntax_budget::performance::fixture_atom',
                              'ignored': False}],
                'errors': [],
            }],
            'gate_passed': outcome == 'PASS',
        }
        (index / 'discovery.command.json').write_text(
            json.dumps(command_record('discovery', wall_ms=discovery_wall)), encoding='utf-8')
        if with_run_command:
            (index / 'run.command.json').write_text(
                json.dumps(command_record('run', exit_code=run_exit, wall_ms=run_wall, error=run_error)),
                encoding='utf-8')
        lines = [json.dumps(e) for e in (events if events is not None else
                 ([] if omit_terminals else [event('ace-syntax-artifacts::conformance$syntax_budget::performance::fixture_atom', 'ok', 12.5)]))]
        (index / 'run.stdout').write_text('\n'.join(lines) + '\n', encoding='utf-8')
        for i, wall in enumerate(tool_walls):
            tools = index / 'tools'
            tools.mkdir(exist_ok=True)
            (tools / ('tool-%d.command.json' % i)).write_text(
                json.dumps(command_record('tool-%d' % i, wall_ms=wall)), encoding='utf-8')
        (self.dir / 'results.json').write_text(json.dumps(self.summary), encoding='utf-8')

    def attribute(self):
        items = report.load_policy_items(POLICY)
        return report.attribute_run(self.dir, items)


class VerdictLinkageTests(unittest.TestCase):

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_green_run_attributed_with_terminals(self):
        run = FixtureRun(self.tmp.name)
        attribution = run.attribute()
        self.assertEqual([], attribution['problems'])
        self.assertIn('K-99', attribution['policy_matches'])
        terminals = attribution['policy_matches']['K-99']['terminals']
        self.assertEqual(['ok'], list(terminals))
        self.assertAlmostEqual(12500.0, terminals['ok']['exec_time_ms'])

    def test_fake_pass_without_run_command_rejected(self):
        run = FixtureRun(self.tmp.name, with_run_command=False)
        self.assertTrue(any('no real execution artifact' in p for p in run.attribute()['problems']))

    def test_pass_without_terminal_events_rejected(self):
        run = FixtureRun(self.tmp.name, omit_terminals=True)
        self.assertTrue(any('terminal ok event' in p for p in run.attribute()['problems']))

    def test_pass_with_zero_executed_rejected(self):
        run = FixtureRun(self.tmp.name, executed=0)
        self.assertTrue(any('fake pass' in p for p in run.attribute()['problems']))

    def test_failed_outcome_surfaces(self):
        run = FixtureRun(self.tmp.name, outcome='FAIL',
                         events=[event('ace-syntax-artifacts::conformance$syntax_budget::performance::fixture_atom', 'failed')])
        self.assertTrue(any('driver outcome FAIL' in p for p in run.attribute()['problems']))

    def test_driver_errors_surface(self):
        run = FixtureRun(self.tmp.name)
        run.summary['results'][0]['errors'] = ['missing actual test terminals: x']
        (run.dir / 'results.json').write_text(json.dumps(run.summary), encoding='utf-8')
        self.assertTrue(any('driver errors' in p for p in run.attribute()['problems']))


class CancellationAndResourceTests(unittest.TestCase):

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_timeout_cancel_is_red_not_green(self):
        run = FixtureRun(self.tmp.name, run_exit=124, run_error='deadline exceeded',
                         events=[event('ace-syntax-artifacts::conformance$syntax_budget::performance::fixture_atom', 'ok')])
        problems = run.attribute()['problems']
        self.assertTrue(any('canceled/timed-out' in p for p in problems))

    def test_nonzero_run_exit_surfaces_via_driver_fail(self):
        run = FixtureRun(self.tmp.name, outcome='FAIL', run_exit=3,
                         events=[event('ace-syntax-artifacts::conformance$syntax_budget::performance::fixture_atom', 'failed')])
        self.assertTrue(run.attribute()['problems'])

    def test_tool_failure_kept_in_phase_exits(self):
        run = FixtureRun(self.tmp.name, tool_walls=(500.0,))
        index = run.dir / '0' / 'tools'
        (index / 'tool-0.command.json').write_text(
            json.dumps(command_record('tool-0', exit_code=3, wall_ms=500.0)), encoding='utf-8')
        attribution = run.attribute()
        self.assertIn(3, attribution['phases']['tool_preparation']['exits'])

    def test_report_refuses_to_overwrite(self):
        run = FixtureRun(self.tmp.name)
        target = Path(self.tmp.name) / 'report.md'
        target.write_text('previous report', encoding='utf-8')
        with self.assertRaises(SystemExit):
            report.main(['--run', str(run.dir), '--out', str(target),
                         '--policy', str(self.write_policy())])
        self.assertEqual('previous report', target.read_text(encoding='utf-8'))

    def write_policy(self):
        path = Path(self.tmp.name) / 'policy.json'
        path.write_text(json.dumps(POLICY), encoding='utf-8')
        return path


class PhaseSensitivityTests(unittest.TestCase):

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_delay_lands_in_preparation_not_work(self):
        base = FixtureRun(self.tmp.name, discovery_wall=8000.0, run_wall=20000.0).attribute()
        slow = FixtureRun(self.tmp.name + '-slow', discovery_wall=20000.0, run_wall=20000.0).attribute()
        self.assertAlmostEqual(base['phases']['preparation_build_discovery']['wall_ms'], 8000.0)
        self.assertAlmostEqual(slow['phases']['preparation_build_discovery']['wall_ms'], 20000.0)
        self.assertAlmostEqual(base['phases']['requested_work']['wall_ms'],
                               slow['phases']['requested_work']['wall_ms'])
        total_base = sum(p['wall_ms'] for p in base['phases'].values() if p.get('wall_ms') is not None)
        total_slow = sum(p['wall_ms'] for p in slow['phases'].values() if p.get('wall_ms') is not None)
        self.assertAlmostEqual(total_base + 12000.0, total_slow)

    def test_delay_lands_in_work_not_preparation(self):
        base = FixtureRun(self.tmp.name, run_wall=20000.0).attribute()
        slow = FixtureRun(self.tmp.name + '-slow', run_wall=50000.0).attribute()
        self.assertAlmostEqual(slow['phases']['requested_work']['wall_ms'], 50000.0)
        self.assertAlmostEqual(base['phases']['preparation_build_discovery']['wall_ms'],
                               slow['phases']['preparation_build_discovery']['wall_ms'])

    def test_peaks_are_maxed_never_summed(self):
        run = FixtureRun(self.tmp.name, run_wall=1000.0).attribute()
        run2 = FixtureRun(self.tmp.name + '-2', discovery_wall=1000.0).attribute()
        self.assertEqual(run['phases']['requested_work']['peak_job_bytes'], 1024)
        self.assertEqual(run2['phases']['preparation_build_discovery']['peak_job_bytes'], 1024)

    def test_intra_run_phases_are_unobserved_not_zero(self):
        attribution = FixtureRun(self.tmp.name).attribute()
        for phase in ('parent_side_verdict', 'persist_gc', 'copy', 'cleanup'):
            self.assertEqual('UNOBSERVED', attribution['phases'][phase]['status'])
            self.assertNotEqual(0, attribution['phases'][phase].get('wall_ms'))
            self.assertIn('boundary', attribution['phases'][phase])

    def test_missing_run_phase_is_unobserved(self):
        run = FixtureRun(self.tmp.name, with_run_command=False, outcome='FAIL', events=[])
        attribution = run.attribute()
        self.assertEqual('UNOBSERVED', attribution['phases']['requested_work']['status'])


class StalenessTests(unittest.TestCase):

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def test_stale_source_state_marked(self):
        run = FixtureRun(self.tmp.name)
        # repo root points at this test file's repo; a synthetic summary cannot
        # match a live tree state, so a mismatch must be reported, not hidden.
        flags = report.staleness(run.summary, Path(__file__).resolve().parents[2])
        self.assertTrue(any('source_state differs' in f for f in flags))


# --------------------------------------------------------------------------- #
# PLAN-1060 NC-01..10: ordinary-module budget mode (read-only, never green by guess)
# --------------------------------------------------------------------------- #

REPO_ROOT = Path(__file__).resolve().parents[2]
FIXED = report.fixed_tests_module(REPO_ROOT)


def frozen_invocation(group='ace-syntax'):
    _, groups = FIXED.configuration()
    selected = FIXED.select([group], {'default': [group], 'sets': {}}, groups)
    return FIXED.invocations(selected)[0]


def run_identity():
    return {'schema': 'fixed-test-run/1',
            'config_sha256': hashlib.sha256((REPO_ROOT / 'scripts/test-groups.toml').read_bytes()).hexdigest(),
            'source_state': FIXED.checkout_state()}


REGULAR_BINARY = 'ace-syntax-artifacts::conformance'
REGULAR_TEST = 'regular_probe::ordinary_case'
REGULAR_NATIVE = REGULAR_BINARY + '$' + REGULAR_TEST


class RegularRun:
    """A synthetic single-module `cargo t <module> --out` directory."""

    def __init__(self, root, *, module='ace-syntax', outcome='PASS', executed=1,
                 selected=None, discovery_ms=8000.0, run_ms=20000.0, overhead_ms=2000.0,
                 events=None, run_exit=0, run_error=None, results_rows=None,
                 extra_result_dir=False, omit_run_stdout=False, expression=None,
                 source_state=None, config_sha256=None, groups=None,
                 omit_result_json=False, checks=None, test_name=None):
        invocation = frozen_invocation(module)
        identity = run_identity()
        self.dir = Path(root) / 'run'
        index = self.dir / '0'
        index.mkdir(parents=True)
        selected_rows = selected if selected is not None else [
            {'binary': REGULAR_BINARY, 'test': REGULAR_TEST, 'ignored': False}]
        result = {'outcome': outcome, 'executed': executed,
                  'groups': groups or [module], 'selected': selected_rows, 'errors': []}
        if checks is not None:
            result['checks'] = checks
            result.pop('selected')
        summary = {**identity, 'selected_groups': groups or [module],
                   'invocations': [{'groups': groups or [module],
                                    'expression': expression or invocation['expression'],
                                    'runner': invocation['runner'],
                                    'workspace': invocation['workspace'],
                                    'args': invocation['args']}],
                   'results': results_rows if results_rows is not None else [result],
                   'wall_seconds': (discovery_ms + run_ms + overhead_ms) / 1000.0}
        if test_name is not None:
            summary['test_name'] = test_name
        if source_state is not None:
            summary['source_state'] = source_state
        if config_sha256 is not None:
            summary['config_sha256'] = config_sha256
        (index / 'discovery.command.json').write_text(
            json.dumps(command_record('discovery', wall_ms=discovery_ms)), encoding='utf-8')
        (index / 'run.command.json').write_text(
            json.dumps(command_record('run', exit_code=run_exit, wall_ms=run_ms, error=run_error)),
            encoding='utf-8')
        lines = [json.dumps(row) for row in (events if events is not None
                                            else [event(REGULAR_NATIVE, 'ok', 12.5)])]
        if not omit_run_stdout:
            (index / 'run.stdout').write_text('\n'.join(lines) + '\n', encoding='utf-8')
        if not omit_result_json:
            (index / 'result.json').write_text(json.dumps(result), encoding='utf-8')
        if extra_result_dir:
            extra = self.dir / '1'
            extra.mkdir()
            (extra / 'result.json').write_text(json.dumps(result), encoding='utf-8')
        (self.dir / 'results.json').write_text(json.dumps(summary), encoding='utf-8')

    def judge(self, extra_args=(), with_json=True):
        out = Path(str(self.dir) + '-report.md')
        json_out = Path(str(self.dir) + '-report.json')
        argv = ['--run', str(self.dir), '--mode', 'regular', '--repo', str(REPO_ROOT),
                '--out', str(out)]
        if with_json:
            argv += ['--json-out', str(json_out)]
        code = report.main(argv + list(extra_args))
        document = json.loads(json_out.read_text(encoding='utf-8')) if with_json else None
        return code, document, out.read_text(encoding='utf-8')

    def module_record(self, extra_args=()):
        code, document, markdown = self.judge(extra_args)
        return code, document['modules'][0], markdown


class RegularBudgetTests(unittest.TestCase):
    """NC-01/NC-02: the budget boundary is the complete warm invocation wall."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def budget_run(self, name, total_ms):
        return RegularRun(Path(self.tmp.name) / name, discovery_ms=total_ms - 20000.0,
                          run_ms=20000.0, overhead_ms=0.0)

    def test_nc01_boundaries_green_within_and_over(self):
        green = self.budget_run('green', 60000.0)
        code, record, _ = green.module_record()
        self.assertEqual(('GREEN', 0), (record['status'], code))
        within = self.budget_run('within', 120000.0)
        code, record, _ = within.module_record()
        self.assertEqual(('WITHIN_BUDGET', 0), (record['status'], code))
        over = self.budget_run('over', 120001.0)
        code, record, _ = over.module_record()
        self.assertEqual(('OVER_BUDGET', 1), (record['status'], code))

    def test_nc01_boundary_is_not_rounded_green(self):
        just_over = self.budget_run('just-over', 120000.4)
        code, record, _ = just_over.module_record()
        self.assertEqual(1, code)
        self.assertEqual('OVER_BUDGET', record['status'])

    def test_nc02_warm_preparation_is_inside_the_window(self):
        # 70s warm preparation + 40s run + 20s copy/cleanup = 130s, although the
        # requested_work command itself only spent 40s.
        slow = RegularRun(Path(self.tmp.name) / 'warm-prep', discovery_ms=70000.0,
                          run_ms=40000.0, overhead_ms=20000.0)
        code, record, markdown = slow.module_record()
        self.assertEqual(1, code)
        self.assertEqual('OVER_BUDGET', record['status'])
        self.assertAlmostEqual(130.0, record['warm_wall_seconds'])
        self.assertAlmostEqual(40000.0, record['phases']['requested_work']['wall_ms'])
        self.assertEqual('UNOBSERVED', record['phases']['copy']['status'])
        self.assertIn('T_warm: 130.000 s', markdown)

    def test_nc03_cold_preparation_is_listed_not_hidden(self):
        cold = RegularRun(Path(self.tmp.name) / 'cold', discovery_ms=980000.0, run_ms=20000.0,
                          overhead_ms=0.0)
        warm = RegularRun(Path(self.tmp.name) / 'warm', discovery_ms=70000.0, run_ms=20000.0,
                          overhead_ms=0.0)
        code, record, markdown = warm.module_record(
            ['--cold-run', str(cold.dir)])
        self.assertEqual(0, code)
        self.assertEqual('WITHIN_BUDGET', record['status'])
        self.assertAlmostEqual(90.0, record['warm_wall_seconds'])
        self.assertAlmostEqual(1000.0, record['cold']['warm_wall_seconds'])
        self.assertAlmostEqual(1090.0, record['first_total_seconds'])
        self.assertIn('T_cold: 1000.0', markdown)

    def test_nc03_cold_build_inside_the_test_body_stays_in_the_budget(self):
        # A test body that clears its artifacts and rebuilds spends that time
        # inside T_warm; it cannot be moved out as "cold preparation".
        body_cold = RegularRun(Path(self.tmp.name) / 'body-cold', discovery_ms=10000.0,
                               run_ms=120000.0, overhead_ms=0.0)
        code, record, _ = body_cold.module_record()
        self.assertEqual(1, code)
        self.assertEqual('OVER_BUDGET', record['status'])


class RegularEvidenceTests(unittest.TestCase):
    """NC-04..NC-06: incomplete, mispaired, duplicated or stale evidence is exit 2."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)

    def make(self, name, **kwargs):
        return RegularRun(Path(self.tmp.name) / name, **kwargs)

    def test_nc04_missing_terminal_is_exit_two(self):
        selected = [{'binary': REGULAR_BINARY, 'test': REGULAR_TEST, 'ignored': False},
                    {'binary': REGULAR_BINARY, 'test': 'regular_probe::second_case', 'ignored': False}]
        run = self.make('missing', executed=2, selected=selected)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertEqual('UNVERIFIABLE', record['status'])
        self.assertTrue(any('missing actual terminal' in item for item in record['missing_evidence']))

    def test_nc04_duplicate_native_identity_is_exit_two(self):
        run = self.make('duplicate', events=[event(REGULAR_NATIVE, 'ok', 1.0),
                                             event(REGULAR_NATIVE, 'ok', 2.0)])
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('duplicate terminal' in item for item in record['missing_evidence']))

    def test_nc04_test_name_subset_cannot_certify_a_module(self):
        frozen = frozen_invocation()
        run = self.make('subset', expression=frozen['expression'] + ' and test(=regular_probe::ordinary_case)')
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('differs from the frozen group expression' in item
                            for item in record['missing_evidence']))
        flagged = self.make('subset-flag', test_name='regular_probe::ordinary_case')
        code, record, _ = flagged.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('--test-name' in item for item in record['missing_evidence']))

    def test_nc05_extra_results_row_is_exit_two(self):
        run = self.make('rows')
        summary = json.loads((run.dir / 'results.json').read_text(encoding='utf-8'))
        summary['results'] = summary['results'] + summary['results']
        (run.dir / 'results.json').write_text(json.dumps(summary), encoding='utf-8')
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('numeric run directories' in item for item in record['missing_evidence']))

    def test_nc05_extra_numeric_directory_is_exit_two(self):
        run = self.make('dirs', extra_result_dir=True)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('numeric run directories' in item for item in record['missing_evidence']))

    def test_nc05_missing_result_json_in_index_is_exit_two(self):
        run = self.make('no-result', omit_result_json=True)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('no result.json' in item for item in record['missing_evidence']))

    def test_nc06_stale_source_state_is_exit_two(self):
        run = self.make('stale', source_state='0' * 64)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(record['stale'])
        self.assertTrue(any('STALE' in item or 'source_state differs' in item
                            for item in record['missing_evidence']))

    def test_nc06_stale_config_hash_is_exit_two(self):
        run = self.make('stale-config', config_sha256='f' * 64)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('test-groups.toml changed' in item for item in record['missing_evidence']))

    def test_nc06_missing_actual_terminals_is_exit_two(self):
        run = self.make('no-stdout', omit_run_stdout=True)
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('missing actual terminal' in item for item in record['missing_evidence']))

    def test_nc08_canceled_run_is_exit_one(self):
        run = self.make('canceled', run_exit=124, run_error='deadline exceeded')
        code, record, _ = run.module_record()
        self.assertEqual(1, code)
        self.assertEqual('FAIL', record['status'])

    def test_nc08_failed_terminal_is_exit_one(self):
        run = self.make('failed', outcome='FAIL', events=[event(REGULAR_NATIVE, 'failed')])
        code, record, _ = run.module_record()
        self.assertEqual(1, code)
        self.assertEqual('FAIL', record['status'])

    def test_nc08_not_applicable_is_not_a_module_pass(self):
        run = self.make('na', outcome='NOT_APPLICABLE', executed=0, selected=[], events=[])
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertEqual('UNVERIFIABLE', record['status'])

    def test_ignored_terminals_outside_the_selection_are_legitimate_skips(self):
        # A real nextest run emits `ignored` terminals for #[ignore] tests that
        # the expression does not select (observed for the excluded PERF legs).
        run = self.make('ignored-outside', events=[
            event(REGULAR_NATIVE, 'ok', 2.0),
            event('ace-syntax-artifacts::conformance$other::performance::ignored_campaign', 'ignored')])
        code, record, _ = run.module_record()
        self.assertEqual(0, code)
        self.assertEqual('GREEN', record['status'])
        self.assertEqual(1, len(record['ignored_outside_selection']))

    def test_selected_test_that_was_ignored_is_exit_two(self):
        run = self.make('selected-ignored', events=[event(REGULAR_NATIVE, 'ignored')])
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('skipped by the runner' in item for item in record['missing_evidence']))

    def test_case_control_override_keeps_an_expected_fail_green(self):
        # A delivered suite carries a one-check meta-oracle whose expected status
        # is FAIL; the report must re-derive that instead of calling it red.
        checks = [{'case': 'ac.src.wrong-stage-build-red', 'check': 'bld-red', 'status': 'FAIL'},
                  {'case': 'ac.legal', 'check': 'run', 'status': 'PASS'}]
        run = self.make('case-control', checks=checks, executed=2, outcome='PASS')
        code, record, _ = run.module_record()
        self.assertEqual(0, code)
        self.assertEqual('GREEN', record['status'])
        self.assertEqual(2, record['selection']['selected'])

    def test_case_check_that_breaks_its_control_is_exit_one(self):
        checks = [{'case': 'ac.src.wrong-stage-build-red', 'check': 'bld-red', 'status': 'PASS'}]
        run = self.make('case-control-missed', checks=checks, executed=1, outcome='PASS')
        code, record, _ = run.module_record()
        self.assertEqual(1, code)
        self.assertTrue(any('differ from the original control expectation' in item
                            for item in record['problems']))

    def test_nc09_two_modules_in_one_invocation_is_exit_two(self):
        run = self.make('two-modules', groups=['ace-syntax', 'ace-native'])
        code, record, _ = run.module_record()
        self.assertEqual(2, code)
        self.assertTrue(any('exactly one fixed group' in item for item in record['missing_evidence']))

    def test_regression_real_nextest_identity_is_mapped_from_discovery(self):
        # Real nextest records `binary` as the binary id while the terminal event
        # uses <package>::<binary-name>: the planner must map it, not read the
        # module as having no terminals (observed on a real ace-a2c-abi run).
        run = self.make('identity-map', selected=[
            {'binary': 'ace-a2x-c', 'test': 'abi_scalar::tests::lane_restriction_enforced',
             'ignored': False}],
            events=[event('ace-a2x-c::ace_a2x_c$abi_scalar::tests::lane_restriction_enforced', 'ok', 1.5)])
        index = run.dir / '0'
        (index / 'discovery.stdout').write_text(json.dumps({'rust-suites': {
            'ace-a2x-c': {'package-name': 'ace-a2x-c', 'binary-name': 'ace_a2x_c',
                          'status': 'listed'}}}), encoding='utf-8')
        code, record, _ = run.module_record()
        self.assertEqual(0, code)
        self.assertEqual('GREEN', record['status'])
        self.assertEqual([], record['missing_evidence'])


class RegularAcceptanceTests(unittest.TestCase):
    """NC-07/NC-10: ordinary tests are not the frozen policy, and the tool is read-only."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.policy = Path(self.tmp.name) / 'policy.json'
        self.policy.write_text(json.dumps(POLICY), encoding='utf-8')

    def test_nc07_regular_accepts_non_policy_tests_policy_mode_rejects(self):
        run = RegularRun(Path(self.tmp.name) / 'accept')
        code, record, _ = run.module_record()
        self.assertEqual(0, code)
        self.assertEqual('GREEN', record['status'])
        attribution = report.attribute_run(run.dir, report.load_policy_items(POLICY))
        self.assertTrue(any('not covered by policy' in problem for problem in attribution['problems']))

    def test_nc07_default_mode_is_unchanged(self):
        run = RegularRun(Path(self.tmp.name) / 'default-mode')
        out = Path(self.tmp.name) / 'policy-report.md'
        code = report.main(['--run', str(run.dir), '--out', str(out), '--policy', str(self.policy),
                            '--repo', str(REPO_ROOT)])
        self.assertEqual(0, code)
        self.assertIn('ACE cost attribution', out.read_text(encoding='utf-8'))

    def test_nc10_regular_mode_starts_no_build_or_runner(self):
        run = RegularRun(Path(self.tmp.name) / 'readonly')
        spawned = []
        original_check_output = subprocess.check_output
        original_run = subprocess.run
        original_popen = subprocess.Popen

        def recorder(real):
            def wrapped(argv, *args, **kwargs):
                spawned.append(argv if isinstance(argv, (list, tuple)) else [argv])
                return real(argv, *args, **kwargs)
            return wrapped

        subprocess.check_output = recorder(original_check_output)
        subprocess.run = recorder(original_run)
        subprocess.Popen = recorder(original_popen)
        try:
            code, _, _ = run.judge()
        finally:
            subprocess.check_output = original_check_output
            subprocess.run = original_run
            subprocess.Popen = original_popen
        self.assertEqual(0, code)
        programs = {Path(str(argv[0])).name for argv in spawned}
        self.assertTrue(programs <= {'git'}, 'unexpected subprocess(es): %s' % programs)

    def test_nc10_refuses_to_overwrite_and_leaves_sources_intact(self):
        run = RegularRun(Path(self.tmp.name) / 'no-overwrite')

        def digest():
            return {path.relative_to(run.dir).as_posix():
                    hashlib.sha256(path.read_bytes()).hexdigest()
                    for path in sorted(run.dir.rglob('*')) if path.is_file()}

        before = digest()
        target = Path(self.tmp.name) / 'existing.md'
        target.write_text('previous report', encoding='utf-8')
        with self.assertRaises(SystemExit):
            report.main(['--run', str(run.dir), '--mode', 'regular', '--out', str(target),
                         '--repo', str(REPO_ROOT)])
        self.assertEqual('previous report', target.read_text(encoding='utf-8'))
        self.assertEqual(before, digest())


if __name__ == '__main__':
    unittest.main()
