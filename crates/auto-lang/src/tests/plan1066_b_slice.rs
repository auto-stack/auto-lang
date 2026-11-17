//! PLAN-1066 T-03[B]/NC-02..06/21: N-family document authority controls.
//!
//! Each control mirrors one real corpus entry into a throwaway root, then runs
//! the SAME product discovery + entry the daily wrappers use, over that root:
//!
//! * doc-only mirror (frozen `.at`/`.as`/`.expected.*` removed) still discovers
//!   and executes the entry (NC-02);
//! * removing the document keeps the entry visible and fails loudly with the
//!   document path — never a sidecar fallback and never a smaller corpus (NC-03);
//! * polluting the frozen sidecars changes nothing, while editing the document
//!   changes the judged result (NC-04);
//! * a wrong `vm-result` in a dual-channel document is red on the result axis
//!   while the stdout channel is untouched (NC-05/NC-21).
//!
//! Mirrors live in the OS temp dir; the author tree is read-only in this test.

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn copy_dir_all(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create mirror dir");
    for entry in std::fs::read_dir(from).expect("read source dir").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copy fixture file");
        }
    }
}

/// Mirror `<family>/<category>/<case>` into `root/<family>/<category>/<case>`.
fn mirror_case(root: &Path, family: &str, category: &str, case: &str) -> PathBuf {
    let from = repo().join("test").join(family).join(category).join(case);
    let to = root.join(family).join(category).join(case);
    copy_dir_all(&from, &to);
    to
}

fn stem_of(case_dir: &Path) -> String {
    case_dir
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.split_once('_'))
        .map(|(_, stem)| stem.to_string())
        .expect("case dir has a stem")
}

fn remove_frozen_sidecars(case_dir: &Path) {
    let stem = stem_of(case_dir);
    for suffix in [
        ".at",
        ".as",
        ".expected.out",
        ".expected.result",
        ".expected.error",
        ".expected.rs",
        ".expected.c",
        ".expected.h",
        ".expected.ts",
        ".expected.error.log",
    ] {
        let path = case_dir.join(format!("{stem}{suffix}"));
        if path.exists() {
            std::fs::remove_file(path).expect("remove frozen sidecar");
        }
    }
}

fn passed(report: &auto_lang_core::test_runner::FileTestReport) -> bool {
    matches!(
        report.outcome,
        auto_lang_core::test_runner::TestOutcome::Passed
    )
}

fn failure(report: &auto_lang_core::test_runner::FileTestReport) -> String {
    match &report.outcome {
        auto_lang_core::test_runner::TestOutcome::Failed(message) => message.clone(),
        auto_lang_core::test_runner::TestOutcome::Passed => "PASSED".to_string(),
    }
}

fn vm_case(root: &Path, category: &str, case: &str) -> auto_lang_core::test_runner::FileTestCase {
    let cases = auto_lang_core::test_runner::discover_vm_tests(&root.join("vm"));
    cases
        .into_iter()
        .find(|c| c.name == format!("{category}/{case}"))
        .unwrap_or_else(|| panic!("vm case {category}/{case} not discovered"))
}

#[test]
fn plan1066_b_doc_only_mirrors_discover_and_execute_per_family() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();

    // vm
    let dir = mirror_case(root, "vm", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_vm_tests(&root.join("vm"));
    assert_eq!(
        cases.len(),
        1,
        "vm doc-only mirror must still discover its entry"
    );
    let report = auto_lang_core::run_vm_file_test(&cases[0]);
    assert!(passed(&report), "vm doc-only run: {}", failure(&report));

    // a2r
    let dir = mirror_case(root, "a2r", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_a2r_tests(&root.join("a2r"), "a2r");
    assert_eq!(
        cases.len(),
        1,
        "a2r doc-only mirror must still discover its entry"
    );
    let report = auto_lang_core::run_a2r_file_test(&cases[0]);
    assert!(passed(&report), "a2r doc-only run: {}", failure(&report));

    // a2c
    let dir = mirror_case(root, "a2c", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_a2c_tests(&root.join("a2c"));
    assert_eq!(
        cases.len(),
        1,
        "a2c doc-only mirror must still discover its entry"
    );
    let report = auto_lang_core::run_a2c_file_test(&cases[0]);
    assert!(passed(&report), "a2c doc-only run: {}", failure(&report));

    // a2ts: the family's 85 goldens predate the current emit (`console.log` →
    // `globalThis.console.log`), so the explicit entry judgment is red at the
    // golden axis and every a2ts case is #[ignore]d. The obligation here is
    // that removing the frozen sidecars does NOT change that judgment — the
    // document (byte-identical to the frozen `.expected.ts`) still owns it.
    let dir = mirror_case(root, "a2ts", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_a2ts_tests(&root.join("a2ts"));
    assert_eq!(
        cases.len(),
        1,
        "a2ts doc-only mirror must still discover its entry"
    );
    let mirror_report = auto_lang_core::run_a2ts_file_test(&cases[0]);
    let real_cases = auto_lang_core::test_runner::discover_a2ts_tests(&repo().join("test/a2ts"));
    let real = real_cases
        .into_iter()
        .find(|c| c.name == "01_basics/001_hello")
        .expect("real a2ts hello");
    let real_report = auto_lang_core::run_a2ts_file_test(&real);
    assert_eq!(
        passed(&mirror_report),
        passed(&real_report),
        "doc-only mirror changes the a2ts judgment: {} vs {}",
        failure(&mirror_report),
        failure(&real_report)
    );
    assert_eq!(
        failure(&mirror_report),
        failure(&real_report),
        "doc-only mirror changes the judged message"
    );
    // The document is the author source: breaking it IS judged differently.
    let doc_path = dir.join("hello.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("read document");
    let mutated = doc.replace("fn main() {", "fn main( {");
    assert_ne!(doc, mutated, "control must edit the document source");
    std::fs::write(&doc_path, mutated).expect("write mutated document");
    let broken = auto_lang_core::run_a2ts_file_test(&{
        let mut case = cases[0].clone();
        case.dir = dir.clone();
        case
    });
    assert!(
        failure(&broken).contains("parse error"),
        "a document source edit must reach the transpiler, got: {}",
        failure(&broken)
    );
}

#[test]
fn plan1066_b_removed_document_keeps_the_entry_and_fails_loudly() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let dir = mirror_case(root, "vm", "01_basics", "001_hello");
    let stem = stem_of(&dir);
    std::fs::remove_file(dir.join(format!("{stem}.case.ad"))).expect("remove document");

    // The frozen selection still exists, so the entry must NOT vanish.
    let cases = auto_lang_core::test_runner::discover_vm_tests(&root.join("vm"));
    assert_eq!(
        cases.len(),
        1,
        "removing the document must not shrink the corpus"
    );
    let report = auto_lang_core::run_vm_file_test(&cases[0]);
    assert!(
        !passed(&report),
        "legacy sidecars must not stand in for the document"
    );
    let message = failure(&report);
    assert!(
        message.contains("migration document"),
        "failure must name the missing document, got: {message}"
    );
    assert!(
        message.contains(".case.ad"),
        "failure must name the document path, got: {message}"
    );
}

#[test]
fn plan1066_b_polluted_sidecars_are_history_but_document_edits_are_judged() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let dir = mirror_case(root, "vm", "01_basics", "001_hello");
    let stem = stem_of(&dir);

    // Pollute every frozen sidecar: the document still owns the payload.
    std::fs::write(
        dir.join(format!("{stem}.at")),
        "fn main() { print(\"BROKEN\") }\n",
    )
    .expect("pollute source sidecar");
    std::fs::write(dir.join(format!("{stem}.expected.out")), "BROKEN\n")
        .expect("pollute oracle sidecar");
    let case = vm_case(root, "01_basics", "001_hello");
    let report = auto_lang_core::run_vm_file_test(&case);
    assert!(
        passed(&report),
        "polluted sidecars must not change the judged result: {}",
        failure(&report)
    );

    // Editing the document IS judged: the stdout channel goes red.
    let doc_path = dir.join(format!("{stem}.case.ad"));
    let doc = std::fs::read_to_string(&doc_path).expect("read document");
    let mutated = doc.replace("print(\"hello\")", "print(\"published\")");
    assert_ne!(doc, mutated, "control must actually edit the document");
    std::fs::write(&doc_path, mutated).expect("write mutated document");
    let report = auto_lang_core::run_vm_file_test(&vm_case(root, "01_basics", "001_hello"));
    assert!(!passed(&report), "a document source edit must be judged");
    assert!(
        failure(&report).contains("stdout mismatch"),
        "the failing axis must be named: {}",
        failure(&report)
    );
}

#[test]
fn plan1066_b_dual_channel_result_is_judged_on_the_result_axis() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let dir = mirror_case(root, "vm", "04_control_flow", "004_if_with_bool");
    let stem = stem_of(&dir);

    // Unmutated dual-channel document: both channels pass in one run.
    let report =
        auto_lang_core::run_vm_file_test(&vm_case(root, "04_control_flow", "004_if_with_bool"));
    assert!(
        passed(&report),
        "dual-channel baseline: {}",
        failure(&report)
    );

    // Only the `vm-result` payload is wrong (stdout stays correct).
    let doc_path = dir.join(format!("{stem}.case.ad"));
    let doc = std::fs::read_to_string(&doc_path).expect("read document");
    let marker = "check-id: \"vm-result\"\nfinal-newline: false\n\n```text\nsucc\n```";
    assert!(
        doc.contains(marker),
        "dual-channel document shape changed: {doc}"
    );
    std::fs::write(
        &doc_path,
        doc.replace(marker, &marker.replace("succ", "succX")),
    )
    .expect("write mutated document");

    let report =
        auto_lang_core::run_vm_file_test(&vm_case(root, "04_control_flow", "004_if_with_bool"));
    assert!(!passed(&report), "a wrong result channel must be red");
    assert!(
        failure(&report).contains("result mismatch"),
        "the failing axis must be the result axis: {}",
        failure(&report)
    );
}

/// PLAN-1066 T-03[B]/V-02: per-item actuals of one family through the same
/// entry the `Test.run_*_dir` shims call. `AUTO_TEST_1066_FAMILY` selects the
/// family; `AUTO_TEST_1066_ACTUALS_OUT` receives one JSON line per item, so a
/// specialty batch can diff old/new payloads per item instead of trusting a
/// bundle verdict. The VM family is replayed through its own daily wrapper
/// instead (its runtime hangs on CWD-sensitive cases, recorded as debt).
#[test]
#[ignore = "explicit PLAN-1066 V-02 tool: AUTO_TEST_1066_FAMILY=<a2r|a2c|a2ts> AUTO_TEST_1066_ACTUALS_OUT=<file>"]
fn plan1066_b_family_actuals() {
    let family = std::env::var("AUTO_TEST_1066_FAMILY").expect("AUTO_TEST_1066_FAMILY");
    let out = std::env::var("AUTO_TEST_1066_ACTUALS_OUT").expect("AUTO_TEST_1066_ACTUALS_OUT");
    let test_root = repo().join("test");
    let mut rows: Vec<(String, String)> = Vec::new();
    match family.as_str() {
        "a2r" => {
            for (dir, suite) in [("a2r", "a2r"), ("cookbook", "a2r")] {
                for case in
                    auto_lang_core::test_runner::discover_a2r_tests(&test_root.join(dir), suite)
                {
                    let report = auto_lang_core::run_a2r_file_test(&case);
                    rows.push((case.name.clone(), format!("{:?}", report.outcome)));
                }
            }
        }
        "a2c" => {
            for case in auto_lang_core::test_runner::discover_a2c_tests(&test_root.join("a2c")) {
                let report = auto_lang_core::run_a2c_file_test(&case);
                rows.push((case.name.clone(), format!("{:?}", report.outcome)));
            }
        }
        "a2ts" => {
            for case in auto_lang_core::test_runner::discover_a2ts_tests(&test_root.join("a2ts")) {
                let report = auto_lang_core::run_a2ts_file_test(&case);
                rows.push((case.name.clone(), format!("{:?}", report.outcome)));
            }
        }
        other => panic!("unknown family {other}"),
    }
    let mut lines = String::new();
    for (id, outcome) in &rows {
        lines.push_str(
            &serde_json::json!({"family": family, "id": id, "outcome": outcome}).to_string(),
        );
        lines.push('\n');
    }
    std::fs::write(&out, lines).expect("write actuals");
    let failed = rows
        .iter()
        .filter(|(_, outcome)| !outcome.starts_with("Passed"))
        .count();
    eprintln!("FAMILY {family}: items={} failed={failed}", rows.len());
    for (id, outcome) in rows
        .iter()
        .filter(|(_, outcome)| !outcome.starts_with("Passed"))
        .take(8)
    {
        eprintln!("  FAILED {id}: {outcome}");
    }
}

/// PLAN-1066 T-03[B]: authoritative payload proof. For every discovered entry
/// the DECODER's source bytes and inline artifact bytes are hashed and emitted,
/// so a scope script can compare them with the frozen sidecars instead of
/// re-implementing the document grammar.
#[test]
#[ignore = "explicit PLAN-1066 payload proof tool: AUTO_TEST_1066_PAYLOAD_OUT=<file>"]
fn plan1066_b_payload_proof() {
    use auto_test_case::sha256::{hex, sha256};
    let out = std::env::var("AUTO_TEST_1066_PAYLOAD_OUT").expect("AUTO_TEST_1066_PAYLOAD_OUT");
    let test_root = repo().join("test");
    let mut entries: Vec<(String, std::path::PathBuf)> = Vec::new();
    for case in auto_lang_core::test_runner::discover_vm_tests(&test_root.join("vm")) {
        entries.push((format!("vm/{}", case.name), case.dir));
    }
    for (dir, suite) in [("a2r", "a2r"), ("cookbook", "a2r")] {
        for case in auto_lang_core::test_runner::discover_a2r_tests(&test_root.join(dir), suite) {
            entries.push((format!("{suite}/{}", case.name), case.dir));
        }
    }
    for case in auto_lang_core::test_runner::discover_a2c_tests(&test_root.join("a2c")) {
        entries.push((format!("a2c/{}", case.name), case.dir));
    }
    for case in auto_lang_core::test_runner::discover_a2ts_tests(&test_root.join("a2ts")) {
        entries.push((format!("a2ts/{}", case.name), case.dir));
    }
    let mut lines = String::new();
    for (id, dir) in &entries {
        let doc = crate::tests::legacy_doc::load_case(dir, &stem_of(dir))
            .unwrap_or_else(|e| panic!("{id}: {e}"));
        let artifacts: Vec<serde_json::Value> = doc
            .case
            .spec
            .checks
            .iter()
            .flat_map(|check| check.expect.artifacts.iter())
            .filter_map(|artifact| match &artifact.payload {
                auto_test_case::contract::ArtifactPayload::Inline { bytes, .. } => {
                    Some(serde_json::json!({"path": artifact.path,
                        "sha256": hex(&sha256(bytes)), "len": bytes.len()}))
                }
                _ => None,
            })
            .collect();
        let checks: Vec<serde_json::Value> = doc
            .case
            .spec
            .checks
            .iter()
            .map(|check| {
                serde_json::json!({
                    "id": check.id,
                    "result_sha256": check.expect.result.as_ref()
                        .map(|result| hex(&sha256(result.text.as_bytes()))),
                    "result_len": check.expect.result.as_ref().map(|result| result.text.len()),
                })
            })
            .collect();
        lines.push_str(
            &serde_json::json!({
                "id": id,
                "dir": dir.strip_prefix(repo()).unwrap_or(dir).to_string_lossy(),
                "source_sha256": hex(&sha256(doc.source_text().as_bytes())),
                "source_len": doc.source_text().len(),
                "source_path": doc.source_path(),
                "artifacts": artifacts,
                "checks": checks,
            })
            .to_string(),
        );
        lines.push('\n');
    }
    std::fs::write(&out, lines).expect("write payload proof");
    eprintln!("PAYLOAD entries={}", entries.len());
}

/// NC-22/NC-07: the a2c header axis is judged for what the document declares.
/// A declared header is compared byte-exactly in the SAME transpile as the C
/// source (`C source mismatch` / `C header mismatch` are separate axes), and a
/// case whose document declares no header keeps the frozen zero-byte role
/// instead of silently skipping the axis.
#[test]
fn plan1066_b_c_header_axis_declared_vs_zero_byte_role() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();

    // A declared header is an independent axis of the same emit.
    let dir = mirror_case(root, "a2c", "01_basics", "001_hello");
    let doc_path = dir.join("hello.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("read document");
    let mutated = doc.replace("#pragma once", "#pragma once\n/*wrong header*/");
    assert_ne!(doc, mutated, "control must change the header payload");
    std::fs::write(&doc_path, mutated).expect("write document");
    let cases = auto_lang_core::test_runner::discover_a2c_tests(&root.join("a2c"));
    let report = auto_lang_core::run_a2c_file_test(&cases[0]);
    assert_eq!(
        failure(&report),
        "C header mismatch",
        "header axis must be named"
    );

    // A document without a header artifact keeps the zero-byte role: the frozen
    // sidecar is zero bytes too, so the case must still pass through the real
    // emit (an "absence = skip" shortcut would let a non-empty header pass).
    let temp2 = tempfile::tempdir().expect("temp root");
    let root2 = temp2.path();
    let dir2 = mirror_case(root2, "a2c", "10_collections", "003_array_declaration");
    let stem = stem_of(&dir2);
    assert!(
        !dir2.join(format!("{stem}.expected.h")).is_file(),
        "fixture must be one of the nine zero-byte header roles"
    );
    let cases2 = auto_lang_core::test_runner::discover_a2c_tests(&root2.join("a2c"));
    let report2 = auto_lang_core::run_a2c_file_test(&cases2[0]);
    assert!(
        passed(&report2),
        "zero-byte header role must still pass: {}",
        failure(&report2)
    );
}

/// NC-07: every family consumes its own artifact vocabulary — a C document
/// never declares a `.ts`/`.rs` golden, an a2ts document never a `.c`, and a
/// cookbook document's VM oracle is `cookbook-stdout` (the Rust artifact is a
/// separate consumer, never the VM comparison).
#[test]
fn plan1066_b_family_artifact_vocabulary_is_not_crossed() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let mut offenders = Vec::new();
    let mut counts = std::collections::BTreeMap::new();

    for case in auto_lang_core::test_runner::discover_a2c_tests(&repo().join("test/a2c")) {
        let doc = crate::tests::legacy_doc::load_case(&case.dir, &stem_of(&case.dir)).expect("doc");
        for name in doc
            .case
            .spec
            .checks
            .iter()
            .flat_map(|c| c.expect.artifacts.iter())
        {
            *counts.entry("a2c".to_string()).or_insert(0) += 1;
            if !(name.path.ends_with(".c") || name.path.ends_with(".h")) {
                offenders.push(format!("a2c case {} declares {}", case.name, name.path));
            }
        }
    }
    for case in auto_lang_core::test_runner::discover_a2ts_tests(&repo().join("test/a2ts")) {
        let doc = crate::tests::legacy_doc::load_case(&case.dir, &stem_of(&case.dir)).expect("doc");
        for name in doc
            .case
            .spec
            .checks
            .iter()
            .flat_map(|c| c.expect.artifacts.iter())
        {
            *counts.entry("a2ts".to_string()).or_insert(0) += 1;
            if !name.path.ends_with(".ts") {
                offenders.push(format!("a2ts case {} declares {}", case.name, name.path));
            }
        }
    }
    for case in auto_lang_core::test_runner::discover_a2r_tests(&repo().join("test/a2r"), "a2r") {
        let doc = crate::tests::legacy_doc::load_case(&case.dir, &stem_of(&case.dir)).expect("doc");
        for name in doc
            .case
            .spec
            .checks
            .iter()
            .flat_map(|c| c.expect.artifacts.iter())
        {
            *counts.entry("a2r".to_string()).or_insert(0) += 1;
            if !name.path.ends_with(".rs") {
                offenders.push(format!("a2r case {} declares {}", case.name, name.path));
            }
        }
    }
    for case in
        auto_lang_core::test_runner::discover_a2r_tests(&repo().join("test/cookbook"), "a2r")
    {
        let doc = crate::tests::legacy_doc::load_case(&case.dir, &stem_of(&case.dir)).expect("doc");
        let vm_oracles: Vec<&str> = doc
            .case
            .spec
            .checks
            .iter()
            .filter(|check| check.expect.result.is_some())
            .map(|check| check.id.as_str())
            .collect();
        // A smoke entry (no stdout golden) is legal; an entry WITH a VM oracle
        // must name the cookbook VM face, never the Rust emit artifact.
        if !vm_oracles.is_empty() && !vm_oracles.contains(&"cookbook-stdout") {
            offenders.push(format!(
                "cookbook case {} judges the wrong face: {vm_oracles:?}",
                case.name
            ));
        }
    }
    assert!(counts["a2c"] > 100 && counts["a2ts"] > 80 && counts["a2r"] > 200);
    assert!(
        offenders.is_empty(),
        "crossed artifact vocabularies: {offenders:?}"
    );
    let _ = temp;
}

/// NC-23: the E78 boundary is a SET/KEY fact, not a count. Every document under
/// the five roots belongs to exactly one of {N entry, other-consumer boundary},
/// and moving one boundary document into N is rejected by name — deleting or
/// adding an entry cannot be hidden behind "1050 again".
#[test]
fn plan1066_b_e78_boundary_is_a_key_set_not_a_count() {
    let fixture_path = repo().join("test/governance/plan1066_a_obligations.json");
    let fixture: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&fixture_path).expect("fixture")).expect("json");
    let mut n_docs = std::collections::BTreeSet::new();
    for row in fixture["n_rows"].as_array().expect("rows") {
        n_docs.insert(row["doc_rel"].as_str().expect("doc_rel").to_string());
    }
    let mut e78 = std::collections::BTreeSet::new();
    for row in fixture["e78"].as_array().expect("e78") {
        e78.insert(row["path"].as_str().expect("path").to_string());
    }
    let overlap: Vec<_> = n_docs.intersection(&e78).cloned().collect();
    assert!(
        overlap.is_empty(),
        "N and the E78 boundary share documents: {overlap:?}"
    );

    // every document under the five roots is classified exactly once
    let mut all_docs = std::collections::BTreeSet::new();
    for family in ["vm", "a2r", "a2c", "a2ts", "cookbook"] {
        for entry in walk(&repo().join("test").join(family)) {
            if entry.extension().and_then(|e| e.to_str()) != Some("ad") {
                continue;
            }
            if !entry
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with(".case.ad")
            {
                continue;
            }
            let rel = entry
                .strip_prefix(repo().parent().unwrap().parent().unwrap())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            all_docs.insert(rel);
        }
    }
    let union = n_docs.len() + e78.len();
    assert_eq!(
        union,
        all_docs.len(),
        "unclassified documents: {all_docs:?}"
    );
    assert_eq!(n_docs.len(), 1060, "N denominator");
    assert_eq!(
        e78.len(),
        70,
        "E78 boundary denominator (68 other consumers + 2 P project documents)"
    );

    // mutation: misclassify ONE boundary document as an N entry -> named reject
    let moved = e78.iter().next().expect("boundary document").clone();
    let mut mutated = n_docs.clone();
    mutated.insert(moved.clone());
    let overlap_after: Vec<_> = mutated.intersection(&e78).cloned().collect();
    assert_eq!(
        overlap_after,
        vec![moved.clone()],
        "duplicate classification must be named"
    );
}

fn walk(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&current) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(path);
            }
        }
    }
    out
}

/// NC-24: a document-only `.as` entry keeps its script mode when the frozen
/// `.as` sidecar is deleted and a decoy `.at` appears, and the cookbook
/// multi-source directories resolve to their frozen `001_*` document instead of
/// whichever `.at` sorting would pick.
#[test]
fn plan1066_b_script_mode_and_cookbook_selection_come_from_the_document() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let dir = mirror_case(root, "vm", "18_ffi", "016_json_is_null");
    let stem = stem_of(&dir);
    let script = dir.join(format!("{stem}.as"));
    assert!(script.is_file(), "fixture must be a frozen .as entry");
    std::fs::remove_file(&script).expect("remove .as sidecar");
    std::fs::write(
        dir.join(format!("{stem}.at")),
        "fn main() { print(\"decoy\") }\n",
    )
    .expect("write decoy .at");

    let cases = auto_lang_core::test_runner::discover_vm_tests(&root.join("vm"));
    assert_eq!(
        cases.len(),
        1,
        "doc-only .as entry must still be discovered"
    );
    assert!(
        cases[0].source_file.to_string_lossy().ends_with(".as"),
        "virtual source keeps the document's script extension: {:?}",
        cases[0].source_file
    );
    let doc = crate::tests::legacy_doc::load_case(&dir, &stem).expect("doc");
    assert!(
        doc.source_is_script(),
        "script mode comes from the document"
    );
    let (oracle, stdout, result) =
        crate::tests::vm_file_tests::run_vm_doc_once(&dir, doc.source_is_script()).expect("run");
    let mirror_mismatches =
        crate::tests::vm_file_tests::diff_vm_channels(&oracle, &stdout, &result);
    // The frozen judge for this entry is pre-existing knowledge
    // (`Json.is_null` now reports `true` where the golden says `1`), so the
    // control proves ROUTING equality rather than inventing a pass: the mirror
    // judges exactly what the repository judges, and the decoy `.at` is never
    // executed (its `print("decoy")` never appears on stdout).
    let repo_doc = repo().join("test/vm/18_ffi/016_json_is_null");
    let (repo_oracle, repo_stdout, repo_result) =
        crate::tests::vm_file_tests::run_vm_doc_once(&repo_doc, true).expect("repo run");
    let repo_mismatches =
        crate::tests::vm_file_tests::diff_vm_channels(&repo_oracle, &repo_stdout, &repo_result);
    assert_eq!(
        mirror_mismatches, repo_mismatches,
        "doc-only script routing differs from the repository judgment"
    );
    assert_eq!(
        stdout, repo_stdout,
        "decoy .at must not change the executed script output"
    );
    assert!(
        !stdout.contains("decoy"),
        "the decoy .at source was executed"
    );

    // cookbook multi-source dirs: the frozen 001_* document is the candidate
    for (category, name, expected) in [
        ("asynchronous", "channel", "001_bounded.case.ad"),
        ("asynchronous", "fs", "001_create.case.ad"),
        ("asynchronous", "rt", "001_tokio_macro.case.ad"),
        ("database", "postgres", "001_create_tables.case.ad"),
        ("database", "sqlite", "001_init.case.ad"),
        ("web", "scraping", "001_extract_links.case.ad"),
    ] {
        let case_dir = repo().join("test/cookbook").join(category).join(name);
        let identity = auto_lang_core::test_cases::execution_trace::cookbook_doc_identity(
            &case_dir, category, name,
        );
        let document = identity["document"]["path"]
            .as_str()
            .expect("document path");
        assert!(
            document.ends_with(expected),
            "{category}/{name} selected {document}, expected {expected}"
        );
    }
}

/// NC-06: the bootstrap rule stays the original one (the `99_bootstrap`
/// category prepends the auto library) and the document body — not the frozen
/// `.at` — is what gets prepended to. A wrong/old body therefore cannot pass.
#[test]
fn plan1066_b_bootstrap_rule_and_body_come_from_the_original_mechanism() {
    let cases = auto_lang_core::test_runner::discover_vm_tests(&repo().join("test/vm"));
    let mut bootstrap = Vec::new();
    for case in &cases {
        let expected = case.name.starts_with("99_bootstrap/");
        assert_eq!(
            case.is_bootstrap, expected,
            "bootstrap flag drift for {}",
            case.name
        );
        if expected {
            bootstrap.push(case.name.clone());
        }
    }
    assert!(!bootstrap.is_empty(), "no bootstrap entries discovered");

    let run = |name: &str| {
        let case = cases.iter().find(|c| c.name == name).expect("case");
        auto_lang_core::run_vm_file_test(case)
    };
    let first = &bootstrap[0];
    let report = run(first);
    assert!(
        passed(&report),
        "bootstrap entry {first} must pass: {}",
        failure(&report)
    );

    // The document body is the prepended payload: editing it is judged.
    let temp = tempfile::tempdir().expect("temp root");
    let (category, case_name) = first.split_once('/').expect("category/case");
    let dir = mirror_case(temp.path(), "vm", category, case_name);
    let stem = stem_of(&dir);
    let doc_path = dir.join(format!("{stem}.case.ad"));
    let doc = std::fs::read_to_string(&doc_path).expect("doc");
    let marker = "\"\"\"";
    let mutated = doc.replace("fn main()", "fn main_BROKEN()");
    assert_ne!(doc, mutated, "control must edit the document body");
    std::fs::write(&doc_path, mutated).expect("write");
    let mirrored = auto_lang_core::test_runner::discover_vm_tests(&temp.path().join("vm"));
    let report = auto_lang_core::run_vm_file_test(&mirrored[0]);
    assert!(
        !passed(&report),
        "a bootstrap document edit must be judged, not ignored ({marker})"
    );
}

/// NC-10/11/12: the two project documents are the only author source of their
/// projects — a missing closure entry fails, the document body drives the
/// product (a body edit turns the finite assertion red), a wrong/over-asking
/// assertion is judged by name, and escaping/duplicate virtual paths are
/// rejected before anything is materialized.
#[test]
fn plan1066_b_project_documents_own_their_projects() {
    let temp = tempfile::tempdir().expect("temp root");
    let root = temp.path();
    let source = repo().join("test/a2r/14_modules/005_multi_file");

    let mirror = |name: &str| -> std::path::PathBuf {
        let dst = root.join(name).join("005_multi_file");
        copy_dir_all(&source, &dst);
        // Drop every frozen sidecar: only the document may carry payloads.
        for entry in walk(&dst) {
            let name = entry.file_name().unwrap().to_string_lossy().to_string();
            if name.ends_with(".at") || name.ends_with(".expected.rs") {
                std::fs::remove_file(&entry).expect("remove sidecar");
            }
        }
        dst
    };

    // NC-10a: the documented closure is complete and the mirror still builds.
    let clean = mirror("clean");
    crate::tests::a2r_tests::run_project_case(&clean)
        .unwrap_or_else(|e| panic!("doc-only project mirror must build: {e}"));

    // NC-10b: a MISSING closure entry is a hard failure (main.at uses db).
    let missing = mirror("missing");
    let doc_path = missing.join("main.project.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("doc");
    let start = doc.find("#### db.at").expect("db.at section");
    let end = doc.find("#### api/mod.at").expect("next section");
    let trimmed = format!("{}{}", &doc[..start], &doc[end..]);
    std::fs::write(&doc_path, trimmed).expect("write");
    let err = crate::tests::a2r_tests::run_project_case(&missing)
        .expect_err("missing module closure must fail the original transpiler");
    assert!(!err.is_empty(), "missing closure error must be located");

    // NC-10c: the document body drives the product; a body edit is judged.
    let body = mirror("body");
    let doc_path = body.join("main.project.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("doc");
    let mutated = doc.replace("struct Connection", "struct Connection_X");
    assert_ne!(doc, mutated, "control must edit the db.at body");
    std::fs::write(&doc_path, mutated).expect("write");
    let err = crate::tests::a2r_tests::run_project_case(&body)
        .expect_err("document body edit must be judged by the finite assertion");
    assert!(
        err.contains("db.rs") && err.contains("struct Connection"),
        "{err}"
    );

    // NC-11: an over-asking assertion fails by name, never silently.
    let wrong = mirror("wrong");
    let doc_path = wrong.join("main.project.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("doc");
    let mutated = doc.replace("\"fn connect()\"", "\"fn connect_nowhere()\"");
    assert_ne!(doc, mutated);
    std::fs::write(&doc_path, mutated).expect("write");
    let err = crate::tests::a2r_tests::run_project_case(&wrong)
        .expect_err("wrong contains assertion must fail");
    assert!(err.contains("fn connect_nowhere()"), "{err}");

    let absent = mirror("absent");
    let doc_path = absent.join("main.project.case.ad");
    let doc = std::fs::read_to_string(&doc_path).expect("doc");
    let mutated = doc.replace(
        "#### Cargo.toml\nexists: true",
        "#### not-produced.txt\nexists: true\n\n#### Cargo.toml\nexists: true",
    );
    assert_ne!(doc, mutated);
    std::fs::write(&doc_path, mutated).expect("write");
    let err = crate::tests::a2r_tests::run_project_case(&absent)
        .expect_err("declared-but-absent product must fail");
    assert!(err.contains("not-produced.txt"), "{err}");

    // NC-12: escaping / duplicate virtual paths are rejected at decode time,
    // before any materialization. The author tree must stay untouched.
    for (label, heading) in [
        ("escape", "#### ../escape.at"),
        ("absolute", "#### /abs/escape.at"),
        ("duplicate", "#### db.at"),
    ] {
        let bad = mirror(&format!("path-{label}"));
        let doc_path = bad.join("main.project.case.ad");
        let doc = std::fs::read_to_string(&doc_path).expect("doc");
        // Inject INSIDE the `### Auto` section (before the first additional
        // module file) so the illegal path is a SOURCE path, not a product.
        let anchor = "#### db.at";
        let injected = doc.replacen(
            anchor,
            &format!("{heading}\n\n```auto\nfn x() {{}}\n```\n\n{anchor}"),
            1,
        );
        assert_ne!(doc, injected);
        std::fs::write(&doc_path, injected).expect("write");
        let err = crate::tests::a2r_tests::run_project_case(&bad)
            .expect_err("illegal virtual path must be rejected");
        assert!(
            err.contains("path") || err.contains("路径") || err.contains("重复"),
            "{label}: unexpected error {err}"
        );
    }
}

/// NC-19: the legacy inventory reports the DOCUMENT as the entry identity.
/// A doc-only mirror still lists the entry (no sidecar fallback), polluting the
/// frozen sidecars does not change the identity, and a removed document shows
/// up as `present: false` with its document path instead of silently falling
/// back to a sidecar or dropping the entry.
#[test]
fn plan1066_b_inventory_identity_is_the_document() {
    let temp = tempfile::tempdir().expect("temp root");
    // The inventory walks the wrapper SOURCE declarations beside the corpus, so
    // the mirror keeps the crate layout (`<root>/test/...` + `<root>/src/tests`).
    let crate_root = temp.path().join("crates/auto-lang");
    let root = crate_root.join("test");
    let src_tests = crate_root.join("src/tests");
    std::fs::create_dir_all(&src_tests).expect("mirror src/tests");
    for entry in std::fs::read_dir(repo().join("src/tests"))
        .expect("real src/tests")
        .flatten()
    {
        if entry.path().extension().and_then(|e| e.to_str()) == Some("rs") {
            std::fs::copy(entry.path(), src_tests.join(entry.file_name())).expect("copy module");
        }
    }
    let dir = {
        let from = repo().join("test/a2r/01_basics/001_hello");
        let to = root.join("a2r/01_basics/001_hello");
        copy_dir_all(&from, &to);
        to
    };

    // doc-only mirror: the entry is inventoried by its document
    remove_frozen_sidecars(&dir);
    let inventory =
        auto_lang_core::test_cases::inventory::legacy_inventory(&root).expect("doc-only inventory");
    let rows = inventory["cases"].as_array().expect("cases");
    assert_eq!(rows.len(), 1, "doc-only mirror must still list its entry");
    let source = rows[0]["source"]["path"].as_str().expect("source path");
    assert!(
        source.ends_with("hello.case.ad"),
        "inventory identity must be the document: {source}"
    );
    assert_eq!(rows[0]["source"]["present"], serde_json::json!(true));
    assert_eq!(rows[0]["source_path"], serde_json::json!("hello.at"));
    assert_eq!(rows[0]["document_checks"], serde_json::json!(["rust"]));

    // polluted sidecars do not change the identity (no sidecar read at all)
    std::fs::write(dir.join("hello.at"), "fn main() { print(\"polluted\") }\n").unwrap();
    std::fs::write(dir.join("hello.expected.rs"), "polluted golden\n").unwrap();
    let polluted =
        auto_lang_core::test_cases::inventory::legacy_inventory(&root).expect("polluted inventory");
    assert_eq!(
        polluted["cases"][0]["source"], rows[0]["source"],
        "sidecar pollution must not change the document identity"
    );

    // removed document: the entry stays visible and names the missing document
    std::fs::remove_file(dir.join("hello.case.ad")).unwrap();
    let missing = auto_lang_core::test_cases::inventory::legacy_inventory(&root)
        .expect("inventory with a removed document");
    let rows = missing["cases"].as_array().expect("cases");
    assert_eq!(
        rows.len(),
        1,
        "a removed document must not shrink the inventory"
    );
    let source = rows[0]["source"]["path"].as_str().expect("source path");
    assert!(source.ends_with("hello.case.ad"), "{source}");
    assert_eq!(rows[0]["source"]["present"], serde_json::json!(false));
}

/// NC-25: no ordinary corpus wrapper writes into the author tree.
///
/// Dynamic proof exists where a mirror is possible (NC-A3 for conformance /
/// cookbook / vm; NC-10 for the projects). This static guard covers the rest:
/// in the family wrapper modules, a `fs::write(`/`File::create(` that targets a
/// `test/`-prefixed author path is a bypass and is rejected by file and line.
/// Scoped diagnostics (`diag_scope::write_scoped`) and the explicit
/// proposal/compile scopes are the only legal writers.
fn author_tree_write_offenders(label: &str, text: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    let count = text.lines().count();
    for (index, line) in text.lines().enumerate() {
        let writes = line.contains("fs::write(")
            || line.contains("File::create(")
            || line.contains("OpenOptions::new()");
        if !writes {
            continue;
        }
        if line.contains("write_scoped") || line.trim_start().starts_with("//") {
            continue;
        }
        let start = index.saturating_sub(3);
        let end = (index + 2).min(count);
        let window = text
            .lines()
            .skip(start)
            .take(end - start)
            .collect::<Vec<_>>()
            .join(" ");
        if window.contains("\"test/") && !window.contains("write_scoped") {
            offenders.push(format!("{label}:{}: {}", index + 1, line.trim()));
        }
    }
    offenders
}

#[test]
fn plan1066_b_no_author_tree_writes_in_corpus_wrappers() {
    let wrappers = [
        "a2r_tests.rs",
        "a2c_tests.rs",
        "a2ts_tests.rs",
        "aavm_runner_tests.rs",
        "ffi_dual_tests.rs",
        "vm_file_tests.rs",
        "cookbook_vm_tests.rs",
        "conformance_tests.rs",
        "plan724_http_client_tests.rs",
        "plan727_http_transfer_tests.rs",
    ];
    let mut offenders = Vec::new();
    for wrapper in wrappers {
        // The module sources are compiled from `src/tests`, so read them from
        // disk to keep the scan honest about ALL of them (not just included).
        let path = repo().join("src/tests").join(wrapper);
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        offenders.extend(author_tree_write_offenders(wrapper, &text));
    }
    // The guard must be able to fail: an injected bypass is detected.
    let injected = "fn x() {\n    let p = d.join(\"test/vm/x/y.wrong.out\");\n    std::fs::write(&p, b\"x\").unwrap();\n}\n";
    assert_eq!(
        author_tree_write_offenders("injected.rs", injected).len(),
        1,
        "the static guard must detect an injected author-tree write"
    );
    assert!(
        offenders.is_empty(),
        "author-tree writes in corpus wrappers (use diag_scope::write_scoped):\n  {}",
        offenders.join("\n  ")
    );
}

/// T-09 classified sidecar-read audit.
///
/// The migration closed the ACTIVE sidecar readers (T-03[B] entry/discovery,
/// T-07 bypasses, T-08 inventories, T-09 HTTP wrappers). What remains is named
/// here with its reason, so the set cannot silently grow: a sidecar read in any
/// file outside this list fails by file name, which forces the increment into a
/// Plan instead of a quiet regression to retired payloads. An injected reader
/// (negative control) must be detected.
#[test]
fn plan1066_b_remaining_sidecar_readers_are_named() {
    // file suffix -> why this sidecar read is allowed
    const ALLOWED: &[(&str, &str)] = &[
        (
            "auto-lang-core/src/test_cases/inventory_helpers.rs",
            "migration-comparison static inventory (old-side baseline consumed by document_case_migration.py)",
        ),
        (
            "auto-lang/src/tests/a2r_tests.rs",
            "NC-27 control: proves the document product equals the frozen sidecar",
        ),
        (
            "auto-lang/src/tests/aavm_runner_tests.rs",
            "AAVM experiment corpus (test/vm/aavm2, no migration documents; manual replay only per the AAVM ruling)",
        ),
        (
            "auto-lang/src/tests/book_listing_tests.rs",
            "external book repository listings (D-1066-BOOK-GENERATOR, out of this plan's scope)",
        ),
        (
            "auto-lang/src/tests/ffi_dep_parity_tests.rs",
            "adjacent legacy family (test/ffi_dep) outside the five N roots; not migrated by PLAN-1062",
        ),
        (
            "auto-lang/src/tests/test_cases_tests.rs",
            "synthetic fixtures for the sidecar-payload contract itself",
        ),
        (
            "a2r-actor-tests/tests/actor_parity.rs",
            "external consumer of a2r/22_actors goldens (named in the plan's P/M scope)",
        ),
    ];
    let sidecar = [
        ".expected.out\"",
        ".expected.result\"",
        ".expected.rs\"",
        ".expected.c\"",
        ".expected.h\"",
        ".expected.ts\"",
        ".expected.error\"",
        "expected_output.txt\"",
        "input.at\"",
    ];
    let read_call = ["read_to_string", "fs::read(", "file_identity", "File::open"];
    let manifest = repo();
    let repo_root = manifest
        .parent()
        .expect("workspace")
        .parent()
        .expect("workspace")
        .to_path_buf();
    let mut offenders = Vec::new();
    let mut seen = Vec::new();
    let mut stack = vec![repo_root.join("crates")];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let lines: Vec<&str> = text.lines().collect();
            for (index, line) in lines.iter().enumerate() {
                if !sidecar.iter().any(|needle| line.contains(needle)) {
                    continue;
                }
                let start = index.saturating_sub(2);
                let end = (index + 3).min(lines.len());
                let window = lines[start..end].join(" ");
                if !read_call.iter().any(|needle| window.contains(needle)) {
                    continue;
                }
                let relative = path.to_string_lossy().replace('\\', "/");
                let Some((_, reason)) = ALLOWED
                    .iter()
                    .find(|(suffix, _)| relative.ends_with(suffix))
                else {
                    offenders.push(format!("{}:{}: {}", relative, index + 1, line.trim()));
                    continue;
                };
                seen.push((relative, *reason, index + 1));
            }
        }
    }
    // the audit must actually be looking at the tree (not an empty walk)
    assert!(
        seen.len() >= 10,
        "sidecar-read audit saw only {} allowed sites — the walk broke",
        seen.len()
    );
    // negative control: an injected reader in an unlisted file is detected
    let injected = "fn x() {\n    let s = std::fs::read_to_string(dir.join(format!(\"{}.expected.out\", name)))\n;\n}\n";
    let injected_hit = injected
        .lines()
        .any(|line| sidecar.iter().any(|needle| line.contains(needle)));
    assert!(
        injected_hit,
        "the audit patterns must match a real sidecar read"
    );
    assert!(
        !ALLOWED
            .iter()
            .any(|(suffix, _)| "crates/auto-lang/src/tests/new_wrapper.rs".ends_with(suffix)),
        "an unlisted module must not be allowed"
    );
    assert!(
        offenders.is_empty(),
        "unnamed sidecar readers (bring them into the Plan):\n  {}",
        offenders.join("\n  ")
    );
}

/// One named negative control's reported row.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Plan1066Control {
    nc: &'static str,
    owner: &'static str,
    verdict: &'static str,
    /// Measured atomic duration; `None` means "no timing recorded", which is a
    /// reporting error, never an implicit pass.
    ms: Option<u128>,
}

const CONTROL_LIMIT_MS: u128 = 10_000;

/// NC-20: reject a control report that cannot be trusted as evidence.
///
/// * a named control missing, duplicated, or outside `NC-01..NC-27`;
/// * a control without a measured duration, or at/over the 10 s budget;
/// * a `NOT_RUN` verdict (not executed is not passed);
/// * a parent row claiming PASS while any child row failed/NOT_RUNs, and a
///   non-zero process exit standing in for a per-control verdict.
fn validate_control_report(parent_ok: bool, rows: &[Plan1066Control]) -> Result<(), String> {
    let mut problems = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    // Completeness: the report must carry exactly the 27 named controls.
    let expected: std::collections::BTreeSet<&str> =
        PLAN1066_CONTROL_OWNERS.iter().map(|(nc, _)| *nc).collect();
    let reported: std::collections::BTreeSet<&str> = rows.iter().map(|row| row.nc).collect();
    for missing in expected.difference(&reported) {
        problems.push(format!("{missing}: control missing from the report"));
    }
    for extra in reported.difference(&expected) {
        problems.push(format!("{extra}: control outside the frozen NC set"));
    }
    for row in rows {
        if !seen.insert(row.nc) {
            problems.push(format!("{}: duplicate control row", row.nc));
        }
        if !row.nc.starts_with("NC-") {
            problems.push(format!("{}: not a NC-* control id", row.nc));
        }
        match row.ms {
            None => problems.push(format!("{}: no measured duration", row.nc)),
            Some(ms) if ms >= CONTROL_LIMIT_MS => problems.push(format!(
                "{}: {} ms at/over the {CONTROL_LIMIT_MS} ms budget",
                row.nc, ms
            )),
            Some(_) => {}
        }
        if row.verdict == "NOT_RUN" {
            problems.push(format!("{}: NOT_RUN is not a verdict", row.nc));
        } else if row.verdict != "PASS" {
            problems.push(format!(
                "{}: verdict {} (expected PASS)",
                row.nc, row.verdict
            ));
        }
        if row.owner.is_empty() {
            problems.push(format!("{}: no owning control", row.nc));
        }
    }
    if parent_ok && !problems.is_empty() {
        problems.push("parent PASS cannot mask a failing/NOT_RUN/unmeasured child".into());
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(problems.join("; "))
    }
}

/// Every NC-01..NC-27 must have a real owning control; this table is the
/// contract a review reads, and the test proves it stays complete.
const PLAN1066_CONTROL_OWNERS: &[(&str, &str)] = &[
    ("NC-01", "plan1066_nc_a1_denominator_tamper_rejected + plan1066_b_e78_boundary_is_a_key_set_not_a_count"),
    ("NC-02", "plan1066_b_doc_only_mirrors_discover_and_execute_per_family"),
    ("NC-03", "plan1066_b_removed_document_keeps_the_entry_and_fails_loudly"),
    ("NC-04", "plan1066_b_polluted_sidecars_are_history_but_document_edits_are_judged"),
    ("NC-05", "plan1066_nc_a2_dual_channel_oracle + plan1066_b_dual_channel_result_is_judged_on_the_result_axis"),
    ("NC-06", "plan1066_b_bootstrap_rule_and_body_come_from_the_original_mechanism"),
    ("NC-07", "plan1066_b_family_artifact_vocabulary_is_not_crossed"),
    ("NC-08", "case plan1066-s-doc-controls (five single-axis mutations)"),
    ("NC-09", "test_fixed_groups.ActiveDocumentCaseEntries (active-entry scan controls)"),
    ("NC-10", "plan1066_b_project_documents_own_their_projects (closure + body edit)"),
    ("NC-11", "plan1066_b_project_documents_own_their_projects (wrong/absent assertion)"),
    ("NC-12", "plan1066_b_project_documents_own_their_projects (escaping/duplicate paths)"),
    ("NC-13", "plan1066_compile_gate_classification_and_receipt_controls (receipt tampering)"),
    ("NC-14", "plan1066_compile_gate_classification_and_receipt_controls (mixed errors/ledger)"),
    ("NC-15", "ExplicitCompileGate (ignored gate + explicit group separation)"),
    ("NC-16", "plan1066_conformance_proposal_admission_controls (scope admission)"),
    ("NC-17", "plan1066_conformance_proposal_admission_controls (closed set = denominator)"),
    ("NC-18", "plan1066_conformance_proposal_admission_controls (retired generators stay retired)"),
    ("NC-19", "plan1066_b_inventory_identity_is_the_document"),
    ("NC-20", "plan1066_b_control_report_discipline (this validator)"),
    ("NC-21", "plan1066_b_dual_channel_result_is_judged_on_the_result_axis (native entry)"),
    ("NC-22", "plan1066_b_c_header_axis_declared_vs_zero_byte_role"),
    ("NC-23", "plan1066_b_e78_boundary_is_a_key_set_not_a_count"),
    ("NC-24", "plan1066_b_script_mode_and_cookbook_selection_come_from_the_document"),
    ("NC-25", "plan1066_b_no_author_tree_writes_in_corpus_wrappers + plan1066_nc_a3_no_author_tree_writes"),
    ("NC-26", "plan1066_nc_a4_receipt_verification + plan1066_b_inventory_identity_is_the_document"),
    ("NC-27", "plan1066_compile_gate_classification_and_receipt_controls (fixtures/parity/M03 oracle)"),
];

#[test]
fn plan1066_b_control_report_discipline() {
    // the table itself is complete and unique
    let mut ids = std::collections::BTreeSet::new();
    for (nc, owner) in PLAN1066_CONTROL_OWNERS {
        assert!(ids.insert(*nc), "duplicate control id {nc}");
        assert!(!owner.is_empty(), "{nc}: no owner");
    }
    assert_eq!(ids.len(), 27, "all 27 named controls must be mapped");
    for index in 1..=27 {
        let id = format!("NC-{index:02}");
        assert!(ids.contains(id.as_str()), "{id} has no owning control");
    }

    // a legitimate report passes
    let good: Vec<Plan1066Control> = PLAN1066_CONTROL_OWNERS
        .iter()
        .enumerate()
        .map(|(index, (nc, owner))| Plan1066Control {
            nc,
            owner,
            verdict: "PASS",
            ms: Some(50 + index as u128 * 10),
        })
        .collect();
    assert!(validate_control_report(true, &good).is_ok());

    // each reporting defect is rejected by name
    let mut over_budget = good.clone();
    over_budget[0].ms = Some(CONTROL_LIMIT_MS); // 10.000 s is already over
    let error = validate_control_report(true, &over_budget).expect_err("over budget");
    assert!(error.contains("budget"), "{error}");

    let mut untimed = good.clone();
    untimed[1].ms = None;
    let error = validate_control_report(true, &untimed).expect_err("missing timing");
    assert!(error.contains("no measured duration"), "{error}");

    let mut not_run = good.clone();
    not_run[2].verdict = "NOT_RUN";
    let error = validate_control_report(true, &not_run).expect_err("NOT_RUN");
    assert!(error.contains("NOT_RUN is not a verdict"), "{error}");

    let mut duplicated = good.clone();
    duplicated.push(good[0].clone());
    let error = validate_control_report(true, &duplicated).expect_err("duplicate");
    assert!(error.contains("duplicate control row"), "{error}");

    let mut missing = good.clone();
    missing.remove(10);
    let error = validate_control_report(true, &missing).expect_err("missing control");
    assert!(error.contains("control missing from the report"), "{error}");

    // a parent pass cannot mask a failing child
    let mut failing_child = good.clone();
    failing_child[5].verdict = "FAIL";
    let error = validate_control_report(true, &failing_child).expect_err("masked child");
    assert!(error.contains("parent PASS cannot mask"), "{error}");
    // ... and with an honest parent verdict it is only the child row that reads red
    assert!(validate_control_report(false, &failing_child).is_err());
}
