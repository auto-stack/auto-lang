//! PLAN-981（B-LANGUAGE-V）：**980/A 正式源**经 981 的 AGE 包面编译。
//!
//! 源来自 980/A 交付 `IF-980-NODE-DESKTOP-A`（auto-lang `7f431bd5f`／auto-ui `403c936d`）：
//!
//! | fixture | 原路径（auto-ui） | SHA256 前 16 |
//! |---|---|---|
//! | `ace_hello_minimal.at` | `examples/ui/ace-hello-minimal/src/front/app.at` | `7f9f79a7714a3fb5` |
//! | `ace_node_counter.at` | `examples/ui/ace-node-counter/src/front/app.at` | `6016eb863f94499e` |
//!
//! 判据是"真实 A 源能过 981 的 SOURCE/LANGUAGE 前门并给出事实"，不是自造样本：
//! - Hello：单 view/文本叶；
//! - Counter：`msg { Inc, Dec, Reset }`、`var count int = 0`、三条 `on` 复合赋值、
//!   view 里的 `onclick: .Dec` 等**事件绑定**（绑定是事实，不是 `self` 字段引用）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const HELLO: &str = include_str!("../fixtures_ma/ace_hello_minimal.at");
const COUNTER: &str = include_str!("../fixtures_ma/ace_node_counter.at");

fn compile(src: &str) -> ace_core::source::age_package::AgePackageDescription {
    compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: src.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("a delivered 980/A source compiles through the AGE package face")
    .description
}

/// 980/A 的 Hello 源：单节点 + 单文本叶。
#[test]
fn p981_ma_hello_source_compiles() {
    let d = compile(HELLO);
    assert_eq!(d.components.len(), 1);
    let c = &d.components[0];
    assert_eq!(c.name, "Hello");
    assert_eq!(c.exits, vec![ace_core::source::age_package::AgeViewExit::View]);
    assert_eq!(c.texts.len(), 1);
    assert_eq!(c.texts[0].fn_name.is_empty(), false, "the text leaf has its hidden fn");
}

/// 980/A 的 Counter 源：消息、模型字段、三条 handler、view 事件绑定全部可读。
#[test]
fn p981_ma_counter_source_compiles_with_its_facts() {
    let d = compile(COUNTER);
    let c = &d.components[0];
    assert_eq!(c.name, "Counter");
    assert_eq!(
        c.messages.iter().map(|m| m.name.as_str()).collect::<Vec<_>>(),
        vec!["Inc", "Dec", "Reset"]
    );
    assert!(c.messages.iter().all(|m| m.payload.is_empty()), "unit messages");
    assert_eq!(c.message_set_name, None, "the delivered source uses the anonymous form");
    assert_eq!(c.fields, vec![("count".to_string(), "int".to_string())]);
    assert_eq!(
        c.handler_msgs,
        vec!["Inc".to_string(), "Dec".to_string(), "Reset".to_string()]
    );
    // view 属性里的 `.Msg` 是事件绑定（不是 self 字段引用，也不是值表达式）。
    let bindings: Vec<(&str, &str)> = c
        .view_events
        .iter()
        .map(|(k, m, _)| (k.as_str(), m.as_str()))
        .collect();
    assert_eq!(
        bindings,
        vec![("onclick", "Dec"), ("onclick", "Reset"), ("onclick", "Inc")]
    );
}

/// 负控：把事件绑定写成未知消息仍具名拒（绑定不能绕过消息声明检查）。
#[test]
fn p981_ma_unknown_event_binding_is_rejected() {
    let broken = COUNTER.replace("onclick: .Inc", "onclick: .Boom");
    let r = compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: broken,
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    );
    assert!(r.is_err(), "an unknown `.Boom` binding must not compile silently");
}