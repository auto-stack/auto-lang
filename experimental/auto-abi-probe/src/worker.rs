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

use crate::cache::{autoabi_object_key, c_object_key, produce_with_cache};
use crate::decl::{self, AbiScalar, BodyKind, FnDecl, MixInputs};
use crate::emit::{emit_callee_suite, emit_caller_suite, SuiteCheck};
use crate::layout::{self, REC_MIX};
use crate::link::link_pe;
use crate::run;
use ac_artifact_probe::store::ManagedStore;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
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
    pub scratch_peak_bytes: u64,
    pub hot_bytes: u64,
    pub rejections: u32,
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
            return Err(format!("deck must start with `deck autoabi/1`, got {line:?}"));
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
    let rest = fn_line
        .strip_prefix("fn ")
        .ok_or("bad fn line")?;
    let (sig, body_tag) = rest
        .split_once("[body:")
        .ok_or("fn line needs a `[body: ...]` tag")?;
    let body = body_tag.trim_end_matches(']').trim().to_string();
    if !matches!(body.as_str(), "mix-v1" | "mix-v1b") {
        return Err(format!("unknown body {:?} (v1 freezes mix-v1/mix-v1b)", body));
    }
    let sig = sig.trim();
    let (name, sig) = sig
        .split_once('(')
        .ok_or("fn line needs `(params)`")?;
    if name.trim() != "p797_mix" {
        return Err(format!("v1 freezes the single function p797_mix, got {name:?}"));
    }
    let (params_paren, ret_txt) = sig
        .rsplit_once("->")
        .ok_or("fn line needs `) -> <ty>`")?;
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
        let (n, t) = p.split_once(':').ok_or_else(|| format!("param {p:?} needs `name:ty`"))?;
        params.push((n.trim().to_string(), t.trim().to_string()));
    }
    for (n, t) in &params {
        if !matches!(t.as_str(), "i32" | "u32" | "i64" | "u64" | "f64" | "bool" | "char") {
            return Err(format!("param {n} has unknown type {t:?}"));
        }
    }
    if !matches!(ret.as_str(), "i32" | "u32" | "i64" | "u64" | "f64" | "bool" | "char") {
        return Err(format!("unknown return type {ret:?}"));
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
        let (n, v) = part.split_once(':').ok_or_else(|| format!("input {part:?} needs `name: value`"))?;
        vals.push((n.trim().to_string(), v.trim().to_string()));
    }
    let get = |name: &str, parse_int: bool| -> Result<String, String> {
        vals.iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.clone())
            .ok_or_else(|| format!("inputs missing {name:?}"))
            .and_then(|v| {
                let (ok, kind) = if parse_int {
                    (v.parse::<i64>().is_ok(), "integer")
                } else {
                    (v.parse::<f64>().is_ok(), "float")
                };
                if ok {
                    Ok(v)
                } else {
                    Err(format!("input {name} is not a {kind}: {v}"))
                }
            })
    };
    let inputs = MixInputs {
        a: get("a", true)?.parse().unwrap(),
        b: get("b", true)?.parse().unwrap(),
        c: get("c", true)?.parse().unwrap(),
        d: get("d", true)?.parse().unwrap(),
        e: get("e", true)?.parse().unwrap(),
        f: get("f", false)?.parse().unwrap(),
        g: get("g", true)?.parse().unwrap(),
        h: get("h", false)?.parse().unwrap(),
    };
    let _ = rec_line; // rec is declaration-only in v1 (layout fixture);
                      // presence/absence does not change the pipeline
    Ok(Deck { params, ret, body, inputs })
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
                self.params.iter().map(|(_, t)| t.as_str()).collect::<Vec<_>>()
            ));
        }
        if map(&self.ret) != AbiScalar::I64 {
            return Err(format!("v1 freezes the i64 return; deck spells {:?}", self.ret));
        }
        Ok(FnDecl {
            symbol: "p797_mix",
            params: decl::MIX_PARAMS,
            ret: AbiScalar::I64,
            body: if self.body == "mix-v1b" {
                BodyKind::MixV1Alt
            } else {
                BodyKind::MixV1
            },
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
}

fn dir_size(p: &std::path::Path) -> u64 {
    let mut total = 0u64;
    if let Ok(rd) = std::fs::read_dir(p) {
        for e in rd.flatten() {
            let Ok(md) = e.metadata() else { continue };
            if md.is_file() {
                total += md.len();
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

fn execute_query(sess: &mut Session) -> StepOut {
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
                diagnostics: vec![DiagOut { code: "deck".into(), message: e }],
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
                diagnostics: vec![DiagOut { code: "deck".into(), message: e }],
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
        Err(e) => return infra_step(format!("store open: {e:?}"), sess),
    };
    let computed = layout::layout_record(&REC_MIX).expect("frozen layout");
    if layout::get_layout(&store, &REC_MIX)
        .expect("layout get")
        .is_none()
    {
        layout::put_layout(&store, &REC_MIX, &computed).expect("layout put");
    }
    stages.push(stage_span("layout", t0));

    // admission: deck fn signature vs the frozen consumer expectation —
    // the descriptor-edit reject path lands HERE (typed diags).
    let t0 = Instant::now();
    let defs = [fn_decl];
    let produced = match crate::build::produce_callee_suite("abi-worker-callee", &defs, false) {
        Ok(p) => p,
        Err(e) => return infra_step(format!("produce: {e}"), sess),
    };
    let descriptor_bytes = match produced.descriptor.encode() {
        Ok(b) => b,
        Err(e) => return infra_step(format!("descriptor encode: {e}"), sess),
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
                    code: format!("admission/{rej:?}").split_whitespace().next().unwrap_or("admission").to_string(),
                    message: format!("{rej}"),
                }],
                stages,
                meter: meter(sess),
                pe_exit: None,
            }
        }
    }
    stages.push(stage_span("admit", t0));

    // callee object through the store (cache tier reported coarsely)
    let t0 = Instant::now();
    let key = autoabi_object_key(&defs, false);
    let callee_obj = match produce_with_cache(&store, &key, || {
        compute_calls += 1;
        emit_callee_suite("abi-worker-callee", &defs, false)
    }) {
        Ok((obj, hit)) => {
            cache = if hit { "hit".into() } else { "published".into() };
            obj
        }
        Err(e) => return infra_step(format!("callee produce: {e:?}"), sess),
    };
    stages.push(stage_span("produce-callee", t0));

    // per-leg consumer (the REAL pipeline; scratch per step)
    let t0 = Instant::now();
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let scratch = sess
        .store_root
        .parent()
        .unwrap_or(&sess.store_root)
        .join(format!("abi-worker-scratch-{}", sess.step_index));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("worker scratch");
    let expected = decl::oracle_mix_v1(&deck.inputs);
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
                Err(e) => return infra_step(format!("caller emit: {e}"), sess),
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
                Err(e) => return infra_step(format!("caller emit: {e}"), sess),
            };
            let cp = scratch.join("caller.obj");
            std::fs::write(&cp, &caller).expect("write caller obj");
            let src = manifest.join("cside/interop_callee.c");
            let content = std::fs::read(&src).expect("cside source");
            let key = c_object_key(&crate::semref::blake3_hex(&content));
            let c_obj = match produce_with_cache(&store, &key, || {
                compute_calls += 1;
                let obj = scratch.join("consumer-once.obj");
                crate::ctool::compile_c(&src, &manifest.join("cside"), &obj)
                    .map_err(|e| format!("cl: {e}"))?;
                std::fs::read(&obj).map_err(|e| format!("read obj: {e}"))
            }) {
                Ok((b, _)) => b,
                Err(e) => return infra_step(format!("C consumer produce: {e:?}"), sess),
            };
            let cop = scratch.join("consumer.obj");
            std::fs::write(&cop, &c_obj).expect("write consumer obj");
            objs = vec![cp, cop];
            entry = "p797_entry";
        }
        "c" => {
            let src = manifest.join("cside/caller_main.c");
            let content = std::fs::read(&src).expect("cside source");
            let key = c_object_key(&crate::semref::blake3_hex(&content));
            let c_obj = match produce_with_cache(&store, &key, || {
                compute_calls += 1;
                let obj = scratch.join("consumer-once.obj");
                crate::ctool::compile_c(&src, &manifest.join("cside"), &obj)
                    .map_err(|e| format!("cl: {e}"))?;
                std::fs::read(&obj).map_err(|e| format!("read obj: {e}"))
            }) {
                Ok((b, _)) => b,
                Err(e) => return infra_step(format!("C consumer produce: {e:?}"), sess),
            };
            let cop = scratch.join("consumer.obj");
            std::fs::write(&cop, &c_obj).expect("write consumer obj");
            let clp = scratch.join("callee.obj");
            std::fs::write(&clp, &callee_obj).expect("write callee obj");
            objs = vec![cop, clp];
            entry = "p797_c_entry";
        }
        other => {
            return infra_step(format!("unknown leg {other:?} (v1 freezes a/b/c)"), sess)
        }
    }
    stages.push(stage_span("consumer", t0));

    // link + run + compare (ALWAYS real; the PE carries the oracle)
    let t0 = Instant::now();
    let exe = match link_pe(&scratch, &objs, entry) {
        Ok(e) => e,
        Err(e) => return infra_step(format!("link: {e}"), sess),
    };
    let pe = match run::run_pe(&exe) {
        Ok(p) => p,
        Err(e) => return infra_step(format!("run: {e}"), sess),
    };
    stages.push(stage_span("link-run", t0));

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
    } else {
        diags.push(DiagOut {
            code: "pe-exit".into(),
            message: format!("PE exited {} (out={:?} err={:?})", pe.code, pe.stdout, pe.stderr),
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
                .map(|n| n.starts_with("abi-worker-scratch-"))
                .unwrap_or(false)
            {
                scratch_peak = scratch_peak.max(dir_size(&p));
            }
        }
    }
    MeterOut {
        scratch_peak_bytes: scratch_peak,
        hot_bytes: dir_size(&sess.store_root.join("blocks")),
        rejections: sess.rejections,
    }
}

fn infra_step(message: String, sess: &Session) -> StepOut {
    StepOut {
        op: "query",
        outcome: "infra",
        cache: "-".into(),
        compute_calls: 0,
        diagnostics: vec![DiagOut { code: "infra".into(), message }],
        stages: vec![],
        meter: meter(sess),
        pe_exit: None,
    }
}

fn replace_once(src: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let hay = String::from_utf8_lossy(src).to_string();
    let first = hay.find(from).ok_or_else(|| format!("edit-source `from` not found: {from:?}"))?;
    let second = hay[first + from.len()..].find(from);
    if second.is_some() {
        return Err(format!("edit-source `from` matches more than once: {from:?}"));
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
        .map_err(|e| DiagOut { code: "deck-b64".into(), message: format!("{e}") })?;
    let mut sess = Session {
        deck,
        leg: req.leg.clone(),
        store_root: std::path::PathBuf::from(&req.store_root),
        step_index: 0,
        rejections: 0,
    };
    std::fs::create_dir_all(&sess.store_root)
        .map_err(|e| DiagOut { code: "store".into(), message: format!("{e}") })?;
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

    let mut begin_line = String::new();
    if rd.read_line(&mut begin_line).unwrap_or(0) == 0 {
        return fail_protocol("begin line missing");
    }
    let begin: SessionBegin = match serde_json::from_str(begin_line.trim()) {
        Ok(b) => b,
        Err(e) => return fail_protocol(&format!("bad begin: {e}")),
    };
    if begin.op != "begin" || begin.schema != WORKER_SCHEMA {
        return fail_protocol("bad begin (expected op=begin, schema=abiworker/1)");
    }

    let mut req_line = String::new();
    let n = rd.read_line(&mut req_line).unwrap_or(0);
    if n == 0 {
        return fail_protocol("request line missing");
    }
    if req_line.len() > MAX_LINE {
        return fail_protocol("request line exceeds the 4MiB bound");
    }
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

    let mut end_line = String::new();
    if rd.read_line(&mut end_line).unwrap_or(0) == 0 {
        return fail_protocol("end line missing");
    }
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
    let mut out = std::io::stdout().lock();
    let line = serde_json::to_string(&resp).unwrap_or_default();
    let _ = writeln!(out, "{line}");
    let _ = out.flush();
    0
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
