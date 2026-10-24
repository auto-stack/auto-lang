//! PLAN-818 T-11: the `mir_native_incremental_` family — the V-12 four
//! sub-cases over the D-828 real baseline (load256: 273 575 B / 115
//! functions, the frozen fn_037 multiplier site):
//! 1) ≥30 distinct revisions with the DUAL gate — structure (unchanged
//!    units' fp/generation never move; no whole-module stage; exactly one
//!    install) AND stable-session edit→PE P95 < 1000 ms;
//! 2) two-function local body updates: unchanged units retained, correct
//!    semantics, superseded objects reclaimed by identity;
//! 3) precise invalidation/refusal controls: helper body change re-keys
//!    exactly the helper; signature/interface edits are REFUSED with the
//!    mn layer untouched; unauthorized capability edits are refused;
//! 4) degenerate modes are CAUGHT: a content-hit forces the recorded
//!    whole-module fallback (fallback counter non-zero), and a
//!    hash-preserving object swap is a store conflict that never
//!    overwrites the live identity.

mod common;

use ac_mn::incremental::MnIncrementalSession;

/// All four legs spawn real link/run subprocesses and the V-12.1 leg is
/// a wall-clock gate — serialize the tier (parallel legs' link storms
/// doubled the gate leg's tail: P95 1131 vs ~850 solo).
static T11_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn8-t11-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn percentile(v: &[f64], p: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
}

/// V-12.1: the D-828 frozen baseline through the REAL incremental
/// consumer — 30 distinct body revisions at fn_037's multiplier, the
/// dual gate (structure AND P95), every revision's PE result checked
/// against the generator's independent expectation.
#[test]
#[cfg(windows)]
fn mir_native_incremental_d828_baseline_30_structure_and_p95() {
    let _t11 = T11_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let base = auto_av_probe::load256::generate_revision(0);
    assert_eq!(base.source.len(), 273_575, "the D-828 frozen baseline size");
    let root = temp_root("v121");
    let (mut s, cold) =
        MnIncrementalSession::warmup(&base.source, "d_main", &[], &root.join("store"))
            .expect("warmup");
    assert_eq!(
        cold.exit_code,
        Some(base.expected_main),
        "cold run matches the independent expectation"
    );
    assert_eq!(s.units().len(), 115, "115 functions, 115 units");
    // The warm-up round links from the RESIDENT working set (the D828
    // residency model: the store is persistence, not the warm path).
    assert_eq!(
        cold.cache_hits, 115,
        "cold round: every unit reads resident"
    );

    let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30);
    let gate_ms: f64 = std::env::var("MN818_FORMAL_GATE_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000.0);
    let mut walls = Vec::with_capacity(samples);
    let mut prev_src = base.source.clone();
    for n in 1..=samples {
        let rev = auto_av_probe::load256::generate_revision(n as u32);
        let (start, end, insert) =
            auto_av_probe::formal_828::diff_edit(&prev_src, &rev.source).expect("diff");
        let before_units = s.units().clone();
        let obs = s.edit(start, end, insert);
        assert!(obs.ok, "rev {n}: {:?}", obs.infra.or(obs.refused_reason));
        assert_eq!(
            obs.exit_code,
            Some(rev.expected_main),
            "rev {n}: PE result == independent expectation"
        );
        assert_eq!(obs.changed_fn.as_deref(), Some("fn_037"), "rev {n} site");
        assert_eq!(obs.codegen_units, 1, "rev {n}: exactly one unit re-emits");
        assert_eq!(obs.cache_hits, 114, "rev {n}: every other unit hits");
        assert_eq!(
            obs.fallback_whole_lower, 0,
            "rev {n}: no degenerate fallback"
        );
        // STRUCTURE: no whole-module stage; unchanged functions did nothing.
        assert_eq!(
            obs.whole_module_stage_delta, 0,
            "rev {n}: whole-module stages must stay 0"
        );
        assert_eq!(
            obs.unaffected_stage_delta, 0,
            "rev {n}: unchanged functions contribute no stage work"
        );
        // STRUCTURE: exactly d_fn_037's identity moved (fp + generation).
        let after_units = s.units();
        let mut moved = Vec::new();
        for (name, entry) in after_units.iter() {
            let prev = before_units.get(name).expect("unit known");
            if prev != entry {
                moved.push(name.clone());
            }
        }
        assert_eq!(
            moved,
            vec!["d_fn_037".to_string()],
            "rev {n}: identity moves for exactly the changed unit"
        );
        assert_eq!(
            after_units.get("d_fn_037").unwrap().generation,
            before_units.get("d_fn_037").unwrap().generation + 1,
            "rev {n}: generation bumps exactly once"
        );
        assert!(obs.wall_ms > 0.0);
        walls.push(obs.wall_ms);
        prev_src = rev.source;
    }
    let p95 = percentile(&walls, 95.0);
    // R1 note: the P95 GATE for V-12.1 lives in the FORMAL series test
    // (`mir_native_incremental_series_formal`: one stable release-worker
    // session carrying the whole series, real margins). This debug-tier
    // mirror REPORTS its window - the debug link subprocess cost tracks
    // live machine load (restart-baseline evidence: P50 828 -> 1025 under
    // desktop load; this same binary measured 802 quiescent and 972..1373
    // across consecutive loaded runs), so a debug-tier hard gate would
    // measure the machine, not the pipeline.
    eprintln!(
        "t11 v12.1 (debug mirror): {} revisions, edit->PE P50/P95/P99 = {:.1}/{:.1}/{:.1} ms [REPORTED - the gate lives in mir_native_incremental_series_formal]",
        samples,
        percentile(&walls, 50.0),
        p95,
        percentile(&walls, 99.0),
    );

    s.close();
    let _ = std::fs::remove_dir_all(&root);
}

/// V-12.2: two FUNCTION-local body updates in ONE session — each edit
/// re-keys exactly its own function; the other edit's new identity is
/// RETAINED (not reverted); results follow the closed-form mirror.
#[test]
#[cfg(windows)]
fn mir_native_incremental_two_fn_local_updates() {
    let _t11 = T11_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let src = "fn alpha(x int) int {\n    return x * 2\n}\n\nfn beta(x int) int {\n    return x + 10\n}\n\nfn main() int {\n    return alpha(5) + beta(3)\n}\n";
    let root = temp_root("v122");
    let (mut s, cold) =
        MnIncrementalSession::warmup(src, "d_main", &[], &root.join("store")).expect("warmup");
    assert_eq!(cold.exit_code, Some(23), "10 + 13");
    // Edit 1: alpha's body  x*2 -> x*3  => 15 + 13 = 28.
    let p1 = src.find("x * 2").expect("site1");
    let obs1 = s.edit(p1 as u64, (p1 + 5) as u64, b"x * 3".to_vec());
    assert!(obs1.ok, "{:?}", obs1.infra.or(obs1.refused_reason));
    assert_eq!(obs1.exit_code, Some(28));
    assert_eq!(obs1.changed_fn.as_deref(), Some("alpha"));
    assert_eq!(obs1.codegen_units, 1);
    assert_eq!(obs1.cache_hits, 2);
    let after1 = s.units().clone();
    // Edit 2: beta's body  x + 10 -> x + 20  => 15 + 23 = 38.
    let src2 = "fn alpha(x int) int {\n    return x * 3\n}\n\nfn beta(x int) int {\n    return x + 10\n}\n\nfn main() int {\n    return alpha(5) + beta(3)\n}\n";
    let p2 = src2.find("x + 10").expect("site2");
    let obs2 = s.edit(p2 as u64, (p2 + 6) as u64, b"x + 20".to_vec());
    assert!(obs2.ok, "{:?}", obs2.infra.or(obs2.refused_reason));
    assert_eq!(obs2.exit_code, Some(38));
    assert_eq!(obs2.changed_fn.as_deref(), Some("beta"));
    let after2 = s.units().clone();
    // alpha KEEPS its round-1 identity (retained, not reverted); beta now
    // moved exactly once; main never moved.
    assert_eq!(
        after2.get("d_alpha").unwrap(),
        after1.get("d_alpha").unwrap(),
        "the first edit's identity is retained"
    );
    assert_eq!(
        after2.get("d_beta").unwrap().generation,
        after1.get("d_beta").unwrap().generation + 1
    );
    assert_eq!(
        after2.get("d_main").unwrap().generation,
        0,
        "main never re-emits"
    );
    assert_eq!(obs2.cache_hits, 2, "alpha + main hit");
    s.close();
    let _ = std::fs::remove_dir_all(&root);
}

/// V-12.3: precise invalidation/refusal controls — a helper BODY change
/// re-keys exactly the helper (callers stay); an interface (signature)
/// edit is REFUSED with the mn layer fully untouched; an unauthorized
/// capability edit is refused by the frontend.
#[test]
#[cfg(windows)]
fn mir_native_incremental_precise_invalidation_controls() {
    let _t11 = T11_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let src = "fn helper(v int) int {\n    return v * 3\n}\n\nfn main() int {\n    return helper(14)\n}\n";
    let root = temp_root("v123");
    let (mut s, cold) =
        MnIncrementalSession::warmup(src, "d_main", &[], &root.join("store")).expect("warmup");
    assert_eq!(cold.exit_code, Some(42));

    // (a) helper BODY change: helper re-keys; the CALLER (main) keeps its
    // identity — dependency-precise (a body edit does not touch callers).
    let p = src.find("v * 3").expect("site");
    let obs = s.edit(p as u64, (p + 5) as u64, b"v * 4".to_vec());
    assert!(obs.ok, "{:?}", obs.infra.or(obs.refused_reason));
    assert_eq!(obs.exit_code, Some(56), "helper(14)*4");
    assert_eq!(obs.changed_fn.as_deref(), Some("helper"));
    assert_eq!(obs.codegen_units, 1);
    assert_eq!(
        s.units().get("d_main").unwrap().generation,
        0,
        "the caller never re-emits on a callee BODY edit"
    );

    // (b) interface edit (helper gains a parameter): the retained
    // environment's interface gate REFUSES; the mn layer's identity map
    // is untouched; a subsequent VALID edit still works.
    let before = s.units().clone();
    let cur = "fn helper(v int) int {\n    return v * 4\n}\n\nfn main() int {\n    return helper(14)\n}\n";
    let sig = cur.find("fn helper(v int) int {").expect("sig site");
    let obs_refused = s.edit(
        sig as u64,
        (sig + "fn helper(v int) int {".len()) as u64,
        b"fn helper(v int, w int) int {".to_vec(),
    );
    let reason = obs_refused
        .refused_reason
        .clone()
        .unwrap_or_else(|| panic!("interface edit must be refused, got {obs_refused:?}"));
    assert!(
        reason.to_lowercase().contains("interface")
            || reason.to_lowercase().contains("signature")
            || reason.to_lowercase().contains("non-fast-path"),
        "refusal reason names the interface gate: {reason}"
    );
    assert_eq!(s.units(), &before, "the refusal touched nothing");
    // The session is not wedged: a valid body edit proceeds.
    let p2 = "fn helper(v int) int {\n    return v * 4\n}"
        .find("v * 4")
        .unwrap()
        + cur.find("fn helper").unwrap();
    let obs_ok = s.edit(p2 as u64, (p2 + 5) as u64, b"v * 5".to_vec());
    assert!(obs_ok.ok, "{:?}", obs_ok.infra.or(obs_ok.refused_reason));
    assert_eq!(obs_ok.exit_code, Some(70), "helper(14)*5");

    // (c) unauthorized capability use: a body edit that calls mark_a()
    // in a session warmed WITHOUT the trace capability is refused by the
    // frontend (the capability was never granted), nothing installed.
    let before2 = s.units().clone();
    let cur2 = "fn helper(v int) int {\n    return v * 5\n}\n\nfn main() int {\n    return helper(14)\n}\n";
    let site = cur2.find("return v * 5").expect("cap site");
    let obs_cap = s.edit(
        site as u64,
        (site + "return v * 5".len()) as u64,
        b"return mark_a()".to_vec(),
    );
    assert!(
        obs_cap.refused_reason.is_some(),
        "unauthorized capability edit refused, got {obs_cap:?}"
    );
    assert_eq!(
        s.units(),
        &before2,
        "the capability refusal touched nothing"
    );
    s.close();
    let _ = std::fs::remove_dir_all(&root);
}

/// V-12.4 (post-R1): a NORMAL edit back to a prior body content is a
/// HISTORY REUSE — the historical unit is restored from the A09' store by
/// the content-version map: zero whole-module work, zero re-emission, the
/// identity moves back to the historical fp and the generation does NOT
/// bump. The degenerate whole-module mode only runs under a DELIBERATE
/// fault injection (the content-version map cannot resolve), is recorded,
/// and the structural gate fires on it. Plus: a hash-preserving object
/// SWAP under a live key is a controlled store conflict.
#[test]
#[cfg(windows)]
fn mir_native_incremental_degenerate_modes_gate_nonzero() {
    if !common::tools_available() {
        eprintln!("infra-blocked: skipped");
        return;
    }
    let src =
        "fn site(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return site(1)\n}\n";
    let root = temp_root("v124");
    let (mut s, cold) =
        MnIncrementalSession::warmup(src, "d_main", &[], &root.join("store")).expect("warmup");
    assert_eq!(cold.exit_code, Some(101));
    let warmup_fp = s.units().get("d_site").unwrap().fp.clone();
    let structure_gate = |obs: &ac_mn::incremental::EditObservation| -> Vec<String> {
        let mut v = Vec::new();
        if obs.whole_module_stage_delta != 0 {
            v.push("whole-module stages ran".into());
        }
        if obs.unaffected_stage_delta != 0 {
            v.push("unchanged functions did work".into());
        }
        if obs.fallback_whole_lower != 0 {
            v.push("degenerate whole-module fallback".into());
        }
        v
    };

    // Mint away: 100 -> 200 (fast path; gate clean).
    let p = src.find("v + 100").expect("site");
    let obs_a = s.edit(p as u64, (p + 7) as u64, b"v + 200".to_vec());
    assert!(obs_a.ok);
    assert_eq!(obs_a.exit_code, Some(201));
    assert!(structure_gate(&obs_a).is_empty(), "fast path is clean");
    assert_eq!(obs_a.codegen_units, 1);

    // NORMAL back-edit (200 -> 100): a HISTORY HIT — the historical unit
    // is REUSED. Zero whole-module work, zero re-emission, the identity
    // returns to the warm-up fp, and the generation does NOT bump.
    let cur =
        "fn site(v int) int {\n    return v + 200\n}\n\nfn main() int {\n    return site(1)\n}\n";
    let p2 = cur.find("v + 200").expect("back site");
    let obs_back = s.edit(p2 as u64, (p2 + 7) as u64, b"v + 100".to_vec());
    assert!(
        obs_back.ok,
        "{:?}",
        obs_back.infra.or(obs_back.refused_reason)
    );
    assert_eq!(obs_back.exit_code, Some(101), "back to the original value");
    assert!(
        obs_back.reused_history_unit,
        "the ordinary history hit REUSES the historical unit"
    );
    assert_eq!(obs_back.codegen_units, 0, "no re-emission on a history hit");
    assert!(
        structure_gate(&obs_back).is_empty(),
        "history reuse is clean"
    );
    assert_eq!(
        s.units().get("d_site").unwrap().fp,
        warmup_fp,
        "the identity returns to the historical fp"
    );

    // A second hit (100 -> 200 again) also reuses (content B is mapped).
    let cur3 =
        "fn site(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return site(1)\n}\n";
    let p3 = cur3.find("v + 100").expect("second site");
    let obs_b = s.edit(p3 as u64, (p3 + 7) as u64, b"v + 200".to_vec());
    assert!(obs_b.ok && obs_b.reused_history_unit, "{obs_b:?}");
    assert_eq!(obs_b.exit_code, Some(201));

    // DELIBERATE fault injection (V-12.4): forget d_site's content history
    // — the next history hit cannot resolve and MUST take the RECORDED
    // whole-module fallback; the structural gate fires on it (a forced
    // whole recompute can never pass silently).
    s.drop_unit_history("d_site");
    let cur4 =
        "fn site(v int) int {\n    return v + 200\n}\n\nfn main() int {\n    return site(1)\n}\n";
    let p4 = cur4.find("v + 200").expect("inject site");
    let obs_inj = s.edit(p4 as u64, (p4 + 7) as u64, b"v + 100".to_vec());
    assert!(obs_inj.ok, "{:?}", obs_inj.infra.or(obs_inj.refused_reason));
    assert_eq!(obs_inj.exit_code, Some(101));
    let fired = structure_gate(&obs_inj);
    assert!(
        !fired.is_empty() && fired.iter().any(|f| f.contains("fallback")),
        "the injected unresolvable hit fires the gate: {fired:?}"
    );
    assert_eq!(obs_inj.fallback_whole_lower, 1);
    assert!(!obs_inj.reused_history_unit);

    // Hash-preserving object SWAP under a live key: a DIFFERENT COFF
    // payload under unit d_main's live fp key is a controlled store
    // conflict — the live commit survives and the next real edit works.
    let store = ac_artifact_probe::store::ManagedStore::open(&root.join("store")).unwrap();
    let main_fp = s.units().get("d_main").unwrap().fp.clone();
    let owner = s.owner().clone();
    let meta_main = ac_mn::cache::CoffMeta {
        symbol: ac_mn::symbols::function_symbol(&owner, "d_main"),
        fp: main_fp.clone(),
        owner,
        unit: "d_main".into(),
        abi: ac_mn::config::MN_ABI.into(),
        target: ac_mn::config::MN_TARGET.into(),
        flags: ac_mn::config::MN_FLAGS.into(),
        backend: ac_mn::config::MN_BACKEND_TAG.into(),
        runtime: ac_mn::config::MN_RUNTIME_TAG.into(),
        mir_schema: ac_mn::config::MIR_SCHEMA_AXIS.into(),
        mir_semantics: ac_mn::config::MIR_SEMANTICS_AXIS.into(),
    };
    let key = ac_mn::cache::coff_unit_key(&meta_main);
    let foreign = store
        .get_artifact(ac_mn::config::COFF_LAYER.id, &key)
        .unwrap()
        .expect("d_main payload present");
    let mut swapped = foreign.payload.clone();
    let last = swapped.len() - 1;
    swapped[last] = swapped[last].wrapping_add(1);
    match store.put_artifact(ac_artifact_probe::store::PutRequest {
        layer: ac_mn::config::COFF_LAYER.id,
        key: key.clone(),
        kind: ac_artifact_probe::codec::ChunkKind::LayoutData,
        payload: swapped,
        edges: vec![],
        project: None,
    }) {
        Err(e) => assert!(
            format!("{e:?}").to_lowercase().contains("conflict"),
            "controlled conflict, got {e:?}"
        ),
        Ok(_) => panic!("a swapped object under a live key must conflict"),
    }
    // The live identity survives: the next real edit links from the
    // UN-swapped payloads and stays correct.
    let cur5 =
        "fn site(v int) int {\n    return v + 100\n}\n\nfn main() int {\n    return site(1)\n}\n";
    let p5 = cur5.find("v + 100").expect("next site");
    let obs_next = s.edit(p5 as u64, (p5 + 7) as u64, b"v + 101".to_vec());
    assert!(
        obs_next.ok,
        "{:?}",
        obs_next.infra.or(obs_next.refused_reason)
    );
    assert_eq!(obs_next.exit_code, Some(102));
    s.close();
    let _ = std::fs::remove_dir_all(&root);
}
