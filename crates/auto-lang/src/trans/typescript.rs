//! TypeScript Transpiler (Plan 100: a2js → a2ts)
//!
//! Transpiles AutoLang AST to TypeScript code with full type annotations.
//! Based on javascript.rs but adds:
//! - Type annotations for function parameters and return types
//! - Type annotations for variable declarations
//! - Interface generation for type declarations
//! - Type alias generation

use super::{Sink, Trans, ToStrError};
use crate::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use std::io::Write;

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

    /// Convert AutoLang type to TypeScript type
    fn type_to_ts(ty: &Type) -> String {
        match ty {
            // Numeric types → number
            Type::Int | Type::I64 | Type::Byte | Type::Char => "number".to_string(),
            Type::Uint | Type::U64 | Type::USize => "number".to_string(),
            Type::Float | Type::Double => "number".to_string(),

            // Boolean → boolean
            Type::Bool => "boolean".to_string(),

            // String types → string
            Type::Str(_) | Type::CStr | Type::StrSlice => "string".to_string(),

            // Array types → T[]
            Type::Array(arr) => {
                let elem_ts = Self::type_to_ts(&arr.elem);
                format!("{}[]", elem_ts)
            }
            Type::RuntimeArray(rta) => {
                let elem_ts = Self::type_to_ts(&rta.elem);
                format!("{}[]", elem_ts)
            }
            Type::List(elem) => {
                let elem_ts = Self::type_to_ts(elem);
                format!("{}[]", elem_ts)
            }
            Type::Slice(slice) => {
                let elem_ts = Self::type_to_ts(&slice.elem);
                format!("{}[]", elem_ts)
            }

            // Pointer/Reference → type (no pointer arithmetic in TS)
            Type::Ptr(ptr) => Self::type_to_ts(&ptr.of.borrow()),
            Type::Reference(inner) => Self::type_to_ts(inner),

            // User-defined types → type name
            Type::User(type_decl) => type_decl.name.to_string(),
            Type::GenericInstance(inst) => {
                if inst.args.is_empty() {
                    inst.base_name.to_string()
                } else {
                    let args: Vec<String> = inst.args.iter()
                        .map(|t| Self::type_to_ts(t))
                        .collect();
                    format!("{}<{}>", inst.base_name, args.join(", "))
                }
            }

            // Enum → type name
            Type::Enum(enum_decl) => enum_decl.borrow().name.to_string(),

            // Spec (interface) → type name
            Type::Spec(spec_decl) => spec_decl.borrow().name.to_string(),

            // Function type
            Type::Fn(params, ret) => {
                let param_ts: Vec<String> = params.iter()
                    .map(|t| Self::type_to_ts(t))
                    .collect();
                let ret_ts = Self::type_to_ts(ret);
                format!("({}) => {}", param_ts.join(", "), ret_ts)
            }

            // Void → void
            Type::Void => "void".to_string(),

            // Unknown → any (or could use unknown for stricter checking)
            Type::Unknown => "any".to_string(),

            // C Struct → type name
            Type::CStruct(type_decl) => type_decl.name.to_string(),

            // Linear type → inner type (no linear types in TS)
            Type::Linear(inner) => Self::type_to_ts(inner),

            // Variadic → ...any[]
            Type::Variadic => "...any[]".to_string(),

            // Union/Tag → any (complex types)
            Type::Union(_) | Type::Tag(_) => "any".to_string(),

            // Storage → type name
            Type::Storage(storage) => storage.to_string(),

            // Plan 120: Option and Result types
            Type::Option(inner) => format!("{} | null", Self::type_to_ts(inner)),
            Type::Result(inner) => format!("{} | Error", Self::type_to_ts(inner)),
            // Plan 121: Handle type - maps to TaskHandle<T> interface
            Type::Handle { task_type } => format!("TaskHandle<{}>", Self::type_to_ts(task_type)),
        }
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

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // TypeScript uses . for all field access
                self.expr(object, out)?;
                out.write_all(b".")?;
                out.write_all(field.as_bytes())?;
                Ok(())
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
            _ => Err(format!("TypeScript Transpiler: unsupported expression: {}", expr).into()),
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

            // Store (variable assignment) - with type annotation
            Stmt::Store(store) => {
                // TypeScript: const for let (immutable), let for var (mutable)
                match store.kind {
                    StoreKind::Let => out.write(b"const ").to()?,
                    StoreKind::Var => out.write(b"let ").to()?,
                    _ => {} // Field and CVar don't need declaration
                };
                out.write_all(store.name.as_bytes())?;

                // Add type annotation if type is known
                if !matches!(store.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&store.ty).as_bytes())?;
                }

                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
                out.write(b";")?;
                Ok(())
            }

            // Function declarations - with type annotations
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

            // Type declarations → interface
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, out)?;
                Ok(())
            }

            // Enum declarations → const enum or union type
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, out)?;
                Ok(())
            }

            // Unsupported statements
            _ => Err(format!("TypeScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"function ")?;
        out.write_all(func.name.as_bytes())?;
        out.write(b"(")?;

        // Parameters with type annotations
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;

            // Add type annotation
            if !matches!(param.ty, Type::Unknown) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
            }
        }

        out.write(b")")?;

        // Return type annotation
        if !matches!(func.ret, Type::Unknown | Type::Void) {
            out.write(b": ")?;
            out.write_all(Self::type_to_ts(&func.ret).as_bytes())?;
        } else if matches!(func.ret, Type::Void) {
            out.write(b": void")?;
        }

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
                    return Err(format!("TypeScript Transpiler: for loop requires range, got: {:?}", for_loop.range).into());
                }

                out.write(b")")?;
                self.if_body(&for_loop.body, out)?;
            }
            _ => {
                return Err(format!("TypeScript Transpiler: unsupported for loop iteration: {:?}", for_loop.iter).into());
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

    /// Generate TypeScript interface for type declaration
    fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"interface ")?;
        out.write_all(type_decl.name.as_bytes())?;
        out.write(b" {")?;

        // Members as interface properties
        for member in &type_decl.members {
            out.write(b"\n    ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;

            // Use member's type if available
            let member_type = if !matches!(member.ty, Type::Unknown) {
                Self::type_to_ts(&member.ty)
            } else {
                "any".to_string()
            };
            out.write_all(member_type.as_bytes())?;
            out.write(b";")?;
        }

        // Methods as interface methods
        for method in &type_decl.methods {
            out.write(b"\n    ")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b"(")?;

            for (i, param) in method.params.iter().enumerate() {
                if i > 0 {
                    out.write(b", ")?;
                }
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    out.write_all(Self::type_to_ts(&param.ty).as_bytes())?;
                }
            }

            out.write(b")")?;

            // Return type
            if !matches!(method.ret, Type::Unknown | Type::Void) {
                out.write(b": ")?;
                out.write_all(Self::type_to_ts(&method.ret).as_bytes())?;
            } else if matches!(method.ret, Type::Void) {
                out.write(b": void")?;
            }

            out.write(b";")?;
        }

        out.write(b"\n}")?;
        Ok(())
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        // Generate TypeScript enum (const enum for better performance)
        out.write(b"const enum ")?;
        out.write_all(enum_decl.name.as_bytes())?;
        out.write(b" {")?;

        for (i, item) in enum_decl.items.iter().enumerate() {
            if i > 0 {
                out.write(b",")?;
            }
            out.write(b"\n    ")?;
            out.write_all(item.name.as_bytes())?;

            // If there's an explicit non-zero value, use it
            // Note: TypeScript enums auto-increment from 0, so we only need explicit values for non-defaults
            if item.value != 0 {
                out.write(b" = ")?;
                write!(out, "{}", item.value)?;
            }
        }

        out.write(b"\n}")?;
        Ok(())
    }
}

impl Trans for TypeScriptTrans {
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
}
