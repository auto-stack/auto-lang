from pathlib import Path
import hashlib, json, re, subprocess

main=Path('D:/autostack/auto-lang')
wt=Path('D:/autostack/.wt/lang-784/auto-lang')
out=main/'docs/plans/attachments/784/review-r1'
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
spec=wt/'docs/specs/repository-boundaries/project.md'
(out/'spec-delta-reviewed.md').write_bytes(spec.read_bytes())
links=[]
for i,line in enumerate(spec.read_text(encoding='utf-8').splitlines(),1):
    for target in re.findall(r'\]\(([^)]+)\)',line):
        if '://' not in target:
            resolved=(spec.parent/target.split('#')[0]).resolve()
            links.append(dict(line=i,target=target,exists=resolved.exists()))
(out/'spec-links.json').write_text(json.dumps(links,ensure_ascii=False,indent=2),encoding='utf-8')
checks=[]
for repo in ('auto-lang','auto-ui','auto-man'):
    p=subprocess.run(['C:/Program Files/Git/bin/bash.exe','D:/autostack/wt-guard.sh',f'D:/autostack/.wt/lang-784/{repo}'],capture_output=True,text=True)
    checks.append(dict(check='wt-guard',repo=repo,exit=p.returncode,output=p.stdout+p.stderr))
p=subprocess.run(['git','-C',str(wt),'diff','--check','7d51845ffcca4c886698caa947cbaca93bf7fdca..HEAD'],capture_output=True,text=True)
checks.append(dict(check='diff-check',exit=p.returncode,output=p.stdout+p.stderr))
checks.append(dict(check='final-head',output=subprocess.check_output(['git','-C',str(wt),'rev-parse','HEAD'],text=True).strip()))
checks.append(dict(check='final-status',output=subprocess.check_output(['git','-C',str(wt),'status','--porcelain'],text=True).strip()))
sources=['docs/design/37-v06-modularization-and-repo-boundaries.md','docs/specs/project-model/project.md','docs/specs/auto-man/project.md','.autoos/module-boundaries.toml','scripts/modularity-audit.py']
inputs={s:sha(main/s) for s in sources if (main/s).is_file()}
fingerprint=dict(spec_delta_sha256=sha(out/'spec-delta-reviewed.md'),spec_inputs=inputs,checks=checks)
(out/'fingerprint.json').write_text(json.dumps(fingerprint,ensure_ascii=False,indent=2),encoding='utf-8')
assert all(c.get('exit',0)==0 for c in checks)
assert checks[-1]['output']==''
print(json.dumps(dict(broken_links=[x for x in links if not x['exists']],checks=checks),ensure_ascii=False))
