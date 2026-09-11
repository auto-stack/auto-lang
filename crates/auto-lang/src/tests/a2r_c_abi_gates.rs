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

/// Plan 610 ⑥: transpile with a source dir (relative use.c JSON manifests
/// resolve against it), matching the corpus test path.
fn transpile_at_src_dir(dir: &Path, name: &str, src: &str) -> String {
    let mut sink = crate::trans::rust::transpile_rust_with_source_dir(dir, name, src)
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


/// Plan 610 ⑥ driver source (AC-04/05/06): the 597 a2c engine-face driver
/// rewritten in Auto — spawn cmd, echo the anchor, poll feed/damage, scan
/// row text through the generated FFI face, witness CFACE_OK. Exit codes
/// mirror 597: 0 = anchor seen, 1 = spawn fail, 2 = anchor timeout. The
/// manifest file name is substituted per leg (S static vs D dynamic).
const USE_C_DRIVER_AT: &str = r##"// Plan 610 ⑥ driver (AC-04/05/06): the 597 a2c engine-face driver
// rewritten in Auto. Sleep/exit via std (portable). Exit codes mirror 597.
use.c <MANIFEST_NAME>
use.rs std::process::exit
use.rs std::thread::sleep
use.rs std::time::Duration

fn main() {
    let h = autoterm_engine_spawn(80, 24, "cmd")
    if cffi_handle_is_null(h) {
        print("SPAWN_FAIL")
        exit(1)
    }
    autoterm_engine_write_input(h, "echo CFACE_OK\r\n", 16)
    var tries = 0
    var found = false
    while tries < 100 && !found {
        if autoterm_engine_feed_ready(h) == 1 {
            let dirty = cffi_buf_new_u32(64)
            autoterm_engine_take_dirty_rows(h, dirty, 64)
            var r = 0
            while r < 24 {
                let buf = cffi_buf_new_u8(256)
                autoterm_engine_row_text(h, r, buf, 256)
                let text = cffi_cstr_read(buf)
                if text.contains("CFACE_OK") {
                    found = true
                }
                r = r + 1
            }
        }
        if !found {
            sleep(Duration.from_millis(50))
        }
        tries = tries + 1
    }
    autoterm_engine_kill(h)
    autoterm_engine_free(h)
    if found {
        print("CFACE_OK")
        exit(0)
    }
    print("ANCHOR_TIMEOUT")
    exit(2)
}
"##;

/// Plan 610 ⑥: build the ⑤ engine-face cdylib (the 005 corpus product
/// against the real autoterm-core) and return its dll path. Shared by the
/// engine-face gate (AC-03) and the use_c gate's AC-05 leg.
fn build_engine_face_cdylib() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let corpus_at = manifest.join("test/a2r/27_c_abi/005_engine_face_auto/engine_face_auto.at");
    let src = std::fs::read_to_string(&corpus_at)
        .unwrap_or_else(|e| panic!("read corpus failed: {}", e));
    let lib_rs = transpile_at_src("engine_face_auto", &src);

    let stage = repo_target_dir().join("plan610/engine_face_auto");
    write_file(
        &stage.join("Cargo.toml"),
        "[package]\nname = \"engine_face_auto\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\nautoterm-core = { path = \"D:/autostack/auto-term/crates/autoterm-core\" }\n\n[workspace]\n",
    );
    write_file(&stage.join("src/lib.rs"), &lib_rs);
    cargo_build(&stage);
    let dll = stage.join("target/debug/engine_face_auto.dll");
    assert!(
        dll.exists(),
        "engine-face cdylib not produced: {}",
        dll.display()
    );
    dll
}

/// AC-04/05/06: the ⑥ use.c driver, three link legs —
/// - AC-04: S form (static #[link]) against the REAL autoterm_core.dll.lib;
/// - AC-05: the SAME transpiled product relinked against the ⑤ product
///   (engine_face_auto.dll.lib staged as autoterm_core.lib — the import
///   lib's embedded DLL reference swaps the runtime engine). Auto driver ×
///   Auto engine face, zero hand-written glue at either end;
/// - AC-06: D form (libloading) loading the real DLL at runtime from the
///   exe dir (env AUTOTERM_CORE_DLL override available).
#[test]
#[ignore = "shells out to cargo + rustc; on-demand use.c closed-loop gate (Plan 610)"]
fn a2r_cabi_use_c_gate() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let stage = repo_target_dir().join("plan610/use_c_gate");
    let engine_dll_lib = PathBuf::from(
        "D:/autostack/auto-term/target/debug/autoterm_core.dll.lib",
    );
    let engine_dll = PathBuf::from("D:/autostack/auto-term/target/debug/autoterm_core.dll");
    assert!(
        engine_dll_lib.is_file() && engine_dll.is_file(),
        "engine prereq missing (cargo build -p autoterm-core in auto-term): {}/{}",
        engine_dll_lib.display(),
        engine_dll.display()
    );

    // --- Leg 1+2 share one transpiled S-form product. ---
    let s_dir = stage.join("s");
    std::fs::create_dir_all(s_dir.join("lib")).unwrap();
    std::fs::create_dir_all(stage.join("s2/lib")).unwrap();
    std::fs::copy(
        &manifest_dir.join("test/a2r/27_c_abi/003_use_c_static/engine_face_full.json"),
        stage.join("engine_face_full.json"),
    )
    .unwrap();
    let driver_s = USE_C_DRIVER_AT.replace("MANIFEST_NAME", "engine_face_full.json");
    write_file(&stage.join("driver_s.at"), &driver_s);
    let product = transpile_at_src_dir(&stage, "driver_s", &driver_s);
    write_file(&stage.join("driver_s.rs"), &product);

    // AC-04: link against the real engine (import lib renamed to the link
    // name the S form requests; the embedded autoterm_core.dll reference
    // makes the staged same-name DLL load at runtime).
    std::fs::copy(&engine_dll_lib, s_dir.join("lib/autoterm_core.lib")).unwrap();
    let link = |lib_dir: &Path, out: &str, product_path: &Path, exe_dir: &Path| {
        let out = Command::new("rustc")
            .args([
                "--edition=2021",
                "--crate-name",
                out,
                "-L",
                &format!("native={}", lib_dir.display()),
            ])
            .arg(product_path)
            .arg("-o")
            .arg(exe_dir.join(format!("{}.exe", out)))
            .output()
            .expect("spawn rustc");
        assert!(
            out.status.success(),
            "rustc link failed:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    link(&s_dir.join("lib"), "driver_s", &stage.join("driver_s.rs"), &s_dir);
    std::fs::copy(&engine_dll, s_dir.join("autoterm_core.dll")).unwrap();
    let (stdout, code) = run_capture(
        Command::new(s_dir.join("driver_s.exe")).current_dir(&s_dir),
    );
    assert_eq!(code, Some(0), "AC-04 leg exit, stdout: {}", stdout);
    assert!(stdout.contains("CFACE_OK"), "AC-04 witness missing: {}", stdout);

    // AC-05: SAME product, relinked against the ⑤ product (Auto-written
    // engine face cdylib from the 005 corpus).
    let auto5_dll = build_engine_face_cdylib();
    let s2 = stage.join("s2");
    std::fs::copy(
        auto5_dll.with_extension("dll.lib"),
        s2.join("lib/autoterm_core.lib"),
    )
    .unwrap();
    link(
        &s2.join("lib"),
        "driver_a5",
        &stage.join("driver_s.rs"),
        &s2,
    );
    std::fs::copy(&auto5_dll, s2.join(auto5_dll.file_name().unwrap())).unwrap();
    let (stdout, code) = run_capture(
        Command::new(s2.join("driver_a5.exe")).current_dir(&s2),
    );
    assert_eq!(code, Some(0), "AC-05 closed-loop exit, stdout: {}", stdout);
    assert!(
        stdout.contains("CFACE_OK"),
        "AC-05 closed-loop witness missing: {}",
        stdout
    );

    // --- AC-06: D form (libloading runtime resolution, exe same-dir). ---
    let dyn_dir = stage.join("dyn");
    std::fs::create_dir_all(dyn_dir.join("src")).unwrap();
    std::fs::copy(
        &manifest_dir.join("test/a2r/27_c_abi/004_use_c_dynamic/engine_face_full_dyn.json"),
        dyn_dir.join("engine_face_full_dyn.json"),
    )
    .unwrap();
    let driver_d = USE_C_DRIVER_AT.replace("MANIFEST_NAME", "engine_face_full_dyn.json");
    let product_d = transpile_at_src_dir(&dyn_dir, "driver_d", &driver_d);
    write_file(
        &dyn_dir.join("Cargo.toml"),
        "[package]\nname = \"driver_dyn\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[bin]]\nname = \"driver_dyn\"\npath = \"src/main.rs\"\n\n[dependencies]\nlibloading = \"0.8\"\n\n[workspace]\n",
    );
    write_file(&dyn_dir.join("src/main.rs"), &product_d);
    cargo_build(&dyn_dir);
    let exe = dyn_dir.join("target/debug/driver_dyn.exe");
    assert!(exe.exists(), "D-form exe not produced");
    std::fs::copy(&exe, dyn_dir.join("driver_dyn.exe")).unwrap();
    std::fs::copy(&engine_dll, dyn_dir.join("autoterm_core.dll")).unwrap();
    let (stdout, code) = run_capture(
        Command::new(dyn_dir.join("driver_dyn.exe")).current_dir(&dyn_dir),
    );
    assert_eq!(code, Some(0), "AC-06 D-form exit, stdout: {}", stdout);
    assert!(stdout.contains("CFACE_OK"), "AC-06 witness missing: {}", stdout);
}
