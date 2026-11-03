//! PLAN-919 (`ac-av/resource-events-v1`): the source→AV full-chain seam
//! with the per-owner lifecycle channel ENABLED, plus the R06 exit knobs
//! (instruction budget, wall budget, host cancellation).
//!
//! Same gate order as the 864/865 support seams — UNTRUSTED builder output
//! → `verify_mir` → `encode_abc` → `verify_abc_bytes` → execute — so the
//! observed run is exactly the production execution path, not a second
//! interpreter. Two REAL production source profiles are admitted — the
//! same two the frozen M1 corpus already runs on AV:
//! - `owned` (`compile_text_record_source` → `lower_checked_owned`, the
//!   `ac-av/owned-run-v1` front door): owned Str + records + `move`
//!   parameters — the T03/R04/R05 resource sources, i.e. cross-frame
//!   ownership transfer (R01);
//! - `collections` (`compile_collections_source` →
//!   `lower_checked_collections`, the `ac-av/collections-source-run-v1`
//!   front door): lists + variants + owned text — V06 (R04) and the deep
//!   owned trees / collections of R06.
//! No third frontend is invented here. The request carries ONLY the
//! profile, operations and budgets; no expectation ever reaches this
//! side (the parent re-judges the facts).

use ace_av_core::abc::encode_abc;
use ace_av_core::av::{execute_observed, ExecLimits};
use ace_av_core::contract::{ExecOutcome, Value};
use ace_av_core::lifecycle::LifecycleEvent;
use ace_av_core::verify_mir;
use ace_core::source::{compile_collections_source, compile_text_record_source};
use ace_lowering::{lower_checked_collections, lower_checked_owned};

/// The two admitted production source profiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SourceProfile {
    Owned,
    Collections,
}

impl SourceProfile {
    pub fn wire(&self) -> &'static str {
        match self {
            SourceProfile::Owned => "owned",
            SourceProfile::Collections => "collections",
        }
    }
    pub fn from_wire(s: &str) -> Option<Self> {
        match s {
            "owned" => Some(SourceProfile::Owned),
            "collections" => Some(SourceProfile::Collections),
            _ => None,
        }
    }
}

/// Operation knobs (R06 exits). `None` keeps the frozen probe defaults.
#[derive(Clone, Copy, Debug, Default)]
pub struct ResourceEventsKnobs {
    pub max_steps: Option<u64>,
    pub wall_ms: Option<u64>,
    /// Host cancellation: a thread flips the cancel flag after this delay
    /// (the VM polls it on its frozen cadence — no instruction-level
    /// interruption is promised, exactly the existing contract).
    pub cancel_after_ms: Option<u64>,
}

/// One observed execution's pure-data face.
#[derive(Clone, Debug)]
pub struct ResourceEventsRun {
    /// success | trap | resource | cancelled | infra
    pub outcome: &'static str,
    pub value: Option<i32>,
    pub trap: Option<String>,
    pub resource: Option<String>,
    pub infra: Option<String>,
    pub steps: u64,
    pub wall_ms: u64,
    /// The full lifecycle stream INCLUDING the `free` facts that fired
    /// while this function dropped the result value.
    pub events: Vec<LifecycleEvent>,
    pub overflowed: bool,
    pub allocs: Vec<(u64, usize)>,
    pub frees: Vec<(u64, usize)>,
    /// Arena live count right after execution (the result value — when
    /// it owns buffers — is still alive here).
    pub live_at_end: usize,
    /// Arena live count after the result value was dropped.
    pub live_after_result_drop: usize,
}

/// Compile and execute `src` with the lifecycle channel on.
pub fn compile_and_execute_observed(
    src: &str,
    entry_name: &str,
    profile: SourceProfile,
    knobs: ResourceEventsKnobs,
) -> Result<ResourceEventsRun, String> {
    let render = |ds: Vec<ace_core::atom_text::Diagnostic>| {
        ds.iter()
            .map(|d| d.render("probe.at", src))
            .collect::<Vec<_>>()
            .join(
                "
",
            )
    };
    let mir = match profile {
        SourceProfile::Owned => {
            let compiled = compile_text_record_source(src, "probe.at", &[]).map_err(render)?;
            lower_checked_owned(&compiled.checked, entry_name, fnv1a64(src.as_bytes()))
                .map_err(|e| format!("lower: {e:?}"))?
        }
        SourceProfile::Collections => {
            let compiled = compile_collections_source(src, "probe.at", &[]).map_err(render)?;
            lower_checked_collections(&compiled.checked, entry_name, 0)
                .map_err(|e| format!("lower: {e:?}"))?
        }
    };
    let _checked = verify_mir(&mir).map_err(|e| format!("verify_mir: {e:?}"))?;
    let abc = encode_abc(&mir).map_err(|e| format!("encode: {e:?}"))?;
    let flat = abc.to_flat();
    let loaded = ace_av_core::abc_verify::verify_abc_bytes(&flat, &mir.owner)
        .map_err(|e| format!("abc admit: {e}"))?;
    Ok(execute_loaded_observed(&loaded, knobs))
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Execute an already-admitted module with the lifecycle channel on.
pub fn execute_loaded_observed(
    loaded: &ace_av_core::abc_verify::LoadedModule,
    knobs: ResourceEventsKnobs,
) -> ResourceEventsRun {
    let mut limits = ExecLimits::default();
    if let Some(n) = knobs.max_steps {
        limits.max_steps = n;
    }
    if let Some(ms) = knobs.wall_ms {
        limits.wall = std::time::Duration::from_millis(ms);
    }
    let cancel = std::sync::Arc::new(ace_av_core::CancelFlag::new(false));
    let canceller = knobs.cancel_after_ms.map(|ms| {
        let flag = std::sync::Arc::clone(&cancel);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(ms));
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        })
    });
    let started = std::time::Instant::now();
    let (out, trace) = execute_observed(loaded, &limits, &cancel);
    let wall_ms = started.elapsed().as_millis() as u64;
    if let Some(h) = canceller {
        // The flag is also set by a finished sleeper; joining keeps the
        // thread from outliving the run (bounded host resources).
        cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        let _ = h.join();
    }
    let live_at_end = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    let (outcome, value, trap, resource, infra, steps, result_value) = match out {
        ExecOutcome::Success { value, steps } => {
            let v = match &value {
                Value::I32(v) => Some(*v),
                _ => None,
            };
            ("success", v, None, None, None, steps, Some(value))
        }
        ExecOutcome::Trap { kind, .. } => (
            "trap",
            None,
            Some(kind.code().to_string()),
            None,
            None,
            0,
            None,
        ),
        ExecOutcome::Resource { kind, .. } => (
            "resource",
            None,
            None,
            Some(kind.code().to_string()),
            None,
            0,
            None,
        ),
        ExecOutcome::Cancelled => ("cancelled", None, None, None, None, 0, None),
        ExecOutcome::Infra(m) => ("infra", None, None, None, Some(m), 0, None),
    };
    // Drop the result value LAST: its buffer frees (if any) append to the
    // same seq-ordered log after the `end` marker.
    drop(result_value);
    let live_after_result_drop = trace.arena.as_ref().map(|a| a.live()).unwrap_or(0);
    let allocs = trace
        .arena
        .as_ref()
        .map(|a| a.alloc_events().iter().map(|e| (e.id, e.bytes)).collect())
        .unwrap_or_default();
    let frees = trace
        .arena
        .as_ref()
        .map(|a| a.free_events().iter().map(|e| (e.id, e.bytes)).collect())
        .unwrap_or_default();
    let steps = if steps == 0 {
        trace
            .lifecycle
            .events()
            .iter()
            .rev()
            .find_map(|e| e.end.as_ref().map(|i| i.steps))
            .unwrap_or(0)
    } else {
        steps
    };
    ResourceEventsRun {
        outcome,
        value,
        trap,
        resource,
        infra,
        steps,
        wall_ms,
        events: trace.lifecycle.events(),
        overflowed: trace.lifecycle.overflowed(),
        allocs,
        frees,
        live_at_end,
        live_after_result_drop,
    }
}

/// The `av.lifecycle/1` wire form of one event (the worker's response).
/// Each action carries exactly its own fields — the parent's typed decode
/// refuses any other shape.
pub fn event_json(e: &LifecycleEvent) -> serde_json::Value {
    use ace_av_core::lifecycle::Act;
    let mut o = serde_json::Map::new();
    o.insert("seq".into(), e.seq.into());
    o.insert("act".into(), e.act.wire().into());
    match e.act {
        Act::FrameEnter => {
            o.insert("frame".into(), e.frame.into());
        }
        Act::FrameLeave => {
            o.insert("frame".into(), e.frame.into());
            o.insert("sweep".into(), e.sweep.into());
        }
        Act::Own | Act::Disown => {
            o.insert("kind".into(), e.kind.map(|k| k.wire()).unwrap_or("").into());
            o.insert("id".into(), e.id.into());
            o.insert("key".into(), e.key.into());
            let via = match e.act {
                Act::Own => e.via_owned.map(|v| v.wire()).unwrap_or(""),
                _ => e.via_disowned.map(|v| v.wire()).unwrap_or(""),
            };
            o.insert("via".into(), via.into());
            o.insert("frame".into(), e.frame.into());
            o.insert("slot".into(), e.slot.into());
            o.insert("gen".into(), e.gen.into());
        }
        Act::Free => {
            o.insert("id".into(), e.id.into());
            o.insert("bytes".into(), (e.bytes as u64).into());
        }
        Act::End => {
            let i = e.end.as_ref().expect("end info");
            o.insert("outcome".into(), i.outcome.into());
            o.insert(
                "value".into(),
                i.value
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            );
            o.insert(
                "trap".into(),
                i.trap
                    .clone()
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            );
            o.insert(
                "resource".into(),
                i.resource
                    .clone()
                    .map(serde_json::Value::from)
                    .unwrap_or(serde_json::Value::Null),
            );
            o.insert("steps".into(), i.steps.into());
        }
    }
    serde_json::Value::Object(o)
}
