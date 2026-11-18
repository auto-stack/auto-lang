//! PLAN-981 B-W1（IF-981-GFX-AV-SEAM）：UI058 图形服务的 **AV 车道分类**——
//! 标量结果的宿主 op 自 `IF-981-GFX-HOST-PORT` 起走注册 provider（见
//! `plan981_gfx_host.rs`）；**返回端口 record** 的 op（query2d/image2d）仍
//! fail-closed：多字段宿主返回 ABI 未接，lowering 具名拒
//! `lower.age-gfx-port-result`（不伪造 record、不拆成 N 次收费调用）。
//! ValueCtor（frame2d.at）与 material2d 同一 Aggregate 执行面。
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

/// 编译包并**执行指定 helper**（无宿主）：宿主 op 命中即具名拒。
fn run_without_host(src: &str, name: &str) -> Result<ace_av_core::contract::Value, String> {
    let compiled = compile_age_package_with_scenario(
        &[input(src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .map_err(|ds| format!("{ds:?}"))?;
    let symbol = compiled
        .symbol_of(name)
        .unwrap_or_else(|| panic!("no symbol for `{name}`"));
    ace_av::age_gfx::execute_without_host(&compiled.checked, &format!("d_{symbol}"))
}

/// 端口结果 op（query2d.segment）现在走**一次宿主调用返回 typed record**；
/// 观察车道没有注册 provider ⇒ 实际执行该 helper 时 `age-gfx.host-unavailable`
/// （fail-closed，不伪造值）。带 provider 的真值在 `plan981_gfx_host.rs`。
#[test]
fn p981_gfx_seam_av_query_lane_refused() {
    const SRC: &str = "node P {\n    view {\n        text \"p\" {}\n    }\n}\n\npub fn probe() bool {\n    let q Cast2D = query2d.segment(1, true, 0.0, 0.0, 1.0, 0.0)\n    return q.hit\n}\n";
    let err = run_without_host(SRC, "probe").expect_err("no provider: the query must refuse");
    assert!(
        err.contains("age-gfx.host-unavailable"),
        "named host-unavailable refusal: {err}"
    );
}

/// HostIdentity（image2d.png）同理：一次宿主调用返回 Image2D 端口；无 provider
/// 时实际执行即 `age-gfx.host-unavailable`（身份不伪造）。
#[test]
fn p981_gfx_seam_av_image_identity_lane_refused() {
    const SRC: &str = "node M {\n    view {\n        text \"m\" {}\n    }\n}\n\npub fn img_len() i32 {\n    let img Image2D = image2d.png(\"assets/x.png\")\n    return img.byte_length\n}\n";
    let err = run_without_host(SRC, "img_len").expect_err("no provider: image2d must refuse");
    assert!(
        err.contains("age-gfx.host-unavailable"),
        "named host-unavailable refusal: {err}"
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
    ace_av::age_package::observe_age_package(&compiled, 1).expect(
        "the view→owned CLONE on the global write verifies (D-981-GLOBAL-CLONE-GAP closed)",
    );

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
    assert!(!err.is_empty(), "named rejection, not silent acceptance");
}

/// 对照：不含 gfx 宿主服务的同构包照常观察（车道拒不误伤普通面）。
#[test]
fn p981_gfx_seam_av_plain_package_unaffected() {
    const SRC: &str = "node P {\n    msg {\n        Tick\n    }\n    on {\n        .Tick -> {\n            .count += 1\n        }\n    }\n    model {\n        count int = 0\n    }\n    view {\n        text \"p\" {}\n    }\n}\n";
    let o = observe(SRC).expect("plain packages keep observing");
    assert_eq!(o.components[0].instance_states, vec![vec![1]]);
}

/// 服务级信用计量（差额表 #6a）：021 原额费率表（segment 8 / sweep 16 / 基价 1）
/// 与每函数 32 credits/callback 预算——第 33 个调用点具名拒（构造性零效果），
/// 32 个的边界照常通过。费率表与 UI058 T-02 联冻，宿主动态计费消费同表。
#[test]
fn p981_gfx_seam_av_service_credits_admission_bound() {
    let mut lets = String::new();
    for i in 0..33 {
        lets.push_str(&format!(
            "    let a{i} Frame2D = frame2d.at(img, {i}, 0, 4, 4)\n"
        ));
    }
    let over = format!(
        "node M {{\n    view {{\n        text \"m\" {{}}\n    }}\n}}\n\npub fn wide(img Image2D) i32 {{\n{lets}    return 0\n}}\n"
    );
    let err = observe(&over).expect_err("the 33rd service call site must name-refuse");
    assert!(
        err.contains("lower.age-gfx-credits") && err.contains("33"),
        "named credit rejection: {err}"
    );

    // 边界：恰好 32 个调用点（32 credits）合法。
    let mut lets32 = String::new();
    for i in 0..32 {
        lets32.push_str(&format!(
            "    let a{i} Frame2D = frame2d.at(img, {i}, 0, 4, 4)\n"
        ));
    }
    let ok = format!(
        "node M {{\n    view {{\n        text \"m\" {{}}\n    }}\n}}\n\npub fn wide(img Image2D) i32 {{\n{lets32}    return 0\n}}\n"
    );
    observe(&ok).expect("exactly 32 credits still verifies");
}

/// 费率表冻结正控（021 原额）：segment 8 / sweep 16 / 基价 1、预算 32——
/// 这是与 UI058 T-02 联冻的基础表，防静默漂移（宿主动态计费消费同表；
/// 降低侧准入界见 admission_bound 用例）。
#[test]
fn p981_gfx_seam_av_credit_rate_table_is_frozen() {
    use ace_core::source::age_resources::{gfx_credit_cost, GFX_CALLBACK_CREDIT_BUDGET};
    assert_eq!(gfx_credit_cost("query2d.segment"), 8);
    assert_eq!(gfx_credit_cost("query2d.sweep"), 16);
    assert_eq!(gfx_credit_cost("frame2d.at"), 1);
    assert_eq!(gfx_credit_cost("image2d.png"), 1);
    assert_eq!(GFX_CALLBACK_CREDIT_BUDGET, 32);
}
