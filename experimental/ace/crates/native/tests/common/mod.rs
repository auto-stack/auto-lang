//! Shared helpers of the PLAN-818 MIR-native test tiers (codegen/parity).

#![allow(dead_code)]

pub mod collections;

use ace_native::driver::{MnDriver, MnRequest};

pub fn tools_available() -> bool {
    ace_core::link::find_rust_lld().is_ok() && ace_core::link::find_sdk_um_dir().is_ok()
}

pub fn repo_root() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..")
}

/// The landed ac-trace-support staticlib (built on first use, exactly like
/// the landed ac-core suites).
pub fn trace_support_lib() -> std::path::PathBuf {
    let support = repo_root().join("experimental/ace/crates/trace-runtime");
    let lib = support.join("../../target/runtime-dev/ace_trace_runtime.lib");
    if !lib.is_file() {
        let mut owned_compiler = std::process::Command::new("cargo");
        owned_compiler
            .args(["build", "--locked", "--profile", "runtime-dev"])
            .current_dir(&support)
            .env_remove("CARGO_TARGET_DIR")
            .arg("--target-dir")
            .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"));
        let out = auto_tool_execution::run_command_output(owned_compiler)
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
pub fn run_av(src: &str, caps: &[&str]) -> ace_av::driver::RequestOutcome {
    let caps: Vec<String> = caps.iter().map(|s| s.to_string()).collect();
    let mut d = ace_av::driver::Driver::ephemeral();
    d.request(src, "d_main", &caps)
}
