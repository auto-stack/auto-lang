//! PLAN-912: the closures FUNCTION frontend — a RETAINED checked environment
//! plus the per-lambda creation-site facts and the checked closures registry,
//! with an explicit single-body edit entry. The warm path re-parses only the
//! target unit's fragment, re-resolves only that unit (an owner re-resolves
//! through the retained-lambda reuse oracle, so unchanged siblings are never
//! walked), emits a scoped document (body-less scaffolds elsewhere) and hands
//! the verifier both — the credential and the registry — so an edit produces
//! exactly the changed units without any other unit being walked, planned,
//! bound, verified or lowered.
//!
//! The cold path is the whole-module truth (`resolve → typecheck → emit →
//! bind → verify_closures`) and BUILDS the retained state; the warm path is
//! an optimization over it that must be observably zero for every unchanged
//! unit (`crate::body_work` per-unit counts + `Stage` attempts, P/M/R axes).

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use crate::atom_text::{Diagnostic, Span, Stage};
use crate::body_work::{self, Stage as StageAxis};
use crate::runtime_contract::sha256_hex;
use crate::verify::{verify_closures, CheckedModule, ClosuresVerifyContext};

use super::adapter::{
    emit_closures_function_batch, emit_function_body, emit_whole_document, BodyEmitReceipt,
    SourceParts,
};
// PLAN-929 T-03：片段生产者切统一前门（旧手写 parser 保留为差分 oracle）。
use super::cst::{parse_closure_fragment_via_cst, parse_function_fragment_via_cst};
use super::resolve::{
    resolve_closures_env, resolve_lambda_in_context, resolve_owner_in_env, BodyState, CheckedEnv,
    LambdaNonFast, LambdaReResolve, LambdaSiteFacts, ReuseEntry,
};
use super::{parse_program_with, SourceProfile};

/// Why an edit did not take a warm path. Each reason is a NAMED decision,
/// never a silent fallback (§5.1 `non_fast_path_reason`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NonFastReason {
    /// The base source SHA does not match the frontend's current source.
    StaleBase,
    /// The range is not inside any one unit (crosses owners, or touches a
    /// declaration outside every function body).
    CrossesUnits,
    /// The owner's signature changed (the retained interface is stale).
    OwnerSignatureChanged,
    /// The owner's lambda topology changed (count differs) — identity cannot
    /// be matched by position; the whole truth path re-establishes it.
    OwnerTopologyChanged { before: usize, after: usize },
    /// A lambda-only edit changed its creation-site fact and the enclosing
    /// owner route also declined (reported with the owner's reason).
    LambdaSiteChanged(LambdaNonFast),
    /// The lambda (old or new) contains nested lambdas and was edited alone;
    /// the owner route handles nesting through reuse.
    LambdaNested,
    /// PLAN-913: the context was restored from persisted DECLARATIONS, so a
    /// retained BODY cannot be reused. An edit that needs one (the owner
    /// route's lambda reuse) is this named non-fast; the caller then runs the
    /// whole-module truth path and reports its production honestly.
    RestoredContext,
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
    /// The unit's editable body span (owner: fn body block; lambda: same as
    /// `span`).
    pub body_span: Span,
    /// PLAN-912 §5.3: span-free canonical digest of the unit's VERIFIED HIR
    /// body (cold: from the whole checked document; warm: from the minted
    /// credential at commit).
    pub body_digest: String,
    /// PLAN-912 §5.3: the unit's span at the revision its LOGICAL identity
    /// was minted (carried unchanged across edits and identity-carrying
    /// rebuilds); `span` is the current coordinate.
    pub mint_span: Span,
}

/// PLAN-912 §5.3: the verified facts a committed unit binds into the
/// catalog (from the credential the session minted for it).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnitCommitFacts {
    pub fn_index: usize,
    pub body_digest: String,
    pub context_revision: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UnitKind {
    /// A top-level function; `lambdas` lists its lambda fn indices in
    /// creation (post-order) order.
    Owner { lambdas: Vec<usize> },
    /// A lambda body; `site` carries the frozen creation-site facts.
    Lambda {
        /// Stable LOGICAL identity minted at cold init and carried across
        /// rebuilds by unique (owner, text) match; never reused for a
        /// different lambda in one frontend lineage.
        logical_id: u64,
        owner: usize,
        site: LambdaSiteFacts,
    },
}

/// One unit the candidate re-resolved (owner or lambda).
pub struct CandidateUnit {
    pub fn_index: usize,
    pub name: String,
    state: BodyState,
}

/// The owner-route bookkeeping a candidate carries: the owner slot plus the
/// post-walk site facts of EVERY lambda of that owner (reused ones carry
/// their re-derived capture sources and current spans).
pub struct OwnerUpdate {
    pub owner_index: usize,
    pub sites: Vec<(usize, LambdaSiteFacts)>,
    pub reused_ordinals: Vec<usize>,
}

/// The candidate of ONE edit: the re-resolved unit entries (moved, never
/// cloned) with the environment identity and publication generation they
/// were prepared against, the scoped document, the new full source and the
/// coordinate delta. `commit` is the only installer (exactly once); a plain
/// drop publishes nothing.
pub struct ClosuresFnCandidate {
    env_id: u64,
    base_generation: u64,
    pub units: Vec<CandidateUnit>,
    pub owner: Option<OwnerUpdate>,
    pub parts: SourceParts,
    pub receipt: BodyEmitReceipt,
    pub new_source: String,
    pub new_sha: String,
    /// (old edit end, byte delta) — the single-point coordinate shift for
    /// the catalog remap on commit.
    pub shift: (usize, i64),
}

impl ClosuresFnCandidate {
    /// The primary (first) unit's fn index — for a lambda edit the lambda,
    /// for an owner edit the owner.
    pub fn fn_index(&self) -> usize {
        self.units[0].fn_index
    }
    /// The primary unit's display name.
    pub fn unit_name(&self) -> &str {
        &self.units[0].name
    }
    /// The def id (`d_<name>`) of a unit in the scoped document.
    pub fn def_id_text_of(&self, unit: &CandidateUnit) -> String {
        format!("d_{}", unit.name)
    }
    /// The primary unit's def id.
    pub fn def_id_text(&self) -> String {
        format!("d_{}", self.units[0].name)
    }
    /// The def INDEX of a unit in the scoped document's module.
    pub fn def_index_of(&self, unit: &CandidateUnit) -> usize {
        let id = self.def_id_text_of(unit);
        self.parts
            .bundle
            .modules
            .first()
            .and_then(|m| m.defs.iter().position(|d| d.id_text == id))
            .unwrap_or(unit.fn_index)
    }
    /// The primary unit's def index.
    pub fn def_index(&self) -> usize {
        self.def_index_of(&self.units[0])
    }
    /// Def ids of every unit whose body is REAL in the document (the
    /// creators the registry overlay must re-derive from).
    pub fn real_def_ids(&self) -> Vec<String> {
        self.units.iter().map(|u| self.def_id_text_of(u)).collect()
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
    /// The declaration-context fingerprint of the accepted document (the
    /// `context_revision` every unit credential binds).
    context_revision: String,
    /// PLAN-913: this frontend came from persisted declarations (bodies are
    /// scaffolds, not retained bodies) — the owner route must decline.
    restored: bool,
}

impl ClosuresFunctionFrontend {
    /// PLAN-913 T-05: build a frontend from a RESTORED declaration context.
    /// The environment carries signature-only entries for every function
    /// (empty bodies/locals) — they are document scaffolds, never bodies the
    /// warm path may reuse. Route vetoes that depend on a retained body are
    /// the caller's named non-fast decision, not a silent widening here.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_restored(
        env: CheckedEnv,
        source: String,
        source_sha: String,
        module_path: String,
        capabilities: Vec<String>,
        units: Vec<UnitEntry>,
        registry: ClosuresVerifyContext,
        next_logical: u64,
        generation: u64,
        context_revision: String,
    ) -> Self {
        ClosuresFunctionFrontend {
            env,
            source,
            source_sha,
            module_path,
            capabilities,
            units,
            registry,
            next_logical,
            generation,
            context_revision,
            restored: true,
        }
    }

    /// PLAN-913 T-05: is this a context rebuilt from persisted declarations
    /// (so a retained BODY cannot be reused)?
    pub fn is_restored(&self) -> bool {
        self.restored
    }
}

fn fnv64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_001b_3);
    }
    h
}

impl ClosuresFunctionFrontend {
    /// Cold init: the whole-module truth path, retained. Returns the
    /// frontend plus the whole document and credential (for the caller's
    /// cold unit production).
    pub fn cold(
        src: &str,
        module_path: &str,
        capabilities: &[String],
    ) -> Result<(Self, SourceParts, CheckedModule), Vec<Diagnostic>> {
        Self::cold_inner(src, module_path, capabilities, None)
    }

    /// Cold rebuild that CARRIES lambda logical ids from a previous frontend
    /// of the same lineage: a lambda is matched by (owner, expression text)
    /// when that match is unique on both sides; anything ambiguous or new
    /// mints a fresh id (never a silent re-hit). The whole truth path still
    /// runs — this only preserves identity bookkeeping (§5.1 "能唯一匹配的
    /// 存活单元保身份，歧义不误命中").
    pub fn cold_carrying(
        src: &str,
        module_path: &str,
        capabilities: &[String],
        previous: &ClosuresFunctionFrontend,
    ) -> Result<(Self, SourceParts, CheckedModule), Vec<Diagnostic>> {
        Self::cold_inner(src, module_path, capabilities, Some(previous))
    }

    fn cold_inner(
        src: &str,
        module_path: &str,
        capabilities: &[String],
        previous: Option<&ClosuresFunctionFrontend>,
    ) -> Result<(Self, SourceParts, CheckedModule), Vec<Diagnostic>> {
        let program = parse_program_with(src, SourceProfile::Closures).map_err(|d| vec![d])?;
        let (mut env, cache) = resolve_closures_env(&program, src, capabilities)?;
        env.typecheck_all()?;
        let parts = emit_whole_document(&env.program, src, module_path, SourceProfile::Closures)?;
        let checked = verify_closures(parts.bundle.clone())?;
        let registry = ClosuresVerifyContext::from_checked(&checked)?;
        // Per-def verified body digests + the document's context revision.
        let module = &parts.bundle.modules[0];
        let context_revision = crate::verify::declaration_context_fingerprint_in(
            module,
            crate::hir::HirDomain::Closures,
        );
        let mut digests: BTreeMap<String, String> = BTreeMap::new();
        for d in &module.defs {
            if let crate::hir::DefKind::Function { body, .. } = &d.kind {
                if let Some(b) = module.bodies.get(body.0 as usize) {
                    digests.insert(
                        d.id_text.clone(),
                        crate::collections_facts::body_content_digest(b),
                    );
                }
            }
        }
        // Catalog: owners in declaration order with their lambdas; lambdas
        // are appended after all owners (resolve_detailed's layout).
        let mut units: Vec<UnitEntry> = Vec::with_capacity(env.program.fns.len());
        let mut owner_of: BTreeMap<String, usize> = BTreeMap::new();
        for (i, f) in env.program.fns.iter().enumerate() {
            units.push(UnitEntry {
                name: f.name.clone(),
                kind: UnitKind::Owner {
                    lambdas: Vec::new(),
                },
                span: f.item_span,
                body_span: f.body.span,
                body_digest: digests
                    .get(&format!("d_{}", f.name))
                    .cloned()
                    .unwrap_or_default(),
                mint_span: f.item_span,
            });
            owner_of.insert(f.name.clone(), i);
        }
        // Previous-lineage identity index: (owner, text fp) → logical id,
        // only for texts unique within their owner on the previous side.
        let mut carry: BTreeMap<(String, u64), (u64, Span)> = BTreeMap::new();
        let mut next_logical: u64 = 1;
        if let Some(prev) = previous {
            let mut seen: BTreeMap<(String, u64), usize> = BTreeMap::new();
            for u in &prev.units {
                if let UnitKind::Lambda {
                    logical_id, owner, ..
                } = &u.kind
                {
                    let key = (
                        prev.units[*owner].name.clone(),
                        fnv64(prev.source[u.span.start..u.span.end].as_bytes()),
                    );
                    *seen.entry(key.clone()).or_default() += 1;
                    carry.insert(key, (*logical_id, u.mint_span));
                }
            }
            for (key, n) in seen {
                if n > 1 {
                    carry.remove(&key);
                }
            }
            next_logical = prev.next_logical;
        }
        // New-side uniqueness: a text that appears twice in one owner is
        // ambiguous on this side too.
        let mut new_counts: BTreeMap<(String, u64), usize> = BTreeMap::new();
        for (owner_name, owner) in &cache.owners {
            for lambda in &owner.lambdas {
                let key = (
                    owner_name.clone(),
                    fnv64(src[lambda.item_span.start..lambda.item_span.end].as_bytes()),
                );
                *new_counts.entry(key).or_default() += 1;
            }
        }
        for (owner_name, owner) in &cache.owners {
            let Some(&owner_idx) = owner_of.get(owner_name) else {
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
                let key = (
                    owner_name.clone(),
                    fnv64(src[lambda.item_span.start..lambda.item_span.end].as_bytes()),
                );
                let (logical_id, mint_span) = match (carry.get(&key), new_counts.get(&key)) {
                    (Some((id, mint)), Some(1)) => (*id, *mint),
                    _ => {
                        let id = next_logical;
                        next_logical += 1;
                        (id, lambda.item_span)
                    }
                };
                units[idx] = UnitEntry {
                    name: lambda.name.clone(),
                    kind: UnitKind::Lambda {
                        logical_id,
                        owner: owner_idx,
                        site,
                    },
                    span: lambda.item_span,
                    body_span: lambda.item_span,
                    body_digest: digests
                        .get(&format!("d_{}", lambda.name))
                        .cloned()
                        .unwrap_or_default(),
                    mint_span,
                };
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
                context_revision,
                restored: false,
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
    /// The next un-minted lambda LOGICAL id (accepted-state export).
    pub fn next_logical_id(&self) -> u64 {
        self.next_logical
    }
    /// The accepted document's declaration-context fingerprint.
    pub fn context_revision(&self) -> &str {
        &self.context_revision
    }
    /// The module path (logical owner) of the accepted source.
    pub fn source_path(&self) -> &str {
        &self.module_path
    }
    /// PLAN-912 §5.5: retained frontend bytes estimate — the source, the
    /// retained resolved program's fns (nodes counted by a bounded walk) and
    /// the catalog/registry. The pool gate (≤16 MiB) reads this.
    pub fn retained_bytes_estimate(&self) -> usize {
        let mut bytes = self.source.len()
            + self.module_path.len()
            + self
                .units
                .iter()
                .map(|u| std::mem::size_of::<UnitEntry>() + u.name.len())
                .sum::<usize>();
        for f in &self.env.program.fns {
            bytes += f.name.len() + f.locals.len() * 64 + count_resolved_nodes(f);
        }
        bytes
    }

    // ── warm single-unit edit ──

    /// The explicit edit entry (§5.1): base full-source SHA + UTF-8 range +
    /// replacement. Only the target unit's fragment is parsed; the rest of
    /// the pipeline touches exactly the changed units. Returns the candidate
    /// (verify/lower/install are the caller's stages), or the NAMED non-fast
    /// reason for the caller's whole-module route.
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
        if !self.source.is_char_boundary(range.start)
            || !self.source.is_char_boundary(range.end)
            || range.start > range.end
            || range.end > self.source.len()
        {
            return Err(vec![Diagnostic::new(
                Stage::Text,
                "source.incremental-fragment",
                "edit range must be ordered, in bounds and on UTF-8 boundaries",
                Span { start: 0, end: 0 },
            )]);
        }
        // Innermost lambda containing the range → lambda route first.
        if let Some(lambda_idx) = innermost_lambda(&self.units, &range) {
            match self.edit_lambda(lambda_idx, range.clone(), replacement)? {
                Ok(candidate) => return Ok(Ok(candidate)),
                Err(LambdaDecline::Diverted(reason)) => {
                    // The creation-site fact changed: the OWNER route owns
                    // the edit (the owner re-resolves; unchanged siblings are
                    // reused). Report the lambda reason if the owner declines.
                    let owner_idx = match &self.units[lambda_idx].kind {
                        UnitKind::Lambda { owner, .. } => *owner,
                        _ => unreachable!("lambda unit"),
                    };
                    return match self.edit_owner(owner_idx, range, replacement)? {
                        Ok(candidate) => Ok(Ok(candidate)),
                        Err(NonFastReason::OwnerSignatureChanged) => {
                            Ok(Err(NonFastReason::OwnerSignatureChanged))
                        }
                        Err(other) => Ok(Err(match other {
                            NonFastReason::OwnerTopologyChanged { .. } => other,
                            _ => NonFastReason::LambdaSiteChanged(reason),
                        })),
                    };
                }
            }
        }
        // Otherwise the owner whose ITEM contains the range.
        if let Some(owner_idx) = owner_containing(&self.units, &range) {
            return self.edit_owner(owner_idx, range, replacement);
        }
        Ok(Err(NonFastReason::CrossesUnits))
    }

    /// The lambda-only fast route.
    fn edit_lambda(
        &mut self,
        target_idx: usize,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<Result<ClosuresFnCandidate, LambdaDecline>, Vec<Diagnostic>> {
        let UnitKind::Lambda { site, .. } = &self.units[target_idx].kind else {
            unreachable!("lambda unit")
        };
        if site.nested != 0 {
            return Ok(Err(LambdaDecline::Diverted(LambdaNonFast::NestedLambdas)));
        }
        let unit_name = self.units[target_idx].name.clone();
        let old_span = self.units[target_idx].span;
        let (new_source, delta) = splice(&self.source, &range, replacement);
        let new_span = Span {
            start: old_span.start,
            end: (old_span.end as i64 + delta) as usize,
        };
        // P axis: one fragment parse for the changed unit only (wire-keyed).
        body_work::record_attempt(StageAxis::ParseProject, &format!("d_{unit_name}"));
        let expr = parse_closure_fragment_via_cst(
            &new_source,
            new_span.start..new_span.end,
            SourceProfile::Closures,
        )
        .map_err(|d| vec![d])?;
        // P axis: one resolve of the changed unit only (inside).
        let state = match resolve_lambda_in_context(&self.env, site, &expr) {
            Err(LambdaReResolve::NonFast(why)) => return Ok(Err(LambdaDecline::Diverted(why))),
            Err(LambdaReResolve::Diagnostics(ds)) => return Err(ds),
            Ok(state) => state,
        };
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
            units: vec![CandidateUnit {
                fn_index: target_idx,
                name: unit_name,
                state: new_state,
            }],
            owner: None,
            parts,
            receipt,
            new_sha: sha256_hex(new_source.as_bytes()),
            new_source,
            shift: (range.end, delta),
        }))
    }

    /// The owner route (PLAN-912 T-03): re-parse the owner ITEM fragment,
    /// re-resolve it through the retained-lambda reuse oracle, graft the
    /// owner plus only the lambdas that genuinely re-resolved, typecheck
    /// those, emit them as one batch document, restore.
    fn edit_owner(
        &mut self,
        owner_idx: usize,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<Result<ClosuresFnCandidate, NonFastReason>, Vec<Diagnostic>> {
        // PLAN-913: a restored context carries declarations only — the owner
        // route would push retained lambda BODIES that do not exist here. It
        // declines by name so the caller runs the whole-module truth path
        // (whose production is reported on the whole axis, not hidden).
        if self.restored {
            return Ok(Err(NonFastReason::RestoredContext));
        }
        let UnitKind::Owner { lambdas } = &self.units[owner_idx].kind else {
            unreachable!("owner unit")
        };
        let lambda_indices = lambdas.clone();
        let owner_name = self.units[owner_idx].name.clone();
        let old_span = self.units[owner_idx].span;
        let (new_source, delta) = splice(&self.source, &range, replacement);
        let new_span = Span {
            start: old_span.start,
            end: (old_span.end as i64 + delta) as usize,
        };
        // P axis: one fragment parse — the owner ITEM (its nested lambda
        // text is part of the owner body; no lambda is resolved by this).
        body_work::record_attempt(StageAxis::ParseProject, &format!("d_{owner_name}"));
        let item = parse_function_fragment_via_cst(
            &new_source,
            new_span.start..new_span.end,
            SourceProfile::Closures,
        )
        .map_err(|d| vec![d])?;
        if item.name.name != owner_name {
            return Ok(Err(NonFastReason::OwnerSignatureChanged));
        }
        // The reuse oracle: this owner's retained lambdas in post-order.
        let mut entries = Vec::with_capacity(lambda_indices.len());
        for &li in &lambda_indices {
            let UnitKind::Lambda { site, .. } = &self.units[li].kind else {
                unreachable!("lambda unit")
            };
            let span = self.units[li].span;
            entries.push(ReuseEntry {
                text_fp: fnv64(self.source[span.start..span.end].as_bytes()),
                site: site.clone(),
                body: self.env.program.fns[li].clone(),
            });
        }
        entries.sort_by_key(|e| e.site.ordinal);
        let recheck = match resolve_owner_in_env(&self.env, &item, owner_idx, entries, &new_source)
        {
            Ok(r) => r,
            Err(ds) => {
                if ds.iter().any(|d| d.code == "source.incremental") {
                    return Ok(Err(NonFastReason::OwnerSignatureChanged));
                }
                return Err(ds);
            }
        };
        if recheck.lambdas.len() != lambda_indices.len() {
            return Ok(Err(NonFastReason::OwnerTopologyChanged {
                before: lambda_indices.len(),
                after: recheck.lambdas.len(),
            }));
        }
        // Position-based identity: retained ordinal k ↔ new ordinal k.
        let mut by_ordinal: BTreeMap<usize, usize> = BTreeMap::new();
        for &li in &lambda_indices {
            if let UnitKind::Lambda { site, .. } = &self.units[li].kind {
                by_ordinal.insert(site.ordinal, li);
            }
        }
        let reused: BTreeSet<usize> = recheck.reused.iter().copied().collect();
        let env_id = self.env.env_id();
        let base_generation = self.env.generation();
        // Graft the owner and the re-resolved lambdas; keep the displaced
        // states for restore.
        let mut displaced: Vec<(usize, BodyState)> = Vec::new();
        let mut units: Vec<CandidateUnit> = Vec::new();
        let mut sites: Vec<(usize, LambdaSiteFacts)> = Vec::new();
        let owner_prev = recheck.state.swap_into(&mut self.env, owner_idx);
        displaced.push((owner_idx, owner_prev));
        let mut changed_lambda_slots: Vec<usize> = Vec::new();
        for (lambda, site) in recheck.lambdas.into_iter().zip(recheck.sites.into_iter()) {
            let Some(&slot) = by_ordinal.get(&site.ordinal) else {
                // Should not happen with equal counts; treat as topology.
                for (idx, state) in displaced.into_iter().rev() {
                    state.swap_into(&mut self.env, idx);
                }
                return Ok(Err(NonFastReason::OwnerTopologyChanged {
                    before: lambda_indices.len(),
                    after: lambda_indices.len(),
                }));
            };
            if reused.contains(&site.ordinal) {
                // Unchanged body: nothing to graft; its site facts carry the
                // re-derived capture sources and span.
                sites.push((slot, site));
                continue;
            }
            // Re-resolved: graft (no pools of its own; the owner's
            // transaction carries the table growth).
            let state = BodyState::entry_only(lambda);
            let prev = state.swap_into(&mut self.env, slot);
            displaced.push((slot, prev));
            changed_lambda_slots.push(slot);
            sites.push((slot, site));
        }
        // Typecheck exactly the owner + the re-resolved lambdas.
        let mut check = super::typecheck::check_body_in(&self.env.program, owner_idx);
        for &slot in &changed_lambda_slots {
            check.extend(super::typecheck::check_body_in(&self.env.program, slot));
        }
        if !check.is_empty() {
            for (idx, state) in displaced.into_iter().rev() {
                state.swap_into(&mut self.env, idx);
            }
            return Err(check);
        }
        let mut indices = vec![owner_idx];
        indices.extend(changed_lambda_slots.iter().copied());
        let emit =
            emit_closures_function_batch(&self.env, &indices, &new_source, &self.module_path);
        let (parts, receipt) = match emit {
            Ok(ok) => ok,
            Err(ds) => {
                for (idx, state) in displaced.into_iter().rev() {
                    state.swap_into(&mut self.env, idx);
                }
                return Err(ds);
            }
        };
        // Restore current; the candidate owns the new states.
        for (idx, prev) in displaced.into_iter().rev() {
            let new_state = prev.swap_into(&mut self.env, idx);
            let name = self.units[idx].name.clone();
            units.push(CandidateUnit {
                fn_index: idx,
                name,
                state: new_state,
            });
        }
        units.reverse(); // owner first, then lambdas in graft order
        Ok(Ok(ClosuresFnCandidate {
            env_id,
            base_generation,
            units,
            owner: Some(OwnerUpdate {
                owner_index: owner_idx,
                sites,
                reused_ordinals: recheck.reused,
            }),
            parts,
            receipt,
            new_sha: sha256_hex(new_source.as_bytes()),
            new_source,
            shift: (range.end, delta),
        }))
    }

    /// Commit ONE verified edit: installs the prepared unit entries, swaps
    /// the source, remaps the catalog coordinates (M axis — spans only,
    /// never a resolved body walk), installs the verified registry overlay
    /// (owner route) and publishes a new generation.
    pub fn commit_candidate(
        &mut self,
        candidate: ClosuresFnCandidate,
        registry_after: Option<ClosuresVerifyContext>,
        facts: &[UnitCommitFacts],
    ) -> Result<(), String> {
        if self.env.env_id() != candidate.env_id {
            return Err(
                "source.foreign: the candidate was prepared against another environment"
                    .to_string(),
            );
        }
        if self.env.generation() != candidate.base_generation {
            return Err(
                "source.stale: the base has been published since the candidate was prepared"
                    .to_string(),
            );
        }
        let primary = candidate.units[0].fn_index;
        for unit in candidate.units {
            let displaced = unit.state.swap_into(&mut self.env, unit.fn_index);
            debug_assert_eq!(displaced.entry.name, unit.name);
            let _ = displaced;
        }
        self.source = candidate.new_source;
        self.source_sha = candidate.new_sha;
        // M axis: shift every catalog span after the edit point (plus the
        // containing units' ends). No resolved body is visited.
        body_work::record_whole(StageAxis::CoordRemap);
        let (edit_end, delta) = candidate.shift;
        if delta != 0 {
            for u in self.units.iter_mut() {
                shift_span(&mut u.span, edit_end, delta);
                shift_span(&mut u.body_span, edit_end, delta);
                if let UnitKind::Lambda { site, .. } = &mut u.kind {
                    site.expr_span = u.span;
                }
            }
        }
        if let Some(owner) = candidate.owner {
            // Owner route: every lambda of the owner carries its post-walk
            // site facts (re-derived capture sources; reused or re-resolved).
            // Their spans were produced in NEW coordinates by the walk, so
            // they are authoritative over the remapped catalog spans.
            for (slot, site) in owner.sites {
                let span = site.expr_span;
                if let UnitKind::Lambda { site: retained, .. } = &mut self.units[slot].kind {
                    *retained = site;
                }
                self.units[slot].span = span;
                self.units[slot].body_span = span;
            }
        }
        let _ = primary;
        if let Some(reg) = registry_after {
            self.registry = reg;
        }
        for f in facts {
            if let Some(u) = self.units.get_mut(f.fn_index) {
                u.body_digest = f.body_digest.clone();
            }
            self.context_revision = f.context_revision.clone();
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

/// Internal: why the lambda route handed the edit to the owner route.
enum LambdaDecline {
    Diverted(LambdaNonFast),
}

fn splice(source: &str, range: &Range<usize>, replacement: &str) -> (String, i64) {
    let delta = replacement.len() as i64 - (range.end - range.start) as i64;
    let mut out = String::with_capacity((source.len() as i64 + delta).max(0) as usize);
    out.push_str(&source[..range.start]);
    out.push_str(replacement);
    out.push_str(&source[range.end..]);
    (out, delta)
}

/// Prefix/suffix single-region diff: the maximal common prefix and suffix;
/// the middle is one replacement range. Both endpoints are clamped to UTF-8
/// char boundaries of both `old` and `new`, so splicing `replacement` into
/// `old` at `range` reconstructs `new` byte-for-byte for any valid UTF-8 inputs.
pub fn diff_single_region(old: &str, new: &str) -> (Range<usize>, String) {
    let ob = old.as_bytes();
    let nb = new.as_bytes();
    let max_prefix = ob.iter().zip(nb.iter()).take_while(|(a, b)| a == b).count();
    let mut prefix = max_prefix;
    while prefix > 0 && (!old.is_char_boundary(prefix) || !new.is_char_boundary(prefix)) {
        prefix -= 1;
    }
    let max_suffix = ob[prefix..]
        .iter()
        .rev()
        .zip(nb[prefix..].iter().rev())
        .take_while(|(a, b)| a == b)
        .count();
    let mut suffix = max_suffix;
    while suffix > 0
        && (!old.is_char_boundary(old.len() - suffix) || !new.is_char_boundary(new.len() - suffix))
    {
        suffix -= 1;
    }
    let range = prefix..old.len() - suffix;
    let replacement = new[prefix..new.len() - suffix].to_string();
    (range, replacement)
}

/// Shift a catalog span across an edit at `edit_end` by `delta`: spans that
/// start at/after the edit point move whole; spans that contain the edit
/// point grow/shrink at their end; spans before it are untouched.
fn shift_span(span: &mut Span, edit_end: usize, delta: i64) {
    if span.start >= edit_end {
        span.start = (span.start as i64 + delta) as usize;
        span.end = (span.end as i64 + delta) as usize;
    } else if span.end >= edit_end {
        span.end = (span.end as i64 + delta) as usize;
    }
}

fn count_resolved_nodes(f: &super::resolve::ResolvedFn) -> usize {
    fn block(b: &super::resolve::RBlock) -> usize {
        b.stmts.iter().map(stmt).sum()
    }
    fn stmt(s: &super::resolve::RStmt) -> usize {
        1 + match s {
            super::resolve::RStmt::Let { value, .. }
            | super::resolve::RStmt::Assign { value, .. }
            | super::resolve::RStmt::Return { value, .. } => expr(value),
            super::resolve::RStmt::If {
                cond, then, els, ..
            } => expr(cond) + block(then) + els.as_ref().map(block).unwrap_or(0),
            super::resolve::RStmt::While { cond, body, .. } => expr(cond) + block(body),
            super::resolve::RStmt::Loop { body, .. } => block(body),
            _ => 0,
        }
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

/// Innermost LAMBDA whose span contains the range.
fn innermost_lambda(units: &[UnitEntry], range: &Range<usize>) -> Option<usize> {
    let mut best: Option<usize> = None;
    for (i, u) in units.iter().enumerate() {
        if matches!(u.kind, UnitKind::Lambda { .. })
            && u.span.start <= range.start
            && range.end <= u.span.end
        {
            match best {
                Some(b) if units[b].span.end - units[b].span.start <= u.span.end - u.span.start => {
                }
                _ => best = Some(i),
            }
        }
    }
    best
}

/// The OWNER whose item span contains the range (body or signature; the
/// resolver's drift guard decides whether a signature edit is non-fast).
fn owner_containing(units: &[UnitEntry], range: &Range<usize>) -> Option<usize> {
    units.iter().position(|u| {
        matches!(u.kind, UnitKind::Owner { .. })
            && u.span.start <= range.start
            && range.end <= u.span.end
    })
}
