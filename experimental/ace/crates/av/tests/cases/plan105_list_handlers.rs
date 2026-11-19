//! PLAN-105 T-02（U03）：组件面 **collections 域**（列表 handler）端到端回归——\n//! `List<T>.new()` + `push` + `get` 在场景事件里执行并写回 int 字段。
use ace_av::age_component::{
    compile_component_scenario_with_initials, observe_component_states, ComponentInitials,
    ComponentScenario, LegacyWidgetAlias, ScenarioOp,
};

#[test]
fn p105_list_handler_executes_end_to_end() {
    let src = "widget App {\n    msg { M }\n    model { var n int = 0 }\n    view { text \"x\" {} }\n    on {\n        .M -> {\n            let xs = List<int>.new()\n            xs.push(7)\n            .n = xs.get(0)\n        }\n    }\n}\n";
    let caps = vec!["ac.runtime.text.v1".to_string()];
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![ScenarioOp::Send {
            instance: 0,
            msg: "M".to_string(),
        }],
    };
    let initials = ComponentInitials {
        per_instance: vec![Vec::new()],
    };
    let compiled = compile_component_scenario_with_initials(
        src,
        "p105.at",
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
        None,
        &initials,
    )
    .expect("collections component compiles");
    let states = observe_component_states(&compiled).expect("states observe");
    let n = states
        .per_instance
        .first()
        .and_then(|f| f.iter().find(|(k, _)| k == "n"))
        .map(|(_, v)| format!("{v:?}"))
        .unwrap_or_default();
    println!("[p105probe] n={n}");
    assert!(n.contains('7'), "list push/get reached the field: {n}");
}
