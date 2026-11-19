//! PLAN-980 lambda-if 刀（core 投影面）：view 内联 lambda 体 if/else
//! （view-stmt 方言——on-if 刀文法臂共用，本刀零文法改动）——投影结构化
//! （`Lambda.stmts` 序保持 If/Assign + `assigns` 深平铺兼容面；if 形不再
//! foreign）。负控：调用形仍 foreign（#6p fail-closed 不铸合同）。
//! 铸造/HIR/发射链由 av 侧（`__age_vl_` 铸造可判别）与 UI lane（if/else
//! 整语句发射文本）用例承载。
#![allow(dead_code)]

use ace_core::source::age_component::project_component_candidates;
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentNode, ComponentStmt, ComponentViewItem,
};

fn project(src: &str) -> ComponentNode {
    let nodes = project_component_candidates(src).expect("project ok");
    nodes.into_iter().next().expect("one node")
}

/// 树内第一条内联 lambda 的三面（stmts/assigns/foreign）。
fn first_lambda(
    n: &ComponentNode,
) -> (
    &Vec<ComponentStmt>,
    &Vec<ace_core::source::age_component::ComponentAssign>,
    bool,
) {
    fn walk(
        items: &[ComponentViewItem],
    ) -> Option<(
        &Vec<ComponentStmt>,
        &Vec<ace_core::source::age_component::ComponentAssign>,
        bool,
    )> {
        for it in items {
            match it {
                ComponentViewItem::Element(e) => {
                    if let Some(v) = walk(&e.items) {
                        return Some(v);
                    }
                }
                ComponentViewItem::Cond(c) => {
                    if let Some(v) = walk(&c.then_items) {
                        return Some(v);
                    }
                    if let Some(v) = walk(&c.else_items) {
                        return Some(v);
                    }
                }
                ComponentViewItem::Entry(en) => {
                    if let ComponentEntryValue::Lambda {
                        stmts,
                        assigns,
                        foreign,
                    } = &en.value
                    {
                        return Some((stmts, assigns, *foreign));
                    }
                }
                // PLAN-1072 刀B：循环子树照走（lambda 收集面）。
                ComponentViewItem::For(f) => {
                    if let Some(v) = walk(&f.items) {
                        return Some(v);
                    }
                }
                ComponentViewItem::Text(_) => {}
            }
        }
        None
    }
    walk(&n.view.tree).expect("a lambda entry in the view tree")
}

/// 赋值 RHS 的整型字面量（本用例族的 RHS 全为 int 字面量）。
fn rhs_int(a: &ace_core::source::age_component::ComponentAssign) -> i32 {
    match &a.rhs.kind {
        ace_core::source::ast::ExprKind::IntLit(v) => *v,
        other => panic!("expected int literal rhs, got {other:?}"),
    }
}

/// ① if-else 形结构化投影：stmts=[If{then:[Assign], els:Some([Assign])}]；
/// assigns 深平铺=2（then 先于 else，源序）；不再 foreign。
#[test]
fn p980_lambda_if_structured_projection() {
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => { if .x == 1 { .x = 2 } else { .x = 3 } } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (stmts, assigns, foreign) = first_lambda(&n);
    assert!(!foreign, "if form is structured, not foreign");
    assert_eq!(stmts.len(), 1, "one statement (the if)");
    match &stmts[0] {
        ComponentStmt::If { then, els, .. } => {
            assert_eq!(then.len(), 1, "then = one assign");
            assert_eq!(
                els.as_deref().map(|e| e.len()),
                Some(1),
                "else = one assign"
            );
            assert!(matches!(then[0], ComponentStmt::Assign(_)));
            assert!(matches!(
                els.as_deref().unwrap()[0],
                ComponentStmt::Assign(_)
            ));
        }
        other => panic!("expected If, got {other:?}"),
    }
    assert_eq!(assigns.len(), 2, "compat face: deep-flattened assigns");
}

/// ② 序保持：赋值→if→赋值 混合体 stmts=[Assign, If, Assign]；assigns 深平
/// 铺序=源序（then 先于 else）。
#[test]
fn p980_lambda_if_order_preserved() {
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => {\n            .x = 1\n            if .x == 1 {\n                .x = 2\n            } else {\n                .x = 3\n            }\n            .x = 4\n        } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (stmts, assigns, foreign) = first_lambda(&n);
    assert!(!foreign);
    assert_eq!(stmts.len(), 3, "assign, if, assign in source order");
    assert!(matches!(stmts[0], ComponentStmt::Assign(_)));
    assert!(matches!(stmts[1], ComponentStmt::If { .. }));
    assert!(matches!(stmts[2], ComponentStmt::Assign(_)));
    assert_eq!(assigns.len(), 4, "1, 2(then), 3(else), 4");
    assert_eq!(
        assigns.iter().map(|a| rhs_int(a)).collect::<Vec<_>>(),
        vec![1, 2, 3, 4],
        "deep-flatten order: top, then, else, top"
    );
}

/// ③ bare if（无 else）与 else-if 链：els=None / els=[嵌套 If]。
#[test]
fn p980_lambda_if_bare_and_else_if_chain() {
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => { if .x == 1 { .x = 2 } } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (stmts, _, foreign) = first_lambda(&n);
    assert!(!foreign);
    match &stmts[0] {
        ComponentStmt::If { els, .. } => assert!(els.is_none(), "bare if: no else"),
        other => panic!("expected If, got {other:?}"),
    }
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => { if .x == 1 { .x = 2 } else if .x == 3 { .x = 4 } else { .x = 5 } } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (stmts, assigns, _) = first_lambda(&n);
    match &stmts[0] {
        ComponentStmt::If { els: Some(e), .. } => {
            assert_eq!(e.len(), 1, "chain collapses to one nested If");
            assert!(matches!(e[0], ComponentStmt::If { .. }));
        }
        other => panic!("expected If, got {other:?}"),
    }
    assert_eq!(assigns.len(), 3, "2, 4, 5 across the chain");
}

/// ④ 嵌套 if（then 内再嵌）结构可判。
#[test]
fn p980_lambda_if_nested() {
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => { if .x == 1 { if .x == 2 { .x = 3 } } } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (stmts, assigns, foreign) = first_lambda(&n);
    assert!(!foreign);
    match &stmts[0] {
        ComponentStmt::If { then, .. } => {
            assert_eq!(then.len(), 1);
            assert!(matches!(then[0], ComponentStmt::If { .. }), "nested if");
        }
        other => panic!("expected If, got {other:?}"),
    }
    assert_eq!(assigns.len(), 1, "deep-flatten reaches the nested arm");
}

/// ⑤ 负控：分支内调用形 → foreign（#6p fail-closed 不铸合同——与
/// av 侧"不铸值函数"用例互补：本面判投影，av 面判受检模块缺席）。
#[test]
fn p980_lambda_if_call_in_branch_stays_foreign() {
    let n = project(
        "widget W {\n    msg Msg { R }\n    model { var x int = 0 }\n    view {\n        col { text \"t\" { oninput: () => { if .x == 1 {\n            .x = 2\n            side()\n        } } } }\n    }\n    on { .R -> { .x = 0 } }\n}\n",
    );
    let (_, _, foreign) = first_lambda(&n);
    assert!(foreign, "call inside a branch marks the lambda foreign");
}
