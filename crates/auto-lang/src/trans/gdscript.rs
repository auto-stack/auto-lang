use super::{escape_str, Sink, ToStrError, Trans};
use crate::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use std::io::Write;

pub struct GDScriptTrans {
    indent: usize,
    #[allow(dead_code)]
    name: AutoStr,
}

impl GDScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self { indent: 0, name }
    }

    fn indent(&mut self) {
        self.indent += 1;
    }

    fn dedent(&mut self) {
        self.indent -= 1;
    }

    fn print_indent(&self, out: &mut impl Write) -> AutoResult<()> {
        for _ in 0..self.indent {
            // GDScript uses Tab indentation
            out.write(b"\t")?;
        }
        Ok(())
    }

    // ========================================================================
    // Expressions
    // ========================================================================

    fn expr(&mut self, expr: &Expr, out: &mut impl Write) -> AutoResult<()> {
        match expr {
            // Literals
            Expr::Int(i) => write!(out, "{}", i).map_err(Into::into),
            Expr::Uint(u) => write!(out, "{}", u).map_err(Into::into),
            Expr::Float(f, _) => write!(out, "{}", f).map_err(Into::into),
            Expr::Double(d, _) => write!(out, "{}", d).map_err(Into::into),
            // GDScript uses lowercase true/false (not Python's True/False)
            Expr::Bool(b) => write!(out, "{}", if *b { "true" } else { "false" }).map_err(Into::into),
            Expr::Char(c) => write!(out, "'{}'", c).map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),

            // F-strings → GDScript % formatting (no f-string support)
            Expr::FStr(fstr) => self.fstr(fstr, out),

            // Identifiers
            Expr::Ident(name) => out.write_all(name.as_bytes()).map_err(Into::into),
            Expr::GenName(name) => out.write_all(name.as_bytes()).map_err(Into::into),

            // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => self.dot(lhs, rhs, out),
                    _ => {
                        self.expr(lhs, out)?;
                        // GDScript supports both keywords and symbols: and/&&, or/||, not/!
                        let op_str = match op {
                            Op::And => "and",
                            Op::Or => "or",
                            _ => op.op(),
                        };
                        out.write(format!(" {} ", op_str).as_bytes()).to()?;
                        self.expr(rhs, out)
                    }
                }
            }

            // Dot expression for field access
            Expr::Dot(object, field) => {
                self.expr(object, out)?;
                out.write_all(b".")?;
                out.write_all(field.as_bytes())?;
                Ok(())
            }

            // Unary operations
            Expr::Unary(op, inner) => {
                let op_str = match op {
                    Op::Not => "not ", // GDScript uses 'not' keyword
                    _ => op.op(),
                };
                out.write(op_str.as_bytes())?;
                self.expr(inner, out)
            }

            // Function calls
            Expr::Call(call) => self.call(call, out),

            // Arrays
            Expr::Array(elems) => self.array(elems, out),

            // Index
            Expr::Index(arr, idx) => self.index(arr, idx, out),

            // Range
            Expr::Range(range) => self.range_expr(range, out),

            // Block expression
            Expr::Block(block) => {
                out.write(b"\n")?;
                self.indent();
                for stmt in &block.stmts {
                    self.stmt(stmt, out)?;
                }
                self.dedent();
                Ok(())
            }

            // Type cast / conversion
            Expr::Cast { expr, target_type } | Expr::To { expr, target_type } => {
                match target_type {
                    Type::Int | Type::Uint | Type::USize
                    | Type::I64 | Type::U64 | Type::Byte => {
                        write!(out, "int(")?;
                        self.expr(expr, out)?;
                        out.write(b")")?;
                    }
                    Type::Float | Type::Double => {
                        write!(out, "float(")?;
                        self.expr(expr, out)?;
                        out.write(b")")?;
                    }
                    Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => {
                        write!(out, "str(")?;
                        self.expr(expr, out)?;
                        out.write(b")")?;
                    }
                    _ => {
                        out.write(b"(")?;
                        self.expr(expr, out)?;
                        out.write(b")")?;
                    }
                }
                Ok(())
            }

            // Option/Result constructors
            // Some(x) -> x (GDScript Variant naturally supports null)
            Expr::Some(e) => self.expr(e, out),
            // None -> null (not Python's None)
            Expr::None => out.write(b"null").to(),
            // Ok(x) -> x
            Expr::Ok(e) => self.expr(e, out),
            // Err(msg) -> as error string
            Expr::Err(e) => {
                write!(out, "str(")?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Null coalescing: x ?? default -> x if x != null else default
            Expr::NullCoalesce(lhs, rhs) => {
                write!(out, "(")?;
                self.expr(lhs, out)?;
                write!(out, " if ")?;
                self.expr(lhs, out)?;
                write!(out, " != null else ")?;
                self.expr(rhs, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Closure/lambda -> GDScript lambda: func (params): body
            Expr::Closure(closure) => {
                write!(out, "func (")?;
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    out.write_all(param.name.as_bytes())?;
                }
                write!(out, "): return ")?;
                self.expr(&closure.body, out)
            }

            // Tuple -> Array in GDScript (no native tuple)
            Expr::Tuple(elems) => {
                write!(out, "[")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    self.expr(elem, out)?;
                }
                write!(out, "]").map_err(Into::into)
            }

            // Object literal -> Dictionary in GDScript
            Expr::Object(pairs) => {
                write!(out, "{{")?;
                for (i, pair) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    self.key(&pair.key, out)?;
                    write!(out, ": ")?;
                    self.expr(&pair.value, out)?;
                }
                write!(out, "}}").map_err(Into::into)
            }

            // Option pattern
            Expr::OptionPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::OptionVariant::Some => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "SomeCase({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "SomeCase(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::OptionVariant::None => {
                        write!(out, "null").map_err(Into::into)
                    }
                }
            }

            // Result pattern
            Expr::ResultPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::ResultVariant::Ok => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "OkCase({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "OkCase(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::ResultVariant::Err => {
                        if let Some(ref binding) = cover.binding {
                            write!(out, "ErrCase({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "ErrCase(_)").map_err(Into::into)
                        }
                    }
                }
            }

            // nil/Null -> null (GDScript uses null, not Python's None)
            Expr::Nil | Expr::Null => out.write(b"null").to(),

            // Error propagate x.? -> x (GDScript doesn't have ?, just pass through)
            Expr::ErrorPropagate(e) => self.expr(e, out),

            // Await expression
            Expr::Await { expr } => {
                out.write(b"await ")?;
                self.expr(expr, out)
            }

            // Go expression -> just emit (GDScript has no spawn/concurrency)
            Expr::Go { expr } => self.expr(expr, out),

            // Pair
            Expr::Pair(pair) => {
                self.key(&pair.key, out)?;
                write!(out, ": ")?;
                self.expr(&pair.value, out)
            }

            // Unsupported
            _ => Err(format!("GDScript Transpiler: unsupported expression: {:?}", expr).into()),
        }
    }

    // ========================================================================
    // Statements
    // ========================================================================

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

            // Pattern matching (is statement -> match)
            Stmt::Is(is_stmt) => {
                self.is_stmt(is_stmt, out)?;
                Ok(true)
            }

            // Break
            Stmt::Break => {
                self.print_indent(out)?;
                out.write(b"break\n")?;
                Ok(true)
            }

            // Continue
            Stmt::Continue => {
                self.print_indent(out)?;
                out.write(b"continue\n")?;
                Ok(true)
            }

            // Return
            Stmt::Return(expr) => {
                self.print_indent(out)?;
                out.write(b"return ")?;
                self.expr(expr, out)?;
                out.write(b"\n")?;
                Ok(true)
            }

            // Empty lines
            Stmt::EmptyLine(n) => {
                for _ in 0..*n {
                    out.write(b"\n")?;
                }
                Ok(true)
            }

            // Comments: Auto's // -> GDScript's #
            Stmt::Comment(cmt) => {
                self.print_indent(out)?;
                out.write(b"# ")?;
                out.write_all(cmt.as_bytes())?;
                out.write(b"\n")?;
                Ok(true)
            }

            // Type declarations (structs -> class)
            Stmt::TypeDecl(type_decl) => {
                self.type_decl(type_decl, out)?;
                Ok(true)
            }

            // Enum declarations
            Stmt::EnumDecl(enum_decl) => {
                self.enum_decl(enum_decl, out)?;
                Ok(true)
            }

            // Union declarations
            Stmt::Union(union_decl) => {
                self.union_decl(union_decl, out)?;
                Ok(true)
            }

            // Tag declarations
            Stmt::Tag(tag_decl) => {
                self.tag_decl(tag_decl, out)?;
                Ok(true)
            }

            // Spec declarations -> GDScript has no Protocol, skip with comment
            Stmt::SpecDecl(spec_decl) => {
                self.print_indent(out)?;
                out.write(b"# spec ")?;
                out.write_all(spec_decl.name.as_bytes())?;
                out.write(b" (not directly representable in GDScript)\n")?;
                Ok(true)
            }

            // Skip alias, use — GDScript uses preload/load, different import system
            Stmt::Alias(_) => Ok(false),
            Stmt::Use(_) => Ok(false),
            Stmt::TypeAlias(_) => Ok(false),
            Stmt::Dep(_) => Ok(false),

            _ => Err(format!("GDScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // ========================================================================
    // Store (variable declarations)
    // ========================================================================

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        match store.kind {
            StoreKind::Let | StoreKind::Var => {
                // Both let/var -> var in GDScript (no immutability concept)
                out.write(b"var ")?;
                out.write_all(store.name.as_bytes())?;
                // Type annotation if known
                if !matches!(store.ty, Type::Unknown) {
                    out.write(b": ")?;
                    let type_name = self.gdscript_type_name(&store.ty);
                    out.write_all(type_name.as_bytes())?;
                }
                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
            }
            StoreKind::Const => {
                // const X = 10 -> const X = 10
                out.write(b"const ")?;
                out.write_all(store.name.as_bytes())?;
                if !matches!(store.ty, Type::Unknown) {
                    out.write(b": ")?;
                    let type_name = self.gdscript_type_name(&store.ty);
                    out.write_all(type_name.as_bytes())?;
                }
                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
            }
            StoreKind::Shared => {
                // Shared storage -> var
                out.write(b"var ")?;
                out.write_all(store.name.as_bytes())?;
                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
            }
            StoreKind::CVar | StoreKind::Field => {
                // Field/CVar -> var
                out.write(b"var ")?;
                out.write_all(store.name.as_bytes())?;
                out.write(b" = ")?;
                self.expr(&store.expr, out)?;
            }
        }
        Ok(())
    }

    // ========================================================================
    // Function declarations
    // ========================================================================

    fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;

        // GDScript: func name(params) -> RetType:
        out.write(b"func ")?;

        // main() -> _ready()
        let fn_name = if func.name == "main" { "_ready" } else { &func.name };
        out.write_all(fn_name.as_bytes())?;

        out.write(b"(")?;

        // Parameters with type annotations
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
            if !matches!(param.ty, Type::Unknown) {
                out.write(b": ")?;
                let type_name = self.gdscript_type_name(&param.ty);
                out.write_all(type_name.as_bytes())?;
            }
        }

        out.write(b")")?;

        // Return type annotation (skip for main/_ready and void)
        if func.name != "main" && !matches!(func.ret, Type::Unknown | Type::Void) {
            out.write(b" -> ")?;
            let type_name = self.gdscript_type_name(&func.ret);
            out.write_all(type_name.as_bytes())?;
        }

        out.write(b":\n")?;

        self.indent();

        // Check if function has a non-void return type (except main)
        let has_return = !matches!(func.ret, Type::Unknown | Type::Void) && func.name != "main";

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
                    self.stmt(last_stmt, out)?;
                }
            }
        } else {
            // No return type, or empty body
            if func.body.stmts.is_empty() {
                self.print_indent(out)?;
                out.write(b"pass\n")?;
            } else {
                self.body(&func.body, out)?;
            }
        }

        self.dedent();
        Ok(())
    }

    /// Emit a method inside a class (adds self parameter)
    fn fn_decl_in_class(&mut self, func: &Fn, _type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;

        // Check if this is a static method (static fn in Auto)
        let is_static = func.is_static;

        if is_static {
            // GDScript static methods
            out.write(b"static ")?;
        }

        out.write(b"func ")?;
        out.write_all(func.name.as_bytes())?;

        if is_static {
            out.write(b"(")?;
        } else {
            out.write(b"(self")?;
        }

        // Parameters
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 || !is_static {
                out.write(b", ")?;
            } else if i == 0 && is_static {
                // First param for static — no comma prefix needed
            }
            out.write_all(param.name.as_bytes())?;
            if !matches!(param.ty, Type::Unknown) {
                out.write(b": ")?;
                let type_name = self.gdscript_type_name(&param.ty);
                out.write_all(type_name.as_bytes())?;
            }
        }

        out.write(b")")?;

        // Return type
        if !matches!(func.ret, Type::Unknown | Type::Void) {
            out.write(b" -> ")?;
            let type_name = self.gdscript_type_name(&func.ret);
            out.write_all(type_name.as_bytes())?;
        }

        out.write(b":\n")?;

        self.indent();

        let has_return = !matches!(func.ret, Type::Unknown | Type::Void);

        if has_return && !func.body.stmts.is_empty() {
            for stmt in func.body.stmts.iter().take(func.body.stmts.len() - 1) {
                self.stmt(stmt, out)?;
            }
            if let Some(last_stmt) = func.body.stmts.last() {
                if let Stmt::Expr(expr) = last_stmt {
                    self.print_indent(out)?;
                    out.write(b"return ")?;
                    self.expr(expr, out)?;
                    out.write(b"\n")?;
                } else {
                    self.stmt(last_stmt, out)?;
                }
            }
        } else {
            if func.body.stmts.is_empty() {
                self.print_indent(out)?;
                out.write(b"pass\n")?;
            } else {
                self.body(&func.body, out)?;
            }
        }

        self.dedent();
        Ok(())
    }

    // ========================================================================
    // Control flow
    // ========================================================================

    fn if_stmt(&mut self, if_stmt: &If, out: &mut impl Write) -> AutoResult<()> {
        // First branch as "if"
        if let Some(first_branch) = if_stmt.branches.first() {
            self.print_indent(out)?;
            out.write(b"if ")?;
            self.expr(&first_branch.cond, out)?;
            out.write(b":\n")?;
            self.indent();
            if first_branch.body.stmts.is_empty() {
                self.print_indent(out)?;
                out.write(b"pass\n")?;
            } else {
                self.body(&first_branch.body, out)?;
            }
            self.dedent();
        }

        // Remaining branches as "elif"
        for branch in if_stmt.branches.iter().skip(1) {
            self.print_indent(out)?;
            out.write(b"elif ")?;
            self.expr(&branch.cond, out)?;
            out.write(b":\n")?;
            self.indent();
            if branch.body.stmts.is_empty() {
                self.print_indent(out)?;
                out.write(b"pass\n")?;
            } else {
                self.body(&branch.body, out)?;
            }
            self.dedent();
        }

        // Else
        if let Some(else_) = &if_stmt.else_ {
            self.print_indent(out)?;
            out.write(b"else:\n")?;
            self.indent();
            if else_.stmts.is_empty() {
                self.print_indent(out)?;
                out.write(b"pass\n")?;
            } else {
                self.body(else_, out)?;
            }
            self.dedent();
        }

        Ok(())
    }

    fn for_loop(&mut self, for_loop: &For, out: &mut impl Write) -> AutoResult<()> {
        // Conditional for loop -> while
        if matches!(&for_loop.iter, Iter::Cond) {
            self.print_indent(out)?;
            out.write(b"while ")?;
            self.expr(&for_loop.range, out)?;
            out.write(b":\n")?;
            self.indent();
            self.body(&for_loop.body, out)?;
            self.dedent();
            return Ok(());
        }

        self.print_indent(out)?;
        out.write(b"for ")?;

        // Iterator
        match &for_loop.iter {
            Iter::Named(name) => {
                out.write_all(name.as_bytes())?;
            }
            Iter::Indexed(index, name) => {
                out.write_all(index.as_bytes())?;
                out.write(b", ")?;
                out.write_all(name.as_bytes())?;
            }
            _ => {
                out.write(b"_")?;
            }
        }

        out.write(b" in ")?;

        // Range expressions
        match &for_loop.range {
            Expr::Range(range) => {
                out.write(b"range(")?;
                self.expr(&range.start, out)?;
                out.write(b", ")?;
                self.expr(&range.end, out)?;
                if range.eq {
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
        // Auto: is x { ... } -> GDScript: match x:
        self.print_indent(out)?;
        out.write(b"match ")?;
        self.expr(&is_stmt.target, out)?;
        out.write(b":\n")?;
        self.indent();

        for branch in &is_stmt.branches {
            self.print_indent(out)?;
            match branch {
                IsBranch::EqBranch(patterns, body) => {
                    // GDScript match patterns don't use 'case' keyword
                    for (i, pat) in patterns.iter().enumerate() {
                        if i > 0 {
                            out.write(b", ")?;
                        }
                        self.expr(pat, out)?;
                    }
                    out.write(b":\n")?;
                    self.indent();
                    if body.stmts.is_empty() {
                        self.print_indent(out)?;
                        out.write(b"pass\n")?;
                    } else {
                        self.body(body, out)?;
                    }
                    self.dedent();
                }
                IsBranch::IfBranch(expr, body) => {
                    // Guard pattern
                    self.expr(expr, out)?;
                    out.write(b" when true:\n")?;
                    self.indent();
                    if body.stmts.is_empty() {
                        self.print_indent(out)?;
                        out.write(b"pass\n")?;
                    } else {
                        self.body(body, out)?;
                    }
                    self.dedent();
                }
                IsBranch::ElseBranch(body) => {
                    // GDScript wildcard: _
                    out.write(b"_:\n")?;
                    self.indent();
                    if body.stmts.is_empty() {
                        self.print_indent(out)?;
                        out.write(b"pass\n")?;
                    } else {
                        self.body(body, out)?;
                    }
                    self.dedent();
                }
            }
        }

        self.dedent();
        Ok(())
    }

    // ========================================================================
    // Type declarations
    // ========================================================================

    fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        // class_name declaration
        self.print_indent(out)?;
        out.write(b"class_name ")?;
        out.write_all(type_decl.name.as_bytes())?;
        out.write(b"\n\n")?;

        // Emit fields as member variables
        for member in &type_decl.members {
            self.print_indent(out)?;
            out.write(b"var ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = self.gdscript_type_name(&member.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b"\n")?;
        }

        // Emit methods if present
        if !type_decl.methods.is_empty() && !type_decl.members.is_empty() {
            out.write(b"\n")?;
        }

        for method in &type_decl.methods {
            self.fn_decl_in_class(method, type_decl, out)?;
        }

        Ok(())
    }

    fn enum_decl(&mut self, enum_decl: &EnumDecl, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"enum ")?;
        out.write_all(enum_decl.name.as_bytes())?;
        out.write(b" { ")?;

        for (i, item) in enum_decl.items.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            // GDScript convention: uppercase enum values
            let name_upper = item.name.as_str().to_uppercase();
            out.write_all(name_upper.as_bytes())?;
        }

        out.write(b" }\n")?;
        Ok(())
    }

    fn union_decl(&mut self, union_decl: &Union, out: &mut impl Write) -> AutoResult<()> {
        // Union -> class with kind discriminator (similar to Python strategy)
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(union_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // kind discriminator
        self.print_indent(out)?;
        out.write(b"var kind: String = \"\"\n")?;

        // Fields with defaults
        for field in &union_decl.fields {
            self.print_indent(out)?;
            out.write(b"var ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = self.gdscript_type_name(&field.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b" = ")?;
            let default_val = self.gdscript_default_value(&field.ty);
            out.write_all(default_val.as_bytes())?;
            out.write(b"\n")?;
        }

        self.dedent();
        Ok(())
    }

    fn tag_decl(&mut self, tag_decl: &Tag, out: &mut impl Write) -> AutoResult<()> {
        // Tag -> class with kind discriminator and factory methods
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(tag_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // kind discriminator
        self.print_indent(out)?;
        out.write(b"var kind: String = \"\"\n")?;

        // Fields with defaults
        for field in &tag_decl.fields {
            self.print_indent(out)?;
            out.write(b"var ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = self.gdscript_type_name(&field.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b" = ")?;
            let default_val = self.gdscript_default_value(&field.ty);
            out.write_all(default_val.as_bytes())?;
            out.write(b"\n")?;
        }

        // Factory methods for each variant
        if !tag_decl.fields.is_empty() {
            out.write(b"\n")?;
        }
        for field in &tag_decl.fields {
            self.print_indent(out)?;
            out.write(b"static func ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"(")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"):\n")?;
            self.indent();
            self.print_indent(out)?;
            out.write(b"var instance = ")?;
            out.write_all(tag_decl.name.as_bytes())?;
            out.write(b".new()\n")?;
            self.print_indent(out)?;
            out.write(b"instance.kind = \"")?;
            let name_str = field.name.as_str();
            if let Some(first) = name_str.chars().next() {
                for (i, c) in name_str.chars().enumerate() {
                    if i == 0 {
                        write!(out, "{}", first.to_ascii_uppercase())?;
                    } else {
                        out.write_all(c.to_string().as_bytes())?;
                    }
                }
            }
            out.write(b"\"\n")?;
            self.print_indent(out)?;
            out.write(b"instance.")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b" = ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"\n")?;
            self.print_indent(out)?;
            out.write(b"return instance\n")?;
            self.dedent();
        }

        // Instance methods
        for method in &tag_decl.methods {
            out.write(b"\n")?;
            self.fn_decl_in_class_for_tag(method, tag_decl, out)?;
        }

        self.dedent();
        Ok(())
    }

    fn fn_decl_in_class_for_tag(&mut self, func: &Fn, _tag: &Tag, out: &mut impl Write) -> AutoResult<()> {
        self.fn_decl_in_class(func, &TypeDecl::builtin(&_tag.name), out)
    }

    // ========================================================================
    // Helper methods
    // ========================================================================

    fn call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
        // Intercept method calls where call.name is Expr::Dot(obj, method_name)
        // Parser generates Expr::Call { name: Expr::Dot(obj, method), args } for obj.method(args)
        if let Expr::Dot(obj, method_name) = call.name.as_ref() {
            return self.method_call(obj, method_name, &call.args, out);
        }

        // Builtin function mapping
        if let Some(ident) = self.extract_call_name(&call.name) {
            match ident.as_ref() {
                // Identical in GDScript — just pass through
                "print" | "len" | "range" | "abs" | "min" | "max" | "str" | "int" | "float"
                | "clamp" | "lerp" | "wrapi" | "wrapf" => {
                    return self.emit_plain_call(call, out);
                }
                // type_name(x) → typeof(x)
                "type_name" => {
                    out.write(b"typeof(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    out.write(b")")?;
                    return Ok(());
                }
                // sleep_ms(ms) → await get_tree().create_timer(ms / 1000.0).timeout
                "sleep_ms" => {
                    out.write(b"await get_tree().create_timer(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    out.write(b" / 1000.0).timeout")?;
                    return Ok(());
                }
                // time_now() → Time.get_ticks_msec() / 1000.0
                "time_now" => {
                    out.write(b"Time.get_ticks_msec() / 1000.0")?;
                    return Ok(());
                }
                _ => {}
            }
        }

        self.emit_plain_call(call, out)
    }

    /// Emit a plain function call without any builtin mapping
    fn emit_plain_call(&mut self, call: &Call, out: &mut impl Write) -> AutoResult<()> {
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

    /// Emit call arguments as comma-separated list
    fn emit_args(&mut self, args: &Args, out: &mut impl Write) -> AutoResult<()> {
        for (i, arg) in args.args.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.arg(arg, out)?;
        }
        Ok(())
    }

    /// Extract a plain identifier name from a call expression
    fn extract_call_name(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Ident(name) => Some(name.clone()),
            _ => None,
        }
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

    fn dot(&mut self, lhs: &Expr, rhs: &Expr, out: &mut impl Write) -> AutoResult<()> {
        // Intercept method calls: lhs.method(args) where rhs is Expr::Call
        if let Expr::Call(call) = rhs {
            if let Expr::Ident(method_name) = call.name.as_ref() {
                return self.method_call(lhs, method_name, &call.args, out);
            }
        }
        // Default: lhs.rhs
        self.expr(lhs, out)?;
        out.write(b".")?;
        self.expr(rhs, out)?;
        Ok(())
    }

    /// Map AutoLang method calls to GDScript equivalents
    fn method_call(
        &mut self,
        receiver: &Expr,
        method: &AutoStr,
        args: &Args,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        match method.as_ref() {
            // ── String methods ──
            // .trim() → .strip()
            "trim" => {
                self.expr(receiver, out)?;
                out.write(b".strip(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .split(sep) → .split(sep) (same in GDScript)
            "split" => {
                self.expr(receiver, out)?;
                out.write(b".split(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .to_upper() → .to_upper() (same in GDScript)
            "to_upper" | "upper" => {
                self.expr(receiver, out)?;
                out.write(b".to_upper()")?;
            }
            // .to_lower() → .to_lower() (same in GDScript)
            "to_lower" | "lower" => {
                self.expr(receiver, out)?;
                out.write(b".to_lower()")?;
            }
            // .starts_with(s) → .begins_with(s) (GDScript uses begins_with)
            "starts_with" | "startswith" => {
                self.expr(receiver, out)?;
                out.write(b".begins_with(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .ends_with(s) → .ends_with(s) (same in GDScript)
            "ends_with" | "endswith" => {
                self.expr(receiver, out)?;
                out.write(b".ends_with(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .replace(old, new) → .replace(old, new) (same in GDScript)
            "replace" => {
                self.expr(receiver, out)?;
                out.write(b".replace(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .len() → len(receiver)
            "len" => {
                out.write(b"len(")?;
                self.expr(receiver, out)?;
                out.write(b")")?;
            }

            // ── List/Array methods ──
            // .push(item) → .append(item) (GDScript uses append)
            "push" => {
                self.expr(receiver, out)?;
                out.write(b".append(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .pop() → .pop() (same in GDScript)
            "pop" => {
                self.expr(receiver, out)?;
                out.write(b".pop(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .contains(item) → item in receiver
            "contains" => {
                if let Some(first_arg) = args.args.first() {
                    self.arg(first_arg, out)?;
                    out.write(b" in ")?;
                    self.expr(receiver, out)?;
                } else {
                    self.expr(receiver, out)?;
                    out.write(b".contains(")?;
                    self.emit_args(args, out)?;
                    out.write(b")")?;
                }
            }
            // .join(sep) → sep.join(receiver)
            "join" => {
                if let Some(first_arg) = args.args.first() {
                    self.arg(first_arg, out)?;
                    out.write(b".join(")?;
                    self.expr(receiver, out)?;
                    out.write(b")")?;
                } else {
                    self.expr(receiver, out)?;
                    out.write(b".join(")?;
                    self.emit_args(args, out)?;
                    out.write(b")")?;
                }
            }

            // ── Dict/Map methods ──
            // .set(k, v) / .insert(k, v) → dict[k] = v
            "set" | "insert" => {
                self.expr(receiver, out)?;
                out.write(b"[")?;
                if let Some(first) = args.args.first() {
                    self.arg(first, out)?;
                }
                out.write(b"] = ")?;
                if args.args.len() > 1 {
                    self.arg(&args.args[1], out)?;
                } else {
                    out.write(b"null")?;
                }
            }
            // .get(key) → receiver.get(key)
            "get" => {
                self.expr(receiver, out)?;
                out.write(b".get(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .has(key) / .contains_key(key) → key in receiver
            "has" | "contains_key" => {
                if let Some(first_arg) = args.args.first() {
                    self.arg(first_arg, out)?;
                    out.write(b" in ")?;
                    self.expr(receiver, out)?;
                } else {
                    self.expr(receiver, out)?;
                    out.write(b".has(")?;
                    self.emit_args(args, out)?;
                    out.write(b")")?;
                }
            }
            // .keys() / .values() — pass through
            "keys" | "values" => {
                self.expr(receiver, out)?;
                out.write(b".")?;
                out.write_all(method.as_bytes())?;
                out.write(b"(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }

            // ── Default: pass through as receiver.method(args) ──
            _ => {
                self.expr(receiver, out)?;
                out.write(b".")?;
                out.write_all(method.as_bytes())?;
                out.write(b"(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
        }
        Ok(())
    }

    fn range_expr(&mut self, range: &Range, out: &mut impl Write) -> AutoResult<()> {
        out.write(b"range(")?;
        self.expr(&range.start, out)?;
        out.write(b", ")?;
        self.expr(&range.end, out)?;
        if range.eq {
            out.write(b" + 1")?;
        }
        out.write(b")")?;
        Ok(())
    }

    /// F-string → GDScript % formatting
    /// Auto: f"Hello $name, age ${age + 1}"
    /// GDScript: "Hello %s, age %s" % [name, str(age + 1)]
    fn fstr(&mut self, fstr: &FStr, out: &mut impl Write) -> AutoResult<()> {
        let mut format_parts: Vec<u8> = Vec::new();
        let mut expr_parts: Vec<Vec<u8>> = Vec::new();

        for part in &fstr.parts {
            match part {
                Expr::Str(s) => {
                    // Literal string part
                    let escaped = s.replace("\"", "\\\"");
                    format_parts.extend_from_slice(escaped.as_bytes());
                }
                Expr::Char(c) => {
                    format_parts.extend_from_slice(c.to_string().as_bytes());
                }
                _ => {
                    // Expression placeholder → %s
                    format_parts.extend_from_slice(b"%s");
                    let mut expr_buf = Vec::new();
                    // Wrap non-string expressions in str()
                    // For simple identifiers, emit directly; for complex exprs, wrap in str()
                    match part {
                        Expr::Ident(_) => {
                            self.expr(part, &mut expr_buf)?;
                        }
                        _ => {
                            expr_buf.extend_from_slice(b"str(");
                            self.expr(part, &mut expr_buf)?;
                            expr_buf.extend_from_slice(b")");
                        }
                    }
                    expr_parts.push(expr_buf);
                }
            }
        }

        // Output: "format_string" % expr OR "format_string" % [expr1, expr2, ...]
        out.write(b"\"")?;
        out.write_all(&format_parts)?;
        out.write(b"\"")?;

        if !expr_parts.is_empty() {
            out.write(b" % ")?;
            if expr_parts.len() == 1 {
                out.write_all(&expr_parts[0])?;
            } else {
                out.write(b"[")?;
                for (i, ep) in expr_parts.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    out.write_all(ep)?;
                }
                out.write(b"]")?;
            }
        }

        Ok(())
    }

    fn key(&mut self, key: &Key, out: &mut impl Write) -> AutoResult<()> {
        match key {
            Key::NamedKey(name) => {
                write!(out, "\"{}\"", name)?;
            }
            Key::IntKey(i) => write!(out, "{}", i)?,
            Key::BoolKey(b) => write!(out, "{}", b)?,
            Key::StrKey(s) => write!(out, "\"{}\"", s)?,
        }
        Ok(())
    }

    fn body(&mut self, body: &Body, out: &mut impl Write) -> AutoResult<()> {
        for stmt in &body.stmts {
            self.stmt(stmt, out)?;
        }
        Ok(())
    }

    // ========================================================================
    // Type mapping
    // ========================================================================

    fn gdscript_type_name(&self, ty: &Type) -> AutoStr {
        match ty {
            Type::Int | Type::Uint | Type::USize | Type::I64 | Type::U64 | Type::Byte => "int".into(),
            Type::Float | Type::Double => "float".into(),
            Type::Bool => "bool".into(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => "String".into(),
            Type::Void => "void".into(),
            Type::User(type_decl) => type_decl.name.clone(),
            Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
            Type::List(_) => "Array".into(),
            Type::Map(_, _) => "Dictionary".into(),
            Type::Option(_) => "Variant".into(),
            Type::Result(_) => "Variant".into(),
            Type::GenericInstance(inst) => {
                if inst.base_name == "Future" {
                    if let Some(inner) = inst.args.first() {
                        return self.gdscript_type_name(inner);
                    }
                }
                "Variant".into()
            }
            _ => "Variant".into(),
        }
    }

    fn gdscript_default_value(&self, ty: &Type) -> AutoStr {
        match ty {
            Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::Byte => "0".into(),
            Type::Float | Type::Double => "0.0".into(),
            Type::Bool => "false".into(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => "\"\"".into(),
            _ => "null".into(),
        }
    }
}

// ============================================================================
// Trans trait implementation
// ============================================================================

impl Trans for GDScriptTrans {
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
        let mut decls: Vec<(Stmt, usize)> = Vec::new();
        let mut main_stmts: Vec<(Stmt, usize)> = Vec::new();

        let source_lines = ast.source_lines;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Skip main function declaration — we'll handle it specially
            if let Stmt::Fn(func) = &stmt {
                if func.name == "main" {
                    continue;
                }
            }

            if stmt.is_decl() {
                decls.push((stmt, line));
            } else {
                main_stmts.push((stmt, line));
            }
        }

        // 1. File header comment
        write!(sink.body, "# Auto-generated from {}.at — do not edit\n\n", self.name)?;

        // 2. extends Node (default for Godot scripts)
        sink.body.write(b"extends Node\n\n")?;

        // 3. Emit declarations (types, enums, non-main functions)
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.set_source_line(*line);
            self.stmt(decl, &mut sink.body)?;
            if i < decls.len() - 1 {
                sink.body.write(b"\n")?;
            }
        }

        // 4. Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            // Output the main function as _ready()
            if !decls.is_empty() {
                sink.body.write(b"\n")?;
            }
            self.stmt(&main_stmt, &mut sink.body)?;
        } else if !main_stmts.is_empty() {
            // Wrap loose statements in a _ready() function
            if !decls.is_empty() {
                sink.body.write(b"\n")?;
            }
            sink.body.write(b"func _ready():\n")?;
            self.indent();
            for (stmt, line) in &main_stmts {
                sink.set_source_line(*line);
                self.stmt(stmt, &mut sink.body)?;
            }
            self.dedent();
        }

        Ok(())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    use std::fs::read_to_string;
    use std::path::PathBuf;

    fn test_a2gd(case: &str) -> AutoResult<()> {
        let last_segment = case.rsplit('/').next().unwrap_or(case);
        let parts: Vec<&str> = last_segment.split("_").collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2gd/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        let _scope = crate::scope_manager::ScopeManager::new();
        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = GDScriptTrans::new(name.into());
        trans.trans(ast, &mut sink)?;
        let gd_code = sink.done()?;

        let expected_path = format!("test/a2gd/{}/{}.expected.gd", case, name);
        let expected_path = d.join(expected_path);
        let expected = read_to_string(expected_path.as_path())?;

        if gd_code != expected.as_bytes() {
            let gen_path = format!("test/a2gd/{}/{}.wrong.gd", case, name);
            let gen_path = d.join(gen_path);
            std::fs::write(&gen_path, gd_code)?;
        }

        assert_eq!(String::from_utf8_lossy(gd_code), expected);
        Ok(())
    }

    #[test]
    fn test_000_hello() {
        test_a2gd("000_hello").unwrap();
    }

    #[test]
    fn test_001_var() {
        test_a2gd("001_var").unwrap();
    }

    #[test]
    fn test_002_func() {
        test_a2gd("002_func").unwrap();
    }

    #[test]
    fn test_010_if() {
        test_a2gd("010_if").unwrap();
    }

    #[test]
    fn test_011_for() {
        test_a2gd("011_for").unwrap();
    }

    #[test]
    fn test_012_match() {
        test_a2gd("012_match").unwrap();
    }

    #[test]
    fn test_013_struct() {
        test_a2gd("013_struct").unwrap();
    }

    #[test]
    fn test_014_enum() {
        test_a2gd("014_enum").unwrap();
    }

    #[test]
    fn test_015_string() {
        test_a2gd("015_string").unwrap();
    }

    #[test]
    fn test_string_methods() {
        test_a2gd("04_strings/001_string_methods").unwrap();
    }

    #[test]
    fn test_array_methods() {
        test_a2gd("10_collections/001_array_methods").unwrap();
    }

    #[test]
    fn test_dict_methods() {
        test_a2gd("10_collections/002_dict_methods").unwrap();
    }

    #[test]
    fn test_builtin_map() {
        test_a2gd("16_gdscript_std/001_builtin_map").unwrap();
    }
}
