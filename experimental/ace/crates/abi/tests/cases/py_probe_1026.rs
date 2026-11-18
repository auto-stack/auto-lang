//! PLAN-1026 T-02 probes (scratch): the AGGREGATES py training lane —
//! surface isolation, the runtime-frame lane, and the §5.1 acceptance
//! training program with the use.py sugar. Folded into permanent tests
//! at T-05; deleted then.
#![cfg(windows)]

use ace_abi::rust_provider::open_py_bridge_session;
use ace_av_core::contract::ExecOutcome;
use ace_av_core::foreign_exec::ForeignStubTable;

const FLOAT_THEN_CALL: &str = r#"use.py math: sqrt
fn main() int { var lr = 0.1d
var q = sqrt(4)
return q.to(i32) }"#;

const CALL_THEN_FLOAT: &str = r#"use.py math: sqrt
fn main() int { var q = sqrt(4)
var lr = 0.1d
return q.to(i32) }"#;

const DOTTED_FLOAT: &str = r#"use.py torch.nn: Linear, MSELoss
fn main() int { var lr = 0.1d
return 1 }"#;

const MULTI_FLOAT: &str = r#"use.py math: sqrt
use.py math: pi
fn main() int { var lr = 0.1d
return 1 }"#;

const THREE_FLOAT: &str = r#"use.py torch: manual_seed
use.py torch.nn: Linear
use.py torch.optim: SGD
fn main() int { var lr = 0.1d
return 1 }"#;

const FLOAT_KWARG2: &str = r#"use.py math: sqrt
fn s(x int, lr double) double { return lr }
fn main() int { var q = s(3, lr: 0.1d)
return q.to(i32) }"#;

const FORCOND_LITERAL: &str = r#"fn d(v int) int { for v >= 10 { v = v + 1 }
return v }
fn main() int { return d(0) }"#;

const FORCOND_PAREN: &str = r#"fn d(n int) int { var left = (n + 3) / 4
for left >= 1 { left = left - 1 }
return left }
fn main() int { return d(8) }"#;

const FORCOND_FSTR: &str = r#"fn d(v int) int { var s = 0
for v >= 1 { var w = f"a${v}b"
s = w.len()
v = v - 1 }
return s }
fn main() int { return d(3) }"#;

const FORCOND_I64: &str = r#"fn d(n int) int { var k = n.to(i64)
for k >= 1 { k = k - 1 }
return k.to(i32) }
fn main() int { return d(5) }"#;

const USE_PY_MATH: &str = r#"use.py math: sqrt
fn main() int { return 1 }"#;

const USE_PY_DOTTED: &str = r#"use.py torch.nn: Linear, MSELoss
fn main() int { return 1 }"#;

const ESC_NL: &str = "fn main() int { var s = \"a\nb\"
return s.len_bytes() }";

const FSTR_CALL_BODY: &str = r#"fn d(v int) int { var s = 0
for v >= 1 { var w = f"x${__dl(v)}y"
s = w.len()
v = v - 1 }
return s }
fn __dl(v int) int { return v }
fn main() int { return d(3) }"#;

const PYTEXT_CLONE: &str = r#"fn __py_dlen(n int) int { var d = 1
var v = n
for v >= 10 { v = v / 10
d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var i = 0
var left = (n + 3) / 4
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = wf.len()
var k = 0
for k < 4 {
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const PYTEXT_V2: &str = r#"fn __py_dlen(n int) int { var d = 1
var v = n
for v >= 10 { v = v / 10
d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var i = 0
var left = (n + 3) / 4
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = wf.len()
var k = 0
for k < 4 {
var d = 1
if k == 1 { d = 10 }
if k == 2 { d = 100 }
if k == 3 { d = 1000 }
var b = w / d % 256
if b != 0 { s = s + char.from_codepoint(b).to(Str) }
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const V2A: &str = r#"fn __py_text(n int) Str {
var s = ""
var left = 4
for left >= 1 {
var k = 0
for k < 4 {
var d = 1
if k == 1 { d = 10 }
if k == 2 { d = 100 }
if k == 3 { d = 1000 }
var b = 7 / d % 256
if b != 0 { s = s + "x" }
k = k + 1 }
left = left - 1 }
return s }
fn main() int { return __py_text(4).len() }"#;

const V2B: &str = r#"fn __py_text(n int) Str {
var s = ""
var left = 4
for left >= 1 {
var k = 0
for k < 4 {
k = k + 1 }
left = left - 1 }
return s }
fn main() int { return __py_text(4).len() }"#;

const V2C: &str = r#"fn __py_text(n int) Str {
var s = ""
var left = 4
for left >= 1 {
var b = 65
s = s + char.from_codepoint(b).to(Str)
left = left - 1 }
return s }
fn main() int { return __py_text(4).len() }"#;

const V2D: &str = r#"fn __py_dlen(n int) int { var d = 1
var v = n
for v >= 10 { v = v / 10
d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var i = 0
var left = (n + 3) / 4
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = wf.len()
var k = 0
var d = 1
var b = w / d % 256
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const FN_A: &str = "fn z(a int, b int) int { return a + b }
fn main() int { return z(1, 2) }";
const FN_B: &str = "fn z(a int) f64 { return 1.0d }
fn main() int { return z(1).to(i32) }";
const FN_C: &str = "fn z(a int, b int) f64 { return 1.0d }
fn main() int { return z(1, 2).to(i32) }";

const F64_FULL: &str = r#"fn __py_f64(hi int, lo int) f64 {
var neg = 0
if hi < 0 { neg = 1 }
var h = hi.to(i64)
var l = lo.to(i64)
if h < 0 { h = h + 2147483648i64 }
if l < 0 { l = l + 2147483648i64 }
var e = h.shr(20i64) % 2048i64
var mant = (h % 1048576i64) * 4294967296i64 + l
var sig = (4503599627370496i64 + mant).to(double)
var kk = e - 1075i64
var p = 1.0d
var t = 2.0d
var up = kk.to(i32)
for q in 0 .. up { p = p * t }
for q in up .. 0 { p = p / t }
var v = sig * p
if neg != 0 { v = 0.0d - v }
return v }
fn main() int { return __py_f64(0, 0).to(i32) }"#;

const F64_NORANGE: &str = r#"fn __py_f64(hi int, lo int) f64 {
var neg = 0
if hi < 0 { neg = 1 }
var h = hi.to(i64)
var l = lo.to(i64)
if h < 0 { h = h + 2147483648i64 }
if l < 0 { l = l + 2147483648i64 }
var e = h.shr(20i64) % 2048i64
var mant = (h % 1048576i64) * 4294967296i64 + l
var sig = (4503599627370496i64 + mant).to(double)
var v = sig
if neg != 0 { v = 0.0d - v }
return v }
fn main() int { return __py_f64(0, 0).to(i32) }"#;

const PYTEXT_V3: &str = r#"fn __py_dlen(n int) int {
var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
if n >= 1000 { d = d + 1 }
if n >= 10000 { d = d + 1 }
if n >= 100000 { d = d + 1 }
if n >= 1000000 { d = d + 1 }
if n >= 10000000 { d = d + 1 }
if n >= 100000000 { d = d + 1 }
if n >= 1000000000 { d = d + 1 }
return d }
fn __py_f64(hi int, lo int) f64 {
var neg = 0
if hi < 0 { neg = 1 }
var h = hi.to(i64)
var l = lo.to(i64)
if h < 0 { h = h + 2147483648i64 }
if l < 0 { l = l + 2147483648i64 }
var e = h.shr(20i64) % 2048i64
var mant = (h % 1048576i64) * 4294967296i64 + l
var sig = (4503599627370496i64 + mant).to(double)
var kk = e - 1075i64
var p = 1.0d
var t = 2.0d
var up = kk.to(i32)
var mul = 0
var div = 0
if up > 0 { mul = 1 }
if up < 0 { div = 1 }
var a = up
if a < 0 { a = 0 - a }
var q = 0
for q < 1075 {
if mul == 1 && a >= 1 { p = p * t }
if div == 1 && a >= 1 { p = p / t }
a = a - 1
q = q + 1 }
var v = sig * p
if neg != 0 { v = 0.0d - v }
return v }
fn __py_text(n int) Str {
var s = ""
var words = (n + 3) / 4
var i = 0
for i < words {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = py_str_word(wf.slice(0, wf.len()))
if w < 0 { w = w + 2147483647 }
if w < 0 { w = w + 1 }
var k = 0
for k < 4 {
var d = 1
if k == 1 { d = 10 }
if k == 2 { d = 100 }
if k == 3 { d = 1000 }
var b = w / d % 256
if b != 0 {
var c = char.from_codepoint(b)
var cs = c.to(Str)
s = s + cs }
k = k + 1 }
i = i + 1 }
return s }
fn main() int { return 0 }"#;

const V4_FSTR_IFS: &str = r#"fn __py_dlen(n int) int { var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var left = (n + 3) / 4
var i = 0
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = wf.len()
var k = 0
for k < 4 {
if k == 1 { w = w + 10 }
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const V5_FOREIGN: &str = r#"fn __py_dlen(n int) int { var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var left = (n + 3) / 4
var i = 0
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = py_str_word(wf.slice(0, wf.len()))
var k = 0
for k < 4 {
if k == 1 { w = w + 10 }
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const V6_FULL: &str = r#"fn __py_dlen(n int) int { var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var left = (n + 3) / 4
var i = 0
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = py_str_word(wf.slice(0, wf.len()))
if w < 0 { w = w + 2147483647 }
if w < 0 { w = w + 1 }
var k = 0
for k < 4 {
var d = 1
if k == 1 { d = 10 }
if k == 2 { d = 100 }
if k == 3 { d = 1000 }
var b = w / d % 256
if b != 0 {
var c = char.from_codepoint(b)
var cs = c.to(Str)
s = s + cs }
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const V7_NOCONCAT: &str = r#"fn __py_dlen(n int) int { var d = 1
if n >= 10 { d = d + 1 }
if n >= 100 { d = d + 1 }
return d }
fn __py_text(n int) Str {
var s = ""
var left = (n + 3) / 4
var i = 0
for left >= 1 {
var wf = f"p967.frame/1
r n 0

p 1
i ${__py_dlen(i)}
${i}
k 0
"
var w = py_str_word(wf.slice(0, wf.len()))
if w < 0 { w = w + 2147483647 }
if w < 0 { w = w + 1 }
var k = 0
for k < 4 {
var d = 1
if k == 1 { d = 10 }
if k == 2 { d = 100 }
if k == 3 { d = 1000 }
var b = w / d % 256
if b != 0 {
var c = char.from_codepoint(b)
var cs = c.to(Str)
var n2 = cs.len()
var n3 = n2 + b }
k = k + 1 }
i = i + 1
left = left - 1 }
return s }
fn main() int { return __py_text(8).len() }"#;

const FLOAT_VAR: &str = r#"use.py math: sqrt
fn main() int { var lr = 0.1d
return 1 }"#;

const FLOAT_KWARG: &str = r#"use.py math: sqrt
fn s(lr double) double { return lr }
fn main() int { var q = s(lr: 0.1d)
return q.to(i32) }"#;

/// The §5.1 acceptance program (verbatim body; `lr: 0.1d` is the one
/// textual adaptation — the 961-frozen float spelling; `fn main() int` +
/// `return 0` wrap the script for the lane's entry contract).
const ACCEPTANCE: &str = r#"use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() int {
    var lr_v = 0.1d
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var mparams = model.parameters()
    var optimizer = SGD(mparams, lr: lr_v)

    var step = 0
    for step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(loss.to(float))
        step = step + 1
    }

    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
    return 0
}"#;

fn caps() -> Vec<String> {
    vec![
        ace_foreign_contract::manual::SOURCE_CAP.to_string(),
        ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
    ]
}

/// The full AGGREGATES py lane pipeline (compile → closures lower →
/// foreign ABC admission → combined console+foreign execution); returns
/// the outcome plus the captured stdout.
fn run_lane(
    src: &str,
    admitted: &ace_foreign_contract::manual::AdmittedForeign,
    table: &mut ForeignStubTable,
) -> Result<(ExecOutcome, Vec<u8>), String> {
    let compiled = ace_core::source::compile_aggregates_source_with_foreign(
        src,
        "pylane.at",
        &caps(),
        admitted,
    )
    .map_err(|ds| format!("compile: {ds:?}"))?;
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    ace_av_core::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
        .map_err(|e| format!("admit: {e:?}"))?;
    let mut capture = ace_av::console_host::ConsoleCapture {
        stdout: Vec::new(),
        stderr: Vec::new(),
        dispatches: 0,
    };
    let (out, _) = ace_av_core::av::execute_console_foreign(
        &loaded,
        &ace_av_core::av::ExecLimits::default(),
        &ace_av_core::av::CancelFlag::new(false),
        &["console.stdout".to_string()],
        &mut capture,
        table,
    )
    .map_err(|e| format!("execute: {e:?}"))?;
    Ok((out, capture.stdout))
}

fn run_lane_limited(
    src: &str,
    admitted: &ace_foreign_contract::manual::AdmittedForeign,
    table: &mut ForeignStubTable,
    limits: ace_av_core::av::ExecLimits,
) -> Result<(ExecOutcome, Vec<u8>), String> {
    let compiled = ace_core::source::compile_aggregates_source_with_foreign(
        src,
        "pylane.at",
        &caps(),
        admitted,
    )
    .map_err(|ds| format!("compile: {ds:?}"))?;
    let mir = ace_lowering::lower_checked_closures(&compiled.checked, "d_main", 0)
        .map_err(|e| format!("lower: {e:?}"))?;
    ace_av_core::verify_mir(&mir).map_err(|e| format!("mir: {e:?}"))?;
    let encoded = ace_av_core::abc::encode_abc(&mir).unwrap().to_flat();
    let decoded = ace_av_core::abc::decode_abc(&encoded).unwrap();
    let loaded = ace_av_core::abc_verify::verify_abc_foreign(decoded, &mir.owner, table)
        .map_err(|e| format!("admit: {e:?}"))?;
    let mut capture = ace_av::console_host::ConsoleCapture {
        stdout: Vec::new(),
        stderr: Vec::new(),
        dispatches: 0,
    };
    let (out, _) = ace_av_core::av::execute_console_foreign(
        &loaded,
        &limits,
        &ace_av_core::av::CancelFlag::new(false),
        &["console.stdout".to_string()],
        &mut capture,
        table,
    )
    .map_err(|e| format!("execute: {e:?}"))?;
    Ok((out, capture.stdout))
}

#[test]
fn p1026_probe_aggregates_parse_isolation() {
    let f0 = open_py_bridge_session(1).expect("session");
    for (what, src) in [
        ("fn two params int", FN_A),
        ("fn one param f64 ret", FN_B),
        ("fn two params f64 ret", FN_C),
        ("f64 full", F64_FULL),
        ("f64 norange", F64_NORANGE),
        ("pytext v3 final", PYTEXT_V3),
        ("v4 fstr+ifs", V4_FSTR_IFS),
        ("v5 foreign arg", V5_FOREIGN),
        ("v6 full", V6_FULL),
        ("v7 noconcat", V7_NOCONCAT),
        ("float kwarg", FLOAT_KWARG),
        ("float then call", FLOAT_THEN_CALL),
        ("call then float", CALL_THEN_FLOAT),
        ("dotted+float", DOTTED_FLOAT),
        ("multi use.py+float", MULTI_FLOAT),
        ("three use.py+float", THREE_FLOAT),
        ("float kwarg pos+kw", FLOAT_KWARG2),
        ("float var", FLOAT_VAR),
        ("esc newline str", ESC_NL),
        ("pytext clone", PYTEXT_CLONE),
        ("pytext v2 ops", PYTEXT_V2),
        ("v2a ifs+arith", V2A),
        ("v2b plain loops", V2B),
        ("v2c char chain", V2C),
        ("v2d fstr+arith", V2D),
        ("fstr call body", FSTR_CALL_BODY),
        ("forcond literal", FORCOND_LITERAL),
        ("forcond paren", FORCOND_PAREN),
        ("forcond fstr", FORCOND_FSTR),
        ("forcond i64", FORCOND_I64),
        ("use.py math", USE_PY_MATH),
        ("use.py dotted", USE_PY_DOTTED),
    ] {
        let r = ace_core::source::compile_aggregates_source_with_foreign(
            src,
            "iso.at",
            &caps(),
            &f0.admitted,
        );
        match r {
            Ok(_) => eprintln!("ISO [{what}] => OK"),
            Err(ds) => eprintln!("ISO [{what}] => {:?}", ds),
        }
    }
}

const ACCEPTANCE_NOPRINT: &str = r#"use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() int {
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var mparams = model.parameters()
    var lr_v = 0.1d
    var optimizer = SGD(mparams, lr: lr_v)
    var step = 0
    for step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        step = step + 1
    }
    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
    return 0
}"#;

/// The §5.1 acceptance program through the full py lane with the sugar.
#[test]
fn p1026_probe_acceptance_training() {
    let mut f0 = open_py_bridge_session(1).expect("session");
    let t0 = std::time::Instant::now();
    // generous step/wall budgets for the 60-step training run (the f64
    // reconstruction's pow2 loop is instruction-heavy by design)
    let limits = ace_av_core::av::ExecLimits {
        max_steps: 200_000_000,
        wall: std::time::Duration::from_secs(120),
        ..Default::default()
    };
    let (out, stdout) = run_lane_limited(ACCEPTANCE, &f0.admitted, f0.session.table(), limits)
        .unwrap_or_else(|e| panic!("lane: {e}"));
    eprintln!("TRAIN outcome: {out:?} ({} ms)", t0.elapsed().as_millis());
    let text0 = String::from_utf8_lossy(&stdout).to_string();
    eprintln!("STDOUT lines before outcome: {}", text0.lines().count());
    eprintln!(
        "last 3: {:?}",
        text0.lines().rev().take(3).collect::<Vec<_>>()
    );
    assert!(matches!(out, ExecOutcome::Success { .. }), "run: {out:?}");
    let text = text0;
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 62, "60 losses + shape + pred0: {text}");
    let first: f32 = lines[0].parse().expect("first loss f32");
    let last: f32 = lines[59].parse().expect("last loss f32");
    eprintln!(
        "first={first} last={last} shape={:?} pred0={}",
        lines[60], lines[61]
    );
    assert!(
        (first - 0.2370201051235199f32).abs() < 1e-6,
        "first loss {first}"
    );
    assert!(
        (last - 0.1013089045882225f32).abs() < 1e-6,
        "last loss {last}"
    );
    assert!(last < first, "training converges");
    assert_eq!(lines[60], "torch.Size([4, 1])", "shape display");
    let pred0: f32 = lines[61].parse().expect("pred0 f32");
    assert!(
        (pred0 - 0.4728480279445648f32).abs() < 1e-5,
        "pred0 {pred0}"
    );
}

/// Bisect: the training loop WITHOUT the per-iteration float print.
#[test]
fn p1026_probe_acceptance_noprint() {
    let mut f0 = open_py_bridge_session(1).expect("session");
    let limits = ace_av_core::av::ExecLimits {
        max_steps: 200_000_000,
        wall: std::time::Duration::from_secs(120),
        ..Default::default()
    };
    let (out, stdout) =
        run_lane_limited(ACCEPTANCE_NOPRINT, &f0.admitted, f0.session.table(), limits)
            .unwrap_or_else(|e| panic!("lane: {e}"));
    let text = String::from_utf8_lossy(&stdout).to_string();
    eprintln!("NOPRINT outcome: {out:?}; stdout: {text:?}");
    assert!(matches!(out, ExecOutcome::Success { .. }), "run: {out:?}");
}

/// Isolation: a single indexed to(float) print (the second acceptance print).
#[test]
fn p1026_probe_index_tofloat() {
    let mut f0 = open_py_bridge_session(1).expect("session");
    let src = r#"use.py math: sqrt
fn main() int {
    var q = sqrt(4)
    print(q.to(float))
    return 0
}"#;
    let limits = ace_av_core::av::ExecLimits {
        max_steps: 20_000_000,
        wall: std::time::Duration::from_secs(30),
        ..Default::default()
    };
    let t0 = std::time::Instant::now();
    let (out, stdout) = run_lane_limited(src, &f0.admitted, f0.session.table(), limits)
        .unwrap_or_else(|e| panic!("lane: {e}"));
    eprintln!(
        "TOFLOAT outcome: {out:?} ({} ms) stdout={:?}",
        t0.elapsed().as_millis(),
        String::from_utf8_lossy(&stdout)
    );
    assert!(matches!(out, ExecOutcome::Success { .. }), "run: {out:?}");
}
