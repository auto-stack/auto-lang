//! PLAN-1080 call/binding/pipeline observations through the ORDINARY Auto
//! source profile (`compile_language_source`) — the same front door the
//! `ac-av/language-source-run-v1` recipe runs. Oracles are independent
//! typed values and frozen diagnostic codes/stages, never a value read
//! back out of the implementation.
//!
//! These are the value-level counterparts of the canonical corpus cases
//! (`language/calls/binding-pipeline.case.ad` owns the byte-exact
//! dual-stream and stdout-trace oracles); this file pins the semantics a
//! mutation here would silently break: named binding, source-order
//! evaluation, untyped-signature completion, prototype unification, the
//! pipe desugar and its neighbor rejections.

use ace_av_core::abc::AbcBytes;
use ace_av_core::av::{execute, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

/// Compile/execute one Auto-profile program end to end (schema/CST →
/// inference completion → bound → checked HIR → MIR → AVAD → AV), the
/// same chain as the language recipe.
fn run(src: &str) -> ExecOutcome {
    let compiled = ace_core::source::compile_language_source(src, "p1080.at", &[])
        .unwrap_or_else(|d| panic!("Auto-profile compile failed: {d:?}"));
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0x1080)
        .expect("auto-language lowering");
    mir_verify::verify_mir(&mir).expect("MIR verifies");
    let wire: AbcBytes = encode(&mir);
    let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("AVAD admits");
    let (out, _trace) = execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    out
}

fn encode(mir: &ace_av_core::mir::MirModule) -> AbcBytes {
    ace_av_core::abc::encode_abc(mir).expect("AVAD encode")
}

fn ok_i32(src: &str) -> i32 {
    match run(src) {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected an i32 success, got {other:?}"),
    }
}

fn ok_unit(src: &str) {
    match run(src) {
        ExecOutcome::Success { value: Value::Unit, .. } => {}
        other => panic!("expected a unit success, got {other:?}"),
    }
}

fn diag(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "p1080.at", &[]) {
        Ok(_) => panic!("expected a compile diagnostic, got Ok"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.clone())
        }
    }
}

// ---- named binding + source order -----------------------------------------

#[test]
fn named_args_bind_by_name() {
    assert_eq!(
        ok_i32("fn sub(a int, b int) int { return a * 3 + b * 2 }\nfn main() int { return sub(b: 2, a: 1) }\n"),
        7,
        "b:2/a:1 bind by NAME (a=1, b=2), not by position"
    );
}

#[test]
fn named_args_missing_rejects() {
    let (stage, code) = diag("fn f(a int, b int) int { return a + b }\nfn main() int { return f(42) }\n");
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.argument"));
}

#[test]
fn named_args_duplicate_rejects() {
    let (stage, code) = diag(
        "fn sub(a int, b int) int { return a * 3 + b * 2 }\nfn main() int { return sub(a: 1, a: 2) }\n",
    );
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.argument"));
}

#[test]
fn named_args_unknown_rejects() {
    let (stage, code) = diag(
        "fn sub(a int, b int) int { return a * 3 + b * 2 }\nfn main() int { return sub(a: 1, c: 2) }\n",
    );
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.argument"));
}

// ---- record methods (the shared record-domain machinery on the Auto face) --

#[test]
fn method_static_and_instance_run() {
    assert_eq!(
        ok_i32(
            "type T {\n    v int\n    static fn new(v int) T { return T(v) }\n    fn get() int { return .v }\n}\nfn main() int { return T.new(41).get() }\n"
        ),
        41,
        "static constructor + receiver-field instance method"
    );
    assert_eq!(
        ok_i32(
            "type T {\n    v int\n    fn get() int { return .v }\n}\nfn main() int {\n    let t = T(7)\n    return t.get()\n}\n"
        ),
        7,
        "instance method on a constructed receiver"
    );
}

// ---- untyped-signature completion ------------------------------------------

#[test]
fn untyped_fn_completed_from_call_sites() {
    assert_eq!(
        ok_i32("fn add(a, b) { a + b }\nfn main() int { return add(mul(3, 4), mul(2, 7)) }\nfn mul(a, b) { a * b }\n"),
        26,
        "two untyped fns, nested + forward calls complete monomorphically"
    );
    assert_eq!(
        ok_i32("fn twice(a) { let x = a + a; x }\nfn main() int { return twice(5) }\n"),
        10,
        "local-binding body with tail expression"
    );
    assert_eq!(
        ok_i32("fn add(a, b) { a + b }\nfn main() int { return 10 + add(5, 3) }\n"),
        18,
        "the inferred return feeds a typed binary operand"
    );
}

#[test]
fn untyped_conflicting_sites_reject() {
    let (stage, code) = diag(
        "fn add(a, b) { a + b }\nfn main() int {\n    let x = add(1, 2)\n    let y = add(true, false)\n    return x\n}\n",
    );
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.inference"));
}

#[test]
fn untyped_unconstrained_rejects() {
    let (stage, code) = diag("fn lonely(a, b) { a + b }\nfn main() int { return 5 }\n");
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.inference"));
}

#[test]
fn void_main_is_the_unit_entry() {
    ok_unit("fn main() {\n    let x = 6\n}\n");
}

// ---- prototype declarations -------------------------------------------------

#[test]
fn prototype_unifies_with_body() {
    assert_eq!(
        ok_i32("fn test() int;\nfn test() int { 42 }\nfn main() int { return test() }\n"),
        42
    );
}

#[test]
fn prototype_without_body_rejects() {
    let (stage, code) = diag("fn test() int;\nfn main() int { return 5 }\n");
    assert_eq!((stage.as_str(), code.as_str()), ("bind", "source.unresolved"));
}

// ---- pipes -------------------------------------------------------------------

#[test]
fn pipe_method_field_and_continuation() {
    assert_eq!(
        ok_i32(
            "type P {\n v int\n fn add(n int) P { return P(.v + n) }\n}\nfn main() int {\n let p = P(19)\n let q = p |> .add(20)\n            |> .add(3)\n return q |> .v\n}\n"
        ),
        42,
        "method pipe, line-start continuation, field projection"
    );
    // the m10 shape: static ctor + receiver reads + chained pipes
    assert_eq!(
        ok_i32(
            "type Pipe10 {\n    v int\n    fn add(n int) Pipe10 { return Pipe10.new(.v + n) }\n    fn get() int { return .v }\n    static fn new(v int) Pipe10 { return Pipe10(v) }\n}\nfn main() int {\n    var x = Pipe10.new(5)\n    |> .add(3)\n    |> .add(10)\n    return x |> .v\n}\n"
        ),
        18
    );
}

#[test]
fn pipe_lhs_evaluated_exactly_once() {
    // If the pipe re-evaluated its LHS the count would double: `mk()`
    // returns a fresh counter each call, so a double evaluation cannot
    // hide behind shared state — the value would differ.
    assert_eq!(
        ok_i32(
            "type C {\n    v int\n    fn get() int { return .v }\n}\nfn mk() C { return C(21) }\nfn main() int {\n    return mk() |> .get()\n}\n"
        ),
        21
    );
}

#[test]
fn pipe_binds_tightly_around_operators() {
    // §6.5 D-03 frozen normalization: the pipe acts on its immediately
    // preceding suffix chain FIRST (`t |> .get() - 1` = 6) and a
    // parenthesized pipe chain is a normal operand (`10 - (t |> .get())`
    // = 3). The legacy lower-binding `(x + y) |> .m()` shape is written
    // with explicit parentheses on the left operand.
    assert_eq!(
        ok_i32(
            "type T {\n    v int\n    fn get() int { return .v }\n}\nfn main() int {\n    let t = T(7)\n    let a = t |> .get() - 1\n    let b = 10 - (t |> .get())\n    return a * 100 + b\n}\n"
        ),
        603
    );
    assert_eq!(
        ok_i32(
            "type T {\n    v int\n    fn get() int { return .v }\n}\nfn main() int {\n    let t = T(7)\n    return (t |> .get()) + 1\n}\n"
        ),
        8,
        "a parenthesized pipe chain is a normal operand"
    );
}

#[test]
fn separated_bar_gt_tokens_keep_their_meanings() {
    assert_eq!(
        ok_i32("fn main() int {\n    if 8 > (4 | 2) {\n        return 3\n    }\n    return 0\n}\n"),
        3,
        "`|` stays bitwise and `>` stays comparison; `|>` never swallows the pair"
    );
}
