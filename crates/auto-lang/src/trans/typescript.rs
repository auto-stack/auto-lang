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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn test_a2ts(case: &str) -> AutoResult<()> {
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2ts/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = std::fs::read_to_string(src_path.as_path())?;

        let _scope = crate::scope_manager::ScopeManager::new();
        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = TypeScriptTrans::new(name.into());
        trans.trans(ast, &mut sink)?;
        let ts_code = sink.done()?;

        let expected_path = format!("test/a2ts/{}/{}.expected.ts", case, name);
        let expected_path = d.join(expected_path);
        let expected = std::fs::read_to_string(expected_path.as_path())?;

        let ts_string = String::from_utf8_lossy(&ts_code);
        if ts_string != expected {
            let wrong_path = format!("test/a2ts/{}/{}.wrong.ts", case, name);
            let wrong_path = d.join(wrong_path);
            std::fs::write(&wrong_path, ts_code)?;
            panic!("Output differs from expected. Check {}.wrong.ts", name);
        }

        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2ts("000_hello").unwrap();
    }

    #[test]
    fn test_003_func() {
        test_a2ts("003_func").unwrap();
    }

    #[test]
    fn test_006_struct() {
        test_a2ts("006_struct").unwrap();
    }

    #[test]
    fn test_007_enum() {
        test_a2ts("007_enum").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2ts("010_if").unwrap();
    }

    #[test]
    fn test_011_for() {
        test_a2ts("011_for").unwrap();
    }

    #[test]
    fn test_013_while() {
        test_a2ts("013_while").unwrap();
    }

    #[test]
    fn test_015_nested_if() {
        test_a2ts("015_nested_if").unwrap();
    }

    #[test]
    fn test_017_loop() {
        test_a2ts("017_loop").unwrap();
    }

    #[test]
    fn test_018_for_each() {
        test_a2ts("018_for_each").unwrap();
    }

    #[test]
    fn test_014_closure() {
        test_a2ts("014_closure").unwrap();
    }

    #[test]
    fn test_019_blocks() {
        test_a2ts("019_blocks").unwrap();
    }

    #[test]
    fn test_008_method() {
        test_a2ts("008_method").unwrap();
    }

    #[test]
    fn test_009_alias() {
        test_a2ts("009_alias").unwrap();
    }

    #[test]
    fn test_013_union() {
        test_a2ts("013_union").unwrap();
    }

    #[test]
    fn test_014_tag() {
        test_a2ts("014_tag").unwrap();
    }

    #[test]
    fn test_017_struct_methods() {
        test_a2ts("017_struct_methods").unwrap();
    }

    #[test]
    fn test_028_object() {
        test_a2ts("028_object").unwrap();
    }

    #[test]
    fn test_029_composition() {
        test_a2ts("029_composition").unwrap();
    }

    #[test]
    fn test_016_basic_spec() {
        test_a2ts("016_basic_spec").unwrap();
    }

    #[test]
    fn test_017_spec() {
        test_a2ts("017_spec").unwrap();
    }

    #[test]
    fn test_030_range_expr() {
        test_a2ts("030_range_expr").unwrap();
    }
}
