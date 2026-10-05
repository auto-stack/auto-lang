import hashlib,json,re,sys
from pathlib import Path
r=Path(sys.argv[1]);out=Path(sys.argv[2]);j=json.loads((r/'docs/reports/743-acc-hir-contract/manual-decisions.json').read_text(encoding='utf-8'))
plan='docs/plans/archive/743-acc-bootstrap-hir-contract.md' if (r/'docs/plans/archive/743-acc-bootstrap-hir-contract.md').is_file() else 'docs/plans/743-acc-bootstrap-hir-contract.md'
def evfile(s):h,_,t=s.rpartition(':');return h if h and t.isdigit() else s
byid={d['id']:d for d in j['decisions']}
current_applicability=[{'path':f['path'],'decision':f.get('decision'),'evidence_covers':any(evfile(e)==f['path'] for e in byid[f['decision']]['evidence'])} for f in j['unknown_families'] if f['disposition']=='resolved']
files=[plan,'docs/design/strategy/auto-acc-bootstrap-contract.md','docs/reports/743-acc-hir-contract/acceptance-matrix.md','docs/reports/743-acc-hir-contract/inventory.md','docs/reports/743-acc-hir-contract/next-work-packages.md','docs/reports/743-acc-hir-contract/run-context.md','docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md','docs/specs/auto-acc/plans.md','docs/specs/auto-hir/plans.md']
broken=[];checked=0
for rel in files:
 f=r/rel;fence=False
 for n,line in enumerate(f.read_text(encoding='utf-8').splitlines(),1):
  if re.match(r'^\s*(```|~~~)',line):fence=not fence;continue
  if fence:continue
  for target in re.findall(r'\[[^\]]*\]\(([^)]+)\)',line):
   target=target.strip('<>').split('#')[0]
   if not target or re.match(r'\w+://',target):continue
   checked+=1
   if not (f.parent/target).exists():broken.append({'file':rel,'line':n,'target':target})
blocks=re.findall(r'~~~markdown\n(.*?)\n~~~',(r/'docs/reports/743-acc-hir-contract/proposed-spec-delta-phase3.md').read_text(encoding='utf-8'),re.S)
canonical=[r/'docs/specs/auto-acc/project.md',r/'docs/specs/auto-acc/project.md',r/'docs/specs/auto-hir/stage-contract.md']
pt=(r/plan).read_text(encoding='utf-8');meta={k:re.search('^'+k+': (.*)$',pt,re.M).group(1) for k in ['status','plan_revision','current_step','total_steps']}
ledger=json.loads((r/'.autoos/specs.json').read_text(encoding='utf-8'));ptr=[]
def walk(obj):
 if isinstance(obj,dict):
  if str(obj.get('id','')).startswith('P743-') and 'file' in obj:ptr.append({'id':obj['id'],'file':obj['file'],'exists':(r/obj['file']).is_file()})
  for v in obj.values():walk(v)
 elif isinstance(obj,list):
  for v in obj:walk(v)
walk(ledger)
res={'links_checked':checked,'broken_links':broken,'canonical_matches_phase3_delta':[b in p.read_text(encoding='utf-8') for b,p in zip(blocks,canonical)],'current_resolved_applicability':current_applicability,'plan_metadata':meta,'ledger_pointers':ptr,'hashes':{p:hashlib.sha256((r/p).read_bytes()).hexdigest() for p in files}}
(out/'audit.json').write_text(json.dumps(res,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:v for k,v in res.items() if k!='hashes'},ensure_ascii=False,indent=2))
