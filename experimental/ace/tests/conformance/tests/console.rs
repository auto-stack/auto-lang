//! Real worker/parent tests: independent byte oracles and fail-closed controls.
use auto_lang_core::test_cases::{
    ac_av, run_suite, stale, ac_cte::EXPECTED_MIR_LOWERING_VERSION,
};
use auto_test_case::{
    admission::Limits,
    discovery::load_suite,
    ir::{resolve_case, resolve_oracle},
    report::Status,
};
use std::path::{Path, PathBuf};

fn corpus() -> PathBuf {
    Path::new(env!("AUTO_LANG_CORE_DIR"))
        .join("../auto-lang/test/cases/ace-m1/console-863/console-863.suite.ad")
}
fn fixture(root: &Path, source: &str, expect: &str) -> PathBuf {
    let dir = root.join("fixture");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("main.at"), source).unwrap();
    let text = r#"```test-case
case { schema: "autolang.case/2", id: "console.control", source: {file: "main.at", syntax: "auto", path: "main.at"}, entry: {mode: "program"},
checks: [{id: "console", adapter: "probe", action: "run", options: {recipe: "ac-av/console-run-v1", av: {entry: "d_main", caps: ["console.stdout", "console.stderr"]}, steps: [{op: "run"}]}, expect: EXPECT}] }
```
"#.replace("EXPECT", expect);
    std::fs::write(dir.join("control.case.ad"), text).unwrap();
    let suite = dir.join("control.suite.ad");
    std::fs::write(&suite, "```test-suite\nsuite {schema: \"autolang.suite/2\", id: \"console-control\", include: [\"control.case.ad\"]}\n```\n").unwrap();
    suite
}
const GOOD: &str = r#"{completion: "success", stdout: {text: "1\n", compare: "bytes"}, stderr: {text: "2\n", compare: "bytes"}, exit: 7}"#;
const SOURCE: &str = "fn main() int {\n print(1)\n eprint(2)\n return 7\n}\n";

#[test]
fn console_real_corpus_and_stale_identity() {
    let temp = tempfile::tempdir().unwrap();
    let suite = corpus();
    let report = run_suite(&suite, &temp.path().join("out"), &Limits::defaults()).unwrap();
    assert!(report.load_errors.is_empty(), "{:?}", report.load_errors);
    assert_eq!(report.results.len(), 15);
    for row in &report.results {
        assert_eq!(
            row.status,
            Status::Pass,
            "{} {:?} {:?}",
            row.case_id,
            row.detail,
            row.mismatches
        );
        let axes: std::collections::BTreeMap<_, _> = row
            .native
            .as_ref()
            .unwrap()
            .toolchain
            .iter()
            .cloned()
            .collect();
        assert_eq!(axes.get("av-profile").map(String::as_str), Some("8"));
        // PLAN-894: the console corpus row carries the SHARED lowering
        // contract version — assert the mirror constant, never a literal
        // (a version bump must not need a fourth site patched by hand).
        assert_eq!(
            axes.get("mir-lowering").map(String::as_str),
            Some(EXPECTED_MIR_LOWERING_VERSION)
        );
    }
    let mut json: serde_json::Value = serde_json::from_str(&report.to_json()).unwrap();
    let fresh = stale::check(&suite, &json).unwrap();
    assert_eq!(fresh.len(), 15);
    assert!(
        fresh
            .iter()
            .all(|r| matches!(r, stale::RowVerdict::Fresh { .. })),
        "{fresh:?}"
    );
    let pairs = json["results"][0]["native"]["toolchain"]
        .as_array_mut()
        .unwrap();
    let axis = pairs.iter_mut().find(|p| p[0] == "av-profile").unwrap();
    axis[1] = "7".into();
    assert!(stale::check(&suite, &json)
        .unwrap()
        .iter()
        .any(|r| matches!(r, stale::RowVerdict::Stale { .. })));
}

#[test]
fn console_three_oracle_axes_fail_independently_and_missing_axes_error() {
    for (old, new, axis) in [
        ("1\\n", "wrong\\n", "stdout"),
        ("2\\n", "wrong\\n", "stderr"),
        ("exit: 7", "exit: 8", "exit"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let suite = fixture(temp.path(), SOURCE, &GOOD.replace(old, new));
        let r = run_suite(&suite, &temp.path().join("out"), &Limits::defaults()).unwrap();
        assert_eq!(r.results.len(), 1, "{:?}", r.load_errors);
        assert_eq!(
            r.results[0].status,
            Status::Fail,
            "{axis}: {:?}",
            r.results[0]
        );
        assert!(
            r.results[0]
                .mismatches
                .iter()
                .any(|m| m.observation.contains(axis)),
            "{axis}: {:?}",
            r.results[0].mismatches
        );
    }
    for field in [
        r#"stdout: {text: "1\n", compare: "bytes"}, "#,
        r#"stderr: {text: "2\n", compare: "bytes"}, "#,
        ", exit: 7",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let bad = GOOD.replace(field, "");
        assert_ne!(bad, GOOD);
        let suite = fixture(temp.path(), SOURCE, &bad);
        match run_suite(&suite, &temp.path().join("out"), &Limits::defaults()) {
            Err(error) => assert!(error.contains("console"), "{error}"),
            Ok(r) => {
                assert!(
                    !r.load_errors.is_empty()
                        || r.results.iter().any(|r| r.status == Status::Error)
                );
                assert!(!r.results.iter().any(|r| r.status == Status::Pass));
            }
        }
    }
}

#[test]
fn console_skip_remains_skip_and_never_creates_a_green_observation() {
    let temp = tempfile::tempdir().unwrap();
    let suite = fixture(temp.path(), SOURCE, GOOD);
    let path = suite.parent().unwrap().join("control.case.ad");
    let text = std::fs::read_to_string(&path).unwrap().replace(
        "entry: {mode: \"program\"}",
        "entry: {mode: \"program\"}, status: {skip: {reason: \"control\", issue: \"PLAN-863\"}}",
    );
    std::fs::write(path, text).unwrap();
    let r = run_suite(&suite, &temp.path().join("out"), &Limits::defaults()).unwrap();
    assert_eq!(r.results.len(), 1);
    assert_eq!(r.results[0].status, Status::Skip);
    assert!(r.results[0].native.is_none());
}

#[test]
fn console_nul_bytes_survive_worker_transport_and_parent_judgement() {
    let temp = tempfile::tempdir().unwrap();
    let suite = fixture(
        temp.path(),
        "fn main() int {\n print(\"a\\0b\")\n eprint(2)\n return 7\n}\n",
        GOOD,
    );
    let limits = Limits::defaults();
    let loaded = load_suite(&suite, &limits).unwrap();
    let case = &loaded.cases[0];
    let resolved = resolve_case(
        case.specs[0].clone(),
        &case.doc,
        &loaded.fixture_base,
        &limits,
    )
    .unwrap();
    let check = &resolved.spec.checks[0];
    let mut oracle =
        resolve_oracle(&resolved.spec.id, check, &loaded.fixture_base, &limits).unwrap();
    oracle.stdout.as_mut().unwrap().text = "a\0b\n".into();
    let project = temp.path().join("project");
    std::fs::create_dir_all(&project).unwrap();
    std::fs::write(project.join("main.at"), &resolved.files[0].bytes).unwrap();
    for (expected, status, name) in [
        ("a\0b\n", Status::Pass, "exact"),
        ("ab\n", Status::Fail, "missing-nul"),
    ] {
        oracle.stdout.as_mut().unwrap().text = expected.into();
        let verdict = ac_av::run_console_check(
            &resolved,
            check,
            &oracle,
            &project,
            &temp.path().join(name),
            temp.path(),
        )
        .unwrap();
        assert_eq!(
            verdict.status, status,
            "{:?} {:?}",
            verdict.detail, verdict.mismatches
        );
    }
}

#[test]
fn console_resource_exhaustion_is_error_with_raw_prefix_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let source = format!(
        "fn main() int {{\n print(1)\n print(\"{}\")\n return 7\n}}\n",
        "x".repeat(65536)
    );
    let suite = fixture(temp.path(), &source, GOOD);
    let r = run_suite(&suite, &temp.path().join("out"), &Limits::defaults()).unwrap();
    assert_eq!(r.results.len(), 1, "{:?}", r.load_errors);
    assert_eq!(r.results[0].status, Status::Error, "{:?}", r.results[0]);
    assert!(
        r.results[0]
            .mismatches
            .iter()
            .any(|m| m.actual.contains("resource.console-output")),
        "{:?}",
        r.results[0].mismatches
    );
    fn find(p: &Path) -> Option<PathBuf> {
        for entry in std::fs::read_dir(p).ok()? {
            let path = entry.ok()?.path();
            if path.file_name()?.to_str()? == "console-observation.json" {
                return Some(path);
            }
            if path.is_dir() {
                if let Some(found) = find(&path) {
                    return Some(found);
                }
            }
        }
        None
    }
    let raw: serde_json::Value =
        serde_json::from_slice(&std::fs::read(find(&temp.path().join("out")).unwrap()).unwrap())
            .unwrap();
    assert_eq!(raw["outcome"], "resource");
    assert_eq!(raw["console"]["stdout_b64"], "MQo=");
    assert_eq!(raw["console"]["stdout_truncated"], false);
}
