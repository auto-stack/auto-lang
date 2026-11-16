//! PLAN-1062 T-06 (CF-01/CF-04): AC check/build/run sections, non-Auto
//! syntax inputs, Syntax parse/edit sections, product-from; malformed
//! shapes are LOCATED errors (NC-05/17/25 author faces).

use std::path::Path;

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorError};
use auto_test_case::contract::{Action, Adapter, Completion, Stage};
use auto_test_case::document::CaseDocument;

fn decode(body: &str) -> Result<Vec<auto_test_case::author::AuthorCase>, AuthorError> {
    let src = format!("## Case: a\n\n{body}");
    let doc = CaseDocument::parse(src.as_bytes()).unwrap();
    decode_document(&doc, "t06/a.case.ad", &Limits::defaults())
}

fn one(body: &str) -> auto_test_case::author::AuthorCase {
    let mut cases = decode(body).expect("decodes");
    assert_eq!(cases.len(), 1);
    cases.remove(0)
}

fn error_of(body: &str) -> AuthorError {
    decode(body).expect_err("must be a located author error")
}

const AUTO: &str = "### Auto\n\n```auto\nfn main() int { return 1 }\n```\n";

#[test]
fn ac_check_build_run_decode_with_full_options() {
    let case = one(&format!(
        "{AUTO}\n### AC chk\naction: \"check\"\nprofile: \"scalar\"\ncaps: [\"hir.test.trace.v1\"]\nsupport-lib: \"trace-marks\"\n\n\
### AC bld\naction: \"build\"\nentry: \"test_entry\"\ntarget: \"x86_64-pc-windows-msvc\"\ncache: \"fresh\"\nbudget: {{build_ms: 100, run_ms: 50, total_ms: 150}}\nproduct-from: \"chk\"\n\n\
### AC run\naction: \"run\"\nexit: 5\n"
    ));
    let checks = &case.spec.checks;
    assert_eq!(checks.len(), 3);
    assert_eq!((checks[0].adapter, checks[0].action), (Adapter::Ac, Action::Check));
    assert_eq!(checks[0].options.ac_profile.as_ref().map(|p| p.as_str()), Some("scalar"));
    assert_eq!(
        checks[0].options.ac_capabilities.as_deref(),
        Some(&["hir.test.trace.v1".to_string()][..])
    );
    assert_eq!(checks[0].options.ac_support_lib.as_deref(), Some("trace-marks"));
    assert_eq!((checks[1].adapter, checks[1].action), (Adapter::Ac, Action::Build));
    assert_eq!(checks[1].options.ac_entry.as_deref(), Some("test_entry"));
    assert_eq!(checks[1].options.ac_cache.as_ref().map(|c| c.as_str()), Some("fresh"));
    let budget = checks[1].options.ac_budget.as_ref().unwrap();
    assert_eq!((budget.build_ms, budget.run_ms, budget.total_ms), (Some(100), Some(50), Some(150)));
    assert_eq!(checks[1].product_from.as_deref(), Some("chk"));
    assert_eq!((checks[2].adapter, checks[2].action), (Adapter::Ac, Action::Run));
    assert_eq!(checks[2].expect.exit, Some(5));
}

#[test]
fn ac_error_expectations_cover_the_full_stage_set() {
    for (stage_text, stage) in [
        ("parse", Stage::Parse),
        ("resolve", Stage::Resolve),
        ("typecheck", Stage::Typecheck),
        ("bind", Stage::Bind),
        ("verify", Stage::Verify),
        ("capability", Stage::Capability),
        ("backend", Stage::Backend),
        ("link", Stage::Link),
        ("run", Stage::Run),
    ] {
        let case = one(&format!(
            "{AUTO}\n### AC bad\naction: \"check\"\nstage: \"{stage_text}\"\ncode: \"E\"\nmessage-contains: \"boom\"\n"
        ));
        match &case.spec.checks[0].expect.completion {
            Completion::Error { stage: s, code, message } => {
                assert_eq!(*s, stage, "stage {stage_text}");
                assert_eq!(code.as_deref(), Some("E"));
                assert!(message.is_some());
            }
            other => panic!("expected error completion for {stage_text}, got {other:?}"),
        }
    }
    let e = error_of(&format!(
        "{AUTO}\n### AC bad\naction: \"check\"\nstage: \"type\"\ncode: \"E\"\n"
    ));
    assert!(e.message.contains("AC"), "{e:?}");
}

#[test]
fn hir_atom_source_rides_the_syntax_attribute() {
    let case = one(
        "### Auto\nsyntax: \"hir-atom\"\npath: \"main.atom\"\n\n```auto\nhir(schema: \"x\") {}\n```\n\n### AC chk\naction: \"check\"\n",
    );
    assert_eq!(case.spec.source.syntax.as_str(), "hir-atom");
    assert_eq!(case.spec.source.path, "main.atom");
    let e = error_of("### Auto\nsyntax: \"wat\"\npath: \"m\"\n\n```auto\nx\n```\n");
    assert!(e.message.contains("未注册"), "{e:?}");
}

#[test]
fn syntax_parse_and_edit_decode() {
    let case = one(&format!(
        "{AUTO}\n### Syntax p\naction: \"parse\"\ngrammar: \"auto-core\"\n\n\
### Syntax e\naction: \"edit\"\nedits: [{{start: 0, end: 4, insert: \"let x\"}}]\nstage: \"parse\"\ncode: \"E1\"\n"
    ));
    let checks = &case.spec.checks;
    assert_eq!((checks[0].adapter, checks[0].action), (Adapter::Syntax, Action::Parse));
    assert_eq!(checks[0].options.syntax_grammar.as_deref(), Some("auto-core"));
    assert_eq!((checks[1].adapter, checks[1].action), (Adapter::Syntax, Action::Edit));
    let edits = checks[1].options.syntax_edits.as_ref().unwrap();
    assert_eq!((edits[0].start, edits[0].end, edits[0].insert.as_str()), (0, 4, "let x"));
    match &checks[1].expect.completion {
        Completion::Error { stage: Stage::Parse, code, .. } => {
            assert_eq!(code.as_deref(), Some("E1"));
        }
        other => panic!("syntax edit error completion expected, got {other:?}"),
    }
}

#[test]
fn ac_syntax_sections_carry_stderr_streams() {
    let case = one(&format!(
        "{AUTO}\n### AC run\naction: \"run\"\n\n#### stderr\n\n```text\nboom\n```\n"
    ));
    assert_eq!(case.spec.checks[0].expect.stderr.as_ref().map(|s| s.text.as_str()), Some("boom\n"));
}

#[test]
fn nc05_edit_shapes_and_grammar_rules_are_located_errors() {
    let e = error_of(&format!(
        "{AUTO}\n### Syntax e\naction: \"edit\"\nedits: [{{start: 4, end: 0, insert: \"x\"}}]\n"
    ));
    assert!(e.message.contains("区间非法"), "{e:?}");

    let e = error_of(&format!(
        "{AUTO}\n### Syntax e\naction: \"edit\"\nedits: [{{start: 0, end: 4}}]\n"
    ));
    assert!(e.message.contains("三键齐全"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### Syntax e\naction: \"edit\"\n"));
    assert!(e.message.contains("edits"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### Syntax p\naction: \"parse\"\nedits: [{{start: 0, end: 1, insert: \"x\"}}]\n"));
    assert!(e.message.contains("不接受 edits"), "{e:?}");
}

#[test]
fn nc25_unknown_actions_profiles_and_attrs_are_located_errors() {
    let e = error_of(&format!("{AUTO}\n### AC x\naction: \"execute\"\n"));
    assert!(e.message.contains("check/build/run"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### AC x\naction: \"check\"\nprofile: \"turbo\"\n"));
    assert!(e.message.contains("core/scalar"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### AC x\naction: \"check\"\ncache: \"always\"\n"));
    assert!(e.message.contains("fresh/reuse/read-only"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### AC x\naction: \"check\"\nexecutor: \"/bin/sh\"\n"));
    assert!(e.message.contains("不属于该节") || e.message.contains("未知"), "{e:?}");

    let e = error_of(&format!("{AUTO}\n### Syntax x\naction: \"transpile\"\n"));
    assert!(e.message.contains("parse/edit"), "{e:?}");
}

#[test]
fn nc17_product_from_cycle_is_rejected_at_admission() {
    // decode level: both directions are representable; the DAG rule
    // (earlier check only) fires at suite admission. Exercise it through
    // the real suite loader with a temp tree.
    let base = std::env::temp_dir().join(format!("t06-nc17-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let cases = base.join("c");
    std::fs::create_dir_all(&cases).unwrap();
    let doc = format!(
        "# t\n\n## Case: a\nid: \"t06.a\"\n\n{AUTO}\n\
### AC first\naction: \"check\"\nproduct-from: \"second\"\n\n\
### AC second\naction: \"run\"\nproduct-from: \"first\"\n"
    );
    let case_file = cases.join("a.case.ad");
    std::fs::write(&case_file, doc).unwrap();
    let suite = base.join("s.suite.ad");
    std::fs::write(
        &suite,
        "```test-suite
suite { schema: \"autolang.suite/2\", id: \"s\", include: [\"c/a.case.ad\"] }
```",
    )
    .unwrap();
    let err = auto_test_case::discovery::load_suite(&suite, &Limits::defaults())
        .err()
        .expect("cycle must be rejected");
    let text = err.to_string();
    assert!(text.contains("product_from") && text.contains("cycle"), "{text}");
}

// cleanup happens best-effort; the temp dir is OS-managed
