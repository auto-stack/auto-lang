//! PLAN-1078 (user ruling): the console-family probe recipes
//! (`ac-av/console-run-v1`, `ac-av/scheduler-source-run-v1`,
//! `ac-av/language-source-run-v1`) default an OMITTED stdout or stderr
//! section to an expected zero-byte stream at LOAD TIME, then the completed
//! oracle goes through the existing validation. Explicit streams compare as
//! written; every other check family keeps its original omission semantics.

use auto_test_case::admission::Limits;
use auto_test_case::author::{decode_document, AuthorError};
use auto_test_case::contract::{Compare, Completion};
use auto_test_case::document::CaseDocument;

fn decode(src: &str, rel: &str) -> Result<Vec<auto_test_case::author::AuthorCase>, AuthorError> {
    let doc = CaseDocument::parse(src.as_bytes()).unwrap();
    decode_document(&doc, rel, &Limits::defaults())
}

const AUTO: &str = "### Auto\n\n```auto\nfn main() int { return 1 }\n```\n";

fn case_doc(body: &str) -> String {
    format!("## Case: a\n\n{AUTO}{body}")
}

#[test]
fn omitted_dual_streams_default_to_zero_bytes_for_language_source() {
    // Both stdout and stderr paragraphs are omitted; the loader defaults
    // each to an expected zero-byte byte-exact stream (success completion).
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/language-source-run-v1\"\nentry: \"d_main\"\nexit: 1\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert_eq!(check.adapter, auto_test_case::contract::Adapter::Probe);
    let out = check.expect.stdout.as_ref().expect("stdout defaulted");
    let err = check.expect.stderr.as_ref().expect("stderr defaulted");
    assert_eq!(out.compare, Compare::Bytes);
    assert_eq!(out.text, "");
    assert_eq!(err.compare, Compare::Bytes);
    assert_eq!(err.text, "");
    assert!(matches!(check.expect.completion, Completion::Success));
}

#[test]
fn omitted_single_stream_defaults_while_the_other_stays_explicit() {
    // stderr is an explicit empty fence; stdout is omitted -> zero default.
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/language-source-run-v1\"\nentry: \"d_main\"\nexit: 1\n\n#### stderr\n\n```text\n```\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert_eq!(check.expect.stdout.as_ref().unwrap().text, "");
    assert_eq!(check.expect.stderr.as_ref().unwrap().text, "");
}

#[test]
fn explicit_empty_fence_stays_explicit() {
    // Both streams explicitly present as empty fences — kept as written.
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/language-source-run-v1\"\nentry: \"d_main\"\nexit: 1\n\n#### stdout\n\n```text\n```\n\n#### stderr\n\n```text\n```\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert!(check.expect.stdout.is_some());
    assert!(check.expect.stderr.is_some());
}

#[test]
fn diagnostic_completion_gets_stream_defaults_too() {
    // A parse-stage diagnostic with both streams omitted still defaults
    // both streams to zero bytes (user ruling: success and diagnostic
    // completions both apply).
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/language-source-run-v1\"\nentry: \"d_main\"\nstage: \"parse\"\ncode: \"source.syntax\"\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert!(matches!(
        check.expect.completion,
        Completion::Error { .. }
    ));
    assert_eq!(check.expect.stdout.as_ref().unwrap().text, "");
    assert_eq!(check.expect.stderr.as_ref().unwrap().text, "");
}

#[test]
fn non_console_probe_keeps_original_omission_semantics() {
    // ac-av/core-run-v1 is NOT a dual-stream console family: omitted
    // streams stay None (the case gate rejects streams there).
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/core-run-v1\"\nentry: \"d_main\"\nexit: 1\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert!(check.expect.stdout.is_none());
    assert!(check.expect.stderr.is_none());
}

#[test]
fn scheduler_source_defaults_streams_like_the_console_family() {
    let src = case_doc(
        "### Probe av\nrecipe: \"ac-av/scheduler-source-run-v1\"\nentry: \"d_main\"\nexit: 0\n",
    );
    let cases = decode(&src, "x/y.case.ad").unwrap();
    let check = &cases[0].spec.checks[0];
    assert_eq!(check.expect.stdout.as_ref().unwrap().text, "");
    assert_eq!(check.expect.stderr.as_ref().unwrap().text, "");
}
