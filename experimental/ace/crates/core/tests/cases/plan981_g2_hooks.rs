//! PLAN-981（G2/G1，2026-11-18 用户裁定）：`on` 体 hook-local `let` / `if-else`
//! 与 `shape2d.layer: render-only` 的**语言面**（投影/描述/脱糖判据）。
//!
//! - G2：`age-on-stmt` 双臂（赋值两面共用；hook-local `let`/`if` 仅 age-package
//!   作者面）。AV 真跑（provider 只收一次 sweep、分支复用）见
//!   `av/tests/cases/plan981_gfx_hooks.rs`；本文件钉**投影/描述/拒绝面**。
//! - G1：`layer: render-only` = 可见不碰撞层——typed 行事实进
//!   `AgeSceneRow.layer`（同一份受检行描述交 AV/AC/A2R）；移除 ⇒ `None`
//!   ⇒ 地面重新进碰撞候选（负控）。
//!
//! TS/WEB 复放车道（980）只复放赋值：`handler_stmts` 描述面给出完整语句事实，
//! 消费方见 Let/If 必须具名拒（不部分复放）；age-component 面文法不含这两臂
//! （见本文件的裁剪面用例）。

use ace_core::source::age_component::{
    project_component_candidates, ComponentScenario, ComponentStmt, ScenarioOp,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{compile_age_package, AgeRowLayer};
use ace_core::source::ast::ExprKind;
use ace_core::source::cst::profile_grammar;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};
use ace_core::source::SourceProfile;

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
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
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

fn compile_err(src: &str) -> Vec<ace_core::atom_text::Diagnostic> {
    compile_age_package(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect_err("the source must be rejected")
}

/// 自然写法：一次 sweep + 分支复用同一结果（UI058 T-04 的请求形状）。
const HOOKS: &str = "node Mover {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    if q.start_overlap {\n        x = x + 1.0\n    } else {\n        x = x - 1.0\n    }\n    }\n    view { text \"t\" {} }\n}\n";

/// 完整语句面按**源序**进描述：`Let q = sweep` → `If q.start_overlap {…} else {…}`；
/// 赋值子集（两条分支赋值）继续在 `handler_assigns` 兼容面上。
#[test]
fn p981_hook_let_and_if_reach_the_description_stmts_in_source_order() {
    let out = compile(HOOKS);
    let m = &out.description.components[0];
    assert_eq!(m.handler_msgs, vec!["Life.Tick".to_string()]);
    let (msg, stmts) = &m.handler_stmts[0];
    assert_eq!(msg, "Life.Tick");
    assert_eq!(stmts.len(), 2, "hook let + if: {:?}", stmts);
    // ① hook-local let：初值就是那一次 sweep 调用。
    match &stmts[0] {
        ComponentStmt::Let { name, value, .. } => {
            assert_eq!(name.name, "q");
            match &value.kind {
                ExprKind::MethodCall {
                    recv, name: mname, ..
                } => {
                    assert!(matches!(recv.kind, ExprKind::Var(ref v) if v.name == "query2d"));
                    assert_eq!(mname.name, "sweep");
                }
                other => panic!("hook let value must be the sweep call: {other:?}"),
            }
        }
        other => panic!("first stmt must be the hook let: {other:?}"),
    }
    // ② if：条件读同一局部的端口字段；两个分支各一条赋值。
    match &stmts[1] {
        ComponentStmt::If {
            cond, then, els, ..
        } => {
            assert!(
                matches!(&cond.kind, ExprKind::Field { base, name }
                    if matches!(base.kind, ExprKind::Var(ref v) if v.name == "q")
                        && name.name == "start_overlap"),
                "the branch reuses the one sweep result: {:?}",
                cond.kind
            );
            assert_eq!(then.len(), 1);
            assert_eq!(els.as_ref().expect("else branch").len(), 1);
        }
        other => panic!("second stmt must be the if: {other:?}"),
    }
    // 兼容面：赋值子集两条（`+1.0` / `-1.0`）——旧消费者不破。
    assert_eq!(m.handler_assigns[0].1.len(), 2);
}

/// age-component（980 WEB 复放面）的裁剪**不含** `age-on-let`/`age-on-if`——
/// `let`/`if` 在该面 `on` 体保持语法拒（fail-closed，"能解析、跑不了"的半面
/// 不开）；age-package 裁剪含两臂。语法拒在 `project_component_candidates`
/// 的解析结果上实测（非静默接受）。
#[test]
fn p981_hook_let_and_if_stay_syntax_refused_on_the_component_face() {
    let component = profile_grammar(SourceProfile::AgeComponent).expect("component trim");
    let names: Vec<&str> = component
        .program()
        .rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert!(
        !names.contains(&"age-on-let"),
        "age-component must not grow the hook-let arm"
    );
    assert!(
        !names.contains(&"age-on-if"),
        "age-component must not grow the hook-if arm"
    );
    let package = profile_grammar(SourceProfile::AgePackage).expect("package trim");
    let pnames: Vec<&str> = package
        .program()
        .rules
        .iter()
        .map(|r| r.name.as_str())
        .collect();
    assert!(pnames.contains(&"age-on-let"));
    assert!(pnames.contains(&"age-on-if"));
    // 行为面：同一 hook 源在组件面是解析拒绝（不是投影成功后丢语句）。
    let err = project_component_candidates(HOOKS).expect_err("component face refuses `let`");
    assert!(
        err.iter()
            .all(|d| d.code != "source.age.component-unsupported"
                || !d.message.contains("not wired")),
        "the refusal must be the grammar-level one, got: {:?}",
        err.iter()
            .map(|d| (d.code.as_str(), d.message.clone()))
            .collect::<Vec<_>>()
    );
}

/// 查询 token 走查（021 fault 13 语言侧缝）覆盖新语句形：同一 callback 内第二次
/// `query2d.*` 使先前 hook-local token 失效，此后读旧 token 具名拒。
#[test]
fn p981_second_query_invalidates_the_earlier_hook_token() {
    let ds = compile_err(
        "node M {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let a Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    let b Sweep2D = query2d.sweep(2, true, 0.5, 0.0)\n    if a.start_overlap {\n        x = x + 1.0\n    }\n    }\n    view { text \"t\" {} }\n}\n",
    );
    assert!(
        ds.iter().any(|d| d.code == "source.age.query-token-stale"),
        "the stale read must be named: {:?}",
        ds.iter()
            .map(|d| (d.code.as_str(), d.message.clone()))
            .collect::<Vec<_>>()
    );
}

/// hook-local 名字遮蔽同拼写的 model 字段（裸名先查局部；`.field` 恒指字段），
/// 未遮蔽的字段照常解析——该语义由脱糖唯一承担，不造第二绑定规则。
#[test]
fn p981_hook_local_let_compiles_through_the_single_verify() {
    // `x` 是 model 字段；hook-local `y` 与字段无关，`x + y` 两者各按其绑定。
    let out = compile(
        "node M {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let y f32 = 2.0\n    if y > 1.0 {\n        x = x + y\n    }\n    }\n    view { text \"t\" {} }\n}\n",
    );
    let m = &out.description.components[0];
    assert_eq!(m.handler_stmts[0].1.len(), 2);
    // 无类型注解的 hook-local（共享推断）同样可表达。
    let out2 = compile(
        "node M2 {\n    model { x f32 = 0.0 }\n    on Life.Tick {\n    let v = 1.5\n    if v > 1.0 {\n        x = v\n    }\n    }\n    view { text \"t\" {} }\n}\n",
    );
    assert_eq!(out2.description.components[0].handler_stmts[0].1.len(), 2);
}

// ---------------------------------------------------------------- G1: layer

/// `layer: render-only` 落进**同一份受检行描述**：typed 字段
/// （`AgeSceneRow.layer == Some(RenderOnly)`）与可观察值
/// （`cpu` 面 `Symbol("render-only")`）并存；无声明的行保持 `None`
/// （照常进碰撞候选）。
#[test]
fn p981_shape2d_layer_render_only_reaches_the_typed_row_fact() {
    let out = compile(
        "node Scene {\n    view-2d {\n        shape2d ground {\n            kind: quad\n            position: (0.0, -10.0)\n            size: (100.0, 10.0)\n            layer: render-only\n        }\n        shape2d mover {\n            kind: circle\n            position: (5.0, 5.0)\n            diameter: 12.0\n        }\n    }\n}\n",
    );
    let scene = &out.description.components[0].scene;
    assert_eq!(scene.len(), 2, "declaration order is the draw order");
    let ground = &scene[0];
    let mover = &scene[1];
    assert_eq!(ground.name, "ground");
    assert_eq!(
        ground.layer,
        Some(AgeRowLayer::RenderOnly),
        "the typed row fact must reach the host-facing description"
    );
    let cpu = ground
        .cpu
        .iter()
        .find(|(k, _)| k == "layer")
        .expect("layer is observable on the cpu face");
    assert_eq!(
        cpu.1.as_symbol(),
        Some("render-only"),
        "cpu decodes the frozen symbol"
    );
    assert_eq!(mover.name, "mover");
    assert_eq!(mover.layer, None, "no declaration = collision candidate");
}

/// 负控：**移除** `layer: render-only` ⇒ typed 事实为 `None`——同一份受检行
/// 描述上，"地面不碰撞"的断言随之失效（地面重新进碰撞候选）。
#[test]
fn p981_removing_layer_returns_the_ground_to_collision_candidates() {
    let with = compile(
        "node Scene {\n    view-2d {\n        shape2d ground {\n            kind: quad\n            layer: render-only\n        }\n    }\n}\n",
    );
    assert_eq!(
        with.description.components[0].scene[0].layer,
        Some(AgeRowLayer::RenderOnly)
    );
    let without = compile(
        "node Scene {\n    view-2d {\n        shape2d ground {\n            kind: quad\n        }\n    }\n}\n",
    );
    let row = &without.description.components[0].scene[0];
    assert_eq!(row.layer, None);
    assert!(
        !row.cpu.iter().any(|(k, _)| k == "layer"),
        "the attribute is gone from the observable value face too"
    );
}

/// 冻结值/元素边界：`layer:` 只收 `render-only`（其它值具名拒
/// `source.age.scene-layer`）；3D 叶上的 `layer:` 具名拒（2D 作者面属性）。
#[test]
fn p981_layer_rejects_other_values_and_other_elements() {
    let bad_value = compile_err(
        "node Scene {\n    view-2d {\n        shape2d g {\n            kind: quad\n            layer: circle\n        }\n    }\n}\n",
    );
    assert!(
        bad_value.iter().any(|d| d.code == "source.age.scene-layer"),
        "non-frozen layer value must be named: {:?}",
        bad_value
            .iter()
            .map(|d| (d.code.as_str(), d.message.clone()))
            .collect::<Vec<_>>()
    );
    let on_3d = compile_err(
        "node Scene {\n    view-3d {\n        cylinder3d pillar {\n            layer: render-only\n        }\n    }\n}\n",
    );
    assert!(
        on_3d.iter().any(|d| d.code == "source.age.scene-attr"),
        "layer is 2D-only: {:?}",
        on_3d
            .iter()
            .map(|d| (d.code.as_str(), d.message.clone()))
            .collect::<Vec<_>>()
    );
}

/// 场景重放（脱糖腿）与 G2 组合：场景行 + hook handler 同一包编译通过——
/// 同一份受检描述上 `layer` 事实与 handler 语句面并存（AV 真跑见 av 侧用例）。
#[test]
fn p981_layer_fact_and_hook_stmts_coexist_in_one_checked_description() {
    let out = compile(
        "node S {\n    model { hit int = 0 }\n    view-2d {\n        shape2d ground {\n            kind: quad\n            layer: render-only\n        }\n    }\n    on Life.Tick {\n    let q Sweep2D = query2d.sweep(1, true, 0.5, 0.0)\n    if q.start_overlap {\n        hit = hit + 1\n    }\n    }\n}\n",
    );
    let m = &out.description.components[0];
    assert_eq!(m.scene[0].layer, Some(AgeRowLayer::RenderOnly));
    assert_eq!(m.handler_stmts[0].1.len(), 2);
}

/// 保留场景（ComponentScenario + Tick）也在编译入口可用（AV 用例经
/// `compile_age_package_with_scenario`；这里只钉入口符号在 core 侧可复用）。
#[test]
fn p981_scenario_entry_accepts_hook_handlers() {
    let compiled = ace_core::source::age_package::compile_age_package_with_scenario(
        &[input(HOOKS)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Tick { instance: 0 }],
        },
    )
    .expect("scenario compile accepts the hook handler");
    let m = &compiled.description.components[0];
    assert_eq!(m.handler_stmts[0].1.len(), 2);
}
