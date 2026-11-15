//! PLAN-1062 T-04 (CF-02/CF-03): diagnostic-message faces and the full
//! registered probe-recipe vocabulary decode losslessly; malformed shapes
//! are LOCATED errors (NC-03/06/07-flavored author gates).

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorError};
use auto_test_case::contract::{Completion, MessageMatcher, Options, Stage};
use auto_test_case::document::CaseDocument;

const AUTO_SECTION: &str = "### Auto\n\n```auto\nfn main() int { return 1 }\n```\n";

fn decode(body: &str) -> Result<Vec<auto_test_case::author::AuthorCase>, AuthorError> {
    let src = format!("## Case: a\n\n{AUTO_SECTION}{body}");
    let doc = CaseDocument::parse(src.as_bytes()).unwrap();
    decode_document(&doc, "t04/a.case.ad", &Limits::defaults())
}

fn one(body: &str) -> auto_test_case::author::AuthorCase {
    let mut cases = decode(body).expect("decodes");
    assert_eq!(cases.len(), 1);
    cases.remove(0)
}

fn error_of(body: &str) -> AuthorError {
    decode(body).expect_err("must be a located author error")
}

#[test]
fn vm_diagnostic_message_exact_and_contains_decode() {
    let case = one(
        "### VM diagnostic message\nstage: \"type\"\ncode: \"E0203\"\n\n```text\n变量类型不匹配\n```\n",
    );
    let check = &case.spec.checks[0];
    match &check.expect.completion {
        Completion::Error {
            stage: Stage::Type,
            code,
            message: Some(MessageMatcher::Exact(text)),
        } => {
            assert_eq!(code.as_deref(), Some("E0203"));
            assert_eq!(text, "变量类型不匹配");
        }
        other => panic!("exact matcher expected, got {other:?}"),
    }

    let case = one(
        "### VM diagnostic message\nstage: \"parse\"\ncode: \"E0101\"\ncontains: true\n\n```text\nunterminated\n```\n",
    );
    match &case.spec.checks[0].expect.completion {
        Completion::Error {
            stage: Stage::Parse,
            code,
            message: Some(MessageMatcher::Contains(text)),
        } => {
            assert_eq!(code.as_deref(), Some("E0101"));
            assert_eq!(text, "unterminated");
        }
        other => panic!("contains matcher expected, got {other:?}"),
    }
}

#[test]
fn rust_diagnostic_decodes_to_a2r_error_check() {
    let case = one(
        "### Rust diagnostic\ncheck-id: \"rust-fail\"\nstage: \"typecheck\"\ncode: \"a2r.E0203\"\ncontains: true\n\n```text\nmismatch\n```\n",
    );
    let check = &case.spec.checks[0];
    assert_eq!(check.id, "rust-fail");
    assert_eq!(check.adapter.as_str(), "a2r");
    match &check.expect.completion {
        Completion::Error {
            stage: Stage::Typecheck,
            code,
            message: Some(MessageMatcher::Contains(text)),
        } => {
            assert_eq!(code.as_deref(), Some("a2r.E0203"));
            assert_eq!(text, "mismatch");
        }
        other => panic!("error completion expected, got {other:?}"),
    }
}

#[test]
fn every_probe_family_wires_its_binding() {
    // sema
    let case = one(
        "### Probe sema-q\nrecipe: \"sema/query-v1\"\npackage: \"p\"\nmodule: \"m\"\nkind: \"infer-body\"\nfn: \"f\"\n",
    );
    let sema = &case.spec.checks[0].options.probe_sema;
    assert_eq!(
        sema.as_ref().map(|s| (&s.package, &s.module, &s.kind, &s.fn_name)),
        Some((
            &"p".to_string(),
            &"m".to_string(),
            &"infer-body".to_string(),
            &"f".to_string()
        ))
    );

    // abi
    let case = one("### Probe abi-leg\nrecipe: \"abi/interop-v1\"\nleg: \"b\"\n");
    assert_eq!(
        case.spec.checks[0]
            .options
            .probe_abi
            .as_ref()
            .map(|a| a.leg.as_str()),
        Some("b")
    );

    // cte with session and declared empty caps
    let case = one(
        "### Probe cte-q\nrecipe: \"cte/scalar-query-v1\"\nentry: \"main\"\nsession: \"s1\"\ncaps: []\n",
    );
    let cte = case.spec.checks[0].options.probe_cte.as_ref().unwrap();
    assert_eq!(cte.entry, "main");
    assert_eq!(cte.session.as_deref(), Some("s1"));
    assert!(cte.caps.is_empty(), "declared empty caps decode as empty list");

    // av with profile + session
    let case = one(
        "### Probe av-run\nrecipe: \"ac-av/owned-run-v1\"\nentry: \"d_main\"\nprofile: \"owned\"\nsession: \"warm\"\n",
    );
    let av = case.spec.checks[0].options.probe_av.as_ref().unwrap();
    assert_eq!(av.entry, "d_main");
    assert_eq!(av.profile.as_deref(), Some("owned"));
    assert_eq!(av.session.as_deref(), Some("warm"));

    // mn with diagnostics
    let case = one(
        "### Probe mn-run\nrecipe: \"ac/mir-native-scalar-v1\"\nentry: \"main\"\ndiagnostics: \"source-v1\"\n",
    );
    let mn = case.spec.checks[0].options.probe_mn.as_ref().unwrap();
    assert_eq!(mn.entry, "main");
    assert_eq!(mn.diagnostics.as_deref(), Some("source-v1"));

    // a2x emit
    let case = one(
        "### Probe emit\nrecipe: \"ac-a2r/hir-scalar-emit-v1\"\nprofile: \"core\"\nentry: \"main\"\n",
    );
    let a2x = case.spec.checks[0]
        .options
        .probe_a2x_hir
        .as_ref()
        .unwrap();
    assert_eq!(a2x.profile, "core");
    assert_eq!(a2x.entry, "main");
}

#[test]
fn probe_error_expectations_carry_message_on_any_family() {
    let case = one(
        "### Probe mn-diag\nrecipe: \"ac/mir-native-scalar-v1\"\nentry: \"main\"\nstage: \"run\"\ncode: \"mn.trap\"\nmessage-contains: \"boom\"\n",
    );
    match &case.spec.checks[0].expect.completion {
        Completion::Error {
            stage: Stage::Run,
            code,
            message: Some(MessageMatcher::Contains(text)),
        } => {
            assert_eq!(code.as_deref(), Some("mn.trap"));
            assert_eq!(text, "boom");
        }
        other => panic!("error completion expected, got {other:?}"),
    }
    assert!(case.spec.checks[0].expect.exit.is_none());
}

#[test]
fn unregistered_recipe_and_wrong_bindings_are_located_errors() {
    let e = error_of("### Probe bad\nrecipe: \"not/a-recipe-v1\"\nentry: \"main\"\n");
    assert!(e.message.contains("未注册"), "{e:?}");

    let e = error_of(
        "### Probe sema-x\nrecipe: \"sema/query-v1\"\nmodule: \"m\"\nkind: \"infer-body\"\nfn: \"f\"\n",
    );
    assert!(e.message.contains("package"), "{e:?}");

    let e = error_of(
        "### Probe sema-caps\nrecipe: \"sema/query-v1\"\npackage: \"p\"\nmodule: \"m\"\nkind: \"infer-body\"\nfn: \"f\"\ncaps: [\"x\"]\n",
    );
    assert!(e.message.contains("不接受 entry/caps"), "{e:?}");

    let e = error_of("### Probe abi-x\nrecipe: \"abi/interop-v1\"\n");
    assert!(e.message.contains("leg"), "{e:?}");

    let e = error_of(
        "### Probe emit-x\nrecipe: \"ac-a2r/hir-scalar-emit-v1\"\nentry: \"main\"\n",
    );
    assert!(e.message.contains("profile"), "{e:?}");
}

#[test]
fn message_without_stage_code_and_conflicting_faces_are_errors() {
    let e = error_of(
        "### Probe m1\nrecipe: \"ac-av/owned-run-v1\"\nentry: \"d_main\"\nmessage: \"x\"\n",
    );
    assert!(e.message.contains("成对"), "{e:?}");

    let e = error_of(
        "### Probe m2\nrecipe: \"ac-av/owned-run-v1\"\nentry: \"d_main\"\nstage: \"run\"\ncode: \"c\"\nmessage: \"x\"\nmessage-contains: \"y\"\n",
    );
    assert!(e.message.contains("互斥"), "{e:?}");

    let e = error_of(
        "### Probe m3\nrecipe: \"ac-av/owned-run-v1\"\nentry: \"d_main\"\nexit: 0\nstage: \"run\"\ncode: \"c\"\n",
    );
    assert!(e.message.contains("exit"), "{e:?}");

    // VM success + VM diagnostic message conflict
    let e = error_of(
        "### VM result\n\n```text\n7\n```\n\n### VM diagnostic message\nstage: \"run\"\ncode: \"c\"\n\n```text\nm\n```\n",
    );
    assert!(e.message.contains("成功"), "{e:?}");

    // Rust golden + Rust diagnostic conflict
    let e = error_of(
        "### Rust\n\n```rust\nfn main() {}\n```\n\n### Rust diagnostic\nstage: \"typecheck\"\ncode: \"c\"\n\n```text\nm\n```\n",
    );
    assert!(e.message.contains("金样"), "{e:?}");
}

#[test]
fn options_default_face_stays_representable() {
    // The default Options face (no probe) is unchanged by T-04: a plain VM
    // result case still decodes with default options.
    let case = one("### VM result\n\n```text\n7\n```\n");
    assert_eq!(case.spec.checks[0].options, Options::default());
}
