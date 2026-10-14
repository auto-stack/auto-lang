"""Independent combinations for the new file-identity guard, only owned temp inputs."""
import json, os, shutil, stat, sys, tempfile, subprocess
from pathlib import Path
from replay import cli, git, commit, digest, summarize, cargo_oracle
IMPL=Path('D:/autostack/.wt/lang-777/auto-lang')
SCRIPT=IMPL/'scripts/modularity-audit.py'
OUT=Path(__file__).resolve().parent
container=Path(tempfile.mkdtemp(prefix='plan777-r4-identity-')).resolve()
results=[]
try:
    root=container/'repo'
    shutil.copytree(IMPL/'test/fixtures/modularity/audit',root)
    git(root,'init','-q')
    commit(root,'tracked assets and source')
    oracle=cargo_oracle(root)
    assert oracle['exit_code']==0,oracle
    generated=subprocess.run(['cargo','generate-lockfile','--offline'],cwd=root,capture_output=True,text=True,timeout=40)
    assert generated.returncode==0 and (root/'Cargo.lock').exists(),generated.stderr
    for label,rel in [('valid-untracked-lock','Cargo.lock'),('tracked-asset','assets/fixture-assets/sample.txt')]:
        original=root/rel
        before=digest(original)
        alias=container/(label+'-report.json')
        os.link(original,alias)
        assert alias.samefile(original)
        run=cli(SCRIPT,root,'--mode','observe','--output',str(alias))
        results.append({'id':label,'relative_input':rel,'samefile':True,'before_sha256':before,
                        'after_sha256':digest(original),'actual':summarize(run),
                        'contract_pass':run['exit_code']==3 and before==digest(original)})
    destination=container/'existing-independent-report.json'
    destination.write_text('previous independent output',encoding='utf-8')
    before=digest(root/'crates/fixture-lang/src/lib.rs')
    run=cli(SCRIPT,root,'--mode','observe','--output',str(destination))
    results.append({'id':'existing-independent-report','actual':summarize(run),
                    'contract_pass':run['exit_code']==0 and json.loads(destination.read_text())['schema'].startswith('modularity-audit')
                    and before==digest(root/'crates/fixture-lang/src/lib.rs')})
finally:
    assert container.parent==Path(tempfile.gettempdir()).resolve()
    def clear_readonly(function,path,exc):
        assert Path(path).resolve().is_relative_to(container)
        os.chmod(path,Path(path).stat().st_mode|stat.S_IWRITE)
        function(path)
    shutil.rmtree(container,onexc=clear_readonly)
(OUT/'identity-combinations.jsonl').write_text(json.dumps({'head':git(IMPL,'rev-parse','HEAD'),'oracle':oracle,'cases':results},ensure_ascii=False)+'\n',encoding='utf-8')
for r in results: print(json.dumps({'id':r['id'],'exit_code':r['actual']['exit_code'],'contract_pass':r['contract_pass']}))
sys.exit(0 if all(r['contract_pass'] for r in results) else 1)
