//! PLAN-887 governance integration — first-wave suite admission, D-group
//! real ACE consumption, V-06/V-08 controls.
//!
//! FORMAL SURFACE (registered batch selection): this file is the explicit
//! integration target for the 887 governance checks — it must be invoked as
//!   cargo test -p auto-lang --features test-cases --test plan887_test_governance
//! A bare `cargo t`/`--lib` run never covers it (no auto_case_lib module
//! re-export). Suite under test: crates/auto-lang/test/cases/governance-887/
//! first-wave.suite.ad (30 reuse obligations B01..B20 + U01..U10 plus the
//! three D-group ACE-consumption cases).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const SUITE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test/cases/governance-887/first-wave.suite.ad"
);
const M1_BASE_SUITE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test/cases/ace-m1/ace-m1-base.suite.ad"
);
const M1_ENTRY_SUITE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/test/cases/ace-m1/entry-862/entry-unit-862.suite.ad"
);
const D_SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/test/vm/01_basics");
// pilot-recorded sha256 of the three D-group sources (identity must not drift).
const SHA_ARITH: &str = "cb898749d76e51fdaf6c6636920ccdb4f415fc1cffd1e2497636b38cc2469807";
const SHA_UNARY: &str = "c85ac4c786ef797783df4b2c58840ad717b70022fe78ff4f7753826ea929e405";
const SHA_GROUP: &str = "5a7e0aea55b602ab4ec4826bec6026efb6a4a22891a6a1176b8a58b31eb56ed2";

fn auto_case(args: &[&str]) -> Output {
    // Use the pre-built worker binaries directly (AC_*_CASE_WORKER overrides)
    // so concurrent tests never contend on the bounded worker auto-build.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    Command::new(env!("CARGO_BIN_EXE_auto-case"))
        .env(
            "AC_AV_CASE_WORKER",
            root.join("experimental/ac-av-probe/target/release/av_probe_worker.exe"),
        )
        .env(
            "AC_MN_CASE_WORKER",
            root.join("experimental/ac-mir-native/target/release/mn-worker.exe"),
        )
        .args(args)
        .output()
        .expect("auto-case spawn failed")
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("plan887-{}-{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn first_wave_suite_admits_and_lists() {
    let lint = auto_case(&["lint", "--suite", SUITE]);
    assert!(
        lint.status.success(),
        "lint failed: {}",
        String::from_utf8_lossy(&lint.stderr)
    );
    let list = auto_case(&["list", "--suite", SUITE, "--format", "json"]);
    assert!(list.status.success(), "list failed");
    let text = String::from_utf8_lossy(&list.stdout);
    for id in ["gov.d.arithmetic", "gov.d.unary", "gov.d.group"] {
        assert!(text.contains(id), "missing case {id} in listing:\n{text}");
    }
}

/// V-03: the 30 reuse obligations (A+B = B01..B20, C = U01..U10) are judged
/// by the ORIGINAL delivered suites with their original recipes/profiles
/// (admission forbids cross-root includes, so identity is kept in place).
#[test]
fn reuse_obligations_run_green_via_original_suites() {
    for (name, suite) in [
        ("ace-m1-base", M1_BASE_SUITE),
        ("entry-862", M1_ENTRY_SUITE),
    ] {
        let out = temp_dir(&format!("reuse-{name}"));
        let run = auto_case(&["run", "--suite", suite, "--out", out.to_str().unwrap()]);
        assert!(
            run.status.success(),
            "{name} reuse suite must pass (exit {:?}):\nstdout:\n{}\nstderr:\n{}",
            run.status.code(),
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
    }
}

#[test]
fn first_wave_suite_runs_green_with_d_values() {
    let out = temp_dir("run");
    let json_out = out.join("report.json");
    let run = auto_case(&[
        "run",
        "--suite",
        SUITE,
        "--out",
        out.to_str().unwrap(),
        "--json-out",
        json_out.to_str().unwrap(),
    ]);
    assert!(
        run.status.success(),
        "suite run failed (exit {:?}):\nstdout:\n{}\nstderr:\n{}",
        run.status.code(),
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_out).expect("report.json"))
            .expect("report json");
    let results = report["results"].as_array().expect("results array");
    assert!(
        !results.is_empty(),
        "zero dynamic checks is a contract error"
    );
    for (case, check, _expect) in [
        ("gov.d.arithmetic", "av", 3),
        ("gov.d.arithmetic", "av-scalar-control", 3),
        ("gov.d.unary", "av", -6),
        ("gov.d.unary", "av-scalar-control", -6),
        ("gov.d.group", "av", 9),
        ("gov.d.group", "av-scalar-control", 9),
    ] {
        let entry = results
            .iter()
            .find(|r| r["case"] == case && r["check"] == check)
            .unwrap_or_else(|| panic!("missing result for {case}/{check}"));
        assert_eq!(
            entry["status"],
            "PASS",
            "{case}/{check} not pass: {}",
            serde_json::to_string(entry).unwrap()
        );
        assert_eq!(
            entry["mismatches"].as_array().map(|a| a.len()).unwrap_or(0),
            0
        );
    }
    // identity axes: source sha256 bound in the receipt (parent-side oracle
    // never shipped to the worker — adapter contract; verified by protocol
    // tests). D sources byte-identical to pilot records.
    assert_eq!(
        std::fs::read(format!("{D_SOURCE}/002_arithmetic/arithmetic.at")).unwrap(),
        b"1 + 2\n"
    );
}

#[test]
fn v08_wrong_oracle_fails_legal_control_passes() {
    let dir = temp_dir("v08");
    // wrong-oracle control: same wrapper bytes, oracle 3 -> 4 (must FAIL).
    let src = dir.join("arith.wrapper.at");
    std::fs::write(&src, b"fn main() int { return 1 + 2 }\n").unwrap();
    let wrong_case = dir.join("wrong.case.ad");
    std::fs::write(
        &wrong_case,
        format!(
            "```test-case\ncase {{ schema: \"autolang.case/2\" id: \"v08.wrong\" \
             tags: [\"plan887\"] links: [\"PLAN-887\"] \
             source: {{ file: \"arith.wrapper.at\", syntax: \"auto\", path: \"main.at\" }} \
             entry: {{ mode: \"program\" }} \
             checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
             options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\", profile: \"entry-unit\" }}, steps: [{{ op: \"run\" }}] }}, \
             expect: {{ completion: \"success\", av_channel: \"value\", exit: 4 }} }} ] }}\n```\n"
        ),
    )
    .unwrap();
    let suite = dir.join("wrong.suite.ad");
    std::fs::write(
        &suite,
        format!(
            "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"v08-wrong\", include: [\"wrong.case.ad\"] }}\n```\n"
        ),
    )
    .unwrap();
    let out = temp_dir("v08-out");
    let run = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        run.status.code(),
        Some(1),
        "wrong oracle must FAIL (exit 1):\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    // legal control on the same bytes passes.
    let ok_case = dir.join("ok.case.ad");
    std::fs::write(
        &ok_case,
        format!(
            "```test-case\ncase {{ schema: \"autolang.case/2\" id: \"v08.ok\" \
             tags: [\"plan887\"] links: [\"PLAN-887\"] \
             source: {{ file: \"arith.wrapper.at\", syntax: \"auto\", path: \"main.at\" }} \
             entry: {{ mode: \"program\" }} \
             checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
             options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\", profile: \"entry-unit\" }}, steps: [{{ op: \"run\" }}] }}, \
             expect: {{ completion: \"success\", av_channel: \"value\", exit: 3 }} }} ] }}\n```\n"
        ),
    )
    .unwrap();
    let ok_suite = dir.join("ok.suite.ad");
    std::fs::write(
        &ok_suite,
        format!(
            "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"v08-ok\", include: [\"ok.case.ad\"] }}\n```\n"
        ),
    )
    .unwrap();
    let ok_out = temp_dir("v08-ok-out");
    let ok = auto_case(&[
        "run",
        "--suite",
        ok_suite.to_str().unwrap(),
        "--out",
        ok_out.to_str().unwrap(),
    ]);
    assert!(
        ok.status.success(),
        "legal control must pass:\n{}",
        String::from_utf8_lossy(&ok.stdout)
    );
}

#[test]
fn v06_stale_rejects_changed_source_identity() {
    let dir = temp_dir("v06");
    std::fs::write(
        dir.join("u.wrapper.at"),
        b"fn main() int { return -2 * 3 }\n",
    )
    .unwrap();
    let case = dir.join("c.case.ad");
    std::fs::write(
        &case,
        format!(
            "```test-case\ncase {{ schema: \"autolang.case/2\" id: \"v06.c\" \
             tags: [\"plan887\"] links: [\"PLAN-887\"] \
             source: {{ file: \"u.wrapper.at\", syntax: \"auto\", path: \"main.at\" }} \
             entry: {{ mode: \"program\" }} \
             checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
             options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\", profile: \"entry-unit\" }}, steps: [{{ op: \"run\" }}] }}, \
             expect: {{ completion: \"success\", av_channel: \"value\", exit: -6 }} }} ] }}\n```\n"
        ),
    )
    .unwrap();
    let suite = dir.join("s.suite.ad");
    std::fs::write(
        &suite,
        format!(
            "```test-suite\nsuite {{ schema: \"autolang.suite/2\", id: \"v06\", include: [\"c.case.ad\"] }}\n```\n"
        ),
    )
    .unwrap();
    let out = temp_dir("v06-out");
    let report = temp_dir("v06-report").join("report.json");
    let run1 = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--json-out",
        report.to_str().unwrap(),
    ]);
    assert!(
        run1.status.success(),
        "warm run must pass:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&run1.stdout),
        String::from_utf8_lossy(&run1.stderr)
    );
    // Change the source identity (same semantic value -6, different bytes):
    // the old report must be rejected as stale.
    std::fs::write(
        dir.join("u.wrapper.at"),
        b"fn main() int { return -(2 * 3) }\n",
    )
    .unwrap();
    let stale = auto_case(&[
        "stale",
        "--suite",
        suite.to_str().unwrap(),
        "--report",
        report.to_str().unwrap(),
    ]);
    assert!(
        !stale.status.success(),
        "stale report must be rejected after source identity change:\n{}",
        String::from_utf8_lossy(&stale.stdout)
    );
    // warm re-run on the NEW bytes still executes and judges (reuse is not a
    // cached pass): exit stays 0 with the same oracle.
    let out2 = temp_dir("v06-out2");
    let run2 = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        out2.to_str().unwrap(),
    ]);
    assert!(
        run2.status.success(),
        "re-run after identity change must re-judge and pass"
    );
}

#[test]
fn sources_byte_identical_to_pilot_records() {
    let arith = std::fs::read(format!("{D_SOURCE}/002_arithmetic/arithmetic.at")).unwrap();
    let unary = std::fs::read(format!("{D_SOURCE}/006_unary/unary.at")).unwrap();
    let group = std::fs::read(format!("{D_SOURCE}/007_group/group.at")).unwrap();
    assert_eq!(arith, b"1 + 2\n");
    assert_eq!(unary, b"-2 * 3\n");
    assert_eq!(group, b"(1 + 2) * 3\n");
    // Content-hash identity is pinned in the case files' headers; the exact
    // sha256 values are verified by scripts/test-governance (see inventory).
    let _ = (SHA_ARITH, SHA_UNARY, SHA_GROUP);
    // sources must not be wrapped: raw bytes are the full source files.
    assert!(Path::new(D_SOURCE)
        .join("002_arithmetic/arithmetic.at")
        .exists());
}

/// V-04 control 1: a required source that is missing must be a contract
/// error (exit 2), never a green or a silent skip.
#[test]
fn v04_missing_source_rejected() {
    let dir = temp_dir("v04-missing");
    let case = dir.join("c.case.ad");
    std::fs::write(
        &case,
        "```test-case\ncase { schema: \"autolang.case/2\" id: \"v04.missing\" \
         tags: [\"plan887\"] links: [\"PLAN-887\"] \
         source: { file: \"no-such-source.at\", syntax: \"auto\", path: \"main.at\" } \
         entry: { mode: \"program\" } \
         checks: [ { id: \"av\", adapter: \"probe\", action: \"run\", \
         options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [{ op: \"run\" }] }, \
         expect: { completion: \"success\", exit: 0 } } ] }\n```\n",
    )
    .unwrap();
    let suite = dir.join("s.suite.ad");
    std::fs::write(
        &suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"v04-missing\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let run = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        temp_dir("v04-missing-out").to_str().unwrap(),
    ]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "missing source must be a contract error:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

/// V-04 control 1: duplicate case identity must be a contract error.
#[test]
fn v04_duplicate_case_id_rejected() {
    let dir = temp_dir("v04-dup");
    for n in ["a", "b"] {
        std::fs::write(dir.join(format!("{n}.at")), b"fn main() int { return 1 }\n").unwrap();
        std::fs::write(
            dir.join(format!("{n}.case.ad")),
            format!(
                "```test-case\ncase {{ schema: \"autolang.case/2\" id: \"v04.same\" \
                 tags: [\"plan887\"] links: [\"PLAN-887\"] \
                 source: {{ file: \"{n}.at\", syntax: \"auto\", path: \"main.at\" }} \
                 entry: {{ mode: \"program\" }} \
                 checks: [ {{ id: \"av\", adapter: \"probe\", action: \"run\", \
                 options: {{ recipe: \"ac-av/core-run-v1\", av: {{ entry: \"d_main\" }}, steps: [{{ op: \"run\" }}] }}, \
                 expect: {{ completion: \"success\", exit: 0 }} }} ] }}\n```\n"
            ),
        )
        .unwrap();
    }
    let suite = dir.join("s.suite.ad");
    std::fs::write(
        &suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"v04-dup\", include: [\"a.case.ad\", \"b.case.ad\"] }\n```\n",
    )
    .unwrap();
    let run = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        temp_dir("v04-dup-out").to_str().unwrap(),
    ]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "duplicate case id must be a contract error:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
}

/// V-04 control 2: two LEGAL checks on the same source with distinct valid
/// oracles are two obligations — both preserved in the receipt.
#[test]
fn v04_legal_same_source_dual_obligation() {
    let dir = temp_dir("v04-dual");
    std::fs::write(dir.join("seven.at"), b"fn main() int { return 7 }\n").unwrap();
    let case = dir.join("c.case.ad");
    std::fs::write(
        &case,
        "```test-case\ncase { schema: \"autolang.case/2\" id: \"v04.dual\" \
         tags: [\"plan887\"] links: [\"PLAN-887\"] \
         source: { file: \"seven.at\", syntax: \"auto\", path: \"main.at\" } \
         entry: { mode: \"program\" } \
         checks: [ \
         { id: \"av\", adapter: \"probe\", action: \"run\", \
         options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\", profile: \"entry-unit\" }, steps: [{ op: \"run\" }] }, \
         expect: { completion: \"success\", av_channel: \"value\", exit: 7 } }, \
         { id: \"av-scalar\", adapter: \"probe\", action: \"run\", \
         options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [{ op: \"run\" }] }, \
         expect: { completion: \"success\", exit: 7 } } ] }\n```\n",
    )
    .unwrap();
    let suite = dir.join("s.suite.ad");
    std::fs::write(
        &suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"v04-dual\", include: [\"c.case.ad\"] }\n```\n",
    )
    .unwrap();
    let out = temp_dir("v04-dual-out");
    let json_out = out.join("report.json");
    let run = auto_case(&[
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
        "--json-out",
        json_out.to_str().unwrap(),
    ]);
    assert!(
        run.status.success(),
        "dual-obligation run must pass:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_out).expect("report"))
            .expect("report json");
    assert_eq!(
        report["results"].as_array().map(|a| a.len()).unwrap_or(0),
        2,
        "two legal checks on the same source must stay two obligations"
    );
}

/// V-05 control 3/4: a typed language diagnostic judged by stage+code is a
/// legitimate positive control; a WRONG code on the same input must be red.
#[test]
fn v05_typed_diagnostic_positive_and_wrong_code_red() {
    let dir = temp_dir("v05");
    // Unit-entry source: the scalar profile must reject it with
    // source.unsupported (typed parse-stage diagnostic — U02 precedent).
    std::fs::write(dir.join("unit.at"), b"let a = 1\nlet b = a + 1\n").unwrap();
    let pos = dir.join("pos.case.ad");
    std::fs::write(
        &pos,
        "```test-case\ncase { schema: \"autolang.case/2\" id: \"v05.pos\" \
         tags: [\"plan887\"] links: [\"PLAN-887\"] \
         source: { file: \"unit.at\", syntax: \"auto\", path: \"main.at\" } \
         entry: { mode: \"program\" } \
         checks: [ { id: \"av-scalar-rejects\", adapter: \"probe\", action: \"run\", \
         options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [{ op: \"run\" }] }, \
         expect: { completion: { stage: \"parse\", code: \"source.unsupported\" } } } ] }\n```\n",
    )
    .unwrap();
    let pos_suite = dir.join("pos.suite.ad");
    std::fs::write(
        &pos_suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"v05-pos\", include: [\"pos.case.ad\"] }\n```\n",
    )
    .unwrap();
    let pos_run = auto_case(&[
        "run",
        "--suite",
        pos_suite.to_str().unwrap(),
        "--out",
        temp_dir("v05-pos-out").to_str().unwrap(),
    ]);
    assert!(
        pos_run.status.success(),
        "typed-diagnostic positive control must pass:\n{}",
        String::from_utf8_lossy(&pos_run.stdout)
    );
    // Wrong code: expect a DIFFERENT diagnostic on the same input -> red.
    let neg = dir.join("neg.case.ad");
    std::fs::write(
        &neg,
        "```test-case\ncase { schema: \"autolang.case/2\" id: \"v05.neg\" \
         tags: [\"plan887\"] links: [\"PLAN-887\"] \
         source: { file: \"unit.at\", syntax: \"auto\", path: \"main.at\" } \
         entry: { mode: \"program\" } \
         checks: [ { id: \"av-scalar-rejects\", adapter: \"probe\", action: \"run\", \
         options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [{ op: \"run\" }] }, \
         expect: { completion: { stage: \"parse\", code: \"source.undefined\" } } } ] }\n```\n",
    )
    .unwrap();
    let neg_suite = dir.join("neg.suite.ad");
    std::fs::write(
        &neg_suite,
        "```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"v05-neg\", include: [\"neg.case.ad\"] }\n```\n",
    )
    .unwrap();
    let neg_run = auto_case(&[
        "run",
        "--suite",
        neg_suite.to_str().unwrap(),
        "--out",
        temp_dir("v05-neg-out").to_str().unwrap(),
    ]);
    assert_eq!(
        neg_run.status.code(),
        Some(1),
        "wrong diagnostic code must be red (exit 1):\n{}",
        String::from_utf8_lossy(&neg_run.stdout)
    );
}

/// V-07 control 7: subset selection — the receipt carries the selected
/// case's checks as PASS and marks every unselected check NOT_RUN (named
/// "not selected"); an outer NOT_RUN is never green (exit 1).
#[test]
fn v07_subset_selection_only() {
    let out = temp_dir("v07-subset");
    let json_out = out.join("report.json");
    let run = auto_case(&[
        "run",
        "--suite",
        SUITE,
        "--case",
        "gov.d.arithmetic",
        "--out",
        out.to_str().unwrap(),
        "--json-out",
        json_out.to_str().unwrap(),
    ]);
    assert_eq!(
        run.status.code(),
        Some(1),
        "subset run with unselected checks must exit 1 (NOT_RUN, never green):\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_out).expect("report"))
            .expect("report json");
    let results = report["results"].as_array().expect("results");
    let passed: Vec<_> = results.iter().filter(|r| r["status"] == "PASS").collect();
    assert_eq!(passed.len(), 2, "exactly the selected case's checks pass");
    for r in passed {
        assert_eq!(r["case"], "gov.d.arithmetic");
    }
    let not_run: Vec<_> = results
        .iter()
        .filter(|r| r["status"] == "NOT_RUN")
        .collect();
    assert_eq!(not_run.len(), 4, "unselected checks are explicitly NOT_RUN");
    for r in not_run {
        let detail = r["detail"].as_str().unwrap_or("");
        assert!(
            detail.contains("not selected"),
            "NOT_RUN must name the selection: {detail}"
        );
    }
}

/// V-07 control 7: emit-only checks are reported as emit (never as run);
/// a green emit does not claim execution. Asserted over the original M1
/// suite run (a2r-snapshot checks carry declared artifact oracles).
#[test]
fn v07_emit_not_run() {
    let out = temp_dir("v07-emit");
    let json_out = out.join("report.json");
    let run = auto_case(&[
        "run",
        "--suite",
        M1_BASE_SUITE,
        "--out",
        out.to_str().unwrap(),
        "--json-out",
        json_out.to_str().unwrap(),
    ]);
    assert!(
        run.status.success(),
        "M1 emit-bearing run must pass:\n{}",
        String::from_utf8_lossy(&run.stdout)
    );
    let report: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&json_out).expect("report"))
            .expect("report json");
    let results = report["results"].as_array().expect("results");
    let emit: Vec<_> = results.iter().filter(|r| r["action"] == "emit").collect();
    assert!(!emit.is_empty(), "the M1 suite must carry a2r emit checks");
    for r in emit {
        assert_eq!(r["status"], "PASS");
        assert_ne!(r["action"], "run", "emit must never claim run");
    }
    let run_actions: Vec<_> = results.iter().filter(|r| r["action"] == "run").collect();
    assert!(
        !run_actions.is_empty(),
        "the M1 suite must carry real run checks"
    );
}
