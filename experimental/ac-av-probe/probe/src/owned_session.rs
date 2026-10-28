//! PLAN-821 R1-F07/R2-F07'/R3-F07: the REAL owned incremental session —
//! a production install channel for `av.owned/1` over the D828 interfaces
//! (`lower_function_owned_unit` → `FunctionCode::admit_unit` →
//! `ProgramSnapshot::link` → `execute_program`).
//!
//! R3-F07: a PURE single-body edit (exactly one free-function body
//! touched, every edit strictly inside that declaration, no `type`
//! declaration touched, signatures unchanged) rides the REAL D828
//! resident-function frontend: ONE fn-item parse of the edited slice,
//! the `prepare_candidate_document` re-check + single-body emission
//! against the RETAINED `CheckedEnv` (producer-attested by
//! `BodyEmitReceipt`: one real body plan, the rest body-less scaffolds),
//! then per-unit lower/verify/encode/admit. The UNCHANGED units'
//! parse/recheck/emit production work is ZERO — not merely
//! counted-visible. Interface/context edits (type-context change,
//! signature drift refused by the retained environment's interface gate,
//! multi-decl or declaration-set edits) take the whole-module channel,
//! where every unit re-frontends — the correct invalidation semantics,
//! never a silent degradation of a body edit.
//!
//! R2 hardening retained: TRANSACTIONAL installs — the ENTIRE candidate
//! builds aside (frontend → invalidation → per-unit lower/verify/encode/
//! admit → checked LINK → entry RUN) and only then PUBLISHES atomically.
//! ANY failure restores the source, revision, identity, residents,
//! production counters AND the executable to the PRIOR revision exactly.
//!
//! The formal `incremental_gate` covers the WHOLE production surface per
//! channel: whole-module frontend compiles, per-unit frontend production
//! (parse/recheck/emit), per-unit backend production (lower/encode/
//! admit), object identity (Arc::ptr_eq) and mint GENERATION moves on
//! untouched residents, and the resident SET membership. A real whole
//! frontend on a single-body edit, a silent whole recompute, an object
//! swap without a mint bump, or a missed invalidation reddens the gate.

use auto_ac_prototype::hir;
use auto_ac_prototype::source::ast::{FnItem, Item};
use auto_ac_prototype::source::{
    compile_text_record_source, env_fn_names, env_note_cold_walks, parse_program_with,
    prepare_candidate_document, resolve_env, CheckedEnv, FnCandidate, SourceProfile,
};
use auto_av_core::abc::{decode_abc, encode_abc};
use auto_av_core::av::{execute_program, CancelFlag, ExecLimits};
use auto_av_core::contract::{fnv1a64, ExecOutcome};
use auto_av_core::function_code::FunctionCode;
use auto_av_core::program_snapshot::ProgramSnapshot;
use auto_av_core::{abc_verify, mir_verify};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Real production counters for one function unit. The frontend triple
/// (parse/recheck/emit) is the per-unit frontend production work; the
/// backend triple (lower/encode/admit) is the unit production. Counters
/// track PUBLISHED work — a rejected candidate publishes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Production {
    pub parse: u32,
    pub recheck: u32,
    pub emit: u32,
    pub lower: u32,
    pub encode: u32,
    pub admit: u32,
}

impl Production {
    /// One whole-module round: the compile walked the body (parse +
    /// resolve/typecheck + emit) and the unit re-produced.
    fn whole_round() -> Production {
        Production {
            parse: 1,
            recheck: 1,
            emit: 1,
            lower: 1,
            encode: 1,
            admit: 1,
        }
    }
}

/// Which frontend channel an edit rode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditChannel {
    /// The D828 single-function channel: one fn-item parse, one body
    /// re-check + emission against the retained environment.
    SingleBody,
    /// The whole-module frontend (correct for type-context / signature /
    /// declaration-set changes — every unit re-frontends).
    WholeModule,
}

/// The checked projection one revision is keyed on: the module's type
/// context fingerprint plus each function def's span-free body
/// fingerprint.
#[derive(Clone, Debug)]
struct RevisionIdentity {
    type_ctx: u64,
    bodies: BTreeMap<String, u64>,
}

/// One top-level free-function declaration (warm-up parse coordinates).
#[derive(Clone, Debug)]
struct FnDecl {
    name: String,
    lo: u64,
    hi: u64,
}

/// The retained-environment facts a whole-module round refreshes.
struct EnvFacts {
    env: CheckedEnv,
    decl_fns: Vec<FnDecl>,
    decl_types: Vec<(u64, u64)>,
    def_ids: Vec<String>,
    sig_mode_overrides: BTreeMap<String, Vec<hir::ParamMode>>,
    fast_path_ok: bool,
}

/// The staged environment publication of a candidate (applied ONLY at
/// the atomic publish point). Every candidate carries one: a single-body
/// round COMMITS its prepared candidate into the retained environment's
/// slot; a whole-module round REPLACES the (now stale) environment.
enum EnvStep {
    /// Single-body: commit the prepared candidate into its slot (D-03 —
    /// the one publication point, exactly once) AND shift the retained
    /// declaration coordinates by the edit's length delta (R4-F07: the
    /// committed function's `hi` grows/shrinks by `delta`; every
    /// declaration positioned AFTER the function's pre-edit end shifts
    /// with it — consecutive variable-length edits must address the
    /// CURRENT source, never the cold-time spans).
    CommitBody {
        candidate: FnCandidate,
        decl_idx: usize,
        old_hi: u64,
        delta: i64,
    },
    /// Whole-module: the interface/context changed — the retained
    /// environment is stale and is REPLACED (with fresh declaration
    /// facts; outstanding candidates die by construction).
    Rebuild(Box<EnvFacts>),
}

impl EnvStep {
    /// Apply the retained-state side of the publication (the environment
    /// slot commit plus the declaration-coordinate shift).
    fn publish(self, s: &mut OwnedFunctionSession) -> Result<(), String> {
        match self {
            EnvStep::CommitBody {
                candidate,
                decl_idx,
                old_hi,
                delta,
            } => {
                candidate.commit(&mut s.env).map_err(|ds| {
                    ds.first()
                        .map(|d| format!("env commit: {} {}", d.code, d.message))
                        .unwrap_or_else(|| "env commit".to_string())
                })?;
                // The edited declaration's end moves by the delta; its
                // start (and every earlier declaration) is untouched;
                // every later declaration shifts wholesale.
                let d = &mut s.decl_fns[decl_idx];
                d.hi = (d.hi as i64 + delta) as u64;
                for dd in s.decl_fns.iter_mut().skip(decl_idx + 1) {
                    dd.lo = (dd.lo as i64 + delta) as u64;
                    dd.hi = (dd.hi as i64 + delta) as u64;
                }
                for t in s.decl_types.iter_mut() {
                    if t.0 >= old_hi {
                        t.0 = (t.0 as i64 + delta) as u64;
                        t.1 = (t.1 as i64 + delta) as u64;
                    }
                }
                Ok(())
            }
            EnvStep::Rebuild(facts) => {
                s.env = facts.env;
                s.decl_fns = facts.decl_fns;
                s.decl_types = facts.decl_types;
                s.def_ids = facts.def_ids;
                s.sig_mode_overrides = facts.sig_mode_overrides;
                s.fast_path_ok = facts.fast_path_ok;
                Ok(())
            }
        }
    }
}

/// One fully-built ASIDE candidate (all stages ran; nothing published).
struct Candidate {
    ident: RevisionIdentity,
    invalidated: Vec<String>,
    code: BTreeMap<String, Arc<FunctionCode>>,
    production: BTreeMap<String, Production>,
    linked: ProgramSnapshot,
    env_step: EnvStep,
    channel: EditChannel,
}

fn strip_spans(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(i) = rest.find("span: Span { start: ") {
        out.push_str(&rest[..i]);
        let after = &rest[i..];
        if let Some(close) = after.find('}') {
            rest = &after[close + 1..];
        } else {
            out.push_str(after);
            return out;
        }
    }
    out.push_str(rest);
    out
}

fn identity_of(checked: &auto_ac_prototype::verify::CheckedModule) -> RevisionIdentity {
    let m = checked
        .bundle()
        .modules
        .first()
        .cloned()
        .unwrap_or_else(|| panic!("checked module without a module payload"));
    let type_ctx = fnv1a64(strip_spans(&format!("{:?}", m.types)).as_bytes());
    let mut bodies = BTreeMap::new();
    for d in &m.defs {
        if let hir::DefKind::Function { ty, body, .. } = &d.kind {
            // `body` is a BodyId INDEX — the CONTENT lives in the bodies
            // table; the fingerprint hashes the span-free content.
            let content = m
                .bodies
                .get(body.0 as usize)
                .map(|b| strip_spans(&format!("{b:?}")))
                .unwrap_or_default();
            bodies.insert(
                d.id_text.clone(),
                fnv1a64(format!("{ty:?}|{content}").as_bytes()),
            );
        }
    }
    RevisionIdentity { type_ctx, bodies }
}

/// The same body-fingerprint formula for ONE def of a candidate bundle
/// (the fast path updates only the edited unit's fingerprint).
fn body_fp_of(module: &hir::Module, def_id: &str) -> Option<u64> {
    for d in &module.defs {
        if d.id_text != def_id {
            continue;
        }
        if let hir::DefKind::Function { ty, body, .. } = &d.kind {
            let content = module
                .bodies
                .get(body.0 as usize)
                .map(|b| strip_spans(&format!("{b:?}")))
                .unwrap_or_default();
            return Some(fnv1a64(format!("{ty:?}|{content}").as_bytes()));
        }
    }
    None
}

/// Whether an edit interval `[start, end)` (an insertion has start == end)
/// lies FULLY INSIDE the declaration span `[lo, hi)` — a replacement may
/// touch the boundaries, an insertion must sit STRICTLY inside (boundary
/// insertions shift the declaration without changing its text).
fn edit_inside_decl(start: u64, end: u64, lo: u64, hi: u64) -> bool {
    if start < lo || end > hi {
        return false;
    }
    if start == end {
        lo < start && start < hi
    } else {
        true
    }
}

/// Whether an edit interval MEETS a declaration span at all (any overlap,
/// boundary insertions included — conservative channel routing).
fn edit_meets_decl(start: u64, end: u64, lo: u64, hi: u64) -> bool {
    let eff_end = if start == end { start + 1 } else { end };
    lo < eff_end && start < hi
}

/// Parse the declaration facts of a text-record program.
fn decl_facts_of(
    program: &auto_ac_prototype::source::ast::Program,
    caps: &[String],
    bundle_module: &hir::Module,
) -> EnvFacts {
    let mut decl_fns = Vec::new();
    let mut decl_types = Vec::new();
    let mut fast_path_ok = true;
    for item in &program.items {
        match item {
            Item::Fn(f) => decl_fns.push(FnDecl {
                name: f.name.name.clone(),
                lo: f.span.start as u64,
                hi: f.span.end as u64,
            }),
            Item::Type(t) => {
                decl_types.push((t.span.start as u64, t.span.end as u64));
                if !t.methods.is_empty() {
                    // Methods ride the type item's span — their body edits
                    // cannot take the free-function fast path.
                    fast_path_ok = false;
                }
            }
            // PLAN-864: enum declarations occupy top-level spans too.
            Item::Enum(e) => {
                decl_types.push((e.span.start as u64, e.span.end as u64));
            }
        }
    }
    let mut env = resolve_env(program, caps, SourceProfile::TextRecord).expect(
        "the already-checked program resolves (same caps); an interface-level failure would have failed the compile first",
    );
    env_note_cold_walks(&mut env);
    // The env's fn slots must align with the top-level free functions in
    // source order (the fast path addresses slots by decl index).
    let env_names = env_fn_names(&env);
    let decl_names: Vec<&str> = decl_fns.iter().map(|d| d.name.as_str()).collect();
    if env_names != decl_names {
        fast_path_ok = false;
    }
    // Retained interface facts: every def's param modes.
    let mut sig_mode_overrides = BTreeMap::new();
    let mut def_ids = Vec::new();
    for d in &bundle_module.defs {
        if let hir::DefKind::Function { body, .. } = &d.kind {
            def_ids.push(d.id_text.clone());
            if let Some(b) = bundle_module.bodies.get(body.0 as usize) {
                sig_mode_overrides.insert(
                    d.id_text.clone(),
                    b.locals
                        .iter()
                        .filter(|l| l.role == hir::LocalRole::Param)
                        .map(|l| l.param_mode)
                        .collect(),
                );
            }
        }
    }
    if def_ids.len() != decl_fns.len() {
        fast_path_ok = false;
    }
    EnvFacts {
        env,
        decl_fns,
        decl_types,
        def_ids,
        sig_mode_overrides,
        fast_path_ok,
    }
}

/// The single-body channel's controlled dispositions.
enum SingleBodyError {
    /// The retained environment's interface gate refused the edit
    /// (signature drift): the whole-module channel owns it.
    Fallback,
    /// A real candidate rejection (frontend/lower/verify/admit/link
    /// diagnostics) — the edit fails; the session keeps the prior
    /// revision.
    Reject(String),
}

/// The owned incremental session.
pub struct OwnedFunctionSession {
    src: String,
    entry: String,
    caps: Vec<String>,
    /// Whole-module frontend compiles issued (the whole channel's real
    /// production point; counted even when the candidate later fails —
    /// the compile really ran).
    pub frontends: u32,
    env: CheckedEnv,
    decl_fns: Vec<FnDecl>,
    decl_types: Vec<(u64, u64)>,
    def_ids: Vec<String>,
    sig_mode_overrides: BTreeMap<String, Vec<hir::ParamMode>>,
    fast_path_ok: bool,
    code: BTreeMap<String, Arc<FunctionCode>>,
    production: BTreeMap<String, Production>,
    identity: RevisionIdentity,
    revision: u64,
    last_invalidated: Vec<String>,
    last_channel: EditChannel,
}

impl OwnedFunctionSession {
    /// Cold warm-up: ONE whole-module frontend compile, every unit
    /// admitted at revision 1, the checked environment retained.
    pub fn warmup(src: &str, entry: &str, caps: &[String]) -> Result<Self, String> {
        // The COLD whole-module frontend (one real compile; the checked
        // bundle also verifies the whole module).
        let compiled = compile_text_record_source(src, "session.at", caps).map_err(|ds| {
            ds.first()
                .map(|d| format!("{} {}", d.code, d.message))
                .unwrap_or_else(|| "frontend".to_string())
        })?;
        let checked = compiled.checked;
        let bundle = checked.bundle();
        let module = bundle
            .modules
            .first()
            .ok_or_else(|| "bundle has no module".to_string())?
            .clone();
        let ident = identity_of(&checked);
        let facts = decl_facts_of(
            &parse_program_with(src, SourceProfile::TextRecord)
                .map_err(|d| format!("decl parse: {}", d.message))?,
            caps,
            &module,
        );

        // Cold production: every unit walked once by the whole frontend
        // (parse/recheck/emit) + produced once (lower/encode/admit).
        // Build-aside then publish (the same transaction as edits).
        let names: Vec<String> = ident.bodies.keys().cloned().collect();
        let mut code = BTreeMap::new();
        let mut production = BTreeMap::new();
        let mut aside = BTreeMap::new();
        for n in &names {
            let (name, fc, _) =
                produce_unit_aside(bundle, n, caps, &facts.sig_mode_overrides, src, 1)?;
            aside.insert(name, fc);
            production.insert(n.clone(), Production::whole_round());
        }
        let linked = ProgramSnapshot::link(aside.values().cloned().collect(), entry, 1)
            .map_err(|e| format!("warmup link: {e}"))?;
        let (out, _) = execute_program(&linked, &ExecLimits::default(), &CancelFlag::new(false));
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("warmup run: {out:?}"));
        }
        code.append(&mut aside);
        Ok(OwnedFunctionSession {
            src: src.to_string(),
            entry: entry.to_string(),
            caps: caps.to_vec(),
            frontends: 1,
            env: facts.env,
            decl_fns: facts.decl_fns,
            decl_types: facts.decl_types,
            def_ids: facts.def_ids,
            sig_mode_overrides: facts.sig_mode_overrides,
            fast_path_ok: facts.fast_path_ok,
            code,
            production,
            identity: ident,
            revision: 1,
            last_invalidated: names,
            last_channel: EditChannel::WholeModule,
        })
    }

    /// Apply ONE source edit (see [`Self::edit_batch`]).
    pub fn edit(&mut self, start: u64, end: u64, insert: &[u8]) -> Result<Vec<String>, String> {
        self.edit_batch(&[(start, end, insert.to_vec())])
    }

    /// Apply a BATCH of edits ATOMICALLY (all ranges against the CURRENT
    /// source, applied back-to-front). The edit routes to a channel by
    /// its REAL shape: a pure single-body edit (exactly one free-function
    /// declaration met, EVERY edit fully inside it, no `type` declaration
    /// met) rides the D828 single-function channel — zero frontend work
    /// on the unchanged units. Everything else (including a signature
    /// drift the retained environment's interface gate refuses) takes the
    /// whole-module channel. The ENTIRE candidate installs aside and
    /// publishes only when EVERY stage succeeds; ANY failure restores the
    /// source, revision, identity, residents, production counters and
    /// keeps the PRIOR program runnable.
    pub fn edit_batch(&mut self, edits: &[(u64, u64, Vec<u8>)]) -> Result<Vec<String>, String> {
        let mut ranges: Vec<(u64, u64, &[u8])> = edits
            .iter()
            .map(|(a, b, i)| (*a, *b, i.as_slice()))
            .collect();
        ranges.sort_by_key(|(a, _, _)| std::cmp::Reverse(*a));
        let mut next = self.src.clone();
        for (start, end, insert) in ranges.iter() {
            let bytes = next.clone().into_bytes();
            if (*start as usize) > bytes.len() || (*end as usize) > bytes.len() || start > end {
                return Err(format!("edit range {start}..{end} out of bounds"));
            }
            let mut patched = String::from_utf8_lossy(&bytes[..*start as usize]).into_owned();
            patched.push_str(&String::from_utf8_lossy(insert));
            patched.push_str(&String::from_utf8_lossy(&bytes[*end as usize..]));
            next = patched;
        }

        // ---- CHANNEL SELECTION (the session's own structural proof) ----
        let mut single_idx: Option<usize> = None;
        let mut pure_single = self.fast_path_ok;
        for (idx, d) in self.decl_fns.iter().enumerate() {
            let met = ranges
                .iter()
                .any(|(s, e, _)| edit_meets_decl(*s, *e, d.lo, d.hi));
            if !met {
                continue;
            }
            // Every edit must lie FULLY inside this one declaration.
            if ranges
                .iter()
                .all(|(s, e, _)| edit_inside_decl(*s, *e, d.lo, d.hi))
            {
                single_idx = Some(idx);
            } else {
                pure_single = false;
            }
        }
        if ranges.iter().any(|(s, e, _)| {
            self.decl_types
                .iter()
                .any(|(lo, hi)| edit_meets_decl(*s, *e, *lo, *hi))
        }) {
            pure_single = false;
        }
        if single_idx.is_none() {
            pure_single = false;
        }

        // ---- CANDIDATE TRANSACTION (build everything aside) ------------
        let candidate = if let (true, Some(idx)) = (pure_single, single_idx) {
            match self.build_single_body_candidate(&next, idx) {
                Ok(c) => Some(c),
                Err(SingleBodyError::Fallback) => None,
                Err(SingleBodyError::Reject(e)) => return Err(e),
            }
        } else {
            None
        };
        let candidate = match candidate {
            Some(c) => c,
            None => self.build_whole_candidate(&next)?,
        };
        // Entry RUN check on the linked candidate BEFORE publishing.
        let (out, _) = execute_program(
            &candidate.linked,
            &ExecLimits::default(),
            &CancelFlag::new(false),
        );
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("candidate run: {out:?}"));
        }

        // ---- ATOMIC PUBLISH ------------------------------------------------
        self.revision += 1;
        self.src = next;
        self.code = candidate.code;
        // Merge production: untouched residents keep their counters;
        // re-produced ones advance by the aside production.
        let mut merged = self.production.clone();
        for (name, prod) in candidate.production {
            let e = merged.entry(name).or_default();
            e.parse += prod.parse;
            e.recheck += prod.recheck;
            e.emit += prod.emit;
            e.lower += prod.lower;
            e.encode += prod.encode;
            e.admit += prod.admit;
        }
        self.production = merged;
        self.identity = candidate.ident;
        self.last_invalidated = candidate.invalidated.clone();
        self.last_channel = candidate.channel;
        // The retained-environment publication (slot commit + declaration
        // coordinate shift, or the whole-round rebuild).
        candidate.env_step.publish(self)?;
        Ok(candidate.invalidated)
    }

    /// The SINGLE-BODY candidate: one fn-item parse of the edited slice,
    /// the `prepare_candidate_document` re-check + emission against the
    /// retained environment (producer receipt asserted), per-unit
    /// lower/verify/encode/admit from the candidate document. The type
    /// context is PROVEN unchanged (no `type` declaration met the edit);
    /// signature drift is the retained environment's own interface gate —
    /// a refusal there means the edit is not a pure body edit and the
    /// caller falls back to the whole-module channel.
    fn build_single_body_candidate(
        &mut self,
        next_src: &str,
        decl_idx: usize,
    ) -> Result<Candidate, SingleBodyError> {
        let decl = self.decl_fns[decl_idx].clone();
        let delta = next_src.len() as i64 - self.src.len() as i64;
        let new_hi = decl.hi as i64 + delta;
        if decl.lo as usize > next_src.len() || new_hi < 0 || new_hi as usize > next_src.len() {
            return Err(SingleBodyError::Reject("slice out of bounds".into()));
        }
        let slice = &next_src[decl.lo as usize..new_hi as usize];
        // ONE fn-item parse — the ONLY parse of this edit (production:
        // parse +1 for the edited unit, ZERO for the unchanged ones).
        let parsed = parse_program_with(slice, SourceProfile::TextRecord).map_err(|d| {
            SingleBodyError::Reject(format!(
                "parse {}@slice: {} {}",
                decl.name, d.code, d.message
            ))
        })?;
        let item: FnItem = match parsed.items.as_slice() {
            [Item::Fn(f)] => f.clone(),
            _ => {
                return Err(SingleBodyError::Reject(format!(
                    "single-body channel: the edited slice of `{}` is not exactly one function",
                    decl.name
                )))
            }
        };
        if item.name.name != decl.name {
            return Err(SingleBodyError::Reject(format!(
                "single-body channel: edited slice parsed as `{}`, expected `{}` (declaration identity changed)",
                item.name.name, decl.name
            )));
        }
        // The retained-environment single-body transaction: re-check
        // (resolve + typecheck of ONE body) + single-body emission. The
        // environment's current entry is restored before returning; the
        // candidate owns the new entry (published only at commit).
        let candidate =
            prepare_candidate_document(&mut self.env, &item, decl_idx, slice, "session.at")
                .map_err(|ds| {
                    if ds.iter().any(|d| d.code == "source.incremental") {
                        SingleBodyError::Fallback
                    } else {
                        SingleBodyError::Reject(
                            ds.first()
                                .map(|d| format!("candidate: {} {}", d.code, d.message))
                                .unwrap_or_else(|| "candidate".to_string()),
                        )
                    }
                })?;
        // Producer attestation: exactly ONE real body planned/rendered,
        // everything else body-less scaffolds.
        let receipt = candidate.receipt();
        if receipt.real_body_plans != 1 || receipt.scaffold_fns + 1 != self.decl_fns.len() {
            return Err(SingleBodyError::Reject(format!(
                "internal: single-body receipt {receipt:?} does not attest one real body"
            )));
        }
        let def_id = format!("d_{}", decl.name);
        // Per-unit production from the CANDIDATE document; the retained
        // interface facts supply the scaffold callees' param modes (the
        // t_MutRet wrapper ABI is not expressible in the document).
        let bundle = candidate.parts().bundle.clone();
        let unit = auto_mir_lower::lower_function_owned_unit_with_modes(
            &bundle,
            &def_id,
            &self.caps,
            fnv1a64(next_src.as_bytes()),
            &self.sig_mode_overrides,
        )
        .map_err(|e| SingleBodyError::Reject(format!("lower {def_id}: {}", e.code())))?;
        mir_verify::verify_mir(&unit)
            .map_err(|e| SingleBodyError::Reject(format!("verify {def_id}: {e:?}")))?;
        let wire = encode_abc(&unit)
            .map_err(|e| SingleBodyError::Reject(format!("encode {def_id}: {}", e.code())))?;
        let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &unit.owner)
            .map_err(|e| SingleBodyError::Reject(format!("abc admit {def_id}: {e:?}")))?;
        let _ = decode_abc(&wire.to_flat());
        let next_revision = self.revision + 1;
        let fc = Arc::new(
            FunctionCode::admit_unit(loaded, next_revision)
                .map_err(|e| SingleBodyError::Reject(format!("admit {def_id}: {e}")))?,
        );
        // The next resident set: unchanged residents keep their objects.
        let mut code: BTreeMap<String, Arc<FunctionCode>> = self
            .code
            .iter()
            .filter(|(n, _)| n.as_str() != def_id.as_str())
            .map(|(n, c)| (n.clone(), c.clone()))
            .collect();
        code.insert(def_id.clone(), fc);
        let mut production = BTreeMap::new();
        production.insert(def_id.clone(), Production::whole_round());
        let linked =
            ProgramSnapshot::link(code.values().cloned().collect(), &self.entry, next_revision)
                .map_err(|e| SingleBodyError::Reject(format!("candidate link: {e}")))?;
        // Identity: the type context is PROVEN unchanged (no type decl
        // met the edit); only the edited body's fingerprint moves (the
        // same formula, computed from the candidate's real def).
        let mut ident = self.identity.clone();
        let cand_module = bundle
            .modules
            .first()
            .ok_or_else(|| SingleBodyError::Reject("candidate bundle has no module".into()))?;
        match body_fp_of(cand_module, &def_id) {
            Some(fp) => {
                ident.bodies.insert(def_id.clone(), fp);
            }
            None => {
                return Err(SingleBodyError::Reject(
                    "internal: candidate document lacks the real def".into(),
                ))
            }
        }
        Ok(Candidate {
            ident,
            invalidated: vec![def_id],
            code,
            production,
            linked,
            env_step: EnvStep::CommitBody {
                candidate,
                decl_idx,
                old_hi: decl.hi,
                delta,
            },
            channel: EditChannel::SingleBody,
        })
    }

    /// The WHOLE-MODULE candidate (R2 transaction retained): one
    /// whole-module frontend compile, identity-diff invalidation, per-unit
    /// lower/verify/encode/admit of the invalidated set, checked LINK of
    /// the full next resident set. The frontend triple advances for EVERY
    /// unit (the whole compile walks every body); the backend triple only
    /// on the invalidated set. `force_all` skips the identity diff (the
    /// forced degradation channel: EVERY unit re-produces).
    fn build_whole_candidate(&mut self, next_src: &str) -> Result<Candidate, String> {
        self.build_whole_candidate_inner(next_src, false)
    }

    fn build_whole_candidate_inner(
        &mut self,
        next_src: &str,
        force_all: bool,
    ) -> Result<Candidate, String> {
        // The frontend compile counts even on later failure (it ran).
        self.frontends += 1;
        let compiled =
            compile_text_record_source(next_src, "session.at", &self.caps).map_err(|ds| {
                ds.first()
                    .map(|d| format!("{} {}", d.code, d.message))
                    .unwrap_or_else(|| "frontend".to_string())
            })?;
        let checked = compiled.checked;
        let bundle = checked.bundle();
        let ident = identity_of(&checked);
        // Invalidation: own body changed, or the module's TYPE CONTEXT
        // changed (signatures/layouts stale every unit's import table).
        let type_ctx_changed = ident.type_ctx != self.identity.type_ctx;
        let mut invalidated = Vec::new();
        for (name, fp) in &ident.bodies {
            let changed =
                force_all || type_ctx_changed || self.identity.bodies.get(name) != Some(fp);
            if changed {
                invalidated.push(name.clone());
            }
        }
        let mut code: BTreeMap<String, Arc<FunctionCode>> = self
            .code
            .iter()
            .filter(|(n, _)| ident.bodies.contains_key(*n))
            .map(|(n, c)| (n.clone(), c.clone()))
            .collect();
        let mut production: BTreeMap<String, Production> = BTreeMap::new();
        let next_revision = self.revision + 1;
        for name in &invalidated {
            let (n, fc, _) = produce_unit_aside(
                bundle,
                name,
                &self.caps,
                &self.sig_mode_overrides,
                next_src,
                next_revision,
            )?;
            code.insert(n, fc);
            production.insert(name.clone(), Production::whole_round());
        }
        let linked =
            ProgramSnapshot::link(code.values().cloned().collect(), &self.entry, next_revision)
                .map_err(|e| format!("candidate link: {e}"))?;
        // The retained environment is STALE after an interface/context
        // round — fresh declaration facts are staged and applied at the
        // atomic publish point.
        let module = bundle
            .modules
            .first()
            .ok_or_else(|| "bundle has no module".to_string())?;
        let facts = decl_facts_of(
            &parse_program_with(next_src, SourceProfile::TextRecord)
                .map_err(|d| format!("decl parse: {}", d.message))?,
            &self.caps,
            module,
        );
        Ok(Candidate {
            ident,
            invalidated,
            code,
            production,
            linked,
            env_step: EnvStep::Rebuild(Box::new(facts)),
            channel: EditChannel::WholeModule,
        })
    }

    /// Link the resident set and run the entry.
    pub fn run(&self) -> (ExecOutcome, auto_av_core::av::ExecTrace) {
        let code: Vec<Arc<FunctionCode>> = self.code.values().cloned().collect();
        let prog = ProgramSnapshot::link(code, &self.entry, self.revision)
            .expect("the resident set links");
        execute_program(&prog, &ExecLimits::default(), &CancelFlag::new(false))
    }

    pub fn run_i32(&self) -> i32 {
        let (out, _) = self.run();
        let ExecOutcome::Success {
            value: auto_av_core::contract::Value::I32(v),
            ..
        } = out
        else {
            panic!("expected i32 success, got {out:?}")
        };
        v
    }

    /// The FORMAL degradation detector (R3: channel-aware, covering the
    /// WHOLE production surface). After an edit whose invalidated set is
    /// `expected`:
    /// - a SINGLE-BODY edit (the channel the session actually routed):
    ///   ZERO whole-module frontend compiles, and every unit's per-unit
    ///   frontend production (parse/recheck/emit) advanced only on the
    ///   expected set — a real whole frontend on a body edit reddens
    ///   HERE;
    /// - a WHOLE-MODULE edit: exactly ONE whole-module frontend compile,
    ///   and every unit's frontend production advanced (the compile
    ///   walked every body) while the backend production stayed on the
    ///   expected set;
    /// - untouched residents keep their Arc identity AND mint generation
    ///   (an object swap without a mint bump reddens);
    /// - the resident set membership matches (missed invalidation and
    ///   silent removal both redden).
    pub fn incremental_gate(
        &self,
        before: &BTreeMap<String, Production>,
        before_snapshot: &GateSnapshot,
        frontends_before: u32,
        expected: &[&str],
    ) -> Result<(), String> {
        let expect_set: std::collections::BTreeSet<&str> = expected.iter().copied().collect();
        // (0) whole-module frontend compiles, by the channel actually rid.
        match self.last_channel {
            EditChannel::SingleBody => {
                if self.frontends != frontends_before {
                    return Err(format!(
                        "incremental gate: a single-body edit ran a WHOLE-MODULE frontend ({} -> {})",
                        frontends_before,
                        self.frontends
                    ));
                }
            }
            EditChannel::WholeModule => {
                if self.frontends != frontends_before + 1 {
                    return Err(format!(
                        "incremental gate: whole-module frontend compiles {} -> {} (expected exactly +1)",
                        frontends_before,
                        self.frontends
                    ));
                }
            }
        }
        // (1) production only on the invalidated set — all six counters;
        //     expected units advanced their frontend triple by one.
        for (name, now) in &self.production {
            let was = before.get(name).copied().unwrap_or_default();
            if expect_set.contains(name.as_str()) {
                if *now == was {
                    return Err(format!(
                        "incremental gate: `{name}` was invalidated but produced NOTHING"
                    ));
                }
                if now.parse != was.parse + 1
                    || now.recheck != was.recheck + 1
                    || now.emit != was.emit + 1
                {
                    return Err(format!(
                        "incremental gate: `{name}` frontend production parse/recheck/emit {}/{}/{} -> {}/{}/{} (expected +1 each)",
                        was.parse, was.recheck, was.emit, now.parse, now.recheck, now.emit
                    ));
                }
            } else if *now != was {
                return Err(format!(
                    "incremental gate: `{name}` produced outside the invalidated set (whole-recompute leakage)"
                ));
            }
        }
        // (2) untouched residents keep object identity + generation.
        for (name, was_code) in &before_snapshot.residents {
            if expect_set.contains(name.as_str()) {
                continue;
            }
            match self.code.get(name) {
                None => {
                    return Err(format!(
                        "incremental gate: untouched resident `{name}` vanished from the set"
                    ))
                }
                Some(now_code) => {
                    if !Arc::ptr_eq(was_code, now_code) {
                        return Err(format!(
                            "incremental gate: untouched resident `{name}` was object-swapped"
                        ));
                    }
                    if now_code.generation() != was_code.generation() {
                        return Err(format!(
                            "incremental gate: untouched resident `{name}` mint bumped without content"
                        ));
                    }
                }
            }
        }
        // (3) the set grows only by invalidated names.
        for name in self.code.keys() {
            if !before_snapshot.residents.contains_key(name) && !expect_set.contains(name.as_str())
            {
                return Err(format!(
                    "incremental gate: `{name}` appeared outside the invalidated set"
                ));
            }
        }
        Ok(())
    }

    /// Production snapshot (the caller takes `before` copies between
    /// edits and hands them to [`Self::incremental_gate`]).
    pub fn production(&self) -> BTreeMap<String, Production> {
        self.production.clone()
    }

    /// The identity snapshot the gate compares residents against.
    pub fn gate_snapshot(&self) -> GateSnapshot {
        GateSnapshot {
            residents: self
                .code
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        }
    }

    pub fn frontends(&self) -> u32 {
        self.frontends
    }

    /// The channel the LAST edit actually rode (the gate cross-checks it).
    pub fn last_channel(&self) -> EditChannel {
        self.last_channel
    }

    pub fn code_of(&self, def: &str) -> Arc<FunctionCode> {
        self.code.get(def).expect("resident").clone()
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn last_invalidated(&self) -> &[String] {
        &self.last_invalidated
    }

    /// The forced-whole channel: re-produce EVERY unit at a new revision
    /// (the degradation the gate must catch when the caller expects an
    /// incremental set).
    pub fn force_whole_recompute(&mut self) -> Result<(), String> {
        let saved_src = self.src.clone();
        let candidate = self.build_whole_candidate_inner(&saved_src, true)?;
        let (out, _) = execute_program(
            &candidate.linked,
            &ExecLimits::default(),
            &CancelFlag::new(false),
        );
        if !matches!(out, ExecOutcome::Success { .. }) {
            return Err(format!("forced run: {out:?}"));
        }
        self.revision += 1;
        self.last_channel = candidate.channel;
        self.last_invalidated = candidate.code.keys().cloned().collect();
        self.code = candidate.code;
        let mut merged = self.production.clone();
        for (name, prod) in candidate.production {
            let e = merged.entry(name).or_default();
            e.parse += prod.parse;
            e.recheck += prod.recheck;
            e.emit += prod.emit;
            e.lower += prod.lower;
            e.encode += prod.encode;
            e.admit += prod.admit;
        }
        self.production = merged;
        self.identity = candidate.ident;
        candidate.env_step.publish(self)?;
        Ok(())
    }
}

/// Lower/verify/encode/admit ONE unit into the ASIDE tables (the
/// production points all run; nothing resident is touched). The source
/// snapshot hashes the revision's WHOLE source (the unit's module
/// context), never a stale one.
fn produce_unit_aside(
    bundle: &hir::Bundle,
    def_id: &str,
    caps: &[String],
    sig_mode_overrides: &BTreeMap<String, Vec<hir::ParamMode>>,
    snapshot_src: &str,
    revision: u64,
) -> Result<(String, Arc<FunctionCode>, Production), String> {
    let snap = fnv1a64(snapshot_src.as_bytes());
    let unit = auto_mir_lower::lower_function_owned_unit_with_modes(
        bundle,
        def_id,
        caps,
        snap,
        sig_mode_overrides,
    )
    .map_err(|e| format!("lower {def_id}: {}", e.code()))?;
    mir_verify::verify_mir(&unit).map_err(|e| format!("verify {def_id}: {e:?}"))?;
    let wire = encode_abc(&unit).map_err(|e| format!("encode {def_id}: {}", e.code()))?;
    let loaded = abc_verify::verify_abc_bytes(&wire.to_flat(), &unit.owner)
        .map_err(|e| format!("abc admit {def_id}: {e:?}"))?;
    let _ = decode_abc(&wire.to_flat());
    let fc =
        FunctionCode::admit_unit(loaded, revision).map_err(|e| format!("admit {def_id}: {e}"))?;
    Ok((def_id.to_string(), Arc::new(fc), Production::whole_round()))
}

/// The resident-identity snapshot for the formal gate.
pub struct GateSnapshot {
    residents: BTreeMap<String, Arc<FunctionCode>>,
}
