//! PLAN-1083 T-05 canonical verification: every case of
//! `language/types/variants.case.ad` and `language/patterns/nested-match.case.ad`
//! runs through the ORDINARY AV chain in-process (compile → lower → MIR verify
//! → ABC → AV execute) with its authored oracle. This is the authoring-level
//! twin of the fixed-group corpus run (`case-av-language-parity`), which owns
//! the formal gate through the prepared worker; the sources here are the
//! canonical files' Auto blocks verbatim.

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::AbcBytes;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

fn run(src: &str) -> (ExecOutcome, ConsoleCapture) {
    let compiled = ace_core::source::compile_language_source(src, "p1083t5.at", &[])
        .unwrap_or_else(|d| panic!("compile failed: {d:?} — source:
{src}"));
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0x1083)
        .expect("lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire: AbcBytes = ace_av_core::abc::encode_abc(&mir).expect("AVAD encode");
    let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits");
    let mut capture = ConsoleCapture::default();
    let (out, _t) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    (out, capture)
}

fn assert_exit(src: &str, want: i32) {
    match run(src).0 {
        ExecOutcome::Success {
            value: Value::I32(v), ..
        } => assert_eq!(v, want, "source:\n{src}"),
        other => panic!("expected exit {want}, got {other:?} — source:\n{src}"),
    }
}

fn assert_stdout(src: &str, want: &str) {
    let (out, c) = run(src);
    assert!(
        matches!(out, ExecOutcome::Success { .. }),
        "expected success — source:\n{src}"
    );
    assert_eq!(
        String::from_utf8_lossy(&c.stdout),
        want,
        "source:\n{src}"
    );
}

fn assert_diag(src: &str, want_code: &str) {
    match ace_core::source::compile_language_source(src, "p1083t5.at", &[]) {
        Ok(_) => panic!("expected diagnostic {want_code}, got success — source:\n{src}"),
        Err(ds) => {
            let d = ds.first().unwrap();
            assert_eq!(d.code, want_code, "source:\n{src}");
        }
    }
}

#[test]
fn t05_types_variants_cases() {
    // named-enum-payload (§6 #1)
    assert_exit(
        "enum Answer { Yes { code int, text Str }; No }\nfn main() int {\n let a = Answer.Yes(code: 42, text: \"ok\")\n is a { Answer.Yes(code, text) -> { if text == \"ok\" { return code }; return 0 }; Answer.No -> { return 0 } }\n}\n",
        42,
    );
    // multi-field-positional
    assert_stdout(
        "enum Shape {\n    Rect { w int, h int }\n}\nfn main() {\n    let s = Shape.Rect(10, 20)\n    is s {\n        Shape.Rect(w, h) -> {\n            print(w)\n            print(h)\n        }\n    }\n}\n",
        "10\n20\n",
    );
    // single-named-payload
    assert_stdout(
        "enum Shape {\n    Circle { r int }\n}\nfn main() {\n    let c = Shape.Circle(5)\n    is c {\n        Shape.Circle(r) -> print(r)\n    }\n}\n",
        "5\n",
    );
    // bare-scalar-arm
    assert_stdout(
        "enum Shape {\n    Point\n}\nfn main() {\n    let p = Shape.Point\n    is p {\n        Shape.Point -> print(\"point\")\n    }\n}\n",
        "point\n",
    );
    // named-construct-ordered
    assert_stdout(
        "enum ApiError {\n    Http { code int, msg Str }\n}\nfn main() {\n    let e1 = ApiError.Http(code: 404, msg: \"not found\")\n    is e1 {\n        ApiError.Http(c, m) -> print(c)\n    }\n}\n",
        "404\n",
    );
    // named-construct-reordered
    assert_stdout(
        "enum ApiError {\n    Http { code int, msg Str }\n}\nfn main() {\n    let e2 = ApiError.Http(msg: \"timeout\", code: 503)\n    is e2 {\n        ApiError.Http(c, m) -> print(c)\n    }\n}\n",
        "503\n",
    );
    // positional-tuple-payload
    assert_stdout(
        "enum ApiError {\n    Timeout { millis int }\n}\nfn main() {\n    let e3 = ApiError.Timeout(5000)\n    is e3 {\n        ApiError.Timeout(ms) -> print(ms)\n    }\n}\n",
        "5000\n",
    );
    // no-binding-arm
    assert_stdout(
        "enum Color {\n    RGB { r int, g int, b int }\n}\nfn main() {\n    let c1 = Color.RGB(100, 150, 200)\n    is c1 {\n        Color.RGB -> print(\"rgb\")\n    }\n}\n",
        "rgb\n",
    );
    // multi-field-arith-destructuring
    assert_stdout(
        "enum Color {\n    RGB { r int, g int, b int }\n}\nfn main() {\n    let c3 = Color.RGB(255, 128, 0)\n    is c3 {\n        Color.RGB(r, g, b) -> print(r + g + b)\n    }\n}\n",
        "383\n",
    );
    // float-payload
    assert_stdout(
        "enum Shape {\n    Circle(float)\n}\nfn main() {\n    let s = Shape.Circle(3.14)\n    is s {\n        Shape.Circle(r) -> print(r)\n    }\n}\n",
        "3.14\n",
    );
    // two-elem-tuple-payload
    assert_stdout(
        "enum Shape {\n    Rect(int, int)\n}\nfn main() {\n    let r = Shape.Rect(10, 20)\n    is r {\n        Shape.Rect(w, h) -> {\n            print(w)\n            print(h)\n        }\n    }\n}\n",
        "10\n20\n",
    );
    // scalar-enum-value
    assert_exit(
        "enum Color { Red = 1, Green = 2, Blue = 3 }\nfn main() int { let c = Color.Red\n is c { Color.Red -> return 1; Color.Green -> return 2; else -> return 0 }\n}\n",
        1,
    );
    // int-literal-arms (types copy)
    assert_exit(
        "fn check(c int) int {\n    return is c {\n        1 -> 10\n        3 -> 30\n        else -> 0\n    }\n}\nfn main() int { return check(1) + check(3) + check(9) }\n",
        40,
    );
    // enum-static-style-method
    assert_stdout(
        "enum Color {\n    Red\n    Green\n    fn name() Str {\n        return \"called-ok\"\n    }\n}\nfn main() {\n    print(Color.Red.name())\n}\n",
        "called-ok\n",
    );
    // enum-instance-method
    assert_stdout(
        "enum Light {\n    Red\n    Green\n    fn is_red() bool {\n        is self {\n            Light.Red -> return true\n        }\n        return false\n    }\n}\nfn main() {\n    if Light.Red.is_red() { print(\"yes\") } else { print(\"no\") }\n    if Light.Green.is_red() { print(\"yes\") } else { print(\"no\") }\n}\n",
        "yes\nno\n",
    );
    // payload-variant-method
    assert_stdout(
        "enum Reg {\n    Yes(int)\n    No\n    fn is_yes() bool {\n        is self {\n            Reg.Yes -> return true\n        }\n        return false\n    }\n}\nfn main() {\n    if Reg.Yes(42).is_yes() { print(\"yes\") } else { print(\"no\") }\n    if Reg.No.is_yes() { print(\"yes\") } else { print(\"no\") }\n}\n",
        "yes\nno\n",
    );
    // wildcard-arm
    assert_stdout(
        "enum Light {
    Red
    Green
    Blue
}
fn main() {
    is Light.Red {
        Light.Red -> print(\"red\")
        _ -> print(\"other\")
    }
    is Light.Green {
        Light.Red -> print(\"red\")
        _ -> print(\"other\")
    }
    is Light.Blue {
        Light.Red -> print(\"red\")
        _ -> print(\"other\")
    }
}
",
        "red
other
other
",
    );
    // option-family
    assert_exit(
        "fn main() int {\n    let a = Some(42)\n    let b = None\n    is a { Some(v) -> { return v }; None -> { return -1 } }\n    let _ = b\n    return 0\n}\n",
        42,
    );
    // option-none-match
    assert_exit(
        "fn main() int { let opt = None\n is opt { Some(v) -> return v; None -> return -1 }\n}\n",
        -1,
    );
    // result-family
    assert_exit(
        "fn main() int {\n    let res = Err(\"fail\")\n    is res {\n        Ok(v) -> return v\n        Err(e) -> return -2\n    }\n}\n",
        -2,
    );
    // unwrap-or-default
    assert_exit(
        "fn main() int { let a = Some(42)\n let b = a.?(0)\n let c = None\n let d = c.?(99)\n return b + d\n}\n",
        141,
    );
    // option-or
    assert_exit(
        "fn main() int { let a = Some(42)\n let b = a.or(0)\n let c = None\n let d = c.or(99)\n return b + d\n}\n",
        141,
    );
    // result-fn-str-error
    assert_exit(
        "fn try_divide(a int, b int) !int {\n    if b == 0 {\n        return Err(\"division by zero\")\n    }\n    return Ok(a / b)\n}\nfn main() int {\n    let r1 = try_divide(10, 2)\n    is r1 {\n        Ok(n) -> return n\n        Err(e) -> return 0\n    }\n}\n",
        5,
    );
    // err-enum-payload
    assert_stdout(
        "enum ParseError {\n    UnexpectedEnd\n}\nfn parse(s Str) !int {\n    if s == \"\" {\n        return Err(ParseError.UnexpectedEnd)\n    }\n    return Ok(42)\n}\nfn main() {\n    let r = parse(\"\")\n    is r {\n        Ok(n) -> print(n)\n        Err(e) -> {\n            is e {\n                ParseError.UnexpectedEnd -> print(\"unexpected end\")\n                else -> print(\"other error\")\n            }\n        }\n    }\n}\n",
        "unexpected end\n",
    );
    // multi-error-family
    assert_stdout(
        "enum ApiError {\n    Http { code int, msg Str }\n    Timeout\n}\nfn fetch(url Str) !int {\n    if url == \"\" {\n        return Err(ApiError.Http(code: 404, msg: \"not found\"))\n    }\n    if url == \"slow\" {\n        return Err(ApiError.Timeout)\n    }\n    return Ok(200)\n}\nfn main() {\n    let r1 = fetch(\"\")\n    is r1 {\n        Ok(data) -> print(data)\n        Err(e) -> {\n            is e {\n                ApiError.Http(code, msg) -> print(code)\n                ApiError.Timeout -> print(\"timeout\")\n            }\n        }\n    }\n    let r2 = fetch(\"slow\")\n    is r2 {\n        Ok(data) -> print(data)\n        Err(e) -> {\n            is e {\n                ApiError.Http(code, msg) -> print(code)\n                ApiError.Timeout -> print(\"timeout\")\n            }\n        }\n    }\n    let r3 = fetch(\"ok\")\n    is r3 {\n        Ok(data) -> print(data)\n        Err(e) -> print(\"error\")\n    }\n}\n",
        "404\ntimeout\n200\n",
    );
    // record-method-reads
    assert_stdout(
        "type Step {\n    v int\n    fn add(n int) Step { return Step.new(.v + n) }\n    fn get() int { return .v }\n    static fn new(v int) Step { return Step(v) }\n}\nfn main() {\n    print(f\"chain=${Step.new(1).add(2).add(3).get()}\")\n}\n",
        "chain=6\n",
    );
    // positional-ctor-family
    assert_stdout(
        "type Point {\n    x int\n    y int\n    fn sum() int { return .x + .y }\n    static fn origin() Point { return Point(0, 0) }\n}\nfn build(v int) Point {\n    return Point(v, v * 2)\n}\nfn main() {\n    let p = build(3)\n    print(f\"build=${p.sum()}\")\n    let o = Point.origin()\n    print(f\"origin=${o.sum()}\")\n}\n",
        "build=9\norigin=0\n",
    );
    // list-elem-method
    assert_stdout(
        "type Item {\n    v int\n    fn get() int { return .v }\n    static fn new(v int) Item { return Item(v) }\n}\nfn total(items List<Item>) int {\n    var t = 0\n    for it in items {\n        t = t + it.get()\n    }\n    return t\n}\nfn main() {\n    let xs = [Item.new(1), Item.new(2), Item.new(3)]\n    print(f\"total=${total(xs)}\")\n    print(f\"idx1=${xs[1].get()}\")\n}\n",
        "total=6\nidx1=2\n",
    );
    // method-value-family
    assert_stdout(
        "type Cube {\n    v int\n    fn get() int { return .v }\n    fn twice() int { return self.get() * 2 }\n    static fn new(v int) Cube { return Cube(v) }\n}\nfn apply(b Cube) int { return b.get() }\nfn make(v int) Cube { return Cube.new(v) }\nfn main() {\n    print(apply(Cube.new(7)))\n    print(make(8).get())\n    let b = Cube.new(6)\n    print(b.twice())\n}\n",
        "7\n8\n12\n",
    );
    // str-field-record
    assert_stdout(
        "type Note { id int; title Str }\nfn make_note(i int, t Str) Note {\n    Note { id: i, title: t }\n}\nfn main() {\n    let n = Note { id: 1, title: \"hello\" }\n    if n.title == \"hello\" {\n        print(\"title ok\")\n    }\n    if n.id == 1 {\n        print(\"id ok\")\n    }\n    let m = make_note(42, \"world\")\n    if m.title == \"world\" {\n        print(\"from fn ok\")\n    }\n}\n",
        "title ok\nid ok\nfrom fn ok\n",
    );
    // record-array-fields
    assert_stdout(
        "type Note { id int; title Str }\nfn main() {\n    let notes = [\n        Note { id: 0, title: \"Welcome\" },\n        Note { id: 1, title: \"Shopping\" },\n    ]\n    if notes[0].id == 0 {\n        print(\"array field ok\")\n    }\n    if notes[1].id == 1 {\n        print(\"second id ok\")\n    }\n}\n",
        "array field ok\nsecond id ok\n",
    );
    // nested-type-instance
    assert_exit(
        "type Inner { val int }\ntype Outer { inner Inner }\nfn main() int {\n    let o = Outer { inner: Inner { val: 10 } }\n    return o.inner.val\n}\n",
        10,
    );
}

#[test]
fn t05_patterns_nested_match_cases() {
    // nested-result-option (§6 #2)
    assert_exit(
        "fn obtain(ok bool) !?int {\n    if ok {\n        return Ok(Some(42))\n    }\n    return Err(\"bad\")\n}\nfn main() int {\n is obtain(true) {\n  Ok(v) -> {\n   is v {\n    Some(n) -> { return n }\n    None -> { return 0 }\n   }\n  }\n  Err(e) -> { return 0 }\n }\n}\n",
        42,
    );
    // tuple-across-call (§6 #3)
    assert_exit(
        "fn pair() (int, int) { return (19, 23) }\nfn main() int { let (a, b) = pair(); return a + b }\n",
        42,
    );
    // duplicate-pattern (§6 #4) — named rejection
    assert_diag(
        "fn main() int { return is 1 { 1 -> 40, 1 -> 42, else -> 0 } }\n",
        "source.pattern-duplicate",
    );
    // nested-is-shadowing
    assert_exit(
        "enum Outer {\n    Wrap(int)\n}\nfn main() int {\n    let v = 7\n    let o = Outer.Wrap(42)\n    is o {\n        Outer.Wrap(v) -> {\n            return v\n        }\n    }\n}\n",
        42,
    );
    // guard-arm
    assert_exit(
        "enum Reg {\n    Yes(int)\n    No\n}\nfn main() int {\n    let r = Reg.Yes(42)\n    is r {\n        Reg.Yes(v) if v > 100 -> { return 1 }\n        Reg.Yes(v) -> { return v }\n        Reg.No -> { return 0 }\n    }\n}\n",
        42,
    );
    // guard-not-evaluated
    assert_stdout(
        "enum Reg {\n    Yes(int)\n    No\n}\nfn side() bool { print(\"g\")\n return true }\nfn main() {\n    let r = Reg.No\n    is r {\n        Reg.Yes(v) if side() -> { print(\"y\") }\n        Reg.No -> { print(\"n\") }\n    }\n}\n",
        "n\n",
    );
    // or-arm
    assert_exit(
        "enum Light {\n    Red\n    Green\n}\nfn main() int {\n    let l = Light.Green\n    is l {\n        Light.Red | Light.Green -> { return 1 }\n    }\n}\n",
        1,
    );
    // nonexhaustive-expr-rejects — named rejection
    assert_diag(
        "fn main() int { return is 1 { 1 -> 40 } }\n",
        "source.pattern-nonexhaustive",
    );
    // statement-fallthrough
    assert_exit(
        "fn main() int {\n    var hit = 0\n    let x = 5\n    is x {\n        1 -> { hit = 1 }\n    }\n    return hit\n}\n",
        0,
    );
    // exhaustiveness-else
    assert_exit(
        "fn main() int { let x = 5\n is x {\n  1 -> { return 10 }\n  else -> { return 0 }\n }\n}\n",
        0,
    );
    // fn-tail-is-value
    assert_exit(
        "fn check(c int) int {\n    return is c {\n        1 -> 10\n        2 -> 20\n        else -> 0\n    }\n}\nfn main() int { return check(1) + check(2) + check(9) }\n",
        30,
    );
    // script-tail-is-value
    assert_exit(
        "let opt = Some(42)\nis opt {\n  Some(v) -> v\n  None -> 0\n}\n",
        42,
    );
}
