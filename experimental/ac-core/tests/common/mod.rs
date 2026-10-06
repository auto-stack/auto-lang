//! Shared test support for PLAN-741/745 native tests: link a COFF object
//! into a PE with rust-lld and run it with a deadline, plus the deadline
//! executor shared by the source test families. Windows only — the native
//! gate must run on the real platform (plan §6).

// Each test binary includes this module wholesale but uses a different
// subset of helpers; unused-in-this-binary is expected, not drift.
#![cfg(windows)]
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

/// Locate rust-lld from the active sysroot.
pub fn find_rust_lld() -> Result<PathBuf, String> {
    let out = Command::new("rustc")
        .arg("--print")
        .arg("sysroot")
        .output()
        .map_err(|e| format!("rustc --print sysroot: {}", e))?;
    let sysroot = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let candidate = Path::new(&sysroot).join("lib/rustlib/x86_64-pc-windows-msvc/bin/rust-lld.exe");
    if candidate.is_file() {
        Ok(candidate)
    } else {
        Err(format!("rust-lld.exe not found under {}", sysroot))
    }
}

/// Locate a directory containing the Windows SDK `kernel32.lib` (um/x64).
/// Order: `AC_SDK_UM_LIB` override, then registry KitsRoot10, then known
/// program-file locations.
pub fn find_sdk_um_dir() -> Result<PathBuf, String> {
    if let Ok(env_dir) = std::env::var("AC_SDK_UM_LIB") {
        let p = PathBuf::from(env_dir);
        if p.join("kernel32.lib").is_file() {
            return Ok(p);
        }
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
                    let root = line[pos + "REG_SZ".len()..].trim();
                    roots.push(PathBuf::from(root));
                }
            }
        }
    }
    roots.push(PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10"));
    roots.push(PathBuf::from(r"D:\Windows Kits\10"));
    for root in roots {
        let lib = root.join("Lib");
        let Ok(entries) = std::fs::read_dir(&lib) else {
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
    Err("no Windows SDK um/x64 kernel32.lib found (set AC_SDK_UM_LIB)".to_string())
}

/// Link `obj` with `/entry:ac_start`, run the exe, return its exit code.
/// Subprocesses get a hard deadline (plan §6).
pub fn link_and_run(case: &str, obj: &[u8]) -> Result<i32, String> {
    let dir = std::env::temp_dir().join("ac741-native").join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).map_err(|e| format!("mkdir {}: {}", dir.display(), e))?;
    let obj_path = dir.join("ac.obj");
    let exe_path = dir.join("ac.exe");
    std::fs::write(&obj_path, obj).map_err(|e| format!("write obj: {}", e))?;

    let lld = find_rust_lld()?;
    let sdk = find_sdk_um_dir()?;
    let args: Vec<String> = vec![
        "-flavor".into(),
        "link".into(),
        "/nologo".into(),
        "/entry:ac_start".into(),
        "/subsystem:console".into(),
        format!("/out:{}", exe_path.display()),
        format!("{}", obj_path.display()),
        format!("/libpath:{}", sdk.display()),
        "kernel32.lib".into(),
    ];
    let out = Command::new(&lld)
        .args(&args)
        .output()
        .map_err(|e| format!("run rust-lld: {}", e))?;
    if !out.status.success() {
        return Err(format!(
            "rust-lld failed: {}\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let mut child = Command::new(&exe_path)
        .spawn()
        .map_err(|e| format!("spawn {}: {}", exe_path.display(), e))?;
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return status
                    .code()
                    .ok_or_else(|| "process terminated by signal".to_string());
            }
            Ok(None) => {
                if Instant::now() > deadline {
                    let _ = child.kill();
                    return Err(format!("{} exceeded 15s deadline", case));
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            Err(e) => return Err(format!("wait: {}", e)),
        }
    }
}

/// Full prototype pipeline for a fixture: bind -> verify -> entry -> lower.
pub fn build_fixture(fixture_rel: &str, entry_id: &str) -> Result<Vec<u8>, String> {
    use auto_ac_prototype::bind_source;
    use auto_ac_prototype::{native, verify};
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = repo.join(fixture_rel);
    let src =
        std::fs::read_to_string(&path).map_err(|e| format!("read {}: {}", path.display(), e))?;
    let bundle = bind_source(&src).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(&fixture_rel, &src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let checked = verify::verify(bundle).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(&fixture_rel, &src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    native::capabilities_check(&checked, &[]).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(&fixture_rel, &src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    let sel = native::find_entry(&checked, entry_id).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(&fixture_rel, &src))
            .collect::<Vec<_>>()
            .join("\n")
    })?;
    native::lower_object(&checked, &sel).map_err(|ds| {
        ds.iter()
            .map(|d| d.render(&fixture_rel, &src))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

/// Outcome of a deadline-guarded process run. `timed_out` runs carry the
/// partial output drained until the kill and the child's exit code if it
/// had already exited — never a fake success (R2-QA-02).
pub struct DeadlinedRun {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    /// A pipe reader reported an IO error; the partial bytes drained until
    /// then are still in stdout/stderr (never silently empty).
    pub drain_error: bool,
}

/// Run a command with piped stdio under ONE hard deadline that covers both
/// process exit and output collection (R1-QA-05 + R2-QA-02):
///
/// - both pipes are drained BY BYTE from dedicated reader threads that
///   report through channels — invalid UTF-8 survives as replacement
///   characters instead of swallowing the whole stream, and no join is
///   ever bounded by the readers (a descendant holding the pipe cannot
///   extend the wait past the deadline);
/// - on expiry the child is killed and reaped, whatever arrived by then is
///   returned with `timed_out = true`;
/// - every exit path waits for the process, so nothing is left running by
///   this call. Reader threads whose pipes never close exit on their own
///   once the senders are dropped.
pub fn run_deadlined(
    mut cmd: std::process::Command,
    deadline: std::time::Duration,
) -> std::io::Result<DeadlinedRun> {
    use std::io::Read;
    use std::process::Stdio;
    use std::sync::mpsc;

    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let out_pipe = child.stdout.take().expect("piped stdout");
    let err_pipe = child.stderr.take().expect("piped stderr");
    // One (bytes, ok) message per pipe, ever; channels make the wait loop
    // free of joins entirely.
    let (out_tx, out_rx) = mpsc::channel::<(Vec<u8>, bool)>();
    let (err_tx, err_rx) = mpsc::channel::<(Vec<u8>, bool)>();
    let reader = |mut pipe: Box<dyn Read + Send>, tx: mpsc::Sender<(Vec<u8>, bool)>| {
        std::thread::spawn(move || {
            let mut buf = Vec::new();
            // read_to_end keeps whatever arrived even on error.
            let ok = pipe.read_to_end(&mut buf).is_ok();
            let _ = tx.send((buf, ok));
        })
    };
    let _out_reader = reader(Box::new(out_pipe), out_tx);
    let _err_reader = reader(Box::new(err_pipe), err_tx);

    let start = std::time::Instant::now();
    let mut out_buf: Vec<u8> = Vec::new();
    let mut err_buf: Vec<u8> = Vec::new();
    let mut drain_error = false;
    let mut out_done = false;
    let mut err_done = false;
    let mut exit_code: Option<i32> = None;
    loop {
        if !out_done {
            if let Ok((bytes, ok)) = out_rx.try_recv() {
                if !ok {
                    drain_error = true;
                }
                out_buf = bytes;
                out_done = true;
            }
        }
        if !err_done {
            if let Ok((bytes, ok)) = err_rx.try_recv() {
                if !ok {
                    drain_error = true;
                }
                err_buf = bytes;
                err_done = true;
            }
        }
        if exit_code.is_some() && out_done && err_done {
            return Ok(DeadlinedRun {
                code: exit_code.unwrap_or(-1),
                stdout: String::from_utf8_lossy(&out_buf).into_owned(),
                stderr: String::from_utf8_lossy(&err_buf).into_owned(),
                timed_out: false,
                drain_error,
            });
        }
        if start.elapsed() >= deadline {
            // Deadline governs execution AND collection (R2-QA-02): kill
            // and reap the child, sweep whatever arrived, and report the
            // timeout honestly — never wait on the readers.
            let _ = child.kill();
            let code = child.wait().ok().and_then(|st| st.code()).unwrap_or(-1);
            if !out_done {
                if let Ok((bytes, ok)) = out_rx.try_recv() {
                    if !ok {
                        drain_error = true;
                    }
                    out_buf = bytes;
                }
            }
            if !err_done {
                if let Ok((bytes, ok)) = err_rx.try_recv() {
                    if !ok {
                        drain_error = true;
                    }
                    err_buf = bytes;
                }
            }
            return Ok(DeadlinedRun {
                code,
                stdout: String::from_utf8_lossy(&out_buf).into_owned(),
                stderr: String::from_utf8_lossy(&err_buf).into_owned(),
                timed_out: true,
                drain_error,
            });
        }
        if exit_code.is_none() {
            if let Some(status) = child.try_wait()? {
                exit_code = Some(status.code().unwrap_or(-1));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}
