//! TypeScript Transpiler (Plan 100: a2js → a2ts)
//!
//! Transpiles AutoLang AST to TypeScript code with full type annotations.
//! 
//! Split across multiple modules for Plan 152:
//! - ts_types.rs: Type mapping
//! - ts_expr.rs: Expression transpilation
//! - ts_stmt.rs: Statement transpilation
//! - ts_runtime.rs: Stdlib runtime generation

use super::{Sink, Trans, ToStrError};
use crate::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use std::io::Write;

#[path = "ts_types.rs"]
pub mod ts_types;

#[path = "ts_expr.rs"]
pub mod ts_expr;

#[path = "ts_stmt.rs"]
pub mod ts_stmt;

#[path = "ts_runtime.rs"]
pub mod ts_runtime;

pub struct TypeScriptTrans {
    #[allow(dead_code)]
    name: AutoStr,
}

impl TypeScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
        }
    }
}

impl Trans for TypeScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 5: Inject runtime helpers first
        self.inject_runtime(&mut sink.body)?;
        sink.body.write(b"\n")?;

        // Find main function
        let main_func = ast.stmts.iter().find(|s| {
            if let Stmt::Fn(func) = s {
                func.name == "main"
            } else {
                false
            }
        }).cloned();

        // Split into declarations and main statements
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main_stmts: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            // Skip main function declaration - we'll handle it specially
            if let Stmt::Fn(func) = &stmt {
                if func.name == "main" {
                    continue;
                }
            }

            // Check if this is a declaration (type, enum, or function)
            if matches!(stmt, Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::Fn(_)) {
                decls.push(stmt);
            } else {
                main_stmts.push(stmt);
            }
        }

        // Generate declarations first
        for (i, decl) in decls.iter().enumerate() {
            self.stmt(decl, &mut sink.body)?;
            if i < decls.len() - 1 {
                sink.body.write(b"\n\n")?;
            }
        }

        // Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            // Output the main function
            if !decls.is_empty() {
                sink.body.write(b"\n\n")?;
            }
            self.stmt(&main_stmt, &mut sink.body)?;

            // Call main at the end
            sink.body.write(b"\n\nmain();\n")?;
        } else if !main_stmts.is_empty() {
            // Wrap statements in a main function
            if !decls.is_empty() {
                sink.body.write(b"\n\n")?;
            }
            sink.body.write(b"function main(): void {")?;

            for stmt in &main_stmts {
                sink.body.write(b"\n    ")?;
                self.stmt(stmt, &mut sink.body)?;
            }

            sink.body.write(b"\n}")?;

            // Call main at the end
            sink.body.write(b"\n\nmain();\n")?;
        }

        Ok(())
    }
}
