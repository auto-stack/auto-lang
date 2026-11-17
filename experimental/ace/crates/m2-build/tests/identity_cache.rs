//! `ace.m2.p978.identity-cache` — V-01..V-04 / V-ID-BUILD / V-10
//! (PLAN-978 T-02/T-10): precise identity axes, per-function semantic
//! invalidation, corrupted-cache refusal with old-state preservation,
//! and the per-build latency bound.

mod common;

use ace_m2_build::driver::{M2Build, ProjectInputs};
use ace_m2_build::identity::{ProjectBuildIdentity, TargetKind};
use common::scratch;

fn two_fn_source(main_body: &str) -> String {
    format!("fn helper() int {{ return 7 }}\nfn main() int {{ return {main_body} }}")
}

/// V-ID-BUILD leg 1: the identity key separates every REAL input axis —
/// source content, caps, entry — and tool/provider axes ride the key
/// only through their own fields (an ordinary Auto edit never touches a
/// provider build identity).
#[test]
fn p978_identity_axes_separate_real_inputs() {
    let base = ProjectInputs::new("p", "main", "main.at", "fn main() int { return 1 }");
    let k0 = ProjectBuildIdentity::of(&base, TargetKind::Av, "g/1", vec![]).key();
    // 1) source content axis
    let mut edited = base.clone();
    edited.sources = vec![("main.at".into(), "fn main() int { return 2 }".into())];
    let k1 = ProjectBuildIdentity::of(&edited, TargetKind::Av, "g/1", vec![]).key();
    assert_ne!(k0, k1, "a source edit must flip the identity");
    // 2) caps axis
    let mut capped = base.clone();
    capped.caps = vec!["io".into()];
    let k2 = ProjectBuildIdentity::of(&capped, TargetKind::Av, "g/1", vec![]).key();
    assert_ne!(k0, k2);
    // 3) entry axis
    let mut re_entry = base.clone();
    re_entry.entry = "other".into();
    assert_ne!(
        k0,
        ProjectBuildIdentity::of(&re_entry, TargetKind::Av, "g/1", vec![]).key()
    );
    // 4) grammar/toolchain axis (a REAL lane change)
    assert_ne!(
        k0,
        ProjectBuildIdentity::of(&base, TargetKind::Av, "g/2", vec![]).key()
    );
    // 5) tool axis (generator identity)
    assert_ne!(
        k0,
        ProjectBuildIdentity::of(
            &base,
            TargetKind::Av,
            "g/1",
            vec![("generator".into(), "ac-build/9.9".into())]
        )
        .key()
    );
    // 6) target axis: same project, different target = different node
    assert_ne!(
        k0,
        ProjectBuildIdentity::of(&base, TargetKind::Native, "g/1", vec![]).key()
    );
    // 7) stdlib closure axis
    let mut with_stdlib = base.clone();
    with_stdlib.stdlib = vec![("stdlib/auto/x.at".into(), "fn f() int { return 1 }".into())];
    assert_ne!(
        k0,
        ProjectBuildIdentity::of(&with_stdlib, TargetKind::Av, "g/1", vec![]).key()
    );
    // 8) determinism: same real inputs, same key (same-content
    // different-root stability, V-08 leg 1).
    assert_eq!(
        k0,
        ProjectBuildIdentity::of(&base, TargetKind::Av, "g/1", vec![]).key()
    );
}

/// V-02/V-10 leg: per-function precise invalidation on the a2r lane — a
/// `main` body edit reproduces exactly ONE fragment; the untouched
/// `helper` rides the semantic/fragment store hits.
#[test]
fn p978_a2r_edit_invalidates_only_changed_function() {
    let root = scratch("a2r-precise");
    let mut driver = M2Build::new(&root);
    let s1 = ProjectInputs::new("twofn", "main", "main.at", &two_fn_source("helper() * 4"));
    let s2 = ProjectInputs::new("twofn", "main", "main.at", &two_fn_source("helper() * 5"));
    let first = driver.build(TargetKind::A2r, &s1).unwrap();
    assert_eq!(first.counts.fragments_produced, 2, "cold: both functions");
    let second = driver.build(TargetKind::A2r, &s2).unwrap();
    assert!(
        second.rebuilt,
        "identity flipped (main body edit) — node rebuilds"
    );
    assert_eq!(
        second.counts.fragments_produced, 1,
        "only the edited function is reproduced (got counts {:?})",
        second.counts
    );
    assert!(
        second.counts.store_hit_semantic + second.counts.store_hit_fragment >= 1,
        "the unchanged function rides store hits (got {:?})",
        second.counts
    );
}

/// V-03 leg 1: a receipt row whose schema drifted (a corrupted or
/// foreign payload landed under the exact identity key) is REFUSED —
/// never served — and the build re-produces; the OLD valid receipt
/// still hits afterwards (failure keeps the prior good state).
#[test]
fn p978_corrupted_receipt_row_refused_and_rebuilt() {
    let root = scratch("bad-cache");
    let mut driver = M2Build::new(&root);
    let inputs = common::b01_inputs();
    let first = driver.build(TargetKind::Av, &inputs).unwrap();
    assert!(first.rebuilt);

    // Poison the NEXT identity's slot with a foreign-schema receipt row
    // written through the same store seam.
    let next = common::b01_edited_inputs();
    let identity = ProjectBuildIdentity::of(
        &next,
        TargetKind::Av,
        "av.core-i32/1 abc/2",
        vec![("generator".into(), "ac-build/0.1.0".into())],
    );
    let store = ace_artifacts::store::ManagedStore::open(&root.join("store").join("m2b"))
        .expect("open m2b");
    let poison = ace_m2_build::identity::BuildReceipt {
        schema: "bogus/schema-9".into(),
        identity: identity.clone(),
        products: vec![],
    };
    store
        .put_artifact(ace_artifacts::store::PutRequest {
            layer: ace_m2_build::identity::M978B_LAYER,
            key: ace_artifacts::identity::QueryKey {
                layer: ace_m2_build::identity::M978B_LAYER,
                components: vec![("identity".into(), identity.key())],
            },
            kind: ace_artifacts::codec::ChunkKind::LayoutData,
            payload: serde_json::to_vec(&poison).unwrap(),
            edges: vec![],
            project: Some("b01".into()),
        })
        .expect("poison row lands (the store accepts the row; the DRIVER refuses it)");

    // The poisoned row is REFUSED with a named rejection (V-03 leg 1:
    // a bad cache never serves); the OLD valid identity still hits.
    let refused = M2Build::new(&root).build(TargetKind::Av, &next);
    let err = refused.expect_err("a corrupted receipt row must be refused");
    assert_eq!(err.code, "m2build.store.poisoned-row");
    let old = M2Build::new(&root).build(TargetKind::Av, &inputs).unwrap();
    assert!(!old.rebuilt, "the old valid receipt still hits");

    // V-03 leg 2: RESTORE purges the regenerable cache layer and the
    // next build re-produces and serves the new value (recovery reads
    // back repaired).
    let report = M2Build::new(&root).restore("b01").unwrap();
    assert!(
        report.iter().any(|l| l.contains("m978b-rows-purged")),
        "restore reports the cache purge: {report:?}"
    );
    let built = M2Build::new(&root).build(TargetKind::Av, &next).unwrap();
    assert!(built.rebuilt, "after restore the poisoned slot re-produces");
    assert_eq!(built.run.unwrap().exit_code, Some(17));
}

/// V-04: the engineering latency bound on the WARM path — the P90 of
/// repeated identical builds (the precise-cache path an editor drives),
/// with the attributed zero-toolchain oracle around the whole loop.
#[test]
fn p978_av_warm_build_p90_bounded() {
    let root = scratch("latency");
    let mut driver = M2Build::new(&root);
    driver.build(TargetKind::Av, &common::b01_inputs()).unwrap();
    let (mut warm_ms, owned, available) = common::owned_tools_guard(|| {
        let mut ms = Vec::new();
        for _ in 0..5 {
            let (out, elapsed) =
                common::timed(|| driver.build(TargetKind::Av, &common::b01_inputs()));
            out.unwrap();
            ms.push(elapsed);
        }
        ms
    });
    assert!(available, "attributed oracle available");
    assert!(owned.is_empty(), "warm loop spawned toolchain: {owned:?}");
    warm_ms.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p90 = warm_ms[4]; // worst of 5 (>= P90 of the sample)
    assert!(
        p90 <= 2000.0,
        "warm-path P90 {p90:.0}ms exceeds the 2000ms project bound ({warm_ms:?})"
    );
}
