import pathlib, subprocess, json, hashlib

ROOT = pathlib.Path('D:/autostack/.wt/lang-782/auto-lang')
HERE = pathlib.Path(__file__).resolve().parent
OUT = ROOT / 'experimental/ac-core/target/review-782-r3'
OUT.mkdir(parents=True, exist_ok=True)
TOOL = ROOT / 'experimental/ac-core/target/debug/auto-ac-prototype.exe'
LIB = ROOT / 'experimental/ac-core/runtime/text/target/release/ac_text_runtime.lib'
CASES = {
'dynamic_skip': '''type Tok { text Str; kind int }
fn yes() bool { return false }
fn check(t Tok) bool { return t.kind > 0 }
fn main() int {
 let a Tok = Tok { text: "a", kind: 1 }
 if yes() && check(a.clone()) { return 100 }
 return a.text.len()
}
''',
'dynamic_take': '''type Tok { text Str; kind int }
fn yes() bool { return true }
fn check(t Tok) bool { return t.kind > 0 }
fn main() int {
 let a Tok = Tok { text: "a", kind: 1 }
 if yes() && check(a.clone()) { return a.text.len() }
 return 100
}
''',
'field_copy': '''type Inner { s Str; n int }
type Outer { i Inner; n int }
fn main() int {
 let a Outer = Outer { i: Inner { s: "abc", n: 1 }, n: 2 }
 let b Inner = a.i
 return b.s.len() + a.i.s.len()
}
''',
'mixed_temp_order': '''type T { s Str }
fn f(a T, b str) int { return a.s.len() + b.len() }
fn main() int { return f(T { s: "first" }, "second".to(Str)) }
''',
'field_clone': '''type Inner { s Str; n int }
type Outer { i Inner; n int }
fn main() int {
 let a Outer = Outer { i: Inner { s: "abc", n: 1 }, n: 2 }
 let b Inner = a.i.clone()
 return b.s.len() + a.i.s.len()
}
''',
'move_return': '''type T { s Str }
fn consume(move t T) T { return t.move }
fn main() int {
 var a T = T { s: "abc" }
 a = consume(a.move)
 return a.s.len()
}
''',
'move_fresh': '''type T { s Str }
fn consume(move t T) T { return t.move }
fn main() int {
 let a T = T { s: "abc" }
 let b T = consume(a.move)
 return b.s.len()
}
''',
'short_loop': '''type Tok { text Str; kind int }
fn check(t Tok) bool { return t.kind > 0 }
fn cond(i int) bool { return i == 0 }
fn main() int {
 let a Tok = Tok { text: "a", kind: 1 }
 var i int = 0
 loop {
  if cond(i) && check(a.clone()) { i = i + 1 } else { i = i + 1 }
  if i == 2 { break }
 }
 return a.text.len()
}
''',
'plain_temp_order': '''type T { s Str }
fn f(a T, b str) int { return a.s.len() + b.len() }
fn main() int { return f(T { s: "first" }, "second".to(Str).to(str)) }
''',
'reverse_temp_order': '''type T { s Str }
fn f(a str, b T) int { return a.len() + b.s.len() }
fn main() int { return f("first".to(Str).to(str), T { s: "second" }) }
''',
'field_copy_return': '''type Inner { s Str; n int }
type Outer { i Inner; n int }
fn extract(a Outer) Inner { return a.i }
fn main() int {
 let a Outer = Outer { i: Inner { s: "abc", n: 1 }, n: 2 }
 let b Inner = extract(a)
 return b.s.len() + a.i.s.len()
}
''',
'mixed_local_order': '''type T { s Str }
fn main() int {
 let s Str = "first"
 let a T = T { s: "second" }
 return s.len() + a.s.len()
}
''',
'nested_replace_borrow': '''type I { s Str }
type O { i I }
fn main() int {
 var a O = O { i: I { s: "old" } }
 let v = a.i.s
 a.i = I { s: "new" }
 return v.len()
}
''',
'field_ctor_copy': '''type I { s Str }
type O { i I }
fn main() int {
 let a O = O { i: I { s: "abc" } }
 let b O = O { i: a.i }
 return b.i.s.len() + a.i.s.len()
}
''',
}

def run(args):
    p = subprocess.run([str(a) for a in args], cwd=ROOT, capture_output=True, text=True, encoding='utf-8', errors='replace', timeout=60)
    return dict(code=p.returncode, stdout=p.stdout, stderr=p.stderr)

results = {'tool_sha256': hashlib.sha256(TOOL.read_bytes()).hexdigest(), 'cases':{}}
for name, src in CASES.items():
    source = HERE / (name + '.at')
    source.write_text(src, encoding='utf-8')
    exe = OUT / (name + '.exe')
    args = [TOOL, 'build-source', source, '--source-profile', 'auto.source.text-record.draft', '--entry', 'main', '--capability', 'ac.runtime.text.v1', '--capability', 'hir.test.trace.v1', '--support-lib', LIB, '--output', exe]
    b = run(args)
    r = run([exe]) if b['code'] == 0 else None
    results['cases'][name] = {'build': b, 'run': r}
    print(name, json.dumps(results['cases'][name], ensure_ascii=False))
(HERE / 'results.json').write_text(json.dumps(results, indent=2, ensure_ascii=False), encoding='utf-8')
