//! PLAN-963（T-02，A2R/A2C 端）：用户泛型记录在投影期单态化为具体 nominal
//! 记录后，由 **A2X 直接消费**（SemanticView → Rust/C provider → 真实
//! rustc / MSVC cl 编译 → 真实运行 → "value:N"）。目标后端不做模板替换，
//! 只见合成记录 `Point_int` 及其方法。
//!
//! oracle：`Point.both(150)` → value:300；换模板体（调用点不动）→ value:301。

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_file_set, lower_program};
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

const POINT963: &str = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    return p.first() + p.y
}
";

const POINT963_SHIFTED: &str = "\
type Point<T> { x T; y T
    static fn both(v T) Point<T> { return Point { x: v, y: v + 1 } }
    fn first() T { return .x }
}
fn main() int {
    let p Point<int> = Point.both(150)
    return p.first() + p.y
}
";

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-963")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Rust 端（真实 rustc 编译 + 运行）；生成面含合成记录，无模板残留。
fn run_rust(src: &str, want: i32, tag: &str) {
    let compiled = ace_core::source::compile_record_source(src, "p963.at", &[])
        .expect("generic record source compiles");
    let checked = compiled.checked;
    let view = SemanticView::from_checked(&checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &compiled.input_hash, "main");
    let joined = fs
        .files
        .iter()
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("Point_int"),
        "Rust file set misses the synthesized record"
    );
    assert!(
        !joined.contains("Point<T>"),
        "no template residue in the Rust file set"
    );

    let dir = out_root().join(format!("rust-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("p963_rust.exe");
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
    let root = out_root().join(format!("store-c-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut s = A2xSession::open(&root, "a2c-963", "probe", 128 << 20, CTargetProvider::v1())
        .expect("open C session");
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Record, &[], "main", false)
        .expect("C assembly of the synthesized record");
    let joined = asm
        .files
        .iter()
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("Point_int"),
        "C file set misses the synthesized record"
    );
    assert!(
        !joined.contains("Point<T>"),
        "no template residue in the C file set"
    );

    let dir = out_root().join(format!("c-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let (stdout, _stderr) =
        ace_a2x_worker::crun::compile_and_run_c(&dir, "p963_c.exe").expect("C compile/run");
    assert_eq!(stdout, format!("value:{want}\n"), "c leg oracle");
    let _ = std::fs::remove_dir_all(&root);
}

/// P1（A2R + A2C 双端）：value:300。
#[test]
fn a2x_generic_record_value_300() {
    run_rust(POINT963, 300, "p1");
    run_c(POINT963, 300, "p1");
}

/// 对照（A2R + A2C 双端）：模板体改动（调用点不动）→ value:301。
#[test]
fn a2x_generic_record_body_change_value_301() {
    run_rust(POINT963_SHIFTED, 301, "p1s");
    run_c(POINT963_SHIFTED, 301, "p1s");
}
