//! PLAN-980 ④-A 首切片 #2（AV 面）：**AGE 包闭包**在 AV 车道上的真实执行。
//!
//! 输入是 ④-A 派生真实例（`ace-node-counter` 派生 + 点分 `use util.steps: step`
//! + 惰性或 `use themes.themes: Auto, Solarized`），场景 2 实例 + 定向 `Inc`。
//! 观察走 `ace_av::age_package::observe_age_package`（同一 lower→MIR verify→ABC
//! →AV 执行链），判据：①被引用 helper 的值进入 AV 观察结果；②改 helper 源即改
//! AV 观察值（不是声明摆设）；③未引用的 themes 导入不要求模块在场。
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package_with_scenario, AgePackageCompilation};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn unit(pkg: &str, id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: pkg.into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 派生真实例：计数业务（初始化 `step(0)`、`Inc` 走 `step(.count)`）。
fn derived_counter() -> String {
    "use util.steps: step\n\
use themes.themes: Auto, Solarized\n\
\n\
node Counter {\n\
    msg { Inc, Reset }\n\
\n\
    model {\n\
        var count int = step(0)\n\
    }\n\
\n\
    view {\n\
        col {\n\
            text \"Counter\"\n\
        }\n\
    }\n\
\n\
    on {\n\
        .Inc -> { .count = step(.count) }\n\
        .Reset -> { .count = 0 }\n\
    }\n\
}\n"
    .to_string()
}

const HELPER: &str = "pub fn step(n int) int {\n    return n + 2\n}\n";

fn compile(helper: &str, ops: Vec<ScenarioOp>) -> AgePackageCompilation {
    let caps = vec![PACKAGES_CAPABILITY.to_string()];
    let scenario = ComponentScenario { instances: 2, ops };
    compile_age_package_with_scenario(
        &[
            unit("app", "main", &derived_counter()),
            unit("util", "steps", helper),
        ],
        "app::main",
        LegacyWidgetAlias::Accept,
        &caps,
        &scenario,
    )
    .expect("derived counter + dotted helper compiles under the scenario")
}

fn inc_once() -> Vec<ScenarioOp> {
    vec![ScenarioOp::Send {
        instance: 0,
        msg: "Inc".to_string(),
    }]
}

/// ① helper 的值进入 AV 观察结果（i0: 初始化 `step(0)`=2 → `Inc`: `step(2)`=4；
/// i1 只初始化=2）；未引用的 themes 模块不在场仍编译通过。
#[test]
fn p980_closure_av_observes_helper_computed_state() {
    let out = compile(HELPER, inc_once());
    let observed =
        ace_av::age_package::observe_age_package(&out, 2).expect("package is observable on AV");
    let c = &observed.components[0];
    assert_eq!(c.name, "Counter");
    assert_eq!(
        c.instance_states[0][0], 4,
        "instance 0: step(0) then step(2) -> 4 via the imported helper"
    );
    assert_eq!(
        c.instance_states[1][0], 2,
        "instance 1: initial step(0) only"
    );
}

/// ② 改 helper 源 → AV 观察值随之改变（helper 真被编译执行，而非只被绑定）。
#[test]
fn p980_closure_av_values_follow_the_helper_source() {
    let changed = HELPER.replace("return n + 2", "return n + 10");
    let out = compile(&changed, inc_once());
    let observed = ace_av::age_package::observe_age_package(&out, 2).expect("observable");
    let c = &observed.components[0];
    assert_eq!(c.instance_states[0][0], 20, "step(0)=10 then step(10)=20");
    assert_eq!(c.instance_states[1][0], 10, "initial step(0)=10");
}
