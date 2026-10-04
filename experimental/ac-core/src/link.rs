//! Platform link driver for the core-i32 prototype (PLAN-741, Windows x64).
//!
//! Links a lowered COFF object into a PE with rust-lld (COFF mode) against
//! the Windows SDK import libraries, then runs the result on demand.
//!
//! Every subprocess — tool discovery (`rustc`, `reg`), linking and running —
//! goes through [`run_with_deadline`], which drains stdout/stderr
//! concurrently so a child that fills either pipe cannot deadlock the wait
//! loop. Artifacts are published as a transaction: exe, obj and receipt are
//! staged under process-unique temporary names, the previous finals are
//! backed up, and only then renamed into place. Any failure rolls the whole
//! publish back — a failed build never overwrites previously successful
//! outputs (plan AC-07, Phase 2 QA-04).

use crate::atom_text::{Diagnostic, Span, Stage};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const SUBPROC_DEADLINE: Duration = Duration::from_secs(60);

fn link_diag(code: &str, msg: impl Into<String>) -> Diagnostic {
    Diagnostic::new(Stage::Link, code, msg, Span { start: 0, end: 0 })
}

/// Path next to `path` with `suffix` appended after the file stem, replacing
/// the extension: sibling("dir/add.exe", ".ac-link.txt") =
/// "dir/add.ac-link.txt". Only for names derived from a single artifact —
/// staging/backup names must use `companion` (stem collisions across
/// .exe/.obj finals would otherwise overwrite each other's backups).
fn sibling(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "ac".to_string());
    path.with_file_name(format!("{stem}{suffix}"))
}

/// Path next to `path` with `suffix` appended to the FULL file name:
/// companion("dir/tx.exe", ".bak-7") = "dir/tx.exe.bak-7". Unlike `sibling`,
/// artifacts with the same stem but different extensions never collide.
fn companion(path: &Path, suffix: &str) -> PathBuf {
    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "ac".to_string());
    path.with_file_name(format!("{name}{suffix}"))
}

/// Process-unique staging tag so concurrent builds targeting one output
/// cannot collide on temporary or backup names.
pub fn staging_tag() -> String {
    format!("ac741-{}", std::process::id())
}

/// Failure modes of the deadline executor, kept distinct so call sites can
/// map them to their own diagnostic codes (e.g. a missing `rustc` is
/// `link.rustc-missing`, not a generic spawn error).
#[derive(Debug)]
enum SubprocError {
    Spawn(std::io::Error),
    Deadline,
    Wait(std::io::Error),
}

fn map_subproc(err: SubprocError, spawn_code: &str, what: &str) -> Diagnostic {
    match err {
        SubprocError::Spawn(e) => link_diag(spawn_code, format!("spawn {what}: {e}")),
        SubprocError::Deadline => link_diag(
            "link.deadline",
            format!("subprocess {what} exceeded {SUBPROC_DEADLINE:?}"),
        ),
        SubprocError::Wait(e) => link_diag("link.wait", format!("wait {what}: {e}")),
    }
}

/// Run a subprocess with a hard deadline. stdout and stderr are drained by
/// dedicated reader threads (QA-06: waiting for exit *before* reading let a
/// child that filled the ~64KB pipe buffer block forever). Returns
/// (exit code, combined output).
fn run_with_deadline(
    cmd: &mut Command,
    deadline: Duration,
) -> Result<(Option<i32>, String), SubprocError> {
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SubprocError::Spawn)?;
    let started = Instant::now();
    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let out_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(p) = stdout_pipe.as_mut() {
            let _ = p.read_to_string(&mut buf);
        }
        buf
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        if let Some(p) = stderr_pipe.as_mut() {
            let _ = p.read_to_string(&mut buf);
        }
        buf
    });
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let out = out_reader.join().unwrap_or_default();
                let err = err_reader.join().unwrap_or_default();
                return Ok((status.code(), format!("{out}{err}")));
            }
            Ok(None) => {
                if started.elapsed() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    // The reader threads unwind on their own once the killed
                    // child closes the pipes; the diagnostics below do not
                    // depend on their buffers.
                    return Err(SubprocError::Deadline);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Err(SubprocError::Wait(e)),
        }
    }
}

/// Locate rust-lld: `AC741_RUST_LLD` override, then the active sysroot.
/// The sysroot query runs under the deadline executor like every other
/// subprocess (QA-06: it used a bare `Command::output` with no deadline).
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
    let (code, out) = run_with_deadline(
        Command::new("rustc").arg("--print").arg("sysroot"),
        SUBPROC_DEADLINE,
    )
    .map_err(|e| map_subproc(e, "link.rustc-missing", "rustc --print sysroot"))?;
    if code != Some(0) {
        return Err(link_diag(
            "link.rustc-missing",
            format!("rustc --print sysroot exited {code:?}"),
        ));
    }
    let sysroot = out.trim().to_string();
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
/// The registry query runs under the deadline executor (QA-06); a missing or
/// hung `reg` falls through to the known roots as before.
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
    if let Ok((_, out)) = run_with_deadline(
        Command::new("reg").args([
            "query",
            r"HKLM\SOFTWARE\Microsoft\Windows Kits\Installed Roots",
            "/v",
            "KitsRoot10",
        ]),
        SUBPROC_DEADLINE,
    ) {
        for line in out.lines() {
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

/// Receipt for a successful link, written next to the exe.
#[derive(Clone, Debug)]
pub struct LinkReceipt {
    pub linker: PathBuf,
    pub obj: PathBuf,
    pub exe: PathBuf,
    pub receipt_path: PathBuf,
    pub entry_symbol: String,
}

/// A successful link staged at a temporary path, not yet published.
pub struct StagedLink {
    pub tmp_exe: PathBuf,
    pub exe: PathBuf,
    pub linker: PathBuf,
    pub entry_symbol: String,
}

/// Link `obj_path` with `/entry:<entry_symbol>` into a temporary exe next to
/// `exe_path`. Nothing is published here; call [`publish_artifacts`] to
/// place exe, obj and receipt as one transaction.
pub fn link_object_staged(
    lld: &Path,
    sdk_um_dir: &Path,
    obj_path: &Path,
    exe_path: &Path,
    entry_symbol: &str,
    extra_libs: &[PathBuf],
) -> Result<StagedLink, Diagnostic> {
    let tmp_exe = companion(exe_path, &format!(".tmp-{}", staging_tag()));
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
    let (code, out) = run_with_deadline(Command::new(lld).args(&args), SUBPROC_DEADLINE)
        .map_err(|e| map_subproc(e, "link.spawn", "rust-lld"))?;
    if code != Some(0) {
        let _ = std::fs::remove_file(&tmp_exe);
        return Err(link_diag(
            "link.failed",
            format!("rust-lld exit {:?}: {}", code, out.trim()),
        ));
    }
    Ok(StagedLink {
        tmp_exe,
        exe: exe_path.to_path_buf(),
        linker: lld.to_path_buf(),
        entry_symbol: entry_symbol.to_string(),
    })
}

/// Publish staged artifacts as one transaction (QA-04: r1 renamed the exe
/// into place before writing the receipt, so a receipt failure left the new
/// exe published and the old one destroyed).
///
/// Sequence: stage the receipt, back up existing exe/obj/receipt finals,
/// rename staged exe/obj/receipt into place, then drop the backups. Any
/// failure removes what this call placed and restores the backups, so the
/// previous successful artifacts survive untouched. A final path occupied by
/// a directory is left alone (not "backed up"), which fails the publish and
/// rolls back rather than deleting user data.
pub fn publish_artifacts(
    staged: &StagedLink,
    tmp_obj: &Path,
    obj_path: &Path,
) -> Result<LinkReceipt, Diagnostic> {
    let tag = staging_tag();
    let receipt_path = sibling(&staged.exe, ".ac-link.txt");
    let tmp_receipt = companion(&receipt_path, &format!(".tmp-{tag}"));
    let receipt_body = format!(
        "PLAN-741 link receipt\nlinker: {}\nobj: {}\nexe: {}\nentry: {}\n",
        staged.linker.display(),
        obj_path.display(),
        staged.exe.display(),
        staged.entry_symbol,
    );
    if let Err(e) = std::fs::write(&tmp_receipt, receipt_body) {
        let _ = std::fs::remove_file(&tmp_receipt);
        return Err(link_diag("link.receipt", format!("write receipt: {}", e)));
    }

    // Back up existing finals (files only). `done` records what the backup
    // step moved so a later failure can restore exactly those.
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new(); // (bak, final)
    for final_path in [&staged.exe, obj_path, &receipt_path] {
        if final_path.is_file() {
            let bak = companion(final_path, &format!(".bak-{tag}"));
            match std::fs::rename(final_path, &bak) {
                Ok(()) => backups.push((bak, final_path.to_path_buf())),
                Err(e) => {
                    let _ = std::fs::remove_file(&tmp_receipt);
                    for (bak, final_path) in backups.iter().rev() {
                        let _ = std::fs::rename(bak, final_path);
                    }
                    return Err(link_diag(
                        "link.replace",
                        format!("backing up {}: {}", final_path.display(), e),
                    ));
                }
            }
        }
    }

    // Publish. On failure: restore backed-up finals, remove what was placed.
    let mut placed: Vec<PathBuf> = Vec::new();
    let result = (|| -> Result<(), Diagnostic> {
        std::fs::rename(&staged.tmp_exe, &staged.exe).map_err(|e| {
            link_diag(
                "link.replace",
                format!("placing {}: {}", staged.exe.display(), e),
            )
        })?;
        placed.push(staged.exe.clone());
        std::fs::rename(tmp_obj, obj_path).map_err(|e| {
            link_diag(
                "link.replace",
                format!("placing {}: {}", obj_path.display(), e),
            )
        })?;
        placed.push(obj_path.to_path_buf());
        std::fs::rename(&tmp_receipt, &receipt_path).map_err(|e| {
            link_diag(
                "link.receipt",
                format!("placing {}: {}", receipt_path.display(), e),
            )
        })?;
        Ok(())
    })();

    match result {
        Ok(()) => {
            for (bak, _) in &backups {
                let _ = std::fs::remove_file(bak);
            }
            Ok(LinkReceipt {
                linker: staged.linker.clone(),
                obj: obj_path.to_path_buf(),
                exe: staged.exe.clone(),
                receipt_path,
                entry_symbol: staged.entry_symbol.clone(),
            })
        }
        Err(d) => {
            // Restore backups first (a rename replaces the placed new file);
            // only placed finals with no previous version get removed, never
            // a file that was just restored from its backup.
            for (bak, final_path) in backups.iter().rev() {
                let _ = std::fs::rename(bak, final_path);
            }
            let restored: std::collections::BTreeSet<&Path> =
                backups.iter().map(|(_, f)| f.as_path()).collect();
            for p in &placed {
                if !restored.contains(p.as_path()) {
                    let _ = std::fs::remove_file(p);
                }
            }
            let _ = std::fs::remove_file(&tmp_receipt);
            Err(d)
        }
    }
}

/// Run a produced exe with a deadline; returns its exit code.
pub fn run_exe(exe_path: &Path) -> Result<i32, Diagnostic> {
    let (code, out) = run_with_deadline(&mut Command::new(exe_path), SUBPROC_DEADLINE)
        .map_err(|e| map_subproc(e, "run.spawn", exe_path.display().to_string().as_str()))?;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(windows)]
    fn deadline_terminates_hanging_helper() {
        let started = Instant::now();
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "ping", "-n", "30", "127.0.0.1"]);
        let err = run_with_deadline(&mut cmd, Duration::from_secs(1)).err();
        assert!(
            matches!(err, Some(SubprocError::Deadline)),
            "expected deadline error, got {err:?}"
        );
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "deadline took {:?} to fire",
            started.elapsed()
        );
    }

    #[test]
    #[cfg(windows)]
    fn large_output_does_not_deadlock() {
        // ~3000 lines x ~42 bytes = ~126KB, far beyond the pipe buffer: the
        // r1 executor (wait-for-exit, then read) hit its 60s deadline here.
        let mut cmd = Command::new("cmd");
        cmd.args([
            "/C",
            "for /L %i in (1,1,3000) do @echo 0123456789012345678901234567890123456789",
        ]);
        let (code, out) = run_with_deadline(&mut cmd, Duration::from_secs(30)).expect("complete");
        assert_eq!(code, Some(0));
        assert!(
            out.len() > 100_000,
            "output truncated to {} bytes",
            out.len()
        );
    }
}
