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
