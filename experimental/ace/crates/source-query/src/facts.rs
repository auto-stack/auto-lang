//! The SQ-A fact payloads (PLAN-934 §5.5 "完整事实身份" / §5.2 keys).
//!
//! A fact page is an IMMUTABLE, portable description of one body's types.
//! It is addressed by (logical owner, body version) and its node entries
//! are indexed by the body's [`NodeId`] — never by variable name, absolute
//! span or intern slot:
//!
//! * names are payload only (a same-named shadowing local is a different
//!   node id),
//! * spans are absent from the payload entirely (whitespace/prefix drift
//!   keeps every semantic value; the current source-map is applied by the
//!   consumer from its own walk),
//! * types are the portable descriptions of the semantic contract, so no
//!   process-local `VT`/`LocalRef`/`ResolvedExpr` is ever serialised.
//!
//! Admission ([`InferBodyFacts::admit`]) is all-or-nothing: a page whose
//! body version differs, whose node ids/kinds do not match the current
//! walk, that repeats a node, that misses an expression node or that
//! carries diagnostics is REJECTED. A consumer must never fall back to
//! "not found, so infer it" — that fallback is exactly the legacy
//! synthesis this plan removes.

use crate::ident::{SQ_ALGO, SQ_PRODUCER, SQ_PROFILE, SQ_SCHEMA};
use crate::walk::{BodyWalk, NodeId};
use ace_semantics_contract::{ContentVersion, LogicalId, TypeDesc};

/// The portable value types of the finite ordinary-source surface.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PortableTy {
    /// Result-only `void` (never a local/params/expression value type).
    Unit,
    I32,
    Bool,
}

impl PortableTy {
    pub fn as_str(self) -> &'static str {
        match self {
            PortableTy::Unit => "void",
            PortableTy::I32 => "i32",
            PortableTy::Bool => "bool",
        }
    }

    /// Convert a contract type description. `None` means the type is
    /// outside the finite profile — a controlled rejection, never an
    /// implicit widening.
    pub fn from_type_desc(t: &TypeDesc) -> Option<PortableTy> {
        match t {
            TypeDesc::Unit => Some(PortableTy::Unit),
            TypeDesc::I32 => Some(PortableTy::I32),
            TypeDesc::Bool => Some(PortableTy::Bool),
            _ => None,
        }
    }
}

/// One expression node's type fact.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeFact {
    pub id: NodeId,
    pub kind: &'static str,
    pub ty: PortableTy,
}

/// One `let`/`var` binding's solved type, keyed by the declaration node.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BindingFact {
    pub node: NodeId,
    pub name: String,
    pub ty: PortableTy,
}

/// A negative dependency: "this name did NOT exist in this domain when
/// the body was solved". It invalidates when the name becomes defined
/// (a syntactic candidate-set change, not a semantic dependency).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct NegativeDep {
    pub domain: String,
    pub name: String,
}

/// A diagnostic carried as data. Positions are NOT persisted: the
/// consumer re-anchors the referenced node in the current source map.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PortableDiag {
    pub code: String,
    pub node: Option<NodeId>,
    pub message: String,
}

/// One immutable infer-body fact page.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InferBodyFacts {
    pub logical: LogicalId,
    /// The body version these ids are valid for.
    pub body_version: ContentVersion,
    pub profile: &'static str,
    pub algo: &'static str,
    pub schema: &'static str,
    pub producer: &'static str,
    /// Request revision this view is bound to (set by the session at
    /// query time; a late result for an older revision is never
    /// installed as current).
    pub revision: u64,
    pub nodes: Vec<NodeFact>,
    pub bindings: Vec<BindingFact>,
    pub diags: Vec<PortableDiag>,
    /// Expression nodes the solver produced no type for.
    pub missing: Vec<NodeId>,
    /// Callee signatures actually read (positive dependencies).
    pub sig_reads: Vec<String>,
    /// Names that did not resolve (negative dependencies).
    pub negatives: Vec<NegativeDep>,
    /// Solver steps consumed (real metering; never reconstructed).
    pub steps: u64,
}

impl InferBodyFacts {
    pub fn ty_of(&self, id: NodeId) -> Option<PortableTy> {
        self.nodes.iter().find(|n| n.id == id).map(|n| n.ty)
    }

    /// Every expression node of the body was typed.
    pub fn is_complete(&self) -> bool {
        self.missing.is_empty()
    }

    /// No definitive type diagnostics.
    pub fn is_clean(&self) -> bool {
        self.diags.is_empty()
    }

    /// Validate identity/shape against the CURRENT walk of the same body.
    pub fn validate_against(&self, walk: &BodyWalk) -> Result<(), FactsError> {
        // Namespace admission first: a page from another producer/schema/
        // algorithm/profile (including the old 796 `ac796/*` surface) must
        // never be reinterpreted, even if its bytes happen to parse.
        if self.producer != SQ_PRODUCER {
            return Err(FactsError::Envelope {
                facts: self.producer,
                expected: SQ_PRODUCER,
            });
        }
        if self.schema != SQ_SCHEMA {
            return Err(FactsError::Envelope {
                facts: self.schema,
                expected: SQ_SCHEMA,
            });
        }
        if self.algo != SQ_ALGO {
            return Err(FactsError::Envelope {
                facts: self.algo,
                expected: SQ_ALGO,
            });
        }
        if self.profile != SQ_PROFILE {
            return Err(FactsError::Envelope {
                facts: self.profile,
                expected: SQ_PROFILE,
            });
        }
        if self.body_version != walk.version {
            return Err(FactsError::BodyVersionMismatch {
                facts: self.body_version.as_str().to_string(),
                body: walk.version.as_str().to_string(),
            });
        }
        let mut seen = std::collections::BTreeSet::new();
        for f in &self.nodes {
            let Some(n) = walk.node(f.id) else {
                return Err(FactsError::UnknownNode { id: f.id.0 });
            };
            if !seen.insert(f.id.0) {
                return Err(FactsError::DuplicateNode { id: f.id.0 });
            }
            if n.kind != f.kind {
                return Err(FactsError::KindMismatch {
                    id: f.id.0,
                    facts: f.kind,
                    body: n.kind,
                });
            }
            if !n.is_expr() {
                return Err(FactsError::KindMismatch {
                    id: f.id.0,
                    facts: f.kind,
                    body: n.kind,
                });
            }
        }
        for b in &self.bindings {
            let Some(n) = walk.node(b.node) else {
                return Err(FactsError::UnknownNode { id: b.node.0 });
            };
            if n.kind != "stmt.let" {
                return Err(FactsError::KindMismatch {
                    id: b.node.0,
                    facts: n.kind,
                    body: "stmt.let",
                });
            }
        }
        if !walk.lets.iter().map(|l| l.node).eq(self.bindings.iter().map(|b| b.node)) {
            return Err(FactsError::BindingAlignment);
        }
        // Coverage is validated against the CURRENT walk, never taken
        // from the producer's own `missing` report: a fact page that lost
        // entries is incomplete, not "absent, so infer it".
        let covered: std::collections::BTreeSet<u32> = self.nodes.iter().map(|n| n.id.0).collect();
        let missing = walk
            .nodes
            .iter()
            .filter(|n| n.is_expr() && !covered.contains(&n.id.0))
            .count();
        if missing > 0 {
            return Err(FactsError::IncompleteCoverage { missing });
        }
        Ok(())
    }

    /// Full admission for a consumer: shape + coverage + no diagnostics.
    pub fn admit(&self, walk: &BodyWalk) -> Result<(), FactsError> {
        self.validate_against(walk)?;
        if !self.is_complete() {
            return Err(FactsError::IncompleteCoverage {
                missing: self.missing.len(),
            });
        }
        if !self.is_clean() {
            return Err(FactsError::Diagnostics {
                count: self.diags.len(),
            });
        }
        Ok(())
    }
}

/// Why a fact page cannot be consumed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FactsError {
    BodyVersionMismatch { facts: String, body: String },
    UnknownNode { id: u32 },
    KindMismatch { id: u32, facts: &'static str, body: &'static str },
    DuplicateNode { id: u32 },
    IncompleteCoverage { missing: usize },
    Diagnostics { count: usize },
    /// Binding facts do not align with the body's `let`/`var` order.
    BindingAlignment,
    /// The page was produced under another profile/schema identity.
    Envelope { facts: &'static str, expected: &'static str },
}

impl std::fmt::Display for FactsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactsError::BodyVersionMismatch { facts, body } => write!(
                f,
                "fact page was produced for body version {facts}, current body is {body}"
            ),
            FactsError::UnknownNode { id } => {
                write!(f, "fact references node {id}, which the current body does not have")
            }
            FactsError::KindMismatch { id, facts, body } => write!(
                f,
                "fact node {id} is `{facts}` but the current body has `{body}` at that id"
            ),
            FactsError::DuplicateNode { id } => write!(f, "node {id} has more than one fact"),
            FactsError::IncompleteCoverage { missing } => {
                write!(f, "{missing} expression node(s) have no type fact")
            }
            FactsError::Diagnostics { count } => {
                write!(f, "fact page carries {count} diagnostic(s)")
            }
            FactsError::BindingAlignment => {
                write!(f, "binding facts do not align with the body's declaration order")
            }
            FactsError::Envelope { facts, expected } => write!(
                f,
                "fact page belongs to envelope `{facts}`, this consumer expects `{expected}`"
            ),
        }
    }
}

/// One function signature fact. Produced from the declaration only — it
/// never reads or solves a body.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FnSigFacts {
    pub logical: LogicalId,
    pub name: String,
    pub version: ContentVersion,
    pub params: Vec<(String, PortableTy)>,
    pub ret: PortableTy,
    pub profile: &'static str,
    pub schema: &'static str,
    pub producer: &'static str,
}

impl FnSigFacts {
    /// Contract identity: the page must carry the SQ-A producer, schema and
    /// profile before a consumer may use it.
    pub fn check_envelope(&self) -> Result<(), FactsError> {
        if self.producer != SQ_PRODUCER {
            return Err(FactsError::Envelope {
                facts: self.producer,
                expected: SQ_PRODUCER,
            });
        }
        if self.schema != SQ_SCHEMA {
            return Err(FactsError::Envelope {
                facts: self.schema,
                expected: SQ_SCHEMA,
            });
        }
        if self.profile != SQ_PROFILE {
            return Err(FactsError::Envelope {
                facts: self.profile,
                expected: SQ_PROFILE,
            });
        }
        if !self.version.as_str().is_empty() {
            // The version must be a real domain hash, not a borrowed span
            // or an empty placeholder.
            if self.version.as_str().len() != 64 {
                return Err(FactsError::BodyVersionMismatch {
                    facts: self.version.as_str().to_string(),
                    body: "<64-hex domain hash>".into(),
                });
            }
        }
        Ok(())
    }
}

/// The page identities a consumer can assert (used by tests and by the
/// T-06 publish gate).
pub fn page_envelope(facts: &InferBodyFacts) -> (&'static str, &'static str, &'static str) {
    (facts.profile, facts.schema, facts.algo)
}

/// Compile-time check that the envelope constants the payload declares
/// are the ones this crate was built with.
pub fn envelope_matches(facts: &InferBodyFacts) -> bool {
    facts.profile == SQ_PROFILE && facts.schema == SQ_SCHEMA && facts.algo == SQ_ALGO
}
