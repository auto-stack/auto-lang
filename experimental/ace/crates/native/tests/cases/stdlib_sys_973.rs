//! PLAN-973 AC leg: the sys973 platform-op shims ship as NAMED-UNSUPPORTED
//! stubs in this emitter (a REAL body replaces its stub in the op's AC
//! increment). The console lane already admits platform reserved names
//! (PLAN-1041), so a console-profile program calling a sys973 op compiles
//! to native, links against the shim COFF, and the stub must return the
//! frozen `ERR_UNSUPPORTED` (-10) — never a link hole, never a silent
//! fallback.
#![cfg(all(windows, target_arch = "x86_64"))]

use ace_native::console::{console_authorize, console_compile, console_native_run};

fn sys_grants() -> Vec<String> {
    vec![
        "platform.io.stdout".into(),
        "platform.io.stderr".into(),
        "platform.io.write".into(),
        "platform.mem".into(),
        "platform.file".into(),
        "platform.file.path".into(),
    ]
}

fn temp_workdir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mn-sys973-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Calling path_attrs_bytes (a sys973 stub) from native machine code:
/// the program prints the stub result; -10 must come back.
#[test]
fn sys973_native_stub_returns_unsupported() {
    let src = r#"
fn write_all(h int, s str) int {
    var off = 0
    var rem = s.len_bytes()
    for rem > 0 {
        let w = io_write_some_slice(h, s, off, rem)
        if w <= 0 {
            return w
        }
        off = off + w
        rem = rem - w
    }
    return off
}

fn main() int {
    let out = io_stdout()
    // sys973 stub: named-unsupported on the AC emitter v1
    let r = path_attrs_bytes("C:/")
    if (r == -10) {
        let w = write_all(out, "stub-unsupported-ok")
        if (w < 0) {
            return 2
        }
        return 0
    }
    return 100 + (0 - r)
}
"#;
    let mir = console_compile(src, "d_main").expect("console compile");
    console_authorize(&mir, &sys_grants()).expect("authorize");
    let run = console_native_run(&mir, "d_main", &temp_workdir("stub")).expect("native run");
    assert_eq!(
        run.exit_code,
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("stub-unsupported-ok"),
        "stdout={}",
        String::from_utf8_lossy(&run.stdout)
    );
}
