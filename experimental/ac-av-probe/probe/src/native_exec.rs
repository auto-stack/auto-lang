//! PLAN-871 T-06: the CONTAINED run executor of the native reference chain,
//! split out before the old HIR→CLIF lowering retires. Executing an
//! already-built PE under a deadline with capped raw-byte capture is NOT
//! part of the lowering — the probe worker and the CTE/probe harnesses keep
//! this primitive.

use std::path::Path;
use std::time::Duration;

/// Frozen per-stream output cap (§3: stdout/stderr each ≤ 1 MiB).
pub const NATIVE_STREAM_CAP: usize = 1024 * 1024;

/// Frozen run budget of the reference chain (§3: run ≤5s).
pub const NATIVE_RUN_BUDGET: Duration = Duration::from_secs(5);

/// The run stage proper (contained, capped). Only the worker's `native-run`
/// subcommand — itself under the parent's remaining-total deadline — calls
/// this with the passed budget and per-stream caps.
pub fn run_stage(
    exe: &Path,
    budget: Duration,
    stdout_cap: usize,
    stderr_cap: usize,
) -> Result<(Option<i32>, Vec<u8>, Vec<u8>), String> {
    let mut cmd = std::process::Command::new(exe);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let run = auto_ac_prototype::link::run_deadlined_captured(
        &mut cmd,
        budget,
        auto_ac_prototype::link::CaptureCaps {
            stdout_max: stdout_cap,
            stderr_max: stderr_cap,
        },
    )
    .map_err(|e| format!("run: {:?}", e))?;
    Ok((run.exit_code, run.stdout, run.stderr))
}
