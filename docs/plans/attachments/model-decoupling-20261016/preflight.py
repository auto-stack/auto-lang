from pathlib import Path
import hashlib,json,re,subprocess
root=Path('D:/autostack/auto-lang');out=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
sources=json.loads((out/'inputs.json').read_text(encoding='utf-8'))
design='docs/design/37-v06-modularization-and-repo-boundaries.md'
sources.setdefault('design_before_batch_sha256',sources['inputs'][design]);sources['inputs'][design]=sha(root/design)
changed_progress=[]
for rel in ['docs/plans/792-generated-ui-independent-build.md','docs/plans/794-scroll-host-provider.md']:
 if sha(root/rel)!=sources['inputs'][rel]:
  changed_progress.append(dict(path=rel,original_sha256=sources['inputs'][rel],current_sha256=sha(root/rel)))
  # Retain an exact copy of the now-read readiness record, without changing the other plan.
  frozen=out/(Path(rel).stem+'-input.md');frozen.write_bytes((root/rel).read_bytes())
  sources['inputs'][rel]=sha(root/rel)
  sources['inputs'][frozen.relative_to(root).as_posix()]=sha(frozen)
sources['concurrent_progress_changes_observed']=changed_progress
(out/'inputs.json').write_text(json.dumps(sources,ensure_ascii=False,indent=2),encoding='utf-8')
allocation=json.loads((out/'allocation.json').read_text());rows=[]
allplans=[p for p in (root/'docs/plans').rglob('*.md') if p.parent in [root/'docs/plans',root/'docs/plans/archive']]
for a in allocation:
 p=root/a['path'];s=p.read_text(encoding='utf-8');n=a['id']
 assert f'plan_id: PLAN-{n}\n' in s and 'status: drafting\n' in s and 'current_step: 0\n' in s
 assert 'plan_revision: 1\n' in s and 'total_steps: 8\n' in s
 assert [int(x) for x in re.findall(r'^## (\d+)\.',s,re.M)]==list(range(11))
 assert re.findall(r'^\| \[ \] \| (T-\d+) \|',s,re.M)==[f'T-{i:02}' for i in range(1,9)]
 assert re.findall(r'^\| (AC-\d+) \| (?:work|review|merge) \|',s,re.M)==[f'AC-{i:02}' for i in range(1,7)]
 assert len([f for f in allplans if f.name.startswith(f'{n}-')])==1
 deps=[int(x.strip().removeprefix('PLAN-')) for x in re.search(r'^depends_on: \[(.*)\]',s,re.M).group(1).split(',')]
 assert not any(x in [792,794,799,800] for x in deps)
 for d in deps:
  files=list((root/'docs/plans/archive').glob(f'{d}-*.md'));assert len(files)==1
  assert 'status: archived' in files[0].read_text(encoding='utf-8')
 for target in re.findall(r'\]\(([^)]+)\)',s):
  if '://' not in target:assert (p.parent/target.split('#')[0]).exists(),(p,target)
 rows.append(dict(plan=n,status='drafting',tasks=8,criteria=6,dependencies=deps,sha256=sha(p)))
report=root/'docs/plans/reports/model-decoupling-parallel-order-20261016.md'
for target in re.findall(r'\]\(([^)]+)\)',report.read_text(encoding='utf-8')):
 if '://' not in target:assert (report.parent/target.split('#')[0]).exists(),target
for p,h in sources['inputs'].items():assert sha(root/p)==h,p
ids=[int(m.group(1)) for p in allplans if (m:=re.match(r'^(\d{3})-',p.name))]
nextid=int((root/'docs/plans/.next-id').read_text().strip());assert nextid==max(ids)+1 and nextid>=801
assert not (root/'docs/plans/.new-plan.lock').exists()
changed=[a['path'] for a in allocation]+['docs/plans/.next-id','docs/plans/reports/model-decoupling-parallel-order-20261016.md','docs/plans/reports/v06-extraction-execution-roadmap.md',design]
q=subprocess.run(['git','-C',str(root),'diff','--check','--',*changed],capture_output=True,text=True);assert q.returncode==0,q.stdout+q.stderr
result=dict(date='2026-10-16',stage='new',contract_preflight='pass',baseline=sources['commit'],plans=rows,next_id=nextid,parallel=[792,794,799,800],recommended_tie_order=[792,794,799,800],hard_wait_for_792_794=False,validation='metadata/unique IDs/archived dependencies/section-task-AC mapping/local links/source SHA/diff-check',scope='plan/design docs only; no implementation/agent/worktree creation/cargo/spec publication/old repo writes')
(out/'preflight.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(result,ensure_ascii=False))
