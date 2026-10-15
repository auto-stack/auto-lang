"""Check corpus-gate failure propagation with a missing required input in an owned index copy."""
import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
root=Path('D:/autostack/.wt/lang-780/auto-lang');out=Path(__file__).resolve().parent
def git(*a,env=None):return subprocess.check_output(['git','-C',str(root),*a],text=True,env=env).strip()
index=Path(git('rev-parse','--git-path','index')).resolve()
before=hashlib.sha256(index.read_bytes()).hexdigest()
owned=root/'scratch/review780-r1/gate-index';owned.parent.mkdir(parents=True,exist_ok=True)
shutil.copyfile(index,owned)
env=dict(os.environ);env['GIT_INDEX_FILE']=str(owned)
source=next(x for x in git('ls-files','*.at').splitlines() if x.endswith('/pac.at'))
blob=git('rev-parse','HEAD:'+source)
missing='scratch/review780-r1/required-missing-pac.at'
assert not (root/missing).exists()
git('update-index','--add','--cacheinfo','100644,'+blob+','+missing,env=env)
assert missing in git('ls-files','*.at',env=env).splitlines()
cmd=['cargo','test','-p','auto-man','--lib','project_model_corpus_comparison_v02','--locked','--','--nocapture']
with (out/'gate-negative.log').open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=180)
after=hashlib.sha256(index.read_bytes()).hexdigest();assert before==after
record={'command':cmd,'temporary_index':str(owned),'missing_input':missing,'oracle_blob_from':source,
        'original_index_unchanged':before==after,'expected':'nonzero (required managed input missing)',
        'actual_exit':q.returncode,'contract_pass':q.returncode!=0}
(out/'gate-negative.json').write_text(json.dumps(record,ensure_ascii=False)+'\n',encoding='utf-8')
print(json.dumps(record,ensure_ascii=False));print('\n'.join((out/'gate-negative.log').read_text().splitlines()[-8:]))
sys.exit(0 if record['contract_pass'] else 1)
