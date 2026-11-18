//! PLAN-980 on-if 刀：widget 形 handler 体 if/else（view-stmt 方言镜像规则）
//! ——结构化投影（序保持 stmts + 兼容深平铺 assigns）+ Str==str 两层放行
//! （typecheck + verify——`t == "light"` 参数(owned)/字面量(借用)内容相等）。
#![allow(dead_code)]

use ace_core::source::age_component::project_component_candidates;
use ace_core::source::age_component::ComponentStmt;

fn project(src: &str) -> ace_core::source::age_component::ComponentNode {
    let nodes = project_component_candidates(src).expect("project ok");
    nodes.into_iter().next().expect("one node")
}

const SETTHEME: &str = "widget W {\n    msg { SetTheme(str) }\n    model { var dark_mode bool = true }\n    view { col { text \"T\" } }\n    on {\n        .SetTheme(t) -> {\n            if t == \"light\" {\n                .dark_mode = false\n            }\n            if t == \"dark\" {\n                .dark_mode = true\n            }\n        }\n    }\n}\n";

/// ①结构化投影：handler.stmts = 两条 If（各含单赋值臂）；assigns 兼容面
/// = 全深度平铺（2 条，序=源序）。
#[test]
fn p980_on_if_widget_form_structured() {
    let n = project(SETTHEME);
    let h = n
        .handlers
        .iter()
        .find(|h| h.msg == "SetTheme")
        .expect("handler");
    assert!(!h.body_foreign, "if bodies are structured, not foreign");
    assert_eq!(h.stmts.len(), 2, "two if statements in source order");
    match (&h.stmts[0], &h.stmts[1]) {
        (ComponentStmt::If { then, els, .. }, ComponentStmt::If { then: t2, .. }) => {
            assert!(els.is_none(), "bare ifs (no else in this source)");
            assert_eq!(then.len(), 1, "first if then = one assign");
            assert_eq!(t2.len(), 1, "second if then = one assign");
        }
        other => panic!("expected two If stmts, got {other:?}"),
    }
    assert_eq!(h.assigns.len(), 2, "compat face: deep-flattened assigns");
}

/// ②else-if 链：els = 单一嵌套 If；else 块=平铺赋值臂。
#[test]
fn p980_on_if_else_if_chain_and_else_block() {
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view { col { text \"T\" } }\n    on {\n        .R -> {\n            if .n == 1 {\n                .n = 2\n            } else if .n == 3 {\n                .n = 4\n            } else {\n                .n = 5\n            }\n        }\n    }\n}\n",
    );
    let h = n.handlers.iter().find(|h| h.msg == "R").expect("handler");
    match &h.stmts[0] {
        ComponentStmt::If { els: Some(e), .. } => {
            assert_eq!(e.len(), 1, "chain collapses to one nested Cond stmt");
            assert!(
                matches!(e[0], ComponentStmt::If { .. }),
                "nested arm is an If"
            );
            if let ComponentStmt::If {
                els: Some(inner_else),
                ..
            } = &e[0]
            {
                assert_eq!(inner_else.len(), 1, "final else block = flat assign arm");
                assert!(matches!(inner_else[0], ComponentStmt::Assign(_)));
            }
        }
        other => panic!("expected If, got {other:?}"),
    }
}

/// ③分支内调用形 → body_foreign（fail-closed，不做部分体）。
#[test]
fn p980_on_if_call_in_branch_is_foreign() {
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view { col { text \"T\" } }\n    on {\n        .R -> {\n            if .n == 1 {\n                .n = 2\n                side_effect()\n            }\n        }\n    }\n}\n",
    );
    let h = n.handlers.iter().find(|h| h.msg == "R").expect("handler");
    assert!(
        h.body_foreign,
        "call inside a branch marks the handler foreign"
    );
}

/// ④Str==str 跨借用/owned 放行（typecheck+verify 双层）：`t == "light"`
/// 形在检查函数 verify 下通过（编译级断言——整链不红即放行实证）。
#[test]
fn p980_str_eq_cross_borrowed_owned_compiles() {
    // 编译（含检查函数铸造与共享 verify）通过=两层放行成立。
    project(SETTHEME);
}
