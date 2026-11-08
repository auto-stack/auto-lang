//! PLAN-939 T-09/T-10 worker-level recovery/invalidation/budget tests
//! (V-05/V-08/V-09/V-10, AC-03/05/06): the resource certification lane
//! driven through the REAL in-process worker entry
//! (`worker_source::execute`) — every query is a full
//! restore→resolve→prepare→publish request against a real store.
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
//! * T-10 budget/atomicity/churn: an over-budget request cancels with a
//!   typed refusal that PRESERVES the accepted root and still completes
//!   its cleanup under the exhausted quota; a pointer-replace failure
//!   mid-publication keeps the accepted root and the retry publishes;
//!   a real revision churn converges under GC with a live lease pinning
//!   an OLD root until its last release (the 512-scale store-level churn
//!   anchor is the 797-lane `abi_cache_512_revision_churn_gc_convergence`,
//!   unchanged and green in this same group).

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
) -> Result<ace_abi::worker::OkOut, ace_abi::worker::DiagOut> {
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
    worker_source::execute(&request)
}

fn expect_query(
    tag: &str,
    deck: &serde_json::Value,
    store_root: &Path,
    leg: &str,
) -> ace_abi::worker::OkOut {
    query(tag, deck, store_root, leg).expect("the resource certification request succeeds")
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
    let q0 = expect_query("t09-q0", &deck("record-mut-fail"), &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(70));
    let root_after_q0 = pointer_bytes(&store_root);

    // The FIRST edit after restore FAILS verification (bool seed where the
    // reviewed i32 lane is required): the worker answers the source
    // diagnostics, and NOTHING is published (no live increment).
    let mut broken = deck("record-mut-fail");
    broken["source"] = serde_json::json!(
        "type Counter { value int }\nfn main() int {\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, true)\n    return now + x.value\n}"
    );
    let q1 = expect_query("t09-q1", &broken, &store_root, "native");
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
    let q2 = expect_query("t09-q2", &deck("record-mut"), &store_root, "native");
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
    let q3 = expect_query("t09-q3", &deck("record-mut-fail"), &store_root, "native");
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

    // --- MISSING axis: the required SESSION pages are gone. The provider
    // object is a reproducible tool product (a clean cache miss recompiles
    // it deterministically), but the accepted root's session artifacts are
    // NOT re-producible: the restore refuses with the typed missing-page
    // error — never a silent cold start that republishes a fresh session
    // as if the accepted root never existed. ---
    let missing_store = dir.join("store-missing");
    let q0 = expect_query("t09p-a0", &base, &missing_store, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(42));
    let root_after_q0 = pointer_bytes(&missing_store);

    let files = blob_files(&missing_store);
    assert!(!files.is_empty(), "publication stored real pages");
    for p in &files {
        std::fs::remove_file(p).unwrap();
    }

    let refused = worker_source::execute(&request_for(&missing_store, "t09p-a1"))
        .expect_err("missing required pages must refuse, never cold-start");
    assert_eq!(
        refused.code, "restore",
        "the refusal is the controlled restore error: {}",
        refused.message
    );
    assert!(
        refused.message.contains("foreign.restore."),
        "the refusal names the restore discipline: {}",
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
    let q0c = expect_query("t09p-b0", &base, &corrupt_store, "native");
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
    let q0f = expect_query("t09p-c0", &base, &forged_store, "native");
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
    let q0 = expect_query("t09i-q0", &deck_json, &store_root, "native");
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
    let q1 = expect_query("t09i-q1", &deck_json, &store_root, "native");
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
    let q2 = expect_query("t09i-q2", &deck_json, &store_root, "native");
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
    let q3 = expect_query("t09i-q3", &deck_json, &store_root, "native");
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
    let q4 = expect_query("t09i-q4", &bias_deck, &store_root, "native");
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

/// T-10 (V-05/V-10, AC-03/06): an over-budget request CANCELS with a typed
/// refusal that preserves the accepted root and still completes its
/// cleanup under the exhausted quota — cleanup consumes no remaining
/// budget. The produced-output budget is host-scoped via
/// `ABI_TEST_PRODUCED_BUDGET_BYTES` (the host-only tightening seam); this
/// test reserves the whole test pool (nextest override) while the env is
/// set, exactly like the PLAN-938 publication controls.
#[test]
fn abi_resource939_over_budget_cancellation_preserves_root_and_cleans_without_quota() {
    let dir = ace_abi::run::scratch("abi_r939_over_budget");
    let store_root = dir.join("store");
    let base = deck("record-mut");

    // q0: establish the accepted root under the normal (default) budget.
    let q0 = expect_query("t10b-q0", &base, &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(42));
    let root_after_q0 = pointer_bytes(&store_root);

    // Squeeze the produced-output budget to a single byte: the very next
    // tool output exceeds the request budget and the request must CANCEL
    // (reserve-before-allocation), not produce past it.
    const BUDGET: &str = "ABI_TEST_PRODUCED_BUDGET_BYTES";
    struct Restore(Option<std::ffi::OsString>);
    impl Drop for Restore {
        fn drop(&mut self) {
            match &self.0 {
                Some(value) => std::env::set_var(BUDGET, value),
                None => std::env::remove_var(BUDGET),
            }
        }
    }
    let _restore = Restore(std::env::var_os(BUDGET));
    std::env::set_var(BUDGET, "1");

    let mut squeezed = base.clone();
    squeezed["source"] = serde_json::json!(
        "type Counter { value int }\nfn main() int {\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, 2)\n    return now + x.value\n}"
    );
    let request = ace_abi::worker::WorkerRequest {
        recipe: "abi/resource-foreign-certify-v1".into(),
        nonce: "t10b-q1".into(),
        request_hash: "t10b-q1".into(),
        deck_b64: base64::engine::general_purpose::STANDARD
            .encode(serde_json::to_vec(&squeezed).unwrap()),
        leg: "native".into(),
        store_root: store_root.to_str().unwrap().into(),
        steps: vec![StepIn {
            op: "query".into(),
            from: None,
            to: None,
        }],
    };
    let refused = worker_source::execute(&request)
        .expect_err("an over-budget request must cancel, never overrun");
    assert_eq!(
        refused.code, "foreign.resource",
        "the budget cancellation is the typed resource refusal: {}",
        refused.message
    );
    assert_eq!(
        pointer_bytes(&store_root),
        root_after_q0,
        "the cancelled request leaves the accepted root untouched"
    );
    // The refusal RETURNED (scratch cleanup finished) — cleanup needs no
    // remaining quota; a `store.cleanup-pending` code here would mean the
    // exhausted budget starved the cleanup path itself.
    drop(_restore);

    // The same edit publishes normally once the budget is restored — the
    // cancellation wedged neither the store nor the session.
    let q2 = expect_query("t10b-q2", &squeezed, &store_root, "native");
    assert_eq!(report_of(&q2).actual_exit, Some(44));
    assert_ne!(pointer_bytes(&store_root), root_after_q0);
}

/// T-10 (V-10, AC-06): a pointer-replace failure MID-PUBLICATION keeps the
/// accepted root (candidate publication is atomic at the accepted root);
/// the retry after the obstacle clears publishes the same edit.
#[test]
fn abi_resource939_pointer_replace_failure_keeps_accepted_root() {
    let dir = ace_abi::run::scratch("abi_r939_pointer_fault");
    let store_root = dir.join("store");
    let base = deck("record-mut");

    let q0 = expect_query("t10c-q0", &base, &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(42));
    let root_after_q0 = pointer_bytes(&store_root);

    // Hold the pointer file open WITHOUT FILE_SHARE_DELETE: the
    // publication's atomic pointer replace fails (sharing violation) at
    // the final step — after artifacts are staged.
    let pointer = store_root.join("source939-current-native.json");
    {
        use std::os::windows::fs::OpenOptionsExt;
        let _blocker = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x1 | 0x2)
            .open(&pointer)
            .expect("the accepted pointer is openable read-only");

        let mut edited = base.clone();
        edited["source"] = serde_json::json!(
            "type Counter { value int }\nfn main() int {\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, 2)\n    return now + x.value\n}"
        );
        let request = ace_abi::worker::WorkerRequest {
            recipe: "abi/resource-foreign-certify-v1".into(),
            nonce: "t10c-q1".into(),
            request_hash: "t10c-q1".into(),
            deck_b64: base64::engine::general_purpose::STANDARD
                .encode(serde_json::to_vec(&edited).unwrap()),
            leg: "native".into(),
            store_root: store_root.to_str().unwrap().into(),
            steps: vec![StepIn {
                op: "query".into(),
                from: None,
                to: None,
            }],
        };
        let refused = worker_source::execute(&request)
            .expect_err("a blocked pointer replace must fail the request");
        assert_eq!(
            refused.code, "store",
            "the pointer fault is a typed store refusal: {}",
            refused.message
        );
        assert_eq!(
            std::fs::read(&pointer).unwrap(),
            root_after_q0,
            "the failed publication leaves the accepted pointer bytes intact"
        );
    } // blocker dropped

    // The retry publishes the SAME edit cleanly.
    let mut edited = base.clone();
    edited["source"] = serde_json::json!(
        "type Counter { value int }\nfn main() int {\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, 2)\n    return now + x.value\n}"
    );
    let q2 = expect_query("t10c-q2", &edited, &store_root, "native");
    assert_eq!(report_of(&q2).actual_exit, Some(44));
    assert_ne!(pointer_bytes(&store_root), root_after_q0);
}

/// T-10 (V-10, AC-06): a REAL revision churn through the full
/// certification path converges under forced GC — unreferenced revisions
/// are reclaimed, a live lease pins an OLD root against collection until
/// its last release, and the current root keeps delivering. The
/// 512-scale store-level churn anchor is the 797-lane
/// `abi_cache_512_revision_churn_gc_convergence` (same group, green);
/// this test adds the resource-lane root churn over the real worker
/// entry (each revision = a full restore→prepare→publish cycle).
#[test]
fn abi_resource939_revision_churn_gc_convergence_and_lease() {
    let dir = ace_abi::run::scratch("abi_r939_churn_gc");
    let store_root = dir.join("store");
    let base = deck("record-mut");
    let seeded = |seed: i32| {
        let mut deck = base.clone();
        deck["source"] = serde_json::json!(format!(
            "type Counter {{ value int }}\nfn main() int {{\n    var x Counter = Counter(20)\n    let now int = t939_bump(x.mut, {seed})\n    return now + x.value\n}}"
        ));
        deck
    };

    // q0: establish the first root and PIN it with a real lease (an old
    // reader that still owns the generation-A closure).
    let q0 = expect_query("t10d-q0", &seeded(1), &store_root, "native");
    assert_eq!(report_of(&q0).actual_exit, Some(42));
    let pointer: serde_json::Value = serde_json::from_slice(&pointer_bytes(&store_root)).unwrap();
    let first_artifact = pointer["stored"]["artifact_id"]
        .as_str()
        .unwrap()
        .to_string();

    let store = ace_artifacts::store::ManagedStore::open(&store_root).unwrap();
    let lease = store
        .acquire_lease("t10-old-generation-pin", 3600, Some(&first_artifact))
        .expect("the lease grants over the first revision");

    // The churn: 24 DISTINCT revisions (seed 2..=25 -> exit 44..90), each
    // a full worker certification cycle publishing a new root.
    let mut mid_reclaims: usize = 0;
    for seed in 2..=25i32 {
        let out = match query(
            &format!("t10d-s{seed}"),
            &seeded(seed),
            &store_root,
            "native",
        ) {
            Ok(out) => out,
            Err(e) => panic!("seed {seed} failed: {e:?}"),
        };
        assert_eq!(
            report_of(&out).actual_exit,
            Some(40 + 2 * seed),
            "revision {seed} delivers its own oracle"
        );
        // Periodic forced GC: the intermediate (unreferenced) revisions
        // are reclaimed while the LEASED first revision and the CURRENT
        // root survive.
        if seed % 8 == 0 {
            let receipt = ace_artifacts::store::gc::run_gc(
                &store,
                ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
            )
            .unwrap();
            if receipt.deleted_chunks > 0 {
                mid_reclaims += receipt.deleted_chunks;
            }
            assert!(
                store.get_artifact_by_id(&first_artifact).unwrap().is_some(),
                "the leased first revision survives every forced GC"
            );
        }
    }
    assert!(
        mid_reclaims > 0,
        "the forced GC reclaimed unreferenced revisions (reclaimed={mid_reclaims})"
    );
    assert!(
        store.get_artifact_by_id(&first_artifact).unwrap().is_some(),
        "the lease still pins the first revision"
    );

    // The current root is warm (P0 production) and delivers.
    let warm = expect_query("t10d-warm", &seeded(25), &store_root, "native");
    assert_eq!(report_of(&warm).actual_exit, Some(90));
    assert_eq!(
        warm.steps[0].compute_calls, 0,
        "the current root is a warm hit"
    );

    // Last release: dropping the pin makes the first revision unreachable
    // — the NEXT forced GC really reclaims it (convergence, no leak).
    drop(lease);
    let receipt = ace_artifacts::store::gc::run_gc(
        &store,
        ace_artifacts::store::gc::GcConfig { high: 0, low: 0 },
    )
    .unwrap();
    assert!(
        receipt.deleted_chunks > 0,
        "the released first revision is reclaimed: {receipt:?}"
    );
    assert!(
        store.get_artifact_by_id(&first_artifact).unwrap().is_none(),
        "the old generation is gone after its last release"
    );
    // The CURRENT root still delivers after the reclaim.
    let after = expect_query("t10d-after", &seeded(25), &store_root, "native");
    assert_eq!(report_of(&after).actual_exit, Some(90));
}
