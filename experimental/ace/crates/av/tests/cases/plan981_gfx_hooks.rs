//! PLAN-981（G2，2026-11-18 用户裁定）：`on` 体 hook-local `let` + `if/else`
//! 的 **AV 真跑**——`let q = query2d.sweep(…)` 本次执行路径求值**一次**
//! （provider 只收一次 sweep 调用），分支**复用同一结果**（不再查询），
//! 局部结果随回调失效（状态函数作用域承担）。
//!
//! 语言面（投影/描述/拒绝面）见 `core/tests/cases/plan981_g2_hooks.rs`。

use std::cell::RefCell;
use std::rc::Rc;

use ace_av::age_gfx::{AgeGfxFault, AgeGfxHost, AgeGfxReturn, AgeGfxSession};
use ace_av_core::contract::Value;
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: "main".into(),
        source: src.into(),
        provider: None,
    }
}

fn compile(src: &str) -> ace_core::source::age_package::AgePackageCompilation {
    compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        // 场景：1 实例，一次 Life.Tick——状态函数 = 字段 init + handler 重放。
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Tick { instance: 0 }],
        },
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the hook source: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

type CallLog = Rc<RefCell<Vec<(String, Vec<String>)>>>;

/// 记录每次派发的 fake provider；`start_overlap` 可配置（两个分支都要真跑）。
struct FakeHost {
    calls: CallLog,
    start_overlap: bool,
}

fn fmt(v: &Value) -> String {
    match v {
        Value::I32(i) => format!("i32:{i}"),
        Value::F32(f) => format!("f32:{f}"),
        Value::Bool(b) => format!("bool:{b}"),
        Value::Str(_) | Value::StrView(_) => "str".into(),
        other => format!("{other:?}"),
    }
}

impl AgeGfxHost for FakeHost {
    fn dispatch(&mut self, symbol: &str, args: &[Value]) -> Result<AgeGfxReturn, AgeGfxFault> {
        self.calls
            .borrow_mut()
            .push((symbol.to_string(), args.iter().map(fmt).collect()));
        match symbol {
            "age.query2d.sweep.v1" => Ok(AgeGfxReturn::Record(vec![
                // hit, node, t, point_x, point_y, origin_y — start_overlap 是最后
                // 一个字段（AGE_PORT_FIELDS 声明序）。
                Value::Bool(true),
                Value::I32(1),
                Value::F32(0.5),
                Value::F32(0.25),
                Value::F32(4.0),
                Value::F32(5.0),
                Value::F32(0.0),
                Value::Bool(self.start_overlap),
            ])),
            other => Err(AgeGfxFault::new(
                "unknown-op",
                format!("`{other}` is not implemented by this provider"),
            )),
        }
    }
}

fn session(start_overlap: bool) -> (AgeGfxSession, CallLog) {
    let calls: CallLog = Rc::new(RefCell::new(Vec::new()));
    let host = FakeHost {
        calls: calls.clone(),
        start_overlap,
    };
    (AgeGfxSession::new(Box::new(host)), calls)
}

fn run(
    session: &mut AgeGfxSession,
    compiled: &ace_core::source::age_package::AgePackageCompilation,
    name: &str,
) -> Result<Value, String> {
    let symbol = compiled
        .symbol_of(name)
        .unwrap_or_else(|| panic!("no symbol for `{name}`"));
    session.run_def(&compiled.checked, &format!("d_{symbol}"))
}

/// 自然写法（UI058 T-04 请求形状）：一次 sweep + 分支 + 同 Tick 呈现。
const HOOKS: &str = "node Mover {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    if q.start_overlap {\n        x = x + 1.0\n    } else {\n        x = x - 1.0\n    }\n    }\n    view { text \"t\" {} }\n}\n";

/// **provider 只收一次 sweep**；两个分支复用同一结果（不再查询）；分支语义
/// 由同一份结果驱动（overlap=false → else → -1.0；overlap=true → then → +1.0）。
#[test]
fn p981_hook_let_sweep_dispatches_once_and_both_branches_reuse_it() {
    let compiled = compile(HOOKS);

    // else 分支：start_overlap=false → x = 0.0 - 1.0。
    let (mut s, calls) = session(false);
    s.begin_tick();
    let value = run(&mut s, &compiled, "__age_state_0_x").expect("hook body executes on AV");
    assert_eq!(value, Value::F32(-1.0), "else branch: x = x - 1.0");
    assert_eq!(s.credits_used(), 16, "one sweep = 16 credits");
    assert_eq!(
        calls.borrow().len(),
        1,
        "the provider receives exactly ONE sweep — the branch must reuse the result"
    );

    // then 分支：同一源、换 provider 事实（start_overlap=true）→ x = +1.0；
    // 调用数仍为 1（分支复用同一局部，不是第二次查询）。
    let (mut s, calls) = session(true);
    s.begin_tick();
    let value = run(&mut s, &compiled, "__age_state_0_x").expect("then branch executes");
    assert_eq!(value, Value::F32(1.0), "then branch: x = x + 1.0");
    assert_eq!(s.credits_used(), 16);
    assert_eq!(
        calls.borrow().len(),
        1,
        "the then branch also reuses the single sweep result"
    );
}

/// 局部结果随回调失效：hook-local 不进任何跨回调状态面——同源重跑（新会话）
/// 重新求值一次 sweep；字段状态 `x` 按上一次重放留在 init（0.0）——两次重放
/// 互不串局部（作用域由隐藏状态函数承担，无第二求值器）。
#[test]
fn p981_hook_local_result_dies_with_the_callback() {
    let compiled = compile(HOOKS);
    let (mut s, calls) = session(true);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "__age_state_0_x").expect("first replay"),
        Value::F32(1.0)
    );
    // 第二次重放：同一条受检产物，重新求值（provider 再收一次）——局部不是
    // 跨回调记忆化；字段值仍从 init 重放（0.0 起步 → +1.0）。
    assert_eq!(
        run(&mut s, &compiled, "__age_state_0_x").expect("second replay re-evaluates"),
        Value::F32(1.0)
    );
    assert_eq!(
        calls.borrow().len(),
        2,
        "each replay evaluates the sweep once"
    );
    assert_eq!(s.credits_used(), 32, "two replays charge separately");
}

/// 分支内赋值/绑定不外泄：then 里绑定的 hook-local 在分支外不可见——该约束由
/// 作用域承担；这里钉**可观察面**：同一 handler 的字段赋值按分支事实发生。
#[test]
fn p981_branch_bindings_stay_inside_their_branch() {
    let compiled = compile(
        "node M {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    if q.start_overlap {\n        let bonus f32 = 2.0\n        x = x + bonus\n    }\n    }\n    view { text \"t\" {} }\n}\n",
    );
    // then 分支：x = 0.0 + 2.0；else 无（无 else ⇒ 无赋值发生，x 保持 init）。
    let (mut s, calls) = session(true);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "__age_state_0_x").expect("branch-local binding executes"),
        Value::F32(2.0)
    );
    assert_eq!(calls.borrow().len(), 1);
    let (mut s, calls) = session(false);
    s.begin_tick();
    assert_eq!(
        run(&mut s, &compiled, "__age_state_0_x").expect("no else: the assign never runs"),
        Value::F32(0.0)
    );
    assert_eq!(calls.borrow().len(), 1, "the sweep ran; the assign did not");
}

/// G1（顺带同源核对）：同一份受检行描述到达 AV 消费车道——`layer` typed 事实
/// 在 AV 编译产物上同样可读（AV/AC/A2R 同一描述，无车道私有副本）。
#[test]
fn p981_layer_fact_reaches_the_av_lane_description() {
    use ace_core::source::age_package::AgeRowLayer;
    let compiled = compile(
        "node S {\n    model { x f32 = 0.0 }\n    view-2d {\n        shape2d ground {\n            kind: quad\n            layer: render-only\n        }\n    }\n    on Life.Tick {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    if q.start_overlap {\n        x = x + 1.0\n    }\n    }\n}\n",
    );
    let m = &compiled.description.components[0];
    assert_eq!(m.scene[0].name, "ground");
    assert_eq!(
        m.scene[0].layer,
        Some(AgeRowLayer::RenderOnly),
        "the same checked row description reaches the AV lane"
    );
    assert_eq!(m.handler_stmts[0].1.len(), 2);
}
