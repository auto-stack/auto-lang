//! Control Flow Analysis for "Last Use" detection
//!
//! This module provides analysis to detect the last use of a variable,
//! enabling automatic cleanup at the right time.

use std::collections::{HashMap, HashSet};
use crate::ast::{Name, Stmt, Expr, Store, For, If};

/// Unique identifier for expressions
pub type ExprId = usize;

/// Analyzer that detects the last use of each variable
pub struct LastUseAnalyzer {
    /// Map from variable name to set of expressions that are its last use
    pub last_uses: HashMap<Name, HashSet<ExprId>>,
    /// Counter for generating unique expression IDs
    next_id: ExprId,
    /// Current expression ID during analysis
    current_id: ExprId,
}

impl LastUseAnalyzer {
    /// Create a new last-use analyzer
    pub fn new() -> Self {
        Self {
            last_uses: HashMap::new(),
            next_id: 0,
            current_id: 0,
        }
    }

    /// Analyze a statement to detect last uses
    pub fn analyze(&mut self, stmt: &Stmt) {
        self.analyze_stmt(stmt);
    }

    /// Get the expression ID for the current position
    fn fresh_id(&mut self) -> ExprId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Analyze a statement
    fn analyze_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Store(store) => {
                // This is a let/mut/var binding or reassignment
                self.analyze_store(store);
            }
            Stmt::Expr(expr) => {
                // Analyze the expression
                self.analyze_expr(expr);
            }
            Stmt::For(for_stmt) => {
                // Analyze the for loop
                self.analyze_for(for_stmt);
            }
            Stmt::If(if_stmt) => {
                // Analyze the if statement
                self.analyze_if(if_stmt);
            }
            Stmt::Block(body) => {
                // Analyze the block
                for stmt in &body.stmts {
                    self.analyze_stmt(stmt);
                }
            }
            Stmt::Break => {
                // Control flow statements
            }
            _ => {
                // Other statements don't affect local variables for now
            }
        }
    }

    /// Analyze a store statement (let/mut/var)
    fn analyze_store(&mut self, store: &Store) {
        // Analyze the initialization expression
        self.analyze_expr(&store.expr);

        // The store creates a new binding or reassigns
        // For reassignment, the old value's last use is here
        // We'll track this in the evaluator
    }

    /// Analyze a for loop
    fn analyze_for(&mut self, for_stmt: &For) {
        // Analyze the range expression
        self.analyze_expr(&for_stmt.range);

        // The loop variable is created fresh each iteration
        // Analyze the body
        for stmt in &for_stmt.body.stmts {
            self.analyze_stmt(stmt);
        }
    }

    /// Analyze an if statement
    fn analyze_if(&mut self, if_stmt: &If) {
        // Analyze each branch
        for branch in &if_stmt.branches {
            self.analyze_expr(&branch.cond);
            for stmt in &branch.body.stmts {
                self.analyze_stmt(stmt);
            }
        }
    }

    /// Analyze an expression
    fn analyze_expr(&mut self, expr: &Expr) {
        self.current_id = self.fresh_id();
        self.analyze_expr_inner(expr);
    }

    /// Inner expression analysis
    fn analyze_expr_inner(&mut self, expr: &Expr) {
        match expr {
            Expr::Ref(name) => {
                // Variable reference - potentially a last use
                // For now, we conservatively mark every use as a potential last use
                self.mark_last_use(name);
            }
            Expr::Unary(_, inner_expr) => {
                self.analyze_expr(inner_expr);
            }
            Expr::Bina(left, _, right) => {
                self.analyze_expr(left);
                self.analyze_expr(right);
            }
            Expr::Index(base, index) => {
                self.analyze_expr(base);
                self.analyze_expr(index);
            }
            Expr::Call(call) => {
                self.analyze_expr(&call.name);
                // Analyze arguments (call.args is of type Args)
                // For now, skip detailed arg analysis
            }
            Expr::Array(elems) => {
                for elem in elems {
                    self.analyze_expr(elem);
                }
            }
            Expr::Object(props) => {
                for pair in props {
                    self.analyze_expr(&pair.value);
                }
            }
            Expr::Pair(pair) => {
                self.analyze_expr(&pair.value);
            }
            _ => {
                // Literals and other expressions don't need analysis
            }
        }
    }

    /// Mark a variable use as a last use
    fn mark_last_use(&mut self, name: &Name) {
        self.last_uses
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(self.current_id);
    }

    /// Check if an expression is a last use for a variable
    pub fn is_last_use(&self, name: &Name, expr_id: ExprId) -> bool {
        self.last_uses
            .get(name)
            .map(|uses| uses.contains(&expr_id))
            .unwrap_or(false)
    }

    /// Get all variables that have last uses tracked
    pub fn tracked_variables(&self) -> impl Iterator<Item = &Name> {
        self.last_uses.keys()
    }
}

impl Default for LastUseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyzer_creation() {
        let analyzer = LastUseAnalyzer::new();
        assert!(analyzer.last_uses.is_empty());
    }
}
