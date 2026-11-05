//! Fact production: one ordinary body → the REUSED 796 solver → an
//! immutable SQ-A fact page (PLAN-934 §5.1/§5.5).
//!
//! The input is a BODY-SCOPED view of the module: the target function is
//! projected in full, every other function is projected as a signature
//! stub with an empty body. That is exactly the "one body, lazy input"
//! rule — a caller's fact production never walks an unrelated body, and
//! the solver can still read the callee signatures the body really calls.
//!
//! The solver is not copied: [`ace_semantics::InferSession`] runs the
//! frozen `ac796-bidir-worklist/1` algorithm over the neutral input the
//! 796 adapter projects from the core public AST. This crate only owns
//! the per-body input assembly, the structural-id mapping and the fact
//! envelope.

use crate::facts::{
    BindingFact, InferBodyFacts, NegativeDep, NodeFact, PortableDiag, PortableTy,
};
use crate::ident::{
    owner_logical, signature_version, SQ_ALGO, SQ_PRODUCER, SQ_PROFILE, SQ_SCHEMA,
};
use crate::walk::{walk_body, WalkError};
use ace_core::source::ast::{FnItem, Item, Program, TypeAnn, TypeRef, TypeWord};
use ace_semantics::{InferOutcome, InferSession};
use std::time::Duration;

/// Per-query limits, from the plan §5.4 frozen budget: the 796 solver
/// envelope (5s wall / 1M steps) is reused unchanged; the node ceiling is
/// this layer's own controlled rejection, not a language limit.
#[derive(Clone, Copy, Debug)]
pub struct ProduceLimits {
    pub max_steps: u64,
    pub wall: Duration,
    pub max_body_nodes: usize,
}

impl Default for ProduceLimits {
    fn default() -> Self {
        ProduceLimits {
            max_steps: 1_000_000,
            wall: Duration::from_secs(5),
            max_body_nodes: 4096,
        }
    }
}

/// Why a fact page could not be produced. Every variant is a controlled
/// state: none of them is a type verdict and none may be cached as one.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ProduceError {
    /// The queried function does not exist in this module.
    UnknownFn { name: String },
    /// Declarations outside the finite ordinary-source surface.
    Profile { what: &'static str },
    /// The body walk rejected the construct.
    Walk(WalkError),
    /// The 796 projection rejected the input (typed as infrastructure).
    Projection(String),
    /// Solver resource state (never a program verdict).
    Resource(String),
    Cancelled,
    /// One source span carries two structural expression nodes.
    DuplicateSpan { start: u32, end: u32 },
    /// The solver typed a span the structural walk does not have.
    UnmappedSpan { count: usize },
    /// Binding facts do not align with the body's declaration order.
    BindingAlignment { expected: usize, found: usize },
    BindingName {
        index: usize,
        expected: String,
        found: String,
    },
    /// A type outside the finite portable set (never widened to `Any`).
    NonPortableType { what: String },
}

impl std::fmt::Display for ProduceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProduceError::UnknownFn { name } => write!(f, "`{name}` is not a function of this module"),
            ProduceError::Profile { what } => {
                write!(f, "`{what}` is outside the finite ordinary-source query profile")
            }
            ProduceError::Walk(e) => write!(f, "{e}"),
            ProduceError::Projection(m) => write!(f, "semantic projection failed: {m}"),
            ProduceError::Resource(m) => write!(f, "solver resource state: {m}"),
            ProduceError::Cancelled => write!(f, "the semantic query was cancelled"),
            ProduceError::DuplicateSpan { start, end } => write!(
                f,
                "two expression nodes share span {start}..{end}; structural fact mapping is ambiguous"
            ),
            ProduceError::UnmappedSpan { count } => write!(
                f,
                "{count} solver span(s) have no structural node in the current body"
            ),
            ProduceError::BindingAlignment { expected, found } => write!(
                f,
                "the body has {expected} declaration(s) but the solver reported {found} binding(s)"
            ),
            ProduceError::BindingName {
                index,
                expected,
                found,
            } => write!(
                f,
                "binding {index} is `{found}` but declaration order expects `{expected}`"
            ),
            ProduceError::NonPortableType { what } => {
                write!(f, "type `{what}` is outside the finite portable set")
            }
        }
    }
}

/// Produce ONE function's signature facts from the declaration alone.
/// This never walks or solves a body (the read set of a signature is the
/// declaration axis only).
pub fn produce_fn_sig(
    program: &Program,
    package: &str,
    module: &str,
    name: &str,
) -> Result<crate::facts::FnSigFacts, ProduceError> {
    let f = find_free_fn(program, name)?;
    let mut params = Vec::with_capacity(f.params.len());
    for p in &f.params {
        params.push((p.name.name.clone(), portable_of_ann(&p.ty)?));
    }
    Ok(crate::facts::FnSigFacts {
        logical: owner_logical(package, module, name),
        name: name.to_string(),
        version: signature_version(f),
        params,
        ret: portable_of_ann(&f.ret)?,
        profile: SQ_PROFILE,
        schema: SQ_SCHEMA,
        producer: SQ_PRODUCER,
    })
}

/// Produce ONE body's fact page through the reused solver.
pub fn produce_infer_body(
    program: &Program,
    package: &str,
    module: &str,
    name: &str,
    limits: ProduceLimits,
    cancel: Option<std::sync::Arc<std::sync::atomic::AtomicBool>>,
) -> Result<InferBodyFacts, ProduceError> {
    if limits.max_body_nodes == 0 {
        return Err(ProduceError::Profile { what: "empty node budget" });
    }
    let target = find_free_fn(program, name)?;
    let walk = walk_body(&target.body).map_err(ProduceError::Walk)?;
    if walk.expr_nodes > limits.max_body_nodes {
        return Err(ProduceError::Profile {
            what: "body node ceiling exceeded",
        });
    }

    // Body-scoped LAZY input (PLAN-934 T-03): the shared adapter projects
    // ONLY the target function's body; every other function is a
    // signature stub and no AST is cloned or walked.
    let input = ace_semantics::adapter::project_body_scoped(program, name, package, module)
        .map_err(ProduceError::Projection)?;

    let mut session = InferSession::new(&input).with_budget(limits.max_steps, limits.wall);
    if let Some(flag) = cancel {
        session = session.with_cancel(flag);
    }
    let outcome = session.infer_function(name);
    let summary = match outcome {
        InferOutcome::Ok(s) | InferOutcome::Diags(s) => s,
        InferOutcome::ResourceExhausted(e) => return Err(ProduceError::Resource(e.to_string())),
        InferOutcome::Cancelled => return Err(ProduceError::Cancelled),
    };

    let spans = walk.span_index().map_err(|d| ProduceError::DuplicateSpan {
        start: d.start,
        end: d.end,
    })?;

    // Types by structural id; a solver span outside the walk is a
    // controlled infrastructure rejection rather than a silent drop.
    let mut typed: std::collections::BTreeMap<u32, PortableTy> = std::collections::BTreeMap::new();
    let mut unmapped = 0usize;
    for (start, end, td) in &summary.facts.expr_types {
        let Some(id) = spans.get(&(*start, *end)) else {
            unmapped += 1;
            continue;
        };
        let ty = PortableTy::from_type_desc(td).ok_or_else(|| ProduceError::NonPortableType {
            what: format!("{td:?}"),
        })?;
        typed.insert(id.0, ty);
    }
    if unmapped > 0 {
        return Err(ProduceError::UnmappedSpan { count: unmapped });
    }

    let mut nodes = Vec::with_capacity(typed.len());
    let mut missing = Vec::new();
    for info in walk.nodes.iter().filter(|n| n.is_expr()) {
        match typed.get(&info.id.0) {
            Some(ty) => nodes.push(NodeFact {
                id: info.id,
                kind: info.kind,
                ty: *ty,
            }),
            None => missing.push(info.id),
        }
    }

    // The solver reports the parameter bindings first (declaration order),
    // then one binding per `let`/`var` in body order; only the latter are
    // body declarations with a structural node.
    let param_count = target.params.len();
    if summary.facts.bindings.len() < param_count {
        return Err(ProduceError::BindingAlignment {
            expected: param_count,
            found: summary.facts.bindings.len(),
        });
    }
    for (i, p) in target.params.iter().enumerate() {
        if summary.facts.bindings[i].0 != p.name.name {
            return Err(ProduceError::BindingName {
                index: i,
                expected: p.name.name.clone(),
                found: summary.facts.bindings[i].0.clone(),
            });
        }
    }
    let bindings = align_bindings(&walk.lets, &summary.facts.bindings[param_count..])?;

    let diags = summary
        .facts
        .diagnostics
        .iter()
        .map(|d| PortableDiag {
            code: d.code.to_string(),
            node: spans.get(&(d.start, d.end)).copied(),
            message: d.message.clone(),
        })
        .collect();

    Ok(InferBodyFacts {
        name: name.to_string(),
        logical: owner_logical(package, module, name),
        body_version: walk.version.clone(),
        profile: SQ_PROFILE,
        algo: SQ_ALGO,
        schema: SQ_SCHEMA,
        producer: SQ_PRODUCER,
        revision: 0,
        nodes,
        bindings,
        diags,
        missing,
        sig_reads: summary.reads.sigs.clone(),
        negatives: summary
            .reads
            .negatives
            .iter()
            .map(|(domain, n, _)| NegativeDep {
                domain: (*domain).to_string(),
                name: n.clone(),
            })
            .collect(),
        steps: summary.steps,
    })
}

fn align_bindings(
    lets: &[crate::walk::LetDecl],
    solved: &[(String, ace_semantics_contract::TypeDesc)],
) -> Result<Vec<BindingFact>, ProduceError> {
    if lets.len() != solved.len() {
        return Err(ProduceError::BindingAlignment {
            expected: lets.len(),
            found: solved.len(),
        });
    }
    let mut out = Vec::with_capacity(lets.len());
    for (i, (decl, (name, td))) in lets.iter().zip(solved.iter()).enumerate() {
        if &decl.name != name {
            return Err(ProduceError::BindingName {
                index: i,
                expected: decl.name.clone(),
                found: name.clone(),
            });
        }
        let ty = PortableTy::from_type_desc(td).ok_or_else(|| ProduceError::NonPortableType {
            what: format!("{td:?}"),
        })?;
        out.push(BindingFact {
            node: decl.node,
            name: decl.name.clone(),
            ty,
        });
    }
    Ok(out)
}

fn find_free_fn<'a>(program: &'a Program, name: &str) -> Result<&'a FnItem, ProduceError> {
    let mut hit: Option<&FnItem> = None;
    for item in &program.items {
        match item {
            Item::Fn(f) => {
                if f.name.name == name {
                    hit = Some(f);
                }
            }
            Item::Type(_) => {
                return Err(ProduceError::Profile {
                    what: "record declaration",
                })
            }
            Item::Enum(_) => {
                return Err(ProduceError::Profile {
                    what: "enum declaration",
                })
            }
        }
    }
    hit.ok_or_else(|| ProduceError::UnknownFn {
        name: name.to_string(),
    })
}

/// Portable type of an annotation; `None`-capable types are a controlled
/// rejection (never an `Any` widening).
fn portable_of_ann(t: &TypeAnn) -> Result<PortableTy, ProduceError> {
    match &t.kind {
        TypeRef::Builtin(TypeWord::Int) | TypeRef::Builtin(TypeWord::I32) => Ok(PortableTy::I32),
        TypeRef::Builtin(TypeWord::Bool) => Ok(PortableTy::Bool),
        TypeRef::Builtin(TypeWord::Void) => Ok(PortableTy::Unit),
        other => Err(ProduceError::NonPortableType {
            what: format!("{other:?}"),
        }),
    }
}
