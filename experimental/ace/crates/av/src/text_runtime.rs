//! PLAN-871 T-06: shared runtime/support-library LOCATION helpers, split out
//! of the retiring `native_reference` module. They locate or build the
//! REPOSITORY's own support artifacts (they never lower or link anything
//! themselves), so they are not part of the old HIR→CLIF retirement surface:
//! the shared-MIR backend keeps consuming the same text runtime library.

use std::path::{Path, PathBuf};

/// PLAN-821: locate-or-build the text runtime support lib (the 782 native
/// chain's `ac_text_runtime.lib`). Cached per process; the ac-core runtime
/// workspace builds under its own target dir exactly as the 782 test
/// harness does.
pub fn text_runtime_lib() -> Result<PathBuf, String> {
    use std::sync::Mutex;
    static CACHE: Mutex<Option<PathBuf>> = Mutex::new(None);
    let mut g = CACHE.lock().expect("text runtime cache");
    if let Some(p) = g.as_ref() {
        return Ok(p.clone());
    }
    let root = probe_workspace_root()?;
    // probe root = <repo>/experimental/ace -> sibling ac-core.
    let rt = root.join("crates/text-runtime");
    if !rt.join("Cargo.toml").is_file() {
        return Err(format!(
            "text runtime workspace missing at {}",
            rt.display()
        ));
    }
    let mut owned_compiler = std::process::Command::new("cargo");
    owned_compiler
        .args([
            "build",
            "-p",
            "ace-text-runtime",
            "--profile",
            "runtime-release",
            "--locked",
        ])
        .current_dir(&rt)
        .env_remove("CARGO_TARGET_DIR")
        .arg("--target-dir")
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    let out = auto_tool_execution::run_command_output(owned_compiler)
        .map_err(|e| format!("spawn cargo for the text runtime: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "text runtime build failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let lib = root.join("target/runtime-release/ac_text_runtime.lib");
    if !lib.is_file() {
        return Err(format!("runtime lib missing at {}", lib.display()));
    }
    *g = Some(lib.clone());
    Ok(lib)
}

fn probe_workspace_root() -> Result<PathBuf, String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if root.join("crates/core/Cargo.toml").is_file() {
        Ok(root)
    } else {
        Err("ACE workspace root not located".into())
    }
}

/// The trace support library shipped with ac-core (built by its test
/// harness); `None` when not yet built — callers treat that as infra-blocked
/// for trace parity, never as a pass.
pub fn trace_support_lib(repo_root: &Path) -> Option<PathBuf> {
    let p = repo_root.join("experimental/ace/target/runtime-dev/ace_trace_runtime.lib");
    p.is_file().then_some(p)
}
