//! PLAN-1041 OUT-01 (AC legs): the common Auto output source
//! (`stdlib/auto/platform/io.at`, inlined like the 1039 helper) compiled
//! to a REAL Windows x64 PE with the platform shim and executed — the
//! redirect path writes byte-exact UTF-8 (NUL is content), both streams
//! are real OS handles, classification sees a pipe (kind 2), and the
//! console write16 op is absent from a byte-sink run. Import audit keeps
//! the kernel32 closure honest (OUT-05 covers the audit side fully).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_native::console::{console_authorize, console_compile, console_native_run};
use std::process::{Command, Stdio};

const IO_AT: &str = include_str!("../../../../../../stdlib/auto/platform/io.at");

fn platform_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
    ]
}

fn temp_workdir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("p1041-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn build_and_run(src: &str) -> (Option<i32>, Vec<u8>, Vec<u8>) {
    let mir = console_compile(src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("out01");
    // Real OS observation: pipe both streams so classification sees byte
    // sinks and the harness reads exact bytes (never host capture).
    let run = console_native_run(&mir, "d_main", &dir).expect("run");
    let _ = std::fs::remove_dir_all(&dir);
    (run.exit_code, run.stdout, run.stderr)
}

/// OUT-01 AC/Windows: byte-exact UTF-8 + NUL on both redirected streams;
/// confirmed UTF-8 byte progress rides the exit code; kind == 2 for pipes.
#[test]
fn out01_ac_windows_redirect_bytes_nul_both_streams() {
    let src = format!(
        "{IO_AT}\nfn main() int {{\n  let h = out_stdout()\n  if out_kind(h) != 2 {{ return -1 }}\n  let a = out_text(h, \"ab\\0cd你好\\n\")\n  let b = out_bytes(h, \"Z中\")\n  let e = out_stderr()\n  let c = out_text(e, \"err\\0!\")\n  return a + b + c\n}}\n"
    );
    let (code, out, err) = build_and_run(&src);
    let expect = "ab\0cd你好\n".len() + "Z中".len() + "err\0!".len();
    assert_eq!(code, Some(expect as i32), "confirmed UTF-8 bytes");
    assert_eq!(out, "ab\0cd你好\nZ中".as_bytes());
    assert_eq!(err, b"err\0!");
}

/// OUT-01 AC/Windows: >4096-byte input crosses the chunk loop on the real
/// byte sink; the full string arrives verbatim (independent byte oracle).
#[test]
fn out01_ac_windows_large_input_chunked() {
    let mut big = String::from("头部");
    for _ in 0..2000 {
        big.push('中'); // 3 bytes each: total > 4096, chunk boundary mid-run
    }
    big.push_str("尾部");
    let mut expect_bytes = big.as_bytes().to_vec();
    let total = big.len();
    let src = format!(
        "{IO_AT}\nfn main() int {{\n  let h = out_stdout()\n  return out_text(h, \"{big}\")\n}}\n"
    );
    // The same literal lands in the expected stream (offset by the head
    // marker is not used; expected == the exact literal bytes).
    expect_bytes = big.as_bytes().to_vec();
    let (code, out, err) = build_and_run(&src);
    assert_eq!(code, Some(total as i32));
    assert_eq!(out, expect_bytes.as_slice());
    assert!(err.is_empty());
}

/// OUT-01 AC/Windows: errors are controlled codes, never hangs — an
/// unauthorized write (no platform.io.write grant) returns -1 with no
/// output; the stdio handles stay usable for the program's exit path.
#[test]
/// OUT-01 AC/Windows negative control: WITHOUT the platform.io.write
/// grant the lane rejects at ADMISSION (the p947 host.unauthorized
/// contract — rejection precedes any run and any output). The AV
/// lane exercises the dispatch-time -1 variant (console_out_1041.rs).
#[test]
fn out01_ac_windows_unauthorized_write_rejects_at_admission() {
    let src = format!(
        "{IO_AT}
fn main() int {{
  let h = out_stdout()
  return out_text(h, \"x\")
}}
"
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    // Grant everything EXCEPT platform.io.write.
    let grants = vec![
        "platform.io.stdout".to_string(),
        "platform.io.stderr".to_string(),
        "platform.mem".to_string(),
    ];
    let err =
        console_authorize(&mir, &grants).expect_err("missing write grant must reject at admission");
    assert_eq!(err.code, "host.unauthorized", "got {err:?}");
}

/// OUT-05 preview on the AC leg: the PE import closure stays exactly the
/// allowed kernel32 set (now including GetConsoleMode/WriteConsoleW even
/// when the shim's console ops are not called — the shim is one object).
#[test]
fn out01_ac_windows_pe_import_closure_includes_console_api() {
    let src =
        format!("{IO_AT}\nfn main() int {{\n  return out_text(out_stdout(), \"audit\\n\")\n}}\n");
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("audit");
    let built = ace_native::console::console_native_build(&mir, "d_main", &dir).expect("build");
    let out = Command::new(&built.exe_path)
        .stdout(Stdio::piped())
        .output()
        .expect("run");
    assert_eq!(out.stdout, b"audit\n");
    let pe_bytes = std::fs::read(&built.exe_path).unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let report = ace_native::platform_io::audit_pe_imports(&pe_bytes).expect("audit");
    let mut symbols: Vec<&str> = report.symbols.iter().map(|(_, s)| s.as_str()).collect();
    symbols.sort_unstable();
    assert_eq!(
        symbols,
        vec![
            "ExitProcess",
            "GetConsoleMode",
            "GetLastError",
            "GetStdHandle",
            "GetSystemInfo",
            "VirtualAlloc",
            "VirtualFree",
            "WriteConsoleW",
            "WriteFile",
        ],
        "the closed kernel32 import set (no CRT/libc sneaks in)"
    );
}
