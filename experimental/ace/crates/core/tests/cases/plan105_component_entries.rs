//! PLAN-105 T-02（U03 W1）永久回归：组件面 **entry 值**的解析边界。
//!
//! 判据（真语料 013/016/022 的最小复现）：元素体里的 `checked: .a == 0 && .c > 0`
//! 这类比较/逻辑绑定必须解析成普通表达式；而 `onclick: .Msg(args)` 仍是事件绑定
//! （投影为裸 `.Msg` Field、实参作宿主通道丢弃），裸 `.Submit` 走表达式面由
//! `is_event + msg 名单` 识别。修复前 component 面把裸 `.a` 贪吃成事件绑定，
//! 把 `== 0` 留给父规则 → 整 view 回滚到 item 级（`source.age.structure`）。
use ace_core::source::age_component::{
    ComponentEntryValue, ComponentScenario, ComponentViewEntry, ComponentViewItem,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn entry(src: &str, key: &str) -> ComponentViewEntry {
    let nodes = ace_core::source::age_component::project_component_candidates(src)
        .unwrap_or_else(|ds| panic!("source must project: {ds:?}"));
    let mut found = Vec::new();
    fn walk(items: &[ComponentViewItem], key: &str, found: &mut Vec<ComponentViewEntry>) {
        for it in items {
            match it {
                ComponentViewItem::Entry(en) => {
                    if en.key == key {
                        found.push(en.clone());
                    }
                }
                ComponentViewItem::Element(e) => walk(&e.items, key, found),
                ComponentViewItem::For(f) => walk(&f.items, key, found),
                ComponentViewItem::Cond(c) => {
                    walk(&c.then_items, key, found);
                    walk(&c.else_items, key, found);
                }
                ComponentViewItem::Text(_) => {}
            }
        }
    }
    for n in &nodes {
        walk(&n.view.tree, key, &mut found);
    }
    found.remove(0)
}

fn compiles(src: &str) -> bool {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    compile_age_package_with_scenario(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "app".into(),
            source: src.into(),
            provider: None,
        }],
        "app::app",
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
    )
    .is_ok()
}

/// 013 真语料形：比较 + 逻辑的 `checked` 绑定是表达式，不是事件绑定。
#[test]
fn p105_component_entry_comparison_binding_projects_as_expr() {
    let src = "widget App {\n\
        msg { T }\n\
        model {\n\
            var a int = 0\n\
            var c int = 1\n\
        }\n\
        view {\n\
            col {\n\
                checkbox {\n\
                    checked: .a == 0 && .c > 0\n\
                    onclick: .T\n\
                }\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "checked");
    assert!(
        matches!(en.value, ComponentEntryValue::Expr(_)),
        "comparison binding must project as Expr, got {:?}",
        en.value
    );
    assert!(!en.is_event, "`checked` is not an event key");
    assert!(compiles(src), "package face must compile the same source");
}

/// `onclick: .Msg(arg)` 仍是事件绑定：投影为裸 `.Msg` Field（实参作宿主通道丢弃）。
#[test]
fn p105_component_entry_msg_with_args_is_event_binding() {
    let src = "widget App {\n\
        msg { SetMode(str) }\n\
        model { var m str = \"\" }\n\
        view {\n\
            button {\n\
                onclick: .SetMode(\"basic\")\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "onclick");
    assert!(en.is_event, "`onclick` must be an event key");
    match &en.value {
        ComponentEntryValue::Expr(expr) => {
            assert!(
                matches!(
                    expr.kind,
                    ace_core::source::ast::ExprKind::Field { .. }
                ),
                "event binding projects to a bare field"
            );
        }
        other => panic!("expected bare-field event binding, got {other:?}"),
    }
    assert!(compiles(src));
}

/// 裸 `.Submit`（无括号）走表达式面；投影层据事件键 + msg 名单仍识别为事件。
#[test]
fn p105_component_entry_bare_msg_stays_field() {
    let src = "widget App {\n\
        msg { Submit }\n\
        model { var s str = \"\" }\n\
        view {\n\
            button {\n\
                onclick: .Submit\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "onclick");
    assert!(en.is_event);
    assert!(
        matches!(en.value, ComponentEntryValue::Expr(_)),
        "bare `.Submit` stays a field expression"
    );
    assert!(compiles(src));
}

/// 条件条目值（1072 刀B）与上面的收窄共存：`style: if …` 仍投影为 If。
#[test]
fn p105_component_entry_conditional_style_still_projects() {
    let src = "widget App {\n\
        msg { T }\n\
        model { var d bool = false }\n\
        view {\n\
            button {\n\
                onclick: .T\n\
                style: if .d { \"a\" } else { \"b\" }\n\
            }\n\
        }\n\
    }\n";
    let en = entry(src, "style");
    assert!(
        matches!(en.value, ComponentEntryValue::If { .. }),
        "conditional style keeps its structural carrier"
    );
    assert!(compiles(src));
}
