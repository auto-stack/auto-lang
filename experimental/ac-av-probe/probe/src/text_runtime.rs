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
    // probe root = <repo>/experimental/ac-av-probe -> sibling ac-core.
    let rt = root
        .parent()
        .map(|p| p.join("ac-core").join("runtime").join("text"));
    let Some(rt) = rt else {
        return Err("ac-core runtime/text workspace not located".to_string());
    };
    if !rt.join("Cargo.toml").is_file() {
        return Err(format!("text runtime workspace missing at {}", rt.display()));
    }
    let out = std::process::Command::new("cargo")
        .args([
            "build",
            "-p",
            "ac-text-runtime-shim",
            "--release",
            "--locked",
        ])
        .current_dir(&rt)
        .output()
        .map_err(|e| format!("spawn cargo for the text runtime: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "text runtime build failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }
    let lib = rt
        .join("target")
        .join("release")
        .join("ac_text_runtime.lib");
    if !lib.is_file() {
        return Err(format!("runtime lib missing at {}", lib.display()));
    }
    *g = Some(lib.clone());
    Ok(lib)
}

fn probe_workspace_root() -> Result<PathBuf, String> {
    // The workspace root is the ancestor holding both member crates.
    let mut dir: Option<&Path> = Some(Path::new(env!("CARGO_MANIFEST_DIR")));
    while let Some(d) = dir {
        if d.join("core").join("Cargo.toml").is_file()
            && d.join("probe").join("Cargo.toml").is_file()
        {
            return Ok(d.to_path_buf());
        }
        dir = d.parent();
    }
    Err("probe workspace root not located".to_string())
}

/// The trace support library shipped with ac-core (built by its test
/// harness); `None` when not yet built — callers treat that as infra-blocked
/// for trace parity, never as a pass.
pub fn trace_support_lib(repo_root: &Path) -> Option<PathBuf> {
    let p = repo_root.join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    p.is_file().then_some(p)
}
