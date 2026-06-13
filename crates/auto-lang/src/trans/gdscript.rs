use super::{escape_str, Sink, ToStrError, Trans};
use crate::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use std::collections::HashMap;
use std::io::Write;

pub struct GDScriptTrans {
    indent: usize,
    #[allow(dead_code)]
    name: AutoStr,
    /// Collected preload paths from `use` statements: (module_path, optional_symbols)
    gd_imports: Vec<(AutoStr, Option<Vec<AutoStr>>)>,
    /// Local variable type tracking
    local_var_types: HashMap<AutoStr, Type>,
}

impl GDScriptTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            indent: 0,
            name,
            gd_imports: Vec::new(),
            local_var_types: HashMap::new(),
        }
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

            // Spec declarations -> GDScript inner class with method stubs
            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, out)?;
                Ok(true)
            }

            // Skip alias, use — GDScript uses preload/load, different import system
            Stmt::Alias(_) => Ok(false),
            Stmt::Use(use_stmt) => {
                self.handle_use(use_stmt);
                Ok(false)
            }
            Stmt::TypeAlias(_) => Ok(false),
            Stmt::Dep(_) => Ok(false),
            // Plan 306 Phase 2b: a `scene` declaration targets the .tscn output,
            // not the .gd — skip it so a mixed file can emit both.
            Stmt::SceneDecl(_) => Ok(false),

            _ => Err(format!("GDScript Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    // ========================================================================
    // Store (variable declarations)
    // ========================================================================

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Plan 306 Phase 2c/3: emit each annotation as a GDScript @-prefix.
        // store.attrs holds the full text e.g. "export", "onready",
        // "export_range(0, 100)", so `@{a} ` renders verbatim on the same line.
        for a in &store.attrs {
            write!(out, "@{} ", a.as_str())?;
        }
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
                // Track variable type
                let ty = if matches!(store.ty, Type::Unknown) {
                    self.infer_type_from_expr(&store.expr)
                } else {
                    store.ty.clone()
                };
                self.local_var_types.insert(store.name.clone(), ty);
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
        // Track parameter types
        self.local_var_types.clear();
        for param in &func.params {
            if !matches!(param.ty, Type::Unknown) {
                self.local_var_types.insert(param.name.clone(), param.ty.clone());
            }
        }

        self.print_indent(out)?;

        // GDScript: func name(params) -> RetType:
        out.write(b"func ")?;

        // main() -> _ready()
        let fn_name = if func.name == "main" { "_ready" } else { &func.name };
        out.write_all(fn_name.as_bytes())?;

        out.write(b"(")?;

        // Parameters with type annotations (skip generic params)
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
            if !matches!(param.ty, Type::Unknown) && !self.is_generic_param(&param.ty, func) {
                out.write(b": ")?;
                let type_name = self.gdscript_type_name(&param.ty);
                out.write_all(type_name.as_bytes())?;
            }
        }

        out.write(b")")?;

        // Return type annotation (skip for main/_ready, void, and generic params)
        if func.name != "main"
            && !matches!(func.ret, Type::Unknown | Type::Void)
            && !self.is_generic_param(&func.ret, func)
        {
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

        // Emit fields as member variables (use Variant for generic type params)
        for member in &type_decl.members {
            self.print_indent(out)?;
            out.write(b"var ")?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = if self.is_type_decl_generic_param(&member.ty, &type_decl.generic_params) {
                AutoStr::from("Variant")
            } else {
                self.gdscript_type_name(&member.ty)
            };
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
            // Plan 306 Phase 5b: preserve source casing (matches Auto's `Color.Red`
            // access) and explicit scalar values (`OK = 200`). Only emit `= N` when
            // the user wrote it explicitly — the parser auto-fills gap values, which
            // we must NOT surface (would make value-less enums noisy).
            out.write_all(item.name.as_bytes())?;
            if item.value_explicit {
                if let Some(v) = item.scalar_value {
                    write!(out, " = {}", v)?;
                }
            }
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
    // Spec declarations
    // ========================================================================

    fn spec_decl(&mut self, spec_decl: &SpecDecl, out: &mut impl Write) -> AutoResult<()> {
        self.print_indent(out)?;
        out.write(b"# Protocol: ")?;
        out.write_all(spec_decl.name.as_bytes())?;
        out.write(b"\n")?;

        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(spec_decl.name.as_bytes())?;
        out.write(b":\n")?;

        self.indent();
        if spec_decl.methods.is_empty() {
            self.print_indent(out)?;
            out.write(b"pass\n")?;
        } else {
            for method in &spec_decl.methods {
                self.print_indent(out)?;
                out.write(b"# Abstract: must override\n")?;
                self.print_indent(out)?;
                out.write(b"func ")?;
                out.write_all(method.name.as_bytes())?;
                out.write(b"(")?;
                for (i, param) in method.params.iter().enumerate() {
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
                if !matches!(method.ret, Type::Unknown | Type::Void) {
                    out.write(b" -> ")?;
                    let type_name = self.gdscript_type_name(&method.ret);
                    out.write_all(type_name.as_bytes())?;
                }
                out.write(b":\n")?;
                self.indent();
                self.print_indent(out)?;
                out.write(b"pass\n")?;
                self.dedent();
            }
        }
        self.dedent();
        Ok(())
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
            // Plan 306 Phase 5a: typed collections — recurse on element types.
            Type::List(elem) => format!("Array[{}]", self.gdscript_type_name(elem)).into(),
            Type::Map(k, v) => format!("Dictionary[{}, {}]", self.gdscript_type_name(k), self.gdscript_type_name(v)).into(),
            // GDScript has no fixed-size array; [N]T / []T / [expr]T → Array[T]
            Type::Array(at) => format!("Array[{}]", self.gdscript_type_name(&at.elem)).into(),
            Type::RuntimeArray(rta) => format!("Array[{}]", self.gdscript_type_name(&rta.elem)).into(),
            Type::Slice(st) => format!("Array[{}]", self.gdscript_type_name(&st.elem)).into(),
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

    // ========================================================================
    // Import handling
    // ========================================================================

    /// Process a `use` statement for GDScript import emission
    fn handle_use(&mut self, use_stmt: &Use) {
        match &use_stmt.kind {
            UseKind::Auto => {
                // Resolve module path
                let module = if let Some(ref mp) = use_stmt.module_path {
                    let display = mp.display();
                    // Strip pac./super. prefixes (AutoLang-only concepts)
                    if let Some(stripped) = display.strip_prefix("pac.") {
                        stripped.to_string()
                    } else if let Some(stripped) = display.strip_prefix("super.") {
                        stripped.to_string()
                    } else {
                        display.to_string()
                    }
                } else if !use_stmt.paths.is_empty() {
                    use_stmt.paths.join("/")
                } else {
                    return;
                };

                let symbols: Option<Vec<AutoStr>> = if use_stmt.items.is_empty() {
                    None
                } else {
                    Some(use_stmt.items.iter().cloned().collect())
                };
                self.gd_imports.push((module.into(), symbols));
            }
            UseKind::Py | UseKind::C | UseKind::Rust => {
                // Python/C/Rust imports not relevant for GDScript — skip
            }
        }
    }

    /// Emit collected GDScript imports (preload statements)
    fn emit_imports(&self, out: &mut impl Write) -> AutoResult<()> {
        for (path, symbols) in &self.gd_imports {
            // use module → const Module = preload("res://module.gd")
            let module_name = path.rsplit('/').next().unwrap_or(path.as_ref());
            let class_name = capitalize_first(module_name);
            write!(out, "const {} = preload(\"res://{}.gd\")\n", class_name, path)?;
            if let Some(syms) = symbols {
                if !syms.is_empty() {
                    write!(out, "# imported: {}\n", syms.join(", "))?;
                }
            }
        }
        if !self.gd_imports.is_empty() {
            out.write(b"\n")?;
        }
        Ok(())
    }

    /// Basic type inference from expression
    fn infer_type_from_expr(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Int(_) => Type::Int,
            Expr::Uint(_) => Type::Uint,
            Expr::Float(_, _) => Type::Float,
            Expr::Double(_, _) => Type::Double,
            Expr::Bool(_) => Type::Bool,
            Expr::Str(_) => Type::StrOwned,
            Expr::Array(_) => Type::List(Box::new(Type::Unknown)),
            Expr::Object(_) => Type::Map(Box::new(Type::Unknown), Box::new(Type::Unknown)),
            Expr::Ident(name) => {
                self.local_var_types.get(name).cloned().unwrap_or(Type::Unknown)
            }
            _ => Type::Unknown,
        }
    }

    /// Check if a type is a generic type parameter (e.g., T in fn foo<T>)
    /// The parser creates synthetic TypeDecl stubs for generic params — they have
    /// no members, no methods, and kind UserType. Real type declarations always
    /// have at least members or are registered differently.
    fn is_generic_param(&self, ty: &Type, _func: &Fn) -> bool {
        if let Type::User(type_decl) = ty {
            // Check func.type_params first (set by fn expression path)
            for tp in &_func.type_params {
                if tp.name == type_decl.name {
                    return true;
                }
            }
            // Heuristic: a synthetic TypeDecl for a generic type parameter has empty
            // members & methods. But Godot builtin types (Vector2, Color, Node, ...)
            // are also opaque here, so exempt them — their annotations must survive.
            // (Plan 306 Phase 2c)
            if type_decl.members.is_empty()
                && type_decl.methods.is_empty()
                && !is_godot_builtin_type(&type_decl.name)
            {
                return true;
            }
        }
        false
    }

    /// Check if a type matches one of the TypeDecl's generic type params
    fn is_type_decl_generic_param(&self, ty: &Type, generic_params: &[GenericParam]) -> bool {
        if let Type::User(type_decl) = ty {
            for gp in generic_params {
                match gp {
                    GenericParam::Type(tp) => {
                        if tp.name == type_decl.name {
                            return true;
                        }
                    }
                    GenericParam::Const(_) => {}
                }
            }
            // Heuristic fallback: synthetic TypeDecl with empty members/methods,
            // but exempt Godot builtin types so their annotations survive.
            // (Plan 306 Phase 2c)
            if type_decl.members.is_empty()
                && type_decl.methods.is_empty()
                && !is_godot_builtin_type(&type_decl.name)
            {
                return true;
            }
        }
        false
    }
}

/// Whether a type name is a Godot builtin class (so its Type::User form, even
/// with an opaque/empty TypeDecl, is concrete and must keep its annotation
/// rather than being mistaken for a generic type parameter). (Plan 306 Phase 2c)
fn is_godot_builtin_type(name: &str) -> bool {
    matches!(
        name,
        // Math types
        "Vector2" | "Vector2i" | "Vector3" | "Vector3i" | "Vector4" | "Vector4i"
            | "Color" | "Rect2" | "Rect2i" | "AABB" | "Plane" | "Quaternion"
            | "Basis" | "Transform2D" | "Transform3D" | "Projection"
            // Core nodes
            | "Node" | "Node2D" | "Node3D" | "Control" | "CanvasItem"
            | "Sprite2D" | "AnimatedSprite2D" | "Area2D" | "Area3D"
            | "RigidBody2D" | "RigidBody3D" | "CharacterBody2D" | "CharacterBody3D"
            | "CollisionShape2D" | "CollisionShape3D" | "CollisionPolygon2D"
            | "Timer" | "Label" | "Button" | "Line2D" | "Marker2D" | "Marker3D"
            | "Camera2D" | "Camera3D" | "PathFollow2D" | "Path2D"
            // Resources
            | "Resource" | "PackedScene" | "Texture2D" | "SpriteFrames"
            | "Shader" | "Material" | "StandardMaterial3D" | "FontFile" | "Font"
            | "AudioStream" | "StyleBox" | "Theme" | "Curve" | "Curve2D"
            | "InputEvent" | "RID" | "StringName" | "Callable" | "Signal"
            | "Tween" | "RefCounted" | "Object" | "Array" | "Dictionary"
            | "PackedStringArray" | "PackedInt32Array" | "PackedFloat32Array"
    )
}

/// Capitalize the first letter of a string
fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
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

        // Plan 306 Phase 2b: when the file declares a `scene`, the generated
        // script should extend the scene's root node type (e.g. Control) so the
        // .gd attaches to the .tscn root correctly. Defaults to Node otherwise.
        // Plan 306: also collect the scene's signal declarations.
        let mut scene_root: Option<String> = None;
        let mut signals: Vec<crate::ast::SceneSignal> = Vec::new();
        for s in &ast.stmts {
            if let Stmt::SceneDecl(scene) = s {
                if scene_root.is_none() {
                    scene_root = Some(scene.node_type.to_string());
                }
                signals.extend(scene.signals.iter().cloned());
            }
        }

        // Split into declarations, main statements, and use statements
        let mut decls: Vec<(Stmt, usize)> = Vec::new();
        let mut main_stmts: Vec<(Stmt, usize)> = Vec::new();

        let source_lines = ast.source_lines;
        // Plan 306 Phase 3: script-level annotations (@tool, @icon(...))
        let file_attrs = ast.file_attrs;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Skip main function — handled specially
            if let Stmt::Fn(func) = &stmt {
                if func.name == "main" {
                    continue;
                }
            }
            // Phase 1: Collect use statements for import emission
            if let Stmt::Use(use_stmt) = &stmt {
                self.handle_use(use_stmt);
                continue;
            }
            // Plan 306: SceneDecl carries only scene metadata (root type → `extends`,
            // signal declarations) already extracted above; it emits no .gd body, so
            // skip it here to avoid polluting main_stmts (which would add an empty
            // `func _ready():` stub).
            if matches!(stmt, Stmt::SceneDecl(_)) {
                continue;
            }
            // EmptyLine carries no code — skip so it doesn't pollute main_stmts
            // (which would add a spurious empty `func _ready():` stub).
            if matches!(stmt, Stmt::EmptyLine(_)) {
                continue;
            }

            if stmt.is_decl() {
                decls.push((stmt, line));
            } else {
                main_stmts.push((stmt, line));
            }
        }

        // Phase 2: Generate code body into temporary buffer
        let mut code_buf: Vec<u8> = Vec::new();

        // Emit declarations (types, enums, non-main functions)
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.set_source_line(*line);
            self.stmt(decl, &mut code_buf)?;
            if i < decls.len() - 1 {
                code_buf.write(b"\n")?;
            }
        }

        // Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            if !decls.is_empty() {
                code_buf.write(b"\n")?;
            }
            self.stmt(&main_stmt, &mut code_buf)?;
        } else if !main_stmts.is_empty() {
            if !decls.is_empty() {
                code_buf.write(b"\n")?;
            }
            code_buf.write(b"func _ready():\n")?;
            self.indent();
            for (stmt, line) in &main_stmts {
                sink.set_source_line(*line);
                self.stmt(stmt, &mut code_buf)?;
            }
            self.dedent();
        }

        // Phase 3: Assemble final output
        // 1. File header
        write!(sink.body, "# Auto-generated from {}.at — do not edit\n\n", self.name)?;

        // 1b. Script-level annotations (@tool, @icon(...)) — Godot requires these
        // before `extends`. Emitted verbatim from #[tool]/#[icon(...)].
        if !file_attrs.is_empty() {
            for a in &file_attrs {
                write!(sink.body, "@{}\n", a.as_str())?;
            }
            sink.body.write(b"\n")?;
        }

        // 2. extends <root node type> (scene root, or Node by default)
        let base = scene_root.as_deref().unwrap_or("Node");
        write!(sink.body, "extends {}\n\n", base)?;

        // 2b. Signal declarations (`signal name(p: T)`), right after `extends`.
        if !signals.is_empty() {
            for sig in &signals {
                if sig.params.is_empty() {
                    write!(sink.body, "signal {}\n", sig.name)?;
                } else {
                    let params = sig.params.iter().map(|p| {
                        format!("{}: {}", p.name, self.gdscript_type_name(&p.ty))
                    }).collect::<Vec<_>>().join(", ");
                    write!(sink.body, "signal {}({})\n", sig.name, params)?;
                }
            }
            sink.body.write(b"\n")?;
        }

        // 3. Emit collected imports
        self.emit_imports(&mut sink.body)?;

        // 4. Append code body
        sink.body.write(&code_buf)?;

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

    #[test]
    fn test_import() {
        test_a2gd("14_modules/001_import").unwrap();
    }

    #[test]
    fn test_typed_vars() {
        test_a2gd("01_basics/031_typed_vars").unwrap();
    }

    #[test]
    fn test_generic_func() {
        test_a2gd("08_generics/001_generic_func").unwrap();
    }

    #[test]
    fn test_generic_struct() {
        test_a2gd("08_generics/002_generic_struct").unwrap();
    }

    #[test]
    fn test_async_func() {
        test_a2gd("03_control_flow/040_async_func").unwrap();
    }

    #[test]
    fn test_basic_spec() {
        test_a2gd("12_specs/001_basic_spec").unwrap();
    }

    // Task 9: Expanded test suite

    #[test]
    fn test_comments() { test_a2gd("01_basics/040_comments").unwrap(); }

    // Plan 306 Phase 2c: Godot builtin types keep their annotations.
    #[test]
    fn test_godot_vector2_sig() { test_a2gd("17_godot_types/001_vector2").unwrap(); }

    // Plan 306 Phase 2c: #[export] var → GDScript @export var.
    #[test]
    fn test_godot_export() { test_a2gd("17_godot_types/002_export").unwrap(); }

    // Plan 306 Phase 3: extended annotations (@onready, @export_range, @export_group).
    #[test]
    fn test_godot_annotations() { test_a2gd("17_godot_types/003_annot").unwrap(); }

    // Plan 306 Phase 3: script-level annotations (@tool, @icon) emitted before `extends`.
    #[test]
    fn test_godot_script_annotations() { test_a2gd("17_godot_types/004_tool").unwrap(); }

    // Plan 306: signal declarations inside a scene → `signal name(p: T)` in .gd.
    #[test]
    fn test_godot_signals() { test_a2gd("17_godot_types/005_signal").unwrap(); }

    // Plan 306 Phase 5a: typed collections Array[T]/Dictionary[K,V].
    #[test]
    fn test_godot_typed_collections() { test_a2gd("17_godot_types/006_typed").unwrap(); }

    // Plan 306 Phase 5b: enum explicit values + source-casing preservation.
    #[test]
    fn test_godot_enum_values() { test_a2gd("17_godot_types/007_enum").unwrap(); }

    #[test]
    fn test_unary_neg() { test_a2gd("01_basics/041_unary_neg").unwrap(); }

    #[test]
    fn test_unary_not() { test_a2gd("01_basics/042_unary_not").unwrap(); }

    #[test]
    fn test_const_decl() { test_a2gd("01_basics/044_const_decl").unwrap(); }

    #[test]
    fn test_boolean_ops() { test_a2gd("01_basics/046_boolean_ops").unwrap(); }

    #[test]
    fn test_arithmetic() { test_a2gd("01_basics/047_arithmetic").unwrap(); }

    #[test]
    fn test_lambda() { test_a2gd("05_expressions/010_lambda").unwrap(); }

    #[test]
    fn test_object() { test_a2gd("05_expressions/021_object").unwrap(); }

    #[test]
    fn test_chained_method() { test_a2gd("05_expressions/032_chained_method").unwrap(); }

    #[test]
    fn test_option() { test_a2gd("09_option_result/001_option").unwrap(); }

    #[test]
    fn test_result_ok() { test_a2gd("09_option_result/003_result_ok").unwrap(); }

    // Cookbook tests from learn-gdscript (GDQuest)
    #[test]
    fn test_cb_health_var() { test_a2gd("cookbook/01_variables/001_health_var").unwrap(); }

    #[test]
    fn test_cb_angular_speed() { test_a2gd("cookbook/01_variables/002_angular_speed").unwrap(); }

    #[test]
    fn test_cb_take_damage() { test_a2gd("cookbook/02_arithmetic/001_take_damage").unwrap(); }

    #[test]
    fn test_cb_heal() { test_a2gd("cookbook/02_arithmetic/002_heal").unwrap(); }

    #[test]
    fn test_cb_level_up() { test_a2gd("cookbook/02_arithmetic/003_level_up").unwrap(); }

    #[test]
    fn test_cb_damage_reduction() { test_a2gd("cookbook/02_arithmetic/004_damage_reduction").unwrap(); }

    #[test]
    fn test_cb_comparisons() { test_a2gd("cookbook/03_conditions/001_comparisons").unwrap(); }

    #[test]
    fn test_cb_limit_health() { test_a2gd("cookbook/03_conditions/002_limit_health").unwrap(); }

    #[test]
    fn test_cb_prevent_zero() { test_a2gd("cookbook/03_conditions/003_prevent_zero").unwrap(); }

    #[test]
    fn test_cb_for_range() { test_a2gd("cookbook/04_loops/001_for_range").unwrap(); }

    #[test]
    fn test_cb_while_move() { test_a2gd("cookbook/04_loops/002_while_move").unwrap(); }

    #[test]
    fn test_cb_for_each() { test_a2gd("cookbook/04_loops/003_for_each").unwrap(); }

    #[test]
    fn test_cb_array_create() { test_a2gd("cookbook/05_arrays/001_create").unwrap(); }

    #[test]
    fn test_cb_append_pop() { test_a2gd("cookbook/05_arrays/002_append_pop").unwrap(); }

    #[test]
    fn test_cb_index_access() { test_a2gd("cookbook/05_arrays/003_index_access").unwrap(); }

    #[test]
    fn test_cb_string_concat() { test_a2gd("cookbook/06_strings/001_concat").unwrap(); }

    #[test]
    fn test_cb_string_array() { test_a2gd("cookbook/06_strings/002_string_array").unwrap(); }

    #[test]
    fn test_cb_return_value() { test_a2gd("cookbook/07_functions/001_return_value").unwrap(); }

    // Plan 308: reverse-translated Godot demo scripts.
    #[test]
    fn test_godot_demo_instancing_ball_factory() {
        test_a2gd("tscn/godot_demos/instancing/002_ball_factory").unwrap();
    }

    #[test]
    fn test_godot_demo_hexagonal_troll() {
        test_a2gd("tscn/godot_demos/hexagonal_map/001_troll").unwrap();
    }

    #[test]
    fn test_godot_demo_kinematic_player() {
        test_a2gd("tscn/godot_demos/kinematic_character/001_player").unwrap();
    }

    #[test]
    fn test_cb_dict_create() { test_a2gd("cookbook/08_dictionaries/001_create").unwrap(); }

    #[test]
    fn test_cb_dict_loop() { test_a2gd("cookbook/08_dictionaries/002_loop").unwrap(); }

    #[test]
    fn test_cb_type_conversion() { test_a2gd("cookbook/09_types/001_conversion").unwrap(); }
}
