//! PLAN-919 T-04 (ac cells of scenario R01/R04/R06): the per-owner
//! lifecycle event stream through the REAL native collections pipeline
//! (source → frontend → MIR → COFF → PE → runtime → stderr) with the
//! R06 exit knobs baked into the entry wrapper.
//!
//! The formal parent-side oracle (strict MN-EVT admission + frozen
//! invariants + per-case tables) lives in `auto-lang-core::test_cases::ac_mn`
//! and the `resource-events-919` suite; these tests pin the runtime facts.

use ace_native::{
    collections_lower::WrapperKnobs,
    collections_observe::{decode, NativeEvt, Outcome},
    driver::MnDriver,
};
use std::path::PathBuf;

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "plan919-mn-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run_events(source: &str, knobs: WrapperKnobs) -> (Outcome, Vec<NativeEvt>) {
    let dir = scratch("run");
    let mut driver = MnDriver::ephemeral();
    let run = driver.request_resource_events(source, "d_main", &dir, None, knobs);
    let exit = run.exit_code.unwrap_or_else(|| {
        panic!(
            "no run facts: err={:?} stderr={}",
            run.error.map(|e| e.message()),
            String::from_utf8_lossy(&run.stderr)
        )
    });
    let obs = decode(Some(exit), &run.stderr)
        .unwrap_or_else(|e| panic!("strict decode of the observed stream: {e:?}"));
    (obs.outcome, obs.events)
}

const R01_SCOPES: &str = r#"
type Item { name Str; code int }
fn pick(s Str, flag int) int {
    let t Str = "tail"
    if flag > 0 {
        return s.len() + t.len()
    }
    let u Str = "u"
    return s.len() + u.len()
}
fn main() int {
    let a Str = "alpha"
    var xs = List<Str>.new()
    xs.push("one")
    xs.push("two")
    var it Item = Item("hello", 1)
    let code = it.code
    let got Str = xs.remove(1)
    let early = pick(a.clone(), 1)
    let late = pick(a.move, 0)
    return early * 100 + late * 10 + got.len() + code + it.name.len()
}
"#;

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

const R06_DEEP: &str = r#"
fn main() int {
    var seed = List<int>.new()
    seed.push(1)
    seed.push(2)
    seed.push(3)
    seed.push(4)
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
                    n = n + a + d
                }
            }
        }
    }
    return n + xs.len()
}
"#;

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
fn r01_scopes_emit_per_owner_events() {
    let (outcome, events) = run_events(
        R01_SCOPES,
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    // early = 5+4 = 9; late = 5+1 = 6; got "two" = 3; code = 1; name 5.
    assert_eq!(outcome, Outcome::Return(969));
    assert!(!events.is_empty());
    // three frames (main + pick + pick), nesting closed by the end marker
    let enters = events.iter().filter(|e| e.act == "frame-enter").count();
    let leaves = events.iter().filter(|e| e.act == "frame-leave").count();
    assert_eq!(enters, 3);
    assert_eq!(leaves, 3);
    // every own has a matching disown (slot alternation)
    use std::collections::BTreeMap;
    let mut slots: BTreeMap<(u64, u32), (usize, usize)> = BTreeMap::new();
    for e in &events {
        match e.act.as_str() {
            "own" => slots.entry((e.frame, e.slot)).or_default().0 += 1,
            "disown" => slots.entry((e.frame, e.slot)).or_default().1 += 1,
            _ => {}
        }
    }
    assert!(!slots.is_empty());
    for (k, (o, d)) in &slots {
        assert_eq!(o, d, "slot {k:?} own/disown must alternate");
    }
    // the payload transfer out of the list: a moved-out with a later received
    let moved = events
        .iter()
        .any(|e| e.act == "disown" && e.via.as_deref() == Some("moved-out"));
    assert!(moved, "xs.remove transfers ownership out of the list");
    // free order is a permutation with each id exactly once
    let mut frees: Vec<u64> = events
        .iter()
        .filter(|e| e.act == "free")
        .map(|e| e.id)
        .collect();
    let n = frees.len();
    frees.sort_unstable();
    frees.dedup();
    assert_eq!(frees.len(), n, "each node frees exactly once");
}

#[test]
fn r04_variant_switch_releases_old_payload_once() {
    let (outcome, events) = run_events(
        R04_V06,
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    assert_eq!(outcome, Outcome::Return(42));
    // exactly two variant packs (Err then Ok) and one text ("old"); every
    // created node frees exactly once (the strict decoder already proves
    // the balances — these pin the R04 shape).
    let tagged = events
        .iter()
        .filter(|e| e.act == "own" && e.kind.as_deref() == Some("tagged"))
        .count();
    let texts = events
        .iter()
        .filter(|e| {
            e.act == "own"
                && e.kind.as_deref() == Some("text")
                && e.via.as_deref() == Some("created")
        })
        .count();
    let frees = events.iter().filter(|e| e.act == "free").count();
    let owns = events
        .iter()
        .filter(|e| e.act == "own" && e.via.as_deref() == Some("created"))
        .count();
    assert_eq!(tagged, 2);
    assert_eq!(texts, 1);
    assert_eq!(frees, owns);
}

#[test]
fn r06_budget_exhaustion_sweeps_every_frame() {
    // A tiny guest budget refuses the deep build mid-run; the abort sweep
    // must still release every frame (events + balanced summary).
    let (outcome, events) = run_events(
        R06_DEEP,
        WrapperKnobs {
            observe: true,
            budget_bytes: Some(128),
            ..Default::default()
        },
    );
    assert_eq!(outcome, Outcome::Trap(22));
    // the sweep closed the root frame
    assert!(events.iter().any(|e| e.act == "frame-leave"));
}

#[test]
fn r06_cancellation_refuses_before_commit_and_sweeps() {
    // cancel_after_work: the first list-insert poll exceeds the allowance
    // → CANCELLED (-26) before mutation; the sweep still runs.
    let (outcome, events) = run_events(
        R06_DEEP,
        WrapperKnobs {
            observe: true,
            cancel_after_work: Some(3),
            ..Default::default()
        },
    );
    assert_eq!(outcome, Outcome::Trap(26));
    assert!(events.iter().any(|e| e.act == "frame-leave"));
}

#[test]
fn r06_trap_inside_callee_sweeps_both_frames() {
    let (outcome, events) = run_events(
        R06_TRAP,
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    // The real lowering renders `/ 0` as the arithmetic guard's overflow
    // face (31) — the fact of the compiled artifact, frozen as-is.
    assert_eq!(outcome, Outcome::Trap(31));
    let leaves: Vec<u64> = events
        .iter()
        .filter(|e| e.act == "frame-leave")
        .map(|e| e.frame)
        .collect();
    assert_eq!(leaves.len(), 2, "both frames unwind by the fatal sweep");
    assert!(
        leaves[0] > leaves[1],
        "the callee (higher activation) leaves first"
    );
}

#[test]
fn observation_off_keeps_the_historical_stream() {
    let dir = scratch("off");
    let mut driver = MnDriver::ephemeral();
    let run = driver.request_resource_events(R04_V06, "d_main", &dir, None, Default::default());
    assert_eq!(run.exit_code, Some(42));
    assert!(
        !run.stderr.windows(7).any(|w| w == b"MN-EVT"),
        "no events with the switch off"
    );
    let obs = decode(run.exit_code, &run.stderr).unwrap();
    assert!(obs.events.is_empty());
    assert_eq!(obs.outcome, Outcome::Return(42));
}

const R06_BIG: &str = r#"
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
            for c in seed {
                for d in seed {
                    xs.push("payload")
                }
            }
        }
    }
    return xs.len()
}
"#;

#[test]
fn event_cap_is_a_named_refusal_never_truncation() {
    // 8192-event cap: the deep program with observations far exceeds it;
    // the runtime prints the cap marker and aborts 22 — strict decode
    // turns that into a typed refusal, never a green truncated stream.
    let dir = scratch("cap");
    let mut driver = MnDriver::ephemeral();
    let run = driver.request_resource_events(
        R06_BIG,
        "d_main",
        &dir,
        None,
        WrapperKnobs {
            observe: true,
            ..Default::default()
        },
    );
    assert!(
        run.stderr.windows(13).any(|w| w == b"MN-COLL-EVT c"),
        "cap marker expected"
    );
    let exit = run.exit_code.expect("run facts retained");
    let decoded = decode(Some(exit), &run.stderr);
    assert!(decoded.is_err(), "a capped stream must not decode green");
}

#[test]
fn zz_dump_native_oracle_rows() {
    use std::collections::BTreeMap;
    let rows: Vec<(&str, WrapperKnobs, &str)> = vec![
        (
            "r01-ac",
            WrapperKnobs {
                observe: true,
                ..Default::default()
            },
            R01_SCOPES,
        ),
        (
            "r04-ac",
            WrapperKnobs {
                observe: true,
                ..Default::default()
            },
            R04_V06,
        ),
        (
            "r06-budget-ac",
            WrapperKnobs {
                observe: true,
                budget_bytes: Some(128),
                ..Default::default()
            },
            R06_DEEP,
        ),
        (
            "r06-cancel-ac",
            WrapperKnobs {
                observe: true,
                cancel_after_work: Some(3),
                ..Default::default()
            },
            R06_DEEP,
        ),
        (
            "r06-trap-ac",
            WrapperKnobs {
                observe: true,
                ..Default::default()
            },
            R06_TRAP,
        ),
    ];
    for (name, knobs, src) in rows {
        let (outcome, events) = run_events(src, knobs);
        let mut created: BTreeMap<&str, usize> = BTreeMap::new();
        for e in &events {
            if e.act == "own" && e.via.as_deref() == Some("created") {
                *created.entry(e.kind.as_deref().unwrap_or("?")).or_default() += 1;
            }
        }
        let free_order: Vec<u64> = events
            .iter()
            .filter(|e| e.act == "free")
            .map(|e| e.id)
            .collect();
        let frames = events.iter().filter(|e| e.act == "frame-enter").count();
        eprintln!(
            "NDUMP {name}: outcome={outcome:?} frames={} created={:?} free_order={:?}",
            frames, created, free_order
        );
    }
}
