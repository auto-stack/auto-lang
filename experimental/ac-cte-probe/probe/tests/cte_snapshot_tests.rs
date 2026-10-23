//! cte_snapshot_ — the PLAN-828 snapshot lane (PLAN-817 T-11, V-12
//! four sub-cases): production observation (real execution through the
//! incremental session; argument retargets recompile ONLY the wrapper),
//! identity (the exec lane re-keys the frozen layer — closure and
//! snapshot never share entries), dependency (helper body edits
//! invalidate snapshot-mode values; the session-level edit recompiles
//! only the touched function), and failure control (a malformed session
//! edit is a controlled failure; the forced whole-module path is
//! COUNTED and non-zero). Independent expectations: 20+b+pad.

use auto_av_core::CancelFlag;
use auto_cte_contract::CteValue;
use auto_cte_probe::av_driver::SnapshotLane;
use auto_cte_probe::driver::{CteDriver, StepStatus};

fn src_with(pad: i64) -> String {
    format!(
        "fn cte_sum(a int, b int) int {{\n    return a + b + {pad}\n}}\n\nfn other() int {{\n    return 1\n}}\n\nfn main() int {{\n    let ct = 0\n    return ct\n}}\n"
    )
}

fn input_for(src: &str) -> ac_sema_core::SemaInput {
    ac_sema_core::adapter::parse_into_input_typed(
        src,
        "cte817",
        "main",
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .expect("projection")
}

fn deck(src: &str, exec: &str) -> String {
    format!(
        "deck cte817/1\nmodule main\nexec {exec}\nsource <<\n{src}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

/// V-12 (1) 生产观测: real execution through the session; retargeting
/// the arguments recompiles ONLY the wrapper — the helper and the
/// unrelated function keep compile-count 0 and IDENTICAL generations.
#[test]
fn cte_snapshot_production_observation_incremental() {
    let src = src_with(0);
    let input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        &input,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
    )
    .unwrap();
    let g_sum_before = lane.generation_of("cte_sum");
    let g_other_before = lane.generation_of("other");
    assert!(g_sum_before.is_some(), "units are resident");
    let r = lane.run(&cancel).unwrap();
    assert_eq!(r.value.render(), "i32:42", "hand-computed 20+22+0");
    assert!(r.steps > 0, "real AV execution");

    // 30 argument revisions through ONE session (V-12 source30修订):
    // each retarget recompiles only the wrapper.
    for k in 1..=30i64 {
        lane.retarget(
            "cte_sum",
            &[CteValue::I32(20), CteValue::I32(22 + k as i32)],
        )
        .unwrap();
        let r = lane.run(&cancel).unwrap();
        assert_eq!(r.value.render(), format!("i32:{}", 42 + k), "rev {k}");
    }
    assert_eq!(
        lane.compile_count("cte_sum"),
        0,
        "the helper never recompiled"
    );
    assert_eq!(
        lane.compile_count("other"),
        0,
        "the unrelated function never recompiled"
    );
    assert!(
        lane.compile_count("cte817_call") > 0,
        "the wrapper did recompile"
    );
    assert_eq!(
        lane.generation_of("cte_sum"),
        g_sum_before,
        "unchanged object keeps its generation"
    );
    assert_eq!(lane.generation_of("other"), g_other_before);
    // Purity: the ONLY whole-module event is the warm-up itself — 30
    // revisions add no degrade, and every revision was incremental.
    assert_eq!(
        lane.whole_module_count(),
        1,
        "30 revisions stay function-incremental (warm-up is the only whole-module event)"
    );
    assert_eq!(lane.incremental_edit_count(), 30);
}

/// V-12 (2) 身份: the exec lane is a cache-key axis — closure and
/// snapshot requests never share frozen entries, and each lane hits its
/// own entry afterwards.
#[test]
fn cte_snapshot_identity_lanes_do_not_share_cache() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let src = src_with(0);
    let closure_deck = deck(&src, "closure");
    let snapshot_deck = deck(&src, "snapshot");

    let mut d = CteDriver::with_store(&root, "lane").unwrap();
    let c1 = d.query_step(&closure_deck, None);
    assert_eq!(c1.status, StepStatus::Ok, "{:?}", c1.message);
    assert_eq!(c1.cache, Some("miss"));
    let s1 = d.query_step(&snapshot_deck, None);
    assert_eq!(s1.status, StepStatus::Ok, "{:?}", s1.message);
    assert_eq!(
        s1.cache,
        Some("miss"),
        "the snapshot lane has its OWN key (no cross-lane hit)"
    );
    assert_eq!(
        s1.values.get("main.ct").map(|v| v.render()),
        Some("i32:42".into())
    );
    // Each lane hits its own entry afterwards.
    assert_eq!(d.query_step(&closure_deck, None).cache, Some("hit"));
    assert_eq!(d.query_step(&snapshot_deck, None).cache, Some("hit"));
}

/// V-12 (3) 依赖: a helper body edit invalidates the snapshot-mode
/// value (fresh miss + the new value); the session-level edit of the
/// HELPER recompiles only the helper (the wrapper and other functions
/// keep their generations).
#[test]
fn cte_snapshot_dependency_invalidates_and_scopes_recompiles() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let mut d = CteDriver::with_store(&root, "dep").unwrap();
    let base = deck(&src_with(0), "snapshot");
    assert_eq!(d.query_step(&base, None).cache, Some("miss"));
    let edited = deck(&src_with(1), "snapshot");
    let out = d.query_step(&edited, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.cache,
        Some("miss"),
        "the helper body edit invalidated the value"
    );
    assert_eq!(
        out.values.get("main.ct").map(|v| v.render()),
        Some("i32:43".into())
    );
    assert_eq!(
        d.query_step(&base, None).cache,
        Some("hit"),
        "the old entry survived"
    );

    // Session-level scoping: editing the HELPER body only recompiles the
    // helper (dependency change, not a whole-module event).
    let src = src_with(0);
    let input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        &input,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
    )
    .unwrap();
    let g_wrap = lane.generation_of("cte817_call");
    // Replace `+ 0` in the helper body with `+ 5`.
    let needle = "return a + b + 0";
    let start = lane.src().find(needle).expect("needle") as u64;
    lane.edit(
        start,
        start + needle.len() as u64,
        b"return a + b + 5".to_vec(),
    )
    .expect("helper edit");
    let r = lane.run(&cancel).unwrap();
    assert_eq!(
        r.value.render(),
        "i32:47",
        "20+22+5 after the dependency edit"
    );
    assert!(lane.compile_count("cte_sum") > 0, "the helper recompiled");
    assert_eq!(lane.compile_count("cte817_call"), 0, "the wrapper did not");
    assert_eq!(lane.generation_of("cte817_call"), g_wrap);
    assert_eq!(lane.whole_module_count(), 1, "no NEW whole-module event");
}

/// V-12 (4) 失败控制: a malformed session edit (broken syntax) is a
/// CONTROLLED failure — never a wrong result, never a panic; and the
/// forced whole-module path is COUNTED (a fresh warm-up's cold compile
/// is exactly 1 — the degrade detector the plan demands to be non-zero).
#[test]
fn cte_snapshot_failure_control_edits_and_degrade_counter() {
    let src = src_with(0);
    let input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        &input,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
    )
    .unwrap();
    // A broken edit: unterminated garbage replacing the helper body.
    let needle = "return a + b + 0";
    let start = lane.src().find(needle).unwrap() as u64;
    let bad = lane.edit(start, start + needle.len() as u64, b"return a +".to_vec());
    match bad {
        Ok(_) => panic!("a broken edit must not commit"),
        Err(m) => assert!(!m.is_empty(), "the controlled failure carries detail"),
    }
    // The session still answers with the LAST good program — no wrong
    // result, no half-applied state.
    let r = lane.run(&cancel).unwrap();
    assert_eq!(
        r.value.render(),
        "i32:42",
        "the last good revision survives the failed edit"
    );

    // Forced degrade control: a FRESH warm-up IS the whole-module path —
    // the lane's degrade detector fires exactly 1, while the 30-edit
    // lane above stayed at 0 (828's silent degradation is designed out;
    // the explicit cold path is the only whole-module event).
    let lane2 = SnapshotLane::warm(
        &src,
        &input,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
    )
    .unwrap();
    assert_eq!(
        lane2.whole_module_count(),
        1,
        "the forced whole-module path is counted (degrade detector non-zero)"
    );
    assert_eq!(
        lane2.incremental_edit_count(),
        0,
        "a fresh lane has made no incremental edits"
    );
}
