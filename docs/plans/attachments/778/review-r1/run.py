"""Review evidence runner; builds/tests only in the implementation worktree."""
import json, hashlib, subprocess
from pathlib import Path
ROOT=Path('D:/autostack/.wt/lang-778/auto-lang')
OUT=Path(__file__).resolve().parent
def git(*args):
    return subprocess.check_output(['git','-C',str(ROOT),*args],text=True).strip()
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()
files=git('ls-files').splitlines()
before={p:digest(ROOT/p) for p in files if (ROOT/p).is_file()}
head=git('rev-parse','HEAD')
jobs=[('external.log',['cargo','test','--manifest-path','test/fixtures/modularity/ui-dialect-consumer/Cargo.toml','--locked','--test','v01_v05_external_dialect']),
      ('scoped.log',['cargo','nextest','run','-p','auto-lang','--lib','--features','ui-iced,test-cases','-E','test(parser::tests) | test(dialect) | test(model) | test(widget)']),
      ('check.log',['cargo','check','-p','auto-lang','--locked'])]
results=[]
for name,args in jobs:
    with (OUT/name).open('w',encoding='utf-8') as log:
        q=subprocess.run(args,cwd=ROOT,stdout=log,stderr=subprocess.STDOUT,timeout=300)
    r={'command':args,'file':name,'exit_code':q.returncode,'sha256':digest(OUT/name)}
    results.append(r)
    print(json.dumps(r),flush=True)
after={p:digest(ROOT/p) for p in before}
spec=Path('D:/autostack/auto-lang/docs/specs/auto-lang/frontend/overview.md')
fp={'head_before':head,'head_after':git('rev-parse','HEAD'),'branch':git('branch','--show-current'),
    'status_after':git('status','--porcelain'),'inputs_unchanged':before==after,'tracked_sha256':before,
    'spec_sha256':digest(spec),'locks':{str(p.relative_to(ROOT)):digest(p) for p in [ROOT/'Cargo.lock',ROOT/'test/fixtures/modularity/ui-dialect-consumer/Cargo.lock'] if p.exists()}}
dep=ROOT.parent/'auto-down'
fp['dependency_head']=subprocess.check_output(['git','-C',str(dep),'rev-parse','HEAD'],text=True).strip()
(OUT/'fingerprints.json').write_text(json.dumps(fp,ensure_ascii=False)+'\n',encoding='utf-8')
(OUT/'runs.jsonl').write_text('\n'.join(json.dumps(r) for r in results)+'\n',encoding='utf-8')
print(json.dumps({'inputs_unchanged':before==after,'status':fp['status_after']}),flush=True)
