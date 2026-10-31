//! PLAN-902 T-04 full-surface conformance (V-02..V-05): mut text params
//! (exclusive views, verifier-proven exclusivity), resource records (owned
//! Str fields, field-level buffer cleanup, deep clone, whole-root moves),
//! the Unicode/NUL char/slice edge matrix, borrowed-return provenance, and
//! branch/loop cleanup schedules — values by real rustc, resource orders
//! by the default-off event ledger, refusals by named diagnostics.

use crate::rt_common::{assert_value, run_chain, Prof};

/// V-02/V-03: mut text params read and forward (exclusivity is
/// source-verified; the same root twice is a verifier refusal below).
#[test]
fn t04_mut_text_read_and_forward() {
    let src = "fn g(mut s Str) int { return s.len_bytes() }\n\
               fn f(mut s Str) int { return g(s) + s.len() }\n\
               fn main() int {\n    var a Str = \"ab\"\n    return f(a) * 10 + a.len()\n}";
    // f = 2 + 2 = 4; a still readable after the exclusive window → 4*10+2 = 42
    let (out, _err) = run_chain("MUT_TEXT", src, Prof::TextRecord, "debug", &[], &[]);
    assert_value("MUT_TEXT", &out, 42);
}

/// V-03: two `mut` text args of one root / view+mut overlap are NAMED
/// verifier refusals (774 D-04 exclusivity at the call sites).
#[test]
fn t04_mut_text_same_root_rejected() {
    for (name, src) in [
        ("two-mut", "fn g(mut a Str, mut b Str) int { return a.len() + b.len() }\nfn main() int {\n    var x Str = \"ab\"\n    return g(x, x)\n}"),
        ("view-mut", "fn g(mut a Str, b Str) int { return a.len() + b.len() }\nfn main() int {\n    var x Str = \"ab\"\n    return g(x, x)\n}"),
    ] {
        let err = ace_core::source::compile_text_record_source(
            src,
            "neg.at",
            &["ac.runtime.text.v1".to_string()],
        )
        .expect_err(name);
        assert!(
            err.iter().any(|d| d.code == "verify.mut-conflict"),
            "{name}: expected verify.mut-conflict, got {:?}",
            err.iter().map(|d| &d.code).collect::<Vec<_>>()
        );
    }
}

/// V-02/V-04/V-05: the resource-record flagship — ctor with a moved-in
/// Str, field view read, field REPLACEMENT (old field buffer freed only
/// after the RHS materialized), explicit deep clone (independent owners),
/// and the unified exit schedule freeing each record's Str FIELD.
#[test]
fn t04_resource_record_clone_replace_events() {
    let src = "type Wrap { inner Str\n    n int }\n\
               fn main() int {\n    var a Str = \"ab\"\n    var w Wrap = Wrap(a.move, 3)\n\
               \x20   w.inner = w.inner + \"c\"\n    let c Wrap = w.clone()\n\
               \x20   return c.inner.len() * 10 + w.n\n}";
    let (out, err) = run_chain(
        "RES_REC",
        src,
        Prof::TextRecord,
        "events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("RES_REC", &out, 33);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "ab"
            "rt:alloc:3", // w.inner + "c" (RHS materializes first)
            "rt:free:2",  // replacement frees the OLD field buffer
            "rt:alloc:3", // deep clone: one allocation for c.inner
            "rt:free:3",  // exit: c.inner (declared after w)
            "rt:free:3",  // exit: w.inner
        ],
        "field-level cleanup follows the verified unified schedule"
    );
}

/// V-04: the Unicode/NUL edge matrix — scalar counts vs UTF-8 bytes,
/// multi-byte slicing by SCALAR index, char_at boundaries, embedded NUL.
#[test]
fn t04_unicode_nul_matrix() {
    // "A中😀": 3 scalars, 1+3+4 = 8 bytes; slice(1,3) = "中😀" len 2;
    // char_at(2) = 😀 (U+1F600 = 128512)
    let src = "fn main() int {\n    let s Str = \"A中😀\"\n\
               \x20   let part str = s.slice(1, 3)\n\
               \x20   let c char = s.char_at(2)\n\
               \x20   return s.len() * 1000 + s.len_bytes() * 10 + part.len() + c.to_codepoint() / 100000\n}";
    let (out, _err) = run_chain("UNI_MATRIX", src, Prof::TextRecord, "debug", &[], &[]);
    // 3*1000 + 8*10 + 2 + 1 = 3083
    assert_value("UNI_MATRIX", &out, 3083);
}

/// V-04: e + combining mark is TWO scalars; eq compares full bytes (same
/// nominal type operands; the comparison rides a helper so the condition
/// ends in a call — an identifier directly before `{` parses as a named
/// construction in this grammar, a source-form rule).
#[test]
fn t04_combining_and_eq() {
    let src = "fn eq(a str, b str) bool { return a == b }\n\
               fn main() int {\n    let a Str = \"e\u{0301}\"\n    let av str = a.to(str)\n    let b str = \"é\"\n\
               \x20   if eq(av, b) { return 1 }\n    if a.len() != 2 { return 2 }\n\
               \x20   if a.len_bytes() != 3 { return 3 }\n    if b.len() != 1 { return 4 }\n    return 42\n}";
    let (out, _err) = run_chain("COMBINING", src, Prof::TextRecord, "debug", &[], &[]);
    assert_value("COMBINING", &out, 42);
}

/// V-04: slice/char_at edge traps — typed TextIndex, never a byte panic.
#[test]
fn t04_slice_char_at_traps() {
    for (name, bind, expr, tail) in [
        (
            "neg-idx",
            "let c char = ",
            "s.char_at(0 - 1)",
            "c.to_codepoint()",
        ),
        ("oob", "let c char = ", "s.char_at(3)", "c.to_codepoint()"),
        ("byte-oob", "let v str = ", "s.slice(1, 9)", "v.len()"),
        ("neg-range", "let v str = ", "s.slice(2, 1)", "v.len()"),
    ] {
        let src = format!(
            "fn main() int {{\n    let s Str = \"ab中\"\n    {bind}{expr}\n    return {tail}\n}}"
        );
        let (out, _err) = run_chain(
            &format!("TRAP_{name}"),
            &src,
            Prof::TextRecord,
            "trap",
            &[],
            &[],
        );
        assert_eq!(
            out, "trap:TextIndex\n",
            "{name}: scalar-indexed bounds, typed trap"
        );
    }
    // char_at of a scalar boundary mid-UTF8 is FINE (scalar indexing)
    let ok = "fn main() int {\n    let s Str = \"ab中\"\n    return s.char_at(2).to_codepoint()\n}";
    let (out, _err) = run_chain("CHAR_AT_OK", ok, Prof::TextRecord, "debug", &[], &[]);
    assert_value("CHAR_AT_OK", &out, 20013); // '中'
}

/// V-04: char codepoint roundtrip + invalid scalar trap.
#[test]
fn t04_char_codepoints() {
    let ok = "fn main() int {\n    let c char = char.from_codepoint(20013)\n    return c.to_codepoint()\n}";
    let (out, _err) = run_chain("CP_OK", ok, Prof::Text, "debug", &[], &[]);
    assert_value("CP_OK", &out, 20013); // '中'

    let bad = "fn main() int {\n    let c char = char.from_codepoint(55296)\n    return c.to_codepoint()\n}";
    let (out, _err) = run_chain("CP_BAD", bad, Prof::Text, "trap", &[], &[]);
    assert_eq!(out, "trap:InvalidScalar\n");
}

/// V-02: borrowed-return provenance — a literal-only str result from a
/// borrowing-free signature is `&'static str`; an owned param's `.to(str)`
/// view borrows from the view param.
#[test]
fn t04_borrowed_return_provenance() {
    let src = "fn lit() str { return \"x\" }\n\
               fn pass(s Str) str { return s.to(str) }\n\
               fn main() int {\n    let a Str = \"ab\"\n    let v str = pass(a)\n    return v.len() * 10 + lit().len()\n}";
    let (out, _err) = run_chain("BORROW_RET", src, Prof::TextRecord, "debug", &[], &[]);
    assert_value("BORROW_RET", &out, 21);
}

/// V-02: borrow escape is a verifier refusal (a local/owned source never
/// becomes a borrowed return).
#[test]
fn t04_borrow_escape_rejected() {
    let err = ace_core::source::compile_text_record_source(
        "fn f() str {\n    let a Str = \"ab\"\n    return a.to(str)\n}\nfn main() int { return f().len() }",
        "neg.at",
        &["ac.runtime.text.v1".to_string()],
    )
    .expect_err("borrow escape must reject");
    assert!(
        err.iter().any(|d| d.code == "verify.borrow-escape"),
        "got {:?}",
        err.iter().map(|d| &d.code).collect::<Vec<_>>()
    );
}

/// V-05: branch-scoped and loop-carried owned locals — the block's natural
/// end frees its own lets (per loop iteration), and the break frees the
/// loop body's owned before leaving.
#[test]
fn t04_branch_loop_cleanup_events() {
    let src = "fn main() int {\n    var t int = 0\n    if t < 1 {\n        let b Str = \"bb\"\n        t = b.len()\n    }\n\
               \x20   var i int = 0\n    loop {\n        let s Str = \"xx\"\n        t = t + s.len_bytes()\n        i = i + 1\n        if i > 2 { break }\n    }\n    return t\n}";
    let (out, err) = run_chain(
        "BRANCH_LOOP",
        src,
        Prof::TextRecord,
        "events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("BRANCH_LOOP", &out, 8);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // branch-local b
            "rt:free:2",  // then-block natural end
            "rt:alloc:2", // iteration 1 s
            "rt:free:2",  // loop body natural end
            "rt:alloc:2", // iteration 2
            "rt:free:2",
            "rt:alloc:2", // iteration 3 — exits via break
            "rt:free:2",  // freed AT the break, before leaving the loop
        ],
        "block-exit and break cleanup follow the verified schedule"
    );
}

/// V-02: whole-root move of a resource record + owned record RETURN (the
/// returned value is NOT freed; everything else live is).
#[test]
fn t04_record_move_and_owned_return() {
    let src = "type W { s Str }\n\
               fn rewrap(move w W) W {\n    let n Str = w.s.clone()\n    return W(n.move)\n}\n\
               fn main() int {\n    var a Str = \"ab\"\n    let w W = W(a.move)\n    let r W = rewrap(w.move)\n    return r.s.len()\n}";
    let (out, err) = run_chain(
        "REC_MOVE_RET",
        src,
        Prof::TextRecord,
        "events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("REC_MOVE_RET", &out, 2);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // main: a = "ab"
            "rt:alloc:2", // rewrap clones w.s
            "rt:free:2",  // rewrap exit: its MOVE-IN root w.s (callee owns it)
            "rt:free:2",  // main exit: r.s (the returned clone — NOT freed in rewrap)
        ],
        "moved-in roots die in the CALLEE's schedule; owned returns are not freed"
    );
}
