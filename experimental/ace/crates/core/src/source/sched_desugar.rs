//! PLAN-968 T-02: the scheduler-surface AST desugar.
//!
//! One pre-resolve rewrite over the scheduler profile's `ast::Program`:
//!
//! * every `task Name { fields | fn start() ! | fn stop() ! | on { arms } }`
//!   declaration becomes ONE synthesized driver function
//!   `__task_<Name>(<field inits...>) void` — state fields are mutable
//!   locals initialized from the spawn-time parameters, the `start` hook
//!   runs first, then a `while true` receive loop dispatches the `on`
//!   arms on the received message literal, and a stop check runs the
//!   `stop` hook and leaves the loop;
//! * the user-facing operations become reserved intrinsic calls the
//!   resolver types against the side-table this pass returns:
//!   `Task.spawn("N", cap, inits..)` → `__sched_spawn("N", cap, inits..)`,
//!   `h.send(m)` → `__sched_send(h, m)`, `h.ask(m)` → `__sched_ask(h, m)`,
//!   `h.stop()` → `__sched_stop(h)`, `reply(v)` → `__sched_reply(v)`,
//!   `sleep(ms)` → `__sched_sleep(ms)`, `await f` → `__sched_await(f)`,
//!   `go f(args)` → `__sched_go("f", args...)`.
//!
//! The rewrite is purely structural; every type decision (handle types,
//! message/reply agreement, unknown task names, stray intrinsic names)
//! stays in `resolve` via the returned [`SchedSurface`] side-table, which
//! carries the per-task field/message/reply facts this pass derives from
//! the declaration itself (literals and arm annotations — the frozen
//! M2-A declaration domain).

use super::ast::{
    Arg, Block, Expr, ExprKind, FnItem, Ident, Item, OnArm, Param, ParamMode, Receiver, Stmt,
    TaskItem, TaskPattern, TypeAnn, TypeRef, TypeWord,
};
use crate::atom_text::{Diagnostic, Span, Stage};

/// Reserved scheduler intrinsic callee names (never user-declarable on
/// this face).
pub const SCHED_INTRINSICS: &[&str] = &[
    "__sched_spawn",
    "__sched_singleton",
    "__sched_send",
    "__sched_ask",
    "__sched_stop",
    "__sched_reply",
    "__sched_sleep",
    "__sched_await",
    "__sched_go",
    "__sched_recv",
    "__sched_port",
    "__sched_stopping",
];

/// One desugared task's declaration facts (resolve-side side-table).
#[derive(Clone, Debug)]
pub struct SchedTaskInfo {
    /// User-facing task name (`task Echo` → "Echo").
    pub name: String,
    pub single: bool,
    /// The synthesized driver function's name (`__task_Echo`).
    pub driver_fn: String,
    /// State fields in declaration order: (name, value type word).
    pub fields: Vec<(String, TypeWord, Expr)>,
    /// The unified message type word.
    pub msg: TypeAnn,
    /// The inferred reply type word (i32 when no `reply` call exists).
    pub reply: TypeWord,
    /// The declaration span (diagnostics).
    pub span: Span,
}

/// The whole scheduler surface's side-table.
#[derive(Clone, Debug, Default)]
pub struct SchedSurface {
    /// Tasks by user-facing name.
    pub tasks: Vec<SchedTaskInfo>,
}

impl SchedSurface {
    pub fn by_name(&self, name: &str) -> Option<&SchedTaskInfo> {
        self.tasks.iter().find(|t| t.name == name)
    }
    pub fn by_driver(&self, driver: &str) -> Option<&SchedTaskInfo> {
        self.tasks.iter().find(|t| t.driver_fn == driver)
    }
}

fn diag(code: &str, message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new(Stage::Text, code, message, span)
}

fn ident(name: &str, span: Span) -> Ident {
    Ident {
        name: name.to_string(),
        span,
    }
}

fn word_ann(w: TypeWord, span: Span) -> TypeAnn {
    TypeAnn {
        kind: TypeRef::Builtin(w),
        span,
    }
}

fn lit_expr(kind: ExprKind, span: Span) -> Expr {
    Expr { span, kind }
}

fn call(callee: &str, args: Vec<Expr>, span: Span) -> Expr {
    lit_expr(
        ExprKind::Call {
            callee: ident(callee, span),
            args: args.into_iter().map(Arg::Pos).collect(),
        },
        span,
    )
}

/// Desugar one scheduler-profile program. Returns the rewritten program
/// plus the side-table; all rejections are named `source.task-*`
/// diagnostics with byte spans into the original source.
pub fn desugar_program(
    program: &super::ast::Program,
) -> Result<(super::ast::Program, SchedSurface), Vec<Diagnostic>> {
    let (mut program, surface) = desugar_tasks(program, false)?;
    let mut diags = Vec::new();
    for item in &mut program.items {
        match item {
            Item::Fn(f) => rewrite_fn(f, &mut diags),
            Item::Type(t) => {
                for m in &mut t.methods {
                    rewrite_fn(m, &mut diags);
                }
            }
            _ => {}
        }
    }
    if diags.is_empty() {
        Ok((program, surface))
    } else {
        Err(diags)
    }
}

/// Project actor declarations into shared checked driver functions. Ordinary
/// Auto binds the operations using the actual receiver and argument types.
pub(crate) fn desugar_tasks(
    program: &super::ast::Program,
    ordinary: bool,
) -> Result<(super::ast::Program, SchedSurface), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let mut surface = SchedSurface::default();
    let mut items = Vec::with_capacity(program.items.len());
    for item in &program.items {
        match item {
            Item::Task(t) => {
                if ordinary {
                    if let Some(messages) = actor_message_enum(t, &mut diags) {
                        items.push(Item::Enum(messages));
                    }
                }
                match build_task(t, &mut surface, ordinary) {
                    Ok(driver) => items.push(Item::Fn(driver)),
                    Err(mut ds) => diags.append(&mut ds),
                }
            }
            other => items.push(other.clone()),
        }
    }

    if !diags.is_empty() {
        return Err(diags);
    }
    Ok((super::ast::Program { items }, surface))
}

/// The value-type word of one field default / reply literal (the frozen
/// M2-A declaration domain: i32 / bool / Str literals).
fn literal_word(e: &Expr, what: &str, code: &str, diags: &mut Vec<Diagnostic>) -> Option<TypeWord> {
    match &e.kind {
        ExprKind::IntLit(_) => Some(TypeWord::I32),
        ExprKind::BoolLit(_) => Some(TypeWord::Bool),
        ExprKind::StrLit(_) => Some(TypeWord::StrOwned),
        _ => {
            diags.push(diag(
                code,
                format!("a {what} must be an int, bool or string literal on the scheduler surface"),
                e.span,
            ));
            None
        }
    }
}

/// The message type word of one arm pattern.
fn pattern_word(p: &TaskPattern, diags: &mut Vec<Diagnostic>) -> Option<TypeWord> {
    match p {
        TaskPattern::Lit(e) => literal_word(
            e,
            "message pattern literal",
            "source.task-message-type",
            diags,
        ),
        TaskPattern::Bind { ty, .. } => match &ty.kind {
            TypeRef::Builtin(w @ (TypeWord::I32 | TypeWord::Int | TypeWord::Bool)) => {
                Some(if matches!(w, TypeWord::Int) {
                    TypeWord::I32
                } else {
                    *w
                })
            }
            TypeRef::Builtin(TypeWord::StrOwned) | TypeRef::Builtin(TypeWord::Str) => {
                Some(TypeWord::StrOwned)
            }
            _ => {
                diags.push(diag(
                    "source.task-message-type",
                    "a message binding type must be int/i32/bool or Str on the scheduler surface",
                    ty.span,
                ));
                None
            }
        },
        TaskPattern::Wildcard(_) | TaskPattern::Named { .. } => None,
    }
}

/// Scan one block for `reply(<literal>)` value words (reply type
/// inference; the first literal wins, mismatches are resolved later).
fn scan_reply_word(b: &Block, out: &mut Option<(TypeWord, Span)>) {
    for s in &b.stmts {
        scan_reply_word_stmt(s, out);
    }
}

fn scan_reply_word_stmt(s: &Stmt, out: &mut Option<(TypeWord, Span)>) {
    match s {
        Stmt::ExprStmt { value, .. } => scan_reply_word_expr(value, out),
        Stmt::Let { value, .. } | Stmt::Assign { value, .. } => scan_reply_word_expr(value, out),
        Stmt::IndexAssign(ia) => {
            scan_reply_word_expr(&ia.value, out);
        }
        Stmt::LetDestruct { value, .. } => scan_reply_word_expr(value, out),
        Stmt::Return { value, .. } => scan_reply_word_expr(value, out),
        Stmt::If {
            cond, then, els, ..
        } => {
            scan_reply_word_expr(cond, out);
            scan_reply_word(then, out);
            if let Some(e) = els {
                scan_reply_word(e, out);
            }
        }
        Stmt::While { cond, body, .. } => {
            scan_reply_word_expr(cond, out);
            scan_reply_word(body, out);
        }
        Stmt::Loop { body, .. } => scan_reply_word(body, out),
        Stmt::Hold { body, .. } => scan_reply_word(body, out),
        _ => {}
    }
}

fn scan_reply_word_expr(e: &Expr, out: &mut Option<(TypeWord, Span)>) {
    match &e.kind {
        // The live CST front end projects `reply(v)` straight to the
        // `__sched_reply` intrinsic (cst/project.rs), so the scan must see
        // BOTH spellings — otherwise every task's reply type silently
        // defaulted to i32 and non-int replies were unreachable from source.
        ExprKind::Call { callee, args }
            if (callee.name == "reply" || callee.name == "__sched_reply") && args.len() == 1 =>
        {
            if let Arg::Pos(v) = &args[0] {
                if let Some(w) =
                    literal_word(v, "reply value", "source.task-reply-type", &mut Vec::new())
                {
                    if out.is_none() {
                        *out = Some((w, v.span));
                    }
                }
            }
        }
        _ => {}
    }
    // Recurse structurally for nested reply calls.
    match &e.kind {
        ExprKind::Binary { lhs, rhs, .. } => {
            scan_reply_word_expr(lhs, out);
            scan_reply_word_expr(rhs, out);
        }
        ExprKind::Unary { operand, .. } | ExprKind::Await(operand) | ExprKind::Go(operand) => {
            scan_reply_word_expr(operand, out);
        }
        ExprKind::Call { args, .. } | ExprKind::ConstCall { args, .. } => {
            for a in args {
                if let Arg::Pos(v) = a {
                    scan_reply_word_expr(v, out);
                }
            }
        }
        ExprKind::MethodCall { recv, args, .. } => {
            scan_reply_word_expr(recv, out);
            for a in args {
                if let Arg::Pos(v) = a {
                    scan_reply_word_expr(v, out);
                }
            }
        }
        _ => {}
    }
}

/// Typed patterns set the mailbox type. Untyped integer handlers use the
/// legacy integer message width so their handles match TaskRef<i64>.
pub(crate) fn ordinary_message_type(task: &TaskItem, diags: &mut Vec<Diagnostic>) -> TypeAnn {
    if task
        .arms
        .iter()
        .any(|arm| matches!(arm.pattern, TaskPattern::Named { .. }))
    {
        return TypeAnn {
            kind: TypeRef::Record(ident(&actor_message_name(&task.name.name), task.name.span)),
            span: task.name.span,
        };
    }
    let declared = task.arms.iter().find_map(|arm| {
        if let TaskPattern::Bind { ty, .. } = &arm.pattern {
            Some(ty.clone())
        } else {
            None
        }
    });
    let ty = declared.unwrap_or_else(|| {
        let word = task.arms.iter().find_map(|arm| {
            if let TaskPattern::Lit(e) = &arm.pattern {
                literal_word(
                    e,
                    "message pattern literal",
                    "source.task-message-type",
                    diags,
                )
                .map(|w| if w == TypeWord::I32 { TypeWord::I64 } else { w })
            } else {
                None
            }
        });
        word_ann(word.unwrap_or(TypeWord::I64), task.span)
    });
    match ty.kind {
        TypeRef::Builtin(TypeWord::Str) => word_ann(TypeWord::StrOwned, ty.span),
        _ => ty,
    }
}

/// Build one task's driver function + registry entry.
fn build_task(
    t: &TaskItem,
    surface: &mut SchedSurface,
    ordinary: bool,
) -> Result<FnItem, Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let driver_name = format!("__task_{}", t.name.name);

    // Fields: literal-typed defaults (spawn inits override positionally).
    let mut fields = Vec::new();
    for f in &t.fields {
        if let Some(w) = literal_word(
            &f.default,
            "task state field default",
            "source.task-field",
            &mut diags,
        ) {
            fields.push((f.name.name.clone(), w, f.default.clone()));
        } else {
            fields.push((f.name.name.clone(), TypeWord::I32, f.default.clone()));
        }
    }

    // Hooks: no parameters; bodies carried for inlining.
    for hook in t.start.iter().chain(t.stop.iter()) {
        if !hook.params.is_empty() {
            diags.push(diag(
                "source.task-lifecycle",
                format!(
                    "the `{}` lifecycle hook takes no parameters on the scheduler surface",
                    hook.name.name
                ),
                hook.name.span,
            ));
        }
        if matches!(hook.ret.kind, TypeRef::Builtin(w) if !matches!(w, TypeWord::Void)) {
            diags.push(diag(
                "source.task-lifecycle",
                format!(
                    "the `{}` lifecycle hook returns no value on the scheduler surface",
                    hook.name.name
                ),
                hook.ret.span,
            ));
        }
    }

    // Message type: unify literal/binding patterns (wildcards carry none).
    let mut msg_word: Option<TypeWord> = None;
    for arm in &t.arms {
        if ordinary {
            break;
        }
        if let Some(w) = pattern_word(&arm.pattern, &mut diags) {
            match msg_word {
                None => msg_word = Some(w),
                Some(prev) if prev == w => {}
                Some(prev) => {
                    diags.push(diag(
                        "source.task-message-type",
                        format!(
                            "the `on` arms of task `{}` must share one message type ({} vs {})",
                            t.name.name,
                            word_name(prev),
                            word_name(w)
                        ),
                        arm.span,
                    ));
                }
            }
        }
    }
    let msg = if ordinary {
        ordinary_message_type(t, &mut diags)
    } else {
        word_ann(msg_word.unwrap_or(TypeWord::I32), t.span)
    };

    // Reply type: first reply() literal; i32 when the task never replies.
    let mut reply_word: Option<(TypeWord, Span)> = None;
    for arm in &t.arms {
        scan_reply_word(&arm.body, &mut reply_word);
    }
    let reply = reply_word.map(|(w, _)| w).unwrap_or(TypeWord::I32);

    if !diags.is_empty() {
        return Err(diags);
    }

    // ---- the driver function ----
    let sp = t.span;
    let mut stmts: Vec<Stmt> = Vec::new();

    // 1) state locals from the spawn parameters
    for (i, (fname, _, _)) in fields.iter().enumerate() {
        let init = lit_expr(ExprKind::Var(ident(&format!("__init_{i}"), sp)), sp);
        // The annotation stays implicit: an owned-text field's let-copy
        // materializes through inference (an explicit StrOwned annotation
        // on the param alias elides the copy).
        stmts.push(Stmt::Let {
            kw: sp,
            name: ident(fname, sp),
            ann: None,
            mutable: true,
            value: init,
            span: sp,
        });
    }

    // 2) the start hook body (inlined — it sees the state locals)
    if let Some(start) = &t.start {
        stmts.extend(start.body.stmts.iter().cloned());
    }

    // 3) the receive/dispatch loop
    let mut loop_stmts: Vec<Stmt> = Vec::new();
    // stop check → stop hook + break
    let mut stop_branch: Vec<Stmt> = Vec::new();
    if let Some(stop) = &t.stop {
        stop_branch.extend(stop.body.stmts.iter().cloned());
    }
    stop_branch.push(Stmt::Break { span: sp });
    loop_stmts.push(Stmt::If {
        cond: call("__sched_stopping", Vec::new(), sp),
        then: Block {
            stmts: stop_branch,
            span: sp,
        },
        els: None,
        span: sp,
    });
    // receive message + reply port
    loop_stmts.push(Stmt::Let {
        kw: sp,
        name: ident("__sched_msg", sp),
        ann: Some(msg.clone()),
        mutable: false,
        value: call("__sched_recv", Vec::new(), sp),
        span: sp,
    });
    loop_stmts.push(Stmt::Let {
        kw: sp,
        name: ident("__sched_port", sp),
        ann: Some(word_ann(TypeWord::I32, sp)),
        mutable: false,
        value: call("__sched_port", Vec::new(), sp),
        span: sp,
    });
    // Post-receive stop check: a wake-by-stop must not dispatch the
    // zero-value message (binding/wildcard arms would otherwise run once
    // on the shutdown handoff).
    let mut stop_branch2: Vec<Stmt> = Vec::new();
    if let Some(stop) = &t.stop {
        stop_branch2.extend(stop.body.stmts.iter().cloned());
    }
    stop_branch2.push(Stmt::Break { span: sp });
    loop_stmts.push(Stmt::If {
        cond: call("__sched_stopping", Vec::new(), sp),
        then: Block {
            stmts: stop_branch2,
            span: sp,
        },
        els: None,
        span: sp,
    });
    if ordinary {
        loop_stmts.extend(build_guarded_dispatch(t, &msg));
    } else {
        loop_stmts.extend(build_frozen_dispatch(t));
    }
    stmts.push(Stmt::While {
        cond: lit_expr(ExprKind::BoolLit(true), sp),
        body: Block {
            stmts: loop_stmts,
            span: sp,
        },
        span: sp,
    });
    // The driver leaves the loop only on stop; the trailing return keeps
    // the closures-family `int` result contract (0 = orderly stop).
    stmts.push(Stmt::Return {
        value: lit_expr(ExprKind::IntLit(0), sp),
        span: sp,
    });

    // Params: one per state field (spawn inits).
    let params: Vec<Param> = fields
        .iter()
        .enumerate()
        .map(|(i, (_, w, _))| Param {
            span: sp,
            name: ident(&format!("__init_{i}"), sp),
            ty: word_ann(*w, sp),
            mode: ParamMode::Default,
            default: None,
        })
        .collect();

    surface.tasks.push(SchedTaskInfo {
        name: t.name.name.clone(),
        single: t.single,
        driver_fn: driver_name.clone(),
        fields,
        msg,
        reply,
        span: t.span,
    });

    Ok(FnItem {
        name: ident(&driver_name, t.name.span),
        type_params: Vec::new(),
        params,
        ret: word_ann(TypeWord::I32, sp),
        body: Block { stmts, span: sp },
        span: t.span,
        receiver: Receiver::Free,
        owner: None,
        script: false,
        proto: false,
        test_attribute: None,
    })
}

fn build_frozen_dispatch(t: &TaskItem) -> Option<Stmt> {
    let sp = t.span;
    // dispatch: literal arms -> an if/else-if chain; the (single)
    // binding/wildcard arm becomes the final else. Binding arms re-bind
    // the received message under the pattern name.
    let mut arms: Vec<&OnArm> = t.arms.iter().collect();
    let fallback_pos = arms
        .iter()
        .position(|a| !matches!(a.pattern, TaskPattern::Lit(_)));
    if let Some(fp) = fallback_pos {
        let fb = arms.remove(fp);
        arms.push(fb);
    }
    // (arm cond, arm body block); `None` cond marks the fallback else.
    let mut chain_parts: Vec<(Option<Expr>, Block)> = Vec::new();
    for arm in &arms {
        let block = match &arm.pattern {
            TaskPattern::Lit(_) => Block {
                stmts: arm.body.stmts.clone(),
                span: arm.body.span,
            },
            TaskPattern::Bind { name, .. } => {
                let mut bs = vec![Stmt::Let {
                    kw: arm.span,
                    name: name.clone(),
                    ann: None,
                    mutable: false,
                    value: lit_expr(ExprKind::Var(ident("__sched_msg", arm.span)), arm.span),
                    span: arm.span,
                }];
                bs.extend(arm.body.stmts.iter().cloned());
                Block {
                    stmts: bs,
                    span: arm.body.span,
                }
            }
            TaskPattern::Wildcard(_) => arm.body.clone(),
            TaskPattern::Named { .. } => unreachable!("named variants are ordinary-Auto only"),
        };
        let cond = match &arm.pattern {
            TaskPattern::Lit(l) => Some(lit_expr(
                ExprKind::Binary {
                    op: super::ast::BinOp::Eq,
                    lhs: Box::new(lit_expr(
                        ExprKind::Var(ident("__sched_msg", arm.span)),
                        arm.span,
                    )),
                    rhs: Box::new(l.clone()),
                },
                arm.span,
            )),
            _ => None,
        };
        chain_parts.push((cond, block));
    }
    // Assemble bottom-up: the fallback (None) or the last literal arm
    // closes the chain.
    let mut else_block: Option<Block> = None;
    let mut chain: Option<Stmt> = None;
    for (cond, block) in chain_parts.into_iter().rev() {
        match cond {
            None => {
                // fallback arm — the innermost else
                else_block = Some(block);
            }
            Some(c) => {
                let stmt = Stmt::If {
                    cond: c,
                    then: block,
                    els: else_block.take().or_else(|| {
                        chain.take().map(|inner| {
                            let ispan = inner_span(&inner);
                            Block {
                                stmts: vec![inner],
                                span: ispan,
                            }
                        })
                    }),
                    span: sp,
                };
                chain = Some(stmt);
            }
        }
    }
    if let Some(fallback) = else_block.take() {
        // a fallback with no literal arms: run unconditionally
        chain = Some(Stmt::If {
            cond: lit_expr(ExprKind::BoolLit(true), sp),
            then: fallback,
            els: None,
            span: sp,
        });
    }
    chain
}

pub(crate) const ACTOR_MESSAGE_PREFIX: &str = "__task_message_";
pub(crate) const ACTOR_PAYLOAD_TYPE: &str = "<actor-payload>";

fn actor_message_name(task: &str) -> String {
    format!("{ACTOR_MESSAGE_PREFIX}{task}")
}

fn actor_message_enum(
    task: &TaskItem,
    diags: &mut Vec<Diagnostic>,
) -> Option<super::ast::EnumItem> {
    use super::ast::{EnumArm, EnumItem};

    let mut arms: Vec<EnumArm> = Vec::new();
    for arm in &task.arms {
        if let TaskPattern::Named { name, bindings } = &arm.pattern {
            let element = TypeAnn {
                kind: TypeRef::Generic {
                    name: ident(ACTOR_PAYLOAD_TYPE, name.span),
                    args: Vec::new(),
                },
                span: name.span,
            };
            let payload = match bindings.len() {
                0 => None,
                1 => Some(element),
                count => Some(TypeAnn {
                    kind: TypeRef::Tuple(vec![element; count]),
                    span: name.span,
                }),
            };
            if let Some(previous) = arms.iter().find(|arm| arm.name.name == name.name) {
                let same_count = match (&previous.payload, &payload) {
                    (None, None) => true,
                    (Some(a), Some(b)) => match (&a.kind, &b.kind) {
                        (TypeRef::Tuple(a), TypeRef::Tuple(b)) => a.len() == b.len(),
                        (TypeRef::Generic { .. }, TypeRef::Generic { .. }) => true,
                        _ => false,
                    },
                    _ => false,
                };
                if !same_count {
                    diags.push(diag(
                        "source.task-pattern",
                        "message arms disagree on their payload arity",
                        name.span,
                    ));
                }
            } else {
                arms.push(EnumArm {
                    name: name.clone(),
                    payload,
                    fields: Vec::new(),
                    disc: None,
                    span: name.span,
                });
            }
        }
    }
    if arms.is_empty() {
        return None;
    }
    for arm in &task.arms {
        if matches!(arm.pattern, TaskPattern::Lit(_) | TaskPattern::Bind { .. }) {
            diags.push(diag(
                "source.task-message-type",
                "an implicit variant mailbox uses named arms and else",
                arm.span,
            ));
        }
    }
    Some(EnumItem {
        name: ident(&actor_message_name(&task.name.name), task.name.span),
        type_params: Vec::new(),
        instance: None,
        spec_witnesses: false,
        arms,
        methods: Vec::new(),
        span: task.span,
    })
}

fn build_guarded_dispatch(task: &TaskItem, message_type: &TypeAnn) -> Vec<Stmt> {
    use super::ast::{AggArmBody, AggMatchArm, MatchPattern};

    if let TypeRef::Record(message) = &message_type.kind {
        if message.name.starts_with(ACTOR_MESSAGE_PREFIX) {
            let span = task.span;
            let mut arms = Vec::new();
            for arm in &task.arms {
                let pat = match &arm.pattern {
                    TaskPattern::Named { name, bindings } => MatchPattern::Variant {
                        qual: Some(message.clone()),
                        name: name.clone(),
                        bindings: bindings.clone(),
                    },
                    TaskPattern::Wildcard(_) => MatchPattern::Else,
                    _ => continue, // mixed mailbox shapes were diagnosed during declaration binding
                };
                arms.push(AggMatchArm {
                    pat,
                    body: AggArmBody::Block(arm.body.clone()),
                    guard: arm.guard.clone(),
                    span: arm.span,
                });
            }
            if !arms
                .iter()
                .any(|arm| matches!(arm.pat, MatchPattern::Else) && arm.guard.is_none())
            {
                arms.push(AggMatchArm {
                    pat: MatchPattern::Else,
                    body: AggArmBody::Block(Block {
                        stmts: Vec::new(),
                        span,
                    }),
                    guard: None,
                    span,
                });
            }
            return vec![Stmt::AggIsMatch {
                scrutinee: lit_expr(ExprKind::Var(ident("__sched_msg", span)), span),
                arms,
                span,
            }];
        }
    }
    use super::ast::{AssignTarget, BinOp, UnaryOp};

    let span = task.span;
    let matched = ident("__sched_matched", span);
    let mut statements = vec![Stmt::Let {
        kw: span,
        name: matched.clone(),
        ann: None,
        mutable: true,
        value: lit_expr(ExprKind::BoolLit(false), span),
        span,
    }];
    for arm in &task.arms {
        let span = arm.span;
        let message = || lit_expr(ExprKind::Var(ident("__sched_msg", span)), span);
        let unmatched = lit_expr(
            ExprKind::Unary {
                op: UnaryOp::Not,
                operand: Box::new(lit_expr(ExprKind::Var(matched.clone()), span)),
            },
            span,
        );
        let mut then = Vec::new();
        let condition = match &arm.pattern {
            TaskPattern::Lit(lit) => {
                let expected = match (&message_type.kind, &lit.kind) {
                    (TypeRef::Builtin(TypeWord::I64), ExprKind::IntLit(n)) => lit_expr(
                        ExprKind::TypedLit {
                            ty: TypeWord::I64,
                            bits: *n as i64 as u64,
                        },
                        lit.span,
                    ),
                    _ => lit.clone(),
                };
                let comparison = lit_expr(
                    ExprKind::Binary {
                        op: BinOp::Eq,
                        lhs: Box::new(message()),
                        rhs: Box::new(expected),
                    },
                    lit.span,
                );
                lit_expr(
                    ExprKind::Binary {
                        op: BinOp::And,
                        lhs: Box::new(unmatched),
                        rhs: Box::new(comparison),
                    },
                    span,
                )
            }
            TaskPattern::Bind { name, ty } => {
                then.push(Stmt::Let {
                    kw: span,
                    name: name.clone(),
                    ann: Some(ty.clone()),
                    mutable: false,
                    value: message(),
                    span,
                });
                unmatched
            }
            TaskPattern::Wildcard(_) => unmatched,
            TaskPattern::Named { .. } => unreachable!("implicit variant dispatch is handled above"),
        };
        let mut body = arm.body.clone();
        body.stmts.insert(
            0,
            Stmt::Assign {
                target: AssignTarget {
                    base: matched.clone(),
                    path: Vec::new(),
                },
                value: lit_expr(ExprKind::BoolLit(true), span),
                span,
            },
        );
        if let Some(guard) = &arm.guard {
            then.push(Stmt::If {
                cond: guard.clone(),
                then: body,
                els: None,
                span,
            });
        } else {
            then.append(&mut body.stmts);
        }
        statements.push(Stmt::If {
            cond: condition,
            then: Block { stmts: then, span },
            els: None,
            span,
        });
    }
    statements
}

fn inner_span(s: &Stmt) -> Span {
    match s {
        // PLAN-1081：嵌套 fn 项（Auto 面；调度面不可达）。
        Stmt::NestedFn { item } => item.span,
        Stmt::If { span, .. }
        | Stmt::While { span, .. }
        | Stmt::Loop { span, .. }
        | Stmt::Let { span, .. }
        | Stmt::Assign { span, .. }
        | Stmt::Return { span, .. }
        | Stmt::ExprStmt { span, .. }
        | Stmt::LetDestruct { span, .. }
        | Stmt::Hold { span, .. }
        | Stmt::IsMatch { span, .. }
        | Stmt::AggIsMatch { span, .. }
        | Stmt::ForIn { span, .. }
        | Stmt::Try { span, .. }
        | Stmt::With { span, .. }
        | Stmt::HashIf { span, .. }
        | Stmt::HashFor { span, .. }
        | Stmt::HashIs { span, .. } => *span,
        // PLAN-1082：place 链赋值。
        Stmt::PlaceAssign(pa) => pa.span,
        Stmt::Break { span } | Stmt::Continue { span } | Stmt::Yield { span, .. } => *span,
        Stmt::IndexAssign(ia) => ia.span,
    }
}

fn word_name(w: TypeWord) -> &'static str {
    match w {
        TypeWord::I32 | TypeWord::Int => "int",
        TypeWord::Bool => "bool",
        TypeWord::StrOwned | TypeWord::Str => "Str",
        _ => "int",
    }
}

/// Rewrite the operation surface inside one function body, then keep the
/// closures-family result contract: a unit-result function (`fn main() { }`)
/// is retyped `int` with a synthesized trailing `return 0` (the canonical
/// sources' unit mains observe through stdout, never through an exit
/// value; 0 keeps the observable surface unchanged).
fn rewrite_fn(f: &mut FnItem, diags: &mut Vec<Diagnostic>) {
    let mut b = std::mem::replace(
        &mut f.body,
        Block {
            stmts: Vec::new(),
            span: Span { start: 0, end: 0 },
        },
    );
    rewrite_block(&mut b, diags);
    let unit_ret = matches!(&f.ret.kind, TypeRef::Builtin(TypeWord::Void));
    if unit_ret && !stmt_is_terminal(b.stmts.last()) {
        let span = b.span;
        b.stmts.push(Stmt::Return {
            value: lit_expr(ExprKind::IntLit(0), span),
            span,
        });
    }
    if unit_ret {
        f.ret = word_ann(TypeWord::I32, f.ret.span);
    }
    f.body = b;
}

/// Whether this statement never falls through (a return at the top level
/// of the block).
fn stmt_is_terminal(s: Option<&Stmt>) -> bool {
    matches!(s, Some(Stmt::Return { .. }))
}

fn rewrite_block(b: &mut Block, diags: &mut Vec<Diagnostic>) {
    for s in b.stmts.iter_mut() {
        rewrite_stmt(s, diags);
    }
}

fn rewrite_stmt(s: &mut Stmt, diags: &mut Vec<Diagnostic>) {
    match s {
        Stmt::ExprStmt { value, .. }
        | Stmt::Let { value, .. }
        | Stmt::Assign { value, .. }
        | Stmt::Return { value, .. } => rewrite_expr(value, diags),
        Stmt::If {
            cond, then, els, ..
        } => {
            rewrite_expr(cond, diags);
            rewrite_block(then, diags);
            if let Some(e) = els {
                rewrite_block(e, diags);
            }
        }
        Stmt::While { cond, body, .. } => {
            rewrite_expr(cond, diags);
            rewrite_block(body, diags);
        }
        Stmt::Loop { body, .. } | Stmt::Hold { body, .. } => rewrite_block(body, diags),
        Stmt::IndexAssign(ia) => rewrite_expr(&mut ia.value, diags),
        Stmt::LetDestruct { value, .. } => rewrite_expr(value, diags),
        _ => {}
    }
}

/// The operation rewrite. `Task.spawn` / `reply` / `sleep` / `await` / `go`
/// become intrinsic calls; `h.send`/`h.ask`/`h.stop` route the receiver
/// into the first argument.
fn rewrite_expr(e: &mut Expr, diags: &mut Vec<Diagnostic>) {
    match &mut e.kind {
        ExprKind::Await(inner) => {
            rewrite_expr(inner, diags);
            let inner_v = std::mem::replace(inner.as_mut(), lit_expr(ExprKind::IntLit(0), e.span));
            e.kind = ExprKind::Call {
                callee: ident("__sched_await", e.span),
                args: vec![Arg::Pos(inner_v)],
            };
        }
        ExprKind::Go(inner) => {
            rewrite_expr(inner, diags);
            let inner_v = std::mem::replace(inner.as_mut(), lit_expr(ExprKind::IntLit(0), e.span));
            match &inner_v.kind {
                ExprKind::Call { callee, args } => {
                    let mut new_args = vec![Arg::Pos(lit_expr(
                        ExprKind::StrLit(callee.name.clone()),
                        callee.span,
                    ))];
                    new_args.extend(args.iter().cloned());
                    e.kind = ExprKind::Call {
                        callee: ident("__sched_go", e.span),
                        args: new_args,
                    };
                }
                _ => {
                    diags.push(diag(
                        "source.task-go",
                        "`go` takes a direct function call (`go f(args)`)",
                        inner_v.span,
                    ));
                }
            }
        }
        ExprKind::MethodCall {
            recv, name, args, ..
        } => {
            rewrite_expr(recv, diags);
            for a in args.iter_mut() {
                if let Arg::Pos(v) = a {
                    rewrite_expr(v, diags);
                }
            }
            let is_task_recv = matches!(&recv.kind, ExprKind::Var(id) if id.name == "Task");
            match (is_task_recv, name.name.as_str()) {
                (true, "spawn") => {
                    let mut new_args: Vec<Arg> = Vec::new();
                    new_args.extend(args.iter().filter_map(|a| match a {
                        Arg::Pos(v) => Some(Arg::Pos(v.clone())),
                        _ => None,
                    }));
                    e.kind = ExprKind::Call {
                        callee: ident("__sched_spawn", e.span),
                        args: new_args,
                    };
                }
                (false, "send" | "ask" | "stop") => {
                    let target = format!("__sched_{}", name.name);
                    let mut new_args = vec![Arg::Pos(recv.as_ref().clone())];
                    new_args.extend(args.iter().filter_map(|a| match a {
                        Arg::Pos(v) => Some(Arg::Pos(v.clone())),
                        _ => None,
                    }));
                    e.kind = ExprKind::Call {
                        callee: ident(&target, e.span),
                        args: new_args,
                    };
                }
                _ => {}
            }
        }
        ExprKind::Call { callee, args } => {
            if callee.name == "reply" {
                callee.name = "__sched_reply".to_string();
            } else if callee.name == "sleep" {
                callee.name = "__sched_sleep".to_string();
            }
            for a in args.iter_mut() {
                if let Arg::Pos(v) = a {
                    rewrite_expr(v, diags);
                }
            }
        }
        ExprKind::Binary { lhs, rhs, .. } => {
            rewrite_expr(lhs, diags);
            rewrite_expr(rhs, diags);
        }
        ExprKind::Unary { operand, .. } => rewrite_expr(operand, diags),
        _ => {
            // Other composite kinds carry no scheduler surface today; the
            // resolver rejects stray usage sites by name.
        }
    }
}
