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

// ---- V-10：坏输入在共享检查处拒（原 span），AV 不执行；修复后同入口恢复 ----

const BAD_HANDLER: &str = "fn bump(n int) int {\n    return n + 1\n}\nfn as_text(n int) str {\n    return \"x\"\n}\nnode Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = as_text(count)\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

#[test]
fn p979_component_v10_bad_handler_type_is_refused_without_execution() {
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let err = observe_component(
        BAD_HANDLER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &scenario,
    )
    .expect_err("a str value cannot be assigned to an int model field");
    // 共享检查具名拒（不是 AV 执行期失败）：类型不匹配由 verify 给出。
    assert!(err.contains("source.type-mismatch"), "{err}");
    assert!(
        err.contains("i32") && err.contains("str"),
        "both types named: {err}"
    );
}

#[test]
fn p979_component_v10_undeclared_assignment_field_is_named_at_its_span() {
    let src = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        other = 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let err = observe_component(src, "counter.at", LegacyWidgetAlias::Reject, &scenario)
        .expect_err("assigning an undeclared model field is refused");
    assert!(err.contains("source.age.component-event"), "{err}");
    assert!(err.contains("other"), "{err}");
}

#[test]
fn p979_component_v10_recovery_after_failure_keeps_the_good_values() {
    // 坏 → 好：同一入口在失败之后仍给出正确观察值（无跨调用状态/工件污染）。
    let bad = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Nope".to_string(),
        }],
    };
    assert!(observe_component(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &bad
    )
    .is_err());
    let good = ComponentScenario {
        instances: 2,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "Inc".to_string(),
        }],
    };
    let o = observe_component(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &good,
    )
    .expect("the good scenario still observes");
    assert_eq!(o.encoded, 1);
    assert_eq!(o.instance_texts[0], vec!["small".to_string()]);
}

#[test]
fn p979_component_v10_scenario_change_invalidates_the_artifact_identity() {
    use ace_core::source::age_component::compile_component_scenario;
    let caps = ["ac.runtime.text.v1".to_string()];
    let a = compile_component_scenario(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            }],
        },
    )
    .expect("scenario a");
    let b = compile_component_scenario(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".to_string(),
                },
                ScenarioOp::Send {
                    instance: 0,
                    msg: "Inc".to_string(),
                },
            ],
        },
    )
    .expect("scenario b");
    // 源哈希不含脱糖：场景是脱糖的一部分 → 换场景换"脱糖身份"（旧工件不得复用）。
    assert_eq!(
        a.input_hash, b.input_hash,
        "source identity is the same source"
    );
    assert_ne!(
        a.desugar_hash, b.desugar_hash,
        "scenario change changes the checked artifact"
    );
    // 同源同场景稳定。
    let a2 = compile_component_scenario(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            }],
        },
    )
    .expect("scenario a again");
    assert_eq!(a.desugar_hash, a2.desugar_hash);
}
