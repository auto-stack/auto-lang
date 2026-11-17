//! PLAN-980 WEB 切片#6m（年龄面语法收敛·永久用例）：widget 工程真语料
//! 主导形态进组件面——事件绑定调用形（`.Msg(args)`；`$event` 由 UI 侧
//! `nevent` 同长归一化承载）、事件键修饰符链（`onclick.self`）、元素括号
//! 属性行（`input (placeholder: "s") { … }`）。编译经 states-only 车道
//! （text 叶门只约束文本观测车道，§13.13）。

use ace_av::age_component::{
    compile_component_scenario_states, ComponentInitials, ComponentScenario, LegacyWidgetAlias,
};

#[test]
fn p980_web_6m_grammar_forms_compile_and_reject_by_name() {
    let ok_cases: &[(&str, &str)] = &[
        (
            "event-binding-call",
            "widget T {\n    msg Msg { Go(str) }\n    model { var x int = 0 }\n    view { text \"t\" oninput: .Go(arg) }\n}\n",
        ),
        (
            "event-key-modifier",
            "widget T {\n    model { var x int = 0 }\n    view { col { text \"t\" button \"b\" { onclick.self: .Go } } }\n}\n",
        ),
        (
            "element-paren-props",
            "widget T {\n    model { var x int = 0 }\n    view { col { text \"t\" input (placeholder: \"s\") { } } }\n}\n",
        ),
        (
            "element-paren-props-bodyless",
            "widget T {\n    model { var x int = 0 }\n    view { col { text \"t\" input (placeholder: \"s\") } }\n}\n",
        ),
    ];
    for (name, src) in ok_cases {
        let r = compile_component_scenario_states(
            src,
            "p980-6m.at",
            LegacyWidgetAlias::Accept,
            &["ac.runtime.text.v1".to_string()],
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
