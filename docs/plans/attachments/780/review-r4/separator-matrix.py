import json,subprocess,sys,hashlib
from pathlib import Path
out=Path(__file__).resolve().parent
root=Path('D:/autostack/.wt/lang-780/auto-lang')
scratch=root/'scratch/review780-r3';scratch.mkdir(parents=True,exist_ok=True)
source=scratch/'separator-matrix.rs';source.write_bytes((out/'separator-matrix.rs').read_bytes());exe=scratch/'separator-matrix.exe'
cmd=json.loads((out/'probe-run.json').read_text())['compile_command']
cmd=[str(source) if x.endswith('independent.rs') else x for x in cmd if x!='--test']
cmd[cmd.index('-o')+1]=str(exe)
with (out/'separator-matrix-build.log').open('w',encoding='utf-8') as f:
    q=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT)
assert q.returncode==0
cases=[]
for i in range(16):
    p=subprocess.Popen([str(exe),str(i)],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding='utf-8')
    timeout=False
    try: stdout,stderr=p.communicate(timeout=2)
    except subprocess.TimeoutExpired:
        timeout=True;p.kill();stdout,stderr=p.communicate()
    cases.append(dict(id=i,timed_out=timeout,exit=p.returncode,stdout=stdout,stderr=stderr,probe_pass=not timeout and p.returncode==0))
(out/'separator-matrix.json').write_text(json.dumps(dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),cases=cases)),encoding='utf-8')
for c in cases: print(json.dumps({k:c[k] for k in ['id','timed_out','exit','probe_pass']}))
sys.exit(0 if all(c['probe_pass'] for c in cases) else 1)
