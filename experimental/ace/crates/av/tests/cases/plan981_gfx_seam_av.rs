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

/// ValueCtor（frame2d.at）record 实参（Image2D 经借用视图到达）：D-981-GLOBAL-CLONE-GAP
/// 关闭后，同名记录视图经**显式 CLONE**（唯一合法视图→拥有跨接指令）深拷贝成
/// owned 槽再进 Aggregate——整个包通过 mir_verify（此前正是在这里具名拒）。
/// 深拷贝独立性的执行级证据在 av-core 的 `av_clone_view_to_owned_deep_copies_resource_record`。
/// 实际执行 frame_w 仍待 UI058 宿主注入 Image2D 身份（image2d.png 保持
/// host-identity 车道拒），本用例钉住"构造器就绪、不再 fail-closed"。
#[test]
fn p981_gfx_seam_av_value_ctor_record_arg_admitted_and_verified() {
    const SRC: &str = "node M {
    model {
        count int = 0
    }
    view {
        text \"m\" {}
    }
}

pub fn frame_w(img Image2D) i32 {
    let f Frame2D = frame2d.at(img, 1, 2, 32, 32)
    return f.w
}
";
    let compiled = compile_age_package_with_scenario(
        &[input(SRC)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect("the value-ctor package compiles");
    // 视图实参经 CLONE 跨接：整包（含 frame_w）通过 mir_verify——
    // 回退 Clone 发射（回到具名拒）或破坏 mir 的视图跨接规则都会红。
    let observed = ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the record-view ctor arg clones across and the module verifies");
    assert_eq!(
        observed.components[0].instance_states,
        vec![vec![0]],
        "no scenario ops ran; count stays at its initial 0 — the ctor merely verifies"
    );
}

/// D-981-GLOBAL-CLONE-GAP 的 GlobalSet 腿：模块 `var`（record 端口）从
/// **视图参数**写入——lowering 发射 Clone{Record, RecordView}，如今合法
/// （此前 mir.clone-type 拒，即债务登记的"不可达分支"）。正例=整包过 verify；
/// 反例=写入**异名** record 端口在前门具名拒（不静默降级）。
#[test]
fn p981_gfx_seam_av_record_global_write_from_view_param() {
    const OK: &str = "node N {
    view {
        text \"t\" {}
    }
}

var g Mesh2D = mesh2d.quad(1.0, 2.0)

pub fn store(x Mesh2D) i32 {
    g = x
    return 0
}
";
    let compiled = compile_age_package_with_scenario(
        &[input(OK)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect("the record-global package compiles");
    ace_av::age_package::observe_age_package(&compiled, 1)
        .expect("the view→owned CLONE on the global write verifies (D-981-GLOBAL-CLONE-GAP closed)");

    // 反例：往 Mesh2D 全局写 Image2D——前门具名拒（类型不符），不静默。
    const BAD: &str = "node N {
    view {
        text \"t\" {}
    }
}

var g Mesh2D = mesh2d.quad(1.0, 2.0)

pub fn store_bad(x Image2D) i32 {
    g = x
    return 0
}
";
    let err = compile_age_package_with_scenario(
        &[input(BAD)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .expect_err("a foreign record port must not enter the Mesh2D global");
    assert!(
        !err.is_empty(),
        "named rejection, not silent acceptance"
    );
}

/// 对照：不含 gfx 宿主服务的同构包照常观察（车道拒不误伤普通面）。
#[test]
fn p981_gfx_seam_av_plain_package_unaffected() {
    const SRC: &str = "node P {\n    msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    let o = observe(SRC).expect("plain packages keep observing");
    assert_eq!(o.components[0].instance_states, vec![vec![1]]);
}
