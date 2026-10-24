//! Shared helpers of the PLAN-818 MIR-native test tiers (codegen/parity).

#![allow(dead_code)]

use ac_mn::driver::{MnDriver, MnRequest};

pub fn tools_available() -> bool {
    auto_ac_prototype::link::find_rust_lld().is_ok()
        && auto_ac_prototype::link::find_sdk_um_dir().is_ok()
}

pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The landed ac-trace-support staticlib (built on first use, exactly like
/// the landed ac-core suites).
pub fn trace_support_lib() -> std::path::PathBuf {
    let support = repo_root().join("experimental/ac-core/test-support");
    let lib = support.join("target/debug/ac_trace_support.lib");
    if !lib.is_file() {
        let out = std::process::Command::new("cargo")
            .args(["build", "--locked"])
            .current_dir(&support)
            .output()
            .expect("spawn cargo for test-support");
        assert!(
            out.status.success(),
            "test-support build failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert!(lib.is_file(), "support lib missing at {}", lib.display());
    lib
}

/// One NEW-native request (ephemeral, no store) under a unique workdir.
pub fn run_mn(src: &str, caps: &[&str], tag: &str) -> MnRequest {
    let workdir = std::env::temp_dir().join(format!("mn8-t03-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    let lib = if caps.is_empty() {
        None
    } else {
        Some(trace_support_lib())
    };
    let mut d = MnDriver::ephemeral();
    let r = d.request(src, "d_main", &caps, &workdir, lib.as_deref());
    let _ = std::fs::remove_dir_all(&workdir);
    r
}

/// One AV leg request (798 delivered executor).
pub fn run_av(src: &str, caps: &[&str]) -> auto_av_probe::driver::RequestOutcome {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    let mut d = auto_av_probe::driver::Driver::ephemeral();
    d.request(src, "d_main", &caps)
}

/// One OLD HIR-native leg (landed reference path).
pub fn run_old(src: &str, caps: &[&str], tag: &str) -> auto_av_probe::native_reference::NativeRun {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    let workdir = std::env::temp_dir().join(format!("mn8-t03-old-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workdir);
    let lib = if caps.is_empty() {
        None
    } else {
        Some(trace_support_lib())
    };
    let run = auto_av_probe::native_reference::build_and_run(
        src,
        &caps,
        "d_main",
        &workdir,
        lib.as_deref(),
    )
    .expect("old-native build+run");
    let _ = std::fs::remove_dir_all(&workdir);
    run
}
