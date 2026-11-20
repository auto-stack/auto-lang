//! PLAN-1083 T-01 feasibility probes: the records/variants/patterns language
//! face through the ORDINARY Auto source profile (`compile_language_source`)
//! — the same front door the `ac-av/language-source-run-v1` recipe runs.
//!
//! Phase 1 (T-01): one OBSERVATION test prints, for each legacy/canonical
//! face variant, the current chain outcome (success value+stdout / first
//! diagnostic stage+code) under `--nocapture`. Sources are the 52 legacy
//! family rows (09_functions.016..019, 10_types.*, 16_option_result.*,
//! 28_enum_methods.*, 99_idiom2.d02/m01..m09) or minimal one-face deltas and
//! the §6 draft cases. The pinned expectations record the T-01 OBSERVED
//! baseline; T-02..04 flip them to the contract (oracle values / named
//! rejections) as the faces land.
//!
//! Observation index:
//!   Records/methods:  O1 m01 field rw  O2 m02 method-calls-method
//!     O3 m03 static-new chain       O4 m04 positional ctor
//!     O5 m05 list-elem method       O6 m06 method value
//!     O7 m08 receiver forms         O8 m09 method name `set`
//!     O9 008 nested type instance   O10 013 fn-return record + display
//!     O11 034 str-field record      O12 036 record in array literal
//!     O13 009 fn Point.sum(self)    O14 001 `type Duck has Wing`
//!     O15 010 ext Point             O16 d02 `struct` misuse
//!   Enums:  O17 fn-local enum       O18 named multi-field payload
//!     O19 single named payload      O20 bare scalar arm
//!     O21 named construct ordered   O22 named construct reordered
//!     O23 positional tuple payload  O24 no-binding arm
//!     O25 multi-field arith destruct O26 float tuple payload
//!     O27 two-elem tuple payload    O28 scalar `= N` discriminant
//!     O29 int literal arms          O30 qualified scalar arm on int
//!     O31 enum static method        O32 enum instance method `is self`
//!     O33 payload variant method    O34 `_` wildcard arm
//!     O35 generic enum Bag<T,E>     O36 `use auto.result` + is_ok
//!   Option/Result:  O37 `?int` ann  O38 `!int` ann
//!     O39 bare None/Some/Ok/Err     O40 is Some binding (script tail)
//!     O41 is None match             O42 is Ok/Err binding
//!     O43 `.?(0)` unwrap-or         O44 `.or(0)`
//!     O45 `!int` fn + return Err    O46 Err enum payload nested is
//!     O47 multi-error nested is
//!   §6 drafts/contract:  O48 `!?int` nested draft
//!     O49 tuple across call         O50 duplicate literal arm
//!     O51 `;` arm separators        O52 guard arm syntax
//!     O53 or-arm syntax             O54 nested is shadowing

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::AbcBytes;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

/// Compile and admit one Auto-profile program (the recipe chain up to the
/// executable module).
fn compile_and_load(src: &str) -> ace_av_core::abc_verify::LoadedModule {
    let compiled = ace_core::source::compile_language_source(src, "p1083.at", &[])
        .unwrap_or_else(|d| panic!("Auto-profile compile failed: {d:?}"));
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0x1083)
        .expect("auto-language lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire: AbcBytes = ace_av_core::abc::encode_abc(&mir).expect("AVAD encode");
    abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits")
}

fn run(src: &str) -> (ExecOutcome, ConsoleCapture) {
    let mut capture = ConsoleCapture::default();
    let loaded = compile_and_load(src);
    let (out, _trace) = execute_sched(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        &mut capture,
    );
    (out, capture)
}

/// Human-readable current outcome of one source: the executed value and
/// captured stdout, or the first compile diagnostic (stage+code).
fn observe(src: &str) -> String {
    match ace_core::source::compile_language_source(src, "p1083.at", &[]) {
        Ok(_) => match run(src) {
            (ExecOutcome::Success { value: Value::I32(v), .. }, c) => format!(
                "ok:{v}|out={:?}",
                String::from_utf8_lossy(&c.stdout)
            ),
            (other, c) => format!(
                "exec:{other:?}|out={:?}",
                String::from_utf8_lossy(&c.stdout)
            ),
        },
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            format!("diag:{:?}:{}", d.stage, d.code).to_lowercase()
        }
    }
}

/// The T-01 observation matrix — prints each face's current outcome. No
/// pinned expectations yet: this run FREEZES the observed baseline that
/// T-02..04 then flip face by face.
#[test]
fn t01_observe_face_matrix() {
    let faces: Vec<(&str, String)> = vec![
        // ---- records / methods family (idiom2 m01..m09, 10_types) ----
        (
            "O1 m01 field rw + static new",
            concat!(
                "type Counter {\n",
                "    value int\n",
                "    fn get() int { return .value }\n",
                "    fn inc() int {\n",
                "        .value = .value + 1\n",
                "        return .value\n",
                "    }\n",
                "    fn set_to(n int) {\n",
                "        .value = n\n",
                "    }\n",
                "    static fn new(v int) Counter { return Counter(v) }\n",
                "}\n",
                "fn main() {\n",
                "    var c = Counter.new(10)\n",
                "    print(f\"get=${c.get()}\")\n",
                "    print(f\"inc=${c.inc()}\")\n",
                "    c.set_to(99)\n",
                "    print(f\"after set_to=${c.get()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O2 m02 method calls method",
            concat!(
                "type Calc {\n",
                "    base int\n",
                "    fn helper(n int) int { return .base + n }\n",
                "    fn call_helper() int { return .helper(3) }\n",
                "    fn chain_local() int { return .helper(.helper(1)) }\n",
                "    static fn new(v int) Calc { return Calc(v) }\n",
                "}\n",
                "fn main() {\n",
                "    let c = Calc.new(10)\n",
                "    print(f\"call_helper=${c.call_helper()}\")\n",
                "    print(f\"nested=${c.chain_local()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O3 m03 static-new chain",
            concat!(
                "type Step {\n",
                "    v int\n",
                "    fn add(n int) Step { return Step.new(.v + n) }\n",
                "    fn get() int { return .v }\n",
                "    static fn new(v int) Step { return Step(v) }\n",
                "}\n",
                "fn main() {\n",
                "    print(f\"chain=${Step.new(1).add(2).add(3).get()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O4 m04 positional ctor in fn",
            concat!(
                "type Point {\n",
                "    x int\n",
                "    y int\n",
                "    fn sum() int { return .x + .y }\n",
                "    static fn origin() Point { return Point(0, 0) }\n",
                "}\n",
                "fn build(v int) Point {\n",
                "    return Point(v, v * 2)\n",
                "}\n",
                "fn main() {\n",
                "    let p = build(3)\n",
                "    print(f\"build=${p.sum()}\")\n",
                "    let o = Point.origin()\n",
                "    print(f\"origin=${o.sum()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O5 m05 list elem method",
            concat!(
                "type Item {\n",
                "    v int\n",
                "    fn get() int { return .v }\n",
                "    static fn new(v int) Item { return Item(v) }\n",
                "}\n",
                "fn total(items List<Item>) int {\n",
                "    var t = 0\n",
                "    for it in items {\n",
                "        t = t + it.get()\n",
                "    }\n",
                "    return t\n",
                "}\n",
                "fn main() {\n",
                "    let xs = [Item.new(1), Item.new(2), Item.new(3)]\n",
                "    print(f\"total=${total(xs)}\")\n",
                "    print(f\"idx1=${xs[1].get()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O6 m06 method value",
            concat!(
                "type Cube {\n",
                "    v int\n",
                "    fn get() int { return .v }\n",
                "    fn twice() int { return .get() * 2 }\n",
                "    static fn new(v int) Cube { return Cube(v) }\n",
                "}\n",
                "fn apply(b Cube) int { return b.get() }\n",
                "fn make(v int) Cube { return Cube.new(v) }\n",
                "fn main() {\n",
                "    print(f\"apply=${apply(Cube.new(7))}\")\n",
                "    print(f\"make_get=${make(8).get()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O7 m08 receiver forms",
            concat!(
                "type Mixed {\n",
                "    a int\n",
                "    b int\n",
                "    fn short_sum() int { return .a + .b }\n",
                "    fn self_sum() int { return self.a + self.b }\n",
                "    fn mixed_set(n int) { self.a = n }\n",
                "    static fn new(a int, b int) Mixed { return Mixed(a, b) }\n",
                "}\n",
                "fn main() {\n",
                "    let m = Mixed.new(1, 2)\n",
                "    print(f\"short=${m.short_sum()}\")\n",
                "    print(f\"self=${m.self_sum()}\")\n",
                "    var n = Mixed.new(10, 20)\n",
                "    n.mixed_set(100)\n",
                "    print(f\"after=${n.short_sum()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O8 m09 method named set",
            concat!(
                "type Holder {\n",
                "    v int\n",
                "    fn set(n int) { self.v = n }\n",
                "    fn get() int { return .v }\n",
                "    static fn new(v int) Holder { return Holder(v) }\n",
                "}\n",
                "fn main() {\n",
                "    var h = Holder.new(1)\n",
                "    h.set(42)\n",
                "    print(f\"set_get=${h.get()}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O9 008 nested type instance",
            concat!(
                "type Inner { val int }\n",
                "type Outer { inner Inner }\n",
                "fn main() int {\n",
                "    let o = Outer { inner: Inner { val: 10 } }\n",
                "    return o.inner.val\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O10 013 fn return record + display",
            concat!(
                "type Point { x int }\n",
                "fn make_point(val int) Point {\n",
                "    Point { x: val }\n",
                "}\n",
                "fn main() {\n",
                "    let p = make_point(42)\n",
                "    print(f\"p.x=${p.x}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O11 034 str-field record",
            concat!(
                "type Note { id int; title Str }\n",
                "fn main() {\n",
                "    let n = Note { id: 1, title: \"hello\" }\n",
                "    if n.title == \"hello\" {\n",
                "        print(\"title ok\")\n",
                "    }\n",
                "    if n.id == 1 {\n",
                "        print(\"id ok\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O12 036 record in array literal",
            concat!(
                "type Note { id int; title Str }\n",
                "fn main() {\n",
                "    let notes = [\n",
                "        Note { id: 0, title: \"Welcome\" },\n",
                "        Note { id: 1, title: \"Shopping\" },\n",
                "    ]\n",
                "    let n = notes[0]\n",
                "    if n.title == \"Welcome\" {\n",
                "        print(\"array field ok\")\n",
                "    }\n",
                "    let second = notes[1]\n",
                "    if second.id == 1 {\n",
                "        print(\"second id ok\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O13 009 fn Point.sum(self)",
            concat!(
                "type Point { x int; y int }\n",
                "fn Point.sum(self) int { self.x + self.y }\n",
                "fn main() int {\n",
                "    var p = Point { x: 100, y: 200 }\n",
                "    return p.sum()\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O14 001 type Duck has Wing",
            concat!(
                "type Wing { speed int }\n",
                "type Duck { name Str }\n",
                "fn Wing.fly(self) { print(\"flap!flap!\") }\n",
                "type Duck has Wing\n",
                "fn main() int {\n",
                "    var duck = Duck { name: \"donald\", speed: 10 }\n",
                "    duck.fly()\n",
                "    return 1\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O15 010 ext Point",
            concat!(
                "type Point { x int }\n",
                "ext Point {\n",
                "    fn get_x(self) int {\n",
                "        self.x\n",
                "    }\n",
                "}\n",
                "fn main() int {\n",
                "    let p = Point { x: 42 }\n",
                "    return p.get_x()\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O16 d02 struct misuse",
            concat!(
                "fn make() int {\n",
                "    struct Val {\n",
                "        x int\n",
                "    }\n",
                "    return 2\n",
                "}\n",
                "fn main() int {\n",
                "    return make()\n",
                "}\n",
            )
            .into(),
        ),
        // ---- enums family ----
        (
            "O17 fn-local enum decl",
            concat!(
                "fn main() {\n",
                "    enum Shape {\n",
                "        Point\n",
                "    }\n",
                "    let p = Shape.Point\n",
                "    is p {\n",
                "        Shape.Point -> print(\"point\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O18 named multi-field payload",
            concat!(
                "enum Shape {\n",
                "    Rect { w int, h int }\n",
                "}\n",
                "fn main() {\n",
                "    let s = Shape.Rect(10, 20)\n",
                "    is s {\n",
                "        Shape.Rect(w, h) -> {\n",
                "            print(w)\n",
                "            print(h)\n",
                "        }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O19 single named payload",
            concat!(
                "enum Shape {\n",
                "    Circle { r int }\n",
                "}\n",
                "fn main() {\n",
                "    let c = Shape.Circle(5)\n",
                "    is c {\n",
                "        Shape.Circle(r) -> print(r)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O20 bare scalar arm",
            concat!(
                "enum Shape {\n",
                "    Point\n",
                "}\n",
                "fn main() {\n",
                "    let p = Shape.Point\n",
                "    is p {\n",
                "        Shape.Point -> print(\"point\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O21 named construct ordered",
            concat!(
                "enum ApiError {\n",
                "    Http { code int, msg Str }\n",
                "}\n",
                "fn main() {\n",
                "    let e1 = ApiError.Http(code: 404, msg: \"not found\")\n",
                "    is e1 {\n",
                "        ApiError.Http(c, m) -> print(c)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O22 named construct reordered",
            concat!(
                "enum ApiError {\n",
                "    Http { code int, msg Str }\n",
                "}\n",
                "fn main() {\n",
                "    let e2 = ApiError.Http(msg: \"timeout\", code: 503)\n",
                "    is e2 {\n",
                "        ApiError.Http(c, m) -> print(c)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O23 positional tuple payload",
            concat!(
                "enum ApiError {\n",
                "    Timeout { millis int }\n",
                "}\n",
                "fn main() {\n",
                "    let e3 = ApiError.Timeout(5000)\n",
                "    is e3 {\n",
                "        ApiError.Timeout(ms) -> print(ms)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O24 no-binding arm",
            concat!(
                "enum Color {\n",
                "    RGB { r int, g int, b int }\n",
                "}\n",
                "fn main() {\n",
                "    let c1 = Color.RGB(100, 150, 200)\n",
                "    is c1 {\n",
                "        Color.RGB -> print(\"rgb\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O25 multi-field arith destruct",
            concat!(
                "enum Color {\n",
                "    RGB { r int, g int, b int }\n",
                "}\n",
                "fn main() {\n",
                "    let c3 = Color.RGB(255, 128, 0)\n",
                "    is c3 {\n",
                "        Color.RGB(r, g, b) -> print(r + g + b)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O26 float tuple payload",
            concat!(
                "enum Shape {\n",
                "    Circle(float)\n",
                "}\n",
                "fn main() {\n",
                "    let s = Shape.Circle(3.14)\n",
                "    is s {\n",
                "        Shape.Circle(r) -> print(r)\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O27 two-elem tuple payload",
            concat!(
                "enum Shape {\n",
                "    Rect(int, int)\n",
                "}\n",
                "fn main() {\n",
                "    let r = Shape.Rect(10, 20)\n",
                "    is r {\n",
                "        Shape.Rect(w, h) -> {\n",
                "            print(w)\n",
                "            print(h)\n",
                "        }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O28 scalar = N discriminant",
            "enum Color { Red = 1, Green = 2, Blue = 3 }\nfn main() int { let c = Color.Red\n is c { Color.Red -> return 1; Color.Green -> return 2; else -> return 0 } }\n".into(),
        ),
        (
            "O29 int literal arms",
            concat!(
                "fn check(c int) int {\n",
                "    is c {\n",
                "        1 -> 10\n",
                "        2 -> 20\n",
                "        else -> 0\n",
                "    }\n",
                "}\n",
                "fn main() int { return check(1) + check(2) + check(9) }\n",
            )
            .into(),
        ),
        (
            "O30 qualified scalar arm on int",
            concat!(
                "enum Color { Red = 1, Green = 2 }\n",
                "fn check(c int) int {\n",
                "    is c {\n",
                "        Color.Red -> 100\n",
                "        else -> 0\n",
                "    }\n",
                "}\n",
                "fn main() int { return check(1) + check(9) }\n",
            )
            .into(),
        ),
        (
            "O31 enum static method",
            concat!(
                "enum Color {\n",
                "    Red\n",
                "    Green\n",
                "    fn name() Str {\n",
                "        return \"called-ok\"\n",
                "    }\n",
                "}\n",
                "fn main() {\n",
                "    print(Color.Red.name())\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O32 enum instance method is self",
            concat!(
                "enum Light {\n",
                "    Red\n",
                "    Green\n",
                "    fn is_red() bool {\n",
                "        is self {\n",
                "            Light.Red -> return true\n",
                "        }\n",
                "        return false\n",
                "    }\n",
                "}\n",
                "fn main() {\n",
                "    if Light.Red.is_red() { print(\"yes\") } else { print(\"no\") }\n",
                "    if Light.Green.is_red() { print(\"yes\") } else { print(\"no\") }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O33 payload variant method",
            concat!(
                "enum Reg {\n",
                "    Yes(int)\n",
                "    No\n",
                "    fn is_yes() bool {\n",
                "        is self {\n",
                "            Reg.Yes -> return true\n",
                "        }\n",
                "        return false\n",
                "    }\n",
                "}\n",
                "fn main() {\n",
                "    if Reg.Yes(42).is_yes() { print(\"yes\") } else { print(\"no\") }\n",
                "    if Reg.No.is_yes() { print(\"yes\") } else { print(\"no\") }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O34 _ wildcard arm",
            concat!(
                "enum Light {\n",
                "    Red\n",
                "    Green\n",
                "}\n",
                "fn main() {\n",
                "    let l = Light.Green\n",
                "    is l {\n",
                "        Light.Red -> print(\"red\")\n",
                "        _ -> print(\"other\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O35 generic enum Bag<T,E>",
            concat!(
                "enum Bag<T, E> {\n",
                "    Hit T\n",
                "    Miss E\n",
                "}\n",
                "fn main() {\n",
                "    let r = Bag.Hit(42)\n",
                "    print(\"made\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O36 use auto.result is_ok",
            concat!(
                "use auto.result\n",
                "fn main() {\n",
                "    let r = Result.Ok(42)\n",
                "    if r.is_ok() { print(\"ok\") }\n",
                "}\n",
            )
            .into(),
        ),
        // ---- Option/Result family ----
        (
            "O37 ?int annotation",
            "fn main() int { let x ?int = None\n is x { Some(v) -> return v; None -> return 1 } }\n".into(),
        ),
        (
            "O38 !int annotation",
            "fn main() int { let x !int = Ok(42)\n is x { Ok(v) -> return v; Err(e) -> return 0 } }\n".into(),
        ),
        (
            "O39 bare None/Some/Ok/Err",
            concat!(
                "fn main() int {\n",
                "    let a = Some(42)\n",
                "    let b = None\n",
                "    let c = Ok(100)\n",
                "    let d = Err(\"msg\")\n",
                "    is a { Some(v) -> return v; None -> return 0 }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O40 is Some binding script tail",
            "let opt = Some(42)\nis opt {\n  Some(v) -> v\n  None -> 0\n}\n".into(),
        ),
        (
            "O41 is None match",
            "fn main() int { let opt = None\n is opt { Some(v) -> return v; None -> return -1 } }\n".into(),
        ),
        (
            "O42 is Ok/Err binding",
            concat!(
                "fn main() int {\n",
                "    let res = Err(\"fail\")\n",
                "    is res {\n",
                "        Ok(v) -> return v\n",
                "        Err(e) -> return -2\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O43 .?(0) unwrap-or",
            "fn main() int { let a = Some(42)\n let b = a.?(0)\n let c = None\n let d = c.?(99)\n return b + d }\n".into(),
        ),
        (
            "O44 .or(0)",
            "fn main() int { let a = Some(42)\n let b = a.or(0)\n let c = None\n let d = c.or(99)\n return b + d }\n".into(),
        ),
        (
            "O45 !int fn + return Err",
            concat!(
                "fn try_divide(a int, b int) !int {\n",
                "    if b == 0 {\n",
                "        return Err(\"division by zero\")\n",
                "    }\n",
                "    Ok(a / b)\n",
                "}\n",
                "fn main() int {\n",
                "    let r1 = try_divide(10, 2)\n",
                "    is r1 {\n",
                "        Ok(n) -> return n\n",
                "        Err(e) -> return 0\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O46 Err enum payload nested is",
            concat!(
                "enum ParseError {\n",
                "    UnexpectedEnd\n",
                "}\n",
                "fn parse(s Str) !int {\n",
                "    if s == \"\" {\n",
                "        return Err(ParseError.UnexpectedEnd)\n",
                "    }\n",
                "    Ok(42)\n",
                "}\n",
                "fn main() {\n",
                "    let r = parse(\"\")\n",
                "    is r {\n",
                "        Ok(n) -> print(n)\n",
                "        Err(e) -> {\n",
                "            is e {\n",
                "                ParseError.UnexpectedEnd -> print(\"unexpected end\")\n",
                "                else -> print(\"other error\")\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O47 multi-error nested is",
            concat!(
                "enum ApiError {\n",
                "    Http { code int, msg Str }\n",
                "    Timeout\n",
                "}\n",
                "fn fetch(url Str) !int {\n",
                "    if url == \"\" {\n",
                "        return Err(ApiError.Http(code: 404, msg: \"not found\"))\n",
                "    }\n",
                "    if url == \"slow\" {\n",
                "        return Err(ApiError.Timeout)\n",
                "    }\n",
                "    Ok(200)\n",
                "}\n",
                "fn main() {\n",
                "    let r1 = fetch(\"\")\n",
                "    is r1 {\n",
                "        Ok(data) -> print(data)\n",
                "        Err(e) -> {\n",
                "            is e {\n",
                "                ApiError.Http(code, msg) -> print(code)\n",
                "                ApiError.Timeout -> print(\"timeout\")\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "    let r3 = fetch(\"ok\")\n",
                "    is r3 {\n",
                "        Ok(data) -> print(data)\n",
                "        Err(e) -> print(\"error\")\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        // ---- §6 drafts / contract faces ----
        (
            "O48 !?int nested draft",
            concat!(
                "fn obtain(ok bool) !?int {\n",
                "    if ok {\n",
                "        return Ok(Some(42))\n",
                "    }\n",
                "    return Err(\"bad\")\n",
                "}\n",
                "fn main() int {\n",
                "    is obtain(true) {\n",
                "        Ok(v) -> {\n",
                "            is v {\n",
                "                Some(n) -> { return n }\n",
                "                None -> { return 0 }\n",
                "            }\n",
                "        }\n",
                "        Err(e) -> { return 0 }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O49 tuple across call",
            concat!(
                "fn pair() (int, int) { return (19, 23) }\n",
                "fn main() int { let (a, b) = pair(); return a + b }\n",
            )
            .into(),
        ),
        (
            "O50 duplicate literal arm",
            "fn main() int { return is 1 { 1 -> 40, 1 -> 42, else -> 0 } }\n".into(),
        ),
        (
            "O51 `;` arm separators in enum",
            "enum Answer { Yes { code int, text Str }; No }\nfn main() int {\n let a = Answer.Yes(code: 42, text: \"ok\")\n is a { Answer.Yes(code, text) -> { if text == \"ok\" { return code }; return 0 }; Answer.No -> { return 0 } }\n}\n".into(),
        ),
        (
            "O52 guard arm syntax",
            concat!(
                "enum Reg {\n",
                "    Yes(int)\n",
                "}\n",
                "fn main() int {\n",
                "    let r = Reg.Yes(42)\n",
                "    is r {\n",
                "        Reg.Yes(v) if v > 100 -> { return 1 }\n",
                "        Reg.Yes(v) -> { return v }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O53 or-arm syntax",
            concat!(
                "enum Light {\n",
                "    Red\n",
                "    Green\n",
                "}\n",
                "fn main() int {\n",
                "    let l = Light.Green\n",
                "    is l {\n",
                "        Light.Red | Light.Green -> { return 1 }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "O54 nested is shadowing",
            concat!(
                "enum Outer {\n",
                "    Wrap(int)\n",
                "}\n",
                "fn main() int {\n",
                "    let v = 7\n",
                "    let o = Outer.Wrap(42)\n",
                "    is o {\n",
                "        Outer.Wrap(v) -> {\n",
                "            return v\n",
                "        }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
    ];
    for (label, src) in &faces {
        println!("[p1083-T01] {label} => {}", observe(src));
    }
}

/// T-01 bisect round: split the compound failures above into single faces
/// so the exact missing seam per legacy row is named.
#[test]
fn t01_bisect_faces() {
    let faces: Vec<(&str, String)> = vec![
        (
            "B1 method field read .value",
            concat!(
                "type Counter {\n",
                "    value int\n",
                "    fn get() int { return .value }\n",
                "    static fn new(v int) Counter { return Counter(v) }\n",
                "}\n",
                "fn main() int { let c = Counter.new(10)\n return c.get() }\n",
            )
            .into(),
        ),
        (
            "B2 method field write .value = .value + 1",
            concat!(
                "type Counter {\n",
                "    value int\n",
                "    fn bump() int { .value = .value + 1\n return .value }\n",
                "    static fn new(v int) Counter { return Counter(v) }\n",
                "}\n",
                "fn main() int { var c = Counter.new(10)\n return c.bump() }\n",
            )
            .into(),
        ),
        (
            "B3 void method (no return)",
            concat!(
                "type Counter {\n",
                "    value int\n",
                "    fn set_to(n int) { .value = n }\n",
                "    fn get() int { return .value }\n",
                "    static fn new(v int) Counter { return Counter(v) }\n",
                "}\n",
                "fn main() int { var c = Counter.new(10)\n c.set_to(99)\n return c.get() }\n",
            )
            .into(),
        ),
        (
            "B4 self.a = n setter",
            concat!(
                "type Mixed {\n",
                "    a int\n",
                "    fn set_a(n int) { self.a = n }\n",
                "    fn get_a() int { return self.a }\n",
                "    static fn new(v int) Mixed { return Mixed(v) }\n",
                "}\n",
                "fn main() int { var m = Mixed.new(1)\n m.set_a(42)\n return m.get_a() }\n",
            )
            .into(),
        ),
        (
            "B5 implicit-self method call .helper(3)",
            concat!(
                "type Calc {\n",
                "    base int\n",
                "    fn helper(n int) int { return .base + n }\n",
                "    fn call_helper() int { return .helper(3) }\n",
                "    static fn new(v int) Calc { return Calc(v) }\n",
                "}\n",
                "fn main() int { let c = Calc.new(10)\n return c.call_helper() }\n",
            )
            .into(),
        ),
        (
            "B6 record as fn arg + method on call result",
            concat!(
                "type Cube {\n",
                "    v int\n",
                "    fn get() int { return .v }\n",
                "    static fn new(v int) Cube { return Cube(v) }\n",
                "}\n",
                "fn apply(b Cube) int { return b.get() }\n",
                "fn main() int { return apply(Cube.new(7)) + make(8).get() }\n",
                "fn make(v int) Cube { return Cube.new(v) }\n",
            )
            .into(),
        ),
        (
            "B7 bare let None no annotation",
            "fn main() int { let x = None\n is x { Some(v) -> { return v }; None -> { return 1 } } }\n".into(),
        ),
        (
            "B8 bare let Err(msg)",
            "fn main() int { let d = Err(\"msg\")\n is d { Ok(v) -> { return v }; Err(e) -> { return 0 } } }\n".into(),
        ),
        (
            "B9 single-line non-block arm body",
            "fn main() int { let r = Ok(5)\n is r {\n  Ok(v) -> return v\n  Err(e) -> return 0\n }\n}\n".into(),
        ),
        (
            "B10 print arm body",
            "fn main() { let r = Ok(5)\n is r {\n  Ok(v) -> print(v)\n  Err(e) -> print(0)\n }\n}\n".into(),
        ),
        (
            "B11 implicit Ok tail in !int fn",
            concat!(
                "fn f(b int) !int {\n",
                "    if b == 0 {\n",
                "        return Err(\"zero\")\n",
                "    }\n",
                "    Ok(7)\n",
                "}\n",
                "fn main() int { let r = f(1)\n is r { Ok(v) -> { return v }; Err(e) -> { return 0 } } }\n",
            )
            .into(),
        ),
        (
            "B12 else arm in is block",
            "fn main() int { let x = 5\n is x {\n  1 -> { return 10 }\n  else -> { return 0 }\n }\n}\n".into(),
        ),
        (
            "B13 fstr record display ${p}",
            concat!(
                "type Point { x int }\n",
                "fn main() {\n",
                "    let p = Point { x: 42 }\n",
                "    print(f\"p=${p}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B14 record literal in list literal",
            concat!(
                "type Note { id int }\n",
                "fn main() int {\n",
                "    let notes = [Note { id: 1 }, Note { id: 2 }]\n",
                "    return notes[0].id + notes[1].id\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B15 named payload decl only",
            "enum Box { Holder { v int } }\nfn main() int { let b = Box.Holder(9)\n is b { Box.Holder(v) -> { return v } } }\n".into(),
        ),
        (
            "B15b named 2-field",
            "enum Box2 { Holder { a int; b int } }\nfn main() int { let b = Box2.Holder(3, 4)\n is b { Box2.Holder(x, y) -> { return x + y } } }\n".into(),
        ),
        (
            "B16 enum int arm statement is",
            "fn main() int { let x = 2\n is x {\n  1 -> { return 10 }\n  2 -> { return 20 }\n }\n}\n".into(),
        ),
        (
            "B17 is expr as value operand",
            "fn main() int { return is 1 { 1 -> 40, else -> 0 } }\n".into(),
        ),
        (
            "B18 is expr as script tail",
            "let opt = Some(42)\nis opt {\n  Some(v) -> v\n  None -> 0\n}\n".into(),
        ),
        (
            "B19 ?int let ann only",
            "fn main() int { let x ?int = Some(3)\n is x { Some(v) -> { return v }; None -> { return 0 } } }\n".into(),
        ),
        (
            "B20 enum decl `;` separated single payload",
            "enum A { X(int); Y }\nfn main() int { let a = A.X(4)\n is a { A.X(v) -> { return v }; A.Y -> { return 0 } } }\n".into(),
        ),
        (
            "B21 two-elem tuple payload construct only",
            "enum A { P(int, int) }\nfn main() int { let p = A.P(3, 4)\n is p { A.P(a, b) -> { return a + b } } }\n".into(),
        ),
        (
            "B22 nested record payload enum",
            "type Pt { x int; y int }\nenum Msg { Move(Pt) }\nfn main() int { let m = Msg.Move(Pt { x: 3, y: 4 })\n is m { Msg.Move(p) -> { return p.x + p.y } } }\n".into(),
        ),
        (
            "B23 List<Record> elem type",
            concat!(
                "type Item { v int }\n",
                "fn main() int {\n",
                "    let xs = List<Item>.new()\n",
                "    return xs.len()\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B24 enum arm payload Str",
            "enum E { Tag(Str) }\nfn main() { let e = E.Tag(\"hi\")\n is e { E.Tag(s) -> { print(s) } } }\n".into(),
        ),
        (
            "B25 scalar enum use as int",
            "enum Color { Red = 1, Green = 2 }\nfn main() int { let c = Color.Red\n return c }\n".into(),
        ),
        (
            "B33 no-binding arm on payload variant",
            "enum Color {\n    RGB { r int, g int, b int }\n}\nfn main() {\n    let c1 = Color.RGB(100, 150, 200)\n    is c1 {\n        Color.RGB -> print(\"rgb\")\n    }\n}\n".into(),
        ),
        (
            "B34 guard duplicate disc",
            "enum Reg {\n    Yes(int)\n    No\n}\nfn main() int {\n    let r = Reg.Yes(42)\n    is r {\n        Reg.Yes(v) if v > 100 -> { return 1 }\n        Reg.Yes(v) -> { return v }\n        Reg.No -> { return 0 }\n    }\n}\n".into(),
        ),
        (
            "B35 Err enum payload",
            "enum ParseError {\n    UnexpectedEnd\n}\nfn parse(s Str) !int {\n    if s == \"\" {\n        return Err(ParseError.UnexpectedEnd)\n    }\n    Ok(42)\n}\nfn main() {\n    let r = parse(\"\")\n    is r {\n        Ok(n) -> print(n)\n        Err(e) -> {\n            is e {\n                ParseError.UnexpectedEnd -> print(\"unexpected end\")\n                else -> print(\"other error\")\n            }\n        }\n    }\n}\n".into(),
        ),
        (
            "B26 array literal trailing comma",
            "fn main() int {\n    let xs = [1, 2, 3,]\n    return xs[0] + xs[1] + xs[2]\n}\n".into(),
        ),
        (
            "B27 record literal array trailing comma",
            concat!(
                "type Note { id int }\n",
                "fn main() int {\n",
                "    let notes = [\n",
                "        Note { id: 1 },\n",
                "        Note { id: 2 },\n",
                "    ]\n",
                "    return notes[0].id + notes[1].id\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B28 O39 multi-let a/b/c/d",
            concat!(
                "fn main() int {\n",
                "    let a = Some(42)\n",
                "    let b = None\n",
                "    let c = Ok(100)\n",
                "    let d = Err(\"msg\")\n",
                "    is a { Some(v) -> { return v }; None -> { return 0 } }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B29 free fn call inside fstr",
            concat!(
                "fn double(x int) int { return x * 2 }\n",
                "fn main() {\n",
                "    print(f\"d=${double(21)}\")\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B30 O39 without unused b",
            concat!(
                "fn main() int {\n",
                "    let a = Some(42)\n",
                "    let c = Ok(100)\n",
                "    let d = Err(\"msg\")\n",
                "    is a { Some(v) -> { return v }; None -> { return 0 } }\n",
                "}\n",
            )
            .into(),
        ),
        (
            "B31 let b = None then is b",
            "fn main() int { let a = Some(42)\n let b = None\n is b { Some(v) -> { return v }; None -> { return 1 } } }\n".into(),
        ),
        (
            "B32 v08 union/tag through ORDINARY entry",
            concat!(
                "tag Mark { Hit(int); Miss(int) }\n",
                "union Val { Num(int); Flag(int) }\n",
                "fn main() int {\n",
                "    let m = Mark.Hit(20)\n",
                "    let v = Val.Num(22)\n",
                "    is m {\n",
                "        Mark.Hit(a) -> {\n",
                "            is v {\n",
                "                Val.Num(b) -> { return a + b }\n",
                "                Val.Flag(b) -> { return b }\n",
                "            }\n",
                "        }\n",
                "        Mark.Miss(a) -> { return a }\n",
                "    }\n",
                "}\n",
            )
            .into(),
        ),
    ];
    for (label, src) in &faces {
        println!("[p1083-B] {label} => {}", observe(src));
    }
}

/// T-02 temp: print full diagnostics for the remaining red faces.
#[test]
fn t02_infra_diag() {
    let faces: &[(&str, &str)] = &[
        (
            "B15 named payload",
            concat!(
                "enum Box { Holder { v int } }\n",
                "fn main() int { let b = Box.Holder(9)\n",
                " is b { Box.Holder(v) -> { return v } } }\n",
            ),
        ),
        (
            "B25 scalar enum",
            concat!(
                "enum Color { Red = 1, Green = 2 }\n",
                "fn main() int { let c = Color.Red\n return c }\n",
            ),
        ),
        (
            "B2 method field write",
            concat!(
                "type Counter {\n",
                "    value int\n",
                "    fn bump() int { .value = .value + 1\n return .value }\n",
                "    static fn new(v int) Counter { return Counter(v) }\n",
                "}\n",
                "fn main() int { var c = Counter.new(10)\n return c.bump() }\n",
            ),
        ),
        (
            "O47 multi-error nested is",
            concat!(
                "enum ApiError {\n",
                "    Http { code int, msg Str }\n",
                "    Timeout\n",
                "}\n",
                "fn fetch(url Str) !int {\n",
                "    if url == \"\" {\n",
                "        return Err(ApiError.Http(code: 404, msg: \"not found\"))\n",
                "    }\n",
                "    if url == \"slow\" {\n",
                "        return Err(ApiError.Timeout)\n",
                "    }\n",
                "    Ok(200)\n",
                "}\n",
                "fn main() {\n",
                "    let r1 = fetch(\"\")\n",
                "    is r1 {\n",
                "        Ok(data) -> print(data)\n",
                "        Err(e) -> {\n",
                "            is e {\n",
                "                ApiError.Http(code, msg) -> print(code)\n",
                "                ApiError.Timeout -> print(\"timeout\")\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "}\n",
            ),
        ),
        (
            "O32 enum instance method",
            concat!(
                "enum Light {\n",
                "    Red\n",
                "    Green\n",
                "    fn is_red() bool {\n",
                "        is self {\n",
                "            Light.Red -> return true\n",
                "        }\n",
                "        return false\n",
                "    }\n",
                "}\n",
                "fn main() {\n",
                "    if Light.Red.is_red() { print(\"yes\") } else { print(\"no\") }\n",
                "}\n",
            ),
        ),
        (
            "B35 Err enum payload",
            concat!(
                "enum ParseError {\n",
                "    UnexpectedEnd\n",
                "}\n",
                "fn parse(s Str) !int {\n",
                "    if s == \"\" {\n",
                "        return Err(ParseError.UnexpectedEnd)\n",
                "    }\n",
                "    return Ok(42)\n",
                "}\n",
                "fn main() {\n",
                "    let r = parse(\"\")\n",
                "    is r {\n",
                "        Ok(n) -> print(n)\n",
                "        Err(e) -> {\n",
                "            is e {\n",
                "                ParseError.UnexpectedEnd -> print(\"unexpected end\")\n",
                "                else -> print(\"other error\")\n",
                "            }\n",
                "        }\n",
                "    }\n",
                "}\n",
            ),
        ),
    ];
    for (label, src) in faces {
        match ace_core::source::compile_language_source(src, "p1083.at", &[]) {
            Ok(_) => println!("[p1083-DIAG] {label} => OK"),
            Err(ds) => {
                for d in &ds {
                    println!(
                        "[p1083-DIAG] {label} => {:?}:{}:{} span={:?}",
                        d.stage, d.code, d.message, d.span
                    );
                }
            }
        }
    }
    let scalar_variants: &[(&str, &str)] = &[
        (
            "S1 single = N arm",
            "enum C { R = 1 }\nfn main() int { return 2 }\n",
        ),
        (
            "S2 newline separated = N",
            "enum C { R = 1\n G = 2 }\nfn main() int { return 2 }\n",
        ),
        (
            "S3 comma separated = N",
            "enum C { R = 1, G = 2 }\nfn main() int { return 2 }\n",
        ),
        (
            "S4 named fields comma",
            "enum B { H { a int, b int } }\nfn main() int { return 2 }\n",
        ),
        (
            "S5 named fields newline",
            "enum B { H { a int\n b int } }\nfn main() int { return 2 }\n",
        ),
        (
            "S6 named fields semi",
            "enum B { H { a int; b int } }\nfn main() int { return 2 }\n",
        ),
        (
            "S9 str-field record in array literal",
            concat!(
                "type Note { id int; title Str }
",
                "fn main() int {
",
                "    let notes = [
",
                "        Note { id: 1, title: \"Welcome\" },
",
                "    ]
",
                "    let n = notes[0]
",
                "    if n.title == \"Welcome\" { return 1 }
",
                "    return 0
",
                "}
",
            ),
        ),
        (
            "S7 enum arms comma plain",
            "enum C { R\n G }\nfn main() int { let c = C.R\n is c { C.R -> { return 1 } C.G -> { return 2 } } }\n",
        ),
        (
            "S8 enum arms comma",
            "enum C { R, G }\nfn main() int { let c = C.R\n is c { C.R -> { return 1 } C.G -> { return 2 } } }\n",
        ),
    ];
    for (label, src) in scalar_variants {
        match ace_core::source::compile_language_source(src, "p1083.at", &[]) {
            Ok(_) => println!("[p1083-DIAG] {label} => OK"),
            Err(ds) => {
                for d in &ds {
                    println!(
                        "[p1083-DIAG] {label} => {:?}:{}:{} span={:?}",
                        d.stage, d.code, d.message, d.span
                    );
                }
            }
        }
    }
}
