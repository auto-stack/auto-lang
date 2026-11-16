//! PLAN-1042: controlled OS Python execution leg (`python3` / `python` / `py -3`)
//! using `ace_core::link::run_deadlined_captured`, plus strict interpreter
//! identity extraction (`PythonRuntimeIdentity`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ace_a2x_python::PythonRuntimeIdentity;
use ace_core::link::{run_deadlined_captured, CaptureCaps, CapturedRun};

/// Deadline per real Python execution (matches A2X 30s execution cap).
pub const PY_RUN_DEADLINE: Duration = Duration::from_secs(30);
/// Stream cap per execution leg (1 MiB).
pub const PY_STREAM_CAP: usize = 1 << 20;
/// PLAN-1034: ALL interpreter candidates share ONE probe deadline (30s,
/// the current execution tier) — not reset per candidate; probe streams
/// are capped at 1 MiB each via the contained runner.
pub const PY_PROBE_DEADLINE: Duration = Duration::from_secs(30);

/// Infrastructure error locating or launching OS Python. Never a PASS or skip.
#[derive(Debug)]
pub struct PyToolError(pub String);

impl std::fmt::Display for PyToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "py-tool infra: {}", self.0)
    }
}

/// Resolved host Python command + interpreter identity.
#[derive(Clone, Debug)]
pub struct ResolvedPython {
    pub program: PathBuf,
    pub pre_args: Vec<String>,
    pub identity: PythonRuntimeIdentity,
}

fn target_triple() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn candidate_commands() -> Vec<(String, Vec<String>)> {
    if let Ok(explicit) = std::env::var("A2PY_PYTHON") {
        let trimmed = explicit.trim().to_string();
        if !trimmed.is_empty() {
            return vec![(trimmed, Vec::new())];
        }
    }
    vec![
        ("python3".to_string(), Vec::new()),
        ("python".to_string(), Vec::new()),
        ("py".to_string(), vec!["-3".to_string()]),
    ]
}

/// Resolve host Python executable, version, and binary SHA-256 identity.
///
/// PLAN-1034: the version probe rides the SAME contained capped runner as
/// the real execution leg (previously an unbounded `Command::output()`).
/// All candidates share ONE 30s probe deadline (remaining time per
/// candidate, never reset); stdout/stderr are capped at 1 MiB each. The
/// interpreter content hash is STREAMED (never caching the whole binary);
/// a read failure is a named Infra error, never a path-string hash.
/// Platform admission follows the contained runner (non-Windows spawn
/// refusal stays a refusal — no support change in this plan).
pub fn resolve_python(runtime_sha256: &str) -> Result<ResolvedPython, PyToolError> {
    let probe_started = std::time::Instant::now();
    let mut attempts = Vec::new();
    for (prog, pre_args) in candidate_commands() {
        let now = std::time::Instant::now();
        let Some(remaining) = PY_PROBE_DEADLINE.checked_sub(now - probe_started) else {
            attempts.push(format!(
                "{prog}: shared probe deadline exhausted after {:?}",
                now - probe_started
            ));
            break;
        };
        let mut cmd = Command::new(&prog);
        cmd.args(&pre_args)
            .arg("-c")
            .arg("import sys; print(sys.executable); print(sys.version.split()[0])");
        let probe = run_deadlined_captured(
            &mut cmd,
            remaining,
            CaptureCaps {
                stdout_max: PY_STREAM_CAP,
                stderr_max: PY_STREAM_CAP,
            },
        );
        match probe {
            Ok(out) if out.exit_code == Some(0) => {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut lines = text.lines();
                let exe_str = lines.next().unwrap_or(&prog).trim().to_string();
                let ver_str = lines.next().unwrap_or("unknown").trim().to_string();
                let exe_path = PathBuf::from(&exe_str);
                let exe_hash = match sha256_file_streaming(&exe_path) {
                    Ok(h) => h,
                    Err(e) => {
                        return Err(PyToolError(format!(
                            "interpreter content hash: read {}: {e}",
                            exe_path.display()
                        )))
                    }
                };
                return Ok(ResolvedPython {
                    program: if exe_path.is_file() {
                        exe_path
                    } else {
                        PathBuf::from(prog)
                    },
                    pre_args: if exe_str.is_empty() {
                        pre_args
                    } else {
                        Vec::new()
                    },
                    identity: PythonRuntimeIdentity {
                        executable: exe_str,
                        version: ver_str,
                        executable_sha256: exe_hash,
                        runtime_sha256: runtime_sha256.to_string(),
                        target_triple: target_triple(),
                    },
                });
            }
            Ok(out) => {
                attempts.push(format!("{prog}: exit {:?}", out.exit_code));
            }
            Err(e) => {
                attempts.push(format!("{prog}: {e:?}"));
            }
        }
    }
    Err(PyToolError(format!(
        "Python interpreter unavailable ({})",
        attempts.join("; ")
    )))
}

/// Streamed SHA-256 of a file (64 KiB chunks; the interpreter binary is
/// never cached whole). Read failures are returned, not substituted.
fn sha256_file_streaming(path: &Path) -> std::io::Result<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut f = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut chunk)?;
        if n == 0 {
            break;
        }
        hasher.update(&chunk[..n]);
    }
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for b in digest {
        use std::fmt::Write;
        let _ = write!(hex, "{b:02x}");
    }
    Ok(hex)
}

/// Run `a2x_main.py` inside `dir` via the contained OS runner.
pub fn run_py_fileset(
    py: &ResolvedPython,
    dir: &Path,
    extra_envs: &[(&str, &str)],
) -> Result<CapturedRun, PyToolError> {
    let main_py = dir.join("a2x_main.py");
    if !main_py.is_file() {
        return Err(PyToolError(format!(
            "missing generated entrypoint {}",
            main_py.display()
        )));
    }
    let mut cmd = Command::new(&py.program);
    cmd.args(&py.pre_args)
        .arg("-I")
        .arg("-B")
        .arg("a2x_main.py")
        .current_dir(dir);
    for (k, v) in extra_envs {
        cmd.env(k, v);
    }
    run_deadlined_captured(
        &mut cmd,
        PY_RUN_DEADLINE,
        CaptureCaps {
            stdout_max: PY_STREAM_CAP,
            stderr_max: PY_STREAM_CAP,
        },
    )
    .map_err(|e| PyToolError(format!("contained python run: {e:?}")))
}

#[cfg(test)]
mod audit1034_py_probe {
    use super::*;

    /// PLAN-1034：显式 A2PY_PYTHON 指向缺失路径 → 全候选耗尽后具名
    /// Infra（PyToolError），不静默回退系统候选。
    #[test]
    fn explicit_missing_interpreter_is_named_infra() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("A2PY_PYTHON");
        std::env::set_var("A2PY_PYTHON", "definitely-missing-python-1034");
        let outcome = resolve_python("runtime-sha");
        match previous {
            Some(v) => std::env::set_var("A2PY_PYTHON", v),
            None => std::env::remove_var("A2PY_PYTHON"),
        }
        let err = outcome.expect_err("missing explicit interpreter must fail");
        assert!(
            err.0.contains("unavailable"),
            "named infra refusal, got: {}",
            err.0
        );
    }

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
}
