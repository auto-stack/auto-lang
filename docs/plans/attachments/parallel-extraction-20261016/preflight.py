from pathlib import Path
import hashlib,json,re,subprocess
root=Path('D:/autostack/auto-lang');out=Path(__file__).resolve().parent
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
data=json.loads((out/'inputs.json').read_text(encoding='utf-8'))
dag='docs/plans/attachments/789/t01-frontend-dag.md'
frozen=out/'789-frontend-dag-frozen.md'
if dag in data['inputs']:
 assert sha(root/dag)==data['inputs'][dag]
 frozen.write_bytes((root/dag).read_bytes())
 data['frozen_dag_origin']=dict(path=dag,sha256=data['inputs'].pop(dag),note='origin was untracked at planning time; frozen bytes included here; implementation closure independently rechecks sources')
 data['inputs'][frozen.relative_to(root).as_posix()]=sha(frozen)
design='docs/design/37-v06-modularization-and-repo-boundaries.md'
data.setdefault('design_before_batch_sha256',data['inputs'][design]);data['inputs'][design]=sha(root/design)
(out/'inputs.json').write_text(json.dumps(data,ensure_ascii=False,indent=2),encoding='utf-8')
alloc=json.loads((out/'allocation.json').read_text());checks=[]
allplans=[p for p in (root/'docs/plans').rglob('*.md') if p.parent in [root/'docs/plans',root/'docs/plans/archive']]
for item in alloc:
 p=root/item['path'];s=p.read_text(encoding='utf-8');n=item['id']
 assert f'plan_id: PLAN-{n}\n' in s and 'status: drafting\n' in s and 'plan_revision: 1\n' in s
 assert 'current_step: 0\n' in s and 'total_steps: 8\n' in s
 assert [int(x) for x in re.findall(r'^## (\d+)\.',s,re.M)]==list(range(11))
 assert re.findall(r'^\| \[ \] \| (T-\d+) \|',s,re.M)==[f'T-{i:02}' for i in range(1,9)]
 assert re.findall(r'^\| (AC-\d+) \| (?:work|review|merge) \|',s,re.M)==[f'AC-{i:02}' for i in range(1,7)]
 assert len([f for f in allplans if f.name.startswith(f'{n}-')])==1
 deps=re.search(r'^depends_on: \[(.*)\]',s,re.M).group(1).split(',')
 for dep in deps:
  d=int(dep.strip().removeprefix('PLAN-'));files=list((root/'docs/plans/archive').glob(f'{d}-*.md'))
  assert len(files)==1 and 'status: archived' in files[0].read_text(encoding='utf-8')
 for target in re.findall(r'\]\(([^)]+)\)',s):
  if '://' not in target:assert (p.parent/target.split('#')[0]).exists(),(p,target)
 checks.append(dict(plan=n,status='drafting',tasks=8,criteria=6,dependencies='all archived',sha256=sha(p)))
for p,h in data['inputs'].items():assert sha(root/p)==h,p
observed_next=int((root/'docs/plans/.next-id').read_text().strip())
all_ids=[int(m.group(1)) for p in allplans if (m:=re.match(r'^(\d{3})-',p.name))]
assert observed_next==max(all_ids)+1 and observed_next>=796
assert not (root/'docs/plans/.new-plan.lock').exists()
paths=[x['path'] for x in alloc]+['docs/plans/.next-id','docs/design/37-v06-modularization-and-repo-boundaries.md','docs/plans/reports/v06-extraction-execution-roadmap.md']
q=subprocess.run(['git','-C',str(root),'diff','--check','--',*paths],capture_output=True,text=True);assert q.returncode==0,q.stdout+q.stderr
result=dict(date='2026-10-16',stage='new',contract_preflight='pass',baseline=data['commit'],plans=checks,batch_next_id_after_allocation=796,observed_global_next_id=observed_next,parallel_allocations_preserved=[x for x in all_ids if x>=796],parallel_lanes=[792,793,794,795],shared_writes='manifest/exports/boundary baseline short serialized windows; later landing rechecks producer closure',scope='docs/plan only; no implementation, agents, worktrees, cargo tests, spec publication, or old repository writes')
(out/'preflight.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False))
