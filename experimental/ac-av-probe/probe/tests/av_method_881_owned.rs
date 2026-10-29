//! PLAN-881 V-03/V-10 owned-resource tier (`av_method_881_owned`): the
//! 782 flagship and Token microprogram through the SHARED owned lowering
//! with the ARENA EVENT LEDGER (alloc/free streams, LIFO, live-0), the
//! method-shaped borrow-lifetime rules, and the allocation-failure
//! injection (StrArena::set_fault_at) proving partial-output rollback.
//!
//! Oracles frozen by hand-audit: p01 = 30; Token = 57 (one 7-byte LIT,
//! one 7-byte CLONE, each freed exactly once, normal-exit live 0);
//! 'A中' = 4 bytes / 2 scalars.

use auto_av_core::abc::encode_abc;
use auto_av_core::abc_verify::verify_abc_bytes;
use auto_av_core::av::{execute_with_arena, CancelFlag, ExecLimits};
use auto_av_core::contract::{ExecOutcome, Value};
use auto_av_core::mir_verify::verify_mir;
use auto_av_core::owned_text::StrArena;

const P01_782: &str = include_str!("../../../ac-core/fixtures/text-record-source/valid/p01-combination.at");

const TOKEN_V12: &str = r#"type Token {
    text Str
    kind int
    line int
    at int
    len int

    fn is_ident() bool {
        return .kind == 1
    }

    static fn ident(move text Str, line int, at int) Token {
        let n int = text.len()
        return Token { text: text.move, kind: 1, line: line, at: at, len: n }
    }
}

fn weigh(move t Token) int { return t.len + t.line + t.at }

fn main() int {
    let s Str = "keyword"
    let a Token = Token.ident(s.move, 3, 12)
    if a.is_ident() == false { return 100 }
    if a.len != 7 { return 101 }
    var b Token = a.clone()
    b.at = 40
    if a.at != 12 { return 102 }
    let w int = weigh(b.move)
    return w + a.text.len()
}
"#;

/// The shared owned pipeline with an instrumented arena.
fn run_owned_ledger(src: &str) -> (ExecOutcome, StrArena) {
    let compiled = auto_ac_prototype::source::compile_text_record_source(src, "o.at", &[])
        .expect("text-record source compiles");
    let mir =
        auto_mir_lower::lower_checked_owned(&compiled.checked, "d_main", 0x881_0003).expect("lower");
    verify_mir(&mir).expect("verify");
    let wire = encode_abc(&mir).expect("encode");
    let loaded = verify_abc_bytes(&wire.to_flat(), &mir.owner).expect("admit");
    let arena = StrArena::new();
    let (out, _) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena.clone(),
    );
    (out, arena)
}

fn owned_front_err(src: &str) -> String {
    auto_ac_prototype::source::compile_text_record_source(src, "e.at", &[])
        .err()
        .and_then(|ds| ds.first().map(|d| d.code.clone()))
        .unwrap_or_else(|| "<compiled>".to_string())
}

// ---------------------------------------------------------------------------
// V-03: the 782 flagship and the Token microprogram
// ---------------------------------------------------------------------------

#[test]
fn v03_782_flagship_value_thirty() {
    let (out, _) = run_owned_ledger(P01_782);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(30), .. }),
        "782 flagship = 30 through the shared owned lowering, got {out:?}"
    );
}

#[test]
fn v03_token_microprogram_fifty_seven() {
    let (out, arena) = run_owned_ledger(TOKEN_V12);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(57), .. }),
        "weigh(b)=50 + a.text.len()=7, got {out:?}"
    );
    // Resource oracle: 'keyword' is ONE 7-byte literal + ONE 7-byte clone;
    // each freed exactly once; normal exit leaves live 0.
    let allocs = arena.alloc_events();
    let lit7: Vec<_> = allocs.iter().filter(|a| a.bytes == 7).collect();
    assert_eq!(lit7.len(), 2, "one LIT + one CLONE of 7B: {allocs:?}");
    let frees = arena.free_events();
    let free7: Vec<_> = frees.iter().filter(|f| f.bytes == 7).collect();
    assert_eq!(free7.len(), 2, "each 7B buffer freed exactly once: {frees:?}");
    assert_eq!(arena.live(), 0, "normal exit: no leaks");
}

// ---------------------------------------------------------------------------
// V-10: borrow lifetimes through METHOD shapes
// ---------------------------------------------------------------------------

/// A view method returning a BORROWED view of a live field: zero alloc,
/// zero clone — the caller keeps ownership and frees once.
#[test]
fn v10_method_field_borrow_return_zero_alloc() {
    let src = r#"type T {
    label Str
    n int
    static fn make() T { return T { label: "A中", n: 1 } }
    fn label_view() str { return .label }
}
fn main() int {
    var t = T.make()
    let v str = t.label_view()
    let k = v.len()
    if k != 2 { return 10 + k }
    if t.label.len_bytes() != 4 { return 20 }
    return k * 10 + t.n
}
"#;
    let (out, arena) = run_owned_ledger(src);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(21), .. }),
        "2*10+1 = 21, got {out:?}"
    );
    // The 4-byte 'A中' buffer: allocated once at construction, freed once
    // by its owner at scope exit; the borrow itself allocated NOTHING.
    let allocs = arena.alloc_events();
    assert_eq!(allocs.len(), 1, "no hidden clone/alloc for the borrow");
    assert_eq!(allocs[0].bytes, 4);
    assert_eq!(arena.live(), 0);
}

/// Returning a field borrow that would OUTLIVE its owner is rejected at
/// the front door (`verify.borrow-escape`): a local's/owned temp's field
/// cannot escape.
#[test]
fn v10_borrow_escape_from_local_rejected() {
    let src = r#"type T {
    label Str
    n int
    fn leak() str {
        let fresh = T { label: "x", n: 0 }
        return fresh.label
    }
    static fn make() T { return T { label: "A中", n: 1 } }
}
fn main() int {
    var t = T.make()
    let v str = t.leak()
    return v.len()
}
"#;
    assert_eq!(owned_front_err(src), "verify.borrow-escape");
}

/// A temporary receiver lives through the call and its legal view
/// consumption (the chained-call shape from 771/782).
#[test]
fn v10_temporary_receiver_consumed_in_call() {
    let src = r#"type T {
    label Str
    n int
    static fn make() T { return T { label: "A中", n: 1 } }
    fn score() int { return .label.len() * 10 + .n }
}
fn main() int {
    let r = T.make().score()
    return r
}
"#;
    let (out, arena) = run_owned_ledger(src);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(21), .. }),
        "2*10+1, got {out:?}"
    );
    assert_eq!(arena.live(), 0, "temporary receiver freed exactly once");
}

// ---------------------------------------------------------------------------
// V-10: second-allocation-failure rollback (partial construction)
// ---------------------------------------------------------------------------

/// A static method building a TWO-Str record: fault at the SECOND
/// allocation — only already-produced new fields roll back; the OLD
/// target value (here: the caller's earlier record) is untouched, and
/// the failure is a controlled resource outcome, never a torn state.
#[test]
fn v10_second_alloc_failure_rolls_back_partially_built_output() {
    let src = r#"type Two {
    a Str
    b Str
    n int
    static fn build(move x Str, move y Str, k int) Two { return Two { a: x.move, b: y.move, n: k } }
    static fn seed() Two { return Two { a: "one", b: "two", n: 0 } }
}
fn main() int {
    var old = Two.seed()
    let x Str = "A中"
    let y Str = "Z"
    let built = Two.build(x.move, y.move, 9)
    return built.a.len() * 100 + built.b.len() * 10 + built.n + old.n
}
"#;
    // Control (no fault): 2*100 + 1*10 + 9 + 0 = 219.
    let (out, _) = run_owned_ledger(src);
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(219), .. }),
        "control run, got {out:?}"
    );

    // Fault at the SECOND allocation inside build: the first moved-in
    // field's transfer is rolled back — the CALLEE produces no value, the
    // caller's moved roots are NOT resurrected as live leaks (the arena
    // reports the partial buffer freed), and the outcome is a Resource
    // trap (allocation failure classification), never Success.
    let compiled = auto_ac_prototype::source::compile_text_record_source(src, "f.at", &[]).unwrap();
    let mir = auto_mir_lower::lower_checked_owned(&compiled.checked, "d_main", 7).unwrap();
    verify_mir(&mir).unwrap();
    let wire = encode_abc(&mir).unwrap();
    let loaded = verify_abc_bytes(&wire.to_flat(), &mir.owner).unwrap();
    let arena = StrArena::new();
    arena.set_fault_at(4); // seed's two + build's first new alloc => next faults
    let (out, trace) = execute_with_arena(
        &loaded,
        &ExecLimits::default(),
        &CancelFlag::new(false),
        arena.clone(),
    );
    // The injected allocator fault is the frozen Infra family naming its
    // attempt (the 821 resource contract) — never a trap, never success.
    match &out {
        ExecOutcome::Infra(msg) => assert!(
            msg.contains("fault-injected at owned allocation attempt #4"),
            "the fault must name its attempt: {msg}"
        ),
        other => panic!("injected fault must be the infra family, got {other:?}"),
    }
    // The partial output rolled back: no leaked buffer from the failed
    // construction (every successful alloc has a matching free).
    let arena = trace.arena.as_ref().unwrap();
    let allocs = arena.alloc_events();
    let frees = arena.free_events();
    let alloc_total: usize = allocs.iter().map(|a| a.bytes).sum();
    let free_total: usize = frees.iter().map(|f| f.bytes).sum();
    assert_eq!(
        alloc_total, free_total,
        "failed construction rolls its partial output back (allocs {allocs:?} frees {frees:?})"
    );
}
