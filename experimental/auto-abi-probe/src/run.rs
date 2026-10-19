//! Probe execution: scratch dirs + running PEs under the contained
//! executor with the frozen budgets (plan §3; contract-decision §6).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Frozen T-01/T-03 probe run budgets (plan §3). The 70s total budget
/// is enforced by the per-request harness at the caller; this module
/// owns the run leg.
pub const RUN_DEADLINE: Duration = Duration::from_secs(5);
/// stdout/stderr acceptance caps: outputs beyond 1 MiB each are a FAILED
/// probe request, never truncated into a green result.
pub const OUTPUT_CAP: usize = 1024 * 1024;

pub use crate::link::ProbeError;

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
/// run's leftovers. Case names must be unique per concurrent test.
pub fn scratch(case: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("autoabi-t01")
        .join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("probe scratch dir");
    dir
}

/// Run a probe PE under the contained executor with the frozen run
/// budget, enforcing the output acceptance caps. A run whose outputs
/// exceed the cap is an error, not a truncated success.
pub fn run_pe(exe: &Path) -> Result<PeRun, ProbeError> {
    let (code, out, err) =
        auto_ac_prototype::link::run_deadlined_split(&mut Command::new(exe), RUN_DEADLINE)
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
