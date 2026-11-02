//! PLAN-912: the closures FUNCTION frontend — a RETAINED checked environment
//! plus the per-lambda creation-site facts and the checked closures registry,
//! with an explicit single-body edit entry. The warm path re-parses only the
//! target unit's fragment, re-resolves only that unit, emits a scoped
//! single-function document (body-less scaffolds elsewhere) and hands the
//! verifier both — the credential and the registry — so a lambda body edit
//! produces exactly ONE unit without its owner or siblings ever being
//! walked, planned, bound, verified or lowered.
//!
//! The cold path is the whole-module truth (`resolve → typecheck → emit →
//! bind → verify_closures`) and BUILDS the retained state; the warm path is
//! an optimization over it that must be observably zero for every unchanged
//! unit (`crate::body_work` per-unit counts + `Stage` attempts, P/M/R axes).

use std::collections::BTreeMap;
use std::ops::Range;

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::body_work::{self, Stage as StageAxis};
use crate::runtime_contract::sha256_hex;
use crate::verify::{
    verify_closures, ClosuresVerifyContext, CheckedModule,
};

use super::adapter::{emit_function_body, emit_whole_document, BodyEmitReceipt, SourceParts};
use super::resolve::{
    resolve_closures_env, resolve_lambda_in_context, CheckedEnv, LambdaNonFast, LambdaReResolve,
    LambdaSiteFacts, BodyState,
};
use super::parser::parse_closure_fragment;
use super::{parse_program_with, SourceProfile};

/// Why an edit did not take the single-unit fast path. Each reason is a
/// NAMED decision, never a silent fallback (§5.1 `non_fast_path_reason`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonFastReason {
    /// The base source SHA does not match the frontend's current source
    /// (`source.stale` for callers; the edit entry is explicit about it).
    StaleBase,
    /// The range crosses unit boundaries (partly outside any one unit).
    CrossesUnits,
    /// The edit touches a top-level function's body (owner path; PLAN-912
    /// T-03 widens this route — until then it is the whole-module path).
    OwnerBodyEdit,
    /// The edit is inside a lambda expression but the re-resolved fragment
    /// changed the creation-site fact (capture uses / params / move / result).
    LambdaSiteChanged(LambdaNonFast),
    /// The lambda (old or new) contains nested lambdas — its ordinal space
    /// is not self-contained.
    LambdaNested,
    /// More than one contiguous changed region.
    MultiRegion,
}

/// One unit of the retained catalog (index = the fn's index in the retained
/// resolved program, owners first then lambdas, mirroring `resolve_detailed`).
#[derive(Clone, Debug, PartialEq)]
pub struct UnitEntry {
    /// Display name (`main`, `__closure_main_0`); wire identity is `d_<name>`.
    pub name: String,
    pub kind: UnitKind,
    /// The unit's whole source span in CURRENT coordinates (owner: fn item;
    /// lambda: lambda expression).
    pub span: Span,
    /// The unit's editable body span (owner: fn body block; lambda: the
    /// expression after the parameter list).
    pub body_span: Span,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnitKind {
    /// A top-level function; `lambdas` lists its lambda fn indices in
    /// creation order.
    Owner { lambdas: Vec<usize> },
    /// A lambda body; `site` carries the frozen creation-site facts.
    Lambda {
        /// Stable LOGICAL identity minted at cold init (never reused; a
        /// topological change that cannot be uniquely matched forces the
        /// non-fast path instead of silently reminting).
        logical_id: u64,
        owner: usize,
        site: LambdaSiteFacts,
    },
}

/// The candidate of ONE single-unit edit: it owns the new body entry (moved,
/// never cloned) with the environment identity and publication generation it
/// was prepared against, the scoped document, the new full source and the
/// coordinate delta. `commit` is the only installer (exactly once); a plain
/// drop publishes nothing.
pub struct ClosuresFnCandidate {
    env_id: u64,
    base_generation: u64,
    pub fn_index: usize,
    pub unit_name: String,
    state: BodyState,
    pub parts: SourceParts,
    pub receipt: BodyEmitReceipt,
    pub new_source: String,
    pub new_sha: String,
    /// (old edit end, byte delta) — the single-point coordinate shift for
    /// the catalog remap on commit.
    pub shift: (usize, i64),
}

impl ClosuresFnCandidate {
    /// The def id (`d_<name>`) of the edited unit in the scoped document.
    pub fn def_id_text(&self) -> String {
        format!("d_{}", self.unit_name)
    }
    /// The def INDEX of the edited unit in the scoped document's module.
    pub fn def_index(&self) -> usize {
        self.parts
            .bundle
            .modules
            .first()
            .and_then(|m| {
                m.defs
                    .iter()
                    .position(|d| d.id_text == self.def_id_text())
            })
            .unwrap_or(self.fn_index)
    }
}

/// The retained closures function frontend.
pub struct ClosuresFunctionFrontend {
    pub(crate) env: CheckedEnv,
    source: String,
    source_sha: String,
    module_path: String,
    capabilities: Vec<String>,
    units: Vec<UnitEntry>,
    registry: ClosuresVerifyContext,
    next_logical: u64,
    /// Successful publications this frontend has served (cold init = 0).
    generation: u64,
}

impl ClosuresFunctionFrontend {
    /// Cold init: the whole-module truth path, retained. Returns the
    /// frontend plus the whole `CompiledSource` (for the caller's cold
    /// unit production — lowering may consume the whole-module bundle there).
    pub fn cold(
        src: &str,
        module_path: &str,
        capabilities: &[String],
    ) -> Result<(Self, SourceParts, CheckedModule), Vec<Diagnostic>> {
        body_work::record_whole(StageAxis::WholeParse);
        let program = parse_program_with(src, SourceProfile::Closures).map_err(|d| vec![d])?;
        let (mut env, cache) = resolve_closures_env(&program, src, capabilities)?;
        env.typecheck_all()?;
        let parts = emit_whole_document(&env.program, src, module_path, SourceProfile::Closures)?;
        let checked = verify_closures(parts.bundle.clone())?;
        let registry = ClosuresVerifyContext::from_checked(&checked)?;
        // Catalog: owners in declaration order with their lambdas; lambdas
        // are appended after all owners (resolve_detailed's layout).
        let mut units: Vec<UnitEntry> = Vec::with_capacity(env.program.fns.len());
        let mut owner_of_lambda: BTreeMap<String, usize> = BTreeMap::new();
        for (i, f) in env.program.fns.iter().enumerate() {
            units.push(UnitEntry {
                name: f.name.clone(),
                kind: UnitKind::Owner { lambdas: Vec::new() },
                span: f.item_span,
                body_span: f.body.span,
            });
            owner_of_lambda.insert(f.name.clone(), i);
        }
        let mut next_logical: u64 = 1;
        for (owner_name, owner) in &cache.owners {
            let Some(&owner_idx) = owner_of_lambda.get(owner_name) else {
                return Err(vec![Diagnostic::new(
                    Stage::Bind,
                    "source.internal",
                    format!("cached owner `{owner_name}` has no resolved slot"),
                    Span { start: 0, end: 0 },
                )]);
            };
            let mut lambda_indices = Vec::with_capacity(owner.lambdas.len());
            for lambda in &owner.lambdas {
                let idx = env
                    .program
                    .fns
                    .iter()
                    .position(|f| f.name == lambda.name)
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Stage::Bind,
                            "source.internal",
                            format!("cached lambda `{}` has no resolved slot", lambda.name),
                            Span { start: 0, end: 0 },
                        )]
                    })?;
                // The site facts of THIS lambda (cache stores per owner, in
                // creation order — same order as `lambdas`).
                let site = owner
                    .sites
                    .iter()
                    .find(|s| s.name == lambda.name)
                    .cloned()
                    .ok_or_else(|| {
                        vec![Diagnostic::new(
                            Stage::Bind,
                            "source.internal",
                            format!("lambda `{}` has no creation-site facts", lambda.name),
                            Span { start: 0, end: 0 },
                        )]
                    })?;
                units[idx] = UnitEntry {
                    name: lambda.name.clone(),
                    kind: UnitKind::Lambda {
                        logical_id: next_logical,
                        owner: owner_idx,
                        site,
                    },
                    span: lambda.item_span,
                    body_span: lambda.item_span,
                };
                next_logical += 1;
                lambda_indices.push(idx);
            }
            if let UnitKind::Owner { lambdas } = &mut units[owner_idx].kind {
                *lambdas = lambda_indices;
            }
        }
        Ok((
            ClosuresFunctionFrontend {
                env,
                source: src.to_string(),
                source_sha: sha256_hex(src.as_bytes()),
                module_path: module_path.to_string(),
                capabilities: capabilities.to_vec(),
                units,
                registry,
                next_logical,
                generation: 0,
            },
            parts,
            checked,
        ))
    }

    // ── observation / accepted-state reads ──

    pub fn source_sha(&self) -> &str {
        &self.source_sha
    }
    /// The current full source text (the diff wrapper reads it; the
    /// frontend owns the accepted state).
    pub fn source(&self) -> &str {
        &self.source
    }
    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn units(&self) -> &[UnitEntry] {
        &self.units
    }
    pub fn registry(&self) -> &ClosuresVerifyContext {
        &self.registry
    }
    pub fn env(&self) -> &CheckedEnv {
        &self.env
    }
    /// The profile capabilities this frontend was cold-initialized with
    /// (carried into the accepted source state; PLAN-912 §5.3).
    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }
    /// The next un-minted lambda LOGICAL id (accepted-state export;
    /// PLAN-912 §5.3).
    pub fn next_logical_id(&self) -> u64 {
        self.next_logical
    }
    /// PLAN-912 §5.5: retained frontend bytes estimate — the source, the
    /// retained resolved program's fns (nodes counted by a bounded walk) and
    /// the catalog/registry. The pool gate (≤16 MiB) reads this.
    pub fn retained_bytes_estimate(&self) -> usize {
        let mut bytes = self.source.len()
            + self.module_path.len()
            + self.units.iter().map(|u| std::mem::size_of::<UnitEntry>() + u.name.len()).sum::<usize>();
        for f in &self.env.program.fns {
            bytes += f.name.len() + f.locals.len() * 64 + count_resolved_nodes(f);
        }
        bytes
    }

    // ── warm single-unit edit ──

    /// The explicit edit entry (§5.1): base full-source SHA + UTF-8 range +
    /// replacement. Only the target unit's fragment is parsed; the rest of
    /// the pipeline (resolve/typecheck/emit/bind) touches exactly the
    /// target. Returns the candidate (verify/lower/install are the caller's
    /// stages), or the NAMED non-fast reason for the caller's whole-module
    /// route.
    pub fn edit_body(
        &mut self,
        base_sha: &str,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<Result<ClosuresFnCandidate, NonFastReason>, Vec<Diagnostic>> {
        body_work::record_whole(StageAxis::SourceHash);
        if base_sha != self.source_sha {
            return Ok(Err(NonFastReason::StaleBase));
        }
        if !self.source.is_char_boundary(range.start) || !self.source.is_char_boundary(range.end)
            || range.start > range.end || range.end > self.source.len()
        {
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.incremental-fragment",
                "edit range must be ordered, in bounds and on UTF-8 boundaries",
                Span { start: 0, end: 0 },
            )]);
        }
        // Classify: the innermost lambda whose span contains the range, else
        // the owner whose body span contains it without touching any lambda.
        let target = classify_edit(&self.units, &range)?;
        let Some(target_idx) = target else {
            return Ok(Err(match classify_edit_loose(&self.units, &range) {
                Some(ClassLoose::Owner) => NonFastReason::OwnerBodyEdit,
                _ => NonFastReason::CrossesUnits,
            }));
        };
        let UnitKind::Lambda { site, .. } = &self.units[target_idx].kind else {
            // Owner body edits are the T-03 route; today: named non-fast.
            return Ok(Err(NonFastReason::OwnerBodyEdit));
        };
        if site.nested != 0 {
            return Ok(Err(NonFastReason::LambdaNested));
        }
        let unit_name = self.units[target_idx].name.clone();
        let old_span = self.units[target_idx].span;
        let delta = replacement.len() as i64 - (range.end - range.start) as i64;
        let mut new_source = String::with_capacity((self.source.len() as i64 + delta) as usize);
        new_source.push_str(&self.source[..range.start]);
        new_source.push_str(replacement);
        new_source.push_str(&self.source[range.end..]);
        let new_span = Span {
            start: old_span.start,
            end: (old_span.end as i64 + delta) as usize,
        };
        // P axis: one fragment parse for the changed unit only (wire-keyed).
        body_work::record_attempt(StageAxis::ParseProject, &format!("d_{unit_name}"));
        let expr =
            parse_closure_fragment(&new_source, new_span.start..new_span.end, SourceProfile::Closures)
                .map_err(|d| vec![d])?;
        // P axis: one resolve of the changed unit only (inside).
        let state = match resolve_lambda_in_context(&self.env, site, &expr) {
            Err(LambdaReResolve::NonFast(why)) => {
                return Ok(Err(match why {
                    LambdaNonFast::NestedLambdas => NonFastReason::LambdaNested,
                    other => NonFastReason::LambdaSiteChanged(other),
                }))
            }
            Err(LambdaReResolve::Diagnostics(ds)) => return Err(ds),
            Ok(state) => state,
        };
        // Graft, typecheck the ONE body, emit the scoped document, restore.
        let env_id = self.env.env_id();
        let base_generation = self.env.generation();
        let previous = state.swap_into(&mut self.env, target_idx);
        let check = super::typecheck::check_body_in(&self.env.program, target_idx);
        if !check.is_empty() {
            previous.swap_into(&mut self.env, target_idx);
            return Err(check);
        }
        let emit = emit_function_body(&self.env, target_idx, &new_source, &self.module_path);
        let (parts, receipt) = match emit {
            Ok(ok) => ok,
            Err(ds) => {
                previous.swap_into(&mut self.env, target_idx);
                return Err(ds);
            }
        };
        let new_state = previous.swap_into(&mut self.env, target_idx);
        Ok(Ok(ClosuresFnCandidate {
            env_id,
            base_generation,
            fn_index: target_idx,
            unit_name,
            state: new_state,
            parts,
            receipt,
            new_sha: sha256_hex(new_source.as_bytes()),
            new_source,
            shift: (range.end, delta),
        }))
    }

    /// Commit ONE verified edit: installs the prepared body entry, swaps the
    /// source, remaps the catalog coordinates (M axis — spans only, never a
    /// resolved body walk) and publishes a new generation. The registry is
    /// unchanged (the fast path's oracle IS that the creation-site facts did
    /// not change).
    pub fn commit_candidate(&mut self, candidate: ClosuresFnCandidate) -> Result<(), String> {
        if self.env.env_id() != candidate.env_id {
            return Err("source.foreign: the candidate was prepared against another environment".to_string());
        }
        if self.env.generation() != candidate.base_generation {
            return Err("source.stale: the base has been published since the candidate was prepared".to_string());
        }
        let state = candidate.state.swap_into(&mut self.env, candidate.fn_index);
        debug_assert_eq!(state.entry.name, candidate.unit_name);
        let _ = state;
        self.source = candidate.new_source;
        self.source_sha = candidate.new_sha;
        // M axis: shift every catalog span after the edit point (plus the
        // edited unit's end). No resolved body is visited.
        body_work::record_whole(StageAxis::CoordRemap);
        let (edit_end, delta) = candidate.shift;
        for (i, u) in self.units.iter_mut().enumerate() {
            let is_target = i == candidate.fn_index;
            if delta != 0 {
                if u.span.start >= edit_end {
                    u.span.start = (u.span.start as i64 + delta) as usize;
                    u.span.end = (u.span.end as i64 + delta) as usize;
                    u.body_span.start = (u.body_span.start as i64 + delta) as usize;
                    u.body_span.end = (u.body_span.end as i64 + delta) as usize;
                } else if is_target || u.span.end > edit_end {
                    // The edited unit (its end moves by delta) or a span
                    // that contains the edit point (only the target can).
                    u.span.end = (u.span.end as i64 + delta) as usize;
                    u.body_span.end = (u.body_span.end as i64 + delta) as usize;
                }
            }
            if is_target {
                if let UnitKind::Lambda { site, .. } = &mut u.kind {
                    site.expr_span = u.span;
                }
            }
        }
        self.env.generation += 1;
        self.generation += 1;
        Ok(())
    }

    /// PLAN-828 D-03 release: the environment's owned body entries.
    pub fn close(&mut self) {
        self.env.close();
    }
}

fn count_resolved_nodes(f: &super::resolve::ResolvedFn) -> usize {
    fn block(b: &super::resolve::RBlock) -> usize {
        b.stmts.iter().map(stmt).sum()
    }
    fn stmt(s: &super::resolve::RStmt) -> usize {
        let self_cost = 1 + match s {
            super::resolve::RStmt::Let { value, .. }
            | super::resolve::RStmt::Assign { value, .. }
            | super::resolve::RStmt::Return { value, .. } => expr(value),
            super::resolve::RStmt::If { cond, then, els, .. } => {
                expr(cond) + block(then) + els.as_ref().map(block).unwrap_or(0)
            }
            super::resolve::RStmt::While { cond, body, .. } => expr(cond) + block(body),
            super::resolve::RStmt::Loop { body, .. } => block(body),
            _ => 0,
        };
        self_cost
    }
    fn expr(e: &super::resolve::ResolvedExpr) -> usize {
        1 + match &e.kind {
            super::resolve::RExprKind::Binary { lhs, rhs, .. } => expr(lhs) + expr(rhs),
            super::resolve::RExprKind::Unary { operand, .. } => expr(operand),
            _ => 0,
        }
    }
    f.locals.len() * 32 + block(&f.body)
}

enum ClassLoose {
    Owner,
}

fn classify_edit(units: &[UnitEntry], range: &Range<usize>) -> Result<Option<usize>, Vec<Diagnostic>> {
    // Innermost LAMBDA containing the range.
    let mut best: Option<usize> = None;
    for (i, u) in units.iter().enumerate() {
        if matches!(u.kind, UnitKind::Lambda { .. }) && u.span.start <= range.start && range.end <= u.span.end {
            match best {
                Some(b) if units[b].span.end - units[b].span.start <= u.span.end - u.span.start => {}
                _ => best = Some(i),
            }
        }
    }
    Ok(best)
}

fn classify_edit_loose(units: &[UnitEntry], range: &Range<usize>) -> Option<ClassLoose> {
    for u in units {
        if matches!(u.kind, UnitKind::Owner { .. })
            && u.body_span.start <= range.start
            && range.end <= u.body_span.end
        {
            return Some(ClassLoose::Owner);
        }
    }
    None
}
