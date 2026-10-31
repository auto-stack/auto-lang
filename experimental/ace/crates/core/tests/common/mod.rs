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
    let mut owned_compiler = Command::new(&lld);
    owned_compiler.args(&args);
    let out = auto_tool_execution::run_command_output(owned_compiler)
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

/// Outcome of a deadline-guarded process run.  runs mean the
/// single deadline was exceeded by the process or its output collection;
/// the contained tree was terminated and reaped, and no success is faked
/// (R2/R3-QA-01..02).
pub struct DeadlinedRun {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    /// Always false in the delegated implementation: reader errors surface
    /// through the executor's error variants instead of being flattened
    /// into output (kept for API stability of existing assertions).
    pub drain_error: bool,
}

/// Run a command under the 741-proven contained executor
/// ([], R3-QA-01): controlled
/// suspended start, Job-Object subtree containment BEFORE the first
/// instruction, one deadline covering process exit AND pipe-EOF collection,
/// byte draining with lossy UTF-8 conversion, and full tree + reader
/// reclamation on every exit (success, deadline, spawn/containment/wait).
/// There is deliberately no test-only lifecycle here — the tests consume
/// the same executor the linker itself uses.
pub fn run_deadlined(
    mut cmd: std::process::Command,
    deadline: std::time::Duration,
) -> std::io::Result<DeadlinedRun> {
    use ace_core::link::{run_deadlined_split, SubprocError};
    match run_deadlined_split(&mut cmd, deadline) {
        Ok((code, out, err)) => Ok(DeadlinedRun {
            code: code.unwrap_or(-1),
            stdout: out,
            stderr: err,
            timed_out: false,
            drain_error: false,
        }),
        Err(SubprocError::Deadline) | Err(SubprocError::DeadlineCollect) => Ok(DeadlinedRun {
            code: -1,
            stdout: String::new(),
            stderr: String::new(),
            timed_out: true,
            drain_error: false,
        }),
        Err(SubprocError::Spawn(e)) => Err(e),
        Err(SubprocError::Wait(e)) => Err(e),
        Err(SubprocError::ContainmentUnavailable(reason)) => {
            // A process that cannot be bounded is a controlled refusal, not
            // a success — surface it as a pseudo-io error for callers.
            Err(std::io::Error::other(format!(
                "containment unavailable: {reason}"
            )))
        }
        Err(SubprocError::OutputCapExceeded {
            stream,
            observed,
            cap,
        }) => {
            // Unreachable through this legacy wrapper (caps are disabled);
            // kept exhaustive so a future cap use cannot silently pass.
            Err(std::io::Error::other(format!(
                "{stream} exceeded its capture cap ({observed} > {cap})"
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// PLAN-770 text runtime support (T-05): build the runtime staticlib and
// refresh its contract sidecar next to it. The sidecar format mirrors
// `runtime/text/tests/contract.rs` (the runtime workspace's own
// consistency test); ac-core tests write it so a CLI build has a valid
// contract to consume without ordering against that workspace's test run.
// ---------------------------------------------------------------------------

/// Minimal SHA-256 (FIPS 180-4), no external deps. Same implementation as
/// `runtime/text/tests/contract.rs` and the CLI's contract gate.
pub fn sha256_hex(data: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    let bitlen = (data.len() as u64) * 8;
    let mut msg = data.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bitlen.to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[i * 4],
                chunk[i * 4 + 1],
                chunk[i * 4 + 2],
                chunk[i * 4 + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }
    h.iter().map(|x| format!("{:08x}", x)).collect()
}

/// The ac.runtime.text.v1 capability + the built runtime staticlib with a
/// refreshed `<lib>.contract.json` sidecar (cached per test process).
pub fn text_runtime_lib() -> PathBuf {
    use std::sync::OnceLock;
    static CACHE: OnceLock<PathBuf> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/text-runtime");
            let mut cmd = Command::new("cargo");
            cmd.args(["build", "-p", "ace-text-runtime", "--profile", "runtime-release", "--locked"])
                .current_dir(&root);
            let r = run_deadlined(cmd, Duration::from_secs(300))
                .expect("spawn cargo for the text runtime");
            assert!(!r.timed_out, "text runtime build exceeded 300s");
            assert_eq!(r.code, 0, "text runtime build failed:\n{}", r.stderr);
            let lib = root.join("../../target/runtime-release/ac_text_runtime.lib");
            assert!(lib.is_file(), "runtime lib missing at {}", lib.display());
            let bytes = std::fs::read(&lib).expect("read runtime lib");
            let contract = root.join("../../target/runtime-release/ac_text_runtime.lib.contract.json");
            std::fs::write(
                &contract,
                format!(
                    "{{\n  \"api\": \"ac.runtime.text.v1\",\n  \"api_revision\": 1,\n  \"target\": \"x86_64-pc-windows-msvc\",\n  \"lib\": \"ac_text_runtime.lib\",\n  \"lib_sha256\": \"{}\",\n  \"symbol_count\": 23\n}}\n",
                    sha256_hex(&bytes)
                ),
            )
            .expect("write contract sidecar");
            lib
        })
        .clone()
}

/// Copy the runtime lib + sidecar into a scratch dir (contract-mutation
/// cases must not touch the shared cache).
pub fn text_runtime_pair_copy(dir: &Path) -> (PathBuf, PathBuf) {
    let lib = text_runtime_lib();
    let lib2 = dir.join("rt.lib");
    let contract2 = dir.join("rt.lib.contract.json");
    std::fs::copy(&lib, &lib2).expect("copy runtime lib");
    let contract = lib.with_file_name("ac_text_runtime.lib.contract.json");
    std::fs::copy(&contract, &contract2).expect("copy contract sidecar");
    (lib2, contract2)
}

/// Write the sidecar with specific field values (mutation oracle).
pub fn write_sidecar(contract: &Path, api: &str, revision: i64, target: &str, sha: &str) {
    std::fs::write(
        contract,
        format!(
            "{{\n  \"api\": \"{}\",\n  \"api_revision\": {},\n  \"target\": \"{}\",\n  \"lib\": \"rt.lib\",\n  \"lib_sha256\": \"{}\",\n  \"symbol_count\": 23\n}}\n",
            api, revision, target, sha
        ),
    )
    .expect("write sidecar");
}

/// The real SHA-256 of the runtime lib (for valid sidecars).
pub fn text_runtime_sha() -> String {
    sha256_hex(&std::fs::read(text_runtime_lib()).expect("read runtime lib"))
}
