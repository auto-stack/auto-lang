//! PLAN-809 T-02 V-02/V-03: generation request contract tests. All cases
//! are pure-data (no FS, no network); parse inputs are inline documents via
//! the static scanner (V-03 face: static admission, unsupported honesty).

use crate::descriptor::SourceInfo;
use crate::generation_request::{
    GenerationRequest, RequestMode, RequestOrigin, ThemeDeclLite,
};
use crate::parse_document;
use crate::UnsupportedKind;

fn req_from(text: &str) -> GenerationRequest {
    let descriptor = parse_document(text, SourceInfo::inline()).expect("static parse ok");
    GenerationRequest::from_descriptor(&descriptor)
}

/// A-group projection: display/runtime/scene/members/toggles map through
/// with the same normalizations Pac::new applies.
#[test]
fn t02_v02_static_projection_a_group() {
    let req = req_from(
        r#"
        name: "contact-form"
        version: "1.0.0"
        scene: "ui"
        render: "vue"
        title: "Contact"
        title_zh: "联系表单"
        icon: "form-input"
        category: "tool"
        theme: "light"
        accent: "ocean"
        desktop_render: "queue"
        ui_config: "auto-edit.at"
        media_root: "E:\\Video"
        photo_root: "C:\\pics"
        front_port: 3018
        back_port: 8018
        window: "1440x900"
        api: "rust"
        shadcn: off
        default_classes: on
        members: ["a", "b"]
        back: "../ash-server"
        "#,
    );
    assert_eq!(req.identity.name.as_deref(), Some("contact-form"));
    assert_eq!(req.identity.version.as_deref(), Some("1.0.0"));
    assert_eq!(req.display.title.as_deref(), Some("Contact"));
    assert_eq!(req.display.title_zh.as_deref(), Some("联系表单"));
    assert_eq!(req.display.icon.as_deref(), Some("form-input"));
    assert_eq!(req.display.category.as_deref(), Some("tool"));
    assert_eq!(req.display.theme.as_deref(), Some("light"));
    assert_eq!(req.display.accent.as_deref(), Some("ocean"));
    assert_eq!(req.display.desktop_render.as_deref(), Some("queue"));
    assert_eq!(req.display.ui_config.as_deref(), Some("auto-edit.at"));
    assert_eq!(req.runtime.media_root.as_deref(), Some("E:\\Video"));
    assert_eq!(req.runtime.photo_root.as_deref(), Some("C:\\pics"));
    assert_eq!(req.runtime.front_port, Some(3018));
    assert_eq!(req.runtime.back_port, Some(8018));
    assert_eq!(req.window_size(), Some((1440.0, 900.0)));
    assert!(!req.window_fit());
    assert_eq!(req.runtime.api.as_deref(), Some("rust"));
    assert_eq!(req.members, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(req.external_backend.as_deref(), Some("../ash-server"));
    // scene "ui" → SceneKind::Ui (carried as runtime.scene already; request
    // convenience field mirrors it)
    assert_eq!(req.scene, crate::descriptor::SceneKind::Ui);
    // toggles normalized with the Pac rule set (off → false; on → true)
    assert!(!req.shadcn());
    assert!(req.default_classes());
    // render selection: single vue frontend
    assert_eq!(req.frontend_names(), vec!["vue".to_string()]);
    assert_eq!(req.backend_name(), None);
    assert!(req.is_evaluated(), "fully static document");
}

/// Locale chain mirrors Pac::display_title_in exactly (PLAN-015).
#[test]
fn t02_v02_display_title_locale_chain() {
    let req = req_from("name: \"x\"\ntitle: \"Calc\"\ntitle_zh: \"计算器\"\n");
    assert_eq!(req.display_title(true).as_deref(), Some("计算器"));
    assert_eq!(req.display_title(false).as_deref(), Some("Calc"));

    let req = req_from("name: \"x\"\ntitle: \"Clock\"\n");
    assert_eq!(req.display_title(true).as_deref(), Some("Clock"));
    assert_eq!(req.display_title(false).as_deref(), Some("Clock"));

    // blank title_zh counts as absent; no title → None
    let req = req_from("name: \"x\"\ntitle: \"Paint\"\ntitle_zh: \"  \"\n");
    assert_eq!(req.display_title(true).as_deref(), Some("Paint"));
    let req = req_from("name: \"x\"\n");
    assert_eq!(req.display_title(true), None);
}

/// B-group: package-asset keys / theme block / exe_name / front / sidecar /
/// dep roots are recovered from the raw catalogue without losing the rest.
#[test]
fn t02_v02_static_projection_b_group() {
    let req = req_from(
        r##"
        name: "gallery"
        render: "vue"
        exe_name: "gallery-bin"
        front: "front"
        npm_deps: ["vue", "vite"]
        styles: ["theme.css"]
        i18n: "zh"
        theme: {
            extends: "stella"
            mode: "Auto"
            colors: {
                primary: "#8b5cf6"
            }
        }
        dep "common" {
            path: "../common"
        }
        dep "bare" { }
        rust_sidecar("extra.rs") { }
        mystery_key: 42
        "##,
    );
    assert_eq!(req.exe_name.as_deref(), Some("gallery-bin"));
    assert_eq!(req.front_decl.as_deref(), Some("front"));
    // npm_deps array survives as a static Value
    let npm = req.package_assets.npm_deps.expect("npm_deps static");
    assert_eq!(npm.to_astr().contains("vue"), true);
    assert!(req.package_assets.styles.is_some());
    assert!(req.package_assets.i18n.is_some());
    // theme block → light declaration (mode lowercased like Pac::new)
    let decl = req.theme_decl.expect("theme block recovered");
    assert_eq!(decl.extends.as_deref(), Some("stella"));
    assert_eq!(decl.mode.as_deref(), Some("auto"));
    assert_eq!(decl.colors.len(), 1);
    assert_eq!(decl.colors[0].0, "primary");
    // dep roots: path dep + bare dep
    assert_eq!(req.dep_roots.len(), 2);
    assert_eq!(req.dep_roots[0].name, "common");
    assert_eq!(req.dep_roots[0].path.as_deref(), Some("../common"));
    assert_eq!(req.dep_roots[1].path, None);
    // sidecar presence detected (node form)
    assert!(req.has_rust_sidecar);
    // unknown key preserved verbatim, not swallowed
    assert!(req.raw.iter().any(|p| p.key == "mystery_key"));
    // sidecar node stays on raw_nodes for the adapter
    assert!(req.raw_nodes.iter().any(|n| n.head == "rust_sidecar"));
}

/// Unsupported constructs make the static request honestly non-evaluated
/// (V-03: static admission never fakes evaluation).
#[test]
fn t02_v03_unsupported_not_fake_evaluated() {
    let req = req_from("name: \"x\"\ntitle: \"${computed}\"\n");
    assert!(!req.is_evaluated(), "interpolation is VM-only");
    match &req.origin {
        RequestOrigin::StaticAdmitted { unsupported, .. } => {
            assert!(unsupported.contains(&UnsupportedKind::Interpolation));
        }
        _ => panic!("static origin expected"),
    }
    // the field itself is absent, not a fabricated value
    assert_eq!(req.display.title, None);

    // dynamic origin is final by construction
    let dyn_req = GenerationRequest::dynamic(SourceInfo::inline());
    assert!(dyn_req.is_evaluated());
    assert_eq!(dyn_req.mode, RequestMode::Generate);
}

/// Unknown-but-static render shape stays visible and never fakes a known
/// frontend (RenderSpec::Unknown keeps the raw string).
#[test]
fn t02_v03_unknown_render_not_fake_frontend() {
    let req = req_from("name: \"x\"\nbackend: \"rust-tauri\"\n");
    assert!(req.frontend_names().is_empty(), "unknown backend is not a frontend");
    // legacy raw string preserved on the render decl
    assert_eq!(req.render.legacy_backend.as_deref(), Some("rust-tauri"));
}

/// Path resolution is pure algebra against the project root — cwd-free
/// (V-02: paths resolve against the source root, not the working dir).
#[test]
fn t02_v02_paths_resolve_against_root() {
    let descriptor =
        parse_document("name: \"x\"\n", SourceInfo::from_path("proj/ui/pac.at"))
            .expect("parse ok");
    let req = GenerationRequest::from_descriptor(&descriptor);
    assert_eq!(req.project_root, Some(std::path::PathBuf::from("proj/ui")));
    assert_eq!(
        req.resolve_against_root("../common"),
        Some(std::path::PathBuf::from("proj/ui/../common"))
    );
    // absolute passes through untouched
    assert_eq!(
        req.resolve_against_root("E:/abs"),
        Some(std::path::PathBuf::from("E:/abs"))
    );
    // no root → None (caller decides), never cwd-join
    let rootless = req_from("name: \"x\"\n");
    assert_eq!(rootless.resolve_against_root("rel"), None);
}

/// window "fit" carries the content-fit mode (Plan 504 parity).
#[test]
fn t02_v02_window_fit() {
    let req = req_from("name: \"x\"\nwindow: \"fit\"\n");
    assert!(req.window_fit());
    assert_eq!(req.window_size(), None);
}

/// ThemeDeclLite parity spot-check against the Pac::new extraction shape
/// (empty block is still a declaration; colors filtered on empty values).
#[test]
fn t02_v02_theme_decl_lite_shape() {
    let lite = ThemeDeclLite {
        name: None,
        extends: None,
        mode: None,
        colors: Vec::new(),
    };
    let req = req_from("name: \"x\"\ntheme: { }\n");
    assert_eq!(req.theme_decl, Some(lite), "empty block = declaration with no overrides");
}
