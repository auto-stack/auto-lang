//! PLAN-1081 named-function-value / closure observations through the
//! ORDINARY Auto source profile (`compile_language_source`) — the same
//! front door the `ac-av/language-source-run-v1` recipe runs. Oracles are
//! independent typed values, byte-exact console streams and frozen
//! diagnostic codes/stages, never a value read back out of the
//! implementation.
//!
//! These are the value-level counterparts of the canonical corpus cases
//! (`language/functions/references-closures.case.ad` owns the byte-exact
//! dual-stream oracles); this file pins the semantics a mutation here
//! would silently break: named function items as checked values (let /
//! argument / return / container slots), recursion safety, nested fn
//! items with the Rust-style no-capture rule, and the closure surface on
//! the Auto face.

use ace_av::console_host::ConsoleCapture;
use ace_av_core::abc::AbcBytes;
use ace_av_core::av::{execute_sched, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::{abc_verify, mir_verify};

/// Compile/execute one Auto-profile program end to end (schema/CST →
/// inference completion → bound → checked HIR → MIR → AVAD → AV), the
/// same chain as the language recipe, with console capture.
/// Compile and admit one Auto-profile program (the recipe chain up to the
/// executable module).
fn compile_and_load(src: &str) -> ace_av_core::abc_verify::LoadedModule {
    let compiled = ace_core::source::compile_language_source(src, "p1081.at", &[])
        .unwrap_or_else(|d| panic!("Auto-profile compile failed: {d:?}"));
    let mir = ace_lowering::lower_checked_auto_language(&compiled.checked, "d_main", 0x1081)
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

fn ok_i32(src: &str) -> i32 {
    match run(src).0 {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("expected an i32 success, got {other:?}"),
    }
}

fn ok_stdout(src: &str) -> String {
    let (out, capture) = run(src);
    match out {
        ExecOutcome::Success { .. } => String::from_utf8(capture.stdout).expect("stdout is UTF-8"),
        other => panic!("expected a success, got {other:?}"),
    }
}

fn diag(src: &str) -> (String, String) {
    match ace_core::source::compile_language_source(src, "p1081.at", &[]) {
        Ok(_) => panic!("expected a compile diagnostic, got Ok"),
        Err(ds) => {
            let d = ds.first().expect("at least one diagnostic");
            (format!("{:?}", d.stage).to_lowercase(), d.code.clone())
        }
    }
}

// ---- named function items as values (PLAN-1081 T-02) -----------------------

#[test]
fn fn_value_let_and_call() {
    assert_eq!(
        ok_i32(
            "fn twice(x int) int { return x * 2 }\nfn main() int { let f = twice; return f(21) }\n"
        ),
        42,
        "`let f = double` binds the fn item; `f(21)` calls through the value"
    );
}

#[test]
fn fn_value_as_argument() {
    assert_eq!(
        ok_i32(
            "fn twice(x int) int { return x * 2 }\nfn apply(f fn(int)int, x int) int { return f(x) }\nfn main() int { return apply(twice, 21) }\n"
        ),
        42,
        "a named fn item passes as a `fn(int)int` argument and is called inside the callee"
    );
}

#[test]
fn fn_value_returned_from_fn() {
    assert_eq!(
        ok_i32(
            "fn twice(x int) int { return x * 2 }\nfn maker() fn(int)int { return twice }\nfn main() int { let f = maker(); return f(21) }\n"
        ),
        42,
        "a fn value crosses a return slot (FN-RETURN lane)"
    );
}

#[test]
fn fn_value_recursive_and_mutual() {
    assert_eq!(
        ok_i32(
            "fn fact(n int) int {
 if n <= 1 {
  return 1
 }
 return n * fact(n - 1)
}\nfn is_even(n int) bool {
 if n == 0 {
  return true
 }
 return is_odd(n - 1)
}\nfn is_odd(n int) bool {
 if n == 0 {
  return false
 }
 return is_even(n - 1)
}\nfn main() int {
 let g = fact
 let e = is_even
 if e(6) {
  return g(5)
 }
 return 0
}\n"
        ),
        120,
        "fn values of recursive and mutually recursive items carry no captures and call the real bodies"
    );
}

#[test]
fn fn_value_not_callable_rejects() {
    let (stage, code) = diag("fn main() int { let f = 42; return f(1) }\n");
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.not-callable"),
        "calling a non-function local keeps the frozen not-callable family"
    );
}

#[test]
fn fn_value_signature_mismatch_rejects() {
    let (stage, code) = diag(
        "fn twice(x int) int { return x * 2 }\nfn apply(f fn(int,int)int, x int) int { return f(x, x) }\nfn main() int { return apply(twice, 21) }\n",
    );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("verify", "source.type-mismatch"),
        "a fn value whose signature differs from the formal type is a named mismatch"
    );
}

#[test]
fn fn_value_multi_site_references() {
    assert_eq!(
        ok_i32(
            "fn twice(x int) int { return x * 2 }\nfn main() int { let f = twice; let g = twice; return f(10) + g(11) }\n"
        ),
        42,
        "multiple value references of the same fn item are independent zero-env closures (no first-case-site limit)"
    );
}

#[test]
fn fn_value_entry_rejects() {
    let (stage, code) = diag("fn main() int { let f = main; return f(1) }\n");
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.fn-as-value"),
        "the entry function is the module's root frame and is never a value"
    );
}

#[test]
fn fn_value_container_slot() {
    assert_eq!(
        ok_i32(
            "fn twice(x int) int { return x * 2 }\nfn main() int {\n let fs = [twice.clone(), twice.clone()]\n let f = fs.remove(0)\n return f(21)\n}\n"
        ),
        42,
        "a fn value crosses a container slot (cloned into a List, moved out via remove) and calls after read-back"
    );
}

// ---- nested fn items (PLAN-1081 d01/d01b, Rust no-capture rule) ------------

#[test]
fn nested_fn_item_calls() {
    assert_eq!(
        ok_stdout("fn main() {\n    fn inner() {\n        print(\"hi from inner\")\n    }\n    inner()\n    print(\"after\")\n}\n"),
        "hi from inner\nafter\n",
        "legacy d01: a nested void fn item runs when called (its body is hoisted, not silently dropped)"
    );
}

#[test]
fn nested_fn_capture_rejects() {
    let (stage, code) = diag(
        "fn main() {\n    let base = 10\n    fn grab(n int) int { return base + n }\n    print(grab(5))\n}\n",
    );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.nested-fn-no-capture"),
        "legacy d01b: a nested fn item cannot capture outer locals (Rust nested-fn semantics); named rejection, never a silent 0"
    );
}

// ---- closures on the Auto face (PLAN-1081 T-03) -----------------------------

#[test]
fn closure_typed_let_and_call() {
    assert_eq!(
        ok_i32("fn main() int { let sub = (a int, b int) => a - b; return sub(12, 5) }\n"),
        7,
        "legacy 008: a typed closure binds in a local let and calls positionally"
    );
}

#[test]
fn closure_block_body_captures() {
    assert_eq!(
        ok_i32(
            "fn main() int {\n let base = 19\n let f = (x int) => { let twice = x * 2; return twice + base }\n return f(11) + 1\n}\n"
        ),
        42,
        "§6 closure-block-local: block body, local let inside, outer capture, multiple-free at exit"
    );
}

#[test]
fn closure_multiple_calls_and_capture_update() {
    assert_eq!(
        ok_i32(
            "fn main() int {\n let factor = 3\n let scale = (x int) => x * factor\n return scale(4) + scale(5)\n}\n"
        ),
        27,
        "a capture-closure calls multiple times (12 + 15) against the same env"
    );
}

#[test]
fn closure_passed_as_argument() {
    assert_eq!(
        ok_i32(
            "fn apply(f fn(int)int, x int) int { return f(x) }\nfn main() int { let inc = (x int) => x + 1; return apply(inc, 41) }\n"
        ),
        42,
        "a closure value passes as a `fn(int)int` argument and is called inside the callee"
    );
}

#[test]
fn closure_returned_from_fn() {
    assert_eq!(
        ok_i32(
            "fn maker(k int) fn(int)int { return (x int) => x * k }\nfn main() int { let f = maker(6); return f(7) }\n"
        ),
        42,
        "a closure crosses a return slot with its captured env intact (FN-RETURN lane)"
    );
}

// ---- FN-RETURN / CLOSURE-LIFETIME extras (PLAN-1081 T-04) -------------------

#[test]
fn nested_fn_same_name_different_bodies() {
    // Two owners each declare a SAME-NAME nested item with different
    // bodies — the owner-mangled identities keep the executed bodies
    // apart (the single-module face of the FN-RETURN identity rule).
    assert_eq!(
        ok_i32(
            "fn left() int {
 fn pick() int {
  return 3
 }
 return pick() * 10
}
fn right() int {
 fn pick() int {
  return 4
 }
 return pick()
}
fn main() int { return left() + right() }
"
        ),
        34,
        "left.pick()=3 and right.pick()=4 keep separate mangled bodies (30 + 4)"
    );
}

#[test]
fn fn_body_edit_switches_executed_body() {
    // The edit/restore half of FN-RETURN on the ordinary compile face:
    // changing a fn body and recompiling switches the EXECUTED body both
    // ways (never a stale cached body).
    let base = "fn maker(k int) int { return k * 2 }
fn main() int { return maker(21) }
";
    assert_eq!(ok_i32(base), 42, "original body");
    let edited = "fn maker(k int) int { return k + 1 }
fn main() int { return maker(41) }
";
    assert_eq!(
        ok_i32(edited),
        42,
        "edited body (different inputs, same oracle)"
    );
    assert_eq!(ok_i32(base), 42, "restored body re-executes the original");
    assert_ne!(
        ok_i32(
            "fn maker(k int) int { return k * 2 }
fn main() int { return maker(20) }
"
        ),
        42
    );
}

#[test]
fn closure_borrow_capture_direct_call_is_legal() {
    assert_eq!(
        ok_i32(
            "fn main() int {
 let s Str = \"abcdef\"
 let size = () => { return s.len() }
 return size()
}
"
        ),
        6,
        "a borrow-capturing closure CALLED DIRECTLY is the legal lane (read-through)"
    );
}

#[test]
fn closure_borrow_escape_rejects() {
    let (stage, code) = diag(
        "fn apply(f fn()int) int { return f() }
fn main() int {
 let s Str = \"abcdef\"
 let size = () => { return s.len() }
 return apply(size)
}
",
    );
    assert_eq!(
        (stage.as_str(), code.as_str()),
        ("bind", "source.closure-borrow-escape"),
        "passing a borrow-capturing closure as an argument is the frozen escape rejection"
    );
}

#[test]
fn closure_owned_capture_released_live_zero() {
    // CLOSURE-LIFETIME: two closure instances with owned captures live
    // across calls and releases; after a clean exit the arena ledger is
    // EMPTY (no leaked env, no double free).
    let src = "fn main() int {
 let base = 20
 let f = (x int) => { return x + base }
 let a = f(1)
 let g = (x int) => { return x * 2 }
 let b = g(10)
 return a + b + f(1)
}
";
    let loaded = compile_and_load(src);
    let (out, trace) =
        ace_av_core::av::execute(&loaded, &ExecLimits::default(), &CancelFlag::new(false));
    match out {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => assert_eq!(
            v, 62,
            "f(1)=21, g(10)=20, f(1)=21 — multiple instances, shared base env"
        ),
        other => panic!("expected an i32 success, got {other:?}"),
    }
    let arena = trace.arena.clone().expect("ledger attached");
    assert_eq!(arena.live(), 0, "every closure env released at exit");
}
