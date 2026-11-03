"""Read-only historical audit for the AutoGraphics 851 milestone; no engine runs."""
from pathlib import Path
import subprocess,json,hashlib,re
ROOT=Path(__file__).resolve().parents[3]
OUT=Path(__file__).resolve().parent
REF='11911a4a6d809bfd515f745b77d00bc8f04f6ba9'
PRE='experimental/autographics-auto-driver/'
def cmd(args,**kw):return subprocess.check_output(args,cwd=ROOT,**kw)
def blob(key,rev=REF):return cmd(['git','show',rev+':'+key])
def batch(keys,rev=REF):
 raw=cmd(['git','cat-file','--batch'],input=('\n'.join(rev+':'+k for k in keys)+'\n').encode());cursor=0;values={}
 for key in keys:
  end=raw.index(b'\n',cursor);parts=raw[cursor:end].decode().split();assert len(parts)==3 and parts[1]=='blob',(key,parts)
  size=int(parts[2]);cursor=end+1;values[key]=raw[cursor:cursor+size];cursor+=size+1
 assert cursor==len(raw)
 return values
main=cmd(['git','rev-parse','HEAD'],text=True).strip()
identity=json.loads(blob(PRE+'research/plan851-identity.json'))
objects=batch(['experimental/'+k for k in identity['files']]);source_bad=[]
for rel,want in identity['files'].items():
 b=objects['experimental/'+rel];p=Path(rel)
 if p.suffix.lower() in identity['text_suffixes'] or p.name.lower() in identity['text_suffixes']:b=b.replace(b'\r\n',b'\n')
 got=hashlib.sha256(b).hexdigest()
 if got!=want:source_bad.append(dict(path=rel,expected=want,actual=got))
treepaths=set(cmd(['git','ls-tree','-r','--name-only',REF,'experimental'],text=True).splitlines());folder_bad=[]
for folder in identity['folders']:
 actual={p[len('experimental/'):] for p in treepaths if p.startswith('experimental/'+folder+'/') and '__pycache__' not in p}
 expected={p for p in identity['files'] if p.startswith(folder+'/')}
 if actual!=expected:folder_bad.append(dict(folder=folder,missing=sorted(expected-actual),extra=sorted(actual-expected)))
projects=[]
for path,b in objects.items():
 if '/samples/' in path and path.endswith('/scene.agscene'):
  d=json.loads(b)
  if not isinstance(d.get('behavior'),dict):continue
  source=d['behavior']['source'];sp=path.rsplit('/',1)[0]+'/scripts/'+source['sha256']+'.at';sb=objects.get(sp) or blob(sp)
  projects.append(dict(scene=path,api=d['behavior']['api'],source_bytes=len(sb),source_sha256=hashlib.sha256(sb).hexdigest(),source_ok=len(sb)==source['byte_length'] and hashlib.sha256(sb).hexdigest()==source['sha256']))
base=json.loads(blob(PRE+'research/baseline.json.txt'));lock=json.loads(blob(PRE+'research/source-lock.json'))['files'];sdkref=base['sdk']['repository_commit'];sdk_bad=[]
for k,b in batch([k for k in lock if k not in ['Cargo.toml','Cargo.lock']],sdkref).items():
 if hashlib.sha256(b).hexdigest()!=lock[k]:sdk_bad.append(k)
toml=blob('Cargo.toml',sdkref).decode();members=[x['path'] for x in base['sdk']['path_sources']]
reduced='[workspace]\nresolver = "2"\nmembers = '+json.dumps(members)+'\n\n'+toml[toml.index('[workspace.dependencies]'):toml.index('[profile.dev]')]
variants={'LF':reduced.encode(),'CRLF':reduced.replace('\r\n','\n').replace('\n','\r\n').encode()};matching=[k for k,b in variants.items() if hashlib.sha256(b).hexdigest()==lock['Cargo.toml']]
if not matching:sdk_bad.append('synthesized Cargo.toml')
if hashlib.sha256(blob(PRE+'research/sdk-Cargo.lock.txt')).hexdigest()!=lock['Cargo.lock']:sdk_bad.append('Cargo.lock')
merge=ROOT/'docs/plans/attachments/851/merge-r4';gates=json.loads((merge/'gates.json.txt').read_text());latest={}
for row in gates:latest[row['name']]=row
gate_bad=[dict(name=k,exit=v['exit'],expected=v['expected'],log=v.get('log')) for k,v in latest.items() if v['exit']!=v['expected']]
missing_logs=[r['log'] for r in latest.values() if r.get('log') and not (merge/r['log']).exists()]
selections=json.loads((merge/'test-selection.json.txt').read_text());totals={}
for family in ['producer-test','consumer-test']:
 text=(merge/selections[family]['log']).read_text(encoding='utf-8-sig',errors='replace')
 chunks=re.split(r'(?=^\s+Running |^\s+Doc-tests )',text,flags=re.M);counts=[]
 for chunk in chunks:
  values=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',chunk)
  if values:counts.append(tuple(map(int,values[-1])))
 sums=[sum(v[i] for v in counts) for i in range(3)];assert sums==selections[family]['totals'],(family,sums)
 totals[family]=dict(totals=sums,completed_binaries=len(counts),log=selections[family]['log'],nested_summary_policy='Last outer libtest summary in each Cargo binary block; asset_feasibility spawns portable_child, which is not another unique suite test.')
reuse=json.loads((merge/'agp-reuse.json.txt').read_text());reuse_bad=[]
for row in reuse['rows']:
 origin=row.get('evidence_origin_commit')
 if origin:
  diff=cmd(['git','diff','--name-only',origin,REF,'--','experimental/autographics-prototype'],text=True).strip()
  if diff:reuse_bad.append(dict(name=row['name'],origin=origin,paths=diff))
agp_diff=cmd(['git','diff','--name-only','6bb85f354e718f6541256f4b3d75ca97af75b8c4',REF,'--','experimental/autographics-prototype'],text=True).strip()
raw_manifest=json.loads((merge/'raw-files.json.txt').read_text());raw_bad=[];raw_modes={'git':0,'working-copy':0,'LF-normalized':0}
for rel,want in raw_manifest.items():
 p=merge/rel;rp=p.relative_to(ROOT).as_posix();r=subprocess.run(['git','show',main+':'+rp],cwd=ROOT,capture_output=True)
 if r.returncode==0:b=r.stdout;mode='git'
 else:b=p.read_bytes();mode='working-copy'
 if hashlib.sha256(b).hexdigest()!=want:
  if hashlib.sha256(b.replace(b'\r\n',b'\n')).hexdigest()==want:mode='LF-normalized'
  else:raw_bad.append(dict(path=rp,expected=want,actual=hashlib.sha256(b).hexdigest()));continue
 raw_modes[mode]+=1
artifacts=json.loads((merge/'motion-artifacts.json.txt').read_text());binaries=[]
for family,row in artifacts.items():
 p=Path(row['retained']);binaries.append(dict(family=family,path=p.as_posix(),expected_sha256=row['sha256'],exists=p.is_file(),hash_ok=p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest()==row['sha256']))
native=[]
for family,row in artifacts.items():
 folder=merge/row['family'];cases=json.loads((folder/'results.json').read_text());details=[]
 for case in cases:
  name=case['case'];report=json.loads((folder/name/'report.json').read_text());positive=name.startswith(('maze2d','corridor3d'));pixels=[x for x in report.get('spatial_records',[]) if x['case']=='spatial-pixels'];traces=[x for x in report.get('spatial_records',[]) if x['case']=='motion-trace']
  ok=case['binary_sha256']==row['sha256'] and report['passed']==positive and case['exit']==(0 if positive else 1)
  if positive:ok=ok and report['spatial_phase']==14 and len(pixels)==3 and all(x['passed'] and all(c['passed'] for c in x['checks']) for x in pixels) and len(traces)==4 and all(x['api']=='autographics.behavior/5' and x['source']==case['user_sha256'] for x in traces)
  reopened=folder/name/'reopened.json';reopen_ok=None
  if positive and name.endswith(('-short','-bind')):
   d=json.loads(reopened.read_text());reopen_ok=d['passed'] and d['spatial_phase']==14;ok=ok and reopen_ok
  if name in ['transparent','missing-part','wrong-asset']:ok=ok and report['spatial_phase']==5 and 'spatial pixels final-content failure' in report.get('error','')
  if name in ['missing-code','bad-code','bad-api','missing-glb']:ok=ok and report['spatial_phase']==0 and bool(report.get('error'))
  if name=='partial':ok=ok and report['spatial_phase']<14 and bool(report.get('error'))
  details.append(dict(case=name,positive=positive,passed=report['passed'],phase=report.get('spatial_phase'),exit=case['exit'],binding_and_assertions_ok=bool(ok),reopen_ok=reopen_ok,error=report.get('error')))
 native.append(dict(family=family,cases=details,all_ok=all(x['binding_and_assertions_ok'] for x in details)))
owner_controls=[]
for prefix in ['', 'consumer-']:
 rows=json.loads((merge/(prefix+'motion-owners/results.json')).read_text())
 ok=len(rows)==7 and all(x['exit']==101 and x['oracle'] for x in rows[:-1]) and rows[-1]['case']=='restored' and rows[-1]['exit']==0
 owner_controls.append(dict(family=prefix+'motion-owners',cases=len(rows),all_ok=bool(ok)))
ancestry=[]
for short in ['ddbde3e62857','764e87559','77302d855','ebeab12d2888','8b62db5cba54','c1c7cfea5c9c','bf22db346520','dd7076a7f40b','6bb85f354e71',REF]:
 full=cmd(['git','rev-parse',short],text=True).strip();r=subprocess.run(['git','merge-base','--is-ancestor',full,REF],cwd=ROOT);ancestry.append(dict(commit=full,is_ancestor=r.returncode==0))
route=['docs/design/45-autographics-wgpu-engine-and-game-prototypes.md','docs/design/strategy/autographics-forge-v06.md','docs/roadmap.md','docs/roadmap-v0.6.md','docs/roadmap-future.md'];broken=[]
for rel in route:
 p=ROOT/rel
 for n,l in enumerate(p.read_text(encoding='utf-8-sig').splitlines(),1):
  for target in re.findall(r'\]\(([^)]+)\)',l):
   if '://' in target or target.startswith('#'):continue
   q=(p.parent/target.split('#',1)[0]).resolve()
   if not q.is_file():broken.append(dict(file=rel,line=n,target=target))
plans=[]
for n in [829,830,833,834,835,836,837,838,839,840,845,846,848,849,851]:
 p=next((ROOT/'docs/plans/archive').glob(f'{n}-*.md'));b=p.read_bytes();m=b.decode('utf-8-sig').split('---',2)[1]
 def field(k):
  v=re.search(r'^'+k+r':\s*(.+)$',m,re.M);return v.group(1) if v else None
 plans.append(dict(plan=n,path=p.relative_to(ROOT).as_posix(),sha256=hashlib.sha256(b).hexdigest(),status=field('status'),revision=field('plan_revision'),current_step=field('current_step'),total_steps=field('total_steps'),completion_kind=field('completion_kind')))
inputs=[]
for rel in route+['docs/specs/auto-graphics/overview.md','docs/specs/auto-graphics/bounded-motion-profile.md','docs/specs/auto-graphics/auto-source-driver.md','docs/specs/auto-graphics/auto-3d-game-profile.md','docs/specs/auto-graphics/scene-assets.md','docs/specs/auto-graphics/scene-document.md','docs/specs/auto-graphics/behavior-api.md']:
 b=(ROOT/rel).read_bytes();inputs.append(dict(path=rel,sha256=hashlib.sha256(b).hexdigest()))
large=[]
for p,b in objects.items():
 if '/src/' in p and p.endswith('.rs') and len(b.splitlines())>=750:large.append(dict(path=p,lines=len(b.splitlines())))
record=dict(date='2026-10-26 Asia/Shanghai',review_kind='Historical milestone evidence audit; not a reopened Plan final review',source_ref=REF,main_ref=main,plans=plans,inputs=inputs,source_identity=dict(manifest=PRE+'research/plan851-identity.json',entries=len(identity['files']),mismatches=source_bad,folder_mismatches=folder_bad),sdk_reconstruction=dict(commit=sdkref,files=len(lock),root_manifest_newline_variants_matching=matching,mismatches=sdk_bad),projects=projects,gate_receipts=dict(attempts=len(gates),latest_groups=len(latest),failed_latest=gate_bad,missing_logs=missing_logs),test_log_totals=totals,agp_reuse=dict(slots=len(reuse['rows']),formal849_to851_unchanged=not agp_diff,origin_differences=reuse_bad),raw_receipts=dict(entries=len(raw_manifest),verified_modes=raw_modes,mismatches=raw_bad),retained_motion_binaries=binaries,motion_native_reports=native,motion_owner_controls=owner_controls,source_ancestry=ancestry,route_broken_links=broken,large_source_files=large,limitations=['No new Cargo/build/native/GPU or performance runs. Source and existing bound evidence inspected.','Checksums/log assertions do not independently certify fresh runtime behavior; historical review and frozen merge evidence reused at stated identities.','No archived Plan reopened; no source, dependency, canonical Spec or ledger changed.'])
(OUT/'audit.json.txt').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
assert len(identity['files'])==430 and not source_bad and not folder_bad
assert len(lock)==681 and not sdk_bad
assert len(projects)==10 and all(x['source_ok'] for x in projects)
assert len(latest)==85 and not gate_bad and not missing_logs
assert not agp_diff and len(reuse['rows'])==23 and not reuse_bad
assert not raw_bad and all(x['hash_ok'] for x in binaries)
assert all(x['all_ok'] for x in native+owner_controls)
assert all(x['is_ancestor'] for x in ancestry)
print(json.dumps(dict(source_mismatches=len(source_bad),folder_mismatches=len(folder_bad),sdk_files=len(lock),sdk_mismatches=sdk_bad,projects=len(projects),projects_ok=all(p['source_ok'] for p in projects),latest_gate_groups=len(latest),gate_failures=gate_bad,missing_logs=missing_logs,tests=totals,raw_entries=len(raw_manifest),raw_bad=raw_bad,binaries=binaries,native_all_ok=all(x['all_ok'] for x in native),ancestry_ok=all(a['is_ancestor'] for a in ancestry),route_broken_links=len(broken)),ensure_ascii=False))
