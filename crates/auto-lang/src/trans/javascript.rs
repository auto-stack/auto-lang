use super::{Sink, Trans, ToStrError};
use crate::ast::*;
use crate::universe::Universe;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use auto_val::{shared, Shared};
use std::io::Write;

pub struct JavaScriptTrans {
    #[allow(dead_code)]
    name: AutoStr,
    scope: Shared<Universe>,
}

impl JavaScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            name,
            scope: shared(Universe::default()),
        }
    }

    pub fn set_scope(&mut self, scope: Shared<Universe>) {
        self.scope = scope;
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

            // F-strings → Template literals (perfect match!)
            Expr::FStr(fstr) => self.fstr(fstr, out),

            // Identifiers
            Expr::Ident(name) => out.write_all(name.as_bytes()).map_err(Into::into),

            // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => self.dot(lhs, rhs, out),
                    _ => {
                        self.expr(lhs, out)?;
                        out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                        self.expr(rhs, out)
                    }
                }
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
                out.write(b"{")?;
                for stmt in &block.stmts {
                    self.stmt(stmt, out)?;
                }
                out.write(b"}")?;
                Ok(())
            }

            // Unsupported expressions
            _ => Err(format!("JavaScript Transpiler: unsupported expression: {}", expr).into()),
        }
    }

    fn stmt(&mut self, stmt: &Stmt, out: &mut impl Write) -> AutoResult<()> {
        match stmt {
            // Expression statements
            Stmt::Expr(expr) => {
                self.expr(expr, out)?;
                out.write(b";")?;
                Ok(())
            }

            // Store (variable assignment)
            Stmt::Store(store) => {
                // AutoLang: let (immutable) → const, mut → let, var → let
                match store.kind {
                    StoreKind::Let => out.write(b"const ").to()?,
                    StoreKind::Mut | StoreKind::Var => out.write(b"let ").to()?,
                    _ => {} // Field and CVar don't need declaration
                };
                out.write_all(store.name.as_bytes())?;
                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
                out.write(b";")?;
                Ok(())
            }

            // Function declarations
            Stmt::Fn(func) => {
                self.fn_decl(func, out)?;
                Ok(())
            }

            // If statements
            Stmt::If(if_stmt) => {
                self.if_stmt(if_stmt, out)?;
                Ok(())
            }

            // For loops
            Stmt::For(for_loop) => {
                self.for_loop(for_loop, out)?;
                Ok(())
            }

            // Break statements
            Stmt::Break => {
                out.write(b"break;")?;
                Ok(())
            }

            // Pattern matching (is)
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, out)?;
                Ok(())
            }

            // Empty lines
            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    out.write(b"\n")?;
                }
                Ok(())
            }

            // Type declarations
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, out)?;
                Ok(())
            }

            // Enum declarations
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, out)?;
                Ok(())
            }

            // Unsupported statements
            _ => Err(format!("JavaScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"function ")?;
        out.write_all(func.name.as_bytes())?;
        out.write(b"(")?;

        // Parameters (without types for JavaScript)
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
        }

        out.write(b")")?;

        // Function body
        self.body(&func.body, out)?;

        Ok(())
    }

    fn body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        out.write(b" {")?;
        for stmt in &body.stmts {
            out.write(b"\n    ")?;
            self.stmt(stmt, out)?;
        }
        out.write(b"\n}")?;
        Ok(())
    }

    fn if_stmt(&mut self, if_stmt: &If, out: &mut impl Write) -> AutoResult<()> {
        // Process first branch as "if"
        if let Some(first_branch) = if_stmt.branches.first() {
            out.write(b"if (")?;
            self.expr(&first_branch.cond, out)?;
            out.write(b")")?;
            self.if_body(&first_branch.body, out)?;
        }

        // Process remaining branches as "else if"
        for branch in if_stmt.branches.iter().skip(1) {
            out.write(b" else if (")?;
            self.expr(&branch.cond, out)?;
            out.write(b")")?;
            self.if_body(&branch.body, out)?;
        }

        // Process else if present
        if let Some(else_) = &if_stmt.else_ {
            out.write(b" else")?;
            self.if_body(else_, out)?;
        }

        Ok(())
    }

    fn for_loop(&mut self, for_loop: &For, out: &mut impl Write) -> AutoResult<()> {
        // For Phase 1, only handle Named iterator with Range
        match &for_loop.iter {
            Iter::Named(name) => {
                // Generate traditional for loop for ranges
                out.write(b"for (let ")?;
                out.write_all(name.as_bytes())?;
                out.write(b" = ")?;

                // Extract range from range expression
                if let Expr::Range(range) = &for_loop.range {
                    self.expr(&range.start, out)?;
                    out.write(b"; ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b" < ")?;
                    self.expr(&range.end, out)?;
                    out.write(b"; ")?;
                    out.write_all(name.as_bytes())?;
                    out.write(b"++")?;
                } else {
                    return Err(format!("JavaScript Transpiler: for loop requires range, got: {:?}", for_loop.range).into());
                }

                out.write(b")")?;
                self.if_body(&for_loop.body, out)?;
            }
            _ => {
                return Err(format!("JavaScript Transpiler: unsupported for loop iteration: {:?}", for_loop.iter).into());
            }
        }
        Ok(())
    }

    fn is_stmt(&mut self, is_stmt: &Is, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"switch (")?;
        self.expr(&is_stmt.target, out)?;
        out.write(b") {")?;

        for branch in &is_stmt.branches {
            match branch {
                IsBranch::EqBranch(expr, body) => {
                    out.write(b"\n        case ")?;
                    self.expr(expr, out)?;
                    out.write(b":")?;
                    self.switch_case_body(body, out)?;
                    out.write(b"\n            break;")?;
                }
                IsBranch::IfBranch(expr, body) => {
                    out.write(b"\n        case ")?;
                    self.expr(expr, out)?;
                    out.write(b":")?;
                    self.switch_case_body(body, out)?;
                    out.write(b"\n            break;")?;
                }
                IsBranch::ElseBranch(body) => {
                    out.write(b"\n        default:")?;
                    self.switch_case_body(body, out)?;
                }
            }
        }

        out.write(b"\n    }")?;
        Ok(())
    }

    #[allow(dead_code)]
    fn body_in_line(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        if body.stmts.len() == 1 {
            out.write(b" ")?;
            self.stmt(&body.stmts[0], out)?;
            // Remove the semicolon since we're in a switch case
            // (already added by stmt)
        } else {
            out.write(b" {")?;
            for stmt in &body.stmts {
                out.write(b"\n        ")?;
                self.stmt(stmt, out)?;
            }
            out.write(b"\n    }")?;
        }
        Ok(())
    }

    fn switch_case_body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        for stmt in &body.stmts {
            out.write(b"\n            ")?;
            self.stmt(stmt, out)?;
        }
        Ok(())
    }

    fn if_body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        if body.stmts.is_empty() {
            out.write(b" {}")?;
        } else if body.stmts.len() == 1 {
            out.write(b" {\n        ")?;
            self.stmt(&body.stmts[0], out)?;
            out.write(b"\n    }")?;
        } else {
            out.write(b" {")?;
            for stmt in &body.stmts {
                out.write(b"\n        ")?;
                self.stmt(stmt, out)?;
            }
            out.write(b"\n    }")?;
        }
        Ok(())
    }

    fn fstr(&mut self, fstr: &FStr, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"`")?;
        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    let escaped = s.replace("`", "\\`").replace("${", "\\${");
                    out.write_all(escaped.as_bytes())?;
                }
                Expr::Char(c) => {
                    out.write_all(c.to_string().as_bytes())?;
                }
                _ => {
                    out.write(b"${")?;
                    self.expr(part, out)?;
                    out.write(b"}")?;
                }
            }
        }
        out.write(b"`")?;
        Ok(())
    }

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Check if this is a print call and convert to console.log
        let is_print = matches!(&*call.name, Expr::Ident(name) if name == "print");

        if is_print {
            out.write(b"console.log")?;
        } else {
            self.expr(&call.name, out)?;
        }

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
                out.write(b": ")?;
                self.expr(expr, out)?;
                Ok(())
            }
        }
    }

    fn array(&mut self, elems: &[Expr], out: &mut impl Write) -> AutoResult<()> {
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

    fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
        self.expr(lhs, out)?;
        out.write(b".")?;
        self.expr(rhs, out)?;
        Ok(())
    }

    fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"class ")?;
        out.write_all(type_decl.name.as_bytes())?;
        out.write(b" {")?;

        // Constructor
        out.write(b"\n    constructor(")?;
        for (i, member) in type_decl.members.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(member.name.as_bytes())?;
        }
        out.write(b") {")?;

        for member in &type_decl.members {
            out.write(b"\n        this.")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b" = ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b";")?;
        }
        out.write(b"\n    }")?;

        // Methods
        for method in &type_decl.methods {
            out.write(b"\n\n    ")?;
            self.method_in_class(method, out)?;
        }

        out.write(b"\n}")?;
        Ok(())
    }

    fn method_in_class(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        out.write_all(func.name.as_bytes())?;
        out.write(b"(")?;

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
        }

        out.write(b") {")?;

        // Method body
        for stmt in &func.body.stmts {
            out.write(b"\n        ")?;
            // Convert .x to this.x in method body
            self.stmt_with_this(stmt, out)?;
        }
        out.write(b"\n    }")?;

        Ok(())
    }

    fn stmt_with_this(&mut self, stmt: &Stmt, out: &mut impl Write) -> AutoResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                out.write(b"return ")?;
                self.expr_with_this(expr, out)?;
                out.write(b";")?;
                Ok(())
            }
            _ => self.stmt(stmt, out),
        }
    }

    fn expr_with_this(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Convert .x (Unary Dot) to this.x
            Expr::Unary(Op::Dot, inner) => {
                out.write(b"this.")?;
                self.expr_with_this(inner, out)
            }
            // Convert self to this
            Expr::Ident(name) if name == "self" => {
                out.write(b"this")?;
                Ok(())
            }
            // For other expressions, recurse and handle Dot in binary ops
            Expr::Bina(lhs, Op::Dot, rhs) => {
                self.expr_with_this(lhs, out)?;
                out.write(b".")?;
                self.expr(rhs, out)
            }
            // For other binary ops, recurse on both sides
            Expr::Bina(lhs, op, rhs) => {
                self.expr_with_this(lhs, out)?;
                out.write(format!(" {} ", op.op()).as_bytes()).to()?;
                self.expr_with_this(rhs, out)
            }
            _ => self.expr(expr, out),
        }
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"const ")?;
        out.write_all(enum_decl.name.as_bytes())?;
        out.write(b" = Object.freeze({")?;

        for (i, item) in enum_decl.items.iter().enumerate() {
            if i > 0 {
                out.write(b",")?;
            }
            out.write(b"\n    ")?;
            out.write_all(item.name.as_bytes())?;
            out.write(b": \"")?;
            out.write_all(item.name.as_bytes())?;
            out.write(b"\"")?;
        }

        out.write(b"\n});")?;
        Ok(())
    }
}

impl Trans for JavaScriptTrans {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
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
            sink.body.write(b"function main() {")?;

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

    fn test_a2j(case: &str) -> AutoResult<()> {
        let parts: Vec<&str> = case.split("_").collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2j/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = std::fs::read_to_string(src_path.as_path())?;

        let scope = shared(Universe::new());
        let mut parser = Parser::new(src.as_str(), scope);
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = JavaScriptTrans::new(name.into());
        trans.set_scope(parser.scope.clone());
        trans.trans(ast, &mut sink)?;
        let js_code = sink.done()?;

        let expected_path = format!("test/a2j/{}/{}.expected.js", case, name);
        let expected_path = d.join(expected_path);
        let expected = std::fs::read_to_string(expected_path.as_path())?;

        let js_string = String::from_utf8_lossy(&js_code);
        if js_string != expected {
            let wrong_path = format!("test/a2j/{}/{}.wrong.js", case, name);
            let wrong_path = d.join(wrong_path);
            std::fs::write(&wrong_path, js_code)?;
            panic!("Output differs from expected. Check {}.wrong.js", name);
        }

        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2j("000_hello").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2j("010_if").unwrap();
    }

    #[test]
    fn test_011_for() {
        test_a2j("011_for").unwrap();
    }

    #[test]
    fn test_012_is() {
        test_a2j("012_is").unwrap();
    }

    #[test]
    fn test_002_array() {
        test_a2j("002_array").unwrap();
    }

    #[test]
    fn test_003_func() {
        test_a2j("003_func").unwrap();
    }

    #[test]
    fn test_006_struct() {
        test_a2j("006_struct").unwrap();
    }

    #[test]
    fn test_007_enum() {
        test_a2j("007_enum").unwrap();
    }

    #[test]
    fn test_008_method() {
        test_a2j("008_method").unwrap();
    }
}
