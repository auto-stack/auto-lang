"""Real prebuilt pilot CLI checks; never builds or silently substitutes a mock."""
import json
import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / 'target/debug/test-case-pilot.exe'
WORKER = ROOT / 'experimental/ac-av-probe/target/debug/av_probe_worker.exe'


@unittest.skipUnless(os.name == 'nt' and EXE.is_file() and WORKER.is_file(),
                     'requires Windows and the explicitly built test-case-batch pilot/worker')
class PilotCLI(unittest.TestCase):
    def call(self, *args):
        return subprocess.run([str(EXE), *args], cwd=ROOT, timeout=65,
                              capture_output=True, text=True,
                              env=dict(os.environ, AC_AV_CASE_WORKER=str(WORKER)))

    def test_group_tail_has_all_original_results_and_per_case_replay(self):
        process = self.call('--mode', 'group', '--group-size', '10', '--parallelism', '2')
        self.assertEqual(process.returncode, 0, process.stderr)
        report = json.loads(process.stdout)
        self.assertEqual([len(g['items']) for g in report['groups']], [10, 5])
        self.assertEqual(sum(g['worker_starts'] for g in report['groups']), 2)
        rows = [r for g in report['groups'] for r in g['items']]
        self.assertEqual(len({(r['case'], r['check']) for r in rows}), 15)
        for row in rows:
            self.assertEqual(row['status'], 'PASS')
            self.assertEqual(row['expected'], row['actual'])
            self.assertTrue(Path(row['origin']['source']).is_file())
            self.assertTrue(Path(row['origin']['oracle_document']).is_file())
            self.assertEqual(row['receipt']['step']['outcome'], 'success')
            self.assertEqual(row['receipt']['step']['counters']['av_execute'], 1)
            self.assertEqual(len(row['source_sha256']), 64)
        # Execute the report's actual argv, rather than reconstructing a filter.
        replay = subprocess.run(rows[12]['replay_argv'], cwd=ROOT, timeout=65,
                                capture_output=True, text=True,
                                env=dict(os.environ, AC_AV_CASE_WORKER=str(WORKER)))
        self.assertEqual(replay.returncode, 0, replay.stderr)
        replay_rows = [r for g in json.loads(replay.stdout)['groups'] for r in g['items']]
        self.assertEqual(len(replay_rows), 1)
        self.assertEqual(replay_rows[0]['case'], rows[12]['case'])
        self.assertEqual(replay_rows[0]['actual'], rows[12]['actual'])

    def test_unknown_zero_duplicate_and_out_of_budget_are_errors(self):
        for args in [('--case', 'not-a-case'), ('--group-size', '0'),
                     ('--parallelism', '3'), ('--repeat', '0'),
                     ('--mode', 'group', '--mode', 'independent'), ('--unknown', 'x')]:
            with self.subTest(args=args):
                result = self.call(*args)
                self.assertEqual(result.returncode, 2)
                self.assertTrue(result.stderr)


if __name__ == '__main__':
    unittest.main()
