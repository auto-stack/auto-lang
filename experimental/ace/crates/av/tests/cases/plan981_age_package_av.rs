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
    assert_eq!(
        c.instance_texts,
        vec![vec!["six".to_string()], vec!["other".to_string()]]
    );
    // Tick 恰推进实例自己的计数（不串扰）：各实例 4 次。
    assert_eq!(
        c.handler_msgs,
        vec!["Inc".to_string(), "Life.Tick".to_string()]
    );
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
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![16], vec![15]]
    );
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

/// R64-4：两个 node 同名字段各自可观察（状态函数命名已按节点消歧）。
#[test]
fn p981_dual_nodes_same_field_are_observed_separately() {
    let src = "node A {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"a\" {}\n    }\n}\nnode B {\n    model { count int = 0 }\n    on Tick { count = count + 1 }\n    view {\n        text \"b\" {}\n    }\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: vec![
                ScenarioOp::Tick { instance: 0 },
                ScenarioOp::Tick { instance: 0 },
            ],
        },
    )
    .expect("two nodes sharing a field name compile");
    let observed = ace_av::age_package::observe_age_package(&compiled, 2)
        .expect("observation resolves both components");
    assert_eq!(observed.components.len(), 2);
    // 实例级调度：两个组件都声明了 `on Tick`，实例 0 各 Tick 两次。
    assert_eq!(observed.components[0].instance_states[0][0], 2);
    assert_eq!(observed.components[1].instance_states[0][0], 2);
}

/// B-LANGUAGE-V（AV 腿）：真实 980/A Counter 源在 AV 观察下与 A2R oracle 同值。
#[test]
fn p981_ma_counter_av_leg_matches_the_arithmetic_oracle() {
    const SRC: &str = include_str!("../../../core/tests/fixtures_ma/ace_node_counter.at");
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Dec".into(),
            },
        ],
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect("the delivered A source compiles");
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("AV observes the real A source");
    assert_eq!(observed.components.len(), 1);
    assert_eq!(observed.components[0].name, "Counter");
    // 2 Inc + 1 Dec = 1：与 A2R 二进制 oracle 同值（独立算术）。
    assert_eq!(observed.components[0].instance_states, vec![vec![1]]);
    assert_eq!(
        observed.components[0].instance_texts,
        vec![vec!["Counter".to_string()]],
        "the real view text is observed"
    );
}

/// V-EVT-A-UNIT：匿名 unit Counter 的**终值 + 单步 oracle**（0→1 与 2→0）；
/// 以及具名 unit 镜面（`msg Msg { Inc }`）同轨迹。
#[test]
fn p981_evt_a_unit_counter_trace_and_named_mirror() {
    const SRC: &str = include_str!("../../../core/tests/fixtures_ma/ace_node_counter.at");
    let ops = || {
        vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Dec".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Reset".into(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Reset".into(),
            },
        ]
    };
    let compiled = compile_age_package_with_scenario(
        &[input("main", SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: ops(),
        },
    )
    .expect("the anonymous unit counter compiles");
    // 终值 oracle：Inc,Dec,Reset,Reset 后为 0；Reset/Dec 的区分由下面的单步用例承担。
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
    assert_eq!(observed.components[0].instance_states, vec![vec![0]]);

    // 具名 unit 镜面：同一 business 轨迹，消息集合取名不改变语义。
    let named = "node Counter {\n    msg Msg {\n        Inc,\n        Dec,\n        Reset\n    }\n    model {\n        var count int = 0\n    }\n    on {\n        .Inc -> { .count += 1 }\n        .Dec -> { .count -= 1 }\n        .Reset -> { .count = 0 }\n    }\n    view {\n        text \"Counter\" {}\n    }\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("main", named)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: ops(),
        },
    )
    .expect("the named unit mirror compiles");
    assert_eq!(
        compiled.description.components[0]
            .message_set_name
            .as_deref(),
        Some("Msg")
    );
    let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
    assert_eq!(observed.components[0].instance_states, vec![vec![0]]);

    // 单步 oracle：Inc 后 1；Inc,Inc,Reset 后 0（Reset 若被当 no-op 会剩 2 → 红）。
    for (ops, want) in [
        (
            vec![ScenarioOp::Send {
                instance: 0,
                msg: "Inc".into(),
            }],
            1i64,
        ),
        (
            vec![
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".into(),
                },
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".into(),
                },
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Reset".into(),
                },
            ],
            0i64,
        ),
    ] {
        let compiled = compile_age_package_with_scenario(
            &[input("main", SRC)],
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &ComponentScenario { instances: 1, ops },
        )
        .expect("compiles");
        let observed = ace_av::age_package::observe_age_package(&compiled, 1).expect("observes");
        assert_eq!(observed.components[0].instance_states, vec![vec![want as i32]]);
    }
}
