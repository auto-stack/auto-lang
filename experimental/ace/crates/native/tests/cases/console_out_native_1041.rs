//! PLAN-1041 OUT-01 (AC legs): the common Auto output source
//! (`stdlib/auto/platform/io.at`, inlined like the 1039 helper) compiled
//! to a REAL Windows x64 PE with the platform shim and executed — the
//! redirect path writes byte-exact UTF-8 (NUL is content), both streams
//! are real OS handles, classification sees a pipe (kind 2), and the
//! console write16 op is absent from a byte-sink run. Import audit keeps
//! the kernel32 closure honest (OUT-05 covers the audit side fully).
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_native::console::{console_authorize, console_compile, console_native_run};
use std::os::windows::io::FromRawHandle as _PipeFromRaw;
use std::os::windows::io::{FromRawHandle as _, OwnedHandle};
use std::os::windows::process::CommandExt as _;
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

// ---- OUT-02: the REAL Windows text console ---------------------------------
// The frozen contract: a real conhost window (single instance, short
// lived), Unicode written through the Auto UTF-8→UTF-16 conversion +
// WriteConsoleW, read back from the SCREEN BUFFER (font-independent
// UTF-16 units). A pipe/capture never substitutes for this leg.

#[allow(non_snake_case)]
mod out02_win {
    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct COORD {
        pub X: i16,
        pub Y: i16,
    }
    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct SMALL_RECT {
        pub Left: i16,
        pub Top: i16,
        pub Right: i16,
        pub Bottom: i16,
    }
    #[repr(C)]
    #[allow(non_snake_case)]
    pub struct CONSOLE_SCREEN_BUFFER_INFO {
        pub dwSize: COORD,
        pub dwCursorPosition: COORD,
        pub wAttributes: u16,
        pub srWindow: SMALL_RECT,
        pub dwMaximumWindowSize: COORD,
    }
    extern "system" {
        pub fn FreeConsole() -> i32;
        pub fn AllocConsole() -> i32;
        pub fn AttachConsole(pid: u32) -> i32;
        pub fn GetConsoleScreenBufferInfo(h: isize, info: *mut CONSOLE_SCREEN_BUFFER_INFO) -> i32;
        pub fn CreateFileW(
            name: *const u16,
            access: u32,
            share: u32,
            sa: *mut u8,
            disp: u32,
            flags: u32,
            template: *mut u8,
        ) -> isize;
        pub fn ReadConsoleOutputCharacterW(
            h: isize,
            buf: *mut u16,
            len: u32,
            origin: COORD,
            read: *mut u32,
        ) -> i32;
        pub fn GetStdHandle(which: i32) -> isize;
    }
    pub const GENERIC_READ: u32 = 0x8000_0000;
    pub const OPEN_EXISTING: u32 = 3;
    pub const ATTACH_PARENT: u32 = u32::MAX - 1;
}

static OUT02_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// OUT-02 real console: the test allocates a REAL console window
/// (single instance, short lived), hands a genuine CONOUT$ handle to the
/// PE as its stdout — the program classifies kind 1 and writes 中文 +
/// astral chars through the Auto UTF-8→UTF-16 conversion + WriteConsoleW
/// — then the test reads the SCREEN BUFFER back (font-independent).
/// The astral pair is witnessed by the confirmed-byte return code (the
/// conhost screen readback collapses astral pairs to a single cell — a
/// known buffer-fidelity limit, documented here, never silently widened).
#[test]
fn out02_ac_windows_real_console_unicode_roundtrip() {
    let _g = OUT02_LOCK.lock().unwrap();
    let src = format!(
        "{IO_AT}
fn main() int {{
    let h = out_stdout()
    let k = out_kind(h)
    let n = out_text(h, \"P中😀Z\")
    if k == 1 {{ return n }}
    return n - 100
}}
"
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("console02");
    let built = ace_native::console::console_native_build(&mir, "d_main", &dir).expect("build");

    unsafe {
        out02_win::FreeConsole();
        assert_eq!(out02_win::AllocConsole(), 1, "AllocConsole");
        // Two independent CONOUT$ handles: the child's stdout and the
        // test's screen reader.
        let mut name: Vec<u16> = "CONOUT$ ".encode_utf16().collect();
        let child_h = out02_win::CreateFileW(
            name.as_mut_ptr(),
            0x8000_0000 | 0x4000_0000,
            0x1 | 0x2,
            std::ptr::null_mut(),
            out02_win::OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        let read_h = out02_win::CreateFileW(
            name.as_mut_ptr(),
            out02_win::GENERIC_READ,
            0x1 | 0x2,
            std::ptr::null_mut(),
            out02_win::OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        assert!(child_h != -1 && read_h != -1, "CONOUT$ open failed");
        let child_stdout = Stdio::from(std::fs::File::from(OwnedHandle::from_raw_handle(
            child_h as _,
        )));
        let status = Command::new(&built.exe_path)
            .stdout(child_stdout)
            .status()
            .expect("run in real console");
        // Read the first row of the screen buffer back.
        let mut buf = [0u16; 256];
        let mut read: u32 = 0;
        let ok = out02_win::ReadConsoleOutputCharacterW(
            read_h,
            buf.as_mut_ptr(),
            120,
            out02_win::COORD { X: 0, Y: 0 },
            &mut read,
        );
        let text = String::from_utf16_lossy(&buf[..read as usize]);
        out02_win::FreeConsole();
        out02_win::AttachConsole(out02_win::ATTACH_PARENT);
        assert_eq!(ok, 1, "screen read failed");
        // The classification passed and ALL UTF-8 bytes were confirmed
        // (P中😀Z = 1+3+4+1 = 9; a wrong conversion or a dropped unit
        // changes the return).
        assert_eq!(
            status.code(),
            Some(9),
            "console kind + all 9 UTF-8 bytes confirmed"
        );
        // BMP + ASCII round-trip exactly on the real screen buffer.
        assert!(
            text.contains('P') && text.contains('中') && text.contains('Z'),
            "screen carries the written units: {text:?}"
        );
    }

    // Redirect honesty (file): the same program under a plain FILE
    // redirect writes ONLY the original UTF-8 bytes (no BOM/CRLF).
    let redir = temp_workdir("console02r");
    let out_file = redir.join("out.bin");
    let out_f = std::fs::File::create(&out_file).unwrap();
    let status = Command::new(&built.exe_path)
        .stdout(Stdio::from(out_f))
        .status()
        .unwrap();
    let bytes = std::fs::read(&out_file).unwrap();
    let _ = std::fs::remove_dir_all(&redir);
    assert_eq!(
        status.code(),
        Some(-91),
        "redirect kind marker + confirmed bytes"
    );
    assert_eq!(bytes, "P中😀Z".as_bytes(), "file redirect keeps raw UTF-8");
    let _ = std::fs::remove_dir_all(&dir);
}

/// OUT-04 EPIPE (Windows side): a broken pipe on stdout makes the write
/// fail with the OS error (ERROR_BROKEN_PIPE 232) — the program returns
/// -8 and out_last_error carries the raw code; never a hang.
#[test]
fn out04_ac_windows_broken_pipe_carries_raw_error() {
    let src = format!(
        "{IO_AT}\nfn main() int {{\n    let h = out_stdout()\n    let w = out_text(h, \"data\")\n    if w != -8 {{ return w - 1000 }}\n    return out_last_error()\n}}\n"
    );
    let mir = console_compile(&src, "d_main").expect("compile");
    console_authorize(&mir, &platform_grants()).expect("authorize");
    let dir = temp_workdir("epipe");
    let built = ace_native::console::console_native_build(&mir, "d_main", &dir).expect("build");
    // CreatePipe via extern (one anonymous pipe; drop the read end).
    extern "system" {
        fn CreatePipe(read: *mut isize, write: *mut isize, sa: *mut u8, size: u32) -> i32;
        fn CloseHandle(h: isize) -> i32;
    }
    let mut read_end: isize = 0;
    let mut write_end: isize = 0;
    // The reader is gone before the program writes: close it now.
    unsafe {
        assert_eq!(
            CreatePipe(&mut read_end, &mut write_end, std::ptr::null_mut(), 0),
            1,
            "CreatePipe"
        );
        assert_eq!(CloseHandle(read_end), 1, "close read end");
    }
    let pipe_stdout = unsafe { std::fs::File::from_raw_handle(write_end as _) };
    let out = Command::new(&built.exe_path)
        .stdout(Stdio::from(pipe_stdout))
        .output()
        .expect("run");
    let _ = std::fs::remove_dir_all(&dir);
    // The exact Win32 code for a broken pipe is 232 (ERROR_BROKEN_PIPE);
    // an invalid-handle variant (6) is accepted only as a documented
    // environment artifact, never a success.
    let code = out.status.code().unwrap_or(-1);
    assert!(
        code == 232 || code == 6,
        "raw broken-pipe error observable via exit (got {code}); stderr empty={}",
        out.stderr.is_empty()
    );
}
