//! V-03：生成期受控拒绝（定位错误、无半成品、合法兄弟可装载）。

use ac_syntax_grammar::compile;

fn rejects(_name: &str, mut_src: &str, expect_contains: &str) {
    let diags = compile(mut_src).unwrap_err();
    assert!(
        diags.iter().any(|d| d.message.contains(expect_contains)),
        "expected `{expect_contains}` in {diags:?}"
    );
}

#[test]
fn grammar_reject_left_recursion_direct() {
    rejects(
        "left-rec",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: " \t\r\n" } a { literal: "a" } plus { literal: "+" } }
            rules {
                program { list { expr() {} } }
                expr { alt { expr() {} plus() {} expr() {} } alt { a() {} } }
            }
        }"#,
        "left recursion",
    );
}

#[test]
fn grammar_reject_left_recursion_via_nullable_prefix() {
    rejects(
        "left-rec-null",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: " \t\r\n" } a { literal: "a" } }
            rules {
                program { list { expr() {} } }
                expr { alt { pre() {} expr() {} } alt { a() {} } }
                pre { a(repeat: "opt") }
            }
        }"#,
        "left recursion",
    );
}

#[test]
fn grammar_reject_nullable_repeat() {
    rejects(
        "nullable-rep",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: " \t\r\n" } a { literal: "a" } }
            rules {
                program { list { expr() {} } }
                expr { pre() {} many { pre() {} } a() {} }
                pre { a(repeat: "opt") }
            }
        }"#,
        "nullable item",
    );
}

#[test]
fn grammar_reject_unknown_references() {
    rejects(
        "unknown-ref",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: " \t\r\n" } a { literal: "a" } }
            rules { program { list { missing-rule() {} } } }
        }"#,
        "neither lit/tok/ref/pratt nor a known rule/token",
    );
    rejects(
        "unknown-tok",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: " \t\r\n" } a { literal: "a" } }
            rules { program { many { missing-tok() {} } } }
        }"#,
        "neither lit/tok/ref/pratt nor a known rule/token",
    );
}

#[test]
fn grammar_reject_nonprogressing_scanner_rule() {
    rejects(
        "no-progress",
        r#"grammar(name: "t", revision: 1) {
            tokens { ws(trivia: true) { charset: "" } a { literal: "a" } }
            rules { program { a() {} } }
        }"#,
        "matches nothing",
    );
}

#[test]
fn grammar_reject_bad_atom_and_missing_sections() {
    let e = compile("this is not an atom grammar").unwrap_err();
    assert!(e.iter().any(|d| d.message.contains("parse error")));
    let e = compile(r#"grammar(name: "t", revision: 1) { tokens { } }"#).unwrap_err();
    assert!(e.iter().any(|d| d.message.contains("no tokens")));
    let e = compile(r#"grammar(name: "t", revision: 1) { nonsense { } tokens { x { literal: "x" } } rules { program { x() {} } } }"#).unwrap_err();
    assert!(e.iter().any(|d| d.message.contains("unknown section")));
}

#[test]
fn grammar_reject_reports_but_sibling_grammar_still_loads() {
    let _ = compile(r#"grammar(name: "bad", revision: 1) { rules { program { missing } } }"#);
    let good = include_str!("../../grammars/interface-mini-v1.at");
    compile(good).expect("sibling grammar loads fine after a rejection");
}
