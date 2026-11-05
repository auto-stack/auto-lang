//! Fact → resolved-program application (PLAN-934 T-02, "事实驱动的
//! Resolved/Emitter 入口").
//!
//! Core keeps binding (names, scopes, signatures, records) and the HIR
//! emitter; this module installs the QUERY-supplied types onto the bound
//! `ResolvedProgram` and admits nothing that does not match:
//!
//! * the fact page must belong to the CURRENT body version,
//! * every fact node must exist in the resolved tree exactly once and be
//!   an expression node of the matching kind,
//! * every expression and `let`/`var` declaration of the body must be
//!   covered (a partial page is rejected, never "filled in" from the
//!   resolver's own synthesis),
//! * the function signature page must match the resolved parameter
//!   names/arity and result before any body fact is installed.
//!
//! A rejection here happens BEFORE the HIR is emitted, so a bad fact can
//! never reach a consumer as a silently mistyped program.

use crate::facts::{FactsError, FnSigFacts, InferBodyFacts, PortableTy};
use crate::session::{QueryError, SourceQuerySession};
use crate::walk::{walk_body, NodeId, WalkError};
use ace_core::source::ast::{Block, Item, Program};
use ace_core::source::resolve::{RExprKind, RStmt, ResolvedExpr, ResolvedFn, ResolvedProgram, VT};
use std::collections::{BTreeMap, BTreeSet};

/// What one body's application installed (producer-side counts).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ApplyReport {
    pub exprs: usize,
    pub bindings: usize,
}

/// Why a fact page cannot be installed into the resolved program.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ApplyError {
    Walk(WalkError),
    Facts(FactsError),
    /// A resolved expression has no structural node of the current body.
    SpanNodeMissing {
        start: u32,
        end: u32,
    },
    /// Two structural nodes share one source span (ambiguous mapping).
    DuplicateSpan {
        start: u32,
        end: u32,
    },
    /// A resolved expression maps to a non-expression node.
    NotAnExpression {
        id: u32,
        kind: &'static str,
    },
    /// A fact node was never consumed by the resolved tree (stale/foreign
    /// page: the fact describes a body core did not bind).
    UnusedFact {
        id: u32,
    },
    /// The body has a declaration node with no binding fact.
    MissingBinding {
        id: u32,
    },
    /// A resolved construct outside the finite application surface.
    UnsupportedExpr {
        what: &'static str,
    },
    /// The signature page disagrees with the bound declaration.
    SigArity {
        facts: usize,
        resolved: usize,
    },
    SigParamName {
        index: usize,
        facts: String,
        resolved: String,
    },
    /// Non-function declarations are outside this bounded profile.
    Profile {
        what: &'static str,
    },
}

impl std::fmt::Display for ApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplyError::Walk(e) => write!(f, "{e}"),
            ApplyError::Facts(e) => write!(f, "{e}"),
            ApplyError::SpanNodeMissing { start, end } => write!(
                f,
                "resolved expression at {start}..{end} has no structural node in the current body"
            ),
            ApplyError::DuplicateSpan { start, end } => {
                write!(f, "two structural nodes share span {start}..{end}")
            }
            ApplyError::NotAnExpression { id, kind } => {
                write!(f, "node {id} is `{kind}`, not an expression")
            }
            ApplyError::UnusedFact { id } => write!(
                f,
                "fact node {id} was never consumed: the page does not describe this resolved body"
            ),
            ApplyError::MissingBinding { id } => {
                write!(f, "declaration node {id} has no binding fact")
            }
            ApplyError::UnsupportedExpr { what } => {
                write!(f, "resolved expression `{what}` is outside the finite application surface")
            }
            ApplyError::SigArity { facts, resolved } => write!(
                f,
                "signature fact has {facts} parameter(s), the resolved declaration has {resolved}"
            ),
            ApplyError::SigParamName {
                index,
                facts,
                resolved,
            } => write!(
                f,
                "signature fact parameter {index} is `{facts}`, the resolved declaration has `{resolved}`"
            ),
            ApplyError::Profile { what } => {
                write!(f, "`{what}` is outside the finite ordinary-source profile")
            }
        }
    }
}

/// A failure of the whole fact-driven entry: either a query did not
/// produce facts, or the produced facts could not be installed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FactsApplyError {
    Query(QueryError),
    Apply(ApplyError),
}

impl From<ApplyError> for FactsApplyError {
    fn from(e: ApplyError) -> Self {
        FactsApplyError::Apply(e)
    }
}

impl std::fmt::Display for FactsApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FactsApplyError::Query(e) => write!(f, "{e}"),
            FactsApplyError::Apply(e) => write!(f, "{e}"),
        }
    }
}

/// Per-function report of one fact-driven preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AppliedFn {
    pub name: String,
    pub sig_hit: bool,
    pub body_hit: bool,
    pub body_version: String,
    pub report: ApplyReport,
}

/// Install the signature page on a bound function: parameter names/arity
/// and the result type must agree, then the types become authoritative.
pub fn apply_fn_sig(resolved: &mut ResolvedFn, sig: &FnSigFacts) -> Result<(), ApplyError> {
    sig.check_envelope().map_err(ApplyError::Facts)?;
    if sig.params.len() != resolved.params.len() {
        return Err(ApplyError::SigArity {
            facts: sig.params.len(),
            resolved: resolved.params.len(),
        });
    }
    for (i, (name, ty)) in sig.params.iter().enumerate() {
        if resolved.param_names.get(i) != Some(name) {
            return Err(ApplyError::SigParamName {
                index: i,
                facts: name.clone(),
                resolved: resolved
                    .param_names
                    .get(i)
                    .cloned()
                    .unwrap_or_else(|| "<missing>".into()),
            });
        }
        resolved.params[i] = vt_of(*ty);
        if let Some(l) = resolved
            .locals
            .iter_mut()
            .find(|l| l.is_param && l.param_index == Some(i as u32))
        {
            l.vt = vt_of(*ty);
        }
    }
    resolved.result = vt_of(sig.ret);
    Ok(())
}

/// Install one body's fact page onto the bound function.
///
/// The AST body is needed only for its STRUCTURAL walk: spans come from
/// the resolved nodes, ids and versions from the walk of the same body.
pub fn apply_body_facts(
    ast_body: &Block,
    resolved: &mut ResolvedFn,
    facts: &InferBodyFacts,
) -> Result<ApplyReport, ApplyError> {
    let walk = walk_body(ast_body).map_err(ApplyError::Walk)?;
    facts.validate_against(&walk).map_err(ApplyError::Facts)?;

    let mut map: BTreeMap<(u32, u32), (NodeId, &'static str)> = BTreeMap::new();
    for n in &walk.nodes {
        let key = (n.span.start as u32, n.span.end as u32);
        if map.insert(key, (n.id, n.kind)).is_some() {
            return Err(ApplyError::DuplicateSpan {
                start: key.0,
                end: key.1,
            });
        }
    }

    let mut report = ApplyReport::default();
    let mut consumed: BTreeSet<u32> = BTreeSet::new();

    // Statement tree first: bindings land on the local declarations.
    // Explicit `&mut` stack (no recursion: legal bodies nest deeply).
    let mut stmts: Vec<&mut RStmt> = resolved.body.stmts.iter_mut().collect();
    while let Some(stmt) = stmts.pop() {
        match stmt {
            RStmt::Let { local, value, span } => {
                let id = node_at(*span, &map)?;
                let Some(b) = facts.bindings.iter().find(|b| b.node == id) else {
                    return Err(ApplyError::MissingBinding { id: id.0 });
                };
                resolved.locals[local.0].vt = vt_of(b.ty);
                report.bindings += 1;
                report.exprs += visit_expr(value, &map, facts, &mut consumed)?;
            }
            RStmt::Assign { value, .. } | RStmt::Return { value, .. } => {
                report.exprs += visit_expr(value, &map, facts, &mut consumed)?;
            }
            RStmt::ExprStmt { value, .. } => {
                report.exprs += visit_expr(value, &map, facts, &mut consumed)?;
            }
            RStmt::If {
                cond, then, els, ..
            } => {
                report.exprs += visit_expr(cond, &map, facts, &mut consumed)?;
                if let Some(e) = els {
                    stmts.extend(e.stmts.iter_mut());
                }
                stmts.extend(then.stmts.iter_mut());
            }
            RStmt::While { cond, body, .. } => {
                report.exprs += visit_expr(cond, &map, facts, &mut consumed)?;
                stmts.extend(body.stmts.iter_mut());
            }
            RStmt::Loop { body, .. } => stmts.extend(body.stmts.iter_mut()),
            RStmt::Break { .. } | RStmt::Continue { .. } => {}
        }
    }

    // Every fact of the page must have been used: an unconsumed node means
    // the page describes a different resolved tree.
    for f in &facts.nodes {
        if !consumed.contains(&f.id.0) {
            return Err(ApplyError::UnusedFact { id: f.id.0 });
        }
    }
    Ok(report)
}

fn node_at(
    span: ace_core::atom_text::Span,
    map: &BTreeMap<(u32, u32), (NodeId, &'static str)>,
) -> Result<NodeId, ApplyError> {
    let key = (span.start as u32, span.end as u32);
    map.get(&key)
        .map(|(id, _)| *id)
        .ok_or(ApplyError::SpanNodeMissing {
            start: key.0,
            end: key.1,
        })
}

fn visit_expr(
    root: &mut ResolvedExpr,
    map: &BTreeMap<(u32, u32), (NodeId, &'static str)>,
    facts: &InferBodyFacts,
    consumed: &mut BTreeSet<u32>,
) -> Result<usize, ApplyError> {
    let mut stack: Vec<&mut ResolvedExpr> = vec![root];
    let mut n = 0usize;
    while let Some(e) = stack.pop() {
        let id = node_at(e.span, map)?;
        let kind = map
            .get(&(e.span.start as u32, e.span.end as u32))
            .map(|(_, k)| *k)
            .unwrap_or("?");
        if !kind.starts_with("expr.") {
            return Err(ApplyError::NotAnExpression { id: id.0, kind });
        }
        let Some(ty) = facts.ty_of(id) else {
            return Err(ApplyError::Facts(FactsError::IncompleteCoverage {
                missing: 1,
            }));
        };
        e.vt = vt_of(ty);
        consumed.insert(id.0);
        n += 1;

        let name = expr_kind_name(&e.kind);
        match &mut e.kind {
            RExprKind::IntLit(_) | RExprKind::BoolLit(_) | RExprKind::ReadLocal(_) => {}
            RExprKind::Binary { lhs, rhs, .. } => {
                stack.push(rhs.as_mut());
                stack.push(lhs.as_mut());
            }
            RExprKind::Unary { operand, .. } => stack.push(operand.as_mut()),
            RExprKind::CallFn { args, .. } => {
                for a in args.iter_mut() {
                    stack.push(&mut a.value);
                }
            }
            _ => return Err(ApplyError::UnsupportedExpr { what: name }),
        }
    }
    Ok(n)
}

fn expr_kind_name(k: &RExprKind) -> &'static str {
    match k {
        RExprKind::IntLit(_) => "int-literal",
        RExprKind::BoolLit(_) => "bool-literal",
        RExprKind::StrLit(_) => "str-literal",
        RExprKind::CharLit(_) => "char-literal",
        RExprKind::Method { .. } => "text-method",
        RExprKind::Move { .. } => "move",
        RExprKind::TypeWord(_) => "type-word",
        RExprKind::ReadLocal(_) => "read-local",
        RExprKind::Unit => "unit",
        RExprKind::CallFn { .. } => "call",
        RExprKind::CallClosure { .. } => "closure-call",
        RExprKind::CreateClosure { .. } => "create-closure",
        RExprKind::CallIntrinsic { .. } => "intrinsic-call",
        RExprKind::Binary { .. } => "binary",
        RExprKind::Unary { .. } => "unary",
        RExprKind::CtorPos { .. } => "positional-construction",
        RExprKind::CtorNamed { .. } => "named-construction",
        RExprKind::FieldGet { .. } => "field-read",
        RExprKind::MethodCall { .. } => "method-call",
        _ => "other",
    }
}

fn vt_of(t: PortableTy) -> VT {
    match t {
        PortableTy::I32 => VT::I32,
        PortableTy::Bool => VT::Bool,
        PortableTy::Unit => VT::Unit,
    }
}

/// The fact-driven entry a consumer host calls after core binding: query
/// the signature and body facts of every free function, admit them, and
/// install them on the resolved program. No typecheck and no second
/// synthesis runs here — the emitter afterwards reads these types.
pub fn apply_facts_to_resolved(
    program: &Program,
    resolved: &mut ResolvedProgram,
    session: &mut SourceQuerySession,
) -> Result<Vec<AppliedFn>, FactsApplyError> {
    let names: Vec<String> = program
        .items
        .iter()
        .map(|item| match item {
            Item::Fn(f) => Ok(f.name.name.clone()),
            Item::Type(_) => Err(ApplyError::Profile {
                what: "record declaration",
            }),
            Item::Enum(_) => Err(ApplyError::Profile {
                what: "enum declaration",
            }),
        })
        .collect::<Result<_, _>>()?;

    let mut out = Vec::with_capacity(names.len());
    for name in names {
        let ast_body = program
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if f.name.name == name => Some(&f.body),
                _ => None,
            })
            .expect("name came from the same program");
        let Some(rfn) = resolved
            .fns
            .iter_mut()
            .find(|f| f.name == name && f.owner_rec.is_none())
        else {
            return Err(FactsApplyError::Apply(ApplyError::Profile {
                what: "unbound free function",
            }));
        };

        let sig = session
            .fn_sig(program, &name)
            .map_err(FactsApplyError::Query)?;
        apply_fn_sig(rfn, &sig.facts)?;

        let body = session
            .infer_body(program, &name)
            .map_err(FactsApplyError::Query)?;
        // Full admission: shape, coverage and a diagnostic-free page.
        let walk = walk_body(ast_body).map_err(|e| FactsApplyError::Apply(ApplyError::Walk(e)))?;
        body.facts
            .admit(&walk)
            .map_err(|e| FactsApplyError::Apply(ApplyError::Facts(e)))?;
        let report = apply_body_facts(ast_body, rfn, &body.facts)?;

        out.push(AppliedFn {
            name,
            sig_hit: sig.hit,
            body_hit: body.hit,
            body_version: body.facts.body_version.as_str().to_string(),
            report,
        });
    }
    Ok(out)
}

// -- PLAN-934 T-04: fact-supplied bind (legacy synthesis cut) ----------

use ace_core::atom_text::{Diagnostic, Span as CoreSpan};
use ace_core::source::resolve::{resolve_with_supply, TypeSupply};
use ace_core::source::SourceProfile;

/// The span-keyed projection of one module's ADMITTED fact pages.
///
/// Spans are only the current source map; every entry here came from a
/// page that was admitted against the body's structural walk first
/// (version, node ids, kinds, coverage). Core reads this through
/// [`TypeSupply`] and treats the returned type as authoritative.
#[derive(Clone, Debug, Default)]
pub struct BodyTypeSupply {
    exprs: BTreeMap<(u32, u32), Vec<VT>>,
    bindings: BTreeMap<(u32, u32), VT>,
    admitted: usize,
}

impl BodyTypeSupply {
    pub fn new() -> Self {
        BodyTypeSupply::default()
    }

    /// Number of admitted bodies whose types this supply carries.
    pub fn admitted_bodies(&self) -> usize {
        self.admitted
    }

    pub fn expr_count(&self) -> usize {
        self.exprs.values().map(|v| v.len()).sum()
    }

    pub fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    /// Admit one body's page and add its types. Any shape/coverage/
    /// diagnostic problem is the caller's rejection, not a partial add.
    ///
    /// A span may carry more than one fact: the core parser gives a unary
    /// expression the span of its operand (`!1`), so the parent and child
    /// facts share it. They are kept as an occurrence list; core installs
    /// the one matching the node's rule and reports a mismatch when none
    /// does (PLAN-934 T-08).
    pub fn add_body(&mut self, ast_body: &Block, facts: &InferBodyFacts) -> Result<(), ApplyError> {
        let walk = walk_body(ast_body).map_err(ApplyError::Walk)?;
        facts.admit(&walk).map_err(ApplyError::Facts)?;
        for n in walk.nodes.iter().filter(|n| n.is_expr()) {
            let ty =
                facts
                    .ty_of(n.id)
                    .ok_or(ApplyError::Facts(FactsError::IncompleteCoverage {
                        missing: 1,
                    }))?;
            self.exprs
                .entry((n.span.start as u32, n.span.end as u32))
                .or_default()
                .push(vt_of(ty));
        }
        for (decl, b) in walk.lets.iter().zip(facts.bindings.iter()) {
            if decl.node != b.node {
                return Err(ApplyError::Facts(FactsError::BindingAlignment));
            }
            let span = walk
                .node(b.node)
                .map(|n| n.span)
                .unwrap_or(CoreSpan { start: 0, end: 0 });
            self.bindings
                .insert((span.start as u32, span.end as u32), vt_of(b.ty));
        }
        self.admitted += 1;
        Ok(())
    }
}

impl TypeSupply for BodyTypeSupply {
    fn expr_ty(&self, span: CoreSpan) -> Option<VT> {
        self.exprs
            .get(&(span.start as u32, span.end as u32))
            .and_then(|v| v.first().copied())
    }

    fn expr_tys(&self, span: CoreSpan) -> Vec<VT> {
        self.exprs
            .get(&(span.start as u32, span.end as u32))
            .cloned()
            .unwrap_or_default()
    }

    fn binding_ty(&self, span: CoreSpan) -> Option<VT> {
        self.bindings
            .get(&(span.start as u32, span.end as u32))
            .copied()
    }
}

/// Why a fact-driven bind failed. Resolve diagnostics are the CORE's own
/// bind-stage verdict (bad fact / missing fact / real source error).
#[derive(Clone, Debug)]
pub enum BindFactsError {
    Query(QueryError),
    Apply(ApplyError),
    Resolve(Vec<Diagnostic>),
    Profile(&'static str),
}

impl std::fmt::Display for BindFactsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BindFactsError::Query(e) => write!(f, "{e}"),
            BindFactsError::Apply(e) => write!(f, "{e}"),
            BindFactsError::Resolve(ds) => {
                let first = ds
                    .first()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .unwrap_or_else(|| "bind failed".into());
                write!(f, "{first} ({} diagnostic(s))", ds.len())
            }
            BindFactsError::Profile(w) => write!(f, "`{w}` is outside the finite profile"),
        }
    }
}

/// PLAN-934 T-04: the fact-driven bind entry.
///
/// Every free function's signature and body page is queried, admitted and
/// turned into a [`BodyTypeSupply`]; `resolve_with_supply` then BINDS with
/// those types as the authority (the resolver's rules are checks only).
/// Signature pages are installed afterwards so parameters/result come from
/// the same fact layer.
pub fn bind_with_facts(
    program: &Program,
    capabilities: &[String],
    package: &str,
    module: &str,
    session: &mut SourceQuerySession,
    profile: SourceProfile,
) -> Result<(ResolvedProgram, Vec<AppliedFn>, BodyTypeSupply), BindFactsError> {
    if session.package() != package || session.module() != module {
        return Err(BindFactsError::Profile("session/module identity mismatch"));
    }
    let mut names: Vec<String> = Vec::new();
    for item in &program.items {
        match item {
            Item::Fn(f) => names.push(f.name.name.clone()),
            Item::Type(_) => return Err(BindFactsError::Profile("record declaration")),
            Item::Enum(_) => return Err(BindFactsError::Profile("enum declaration")),
        }
    }
    if names.is_empty() {
        return Ok((
            ResolvedProgram {
                fns: Vec::new(),
                records: Vec::new(),
                lists: Vec::new(),
                variants: Vec::new(),
                function_types: Vec::new(),
            },
            Vec::new(),
            BodyTypeSupply::new(),
        ));
    }

    let mut supply = BodyTypeSupply::new();
    let mut sigs: Vec<(String, FnSigFacts, bool)> = Vec::new();
    let mut applied: Vec<AppliedFn> = Vec::new();
    for name in &names {
        let ast_body = program
            .items
            .iter()
            .find_map(|item| match item {
                Item::Fn(f) if &f.name.name == name => Some(&f.body),
                _ => None,
            })
            .expect("name came from the same program");
        let sig = session
            .fn_sig(program, name)
            .map_err(BindFactsError::Query)?;
        sig.facts
            .check_envelope()
            .map_err(|e| BindFactsError::Apply(ApplyError::Facts(e)))?;
        let body = session
            .infer_body(program, name)
            .map_err(BindFactsError::Query)?;
        supply
            .add_body(ast_body, &body.facts)
            .map_err(BindFactsError::Apply)?;
        applied.push(AppliedFn {
            name: name.clone(),
            sig_hit: sig.hit,
            body_hit: body.hit,
            body_version: body.facts.body_version.as_str().to_string(),
            report: ApplyReport {
                exprs: body.facts.nodes.len(),
                bindings: body.facts.bindings.len(),
            },
        });
        sigs.push((name.clone(), sig.facts, sig.hit));
    }

    let mut resolved = resolve_with_supply(program, capabilities, profile, &supply)
        .map_err(BindFactsError::Resolve)?;

    // The signature pages are authoritative too: parameters and the result
    // type are installed (and checked) from the same fact layer.
    for (name, sig, _) in &sigs {
        if let Some(rfn) = resolved
            .fns
            .iter_mut()
            .find(|f| &f.name == name && f.owner_rec.is_none())
        {
            apply_fn_sig(rfn, sig).map_err(BindFactsError::Apply)?;
        }
    }
    Ok((resolved, applied, supply))
}
