from pathlib import Path
import hashlib,json,subprocess
root=Path('D:/autostack/auto-lang');wt=Path('D:/autostack/.wt/lang-784/auto-lang');out=Path(__file__).resolve().parent
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
r1=json.loads((out.parent/'review-r1/results.json').read_text(encoding='utf-8'))
r2=json.loads((out/'results.json').read_text(encoding='utf-8'))
sources=['docs/design/37-v06-modularization-and-repo-boundaries.md','docs/specs/project-model/project.md','docs/specs/auto-man/project.md','.autoos/module-boundaries.toml','.autoos/module-boundaries-baseline.json','scripts/modularity-audit.py','docs/plans/attachments/784/t04/migration-inventory.md']
inputs={p:{'sha256':sha(wt/p),'unchanged_from_r1':sha(wt/p)==r1['tracked_sha256'].get(p)} for p in sources}
checks=[]
for repo in ['auto-lang','auto-ui','auto-man']:
 p=subprocess.run(['C:/Program Files/Git/bin/bash.exe','D:/autostack/wt-guard.sh',f'D:/autostack/.wt/lang-784/{repo}'],capture_output=True,text=True)
 checks.append(dict(id='guard-'+repo,exit=p.returncode,output=p.stdout+p.stderr))
for label,args in [('diff-check',['diff','--check','7d51845ffcca4c886698caa947cbaca93bf7fdca..HEAD']),('head',['rev-parse','HEAD']),('status',['status','--porcelain'])]:
 p=subprocess.run(['git','-C',str(wt),*args],capture_output=True,text=True);checks.append(dict(id=label,exit=p.returncode,output=p.stdout+p.stderr))
protected={}
for name,old in r2['protected_before'].items():
 repo=Path('D:/autostack')/name;payload=Path('D:/autostack/.recovery/784-legacy-preservation')/name
 now={k:subprocess.check_output(['git','-C',str(repo),*args],text=True).strip() for k,args in dict(head=['rev-parse','HEAD'],branch=['branch','--show-current'],status=['status','--porcelain=v1','-uall'],refs=['for-each-ref','--format=%(refname) %(objectname)'],remotes=['remote','-v']).items()}
 now['payload_sha256']={p.relative_to(payload).as_posix():sha(p) for p in payload.rglob('*') if p.is_file()}
 now['source_sha256']={p:sha(repo/p) for p in old['source_sha256']}
 protected[name]=now==old
assert all(protected.values()) and all(x['exit']==0 for x in checks)
assert checks[-1]['output']=='' and checks[-2]['output'].strip()==r2['head']
assert all(sha(wt/p)==h for p,h in r2['tracked_sha256'].items())
record=dict(reviewed_commit=r2['head'],base_commit='7d51845ffcca4c886698caa947cbaca93bf7fdca',spec_delta_sha256=sha(out/'spec-delta-reviewed.md'),inputs=inputs,checks=checks,protected_unchanged=protected,implementation_tracked_unchanged=True,evidence_sha256={p.name:sha(p) for p in out.iterdir() if p.is_file() and p.name!='fingerprint.json'})
(out/'fingerprint.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(dict(inputs_unchanged=all(x['unchanged_from_r1'] for x in inputs.values()),checks=checks,protected_unchanged=protected),ensure_ascii=False))
