from pathlib import Path
import hashlib,json,re,subprocess
root=Path('D:/autostack/auto-lang');out=Path(__file__).resolve().parent
allocation=json.loads((out/'allocation.json').read_text())
manifest=json.loads((out/'inputs.json').read_text(encoding='utf-8'))
# Design 37 has now received this batch's proposed architecture; retain both fingerprints.
design='docs/design/37-v06-modularization-and-repo-boundaries.md'
manifest['design_before_batch_sha256']=manifest.get('design_before_batch_sha256',manifest['inputs'][design])
manifest['inputs'][design]=hashlib.sha256((root/design).read_bytes()).hexdigest()
(out/'inputs.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
checks=[]
allplans=[p for p in (root/'docs/plans').rglob('*.md') if p.parent in [root/'docs/plans',root/'docs/plans/archive']]
for record in allocation:
 p=root/record['path'];s=p.read_text(encoding='utf-8');id=record['id']
 assert f'plan_id: PLAN-{id}\n' in s and 'status: drafting\n' in s and 'current_step: 0\n' in s
 assert 'plan_revision: 1\n' in s and 'total_steps: 8\n' in s
 assert len([f for f in allplans if f.name.startswith(str(id)+'-')])==1
 assert [int(n) for n in re.findall(r'^## (\d+)\.',s,re.M)]==list(range(11))
 assert re.findall(r'^\| \[ \] \| (T-\d+) \|',s,re.M)==[f'T-{n:02}' for n in range(1,9)]
 assert re.findall(r'^\| (AC-\d+) \| (?:work|review|merge) \|',s,re.M)==[f'AC-{n:02}' for n in range(1,7)]
 for target in re.findall(r'\]\(([^)]+)\)',s):
  if '://' not in target:assert (p.parent/target.split('#')[0]).exists(),(p,target)
 checks.append(dict(plan=id,revision=1,status='drafting',tasks=8,acceptance=6,sections=11,local_links='pass',sha256=hashlib.sha256(p.read_bytes()).hexdigest()))
assert int((root/'docs/plans/.next-id').read_text().strip())==max(record['id'] for record in allocation)+1
assert not (root/'docs/plans/.new-plan.lock').exists()
for source,sha in manifest['inputs'].items():assert hashlib.sha256((root/source).read_bytes()).hexdigest()==sha,source
args=['git','-C',str(root),'diff','--check','--','docs/design/37-v06-modularization-and-repo-boundaries.md','docs/plans/reports/v06-extraction-execution-roadmap.md','docs/plans/.next-id',*[r['path'] for r in allocation]]
q=subprocess.run(args,capture_output=True,text=True);assert q.returncode==0,q.stdout+q.stderr
result=dict(date='2026-10-16',stage='new',contract_preflight='pass',handoff={'789':'contract ready; scope confirmation before work','790':'contract ready; native hotspot released before work','791':'contract ready; scope confirmation before work','792':'blocked on 789/790/791 delivery; drafting'},baseline=manifest['commit'],plans=checks,next_id=793,scope='planning only; no implementation/spec publication/worktrees/old repo writes',validation='metadata/task/AC/section/link/ID uniqueness/source fingerprint/diff-check; no cargo or docs_gen',parallelism='789/790/791 subject to shared-file windows; 792 after all three merge')
(out/'preflight.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(result,ensure_ascii=False))
