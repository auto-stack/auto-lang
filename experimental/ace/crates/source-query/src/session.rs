//! The ordinary-source query session (PLAN-934 §5.2/§5.3/§5.5).
//!
//! One session answers per-owner queries against the CURRENT module:
//!
//! * a hit requires BOTH the body version and every recorded positive
//!   dependency (callee signature versions, self signature) plus the
//!   negative-dependency candidate set to still match — never "the module
//!   source hash changed" and never "not found, so solve it";
//! * a miss solves through the reused 796 solver under the frozen budget
//!   and publishes an immutable page bound to the request revision;
//! * the solver can be disabled for a NEGATIVE CONTROL (a warm hit must
//!   still succeed with no producer at all, a cold miss must fail) — that
//!   is how "the fact is the authority" is observable rather than
//!   assumed.
//!
//! Consumers are the AV/native/A2X hosts: they ask for the facts of the
//! bodies they are (re)building and install them together with the
//! target-ready record. This crate never builds a target itself.

use crate::facts::{FnSigFacts, InferBodyFacts, NegativeDep};
use crate::ident::{owner_logical, signature_version, sq_hash};
use crate::produce::{produce_fn_sig, produce_infer_body, ProduceError, ProduceLimits};
use crate::walk::walk_body;
use ace_core::source::ast::{FnItem, Item, Program};
use ace_semantics_contract::{ContentVersion, LogicalId};
use std::collections::BTreeMap;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

/// Production counters of one session (real producer-side numbers, never
/// reconstructed by a test from "the result looked right").
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// Solver executions (cold or invalidated bodies only).
    pub solver_runs: u64,
    pub sig_hits: u64,
    pub sig_misses: u64,
    pub body_hits: u64,
    pub body_misses: u64,
    /// Queries rejected before any solve (disabled solver, rejection).
    pub rejections: u64,
}

/// Query failure. `SolverDisabled` exists so a negative control can prove
/// that a cold miss REALLY goes through the solver and a warm hit does
/// not.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueryError {
    Produce(ProduceError),
    /// The negative-control switch is off and the answer would have
    /// required a solver run.
    SolverDisabled { query: &'static str, owner: String },
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Produce(e) => write!(f, "{e}"),
            QueryError::SolverDisabled { query, owner } => write!(
                f,
                "{query} for `{owner}` misses the query cache and the solver is disabled"
            ),
        }
    }
}

/// A query answer plus its provenance: `hit == false` means a real solve
/// happened for this revision.
#[derive(Clone, Debug)]
pub struct QueryOutcome<T> {
    pub facts: T,
    pub hit: bool,
}

struct BodyPage {
    facts: InferBodyFacts,
    /// Positive dependencies: callee name → signature version read.
    sig_deps: Vec<(String, ContentVersion)>,
    /// Negative dependencies (name absent at solve time).
    negatives: Vec<NegativeDep>,
    /// Function candidate-set fingerprint at solve time.
    fn_candidates_fp: String,
}

/// The per-module query session.
pub struct SourceQuerySession {
    package: String,
    module: String,
    limits: ProduceLimits,
    allow_solver: bool,
    cancel: Option<Arc<AtomicBool>>,
    revision: u64,
    sigs: BTreeMap<String, FnSigFacts>,
    bodies: BTreeMap<LogicalId, BodyPage>,
    counters: Counters,
}

impl SourceQuerySession {
    pub fn new(package: &str, module: &str) -> Self {
        SourceQuerySession {
            package: package.to_string(),
            module: module.to_string(),
            limits: ProduceLimits::default(),
            allow_solver: true,
            cancel: None,
            revision: 0,
            sigs: BTreeMap::new(),
            bodies: BTreeMap::new(),
            counters: Counters::default(),
        }
    }

    pub fn with_limits(mut self, limits: ProduceLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn with_solver(mut self, allow: bool) -> Self {
        self.allow_solver = allow;
        self
    }

    pub fn with_cancel(mut self, flag: Arc<AtomicBool>) -> Self {
        self.cancel = Some(flag);
        self
    }

    pub fn set_allow_solver(&mut self, allow: bool) {
        self.allow_solver = allow;
    }

    pub fn counters(&self) -> Counters {
        self.counters
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    /// Drop every cached page (test/measurement helper; a production
    /// consumer relies on invalidation, never on clearing the store to
    /// hide a stale hit).
    pub fn clear(&mut self) {
        self.sigs.clear();
        self.bodies.clear();
    }

    /// Signature query: reads ONLY the declaration axis. A body-only edit
    /// must keep this a hit.
    pub fn fn_sig(
        &mut self,
        program: &Program,
        name: &str,
    ) -> Result<QueryOutcome<FnSigFacts>, QueryError> {
        self.revision += 1;
        let current = produce_fn_sig(program, &self.package, &self.module, name)
            .map_err(QueryError::Produce)?;
        if let Some(cached) = self.sigs.get(name) {
            if cached.version == current.version {
                self.counters.sig_hits += 1;
                return Ok(QueryOutcome {
                    facts: cached.clone(),
                    hit: true,
                });
            }
        }
        self.counters.sig_misses += 1;
        self.sigs.insert(name.to_string(), current.clone());
        Ok(QueryOutcome {
            facts: current,
            hit: false,
        })
    }

    /// Infer-body query: a hit requires the body version AND every
    /// recorded dependency to still hold.
    pub fn infer_body(
        &mut self,
        program: &Program,
        name: &str,
    ) -> Result<QueryOutcome<InferBodyFacts>, QueryError> {
        self.revision += 1;
        let rev = self.revision;
        let f = find_fn(program, name).ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        let walk = walk_body(&f.body).map_err(|e| QueryError::Produce(ProduceError::Walk(e)))?;
        let logical = owner_logical(&self.package, &self.module, name);

        if let Some(page) = self.bodies.get(&logical) {
            if page.facts.body_version == walk.version && self.page_is_fresh(page, program) {
                self.counters.body_hits += 1;
                let mut facts = page.facts.clone();
                facts.revision = rev;
                return Ok(QueryOutcome { facts, hit: true });
            }
        }

        self.counters.body_misses += 1;
        if !self.allow_solver {
            self.counters.rejections += 1;
            return Err(QueryError::SolverDisabled {
                query: "infer-body",
                owner: name.to_string(),
            });
        }
        let mut facts = produce_infer_body(
            program,
            &self.package,
            &self.module,
            name,
            self.limits,
            self.cancel.clone(),
        )
        .map_err(QueryError::Produce)?;
        self.counters.solver_runs += 1;
        facts.revision = rev;

        let mut sig_deps = vec![(name.to_string(), signature_version(f))];
        for callee in &facts.sig_reads {
            if callee == name {
                continue;
            }
            if let Some(cf) = find_fn(program, callee) {
                sig_deps.push((callee.clone(), signature_version(cf)));
            }
        }
        let page = BodyPage {
            facts: facts.clone(),
            sig_deps,
            negatives: facts.negatives.clone(),
            fn_candidates_fp: fn_candidates_fp(program),
        };
        self.bodies.insert(logical, page);
        Ok(QueryOutcome { facts, hit: false })
    }

    /// Dependency validation of a cached page against the CURRENT module:
    /// body version is checked by the caller; here every recorded positive
    /// must match and, when the page depended on an ABSENT name, the
    /// candidate set must be unchanged.
    fn page_is_fresh(&self, page: &BodyPage, program: &Program) -> bool {
        for (name, version) in &page.sig_deps {
            match find_fn(program, name) {
                Some(f) if &signature_version(f) == version => {}
                _ => return false,
            }
        }
        if page.negatives.iter().any(|n| n.domain == "fn")
            && page.fn_candidates_fp != fn_candidates_fp(program)
        {
            return false;
        }
        true
    }
}

fn find_fn<'a>(program: &'a Program, name: &str) -> Option<&'a FnItem> {
    program.items.iter().find_map(|item| match item {
        Item::Fn(f) if f.name.name == name => Some(f),
        _ => None,
    })
}

fn fn_candidates_fp(program: &Program) -> String {
    let mut names: Vec<&str> = program
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Fn(f) => Some(f.name.name.as_str()),
            _ => None,
        })
        .collect();
    names.sort_unstable();
    names.dedup();
    let parts: Vec<&[u8]> = names.iter().map(|n| n.as_bytes()).collect();
    sq_hash("ac934/candidates/fn/v1", &parts)
}
