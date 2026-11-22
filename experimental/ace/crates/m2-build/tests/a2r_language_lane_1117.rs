//! PLAN-1117 T-02 (V-01 build-lane pin): the A2R build lane rides the
//! ORDINARY-LANGUAGE channel — an old-project-shaped source (record +
//! list + closure + nested place write, beyond the scalar surface) emits
//! the LANGUAGE fileset schema, compiles through REAL rustc into the
//! precompiled exe, and the packed SDK runs the expected value after
//! relocation. The lane's channel identity (manifest schema + generated
//! header) is the dispatch witness: these products cannot come from the
//! scalar channel or the legacy emitter.

mod common;

use ace_m2_build::driver::M2Build;
use ace_m2_build::identity::TargetKind;
use common::{driver, scratch};

fn rustc_available() -> bool {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 1115 v01-e shape (single file): the list becomes [19, 23] through the
/// nested-place write, `each_plus` sums x*1 = 42 — the mutation is
/// load-bearing for the oracle.
const LANGUAGE_SRC: &str = "fn each_plus(f fn(int)int, xs List<int>) int {\n\
     var n = 0\n\
     for x in xs {\n\
         let y = f(x)\n\
         n = n + y\n\
     }\n\
     return n\n\
 }\n\
 type Row { xs List<int> }\n\
 fn main() int {\n\
     var r = Row([19, 20])\n\
     r.xs[1] += 3\n\
     return each_plus((x int) => x * 1, r.xs)\n\
 }";

#[test]
fn p1117_a2r_lane_rides_the_language_channel() {
    if !rustc_available() {
        eprintln!("SKIP: rustc unavailable on this host");
        return;
    }
    let root = scratch("p1117-a2r-lang");
    let mut d = driver(&root);
    let inputs = ace_m2_build::driver::ProjectInputs::new("lang01", "main", "main.at", LANGUAGE_SRC);

    let outcome = d.build(TargetKind::A2r, &inputs).expect("a2r language build");
    // The channel identity: the lane's assembly manifest carries the
    // LANGUAGE fileset schema (not the scalar `auto.a2x.rust.fileset/1`).
    let manifest = outcome
        .products
        .iter()
        .find(|(p, _)| p == "targets/a2r/assembly-manifest.json")
        .expect("the assembly manifest rides the products");
    let manifest_text = String::from_utf8_lossy(&manifest.1).into_owned();
    assert!(
        manifest_text.contains("auto.a2x.rust.language.fileset/1"),
        "language fileset schema: {manifest_text}"
    );
    let generated = outcome
        .products
        .iter()
        .find(|(p, _)| p == "targets/a2r/generated.rs")
        .expect("generated.rs rides the products");
    assert!(
        String::from_utf8_lossy(&generated.1).contains("PLAN-1115 IF-ACE-A2R-LANGUAGE/1"),
        "the generated header carries the language channel identity"
    );
    // One counted rustc spawn, a PE exe, and the SDK run observes 42
    // (19 + 23) through the precompiled artifact after relocation.
    assert_eq!(
        outcome.counts.tool_spawns.get("rustc-a2r").copied().unwrap_or(0),
        1,
        "exactly one counted rustc spawn"
    );
    let exe = outcome
        .products
        .iter()
        .find(|(p, _)| p == "targets/a2r/lang01.exe")
        .expect("the precompiled exe");
    assert!(exe.1.windows(2).any(|w| w == b"MZ"), "exe is a PE image");

    let sdk = root.join("sdk");
    d.build_sdk(&inputs, &[TargetKind::A2r], &sdk, Vec::new(), "test-grant")
        .expect("sdk pack");
    M2Build::verify_sdk(&sdk).expect("sdk verifies");
    let run = d
        .run_sdk(&sdk, TargetKind::A2r, "test-grant")
        .expect("sdk run");
    assert_eq!(run.exit_code, Some(42), "19 + 23 through the packed exe");
    assert_eq!(run.channel, "process-stdout");

    let _ = std::fs::remove_dir_all(&root);
}
