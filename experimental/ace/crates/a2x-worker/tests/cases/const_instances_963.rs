//! PLAN-963 T-04 / V-G04（A2R + A2C 端）：值参数（`<const N>`）实例的具体
//! Checked HIR 由 A2X 直接消费（Rust/C provider → 真实 rustc / MSVC cl 编译
//! → 真实运行）。目标端不做模板替换，只见具体函数（rep_u32_2 / rep_u32_3）。
//!
//! oracle（闭式手算）：`rep<const N>(x) = N * x` → `rep<2>(21)`=42、
//! `rep<3>(21)`=63、两者相加=105。

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_file_set, lower_program};
use ace_a2x_store::A2xSession;
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{compile_scalar_product, CompileLane, SourceQuerySession};
use std::path::PathBuf;
use std::process::Command;

const REP_TWO: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) }
";

const REP_THREE: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<3>(21) }
";

const REP_BOTH: &str = "\
fn rep<const N>(x int) int { return N * x }
fn main() int { return rep<2>(21) + rep<3>(21) }
";

fn caps() -> Vec<String> {
    vec![INSTANCES_CAPABILITY.to_string()]
}

fn compile_instances(src: &str, want_instances: usize) -> ace_source_query::ProductCompile {
    let mut session = SourceQuerySession::new("plan963", "constrep.at");
    let p = compile_scalar_product(
        src,
        "constrep.at",
        &caps(),
        "plan963",
        "constrep.at",
        &mut session,
    )
    .expect("instances-lane compile");
    assert_eq!(p.lane, CompileLane::Instances);
    assert_eq!(p.instances.len(), want_instances, "value instances");
    p
}

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-963")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Rust 端（真实 rustc 编译 + 运行）。
fn run_rust(src: &str, want: i32, tag: &str, syms: &[&str], tpl_residue: &str) {
    let p = compile_instances(src, if syms.len() > 1 { 2 } else { 1 });
    let view = SemanticView::from_checked(&p.compiled.checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &p.compiled.input_hash, "main");
    let joined = fs
        .files
        .iter()
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>()
        .join("\n");
    for sym in syms {
        assert!(
            joined.contains(sym),
            "Rust file set misses the `{sym}` instance"
        );
    }
    assert!(
        !joined.contains(tpl_residue),
        "no template residue in the Rust file set"
    );

    let dir = out_root().join(format!("rust-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("const963_rust.exe");
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
fn run_c(src: &str, want: i32, tag: &str, syms: &[&str], tpl_residue: &str) {
    let root = out_root().join(format!("store-c-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let mut s = A2xSession::open(&root, "a2c-963", "probe", 128 << 20, CTargetProvider::v1())
        .expect("open C session");
    let asm = s
        .cold_assemble(src, true, &caps(), "main", false)
        .expect("C assembly through the instances lane");
    let joined = asm
        .files
        .iter()
        .map(|(_, c)| c.clone())
        .collect::<Vec<_>>()
        .join("\n");
    for sym in syms {
        assert!(
            joined.contains(sym),
            "C file set misses the `{sym}` instance"
        );
    }
    assert!(
        !joined.contains(tpl_residue),
        "no template residue in the C file set"
    );

    let dir = out_root().join(format!("c-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &asm.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let (stdout, _stderr) =
        ace_a2x_worker::crun::compile_and_run_c(&dir, "const963_c.exe").expect("C compile/run");
    assert_eq!(stdout, format!("value:{want}\n"), "c leg oracle");
    let _ = std::fs::remove_dir_all(&root);
}

/// V-G04：`rep<2>(21)` → value:42（双端）。
#[test]
fn a2x_const_value_two_value_42() {
    let syms = ["rep_u32_2"];
    run_rust(REP_TWO, 42, "n2", &syms, "rep<const");
    run_c(REP_TWO, 42, "n2", &syms, "rep<const");
}

/// V-G04：`rep<3>(21)` → value:63（不同值 = 不同具体函数）。
#[test]
fn a2x_const_value_three_value_63() {
    let syms = ["rep_u32_3"];
    run_rust(REP_THREE, 63, "n3", &syms, "rep<const");
    run_c(REP_THREE, 63, "n3", &syms, "rep<const");
}

/// V-G04：两个值实例同表达式 → value:105（身份不串体）。
#[test]
fn a2x_two_values_value_105() {
    let syms = ["rep_u32_2", "rep_u32_3"];
    run_rust(REP_BOTH, 105, "both", &syms, "rep<const");
    run_c(REP_BOTH, 105, "both", &syms, "rep<const");
}
