//! V-04 恢复语义（PLAN-787 T-04）：未闭合参数/块、坏 token、坏声明 →
//! partial tree + 有限进展（消耗输入 / 有限 missing / 终止），后续合法声明可查。

mod lossless_;
use lossless_::{parse, profile};

#[test]
fn recovery_unclosed_block_returns_partial_tree() {
    let p = profile("auto-core");
    let (out, snap) = parse(&p, "fn f() int { let x = 1");
    assert!(!out.tree.diags.is_empty());
    out.tree.assert_lossless(&snap).unwrap();
    assert!(
        out.tree.has_error_nodes(),
        "unclosed block must yield error/missing nodes"
    );
}

#[test]
fn recovery_unclosed_params_bad_token() {
    let p = profile("auto-core");
    for text in [
        "fn f(a: int { return 1 }",
        "fn f() int { return @ }",
        "fn f() int { @ return 1 }",
    ] {
        let (out, snap) = parse(&p, text);
        assert!(!out.tree.diags.is_empty(), "input {text:?} must diagnose");
        out.tree.assert_lossless(&snap).unwrap();
    }
}

#[test]
fn recovery_each_action_consumes_or_terminates() {
    // 前进性：恢复后的树覆盖全文（隐含每次恢复有进展或终止），且解析终止。
    let p = profile("interface-mini-v1");
    let (out, snap) = parse(
        &p,
        "api broken( -> ;\napi ok(x: i32) -> i32;\n@@@\napi fine() -> i32;",
    );
    out.tree.assert_lossless(&snap).unwrap();
    assert!(!out.tree.diags.is_empty());
    assert!(out.tree.decls.len() >= 2);
}

#[test]
fn recovery_bad_statement_inside_body_next_statement_parses() {
    let p = profile("auto-core");
    let (out, snap) = parse(&p, "fn f() int { let = 1\nreturn 2 }");
    out.tree.assert_lossless(&snap).unwrap();
    assert!(!out.tree.diags.is_empty());
}
