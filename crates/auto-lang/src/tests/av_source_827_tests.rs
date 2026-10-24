//! PLAN-827: the av-source-827 corpus — REAL Auto source files (byte-
//! identical copies of the AP seeds, identity receipts in the corpus
//! README) through the formal `ac-av/core-run-v1` recipe, judged
//! PARENT-SIDE by the unified runner. Mirrors the ac-av corpus gate; the
//! recipe/worker/protocol are reused unchanged (T-01 reuse ruling — no
//! new recipe, worker command, or workspace).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

pub(crate) static AV_SOURCE_827_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("atc-av-src-827-{}-{}", tag, std::process::id()));
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

/// V-01/V-10 first case (formal path): the av-source-827 corpus runs END
/// TO END through the unified runner — a real av worker process, real
/// store on disk, parent-side oracle judgement. The suite's checked
/// population is asserted (>= 1 check now; grows with T-04/T-05 cases —
/// an empty or shrunken suite can never pass this gate silently).
#[test]
fn av_source_827_corpus_green() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let suite_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../auto-lang/test/cases/av-source-827");
    let root = temp_root("corpus");
    let out = out_of(&root, "out");
    let report = run_suite(
        &suite_dir.join("av-source-827.suite.ad"),
        &out,
        &Limits::defaults(),
    )
    .expect("av-source-827 suite runs");
    let failing: Vec<String> = report
        .results
        .iter()
        .filter(|r| r.status != auto_test_case::report::Status::Pass)
        .map(|r| {
            format!(
                "{}: {:?} {} | mismatches: {:?}",
                r.check_id,
                r.status,
                r.detail.clone().unwrap_or_default(),
                r.mismatches
            )
        })
        .collect();
    assert!(
        failing.is_empty(),
        "av-source-827 corpus all green ({} results): {:#?}",
        report.results.len(),
        failing
    );
    assert!(
        !report.results.is_empty(),
        "an empty corpus is not a pass — the suite must carry its checks"
    );
    assert!(
        report.load_errors.is_empty(),
        "every included case must load — load errors: {:#?}",
        report.load_errors
    );
    let ids: Vec<&str> = report
        .results
        .iter()
        .map(|r| r.check_id.as_str())
        .collect();
    assert!(
        ids.contains(&"run-first"),
        "the first-case check must be present: {ids:?}"
    );
    // T-04 population: the semantics + unsupported/admission faces are part
    // of the required suite population (a silently shrunken suite is red).
    // (The wrong-entry face is NOT here: the probe adapter's declarative
    // stage vocabulary has no verify face — it is asserted with its exact
    // stage/code in the probe-side `av_source_827_semantics` tests.)
    for required in [
        "run-first",
        "run-binop-args",
        "run-trap-div-zero",
        "run-unsupported-print",
        "run-unsupported-list",
        "warm-run",
        "overlay-edit-run",
    ] {
        assert!(ids.contains(&required), "required check {required}: {ids:?}");
    }
}

/// V-04 admission negatives at the RUNNER level (crafted temp suites): an
/// invalid-UTF-8 source is an INFRA/resolve failure — never a guest trap
/// and never a green language verdict; a missing source file is a resolve
/// error naming the path.
#[test]
fn av_source_827_admission_invalid_utf8_is_infra_not_trap() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("utf8");
    // A source file with an invalid UTF-8 byte: resolution carries BYTES
    // (identity is content-based); the worker rejects the encoding, which
    // the parent admits as an INFRA outcome — distinct from any trap.
    std::fs::write(root.join("main.at"), b"fn main() int {\n    return \xff\xfe\n}\n").unwrap();
    std::fs::write(
        root.join("u.suite.ad"),
        r#"# u suite
```test-suite
suite { schema: "autolang.suite/2", id: "u827", include: ["u.case.ad"] }
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("u.case.ad"),
        r#"# u case
```test-case
case {
  schema: "autolang.case/2",
  id: "u827.bad-utf8",
  source: { file: "main.at", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "run-bad-utf8", adapter: "probe", action: "run",
      options: {
        recipe: "ac-av/core-run-v1",
        av: { entry: "d_main" },
        steps: [ { op: "run" } ]
      },
      expect: { completion: "success", exit: 0 } }
  ]
}
```
"#,
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report = run_suite(&root.join("u.suite.ad"), &out, &Limits::defaults())
        .expect("suite runs (resolution is bytes-based)");
    let row = report
        .results
        .iter()
        .find(|r| r.check_id == "run-bad-utf8")
        .expect("check present");
    assert_eq!(
        row.status,
        auto_test_case::report::Status::Error,
        "invalid UTF-8 is infra, not a trap and not a green verdict: {:?} {:?}",
        row.detail,
        row.mismatches
    );
}

/// V-04: a missing source sidecar is a resolve-stage admission error
/// naming the file — never a silent skip, never a guest outcome.
#[test]
fn av_source_827_admission_missing_file_is_resolve_error() {
    let root = temp_root("missing");
    std::fs::write(
        root.join("m.suite.ad"),
        r#"# m suite
```test-suite
suite { schema: "autolang.suite/2", id: "m827", include: ["m.case.ad"] }
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("m.case.ad"),
        r#"# m case
```test-case
case {
  schema: "autolang.case/2",
  id: "m827.missing",
  source: { file: "not-there.at", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "run-missing", adapter: "probe", action: "run",
      options: {
        recipe: "ac-av/core-run-v1",
        av: { entry: "d_main" },
        steps: [ { op: "run" } ]
      },
      expect: { completion: "success", exit: 0 } }
  ]
}
```
"#,
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report = run_suite(&root.join("m.suite.ad"), &out, &Limits::defaults())
        .expect("the suite runs; the missing sidecar is a per-check outcome");
    let row = report
        .results
        .iter()
        .find(|r| r.check_id == "run-missing")
        .expect("check present");
    assert_eq!(
        row.status,
        auto_test_case::report::Status::Error,
        "a missing sidecar is an admission/infra error, never a skip or a verdict"
    );
    let detail = row.detail.clone().unwrap_or_default();
    assert!(
        detail.contains("not-there.at") && detail.contains("payload resolution failed"),
        "error names the missing file and the resolution stage: {detail}"
    );
}

/// V-04: an out-of-root `source.file` (`../` escape, absolute path, or a
/// drive form) is rejected at DECODE — the payload domain is the case
/// directory; payloads cannot reach across trees. (Backslash forms are
/// structurally unreachable: `validate_virtual_path` rejects any `\`, so
/// only forward-slash forms can even express an escape attempt.)
#[test]
fn av_source_827_admission_path_escape_rejected_at_decode() {
    let root = temp_root("escape");
    for (label, file_ref) in [
        ("parent-escape", "../outside.at"),
        ("absolute", "/etc/passwd"),
        ("drive", "C:/windows/win.ini"),
    ] {
        std::fs::write(
            root.join(format!("{label}.case.ad")),
            format!(
                r#"# e case {label}
```test-case
case {{
  schema: "autolang.case/2",
  id: "e827.{label}",
  source: {{ file: "{file_ref}", syntax: "auto", path: "main.at" }},
  entry: {{ mode: "program" }},
  checks: [
    {{ id: "run-escape", adapter: "probe", action: "run",
      options: {{
        recipe: "ac-av/core-run-v1",
        av: {{ entry: "d_main" }},
        steps: [ {{ op: "run" }} ]
      }},
      expect: {{ completion: "success", exit: 0 }} }}
  ]
}}
```
"#
            ),
        )
        .unwrap();
        std::fs::write(
            root.join(format!("{label}.suite.ad")),
            format!(
                r#"# e suite {label}
```test-suite
suite {{ schema: "autolang.suite/2", id: "e827-{label}", include: ["{label}.case.ad"] }}
```
"#
            ),
        )
        .unwrap();
        let out = out_of(&root, &format!("out-{label}"));
        let err = run_suite(
            &root.join(format!("{label}.suite.ad")),
            &out,
            &Limits::defaults(),
        )
        .expect_err("path escapes must be rejected at decode");
        assert!(
            err.to_string().to_lowercase().contains("escape"),
            "{label}: decode error classifies the escape: {err}"
        );
    }
}
