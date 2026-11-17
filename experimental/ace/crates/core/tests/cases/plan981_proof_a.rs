//! PLAN-981（B-PROOF-[A]）：MIN-A-AGE 的 2D/3D **真实来源**证明面（A 只 AV）。
//!
//! 985 r26 反馈要求：g01/Showcase 最小真实 UI＋一个 2D/一个 3D 呈现＋一次事件/更新/
//! 关闭，只 AV；独立 CPU 值/可见 ROI；错误编辑保旧、修复恢复。
//!
//! 本文件冻结 **SOURCE/前门 + 呈现事实** 半边（AV 观察腿在 `ace-av` 的
//! `p981_proof_a_av`，宿主关闭/更新腿在 auto-ui 的 `s2`）：
//! - `g01` 的**原样三模块源**（main/resources/shapes，来源与哈希见下）经共同前门编译，
//!   出口 `view-2d`，4 行场景事实的 **CPU 值**对齐原 G01 基线（独立算术/原值）；
//! - 最小**组合**节点（UI `view` + `view-2d` + `view-3d`）一次选择三出口，事件/生命周期
//!   预算不随出口倍增；
//! - 坏编辑具名拒（旧描述仍可用），修复后同一入口恢复。
//!
//! 夹具来源（age-examples `ea0fc59`）：
//! | fixture | 原路径 | SHA256 前 16 |
//! |---|---|---|
//! | `g01_main.at` | `examples/g01-shapes/g01-main/src/main.at` | `9b809a1c6d99985b` |
//! | `g01_resources.at` | `examples/g01-shapes/g01-main/src/resources.at` | `6f993775477ebc39` |
//! | `g01_shapes.at` | `examples/g01-shapes/g01-main/src/shapes.at` | `dbdc5243ec590dab` |
//!
//! 已知边界（具名，不冒完成）：`g03-3d` 的 `player3d`/`mob3d`/`group3d` 嵌套属 B 波
//! 游戏叶/层级格，不在本 A 面 3D 叶集合内；A 的 3D 呈现用语法内 `cylinder3d` 冻结。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package, select_exit, AgeSceneValue, AgeViewExit, AgeViewRequest,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const G01_MAIN: &str = include_str!("../fixtures_proof/g01_main.at");
const G01_RESOURCES: &str = include_str!("../fixtures_proof/g01_resources.at");
const G01_SHAPES: &str = include_str!("../fixtures_proof/g01_shapes.at");

fn input(id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 组合 MIN-A 节点：UI + 2D + 3D + 一个事件 + model（一次事件/更新/关闭的源）。
const MIX: &str = "node Mix {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"ui\" {}\n    }\n    view-2d {\n        shape2d bar {\n            kind: quad\n            position: (0.0, 0.0)\n            size: (10.0, 4.0)\n        }\n    }\n    view-3d {\n        cylinder3d tower {\n            position: (0.0, 1.0, 2.0)\n        }\n    }\n}\n";

fn compile_g01() -> ace_core::source::age_package::AgePackageDescription {
    compile_age_package(
        &[
            input("main", G01_MAIN),
            input("resources", G01_RESOURCES),
            input("shapes", G01_SHAPES),
        ],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the real g01 three-module source compiles")
    .description
}

fn compile_mix(
    src: &str,
) -> Result<
    ace_core::source::age_package::AgePackageDescription,
    Vec<ace_core::atom_text::Diagnostic>,
> {
    compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .map(|o| o.description)
}

/// PROOF-I[A]：g01 原源 → `view-2d` 出口 + 4 行场景事实，CPU 值对齐原 G01 基线。
#[test]
fn p981_proof_a_g01_real_2d_source_keeps_its_baseline_values() {
    let d = compile_g01();
    let c = &d.components[0];
    assert_eq!(c.name, "Main");
    assert_eq!(c.exits, vec![AgeViewExit::View2d]);
    assert_eq!(
        c.scene
            .iter()
            .map(|s| (s.kind.as_str(), s.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("Disc", "red_disc"),
            ("shape2d", "blue_quad"),
            ("Disc", "green_disc"),
            ("shape2d", "yellow_quad"),
        ]
    );

    // 独立 CPU 值（取自原 G01 基线，不由被测实现推导）：
    // red circle (-20,10) d30 α0.5；blue quad (0,0) 30；green circle (25,-15) d24 α0.6；
    // yellow quad (-45,-25) 12 且 visible=false。
    let cpu = |row: &ace_core::source::age_package::AgeSceneRow, key: &str| -> AgeSceneValue {
        row.cpu
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("row {} lacks `{key}`", row.name))
    };
    assert_eq!(cpu(&c.scene[0], "x"), AgeSceneValue::Float(-20.0));
    assert_eq!(cpu(&c.scene[0], "y"), AgeSceneValue::Float(10.0));
    assert_eq!(cpu(&c.scene[0], "diameter"), AgeSceneValue::Float(30.0));
    assert_eq!(
        cpu(&c.scene[1], "position"),
        AgeSceneValue::Floats(vec![0.0, 0.0])
    );
    assert_eq!(
        cpu(&c.scene[1], "size"),
        AgeSceneValue::Floats(vec![30.0, 30.0])
    );
    assert_eq!(cpu(&c.scene[2], "x"), AgeSceneValue::Float(25.0));
    assert_eq!(cpu(&c.scene[2], "y"), AgeSceneValue::Float(-15.0));
    assert_eq!(cpu(&c.scene[2], "diameter"), AgeSceneValue::Float(24.0));
    assert_eq!(
        cpu(&c.scene[3], "position"),
        AgeSceneValue::Floats(vec![-45.0, -25.0])
    );
    assert_eq!(
        cpu(&c.scene[3], "size"),
        AgeSceneValue::Floats(vec![12.0, 12.0])
    );
    assert_eq!(cpu(&c.scene[3], "visible"), AgeSceneValue::Bool(false));
    // 可见 ROI 的形状（2D 呈现的可见矩形由 position/size 决定，独立算术给出）。
    let roi = |row: &ace_core::source::age_package::AgeSceneRow| -> (f32, f32, f32, f32) {
        match (cpu(row, "position"), cpu(row, "size")) {
            (AgeSceneValue::Floats(p), AgeSceneValue::Floats(s)) => (p[0], p[1], s[0], s[1]),
            _ => panic!("row {} is not a positioned quad", row.name),
        }
    };
    assert_eq!(roi(&c.scene[3]), (-45.0, -25.0, 12.0, 12.0));
}

/// PROOF-V[A]（组合）：UI + 2D + 3D 三出口、一次选择、事件与生命周期预算。
#[test]
fn p981_proof_a_mixed_ui_2d_3d_exits_and_budget() {
    let d = compile_mix(MIX).expect("the composed MIN-A source compiles");
    let c = &d.components[0];
    assert_eq!(
        c.exits,
        vec![AgeViewExit::View, AgeViewExit::View2d, AgeViewExit::View3d]
    );
    let ui = AgeViewRequest::Ui {
        width: 800,
        height: 600,
        widget_host: false,
    };
    assert_eq!(select_exit(&c.exits, ui).unwrap(), AgeViewExit::View);
    assert_eq!(
        select_exit(&c.exits, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    assert_eq!(
        select_exit(&c.exits, AgeViewRequest::Scene3d).unwrap(),
        AgeViewExit::View3d
    );
    // 呈现事实：UI 文本 1 个；2D/3D 各一行，3D 位置来自源。
    assert_eq!(c.texts.len(), 1);
    assert_eq!(
        c.scene
            .iter()
            .map(|s| (s.kind.as_str(), s.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("shape2d", "bar"), ("cylinder3d", "tower")]
    );
    let tower = &c.scene[1];
    assert_eq!(
        tower
            .cpu
            .iter()
            .find(|(k, _)| k == "position")
            .map(|(_, v)| v.clone()),
        Some(AgeSceneValue::Floats(vec![0.0, 1.0, 2.0]))
    );
    // 一次事件：unit 消息 + handler 事实可读。
    assert_eq!(c.handler_msgs, vec!["Inc".to_string()]);
    // 生命周期预算不随出口数量倍增（3 个出口仍是 Start=1/Tick=N/Frame=M）。
    let b = ace_core::source::age_package::lifecycle_budget(5, 3, c.exits.len()).unwrap();
    assert_eq!((b.starts, b.ticks, b.frames), (1, 5, 3));
}

/// PROOF-I/V[A]：坏编辑具名拒，旧描述仍可用；修复后同一入口恢复（无半写状态）。
#[test]
fn p981_proof_a_bad_edit_keeps_old_and_recovers() {
    let good = compile_mix(MIX).expect("good source compiles");
    // `yaw` 不适用于 `shape2d`（plan981_attr_shape 已冻的坏属性形）——具名拒。
    let bad = MIX.replace(
        "size: (10.0, 4.0)",
        "size: (10.0, 4.0)\n            yaw: 1.0",
    );
    let err = compile_mix(&bad).expect_err("a malformed 2D row must be rejected");
    assert_eq!(
        err[0].code, "source.age.scene-attr",
        "the bad 2D attribute is named"
    );
    assert!(
        err[0].code.starts_with("source."),
        "the rejection is named: {:?}",
        err[0]
    );
    // 旧编译结果不受坏编辑影响（同一进程内仍可读、可再选择）。
    assert_eq!(good.components[0].exits.len(), 3);
    assert_eq!(
        select_exit(&good.components[0].exits, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    // 修复后同一入口恢复。
    let fixed = compile_mix(MIX).expect("the fixed source compiles again");
    assert_eq!(fixed.components[0].scene.len(), 2);
}
