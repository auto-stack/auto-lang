//! PLAN-919 (`ac/mir-native-resource-events-v1`): the parent-side
//! INDEPENDENT oracle for the native per-owner event stream (`MN-EVT` on
//! the PE's stderr).
//!
//! Mirrors the av-side discipline: the request carries only the R06 exit
//! knobs; THIS module strictly decodes the stderr facts, checks the frozen
//! generic invariants (1-based contiguous seq; frame nesting; per-slot
//! own/disown alternation — every logical owner released; each node freed
//! exactly once; the `MN-EVTEND` marker matches) and judges the frozen
//! per-case table. The native/av channels are separate frozen vocabularies
//! (native seq is 1-based; native `frame-leave` has no sweep flag; native
//! disown adds `dropped`).

use super::hex_stream;
use auto_test_case::compare::Mismatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeEvt {
    pub seq: u64,
    pub act: String,
    pub kind: Option<String>,
    pub id: u64,
    pub key: u64,
    pub via: Option<String>,
    pub frame: u64,
    pub slot: u32,
    pub bytes: u64,
}

#[derive(Debug, Clone, Default)]
pub struct NativeFacts {
    pub frames: usize,
    pub created: std::collections::BTreeMap<String, usize>,
    pub free_order: Vec<u64>,
    pub evts: Vec<NativeEvt>,
    pub trap: Option<u32>,
}

fn bad(msg: impl Into<String>) -> String {
    format!("native lifecycle admission: {}", msg.into())
}

/// Strict decode of the event lines inside one observed stderr TEXT.
/// `text` is the already-unhexed stderr.
pub fn decode_events(text: &str) -> Result<Option<NativeFacts>, String> {
    let mut evts: Vec<NativeEvt> = Vec::new();
    let mut end_total: Option<u64> = None;
    let mut trap: Option<u32> = None;
    for line in text.split_terminator('\n') {
        if let Some(rest) = line.strip_prefix("MN-COLL-TRAP code=") {
            let code: u32 = rest
                .parse()
                .map_err(|_| bad("malformed native trap reason"))?;
            if rest != code.to_string() || !matches!(code, 21..=26 | 30 | 31) {
                return Err(bad("unknown native trap reason"));
            }
            if trap.replace(code).is_some() {
                return Err(bad("duplicate native trap"));
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("MN-EVTEND total=") {
            if end_total
                .replace(rest.parse().map_err(|_| bad("malformed evt end"))?)
                .is_some()
            {
                return Err(bad("duplicate evt end marker"));
            }
            continue;
        }
        if line.starts_with("MN-COLL-EVT cap=") {
            return Err(bad(
                "the bounded observation log overflowed (named refusal)",
            ));
        }
        let Some(rest) = line.strip_prefix("MN-EVT ") else {
            continue;
        };
        let toks: Vec<&str> = rest.split(' ').collect();
        if toks.len() < 2 || !toks[0].starts_with("seq=") || !toks[1].starts_with("act=") {
            return Err(bad("malformed event line"));
        }
        let num = |v: &str| -> Result<u64, String> {
            if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err(bad("malformed event number"));
            }
            v.parse().map_err(|_| bad("event number overflow"))
        };
        let mut kv: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
        for t in &toks {
            let (k, v) = t
                .split_once('=')
                .ok_or_else(|| bad("malformed event field"))?;
            if kv.insert(k, v).is_some() {
                return Err(bad("duplicate event field"));
            }
        }
        if kv.len() != toks.len() {
            return Err(bad("event field set mismatch"));
        }
        let act = kv.get("act").copied().unwrap_or("");
        let mut ev = NativeEvt {
            seq: num(kv.get("seq").copied().unwrap_or(""))?,
            act: act.to_string(),
            kind: None,
            id: 0,
            key: 0,
            via: None,
            frame: 0,
            slot: u32::MAX,
            bytes: 0,
        };
        let mut want = match ev.act.as_str() {
            "frame-enter" | "frame-leave" => vec!["seq", "act", "frame"],
            "own" | "disown" => {
                vec!["seq", "act", "kind", "id", "key", "via", "frame", "slot"]
            }
            "free" => vec!["seq", "act", "id", "bytes"],
            _ => return Err(bad("event act not in vocabulary")),
        };
        want.sort_unstable();
        let mut got: Vec<&str> = kv.keys().copied().collect();
        got.sort_unstable();
        if got != want {
            return Err(bad("event key set mismatch"));
        }
        match ev.act.as_str() {
            "frame-enter" | "frame-leave" => ev.frame = num(kv["frame"])?,
            "own" | "disown" => {
                let kind = kv["kind"];
                if !matches!(kind, "text" | "record" | "list" | "tagged" | "closure") {
                    return Err(bad("event kind not in vocabulary"));
                }
                let via = kv["via"];
                let ok_via = match ev.act.as_str() {
                    "own" => matches!(via, "created" | "received"),
                    _ => matches!(via, "moved-out" | "replaced" | "swept" | "dropped"),
                };
                if !ok_via {
                    return Err(bad("event via not in vocabulary"));
                }
                ev.kind = Some(kind.to_string());
                ev.id = num(kv["id"])?;
                ev.key = num(kv["key"])?;
                ev.via = Some(via.to_string());
                ev.frame = num(kv["frame"])?;
                let slot = num(kv["slot"])?;
                if slot > u32::MAX as u64 {
                    return Err(bad("event slot out of range"));
                }
                ev.slot = slot as u32;
            }
            "free" => {
                ev.id = num(kv["id"])?;
                ev.bytes = num(kv["bytes"])?;
            }
            _ => unreachable!("filtered above"),
        }
        if ev.seq != evts.len() as u64 + 1 {
            return Err(bad("event seq not 1-based contiguous"));
        }
        evts.push(ev);
    }
    if evts.is_empty() && end_total.is_none() {
        return Ok(None);
    }
    match (evts.is_empty(), end_total) {
        (false, Some(t)) if t == evts.len() as u64 => {}
        (false, _) => return Err(bad("missing/mismatched evt end marker")),
        (true, Some(_)) => return Err(bad("evt end marker without events")),
        (true, None) => {}
    }
    // ---- frozen generic invariants ------------------------------------
    let mut facts = NativeFacts {
        frames: evts.iter().filter(|e| e.act == "frame-enter").count(),
        ..Default::default()
    };
    // frame nesting: enter/leave balanced, leave of the innermost only
    let mut open: Vec<u64> = Vec::new();
    for e in &evts {
        match e.act.as_str() {
            "frame-enter" => {
                if open.contains(&e.frame) {
                    return Err(bad("a frame activation enters twice"));
                }
                open.push(e.frame);
            }
            "frame-leave" => match open.last() {
                Some(&top) if top == e.frame => {
                    open.pop();
                }
                _ => return Err(bad("frame-leave does not match the innermost open frame")),
            },
            _ => {}
        }
    }
    if !open.is_empty() {
        return Err(bad(format!("{} frame(s) never left", open.len())));
    }
    // per (frame, slot): own/disown alternation starting with own
    use std::collections::BTreeMap;
    let mut slots: BTreeMap<(u64, u32), Vec<&str>> = BTreeMap::new();
    for e in &evts {
        match e.act.as_str() {
            "own" => slots.entry((e.frame, e.slot)).or_default().push("own"),
            "disown" => slots.entry((e.frame, e.slot)).or_default().push("disown"),
            _ => {}
        }
    }
    for (k, seq) in &slots {
        if seq.first() != Some(&"own") {
            return Err(bad(format!("slot {k:?} disowns before owning")));
        }
        let mut expect_own = true;
        for s in seq {
            let is_own = *s == "own";
            if is_own != expect_own {
                return Err(bad(format!(
                    "slot {k:?} own/disown does not alternate (double release or leak)"
                )));
            }
            expect_own = !expect_own;
        }
        if !expect_own {
            return Err(bad(format!("slot {k:?} ends owning (leak)")));
        }
    }
    // every created node frees exactly once, in destruction order
    let mut created_total = 0usize;
    for e in &evts {
        if e.act == "own" && e.via.as_deref() == Some("created") {
            *facts
                .created
                .entry(e.kind.clone().unwrap_or_default())
                .or_default() += 1;
            created_total += 1;
        }
        if e.act == "free" {
            facts.free_order.push(e.id);
        }
    }
    let mut sorted = facts.free_order.clone();
    sorted.sort_unstable();
    sorted.dedup();
    if sorted.len() != facts.free_order.len() {
        return Err(bad("a node frees twice (double free)"));
    }
    if facts.free_order.len() != created_total {
        return Err(bad(format!(
            "{} nodes created but {} freed (leak or forged frees)",
            created_total,
            facts.free_order.len()
        )));
    }
    facts.evts = evts;
    facts.trap = trap;
    Ok(Some(facts))
}

/// Decode from the worker's `stderr_hex` field.
pub fn decode_events_hex(stderr_hex: &str) -> Result<Option<NativeFacts>, String> {
    let bytes = hex_stream(stderr_hex)?;
    if bytes.last() != Some(&b'\n') {
        return Err(bad("missing/truncated native observation"));
    }
    let text = std::str::from_utf8(&bytes).map_err(|_| bad("native observation UTF8"))?;
    decode_events(text)
}

/// The frozen per-case oracle row.
pub struct NativeOracle {
    /// success | trap | budget | injected | cancelled
    pub outcome: &'static str,
    pub exit: Option<i32>,
    pub frames: usize,
    pub created: &'static [(&'static str, usize)],
    pub free_order: &'static [u64],
}

static ORACLES: &[(&str, NativeOracle)] = &[
    (
        "ace.m1.r01.mn/ac-native",
        NativeOracle {
            outcome: "success",
            exit: Some(969),
            frames: 3,
            created: &[("list", 1), ("record", 1), ("text", 8)],
            free_order: &[8, 7, 10, 9, 1, 4, 6, 5, 2, 3],
        },
    ),
    (
        "ace.m1.r04.mn/ac-native",
        NativeOracle {
            outcome: "success",
            exit: Some(42),
            frames: 1,
            created: &[("tagged", 2), ("text", 1)],
            free_order: &[2, 1, 3],
        },
    ),
    (
        "ace.m1.r06-budget.mn/ac-native",
        NativeOracle {
            outcome: "budget",
            exit: Some(70),
            frames: 1,
            created: &[("list", 1)],
            free_order: &[1],
        },
    ),
    (
        "ace.m1.r06-cancel.mn/ac-native",
        NativeOracle {
            outcome: "cancelled",
            exit: Some(70),
            frames: 1,
            created: &[("list", 1)],
            free_order: &[1],
        },
    ),
    (
        "ace.m1.r06-trap.mn/ac-native",
        NativeOracle {
            outcome: "trap",
            exit: Some(70),
            frames: 2,
            created: &[("list", 1), ("text", 4)],
            free_order: &[5, 4, 1, 3, 2],
        },
    ),
];

pub fn oracle_for(key: &str) -> Option<&'static NativeOracle> {
    ORACLES.iter().find(|(k, _)| *k == key).map(|(_, o)| o)
}

/// The typed Run-stage error code of a native lifecycle exit face.
pub fn completion_code(trap: u32) -> Option<&'static str> {
    match trap {
        22 => Some("mn.resource.execution-budget"),
        25 => Some("mn.resource.allocation-injected"),
        26 => Some("mn.resource.cancelled"),
        21 => Some("mn.trap.list-index"),
        24 => Some("mn.trap.variant-arm"),
        30 => Some("mn.trap.division"),
        31 => Some("mn.trap.overflow"),
        _ => None,
    }
}

/// Judge one native resource-events run against the frozen table.
pub fn judge_native(
    oracle: &NativeOracle,
    exit: Option<i32>,
    facts: &NativeFacts,
    mismatches: &mut Vec<Mismatch>,
) {
    fn m(mismatches: &mut Vec<Mismatch>, kind: &'static str, expected: String, actual: String) {
        mismatches.push(Mismatch {
            observation: "native-lifecycle".into(),
            kind,
            expected,
            actual,
            first_diff: None,
        });
    }
    let outcome = match facts.trap {
        None => "success",
        Some(22) => "budget",
        Some(25) => "injected",
        Some(26) => "cancelled",
        Some(_) => "trap",
    };
    if outcome != oracle.outcome {
        m(mismatches, "outcome", oracle.outcome.into(), outcome.into());
    }
    if exit != oracle.exit {
        m(
            mismatches,
            "exit",
            format!("{:?}", oracle.exit),
            format!("{exit:?}"),
        );
    }
    if facts.frames != oracle.frames {
        m(
            mismatches,
            "frames",
            oracle.frames.to_string(),
            facts.frames.to_string(),
        );
    }
    for (kind, want) in oracle.created {
        let got = facts.created.get(*kind).copied().unwrap_or(0);
        if got != *want {
            m(
                mismatches,
                "created-count",
                format!("{kind}={want}"),
                format!("{kind}={got}"),
            );
        }
    }
    if facts.created.len() != oracle.created.len() {
        m(
            mismatches,
            "created-kinds",
            format!("{:?}", oracle.created),
            format!("{:?}", facts.created),
        );
    }
    if facts.free_order != oracle.free_order {
        mismatches.push(Mismatch {
            observation: "native-lifecycle".into(),
            kind: "free-order",
            expected: format!("{:?}", oracle.free_order),
            actual: format!("{:?}", facts.free_order),
            first_diff: None,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_stream() -> String {
        [
            "MN-EVT seq=1 act=frame-enter frame=1",
            "MN-EVT seq=2 act=own kind=text id=1 key=1 via=created frame=1 slot=2",
            "MN-EVT seq=3 act=disown kind=text id=1 key=1 via=swept frame=1 slot=2",
            "MN-EVT seq=4 act=free id=1 bytes=64",
            "MN-EVT seq=5 act=frame-leave frame=1",
            "MN-EVTEND total=5",
            "MN-COLL-TRAP code=22",
            "",
        ]
        .join("\n")
    }

    #[test]
    fn admits_a_well_formed_native_stream() {
        let f = decode_events(&base_stream()).unwrap().unwrap();
        assert_eq!(f.frames, 1);
        assert_eq!(f.free_order, vec![1]);
        assert_eq!(f.trap, Some(22));
    }

    #[test]
    fn rejects_missing_end_marker() {
        let s: String = base_stream()
            .lines()
            .filter(|l| !l.starts_with("MN-EVTEND"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert!(decode_events(&s).is_err());
    }

    #[test]
    fn rejects_double_release() {
        let s = base_stream().replace(
            "MN-EVT seq=4",
            "MN-EVT seq=4 act=disown kind=text id=1 key=1 via=swept frame=1 slot=2\nMN-EVT seq=5",
        );
        // renumber to keep contiguity so the alternation rule is what fires
        let lines: Vec<&str> = s.lines().collect();
        let mut renumbered = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            let mut l = l.to_string();
            if l.starts_with("MN-EVT seq=") {
                let rest = l.split_off("MN-EVT seq=".len());
                let tail = rest.split_once(' ').unwrap().1.to_string();
                l = format!("MN-EVT seq={} {}", i + 1, tail);
            }
            renumbered.push(l);
        }
        let s2 = renumbered.join("\n") + "\n";
        assert!(decode_events(&s2).is_err());
    }

    #[test]
    fn rejects_leak() {
        let s: String = base_stream()
            .lines()
            .filter(|l| !l.contains("via=swept") && !l.contains("act=free"))
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";
        assert!(decode_events(&s).is_err());
    }

    #[test]
    fn rejects_cap_marker() {
        let s = base_stream() + "MN-COLL-EVT cap=8192\n";
        assert!(decode_events(&s).is_err());
    }

    #[test]
    fn rejects_forged_vocabulary() {
        let s = base_stream().replace("via=created", "via=fresh");
        assert!(decode_events(&s).is_err());
    }

    #[test]
    fn none_when_no_events() {
        assert!(decode_events("MN-COLL live=0\n").unwrap().is_none());
    }
}
