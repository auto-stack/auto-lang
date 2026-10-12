from pathlib import Path
import sys,copy,json,importlib.util,subprocess,os,hashlib
repo=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
os.environ['PYTHONUTF8']='1'
def load(name,path):
    spec=importlib.util.spec_from_file_location(name,path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module
ai=load('edge_inventory',repo/'scripts/acc_inventory.py');tm=load('edge_tests',repo/'scripts/tests/test_acc_inventory.py')
c=tm.StrictDecisionsGate();c.setUp();results={}
def cli(*flags):
    p=subprocess.run([sys.executable,'-B',str(repo/'scripts/acc_inventory.py'),'--root',str(c.repo),'--output',str(c.out),*flags],capture_output=True,text=True,encoding='utf8',timeout=20,env={**os.environ,'PYTHONUTF8':'1'})
    return {'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
try:
    assert cli('--write')['exit']==0
    base=c.complete_layer()
    assert cli('--check','--require-decisions')['exit']==0
    scans={p:ai.scan_module(p,(c.repo/p).read_text(encoding='utf8')) for p in ai.SOURCE_MODULES}
    inputs=[{'path':p,'sha256':ai.sha256_of(c.repo/p)} for p in ai.MANAGED_INPUTS]
    unknowns=ai._unknown_candidates_from_scans(scans);original=ai._evidence_file
    for name in ('valid','bound_file_exists','missing_bound_file','bound_file_restored','duplicate_bad_type_first','duplicate_bad_type_last','duplicate_valid_first','duplicate_valid_last'):
        layer=copy.deepcopy(base);d=layer['decisions'][-1]
        d['evidence']=[original(ev)+':911' for ev in d['evidence']];tags=set(d['evidence'])
        if name in ('bound_file_exists','missing_bound_file','bound_file_restored'):
            reference=c.repo/'docs/review-bound-reference.txt'
            reference.parent.mkdir(exist_ok=True)
            payload=b'Bound reference used by the approved decision\n'
            if name=='missing_bound_file':reference.unlink()
            else:reference.write_bytes(payload)
            d['bound_input_hashes']['docs/review-bound-reference.txt']=hashlib.sha256(payload).hexdigest()
        if name.startswith('duplicate_'):
            other=copy.deepcopy(d)
            if 'bad_type' in name:other['evidence']=17
            if name.endswith('first'):layer['decisions'].insert(0,other)
            else:layer['decisions'].append(other)
        path=c.out/ai.DECISIONS_NAME;path.write_text(json.dumps(layer),encoding='utf8')
        reads=[]
        def observe(ev):
            if ev in tags:reads.append({'evidence':ev,'caller':sys._getframe(1).f_code.co_name,'line':sys._getframe(1).f_lineno})
            return original(ev)
        ai._evidence_file=observe
        try:status,msg=ai.validate_decisions(path,inputs,c.repo,unknowns,require=True)
        finally:ai._evidence_file=original
        result={'status':status,'messages':msg,'reads':reads,'consumer_reads':[x for x in reads if x['caller']=='<genexpr>'],'cli':cli('--check','--require-decisions')}
        results[name]=result
        print(name,'state',status,'cli',result['cli']['exit'],'consumer_reads',len(result['consumer_reads']),'traceback','Traceback' in result['cli']['stderr'])
finally:c.doCleanups()
(out/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf8')
