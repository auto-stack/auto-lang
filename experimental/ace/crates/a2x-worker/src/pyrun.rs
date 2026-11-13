//! PLAN-1042: controlled OS Python execution leg (`python3` / `python` / `py -3`)
//! using `ace_core::link::run_deadlined_captured`, plus strict interpreter
//! identity extraction (`PythonRuntimeIdentity`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ace_a2x_python::PythonRuntimeIdentity;
use ace_core::link::{run_deadlined_captured, CaptureCaps, CapturedRun};
use ace_core::runtime_contract::sha256_hex;

/// Deadline per real Python execution (matches A2X 30s execution cap).
pub const PY_RUN_DEADLINE: Duration = Duration::from_secs(30);
/// Stream cap per execution leg (1 MiB).
pub const PY_STREAM_CAP: usize = 1 << 20;

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
pub fn resolve_python(runtime_sha256: &str) -> Result<ResolvedPython, PyToolError> {
    let mut attempts = Vec::new();
    for (prog, pre_args) in candidate_commands() {
        let mut cmd = Command::new(&prog);
        cmd.args(&pre_args)
            .arg("-c")
            .arg("import sys; print(sys.executable); print(sys.version.split()[0])");
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut lines = text.lines();
                let exe_str = lines.next().unwrap_or(&prog).trim().to_string();
                let ver_str = lines.next().unwrap_or("unknown").trim().to_string();
                let exe_path = PathBuf::from(&exe_str);
                let exe_hash = std::fs::read(&exe_path)
                    .map(|b| sha256_hex(&b))
                    .unwrap_or_else(|_| sha256_hex(exe_str.as_bytes()));
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
                attempts.push(format!("{prog}: exit {:?}", out.status.code()));
            }
            Err(e) => {
                attempts.push(format!("{prog}: {e}"));
            }
        }
    }
    Err(PyToolError(format!(
        "Python interpreter unavailable ({})",
        attempts.join("; ")
    )))
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
