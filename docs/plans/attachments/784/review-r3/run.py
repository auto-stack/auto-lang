"""Independent R3 checks. All mutations use fresh disposable repositories."""
import hashlib,json,subprocess,sys,uuid,re
from pathlib import Path
root=Path('D:/autostack/.wt/lang-784/auto-lang');out=Path(__file__).resolve().parent
tool=root/'scripts/legacy-preservation.py';scratch=Path('D:/autostack/.review')/('plan784-r3-'+uuid.uuid4().hex)
assert scratch.parent==Path('D:/autostack/.review') and not scratch.exists()
scratch.mkdir(parents=True)
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def git(repo,*args):return subprocess.check_output(['git','-C',str(repo),*args],text=True).strip()
def hashes(path):return {p.relative_to(path).as_posix():sha(p) for p in path.rglob('*') if p.is_file()}
def invoke(label,args):
 q=subprocess.run(args,cwd=root,capture_output=True,text=True,encoding='utf-8',errors='replace',timeout=120)
 (out/(label+'.log')).write_text(q.stdout+'\n'+q.stderr,encoding='utf-8');return q
def cli(label,verb,repo,payload,*extra):
 return invoke(label,[sys.executable,str(tool),verb,'--repo',str(repo),'--payload-dir',str(payload),*extra])
def facts(repo):return {k:git(repo,*v) for k,v in dict(head=['rev-parse','HEAD'],branch=['branch','--show-current'],status=['status','--porcelain=v1','-uall'],refs=['for-each-ref','--format=%(refname) %(objectname)'],remotes=['remote','-v']).items()}
def protected():
 result={}
 for name in ['auto-ui','auto-man']:
  repo=Path('D:/autostack')/name;payload=Path('D:/autostack/.recovery/784-legacy-preservation')/name
  result[name]=facts(repo);result[name]['payload_sha256']=hashes(payload)
  m=json.loads((root/f'docs/plans/attachments/784/t02-manifest-{name}.json').read_text())
  result[name]['source_sha256']={e['path']:sha(repo/e['path']) for e in m['wip']+m['ignored_keep']+m['untracked']}
 return result
tracked={p:sha(root/p) for p in git(root,'ls-files').splitlines() if (root/p).is_file()}
head=git(root,'rev-parse','HEAD');before=protected();results=[]
for name in ['auto-ui','auto-man']:
 repo=Path('D:/autostack')/name;payload=Path('D:/autostack/.recovery/784-legacy-preservation');base=payload/name
 sid=(base/'LATEST').read_text().strip();snap=base/'snapshots'/sid;m=json.loads((snap/'manifest.json').read_text())
 q=cli('real-verify-'+name,'verify',repo,payload,'--snapshot',sid,'--tip',m['repo']['head'],'--branch','legacy/pre-v06-extraction','--tag','legacy-pre-v06-'+m['repo']['head'][:8])
 results.append(dict(id='real-verify-'+name,exit=q.returncode,pass_=q.returncode==0 and json.loads(q.stdout)['ok']))
 restored=scratch/(name+'-independent-bare');subprocess.run(['git','init','-q','--bare',str(restored)],check=True)
 bundle=snap/'bundle'/f'{name}-full.bundle';subprocess.run(['git','-C',str(restored),'fetch','-q',str(bundle),'refs/*:refs/*'],check=True)
 refs=dict(x.split(' ',1) for x in git(restored,'for-each-ref','--format=%(refname) %(objectname)').splitlines())
 ok=all(refs.get(k)==v for k,v in m['refs_frozen'].items()) and git(restored,'rev-parse',m['repo']['head']+'^{tree}')==git(repo,'rev-parse',m['repo']['head']+'^{tree}')
 for e in m['state']['wip']:
  if 'work_sha256' in e:
   backup=snap/'wip'/(e['path'].replace('/','__')+'.work');assert sha(backup)==e['work_sha256']==sha(repo/e['path'])
   recovered=scratch/(name+'-wip')/e['path'];recovered.parent.mkdir(parents=True,exist_ok=True);recovered.write_bytes(backup.read_bytes());assert sha(recovered)==e['work_sha256']
 for key,sub in [('untracked','untracked'),('ignored_keep','ignored')]:
  for e in m['state'][key]:assert sha(snap/sub/e['path'])==e['sha256']==sha(repo/e['path'])
 results.append(dict(id='independent-restore-'+name,snapshot=sid,refs=len(refs),tags=sum(k.startswith('refs/tags/') for k in refs),pass_=ok))
q=invoke('drill',[sys.executable,str(tool),'drill','--scratch',str(scratch/'drill'),'--keep']);results.append(dict(id='permanent-drill',exit=q.returncode,pass_=q.returncode==0 and json.loads(q.stdout)['ok']))
def fixture(name):
 repo=scratch/name;repo.mkdir();subprocess.run(['git','init','-q','-b','master',str(repo)],check=True)
 for k,v in [('user.name','review'),('user.email','review@local')]:git(repo,'config',k,v)
 (repo/'f.txt').write_bytes(b'committed\n');(repo/'.gitignore').write_bytes(b'private.cfg\n')
 git(repo,'add','.');git(repo,'commit','-qm','baseline');return repo
repo=fixture('retention');payload=scratch/'retention-payload'
(repo/'f.txt').write_bytes(b'A\n');q=cli('retain-a','preserve',repo,payload);assert q.returncode==0
ma=json.loads(q.stdout);snap_a=Path(ma['manifest']).parent;before_a=hashes(snap_a)
(repo/'f.txt').write_bytes(b'B\n');q=cli('retain-b','preserve',repo,payload);assert q.returncode==0
mb=json.loads(q.stdout);before_all=hashes(payload);q=cli('retain-b-rerun','preserve',repo,payload)
results.append(dict(id='R1-01-retention',pass_=ma['snapshot']!=mb['snapshot'] and before_a==hashes(snap_a) and (snap_a/'wip/f.txt.work').read_bytes()==b'A\n' and before_all==hashes(payload) and json.loads(q.stdout)['action']=='reuse'))
repo=fixture('source-matrix');payload=scratch/'matrix-payload';tip=git(repo,'rev-parse','HEAD')
for path,value in [('f.txt',b'dirty\n'),('u.txt',b'untracked\n'),('private.cfg',b'ignored\n')]: (repo/path).write_bytes(value)
q=cli('matrix-preserve','preserve',repo,payload,'--ignored-keep','private.cfg');assert q.returncode==0
invoke('matrix-anchor',[sys.executable,str(tool),'anchor','--repo',str(repo),'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre'])
def verify(label):return cli(label,'verify',repo,payload,'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre')
backup_before=hashes(payload);q=verify('matrix-positive');assert q.returncode==0
for path in ['f.txt','u.txt','private.cfg']:
 original=(repo/path).read_bytes()
 for mutation in ['change','delete']:
  if mutation=='change':(repo/path).write_bytes(b'changed\n')
  else:(repo/path).unlink()
  q=verify('matrix-'+path+'-'+mutation);results.append(dict(id='source-'+path+'-'+mutation,exit=q.returncode,pass_=q.returncode!=0 and json.loads(q.stdout)['ok']==False and hashes(payload)==backup_before))
  (repo/path).write_bytes(original)
assert verify('matrix-restored').returncode==0
# New reference to an otherwise unreachable commit: HEAD and work bytes stay fixed.
repo=fixture('refs-only');payload=scratch/'refs-payload';q=cli('refs-before','preserve',repo,payload);assert q.returncode==0
m0=json.loads(q.stdout);snap0=Path(m0['manifest']).parent;snap0_before=hashes(snap0)
tree=git(repo,'rev-parse','HEAD^{tree}');new_commit=git(repo,'commit-tree',tree,'-m','unique archival commit');git(repo,'tag','archived-extra',new_commit)
q=cli('refs-after','preserve',repo,payload);m1=json.loads(q.stdout)
restored=scratch/'refs-independent-bare';subprocess.run(['git','init','-q','--bare',str(restored)],check=True)
bundle=Path(m1['manifest']).parent/'bundle'/f'{repo.name}-full.bundle';subprocess.run(['git','-C',str(restored),'fetch','-q',str(bundle),'refs/*:refs/*'],check=True)
recovered=subprocess.run(['git','-C',str(restored),'rev-parse','--verify','refs/tags/archived-extra'],capture_output=True,text=True)
results.append(dict(id='refs-only-new-tag-retained',exit=q.returncode,action=m1['action'],before_snapshot=m0['snapshot'],after_snapshot=m1['snapshot'],new_tag=new_commit,recovered_tag=recovered.stdout.strip(),old_snapshot_unchanged=hashes(snap0)==snap0_before,pass_=q.returncode==0 and recovered.returncode==0 and recovered.stdout.strip()==new_commit))
# Inserted before final fingerprint/assertions in the R2 replay, in the same fresh scratch.
all_refs=lambda r:dict(x.split(' ',1) for x in git(r,'for-each-ref','--format=%(refname) %(objectname)').splitlines())
def restored_refs(label,manifest):
 bare=scratch/(label+'-bare');subprocess.run(['git','init','-q','--bare',str(bare)],check=True)
 bundle=Path(manifest['manifest']).parent/'bundle'/f"{manifest['repo']}-full.bundle"
 subprocess.run(['git','-C',str(bare),'fetch','-q',str(bundle),'refs/*:refs/*'],check=True)
 return all_refs(bare)
# Continue the independent R2 refs-only fixture: add -> move -> delete -> initial state.
old0=hashes(Path(m0['manifest']).parent);old1=hashes(Path(m1['manifest']).parent)
git(repo,'tag','-f','archived-extra',git(repo,'rev-parse','HEAD'))
q=cli('refs-moved','preserve',repo,payload);m2=json.loads(q.stdout)
results.append(dict(id='refs-move-restored',exit=q.returncode,pass_=q.returncode==0 and m2['snapshot'] not in [m0['snapshot'],m1['snapshot']] and restored_refs('refs-moved',m2)==all_refs(repo) and hashes(Path(m0['manifest']).parent)==old0 and hashes(Path(m1['manifest']).parent)==old1))
old2=hashes(Path(m2['manifest']).parent);git(repo,'tag','-d','archived-extra')
q=cli('refs-deleted','preserve',repo,payload);m3=json.loads(q.stdout)
results.append(dict(id='refs-delete-restored',exit=q.returncode,pass_=q.returncode==0 and m3['snapshot']==m0['snapshot'] and restored_refs('refs-deleted',m3)==all_refs(repo) and hashes(Path(m2['manifest']).parent)==old2))
# Same HEAD, a branch-only change: exercise a non-tag namespace.
git(repo,'branch','retained-side',new_commit);q=cli('refs-branch-added','preserve',repo,payload);m4=json.loads(q.stdout)
results.append(dict(id='refs-branch-restored',exit=q.returncode,pass_=q.returncode==0 and m4['snapshot']!=m0['snapshot'] and restored_refs('refs-branch-added',m4)==all_refs(repo)))
# Valid old bundle + updated matching checksum: refs coverage must reject independently of hash.
repo=fixture('bundle-oracle');payload=scratch/'oracle-payload';tip=git(repo,'rev-parse','HEAD')
q=invoke('oracle-anchor',[sys.executable,str(tool),'anchor','--repo',str(repo),'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre']);assert q.returncode==0
q=cli('oracle-before','preserve',repo,payload);assert q.returncode==0;o0=json.loads(q.stdout)
tree=git(repo,'rev-parse','HEAD^{tree}');side=git(repo,'commit-tree',tree,'-m','oracle independent commit');git(repo,'tag','oracle-extra',side)
q=cli('oracle-after','preserve',repo,payload);assert q.returncode==0;o1=json.loads(q.stdout)
oldbundle=Path(o0['manifest']).parent/'bundle'/f'{repo.name}-full.bundle';newbundle=Path(o1['manifest']).parent/'bundle'/f'{repo.name}-full.bundle'
newbundle.write_bytes(oldbundle.read_bytes());mp=Path(o1['manifest']);mm=json.loads(mp.read_text());mm['bundle']['sha256']=sha(newbundle);mp.write_text(json.dumps(mm,ensure_ascii=False,indent=2),encoding='utf-8')
before_refuse=hashes(payload)
q=cli('oracle-verify-refused','verify',repo,payload,'--snapshot',o1['snapshot'],'--tip',tip,'--branch','legacy/pre','--tag','legacy-pre')
v=json.loads(q.stdout)
results.append(dict(id='oracle-valid-bundle-missing-ref-verify',exit=q.returncode,pass_=q.returncode!=0 and any('bundle 未覆盖 ref' in x and 'oracle-extra' in x for x in v['failures']) and hashes(payload)==before_refuse))
q=cli('oracle-preserve-refused','preserve',repo,payload)
results.append(dict(id='oracle-valid-bundle-missing-ref-preserve',exit=q.returncode,pass_=q.returncode!=0 and 'bundle 未覆盖 ref' in q.stderr and hashes(payload)==before_refuse))

spec=root/'docs/specs/repository-boundaries/project.md';(out/'spec-delta-reviewed.md').write_bytes(spec.read_bytes());links=[]
for i,line in enumerate(spec.read_text(encoding='utf-8').splitlines(),1):
 for target in re.findall(r'\]\(([^)]+)\)',line):
  if '://' not in target:links.append(dict(line=i,target=target,exists=(spec.parent/target.split('#')[0]).exists()))
results.append(dict(id='R1-03-spec-links',links=links,pass_=all(x['exists'] for x in links)))
assert before==protected();assert git(root,'rev-parse','HEAD')==head and git(root,'status','--porcelain')=='';assert all(sha(root/p)==h for p,h in tracked.items())
record=dict(head=head,scratch=str(scratch),protected_before=before,protected_unchanged=True,implementation_unchanged=True,tracked_sha256=tracked,spec_delta_sha256=sha(spec),results=results)
(out/'results.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8')
for row in results:print(json.dumps(row,ensure_ascii=False))
sys.exit(0 if all(r['pass_'] for r in results) else 1)
