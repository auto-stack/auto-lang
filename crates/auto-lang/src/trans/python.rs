use super::{escape_str, Sink, Trans, ToStrError};
use crate::ast::*;
use crate::AutoResult;
use auto_val::AutoStr;
use auto_val::Op;
use std::collections::{HashMap, HashSet};
use std::io::Write;

/// Plan 283 Task 4.2: Tracked third-party Python package dependency
#[derive(Debug, Clone)]
pub struct PyDep {
    pub name: AutoStr,
    pub version: AutoStr,
}

pub struct PythonTrans {
    indent: usize,
    /// Typing/dataclass imports (Optional, Protocol, dataclass, Enum, etc.)
    imports: HashSet<AutoStr>,
    /// Python module imports collected from `use` / `use.py` statements
    /// Each entry: (module_path, imported_items) — items empty means `import module`
    py_imports: Vec<(AutoStr, Vec<AutoStr>)>,
    /// Python wildcard imports (e.g., `from module import *`)
    py_wildcards: Vec<AutoStr>,
    /// Third-party Python package dependencies (for requirements.txt generation)
    /// Plan 283 Task 4.2
    #[allow(dead_code)] // planned future-use (Plan 283); not yet emitted
    py_deps: Vec<PyDep>,
    /// Plan 283 Task 2.1: Local variable type tracking for ErrorPropagate and type-aware codegen.
    /// Populated from store.ty (explicit annotations) and basic expression inference.
    local_var_types: HashMap<AutoStr, Type>,
    #[allow(dead_code)]
    name: AutoStr,
}

impl PythonTrans {
    pub fn new(name: AutoStr) -> Self {
        Self {
            indent: 0,
            imports: HashSet::new(),
            py_imports: Vec::new(),
            py_wildcards: Vec::new(),
            py_deps: Vec::new(),
            local_var_types: HashMap::new(),
            name,
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
            Expr::Bool(b) => write!(out, "{}", if *b { "True" } else { "False" }).map_err(Into::into),
            Expr::Char(c) => write!(out, "'{}'", c).map_err(Into::into),
            Expr::Str(s) => write!(out, "\"{}\"", escape_str(s)).map_err(Into::into),
            Expr::CStr(s) => write!(out, "\"{}\"", s).map_err(Into::into),

            // F-strings (direct mapping - AutoLang and Python have identical syntax!)
            Expr::FStr(fstr) => self.fstr(fstr, out),

            // Identifiers
            Expr::Ident(name) => out.write_all(name.as_bytes()).map_err(Into::into),

            // Binary operations
            Expr::Bina(lhs, op, rhs) => {
                match op {
                    Op::Dot => self.dot(lhs, rhs, out),
                    _ => {
                        self.expr(lhs, out)?;
                        // Python uses 'and'/'or' keywords, not '&&'/'||'
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

            // Plan 056: Dot expression for field access
            Expr::Dot(object, field) => {
                // Python uses . for all field access (including pointers)
                self.expr(object, out)?;
                out.write_all(b".")?;
                out.write_all(field.as_bytes())?;
                Ok(())
            }

            // Unary operations
            Expr::Unary(op, expr) => {
                // Python uses 'not' keyword instead of '!'
                match op {
                    Op::Not => {
                        out.write(b"not ")?;
                        self.expr(expr, out)
                    }
                    _ => {
                        out.write(format!("{}", op.op()).as_bytes()).to()?;
                        self.expr(expr, out)
                    }
                }
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

            // Plan 213: Option/Result constructors
            // Some(x) -> x (in Python, values are just values)
            Expr::Some(e) => self.expr(e, out),
            // None -> None
            Expr::None => out.write(b"None").to(),
            // Ok(x) -> x
            Expr::Ok(e) => self.expr(e, out),
            // Err(msg) -> raise Exception(msg) - but as expression, just emit the value
            Expr::Err(e) => {
                write!(out, "Exception(")?;
                self.expr(e, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Plan 213: Null coalescing - x ?? default -> x if x is not None else default
            Expr::NullCoalesce(lhs, rhs) => {
                write!(out, "(")?;
                self.expr(lhs, out)?;
                write!(out, " if ")?;
                self.expr(lhs, out)?;
                write!(out, " is not None else ")?;
                self.expr(rhs, out)?;
                write!(out, ")").map_err(Into::into)
            }

            // Plan 213: Closure/lambda - (x) => expr -> lambda x: expr
            Expr::Closure(closure) => {
                write!(out, "lambda ")?;
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    out.write_all(param.name.as_bytes())?;
                }
                write!(out, ": ")?;
                self.expr(&closure.body, out)
            }

            // Plan 213: Tuple - (a, b) -> (a, b)
            Expr::Tuple(elems) => {
                write!(out, "(")?;
                for (i, elem) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ")?;
                    }
                    self.expr(elem, out)?;
                }
                // Single-element tuple needs trailing comma
                if elems.len() == 1 {
                    write!(out, ",")?;
                }
                write!(out, ")").map_err(Into::into)
            }

            // Plan 213: Object literal - {key: value} -> {"key": value}
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

            // Plan 213: Option pattern in is branches
            Expr::OptionPattern(cover) => {
                match cover.variant {
                    crate::ast::cover::OptionVariant::Some => {
                        if let Some(ref binding) = cover.binding {
                            // Some(x) -> capture pattern like ("some", x)
                            write!(out, "SomeCase({})", binding).map_err(Into::into)
                        } else {
                            write!(out, "SomeCase(_)").map_err(Into::into)
                        }
                    }
                    crate::ast::cover::OptionVariant::None => {
                        write!(out, "None").map_err(Into::into)
                    }
                }
            }

            // Plan 213: Result pattern in is branches
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

            // nil/Null -> None
            Expr::Nil | Expr::Null => out.write(b"None").to(),

            // Plan 213 Task 5: Error propagate x.? -> x (Python doesn't have ?, just pass through)
            Expr::ErrorPropagate(e) => self.expr(e, out),

            // Plan 213 Task 7: Await expression expr.await -> await expr
            Expr::Await { expr } => {
                out.write(b"await ")?;
                self.expr(expr, out)
            }

            // Plan 213 Task 7: Go expression expr.go -> just spawn (fire-and-forget)
            Expr::Go { expr } => {
                // Python doesn't have spawn, just emit as expression
                self.expr(expr, out)
            }

            // Plan 165: Struct destructuring pattern for is match arms
            // Point { x, y } → Python 3.10+ dataclass pattern: case Point(x, y)
            Expr::StructPattern(sc) => {
                match &sc.variant {
                    Some(variant) => {
                        // Enum variant with struct destructuring: Type.Variant { field }
                        out.write_all(variant.as_bytes())?;
                    }
                    None => {
                        // Plain struct destructuring: Type { x, y }
                        out.write_all(sc.type_name.as_bytes())?;
                    }
                }
                out.write(b"(")?;
                for (i, fb) in sc.fields.iter().enumerate() {
                    if i > 0 {
                        out.write(b", ")?;
                    }
                    if fb.field == fb.binding {
                        // Shorthand: field name equals binding name
                        out.write_all(fb.field.as_bytes())?;
                    } else {
                        // Explicit alias: field=binding
                        out.write_all(fb.field.as_bytes())?;
                        out.write(b"=")?;
                        out.write_all(fb.binding.as_bytes())?;
                    }
                }
                out.write(b")")?;
                Ok(())
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

            // Continue statement
            Stmt::Continue => {
                self.print_indent(out)?;
                out.write(b"continue\n")?;
                Ok(true)
            }

            // Return statement
            Stmt::Return(expr) => {
                self.print_indent(out)?;
                out.write(b"return ")?;
                self.expr(expr, out)?;
                out.write(b"\n")?;
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

            // Plan 213 Task 8: Spec declarations → Protocol
            Stmt::SpecDecl(spec_decl) => {
                self.spec_decl(spec_decl, out)?;
                Ok(true)
            }

            // Plan 213 Task 9: Union declarations → dataclass
            Stmt::Union(union_decl) => {
                self.union_decl(union_decl, out)?;
                Ok(true)
            }

            // Plan 213 Task 9: Tag declarations → dataclass with factory methods
            Stmt::Tag(tag_decl) => {
                self.tag_decl(tag_decl, out)?;
                Ok(true)
            }

            // Skip alias for now
            Stmt::Alias(_) => Ok(false),

            // Plan 283 Task 1.1 + 4.1: use / use.py → Python import
            Stmt::Use(use_stmt) => {
                self.handle_use(use_stmt);
                Ok(false) // imports are collected, emitted at top of file
            }

            _ => Err(format!("Python Transpiler: unsupported statement: {:?}", stmt).into()),
        }
    }

    fn store(&mut self, store: &Store, out: &mut impl Write) -> AutoResult<()> {
        // Plan 283 Task 2.1: Track variable type for type-aware codegen
        let effective_ty = if matches!(store.ty, Type::Unknown) {
            self.infer_type_from_expr(&store.expr)
        } else {
            store.ty.clone()
        };
        self.local_var_types.insert(store.name.clone(), effective_ty);

        out.write_all(store.name.as_bytes())?;
        out.write(b" = ")?;
        self.expr(&store.expr, out)?;
        Ok(())
    }

    /// Check if a function return type is Future (~T), meaning the function is async
    fn is_async_fn(&self, func: &Fn) -> bool {
        matches!(&func.ret, Type::GenericInstance(inst) if inst.base_name == "Future")
            || matches!(&func.ret, Type::Handle { .. })
    }

    /// Plan 213 Task 7: Check if a function body contains .await expressions
    fn has_await(stmts: &[Stmt]) -> bool {
        for stmt in stmts {
            match stmt {
                Stmt::Expr(expr) if Self::expr_has_await(expr) => return true,
                Stmt::Store(store) if Self::expr_has_await(&store.expr) => return true,
                Stmt::Return(expr) if Self::expr_has_await(expr) => return true,
                _ => {}
            }
        }
        false
    }

    fn expr_has_await(expr: &Expr) -> bool {
        match expr {
            Expr::Await { .. } => true,
            Expr::Call(call) => Self::expr_has_await(call.name.as_ref()),
            Expr::Bina(lhs, _, rhs) => Self::expr_has_await(lhs) || Self::expr_has_await(rhs),
            _ => false,
        }
    }

    /// Plan 213 Task 6: Check if a type is a generic type parameter (e.g., T in fn foo<T>)
    fn is_generic_param(&self, ty: &Type, func: &Fn) -> bool {
        if let Type::User(type_decl) = ty {
            // Check if this type name matches any of the function's type params
            for tp in &func.type_params {
                if tp.name == type_decl.name {
                    return true;
                }
            }
        }
        false
    }

    fn fn_decl(&mut self, func: &Fn, out: &mut impl Write) -> AutoResult<()> {
        // Plan 283 Task 2.1: Clear and populate local_var_types from function params
        self.local_var_types.clear();
        for param in &func.params {
            if !matches!(param.ty, Type::Unknown) {
                self.local_var_types.insert(param.name.clone(), param.ty.clone());
            }
        }

        self.print_indent(out)?;

        // Plan 213 Task 7: async def for ~T return types, or main with .await
        let is_async = self.is_async_fn(func)
            || (func.name == "main" && Self::has_await(&func.body.stmts));
        if is_async {
            out.write(b"async ")?;
        }

        out.write(b"def ")?;
        out.write_all(func.name.as_bytes())?;
        // Plan 213 Task 6: Skip generic type params (<T> erased in Python)
        out.write(b"(")?;

        // Plan 213 Task 4: Parameters with type annotations
        // Plan 213 Task 6: Skip type annotations for generic params (type erasure)
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            out.write_all(param.name.as_bytes())?;
            // Add type annotation if known and not a generic param
            if !matches!(param.ty, Type::Unknown) && !self.is_generic_param(&param.ty, func) {
                out.write(b": ")?;
                let type_name = self.python_type_name(&param.ty);
                out.write_all(type_name.as_bytes())?;
                // Track Optional import
                if matches!(param.ty, Type::Option(_)) {
                    self.imports.insert("Optional".into());
                }
            }
        }

        // Plan 213 Task 4: Return type annotation
        let ret_type_for_annotation = self.fn_return_type_for_annotation(func);
        if let Some(ret_type_str) = &ret_type_for_annotation {
            out.write(b") -> ")?;
            out.write_all(ret_type_str.as_bytes())?;
            out.write(b":\n")?;
        } else {
            out.write(b"):\n")?;
        }

        // Track Optional import for return type
        if matches!(&func.ret, Type::Option(_)) {
            self.imports.insert("Optional".into());
        }
        if matches!(&func.ret, Type::Result(_)) {
            self.imports.insert("Result".into());
        }

        self.indent();

        // Check if function has a non-void return type (except main)
        let has_return = !matches!(func.ret, Type::Unknown | Type::Void) && func.name != "main";
        // Async functions also need return on last expression
        let is_async = self.is_async_fn(func);

        // Process body statements
        if (has_return || is_async) && !func.body.stmts.is_empty() {
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

    /// Get the Python return type annotation string for a function.
    /// Returns None if no annotation should be emitted (Unknown type, main, void, generic param).
    fn fn_return_type_for_annotation(&self, func: &Fn) -> Option<AutoStr> {
        // Don't annotate main or functions with unknown/void return
        if func.name == "main" {
            return None;
        }
        match &func.ret {
            Type::Unknown | Type::Void => None,
            // Plan 213 Task 6: Skip generic type params (type erasure)
            _ if self.is_generic_param(&func.ret, func) => None,
            _ => Some(self.python_type_name(&func.ret)),
        }
    }

    fn fn_decl_in_class(&mut self, func: &Fn, _type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        // Plan 283 Task 2.1: Clear and populate local_var_types from method params
        self.local_var_types.clear();
        for param in &func.params {
            if !matches!(param.ty, Type::Unknown) {
                self.local_var_types.insert(param.name.clone(), param.ty.clone());
            }
        }

        self.print_indent(out)?;

        // Plan 283 Task 2.2: Static methods get @staticmethod decorator
        let is_static = func.is_static;
        if is_static {
            out.write(b"@staticmethod\n")?;
            self.print_indent(out)?;
        }

        // Plan 213 Task 7: async def for ~T return types
        if self.is_async_fn(func) {
            out.write(b"async ")?;
        }

        out.write(b"def ")?;
        out.write_all(func.name.as_bytes())?;
        // Plan 283 Task 2.2: Static methods don't have self parameter
        if !is_static {
            out.write(b"(self")?;
        } else {
            out.write(b"(")?;
        }

        // Plan 213 Task 4: Parameters with type annotations
        for (i, param) in func.params.iter().enumerate() {
            // Add comma separator: instance methods already have "self", static methods need comma after first param
            if !is_static || i > 0 {
                out.write(b", ")?;
            } else {
                // First param of static method — no preceding comma needed
                // (we opened with "(" above, so just write the param)
            }
            out.write_all(param.name.as_bytes())?;
            // Add type annotation if known
            if !matches!(param.ty, Type::Unknown) {
                out.write(b": ")?;
                let type_name = self.python_type_name(&param.ty);
                out.write_all(type_name.as_bytes())?;
                if matches!(param.ty, Type::Option(_)) {
                    self.imports.insert("Optional".into());
                }
            }
        }

        // Plan 213 Task 4: Return type annotation
        let ret_type_for_annotation = self.fn_return_type_for_annotation(func);
        if let Some(ret_type_str) = &ret_type_for_annotation {
            out.write(b") -> ")?;
            out.write_all(ret_type_str.as_bytes())?;
            out.write(b":\n")?;
        } else {
            out.write(b"):\n")?;
        }

        self.indent();

        // Check if function has a non-void return type
        let has_return = !matches!(func.ret, Type::Unknown | Type::Void);
        let is_async = self.is_async_fn(func);

        // Process body statements
        if (has_return || is_async) && !func.body.stmts.is_empty() {
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
        // Plan 213 Task 10: Handle conditional for loop (while in Python)
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
            Iter::Call(_call) => {
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
                IsBranch::EqBranch(patterns, body) => {
                    for (i, pat) in patterns.iter().enumerate() {
                        if i == 0 { out.write(b"case ")?; }
                        else { out.write(b" | ")?; }
                        self.expr(pat, out)?;
                    }
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
        // Plan 283 Task 1.3: Intercept method calls where call.name is Expr::Dot.
        // Parser generates Expr::Call { name: Expr::Dot(obj, method_name), args: [...] }
        // for method calls like items.push(4), "hello".trim(), etc.
        // Note: Expr::Dot(Box<Expr>, Name) — Name is already AutoStr, not Expr.
        if let Expr::Dot(obj, method_name) = call.name.as_ref() {
            return self.method_call(obj, method_name, &call.args, out);
        }

        // Plan 283 Task 1.2: Map AutoLang builtins to Python stdlib equivalents
        if let Some(ident) = self.extract_call_name(&call.name) {
            match ident.as_ref() {
                // Identical in Python — just pass through
                "print" | "len" | "range" | "type" | "abs" | "min" | "max" | "sum"
                | "sorted" | "reversed" | "enumerate" | "zip" | "map" | "filter"
                | "isinstance" | "hasattr" | "getattr" | "setattr" => {
                    return self.emit_plain_call(call, out);
                }
                // type_name(x) → type(x).__name__
                "type_name" => {
                    out.write(b"type(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    out.write(b").__name__")?;
                    return Ok(());
                }
                // sleep_ms(ms) → time.sleep(ms / 1000)
                "sleep_ms" => {
                    self.py_imports.push(("time".into(), Vec::new()));
                    out.write(b"time.sleep(")?;
                    if let Some(arg) = call.args.args.first() {
                        self.arg(arg, out)?;
                    }
                    out.write(b" / 1000)")?;
                    return Ok(());
                }
                // time_now() → time.time()
                "time_now" => {
                    self.py_imports.push(("time".into(), Vec::new()));
                    out.write(b"time.time()")?;
                    return Ok(());
                }
                _ => {}
            }
        }
        self.emit_plain_call(call, out)
    }

    /// Extract a plain identifier name from a call expression (e.g., Expr::Ident("foo"))
    fn extract_call_name(&self, expr: &Expr) -> Option<AutoStr> {
        match expr {
            Expr::Ident(name) => Some(name.clone()),
            _ => None,
        }
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
        // Plan 283 Task 1.3: Intercept method calls for Pythonic mapping
        // Method call pattern: lhs.method(args) → rhs is Expr::Call
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

    /// Plan 283 Task 1.3: Map AutoLang method calls to Pythonic equivalents
    fn method_call(
        &mut self,
        receiver: &Expr,
        method: &AutoStr,
        args: &Args,
        out: &mut impl Write,
    ) -> AutoResult<()> {
        match method.as_ref() {
            // ── List methods ──
            // .push(item) → .append(item)
            "push" => {
                self.expr(receiver, out)?;
                out.write(b".append(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .pop() → .pop()
            "pop" => {
                self.expr(receiver, out)?;
                out.write(b".pop(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .len() → len(receiver)
            "len" => {
                out.write(b"len(")?;
                self.expr(receiver, out)?;
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
            // .join(sep) → sep.join(receiver)  (Python string.join takes iterable)
            "join" => {
                // Auto: list.join(sep) → Python: sep.join(list)
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
            // .set(key, val) → receiver[key] = val  (as statement)
            // Note: this only works as a statement, not expression. For now emit as method.
            "set" | "insert" => {
                // Emit as dict[key] = val only when used as statement
                // As expression fallback: dict.__setitem__(key, val)
                self.expr(receiver, out)?;
                out.write(b"[")?;
                if let Some(first) = args.args.first() {
                    self.arg(first, out)?;
                }
                out.write(b"] = ")?;
                if args.args.len() > 1 {
                    self.arg(&args.args[1], out)?;
                } else {
                    out.write(b"None")?;
                }
            }
            // .get(key) → receiver.get(key) (same in Python)
            "get" => {
                self.expr(receiver, out)?;
                out.write(b".get(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .has(key) → key in receiver
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
            // .keys() / .values() / .items() — pass through
            "keys" | "values" | "items" => {
                self.expr(receiver, out)?;
                out.write(b".")?;
                out.write_all(method.as_bytes())?;
                out.write(b"(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }

            // ── String methods ──
            // .trim() → .strip()
            "trim" => {
                self.expr(receiver, out)?;
                out.write(b".strip(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .split(sep) → .split(sep) (same in Python)
            "split" => {
                self.expr(receiver, out)?;
                out.write(b".split(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .to_upper() → .upper()
            "to_upper" | "upper" => {
                self.expr(receiver, out)?;
                out.write(b".upper()")?;
            }
            // .to_lower() → .lower()
            "to_lower" | "lower" => {
                self.expr(receiver, out)?;
                out.write(b".lower()")?;
            }
            // .starts_with(s) → .startswith(s)
            "starts_with" | "startswith" => {
                self.expr(receiver, out)?;
                out.write(b".startswith(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .ends_with(s) → .endswith(s)
            "ends_with" | "endswith" => {
                self.expr(receiver, out)?;
                out.write(b".endswith(")?;
                self.emit_args(args, out)?;
                out.write(b")")?;
            }
            // .replace(old, new) → .replace(old, new) (same in Python)
            "replace" => {
                self.expr(receiver, out)?;
                out.write(b".replace(")?;
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

    /// Emit call arguments
    fn emit_args(&mut self, args: &Args, out: &mut impl Write) -> AutoResult<()> {
        for (i, arg) in args.args.iter().enumerate() {
            if i > 0 {
                out.write(b", ")?;
            }
            self.arg(arg, out)?;
        }
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

    fn key(&mut self, key: &Key, out: &mut impl Write) -> AutoResult<()> {
        match key {
            Key::NamedKey(name) => {
                // Named keys become string keys in Python: x -> "x"
                write!(out, "\"{}\"", name)?;
            }
            Key::IntKey(i) => write!(out, "{}", i)?,
            Key::BoolKey(b) => write!(out, "{}", b)?,
            Key::StrKey(s) => write!(out, "\"{}\"", s)?,
        }
        Ok(())
    }
    /// Plan 213 Task 6: Check if a type is a generic param of a TypeDecl
    fn is_type_decl_generic_param(&self, ty: &Type, type_decl: &TypeDecl) -> bool {
        if let Type::User(user_td) = ty {
            for gp in &type_decl.generic_params {
                match gp {
                    super::super::ast::GenericParam::Type(tp) => {
                        if tp.name == user_td.name {
                            return true;
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    fn type_decl(&mut self, type_decl: &TypeDecl, out: &mut impl Write) -> AutoResult<()> {
        // Plan 283 Task 3.1: Always use @dataclass for consistency and Pythonic output
        self.print_indent(out)?;
        out.write(b"@dataclass\n")?;

        // Plan 213 Task 6: Skip generic type params (<T> erased in Python)
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(type_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // Emit fields using @dataclass style (field: type)
        for member in &type_decl.members {
            self.print_indent(out)?;
            out.write_all(member.name.as_bytes())?;
            out.write(b": ")?;
            // Plan 213 Task 6: Use Any for generic param types
            if self.is_type_decl_generic_param(&member.ty, type_decl) {
                out.write(b"Any")?;
            } else {
                let type_name = self.python_type_name(&member.ty);
                out.write_all(type_name.as_bytes())?;
            }
            out.write(b"\n")?;
        }

        // Python requires at least one statement in class body
        let is_empty = type_decl.members.is_empty() && type_decl.methods.is_empty();
        if is_empty {
            self.print_indent(out)?;
            out.write(b"pass\n")?;
        }

        // Emit methods (add blank line before first method)
        for method in type_decl.methods.iter() {
            out.write(b"\n")?;
            self.fn_decl_in_class(method, type_decl, out)?;
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

    /// Plan 213 Task 8: spec → Protocol
    /// spec Comparable { fn compare(self, other Self) int }
    /// →
    /// class Comparable(Protocol):
    ///     def compare(self, other: 'Comparable') -> int: ...
    fn spec_decl(&mut self, spec_decl: &SpecDecl, out: &mut impl Write) -> AutoResult<()> {
        self.imports.insert("Protocol".into());

        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(spec_decl.name.as_bytes())?;
        out.write(b"(Protocol):\n")?;
        self.indent();

        for method in &spec_decl.methods {
            self.print_indent(out)?;
            out.write(b"def ")?;
            out.write_all(method.name.as_bytes())?;
            out.write(b"(self")?;

            for param in &method.params {
                out.write(b", ")?;
                out.write_all(param.name.as_bytes())?;
                if !matches!(param.ty, Type::Unknown) {
                    out.write(b": ")?;
                    let type_name = self.python_type_name(&param.ty);
                    out.write_all(type_name.as_bytes())?;
                }
            }

            // Return type annotation
            if !matches!(method.ret, Type::Void | Type::Unknown) {
                out.write(b") -> ")?;
                let type_name = self.python_type_name(&method.ret);
                out.write_all(type_name.as_bytes())?;
            } else {
                out.write(b")")?;
            }

            out.write(b": ...\n")?;
        }

        self.dedent();
        Ok(())
    }

    /// Plan 213 Task 9: union → dataclass
    /// union MyUnion { i int, f float }
    /// →
    /// @dataclass
    /// class MyUnion:
    ///     kind: str = ''
    ///     i: int = 0
    ///     f: float = 0.0
    fn union_decl(&mut self, union_decl: &Union, out: &mut impl Write) -> AutoResult<()> {
        self.imports.insert("dataclass".into());

        self.print_indent(out)?;
        out.write(b"@dataclass\n")?;
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(union_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // kind discriminator field
        self.print_indent(out)?;
        out.write(b"kind: str = ''\n")?;

        // Fields with default values
        for field in &union_decl.fields {
            self.print_indent(out)?;
            out.write_all(field.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = self.python_type_name(&field.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b" = ")?;
            let default_val = self.python_default_value(&field.ty);
            out.write_all(default_val.as_bytes())?;
            out.write(b"\n")?;
        }

        self.dedent();
        Ok(())
    }

    /// Plan 213 Task 9: tag → dataclass with factory methods
    /// tag Shape { Circle(radius float), Rect(w float, h float) }
    /// →
    /// @dataclass
    /// class Shape:
    ///     kind: str = ''
    ///     radius: float = 0.0
    ///     w: float = 0.0
    ///     h: float = 0.0
    ///
    ///     @staticmethod
    ///     def Circle(radius): ...
    ///     @staticmethod
    ///     def Rect(w, h): ...
    fn tag_decl(&mut self, tag_decl: &Tag, out: &mut impl Write) -> AutoResult<()> {
        self.imports.insert("dataclass".into());

        self.print_indent(out)?;
        out.write(b"@dataclass\n")?;
        self.print_indent(out)?;
        out.write(b"class ")?;
        out.write_all(tag_decl.name.as_bytes())?;
        out.write(b":\n")?;
        self.indent();

        // kind discriminator field
        self.print_indent(out)?;
        out.write(b"kind: str = ''\n")?;

        // Fields with default values
        for field in &tag_decl.fields {
            self.print_indent(out)?;
            out.write_all(field.name.as_bytes())?;
            out.write(b": ")?;
            let type_name = self.python_type_name(&field.ty);
            out.write_all(type_name.as_bytes())?;
            out.write(b" = ")?;
            let default_val = self.python_default_value(&field.ty);
            out.write_all(default_val.as_bytes())?;
            out.write(b"\n")?;
        }

        // Factory methods for each variant
        // In Auto tag, each field is a variant. The factory method takes the field's value.
        // Since tag fields can be simple (just a type) or complex, we emit one factory per field.
        // Note: In the Auto parser, tag fields are parsed as TagField with name and type.
        // The factory method name is the field name, and it takes the field type as parameter.
        if !tag_decl.fields.is_empty() {
            out.write(b"\n")?;
        }
        for field in &tag_decl.fields {
            self.print_indent(out)?;
            out.write(b"@staticmethod\n")?;
            self.print_indent(out)?;
            out.write(b"def ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"(")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"):\n")?;
            self.indent();
            self.print_indent(out)?;
            out.write(b"return ")?;
            out.write_all(tag_decl.name.as_bytes())?;
            out.write(b"('")?;
            // Capitalize first letter of variant name
            let name_str = field.name.as_str();
            let mut capped = String::new();
            if let Some(first) = name_str.chars().next() {
                for (i, c) in name_str.chars().enumerate() {
                    if i == 0 {
                        capped.push(first.to_ascii_uppercase());
                    } else {
                        capped.push(c);
                    }
                }
            }
            out.write_all(capped.as_bytes())?;
            out.write(b"', ")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b"=")?;
            out.write_all(field.name.as_bytes())?;
            out.write(b")\n")?;
            self.dedent();
        }

        // Emit methods if present
        for (i, method) in tag_decl.methods.iter().enumerate() {
            if i == 0 || !tag_decl.fields.is_empty() {
                out.write(b"\n")?;
            }
            self.fn_decl_in_class_for_tag(method, tag_decl, out)?;
        }

        self.dedent();
        Ok(())
    }

    /// Helper: emit a method inside a tag class
    fn fn_decl_in_class_for_tag(&mut self, func: &Fn, _tag: &Tag, out: &mut impl Write) -> AutoResult<()> {
        // Reuse the same logic as fn_decl_in_class
        self.fn_decl_in_class(func, &TypeDecl::builtin(&_tag.name), out)
    }

    /// Helper: get Python default value for a type
    fn python_default_value(&self, ty: &Type) -> AutoStr {
        match ty {
            Type::Int | Type::Uint | Type::I64 | Type::U64 | Type::Byte => "0".into(),
            Type::Float | Type::Double => "0.0".into(),
            Type::Bool => "False".into(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice | Type::CStrLit => "\"\"".into(),
            _ => "None".into(),
        }
    }

    fn python_type_name(&self, ty: &Type) -> AutoStr {
        match ty {
            Type::Int => "int".into(),
            Type::Uint => "int".into(),
            Type::Float => "float".into(),
            Type::Double => "float".into(),
            Type::Bool => "bool".into(),
            Type::StrFixed(_) | Type::StrOwned | Type::StrSlice => "str".into(),
            Type::CStrLit => "str".into(),
            Type::User(type_decl) => type_decl.name.clone(),
            Type::Enum(enum_decl) => enum_decl.borrow().name.clone(),
            Type::List(_) => "list".into(),  // List<T> → list in Python
            Type::Map(_, _) => "dict".into(),  // Map<K, V> → dict in Python (Plan 160)
            Type::Option(inner) => format!("Optional[{}]", self.python_type_name(inner)).into(),
            Type::Result(inner) => format!("Result[{}]", self.python_type_name(inner)).into(),
            Type::GenericInstance(inst) => {
                // Future<T> -> the inner type (async handles the wrapping)
                if inst.base_name == "Future" {
                    if let Some(inner) = inst.args.first() {
                        return self.python_type_name(inner);
                    }
                }
                "Any".into()
            }
            _ => "Any".into(), // Fallback for complex types
        }
    }

    /// Collect imports needed for type annotations (Optional, Result from typing)
    fn collect_type_imports(&mut self, types: &[&Type]) {
        for ty in types {
            self.collect_type_import_for(ty);
        }
    }

    fn collect_type_import_for(&mut self, ty: &Type) {
        match ty {
            Type::Option(_) => { self.imports.insert("Optional".into()); }
            Type::Result(_) => { self.imports.insert("Result".into()); }
            Type::GenericInstance(inst) => {
                if inst.base_name == "Future" {
                    // async functions don't need special import for the type
                    // but we may want to import the inner type
                    if let Some(inner) = inst.args.first() {
                        self.collect_type_import_for(inner);
                    }
                }
            }
            _ => {}
        }
    }

    /// Plan 283 Task 2.1: Basic expression type inference for local variable tracking.
    /// Simplified version of a2r's infer_type_from_expr (rust.rs line 5611).
    /// Focuses on detecting Option/Result types for ErrorPropagate codegen.
    fn infer_type_from_expr(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Some(_) => Type::Option(Box::new(Type::Unknown)),
            Expr::None | Expr::Nil | Expr::Null => Type::Option(Box::new(Type::Unknown)),
            Expr::Ok(_) => Type::Result(Box::new(Type::Unknown)),
            Expr::Err(_) => Type::Result(Box::new(Type::Unknown)),
            Expr::Ident(name) => {
                self.local_var_types.get(name).cloned().unwrap_or(Type::Unknown)
            }
            Expr::Str(_) | Expr::CStr(_) | Expr::FStr(_) => Type::StrSlice,
            Expr::Int(_) => Type::Int,
            Expr::Uint(_) => Type::Uint,
            Expr::Float(_, _) => Type::Float,
            Expr::Double(_, _) => Type::Double,
            Expr::Bool(_) => Type::Bool,
            Expr::Array(items) => {
                if let Some(first) = items.first() {
                    let elem_ty = self.infer_type_from_expr(first);
                    Type::List(Box::new(elem_ty))
                } else {
                    Type::Unknown
                }
            }
            Expr::NullCoalesce(lhs, _) => {
                let lhs_ty = self.infer_type_from_expr(lhs);
                match lhs_ty {
                    Type::Option(inner) => *inner,
                    other => other,
                }
            }
            _ => Type::Unknown,
        }
    }

    // ── Plan 283 Task 1.1 + 4.1: use / use.py → Python import ─────────────

    /// Collect a `use` statement for later emission at the top of the file.
    /// - `UseKind::Auto` / `UseKind::Py` → Python import
    /// - `UseKind::C` / `UseKind::Rust` → skip (not relevant for Python target)
    fn handle_use(&mut self, use_stmt: &Use) {
        match use_stmt.kind {
            UseKind::C | UseKind::Rust => return, // not relevant for Python
            UseKind::Auto | UseKind::Py => {}
        }

        // Resolve the module path
        let module = if let Some(ref mp) = use_stmt.module_path {
            // New-style module_path (Plan 131)
            let display = mp.display();
            // Strip pac./super. prefixes for Python (they are AutoLang-only concepts)
            if let Some(stripped) = display.strip_prefix("pac.") {
                stripped.to_string()
            } else if let Some(stripped) = display.strip_prefix("super.") {
                stripped.to_string()
            } else {
                display.to_string()
            }
        } else if !use_stmt.paths.is_empty() {
            // Legacy paths — join with dots
            use_stmt.paths.join(".")
        } else {
            return; // no module to import from
        };

        // Collect into py_imports
        if use_stmt.is_wildcard {
            self.py_wildcards.push(module.into());
        } else if use_stmt.items.is_empty() {
            // `use json` → `import json`
            self.py_imports.push((module.into(), Vec::new()));
        } else {
            // `use json: dumps, loads` → `from json import dumps, loads`
            self.py_imports.push((module.into(), use_stmt.items.clone()));
        }
    }

    /// Emit collected Python imports to the output. Called once at the top of the file.
    fn emit_py_imports(&self, out: &mut impl Write) -> AutoResult<()> {
        // Deduplicate: collect modules seen to avoid duplicate `import time`
        let mut seen: HashSet<String> = HashSet::new();

        // Emit `import module` (no items) — from use stmts + builtin mappings
        for (module, items) in &self.py_imports {
            if items.is_empty() {
                let key = module.to_string();
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
                writeln!(out, "import {}", module)?;
            } else {
                let key = format!("{}:{}", module, items.join(","));
                if seen.contains(&key) {
                    continue;
                }
                seen.insert(key);
                let items_str = items.join(", ");
                writeln!(out, "from {} import {}", module, items_str)?;
            }
        }

        // Emit wildcard imports
        for module in &self.py_wildcards {
            writeln!(out, "from {} import *", module)?;
        }

        Ok(())
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

        // Split into declarations and main statements, preserving source line info
        let mut decls: Vec<(Stmt, usize)> = Vec::new(); // (stmt, source_line)
        let mut main_stmts: Vec<(Stmt, usize)> = Vec::new();  // (stmt, source_line)

        let source_lines = ast.source_lines;
        for (i, stmt) in ast.stmts.into_iter().enumerate() {
            let line = source_lines.get(i).copied().unwrap_or(0);
            // Skip main function declaration - we'll handle it specially
            if let Stmt::Fn(func) = &stmt {
                if func.name == "main" {
                    continue;
                }
            }

            // Plan 283 Task 1.1: Collect use statements early for import emission
            if let Stmt::Use(use_stmt) = &stmt {
                self.handle_use(use_stmt);
                // Don't push to decls or main_stmts — imports go to file top
                continue;
            }

            if stmt.is_decl() {
                decls.push((stmt, line));
            } else {
                main_stmts.push((stmt, line));
            }
        }

        // First pass: process declarations to collect typing/dataclass imports
        for (decl, _line) in &decls {
            if let Stmt::TypeDecl(type_decl) = decl {
                // Only use dataclass if there are no methods
                if type_decl.members.len() > 0 && type_decl.methods.is_empty() {
                    self.imports.insert("dataclass".into());
                }
                // Scan methods for Optional/Result types
                for method in &type_decl.methods {
                    self.collect_type_imports(&method.params.iter().map(|p| &p.ty).collect::<Vec<_>>());
                    self.collect_type_import_for(&method.ret);
                }
            } else if let Stmt::EnumDecl(enum_decl) = decl {
                // Collect import without emitting
                if enum_decl.items.len() > 0 {
                    self.imports.insert("Enum".into());
                }
            } else if let Stmt::Fn(func) = decl {
                // Scan function params and return type for Optional/Result
                self.collect_type_imports(&func.params.iter().map(|p| &p.ty).collect::<Vec<_>>());
                self.collect_type_import_for(&func.ret);
            } else if let Stmt::SpecDecl(_) = decl {
                // spec → Protocol (import collected at emit time via spec_decl method)
                self.imports.insert("Protocol".into());
            } else if let Stmt::Union(_) = decl {
                // union → dataclass
                self.imports.insert("dataclass".into());
            } else if let Stmt::Tag(_) = decl {
                // tag → dataclass
                self.imports.insert("dataclass".into());
            }
        }

        // ── Phase 2: Generate code body into a temporary buffer ──
        // Plan 283: We generate body first, then prepend imports.
        // This allows builtin function calls to add stdlib imports during codegen.
        let mut code_buf: Vec<u8> = Vec::new();

        // Generate declarations (excluding main)
        for (i, (decl, line)) in decls.iter().enumerate() {
            sink.set_source_line(*line);
            self.stmt(decl, &mut code_buf)?;
            // Add newline between declarations, but not after the last one
            if i < decls.len() - 1 {
                code_buf.write(b"\n")?;
            }
        }

        // Generate main function or wrap statements
        if let Some(main_stmt) = main_func {
            // Output the main function
            if !decls.is_empty() {
                code_buf.write(b"\n")?;
            }
            self.stmt(&main_stmt, &mut code_buf)?;

            // Check if main is async (has .await in body or async return type)
            let main_is_async = if let Stmt::Fn(func) = &main_stmt {
                self.is_async_fn(func) || Self::has_await(&func.body.stmts)
            } else {
                false
            };

            // Add main guard
            code_buf.write(b"\nif __name__ == \"__main__\":\n")?;
            self.indent();
            if main_is_async {
                code_buf.write(b"    asyncio.run(main())\n")?;
            } else {
                code_buf.write(b"    main()\n")?;
            }
            self.dedent();
        } else if !main_stmts.is_empty() {
            // Wrap statements in a main function
            if !decls.is_empty() {
                code_buf.write(b"\n")?;
            }
            code_buf.write(b"def main():\n")?;
            self.indent();
            for (stmt, line) in &main_stmts {
                sink.set_source_line(*line);
                self.stmt(stmt, &mut code_buf)?;
            }
            self.dedent();

            // Add main guard
            code_buf.write(b"\n\nif __name__ == \"__main__\":\n")?;
            self.indent();
            code_buf.write(b"main()\n")?;
            self.dedent();
        }

        // ── Phase 3: Now emit imports + code body to sink ──
        // Collect all imports (from use stmts + builtin mappings + typing/dataclass)

        // Emit Python module imports (from `use` and builtins like time)
        let has_py_imports = !self.py_imports.is_empty() || !self.py_wildcards.is_empty();
        self.emit_py_imports(&mut sink.body)?;

        // Emit typing/dataclass/enum imports
        // Add blank line between Python module imports and typing imports
        let has_typing_imports = self.imports.contains("Optional")
            || self.imports.contains("Result")
            || self.imports.contains("Protocol")
            || self.imports.contains("dataclass")
            || self.imports.contains("Enum");
        if has_py_imports && has_typing_imports {
            sink.body.write(b"\n")?;
        }
        let mut typing_imports = Vec::new();
        if self.imports.contains("Optional") {
            typing_imports.push("Optional");
        }
        if self.imports.contains("Result") {
            typing_imports.push("Result");
        }
        if self.imports.contains("Protocol") {
            typing_imports.push("Protocol");
        }
        if !typing_imports.is_empty() {
            write!(sink.body, "from typing import {}\n", typing_imports.join(", "))?;
        }
        if self.imports.contains("dataclass") {
            sink.body.write(b"from dataclasses import dataclass\n")?;
        }
        if self.imports.contains("Enum") {
            sink.body.write(b"from enum import Enum, auto\n")?;
        }
        // Blank line after all imports, before code body
        if has_py_imports || has_typing_imports {
            sink.body.write(b"\n")?;
        }

        // Append code body
        sink.body.write(&code_buf)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;
    // Plan 091: Universe removed
    
    use std::fs::read_to_string;
    use std::path::PathBuf;

    fn test_a2p(case: &str) -> AutoResult<()> {
        // Extract the test name from the last path segment's numeric prefix
        // e.g., "09_option_result/001_option" -> "option"
        // e.g., "000_hello" -> "hello"
        let last_segment = case.rsplit('/').next().unwrap_or(case);
        let parts: Vec<&str> = last_segment.split("_").collect();
        let name = parts[1..].join("_");
        let name = name.as_str();

        let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let src_path = format!("test/a2p/{}/{}.at", case, name);
        let src_path = d.join(src_path);
        let src = read_to_string(src_path.as_path())?;

        // Plan 091: PythonTrans no longer needs Universe, but Parser still requires it
        // Plan 091: Universe removed
    let _scope = crate::scope_manager::ScopeManager::new();
        let mut parser = Parser::from(src.as_str());
        let ast = parser.parse()?;
        let mut sink = Sink::new(name.into());
        let mut trans = PythonTrans::new(name.into());
        // Note: set_scope() removed - PythonTrans no longer uses Universe
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

    #[test]
    fn test_008_method() {
        test_a2p("008_method").unwrap();
    }

    // Plan 213 Task 1: Option/Result tests
    #[test]
    fn test_09_001_option() {
        test_a2p("09_option_result/001_option").unwrap();
    }

    #[test]
    fn test_09_002_option_default() {
        test_a2p("09_option_result/002_option_default").unwrap();
    }

    #[test]
    fn test_09_003_result_ok() {
        test_a2p("09_option_result/003_result_ok").unwrap();
    }

    #[test]
    fn test_09_004_option_pattern() {
        test_a2p("09_option_result/004_option_pattern").unwrap();
    }

    #[test]
    fn test_09_005_result_pattern() {
        test_a2p("09_option_result/005_result_pattern").unwrap();
    }

    // Plan 213 Task 2: Lambda/Closure tests
    #[test]
    fn test_05_010_lambda() {
        test_a2p("05_expressions/010_lambda").unwrap();
    }

    #[test]
    fn test_05_011_lambda_map() {
        test_a2p("05_expressions/011_lambda_map").unwrap();
    }

    #[test]
    fn test_05_012_closure_var() {
        test_a2p("05_expressions/012_closure_var").unwrap();
    }

    // Plan 213 Task 3: Tuple, Object, Continue, Return tests
    #[test]
    fn test_05_020_tuple() {
        test_a2p("05_expressions/020_tuple").unwrap();
    }

    #[test]
    fn test_05_021_object() {
        test_a2p("05_expressions/021_object").unwrap();
    }

    #[test]
    fn test_05_022_continue() {
        test_a2p("05_expressions/022_continue").unwrap();
    }

    #[test]
    fn test_05_023_return() {
        test_a2p("05_expressions/023_return").unwrap();
    }

    // Plan 213 Task 4: Type annotations
    #[test]
    fn test_01_030_typed_func() {
        test_a2p("01_basics/030_typed_func").unwrap();
    }

    #[test]
    fn test_01_031_typed_vars() {
        test_a2p("01_basics/031_typed_vars").unwrap();
    }

    // Plan 213 Task 5: Error propagation ?.
    #[test]
    fn test_09_006_propagate() {
        test_a2p("09_option_result/006_propagate").unwrap();
    }

    #[test]
    fn test_09_007_propagate_chain() {
        test_a2p("09_option_result/007_propagate_chain").unwrap();
    }

    #[test]
    fn test_09_008_propagate_result() {
        test_a2p("09_option_result/008_propagate_result").unwrap();
    }

    #[test]
    fn test_09_009_propagate_in_call() {
        test_a2p("09_option_result/009_propagate_in_call").unwrap();
    }

    // Plan 213 Task 6: Generics (type erasure)
    #[test]
    fn test_08_001_generic_func() {
        test_a2p("08_generics/001_generic_func").unwrap();
    }

    #[test]
    fn test_08_002_generic_struct() {
        test_a2p("08_generics/002_generic_struct").unwrap();
    }

    // Plan 213 Task 7: async/await
    #[test]
    fn test_03_040_async_func() {
        test_a2p("03_control_flow/040_async_func").unwrap();
    }

    #[test]
    fn test_03_041_await_call() {
        test_a2p("03_control_flow/041_await_call").unwrap();
    }

    // Plan 213 Task 8: spec → Protocol
    #[test]
    fn test_12_001_basic_spec() {
        test_a2p("12_specs/001_basic_spec").unwrap();
    }

    #[test]
    fn test_12_002_spec_impl() {
        test_a2p("12_specs/002_spec_impl").unwrap();
    }

    // Plan 213 Task 9: union/tag → dataclass
    #[test]
    fn test_02_003_union() {
        test_a2p("02_types/003_union").unwrap();
    }

    #[test]
    fn test_02_004_union_match() {
        test_a2p("02_types/004_union_match").unwrap();
    }

    #[test]
    fn test_02_005_tag() {
        test_a2p("02_types/005_tag").unwrap();
    }

    // Plan 213 Task 10: Batch test expansion

    // 01_basics: comments, unary ops, multi-expr, const, nested calls
    #[test]
    fn test_01_040_comments() {
        test_a2p("01_basics/040_comments").unwrap();
    }

    #[test]
    fn test_01_041_unary_neg() {
        test_a2p("01_basics/041_unary_neg").unwrap();
    }

    #[test]
    fn test_01_042_unary_not() {
        test_a2p("01_basics/042_unary_not").unwrap();
    }

    #[test]
    fn test_01_043_multi_expr() {
        test_a2p("01_basics/043_multi_expr").unwrap();
    }

    #[test]
    fn test_01_044_const_decl() {
        test_a2p("01_basics/044_const_decl").unwrap();
    }

    #[test]
    fn test_01_045_nested_call() {
        test_a2p("01_basics/045_nested_call").unwrap();
    }

    #[test]
    fn test_01_046_boolean_ops() {
        test_a2p("01_basics/046_boolean_ops").unwrap();
    }

    #[test]
    fn test_01_047_arithmetic() {
        test_a2p("01_basics/047_arithmetic").unwrap();
    }

    // 02_types: nested struct, type alias, enum with data
    #[test]
    fn test_02_006_nested_struct() {
        test_a2p("02_types/006_nested_struct").unwrap();
    }

    #[test]
    fn test_02_007_type_with_methods() {
        test_a2p("02_types/007_type_with_methods").unwrap();
    }

    #[test]
    fn test_02_008_enum_simple() {
        test_a2p("02_types/008_enum_simple").unwrap();
    }

    #[test]
    fn test_02_009_struct_empty() {
        test_a2p("02_types/009_struct_empty").unwrap();
    }

    #[test]
    fn test_02_010_struct_many_fields() {
        test_a2p("02_types/010_struct_many_fields").unwrap();
    }

    // 03_control_flow: while loop (for cond), nested loops, if-elif chains
    #[test]
    fn test_03_042_for_cond() {
        test_a2p("03_control_flow/042_for_cond").unwrap();
    }

    #[test]
    fn test_03_043_nested_loop() {
        test_a2p("03_control_flow/043_nested_loop").unwrap();
    }

    #[test]
    fn test_03_044_if_elif() {
        test_a2p("03_control_flow/044_if_elif").unwrap();
    }

    // 04_strings: string methods, f-string edge cases
    #[test]
    fn test_04_001_string_basic() {
        test_a2p("04_strings/001_string_basic").unwrap();
    }

    #[test]
    fn test_04_002_fstring_expr() {
        test_a2p("04_strings/002_fstring_expr").unwrap();
    }

    #[test]
    fn test_04_003_string_concat() {
        test_a2p("04_strings/003_string_concat").unwrap();
    }

    // 05_expressions: ternary, null coalesce, composition
    #[test]
    fn test_05_030_null_coalesce() {
        test_a2p("05_expressions/030_null_coalesce").unwrap();
    }

    #[test]
    fn test_05_031_nested_call() {
        test_a2p("05_expressions/031_nested_call").unwrap();
    }

    #[test]
    fn test_05_032_chained_method() {
        test_a2p("05_expressions/032_chained_method").unwrap();
    }

    #[test]
    fn test_05_033_binary_expr() {
        test_a2p("05_expressions/033_binary_expr").unwrap();
    }

    #[test]
    fn test_05_034_index_expr() {
        test_a2p("05_expressions/034_index_expr").unwrap();
    }

    // 06_pattern_matching: struct destructuring, wildcard patterns
    #[test]
    fn test_06_001_is_wildcard() {
        test_a2p("06_pattern_matching/001_is_wildcard").unwrap();
    }

    #[test]
    fn test_06_002_is_multi_pattern() {
        test_a2p("06_pattern_matching/002_is_multi_pattern").unwrap();
    }

    #[test]
    fn test_06_003_is_else() {
        test_a2p("06_pattern_matching/003_is_else").unwrap();
    }

    // 08_generics: generic constraints
    #[test]
    fn test_08_003_generic_method() {
        test_a2p("08_generics/003_generic_method").unwrap();
    }

    #[test]
    fn test_08_004_generic_multi_param() {
        test_a2p("08_generics/004_generic_multi_param").unwrap();
    }

    // 09_option_result: option chains, result propagation, is_ok/is_err
    #[test]
    fn test_09_010_option_chain() {
        test_a2p("09_option_result/010_option_chain").unwrap();
    }

    #[test]
    fn test_09_011_some_expr() {
        test_a2p("09_option_result/011_some_expr").unwrap();
    }

    #[test]
    fn test_09_012_none_value() {
        test_a2p("09_option_result/012_none_value").unwrap();
    }

    #[test]
    fn test_09_013_ok_value() {
        test_a2p("09_option_result/013_ok_value").unwrap();
    }

    #[test]
    fn test_09_014_err_value() {
        test_a2p("09_option_result/014_err_value").unwrap();
    }

    #[test]
    fn test_09_015_option_if() {
        test_a2p("09_option_result/015_option_if").unwrap();
    }

    #[test]
    fn test_09_016_result_func() {
        test_a2p("09_option_result/016_result_func").unwrap();
    }

    #[test]
    fn test_09_017_option_return() {
        test_a2p("09_option_result/017_option_return").unwrap();
    }

    // 10_collections: list/dict operations, array methods
    #[test]
    fn test_10_001_array_basic() {
        test_a2p("10_collections/001_array_basic").unwrap();
    }

    #[test]
    fn test_10_002_array_index() {
        test_a2p("10_collections/002_array_index").unwrap();
    }

    #[test]
    fn test_10_003_object_literal() {
        test_a2p("10_collections/003_object_literal").unwrap();
    }

    // 11_methods: static methods, mut self methods
    #[test]
    fn test_11_001_static_method() {
        test_a2p("11_methods/001_static_method").unwrap();
    }

    #[test]
    fn test_11_002_method_call() {
        test_a2p("11_methods/002_method_call").unwrap();
    }

    #[test]
    fn test_11_003_method_params() {
        test_a2p("11_methods/003_method_params").unwrap();
    }

    // Additional tests for coverage
    #[test]
    fn test_01_050_range_expr() {
        test_a2p("01_basics/050_range_expr").unwrap();
    }

    #[test]
    fn test_01_051_var_mutable() {
        test_a2p("01_basics/051_var_mutable").unwrap();
    }

    #[test]
    fn test_03_045_for_inclusive() {
        test_a2p("03_control_flow/045_for_inclusive").unwrap();
    }

    #[test]
    fn test_05_035_cast_expr() {
        test_a2p("05_expressions/035_cast_expr").unwrap();
    }

    #[test]
    fn test_05_036_to_expr() {
        test_a2p("05_expressions/036_to_expr").unwrap();
    }

    #[test]
    fn test_01_052_multi_assign() {
        test_a2p("01_basics/052_multi_assign").unwrap();
    }

    #[test]
    fn test_01_053_print_var() {
        test_a2p("01_basics/053_print_var").unwrap();
    }

    #[test]
    fn test_03_046_loop_break() {
        test_a2p("03_control_flow/046_loop_break").unwrap();
    }

    // Plan 283 Task 1.1: use → Python import tests
    #[test]
    fn test_14_001_import() {
        test_a2p("14_modules/001_import").unwrap();
    }

    #[test]
    fn test_14_002_from_import() {
        test_a2p("14_modules/002_from_import").unwrap();
    }

    // Plan 283 Task 4.1: use.py → Python import tests
    #[test]
    fn test_14_003_use_py() {
        test_a2p("14_modules/003_use_py").unwrap();
    }

    #[test]
    fn test_14_004_import_with_dataclass() {
        test_a2p("14_modules/004_import_with_dataclass").unwrap();
    }

    // Plan 283 Task 1.2: Builtin function mapping tests
    #[test]
    fn test_16_001_builtin_map() {
        test_a2p("16_python_std/001_builtin_map").unwrap();
    }

    // Plan 283 Task 1.3: Collection method mapping tests
    #[test]
    fn test_16_002_method_map() {
        test_a2p("16_python_std/002_method_map").unwrap();
    }

    // Plan 283 Task 2.2: Static method decorator test
    #[test]
    fn test_11_004_static_decorator() {
        test_a2p("11_methods/004_static_decorator").unwrap();
    }

    // Plan 283 Task 2.4: Struct destructuring pattern matching
    #[test]
    fn test_06_004_struct_destructure() {
        test_a2p("06_pattern_matching/004_struct_destructure").unwrap();
    }
}
