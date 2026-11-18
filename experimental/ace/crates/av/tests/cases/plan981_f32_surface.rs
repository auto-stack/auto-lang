//! PLAN-981（D-981-AGE-F32-SURFACE）：AGE **f32** 面的 AV 执行证据。
//!
//! 真实 AGE 源（不是等价手写 HIR）：f32 模型字段 + handler 体里的同型
//! 四则/比较/一元负号；Tick 重放后逐字段状态 def 经共享
//! lower→MIR verify→ABC admit→AV 取真值。
//!
//! 观察出口：窄枚举 `ComponentFieldValue` 只承载 int/bool/str（auto-ui
//! `AceStateValue` 对它穷尽匹配，扩 F32 须跨仓同窗），因此 f32 走
//! `execute_component_def` 返回共享 `Value`——与状态对象是**同一条**执行链，
//! 不另建求值路径。
//!
//! 边界（本片不覆盖，具名）：①`float`/`f64` 双宽度字面量（`1.5d`）不参与本面；
//! ②`ComponentFieldValue` 仍未含 F32（消费端同步改造，见 `observe_component_states`
//! 的具名错误）；③f32 文本呈现（text 叶/f-string）仍按原子编码的 i32 面拒绝。

use ace_av::age_component::{
    compile_component_scenario_states, execute_component_def, ComponentCompilation,
    ComponentInitials, ComponentScenario, LegacyWidgetAlias, ScenarioOp,
};
use ace_av_core::contract::Value;

/// 最小真实源：三个 f32 字段 + 一条积分 handler（与 UI058 移动积分同形）。
const MOVER: &str = "node Mover {\n    model {\n        x f32 = 0.0\n        vx f32 = 2.0\n        dt f32 = 0.5\n    }\n    on Tick {\n        x = x + vx * dt\n    }\n    view {\n        text \"m\" {}\n    }\n}\n";

/// 同型运算全表：四则/取余/比较/一元负号都用**字段**承载（不靠字面量折叠）。
const OPS: &str = "node Ops {\n    model {\n        x f32 = 9.0\n        y f32 = 2.0\n        sum f32 = 0.0\n        dif f32 = 0.0\n        mul f32 = 0.0\n        div f32 = 0.0\n        rem f32 = 0.0\n        neg f32 = 0.0\n        eq bool = false\n        ne bool = false\n        lt bool = false\n        le bool = false\n        gt bool = false\n        ge bool = false\n    }\n    on Tick {\n        sum = x + y\n        dif = x - y\n        mul = x * y\n        div = x / y\n        rem = x % y\n        neg = -y\n        eq = x == y\n        ne = x != y\n        lt = x < y\n        le = x <= y\n        gt = x > y\n        ge = x >= y\n    }\n    view {\n        text \"o\" {}\n    }\n}\n";

fn initials(instances: usize) -> ComponentInitials {
    ComponentInitials {
        per_instance: vec![Vec::new(); instances],
    }
}

fn compile(src: &str, ticks: usize) -> Result<ComponentCompilation, String> {
    let ops = (0..ticks)
        .map(|_| ScenarioOp::Tick { instance: 0 })
        .collect();
    let scenario = ComponentScenario { instances: 1, ops };
    compile_component_scenario_states(
        src,
        "mover.at",
        LegacyWidgetAlias::Reject,
        &["ac.runtime.text.v1".to_string()],
        &scenario,
        &initials(1),
    )
    .map_err(|ds| {
        ds.iter()
            .map(|d| format!("{}: {}", d.code, d.message))
            .collect::<Vec<_>>()
            .join("; ")
    })
}

fn state_value(compiled: &ComponentCompilation, field: &str) -> Value {
    let def = compiled.description.nodes[0].instance_states[0]
        .iter()
        .find(|s| s.field == field)
        .unwrap_or_else(|| panic!("no state def for `{field}`"))
        .def_id
        .clone();
    execute_component_def(compiled, &def)
        .unwrap_or_else(|e| panic!("state `{field}` executes: {e}"))
}

/// 一次 Tick：`x = x + vx * dt` 真的落成 f32 值（2.0*0.5）。
#[test]
fn p981_f32_field_integration_updates_the_state_under_tick() {
    let compiled = compile(MOVER, 1).expect("f32 AGE source compiles");
    assert_eq!(state_value(&compiled, "x"), Value::F32(1.0));
    // 未参与运算的字段保持源内初值——模型字段确实按 f32 持有。
    assert_eq!(state_value(&compiled, "vx"), Value::F32(2.0));
    assert_eq!(state_value(&compiled, "dt"), Value::F32(0.5));
}

/// 两次 Tick 累加：同一状态函数重复重放不重置（1.0 → 2.0）。
#[test]
fn p981_f32_integration_accumulates_across_ticks() {
    let compiled = compile(MOVER, 2).expect("two ticks compile");
    assert_eq!(state_value(&compiled, "x"), Value::F32(2.0));
}

/// 零 Tick：初值 `0.0` 本身是 f32（不是被当成 int 的 0）。
#[test]
fn p981_f32_initial_is_held_as_f32() {
    let compiled = compile(MOVER, 0).expect("no-op scenario compiles");
    assert_eq!(state_value(&compiled, "x"), Value::F32(0.0));
}

/// 全表：add/sub/mul/div/rem 与 lt/le/gt/ge/eq/ne，外加一元 `-`——一次 Tick
/// 内全部落值（任一算符缺席都会在 Atom 绑定期具名失败，不会静默留 0）。
#[test]
fn p981_f32_all_same_type_operators_execute() {
    let compiled = compile(OPS, 1).expect("f32 operator surface compiles");
    assert_eq!(state_value(&compiled, "sum"), Value::F32(11.0));
    assert_eq!(state_value(&compiled, "dif"), Value::F32(7.0));
    assert_eq!(state_value(&compiled, "mul"), Value::F32(18.0));
    assert_eq!(state_value(&compiled, "div"), Value::F32(4.5));
    assert_eq!(state_value(&compiled, "rem"), Value::F32(1.0));
    assert_eq!(state_value(&compiled, "neg"), Value::F32(-2.0));
    assert_eq!(state_value(&compiled, "eq"), Value::Bool(false));
    assert_eq!(state_value(&compiled, "ne"), Value::Bool(true));
    assert_eq!(state_value(&compiled, "lt"), Value::Bool(false));
    assert_eq!(state_value(&compiled, "le"), Value::Bool(false));
    assert_eq!(state_value(&compiled, "gt"), Value::Bool(true));
    assert_eq!(state_value(&compiled, "ge"), Value::Bool(true));
}

/// 混宽运算具名拒（不做隐式提升）：f32 字段 + int 字面量。
#[test]
fn p981_f32_mixed_width_arithmetic_is_named_rejected() {
    let src = MOVER.replace("x = x + vx * dt", "x = x + 1");
    let err = compile(&src, 1).expect_err("mixed f32/int must not silently promote");
    assert!(
        err.contains("type-mismatch") || err.contains("same-type numeric"),
        "named mixed-width rejection: {err}"
    );
}
