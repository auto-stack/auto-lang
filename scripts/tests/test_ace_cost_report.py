#!/usr/bin/env python3
"""PLAN-893 V-03/V-04/V-05: read-only attribution cannot mint green verdicts.

Synthetic run fixtures (no builds, no product code) drive the same code paths
as real `cargo t --out` artifacts: verdict linkage needs real terminal events,
canceled/timeout exits stay red, phases are attributed to the bucket that
actually spent the time (and the total window responds), intra-run phases stay
UNOBSERVED instead of zero, stale runs are marked, and existing reports are
never overwritten.
"""
import json
from pathlib import Path
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


if __name__ == '__main__':
    unittest.main()
