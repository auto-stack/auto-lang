//! PLAN-939 T-09 worker-level recovery/invalidation tests (V-08/V-09,
//! AC-05): the resource certification lane driven through the REAL
//! in-process worker entry (`worker_source::execute`) — every query is a
//! full restore→resolve→prepare→publish request against a real store.
//!
//! * first-error discipline: a restored session's FIRST body edit that
//!   fails verification produces the source diagnostics and publishes
//!   NOTHING (the accepted root survives byte-identical); the fixed edit
//!   then publishes and the ORIGINAL body re-delivers afterwards.
//! * required pages: a store whose CAS blobs are gone (or corrupted)
//!   refuses restore with a controlled error — never a silent cold start
//!   that republishes as if nothing happened.
//! * precise reader invalidation: a body edit re-produces only the
//!   edited function's artifacts (per-stage producer counts pinned);
//!   unchanged warm queries produce NOTHING; a provider-only transition
//!   (same contract, new provider object) re-produces the provider and
//!   rebinds without re-producing the unchanged source body.

use ace_abi::worker::StepIn;
use ace_abi::worker_source;
use base64::Engine as _;
use std::path::{Path, PathBuf};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn deck_path(name: &str) -> PathBuf {
    manifest().join(format!(
        "../../../../crates/auto-lang/test/cases/ace-ca-d-resources-939/src/{name}.atomdata"
    ))
}

/// One full worker certification query against `store_root` with `deck`
/// (already JSON-mutated by the caller). Returns the typed worker output.
fn query(
    tag: &str,
    deck: &serde_json::Value,
    store_root: &Path,
    leg: &str,
) -> ace_abi::worker::OkOut {
    let request = ace_abi::worker::WorkerRequest {
        recipe: "abi/resource-foreign-certify-v1".into(),
        nonce: tag.into(),
        request_hash: tag.into(),
        deck_b64: base64::engine::general_purpose::STANDARD
            .encode(serde_json::to_vec(deck).unwrap()),
        leg: leg.into(),
        store_root: store_root.to_str().unwrap().into(),
        steps: vec![StepIn {
            op: "query".into(),
            from: None,
            to: None,
        }],
    };
    worker_source::execute(&request).expect("the resource certification request succeeds")
}

fn report_of(out: &ace_abi::worker::OkOut) -> &ace_abi::worker_source::SourceReport {
    out.source_import
        .as_ref()
        .expect("resource query carries its report")
}

fn deck(name: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(deck_path(name)).unwrap()).unwrap()
}

/// The store pointer's raw bytes (identity of the accepted root).
fn pointer_bytes(store_root: &Path) -> Vec<u8> {
    std::fs::read(store_root.join("source939-current-native.json")).unwrap()
}

#[test]
fn abi_resource939_restore_first_edit_fails_publishes_nothing_then_fix_and_recover() {
    let dir = ace_abi::run::scratch("abi_r939_first_error");
    let store_root = dir.join("store");

    // q0: publish the reviewed FAILING scenario (exit 70 with the pinned
    // -2 transport) — the deck the first edit below fails against.
    let q0 = query("t09-q0", &deck("record-mut-fail"), &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(70));
    let root_after_q0 = pointer_bytes(&store_root);

    // The FIRST edit after restore FAILS verification (bool seed where the
    // reviewed i32 lane is required): the worker answers the source
    // diagnostics, and NOTHING is published (no live increment).
    let mut broken = deck("record-mut-fail");
    broken["source"] = serde_json::json!(
        "type Counter { value int }\nfn main() int {\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, true)\n    return now + x.value\n}"
    );
    let q1 = query("t09-q1", &broken, &store_root, "native");
    assert_eq!(
        q1.steps[0].outcome, "diags",
        "the failed first edit is a verify refusal"
    );
    let r1 = report_of(&q1);
    assert_eq!(
        r1.diagnostic_stage, "verify",
        "the refusal is the source verifier, not an infra error"
    );
    assert!(
        !r1.diagnostics.is_empty(),
        "the refusal carries its diagnostics"
    );
    assert_eq!(
        pointer_bytes(&store_root),
        root_after_q0,
        "a failed edit publishes NOTHING — the accepted root survives byte-identical"
    );
    // A refusal carries no execution evidence (the admission contract).
    assert_eq!(r1.actual_exit, None);
    assert!(r1.actual_stderr.is_empty());
    assert!(r1.actual_events.is_empty());

    // The FIX: the seed returns to the reviewed legal value (1) and the
    // deck's reference scenario switches with it — the fixed deck is
    // byte-exact the reviewed record-mut deck, so the independent
    // reference oracle mirrors it under the review table.
    let q2 = query("t09-q2", &deck("record-mut"), &store_root, "native");
    let r2 = report_of(&q2);
    assert_eq!(q2.steps[0].outcome, "ok");
    assert_eq!(r2.actual_exit, Some(42));
    assert_eq!(
        r2.reference_exit,
        Some(42),
        "the independent reference mirrors the fix"
    );
    let root_after_q2 = pointer_bytes(&store_root);
    assert_ne!(root_after_q2, root_after_q0, "the fix really published");

    // The RE-RESTORE of the reviewed FAILING scenario re-delivers its
    // reviewed failure transport (the record-mut-fail deck, exit 70 with
    // the pinned -2 status).
    let q3 = query("t09-q3", &deck("record-mut-fail"), &store_root, "native");
    let r3 = report_of(&q3);
    assert_eq!(q3.steps[0].outcome, "ok");
    assert_eq!(r3.actual_exit, Some(70));
    assert_eq!(r3.reference_exit, Some(70));
    assert_eq!(
        r3.actual_stderr.trim(),
        "source939.call=t939_bump\r\nforeign.status=-2",
        "the restored scenario carries exactly the reviewed failure transport"
    );
}

#[test]
fn abi_resource939_missing_or_corrupt_required_pages_refuse_restore_never_cold_start() {
    let dir = ace_abi::run::scratch("abi_r939_required_pages");
    let base = deck("record-mut");
    // The CAS lays blobs out in shard subdirectories: blocks/blobs/<shard>/<file>.
    let blob_files = |store: &Path| -> Vec<PathBuf> {
        let mut files = Vec::new();
        for shard in std::fs::read_dir(store.join("blocks").join("blobs"))
            .expect("the CAS blob dir exists after publication")
            .flatten()
        {
            if !shard.file_type().unwrap().is_dir() {
                continue;
            }
            for e in std::fs::read_dir(shard.path()).unwrap().flatten() {
                if e.file_type().unwrap().is_file()
                    && e.path().extension().and_then(|x| x.to_str()) != Some("tmp")
                {
                    files.push(e.path());
                }
            }
        }
        files
    };
    let request_for = |store_root: &Path, nonce: &str| ace_abi::worker::WorkerRequest {
        recipe: "abi/resource-foreign-certify-v1".into(),
        nonce: nonce.into(),
        request_hash: nonce.into(),
        deck_b64: base64::engine::general_purpose::STANDARD
            .encode(serde_json::to_vec(&base).unwrap()),
        leg: "native".into(),
        store_root: store_root.to_str().unwrap().into(),
        steps: vec![StepIn {
            op: "query".into(),
            from: None,
            to: None,
        }],
    };

    // --- MISSING axis: the required store pages (session artifacts AND
    // the provider generation entry's blobs) are gone. The ledger refuses
    // the half-state as an UNVERIFIABLE entry — a typed refusal, never a
    // silent cold start that recompiles/republishes under it. ---
    let missing_store = dir.join("store-missing");
    let q0 = query("t09p-a0", &base, &missing_store, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(42));
    let root_after_q0 = pointer_bytes(&missing_store);

    let files = blob_files(&missing_store);
    assert!(!files.is_empty(), "publication stored real pages");
    for p in &files {
        std::fs::remove_file(p).unwrap();
    }

    let refused = worker_source::execute(&request_for(&missing_store, "t09p-a1"))
        .expect_err("missing required pages must refuse, never cold-start");
    assert!(
        refused.message.contains("unverifiable"),
        "the store names the half-missing state: {} ({})",
        refused.code,
        refused.message
    );
    assert_eq!(
        pointer_bytes(&missing_store),
        root_after_q0,
        "the refusal leaves the accepted pointer untouched"
    );

    // --- CORRUPT axis: every blob NAME survives but the chunk bytes are
    // garbage — the self-verifying envelopes must reject the tampered
    // pages instead of decoding them (伪 facts 不能成功 restore). ---
    let corrupt_store = dir.join("store-corrupt");
    let q0c = query("t09p-b0", &base, &corrupt_store, "native");
    assert_eq!(report_of(&q0c).actual_exit, Some(42));
    let root_after_q0c = pointer_bytes(&corrupt_store);
    let mut tampered = 0usize;
    for path in blob_files(&corrupt_store) {
        let mut bytes = std::fs::read(&path).unwrap();
        assert!(bytes.len() > 8, "a stored chunk is a real framed envelope");
        // flip bytes in the MIDDLE (keep the file/name layout intact)
        for b in bytes.iter_mut().skip(8).step_by(2) {
            *b = b.wrapping_add(1);
        }
        std::fs::write(&path, bytes).unwrap();
        tampered += 1;
    }
    assert!(tampered > 0);
    let refused = worker_source::execute(&request_for(&corrupt_store, "t09p-b1"))
        .expect_err("a tampered required page must refuse restore");
    assert_eq!(
        refused.code, "infra",
        "the tampered page is a typed integrity refusal: {}",
        refused.message
    );
    assert_eq!(
        pointer_bytes(&corrupt_store),
        root_after_q0c,
        "the tamper refusal also leaves the pointer untouched"
    );

    // --- FORGED POINTER axis: the pointer record itself is tampered
    // (伪 credential / wrong schema) — the metadata decode must refuse
    // restore instead of accepting the forged root. ---
    let forged_store = dir.join("store-forged");
    let q0f = query("t09p-c0", &base, &forged_store, "native");
    assert_eq!(report_of(&q0f).actual_exit, Some(42));
    let pointer = forged_store.join("source939-current-native.json");
    let mut forged = std::fs::read(&pointer).unwrap();
    for b in forged.iter_mut().skip(16).step_by(3) {
        *b = b.wrapping_add(1);
    }
    std::fs::write(&pointer, &forged).unwrap();
    let refused = worker_source::execute(&request_for(&forged_store, "t09p-c1"))
        .expect_err("a forged pointer record must refuse restore");
    assert_eq!(
        refused.code, "restore",
        "the forged pointer is a controlled restore refusal: {}",
        refused.message
    );
    assert_eq!(
        std::fs::read(&pointer).unwrap(),
        forged,
        "the refusal does not overwrite the forged record (no silent repair)"
    );
}

#[test]
fn abi_resource939_body_edit_reproduces_only_the_edited_function() {
    let dir = ace_abi::run::scratch("abi_r939_increment_axes");
    let store_root = dir.join("store");
    let mut deck_json = deck("callback-reentrant");

    // q0: fresh publish — the full two-function module (main + cb). The
    // per-stage producer counts are PINNED (T-09 全stage真实计数): two
    // functions parsed/HIR'd/lowered/emitted, zero C fragments (native leg).
    let q0 = query("t09i-q0", &deck_json, &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(21));
    let r0 = report_of(&q0);
    assert_eq!(
        (
            r0.source_parse,
            r0.solver,
            r0.hir,
            r0.mir,
            r0.native,
            r0.c_fragment
        ),
        (2, 0, 2, 2, 2, 0)
    );

    // q1: warm hit — NOTHING is produced: P0 on EVERY stage (the warm hit
    // serves the accepted root without even re-parsing the body).
    let q1 = query("t09i-q1", &deck_json, &store_root, "native");
    let r1 = report_of(&q1);
    assert_eq!(
        (
            r1.source_parse,
            r1.solver,
            r1.hir,
            r1.mir,
            r1.native,
            r1.c_fragment
        ),
        (0, 0, 0, 0, 0, 0)
    );

    // q2: a semantics-preserving body edit to MAIN only (`0 +` inserted).
    // Only main re-produces: exactly one parse/HIR/lower/emit; cb's
    // readers and every signature reader stay cached (its fragments are
    // NOT re-counted on the native leg).
    deck_json["source"] = serde_json::json!(deck_json["source"]
        .as_str()
        .unwrap()
        .replace("return sum", "return 0 + sum"));
    let q2 = query("t09i-q2", &deck_json, &store_root, "native");
    let r2 = report_of(&q2);
    assert_eq!(r2.actual_exit, Some(21), "the edit is semantics-preserving");
    assert_eq!(
        (
            r2.source_parse,
            r2.solver,
            r2.hir,
            r2.mir,
            r2.native,
            r2.c_fragment
        ),
        (1, 0, 1, 1, 1, 0),
        "exactly ONE function re-parsed/re-lowered/re-emitted"
    );

    // q3: the edited module warm again — P0 across every stage.
    let q3 = query("t09i-q3", &deck_json, &store_root, "native");
    let r3 = report_of(&q3);
    assert_eq!(
        (
            r3.source_parse,
            r3.solver,
            r3.hir,
            r3.mir,
            r3.native,
            r3.c_fragment
        ),
        (0, 0, 0, 0, 0, 0)
    );
    assert_eq!(r3.actual_exit, Some(21));

    // Provider axis: the SAME edited source with a different provider
    // object (bias 1) — the provider object re-produces (its compile is
    // the provider lane, not a source stage) and the binding generation
    // increments, but the UNCHANGED source body's readers stay cached
    // (§5.5: provider body change with unchanged contract is P0 for every
    // non-reader of the provider body).
    let mut bias_deck = deck_json.clone();
    bias_deck["provider_bias"] = serde_json::json!(1);
    let q4 = query("t09i-q4", &bias_deck, &store_root, "native");
    let r4 = report_of(&q4);
    assert_eq!(
        (
            r4.source_parse,
            r4.solver,
            r4.hir,
            r4.mir,
            r4.native,
            r4.c_fragment
        ),
        (0, 0, 0, 0, 0, 0),
        "a provider-only transition re-produces the provider object, never the unchanged body"
    );
    assert_ne!(
        r4.provider_digest, r0.provider_digest,
        "bias 1 is a distinct provider object"
    );
    assert_eq!(
        r4.actual_exit,
        Some(21),
        "the bias transition still delivers"
    );
    let pointer: serde_json::Value = serde_json::from_slice(&pointer_bytes(&store_root)).unwrap();
    assert_eq!(
        pointer["binding_generation"].as_u64().unwrap(),
        2,
        "the provider transition increments the binding generation"
    );
}
