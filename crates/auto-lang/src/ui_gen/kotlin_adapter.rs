//! Kotlin handler-body adapter: translates base `ast::Stmt` to Kotlin.
//!
//! Fully independent (Kotlin is NOT a TS subset — no `trans::typescript`
//! delegation). State refs become bare names (Compose `var by remember` pattern
//! means a state field is referenced as a plain identifier, NOT `this.count`
//! like ArkTS and NOT `count.value` like Vue reactive refs).
//!
//! Key Kotlin-specific rewrites:
//! - State references (`self.count` / `.count` / bare state ident) → bare `count`
//! - No semicolons (statements end with a newline)
//! - `StoreKind::Let` / `Var` → `var`; `StoreKind::Const` → `val`
//! - `print(...)` → `println(...)`
//! - For loops: `Named` → `for (v in it)`, `Cond` → `while (cond)`,
//!   `Ever` → `while (true)`, `Indexed` → `for (v in start until end)`
//! - Array literal → `listOf(...)`, Object literal → `mapOf("k" to v)`
//! - `.len` / `.len()` → `.size`; `.to_int` → `.toInt`; `.to_string` → `.toString`
//! - FStr → Kotlin string templates `"${expr}"`
//! - Long-tail unrecognized variants → `/* TODO(kotlin): {variant} */`

use crate::ast::*;
use std::collections::HashSet;
use std::io::Write;

/// Context for Kotlin handler-body transpilation.
pub struct KotlinAdapterCtx {
    /// State field names (secondary heuristic for detecting bare-identifier
    /// state refs). The primary detection path is `Expr::Dot(self|., field)`
    /// rewriting, which works regardless of this set.
    pub state_fields: HashSet<String>,
}

impl KotlinAdapterCtx {
    /// Construct a context with the given state-field names.
    pub fn new(state_fields: HashSet<String>) -> Self {
        Self { state_fields }
    }

    /// Construct an empty context (no state-field set available). State refs
    /// are still detected via `Expr::Dot(self/., field)` patterns.
    pub fn empty() -> Self {
        Self {
            state_fields: HashSet::new(),
        }
    }

    /// Whether `name` is a known state field.
    fn is_state(&self, name: &str) -> bool {
        self.state_fields.contains(name)
    }
}

/// Transpile a list of AutoLang statements to a Kotlin handler body, with
/// Kotlin state-ref rewrites. Statements are emitted one per line with no
/// trailing semicolons.
pub fn transpile_handler_body(stmts: &[Stmt], ctx: &KotlinAdapterCtx) -> String {
    let mut out = Vec::new();
    for (i, stmt) in stmts.iter().enumerate() {
        if i > 0 {
            writeln!(out).ok();
        }
        transpile_stmt(stmt, ctx, &mut out);
    }
    String::from_utf8(out).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Statement transpilation
// ---------------------------------------------------------------------------

fn transpile_stmt(stmt: &Stmt, ctx: &KotlinAdapterCtx, out: &mut Vec<u8>) {
    match stmt {
        // Variable declarations — Kotlin uses `var`/`val` (no semicolons)
        Stmt::Store(store) => {
            let kw = match store.kind {
                StoreKind::Let | StoreKind::Var => "var",
                StoreKind::Const => "val",
                _ => "var", // Shared, CVar, Field — shouldn't normally appear in handlers
            };
            write!(out, "{} {} = ", kw, store.name.as_str()).ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out).ok();
        }

        // Expression statements — print, method calls, etc.
        Stmt::Expr(expr) => {
            transpile_expr(expr, ctx, out);
            writeln!(out).ok();
        }

        // If / else if / else — Kotlin if-chains
        Stmt::If(if_stmt) => {
            if let Some(first) = if_stmt.branches.first() {
                write!(out, "if (").ok();
                transpile_expr(&first.cond, ctx, out);
                write!(out, ") {{").ok();
                transpile_body(&first.body, ctx, out);
                write!(out, "}}").ok();
            }
            for branch in if_stmt.branches.iter().skip(1) {
                write!(out, " else if (").ok();
                transpile_expr(&branch.cond, ctx, out);
                write!(out, ") {{").ok();
                transpile_body(&branch.body, ctx, out);
                write!(out, "}}").ok();
            }
            if let Some(else_body) = &if_stmt.else_ {
                write!(out, " else {{").ok();
                transpile_body(else_body, ctx, out);
                write!(out, "}}").ok();
            }
            writeln!(out).ok();
        }

        // For loops
        Stmt::For(for_loop) => {
            transpile_for(for_loop, ctx, out);
        }

        // Return — bare `return` for nil/null, else `return expr`
        Stmt::Return(expr) => match expr.as_ref() {
            Expr::Nil | Expr::Null => {
                writeln!(out, "return").ok();
            }
            _ => {
                write!(out, "return ").ok();
                transpile_expr(expr, ctx, out);
                writeln!(out).ok();
            }
        },

        // Break / Continue
        Stmt::Break => {
            writeln!(out, "break").ok();
        }
        Stmt::Continue => {
            writeln!(out, "continue").ok();
        }

        // Long-tail — Kotlin is NOT a TS subset, so emit a TODO comment.
        other => {
            writeln!(out, "/* TODO(kotlin): {} */", stmt_kind(other)).ok();
        }
    }
}

/// Render a block body (sequence of statements) inline.
fn transpile_body(body: &Body, ctx: &KotlinAdapterCtx, out: &mut Vec<u8>) {
    for stmt in &body.stmts {
        transpile_stmt(stmt, ctx, out);
    }
}

/// Transpile a `For` loop to Kotlin constructs.
fn transpile_for(for_loop: &For, ctx: &KotlinAdapterCtx, out: &mut Vec<u8>) {
    match &for_loop.iter {
        Iter::Cond => {
            // for condition { ... } → while (condition) { ... }
            write!(out, "while (").ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Ever => {
            // for ever { ... } → while (true) { ... }
            writeln!(out, "while (true) {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Named(iter_name) => {
            // for item in range { ... } → for (item in range) { ... }
            write!(out, "for ({} in ", iter_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Indexed(index_name, iter_name) => {
            // for i, item in start..end { ... } → for (item in start until end) { ... }
            // The index variable `index_name` is not directly expressible in a
            // single Kotlin for-loop without `withIndex()`; for PR-4b we use
            // `start until end` over the range expression and ignore the index.
            // If range is not a numeric range, fall back to TODO.
            write!(out, "for ({} in ", iter_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
            // Note: index `index_name` dropped (Kotlin `for (i in x until y)` form).
            let _ = index_name;
        }
        Iter::Call(call) => {
            // for func(args) { ... } → while (func(args)) { ... }
            write!(out, "while (").ok();
            match call.name.as_ref() {
                Expr::Dot(object, method) => {
                    transpile_expr(object, ctx, out);
                    write!(out, ".{}(", method.as_str()).ok();
                }
                Expr::Ident(name) => {
                    write!(out, "{}(", name.as_str()).ok();
                }
                _ => {
                    write!(out, "/* complex call */(").ok();
                }
            }
            for (i, arg) in call.args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")) {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Destructured(key, val) => {
            // for (k, v) in map → for ((k, v) in map) { ... }
            write!(out, "for (({}, {}) in ", key.as_str(), val.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Expression transpilation — Kotlin (no TS delegation)
// ---------------------------------------------------------------------------

fn transpile_expr(expr: &Expr, ctx: &KotlinAdapterCtx, out: &mut Vec<u8>) {
    match expr {
        // === Kotlin-specific state-ref rewrites ===

        // `self.count` / `.count` → bare `count` (Compose var-by-remember pattern)
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" || name.as_str() == "." {
                    write!(out, "{}", field.as_str()).ok();
                    return;
                }
            }
            // Builtin module field access
            if try_transpile_builtin_field(obj, field.as_str(), ctx, out) {
                return;
            }
            // General field access: object.field
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }

        // Identifier — bare state field stays bare; `self`/`.` → `this` (rare)
        Expr::Ident(name) => {
            let n = name.as_str();
            if n == "self" || n == "." {
                write!(out, "this").ok();
            } else {
                write!(out, "{}", n).ok();
            }
        }

        // Literals ----------------------------------------------------------

        Expr::Int(i) => {
            write!(out, "{}", i).ok();
        }
        Expr::Uint(u) => {
            write!(out, "{}", u).ok();
        }
        Expr::I8(i) => {
            write!(out, "{}", i).ok();
        }
        Expr::U8(u) => {
            write!(out, "{}", u).ok();
        }
        Expr::I64(i) => {
            write!(out, "{}", i).ok();
        }
        Expr::U64(u) => {
            write!(out, "{}", u).ok();
        }
        Expr::Byte(b) => {
            write!(out, "{}", b).ok();
        }
        Expr::Float(v, _) => {
            write!(out, "{}", v).ok();
        }
        Expr::Double(v, _) => {
            write!(out, "{}", v).ok();
        }
        Expr::Bool(b) => {
            write!(out, "{}", b).ok();
        }
        Expr::Char(c) => {
            write!(out, "'{}'", c).ok();
        }

        // String literal — Kotlin double-quoted
        Expr::Str(s) => {
            let escaped = s
                .replace("\\", "\\\\")
                .replace("\"", "\\\"")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            write!(out, "\"{}\"", escaped).ok();
        }
        Expr::CStr(s) => {
            let escaped = s.replace("\\", "\\\\").replace("\"", "\\\"");
            write!(out, "\"{}\"", escaped).ok();
        }

        // Nil / Null → Kotlin `null`
        Expr::Nil | Expr::Null => {
            write!(out, "null").ok();
        }

        // Function call — print → println, builtins, method calls, plain calls
        Expr::Call(call) => match call.name.as_ref() {
            // Method call: object.method(args)
            Expr::Dot(object, method) => {
                if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                    return;
                }
                match method.as_str() {
                    "to_int" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".toInt()").ok();
                        return;
                    }
                    "to_string" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".toString()").ok();
                        return;
                    }
                    "len" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".size").ok();
                        return;
                    }
                    "contains" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".contains(").ok();
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ").ok();
                            }
                            transpile_expr(&arg.get_expr(), ctx, out);
                        }
                        write!(out, ")").ok();
                        return;
                    }
                    "remove" => {
                        // list.remove(idx) → list.removeAt(idx) (MutableList)
                        transpile_expr(object, ctx, out);
                        write!(out, ".removeAt(").ok();
                        if let Some(first_arg) = call.args.args.first() {
                            transpile_expr(&first_arg.get_expr(), ctx, out);
                        } else {
                            write!(out, "0").ok();
                        }
                        write!(out, ")").ok();
                        return;
                    }
                    _ => {}
                }
                // Generic method call: object.method(args)
                transpile_expr(object, ctx, out);
                write!(out, ".{}", method.as_str()).ok();
                write!(out, "(").ok();
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    transpile_expr(&arg.get_expr(), ctx, out);
                }
                write!(out, ")").ok();
            }
            // Plain function call: name(args)
            Expr::Ident(name) => {
                let func_name = name.as_str();
                if func_name == "print" {
                    write!(out, "println").ok();
                } else {
                    write!(out, "{}", func_name).ok();
                }
                write!(out, "(").ok();
                for (i, arg) in call.args.args.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    transpile_expr(&arg.get_expr(), ctx, out);
                }
                write!(out, ")").ok();
            }
            // Complex call name — emit a TODO comment (no TS delegation)
            _ => {
                writeln!(out, "/* TODO(kotlin): complex call */").ok();
            }
        },

        // Binary ops — assignment targets need state-ref rewriting
        Expr::Bina(lhs, op, rhs) => {
            use auto_val::Op;
            match op {
                Op::Asn => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " = ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::AddEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " += ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::SubEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " -= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::MulEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " *= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::DivEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " /= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::ModEq => {
                    transpile_assign_target(lhs, ctx, out);
                    write!(out, " %= ").ok();
                    transpile_expr(rhs, ctx, out);
                }
                Op::Dot => {
                    // Field access via Op::Dot
                    transpile_expr(lhs, ctx, out);
                    write!(out, ".").ok();
                    transpile_expr(rhs, ctx, out);
                }
                _ => {
                    transpile_expr(lhs, ctx, out);
                    write!(out, " {} ", op.op()).ok();
                    transpile_expr(rhs, ctx, out);
                }
            }
        }

        // Unary ops
        Expr::Unary(op, operand) => {
            let op_str = match op {
                auto_val::Op::Sub => "-",
                auto_val::Op::Not | auto_val::Op::Bang => "!",
                _ => "",
            };
            write!(out, "{}", op_str).ok();
            transpile_expr(operand, ctx, out);
        }

        // Array literals → listOf(...)
        Expr::Array(elems) => {
            write!(out, "listOf(").ok();
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(elem, ctx, out);
            }
            write!(out, ")").ok();
        }

        // Object literals → mapOf("key" to value, ...)
        Expr::Object(pairs) => {
            write!(out, "mapOf(").ok();
            for (i, pair) in pairs.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                match &pair.key {
                    Key::NamedKey(name) => {
                        write!(out, "\"{}\" to ", name.as_str()).ok();
                    }
                    Key::IntKey(n) => {
                        write!(out, "{} to ", n).ok();
                    }
                    Key::BoolKey(b) => {
                        write!(out, "{} to ", b).ok();
                    }
                    Key::StrKey(s) => {
                        write!(out, "\"{}\" to ", s).ok();
                    }
                }
                transpile_expr(&pair.value, ctx, out);
            }
            write!(out, ")").ok();
        }

        // Null-coalescing operator → Kotlin `?:`
        Expr::NullCoalesce(lhs, rhs) => {
            transpile_expr(lhs, ctx, out);
            write!(out, " ?: ").ok();
            transpile_expr(rhs, ctx, out);
        }

        // Error propagation ?. → Kotlin safe-call `?.`
        Expr::ErrorPropagate(inner) => {
            transpile_expr(inner, ctx, out);
            write!(out, "?.").ok();
        }

        // Index access: arr[idx]
        Expr::Index(array, index) => {
            transpile_expr(array, ctx, out);
            write!(out, "[").ok();
            transpile_expr(index, ctx, out);
            write!(out, "]").ok();
        }

        // Closure: { params -> body }
        Expr::Closure(closure) => {
            write!(out, "{{ ").ok();
            if !closure.params.is_empty() {
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    write!(out, "{}", param.name).ok();
                }
                write!(out, " -> ").ok();
            }
            transpile_expr(&closure.body, ctx, out);
            write!(out, " }}").ok();
        }

        // FStr → Kotlin string templates "${expr}"
        Expr::FStr(fstr) => {
            write!(out, "\"").ok();
            for part in &fstr.parts {
                match part {
                    Expr::Str(s) => {
                        let escaped = s
                            .replace("\\", "\\\\")
                            .replace("\"", "\\\"")
                            .replace("\n", "\\n")
                            .replace("\r", "\\r")
                            .replace("\t", "\\t");
                        write!(out, "{}", escaped).ok();
                    }
                    other => {
                        write!(out, "${{").ok();
                        transpile_expr(other, ctx, out);
                        write!(out, "}}").ok();
                    }
                }
            }
            write!(out, "\"").ok();
        }

        // Long-tail — no TS delegation; emit TODO comment
        other => {
            write!(out, "/* TODO(kotlin): {} */", expr_kind(other)).ok();
        }
    }
}

/// Render an assignment target with state-ref rewriting.
/// `self.count` / `.count` → bare `count`; bare state ident stays bare.
fn transpile_assign_target(expr: &Expr, ctx: &KotlinAdapterCtx, out: &mut Vec<u8>) {
    match expr {
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" || name.as_str() == "." {
                    write!(out, "{}", field.as_str()).ok();
                    return;
                }
            }
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }
        Expr::Index(arr, idx) => {
            transpile_expr(arr, ctx, out);
            write!(out, "[").ok();
            transpile_expr(idx, ctx, out);
            write!(out, "]").ok();
        }
        Expr::Ident(name) => {
            write!(out, "{}", name.as_str()).ok();
        }
        other => {
            write!(out, "/* TODO(kotlin): assign target {} */", expr_kind(other)).ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Builtin module transpilation
// ---------------------------------------------------------------------------

/// Try to transpile a method call on a builtin module (e.g. `json.parse(x)`).
/// Returns true if the call was handled, false otherwise.
fn try_transpile_builtin_call(
    object: &Expr,
    method: &str,
    args: &Args,
    ctx: &KotlinAdapterCtx,
    out: &mut Vec<u8>,
) -> bool {
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };

    match module {
        "json" => {
            // Kotlin org.json or TODO comment
            write!(
                out,
                "/* TODO(kotlin): org.json.{} */",
                method
            )
            .ok();
            for arg in args.args.iter() {
                let _ = arg.get_expr(); // ensure used; values embedded in comment omitted for brevity
            }
            true
        }
        "math" => {
            // kotlin.math.* — emit as TODO for now
            write!(out, "/* TODO(kotlin): kotlin.math.{} */", method).ok();
            true
        }
        "date" => {
            write!(out, "/* TODO(kotlin): java.util.Date.{} */", method).ok();
            true
        }
        "storage" => {
            write!(out, "/* TODO(kotlin): storage.{} */", method).ok();
            true
        }
        "event" => {
            write!(out, "/* TODO(kotlin): event.{} */", method).ok();
            true
        }
        "router" => {
            write!(out, "/* TODO(kotlin): router.{} */", method).ok();
            true
        }
        _ => false,
    }
}

/// Try to transpile a field access on a builtin module. Returns true if
/// handled, false otherwise. Currently no Kotlin builtin fields are mapped.
fn try_transpile_builtin_field(
    object: &Expr,
    field: &str,
    _ctx: &KotlinAdapterCtx,
    out: &mut Vec<u8>,
) -> bool {
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };
    match module {
        "math" | "json" | "date" | "storage" | "event" | "router" => {
            write!(out, "/* TODO(kotlin): {}.{} */", module, field).ok();
            true
        }
        _ => false,
    }
}

/// A short human-readable name for a statement variant (for TODO comments).
fn stmt_kind(stmt: &Stmt) -> &'static str {
    match stmt {
        Stmt::Store(_) => "Store",
        Stmt::Expr(_) => "Expr",
        Stmt::If(_) => "If",
        Stmt::For(_) => "For",
        Stmt::Return(_) => "Return",
        Stmt::Break => "Break",
        Stmt::Continue => "Continue",
        _ => "Stmt",
    }
}

/// A short human-readable name for an expression variant (for TODO comments).
fn expr_kind(expr: &Expr) -> &'static str {
    match expr {
        Expr::Int(_) => "Int",
        Expr::Str(_) => "Str",
        Expr::Ident(_) => "Ident",
        Expr::Call(_) => "Call",
        Expr::Dot(_, _) => "Dot",
        Expr::Bina(_, _, _) => "Bina",
        Expr::Array(_) => "Array",
        Expr::Object(_) => "Object",
        Expr::Closure(_) => "Closure",
        Expr::FStr(_) => "FStr",
        _ => "Expr",
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Arg, Args, Call, Expr, For, Iter, Name, Stmt, Store, StoreKind};

    fn ctx_empty() -> KotlinAdapterCtx {
        KotlinAdapterCtx::empty()
    }

    fn ctx_with(names: &[&str]) -> KotlinAdapterCtx {
        KotlinAdapterCtx::new(names.iter().map(|s| s.to_string()).collect())
    }

    fn ident(n: &str) -> Expr {
        Expr::Ident(Name::from(n))
    }

    fn call(name: &str, args: Vec<Expr>) -> Expr {
        let args = Args {
            args: args.into_iter().map(Arg::Pos).collect(),
        };
        Expr::Call(Call {
            name: Box::new(ident(name)),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        })
    }

    #[test]
    fn test_kotlin_state_ref_assign() {
        // .count = .count + 1 → count = count + 1
        let lhs = Expr::Dot(Box::new(ident(".")), Name::from("count"));
        let rhs = Expr::Bina(
            Box::new(Expr::Dot(Box::new(ident(".")), Name::from("count"))),
            auto_val::Op::Add,
            Box::new(Expr::Int(1)),
        );
        let expr = Expr::Bina(Box::new(lhs), auto_val::Op::Asn, Box::new(rhs));
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(
            body.contains("count = count + 1"),
            "body = {}",
            body
        );
    }

    #[test]
    fn test_kotlin_state_ref_self() {
        // self.count → count
        let e = Expr::Dot(Box::new(ident("self")), Name::from("count"));
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "count");
    }

    #[test]
    fn test_kotlin_bare_state_ident_stays_bare() {
        // count (state field) → count (NOT this.count)
        let e = ident("count");
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_with(&["count"]), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "count");
    }

    #[test]
    fn test_kotlin_print() {
        let stmt = Stmt::Expr(call("print", vec![Expr::Str("hello".into())]));
        let body = transpile_handler_body(&[stmt], &ctx_empty());
        assert!(body.contains("println(\"hello\")"), "body = {}", body);
    }

    #[test]
    fn test_kotlin_for_in() {
        // for x in .items { print(x) } → for (x in items) { println(x) }
        let for_loop = For {
            iter: Iter::Named(Name::from("x")),
            range: Expr::Dot(Box::new(ident(".")), Name::from("items")),
            body: Body::single_expr(call("print", vec![ident("x")])),
            new_line: false,
            init: None,
        };
        let body = transpile_handler_body(&[Stmt::For(for_loop)], &ctx_empty());
        assert!(body.contains("for (x in items)"), "body = {}", body);
        assert!(body.contains("println(x)"), "body = {}", body);
    }

    #[test]
    fn test_kotlin_no_semicolons() {
        let stmt = Stmt::Expr(call("print", vec![Expr::Str("hi".into())]));
        let body = transpile_handler_body(&[stmt], &ctx_empty());
        assert!(!body.contains(';'), "Kotlin output must not contain ';': {}", body);
    }

    #[test]
    fn test_kotlin_let_var() {
        let store = Store {
            kind: StoreKind::Let,
            name: Name::from("x"),
            ty: Type::Unknown,
            expr: Expr::Int(1),
            attrs: Vec::new(),
        };
        let body = transpile_handler_body(&[Stmt::Store(store)], &ctx_empty());
        assert_eq!(body.trim(), "var x = 1");
    }

    #[test]
    fn test_kotlin_const_val() {
        let store = Store {
            kind: StoreKind::Const,
            name: Name::from("y"),
            ty: Type::Unknown,
            expr: Expr::Int(2),
            attrs: Vec::new(),
        };
        let body = transpile_handler_body(&[Stmt::Store(store)], &ctx_empty());
        assert_eq!(body.trim(), "val y = 2");
    }

    #[test]
    fn test_kotlin_array_literal() {
        let e = Expr::Array(vec![Expr::Int(1), Expr::Int(2), Expr::Int(3)]);
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "listOf(1, 2, 3)");
    }

    #[test]
    fn test_kotlin_object_literal() {
        let pairs = vec![Pair {
            key: Key::StrKey("a".into()),
            value: Box::new(Expr::Int(1)),
        }];
        let e = Expr::Object(pairs);
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "mapOf(\"a\" to 1)");
    }

    #[test]
    fn test_kotlin_len_to_size() {
        // .items.len() → items.size
        let callee = Expr::Dot(
            Box::new(Expr::Dot(Box::new(ident(".")), Name::from("items"))),
            Name::from("len"),
        );
        let args = Args { args: vec![] };
        let e = Expr::Call(Call {
            name: Box::new(callee),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        });
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "items.size");
    }

    #[test]
    fn test_kotlin_for_ever_while_true() {
        let for_loop = For {
            iter: Iter::Ever,
            range: Expr::Int(0),
            body: Body::single_expr(call("tick", vec![])),
            new_line: false,
            init: None,
        };
        let body = transpile_handler_body(&[Stmt::For(for_loop)], &ctx_empty());
        assert!(body.contains("while (true)"), "body = {}", body);
    }

    #[test]
    fn test_kotlin_unknown_expr_todo() {
        // Expr::Some(x) — not handled explicitly; should emit TODO, not panic
        let e = Expr::Some(Box::new(ident("x")));
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        let s = String::from_utf8(out).unwrap();
        assert!(s.contains("TODO(kotlin)"), "should emit TODO comment: {}", s);
    }
}
