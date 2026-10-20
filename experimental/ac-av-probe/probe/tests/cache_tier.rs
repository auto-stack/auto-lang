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

// Publish an explicitly snapshot-bound test manifest so sidecar-negative
// controls still reach coverage checks rather than fail an earlier binding.
fn publish_snapshot_manifest(
    store: &ac_artifact_probe::store::ManagedStore,
    manifest: &cache::ModuleManifest,
    snapshot: u64,
) -> String {
    let mut bound = manifest.clone();
    bound.diagnostics_snapshot = snapshot;
    cache::put_enveloped(
        store,
        &cache::manifest_key(&bound.owner, &bound.entry, &bound.fingerprint()),
        "A09",
        &EnvelopeIdentity::manifest(&bound.owner, 0),
        &serde_json::to_vec(&bound).unwrap(),
        vec![],
        Some("p"),
    )
    .unwrap()
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
        unit: "",
        body_fp: "",
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
    // The leased unit still loads through the manifest chain (bound to
    // the manifest link's unit/body-fingerprint identity).
    let leased_link = &manifest.units[0];
    let payload = cache::get_by_artifact_id(
        &store,
        &sentinel,
        &EnvelopeIdentity::abc_bound(&manifest.owner, &leased_link.name, &leased_link.body_fp),
    );
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
        other => panic!(
            "drifted request keeps the trap AND its span: {other:?} diag={:?}",
            r2.diag
        ),
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
    // Honest control first (the REQUEST's snapshot: sidecars are required
    // for the current snapshot since F798-R6-03).
    let ok = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &honest.owner,
        &honest.entry,
        &art.artifact_id,
        r.snapshot,
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
            r.snapshot,
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

/// F798-R6-01 replay: a VALID payload stored under ANOTHER body's key is
/// a controlled identity rejection, never a wrong-body hit — on BOTH the
/// A09 (ABC) and A08 (MIR) layers.
#[test]
fn av_cache_valid_payload_under_wrong_body_key_rejected() {
    let root = tmpdir("wrong-key");
    let src7 = "fn main() int { return 7 }";
    let src9 = "fn main() int { return 9 }";
    let c7 = auto_ac_prototype::source::compile_scalar_source(src7, "probe.at", &[]).unwrap();
    let c9 = auto_ac_prototype::source::compile_scalar_source(src9, "probe.at", &[]).unwrap();
    let h9 = &c9.checked.bundle().modules[0];
    let owner = h9.id_text.clone();
    let (name, body) = h9
        .defs
        .iter()
        .find_map(|d| match &d.kind {
            auto_ac_prototype::hir::DefKind::Function { body, .. } => {
                Some((d.id_text.clone(), *body))
            }
            _ => None,
        })
        .unwrap();
    let dump = cache::canonical_hir_function(h9, body);
    let fp9 = auto_av_probe::semrefs::content_version_of(&dump);
    let caps_fp = cache::caps_fingerprint(&[]);

    // The `return 7` unit, stored under the ABC key the `return 9` body
    // requires (legacy UNBOUND envelope — exactly the review's shape).
    let m7 = auto_av_probe::hir_adapter::lower_checked(&c7.checked, "d_main", 0).unwrap();
    let mut u7 = cache::split_units(&m7).unwrap().remove(0);
    u7.funcs[0].spans.clear();
    u7.funcs[0].source_snapshot = 0;
    {
        let store = open_store(&root).unwrap();
        let bad_key = cache::abc_unit_key(&owner, &name, &fp9, &caps_fp);
        cache::put_enveloped(
            &store,
            &bad_key,
            "A09",
            &EnvelopeIdentity::abc(&owner),
            &auto_av_core::encode_abc(&u7).unwrap().to_flat(),
            vec![],
            Some("wrong-key"),
        )
        .unwrap();
        // And the A08 layer under the wrong body's MIR key.
        let bad_mir = cache::mir_unit_key(&owner, &name, &fp9, &caps_fp);
        cache::put_enveloped(
            &store,
            &bad_mir,
            "A08",
            &EnvelopeIdentity::mir(&owner, 0),
            &auto_av_core::encode_abc(&u7).unwrap().to_flat(),
            vec![],
            Some("wrong-key"),
        )
        .unwrap();
    }
    let mut d = Driver::with_store(&root, "wrong-key").unwrap();
    let r = d.request(src9, "d_main", &[]);
    let diag = r
        .diag
        .clone()
        .expect("wrong-body key must be a controlled rejection");
    assert_eq!(diag.stage, "cache");
    assert_eq!(diag.code, "cache.identity");
    assert!(
        diag.message.contains("unit binding"),
        "the unit/body-fingerprint binding fires: {}",
        diag.message
    );
    assert_eq!(
        *r.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "never counted as a hit"
    );
    assert_eq!(r.value_i32(), None, "no wrong value published");
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R6-01 replay: a manifest link renamed onto another declaration —
/// even with a re-derived self-consistent logical id — is rejected at the
/// bound envelope / decoded-name admission.
#[test]
fn av_cache_restore_renamed_declaration_rejected() {
    let root = tmpdir("renamed");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(BASE, "d_main", &[]);
    assert!(r.ok());
    let honest = r.manifest.clone().unwrap();
    let store = open_store(&root).unwrap();
    let mut forged = honest.clone();
    for link in &mut forged.units {
        link.name = "totally-different-declaration".into();
        link.logical = auto_av_probe::semrefs::unit_logical_id(
            &auto_av_probe::semrefs::av_package(),
            &honest.owner,
            &link.name,
        );
    }
    let k = cache::manifest_key(&forged.owner, &forged.entry, &forged.fingerprint());
    let id = cache::put_enveloped(
        &store,
        &k,
        "A09",
        &EnvelopeIdentity::manifest(&forged.owner, 0),
        &serde_json::to_vec(&forged).unwrap(),
        vec![],
        Some("p"),
    )
    .unwrap();
    drop(store);
    let out = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &honest.owner,
        &honest.entry,
        &id,
        r.snapshot,
        &[],
    );
    assert!(
        !out.executed_ok,
        "renamed link never restores: {:?}",
        out.error
    );
    let err = out.error.unwrap();
    assert!(
        err.contains("unit binding") || err.contains("declaration/payload mismatch"),
        "name/payload binding diagnosed: {}",
        err
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R6-03 replay: restore under a snapshot with NO sidecar is a
/// controlled unavailable rejection — never a source-less success.
#[test]
fn av_cache_restore_missing_sidecar_refused() {
    let root = tmpdir("noside");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request("fn main() int { return 1 / 0 }", "d_main", &[]);
    assert!(!r.ok() || r.value_i32().is_none(), "trap source");
    let manifest = r.manifest.clone().unwrap();
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().unwrap();
    // Current snapshot restores WITH its span-bound trap.
    let ok = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot,
        &[],
    );
    assert!(ok.executed_ok, "honest snapshot restores: {:?}", ok.error);
    assert!(
        ok.trap.as_deref().unwrap_or("").contains('@'),
        "trap carries its source span: {:?}",
        ok.trap
    );
    // Wrong snapshot is refused even if it names a valid sidecar elsewhere.
    let wrong = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot.wrapping_add(1),
        &[],
    );
    assert!(wrong.error.unwrap().contains("manifest snapshot mismatch"));
    // An explicitly bound snapshot with no sidecar reaches the missing-map gate.
    let missing_art = publish_snapshot_manifest(&store, &manifest, r.snapshot.wrapping_add(1));
    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &missing_art,
        r.snapshot.wrapping_add(1),
        &[],
    );
    assert!(!denied.executed_ok, "missing sidecar refuses");
    assert!(
        denied.error.unwrap().contains("no source sidecar"),
        "unavailable diagnosed"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R7-03: a DECODABLE but EMPTY (or non-covering) sidecar loses the
/// trap's source exactly like a missing one — restore refuses unless the
/// map covers every trap-capable instruction; the honest control's trap
/// carries its real span content.
#[test]
fn av_cache_restore_empty_or_partial_sidecar_refused() {
    let root = tmpdir("empty-side");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request("fn main() int { return 1 / 0 }", "d_main", &[]);
    let manifest = r.manifest.clone().unwrap();
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().unwrap();

    // Honest control: the trap carries its REAL span (not just a '@').
    let ok = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot,
        &[],
    );
    assert!(ok.executed_ok, "honest control: {:?}", ok.error);
    assert_eq!(
        ok.trap.unwrap(),
        "av.trap.div-by-zero@func#0:23..28",
        "real span content, not a source-less trap string"
    );

    // Empty-but-decodable sidecar under a fresh snapshot: refused.
    let unit = &manifest.units[0];
    let snap2 = r.snapshot ^ 1;
    let skey = cache::sidecar_key(&manifest.owner, &unit.name, snap2);
    cache::put_enveloped(
        &store,
        &skey,
        "A08",
        &EnvelopeIdentity::sidecar(&manifest.owner, snap2),
        &cache::encode_sidecar(&[]),
        vec![],
        Some("p"),
    )
    .unwrap();
    let bound_art = publish_snapshot_manifest(&store, &manifest, snap2);
    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &bound_art,
        snap2,
        &[],
    );
    assert!(!denied.executed_ok, "empty sidecar refuses");
    assert!(
        denied
            .error
            .unwrap()
            .contains("does not cover trap-capable instruction"),
        "coverage diagnosed"
    );

    // Partial sidecar: covers a bogus index but not the div — refused.
    let snap3 = r.snapshot ^ 2;
    let skey3 = cache::sidecar_key(&manifest.owner, &unit.name, snap3);
    let bogus = vec![(
        99u32,
        auto_av_core::contract::SpanInfo {
            func: 0,
            start: 1,
            end: 2,
        },
    )];
    cache::put_enveloped(
        &store,
        &skey3,
        "A08",
        &EnvelopeIdentity::sidecar(&manifest.owner, snap3),
        &cache::encode_sidecar(&bogus),
        vec![],
        Some("p"),
    )
    .unwrap();
    let bound_art = publish_snapshot_manifest(&store, &manifest, snap3);
    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &bound_art,
        snap3,
        &[],
    );
    assert!(!denied.executed_ok, "partial sidecar refuses");
    assert!(
        denied.error.unwrap().contains("does not cover"),
        "coverage diagnosed"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R8-02: the unary MIN negation traps (IntOverflow) — a restored
/// sidecar must cover it exactly like a Bin. Honest snapshot carries the
/// real span; an empty sidecar under the unary-only source is refused.
#[test]
fn av_cache_restore_unary_trap_requires_span() {
    let src = "fn main() int { var x = -2147483648\n    return -x\n}\n";
    let root = tmpdir("unary-trap");
    let mut d = Driver::with_store(&root, "p").unwrap();
    let r = d.request(src, "d_main", &[]);
    use auto_av_core::contract::ExecOutcome;
    assert!(
        matches!(&r.av, Some(ExecOutcome::Trap { kind, .. }) if kind.code() == "av.trap.int-overflow"),
        "the unary negation traps: {:?}",
        r.av
    );
    let manifest = r.manifest.clone().unwrap();
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let art = store.get_artifact("A09", &key).unwrap().unwrap();

    // Honest control: the unary overflow trap carries its real span.
    let ok = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &art.artifact_id,
        r.snapshot,
        &[],
    );
    assert!(ok.executed_ok, "honest control: {:?}", ok.error);
    assert_eq!(
        ok.trap.unwrap(),
        "av.trap.int-overflow@func#0:48..49",
        "real unary-negation span content"
    );

    // Empty (decodable) sidecar under a fresh snapshot: refused — the
    // unary-only source has NO Bin, so only the shared can_trap set
    // catches it.
    let snap2 = r.snapshot ^ 1;
    let unit = &manifest.units[0];
    let skey = cache::sidecar_key(&manifest.owner, &unit.name, snap2);
    cache::put_enveloped(
        &store,
        &skey,
        "A08",
        &EnvelopeIdentity::sidecar(&manifest.owner, snap2),
        &cache::encode_sidecar(&[]),
        vec![],
        Some("p"),
    )
    .unwrap();
    let bound_art = publish_snapshot_manifest(&store, &manifest, snap2);
    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &bound_art,
        snap2,
        &[],
    );
    assert!(
        !denied.executed_ok,
        "empty sidecar over a unary trap refuses"
    );
    assert!(
        denied
            .error
            .unwrap()
            .contains("does not cover trap-capable instruction"),
        "the NegI32 coverage fires"
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R12-03 (K14-06): kind-less /1 wire artifacts are a DIFFERENT format
/// generation. Raw /1 bytes face a controlled unsupported-version
/// rejection; a stale payload seeded under a CURRENT key is a deterministic
/// miss-rebuild — never read in place as new-semantics bytes, never a hard
/// error; the stale row survives (store conflict semantics) while the
/// request serves the freshly re-verified rebuild.
#[test]
fn av_cache_kindless_v1_wire_isolated() {
    let src = "fn main() int { return 7 }";
    let c = auto_ac_prototype::source::compile_scalar_source(src, "legacy.at", &[]).unwrap();
    let m = auto_av_probe::hir_adapter::lower_checked(&c.checked, "d_main", 0).unwrap();
    // Forge a genuine /1 artifact: current encode, version field back to 1,
    // kind bit cleared, FNV trailer recomputed — the exact byte shape the
    // pre-R12 writer persisted for a unit.
    let mut unit = cache::split_units(&m).unwrap().remove(0);
    unit.funcs[0].spans.clear();
    unit.funcs[0].source_snapshot = 0;
    let mut old = auto_av_core::encode_abc(&unit).unwrap().to_flat();
    old[4..6].copy_from_slice(&1u16.to_le_bytes());
    let flags = u16::from_le_bytes([old[8], old[9]]) & !auto_av_core::abc::FLAG_IS_UNIT;
    old[8..10].copy_from_slice(&flags.to_le_bytes());
    let end = old.len() - 8;
    let sum = fnv1a64(&old[..end]);
    old[end..].copy_from_slice(&sum.to_le_bytes());

    // Raw: unsupported-version rejection (controlled, never a panic).
    let err = auto_av_core::decode_abc(&old).unwrap_err();
    assert_eq!(err.code(), "abc.decode.version");

    // Managed: seed the stale payload under the CURRENT key + envelope (the
    // exact ambiguity K14-06 forbids reading in place) and drive requests.
    let root = tmpdir("legacy-kind");
    let h = &c.checked.bundle().modules[0];
    let owner = h.id_text.clone();
    let (name, body) = h
        .defs
        .iter()
        .find_map(|d| match &d.kind {
            auto_ac_prototype::hir::DefKind::Function { body, .. } => {
                Some((d.id_text.clone(), *body))
            }
            _ => None,
        })
        .unwrap();
    let fp = auto_av_probe::semrefs::content_version_of(&cache::canonical_hir_function(h, body));
    let key = abc_unit_key(&owner, &name, &fp, &caps_fingerprint(&[]));
    let store = open_store(&root).unwrap();
    put_enveloped(
        &store,
        &key,
        "A09",
        &EnvelopeIdentity::abc_bound(&owner, &name, &fp),
        &old,
        vec![],
        Some("legacy"),
    )
    .unwrap();
    drop(store);

    let mut d = Driver::with_store(&root, "legacy").unwrap();
    let r = d.request(src, "d_main", &[]);
    assert!(r.ok(), "stale payload rebuilds and serves: {:?}", r.diag);
    assert_eq!(r.value_i32(), Some(7));
    assert_eq!(
        *r.counters.get("cache_hit_abc").unwrap_or(&0),
        0,
        "the stale row is never a hit"
    );
    assert_eq!(*r.counters.get("cache_stale_abc_miss").unwrap_or(&0), 1);
    assert_eq!(*r.counters.get("abc_units_emitted").unwrap_or(&0), 1);
    assert_eq!(
        *r.counters.get("cache_stale_rebuild_conflict").unwrap_or(&0),
        1,
        "re-persisting over the stale row conflicts and is tolerated"
    );
    // Deterministic replay: the stale row survives (never overwritten in
    // place), so a second request stale-misses and rebuilds identically.
    let r2 = d.request(src, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    assert_eq!(r2.value_i32(), Some(7));
    assert_eq!(*r2.counters.get("cache_hit_abc").unwrap_or(&0), 0);
    assert_eq!(*r2.counters.get("cache_stale_abc_miss").unwrap_or(&0), 1);
    drop(d);
    let _ = std::fs::remove_dir_all(&root);
}

/// F798-R15-01..03: cache-consumption invariants at the ordinary Driver
/// entrance — grant re-admission vs the CURRENT request, unit-envelope
/// category conflict, and sidecar APPLICABILITY (an empty map re-derives
/// from the current source; the trap keeps its span).
#[test]
fn av_cache_r15_consumption_invariants() {
    // F798-R15-01: the REQUEST source is trace-free (the frontend cannot
    // reject it); the cached payload is a hand-built, semantically legal
    // unit that DECLARES the trace capability and contains MarkA. A hit
    // under caps=[] must refuse (cache.grant); under the grant it
    // executes with the trace event.
    let plain_src = "fn main() int { return 7 }";
    let trace_caps = vec!["hir.test.trace.v1".to_string()];
    let root = tmpdir("r15-invariants");
    {
        let plain =
            auto_ac_prototype::source::compile_scalar_source(plain_src, "p.at", &[]).unwrap();
        let h = &plain.checked.bundle().modules[0];
        let (name, body) = h
            .defs
            .iter()
            .find_map(|d| match &d.kind {
                auto_ac_prototype::hir::DefKind::Function { body, .. } => {
                    Some((d.id_text.clone(), *body))
                }
                _ => None,
            })
            .unwrap();
        let owner = h.id_text.clone();
        let fp =
            auto_av_probe::semrefs::content_version_of(&cache::canonical_hir_function(h, body));
        let tc = auto_ac_prototype::source::compile_scalar_source(
            "fn main() int {
    var t = mark_a()
    return 7
}
",
            "t.at",
            &trace_caps,
        )
        .unwrap();
        let m = auto_av_probe::hir_adapter::lower_checked(&tc.checked, "d_main", 0).unwrap();
        let mut u = cache::split_units(&m).unwrap().remove(0);
        u.funcs[0].spans.clear();
        u.funcs[0].source_snapshot = 0;
        let bytes = auto_av_core::encode_abc(&u).unwrap().to_flat();
        let store = open_store(&root).unwrap();
        let key = abc_unit_key(&owner, &name, &fp, &caps_fingerprint(&[]));
        let expect = EnvelopeIdentity::abc_bound(&owner, &name, &fp);
        put_enveloped(&store, &key, "A09", &expect, &bytes, vec![], Some("p")).unwrap();
        drop(store);
        let mut d = Driver::with_store(&root, "p").unwrap();
        let r = d.request(plain_src, "d_main", &[]);
        // The hit WAS taken (the lookup axis is the request's own caps)
        // and the payload's required grant is re-admitted at consumption.
        assert_eq!(*r.counters.get("cache_hit_abc").unwrap_or(&0), 1);
        let diag = r.diag.expect("refused without the current grant");
        assert_eq!(diag.code, "cache.grant");
        // Positive control (the R15 probe shape): the SAME payload seeded
        // under the GRANTED key executes WITH the trace event.
        {
            let store = open_store(&root).unwrap();
            let key = abc_unit_key(&owner, &name, &fp, &caps_fingerprint(&trace_caps));
            let expect = EnvelopeIdentity::abc_bound(&owner, &name, &fp);
            put_enveloped(&store, &key, "A09", &expect, &bytes, vec![], Some("p")).unwrap();
            drop(store);
        }
        let r = d.request(plain_src, "d_main", &trace_caps);
        assert!(r.ok(), "{:?}", r.diag);
        assert_eq!(*r.counters.get("cache_hit_abc").unwrap_or(&0), 1);
        assert!(r
            .trace
            .iter()
            .any(|e| matches!(e, auto_av_core::mir::TraceEvent::MarkA)));
    }
    let _ = std::fs::remove_dir_all(&root);

    // F798-R15-02: a complete-MODULE payload under a UNIT envelope/key is
    // a category conflict on both layers.
    let (owner, name, fp) = {
        let c = auto_ac_prototype::source::compile_scalar_source(plain_src, "t.at", &trace_caps)
            .unwrap();
        let h = &c.checked.bundle().modules[0];
        let (name, body) = h
            .defs
            .iter()
            .find_map(|d| match &d.kind {
                auto_ac_prototype::hir::DefKind::Function { body, .. } => {
                    Some((d.id_text.clone(), *body))
                }
                _ => None,
            })
            .unwrap();
        (
            h.id_text.clone(),
            name,
            auto_av_probe::semrefs::content_version_of(&cache::canonical_hir_function(h, body)),
        )
    };
    for layer in ["A09", "A08"] {
        let r2 = tmpdir(&format!("r15-kind-{layer}"));
        let c = auto_ac_prototype::source::compile_scalar_source(plain_src, "t.at", &trace_caps)
            .unwrap();
        let m = auto_av_probe::hir_adapter::lower_checked(&c.checked, "d_main", 0).unwrap();
        let mut u = cache::split_units(&m).unwrap().remove(0);
        u.funcs[0].spans.clear();
        u.funcs[0].source_snapshot = 0;
        u.is_unit = false;
        let bytes = auto_av_core::encode_abc(&u).unwrap().to_flat();
        let store = open_store(&r2).unwrap();
        let (key, expect) = if layer == "A09" {
            (
                abc_unit_key(&owner, &name, &fp, &caps_fingerprint(&trace_caps)),
                EnvelopeIdentity::abc_bound(&owner, &name, &fp),
            )
        } else {
            (
                cache::mir_unit_key(&owner, &name, &fp, &caps_fingerprint(&trace_caps)),
                EnvelopeIdentity::mir_bound(&owner, &name, &fp),
            )
        };
        put_enveloped(&store, &key, layer, &expect, &bytes, vec![], Some("p")).unwrap();
        drop(store);
        let mut d = Driver::with_store(&r2, "p").unwrap();
        let r = d.request(plain_src, "d_main", &trace_caps);
        let diag = r.diag.expect("category conflict refused");
        assert_eq!(diag.code, "cache.identity", "{layer}");
        assert!(diag.message.contains("complete-module payload"), "{layer}");
        let _ = std::fs::remove_dir_all(&r2);
    }

    // F798-R15-03: a hand-seeded hit + EMPTY sidecar — the driver
    // re-derives from the current source; the trap keeps its span.
    {
        let r3 = tmpdir("r15-sidecar");
        let div_src = "fn main() int {
    return 1 / 0
}
";
        let c = auto_ac_prototype::source::compile_scalar_source(div_src, "d.at", &[]).unwrap();
        let h = &c.checked.bundle().modules[0];
        let (nm, body) = h
            .defs
            .iter()
            .find_map(|d| match &d.kind {
                auto_ac_prototype::hir::DefKind::Function { body, .. } => {
                    Some((d.id_text.clone(), *body))
                }
                _ => None,
            })
            .unwrap();
        let owner2 = h.id_text.clone();
        let fp2 =
            auto_av_probe::semrefs::content_version_of(&cache::canonical_hir_function(h, body));
        let m = auto_av_probe::hir_adapter::lower_checked(&c.checked, "d_main", 0).unwrap();
        let mut u = cache::split_units(&m).unwrap().remove(0);
        u.funcs[0].spans.clear();
        u.funcs[0].source_snapshot = 0;
        let bytes = auto_av_core::encode_abc(&u).unwrap().to_flat();
        let store = open_store(&r3).unwrap();
        let key = abc_unit_key(&owner2, &nm, &fp2, &caps_fingerprint(&[]));
        let expect = EnvelopeIdentity::abc_bound(&owner2, &nm, &fp2);
        put_enveloped(&store, &key, "A09", &expect, &bytes, vec![], Some("p")).unwrap();
        let skey = cache::sidecar_key(&owner2, &nm, 0);
        let sexpect = EnvelopeIdentity::sidecar(&owner2, 0);
        put_enveloped(
            &store,
            &skey,
            "A08",
            &sexpect,
            &cache::encode_sidecar(&[]),
            vec![],
            Some("p"),
        )
        .unwrap();
        drop(store);
        let mut d = Driver::with_store(&r3, "p").unwrap();
        let r = d.request(div_src, "d_main", &[]);
        assert!(r.diag.is_none(), "empty sidecar re-derives: {:#?}", r.diag);
        assert_eq!(
            *r.counters.get("sidecar_rebind_lower_modules").unwrap_or(&0),
            1
        );
        let trap = format!("{:?}", r.av);
        assert!(trap.contains("DivByZero"), "{trap}");
        assert!(
            trap.contains("start:"),
            "trap carries its source span: {trap}"
        );
        let _ = std::fs::remove_dir_all(&r3);
    }
}

/// Each consumed sibling is pinned; a first-unit lease cannot reach other
/// units through reverse manifest edges. Manifest restore must pin its own
/// required-data closure, and every success/failure return releases locks.
#[test]
fn av_cache_consumption_leases_cover_siblings_and_release() {
    let root = tmpdir("consume-leases");
    let mut driver = Driver::with_store(&root, "lease-project").unwrap();
    assert!(driver.request(BASE, "d_main", &[]).ok());
    let store = open_store(&root).unwrap();
    let hot = driver.request(BASE, "d_main", &[]);
    assert!(hot.ok(), "{:?}", hot.diag);
    let manifest = hot.manifest.unwrap();
    let pins: Vec<String> = store
        .db()
        .prepare("SELECT artifact FROM leases WHERE purpose='plan798-request-consume'")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    assert_eq!(pins.len(), manifest.units.len());
    for unit in &manifest.units {
        assert!(pins.contains(&unit.artifact));
    }
    let key = cache::manifest_key(&manifest.owner, &manifest.entry, &manifest.fingerprint());
    let artifact = store
        .get_artifact("A09", &key)
        .unwrap()
        .unwrap()
        .artifact_id;
    let restored = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &manifest.owner,
        &manifest.entry,
        &artifact,
        hot.snapshot,
        &[],
    );
    assert!(restored.executed_ok, "{:?}", restored.error);
    let count: i64 = store
        .db()
        .query_row(
            "SELECT COUNT(*) FROM leases WHERE purpose='plan798-restore-manifest' AND artifact=?1",
            [&artifact],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1, "real restore entrance pins the manifest");
    let refused = auto_av_probe::worker_restore::restore_and_run(
        &root,
        "wrong-owner",
        &manifest.entry,
        &artifact,
        hot.snapshot,
        &[],
    );
    assert!(!refused.executed_ok);
    // Removing ABC rows drives the A08 restore path; its pins must cover both
    // consumed MIR artifacts too, not only the ordinary A09 path.
    store.purge_layer("A09").unwrap();
    let mir = driver.request(BASE, "d_main", &[]);
    assert!(mir.ok(), "{:?}", mir.diag);
    let count: i64 = store
        .db()
        .query_row(
            "SELECT COUNT(DISTINCT artifact) FROM leases WHERE purpose='plan798-request-mir'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count as usize, manifest.units.len());
    let ids: Vec<String> = store
        .db()
        .prepare("SELECT lease_id FROM leases")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    for id in ids {
        assert!(!store.lease_live(&id).unwrap(), "leaked {id}");
    }
    drop(driver);
    drop(store);
    let _ = std::fs::remove_dir_all(root);
}

/// Position edits share function bytes but have distinct diagnostic closures.
/// A real manifest lease, without project roots, must retain the NEW sidecar.
#[test]
fn av_cache_snapshot_manifest_lease_survives_position_edit_gc() {
    let root = tmpdir("snapshot-manifest-gc");
    let mut driver = Driver::with_store(&root, "snapshot-project").unwrap();
    let first = driver.request("fn main() int { return 1 / 0 }", "d_main", &[]);
    let next = driver.request("\n\nfn main() int { return 1 / 0 }", "d_main", &[]);
    let old = first.manifest.unwrap();
    let current = next.manifest.unwrap();
    assert_eq!(
        old.units[0].artifact, current.units[0].artifact,
        "unit code unchanged"
    );
    assert_eq!(next.counters.get("cache_hit_abc"), Some(&1));
    assert_eq!(next.counters.get("abc_units_emitted"), Some(&0));
    assert_ne!(old.fingerprint(), current.fingerprint());
    let store = open_store(&root).unwrap();
    let key = cache::manifest_key(&current.owner, &current.entry, &current.fingerprint());
    let artifact = store
        .get_artifact("A09", &key)
        .unwrap()
        .unwrap()
        .artifact_id;
    let pin = store
        .acquire_lease("snapshot-test", 60, Some(&artifact))
        .unwrap();
    store.remove_project_roots("snapshot-project").unwrap();
    let gc = ac_artifact_probe::store::gc::run_gc(
        &store,
        ac_artifact_probe::store::gc::GcConfig { high: 0, low: 0 },
    )
    .unwrap();
    assert!(
        gc.deleted_chunks > 0,
        "old unpinned metadata actually collected"
    );
    let restored = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &current.owner,
        &current.entry,
        &artifact,
        next.snapshot,
        &[],
    );
    assert!(restored.executed_ok, "{:?}", restored.error);
    assert_eq!(
        restored.trap.as_deref(),
        Some("av.trap.div-by-zero@func#0:25..30")
    );
    let denied = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &current.owner,
        &current.entry,
        &artifact,
        first.snapshot,
        &[],
    );
    assert!(denied.error.unwrap().contains("manifest snapshot mismatch"));
    pin.release();
    ac_artifact_probe::store::gc::run_gc(
        &store,
        ac_artifact_probe::store::gc::GcConfig { high: 0, low: 0 },
    )
    .unwrap();
    assert!(
        store.get_artifact("A09", &key).unwrap().is_none(),
        "released closure collectible"
    );
    drop(driver);
    drop(store);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn av_cache_manifest_generation_and_snapshot_are_required() {
    let root = tmpdir("manifest-generation");
    let mut driver = Driver::with_store(&root, "manifest-project").unwrap();
    let request = driver.request("fn main() int { return 7 }", "d_main", &[]);
    let manifest = request.manifest.unwrap();
    let store = open_store(&root).unwrap();
    for (tag, field) in [
        ("legacy-format", "format"),
        ("legacy-snapshot", "diagnostics_snapshot"),
    ] {
        let mut old = serde_json::to_value(&manifest).unwrap();
        old.as_object_mut().unwrap().remove(field);
        let artifact = cache::put_enveloped(
            &store,
            &cache::manifest_key(&manifest.owner, &manifest.entry, tag),
            "A09",
            &EnvelopeIdentity::manifest(&manifest.owner, 0),
            &serde_json::to_vec(&old).unwrap(),
            vec![],
            Some("manifest-project"),
        )
        .unwrap();
        let refused = auto_av_probe::worker_restore::restore_and_run(
            &root,
            &manifest.owner,
            &manifest.entry,
            &artifact,
            request.snapshot,
            &[],
        );
        assert!(!refused.executed_ok);
        assert!(refused.error.unwrap().contains("manifest decode"));
    }
    let mut future = manifest.clone();
    future.format = 3;
    let artifact = cache::put_enveloped(
        &store,
        &cache::manifest_key(&future.owner, &future.entry, &future.fingerprint()),
        "A09",
        &EnvelopeIdentity::manifest(&future.owner, 0),
        &serde_json::to_vec(&future).unwrap(),
        vec![],
        Some("manifest-project"),
    )
    .unwrap();
    let refused = auto_av_probe::worker_restore::restore_and_run(
        &root,
        &future.owner,
        &future.entry,
        &artifact,
        request.snapshot,
        &[],
    );
    assert!(refused
        .error
        .unwrap()
        .contains("unsupported manifest format"));
    drop(driver);
    drop(store);
    let _ = std::fs::remove_dir_all(root);
}
