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
        .join("target/autoabi-t01")
        .join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("probe scratch dir");
    dir
}

/// Run a probe PE under the contained executor with the frozen run
/// budget. F-797-R4-10: this consumes the merged RAW CAPPED executor
/// (PLAN-788 T-03) — a cap TERMINATES the tree the moment a reader
/// passes it (never a truncated success) and the bytes are the exact
/// raw stream (invalid UTF-8 included); lossy rendering is display-only.
pub fn run_pe(exe: &Path) -> Result<PeRun, ProbeError> {
    let run = ace_core::link::run_deadlined_captured(
        &mut Command::new(exe),
        RUN_DEADLINE,
        ace_core::link::CaptureCaps {
            stdout_max: OUTPUT_CAP,
            stderr_max: OUTPUT_CAP,
        },
    )
    .map_err(|e| ProbeError(format!("run {}: {e:?}", exe.display())))?;
    let code = run.exit_code.ok_or_else(|| {
        ProbeError(format!(
            "{} terminated abnormally (deadline/kill); out={} err={}",
            exe.display(),
            String::from_utf8_lossy(&run.stdout).trim(),
            String::from_utf8_lossy(&run.stderr).trim()
        ))
    })?;
    Ok(PeRun {
        code,
        stdout: String::from_utf8_lossy(&run.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&run.stderr).into_owned(),
    })
}
