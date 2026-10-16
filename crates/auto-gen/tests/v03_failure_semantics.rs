//! PLAN-791 V03 — failure/back-stop semantics of the CURRENT (pre-extraction)
//! `auto-gen` output orchestration, frozen before the auto-gen-core
//! extraction. Invariants under test:
//! - render/guard failure must NEVER overwrite or touch an existing output
//!   file (the write only happens after render+merge succeeded);
//! - `dry_run` produces zero directories and zero files;
//! - a failing template does not abort the batch: earlier outputs persist,
//!   later templates still run, errors are collected per template;
//! - guard sections are preserved from the existing file on regeneration and
//!   dropped only with `overwrite_guarded: true`.

mod common;

use auto_gen::{
    CodeGenerator, DataSource, GenerationSpec, GeneratorConfig, GuardProcessor, TemplateSpec,
};
use common::{sample_atom, templates_dir};

fn generate(
    out_dir: &std::path::Path,
    dry_run: bool,
    overwrite_guarded: bool,
    templates: Vec<TemplateSpec>,
) -> auto_gen::GenReport {
    let mut gen = CodeGenerator::new(GeneratorConfig {
        output_dir: out_dir.to_path_buf(),
        dry_run,
        overwrite_guarded,
        ..Default::default()
    });
    let spec = GenerationSpec {
        data_source: DataSource::Atom(sample_atom()),
        templates,
        lib_files: vec![],
        lib_paths: vec![],
    };
    gen.generate(&spec)
        .unwrap_or_else(|e| panic!("top-level error in Atom case: {e}"))
}

fn tpl(name: &str) -> TemplateSpec {
    TemplateSpec {
        template_path: templates_dir().join(name),
        output_name: None,
        rename: false,
    }
}

/// Regeneration preserves hand-edited guarded sections byte-exactly while
/// regenerating the rest of the file.
#[test]
fn guard_preserved_on_regenerate() {
    let tmp = tempfile::tempdir().unwrap();
    let existing = tmp.path().join("guarded.txt");
    std::fs::write(
        &existing,
        "stale header\n/// ---------- begin of guard: <user> ---\nHAND EDITED\n/// ---------- end of guard: ---\nstale footer\n",
    )
    .unwrap();

    let report = generate(tmp.path(), false, false, vec![tpl("guarded.txt.at")]);
    assert!(report.errors.is_empty(), "{:?}", report.errors);

    let got = std::fs::read_to_string(&existing).unwrap();
    let want = "generated header\n/// ---------- begin of guard: <user> ---\nHAND EDITED\n/// ---------- end of guard: ---\ngenerated footer\n";
    assert_eq!(got, want);
}

/// `overwrite_guarded: true` replaces the guarded body with the template's.
#[test]
fn overwrite_guarded_replaces() {
    let tmp = tempfile::tempdir().unwrap();
    let existing = tmp.path().join("guarded.txt");
    std::fs::write(
        &existing,
        "stale header\n/// ---------- begin of guard: <user> ---\nHAND EDITED\n/// ---------- end of guard: ---\nstale footer\n",
    )
    .unwrap();

    let report = generate(tmp.path(), false, true, vec![tpl("guarded.txt.at")]);
    assert!(report.errors.is_empty(), "{:?}", report.errors);

    let got = std::fs::read_to_string(&existing).unwrap();
    assert!(
        got.contains("template body"),
        "guard body not overwritten: {got}"
    );
    assert!(!got.contains("HAND EDITED"), "stale body survived: {got}");
}

/// A template whose render fails must not overwrite the existing output
/// file: its bytes stay exactly as before the run.
#[test]
fn render_failure_keeps_existing_bytes() {
    let tmp = tempfile::tempdir().unwrap();
    let sentinel = b"precious bytes\n".to_vec();
    let existing = tmp.path().join("bad_lookup.txt");
    std::fs::write(&existing, &sentinel).unwrap();

    let report = generate(tmp.path(), false, false, vec![tpl("bad_lookup.txt.at")]);
    assert_eq!(report.files_generated.len(), 0);
    assert_eq!(report.errors.len(), 1);

    let after = std::fs::read(&existing).unwrap();
    assert_eq!(after, sentinel, "failed render overwrote existing output");
}

/// Render failure must not even create the parent directory of the output.
#[test]
fn render_failure_creates_no_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = TemplateSpec {
        template_path: templates_dir().join("bad_lookup.txt.at"),
        output_name: Some("nested/deep/out.txt".into()),
        rename: false,
    };
    let report = generate(tmp.path(), false, false, vec![spec]);
    assert_eq!(report.errors.len(), 1);
    assert!(
        !tmp.path().join("nested").exists(),
        "parent dir created despite render failure"
    );
}

/// dry_run reports the would-be files but writes nothing — no files, no dirs.
#[test]
fn dry_run_zero_writes() {
    let tmp = tempfile::tempdir().unwrap();
    let report = generate(
        tmp.path(),
        true,
        false,
        vec![tpl("hello.txt.at"), tpl("static.txt.at")],
    );
    assert_eq!(report.files_generated.len(), 2);
    assert!(report.errors.is_empty());
    let entries: Vec<_> = std::fs::read_dir(tmp.path()).unwrap().collect();
    assert!(
        entries.is_empty(),
        "dry_run created {entries:?} under the output dir"
    );
}

/// A successful output whose output_name nests directories creates them on
/// write (positive counterpart of `render_failure_creates_no_dirs`).
#[test]
fn nested_output_name_creates_dirs() {
    let tmp = tempfile::tempdir().unwrap();
    let spec = TemplateSpec {
        template_path: templates_dir().join("hello.txt.at"),
        output_name: Some("nested/deep/hello.txt".into()),
        rename: false,
    };
    let report = generate(tmp.path(), false, false, vec![spec]);
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let content = std::fs::read_to_string(tmp.path().join("nested/deep/hello.txt")).unwrap();
    assert_eq!(content, "Hello, AuthService!\nV: 2");
}

/// Batch continues past a failing template: the good template before AND
/// after the bad one are written; exactly one error is collected and
/// attributed to the failing template.
#[test]
fn batch_continues_past_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let report = generate(
        tmp.path(),
        false,
        false,
        vec![
            tpl("hello.txt.at"),
            tpl("bad_lookup.txt.at"),
            tpl("static.txt.at"),
        ],
    );
    assert_eq!(report.files_generated.len(), 2);
    assert_eq!(report.errors.len(), 1);
    let msg = format!("{}", report.errors[0]);
    assert!(
        msg.contains("bad_lookup.txt"),
        "error attributed to wrong template: {msg}"
    );
    assert!(tmp.path().join("hello.txt").is_file());
    assert!(tmp.path().join("static.txt").is_file());
    assert!(!tmp.path().join("bad_lookup.txt").exists());
}

/// Guard conflict detection (public GuardProcessor API) stays available and
/// honest: differing guard bodies yield a conflict, identical bodies do not.
#[test]
fn detect_conflicts_api() {
    let processor = GuardProcessor::new();
    let a = "/// ---------- begin of guard: <g1> ---\none\n/// ---------- end of guard: ---\n";
    let b = "/// ---------- begin of guard: <g1> ---\ntwo\n/// ---------- end of guard: ---\n";
    let conflicts = processor.detect_conflicts(a, b);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].guard_id, "g1");
    assert!(processor.detect_conflicts(a, a).is_empty());
}
