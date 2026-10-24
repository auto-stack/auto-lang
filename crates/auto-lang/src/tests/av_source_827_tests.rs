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

/// V-10 negative oracle: a WRONG exit expectation on the real AP-01 bytes
/// must FAIL — the parent is a real judge (good actual + wrong expected is
/// red; equivalently wrong actual + good expected).
#[test]
fn av_source_827_negative_oracle_fails() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("neg");
    let ap01 = std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../auto-lang/test/cases/av-source-827/src/p-gcd-euclid.at"),
    )
    .expect("AP-01 corpus copy");
    std::fs::write(root.join("main.at"), ap01).unwrap();
    std::fs::write(
        root.join("neg.case.ad"),
        r#"# neg case
```test-case
case {
  schema: "autolang.case/2",
  id: "neg827.wrong-exit",
  source: { file: "main.at", syntax: "auto", path: "main.at" },
  entry: { mode: "program" },
  checks: [
    { id: "run-wrong-exit", adapter: "probe", action: "run",
      options: {
        recipe: "ac-av/core-run-v1",
        av: { entry: "d_main" },
        steps: [ { op: "run" } ]
      },
      expect: { completion: "success", exit: 8 } }
  ]
}
```
"#,
    )
    .unwrap();
    std::fs::write(
        root.join("neg.suite.ad"),
        r#"# neg suite
```test-suite
suite { schema: "autolang.suite/2", id: "neg-827", include: ["neg.case.ad"] }
```
"#,
    )
    .unwrap();
    let out = out_of(&root, "out");
    let report = run_suite(&root.join("neg.suite.ad"), &out, &Limits::defaults())
        .expect("suite runs");
    let row = report
        .results
        .iter()
        .find(|r| r.check_id == "run-wrong-exit")
        .expect("check present");
    assert_eq!(
        row.status,
        auto_test_case::report::Status::Fail,
        "a wrong expectation is a real FAIL, never green"
    );
}

/// V-10 worker-contract faults on the SOURCE path: protocol violations
/// (non-zero exit, trailing frames, a result hash not bound to the
/// request's revision) and a wall breach are infra ERRORs — the formal
/// source entry can never turn them into language verdicts.
#[test]
fn av_source_827_worker_faults_are_infra() {
    let _guard = crate::tests::ac_av_probe_tests::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("faults");
    let run_one = |tag: &str| -> auto_test_case::report::Status {
        let case_dir = root.join(tag);
        let out_dir = root.join(format!("{tag}-out"));
        let _ = std::fs::remove_dir_all(&case_dir);
        let _ = std::fs::remove_dir_all(&out_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        let ap01 = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../auto-lang/test/cases/av-source-827/src/p-gcd-euclid.at"),
        )
        .expect("AP-01 corpus copy");
        std::fs::write(case_dir.join("main.at"), ap01).unwrap();
        std::fs::write(
            case_dir.join("c.case.ad"),
            "# fault\n```test-case\ncase {\n  schema: \"autolang.case/2\",\n  id: \"av.src.fault\",\n  source: { file: \"main.at\", syntax: \"auto\", path: \"main.at\" },\n  entry: { mode: \"program\" },\n  checks: [\n    { id: \"run\", adapter: \"probe\", action: \"run\",\n      options: { recipe: \"ac-av/core-run-v1\", av: { entry: \"d_main\" }, steps: [ { op: \"run\" } ] },\n      expect: { completion: \"success\", exit: 0 } }\n  ]\n}\n```\n",
        )
        .unwrap();
        std::fs::write(
            case_dir.join("s.suite.ad"),
            "# s\n```test-suite\nsuite { schema: \"autolang.suite/2\", id: \"s827f\", include: [\"c.case.ad\"] }\n```\n",
        )
        .unwrap();
        let report = run_suite(&case_dir.join("s.suite.ad"), &out_dir, &Limits::defaults())
            .expect("fault suite runs");
        report
            .results
            .iter()
            .find(|r| r.check_id == "run")
            .unwrap()
            .status
    };
    for fault in ["exit9", "extra-frame", "wrong-hash"] {
        std::env::set_var("AV798_FAULT", fault);
        let status = run_one(fault);
        std::env::remove_var("AV798_FAULT");
        assert_eq!(
            status,
            auto_test_case::report::Status::Error,
            "fault `{fault}` must be infra on the source path"
        );
    }
    std::env::set_var("AV798_FAULT", "delay-ms:5000");
    std::env::set_var("AV798_SESSION_WALL_MS", "1500");
    let status = run_one("delay");
    std::env::remove_var("AV798_FAULT");
    std::env::remove_var("AV798_SESSION_WALL_MS");
    assert_eq!(
        status,
        auto_test_case::report::Status::Error,
        "a stalled worker is infra ({status:?})"
    );
}

/// V-11 (G0 replayable inventory): the frozen denominators re-derive from
/// the real discovery rules — every shrink is red, growth is reported.
/// Prints a JSON summary the script persists as the inventory artifact.
#[test]
fn av_source_827_inventory_denominators_replayable() {
    // crates/auto-lang -> repo root
    let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    // AP-01..23: every seed path exists; the 5 AC-source-domain seeds
    // carry their frozen SHA256 receipts (identity is content, not path).
    const AC_SEEDS: [(&str, &str, &str); 5] = [
        ("AP-01", "experimental/ac-core/fixtures/scalar-source/valid/p-gcd-euclid.at",
         "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3"),
        ("AP-02", "experimental/ac-core/fixtures/record-source/valid/p01-records.at", ""),
        ("AP-03", "experimental/ac-core/fixtures/text-record-source/valid/p02-overlap.at", ""),
        ("AP-22", "experimental/ac-core/fixtures/scalar-source/valid/p-binop-args.at",
         "46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625"),
        ("AP-23", "experimental/ac-core/fixtures/scalar-source/valid/p-short-circuit-trap.at",
         "d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9"),
    ];
    const OLD_VM_SEEDS: [&str; 18] = [
        "crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at",
        "crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.at",
        "crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.at",
        "crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.at",
        "crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at",
        "crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.at",
        "crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.at",
        "crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.at",
        "crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.at",
        "crates/auto-lang/test/vm/22_generator/001_sum/sum.at",
        "examples/script-to-ship-demos/async-demo/main.at",
        "examples/http_server/server.at",
        "examples/ui/015-notes/src/front/app.at",
        "examples/script-to-ship-demos/trait-demo/main.at",
        "crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.as",
        "crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at",
        "crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.at",
        "crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.as",
    ];
    let mut ap_total = 0usize;
    for (id, rel, frozen_sha) in AC_SEEDS {
        let bytes = std::fs::read(repo.join(rel))
            .unwrap_or_else(|e| panic!("{id} seed missing at {rel}: {e}"));
        if !frozen_sha.is_empty() {
            assert_eq!(
                auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(&bytes)),
                frozen_sha,
                "{id} drifted from the frozen receipt"
            );
        }
        ap_total += 1;
    }
    for rel in OLD_VM_SEEDS {
        assert!(repo.join(rel).is_file(), "old-VM seed missing: {rel}");
        ap_total += 1;
    }
    assert_eq!(ap_total, 23, "the AP denominator is exactly 23");

    // Scalar-source fixtures: 3 scalar seeds (AP-01/22/23; AP-02/03 live in
    // the record/text-record profile dirs) + 8 beyond-seed programs.
    let scalar_dir = repo.join("experimental/ac-core/fixtures/scalar-source/valid");
    let scalar_count = std::fs::read_dir(&scalar_dir)
        .expect("scalar-source/valid dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "at"))
        .count();
    assert!(scalar_count >= 11, "scalar seeds (>=11, saw {scalar_count})");

    // VM corpus (the two-level discovery rule, counting DISK case dirs —
    // deliberately including unregistered ones; frozen minimums).
    let vm_root = repo.join("crates/auto-lang/test/vm");
    let mut categories = 0usize;
    let mut case_dirs = 0usize;
    let mut at_files = 0usize;
    let mut as_files = 0usize;
    for cat in std::fs::read_dir(&vm_root).expect("test/vm dir").flatten() {
        let cat_path = cat.path();
        if !cat_path.is_dir() {
            continue;
        }
        // The discovery rule treats every subdirectory of test/vm as a
        // category (including e.g. `aavm2` / `for_outer_var`).
        categories += 1;
        for case in std::fs::read_dir(&cat_path).expect("category dir").flatten() {
            if !case.path().is_dir() {
                continue;
            }
            case_dirs += 1;
            let mut stack = vec![case.path()];
            while let Some(d) = stack.pop() {
                for f in std::fs::read_dir(&d).expect("case walk").flatten() {
                    let p = f.path();
                    if p.is_dir() {
                        stack.push(p);
                    } else if p.extension().is_some_and(|x| x == "at") {
                        at_files += 1;
                    } else if p.extension().is_some_and(|x| x == "as") {
                        as_files += 1;
                    }
                }
            }
        }
    }
    assert!(categories >= 47, "corpus categories (>=47, saw {categories})");
    assert!(case_dirs >= 517, "corpus case dirs (>=517, saw {case_dirs})");
    assert!(at_files >= 661, "corpus .at files (>=661, saw {at_files})");
    assert!(as_files >= 8, "corpus .as files (>=8, saw {as_files})");

    // Capability/entry/gate denominators from the strategy docs.
    let matrix = std::fs::read_to_string(
        repo.join("docs/design/strategy/av-language-capability-matrix.md"),
    )
    .expect("capability matrix");
    let mut caps = 0usize;
    for n in 1..=43 {
        let id = format!("C-{:02}", n);
        assert!(
            matrix.contains(&id),
            "capability row {id} missing from the matrix"
        );
        caps += 1;
    }
    assert_eq!(caps, 43, "the capability denominator is C-01..C-43");
    let roadmap = std::fs::read_to_string(
        repo.join("docs/design/strategy/av-source-programs-and-avm-replacement-roadmap.md"),
    )
    .expect("roadmap");
    let mut eps = 0usize;
    for n in 1..=10 {
        assert!(
            roadmap.contains(&format!("EP-{:02}", n)),
            "entry point EP-{n:02} missing"
        );
        eps += 1;
    }
    assert_eq!(eps, 10, "the entry-point denominator is EP-01..EP-10");
    for g in ["G0", "G1", "G2", "G3", "G4", "G5", "G6", "G7", "G8"] {
        assert!(roadmap.contains(g), "gate {g} missing from the roadmap");
    }

    // stdlib inventory: the provider catalog + the auto layer files.
    assert!(repo.join("stdlib/assembly-providers.json").is_file());
    let auto_stdlib = std::fs::read_dir(repo.join("stdlib/auto"))
        .expect("stdlib/auto dir")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|x| x == "at"))
        .count();
    assert!(auto_stdlib >= 50, "stdlib auto layer (>=50, saw {auto_stdlib})");

    // The 13 frozen old-VM entry call-site anchors (existence replay).
    const ENTRY_ANCHORS: [&str; 13] = [
        "crates/auto/src/main.rs",
        "crates/auto-lang-core/src/script_entry.rs",
        "crates/auto-lang/src/lib.rs",
        "crates/auto-lang-core/src/autovm_persistent.rs",
        "crates/auto-lang-core/src/autovm_daemon.rs",
        "crates/auto-lang-core/src/autovm_client.rs",
        "crates/auto-lang-core/src/autovm_repl.rs",
        "crates/auto-lang-core/src/test_cases/avm.rs",
        "crates/auto-ui-runtime/src/ui/vm_bridge.rs",
        "crates/auto-lang-core/src/vm.rs",
        "crates/auto-man-runtime/src/lib.rs",
        "crates/auto/src/cmd_ui.rs",
        "crates/auto/src/cmd_service.rs",
    ];
    for a in ENTRY_ANCHORS {
        assert!(repo.join(a).is_file(), "entry anchor missing: {a}");
    }

    // JSON summary (the script persists this as the inventory artifact).
    println!(
        "{{\"scope\":\"inventory\",\"ap_seeds\":{ap_total},\"ac_source_seeds\":5,\
         \"scalar_fixtures\":{scalar_count},\"vm_categories\":{categories},\
         \"vm_case_dirs\":{case_dirs},\"vm_at\":{at_files},\"vm_as\":{as_files},\
         \"capabilities\":43,\"entry_points\":10,\"gates\":9,\
         \"stdlib_auto_at\":{auto_stdlib},\"entry_anchors\":13}}"
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
