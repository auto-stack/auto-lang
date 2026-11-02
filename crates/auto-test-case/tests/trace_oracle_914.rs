//! PLAN-914: the AV run-step MARK TRACE oracle (`expect.trace`) admission.
//!
//! The trace is the executed `hir.test.trace.v1` event sequence
//! (`step.trace`, e.g. "ba"). It is admitted on the AV probe recipes only
//! (the only recipes whose run steps report it), it is a success-completion
//! observation, its alphabet is the delivered mark set ("a"/"b"), and it
//! participates in the content-bound oracle identity so an edited trace
//! invalidates old PASSes.

use auto_test_case::admission::Limits;
use auto_test_case::atom::parse_contract;
use auto_test_case::contract::decode_case;
use auto_test_case::ir::resolve_oracle;

fn probe_case(recipe: &str, binding: &str, expect: &str) -> String {
    format!(
        "case {{ schema: \"autolang.case/2\", id: \"f914.trace\", \
         source: {{ text: \"fn main() int {{ return 7 }}\", syntax: \"auto\", path: \"main.at\" }}, \
         entry: {{ mode: \"program\" }}, \
         checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
         options: {{ recipe: \"{}\", {}, steps: [ {{ op: \"run\" }} ] }}, \
         expect: {{ {} }} }} ] }}",
        recipe, binding, expect
    )
}

fn av_case(expect: &str) -> String {
    probe_case("ac-av/core-run-v1", "av: { entry: \"d_main\" }", expect)
}

fn mn_case(expect: &str) -> String {
    probe_case(
        "ac/mir-native-scalar-v1",
        "mn: { entry: \"d_main\" }",
        expect,
    )
}

fn decode(src: &str) -> Result<auto_test_case::contract::CaseSpec, auto_test_case::contract::ContractError> {
    let v = parse_contract(src).unwrap();
    decode_case(&v, &Limits::defaults())
}

#[test]
fn trace_oracle_decodes_on_av_recipes() {
    let spec = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"ba\" }",
    ))
    .unwrap();
    let trace = spec.checks[0].expect.trace.as_ref().expect("trace decoded");
    assert_eq!(trace.text, "ba");
    assert_eq!(trace.compare, auto_test_case::contract::Compare::Bytes);

    // The lf-normalized form is the same stream comparator.
    let spec = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"ba\", compare: \"lf\" }",
    ))
    .unwrap();
    assert_eq!(
        spec.checks[0].expect.trace.as_ref().unwrap().compare,
        auto_test_case::contract::Compare::Lf
    );

    // An empty trace is the declared "no mark executed" oracle.
    let spec = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"\" }",
    ))
    .unwrap();
    assert_eq!(spec.checks[0].expect.trace.as_ref().unwrap().text, "");
}

#[test]
fn trace_oracle_is_av_recipe_only() {
    // The MIR-native (ac) recipe never reports a mark trace, so the field
    // must be rejected at admission instead of parsed-then-ignored.
    let err = decode(&mn_case(
        "completion: \"success\", exit: 7, trace: { text: \"ba\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("AV run-step observation"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn trace_oracle_rejects_unknown_event_bytes() {
    let err = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"bx\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("unknown trace event byte"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn trace_oracle_requires_success_completion() {
    let err = decode(&av_case(
        "completion: { stage: \"run\", code: \"run.trap\" }, trace: { text: \"ba\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("require completion"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn trace_oracle_is_part_of_the_oracle_identity() {    let base = std::env::temp_dir();
    let limits = Limits::defaults();
    let spec_ba = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"ba\" }",
    ))
    .unwrap();
    let spec_ab = decode(&av_case(
        "completion: \"success\", exit: 7, trace: { text: \"ab\" }",
    ))
    .unwrap();
    let spec_none = decode(&av_case("completion: \"success\", exit: 7")).unwrap();
    let a = resolve_oracle("f914", &spec_ba.checks[0], &base, &limits).unwrap();
    let b = resolve_oracle("f914", &spec_ab.checks[0], &base, &limits).unwrap();
    let c = resolve_oracle("f914", &spec_none.checks[0], &base, &limits).unwrap();
    assert_ne!(
        a.oracle_sha256, b.oracle_sha256,
        "an edited trace must change the content-bound oracle identity"
    );
    assert_ne!(
        a.oracle_sha256, c.oracle_sha256,
        "declaring a trace must change the oracle identity"
    );
    assert_eq!(a.trace.as_ref().unwrap().text, "ba");
    assert!(c.trace.is_none());
}

// ---- PLAN-914: the MIR-native PE stderr channel (the `ac` scenario path) ----

#[test]
fn mn_stderr_oracle_is_admitted_and_decodes() {
    let spec = decode(&mn_case(
        "completion: \"success\", exit: 7, stderr: { text: \"ba\" }",
    ))
    .unwrap();
    let stderr = spec.checks[0]
        .expect
        .stderr
        .as_ref()
        .expect("stderr decoded");
    assert_eq!(stderr.text, "ba");
    assert_eq!(stderr.compare, auto_test_case::contract::Compare::Bytes);
}

#[test]
fn mn_stdout_stays_console_only() {
    // No M1 consumer needs the mn stdout face: declaring it must stay an
    // admission error instead of a silently-ignored assertion.
    let err = decode(&mn_case(
        "completion: \"success\", exit: 7, stdout: { text: \"ba\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("console-run-v1"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn av_scalar_stderr_stays_console_only() {
    // The AV run-step recipes observe the mark trace, not process streams:
    // stderr on ac-av/core-run-v1 is still rejected (only the console
    // recipe exposes byte streams).
    let err = decode(&av_case(
        "completion: \"success\", exit: 7, stderr: { text: \"ba\" }",
    ))
    .unwrap_err();
    assert!(
        err.message.contains("console-run-v1"),
        "unexpected error: {}",
        err
    );
}

#[test]
fn mn_stderr_is_part_of_the_oracle_identity() {
    let base = std::env::temp_dir();
    let limits = Limits::defaults();
    let spec = decode(&mn_case(
        "completion: \"success\", exit: 7, stderr: { text: \"ba\" }",
    ))
    .unwrap();
    let other = decode(&mn_case(
        "completion: \"success\", exit: 7, stderr: { text: \"ab\" }",
    ))
    .unwrap();
    let a = resolve_oracle("f914", &spec.checks[0], &base, &limits).unwrap();
    let b = resolve_oracle("f914", &other.checks[0], &base, &limits).unwrap();
    assert_ne!(
        a.oracle_sha256, b.oracle_sha256,
        "an edited stderr oracle must change the content-bound identity"
    );
}
