import hashlib, json, os, subprocess
from pathlib import Path
ROOT = Path('D:/autostack/.wt/lang-778/auto-lang')
OUT = Path(__file__).resolve().parent
def git(*args):
    return subprocess.check_output(['git', '-C', str(ROOT), *args], text=True).strip()
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
before = {p: sha(ROOT/p) for p in git('ls-files').splitlines() if (ROOT/p).is_file()}
head = git('rev-parse', 'HEAD')
scratch = ROOT/'scratch/review778-r2'
(scratch/'tests').mkdir(parents=True, exist_ok=True)
(scratch/'Cargo.toml').write_text('[package]\nname="review778-r2"\nversion="0.1.0"\nedition="2021"\n[dependencies]\nauto-lang={path="../../crates/auto-lang"}\nui-dialect-consumer={path="../../test/fixtures/modularity/ui-dialect-consumer"}\n[workspace]\n', encoding='utf-8')
source = (OUT.parent/'review-r1/independent.rs').read_text(encoding='utf-8')
source = source.replace('assert!(result.is_err());\n    { let mut b', 'assert!(result.is_err());\n    assert!(!ParserBridge::new(&mut p).in_fn_body(), "context must be restored BEFORE harness checkpoint restore");\n    { let mut b')
source = source.replace('vec![false,false], "Err must restore', 'vec![false,true,false], "Err must restore')
source += '''
#[test]
fn frozen_optional_type_and_initializer() {
    for input in ["model { count }", "model { count\\nother int = 2 }", "model { count int }"] {
        let probe=Arc::new(Probe::default());
        let result=Parser::from(input).with_session(CompilerSession::ui())
            .with_dialect(Box::new(ExternalModelDialect::new(probe.clone())),DialectPriority::Override).parse();
        println!("optional input={input:?}, result={result:?}");
        assert!(result.is_ok(), "frozen name [type] [= expr] subset: {input}");
        assert_eq!(probe.accepts.load(Ordering::SeqCst),1);
    }
}
'''
(OUT/'independent.rs').write_text(source, encoding='utf-8')
(scratch/'tests/independent.rs').write_text(source, encoding='utf-8')
env = dict(os.environ)
env['CARGO_TARGET_DIR'] = str(ROOT/'test/fixtures/modularity/ui-dialect-consumer/target')
jobs = [('external.log',['cargo','test','--manifest-path','test/fixtures/modularity/ui-dialect-consumer/Cargo.toml','--locked']),
        ('independent.log',['cargo','test','--manifest-path',str(scratch/'Cargo.toml'),'--test','independent','--','--nocapture']),
        ('scoped.log',['cargo','nextest','run','-p','auto-lang','--lib','--features','ui-iced,test-cases','-E','test(parser::tests) | test(dialect) | test(model) | test(widget)']),
        ('check.log',['cargo','check','-p','auto-lang','--locked'])]
results=[]
for name, cmd in jobs:
    with (OUT/name).open('w',encoding='utf-8') as log:
        q=subprocess.run(cmd,cwd=ROOT,env=env if name in ['external.log','independent.log'] else None,stdout=log,stderr=subprocess.STDOUT,timeout=300)
    r={'command':cmd,'file':name,'exit_code':q.returncode,'sha256':sha(OUT/name)}
    results.append(r)
    print(json.dumps(r),flush=True)
after={p:sha(ROOT/p) for p in before}
dep=ROOT.parent/'auto-down'
fp={'head_before':head,'head_after':git('rev-parse','HEAD'),'branch':git('branch','--show-current'),'status':git('status','--porcelain'),'inputs_unchanged':before==after,'tracked_sha256':before,'dependency_head':subprocess.check_output(['git','-C',str(dep),'rev-parse','HEAD'],text=True).strip(),'spec_sha256':sha(Path('D:/autostack/auto-lang/docs/specs/auto-lang/frontend/overview.md')),'locks':{str(p.relative_to(ROOT)):sha(p) for p in [ROOT/'Cargo.lock',ROOT/'test/fixtures/modularity/ui-dialect-consumer/Cargo.lock'] if p.exists()}}
(OUT/'fingerprints.json').write_text(json.dumps(fp)+'\n',encoding='utf-8')
(OUT/'runs.jsonl').write_text('\n'.join(json.dumps(r) for r in results)+'\n',encoding='utf-8')
print(json.dumps({'inputs_unchanged':before==after,'status':fp['status']}),flush=True)
