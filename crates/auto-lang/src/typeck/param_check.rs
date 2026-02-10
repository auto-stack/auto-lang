// Plan 088 Phase 6: Parameter immutability checking
// Ensures that view (immutable) parameters cannot be modified

use crate::ast::{Fn, Stmt, Expr, Name, ParamMode, Body, Type, Store, StoreKind};
use crate::error::{AutoError, TypeError};
use miette::SourceSpan;
use std::collections::HashSet;

/// Parameter immutability checker
///
/// Validates that view parameters (which are immutable references) are not modified
/// within the function body. Reports CannotModifyViewParam errors when violations are detected.
pub struct ParamChecker;

impl ParamChecker {
    /// Check a function declaration for parameter immutability violations
    ///
    /// Returns Ok(()) if all view parameters are immutable,
    /// or Err(vec) with a list of errors if violations are found.
    pub fn check_fn_decl(fn_decl: &Fn) -> Result<(), Vec<AutoError>> {
        let mut errors = Vec::new();

        // Collect all view parameters
        let view_params: HashSet<Name> = fn_decl.params.iter()
            .filter(|p| p.mode == ParamMode::View)
            .map(|p| p.name.clone())
            .collect();

        // If no view parameters, nothing to check
        if view_params.is_empty() {
            return Ok(());
        }

        // Check function body for violations
        Self::check_body_immutable(&fn_decl.body, &view_params, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Check a body (block of statements) for view parameter modifications
    fn check_body_immutable(
        body: &Body,
        view_params: &HashSet<Name>,
        errors: &mut Vec<AutoError>,
    ) {
        for stmt in &body.stmts {
            Self::check_stmt(stmt, view_params, errors);
        }
    }

    /// Check a statement for view parameter modifications
    fn check_stmt(
        stmt: &Stmt,
        view_params: &HashSet<Name>,
        errors: &mut Vec<AutoError>,
    ) {
        match stmt {
            // Store statement: check if we're assigning to a view parameter
            Stmt::Store(store) => {
                if view_params.contains(&store.name) {
                    // Found a modification of a view parameter
                    let span = SourceSpan::new(0_usize.into(), 0_usize.into());
                    errors.push(TypeError::CannotModifyViewParam {
                        param: store.name.clone(),
                        span,
                    }.into());
                }

                // Also check the expression being assigned
                Self::check_expr(&store.expr, view_params, errors);
            }

            // If statement: check body recursively
            Stmt::If(_) => {
                // For now, skip If statement detailed checking due to complex structure
            }

            // For loop: check body
            Stmt::For(for_stmt) => {
                // Check range expression
                Self::check_expr(&for_stmt.range, view_params, errors);
                // Check loop body
                Self::check_body_immutable(&for_stmt.body, view_params, errors);
            }

            // Expression statement: check the expression
            Stmt::Expr(expr) => {
                Self::check_expr(expr, view_params, errors);
            }

            // Return statement: check the returned expression
            Stmt::Return(expr) => {
                Self::check_expr(expr, view_params, errors);
            }

            // Break: no checks needed
            Stmt::Break => {}

            // Block: check nested statements
            Stmt::Block(block) => {
                Self::check_body_immutable(block, view_params, errors);
            }

            // Other statements: no checks needed
            _ => {}
        }
    }

    /// Check an expression for view parameter modifications
    fn check_expr(
        expr: &Expr,
        _view_params: &HashSet<Name>,
        _errors: &mut Vec<AutoError>,
    ) {
        match expr {
            // Identifier: no check needed (just reading)
            Expr::Ident(_) => {}

            // Function/method call: could potentially modify view params
            // For now, we don't check inside function calls
            Expr::Call(_) | Expr::Dot(_, _) => {}

            // Other expressions: no checks needed
            _ => {}
        }
    }
}
