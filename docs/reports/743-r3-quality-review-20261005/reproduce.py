import copy,hashlib,importlib.util,json,os,re,shutil,subprocess,sys
from pathlib import Path
repo=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=True)
s=importlib.util.spec_from_file_location('inv',repo/'scripts/acc_inventory.py');ai=importlib.util.module_from_spec(s);s.loader.exec_module(ai)
env={**os.environ,'PYTHONUTF8':'1'}
def run(root,report,*args):
 p=subprocess.run([sys.executable,'-B',str(repo/'scripts/acc_inventory.py'),'--root',str(root),'--output',str(report),*args],capture_output=True,text=True,encoding='utf-8',timeout=20,env=env)
 return {'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
root=out/'fixture-repo';shutil.copytree(repo/'scripts/tests/fixtures/acc-inventory/repo',root);report=out/'fixture-report'
res={'fixture_write':run(root,report,'--write')};scans={p:ai.scan_module(p,(root/p).read_text(encoding='utf-8')) for p in ai.SOURCE_MODULES};unks=ai._unknown_candidates_from_scans(scans);pairs=sorted({(p,k) for p,k,n in unks});bindings={p:ai.sha256_of(root/p) for p in ai.MANAGED_INPUTS}
layer={'format_version':1,'decisions':[{'id':f'MD-REVIEW-{i:03}','subject':p,'kind':'module-role','conclusion':'adapt','evidence':[p+':1'],'note':'review control','bound_input_hashes':{p:bindings[p]}} for i,p in enumerate(ai.MANAGED_INPUTS)],'unknown_families':[{'path':p,'kind':k,'names':'*','disposition':'resolved','decision':'MD-REVIEW-900','note':'review control'} for p,k in pairs]}
unknown_paths=sorted({p for p,k in pairs});layer['decisions'].append({'id':'MD-REVIEW-900','subject':'review-unknown-families','kind':'unknown-resolution','conclusion':'resolved','evidence':[p+':1' for p in unknown_paths],'note':'review control','bound_input_hashes':{p:bindings[p] for p in unknown_paths}})
def check(label,data):
 (report/ai.DECISIONS_NAME).write_text(json.dumps(data,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');res[label]=run(root,report,'--check','--require-decisions')
check('valid_control',layer)
for key,value in [('evidence',17),('evidence',[17]),('bound_input_hashes',17),('bound_input_hashes',None)]:
 bad=copy.deepcopy(layer);bad['decisions'][-1][key]=value;check('invalid_referenced_'+key+'_'+type(value).__name__,bad)
# Applicable kind and result, but evidence deliberately cites only unrelated registry host.
for bypass in ['binding','subject']:
 bad=copy.deepcopy(layer);d=bad['decisions'][-1];d['evidence']=[ai.REGISTRY_HOST+':1'];d['bound_input_hashes']={ai.REGISTRY_HOST:bindings[ai.REGISTRY_HOST]}
 if bypass=='binding':d['bound_input_hashes'].update({p:bindings[p] for p in unknown_paths})
 else:
  target=bad['unknown_families'][0];d['subject']=target['path'];bad['unknown_families']=bad['unknown_families'][:1]
  # Remaining candidates remain explicitly open under another responsible owner.
  for p,k in pairs[1:]:bad['unknown_families'].append({'path':p,'kind':k,'names':'*','disposition':'open','owner':'research-line','probe':'probe-demo','work_package':'later-runtime'})
 check('evidence_applicability_'+bypass,bad)
# Original r2 binding/ref failure modes must be controlled after r3.
bad=copy.deepcopy(layer);(root/'extra-evidence.txt').write_text('first',encoding='utf-8');bad['decisions'][0]['evidence'].append('extra-evidence.txt:1');check('original_unbound_evidence',bad);bad['decisions'][0]['bound_input_hashes']['extra-evidence.txt']=ai.sha256_of(root/'extra-evidence.txt');check('bound_evidence_control',bad);(root/'extra-evidence.txt').write_text('second',encoding='utf-8');check('bound_evidence_changed',bad)
bad=copy.deepcopy(layer)
for f in bad['unknown_families']:f.pop('decision')
check('original_resolved_no_ref',bad)
samples={'import_shadow':'use other: IO, List, File, process, print\nfn demo() {\n IO.read_line()\n List.new()\n File.read_text()\n process.args()\n print()\n}\n','parameter_shadow':'type Meter {\n fn read_line() int { return 1 }\n}\nfn demo(IO Meter) int {\n return IO.read_line()\n}\n'}
for name in ['shadowed_host','different_owner','unrelated_dot_pipe']:
 samples[name]=(repo/'docs/reports/743-r2-quality-review-20261005'/f'{name}.at').read_text(encoding='utf-8')
for name,source in samples.items():
 (out/(name+'.at')).write_text(source,encoding='utf-8');res[name]=ai.scan_module(name+'.at',source).to_json()
plan='docs/plans/archive/743-acc-bootstrap-hir-contract.md' if (repo/'docs/plans/archive/743-acc-bootstrap-hir-contract.md').exists() else 'docs/plans/743-acc-bootstrap-hir-contract.md'
res['hashes']={p:ai.sha256_of(repo/p) for p in ['scripts/acc_inventory.py','scripts/tests/test_acc_inventory.py',plan,'docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md','docs/design/strategy/auto-acc-bootstrap-contract.md','docs/reports/743-acc-hir-contract/manual-decisions.json','docs/reports/743-acc-hir-contract/proposed-spec-delta-phase3.md']}
(out/'counterexamples.json').write_text(json.dumps(res,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({k:({'exit':v['exit'],'traceback':'Traceback' in v['stderr'],'diagnostic':v['stderr'][-210:]} if 'exit' in v else v.get('call_candidates') if 'call_candidates' in v else 'hashes') for k,v in res.items()},ensure_ascii=False,indent=2))
