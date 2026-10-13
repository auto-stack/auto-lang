from pathlib import Path
import contextlib,io,importlib.util,json,re,shutil,sys

repo=Path(sys.argv[1]).resolve();out=Path(sys.argv[2]).resolve();out.mkdir(exist_ok=False)
fixture=out/'repo';rel='docs/plans/archive/741-ac-hir-native-core.md'
def copy(rel):
    dest=fixture/rel;dest.parent.mkdir(parents=True,exist_ok=True)
    shutil.copyfile(repo/rel,dest)
for f in [rel,'experimental/ac-core/README.md','docs/specs/auto-ac/plans.md','docs/specs/auto-hir/plans.md','.autoos/specs.json']:
    copy(f)
plan=fixture/rel;original=plan.read_text(encoding='utf8')
spec=importlib.util.spec_from_file_location('review_validator',repo/'docs/reports/741-phase5-link-cleanup/final_assertions.py')
mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);mod.REPO=fixture
body=re.sub(r'`[^`]*`','',mod.FENCE.sub('',original))
for m in mod.LINK.finditer(body):
    target=m.group(1)
    if '://' in target or target.startswith('/'):continue
    source=(repo/rel).parent.joinpath(target).resolve()
    copy(source.relative_to(repo))
ledger=json.loads((fixture/'.autoos/specs.json').read_text(encoding='utf8'))
for sec in ledger['sections']:
    for it in sec.get('items',[]):
        if it.get('id','').startswith('P741-') and it['file']!=rel:copy(it['file'])
def run(location):
    saved=sys.argv;sys.argv=['final_assertions.py','--plan',str(plan),'--location',location]
    text=io.StringIO();code=0
    try:
        with contextlib.redirect_stdout(text):mod.main()
    except SystemExit as e:code=e.code
    finally:sys.argv=saved
    return {'exit':code,'stdout':text.getvalue()}
results={}
good=original.replace('current_step: 34','current_step: 35',1)
for name,value in {
    'valid_35_35_control':good,
    'actual_34_35':original,
    'current_zero':good.replace('current_step: 35','current_step: 0',1),
    'total_999':good.replace('total_steps: 35','total_steps: 999',1),
    'missing_current':good.replace('current_step: 35\n','',1),
    'broken_link_control':good+'\n[missing](../../reports/review-fixture-missing.md)\n',
}.items():
    plan.write_text(value,encoding='utf8');results[name]=run('archive')
# Exercise the documented active mode with legitimate active navigation.
active=fixture/'docs/plans/741-ac-hir-native-core.md'
plan.unlink();plan=active
active.write_text(good.replace('status: archived','status: executing',1),encoding='utf8')
for rel in ['experimental/ac-core/README.md','docs/specs/auto-ac/plans.md','docs/specs/auto-hir/plans.md']:
    p=fixture/rel;s=p.read_text(encoding='utf8');s=s.replace('plans/archive/741-ac-hir-native-core.md','plans/741-ac-hir-native-core.md');p.write_text(s,encoding='utf8')
for sec in ledger['sections']:
    for it in sec.get('items',[]):
        if it.get('id')=='P741-7':it['file']='docs/plans/741-ac-hir-native-core.md'
(fixture/'.autoos/specs.json').write_text(json.dumps(ledger),encoding='utf8')
results['active']=run('active')
broken=[]
for m in mod.LINK.finditer(body):
    target=m.group(1)
    if '://' in target or target.startswith('/'):continue
    if not active.parent.joinpath(target).resolve().exists():broken.append(target)
results['active']['actual_parent_broken_links']=broken
(out/'results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2)+'\n',encoding='utf8')
for name,result in results.items():print(name,'exit',result['exit'],'actual_broken',len(result.get('actual_parent_broken_links',[])))
