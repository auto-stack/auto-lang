//! PLAN-796 T-06: `probe` adapter integration through the REAL runner —
//! the sema worker is spawned per session, `restart` steps prove REAL
//! cross-process persistence inside the unified runner, and every
//! judgement (cache outcome, compute counter, bindings oracle, stage
//! trace) happens parent-side. Protocol-level negatives live here too:
//! unregistered recipes, v1 incompatibility, and the recipe×source
//! admission matrix (V-08).

#![cfg(windows)]

use std::path::{Path, PathBuf};

use auto_test_case::admission::Limits;
use auto_test_case::report::Status;

use crate::test_cases::run_suite;

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-probe-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn out_of(root: &Path, sub: &str) -> PathBuf {
    root.parent().unwrap_or(Path::new(".")).join(format!(
        "{}-{}",
        root.file_name().unwrap().to_string_lossy(),
        sub
    ))
}

/// Serializes every test that spawns the real sema worker: fault
/// injection is process-global state, so worker-spawning tests must not
/// interleave with it (or with each other's fault windows).
pub(crate) static FAULT_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn status_of(report: &auto_test_case::report::RunReport, check: &str) -> Status {
    report
        .results
        .iter()
        .find(|r| r.check_id == check)
        .unwrap_or_else(|| panic!("check {} present in {:?}", check, report.results))
        .status
}

/// V-08: the committed ac-semantics corpus runs END TO END through the
/// unified runner — real worker processes, real store on disk, real
/// restart boundary, parent-side oracle judgement.
#[test]
fn probe_adapter_corpus_executes_green() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ac-semantics");
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &suite_dir.join("ac-semantics.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("ac-semantics suite runs");
    for check in ["bindings", "roundtrip", "callee-sig", "stages", "parse"] {
        assert_eq!(
            status_of(&report, check),
            Status::Pass,
            "check {check} in {:#?}",
            report
                .results
                .iter()
                .map(|r| (r.check_id.clone(), r.status, r.detail.clone()))
                .collect::<Vec<_>>()
        );
    }
}

/// A NEGATIVE oracle flips the verdict: the runner is a real judge, not
/// a green stamp — expecting compute_calls 1 on a restart-hit step must
/// FAIL (the worker really served it from the store without computing).
#[test]
fn probe_adapter_negative_oracle_fails() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let root = temp_root("neg");
    std::fs::write(
        root.join("neg.suite.ad"),
        r#"# neg suite
```test-suite
suite { schema: "autolang.suite/2", id: "neg", include: ["neg.case.ad"] }
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("neg.case.ad"),
        r#"# neg case
```test-case
case {
  schema: "autolang.case/2",
  id: "neg.case",
  source: { text: "fn main() int { return 1 + 1 }", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "wrong-compute", adapter: "probe", action: "run",
      options: {
        recipe: "sema/query-v1",
        sema: { package: "neg", module: "neg", kind: "infer-body", fn: "main" },
        steps: [ { op: "query" }, { op: "restart" }, { op: "query" } ]
      },
      expect: { completion: "success", probe: [
        { cache: "published", compute_calls: 1 },
        { cache: "hit", compute_calls: 1 }
      ] } }
  ]
}
```
"#,
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report =
        run_suite(&root.join("neg.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
    assert_eq!(status_of(&report, "wrong-compute"), Status::Fail);
}

/// Admission negatives at DECODE: unregistered recipes and probe fields
/// in a v1 document are located errors (v1 accept/reject stays
/// byte-identical — the compat guarantee).
#[test]
fn probe_adapter_decode_negatives() {
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;
    let decode_case_doc = |doc: &str| -> Result<
        auto_test_case::contract::CaseSpec,
        auto_test_case::contract::ContractError,
    > {
        let v = parse_contract(doc).unwrap();
        decode_case(&v, &Limits::defaults())
    };
    let base = |schema: &str, adapter_line: &str| {
        format!(
            r#"case {{
  schema: "{schema}",
  id: "neg.decode",
  source: {{ text: "fn main() int {{ return 1 }}", syntax: "auto", path: "main.at" }},
  entry: {{ mode: "program" }},
  checks: [
    {{ id: "c", {adapter_line}
      expect: {{ completion: "success" }} }}
  ]
}}
"#
        )
    };
    let doc = base(
        "autolang.case/2",
        r#"adapter: "probe", action: "run",
      options: { recipe: "my-shell-exploit/v9", steps: [ { op: "query" } ] }"#,
    );
    let err: auto_test_case::contract::ContractError =
        decode_case_doc(&doc).expect_err("unregistered recipe must be rejected");
    assert!(format!("{err}").contains("not registered"), "{err}");

    // v1 + probe = rejected (v1 never grows adapters)
    let doc = base(
        "autolang.case/1",
        r#"adapter: "probe", action: "run",
      options: { recipe: "sema/query-v1", steps: [ { op: "query" } ] }"#,
    );
    let err: auto_test_case::contract::ContractError =
        decode_case_doc(&doc).expect_err("probe in v1 must be rejected");
    assert!(format!("{err}").contains("probe"), "{err}");

    // steps missing → decode error
    let doc = base(
        "autolang.case/2",
        r#"adapter: "probe", action: "run",
      options: { recipe: "sema/query-v1" }"#,
    );
    assert!(decode_case_doc(&doc).is_err());

    // unknown step op → decode error
    let doc = base(
        "autolang.case/2",
        r#"adapter: "probe", action: "run",
      options: { recipe: "sema/query-v1", steps: [ { op: "shell" } ] }"#,
    );
    let err: auto_test_case::contract::ContractError =
        decode_case_doc(&doc).expect_err("shell op must be rejected");
    assert!(format!("{err}").contains("unknown probe op"), "{err}");
}

/// The seam's atom-data source kind decodes (v2) and is REJECTED by the
/// sema recipe at execution admission (the kind is registered for other
/// recipes; sema/query-v1 consumes .at sources).
#[test]
fn probe_adapter_atom_data_kind_admission() {
    use auto_test_case::atom::parse_contract;
    use auto_test_case::contract::decode_case;
    let decode_case_doc = |doc: &str| -> Result<
        auto_test_case::contract::CaseSpec,
        auto_test_case::contract::ContractError,
    > {
        let v = parse_contract(doc).unwrap();
        decode_case(&v, &Limits::defaults())
    };
    let doc_case = r#"case {
  schema: "autolang.case/2",
  id: "atom.decode",
  source: { text: "bind x 1", syntax: "atom-data", path: "input.atomdata" },
  entry: { mode: "program" },
  checks: [
    { id: "c", adapter: "probe", action: "run",
      options: {
        recipe: "sema/query-v1",
        sema: { package: "p", module: "m", kind: "infer-body", fn: "main" },
        steps: [ { op: "query" } ]
      },
      expect: { completion: "success", probe: [ { cache: "miss" } ] } }
  ]
}
"#;
    // decodes fine — the kind is registered with the seam
    decode_case_doc(doc_case).expect("atom-data source decodes");

    // but executing it through sema/query-v1 is an executor admission
    // error (recipe consumes .at sources)
    let root = temp_root("atom");
    std::fs::write(
        root.join("atom.suite.ad"),
        r#"# atom suite
```test-suite
suite { schema: "autolang.suite/2", id: "atom", include: ["atom.case.ad"] }
```
"#,
    )
    .unwrap();
    let fenced = format!(
        "# atom case
```test-case
{doc_case}```
"
    );
    std::fs::write(root.join("atom.case.ad"), fenced).unwrap();
    let out = out_of(&root, "out");
    let report =
        run_suite(&root.join("atom.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
    assert_eq!(status_of(&report, "c"), Status::Error);
}

/// v1/v2 compatibility: the legacy pilot suite still runs green with the
/// probe seam present (the decoder change is a strict v2 superset; the
/// v1 path is untouched).
#[test]
fn probe_adapter_v1_pilot_untouched() {
    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/pilot");
    let root = temp_root("v1");
    let out = out_of(&root, "out");
    let report = run_suite(&suite_dir.join("pilot.suite.ad"), &out, &Limits::defaults())
        .expect("pilot suite still runs");
    assert!(!report.results.is_empty());
    for r in &report.results {
        assert_ne!(
            r.status,
            Status::Error,
            "pilot check {} errored",
            r.check_id
        );
    }
}

// ---- F-796-R4-02/03/04 permanent controls ---------------------------------

/// Build a one-case suite where `check_tail` is the full check body
/// after `id:` (adapter/options/expect). The source embeds verbatim.
fn run_fault_case(tag: &str, source: &str, check_tail: &str) -> auto_test_case::report::RunReport {
    run_fault_case_at(tag, source, check_tail).0
}

/// The same case, also returning the suite path (stale adjudication
/// re-reads the suite from disk).
fn run_fault_case_at(
    tag: &str,
    source: &str,
    check_tail: &str,
) -> (auto_test_case::report::RunReport, PathBuf) {
    let root = temp_root(tag);
    let doc = format!(
        "# fault suite\n```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"fault\", include: [\"fault.case.ad\"] }}\n```\n\n```test-case\ncase {{ schema: \"autolang.case/2\", id: \"fault.case\",\n  source: {{ text: \"{src}\", syntax: \"auto\", path: \"main.at\" }},\n  entry: {{ mode: \"program\" }},\n  checks: [ {{ id: \"c\", {tail} }} ] }}\n```\n",
        src = source.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n"),
        tail = check_tail,
    );
    std::fs::write(root.join("fault.suite.ad"), doc.clone()).unwrap();
    std::fs::write(root.join("fault.case.ad"), doc).unwrap();
    let out = out_of(&root, "out");
    let report =
        run_suite(&root.join("fault.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
    (report, root.join("fault.suite.ad"))
}

const QUERY_C: &str = "adapter: \"probe\", action: \"run\", \
options: { recipe: \"sema/query-v1\", \
sema: { package: \"p\", module: \"m\", kind: \"infer-body\", fn: \"main\" }, \
steps: [ { op: \"query\" } ] }";

const QUERY_MISSING_C: &str = "adapter: \"probe\", action: \"run\", \
options: { recipe: \"sema/query-v1\", \
sema: { package: \"p\", module: \"m\", kind: \"infer-body\", fn: \"missing\" }, \
steps: [ { op: \"query\" } ] }";

/// F-796-R4-02: a REAL type error (`return true` from an int fn) with
/// a SUCCESS expectation must FAIL — never PASS on a cache string.
#[test]
fn probe_adapter_type_error_with_success_expect_fails() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let report = run_fault_case(
        "te-succ",
        "fn main() int { return true }",
        &format!(
            "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ compute_calls: 1 }} ] }} "
        ),
    );
    assert_eq!(status_of(&report, "c"), Status::Fail);
}

/// F-796-R4-02: the same type error with the DECLARED error completion
/// (typecheck/type-mismatch) passes — language negatives are judged,
/// distinguishable from infra errors.
#[test]
fn probe_adapter_type_error_with_error_expect_passes() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let report = run_fault_case(
        "te-err",
        "fn main() int { return true }",
        &format!(
            "{QUERY_C}, expect: {{ completion: {{ stage: \"typecheck\", code: \"type-mismatch\" }}, probe: [ {{ cache: \"diags\" }} ] }} "
        ),
    );
    assert_eq!(status_of(&report, "c"), Status::Pass);
}

/// F-796-R4-02: querying a missing function is a resolve-stage error
/// (unknown-name), declarable and judged.
#[test]
fn probe_adapter_unknown_fn_is_error_completion() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let report = run_fault_case(
        "unk",
        "fn main() int { return 1 }",
        &format!(
            "{QUERY_MISSING_C}, expect: {{ completion: {{ stage: \"resolve\", code: \"unknown-name\" }}, probe: [ {{ cache: \"diags\" }} ] }} "
        ),
    );
    assert_eq!(status_of(&report, "c"), Status::Pass);
}

fn set_faults(envs: Vec<(String, String)>, wall_ms: Option<u64>) {
    crate::test_cases::sema::TEST_FAULTS
        .write()
        .unwrap()
        .replace(crate::test_cases::sema::TestFaults {
            child_envs: envs,
            session_wall_ms: wall_ms,
        });
}

fn clear_faults() {
    crate::test_cases::sema::TEST_FAULTS.write().unwrap().take();
}

/// F-796-R4-03: a worker that responds correctly and then SLEEPS past
/// the (test-shortened) parent wall is killed and the check is an
/// infrastructure ERROR — not a pass seconds later.
#[test]
fn probe_adapter_worker_hang_after_response_is_infra_error() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    set_faults(
        vec![("SEMA_TEST_HANG_AFTER_RESPONSE_SECS".into(), "30".into())],
        Some(3_000),
    );
    let result = std::panic::catch_unwind(|| {
        let report = run_fault_case(
            "hang",
            "fn main() int { return 1 }",
            &format!(
                "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
            ),
        );
        status_of(&report, "c")
    });
    clear_faults();
    assert_eq!(result.expect("suite ran"), Status::Error);
}

/// F-796-R4-04: a worker that exits 7 after a correct response is an
/// infrastructure ERROR, never a pass.
#[test]
fn probe_adapter_worker_bad_exit_is_infra_error() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    set_faults(
        vec![("SEMA_TEST_EXIT_AFTER_RESPONSE".into(), "7".into())],
        None,
    );
    let result = std::panic::catch_unwind(|| {
        let report = run_fault_case(
            "exit7",
            "fn main() int { return 1 }",
            &format!(
                "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
            ),
        );
        status_of(&report, "c")
    });
    clear_faults();
    assert_eq!(result.expect("suite ran"), Status::Error);
}

/// F-796-R4-04: a worker reporting a WRONG version is admitted against
/// the mirrored expectation and rejected as infra.
#[test]
fn probe_adapter_worker_bad_version_is_infra_error() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    set_faults(
        vec![("SEMA_TEST_FAKE_VERSION".into(), "INVALID".into())],
        None,
    );
    let result = std::panic::catch_unwind(|| {
        let report = run_fault_case(
            "badver",
            "fn main() int { return 1 }",
            &format!(
                "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
            ),
        );
        status_of(&report, "c")
    });
    clear_faults();
    assert_eq!(result.expect("suite ran"), Status::Error);
}

/// F-796-R6-07: an invalid source is a TYPED language parse negative —
/// the parent judges it against the declared `parse`/`source.syntax`
/// completion and PASSES; it is never an infrastructure ERROR.
#[test]
fn probe_adapter_parse_error_is_language_negative() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let report = run_fault_case(
        "parse-neg",
        "fn main() int { return 3",
        &format!(
            "{QUERY_C}, expect: {{ completion: {{ stage: \"parse\", code: \"source.syntax\" }}, probe: [ {{ }} ] }} "
        ),
    );
    assert_eq!(
        status_of(&report, "c"),
        Status::Pass,
        "typed parse diagnostics are language verdicts: {:#?}",
        report
            .results
            .iter()
            .map(|r| (r.status, r.detail.clone()))
            .collect::<Vec<_>>()
    );
}

/// F-796-R6-03: a correct response followed by a descendant that
/// INHERITS stdout/stderr and outlives the worker must not wedge the
/// parent. Worker-side self-containment is DISABLED here — the parent's
/// own Job containment must reap the tree and return promptly.
#[test]
fn probe_adapter_descendant_held_pipes_bounded_by_parent_containment() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    set_faults(
        vec![
            ("SEMA_TEST_SPAWN_LINGER_SECS".into(), "25".into()),
            ("SEMA_TEST_NO_SELF_CONTAINMENT".into(), "1".into()),
        ],
        None,
    );
    let started = std::time::Instant::now();
    let result = std::panic::catch_unwind(|| {
        let report = run_fault_case(
            "linger",
            "fn main() int { return 1 }",
            &format!(
                "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
            ),
        );
        status_of(&report, "c")
    });
    let elapsed = started.elapsed();
    clear_faults();
    assert!(
        elapsed < std::time::Duration::from_secs(15),
        "the parent must not wait for the descendant's inherited pipes (took {elapsed:?})"
    );
    assert_eq!(
        result.expect("suite ran"),
        Status::Pass,
        "the correct response stays valid once the tree is contained"
    );
}

/// F-796-R6-03: the same lifetime with the worker's OWN self-containment
/// active (the production path) — still bounded and green.
#[test]
fn probe_adapter_descendant_reaped_by_worker_self_containment() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    set_faults(
        vec![("SEMA_TEST_SPAWN_LINGER_SECS".into(), "25".into())],
        None,
    );
    let started = std::time::Instant::now();
    let result = std::panic::catch_unwind(|| {
        let report = run_fault_case(
            "linger-self",
            "fn main() int { return 1 }",
            &format!(
                "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
            ),
        );
        status_of(&report, "c")
    });
    let elapsed = started.elapsed();
    clear_faults();
    assert!(
        elapsed < std::time::Duration::from_secs(15),
        "worker self-containment must reap the descendant (took {elapsed:?})"
    );
    assert_eq!(result.expect("suite ran"), Status::Pass);
}

/// F-796-R6-04: a REAL, unchanged probe report is FRESH (the previous
/// implementation was unconditionally STALE: missing worker-version and
/// two unjustified extra axes). A swapped worker version is STALE.
#[test]
fn probe_adapter_stale_unchanged_report_fresh_and_swap_stale() {
    let _guard = FAULT_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let (report, suite) = run_fault_case_at(
        "stale-fresh",
        "fn main() int { return 1 }",
        &format!(
            "{QUERY_C}, expect: {{ completion: \"success\", probe: [ {{ cache: \"published\" }} ] }} "
        ),
    );
    assert_eq!(status_of(&report, "c"), Status::Pass);
    let json: serde_json::Value = serde_json::from_str(&report.to_json()).expect("report is JSON");
    let same = crate::test_cases::sema::current_toolchain_identity();
    let verdicts =
        crate::test_cases::stale::check_with_families(&suite, &json, None, Some(same.clone()))
            .expect("stale reads the report");
    assert!(
        verdicts
            .iter()
            .all(|v| matches!(v, crate::test_cases::stale::RowVerdict::Fresh { .. })),
        "an unchanged report must be FRESH: {verdicts:?}"
    );
    let mut drifted = same;
    drifted.insert("sema-worker-version".into(), "ac-sema-probe/9.9.9".into());
    let verdicts =
        crate::test_cases::stale::check_with_families(&suite, &json, None, Some(drifted))
            .expect("stale reads the report again");
    assert!(
        verdicts.iter().any(|v| matches!(
            v,
            crate::test_cases::stale::RowVerdict::Stale { drifted_axes, .. }
                if drifted_axes.iter().any(|a| a.contains("sema-worker-version"))
        )),
        "a swapped worker version must be STALE: {verdicts:?}"
    );
}

/// F-796-R6-04: a MIXED ac+probe report resolves each family's axes
/// against its OWN current identity map (one shared lazy map was the
/// cross-family contamination risk).
#[test]
fn probe_adapter_mixed_report_uses_per_family_toolchain_identity() {
    use auto_test_case::admission::Limits;
    let root = temp_root("mixed-stale");
    std::fs::write(
        root.join("mixed.suite.ad"),
        r#"# mixed family suite
```test-suite
suite { schema: "autolang.suite/2", id: "mixed", include: ["mixed.case.ad"] }
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("mixed.case.ad"),
        r#"# mixed family case
```test-case
case {
  schema: "autolang.case/2",
  id: "mixed",
  source: { text: "fn main() int { return 1 }", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "ac1", adapter: "ac", action: "run", expect: { completion: "success" } },
    { id: "p1", adapter: "probe", action: "run",
      options: { recipe: "sema/query-v1",
        sema: { package: "mixed", module: "m", kind: "infer-body", fn: "main" },
        steps: [ { op: "query" } ] },
      expect: { completion: "success", probe: [ { cache: "published" } ] } }
  ]
}
```
"#,
    )
    .unwrap();
    let suite = root.join("mixed.suite.ad");
    let limits = Limits::defaults();
    let idents =
        crate::test_cases::stale::current_identities(&suite, &limits).expect("identities resolve");
    let identity_json = |key: &str| {
        let h = &idents.get(key).expect("check declared").0;
        serde_json::json!({
            "suite": h.suite.clone(), "case_file": h.case_file.clone(), "source": h.source.clone(),
            "oracle": h.oracle.clone(), "contract": h.contract.clone(),
        })
    };
    let ac_axes: std::collections::BTreeMap<String, String> = [
        ("worker-exe-sha256".to_string(), "ac-hash-1".to_string()),
        ("ac-worker-protocol".to_string(), "acworker/1".to_string()),
    ]
    .into_iter()
    .collect();
    let probe_axes: std::collections::BTreeMap<String, String> = [
        (
            "sema-worker-exe-sha256".to_string(),
            "sema-hash-1".to_string(),
        ),
        (
            "sema-worker-version".to_string(),
            "ac-sema-probe/0.1.0".to_string(),
        ),
        ("sema-protocol".to_string(), "semaworker/1".to_string()),
        ("recipe".to_string(), "sema/query-v1".to_string()),
    ]
    .into_iter()
    .collect();
    let row = |check: &str, adapter: &str, axes: &[(&str, &str)]| {
        serde_json::json!({
            "case": "mixed",
            "check": check,
            "adapter": adapter,
            "identity": identity_json(&format!("mixed:{check}")),
            "native": { "toolchain": axes },
        })
    };
    let report = serde_json::json!({
        "toolchain": crate::test_cases::stale::toolchain_axes(&limits),
        "results": [
            row("ac1", "ac", &[("worker-exe-sha256", "ac-hash-1"), ("ac-worker-protocol", "acworker/1")]),
            row("p1", "probe", &[
                ("sema-worker-exe-sha256", "sema-hash-1"),
                ("sema-worker-version", "ac-sema-probe/0.1.0"),
                ("sema-protocol", "semaworker/1"),
                ("recipe", "sema/query-v1"),
            ]),
        ],
    });
    let verdicts = crate::test_cases::stale::check_with_families(
        &suite,
        &report,
        Some(ac_axes.clone()),
        Some(probe_axes.clone()),
    )
    .expect("mixed report adjudicates");
    assert!(
        verdicts
            .iter()
            .all(|v| matches!(v, crate::test_cases::stale::RowVerdict::Fresh { .. })),
        "both families are fresh with their own identities: {verdicts:?}"
    );
    let mut bad_probe = probe_axes;
    bad_probe.insert("sema-worker-exe-sha256".into(), "sema-hash-DRIFT".into());
    let verdicts = crate::test_cases::stale::check_with_families(
        &suite,
        &report,
        Some(ac_axes),
        Some(bad_probe),
    )
    .expect("mixed report adjudicates");
    let stale: Vec<_> = verdicts
        .iter()
        .filter(|v| matches!(v, crate::test_cases::stale::RowVerdict::Stale { .. }))
        .collect();
    assert_eq!(stale.len(), 1, "only the probe row drifts: {verdicts:?}");
    assert!(
        matches!(
            stale[0],
            crate::test_cases::stale::RowVerdict::Stale { check, .. } if check == "p1"
        ),
        "{verdicts:?}"
    );
}
