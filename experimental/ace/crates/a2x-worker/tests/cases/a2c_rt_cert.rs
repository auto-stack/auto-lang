//! PLAN-925 T-08/T-09 (V-01/02/03/10/11): the C RECORD-TEXT certification
//! contract — the real two-leg certification (actual + hand-written
//! reference, both compiled and run independently under the real MSVC cl
//! against one closed-form oracle) through the record-text channel's OWN
//! identities (recipe `ac-a2c/hir-record-text-emit-v1`, schema
//! `ace.a2c-hir.record-text.cert/1`), and the STRICT bad-evidence
//! controls: wrong oracle, cross-CHANNEL impersonation (a scalar
//! schema/recipe pair under the record-text cert and the reverse), trimmed
//! evidence and non-PASS verdicts are all REJECTED.

use std::path::PathBuf;
use std::process::Command;

fn suite_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../crates/auto-lang/test/cases/a2c-hir-record-text-925")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out-c-rt/cert-925")
}

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/a2x-worker.exe");
    assert!(p.exists(), "prebuilt release worker required (rebuild it)");
    p
}

fn certify(req: &serde_json::Value) -> (bool, serde_json::Value) {
    use std::io::Write as _;
    let mut proc = Command::new(worker_path())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("worker spawn");
    proc.stdin
        .as_mut()
        .unwrap()
        .write_all(req.to_string().as_bytes())
        .expect("write request");
    let out = proc.wait_with_output().expect("worker complete");
    let json: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
    (out.status.code() == Some(0) && json["status"] == "ok", json)
}

fn base_req(tag: &str) -> serde_json::Value {
    let dir = out_base().join(tag);
    let _ = std::fs::remove_dir_all(&dir);
    serde_json::json!({
        "op": "certify-c",
        "source_path": suite_dir().join("src/R07.at").display().to_string(),
        "profile": "record",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": suite_dir().join("reference-R07.c").display().to_string(),
        "oracle_kind": "value",
        "oracle_value": 42,
        "case_id": "R07",
        "nonce": "cert-rt",
    })
}

/// One certification case: an inline source with a HAND-WRITTEN reference
/// and a closed-form oracle. The two legs run independently.
pub(super) struct CertCase {
    pub(super) id: &'static str,
    pub(super) src: &'static str,
    pub(super) profile: &'static str,
    pub(super) caps: &'static [&'static str],
    pub(super) oracle_kind: &'static str,
    pub(super) oracle_value: i64,
    pub(super) oracle_trap: &'static str,
    pub(super) reference: String,
}

/// Certify one inline case: stage src+reference under the tag dir and run
/// the worker `certify-c` op; assert the PASS verdict and BOTH observed
/// legs equal the oracle line.
fn certify_inline_case(c: &CertCase) {
    let dir = out_base().join(c.id);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let src_path = dir.join("case.at");
    let ref_path = dir.join("reference.c");
    std::fs::write(&src_path, c.src).unwrap();
    std::fs::write(&ref_path, &c.reference).unwrap();
    let mut req = serde_json::json!({
        "op": "certify-c",
        "source_path": src_path.display().to_string(),
        "profile": c.profile,
        "entry": "main",
        "caps": c.caps,
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "oracle_kind": c.oracle_kind,
        "case_id": c.id,
        "nonce": "cert-rt-matrix",
    });
    if c.oracle_kind == "value" {
        req["oracle_value"] = c.oracle_value.into();
    } else {
        req["oracle_trap"] = c.oracle_trap.into();
    }
    let (ok, r) = certify(&req);
    assert!(ok, "{} failed: {}", c.id, r);
    let line = if c.oracle_kind == "value" {
        format!("value:{}", c.oracle_value)
    } else {
        format!("trap:{}", c.oracle_trap)
    };
    assert_eq!(r["actual_stdout"], line.as_str(), "{} actual", c.id);
    assert_eq!(r["reference_stdout"], line.as_str(), "{} reference", c.id);
    assert_eq!(
        r["recipe"], "ac-a2c/hir-record-text-emit-v1",
        "{} recipe",
        c.id
    );
    assert_eq!(r["spawned_rustc"], 0, "{} never spawns rustc", c.id);
    assert!(
        r["spawned_cl"].as_i64().unwrap() >= 2,
        "{} two cl legs",
        c.id
    );
}

/// The shared hand-written C prologue (binary stdout) — every reference
/// below is written directly from the SOURCE semantics, never translated
/// from the generated C.
const REF_HEAD: &str = "#include <fcntl.h>\n#include <io.h>\n#include <stdint.h>\n#include <stdio.h>\n#include <string.h>\n";

fn main_ref(body: &str) -> String {
    format!(
        "{REF_HEAD}int main(void) {{\n    _setmode(_fileno(stdout), _O_BINARY);\n{body}    return 0;\n}}\n"
    )
}

/// The two-leg pass through the RECORD-TEXT channel identities; byte-stable
/// certificate serialization; the emitted daily set carries the rt fileset.
#[test]
fn rt_certify_two_leg_pass_and_stable_bytes() {
    let (ok, r) = certify(&base_req("pass-a"));
    assert!(ok, "{}", r);
    assert_eq!(r["verdict"], "PASS");
    assert_eq!(r["actual_stdout"], "value:42");
    assert_eq!(r["reference_stdout"], "value:42");
    assert_eq!(r["recipe"], "ac-a2c/hir-record-text-emit-v1");
    assert_eq!(r["spawned_rustc"], 0, "the C channel never spawns rustc");
    assert!(r["spawned_cl"].as_i64().unwrap() >= 2, "two real cl legs");
    let cert_a = std::fs::read_to_string(r["cert_path"].as_str().unwrap()).unwrap();
    assert!(cert_a.contains("ace.a2c-hir.record-text.cert/1"));
    assert!(cert_a.contains("\"backend\": \"auto-a2x-c\""));
    assert!(cert_a.contains("\"standard\": \"c11\""));
    assert!(cert_a.contains("\"vcvars_sha256\""));
    // the actual set is the RECORD-TEXT fileset (rt runtime header)
    let dir = out_base().join("pass-a");
    assert!(dir.join("a2x_c_rt_runtime.h").is_file());
    let (_, r2) = certify(&base_req("pass-b"));
    let cert_b = std::fs::read_to_string(r2["cert_path"].as_str().unwrap()).unwrap();
    assert_eq!(cert_a, cert_b, "cert serialization is byte-stable");
}

/// A WRONG oracle is refused (both legs observe the closed-form oracle).
#[test]
fn rt_certify_wrong_oracle_refused() {
    let mut req = base_req("wrong-oracle");
    req["oracle_value"] = 43.into();
    let (ok, r) = certify(&req);
    assert!(!ok, "wrong oracle must fail: {}", r);
    assert!(
        r["detail"].as_str().unwrap_or("").contains("a2x.oracle"),
        "{}",
        r
    );
}

/// STRICT cross-channel controls: the schema/recipe PAIR must match the
/// channel — a scalar pair under the record-text schema (and the reverse)
/// is rejected tamper, as are trimmed files and non-PASS verdicts.
#[test]
fn rt_cert_draft_strict_controls() {
    use ace_a2x_c::cert::{
        BoundFile, CCertDraft, CToolchainIdentity, C_CERT_SCHEMA, C_RT_CERT_SCHEMA,
    };
    fn draft(schema: &str, recipe: &str) -> CCertDraft {
        CCertDraft {
            schema: schema.to_string(),
            recipe: recipe.to_string(),
            case: "a2c-hir-925/R07".into(),
            profile: "record".into(),
            hir_domain: "auto.hir.text-record".into(),
            entry: "main".into(),
            verdict: "PASS".into(),
            oracle_kind: "value".into(),
            oracle_value: "42".into(),
            input_hash: "ih".into(),
            source_sha256: "a".repeat(64),
            actual_files: vec![BoundFile {
                path: "main.c".into(),
                sha256: "b".repeat(64),
            }],
            golden_sha256: None,
            wrapper_sha256: "c".repeat(64),
            reference_sha256: "d".repeat(64),
            backend: "auto-a2x-c".into(),
            backend_version: "0.1.0".into(),
            standard: "c11".into(),
            shared_view_version: "0.1.0".into(),
            protocol: "a2x-observation/1".into(),
            toolchain: CToolchainIdentity {
                cl_version: "19.xx".into(),
                cl_hash: "e".repeat(64),
                vcvars_sha256: "f".repeat(64),
                flags: vec!["-nologo".into()],
            },
            execution_config: None,
            capabilities: vec![],
        }
    }
    // the exact pairing validates
    assert!(draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1")
        .validate()
        .is_ok());
    // cross-channel impersonation is rejected BOTH ways
    assert!(
        draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-scalar-emit-v1")
            .validate()
            .is_err(),
        "a scalar recipe under the record-text schema is tamper"
    );
    assert!(
        draft(C_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1")
            .validate()
            .is_err(),
        "the record-text recipe under the scalar schema is tamper"
    );
    // old/foreign schemas stay untrusted
    assert!(draft("ace.m1.cert/1", "ac-a2c/hir-record-text-emit-v1")
        .validate()
        .is_err());
    assert!(draft(
        "ace.a2r-hir.record-text.cert/1",
        "ac-a2c/hir-record-text-emit-v1"
    )
    .validate()
    .is_err());
    // trimmed evidence / non-PASS verdicts
    let mut d = draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1");
    d.actual_files.clear();
    assert!(d.validate().is_err(), "trimmed actual files rejected");
    let mut d = draft(C_RT_CERT_SCHEMA, "ac-a2c/hir-record-text-emit-v1");
    d.verdict = "MAYBE".into();
    assert!(d.validate().is_err(), "non-PASS/REJECTED verdict rejected");
}

/// PLAN-925 T-09 (V-01): the WHOLE frozen first-suite surface through the
/// certification channel — the four 902 frozen sources plus the C04
/// method composite, each actual set and its reviewed hand-written
/// reference-<case>.c compiled and run independently against the frozen
/// oracle table (42/42/23/42/42).
#[test]
fn rt_certify_all_suite_cases() {
    // (case, profile, caps, oracle)
    let table: &[(&str, &str, &[&str], i64)] = &[
        ("R07", "record", &[], 42),
        ("R08", "record", &[], 42),
        ("T03", "text-record", &["ac.runtime.text.v1"], 23),
        ("T05", "text", &["ac.runtime.text.v1"], 42),
        ("C04", "text-record", &["ac.runtime.text.v1"], 42),
    ];
    for (case, profile, caps, oracle) in table {
        let dir = out_base().join(format!("suite-{case}"));
        let _ = std::fs::remove_dir_all(&dir);
        let req = serde_json::json!({
            "op": "certify-c",
            "source_path": suite_dir().join(format!("src/{case}.at")).display().to_string(),
            "profile": profile,
            "entry": "main",
            "caps": caps,
            "out_dir": dir.display().to_string(),
            "reference_path": suite_dir().join(format!("reference-{case}.c")).display().to_string(),
            "oracle_kind": "value",
            "oracle_value": oracle,
            "case_id": case,
            "nonce": "cert-rt-suite",
        });
        let (ok, r) = certify(&req);
        assert!(ok, "{case} failed: {}", r);
        assert_eq!(r["verdict"], "PASS", "{case}");
        assert_eq!(r["actual_stdout"], format!("value:{oracle}"), "{case}");
        assert_eq!(r["reference_stdout"], format!("value:{oracle}"), "{case}");
        let cert = std::fs::read_to_string(r["cert_path"].as_str().unwrap()).unwrap();
        assert!(cert.contains("ace.a2c-hir.record-text.cert/1"), "{case}");
        assert!(
            cert.contains(&format!("\"case\": \"a2c-hir-925/{case}\"")),
            "{case}"
        );
    }
}

/// PLAN-925 T-09 (V-02 axis): the modes/methods matrix through the
/// certification channel — every parameter mode and method shape with its
/// own hand-written reference, closing the same frozen oracle values the
/// daily actual-only matrix pins.
pub(super) fn mode_contracts_941() -> Vec<CertCase> {
    vec![
        // Copy snapshot: evaluation-point value survives the later writeback
        CertCase {
            id: "M_COPY_SNAP",
            src: "type Counter { value int }\nfn bump(mut x Counter, delta int) int {\n    x.value = x.value + delta\n    return x.value\n}\nfn main() int {\n    var x Counter = Counter(20)\n    let snap Counter = x\n    let now int = bump(x.mut, 1)\n    return snap.value * 10 + now\n}",
            profile: "record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 221,
            oracle_trap: "",
            reference: main_ref(
                "    int32_t snap = 20;      /* copy taken at evaluation point */\n    int32_t now = 21;        /* the callee writes back 20+1 */\n    printf(\"value:%d\\n\", snap * 10 + now);\n",
            ),
        },
        // multiple mut roots, real writeback on both
        CertCase {
            id: "M_MULTI_MUT",
            src: "type Cell { v int }\nfn pair(mut a Cell, mut b Cell) int {\n    a.v = 1\n    b.v = 2\n    return a.v * 10 + b.v\n}\nfn main() int {\n    var x Cell = Cell(0)\n    var y Cell = Cell(0)\n    return pair(x.mut, y.mut)\n}",
            profile: "record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 12,
            oracle_trap: "",
            reference: main_ref(
                "    int32_t a = 1, b = 2;   /* both mut roots written back */\n    printf(\"value:%d\\n\", a * 10 + b);\n",
            ),
        },
        // view record param: read-only, no copy
        CertCase {
            id: "M_VIEW_RECORD",
            src: "type Counter { value int }\nfn scale(x Counter, k int) int {\n    return x.value * k\n}\nfn main() int {\n    let x Counter = Counter(6)\n    return scale(x, 7)\n}",
            profile: "record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 42,
            oracle_trap: "",
            reference: main_ref("    printf(\"value:%d\\n\", 6 * 7);\n"),
        },
        // mut TEXT param: exclusive forward domain
        CertCase {
            id: "M_MUT_TEXT",
            src: "fn twice(mut s Str) int {\n    let t Str = s + s\n    return t.len()\n}\nfn main() int {\n    var a Str = \"ab\"\n    return twice(a)\n}",
            profile: "text-record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 4,
            oracle_trap: "",
            reference: main_ref("    printf(\"value:%d\\n\", 4); /* \"ab\"+\"ab\" = 4 chars */\n"),
        },
        // borrow return: slice view of a view param
        CertCase {
            id: "M_BORROW_RETURN",
            src: "fn tail(s str) str {\n    return s.slice(1, 3)\n}\nfn main() int {\n    let a Str = \"wxyz\"\n    let t str = tail(a.to(str))\n    return t.len()\n}",
            profile: "text",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 2,
            oracle_trap: "",
            reference: main_ref("    printf(\"value:%d\\n\", 2); /* slice(1,3) of wxyz = xy */\n"),
        },
        // owned return: the caller owns the returned buffer
        CertCase {
            id: "M_OWNED_RETURN",
            src: "fn dupe(s str) Str {\n    return s.clone()\n}\nfn main() int {\n    let a Str = \"ab\"\n    let b Str = dupe(a.to(str))\n    return b.len() * 10 + a.len()\n}",
            profile: "text",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 22,
            oracle_trap: "",
            reference: main_ref("    printf(\"value:%d\\n\", 2 * 10 + 2);\n"),
        },
        // named construction places by NAME, not written position
        CertCase {
            id: "M_NAMED_CTOR",
            src: "type P {\n    a int\n    b int\n}\nfn main() int {\n    let p P = P { b: 2, a: 1 }\n    return p.a * 10 + p.b\n}",
            profile: "record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 12,
            oracle_trap: "",
            reference: main_ref(
                "    int32_t a = 1; int32_t b = 2; /* by name */\n    printf(\"value:%d\\n\", a * 10 + b);\n",
            ),
        },
        // same-named free fn + two-owner methods (nominal namespaces)
        CertCase {
            id: "M_NAME_COLLISION",
            src: "type Counter { value int\n  fn read() int { return .value } }\ntype Gauge { value int\n  fn read() int { return .value + 1 } }\nfn read() int { return 7 }\nfn main() int {\n    let c Counter = Counter(1)\n    let g Gauge = Gauge(2)\n    return c.read() * 100 + g.read() * 10 + read()\n}",
            profile: "record",
            caps: &[],
            oracle_kind: "value",
            oracle_value: 137,
            oracle_trap: "",
            reference: main_ref(
                "    int32_t c = 1, g = 2;\n    printf(\"value:%d\\n\", c * 100 + (g + 1) * 10 + 7);\n",
            ),
        },
    ]
}

#[test]
fn rt_certify_modes_matrix() {
    let cases = mode_contracts_941();
    for c in &cases {
        certify_inline_case(c);
    }
}

/// PLAN-925 T-09 (V-03 axis): the trap surface certifies with kind=trap —
/// the boundary check is an independent decision in each leg; a wrong
/// trap NAME is refused just like a wrong value.
pub(super) fn trap_contracts_941() -> Vec<CertCase> {
    vec![
        // negative char index traps before any pointer arithmetic
        CertCase {
            id: "T_CHAR_AT_NEG",
            src: "fn main() int {\n    let s Str = \"abc\"\n    let c char = s.char_at(-1)\n    return c.to_codepoint()\n}",
            profile: "text",
            caps: &["ac.runtime.text.v1"],
            oracle_kind: "trap",
            oracle_value: 0,
            oracle_trap: "TextIndex",
            reference: main_ref(
                "    int64_t i = -1; int64_t n = 3;\n    if (i < 0 || i >= n) { printf(\"trap:TextIndex\\n\"); return 0; }\n    printf(\"value:99\\n\");\n",
            ),
        },
        // slice past the scalar length traps
        CertCase {
            id: "T_SLICE_OOB",
            src: "fn main() int {\n    let s Str = \"héllo\"\n    let t str = s.slice(0, 6)\n    return t.len()\n}",
            profile: "text",
            caps: &["ac.runtime.text.v1"],
            oracle_kind: "trap",
            oracle_value: 0,
            oracle_trap: "TextIndex",
            reference: main_ref(
                "    int64_t n = 5; int64_t end = 6; /* h é l l o */\n    if (end > n) { printf(\"trap:TextIndex\\n\"); return 0; }\n    printf(\"value:99\\n\");\n",
            ),
        },
        // surrogate codepoint is an invalid scalar
        CertCase {
            id: "T_SURROGATE",
            src: "fn main() int {\n    let c char = char.from_codepoint(55296)\n    return c.to_codepoint()\n}",
            profile: "text",
            caps: &["ac.runtime.text.v1"],
            oracle_kind: "trap",
            oracle_value: 0,
            oracle_trap: "InvalidScalar",
            reference: main_ref(
                "    uint32_t cp = 55296u;\n    if (cp >= 0xD800u && cp <= 0xDFFFu) { printf(\"trap:InvalidScalar\\n\"); return 0; }\n    printf(\"value:99\\n\");\n",
            ),
        },
    ]
}

#[test]
fn rt_certify_trap_axis() {
    let cases = trap_contracts_941();
    for c in &cases {
        certify_inline_case(c);
    }
    // wrong trap NAME refused (both legs observe the closed-form oracle)
    let dir = out_base().join("wrong-trap");
    let _ = std::fs::remove_dir_all(&dir);
    let req = serde_json::json!({
        "op": "certify-c",
        "source_path": suite_dir().join("src/T05.at").display().to_string(),
        "profile": "text",
        "entry": "main",
        "caps": ["ac.runtime.text.v1"],
        "out_dir": dir.display().to_string(),
        "reference_path": suite_dir().join("reference-T05.c").display().to_string(),
        "oracle_kind": "trap",
        "oracle_trap": "TextIndex",
        "case_id": "T05",
        "nonce": "cert-rt-neg",
    });
    let (ok, r) = certify(&req);
    assert!(!ok, "wrong trap name must fail: {}", r);
    assert!(
        r["detail"].as_str().unwrap_or("").contains("a2x.oracle"),
        "{}",
        r
    );
}

/// PLAN-925 T-09 negative controls: a hand-written but WRONG reference
/// (compiles cleanly, prints the wrong value) and a SYNTACTICALLY BROKEN
/// reference are both refused at the reference leg — the certification
/// really compiles and observes the reference, not a hash of it. The
/// broken shape is the exact regression found in the first T-09 review
/// pass (a literal `\n` stranded in the C source of reference-T03.c).
#[test]
fn rt_certify_bad_references_refused() {
    // 1) wrong-value reference: valid C, wrong oracle answer
    let dir = out_base().join("bad-ref-value");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let ref_wrong = dir.join("reference.c");
    std::fs::write(&ref_wrong, main_ref("    printf(\"value:43\\n\");\n")).unwrap();
    let mut req = base_req("bad-ref-value-anchor");
    req["out_dir"] = out_base()
        .join("bad-ref-value-anchor")
        .display()
        .to_string()
        .into();
    req["reference_path"] = ref_wrong.display().to_string().into();
    let (ok, r) = certify(&req);
    assert!(!ok, "a wrong-value reference must fail: {}", r);
    let detail = r["detail"].as_str().unwrap_or("");
    assert!(
        detail.contains("a2x.oracle") && detail.contains("reference"),
        "{}",
        r
    );

    // 2) syntactically broken reference (the literal-\n regression shape)
    let dir2 = out_base().join("bad-ref-syntax");
    let _ = std::fs::remove_dir_all(&dir2);
    std::fs::create_dir_all(&dir2).unwrap();
    let ref_broken = dir2.join("reference.c");
    std::fs::write(
        &ref_broken,
        format!("{REF_HEAD}int main(void) {{\n    const char* a = \"ab\";\\n    size_t n = 2;\n    (void)a; (void)n;\n    return 0;\n}}\n"),
    )
    .unwrap();
    let mut req2 = base_req("bad-ref-syntax-anchor");
    req2["out_dir"] = out_base()
        .join("bad-ref-syntax-anchor")
        .display()
        .to_string()
        .into();
    req2["reference_path"] = ref_broken.display().to_string().into();
    let (ok2, r2) = certify(&req2);
    assert!(!ok2, "a broken reference must fail to compile: {}", r2);
    assert!(
        r2["detail"]
            .as_str()
            .unwrap_or("")
            .contains("reference compile"),
        "{}",
        r2
    );
}
