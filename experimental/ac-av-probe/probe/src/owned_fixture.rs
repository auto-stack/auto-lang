//! PLAN-821 T-02: the frozen `av.owned-fixture/1` table — the formal
//! `ac-av/owned-run-v1` recipe's owned entry face. An `owned:<id>` entry
//! selects one of these frozen text-record sources and runs it through the
//! SAME owned whole-path pipeline as any admitted unit (front door → 782
//! checked HIR → owned Typed MIR → AVAD v2 → store persist → re-verify →
//! AV). The WORKER reports the observed result as a canonical face plus
//! the arena ledger AFTER the host-side release; the JUDGING PARENT holds
//! its OWN frozen expectation (result + live=0 + alloc/free order) and
//! compares independently — the worker's word is never the oracle
//! (contract-decision §10/§11).
//!
//! Canonical scalar bits (both sides): lowercase hex, no `0x`; i64/u64/f64
//! 16 digits, f32 8 digits, i32/bool/char unpadded (`0` is the unique
//! zero). The Str face carries the exact UTF-8 bytes; the record face
//! carries the nominal name and fields in DECLARATION order.

use auto_av_core::contract::{ExecOutcome, Value};
use std::path::Path;

use crate::owned_driver::{request_owned, OwnedRequestOutcome};

/// One frozen owned fixture: id, source bytes, entry.
#[derive(Clone, Copy, Debug)]
pub struct OwnedFixture {
    pub id: &'static str,
    pub src: &'static str,
    pub entry: &'static str,
}

/// The frozen first-case source (contract-decision §11; byte-identical to
/// `crates/auto-lang/test/cases/av-data/821/src/first-case.at` — the
/// corpus-identity test pins the pair by SHA-256).
pub const FIRST_CASE_SRC: &str = "\
type Note { body Str; n int }
fn rebody(move u Note) Note {
    let m int = u.body.len()
    let c Str = u.body.clone()
    u.body = c.move
    u.n = m
    return u.move
}
fn main() Note {
    let a Note = Note { body: \"Auto\", n: 1 }
    return rebody(a.move)
}
";

/// The frozen fixture set.
pub const OWNED_FIXTURES: &[OwnedFixture] = &[OwnedFixture {
    id: "first-case",
    src: FIRST_CASE_SRC,
    entry: "d_main",
}];

pub fn owned_fixture(id: &str) -> Option<&'static OwnedFixture> {
    OWNED_FIXTURES.iter().find(|f| f.id == id)
}

/// Canonicalize an owned-path result for the wire face. `names` maps record
/// type-table indices to nominal names. `None` = a value that may never
/// cross the entry boundary (a view or write token — §6): the parent
/// treats that shape as an admission violation.
pub fn canonical_owned(
    value: &Value,
    names: &std::collections::BTreeMap<u32, String>,
) -> Option<serde_json::Value> {
    Some(match value {
        Value::I32(v) => serde_json::json!({ "kind": "i32", "bits": format!("{v:x}") }),
        Value::Bool(v) => serde_json::json!({
            "kind": "bool",
            "bits": if *v { "1" } else { "0" },
        }),
        Value::I64(v) => serde_json::json!({ "kind": "i64", "bits": format!("{v:016x}") }),
        Value::U64(v) => serde_json::json!({ "kind": "u64", "bits": format!("{v:016x}") }),
        Value::F32(v) => {
            serde_json::json!({ "kind": "f32", "bits": format!("{:08x}", v.to_bits()) })
        }
        Value::F64(v) => {
            serde_json::json!({ "kind": "f64", "bits": format!("{:016x}", v.to_bits()) })
        }
        Value::Char(c) => serde_json::json!({ "kind": "char", "bits": format!("{:x}", *c as u32) }),
        Value::Str(h) => serde_json::json!({
            "kind": "str",
            "utf8": String::from_utf8_lossy(h.bytes()),
        }),
        Value::Record(r) => {
            let name = names.get(&r.ty).cloned().unwrap_or_else(|| "?".to_string());
            let mut fields = Vec::with_capacity(r.fields.len());
            for f in &r.fields {
                fields.push(canonical_owned(f, names)?);
            }
            serde_json::json!({ "kind": "record", "name": name, "fields": fields })
        }
        // A view/token never crosses the entry boundary (§6).
        Value::StrView(_) | Value::MutRef(_) => return None,
    })
}

/// Run one frozen fixture through the REAL owned pipeline (store round
/// trip included), canonicalize the success value, RELEASE it host-side,
/// then read the arena ledger — the reported face the parent judges:
/// `{ result, live_after_release, allocs: [{id,bytes}], frees: [{id}] }`.
pub fn run_owned_fixture(
    f: &OwnedFixture,
    store_root: Option<&Path>,
    project: &str,
) -> Result<serde_json::Value, String> {
    let mut outcome: OwnedRequestOutcome = request_owned(store_root, project, f.src, f.entry, &[])?;
    let av = outcome.av.take().unwrap();
    let ExecOutcome::Success { value, .. } = av else {
        return Err(format!("fixture {} did not succeed: {av:?}", f.id));
    };
    let result = canonical_owned(&value, &outcome.record_names).ok_or_else(|| {
        "fixture result is not canonicalizable (view/token crossed entry)".to_string()
    })?;
    // Host release — the single responsibility; while held the arena keeps
    // the buffer live. The ledger read happens strictly AFTER the drop.
    drop(value);
    let arena = outcome
        .arena
        .clone()
        .ok_or("owned execution carries no arena")?;
    let allocs: Vec<serde_json::Value> = arena
        .alloc_events()
        .iter()
        .map(|a| serde_json::json!({ "id": a.id, "bytes": a.bytes }))
        .collect();
    let frees: Vec<serde_json::Value> = arena
        .free_events()
        .iter()
        .map(|e| serde_json::json!({ "id": e.id }))
        .collect();
    Ok(serde_json::json!({
        "result": result,
        "live_after_release": arena.live(),
        "allocs": allocs,
        "frees": frees,
    }))
}
