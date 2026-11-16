//! PLAN-976 T-03: controlled OS node execution leg for the TypeScript /
//! JavaScript filesets (`node` type-strip execution of `.ts` entry or plain
//! ESM `.mjs`), using `ace_core::link::run_deadlined_captured`, plus strict
//! engine identity extraction (`ScriptRuntimeIdentity`).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ace_a2x_ts::{ScriptLang, ScriptRuntimeIdentity, A2TS_RUNTIME_VERSION};
use ace_core::link::{run_deadlined_captured, CaptureCaps, CapturedRun};
use ace_core::runtime_contract::sha256_hex;

/// Deadline per real node execution (matches the A2X 30s execution cap).
pub const TS_RUN_DEADLINE: Duration = Duration::from_secs(30);
/// Stream cap per execution leg (1 MiB).
pub const TS_STREAM_CAP: usize = 1 << 20;

/// Infrastructure error locating or launching node. Never a PASS or skip.
#[derive(Debug)]
pub struct TsToolError(pub String);

impl std::fmt::Display for TsToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ts-tool infra: {}", self.0)
    }
}

/// Resolved host node command + engine identity.
#[derive(Clone, Debug)]
pub struct ResolvedNode {
    pub program: PathBuf,
    pub identity: ScriptRuntimeIdentity,
}

fn target_triple() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn candidate_commands() -> Vec<String> {
    if let Ok(explicit) = std::env::var("A2TS_NODE") {
        let trimmed = explicit.trim().to_string();
        if !trimmed.is_empty() {
            return vec![trimmed];
        }
    }
    vec!["node".to_string(), "node.exe".to_string()]
}

/// Resolve the host node executable, version, and binary SHA-256 identity.
///
/// `runtime_sha256` is the emitted runtime module body hash the artifacts
/// bind (`a2x_rt.mjs`); it participates in the identity fingerprint so two
/// runtimes can never collide.
pub fn resolve_node(runtime_sha256: &str) -> Result<ResolvedNode, TsToolError> {
    let mut attempts = Vec::new();
    for prog in candidate_commands() {
        let mut cmd = Command::new(&prog);
        cmd.arg("--version");
        match cmd.output() {
            Ok(out) if out.status.success() => {
                let text = String::from_utf8_lossy(&out.stdout);
                let ver = text.trim().to_string();
                // `process.execPath` gives the absolute engine executable.
                let mut exe_cmd = Command::new(&prog);
                exe_cmd.arg("-e").arg("console.log(process.execPath)");
                let exe_str = match exe_cmd.output() {
                    Ok(o) if o.status.success() => {
                        String::from_utf8_lossy(&o.stdout).trim().to_string()
                    }
                    _ => prog.clone(),
                };
                let exe_path = PathBuf::from(&exe_str);
                let exe_hash = std::fs::read(&exe_path)
                    .map(|b| sha256_hex(&b))
                    .unwrap_or_else(|_| sha256_hex(exe_str.as_bytes()));
                return Ok(ResolvedNode {
                    program: if exe_path.is_file() {
                        exe_path
                    } else {
                        PathBuf::from(prog)
                    },
                    identity: ScriptRuntimeIdentity {
                        engine: "node".to_string(),
                        version: ver,
                        executable_sha256: exe_hash,
                        runtime_version: A2TS_RUNTIME_VERSION.to_string(),
                        runtime_sha256: runtime_sha256.to_string(),
                        target_triple: target_triple(),
                    },
                });
            }
            Ok(out) => attempts.push(format!("{prog}: exit {:?}", out.status.code())),
            Err(e) => attempts.push(format!("{prog}: {e}")),
        }
    }
    Err(TsToolError(format!(
        "node engine unavailable ({})",
        attempts.join("; ")
    )))
}

/// Run `a2x_main.<ts|mjs>` inside `dir` via the contained OS runner.
pub fn run_script_fileset(
    node: &ResolvedNode,
    dir: &Path,
    lang: ScriptLang,
    extra_envs: &[(&str, &str)],
) -> Result<CapturedRun, TsToolError> {
    let main = dir.join(lang.main_file());
    if !main.is_file() {
        return Err(TsToolError(format!(
            "missing generated entrypoint {}",
            main.display()
        )));
    }
    let mut cmd = Command::new(&node.program);
    cmd.arg(lang.main_file()).current_dir(dir);
    for (k, v) in extra_envs {
        cmd.env(k, v);
    }
    run_deadlined_captured(
        &mut cmd,
        TS_RUN_DEADLINE,
        CaptureCaps {
            stdout_max: TS_STREAM_CAP,
            stderr_max: TS_STREAM_CAP,
        },
    )
    .map_err(|e| TsToolError(format!("contained node run: {e:?}")))
}
