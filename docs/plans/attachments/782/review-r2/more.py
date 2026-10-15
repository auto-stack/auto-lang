import pathlib, subprocess, json
R=pathlib.Path('D:/autostack/.wt/lang-782/auto-lang'); H=pathlib.Path(__file__).resolve().parent
T=R/'experimental/ac-core/target/debug/auto-ac-prototype.exe'; L=R/'experimental/ac-core/runtime/text/target/release/ac_text_runtime.lib'
cases={n:(H/(n+'.at')).read_text() for n in ['partial_reown','nested_reown']}
cases['field_live_control']=cases['partial_reown'].replace('extract(r.move)', '"cc".to(Str)')
cases['whole_rebuild_control']='''type T { a Str; b Str }
fn rebuild(move x T) T { return x.move }
fn main() int {
 var r T = T { a: "aa", b: "bbb" }
 r = rebuild(r.move)
 return r.b.len()
}
'''
out={}
def run(a):
 p=subprocess.run([str(x) for x in a],cwd=R,capture_output=True,text=True,timeout=60);return dict(code=p.returncode,stdout=p.stdout,stderr=p.stderr)
for n,s in cases.items():
 p=H/(n+'.at');p.write_text(s);e=R/('experimental/ac-core/target/review-782-r1/'+n+'.exe')
 b=run([T,'build-source',p,'--source-profile','auto.source.text-record.draft','--entry','main','--capability','ac.runtime.text.v1','--capability','hir.test.trace.v1','--support-lib',L,'--output',e]);out[n]={'build':b,'run':run([e]) if b['code']==0 else None};print(n,json.dumps(out[n]))
(H/'more-results.json').write_text(json.dumps(out,indent=2))
