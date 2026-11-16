//! PLAN-904 T-07/T-08 certify conformance (V-09/V-10/V-16): the explicit
//! two-leg certification of the collections channel through the REAL
//! prebuilt worker — the same-session ACTUAL file set and the case's
//! HAND-WRITTEN std-only reference both really compile and run against
//! the independent oracle; wrong oracles/references are REJECTED; the
//! execution-config identity (mode/budget/fault/events/cancel) is key
//! material of the collections certificate schema (mutually exclusive
//! with the older schemas).

use std::path::PathBuf;
use std::process::Command;
const WORKER: &str = "../../target/release/a2x-worker.exe";

fn worker_path() -> PathBuf {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(WORKER);
    if p.exists() {
        return p;
    }
    // dev-tree fallback
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../target/release/a2x-worker.exe")
}

fn out_base() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/a2x-out/cert-coll-904")
}

fn certify(
    case: &str,
    oracle_kind: &str,
    oracle_value: i64,
    reference: &str,
    mode: &str,
) -> serde_json::Value {
    let dir = out_base().join(case);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap();
    let ref_path = dir.join("reference.rs");
    std::fs::write(&ref_path, reference).unwrap();
    let src_path = dir.join("main.at");
    std::fs::write(
        &src_path,
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(41)\n    xs.push(1)\n    xs.set(1, 1)\n    return xs.get(0) + xs.get(1)\n}",
    )
    .unwrap();
    let req = serde_json::json!({
        "op": "certify",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "oracle_kind": oracle_kind,
        "oracle_value": oracle_value,
        "case_id": case,
        "mode": mode,
    });
    // stdin form (Windows argv quoting mangles long JSON requests)
    use std::io::Write as _;
    let mut child = std::process::Stdio::piped();
    let _ = &mut child;
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "worker failed: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    serde_json::from_slice(&out.stdout).expect("worker json")
}

/// The L01-shaped reference (independent std-only implementation).
const REF_L01: &str = r#"fn main() {
    let mut xs: Vec<i32> = Vec::new();
    xs.push(41);
    xs.push(1);
    xs[1] = 1;
    println!("value:{}", xs[0] + xs[1]);
}
"#;

#[test]
fn cert_coll_two_leg_pass_and_wrong_oracle_rejected() {
    // PASS: both legs equal the independent oracle (42)
    let r = certify("pass", "value", 42, REF_L01, "debug");
    assert_eq!(r["verdict"], "PASS", "{r}");
    assert_eq!(r["recipe"], "ac-a2r/hir-collections-emit-v1");
    assert_eq!(r["actual_observed"], "value:42");
    assert_eq!(r["reference_observed"], "value:42");
    // the collections certificate carries the execution-config identity
    let cert = std::fs::read_to_string(out_base().join("pass").join("case.cert.json")).unwrap();
    assert!(cert.contains("\"execution_config\": \"{\\\"budget_work_units\\\":16777216"));
    assert!(cert.contains("ace.a2r-hir.collections.cert/1"));

    // WRONG oracle (70 — the value L06-style traps would never produce as
    // a plain value): both legs disagree -> REJECTED
    let r = certify("wrong-oracle", "value", 70, REF_L01, "debug");
    assert_eq!(r["verdict"], "REJECTED", "{r}");

    // WRONG reference (independently computes a different value): the
    // reference leg disagrees with the oracle -> REJECTED
    let bad_ref = REF_L01.replace("value:{}", "value:{} + 1");
    let bad_ref = bad_ref.replace(
        "println!(\"value:{} + 1\", xs[0] + xs[1]);",
        "println!(\"value:{}\", xs[0] + xs[1] + 1);",
    );
    let r = certify("wrong-ref", "value", 42, &bad_ref, "debug");
    assert_eq!(r["verdict"], "REJECTED", "{r}");
}

#[test]
fn cert_coll_trap_case_named_identity() {
    // L06-shaped trap: the ONLY certified outcome is the named ListIndex
    // trap — never a value, a panic, or an arbitrary nonzero
    let dir = out_base().join("trap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let dir = std::fs::canonicalize(&dir).unwrap();
    let ref_path = dir.join("reference.rs");
    std::fs::write(
        &ref_path,
        "fn main() {\n    let xs: Vec<i32> = vec![42];\n    match xs.get(1) {\n        Some(v) => println!(\"value:{}\", v),\n        None => println!(\"trap:ListIndex\"),\n    }\n}\n",
    )
    .unwrap();
    let src_path = dir.join("main.at");
    std::fs::write(
        &src_path,
        "fn main() int {\n    var xs = List<int>.new()\n    xs.push(42)\n    return xs.get(1)\n}",
    )
    .unwrap();
    let req = serde_json::json!({
        "op": "certify",
        "source_path": src_path.display().to_string(),
        "profile": "collections",
        "entry": "main",
        "caps": [],
        "out_dir": dir.display().to_string(),
        "reference_path": ref_path.display().to_string(),
        "oracle_kind": "trap",
        "oracle_trap": "ListIndex",
        "case_id": "trap",
        "mode": "debug",
    });
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
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["verdict"], "PASS", "{r}");
    assert_eq!(r["actual_observed"], "trap:ListIndex");
    assert_eq!(r["reference_observed"], "trap:ListIndex");
}

#[test]
fn cert_coll_mode_is_key_material() {
    // V-16: debug and release are DIFFERENT config identities — the
    // certificate's execution_config differs per mode (separate keys,
    // never cross-trusted)
    let d = certify("mode-debug", "value", 42, REF_L01, "debug");
    assert_eq!(d["verdict"], "PASS");
    let r = certify("mode-release", "value", 42, REF_L01, "release");
    assert_eq!(r["verdict"], "PASS");
    let cert_d =
        std::fs::read_to_string(out_base().join("mode-debug").join("case.cert.json")).unwrap();
    let cert_r =
        std::fs::read_to_string(out_base().join("mode-release").join("case.cert.json")).unwrap();
    assert!(
        cert_d.contains("mode\\\":\\\"debug"),
        "debug config: {cert_d}"
    );
    assert!(
        cert_r.contains("mode\\\":\\\"release"),
        "release config: {cert_r}"
    );
    assert_ne!(cert_d, cert_r, "configs never share one certificate");
}

#[test]
fn cert_coll_schema_isolation() {
    // the collections certificate schema is strictly disjoint from the
    // older channels: a collections draft cannot validate as scalar/rt
    let draft = ace_a2x_rust::CertDraft {
        schema: ace_a2x_rust::COLL_CERT_SCHEMA,
        recipe: ace_a2x_rust::coll::COLL_RECIPE,
        case: "c".into(),
        profile: "auto.source.collections.draft".into(),
        hir_domain: "collections-draft".into(),
        entry: "main".into(),
        verdict: "PASS".into(),
        oracle_kind: "value",
        oracle_value: "42".into(),
        input_hash: "0123456789abcdef".into(),
        source_sha256: "a".repeat(64),
        actual_files: vec![ace_a2x_rust::BoundFile {
            path: "main.rs".into(),
            sha256: "b".repeat(64),
        }],
        golden_sha256: None,
        wrapper_sha256: "c".repeat(64),
        reference_sha256: "d".repeat(64),
        backend: "auto-a2x-rust",
        backend_version: "0.1.0",
        shared_view_version: "0.1.0",
        protocol: "a2x-observation/1",
        toolchain: ace_a2x_rust::ToolchainIdentity::observed(
            "rustc x".into(),
            vec!["--edition".into(), "2021".into()],
        ),
        capabilities: vec![],
        execution_config: Some(ace_a2x_rust::coll::canonical_execution_config(
            "debug", 16777216, None, false, false,
        )),
    };
    assert!(draft
        .validate_as(
            ace_a2x_rust::COLL_CERT_SCHEMA,
            ace_a2x_rust::coll::COLL_RECIPE
        )
        .is_ok());
    // impersonating the older schemas is a named rejection
    assert!(draft
        .validate_as(ace_a2x_rust::CERT_SCHEMA, ace_a2x_rust::RECIPE)
        .is_err());
    assert!(draft
        .validate_as(ace_a2x_rust::RT_CERT_SCHEMA, ace_a2x_rust::RT_RECIPE)
        .is_err());
    // and a collections draft WITHOUT the execution-config identity is
    // invalid (config is key material, never optional here)
    let no_cfg = ace_a2x_rust::CertDraft {
        execution_config: None,
        ..draft
    };
    assert!(no_cfg
        .validate_as(
            ace_a2x_rust::COLL_CERT_SCHEMA,
            ace_a2x_rust::coll::COLL_RECIPE
        )
        .is_err());
    // while the OLDER schemas must NOT carry one
    let old = ace_a2x_rust::CertDraft {
        schema: ace_a2x_rust::CERT_SCHEMA,
        recipe: ace_a2x_rust::RECIPE,
        profile: "auto.source.scalar-i32.draft".into(),
        hir_domain: "scalar-i32-draft".into(),
        execution_config: None,
        ..no_cfg
    };
    assert!(old.validate().is_ok());
}

// ---- PLAN-1036 (AUDIT-1024 F35): the continue back-edge poll ----------
//
// F35: the C/Rust collections emitters appended the cooperative poll at the
// END of the loop body. A `continue` jumped straight to the loop header, so
// every continue-driven back edge skipped BOTH the 1-WU budget charge and
// the cancellation check. The loop below routes every back edge through a
// conditional `continue` and its only other exit is `break` — so the
// pre-fix run observes ZERO polls while the fixed run observes exactly
// three (3 back edges, 0 from the never-executed fall-through).

/// The frozen LOOP-03 source: n counts 1..=4, `break` at 4, conditional
/// `continue` otherwise. Value oracle: 40 (n*10 + t, t never reached).
pub const LOOP_03_SRC: &str =
    "fn main() int {\n    var n = 0\n    var t = 0\n    loop {\n        n = n + 1\n        if n == 4 { break }\n        if n < 4 { continue }\n        t = t + 1\n    }\n    return n * 10 + t\n}";

/// Emit + compile (real rustc) + run the collections Rust leg with `env`;
/// returns (stdout, stderr).
fn run_coll_rust_leg(name: &str, src: &str, env: &[(&str, &str)]) -> (String, String) {
    use ace_a2x_core::sha256::sha256_hex;
    use ace_a2x_core::SemanticView;
    use ace_a2x_rust::coll::assemble_collections_file_set;
    use ace_a2x_rust::lower_coll::lower_collections_program;
    use ace_core::source::compile_collections_source;

    let compiled = compile_collections_source(src, "m.at", &[]).unwrap_or_else(|ds| {
        panic!(
            "frontend diagnostics for {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_collections_program(&view, "main").expect("collections lowering");
    let set = assemble_collections_file_set(
        &view,
        &program,
        "input-hash",
        &sha256_hex(src.as_bytes()),
        "{\"budget_work_units\":16777216,\"cancel\":false,\"events\":false,\"fault_alloc\":null,\"mode\":\"debug\"}",
        "main",
    );
    let dir = out_base().join(format!("loop-03-{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &set.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &set.manifest).unwrap();

    let exe = dir.join("loop03.exe");
    let mut compiler = Command::new(std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into()));
    compiler
        .args(["--edition", "2021", "-o"])
        .arg(&exe)
        .arg("main.rs")
        .current_dir(&dir);
    let out = auto_tool_execution::run_command_capture(
        compiler,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert_eq!(
        out.kind,
        auto_tool_execution::ExitKind::Success,
        "rustc failed for {name}: {}",
        String::from_utf8_lossy(out.stderr.as_ref().expect("captured stderr"))
    );

    let mut command = Command::new(&exe);
    command.current_dir(&dir);
    for (k, v) in env {
        command.env(k, v);
    }
    let run = auto_tool_execution::run_command_capture(
        command,
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit non-zero for {name}"
    );
    (
        String::from_utf8_lossy(run.stdout.as_ref().expect("captured stdout")).to_string(),
        String::from_utf8_lossy(run.stderr.as_ref().expect("captured stderr")).to_string(),
    )
}

fn count_polls(stderr: &str) -> usize {
    stderr.lines().filter(|l| l.trim() == "coll:poll").count()
}

/// F35 / LOOP-03 (Rust leg): exactly one poll per real back edge — three
/// continue-driven back edges, none for the `break` exit — and the poll is
/// the charge/cancel point the budget and cancellation knobs observe.
#[test]
fn audit1036_loop_03_continue_backedge_polls() {
    // clean run: value + exactly 3 polls (pre-fix: 0)
    let (out, err) = run_coll_rust_leg("clean", LOOP_03_SRC, &[("A2X_RT_EVENTS", "1")]);
    assert_eq!(out, "value:40\n", "LOOP-03 value");
    assert_eq!(
        count_polls(&err),
        3,
        "each continue back edge polls exactly once; stderr:\n{err}"
    );

    // The same loop through the FALL-THROUGH shape (no continue on the
    // last pass) must also account exactly one poll per back edge.
    let fallthrough = "fn main() int {\n    var n = 0\n    loop {\n        n = n + 1\n        if n == 3 { break }\n        if n == 1 { continue }\n    }\n    return n\n}";
    let (out, err) = run_coll_rust_leg("fallthrough", fallthrough, &[("A2X_RT_EVENTS", "1")]);
    assert_eq!(out, "value:3\n");
    assert_eq!(
        count_polls(&err),
        2,
        "n=1 continues (1 poll), n=2 falls through (1 poll), n=3 breaks; stderr:\n{err}"
    );

    // Budget boundary: Wbase is the clean run's own charge total
    // (4 WU call frame + 3 polls = 7). Wbase completes; Wbase-1 traps at
    // the third poll — the pre-fix path (no poll charge) completed both.
    let (out, err) = run_coll_rust_leg("budget-exact", LOOP_03_SRC, &[("A2X_RT_BUDGET", "7")]);
    assert_eq!(out, "value:40\n", "Wbase = 4 call + 3 polls: {err}");
    let (out, _err) = run_coll_rust_leg("budget-short", LOOP_03_SRC, &[("A2X_RT_BUDGET", "6")]);
    assert_eq!(out, "trap:Budget\n", "one WU short must trap at a poll");

    // Cancellation reaches the guest loop at its FIRST poll.
    let (out, _err) = run_coll_rust_leg("cancel", LOOP_03_SRC, &[("A2X_RT_CANCEL", "1")]);
    assert_eq!(
        out, "trap:Cancelled\n",
        "an armed cancel must be observed by a continue-driven back edge"
    );
}
