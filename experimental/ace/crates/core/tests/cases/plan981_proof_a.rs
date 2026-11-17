//! PLAN-981（B-PROOF-[A]）：MIN-A-AGE 的 2D/3D **真实来源**证明面（A 只 AV）。
//!
//! 985 r26 反馈要求：g01/Showcase 最小真实 UI＋一个 2D/一个 3D 呈现＋一次事件/更新/
//! 关闭，只 AV；独立 CPU 值/可见 ROI；错误编辑保旧、修复恢复。
//!
//! 本文件冻结 **SOURCE/前门 + 呈现事实** 半边（AV 观察腿在 `ace-av` 的
//! `plan981_age_package_av::p981_proof_a_*`，宿主关闭/更新腿在 auto-ui 的 `s2`）：
//! - `g01` 的**原样三模块源**（`fixtures_g01/`，与 SOURCE 批共用，来源与哈希见
//!   `plan981_original_g01.rs`）经共同前门编译，出口 `view-2d`，4 行场景事实的 **CPU 值**
//!   对齐原 G01 基线；
//! - `g08/3d` 原 scene 的**派生 3D 源**（`fixtures_proof/g08_3d_cylinders.at`，保留
//!   cylinder3d 行；期望值取自原 `scene.agscene`）出口 `view-3d`，4 行的位置/缩放对齐原 scene；
//! - 最小**组合**包（UI 事件节点 + g01 的 2D 节点 + g08 的 3D 节点）在一个包内三出口分别成立，
//!   选择按请求返回各自出口，生命周期预算不随出口数量倍增；
//! - 坏编辑具名拒（`yaw` 不适用于 `shape2d`），旧描述仍可用，修复后同一入口恢复。
//!
//! 具名边界（不冒完成）：AV 的 2D/3D 观察格归 B-AV（`D-981-PROOF-AV-SCENE`）；
//! `camera3d`/`character3d`/`player3d`/`mob3d`/`group3d` 与 `solid` 等属性属 B 波格。
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::{
    compile_age_package, select_exit, AgeSceneValue, AgeViewExit, AgeViewRequest,
};
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

const G01_MAIN: &str = include_str!("../fixtures_g01/main.at");
const G01_RESOURCES: &str = include_str!("../fixtures_g01/resources.at");
const G01_SHAPES: &str = include_str!("../fixtures_g01/shapes.at");
const G08_3D: &str = include_str!("../fixtures_proof/g08_3d_cylinders.at");

fn input(id: &str, src: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: src.into(),
        provider: None,
    }
}

/// 组合 MIN-A 包的 UI 节点（一次事件 + model + 一个文本叶）。
const UI: &str = "node Ui {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view {\n        text \"ui\" {}\n    }\n}\n";

const UI_SRC: &str = UI;
const G01_SRC: &str = G01_MAIN;
const G08_SRC: &str = G08_3D;

fn compile_composed() -> ace_core::source::age_package::AgePackageDescription {
    compile_age_package(
        &[
            input("ui", UI_SRC),
            input("main", G01_SRC),
            input("resources", G01_RESOURCES),
            input("shapes", G01_SHAPES),
            input("scene3d", G08_SRC),
        ],
        "app::ui",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the composed MIN-A package (UI + 2D + 3D) compiles")
    .description
}

/// PROOF-I[A]：g01 原源 → `view-2d` 出口 + 4 行场景事实，CPU 值与可见 ROI 对齐原基线。
#[test]
fn p981_proof_a_g01_real_2d_source_keeps_its_baseline_values() {
    let d = compile_age_package(
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
    .description;
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

    // 独立 CPU 值（取自原 G01 基线，不由被测实现推导）。
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
    let roi = |row: &ace_core::source::age_package::AgeSceneRow| -> (f32, f32, f32, f32) {
        match (cpu(row, "position"), cpu(row, "size")) {
            (AgeSceneValue::Floats(p), AgeSceneValue::Floats(s)) => (p[0], p[1], s[0], s[1]),
            _ => panic!("row {} is not a positioned quad", row.name),
        }
    };
    assert_eq!(roi(&c.scene[3]), (-45.0, -25.0, 12.0, 12.0));
}

/// PROOF-I[A]：g08/3d 派生源 → `view-3d` 出口，4 行的位置/缩放对齐原 `scene.agscene`
/// （objects 2..5：ground (0,-0.5,0) scale (10,0.5,12)；wall (6,1.5,0) scale (0.1,1.5,12)；
/// east (4,1,-4)；west (-4,1,4)）。
#[test]
fn p981_proof_a_g08_derived_3d_source_keeps_its_baseline_values() {
    let d = compile_age_package(
        &[input("main", G08_3D)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("the derived g08 3D source compiles")
    .description;
    let c = &d.components[0];
    assert_eq!(c.exits, vec![AgeViewExit::View3d]);
    assert_eq!(
        c.scene.iter().map(|s| s.name.as_str()).collect::<Vec<_>>(),
        vec!["ground", "wall", "east", "west"]
    );
    let cpu = |i: usize, key: &str| -> AgeSceneValue {
        c.scene[i]
            .cpu
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("row {} lacks `{key}`", c.scene[i].name))
    };
    assert_eq!(
        cpu(0, "position"),
        AgeSceneValue::Floats(vec![0.0, -0.5, 0.0])
    );
    assert_eq!(
        cpu(0, "scale"),
        AgeSceneValue::Floats(vec![10.0, 0.5, 12.0])
    );
    assert_eq!(
        cpu(1, "position"),
        AgeSceneValue::Floats(vec![6.0, 1.5, 0.0])
    );
    assert_eq!(cpu(1, "scale"), AgeSceneValue::Floats(vec![0.1, 1.5, 12.0]));
    assert_eq!(
        cpu(2, "position"),
        AgeSceneValue::Floats(vec![4.0, 1.0, -4.0])
    );
    assert_eq!(
        cpu(3, "position"),
        AgeSceneValue::Floats(vec![-4.0, 1.0, 4.0])
    );
}

/// PROOF-V[A]（组合）：一个包内 UI/2D/3D 三个节点各持其出口，选择按请求返回；
/// 生命周期预算与出口数量无关（1 出口 vs 3 出口同 Tick/Frame）。
#[test]
fn p981_proof_a_mixed_ui_2d_3d_exits_and_budget() {
    let d = compile_composed();
    let names: Vec<&str> = d.components.iter().map(|c| c.name.as_str()).collect();
    for want in ["Ui", "Main", "Scene3d"] {
        assert!(
            names.contains(&want),
            "composed package lacks `{want}`: {names:?}"
        );
    }
    let by_name = |n: &str| d.components.iter().find(|c| c.name == n).unwrap();
    let ui = by_name("Ui");
    let two = by_name("Main");
    let three = by_name("Scene3d");
    assert_eq!(ui.exits, vec![AgeViewExit::View]);
    assert_eq!(two.exits, vec![AgeViewExit::View2d]);
    assert_eq!(three.exits, vec![AgeViewExit::View3d]);

    // 各自请求只命中自己的出口（域不跨）。
    let req = AgeViewRequest::Ui {
        width: 800,
        height: 600,
        widget_host: false,
    };
    assert_eq!(select_exit(&ui.exits, req).unwrap(), AgeViewExit::View);
    assert!(select_exit(&two.exits, req).is_err());
    assert_eq!(
        select_exit(&two.exits, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    assert_eq!(
        select_exit(&three.exits, AgeViewRequest::Scene3d).unwrap(),
        AgeViewExit::View3d
    );
    assert!(select_exit(&three.exits, AgeViewRequest::Scene2d).is_err());

    // 一次事件：UI 节点的 unit 消息与 handler 事实可读。
    assert_eq!(ui.handler_msgs, vec!["Inc".to_string()]);
    // 生命周期预算不随出口数量/呈现面倍增（1 个出口的 UI 与 3 出口组合同预算）。
    let one = ace_core::source::age_package::lifecycle_budget(5, 3, ui.exits.len()).unwrap();
    let all = ace_core::source::age_package::lifecycle_budget(5, 3, 3).unwrap();
    assert_eq!((one.starts, one.ticks, one.frames), (1, 5, 3));
    assert_eq!((all.starts, all.ticks, all.frames), (1, 5, 3));
}

/// PROOF-I/V[A]：坏编辑具名拒，旧描述仍可用；修复后同一入口恢复。
#[test]
fn p981_proof_a_bad_edit_keeps_old_and_recovers() {
    let good = compile_composed();
    // `yaw` 不适用于 `shape2d`（plan981_attr_shape 已冻的坏属性形）。
    let bad_main = G01_MAIN.replace(
        "        size: (30.0, 30.0)",
        "        size: (30.0, 30.0)\n        yaw: 1.0",
    );
    assert_ne!(bad_main, G01_MAIN, "the bad edit must change the source");
    let err = compile_age_package(
        &[
            input("ui", UI_SRC),
            input("main", &bad_main),
            input("resources", G01_RESOURCES),
            input("shapes", G01_SHAPES),
            input("scene3d", G08_SRC),
        ],
        "app::ui",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect_err("a malformed 2D row must be rejected");
    assert_eq!(
        err[0].code, "source.age.scene-attr",
        "the bad 2D attribute is named"
    );
    let names: Vec<&str> = good.components.iter().map(|c| c.name.as_str()).collect();
    for want in ["Ui", "Main", "Scene3d"] {
        assert!(names.contains(&want), "old description lost `{want}`");
    }
    assert_eq!(
        select_exit(&good.components[1].exits, AgeViewRequest::Scene2d).unwrap(),
        AgeViewExit::View2d
    );
    // 修复后同一入口恢复。
    let fixed: Vec<String> = compile_composed()
        .components
        .iter()
        .map(|c| c.name.clone())
        .collect();
    assert!(
        fixed.iter().any(|n| n == "Main"),
        "the fixed source recovers: {fixed:?}"
    );
}
