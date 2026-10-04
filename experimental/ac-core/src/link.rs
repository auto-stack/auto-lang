//! Platform link driver for the core-i32 prototype (PLAN-741, Windows x64).
//!
//! Links a lowered COFF object into a PE with rust-lld (COFF mode) against
//! the Windows SDK import libraries, then runs the result on demand. All
//! subprocesses carry a hard deadline. Artifacts are replaced atomically: a
//! failed build never overwrites previously successful outputs (plan AC-07).

use crate::atom_text::{Diagnostic, Span, Stage};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

const SUBPROC_DEADLINE: Duration = Duration::from_secs(60);

fn link_diag(code: &str, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Stage::Link, code, msg, Span { start: 0, end: 0 })
}

/// Locate rust-lld: `AC741_RUST_LLD` override, then the active sysroot.
pub fn find_rust_lld() -> Result<PathBuf, Diagnostic> {
    if let Ok(env_path) = std::env::var("AC741_RUST_LLD") {
        let p = PathBuf::from(env_path);
        if p.is_file() {
            return Ok(p);
        }
        return Err(link_diag(
            "link.lld-not-found",
            format!("AC741_RUST_LLD points at a missing file: {}", p.display()),
        ));
    }
    let out = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .map_err(|e| {
            link_diag(
                "link.rustc-missing",
                format!("rustc --print sysroot: {}", e),
            )
        })?;
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let candidate = Path::new(&sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(link_diag(
            "link.lld-not-found",
            format!("rust-lld.exe not found under sysroot {}", sysroot),
        ))
    }
}

/// Locate a directory containing the Windows SDK `kernel32.lib` (um/x64):
/// `AC741_SDK_UM_LIB` override, then registry KitsRoot10, then known roots.
pub fn find_sdk_um_dir() -> Result<PathBuf, Diagnostic> {
    if let Ok(env_dir) = std::env::var("AC741_SDK_UM_LIB") {
        let p = PathBuf::from(env_dir);
        if p.join("kernel32.lib").is_file() {
            return Ok(p);
        }
        return Err(link_diag(
            "link.sdk-not-found",
            format!("AC741_SDK_UM_LIB has no kernel32.lib: {}", p.display()),
        ));
    }
    let mut roots: Vec<PathBuf> = Vec::new();
    if let Ok(out) = Command::new("reg")
        .args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots",
            "/v",
            "KitsRoot10",
        ])
        .output()
    {
        let text = String::from_utf8_lossy(&out.stdout);
        for line in text.lines() {
            if line.contains("KitsRoot10") && line.contains("REG_SZ") {
                if let Some(pos) = line.rfind("REG_SZ") {
                    roots.push(PathBuf::from(line[pos + "REG_SZ".len()..].trim()));
                }
            }
        }
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10"));
    roots.push(PathBuf::from(r"D:\Windows Kits\10"));
    for root in roots {
        let Ok(entries) = std::fs::read_dir(root.join("Lib")) else {
            continue;
        };
        let mut versions: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("um/x64/kernel32.lib").is_file())
            .collect();
        versions.sort();
        if let Some(latest) = versions.pop() {
            return Ok(latest.join("um/x64"));
        }
    }
    Err(link_diag(
        "link.sdk-not-found",
        "no Windows SDK um/x64 kernel32.lib found (set AC741_SDK_UM_LIB)",
    ))
}

/// Run a subprocess with a hard deadline; returns (exit code, combined output).
fn run_with_deadline(cmd: &mut Command) -> Result<(Option<i32>, String), Diagnostic> {
    use std::io::Read;
    let mut child = cmd
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            link_diag(
                "link.spawn",
                format!("spawn {:?}: {}", cmd.get_program(), e),
            )
        })?;
    let deadline = Instant::now() + SUBPROC_DEADLINE;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut out = String::new();
                if let Some(mut io) = child.stdout.take() {
                    let _ = io.read_to_string(&mut out);
                }
                if let Some(mut io) = child.stderr.take() {
                    let _ = io.read_to_string(&mut out);
                }
                return Ok((status.code(), out));
            }
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    return Err(link_diag(
                        "link.deadline",
                        format!(
                            "subprocess {:?} exceeded {:?}",
                            cmd.get_program(),
                            SUBPROC_DEADLINE
                        ),
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(link_diag("link.wait", format!("wait: {}", e))),
        }
    }
}

/// Receipt for a successful link, written next to the exe.
#[derive(Clone, Debug)]
pub struct LinkReceipt {
    pub linker: PathBuf,
    pub obj: PathBuf,
    pub exe: PathBuf,
    pub entry_symbol: String,
}

/// Link `obj_path` into `exe_path` with `/entry:<entry_symbol>`.
///
/// Atomicity: the exe is produced at a temporary path first; on success the
/// temporary replaces the target. Any failure leaves an existing exe and its
/// receipt untouched.
pub fn link_object(
    lld: &Path,
    sdk_um_dir: &Path,
    obj_path: &Path,
    exe_path: &Path,
    entry_symbol: &str,
    extra_libs: &[PathBuf],
) -> Result<LinkReceipt, Diagnostic> {
    let tmp_exe = exe_path.with_extension("exe.tmp-ac741");
    let mut args: Vec<String> = vec![
        "-flavor".into(),
        "link".into(),
        "/nologo".into(),
        format!("/entry:{}", entry_symbol),
        "/subsystem:console".into(),
        format!("/out:{}", tmp_exe.display()),
        format!("{}", obj_path.display()),
        format!("/libpath:{}", sdk_um_dir.display()),
        "kernel32.lib".into(),
    ];
    for lib in extra_libs {
        args.push(lib.display().to_string());
    }
    let (code, out) = run_with_deadline(Command::new(lld).args(&args))?;
    if code != Some(0) {
        let _ = std::fs::remove_file(&tmp_exe);
        return Err(link_diag(
            "link.failed",
            format!("rust-lld exit {:?}: {}", code, out.trim()),
        ));
    }
    // Replace prior artifacts only now that the new exe exists.
    if let Err(e) = std::fs::rename(&tmp_exe, exe_path) {
        let _ = std::fs::remove_file(&tmp_exe);
        return Err(link_diag(
            "link.replace",
            format!("replacing {}: {}", exe_path.display(), e),
        ));
    }
    let receipt = LinkReceipt {
        linker: lld.to_path_buf(),
        obj: obj_path.to_path_buf(),
        exe: exe_path.to_path_buf(),
        entry_symbol: entry_symbol.to_string(),
    };
    write_receipt(exe_path, &receipt)?;
    Ok(receipt)
}

fn write_receipt(exe_path: &Path, r: &LinkReceipt) -> Result<(), Diagnostic> {
    let receipt_path = exe_path.with_extension("ac-link.txt");
    let body = format!(
        "PLAN-741 link receipt\nlinker: {}\nobj: {}\nexe: {}\nentry: {}\n",
        r.linker.display(),
        r.obj.display(),
        r.exe.display(),
        r.entry_symbol,
    );
    std::fs::write(&receipt_path, body)
        .map_err(|e| link_diag("link.receipt", format!("write receipt: {}", e)))
}

/// Run a produced exe with a deadline; returns its exit code.
pub fn run_exe(exe_path: &Path) -> Result<i32, Diagnostic> {
    let (code, out) = run_with_deadline(&mut Command::new(exe_path))?;
    match code {
        Some(c) => Ok(c),
        None => Err(link_diag(
            "run.signal",
            format!(
                "{} terminated abnormally; output: {}",
                exe_path.display(),
                out.trim()
            ),
        )),
    }
}
