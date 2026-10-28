//! Exact type checking for the `auto.source.core-i32.draft` and
//! `auto.source.scalar-i32.draft` profiles (PLAN-745 T-03, PLAN-767 T-03).
//!
//! Runs over the resolver output (names resolved, value types inferred) and
//! enforces the profile's exact-type rules with stage `verify` /
//! `source.type-mismatch` diagnostics:
//!
//! - arithmetic (`+ - * / %`) take two i32 operands and produce i32,
//! - relational (`< > <= >=`) take two i32 operands and produce bool,
//! - `== !=` take two operands of the SAME type (both i32 or both bool)
//!   and produce bool — mixed types are rejected, no implicit conversion,
//! - `&& ||` take two bool operands and produce bool,
//! - unary `-` takes i32 and produces i32; unary `!` takes bool and
//!   produces bool,
//! - `if`/`while`/`for` conditions must be bool,
//! - `return` values must match the function result type,
//! - call arguments must match the bound formal's type (through the
//!   named-argument map, not by position),
//! - annotated `let`/`var` initializers must match the annotation,
//! - assignment values must match the target local's type.
//!
//! Control flow (all-paths return) and mutability are deliberately NOT
//! checked here: the emitted HIR goes through the existing semantic
//! verifier (`crate::verify`), whose conservative rules own those rejections.

use super::ast::{BinOp, UnaryOp};
use super::resolve::{
    param_position, RBlock, RExprKind, RStmt, ResolvedArg, ResolvedExpr, ResolvedFn,
    ResolvedProgram, VT,
};
use super::SourceProfile;
use crate::atom_text::{Diagnostic, Span, Stage};

pub fn typecheck(program: &ResolvedProgram) -> Result<(), Vec<Diagnostic>> {
    typecheck_with(program, SourceProfile::Core)
}

/// Typecheck with an explicit profile surface. The core surface cannot
/// reach scalar-only nodes (the parser rejected them), so the walk is
/// shared; only documentation of the checked rules differs.
pub fn typecheck_with(
    program: &ResolvedProgram,
    _surface: SourceProfile,
) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    for f in &program.fns {
        check_block(&f.body, program, f, &mut diags);
    }
    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}

/// PLAN-828 D-01: typecheck ONE function body against the retained
/// (grafted) environment — the same `check_block` walk the whole-module
/// pass runs per function; every other entry is consulted, never
/// re-walked.
pub fn check_body_in(program: &ResolvedProgram, fn_index: usize) -> Vec<Diagnostic> {
    let mut diags = Vec::new();
    let f = &program.fns[fn_index];
    check_block(&f.body, program, f, &mut diags);
    diags
}

fn mismatch(diags: &mut Vec<Diagnostic>, expected: VT, found: VT, span: Span) {
    diags.push(Diagnostic::new(
        Stage::Verify,
        "source.type-mismatch",
        format!("expected {}, found {}", expected.as_str(), found.as_str()),
        span,
    ));
}

fn check_block(
    root: &RBlock,
    program: &ResolvedProgram,
    f: &ResolvedFn,
    diags: &mut Vec<Diagnostic>,
) {
    crate::body_work::record_typecheck(&f.name);
    // Iterative walk (R6: block nesting is unlimited, so no recursion).
    // Frames advance one statement at a time; an if pushes its else-arm
    // frame before its then-arm frame (LIFO), so conditions, arms and
    // following statements are checked in exactly the previous recursive
    // order.
    struct TCFrame<'a> {
        src: &'a RBlock,
        idx: usize,
    }
    let mut frames: Vec<TCFrame> = vec![TCFrame { src: root, idx: 0 }];
    while let Some(frame) = frames.last_mut() {
        let next = {
            let src: &RBlock = frame.src;
            let n = src.stmts.get(frame.idx);
            if n.is_some() {
                frame.idx += 1;
            }
            n
        };
        let Some(s) = next else {
            frames.pop();
            continue;
        };
        match s {
            RStmt::Let { local, value, .. } => {
                check_expr(value, program, f, diags);
                // With an annotation the resolver pinned the local's type to
                // it; without one the local took the value's type, so this
                // compares annotation vs initializer exactly.
                let want = f.locals[local.0].vt;
                if value.vt != want {
                    mismatch(diags, want, value.vt, value.span);
                }
            }
            RStmt::Assign {
                local, path, value, ..
            } => {
                check_expr(value, program, f, diags);
                if local.0 != usize::MAX {
                    // the write target's type: the path's final field for
                    // field-path places, the local itself otherwise
                    let mut want = f.locals[local.0].vt;
                    for (ri, fi) in path {
                        want = program.records[*ri].fields[*fi].vt;
                    }
                    if value.vt != want {
                        mismatch(diags, want, value.vt, value.span);
                    }
                }
            }
            // PLAN-862: the script tail — its assertions live in resolve
            // (position + i32-only); here it is just one more expression
            // whose operators/arguments must typecheck.
            RStmt::ExprStmt { value, .. } => check_expr(value, program, f, diags),
            RStmt::Return { value, .. } => {
                check_expr(value, program, f, diags);
                if value.vt != f.result {
                    mismatch(diags, f.result, value.vt, value.span);
                }
            }
            RStmt::If {
                cond, then, els, ..
            } => {
                check_expr(cond, program, f, diags);
                if cond.vt != VT::Bool {
                    mismatch(diags, VT::Bool, cond.vt, cond.span);
                }
                if let Some(e) = els {
                    frames.push(TCFrame { src: e, idx: 0 });
                }
                frames.push(TCFrame { src: then, idx: 0 });
            }
            RStmt::While { cond, body, .. } => {
                check_expr(cond, program, f, diags);
                if cond.vt != VT::Bool {
                    mismatch(diags, VT::Bool, cond.vt, cond.span);
                }
                frames.push(TCFrame { src: body, idx: 0 });
            }
            RStmt::Loop { body, .. } => frames.push(TCFrame { src: body, idx: 0 }),
            RStmt::Break { .. } | RStmt::Continue { .. } => {}
        }
    }
}

/// Borrow-mode contract (PLAN-771 / 774 D-04): a record param declared
/// `mut` must receive a `p.mut` argument, and a `p.mut` argument must land
/// on a `mut` param — both directions are `source.type-mismatch` at the
/// call site (the HIR verifier re-checks on the emitted borrows).
fn check_arg_modes(
    fn_index: usize,
    first_visible: usize,
    args: &[ResolvedArg],
    program: &ResolvedProgram,
    diags: &mut Vec<Diagnostic>,
    span: Span,
) {
    let Some(callee) = program.fns.get(fn_index) else {
        return;
    };
    for a in args {
        let pos = match &a.name {
            None => args.iter().position(|x| std::ptr::eq(x, a)),
            Some(n) => callee.param_names.iter().position(|p| *p == n.name),
        };
        let Some(pos) = pos else { continue };
        let param_i = pos + first_visible;
        let param_mut = callee.param_modes.get(param_i).copied() == Some(true);
        let arg_is_mut = matches!(a.value.kind, RExprKind::BorrowMut { .. });
        let param_is_record = matches!(callee.params.get(param_i), Some(VT::Rec(_)));
        if param_is_record && param_mut && !arg_is_mut {
            diags.push(Diagnostic::new(
                Stage::Verify,
                "source.type-mismatch",
                format!(
                    "parameter `{}` of `{}` is declared `mut` and requires a `.mut` argument at the call site",
                    callee.param_names.get(param_i).unwrap_or(&"?".to_string()),
                    callee.name
                ),
                span,
            ));
        }
        if arg_is_mut && (!param_mut || !param_is_record) {
            diags.push(Diagnostic::new(
                Stage::Verify,
                "source.type-mismatch",
                format!(
                    "a `.mut` argument requires the parameter to be a `mut` record parameter of `{}`",
                    callee.name
                ),
                span,
            ));
        }
    }
}

fn check_expr(
    root: &ResolvedExpr,
    program: &ResolvedProgram,
    _f: &ResolvedFn,
    diags: &mut Vec<Diagnostic>,
) {
    // Iterative walk (R3-QA-02): legal operator chains are unbounded in
    // depth, so no recursion. Visitation order (operands left-to-right
    // before the node's own checks) matches the previous recursive form.
    let mut stack: Vec<&ResolvedExpr> = vec![root];
    while let Some(e) = stack.pop() {
        match &e.kind {
            RExprKind::IntLit(_)
            | RExprKind::BoolLit(_)
            | RExprKind::ReadLocal(_)
            | RExprKind::StrLit(_)
            | RExprKind::CharLit(_)
            | RExprKind::Method { .. }
            | RExprKind::Move { .. } => {}
            RExprKind::TypeWord(_) => {
                diags.push(Diagnostic::new(
                    Stage::Verify,
                    "source.type-mismatch",
                    "a type word is only allowed inside `.to(...)` or as the `char.from_codepoint` receiver",
                    e.span,
                ));
            }
            RExprKind::CallIntrinsic { .. } => {}
            RExprKind::CallFn { fn_index, args } => {
                let callee = program.fns.get(*fn_index);
                for (idx, a) in args.iter().enumerate() {
                    stack.push(&a.value);
                    if let Some(c) = callee {
                        if let Some(pos) = param_position(idx, a, c) {
                            let want = c.params[pos];
                            if a.value.vt != want {
                                mismatch(diags, want, a.value.vt, a.value.span);
                            }
                        }
                    }
                }
                check_arg_modes(*fn_index, 0, args, program, diags, e.span);
            }
            RExprKind::Binary { op, lhs, rhs, .. } => {
                // push rhs first so lhs is checked first (LIFO)
                stack.push(rhs.as_ref());
                stack.push(lhs.as_ref());
                match op {
                    BinOp::Add => {
                        // PLAN-770: text + text is concatenation; i32 + i32
                        // stays arithmetic. Any mixed shape is a mismatch.
                        let both_text = lhs.vt.is_text() && rhs.vt.is_text();
                        let both_i32 = lhs.vt == VT::I32 && rhs.vt == VT::I32;
                        if !both_text && !both_i32 {
                            mismatch(diags, VT::I32, lhs.vt, lhs.span);
                            mismatch(diags, VT::I32, rhs.vt, rhs.span);
                        }
                    }
                    BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Rem => {
                        for (role, v) in [("lhs", lhs), ("rhs", rhs)] {
                            if v.vt != VT::I32 {
                                mismatch(diags, VT::I32, v.vt, v.span);
                                let _ = role;
                            }
                        }
                    }
                    BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                        for v in [lhs.as_ref(), rhs.as_ref()] {
                            if v.vt != VT::I32 {
                                mismatch(diags, VT::I32, v.vt, v.span);
                            }
                        }
                    }
                    BinOp::Eq | BinOp::Ne => {
                        if lhs.vt != rhs.vt {
                            diags.push(Diagnostic::new(
                                Stage::Verify,
                                "source.type-mismatch",
                                format!(
                                    "both operands of `{}` must have the same type ({} vs {})",
                                    op.as_str(),
                                    lhs.vt.as_str(),
                                    rhs.vt.as_str()
                                ),
                                lhs.span.join(rhs.span),
                            ));
                        }
                    }
                    BinOp::And | BinOp::Or => {
                        for v in [lhs.as_ref(), rhs.as_ref()] {
                            if v.vt != VT::Bool {
                                mismatch(diags, VT::Bool, v.vt, v.span);
                            }
                        }
                    }
                }
            }
            // -- record profile (PLAN-771): typing was resolved inline in
            // the resolve stage (nominal ctor fields, per-segment field
            // paths, method sigs); here the borrow-mode contract is
            // enforced (a `mut` record param requires its `p.mut` argument;
            // a `p.mut` argument requires a `mut` param) and children walk.
            RExprKind::CtorPos { args, .. } => {
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::CtorNamed { fields, .. } => {
                for (_, v) in fields {
                    stack.push(v);
                }
            }
            RExprKind::FieldGet { base, .. } => {
                stack.push(base.as_ref());
            }
            RExprKind::BorrowMut { .. } => {}
            // PLAN-782: `r.clone()` carries the receiver's record type;
            // nominal equality was already established at dispatch.
            RExprKind::RecordClone { .. } => {}
            RExprKind::MethodCall {
                fn_index,
                recv,
                args,
            } => {
                if let Some(r) = recv {
                    stack.push(r.as_ref());
                }
                check_arg_modes(
                    *fn_index,
                    if recv.is_some() { 1 } else { 0 },
                    args,
                    program,
                    diags,
                    e.span,
                );
                for a in args {
                    stack.push(&a.value);
                }
            }
            RExprKind::Unary { op, operand } => {
                stack.push(operand.as_ref());
                match op {
                    UnaryOp::Neg => {
                        if operand.vt != VT::I32 {
                            mismatch(diags, VT::I32, operand.vt, operand.span);
                        }
                    }
                    UnaryOp::Not => {
                        if operand.vt != VT::Bool {
                            mismatch(diags, VT::Bool, operand.vt, operand.span);
                        }
                    }
                }
            }
        }
    }
}
