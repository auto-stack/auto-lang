//! T-04 tier: real unit cache, dependency/invalidation, restore admission,
//! wrong-identity/concurrency/GC (PLAN-798 V-05 envelope leg, V-06..V-08)
//! plus the cross-process worker restore.

use auto_av_core::contract::fnv1a64;
use auto_av_probe::cache::{
    self, abc_unit_key, caps_fingerprint, envelope_unwrap, envelope_wrap, get_enveloped_opt,
    open_store, put_enveloped, EnvelopeIdentity,
};
use auto_av_probe::driver::Driver;

const BASE: &str = "fn helper(a int, b int) int {\n    return a * 10 + b\n}\n\nfn main() int {\n    if helper(3, 4) != 34 {\n        return 1\n    }\n    return 0\n}\n";

fn tmpdir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("av798-cache-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

/// V-06: fresh → same-process hot → restart (new driver, warm disk) →
/// cross-project sharing; hits skip mir_build/abc_emit, ALWAYS re-execute
/// and re-compare.
#[test]
fn av_cache_hit_lifecycle() {
    let root = tmpdir("lifecycle");
    let mut d = Driver::with_store(&root, "proj-a").unwrap();
    let r1 = d.request(BASE, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(*r1.counters.get("cache_hit_abc").unwrap_or(&0), 0);

    let r2 = d.request(BASE, "d_main", &[]);
    assert!(r2.ok());
    assert_eq!(*r2.counters.get("cache_hit_abc").unwrap_or(&0), 2);
    assert_eq!(*r2.counters.get("mir_lower_modules").unwrap_or(&0), 0);
    assert_eq!(*r2.counters.get("mir_units_persisted").unwrap_or(&0), 0);
    assert_eq!(*r2.counters.get("abc_units_emitted").unwrap_or(&0), 0);
    assert!(
        *r2.counters.get("av_execute").unwrap_or(&0) > 0,
        "hit still executes"
    );
    assert_eq!(
        r2.value_i32(),
        r1.value_i32(),
        "independent re-comparison each request"
    );

    // Restart: a NEW driver instance (cold in-process state) over the same
    // store root.
    let mut d3 = Driver::with_store(&root, "proj-a").unwrap();
    let r3 = d3.request(BASE, "d_main", &[]);
    assert!(r3.ok(), "restart/disk-hot: {:?}", r3.diag);
    assert_eq!(
        *r3.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "disk-hot hits"
    );

    // Cross-project: a different project label shares the same logical
    // units (content identity, not project identity).
    let mut d4 = Driver::with_store(&root, "proj-b").unwrap();
    let r4 = d4.request(BASE, "d_main", &[]);
    assert!(r4.ok());
    assert_eq!(
        *r4.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "cross-project sharing"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// V-07: body-only edit re-keys ONLY the edited unit (callers' code units
/// stay reusable); a signature edit re-keys every caller.
#[test]
fn av_cache_dependency_invalidation() {
    let root = tmpdir("deps");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r0 = d.request(BASE, "d_main", &[]);
    assert!(r0.ok());

    // Body-only edit of helper: same signature, different body.
    let body_edit = BASE.replace("a * 10 + b", "a * 10 - b");
    let r1 = d.request(&body_edit, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(
        *r1.counters.get("cache_miss_abc").unwrap_or(&0),
        1,
        "only helper misses"
    );
    assert_eq!(
        *r1.counters.get("cache_hit_abc").unwrap_or(&0),
        1,
        "d_main unit reuses"
    );
    // And the result CHANGED (helper(3,4) = 26 now → main returns 1).
    assert_eq!(r1.value_i32(), Some(1));

    // Back to the base body: full hit again (both keys live).
    let r2 = d.request(BASE, "d_main", &[]);
    assert_eq!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0),
        2,
        "both variants hit"
    );

    // Signature edit: helper gains a param → callers' fingerprints embed
    // callee signature content → BOTH units re-key; the request itself now
    // fails binding (arity mismatch) with a frontend diagnostic.
    let sig_edit = BASE.replace(
        "fn helper(a int, b int) int {",
        "fn helper(a int, b int, c int) int {",
    );
    let r3 = d.request(&sig_edit, "d_main", &[]);
    assert!(
        r3.diag.is_some(),
        "signature conflict is a diagnosed request: {:?}",
        r3.av
    );
    assert_eq!(r3.diag.as_ref().unwrap().stage, "frontend");
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: a live-but-wrong-identity artifact (self-consistent envelope from
/// another producer/schema/owner/stage) is a CONTROLLED REJECTION at the
/// real consumption entry — never a silent hit or rebuild-over.
#[test]
fn av_cache_wrong_identity_rejected() {
    let root = tmpdir("ident");
    let store = open_store(&root).unwrap();
    // Envelope from a foreign producer over otherwise-identical payload.
    let payload = b"some-abc-bytes".to_vec();
    let foreign = EnvelopeIdentity {
        kind: cache::EnvelopeKind::AbcUnit,
        producer: "other-plan/x",
        schema: auto_av_core::contract::ABC_SCHEMA,
        owner_module: "m",
        stage: 2,
        mir_sem: 1,
        abc_sem: 1,
        av_sem: 1,
        snapshot: 0,
    };
    let key = abc_unit_key("m", "d_x", "fp", &caps_fingerprint(&[]));
    put_enveloped(&store, &key, "A09", &foreign, &payload, vec![], Some("p")).unwrap();
    // Reading with OUR identity must reject (not return the payload).
    let ours = EnvelopeIdentity::abc("m");
    let err = get_enveloped_opt(&store, &key, "A09", &ours).unwrap_err();
    assert!(
        matches!(err, cache::CacheError::Envelope(_)),
        "wrong producer rejected: {err}"
    );
    // Same for a schema/owner/stage mismatch.
    let mut stage_wrong = EnvelopeIdentity::abc("m");
    stage_wrong.stage = 1;
    let wrapped = envelope_wrap(&payload, &EnvelopeIdentity::abc("m"));
    let e2 = envelope_unwrap(&wrapped, &stage_wrong).unwrap_err();
    assert!(matches!(e2, cache::CacheError::Envelope(_)));
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: corrupted framing — a self-consistent frame over tampered inner
/// bytes is rejected by the checksum/decode chain (store admission + inner
/// envelope + codec all guard).
#[test]
fn av_cache_corrupt_frame_rejected() {
    let payload = vec![1u8, 2, 3, 4, 5, 6, 7, 8];
    let id = EnvelopeIdentity::abc("m");
    let mut env = envelope_wrap(&payload, &id);
    let mid = env.len() / 2;
    env[mid] ^= 0xff; // corrupt inner, frame checksum now stale
    let err = envelope_unwrap(&env, &id).unwrap_err();
    assert!(
        matches!(err, cache::CacheError::Envelope(_)),
        "tampered envelope: {err}"
    );
}

/// V-08: same-key different-content is a Conflict — the previously
/// committed result survives byte-identically.
#[test]
fn av_cache_conflict_preserves_committed() {
    let root = tmpdir("conflict");
    let store = open_store(&root).unwrap();
    let key = abc_unit_key("m", "d_x", "fp", &caps_fingerprint(&[]));
    let id = EnvelopeIdentity::abc("m");
    let a1 = put_enveloped(&store, &key, "A09", &id, b"content-one", vec![], Some("p")).unwrap();
    let err = put_enveloped(&store, &key, "A09", &id, b"content-two", vec![], Some("p"));
    assert!(
        matches!(
            err,
            Err(cache::CacheError::Store(
                ac_artifact_probe::store::StoreError::Conflict { .. }
            ))
        ),
        "same key different content conflicts: {:?}",
        err
    );
    // The old committed artifact still reads back intact.
    let got = get_enveloped_opt(&store, &key, "A09", &id)
        .unwrap()
        .unwrap();
    assert_eq!(got, b"content-one".to_vec());
    let _ = a1;
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08: GC + lease — a lease keeps live data pinned; roots keep soft
/// artifacts alive; stale leases sweep.
#[test]
fn av_cache_gc_and_lease() {
    let root = tmpdir("gc");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    // Grab the manifest artifact for a lease pin.
    let manifest = r.manifest.as_ref().expect("manifest written");
    let store = open_store(&root).unwrap();
    let sentinel = manifest.units[0].artifact.clone();
    let lease = store
        .acquire_lease("exec", 60, Some(&sentinel))
        .expect("lease");
    // GC under a live lease must not delete the leased artifact's payload.
    let cfg = ac_artifact_probe::store::gc::gc_config_default();
    let _receipt = ac_artifact_probe::store::gc::run_gc(&store, cfg).expect("gc");
    // The leased unit still loads through the manifest chain.
    let payload =
        cache::get_by_artifact_id(&store, &sentinel, &EnvelopeIdentity::abc(&manifest.owner));
    assert!(
        payload.is_ok(),
        "leased artifact survives GC: {:?}",
        payload.err()
    );
    lease.release();
    // Stale lease sweep is a no-op now.
    let swept = store.sweep_stale_leases().expect("sweep");
    let _ = swept;
    // After releasing + unrooting, a follow-up request still works (roots
    // hold soft references from the original project).
    let mut d2 = Driver::with_store(&root, "p").unwrap();
    let r2 = d2.request(BASE, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    let _ = std::fs::remove_dir_all(&root);
}

/// T-01/T-04: cross-process restore — a NEW process loads the manifest +
/// units from disk, re-verifies, executes. Nothing in-memory carries over.
#[test]
fn av_cache_worker_restore_new_process() {
    let root = tmpdir("worker");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    let manifest = r.manifest.as_ref().expect("manifest");
    // The manifest's own artifact id: find it via its key.
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store
        .get_artifact("A09", &key)
        .unwrap()
        .expect("manifest artifact");
    drop(store);

    let worker = env!("CARGO_BIN_EXE_av_probe_worker");
    let out = std::process::Command::new(worker)
        .args([
            "restore-run",
            "--store",
            root.to_str().unwrap(),
            "--owner",
            &manifest.owner,
            "--entry",
            &manifest.entry,
            "--manifest-artifact",
            &art.artifact_id,
            "--snapshot",
            &format!("{:016x}", r.snapshot),
        ])
        .output()
        .expect("spawn worker");
    assert!(
        out.status.success(),
        "worker restore-run failed: {}\nstdout: {}\nstderr: {}",
        out.status,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let line = String::from_utf8_lossy(&out.stdout);
    let parsed: auto_av_probe::worker_restore::RestoreOutcome =
        serde_json::from_str(line.trim()).expect("worker JSON");
    assert!(parsed.executed_ok, "{:?}", parsed.error);
    assert_eq!(parsed.value, r.value_i32(), "restored execution matches");
    assert_eq!(
        parsed.units_verified, 2,
        "both units re-verified in the new process"
    );
    let _ = fnv1a64;
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R3-03: a position-only drift (same canonical bodies, new snapshot)
/// must NOT lose the trap's source binding on a cache hit — the unit
/// re-binds to the CURRENT revision (sidecar miss triggers a bounded,
/// separately-counted diagnostics re-derive), never a spanless module.
#[test]
fn av_cache_position_drift_keeps_trap_span() {
    let root = tmpdir("span-drift");
    let mut d = Driver::with_store(&root, "proj-span").unwrap();
    let src1 = "fn main() int {
    return 1 / 0
}
";
    let r1 = d.request(src1, "d_main", &[]);
    use auto_av_core::contract::ExecOutcome;
    let span1 = match &r1.av {
        Some(ExecOutcome::Trap {
            span: Some(s),
            kind,
        }) => {
            assert_eq!(kind.code(), "av.trap.div-by-zero");
            *s
        }
        other => panic!("first request traps with a span: {other:?}"),
    };
    // Two prepended lines: same bodies, drifted positions, same entry.
    let src2 = format!(
        "

{src1}"
    );
    let r2 = d.request(&src2, "d_main", &[]);
    assert_eq!(
        *r2.counters.get("cache_hit_abc").unwrap_or(&0),
        1,
        "full hit"
    );
    assert_eq!(*r2.counters.get("mir_lower_modules").unwrap_or(&0), 0);
    assert_eq!(
        *r2.counters
            .get("sidecar_rebind_lower_modules")
            .unwrap_or(&0),
        1,
        "the span re-bind lower fired exactly once and is counted honestly"
    );
    match &r2.av {
        Some(ExecOutcome::Trap { span: Some(s2), .. }) => {
            assert_eq!(s2.func, span1.func, "same function binds");
            assert!(
                s2.start > span1.start,
                "span re-bind points at the CURRENT (drifted) position: {s2:?} vs {span1:?}"
            );
        }
        other => panic!("drifted request keeps the trap AND its span: {other:?}"),
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R3-02: restore admission is closure-identity-complete. Tampering
/// any single manifest axis — profile, av semantics, unit signature,
/// logical id, payload binding, or the sema-ref envelope itself — against
/// an otherwise VALID store is a controlled rejection; the honest control
/// still restores and executes.
#[test]
fn av_cache_restore_identity_negative_controls() {
    let root = tmpdir("restore-neg");
    let mut d = Driver::with_store(&root, "p2").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    let honest = r.manifest.clone().expect("manifest");
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&honest.owner, &honest.entry, &honest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().expect("artifact");
    drop(store);
    // Honest control first.
    let ok = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &honest.owner,
        &honest.entry,
        &art.artifact_id,
        0,
        &[],
    );
    assert!(ok.executed_ok, "honest control restores: {:?}", ok.error);

    // Write a tampered manifest into the SAME real store under its own key
    // (self-consistent envelope, honest payload pointers) and try to
    // restore THROUGH it.
    let tamper = |mutate: &dyn Fn(&mut auto_av_probe::cache::ModuleManifest)| {
        let mut m = honest.clone();
        mutate(&mut m);
        let store = open_store(&root).unwrap();
        let k = cache::manifest_key(&m.owner, &m.entry, &m.fingerprint());
        let body = serde_json::to_vec(&m).unwrap();
        put_enveloped(
            &store,
            &k,
            "A09",
            &EnvelopeIdentity::manifest(&m.owner, 0),
            &body,
            vec![],
            Some("p2"),
        )
        .expect("tampered manifest put");
        let art = store
            .get_artifact("A09", &k)
            .unwrap()
            .expect("tampered artifact");
        drop(store);
        auto_av_probe::worker_restore::restore_and_run(
            &root,
            &honest.owner,
            &honest.entry,
            &art.artifact_id,
            0,
            &[],
        )
    };
    let o = tamper(&|m| m.profile = "unknown-profile".to_string());
    assert!(!o.executed_ok, "profile tamper must reject");
    assert!(
        o.error.unwrap().contains("profile"),
        "profile axis diagnosed"
    );

    let o = tamper(&|m| m.av_sem = 999);
    assert!(!o.executed_ok, "av_sem tamper must reject");
    assert!(o.error.unwrap().contains("semantics"));

    let o = tamper(&|m| m.units[0].sig = "(bool)->bool".to_string());
    assert!(!o.executed_ok, "signature tamper must reject");
    assert!(o.error.unwrap().contains("signature"));

    let o = tamper(&|m| m.units[0].logical = "foreign::logical::id".to_string());
    assert!(!o.executed_ok, "logical tamper must reject");
    assert!(o.error.unwrap().contains("logical"));

    let o = tamper(&|m| m.units[0].payload_fp = "deadbeefdeadbeef".to_string());
    assert!(!o.executed_ok, "payload binding tamper must reject");
    assert!(o.error.unwrap().contains("payload"));

    let o = tamper(&|m| m.units[0].sema = None);
    assert!(!o.executed_ok, "missing envelope must reject");
    assert!(o.error.unwrap().contains("sema-ref"));

    let o = tamper(&|m| {
        m.units[0].sema = Some(auto_av_probe::cache::SemaRefSer {
            producer: "other/producer".to_string(),
            schema: "ac796/semaref/v1".to_string(),
            payload_id: m.units[0].payload_fp.clone(),
        })
    });
    assert!(!o.executed_ok, "foreign producer envelope must reject");
    assert!(o.error.unwrap().contains("producer"));
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R3-02: capability re-admission on restore — a trace-requiring
/// module restores ONLY under a live trace grant; persisted capability
/// claims never inherit authority.
#[test]
fn av_cache_restore_requires_current_trace_grant() {
    let root = tmpdir("restore-caps");
    let src = "fn main() int {
    let a = mark_a()
    return a
}
";
    let caps = vec!["hir.test.trace.v1".to_string()];
    let mut d = Driver::with_store(&root, "p3").unwrap();
    let r = d.request(src, "d_main", &caps);
    assert!(r.ok(), "{:?}", r.diag);
    let manifest = r.manifest.clone().expect("manifest");
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().expect("artifact");
    drop(store);

    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot,
        &[],
    );
    assert!(!denied.executed_ok, "no live grant → no restore execution");
    assert!(
        denied.error.unwrap().contains("capability"),
        "capability axis diagnosed"
    );

    let granted = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot,
        &caps,
    );
    assert!(
        granted.executed_ok,
        "live grant restores: {:?}",
        granted.error
    );
    let _ = std::fs::remove_dir_all(&root);
}
