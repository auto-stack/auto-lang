import pathlib, subprocess, json
R=pathlib.Path('D:/autostack/.wt/lang-782/auto-lang'); H=pathlib.Path(__file__).resolve().parent
T=R/'experimental/ac-core/target/debug/auto-ac-prototype.exe'; L=R/'experimental/ac-core/runtime/text/target/release/ac_text_runtime.lib'
base='''type T { s Str; n int }
fn yes() bool { return true }
fn cond(i int) bool { return i == 0 }
fn check(t T) bool { return t.n > 0 }
fn main() int {
 let a T = T { s: "abc", n: 1 }
 var i int = 0
 loop {
  if EXPR { i = i + 1 } else { i = i + 1 }
  if i == 2 { break }
 }
 return a.s.len()
}
'''
cases={'nested_and':base.replace('EXPR','yes() && (cond(i) && check(a.clone()))'), 'nested_or':base.replace('EXPR','yes() && (cond(i) || check(a.clone()))')}
cases['nested_or_take_skip']=base.replace('EXPR','yes() && (!cond(i) || check(a.clone()))')
cases['outer_skip']=base.replace('EXPR','cond(i) && (yes() && check(a.clone()))')
out={}
def run(a):
 p=subprocess.run([str(x) for x in a],cwd=R,capture_output=True,text=True,timeout=60);return dict(code=p.returncode,stdout=p.stdout,stderr=p.stderr)
for n,s in cases.items():
 p=H/(n+'.at');p.write_text(s);e=R/('experimental/ac-core/target/review-782-r3/'+n+'.exe')
 b=run([T,'build-source',p,'--source-profile','auto.source.text-record.draft','--entry','main','--capability','ac.runtime.text.v1','--capability','hir.test.trace.v1','--support-lib',L,'--output',e]); out[n]={'build':b,'run':run([e]) if b['code']==0 else None};print(n,json.dumps(out[n]))
(H/'extra-results.json').write_text(json.dumps(out,indent=2))
