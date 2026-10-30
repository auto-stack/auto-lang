//! PLAN-788 T-06: worker-side syntax adapter over the merged PLAN-787
//! ac-syntax runtime (D-787 satisfied — bound against the real merged API,
//! never copied from its in-flight tree).
//!
//! Contract (contract-decision §11):
//! - `parse`: full parse under the chosen grammar; observables are the
//!   lossless reconstruction, the outline projection and the canonical CST
//!   form; diagnostics are typed (stage `parse`, code split by REAL source
//!   lex vs tree, spans absolute).
//! - `edit`: ordered single-edit steps; EACH step applies one edit,
//!   reparses INCREMENTALLY, and the worker PROVES fresh↔incremental
//!   equivalence (canonical tree, diagnostics count, lossless
//!   reconstruction) — a mismatch is an infrastructure error, never a
//!   silently divergent projection. Final observables belong to the final
//!   revision; rescan/reparse/reuse counters and fallback counts are the
//!   real runtime numbers.

use crate::protocol::{
    ArtifactOut, DiagOut, OkOut, SpanOut, ToolchainOut, WorkerError, WorkerRequest,
};
use ace_syntax_grammar;
use ace_syntax_runtime::cst;
use ace_syntax_runtime::edit::incremental_reparse;
use ace_syntax_runtime::lexer::CompiledScanner;
use ace_syntax_runtime::parse::{parse_snapshot, ParseOutput};
use ace_syntax_runtime::program::Budget;
use ace_syntax_runtime::snapshot::{Edit, EditRequest, Snapshot};
use base64::Engine;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

const AUTO_CORE: &str = include_str!("../../../grammars/auto-core.at");
const INTERFACE_MINI: &str = include_str!("../../../grammars/interface-mini-v1.at");

thread_local! {
    static CORE: RefCell<(Rc<ace_syntax_runtime::program::GrammarProgram>, CompiledScanner)> =
        RefCell::new(compile_pair(AUTO_CORE));
    static MINI: RefCell<(Rc<ace_syntax_runtime::program::GrammarProgram>, CompiledScanner)> =
        RefCell::new(compile_pair(INTERFACE_MINI));
}

fn compile_pair(
    src: &'static str,
) -> (
    Rc<ace_syntax_runtime::program::GrammarProgram>,
    CompiledScanner,
) {
    let p = ace_syntax_grammar::compile(src).expect("embedded grammar must compile");
    let cs = CompiledScanner::compile(&p).expect("embedded scanner must compile");
    (Rc::new(p), cs)
}

/// Everything the runtime owns is Rc/RefCell — one grammar per thread is
/// the worker's shape (single-threaded request handling).
fn with_grammar<R>(
    name: &str,
    f: impl FnOnce(&Rc<ace_syntax_runtime::program::GrammarProgram>, &CompiledScanner) -> R,
) -> R {
    match name {
        "interface-mini-v1" => MINI.with(|c| {
            let c = c.borrow();
            f(&c.0, &c.1)
        }),
        _ => CORE.with(|c| {
            let c = c.borrow();
            f(&c.0, &c.1)
        }),
    }
}

pub fn execute(req: &WorkerRequest) -> Result<OkOut, WorkerError> {
    let started = Instant::now();
    let grammar = req.grammar.as_deref().unwrap_or("auto-core");
    with_grammar(grammar, |p, cs| exec_inner(req, grammar, p, cs, started))
}

fn exec_inner(
    req: &WorkerRequest,
    grammar: &str,
    p: &Rc<ace_syntax_runtime::program::GrammarProgram>,
    cs: &CompiledScanner,
    started: Instant,
) -> Result<OkOut, WorkerError> {
    // R2 (F07): the request total budget bounds the WHOLE syntax request —
    // parse, every edit step, the fresh-equivalence oracle, projections
    // and artifact writes. runtime Budget::default is an internal resource
    // policy, never a substitute for this deadline.
    let total = req.budget.total_ms;
    let check_budget = |phase: &str| -> Result<(), WorkerError> {
        if started.elapsed().as_millis() as u64 > total {
            return Err(infra(
                "budget.total",
                format!(
                    "syntax {} phase exceeded the {}ms request budget",
                    phase, total
                ),
            ));
        }
        Ok(())
    };
    let input = match &req.input {
        crate::protocol::InputSpec::Source { source_b64 } => {
            b64d(source_b64).map_err(|e| infra("request.input", format!("base64: {}", e)))?
        }
        _ => {
            return Err(infra(
                "request.input",
                "syntax requests take auto source text",
            ))
        }
    };
    let src = String::from_utf8(input)
        .map_err(|_| infra("request.input", "syntax source must be UTF-8 text"))?;
    let store = Rc::new(ace_syntax_runtime::snapshot::PageStore::new());
    let mut snap = Snapshot::from_text(&store, &src);

    // ---- full parse (both kinds) ----------------------------------------
    let t_parse = Instant::now();
    let cur: ParseOutput = parse_snapshot(p, cs, &snap, &Budget::default())
        .map_err(|e| infra("syntax.resource", format!("{:?}", e)))?;
    let mut trace = vec![
        SpanOut {
            stage: "lex".into(),
            wall_ms: ms(t_parse),
        },
        SpanOut {
            stage: "parse".into(),
            wall_ms: ms(t_parse),
        },
    ];
    let mut total_rescan: u64 = 0;
    let mut total_reparsed: usize = 0;
    let mut total_reused: usize = 0;
    let mut fallbacks: usize = 0;
    let mut revision: u32 = snap.revision();

    let mut cur = cur;
    check_budget("parse")?;
    if req.kind == "edit" {
        let steps = req.edits.as_deref().unwrap_or(&[]);
        for (i, step) in steps.iter().enumerate() {
            let t_step = Instant::now();
            let insert = b64d(&step.insert_b64)
                .map_err(|e| infra("request.edits", format!("edit[{}] base64: {}", i, e)))?;
            let insert = String::from_utf8(insert)
                .map_err(|_| infra("request.edits", format!("edit[{}] insert must be UTF-8", i)))?;
            let edit = Edit {
                start: step.start,
                end: step.end,
                insert: insert.into_bytes(),
            };
            let old_snap = snap.clone();
            snap = snap
                .edit(&EditRequest {
                    base_revision: snap.revision(),
                    edits: vec![edit.clone()],
                })
                .map_err(|e| {
                    infra(
                        "request.edits",
                        format!("edit[{}] rejected by the snapshot engine: {:?}", i, e),
                    )
                })?;
            let new_snap = snap.clone();
            let inc = incremental_reparse(
                p,
                cs,
                &cur,
                &old_snap,
                &new_snap,
                &[edit],
                &Budget::default(),
            )
            .map_err(|e| infra("syntax.resource", format!("edit[{}]: {:?}", i, e)))?;
            if inc.fallback.is_some() {
                fallbacks += 1;
            }
            total_rescan += inc.rescan_bytes;
            total_reparsed += inc.reparsed_decls;
            total_reused += inc.reused_decls;

            // Equivalence proof at EVERY step: incremental must equal a
            // fresh reparse (canonical form, diagnostics count, lossless).
            let fresh = parse_snapshot(p, cs, &new_snap, &Budget::default())
                .map_err(|e| infra("syntax.resource", format!("edit[{}] fresh: {:?}", i, e)))?;
            if inc.tree.canonical() != fresh.tree.canonical()
                || inc.tree.diags.len() != fresh.tree.diags.len()
                || inc.lex_diags.len() != fresh.lex_diags.len()
            {
                return Err(infra(
                    "syntax.inconsistent",
                    format!(
                        "edit[{}]: incremental result diverges from fresh reparse — never published",
                        i
                    ),
                ));
            }
            if cst::reconstruct(&inc.tree, &new_snap).map_err(|e| {
                infra(
                    "syntax.inconsistent",
                    format!("edit[{}] reconstruct: {:?}", i, e),
                )
            })? != new_snap
                .read_bytes(0, new_snap.len())
                .map_err(|e| infra("syntax.inconsistent", format!("edit[{}] read: {:?}", i, e)))?
            {
                return Err(infra(
                    "syntax.inconsistent",
                    format!("edit[{}]: reconstruction is not lossless", i),
                ));
            }
            // Continuous-session semantics: the next step's baseline is the
            // INCREMENTAL artifact (the 787 replay contract).
            cur = ParseOutput {
                tree: inc.tree,
                tokens: inc.tokens,
                lex_diags: inc.lex_diags,
                grafted_decls: inc.reused_decls,
            };
            revision = new_snap.revision();
            check_budget(&format!("edit-{}", i))?;
            trace.push(SpanOut {
                stage: format!("edit-{}", i),
                wall_ms: ms(t_step),
            });
        }
    }

    // ---- typed diagnostics ----------------------------------------------
    let mut diagnostics: Vec<DiagOut> = Vec::new();
    for d in &cur.lex_diags {
        diagnostics.push(DiagOut {
            stage: "parse".into(),
            code: "syntax.lex".into(),
            message: d.message.clone(),
            span_start: d.start as usize,
            span_end: d.end as usize,
        });
    }
    for d in &cur.tree.diags {
        diagnostics.push(DiagOut {
            stage: "parse".into(),
            code: "syntax.parse".into(),
            message: d.message.clone(),
            span_start: d.start as usize,
            span_end: d.end as usize,
        });
    }
    diagnostics.sort_by_key(|d| (d.span_start, d.span_end));

    // ---- projections of the FINAL revision -------------------------------
    let scratch = std::path::Path::new(&req.scratch);
    let reconstructed = cst::reconstruct(&cur.tree, &snap)
        .map_err(|e| infra("syntax.resource", format!("reconstruct: {:?}", e)))?;
    let outline_items = ace_syntax_runtime::outline::outline(p, &cur.tree, &snap)
        .map_err(|e| infra("syntax.resource", format!("outline: {:?}", e)))?;
    let canonical = cur.tree.canonical();
    let outline_json = render_outline(&outline_items);

    check_budget("projection")?;
    let mut artifacts = Vec::new();
    let mut write_artifact = |logical: &str, bytes: &[u8]| -> Result<(), WorkerError> {
        let path = scratch.join(logical);
        std::fs::write(&path, bytes)
            .map_err(|e| infra("request.scratch", format!("write {}: {}", logical, e)))?;
        let sha = crate::exec::sha256_hex_public(bytes);
        artifacts.push(ArtifactOut {
            kind: "syntax".into(),
            logical: logical.to_string(),
            path: path.display().to_string(),
            bytes: bytes.len(),
            sha256: sha,
        });
        Ok(())
    };
    write_artifact("reconstructed.at", &reconstructed)?;
    write_artifact("outline.json", outline_json.as_bytes())?;
    write_artifact("cst.canonical", canonical.as_bytes())?;

    check_budget("done")?;
    trace.push(SpanOut {
        stage: "done".into(),
        wall_ms: ms(started),
    });

    Ok(OkOut {
        diagnostics,
        hir_b64: None,
        artifacts,
        run: None,
        trace,
        toolchain: ToolchainOut {
            lld_path: String::new(),
            lld_sha256: String::new(),
            sdk_um: String::new(),
            sdk_um_sha256: String::new(),
            ac_core_version: String::new(),
            worker_version: String::new(),
            worker_exe_sha256: crate::exec::worker_exe_sha256_public(),
            // Syntax parse/edit never lowers — no native backend ran.
            backend: String::new(),
        },
        cache: Some(crate::protocol::CacheOut {
            mode: format!("syntax:{}:revision-{}", grammar, revision),
            hits: vec![
                ("grammar".into(), grammar.to_string()),
                ("revision".into(), revision.to_string()),
                ("fallbacks".into(), fallbacks.to_string()),
            ],
            counters: vec![
                ("rescan_bytes".into(), total_rescan as u32),
                ("reparsed_decls".into(), total_reparsed as u32),
                ("reused_decls".into(), total_reused as u32),
                ("fallbacks".into(), fallbacks as u32),
                ("revision".into(), revision),
            ],
        }),
        metric_gaps: vec!["cpu_ms".into(), "io_bytes".into(), "peak_rss".into()],
    })
}

fn render_outline(items: &[ace_syntax_runtime::outline::OutlineItem]) -> String {
    use std::fmt::Write;
    let mut s = String::from("[\n");
    for (i, it) in items.iter().enumerate() {
        let params = it
            .params_text
            .as_deref()
            .map(|t| serde_json::to_string(t).unwrap_or_default())
            .unwrap_or_else(|| "null".into());
        let _ = write!(
            s,
            "  {{ \"name\": {}, \"params\": {}, \"start\": {}, \"end\": {}, \"syntax_ok\": {} }}{}\n",
            serde_json::to_string(&it.name).unwrap_or_default(),
            params,
            it.start,
            it.end,
            it.syntax_ok,
            if i + 1 < items.len() { "," } else { "" }
        );
    }
    s.push(']');
    s
}

fn infra(code: &str, message: impl Into<String>) -> WorkerError {
    WorkerError {
        code: code.to_string(),
        message: message.into(),
    }
}

fn ms(t: Instant) -> u64 {
    t.elapsed().as_millis() as u64
}

fn b64d(s: &str) -> Result<Vec<u8>, base64::DecodeError> {
    base64::engine::general_purpose::STANDARD.decode(s)
}
