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
    /// Runtime import path (e.g., "./runtime" or "../stdlib/runtime")
    pub runtime_path: String,
    /// Track which runtime symbols are needed
    pub needs_range: bool,
    pub needs_print: bool,
}

impl TypeScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
            runtime_path: "./runtime".to_string(),
            needs_range: false,
            needs_print: false,
        }
    }

    /// Set the runtime import path
    pub fn with_runtime_path(mut self, path: impl Into<String>) -> Self {
        self.runtime_path = path.into();
        self
    }
}

impl Trans for TypeScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Phase 1: Transpile AST into a buffer (this sets needs_range, needs_print)
        let mut body_buf: Vec<u8> = Vec::new();

        // Find main function
        let main_func = ast.stmts.iter().find(|s| {
            matches!(s, Stmt::Fn(func) if func.name == "main")
        }).cloned();

        // Split into declarations and main statements
        let mut decls: Vec<Stmt> = Vec::new();
        let mut main_stmts: Vec<Stmt> = Vec::new();

        for stmt in ast.stmts.into_iter() {
            // Skip main function declaration - we'll handle it specially
            if matches!(&stmt, Stmt::Fn(func) if func.name == "main") {
                continue;
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
            self.stmt(decl, &mut body_buf)?;
            if i < decls.len() - 1 {
                body_buf.write(b"\n\n")?;
            }
        }

        // Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            // Output the main function
            if !decls.is_empty() {
                body_buf.write(b"\n\n")?;
            }
            self.stmt(&main_stmt, &mut body_buf)?;

            // Call main at the end
            body_buf.write(b"\n\nmain();\n")?;
        } else if !main_stmts.is_empty() {
            // Wrap statements in a main function
            if !decls.is_empty() {
                body_buf.write(b"\n\n")?;
            }
            body_buf.write(b"function main(): void {")?;

            for stmt in &main_stmts {
                body_buf.write(b"\n    ")?;
                self.stmt(stmt, &mut body_buf)?;
            }

            body_buf.write(b"\n}")?;

            // Call main at the end
            body_buf.write(b"\n\nmain();\n")?;
        }

        // Phase 2: Write conditional runtime import based on what was used
        self.inject_runtime_import(&mut sink.body)?;
        if self.needs_range || self.needs_print {
            sink.body.write(b"\n")?;
        }

        // Phase 3: Append the transpiled body
        sink.body.write_all(&body_buf)?;

        Ok(())
    }
}
