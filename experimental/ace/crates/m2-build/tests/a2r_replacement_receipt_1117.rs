//! PLAN-1117 T-04 (V-03): the `IF-ACE-A2R-V05-REPLACEMENT/1` receiving
//! gate. A GENUINE receipt (real builds, real runs, real hashes) passes;
//! each frozen negative — a dropped required row, an emit-only row, a
//! tampered artifact hash, an ACE claim over a legacy dispatch — is
//! rejected BY NAME.

mod common;

use ace_m2_build::driver::ProjectInputs;
use ace_m2_build::identity::TargetKind;
use ace_m2_build::receipt::{v05_row_from_sdk, V05Receipt, V05_RECEIPT_SCHEMA};
use common::{driver, scratch};

fn rustc_available() -> bool {
    std::process::Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string()))
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

const VALUE_SRC: &str = "fn main() int { let label = \"auto\".clone()\n return 12 + label.len() }";
const TRAP_SRC: &str = "fn main() int { let big = 2147483647\n return big + big }";

#[test]
fn p1117_v05_replacement_receipt_gate() {
    if !rustc_available() {
        eprintln!("SKIP: rustc unavailable on this host");
        return;
    }
    let root = scratch("p1117-v05-receipt");
    let mut d = driver(&root);

    // The candidate tree: one value SDK + one trap SDK (real builds).
    let value_inputs = ProjectInputs::new("oldproj", "main", "main.at", VALUE_SRC);
    let trap_inputs = ProjectInputs::new("trapproj", "main", "main.at", TRAP_SRC);
    for (inputs, sdk_name) in [(&value_inputs, "oldproj"), (&trap_inputs, "trapproj")] {
        d.build(TargetKind::A2r, inputs).expect("build");
        d.build_sdk(inputs, &[TargetKind::A2r], &root.join(sdk_name), Vec::new(), "test-grant")
            .expect("sdk");
    }
    let value_run = d
        .run_sdk(&root.join("oldproj"), TargetKind::A2r, "test-grant")
        .expect("value run");
    let trap_run = d
        .run_sdk(&root.join("trapproj"), TargetKind::A2r, "test-grant")
        .expect("trap run");
    assert_eq!(value_run.exit_code, Some(16), "12 + len(\"auto\")");
    assert_eq!(trap_run.stdout.trim(), "trap:Overflow");

    // The genuine receipt: four required rows from the real runs, each
    // hash-bound to its SDK, with a genuine ACE dispatch proof.
    let rows = [
        v05_row_from_sdk(
            "route.auto-rust",
            "oldproj",
            &root.join("oldproj"),
            value_run.exit_code,
            &value_run.stdout,
            &value_run.channel,
            "ace",
            1,
            0,
        )
        .unwrap(),
        v05_row_from_sdk(
            "route.build-lane",
            "oldproj",
            &root.join("oldproj"),
            value_run.exit_code,
            &value_run.stdout,
            &value_run.channel,
            "ace",
            1,
            0,
        )
        .unwrap(),
        v05_row_from_sdk(
            "route.trap",
            "trapproj",
            &root.join("trapproj"),
            trap_run.exit_code,
            &trap_run.stdout,
            &trap_run.channel,
            "ace",
            1,
            0,
        )
        .unwrap(),
        v05_row_from_sdk(
            "route.rollback",
            "oldproj",
            &root.join("oldproj"),
            value_run.exit_code,
            &value_run.stdout,
            &value_run.channel,
            "ace",
            1,
            0,
        )
        .unwrap(),
    ];
    let receipt = V05Receipt {
        schema: V05_RECEIPT_SCHEMA.into(),
        commit: "0daef20f9".into(),
        toolchain: "rustc 1.98.0 (test)".into(),
        rows: rows.to_vec(),
    };
    let json = receipt.to_json();
    let validated = V05Receipt::validate(&json, Some(&root)).expect("the genuine receipt passes");
    assert_eq!(validated, receipt);

    // PLAN-1117 T-04 issuance hook: with AUTO_LANG_V05_RECEIPT_OUT set to an
    // explicit path, the same validated genuine receipt is written there
    // (the signed-off sample the 985/986/987 chain can re-validate). No
    // default writes — the daily run never touches files outside its scratch.
    if let Some(out) = std::env::var_os("AUTO_LANG_V05_RECEIPT_OUT") {
        std::fs::write(&out, &json).expect("write the issued receipt sample");
    }

    // Negative 1 — a dropped required row.
    let mut dropped = receipt.clone();
    dropped.rows.retain(|r| r.id != "route.trap");
    let err = V05Receipt::validate(&dropped.to_json(), Some(&root)).expect_err("missing row");
    assert_eq!(err.code, "v05.required-row", "{err}");

    // Negative 2 — emit-only evidence masquerading as a run.
    let mut emit_only = receipt.clone();
    emit_only.rows[0].stage = "emit".into();
    let err = V05Receipt::validate(&emit_only.to_json(), Some(&root)).expect_err("emit-only");
    assert_eq!(err.code, "v05.stage", "{err}");

    // Negative 3 — a tampered artifact hash.
    let mut wrong_artifact = receipt.clone();
    wrong_artifact.rows[1].artifact_sha256 = "0".repeat(64);
    let err =
        V05Receipt::validate(&wrong_artifact.to_json(), Some(&root)).expect_err("wrong artifact");
    assert_eq!(err.code, "v05.artifact", "{err}");

    // Negative 4 — labelled ACE, actually dispatched legacy.
    let mut legacy = receipt.clone();
    legacy.rows[2].legacy_dispatches = 1;
    let err = V05Receipt::validate(&legacy.to_json(), Some(&root)).expect_err("legacy dispatch");
    assert_eq!(err.code, "v05.dispatch", "{err}");

    let _ = std::fs::remove_dir_all(&root);
}
