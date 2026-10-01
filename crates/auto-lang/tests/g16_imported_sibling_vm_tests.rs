//! PLAN-093 G-16: a use-IMPORTED child widget's template referencing its
//! same-file sibling must instantiate that sibling's subtree on the VM
//! track. musk real-app conviction (2026-10-01): structure column
//! (cross-file) rendered while canvas column (same-file sibling of an
//! imported widget) rendered an empty shell — zero AURA-CHILD-MISS (the
//! tracked component arm degrades to a silent `<Name />` text fallback).

use std::path::Path;

#[test]
fn g16_imported_widget_same_file_sibling_subtree_instantiates() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures/g16_imported");
    let fixture = fixture_dir.join("src/front/app.at");
    let code = std::fs::read_to_string(&fixture).expect("fixture app.at readable");

    let fixture_str = fixture.to_string_lossy().to_string();
    let comp = auto_lang::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("component builds");

    let (view, _, _) = comp.view_with_debug_gated(false);
    let debug = format!("{view:?}");

    assert!(
        debug.contains("host above panel") && debug.contains("host below panel"),
        "host shell nodes must survive: {}",
        line_snip(&debug)
    );
    assert!(
        debug.contains("panel shell"),
        "imported widget subtree must instantiate: {}",
        line_snip(&debug)
    );
    assert!(
        debug.contains("item body node"),
        "imported widget's same-file sibling subtree must instantiate: {}",
        line_snip(&debug)
    );
}

fn line_snip(s: &str) -> String {
    let mut out = s.chars().take(1600).collect::<String>();
    if out.len() < s.len() {
        out.push_str("…");
    }
    out
}
