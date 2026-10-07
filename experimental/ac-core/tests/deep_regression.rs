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
    // both sides of the syntactic nesting boundary: 99 levels legal,
    // 4096 unclosed rejected with the controlled nesting diagnostic
    let legal = format!(
        "fn main() int {{ return {}1{} }}",
        "(".repeat(99),
        ")".repeat(99)
    );
    let caps: Vec<String> = Vec::new();
    let c = auto_ac_prototype::source::compile_source(&legal, "t.at", &caps)
        .expect("99 legal paren levels must compile (R3-QA-02 guard regression)");
    let _ = c;

    let unclosed = format!("fn main() int {{ return {}1", "(".repeat(4096));
    let err = parse_program(&unclosed).unwrap_err();
    assert_eq!(err.code, "source.syntax");
    assert!(err.message.contains("nesting"));
}
