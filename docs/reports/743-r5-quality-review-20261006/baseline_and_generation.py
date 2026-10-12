from pathlib import Path
import os,json,hashlib,subprocess,shutil,re
R=Path('D:/autostack/auto-lang');E=Path(__file__).parent
env={**os.environ,'PYTHONUTF8':'1'}
def git(*args):return subprocess.check_output(['git','-c',f'safe.directory={R}','-c','core.excludesfile=','-C',str(R),*args])
head=git('rev-parse','HEAD').decode().strip()
base='ad1fe269c21bc9021220645b52a32301e680f321'
paths=['scripts/acc_inventory.py','scripts/tests/test_acc_inventory.py','docs/plans/archive/743-acc-bootstrap-hir-contract.md','docs/specs/auto-acc/project.md','docs/specs/auto-hir/stage-contract.md','docs/specs/auto-ac/project.md','docs/reports/743-acc-hir-contract/manual-decisions.json','docs/reports/743-acc-hir-contract/source-manifest.json','docs/reports/743-acc-hir-contract/proposed-spec-delta-phase4.md','docs/reports/743-r4-quality-review-20261005/proposed-spec-delta-phase5.md','docs/reports/743-phase4-consumer-fixes/final_assertions.py']
hashes={p:hashlib.sha256((R/p).read_bytes()).hexdigest() for p in paths}
branch='437a6b2f9243e35e5e23ab9ce6f005712aa89f7e'
same={p:hashlib.sha256(git('show',branch+':'+p)).hexdigest()==hashes[p] for p in paths[:2]}
assert all(same.values())
for commit in ('d64dc0f5c','f609e916b','454825ac5','8f2a021d6','27803ef74','638f25df9','9111c21e5'):
    assert subprocess.run(['git','-c',f'safe.directory={R}','-C',str(R),'merge-base','--is-ancestor',commit,head],capture_output=True).returncode==0
manifest={'reviewed_commit':head,'plan_revision':5,'diff_base':base,'tests_executed_snapshot':branch,'implementation_files_identical_to_test_snapshot':same,'hashes':hashes,'category':'A: Python research tools/docs only; no Cargo/native/docgen runs','code_inputs_changed_during_review':False}
(E/'source-manifest.json').write_text(json.dumps(manifest,indent=2),encoding='utf8')
manual=R/'docs/reports/743-acc-hir-contract/manual-decisions.json';before=manual.read_bytes()
results={}
for name in ('gen1','gen2'):
    d=E/name;d.mkdir(exist_ok=False);shutil.copy2(manual,d/manual.name)
    for mode in ('--write','--check'):
        args=[os.sys.executable,'-B',str(R/'scripts/acc_inventory.py'),'--root',str(R),'--output',str(d),mode]
        if mode=='--check':args.append('--require-decisions')
        p=subprocess.run(args,env=env,capture_output=True,text=True,encoding='utf8',timeout=20)
        results[name+mode]={'exit':p.returncode,'stdout':p.stdout,'stderr':p.stderr};assert p.returncode==0
    assert (d/manual.name).read_bytes()==before
def normalized(path):
    v=json.loads(path.read_text(encoding='utf8'));v['source_identity']['head_commit_audit_only']=None
    return v
results['manifest_three_way_equal']=normalized(E/'gen1/source-manifest.json')==normalized(E/'gen2/source-manifest.json')==normalized(R/'docs/reports/743-acc-hir-contract/source-manifest.json')
results['summary_three_way_equal']=(E/'gen1/scan-summary.md').read_bytes()==(E/'gen2/scan-summary.md').read_bytes()==(R/'docs/reports/743-acc-hir-contract/scan-summary.md').read_bytes()
results['manual_bytes_unchanged']=manual.read_bytes()==before
assert all(results[k] for k in ('manifest_three_way_equal','summary_three_way_equal','manual_bytes_unchanged'))
(E/'generation.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf8')
print(json.dumps({'reviewed_commit':head,'tests_identical':same,'generation_controls':{k:results[k] for k in results if not k.startswith('gen')}},indent=2))
