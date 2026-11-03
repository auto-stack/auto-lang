//! PLAN-921 CA-A (T-05): the controlled MSVC C11 compile+run leg.
//!
//! Toolchain resolution follows the repo's frozen order (the PLAN-797
//! `abi::ctool` locator): `AUTOABI_MSVC_CL` env → vswhere → highest
//! MSVC toolset version. A missing tool is an INFRA error, never a skip.
//!
//! The compile/link runs the REAL `cl.exe` with `/std:c11 /TC` under the
//! ac-core contained executor (CREATE_SUSPENDED + Job subtree, single
//! monotonic deadline, 1 MiB stream caps). The CRT header/lib closure
//! (stdio/inttypes/stdbool + the link step) needs the full MSVC+SDK
//! environment, which `vcvars64.bat` provides; the vcvars path is DERIVED
//! from the resolved cl.exe location (BuildTools or Community — no
//! hardcoded edition), overridable via `A2C_VCVARS64`. `vcvars` itself
//! only sets the environment — the compiler process is the same cl.exe
//! resolved above, still contained and deadlined.
//!
//! Error classification (plan §5.1): a failed compile is an INFRA fact
//! carrying the compiler's diagnostics; it can never stand in for an Auto
//! diagnostic or trap.

use ace_core::link::{run_deadlined_captured, CaptureCaps, CapturedRun};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// Compile deadline (the 852 budget: 120s compile).
pub const C_COMPILE_DEADLINE: Duration = Duration::from_secs(120);
/// Run deadline (the 852 budget: 30s run).
pub const C_RUN_DEADLINE: Duration = Duration::from_secs(30);
/// Stream cap per leg (1 MiB).
pub const C_STREAM_CAP: usize = 1 << 20;

/// The frozen MSVC C11 flags of this channel (part of every tool
/// identity/cert binding).
pub const C_FLAGS: [&str; 7] = ["-nologo", "-std:c11", "-TC", "-W4", "-WX", "-O2", "-MT"];

/// A controlled infra failure (missing tool / compiler error). Never an
/// Auto-level trap.
#[derive(Debug)]
pub struct CToolError(pub String);

impl std::fmt::Display for CToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "c-tool infra: {}", self.0)
    }
}

/// Resolve the cl.exe path (env → vswhere → highest toolset; the PLAN-797
/// locator, reused verbatim). A missing tool is an infra error.
pub fn cl_exe() -> Result<PathBuf, CToolError> {
    let tool = ace_abi::ctool::find_cl().map_err(|e| CToolError(format!("{e:?}")))?;
    Ok(tool.cl_exe.clone())
}

/// The full MSVC/SDK toolchain environment for the C channel: cl path +
/// the derived vcvars64 path + the frozen flags. This is the identity the
/// certification binds.
pub struct CToolchain {
    pub cl_exe: PathBuf,
    pub cl_version: String,
    pub cl_hash: String,
    pub vcvars64: PathBuf,
}

/// Resolve the full toolchain (cl via the frozen locator; vcvars64
/// derived from the cl location, or `A2C_VCVARS64`).
pub fn resolve_toolchain() -> Result<CToolchain, CToolError> {
    let tool = ace_abi::ctool::find_cl().map_err(|e| CToolError(format!("{e:?}")))?;
    let vcvars64 = match std::env::var("A2C_VCVARS64") {
        Ok(p) if !p.is_empty() => PathBuf::from(p),
        _ => {
            // cl = <VS>/VC/Tools/MSVC/<v>/bin/Hostx64/x64/cl.exe
            // ancestors() includes the file itself: nth(4) = the MSVC
            // toolset dir (the abi::ctool convention), nth(7) = <VS>/VC
            let vc = tool
                .cl_exe
                .ancestors()
                .nth(7)
                .ok_or_else(|| CToolError("cannot derive VC dir from cl path".into()))?
                .to_path_buf();
            let p = vc.join("Auxiliary").join("Build").join("vcvars64.bat");
            if !p.is_file() {
                return Err(CToolError(format!(
                    "vcvars64.bat not found at {} (set A2C_VCVARS64 to override)",
                    p.display()
                )));
            }
            p
        }
    };
    Ok(CToolchain {
        cl_exe: tool.cl_exe.clone(),
        cl_version: tool.version.clone(),
        cl_hash: tool.cl_hash.clone(),
        vcvars64,
    })
}

/// Run `cmd /D /C call vcvars64 >nul && <argv...> 2>&1` under the
/// contained executor. vcvars only seeds INCLUDE/LIB; the spawned
/// compiler is still job-contained and deadlined.
fn run_via_vcvars(
    tc: &CToolchain,
    argv: &[String],
    cwd: &Path,
    deadline: Duration,
) -> Result<CapturedRun, CToolError> {
    let mut line = String::new();
    line.push_str(&format!("call \"{}\" >nul && ", tc.vcvars64.display()));
    line.push_str(&format!("\"{}\"", tc.cl_exe.display()));
    for a in argv {
        line.push(' ');
        // quote args that contain spaces (paths); the frozen flags and
        // plain file names pass through untouched
        if a.contains(' ') {
            line.push_str(&format!("\"{a}\""));
        } else {
            line.push_str(a);
        }
    }
    let mut cmd = Command::new("cmd.exe");
    cmd.arg("/D").arg("/C");
    // the whole `call ... && cl ...` line must reach cmd VERBATIM — the
    // std default quoting would double-wrap it on Windows
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.raw_arg(&line);
    }
    #[cfg(not(windows))]
    {
        cmd.arg(&line);
    }
    cmd.current_dir(cwd);
    run_deadlined_captured(
        &mut cmd,
        deadline,
        CaptureCaps {
            stdout_max: C_STREAM_CAP,
            stderr_max: C_STREAM_CAP,
        },
    )
    .map_err(|e| CToolError(format!("contained spawn: {e:?}")))
}

/// The C toolchain identity for certification binding (cl banner + hash
/// + vcvars content hash + frozen flags).
pub fn toolchain_identity() -> Result<ace_a2x_c::cert::CToolchainIdentity, CToolError> {
    let tc = resolve_toolchain()?;
    let vcvars_bytes =
        std::fs::read(&tc.vcvars64).map_err(|e| CToolError(format!("read vcvars: {e}")))?;
    Ok(ace_a2x_c::cert::CToolchainIdentity {
        cl_version: tc.cl_version.clone(),
        cl_hash: tc.cl_hash.clone(),
        vcvars_sha256: ace_a2x_core::sha256::sha256_hex(&vcvars_bytes),
        flags: C_FLAGS.iter().map(|s| s.to_string()).collect(),
    })
}

/// Compile ONE standalone C source (the hand-written certification
/// reference) into an exe with the same frozen flags + contained
/// executor — its own translation unit, no generated inputs.
pub fn compile_c_standalone(src: &Path, exe: &Path) -> Result<(), CToolError> {
    let tc = resolve_toolchain()?;
    let mut argv: Vec<String> = C_FLAGS.iter().map(|s| s.to_string()).collect();
    argv.push(format!("-Fe:{}", exe.display()));
    argv.push(src.display().to_string());
    let cwd = src
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    let run = run_via_vcvars(&tc, &argv, &cwd, C_COMPILE_DEADLINE)?;
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    let err = String::from_utf8_lossy(&run.stderr).into_owned();
    if run.exit_code != Some(0) {
        return Err(CToolError(format!(
            "cl exited {:?}:\n{}\n{}",
            run.exit_code,
            out.trim(),
            err.trim()
        )));
    }
    if !exe.is_file() {
        return Err(CToolError(format!(
            "cl reported success but exe missing: {}",
            exe.display()
        )));
    }
    Ok(())
}

/// Compile the C file set (main.c + generated.c, runtime header beside
/// them) into `exe` with the frozen flags. A nonzero exit is an infra
/// error carrying the diagnostics.
pub fn compile_c_fileset(dir: &Path, exe: &Path) -> Result<(), CToolError> {
    let tc = resolve_toolchain()?;
    let mut argv: Vec<String> = C_FLAGS.iter().map(|s| s.to_string()).collect();
    argv.push(format!("-Fe:{}", exe.display()));
    argv.push("main.c".to_string());
    argv.push("generated.c".to_string());
    let run = run_via_vcvars(&tc, &argv, dir, C_COMPILE_DEADLINE)?;
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    let err = String::from_utf8_lossy(&run.stderr).into_owned();
    if run.exit_code != Some(0) {
        return Err(CToolError(format!(
            "cl exited {:?}:\n{}\n{}",
            run.exit_code,
            out.trim(),
            err.trim()
        )));
    }
    if !exe.is_file() {
        return Err(CToolError(format!(
            "cl reported success but exe missing: {}",
            exe.display()
        )));
    }
    Ok(())
}

/// Run a produced exe under the contained executor (30s deadline).
pub fn run_exe(exe: &Path) -> Result<CapturedRun, CToolError> {
    let mut cmd = Command::new(exe);
    run_deadlined_captured(
        &mut cmd,
        C_RUN_DEADLINE,
        CaptureCaps {
            stdout_max: C_STREAM_CAP,
            stderr_max: C_STREAM_CAP,
        },
    )
    .map_err(|e| CToolError(format!("contained run: {e:?}")))
}

/// Compile AND run the file set already materialized in `dir`; returns
/// (stdout, stderr) of the run. Used by the corpus leg and certification
/// (actual and reference compile the same way, independently).
pub fn compile_and_run_c(dir: &Path, exe_name: &str) -> Result<(String, String), CToolError> {
    let exe = dir.join(exe_name);
    compile_c_fileset(dir, &exe)?;
    let run = run_exe(&exe)?;
    if run.exit_code != Some(0) {
        return Err(CToolError(format!(
            "program exited {:?}: {}",
            run.exit_code,
            String::from_utf8_lossy(&run.stderr)
        )));
    }
    Ok((
        String::from_utf8_lossy(&run.stdout).into_owned(),
        String::from_utf8_lossy(&run.stderr).into_owned(),
    ))
}
