from pathlib import Path
import os,sys,subprocess,json,importlib.util,re,tempfile,io,contextlib
R=Path(sys.argv[1]).resolve();E=Path(sys.argv[2]).resolve();E.mkdir(exist_ok=True)
s=importlib.util.spec_from_file_location('p741_current',R/'docs/reports/741-phase7-lifecycle-fixture-nav/fixture_tests.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
text=(R/'docs/plans/archive/741-ac-hir-native-core.md').read_text(encoding='utf8');results={}
with tempfile.TemporaryDirectory(prefix='p741r8-independent-') as td:
 root=Path(td)
 for mode in ['archive','active']:
  base=text if mode=='archive' else m.convert_to_active(m.set_fm(text,'status','executing'))
  for name in ['valid','stale-module-nav','missing-readme-link','wrong-module-target','wrong-readme-target','wrong-ledger-target']:
   if mode=='archive':repo=m.build_nav_sandbox(root/(mode+'-'+name),base,name=='stale-module-nav');plan=repo/'docs/plans/archive'/m.PLAN_NAME
   else:
    repo=m.build_active_control_repo(root,mode+'-'+name,base);plan=repo/'docs/plans'/m.PLAN_NAME
    if name=='stale-module-nav':
     p=repo/'docs/specs/auto-ac/plans.md';p.write_text(p.read_text(encoding='utf8').replace('../../plans/741-ac-hir-native-core.md','../../plans/archive/741-ac-hir-native-core.md'),encoding='utf8')
   prefix='docs/plans/archive/' if mode=='archive' else 'docs/plans/'
   wrong=repo/prefix/'743-existing-other-plan.md';wrong.parent.mkdir(parents=True,exist_ok=True);wrong.write_text('other existing Plan',encoding='utf8')
   if name=='missing-readme-link':(repo/'experimental/ac-core/README.md').write_text('# no required plan reference',encoding='utf8')
   if name=='wrong-module-target':
    p=repo/'docs/specs/auto-ac/plans.md';p.write_text(p.read_text(encoding='utf8').replace('741-ac-hir-native-core.md)','743-existing-other-plan.md)'),encoding='utf8')
   if name=='wrong-readme-target':(repo/'experimental/ac-core/README.md').write_text('[741-ac-hir-native-core.md](../../'+prefix+'743-existing-other-plan.md)',encoding='utf8')
   if name=='wrong-ledger-target':
    p=repo/'.autoos/specs.json';d=json.loads(p.read_text(encoding='utf8'));d['sections'][0]['items'][0]['file']=prefix+'743-existing-other-plan.md';p.write_text(json.dumps(d),encoding='utf8')
   code,log=m.run_assert(plan,mode,repo);want=0 if name=='valid' else 1;assert code==want,(mode,name,code,log)
   if name=='missing-readme-link':assert 'missing required 741 plan reference' in log
   if name.startswith('wrong-'):assert 'does not match the canonical plan' in log
   results[mode+'-'+name]={'exit':code,'output':log};print(mode,name,'exit',code,'PASS')
 for label in ['executing-partial','execution_done-complete','reviewed-complete']:
  a=m.convert_to_active(m.set_fm(text,'status',label.split('-')[0]))
  if label.startswith('executing'):
   a=re.sub(r'^- \[x\] \*\*T-44\*\*','- [ ] **T-44**',a,flags=re.M);a=m.set_fm(a,'current_step','43')
  repo=m.build_active_control_repo(root,label,a);old=m.REPO;m.REPO=repo
  try:lines=m.run_matrix('active ('+label+')',a,root/(label+'-matrix'))
  finally:m.REPO=old
  assert len(lines)==12 and all(x.startswith('PASS') for x in lines),(label,lines)
  results[label]={'executed':len(lines),'lines':lines};print(label,'12/12 PASS')
 # In-memory review-only fault: the actual stale-navigation case is made to return 0.
 # This verifies the fixture's own failure propagation without editing product files.
 original=m.run_assert
 def wrong_acceptance(plan,location,repo):
  if 'nav-stale' in str(plan):return 0,'review-only forced acceptance of forgotten navigation'
  return original(plan,location,repo)
 m.run_assert=wrong_acceptance;prior_argv=sys.argv;sys.argv=[str(R/'docs/reports/741-phase7-lifecycle-fixture-nav/fixture_tests.py'),'--source','archive'];capture=io.StringIO();rc=None
 try:
  with contextlib.redirect_stdout(capture):
   try:m.main()
   except SystemExit as x:rc=x.code
 finally:m.run_assert=original;sys.argv=prior_argv
 assert rc==1 and 'FAIL  nav-controls-forgotten-nav-update' in capture.getvalue()
 results['fixture-failure-propagation']={'exit':rc,'output':capture.getvalue()};print('Fixture failure propagation: exit1 PASS')
(E/'independent-results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf8')
log=(E/'fixture-real-archive.txt').read_text(encoding='utf8');n=sum(x.startswith('PASS ') for x in log.splitlines());assert n==13 and '13 case(s) executed' in log;print('Actual archive count: 13 executed / 13 reported PASS')
