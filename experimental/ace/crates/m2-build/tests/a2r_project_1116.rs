//! PLAN-1116 T-04 V-03: the A2R build lane and relocatable SDK — the
//! emitted language fileset goes through REAL rustc into a precompiled
//! exe; the packed SDK verifies, relocates out of the development tree,
//! runs the precompiled artifact, rejects a tampered artifact by name,
//! reads the old usable root back, and a business edit rebuilds under a
//! new identity.

mod common;

use ace_m2_build::driver::M2Build;
use ace_m2_build::identity::TargetKind;
use common::{b01_edited_inputs, b01_inputs, driver, scratch, B01_EDITED, B01_SRC};

fn rustc_available() -> bool {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Recursively copy a directory (the relocation step).
fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).expect("relocate mkdir");
    for entry in std::fs::read_dir(src).expect("relocate read") {
        let entry = entry.expect("relocate entry");
        let to = dst.join(entry.file_name());
        if entry.file_type().expect("relocate type").is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).expect("relocate copy");
        }
    }
}

#[test]
fn p1116_a2r_build_sdk_relocate_run_recover() {
    if !rustc_available() {
        eprintln!("SKIP: rustc unavailable on this host");
        return;
    }
    let root = scratch("p1116-a2r-sdk");
    let mut d = driver(&root);
    let inputs = b01_inputs();

    // 1) The A2R BUILD lane: real rustc over the emitted fileset — the
    //    precompiled exe is a product, with the build receipt's tool
    //    identity and a counted tool spawn.
    let outcome = d
        .build(TargetKind::A2r, &inputs)
        .expect("a2r build");
    let exe = outcome
        .products
        .iter()
        .find(|(p, _)| p == "targets/a2r/b01.exe")
        .expect("the a2r lane produces a precompiled exe");
    assert!(exe.1.windows(2).any(|w| w == b"MZ"), "exe is a PE image");
    let receipt = outcome
        .products
        .iter()
        .find(|(p, _)| p == "targets/a2r/build-receipt.json")
        .expect("the build receipt rides the products");
    let receipt: serde_json::Value =
        serde_json::from_slice(&receipt.1).expect("receipt json");
    assert!(receipt["rustc_identity"]
        .as_str()
        .unwrap_or_default()
        .starts_with("rustc "),);
    assert_eq!(
        outcome.counts.tool_spawns.get("rustc-a2r").copied().unwrap_or(0),
        1,
        "exactly one counted rustc spawn"
    );

    // 2) The SDK packs the exe with the run closure's identity axes.
    let sdk = root.join("sdk");
    let recipe = d
        .build_sdk(&inputs, &[TargetKind::A2r], &sdk, Vec::new(), "test-grant")
        .expect("sdk pack");
    assert!(recipe
        .run_dependencies
        .iter()
        .any(|r| r.kind == "crt"),);
    assert!(recipe
        .build_inputs
        .iter()
        .any(|b| b.name == "rustc"),);
    M2Build::verify_sdk(&sdk).expect("sdk verifies");

    // 3) Relocate OUT of the development tree and run the precompiled
    //    artifact (the observation is the process exit = the value).
    let relocated = root.join("relocated-sdk");
    copy_dir(&sdk, &relocated);
    let run = d
        .run_sdk(&relocated, TargetKind::A2r, "test-grant")
        .expect("relocated run");
    assert_eq!(run.exit_code, Some(14), "2 + 3 * 4 through the packed exe");
    assert_eq!(run.channel, "process-stdout");

    // 4) A tampered artifact is a named rejection; the old usable root
    //    reads back and still runs.
    let exe_path = relocated.join("targets").join("a2r").join("b01.exe");
    let good = std::fs::read(&exe_path).unwrap();
    let mut bad = good.clone();
    bad[0x400] ^= 0x01;
    std::fs::write(&exe_path, &bad).unwrap();
    let rejected = d.run_sdk(&relocated, TargetKind::A2r, "test-grant");
    assert!(
        rejected.is_err(),
        "the tampered exe must not run"
    );
    std::fs::write(&exe_path, &good).unwrap();
    let run = d
        .run_sdk(&relocated, TargetKind::A2r, "test-grant")
        .expect("restored run");
    assert_eq!(run.exit_code, Some(14));

    // 5) A BUSINESS edit rebuilds under a new identity and the new exe
    //    observes the new value (2 + 3 * 5 = 17).
    let edited = b01_edited_inputs();
    let outcome2 = d.build(TargetKind::A2r, &edited).expect("edited build");
    assert_ne!(
        outcome.identity.key(),
        outcome2.identity.key(),
        "a business edit changes the build identity"
    );
    let sdk2 = root.join("sdk2");
    d.build_sdk(&edited, &[TargetKind::A2r], &sdk2, Vec::new(), "test-grant")
        .expect("edited sdk");
    let run = d
        .run_sdk(&sdk2, TargetKind::A2r, "test-grant")
        .expect("edited run");
    assert_eq!(run.exit_code, Some(17), "{B01_EDITED} vs {B01_SRC}");

    let _ = std::fs::remove_dir_all(&root);
}
