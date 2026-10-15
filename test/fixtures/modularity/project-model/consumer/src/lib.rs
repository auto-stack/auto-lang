//! PLAN-780 V-01 external consumer (AC-01).
//!
//! This crate's manifest is the acceptance evidence: its only dependency is
//! `auto-project-model` (path) — building and running these tests proves the
//! light model parses real pac.at declarations with no auto-lang / auto-man
//! / auto-vm / UI crates in the tree.

use auto_project_model::{
    parse_file, ErrorStage, RenderSpec, SceneKind, ToggleValue, UnsupportedKind, WindowSpec,
};
use std::path::Path;

fn fixture(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("fixtures")
        .join(name)
}

#[test]
fn consumer_parses_static_pac_without_compiler_stack() {
    let d = parse_file(&fixture("static-basic.pac.at")).expect("static parse");
    assert!(d.is_static_equivalent());
    assert_eq!(d.identity.name.as_deref(), Some("fixture-basic"));
    assert_eq!(d.runtime.scene, Some(SceneKind::Ui));
    assert_eq!(d.render.render, RenderSpec::Single("vue".into()));
    // four-name contract surface
    assert_eq!(d.display.title.as_deref(), Some("Fixture Basic"));
    assert_eq!(d.display.title_zh.as_deref(), Some("基础夹具"));

    let ws = parse_file(&fixture("static-workspace-fields.pac.at")).expect("ws parse");
    assert_eq!(ws.runtime.front_port, Some(4046));
    assert_eq!(ws.runtime.window, Some(WindowSpec::Size(1440.0, 900.0)));
    assert_eq!(ws.identity.lang.as_deref(), Some("rust"));
    assert_eq!(
        ws.members,
        Some(vec!["source/front".into(), "source/back".into()])
    );

    let deps = parse_file(&fixture("static-dep-forms.pac.at")).expect("deps parse");
    assert_eq!(deps.deps.len(), 3);
    assert_eq!(
        deps.deps[0].path.as_deref(),
        Some("../../packages/blueprints")
    );

    let nested = parse_file(&fixture("static-nested-objects.pac.at")).expect("nested parse");
    assert_eq!(nested.external_backend.as_deref(), Some("../ash-server"));
    assert!(nested.raw_props.iter().any(|p| p.key == "npm_deps"));
    assert!(nested.raw_props.iter().any(|p| p.key == "theme"));
}

#[test]
fn consumer_sees_dynamic_and_bad_inputs_classified_not_faked() {
    let dyn_t = parse_file(&fixture("dynamic-template.pac.at")).expect("parse continues");
    assert!(!dyn_t.is_static_equivalent());
    assert_eq!(dyn_t.unsupported[0].kind, UnsupportedKind::Interpolation);
    assert_eq!(dyn_t.identity.name, None, "no faked value");

    let e = parse_file(&fixture("bad-missing-brace.pac.at")).unwrap_err();
    assert_eq!(e.stage, ErrorStage::Value);
    assert!(e.to_string().contains("bad-missing-brace.pac.at"));

    let bad_type = parse_file(&fixture("bad-type-members.pac.at")).expect("parse");
    assert_eq!(
        bad_type.unsupported[0].kind,
        UnsupportedKind::TypeMismatch { field: "members" }
    );

    let e = parse_file(&fixture("bad-missing-value.pac.at")).unwrap_err();
    assert_eq!(e.stage, ErrorStage::Value);
}

#[test]
fn consumer_toggle_normalization_matches_pac_rule() {
    let t = parse_file(&fixture("static-toggles.pac.at")).expect("toggles");
    assert!(matches!(&t.toggles.shadcn, ToggleValue::Str(s) if s == "off"));
    assert!(!t.toggles.shadcn.normalized());
    assert!(!t.toggles.default_classes.normalized());
}
