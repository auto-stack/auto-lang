//! Read-only ac-core AST adapter (PLAN-796 §5.2) — feature `core-ast`.
//!
//! Projects the existing ac-core PUBLIC AST (parsed by
//! `parse_program_with`) into the neutral [`SemaInput`] model, and
//! exposes the OLD pipeline verdict (resolve + typecheck) for
//! differential cross-checks. The projection is READ-ONLY: this crate
//! never rewrites the parser, never calls the old typecheck and relabels
//! its output as new inference — the two run independently and results
//! are compared by tests.
//!
//! Constructions outside the limited inference surface (text methods,
//! records in value position, named-construction, field paths in
//! assignment targets…) project to explicit `Unsupported` nodes so the
//! solver reports `unsupported-construct` — never a silent mistype.
//! Named call arguments are mapped to positional form here (argument
//! NAME→POSITION mapping is a frontend concern; inference checks types).

use crate::input::{
    Ann, BinOpKind, InputBlock, InputExpr, InputExprKind, InputFn, InputRecord, InputStmt,
    SemaInput, Span,
};
#[cfg(feature = "core-ast")]
use auto_ac_prototype::source::ast as core_ast;
#[cfg(feature = "core-ast")]
use auto_ac_prototype::source::SourceProfile;

/// Typed language load failure (F-796-R6-07): a PARSE diagnostic is a
/// language verdict (typed code/message, judged against the case
/// oracle), never an infrastructure error. Projection failures remain
/// infrastructure.
#[cfg(feature = "core-ast")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LanguageDiagnostic {
    Parse { code: String, message: String },
    Projection(String),
}

/// Parse `.at` source under a profile and project it into the neutral
/// input model. A parse diagnostic is surfaced as a TYPED language
/// error; the probe never invents a partial program.
#[cfg(feature = "core-ast")]
pub fn parse_into_input_typed(
    src: &str,
    package: &str,
    module: &str,
    profile: SourceProfile,
) -> Result<SemaInput, LanguageDiagnostic> {
    let prog = auto_ac_prototype::source::parse_program_with(src, profile).map_err(|d| {
        LanguageDiagnostic::Parse {
            code: d.code,
            message: d.message,
        }
    })?;
    let mut input = project(&prog, package, module).map_err(LanguageDiagnostic::Projection)?;
    input.source_fp =
        ac_sema_contract::identity::domain_hash("ac796/source-fp/v1", &[src.as_bytes()]);
    Ok(input)
}

/// Back-compat string form used by library/bin callers.
#[cfg(feature = "core-ast")]
pub fn parse_into_input(
    src: &str,
    package: &str,
    module: &str,
    profile: SourceProfile,
) -> Result<SemaInput, String> {
    parse_into_input_typed(src, package, module, profile).map_err(|e| match e {
        LanguageDiagnostic::Parse { code, message } => format!("parse: {} ({})", message, code),
        LanguageDiagnostic::Projection(m) => m,
    })
}

/// The old pipeline's verdict for differential cross-checks: accept iff
/// resolve AND typecheck both pass; on accept, the resolved local
/// binding types come back for fact comparison.
#[cfg(feature = "core-ast")]
pub fn old_pipeline_verdict(src: &str, profile: SourceProfile) -> OldVerdict {
    let prog = match auto_ac_prototype::source::parse_program_with(src, profile) {
        Ok(p) => p,
        Err(d) => return OldVerdict::Rejected(format!("parse: {}", d.message)),
    };
    let resolved = match auto_ac_prototype::source::resolve_with(&prog, &[], profile) {
        Ok(r) => r,
        Err(diags) => return OldVerdict::Rejected(format!("resolve: {}", diags[0].message)),
    };
    match auto_ac_prototype::source::typecheck_with(&resolved, profile) {
        Ok(()) => OldVerdict::Accepted(Box::new(OldFacts {
            fns: resolved
                .fns
                .iter()
                .map(|f| OldFn {
                    name: f.name.clone(),
                    params: f
                        .param_names
                        .iter()
                        .map(|n| n.clone())
                        .zip(f.params.iter().map(vt_name))
                        .collect(),
                    result: vt_name(&f.result),
                    locals: f
                        .locals
                        .iter()
                        .map(|l| (l.name.clone(), vt_name(&l.vt)))
                        .collect(),
                })
                .collect(),
        })),
        Err(diags) => OldVerdict::Rejected(format!("typecheck: {}", diags[0].message)),
    }
}

fn vt_name(v: &auto_ac_prototype::source::resolve::VT) -> &'static str {
    use auto_ac_prototype::source::resolve::VT;
    match v {
        VT::I32 => "i32",
        VT::Bool => "bool",
        VT::Str | VT::StrOwned | VT::Char => "text",
        VT::Rec(_) => "record",
        // PLAN-862: Unit is a result-only type; the semantic adapter has no
        // unit value channel, so it is reported as its own name (never
        // folded into "i32").
        VT::Unit => "unit",
        // PLAN-864: collections composites are outside this adapter's
        // limited inference surface; they report their own kind names.
        VT::List(_) => "list",
        VT::Variant(_) => "variant",
    }
}

/// Outcome of the old pipeline on one source.
#[cfg(feature = "core-ast")]
#[derive(Debug)]
pub enum OldVerdict {
    Accepted(Box<OldFacts>),
    Rejected(String),
}

#[cfg(feature = "core-ast")]
#[derive(Debug)]
pub struct OldFacts {
    pub fns: Vec<OldFn>,
}

#[cfg(feature = "core-ast")]
#[derive(Debug)]
pub struct OldFn {
    pub name: String,
    pub params: Vec<(String, &'static str)>,
    pub result: &'static str,
    /// (name, type) per local in registration order.
    pub locals: Vec<(String, &'static str)>,
}

#[cfg(feature = "core-ast")]
pub fn project(prog: &core_ast::Program, package: &str, module: &str) -> Result<SemaInput, String> {
    let mut fns = Vec::new();
    let mut records = Vec::new();
    for item in &prog.items {
        match item {
            core_ast::Item::Fn(f) => fns.push(project_fn(f)?),
            // PLAN-864: enum declarations project nothing here (no
            // functions; the sema input has no variant record shape).
            core_ast::Item::Enum(_) => {}
            core_ast::Item::Type(t) => {
                records.push(InputRecord {
                    name: t.name.name.clone(),
                    fields: t
                        .fields
                        .iter()
                        .map(|fd| Ok::<_, String>((fd.name.name.clone(), ann_of(&fd.ty)?)))
                        .collect::<Result<Vec<_>, _>>()?,
                    methods: t
                        .methods
                        .iter()
                        .map(project_fn)
                        .collect::<Result<Vec<_>, _>>()?,
                });
            }
        }
    }
    Ok(SemaInput {
        package: package.into(),
        module: module.into(),
        fns,
        records,
        source_fp: String::new(),
    })
}

fn project_fn(f: &core_ast::FnItem) -> Result<InputFn, String> {
    Ok(InputFn {
        name: f.name.name.clone(),
        params: f
            .params
            .iter()
            .map(|p| Ok::<_, String>((p.name.name.clone(), ann_of(&p.ty)?)))
            .collect::<Result<Vec<_>, _>>()?,
        ret: ann_of(&f.ret)?,
        body: project_block(&f.body)?,
    })
}

fn ann_of(a: &core_ast::TypeAnn) -> Result<Ann, String> {
    match &a.kind {
        core_ast::TypeRef::Builtin(w) => Ok(match w {
            core_ast::TypeWord::Int | core_ast::TypeWord::I32 => Ann::I32,
            core_ast::TypeWord::Bool => Ann::Bool,
            // text/char annotations are outside the limited profile
            core_ast::TypeWord::Str | core_ast::TypeWord::StrOwned | core_ast::TypeWord::Char => {
                Ann::Unsupported("text-annotation")
            }
            // PLAN-862: the Unit result word is outside this adapter's
            // annotation model (it is a RESULT type, not a value type).
            core_ast::TypeWord::Void => Ann::Unsupported("unit-annotation"),
        }),
        core_ast::TypeRef::Record(ident) => Ok(Ann::Record(record_static(&ident.name)?)),
        // PLAN-864: the collections type grammar (?T / Name<..>) is outside
        // this adapter's annotation model.
        core_ast::TypeRef::Optional(_) | core_ast::TypeRef::Generic { .. } => {
            Ok(Ann::Unsupported("collections-type-annotation"))
        }
    }
}

fn record_static(name: &str) -> Result<&'static str, String> {
    match name {
        "Pair" => Ok("Pair"),
        "Node" => Ok("Node"),
        "List" => Ok("List"),
        "Tree" => Ok("Tree"),
        "Cell" => Ok("Cell"),
        "Outer" => Ok("Outer"),
        "Inner" => Ok("Inner"),
        other => registered(other),
    }
}

/// BOUNDED registry for dynamically-named records (F-796-R4-08): a
/// fixed-capacity intern table replaces unbounded `Box::leak` — the
/// residency is accounted and a table overflow is a controlled error,
/// not silent process-lifetime growth.
fn registered(name: &str) -> Result<&'static str, String> {
    use std::collections::BTreeSet;
    use std::sync::Mutex;
    const MAX_INTERNED: usize = 8192;
    static TABLE: Mutex<Option<BTreeSet<&'static str>>> = Mutex::new(None);
    let mut guard = TABLE.lock().map_err(|_| "intern table poisoned")?;
    let table = guard.get_or_insert_with(BTreeSet::new);
    if let Some(s) = table.get(name) {
        return Ok(s);
    }
    if table.len() >= MAX_INTERNED {
        return Err(format!(
            "record name registry full ({} entries) — refusing unbounded residency",
            MAX_INTERNED
        ));
    }
    let leaked: &'static str = Box::leak(name.to_string().into_boxed_str());
    table.insert(leaked);
    Ok(leaked)
}

fn project_block(b: &core_ast::Block) -> Result<InputBlock, String> {
    Ok(InputBlock {
        stmts: b
            .stmts
            .iter()
            .map(project_stmt)
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn project_stmt(s: &core_ast::Stmt) -> Result<InputStmt, String> {
    Ok(match s {
        core_ast::Stmt::Let {
            name,
            ann,
            mutable,
            value,
            span: sp,
            ..
        } => InputStmt::Let {
            name: name.name.clone(),
            ann: ann
                .as_ref()
                .map(ann_of)
                .transpose()?
                .unwrap_or(Ann::Inferred),
            mutable: *mutable,
            value: project_expr(&value),
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::Assign {
            target,
            value,
            span: sp,
        } => InputStmt::Assign {
            target: target.base.name.clone(),
            path: target.path.iter().map(|p| p.name.clone()).collect(),
            value: project_expr(&value),
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::Return { value, span: sp } => InputStmt::Return {
            value: project_expr(&value),
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::If {
            cond,
            then,
            els,
            span: sp,
        } => InputStmt::If {
            cond: project_expr(&cond),
            then: project_block(&then)?,
            els: els.as_ref().map(project_block).transpose()?,
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::While {
            cond,
            body,
            span: sp,
        } => InputStmt::While {
            cond: project_expr(&cond),
            body: project_block(&body)?,
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::Loop { body, span: sp } => InputStmt::Loop {
            body: project_block(&body)?,
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::Break { span: sp } => InputStmt::Break {
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        core_ast::Stmt::Continue { span: sp } => InputStmt::Continue {
            span: Span::new(sp.start as u32, sp.end as u32),
        },
        // PLAN-862: an expression statement only exists as the entry-unit
        // script tail, which has no projection into this restricted
        // inference surface — a controlled rejection, never a silent drop.
        core_ast::Stmt::ExprStmt { .. } => {
            return Err(
                "expression statement (entry-unit script tail) is outside this adapter".into(),
            )
        }
        // PLAN-864: the collections statements are outside this limited
        // inference surface — controlled rejections, never silent drops.
        core_ast::Stmt::IsMatch { span, .. } => {
            let _ = span;
            return Err("`is` dispatch is outside this adapter's inference surface".into());
        }
        core_ast::Stmt::ForIn { span, .. } => {
            let _ = span;
            return Err("`for-in` iteration is outside this adapter's inference surface".into());
        }
    })
}

fn project_arg(a: &core_ast::Arg) -> Option<InputExpr> {
    match a {
        core_ast::Arg::Pos(e) | core_ast::Arg::MovePos(e) => Some(project_expr(e)),
        // named-argument calls are outside the limited inference surface:
        // the whole call projects to an explicit unsupported node
        core_ast::Arg::Named { .. } => None,
    }
}

fn project_expr(e: &core_ast::Expr) -> InputExpr {
    let span = Span::new(e.span.start as u32, e.span.end as u32);
    let kind = match &e.kind {
        core_ast::ExprKind::IntLit(v) => InputExprKind::IntLit(*v),
        core_ast::ExprKind::BoolLit(v) => InputExprKind::BoolLit(*v),
        core_ast::ExprKind::Var(v) => InputExprKind::Var(v.name.clone()),
        core_ast::ExprKind::Call { callee, args } => {
            if args
                .iter()
                .any(|a| matches!(a, core_ast::Arg::Named { .. }))
            {
                InputExprKind::Unsupported {
                    what: "named-arg-call",
                }
            } else {
                InputExprKind::Call {
                    callee: callee.name.clone(),
                    args: args.iter().filter_map(project_arg).collect(),
                }
            }
        }
        core_ast::ExprKind::Binary { op, lhs, rhs } => InputExprKind::Binary {
            op: binop(*op),
            lhs: Box::new(project_expr(&lhs)),
            rhs: Box::new(project_expr(&rhs)),
        },
        core_ast::ExprKind::Unary { op, operand } => InputExprKind::Unary {
            neg: matches!(op, core_ast::UnaryOp::Neg),
            operand: Box::new(project_expr(&operand)),
        },
        // -- outside the limited inference surface: explicit unsupported --
        core_ast::ExprKind::StrLit(_) => InputExprKind::Unsupported {
            what: "str-literal",
        },
        core_ast::ExprKind::CharLit(_) => InputExprKind::Unsupported {
            what: "char-literal",
        },
        core_ast::ExprKind::Method { .. } => InputExprKind::Unsupported {
            what: "text-method",
        },
        core_ast::ExprKind::TypeWordArg(_) => InputExprKind::Unsupported {
            what: "type-word-arg",
        },
        core_ast::ExprKind::CtorNamed { .. } => InputExprKind::Unsupported {
            what: "record-construction",
        },
        core_ast::ExprKind::Field { .. } => InputExprKind::Unsupported { what: "field-read" },
        core_ast::ExprKind::MethodCall { .. } => InputExprKind::Unsupported {
            what: "record-method-call",
        },
        core_ast::ExprKind::MutArg { .. } => InputExprKind::Unsupported { what: "mut-arg" },
        // PLAN-864: a generic type receiver (List<T>.new()) is outside the
        // limited inference surface.
        core_ast::ExprKind::GenericType { .. } => InputExprKind::Unsupported {
            what: "generic-type-receiver",
        },
    };
    InputExpr { kind, span }
}

fn binop(op: core_ast::BinOp) -> BinOpKind {
    use core_ast::BinOp as B;
    match op {
        B::Add => BinOpKind::Add,
        B::Sub => BinOpKind::Sub,
        B::Mul => BinOpKind::Mul,
        B::Div => BinOpKind::Div,
        B::Rem => BinOpKind::Rem,
        B::Lt => BinOpKind::Lt,
        B::Le => BinOpKind::Le,
        B::Gt => BinOpKind::Gt,
        B::Ge => BinOpKind::Ge,
        B::Eq => BinOpKind::Eq,
        B::Ne => BinOpKind::Ne,
        B::And => BinOpKind::And,
        B::Or => BinOpKind::Or,
    }
}
