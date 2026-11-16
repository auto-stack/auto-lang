//! `ace.m2.p978.load-selection` — V-LOAD (PLAN-978 T-09, SDK-side
//! ledger): the 1043 suffix-layer selection into a candidate SDK —
//! correct selected/candidate partition per target and the named
//! rejections (unknown suffix, missing public layer, duplicate layer,
//! non-Auto file).

mod common;

use ace_m2_build::load::select_layers;
use ace_m2_build::M2Reject;

fn files() -> Vec<(String, String)> {
    vec![
        ("main.at".into(), "fn main() int { return 1 }".into()),
        (
            "math.at".into(),
            "pub fn add(a: int, b: int) int { return a + b }".into(),
        ),
        (
            "math.rs.at".into(),
            "pub fn add(a: int, b: int) int { return a + b }".into(),
        ),
        (
            "math.c.at".into(),
            "pub fn add(a: int, b: int) int { return a + b }".into(),
        ),
    ]
}

/// The a2r target selects public + the `.rs` layer; the `.c` layer stays
/// a CANDIDATE (same-directory unselected — never "actual"). The a2c
/// target mirrors it. Unselected layers never bind.
#[test]
fn p978_load_selection_partitions_per_target() {
    let for_rust = select_layers(&files(), Some("rs")).unwrap();
    assert_eq!(for_rust.target, "rs");
    assert!(for_rust.selected.iter().any(|l| l.file == "main.at"));
    assert!(for_rust.selected.iter().any(|l| l.file == "math.at"));
    assert!(for_rust.selected.iter().any(|l| l.file == "math.rs.at"));
    assert!(
        for_rust
            .candidate_files
            .iter()
            .any(|l| l.file == "math.c.at"),
        "the c layer stays a candidate for the rs target"
    );
    let for_c = select_layers(&files(), Some("c")).unwrap();
    assert!(for_c.selected.iter().any(|l| l.file == "math.c.at"));
    assert!(for_c.candidate_files.iter().any(|l| l.file == "math.rs.at"));
    // Public-only target (AV): both provider layers stay candidates.
    let for_av = select_layers(&files(), None).unwrap();
    assert_eq!(for_av.selected.len(), 2);
    assert_eq!(for_av.candidate_files.len(), 2);
}

/// Unknown provider suffixes reject BEFORE any binding (the 1043 frozen
/// suffix set — `.foo.at` is not a layer, `x.rs` is not an Auto file).
#[test]
fn p978_load_unknown_suffix_rejected() {
    let err = select_layers(&[("m.foo.at".into(), String::new())], Some("rs"))
        .expect_err("unknown suffix must reject");
    assert_eq!(err.code, "m2build.load.unknown-suffix");
    let err = select_layers(&[("lib.rs".into(), String::new())], Some("rs"))
        .expect_err("non-.at file must reject");
    assert_eq!(err.code, "m2build.load.not-auto-layer");
}

/// A provider layer without its public layer cannot be selected by any
/// target — the public face defines the module.
#[test]
fn p978_load_missing_public_rejected() {
    let err = select_layers(&[("math.rs.at".into(), String::new())], Some("rs"))
        .expect_err("orphan provider layer must reject");
    assert_eq!(err.code, "m2build.load.missing-public");
}

/// Two files claiming the same (module, layer) role reject — a duplicate
/// implementation is never silently resolved.
#[test]
fn p978_load_duplicate_layer_rejected() {
    let err = select_layers(
        &[
            ("math.at".into(), String::new()),
            ("math.rs.at".into(), String::new()),
            ("math.rs.at".into(), String::new()),
        ],
        Some("rs"),
    )
    .expect_err("duplicate (module, layer) must reject");
    assert_eq!(err.code, "m2build.load.duplicate-layer");
}

/// The rejection type is the crate's named M2Reject (pre-dispatch codes
/// ride the same channel as every other boundary).
#[test]
fn p978_load_rejections_are_named_m2() {
    let e = M2Reject::new("m2build.load.unknown-suffix", "x");
    assert!(e.code.starts_with("m2build."));
    assert!(!e.detail.is_empty());
}

/// V-LOAD (producer-side conformance): a packed candidate SDK carries
/// the EXACT layout the language-side loader's SDK contract consumes
/// (loader.rs `AUTO_DELIVERY_SOURCE_ROOT`/exe-adjacent branches):
/// `install-manifest.json` at the root + the declared stdlib closure at
/// `stdlib/auto/...`. The loader-side oracle itself is PLAN-890's
/// `selected_sdk_stdlib_is_authoritative` (not duplicated here).
#[test]
fn p978_sdk_layout_conforms_to_loader_contract() {
    let root = common::scratch("sdk-loader-shape");
    let mut inputs = common::b01_inputs();
    inputs.stdlib = vec![(
        "stdlib/auto/math.at".to_string(),
        "pub fn add(a: int, b: int) int { return a + b }".to_string(),
    )];
    let mut driver = common::driver(&root);
    let sdk = root.join("sdk");
    driver
        .build_sdk(
            &inputs,
            &[ace_m2_build::identity::TargetKind::Av],
            &sdk,
            vec![],
            "run:load",
        )
        .unwrap();
    // The two branches the loader probes.
    assert!(
        sdk.join("install-manifest.json").is_file(),
        "the recipe document sits at the SDK root"
    );
    let stdlib_member = sdk.join("stdlib").join("auto").join("math.at");
    assert!(
        stdlib_member.is_file(),
        "the declared stdlib closure lands at stdlib/auto (got {:?})",
        stdlib_member
    );
    // The declared member is recorded in the recipe's provenance.
    let (recipe, _files) = ace_m2_build::recipe::SdkRecipe::load_verify(&sdk).unwrap();
    assert!(
        recipe
            .provenance
            .iter()
            .any(|p| p.stdlib.iter().any(|s| s.path == "stdlib/auto/math.at")),
        "the stdlib closure is part of the build identity"
    );
    // The recipe still verifies with the extra stdlib tree present.
    assert!(M2Build::verify_sdk(&sdk).is_ok());
}

use ace_m2_build::driver::M2Build;
