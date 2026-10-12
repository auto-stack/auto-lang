import importlib.util, json, os, shutil, subprocess, sys, hashlib
from pathlib import Path
repo=Path(sys.argv[1]).resolve(); out=Path(sys.argv[2]).resolve(); out.mkdir(parents=True,exist_ok=True)
spec=importlib.util.spec_from_file_location('inv',repo/'scripts/acc_inventory.py'); ai=importlib.util.module_from_spec(spec); spec.loader.exec_module(ai)
def run(root,report,*args):
 p=subprocess.run([sys.executable,'-B',str(repo/'scripts/acc_inventory.py'),'--root',str(root),'--output',str(report),*args],capture_output=True,text=True,encoding='utf-8',timeout=20,env={**os.environ,'PYTHONUTF8':'1'})
 return {'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
root=out/'fixture-repo'; shutil.copytree(repo/'scripts/tests/fixtures/acc-inventory/repo',root)
report=out/'fixture-report'; results={'fixture_write':run(root,report,'--write')}
scans={p:ai.scan_module(p,(root/p).read_text(encoding='utf-8')) for p in ai.SOURCE_MODULES}; unknowns=ai._unknown_candidates_from_scans(scans)
layer={'format_version':1,'decisions':[{'id':f'MD-R-{i:03}','subject':p,'kind':'module-role','conclusion':'adapt','evidence':[p+':1'],'note':'review control','bound_input_hashes':{p:ai.sha256_of(root/p)}} for i,p in enumerate(ai.MANAGED_INPUTS)],'unknown_families':[{'path':p,'kind':k,'names':'*','disposition':'resolved','decision':'MD-R-000','note':'review control'} for p,k in sorted({(p,k) for p,k,n in unknowns})]}
def check(label,data):
 (report/ai.DECISIONS_NAME).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8'); results[label]=run(root,report,'--check','--require-decisions')
check('valid_control',layer)
import copy
(root/'review-evidence.txt').write_text('original semantic evidence',encoding='utf-8')
bad=copy.deepcopy(layer); bad['decisions'][0]['evidence'].append('review-evidence.txt:1'); check('unbound_evidence_before',bad)
(root/'review-evidence.txt').write_text('changed semantic evidence',encoding='utf-8'); check('unbound_evidence_after',bad)
bad=copy.deepcopy(layer)
for f in bad['unknown_families']: f.pop('decision')
check('resolved_families_no_decision',bad)
for field,value in [('kind',[]),('conclusion',[]),('bound_input_hashes',['not-a-map']),('evidence',17),('id',{'bad':'id'})]:
 bad=copy.deepcopy(layer); bad['decisions'][0][field]=value; check('malformed_'+field,bad)
samples={
 'shadowed_host':'type IO {\n static fn read_line() int { return 1 }\n}\nfn print() int { return 2 }\nfn demo() int {\n print()\n return IO.read_line()\n}\n',
 'unrelated_dot_pipe':'type P {\n fn next() int { return 1 }\n}\nfn unrelated() int {\n .next()\n return 2 |> .next()\n}\n',
 'different_owner':'type P {\n fn next() int { return 1 }\n}\ntype Q {\n fn go() int { return .next() }\n}\n'}
for name,source in samples.items():
 (out/(name+'.at')).write_text(source,encoding='utf-8'); results[name]=ai.scan_module(name+'.at',source).to_json()
results['hashes']={p:ai.sha256_of(repo/p) for p in ['scripts/acc_inventory.py','scripts/tests/test_acc_inventory.py',('docs/plans/archive/743-acc-bootstrap-hir-contract.md' if (repo/'docs/plans/archive/743-acc-bootstrap-hir-contract.md').is_file() else 'docs/plans/743-acc-bootstrap-hir-contract.md'),'docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md','docs/design/strategy/auto-acc-bootstrap-contract.md','docs/reports/743-acc-hir-contract/manual-decisions.json','docs/reports/743-acc-hir-contract/proposed-spec-delta-phase2.md']}
(out/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:({'exit':v['exit'],'stderr_tail':v['stderr'][-550:]} if 'exit' in v else v.get('call_candidates') if 'call_candidates' in v else 'hashes') for k,v in results.items()},ensure_ascii=False,indent=2))
