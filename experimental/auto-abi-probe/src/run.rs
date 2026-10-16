//! Link + run helpers reusing the merged ac-core entries verbatim
//! (plan §3): `find_rust_lld` / `find_sdk_um_dir` /
//! `link_object_staged` / `run_deadlined_split`. Nothing here invents a
//! second linker or executor.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Frozen T-01 probe run budgets (plan §3; full enforcement design in
/// contract-decision.md §6). The 70s total budget is enforced by the
/// per-request harness at the caller; this module owns the run leg.
pub const RUN_DEADLINE: Duration = Duration::from_secs(5);
/// stdout/stderr acceptance caps: outputs beyond 1 MiB each are a FAILED
/// probe request, never truncated into a green result.
pub const OUTPUT_CAP: usize = 1024 * 1024;

#[derive(Debug)]
pub struct ProbeError(pub String);

impl std::fmt::Display for ProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.probe: {}", self.0)
    }
}

/// Observed outcome of one PE run: exit code plus collected outputs.
#[derive(Debug, Clone)]
pub struct PeRun {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Scratch dir for one named probe case (under the workspace target/ so
/// nothing leaks outside the build tree). Recreated on every call — a
/// probe request starts from an empty directory, never from a previous
/// run's leftovers.
pub fn scratch(case: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("autoabi-t01")
        .join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("probe scratch dir");
    dir
}

/// Link object files into a staged PE (not published) with the REAL
/// ac-core staged-link entry. `objs[0]` is the primary object (it must
/// contain the entry symbol); the rest ride along as additional link
/// inputs — exactly how mixed-producer PEs are formed.
pub fn link_pe(dir: &Path, objs: &[PathBuf], entry: &str) -> Result<PathBuf, ProbeError> {
    assert!(objs.len() >= 2, "interop legs link at least two objects");
    let lld = auto_ac_prototype::link::find_rust_lld()
        .map_err(|e| ProbeError(format!("find_rust_lld: {e:?}")))?;
    let sdk = auto_ac_prototype::link::find_sdk_um_dir()
        .map_err(|e| ProbeError(format!("find_sdk_um_dir: {e:?}")))?;
    let exe = dir.join("probe.exe");
    let staged = auto_ac_prototype::link::link_object_staged(
        &lld,
        &sdk,
        &objs[0],
        &exe,
        entry,
        &objs[1..],
    )
    .map_err(|e| ProbeError(format!("staged link: {e:?}")))?;
    Ok(staged.tmp_exe)
}

/// Run a probe PE under the contained executor with the frozen run
/// budget, enforcing the output acceptance caps. A run whose outputs
/// exceed the cap is an error, not a truncated success.
pub fn run_pe(exe: &Path) -> Result<PeRun, ProbeError> {
    let (code, out, err) = auto_ac_prototype::link::run_deadlined_split(
        &mut Command::new(exe),
        RUN_DEADLINE,
    )
    .map_err(|e| ProbeError(format!("run {}: {e:?}", exe.display())))?;
    let code = code.ok_or_else(|| {
        ProbeError(format!(
            "{} terminated abnormally (deadline/kill); out={} err={}",
            exe.display(),
            out.trim(),
            err.trim()
        ))
    })?;
    if out.len() > OUTPUT_CAP || err.len() > OUTPUT_CAP {
        return Err(ProbeError(format!(
            "output cap exceeded ({} / {} bytes) — probe request fails, \
             never truncates to green",
            out.len(),
            err.len()
        )));
    }
    Ok(PeRun {
        code,
        stdout: out,
        stderr: err,
    })
}
