//! PLAN-981（IF-981-EVENT-A）：980/A 正式源带来的**事件 A 子面**。
//!
//! 覆盖 985/EVENT-SPLIT-20261116 要求的三个 A 键在 981 侧的最小可交面：
//! - **UI-EVT-MSG**：`msg Msg { … }` 具名段与匿名 `msg { … }` 并存，变体名/原 span/具名段
//!   保真，重复变体仍具名拒；
//! - **UI-EVT-BIND**：handler 参数名/span 被**读取保留**；参数**行为**在 A 未实现，因此带参
//!   handler 在 admit 前具名拒（`source.age.event-params-unsupported`），零状态/UI 效果——
//!   不静默丢参后跑常量体；
//! - **UI-EVT-PAYLOAD**：声明侧允许（`Add(str)`），但执行侧未实现；拒绝由 AGE 前门在
//!   dispatch 前给出（编译期拒绝，宿主透出该诊断；见 auto-ui 侧 `s2` 用例）。
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

fn compile(src: &str) -> Result<ace_core::source::age_package::AgePackageDescription, Vec<ace_core::atom_text::Diagnostic>> {
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
    assert!(c.messages.iter().all(|m| m.payload.is_empty()), "unit variants carry no payload");
    assert_eq!(c.handler_params, vec![("Add".to_string(), Vec::new())]);

    let anon = "node M {\n    msg {\n        Inc\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let d = compile(anon).expect("an anonymous message set still compiles");
    assert_eq!(d.components[0].message_set_name, None);
    assert_eq!(d.components[0].messages.len(), 1);
}

/// UI-EVT-BIND：参数名/span 被保留并可从描述面读到；**派发**时具名拒（零效果）。
#[test]
fn p981_event_a_param_handler_is_named_rejected() {
    let src = "node M {\n    msg Msg {\n        Add\n    }\n    on {\n        .Add(t) -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    let want = src.find("(t)").expect("fixture has the param") + 1;
    // 解析与描述成功：参数事实可读（不是静默丢弃）。
    let d = compile(src).expect("a parameterised handler parses and is described");
    assert_eq!(
        d.components[0].handler_params,
        vec![("Add".to_string(), vec!["t".to_string()])]
    );
    // 派发时具名拒，span 指参数名。
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
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
        Ok(_) => panic!("dispatching a parameterised handler must be rejected"),
        Err(ds) => {
            assert_eq!(ds[0].code, "source.age.event-params-unsupported", "{ds:?}");
            assert_eq!(ds[0].span.start, want);
        }
    }
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

/// R65-2：unit 变体在前、有载荷变体在后时，无值派发仍必须拒。
#[test]
fn p981_event_a_unit_first_payload_second_dispatch_is_rejected() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node M {\n    msg {\n        Add\n    }\n    msg Msg {\n        Add(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    // 该源本身因跨块重复变体被拒（R65-1）；构造一个只声明一次、跨块不重复的
    // 对照组：unit `Add` 与 payload `Set`，两处都能走到 payload 守卫。
    let ok = "node M {\n    msg {\n        Add\n    }\n    msg Msg {\n        Set(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
    assert!(compile(src).is_err(), "cross-block duplicates already rejected");
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
        Ok(_) => panic!("dispatching the payload variant must be rejected"),
        Err(ds) => assert_eq!(ds[0].code, "source.age.event-payload-unsupported", "{ds:?}"),
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
    assert_eq!(out.description.components[0].message_set_name.as_deref(), Some("Msg"));
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
/// UI-EVT-PAYLOAD：向有载荷的消息投递（无值）必须具名拒，零效果；unit 消息照常。
#[test]
fn p981_event_a_payload_dispatch_is_named_rejected() {
    use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
    let src = "node M {\n    msg Msg {\n        Add(str)\n    }\n    on {\n        .Add -> {\n            .count = count + 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"t\" {}\n    }\n}\n";
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
            assert_eq!(ds[0].code, "source.age.event-payload-unsupported", "{ds:?}");
            assert!(
                ds[0].message.contains("payload"),
                "the rejection names the payload: {}",
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
    assert!(out.is_ok(), "a unit message dispatch still compiles: {out:?}");
}
