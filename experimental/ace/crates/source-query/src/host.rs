//! PLAN-934 T-06: the host-side install protocol (two phases, dependency
//! revalidation, cancellation, single-flight, late results).
//!
//! The query layer never installs anything itself. A consumer host owns
//! the CURRENT record — source/semantic root + target handle — and swaps
//! it in ONE step only after every required condition still holds:
//!
//! 1. **prepare**: compute/query the candidate facts for one complete key
//!    under the host's current revision (no install, no target);
//! 2. **target-ready**: the backend lane preparing an executable target
//!    attaches its ready handle;
//! 3. **revalidate**: the base revision is still current, the complete key
//!    still matches the source (body/signature/capabilities), and every
//!    positive dependency version is unchanged;
//! 4. **switch**: publish the immutable page, then replace the current
//!    record once. Any failure before the switch keeps the old record and
//!    its target untouched; a cancelled or late result is never installed.
//!
//! A read-only semantic query returns a view bound to a revision and never
//! builds or installs a target (a hit report is not an executable
//! replacement).
//!
//! Single-flight is per COMPLETE key: one computation is shared, a waiter's
//! own cancellation does not cancel the shared result or another waiter,
//! and the same key with a different payload is a hard rejection (never a
//! silent overwrite, never "clear the store").
//!
//! The `PageSink` seam is where the unified ManagedStore adapter lands; the
//! in-memory sink implements the same protocol for bounded verification.

use crate::facts::InferBodyFacts;
use crate::keys::QueryKey;
use crate::session::{QueryError, SourceQuerySession};
use ace_core::source::ast::Program;
use ace_semantics_contract::ContentVersion;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};

/// Monotone host revision (one per installed switch).
pub type Revision = u64;

/// A backend's ready handle for the target that belongs to one candidate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TargetHandle(pub u64);

/// Where immutable pages go. The ManagedStore adapter implements this; the
/// protocol above does not depend on the storage engine.
pub trait PageSink {
    fn put(&mut self, key: &QueryKey, facts: &InferBodyFacts) -> Result<String, String>;
    fn get(&self, artifact: &str) -> Option<InferBodyFacts>;
}

/// Deterministic in-memory sink for bounded verification.
#[derive(Default)]
pub struct MemorySink {
    pages: BTreeMap<String, (QueryKey, InferBodyFacts)>,
    next: u64,
}

impl MemorySink {
    pub fn new() -> Self {
        MemorySink::default()
    }

    pub fn len(&self) -> usize {
        self.pages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pages.is_empty()
    }
}

impl PageSink for MemorySink {
    fn put(&mut self, key: &QueryKey, facts: &InferBodyFacts) -> Result<String, String> {
        self.next += 1;
        let id = format!("mem-{}-{}", key.canonical(), self.next);
        self.pages.insert(id.clone(), (key.clone(), facts.clone()));
        Ok(id)
    }

    fn get(&self, artifact: &str) -> Option<InferBodyFacts> {
        self.pages.get(artifact).map(|(_, f)| f.clone())
    }
}

/// A prepared candidate: facts for one complete key, bound to the host
/// revision it was computed under, plus the dependency versions it read.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub name: String,
    pub key: QueryKey,
    pub facts: InferBodyFacts,
    pub base: Revision,
    pub deps: Vec<(String, ContentVersion)>,
    pub target: Option<TargetHandle>,
}

/// The host's current record: semantic root, page artifact and target, as
/// ONE value (there is no half-published state).
#[derive(Clone, Debug)]
pub struct CurrentRecord {
    pub revision: Revision,
    pub artifact: String,
    pub key: QueryKey,
    pub facts: InferBodyFacts,
    pub target: Option<TargetHandle>,
}

/// Why an install was refused. Every variant leaves the old record intact.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CommitError {
    /// The request was cancelled before the switch.
    Cancelled,
    /// The host revision advanced after the candidate was prepared: a late
    /// result is never installed.
    LateResult { current: Revision, candidate: Revision },
    /// The source now yields a different complete key (body/signature/
    /// capability change) than the one the candidate was computed for.
    KeyChanged { expected: String, current: String },
    /// A positive dependency changed after the candidate was prepared.
    DepChanged { name: String },
    /// The sink failed to publish; nothing was switched.
    Sink(String),
}

impl std::fmt::Display for CommitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommitError::Cancelled => write!(f, "the install was cancelled"),
            CommitError::LateResult { current, candidate } => write!(
                f,
                "late result for base revision {candidate} (current is {current})"
            ),
            CommitError::KeyChanged { expected, current } => write!(
                f,
                "the source now maps to key {current}, not the prepared {expected}"
            ),
            CommitError::DepChanged { name } => {
                write!(f, "dependency `{name}` changed after the candidate was prepared")
            }
            CommitError::Sink(m) => write!(f, "page sink failed: {m}"),
        }
    }
}

/// A read-only semantic view: bound to a revision, never an executable
/// replacement.
#[derive(Clone, Debug)]
pub struct QueryView {
    pub key: QueryKey,
    pub revision: Revision,
    pub facts: InferBodyFacts,
    pub hit: bool,
}

/// Single-flight failure.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FlightError {
    /// The same complete key produced a different payload: rejected and
    /// located, never silently overwritten.
    PayloadMismatch { key: String },
    /// This waiter was cancelled; the shared flight (if any) is untouched.
    Cancelled,
    /// The shared computation failed.
    Failed(String),
}

impl std::fmt::Display for FlightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlightError::PayloadMismatch { key } => {
                write!(f, "same query key produced two payloads: {key}")
            }
            FlightError::Cancelled => write!(f, "this waiter was cancelled"),
            FlightError::Failed(m) => write!(f, "shared computation failed: {m}"),
        }
    }
}

struct Flight {
    facts: InferBodyFacts,
}

/// The consumer host's install state.
pub struct QueryHost<S: PageSink> {
    revision: Revision,
    current: Option<CurrentRecord>,
    sink: S,
    flights: BTreeMap<String, Flight>,
}

impl<S: PageSink> QueryHost<S> {
    pub fn new(sink: S) -> Self {
        QueryHost {
            revision: 0,
            current: None,
            sink,
            flights: BTreeMap::new(),
        }
    }

    pub fn revision(&self) -> Revision {
        self.revision
    }

    pub fn current(&self) -> Option<&CurrentRecord> {
        self.current.as_ref()
    }

    pub fn sink(&self) -> &S {
        &self.sink
    }

    /// Phase 1 — prepare a candidate for one owner without installing it.
    pub fn prepare(
        &self,
        session: &mut SourceQuerySession,
        program: &Program,
        name: &str,
    ) -> Result<Candidate, QueryError> {
        let key = session.body_key(program, name)?;
        let out = session.infer_body(program, name)?;
        let catalog = session.catalog(program)?;
        let mut deps = Vec::new();
        if let Some(v) = catalog.version(name) {
            deps.push((name.to_string(), v.clone()));
        }
        for callee in &out.facts.sig_reads {
            if callee == name {
                continue;
            }
            if let Some(v) = catalog.version(callee) {
                deps.push((callee.clone(), v.clone()));
            }
        }
        Ok(Candidate {
            name: name.to_string(),
            key,
            facts: out.facts,
            base: self.revision,
            deps,
            target: None,
        })
    }

    /// Phase 2 helper — attach a backend's ready target handle.
    pub fn attach_target(candidate: &mut Candidate, target: TargetHandle) {
        candidate.target = Some(target);
    }

    /// Phase 3/4 — revalidate everything and switch the current record in
    /// ONE step. Any failure keeps the old record and target untouched.
    pub fn commit(
        &mut self,
        candidate: Candidate,
        session: &SourceQuerySession,
        program: &Program,
        cancel: Option<&AtomicBool>,
    ) -> Result<&CurrentRecord, CommitError> {
        if cancel.map(|c| c.load(Ordering::Relaxed)).unwrap_or(false) {
            return Err(CommitError::Cancelled);
        }
        if candidate.base != self.revision {
            return Err(CommitError::LateResult {
                current: self.revision,
                candidate: candidate.base,
            });
        }
        // The complete key must still describe the CURRENT source.
        let current_key = session
            .body_key(program, &candidate.name)
            .map_err(|_| CommitError::KeyChanged {
                expected: candidate.key.canonical(),
                current: "<unknown>".into(),
            })?;
        if current_key.canonical() != candidate.key.canonical() {
            return Err(CommitError::KeyChanged {
                expected: candidate.key.canonical(),
                current: current_key.canonical(),
            });
        }
        // Every positive dependency must still hold.
        let catalog = session.catalog(program).map_err(|e| CommitError::Sink(e.to_string()))?;
        for (dep, version) in &candidate.deps {
            match catalog.version(dep) {
                Some(v) if v == version => {}
                _ => {
                    return Err(CommitError::DepChanged {
                        name: dep.clone(),
                    })
                }
            }
        }
        let artifact = self
            .sink
            .put(&candidate.key, &candidate.facts)
            .map_err(CommitError::Sink)?;
        self.revision += 1;
        let record = CurrentRecord {
            revision: self.revision,
            artifact,
            key: candidate.key.clone(),
            facts: candidate.facts.clone(),
            target: candidate.target,
        };
        self.current = Some(record);
        Ok(self.current.as_ref().expect("just installed"))
    }

    /// A read-only view: never installs, never requires a target.
    pub fn view(
        &self,
        session: &mut SourceQuerySession,
        program: &Program,
        name: &str,
    ) -> Result<QueryView, QueryError> {
        let key = session.body_key(program, name)?;
        let out = session.infer_body(program, name)?;
        Ok(QueryView {
            key,
            revision: self.revision,
            facts: out.facts,
            hit: out.hit,
        })
    }

    /// Single-flight: one computation per complete key, shared by every
    /// joiner. The expected payload guards against the same key producing
    /// different facts.
    pub fn join(
        &mut self,
        key: &QueryKey,
        expected: &InferBodyFacts,
        cancel: Option<&AtomicBool>,
        compute: impl FnOnce() -> Result<InferBodyFacts, String>,
    ) -> Result<(InferBodyFacts, bool), FlightError> {
        if cancel.map(|c| c.load(Ordering::Relaxed)).unwrap_or(false) {
            // This waiter leaves; a flight already running is untouched.
            return Err(FlightError::Cancelled);
        }
        let id = key.canonical();
        if let Some(flight) = self.flights.get(&id) {
            if &flight.facts != expected {
                return Err(FlightError::PayloadMismatch { key: id });
            }
            return Ok((flight.facts.clone(), true));
        }
        let facts = compute().map_err(FlightError::Failed)?;
        if &facts != expected {
            return Err(FlightError::PayloadMismatch { key: id });
        }
        self.flights.insert(id, Flight { facts: facts.clone() });
        Ok((facts, false))
    }
}

