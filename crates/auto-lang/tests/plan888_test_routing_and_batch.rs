//! PLAN-888: real discovery, routing and batch-consumer regressions.
#![cfg(feature = "test-cases")]
use auto_lang::test_cases::inventory::suite_inventory;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test/cases/governance-887")
}

#[test]
fn module_case_cli_keeps_whole_suite_not_run_and_uses_original_oracle() {
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("module-case-{}-{stamp}", std::process::id()));
    let inputs = root.join("inputs");
    std::fs::create_dir_all(&inputs).unwrap();
    let suite = inputs.join("s.suite.ad");
    std::fs::write(&suite,"```test-suite\nsuite { schema: \"autolang.suite/1\", id: \"scope\", include: [\"c.case.ad\"] }\n```\n").unwrap();
    let doc = |expected| {
        format!("```test-case\ncase {{ schema: \"autolang.case/1\", id: \"first\", source: {{ text: \"1+2\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"expression\" }}, checks: [{{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: {{ completion: \"success\", result: {{ format: \"legacy-display\", text: \"{expected}\" }} }} }}] }}\n```\n```test-case\ncase {{ schema: \"autolang.case/1\", id: \"second\", source: {{ text: \"4\", syntax: \"auto\", path: \"main.at\" }}, entry: {{ mode: \"expression\" }}, checks: [{{ id: \"vm\", adapter: \"avm\", action: \"run\", expect: {{ completion: \"success\", result: {{ format: \"legacy-display\", text: \"4\" }} }} }}] }}\n```\n")
    };
    for (expected, code, status) in [(3, 0, "PASS"), (4, 1, "FAIL")] {
        std::fs::write(inputs.join("c.case.ad"), doc(expected)).unwrap();
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_auto-case"))
            .arg("module-case")
            .arg("--suite")
            .arg(&suite)
            .arg("--case")
            .arg("first")
            .arg("--out")
            .arg(root.join(format!("out-{expected}")))
            .output()
            .unwrap();
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["scope"], "case");
        assert_eq!(report["suite_gate_exit"], 1);
        assert_eq!(report["unselected"][0]["status"], "NOT_RUN");
        let rows = report["selected_report"]["results"].as_array().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["status"], status);
    }
    std::fs::remove_dir_all(root).unwrap();
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
    use auto_test_case::report::Status;
    use auto_test_case::{contract::PayloadSpec, ir::resolve_case};

    use test_batch::pilot::Bound;
    fn pilot(tag: &str) -> Vec<Bound> {
        test_batch::pilot::load(tag).unwrap()
    }
    fn items<'a>(values: &'a [Bound], _label: &str) -> Vec<test_batch::Item<'a>> {
        test_batch::pilot::items(values)
    }

    #[test]
    fn combined_d3_counts_real_producers_maps_source_and_preserves_parent_failures() {
        let mut values = pilot("combined-d3");
        let report =
            test_batch::combined::run(&values[12..], std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(report["worker_starts"], 1);
        assert_eq!(report["observation"]["metrics"]["calls"]["compile"], 1);
        assert_eq!(report["observation"]["metrics"]["calls"]["vm"], 1);
        assert_eq!(report["observation"]["profile"], "av.owned/1");
        assert!(report["items"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["status"] == "PASS"));
        assert!(!process_is_live(
            report["worker_pid"].as_u64().unwrap() as u32
        ));
        let source = report["source"].as_str().unwrap().as_bytes();
        for row in report["items"].as_array().unwrap() {
            let origin = &row["origin"];
            let original = std::fs::read(origin["path"].as_str().unwrap()).unwrap();
            let begin = origin["source_byte_range"][0].as_u64().unwrap() as usize;
            let end = origin["source_byte_range"][1].as_u64().unwrap() as usize;
            assert_eq!(&source[begin..end], original);
        }
        values[12].oracle.exit = Some(4);
        values[14].oracle.exit = Some(10);
        let wrong =
            test_batch::combined::run(&values[12..], std::time::Duration::from_secs(5)).unwrap();
        assert_eq!(
            wrong["items"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v["status"].as_str().unwrap())
                .collect::<Vec<_>>(),
            ["FAIL", "PASS", "FAIL"]
        );
        for (fault, expected) in [
            ("wrong-source", "identity"),
            ("wrong-index", "identity"),
            ("wrong-count", "count"),
            ("wrong-vm-count", "count"),
            ("extra-after-done", "extra combined"),
            ("no-read", "deadline"),
        ] {
            let start = std::time::Instant::now();
            let error = test_batch::combined::run_with_fault(
                &values[12..],
                std::time::Duration::from_secs(1),
                Some(fault),
            )
            .unwrap_err();
            assert!(error.contains(expected), "{fault}: {error}");
            assert!(start.elapsed() < std::time::Duration::from_secs(4));
        }
        let fresh = pilot("after-combined-fatal");
        let independent = ac_av::run_check(
            &fresh[12].case,
            &fresh[12].check,
            &fresh[12].oracle,
            &fresh[12].root,
            &fresh[12].root,
            &fresh[12].root,
        )
        .unwrap();
        assert_eq!(independent.status, Status::Pass);
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
        // Original D expectations are 3, -6, 9. Break two parent oracles,
        // including the prescribed 3 -> 4 counter-control.
        values[12].oracle.exit = Some(4);
        values[14].oracle.exit = Some(10);
        let borrowed = items(&values[12..15], "group");
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
            assert!(report.items[index]
                .id
                .contains(&values[index + 12].case.spec.id));
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
            // Deadline probes deliberately have a one-second budget. For
            // protocol probes give the three original programs enough time
            // to reach the injected frame, rather than accepting a timeout
            // under machine load as evidence of the intended rejection.
            let seconds = if ["hang-init", "no-read"].contains(&fault) {
                1
            } else {
                5
            };
            let report = test_batch::run_group_with_fault(
                &borrowed,
                std::time::Duration::from_secs(seconds),
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
                started.elapsed() < std::time::Duration::from_secs(seconds + 3),
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
            let values = pilot("reordered");
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
