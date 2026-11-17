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
use ace_core::source::age_package::{collect_facts, compile_age_package};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const HELLO: &str = include_str!("../fixtures_ma/ace_hello_minimal.at");
const COUNTER: &str = include_str!("../fixtures_ma/ace_node_counter.at");
/// R67-5：fixture 与 auto-ui canonical 源逐字节同源（818 B；SHA256 前 16 `6016eb863f94499e`）。
/// auto-ui 侧测试直接 `include_str!` 读 examples 源，避免字面量漂移；这里用长度钉住。
const COUNTER_LEN: usize = 818;

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
    assert_eq!(
        c.exits,
        vec![ace_core::source::age_package::AgeViewExit::View]
    );
    assert_eq!(c.texts.len(), 1);
    assert_eq!(
        c.texts[0].fn_name.is_empty(),
        false,
        "the text leaf has its hidden fn"
    );
}

/// 980/A 的 Counter 源：消息、模型字段、三条 handler、view 事件绑定全部可读。
#[test]
fn p981_ma_counter_source_compiles_with_its_facts() {
    assert_eq!(
        COUNTER.len(),
        COUNTER_LEN,
        "fixture must stay byte-stable with the canonical A source"
    );
    let d = compile(COUNTER);
    let c = &d.components[0];
    assert_eq!(c.name, "Counter");
    assert_eq!(
        c.messages
            .iter()
            .map(|m| m.name.as_str())
            .collect::<Vec<_>>(),
        vec!["Inc", "Dec", "Reset"]
    );
    assert!(
        c.messages.iter().all(|m| m.payload.is_empty()),
        "unit messages"
    );
    assert_eq!(
        c.message_set_name, None,
        "the delivered source uses the anonymous form"
    );
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
    assert!(
        r.is_err(),
        "an unknown `.Boom` binding must not compile silently"
    );
}

/// B-LANGUAGE-V（V-L-VIEW / V-L-WIRE / V-L-COVER）：真实 A 源在 SOURCE/LANGUAGE
/// 面的 V 判据——出口选择、事件绑定、构造分母、原 span 保真。
#[test]
fn p981_ma_sources_satisfy_the_language_v_checks() {
    use ace_core::source::age_package::{select_exit, AgeViewExit, AgeViewRequest};
    let d = compile(COUNTER);
    let c = &d.components[0];

    // V-L-VIEW：真实源只有默认 view 出口；任意 UI 请求都选它，场景请求无跨域。
    assert_eq!(c.exits, vec![AgeViewExit::View]);
    let ui = AgeViewRequest::Ui {
        width: 400,
        height: 900,
        widget_host: false,
    };
    assert_eq!(select_exit(&c.exits, ui).unwrap(), AgeViewExit::View);
    assert!(select_exit(&c.exits, AgeViewRequest::Scene2d).is_err());

    // V-L-WIRE：三条 unit 消息 + 三条 handler + onclick 绑定，键与消息都对得上。
    let handlers: Vec<&str> = c.handler_msgs.iter().map(|s| s.as_str()).collect();
    assert_eq!(handlers, vec!["Inc", "Dec", "Reset"]);
    for (key, msg, span) in &c.view_events {
        assert_eq!(key, "onclick");
        assert!(
            c.messages.iter().any(|m| &m.name == msg),
            "binding `{msg}` names a declared message"
        );
        let text = &COUNTER[span.start..span.end];
        assert!(
            text.contains(msg),
            "binding span {span:?} must cover `{msg}` in the real source, got {text:?}"
        );
    }

    // V-L-COVER：真实源覆盖 node/model/on/view 四类事实。
    let facts = collect_facts(&d);
    for key in ["node", "model", "on", "view"] {
        assert!(
            facts.contains(&key),
            "real A source must cover `{key}`: {facts:?}"
        );
    }

    // 原名保真：消息变体的原 span 指回真实源文本。
    for v in &c.messages {
        let text = &COUNTER[v.span.start..v.span.end];
        assert!(
            text.starts_with(&v.name),
            "variant span must start at its name: {text:?}"
        );
    }
}
