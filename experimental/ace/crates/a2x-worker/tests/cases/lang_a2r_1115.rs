//! PLAN-1115 T-02 V-01: the ordinary-language A2R TRUE chain — ordinary
//! Auto source (`auto.source.language.av/v1`) → checked HIR → semantic
//! view → unified language-channel lowering → fileset → REAL rustc →
//! real run → parent-side exact oracles.
//!
//! V-01 families (plan §6): 同源数值边界 / 含 NUL 文本 / 调用副作用序 /
//! 闭包与嵌套 place — exact values and traces, named traps for overflow
//! and out-of-domain arithmetic. Every oracle below is independently
//! hand-computed (the wrapper prints exactly one `value:<n>`/`trap:<Name>`
//! line; authorized marks land on stderr in execution order).

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{
    assemble_language_file_set, canonical_execution_config, lower_language_program,
};
use ace_core::source::compile_language_source;

const LANG_CAP: &str = "ac.source.language.av/v1";
const TRACE_CAP: &str = "hir.test.trace.v1";

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

fn tools_available() -> bool {
    Command::new(rustc())
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Emit + compile + run one language-channel case; returns (stdout, stderr).
fn run_lang(name: &str, src: &str, caps: &[&str]) -> (String, String) {
    let caps: Vec<String> = caps.iter().map(|c| c.to_string()).collect();
    let compiled = compile_language_source(src, "main.at", &caps).unwrap_or_else(|ds| {
        panic!(
            "frontend diagnostics for {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    let program =
        lower_language_program(&view, "main").unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let cfg = canonical_execution_config("debug", 16_777_216, None, false, false);
    let fs = assemble_language_file_set(
        &view,
        &program,
        &compiled.input_hash,
        "1115",
        &cfg,
        "main",
    );

    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/lang-1115")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();
    assert!(
        fs.manifest.contains("auto.a2x.rust.language.fileset/1"),
        "the language channel manifest names its own schema"
    );

    let exe = dir.join("a2x_lang_case.exe");
    let mut compiler = Command::new(rustc());
    compiler
        .args(["--edition", "2021"])
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let compile_out = auto_tool_execution::run_command_capture(
        compiler,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert!(
        compile_out.kind == auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}:\n{}",
        String::from_utf8_lossy(compile_out.stderr.as_ref().expect("compiler stderr"))
    );

    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit non-zero for {name}"
    );
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("stderr")).to_string(),
    )
}

fn assert_value(name: &str, out: &str, expected: i32) {
    assert_eq!(out, format!("value:{expected}\n"), "stdout for {name}");
}

fn assert_trap(name: &str, out: &str, trap: &str) {
    assert_eq!(out, format!("trap:{trap}\n"), "stdout for {name}");
}

/// V-01a — numeric bounds through the same source the AV lane runs
/// (1079 numbers canonical family): i64 exactness past the f64 integer
/// window, the i64::MIN edge by checked subtraction, narrowing `.to(int)`.
#[test]
fn v01_a_numeric_bounds_exact() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let src = "fn main() int {\n\
         let big = 9007199254740993i64\n\
         let prev = 9007199254740992i64\n\
         if big - prev != 1i64 { return 1 }\n\
         let min = -9223372036854775807i64 - 1i64\n\
         if min + 1i64 != -9223372036854775807i64 { return 5 }\n\
         let wide = 40i64\n\
         return (wide + 2i64).to(int)\n\
     }";
    let (out, _) = run_lang("v01-a-numeric-bounds", src, &[LANG_CAP]);
    assert_value("v01-a-numeric-bounds", &out, 42);
}

/// V-01b — named arithmetic traps: checked i64 overflow, i32 division and
/// remainder by zero (the five-trap contract's names).
#[test]
fn v01_b_numeric_named_traps() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let overflow = "fn main() int { let max = 9223372036854775807i64\n return (max + 1i64).to(int) }";
    let div0 = "fn main() int { var d = 0\n return 7 / d }";
    let rem0 = "fn main() int { var d = 0\n return 7 % d }";
    let (a, _) = run_lang("v01-b-overflow", overflow, &[LANG_CAP]);
    assert_trap("v01-b-overflow", &a, "Overflow");
    let (b, _) = run_lang("v01-b-div-zero", div0, &[LANG_CAP]);
    assert_trap("v01-b-div-zero", &b, "DivByZero");
    let (c, _) = run_lang("v01-b-rem-zero", rem0, &[LANG_CAP]);
    assert_trap("v01-b-rem-zero", &c, "RemByZero");
}

/// V-01c — embedded NUL is ordinary text (1079 canonical): `"A中😀\u{0}"`
/// is 4 scalars / 9 UTF-8 bytes; concat stays exact across the NUL.
#[test]
fn v01_c_nul_text_exact() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let src = "fn main() int {\n\
         let s = \"A中😀\\u{0}\".clone()\n\
         if s.len() != 4 { return 1 }\n\
         if s.len_bytes() != 9 { return 2 }\n\
         let t = s + \"b\"\n\
         if t.len() != 5 { return 3 }\n\
         return 42\n\
     }";
    let (out, _) = run_lang("v01-c-nul-text", src, &[LANG_CAP]);
    assert_value("v01-c-nul-text", &out, 42);
}

/// V-01d — call side-effect order (1080 canonical, 892 observation
/// protocol): the named argument expressions evaluate in SOURCE order
/// (mark_b's operand first) while binding is by name — the value channel
/// (23: a=mark_a()*2=2, b=mark_b()+1=3 — marks return 1/2 per the frozen
/// 892 contract) and the mark trace channel (stderr) are asserted
/// separately.
#[test]
fn v01_d_call_side_effect_order() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let src = "fn sub(a int, b int) int { return a * 10 + b }\n\
         fn main() int { return sub(b: mark_b() + 1, a: mark_a() * 2) }";
    let (out, err) = run_lang("v01-d-side-effect-order", src, &[LANG_CAP, TRACE_CAP]);
    assert_value("v01-d-side-effect-order", &out, 23);
    assert_eq!(err, "mark_b\nmark_a\n", "marks land on stderr in execution order");
}

/// V-01e — closures and nested places (1081+1082 canonical families): an
/// immutable capturing closure over a HOF-typed call, and the field×index
/// chained place with compound write-back on a typed record of List<int>.
#[test]
fn v01_e_closures_and_nested_places() {
    if !tools_available() {
        eprintln!("infra-blocked: rustc unavailable — skipped");
        return;
    }
    let src = "fn each_plus(f fn(int)int, xs List<int>) int {\n\
         var n = 0\n\
         for x in xs {\n\
             let y = f(x)\n\
             n = n + y\n\
         }\n\
         return n\n\
     }\n\
     type Row { xs List<int> }\n\
     fn main() int {\n\
         var r = Row([19, 20])\n\
         r.xs[1] += 3\n\
         let factor = 1\n\
         return each_plus((x int) => x * factor, r.xs)\n\
     }";
    let (out, _) = run_lang("v01-e-closures-places", src, &[LANG_CAP]);
    // r.xs[1] += 3 makes the list [19, 23]; each_plus sums x*1 = 42.
    // (Without the nested-place write the sum is 39 — the mutation is
    // load-bearing for the oracle.)
    assert_value("v01-e-closures-places", &out, 42);
}
