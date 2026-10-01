//! PLAN-095 T-07 (G-11): same-file sibling widget instantiation on the VM
//! track.
//!
//! 093 §8.16 observed widget→widget child references rendering ZERO nodes on
//! the VM track when the child is a same-file sibling (no `use` line): the
//! root-file load took only the FIRST WidgetDecl, siblings never reached the
//! WidgetRegistry, and the component arm's registry miss degraded silently.
//! The fix registers same-file siblings (lib.rs root extraction) and emits a
//! locatable diagnostic on unresolvable references (aura_view_builder).

use std::path::Path;

#[test]
fn g11_same_file_sibling_subtree_instantiates() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let fixture_dir = manifest_dir.join("tests/fixtures/g11_sibling");
    let fixture = fixture_dir.join("src/front/app.at");
    let code = std::fs::read_to_string(&fixture).expect("fixture app.at readable");

    let fixture_str = fixture.to_string_lossy().to_string();
    let comp = auto_lang::build_dynamic_component(&code, Some(fixture_str.as_str()))
        .expect("component builds");

    let (view, _, _) = comp.view_with_debug_gated(false);
    let debug = format!("{view:?}");

    // Host shell intact.
    assert!(
        debug.contains("host above sibling") && debug.contains("host below sibling"),
        "host shell nodes must survive: {}",
        line_snip(&debug)
    );
    // G-11 contract: the sibling child's SUBTREE instantiates (pre-fix this
    // rendered zero nodes — not even the <G11Child /> placeholder).
    assert!(
        debug.contains("child body node"),
        "same-file sibling child subtree must instantiate on the VM track: {}",
        line_snip(&debug)
    );
    assert!(
        debug.contains("child-content"),
        "sibling child model-bound text must instantiate: {}",
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
