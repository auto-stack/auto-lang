//! PLAN-828 T-07 remainder — V-07 reclamation matrix and C05-02/03
//! lifetime observability.
//!
//! The reclamation contract (§5.4/V-07/C05-02/03): reachable old
//! generations stay ALIVE and runnable while a real owner holds them (an
//! externally pinned snapshot, a live PreparedEdit); when the last owner
//! goes away the object is actually reclaimable — observed through Weak
//! handles captured while the object was resident, never through
//! `drop(session)` or `history_len`. The session's own persistence never
//! disturbs unrelated user rows in the shared store (the V-07 sentinel).

use ace_av::session::{warmup, warmup_persisted, HISTORY_CAP};
use ace_av_core::av::{execute_program, CancelFlag, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};

const SRC: &str = "fn helper(x int) int { let r = x * 3; return r + 7 }\nfn main() int { let a = helper(5); return a + 1 }\nfn tail() int { return 9 }\nfn extra() int { return 6 }\n";

fn value_of(s: &ace_av::session::FunctionIncrementalSession) -> i32 {
    match s.run(&ExecLimits::default(), &CancelFlag::new(false)).0 {
        ExecOutcome::Success {
            value: Value::I32(v),
            ..
        } => v,
        other => panic!("{other:?}"),
    }
}

/// Replace the literal after `x * ` with `k` — one DISTINCT body content
/// per k (the content identity keys on the canonical HIR, so different
/// constants are different contents).
fn edit_factor(s: &mut ace_av::session::FunctionIncrementalSession, k: u32) -> bool {
    let src = s.src().to_string();
    let at = src.find("x * ").expect("factor site") + 4;
    let digits: String = src[at..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    let end = at + digits.len();
    let out = s
        .edit(at as u64, end as u64, k.to_string().into_bytes())
        .expect("edit");
    assert!(out.ok, "{out:?}");
    out.ok
}

/// V-07: an externally pinned OLD snapshot keeps its program runnable;
/// releasing the last pin reclaims the root (Weak-observed — no
/// drop(session), no history_len).
#[test]
fn v07_old_snapshot_pin_hold_then_release_reclaims() {
    let dir = std::env::temp_dir().join(format!("p828-t07-pin-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828-t07").expect("warm-up");
    let cancel = CancelFlag::new(false);
    assert_eq!(value_of(&s), 23);

    let pin0 = s.current();
    let root0 = std::sync::Arc::downgrade(&pin0);
    // One body edit installs a new revision; the pinned old root is NOT
    // yanked (V-05).
    let src = s.src().to_string();
    let at = src.find("x * 3").unwrap() as u64 + 4;
    let out = s.edit(at, at + 1, b"4".to_vec()).unwrap();
    assert!(out.ok);
    assert_eq!(s.revision(), 1);
    assert_eq!(value_of(&s), 28, "the new revision serves new calls");
    let (old, _) = execute_program(&pin0, &ExecLimits::default(), &cancel);
    assert!(
        matches!(
            old,
            ExecOutcome::Success {
                value: Value::I32(23),
                ..
            }
        ),
        "the pinned old snapshot keeps its own program: {old:?}"
    );
    // Release the last external owner: the old root dies HERE (no other
    // owner exists — the session moved on and history never holds roots).
    drop(pin0);
    assert!(
        root0.upgrade().is_none(),
        "releasing the last pin reclaims the old root"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// V-07/C05-02: a RETIRED history pair dies once nothing owns it — an
/// unpinned evicted content's code is reclaimed, while the SAME
/// generation held by an external pin stays alive and runnable until the
/// pin goes away. The resident history stays bounded throughout.
#[test]
fn v07_retired_history_dies_unpinned_and_survives_pin() {
    let dir = std::env::temp_dir().join(format!("p828-t07-gc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828-t07").expect("warm-up");
    let cancel = CancelFlag::new(false);

    // The warm content W (x*3): pin its snapshot and capture weak views.
    let pin_w = s.current();
    let (_, w_hir, _) = s.resident("helper").unwrap();
    let w_code = std::sync::Arc::downgrade(&_w_code(&s));
    let _ = w_hir;
    assert_eq!(value_of(&s), 23);

    // Mint DISTINCT contents, capturing a weak view of EVERY minted
    // pair (none of them is pinned). Skip k=3 — the warm content is
    // already in history.
    let mut codes: Vec<std::sync::Weak<ace_av_core::function_code::FunctionCode>> = Vec::new();
    let mut k = 1u32;
    let mut distinct = 0usize;
    while distinct < HISTORY_CAP + 10 {
        if k != 3 {
            edit_factor(&mut s, k);
            codes.push(std::sync::Arc::downgrade(&_w_code(&s)));
            distinct += 1;
        }
        k += 1;
    }
    // Total distinct contents = warm + `distinct`; the per-function cap
    // keeps HISTORY_CAP — under ANY eviction order at least
    // (total - cap - 1) of the UNPINNED captured pairs must be gone (the
    // -1 allows the warm entry to be an eviction victim itself, which
    // the pin keeps alive regardless).
    let total_contents = distinct + 1;
    let must_be_dead = total_contents - HISTORY_CAP - 1;
    // history_len sums EVERY function (helper ≤ HISTORY_CAP; the three
    // unchanged functions keep one warm entry each).
    assert!(
        s.history_len() <= HISTORY_CAP + 3,
        "resident history bounded (per-function cap + warm entries): {}",
        s.history_len()
    );

    // The PINNED warm generation is still alive and runs its own program.
    assert!(
        w_code.upgrade().is_some(),
        "a pinned generation stays alive"
    );
    let (old, _) = execute_program(&pin_w, &ExecLimits::default(), &cancel);
    assert!(
        matches!(
            old,
            ExecOutcome::Success {
                value: Value::I32(23),
                ..
            }
        ),
        "the pinned old program keeps running: {old:?}"
    );
    // The unpinned evicted contents are RECLAIMED (their roots died as
    // the session moved on; their history entries were evicted) — no
    // zombie retention beyond the cap.
    let dead = codes.iter().filter(|w| w.upgrade().is_none()).count();
    assert!(
        dead >= must_be_dead,
        "retired, unpinned history pairs are reclaimed (dead {dead} of {} captured,          at least {must_be_dead} required)",
        codes.len()
    );

    // C05-02 deterministic release leg: close() drops every session-owned
    // record and history entry — the pin becomes the pinned generation's
    // LAST owner (the pinned program keeps running across the close),
    // and releasing it reclaims the code.
    s.close();
    assert!(
        w_code.upgrade().is_some(),
        "an external pin keeps its generation alive across close()"
    );
    let (old, _) = execute_program(&pin_w, &ExecLimits::default(), &cancel);
    assert!(
        matches!(
            old,
            ExecOutcome::Success {
                value: Value::I32(23),
                ..
            }
        ),
        "the pinned program still runs after close: {old:?}"
    );
    drop(pin_w);
    assert!(
        w_code.upgrade().is_none(),
        "the pinned generation dies with its last owner"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

fn _w_code(
    s: &ace_av::session::FunctionIncrementalSession,
) -> std::sync::Arc<ace_av_core::function_code::FunctionCode> {
    s.resident("helper").unwrap().2
}

/// C05-03: a PreparedEdit OWNS its never-installed pair — the weak view
/// stays alive while the candidate is held, and dies exactly at
/// consumption: cancel, a closed-session commit refusal, or drop. A
/// candidate that COMMITS stays alive (it became the current code).
#[test]
fn c05_candidate_pair_dies_at_consumption_not_before() {
    let src = SRC;
    let mut s = warmup(src, "d_main", &[]).expect("warm-up");
    let plus1 = src.find("+ 1").unwrap() as u64 + 2;

    // Held candidate: alive while the PreparedEdit exists.
    let held = s
        .prepare_edit(plus1, plus1 + 1, b"2".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    let held_weak = held.code_weak();
    assert!(held_weak.upgrade().is_some(), "the candidate owns its pair");
    // Cancel consumes: the never-installed pair dies with it.
    let out = s.cancel_prepared(held);
    assert!(!out.ok);
    assert!(
        held_weak.upgrade().is_none(),
        "a cancelled candidate's pair dies at consumption"
    );

    // A closed session's commit REFUSAL also consumes the candidate.
    let refused = s
        .prepare_edit(plus1, plus1 + 1, b"2".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    let refused_weak = refused.code_weak();
    s.close();
    let out = s.commit(refused);
    assert!(!out.ok, "closed commit is refused");
    assert!(
        refused_weak.upgrade().is_none(),
        "the refused candidate's pair dies at consumption"
    );

    // The positive control: a candidate that COMMITS becomes the current
    // code — its pair stays alive (owned by the session now).
    let mut s2 = warmup(src, "d_main", &[]).expect("warm-up 2");
    let committed = s2
        .prepare_edit(plus1, plus1 + 1, b"2".to_vec())
        .unwrap()
        .ok()
        .unwrap();
    let committed_weak = committed.code_weak();
    let out = s2.commit(committed);
    assert!(out.ok);
    assert!(
        committed_weak.upgrade().is_some(),
        "an installed candidate's pair is owned by the session"
    );
    assert_eq!(value_of(&s2), 5 * 3 + 7 + 2);
}

/// V-07 sentinel: the session's persistence and reclamation never touch
/// unrelated USER rows in the shared store — a sentinel row written
/// before the soak is byte-identical after dozens of revisions, a
/// restore and its re-persist.
#[test]
fn v07_user_sentinel_row_untouched_by_soak_and_restore() {
    use ace_av::cache::{open_store, ABC_LAYER};
    let dir = std::env::temp_dir().join(format!("p828-t07-sent-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    // A user-owned row in the same store, outside the session's key
    // space (its own owner/kind components).
    let sentinel_key = ace_artifacts::identity::QueryKey::new(
        &ABC_LAYER,
        vec![
            ("owner".into(), "user-cache-sentinel".into()),
            ("kind".into(), "user-row".into()),
            ("fp".into(), "v1".into()),
        ],
    );
    let sentinel_payload = b"user-sentinel-payload-v1".to_vec();
    {
        let st = open_store(&dir).unwrap();
        st.put_artifact(ace_artifacts::store::PutRequest {
            layer: "A09",
            key: sentinel_key.clone(),
            kind: ace_artifacts::codec::ChunkKind::LayoutData,
            payload: sentinel_payload.clone(),
            edges: vec![],
            project: None,
        })
        .expect("sentinel put");
    }

    // Soak: dozens of persisted revisions + a warm restart that
    // re-persists over the same store.
    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828-t07").unwrap();
    let mut k = 2u32;
    let mut distinct = 0usize;
    while distinct < HISTORY_CAP + 2 {
        if k != 3 {
            edit_factor(&mut s, k);
            distinct += 1;
        }
        k += 1;
    }
    let edited = s.src().to_string();
    drop(s);
    let r = ace_av::session::restore(&edited, "d_main", &[], &dir, "p828-t07").unwrap();
    let last_k = k - 1;
    let expect: i32 = 5 * last_k as i32 + 7 + 1;
    let (out, _) = r.run(&ExecLimits::default(), &CancelFlag::new(false));
    assert!(
        matches!(out, ExecOutcome::Success { value: Value::I32(v), .. } if v == expect),
        "restored value {out:?} expected {expect}"
    );

    // The sentinel row survived byte-identical.
    let st = open_store(&dir).unwrap();
    let loaded = st
        .get_artifact("A09", &sentinel_key)
        .expect("sentinel read")
        .expect("the user sentinel row is still present");
    assert_eq!(
        loaded.payload, sentinel_payload,
        "session persistence/GC never rewrites unrelated user rows"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// F-11: the session manifest is a STORE ROOT that restore CONSUMES. A
/// matching root admits the warm path (counter `restore_manifest_matched`);
/// a store whose registered roots do NOT describe the current program
/// falls back cold (self-healing: the cold path re-persists a matching
/// manifest); a store with NO registered manifest roots is a partial/
/// legacy store served by the per-unit content-addressed path exactly as
/// before (`restore_manifest_absent`).
#[test]
fn f11_manifest_matched_absent_and_rejected_paths() {
    use ace_av::cache::{caps_fingerprint, open_store, put_enveloped, EnvelopeIdentity};
    use ace_av::session::restore;

    let dir = std::env::temp_dir().join(format!("p828-t07-man-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cancel = CancelFlag::new(false);

    // Matched: a session store restores warm through its manifest root.
    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828-t07").unwrap();
    let at = SRC.find("x * 3").unwrap() as u64 + 4;
    assert!(s.edit(at, at + 1, b"4".to_vec()).unwrap().ok);
    let edited = s.src().to_string();
    drop(s);
    let r = restore(&edited, "d_main", &[], &dir, "p828-t07").unwrap();
    assert_eq!(r.counters().get("restore_manifest_matched"), 1);
    assert!(
        r.counters().get("restore_units_admitted") > 0,
        "warm restore"
    );
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(28),
            ..
        }
    ));
    drop(r);

    // Absent roots: a partial store (all manifest roots removed) is
    // served by the per-unit content-addressed path — same warm units.
    {
        let st = open_store(&dir).unwrap();
        let removed = st
            .remove_roots("p828-t07", "session-manifest")
            .expect("remove roots");
        assert!(removed >= 1);
    }
    let r = restore(&edited, "d_main", &[], &dir, "p828-t07").unwrap();
    assert_eq!(
        r.counters().get("restore_manifest_absent"),
        1,
        "no registered manifest roots — partial-store path"
    );
    assert!(
        r.counters().get("restore_units_admitted") > 0,
        "per-unit admission still serves the partial store"
    );
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(28),
            ..
        }
    ));
    drop(r);
    // (The cold path never ran, so the roots stay absent — restore again
    // keeps taking the partial-store path.)
    let r = restore(&edited, "d_main", &[], &dir, "p828-t07").unwrap();
    assert_eq!(r.counters().get("restore_manifest_absent"), 1);
    drop(r);

    // Rejected: registered roots that do NOT describe this program force
    // the cold fallback — the store describes the OLD source. The cold
    // path re-persists a matching manifest, so the NEXT restore warms.
    let dir2 = std::env::temp_dir().join(format!("p828-t07-man2-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir2);
    std::fs::create_dir_all(&dir2).unwrap();
    drop(warmup_persisted(SRC, "d_main", &[], &dir2, "p828-t07").unwrap());
    let r = restore(&edited, "d_main", &[], &dir2, "p828-t07").unwrap();
    assert_eq!(
        r.counters().get("restore_manifest_rejected"),
        1,
        "the store's manifest describes another program"
    );
    assert_eq!(
        r.counters().get("restore_units_admitted"),
        0,
        "rejected manifest → cold fallback"
    );
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(28),
            ..
        }
    ));
    drop(r);
    let r = restore(&edited, "d_main", &[], &dir2, "p828-t07").unwrap();
    assert_eq!(
        r.counters().get("restore_manifest_matched"),
        1,
        "the cold fallback re-persisted a matching manifest (self-heal)"
    );
    assert!(r.counters().get("restore_units_admitted") > 0);
    drop(r);

    // Garbage root: a registered but undecodable manifest root is a
    // non-candidate; with no other root it REJECTS (units alone do not
    // authorize the warm path past a registered manifest), and the cold
    // path heals by adding a real root.
    let dir3 = std::env::temp_dir().join(format!("p828-t07-man3-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir3);
    std::fs::create_dir_all(&dir3).unwrap();
    drop(warmup_persisted(SRC, "d_main", &[], &dir3, "p828-t07").unwrap());
    {
        let st = open_store(&dir3).unwrap();
        // A foreign-owner manifest envelope: registered as a root of the
        // kind but never unwrappable for this module — a non-candidate.
        let gkey = ace_artifacts::identity::QueryKey::new(
            &ace_av::cache::ABC_LAYER,
            vec![("kind".into(), "garbage-manifest".into())],
        );
        let gid = EnvelopeIdentity::manifest("garbage-owner", 0);
        let garbage = put_enveloped(
            &st,
            &gkey,
            "A09",
            &gid,
            b"not-json",
            vec![],
            Some("p828-t07"),
        )
        .unwrap();
        assert!(st.remove_roots("p828-t07", "session-manifest").unwrap() >= 1);
        st.register_root("p828-t07", &garbage, "session-manifest")
            .unwrap();
    }
    let r = restore(SRC, "d_main", &[], &dir3, "p828-t07").unwrap();
    assert_eq!(
        r.counters().get("restore_manifest_rejected"),
        1,
        "a garbage-only manifest root rejects the warm path"
    );
    assert_eq!(r.counters().get("restore_units_admitted"), 0);
    let (out, _) = r.run(&ExecLimits::default(), &cancel);
    assert!(matches!(
        out,
        ExecOutcome::Success {
            value: Value::I32(23),
            ..
        }
    ));
    drop(r);
    let r = restore(SRC, "d_main", &[], &dir3, "p828-t07").unwrap();
    assert_eq!(r.counters().get("restore_manifest_matched"), 1, "healed");
    let _ = caps_fingerprint;
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&dir2);
    let _ = std::fs::remove_dir_all(&dir3);
}

/// F-09/V-07: the resident inventory is a bounded watermark — it does
/// not grow with the edit count (per-function history cap; the cold
/// bundle is fixed), and close() releases every session-owned axis.
#[test]
fn f09_resident_inventory_bounded_watermark_and_close_release() {
    let dir = std::env::temp_dir().join(format!("p828-t07-inv-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = warmup_persisted(SRC, "d_main", &[], &dir, "p828-t07").unwrap();

    let base = s.resident_inventory();
    assert!(base.functions == 4 && base.cold_bundle_nodes > 0 && base.history_entries == 4);

    // Soak well past the per-function cap.
    let mut k = 1u32;
    let mut distinct = 0usize;
    while distinct < HISTORY_CAP + 10 {
        if k != 3 {
            edit_factor(&mut s, k);
            distinct += 1;
        }
        k += 1;
    }
    let soaked = s.resident_inventory();
    assert!(
        soaked.history_entries <= HISTORY_CAP * soaked.functions,
        "history watermark bounded: {:?}",
        soaked
    );
    // More edits do NOT move the watermark (eviction really runs).
    for extra in [100u32, 101, 102, 103, 104, 105, 106, 107] {
        edit_factor(&mut s, extra);
    }
    let after = s.resident_inventory();
    assert_eq!(
        after, soaked,
        "the resident watermark converged — it does not grow with edits"
    );

    // close() releases every session-owned axis of the inventory.
    s.close();
    let closed = s.resident_inventory();
    assert_eq!(closed.functions, 0);
    assert_eq!(closed.hir_body_nodes, 0);
    assert_eq!(closed.cold_bundle_nodes, 0);
    assert_eq!(closed.history_entries, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

/// T07-R2-01/R2-02 (collected from the R2 review's t07-controls.rs, same
/// inputs/oracles): manifest STRICTNESS — a duplicate entry cannot
/// authorize a missing function, an unknown format is not this program's
/// directory — and REQUIRED REFERENCES — the manifest's edges protect
/// the current restore closure through real GC, independently of
/// per-artifact soft pins.
#[test]
fn r2_manifest_strictness_and_required_references() {
    use ace_artifacts::identity::QueryKey;
    use ace_artifacts::store::{run_gc, GcConfig};
    use ace_av::cache::{
        get_by_artifact_id, open_store, put_enveloped, EnvelopeIdentity, ABC_LAYER,
    };
    use ace_av::session::restore;
    use serde_json::Value;
    use std::time::{SystemTime, UNIX_EPOCH};

    const R2SRC: &str = "fn helper() int { return 7 }\nfn main() int { return helper() + 2 }\nfn tail() int { return 9 }\n";
    const PROJECT: &str = "p828-r2-perm";
    fn dir(tag: &str) -> std::path::PathBuf {
        let p = std::env::temp_dir().join(format!(
            "p828-r2perm-{tag}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
    fn seed(tag: &str) -> (std::path::PathBuf, String, Value) {
        let path = dir(tag);
        let s = warmup_persisted(R2SRC, "d_main", &[], &path, PROJECT).unwrap();
        let owner = s.current().code()[0].unit().module().owner.module.clone();
        drop(s);
        let st = open_store(&path).unwrap();
        let roots = st.root_artifacts(PROJECT, "session-manifest").unwrap();
        assert_eq!(roots.len(), 1, "exactly one live manifest root");
        let body =
            get_by_artifact_id(&st, &roots[0], &EnvelopeIdentity::manifest(&owner, 0)).unwrap();
        let v = serde_json::from_slice(&body).unwrap();
        (path, owner, v)
    }
    fn replace(path: &std::path::PathBuf, owner: &str, v: &Value, tag: &str) {
        let st = open_store(path).unwrap();
        st.remove_roots(PROJECT, "session-manifest").unwrap();
        let key = QueryKey::new(&ABC_LAYER, vec![("kind".into(), format!("perm-{tag}"))]);
        let id = put_enveloped(
            &st,
            &key,
            "A09",
            &EnvelopeIdentity::manifest(owner, 0),
            &serde_json::to_vec(v).unwrap(),
            vec![],
            Some(PROJECT),
        )
        .unwrap();
        st.register_root(PROJECT, &id, "session-manifest").unwrap();
    }

    // Valid manifest warms.
    {
        let (p, _, _) = seed("valid");
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(s.counters().get("restore_manifest_matched"), 1);
        assert_eq!(s.counters().get("restore_units_admitted"), 3);
    }
    // A duplicate entry cannot hide a missing function.
    {
        let (p, owner, mut v) = seed("duplicate");
        let units = v["units"].as_array_mut().unwrap();
        assert_eq!(units.len(), 3);
        units[1] = units[0].clone();
        replace(&p, &owner, &v, "duplicate");
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(
            s.counters().get("restore_manifest_rejected"),
            1,
            "missing function cannot be authorized by a duplicate listed pair"
        );
    }
    // Unknown format is not the current directory.
    {
        let (p, owner, mut v) = seed("format");
        v["format"] = Value::from(999);
        replace(&p, &owner, &v, "future");
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(s.counters().get("restore_manifest_rejected"), 1);
    }
    // A legal REORDERED directory still matches (set equality, not order).
    {
        let (p, owner, mut v) = seed("reorder");
        let units = v["units"].as_array_mut().unwrap();
        units.reverse();
        replace(&p, &owner, &v, "reorder");
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(s.counters().get("restore_manifest_matched"), 1);
        assert_eq!(s.counters().get("restore_units_admitted"), 3);
    }
    // The live manifest's required closure survives REAL GC.
    {
        let (p, owner, _) = seed("gc");
        let st = open_store(&p).unwrap();
        let roots = st.root_artifacts(PROJECT, "session-manifest").unwrap();
        let receipt = run_gc(&st, GcConfig { high: 1, low: 0 }).unwrap();
        let root_present =
            get_by_artifact_id(&st, &roots[0], &EnvelopeIdentity::manifest(&owner, 0)).is_ok();
        assert!(receipt.ran && root_present);
        drop(st);
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(
            s.counters().get("restore_units_admitted"),
            3,
            "a retained live manifest keeps its required current ABC/sidecar closure"
        );
    }
    // Retiring every per-artifact soft pin still leaves the closure
    // alive through the manifest's required edges.
    {
        let (p, owner, _) = seed("required");
        let st = open_store(&p).unwrap();
        let roots = st.root_artifacts(PROJECT, "session-manifest").unwrap();
        let retired = st.remove_roots(PROJECT, "soft").unwrap();
        assert!(retired > 0, "the regenerable HIR cache carries soft pins");
        assert_eq!(
            st.root_artifacts(PROJECT, "session-manifest").unwrap(),
            roots
        );
        let receipt = run_gc(&st, GcConfig { high: 1, low: 0 }).unwrap();
        let root_present =
            get_by_artifact_id(&st, &roots[0], &EnvelopeIdentity::manifest(&owner, 0)).is_ok();
        assert!(receipt.ran && root_present);
        drop(st);
        let s = restore(R2SRC, "d_main", &[], &p, PROJECT).unwrap();
        assert_eq!(
            s.counters().get("restore_units_admitted"),
            3,
            "manifest's required edges protect the current units independently of soft pins"
        );
    }
}

/// T07-R2-02 stable watermark: after a revision superseded by a new
/// manifest, the OLD unit payloads lose their required edges (the old
/// manifest root retired) and real GC reclaims them, while a user-owned
/// row under its own root survives untouched.
#[test]
fn r2_superseded_units_reclaimed_and_user_row_survives_gc() {
    use ace_artifacts::identity::{QueryKey, RefKind};
    use ace_artifacts::store::{run_gc, GcConfig};
    use ace_av::cache::{
        get_enveloped_opt, open_store, put_enveloped, EnvelopeIdentity, ABC_LAYER,
    };

    const R2SRC: &str = "fn helper() int { return 7 }\nfn main() int { return helper() + 2 }\nfn tail() int { return 9 }\n";
    const PROJECT: &str = "p828-r2-reclaim";
    let dir = std::env::temp_dir().join(format!("p828-r2reclaim-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let mut s = warmup_persisted(R2SRC, "d_main", &[], &dir, PROJECT).unwrap();
    // A user-owned row pinned by its OWN root (shared project).
    let user_key = QueryKey::new(
        &ABC_LAYER,
        vec![("owner".into(), "user-reclaim-sentinel".into())],
    );
    let user_payload = b"user-row-payload".to_vec();
    {
        let st = open_store(&dir).unwrap();
        let id = put_enveloped(
            &st,
            &user_key,
            "A09",
            &EnvelopeIdentity::abc_bound("user-reclaim-sentinel", "d_user", "v1"),
            &user_payload,
            vec![],
            Some(PROJECT),
        )
        .unwrap();
        st.register_root(PROJECT, &id, "user-row").unwrap();
    }
    // Revise helper: the OLD ABC payload keeps serving the warm history
    // until the new manifest retires the old root.
    let at = R2SRC.find("return 7").unwrap() as u64 + 7;
    assert!(s.edit(at, at + 1, b"8".to_vec()).unwrap().ok);
    let edited = s.src().to_string();
    drop(s);
    {
        let st = open_store(&dir).unwrap();
        assert_eq!(
            st.root_artifacts(PROJECT, "session-manifest")
                .unwrap()
                .len(),
            1,
            "the superseded manifest root retired at publish"
        );
        let receipt = run_gc(&st, GcConfig { high: 1, low: 0 }).unwrap();
        assert!(receipt.ran);
        assert!(receipt.deleted_chunks > 0, "superseded payloads reclaimed");
        // The user row survived byte-identical under its own root.
        let loaded = get_enveloped_opt(
            &st,
            &user_key,
            "A09",
            &EnvelopeIdentity::abc_bound("user-reclaim-sentinel", "d_user", "v1"),
        )
        .expect("user read")
        .expect("user row alive");
        assert_eq!(loaded, user_payload);
        let _ = RefKind::RequiredData;
    }
    // The CURRENT closure still restores warm.
    let r = ace_av::session::restore(&edited, "d_main", &[], &dir, PROJECT).unwrap();
    assert_eq!(r.counters().get("restore_units_admitted"), 3);
    let _ = std::fs::remove_dir_all(&dir);
}
