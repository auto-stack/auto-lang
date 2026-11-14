//! PLAN-1040: the WSL execution leg — the current required Linux
//! acceptance environment (2026-11-14 user constraint: Windows host +
//! WSL2; T-01 froze `Ubuntu-22.04`, kernel 6.18.58.1-microsoft-standard-
//! WSL2, x86-64). Runs a linked Linux ELF inside the explicitly selected
//! WSL distribution with byte-exact capture, a deadline, and cleanup that
//! kills ONLY this run's own process (never `wsl --shutdown`, never the
//! whole distribution).
//!
//! Channel contract (all frozen by T-01 probes on this host):
//! - `wsl.exe --distribution <d> --exec ...` propagates the exec'd
//!   program's exit status as wsl.exe's own exit code (probe: exit 7 → 7);
//! - stdout/stderr bytes cross the Windows pipes unmodified (UTF-8/NUL
//!   preserved — no shell/PowerShell text pipeline in between);
//! - the Linux-visible path of a Windows file comes from `wslpath -a`
//!   (never a hardcoded /mnt/<letter> assumption);
//! - deadline: kill the wsl.exe host side, then kill the exact Linux PID
//!   recorded by the `sh` wrapper before `exec` (targeted cleanup).

use crate::config::{RUN_BUDGET, STREAM_CAP, WSL_DEFAULT_DISTRO};
use crate::link_driver::PeRun;
use crate::MnError;
use ace_core::link::{run_deadlined_captured, CaptureCaps};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The resolved WSL environment (distribution + verified wslpath of the
/// executable this run will launch).
#[derive(Debug, Clone)]
pub struct WslEnv {
    pub distro: String,
    /// Linux-side absolute path of the executable (`wslpath -a` result).
    pub linux_path: String,
}

/// The explicitly selected WSL distribution (env `AC1040_WSL_DISTRO`
/// override, else the T-01 frozen default). Never auto-installed or
/// switched by this code.
pub fn wsl_distro() -> String {
    std::env::var("AC1040_WSL_DISTRO")
        .ok()
        .filter(|d| !d.trim().is_empty())
        .unwrap_or_else(|| WSL_DEFAULT_DISTRO.to_string())
}

/// Build a `wsl.exe` command for the distribution without MSYS path
/// mangling interference (arguments are passed verbatim).
fn wsl_command(distro: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new("wsl.exe");
    cmd.arg("--distribution").arg(distro);
    cmd
}

/// Resolve the Linux-side path of `win_path` via `wslpath -a` inside the
/// distribution. Fails closed (Infra) when WSL/the distribution is not
/// available — the caller records the environment gap, never a fake pass.
pub fn wsl_resolve_path(win_path: &Path, distro: &str) -> Result<WslEnv, MnError> {
    let win = win_path
        .to_str()
        .ok_or_else(|| MnError::Infra("wsl.path: non-UTF-16 path".into()))?
        .to_string();
    let out = wsl_command(distro)
        .arg("--exec")
        .arg("wslpath")
        .arg("-a")
        .arg(&win)
        .output()
        .map_err(|e| MnError::Infra(format!("wsl.exe spawn: {e}")))?;
    if !out.status.success() {
        return Err(MnError::Infra(format!(
            "wsl.path: distro `{distro}` rejected the query (exit {:?}): {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    let linux_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if linux_path.is_empty() || linux_path.starts_with('/') == false {
        return Err(MnError::Infra(format!(
            "wsl.path: unexpected wslpath output `{linux_path}`"
        )));
    }
    Ok(WslEnv {
        distro: distro.to_string(),
        linux_path,
    })
}

/// One WSL run observation (byte-exact stdout/stderr + the Linux process
/// exit code propagated through wsl.exe).
pub type WslRun = PeRun;

/// Run a Linux ELF inside the WSL distribution with a deadline and capped
/// raw capture. The `sh` wrapper records the child PID in `/tmp` BEFORE
/// `exec`, so a deadline breach kills exactly this run's process; the
/// wrapper/exit-code chain preserves the child's real exit status.
pub fn run_elf_via_wsl(
    exe_path: &Path,
    args: &[String],
    budget: Duration,
    caps: CaptureCaps,
) -> Result<WslRun, MnError> {
    let distro = wsl_distro();
    let env = wsl_resolve_path(exe_path, &distro)?;
    let tag = format!(
        "ac1040-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let pid_file = format!("/tmp/{tag}.pid");
    // sh -c 'echo $$ > "$1"; shift; exec "$@"' sh <pidfile> <exe> [args...]
    // — argv crosses verbatim (no quoting hazards), PID stable across exec.
    let script = "echo $$ > \"$1\"; shift; exec \"$@\"";
    let mut cmd = wsl_command(&distro);
    cmd.arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(script)
        .arg("sh")
        .arg(&pid_file)
        .arg(&env.linux_path);
    for a in args {
        cmd.arg(a);
    }
    let captured = run_deadlined_captured(&mut cmd, budget, caps).map_err(|e| {
        // The executor already killed the wsl.exe side on deadline; also
        // take down the Linux child via the recorded PID (targeted).
        kill_wsl_pid(&distro, &pid_file);
        MnError::Infra(format!("wsl run: {e:?}"))
    })?;
    let _ = wsl_remove(&distro, &pid_file);
    Ok(PeRun {
        exit_code: captured.exit_code,
        stdout: captured.stdout,
        stderr: captured.stderr,
        exe_path: exe_path.to_path_buf(),
    })
}

/// Kill exactly the PID this run recorded (if it still exists).
fn kill_wsl_pid(distro: &str, pid_file: &str) {
    let script = "p=$(cat \"$1\" 2>/dev/null) && kill \"$p\" 2>/dev/null; rm -f \"$1\"";
    let _ = wsl_command(distro)
        .arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(script)
        .arg("sh")
        .arg(pid_file)
        .output();
}

fn wsl_remove(distro: &str, pid_file: &str) -> bool {
    let script = "rm -f \"$1\"";
    wsl_command(distro)
        .arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(script)
        .arg("sh")
        .arg(pid_file)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Convenience: run with the frozen console-lane budget/caps.
pub fn run_elf_console_lane(exe_path: &Path) -> Result<WslRun, MnError> {
    run_elf_via_wsl(
        exe_path,
        &[],
        RUN_BUDGET,
        CaptureCaps {
            stdout_max: STREAM_CAP,
            stderr_max: STREAM_CAP,
        },
    )
}

/// Run an arbitrary Linux binary (e.g. the AV runner) inside the WSL
/// distribution with explicit argv, deadline and capture — the AV-side
/// twin of [`run_elf_via_wsl`].
pub fn run_wsl_argv(
    linux_exe: &str,
    argv: &[String],
    budget: Duration,
    caps: CaptureCaps,
) -> Result<WslRun, MnError> {
    let distro = wsl_distro();
    let env = wsl_resolve_path(Path::new(linux_exe), &distro)?;
    let tag = format!(
        "ac1040av-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    let pid_file = format!("/tmp/{tag}.pid");
    let script = "echo $$ > \"$1\"; shift; exec \"$@\"";
    let mut cmd = wsl_command(&distro);
    cmd.arg("--exec")
        .arg("/bin/sh")
        .arg("-c")
        .arg(script)
        .arg("sh")
        .arg(&pid_file)
        .arg(&env.linux_path);
    for a in argv {
        cmd.arg(a);
    }
    let captured = run_deadlined_captured(&mut cmd, budget, caps).map_err(|e| {
        kill_wsl_pid(&distro, &pid_file);
        MnError::Infra(format!("wsl argv run: {e:?}"))
    })?;
    let _ = wsl_remove(&distro, &pid_file);
    Ok(PeRun {
        exit_code: captured.exit_code,
        stdout: captured.stdout,
        stderr: captured.stderr,
        exe_path: Path::new(linux_exe).to_path_buf(),
    })
}

/// Locate the build tree's musl AV runner if it was cross-built into the
/// given cargo target dir (tests build it once; this only resolves).
pub fn av_runner_path(target_dir: &Path) -> PathBuf {
    target_dir
        .join("x86_64-unknown-linux-musl")
        .join("debug")
        .join("mn-linux-av-runner")
}
