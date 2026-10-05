import json,re,hashlib,subprocess,sys
from pathlib import Path
repo=Path(sys.argv[1]);out=Path(sys.argv[2]);audit={}
files=[repo/'docs/design/strategy/auto-acc-bootstrap-contract.md',*sorted((repo/'docs/reports/743-acc-hir-contract').glob('*.md')),repo/'docs/specs/auto-acc/project.md',repo/'docs/specs/auto-hir/stage-contract.md']
broken=[];checked=0
for f in files:
    fence=False
    for n,line in enumerate(f.read_text(encoding="utf-8").splitlines(),1):
        if re.match(r'^\s*(~~~|'+chr(96)*3+')',line):fence=not fence;continue
        if fence:continue
        for label,target in re.findall(r'\[([^\]]*)\]\(([^)]+)\)',line):
            if target.startswith(('http:','https:','#')):continue
            path=target.split('#')[0].strip('<>')
            checked+=1
            if not (f.parent/path).is_file():broken.append(dict(file=str(f.relative_to(repo)),line=n,target=target))
audit['markdown_links']={'checked':checked,'broken':broken}
proposal=(repo/'docs/reports/743-acc-hir-contract/proposed-spec-delta.md').read_text(encoding="utf-8")
blocks=re.findall(r'~~~markdown\n(.*?)\n~~~',proposal,re.S)
audit['frozen_delta']={}
for p,block in zip(['docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md'],blocks[:2]):
    audit['frozen_delta'][p]={'matches_proposed_body':(repo/p).read_text(encoding="utf-8").strip()==block.strip(),'sha256':hashlib.sha256((repo/p).read_bytes()).hexdigest()}
manifest=json.loads((repo/'docs/reports/743-acc-hir-contract/source-manifest.json').read_text(encoding="utf-8"))
audit['coverage']={'inputs':len(manifest['inputs']),'modules':len(manifest['observations']['modules']),'use_edges':sum(len(m['use_edges']) for m in manifest['observations']['modules'])}
audit['actual_type_new_calls']=[{'file':m['path'],**c} for m in manifest['observations']['modules'] for c in m['call_candidates'] if c['receiver'] in ['P','CG','Ar'] and c['name']=='new']
audit['stale_decisions']={}
decisions=json.loads((repo/'docs/reports/743-acc-hir-contract/manual-decisions.json').read_text(encoding="utf-8"))
for d in decisions['decisions']:
    changed=[p for p,h in d['bound_input_hashes'].items() if not (repo/p).is_file() or hashlib.sha256((repo/p).read_bytes()).hexdigest()!=h]
    if changed:audit['stale_decisions'][d['id']]=changed
plan_path=repo/'docs/plans/743-acc-bootstrap-hir-contract.md'
if not plan_path.is_file():plan_path=repo/'docs/plans/archive/743-acc-bootstrap-hir-contract.md'
plan=plan_path.read_text(encoding="utf-8")
audit['plan_task_ticks']={m.group(2):m.group(1) for m in re.finditer(r'^- \[([ x])\] (T-\d+)',plan,re.M)}
audit['candidate_quotes']={'candidate1_a8':'正向=741 fixtures 的源码改写版 + A8 微程序首批' in (repo/'docs/reports/743-acc-hir-contract/next-work-packages.md').read_text(encoding="utf-8"),'a2_source_existing':re.findall(r'^.*\| A2 \|.*$',(repo/'docs/reports/743-acc-hir-contract/acceptance-matrix.md').read_text(encoding="utf-8"),re.M)}
audit['ancestors']={}
for sha in ['9abc87acc','05d0127a4','bbfca37b8','274a9ded4','bab649682']:
    p=subprocess.run(['git','-c','safe.directory='+str(repo).replace('\\','/'),'-c','core.excludesfile=','-C',str(repo),'merge-base','--is-ancestor',sha,'HEAD'],capture_output=True,text=True)
    audit['ancestors'][sha]=p.returncode
(out/'audit.json').write_text(json.dumps(audit,ensure_ascii=False,indent=2)+'\n',encoding="utf-8")
print(json.dumps(audit,ensure_ascii=False,indent=2))
