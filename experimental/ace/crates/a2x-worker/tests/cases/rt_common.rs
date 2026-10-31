//! PLAN-902 shared record/text true-chain harness (worker conformance):
//! compile → widened view → Rust provider → fileset → REAL rustc → real
//! run → parent-side oracle. Used by record_text_902.rs and
//! record_text_modes.rs.

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::{
    compile_record_source, compile_text_record_source, compile_text_source, CompiledSource,
};

pub const RUNTIME_TEXT_CAP: &str = "ac.runtime.text.v1";

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Prof {
    Record,
    Text,
    TextRecord,
}

pub fn compile_case(name: &str, src: &str, prof: Prof) -> CompiledSource {
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

pub fn out_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/record-text-902")
}

pub fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string())
}

/// Emit + compile + run one case; returns (stdout, stderr).
pub fn run_chain(
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

pub fn assert_value(name: &str, out: &str, expected: i32) {
    assert_eq!(
        out, format!("value:{expected}\n"),
        "stdout mismatch for {name}"
    );
}
