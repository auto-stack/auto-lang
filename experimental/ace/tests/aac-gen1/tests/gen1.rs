//! PLAN-1053 T-02 conformance: the Gen1 native AAC executable.
//!
//! Daily lane — consumes the prepared executable via `AAC1053_GEN1_DIR`
//! (built by the explicit `prepare-gen1` build/integration lane, never
//! inside these tests). The boot reduction oracle is the independent AV
//! lane over identical Auto source: the native PE's exit code must equal
//! the AV-executed reduction of the same whole-subject program.

use std::process::Command;
use std::time::Instant;

use ace_aac_gen1_conformance::{
    compile_gen1, prepared_gen1, run_gen1_boot, scratch, gen1_fingerprint, DRIVER_MANIFEST,
    GEN1_EXE,
};

/// The AV-lane reference: the same whole-subject program (all of S plus
/// the boot entry) executed on AV; its `main` return is the reduction the
/// native exit code must reproduce.
fn av_reference_reduction() -> i32 {
    let product = compile_gen1().expect("compile whole subject S for AV reference");
    let dir = scratch("av-ref");
    let av = ace_av::modules::packages::PackageAvTarget::cold(&dir, &product)
        .expect("AV cold target for whole-subject reference");
    let limits = ace_av_core::ExecLimits {
        max_steps: 64_000_000,
        wall: std::time::Duration::from_secs(300),
        ..ace_av_core::ExecLimits::default()
    };
    let (out, _trace) =
        ace_av_core::execute_program(&av.current, &limits, &ace_av_core::CancelFlag::default());
    let _ = std::fs::remove_dir_all(&dir);
    match out {
        ace_av_core::contract::ExecOutcome::Success {
            value: ace_av_core::contract::Value::I32(v),
            ..
        } => v,
        other => panic!("AV reference for Gen1 boot: {other:?}"),
    }
}

#[test]
fn gen1_boot_native_matches_av_reference() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe (run prepare-gen1, set AAC1053_GEN1_DIR)");
    let reference = av_reference_reduction();
    let observed = run_gen1_boot(exe).expect("run Gen1 boot");
    assert_eq!(
        observed, reference,
        "native Gen1 boot reduction must equal the independent AV reference"
    );
    assert_ne!(observed, 0, "boot reduction must not degenerate to zero");
    eprintln!("gen1 boot parity: exit={observed} in {:?} (<=10s gate)", started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "gen1 boot case exceeded the 10-second gate: {:?}",
        started.elapsed()
    );
}

#[test]
fn gen1_exe_is_a_pe_with_manifest_identity() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe");
    let bytes = std::fs::read(exe).expect("read Gen1 exe");
    assert_eq!(&bytes[..2], b"MZ", "Gen1 executable must be a Windows PE");
    let manifest = std::fs::read_to_string(
        exe.parent().expect("exe parent").join(DRIVER_MANIFEST),
    )
    .expect("read manifest");
    let mut fields = manifest.lines();
    assert_eq!(
        fields.next().map(str::to_owned),
        Some(gen1_fingerprint().to_hex()),
        "manifest fingerprint must bind the current whole-tree source identity"
    );
    assert_eq!(
        fields.next().map(str::to_owned),
        Some(blake3::hash(&bytes).to_hex()),
        "manifest executable digest must match the executable bytes"
    );
    let runtime_id = fields.next().expect("manifest runtime identity line");
    assert!(!runtime_id.is_empty());
    let tool = fields.next().expect("manifest tool identity line");
    assert!(tool.contains("rustc"), "tool identity must record the linker toolchain");
    eprintln!("gen1 identity: {} bytes, PE ok, manifest ok, {:?}", bytes.len(), started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "identity case exceeded the 10-second gate"
    );
}

#[test]
fn gen1_stale_preparation_is_refused() {
    let started = Instant::now();
    let exe = prepared_gen1().expect("prepared Gen1 exe");
    let root = exe.parent().expect("exe parent").to_path_buf();
    let tampered = scratch("tampered-manifest");
    std::fs::copy(&root.join(GEN1_EXE), tampered.join(GEN1_EXE)).expect("copy exe");
    let manifest = std::fs::read_to_string(root.join(DRIVER_MANIFEST)).expect("read manifest");
    let mut lines: Vec<String> = manifest.lines().map(str::to_owned).collect();
    lines[0] = "0".repeat(64);
    std::fs::write(tampered.join(DRIVER_MANIFEST), lines.join("\n")).expect("write tampered");
    // A child process with the tampered directory must fail admission.
    let output = Command::new(std::env::current_exe().expect("current test exe"))
        .arg("--exact")
        .arg("gen1_boot_native_matches_av_reference")
        .arg("--nocapture")
        .env("AAC1053_GEN1_DIR", &tampered)
        .output()
        .expect("spawn child with tampered preparation");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "tampered preparation must fail admission, child exited {:?}",
        output.status
    );
    assert!(
        stderr.contains("stale") || stderr.contains("fingerprint"),
        "child refusal must name the stale fingerprint, got: {stderr}"
    );
    eprintln!("gen1 stale refusal ok in {:?}", started.elapsed());
    assert!(
        started.elapsed().as_secs_f64() <= 10.0,
        "stale-refusal case exceeded the 10-second gate"
    );
}
