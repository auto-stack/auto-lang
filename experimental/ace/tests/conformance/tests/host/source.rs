//! PLAN-827: the av-source-827 corpus — REAL Auto source files (byte-
//! identical copies of the AP seeds, identity receipts in the corpus
//! README) through the formal `ac-av/core-run-v1` recipe, judged
//! PARENT-SIDE by the unified runner. Mirrors the ac-av corpus gate; the
//! recipe/worker/protocol are reused unchanged (T-01 reuse ruling — no
//! new recipe, worker command, or workspace).

use crate::test_cases::run_suite;
use auto_test_case::admission::Limits;
use std::path::{Path, PathBuf};

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
fn av_source_corpus_green() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());

    let suite_dir =
        PathBuf::from(env!("AUTO_LANG_HOST_DIR")).join("../auto-lang/test/cases/av-source-827");
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
    let ids: Vec<&str> = report.results.iter().map(|r| r.check_id.as_str()).collect();
    // R1-F01 duplicate control: a check id may appear exactly ONCE — a
    // duplicated row would double-count the required population.
    let mut sorted_ids = ids.clone();
    sorted_ids.sort_unstable();
    sorted_ids.dedup();
    assert_eq!(
        sorted_ids.len(),
        ids.len(),
        "duplicated check ids in the report: {ids:?}"
    );
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
        assert!(
            ids.contains(&required),
            "required check {required}: {ids:?}"
        );
    }
}

/// V-04 admission negatives at the RUNNER level (crafted temp suites): an
/// invalid-UTF-8 source is an INFRA/resolve failure — never a guest trap
/// and never a green language verdict; a missing source file is a resolve
/// error naming the path.
#[test]
fn av_source_admission_invalid_utf8_is_infra_not_trap() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("utf8");
    // A source file with an invalid UTF-8 byte: resolution carries BYTES
    // (identity is content-based); the worker rejects the encoding, which
    // the parent admits as an INFRA outcome — distinct from any trap.
    std::fs::write(
        root.join("main.at"),
        b"fn main() int {\n    return \xff\xfe\n}\n",
    )
    .unwrap();
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
fn av_source_admission_missing_file_is_resolve_error() {
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
fn av_source_negative_oracle_fails() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    let root = temp_root("neg");
    let ap01 = std::fs::read(
        PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
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
    let report =
        run_suite(&root.join("neg.suite.ad"), &out, &Limits::defaults()).expect("suite runs");
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
fn av_source_worker_faults_are_infra() {
    let _guard = crate::tests::av_protocol::AV_CASE_LOCK
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
            PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
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

// ---------------------------------------------------------------------------
// G0 item-level inventory (V-11 / R1-F03): durable per-item discovery with
// owner/status/gap retained as ROWS (scan failures are rows, never silent
// drops), reconciled against the frozen denominators, persisted as JSON.
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, Debug)]
struct InvItem {
    id: String,
    kind: &'static str,
    path: String,
    sha256: String,
    status: String,
    owner: String,
    note: String,
}

fn inv_error(id: &str, kind: &'static str, path: &str, msg: &str) -> InvItem {
    InvItem {
        id: id.to_string(),
        kind,
        path: path.to_string(),
        sha256: String::new(),
        status: "scan-error".into(),
        owner: "-".into(),
        note: msg.to_string(),
    }
}

fn sha256_of(bytes: &[u8]) -> String {
    auto_test_case::sha256::hex(&auto_test_case::sha256::sha256(bytes))
}

fn scan_file(repo: &Path, rel: &str, id: &str, kind: &'static str) -> InvItem {
    let full = repo.join(rel);
    match std::fs::read(&full) {
        Ok(bytes) => InvItem {
            id: id.to_string(),
            kind,
            path: rel.to_string(),
            sha256: sha256_of(&bytes),
            status: "present".into(),
            owner: "-".into(),
            note: format!("{} bytes", bytes.len()),
        },
        Err(e) => inv_error(id, kind, rel, &format!("read failed: {e}")),
    }
}

/// One AP seed: file presence + SHA + oracle sidecar probe + frozen
/// nature/owner/status from the catalog (hand-audited at T-01/T-02).
fn scan_ap_seed(
    repo: &Path,
    id: &str,
    rel: &str,
    nature: &str,
    owner: &str,
    status: &str,
) -> InvItem {
    let mut item = scan_file(repo, rel, id, "ap-seed");
    item.note = format!("nature={nature}");
    if item.status == "present" {
        // Oracle sidecar probe (uniform rule): .expected.out / .expected.result.
        let out = repo.join(format!("{rel}.expected.out")).is_file();
        let result = repo.join(format!("{rel}.expected.result")).is_file();
        let oracle = if out {
            "expected.out"
        } else if result {
            "expected.result"
        } else {
            "none"
        };
        item.status = status.to_string();
        item.owner = owner.to_string();
        item.note = format!("nature={nature}; oracle={oracle}");
    }
    item
}

/// Scan one directory of .at fixtures; enumeration failures are rows.
fn scan_fixture_dir(repo: &Path, rel_dir: &str, kind: &'static str, owner: &str) -> Vec<InvItem> {
    let full = repo.join(rel_dir);
    let entries = match std::fs::read_dir(&full) {
        Ok(rd) => rd.collect::<Vec<_>>(),
        Err(e) => {
            return vec![inv_error(
                rel_dir,
                kind,
                rel_dir,
                &format!("read_dir failed: {e}"),
            )]
        }
    };
    let mut files: Vec<String> = Vec::new();
    for e in entries {
        match e {
            Ok(e) => {
                let p = e.path();
                if p.extension().is_some_and(|x| x == "at") {
                    files.push(e.file_name().to_string_lossy().to_string());
                }
            }
            Err(err) => {
                files.push(format!("__enumerate-error:{err}"));
            }
        }
    }
    files.sort();
    files
        .into_iter()
        .map(|name| {
            if let Some(err) = name.strip_prefix("__enumerate-error:") {
                return inv_error(
                    &format!("{rel_dir}/{name}"),
                    kind,
                    rel_dir,
                    &format!("entry error: {err}"),
                );
            }
            let rel = format!("{rel_dir}/{name}");
            let mut item = scan_file(repo, &rel, &rel, kind);
            if item.status == "present" {
                // Beyond-seed fixtures have no committed oracle: gap rows.
                let seed = matches!(
                    name.as_str(),
                    "p-gcd-euclid.at" | "p-binop-args.at" | "p-short-circuit-trap.at"
                ) || rel.contains("record-source")
                    || rel.contains("text-record-source");
                item.status = if seed {
                    "seed".into()
                } else {
                    "gap-no-oracle".into()
                };
                item.owner = owner.to_string();
            }
            item
        })
        .collect()
}

/// The VM corpus (two-level discovery rule): one row per case dir, with
/// its source kind and oracle sidecar state. No sidecar = gap row.
fn scan_vm_corpus(repo: &Path, vm_rel: &str) -> Vec<InvItem> {
    let root = repo.join(vm_rel);
    let mut items = Vec::new();
    let cats = match std::fs::read_dir(&root) {
        Ok(rd) => rd.collect::<Vec<_>>(),
        Err(e) => {
            return vec![inv_error(
                vm_rel,
                "vm-corpus",
                vm_rel,
                &format!("read_dir failed: {e}"),
            )]
        }
    };
    let mut cat_names: Vec<String> = Vec::new();
    for c in cats {
        match c {
            Ok(c) if c.path().is_dir() => {
                cat_names.push(c.file_name().to_string_lossy().to_string())
            }
            Ok(_) => {}
            Err(err) => items.push(inv_error(
                &format!("{vm_rel}/__cat"),
                "vm-corpus",
                vm_rel,
                &format!("category entry error: {err}"),
            )),
        }
    }
    cat_names.sort();
    for cat in cat_names {
        let cat_dir = root.join(&cat);
        let cases = match std::fs::read_dir(&cat_dir) {
            Ok(rd) => rd.collect::<Vec<_>>(),
            Err(e) => {
                items.push(inv_error(
                    &format!("{vm_rel}/{cat}"),
                    "vm-corpus",
                    &format!("{vm_rel}/{cat}"),
                    &format!("read_dir failed: {e}"),
                ));
                continue;
            }
        };
        let mut case_names: Vec<String> = Vec::new();
        for c in cases {
            match c {
                Ok(c) if c.path().is_dir() => {
                    case_names.push(c.file_name().to_string_lossy().to_string())
                }
                Ok(_) => {}
                Err(err) => items.push(inv_error(
                    &format!("{vm_rel}/{cat}/__case"),
                    "vm-corpus",
                    &format!("{vm_rel}/{cat}"),
                    &format!("case entry error: {err}"),
                )),
            }
        }
        // Category-level row: the directory denominator (a category with
        // only loose files — e.g. for_outer_var/main.at — still counts).
        items.push(InvItem {
            id: format!("{vm_rel}/{cat}"),
            kind: "vm-corpus-category",
            path: format!("{vm_rel}/{cat}"),
            sha256: String::new(),
            status: format!("cases={}", case_names.len()),
            owner: "vm-corpus".into(),
            note: String::new(),
        });
        case_names.sort();
        for case in case_names {
            let case_dir = cat_dir.join(&case);
            let name = case
                .split_once('_')
                .map(|(_, rest)| rest)
                .unwrap_or(&case)
                .to_string();
            let at_path = case_dir.join(format!("{name}.at"));
            let as_path = case_dir.join(format!("{name}.as"));
            let out_path = case_dir.join(format!("{name}.expected.out"));
            let result_path = case_dir.join(format!("{name}.expected.result"));
            let error_path = case_dir.join(format!("{name}.expected.error"));
            let (at, asf) = (at_path.is_file(), as_path.is_file());
            let (out, result, errorf) = (
                out_path.is_file(),
                result_path.is_file(),
                error_path.is_file(),
            );
            // R2-F03: bind the SOURCE identity (bytes hashed — a later edit
            // changes the row) and the ORACLE sidecar identity; read
            // failures are RETAINED rows, never silent drops.
            let src_path = if at {
                Some(at_path)
            } else if asf {
                Some(as_path)
            } else {
                None
            };
            let (src_sha, src_err) = match &src_path {
                Some(p) => match std::fs::read(p) {
                    Ok(b) => (sha256_of(&b), None),
                    Err(e) => (String::new(), Some(format!("source read failed: {e}"))),
                },
                None => (String::new(), None),
            };
            let oracle_path = if out {
                Some(out_path)
            } else if result {
                Some(result_path)
            } else if errorf {
                Some(error_path)
            } else {
                None
            };
            let (oracle_sha, oracle_err) = match &oracle_path {
                Some(p) => match std::fs::read(p) {
                    Ok(b) => (sha256_of(&b), None),
                    Err(e) => (String::new(), Some(format!("oracle read failed: {e}"))),
                },
                None => (String::new(), None),
            };
            let source_kind = if at {
                "at"
            } else if asf {
                "as"
            } else {
                "none"
            };
            let oracle = if out {
                "expected.out"
            } else if result {
                "expected.result"
            } else if errorf {
                "expected.error"
            } else {
                "none"
            };
            if let Some(err) = src_err.or(oracle_err) {
                // Identity read failure: retained scan-error row.
                items.push(inv_error(
                    &format!("{vm_rel}/{cat}/{case}"),
                    "vm-corpus-case",
                    &format!("{vm_rel}/{cat}/{case}"),
                    &err,
                ));
                continue;
            }
            // Disk rows include deliberately-unregistered cases (the
            // in-repo caveat) — the denominator counts disk, the status
            // marks the oracle state without deleting the row. sha256 =
            // the SOURCE identity; the oracle identity rides the note.
            items.push(InvItem {
                id: format!("{vm_rel}/{cat}/{case}"),
                kind: "vm-corpus-case",
                path: format!("{vm_rel}/{cat}/{case}"),
                sha256: src_sha,
                status: format!("source={source_kind};oracle={oracle}"),
                owner: if oracle == "none" {
                    "future-G".into()
                } else {
                    "vm-corpus".into()
                },
                note: if oracle == "none" {
                    "gap: no oracle sidecar".into()
                } else {
                    format!("oracle_sha256={oracle_sha}")
                },
            });
        }
    }
    items
}

/// stdlib provider declarations (PLAN-738): one row per module x target
/// with its declared status; parse failures are rows, never silent.
fn scan_providers(repo: &Path, rel: &str) -> Vec<InvItem> {
    let bytes = match std::fs::read(repo.join(rel)) {
        Ok(b) => b,
        Err(e) => {
            return vec![inv_error(
                "providers",
                "stdlib-provider",
                rel,
                &format!("read failed: {e}"),
            )]
        }
    };
    let value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(e) => {
            return vec![inv_error(
                "providers",
                "stdlib-provider",
                rel,
                &format!("parse failed: {e}"),
            )]
        }
    };
    let Some(arr) = value.get("providers").and_then(|v| v.as_array()) else {
        return vec![inv_error(
            "providers",
            "stdlib-provider",
            rel,
            "no `providers` array in the catalog",
        )];
    };
    arr.iter()
        .enumerate()
        .map(|(i, p)| {
            let module = p.get("module").and_then(|v| v.as_str()).unwrap_or("?");
            let target = p.get("target").and_then(|v| v.as_str()).unwrap_or("?");
            let status = p.get("status").and_then(|v| v.as_str()).unwrap_or("?");
            InvItem {
                id: format!("provider:{module}:{target}"),
                kind: "stdlib-provider",
                path: rel.to_string(),
                sha256: String::new(),
                status: status.to_string(),
                owner: "PLAN-738".into(),
                note: p
                    .get("reason")
                    .or_else(|| p.get("notes"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }
            .with_index(i)
        })
        .collect()
}

trait WithIndex {
    fn with_index(self, _: usize) -> InvItem;
}
impl WithIndex for InvItem {
    fn with_index(mut self, i: usize) -> InvItem {
        if self.id.contains(':') && self.id.ends_with("?") {
            self.id = format!("provider[{i}]");
        }
        self
    }
}

/// R2-F03: stdlib/auto FILE members — every `.at` layer file is a durable
/// row (public layer / `.vm.at` VM shim / `.rs.at` Rust mirror / `.c.at`
/// C layer), bytes hashed; enumeration/read failures are retained rows.
/// These are the actual stdlib denominator members the provider rows
/// only declare ownership over.
fn scan_stdlib_auto(repo: &Path, rel_dir: &str) -> Vec<InvItem> {
    let full = repo.join(rel_dir);
    let entries = match std::fs::read_dir(&full) {
        Ok(rd) => rd.collect::<Vec<_>>(),
        Err(e) => {
            return vec![inv_error(
                rel_dir,
                "stdlib-auto-file",
                rel_dir,
                &format!("read_dir failed: {e}"),
            )]
        }
    };
    let mut names: Vec<String> = Vec::new();
    for e in entries {
        match e {
            Ok(e) => {
                if e.path().extension().is_some_and(|x| x == "at") {
                    names.push(e.file_name().to_string_lossy().to_string());
                }
            }
            Err(err) => {
                names.push(format!("__enumerate-error:{err}"));
            }
        }
    }
    names.sort();
    names
        .into_iter()
        .map(|name| {
            if let Some(err) = name.strip_prefix("__enumerate-error:") {
                return inv_error(
                    &format!("{rel_dir}/{name}"),
                    "stdlib-auto-file",
                    rel_dir,
                    &format!("entry error: {err}"),
                );
            }
            let rel = format!("{rel_dir}/{name}");
            let layer = if name.ends_with(".vm.at") {
                "vm-shim"
            } else if name.ends_with(".rs.at") {
                "rust-mirror"
            } else if name.ends_with(".c.at") {
                "c-layer"
            } else {
                "public"
            };
            match std::fs::read(repo.join(&rel)) {
                Ok(bytes) => InvItem {
                    id: format!("stdlib-auto/{name}"),
                    kind: "stdlib-auto-file",
                    path: rel,
                    sha256: sha256_of(&bytes),
                    status: format!("layer={layer}"),
                    owner: "PLAN-738".into(),
                    note: format!("{} bytes", bytes.len()),
                },
                Err(e) => inv_error(&rel, "stdlib-auto-file", &rel, &format!("read failed: {e}")),
            }
        })
        .collect()
}

/// Markdown table rows `| <ID> | ...` for a fixed id prefix — per-row
/// parse with owner extraction; a missing row becomes a generated gap row
/// so the denominator cannot shrink silently.
fn scan_md_rows(
    text: &str,
    prefix: &str,
    count: usize,
    kind: &'static str,
    owner_col: usize,
    gate_col: usize,
) -> Vec<InvItem> {
    let mut by_id: std::collections::BTreeMap<String, InvItem> = Default::default();
    for line in text.lines() {
        let t = line.trim();
        if !t.starts_with(&format!("| {prefix}")) {
            continue;
        }
        let cells: Vec<&str> = t.split('|').collect();
        if cells.len() < 3 {
            continue;
        }
        let id = cells[1].trim().to_string();
        if !id.starts_with(prefix) {
            continue;
        }
        // Gate rows merge id+name in one cell ("G0 库存与差异基线") —
        // normalize to the bare id; a table header like "Gate ID" is not
        // a gate row (the id must be G<digit>...).
        let id = id.split_whitespace().next().unwrap_or(&id).to_string();
        if prefix == "G" && !id.as_bytes().get(1).is_some_and(|b| b.is_ascii_digit()) {
            continue;
        }
        let owner = cells
            .get(owner_col)
            .map(|c| c.trim().to_string())
            .unwrap_or_default();
        let gate = cells
            .get(gate_col)
            .map(|c| c.trim().to_string())
            .unwrap_or_default();
        let name = cells
            .get(2)
            .map(|c| c.trim().to_string())
            .unwrap_or_default();
        by_id.insert(
            id.clone(),
            InvItem {
                id,
                kind,
                path: String::new(),
                sha256: String::new(),
                status: "declared".into(),
                owner,
                note: format!("gate={gate}; {name}"),
            },
        );
    }
    // Denominator completion: every expected id must exist — missing ones
    // are RETAINED as gap rows (and the assertion below turns them red).
    let mut items: Vec<InvItem> = by_id.into_values().collect();
    let expected_ids: Vec<String> = if prefix == "G" {
        // The roadmap splits G2 into G2a/G2b — the frozen gate denominator.
        ["G0", "G1", "G2a", "G2b", "G3", "G4", "G5", "G6", "G7", "G8"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        (1..=count).map(|n| format!("{prefix}{n:02}")).collect()
    };
    for id in expected_ids {
        if !items.iter().any(|it| it.id == id) {
            items.push(InvItem {
                id,
                kind,
                path: String::new(),
                sha256: String::new(),
                status: "gap-missing-row".into(),
                owner: "-".into(),
                note: "expected row absent from the document".into(),
            });
        }
    }
    items
}

/// Entry call-site anchors WITH symbol discovery: the row is green only
/// when the file exists AND carries its characteristic call-site symbol.
fn scan_entry_anchors(repo: &Path) -> Vec<InvItem> {
    const ANCHORS: [(&str, &str, &str); 7] = [
        (
            "entry:script-entry",
            "crates/auto-lang-core/src/script_entry.rs",
            "run_file",
        ),
        (
            "entry:sdk-staged",
            "crates/auto-lang/src/lib.rs",
            "run_with_capture_and_path_staged",
        ),
        (
            "entry:persistent-vm",
            "crates/auto-lang-core/src/autovm_persistent.rs",
            "AutovmReplSession",
        ),
        (
            "entry:daemon",
            "crates/auto-lang-core/src/autovm_daemon.rs",
            "AutovmDaemon",
        ),
        (
            "entry:client",
            "crates/auto-lang-core/src/autovm_client.rs",
            "eval_one_shot",
        ),
        (
            "entry:testcase-avm-adapter",
            "crates/auto-lang-core/src/test_cases/avm.rs",
            "Verdict",
        ),
        (
            "entry:old-vm-engine",
            "crates/auto-lang-core/src/vm.rs",
            "register_module",
        ),
    ];
    ANCHORS
        .iter()
        .map(
            |(id, rel, symbol)| match std::fs::read_to_string(repo.join(rel)) {
                Ok(text) => {
                    let found = text.contains(symbol);
                    InvItem {
                        id: id.to_string(),
                        kind: "entry-call-site",
                        path: rel.to_string(),
                        sha256: String::new(),
                        status: if found {
                            "symbol-found".into()
                        } else {
                            "symbol-missing".into()
                        },
                        owner: "EP-anchor".into(),
                        note: format!("symbol `{symbol}`"),
                    }
                }
                Err(e) => inv_error(id, "entry-call-site", rel, &format!("read failed: {e}")),
            },
        )
        .collect()
}

/// PLAN-831 moved the ui-vm-bridge call-site file out of this repository
/// (abd34a355): the member STAYS in the denominator as a moved-external row
/// with its new owner instead of a scan attempt on the removed path (the
/// merge integration fix for the frozen R2 anchor list). PLAN-832 moved the
/// CLI crate (`crates/auto`), its service command and `auto-man-runtime` to
/// the independent auto-man repo: those anchors follow the same moved-external
/// convention (the denominator keeps all 15 members; a lang-only checkout has
/// no auto-man sibling, so the rows carry the new owner instead of a read).
fn moved_entry_rows() -> Vec<InvItem> {
    let moved_external = |id: &str, path: &str, symbol: &str, owner: &str| InvItem {
        id: id.to_string(),
        kind: "entry-call-site",
        path: path.to_string(),
        sha256: String::new(),
        status: "moved-external".to_string(),
        owner: owner.to_string(),
        note: format!(
            "symbol `{symbol}` (file moved to the independent {} repo)",
            owner.split('/').next_back().unwrap_or(owner)
        ),
    };
    vec![
        InvItem {
            id: "entry:ui-vm-bridge".to_string(),
            kind: "entry-call-site",
            path: "auto-ui:crates/auto-ui-runtime/src/ui/vm_bridge.rs".to_string(),
            sha256: String::new(),
            status: "moved-external".to_string(),
            owner: "PLAN-831/auto-ui".to_string(),
            note: "symbol `vm_bridge` (file moved to the independent auto-ui repo)".to_string(),
        },
        moved_external(
            "entry:cli-run",
            "auto-man:crates/auto/src/main.rs",
            "Commands::Run",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:cli-script-positional",
            "auto-man:crates/auto/src/main.rs",
            "run_file_with_args",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:cli-eval",
            "auto-man:crates/auto/src/main.rs",
            "Commands::Eval",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:cli-old-repl",
            "auto-man:crates/auto/src/main.rs",
            "autovm_repl",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:cli-mcp-daemon",
            "auto-man:crates/auto/src/main.rs",
            "AutovmDaemon",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:cli-service",
            "auto-man:crates/auto/src/cmd_service.rs",
            "cmd_service",
            "PLAN-832/auto-man",
        ),
        moved_external(
            "entry:man-runtime",
            "auto-man:crates/auto-man-runtime/src/lib.rs",
            "Automan",
            "PLAN-832/auto-man",
        ),
    ]
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("AUTO_LANG_HOST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

const AP_SEEDS: [(&str, &str, &str, &str, &str); 23] = [
    (
        "AP-01",
        "experimental/ace/crates/core/fixtures/scalar-source/valid/p-gcd-euclid.at",
        "ac-source",
        "827",
        "supported-G1",
    ),
    (
        "AP-02",
        "experimental/ace/crates/core/fixtures/record-source/valid/p01-records.at",
        "ac-source",
        "G2a",
        "gap",
    ),
    (
        "AP-03",
        "experimental/ace/crates/core/fixtures/text-record-source/valid/p02-overlap.at",
        "ac-source",
        "G2a",
        "gap",
    ),
    (
        "AP-04",
        "crates/auto-lang/test/vm/09_functions/013_closure_capture_hof/closure_capture_hof.at",
        "vm-corpus",
        "G2b",
        "gap",
    ),
    (
        "AP-05",
        "crates/auto-lang/test/vm/09_functions/015_list_chain_pipeline/list_chain_pipeline.at",
        "vm-corpus",
        "G2b",
        "gap",
    ),
    (
        "AP-06",
        "crates/auto-lang/test/vm/16_option_result/022_result_multi_error/result_multi_error.at",
        "vm-corpus",
        "G2b",
        "gap",
    ),
    (
        "AP-07",
        "crates/auto-lang/test/vm/17_modules/001_use_fn/use_fn.at",
        "vm-corpus",
        "G3",
        "gap",
    ),
    (
        "AP-08",
        "crates/auto-lang/test/vm/24_generics/001_list_struct/list_struct.at",
        "vm-corpus",
        "G2b",
        "gap",
    ),
    (
        "AP-09",
        "crates/auto-lang/test/vm/18_ffi/038_json_encode_parse/json_encode_parse.at",
        "vm-corpus",
        "G3",
        "gap",
    ),
    (
        "AP-10",
        "crates/auto-lang/test/vm/18_ffi/051_file_write_read/file_write_read.at",
        "vm-corpus",
        "G4",
        "gap",
    ),
    (
        "AP-11",
        "crates/auto-lang/test/vm/18_ffi/055_process_args/process_args.at",
        "vm-corpus",
        "G4",
        "gap",
    ),
    (
        "AP-12",
        "crates/auto-lang/test/vm/23_actor/006_state_increment/state_increment.at",
        "vm-corpus",
        "G5",
        "gap",
    ),
    (
        "AP-13",
        "crates/auto-lang/test/vm/22_generator/001_sum/sum.at",
        "vm-corpus",
        "G5",
        "gap",
    ),
    (
        "AP-14",
        "examples/script-to-ship-demos/async-demo/main.at",
        "example-app",
        "G5",
        "gap",
    ),
    (
        "AP-15",
        "examples/http_server/server.at",
        "example-app",
        "G5",
        "gap",
    ),
    (
        "AP-16",
        "examples/ui/015-notes/src/front/app.at",
        "example-ui",
        "G6",
        "gap",
    ),
    (
        "AP-17",
        "examples/script-to-ship-demos/trait-demo/main.at",
        "example-app",
        "G3",
        "gap",
    ),
    (
        "AP-18",
        "crates/auto-lang/test/vm/99_script_err/05_obj_call_auto_catch/obj_call_auto_catch.as",
        "script-sugar",
        "G6",
        "gap-witness",
    ),
    (
        "AP-19",
        "crates/auto-lang/test/vm/07_objects/010_map_bracket_write/map_bracket_write.at",
        "vm-corpus",
        "G3",
        "gap",
    ),
    (
        "AP-20",
        "crates/auto-lang/test/vm/18_ffi/056_math_pow/math_pow.at",
        "vm-corpus",
        "G3",
        "gap",
    ),
    (
        "AP-21",
        "crates/auto-lang/test/vm/99_py_dispatch/01_dual_len/dual_len.as",
        "script-sugar",
        "G6",
        "gap-witness",
    ),
    (
        "AP-22",
        "experimental/ace/crates/core/fixtures/scalar-source/valid/p-binop-args.at",
        "ac-source",
        "827",
        "supported-G1",
    ),
    (
        "AP-23",
        "experimental/ace/crates/core/fixtures/scalar-source/valid/p-short-circuit-trap.at",
        "ac-source",
        "827",
        "supported-G1",
    ),
];

/// V-11 (G0 replayable ITEM inventory, R1-F03): every denominator member
/// is a durable row with identity/status/owner; failures stay as rows;
/// counts reconcile with the frozen denominators; the JSON artifact is
/// persisted under docs/reports/plan827/inventory.json.
#[test]
fn av_source_inventory_denominators_replayable() {
    let repo = repo_root();
    let mut items: Vec<InvItem> = Vec::new();

    // 1) AP seeds: 23 rows, frozen receipts for the three G1 seeds.
    for (id, rel, nature, owner, status) in AP_SEEDS {
        items.push(scan_ap_seed(&repo, id, rel, nature, owner, status));
    }
    assert_eq!(
        items.iter().filter(|i| i.kind == "ap-seed").count(),
        23,
        "the AP denominator is exactly 23"
    );
    let row_of = |id: &str| -> (String, String) {
        let it = items
            .iter()
            .find(|i| i.id == id)
            .unwrap_or_else(|| panic!("{id} row"));
        (it.sha256.clone(), it.note.clone())
    };
    let (sha, _) = row_of("AP-01");
    assert_eq!(
        sha,
        "6d466c03e1b0012f7ac9b883cf88f253345fd911ac3ced2d1f282ce042799af3"
    );
    let (sha, _) = row_of("AP-22");
    assert_eq!(
        sha,
        "46d6a431a5c1b9bbe6e2e87b8732c09a9c584b58eb92799fdb4ab073f423e625"
    );
    let (sha, _) = row_of("AP-23");
    assert_eq!(
        sha,
        "d6871b77c9b02e797c77a2cf34900e005fe97062bf1fc95ffcdf1cf7707052a9"
    );
    // AP-11 is the no-oracle gap row the catalog flags (argv oracle TBD).
    let (_, note) = row_of("AP-11");
    assert!(note.contains("oracle=none"), "AP-11: {note}");

    // 2) AC-source fixture dirs (all three profiles).
    for (rel, owner) in [
        (
            "experimental/ace/crates/core/fixtures/scalar-source/valid",
            "827/G2",
        ),
        (
            "experimental/ace/crates/core/fixtures/record-source/valid",
            "G2a",
        ),
        (
            "experimental/ace/crates/core/fixtures/text-record-source/valid",
            "G2a",
        ),
    ] {
        items.extend(scan_fixture_dir(&repo, rel, "ac-fixture", owner));
    }
    let fixtures = items.iter().filter(|i| i.kind == "ac-fixture").count();
    assert!(fixtures >= 14, "ac fixtures (>=14, saw {fixtures})");

    // 3) VM corpus: every disk case dir is a row (registered or not).
    items.extend(scan_vm_corpus(&repo, "crates/auto-lang/test/vm"));
    let corpus = items.iter().filter(|i| i.kind == "vm-corpus-case").count();
    assert!(corpus >= 517, "corpus case rows (>=517, saw {corpus})");
    let cats = items
        .iter()
        .filter(|i| i.kind == "vm-corpus-category")
        .count();
    assert!(cats >= 47, "corpus category rows (>=47, saw {cats})");

    // 4) stdlib provider declarations + the FILE members they own (R2-F03:
    //    the stdlib denominator is the on-disk layer population, not only
    //    the 37 declaration rows).
    items.extend(scan_providers(&repo, "stdlib/assembly-providers.json"));
    let providers = items.iter().filter(|i| i.kind == "stdlib-provider").count();
    assert!(
        providers >= 18,
        "provider rows (>=18 six-core 3-target grid, saw {providers})"
    );
    items.extend(scan_stdlib_auto(&repo, "stdlib/auto"));
    let stdlib_files = items
        .iter()
        .filter(|i| i.kind == "stdlib-auto-file")
        .count();
    assert!(
        stdlib_files >= 102,
        "stdlib/auto file member rows (>=102, saw {stdlib_files})"
    );

    // 5) Capability matrix C-01..C-43 (per-row owner/gate extraction).
    let matrix =
        std::fs::read_to_string(repo.join("docs/design/strategy/av-language-capability-matrix.md"))
            .expect("capability matrix");
    items.extend(scan_md_rows(&matrix, "C-", 43, "capability", 6, 8));
    // 6) Roadmap EP-01..10 + G0..G8.
    let roadmap = std::fs::read_to_string(
        repo.join("docs/design/strategy/av-source-programs-and-avm-replacement-roadmap.md"),
    )
    .expect("roadmap");
    items.extend(scan_md_rows(&roadmap, "EP-", 10, "entry-point", 2, 2));
    items.extend(scan_md_rows(&roadmap, "G", 10, "gate", 3, 3));

    // 7) Old-VM entry call-site anchors WITH symbol discovery.
    items.extend(scan_entry_anchors(&repo));
    items.extend(moved_entry_rows());

    // Reconciliation: no scan errors, no missing rows, anchors all found.
    let errors: Vec<&InvItem> = items.iter().filter(|i| i.status == "scan-error").collect();
    assert!(
        errors.is_empty(),
        "scan failures must be zero (they would be retained rows): {errors:?}"
    );
    let missing: Vec<&InvItem> = items
        .iter()
        .filter(|i| i.status == "gap-missing-row" || i.status == "symbol-missing")
        .collect();
    assert!(
        missing.is_empty(),
        "missing denominator rows / symbols: {missing:?}"
    );
    // R2-F03 identity completeness: every discovered SOURCE-bearing row
    // carries real bytes identity (an edit changes the row; a vanished
    // file becomes a scan-error row above).
    let identityless: Vec<&InvItem> = items
        .iter()
        .filter(|i| {
            (i.kind == "vm-corpus-case" && i.sha256.is_empty() && i.status.contains("source=at")
                || i.kind == "vm-corpus-case"
                    && i.sha256.is_empty()
                    && i.status.contains("source=as"))
                || (i.kind == "stdlib-auto-file" && i.sha256.is_empty())
                || (i.kind == "ac-fixture" && i.sha256.is_empty() && i.status == "present")
                || (i.kind == "ap-seed" && i.sha256.is_empty())
        })
        .collect();
    assert!(
        identityless.is_empty(),
        "source-bearing rows without byte identity: {identityless:?}"
    );
    let caps = items.iter().filter(|i| i.kind == "capability").count();
    assert_eq!(caps, 43, "capability denominator");
    let eps = items.iter().filter(|i| i.kind == "entry-point").count();
    assert_eq!(eps, 10, "entry-point denominator");
    let gates = items.iter().filter(|i| i.kind == "gate").count();
    assert_eq!(gates, 10, "gate denominator (G0..G8 with G2a/G2b split)");
    let anchors = items
        .iter()
        .filter(|i| i.kind == "entry-call-site" && i.status == "symbol-found")
        .count();
    assert_eq!(
        anchors, 7,
        "all 7 in-repo call-site anchors verified with symbols (CLI/service/man moved to auto-man by PLAN-832)"
    );
    // PLAN-831 moved the 15th (ui-vm-bridge) to the independent auto-ui repo
    // and PLAN-832 moved the CLI/service/man-runtime anchors to the
    // independent auto-man repo: the denominator keeps all of them as
    // moved-external owner rows.
    let moved = items
        .iter()
        .filter(|i| i.status == "moved-external")
        .count();
    assert_eq!(
        moved, 8,
        "the moved-out call-sites stay in the denominator (1×831 auto-ui + 7×832 auto-man)"
    );
    // Every capability row carries a real owner cell.
    for it in items.iter().filter(|i| i.kind == "capability") {
        assert!(
            !it.owner.is_empty(),
            "capability {} has no owner cell: {:?}",
            it.id,
            it.note
        );
    }

    // Persist the durable item artifact + the summary line.
    let artifact = serde_json::json!({
        "scope": "inventory",
        "repo_head": option_env!("AV827_INV_HEAD").unwrap_or("-"),
        "items": items,
        "summary": {
            "ap_seeds": 23,
            "ac_fixtures": fixtures,
            "vm_corpus_cases": corpus,
            "vm_categories": cats,
            "stdlib_providers": providers,
            "stdlib_auto_files": stdlib_files,
            "capabilities": caps,
            "entry_points": eps,
            "gates": gates,
            "entry_anchors": anchors,
        },
    });
    let out_path = repo
        .join("docs/reports/plan827/inventory.json")
        .to_string_lossy()
        .to_string();
    std::fs::write(
        &out_path,
        serde_json::to_string_pretty(&artifact).expect("serialize inventory"),
    )
    .unwrap_or_else(|e| panic!("write inventory artifact {out_path}: {e}"));
    println!(
        "inventory: {} items (23 AP / {fixtures} fixtures / {corpus} corpus cases / {providers} providers / {stdlib_files} stdlib files / 43 C / 10 EP / {gates} G / 15 anchors) -> {out_path}",
        items.len()
    );
}

/// R1-F03 negative controls: the scanner RETAINS failures as rows and
/// keeps gap semantics — a missing directory, a malformed provider
/// catalog, and an oracle-less program each surface as their own row
/// class instead of silently shrinking the denominator.
#[test]
fn av_source_inventory_negative_controls() {
    let tmp = std::env::temp_dir().join(format!("av827-inv-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();

    // Missing directory -> a retained scan-error row (not an empty set).
    let rows = scan_fixture_dir(&tmp, "no-such-dir", "ac-fixture", "x");
    assert_eq!(rows.len(), 1, "one retained row");
    assert_eq!(rows[0].status, "scan-error");
    assert!(rows[0].note.contains("read_dir failed"));

    // Malformed provider JSON -> a retained parse-error row.
    std::fs::write(tmp.join("bad-providers.json"), b"{ not json").unwrap();
    let rows = scan_providers(&tmp, "bad-providers.json");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].status, "scan-error");
    assert!(rows[0].note.contains("parse failed"));

    // Structurally-valid JSON without the providers array -> its own row.
    std::fs::write(tmp.join("empty-providers.json"), b"{}").unwrap();
    let rows = scan_providers(&tmp, "empty-providers.json");
    assert_eq!(rows.len(), 1);
    assert!(rows[0].note.contains("no `providers` array"));

    // A corpus case dir WITHOUT any oracle sidecar stays in the
    // denominator as a gap row with a future-G owner; the row carries the
    // SOURCE byte identity (R2-F03).
    let case = tmp.join("vmroot/01_cat/001_prog");
    std::fs::create_dir_all(&case).unwrap();
    std::fs::write(case.join("prog.at"), b"fn main() int { return 0 }").unwrap();
    let rows = scan_vm_corpus(&tmp, "vmroot");
    let case_rows: Vec<&InvItem> = rows.iter().filter(|r| r.kind == "vm-corpus-case").collect();
    assert_eq!(case_rows.len(), 1, "the oracle-less case is still a row");
    assert!(case_rows[0].status.contains("oracle=none"));
    assert_eq!(case_rows[0].owner, "future-G");
    let sha_before = case_rows[0].sha256.clone();
    assert!(!sha_before.is_empty(), "the case row binds source bytes");

    // R2-F03 source-change control: editing the program source CHANGES the
    // row's identity — a silent identical row would hide the edit.
    std::fs::write(case.join("prog.at"), b"fn main() int { return 9 }").unwrap();
    let rows = scan_vm_corpus(&tmp, "vmroot");
    let after: &InvItem = rows
        .iter()
        .find(|r| r.kind == "vm-corpus-case")
        .expect("row after edit");
    assert_ne!(
        after.sha256, sha_before,
        "a source edit must move the row identity"
    );

    // R2-F03 unreadable-source control: an EXCLUSIVELY-LOCKED source file
    // (share_mode(0), same process) makes the read REALLY fail — the row
    // is a RETAINED scan-error row naming the failure, never a silent
    // drop; releasing the lock restores the normal row. (Windows-gated:
    // the lock is a Win32 share-mode violation.)
    #[cfg(windows)]
    {
        let lock_case = tmp.join("vmlock/01_cat/003_locked");
        std::fs::create_dir_all(&lock_case).unwrap();
        std::fs::write(lock_case.join("locked.at"), b"fn main() int { return 1 }").unwrap();
        {
            use std::os::windows::fs::OpenOptionsExt;
            let _exclusive = std::fs::OpenOptions::new()
                .read(true)
                .share_mode(0)
                .open(lock_case.join("locked.at"))
                .expect("open exclusive");
            let rows = scan_vm_corpus(&tmp, "vmlock");
            let locked: Vec<&InvItem> =
                rows.iter().filter(|r| r.kind == "vm-corpus-case").collect();
            assert_eq!(locked.len(), 1, "the unreadable case is retained");
            assert_eq!(locked[0].status, "scan-error");
            assert!(
                locked[0].note.contains("source read failed"),
                "the read failure is named: {:?}",
                locked[0].note
            );
        }
        // Lock released: the same scan now produces the normal row.
        let rows = scan_vm_corpus(&tmp, "vmlock");
        let normal: Vec<&InvItem> = rows.iter().filter(|r| r.kind == "vm-corpus-case").collect();
        assert_eq!(normal.len(), 1);
        assert_ne!(normal[0].status, "scan-error");
        assert!(!normal[0].sha256.is_empty());
    }

    // A missing markdown row is completed as a gap-missing-row (and the
    // main test turns any such row red).
    let rows = scan_md_rows(
        "| C-01 | x | a | b | c | d | e | f | g |",
        "C-",
        2,
        "capability",
        6,
        8,
    );
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .any(|r| r.status == "gap-missing-row" && r.id == "C-02"),
        "the absent C-02 row is retained as a gap: {rows:?}"
    );

    let _ = std::fs::remove_dir_all(&tmp);
}

/// V-04: an out-of-root `source.file` (`../` escape, absolute path, or a
/// drive form) is rejected at DECODE — the payload domain is the case
/// directory; payloads cannot reach across trees. (Backslash forms are
/// structurally unreachable: `validate_virtual_path` rejects any `\`, so
/// only forward-slash forms can even express an escape attempt.)
#[test]
fn av_source_admission_path_escape_rejected_at_decode() {
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
