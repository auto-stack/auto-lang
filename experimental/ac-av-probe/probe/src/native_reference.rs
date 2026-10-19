//! Native cross-check reference (PLAN-798 §5.3).
//!
//! The reference walks the ALREADY-LANDED path — Checked HIR → Cranelift →
//! COFF → PE — and runs the produced executable under the deadline/job
//! executor from ac-core. Expected outputs are INDEPENDENT (fixed
//! fixtures/hand-computed values); comparing AV against native alone is
//! never sufficient, and native exit 70 is the existing trap contract. The
//! reference never generates its expectations from the new MIR/ABC.
//!
//! Budgets (frozen §3): build ≤60s, run ≤5s, total ≤70s per request —
//! enforced with the same deadline machinery ac-core uses.

use auto_ac_prototype::link;
use auto_ac_prototype::native;
use auto_ac_prototype::verify::CheckedModule;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const NATIVE_BUILD_BUDGET: Duration = Duration::from_secs(60);
pub const NATIVE_RUN_BUDGET: Duration = Duration::from_secs(5);
pub const NATIVE_TOTAL_BUDGET: Duration = Duration::from_secs(70);

/// Exit code the landed native path uses for language traps.
pub const TRAP_EXIT_CODE: i32 = 70;

#[derive(Clone, Debug)]
pub struct NativeRun {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub obj_path: PathBuf,
    pub exe_path: PathBuf,
    pub receipt: PathBuf,
}

#[derive(Clone, Debug)]
pub enum NativeError {
    Toolchain(String),
    Build(String),
    Link(String),
    Run(String),
    Budget(&'static str),
}

impl std::fmt::Display for NativeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NativeError::Toolchain(m) => write!(f, "native toolchain: {}", m),
            NativeError::Build(m) => write!(f, "native build: {}", m),
            NativeError::Link(m) => write!(f, "native link: {}", m),
            NativeError::Run(m) => write!(f, "native run: {}", m),
            NativeError::Budget(w) => write!(f, "native budget: {}", w),
        }
    }
}

/// Build + link + run one module's entry natively. `trace_lib` adds the
/// PLAN-770 trace support object (mark_a/mark_b) for trace parity runs.
///
/// F798-R3-11: the run stage uses the landed RAW-BYTES capped executor
/// (`run_deadlined_captured`) with the frozen 1 MiB-per-stream caps — the
/// legacy `run_deadlined_split` wrapper is display-only and silently maps
/// to `CaptureCaps::none()`. The build stages share ONE absolute budget
/// clock started at entry: each stage is refused when the remaining
/// budget is already spent (pre-check) and a post-check attributes any
/// overrun; the run stage's deadline is the REMAINING total budget
/// (bounded by the per-run budget), so a link stage that consumed the
/// build clock cannot borrow run time.
pub fn build_and_run(
    checked: &CheckedModule,
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<NativeRun, NativeError> {
    build_and_run_with_caps(
        checked,
        entry,
        workdir,
        trace_lib,
        NATIVE_STREAM_CAP,
        NATIVE_STREAM_CAP,
    )
}

/// Frozen per-stream output cap (§3: stdout/stderr each ≤ 1 MiB).
pub const NATIVE_STREAM_CAP: usize = 1024 * 1024;

/// The budget-carrying core (caps injectable for the failure counter-test).
#[allow(clippy::too_many_arguments)]
pub fn build_and_run_with_caps(
    checked: &CheckedModule,
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
    stdout_cap: usize,
    stderr_cap: usize,
) -> Result<NativeRun, NativeError> {
    let started = std::time::Instant::now();
    let build_deadline = NATIVE_BUILD_BUDGET;
    // Test-only budget fault seam (F798-R6-07 counter-control): simulates
    // the pre-link stages having already consumed this much of the shared
    // build clock, so the stage admissions refuse DETERMINISTICALLY
    // without a real 60s stall. Never set in production paths.
    let fake_stage_elapsed = || {
        std::time::Duration::from_millis(
            std::env::var("AV798_NATIVE_FAKE_ELAPSED_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0u64),
        )
    };
    let elapsed = || started.elapsed() + fake_stage_elapsed();
    let stage_precheck = |stage: &'static str| -> Result<(), NativeError> {
        if elapsed() > build_deadline {
            Err(NativeError::Budget(stage))
        } else {
            Ok(())
        }
    };
    std::fs::create_dir_all(workdir).map_err(|e| NativeError::Build(e.to_string()))?;
    let obj = workdir.join("module.obj");
    let tmp_obj = workdir.join(".tmp-module.obj");
    let exe = workdir.join("module.exe");

    // Lower to a STAGED object; publish moves it into place transactionally.
    stage_precheck("lower overran the 60s build budget")?;
    let sel =
        native::find_entry(checked, entry).map_err(|ds| NativeError::Build(format!("{:?}", ds)))?;
    let bytes = native::lower_object(checked, &sel, &native::RuntimeInjection::default())
        .map_err(|ds| NativeError::Build(format!("{:?}", ds)))?;
    std::fs::write(&tmp_obj, &bytes).map_err(|e| NativeError::Build(e.to_string()))?;

    // Link (staged + publish, reusing ac-core's transactional machinery).
    stage_precheck("link start overran the 60s build budget")?;
    let lld = link::find_rust_lld().map_err(|d| NativeError::Toolchain(d.message))?;
    let um = link::find_sdk_um_dir().map_err(|d| NativeError::Toolchain(d.message))?;
    let mut libs: Vec<PathBuf> = Vec::new();
    if let Some(t) = trace_lib {
        libs.push(t.to_path_buf());
    }
    let staged = link::link_object_staged(&lld, &um, &tmp_obj, &exe, &sel.symbol, &libs)
        .map_err(|d| NativeError::Link(d.message))?;
    let receipt = link::publish_artifacts(&staged, &tmp_obj, &obj)
        .map_err(|d| NativeError::Link(d.message))?;
    if elapsed() > NATIVE_BUILD_BUDGET {
        return Err(NativeError::Budget("build exceeded 60s"));
    }

    // Run under the contained executor with RAW capped streams; the
    // deadline is the REMAINING total budget (≤ the per-run budget).
    stage_precheck("run start overran the total 70s budget")?;
    let total_remaining = NATIVE_TOTAL_BUDGET
        .checked_sub(elapsed())
        .unwrap_or(Duration::ZERO);
    let run_deadline = NATIVE_RUN_BUDGET.min(total_remaining);
    let mut cmd = std::process::Command::new(&exe);
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let run = link::run_deadlined_captured(
        &mut cmd,
        run_deadline,
        link::CaptureCaps {
            stdout_max: stdout_cap,
            stderr_max: stderr_cap,
        },
    )
    .map_err(|e| NativeError::Run(format!("{:?}", e)))?;
    if elapsed() > NATIVE_TOTAL_BUDGET {
        return Err(NativeError::Budget("total exceeded 70s"));
    }
    let exit_code = run
        .exit_code
        .ok_or_else(|| NativeError::Run("process terminated abnormally".into()))?;
    if run.stdout.len() > stdout_cap || run.stderr.len() > stderr_cap {
        return Err(NativeError::Run(format!(
            "native output exceeded the frozen {}-byte per-stream cap",
            NATIVE_STREAM_CAP
        )));
    }
    Ok(NativeRun {
        exit_code,
        stdout: String::from_utf8_lossy(&run.stdout).to_string(),
        stderr: String::from_utf8_lossy(&run.stderr).to_string(),
        obj_path: obj,
        exe_path: exe,
        receipt: receipt.receipt_path,
    })
}

/// The trace support library shipped with ac-core (built by its test
/// harness); `None` when not yet built — callers treat that as infra-blocked
/// for trace parity, never as a pass.
pub fn trace_support_lib(repo_root: &Path) -> Option<PathBuf> {
    let p = repo_root.join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    p.is_file().then_some(p)
}
