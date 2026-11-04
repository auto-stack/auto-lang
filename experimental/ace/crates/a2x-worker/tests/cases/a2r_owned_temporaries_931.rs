//! PLAN-931 owned view-consumed temporaries must survive until their REAL
//! consumer (D-925-2 Rust-side closure): probe/V-case file (T-02 seeds the
//! reds; T-06 freezes the full set). True chain: source → checked HIR →
//! view → Rust lowering → REAL rustc → real run → parent-side oracle with
//! the runtime event ledger (`A2X_RT_EVENTS=1`) and the frozen fault hook.

use ace_a2x_core::SemanticView;
use ace_a2x_rust::lower_program;
use ace_core::source::{compile_parts_with, SourceProfile};
use ace_core::text_resources::seed_param_exclusions;
use ace_core::verify::verify_text_function;

use crate::rt_common::{assert_value, compile_case, run_chain, Prof, RUNTIME_TEXT_CAP};

/// V-01 first case: the byte-exact 925 Wrap source (a2c_rt_restore::SRC) —
/// two owned literals as view args of `mk`; independent oracle span=5
/// ("ab" + "cde" hand-audited, C925 peer only).
const WRAP: &str = "type Wrap {\n    head Str\n    tail Str\n    static fn mk(h Str, t Str) Wrap { return Wrap { head: h.clone(), tail: t.clone() } }\n    fn span() int { return .head.len() + .tail.len() }\n}\nfn main() int {\n    let w Wrap = Wrap.mk(\"ab\", \"cde\")\n    return w.span()\n}";

/// P1 value red: head/tail must be the full original bytes.
#[test]
fn p931_wrap_span_value() {
    let (out, _err) = run_chain("P931_WRAP", WRAP, Prof::TextRecord, "probe", &[], &[]);
    assert_value("P931_WRAP", &out, 5);
}

/// P1 event red: the first literal's free must come AFTER the `mk` call
/// consumed it. Current full-drain shape frees it at the second literal's
/// pin — the ledger dump (eprintln) captures the actual order.
#[test]
fn p931_wrap_event_order() {
    let (out, err) = run_chain(
        "P931_WRAP",
        WRAP,
        Prof::TextRecord,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    eprintln!("--- P931_WRAP events ---\n{err}");
    assert_value("P931_WRAP", &out, 5);
}

/// P2 statement-root red: a pinned view operand (the clone temp) must be
/// freed after the consuming `let n = ..` statement, not before it.
#[test]
fn p931_stmt_root_len_of_clone() {
    let src = r#"fn main() int {
    let a Str = "ab"
    let n = a.clone().len()
    return n
}"#;
    let (out, _err) = run_chain("P931_LEN", src, Prof::Text, "probe", &[], &[]);
    assert_value("P931_LEN", &out, 2);
}

/// Short-circuit isolation: an untaken RHS side allocates and frees
/// NOTHING (its pins live inside the side block); the value oracle is
/// hand-audited (false side → else branch).
#[test]
fn p931_short_circuit_untaken_side_zero_alloc() {
    let src = r#"fn main() int {
    let b Str = "cde"
    let ok bool = false && b.clone() == b
    if ok { return 1 } else { return 7 }
}"#;
    let (out, err) = run_chain(
        "P931_SC_UNTAKEN",
        src,
        Prof::Text,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_SC_UNTAKEN", &out, 7);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    // exactly b's own lifecycle: the untaken side's clone temp never runs
    assert_eq!(
        events,
        vec!["rt:alloc:3", "rt:free:3"],
        "untaken side must not allocate/free beyond b's own lifecycle: {events:?}"
    );
}

/// Outer pending across a short-circuit: the LHS text-compare's pinned
/// operand temp survives INTO and past the RHS side block (the side frees
/// only its own temps inside itself), then frees after its real consumer
/// (the `let ok` statement). Hand-audited ledger.
#[test]
fn p931_short_circuit_outer_pending_survives() {
    let src = r#"fn main() int {
    let a Str = "ab"
    let b Str = "cde"
    let ok bool = a.clone() == a && b.clone() == b
    if ok { return 5 } else { return 0 }
}"#;
    let (out, err) = run_chain(
        "P931_SC_OUTER",
        src,
        Prof::Text,
        "probe-events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("P931_SC_OUTER", &out, 5);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "ab" (owned local)
            "rt:alloc:3", // b = "cde" (owned local)
            "rt:alloc:2", // t0 = a.clone() (LHS compare operand)
            "rt:alloc:3", // side: t2 = b.clone() (taken side)
            "rt:free:3",  // side frees ITS OWN temp inside the block
            "rt:free:2",  // t0 frees after its real consumer (`let ok`)
            "rt:free:3",  // exit: b
            "rt:free:2",  // exit: a
        ],
        "outer pending must survive the side; the side frees only its own"
    );
}

/// Failure atomicity (frozen trap semantics): faulting the SECOND literal
/// allocation (t1) in the Wrap main traps Resource with NO free of the
/// already-allocated t0 — frees never run before the failed allocation
/// (no partial cleanup, no early release).
#[test]
fn p931_fault_second_arg_no_free() {
    let (out, err) = run_chain(
        "P931_WRAP",
        WRAP,
        Prof::TextRecord,
        "probe-fault2",
        &[],
        &[("A2X_RT_FAULT_ALLOC", "2"), ("A2X_RT_EVENTS", "1")],
    );
    assert_eq!(out, "trap:Resource\n");
    let frees: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:free")).collect();
    assert!(
        frees.is_empty(),
        "a failed allocation must trigger no frees: {err}"
    );
}

/// External fragment modes (V-08 seed): a fragment body calling
/// move/mut-param helpers renders byte-identically to the whole-module
/// emission — the VERIFIED external modes (MoveOwn/MutBorrow) propagate,
/// never a default View; the value oracle is hand-audited (2*10+2).
#[test]
fn p931_external_modes_fragment_equivalence() {
    let src = "fn own_len(move s Str) int { return s.len() }\nfn mut_peek(mut s Str) int { return s.len() }\nfn main() int {\n    let a Str = \"ab\"\n    var b Str = \"xy\"\n    let n1 int = own_len(a.move)\n    let n2 int = mut_peek(b)\n    return n1 * 10 + n2\n}";
    // and the whole chain runs to the hand-audited oracle (move args move;
    // mut args borrow exclusively; view args pass content)
    let (out, _err) = run_chain("P931_MODES", src, Prof::TextRecord, "probe", &[], &[]);
    assert_value("P931_MODES", &out, 22);

    // whole-module reference
    let whole = compile_case("P931_MODES", src, Prof::TextRecord);
    let whole_view = SemanticView::from_checked(&whole.checked).expect("whole view");
    let whole_prog = lower_program(&whole_view, "main").expect("whole lower");
    let whole_text = ace_a2x_rust::print_program(&whole_prog);

    // single-body credential for main: take/peek become signature-only
    // externals whose VERIFIED modes the lowering must consume
    let parts = compile_parts_with(
        src,
        "P931_MODES.at",
        &[RUNTIME_TEXT_CAP.to_string()],
        SourceProfile::TextRecord,
    )
    .expect("parts");
    let main_def = parts.bundle.modules[0]
        .defs
        .iter()
        .position(|d| d.id_text == "d_main")
        .expect("main def");
    let excl = seed_param_exclusions(&parts.bundle);
    let cred = verify_text_function(&parts.bundle, main_def, &excl)
        .unwrap_or_else(|ds| panic!("P931_MODES credential: {ds:?}"));
    let frag_view = SemanticView::from_parts(&parts, &cred).expect("fragment view");
    let frag_prog = lower_program(&frag_view, "main").expect("fragment lower");
    let frag_text = ace_a2x_rust::print_program(&frag_prog);
    // compare ONLY main's function bytes: the fragment is single-body by
    // design (the whole emission also carries own_len/mut_peek bodies)
    let fn_bytes = |text: &str| -> String {
        let start = text.find("pub fn fn_main").expect("fn_main rendered");
        let end = text[start..].find("}\n").map(|i| start + i + 2).expect("fn end");
        text[start..end].to_string()
    };
    assert_eq!(
        fn_bytes(&whole_text),
        fn_bytes(&frag_text),
        "fragment must render fn_main identically: verified external modes propagate"
    );
}
