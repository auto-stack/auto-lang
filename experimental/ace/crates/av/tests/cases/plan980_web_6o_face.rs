//! PLAN-980 WEB 切片#6o（④语法收全·fail-closed 承载·永久用例）：widget
//! 头参数表（age-component 开放 age-node-params——参数=宿主通道 prop，
//! 不进字段/handler 语义）；`;` 单行语句分隔；赋值体经 states-only 编译。

use ace_av::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario, LegacyWidgetAlias,
};

#[test]
fn p980_web_6o_head_params_and_semi_compile() {
    let ok_cases: &[(&str, &str)] = &[
        (
            "head-props",
            "widget App(on_delete: msg, count: int) {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { text \"t\" } }
    on { .Go -> { .x = 1 } }
}
",
        ),
        (
            "semi-separated",
            "widget T {
    msg Msg { Go }
    model { var x int = 0 }
    view { col { text \"t\" } }
    on { .Go -> { .x = 1; .x = 2 } }
}
",
        ),
    ];
    for (name, src) in ok_cases {
        let r = compile_component_scenario_states(
            src,
            "p980-6o.at",
            LegacyWidgetAlias::Accept,
            &[],
            &ComponentScenario {
                instances: 1,
                ops: vec![],
            },
            &ComponentInitials {
                per_instance: vec![vec![]],
            },
        );
        assert!(r.is_ok(), "{name}: {r:?}");
    }
}
