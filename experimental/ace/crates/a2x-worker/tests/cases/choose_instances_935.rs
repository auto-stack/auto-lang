//! PLAN-935 T-02（A2R/A2C 端）：choose 双实例经实例 lane 的具体 Checked
//! HIR 由 **A2X 直接消费**（SemanticView → Rust/C provider → 真实 rustc /
//! MSVC cl 编译 → 真实运行 → "value:N"）。目标后端不做自己的模板替换——
//! 它们只看到具体函数（choose_i32/choose_bool）。
//!
//! oracle：原模板 value:42；交换模板分支 value:0（双端各自独立断言）。

use ace_a2x_c::CTargetProvider;
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_file_set, lower_program};
use ace_a2x_store::A2xSession;
use ace_core::source::cst::INSTANCES_CAPABILITY;
use ace_source_query::{compile_scalar_product, CompileLane, SourceQuerySession};
use std::path::PathBuf;
use std::process::Command;

const CHOOSE: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return a }
    return b
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

const CHOOSE_SWAPPED: &str = "\
fn choose<T>(a T, b T, flag bool) T {
    if flag { return b }
    return a
}
fn main() int {
    if choose(true, false, true) { return choose(40, 42, false) }
    return 0
}
";

fn caps() -> Vec<String> {
    vec![INSTANCES_CAPABILITY.to_string()]
}

fn compile_instances(src: &str) -> ace_source_query::ProductCompile {
    let mut session = SourceQuerySession::new("plan935", "choose.at");
    let p = compile_scalar_product(
        src,
        "choose.at",
        &caps(),
        "plan935",
        "choose.at",
        &mut session,
    )
    .expect("instances-lane compile");
    assert_eq!(p.lane, CompileLane::Instances);
    p
}

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-935")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Rust 端（真实 rustc 编译 + 运行）。
fn run_rust(src: &str, want: i32, tag: &str, syms: &[&str], tpl_residue: &str) {
    let p = compile_instances(src);
    let view = SemanticView::from_checked(&p.compiled.checked).expect("view admission");
    let program = lower_program(&view, "main").expect("lowering");
    let fs = assemble_file_set(&view, &program, &p.compiled.input_hash, "main");
    // 目标端只见具体函数：生成面含需求实例符号，无模板符号。
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
    let exe = dir.join("choose935_rust.exe");
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
    let mut s = A2xSession::open(&root, "a2c-935", "probe", 128 << 20, CTargetProvider::v1())
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
        ace_a2x_worker::crun::compile_and_run_c(&dir, "choose935_c.exe").expect("C compile/run");
    assert_eq!(stdout, format!("value:{want}\n"), "c leg oracle");
    let _ = std::fs::remove_dir_all(&root);
}

/// O-3（A2R + A2C 双端）：value:42。
#[test]
fn a2x_choose_original_value_42() {
    let syms = ["choose_i32", "choose_bool"];
    run_rust(CHOOSE, 42, "orig", &syms, "choose<T>");
    run_c(CHOOSE, 42, "orig", &syms, "choose<T>");
}

/// O-4（A2R + A2C 双端）：交换模板分支 → value:0。
#[test]
fn a2x_choose_swapped_value_0() {
    let syms = ["choose_i32", "choose_bool"];
    run_rust(CHOOSE_SWAPPED, 0, "swap", &syms, "choose<T>");
    run_c(CHOOSE_SWAPPED, 0, "swap", &syms, "choose<T>");
}

/// PLAN-963 T-02（A2R + A2C 双端）：多类型参数两排列实例 → value:42；
/// 生成面只有具体实例符号，无模板残留。
const PICK963: &str = "\
fn pick<T, U>(a T, b U, flag bool) T {
    if flag { return a }
    return a
}
fn main() int {
    if pick(true, 7, false) { return pick(42, true, false) }
    return 0
}
";

#[test]
fn a2x_multi_param_instances_value_42() {
    let syms = ["pick_i32_bool", "pick_bool_i32"];
    run_rust(PICK963, 42, "multi963", &syms, "pick<T");
    run_c(PICK963, 42, "multi963", &syms, "pick<T");
}
