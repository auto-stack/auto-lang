//! Bounded parent observation of the native runtime's existing exit/raw stderr.
//! This is not a guest PASS protocol or authentication of an arbitrary process.
use crate::{config, MnError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Return(i32),
    Trap(u32),
}
#[derive(Debug, Clone)]
pub struct Observation {
    pub outcome: Outcome,
    pub resources: BTreeMap<String, u64>,
    pub text: BTreeMap<String, u64>,
    /// PLAN-919: the strictly-decoded per-owner event stream (empty when
    /// the observation channel was off).
    pub events: Vec<NativeEvt>,
}

/// PLAN-919: one decoded `MN-EVT` line (pure data; 1-based contiguous seq).
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
fn bad(message: &str) -> MnError {
    MnError::Admit("collections-observation", message.into())
}
fn fields(line: &str, keys: &[&str]) -> Result<BTreeMap<String, u64>, MnError> {
    let mut result = BTreeMap::new();
    for field in line.split(' ') {
        let (k, v) = field
            .split_once('=')
            .ok_or_else(|| bad("malformed metric"))?;
        if !keys.contains(&k) || v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
            return Err(bad("unknown/malformed metric"));
        }
        let value = v.parse().map_err(|_| bad("metric overflow"))?;
        if result.insert(k.to_owned(), value).is_some() {
            return Err(bad("duplicate metric"));
        }
    }
    if result.len() != keys.len() {
        return Err(bad("missing metric"));
    }
    Ok(result)
}
pub fn decode(exit: Option<i32>, stderr: &[u8]) -> Result<Observation, MnError> {
    let exit =
        exit.ok_or_else(|| MnError::Infra("collections process has no exit status".into()))?;
    if stderr.len() > config::STREAM_CAP || stderr.last() != Some(&b'\n') {
        return Err(bad("missing/truncated/over-cap stderr"));
    }
    let text = std::str::from_utf8(stderr).map_err(|_| bad("non-UTF8 stderr"))?;
    let mut trap = None;
    let mut resources: BTreeMap<String, u64> = BTreeMap::new();
    let mut summary = None;
    let mut free = None;
    let mut events: Vec<NativeEvt> = Vec::new();
    let mut evt_end: Option<u64> = None;
    const KEYS: &[&str] = &[
        "live",
        "bytes",
        "guest",
        "peak",
        "allocs",
        "frees",
        "invalid",
        "arena_live",
        "arena_commit",
        "arena_peak",
        "arena_invalid",
        "text_metadata",
        "registered_bytes",
        "managed_commit_peak",
        "process_private_base",
        "process_private_end",
        "process_commit_peak",
        "process_commit_delta",
        "process_sample_ok",
    ];
    for line in text.split_terminator('\n') {
        if line.contains("MN-COLL-TRAP") {
            let value = line
                .strip_prefix("MN-COLL-TRAP code=")
                .ok_or_else(|| bad("forged trap prefix"))?;
            let reason: u32 = value.parse().map_err(|_| bad("malformed trap reason"))?;
            if value != reason.to_string()
                || !matches!(reason, 21..=27 | 30 | 31)
                || trap.replace(reason).is_some()
            {
                return Err(bad("unknown/duplicate trap reason"));
            }
        } else if let Some(metric) = line.strip_prefix("MN-COLL ") {
            let (k, v) = metric
                .split_once('=')
                .ok_or_else(|| bad("malformed resource line"))?;
            if !KEYS.contains(&k) || v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                return Err(bad("unknown/malformed resource line"));
            }
            let value = v.parse().map_err(|_| bad("resource metric overflow"))?;
            if resources.insert(k.to_owned(), value).is_some() {
                return Err(bad("duplicate resource line"));
            }
        } else if let Some(line) = line.strip_prefix("SUMMARY ") {
            if summary
                .replace(fields(
                    line,
                    &["attempts", "allocs", "frees", "live", "live_bytes", "peak"],
                )?)
                .is_some()
            {
                return Err(bad("duplicate text summary"));
            }
        } else if let Some(line) = line.strip_prefix("FREESTAT ") {
            if free
                .replace(fields(line, &["free_attempts", "frees_ok", "invalid"])?)
                .is_some()
            {
                return Err(bad("duplicate free summary"));
            }
        } else if let Some(rest) = line.strip_prefix("MN-EVTEND total=") {
            if evt_end.replace(rest.parse().map_err(|_| bad("malformed evt end"))?).is_some() {
                return Err(bad("duplicate evt end marker"));
            }
        } else if let Some(rest) = line.strip_prefix("MN-COLL-EVT cap=") {
            let _ = rest;
            return Err(MnError::Resource(
                "collections-observation-events",
                "the bounded observation log overflowed (named refusal, never truncation)".into(),
            ));
        } else if let Some(rest) = line.strip_prefix("MN-EVT ") {
            // PLAN-919 strict per-event decode: exact key set per action,
            // digits-only numbers, 1-based contiguous seq, fixed word
            // vocabularies. Anything else is a forged stream.
            let toks: Vec<&str> = rest.split(' ').collect();
            if toks.len() < 2 {
                return Err(bad("malformed event line"));
            }
            let num = |v: &str| -> Result<u64, MnError> {
                if v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(bad("malformed event number"));
                }
                v.parse().map_err(|_| bad("event number overflow"))
            };
            let mut kv: BTreeMap<&str, &str> = BTreeMap::new();
            for t in &toks {
                let (k, v) = t.split_once('=').ok_or_else(|| bad("malformed event field"))?;
                if kv.insert(k, v).is_some() {
                    return Err(bad("duplicate event field"));
                }
            }
            if kv.len() != toks.len()
                || !toks[0].starts_with("seq=")
                || !toks[1].starts_with("act=")
            {
                return Err(bad("event field order/set mismatch"));
            }
            let act = kv.get("act").copied().ok_or_else(|| bad("event without act"))?.to_string();
            let mut ev = NativeEvt {
                seq: num(kv.get("seq").copied().unwrap_or(""))?,
                act: act.clone(),
                kind: None,
                id: 0,
                key: 0,
                via: None,
                frame: 0,
                slot: u32::MAX,
                bytes: 0,
            };
            let want: &[&str] = match ev.act.as_str() {
                "frame-enter" => &["seq", "act", "frame"],
                "frame-leave" => &["seq", "act", "frame"],
                "own" | "disown" => &["seq", "act", "kind", "id", "key", "via", "frame", "slot"],
                "free" => &["seq", "act", "id", "bytes"],
                _ => return Err(bad("event act not in vocabulary")),
            };
            let mut got: Vec<&str> = kv.keys().copied().collect();
            got.sort_unstable();
            let mut want_sorted = want.to_vec();
            want_sorted.sort_unstable();
            if got != want_sorted {
                return Err(bad("event key set mismatch"));
            }
            match ev.act.as_str() {
                "frame-enter" | "frame-leave" => ev.frame = num(kv["frame"])?,
                "own" | "disown" => {
                    let kind = kv["kind"];
                    if !matches!(kind, "text" | "record" | "list" | "tagged") {
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
                _ => unreachable!("act filtered above"),
            }
            if ev.seq != events.len() as u64 + 1 {
                return Err(bad("event seq not 1-based contiguous"));
            }
            events.push(ev);
        }
    }
    // PLAN-919: events present require exactly one end marker whose total
    // matches; an end marker without events is forged.
    match (events.is_empty(), evt_end) {
        (false, Some(total)) if total == events.len() as u64 => {}
        (false, _) => return Err(bad("missing/mismatched evt end marker")),
        (true, Some(_)) => return Err(bad("evt end marker without events")),
        (true, None) => {}
    }
    if resources.len() != KEYS.len() {
        return Err(bad("missing resource observation"));
    }
    let summary = summary.ok_or_else(|| bad("missing text summary"))?;
    let free = free.ok_or_else(|| bad("missing text release summary"))?;
    if resources["process_sample_ok"] != 1
        || resources["process_private_base"] == 0
        || resources["process_private_end"] == 0
        || resources["process_commit_peak"] == 0
    {
        return Err(bad("missing/failed process memory observation"));
    }
    if resources["process_commit_delta"]
        != resources["process_commit_peak"].saturating_sub(resources["process_private_base"])
        || resources["process_commit_delta"] > 128 << 20
    {
        return Err(MnError::Resource(
            "collections-process",
            "invalid/over-cap process commit delta".into(),
        ));
    }
    for key in [
        "live",
        "bytes",
        "guest",
        "invalid",
        "arena_live",
        "arena_commit",
        "arena_invalid",
        "text_metadata",
        "registered_bytes",
    ] {
        if resources[key] != 0 {
            return Err(MnError::Resource(
                "collections-live",
                format!("{key}={}", resources[key]),
            ));
        }
    }
    if resources["allocs"] != resources["frees"]
        || summary["allocs"] != summary["frees"]
        || summary["live"] != 0
        || summary["live_bytes"] != 0
        || free["invalid"] != 0
        || free["frees_ok"] != summary["frees"]
    {
        return Err(MnError::Resource(
            "collections-release",
            "unbalanced native/text release".into(),
        ));
    }
    if resources["peak"] > 32 << 20
        || resources["arena_peak"] > 32 << 20
        || resources["managed_commit_peak"] > 32 << 20
        || summary["peak"] > 8 << 20
    {
        return Err(MnError::Resource(
            "collections-peak",
            "runtime peak exceeds fixed budget".into(),
        ));
    }
    let outcome = match trap {
        Some(reason) if exit == config::TRAP_EXIT_CODE => Outcome::Trap(reason),
        Some(_) => return Err(bad("trap evidence with non-trap exit")),
        None => Outcome::Return(exit),
    };
    Ok(Observation {
        outcome,
        resources,
        text: summary,
        events,
    })
}
