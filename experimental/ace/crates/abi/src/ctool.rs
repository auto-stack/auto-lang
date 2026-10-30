//! Independent C compiler toolchain for the interop legs (plan §3, T-01).
//!
//! Rules frozen here:
//! * The C side is compiled by a REAL, separately located MSVC `cl.exe`.
//!   A missing tool is a controlled INFRA error — the affected legs stop;
//!   there is no fallback to "Rust repr(C) is good enough" and nothing is
//!   auto-downloaded or installed.
//! * Discovery: `AUTOABI_MSVC_CL` env override -> vswhere (fixed installer
//!   path) -> highest `VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe`. The
//!   resolved identity (path + banner version + content hash) is part of
//!   the toolchain fingerprint that later feeds cache keys; it is pinned
//!   in contract-decision.md §5.
//! * Linking stays with the merged ac-core entries (rust-lld); cl.exe is
//!   only ever invoked with `/c` (compile to object).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::Duration;

const VSWHERE: &str = r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe";
const DISCOVERY_DEADLINE: Duration = Duration::from_secs(10);
pub const COMPILE_DEADLINE: Duration = Duration::from_secs(60);

/// Frozen flags for every C compile (freestanding objects: no CRT init,
/// no stack cookies, no default-lib directives, warnings as errors).
pub const CL_FLAGS: &[&str] = &["-c", "-nologo", "-GS-", "-Zl", "-W4", "-WX", "-O2"];

#[derive(Debug)]
pub struct CToolError(pub String);

impl std::fmt::Display for CToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "abi.ctool: {}", self.0)
    }
}

/// The resolved C toolchain identity.
#[derive(Clone, Debug)]
pub struct CToolchain {
    pub cl_exe: PathBuf,
    /// Version banner line from `cl -?` (e.g. "19.44.35222 for x64").
    pub version: String,
    /// blake3 content hash of the cl.exe binary.
    pub cl_hash: String,
}

/// Locate cl.exe per the frozen discovery order. Result cached for the
/// process; a missing tool is an error, never a skip.
pub fn find_cl() -> Result<&'static CToolchain, CToolError> {
    static CACHE: OnceLock<Result<CToolchain, String>> = OnceLock::new();
    CACHE
        .get_or_init(|| resolve_cl().map_err(|e| e.0))
        .as_ref()
        .map_err(|e| CToolError(e.clone()))
}

fn resolve_cl() -> Result<CToolchain, CToolError> {
    let cl = if let Ok(env_path) = std::env::var("AUTOABI_MSVC_CL") {
        let p = PathBuf::from(env_path);
        if p.is_file() {
            p
        } else {
            return Err(CToolError(format!(
                "AUTOABI_MSVC_CL points at a missing file: {}",
                p.display()
            )));
        }
    } else {
        locate_via_vswhere()?
    };

    // Banner version: `cl -?` prints the version line and exits 0. Runs
    // under the contained executor like every other subprocess.
    let mut cmd = Command::new(&cl);
    cmd.arg("-?");
    let (code, out, err) =
        run_deadlined(&mut cmd).map_err(|e| CToolError(format!("cl -? spawn: {e:?}")))?;
    if code != Some(0) {
        return Err(CToolError(format!(
            "cl -? exited {code:?}: {}{}",
            out.trim(),
            err.trim()
        )));
    }
    // Banner version: `cl -?` prints the version banner on STDERR and
    // the option help on stdout. The banner is e.g.
    // "用于 x64 的 Microsoft (R) C/C++ 优化编译器 19.44.35222 版" — note
    // the trailing single "版", NOT "版本" — so the match requires a
    // compiler word plus a dotted version-number token; a help line can
    // never satisfy both.
    let banner = err
        .lines()
        .chain(out.lines())
        .find(|l| {
            let compiler_word =
                l.contains("编译器") || l.contains("Compiler") || l.contains("Version");
            let dotted = l.split_whitespace().any(|w| {
                w.chars().filter(|c| *c == '.').count() >= 2
                    && w.chars().next().is_some_and(|c| c.is_ascii_digit())
            });
            compiler_word && dotted
        })
        .unwrap_or("")
        .trim()
        .to_string();
    if banner.is_empty() {
        return Err(CToolError("cl banner version line not found".into()));
    }

    let bytes = std::fs::read(&cl).map_err(|e| CToolError(format!("read cl.exe: {e}")))?;
    let cl_hash = crate::semref::blake3_hex(&bytes);

    Ok(CToolchain {
        cl_exe: cl,
        version: banner,
        cl_hash,
    })
}

fn locate_via_vswhere() -> Result<PathBuf, CToolError> {
    if !Path::new(VSWHERE).is_file() {
        return Err(CToolError(format!(
            "vswhere missing at {VSWHERE} and AUTOABI_MSVC_CL unset — \
             C toolchain unavailable (infra blocker, plan §3)"
        )));
    }
    let mut cmd = Command::new(VSWHERE);
    cmd.args([
        "-latest",
        "-products",
        "*",
        "-requires",
        "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "-property",
        "installationPath",
    ]);
    let (code, out, err) =
        run_deadlined(&mut cmd).map_err(|e| CToolError(format!("vswhere spawn: {e:?}")))?;
    if code != Some(0) {
        return Err(CToolError(format!(
            "vswhere exited {code:?}: {}{}",
            out.trim(),
            err.trim()
        )));
    }
    let inst = PathBuf::from(out.trim());
    if inst.as_os_str().is_empty() {
        return Err(CToolError(
            "vswhere returned no VC.Tools installation (infra blocker, plan §3)".into(),
        ));
    }
    let tools = inst.join("VC/Tools/MSVC");
    let mut versions: Vec<PathBuf> = std::fs::read_dir(&tools)
        .map_err(|e| CToolError(format!("read {}: {e}", tools.display())))?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.join("bin/Hostx64/x64/cl.exe").is_file())
        .collect();
    versions.sort(); // version-named dirs: lexical == numeric for MSVC
    versions
        .pop()
        .map(|v| v.join("bin/Hostx64/x64/cl.exe"))
        .ok_or_else(|| {
            CToolError(format!(
                "no VC/Tools/MSVC/*/bin/Hostx64/x64/cl.exe under {} (infra blocker)",
                inst.display()
            ))
        })
}

fn run_deadlined(
    cmd: &mut Command,
) -> Result<(Option<i32>, String, String), ace_core::link::SubprocError> {
    ace_core::link::run_deadlined_split(cmd, DISCOVERY_DEADLINE)
}

/// Compile one C source into `obj_out` with the frozen flags. `include_dir`
/// is the probe's `cside/` directory; the resolved cl.exe's own MSVC
/// include dir is added automatically so freestanding headers (stdint.h)
/// resolve without a vcvars environment. The wider SDK/ucrt header
/// closure is NOT implied — T-01/T-02 C sources include only probe-owned
/// headers plus freestanding fixed-width types; header closure enters
/// the cache keys with the T-05 recipe.
pub fn compile_c(src: &Path, include_dir: &Path, obj_out: &Path) -> Result<(), CToolError> {
    compile_c_with_deps(src, include_dir, obj_out).map(|_| ())
}

/// F-797-R4-07 (R8): compile and OBSERVE the real dependency closure.
/// The compile runs with `-showIncludes` (a reporting-only flag: it does
/// not affect codegen, so it stays OUT of the frozen CL_FLAGS
/// fingerprint); every file the preprocessor actually read — probe-local
/// AND system/MSVC headers — is returned as an absolute path. This is
/// the dependency truth the cache consumes: text scanning cannot see
/// comment-hidden includes, line splices, or macro includes, but the
/// compiler's own preprocessor can.
pub fn compile_c_with_deps(
    src: &Path,
    include_dir: &Path,
    obj_out: &Path,
) -> Result<Vec<PathBuf>, CToolError> {
    let tool = find_cl()?;
    // cl.exe = <msvc>/bin/Hostx64/x64/cl.exe -> <msvc>/include
    let msvc_root = tool
        .cl_exe
        .ancestors()
        .nth(4)
        .ok_or_else(|| CToolError("cannot derive MSVC root from cl path".into()))?;
    let msvc_include = msvc_root.join("include");
    if !msvc_include.join("stdint.h").is_file() {
        return Err(CToolError(format!(
            "MSVC include dir missing stdint.h: {}",
            msvc_include.display()
        )));
    }
    let mut cmd = Command::new(&tool.cl_exe);
    cmd.args(CL_FLAGS);
    cmd.arg("-showIncludes");
    cmd.arg(format!("-I{}", include_dir.display()));
    cmd.arg(format!("-I{}", msvc_include.display()));
    cmd.arg(src);
    cmd.arg(format!("-Fo:{}", obj_out.display()));
    // F-797-R4-10 (R5): compile through the merged RAW CAPPED executor
    // — a cap violation terminates the tree (no truncated-green), raw
    // bytes for display.
    let run = ace_core::link::run_deadlined_captured(
        &mut cmd,
        COMPILE_DEADLINE,
        ace_core::link::CaptureCaps {
            stdout_max: 1 << 20,
            stderr_max: 1 << 20,
        },
    )
    .map_err(|e| CToolError(format!("cl spawn: {e:?}")))?;
    let code = run.exit_code;
    let out = String::from_utf8_lossy(&run.stdout).into_owned();
    let err = String::from_utf8_lossy(&run.stderr).into_owned();
    if code != Some(0) {
        return Err(CToolError(format!(
            "cl exited {code:?} for {}:\n{}\n{}",
            src.display(),
            out.trim(),
            err.trim()
        )));
    }
    if !obj_out.is_file() {
        return Err(CToolError(format!(
            "cl reported success but object missing: {}",
            obj_out.display()
        )));
    }
    Ok(parse_show_includes(&out, src))
}

/// Extract the dependency paths from `/showIncludes` output. Each note
/// line carries an absolute path after a locale-dependent prefix ("Note:
/// including file:" / "注意: 包含文件:") — scanning for the Windows
/// drive-letter pattern is prefix-agnostic. Only files that exist are
/// accepted; the compiled source itself is excluded.
fn parse_show_includes(stdout: &str, src: &Path) -> Vec<PathBuf> {
    let mut deps: Vec<PathBuf> = Vec::new();
    for line in stdout.lines() {
        let Some(path) = extract_drive_path(line) else {
            continue;
        };
        let p = PathBuf::from(&path);
        if !p.is_file() || p == src {
            continue;
        }
        if !deps.contains(&p) {
            deps.push(p);
        }
    }
    deps
}

fn extract_drive_path(line: &str) -> Option<String> {
    let bytes = line.as_bytes();
    for i in 0..bytes.len().saturating_sub(2) {
        if bytes[i].is_ascii_alphabetic()
            && bytes[i + 1] == b':'
            && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/')
        {
            let candidate = line[i..].trim().trim_matches('"');
            if candidate.contains('\\') || candidate.contains('/') {
                return Some(candidate.to_string());
            }
        }
    }
    None
}

/// Stable toolchain fingerprint for cache keys and receipts (cl identity
/// + frozen flags; the linker/SDK side comes from the ac-core entries and
/// is pinned in contract-decision.md §5).
pub fn toolchain_fingerprint() -> Result<String, CToolError> {
    let t = find_cl()?;
    Ok(format!(
        "cl:{}|ver:{}|hash:{}|flags:{}",
        t.cl_exe.display(),
        t.version,
        t.cl_hash,
        CL_FLAGS.join(" ")
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tool presence is a hard infra precondition on this machine; the
    /// test asserts the pinned identity shape (path exists, banner
    /// captured, hash 64-hex) and prints it as freeze evidence.
    #[test]
    fn cl_toolchain_resolves() {
        let t = find_cl().expect("MSVC cl.exe must resolve (infra blocker otherwise)");
        assert!(t.cl_exe.is_file());
        assert!(!t.version.is_empty());
        assert_eq!(t.cl_hash.len(), 64);
        println!("cl.exe = {}", t.cl_exe.display());
        println!("banner = {}", t.version);
        println!("blake3 = {}", t.cl_hash);
    }
}
