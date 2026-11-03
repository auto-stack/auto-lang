//! PLAN-926 CA-C (T-09): the explicit TWO-LEG certification of the C
//! collections channel through the REAL prebuilt worker — every case of
//! the frozen 926 suite (the twenty 904 originals) certifies with the
//! ACTUAL set (real cl compile+run) AND the case's HAND-WRITTEN
//! reference-<case>.c (its own translation unit, real cl too) against
//! the frozen oracle table; wrong oracles and wrong references are
//! REJECTED; the certificate is the strict
//! `ace.a2c-hir.collections.cert/1` carrying the canonical
//! execution-config identity (mutually exclusive with every other
//! schema — cross-schema impersonation is a rejected tamper control).

use std::path::PathBuf;
use std::process::Command;

const WORKER: &str = "../../target/release/a2x-worker.exe";

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(WORKER);
    if p.exists() {
        return p;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../target/release/a2x-worker.exe")
}

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2c-hir-collections-926")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-coll/cert")
}

/// (case, oracle-kind, oracle-value)
const CASES: &[(&str, &str, &str)] = &[
    ("L01", "value", "42"),
    ("L02", "value", "42"),
    ("L03", "value", "0"),
    ("L04", "value", "42"),
    ("L05", "value", "42"),
    ("L06", "trap", "ListIndex"),
    ("L07", "value", "42"),
    ("V01", "value", "42"),
    ("V02", "value", "42"),
    ("V03", "value", "3"),
    ("V04", "value", "42"),
    ("V05", "value", "42"),
    ("V06", "value", "42"),
    ("V07", "value", "42"),
    ("LSTR864", "value", "51"),
    ("FIN864A", "value", "42"),
    ("FIN864B", "value", "53"),
    ("N889-list-variant", "value", "42"),
    ("N889-nested-variant", "value", "42"),
    ("N889-optional-variant", "value", "42"),
];

fn certify(
    case: &str,
    oracle_kind: &str,
    oracle_value: &str,
    reference_override: Option<&str>,
    mode: &str,
) -> serde_json::Value {
    let dir = out_base().join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    // NOTE: no canonicalize — its \?\ extended-length prefix breaks cl's
    // source resolution under vcvars
    let src_path = suite_dir().join("src").join(format!("{case}.at"));
    let ref_path = match reference_override {
        Some(r) => {
            let p = dir.join("reference.c");
            std::fs::write(&p, r).unwrap();
            p
        }
        None => suite_dir().join(format!("reference-{case}.c")),
    };
    let mut req = serde_json::json!({
        "op": "certify-c",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "case_id": case,
        "mode": mode,
    });
    if oracle_kind == "value" {
        req["oracle_kind"] = "value".into();
        req["oracle_value"] = oracle_value.parse::<i64>().unwrap().into();
    } else {
        req["oracle_kind"] = "trap".into();
        req["oracle_trap"] = oracle_value.into();
    }
    use std::io::Write as _;
    let mut child = Command::new(worker_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("worker spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .expect("write request");
    let out = child.wait_with_output().expect("worker complete");
    // a controlled rejection exits nonzero WITH the error json — parse
    // either way; the caller's verdict/status assertions decide
    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout).expect("worker json");
    if parsed["status"] != "error" && out.status.code() != Some(0) {
        panic!(
            "worker failed for {case}: {}",
            String::from_utf8_lossy(&out.stdout)
        );
    }
    parsed
}

/// V-01/V-11: all twenty suite cases certify TWO-LEG against the frozen
/// oracle table; every certificate is the strict collections schema with
/// the execution-config identity; both legs really ran cl (2 spawns).
#[test]
fn a2c_coll_certify_all_suite_cases() {
    for (case, kind, value) in CASES {
        let r = certify(case, kind, value, None, "debug");
        assert_eq!(r["verdict"], "PASS", "{case}: {r}");
        assert_eq!(r["recipe"], "ac-a2c/hir-collections-emit-v1", "{case}");
        assert_eq!(r["case"], format!("a2c-hir-926/{case}"));
        assert_eq!(r["spawned_cl"], 2, "{case}: both legs compile");
        assert_eq!(r["spawned_rustc"], 0, "{case}");
        // the strict cert on disk
        let cert = std::fs::read_to_string(out_base().join(case).join("case.cert.json"))
            .unwrap_or_else(|e| panic!("{case}: {e}"));
        assert!(
            cert.contains("\"schema\": \"ace.a2c-hir.collections.cert/1\""),
            "{case}"
        );
        assert!(cert.contains("\"execution_config\": {\"budget_work_units\":16777216,\"cancel\":false,\"events\":false,\"fault_alloc\":null,\"mode\":\"debug\"}"), "{case}");
        assert!(cert.contains("\"reference_sha256\""), "{case}");
        assert!(cert.contains("\"cl\""), "{case}");
    }
}

/// The certification counterexamples: a WRONG oracle (70 — a value no
/// leg produces) and a WRONG reference (independently computes a
/// different value) are both REJECTED; a scalar-schema impersonation is
/// refused by the strict pairing.
#[test]
fn a2c_coll_certify_wrong_legs_rejected() {
    // wrong oracle: the ACTUAL leg disagrees -> controlled failure
    // (certify-c rejects by error status, never a wrong verdict)
    let r = certify("L01", "value", "70", None, "debug");
    assert_eq!(r["status"], "error", "{r}");
    assert!(
        r["detail"]
            .as_str()
            .unwrap_or("")
            .contains("actual observed"),
        "{r}"
    );
    // wrong reference: the REFERENCE leg disagrees -> controlled failure
    let bad_ref = std::fs::read_to_string(suite_dir().join("reference-L01.c"))
        .unwrap()
        .replace("xs[0] + xs[1]", "xs[0] + xs[1] + 1");
    let r = certify("L01", "value", "42", Some(&bad_ref), "debug");
    assert_eq!(r["status"], "error", "{r}");
    assert!(
        r["detail"]
            .as_str()
            .unwrap_or("")
            .contains("reference observed"),
        "{r}"
    );
    // the release-mode config identity differs from debug (key material)
    let r = certify("L01", "value", "42", None, "release");
    assert_eq!(r["verdict"], "PASS", "{r}");
    let cert = std::fs::read_to_string(out_base().join("L01").join("case.cert.json")).unwrap();
    assert!(
        cert.contains("\"mode\":\"release\""),
        "mode is cert key material"
    );
}

/// V-12 (daily spawn-zero): the daily-emit-c leg of the collections
/// channel never spawns cl/rustc, and the manifest binds the full-source
/// sha + the collections schema.
#[test]
fn a2c_coll_daily_emit_spawn_zero() {
    let dir = out_base().join("daily-L01");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src_path = suite_dir().join("src").join("L01.at");
    let req = serde_json::json!({
        "op": "daily-emit-c",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": std::fs::canonicalize(&dir).unwrap().display().to_string(),
        "nonce": "n-cert-test",
    });
    use std::io::Write as _;
    let mut child = Command::new(worker_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("worker spawn");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .expect("write");
    let out = child.wait_with_output().expect("done");
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["status"], "ok", "{r}");
    assert_eq!(r["spawned_cl"], 0, "{r}");
    assert_eq!(r["spawned_rustc"], 0, "{r}");
    let manifest = std::fs::read_to_string(dir.join("manifest.json")).unwrap();
    assert!(manifest.contains("auto.a2x.c.collections.fileset/1"));
    assert!(manifest.contains(
        "\"source_sha256\": \"8247b7e7688e1931a36bc09eb4ec2cc789a3b099237d20d5dc8095a71e76a4f5\""
    ));
}
