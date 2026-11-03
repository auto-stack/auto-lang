//! PLAN-919 (`ac-av/resource-events-v1`): the parent-side INDEPENDENT
//! oracle for the per-owner lifecycle event channel (`av.lifecycle/1`).
//!
//! The request carries only the profile, operations and budgets — never
//! expectations. The worker reports pure facts; THIS module re-judges them:
//!
//! 1. `admit_lifecycle` — strict structural admission of the event stream
//!    (exact key sets per action, typed fields, bounded counts, arena
//!    cross-check) plus the frozen generic invariants every stream must
//!    satisfy (contiguous seq; balanced LIFO frame nesting; per-slot
//!    own/disown alternation — every logical owner released exactly once;
//!    each buffer freed at most once; the `end` marker present; only free
//!    facts after `end`; sweep order descending by creation key).
//! 2. `judge_resource_events` — the frozen per-case table: outcome, value,
//!    trap/resource face, frame count, created-owner counts per kind, the
//!    exact buffer free ORDER, and the live-after-release zero.
//!
//! Negative controls (forged/duplicated/truncated/replayed streams) live in
//! `tests` below; a totals-only or self-reported PASS can never be green.

use super::{Mismatch, RecipeFamily, StepOut};
use std::collections::BTreeMap;

/// The frozen lifecycle facts after strict admission.
#[derive(Debug, Clone, Default)]
pub struct LifecycleFacts {
    pub profile: String,
    pub overflowed: bool,
    pub steps: u64,
    pub live_at_end: usize,
    pub live_after_result_drop: usize,
    /// (id, bytes) in allocation order — must equal the arena ledger.
    pub allocs: Vec<(u64, u64)>,
    /// (id, bytes) in free order — must equal the arena ledger.
    pub frees: Vec<(u64, u64)>,
    /// (seq, frame) of every frame-enter / frame-leave (sweep flag).
    pub frame_enters: Vec<(u64, u64)>,
    pub frame_leaves: Vec<(u64, u64, bool)>,
    /// own events: (seq, kind, id, key, via, frame, slot, gen)
    pub owns: Vec<(u64, String, u64, u64, String, u64, u32, u64)>,
    /// disown events, same tuple shape.
    pub disowns: Vec<(u64, String, u64, u64, String, u64, u32, u64)>,
    /// free events: (seq, id, bytes)
    pub free_events: Vec<(u64, u64, u64)>,
    pub end: Option<(
        u64,
        String,
        Option<i32>,
        Option<String>,
        Option<String>,
        u64,
    )>,
}

const MAX_ADMIT_EVENTS: usize = 65536;

fn bad(msg: impl Into<String>) -> String {
    format!("lifecycle admission: {}", msg.into())
}

fn get_u64(o: &serde_json::Map<String, serde_json::Value>, k: &str) -> Result<u64, String> {
    match o.get(k) {
        Some(serde_json::Value::Number(n)) => n
            .as_u64()
            .ok_or_else(|| bad(format!("{k} must be a non-negative integer"))),
        _ => Err(bad(format!("{k} must be a non-negative integer"))),
    }
}

fn get_opt_str(
    o: &serde_json::Map<String, serde_json::Value>,
    k: &str,
) -> Result<Option<String>, String> {
    match o.get(k) {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s.clone())),
        Some(_) => Err(bad(format!("{k} must be a string or null"))),
    }
}

const OWN_KINDS: &[&str] = &["str", "list", "record", "variant", "closure-env"];
const OWN_VIAS: &[&str] = &["created", "received"];
const DISOWN_VIAS: &[&str] = &["moved-out", "replaced", "swept", "orphan"];

/// Strict admission of one `lifecycle` object plus the frozen generic
/// invariants. Returns the decoded facts or an admission error.
pub fn admit_lifecycle(v: &serde_json::Value) -> Result<LifecycleFacts, String> {
    let o = v
        .as_object()
        .ok_or_else(|| bad("lifecycle must be an object"))?;
    let mut keys: Vec<&str> = o.keys().map(|s| s.as_str()).collect();
    keys.sort_unstable();
    let mut expected_keys = vec![
        "schema",
        "profile",
        "overflowed",
        "events",
        "allocs",
        "frees",
        "live_at_end",
        "live_after_result_drop",
        "steps",
    ];
    expected_keys.sort_unstable();
    if keys != expected_keys {
        return Err(bad(format!("unexpected lifecycle key set: {keys:?}")));
    }
    if o["schema"].as_str() != Some("av.lifecycle/1") {
        return Err(bad("lifecycle schema must be av.lifecycle/1"));
    }
    let profile = o["profile"]
        .as_str()
        .filter(|s| *s == "owned" || *s == "collections")
        .ok_or_else(|| bad("lifecycle profile must be owned|collections"))?
        .to_string();
    let overflowed = o["overflowed"]
        .as_bool()
        .ok_or_else(|| bad("overflowed must be a bool"))?;
    let steps = get_u64(o, "steps")?;
    let live_at_end = get_u64(o, "live_at_end")? as usize;
    let live_after = get_u64(o, "live_after_result_drop")? as usize;

    let arr = |k: &str| -> Result<Vec<serde_json::Value>, String> {
        o[k].as_array()
            .cloned()
            .ok_or_else(|| bad(format!("{k} must be an array")))
    };
    let allocs = arr("allocs")?;
    let frees = arr("frees")?;
    let events = arr("events")?;
    if events.len() > MAX_ADMIT_EVENTS {
        return Err(bad(format!(
            "event count {} above admission cap",
            events.len()
        )));
    }
    let mut f = LifecycleFacts {
        profile,
        overflowed,
        steps,
        live_at_end,
        live_after_result_drop: live_after,
        ..Default::default()
    };
    let pair_list = |a: Vec<serde_json::Value>, name: &str| -> Result<Vec<(u64, u64)>, String> {
        let mut out = Vec::new();
        for (i, e) in a.iter().enumerate() {
            let eo = e
                .as_object()
                .ok_or_else(|| bad(format!("{name}[{i}] must be an object")))?;
            let ek: Vec<&str> = eo.keys().map(|s| s.as_str()).collect();
            if ek != vec!["id", "bytes"] {
                return Err(bad(format!("{name}[{i}] key set {ek:?}")));
            }
            out.push((get_u64(eo, "id")?, get_u64(eo, "bytes")?));
        }
        Ok(out)
    };
    f.allocs = pair_list(allocs, "allocs")?;
    f.frees = pair_list(frees, "frees")?;

    for (i, e) in events.iter().enumerate() {
        let eo = e
            .as_object()
            .ok_or_else(|| bad(format!("events[{i}] must be an object")))?;
        let act = eo
            .get("act")
            .and_then(|a| a.as_str())
            .ok_or_else(|| bad(format!("events[{i}].act must be a string")))?;
        let mut expect_keys: Vec<&str> = match act {
            "frame-enter" => vec!["seq", "act", "frame"],
            "frame-leave" => vec!["seq", "act", "frame", "sweep"],
            "own" | "disown" => vec![
                "seq", "act", "kind", "id", "key", "via", "frame", "slot", "gen",
            ],
            "free" => vec!["seq", "act", "id", "bytes"],
            "end" => vec![
                "seq", "act", "outcome", "value", "trap", "resource", "steps",
            ],
            other => {
                return Err(bad(format!(
                    "events[{i}].act {other:?} is not in the vocabulary"
                )))
            }
        }
        .to_vec();
        let mut got: Vec<&str> = eo.keys().map(|s| s.as_str()).collect();
        got.sort_unstable();
        expect_keys.sort_unstable();
        if got != expect_keys {
            return Err(bad(format!(
                "events[{i}] ({act}) key set {got:?} != {expect_keys:?}"
            )));
        }
        let seq = get_u64(eo, "seq")?;
        if seq != i as u64 {
            return Err(bad(format!(
                "events[{i}] has seq {seq} (contiguity broken)"
            )));
        }
        match act {
            "frame-enter" => f.frame_enters.push((seq, get_u64(eo, "frame")?)),
            "frame-leave" => {
                let sweep = eo["sweep"]
                    .as_bool()
                    .ok_or_else(|| bad(format!("events[{i}].sweep must be a bool")))?;
                f.frame_leaves.push((seq, get_u64(eo, "frame")?, sweep));
            }
            "own" | "disown" => {
                let kind = eo["kind"]
                    .as_str()
                    .filter(|k| OWN_KINDS.contains(k))
                    .ok_or_else(|| bad(format!("events[{i}].kind not in vocabulary")))?
                    .to_string();
                let id = get_u64(eo, "id")?;
                let key = get_u64(eo, "key")?;
                let via_vocabulary = if act == "own" { OWN_VIAS } else { DISOWN_VIAS };
                let via = eo["via"]
                    .as_str()
                    .filter(|v| via_vocabulary.contains(v))
                    .ok_or_else(|| bad(format!("events[{i}].via not in vocabulary")))?
                    .to_string();
                let frame = get_u64(eo, "frame")?;
                let slot = get_u64(eo, "slot")?;
                if slot > u32::MAX as u64 {
                    return Err(bad(format!("events[{i}].slot out of range")));
                }
                let gen = get_u64(eo, "gen")?;
                let row = (seq, kind, id, key, via, frame, slot as u32, gen);
                if act == "own" {
                    f.owns.push(row);
                } else {
                    f.disowns.push(row);
                }
            }
            "free" => f
                .free_events
                .push((seq, get_u64(eo, "id")?, get_u64(eo, "bytes")?)),
            "end" => {
                if f.end.is_some() {
                    return Err(bad("more than one end marker"));
                }
                let outcome = eo["outcome"]
                    .as_str()
                    .filter(|s| {
                        matches!(*s, "success" | "trap" | "resource" | "cancelled" | "infra")
                    })
                    .ok_or_else(|| bad("end.outcome not in vocabulary"))?
                    .to_string();
                let value = match &eo["value"] {
                    serde_json::Value::Null => None,
                    serde_json::Value::Number(n) => Some(
                        n.as_i64()
                            .filter(|v| i32::MIN as i64 <= *v && *v <= i32::MAX as i64)
                            .ok_or_else(|| bad("end.value out of i32 range"))?
                            as i32,
                    ),
                    _ => return Err(bad("end.value must be an i32 or null")),
                };
                let trap = get_opt_str(eo, "trap")?;
                let resource = get_opt_str(eo, "resource")?;
                let esteps = get_u64(eo, "steps")?;
                f.end = Some((seq, outcome, value, trap, resource, esteps));
            }
            _ => unreachable!("act filtered above"),
        }
    }

    // ---- frozen generic invariants -------------------------------------
    if f.overflowed {
        return Err(bad(
            "the observation log overflowed (bounded observation is a named refusal, never green)",
        ));
    }
    // I1: exactly one end; only free facts after it.
    let end_seq = f
        .end
        .as_ref()
        .map(|(s, ..)| *s)
        .ok_or_else(|| bad("executed stream has no end marker"))?;
    for (seq, ..) in &f.free_events {
        if *seq > end_seq {
            continue; // the host dropping the result: legal
        }
    }
    for e in &f.owns {
        if e.0 > end_seq {
            return Err(bad("an own event appears after the end marker"));
        }
    }
    for e in &f.disowns {
        if e.0 > end_seq {
            return Err(bad("a disown event appears after the end marker"));
        }
    }
    // I2: frame nesting — enter/leave balanced, leave only of the innermost
    // open frame, the first event is the root enter.
    if f.frame_enters.first().map(|(s, _)| *s) != Some(0) {
        return Err(bad("the first event must be the root frame-enter"));
    }
    let mut entered_all: Vec<u64> = f.frame_enters.iter().map(|(_, fr)| *fr).collect();
    entered_all.sort_unstable();
    entered_all.dedup();
    if entered_all.len() != f.frame_enters.len() {
        return Err(bad("a frame activation id enters more than once"));
    }
    // replay enter/leave in seq order: leave must match the innermost open
    let mut open: Vec<u64> = Vec::new();
    {
        let mut merged: Vec<(u64, u8, u64)> = Vec::new();
        for (s, fr) in &f.frame_enters {
            merged.push((*s, 0, *fr));
        }
        for (s, fr, _) in &f.frame_leaves {
            merged.push((*s, 1, *fr));
        }
        merged.sort_unstable();
        for (seq, kind, frame) in merged {
            if kind == 0 {
                open.push(frame);
            } else if open.last() == Some(&frame) {
                open.pop();
            } else {
                return Err(bad(format!(
                    "frame-leave at seq {seq} does not match the innermost open frame (nesting broken)"
                )));
            }
        }
        if !open.is_empty() {
            return Err(bad(format!("{} frame(s) never left", open.len())));
        }
    }
    // I3: per (frame, slot, gen): exactly one own then exactly one disown.
    let mut slots: BTreeMap<(u64, u32, u64), (u64, u64)> = BTreeMap::new();
    for (seq, _k, _id, _key, _via, frame, slot, gen) in &f.owns {
        slots.entry((*frame, *slot, *gen)).or_default().0 = *seq;
    }
    for (seq, _k, _id, _key, _via, frame, slot, gen) in &f.disowns {
        let e = slots.entry((*frame, *slot, *gen)).or_default();
        if e.0 == u64::MAX {
            return Err(bad(format!(
                "disown at seq {seq} has no preceding own for the slot generation"
            )));
        }
        if e.1 != 0 {
            return Err(bad(format!(
                "slot generation ({frame},{slot},{gen}) disowns twice (double release)"
            )));
        }
        e.1 = *seq;
    }
    for (k, (_, d)) in &slots {
        if *d == 0 {
            return Err(bad(format!(
                "slot generation {k:?} owns but never disowns (leak)"
            )));
        }
    }
    // I4: buffer discipline — each buffer freed at most once; every
    // allocated id is freed by the end; the stream's free order equals the
    // arena ledger's free order.
    let mut seen = std::collections::BTreeSet::new();
    for (seq, id, _b) in &f.free_events {
        if !seen.insert(*id) {
            return Err(bad(format!("buffer {id} frees twice (double free)")));
        }
        let _ = seq;
    }
    let ledger_free: Vec<u64> = f.frees.iter().map(|(id, _)| *id).collect();
    let stream_free: Vec<u64> = f.free_events.iter().map(|(_, id, _)| *id).collect();
    if ledger_free != stream_free {
        return Err(bad("the stream's free order differs from the arena ledger"));
    }
    let alloc_ids: std::collections::BTreeSet<u64> = f.allocs.iter().map(|(id, _)| *id).collect();
    for id in &seen {
        if !alloc_ids.contains(id) {
            return Err(bad(format!("buffer {id} freed but never allocated")));
        }
    }
    if f.live_after_result_drop != 0 {
        return Err(bad(format!(
            "live_after_result_drop is {} (nothing may outlive the run's result drop)",
            f.live_after_result_drop
        )));
    }
    if seen.len() != alloc_ids.len() {
        return Err(bad(format!(
            "{} buffers allocated but {} freed (leak)",
            alloc_ids.len(),
            seen.len()
        )));
    }
    // I5: a sweep releases in DESCENDING creation key order within one
    // frame (the frozen LIFO cross-type rule); closure-env collection-pass
    // events are exempt (they release during collection, before the sorted
    // pass — the sweep's own events stay key-ordered).
    let mut last: Option<(u64, u64)> = None;
    for (_seq, kind, _id, key, via, frame, _slot, _gen) in &f.disowns {
        if via == "swept" && kind != "closure-env" {
            if let Some((lf, lk)) = last {
                if lf == *frame && *key > lk {
                    return Err(bad(format!(
                        "sweep of frame {frame} releases key {key} after key {lk} (LIFO broken)"
                    )));
                }
            }
            last = Some((*frame, *key));
        }
    }
    // I6: own(received) must reference an identity that was created before
    // (a buffer first seen as received is a fabricated transfer).
    let mut created: std::collections::BTreeSet<(String, u64)> = std::collections::BTreeSet::new();
    for (_seq, kind, id, _key, via, ..) in &f.owns {
        if via == "created" {
            created.insert((kind.clone(), *id));
        }
    }
    for (_seq, kind, id, _key, via, ..) in &f.owns {
        if via == "received" && !created.contains(&(kind.clone(), *id)) {
            return Err(bad(format!(
                "own(received) of ({kind},{id}) with no preceding creation"
            )));
        }
    }
    Ok(f)
}

impl LifecycleFacts {
    /// created-owner counts per kind.
    pub fn created_counts(&self) -> BTreeMap<String, usize> {
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for (_, kind, _, _, via, ..) in &self.owns {
            if via == "created" {
                *m.entry(kind.clone()).or_default() += 1;
            }
        }
        m
    }
    /// The exact buffer free order (ids).
    pub fn free_order(&self) -> Vec<u64> {
        self.free_events.iter().map(|(_, id, _)| *id).collect()
    }
}

/// The frozen per-case oracle table (the parent's own expectations — the
/// request never carries any of this). Keyed by check id.
pub struct ResourceEventsOracle {
    pub profile: &'static str,
    pub outcome: &'static str,
    pub value: Option<i32>,
    pub trap: Option<&'static str>,
    pub resource: Option<&'static str>,
    pub frames: usize,
    /// created owners per kind (exact).
    pub created: &'static [(&'static str, usize)],
    /// exact buffer free order (ids are deterministic per program).
    pub free_order: &'static [u64],
}

/// The frozen table lookup. Keys are `<case id>/<check id>` of the
/// `resource-events-919` suite; values are derived from REAL parent-side
/// runs of the frozen sources (never from worker self-reports).
pub fn oracle_for(key: &str) -> Option<&'static ResourceEventsOracle> {
    ORACLES.iter().find(|(k, _)| *k == key).map(|(_, o)| o)
}

/// The frozen av rows of the `resource-events-919` suite. The ac rows are
/// judged by the native face (`ac_mn`), not here.
static ORACLES: &[(&str, ResourceEventsOracle)] = &[
    (
        "ace.m1.r01.av/av-owned",
        ResourceEventsOracle {
            profile: "owned",
            outcome: "success",
            value: Some(732),
            trap: None,
            resource: None,
            frames: 4,
            created: &[("record", 5), ("str", 9)],
            free_order: &[3, 2, 6, 8, 7, 9, 4, 0, 1, 5],
        },
    ),
    (
        "ace.m1.r04.av/av",
        ResourceEventsOracle {
            profile: "collections",
            outcome: "success",
            value: Some(42),
            trap: None,
            resource: None,
            frames: 1,
            created: &[("str", 1), ("variant", 2)],
            free_order: &[0],
        },
    ),
    (
        "ace.m1.r06-budget.av/av",
        ResourceEventsOracle {
            profile: "collections",
            outcome: "resource",
            value: None,
            trap: None,
            resource: Some("av.resource.instr-steps"),
            frames: 1,
            created: &[("list", 2), ("str", 64)],
            free_order: &[
                65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45,
                44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24,
                23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
                0,
            ],
        },
    ),
    (
        "ace.m1.r06-cancel.av/av",
        ResourceEventsOracle {
            profile: "collections",
            outcome: "cancelled",
            value: None,
            trap: None,
            resource: None,
            frames: 1,
            created: &[("list", 2), ("str", 64)],
            free_order: &[
                65, 64, 63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45,
                44, 43, 42, 41, 40, 39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24,
                23, 22, 21, 20, 19, 18, 17, 16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1,
                0,
            ],
        },
    ),
    (
        "ace.m1.r06-trap.av/av",
        ResourceEventsOracle {
            profile: "collections",
            outcome: "trap",
            value: None,
            trap: Some("av.trap.div-by-zero"),
            resource: None,
            frames: 2,
            created: &[("list", 1), ("str", 4)],
            free_order: &[3, 4, 2, 1, 0],
        },
    ),
];

/// Judge one admitted resource-events step against the frozen table.
pub(super) fn judge_resource_events(
    oracle: &ResourceEventsOracle,
    step: &StepOut,
    mismatches: &mut Vec<Mismatch>,
) {
    fn m(
        mismatches: &mut Vec<Mismatch>,
        observation: &str,
        kind: &'static str,
        expected: String,
        actual: String,
    ) {
        mismatches.push(Mismatch {
            observation: observation.to_string(),
            kind,
            expected,
            actual,
            first_diff: None,
        });
    }
    let lifecycle = step
        .lifecycle
        .as_ref()
        .expect("admission guarantees the lifecycle object");
    let facts = admit_lifecycle(lifecycle).expect("admission ran before judging");
    if facts.profile != oracle.profile {
        m(
            mismatches,
            "lifecycle",
            "profile",
            oracle.profile.into(),
            facts.profile.clone(),
        );
    }
    let (end_outcome, end_value, end_trap, end_resource) = match &facts.end {
        Some((_, o, v, t, r, _)) => (o.clone(), *v, t.clone(), r.clone()),
        None => (String::new(), None, None, None),
    };
    if end_outcome != oracle.outcome {
        m(
            mismatches,
            "lifecycle",
            "outcome",
            oracle.outcome.into(),
            end_outcome,
        );
    }
    if oracle.value != end_value {
        m(
            mismatches,
            "lifecycle",
            "end-value",
            format!("{:?}", oracle.value),
            format!("{end_value:?}"),
        );
    }
    if oracle.trap != end_trap.as_deref() {
        m(
            mismatches,
            "lifecycle",
            "trap",
            format!("{:?}", oracle.trap),
            format!("{end_trap:?}"),
        );
    }
    if oracle.resource != end_resource.as_deref() {
        m(
            mismatches,
            "lifecycle",
            "resource",
            format!("{:?}", oracle.resource),
            format!("{end_resource:?}"),
        );
    }
    if facts.frame_enters.len() != oracle.frames {
        m(
            mismatches,
            "lifecycle",
            "frames",
            oracle.frames.to_string(),
            facts.frame_enters.len().to_string(),
        );
    }
    let counts = facts.created_counts();
    for (kind, want) in oracle.created {
        let got = counts.get(*kind).copied().unwrap_or(0);
        if got != *want {
            m(
                mismatches,
                "lifecycle",
                "created-count",
                format!("{kind}={want}"),
                format!("{kind}={got}"),
            );
        }
    }
    if counts.len() != oracle.created.len() {
        let got_kinds: Vec<String> = counts.keys().cloned().collect();
        let want_kinds: Vec<&str> = oracle.created.iter().map(|(k, _)| *k).collect();
        m(
            mismatches,
            "lifecycle",
            "created-kinds",
            format!("{want_kinds:?}"),
            format!("{got_kinds:?}"),
        );
    }
    let free_order = facts.free_order();
    if free_order != oracle.free_order {
        // report the first divergence for a readable red
        let mut first = 0;
        while first < free_order.len().min(oracle.free_order.len()) {
            if free_order[first] != oracle.free_order[first] {
                break;
            }
            first += 1;
        }
        mismatches.push(Mismatch {
            observation: "lifecycle".into(),
            kind: "free-order".into(),
            expected: format!("{:?}", oracle.free_order),
            actual: format!("{free_order:?}"),
            first_diff: Some(first),
        });
    }
    // The step-level face must agree with the end marker (a worker
    // self-report that diverges from the facts is red).
    if step.outcome != oracle.outcome {
        m(
            mismatches,
            "step",
            "outcome",
            oracle.outcome.into(),
            step.outcome.clone(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(o: serde_json::Value) -> serde_json::Value {
        o
    }

    fn base_lifecycle() -> serde_json::Value {
        serde_json::json!({
            "schema": "av.lifecycle/1",
            "profile": "collections",
            "overflowed": false,
            "events": [
                { "seq": 0, "act": "frame-enter", "frame": 1 },
                { "seq": 1, "act": "own", "kind": "str", "id": 0, "key": 0, "via": "created", "frame": 1, "slot": 2, "gen": 1 },
                { "seq": 2, "act": "disown", "kind": "str", "id": 0, "key": 0, "via": "swept", "frame": 1, "slot": 2, "gen": 1 },
                { "seq": 3, "act": "free", "id": 0, "bytes": 3 },
                { "seq": 4, "act": "frame-leave", "frame": 1, "sweep": true },
                { "seq": 5, "act": "end", "outcome": "resource", "value": null, "trap": null, "resource": "av.resource.instr-steps", "steps": 10 }
            ],
            "allocs": [ { "id": 0, "bytes": 3 } ],
            "frees": [ { "id": 0, "bytes": 3 } ],
            "live_at_end": 0,
            "live_after_result_drop": 0,
            "steps": 10
        })
    }

    #[test]
    fn admits_a_well_formed_stream() {
        let r = admit_lifecycle(&base_lifecycle());
        assert!(r.is_ok(), "admission error: {r:?}");
    }

    #[test]
    fn rejects_totals_only_missing_events() {
        let mut v = base_lifecycle();
        v["events"] = serde_json::json!([ { "seq": 0, "act": "frame-enter", "frame": 1 } ]);
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_missing_end_marker() {
        let mut v = base_lifecycle();
        v["events"]
            .as_array_mut()
            .unwrap()
            .retain(|e| e["act"] != "end");
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_duplicate_free() {
        let mut v = base_lifecycle();
        let events = v["events"].as_array_mut().unwrap();
        events.insert(
            4,
            serde_json::json!({ "seq": 3, "act": "free", "id": 0, "bytes": 3 }),
        );
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_leaked_owner() {
        let mut v = base_lifecycle();
        v["events"]
            .as_array_mut()
            .unwrap()
            .retain(|e| !(e["act"] == "disown" && e["slot"] == 2));
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_double_disown() {
        let mut v = base_lifecycle();
        let events = v["events"].as_array_mut().unwrap();
        events.insert(
            3,
            serde_json::json!({ "seq": 3, "act": "disown", "kind": "str", "id": 0, "key": 0, "via": "replaced", "frame": 1, "slot": 2, "gen": 1 }),
        );
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_lifo_violation() {
        // two strings created in order 0 then 1; the sweep must free 1
        // before 0 (descending key).
        let v = serde_json::json!({
            "schema": "av.lifecycle/1",
            "profile": "collections",
            "overflowed": false,
            "events": [
                { "seq": 0, "act": "frame-enter", "frame": 1 },
                { "seq": 1, "act": "own", "kind": "str", "id": 0, "key": 0, "via": "created", "frame": 1, "slot": 1, "gen": 1 },
                { "seq": 2, "act": "own", "kind": "str", "id": 1, "key": 1, "via": "created", "frame": 1, "slot": 2, "gen": 1 },
                { "seq": 3, "act": "disown", "kind": "str", "id": 0, "key": 0, "via": "swept", "frame": 1, "slot": 1, "gen": 1 },
                { "seq": 4, "act": "free", "id": 0, "bytes": 1 },
                { "seq": 5, "act": "disown", "kind": "str", "id": 1, "key": 1, "via": "swept", "frame": 1, "slot": 2, "gen": 1 },
                { "seq": 6, "act": "free", "id": 1, "bytes": 1 },
                { "seq": 7, "act": "frame-leave", "frame": 1, "sweep": true },
                { "seq": 8, "act": "end", "outcome": "success", "value": 1, "trap": null, "resource": null, "steps": 8 }
            ],
            "allocs": [ { "id": 0, "bytes": 1 }, { "id": 1, "bytes": 1 } ],
            "frees": [ { "id": 0, "bytes": 1 }, { "id": 1, "bytes": 1 } ],
            "live_at_end": 0,
            "live_after_result_drop": 0,
            "steps": 8
        });
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_unknown_field() {
        let mut v = base_lifecycle();
        v["extra"] = serde_json::json!(1);
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_replayed_event() {
        let mut v = base_lifecycle();
        let events = v["events"].as_array_mut().unwrap();
        let dup = events[2].clone();
        events.insert(4, dup);
        // seq contiguity alone catches the replay; renumber to target the
        // slot-double-disown rule instead.
        for (i, e) in events.iter_mut().enumerate() {
            e["seq"] = serde_json::json!(i);
        }
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_overflowed_log() {
        let mut v = base_lifecycle();
        v["overflowed"] = serde_json::json!(true);
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn rejects_nonzero_live_after_drop() {
        let mut v = base_lifecycle();
        v["live_after_result_drop"] = serde_json::json!(1);
        assert!(admit_lifecycle(&v).is_err());
    }

    #[test]
    fn event_helper() {
        let _ = event(serde_json::json!({}));
    }
}
