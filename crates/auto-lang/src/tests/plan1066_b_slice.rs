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
        ".at", ".as", ".expected.out", ".expected.result", ".expected.error",
        ".expected.rs", ".expected.c", ".expected.h", ".expected.ts",
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
    assert_eq!(cases.len(), 1, "vm doc-only mirror must still discover its entry");
    let report = auto_lang_core::run_vm_file_test(&cases[0]);
    assert!(passed(&report), "vm doc-only run: {}", failure(&report));

    // a2r
    let dir = mirror_case(root, "a2r", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_a2r_tests(&root.join("a2r"), "a2r");
    assert_eq!(cases.len(), 1, "a2r doc-only mirror must still discover its entry");
    let report = auto_lang_core::run_a2r_file_test(&cases[0]);
    assert!(passed(&report), "a2r doc-only run: {}", failure(&report));

    // a2c
    let dir = mirror_case(root, "a2c", "01_basics", "001_hello");
    remove_frozen_sidecars(&dir);
    let cases = auto_lang_core::test_runner::discover_a2c_tests(&root.join("a2c"));
    assert_eq!(cases.len(), 1, "a2c doc-only mirror must still discover its entry");
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
    assert_eq!(cases.len(), 1, "a2ts doc-only mirror must still discover its entry");
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
    assert_eq!(cases.len(), 1, "removing the document must not shrink the corpus");
    let report = auto_lang_core::run_vm_file_test(&cases[0]);
    assert!(!passed(&report), "legacy sidecars must not stand in for the document");
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
    std::fs::write(dir.join(format!("{stem}.at")), "fn main() { print(\"BROKEN\") }\n")
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
    let report = auto_lang_core::run_vm_file_test(&vm_case(root, "04_control_flow", "004_if_with_bool"));
    assert!(passed(&report), "dual-channel baseline: {}", failure(&report));

    // Only the `vm-result` payload is wrong (stdout stays correct).
    let doc_path = dir.join(format!("{stem}.case.ad"));
    let doc = std::fs::read_to_string(&doc_path).expect("read document");
    let marker = "check-id: \"vm-result\"\nfinal-newline: false\n\n```text\nsucc\n```";
    assert!(doc.contains(marker), "dual-channel document shape changed: {doc}");
    std::fs::write(&doc_path, doc.replace(marker, &marker.replace("succ", "succX")))
        .expect("write mutated document");

    let report = auto_lang_core::run_vm_file_test(&vm_case(root, "04_control_flow", "004_if_with_bool"));
    assert!(!passed(&report), "a wrong result channel must be red");
    assert!(
        failure(&report).contains("result mismatch"),
        "the failing axis must be the result axis: {}",
        failure(&report)
    );
}
