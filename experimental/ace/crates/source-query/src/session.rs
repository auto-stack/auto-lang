//! The ordinary-source query session (PLAN-934 §5.2/§5.3/§5.5).
//!
//! One session answers per-owner queries against the CURRENT module:
//!
//! * every answer is addressed by a COMPLETE [`QueryKey`] — logical owner,
//!   signature axis, body axis, profile/algorithm/schema/producer/target
//!   semantics and the caller's capability set; a change on any axis is a
//!   miss, never a reinterpretation;
//! * a hit additionally requires every recorded positive dependency
//!   (callee signature versions, self signature) and the negative
//!   candidate set to still hold — never "the module source hash changed"
//!   and never "not found, so solve it";
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

use crate::catalog::SignatureCatalog;
use crate::facts::{FactsError, FnSigFacts, InferBodyFacts, NegativeDep};
use crate::ident::owner_logical;
use crate::keys::QueryKey;
use crate::produce::{produce_infer_body, ProduceError, ProduceLimits};
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
    /// PLAN-934 T-08: product requests whose fact layer reported a
    /// LANGUAGE diagnostic (a page carrying solver diagnostics or missing
    /// coverage) and whose verdict therefore came from the canonical
    /// diagnostic oracle. Zero on every accepted program: the fact layer
    /// produces the types, it never asks the legacy engine for a verdict.
    pub diagnostic_verdicts: u64,
}

/// Query failure. `SolverDisabled` exists so a negative control can prove
/// that a cold miss REALLY goes through the solver and a warm hit does
/// not.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum QueryError {
    Produce(ProduceError),
    /// The negative-control switch is off and the answer would have
    /// required a solver run.
    SolverDisabled {
        query: &'static str,
        owner: String,
    },
    /// A restored page was refused against the current source (stale,
    /// foreign, incomplete or carrying diagnostics).
    Reject(FactsError),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::Produce(e) => write!(f, "{e}"),
            QueryError::SolverDisabled { query, owner } => write!(
                f,
                "{query} for `{owner}` misses the query cache and the solver is disabled"
            ),
            QueryError::Reject(e) => write!(f, "restored page refused: {e}"),
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

/// Why one cached page is (not) reusable right now.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageAudit {
    pub name: String,
    pub fresh: bool,
    /// `"ok"`, `"body-changed"`, `"signature-changed"`, `"key-changed"`,
    /// `"dep-changed"` or `"negative-flipped"`.
    pub reason: &'static str,
}

struct SigPage {
    facts: FnSigFacts,
    key: QueryKey,
}

struct BodyPage {
    name: String,
    facts: InferBodyFacts,
    key: QueryKey,
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
    capabilities: Vec<String>,
    declaration_contexts: BTreeMap<String, String>,
    limits: ProduceLimits,
    allow_solver: bool,
    cancel: Option<Arc<AtomicBool>>,
    revision: u64,
    sigs: BTreeMap<String, SigPage>,
    bodies: BTreeMap<LogicalId, BodyPage>,
    counters: Counters,
}

impl SourceQuerySession {
    pub fn new(package: &str, module: &str) -> Self {
        SourceQuerySession {
            package: package.to_string(),
            module: module.to_string(),
            capabilities: Vec::new(),
            declaration_contexts: BTreeMap::new(),
            limits: ProduceLimits::default(),
            allow_solver: true,
            cancel: None,
            revision: 0,
            sigs: BTreeMap::new(),
            bodies: BTreeMap::new(),
            counters: Counters::default(),
        }
    }

    /// The caller's capability set participates in every key: enabling or
    /// disabling a capability can change what the source means, so old
    /// answers are never reused across it.
    pub fn with_capabilities<I, S>(mut self, caps: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = caps.into_iter().map(Into::into).collect();
        self
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

    /// Change the capability set in place (used by hosts when a capability
    /// is granted/revoked between edits). Every cached key stops matching,
    /// so the next query re-solves instead of reusing a different meaning.
    pub fn set_capabilities<I, S>(&mut self, caps: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.capabilities = caps.into_iter().map(Into::into).collect();
    }

    pub fn counters(&self) -> Counters {
        self.counters
    }

    /// PLAN-934 T-08: record that one product request left the facts lane
    /// because the query answered with a language diagnostic verdict
    /// (never because a fact was missing for an accepted program).
    pub fn note_diagnostic_verdict(&mut self) {
        self.counters.diagnostic_verdicts += 1;
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn capabilities(&self) -> &[String] {
        &self.capabilities
    }

    pub fn package(&self) -> &str {
        &self.package
    }

    pub fn module(&self) -> &str {
        &self.module
    }

    /// Drop every cached page (test/measurement helper; a production
    /// consumer relies on invalidation, never on clearing the store to
    /// hide a stale hit).
    pub fn clear(&mut self) {
        self.sigs.clear();
        self.bodies.clear();
    }

    /// The current module's signature catalog (declarations only: building
    /// it never reads a body).
    pub fn catalog(&self, program: &Program) -> Result<SignatureCatalog, QueryError> {
        SignatureCatalog::build(program, &self.package, &self.module)
            .and_then(|c| c.with_declaration_contexts(&self.declaration_contexts))
            .map_err(QueryError::Produce)
    }

    pub fn declaration_contexts(&self) -> &BTreeMap<String, String> {
        &self.declaration_contexts
    }

    /// Existing positive read sets identify affected declarations without
    /// walking, parsing or solving any unrelated body. Used when the host
    /// changes explicit requirements while source text stays identical.
    pub fn declaration_readers(
        &self,
        changed: &std::collections::BTreeSet<String>,
    ) -> std::collections::BTreeSet<String> {
        let mut result = changed.clone();
        result.extend(
            self.bodies
                .values()
                .filter(|page| page.sig_deps.iter().any(|(name, _)| changed.contains(name)))
                .map(|page| page.name.clone()),
        );
        result
    }

    /// Candidate contexts are validated against current declarations before
    /// replacing the session's inputs. This does not flush unrelated pages.
    pub fn set_declaration_contexts(
        &mut self,
        program: &Program,
        contexts: BTreeMap<String, String>,
    ) -> Result<(), QueryError> {
        SignatureCatalog::build(program, &self.package, &self.module)
            .and_then(|c| c.with_declaration_contexts(&contexts))
            .map_err(QueryError::Produce)?;
        self.declaration_contexts = contexts;
        Ok(())
    }

    /// Complete key of one signature query against the current module.
    pub fn sig_key(&self, program: &Program, name: &str) -> Result<QueryKey, QueryError> {
        let catalog = self.catalog(program)?;
        let version = catalog.current_version(name).ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        Ok(QueryKey::fn_sig(
            owner_logical(&self.package, &self.module, name),
            version,
            &self.capabilities,
        ))
    }

    /// Complete key of one infer-body query against the current module.
    pub fn body_key(&self, program: &Program, name: &str) -> Result<QueryKey, QueryError> {
        let catalog = self.catalog(program)?;
        let f = find_fn(program, name).ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        let walk = walk_body(&f.body).map_err(|e| QueryError::Produce(ProduceError::Walk(e)))?;
        let sig = catalog.current_version(name).ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        Ok(QueryKey::infer_body(
            owner_logical(&self.package, &self.module, name),
            sig,
            walk.version,
            &self.capabilities,
        ))
    }

    /// Seed one RESTORED page into the cache after deep verification
    /// against the CURRENT source. The page is refused unless it describes
    /// this body version exactly (never trusted because it came from the
    /// store), and its logical id must be this module's own derivation.
    pub fn seed(&mut self, program: &Program, page: &InferBodyFacts) -> Result<(), QueryError> {
        let name = page.name.clone();
        let f = find_fn(program, &name)
            .ok_or_else(|| QueryError::Produce(ProduceError::UnknownFn { name: name.clone() }))?;
        let walk = walk_body(&f.body).map_err(|e| QueryError::Produce(ProduceError::Walk(e)))?;
        page.validate_against(&walk).map_err(QueryError::Reject)?;
        page.admit(&walk).map_err(QueryError::Reject)?;
        let own = owner_logical(&self.package, &self.module, &name);
        if page.logical != own {
            return Err(QueryError::Reject(FactsError::BodyVersionMismatch {
                facts: page.logical.as_str().to_string(),
                body: own.as_str().to_string(),
            }));
        }
        let catalog = self.catalog(program)?;
        let sig_now = catalog
            .current_version(&name)
            .ok_or_else(|| QueryError::Produce(ProduceError::UnknownFn { name: name.clone() }))?;
        let key = QueryKey::infer_body(
            own.clone(),
            sig_now.clone(),
            walk.version.clone(),
            &self.capabilities,
        );
        let mut sig_deps = vec![(name.clone(), sig_now)];
        for callee in &page.sig_reads {
            if callee == &name {
                continue;
            }
            if let Some(v) = catalog.version(callee) {
                sig_deps.push((callee.clone(), v.clone()));
            }
        }
        self.bodies.insert(
            own,
            BodyPage {
                name,
                facts: page.clone(),
                key,
                sig_deps,
                negatives: page.negatives.clone(),
                fn_candidates_fp: catalog.candidates_fp().to_string(),
            },
        );
        Ok(())
    }
    /// Signature query: reads ONLY the declaration axis (through the
    /// signature catalog). A body-only edit must keep this a hit.
    pub fn fn_sig(
        &mut self,
        program: &Program,
        name: &str,
    ) -> Result<QueryOutcome<FnSigFacts>, QueryError> {
        let catalog = self.catalog(program)?;
        self.fn_sig_with(program, &catalog, name)
    }

    /// PLAN-934 T-08: the same signature query against a catalog the
    /// caller already built. A whole-module compile queries every body, so
    /// rebuilding the declaration catalog per query would make one compile
    /// O(functions²) hashing for no semantic gain.
    pub fn fn_sig_with(
        &mut self,
        _program: &Program,
        catalog: &SignatureCatalog,
        name: &str,
    ) -> Result<QueryOutcome<FnSigFacts>, QueryError> {
        self.revision += 1;
        let current = catalog.get(name).cloned().ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        let key = QueryKey::fn_sig(
            current.logical.clone(),
            current.version.clone(),
            &self.capabilities,
        );
        if let Some(page) = self.sigs.get(name) {
            if page.key == key {
                self.counters.sig_hits += 1;
                return Ok(QueryOutcome {
                    facts: page.facts.clone(),
                    hit: true,
                });
            }
        }
        self.counters.sig_misses += 1;
        self.sigs.insert(
            name.to_string(),
            SigPage {
                facts: current.clone(),
                key,
            },
        );
        Ok(QueryOutcome {
            facts: current,
            hit: false,
        })
    }

    /// Infer-body query: a hit requires the COMPLETE key (all axes +
    /// capabilities) AND every recorded dependency to still hold.
    pub fn infer_body(
        &mut self,
        program: &Program,
        name: &str,
    ) -> Result<QueryOutcome<InferBodyFacts>, QueryError> {
        let catalog = self.catalog(program)?;
        self.infer_body_with(program, &catalog, name)
    }

    /// Read-only diagnostic projection; never runs the solver on rejection.
    pub(crate) fn cached_body_facts(
        &self,
        program: &Program,
        name: &str,
    ) -> Option<&InferBodyFacts> {
        let key = self.body_key(program, name).ok()?;
        let catalog = self.catalog(program).ok()?;
        self.bodies
            .values()
            .find(|p| p.name == name && p.key == key && page_deps_fresh(p, &catalog).is_ok())
            .map(|p| &p.facts)
    }

    /// PLAN-934 T-08: the same body query against a caller-built catalog
    /// (see [`Self::fn_sig_with`]).
    pub fn infer_body_with(
        &mut self,
        program: &Program,
        catalog: &SignatureCatalog,
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
        let sig_now = catalog.current_version(name).ok_or_else(|| {
            QueryError::Produce(ProduceError::UnknownFn {
                name: name.to_string(),
            })
        })?;
        let key = QueryKey::infer_body(
            logical.clone(),
            sig_now,
            walk.version.clone(),
            &self.capabilities,
        );

        if let Some(page) = self.bodies.get(&logical) {
            if page.key == key && page_deps_fresh(page, &catalog).is_ok() {
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
            &self.capabilities,
            self.limits,
            self.cancel.clone(),
        )
        .map_err(QueryError::Produce)?;
        self.counters.solver_runs += 1;
        facts.revision = rev;

        let mut sig_deps = Vec::new();
        if let Some(v) = catalog.version(name) {
            sig_deps.push((name.to_string(), v.clone()));
        }
        for callee in &facts.sig_reads {
            if callee == name {
                continue;
            }
            if let Some(v) = catalog.version(callee) {
                sig_deps.push((callee.clone(), v.clone()));
            }
        }
        let page = BodyPage {
            name: name.to_string(),
            facts: facts.clone(),
            key,
            sig_deps,
            negatives: facts.negatives.clone(),
            fn_candidates_fp: catalog.candidates_fp().to_string(),
        };
        self.bodies.insert(logical, page);
        Ok(QueryOutcome { facts, hit: false })
    }

    /// Audit every cached body page against the CURRENT module: the
    /// complete key (all axes + capabilities) plus the positive and
    /// negative dependencies. This is the same check a hit performs, made
    /// observable per page.
    pub fn audit(&self, program: &Program) -> Result<Vec<PageAudit>, QueryError> {
        let catalog = self.catalog(program)?;
        let mut out = Vec::new();
        for page in self.bodies.values() {
            let Some(f) = find_fn(program, &page.name) else {
                out.push(PageAudit {
                    name: page.name.clone(),
                    fresh: false,
                    reason: "key-changed",
                });
                continue;
            };
            let Ok(walk) = walk_body(&f.body) else {
                out.push(PageAudit {
                    name: page.name.clone(),
                    fresh: false,
                    reason: "key-changed",
                });
                continue;
            };
            let Some(sig_now) = catalog.current_version(&page.name) else {
                out.push(PageAudit {
                    name: page.name.clone(),
                    fresh: false,
                    reason: "key-changed",
                });
                continue;
            };
            let key = QueryKey::infer_body(
                page.facts.logical.clone(),
                sig_now,
                walk.version,
                &self.capabilities,
            );
            let reason = if page.key.body_version != key.body_version {
                "body-changed"
            } else if page.key.sig_version != key.sig_version {
                "signature-changed"
            } else if page.key.capabilities != key.capabilities
                || page.key.profile != key.profile
                || page.key.algo != key.algo
                || page.key.schema != key.schema
                || page.key.producer != key.producer
                || page.key.target != key.target
            {
                "key-changed"
            } else {
                match page_deps_fresh(page, &catalog) {
                    Ok(()) => "ok",
                    Err(r) => r,
                }
            };
            out.push(PageAudit {
                name: page.name.clone(),
                fresh: reason == "ok",
                reason,
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
}

/// Dependency validation of a cached page against the CURRENT catalog:
/// every recorded positive must still resolve to the same version and,
/// when the page depended on an ABSENT name, the candidate set must be
/// unchanged. Both reads are declaration-only.
fn page_deps_fresh(page: &BodyPage, catalog: &SignatureCatalog) -> Result<(), &'static str> {
    for (name, version) in &page.sig_deps {
        match catalog.version(name) {
            Some(current) if current == version => {}
            _ => return Err("dep-changed"),
        }
    }
    if page.negatives.iter().any(|n| n.domain == "fn")
        && page.fn_candidates_fp != catalog.candidates_fp()
    {
        return Err("negative-flipped");
    }
    Ok(())
}

fn find_fn<'a>(program: &'a Program, name: &str) -> Option<&'a FnItem> {
    program.items.iter().find_map(|item| match item {
        Item::Fn(f) if f.name.name == name => Some(f),
        _ => None,
    })
}
