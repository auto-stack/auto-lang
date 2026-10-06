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

/// Outcome of a deadline-guarded process run (QA-05): `timed_out` runs carry
/// the partial output drained until the kill, not a fake success.
pub struct DeadlinedRun {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// Run a command with piped stdio under a hard deadline, DRAINING BOTH PIPES
/// CONCURRENTLY while waiting — a full pipe can never wedge the wait (the
/// poll-before-drain ordering deadlocked at a 4096-char stderr in review
/// QA-05's control). On expiry the child is killed and reaped; every exit
/// path waits for the process, so nothing is left running. This mirrors the
/// link.rs executor discipline (bounded waits, drained output, reaped
/// children) for the source test families.
pub fn run_deadlined(
    mut cmd: std::process::Command,
    deadline: std::time::Duration,
) -> std::io::Result<DeadlinedRun> {
    use std::io::Read;
    use std::process::Stdio;
    let mut child = cmd.stdout(Stdio::piped()).stderr(Stdio::piped()).spawn()?;
    let mut out_pipe = child.stdout.take().expect("piped stdout");
    let mut err_pipe = child.stderr.take().expect("piped stderr");
    let out_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out_pipe.read_to_string(&mut s);
        s
    });
    let err_t = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = err_pipe.read_to_string(&mut s);
        s
    });
    let start = std::time::Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let code = status.code().unwrap_or(-1);
            return Ok(DeadlinedRun {
                code,
                stdout: out_t.join().unwrap_or_default(),
                stderr: err_t.join().unwrap_or_default(),
                timed_out: false,
            });
        }
        if start.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait(); // reap; closes the pipes for the drain threads
            return Ok(DeadlinedRun {
                code: -1,
                stdout: out_t.join().unwrap_or_default(),
                stderr: err_t.join().unwrap_or_default(),
                timed_out: true,
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(15));
    }
}
