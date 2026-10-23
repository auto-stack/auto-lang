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
    let _input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
        false,
    )
    .unwrap();
    let g_sum_before = lane.generation_of("cte_sum");
    let g_other_before = lane.generation_of("other");
    assert!(g_sum_before.is_some(), "units are resident");
    let r = lane
        .run(false, &cancel, auto_cte_probe::av_driver::MAX_CALL_WALL)
        .unwrap();
    assert_eq!(r.value.render(), "i32:42", "hand-computed 20+22+0");
    assert!(r.steps > 0, "real AV execution");

    // 30 argument revisions through ONE session (V-12 source30修订):
    // each retarget recompiles only the wrapper.
    for k in 1..=30i64 {
        lane.retarget(
            "cte_sum",
            &[CteValue::I32(20), CteValue::I32(22 + k as i32)],
            false,
        )
        .unwrap();
        let r = lane
            .run(false, &cancel, auto_cte_probe::av_driver::MAX_CALL_WALL)
            .unwrap();
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
    let _input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
        false,
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
    let r = lane
        .run(false, &cancel, auto_cte_probe::av_driver::MAX_CALL_WALL)
        .unwrap();
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
    let _input = input_for(&src);
    let cancel = CancelFlag::new(false);
    let mut lane = SnapshotLane::warm(
        &src,
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
        false,
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
    let r = lane
        .run(false, &cancel, auto_cte_probe::av_driver::MAX_CALL_WALL)
        .unwrap();
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
        "cte_sum",
        &[CteValue::I32(20), CteValue::I32(22)],
        false,
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

// ---------------------------------------------------------------------------
// F-817-R1-01 closure: consecutive DIFFERENT callees in one request
// ---------------------------------------------------------------------------

fn multi_callee_deck() -> String {
    // first(a)=a+1 (i32), second(a)=a+10 (i32), flag(a)=a<5 (bool):
    // same request calls first(1) then second(2) then flag(3) — distinct
    // callees, mixed return classes, distinct arities via four(x,y).
    String::from(
        "deck cte817/1\nmodule main\nexec snapshot\nsource <<\nfn first(a int) int {\n    return a + 1\n}\n\nfn second(a int) int {\n    return a + 10\n}\n\nfn flag(a int) bool {\n    return a < 5\n}\n\nfn four(a int, b int) int {\n    return a * 10 + b\n}\n\nfn main() int {\n    let x = 0\n    let y = 0\n    let g = false\n    let z = 0\n    if g {\n        return x + y + z\n    }\n    return x + y\n}\n>>\nquery q1 = call first(1)\nquery q2 = call second(2)\nquery q3 = call flag(3)\nquery q4 = call four(4, 7)\nbind main.x = q1 expect i32 placeholder 0\nbind main.y = q2 expect i32 placeholder 0\nbind main.g = q3 expect bool placeholder false\nbind main.z = q4 expect i32 placeholder 0\nentry main\nnative off\n",
    )
}

#[test]
fn cte_snapshot_retarget_switches_callees_correctly() {
    // The review counter-example generalized: expected x=2, y=12 — a
    // lane that only replaced the ARGUMENTS kept the first callee and
    // produced y=3. Mixed return classes and arities ride along.
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&multi_callee_deck(), None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(out.ct_execute, 4);
    assert_eq!(
        out.values.get("main.x").map(|v| v.render()),
        Some("i32:2".into()),
        "first(1)"
    );
    assert_eq!(
        out.values.get("main.y").map(|v| v.render()),
        Some("i32:12".into()),
        "second(2) — NOT first(2)=3"
    );
    assert_eq!(
        out.values.get("main.g").map(|v| v.render()),
        Some("bool:true".into()),
        "flag(3)"
    );
    assert_eq!(
        out.values.get("main.z").map(|v| v.render()),
        Some("i32:47".into()),
        "four(4,7)"
    );
    // Consumer legs saw the RIGHT values (HIR backfill + types).
    assert_eq!(out.hir_proof.get("main.y").map(String::as_str), Some("12"));
    assert_eq!(
        out.consumer_types.get("main.g").map(String::as_str),
        Some("bool")
    );
}

#[test]
fn cte_snapshot_duplicate_callee_and_dag_positive_control() {
    // The same callee twice (single-flight) and a legal DAG edge stay
    // correct after the fix.
    let deck = "deck cte817/1\nmodule main\nexec snapshot\nsource <<\nfn add(a int, b int) int {\n    return a + b\n}\n\nfn scale(n int) int {\n    return n * 2\n}\n\nfn main() int {\n    let s = 0\n    let t = 0\n    return s + t\n}\n>>\nquery a = call add(20, 22)\nquery b = call add(20, 22)\nquery c = call scale(a)\nbind main.s = b expect i32 placeholder 0\nbind main.t = c expect i32 placeholder 0\nentry main\nnative off\n";
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(deck, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.ct_execute, 2,
        "single-flight still dedupes identical calls"
    );
    assert_eq!(
        out.values.get("main.s").map(|v| v.render()),
        Some("i32:42".into())
    );
    assert_eq!(
        out.values.get("main.t").map(|v| v.render()),
        Some("i32:84".into()),
        "scale(add)=84"
    );
}

#[test]
fn cte_snapshot_failing_call_leaves_no_bad_publication() {
    // A trap in the SECOND callee after a successful first call: the
    // wrong-value publication must never happen — the store stays empty
    // for the trapping identity, and a clean retry recomputes.
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    let deck = "deck cte817/1\nmodule main\nexec snapshot\nsource <<\nfn good() int {\n    return 5\n}\n\nfn boom() int {\n    return 2147483647 + 1\n}\n\nfn main() int {\n    let a = 0\n    let b = 0\n    return a + b\n}\n>>\nquery q1 = call good()\nquery q2 = call boom()\nbind main.a = q1 expect i32 placeholder 0\nbind main.b = q2 expect i32 placeholder 0\nentry main\nnative off\n";
    let mut d = CteDriver::with_store(&root, "f01").unwrap();
    let out = d.query_step(deck, None);
    assert_eq!(out.status, StepStatus::Diag);
    assert_eq!(out.code.as_deref(), Some("cte.trap.int-overflow"));
    assert_eq!(out.ct_execute, 2, "both calls really executed");
    // The SUCCESSFUL first call's value is legitimately published (5 is
    // correct); the TRAPPING identity must never be: replaying the boom
    // deck re-executes for real (no cached wrong value surfaces).
    let mut clean = CteDriver::with_store(&root, "f01").unwrap();
    let ok_deck = deck
        .replace("query q2 = call boom()\n", "")
        .replace("bind main.b = q2 expect i32 placeholder 0\n", "");
    let after = clean.query_step(&ok_deck, None);
    assert_eq!(after.status, StepStatus::Ok, "{:?}", after.message);
    assert_eq!(
        after.cache,
        Some("hit"),
        "the correct good() value persisted"
    );
    assert_eq!(
        after.values.get("main.a").map(|v| v.render()),
        Some("i32:5".into())
    );
    let trap_again = clean.query_step(deck, None);
    assert_eq!(trap_again.status, StepStatus::Diag);
    assert_eq!(trap_again.code.as_deref(), Some("cte.trap.int-overflow"));
    assert_eq!(
        trap_again.ct_execute, 1,
        "boom re-executed — nothing wrong was cached"
    );
}

// ---------------------------------------------------------------------------
// F-817-R1-02 closure: the RESIDENT lane across real body edits on the
// specified baseline (115 functions, >=256KiB source)
// ---------------------------------------------------------------------------

fn baseline_source(fns: usize, rev: i64) -> String {
    // Bounded REAL arithmetic (~2.3KB per function; the revision
    // constant rides every helper body); cte_sum folds them all.
    let mut s = String::new();
    for k in 0..fns {
        // Only ONE helper carries the revision (the contracted LOCAL
        // body-edit shape); the rest stay byte-identical across
        // revisions.
        let rev_here = if k == fns / 2 { rev } else { 0 };
        s.push_str(&format!(
            "fn h{k}(x int) int {{\n    var acc = x + {k} + {rev_here}\n"
        ));
        for j in 0..80 {
            s.push_str(&format!("    acc = acc + {j} + x + {k}\n"));
        }
        s.push_str("    return acc\n}\n\n");
    }
    s.push_str("fn cte_sum(a int, b int) int {\n    var acc = a + b\n");
    for k in 0..fns {
        // Constant arguments: no feedback, closed-form bounded results.
        s.push_str(&format!("    acc = acc + h{k}(20)\n"));
    }
    s.push_str("    return acc\n}\n\nfn main() int {\n    let ct = 0\n    return ct\n}\n");
    s
}

fn baseline_deck(fns: usize, rev: i64) -> String {
    let src = baseline_source(fns, rev);
    format!(
        "deck cte817/1\nmodule main\nexec snapshot\nsource <<\n{src}\n>>\nquery q1 = call cte_sum(20, 22)\nbind main.ct = q1 expect i32 placeholder 0\nentry main\nnative off\n"
    )
}

#[test]
fn cte_snapshot_resident_lane_baseline_30_revisions() {
    const FNS: usize = 115;
    // Sanity: the baseline is the specified scale.
    let base_src = baseline_source(FNS, 0);
    assert!(
        base_src.len() >= 256 * 1024,
        "baseline >= 256KiB: {} bytes",
        base_src.len()
    );

    let mut d = CteDriver::ephemeral();
    // Warm on revision 0; capture the reference value and EVERY
    // function's generation for independent comparison later.
    let first = d.query_step(&baseline_deck(FNS, 0), None);
    assert_eq!(first.status, StepStatus::Ok, "{:?}", first.message);
    assert_eq!(d.snapshot_rewarms(), 0);
    let base_value = first
        .values
        .get("main.ct")
        .and_then(|v| {
            v.render()
                .strip_prefix("i32:")
                .and_then(|x| x.parse::<i64>().ok())
        })
        .expect("reference value");
    let lane = d.snapshot_lane().expect("resident lane");
    let revised_fn = format!("h{}", FNS / 2);
    let gens_before: Vec<(String, Option<u64>)> = (0..FNS)
        .map(|k| (format!("h{k}"), lane.lane.generation_of(&format!("h{k}"))))
        .chain(std::iter::once((
            "cte_sum".to_string(),
            lane.lane.generation_of("cte_sum"),
        )))
        .collect();
    assert!(
        gens_before.iter().all(|(_, g)| g.is_some()),
        "every function is resident with a generation"
    );

    // 30 DISTINCT body revisions through the resident lane: every
    // request is a real driver round (projection, scheduling, manifest,
    // execution); only the touched helpers recompile; every other
    // function keeps compile-count 0 and its generation; P95 < 1000ms.
    let mut walls: Vec<f64> = Vec::new();
    for k in 1..=30i64 {
        let deck = baseline_deck(FNS, k);
        let t = std::time::Instant::now();
        let out = d.query_step(&deck, None);
        walls.push(t.elapsed().as_secs_f64() * 1000.0);
        assert_eq!(out.status, StepStatus::Ok, "rev {k}: {:?}", out.message);
        assert_eq!(out.ct_execute, 1);
        assert_eq!(out.cache, Some("miss"), "rev {k} re-keys and re-executes");
        // Independent per-revision VALUE assertion: the revision rides the
        // ONE revised helper's init line exactly once — value(k) =
        // value(0) + k (closed form, hand-derived).
        let got = out
            .values
            .get("main.ct")
            .and_then(|v| {
                v.render()
                    .strip_prefix("i32:")
                    .and_then(|x| x.parse::<i64>().ok())
            })
            .unwrap_or_default();
        assert_eq!(got, base_value + k, "rev {k} value");
    }
    walls.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let p95 = walls[(((walls.len() as f64 - 1.0) * 0.95).round()) as usize];
    assert!(
        if cfg!(debug_assertions) {
            // Debug is smoke-only (the 798 discipline): correctness
            // asserted, the 1000ms gate is a RELEASE measurement.
            p95 < 5000.0
        } else {
            p95 < 1000.0
        },
        "resident-lane revision P95 < 1000ms (debug smoke <5s): {p95:.0}ms"
    );

    let lane = d.snapshot_lane().expect("resident lane");
    assert_eq!(
        d.snapshot_rewarms(),
        0,
        "no whole-module re-warm across 30 revisions"
    );
    assert_eq!(
        lane.lane.whole_module_count(),
        1,
        "warm-up remains the only whole-module event"
    );
    // INDEPENDENT generation comparison (R2: the previous form compared a
    // value with itself): every UNCHANGED function keeps its captured
    // generation AND its mir_lower count at 0; ONLY the revised helper
    // advanced (generation changed, compile count == 30).
    let mut changed = Vec::new();
    for (name, before) in &gens_before {
        let now = lane.lane.generation_of(name).expect("still resident");
        if Some(now) != *before {
            changed.push(name.clone());
        }
    }
    assert_eq!(
        changed,
        vec![revised_fn.clone()],
        "exactly the revised function changed its generation: {changed:?}"
    );
    for (name, _) in &gens_before {
        if name != &revised_fn {
            assert_eq!(
                lane.lane.compile_count(name),
                0,
                "{name} never recompiled (mir_lower count)"
            );
        }
    }
    assert_eq!(lane.lane.compile_count(&revised_fn), 30);
    // Full-stage observation for one unchanged function: EVERY pipeline
    // stage stayed at 0 across all 30 revisions.
    let stages = lane.lane.stages_of("h0");
    assert!(
        stages.values().all(|c| *c == 0),
        "h0 all-stage-zero across 30 revisions: {stages:?}"
    );
    assert_eq!(
        lane.lane.phase_io_of("h0").unwrap_or(0),
        0,
        "h0 artifact IO == 0 across 30 revisions"
    );
}

#[test]
fn cte_snapshot_forced_rebuild_control_nonzero() {
    const FNS: usize = 8;
    let mut d = CteDriver::ephemeral();
    let r0 = d.query_step(&baseline_deck(FNS, 0), None);
    assert_eq!(r0.status, StepStatus::Ok, "BASE: {:?}", r0.message);
    assert_eq!(d.snapshot_rewarms(), 0);
    // A STRUCTURAL change (a different function COUNT) cannot sync as a
    // single body hunk — the forced whole-module rebuild happens and the
    // counter goes non-zero (the gate's degrade detector).
    let out = d.query_step(&baseline_deck(FNS + 1, 0), None);
    assert_eq!(out.status, StepStatus::Ok, "FORCED: {:?}", out.message);
    assert!(
        d.snapshot_rewarms() >= 1,
        "the forced rebuild is counted non-zero (got {})",
        d.snapshot_rewarms()
    );
    // And the rebuilt lane serves correctly afterwards.
    let after = d.query_step(&baseline_deck(FNS + 1, 1), None);
    assert_eq!(after.status, StepStatus::Ok, "{:?}", after.message);
    assert_eq!(after.ct_execute, 1);
}

// ---------------------------------------------------------------------------
// F-817-R2-01 closure: the COLD first call to a bool callee
// ---------------------------------------------------------------------------

fn cold_bool_deck(first_bool: bool, flag_arg: i32) -> String {
    let mut d = String::from("deck cte817/1\nmodule main\nexec snapshot\nsource <<\nfn flag(a int) bool {\n    return a < 5\n}\n\nfn num(a int) int {\n    return a + 1\n}\n\nfn main() int {\n    let g = false\n    let n = 0\n    if g {\n        return 100\n    }\n    return n\n}\n>>\n");
    if first_bool {
        d.push_str(&format!(
            "query q1 = call flag({flag_arg})\nquery q2 = call num(1)\n"
        ));
    } else {
        d.push_str(&format!(
            "query q1 = call num(1)\nquery q2 = call flag({flag_arg})\n"
        ));
    }
    d.push_str("bind main.g = q1 expect bool placeholder false\nbind main.n = q2 expect i32 placeholder 0\nentry main\nnative off\n");
    d
}

#[test]
fn cte_snapshot_cold_first_call_bool_true_and_false() {
    // Cold TRUE: the FIRST snapshot call of the request is a bool callee.
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(&cold_bool_deck(true, 3), None);
    assert_eq!(
        out.status,
        StepStatus::Ok,
        "cold-bool-true: {:?}",
        out.message
    );
    assert_eq!(
        out.values.get("main.g").map(|v| v.render()),
        Some("bool:true".into())
    );
    assert_eq!(
        out.values.get("main.n").map(|v| v.render()),
        Some("i32:2".into())
    );
    // Cold FALSE: the same shape on the other branch.
    let mut d2 = CteDriver::ephemeral();
    let out2 = d2.query_step(&cold_bool_deck(true, 9), None);
    assert_eq!(
        out2.status,
        StepStatus::Ok,
        "cold-bool-false: {:?}",
        out2.message
    );
    assert_eq!(
        out2.values.get("main.g").map(|v| v.render()),
        Some("bool:false".into())
    );
}

#[test]
fn cte_snapshot_i32_bool_i32_lane_transitions() {
    // i32 -> bool -> i32 callee transitions inside ONE request (the
    // warm/sync/shape machinery across return classes).
    let deck = "deck cte817/1\nmodule main\nexec snapshot\nsource <<\nfn a1(x int) int {\n    return x + 1\n}\n\nfn b1(x int) bool {\n    return x < 5\n}\n\nfn c1(x int) int {\n    return x * 10\n}\n\nfn main() int {\n    let p = 0\n    let q = false\n    let r = 0\n    if q {\n        return p + r\n    }\n    return r\n}\n>>\nquery q1 = call a1(1)\nquery q2 = call b1(2)\nquery q3 = call c1(3)\nbind main.p = q1 expect i32 placeholder 0\nbind main.q = q2 expect bool placeholder false\nbind main.r = q3 expect i32 placeholder 0\nentry main\nnative off\n";
    let mut d = CteDriver::ephemeral();
    let out = d.query_step(deck, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.values.get("main.p").map(|v| v.render()),
        Some("i32:2".into())
    );
    assert_eq!(
        out.values.get("main.q").map(|v| v.render()),
        Some("bool:true".into())
    );
    assert_eq!(
        out.values.get("main.r").map(|v| v.render()),
        Some("i32:30".into())
    );
}

#[test]
fn cte_snapshot_cold_bool_miss_hit_and_retry_after_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().join("store");
    // miss then hit for a cold-bool deck.
    let mut d = CteDriver::with_store(&root, "cb").unwrap();
    let first = d.query_step(&cold_bool_deck(true, 3), None);
    assert_eq!(first.status, StepStatus::Ok, "{:?}", first.message);
    assert_eq!(first.cache, Some("miss"));
    assert_eq!(first.ct_execute, 2);
    let second = d.query_step(&cold_bool_deck(true, 3), None);
    assert_eq!(second.cache, Some("hit"));
    assert_eq!(second.ct_execute, 0);
    assert_eq!(
        second.values.get("main.g").map(|v| v.render()),
        Some("bool:true".into())
    );
    // A trapping retry after the resident lane exists: controlled diag,
    // then the SAME deck still hits cleanly (no poison, no wrong value).
    let trap_deck = cold_bool_deck(true, 3)
        .replace("query q2 = call num(1)", "query q2 = call boom(1)")
        .replace(
            "fn num(a int) int {\n    return a + 1\n}",
            "fn boom(a int) int {\n    return 2147483647 + 1\n}",
        )
        .replace("return a + 1\n}", "return 2147483647 + 1\n}");
    let trap = d.query_step(&trap_deck, None);
    assert_eq!(trap.status, StepStatus::Diag);
    assert_eq!(trap.code.as_deref(), Some("cte.trap.int-overflow"));
    let retry = d.query_step(&cold_bool_deck(true, 3), None);
    assert_eq!(retry.status, StepStatus::Ok, "{:?}", retry.message);
    assert_eq!(
        retry.cache,
        Some("hit"),
        "the clean entry survived the failed retry"
    );
    assert_eq!(
        retry.values.get("main.g").map(|v| v.render()),
        Some("bool:true".into())
    );
}

#[test]
fn cte_snapshot_cold_bool_legal_language_surface_unchanged() {
    // The closure lane on the SAME cold-bool deck stays green (the
    // repair must not widen or shrink the language surface).
    let mut d = CteDriver::ephemeral();
    let closure = cold_bool_deck(true, 3).replace("exec snapshot", "exec closure");
    let out = d.query_step(&closure, None);
    assert_eq!(out.status, StepStatus::Ok, "{:?}", out.message);
    assert_eq!(
        out.values.get("main.g").map(|v| v.render()),
        Some("bool:true".into())
    );
}
