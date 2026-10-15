//! V-12 深度（PLAN-787 T-04）：平链 / 连续和交错嵌套 / 缺括号，各 ≥4096 层——
//! 构建、查询、canonical、Drop 全路径无栈崩（显式栈；arena POD Drop）。
//! 编解码路径随 T-06 分页序列化补充。

mod lossless_;
use lossless_::{parse, profile};
use ac_syntax_rt::program::Budget;

const DEPTH: usize = 4096;

#[test]
fn depth_flat_chain_deep_expr() {
    // 连续和：1 + 1 + ... （左结合链 = 深左脊）。
    let p = profile("auto-core");
    let mut expr = String::from("1");
    for _ in 0..DEPTH {
        expr.push_str(" + 1");
    }
    let text = format!("fn f() int {{ return {expr} }}");
    let (out, snap) = parse(&p, &text);
    out.tree.assert_lossless(&snap).unwrap();
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    let c = out.tree.canonical(); // 迭代查询全树
    assert!(c.lines().count() > DEPTH);
}

#[test]
fn depth_nested_parens_interleaved() {
    // 交错嵌套：((((1)))+(((1))))*… 构建查询释放全路径。
    let p = profile("auto-core");
    let mut expr = String::new();
    for i in 0..DEPTH {
        if i % 2 == 0 {
            expr.push_str(&format!("{}(", if i == 0 { "" } else { " + " }));
        } else {
            expr.push_str("1)");
        }
    }
    if DEPTH % 2 == 0 {
        expr.push_str(" * 2");
    }
    let text = format!("fn f() int {{ return {expr} }}");
    let (out, snap) = parse(&p, &text);
    out.tree.assert_lossless(&snap).unwrap();
    assert!(out.tree.diags.is_empty(), "{:?}", out.tree.diags);
    let _ = out.tree.canonical();
    drop(out.tree); // arena Drop 非递归
}

#[test]
fn depth_missing_parens_partial_tree_no_crash() {
    // 缺括号：((((1 —— 未闭合深嵌套 → 恢复/失败路径同样无栈崩、无损。
    let p = profile("auto-core");
    let text = format!("fn f() int {{ return {}1 }}", "(".repeat(DEPTH + 8));
    let (out, snap) = parse(&p, &text);
    out.tree.assert_lossless(&snap).unwrap();
}

#[test]
fn depth_resource_exhausted_is_controlled_not_crash() {
    // 巨深嵌套在预算内受控 ResourceExhausted（非栈崩、非静默截断）。
    let p = profile("auto-core");
    use ac_syntax_rt::lexer::CompiledScanner;
    use ac_syntax_rt::parse::parse_snapshot;
    use ac_syntax_rt::snapshot::{PageStore, Snapshot};
    let cs = CompiledScanner::compile(&p).unwrap();
    let text = format!("fn f() int {{ return {}1{} }}", "(".repeat(1 << 21), ")".repeat(1 << 21));
    let snap = Snapshot::from_text(&std::rc::Rc::new(PageStore::new()), &text);
    let r = parse_snapshot(&p, &cs, &snap, &Budget { max_parse_depth: 4096, work_bytes: 64 << 20 });
    assert!(
        matches!(r, Err(ac_syntax_rt::program::RtError::ResourceExhausted { .. })),
        "deep nesting must hit controlled ResourceExhausted, got {r:?}"
    );
}
