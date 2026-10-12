from pathlib import Path
import sys,copy,json,importlib.util,subprocess,os,shutil
repo=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(parents=True,exist_ok=False);os.environ['PYTHONUTF8']='1'
def load(name,path):
 s=importlib.util.spec_from_file_location(name,path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
ai=load('review_inventory',repo/'scripts/acc_inventory.py');tm=load('review_tests',repo/'scripts/tests/test_acc_inventory.py');c=tm.StrictDecisionsGate();c.setUp();results={}
def run(*flags):
 p=subprocess.run([sys.executable,'-B',str(repo/'scripts/acc_inventory.py'),'--root',str(c.repo),'--output',str(c.out),*flags],capture_output=True,text=True,encoding='utf8',timeout=20,env={**os.environ,'PYTHONUTF8':'1'})
 return {'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr}
try:
 results['fixture_write']=run('--write');base=c.complete_layer();results['valid_control']=run('--check','--require-decisions');assert results['valid_control']['exit']==0
 samples={
  'mut_parameter':'type Meter {\n fn read_line() int { return 1 }\n}\nfn demo(mut IO Meter) int {\n return IO.read_line()\n}\n',
  'method_parameter':'type Meter {\n fn demo(IO Other) int {\n  return IO.read_line()\n }\n}\n',
  'bare_parameter':'fn demo(print Other) int {\n return print()\n}\n',
  'method_bare_parameter':'type Meter {\n fn demo(print Other) int {\n  return print()\n }\n}\n',
  'usual_parameter':'fn demo(IO Meter) int {\n return IO.read_line()\n}\n',
  'conflict_free':'fn demo() {\n IO.read_line()\n print()\n}\n'}
 for name,source in samples.items():
  (out/(name+'.at')).write_text(source,encoding='utf8');results[name]=ai.scan_module(name+'.at',source).to_json()
 saved=(c.out/ai.MANIFEST_NAME).read_bytes();manifest=json.loads(saved)
 for name,value in {'list':[],'integer':1,'source_identity_list':{**manifest,'source_identity':[]},'source_identity_null':{**manifest,'source_identity':None}}.items():
  (c.out/ai.MANIFEST_NAME).write_text(json.dumps(value),encoding='utf8');results['manifest_'+name]=run('--check','--require-decisions')
 (c.out/ai.MANIFEST_NAME).write_bytes(saved)
 scans={p:ai.scan_module(p,(c.repo/p).read_text(encoding='utf8')) for p in ai.SOURCE_MODULES};inputs=[{'path':p,'sha256':ai.sha256_of(c.repo/p)} for p in ai.MANAGED_INPUTS];unknowns=ai._unknown_candidates_from_scans(scans);original=ai._evidence_file
 for name in ['valid','invalid_conclusion','stale_binding','duplicate','invalid_type']:
  layer=copy.deepcopy(base);d=layer['decisions'][-1];d['evidence']=[original(e)+':910' for e in d['evidence']];tags=set(d['evidence'])
  if name=='invalid_conclusion':d['conclusion']='invalid'
  if name=='stale_binding':d['bound_input_hashes'][next(iter(d['bound_input_hashes']))]='0'*64
  if name=='duplicate':layer['decisions'].append(copy.deepcopy(d))
  if name=='invalid_type':d['evidence']=17
  path=c.out/ai.DECISIONS_NAME;path.write_text(json.dumps(layer),encoding='utf8');reads=[]
  def observe(ev):
   if ev in tags:reads.append({'evidence':ev,'caller_function':sys._getframe(1).f_code.co_name,'caller_line':sys._getframe(1).f_lineno})
   return original(ev)
  ai._evidence_file=observe
  try:state,msg=ai.validate_decisions(path,inputs,c.repo,unknowns,require=True)
  finally:ai._evidence_file=original
  # The observer forwards every argument/result unchanged; product CLI is run separately.
  results['consumer_'+name]={'status':state,'messages':msg,'cli':run('--check','--require-decisions'),'evidence_reads':reads,'consumer_reads':[x for x in reads if x['caller_function']=='<genexpr>']}
finally:c.doCleanups()
(out/'counterexamples.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
print('fixture_control_exit',results['valid_control']['exit'])
for name in samples:print(name,[(c['name'],c['kind']) for c in results[name]['call_candidates']])
for name in ['manifest_list','manifest_integer','manifest_source_identity_list','manifest_source_identity_null']:print(name,'exit',results[name]['exit'],'traceback','Traceback' in results[name]['stderr'])
for name in ['invalid_conclusion','stale_binding','duplicate','invalid_type']:print('consumer_'+name,'exit',results['consumer_'+name]['cli']['exit'],'consumer_reads',len(results['consumer_'+name]['consumer_reads']))
