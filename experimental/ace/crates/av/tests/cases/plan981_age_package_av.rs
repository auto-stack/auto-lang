//! PLAN-981（SOURCE 批，IF-981-MODULE）：AGE 包面的 **AV 真实值**回归。
//!
//! 这是 979 V-09④ 要的那条腿的 AV 半边：两文件 AGE 工程 → 同一包前门 →
//! 两实例 2:1 定向事件、各 4 次 `Life.Tick` → 经共享 AV 链观察**逐实例状态与
//! 文本**，并与独立算术 oracle 对齐。
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const HELPER: &str = "pub fn seed() int { return 0 }\n\
pub fn label(n int) str {\n    if n == 6 {\n        return \"six\"\n    }\n    return \"other\"\n}\n";

const MAIN: &str = "use helper.seed\n\
use helper.label\n\
node Counter {\n\
    model { count int = seed() }\n\
    on Inc { count = count + 1 }\n\
    on Life.Tick { count = count + 1 }\n\
    view { text label(count) {} }\n\
}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn caps() -> Vec<String> {
    vec![PACKAGES_CAPABILITY.to_string()]
}

/// 两实例 2:1 事件 + 各 4 次 Tick：
///   实例 0：2 次 Inc + 4 次 Tick = 6  → 文本 "six"
///   实例 1：1 次 Inc + 4 次 Tick = 5  → 文本 "other"
fn mixed_scenario() -> ComponentScenario {
    let mut ops = Vec::new();
    // 定向事件按 2:1 交错投递（顺序本身也必须不影响各实例自己的计数）。
    ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Inc".into(),
    });
    ops.push(ScenarioOp::Send {
        instance: 1,
        msg: "Inc".into(),
    });
    ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Inc".into(),
    });
    for _ in 0..4 {
        ops.push(ScenarioOp::Tick { instance: 0 });
        ops.push(ScenarioOp::Tick { instance: 1 });
    }
    ComponentScenario { instances: 2, ops }
}

#[test]
fn p981_age_package_two_instances_two_to_one_events_and_four_ticks() {
    let scenario = mixed_scenario();
    let compiled = compile_age_package_with_scenario(
        &[input("helper", HELPER), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &scenario,
    )
    .expect("two-instance AGE package compiles");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("the shared AV chain runs both instances");

    assert_eq!(observed.components.len(), 1);
    let c = &observed.components[0];
    assert_eq!(c.name, "Counter");

    // 独立算术 oracle：6 / 5（helper 初值 0，实例 0 得 2 Inc、实例 1 得 1 Inc）。
    assert_eq!(
        c.instance_states,
        vec![vec![6], vec![5]],
        "instance 0 = 2 Inc + 4 Tick, instance 1 = 1 Inc + 4 Tick"
    );
    // 同一 node 定义、两个**独立**实例：文本随各自状态取不同分支。
    assert_eq!(c.instance_texts, vec![vec!["six".to_string()], vec!["other".to_string()]]);
    // Tick 恰推进实例自己的计数（不串扰）：各实例 4 次。
    assert_eq!(c.handler_msgs, vec!["Inc".to_string(), "Life.Tick".to_string()]);
}

/// 改被导入 helper 的初值 → 同一场景下两个实例的值都必须变（跨模块真受检）。
#[test]
fn p981_age_package_imported_helper_value_drives_both_instances() {
    let changed = HELPER.replace("return 0 }", "return 10 }");
    let scenario = mixed_scenario();
    let compiled = compile_age_package_with_scenario(
        &[input("helper", &changed), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &scenario,
    )
    .expect("changed helper compiles");
    let observed =
        ace_av::age_package::observe_age_package(&compiled, 2).expect("AV observes both");
    assert_eq!(observed.components[0].instance_states, vec![vec![16], vec![15]]);
    // 两个实例都越过了 6，故都走 "other" 分支 —— 文本不是常量补的。
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["other".to_string()], vec!["other".to_string()]]
    );
}

/// 场景自检：未声明的消息与越界实例在编译期具名拒绝（不进入运行）。
#[test]
fn p981_age_package_scenario_failures_are_named() {
    let mut bad_msg = mixed_scenario();
    bad_msg.ops.push(ScenarioOp::Send {
        instance: 0,
        msg: "Nope".into(),
    });
    let ds = compile_age_package_with_scenario(
        &[input("helper", HELPER), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps(),
        &bad_msg,
    )
    .expect_err("an undeclared message is rejected");
    assert!(
        ds.iter().any(|d| d.code == "source.age.component-event"),
        "{ds:?}"
    );
}
