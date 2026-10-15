import pathlib,subprocess,json
R=pathlib.Path('D:/autostack/.wt/lang-782/auto-lang');H=pathlib.Path(__file__).resolve().parent
T=R/'experimental/ac-core/target/debug/auto-ac-prototype.exe';L=R/'experimental/ac-core/runtime/text/target/release/ac_text_runtime.lib'
base='''type T { s Str; n int }
fn yes() bool { return true }
fn cond(i int) bool { return i == 0 }
fn check(t T) bool { return t.n > 0 }
fn fresh() T { return T { s: "abc", n: 1 } }
fn observe(b bool, s str) int { return s.len() }
fn main() int {
 let a T = T { s: "abc", n: 1 }
 var i int = 0
 loop {
  let z = observe(yes() && (cond(i) && check(VALUE)), "z".to(Str).to(str))
  i = i + 1
  if i == 2 { break }
 }
 return a.s.len()
}
'''
cases={}
for name,v in [('clone','a.clone()'),('ctor','T { s: "abc", n: 1 }'),('call','fresh()')]:
 for fail in [None,3,4]:cases[name+('_normal' if fail is None else '_fail'+str(fail))]=(base.replace('VALUE',v),fail)
for field,ty,expr in [('s','Str','x.s.to(Str)'),('i','I','x.i.clone()'),('n','int','x.n')]:
 for same in [True,False]:
  src='''type I { t Str }
type T { s Str; i I; n int }
fn extract(move x T) RET { return EXPR }
fn main() int {
 var a T = T { s: "a", i: I { t: "b" }, n: 1 }
 STMT
 return 0
}
'''.replace('RET',ty).replace('EXPR',expr)
  stmt='a.'+field+' = extract(a.move)' if same else 'let k = extract(a.move)\n a.'+field+' = '+{'s':'"z".to(Str)','i':'I { t: "z" }','n':'4'}[field]
  cases['field_'+field+('_same' if same else '_prior')]=(src.replace('STMT',stmt),None)
def run(args):
 p=subprocess.run([str(a) for a in args],cwd=R,capture_output=True,text=True,timeout=60);return dict(code=p.returncode,stdout=p.stdout,stderr=p.stderr)
results={}
for n,(s,f) in cases.items():
 p=H/(n+'.at');p.write_text(s);e=R/('experimental/ac-core/target/review-782-r3/'+n+'.exe')
 args=[T,'build-source',p,'--source-profile','auto.source.text-record.draft','--entry','main','--capability','ac.runtime.text.v1','--capability','hir.test.trace.v1','--support-lib',L,'--output',e]
 if f is not None:args+=['--rt-fail-nth-alloc',f]
 b=run(args);r=run([e]) if b['code']==0 else None
 if n.startswith('field_'):ok=b['code']==1 and 'verify.use-after-move' in b['stderr']
 elif f is None:ok=b['code']==0 and r['code']==3 and 'FREE-INVALID' not in r['stderr'] and r['stderr'].count('FREE id=')==4
 else:ok=b['code']==0 and r['code']==70 and 'FREE-INVALID' not in r['stderr'] and r['stderr'].count('FREE id=')==(1 if f==3 else 2)
 results[n]={'build':b,'run':r,'oracle':ok};print(n,ok,json.dumps(r or b))
(H/'matrix-results.json').write_text(json.dumps(results,indent=2))
assert all(x['oracle'] for x in results.values())

