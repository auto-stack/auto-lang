//! PLAN-902 T-02 record/text TRUE-CHAIN conformance (a2x-worker private):
//! the four FROZEN required sources (R07/R08/T03/T05, byte-exact ace-m1
//! originals) + the cleanup counterexample + fault injection + the
//! record-method single-body credential — source → checked HIR (with the
//! verifier's resource plan) → widened semantic view → Rust provider →
//! REAL rustc → real run → parent-side oracle.
//!
//! Oracles are the frozen hand-audited values (42/42/23/42); the resource
//! event sequences pin the verifier-plan cleanup order (creation-order
//! replacement, declaration-order LIFO exits) and the default-off fault
//! hook turns the n-th allocation into the typed `Resource` trap with NO
//! partial commit of the old target.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_functions, lower_program};
use ace_core::source::{
    compile_record_source, compile_text_record_source, compile_text_source, compile_parts_with,
    SourceProfile, CompiledSource,
};
use ace_core::verify::verify_function_ref;

const R07: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/R07.at");
const R08: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/R08.at");
const T03: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T03.at");
const T05: &str = include_str!("../../../../../../crates/auto-lang/test/cases/ace-m1/src/T05.at");

const RUNTIME_TEXT_CAP: &str = "ac.runtime.text.v1";

/// PLAN-902 §5.3 counterexample: declare a, declare b, replace a — the
/// explicit cleanup must free the replacement's OLD buffer after the RHS
/// materialized and close exits in the plan's (declaration-LIFO) order,
/// which plain lexical Drop cannot be trusted to express.
const CLEANUP: &str = r#"fn main() int {
    var a Str = "aa"
    let b Str = a.clone()
    a = a + "bb"
    return a.len() * 10 + b.len()
}"#;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Prof {
    Record,
    Text,
    TextRecord,
}

fn compile_case(name: &str, src: &str, prof: Prof) -> CompiledSource {
    let text_caps = [RUNTIME_TEXT_CAP.to_string()];
    let compiled = match prof {
        Prof::Record => compile_record_source(src, name, &[]),
        Prof::Text => compile_text_source(src, name, &text_caps),
        Prof::TextRecord => compile_text_record_source(src, name, &text_caps),
    };
    compiled.unwrap_or_else(|ds| {
        panic!(
            "frontend diagnostics for {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    })
}

fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/record-text-902")
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Emit + compile + run one case; returns (stdout, stderr).
fn run_chain(
    name: &str,
    src: &str,
    prof: Prof,
    flavor: &str,
    extra_flags: &[&str],
    env: &[(&str, &str)],
) -> (String, String) {
    let compiled = compile_case(name, src, prof);
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    let program = lower_program(&view, "main")
        .unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(name).join(flavor);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();

    let exe = dir.join("a2x_rt_case.exe");
    let mut compiler = Command::new(rustc());
    compiler
        .arg("--edition")
        .arg("2021")
        .args(extra_flags)
        .arg("-o")
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    for (k, v) in env {
        compiler.env(k, v);
    }
    let compile_out = auto_tool_execution::run_command_capture(
        compiler,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert!(
        compile_out.kind == auto_tool_execution::ExitKind::Success,
        "rustc failed for {name} ({flavor}):\n{}",
        String::from_utf8_lossy(compile_out.stderr.as_ref().expect("captured compiler stderr"))
    );

    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit non-zero for {name} ({flavor})"
    );
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("captured stderr")).to_string(),
    )
}

fn assert_value(name: &str, out: &str, expected: i32) {
    assert_eq!(
        out, format!("value:{expected}\n"),
        "stdout mismatch for {name}"
    );
}

/// The four frozen required sources through the real chain, debug/release
/// paired, against the frozen oracle 42/42/23/42.
#[test]
fn record_text_902_required_four() {
    let cases: &[(&str, &str, Prof, i32)] = &[
        ("R07", R07, Prof::Record, 42),
        ("R08", R08, Prof::Record, 42),
        ("T03", T03, Prof::TextRecord, 23),
        ("T05", T05, Prof::Text, 42),
    ];
    for (name, src, prof, value) in cases {
        for (flavor, flags) in [("debug", &[][..]), ("release", &["-O"][..])] {
            let (out, _err) = run_chain(name, src, *prof, flavor, flags, &[]);
            assert_value(name, &out, *value);
        }
    }
}

/// The cleanup counterexample: value oracle plus the EXACT runtime
/// alloc/free event order (replacement frees only after the RHS
/// materialized; exits in the plan's declaration-LIFO order — b before a).
#[test]
fn record_text_902_cleanup_counterexample_events() {
    let (out, err) = run_chain(
        "CLEANUP",
        CLEANUP,
        Prof::Text,
        "events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("CLEANUP", &out, 42);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "aa"
            "rt:alloc:2", // b = a.clone()
            "rt:alloc:4", // a + "bb" (RHS materializes first)
            "rt:free:2",  // replacement frees OLD a after the RHS
            "rt:free:2",  // exit: b (declared after a)
            "rt:free:4",  // exit: a's CURRENT buffer (created after b)
        ],
        "resource event order must follow the verified plan"
    );
}

/// T03's event ledger: owned moves transfer without allocation; the old c
/// frees only after the concat materialized; a is never re-allocated.
#[test]
fn record_text_902_t03_events() {
    let (out, err) = run_chain(
        "T03",
        T03,
        Prof::TextRecord,
        "events",
        &[],
        &[("A2X_RT_EVENTS", "1")],
    );
    assert_value("T03", &out, 23);
    let events: Vec<&str> = err.lines().filter(|l| l.starts_with("rt:")).collect();
    assert_eq!(
        events,
        vec![
            "rt:alloc:2", // a = "ab"
            "rt:alloc:2", // b = a.clone()
            "rt:alloc:5", // c + "中" (2 + 3 bytes)
            "rt:free:2",  // replacement frees old c (b's buffer moved in)
            "rt:free:5",  // exit: c
            "rt:free:2",  // exit: a
        ],
        "resource event order must follow the verified plan"
    );
}

/// The default-off fault hook: the n-th allocation fails with the typed
/// `Resource` trap; the OLD target is never partially committed and no
/// replacement free runs (the RHS never materialized).
#[test]
fn record_text_902_fault_injection() {
    // fault the concat (3rd allocation): a and b allocated, then Resource
    let (out, err) = run_chain(
        "CLEANUP",
        CLEANUP,
        Prof::Text,
        "fault3",
        &[],
        &[("A2X_RT_FAULT_ALLOC", "3")],
    );
    assert_eq!(out, "trap:Resource\n");
    assert!(
        !err.lines().any(|l| l.starts_with("rt:free")),
        "a failed replacement must not free the old target: {err}"
    );

    // T03 with the clone faulted (2nd allocation)
    let (out, _err) = run_chain(
        "T03",
        T03,
        Prof::TextRecord,
        "fault2",
        &[],
        &[("A2X_RT_FAULT_ALLOC", "2")],
    );
    assert_eq!(out, "trap:Resource\n");
}

/// Text indexing traps are typed, never byte-level panics.
#[test]
fn record_text_902_named_traps() {
    let src = r#"fn main() int {
    let s str = "ab中"
    let c char = s.char_at(5)
    return c.to_codepoint()
}"#;
    let (out, _err) = run_chain("CHAR_AT_OOB", src, Prof::Text, "trap", &[], &[]);
    assert_eq!(out, "trap:TextIndex\n");
}

/// The S-window credential: a record-domain METHOD verifies single-body
/// (`verify_function_ref`) and its fragment view renders exactly the
/// method's function — the same bytes the whole-module emission produced.
#[test]
fn record_text_902_method_fragment_credential() {
    // whole-module reference rendering
    let compiled = compile_case("R08", R08, Prof::Record);
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let whole = lower_program(&view, "main").expect("lower");
    let whole_text = ace_a2x_rust::print_program(&whole);
    let read_def = view
        .functions()
        .iter()
        .find(|f| f.source_name == "Counter_read")
        .expect("Counter_read in view")
        .def_index;

    // single-body credential over the same bundle
    let parts = compile_parts_with(R08, "R08.at", &[], SourceProfile::Record)
        .expect("parts");
    let credential = verify_function_ref(&parts.bundle, read_def)
        .unwrap_or_else(|ds| panic!("record method credential: {ds:?}"));
    assert_eq!(credential.domain(), ace_core::hir::HirDomain::Record);
    assert_eq!(credential.def_id_text(), "d_f1_read");

    let frag_view = SemanticView::from_parts(&parts, &credential).expect("fragment view");
    let frag = lower_functions(&frag_view).expect("fragment lowering");
    // exactly the credential's function carries a body; everything else is
    // signature-only
    assert_eq!(frag_view.functions().len(), 1);
    assert_eq!(frag.functions.len(), 1);
    assert_eq!(frag.functions[0].name, "mt_Counter_read");
    let frag_text = ace_a2x_rust::print_program(&frag);
    let whole_fn: String = {
        let start = whole_text.find("pub fn mt_Counter_read").expect("method in whole");
        let end = whole_text[start..]
            .find("}\n")
            .map(|i| start + i + 2)
            .expect("method end");
        whole_text[start..end].to_string()
    };
    let frag_fn: String = {
        let start = frag_text.find("pub fn mt_Counter_read").expect("method in fragment");
        let end = frag_text[start..]
            .find("}\n")
            .map(|i| start + i + 2)
            .expect("fragment end");
        frag_text[start..end].to_string()
    };
    assert_eq!(whole_fn, frag_fn, "fragment must render the method identically");
}

/// The text domains keep the whole-module credential gate (the cross-body
/// exclusivity fixpoint is whole-module until T-03's per-body resource
/// credential) — a named rejection, never a silent pass.
#[test]
fn record_text_902_text_function_gate() {
    let parts = compile_parts_with(T03, "T03.at", &[RUNTIME_TEXT_CAP.to_string()], SourceProfile::TextRecord)
        .expect("parts");
    let err = verify_function_ref(&parts.bundle, 0).expect_err("text fragment must reject");
    assert_eq!(err[0].code, "verify.domain");
}

/// Honest refusal of the not-yet-frozen T-04 surface: resource records
/// (owned Str fields) and `mut` text params are named rejections, never
/// silently wrong Rust.
#[test]
fn record_text_902_t04_boundaries() {
    let resource_record = r#"type Wrap { inner Str }
fn main() int {
    let b Wrap = Wrap("x")
    return b.inner.len()
}"#;
    let compiled = compile_case("RES_REC", resource_record, Prof::TextRecord);
    let err = SemanticView::from_checked(&compiled.checked)
        .and_then(|v| lower_program(&v, "main"))
        .expect_err("resource record must reject");
    assert!(
        err.code.starts_with("a2x."),
        "named rejection, got {err}"
    );

    let mut_text = "fn f(mut s str) int { return s.len_bytes() }\nfn main() int { return f(\"ab\") }";
    let compiled = compile_case("MUT_STR", mut_text, Prof::Text);
    let err = SemanticView::from_checked(&compiled.checked)
        .and_then(|v| lower_program(&v, "main"))
        .expect_err("mut str param must reject");
    assert_eq!(err.code, "a2x.param-mode");
}
