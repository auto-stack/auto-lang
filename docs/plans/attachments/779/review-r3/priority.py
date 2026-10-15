import json,subprocess,hashlib,os
from pathlib import Path
out=Path(__file__).resolve().parent;root=Path('D:/autostack/.wt/lang-779/auto-lang')
scratch=root/'scratch/review779-r3';scratch.mkdir(parents=True,exist_ok=True)
source=scratch/'priority.rs';source.write_bytes((out/'priority.rs').read_bytes());exe=scratch/'priority.exe'
cmd=json.loads((out/'ui-consumer.json').read_text())['compile_command']
cmd=[str(source) if x.endswith('provider_vm.rs') else x for x in cmd if x!='--test'];cmd[cmd.index('-o')+1]=str(exe)
with (out/'priority-build.log').open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT,timeout=180)
assert q.returncode==0
env=dict(os.environ);env.pop('AUTO_FRAME_BENCH',None)
with (out/'priority.log').open('w',encoding='utf-8') as f:q=subprocess.run([str(exe)],cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=30)
(out/'priority.json').write_text(json.dumps(dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),compile_command=cmd,exit=q.returncode)),encoding='utf-8')
print((out/'priority.log').read_text());assert q.returncode==0
