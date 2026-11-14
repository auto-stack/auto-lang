//! PLAN-979 T-03：组件场景的 **AV 真值**（共享 lower→MIR→ABC→AV）。
//!
//! 两个同定义实例、定向事件与 Tick：观察编码 22 = instance0(2) + instance1(2)*10，
//! 证明实例隔离（事件只改目标实例）与 Tick 推进次数精确；文本叶仍取共享 Str 值。

use ace_av::age_component::observe_component;
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;

const COUNTER: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Tick {\n        count = count + 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

fn scenario() -> ComponentScenario {
    ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Tick { instance: 1 },
        ],
    }
}

#[test]
fn p979_component_av_observes_two_isolated_instances() {
    let o = observe_component(
        COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &scenario(),
    )
    .expect("AV observes the component scenario");
    // instance0 收到 2 次 Inc；instance1 收到 1 次 Inc + 1 次 Tick；编码 2 + 2*10。
    assert_eq!(o.encoded, 22);
    assert_eq!(o.instance_texts.len(), 2);
    assert_eq!(o.instance_texts[0], vec!["c".to_string()]);
    assert_eq!(o.instance_texts[1], vec!["c".to_string()]);
}

#[test]
fn p979_component_av_event_order_changes_the_observation() {
    // 只给 instance1 发事件：instance0 保持 0 → 0 + 1*10 = 10（定向 2:1 的证据）。
    let one_sided = ComponentScenario {
        instances: 2,
        ops: vec![ScenarioOp::Send {
            instance: 1,
            msg: "Inc".to_string(),
        }],
    };
    let o = observe_component(COUNTER, "counter.at", LegacyWidgetAlias::Reject, &one_sided)
        .expect("one-sided events observe");
    assert_eq!(o.encoded, 10);
}

#[test]
fn p979_component_av_rejects_undeclared_event_before_execution() {
    let bad = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Nope".to_string(),
        }],
    };
    let err = observe_component(COUNTER, "counter.at", LegacyWidgetAlias::Reject, &bad)
        .expect_err("undeclared event is refused");
    assert!(err.contains("source.age.component-event"), "{err}");
}

/// V-09 完整形状：helper 参与 init/handler，view 文本读状态（typed 绑定），N 次 Tick。
const HELPER_COUNTER: &str = "fn seed() int {\n    return 0\n}\nfn bump(n int) int {\n    return n + 1\n}\nfn label(n int) str {\n    if n > 2 {\n        return \"big\"\n    }\n    return \"small\"\n}\nnode Counter {\n    model {\n        count int = seed()\n    }\n    on Inc {\n        count = bump(count)\n    }\n    on Tick {\n        count = bump(count)\n    }\n    view {\n        text label(count) {}\n    }\n}\n";

#[test]
fn p979_component_helpers_participate_and_view_reads_state() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
        ],
    };
    let o = observe_component(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &scenario,
    )
    .expect("helpers observe");
    assert_eq!(o.encoded, 13);
    assert_eq!(o.instance_texts[0], vec!["big".to_string()]);
    assert_eq!(o.instance_texts[1], vec!["small".to_string()]);
}

#[test]
fn p979_component_n_ticks_advance_each_instance_exactly_n_times() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 1 },
            ScenarioOp::Tick { instance: 1 },
        ],
    };
    let o = observe_component(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &scenario,
    )
    .expect("tick matrix observes");
    assert_eq!(o.encoded, 24);
    assert_eq!(o.instance_texts[0], vec!["big".to_string()]);
    assert_eq!(o.instance_texts[1], vec!["small".to_string()]);
}
