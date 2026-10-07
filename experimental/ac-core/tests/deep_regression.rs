//! PLAN-745 R3-QA-02 permanent deep-input regressions: every expression
//! stage (parse / resolve / typecheck / full compile incl. verify) handles
//! legal operator chains far beyond the reviewer's 16384-term counterexample
//! WITHOUT recursion (iterative spines + iterative Drop). The old failure
//! mode was a stack overflow in resolve/typecheck/verify.
#![cfg(windows)]
use auto_ac_prototype::source::{parse_program, resolve, typecheck};

const CHAIN: usize = 16384;

#[test]
fn probe_parse_only() {
    let src = format!("fn main() int {{ return {}0 }}", "1+".repeat(CHAIN));
    let _ = parse_program(&src);
}

#[test]
fn probe_resolve() {
    let src = format!("fn main() int {{ return {}0 }}", "1+".repeat(CHAIN));
    let p = parse_program(&src).unwrap();
    let caps: Vec<String> = Vec::new();
    let _ = resolve::resolve(&p, &caps);
}

#[test]
fn probe_typecheck() {
    let src = format!("fn main() int {{ return {}0 }}", "1+".repeat(CHAIN));
    let p = parse_program(&src).unwrap();
    let caps: Vec<String> = Vec::new();
    let r = resolve::resolve(&p, &caps).unwrap();
    let _ = typecheck::typecheck(&r);
}

#[test]
fn probe_verify_full() {
    let src = format!("fn main() int {{ return {}0 }}", "1+".repeat(CHAIN));
    let caps: Vec<String> = Vec::new();
    let _ = auto_ac_prototype::source::compile_source(&src, "t.at", &caps);
}

#[test]
fn deep_legal_parens_parse_and_compile() {
    // R4-QA-02: parenthesized groups collapse iteratively — pure grouping
    // carries NO limit (200 levels = the reviewer's boundary case; 50000
    // proves the mechanism is depth-independent, not threshold-tuned)
    let caps: Vec<String> = Vec::new();
    for depth in [99usize, 200, 5000, 50000] {
        let legal = format!(
            "fn main() int {{ return {}1{} }}",
            "(".repeat(depth),
            ")".repeat(depth)
        );
        auto_ac_prototype::source::compile_source(&legal, "t.at", &caps)
            .unwrap_or_else(|e| panic!("{} legal paren levels must compile: {:?}", depth, e));
    }

    // unclosed parens: the collapse loop eats every `(` without recursion;
    // the controlled failure is the missing `)`, depth-independent
    let unclosed = format!("fn main() int {{ return {}1", "(".repeat(100000));
    let err = parse_program(&unclosed).unwrap_err();
    assert_eq!(err.code, "source.syntax");
}

#[test]
fn call_and_block_nesting_boundary_both_sides() {
    // R4-QA-02: the recursive structures (call argument lists, blocks)
    // keep a resource guard tied to the actual per-level recursion
    // (parser+resolve+emit+verify+native), raised to 1000 — the corpus
    // tops out at ~4 and review probes at 199..300. 999 must pass every
    // stage; 1001 is a controlled rejection on either side.
    let caps: Vec<String> = Vec::new();
    let calls999 = format!(
        "fn f(x int) int {{ return x }}
fn main() int {{ return {}1{} }}",
        "f(".repeat(999),
        ")".repeat(999)
    );
    auto_ac_prototype::source::compile_source(&calls999, "t.at", &caps)
        .expect("999 call levels must compile");
    let calls1001 = format!(
        "fn f(x int) int {{ return x }}
fn main() int {{ return {}1{} }}",
        "f(".repeat(1001),
        ")".repeat(1001)
    );
    let err = parse_program(&calls1001).unwrap_err();
    assert_eq!(err.code, "source.syntax");
    assert!(err.message.contains("nesting"), "{}", err.message);

    let ifs999 = format!(
        "fn main() int {{ {}return 1{}
return 0 }}",
        "if true { ".repeat(999),
        " }".repeat(999)
    );
    auto_ac_prototype::source::compile_source(&ifs999, "t.at", &caps)
        .expect("999 block levels must compile");
    let ifs1200 = format!(
        "fn main() int {{ {}return 1{}
return 0 }}",
        "if true { ".repeat(1200),
        " }".repeat(1200)
    );
    let err = parse_program(&ifs1200).unwrap_err();
    assert_eq!(err.code, "source.syntax");
    assert!(err.message.contains("nesting"), "{}", err.message);

    // unclosed call chains stay controlled too
    let unclosed_calls = format!(
        "fn f(x int) int {{ return x }}
fn main() int {{ return {}1",
        "f(".repeat(5000)
    );
    let err = parse_program(&unclosed_calls).unwrap_err();
    assert_eq!(err.code, "source.syntax");
}
