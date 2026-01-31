// =============================================================================
// Dependency Scanner: Build fragment-level dependency graphs
// =============================================================================
//
// The dependency scanner walks function bodies to find:
// 1. Function calls (Expr::Call)
// 2. Type usages
// 3. Builds fine-grained dependency graph for intelligent recompilation
//
// Phase 3.3: Fragment-level dependency tracking

use crate::ast::{Body, Expr, Fn, Stmt};
use crate::database::{Database, FragId};
use auto_val::AutoStr;
use std::collections::HashSet;

/// Dependency scanner for building fragment-level dependency graphs
///
/// Scans function bodies to find what other functions/types they depend on.
pub struct DepScanner<'db> {
    db: &'db Database,
}

impl<'db> DepScanner<'db> {
    /// Create a new dependency scanner
    pub fn new(db: &'db Database) -> Self {
        Self { db }
    }

    /// Scan a function and extract its fragment dependencies
    ///
    /// Returns a list of FragIds that this function depends on (calls/uses).
    pub fn scan_fn(&self, fn_decl: &Fn) -> Vec<FragId> {
        let mut deps = HashSet::new();

        // Walk the function body to find calls
        self.walk_body(&fn_decl.body, &mut deps);

        deps.into_iter().collect()
    }

    /// Walk a body statement list to find dependencies
    fn walk_body(&self, body: &Body, deps: &mut HashSet<FragId>) {
        for stmt in &body.stmts {
            self.walk_stmt(stmt, deps);
        }
    }

    /// Walk a statement to find dependencies
    fn walk_stmt(&self, stmt: &Stmt, deps: &mut HashSet<FragId>) {
        match stmt {
            // Expression statements - check for calls
            Stmt::Expr(expr) => self.walk_expr(expr, deps),

            // Store declarations - check type usage
            Stmt::Store(store) => {
                // Check the type (Phase 3.3: could track type dependencies)
                let _ = &store.ty;
                // Walk initializer expression
                self.walk_expr(&store.expr, deps);
            }

            // Return statements - check the return expression
            Stmt::Return(ret) => {
                self.walk_expr(ret, deps);
            }

            // If statements - walk branches
            Stmt::If(if_) => {
                for branch in &if_.branches {
                    for stmt in &branch.body.stmts {
                        self.walk_stmt(stmt, deps);
                    }
                }
                if let Some(else_body) = &if_.else_ {
                    for stmt in &else_body.stmts {
                        self.walk_stmt(stmt, deps);
                    }
                }
            }

            // Loops - walk body
            Stmt::For(loop_) => {
                for stmt in &loop_.body.stmts {
                    self.walk_stmt(stmt, deps);
                }
            }

            // Block statements - walk inner statements
            Stmt::Block(block) => {
                for stmt in &block.stmts {
                    self.walk_stmt(stmt, deps);
                }
            }

            // Use statements - handled at file level by indexer
            Stmt::Use(_) => {
                // File-level dependencies, not fragment-level
            }

            // Other statements - skip for now
            Stmt::Break => {}
            Stmt::Is(_) => {}
            Stmt::Fn(_) => {}
            Stmt::Union(_) => {}
            Stmt::Tag(_) => {}
            Stmt::SpecDecl(_) => {}
            Stmt::Ext(_) => {}
            Stmt::TypeDecl(_) => {}
            Stmt::EnumDecl(_) => {}
            Stmt::OnEvents(_) => {}
            Stmt::Node(_) => {}
            Stmt::Comment(_) => {}
            Stmt::Alias(_) => {}
            Stmt::TypeAlias(_) => {}
            Stmt::EmptyLine(_) => {}
        }
    }

    /// Walk an expression to find function calls
    fn walk_expr(&self, expr: &Expr, deps: &mut HashSet<FragId>) {
        match expr {
            // Function calls - RESOLVE DEPENDENCY
            Expr::Call(call) => {
                if let Some(callee) = self.resolve_call(&call.name) {
                    deps.insert(callee);
                }

                // Walk arguments recursively (they might contain nested calls)
                for arg in &call.args.args {
                    self.walk_expr(&arg.get_expr(), deps);
                }
            }

            // Binary operations - walk both sides
            Expr::Bina(left, _op, right) => {
                self.walk_expr(left, deps);
                self.walk_expr(right, deps);
            }

            // Unary operations - walk operand
            Expr::Unary(_op, operand) => {
                self.walk_expr(operand, deps);
            }

            // Index operations - walk array and index
            Expr::Index(array, index) => {
                self.walk_expr(array, deps);
                self.walk_expr(index, deps);
            }

            // If expressions - walk branches
            Expr::If(if_) => {
                for branch in &if_.branches {
                    for stmt in &branch.body.stmts {
                        self.walk_stmt(stmt, deps);
                    }
                }
                if let Some(else_body) = &if_.else_ {
                    for stmt in &else_body.stmts {
                        self.walk_stmt(stmt, deps);
                    }
                }
            }

            // Block expressions - walk statements
            Expr::Block(block) => {
                for stmt in &block.stmts {
                    self.walk_stmt(stmt, deps);
                }
            }

            // Array literals - walk elements
            Expr::Array(elems) => {
                for elem in elems {
                    self.walk_expr(elem, deps);
                }
            }

            // Hold expressions - walk held expression
            Expr::Hold(hold) => {
                self.walk_expr(&hold.path, deps);
            }

            // Take expressions - walk taken expression
            Expr::Take(expr) => {
                self.walk_expr(expr, deps);
            }

            // Other expressions - no dependencies
            Expr::Int(_)
            | Expr::Uint(_)
            | Expr::I8(_)
            | Expr::U8(_)
            | Expr::I64(_)
            | Expr::Byte(_)
            | Expr::Float(_, _)
            | Expr::Double(_, _)
            | Expr::Bool(_)
            | Expr::Char(_)
            | Expr::Ident(_)
            | Expr::Str(_)
            | Expr::CStr(_)
            | Expr::Nil => {}
            | Expr::Lambda(_) => {}  // TODO: Phase 3.3 - track closure captures
            | Expr::Closure(_) => {}  // TODO: Phase 3.3 - track closure captures
            | Expr::Pair(_) => {}
            | Expr::Object(pairs) => {
                for pair in pairs {
                    self.walk_expr(&pair.value, deps);
                }
            }
            | Expr::Dot(_, _) => {}
            | Expr::Node(_) => {}
            | Expr::FStr(_) => {}
            | Expr::Grid(_) => {}
            | Expr::Cover(_) => {}
            | Expr::Uncover(_) => {}
            | Expr::GenName(_) => {}
            | Expr::View(_) => {}
            | Expr::Mut(_) => {}
            | Expr::Range(_) => {}
            | Expr::Ref(_) => {}
            | Expr::Null => {}
            | Expr::NullCoalesce(_, _) => {}
            | Expr::ErrorPropagate(_) => {}
        }
    }

    /// Resolve a function call expression to a FragId
    ///
    /// Given a function name expression (e.g., Ident("foo")), find the
    /// corresponding FragId in the database.
    fn resolve_call(&self, name_expr: &Expr) -> Option<FragId> {
        // Extract the function name
        let name = match name_expr {
            Expr::Ident(ident) => ident,
            // Phase 3.3: Support indirect calls (e.g., through variables)
            // For now, only direct identifier calls are supported
            _ => return None,
        };

        // Search all fragments for matching name
        // Note: This is O(n) - Phase 3.3 could add name -> FragId index
        for file_id in self.db.get_files() {
            for frag_id in self.db.get_fragments_in_file(file_id) {
                if let Some(meta) = self.db.get_fragment_meta(&frag_id) {
                    if matches!(meta.kind, crate::database::FragKind::Function) {
                        if meta.name.as_ref() == name.as_str() {
                            return Some(frag_id);
                        }
                    }
                }
            }
        }

        None
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Body, FnKind, Type};
    use crate::database::Database;

    #[test]
    fn test_scanner_no_calls() {
        let db = Database::new();
        let scanner = DepScanner::new(&db);

        // Function with no calls
        let fn_decl = Fn::new(
            FnKind::Function,
            AutoStr::from("standalone"),
            None,
            vec![],
            Body::new(),
            Type::Int,
        );

        let deps = scanner.scan_fn(&fn_decl);
        assert_eq!(deps.len(), 0, "Function with no calls should have no dependencies");
    }
}
