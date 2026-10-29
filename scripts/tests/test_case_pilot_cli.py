"""Real prebuilt pilot CLI checks; never builds or silently substitutes a mock."""
import json
import os
from pathlib import Path
import subprocess
import unittest

ROOT = Path(__file__).resolve().parents[2]
EXE = ROOT / 'target/debug/test-case-pilot.exe'
WORKER = Path(os.environ.get('AC_AV_CASE_WORKER',
              ROOT / 'experimental/ac-av-probe/target/debug/av_probe_worker.exe'))


class PilotRoundAccounting(unittest.TestCase):
    def test_interference_discards_entire_round_and_missing_cohort_never_counts(self):
        import sys
        sys.path.insert(0,str(ROOT/'scripts/test-governance'))
        from measure_pilot import settle_round,comparisons
        cohorts=[{'id':'a'},{'id':'b'}]
        prior=[{'valid':True},{'valid':True}]
        self.assertFalse(settle_round(prior,cohorts,[{'Id':123}]))
        self.assertTrue(all(s['round_accepted'] is False for s in prior))
        partial=[{'valid':True}]
        self.assertFalse(settle_round(partial,cohorts,[]))
        final=[{'valid':True},{'valid':True}]
        self.assertTrue(settle_round(final,cohorts,[]))
        self.assertTrue(all(s['round_accepted'] for s in final))
        # Fast discarded samples must not fill the required formal denominator.
        contract={'parallelism':[1],'rounds':10,'cohorts':[{'id':'b','set':'pilot15','mode':'group'}]}
        samples=[dict(series='measurement',valid=True,round_accepted=False,parallelism=1,cohort=c)
                 for c in ('a15','b') for _ in range(10)]
        self.assertEqual(comparisons(samples,contract),[])


@unittest.skipUnless(os.name == 'nt' and EXE.is_file() and WORKER.is_file(),
                     'requires Windows and the explicitly built test-case-batch pilot/worker')
class PilotCLI(unittest.TestCase):
    def test_formal_measurement_validator_rejects_missing_spawns_calls_and_checks(self):
        import sys,copy
        sys.path.insert(0,str(ROOT/'scripts/test-governance'))
        from measure_pilot import validate_report
        from module_catalog import load
        from native_meter import measure
        contract=load(ROOT/'scripts/test-governance/measurement-contract.json')
        cohort=next(c for c in contract['cohorts'] if c['id']=='a-d3')
        meter=measure([str(EXE),'--mode','independent','--set','d3','--repeat','3','--parallelism','2'],
                      cwd=ROOT,env=dict(os.environ,AC_AV_CASE_WORKER=str(WORKER)),timeout=65)
        report=json.loads(meter['stdout'])
        observed=validate_report(report,cohort,2,contract,meter)
        self.assertEqual(observed['actual_spawn_events'],9)
        self.assertEqual(observed['compile_calls'],9)
        for mutation in ('spawn','compile','case','status'):
            bad=copy.deepcopy(report)
            if mutation=='spawn':bad['groups'][0]['parent_observations']['worker_pids']=[]
            if mutation=='compile':bad['groups'][0]['items'][0]['receipt']['step']['counters']['frontend_compile_calls']=0
            if mutation=='case':bad['groups'][0]['items']=[]
            if mutation=='status':bad['groups'][0]['items'][0]['status']='ERROR'
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):validate_report(bad,cohort,2,contract,meter)

    def test_default_off_keeps_original_case_path_and_rejects_batch_without_fallback(self):
        regular=ROOT/'target/test-tools/regular/release/av_probe_worker.exe'
        self.assertTrue(regular.is_file(),'source-declared default-off worker preparation required')
        env=dict(os.environ,AC_AV_CASE_WORKER=str(regular))
        original=subprocess.run([str(EXE),'--mode','independent','--set','d3'],cwd=ROOT,env=env,capture_output=True,text=True,timeout=65)
        self.assertEqual(original.returncode,0,original.stderr)
        self.assertEqual([r['status'] for g in json.loads(original.stdout)['groups'] for r in g['items']],['PASS']*3)
        rejected=subprocess.run([str(EXE),'--mode','group','--group-size','3','--set','d3'],cwd=ROOT,env=env,capture_output=True,text=True,timeout=65)
        self.assertEqual(rejected.returncode,2,rejected.stdout+rejected.stderr)
        groups=json.loads(rejected.stdout)['groups']
        self.assertTrue(groups[0]['error'])
        self.assertTrue(all(r['status']=='NOT_RUN' for g in groups for r in g['items']))

    def test_independent_reports_actual_spawn_and_disjoint_parent_phases(self):
        result=self.call('--mode','independent','--set','d3','--parallelism','2')
        self.assertEqual(result.returncode,0,result.stderr)
        report=json.loads(result.stdout)
        self.assertEqual(len(report['groups']),3)
        self.assertGreater(report['parent_pid'],0)
        self.assertGreater(report['input_load_ms'],0)
        for group in report['groups']:
            observations=group['parent_observations']
            self.assertEqual(observations['worker_pids'],[group['worker_pid']])
            self.assertNotEqual(group['worker_pid'],report['parent_pid'])
            self.assertEqual(group['worker_starts'],len(observations['worker_pids']))
            phases=observations['phases_ms']
            self.assertEqual(set(phases),{'prepare_ms','transport_ms','admission_compare_receipt_ms','cleanup_ms'})
            self.assertTrue(all(value>0 for value in phases.values()))
            self.assertLessEqual(sum(phases.values()),group['wall_ms']+1)

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

    def test_combined_is_separate_d3_observation_with_real_counts_and_replay(self):
        result = self.call('--mode', 'combined', '--repeat', '3', '--parallelism', '2')
        self.assertEqual(result.returncode, 0, result.stderr)
        report = json.loads(result.stdout)
        self.assertEqual(report['set'], 'd3')
        self.assertEqual(len(report['groups']), 3)
        self.assertEqual(len({g['worker_pid'] for g in report['groups']}), 3)
        for group in report['groups']:
            self.assertEqual(group['observation']['metrics']['calls']['compile'], 1)
            self.assertEqual(group['observation']['metrics']['calls']['vm'], 1)
            self.assertEqual([r['actual'] for r in group['items']], [3, -6, 9])
        row = report['groups'][0]['items'][0]
        replay = subprocess.run(row['replay_argv'], cwd=ROOT, timeout=65,
                                capture_output=True, text=True,
                                env=dict(os.environ, AC_AV_CASE_WORKER=str(WORKER)))
        self.assertEqual(replay.returncode, 0, replay.stderr)
        self.assertEqual(sum(len(g['items']) for g in json.loads(replay.stdout)['groups']), 1)

    def test_pilot_requires_explicit_prebuilt_worker(self):
        env = dict(os.environ)
        env.pop('AC_AV_CASE_WORKER', None)
        result = subprocess.run([str(EXE), '--mode', 'group'], cwd=ROOT,
                                capture_output=True, text=True, timeout=5, env=env)
        self.assertEqual(result.returncode, 2)
        self.assertIn('prebuilt', result.stderr)


if __name__ == '__main__':
    unittest.main()
