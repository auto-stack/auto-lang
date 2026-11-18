//! PLAN-981（view-if 第二片，2026-11-19 用户排程）：主线（980 `d69f29f91`
//! view-if 刀）之后的**981 侧缺口收口**。
//!
//! 主线设计（保持不动）：两支文本叶**平铺进** `view.texts`——绑定验证同覆盖，
//! 编译保持通过。本文件钉 981 侧三件仍缺的验收：
//!
//! 1. **states-only 可用**：`compile_component_scenario_states`（SFC/桌面邻接
//!    车道）对含条件 view 的源照常编译——不消费观察面，不受 fail-closed 影响。
//! 2. **描述面携带条件事实**：`ComponentViewOut.cond_spans` / 包描述
//!    `AgePackageComponent.cond_spans`（树序全量，含 else-if 链每层与命名
//!    view）——观察入口具名拒的判据与锚点。
//! 3. **诊断锚定作者 `if`**：`cond_spans[0]` 恰为作者最前 `if` 关键字字节偏移
//!    （av 观察入口的拒收消息携带同一偏移，见 av/tests `plan981_view_if_av`）。
//!
//! 边界：文本**观察**入口（静态/场景/包）的具名拒在 av crate 用例；编译与
//! 绑定检查在本面必须保持通过（分支叶平铺是主线设计，本片不移除）。

use ace_core::source::age_component::{
    compile_component_scenario, compile_component_scenario_states, project_component_candidates,
    ComponentInitials, ComponentScenario,
};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

fn states(src: &str) -> Result<(), Vec<ace_core::atom_text::Diagnostic>> {
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let initials = ComponentInitials {
        per_instance: vec![Vec::new()],
    };
    compile_component_scenario_states(
        src,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
        &initials,
    )
    .map(|_| ())
}

/// states-only 车道（SFC/桌面邻接）对含条件 view 的源照常编译。
#[test]
fn p981_states_lane_compiles_conditional_view() {
    let src = "node N {\n    model {\n        count int = 0\n        open bool = true\n    }\n    view {\n        if .open {\n            text \"on\" {}\n        } else {\n            text \"off\" {}\n        }\n    }\n}\n";
    states(src).expect("states-only lane compiles conditional views (no observation surface)");
}

/// 描述面条件事实：else-if 链=树序两层 span；首个恰为作者最前 `if` 偏移；
/// 元素内嵌套与命名 view 内的条件都入表。
#[test]
fn p981_description_cond_spans_anchor_at_author_if() {
    let src = "node N {\n    model {\n        open bool = true\n        mode int = 0\n    }\n    view {\n        if .open {\n            text \"a\"\n        } else if .mode == 1 {\n            text \"b\"\n        } else {\n            text \"c\"\n        }\n    }\n    view panel {\n        col {\n            if .open {\n                text \"d\"\n            }\n        }\n    }\n}\n";
    // states-only 编译可用性由上一例钉；这里编译取描述面事实。
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let initials = ComponentInitials {
        per_instance: vec![Vec::new()],
    };
    let out = compile_component_scenario_states(
        src,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
        &initials,
    )
    .expect("states compile");
    let spans = &out.description.nodes[0].view.cond_spans;
    // 主 view 的 else-if 链（外层+嵌套）+ 命名 view panel 内一层 = 3。
    assert_eq!(spans.len(), 3, "tree-order spans: {spans:?}");
    assert_eq!(
        spans[0].start,
        src.find("if .open").expect("outer if offset"),
        "first span anchors at the author's foremost `if`"
    );
    assert_eq!(
        spans[1].start,
        src.find("if .mode").expect("nested else-if offset"),
        "second span anchors at the nested else-if"
    );
    assert_eq!(
        spans[2].start,
        src.rfind("if .open").expect("named-view if offset"),
        "named-view conditional is carried too"
    );
}

/// 主线平铺设计保持：分支内文本叶的绑定检查仍在——分支叶引用未声明字段在
/// 编译（非观察）处红；换成已声明的 str 字段则同源编译通过（绑定通道真实
/// 工作，不是一刀切拒）。判别：删平铺则 bad 例静默通过。
#[test]
fn p981_conditional_branch_texts_still_enter_binding_checks() {
    let scenario = ComponentScenario {
        instances: 1,
        ops: Vec::new(),
    };
    let bad = "node N {\n    model {\n        count int = 0\n        open bool = true\n        label str = \"x\"\n    }\n    view {\n        if .open {\n            text .missing_field\n        }\n    }\n}\n";
    let ds = compile_component_scenario(
        bad,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect_err("branch leaf binding must be checked at compile");
    assert!(
        ds.iter()
            .any(|d| d.message.contains("missing_field") || d.code.starts_with("source.")),
        "unknown field in a branch text leaf must be a named rejection: {ds:?}"
    );
    let ok = bad.replace("text .missing_field", "text .label");
    compile_component_scenario(
        &ok,
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
        &scenario,
    )
    .expect("declared str field in a branch leaf compiles (binding channel works)");
}

/// 包描述同样携带条件事实（包面文法接受；编译保持通过——观察拒在 av 侧）。
#[test]
fn p981_package_description_carries_cond_spans() {
    let src = "node N {\n    model {\n        open bool = true\n    }\n    view {\n        if .open {\n            text \"on\" {}\n        } else {\n            text \"off\" {}\n        }\n    }\n}\n";
    let out = compile_age_package(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &[PACKAGES_CAPABILITY.to_string()],
    )
    .expect("package compile passes (binding checks keep branch leaves)");
    let spans = &out.description.components[0].cond_spans;
    assert_eq!(spans.len(), 1, "one conditional in the package view");
    assert_eq!(
        spans[0].start,
        src.find("if .open").expect("if offset"),
        "package fact anchors at the author `if`"
    );
}

/// 负控：无条件的 view——描述面 cond_spans 为空（观察入口不误拒的判据面）。
#[test]
fn p981_plain_view_has_no_cond_spans() {
    let nodes =
        project_component_candidates("node N {\n    view {\n        text \"plain\" {}\n    }\n}\n")
            .expect("plain view projects");
    assert!(nodes[0].view.all_cond_spans().is_empty());
}
