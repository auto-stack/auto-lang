//! PLAN-657: L1 组装样板（047-bp-admin）回归锚。
//!
//! - t01 VM 轨：047 app.at 全量（四包五变体 `use bps.*` 直连 + back.api
//!   mock 数据面）过解释器——view 结构断言四区域关键文案（壳 brand/nav
//!   标签/表格 chrome/种子行名）。
//! - t02 vue 轨：app.at SFC 发射——四包 import、on_* 回调绑定、零副本
//!   负断言（app 产物无 bp 组件源码拷贝/私有文案）。
//! - t03 语料面：五个被消费 reference（参数化修整后）单独生成 SFC 绿
//!   + 全包 palette 零漂移不回归 + 047 dep 探测（635 声明门控链路）。
//!
//! VM 轨断言依赖 `ui-iced`——与 plan649 bp tests 同款门控。
#![cfg(feature = "ui-iced")]

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(std::path::Path::to_path_buf)
        .expect("CARGO_MANIFEST_DIR has a repo-root ancestor")
}

/// PLAN-689：047-bp-admin 迁出 examples/ui 升格独立示例 examples/bp-admin，
/// 回归锚随迁新路径（语料本身零改动，历史随行 git mv）。
fn example_047() -> PathBuf {
    repo_root().join("examples/bp-admin")
}

/// Runtime View 收集器（文本面）——plan649 同款。
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
        // header/footer/nav 等语义容器经 convert_container_tracked_ctx 落
        // View::Container（单 child）——壳 brand/菜单文案在此层下。
        View::Container { child, .. } => collect_view_texts(child, out),
        // PLAN-688 r2：nav 列表容器带 overflow-y-auto 落 View::Scrollable，
        // 收集器须穿透（plan649 同款修正）。
        View::Scrollable { child, .. } => collect_view_texts(child, out),
        View::AnchorSlot { child, .. } => collect_view_texts(child, out),
        _ => {}
    }
}

/// AC-01/AC-02 VM 轨：四包直连的完整 app 过解释器。断言三组文案：
/// 壳静态面（brand/菜单）、nav 标签（Init 经 back.api mock 播种）、
/// 主区 data 视图的表格 chrome 与种子行名（default active_nav = "data"）。
#[test]
fn t01_vm_track_full_assembly_renders_four_regions() {
    let app = example_047().join("src/front/app.at");
    let src = std::fs::read_to_string(&app).unwrap();
    let mut dc = crate::build_dynamic_component(&src, Some(&app.to_string_lossy()))
        .expect("047 full assembly must compile on the VM track");
    // PLAN-702 段驱动适配：挂载自发 Init 经 back.api mock 播种即 park——
    // 驱动恢复泵至 nav 标签落账（生产 = __parked_resume_tick 泵）。
    crate::plan370_test_support::drive_parked_segments(&mut dc, |dc| {
        let (view, _m, _p) = dc.view_with_debug();
        let mut texts = Vec::new();
        collect_view_texts(&view, &mut texts);
        let joined = texts.join("
");
        ["Data", "Settings", "Reports", "About"].iter().all(|e| joined.contains(e))
    }, "plan657 t01 nav seeding");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    let joined = texts.join("\n");

    // 壳静态面（SidebarShell reference 本体；"Sign out" 在关闭态
    // dropdown-menu 的 content 内——交互门控面，T-05 走查验证，不在此断言）
    for expected in ["Acme"] {
        assert!(
            joined.contains(expected),
            "shell must carry {expected:?}; snapshot:\n{joined}"
        );
    }
    // nav 标签（Init → nav_count → nav_tree 播种链路）
    for expected in ["Data", "Settings", "Reports", "About"] {
        assert!(
            joined.contains(expected),
            "nav label {expected:?} must render (Init seeding); snapshot:\n{joined}"
        );
    }
    // 主区 data 视图（DataTableCrudDialog with_dialog 变体 chrome + 种子行）
    for expected in ["Search…", "New item", "Analytical Engine"] {
        assert!(
            joined.contains(expected),
            "table region must carry {expected:?}; snapshot:\n{joined}"
        );
    }
}

/// AC-02 vue 轨：app SFC 引用四个 bp 组件工件（非内联拷贝），on_* 回调
/// 绑定齐备；零副本负断言——bp 私有文案/源码不进 app 产物。
#[test]
fn t02_vue_track_imports_and_zero_copy() {
    let app = example_047().join("src/front/app.at");
    let result = crate::ui_gen::generate_component_from_file(
        &app,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("047 app.at must generate SFC");
    crate::drain_store_extra_files();
    let code = &result.vue_code;

    for import in [
        "import SidebarShell from '@/components/SidebarShell.vue'",
        "import DataTableCrudDialog from '@/components/DataTableCrudDialog.vue'",
        "import SettingsScreen from '@/components/SettingsScreen.vue'",
        "import EmptyStateFirstUse from '@/components/EmptyStateFirstUse.vue'",
        "import EmptyStateNoResult from '@/components/EmptyStateNoResult.vue'",
    ] {
        assert!(
            code.contains(import),
            "app SFC must import the bp component; missing {import:?}; code:\n{code}"
        );
    }
    for binding in ["@Nav=", "@SignOut=", "@Query=", "@Create=", "@Update=", "@Delete=", "@Save=", "@Primary="] {
        assert!(
            code.contains(binding),
            "app SFC must wire callback {binding:?}; code:\n{code}"
        );
    }
    // 零副本负断言：bp 内部文案与 widget 声明不落 app 产物（只在自己的 SFC 里）
    assert!(
        !code.contains("App content mounts here"),
        "bp-internal fallback copy must not leak into the app SFC"
    );
    assert!(
        !code.contains("widget SidebarShell"),
        "bp widget source must not be copied into the app SFC"
    );
}

/// AC-04 语料面：五个被消费 reference（参数化修整后）单独生成 SFC 绿；
/// 全包 palette 零漂移不回归（PLAN-649 t06 锚的延续）；047 的 dep 探测
/// 走通（635 声明门控 → 649 连字符变体探测链路）。
#[test]
fn t03_corpus_references_generate_and_palette_zero_drift() {
    let references = [
        "navigation/sidebar-shell/reference/default.at",
        "data-display/data-table-crud/reference/with_dialog.at",
        "form/settings/reference/default.at",
        "feedback/empty-state/reference/first_use.at",
        "feedback/empty-state/reference/no_result.at",
    ];
    for rel in references {
        let path = repo_root().join("blueprints").join(rel);
        let result = crate::ui_gen::generate_component_from_file(
            &path,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .unwrap_or_else(|e| panic!("{rel} must generate SFC after parameterization: {e}"));
        crate::drain_store_extra_files();
        assert!(
            !result.vue_code.trim().is_empty(),
            "{rel} SFC must be non-empty"
        );
    }

    // palette 零漂移（全包扫描——参数化修整不得引入词表外 widget）
    use crate::ui_gen::bp::BlueprintRegistry;
    use crate::ui_gen::widget::WidgetRegistry;
    let reg = BlueprintRegistry::with_defaults();
    let widgets = WidgetRegistry::with_defaults();
    let drift = reg.palette_drift(&widgets);
    assert!(drift.is_empty(), "palette drift after PLAN-657: {drift:?}");

    // 047 dep 探测：pac.at dep bps → kebab 包 use 路径可达（635/649 链路）
    let hit = crate::resolve_module_path(
        &example_047(),
        "bps.navigation.sidebar_shell.reference.default",
    );
    assert!(
        hit.is_some(),
        "047 dep probe must reach the kebab bp package"
    );
}
