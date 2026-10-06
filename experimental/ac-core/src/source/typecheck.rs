//! Exact type checking for the `auto.source.core-i32.draft` profile
//! (PLAN-745 T-03).
//!
//! Runs over the resolver output (names resolved, value types inferred) and
//! enforces the profile's exact-type rules with stage `verify` /
//! `source.type-mismatch` diagnostics:
//!
//! - `+`/`*` take two i32 operands and produce i32; `<` takes two i32
//!   operands and produces bool (no implicit bool/int conversion),
//! - `if`/`while` conditions must be bool,
//! - `return` values must match the function result type,
//! - call arguments must match the bound formal's type (through the
//!   named-argument map, not by position),
//! - annotated `let`/`var` initializers must match the annotation,
//! - assignment values must match the target local's type.
//!
//! Control flow (all-paths return) and mutability are deliberately NOT
//! checked here: the emitted HIR goes through the existing semantic
//! verifier (`crate::verify`), whose conservative rules own those rejections.

use super::resolve::{
    param_position, RBlock, RExprKind, RStmt, ResolvedExpr, ResolvedFn, ResolvedProgram, VT,
};
use crate::atom_text::{Diagnostic, Span, Stage};

pub fn typecheck(program: &ResolvedProgram) -> Result<(), Vec<Diagnostic>> {
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

fn mismatch(diags: &mut Vec<Diagnostic>, expected: VT, found: VT, span: Span) {
    diags.push(Diagnostic::new(
        Stage::Verify,
        "source.type-mismatch",
        format!("expected {}, found {}", expected.as_str(), found.as_str()),
        span,
    ));
}

fn check_block(b: &RBlock, program: &ResolvedProgram, f: &ResolvedFn, diags: &mut Vec<Diagnostic>) {
    for s in &b.stmts {
        check_stmt(s, program, f, diags);
    }
}

fn check_stmt(s: &RStmt, program: &ResolvedProgram, f: &ResolvedFn, diags: &mut Vec<Diagnostic>) {
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
        RStmt::Assign { local, value, .. } => {
            check_expr(value, program, f, diags);
            if local.0 != usize::MAX {
                let want = f.locals[local.0].vt;
                if value.vt != want {
                    mismatch(diags, want, value.vt, value.span);
                }
            }
        }
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
            check_block(then, program, f, diags);
            if let Some(e) = els {
                check_block(e, program, f, diags);
            }
        }
        RStmt::While { cond, body, .. } => {
            check_expr(cond, program, f, diags);
            if cond.vt != VT::Bool {
                mismatch(diags, VT::Bool, cond.vt, cond.span);
            }
            check_block(body, program, f, diags);
        }
        RStmt::Loop { body, .. } => check_block(body, program, f, diags),
        RStmt::Break { .. } | RStmt::Continue { .. } => {}
    }
}

fn check_expr(
    e: &ResolvedExpr,
    program: &ResolvedProgram,
    f: &ResolvedFn,
    diags: &mut Vec<Diagnostic>,
) {
    match &e.kind {
        RExprKind::IntLit(_) | RExprKind::BoolLit(_) | RExprKind::ReadLocal(_) => {}
        RExprKind::CallIntrinsic { .. } => {}
        RExprKind::CallFn { fn_index, args } => {
            let callee = program.fns.get(*fn_index);
            for (idx, a) in args.iter().enumerate() {
                check_expr(&a.value, program, f, diags);
                if let Some(c) = callee {
                    if let Some(pos) = param_position(idx, a, c) {
                        let want = c.params[pos];
                        if a.value.vt != want {
                            mismatch(diags, want, a.value.vt, a.value.span);
                        }
                    }
                }
            }
        }
        RExprKind::Binary { lhs, rhs, .. } => {
            check_expr(lhs, program, f, diags);
            check_expr(rhs, program, f, diags);
            if lhs.vt != VT::I32 {
                mismatch(diags, VT::I32, lhs.vt, lhs.span);
            }
            if rhs.vt != VT::I32 {
                mismatch(diags, VT::I32, rhs.vt, rhs.span);
            }
        }
    }
}
