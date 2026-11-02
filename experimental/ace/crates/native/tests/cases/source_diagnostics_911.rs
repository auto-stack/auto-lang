//! PLAN-911 T-01 minimal chain: the native source-diagnostic fact channel.
//!
//! 1. The three M1 negatives (B17/B18/B19 shapes) reject through the REAL
//!    shared frontend and the operation owns a typed snapshot — raw
//!    verify/<code>, terminating profile, attempts, source identity — with
//!    the legacy `mn.unsupported.frontend` code/message unchanged and zero
//!    guest production (no codegen/link/run).
//! 2. The legal neighbor (NB17 shape) still runs through the real pipeline.
//! 3. A record-domain semantic error terminates at the RECORD profile after
//!    a recorded scalar surface rejection — the escalation is observable,
//!    the primary comes from the terminating domain.
//! 4. An illegal warm single-body edit refuses with a typed
//!    `LanguageDiagnostics`/`Frontend` refusal whose snapshot binds the
//!    failing candidate (base + candidate SHAs, session revision, edit
//!    region), while unchanged functions show zero body work and the
//!    session stays usable for the NEXT independent edit.
//! 5. The frozen capture caps (64 causes / 4 KiB message / 64 KiB payload)
//!    are typed `mn.resource.source-diagnostic` breaches, never truncation.

#[path = "../common/mod.rs"]
mod common;

use ace_core::atom_text::{Diagnostic, Span, Stage};
use ace_native::driver::MnDriver;
use ace_native::incremental::MnIncrementalSession;
use ace_native::source_diagnostics as sd;
use ace_native::source_diagnostics::{OperationKind, RawCause, SourceProfile};
use ace_native::MnError;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn911-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// The frozen corpus shapes (PLAN-852 src/B17..19.at, NB17.at). The case
// suite (T-04) references the real files; these unit legs pin the CHANNEL.
const B17: &str = "fn main() int { return true + 1 }\n";
const B18: &str = "fn main() int {\n    let x int = 1\n    x = 2\n    return x\n}\n";
const B19: &str = "fn main() int { if false { return 1 } }\n";
const NB17: &str = "fn main() int { return 1 + 1 }\n";

#[test]
fn mn_sd911_cold_negative_snapshots() {
    // A frontend rejection never reaches codegen/link/run: no toolchain.
    for (name, src, code) in [
        ("b17", B17, "source.type-mismatch"),
        ("b18", B18, "verify.immutable-write"),
        ("b19", B19, "verify.no-return-path"),
    ] {
        let root = temp_dir(&format!("cold-{name}"));
        let mut d = MnDriver::ephemeral();
        let r = d.request(src, "d_main", &[], &root, None);
        assert!(!r.ok, "{name}: must be rejected");
        let e = r.error.as_ref().expect("error present");
        // Default-channel compatibility: same code, same first-cause string.
        assert_eq!(e.code(), "mn.unsupported.frontend", "{name}");
        let snap = e
            .source_diagnostic()
            .unwrap_or_else(|| panic!("{name}: expected a typed Source snapshot, got {e:?}"));
        // Snapshot identity: this exact source, this operation.
        assert_eq!(snap.schema, sd::SD_SCHEMA);
        assert_eq!(snap.mode, sd::SD_MODE);
        assert_eq!(snap.producer, sd::SD_PRODUCER);
        assert_eq!(snap.profile, SourceProfile::Scalar);
        assert_eq!(snap.attempted, vec![SourceProfile::Scalar]);
        assert!(
            snap.surface_rejections.is_empty(),
            "{name}: scalar is the terminating domain, no escalation"
        );
        assert_eq!(snap.source_sha256, sd::sha256_hex(src.as_bytes()));
        assert_eq!(snap.source_len, src.len());
        assert_eq!(snap.path, "mn.at");
        assert_eq!(snap.span_basis, "utf8-byte");
        assert_eq!(snap.primary, 0);
        assert!(!snap.causes.is_empty());
        let c0 = &snap.causes[0];
        assert_eq!(c0.stage, "verify", "{name}: raw producer stage");
        assert_eq!(c0.code, code, "{name}: raw producer code");
        assert!(c0.span.start <= c0.span.end, "{name}: span ordered");
        assert!(c0.span.end <= snap.source_len, "{name}: span in bounds");
        // The primary is the compiler's first cause — never oracle-picked.
        assert_eq!(snap.primary_cause(), c0);
        // The legacy display is a projection of the SAME fact.
        assert_eq!(e.message(), c0.legacy_projection());
        assert!(
            e.message().starts_with(&format!("Verify/{code}: ")),
            "{name}: legacy projection shape: {}",
            e.message()
        );
        // Operation binding: cold request, this source.
        assert_eq!(snap.operation.kind, OperationKind::Cold);
        assert_eq!(
            snap.operation.candidate_sha256,
            sd::sha256_hex(src.as_bytes())
        );
        assert!(snap.operation.base_sha256.is_none());
        assert!(snap.operation.edit.is_none());
        // Zero guest production: no codegen, no link, no run, no streams.
        assert_eq!(*r.obs.counters.get("codegen_emit_units").unwrap_or(&0), 0);
        assert_eq!(*r.obs.counters.get("link_invocations").unwrap_or(&0), 0);
        assert_eq!(*r.obs.counters.get("run_invocations").unwrap_or(&0), 0);
        assert_eq!(r.exit_code, None);
        assert!(r.stdout.is_empty() && r.stderr.is_empty());
        // JSON round-trip is lossless (the parent's fact artifact basis).
        let back: sd::SourceDiagnosticSnapshot =
            serde_json::from_value(snap.to_json()).expect("snapshot JSON round-trips");
        assert_eq!(&back, snap);
        let _ = std::fs::remove_dir_all(&root);
    }
}

#[test]
fn mn_sd911_cold_legal_neighbor_still_runs() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let root = temp_dir("cold-nb17");
    let mut d = MnDriver::ephemeral();
    let r = d.request(NB17, "d_main", &[], &root, None);
    assert!(r.ok, "NB17 must run: {:?}", r.error);
    assert_eq!(r.exit_code, Some(2));
    assert!(r.error.is_none());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_cold_escalation_names_terminating_profile() {
    // A record-domain program with a SEMANTIC error: the scalar surface
    // rejects (`source.unsupported` at Text — a recorded escalation input),
    // the RECORD domain terminates with the real type error.
    let src = "type P { v int\n    fn mix(q P) int { return true + 1 }\n    static fn mk(n int) P { return P { v: n } }\n}\nfn main() int {\n    var p = P.mk(7)\n    return p.v\n}\n";
    let root = temp_dir("cold-esc");
    let mut d = MnDriver::ephemeral();
    let r = d.request(src, "d_main", &[], &root, None);
    assert!(!r.ok);
    let e = r.error.as_ref().expect("error");
    let snap = e
        .source_diagnostic()
        .unwrap_or_else(|| panic!("typed snapshot, got {e:?}"));
    assert_eq!(snap.profile, SourceProfile::Record, "record terminates");
    assert_eq!(
        snap.attempted,
        vec![SourceProfile::Scalar, SourceProfile::Record]
    );
    // The escalation's surface rejection is recorded, not silently consumed.
    assert_eq!(snap.surface_rejections.len(), 1);
    assert_eq!(snap.surface_rejections[0].profile, SourceProfile::Scalar);
    assert!(snap
        .surface_rejections
        .iter()
        .all(|s| s.codes.iter().all(|c| c == "source.unsupported")));
    // The primary is the TERMINATING domain's real cause.
    assert_eq!(snap.causes[0].code, "source.type-mismatch");
    assert_eq!(snap.causes[0].stage, "verify");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_warm_illegal_body_edit_typed_refusal() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let src =
        "fn alpha(x int) int {\n    return x * 2\n}\n\nfn main() int {\n    return alpha(5)\n}\n";
    let root = temp_dir("warm-illegal");
    let (mut s, cold) = MnIncrementalSession::warmup(src, "d_main", &[], &root.join("store"), None)
        .expect("warmup");
    assert_eq!(cold.exit_code, Some(10), "alpha(5) = 10");
    let units_before = s.units().clone();
    let revision_before = s.session_revision();

    // The illegal candidate: alpha's body `x * 2` -> `true + 1`.
    let site = src.find("x * 2").expect("edit site");
    let insert = b"true + 1".to_vec();
    let mut candidate = src.to_string();
    candidate.replace_range(site..site + 5, "true + 1");
    let obs = s.edit(site as u64, (site + 5) as u64, insert);
    assert!(!obs.ok, "the illegal candidate must refuse");
    assert!(obs.infra.is_none(), "a language refusal is not infra");
    let refusal = obs.refusal.as_ref().expect("typed refusal");
    assert_eq!(refusal.kind, sd::RefusalKind::LanguageDiagnostics);
    assert_eq!(refusal.phase, Some(sd::RejectPhase::Frontend));
    assert_eq!(
        obs.refused_reason.as_deref(),
        Some(refusal.reason.as_str()),
        "legacy reason is the refusal's projection"
    );
    let snap = refusal
        .source_error
        .as_ref()
        .expect("language refusal carries the snapshot");
    assert_eq!(snap.schema, sd::SD_SCHEMA);
    assert_eq!(snap.profile, SourceProfile::Scalar);
    assert_eq!(snap.path, "probe.at", "the 828 producer's real path");
    assert_eq!(snap.causes[0].stage, "verify");
    assert_eq!(snap.causes[0].code, "source.type-mismatch");
    // Candidate binding: base = the session's accepted prefix, candidate =
    // exactly the post-edit source the compiler saw.
    assert_eq!(
        snap.operation.base_sha256.as_deref(),
        Some(sd::sha256_hex(src.as_bytes()).as_str())
    );
    assert_eq!(
        snap.operation.candidate_sha256,
        sd::sha256_hex(candidate.as_bytes())
    );
    assert_eq!(snap.source_len, candidate.len());
    assert!(snap.source_sha256 == snap.operation.candidate_sha256);
    assert_eq!(snap.operation.session_revision, Some(revision_before));
    assert_eq!(snap.operation.kind, OperationKind::WarmEdit);
    let region = snap.operation.edit.as_ref().expect("edit region");
    assert_eq!(region.start, site as u64);
    assert_eq!(region.end, (site + 5) as u64);
    assert_eq!(region.insert_len, 8, "true + 1 is 8 bytes");
    // Honest accounting on a FAILED edit: nothing whole-module, nothing
    // unaffected, no fallback, no codegen — and the changed body's own
    // frontend attempt IS visible in the producer's body work.
    assert_eq!(obs.unaffected_stage_delta, 0);
    assert_eq!(obs.whole_module_stage_delta, 0);
    assert_eq!(obs.fallback_whole_lower, 0);
    assert_eq!(obs.codegen_units, 0);
    let alpha = obs
        .body_work
        .get("alpha")
        .expect("the failed body was reached");
    assert!(
        alpha.typecheck >= 1 || alpha.resolve >= 1,
        "attempt visible"
    );
    let main = obs.body_work.get("main");
    assert!(
        main.map(|w| w.resolve + w.typecheck + w.plan + w.bind == 0)
            .unwrap_or(true),
        "unchanged body did zero work: {:?}",
        main
    );
    // The refusal committed nothing: identities and source are unchanged.
    assert_eq!(s.units(), &units_before);
    assert_eq!(s.source(), src);

    // The NEXT independent edit (repair on the still-accepted base) works.
    let obs2 = s.edit(site as u64, (site + 5) as u64, b"x + 2".to_vec());
    assert!(obs2.ok, "repair on the accepted base: {:?}", obs2.infra);
    assert_eq!(obs2.exit_code, Some(7), "alpha(5) with x+2 = 7");
    s.close();
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_capture_caps_are_typed_breaches() {
    let mk = |n: usize, msg: &str| sd::CompileFailure {
        profile: SourceProfile::Scalar,
        attempted: vec![SourceProfile::Scalar],
        surface_rejections: Vec::new(),
        diagnostics: (0..n)
            .map(|_| {
                Diagnostic::new(
                    Stage::Verify,
                    "source.type-mismatch",
                    msg,
                    Span { start: 0, end: 1 },
                )
            })
            .collect(),
    };
    let op = sd::Operation::cold("fn main() int { return 1 }\n");
    // 65 causes: breach, not truncation.
    match sd::SourceDiagnosticSnapshot::capture("src", "mn.at", mk(65, "x"), op.clone()) {
        Err(MnError::Resource(k, _)) => assert_eq!(k, "source-diagnostic"),
        other => panic!("65 causes must breach: {other:?}"),
    }
    // One over-long message (4 KiB cap): breach.
    match sd::SourceDiagnosticSnapshot::capture(
        "src",
        "mn.at",
        mk(1, &"m".repeat(4097)),
        op.clone(),
    ) {
        Err(MnError::Resource(k, _)) => assert_eq!(k, "source-diagnostic"),
        other => panic!("over-long message must breach: {other:?}"),
    }
    // Empty diagnostics keep the legacy generic error (no fake snapshot).
    match sd::SourceDiagnosticSnapshot::capture("src", "mn.at", mk(0, "x"), op.clone()) {
        Err(MnError::Unsupported("frontend", m)) => assert_eq!(m, "frontend failed"),
        other => panic!("empty diagnostics stay legacy: {other:?}"),
    }
    // The payload cap (64 KiB): 40 causes x ~1.6 KiB each breaches it.
    match sd::SourceDiagnosticSnapshot::capture("src", "mn.at", mk(40, &"m".repeat(1600)), op) {
        Err(MnError::Resource(k, _)) => assert_eq!(k, "source-diagnostic"),
        other => panic!("payload over cap must breach: {other:?}"),
    }
}

#[test]
fn mn_sd911_raw_cause_projection_shape() {
    // The pre-911 display used the Stage DEBUG name — the projection must
    // keep those exact bytes for every stage vocabulary member.
    for (stage, debug) in [
        (Stage::Text, "Text"),
        (Stage::Bind, "Bind"),
        (Stage::Verify, "Verify"),
        (Stage::Capability, "Capability"),
        (Stage::Backend, "Backend"),
        (Stage::Link, "Link"),
        (Stage::Run, "Run"),
    ] {
        let d = Diagnostic::new(stage, "some.code", "boom", Span { start: 2, end: 4 });
        let c = RawCause::from_diagnostic(&d);
        assert_eq!(c.legacy_projection(), format!("{debug}/some.code: boom"));
    }
}

// ---------------------------------------------------------------------------
// PLAN-911 T-02: the mn-worker `source-v1` protocol legs (real binary).
// ---------------------------------------------------------------------------

use serde_json::{json, Value};

fn spawn_worker(req: &Value) -> (Value, Option<i32>) {
    use std::io::{BufRead, Write};
    use std::process::{Command, Stdio};
    let mut child = Command::new(env!("CARGO_BIN_EXE_mn-worker"))
        .arg("case-run")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mn-worker");
    child
        .stdin
        .take()
        .unwrap()
        .write_all(format!("{req}\n").as_bytes())
        .expect("write request");
    let mut line = String::new();
    {
        use std::io::BufRead as _;
        let mut reader = std::io::BufReader::new(child.stdout.take().unwrap());
        reader.read_line(&mut line).expect("read response");
    }
    let status = child.wait().expect("reap worker");
    let resp: Value = serde_json::from_str(line.trim()).expect("one JSON response line");
    (resp, status.code())
}

fn req_json(
    tag: &str,
    src: &str,
    steps: Value,
    diagnostics: Option<&str>,
) -> (Value, std::path::PathBuf) {
    use base64::Engine as _;
    let root = temp_dir(tag);
    let source_b64 = base64::engine::general_purpose::STANDARD.encode(src.as_bytes());
    let mut r = json!({
        "recipe": "ac/mir-native-scalar-v1",
        "nonce": "n911",
        "source_b64": source_b64,
        "entry": "d_main",
        "caps": [],
        "store_root": root.join("mn-store").display().to_string().replace('/', "\\"),
        "workdir": root.join("mn-work").display().to_string().replace('/', "\\"),
        "steps": steps,
    });
    if let Some(m) = diagnostics {
        r["diagnostics"] = json!(m);
    }
    r["request_hash"] = json!(sd::sha256_hex(r.to_string().as_bytes()));
    (r, root)
}

const WARM_SRC: &str =
    "fn alpha(x int) int {\n    return x * 2\n}\n\nfn main() int {\n    return alpha(5)\n}\n";

#[test]
fn mn_sd911_worker_cold_strict_rejection_shape() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let (req, root) = req_json(
        "w-cold-strict",
        B17.trim(),
        json!([{ "op": "run" }]),
        Some("source-v1"),
    );
    let (resp, code) = spawn_worker(&req);
    assert_eq!(
        code,
        Some(0),
        "a diagnosed rejection is a delivered verdict"
    );
    assert_eq!(resp["schema"], "mnworker/1");
    assert_eq!(resp["nonce"], "n911");
    assert_eq!(resp["request_hash"], req["request_hash"]);
    // Shape echo on ok; no execution result.
    assert_eq!(resp["ok"]["diagnostics_mode"], "source-v1");
    assert_eq!(resp["ok"]["diagnostics_schema"], "mn.source-diagnostic/1");
    assert!(resp["ok"].get("exit_code").is_none());
    assert!(resp["ok"].get("stdout_hex").is_none());
    assert!(resp["ok"].get("manifest_fp").is_none());
    // The typed fact + the legacy projection.
    let snap = &resp["source_error"];
    assert_eq!(snap["schema"], "mn.source-diagnostic/1");
    assert_eq!(snap["profile"], "scalar");
    assert_eq!(snap["causes"][0]["stage"], "verify");
    assert_eq!(snap["causes"][0]["code"], "source.type-mismatch");
    assert_eq!(
        snap["primary"].as_u64(),
        Some(0),
        "primary is JSON integer 0"
    );
    assert_eq!(resp["diagnostic"]["code"], "mn.unsupported.frontend");
    let want = format!(
        "Verify/source.type-mismatch: {}",
        snap["causes"][0]["message"].as_str().unwrap_or("")
    );
    assert_eq!(resp["diagnostic"]["message"].as_str().unwrap(), want);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_cold_strict_legal_echo() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let (req, root) = req_json(
        "w-cold-legal",
        NB17.trim(),
        json!([{ "op": "run" }]),
        Some("source-v1"),
    );
    let (resp, code) = spawn_worker(&req);
    assert_eq!(code, Some(0));
    assert_eq!(
        resp["ok"]["diagnostics_mode"], "source-v1",
        "a LEGAL program still echoes the mode"
    );
    assert_eq!(resp["ok"]["exit_code"].as_i64(), Some(2));
    assert!(resp.get("source_error").is_none());
    assert!(resp.get("diagnostic").is_none());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_cold_default_shape_unchanged() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let (req, root) = req_json("w-cold-default", B17.trim(), json!([{ "op": "run" }]), None);
    let (resp, code) = spawn_worker(&req);
    assert_eq!(code, Some(0));
    assert!(
        resp["ok"].get("diagnostics_mode").is_none(),
        "default mode gains no new field"
    );
    assert!(resp.get("source_error").is_none());
    assert_eq!(resp["diagnostic"]["code"], "mn.unsupported.frontend");
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_unknown_mode_is_named_error() {
    let (req, root) = req_json(
        "w-bad-mode",
        NB17.trim(),
        json!([{ "op": "run" }]),
        Some("source-v2"),
    );
    let (resp, code) = spawn_worker(&req);
    assert_ne!(code, Some(0));
    assert_eq!(resp["error"]["code"], "mn.worker.request");
    assert!(resp["error"]["message"]
        .as_str()
        .unwrap()
        .contains("unknown diagnostics mode"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_incremental_strict_first_failure_stops() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    // A legal edit, then B an ILLEGAL candidate, then C a repair whose
    // literal matches the accepted prefix. Strict mode stops at B: C is
    // never executed, the accepted prefix is the post-A source, and the
    // failing candidate's snapshot names B's own source.
    let steps = json!([
        { "op": "edit-source", "from": "x * 2", "to": "x + 2" },
        { "op": "edit-source", "from": "x + 2", "to": "true + 1" },
        { "op": "edit-source", "from": "x + 2", "to": "x * 3" },
        { "op": "run" },
    ]);
    let (req, root) = req_json("w-inc-strict", WARM_SRC, steps, Some("source-v1"));
    let (resp, code) = spawn_worker(&req);
    assert_eq!(code, Some(0), "a first-failure stop is a delivered verdict");
    let src_after_a = WARM_SRC.replacen("x * 2", "x + 2", 1);
    let src_after_b = src_after_a.replacen("x + 2", "true + 1", 1);
    // Step accounting: warmup + A + B; C and the trailing run are absent.
    let steps_out = resp["ok"]["steps"].as_array().expect("steps");
    assert_eq!(steps_out.len(), 3, "stopped at the first failure");
    assert_eq!(steps_out[0]["op"], "warmup");
    assert_eq!(steps_out[1]["op"], "edit-source");
    assert!(steps_out[1].get("refusal").is_none(), "A accepted");
    assert_eq!(steps_out[2]["op"], "edit-source");
    let refusal = &steps_out[2]["refusal"];
    assert_eq!(refusal["kind"], "language-diagnostics");
    assert_eq!(refusal["phase"], "frontend");
    assert!(
        refusal["source_error"].is_object(),
        "language refusals carry the snapshot in the step record too"
    );
    let bw = steps_out[2]["body_work"].as_object().expect("body work");
    assert!(bw.get("alpha").is_some(), "the failed body was reached");
    // Accepted prefix: post-A source, B not applied.
    assert_eq!(
        resp["ok"]["accepted_prefix"]["steps_applied"].as_u64(),
        Some(1)
    );
    assert_eq!(
        resp["ok"]["accepted_prefix"]["source_sha256"],
        sd::sha256_hex(src_after_a.as_bytes())
    );
    // No judged execution result.
    assert!(resp["ok"].get("exit_code").is_none());
    assert!(resp["ok"].get("stdout_hex").is_none());
    // The typed fact at the top level, bound to B's candidate.
    let snap = &resp["source_error"];
    assert_eq!(snap["operation"]["kind"], "warm-edit");
    assert_eq!(
        snap["operation"]["candidate_sha256"],
        sd::sha256_hex(src_after_b.as_bytes())
    );
    assert_eq!(
        snap["operation"]["base_sha256"],
        sd::sha256_hex(src_after_a.as_bytes())
    );
    assert_eq!(snap["causes"][0]["code"], "source.type-mismatch");
    assert_eq!(resp["diagnostic"]["code"], "mn.unsupported.frontend");
    // F-911-R1-01: the step axis is the 0-based index within the request's
    // step sequence — B is step 1 (A was 0).
    assert_eq!(snap["operation"]["step"].as_u64(), Some(1));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_incremental_default_keeps_legacy_covered_result() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    // The PRE-911 default behavior, byte-shape: B's refusal is recorded as
    // a `mn.incremental.refused` diagnostic, the sequence CONTINUES past
    // it (C repairs from the accepted prefix), and the judged result is
    // C's run — the exact covered-intermediate-state behavior source-v1
    // exists to end (kept here as the default compatibility control).
    let steps = json!([
        { "op": "edit-source", "from": "x * 2", "to": "x + 2" },
        { "op": "edit-source", "from": "x + 2", "to": "true + 1" },
        { "op": "edit-source", "from": "x + 2", "to": "x * 3" },
        { "op": "run" },
    ]);
    let (req, root) = req_json("w-inc-default", WARM_SRC, steps, None);
    let (resp, code) = spawn_worker(&req);
    assert_eq!(code, Some(0));
    assert_eq!(resp["diagnostic"]["code"], "mn.incremental.refused");
    // The response level reports no judged exit once a refusal occurred,
    // but the step records prove C and the trailing run REALLY executed
    // after B's refusal (the covered-intermediate-state behavior).
    let steps_out = resp["ok"]["steps"].as_array().expect("steps");
    assert_eq!(steps_out.len(), 5, "warmup + A + B(refused) + C + run");
    let run_rec = steps_out.last().unwrap();
    assert_eq!(run_rec["op"], "run");
    assert_eq!(
        run_rec["exit_code"].as_i64(),
        Some(15),
        "alpha(5) with x*3 = 15 — C ran past the refusal"
    );
    assert!(resp["ok"].get("exit_code").is_none());
    assert!(resp["ok"].get("accepted_prefix").is_none());
    assert!(resp.get("source_error").is_none());
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn mn_sd911_worker_incremental_warmup_reject_shapes() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped (link tools)");
        return;
    }
    let steps = json!([
        { "op": "edit-source", "from": "true + 1", "to": "1 + 1" },
        { "op": "run" },
    ]);
    // Strict: the warm-up rejection is the typed fact (shape 2, exit 0).
    let (req, root) = req_json(
        "w-warm-strict",
        B17.trim(),
        steps.clone(),
        Some("source-v1"),
    );
    let (resp, code) = spawn_worker(&req);
    assert_eq!(code, Some(0));
    assert_eq!(resp["ok"]["diagnostics_mode"], "source-v1");
    assert_eq!(
        resp["source_error"]["causes"][0]["code"],
        "source.type-mismatch"
    );
    assert_eq!(resp["diagnostic"]["code"], "mn.unsupported.frontend");
    let _ = std::fs::remove_dir_all(&root);
    // Default: the pre-911 infra error shape (exit FAILURE).
    let (req, root) = req_json("w-warm-default", B17.trim(), steps, None);
    let (resp, code) = spawn_worker(&req);
    assert_ne!(code, Some(0));
    assert_eq!(resp["error"]["code"], "mn.worker.incremental");
    assert!(resp.get("ok").is_none());
    let _ = std::fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------------------
// PLAN-911 T-05 V-13: the formal four-state leg in `source-v1` mode through
// the REAL worker over the frozen reference workload (115 functions) —
// registered under `::performance::` for `cargo t ace-performance`.
// ---------------------------------------------------------------------------
mod performance {
    use super::*;

    fn percentile(v: &[f64], p: f64) -> f64 {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.partial_cmp(b).unwrap());
        s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
    }

    fn counter(resp: &Value, k: &str) -> u64 {
        resp["ok"]["counters"][k].as_u64().unwrap_or(0)
    }

    /// One strict request against a given store root; returns (wall_ms, resp).
    fn strict_request(tag: &str, store: &std::path::Path, src: &str) -> (f64, Value) {
        use base64::Engine as _;
        let work = store.parent().unwrap().join(format!("w-{tag}"));
        let _ = std::fs::create_dir_all(&work);
        let mut r = json!({
            "recipe": "ac/mir-native-scalar-v1",
            "nonce": format!("n-{tag}"),
            "source_b64": base64::engine::general_purpose::STANDARD.encode(src.as_bytes()),
            "entry": "d_main",
            "caps": [],
            "store_root": store.display().to_string().replace('/', "\\"),
            "workdir": work.display().to_string().replace('/', "\\"),
            "steps": [{ "op": "run" }],
            "diagnostics": "source-v1",
        });
        r["request_hash"] = json!(sd::sha256_hex(r.to_string().as_bytes()));
        let t0 = std::time::Instant::now();
        let (resp, code) = spawn_worker(&r);
        let wall = t0.elapsed().as_secs_f64() * 1000.0;
        assert_eq!(code, Some(0), "{tag}: worker exit");
        assert_eq!(
            resp["ok"]["diagnostics_mode"], "source-v1",
            "{tag}: mode echo"
        );
        (wall, resp)
    }

    /// Four states, 30 samples each (MN818_FORMAL_SAMPLES overrides), dual
    /// gate: the structural facts of each state (fresh = all units emitted;
    /// restart-warm = zero codegen, all hits; body-edit = exactly one unit
    /// emitted, the rest hits; signature-reject = the typed fact with zero
    /// production) AND P90 <= 2000 ms on the restart-warm / body-edit /
    /// signature-reject states. `fresh` (a cold 115-function compile + 115
    /// COFF emits + link + run in a debug-tier worker process) is REPORTED,
    /// not gated — exactly the native tier's existing convention (the 818
    /// turnaround legs report cold absolutes; only the edit paths are
    /// gated) — and a DEFAULT-mode fresh control is sampled alongside to
    /// show the strict mode adds no cold cost. Preparation (store warm-up)
    /// is outside the sampled windows.
    #[test]
    #[cfg(windows)]
    fn mn_sd911_source_v1_four_state_p90_30() {
        if !common::tools_available() {
            eprintln!("infra-blocked: skipped (link tools)");
            return;
        }
        let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);
        let root = temp_dir("perf4");
        let wl0 = ace_native::workload::generate(0);
        let units = 115u64;

        // ---- fresh: a NEW store per sample, cold request ------------------
        let mut fresh = Vec::with_capacity(samples);
        for i in 0..samples {
            let store = root.join(format!("fresh-{i}")).join("mn-store");
            std::fs::create_dir_all(&store).unwrap();
            let (w, resp) = strict_request(&format!("fresh-{i}"), &store, &wl0.src);
            assert_eq!(resp["ok"]["exit_code"].as_i64(), Some(wl0.expected as i64));
            assert_eq!(
                counter(&resp, "codegen_emit_units"),
                units,
                "fresh emits every unit"
            );
            assert_eq!(counter(&resp, "cache_hit_coff"), 0);
            fresh.push(w);
        }

        // ---- restart-warm: ONE store warmed once, every sample a new
        // process restoring the same source (zero codegen, all hits) -------
        let warm_store = root.join("warm").join("mn-store");
        std::fs::create_dir_all(&warm_store).unwrap();
        let _ = strict_request("warmup", &warm_store, &wl0.src); // preparation, unsampled
        let mut restart = Vec::with_capacity(samples);
        for i in 0..samples {
            let (w, resp) = strict_request(&format!("restart-{i}"), &warm_store, &wl0.src);
            assert_eq!(resp["ok"]["exit_code"].as_i64(), Some(wl0.expected as i64));
            assert_eq!(
                counter(&resp, "codegen_emit_units"),
                0,
                "restart-warm: no codegen"
            );
            assert_eq!(counter(&resp, "cache_hit_coff"), units);
            restart.push(w);
        }

        // ---- body-edit: each sample a NEW revision of fn_041's body on the
        // warm store — exactly one unit re-emitted, 114 hits --------------
        let mut edit = Vec::with_capacity(samples);
        for n in 1..=samples {
            let wl = ace_native::workload::generate(n as i64);
            let (w, resp) = strict_request(&format!("edit-{n}"), &warm_store, &wl.src);
            assert_eq!(
                resp["ok"]["exit_code"].as_i64(),
                Some(wl.expected as i64),
                "rev {n}"
            );
            assert_eq!(counter(&resp, "codegen_emit_units"), 1, "rev {n}: one unit");
            assert_eq!(
                counter(&resp, "cache_hit_coff"),
                units - 1,
                "rev {n}: the rest hit"
            );
            edit.push(w);
        }

        // ---- signature-reject: the frozen signature-conflict source — a
        // REAL frontend rejection returns the typed fact with zero guest
        // production (no codegen/link/run) ---------------------------------
        let conflict = ace_native::workload::signature_conflict_source();
        let mut reject = Vec::with_capacity(samples);
        for i in 0..samples {
            let (w, resp) = strict_request(&format!("reject-{i}"), &warm_store, &conflict);
            assert!(resp["source_error"].is_object(), "typed fact present");
            assert_eq!(resp["diagnostic"]["code"], "mn.unsupported.frontend");
            assert!(resp["ok"].get("exit_code").is_none());
            assert_eq!(counter(&resp, "codegen_emit_units"), 0);
            assert_eq!(counter(&resp, "link_invocations"), 0);
            assert_eq!(counter(&resp, "run_invocations"), 0);
            reject.push(w);
        }

        // ---- fresh DEFAULT-mode control (5 samples): the strict mode must
        // not change the cold cost class ------------------------------------
        let mut fresh_default = Vec::new();
        for i in 0..5usize {
            use base64::Engine as _;
            let store = root.join(format!("freshd-{i}")).join("mn-store");
            std::fs::create_dir_all(&store).unwrap();
            let work = root.join(format!("freshd-{i}")).join("w");
            let _ = std::fs::create_dir_all(&work);
            let mut r = json!({
                "recipe": "ac/mir-native-scalar-v1",
                "nonce": format!("nd-{i}"),
                "source_b64": base64::engine::general_purpose::STANDARD.encode(wl0.src.as_bytes()),
                "entry": "d_main",
                "caps": [],
                "store_root": store.display().to_string().replace('/', "\\"),
                "workdir": work.display().to_string().replace('/', "\\"),
                "steps": [{ "op": "run" }],
            });
            r["request_hash"] = json!(sd::sha256_hex(r.to_string().as_bytes()));
            let t0 = std::time::Instant::now();
            let (resp, code) = spawn_worker(&r);
            fresh_default.push(t0.elapsed().as_secs_f64() * 1000.0);
            assert_eq!(code, Some(0));
            assert_eq!(resp["ok"]["exit_code"].as_i64(), Some(wl0.expected as i64));
        }

        let mut red = Vec::new();
        for (name, v, gated) in [
            ("fresh", &fresh, false),
            ("fresh(default-mode control)", &fresh_default, false),
            ("restart-warm", &restart, true),
            ("body-edit", &edit, true),
            ("signature-reject", &reject, true),
        ] {
            let (p50, p90, p99) = (
                percentile(v, 50.0),
                percentile(v, 90.0),
                percentile(v, 99.0),
            );
            eprintln!(
                "mn911 source-v1 {name}: P50/P90/P99 = {p50:.1}/{p90:.1}/{p99:.1} ms ({} samples, debug tier{})",
                v.len(),
                if gated { ", gated P90<=2000" } else { ", reported" }
            );
            if gated && p90 > 2000.0 {
                red.push(format!("{name} P90 {p90:.1} ms > 2000 ms"));
            }
        }
        let _ = std::fs::remove_dir_all(&root);
        assert!(red.is_empty(), "V-13 P90 gate red: {red:?}");
    }
}
