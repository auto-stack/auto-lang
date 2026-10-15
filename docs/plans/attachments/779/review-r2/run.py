import hashlib,json,os,subprocess
from pathlib import Path
root=Path('D:/autostack/.wt/lang-779/auto-lang');out=Path(__file__).resolve().parent
def git(*a):return subprocess.check_output(['git','-C',str(root),*a],text=True).strip()
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
before={p:sha(root/p) for p in git('ls-files').splitlines() if (root/p).is_file()}
head=git('rev-parse','HEAD')
locks=[root/'Cargo.lock',root/'test/fixtures/modularity/native-provider-consumer/Cargo.lock']
lock_before={str(p.relative_to(root)):sha(p) for p in locks if p.exists()}
jobs=[('consumer.log',['cargo','test','--manifest-path','test/fixtures/modularity/native-provider-consumer/Cargo.toml','--locked'],{}),
('plan779.log',['cargo','test','-p','auto-lang','--lib','plan779','--locked','--','--nocapture'],{}),
('plan755.log',['cargo','test','-p','auto-lang','--lib','plan755','--locked'],{}),
('plan716.log',['cargo','test','-p','auto-lang','--lib','plan716_supply','--locked'],{}),
('gate-program.log',['cargo','test','-p','auto-lang','--lib','frame_program_track_gate_on','--locked','--','--nocapture'],{'AUTO_FRAME_BENCH':'1'}),
('gate-bridge.log',['cargo','test','-p','auto-lang','--lib','frame_value_flow_gate_on','--locked','--','--nocapture'],{'AUTO_FRAME_BENCH':'1'}),
('check.log',['cargo','check','-p','auto-lang','-p','auto','--locked'],{}),
('consumer-deps.log',['cargo','tree','--manifest-path','test/fixtures/modularity/native-provider-consumer/Cargo.toml','--locked','--offline','--edges','features'],{})]
with (out/'runs.jsonl').open('w',encoding='utf-8') as records:
 for name,cmd,env_delta in jobs:
  env=dict(os.environ);env.update(env_delta)
  with (out/name).open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=600)
  row=dict(file=name,command=cmd,env=env_delta,exit=q.returncode,sha256=sha(out/name));records.write(json.dumps(row)+'\n');records.flush();print(json.dumps(row),flush=True)
fp=dict(head_before=head,head_after=git('rev-parse','HEAD'),branch=git('branch','--show-current'),tracked_sha256=before,inputs_unchanged=all(sha(root/p)==h for p,h in before.items()),status=git('status','--porcelain'),locks_before=lock_before,locks_after={str(p.relative_to(root)):sha(p) for p in locks if p.exists()},specs={p:sha(Path('D:/autostack/auto-lang')/p) for p in ['docs/specs/auto-lang/vm/overview.md','docs/specs/auto-lang/ui/overview.md']},dependency_head=subprocess.check_output(['git','-C',str(root.parent/'auto-down'),'rev-parse','HEAD'],text=True).strip())
(out/'fingerprints.json').write_text(json.dumps(fp),encoding='utf-8');print(json.dumps({k:fp[k] for k in ['head_after','inputs_unchanged','status']}))
