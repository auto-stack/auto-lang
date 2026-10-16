//! PLAN-788 T-06 syntax adapter controls (V-11/V-13 anchors at the worker
//! level): both grammars, lossless reconstruction, typed parse diagnostics,
//! fresh↔incremental equivalence (worker-proven), sequential revisions, and
//! truthful reuse counters. Windows-only for uniformity with the family.

#![cfg(windows)]

use ac_case_worker::exec;
use ac_case_worker::protocol::{EditStep, InputSpec, ReqBudget, WorkerRequest, WORKER_SCHEMA};
use base64::Engine;
use std::path::PathBuf;

fn b64(s: &str) -> String {
    base64::engine::general_purpose::STANDARD.encode(s.as_bytes())
}

static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn scratch() -> PathBuf {
    let p = std::env::temp_dir().join(format!(
        "ac788-syn-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn req(kind: &str, grammar: &str, source: &str, edits: Vec<EditStep>) -> WorkerRequest {
    let n = SEQ.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    WorkerRequest {
        schema: WORKER_SCHEMA.into(),
        kind: kind.into(),
        nonce: format!("n-{}", n),
        request_hash: format!("h-{}", n),
        input: InputSpec::Source {
            source_b64: b64(source),
        },
        profile: "core".into(),
        entry: "main".into(),
        budget: ReqBudget {
            build_ms: 60_000,
            run_ms: 5_000,
            total_ms: 70_000,
        },
        scratch: scratch().display().to_string(),
        store_root: None,
        cache: None,
        project: None,
        include_hir: None,
        capabilities: Vec::new(),
        support_lib: None,
        grammar: Some(grammar.into()),
        edits: (!edits.is_empty()).then_some(edits),
    }
}

fn artifact<'a>(ok: &'a ac_case_worker::OkOut, logical: &str) -> &'a ac_case_worker::ArtifactOut {
    ok.artifacts.iter().find(|a| a.logical == logical).unwrap()
}

fn counter(ok: &ac_case_worker::OkOut, name: &str) -> u32 {
    ok.cache
        .as_ref()
        .and_then(|c| c.counters.iter().find(|(n, _)| n == name))
        .map(|(_, v)| *v)
        .unwrap_or(0)
}

const P04: &str = "fn main() int { var i = 0; while i < 3 { i = i + 1 }; return i }";
const P04_4: &str = "fn main() int { var i = 0; while i < 4 { i = i + 1 }; return i }";
const MINI: &str = "api add(a: i32, b: i32) -> i32; note \"adds two numbers\";";

#[test]
fn syntax_case_parse_lossless_core_grammar() {
    let ok = exec::execute(&req("parse", "auto-core", P04, vec![])).unwrap();
    assert!(ok.diagnostics.is_empty(), "{:#?}", ok.diagnostics);
    // Lossless: the reconstruction IS the input, byte for byte.
    let rec = artifact(&ok, "reconstructed.at");
    let bytes = std::fs::read(&rec.path).unwrap();
    assert_eq!(bytes, P04.as_bytes(), "reconstruction is lossless");
    // Hash binds the real file.
    assert_eq!(rec.bytes, P04.len());
    // Outline and canonical projections present and non-empty.
    assert!(artifact(&ok, "outline.json").bytes > 2);
    assert!(artifact(&ok, "cst.canonical").bytes > 0);
}

#[test]
fn syntax_case_parse_diag_typed_with_span() {
    let bad = "fn main( int { return 3 }";
    let ok = exec::execute(&req("parse", "auto-core", bad, vec![])).unwrap();
    assert!(
        !ok.diagnostics.is_empty(),
        "an invalid program must diagnose"
    );
    for d in &ok.diagnostics {
        assert_eq!(d.stage, "parse");
        assert!(d.code == "syntax.lex" || d.code == "syntax.parse");
        assert!(d.span_end >= d.span_start);
    }
    // A broken source still reconstructs losslessly (diagnostics never
    // corrupt the text).
    let bytes = std::fs::read(&artifact(&ok, "reconstructed.at").path).unwrap();
    assert_eq!(bytes, bad.as_bytes());
}

#[test]
fn syntax_case_interface_mini_grammar() {
    let ok = exec::execute(&req("parse", "interface-mini-v1", MINI, vec![])).unwrap();
    assert!(ok.diagnostics.is_empty(), "{:#?}", ok.diagnostics);
    let bytes = std::fs::read(&artifact(&ok, "reconstructed.at").path).unwrap();
    assert_eq!(bytes, MINI.as_bytes());

    let bad = "api ( : ; )";
    let ok = exec::execute(&req("parse", "interface-mini-v1", bad, vec![])).unwrap();
    assert!(
        !ok.diagnostics.is_empty(),
        "invalid interface-mini must diagnose"
    );
}

#[test]
fn syntax_case_edit_equivalence_and_counters() {
    // A MULTI-decl program so a body edit in main leaves helper decls to
    // graft: reuse is observable with the real runtime counters.
    let src = format!(
        "fn helper() int {{ return 7 }}
{}",
        P04
    );
    let pos = src.find("3").unwrap() as u64;
    let edit = EditStep {
        start: pos,
        end: pos + 1,
        insert_b64: b64("4"),
    };
    let ok = exec::execute(&req("edit", "auto-core", &src, vec![edit])).unwrap();
    assert!(ok.diagnostics.is_empty(), "{:#?}", ok.diagnostics);
    // Final revision's reconstruction is the EDITED text, lossless.
    let bytes = std::fs::read(&artifact(&ok, "reconstructed.at").path).unwrap();
    assert_eq!(bytes, src.replace("3", "4").as_bytes());
    assert_eq!(counter(&ok, "revision"), 1);
    // Truthful reuse: the helper decl is grafted (reused_decls >= 1) or
    // the fallback is VISIBLE — the counters are the real runtime numbers,
    // and fresh↔incremental equivalence was proven inside (else infra).
    let reused = counter(&ok, "reused_decls");
    let fallbacks = counter(&ok, "fallbacks");
    assert!(
        reused >= 1 || fallbacks >= 1,
        "either grafts were reused or the fallback is visible (reused={}, fallbacks={})",
        reused,
        fallbacks
    );
    if fallbacks == 0 {
        assert!(
            counter(&ok, "rescan_bytes") <= 64 * 1024,
            "rescan window bound"
        );
        assert!(
            counter(&ok, "reparsed_decls") <= 2,
            "a local body edit reparses at most the edited decl"
        );
    }
}

#[test]
fn revision_trace_sequential_edits_advance() {
    // Two sequential single-edit steps: 0→1→2; step-2 coordinates are
    // relative to the revision-1 text.
    let pos3 = P04.find("3").unwrap() as u64;
    let step1 = EditStep {
        start: pos3,
        end: pos3 + 1,
        insert_b64: b64("4"),
    };
    // After step 1 the text is P04_4; find a stable anchor for step 2:
    // replace the leading "var" with "var" + no-op? Use the '0' of `= 0`.
    let pos0 = P04.find("= 0").unwrap() as u64 + 2;
    let step2 = EditStep {
        start: pos0,
        end: pos0 + 1,
        insert_b64: b64("1"),
    };
    let ok = exec::execute(&req("edit", "auto-core", P04, vec![step1, step2])).unwrap();
    assert!(ok.diagnostics.is_empty(), "{:#?}", ok.diagnostics);
    assert_eq!(counter(&ok, "revision"), 2, "two steps, two revisions");
    let final_text =
        String::from_utf8(std::fs::read(&artifact(&ok, "reconstructed.at").path).unwrap()).unwrap();
    assert!(final_text.contains("while i < 4"), "{}", final_text);
    assert!(final_text.contains("= 1"), "{}", final_text);
    // The trace carries one span per edit step.
    let stages: Vec<&str> = ok.trace.iter().map(|s| s.stage.as_str()).collect();
    assert!(
        stages.contains(&"edit-0") && stages.contains(&"edit-1"),
        "{:?}",
        stages
    );
}

#[test]
fn syntax_case_edit_rejected_by_engine_is_admission() {
    // An out-of-range span is rejected by the snapshot engine BEFORE any
    // parsing — a located admission error, not a diagnostics observation.
    let bad = EditStep {
        start: 1_000_000,
        end: 1_000_001,
        insert_b64: b64("x"),
    };
    let err = exec::execute(&req("edit", "auto-core", P04, vec![bad])).unwrap_err();
    assert_eq!(err.code, "request.edits", "{}", err.message);
}

#[test]
fn syntax_case_wrong_grammar_for_kind_rejected() {
    // grammar must be present exactly on parse/edit, and only known names.
    let mut r = req("check", "auto-core", P04, vec![]);
    r.grammar = Some("nope".into());
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.grammar", "{}", err.message);

    let mut r = req("parse", "auto-core", P04, vec![]);
    r.grammar = None;
    let err = exec::execute(&r).unwrap_err();
    assert_eq!(err.code, "request.grammar");
}
