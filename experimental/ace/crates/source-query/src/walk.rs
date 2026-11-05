//! Canonical structural walk of one function body (PLAN-934 §5.5).
//!
//! The walk is the SINGLE definition of the body's structural identity:
//!
//! * every statement and expression node gets a [`NodeId`] in canonical
//!   pre-order (block statements in source order; a statement's child
//!   expressions/blocks in source order);
//! * the same traversal emits the span-free canonical bytes of the body,
//!   whose content version is the body axis of the semantic key.
//!
//! Because ids are positions in the walk of THIS body version, a fact
//! page can only be used against a body whose version matches: whitespace
//! or prefix drift keeps the version (spans are excluded), while any
//! change of structure, literal value, name, operator or argument
//! binding changes it. Names/absolute spans/intern slots are therefore
//! never identity by themselves.
//!
//! The traversal is iterative (explicit task stack): legal operator
//! spines are unbounded, so a recursive walk could overflow on a long
//! chain — the same discipline the core AST/projection apply.

use crate::ident::{content_version, type_canonical, BODY_DOMAIN};
use ace_core::atom_text::Span;
use ace_core::source::ast::{Arg, BinOp, Block, Expr, ExprKind, Stmt, UnaryOp};
use ace_semantics_contract::ContentVersion;

/// Upper bound on one body's structural nodes (a controlled rejection;
/// the plan's deep-chain control is 4096 semantic edges, this is only a
/// sanity ceiling for a single body's syntax nodes).
pub const MAX_BODY_NODES: usize = 1_000_000;

/// Structural node handle inside one body version. Only meaningful
/// together with [`BodyWalk::version`].
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct NodeId(pub u32);

/// One structural node: id, kind tag and the CURRENT source span (the
/// span is source-map data — refreshed on drift, never an identity).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeInfo {
    pub id: NodeId,
    pub kind: &'static str,
    pub span: Span,
}

impl NodeInfo {
    /// Expression nodes carry a type fact; statement nodes only anchor
    /// structure and block scoping.
    pub fn is_expr(&self) -> bool {
        self.kind.starts_with("expr.")
    }
}

/// One `let`/`var` declaration in body order (the binding facts align on
/// this list; the declaration node id is its structural identity).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LetDecl {
    pub node: NodeId,
    pub name: String,
}

/// The canonical walk of one body.
#[derive(Clone, Debug)]
pub struct BodyWalk {
    pub nodes: Vec<NodeInfo>,
    pub lets: Vec<LetDecl>,
    /// Span-free semantic version of exactly this structure.
    pub version: ContentVersion,
    /// Number of expression nodes (their type facts are required for a
    /// complete page).
    pub expr_nodes: usize,
}

impl BodyWalk {
    pub fn node(&self, id: NodeId) -> Option<&NodeInfo> {
        self.nodes.get(id.0 as usize)
    }

    pub fn expr_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes
            .iter()
            .filter(|n| n.is_expr())
            .map(|n| n.id)
    }

    /// Map `(start,end)` → node id for expression nodes of THIS walk.
    /// Duplicate spans are reported (an ambiguous fact mapping is an
    /// infrastructure error, never a guess).
    pub fn span_index(&self) -> Result<std::collections::BTreeMap<(u32, u32), NodeId>, SpanDup> {
        let mut map = std::collections::BTreeMap::new();
        for n in self.nodes.iter().filter(|n| n.is_expr()) {
            let key = (n.span.start as u32, n.span.end as u32);
            if let std::collections::btree_map::Entry::Occupied(prev) = map.entry(key) {
                return Err(SpanDup {
                    start: key.0,
                    end: key.1,
                    first: *prev.get(),
                    second: n.id,
                });
            }
            map.insert(key, n.id);
        }
        Ok(map)
    }
}

/// Two expression nodes share one source span — the fact mapping cannot
/// be resolved structurally.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpanDup {
    pub start: u32,
    pub end: u32,
    pub first: NodeId,
    pub second: NodeId,
}

/// Why a body cannot be described by this bounded SQ-A layer.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum WalkError {
    /// Construct outside the finite ordinary-source surface. Reported,
    /// never silently skipped.
    Unsupported { what: &'static str, span: Span },
    /// Node ceiling of one body exceeded (controlled rejection).
    TooManyNodes { count: usize, max: usize },
}

impl std::fmt::Display for WalkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalkError::Unsupported { what, span } => write!(
                f,
                "`{what}` at {}..{} is outside the finite ordinary-source query surface",
                span.start, span.end
            ),
            WalkError::TooManyNodes { count, max } => {
                write!(f, "body has {count} structural nodes (limit {max})")
            }
        }
    }
}

enum Task<'a> {
    Tok(&'static str),
    Name(&'a str),
    Int(i32),
    Bool(bool),
    Stmt(&'a Stmt),
    Expr(&'a Expr),
    Arg(&'a Arg),
    Block(&'a Block),
}

/// Walk one function body: structural ids + span-free body version.
pub fn walk_body(body: &Block) -> Result<BodyWalk, WalkError> {
    let mut nodes: Vec<NodeInfo> = Vec::new();
    let mut lets: Vec<LetDecl> = Vec::new();
    let mut canon: Vec<u8> = Vec::new();
    let mut stack: Vec<Task<'_>> = vec![Task::Block(body)];
    while let Some(task) = stack.pop() {
        match task {
            Task::Tok(t) => {
                canon.extend_from_slice(t.as_bytes());
                canon.push(0);
            }
            Task::Name(n) => {
                canon.extend_from_slice(n.as_bytes());
                canon.push(0);
            }
            Task::Int(v) => {
                canon.extend_from_slice(&v.to_le_bytes());
                canon.push(0);
            }
            Task::Bool(b) => canon.push(u8::from(b)),
            Task::Block(b) => {
                canon.extend_from_slice(b"block");
                canon.push(0);
                stack.push(Task::Tok("end-block"));
                for s in b.stmts.iter().rev() {
                    stack.push(Task::Stmt(s));
                }
            }
            Task::Stmt(s) => walk_stmt(s, &mut nodes, &mut lets, &mut canon, &mut stack)?,
            Task::Expr(e) => walk_expr(e, &mut nodes, &mut canon, &mut stack)?,
            Task::Arg(a) => match a {
                Arg::Pos(e) => {
                    canon.extend_from_slice(b"pos");
                    canon.push(0);
                    stack.push(Task::Expr(e));
                }
                Arg::MovePos(e) => {
                    canon.extend_from_slice(b"move");
                    canon.push(0);
                    stack.push(Task::Expr(e));
                }
                Arg::Named { name, value, .. } => {
                    canon.extend_from_slice(b"named");
                    canon.push(0);
                    canon.extend_from_slice(name.name.as_bytes());
                    canon.push(0);
                    stack.push(Task::Expr(value));
                }
            },
        }
        if nodes.len() > MAX_BODY_NODES {
            return Err(WalkError::TooManyNodes {
                count: nodes.len(),
                max: MAX_BODY_NODES,
            });
        }
    }
    let expr_nodes = nodes.iter().filter(|n| n.is_expr()).count();
    Ok(BodyWalk {
        nodes,
        lets,
        version: content_version(BODY_DOMAIN, &canon),
        expr_nodes,
    })
}

fn push_node(
    nodes: &mut Vec<NodeInfo>,
    canon: &mut Vec<u8>,
    kind: &'static str,
    span: Span,
) -> NodeId {
    let id = NodeId(nodes.len() as u32);
    nodes.push(NodeInfo { id, kind, span });
    canon.extend_from_slice(kind.as_bytes());
    canon.push(0);
    id
}

fn walk_stmt<'a>(
    s: &'a Stmt,
    nodes: &mut Vec<NodeInfo>,
    lets: &mut Vec<LetDecl>,
    canon: &mut Vec<u8>,
    stack: &mut Vec<Task<'a>>,
) -> Result<(), WalkError> {
    match s {
        Stmt::Let {
            name,
            ann,
            mutable,
            value,
            span,
            ..
        } => {
            let id = push_node(nodes, canon, "stmt.let", *span);
            lets.push(LetDecl {
                node: id,
                name: name.name.clone(),
            });
            canon.extend_from_slice(if *mutable { b"var" } else { b"let" });
            canon.push(0);
            canon.extend_from_slice(name.name.as_bytes());
            canon.push(0);
            match ann {
                Some(a) => {
                    canon.extend_from_slice(b"ann");
                    canon.push(0);
                    type_canonical(a, canon);
                }
                None => {
                    canon.extend_from_slice(b"no-ann");
                    canon.push(0);
                }
            }
            stack.push(Task::Tok("end-let"));
            stack.push(Task::Expr(value));
        }
        Stmt::Assign {
            target,
            value,
            span,
        } => {
            push_node(nodes, canon, "stmt.assign", *span);
            canon.extend_from_slice(b"assign");
            canon.push(0);
            canon.extend_from_slice(target.base.name.as_bytes());
            canon.push(0);
            for p in &target.path {
                canon.extend_from_slice(p.name.as_bytes());
                canon.push(0);
            }
            stack.push(Task::Tok("end-assign"));
            stack.push(Task::Expr(value));
        }
        Stmt::Return { value, span } => {
            push_node(nodes, canon, "stmt.return", *span);
            canon.extend_from_slice(b"return");
            canon.push(0);
            stack.push(Task::Tok("end-return"));
            stack.push(Task::Expr(value));
        }
        Stmt::ExprStmt { value, span } => {
            push_node(nodes, canon, "stmt.expr", *span);
            canon.extend_from_slice(b"expr-stmt");
            canon.push(0);
            stack.push(Task::Tok("end-expr-stmt"));
            stack.push(Task::Expr(value));
        }
        Stmt::If {
            cond,
            then,
            els,
            span,
        } => {
            push_node(nodes, canon, "stmt.if", *span);
            canon.extend_from_slice(b"if");
            canon.push(0);
            stack.push(Task::Tok("end-if"));
            if let Some(e) = els {
                stack.push(Task::Block(e));
            } else {
                stack.push(Task::Tok("no-else"));
            }
            stack.push(Task::Block(then));
            stack.push(Task::Expr(cond));
        }
        Stmt::While { cond, body, span } => {
            push_node(nodes, canon, "stmt.while", *span);
            canon.extend_from_slice(b"while");
            canon.push(0);
            stack.push(Task::Tok("end-while"));
            stack.push(Task::Block(body));
            stack.push(Task::Expr(cond));
        }
        Stmt::Loop { body, span } => {
            push_node(nodes, canon, "stmt.loop", *span);
            canon.extend_from_slice(b"loop");
            canon.push(0);
            stack.push(Task::Tok("end-loop"));
            stack.push(Task::Block(body));
        }
        Stmt::Break { span } => {
            push_node(nodes, canon, "stmt.break", *span);
            canon.extend_from_slice(b"break");
            canon.push(0);
        }
        Stmt::Continue { span } => {
            push_node(nodes, canon, "stmt.continue", *span);
            canon.extend_from_slice(b"continue");
            canon.push(0);
        }
        // Collections/entry-unit statements are outside the finite
        // ordinary-source i32/bool surface of this step.
        Stmt::IsMatch { span, .. } => {
            return Err(WalkError::Unsupported {
                what: "is-dispatch",
                span: *span,
            })
        }
        Stmt::ForIn { span, .. } => {
            return Err(WalkError::Unsupported {
                what: "for-in",
                span: *span,
            })
        }
    }
    Ok(())
}

fn walk_expr<'a>(
    e: &'a Expr,
    nodes: &mut Vec<NodeInfo>,
    canon: &mut Vec<u8>,
    stack: &mut Vec<Task<'a>>,
) -> Result<(), WalkError> {
    match &e.kind {
        ExprKind::IntLit(v) => {
            push_node(nodes, canon, "expr.int", e.span);
            canon.extend_from_slice(b"int");
            canon.push(0);
            stack.push(Task::Int(*v));
        }
        ExprKind::BoolLit(b) => {
            push_node(nodes, canon, "expr.bool", e.span);
            canon.extend_from_slice(b"bool");
            canon.push(0);
            stack.push(Task::Bool(*b));
        }
        ExprKind::Var(id) => {
            push_node(nodes, canon, "expr.var", e.span);
            canon.extend_from_slice(b"var");
            canon.push(0);
            stack.push(Task::Name(&id.name));
        }
        ExprKind::Call { callee, args } => {
            push_node(nodes, canon, "expr.call", e.span);
            canon.extend_from_slice(b"call");
            canon.push(0);
            canon.extend_from_slice(callee.name.as_bytes());
            canon.push(0);
            canon.extend_from_slice(&(args.len() as u32).to_le_bytes());
            for a in args.iter().rev() {
                stack.push(Task::Arg(a));
            }
        }
        ExprKind::Binary { op, lhs, rhs } => {
            push_node(nodes, canon, "expr.binary", e.span);
            canon.extend_from_slice(b"bin");
            canon.push(0);
            canon.extend_from_slice(bin_tag(*op).as_bytes());
            canon.push(0);
            stack.push(Task::Expr(rhs));
            stack.push(Task::Expr(lhs));
        }
        ExprKind::Unary { op, operand } => {
            push_node(nodes, canon, "expr.unary", e.span);
            canon.extend_from_slice(b"un");
            canon.push(0);
            canon.extend_from_slice(unary_tag(*op).as_bytes());
            canon.push(0);
            stack.push(Task::Expr(operand));
        }
        ExprKind::StrLit(_) => unsupported("str-literal", e.span)?,
        ExprKind::CharLit(_) => unsupported("char-literal", e.span)?,
        ExprKind::Method { .. } => unsupported("text-method", e.span)?,
        ExprKind::TypeWordArg(_) => unsupported("type-word-arg", e.span)?,
        ExprKind::CallValue { .. } => unsupported("closure-call", e.span)?,
        ExprKind::CtorNamed { .. } => unsupported("record-construction", e.span)?,
        ExprKind::Field { .. } => unsupported("field-read", e.span)?,
        ExprKind::MethodCall { .. } => unsupported("record-method-call", e.span)?,
        ExprKind::MutArg { .. } => unsupported("mut-arg", e.span)?,
        ExprKind::GenericType { .. } => unsupported("generic-type-receiver", e.span)?,
        ExprKind::Closure { .. } => unsupported("closure-expression", e.span)?,
    }
    Ok(())
}

fn unsupported(what: &'static str, span: Span) -> Result<(), WalkError> {
    Err(WalkError::Unsupported { what, span })
}

fn bin_tag(op: BinOp) -> &'static str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::Lt => "<",
        BinOp::Le => "<=",
        BinOp::Gt => ">",
        BinOp::Ge => ">=",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::And => "&&",
        BinOp::Or => "||",
    }
}

fn unary_tag(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
    }
}
