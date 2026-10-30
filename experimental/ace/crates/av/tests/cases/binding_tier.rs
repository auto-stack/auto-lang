//! T-06 tier: real 796 semantic-ref binding (PLAN-798 V-13, §5.6).
//!
//! The av profile's identities now ride the landed `ac-sema-contract`
//! domain. The plan-local FNV fingerprint snapshot is retired (the HIR
//! adapter itself stays); these tests pin the binding invariants:
//! same-name/different-owner separation, restart/ordering stability,
//! schema/version mismatch rejection, tampered-ref rejection, and the
//! cache keys actually keying on the shared domain.

use ace_av::driver::Driver;
use ace_av::semrefs::{
    admit_unit_ref, av_package, content_version_of, unit_decl_ref, unit_logical_id, unit_sema_ref,
};
use ace_semantics_contract::{DeclKind, PackageId, SemaRef, SEMA_PRODUCER_ID, SEMA_REF_SCHEMA_V1};

/// V-13: same function name under different owners (package/module) mints
/// DIFFERENT logical ids — no cross-owner collision, and both live as
/// independent cache units in one store.
#[test]
fn av_binding_same_name_different_owner_separated() {
    let pkg = av_package();
    let foreign = PackageId::new("other-plan/module-set");
    let a = unit_logical_id(&pkg, "m_left", "d_main");
    let b = unit_logical_id(&pkg, "m_right", "d_main");
    let c = unit_logical_id(&foreign, "m_left", "d_main");
    let mut all = vec![a.clone(), b.clone(), c.clone()];
    all.sort();
    all.dedup();
    assert_eq!(all.len(), 3, "same name, three owner scopes → three ids");

    // Cache-level separation: two modules with the same function name in
    // ONE store hit their OWN units (no cross-contamination).
    let root = std::env::temp_dir().join(format!("av798-bind-own-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let src = "fn main() int {\n    return 7\n}\n";
    let mut left = Driver::with_store(&root, "p").unwrap();
    left.set_owner(Some("m_left".to_string()));
    let r1 = left.request(src, "d_main", &[]);
    assert!(r1.ok(), "{:?}", r1.diag);
    assert_eq!(r1.value_i32(), Some(7));
    let mut right = Driver::with_store(&root, "p").unwrap();
    right.set_owner(Some("m_right".to_string()));
    let r2 = right.request(src, "d_main", &[]);
    assert!(r2.ok(), "{:?}", r2.diag);
    // Each owner's second request hits its OWN unit (1 unit each).
    let r1b = left.request(src, "d_main", &[]);
    assert_eq!(
        *r1b.counters.get("cache_hit_abc").unwrap_or(&0),
        1,
        "m_left hits own unit"
    );
    let r2b = right.request(src, "d_main", &[]);
    assert_eq!(
        *r2b.counters.get("cache_hit_abc").unwrap_or(&0),
        1,
        "m_right hits own unit"
    );
    // The manifests carry DISTINCT shared-domain logical ids.
    let l1 = r1b.manifest.as_ref().unwrap().units[0].logical.clone();
    let l2 = r2b.manifest.as_ref().unwrap().units[0].logical.clone();
    assert!(!l1.is_empty() && !l2.is_empty() && l1 != l2);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-13: registration/restart ordering stability — logical ids are
/// component-derived, so minting the same decls in a different order (or
/// in a different process) yields the IDENTICAL id set; content versions
/// are canonical-derived and equally stable.
#[test]
fn av_binding_restart_ordering_stable() {
    let pkg = av_package();
    let forward: Vec<String> = (0..8)
        .map(|i| unit_logical_id(&pkg, "m", &format!("d_fn_{i}")))
        .collect();
    let mut reverse: Vec<String> = (0..8)
        .rev()
        .map(|i| unit_logical_id(&pkg, "m", &format!("d_fn_{i}")))
        .collect();
    reverse.sort();
    let mut sorted_forward = forward.clone();
    sorted_forward.sort();
    assert_eq!(
        sorted_forward, reverse,
        "id sets identical under re-ordering"
    );

    // Cross-PROCESS stability: the worker `request` path (a fresh process)
    // derives the same logical id for the same decl — the manifest is the
    // observable. Also proves the unit still HITS after the restart (keys
    // re-derive identically).
    let root = std::env::temp_dir().join(format!("av798-bind-restart-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let src = "fn main() int {\n    return 3\n}\n";
    let src_file = root.with_extension("at");
    std::fs::write(&src_file, src).unwrap();
    let mut d = Driver::with_store(&root, "p").unwrap();
    let warm = d.request(src, "d_main", &[]);
    assert!(warm.ok());
    let warm_logical = warm.manifest.as_ref().unwrap().units[0].logical.clone();

    let out = std::process::Command::new(env!("CARGO_BIN_EXE_av-worker"))
        .args([
            "request",
            "--store",
            root.to_str().unwrap(),
            "--src-file",
            src_file.to_str().unwrap(),
            "--entry",
            "d_main",
        ])
        .output()
        .expect("spawn worker");
    assert!(out.status.success(), "restart request ok");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("worker json");
    assert!(v["ok"].as_bool().unwrap(), "{v}");
    // Hit in the restarted process ⇒ the key re-derived identically.
    let counters = v["counters"].as_object().unwrap();
    assert_eq!(
        counters.get("cache_hit_abc").and_then(|x| x.as_u64()),
        Some(1)
    );

    // The manifest logical id re-derives from its OWN owner component
    // (the HIR module id) — the restart-stability property is that the
    // id is a pure function of (package, module, kind, name), minted
    // identically in every process.
    let owner = warm.manifest.as_ref().unwrap().owner.clone();
    let rederived = unit_logical_id(&av_package(), &owner, "d_main");
    assert_eq!(
        warm_logical, rederived,
        "manifest logical == freshly minted id"
    );
    assert!(!owner.is_empty());
    let _ = std::fs::remove_dir_all(&root);
}

/// V-13: schema / version mismatch — the persisted envelope must admit
/// under the sema producer + v1 schema; a wrong-schema envelope (or a
/// foreign owner) is a controlled rejection, and a body edit changes the
/// content version (different key, miss not wrong hit).
#[test]
fn av_binding_schema_and_version_mismatch_rejected() {
    let pkg = av_package();
    let decl = unit_decl_ref(&pkg, "m", "d_main", b"canonical-body");
    let good = unit_sema_ref(decl.logical.as_str());
    assert!(
        admit_unit_ref(&good, &decl, &good.payload_id).is_ok(),
        "well-formed ref admits"
    );
    // F798-R3-02: the envelope must name the EXACT payload being consumed —
    // an honest envelope over an unrelated payload does not admit it.
    let err = admit_unit_ref(&good, &decl, "unrelated-payload-id").unwrap_err();
    assert!(
        matches!(err, ace_semantics_contract::RefError::UnknownPayload { .. }),
        "unbound payload rejected: {err:?}"
    );

    // Wrong schema.
    let bad_schema = SemaRef {
        owner: SEMA_PRODUCER_ID,
        schema: "ac796/semaref/v999",
        payload_id: decl.logical.as_str().to_string(),
    };
    let err = admit_unit_ref(&bad_schema, &decl, &bad_schema.payload_id).unwrap_err();
    assert!(
        matches!(err, ace_semantics_contract::RefError::WrongSchema { .. }),
        "{err:?}"
    );

    // Wrong owner (foreign producer envelope).
    let bad_owner = SemaRef {
        owner: "other/producer",
        schema: SEMA_REF_SCHEMA_V1,
        payload_id: decl.logical.as_str().to_string(),
    };
    let err = admit_unit_ref(&bad_owner, &decl, &bad_owner.payload_id).unwrap_err();
    assert!(
        matches!(err, ace_semantics_contract::RefError::WrongOwner { .. }),
        "{err:?}"
    );

    // Tampered logical id (foreign-minted id on an honest envelope).
    let mut tampered = unit_decl_ref(&pkg, "m", "d_main", b"canonical-body");
    let flipped = format!("{}x", tampered.logical.as_str());
    tampered.logical = ace_semantics_contract::LogicalId(flipped);
    let err = admit_unit_ref(&good, &tampered, &good.payload_id).unwrap_err();
    assert!(
        matches!(
            err,
            ace_semantics_contract::RefError::TamperedLogical { .. }
        ),
        "tampered logical rejected: {err:?}"
    );

    // Content version: a body edit changes the version → a different key.
    let v1 = content_version_of(b"canonical-body");
    let v2 = content_version_of(b"canonical-body-EDITED");
    assert_ne!(v1, v2);
    // And the edited decl's ref carries the new content version.
    let edited = unit_decl_ref(&pkg, "m", "d_main", b"canonical-body-EDITED");
    assert_ne!(decl.content.as_str(), edited.content.as_str());

    // Kind axis: an fn ref is NOT a record ref of the same name.
    let as_record = ace_semantics_contract::logical_id(&pkg, "m", DeclKind::Record, "d_main");
    assert_ne!(decl.logical.as_str(), as_record.as_str());
}

/// V-13: no product dependency — the binding consumes only the contract
/// crate (compile-time property; this test pins the crate surface used).
#[test]
fn av_binding_consumes_contract_surface_only() {
    // The full consumed surface in one place: if 796 renames any of these,
    // this file fails to compile — the frozen consumption point.
    let _ = ace_semantics_contract::SEMA_PRODUCER_ID;
    let _ = ace_semantics_contract::SEMA_REF_SCHEMA_V1;
    let _ = ace_semantics_contract::SEMA_PAGE_SCHEMA_V1;
    let p = av_package();
    let d = unit_decl_ref(&p, "m", "d_x", b"dump");
    let env = unit_sema_ref(d.logical.as_str());
    assert!(admit_unit_ref(&env, &d, &env.payload_id).is_ok());
}
