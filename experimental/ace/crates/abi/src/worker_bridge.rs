//! PLAN-937 CA-D1 (T-09): the `abi/ca-d1-bridge-v1` worker recipe — a
//! PARALLEL recipe on the same `abiworker/1` envelope, strictly
//! mutually exclusive with `abi/interop-v1` (deck grammar, legs, and
//! expectations are disjoint; a bridge deck on the old recipe — or the
//! reverse — is a named `deck`/`recipe` rejection, never a borrowed
//! PASS).
//!
//! Legs (frozen):
//! * `source`     — the source-export chain (V-02): real Auto source →
//!   C library fileset → CRT-closed compile → independent caller EXECUTES.
//! * `typed-self` / `typed-n2c` / `typed-c2n` — the typed peer chain's
//!   three legs (V-01/V-03).
//!
//! Discipline inherited unchanged from the 797 worker: one response
//! line, budgeted producers, real PE exits, edit-source applied
//! parent-side (the deck carries the current source), `compute_calls`
//! = REAL producer invocations (0 on a pure-hit query), `cache` =
//! hit iff zero producers ran.

use crate::bridge::{
    self, run_source_export, run_typed_chain, BridgeCounters, SourceExportRequest, TypedLeg,
    BRIDGE_TOTAL_DEADLINE,
};
use crate::worker::{DiagOut, MeterOut, OkOut, StageSpan, StepOut, WorkerRequest, MAX_LINE};

use ace_artifacts::store::ManagedStore;
use base64::Engine as _;

pub const BRIDGE_RECIPE: &str = "abi/ca-d1-bridge-v1";
pub const BRIDGE_DECK_HEADER: &str = "deck ace-ca-d1/1";
pub const BRIDGE_WORKER_VERSION: &str = crate::worker::WORKER_VERSION;

/// The bridge deck grammar (deny-unknown by construction):
///
/// ```text
/// deck ace-ca-d1/1
/// leg source
/// module t937_source
/// export sum as t937_sum
/// export div as t937_div
/// src {
/// fn sum(a int, b int) int { return a + b }
/// }
/// ```
///
/// typed legs carry only `deck` + `leg` (the typed declarations are the
/// frozen code set, exactly like the 797 frozen probe business).
#[derive(Debug, Clone)]
pub struct BridgeDeck {
    pub leg: String,
    pub module: String,
    pub exports: Vec<(String, String)>,
    pub src: String,
}

pub fn parse_bridge_deck(text: &str) -> Result<BridgeDeck, String> {
    let mut lines = text.lines().skip_while(|l| {
        let t = l.trim();
        t.is_empty() || t.starts_with('#')
    });
    let header = lines.next().unwrap_or("").trim();
    if header != BRIDGE_DECK_HEADER {
        return Err(format!(
            "deck header must be {BRIDGE_DECK_HEADER:?}, got {header:?}"
        ));
    }
    let mut deck = BridgeDeck {
        leg: String::new(),
        module: String::new(),
        exports: vec![],
        src: String::new(),
    };
    let mut in_src = false;
    let mut src_lines: Vec<String> = Vec::new();
    let mut saw_src_close = false;
    for line in lines {
        if in_src {
            if line.trim() == "}" {
                in_src = false;
                saw_src_close = true;
                continue;
            }
            src_lines.push(line.to_string());
            continue;
        }
        let t = line.trim();
        if t.is_empty() || t.starts_with('#') {
            continue;
        }
        if let Some(leg) = t.strip_prefix("leg ") {
            if !deck.leg.is_empty() {
                return Err("duplicate leg".into());
            }
            deck.leg = leg.trim().to_string();
        } else if let Some(m) = t.strip_prefix("module ") {
            deck.module = m.trim().to_string();
        } else if let Some(e) = t.strip_prefix("export ") {
            let parts: Vec<&str> = e.splitn(3, " as ").collect();
            if parts.len() != 2 {
                return Err(format!("export line must be `export <fn> as <sym>`: {t:?}"));
            }
            deck.exports.push((parts[0].trim().to_string(), parts[1].trim().to_string()));
        } else if t == "src {" {
            in_src = true;
        } else {
            return Err(format!("unknown deck line {t:?}"));
        }
    }
    match deck.leg.as_str() {
        "source" => {
            if !saw_src_close {
                return Err("src block not closed".into());
            }
            deck.src = src_lines.join("\n");
            if deck.module.is_empty() || deck.exports.is_empty() || deck.src.trim().is_empty() {
                return Err("source leg needs module, >=1 export and a non-empty src block".into());
            }
        }
        "typed-self" | "typed-n2c" | "typed-c2n" => {
            if !deck.module.is_empty() || !deck.exports.is_empty() || !src_lines.is_empty() {
                return Err("typed legs carry only deck+leg (frozen typed declarations)".into());
            }
        }
        other => return Err(format!("unknown leg {other:?}")),
    }
    Ok(deck)
}

fn meter_of(store: &ManagedStore, scratch_root: &std::path::Path) -> MeterOut {
    let space = store.space();
    let mut scratch_peak = 0u64;
    if let Ok(rd) = std::fs::read_dir(scratch_root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                let mut n = 0u64;
                fn walk(p: &std::path::Path, n: &mut u64) {
                    if let Ok(rd) = std::fs::read_dir(p) {
                        for e in rd.flatten() {
                            if let Ok(md) = e.metadata() {
                                if md.is_dir() {
                                    walk(&e.path(), n);
                                } else {
                                    *n += md.len();
                                }
                            }
                        }
                    }
                }
                walk(&p, &mut n);
                scratch_peak = scratch_peak.max(n);
            }
        }
    }
    MeterOut {
        scratch_peak_disk_bytes: scratch_peak,
        hot_bytes: space.total.saturating_sub(space.db_main + space.locks + space.logs),
        rejections: 0,
        gc_runs: 0,
        gc_deleted_chunks: 0,
        gc_freed_bytes: 0,
    }
}

fn diag_step(code: &str, message: String, op: &'static str, meter: MeterOut) -> StepOut {
    StepOut {
        op,
        outcome: "diags",
        cache: "-".into(),
        compute_calls: 0,
        diagnostics: vec![DiagOut {
            code: code.into(),
            message,
        }],
        stages: vec![],
        meter,
        pe_exit: None,
    }
}

fn infra_step(message: String, meter: MeterOut) -> StepOut {
    StepOut {
        op: "query",
        outcome: "infra",
        cache: "-".into(),
        compute_calls: 0,
        diagnostics: vec![DiagOut {
            code: "infra".into(),
            message,
        }],
        stages: vec![],
        meter,
        pe_exit: None,
    }
}

fn resource_step(message: String, meter: MeterOut) -> StepOut {
    StepOut {
        op: "query",
        outcome: "resource",
        cache: "-".into(),
        compute_calls: 0,
        diagnostics: vec![DiagOut {
            code: "resource".into(),
            message,
        }],
        stages: vec![],
        meter,
        pe_exit: None,
    }
}

fn bridge_query_step(sess: &mut BridgeSession) -> StepOut {
    let started = std::time::Instant::now();
    let deck = match parse_bridge_deck(&sess.deck_text) {
        Ok(d) => d,
        Err(e) => return diag_step("deck", e, "query", sess.meter()),
    };
    let store = match ManagedStore::open(std::path::Path::new(&sess.store_root)) {
        Ok(s) => s,
        Err(e) => return infra_step(format!("store open: {e:?}"), sess.meter()),
    };
    let counters = BridgeCounters::new();
    let scratch_root = std::path::Path::new(&sess.store_root).join("bridge-scratch");
    let res = match deck.leg.as_str() {
        "source" => run_source_export(
            &store,
            &counters,
            &scratch_root,
            &SourceExportRequest {
                src: deck.src.clone(),
                module: deck.module.clone(),
                exports: deck.exports.clone(),
            },
            BRIDGE_TOTAL_DEADLINE,
        )
        .map(|o| (o.caller_exit, counters.clone())),
        leg => {
            let tl = match leg {
                "typed-self" => TypedLeg::SelfCheck,
                "typed-n2c" => TypedLeg::NativeToC,
                "typed-c2n" => TypedLeg::CToNative,
                _ => unreachable!("parser validated the leg"),
            };
            run_typed_chain(&store, &counters, &scratch_root, &[tl], BRIDGE_TOTAL_DEADLINE)
                .map(|o| (o.legs.first().map(|l| l.1).unwrap_or(-1), counters.clone()))
        }
    };
    let (pe_exit, counters) = match res {
        Ok(v) => v,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("budget exhausted") {
                return resource_step(msg, sess.meter());
            }
            if msg.contains("admission rejected") {
                return diag_step("admission/bridge", msg, "query", sess.meter());
            }
            return infra_step(msg, sess.meter());
        }
    };
    let compute = counters.total_producer_calls().min(u32::MAX as u64) as u32;
    sess.last_compute = compute;
    // Exit-code discipline (the 797 contract): 0 = ok; a POSITIVE code
    // (failed in-PE checks) is a `pe-exit` DIAGNOSTIC — never a green
    // step; a negative NTSTATUS is a crash -> infra.
    if pe_exit < 0 {
        return infra_step(format!("bridge PE crashed (exit {pe_exit})"), sess.meter());
    }
    if pe_exit != 0 {
        let mut out = diag_step(
            "pe-exit",
            format!("bridge PE exited {pe_exit} (failed in-PE checks)"),
            "query",
            sess.meter(),
        );
        out.compute_calls = compute;
        out.cache = if compute == 0 { "hit".into() } else { "published".into() };
        return out;
    }
    StepOut {
        op: "query",
        outcome: "ok",
        cache: if compute == 0 { "hit".into() } else { "published".into() },
        compute_calls: compute,
        diagnostics: vec![],
        stages: vec![StageSpan {
            name: "bridge".into(),
            wall_ns: started.elapsed().as_nanos(),
        }],
        meter: sess.meter(),
        pe_exit: Some(pe_exit),
    }
}

struct BridgeSession {
    deck_text: String,
    store_root: String,
    last_compute: u32,
    _scratch_tag: String,
}

impl BridgeSession {
    fn meter(&self) -> MeterOut {
        match ManagedStore::open(std::path::Path::new(&self.store_root)) {
            Ok(s) => {
                let scratch = std::path::Path::new(&self.store_root).join("bridge-scratch");
                meter_of(&s, &scratch)
            }
            Err(_) => MeterOut {
                scratch_peak_disk_bytes: 0,
                hot_bytes: 0,
                rejections: 0,
                gc_runs: 0,
                gc_deleted_chunks: 0,
                gc_freed_bytes: 0,
            },
        }
    }
}

/// Execute a `abi/ca-d1-bridge-v1` request (same step vocabulary:
/// query / edit-source / restart; edits apply to the deck TEXT like the
/// 797 protocol).
pub fn execute_bridge(req: &WorkerRequest) -> Result<OkOut, DiagOut> {
    if req.recipe != BRIDGE_RECIPE {
        return Err(DiagOut {
            code: "recipe".into(),
            message: format!("bridge worker implements {BRIDGE_RECIPE}, got {}", req.recipe),
        });
    }
    // the request leg (from the parent binding) must AGREE with the
    // deck leg when supplied — a mismatched binding is a named rejection
    // before any work
    let deck_text_probe = String::from_utf8_lossy(
        &base64::engine::general_purpose::STANDARD
            .decode(&req.deck_b64)
            .unwrap_or_default(),
    )
    .into_owned();
    if let Ok(d) = parse_bridge_deck(&deck_text_probe) {
        if !req.leg.is_empty() && req.leg != d.leg {
            return Err(DiagOut {
                code: "leg".into(),
                message: format!(
                    "binding leg {:?} does not match deck leg {:?}",
                    req.leg, d.leg
                ),
            });
        }
    }
    let deck = base64::engine::general_purpose::STANDARD
        .decode(&req.deck_b64)
        .map_err(|e| DiagOut {
            code: "deck-b64".into(),
            message: format!("{e}"),
        })?;
    if deck.len() > MAX_LINE {
        return Err(DiagOut {
            code: "deck-b64".into(),
            message: format!("deck {} bytes exceeds {} bound", deck.len(), MAX_LINE),
        });
    }
    // NOTE: no upfront deck parse — deck diagnostics surface as STEP
    // diags (code "deck") inside the query, exactly like the 797 recipe,
    // so negative cases complete as parse-stage errors, not protocol
    // failures. Mutual exclusion with the 797 grammar happens inside
    // the parser via the header check.
    let deck_text = String::from_utf8_lossy(&deck).into_owned();
    std::fs::create_dir_all(&req.store_root).map_err(|e| DiagOut {
        code: "store".into(),
        message: format!("{e}"),
    })?;
    let mut sess = BridgeSession {
        deck_text,
        store_root: req.store_root.clone(),
        last_compute: 0,
        _scratch_tag: format!("bridge-{}", std::process::id()),
    };
    let mut steps_out = Vec::new();
    let mut restart_pending = false;
    for st in &req.steps {
        match st.op.as_str() {
            "query" => steps_out.push(bridge_query_step(&mut sess)),
            "edit-source" => {
                let from = st.from.as_deref().unwrap_or("");
                let to = st.to.as_deref().unwrap_or("");
                let hay = sess.deck_text.clone();
                let first = hay.find(from).ok_or_else(|| DiagOut {
                    code: "edit-source".into(),
                    message: format!("edit-source `from` not found: {from:?}"),
                })?;
                if hay[first + from.len()..].contains(from) {
                    return Err(DiagOut {
                        code: "edit-source".into(),
                        message: format!("edit-source `from` matches more than once: {from:?}"),
                    });
                }
                sess.deck_text = format!("{}{}{}", &hay[..first], to, &hay[first + from.len()..]);
                steps_out.push(StepOut {
                    op: "edit-source",
                    outcome: "ok",
                    cache: "-".into(),
                    compute_calls: 0,
                    diagnostics: vec![],
                    stages: vec![],
                    meter: sess.meter(),
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
                    meter: sess.meter(),
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
        recipe: BRIDGE_RECIPE,
        steps: steps_out,
        restart_pending,
        worker_version: BRIDGE_WORKER_VERSION.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_deck_parses() {
        let deck = "deck ace-ca-d1/1\nleg source\nmodule m\nexport sum as t937_sum\nsrc {\nfn sum(a int, b int) int { return a + b }\n}\n";
        let d = parse_bridge_deck(deck).unwrap();
        assert_eq!(d.leg, "source");
        assert_eq!(d.exports, vec![("sum".to_string(), "t937_sum".to_string())]);
        assert!(d.src.contains("fn sum"));
    }

    #[test]
    fn typed_deck_is_minimal_and_legs_frozen() {
        for leg in ["typed-self", "typed-n2c", "typed-c2n"] {
            let d = parse_bridge_deck(&format!("deck ace-ca-d1/1\nleg {leg}\n")).unwrap();
            assert_eq!(d.leg, leg);
        }
    }

    #[test]
    fn old_recipe_deck_is_rejected_on_the_bridge_parser() {
        let old = "deck autoabi/1\nfn p797_mix(...)";
        assert!(parse_bridge_deck(old).unwrap_err().contains("deck header"));
    }

    #[test]
    fn unknown_lines_and_unclosed_src_rejected() {
        assert!(parse_bridge_deck("deck ace-ca-d1/1\nleg source\nbogus x\n").is_err());
        assert!(parse_bridge_deck("deck ace-ca-d1/1\nleg source\nmodule m\nexport a as b\nsrc {\nfn a() int").is_err());
        assert!(parse_bridge_deck("deck ace-ca-d1/1\nleg nope\n").is_err());
        // source leg needs exports+src
        assert!(parse_bridge_deck("deck ace-ca-d1/1\nleg source\nmodule m\n").is_err());
    }
}
