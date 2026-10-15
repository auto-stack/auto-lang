"""Plan780 independent scoped acceptance evidence; no full Rust tiers."""
import hashlib,json,subprocess
from pathlib import Path
ROOT=Path('D:/autostack/.wt/lang-780/auto-lang')
OUT=Path(__file__).resolve().parent
def git(*args): return subprocess.check_output(['git','-C',str(ROOT),*args],text=True).strip()
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
before={p:digest(ROOT/p) for p in git('ls-files').splitlines() if (ROOT/p).is_file()}
head=git('rev-parse','HEAD')
locks=[ROOT/'Cargo.lock',ROOT/'test/fixtures/modularity/project-model/consumer/Cargo.lock']
lock_before={str(p.relative_to(ROOT)):digest(p) for p in locks if p.exists()}
jobs=[('model.log',['cargo','test','-p','auto-project-model','--locked']),
 ('consumer.log',['cargo','test','--manifest-path','test/fixtures/modularity/project-model/consumer/Cargo.toml','--locked']),
 ('adapter.log',['cargo','test','-p','auto-man','--lib','project_model','--locked','--','--nocapture']),
 ('check.log',['cargo','check','-p','auto-lang','-p','auto-man','--locked']),
 ('fmt-model.log',['cargo','fmt','-p','auto-project-model','--','--check'])]
results=[]
for name,args in jobs:
    with (OUT/name).open('w',encoding='utf-8') as f:
        q=subprocess.run(args,cwd=ROOT,stdout=f,stderr=subprocess.STDOUT,timeout=300)
    result={'file':name,'command':args,'exit_code':q.returncode,'sha256':digest(OUT/name)}
    results.append(result);print(json.dumps(result),flush=True)
fp={'head_before':head,'head_after':git('rev-parse','HEAD'),'branch':git('branch','--show-current'),
 'status_after':git('status','--porcelain'),'inputs_unchanged':all(digest(ROOT/p)==h for p,h in before.items()),
 'tracked_sha256':before,'locks_before':lock_before,'locks_after':{str(p.relative_to(ROOT)):digest(p) for p in locks if p.exists()},
 'spec_sha256':digest(Path('D:/autostack/auto-lang/docs/specs/auto-man/project.md')),
 'dependency_head':subprocess.check_output(['git','-C',str(ROOT.parent/'auto-down'),'rev-parse','HEAD'],text=True).strip()}
(OUT/'fingerprints.json').write_text(json.dumps(fp,ensure_ascii=False)+'\n',encoding='utf-8')
(OUT/'runs.jsonl').write_text('\n'.join(json.dumps(r) for r in results)+'\n',encoding='utf-8')
print(json.dumps({'inputs_unchanged':fp['inputs_unchanged'],'status':fp['status_after']}))
