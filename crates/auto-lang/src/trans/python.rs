use super::{Sink, Trans, ToStrError};
use crate::ast::*;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::{shared, Shared};
use std::collections::HashSet;
use std::io::Write;

pub struct PythonTrans {
    indent: usize,
    imports: HashSet<AutoStr>,
    name: AutoStr,
    scope: Shared<Universe>,
}

impl PythonTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            indent: 0,
            imports: HashSet::new(),
            name,
            scope: shared(Universe::default()),
        }
    }

    pub fn set_scope(&mut self, scope: Shared<Universe>) {
        self.scope = scope;
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            out.write(b"    ")?;
        }
        Ok(())
    }

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::Float(f, _) => write!(out, "{}", f).map_err(Into::into),
            Expr::Double(d, _) => write!(out, "{}", d).map_err(Into::into),
            Expr::Bool(b) => write!(out, "{}", b).map_err(Into::into),
            Expr::Char(c) => write!(out, "'{}'", c).map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", s).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),

            // F-strings (direct mapping - AutoLang and Python have identical syntax!)
            Expr::FStr(fstr) => self.fstr(fstr, out),

            // Identifiers
            Expr::Ident(name) => out.write_all(name.as_bytes()).map_err(Into::into),

            // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                self.expr(lhs, out)?;
                out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                self.expr(rhs, out)
            }

            // Unary operations
            Expr::Unary(op, expr) => {
                out.write(format!("{}", op.op()).as_bytes()).to()?;
                self.expr(expr, out)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // Arrays
            Expr::Array(elems) => self.array(elems, out),

            // Index
            Expr::Index(arr, idx) => self.index(arr, idx, out),

            // Block
            Expr::Block(block) => {
                out.write(b"{\n")?;
                self.indent();
                for stmt in &block.stmts {
                    self.stmt(stmt, out)?;
                }
                self.dedent();
                self.print_indent(out)?;
                out.write(b"}").to()
            }

            // Unsupported - return error for now
            _ => Err(format!("Python Transpiler: unsupported expression: {:?}", expr).into()),
        }
    }

    fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> AutoResult<bool> {
        match stmt {
            // Expression statements
            Stmt::Expr(expr) => {
                self.print_indent(out)?;
                self.expr(expr, out)?;
                out.write(b"\n")?;
                Ok(true)
            }

            // Store (variable assignment)
            Stmt::Store(store) => {
                self.print_indent(out)?;
                self.store(store, out)?;
                out.write(b"\n")?;
                Ok(true)
            }

            // Function declarations
            Stmt::Fn(func) => {
                self.fn_decl(func, out)?;
                Ok(true)
            }

            // If statements
            Stmt::If(if_stmt) => {
                self.if_stmt(if_stmt, out)?;
                Ok(true)
            }

            // For loops
            Stmt::For(for_loop) => {
                self.for_loop(for_loop, out)?;
                Ok(true)
            }

            // Pattern matching (is statement)
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, out)?;
                Ok(true)
            }

            // Break statement
            Stmt::Break => {
                self.print_indent(out)?;
                out.write(b"break\n")?;
                Ok(true)
            }

            // Empty lines (for formatting)
            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    out.write(b"\n")?;
                }
                Ok(true)
            }

            // Type declarations (structs)
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, out)?;
                Ok(true)
            }

            // Enum declarations
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, out)?;
                Ok(true)
            }

            // Skip alias, use, union, tag for now
            Stmt::Alias(_) => Ok(false),
            Stmt::Use(_) => Ok(false),
            Stmt::Union(_) => Ok(false),
            Stmt::Tag(_) => Ok(false),

            _ => Err(format!("Python Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // For now, just simple assignment without type
        out.write_all(store.name.as_bytes())?;
        out.write(b" = ")?;
        self.expr(&store.expr, out)?;
        Ok(())
    }

    fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"def ")?;
        out.write_all(func.name.as_bytes())?;
        out.write(b"(")?;

        // Parameters (without types for now)
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
        }

        out.write(b"):\n")?;
        self.indent();

        // Check if function has a non-void return type (except main)
        let has_return = !matches!(func.ret, Type::Unknown) && func.name != "main";

        // Process body statements
        if has_return && !func.body.stmts.is_empty() {
            // Handle all but last statement normally
            for stmt in func.body.stmts.iter().take(func.body.stmts.len() - 1) {
                self.stmt(stmt, out)?;
            }

            // Add return before last statement if it's an expression
            if let Some(last_stmt) = func.body.stmts.last() {
                if let Stmt::Expr(expr) = last_stmt {
                    self.print_indent(out)?;
                    out.write(b"return ")?;
                    self.expr(expr, out)?;
                    out.write(b"\n")?;
                } else {
                    // Last statement is not an expression, just process it normally
                    self.stmt(last_stmt, out)?;
                }
            }
        } else {
            // No return type, process body normally
            self.body(&func.body, out)?;
        }

        self.dedent();

        Ok(())
    }

    fn if_stmt(&mut self, if_stmt: &If, out: &mut impl Write) -> AutoResult<()> {
        // Process first branch as "if"
        if let Some(first_branch) = if_stmt.branches.first() {
            self.print_indent(out)?;
            out.write(b"if ")?;
            self.expr(&first_branch.cond, out)?;
            out.write(b":\n")?;
            self.indent();
            self.body(&first_branch.body, out)?;
            self.dedent();
        }

        // Process remaining branches as "elif"
        for branch in if_stmt.branches.iter().skip(1) {
            self.print_indent(out)?;
            out.write(b"elif ")?;
            self.expr(&branch.cond, out)?;
            out.write(b":\n")?;
            self.indent();
            self.body(&branch.body, out)?;
            self.dedent();
        }

        // Process else if present
        if let Some(else_) = &if_stmt.else_ {
            self.print_indent(out)?;
            out.write(b"else:\n")?;
            self.indent();
            self.body(else_, out)?;
            self.dedent();
        }

        Ok(())
    }

    fn for_loop(&mut self, for_loop: &For, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"for ")?;

        // Handle iterator based on type
        match &for_loop.iter {
            Iter::Named(name) => {
                // Simple: for name in range
                out.write_all(name.as_bytes())?;
            }
            Iter::Indexed(index, name) => {
                // for i, name in enumerate(...)
                out.write_all(index.as_bytes())?;
                out.write(b", ")?;
                out.write_all(name.as_bytes())?;
            }
            Iter::Call(call) => {
                // Function call as iterator - skip for now
                out.write(b"_")?;
            }
            _ => {
                out.write(b"_")?;
            }
        }

        out.write(b" in ")?;

        // Handle range expressions
        match &for_loop.range {
            Expr::Range(range) => {
                out.write(b"range(")?;
                self.expr(&range.start, out)?;
                out.write(b", ")?;
                self.expr(&range.end, out)?;
                if range.eq {
                    // Inclusive range: add 1
                    out.write(b" + 1")?;
                }
                out.write(b")")?;
            }
            _ => {
                self.expr(&for_loop.range, out)?;
            }
        }

        out.write(b":\n")?;
        self.indent();
        self.body(&for_loop.body, out)?;
        self.dedent();

        Ok(())
    }

    fn is_stmt(&mut self, is_stmt: &Is, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"match ")?;
        self.expr(&is_stmt.target, out)?;
        out.write(b":\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(out)?;
            match branch {
                IsBranch::EqBranch(expr, body) => {
                    out.write(b"case ")?;
                    self.expr(expr, out)?;
                    out.write(b":\n")?;
                    self.indent();
                    self.body(body, out)?;
                    self.dedent();
                }
                IsBranch::IfBranch(expr, body) => {
                    // Guard pattern - Python supports this with if guards
                    out.write(b"case ")?;
                    self.expr(expr, out)?;
                    out.write(b" if True:\n")?; // TODO: extract guard condition
                    self.indent();
                    self.body(body, out)?;
                    self.dedent();
                }
                IsBranch::ElseBranch(body) => {
                    out.write(b"case _:\n")?;
                    self.indent();
                    self.body(body, out)?;
                    self.dedent();
                }
            }
        }

        self.dedent();
        Ok(())
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        self.expr(&call.name, out)?;
        out.write(b"(")?;

        for (i, arg) in call.args.args.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.arg(arg, out)?;
        }

        out.write(b")")?;
        Ok(())
    }

    fn arg(&mut self, arg: &Arg, out: &mut impl Write) -> AutoResult<()> {
        match arg {
            Arg::Pos(expr) => self.expr(expr, out),
            Arg::Name(name) => out.write_all(name.as_bytes()).map_err(Into::into),
            Arg::Pair(key, expr) => {
                out.write_all(key.as_bytes())?;
                out.write(b"=")?;
                self.expr(expr, out)
            }
        }
    }

    fn array(&mut self, elems: &Vec<Expr>, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"[")?;
        for (i, elem) in elems.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.expr(elem, out)?;
        }
        out.write(b"]")?;
        Ok(())
    }

    fn index(&mut self, arr: &Box<Expr>, idx: &Box<Expr>, out: &mut impl Write) -> AutoResult<()> {
        self.expr(arr, out)?;
        out.write(b"[")?;
        self.expr(idx, out)?;
        out.write(b"]")?;
        Ok(())
    }

    fn fstr(&mut self, fstr: &FStr, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"f\"")?;
        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    // Literal string part - escape quotes
                    let escaped = s.replace("\"", "\\\"");
                    out.write_all(escaped.as_bytes())?;
                }
                Expr::Char(c) => {
                    // Character
                    out.write_all(c.to_string().as_bytes())?;
                }
                _ => {
                    // Expression placeholder - AutoLang uses $, Python uses {}
                    out.write(b"{")?;
                    self.expr(part, out)?;
                    out.write(b"}")?;
                }
            }
        }
        out.write(b"\"")?;
        Ok(())
    }

    fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"@dataclass\n")?;
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(type_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // Emit fields
        for member in &type_decl.members {
            self.print_indent(out)?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;
            // Simple type mapping for common types
            let type_name = self.python_type_name(&member.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b"\n")?;
        }

        self.dedent();
        Ok(())
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(enum_decl.name.as_bytes())?;
        out.write(b"(Enum):\n")?;
        self.indent();

        for item in &enum_decl.items {
            self.print_indent(out)?;
            out.write_all(item.name.as_bytes())?;
            out.write(b" = auto()\n")?;
        }

        self.dedent();
        Ok(())
    }

    fn python_type_name(&self, ty: &Type) -> AutoStr {
        match ty {
            Type::Int => "int".into(),
            Type::Uint => "int".into(),
            Type::Float => "float".into(),
            Type::Double => "float".into(),
            Type::Bool => "bool".into(),
            Type::Str(_) => "str".into(),
            Type::CStr => "str".into(),
            Type::User(type_decl) => type_decl.name.clone(),
            Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
            _ => "Any".into(), // Fallback for complex types
        }
    }

    fn body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        for stmt in &body.stmts {
            self.stmt(stmt, out)?;
        }
        Ok(())
    }
}

impl Trans for PythonTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // Find and save main function if it exists
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

            if stmt.is_decl() {
                decls.push(stmt);
            } else {
                main_stmts.push(stmt);
            }
        }

        // First pass: process declarations to collect imports
        for decl in &decls {
            if let Stmt::TypeDecl(type_decl) = decl {
                // Collect import without emitting
                if type_decl.members.len() > 0 {
                    self.imports.insert("dataclass".into());
                }
            } else if let Stmt::EnumDecl(enum_decl) = decl {
                // Collect import without emitting
                if enum_decl.items.len() > 0 {
                    self.imports.insert("Enum".into());
                }
            }
        }

        // Emit imports if needed
        if self.imports.contains("dataclass") {
            sink.body.write(b"from dataclasses import dataclass\n")?;
        }
        if self.imports.contains("Enum") {
            sink.body.write(b"from enum import Enum, auto\n")?;
        }
        if !self.imports.is_empty() {
            sink.body.write(b"\n")?;
        }

        // Generate declarations (excluding main)
        for (i, decl) in decls.iter().enumerate() {
            self.stmt(decl, &mut sink.body)?;
            // Add newline between declarations, but not after the last one
            if i < decls.len() - 1 {
                sink.body.write(b"\n")?;
            }
        }

        // Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            // Output the main function
            if !decls.is_empty() {
                sink.body.write(b"\n")?;
            }
            self.stmt(&main_stmt, &mut sink.body)?;

            // Add main guard
            sink.body.write(b"\nif __name__ == \"__main__\":\n")?;
            self.indent();
            sink.body.write(b"    main()\n")?;
            self.dedent();
        } else if !main_stmts.is_empty() {
            // Wrap statements in a main function
            if !decls.is_empty() {
                sink.body.write(b"\n")?;
            }
            sink.body.write(b"def main():\n")?;
            self.indent();
            for stmt in &main_stmts {
                self.stmt(stmt, &mut sink.body)?;
            }
            self.dedent();

            // Add main guard
            sink.body.write(b"\n\nif __name__ == \"__main__\":\n")?;
            self.indent();
            sink.body.write(b"main()\n")?;
            self.dedent();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    fn test_a2p(case: &str) -> AutoResult<()> {
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2p/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let scope = shared(Universe::new());
        let mut parser = Parser::new(src.as_str(), scope);
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = PythonTrans::new(name.into());
        trans.set_scope(parser.scope.clone());
        trans.trans(ast, &mut sink)?;
        let py_code = sink.done()?;

        let expected_path = format!("test/a2p/{}/{}.expected.py", case, name);
        let expected_path = d.join(expected_path);
        let expected = read_to_string(expected_path.as_path())?;

        if py_code != expected.as_bytes() {
            let gen_path = format!("test/a2p/{}/{}.wrong.py", case, name);
            let gen_path = d.join(gen_path);
            std::fs::write(&gen_path, py_code)?;
        }

        assert_eq!(String::from_utf8_lossy(py_code), expected);
        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2p("000_hello").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2p("010_if").unwrap();
    }

    #[test]
    fn test_011_for() {
        test_a2p("011_for").unwrap();
    }

    #[test]
    fn test_003_func() {
        test_a2p("003_func").unwrap();
    }

    #[test]
    fn test_012_is() {
        test_a2p("012_is").unwrap();
    }

    #[test]
    fn test_002_array() {
        test_a2p("002_array").unwrap();
    }

    #[test]
    fn test_015_str() {
        test_a2p("015_str").unwrap();
    }

    #[test]
    fn test_006_struct() {
        test_a2p("006_struct").unwrap();
    }

    #[test]
    fn test_007_enum() {
        test_a2p("007_enum").unwrap();
    }
}
