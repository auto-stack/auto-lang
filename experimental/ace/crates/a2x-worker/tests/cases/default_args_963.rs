//! PLAN-963（T-08，L-24，A2R + A2C 端）：声明位默认实参在绑定层补位后成为
//! 普通实参，A2X 只见补满的调用（无默认值专用构造），由真实 rustc / MSVC cl
//! 编译并真实运行。
//!
//! oracle（手算闭式）：`rep(14)` = 42；`rep(14)`+`rep(14,2)` = 70；
//! 命名式补洞 `box(b: 2)` = 40+2 = 42。

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_file_set, lower_program};
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

const OMITTED_DEFAULT: &str = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    return rep(14)
}
";

const EXPLICIT_OVERRIDE: &str = "\
fn rep(x int, n int = 3) int { return x * n }
fn main() int {
    let a int = rep(14)
    let b int = rep(14, 2)
    return a + b
}
";

const NAMED_HOLE: &str = "\
fn box(a int = 40, b int = 2) int { return a + b }
fn main() int {
    return box(b: 2)
}
";

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-963")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Rust 端（真实 rustc 编译 + 运行）。
fn run_rust(src: &str, want: i32, tag: &str) {
    let compiled = ace_core::source::compile_record_source(src, "d963.at", &[])
        .expect("default-argument source compiles");
    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(format!("rust-d963-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("d963_rust.exe");
    let out = Command::new(rustc())
        .arg("--edition")
        .arg("2021")
        .arg("-O")
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir)
        .output()
        .expect("spawn rustc");
    assert!(
        out.status.success(),
        "rustc failed:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let run = Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run exe");
    assert!(
        run.status.success(),
        "program failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&run.stdout).to_string(),
        format!("value:{want}\n"),
        "rust leg oracle"
    );
}

/// C 端（A2xSession + C provider → 真实 MSVC cl + 运行）。
fn run_c(src: &str, want: i32, tag: &str) {
    let root = out_root().join(format!("store-c-d963-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut s = A2xSession::open(&root, "a2c-963", "probe", 128 << 20, CTargetProvider::v1())
        .expect("open C session");
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Record, &[], "main", false)
        .expect("C assembly");

    let dir = out_root().join(format!("c-d963-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let (stdout, _stderr) =
        ace_a2x_worker::crun::compile_and_run_c(&dir, "d963_c.exe").expect("C compile/run");
    assert_eq!(stdout, format!("value:{want}\n"), "c leg oracle");
    let _ = std::fs::remove_dir_all(&root);
}

/// O-963-D13（A2R + A2C）：省略末尾默认实参 → value:42。
#[test]
fn a2x_default_argument_omitted_value_42() {
    run_rust(OMITTED_DEFAULT, 42, "d13");
    run_c(OMITTED_DEFAULT, 42, "d13");
}

/// O-963-D14（A2R + A2C）：显式实参覆盖默认值，两调用相加 → value:70。
#[test]
fn a2x_default_argument_explicit_override_value_70() {
    run_rust(EXPLICIT_OVERRIDE, 70, "d14");
    run_c(EXPLICIT_OVERRIDE, 70, "d14");
}

/// O-963-D15（A2R + A2C）：命名式补洞 → value:42。
#[test]
fn a2x_default_argument_named_hole_value_42() {
    run_rust(NAMED_HOLE, 42, "d15");
    run_c(NAMED_HOLE, 42, "d15");
}
