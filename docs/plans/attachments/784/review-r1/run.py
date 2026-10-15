"""Independent preservation checks; mutations confined to fresh disposable repositories."""
import hashlib,json,subprocess,sys,uuid
from pathlib import Path
root=Path('D:/autostack/.wt/lang-784/auto-lang');out=Path(__file__).resolve().parent
tool=root/'scripts/legacy-preservation.py';scratch=Path('D:/autostack/.review')/('plan784-r1-'+uuid.uuid4().hex)
assert scratch.parent==Path('D:/autostack/.review') and not scratch.exists()
scratch.mkdir(parents=True)
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def git(repo,*a):return subprocess.check_output(['git','-C',str(repo),*a],text=True).strip()
def invoke(label,args):
 q=subprocess.run(args,cwd=root,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=120)
 (out/(label+'.log')).write_text(q.stdout+'\n'+q.stderr,encoding='utf-8')
 return q
def facts(repo):
 return dict(head=git(repo,'rev-parse','HEAD'),branch=git(repo,'branch','--show-current'),status=git(repo,'status','--porcelain=v1','-uall'),refs=git(repo,'for-each-ref','--format=%(refname) %(objectname)'),remotes=git(repo,'remote','-v'))
def protected():
 result={}
 for name in ['auto-ui','auto-man']:
  repo=Path('D:/autostack')/name;m=json.loads((root/f'docs/plans/attachments/784/t02-manifest-{name}.json').read_text())
  result[name]=facts(repo)
  result[name]['source_sha256']={e['path']:sha(repo/e['path']) for e in m['wip']+m['ignored_keep']+m['untracked']}
  payload=Path(m['payload_root'])/name
  result[name]['payload_sha256']={p.relative_to(payload).as_posix():sha(p) for p in payload.rglob('*') if p.is_file()}
 return result
tracked={p:sha(root/p) for p in git(root,'ls-files').splitlines() if (root/p).is_file()}
before=protected();head=git(root,'rev-parse','HEAD');results=[]
for name in ['auto-ui','auto-man']:
 repo=Path('D:/autostack')/name;m=json.loads((root/f'docs/plans/attachments/784/t02-manifest-{name}.json').read_text());payload=Path(m['payload_root'])/name
 cmd=[sys.executable,str(tool),'verify','--repo',str(repo),'--payload-dir',m['payload_root'],'--tip',m['repo']['head'],'--branch','legacy/pre-v06-extraction','--tag','legacy-pre-v06-'+m['repo']['head'][:8]]
 q=invoke('verify-'+name,cmd);results.append(dict(id='real-verify-'+name,exit=q.returncode,contract_pass=q.returncode==0))
 restored=scratch/(name+'-independent-bare');subprocess.run(['git','init','-q','--bare',str(restored)],check=True)
 subprocess.run(['git','-C',str(restored),'fetch','-q',str(payload/'bundle'/m['bundle']['file']),'refs/*:refs/*'],check=True)
 refs=dict(x.split(' ',1) for x in git(restored,'for-each-ref','--format=%(refname) %(objectname)').splitlines())
 ok=all(refs.get(k)==v for k,v in m['refs_frozen'].items()) and git(restored,'rev-parse',m['repo']['head']+'^{tree}')==git(repo,'rev-parse',m['repo']['head']+'^{tree}')
 for e in m['wip']:
  assert sha(payload/e['payload']['work'])==e['work_sha256']==sha(repo/e['path'])
  recovered=scratch/(name+'-wip')/e['path'];recovered.parent.mkdir(parents=True,exist_ok=True);recovered.write_bytes((payload/e['payload']['work']).read_bytes());assert sha(recovered)==e['work_sha256']
 for e in m['ignored_keep']+m['untracked']:assert sha(payload/e['payload'])==e['sha256']==sha(repo/e['path'])
 results.append(dict(id='independent-restore-'+name,refs=len(m['refs_frozen']),tags=sum(k.startswith('refs/tags/') for k in m['refs_frozen']),contract_pass=ok))
q=invoke('original-drill',[sys.executable,str(tool),'drill','--scratch',str(scratch/'original-drill'),'--keep']);results.append(dict(id='original-drill',exit=q.returncode,contract_pass=q.returncode==0))
def fixture(name):
 repo=scratch/name;repo.mkdir();subprocess.run(['git','init','-q','-b','master',str(repo)],check=True)
 for key,value in [('user.name','review'),('user.email','review@local')]:subprocess.run(['git','-C',str(repo),'config',key,value],check=True)
 (repo/'f.txt').write_text('committed\n');(repo/'.gitignore').write_text('private.cfg\n')
 subprocess.run(['git','-C',str(repo),'add','.'],check=True);subprocess.run(['git','-C',str(repo),'commit','-qm','baseline'],check=True)
 return repo
repo=fixture('dirty-overwrite');payload=scratch/'payload-dirty';(repo/'f.txt').write_text('first-uncommitted-snapshot\n')
args=[sys.executable,str(tool),'preserve','--repo',str(repo),'--payload-dir',str(payload)]
assert invoke('dirty-first',args).returncode==0
mp=payload/repo.name/'manifest.json';m=json.loads(mp.read_text());backup=payload/repo.name/m['wip'][0]['payload']['work'];old=sha(backup);old_manifest=sha(mp)
(repo/'f.txt').write_text('second-uncommitted-snapshot\n');q=invoke('dirty-second',args)
results.append(dict(id='preserve-must-retain-prior-snapshot',exit=q.returncode,prior_backup_sha256=old,current_backup_sha256=sha(backup),original_manifest_retained=sha(mp)==old_manifest,contract_pass=sha(backup)==old))
repo=fixture('ignored-drift');payload=scratch/'payload-ignored';(repo/'private.cfg').write_text('initial\n')
args=[sys.executable,str(tool),'preserve','--repo',str(repo),'--payload-dir',str(payload),'--ignored-keep','private.cfg'];assert invoke('ignored-first',args).returncode==0
tip=git(repo,'rev-parse','HEAD');assert invoke('ignored-anchor',[sys.executable,str(tool),'anchor','--repo',str(repo),'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre']).returncode==0
(repo/'private.cfg').write_text('changed-original\n');q=invoke('ignored-verify',[sys.executable,str(tool),'verify','--repo',str(repo),'--payload-dir',str(payload),'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre'])
results.append(dict(id='verify-original-ignored-drift',exit=q.returncode,expected='nonzero: original kept file changed',contract_pass=q.returncode!=0))
args=[sys.executable,str(root/'scripts/modularity-audit.py'),'--root',str(root),'--mode','strict','--output',str(out/'audit-worktree.json')]
q=invoke('audit-worktree',args);results.append(dict(id='strict-worktree',exit=q.returncode,contract_pass=q.returncode==0))
args=[sys.executable,str(root/'scripts/modularity-audit.py'),'--root','D:/autostack/auto-lang','--baseline',str(root/'.autoos/module-boundaries-baseline.json'),'--mode','strict','--output',str(out/'audit-main.json')]
q=invoke('audit-main',args);results.append(dict(id='strict-main-same-delivery-baseline',exit=q.returncode,contract_pass=q.returncode==0))
after=protected();assert before==after,'Protected state changed'
assert git(root,'rev-parse','HEAD')==head and not git(root,'status','--porcelain')
assert all(sha(root/p)==h for p,h in tracked.items())
record=dict(head=head,scratch=str(scratch),protected_before=before,protected_after=after,tracked_sha256=tracked,tracked_unchanged=True,protected_unchanged=True,results=results)
(out/'results.json').write_text(json.dumps(record,ensure_ascii=False),encoding='utf-8')
for row in results:print(json.dumps(row,ensure_ascii=False))
sys.exit(0 if all(r['contract_pass'] for r in results) else 1)
