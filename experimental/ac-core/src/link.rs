//! Platform link driver for the core-i32 prototype (PLAN-741, Windows x64).
//!
//! Links a lowered COFF object into a PE with rust-lld (COFF mode) against
//! the Windows SDK import libraries, then runs the result on demand.
//!
//! Every subprocess — tool discovery (`rustc`, `reg`), linking and running —
//! goes through [`run_with_deadline`], which drains stdout/stderr
//! concurrently so a child that fills either pipe cannot deadlock the wait
//! loop. One monotonic deadline covers BOTH the process wait and the output
//! collection, and on Windows the child is assigned to a Job Object whose
//! close kills the whole subtree: a descendant that inherits the pipes can
//! no longer wedge the executor past the deadline (Phase 3 R2-QA-02).
//! Artifacts are published as a transaction: exe, obj and receipt are
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
    /// Containment (job creation/assignment or controlled resume) failed.
    /// The child was terminated before running a single instruction — the
    /// executor refuses to run a process it cannot bound (R3-QA-01: no
    /// silent degrade into detached readers).
    ContainmentUnavailable(&'static str),
    /// The direct child did not exit before the deadline.
    Deadline,
    /// The child exited but its output collection (readers blocked by a
    /// descendant that inherited the pipe handles) did not finish within the
    /// same deadline.
    DeadlineCollect,
    Wait(std::io::Error),
}

fn map_subproc(err: SubprocError, spawn_code: &str, what: &str) -> Diagnostic {
    match err {
        SubprocError::Spawn(e) => link_diag(spawn_code, format!("spawn {what}: {e}")),
        SubprocError::ContainmentUnavailable(reason) => link_diag(
            "link.containment",
            format!("subprocess {what} not started: {reason}"),
        ),
        SubprocError::Deadline => link_diag(
            "link.deadline",
            format!("subprocess {what} exceeded {SUBPROC_DEADLINE:?}"),
        ),
        SubprocError::DeadlineCollect => link_diag(
            "link.deadline",
            format!(
                "subprocess {what} exited but its output did not finish within {SUBPROC_DEADLINE:?} (a descendant may still hold the pipe)"
            ),
        ),
        SubprocError::Wait(e) => link_diag("link.wait", format!("wait {what}: {e}")),
    }
}

/// Windows Job Object containment (R2-QA-02, R3-QA-01): the direct child is
/// spawned SUSPENDED, assigned to a job with KILL_ON_JOB_CLOSE *before* its
/// first instruction runs, then resumed. The r3 spawn-then-assign window let
/// a fast child create descendants that escaped the job and survived the
/// guard drop; the suspended start closes that window entirely. Dropping the
/// guard terminates the whole subtree — their write ends close, the reader
/// threads reach EOF, and no thread outlives the call. Containment failure
/// is never silently degraded: the executor rejects the spawn with
/// [`SubprocError::ContainmentUnavailable`] after a safe teardown.
#[cfg(windows)]
mod job {
    use std::os::windows::io::AsRawHandle;
    use std::process::Child;
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };
    use windows_sys::Win32::System::Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME};

    pub const CREATE_SUSPENDED: u32 = 0x0000_0004;

    pub struct JobGuard(HANDLE);

    impl JobGuard {
        /// Child must be suspended: the job is in place before the process
        /// can run (and therefore before it can spawn descendants).
        pub fn for_child(child: &Child) -> Option<JobGuard> {
            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    return None;
                }
                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                if SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const core::ffi::c_void,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                ) == 0
                {
                    CloseHandle(job);
                    return None;
                }
                if AssignProcessToJobObject(job, child.as_raw_handle() as HANDLE) == 0 {
                    CloseHandle(job);
                    return None;
                }
                Some(JobGuard(job))
            }
        }
    }

    impl Drop for JobGuard {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }

    /// Resume the suspended child. A freshly spawned suspended process has
    /// exactly one thread; find it through the system thread snapshot and
    /// resume it. Err on any snapshot/open failure — the caller then tears
    /// the child down instead of leaving it frozen.
    pub fn resume_suspended_process(child: &Child) -> Result<(), &'static str> {
        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err("thread snapshot failed while resuming contained child");
            }
            let mut entry: THREADENTRY32 = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<THREADENTRY32>() as u32;
            if Thread32First(snapshot, &mut entry) == 0 {
                CloseHandle(snapshot);
                return Err("thread enumeration failed while resuming contained child");
            }
            let mut resumed = false;
            loop {
                if entry.th32OwnerProcessID == child.id() {
                    let thread = OpenThread(THREAD_SUSPEND_RESUME, 0, entry.th32ThreadID);
                    if thread.is_null() {
                        CloseHandle(snapshot);
                        return Err("failed to open the contained child's main thread");
                    }
                    if ResumeThread(thread) == u32::MAX {
                        CloseHandle(thread);
                        CloseHandle(snapshot);
                        return Err("failed to resume the contained child's main thread");
                    }
                    CloseHandle(thread);
                    resumed = true;
                }
                if Thread32Next(snapshot, &mut entry) == 0 {
                    break;
                }
            }
            CloseHandle(snapshot);
            if resumed {
                Ok(())
            } else {
                Err("contained child's main thread not found in snapshot")
            }
        }
    }
}

/// Non-Windows stub: the prototype's link/run path is Windows-only; the
/// executor requires containment, so every spawn is rejected with a
/// controlled error instead of running unbounded.
#[cfg(not(windows))]
mod job {
    use std::process::Child;

    pub struct JobGuard;

    impl JobGuard {
        pub fn for_child(_child: &Child) -> Option<JobGuard> {
            None
        }
    }
}

/// Run a subprocess with a hard deadline that covers the whole lifecycle:
/// the wait for the exit status AND the drain of stdout/stderr (R2-QA-02: a
/// child that exits while a descendant still holds the inherited pipes used
/// to wedge the reader joins far past the deadline). The child starts
/// SUSPENDED and enters the job before its first instruction, so no
/// descendant can be created outside containment (R3-QA-01: the r3
/// spawn-then-assign window let fast descendants escape and survive the job
/// close). Readers run on dedicated threads; results arrive over a channel
/// so the collection phase is bounded by the same monotonic deadline. A
/// truncated success is never returned; containment failure is a controlled
/// rejection, never a silent degrade into detached readers.
fn run_with_deadline(
    cmd: &mut Command,
    deadline: Duration,
) -> Result<(Option<i32>, String), SubprocError> {
    run_with_deadline_inner(cmd, deadline, job::JobGuard::for_child)
}

/// Inner executor with an injectable containment factory so tests can
/// deterministically drive the containment-failure branch (no product fault
/// injection switch).
fn run_with_deadline_inner(
    cmd: &mut Command,
    deadline: Duration,
    job_factory: fn(&std::process::Child) -> Option<job::JobGuard>,
) -> Result<(Option<i32>, String), SubprocError> {
    // Controlled start (R3-QA-01): the suspended child cannot run, and
    // therefore cannot spawn descendants, until it is inside the job.
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(job::CREATE_SUSPENDED);
    }
    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(SubprocError::Spawn)?;
    let started = Instant::now();
    // Contain BEFORE the first instruction runs. Any containment failure is
    // a controlled rejection with safe teardown: the child is killed before
    // it ever ran, so nothing can leak — no silent degrade, no detached
    // readers.
    let Some(subtree) = job_factory(&child) else {
        let _ = child.kill();
        let _ = child.wait();
        return Err(SubprocError::ContainmentUnavailable(
            "job creation/assignment failed; child terminated before its first instruction",
        ));
    };
    #[cfg(windows)]
    if let Err(reason) = job::resume_suspended_process(&child) {
        let _ = child.kill();
        let _ = child.wait();
        drop(subtree);
        return Err(SubprocError::ContainmentUnavailable(reason));
    }

    let mut stdout_pipe = child.stdout.take();
    let mut stderr_pipe = child.stderr.take();
    let (out_tx, mut out_rx) = std::sync::mpsc::channel::<String>();
    let (err_tx, mut err_rx) = std::sync::mpsc::channel::<String>();
    let out_reader = std::thread::spawn(move || {
        // read_to_end + lossy conversion: read_to_string validates UTF-8
        // all-or-nothing, so one invalid byte (localized tool output, e.g.
        // GBK ping) would discard every valid byte read before it — exactly
        // the truncated-success-output the contract forbids.
        let mut buf = Vec::new();
        if let Some(p) = stdout_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        let _ = out_tx.send(String::from_utf8_lossy(&buf).to_string());
    });
    let err_reader = std::thread::spawn(move || {
        let mut buf = Vec::new();
        if let Some(p) = stderr_pipe.as_mut() {
            let _ = p.read_to_end(&mut buf);
        }
        let _ = err_tx.send(String::from_utf8_lossy(&buf).to_string());
    });

    // Wait for one reader result within the remaining deadline. Err(()) =
    // collection timed out (the pipe write end is still held by someone).
    let collect = |rx: &mut std::sync::mpsc::Receiver<String>| -> Result<String, ()> {
        let remaining = deadline.saturating_sub(started.elapsed());
        if remaining.is_zero() {
            return Err(());
        }
        match rx.recv_timeout(remaining) {
            Ok(s) => Ok(s),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(()),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => Ok(String::new()),
        }
    };
    // Reap both readers on every exit path: dropping the job guard kills the
    // whole contained tree, their write ends close, and the joins return
    // promptly. Containment is a hard precondition here, so there is no
    // degraded branch and no thread is ever left detached.
    let reap = |subtree: job::JobGuard,
                out: std::thread::JoinHandle<()>,
                err: std::thread::JoinHandle<()>| {
        drop(subtree);
        let _ = out.join();
        let _ = err.join();
    };

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (out, err) = (collect(&mut out_rx), collect(&mut err_rx));
                match (out, err) {
                    (Ok(out), Ok(err)) => {
                        // Readers have sent their buffers; join reclaims the
                        // threads (they exit right after send).
                        let _ = out_reader.join();
                        let _ = err_reader.join();
                        drop(subtree);
                        return Ok((status.code(), format!("{out}{err}")));
                    }
                    _ => {
                        // The child exited but a descendant still holds the
                        // pipes: the deadline applies to collection too.
                        reap(subtree, out_reader, err_reader);
                        return Err(SubprocError::DeadlineCollect);
                    }
                }
            }
            Ok(None) => {
                if started.elapsed() > deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    reap(subtree, out_reader, err_reader);
                    return Err(SubprocError::Deadline);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                reap(subtree, out_reader, err_reader);
                return Err(SubprocError::Wait(e));
            }
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
    let (code, out) = match run_with_deadline(Command::new(lld).args(&args), SUBPROC_DEADLINE) {
        Ok(r) => r,
        Err(e) => {
            // R2-QA-03: this call owns the staged exe until the caller takes
            // it — every error exit must reclaim it, not just the nonzero-
            // exit path below.
            let _ = std::fs::remove_file(&tmp_exe);
            return Err(map_subproc(e, "link.spawn", "rust-lld"));
        }
    };
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

/// Publish staged artifacts as one transaction.
///
/// Sequence: stage the receipt, back up existing exe/obj/receipt finals,
/// rename staged exe/obj/receipt into place, then drop the backups.
///
/// Failure semantics (Phase 2 QA-04 + Phase 3 R2-QA-03): every failure exit
/// — receipt staging, backup, any of the three placements — reclaims exactly
/// the three staged files this call created (the already-linked staged exe
/// included) and restores the backups, so previous successful artifacts
/// survive byte-identical. A final path occupied by a directory is user
/// property: it is never backed up, never deleted, and fails the publish.
/// If a rollback itself is blocked by an IO error, the call returns a
/// `link.restore` diagnostic listing the surviving backup paths instead of
/// claiming a complete rollback.
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
    // Staged-file ownership: these three paths are created by this call and
    // must be reclaimed on every failure exit. Cleanup errors are COLLECTED,
    // never swallowed (R3-QA-02): a blocked remove (e.g. the staged exe held
    // by a handle without FILE_SHARE_DELETE) is real state that the caller
    // must see. NotFound means the file is already gone — that is success.
    let discard_staged = || -> Vec<String> {
        let mut failures = Vec::new();
        for p in [&staged.tmp_exe, tmp_obj, &tmp_receipt] {
            match std::fs::remove_file(p) {
                Ok(()) => {}
                // Already gone: reclaimed earlier on this path or by the
                // caller — that is success.
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                // A directory occupying the path is a USER placeholder, not
                // our staged file; it is never ours to remove and never a
                // cleanup failure.
                Err(_) if p.is_dir() => {}
                Err(e) => failures.push(format!("{}: {}", p.display(), e)),
            }
        }
        failures
    };
    // A cleanup failure changes what the caller must be told: the original
    // failure stands, the transaction's commit state is stated explicitly,
    // and every residual own path plus its real OS error is listed.
    let with_cleanup = |original: Diagnostic, cleanup_failures: Vec<String>| -> Diagnostic {
        if cleanup_failures.is_empty() {
            original
        } else {
            link_diag(
                "link.cleanup",
                format!(
                    "{}; transaction NOT committed; own staged files remain (release the blocking handles, then remove): {}",
                    original.render("", "").trim_end(),
                    cleanup_failures.join("; ")
                ),
            )
        }
    };

    if let Err(e) = std::fs::write(&tmp_receipt, receipt_body) {
        let cleanup = discard_staged();
        return Err(with_cleanup(
            link_diag(
                "link.receipt",
                format!("stage receipt at {}: {}", tmp_receipt.display(), e),
            ),
            cleanup,
        ));
    }

    // Back up existing finals (files only).
    let mut backups: Vec<(PathBuf, PathBuf)> = Vec::new(); // (bak, final)
    for final_path in [&staged.exe, obj_path, &receipt_path] {
        if final_path.is_file() {
            let bak = companion(final_path, &format!(".bak-{tag}"));
            match std::fs::rename(final_path, &bak) {
                Ok(()) => backups.push((bak, final_path.to_path_buf())),
                Err(e) => {
                    // Same restore-reporting contract as the placement path
                    // (P741P3-R1): a blocked rollback must surface the
                    // surviving backup paths instead of swallowing the error.
                    let cleanup = discard_staged();
                    let mut restore_failures: Vec<String> = Vec::new();
                    for (bak, final_path) in backups.iter().rev() {
                        if let Err(re) = std::fs::rename(bak, final_path) {
                            restore_failures.push(format!(
                                "restore {} from backup {}: {}",
                                final_path.display(),
                                bak.display(),
                                re
                            ));
                        }
                    }
                    let base = if restore_failures.is_empty() {
                        link_diag(
                            "link.replace",
                            format!("backing up {}: {}", final_path.display(), e),
                        )
                    } else {
                        link_diag(
                            "link.restore",
                            format!(
                                "backing up {}: {}; rollback incomplete, previous artifacts kept at the listed backup paths: {}",
                                final_path.display(),
                                e,
                                restore_failures.join("; ")
                            ),
                        )
                    };
                    return Err(with_cleanup(base, cleanup));
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
            // Committed: exe/obj/receipt are the new build. Dropping the
            // backups is post-commit cleanup — a blocked removal must not be
            // reported as an ordinary success (R3-QA-02).
            let mut cleanup_failures = Vec::new();
            for (bak, _) in &backups {
                if let Err(e) = std::fs::remove_file(bak) {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        cleanup_failures.push(format!("{}: {}", bak.display(), e));
                    }
                }
            }
            if cleanup_failures.is_empty() {
                Ok(LinkReceipt {
                    linker: staged.linker.clone(),
                    obj: obj_path.to_path_buf(),
                    exe: staged.exe.clone(),
                    receipt_path,
                    entry_symbol: staged.entry_symbol.clone(),
                })
            } else {
                Err(link_diag(
                    "link.cleanup",
                    format!(
                        "publish COMMITTED (exe/obj/receipt are the new build); backup cleanup incomplete (release the blocking handles, then remove): {}",
                        cleanup_failures.join("; ")
                    ),
                ))
            }
        }
        Err(d) => {
            // Restore backups first (a rename replaces the placed new file);
            // then remove only placed finals with no surviving old copy: a
            // restored final holds the old bytes again, a restore-failed
            // final keeps the new bytes while its backup preserves the old
            // ones (reported below), and everything else this call placed is
            // a stray to delete. Removal errors are collected, not swallowed
            // (R3-QA-02).
            let mut restored_ok: std::collections::BTreeSet<PathBuf> =
                std::collections::BTreeSet::new();
            let mut restore_failed: std::collections::BTreeSet<PathBuf> =
                std::collections::BTreeSet::new();
            let mut restore_failures: Vec<String> = Vec::new();
            for (bak, final_path) in backups.iter().rev() {
                match std::fs::rename(bak, final_path) {
                    Ok(()) => {
                        restored_ok.insert(final_path.clone());
                    }
                    Err(e) => {
                        restore_failed.insert(final_path.clone());
                        restore_failures.push(format!(
                            "restore {} from backup {}: {}",
                            final_path.display(),
                            bak.display(),
                            e
                        ));
                    }
                }
            }
            let mut cleanup_failures: Vec<String> = Vec::new();
            for p in &placed {
                if restored_ok.contains(p) || restore_failed.contains(p) {
                    continue;
                }
                if let Err(e) = std::fs::remove_file(p) {
                    if e.kind() != std::io::ErrorKind::NotFound {
                        cleanup_failures.push(format!("{}: {}", p.display(), e));
                    }
                }
            }
            cleanup_failures.extend(discard_staged());
            if !restore_failures.is_empty() {
                // P741P4-R2: a compound failure (rollback AND staged cleanup
                // both blocked) must report every residual own path, not
                // just the restore problems.
                let mut parts = vec![format!(
                    "{}; rollback incomplete, previous artifacts kept at the listed backup paths: {}",
                    d.render("", "").trim_end(),
                    restore_failures.join("; ")
                )];
                if !cleanup_failures.is_empty() {
                    parts.push(format!(
                        "own staged files also remain (release the blocking handles, then remove): {}",
                        cleanup_failures.join("; ")
                    ));
                }
                return Err(link_diag("link.restore", parts.join("; ")));
            }
            Err(with_cleanup(d, cleanup_failures))
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

    /// A `start /b` descendant inherits cmd's stdout/stderr handles and keeps
    /// them open after the direct child exits. R2-QA-02: the r2 executor
    /// blocked in the reader joins far past the deadline (the review measured
    /// ~115s against the 60s cap). The collection phase must hit the same
    /// deadline, and the job guard must take the descendant down with it —
    /// no background ping may survive this test.
    #[test]
    #[cfg(windows)]
    fn collect_deadline_when_descendant_holds_pipe() {
        let started = Instant::now();
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start /b ping -n 30 127.0.0.1 & exit"]);
        let err = run_with_deadline(&mut cmd, Duration::from_secs(1)).err();
        let elapsed = started.elapsed();
        assert!(
            matches!(
                err,
                Some(SubprocError::Deadline) | Some(SubprocError::DeadlineCollect)
            ),
            "expected a deadline error, got {err:?}"
        );
        assert!(
            elapsed < Duration::from_secs(3),
            "collection deadline took {elapsed:?} (must fire within 3s)"
        );
        // The job guard closed with the call: no descendant may linger.
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let out = Command::new("tasklist")
                .args(["/FI", "IMAGENAME eq ping.exe", "/NH"])
                .output()
                .expect("tasklist");
            let listing = String::from_utf8_lossy(&out.stdout).to_string();
            if !listing.contains("ping.exe") {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "descendant ping.exe survived the job guard: {listing}"
            );
            std::thread::sleep(Duration::from_millis(200));
        }
    }

    /// Positive case: a descendant that exits promptly must not disturb the
    /// success path — full output (the echo that ran before the descendant
    /// finished), exit code, and prompt return.
    #[test]
    #[cfg(windows)]
    fn descendant_exits_promptly_success_path() {
        let started = Instant::now();
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "echo done & start /b ping -n 1 127.0.0.1"]);
        let (code, out) = run_with_deadline(&mut cmd, SUBPROC_DEADLINE).expect("complete");
        assert_eq!(code, Some(0));
        assert!(out.contains("done"), "output: {out}");
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "took {:?}",
            started.elapsed()
        );
    }

    /// AC-17 evidence at the real production deadline (60s): the exact
    /// run_exe configuration against a descendant holding the pipes for
    /// 120s must fail with link.deadline at ~60s instead of blocking until
    /// the descendant exits (~115s+ in the r2 review measurement). Ignored
    /// by default (60s runtime); run explicitly for the acceptance report:
    /// `cargo test --lib -- --ignored`
    #[test]
    #[cfg(windows)]
    #[ignore = "60s acceptance evidence for AC-17; run explicitly"]
    fn run_exe_60s_deadline_bounds_descendant_pipe_hold() {
        let started = Instant::now();
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "start /b ping -n 120 127.0.0.1 & exit"]);
        let err = run_with_deadline(&mut cmd, SUBPROC_DEADLINE).err();
        let elapsed = started.elapsed();
        assert!(
            matches!(err, Some(SubprocError::DeadlineCollect)),
            "expected DeadlineCollect at the production deadline, got {err:?} after {elapsed:?}"
        );
        assert!(
            (50..70).contains(&elapsed.as_secs()),
            "deadline fired at {elapsed:?}, expected ~60s"
        );
    }

    // -- publish transaction failure matrix (R2-QA-03) ------------------------

    /// Byte content of the fake staged artifacts; publish only renames, so
    /// real PE bytes are not needed at this layer (the CLI tests cover the
    /// real-PE paths end to end).
    const STAGE_EXE: &[u8] = b"staged-exe-bytes-v1";
    const STAGE_OBJ: &[u8] = b"staged-obj-bytes-v1";
    const OLD_EXE: &[u8] = b"old-exe-bytes";
    const OLD_OBJ: &[u8] = b"old-obj-bytes";
    const OLD_RECEIPT: &[u8] = b"old-receipt-bytes";

    fn staged_paths(dir: &Path) -> (PathBuf, PathBuf) {
        let exe = dir.join("tx.exe");
        let tmp_exe = companion(&exe, &format!(".tmp-{}", staging_tag()));
        let tmp_obj = companion(&dir.join("tx.obj"), &format!(".tmp-{}", staging_tag()));
        (tmp_exe, tmp_obj)
    }

    fn make_staged(dir: &Path) -> (StagedLink, PathBuf) {
        let (tmp_exe, tmp_obj) = staged_paths(dir);
        std::fs::write(&tmp_exe, STAGE_EXE).unwrap();
        std::fs::write(&tmp_obj, STAGE_OBJ).unwrap();
        let staged = StagedLink {
            tmp_exe: tmp_exe.clone(),
            exe: dir.join("tx.exe"),
            linker: PathBuf::from("rust-lld"),
            entry_symbol: "ac_start".to_string(),
        };
        (staged, tmp_obj)
    }

    fn make_finals(dir: &Path) {
        std::fs::write(dir.join("tx.exe"), OLD_EXE).unwrap();
        std::fs::write(dir.join("tx.obj"), OLD_OBJ).unwrap();
        std::fs::write(dir.join("tx.ac-link.txt"), OLD_RECEIPT).unwrap();
    }

    fn strays(dir: &Path) -> Vec<String> {
        dir.read_dir()
            .unwrap()
            .filter_map(|e| {
                let p = e.unwrap().path();
                let name = p.file_name().unwrap().to_string_lossy().to_string();
                let is_staged_name = name.contains(".tmp-ac741") || name.contains(".bak-ac741");
                if is_staged_name && p.is_file() {
                    Some(name)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Receipt *staging* failure (directory occupying the tmp receipt path):
    /// the already-linked staged exe, the staged obj and the tmp receipt must
    /// all be reclaimed and the old artifact set must survive byte-identical.
    /// R2-QA-03: the r2 code left the staged exe behind here.
    #[test]
    #[cfg(windows)]
    fn receipt_stage_failure_cleans_staged_and_keeps_old_artifacts() {
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("stage-fail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_finals(&dir);
        let (staged, tmp_obj) = make_staged(&dir);
        // Occupy the tmp receipt path (same process => same staging tag).
        let tmp_receipt = companion(
            &dir.join("tx.ac-link.txt"),
            &format!(".tmp-{}", staging_tag()),
        );
        std::fs::create_dir(&tmp_receipt).unwrap();

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        let d = err.expect("publish must fail");
        assert_eq!(d.code, "link.receipt", "{}", d.render("f", ""));

        assert_eq!(std::fs::read(dir.join("tx.exe")).unwrap(), OLD_EXE);
        assert_eq!(std::fs::read(dir.join("tx.obj")).unwrap(), OLD_OBJ);
        assert_eq!(
            std::fs::read(dir.join("tx.ac-link.txt")).unwrap(),
            OLD_RECEIPT
        );
        assert!(!staged.tmp_exe.exists(), "staged exe left behind");
        assert!(!tmp_obj.exists(), "staged obj left behind");
        assert!(strays(&dir).is_empty(), "strays: {:?}", strays(&dir));
        // The user's placeholder directory must survive.
        assert!(tmp_receipt.is_dir());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Same failure with no previous artifacts: no half-published exe/obj may
    /// remain (AC-18 "无旧制品时失败不留半成品").
    #[test]
    #[cfg(windows)]
    fn receipt_stage_failure_without_prior_artifacts_leaves_nothing() {
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("stage-fail-empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (staged, tmp_obj) = make_staged(&dir);
        let tmp_receipt = companion(
            &dir.join("tx.ac-link.txt"),
            &format!(".tmp-{}", staging_tag()),
        );
        std::fs::create_dir(&tmp_receipt).unwrap();

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        assert_eq!(err.expect("must fail").code, "link.receipt");

        assert!(!dir.join("tx.exe").exists());
        assert!(!dir.join("tx.obj").exists());
        assert!(strays(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Backup failure (directory occupying the exe backup name): the
    /// transaction aborts before any publish, staged files are reclaimed,
    /// old artifacts untouched, and the user's placeholder survives.
    #[test]
    #[cfg(windows)]
    fn backup_failure_aborts_cleanly() {
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("backup-fail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_finals(&dir);
        let (staged, tmp_obj) = make_staged(&dir);
        let exe_bak = companion(&dir.join("tx.exe"), &format!(".bak-{}", staging_tag()));
        std::fs::create_dir(&exe_bak).unwrap();

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        assert_eq!(err.expect("must fail").code, "link.replace");

        assert_eq!(std::fs::read(dir.join("tx.exe")).unwrap(), OLD_EXE);
        assert_eq!(std::fs::read(dir.join("tx.obj")).unwrap(), OLD_OBJ);
        assert_eq!(
            std::fs::read(dir.join("tx.ac-link.txt")).unwrap(),
            OLD_RECEIPT
        );
        assert!(!staged.tmp_exe.exists());
        assert!(!tmp_obj.exists());
        assert!(strays(&dir).is_empty());
        assert!(exe_bak.is_dir(), "user placeholder must survive");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Publish failure at the exe placement (exe final path occupied by a
    /// directory): nothing published, obj/receipt restored, no strays.
    #[test]
    #[cfg(windows)]
    fn publish_exe_failure_keeps_old_artifacts() {
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("pub-exe-fail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("tx.obj"), OLD_OBJ).unwrap();
        std::fs::write(dir.join("tx.ac-link.txt"), OLD_RECEIPT).unwrap();
        std::fs::create_dir(dir.join("tx.exe")).unwrap(); // user placeholder
        let (staged, tmp_obj) = make_staged(&dir);

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        assert_eq!(err.expect("must fail").code, "link.replace");

        assert_eq!(std::fs::read(dir.join("tx.obj")).unwrap(), OLD_OBJ);
        assert_eq!(
            std::fs::read(dir.join("tx.ac-link.txt")).unwrap(),
            OLD_RECEIPT
        );
        assert!(dir.join("tx.exe").is_dir(), "user placeholder must survive");
        assert!(strays(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Publish failure at the obj placement: the already-placed new exe is
    /// rolled back to the old exe (byte-identical), the receipt is restored,
    /// and the user's directory placeholder survives.
    #[test]
    #[cfg(windows)]
    fn publish_obj_failure_restores_exe_and_receipt() {
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("pub-obj-fail");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_finals(&dir);
        let (staged, tmp_obj) = make_staged(&dir);
        std::fs::remove_file(dir.join("tx.obj")).unwrap();
        std::fs::create_dir(dir.join("tx.obj")).unwrap(); // user placeholder

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        assert_eq!(err.expect("must fail").code, "link.replace");

        assert_eq!(std::fs::read(dir.join("tx.exe")).unwrap(), OLD_EXE);
        assert_eq!(
            std::fs::read(dir.join("tx.ac-link.txt")).unwrap(),
            OLD_RECEIPT
        );
        assert!(dir.join("tx.obj").is_dir(), "user placeholder must survive");
        assert!(strays(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// R3-QA-01: containment failure is a CONTROLLED rejection — the child
    /// is killed before its first instruction, so a long-running command
    /// never actually starts (no ping may leak), and the error names the
    /// cause instead of degrading into detached readers.
    #[test]
    #[cfg(windows)]
    fn containment_failure_is_controlled_rejection() {
        use std::time::Instant;
        let started = Instant::now();
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", "ping", "-n", "30", "127.0.0.1"]);
        let err = run_with_deadline_inner(&mut cmd, Duration::from_secs(10), |_| None).err();
        let elapsed = started.elapsed();
        assert!(
            matches!(err, Some(SubprocError::ContainmentUnavailable(_))),
            "expected containment rejection, got {err:?}"
        );
        assert!(
            elapsed < Duration::from_secs(3),
            "rejection took {elapsed:?}"
        );
        // The suspended child was killed before running: ping must not exist.
        let out = Command::new("tasklist")
            .args(["/FI", "IMAGENAME eq ping.exe", "/NH"])
            .output()
            .expect("tasklist");
        assert!(
            !String::from_utf8_lossy(&out.stdout).contains("ping.exe"),
            "child leaked past containment failure"
        );
    }

    /// R3-QA-02: a cleanup blocked by a real sharing lock (staged exe opened
    /// WITHOUT FILE_SHARE_DELETE, receipt staging path occupied by a
    /// directory) must surface as `link.cleanup` carrying the original
    /// failure, the residual own path and the real OS error, with the
    /// commit state stated; the previous artifacts stay untouched and the
    /// lock holder can clean up after releasing.
    #[test]
    #[cfg(windows)]
    fn cleanup_failure_reports_residual_staged_exe() {
        use std::os::windows::fs::OpenOptionsExt;
        let dir = std::env::temp_dir()
            .join("ac741-link-tx")
            .join("cleanup-locked");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        make_finals(&dir);
        let (staged, tmp_obj) = make_staged(&dir);
        let tmp_receipt = companion(
            &dir.join("tx.ac-link.txt"),
            &format!(".tmp-{}", staging_tag()),
        );
        std::fs::create_dir(&tmp_receipt).unwrap();
        // Hold the staged exe without FILE_SHARE_DELETE (share_mode READ only).
        let lock = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(1)
            .open(&staged.tmp_exe)
            .expect("lock staged exe");

        let err = publish_artifacts(&staged, &tmp_obj, &dir.join("tx.obj")).err();
        let d = err.expect("must fail");
        assert_eq!(d.code, "link.cleanup", "{}", d.render("f", ""));
        let msg = d.render("f", "");
        assert!(msg.contains("link.receipt"), "original stage lost: {msg}");
        assert!(msg.contains("NOT committed"), "commit state unclear: {msg}");
        assert!(
            msg.contains(&staged.tmp_exe.display().to_string()),
            "residual staged exe path missing: {msg}"
        );
        assert!(msg.contains("os error"), "real OS error missing: {msg}");

        // Previous artifacts untouched.
        assert_eq!(std::fs::read(dir.join("tx.exe")).unwrap(), OLD_EXE);
        assert_eq!(std::fs::read(dir.join("tx.obj")).unwrap(), OLD_OBJ);
        assert_eq!(
            std::fs::read(dir.join("tx.ac-link.txt")).unwrap(),
            OLD_RECEIPT
        );
        assert!(tmp_receipt.is_dir(), "user placeholder must survive");

        // Release the lock: the residual staged exe becomes removable.
        drop(lock);
        std::fs::remove_file(&staged.tmp_exe).expect("cleanup after release");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
