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
pub fn build_and_run(
    checked: &CheckedModule,
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<NativeRun, NativeError> {
    let started = std::time::Instant::now();
    std::fs::create_dir_all(workdir).map_err(|e| NativeError::Build(e.to_string()))?;
    let obj = workdir.join("module.obj");
    let tmp_obj = workdir.join(".tmp-module.obj");
    let exe = workdir.join("module.exe");

    // Lower to a STAGED object; publish moves it into place transactionally.
    let sel = native::find_entry(checked, entry)
        .map_err(|ds| NativeError::Build(format!("{:?}", ds)))?;
    let bytes = native::lower_object(checked, &sel, &native::RuntimeInjection::default())
        .map_err(|ds| NativeError::Build(format!("{:?}", ds)))?;
    std::fs::write(&tmp_obj, &bytes).map_err(|e| NativeError::Build(e.to_string()))?;

    // Link (staged + publish, reusing ac-core's transactional machinery).
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
    if started.elapsed() > NATIVE_BUILD_BUDGET {
        return Err(NativeError::Budget("build exceeded 60s"));
    }

    // Run with output capture under the contained executor.
    let mut cmd = std::process::Command::new(&exe);
    let (code, stdout, stderr) =
        link::run_deadlined_split(&mut cmd, NATIVE_RUN_BUDGET).map_err(|e| {
            NativeError::Run(format!("{:?}", e))
        })?;
    if started.elapsed() > NATIVE_TOTAL_BUDGET {
        return Err(NativeError::Budget("total exceeded 70s"));
    }
    let exit_code = code.ok_or_else(|| NativeError::Run("process terminated abnormally".into()))?;
    Ok(NativeRun {
        exit_code,
        stdout,
        stderr,
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
