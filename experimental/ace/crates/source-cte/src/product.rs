//! The CT-aware product compile of the source lane (PLAN-936 T-07, §5.3).
//!
//! The single door a PRODUCT consumer calls for a scalar module that may
//! carry `#{ ... }` demands:
//!
//! ```text
//! CT face parse (caps front door) → serve every demand (T-05 planner:
//!   real AV / frozen hits / local meta, one shared root budget)
//! → substitute every hash site with its frozen literal (structural —
//!   no text is formatted or re-parsed)
//! → the FACT lane compiles the substituted program (934 bind_with_facts
//!   → emit → real verify) — the type authority is the query layer, the
//!   HIR is the same artifact the plain lane produces for the served
//!   program
//! → CheckedModule: every consumer end (AV/native/Rust/C) consumes it
//!   through its existing lane; the CT call is GONE from the module (the
//!   owner body carries the constant, re-derived and re-verified).
//! ```
//!
//! The unserved-HashExpr rejection cannot fire here: every site is served
//! (or the root failed) before the fact lane binds. A compile of a
//! hash-carrying program through the PLAIN lanes keeps rejecting at
//! typecheck (`source.cte.unserved`) — the CT lane is the only server.

use crate::schedule::{serve_program, RootBudget, ServedRoot};
use crate::service::CtService;
use crate::store::CtEnvelopeSink;
use ace_core::atom_text::Diagnostic;
use ace_core::source::ast::Program;
use ace_core::source::cst::{
    parse_front_with_caps, FrontBudget, FrontKind, FrontOutcome, FrontRequest,
};
use ace_core::source::{emit_resolved_parts, SourceProfile};
use ace_core::verify::{verify_scalar, CheckedModule};
use ace_source_query::{apply::BodyTypeSupply, SourceQuerySession};

/// The CT product: the checked module of the SERVED program plus the
/// serving facts (per-site frozen values, counters-adjacent provenance).
#[derive(Debug)]
pub struct CtProduct {
    pub checked: CheckedModule,
    /// The fully substituted program (every hash site replaced by its
    /// frozen literal — the structural backfill itself).
    pub program: Program,
    pub served: ServedRoot,
    /// The fact supply of the substituted program's bodies (hosts that
    /// retain a checked environment seed from the same facts).
    pub supply: BodyTypeSupply,
}

/// Why a CT product compile failed.
#[derive(Debug)]
pub enum CtProductError {
    /// Front-door/language diagnostics (parse/bind/emit/verify), reported
    /// unchanged (code/stage/message/span).
    Diagnostics(Vec<Diagnostic>),
    /// A CT demand failed (planner/executor; includes the named chain of
    /// a query cycle and the deadline resource state).
    Plan(crate::schedule::PlanError),
}

impl From<crate::schedule::PlanError> for CtProductError {
    fn from(p: crate::schedule::PlanError) -> Self {
        CtProductError::Plan(p)
    }
}

/// Compile one scalar source that may carry `#{ ... }` demands.
///
/// `capabilities` must include [`CT_CAPABILITY`] for a hash-carrying
/// module (the front door routes it to the scalar-ct face; a module
/// without hashes compiles identically through the same face).
#[allow(clippy::too_many_arguments)]
pub fn compile_scalar_ct<S: CtEnvelopeSink>(
    svc: &mut CtService<S>,
    src: &str,
    module_path: &str,
    capabilities: &[String],
    package: &str,
    module: &str,
) -> Result<CtProduct, CtProductError> {
    // 1) CT-face parse (the caps front door; meta selectors need the meta
    //    capability riding the same list).
    let program = match parse_front_with_caps(
        FrontRequest {
            src,
            profile: SourceProfile::Scalar,
            kind: FrontKind::WholeModule,
            budget: FrontBudget::WORK_OWNED_64MIB,
        },
        capabilities,
    ) {
        FrontOutcome::Ok(ok) => ok.program,
        FrontOutcome::Diag(ds) => return Err(CtProductError::Diagnostics(ds)),
        FrontOutcome::ResourceExhausted { what, limit } => {
            return Err(CtProductError::Diagnostics(vec![Diagnostic::new(
                ace_core::atom_text::Stage::Text,
                "source.resource",
                format!("resource exhausted: {what} (limit {limit})"),
                ace_core::atom_text::Span { start: 0, end: 0 },
            )]))
        }
        FrontOutcome::StaleForeign(m) | FrontOutcome::Infra(m) => {
            return Err(CtProductError::Diagnostics(vec![Diagnostic::new(
                ace_core::atom_text::Stage::Text,
                "source.infra",
                m,
                ace_core::atom_text::Span { start: 0, end: 0 },
            )]))
        }
    };
    // 2) Serve every demand under one shared root budget.
    let served = serve_program(
        svc,
        &program,
        src,
        module_path,
        package,
        module,
        &RootBudget::new(RootBudget::default_total()),
    )?;
    // 3) Structural backfill: every served site -> its frozen literal.
    let mut substituted = program.clone();
    substitute_all(&mut substituted, &served);
    // 4) The fact lane compiles the substituted program (the query layer
    //    is the type authority; HIR + real verification as the plain lane).
    let mut session = SourceQuerySession::new(package, module);
    session.set_capabilities(capabilities.iter().cloned());
    let (resolved, _applied, supply) = ace_source_query::bind_with_facts(
        &substituted,
        capabilities,
        package,
        module,
        &mut session,
        SourceProfile::Scalar,
    )
    .map_err(|e| {
        CtProductError::Diagnostics(vec![Diagnostic::new(
            ace_core::atom_text::Stage::Bind,
            "source.cte.bind",
            format!("{e}"),
            ace_core::atom_text::Span { start: 0, end: 0 },
        )])
    })?;
    let parts = emit_resolved_parts(&resolved, src, module_path, SourceProfile::Scalar)
        .map_err(CtProductError::Diagnostics)?;
    let checked = verify_scalar(parts.bundle).map_err(CtProductError::Diagnostics)?;
    Ok(CtProduct {
        checked,
        program: substituted,
        served,
        supply,
    })
}

/// Replace every served hash site (by span) with its frozen literal —
/// the whole-program form of the planner's per-site substitution.
fn substitute_all(p: &mut Program, served: &ServedRoot) {
    use ace_core::source::ast::{Arg, Expr, ExprKind, Item, Stmt};
    for item in &mut p.items {
        if let Item::Fn(f) = item {
            let mut stmts = std::mem::take(&mut f.body.stmts);
            for st in stmts.iter_mut() {
                substitute_stmt_all(st, served);
            }
            f.body.stmts = stmts;
        }
    }
    fn repl(e: &mut Expr, served: &ServedRoot) {
        if let ExprKind::HashExpr { .. } = e.kind {
            if let Some(v) = served.value_at(e.span) {
                e.kind = match v {
                    crate::identity::CtValue::I32(x) => ExprKind::IntLit(x),
                    crate::identity::CtValue::Bool(b) => ExprKind::BoolLit(b),
                };
                return;
            }
        }
        match &mut e.kind {
            ExprKind::HashExpr { expr: inner } => repl(inner, served),
            ExprKind::Call { args, .. }
            | ExprKind::Method { args, .. }
            | ExprKind::MethodCall { args, .. } => {
                for a in args.iter_mut() {
                    match a {
                        Arg::Pos(x) | Arg::MovePos(x) => repl(x, served),
                        _ => {}
                    }
                }
            }
            ExprKind::CallValue { callee, args } => {
                repl(callee, served);
                for a in args.iter_mut() {
                    match a {
                        Arg::Pos(x) | Arg::MovePos(x) => repl(x, served),
                        _ => {}
                    }
                }
            }
            ExprKind::Binary { lhs, rhs, .. } => {
                repl(lhs, served);
                repl(rhs, served);
            }
            ExprKind::Unary { operand, .. } => repl(operand, served),
            ExprKind::Field { base, .. } => repl(base, served),
            _ => {}
        }
    }
    fn substitute_stmt_all(s: &mut Stmt, served: &ServedRoot) {
        use ace_core::source::ast::Stmt;
        match s {
            Stmt::Return { value, .. }
            | Stmt::ExprStmt { value, .. }
            | Stmt::Let { value, .. }
            | Stmt::Assign { value, .. } => repl(value, served),
            Stmt::If {
                cond, then, els, ..
            } => {
                repl(cond, served);
                for st in then.stmts.iter_mut() {
                    substitute_stmt_all(st, served);
                }
                if let Some(b) = els {
                    for st in b.stmts.iter_mut() {
                        substitute_stmt_all(st, served);
                    }
                }
            }
            Stmt::While { cond, body, .. } => {
                repl(cond, served);
                for st in body.stmts.iter_mut() {
                    substitute_stmt_all(st, served);
                }
            }
            Stmt::Loop { body, .. } => {
                for st in body.stmts.iter_mut() {
                    substitute_stmt_all(st, served);
                }
            }
            Stmt::IsMatch { arms, .. } => {
                for a in arms.iter_mut() {
                    for st in a.body.stmts.iter_mut() {
                        substitute_stmt_all(st, served);
                    }
                }
            }
            Stmt::ForIn { iter, body, .. } => {
                repl(iter, served);
                for st in body.stmts.iter_mut() {
                    substitute_stmt_all(st, served);
                }
            }
            Stmt::Break { .. } | Stmt::Continue { .. } => {}
        }
    }
}
