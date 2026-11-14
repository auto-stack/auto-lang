//! PLAN-964（T-03，A2R/A2C 端）：FStr 插值（插值表达式 + i32 显示）经
//! TextConcat 链与 TextDisplay 由 A2X 直接消费——SemanticView → Rust/C
//! provider → 真实 rustc / MSVC cl 编译 → 真实运行 → "value:N"。对象/闭包
//! 面不在 A2X 域（"按原支持域"；A2X 的 closure 一词指 ABI 记录闭包），FStr
//! 是 A2X 腿的真实载体。oracle 手算闭式 value:42。

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_a2x_store::A2xSession;
use ace_core::source::SourceProfile;
use std::path::PathBuf;
use std::process::Command;

const FSTRING: &str = "\
fn main() int {
    let name = \"AutoLang\"
    let age = 1
    let greeting = f\"Hello, $name\"
    let info = f\"Name: $name, Age: $age\"
    let x = 10
    let y = 20
    let result = f\"Result: ${x + y}\"
    let msg = f\"The value is $x and ${y * 2}\"
    let a = 5
    let b = 3
    let sum = f\"Sum: ${a + b}, Product: ${a * b}\"
    if greeting != \"Hello, AutoLang\" { return 0 }
    if info != \"Name: AutoLang, Age: 1\" { return 0 }
    if result != \"Result: 30\" { return 0 }
    if msg != \"The value is 10 and 40\" { return 0 }
    if sum != \"Sum: 8, Product: 15\" { return 0 }
    return 42
}
";

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-964")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Rust 腿（真实 rustc 编译 + 运行）：stdout 精确等于 value:42。
fn run_rust(src: &str, want: i32, tag: &str) {
    let compiled = ace_core::source::compile_aggregates_source(src, "p964.at", &[])
        .expect("aggregates source compiles");
    let checked = compiled.checked;
    let view = SemanticView::from_checked(&checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let joined = fs
        .files
        .iter()
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        joined.contains("a2x_str_concat") && joined.contains("A2xStr"),
        "the FStr leg needs the owned-Str runtime"
    );
    let files: Vec<(String, String)> = fs.files.clone();

    let dir = out_root().join(format!("rust-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("p964_rust.exe");
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
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    assert_eq!(stdout, format!("value:{want}\n"), "rust leg oracle");
}

/// C 腿（MSVC cl 编译 + 运行）：stdout 精确等于 value:42。
fn run_c(src: &str, want: i32, tag: &str) {
    let root = out_root().join("a2c-964");
    let _ = std::fs::remove_dir_all(&root);
    let mut s = A2xSession::open(
        &root,
        "a2c-964",
        "probe",
        128 << 20,
        CTargetProvider::v1(),
    )
    .expect("a2c session");
    let asm = s
        .cold_assemble_profile(src, SourceProfile::Aggregates, &[], "main", false)
        .expect("a2c cold assemble");
    let dir = out_root().join(format!("c-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in asm.files.iter() {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("p964_c.exe");
    ace_a2x_worker::crun::compile_and_run_c(&dir, "p964_c.exe")
        .expect("compile and run the C leg");
    let run = Command::new(&exe)
        .current_dir(&dir)
        .output()
        .expect("run c exe");
    assert!(
        run.status.success(),
        "c program failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout).to_string();
    assert_eq!(stdout, format!("value:{want}\n"), "c leg oracle");
}

/// FStr 全语料（插值表达式 + i32 显示）→ A2R 与 A2C 双端 value:42。
#[test]
fn a2x_fstring_value_42() {
    run_rust(FSTRING, 42, "fstr");
    run_c(FSTRING, 42, "fstr");
}
