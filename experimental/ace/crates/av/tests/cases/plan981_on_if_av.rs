//! PLAN-981（on-if 组件面刀）：**组件面 on 体 if/let 的 AV 真值**——分支
//! 选择真跑（G2 共享脱糖链 `hook_stmts_lower` 在组件面的首次真执行）。
//!
//! 判别力：错误的实现（如两支都执行/条件恒真/let 绑定丢失）会让分支断言红；
//! 观察走 `observe_component_scenario_states`（纯状态面——不涉 view-if 观察
//! 门，与本刀无关面正交）。

use ace_av::age_component::{
    observe_component_scenario_states, ComponentFieldValue, ComponentInitials, ComponentScenario,
    ScenarioOp,
};
use ace_core::source::age_h0::LegacyWidgetAlias;

fn int_lit(src: &str, anchor: &str, v: i32) -> ace_core::source::ast::Expr {
    let at = src.find(anchor).expect("anchor");
    ace_core::source::ast::Expr {
        span: ace_core::atom_text::Span {
            start: at,
            end: at + anchor.len(),
        },
        kind: ace_core::source::ast::ExprKind::IntLit(v),
    }
}

fn value_row(row: &[(String, ComponentFieldValue)]) -> Vec<ComponentFieldValue> {
    row.iter().map(|(_, v)| v.clone()).collect()
}

fn states_after(
    src: &str,
    ops: Vec<ScenarioOp>,
) -> Result<Vec<Vec<(String, ComponentFieldValue)>>, String> {
    let out = observe_component_scenario_states(
        src,
        "app::main",
        LegacyWidgetAlias::Reject,
        &ComponentScenario { instances: 1, ops },
        &ComponentInitials {
            per_instance: vec![Vec::new()],
        },
    )?;
    Ok(out.per_instance)
}

/// 006/010 语料形（`.Set(v) -> { if v == 1 { … } else { … } }`）：两次派发
/// 各走一支——分支选择真跑，不是两支同放。
#[test]
fn p981_widget_if_else_selects_branch_on_dispatch() {
    let src = "node W {\n    msg { Set(int) }\n    model { var mode int = 0 }\n    on { .Set(v) -> { if v == 1 { .mode = 1 } else { .mode = 0 } } }\n    view { text \"t\" {} }\n}\n";
    let rows = states_after(
        src,
        vec![
            ScenarioOp::SendArgs {
                instance: 0,
                msg: "Set".into(),
                args: vec![int_lit(src, "Set(int)", 1)],
            },
            ScenarioOp::SendArgs {
                instance: 0,
                msg: "Set".into(),
                args: vec![int_lit(src, "Set(int)", 0)],
            },
        ],
    )
    .expect("both dispatches execute");
    // 末态：第二次派发走 else ⇒ mode=0（若两支同放或条件失效则必≠0——
    // 判别：错误实现下 then 的 mode=1 会赢）。
    assert_eq!(
        value_row(&rows[0]),
        vec![ComponentFieldValue::Int(0)],
        "final state must reflect the ELSE branch of the second dispatch"
    );
    // 判别强化：只派发 1 一次 ⇒ mode=1（then 支生效）。
    let rows = states_after(
        src,
        vec![ScenarioOp::SendArgs {
            instance: 0,
            msg: "Set".into(),
            args: vec![int_lit(src, "Set(int)", 1)],
        }],
    )
    .expect("single dispatch executes");
    assert_eq!(
        value_row(&rows[0]),
        vec![ComponentFieldValue::Int(1)],
        "single dispatch must take the THEN branch"
    );
}

/// 006 真语料形：裸 `if`（无 else）——条件假时**不发生**赋值（分支外状态
/// 保持），条件真时发生。
#[test]
fn p981_widget_bare_if_skips_assignment_when_false() {
    // 载荷用 int：`str` 载荷与字面量的 `==`（借用 str vs owned Str）是共享
    // 检查的既有类型边界（实测具名红），与本刀正交、另归其 owner；分支
    // 判别性质（假⇒跳过）与载荷类型无关。
    let src = "node W {\n    msg { SetTheme(int) }\n    model { var dark_mode bool = true }\n    on { .SetTheme(t) -> { if t == 1 { .dark_mode = false } } }\n    view { text \"t\" {} }\n}\n";
    let lit = |v: i32| int_lit(src, "SetTheme(int)", v);
    // 条件假（t=0）：dark_mode 保持 true（赋值未发生）。
    let rows = states_after(
        src,
        vec![ScenarioOp::SendArgs {
            instance: 0,
            msg: "SetTheme".into(),
            args: vec![lit(0)],
        }],
    )
    .expect("false-cond dispatch executes");
    assert_eq!(
        value_row(&rows[0]),
        vec![ComponentFieldValue::Bool(true)],
        "false cond must skip the branch body (state unchanged)"
    );
    // 条件真（t=1）：dark_mode=false。
    let rows = states_after(
        src,
        vec![ScenarioOp::SendArgs {
            instance: 0,
            msg: "SetTheme".into(),
            args: vec![lit(1)],
        }],
    )
    .expect("true-cond dispatch executes");
    assert_eq!(
        value_row(&rows[0]),
        vec![ComponentFieldValue::Bool(false)],
        "true cond must run the branch body"
    );
}

/// hook-local `let`：绑定求值一次、分支内可见（条件+体都用它）；分支外
/// 不可见（不进字段面——core 侧已钉，这里验真值）。
#[test]
fn p981_widget_let_binds_for_the_branch() {
    let src = "node W {\n    msg { Tick }\n    model { var mode int = 0 }\n    on { .Tick -> { let step = 2  if mode < 3 { mode = mode + step } } }\n    view { text \"t\" {} }\n}\n";
    // Tick 1：0<3 ⇒ mode=0+2=2；Tick 2：2<3 ⇒ 4；Tick 3：4≮3 ⇒ 保持 4。
    let rows = states_after(
        src,
        vec![
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
        ],
    )
    .expect("three ticks execute");
    assert_eq!(
        value_row(&rows[0]),
        vec![ComponentFieldValue::Int(4)],
        "let binding drives both cond and body; the third tick's false cond keeps 4"
    );
}
