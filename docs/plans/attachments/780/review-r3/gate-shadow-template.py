"""Unexpected malformed required manifest must not be treated like an approved bad fixture."""
import hashlib,json,os,shutil,subprocess,sys
from pathlib import Path
root=Path('D:/autostack/.wt/lang-780/auto-lang');out=Path(__file__).resolve().parent
def git(*a,env=None):return subprocess.check_output(['git','-C',str(root),*a],text=True,env=env).strip()
index=Path(git('rev-parse','--git-path','index')).resolve()
before=hashlib.sha256(index.read_bytes()).hexdigest()
scratch=root/'scratch/review780-r2';scratch.mkdir(parents=True,exist_ok=True)
owned=scratch/'malformed-index';shutil.copyfile(index,owned)
env=dict(os.environ);env['GIT_INDEX_FILE']=str(owned)
source=next(x for x in git('ls-files','*.at').splitlines() if x.endswith('/pac.at'))
blob=git('rev-parse','HEAD:'+source)
relative='scratch/review780-r3/shadow/fixtures/dynamic-expressions.pac.at';input_path=root/relative
input_path.parent.mkdir(parents=True,exist_ok=True)
input_path.write_text('name: unknown_symbol\n',encoding='utf-8')
git('update-index','--add','--cacheinfo','100644,'+blob+','+relative,env=env)
cmd=['cargo','test','-p','auto-man','--lib','project_model_corpus_comparison_v02','--locked','--','--nocapture']
with (out/'gate-shadow-template.log').open('w',encoding='utf-8') as f:q=subprocess.run(cmd,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT,timeout=120)
assert hashlib.sha256(index.read_bytes()).hexdigest()==before
record={'head':git('rev-parse','HEAD'),'input_path':relative,'input':input_path.read_text(),
 'approved_bad_fixture':False,'original_index_unchanged':True,'command':cmd,
 'expected':'nonzero: new required input is malformed and not an approved bad fixture',
 'actual_exit':q.returncode,'contract_pass':q.returncode!=0}
(out/'gate-shadow-template.json').write_text(json.dumps(record,ensure_ascii=False)+'\n',encoding='utf-8')
print(json.dumps(record,ensure_ascii=False));print('\n'.join((out/'gate-shadow-template.log').read_text().splitlines()[-8:]))
sys.exit(0 if record['contract_pass'] else 1)
