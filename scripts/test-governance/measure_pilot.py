#!/usr/bin/env python3
"""Frozen, interleaved A/B/C measurements; every attempted outcome is retained."""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import platform
import random
import statistics
import subprocess
import sys
import time

from governance_io import ROOT,load,source_identity,write
from native_meter import measure

PHASES={'prepare_ms','transport_ms','admission_compare_receipt_ms','cleanup_ms'}


def sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def validate_report(report,cohort,parallelism,contract,meter):
    if (report.get('schema')!='test-case-pilot/1' or report.get('exit')!=0
        or report.get('mode')!=cohort['mode'] or report.get('set')!=cohort['set']
        or report.get('parallelism')!=parallelism or report.get('repeat')!=contract['invocation_repeats']
        or report.get('group_size')!=cohort['group_size']):
        raise ValueError('pilot workload/configuration differs from frozen cohort')
    groups=report['groups'];pids=[];rows=[];compile_calls=0;vm_calls=0;spans=[]
    phases={key:0.0 for key in PHASES}
    if not report.get('parent_pid') or report.get('input_load_ms',-1)<0:raise ValueError('missing actual parent/loading observation')
    for group in groups:
        if group.get('error') or group.get('cleanup_complete') is False:raise ValueError('group infrastructure/cleanup failure')
        observed=group['parent_observations'];actual=observed['worker_pids']
        if (len(actual)!=1 or group['worker_starts']!=len(actual) or actual[0]!=group['worker_pid']
            or actual[0]<=0 or actual[0]==report['parent_pid']):raise ValueError('actual successful spawn identity/count mismatch')
        pids.extend(actual)
        if set(observed['phases_ms'])!=PHASES:raise ValueError('missing parent phase')
        for key,value in observed['phases_ms'].items():
            if not isinstance(value,(int,float)) or not math.isfinite(value) or value<0:raise ValueError('invalid parent phase duration')
            phases[key]+=value
        if sum(observed['phases_ms'].values())>group['wall_ms']+2:raise ValueError('parent phases overlap/exceed group wall')
        for row in group['items']:
            if row['status']!='PASS' or row['actual']!=row['expected']:raise ValueError('original parent oracle did not pass')
            if not row.get('origin') or not row.get('replay_argv'):raise ValueError('missing original origin/replay')
            rows.append(row)
            if cohort['mode']!='combined':
                receipt=row['receipt'];step=receipt['step']
                if step['counters'].get('frontend_compile_calls')!=1 or step['counters'].get('av_execute')!=1:
                    raise ValueError('original scalar producer counts missing/not fresh per item')
                compile_calls+=step['counters']['frontend_compile_calls'];vm_calls+=step['counters']['av_execute']
                if not step['spans']:raise ValueError('missing actual worker spans')
                spans.append(step['spans'])
        if cohort['mode']=='combined':
            observation=group['observation'];calls=observation['metrics']['calls']
            if calls['compile']!=1 or calls['vm']!=1 or observation['live_after_release']!=0:
                raise ValueError('combined real compile/VM/resource counts differ')
            compile_calls+=calls['compile'];vm_calls+=calls['vm'];spans.append(observation['metrics']['spans_ms'])
    if (len(rows)!=cohort['logical_checks'] or len(pids)!=cohort['worker_starts']
        or compile_calls!=cohort['compile_calls'] or vm_calls!=cohort['vm_calls']):
        raise ValueError('workload logical/actual spawn/compile/VM totals differ')
    distinct={(row['case'],row['check']) for row in rows}
    expected_distinct=15 if cohort['set']=='pilot15' else 3
    if len(distinct)!=expected_distinct:raise ValueError('lost/duplicated original input identity')
    for repeat in range(contract['invocation_repeats']):
        round_rows=[row for group in groups if group['repeat']==repeat for row in group['items']]
        if len(round_rows)!=expected_distinct or {(r['case'],r['check']) for r in round_rows}!=distinct:
            raise ValueError('repeat lost or duplicated an original check')
    if (meter['error'] or meter['exit']!=0 or meter['workload_cpu_ms']<=0
        or meter['job_processes']<len(pids)+1 or meter['peak_job_bytes']<=0):
        raise ValueError('native Job process-tree accounting/infrastructure incomplete')
    # PID reuse across sequential starts is legal: count observed start events,
    # not distinct OS PID numbers. Native totals may include runtime helpers.
    return {'actual_spawn_pids':pids,'actual_spawn_events':len(pids),'compile_calls':compile_calls,
        'vm_calls':vm_calls,'logical_checks':len(rows),'distinct_checks':sorted(distinct),
        'parent_phases_ms':phases,'input_load_ms':report['input_load_ms'],'worker_stage_spans':spans,
        'native_extra_processes':meter['job_processes']-len(pids)-1}


def percentile(values,p):
    if not values:raise ValueError('no percentile samples')
    return sorted(values)[math.ceil(p*len(values))-1]


def comparisons(samples,contract):
    result=[]
    good=[s for s in samples if s['series']=='measurement' and s['valid'] and s.get('round_accepted') is True]
    for parallelism in contract['parallelism']:
        for candidate in contract['cohorts']:
            if candidate['id'].startswith('a'):continue
            baseline='a15' if candidate['set']=='pilot15' else 'a-d3'
            base=[s for s in good if s['parallelism']==parallelism and s['cohort']==baseline]
            new=[s for s in good if s['parallelism']==parallelism and s['cohort']==candidate['id']]
            if len(base)!=contract['rounds'] or len(new)!=contract['rounds']:continue
            metrics={}
            for metric in ('wall_ms','total_cpu_ms'):
                old=[s['meter'][metric] for s in base];values=[s['meter'][metric] for s in new]
                center=statistics.median(old)
                noise=max(1.0,2*statistics.median(abs(x-center) for x in old))
                metrics[metric]={'baseline_p50':percentile(old,.5),'candidate_p50':percentile(values,.5),
                    'baseline_p90':percentile(old,.9),'candidate_p90':percentile(values,.9),'noise_ms':noise}
            wall,cpu=metrics['wall_ms'],metrics['total_cpu_ms']
            p=contract['promotion']
            benefit=any(m['candidate_p50']<=(1-p['minimum_p50_improvement'])*m['baseline_p50']
                        and m['baseline_p50']-m['candidate_p50']>m['noise_ms'] for m in (wall,cpu))
            stable=all(m['candidate_p50']<=(1+p['other_p50_max_degradation'])*m['baseline_p50'] for m in (wall,cpu))
            tail=wall['candidate_p90']<=(1+p['wall_p90_max_degradation'])*wall['baseline_p90']
            peak=max(s['meter']['peak_job_bytes'] for s in new)
            memory=peak<=2*(1<<30)  # existing repository <=2GiB machine budget; guest limits unchanged
            result.append({'cohort':candidate['id'],'parallelism':parallelism,'baseline':baseline,'metrics':metrics,
                'peak_job_bytes':peak,'memory_budget_bytes':2*(1<<30),'numeric_gate_pass':benefit and stable and tail and memory,
                'scope':'C separate owned-record observation' if candidate['mode']=='combined' else 'original scalar A/B'})
    return result


def binding(pilot,worker):
    return {'source_identity':source_identity(ROOT),'pilot_sha256':sha(pilot),'worker_sha256':sha(worker),
        'producer_sha256':sha(__file__),'native_meter_sha256':sha(Path(__file__).with_name('native_meter.py')),
        'contract_sha256':sha(Path(__file__).with_name('measurement-contract.json')),
        'external_workload_guard_sha256':sha(Path(__file__).with_name('external_workloads.py'))}


_previous_workload_snapshot = None
_background_observations = []


def busy_processes():
    from external_workloads import QUERY, classify
    global _previous_workload_snapshot
    result=subprocess.run(['powershell','-NoProfile','-Command',QUERY],capture_output=True,text=True,timeout=20)
    if result.returncode:
        raise ValueError('external workload observation failed; no sample admission')
    snapshot=json.loads(result.stdout)
    snapshot['observed_monotonic_seconds']=time.monotonic()
    busy,services=classify(snapshot,_previous_workload_snapshot)
    _previous_workload_snapshot=snapshot
    _background_observations.append({'time':time.time(),'idle_loopback_services':services,'busy':busy})
    return busy


def settle_round(attempt_samples,cohorts,interference):
    """Accept the whole predeclared round or discard all its numerical samples."""
    accepted=not interference and len(attempt_samples)==len(cohorts) and all(s['valid'] for s in attempt_samples)
    for sample in attempt_samples:sample['round_accepted']=accepted
    return accepted


def wait_for_quiet(document,path,timeout=600):
    deadline=time.monotonic()+timeout
    while True:
        busy=busy_processes()
        if not busy:return
        document.setdefault('quiet_waits',[]).append({'time':time.time(),'processes':busy})
        write(path,document)
        if time.monotonic()>=deadline:raise ValueError('external workload did not release measurement window within 600s')
        print(f'waiting for external workload: {busy}',flush=True)
        time.sleep(min(30,max(0,deadline-time.monotonic())))


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--pilot',type=Path,required=True);parser.add_argument('--worker',type=Path,required=True)
    parser.add_argument('--output',type=Path,required=True)
    parser.add_argument('--smoke',action='store_true',help='One diagnostic round; never formal completion/promotion')
    args=parser.parse_args()
    if os.name!='nt':raise ValueError('native Job measurements require Windows')
    args.pilot=args.pilot.resolve(strict=True);args.worker=args.worker.resolve(strict=True)
    args.output.mkdir(parents=True,exist_ok=True)
    if (args.output/'campaign.json').exists():raise ValueError('retain prior campaign; choose a fresh output directory')
    contract=load(Path(__file__).with_name('measurement-contract.json'))
    before=binding(args.pilot,args.worker);samples=[]
    env={**os.environ,'AC_AV_CASE_WORKER':str(args.worker)}
    document={'schema':'test-pilot-campaign/1','plan':888,'diagnostic':args.smoke,
        'machine':{'platform':platform.platform(),'logical_cpus':os.cpu_count(),'python':sys.version},
        'binding':before,'contract':contract,'samples':samples,'complete':False,
        'planned_measurements':len(contract['cohorts'])*len(contract['parallelism'])*contract['rounds'],
        'planned_warmups':len(contract['cohorts'])*len(contract['parallelism'])*contract['warmup_rounds'],
        'background_workload_observations':_background_observations}
    write(args.output/'campaign.json',document)
    spec=importlib.util.spec_from_file_location('measurement_machine_gate',ROOT/'scripts/run-full-regression.py')
    gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
    with gate.machine_gate(),gate.machine_gate('AutoLangFormalPerformance'):
        for parallelism in contract['parallelism']:
            plan=[('diagnostic',0)] if args.smoke else [('warmup',-1),*[('measurement',i) for i in range(contract['rounds'])]]
            for series,round_index in plan:
                order=list(contract['cohorts'])
                random.Random(contract['variant_order_seed']+parallelism*100+round_index).shuffle(order)
                if round_index%2:order.reverse()
                for attempt in range(1,4):
                    wait_for_quiet(document,args.output/'campaign.json')
                    attempt_samples=[];interference=[]
                    for cohort in order:
                        interference=busy_processes()
                        if interference:break
                        command=[str(args.pilot),'--mode',cohort['mode'],'--set',cohort['set'],'--group-size',str(cohort['group_size']),
                            '--repeat',str(contract['invocation_repeats']),'--parallelism',str(parallelism)]
                        meter=measure(command,cwd=ROOT,env=env,timeout=cohort['worker_starts']*60+10)
                        index=len(samples);raw=meter.pop('stdout');stderr=meter.pop('stderr')
                        (args.output/f'{index:03}-stdout.json').write_bytes(raw);(args.output/f'{index:03}-stderr.log').write_bytes(stderr)
                        sample={'series':series,'round':round_index,'attempt':attempt,'round_accepted':False,
                            'cohort':cohort['id'],'parallelism':parallelism,'meter':meter,'valid':False,
                            'stdout_sha256':hashlib.sha256(raw).hexdigest()}
                        try:
                            sample['observations']=validate_report(json.loads(raw),cohort,parallelism,contract,meter)
                            sample['valid']=True
                        except (ValueError,KeyError,TypeError) as error:sample['error']=str(error)
                        samples.append(sample);attempt_samples.append(sample)
                        write(args.output/'campaign.json',document)
                        print(f'{series} round={round_index} attempt={attempt} p={parallelism} {cohort["id"]}: valid={sample["valid"]} wall={meter["wall_ms"]:.2f}ms CPU={meter["total_cpu_ms"]:.2f}ms',flush=True)
                        # Mechanical/oracle failures always stop; interference cannot excuse them.
                        if not sample['valid']:raise ValueError('mechanical sample failure; repair before a new frozen campaign')
                        interference=busy_processes()
                        if interference:break
                    accepted=settle_round(attempt_samples,order,interference)
                    document.setdefault('round_attempts',[]).append({'series':series,'round':round_index,
                        'parallelism':parallelism,'attempt':attempt,'accepted':accepted,
                        'sample_count':len(attempt_samples),'external_workload':interference,
                        'unrun_cohorts':[c['id'] for c in order[len(attempt_samples):]]})
                    write(args.output/'campaign.json',document)
                    if accepted:break
                    print(f'whole round discarded due to observed external workload: {interference}',flush=True)
                else:raise ValueError('external interference exhausted three whole-round attempts; campaign incomplete')
        after=binding(args.pilot,args.worker)
        if after!=before:raise ValueError('candidate/input/configuration changed during campaign')
    document['binding_after']=after
    document['complete']=not args.smoke and sum(s.get('round_accepted') is True for s in samples)==document['planned_measurements']+document['planned_warmups']
    document['comparisons']=comparisons(samples,contract) if document['complete'] else []
    document['promotion']='pending all compatibility/isolation controls and parallelism-consistent decision; numeric gates alone do not promote'
    write(args.output/'campaign.json',document)
    return 0


if __name__=='__main__':
    try:sys.exit(main())
    except (ValueError,KeyError,OSError,subprocess.SubprocessError) as error:
        print(f'pilot measurement ERROR: {error}',file=sys.stderr);sys.exit(2)
