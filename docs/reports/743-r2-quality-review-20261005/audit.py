import json,re,sys,hashlib
from pathlib import Path
repo=Path(sys.argv[1]); out=Path(sys.argv[2]); plan_rel='docs/plans/archive/743-acc-bootstrap-hir-contract.md' if (repo/'docs/plans/archive/743-acc-bootstrap-hir-contract.md').is_file() else 'docs/plans/743-acc-bootstrap-hir-contract.md'
data=json.loads((repo/'docs/reports/743-acc-hir-contract/manual-decisions.json').read_text(encoding='utf-8'))
def evfile(s):
 h,_,t=s.rpartition(':');return h if h and t.isdigit() else s
unbound=[{'id':d['id'],'file':evfile(e)} for d in data['decisions'] for e in d['evidence'] if evfile(e) not in d['bound_input_hashes']]
files=[plan_rel,'docs/design/strategy/auto-acc-bootstrap-contract.md','docs/reports/743-acc-hir-contract/acceptance-matrix.md','docs/reports/743-acc-hir-contract/inventory.md','docs/reports/743-acc-hir-contract/next-work-packages.md','docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md','docs/specs/auto-acc/plans.md','docs/specs/auto-hir/plans.md']
broken=[]; count=0
for rel in files:
 f=repo/rel; text=f.read_text(encoding='utf-8'); fence=False
 for n,line in enumerate(text.splitlines(),1):
  if re.match(r'^\s*(```|~~~)',line):fence=not fence;continue
  if fence:continue
  for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)',line):
   target=target.strip('<>').split('#')[0]
   if not target or re.match(r'\w+://',target):continue
   count+=1
   if not (f.parent/target).exists():broken.append({'file':rel,'line':n,'target':target})
p=repo/'docs/reports/743-acc-hir-contract/proposed-spec-delta-phase2.md'; blocks=re.findall(r'~~~markdown\r?\n(.*?)\r?\n~~~',p.read_text(encoding='utf-8'),re.S)
delta={rel:block in (repo/rel).read_text(encoding='utf-8') for rel,block in zip(['docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md'],blocks)}
result={'unbound_current_evidence':unbound,'local_links_checked':count,'broken_links':broken,'canonical_delta_text_matches_frozen':delta,'hashes':{rel:hashlib.sha256((repo/rel).read_bytes()).hexdigest() for rel in files}}
(out/'audit.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:v for k,v in result.items() if k!='hashes'},ensure_ascii=False,indent=2))
