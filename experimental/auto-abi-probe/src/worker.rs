//! `abi/interop-v1` worker (PLAN-797 T-06): the execution half of the
//! restricted probe/run seam, mirroring `semaworker/1`'s envelope
//! discipline (begin/request/end, one response line, nonce/request_hash
//! echo, deny_unknown_fields, bounded reads). The worker drives the
//! REAL probe pipeline — deck parse → layout (store) → callee produce
//! (store) → per-leg consumer → admission → link → run → exit compare —
//! and reports typed step outcomes for PARENT-side judgement; its word
//! is never trusted.
//!
//! The deck is DATA (the case's `atomdata` source): a frozen, tiny,
//! worker-owned grammar. A case can never name an executable.

use crate::cache::{autoabi_object_key, produce_c_object, produce_with_cache};
use crate::decl::{self, AbiScalar, BodyKind, FnDecl, MixInputs};
use crate::emit::{emit_callee_suite, emit_caller_suite, SuiteCheck};
use crate::layout::{self, REC_MIX};
use crate::link::link_pe;
use crate::run;
use ac_artifact_probe::store::ManagedStore;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::time::Instant;

pub const WORKER_SCHEMA: &str = "abiworker/1";
pub const RECIPE: &str = "abi/interop-v1";
pub const WORKER_VERSION: &str = concat!("abi-worker/", env!("CARGO_PKG_VERSION"));

/// Bounded request line (the seam freezes request metadata at 4 MiB).
const MAX_LINE: usize = 4 << 20;

// ---------------------------------------------------------------------------
// wire types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct SessionBegin {
    op: String,
    schema: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerRequest {
    pub recipe: String,
    pub nonce: String,
    pub request_hash: String,
    pub deck_b64: String,
    pub leg: String,
    pub store_root: String,
    pub steps: Vec<StepIn>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepIn {
    pub op: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct MeterOut {
    /// DISK bytes: the largest scratch directory currently on disk for
    /// this store's worker (NOT a memory figure — F-797-R4-10 R8: the
    /// process memory peak / I/O attribution lives parent-side, from
    /// the contained worker's Job object).
    pub scratch_peak_disk_bytes: u64,
    pub hot_bytes: u64,
    pub rejections: u32,
    /// GC receipt facts (cumulative for the session).
    pub gc_runs: u32,
    pub gc_deleted_chunks: u32,
    pub gc_freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageSpan {
    pub name: String,
    pub wall_ns: u128,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagOut {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StepOut {
    pub op: &'static str,
    /// ok | diags | resource | infra — mutually exclusive typed outcome.
    pub outcome: &'static str,
    pub cache: String,
    pub compute_calls: u32,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<DiagOut>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub stages: Vec<StageSpan>,
    pub meter: MeterOut,
    /// The real PE exit code (query steps only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pe_exit: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OkOut {
    pub recipe: &'static str,
    pub steps: Vec<StepOut>,
    pub restart_pending: bool,
    pub worker_version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(deny_unknown_fields)]
struct ResponseEnvelope {
    schema: &'static str,
    nonce: String,
    request_hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    ok: Option<OkOut>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<DiagOut>,
}

// ---------------------------------------------------------------------------
// deck grammar (data, frozen; deny-unknown by construction)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Deck {
    pub params: Vec<(String, String)>, // (name, ty spelling)
    pub ret: String,
    pub body: String, // "mix-v1" | "mix-v1b"
    pub inputs: MixInputs,
}

fn parse_deck(text: &str) -> Result<Deck, String> {
    let mut header: Option<()> = None;
    let mut fn_line: Option<&str> = None;
    let mut rec_line: Option<&str> = None;
    let mut inputs_line: Option<&str> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if header.is_none() {
            if line == "deck autoabi/1" {
                header = Some(());
                continue;
            }
            return Err(format!(
                "deck must start with `deck autoabi/1`, got {line:?}"
            ));
        }
        if line.starts_with("fn ") && fn_line.is_none() {
            fn_line = Some(line);
        } else if line.starts_with("rec ") && rec_line.is_none() {
            rec_line = Some(line);
        } else if line.starts_with("inputs {") && inputs_line.is_none() {
            inputs_line = Some(line);
        } else {
            return Err(format!("unknown or duplicate deck line: {line:?}"));
        }
    }
    header.ok_or("deck missing the `deck autoabi/1` header")?;
    let fn_line = fn_line.ok_or("deck needs one `fn` declaration")?;
    let inputs_line = inputs_line.ok_or("deck needs one `inputs` block")?;

    // fn <sym>(<n>:<ty>, ...) -> <ty> [body: <id>]
    let rest = fn_line.strip_prefix("fn ").ok_or("bad fn line")?;
    let (sig, body_tag) = rest
        .split_once("[body:")
        .ok_or("fn line needs a `[body: ...]` tag")?;
    let body = body_tag.trim_end_matches(']').trim().to_string();
    let weight_tag = body
        .strip_prefix("mix-w")
        .and_then(|k| k.parse::<i64>().ok());
    let alt_tag = body
        .strip_prefix("mix-alt")
        .and_then(|k| k.parse::<u64>().ok());
    if !matches!(body.as_str(), "mix-v1" | "mix-v1b") && weight_tag.is_none() && alt_tag.is_none() {
        return Err(format!(
            "unknown body {:?} (v1 freezes mix-v1 / mix-v1b / mix-w<weight> / mix-alt<seed>)",
            body
        ));
    }
    let sig = sig.trim();
    let (name, sig) = sig.split_once('(').ok_or("fn line needs `(params)`")?;
    if name.trim() != "p797_mix" {
        return Err(format!(
            "v1 freezes the single function p797_mix, got {name:?}"
        ));
    }
    let (params_paren, ret_txt) = sig.rsplit_once("->").ok_or("fn line needs `) -> <ty>`")?;
    let params_txt = params_paren
        .trim()
        .strip_suffix(')')
        .ok_or("fn line needs a closing `)` before `->`")?;
    let ret = ret_txt.trim().to_string();
    let mut params = Vec::new();
    for p in params_txt.split(',') {
        let p = p.trim();
        if p.is_empty() {
            continue;
        }
        let (n, t) = p
            .split_once(':')
            .ok_or_else(|| format!("param {p:?} needs `name:ty`"))?;
        params.push((n.trim().to_string(), t.trim().to_string()));
    }
    // F-797-R4-01: the deck spells LOGICAL types; they must equal the
    // SEMANTIC fixture signature exactly (per-position kinds minted by
    // the real contract). Same-width substitutions (u32 for i32) are
    // typed deviations here, never silently accepted.
    for (_, t) in &params {
        if crate::semref::sem_kind_of(t).is_none() {
            return Err(format!("param type {t:?} is not a semantic kind"));
        }
    }
    let want_kinds: Vec<crate::semref::SemKind> = crate::semref::mix_param_semantics().to_vec();
    let got: Vec<crate::semref::SemKind> = params
        .iter()
        .map(|(_, t)| crate::semref::sem_kind_of(t).expect("checked above"))
        .collect();
    if got != want_kinds {
        let want_s: Vec<String> = want_kinds.iter().map(|k| format!("{k:?}")).collect();
        let got_s: Vec<String> = got.iter().map(|k| format!("{k:?}")).collect();
        return Err(format!(
            "v1 freezes the p797_mix SEMANTIC signature {:?}; deck spells {:?}",
            want_s, got_s
        ));
    }
    if crate::semref::sem_kind_of(&ret) != Some(crate::semref::SemKind::I64) {
        return Err(format!(
            "v1 freezes the i64 return; deck spells {ret:?} (logical kinds, not widths)"
        ));
    }

    // inputs { a: 7, ..., f: 1.5, ... }
    let inner = inputs_line
        .trim()
        .strip_prefix("inputs {")
        .and_then(|s| s.strip_suffix('}'))
        .ok_or("bad inputs block")?;
    let mut vals: Vec<(String, String)> = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (n, v) = part
            .split_once(':')
            .ok_or_else(|| format!("input {part:?} needs `name: value`"))?;
        vals.push((n.trim().to_string(), v.trim().to_string()));
    }
    // F-797-R4-01: every input parses IN RANGE for its SEMANTIC param
    // kind — an i32 overflow (e.g. 2147483648) or a non-finite f64 is a
    // typed diagnostic, never a panic and never a native trap.
    let raw = |name: &str| -> Result<&String, String> {
        vals.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
            .ok_or_else(|| format!("inputs missing {name:?}"))
    };
    let i32_of = |name: &str| -> Result<i32, String> {
        raw(name)?.parse::<i32>().map_err(|_| {
            format!(
                "input {name} is not an in-range i32: {}",
                raw(name).cloned().unwrap_or_default()
            )
        })
    };
    let i64_of = |name: &str| -> Result<i64, String> {
        raw(name)?.parse::<i64>().map_err(|_| {
            format!(
                "input {name} is not an in-range i64: {}",
                raw(name).cloned().unwrap_or_default()
            )
        })
    };
    let finite_f64_of = |name: &str| -> Result<f64, String> {
        let v: f64 = raw(name)?.parse().map_err(|_| {
            format!(
                "input {name} is not an f64: {}",
                raw(name).cloned().unwrap_or_default()
            )
        })?;
        if !v.is_finite() {
            return Err(format!(
                "input {name} must be a FINITE f64 (profile rejects NaN/Inf before any native call): {v}"
            ));
        }
        // F-797-R4-01 (R5): the recipe's frozen bodies convert f64 -> i64
        // (truncating); values outside ±2^53 are not convertible without
        // a native trap — reject at the deck gate, never in native code.
        if v.abs() > 9_007_199_254_740_992.0 {
            return Err(format!(
                "input {name} = {v} is outside the recipe's convertible f64->i64 range (|v| <= 2^53):                  typed rejection before any native call"
            ));
        }
        Ok(v)
    };
    let inputs = MixInputs {
        a: i32_of("a")?,
        b: i32_of("b")?,
        c: i32_of("c")?,
        d: i32_of("d")?,
        e: i64_of("e")?,
        f: finite_f64_of("f")?,
        g: i32_of("g")?,
        h: finite_f64_of("h")?,
    };
    // F-797-R4-01: the rec line is CONSUMED — it must spell the frozen
    // RecMix semantic shape exactly (field order, names, kinds); any
    // deviation (including same-width re-spellings) is a typed deck
    // diagnostic.
    let rec_src =
        rec_line.ok_or("deck needs the `rec RecMix { ... }` line (semantic shape is consumed)")?;
    let (rec_name, rec_body) = rec_src
        .trim()
        .strip_prefix("rec ")
        .and_then(|r| r.split_once('{'))
        .ok_or("bad rec line (expected `rec <Name> { ty name; ... }`)")?;
    if rec_name.trim() != "RecMix" {
        return Err(format!(
            "v1 freezes the RecMix record; deck spells {:?}",
            rec_name.trim()
        ));
    }
    let rec_inner = rec_body
        .rsplit_once('}')
        .map(|(b, _)| b)
        .ok_or("bad rec line (missing `}`)")?;
    {
        let mut fields: Vec<(String, String)> = Vec::new();
        for part in rec_inner.split(';') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            let (t, n) = part
                .split_once(' ')
                .ok_or_else(|| format!("rec field {part:?} needs `ty name`"))?;
            fields.push((t.trim().to_string(), n.trim().to_string()));
        }
        let want = crate::semref::rec_mix_semantics();
        if fields.len() != want.len() {
            return Err(format!(
                "RecMix freezes {} semantic fields; deck spells {}",
                want.len(),
                fields.len()
            ));
        }
        for (i, ((dt, dn), (wn, wk))) in fields.iter().zip(want.iter()).enumerate() {
            let dk = crate::semref::sem_kind_of(dt)
                .ok_or_else(|| format!("rec field {dn} has non-semantic type {dt:?}"))?;
            if dn != wn || dk != *wk {
                return Err(format!(
                    "RecMix field {i} must be `{} {:?}`; deck spells `{} {}`",
                    wn, wk, dt, dn
                ));
            }
        }
    }
    Ok(Deck {
        params,
        ret,
        body,
        inputs,
    })
}

impl Deck {
    /// The declaration the emitter builds from this deck. v1 freezes the
    /// single function's machine signature (8 params, i32x4/i64/f64/i32/
    /// f64, i64 return) — a deck that spells anything else is a TYPED
    /// DECK DIAGNOSTIC (the discriminating descriptor-edit negative for
    /// the formal AutoDown flow), never a silent re-shape.
    fn fn_decl(&self) -> Result<FnDecl, String> {
        let map = |t: &str| match t {
            "i32" | "u32" | "bool" | "char" => AbiScalar::I32,
            "i64" | "u64" => AbiScalar::I64,
            "f64" => AbiScalar::F64,
            _ => AbiScalar::I32,
        };
        let got: Vec<AbiScalar> = self.params.iter().map(|(_, t)| map(t)).collect();
        if got != decl::MIX_PARAMS.to_vec() {
            return Err(format!(
                "v1 freezes the p797_mix machine signature; deck spells {:?}",
                self.params
                    .iter()
                    .map(|(_, t)| t.as_str())
                    .collect::<Vec<_>>()
            ));
        }
        if map(&self.ret) != AbiScalar::I64 {
            return Err(format!(
                "v1 freezes the i64 return; deck spells {:?}",
                self.ret
            ));
        }
        let body = if let Some(k) = self.body.strip_prefix("mix-w") {
            BodyKind::MixW(k.parse::<i64>().expect("validated at parse"))
        } else if let Some(k) = self.body.strip_prefix("mix-alt") {
            BodyKind::MixAlt(k.parse::<u64>().expect("validated at parse"))
        } else if self.body == "mix-v1b" {
            BodyKind::MixV1Alt
        } else {
            BodyKind::MixV1
        };
        Ok(FnDecl {
            symbol: "p797_mix",
            params: decl::MIX_PARAMS,
            ret: AbiScalar::I64,
            body,
        })
    }
}

// ---------------------------------------------------------------------------
// step execution
// ---------------------------------------------------------------------------

struct Session {
    deck: Vec<u8>,
    leg: String,
    store_root: std::path::PathBuf,
    step_index: usize,
    rejections: u32,
    /// F-797-R5-03: degraded store operations (corrupt block, unwritable
    /// store) — reported in the meter, never a panic.
    store_rejections: u32,
    /// F-797-R4-10 (R9): cumulative GC receipt facts for this session —
    /// the meter publishes them so reclamation is ASSERTABLE, not
    /// assumed (a forced run below the water level legitimately reports
    /// ran=false / deleted=0).
    gc_runs: u32,
    gc_deleted_chunks: u32,
    gc_freed_bytes: u64,
    /// F-797-R4-10 (R5): bytes PRODUCED this session (emitted objects +
    /// compiled C) against the per-request budget (128MiB by contract;
    /// the TEST seam `ABI_TEST_PRODUCED_BUDGET_BYTES` shrinks it so the
    /// budget line is exercisable by a real worker — production code
    /// only READS it; cases can never set child env).
    produced_bytes: u64,
    budget_cap: u64,
}

/// F-797-R4-10: RECURSIVE directory size — the store keeps blobs under
/// blocks/blobs/<shard>/, so a top-level-only count constantly read 0.
fn dir_size(p: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let Ok(md) = e.metadata() else { continue };
            if md.is_file() {
                total += md.len();
            } else if md.is_dir() {
                total += dir_size(&e.path());
            }
        }
    }
    total
}

fn stage_span(name: &str, t0: Instant) -> StageSpan {
    StageSpan {
        name: name.into(),
        wall_ns: t0.elapsed().as_nanos(),
    }
}

fn budget_msg(sess: &Session) -> String {
    format!(
        "session produced-bytes budget exhausted: {} produced, cap {}",
        sess.produced_bytes, sess.budget_cap
    )
}

fn over_budget(sess: &Session) -> bool {
    sess.produced_bytes > sess.budget_cap
}

fn execute_query(sess: &mut Session) -> StepOut {
    // F-797-R4-03 (R5) TEST seam: report an infra outcome without doing
    // any work (drives the PARENT's infrastructure short-circuit).
    if std::env::var("ABI_TEST_INFRA_STEP").is_ok() {
        return infra_step(
            "fault seam: simulated infrastructure failure".into(),
            0,
            sess,
        );
    }
    let mut stages = Vec::new();
    let mut diags = Vec::new();
    let mut compute_calls = 0u32;
    let mut cache = "-".to_string();

    // deck parse (typed diags — a data error, not infra)
    let t0 = Instant::now();
    let deck = match parse_deck(&String::from_utf8_lossy(&sess.deck)) {
        Ok(d) => d,
        Err(e) => {
            return StepOut {
                op: "query",
                outcome: "diags",
                cache,
                compute_calls: 0,
                diagnostics: vec![DiagOut {
                    code: "deck".into(),
                    message: e,
                }],
                stages: vec![stage_span("parse", t0)],
                meter: meter(sess),
                pe_exit: None,
            }
        }
    };
    let fn_decl = match deck.fn_decl() {
        Ok(d) => d,
        Err(e) => {
            return StepOut {
                op: "query",
                outcome: "diags",
                cache,
                compute_calls: 0,
                diagnostics: vec![DiagOut {
                    code: "deck".into(),
                    message: e,
                }],
                stages: vec![stage_span("parse", t0)],
                meter: meter(sess),
                pe_exit: None,
            }
        }
    };
    stages.push(stage_span("parse", t0));

    // layout tier (store-backed since T-01)
    let t0 = Instant::now();
    let store = match ManagedStore::open(&sess.store_root) {
        Ok(s) => s,
        Err(e) => return infra_step(format!("store open: {e:?}"), compute_calls, sess),
    };
    let computed = layout::layout_record(&REC_MIX).expect("frozen layout");
    // F-797-R5-03: the layout cache is a CACHE — a corrupt/missing/
    // unwritable store entry degrades to recompute-and-continue (or
    // skip persisting), NEVER a panic on the production path. Store
    // rejections are counted in the meter for observability.
    match layout::get_layout(&store, &REC_MIX) {
        Ok(None) => {
            if let Err(e) = layout::put_layout(&store, &REC_MIX, &computed) {
                sess.store_rejections += 1;
                eprintln!("# abi-worker: layout cache put degraded (continuing uncached): {e:?}");
            }
        }
        Ok(Some(_)) => {}
        Err(e) => {
            sess.store_rejections += 1;
            eprintln!("# abi-worker: layout cache read degraded (recomputing): {e:?}");
        }
    }
    stages.push(stage_span("layout", t0));

    // admission: deck fn signature vs the frozen consumer expectation —
    // the descriptor-edit reject path lands HERE (typed diags).
    // F-797-R4-08: the descriptor is DERIVED, never emitted — admission
    // cannot secretly run the producer on the cache-hit path.
    let t0 = Instant::now();
    let defs = [fn_decl];
    let descriptor = crate::build::descriptor_for_defs(&defs);
    let descriptor_bytes = match descriptor.encode() {
        Ok(b) => b,
        Err(e) => return infra_step(format!("descriptor encode: {e}"), compute_calls, sess),
    };
    let expectation = crate::build::suite_expectation(&defs);
    match crate::validate::admit(&descriptor_bytes, &expectation) {
        Ok(_) => {}
        Err(rej) => {
            sess.rejections += 1;
            stages.push(stage_span("admit", t0));
            return StepOut {
                op: "query",
                outcome: "diags",
                cache,
                compute_calls: 0,
                diagnostics: vec![DiagOut {
                    code: format!("admission/{rej:?}")
                        .split_whitespace()
                        .next()
                        .unwrap_or("admission")
                        .to_string(),
                    message: format!("{rej}"),
                }],
                stages,
                meter: meter(sess),
                pe_exit: None,
            };
        }
    }
    stages.push(stage_span("admit", t0));

    // callee object through the store (cache tier reported coarsely)
    let t0 = Instant::now();
    let key = autoabi_object_key(&defs, false);
    // F-797-R4-10 (R8): the produced-bytes budget is a CUMULATIVE cap,
    // checked BEFORE every producer AND re-checked after each
    // production — a production that pushes the session over the cap
    // ends THIS step as a resource outcome, so no later producer (the
    // C consumer compile) ever runs on an over-budget session.
    if over_budget(sess) {
        return resource_step(budget_msg(sess), compute_calls, sess);
    }
    let callee_obj = match produce_with_cache(&store, &key, || {
        compute_calls += 1;
        let obj = emit_callee_suite("abi-worker-callee", &defs, false)?;
        sess.produced_bytes += obj.len() as u64;
        Ok(obj)
    }) {
        Ok((obj, hit)) => {
            cache = if hit {
                "hit".into()
            } else {
                "published".into()
            };
            obj
        }
        Err(e) => return infra_step(format!("callee produce: {e:?}"), compute_calls, sess),
    };
    stages.push(stage_span("produce-callee", t0));
    if over_budget(sess) {
        return resource_step(budget_msg(sess), compute_calls, sess);
    }

    // F-797-R4-10 lifecycle wiring on the REAL path: the current callee
    // object gets a project ROOT (survives GC as the deliverable) and a
    // per-query LEASE (held for the duration of the request; released on
    // BOTH success and failure — a cancelled/failed query frees its
    // lease and never deletes delivered objects).
    let callee_art_id = store
        .get_artifact(crate::cache::LAYER_OBJECT_V01.id, &key)
        .ok()
        .flatten()
        .map(|a| a.artifact_id);
    let query_lease = callee_art_id.as_ref().and_then(|id| {
        store
            .acquire_lease("abi-worker-query", 60, Some(id))
            .ok()
            .map(|l| (id.clone(), l))
    });

    // per-leg consumer (the REAL pipeline; scratch per step)
    let t0 = Instant::now();
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let store_tag = sess
        .store_root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("store")
        .to_string();
    // scratch is namespaced by the STORE directory AND the worker PID:
    // sibling stores never share, and CONCURRENT workers over the SAME
    // store (one worker process per request) each own a private scratch
    // — without the PID, two workers' cl/link collide on the same
    // `consumer-once.obj` (os error 32, caught by the concurrent-GC
    // full-chain test).
    let scratch = sess
        .store_root
        .parent()
        .unwrap_or(&sess.store_root)
        .join(format!(
            "{store_tag}-abi-worker-scratch-{}-{}",
            std::process::id(),
            sess.step_index
        ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("worker scratch");
    // MixAlt carries EXACTLY MixV1 semantics — the frozen oracle holds.
    let expected = match deck.fn_decl() {
        Ok(d) => match d.body {
            BodyKind::MixW(w) => decl::oracle_mix_w(&deck.inputs, w),
            _ => decl::oracle_mix_v1(&deck.inputs),
        },
        Err(_) => decl::oracle_mix_v1(&deck.inputs),
    };
    let (objs, entry): (Vec<std::path::PathBuf>, &str);
    match sess.leg.as_str() {
        "a" => {
            let checks = [SuiteCheck::Mix {
                symbol: "p797_mix",
                inputs: deck.inputs,
                expected,
                swap_fh: false,
            }];
            let caller = match emit_caller_suite("abi-worker-caller", &checks) {
                Ok(c) => c,
                Err(e) => return infra_step(format!("caller emit: {e}"), compute_calls, sess),
            };
            let cp = scratch.join("caller.obj");
            std::fs::write(&cp, &caller).expect("write caller obj");
            let clp = scratch.join("callee.obj");
            std::fs::write(&clp, &callee_obj).expect("write callee obj");
            objs = vec![cp, clp];
            entry = "p797_entry";
        }
        "b" => {
            let checks = [SuiteCheck::Mix {
                symbol: "p797_cmix",
                inputs: deck.inputs,
                expected,
                swap_fh: false,
            }];
            let caller = match emit_caller_suite("abi-worker-caller", &checks) {
                Ok(c) => c,
                Err(e) => return infra_step(format!("caller emit: {e}"), compute_calls, sess),
            };
            let cp = scratch.join("caller.obj");
            std::fs::write(&cp, &caller).expect("write caller obj");
            let src = manifest.join("cside/interop_callee.c");
            // F-797-R4-07 (R8/R9) + F-797-R9-01: the C object is cached
            // by source identity under the CURRENT target/toolchain;
            // the observed dependency closure AND the search-domain
            // facts (src dir, include dir, listings) are stored beside
            // the object and re-verified on every hit.
            if over_budget(sess) {
                return resource_step(budget_msg(sess), compute_calls, sess);
            }
            let c_obj = match produce_c_object(&store, &src, &manifest.join("cside"), &[], || {
                compute_calls += 1;
                let obj = scratch.join("consumer-once.obj");
                let deps = crate::ctool::compile_c_with_deps(&src, &manifest.join("cside"), &obj)
                    .map_err(|e| format!("cl: {e}"))?;
                let bytes = std::fs::read(&obj).map_err(|e| format!("read obj: {e}"))?;
                sess.produced_bytes += bytes.len() as u64;
                Ok((bytes, deps))
            }) {
                Ok((b, _)) => b,
                Err(e) => {
                    return infra_step(format!("C consumer produce: {e:?}"), compute_calls, sess)
                }
            };
            if over_budget(sess) {
                return resource_step(budget_msg(sess), compute_calls, sess);
            }
            let cop = scratch.join("consumer.obj");
            std::fs::write(&cop, &c_obj).expect("write consumer obj");
            objs = vec![cp, cop];
            entry = "p797_entry";
        }
        "c" => {
            let src = manifest.join("cside/caller_main.c");
            if over_budget(sess) {
                return resource_step(budget_msg(sess), compute_calls, sess);
            }
            let c_obj = match produce_c_object(&store, &src, &manifest.join("cside"), &[], || {
                compute_calls += 1;
                let obj = scratch.join("consumer-once.obj");
                let deps = crate::ctool::compile_c_with_deps(&src, &manifest.join("cside"), &obj)
                    .map_err(|e| format!("cl: {e}"))?;
                let bytes = std::fs::read(&obj).map_err(|e| format!("read obj: {e}"))?;
                sess.produced_bytes += bytes.len() as u64;
                Ok((bytes, deps))
            }) {
                Ok((b, _)) => b,
                Err(e) => {
                    return infra_step(format!("C consumer produce: {e:?}"), compute_calls, sess)
                }
            };
            if over_budget(sess) {
                return resource_step(budget_msg(sess), compute_calls, sess);
            }
            let cop = scratch.join("consumer.obj");
            std::fs::write(&cop, &c_obj).expect("write consumer obj");
            let clp = scratch.join("callee.obj");
            std::fs::write(&clp, &callee_obj).expect("write callee obj");
            objs = vec![cop, clp];
            entry = "p797_c_entry";
        }
        other => {
            return infra_step(
                format!("unknown leg {other:?} (v1 freezes a/b/c)"),
                compute_calls,
                sess,
            )
        }
    }
    stages.push(stage_span("consumer", t0));

    // link + run + compare (ALWAYS real; the PE carries the oracle)
    let t0 = Instant::now();
    let exe = match link_pe(&scratch, &objs, entry) {
        Ok(e) => e,
        Err(e) => return infra_step(format!("link: {e}"), compute_calls, sess),
    };
    let pe = match run::run_pe(&exe) {
        Ok(p) => p,
        Err(e) => return infra_step(format!("run: {e}"), compute_calls, sess),
    };
    stages.push(stage_span("link-run", t0));

    // release the query lease (success path). F-797-R4-10 (R5): the
    // `current` root is REPLACED, not accumulated — remove_roots clears
    // the previous current markers, then the fresh deliverable is
    // registered; a failed query keeps the OLD current (never pins all
    // history). GC errors are COUNTED in the meter, never silently
    // swallowed.
    drop(query_lease);
    if pe.code == 0 {
        if let Err(e) = store.remove_roots("abi-worker", "current") {
            sess.store_rejections += 1;
            eprintln!("# abi-worker: root replacement remove failed: {e:?}");
        }
        if let Some(id) = &callee_art_id {
            if let Err(e) = store.register_root("abi-worker", id, "current") {
                sess.store_rejections += 1;
                eprintln!("# abi-worker: current root registration failed: {e:?}");
            }
        }
    }
    {
        let hot = dir_size(&sess.store_root.join("blocks"));
        // F-797-R4-10 (R8/R9): the hot-set budget GC. Water levels are
        // the frozen 8/6MiB by contract; the TEST seams
        // `ABI_TEST_GC_{HIGH,LOW}_BYTES` shrink them (and
        // `ABI_TEST_GC_EVERY_STEP` forces a pass per query) so REAL
        // reclamation above the water level is exercisable end to end —
        // the receipt (ran/deleted/freed) is accumulated into the meter,
        // making "GC actually reclaimed" assertable.
        let env_u64 = |k: &str| std::env::var(k).ok().and_then(|v| v.parse::<u64>().ok());
        let high = env_u64("ABI_TEST_GC_HIGH_BYTES").unwrap_or(8 * 1024 * 1024);
        let low = env_u64("ABI_TEST_GC_LOW_BYTES")
            .unwrap_or(6 * 1024 * 1024)
            .min(high * 3 / 4);
        if hot > high || std::env::var("ABI_TEST_GC_EVERY_STEP").is_ok() {
            match ac_artifact_probe::store::run_gc(
                &store,
                ac_artifact_probe::store::GcConfig { high, low },
            ) {
                Ok(rx) => {
                    sess.gc_runs += u32::from(rx.ran);
                    sess.gc_deleted_chunks += rx.deleted_chunks as u32;
                    sess.gc_freed_bytes += rx.freed_bytes;
                }
                Err(e) => {
                    sess.store_rejections += 1;
                    eprintln!("# abi-worker: budget GC failed: {e:?}");
                }
            }
        }
    }
    if pe.code == 0 {
        StepOut {
            op: "query",
            outcome: "ok",
            cache,
            compute_calls,
            diagnostics: diags,
            stages,
            meter: meter(sess),
            pe_exit: Some(pe.code),
        }
    } else if pe.code < 0 {
        // F-797-R4-03: a native crash (negative NTSTATUS exit, e.g.
        // 0xC0000005 access violation) is an INFRASTRUCTURE outcome —
        // never a language diagnostic and never an expected pass.
        infra_step(
            format!(
                "native PE crashed (exit {:#x}); out={:?} err={:?}",
                pe.code as u32, pe.stdout, pe.stderr
            ),
            compute_calls,
            sess,
        )
    } else {
        diags.push(DiagOut {
            code: "pe-exit".into(),
            message: format!(
                "PE exited {} (out={:?} err={:?})",
                pe.code, pe.stdout, pe.stderr
            ),
        });
        StepOut {
            op: "query",
            outcome: "diags",
            cache,
            compute_calls,
            diagnostics: diags,
            stages,
            meter: meter(sess),
            pe_exit: Some(pe.code),
        }
    }
}

fn meter(sess: &Session) -> MeterOut {
    let scratch_root = sess.store_root.parent().unwrap_or(&sess.store_root);
    let mut scratch_peak = 0u64;
    if let Ok(rd) = std::fs::read_dir(scratch_root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.contains("-abi-worker-scratch-"))
                .unwrap_or(false)
            {
                scratch_peak = scratch_peak.max(dir_size(&p));
            }
        }
    }
    MeterOut {
        scratch_peak_disk_bytes: scratch_peak,
        hot_bytes: dir_size(&sess.store_root.join("blocks")),
        rejections: sess.rejections + sess.store_rejections,
        gc_runs: sess.gc_runs,
        gc_deleted_chunks: sess.gc_deleted_chunks,
        gc_freed_bytes: sess.gc_freed_bytes,
    }
}

/// F-797-R4-10 (R5): a typed RESOURCE outcome (budgets), distinct from
/// infra — the parent reports both as infrastructure errors, never as
/// language verdicts.
fn resource_step(message: String, compute_calls: u32, sess: &Session) -> StepOut {
    StepOut {
        op: "query",
        outcome: "resource",
        cache: "-".into(),
        compute_calls,
        diagnostics: vec![DiagOut {
            code: "resource".into(),
            message,
        }],
        stages: vec![],
        meter: meter(sess),
        pe_exit: None,
    }
}

fn infra_step(message: String, compute_calls: u32, sess: &Session) -> StepOut {
    StepOut {
        op: "query",
        outcome: "infra",
        cache: "-".into(),
        compute_calls,
        diagnostics: vec![DiagOut {
            code: "infra".into(),
            message,
        }],
        stages: vec![],
        meter: meter(sess),
        pe_exit: None,
    }
}

fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let hay = String::from_utf8_lossy(src).to_string();
    let first = hay
        .find(from)
        .ok_or_else(|| format!("edit-source `from` not found: {from:?}"))?;
    let second = hay[first + from.len()..].find(from);
    if second.is_some() {
        return Err(format!(
            "edit-source `from` matches more than once: {from:?}"
        ));
    }
    let mut out = String::with_capacity(hay.len() + to.len());
    out.push_str(&hay[..first]);
    out.push_str(to);
    out.push_str(&hay[first + from.len()..]);
    Ok(out.into_bytes())
}

fn execute(req: &WorkerRequest) -> Result<OkOut, DiagOut> {
    if req.recipe != RECIPE {
        return Err(DiagOut {
            code: "recipe".into(),
            message: format!("worker implements {RECIPE}, got {}", req.recipe),
        });
    }
    use base64::Engine as _;
    let deck = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| DiagOut {
            code: "deck-b64".into(),
            message: format!("{e}"),
        })?;
    let mut sess = Session {
        deck,
        leg: req.leg.clone(),
        store_root: std::path::PathBuf::from(&req.store_root),
        step_index: 0,
        rejections: 0,
        store_rejections: 0,
        gc_runs: 0,
        gc_deleted_chunks: 0,
        gc_freed_bytes: 0,
        produced_bytes: 0,
        budget_cap: std::env::var("ABI_TEST_PRODUCED_BUDGET_BYTES")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(128 * 1024 * 1024),
    };
    std::fs::create_dir_all(&sess.store_root).map_err(|e| DiagOut {
        code: "store".into(),
        message: format!("{e}"),
    })?;
    let mut steps_out = Vec::new();
    let mut restart_pending = false;
    for st in &req.steps {
        match st.op.as_str() {
            "query" => {
                let out = execute_query(&mut sess);
                sess.step_index += 1;
                steps_out.push(out);
            }
            "edit-source" => {
                let from = st.from.as_deref().unwrap_or("");
                let to = st.to.as_deref().unwrap_or("");
                sess.deck = replace_once(&sess.deck, from, to).map_err(|m| DiagOut {
                    code: "edit-source".into(),
                    message: m,
                })?;
                steps_out.push(StepOut {
                    op: "edit-source",
                    outcome: "ok",
                    cache: "-".into(),
                    compute_calls: 0,
                    diagnostics: vec![],
                    stages: vec![],
                    meter: meter(&sess),
                    pe_exit: None,
                });
            }
            "restart" => {
                restart_pending = true;
                steps_out.push(StepOut {
                    op: "restart",
                    outcome: "ok",
                    cache: "-".into(),
                    compute_calls: 0,
                    diagnostics: vec![],
                    stages: vec![],
                    meter: meter(&sess),
                    pe_exit: None,
                });
                break;
            }
            other => {
                return Err(DiagOut {
                    code: "op".into(),
                    message: format!("unknown op {other:?}"),
                })
            }
        }
    }
    Ok(OkOut {
        recipe: RECIPE,
        steps: steps_out,
        restart_pending,
        worker_version: WORKER_VERSION.to_string(),
    })
}

/// Serve one session from stdin; exactly one response line on stdout;
/// exit 0. Malformed input is a controlled error response.
pub fn serve() -> i32 {
    let stdin = std::io::stdin();
    let mut rd = stdin.lock();

    // F-797-R4-10: EVERY session line is read through a capped reader —
    // the metadata bound is enforced while reading, before the full
    // line is materialized (never read_line-then-check).
    let begin_line = match read_capped(&mut rd) {
        Ok(Some(l)) => l,
        Ok(None) => return fail_protocol("begin line missing"),
        Err(observed) => {
            return fail_protocol(&format!(
                "begin line exceeds the {}-byte bound ({} observed)",
                MAX_LINE, observed
            ))
        }
    };
    let begin: SessionBegin = match serde_json::from_str(begin_line.trim()) {
        Ok(b) => b,
        Err(e) => return fail_protocol(&format!("bad begin: {e}")),
    };
    if begin.op != "begin" || begin.schema != WORKER_SCHEMA {
        return fail_protocol("bad begin (expected op=begin, schema=abiworker/1)");
    }

    let req_line = match read_capped(&mut rd) {
        Ok(Some(l)) => l,
        Ok(None) => return fail_protocol("request line missing"),
        Err(observed) => {
            return fail_protocol(&format!(
                "request line exceeds the {}-byte bound ({} observed)",
                MAX_LINE, observed
            ))
        }
    };
    #[derive(Deserialize)]
    struct Envelope {
        op: String,
        request: WorkerRequest,
    }
    let env: Envelope = match serde_json::from_str(req_line.trim()) {
        Ok(e) => e,
        Err(e) => return fail_protocol(&format!("request malformed: {e}")),
    };
    if env.op != "request" {
        return fail_protocol("expected op=request");
    }

    let end_line = match read_capped(&mut rd) {
        Ok(Some(l)) => l,
        Ok(None) => return fail_protocol("end line missing"),
        Err(observed) => {
            return fail_protocol(&format!(
                "end line exceeds the {}-byte bound ({} observed)",
                MAX_LINE, observed
            ))
        }
    };
    if !end_line.contains(r#""op":"end""#) && !end_line.contains(r#""op": "end""#) {
        return fail_protocol("end line malformed");
    }

    let (nonce, request_hash) = (env.request.nonce.clone(), env.request.request_hash.clone());
    let resp = match execute(&env.request) {
        Ok(ok) => ResponseEnvelope {
            schema: WORKER_SCHEMA,
            nonce,
            request_hash,
            ok: Some(ok),
            error: None,
        },
        Err(err) => ResponseEnvelope {
            schema: WORKER_SCHEMA,
            nonce,
            request_hash,
            ok: None,
            error: Some(err),
        },
    };
    // F-797-R4-03 TEST-ONLY fault seams: never set by production paths;
    // the parent's lifecycle/admission tests drive them through the
    // child environment to prove the PARENT actually enforces the wall,
    // the stderr cap after the response, and the one-envelope rule.
    let mut out = std::io::stdout().lock();
    let line = if std::env::var("ABI_TEST_BOTH_ENVELOPE").is_ok() {
        // deliberately contradictory envelope (both ok and error)
        let mut v = serde_json::to_value(&resp).unwrap_or_default();
        v["error"] = serde_json::json!({"code": "fault", "message": "both"});
        v.to_string()
    } else {
        serde_json::to_string(&resp).unwrap_or_default()
    };
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    if let Ok(secs) = std::env::var("ABI_TEST_LATE_STDERR_OVERFLOW_MS") {
        let ms: u64 = secs.parse().unwrap_or(150);
        std::thread::sleep(std::time::Duration::from_millis(ms));
        let mut sink = std::io::stderr().lock();
        let chunk = vec![b'x'; 1024];
        for _ in 0..((MAX_LINE / 1024) + 8) {
            let _ = sink.write_all(&chunk);
        }
        let _ = sink.flush();
    }
    // F-797-R4-03 (R5) combined seam: overflow stderr and KEEP RUNNING
    // — proves the parent's exit loop polls the overflow while the tree
    // is still alive (single-fault seams cannot show this).
    if let Ok(hang) = std::env::var("ABI_TEST_OVERFLOW_THEN_HANG_SECS") {
        let secs: u64 = hang.parse().unwrap_or(10);
        let mut sink = std::io::stderr().lock();
        let chunk = vec![b'x'; 1024];
        for _ in 0..((MAX_LINE / 1024) + 8) {
            let _ = sink.write_all(&chunk);
        }
        let _ = sink.flush();
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
    if let Ok(hang) = std::env::var("ABI_TEST_HANG_AFTER_RESPONSE_SECS") {
        let secs: u64 = hang.parse().unwrap_or(61);
        std::thread::sleep(std::time::Duration::from_secs(secs));
    }
    0
}

/// F-797-R4-10: read ONE line with a hard byte cap; Err(observed_len)
/// reports an oversized line the moment the cap is crossed.
fn read_capped(rd: &mut std::io::StdinLock) -> Result<Option<String>, usize> {
    let mut line = Vec::new();
    let mut byte = [0u8; 1];
    loop {
        match rd.read(&mut byte) {
            Ok(0) => {
                if line.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
            Ok(_) => {
                if byte[0] == b'\n' {
                    return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
                }
                line.push(byte[0]);
                if line.len() > MAX_LINE {
                    // keep counting to report the observed size, bounded
                    // by a small overshoot so a hostile stream cannot
                    // grow the counter unboundedly either
                    let mut observed = line.len();
                    while observed <= MAX_LINE + 4096 {
                        match rd.read(&mut byte) {
                            Ok(0) => break,
                            Ok(_) => observed += 1,
                            Err(_) => break,
                        }
                    }
                    return Err(observed);
                }
            }
            Err(_) => {
                if line.is_empty() {
                    return Ok(None);
                }
                return Ok(Some(String::from_utf8_lossy(&line).into_owned()));
            }
        }
    }
}

fn fail_protocol(msg: &str) -> i32 {
    let resp = ResponseEnvelope {
        schema: WORKER_SCHEMA,
        nonce: String::new(),
        request_hash: String::new(),
        ok: None,
        error: Some(DiagOut {
            code: "protocol".into(),
            message: msg.to_string(),
        }),
    };
    let mut out = std::io::stdout().lock();
    let line = serde_json::to_string(&resp).unwrap_or_default();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    1
}
