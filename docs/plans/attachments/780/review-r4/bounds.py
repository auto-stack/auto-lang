"""Subprocess deadlines test the replacement classifier's failure exits; never hangs the reviewer."""
import hashlib,json,subprocess,sys
from pathlib import Path
root=Path('D:/autostack/.wt/lang-780/auto-lang');out=Path(__file__).resolve().parent
scratch=root/'scratch/review780-r2';scratch.mkdir(parents=True,exist_ok=True)
source=scratch/'bounds.rs';source.write_bytes((out/'bounds.rs').read_bytes());exe=scratch/'bounds.exe'
cmd=json.loads((out/'probe-run.json').read_text())['compile_command']
cmd=[str(source) if x.endswith('independent.rs') else x for x in cmd if x!='--test']
cmd[cmd.index('-o')+1]=str(exe)
with (out/'bounds-build.log').open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,stdout=f,stderr=subprocess.STDOUT)
assert q.returncode==0,'probe compilation failed'
results=[]
for case in ['valid_array','valid_object','arithmetic_array','invalid_array','mismatched_bracket','invalid_object_key']:
    p=subprocess.Popen([str(exe),case],cwd=root,stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True,encoding='utf-8')
    timed_out=False
    try:stdout,stderr=p.communicate(timeout=2)
    except subprocess.TimeoutExpired:
        timed_out=True;p.kill();stdout,stderr=p.communicate()
    results.append({'id':case,'deadline_seconds':2,'timed_out':timed_out,'exit_code':p.returncode,'stdout':stdout,'stderr':stderr,'contract_pass':not timed_out and p.returncode==0})
(out/'bounds.json').write_text(json.dumps({'head':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'source_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),'cases':results},ensure_ascii=False)+'\n',encoding='utf-8')
for r in results:print(json.dumps({k:r[k] for k in ['id','timed_out','exit_code','contract_pass']}))
sys.exit(0 if all(r['contract_pass'] for r in results) else 1)
