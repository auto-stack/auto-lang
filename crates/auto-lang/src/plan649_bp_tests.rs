//! PLAN-649: bp 消费地基。
//!
//! - T-02（SD-01/contract Q5）：解析链连字符变体探测——`use` 点号路径按下划线
//!   书写（`bps.feedback.empty_state.reference.error`），bp 包磁盘为 kebab
//!   （`feedback/empty-state/`）。字面量优先 + 下划线段连字符变体（上限 4），
//!   probe 单测走 tmp fixture（`crate::resolve_module_path`）。
//! - T-05（AC-01）：L1 直连端到端——连字符包经 `use bps.<kind>.<name>...`
//!   直连导入的双轨渲染（VM view + vue SFC）；640 AC-08 的 empty-state
//!   "全黑"对照转绿。
//!
//! VM 轨断言依赖 `ui-iced`——与 plan639/640 bp tests 同款门控。
#![cfg(feature = "ui-iced")]

use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// T-02: probe 单测（tmp fixture）
// ---------------------------------------------------------------------------

fn tmp_base(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("plan649-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    base
}

/// 连字符包经下划线 use 路径可达：`feedback.empty_state.reference.error` →
/// `feedback/empty-state/reference/error.at`（variant 探测命中）。
#[test]
fn t01_hyphen_variant_resolves_kebab_dir() {
    let base = tmp_base("t01");
    let pkg = base.join("feedback/empty-state/reference");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("error.at"), "// fixture\n").unwrap();

    let hit = crate::resolve_module_path(&base, "feedback.empty_state.reference.error");
    assert!(
        hit.is_some(),
        "underscore use path must reach kebab directory via variant probe"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// 字面量优先：`foo_bar/` 与 `foo-bar/` 并存时字面量 `foo_bar` 胜出，
/// 变体不改变既有解析结果。
#[test]
fn t02_literal_form_wins_over_variant() {
    let base = tmp_base("t02");
    for dir in ["foo_bar", "foo-bar"] {
        std::fs::create_dir_all(base.join(dir)).unwrap();
        std::fs::write(base.join(dir).join("mod.at"), "// fixture\n").unwrap();
    }

    let hit = crate::resolve_module_path(&base, "foo_bar")
        .expect("literal directory must resolve");
    assert!(
        hit.parent().is_some_and(|p| p.ends_with("foo_bar")),
        "literal form must win when both spellings exist; got {hit:?}"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// 无下划线路径行为不变：只有字面量候选，无变体探测（零扰动面）。
#[test]
fn t03_no_underscore_path_literal_only() {
    let base = tmp_base("t03");
    let kebab = base.join("plain-name");
    std::fs::create_dir_all(&kebab).unwrap();
    std::fs::write(kebab.join("mod.at"), "// fixture\n").unwrap();

    let plain = crate::resolve_module_path(&base, "plainname");
    assert!(
        plain.is_none(),
        "non-underscore module must not probe hyphen variants; got {plain:?}"
    );
    let literal = crate::resolve_module_path(&base, "plain_name");
    assert!(
        literal.is_some(),
        "underscore module must reach kebab dir; got {literal:?}"
    );
    assert!(
        literal.unwrap().parent().is_some_and(|p| p.ends_with("plain-name")),
        "resolved mod.at must live in the kebab dir"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// mod.at 形态同样走变体：`feedback.empty_state.reference.default` →
/// `feedback/empty-state/reference/default/mod.at`（目录式变体）。
#[test]
fn t04_variant_reaches_mod_at_form() {
    let base = tmp_base("t04");
    let dir = base.join("feedback/empty-state/reference/default");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("mod.at"), "// fixture\n").unwrap();

    let hit = crate::resolve_module_path(&base, "feedback.empty_state.reference.default");
    assert!(
        hit.as_ref().is_some_and(|p| p.ends_with("mod.at")),
        "variant probe must reach the mod.at form; got {hit:?}"
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// 依赖链同样受益：pac.at `dep bps { path: ... }` + dep 包内 kebab 目录 ——
/// probe_pkg 候选序列走变体（046-bp-import 形态的 tmp 最小复刻）。
#[test]
fn t05_dep_probe_pkg_reaches_kebab_dir() {
    let base = tmp_base("t05");
    let app = base.join("app");
    std::fs::create_dir_all(&app).unwrap();
    std::fs::write(
        app.join("pac.at"),
        format!("dep bps {{\n    path: \"{}\"\n}}\n", base.join("blueprints").display()),
    )
    .unwrap();
    let pkg = base.join("blueprints/feedback/empty-state/reference");
    std::fs::create_dir_all(&pkg).unwrap();
    std::fs::write(pkg.join("error.at"), "// fixture\n").unwrap();

    let hit = crate::resolve_module_path(&app, "bps.feedback.empty_state.reference.error");
    assert!(
        hit.is_some(),
        "dep probe_pkg must reach kebab bp dirs via variant probe"
    );
    let _ = std::fs::remove_dir_all(&base);
}

// ---------------------------------------------------------------------------
// T-03: icon 词汇面（SD-02）——filetree palette 恢复正/负
// ---------------------------------------------------------------------------

/// filetree palette 恢复 `["icon", "text"]` 零漂移（正）：icon 经 PLAN-649
/// 注册进 WidgetRegistry（schema `builtin_widget` 首个入 palette 案例），
/// 全包扫描含 filetree 在内零漂移；词表外未知名仍拒（负）。
#[test]
fn t06_filetree_palette_restored_zero_drift() {
    use crate::ui_gen::bp::BlueprintRegistry;
    use crate::ui_gen::widget::WidgetRegistry;

    let reg = BlueprintRegistry::with_defaults();
    let widgets = WidgetRegistry::with_defaults();

    // 注册面：icon 进 WidgetRegistry（canonical 小写键），双端原生（import None）。
    let icon = widgets.get("icon").expect("icon must be registered (PLAN-649)");
    assert_eq!(icon.name, "Icon");
    assert!(
        icon.backend("vue").is_some() && icon.backend("iced").is_some(),
        "icon spec must carry vue+iced native mappings"
    );

    // filetree palette 恢复终值 + 全包零漂移（正）。
    let filetree = reg
        .get("navigation", "filetree")
        .expect("filetree scanned from blueprints root");
    assert_eq!(
        filetree.spec.palette,
        vec!["icon".to_string(), "text".to_string()],
        "filetree palette must be the restored minimal set"
    );
    let drift = reg.palette_drift(&widgets);
    assert!(drift.is_empty(), "palette drift after restore: {drift:?}");

    // 负：词表外未知名仍拒（icon 注册不放宽合法集边界）——tmp 扫描一个
    // palette 带未知名的最小包。
    let neg_base = tmp_base("t06-neg");
    let pkg_dir = neg_base.join("dashboard/unknown-consumer");
    std::fs::create_dir_all(pkg_dir.join("reference")).unwrap();
    std::fs::write(
        pkg_dir.join("spec.md"),
        "+++\nkind = \"dashboard\"\nname = \"unknown-consumer\"\npalette = [\"not-a-widget\"]\nextension_points = []\nvariants = [\"default\"]\n+++\n\n# Unknown consumer\n",
    )
    .unwrap();
    std::fs::write(
        pkg_dir.join("reference").join("default.at"),
        "widget UnknownConsumer {\n    view {\n        text \"t\"\n    }\n}\n",
    )
    .unwrap();
    let reg2 = BlueprintRegistry::scan_dir(&neg_base).expect("tmp package must scan");
    let drift2 = reg2.palette_drift(&widgets);
    assert!(
        drift2.iter().any(|d| d.contains("not-a-widget")),
        "unknown palette tag must still drift; got {drift2:?}"
    );
    let _ = std::fs::remove_dir_all(&neg_base);
}

// ---------------------------------------------------------------------------
// T-04: data-table 复核回归锁（640-D89 残余改写的事实依据）
// ---------------------------------------------------------------------------

/// data-table 词汇面现状锁定：WidgetRegistry 注册（alias `data-table`）在案，
/// 且 vue 映射经 schema `datatable` 元素（折叠键）+ P4-4 overlay 灌进 spec
/// ——"未入 WidgetRegistry / 缺 vue 映射"两说皆不成立，债行按事实改写
/// （回避维持的真实理由 = 语料零消费）。
#[test]
fn t07_data_table_vocabulary_face_locked() {
    use crate::ui_gen::widget::WidgetRegistry;

    let widgets = WidgetRegistry::with_defaults();
    let spec = widgets.get("data-table").expect("data-table alias registered");
    assert_eq!(spec.name, "DataTable");

    // P4-4 overlay 活链路：schema `datatable` 的 vue 映射在 spec 上。
    assert_eq!(
        widgets.get_primary_component("vue", "data-table"),
        Some("DataTable".to_string()),
        "schema datatable element must overlay the vue mapping onto the DataTable spec"
    );
    assert!(
        widgets.is_backend_supported("vue", "data-table"),
        "data-table must be vue-backend supported via the overlay"
    );
}

// ---------------------------------------------------------------------------
// T-05: L1 直连端到端（AC-01）——kebab 包 `use bps.<kind>.<name>...` 直连导入
// 双轨渲染；640 AC-08 走查"empty-state 全黑"对照转绿。
// ---------------------------------------------------------------------------

/// tmp 宿主 fixture：pac.at 声明 `dep bps`（指向本仓 blueprints/）+ app.at
/// 直连 `use bps.<dotted>: <Widget>` + 视图实例化。返回 app.at 路径。
/// PLAN-657 起 empty-state first_use 等被消费变体带必填回调参数——`props`
/// 注入实例化面（如 `(on_primary: .Primary)`）；老的无参形态传 ""。
fn e2e_host(tag: &str, dotted: &str, widget: &str, props: &str) -> PathBuf {
    let base = tmp_base(tag);
    let front = base.join("src/front");
    std::fs::create_dir_all(&front).unwrap();
    std::fs::write(
        base.join("pac.at"),
        format!(
            "dep bps {{\n    path: \"{}\"\n}}\n",
            repo_root().join("blueprints").display()
        ),
    )
    .unwrap();
    let app = front.join("app.at");
    std::fs::write(
        &app,
        format!(
            "use {dotted}: {widget}\n\nwidget App {{\n    model {{\n        var nav List = [{{ id: \"h\", label: \"Home\", badge: \"\", icon: \"⌂\" }}]\n    }}\n    msg {{ Primary, NavSel }}\n    view {{\n        col {{\n            {widget}{props} {{}}\n            style: \"min-h-screen p-6\"\n        }}\n    }}\n}}\n"
        ),
    )
    .unwrap();
    app
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .expect("CARGO_MANIFEST_DIR has a repo-root ancestor")
}

/// Runtime View 收集器：文本面（Text/Button/Input/容器）——与 plan639/640
/// bp tests 同款。
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
        // PLAN-657：语义容器（header/footer/nav）落 View::Container——壳
        // brand/user menu 文案在此层下（此前收集器不遍历，断言面未覆盖）。
        View::Container { child, .. } => collect_view_texts(child, out),
        View::AnchorSlot { child, .. } => collect_view_texts(child, out),
        _ => {}
    }
}

/// AC-01 VM 轨：empty-state（连字符包）直连导入后，bp 文案进入运行时
/// View 树——640 AC-08 走查的"全黑"对照转绿。判定逻辑同 plan639 t04：
/// 解析失败时 `EmptyStateFirstUse {}` 折叠为未知 tag 占位文本。
#[test]
fn t08_e2e_vm_track_empty_state_direct_import() {
    let app = e2e_host(
        "t08",
        "bps.feedback.empty_state.reference.first_use",
        "EmptyStateFirstUse",
        " (illustration: \"\", on_primary: .Primary)",
    );
    let src = std::fs::read_to_string(&app).unwrap();
    let dc = crate::build_dynamic_component(&src, Some(&app.to_string_lossy()))
        .expect("hyphen-package direct import must compile");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    assert!(
        !texts.iter().any(|t| t.contains("<EmptyStateFirstUse")),
        "bp must resolve to a real child widget, not the unknown-tag placeholder; texts: {texts:?}"
    );
    for expected in ["Nothing here yet", "Create your first item"] {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "VM view must contain bp text {expected:?}; snapshot: {texts:?}"
        );
    }
    let _ = std::fs::remove_dir_all(app.parent().unwrap().parent().unwrap().parent().unwrap());
}

/// AC-01 VM 轨：sidebar-shell / data-table-crud（连字符包）直连导入双样本
/// + form/signup（无连字符包）不回归。
#[test]
fn t09_e2e_vm_track_more_kebab_packages_and_no_regression() {
    let cases: &[(&str, &str, &str, &[&str])] = &[
        (
            "bps.navigation.sidebar_shell.reference.default",
            "SidebarShell",
            // PLAN-657 参数化后 props 必填；nav 经 .nav 播种（Home 标签），
            // content 区为 slot fallback（占位文案保留——裸形态断言不变义）。
            " (nav_tree: .nav, user_name: \"u\", user_email: \"e\", on_nav: .NavSel, on_sign_out: .Primary)",
            &["Home", "Acme", "App content mounts here"],
        ),
        (
            "bps.data_display.data_table_crud.reference.minimal",
            "DataTableCrud",
            "",
            &["Search", "No results"],
        ),
        (
            "bps.form.signup.reference.minimal",
            "SignupForm",
            "",
            &["Create account"],
        ),
    ];
    for (i, (dotted, widget, props, expected)) in cases.iter().enumerate() {
        let app = e2e_host(&format!("t09-{i}"), dotted, widget, props);
        let src = std::fs::read_to_string(&app).unwrap();
        let dc = crate::build_dynamic_component(&src, Some(&app.to_string_lossy()))
            .unwrap_or_else(|e| panic!("{dotted} direct import must compile: {e}"));
        let (view, _debug_map, _probe) = dc.view_with_debug();
        let mut texts = Vec::new();
        collect_view_texts(&view, &mut texts);
        for e in *expected {
            assert!(
                texts.iter().any(|t| t.contains(e)),
                "{dotted}: VM view must contain {e:?}; snapshot: {texts:?}"
            );
        }
        let _ = std::fs::remove_dir_all(
            app.parent().unwrap().parent().unwrap().parent().unwrap(),
        );
    }
}

/// AC-01 vue 轨：直连导入的宿主 SFC 引用 bp 组件工件（单文件 generate 形态
/// 与 `auto build` 全量构建一致，plan639 t04 vue 侧同款判定）；bp reference
/// 本体的 SFC 发射面含关键文案。
#[test]
fn t10_e2e_vue_track_direct_import() {
    let app = e2e_host(
        "t10",
        "bps.feedback.empty_state.reference.first_use",
        "EmptyStateFirstUse",
        " (illustration: \"\", on_primary: .Primary)",
    );
    let result = crate::ui_gen::generate_component_from_file(
        &app,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("hyphen-package direct import host must generate SFC");
    crate::drain_store_extra_files();
    assert!(
        result
            .vue_code
            .contains("import EmptyStateFirstUse from '@/components/EmptyStateFirstUse.vue'"),
        "host SFC must import the resolved bp component; code:\n{}",
        result.vue_code
    );
    let _ = std::fs::remove_dir_all(app.parent().unwrap().parent().unwrap().parent().unwrap());

    // bp reference 本体发射面（empty-state first_use 关键文案进 SFC）。
    let reference = repo_root().join("blueprints/feedback/empty-state/reference/first_use.at");
    let bp = crate::ui_gen::generate_component_from_file(
        &reference,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("bp reference must generate SFC");
    crate::drain_store_extra_files();
    assert!(
        bp.vue_code.contains("Create your first item"),
        "bp reference SFC must carry key copy; code:\n{}",
        bp.vue_code
    );
}
