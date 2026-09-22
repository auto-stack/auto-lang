//! PLAN-640 T-09: Tier-0 官方默认集扩容门禁。
//!
//! - 全包面：13 包扫描干净（name↔目录、变体文件由 `load_package` 强制）+
//!   gotchas 全覆盖 + 9 新包契约五字段非空（props/actions/dataSource/
//!   extension_points/acceptance——"六问可答"示范集，AC-02）；
//! - 代表包双轨（AC-03）：signup / data-table-crud / empty-state 的
//!   reference `.at` 分别经 VM 管道（`build_dynamic_component` → view 结构）
//!   与 vue 轨（`generate_component_from_file` → SFC 发射面）断言关键文案。
//!
//! VM 轨断言依赖 `ui-iced`——与 plan639_bp_tests 同款门控。
#![cfg(feature = "ui-iced")]

use crate::ui_gen::bp::BlueprintRegistry;
use std::path::{Path, PathBuf};

/// The 9 packages added by PLAN-640 (Tier-0 expansion).
const PLAN640_PACKAGES: &[(&str, &str)] = &[
    ("dashboard", "overview"),
    ("data-display", "data-table-crud"),
    ("data-display", "master-detail"),
    ("feedback", "empty-state"),
    ("feedback", "result-page"),
    ("form", "settings"),
    ("form", "signup"),
    ("form", "wizard"),
    ("navigation", "sidebar-shell"),
];

/// All 13 official Tier-0 catalog keys.
const TIER0_KEYS: &[&str] = &[
    "dashboard/overview",
    "data-display/data-table-crud",
    "data-display/master-detail",
    "data-display/note-list",
    "editor/note-editor",
    "feedback/empty-state",
    "feedback/result-page",
    "form/login",
    "form/settings",
    "form/signup",
    "form/wizard",
    "navigation/sidebar-nav",
    "navigation/sidebar-shell",
];

fn reference_path(kind: &str, name: &str, variant: &str) -> PathBuf {
    repo_root().join(format!("blueprints/{kind}/{name}/reference/{variant}.at"))
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .expect("CARGO_MANIFEST_DIR has a repo-root ancestor")
}

/// AC-01/AC-02: every Tier-0 package scans (name↔dir + variant files are
/// enforced by `load_package`), ships gotchas, and the 9 PLAN-640 packages
/// declare the full contract surface (props/actions/dataSource/
/// extension_points/acceptance non-empty).
#[test]
fn t01_tier0_catalog_scans_with_full_contracts() {
    let reg = BlueprintRegistry::with_defaults();
    for key in TIER0_KEYS {
        let (kind, name) = key.split_once('/').unwrap();
        let pkg = reg
            .get(kind, name)
            .unwrap_or_else(|| panic!("package {key} missing from registry scan"));
        assert!(
            pkg.gotchas.is_some(),
            "{key}: official packages must ship gotchas.md"
        );
    }
    for (kind, name) in PLAN640_PACKAGES {
        let pkg = reg.get(kind, name).unwrap();
        let key = pkg.key();
        assert!(!pkg.spec.props.is_empty(), "{key}: props must be declared");
        assert!(!pkg.spec.actions.is_empty(), "{key}: actions must be declared");
        assert!(
            !pkg.spec.data_source.is_empty(),
            "{key}: dataSource slots must be declared"
        );
        assert!(
            !pkg.spec.extension_points.is_empty(),
            "{key}: extension_points must be declared"
        );
        assert!(
            !pkg.spec.acceptance.is_empty(),
            "{key}: acceptance checklist must be declared"
        );
    }
}

/// AC-03 (VM track): form/signup minimal reference renders its key surface
/// through the VM pipeline (button label / input placeholder / hint text).
#[test]
fn t02_vm_track_signup_reference() {
    let path = reference_path("form", "signup", "minimal");
    let src = std::fs::read_to_string(&path).expect("signup reference exists");
    let dc = crate::build_dynamic_component(&src, Some(&path.to_string_lossy()))
        .expect("signup reference must compile through the VM pipeline");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    for expected in ["Create account", "you@example.com", "At least 8 characters"] {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "signup VM view must contain {expected:?}; snapshot: {texts:?}"
        );
    }
}

/// AC-03 (VM track): data-table-crud minimal reference renders its query
/// chrome (search placeholder + explicit empty branch).
#[test]
fn t03_vm_track_data_table_crud_reference() {
    let path = reference_path("data-display", "data-table-crud", "minimal");
    let src = std::fs::read_to_string(&path).expect("data-table-crud reference exists");
    let dc = crate::build_dynamic_component(&src, Some(&path.to_string_lossy()))
        .expect("data-table-crud reference must compile through the VM pipeline");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    for expected in ["Search", "No results. Adjust your filters."] {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "data-table-crud VM view must contain {expected:?}; snapshot: {texts:?}"
        );
    }
}

/// AC-03 (VM track): empty-state first_use reference renders headline, body,
/// and its primary CTA.
#[test]
fn t04_vm_track_empty_state_reference() {
    let path = reference_path("feedback", "empty-state", "first_use");
    let src = std::fs::read_to_string(&path).expect("empty-state reference exists");
    let dc = crate::build_dynamic_component(&src, Some(&path.to_string_lossy()))
        .expect("empty-state reference must compile through the VM pipeline");
    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);
    for expected in
        ["Nothing here yet", "Create your first item to get started.", "New item"]
    {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "empty-state VM view must contain {expected:?}; snapshot: {texts:?}"
        );
    }
}

/// AC-03 (vue track): the same three references emit SFCs importing the
/// shadcn-vue components they declare in palette (`@/components/ui/*`) and
/// carrying their key copy.
#[test]
fn t05_vue_track_representative_references() {
    let cases: &[((&str, &str, &str), &[&str])] = &[
        (
            ("form", "signup", "minimal"),
            &["@/components/ui/button", "Create account", "you@example.com"],
        ),
        (
            ("data-display", "data-table-crud", "minimal"),
            &["@/components/ui/", "No results. Adjust your filters."],
        ),
        (
            ("feedback", "empty-state", "first_use"),
            &["New item", "Create your first item"],
        ),
    ];
    for ((kind, name, variant), expectations) in cases {
        let path = reference_path(kind, name, variant);
        let result = crate::ui_gen::generate_component_from_file(
            &path,
            crate::ui_gen::ComponentGenOptions::default(),
        )
        .unwrap_or_else(|e| panic!("{kind}/{name}/{variant} must generate SFC: {e}"));
        crate::drain_store_extra_files();
        for expected in *expectations {
            assert!(
                result.vue_code.contains(expected),
                "{kind}/{name}/{variant} SFC must contain {expected:?}; code:\n{}",
                result.vue_code
            );
        }
    }
}

/// Runtime View collector: text face (Text/Button/Input/containers) — same
/// shape as plan639_bp_tests.
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
