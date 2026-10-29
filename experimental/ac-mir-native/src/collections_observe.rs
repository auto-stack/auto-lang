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
                || !matches!(reason, 21..=26 | 30 | 31)
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
        }
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
    })
}
