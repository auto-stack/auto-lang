//! Native cross-check reference (PLAN-798 §5.3), FULLY SUPERVISED
//! (F798-R7-01).
//!
//! The reference walks the ALREADY-LANDED path — Checked HIR → Cranelift →
//! COFF → PE — and runs the produced executable. Expected outputs are
//! INDEPENDENT (fixed fixtures/hand-computed values); comparing AV against
//! native alone is never sufficient, and native exit 70 is the existing
//! trap contract. The reference never generates its expectations from the
//! new MIR/ABC.
//!
//! Supervision model (the frozen §3 budgets are ENFORCED, not observed):
//! each stage runs inside the plan's OWN bounded worker (`av_probe_worker
//! native-build` / `native-run`), and the parent drives every call through
//! ac-core's contained executor with a REMAINING-time deadline measured
//! from the request start — a linker that stalls past the remaining build
//! budget is KILLED with its whole tree (the in-flight overrun case the
//! stage-precheck-only scheme could not cover), never left to its own
//! internal 60s. Budgets: build ≤60s (remaining-time killed), run ≤5s,
//! total ≤70s — all absolute from request start.

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

/// Test-only budget fault seam (F798-R6-07/R7-01 counter-controls):
/// simulates the pre-build stages having already consumed this much of
/// the shared clock, so deadline computation refuses deterministically
/// without a real 60s stall. Never set in production paths.
fn fake_stage_elapsed() -> Duration {
    Duration::from_millis(
        std::env::var("AV798_NATIVE_FAKE_ELAPSED_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0u64),
    )
}

// ---------------------------------------------------------------------------
// Stage bodies (shared with the bounded worker's helper subcommands)
// ---------------------------------------------------------------------------

/// The build stages proper (find entry → lower → link → publish). Runs
/// UNSUPERVISED — only the worker's `native-build` subcommand (which the
/// parent kills at the remaining-budget deadline) calls this.
pub fn build_stages(
    checked: &CheckedModule,
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    std::fs::create_dir_all(workdir).map_err(|e| format!("build: {}", e))?;
    let obj = workdir.join("module.obj");
    let tmp_obj = workdir.join(".tmp-module.obj");
    let exe = workdir.join("module.exe");

    let sel = native::find_entry(checked, entry).map_err(|ds| format!("build: {:?}", ds))?;
    let bytes = native::lower_object(checked, &sel, &native::RuntimeInjection::default())
        .map_err(|ds| format!("build: {:?}", ds))?;
    std::fs::write(&tmp_obj, &bytes).map_err(|e| format!("build: {}", e))?;

    let lld = link::find_rust_lld().map_err(|d| format!("toolchain: {}", d.message))?;
    let um = link::find_sdk_um_dir().map_err(|d| format!("toolchain: {}", d.message))?;
    let mut libs: Vec<PathBuf> = Vec::new();
    if let Some(t) = trace_lib {
        libs.push(t.to_path_buf());
    }
    let staged = link::link_object_staged(&lld, &um, &tmp_obj, &exe, &sel.symbol, &libs)
        .map_err(|d| format!("link: {}", d.message))?;
    let receipt = link::publish_artifacts(&staged, &tmp_obj, &obj)
        .map_err(|d| format!("link: {}", d.message))?;
    Ok((obj, exe, receipt.receipt_path))
}

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
    let run = link::run_deadlined_captured(
        &mut cmd,
        budget,
        link::CaptureCaps {
            stdout_max: stdout_cap,
            stderr_max: stderr_cap,
        },
    )
    .map_err(|e| format!("run: {:?}", e))?;
    Ok((run.exit_code, run.stdout, run.stderr))
}

// ---------------------------------------------------------------------------
// Supervised entry points
// ---------------------------------------------------------------------------

/// Locate (and on first use build) the probe worker binary that hosts the
/// `native-build` / `native-run` helper subcommands. Env override first;
/// otherwise the probe workspace's release target.
pub fn native_worker_binary() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var("AV798_NATIVE_WORKER") {
        let p = PathBuf::from(&p);
        if p.is_file() {
            return Ok(p);
        }
        return Err(format!(
            "AV798_NATIVE_WORKER points at a missing file: {}",
            p.display()
        ));
    }
    let root = probe_workspace_root()?;
    let exe = root.join("target").join("release").join(worker_exe_name());
    if !exe.is_file() {
        bounded_worker_build(&root, &exe)?;
    }
    if exe.is_file() {
        // F798-R9-02: bind the EXISTING binary's functional identity — it
        // must execute and expose this plan's helper subcommands (the
        // usage banner lists `native-build`/`native-run`). A stale or
        // foreign binary at the expected path fails here, bounded.
        let mut probe = std::process::Command::new(&exe);
        probe
            .arg("native-build")
            .stdout(std::process::Stdio::piped());
        let run = link::run_deadlined_captured(
            &mut probe,
            Duration::from_secs(10),
            link::CaptureCaps {
                stdout_max: NATIVE_STREAM_CAP,
                stderr_max: NATIVE_STREAM_CAP,
            },
        )
        .map_err(|e| match e {
            link::SubprocError::Deadline | link::SubprocError::DeadlineCollect => {
                "worker identity probe timed out".to_string()
            }
            other => format!("worker identity probe: {:?}", other),
        })?;
        let usage = String::from_utf8_lossy(&run.stderr);
        if !usage.contains("native-build") || !usage.contains("native-run") {
            return Err(format!(
                "worker at {} does not expose the native helper subcommands (stale binary?)",
                exe.display()
            ));
        }
        Ok(exe)
    } else {
        Err(format!("worker binary missing at {}", exe.display()))
    }
}

/// Preparation wall for the FIRST worker build (F798-R8-03): the plan
/// forbids unbounded launchers, and cold tool preparation must itself be
/// bounded and killed on overrun — even though it is metered SEPARATELY
/// from (and before) the supervised request budgets.
pub const NATIVE_WORKER_PREP_WALL: Duration = Duration::from_secs(300);

/// The bounded first build (single absolute wall, capped output, killed on
/// overrun). Test seams: AV798_NATIVE_PREP_WALL_MS shortens the wall and
/// AV798_NATIVE_PREP_CARGO redirects the build command (fault injection
/// only — never set in production paths).
pub fn bounded_worker_build(root: &Path, exe: &Path) -> Result<(), String> {
    let wall = Duration::from_millis(
        std::env::var("AV798_NATIVE_PREP_WALL_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(NATIVE_WORKER_PREP_WALL.as_millis() as u64),
    );
    let cargo =
        std::env::var("AV798_NATIVE_PREP_CARGO").unwrap_or_else(|_| env!("CARGO").to_string());
    // F798-R9-02: the preparation runs through ac-core's CONTAINED
    // executor — suspended start + job containment (the whole subtree is
    // killed at the wall), BOTH streams drained under hard caps (an
    // over-cap output is a failure, never a truncated success; a large
    // stdout cannot backpressure into a false timeout), one absolute
    // deadline. No private spawn/supervise plumbing.
    let mut cmd = std::process::Command::new(&cargo);
    cmd.args(["build", "--release", "--bin", "av_probe_worker"])
        .current_dir(root)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let run = link::run_deadlined_captured(
        &mut cmd,
        wall,
        link::CaptureCaps {
            stdout_max: NATIVE_STREAM_CAP,
            stderr_max: NATIVE_STREAM_CAP,
        },
    )
    .map_err(|e| match e {
        link::SubprocError::Deadline | link::SubprocError::DeadlineCollect => format!(
            "native worker preparation exceeded its {:?} wall (tree killed)",
            wall
        ),
        link::SubprocError::OutputCapExceeded { .. } => {
            "native worker preparation output exceeded the stream cap".to_string()
        }
        other => format!("native worker preparation spawn: {:?}", other),
    })?;
    let status = run
        .exit_code
        .ok_or_else(|| "native worker preparation terminated abnormally".to_string())?;
    if status != 0 {
        return Err(format!(
            "worker build failed:
{}",
            String::from_utf8_lossy(&run.stderr)
        ));
    }
    if !exe.is_file() {
        return Err(format!(
            "worker build produced no binary at {}",
            exe.display()
        ));
    }
    Ok(())
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

#[cfg(windows)]
fn worker_exe_name() -> &'static str {
    "av_probe_worker.exe"
}
#[cfg(not(windows))]
fn worker_exe_name() -> &'static str {
    "av_probe_worker"
}

/// Build + link + run one module's entry natively under the FROZEN budgets,
/// with every stage inside the contained worker and the parent enforcing
/// REMAINING-time deadlines from request start (F798-R7-01).
pub fn build_and_run(
    src: &str,
    caps: &[String],
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<NativeRun, NativeError> {
    build_and_run_with_caps(
        src,
        caps,
        entry,
        workdir,
        trace_lib,
        NATIVE_STREAM_CAP,
        NATIVE_STREAM_CAP,
    )
}

/// PLAN-820 F820-R1-03: the SAME supervised native comparison under the
/// RECORD source domain — the first case's independent oracle (§5.2: the
/// same record source through the old HIR-native path vs the fixed manual
/// expected value). Identical budgets/supervision to [`build_and_run`];
/// only the worker's compile profile differs (`--profile record`).
pub fn build_and_run_record(
    src: &str,
    caps: &[String],
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
) -> Result<NativeRun, NativeError> {
    build_and_run_profile(
        src,
        caps,
        entry,
        workdir,
        trace_lib,
        NATIVE_STREAM_CAP,
        NATIVE_STREAM_CAP,
        true,
    )
}

/// Frozen per-stream output cap (§3: stdout/stderr each ≤ 1 MiB).
pub const NATIVE_STREAM_CAP: usize = 1024 * 1024;

#[allow(clippy::too_many_arguments)]
pub fn build_and_run_with_caps(
    src: &str,
    caps: &[String],
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
    stdout_cap: usize,
    stderr_cap: usize,
) -> Result<NativeRun, NativeError> {
    build_and_run_profile(
        src, caps, entry, workdir, trace_lib, stdout_cap, stderr_cap, false,
    )
}

#[allow(clippy::too_many_arguments)]
fn build_and_run_profile(
    src: &str,
    caps: &[String],
    entry: &str,
    workdir: &Path,
    trace_lib: Option<&Path>,
    stdout_cap: usize,
    stderr_cap: usize,
    record: bool,
) -> Result<NativeRun, NativeError> {
    // F798-R8-03: worker preparation happens BEFORE the request clock,
    // under its OWN bounded wall — the supervised budgets below measure
    // only the actual native request, never cold tool preparation.
    let worker = native_worker_binary().map_err(NativeError::Toolchain)?;
    let started = std::time::Instant::now();
    let elapsed = || started.elapsed() + fake_stage_elapsed();
    std::fs::create_dir_all(workdir).map_err(|e| NativeError::Build(e.to_string()))?;
    let src_file = workdir.join("native-src.at");
    std::fs::write(&src_file, src).map_err(|e| NativeError::Build(e.to_string()))?;

    // ---- Phase 1: build, inside the worker, killed at the REMAINING
    // build budget (the frozen 60s measured from request start).
    if elapsed() > NATIVE_BUILD_BUDGET {
        return Err(NativeError::Budget(
            "build budget already consumed before the build stage",
        ));
    }
    let build_remaining = NATIVE_BUILD_BUDGET - elapsed();
    let mut cmd = std::process::Command::new(&worker);
    cmd.args(["native-build", "--src-file"])
        .arg(&src_file)
        .args(["--entry", entry])
        .arg("--workdir")
        .arg(workdir);
    if let Some(t) = trace_lib {
        cmd.arg("--trace-lib").arg(t);
    }
    if !caps.is_empty() {
        cmd.arg("--caps").arg(caps.join(","));
    }
    if record {
        cmd.arg("--profile").arg("record");
    }
    let built = supervise(&mut cmd, build_remaining, NATIVE_STREAM_CAP)
        .map_err(|e| map_deadline(e, "build exceeded the remaining 60s budget (tree killed)"))?;
    let bj: serde_json::Value = serde_json::from_slice(&built.stdout)
        .map_err(|e| NativeError::Build(format!("native-build response: {}", e)))?;
    if !bj["ok"].as_bool().unwrap_or(false) {
        return Err(map_stage_error(bj["error"].as_str().unwrap_or("unknown")));
    }
    let obj_path = PathBuf::from(bj["obj"].as_str().unwrap_or_default());
    let exe_path = PathBuf::from(bj["exe"].as_str().unwrap_or_default());
    let receipt = PathBuf::from(bj["receipt"].as_str().unwrap_or_default());

    // ---- Phase 2: run, inside the worker, killed at the remaining
    // total budget (bounded by the per-run 5s).
    let total_remaining = NATIVE_TOTAL_BUDGET
        .checked_sub(elapsed())
        .unwrap_or(Duration::ZERO);
    let run_deadline = NATIVE_RUN_BUDGET.min(total_remaining);
    if run_deadline.is_zero() {
        return Err(NativeError::Budget("no total budget left for the run"));
    }
    let mut cmd = std::process::Command::new(&worker);
    cmd.args(["native-run", "--exe"])
        .arg(&exe_path)
        .arg("--budget-ms")
        .arg(run_deadline.as_millis().to_string())
        .arg("--stdout-cap")
        .arg(stdout_cap.to_string())
        .arg("--stderr-cap")
        .arg(stderr_cap.to_string());
    let ran = supervise(
        &mut cmd,
        run_deadline + Duration::from_millis(500),
        NATIVE_STREAM_CAP,
    )
    .map_err(|e| {
        map_deadline(
            e,
            "run exceeded the remaining run/total budget (tree killed)",
        )
    })?;
    let rj: serde_json::Value = serde_json::from_slice(&ran.stdout)
        .map_err(|e| NativeError::Run(format!("native-run response: {}", e)))?;
    if !rj["ok"].as_bool().unwrap_or(false) {
        let m = rj["error"].as_str().unwrap_or("unknown");
        return Err(NativeError::Run(m.to_string()));
    }
    let exit_code = rj["exit_code"]
        .as_i64()
        .ok_or_else(|| NativeError::Run("process terminated abnormally".to_string()))?
        as i32;
    use base64::Engine as _;
    let stdout = String::from_utf8_lossy(
        &base64::engine::general_purpose::STANDARD
            .decode(rj["stdout_b64"].as_str().unwrap_or_default())
            .unwrap_or_default(),
    )
    .to_string();
    let stderr = String::from_utf8_lossy(
        &base64::engine::general_purpose::STANDARD
            .decode(rj["stderr_b64"].as_str().unwrap_or_default())
            .unwrap_or_default(),
    )
    .to_string();
    Ok(NativeRun {
        exit_code,
        stdout,
        stderr,
        obj_path,
        exe_path,
        receipt,
    })
}

/// One supervised worker call: contained executor, single absolute
/// deadline, capped output (the helper prints one JSON line).
fn supervise(
    cmd: &mut std::process::Command,
    deadline: Duration,
    cap: usize,
) -> Result<link::CapturedRun, link::SubprocError> {
    link::run_deadlined_captured(
        cmd,
        deadline,
        link::CaptureCaps {
            stdout_max: cap,
            stderr_max: cap,
        },
    )
}

fn map_deadline(e: link::SubprocError, what: &'static str) -> NativeError {
    match e {
        link::SubprocError::Deadline | link::SubprocError::DeadlineCollect => {
            NativeError::Budget(what)
        }
        other => NativeError::Toolchain(format!("{:?}", other)),
    }
}

fn map_stage_error(m: &str) -> NativeError {
    if let Some(rest) = m.strip_prefix("toolchain: ") {
        NativeError::Toolchain(rest.to_string())
    } else if let Some(rest) = m.strip_prefix("link: ") {
        NativeError::Link(rest.to_string())
    } else if let Some(rest) = m.strip_prefix("run: ") {
        NativeError::Run(rest.to_string())
    } else {
        NativeError::Build(m.trim_start_matches("build: ").to_string())
    }
}

/// The trace support library shipped with ac-core (built by its test
/// harness); `None` when not yet built — callers treat that as infra-blocked
/// for trace parity, never as a pass.
pub fn trace_support_lib(repo_root: &Path) -> Option<PathBuf> {
    let p = repo_root.join("experimental/ac-core/test-support/target/debug/ac_trace_support.lib");
    p.is_file().then_some(p)
}
