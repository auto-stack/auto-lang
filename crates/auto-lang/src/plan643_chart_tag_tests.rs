//! PLAN-643 —— chart 裸名归属统一（tag 双态归属）验收测试。
//!
//! 覆盖面:
//! 1. T-06 with_charts 变体:VM 轨 build_dynamic_component（动态分支 package
//!    装载）渲染官方 chart 组件文本面；vue 轨 SFC 生成引用包组件；
//! 2. T-05 palette 包词汇面:dashboard/overview spec palette（含 chart 四
//!    tag）palette_drift 零漂移（正断言与 bp/registry.rs 单测互为镜像）；
//! 3. 测试物化契约:with_charts reference 以消费方约定 `from "components"`
//!    引官方包——bp 目录不携带第 4 份 chart 组件副本（§5.2 选项 A 零物理
//!    搬运），测试用 scratch 目录拷贝物化（禁 symlink，worktree 红线）。

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .expect("CARGO_MANIFEST_DIR has a repo-root ancestor")
}

/// 官方 chart 包源目录——用 charts-gallery 副本（本仓、master 干净可装载；
/// 024-charts 副本待 PLAN-642 语料笔误修复合入，属其辖区）。三副本同构。
fn official_chart_pkg_dir() -> PathBuf {
    repo_root().join("examples/charts-gallery/src/front/components")
}

/// 把官方 chart 包物化到 scratch 目录（plain copy,无链接）,并写入
/// with_charts reference,返回 scratch 下的 reference 路径。
fn materialize_with_charts_fixture() -> PathBuf {
    let pkg_dir = official_chart_pkg_dir();
    let scratch = std::env::temp_dir().join(format!("plan643_with_charts_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(scratch.join("components")).expect("scratch components dir");
    for entry in std::fs::read_dir(&pkg_dir).expect("official package dir") {
        let entry = entry.expect("readdir entry");
        let name = entry.file_name();
        let target = scratch.join("components").join(&name);
        std::fs::copy(entry.path(), &target)
            .unwrap_or_else(|e| panic!("copy {}: {e}", name.to_string_lossy()));
    }
    let reference = repo_root().join("blueprints/dashboard/overview/reference/with_charts.at");
    let dest = scratch.join("with_charts.at");
    std::fs::copy(&reference, &dest).expect("copy reference into scratch");
    dest
}

/// T-06 VM 轨:with_charts 变体经 build_dynamic_component 渲染——官方 chart
/// 组件装载成功（动态分支 package 臂）,图例文本面进入 View 树。
#[test]
fn t01_with_charts_vm_track_renders_charts() {
    let fixture = materialize_with_charts_fixture();
    let src = std::fs::read_to_string(&fixture).expect("fixture readable");
    let dc = crate::build_dynamic_component(&src, Some(&fixture.to_string_lossy()))
        .expect("with_charts reference must compile through the VM pipeline");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    // 官方 LineChart 出图证明：月份轴标签由组件 Init 从 .monthly 数据经
    // nice-ticks/band scale 计算（静态语料不写月份文本，出现即组件装载 +
    // 派生链通的硬证据）。图例 labels 走组件槽位字段（R006 形态），header
    // 走 header 变体——均不在本文本采集器行走面，不作断言。
    for expected in ["Jan", "Jun", "Total users", "Traffic"] {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "with_charts VM view must contain {expected:?}; snapshot: {texts:?}"
        );
    }
    let _ = std::fs::remove_dir_all(
        fixture
            .parent()
            .expect("fixture has scratch parent"),
    );
}

/// T-06 vue 轨:with_charts 变体生成 SFC——chart 组件引用与 bp 文本面进码。
#[test]
fn t02_with_charts_vue_track_generates_sfc() {
    let fixture = materialize_with_charts_fixture();
    let result = crate::ui_gen::generate_component_from_file(
        &fixture,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("with_charts must generate SFC");
    crate::drain_store_extra_files();
    assert!(
        result.vue_code.contains("Overview") && result.vue_code.contains("Traffic"),
        "SFC keeps bp text surface"
    );
    // S004（builtin wins）不得再对 chart 组件出现——package_origin 排除。
    assert!(
        !result
            .validation_warnings
            .iter()
            .any(|w| w.rule == "S004" && w.message.contains("builtin")),
        "no S004 suppression for chart components: {:?}",
        result
            .validation_warnings
            .iter()
            .filter(|w| w.rule == "S004")
            .collect::<Vec<_>>()
    );
    let _ = std::fs::remove_dir_all(
        fixture
            .parent()
            .expect("fixture has scratch parent"),
    );
}

/// T-05 正断言（bp 级）:dashboard/overview palette（含 chart 四 tag）对
/// WidgetRegistry 零漂移——package_origin 词汇面被 palette_drift 接受。
#[test]
fn t03_dashboard_overview_palette_with_charts_has_no_drift() {
    let reg = crate::ui_gen::bp::BlueprintRegistry::with_defaults();
    let pkg = reg.get("dashboard", "overview").expect("overview package");
    assert_eq!(
        pkg.spec.variants,
        vec!["default".to_string(), "with_charts".to_string()],
        "with_charts variant declared"
    );
    for tag in ["line-chart", "bar-chart", "area-chart", "donut-chart"] {
        assert!(
            pkg.spec.palette.iter().any(|p| p == tag),
            "palette declares {tag}"
        );
    }
    let widgets = crate::ui_gen::WidgetRegistry::with_defaults();
    let drift = reg.palette_drift(&widgets);
    assert!(drift.is_empty(), "palette drift: {drift:?}");
}

/// Runtime View collector: text face（plan640 同形）。
fn collect_view_texts(
    view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>,
    out: &mut Vec<String>,
) {
    use crate::ui::view::View;
    match view {
        View::Text { content, .. } => out.push(content.clone()),
        View::Button { label, content, .. } => {
            out.push(label.clone());
            if let Some(c) = content {
                collect_view_texts(c, out);
            }
        }
        View::Input { placeholder, value, .. } => {
            out.push(placeholder.clone());
            out.push(value.clone());
        }
        View::Row { children, .. } | View::Column { children, .. } => {
            for c in children {
                collect_view_texts(c, out);
            }
        }
        View::AnchorSlot { child, .. } => collect_view_texts(child, out),
        _ => {}
    }
}
