//! PLAN-1062 T-10 (CF-06/CF-07): full ordered step vocabulary, AV/MN nested
//! oracles, per-step Observation expectations; malformed shapes are LOCATED
//! errors (NC-04/12/13/16 author faces).

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorError};
use auto_test_case::contract::{Completion, MessageMatcher, ProbeStep, Stage};
use auto_test_case::document::CaseDocument;

const AUTO_SECTION: &str = "### Auto\n\n```auto\nfn main() int { return 1 }\n```\n";

fn decode(body: &str) -> Result<Vec<auto_test_case::author::AuthorCase>, AuthorError> {
    let src = format!("## Case: a\n\n{AUTO_SECTION}{body}");
    let doc = CaseDocument::parse(src.as_bytes()).unwrap();
    decode_document(&doc, "t10/a.case.ad", &Limits::defaults())
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
fn full_step_vocabulary_decodes_in_order() {
    let steps_line = String::from("steps: [{op: \"run\"}, {op: \"edit-source\", from: \"a\", to: \"b\"}, ")
        + "{op: \"query\"}, {op: \"partial-rebuild\", from: \"a\", to: \"b\", defs: [\"d_x\", \"d_y\"]}, "
        + "{op: \"declaration-invalidate\", from: \"a\", to: \"b\"}, {op: \"restart\"}, "
        + "{op: \"resident-slice\", slices: 3}, {op: \"resident-apply\"}, "
        + "{op: \"resident-finish\"}, \"run\"]";
    let case = one(&format!(
        "### Probe flow\nrecipe: \"ac-av/data-incremental-v1\"\nentry: \"d_main\"\n{steps_line}\n"
    ));
    let steps = case.spec.checks[0].options.probe_steps.as_ref().unwrap();
    assert_eq!(steps.len(), 10);
    assert!(matches!(steps[0], ProbeStep::Run));
    assert!(matches!(
        &steps[1],
        ProbeStep::EditSource { from, to } if from == "a" && to == "b"
    ));
    assert!(matches!(steps[2], ProbeStep::Query));
    assert!(matches!(
        &steps[3],
        ProbeStep::PartialRebuild { from, to, defs } if from == "a" && to == "b"
            && defs == &vec!["d_x".to_string(), "d_y".to_string()]
    ));
    assert!(matches!(
        &steps[4],
        ProbeStep::DeclarationInvalidate { from, to } if from == "a" && to == "b"
    ));
    assert!(matches!(steps[5], ProbeStep::Restart));
    assert!(matches!(steps[6], ProbeStep::ResidentSlice { slices: 3 }));
    assert!(matches!(steps[7], ProbeStep::ResidentApply));
    assert!(matches!(steps[8], ProbeStep::ResidentFinish));
    assert!(matches!(steps[9], ProbeStep::Run));
}

#[test]
fn av_nested_oracles_decode() {
    let case = one(concat!(
        "### Probe incr\n",
        "recipe: \"ac-av/data-incremental-v1\"\n",
        "entry: \"d_main\"\n",
        "invalidation: {stable_defs: [\"d_a\"], rebuilt_defs: [\"d_b\", \"d_c\"]}\n",
        "av-resource: {profile: \"owned\", max_steps: 1000, cancel_after_ms: 500}\n",
    ));
    let av = case.spec.checks[0].options.probe_av.as_ref().unwrap();
    let inv = av.invalidation.as_ref().unwrap();
    assert_eq!(inv.stable_defs, vec!["d_a"]);
    assert_eq!(inv.rebuilt_defs, vec!["d_b", "d_c"]);
    let res = av.resource.as_ref().unwrap();
    assert_eq!(res.profile, "owned");
    assert_eq!(res.max_steps, Some(1000));
    assert_eq!(res.cancel_after_ms, Some(500));
    assert_eq!(res.wall_ms, None);
}

#[test]
fn mn_nested_oracles_decode() {
    let case = one(concat!(
        "### Probe mn\n",
        "recipe: \"ac/mir-native-resource-events-v1\"\n",
        "entry: \"main\"\n",
        "mn-resource: {budget_bytes: 4096, fail_seam: [9, 1], fail_text_nth: 2}\n",
        "declaration: {min_frontend_invalidations: 2, max_codegen_units: 4, stable_defs: [\"d_u\"]}\n",
        "history: {min_cache_hit_coff: 1}\n",
        "turnaround: {p90_max_ms: 2000, min_edits: 3}\n",
    ));
    let mn = case.spec.checks[0].options.probe_mn.as_ref().unwrap();
    let res = mn.resource.as_ref().unwrap();
    assert_eq!(res.budget_bytes, Some(4096));
    assert_eq!(res.fail_seam, Some((9, 1)));
    assert_eq!(res.fail_text_nth, Some(2));
    let dec = mn.declaration.as_ref().unwrap();
    assert_eq!(dec.min_frontend_invalidations, 2);
    assert_eq!(dec.max_codegen_units, Some(4));
    assert_eq!(dec.stable_defs, vec!["d_u"]);
    assert_eq!(mn.history.as_ref().unwrap().min_cache_hit_coff, 1);
    let ta = mn.turnaround.as_ref().unwrap();
    assert_eq!((ta.p90_max_ms, ta.min_edits), (2000, 3));
}

#[test]
fn observation_sections_carry_inline_bindings() {
    let case = one(concat!(
        "### Probe sema-q\n",
        "recipe: \"sema/query-v1\"\n",
        "package: \"p\"\nmodule: \"m\"\nkind: \"infer-body\"\nfn: \"f\"\n",
        "\n#### Observation 1\n",
        "cache: \"published\"\n",
        "compute-calls: 1\n",
        "stages-include: [\"infer\"]\n",
        "\n```json\n",
        "{\"bindings\": []}\n",
        "```\n",
    ));
    let probe = case.spec.checks[0].expect.probe.as_ref().unwrap();
    assert_eq!(probe.len(), 1);
    let st = &probe[0];
    assert_eq!(st.cache.as_deref(), Some("published"));
    assert_eq!(st.compute_calls, Some(1));
    assert_eq!(st.stages_include, vec!["infer"]);
    assert!(st.bindings_text.is_some());
    assert!(st.bindings_file.is_none());
}

#[test]
fn vm_fail_alloc_at_decodes() {
    let case = one("### VM result\nfail-alloc-at: 9\n\n```text\n7\n```\n");
    assert_eq!(case.spec.checks[0].options.avm_fail_alloc_at, Some(9));
    let e = error_of("### VM result\nfail-alloc-at: 0\n\n```text\n7\n```\n");
    assert!(e.message.contains("≥1"), "{e:?}");
}

#[test]
fn nc04_fail_seam_and_nested_shapes_are_validated() {
    // fail_seam must be exactly [seam, nth] integers
    let e = error_of(concat!(
        "### Probe mn\n",
        "recipe: \"ac/mir-native-resource-events-v1\"\n",
        "entry: \"main\"\n",
        "mn-resource: {fail_seam: [9]}\n",
    ));
    assert!(e.message.contains("fail_seam"), "{e:?}");

    // unknown nested keys rejected (NC-25-flavored deep admission)
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/owned-run-v1\"\n",
        "entry: \"d_main\"\n",
        "invalidation: {stable_defs: [\"a\"], wat: 1}\n",
    ));
    assert!(e.message.contains("未知"), "{e:?}");

    // family gating: mn oracle on av recipe rejected
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/owned-run-v1\"\n",
        "entry: \"d_main\"\n",
        "history: {min_cache_hit_coff: 1}\n",
    ));
    assert!(e.message.contains("仅属于 mn"), "{e:?}");
}

#[test]
fn nc12_nc13_step_shapes_are_validated() {
    // edit-source needs both from and to
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/data-incremental-v1\"\n",
        "entry: \"d_main\"\n",
        "steps: [{op: \"edit-source\", from: \"a\"}]\n",
    ));
    assert!(e.message.contains("from/to"), "{e:?}");

    // partial-rebuild needs defs
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/data-incremental-v1\"\n",
        "entry: \"d_main\"\n",
        "steps: [{op: \"partial-rebuild\", from: \"a\", to: \"b\"}]\n",
    ));
    assert!(e.message.contains("defs"), "{e:?}");

    // resident-slice needs slices
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/resident-run-v1\"\n",
        "entry: \"d_main\"\n",
        "steps: [{op: \"resident-slice\"}]\n",
    ));
    assert!(e.message.contains("slices"), "{e:?}");

    // unknown op rejected
    let e = error_of(concat!(
        "### Probe av\n",
        "recipe: \"ac-av/owned-run-v1\"\n",
        "entry: \"d_main\"\n",
        "steps: [{op: \"transmute\"}]\n",
    ));
    assert!(e.message.contains("未知"), "{e:?}");
}

#[test]
fn nc16_observation_order_and_cache_values_validated() {
    let e = error_of(concat!(
        "### Probe cte-q\n",
        "recipe: \"cte/scalar-query-v1\"\n",
        "entry: \"main\"\n",
        "\n#### Observation 2\n",
        "cache: \"hit\"\n",
    ));
    assert!(e.message.contains("连续"), "{e:?}");

    let e = error_of(concat!(
        "### Probe cte-q\n",
        "recipe: \"cte/scalar-query-v1\"\n",
        "entry: \"main\"\n",
        "\n#### Observation 1\n",
        "cache: \"maybe\"\n",
    ));
    assert!(e.message.contains("hit/miss/stale/published"), "{e:?}");
}

#[test]
fn probe_error_faces_still_decode_with_nested_oracles() {
    let case = one(concat!(
        "### Probe mn-diag\n",
        "recipe: \"ac/mir-native-scalar-v1\"\n",
        "entry: \"main\"\n",
        "diagnostics: \"source-v1\"\n",
        "stage: \"run\"\n",
        "code: \"mn.trap\"\n",
        "message-contains: \"boom\"\n",
        "mn-resource: {fail_nth: 3}\n",
    ));
    let check = &case.spec.checks[0];
    assert_eq!(
        check.options.probe_mn.as_ref().unwrap().resource.as_ref().unwrap().fail_nth,
        Some(3)
    );
    match &check.expect.completion {
        Completion::Error {
            stage: Stage::Run,
            code,
            message: Some(MessageMatcher::Contains(t)),
        } => {
            assert_eq!(code.as_deref(), Some("mn.trap"));
            assert_eq!(t, "boom");
        }
        other => panic!("error completion expected, got {other:?}"),
    }
}
