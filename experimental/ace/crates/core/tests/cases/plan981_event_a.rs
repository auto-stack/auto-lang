//! PLAN-981（IF-981-EVENT-A）：980/A 正式源带来的**事件 A 子面**。
//!
//! 覆盖 985/EVENT-SPLIT-20261116 要求的三个 A 键在 981 侧的最小可交面：
//! - **UI-EVT-MSG**：`msg Msg { … }` 具名段与匿名 `msg { … }` 并存，变体名/原 span/具名段
//!   保真，重复变体仍具名拒；
//! - **UI-EVT-BIND**：handler 参数名/span 被**读取保留**；B-W1（IF-981-PARAMS-B）起
//!   形参数与载荷槽在**声明级**核对一致（unit 变体 + 带参 handler 是域错误）；
//! - **UI-EVT-PAYLOAD**：声明侧允许（`Add(str)`）；B-W1（IF-981-EVENT-PAYLOAD-B）起
//!   `Send` 恒等价空载荷——向载荷变体的空 Send 是 arity 域错误，带值派发走真实执行
//!   （见 `plan981_event_b`）。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(
    src: &str,
) -> Result<
    ace_core::source::age_package::AgePackageDescription,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .map(|out| out.description)
}

/// UI-EVT-MSG：具名 unit 消息 + 匿名消息并存；名字/变体/具名段保真。
#[test]
fn p981_event_a_named_unit_messages_are_projected() {
    let named = "node M {\n    msg Msg {\n        Add,\n        Set\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let d = compile(named).expect("a named unit message set compiles");
    let c = &d.components[0];
    assert_eq!(c.message_set_name.as_deref(), Some("Msg"));
    let names: Vec<&str> = c.messages.iter().map(|m| m.name.as_str()).collect();
    assert_eq!(names, vec!["Add", "Set"]);
    assert!(
        c.messages.iter().all(|m| m.payload.is_empty()),
        "unit variants carry no payload"
    );
    assert_eq!(c.handler_params, vec![("Add".to_string(), Vec::new())]);

    let anon =
        "node M {\n    msg {\n        Inc\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let d = compile(anon).expect("an anonymous message set still compiles");
    assert_eq!(d.components[0].message_set_name, None);
    assert_eq!(d.components[0].messages.len(), 1);
}

/// UI-EVT-BIND（A 语义 → B-W1 升级，IF-981-PARAMS-B）：形参数必须与载荷槽
/// 数一致——unit 变体 + 带参 handler 现在是**声明级**域错误（未派发也拒），
/// 取代 A 期“解析保留、派发时具名拒”。合法形参声明（槽=形参数）仍可读回。
#[test]
fn p981_event_a_param_handler_is_named_rejected() {
    // unit 变体 `Add` + 单参 handler `.Add(t)` → 声明级 arity 域错误。
    let src = "node M {\n    msg Msg {\n        Add\n    }\n    on {\n        .Add(t) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(src) {
        Ok(_) => panic!("a unit variant with a parameterised handler must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(ds[0].message.contains("payload slot"), "{ds:?}");
        }
    }
    // 对照：载荷槽与形参数一致时，参数事实照旧可读（A 的保真面不回退）。
    let ok = "node M {\n    msg Msg {\n        Add(int)\n    }\n    on {\n        .Add(t) -> {\n            .count = count + t\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let d = compile(ok).expect("a payload variant with a matching handler parses");
    assert_eq!(
        d.components[0].handler_params,
        vec![("Add".to_string(), vec!["t".to_string()])]
    );
}

/// R65-1/R65-4：跨 `msg` 块的重复变体、第二个具名集合都具名拒。
#[test]
fn p981_event_a_cross_block_duplicates_are_rejected() {
    let cross = "node M {\n    msg {\n        Inc\n    }\n    msg {\n        Inc\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(cross) {
        Ok(_) => panic!("cross-block duplicate variants must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.msg-duplicate", "{ds:?}"),
    }
    let sets = "node M {\n    msg A {\n        Inc\n    }\n    msg B {\n        Dec\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(sets) {
        Ok(_) => panic!("a second named message set must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.msg-set-duplicate", "{ds:?}"),
    }
}

/// R65-3：widget 形 handler 指向未声明消息 → 具名拒。
#[test]
fn p981_event_a_unknown_handler_message_is_rejected() {
    let src = "node M {\n    on {\n        .Unknown -> {\n            .count = 0\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(src) {
        Ok(_) => panic!("a handler for an undeclared message must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.msg-unknown", "{ds:?}"),
    }
}

/// R65-2/R66-2：同名 unit+payload 变体已被 R65-1 的跨块重复拒拦下；B-W1 起
/// 向载荷变体投递**空载荷 Send** 是 arity 域错误（`Send` 恒等价空载荷，不再
/// 是 A 期的具名 unsupported——真实载荷执行见 plan981_event_b）。
#[test]
fn p981_event_a_unit_first_payload_second_dispatch_is_rejected() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node M {\n    msg {\n        Add\n    }\n    msg Msg {\n        Add(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    // 该源本身因跨块重复变体被拒（R65-1）；构造一个只声明一次、跨块不重复的
    // 对照组：unit `Add` 与 payload `Set`（形参数对齐载荷槽）。
    let ok = "node M {\n    msg {\n        Add\n    }\n    msg Msg {\n        Set(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n        .Set(s) -> {\n            .count = count + 2\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    assert!(
        compile(src).is_err(),
        "cross-block duplicates already rejected"
    );
    let d = compile(ok).expect("distinct variants across blocks stay legal");
    assert_eq!(d.components[0].messages.len(), 2);
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Set".into(),
        }],
    };
    let r = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(ok)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    );
    match r {
        Ok(_) => panic!("an empty-payload Send to a payload variant must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(
                ds[0].message.contains("payload slot"),
                "the rejection names the arity: {}",
                ds[0].message
            );
        }
    }
}

/// R65-11：具名 unit 集合的消息派发仍然工作（同一独立轨迹）。
#[test]
fn p981_event_a_named_unit_dispatch_works() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node M {\n    msg Msg {\n        Add\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Add".into(),
        }],
    };
    let out = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect("a named unit set dispatches");
    assert_eq!(
        out.description.components[0].message_set_name.as_deref(),
        Some("Msg")
    );
}
/// UI-EVT-PAYLOAD：声明侧允许 payload 变体（执行拒绝在 SESSION dispatch 前）。
#[test]
fn p981_event_a_payload_variants_are_declared_not_executed() {
    let src = "node M {\n    msg Msg {\n        Add(str),\n        Set(int)\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let d = compile(src).expect("payload variants are legal declarations");
    let add = d.components[0]
        .messages
        .iter()
        .find(|m| m.name == "Add")
        .expect("Add variant");
    assert_eq!(add.payload, vec!["str".to_string()]);
    assert!(
        d.components[0].handler_params.is_empty(),
        "no handler declares parameters here"
    );
}

/// UI-EVT-MSG 负控：重复变体仍具名拒；普通标识符不受消息名影响。
#[test]
fn p981_event_a_message_controls_hold() {
    let dup = "node M {\n    msg Msg {\n        Inc,\n        Inc\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    match compile(dup) {
        Ok(_) => panic!("a duplicate variant must be rejected"),
        Err(ds) => assert!(
            ds[0].code.contains("duplicate") || ds[0].message.contains("duplicate"),
            "{ds:?}"
        ),
    }
    let plain = "node M {\n    model {\n        Add int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    compile(plain).expect("ordinary identifiers keep working next to message names");
}
/// UI-EVT-PAYLOAD（A 语义 → B-W1 升级）：向有载荷的消息投递**无值 Send** 是
/// arity 域错误（零效果，`Send` = 空载荷）；unit 消息照常。真实带值派发见
/// `plan981_event_b`（IF-981-EVENT-PAYLOAD-B）。
#[test]
fn p981_event_a_payload_dispatch_is_named_rejected() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node M {\n    msg Msg {\n        Add(str)\n    }\n    on {\n        .Add(s) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Add".into(),
        }],
    };
    let r = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    );
    match r {
        Ok(_) => panic!("a payload dispatch without its value must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.component-event", "{ds:?}");
            assert!(
                ds[0].message.contains("payload slot"),
                "the rejection names the arity: {}",
                ds[0].message
            );
        }
    }

    // unit 消息的同一条 Send 路径不受影响。
    let unit = "node M {\n    msg {\n        Inc\n    }\n    on {\n        .Inc -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let unit_scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        }],
    };
    let out = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(unit)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &unit_scenario,
    );
    assert!(
        out.is_ok(),
        "a unit message dispatch still compiles: {out:?}"
    );
}

/// R66-1：另一个组件的同名 payload 变体不能误伤合法 unit 派发（只有真的处理该
/// 消息的 node 才参与 payload 判定）。
#[test]
fn p981_event_a_cross_component_payload_does_not_block_unit_dispatch() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node A {\n    msg {\n        Add\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    msg Msg {\n        Add(str)\n    }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Add".into(),
        }],
    };
    let out = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    );
    assert!(
        out.is_ok(),
        "node B declares Add(str) but does not handle it; A's unit dispatch stays legal: {out:?}"
    );
}
