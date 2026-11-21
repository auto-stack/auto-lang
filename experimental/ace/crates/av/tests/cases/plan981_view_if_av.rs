//! PLAN-981（view-if 第二片）：**实际消费平铺文本的观察入口**的具名拒。
//!
//! 主线（980 `d69f29f91`）把两支文本平铺进 `view.texts`（绑定验证同覆盖，
//! 编译保持通过）。观察入口若照平铺逐叶取值，两支会被当作**同时可见**——
//! 本文件在三个真实观察入口钉 fail-closed：
//!
//! - `observe_component_texts`（组件静态文本观察）；
//! - `observe_component`（场景观察，经 `observe_compiled` 暖路径）；
//! - `observe_age_package`（包观察）。
//!
//! 拒收均为 `source.age.view-conditional` 且消息携带作者最前 `if` 的字节偏移
//! （锚定事实来自描述面 `cond_spans`，core 侧 `plan981_view_if` 钉同一偏移）。
//! states-only/SFC 车道不经观察面，不受影响（core 侧用例钉）。

use ace_av::age_component::{
    compile_component_scenario_states, compile_component_scenario_with_initials, observe_component,
    observe_component_states_and_texts, observe_component_texts,
};
use ace_av::age_package::observe_age_package;
use ace_core::source::age_component::ComponentScenario;
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, compile_age_package_with_scenario};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

/// 场景观察腿（需 int 观测字段）。
const SRC: &str = "node N {\n    model {\n        count int = 0\n        open bool = true\n    }\n    view {\n        if .open {\n            text \"on\" {}\n        } else {\n            text \"off\" {}\n        }\n    }\n}\n";
/// 静态文本观察腿（`compile_age_component` 旧静态面：无 model/handler 才入）。
const SRC_STATIC: &str = "node N {\n    view {\n        if .open {\n            text \"on\" {}\n        } else {\n            text \"off\" {}\n        }\n    }\n}\n";

fn if_at() -> usize {
    SRC.find("if .open").expect("author if offset")
}

fn if_at_static() -> usize {
    SRC_STATIC.find("if .open").expect("author if offset")
}

/// 组件静态文本观察：具名拒 + 消息锚作者 `if` 偏移。
#[test]
fn p981_observe_component_texts_rejects_conditional_named() {
    let err = observe_component_texts(SRC_STATIC, "app::main", LegacyWidgetAlias::Reject)
        .expect_err("static observation must fail closed on conditionals");
    assert!(
        err.contains("source.age.view-conditional"),
        "named code, got: {err}"
    );
    assert!(
        err.contains(&format!("@{}..", if_at_static())),
        "message anchors at the author `if` offset, got: {err}"
    );
}

/// 场景观察（observe_compiled 暖路径）：同拒。
#[test]
fn p981_observe_component_scenario_rejects_conditional_named() {
    let err = observe_component(
        SRC,
        "app::main",
        LegacyWidgetAlias::Reject,
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect_err("scenario observation must fail closed on conditionals");
    assert!(
        err.contains("source.age.view-conditional") && err.contains(&format!("@{}..", if_at())),
        "named code + author-if anchor, got: {err}"
    );
}

/// 包观察：编译保持通过（绑定检查），观察入口具名拒且非解析墙。
#[test]
fn p981_observe_age_package_rejects_conditional_named() {
    let compiled = compile_age_package_with_scenario(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: SRC.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 2,
            ops: Vec::new(),
        },
    )
    .expect("package compile passes; only observation fails closed");
    let err = observe_age_package(&compiled, 2)
        .expect_err("package observation must fail closed on conditionals");
    assert!(
        err.contains("source.age.view-conditional") && err.contains(&format!("@{}..", if_at())),
        "named code + author-if anchor, got: {err}"
    );
    let layout = ace_av::age_package::observe_age_package_layout_inputs(&compiled, 2)
        .expect("renderer receives the checked state and every declared text slot");
    assert_eq!(layout.components[0].instance_states, [vec![0], vec![0]]);
    assert_eq!(
        layout.components[0].instance_texts,
        [vec!["on", "off"], vec!["on", "off"]]
    );
}

/// NATIVE 双观察面（复审 F-1 补口）：texts 半边同律具名拒——states-only 编译
/// 通过（SFC/桌面邻接车道不动），双观察入口拒且锚作者 `if`。
#[test]
fn p981_observe_states_and_texts_rejects_conditional_named() {
    use ace_core::source::age_component::ComponentInitials;
    let compiled = compile_component_scenario_states(
        SRC,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
        &ComponentInitials {
            per_instance: vec![Vec::new()],
        },
    )
    .expect("states-only compile passes; only the dual observation fails closed");
    let err = observe_component_states_and_texts(&compiled)
        .expect_err("dual observation texts half must fail closed on conditionals");
    assert!(
        err.contains("source.age.view-conditional") && err.contains(&format!("@{}..", if_at())),
        "named code + author-if anchor, got: {err}"
    );
}

/// 负控：无条件的普通 view——三个观察入口照常工作（门不误拒）。
#[test]
fn p981_observation_entries_pass_plain_views() {
    let plain = "node N {\n    model {\n        count int = 0\n    }\n    view {\n        text \"plain\" {}\n    }\n}\n";
    // 静态腿走旧静态面（无 model 源）。
    let texts = observe_component_texts(
        "node N {\n    view {\n        text \"plain\" {}\n    }\n}\n",
        "app::main",
        LegacyWidgetAlias::Reject,
    )
    .expect("plain static observation unaffected");
    assert_eq!(texts, vec![vec!["plain".to_string()]]);
    observe_component(
        plain,
        "app::main",
        LegacyWidgetAlias::Reject,
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect("plain scenario observation unaffected");
    // NATIVE 双观察面（states+texts）同样不误拒。texts 半边需要发射文本叶
    // 函数的编译道（initials 道，emit_view_texts=true）；states-only 道按合同
    // 给空行，不在此断言。
    {
        use ace_core::source::age_component::ComponentInitials;
        let compiled = compile_component_scenario_with_initials(
            plain,
            "app::main",
            LegacyWidgetAlias::Reject,
            &[PACKAGES_CAPABILITY.to_string()],
            &ComponentScenario {
                instances: 1,
                ops: Vec::new(),
            },
            None,
            &ComponentInitials {
                per_instance: vec![Vec::new()],
            },
        )
        .expect("plain initials compile (emits text leaves)");
        let (_, texts) = observe_component_states_and_texts(&compiled)
            .expect("plain dual observation unaffected");
        assert_eq!(texts, vec![vec!["plain".to_string()]]);
    }
    let compiled = compile_age_package(
        &[PackageModuleInput {
            package: "app".into(),
            version: "1.0.0".into(),
            id: "main".into(),
            source: plain.into(),
            provider: None,
        }],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("plain package compiles");
    observe_age_package(&compiled, 1).expect("plain package observation unaffected");
}
