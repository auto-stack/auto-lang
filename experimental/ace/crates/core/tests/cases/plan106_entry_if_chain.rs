//! PLAN-106 T-02（014 真语料）：条件条目值 **else-if 链** 的永久回归。
//!
//! 红相（刀前）：`style: if A { … } else if B { … } else { … }` 在
//! age-component/age-package 两面都整源回滚——`source.age.structure` /
//! `source.syntax: expected fn, found widget`（014 app.at 的 hero 渐变有
//! 6 分支 else-if 链 ×2 布局 + AQI 徽章，共 29 处 else-if）。刀后：
//! `ComponentEntryValue::If` 增 `else_chain: Vec<(cond, value)>`，链臂
//! 按源序承载；末 `else` 仍落 `else_value`。
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentViewItem, ComponentViewEntry,
};
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_core::source::age_h0::LegacyWidgetAlias;

fn project_entry(src: &str) -> ComponentViewEntry {
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .unwrap_or_else(|ds| panic!("source must project: {ds:?}"));
    let mut found = None;
    fn walk<'a>(
        items: &'a [ComponentViewItem],
        found: &mut Option<&'a ComponentViewEntry>,
    ) {
        for it in items {
            match it {
                ComponentViewItem::Entry(en) => {
                    if en.key == "style" {
                        *found = Some(en);
                        return;
                    }
                }
                ComponentViewItem::Element(e) => walk(&e.items, found),
                ComponentViewItem::For(f) => walk(&f.items, found),
                ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, found);
                    walk(&c.else_items, found);
                }
                ComponentViewItem::Text(_) => {}
            }
        }
    }
    for n in &nodes {
        walk(&n.view.tree, &mut found);
        if found.is_some() {
            break;
        }
    }
    found.expect("style entry present").clone()
}

fn package_compiles(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    compile_age_package_with_scenario(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps,
        &ace_core::source::age_component::ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .map(|_| ())
}

const CHAIN: &str = "\nwidget App {\n    msg { T }\n    model {\n        var condition str = \"sunny\"\n    }\n    view {\n        col {\n            style: if .condition == \"sunny\" {\n                \"w-64 bg-amber-400\"\n            } else if .condition == \"rain\" {\n                \"w-64 bg-blue-400\"\n            } else if .condition == \"snow\" {\n                \"w-64 bg-cyan-300\"\n            } else {\n                \"w-64 bg-sky-400\"\n            }\n        }\n    }\n    on {\n        .T -> { .condition = \"rain\" }\n    }\n}\n";

/// ①组件面：else-if 链投影为 `else_chain` 按源序承载，末 else 落 `else_value`。
#[test]
fn p106_entry_value_else_if_chain_projects() {
    let en = project_entry(CHAIN);
    match &en.value {
        ComponentEntryValue::If {
            cond,
            then_value,
            else_chain,
            else_value,
        } => {
            use ace_core::source::ast::ExprKind;
            assert!(
                matches!(cond.kind, ExprKind::Binary { .. }),
                "head cond is a comparison"
            );
            assert!(
                matches!(then_value.kind, ExprKind::StrLit(ref s) if s.contains("amber")),
                "head then is the first branch literal"
            );
            assert_eq!(else_chain.len(), 2, "two else-if arms in source order");
            for (c, v) in else_chain {
                assert!(matches!(c.kind, ExprKind::Binary { .. }), "chain cond");
                assert!(matches!(v.kind, ExprKind::StrLit(_)), "chain value");
            }
            assert!(
                matches!(else_value.kind, ExprKind::StrLit(ref s) if s.contains("sky")),
                "final else is the last literal"
            );
        }
        other => panic!("style entry must project as If, got {other:?}"),
    }
}

/// ②组件面：else-if 链整源编译通过（此前整源回滚）。
#[test]
fn p106_entry_value_else_if_chain_component_compiles() {
    let caps = vec!["ac.runtime.text.v1".to_string()];
    ace_core::source::age_component::compile_component_scenario_states(
        CHAIN,
        "p106-chain.at",
        LegacyWidgetAlias::Accept,
        &caps,
        &ace_core::source::age_component::ComponentScenario {
            instances: 1,
            ops: vec![ace_core::source::age_component::ScenarioOp::Send {
                instance: 0,
                msg: "T".to_string(),
            }],
        },
        &ace_core::source::age_component::ComponentInitials {
            per_instance: vec![Vec::new()],
        },
    )
    .expect("else-if chain compiles on the component face");
}

/// ③包面：else-if 链整源编译通过（此前回滚到 `expected fn, found widget`）。
#[test]
fn p106_entry_value_else_if_chain_package_compiles() {
    package_compiles(CHAIN).expect("else-if chain compiles on the package face");
}

/// ④正反边界：单 else 保持原语义（else_chain 为空）；未知拼写仍具名拒。
#[test]
fn p106_single_else_keeps_empty_chain_and_bad_source_still_rejects() {
    let single = "\nwidget App {\n    model { var condition str = \"sunny\" }\n    view {\n        col {\n            style: if .condition == \"sunny\" { \"a\" } else { \"b\" }\n        }\n    }\n}\n";
    let en = project_entry(single);
    match &en.value {
        ComponentEntryValue::If { else_chain, .. } => {
            assert!(else_chain.is_empty(), "single-else keeps an empty chain");
        }
        other => panic!("single else must still project as If, got {other:?}"),
    }
    // 组件面坏源（裸 else-if 后无值）仍整源拒——不静默吞。
    let bad = "\nwidget App {\n    view {\n        col {\n            style: if .a { \"x\" } else if { \"y\" }\n        }\n    }\n}\n";
    let err = ace_core::source::age_component::project_component_candidates(bad)
        .err()
        .expect("bad else-if must reject");
    assert!(
        err.iter().any(|d| d.code == "source.age.structure"),
        "bad chain rejects with a structure diagnostic: {err:?}"
    );
}

/// ⑤PLAN-106 第二刀（014 真语料）：handler 表达式面的**对象字面量**——
/// `.hourly = [ { time: "现在", … } ]` 文本记录列表重写**可解析**（刀前
/// `source.age.structure: unexpected token in a bare element tag`）。
/// 语义侧（无类型列表字段的可写 state/对象类型解析）是共享 typed-list
/// 写面（§4 具名，105 已交 `[]T` 写、114 在途），不在本刀闭合。
#[test]
fn p106_handler_object_literal_list_write_parses() {
    let src = "\nwidget App {\n    msg { M }\n    model {\n        var hourly = [\n            { time: \"现在\", icon: \"⛅\", temp: \"23°\", now: true },\n        ]\n    }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            .hourly = [\n                { time: \"现在\", icon: \"🌧️\", temp: \"21°\", now: true },\n            ]\n        }\n    }\n}\n";
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .expect("handler object-literal list write must parse on the component face");
    assert!(!nodes.is_empty(), "node projected");
}
