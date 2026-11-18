//! PLAN-980 view-if 刀：组件/包面 view 级条件语句（`if <expr> { … } [else …]`）
//! ——文法（age-view-if/if-then/else；else-if 链）+ 投影（`ComponentViewItem::
//! Cond` 结构承载：cond 表达式+双子树；文本叶平铺进 texts）+ 负控。
//! 语义消费：模板腿 v-if（auto-ui ace_package_sfc）；native/desktop 渲染腿
//! 装载点具名拒（view_contains_cond）——本面不判真值。
#![allow(dead_code)]

use ace_core::source::age_component::project_component_candidates;
use ace_core::source::age_component::{view_contains_cond, ComponentViewItem};

fn project(src: &str) -> ace_core::source::age_component::ComponentNode {
    let nodes = project_component_candidates(src).expect("project ok");
    nodes.into_iter().next().expect("one node")
}

/// 四形全解析+投影为 Cond（结构判据：then/else 子树形状）。
#[test]
fn p980_view_if_shapes_project_as_cond() {
    // bare if
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { if .n == 1 { text \"A\" } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    match &n.view.tree[0] {
        ComponentViewItem::Element(e) => match &e.items[0] {
            ComponentViewItem::Cond(c) => {
                assert_eq!(c.then_items.len(), 1, "then has the text leaf");
                assert!(c.else_items.is_empty(), "bare if: no else");
            }
            other => panic!("expected Cond, got {other:?}"),
        },
        other => panic!("expected element, got {other:?}"),
    }
    // if-else
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { if .n == 1 { text \"A\" } else { text \"B\" text \"C\" } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    let tree = &n.view.tree[0];
    let cond = view_contains_cond(&[tree.clone()]).expect("cond present");
    let _ = cond;
    if let ComponentViewItem::Element(e) = tree {
        if let ComponentViewItem::Cond(c) = &e.items[0] {
            assert_eq!(c.then_items.len(), 1);
            assert_eq!(c.else_items.len(), 2, "else carries both texts");
        }
    }
    // else-if 链：else_items = 单一嵌套 Cond。
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { if .n == 1 { text \"A\" } else if .n == 2 { text \"B\" } else { text \"C\" } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    if let ComponentViewItem::Element(e) = &n.view.tree[0] {
        if let ComponentViewItem::Cond(c) = &e.items[0] {
            assert_eq!(c.else_items.len(), 1, "chain collapses to one nested Cond");
            assert!(
                matches!(c.else_items[0], ComponentViewItem::Cond(_)),
                "nested arm is a Cond"
            );
        }
    }
    // 嵌套：then 内再嵌 if。
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { if .n == 1 { row { if .n == 2 { text \"X\" } } } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    assert!(view_contains_cond(&n.view.tree).is_some());
}

/// 条件内文本叶平铺进 view.texts（绑定验证通道同覆盖）。
#[test]
fn p980_view_if_texts_flatten_into_leaf_list() {
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col {\n            text \"always\"\n            if .n == 1 { text \"then-leaf\" } else { text \"else-leaf\" }\n        }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    let debug_all = format!("{:?}", n.view.texts);
    for needle in ["always", "then-leaf", "else-leaf"] {
        assert!(
            debug_all.contains(needle),
            "text `{needle}` flattens into texts"
        );
    }
}

/// 负控：`if` 缺体块 / 裸 `else` / cond 非表达式位——具名拒（不静默）。
#[test]
fn p980_view_if_malformed_named_rejects() {
    // 缺体块：`if .n {` 后无 lbrace——parse 错。
    let r = project_component_candidates(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { if .n text \"A\" }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    assert!(r.is_err(), "if without block must be rejected");
    // 裸 else（无前导 if）。
    let r = project_component_candidates(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { else { text \"B\" } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    assert!(r.is_err(), "bare else must be rejected");
}

/// view_contains_cond：跨元素深处的条件项可判（渲染腿 fail-closed 判据）。
#[test]
fn p980_view_contains_cond_finds_nested() {
    let n = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { row { col { if .n == 1 { text \"A\" } } } }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    assert!(view_contains_cond(&n.view.tree).is_some());
    let clean = project(
        "widget W {\n    msg { R }\n    model { var n int = 0 }\n    view {\n        col { text \"A\" }\n    }\n    on { .R -> { .n = 0 } }\n}\n",
    );
    assert!(view_contains_cond(&clean.view.tree).is_none());
}
