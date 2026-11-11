"""Pure-Python pilot round/interference accounting checks (no ACE worker needed).

PLAN-1014 T-04: the real CLI checks moved to test_case_pilot_ace.py, wired to the
explicit ACE pilot group with source-bound tool preparation (case-parent/batch-av/
av-regular). This legacy group keeps only the worker-free governance checks."""
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class PilotRoundAccounting(unittest.TestCase):
    def test_idle_service_reference_bounds_cpu_and_never_exempts_builds_or_tests(self):
        import copy,sys
        sys.path.insert(0,str(ROOT/'scripts/test-governance'))
        from external_workloads import classify
        snapshot={'observed_monotonic_seconds':10,'processes':[
            {'pid':1,'parent_pid':0,'name':'cargo.exe','created':'one','cpu':0.1,
             'command':'"cargo" run --release --manifest-path D:/fixture/example-back/Cargo.toml'},
            {'pid':2,'parent_pid':1,'name':'example-back.exe','created':'two','cpu':0.2,'command':None}],
            'listeners':[{'pid':2,'address':'127.0.0.1','port':17837}]}
        self.assertTrue(classify(snapshot)[0], 'one observation never proves an idle service')
        previous=copy.deepcopy(snapshot)
        previous['observed_monotonic_seconds']=0
        busy,services=classify(snapshot,previous)
        self.assertFalse(busy)
        self.assertEqual(services[0]['backend_pid'],2)
        maintenance=copy.deepcopy(snapshot)
        maintenance['processes'][1]['cpu']+=1/64
        self.assertFalse(classify(maintenance,previous)[0], 'one accounting tick is not an active build/test')
        self.assertLessEqual(classify(maintenance,previous)[1][0]['cpu_delta_seconds'],0.1)
        for axis in ('cpu','pid-reuse','unknown-child','test','relative','not-listening','remote','missing-cpu','compiler'):
            changed=copy.deepcopy(snapshot)
            if axis=='cpu':changed['processes'][1]['cpu']+=0.5
            elif axis=='pid-reuse':changed['processes'][1]['created']='reused'
            elif axis=='unknown-child':changed['processes'][1]['name']='unknown.exe'
            elif axis=='test':changed['processes'][0]['command']='cargo test --test run --manifest-path D:/fixture/example-back/Cargo.toml'
            elif axis=='relative':changed['processes'][0]['command']='cargo run --manifest-path example-back/Cargo.toml'
            elif axis=='not-listening':changed['listeners']=[]
            elif axis=='remote':changed['listeners'][0]['address']='0.0.0.0'
            elif axis=='missing-cpu':changed['processes'][1]['cpu']=None
            else:changed['processes'].append({'pid':3,'parent_pid':1,'name':'rustc.exe','created':'three','cpu':0,'command':None})
            self.assertTrue(classify(changed,previous)[0],axis)
        for elapsed in (0,-1,91):
            changed=copy.deepcopy(snapshot);changed['observed_monotonic_seconds']=elapsed
            self.assertTrue(classify(changed,previous)[0],elapsed)
        combined=copy.deepcopy(snapshot)
        other=copy.deepcopy(snapshot['processes'])
        other[0].update(pid=3,command='cargo run --manifest-path D:/fixture/other-back/Cargo.toml')
        other[1].update(pid=4,parent_pid=3,name='other-back.exe')
        combined['processes']+=other
        combined['listeners'].append({'pid':4,'address':'127.0.0.1','port':17838})
        prior=copy.deepcopy(combined);prior['observed_monotonic_seconds']=0
        combined['processes'][1]['cpu']+=0.06
        combined['processes'][3]['cpu']+=0.06
        busy,services=classify(combined,prior)
        self.assertTrue(any('aggregate' in r['reason'] for r in busy))
        self.assertFalse(services)
        wrapped=copy.deepcopy(snapshot)
        wrapped['processes'][1]['parent_pid']=3
        wrapper=copy.deepcopy(snapshot['processes'][0]);wrapper.update(pid=3,parent_pid=1,created='three')
        wrapped['processes'].append(wrapper)
        prior=copy.deepcopy(wrapped);prior['observed_monotonic_seconds']=0
        wrapped['processes'][1]['cpu']+=0.08
        self.assertFalse(classify(wrapped,prior)[0], 'Cargo proxy and child must not double-count one backend CPU')

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
