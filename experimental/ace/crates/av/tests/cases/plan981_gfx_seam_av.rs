//! PLAN-981 B-W1（IF-981-GFX-AV-SEAM）：UI058 图形服务的 **AV 车道分类**——
//! HostAlgorithm/HostIdentity 在 AV 观察车道调用点具名拒（fail-closed、
//! 零效果、算法归图形宿主车道）；ValueCtor（frame2d.at）与 material2d 同一
//! Aggregate 执行面（端到端 AV 执行待 UI058 宿主注入 Image2D 身份，边界
//! 见计划 §9 差额表 #4）。
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

fn observe(src: &str) -> Result<ace_av::age_package::ObservedAgePackage, String> {
    let compiled = compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: vec![ScenarioOp::Send {
                instance: 0,
                msg: "Tick".into(),
            }],
        },
    )
    .map_err(|ds| format!("{ds:?}"))?;
    ace_av::age_package::observe_age_package(&compiled, 1)
}

/// HostAlgorithm（query2d.segment 在派发 handler 内）：AV 观察车道具名拒、
/// 零效果——查询算法归 UI058 宿主车道，不伪造值。
#[test]
fn p981_gfx_seam_av_query_lane_refused() {
    const SRC: &str = "node P {\n    msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn probe(r i32) bool {\n    let q Cast2D = query2d.segment(r, true, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n";
    // The helper itself is never called by the scenario — the module still
    // lowers whole-module: the intrinsic call in ANY fn hits the lane gate.
    let err = observe(SRC).expect_err("the host-algorithm lane must name-refuse in AV");
    assert!(
        err.contains("lower.age-gfx-lane") && err.contains("age.query2d.segment.v1"),
        "named lane refusal: {err}"
    );
}

/// HostIdentity（image2d.png）：文件内容身份归宿主计算注入——AV 观察车道
/// 具名拒（同 model3d.glb 先例）。
#[test]
fn p981_gfx_seam_av_image_identity_lane_refused() {
    const SRC: &str = "node M {\n    view {\n        text \"m\" {}\n    }\n}\n\npub fn img_len() i32 {\n    let img Image2D = image2d.png(\"assets/x.png\")\n    return img.byte_length\n}\n";
    let err = observe(SRC).expect_err("the host-identity lane must name-refuse in AV");
    assert!(
        err.contains("lower.age-gfx-lane") && err.contains("age.image2d.png.v1"),
        "named lane refusal: {err}"
    );
}

/// 对照：不含 gfx 宿主服务的同构包照常观察（车道拒不误伤普通面）。
#[test]
fn p981_gfx_seam_av_plain_package_unaffected() {
    const SRC: &str = "node P {\n    msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    let o = observe(SRC).expect("plain packages keep observing");
    assert_eq!(o.components[0].instance_states, vec![vec![1]]);
}
