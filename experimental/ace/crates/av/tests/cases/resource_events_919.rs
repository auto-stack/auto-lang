//! PLAN-919 T-02/T-03 (av cells of scenario R01/R04/R06): the per-owner
//! lifecycle channel through the REAL source→HIR→MIR→ABC→AV path.
//!
//! These are the executor-side feasibility and discrimination checks; the
//! formal parent-side oracle (strict admission + frozen invariants +
//! negative controls) lives in `auto-lang-core::test_cases::ac_av`
//! (`judge_resource_events`) and the `resource-events-919` suite.

use ace_av::resource_events::{
    compile_and_execute_observed, ResourceEventsKnobs, ResourceEventsRun, SourceProfile,
};
use ace_av_core::lifecycle::{Act, DisownVia, OwnVia, OwnerKind};

fn run(src: &str, knobs: ResourceEventsKnobs) -> ResourceEventsRun {
    compile_and_execute_observed(src, "main", SourceProfile::Collections, knobs)
        .unwrap_or_else(|e| panic!("exec failed: {e}"))
}

fn run_owned(src: &str, knobs: ResourceEventsKnobs) -> ResourceEventsRun {
    compile_and_execute_observed(src, "d_main", SourceProfile::Owned, knobs)
        .unwrap_or_else(|e| panic!("exec failed: {e}"))
}

/// The frozen executor-side invariants every observed run must satisfy
/// (mirrors the parent's rules; the parent re-derives them independently).
fn assert_stream_invariants(r: &ResourceEventsRun) {
    assert!(!r.overflowed, "bounded observation must not overflow here");
    // seq contiguous
    for (i, e) in r.events.iter().enumerate() {
        assert_eq!(e.seq as usize, i, "seq must be contiguous");
    }
    assert_eq!(
        r.events.first().map(|e| e.act),
        Some(Act::FrameEnter),
        "root frame-enter first"
    );
    // exactly one end; after it only free facts
    let ends: Vec<usize> = r
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| e.act == Act::End)
        .map(|(i, _)| i)
        .collect();
    assert_eq!(ends.len(), 1, "exactly one end marker");
    for e in &r.events[ends[0] + 1..] {
        assert_eq!(e.act, Act::Free, "only free facts after end");
    }
    // frame nesting: enter/leave balanced, LIFO
    let mut open: Vec<u64> = Vec::new();
    for e in &r.events {
        match e.act {
            Act::FrameEnter => open.push(e.frame),
            Act::FrameLeave => {
                let top = open.pop().expect("leave without enter");
                assert_eq!(top, e.frame, "frame leave must be the innermost open frame");
            }
            _ => {}
        }
    }
    assert!(open.is_empty(), "every frame must leave by the end");
    // slot discipline: per (frame, slot, gen) exactly one own then one disown
    use std::collections::BTreeMap;
    let mut slots: BTreeMap<(u64, u32, u64), (usize, usize)> = BTreeMap::new();
    for e in &r.events {
        match e.act {
            Act::Own => slots.entry((e.frame, e.slot, e.gen)).or_default().0 += 1,
            Act::Disown => slots.entry((e.frame, e.slot, e.gen)).or_default().1 += 1,
            _ => {}
        }
    }
    for (k, (owns, disowns)) in &slots {
        assert_eq!(*owns, 1, "slot {k:?} must own exactly once");
        assert_eq!(
            *disowns, 1,
            "slot {k:?} must disown exactly once (got {disowns})"
        );
    }
    // buffer discipline: each buffer freed at most once; every allocated id freed by the end
    let mut freed: BTreeMap<u64, usize> = BTreeMap::new();
    for e in r.events.iter().filter(|e| e.act == Act::Free) {
        *freed.entry(e.id).or_default() += 1;
    }
    for (id, n) in &freed {
        assert_eq!(*n, 1, "buffer {id} freed exactly once");
    }
    for (id, _) in &r.allocs {
        assert!(
            freed.contains_key(id),
            "allocated buffer {id} must be freed by the end"
        );
    }
    assert_eq!(
        r.live_after_result_drop, 0,
        "nothing alive after the result dropped"
    );
    // the arena's own ledger and the stream's free facts agree in order
    let stream_frees: Vec<u64> = r
        .events
        .iter()
        .filter(|e| e.act == Act::Free)
        .map(|e| e.id)
        .collect();
    let arena_frees: Vec<u64> = r.frees.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        stream_frees, arena_frees,
        "stream free order == arena ledger order"
    );
    // LIFO: inside one frame-leave sweep the swept disowns descend by key
    let mut last_key: Option<(u64, u64)> = None; // (frame, key)
    for e in &r.events {
        match e.act {
            Act::Disown
                if e.via_disowned == Some(DisownVia::Swept)
                    && e.kind != Some(OwnerKind::ClosureEnv) =>
            {
                if let Some((f, k)) = last_key {
                    if f == e.frame {
                        assert!(e.key <= k, "sweep must release in descending creation key (frame {f}: {k} then {})", e.key);
                    }
                }
                last_key = Some((e.frame, e.key));
            }
            Act::FrameLeave => last_key = None,
            _ => {}
        }
    }
}

/// R01 (owned profile): the T03/R04/R05 resource shapes in one program —
/// clone/move/concat (T03), record field re-assignment (R04), a `move`
/// parameter relay across frames (R05) — plus an early-return branch with
/// a temporary that dies with the callee frame.
const R01_SCOPES: &str = r#"
type Item { name Str; code int }
fn relay(move x Item) Item { return x.move }
fn pick(move s Str) int {
    let t Str = "tail"
    if s.len() > 2 {
        return s.len() + t.len()
    }
    let u Str = "u"
    return s.len() + u.len()
}
fn main() int {
    let a Str = "ab"
    let b Str = a.clone()
    var c Str = b.move
    c = c + "中"
    var it Item = Item("hello", 1)
    let keep Item = it.clone()
    it.name = "x"
    let got Item = relay(keep.move)
    let early = pick(c.move)
    let d Str = a.clone()
    let late = pick(d.move)
    return early * 100 + late * 10 + got.code + it.name.len()
}
"#;

#[test]
fn r01_scopes_emit_per_owner_lifo_events() {
    let r = run_owned(R01_SCOPES, ResourceEventsKnobs::default());
    assert_eq!(r.outcome, "success", "{:?}", r.infra);
    // c = "ab" + "中" → 3 scalars; early = pick(c) = 3 + 4 = 7 (early return);
    // late = pick("ab") = 2 + 1 = 3; got.code = 1; it.name = "x" → 1
    assert_eq!(r.value, Some(7 * 100 + 3 * 10 + 1 + 1));
    assert_stream_invariants(&r);
    // four frames (main + relay + pick + pick), all left on the return path
    let enters = r.events.iter().filter(|e| e.act == Act::FrameEnter).count();
    assert_eq!(enters, 4);
    assert!(
        r.events
            .iter()
            .filter(|e| e.act == Act::FrameLeave)
            .all(|e| !e.sweep),
        "return-path leaves only"
    );
    // the argument transfer: main's `a` moved out, inner's param received the same buffer id
    let moved: Vec<&_> = r
        .events
        .iter()
        .filter(|e| e.act == Act::Disown && e.via_disowned == Some(DisownVia::MovedOut))
        .collect();
    assert!(!moved.is_empty(), "the call argument must move out");
    let received: Vec<&_> = r
        .events
        .iter()
        .filter(|e| e.act == Act::Own && e.via_owned == Some(OwnVia::Received))
        .collect();
    assert!(
        received
            .iter()
            .any(|o| moved.iter().any(|m| m.id == o.id && m.kind == o.kind)),
        "a received own must match a moved-out id"
    );
    // every owned Str/List was created exactly once
    let created: Vec<u64> = r
        .events
        .iter()
        .filter(|e| {
            e.act == Act::Own
                && e.via_owned == Some(OwnVia::Created)
                && matches!(e.kind, Some(OwnerKind::Str) | Some(OwnerKind::List))
        })
        .map(|e| e.id)
        .collect();
    let mut dedup = created.clone();
    dedup.sort_unstable();
    dedup.dedup();
    assert_eq!(created.len(), dedup.len(), "a buffer is created once");
}

const R04_V06: &str = r#"
fn main() int {
    var r Result<int, Str> = Err("old")
    r = Ok(42)
    is r {
        Ok(v) -> { return v }
        Err(e) -> { return e.len() }
    }
}
"#;

#[test]
fn r04_variant_switch_releases_old_arm_exactly_once() {
    let r = run(R04_V06, ResourceEventsKnobs::default());
    assert_eq!(r.outcome, "success", "{:?}", r.infra);
    assert_eq!(r.value, Some(42));
    assert_stream_invariants(&r);
    // exactly one buffer ("old", 3 bytes) allocated and freed once
    assert_eq!(r.allocs.len(), 1, "one owned buffer: \"old\"");
    assert_eq!(r.allocs[0].1, 3);
    let old_id = r.allocs[0].0;
    let frees: Vec<&_> = r
        .events
        .iter()
        .filter(|e| e.act == Act::Free && e.id == old_id)
        .collect();
    assert_eq!(frees.len(), 1, "old arm payload freed exactly once");
    // the free happens at the switch: a `replaced` disown of the variant
    // slot precedes it, and the new arm's own follows it, all before `end`
    let free_seq = frees[0].seq;
    let replaced = r
        .events
        .iter()
        .find(|e| {
            e.act == Act::Disown
                && e.via_disowned == Some(DisownVia::Replaced)
                && e.kind == Some(OwnerKind::Variant)
        })
        .expect("the old live arm is replaced at the switch");
    assert!(
        replaced.seq < free_seq,
        "replaced disown precedes the payload free"
    );
    let new_own = r
        .events
        .iter()
        .find(|e| e.act == Act::Own && e.kind == Some(OwnerKind::Variant) && e.seq > free_seq)
        .expect("the new arm is owned after the old payload freed");
    let end_seq = r.events.iter().find(|e| e.act == Act::End).unwrap().seq;
    assert!(new_own.seq < end_seq);
    // inactive payload access = 0: nothing owns/receives the old buffer after its free
    assert!(r.events.iter().all(|e| !(e.seq > free_seq
        && e.act == Act::Own
        && e.id == old_id
        && e.kind == Some(OwnerKind::Str))));
}

const R06_DEEP: &str = r#"
fn main() int {
    var seed = List<int>.new()
    seed.push(1)
    seed.push(2)
    seed.push(3)
    seed.push(4)
    seed.push(5)
    seed.push(6)
    seed.push(7)
    seed.push(8)
    var xs = List<Str>.new()
    for a in seed {
        for b in seed {
            xs.push("payload")
        }
    }
    var n = 0
    for a in seed {
        for b in seed {
            for c in seed {
                for d in seed {
                    for e in seed {
                        for f in seed {
                            n = n + a + f
                        }
                    }
                }
            }
        }
    }
    return n + xs.len()
}
"#;

#[test]
fn r06_budget_exhaustion_sweeps_every_owner() {
    let r = run(
        R06_DEEP,
        ResourceEventsKnobs {
            max_steps: Some(5000),
            ..Default::default()
        },
    );
    assert_eq!(r.outcome, "resource");
    assert_eq!(r.resource.as_deref(), Some("av.resource.instr-steps"));
    assert_stream_invariants(&r);
    // the unwind path: the root frame leaves by SWEEP, and the live list's
    // 64 payload buffers all free in the sweep (reverse push order)
    assert!(r.events.iter().any(|e| e.act == Act::FrameLeave && e.sweep));
    assert!(
        r.allocs.len() >= 65,
        "seed/xs lists + 64 payload buffers alive at the budget exit"
    );
    assert_eq!(
        r.live_at_end, 0,
        "budget exit must release every owner (no OS reclamation)"
    );
}

#[test]
fn r06_cancellation_releases_every_owner() {
    let r = run(
        R06_DEEP,
        ResourceEventsKnobs {
            max_steps: Some(u64::MAX / 4),
            wall_ms: Some(30_000),
            cancel_after_ms: Some(5),
            ..Default::default()
        },
    );
    // The program spins 8^6 scalar iterations (millions of steps, no
    // allocations — the bounded event log stays small) while 64 owned
    // payloads are alive, so the poll observes the flag; the frozen
    // cancel contract promises the NEXT poll, not instruction-level
    // interruption.
    assert_eq!(
        r.outcome, "cancelled",
        "resource={:?} infra={:?}",
        r.resource, r.infra
    );
    assert_stream_invariants(&r);
    assert!(r.events.iter().any(|e| e.act == Act::FrameLeave && e.sweep));
    assert_eq!(r.live_at_end, 0);
}

const R06_TRAP: &str = r#"
fn boom(s Str, d int) int {
    let t Str = "inner"
    return (s.len() + t.len()) / d
}

fn main() int {
    var xs = List<Str>.new()
    xs.push("a")
    xs.push("bb")
    let keep Str = "kept"
    return boom(keep, 0) + xs.len()
}
"#;

#[test]
fn r06_trap_inside_callee_sweeps_both_frames() {
    let r = run(R06_TRAP, ResourceEventsKnobs::default());
    assert_eq!(r.outcome, "trap");
    assert_eq!(r.trap.as_deref(), Some("av.trap.div-by-zero"));
    assert_stream_invariants(&r);
    let sweeps = r
        .events
        .iter()
        .filter(|e| e.act == Act::FrameLeave && e.sweep)
        .count();
    assert_eq!(sweeps, 2, "both frames unwind by sweep (callee first)");
    let leaves: Vec<u64> = r
        .events
        .iter()
        .filter(|e| e.act == Act::FrameLeave)
        .map(|e| e.frame)
        .collect();
    assert!(
        leaves[0] > leaves[1],
        "innermost (callee) frame leaves first"
    );
    assert_eq!(r.live_at_end, 0);
}

#[test]
fn t06_observation_budget_and_worst_case_bounds() {
    // PLAN-919 §5.2: observation is bounded per operation — the 16384-event
    // cap keeps even the worst stream far below the 4 MiB budget, and the
    // native channel's 8192 × ≤256-byte lines stay below the 1 MiB stderr
    // cap. This pins the AV side: the deep program (largest suite stream)
    // plus the budget-exit sweep stays well inside the cap.
    let r = run(
        R06_DEEP,
        ResourceEventsKnobs {
            max_steps: Some(5000),
            ..Default::default()
        },
    );
    assert!(r.events.len() <= ace_av_core::lifecycle::MAX_EVENTS);
    // the serialized form the worker reports (compact JSON) is bounded by
    // ~96 B/event + envelopes — assert the concrete worst case observed
    // here stays an order of magnitude below 4 MiB.
    let worst_bytes = r.events.len() * 96;
    assert!(
        worst_bytes + 4096 < 4 * 1024 * 1024,
        "event stream budget exceeded: {worst_bytes}"
    );
    // the deepest suite stream's steps stay far below the 1M fuel cap and
    // the run finishes well inside the 2s wall budget.
    assert!(r.wall_ms < 2000, "wall budget exceeded: {} ms", r.wall_ms);
}

#[test]
fn observation_off_keeps_language_result_and_emits_nothing() {
    // The same programs through the un-observed entry give identical
    // values and never produce lifecycle events.
    let plain = ace_av::collections_864_support::compile_and_execute(R04_V06, "main").unwrap();
    assert_eq!(plain.value, 42);
    assert_eq!(plain.live_after, 0);
    let observed = run(R04_V06, ResourceEventsKnobs::default());
    assert_eq!(observed.value, Some(plain.value));
    assert_eq!(observed.allocs.len(), plain.allocs);
    assert_eq!(observed.frees.len(), plain.frees);
}

#[test]
fn zz_dump_oracle_rows() {
    let rows: &[(&str, ResourceEventsRun)] = &[
        (
            "r01-owned",
            run_owned(R01_SCOPES, ResourceEventsKnobs::default()),
        ),
        (
            "r04-collections",
            run(R04_V06, ResourceEventsKnobs::default()),
        ),
        (
            "r06-budget",
            run(
                R06_DEEP,
                ResourceEventsKnobs {
                    max_steps: Some(5000),
                    ..Default::default()
                },
            ),
        ),
        (
            "r06-cancel",
            run(
                R06_DEEP,
                ResourceEventsKnobs {
                    max_steps: Some(u64::MAX / 4),
                    wall_ms: Some(30_000),
                    cancel_after_ms: Some(5),
                    ..Default::default()
                },
            ),
        ),
        ("r06-trap", run(R06_TRAP, ResourceEventsKnobs::default())),
    ];
    for (name, r) in rows {
        use std::collections::BTreeMap;
        let mut created: BTreeMap<&str, usize> = BTreeMap::new();
        for e in &r.events {
            if e.act == Act::Own && e.via_owned == Some(OwnVia::Created) {
                if let Some(k) = e.kind {
                    *created.entry(k.wire()).or_default() += 1;
                }
            }
        }
        let free_order: Vec<u64> = r
            .events
            .iter()
            .filter(|e| e.act == Act::Free)
            .map(|e| e.id)
            .collect();
        let frames = r.events.iter().filter(|e| e.act == Act::FrameEnter).count();
        eprintln!(
            "DUMP {name}: outcome={} value={:?} trap={:?} resource={:?} frames={} created={:?} free_order={:?} live_after={} allocs={:?}",
            r.outcome, r.value, r.trap, r.resource, frames, created, free_order, r.live_after_result_drop, r.allocs
        );
    }
}
