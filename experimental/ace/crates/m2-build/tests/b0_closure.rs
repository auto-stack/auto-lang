//! `ace.m2.p978.b0-closure` — V-B0-CLOSURE / V-B0-EDIT / V-DETAIL-01
//! (PLAN-978 T-B0-I/T-B0-V): the minimal AV build/run closure, repeat
//! run + Auto body edit with ZERO cargo/rustc/bindgen processes, the
//! value following the edit, and the same-entry failure recovery.

mod common;

use ace_m2_build::driver::M2Build;
use ace_m2_build::identity::TargetKind;
use common::{b01_edited_inputs, b01_inputs, owned_tools_guard, scratch};

/// V-B0-EDIT / V-DETAIL-01 (leg 1): B01 runs 14 on AV; a repeat run is a
/// precise-cache hit; a body edit 4→5 reruns 17 — across the WHOLE
/// sequence zero cargo/rustc/bindgen processes descend from this test
/// (the independent attributed oracle; external agents' builds are not
/// ours and cannot false-positive it).
#[test]
fn p978_b0_edit_runs_new_value_with_zero_tool_processes() {
    let root = scratch("b0-edit");
    let ((first, hit, edited), owned, available) = owned_tools_guard(|| {
        let mut driver = M2Build::new(&root);
        let first = driver.build(TargetKind::Av, &b01_inputs()).unwrap();
        let hit = driver.build(TargetKind::Av, &b01_inputs()).unwrap();
        let edited = driver.build(TargetKind::Av, &b01_edited_inputs()).unwrap();
        (first, hit, edited)
    });
    assert!(available, "the attributed process oracle must be available");
    assert!(
        owned.is_empty(),
        "AV repeat + body edit must launch zero toolchain processes: {owned:?}"
    );
    let first_run = first.run.expect("first build runs");
    assert_eq!(first_run.exit_code, Some(14), "B01 must run 14 on AV");
    assert_eq!(first_run.channel, "av-value");
    assert!(!hit.rebuilt, "identical identity must be a cache hit");
    assert!(edited.rebuilt, "a body edit flips the build identity");
    assert_eq!(edited.run.unwrap().exit_code, Some(17));
}

/// V-B0-CLOSURE (leg 1, AV): the minimal closure rebuilds from scratch in
/// an isolated root and the failure→recovery sequence (illegal neighbor,
/// then the same entry again) keeps working — P2/P3 of the bounded probe
/// as a permanent case.
#[test]
fn p978_b0_illegal_neighbor_and_recovery() {
    let root = scratch("b0-neg");
    let mut driver = M2Build::new(&root);
    // Same-type illegal neighbor: a syntax error is a NAMED rejection,
    // never a panic and never a fallback.
    let bad = ace_m2_build::driver::ProjectInputs::new(
        "bad",
        "main",
        "main.at",
        "fn main() int { return 2 + }",
    );
    let err = driver
        .build(TargetKind::Av, &bad)
        .expect_err("illegal neighbor must reject");
    assert_eq!(err.code, "m2build.lane-av");
    assert!(
        err.detail.contains("source.syntax"),
        "named syntax diagnostic, got: {}",
        err.detail
    );
    // Failure keeps the store usable: the same entry succeeds right
    // after, and the previous good identity still hits.
    let ok = driver.build(TargetKind::Av, &b01_inputs()).unwrap();
    assert_eq!(ok.run.unwrap().exit_code, Some(14));
    // Multi-source project is a named boundary (no silent merge).
    let multi = ace_m2_build::driver::ProjectInputs {
        name: "multi".into(),
        entry: "main".into(),
        caps: vec![],
        sources: vec![
            ("a.at".into(), "fn main() int { return 1 }".into()),
            ("b.at".into(), "fn main() int { return 2 }".into()),
        ],
        stdlib: vec![],
        rust_provider: None,
    };
    let err = M2Build::new(&root)
        .build(TargetKind::Av, &multi)
        .expect_err("multi-source must be a named boundary");
    assert_eq!(err.code, "m2build.project.multi-source");
}

/// The B0 public-entry seam: the prepared CLI builds and the same
/// prepared CLI runs — the recipe a 980/HELLO consumer would follow
/// (V-B0-CLOSURE, command-level).
#[test]
fn p978_b0_cli_build_av_runs_14() {
    let dir = scratch("b0-cli");
    let proj = dir.join("proj");
    std::fs::create_dir_all(proj.join("src")).unwrap();
    std::fs::write(
        proj.join("project.json"),
        r#"{"name":"b01","entry":"main"}"#,
    )
    .unwrap();
    std::fs::write(proj.join("src/main.at"), common::B01_SRC).unwrap();
    let out = std::process::Command::new(common::ac_build_exe())
        .args([
            "build",
            "--project",
            proj.to_str().unwrap(),
            "--target",
            "av",
            "--root",
            dir.join("root").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "cli build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(doc["builds"][0]["run"]["exit_code"], 14);
}
