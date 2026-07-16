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
use std::collections::HashSet;
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
    /// Current indentation level for block bodies
    indent: usize,
    /// Names of scalar (C-style) enums, used to emit correct patterns.
    scalar_enums: HashSet<AutoStr>,
    /// Counter for generating unique temporary variable names in `is` statements.
    is_counter: usize,
}

impl TypeScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
            runtime_path: "./runtime".to_string(),
            needs_range: false,
            needs_print: false,
            indent: 0,
            scalar_enums: HashSet::new(),
            is_counter: 0,
        }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        if self.indent > 0 {
            self.indent -= 1;
        }
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ")?;
        }
        Ok(())
    }

    /// Write an opening brace and increase indentation for the block body.
    fn open_block(&mut self, out: &mut impl Write) -> AutoResult<()> {
        out.write(b" {")?;
        self.indent();
        Ok(())
    }

    /// Close the current block at the current indentation level.
    fn close_block(&mut self, out: &mut impl Write) -> AutoResult<()> {
        self.dedent();
        out.write(b"\n")?;
        self.print_indent(out)?;
        out.write(b"}")?;
        Ok(())
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

        // Split into declarations and main statements, preserving source line info
        let mut decls: Vec<(Stmt, usize)> = Vec::new(); // (stmt, source_line)
        let mut main_stmts: Vec<(Stmt, usize)> = Vec::new();  // (stmt, source_line)

        let source_lines = ast.source_lines;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Skip main function declaration - we'll handle it specially
            if matches!(&stmt, Stmt::Fn(func) if func.name == "main") {
                continue;
            }

            // Check if this is a declaration (type, enum, or function)
            if matches!(stmt, Stmt::TypeDecl(_) | Stmt::EnumDecl(_) | Stmt::Fn(_)) {
                decls.push((stmt, line));
            } else {
                main_stmts.push((stmt, line));
            }
        }

        // Generate declarations first
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.set_source_line(*line);
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
            body_buf.write(b"function main(): void")?;
            self.open_block(&mut body_buf)?;

            for (stmt, line) in &main_stmts {
                sink.set_source_line(*line);
                body_buf.write(b"\n")?;
                self.print_indent(&mut body_buf)?;
                self.stmt(stmt, &mut body_buf)?;
            }

            self.close_block(&mut body_buf)?;

            // Call main at the end
            body_buf.write(b"\n\nmain();\n")?;
        }

        // Phase 2: Write conditional runtime import based on what was used
        sink.clear_source_line();
        self.inject_runtime_import(&mut sink.body)?;
        if self.needs_range || self.needs_print {
            sink.body.write(b"\n")?;
        }

        // Phase 3: Append the transpiled body
        // Track newlines from body_buf through sink for source mapping
        sink.track_newlines(&body_buf);
        sink.body.write_all(&body_buf)?;

        Ok(())
    }
}
