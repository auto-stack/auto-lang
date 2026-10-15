"""Replay the unchanged external consumer against the default UI-enabled library."""
import hashlib,json,subprocess,sys
from pathlib import Path
root=Path('D:/autostack/.wt/lang-779/auto-lang');out=Path(__file__).resolve().parent
scratch=root/'scratch/review779-r2';scratch.mkdir(parents=True,exist_ok=True)
source=root/'test/fixtures/modularity/native-provider-consumer/tests/provider_vm.rs'
frozen=out/'provider_vm.rs';frozen.write_bytes(source.read_bytes())
(scratch/'provider_vm.rs').write_bytes(frozen.read_bytes())
with (out/'ui-build.jsonl').open('w',encoding='utf-8') as stdout,(out/'ui-build.log').open('w',encoding='utf-8') as stderr:
 q=subprocess.run(['cargo','build','-p','auto-lang','--lib','--locked','--message-format=json'],cwd=root,stdout=stdout,stderr=stderr,timeout=600)
assert q.returncode==0,'UI library build failed'
libs={};native=set()
for line in (out/'ui-build.jsonl').read_text().splitlines():
 try:r=json.loads(line)
 except json.JSONDecodeError:continue
 if r.get('reason')=='build-script-executed':native.update(r.get('linked_paths',[]))
 if r.get('reason')=='compiler-artifact' and r['target']['name'] in ['auto_lang','auto_val']:
  for p in r['filenames']:
   if p.endswith('.rlib'):libs[r['target']['name']]=p
assert set(libs)=={'auto_lang','auto_val'}
exe=scratch/'ui-consumer.exe'
cmd=['rustc','--edition','2021','--test',str(scratch/'provider_vm.rs'),'-L','dependency='+str(root/'target/debug/deps'),'-o',str(exe)]
for name,path in libs.items():cmd+=['--extern',name+'='+path]
for p in sorted(native):cmd+=['-L',p]
with (out/'ui-consumer-build.log').open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT,timeout=180)
assert q.returncode==0,'probe compilation failed'
with (out/'ui-consumer.log').open('w',encoding='utf-8') as f:q=subprocess.run([str(exe),'--nocapture','--test-threads=1'],cwd=root,stdout=f,stderr=subprocess.STDOUT,timeout=60)
record=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),frozen_source_unchanged=frozen.read_bytes()==source.read_bytes(),compile_command=cmd,exit=q.returncode)
(out/'ui-consumer.json').write_text(json.dumps(record),encoding='utf-8')
print(json.dumps({k:record[k] for k in ['head','source_sha256','frozen_source_unchanged','exit']}))
print('\n'.join((out/'ui-consumer.log').read_text().splitlines()[-20:]));sys.exit(q.returncode)
