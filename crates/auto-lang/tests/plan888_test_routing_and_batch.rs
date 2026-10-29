//! PLAN-888: real discovery, routing and batch-consumer regressions.
#![cfg(feature = "test-cases")]
use auto_lang::test_cases::inventory::suite_inventory;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/governance-887")
}

#[test]
fn legacy_inventory_uses_real_discovery_and_preserves_distinct_engine_obligations() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test");
    let inventory = auto_lang::test_cases::inventory::legacy_inventory(&root).unwrap();
    assert_eq!(inventory["executed"], 0);
    let rows = inventory["cases"].as_array().unwrap();
    let vm = auto_lang::test_runner::discover_vm_tests(&root.join("vm"));
    assert_eq!(
        rows.iter().filter(|r| r["family"] == "vm").count(),
        vm.len()
    );
    assert!(rows.iter().any(|r| r["family"] == "a2r"));
    assert!(rows.iter().any(|r| r["family"] == "a2c"));
    assert!(rows.iter().any(|r| r["family"] == "a2ts"));
    for row in rows {
        assert_eq!(row["source"]["sha256"].as_str().unwrap().len(), 64);
        assert!(!row["oracles"].as_array().unwrap().is_empty());
    }
}

#[test]
fn inventory_preserves_atomic_checks_profiles_and_oracles() {
    let value =
        suite_inventory(&fixture().join("first-wave.suite.ad"), &Limits::defaults()).unwrap();
    let rows = value["checks"].as_array().unwrap();
    assert_eq!(rows.len(), 6);
    assert_eq!(value["executed"], 0);
    let arithmetic: Vec<_> = rows
        .iter()
        .filter(|r| r["case"] == "gov.d.arithmetic")
        .collect();
    assert_eq!(arithmetic.len(), 2);
    assert!(arithmetic.iter().any(|r| r["profile"] == "entry-unit"));
    assert!(arithmetic.iter().any(|r| r["profile"].is_null()));
    for row in rows {
        assert_eq!(row["source_sha256"].as_str().unwrap().len(), 64);
        assert_eq!(row["oracle_sha256"].as_str().unwrap().len(), 64);
        assert!(row["span"]["open_line"].as_u64().unwrap() > 0);
        assert_eq!(row["state"], "RUN");
    }
    assert!(arithmetic.iter().all(|r| r["expected_exit"] == 3));
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &to.join(entry.file_name()));
        } else {
            std::fs::copy(entry.path(), to.join(entry.file_name())).unwrap();
        }
    }
}

#[test]
fn inventory_binds_changed_source_without_following_its_oracle() {
    let root = std::env::temp_dir().join(format!("plan888-inventory-{}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).unwrap();
    }
    copy_tree(&fixture(), &root);
    let suite = root.join("first-wave.suite.ad");
    let before = suite_inventory(&suite, &Limits::defaults()).unwrap();
    let source = root.join("src/arithmetic.wrapper.at");
    let text = std::fs::read_to_string(&source).unwrap();
    let changed = text.replace("1 + 2", "1 + 3");
    assert_ne!(changed, text);
    std::fs::write(&source, changed).unwrap();
    let after = suite_inventory(&suite, &Limits::defaults()).unwrap();
    let pick = |value: &serde_json::Value| {
        value["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["case"] == "gov.d.arithmetic" && r["check"] == "av")
            .unwrap()
            .clone()
    };
    let (before, after) = (pick(&before), pick(&after));
    assert_ne!(before["source_sha256"], after["source_sha256"]);
    assert_eq!(before["oracle_sha256"], after["oracle_sha256"]);
    assert_eq!(after["expected_exit"], 3);
    std::fs::remove_file(&source).unwrap();
    assert!(suite_inventory(&suite, &Limits::defaults()).is_err());
    std::fs::remove_dir_all(&root).unwrap();
}

#[cfg(all(feature = "test-case-batch", windows))]
mod batch {
    use super::*;
    use auto_lang::test_cases::ac_av::{self, test_batch};
    use auto_test_case::contract::{CheckSpec, PayloadSpec};
    use auto_test_case::discovery::{load_suite, LoadedSuite};
    use auto_test_case::ir::{resolve_case, resolve_oracle, ResolvedCase, ResolvedOracle};
    use auto_test_case::report::Status;
    use auto_test_case::suite::apply_defaults;

    struct Bound {
        case: ResolvedCase,
        check: CheckSpec,
        oracle: ResolvedOracle,
        root: PathBuf,
    }

    fn bind(suite: &LoadedSuite, case_id: &str, check_id: &str, root: PathBuf) -> Bound {
        let (file, spec) = suite
            .cases
            .iter()
            .flat_map(|f| f.specs.iter().map(move |s| (f, s)))
            .find(|(_, s)| s.id == case_id)
            .expect("original case identity");
        let case = resolve_case(
            spec.clone(),
            &file.doc,
            &suite.fixture_base,
            &Limits::defaults(),
        )
        .unwrap();
        let declared = spec
            .checks
            .iter()
            .find(|c| c.id == check_id)
            .expect("original check identity");
        let (check, _) = apply_defaults(declared, &suite.suite.defaults);
        let oracle =
            resolve_oracle(case_id, &check, &suite.fixture_base, &Limits::defaults()).unwrap();
        Bound {
            case,
            check,
            oracle,
            root,
        }
    }

    fn pilot(tag: &str) -> Vec<Bound> {
        let root = std::env::temp_dir().join(format!("plan888-batch-{}-{tag}", std::process::id()));
        if root.exists() {
            std::fs::remove_dir_all(&root).unwrap();
        }
        std::fs::create_dir_all(&root).unwrap();
        let corpus = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/ace-m1");
        let suite = load_suite(&corpus.join("ace-m1-base.suite.ad"), &Limits::defaults()).unwrap();
        assert!(suite.errors.is_empty());
        let binding = std::fs::read_to_string(corpus.join("binding.tsv")).unwrap();
        let mut result = Vec::new();
        for n in 1..=12 {
            let case_id = format!("ace.m1.b{n:02}");
            let mut item = if matches!(n, 8 | 11) {
                // These sources have existing binding-table AV obligations but
                // no green four-path suite row (legacy differs). Create a new
                // AV-only consumer using the ORIGINAL source and binding oracle.
                let mut item = bind(&suite, "ace.m1.b01", "av", root.join(format!("b{n:02}")));
                let original_id = format!("B{n:02}");
                let row: Vec<_> = binding
                    .lines()
                    .find(|l| l.starts_with(&format!("{original_id}\tcorpus\t")))
                    .expect("original binding row")
                    .split('\t')
                    .collect();
                assert_eq!(row[6], "ok", "AV obligation already registered");
                let expected: i32 = row[4].parse().unwrap();
                let file = &suite.cases[0];
                let mut spec = item.case.spec.clone();
                spec.id = format!("ace.m1.binding.b{n:02}");
                spec.source.payload = PayloadSpec::File(format!("src/{original_id}.at"));
                item.case = resolve_case(spec, &file.doc, &suite.fixture_base, &Limits::defaults())
                    .unwrap();
                item.check.expect.exit = Some(expected);
                item.oracle = resolve_oracle(
                    &item.case.spec.id,
                    &item.check,
                    &suite.fixture_base,
                    &Limits::defaults(),
                )
                .unwrap();
                item
            } else {
                bind(&suite, &case_id, "av", root.join(format!("b{n:02}")))
            };
            item.case.spec.checks = vec![item.check.clone()];
            result.push(item);
        }
        let suite =
            load_suite(&fixture().join("first-wave.suite.ad"), &Limits::defaults()).unwrap();
        for name in ["arithmetic", "unary", "group"] {
            result.push(bind(
                &suite,
                &format!("gov.d.{name}"),
                "av-scalar-control",
                root.join(name),
            ));
        }
        assert_eq!(result.len(), 15);
        result
    }

    fn items<'a>(values: &'a [Bound], label: &str) -> Vec<test_batch::Item<'a>> {
        // Each mode has a separate parent artifact/store directory.
        for value in values {
            std::fs::create_dir_all(value.root.join(label)).unwrap();
        }
        // Root itself is unique per pilot/mode. Callers pass a fresh pilot to
        // each mode, so the borrowed paths need no temporary allocation here.
        values
            .iter()
            .map(|v| test_batch::Item {
                case: &v.case,
                check: &v.check,
                oracle: &v.oracle,
                project_dir: &v.root,
                artifact_dir: &v.root,
                run_dir: &v.root,
            })
            .collect()
    }

    #[test]
    fn batch_real_15_matches_independent_with_k1_k3_k10_and_tail() {
        let baseline = pilot("independent");
        let mut expected = Vec::new();
        for item in &baseline {
            let verdict = ac_av::run_check(
                &item.case,
                &item.check,
                &item.oracle,
                &item.root,
                &item.root,
                &item.root,
            )
            .unwrap();
            assert_eq!(
                verdict.status,
                Status::Pass,
                "{} {:?}",
                item.case.spec.id,
                verdict.detail
            );
            let actual = verdict.native.unwrap().exit_code;
            assert_eq!(actual, item.oracle.exit);
            expected.push(actual);
        }
        for k in [1, 3, 10] {
            let values = pilot(&format!("group-k{k}"));
            let borrowed = items(&values, "group");
            let mut actuals = Vec::new();
            let mut starts = 0;
            for chunk in borrowed.chunks(k) {
                let report =
                    test_batch::run_group(chunk, std::time::Duration::from_secs(60)).unwrap();
                assert!(
                    report.error.is_none(),
                    "{:?}; stderr={}",
                    report.error,
                    report.stderr
                );
                starts += report.worker_starts;
                assert_eq!(report.items.len(), chunk.len());
                for item in report.items {
                    assert_eq!(item.status, Status::Pass, "{} {:?}", item.id, item.detail);
                    actuals.push(item.verdict.unwrap().native.unwrap().exit_code);
                }
            }
            assert_eq!(starts, 15usize.div_ceil(k));
            assert_eq!(actuals, expected);
        }
    }

    #[test]
    fn batch_wrong_oracles_are_individual_failures_and_safe_tail_still_runs() {
        let mut values = pilot("wrong-oracles");
        values[0].oracle.exit = Some(999);
        values[2].oracle.exit = Some(998);
        let borrowed = items(&values[..3], "group");
        let report = test_batch::run_group(&borrowed, std::time::Duration::from_secs(60)).unwrap();
        assert!(
            report.error.is_none(),
            "{:?}; {}",
            report.error,
            report.stderr
        );
        assert_eq!(
            report.items.iter().map(|r| r.status).collect::<Vec<_>>(),
            [Status::Fail, Status::Pass, Status::Fail]
        );
        for index in [0, 2] {
            let verdict = report.items[index].verdict.as_ref().unwrap();
            assert!(verdict.mismatches.iter().any(|m| m.observation == "exit"));
            assert!(report.items[index].id.contains(&values[index].case.spec.id));
        }
    }

    #[test]
    fn batch_rejects_unlisted_source_and_negative_oracle_before_dispatch() {
        let mut values = pilot("whitelist-negative");
        values[0].case.files[0]
            .bytes
            .extend_from_slice(b"\n// unreviewed source bytes\n");
        let borrowed = items(&values[..1], "group");
        let error = test_batch::run_group(&borrowed, std::time::Duration::from_secs(60))
            .err()
            .expect("unreviewed source must remain independent");
        assert!(error.contains("15-case batch allowlist"));
        let mut values = pilot("negative-oracle");
        values[0].oracle.completion = auto_test_case::contract::Completion::Error {
            stage: auto_test_case::contract::Stage::Parse,
            code: None,
            message: None,
        };
        let borrowed = items(&values[..1], "group");
        let error = test_batch::run_group(&borrowed, std::time::Duration::from_secs(60))
            .err()
            .expect("negative tests are not batch pilot inputs");
        assert!(error.contains("scalar whitelist"));
    }

    #[cfg(windows)]
    fn process_is_live(pid: u32) -> bool {
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
            fn GetExitCodeProcess(process: *mut std::ffi::c_void, code: *mut u32) -> i32;
            fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
        }
        unsafe {
            let process = OpenProcess(0x1000, 0, pid);
            if process.is_null() {
                return false;
            }
            let mut code = 259;
            let live = GetExitCodeProcess(process, &mut code) != 0 && code == 259;
            CloseHandle(process);
            live
        }
    }

    #[test]
    #[cfg(windows)]
    fn batch_fatal_protocol_and_io_faults_reap_tree_and_preserve_not_run_tail() {
        for fault in [
            "exit-item-1",
            "wrong-index",
            "wrong-nonce",
            "wrong-hash",
            "wrong-source",
            "truncated-item",
            "no-done",
            "extra-after-done",
            "exit-after-done",
            "hang-init",
            "no-read",
            "stdout-flood",
            "stderr-flood",
        ] {
            let values = pilot(fault);
            let borrowed = items(&values[..3], "group");
            let started = std::time::Instant::now();
            let report = test_batch::run_group_with_fault(
                &borrowed,
                std::time::Duration::from_secs(1),
                Some(fault),
            )
            .unwrap();
            assert!(report.error.is_some(), "{fault}: false green");
            let expected_error = match fault {
                "hang-init" | "no-read" => "deadline",
                "stdout-flood" => "metadata cap",
                "stderr-flood" => "stderr cap",
                "wrong-index" => "identity/order",
                "wrong-nonce" | "wrong-hash" => "echo mismatch",
                "wrong-source" => "source_hash",
                "truncated-item" => "truncated",
                "no-done" => "missing batch done",
                "extra-after-done" => "extra batch output",
                "exit-after-done" => "nonzero exit",
                "exit-item-1" => "EOF before item actual",
                _ => unreachable!(),
            };
            assert!(
                report.error.as_deref().unwrap().contains(expected_error),
                "{fault}: wrong rejection kind {:?}",
                report.error
            );
            assert!(report.cleanup_complete, "{fault}: {:?}", report.error);
            assert!(
                !process_is_live(report.worker_pid),
                "{fault}: worker still live"
            );
            assert!(
                started.elapsed() < std::time::Duration::from_secs(4),
                "{fault}: unbounded cleanup"
            );
            assert_eq!(report.items.len(), 3);
            if fault == "exit-item-1" {
                assert_eq!(
                    report.items.iter().map(|r| r.status).collect::<Vec<_>>(),
                    [Status::Pass, Status::Error, Status::NotRun]
                );
            }
            if [
                "wrong-index",
                "wrong-nonce",
                "wrong-hash",
                "wrong-source",
                "truncated-item",
            ]
            .contains(&fault)
            {
                assert_eq!(report.items[0].status, Status::Error);
                assert!(report.items[1..].iter().all(|r| r.status == Status::NotRun));
            }
            if ["no-done", "extra-after-done", "exit-after-done"].contains(&fault) {
                assert!(report.items.iter().all(|r| r.status == Status::Pass));
            }
            let control = &values[0];
            let independent = ac_av::run_check(
                &control.case,
                &control.check,
                &control.oracle,
                &control.root,
                &control.root,
                &control.root,
            )
            .unwrap();
            assert_eq!(
                independent.status,
                Status::Pass,
                "{fault}: following independent control"
            );
        }
    }

    #[test]
    #[cfg(windows)]
    fn batch_blocked_request_writer_is_deadlined_and_reaped() {
        let mut values = pilot("blocked-writer");
        let mut spec = values[0].case.spec.clone();
        let code = String::from_utf8(values[0].case.files[0].bytes.clone()).unwrap()
            + &"\n// bounded padding".repeat(5000);
        spec.source.payload = PayloadSpec::Text(code);
        let doc = auto_test_case::document::CaseDocument::parse(b"# padding control\n").unwrap();
        values[0].case = resolve_case(spec, &doc, &values[0].root, &Limits::defaults()).unwrap();
        let borrowed = items(&values[..1], "group");
        let report = test_batch::run_group_with_fault(
            &borrowed,
            std::time::Duration::from_secs(1),
            Some("no-read-after-ready"),
        )
        .unwrap();
        assert!(report.error.as_deref().unwrap().contains("deadline"));
        assert!(report.cleanup_complete);
        assert!(!process_is_live(report.worker_pid));
        assert_eq!(report.items[0].status, Status::Error);
        let control = &values[0];
        let independent = ac_av::run_check(
            &control.case,
            &control.check,
            &control.oracle,
            &control.root,
            &control.root,
            &control.root,
        )
        .unwrap();
        assert_eq!(
            independent.status,
            Status::Pass,
            "supported padded source must remain valid"
        );
    }

    #[test]
    fn batch_reverse_and_fixed_shuffle_keep_per_case_values_and_fresh_stores() {
        for order in [vec![2, 1, 0], vec![1, 0, 2]] {
            let values = pilot(&format!("order-{order:?}"));
            let borrowed = items(&values[..3], "group");
            let reordered: Vec<_> = order
                .iter()
                .map(|&i| test_batch::Item {
                    case: borrowed[i].case,
                    check: borrowed[i].check,
                    oracle: borrowed[i].oracle,
                    project_dir: borrowed[i].project_dir,
                    artifact_dir: borrowed[i].artifact_dir,
                    run_dir: borrowed[i].run_dir,
                })
                .collect();
            let report =
                test_batch::run_group(&reordered, std::time::Duration::from_secs(60)).unwrap();
            assert!(report.error.is_none(), "{:?}", report.error);
            for (row, index) in report.items.iter().zip(order) {
                assert_eq!(row.status, Status::Pass);
                assert_eq!(
                    row.verdict
                        .as_ref()
                        .unwrap()
                        .native
                        .as_ref()
                        .unwrap()
                        .exit_code,
                    values[index].oracle.exit
                );
                let store = values[index].root.join("store");
                assert!(
                    std::fs::read_dir(store).unwrap().next().is_none(),
                    "request store must be removed"
                );
            }
        }
    }
}
