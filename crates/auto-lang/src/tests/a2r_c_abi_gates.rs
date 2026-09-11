//! Plan 610 ⑤ real-compile gates (599 #[ignore] paradigm — shell out to
//! cargo/MSVC on demand; run with
//! `cargo test -p auto-lang --lib --features test-trans a2r_cabi -- --ignored`).
//!
//! Two gates:
//! - `a2r_cabi_export_gate` (AC-02): MVP #[export] face → cargo cdylib →
//!   std-only consumer exe dlopens it via LoadLibraryA/GetProcAddress and
//!   asserts add(2,3)==5 + a cstr round-trip witness.
//! - `a2r_cabi_engine_face_gate` (AC-03): the 005 corpus (12-symbol engine
//!   face written in Auto) → cdylib against the real autoterm-core rlib →
//!   the 597 a2c driver is rebuilt against THIS dll (AUTOTERM_ENGINE_DLL
//!   override) and must run CFACE_OK / exit 0. The driver's C side declares
//!   exactly the 12 externs, so a successful MSVC link doubles as the
//!   symbol-presence check.
//!
//! Projects stage under `<repo>/target/plan610/` (gitignored, warm cargo
//! cache across runs). Prereqs: cargo + MSVC (vcvars64) on PATH defaults,
//! auto-term main checkout at D:/autostack/auto-term (597 layout).

use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

fn repo_target_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target")
}

fn transpile_at_src(name: &str, src: &str) -> String {
    let mut sink = crate::trans::rust::transpile_rust(name, src)
        .unwrap_or_else(|e| panic!("transpile {} failed: {}", name, e));
    let bytes = sink
        .done()
        .unwrap_or_else(|e| panic!("transpile {} done() failed: {}", name, e));
    String::from_utf8(bytes.to_vec()).expect("utf8 product")
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut f = std::fs::File::create(path).unwrap();
    f.write_all(content.as_bytes()).unwrap();
}

fn cargo_build(dir: &Path) {
    let out = Command::new("cargo")
        .arg("build")
        .current_dir(dir)
        .output()
        .expect("spawn cargo");
    if !out.status.success() {
        panic!(
            "cargo build failed in {}:\n{}",
            dir.display(),
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

fn run_capture(cmd: &mut Command) -> (String, Option<i32>) {
    let out = cmd.output().expect("spawn command");
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code(),
    )
}

/// AC-02: MVP #[export] cdylib + std-only dlopen consumer.
#[test]
#[ignore = "shells out to cargo; on-demand cdylib real-compile gate (Plan 610)"]
fn a2r_cabi_export_gate() {
    let stage = repo_target_dir().join("plan610/mvp_export");
    let lib_dir = stage.join("engine_add_auto");
    let bin_dir = stage.join("mvp_consumer");

    // 1. Transpile the MVP export face.
    let src = r##"#[export]
fn add(a int, b int) int {
    return a + b
}

#[export]
fn dup(s cstr) cstr {
    return f"${s}${s}"
}

fn main() {
    print(add(2, 3))
}
"##;
    let lib_rs = transpile_at_src("mvp_export", src);

    // 2. Cargo cdylib project (std-only, offline-safe).
    write_file(
        &lib_dir.join("Cargo.toml"),
        "[package]\nname = \"engine_add_auto\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[workspace]\n",
    );
    write_file(&lib_dir.join("src/lib.rs"), &lib_rs);
    cargo_build(&lib_dir);
    let dll = lib_dir.join("target/debug/engine_add_auto.dll");
    assert!(dll.exists(), "cdylib not produced: {}", dll.display());

    // 3. Std-only consumer: LoadLibraryA + GetProcAddress (kernel32 via the
    //    S-form extern block — zero external crates, registry-free).
    let consumer_src = r##"#[link(name = "kernel32")]
extern "system" {
    fn LoadLibraryA(name: *const u8) -> *mut std::ffi::c_void;
    fn GetProcAddress(h: *mut std::ffi::c_void, name: *const u8) -> *mut std::ffi::c_void;
}

type AddFn = unsafe extern "C" fn(i32, i32) -> i32;
type DupFn = unsafe extern "C" fn(*const std::os::raw::c_char) -> *mut std::os::raw::c_char;

fn main() {
    let h = unsafe { LoadLibraryA(b"engine_add_auto.dll\0".as_ptr()) };
    assert!(!h.is_null(), "LoadLibraryA failed");
    let add: AddFn = unsafe { std::mem::transmute(GetProcAddress(h, b"add\0".as_ptr())) };
    assert!(add as usize != 0, "GetProcAddress(add) failed");
    assert_eq!(unsafe { add(2, 3) }, 5, "int width bridge broke add");
    let dup: DupFn = unsafe { std::mem::transmute(GetProcAddress(h, b"dup\0".as_ptr())) };
    assert!(dup as usize != 0, "GetProcAddress(dup) failed");
    let arg = std::ffi::CString::new("ab").unwrap();
    let ret = unsafe { dup(arg.as_ptr()) };
    let text = unsafe { std::ffi::CStr::from_ptr(ret) }.to_string_lossy();
    assert_eq!(text, "abab", "cstr boundary broke dup");
    println!("cabi_export_ok");
}
"##;
    write_file(
        &bin_dir.join("Cargo.toml"),
        "[package]\nname = \"mvp_consumer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"mvp_consumer\"\npath = \"src/main.rs\"\n\n[workspace]\n",
    );
    write_file(&bin_dir.join("src/main.rs"), consumer_src);
    cargo_build(&bin_dir);
    let exe = bin_dir.join("target/debug/mvp_consumer.exe");
    assert!(exe.exists(), "consumer exe not produced");

    // 4. Stage the DLL next to the exe (loader same-dir search) and run.
    let run_dir = stage.join("run");
    std::fs::create_dir_all(&run_dir).unwrap();
    std::fs::copy(&dll, run_dir.join("engine_add_auto.dll")).unwrap();
    std::fs::copy(&exe, run_dir.join("mvp_consumer.exe")).unwrap();
    let (stdout, code) = run_capture(Command::new(run_dir.join("mvp_consumer.exe")).current_dir(&run_dir));
    assert_eq!(code, Some(0), "consumer exit code, stdout: {}", stdout);
    assert!(stdout.contains("cabi_export_ok"), "witness missing: {}", stdout);
}

/// AC-03: 005 corpus → cdylib (real autoterm-core) → 597 a2c driver relinked
/// against it must run CFACE_OK / exit 0.
#[test]
#[ignore = "shells out to cargo + MSVC; on-demand engine-face closed-loop gate (Plan 610)"]
fn a2r_cabi_engine_face_gate() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus_at = manifest.join("test/a2r/27_c_abi/005_engine_face_auto/engine_face_auto.at");
    let src = std::fs::read_to_string(&corpus_at)
        .unwrap_or_else(|e| panic!("read corpus failed: {}", e));
    let lib_rs = transpile_at_src("engine_face_auto", &src);

    // 1. cdylib project against the real engine (597 layout prerequisite).
    let stage = repo_target_dir().join("plan610/engine_face_auto");
    write_file(
        &stage.join("Cargo.toml"),
        "[package]\nname = \"engine_face_auto\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nautoterm-core = { path = \"D:/autostack/auto-term/crates/autoterm-core\" }\n\n[workspace]\n",
    );
    write_file(&stage.join("src/lib.rs"), &lib_rs);
    cargo_build(&stage);
    let dll = stage.join("target/debug/engine_face_auto.dll");
    assert!(dll.exists(), "engine-face cdylib not produced: {}", dll.display());
    // Windows backslashes: the 597 script's copy /y chokes on fwd-slash paths.
    let dll_win = dll.to_string_lossy().replace('/', "\\");

    // 2. Rebuild the 597 a2c driver against THIS dll and run it.
    let script = manifest.join("../../scripts/build-engine-face-a2c.cmd");
    let out = Command::new("cmd")
        .args(["/c", &script.to_string_lossy()])
        .env("AUTOTERM_ENGINE_DLL", &dll_win)
        .output()
        .expect("spawn build-engine-face-a2c.cmd");
    assert!(
        out.status.success(),
        "driver build failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let driver_exe = repo_target_dir().join("engine-face-a2c/engine-face-a2c.exe");
    assert!(driver_exe.exists(), "driver exe missing");
    let (stdout, code) = run_capture(Command::new(&driver_exe).current_dir(driver_exe.parent().unwrap()));
    assert_eq!(code, Some(0), "driver exit code, stdout: {}", stdout);
    assert!(stdout.contains("CFACE_OK"), "CFACE_OK witness missing: {}", stdout);
}
