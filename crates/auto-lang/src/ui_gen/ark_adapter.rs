//! Ark handler-body adapter: translates base `ast::Stmt` to ArkTS.
//!
//! Mirrors `ts_adapter` but emits ArkTS-specific constructs:
//! - State references (`.count` / `self.count` / bare state-field ident) → `this.count`
//!   (NOT `count.value` like Vue reactive refs).
//! - No `await` prefix on API/function calls (ArkTS dispatch is synchronous).
//! - No `props.` rewrite — props are accessed as `this.xxx`, same as state fields.
//! - `print()` → `console.log()` (same as Vue).
//! - `math.*` → `Math.*`, `json.*` → `JSON.*` (same as Vue).
//! - `storage` / `event` / `router` → left as TODO comments (HarmonyOS API
//!   mapping is separate work; PR-4a keeps these as placeholders).
//!
//! Long-tail delegation: unrecognized `Stmt`/`Expr` variants fall back to
//! `trans::typescript::TypeScriptTrans`. ArkTS is a strict subset of TypeScript,
//! so generic TS syntax produced by the delegate is valid ArkTS in the vast
//! majority of cases (this is the key reuse point — no need to reimplement the
//! full expression transpiler here).

use crate::ast::*;
use crate::trans::Sink;
use crate::trans::typescript::TypeScriptTrans;
use std::collections::HashSet;
use std::io::Write;

/// Context for Ark handler-body transpilation.
pub struct ArkAdapterCtx {
    /// State field names (secondary heuristic for detecting bare-identifier
    /// state refs like `count` → `this.count`). The primary detection path is
    /// `Expr::Dot(self|., field)` rewriting, which works regardless of this set.
    pub state_fields: HashSet<String>,
}

impl ArkAdapterCtx {
    /// Construct a context with the given state-field names.
    pub fn new(state_fields: HashSet<String>) -> Self {
        Self { state_fields }
    }

    /// Construct an empty context (no state-field set available). State refs are
    /// still detected via `Expr::Dot(self/., field)` patterns.
    pub fn empty() -> Self {
        Self {
            state_fields: HashSet::new(),
        }
    }

    /// Whether `name` is a known state field (bare-identifier heuristic).
    fn is_state(&self, name: &str) -> bool {
        self.state_fields.contains(name)
    }
}

/// Transpile a list of AutoLang statements to ArkTS handler body, with Ark
/// state-ref rewrites. Statements are emitted one per line.
pub fn transpile_handler_body(stmts: &[Stmt], ctx: &ArkAdapterCtx) -> String {
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

fn transpile_stmt(stmt: &Stmt, ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
    match stmt {
        // Variable declarations — value rewriting via transpile_expr
        Stmt::Store(store) => {
            let kw = match store.kind {
                StoreKind::Let => "let",
                StoreKind::Var => "let",
                StoreKind::Const => "const",
                _ => "let", // Shared, CVar, Field — shouldn't normally appear in handlers
            };
            write!(out, "{} {} = ", kw, store.name.as_str()).ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // Expression statements — print, method calls, etc.
        Stmt::Expr(expr) => {
            transpile_expr(expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // If / else if / else — emit ArkTS if-chains
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
        }

        // For loops
        Stmt::For(for_loop) => {
            transpile_for(for_loop, ctx, out);
        }

        // Return — bare `return;` for nil/null, else `return expr;`
        Stmt::Return(expr) => match expr.as_ref() {
            Expr::Nil | Expr::Null => {
                writeln!(out, "return;").ok();
            }
            _ => {
                write!(out, "return ").ok();
                transpile_expr(expr, ctx, out);
                writeln!(out, ";").ok();
            }
        },

        // Break / Continue
        Stmt::Break => {
            writeln!(out, "break;").ok();
        }
        Stmt::Continue => {
            writeln!(out, "continue;").ok();
        }

        // Fallback — delegate to TypeScriptTrans (ArkTS is a TS subset)
        _ => {
            let mut ts = TypeScriptTrans::new("fragment".into());
            let mut sink = Sink::new("fragment".into());
            let _ = ts.stmt(stmt, &mut sink);
            let _ = out.write_all(&sink.body);
        }
    }
}

/// Transpile a block body (sequence of statements) inline.
fn transpile_body(body: &Body, ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
    for stmt in &body.stmts {
        transpile_stmt(stmt, ctx, out);
    }
}

/// Transpile a `For` loop. Mirrors ts_adapter's for-loop handling but emits
/// ArkTS-compatible constructs (no `range.forEach` arrow functions in hot
/// paths; prefer `for...of` / `while`).
fn transpile_for(for_loop: &For, ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
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
            // for item in range { ... } → for (const item of range) { ... }
            write!(out, "for (const {} of ", iter_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
        Iter::Indexed(index_name, iter_name) => {
            // ArkTS classic indexed for-loop:
            //   for (let i = 0; i < range.length; i++) { const item = range[i]; ... }
            write!(out, "for (let {} = 0; ", index_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ".length; {}++) {{", index_name.as_str()).ok();
            writeln!(out).ok();
            write!(out, "const {} = ", iter_name.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, "[{}];", index_name.as_str()).ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
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
            // for (k, v) in map → for (const [k, v] of Object.entries(map)) { ... }
            write!(
                out,
                "for (const [{}, {}] of Object.entries(",
                key.as_str(),
                val.as_str()
            )
            .ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ")) {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Expression transpilation — Ark-aware with TypeScriptTrans delegation
// ---------------------------------------------------------------------------

fn transpile_expr(expr: &Expr, ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
    match expr {
        // === Ark-specific rewrites ===

        // Field access / state ref:
        //   `.count` and `self.count` both parse as Expr::Dot(Ident("self") | Ident("."), "count")
        //   → `this.count` in ArkTS (no `.value`, unlike Vue refs).
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" || name.as_str() == "." {
                    // State field or prop — both become `this.field` in ArkTS.
                    write!(out, "this.{}", field.as_str()).ok();
                    return;
                }
            }
            // Builtin module field access (e.g. router.path) — currently no
            // Ark-specific builtins are mapped, so fall through to general path.
            if try_transpile_builtin_field(obj, field.as_str(), ctx, out) {
                return;
            }
            // General field access: object.field
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }

        // Identifier — bare state field → `this.name`; `self`/`.` standalone → `this`
        Expr::Ident(name) => {
            let n = name.as_str();
            if n == "self" || n == "." {
                write!(out, "this").ok();
            } else if ctx.is_state(n) {
                write!(out, "this.{}", n).ok();
            } else {
                write!(out, "{}", n).ok();
            }
        }

        // Literals ----------------------------------------------------------

        // Integer literals
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

        // Float / Double
        Expr::Float(v, _) => {
            write!(out, "{}", v).ok();
        }
        Expr::Double(v, _) => {
            write!(out, "{}", v).ok();
        }

        // Bool
        Expr::Bool(b) => {
            write!(out, "{}", b).ok();
        }

        // Char
        Expr::Char(c) => {
            write!(out, "'{}'", c).ok();
        }

        // String literal — single-quoted, escaped (matches ts_adapter)
        Expr::Str(s) => {
            let escaped = s
                .replace("\\", "\\\\")
                .replace("'", "\\'")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            write!(out, "'{}'", escaped).ok();
        }
        Expr::CStr(s) => {
            let escaped = s.replace("\\", "\\\\").replace("'", "\\'");
            write!(out, "'{}'", escaped).ok();
        }

        // Nil / Null
        Expr::Nil | Expr::Null => {
            write!(out, "null").ok();
        }

        // Function call — print, builtins, method calls, plain calls
        Expr::Call(call) => match call.name.as_ref() {
            // Method call: object.method(args)
            Expr::Dot(object, method) => {
                // Builtin module methods (json.*, math.*, ...)
                if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                    return;
                }
                // Common method conversions (shared with ts_adapter)
                match method.as_str() {
                    "to_int" => {
                        write!(out, "parseInt(").ok();
                        transpile_expr(object, ctx, out);
                        write!(out, ")").ok();
                        return;
                    }
                    "to_string" => {
                        write!(out, "(").ok();
                        transpile_expr(object, ctx, out);
                        write!(out, ").toString()").ok();
                        return;
                    }
                    "len" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".length").ok();
                        return;
                    }
                    // Auto `.contains` -> JS/ArkTS `.includes`
                    "contains" => {
                        transpile_expr(object, ctx, out);
                        write!(out, ".includes(").ok();
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
                        // notes.remove(idx) → this.notes.splice(idx, 1)
                        transpile_expr(object, ctx, out);
                        write!(out, ".splice(").ok();
                        if let Some(first_arg) = call.args.args.first() {
                            transpile_expr(&first_arg.get_expr(), ctx, out);
                        } else {
                            write!(out, "0").ok();
                        }
                        write!(out, ", 1)").ok();
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
                    write!(out, "console.log").ok();
                } else {
                    // No `await` prefix in ArkTS — dispatch is synchronous.
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
            // Complex call name — delegate to TypeScriptTrans
            _ => delegate_expr(expr, ctx, out),
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
                    // Field access via Op::Dot — delegate to be safe.
                    delegate_expr(expr, ctx, out);
                }
                _ => {
                    // Standard binary op: left op right
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

        // Array literals
        Expr::Array(elems) => {
            write!(out, "[").ok();
            for (i, elem) in elems.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(elem, ctx, out);
            }
            write!(out, "]").ok();
        }

        // Object literals — recurse values through transpile_expr so builtins work
        Expr::Object(pairs) => {
            write!(out, "{{ ").ok();
            for (i, pair) in pairs.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                match &pair.key {
                    Key::NamedKey(name) => {
                        write!(out, "{}: ", name.as_str()).ok();
                    }
                    Key::IntKey(n) => {
                        write!(out, "{}: ", n).ok();
                    }
                    Key::BoolKey(b) => {
                        write!(out, "{}: ", b).ok();
                    }
                    Key::StrKey(s) => {
                        write!(out, "\"{}\": ", s).ok();
                    }
                }
                transpile_expr(&pair.value, ctx, out);
            }
            write!(out, " }}").ok();
        }

        // Null-coalescing operator ??
        Expr::NullCoalesce(lhs, rhs) => {
            transpile_expr(lhs, ctx, out);
            write!(out, " ?? ").ok();
            transpile_expr(rhs, ctx, out);
        }

        // Error propagation ?. (emit optional chaining)
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

        // Closure: x => expr or (a, b) => expr
        Expr::Closure(closure) => {
            if closure.params.len() == 1 {
                write!(out, "{}", closure.params[0].name).ok();
            } else {
                write!(out, "(").ok();
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        write!(out, ", ").ok();
                    }
                    write!(out, "{}", param.name).ok();
                }
                write!(out, ")").ok();
            }
            write!(out, " => ").ok();
            transpile_expr(&closure.body, ctx, out);
        }

        // If expression — wrap in IIFE so it works in expression position
        Expr::If(if_expr) => {
            write!(out, "(() => {{ ").ok();
            if let Some(first) = if_expr.branches.first() {
                write!(out, "if (").ok();
                transpile_expr(&first.cond, ctx, out);
                write!(out, ") {{ ").ok();
                for stmt in &first.body.stmts {
                    transpile_stmt(stmt, ctx, out);
                }
                write!(out, " }}").ok();
            }
            if let Some(else_body) = &if_expr.else_ {
                write!(out, " else {{ ").ok();
                for stmt in &else_body.stmts {
                    transpile_stmt(stmt, ctx, out);
                }
                write!(out, " }}").ok();
            }
            write!(out, " }})()").ok();
        }

        // === Delegate to TypeScriptTrans for everything else ===
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Render an assignment target with state-ref rewriting.
/// `self.count` / `.count` → `this.count`; bare state ident → `this.name`.
fn transpile_assign_target(expr: &Expr, ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
    match expr {
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" || name.as_str() == "." {
                    write!(out, "this.{}", field.as_str()).ok();
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
            let n = name.as_str();
            if ctx.is_state(n) {
                write!(out, "this.{}", n).ok();
            } else {
                write!(out, "{}", n).ok();
            }
        }
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Delegate an expression to the TypeScriptTrans transpiler for standard
/// transpilation. ArkTS is a TS subset, so the emitted output is valid ArkTS
/// for the long tail of expression kinds (f-strings, ranges, lambdas, casts,
/// option/result constructors, tuples, etc.) not handled explicitly above.
fn delegate_expr(expr: &Expr, _ctx: &ArkAdapterCtx, out: &mut Vec<u8>) {
    let mut ts = TypeScriptTrans::new("fragment".into());
    let mut sink = Sink::new("fragment".into());
    let _ = ts.expr(expr, &mut sink);
    let _ = out.write_all(&sink.body);
}

// ---------------------------------------------------------------------------
// Builtin module transpilation
// ---------------------------------------------------------------------------

/// Try to transpile a method call on a builtin module (e.g. `json.parse(x)`).
/// Returns true if the call was handled, false otherwise.
///
/// For ArkTS:
/// - `json.*` → `JSON.*` (same as Vue)
/// - `math.*` → `Math.*` (same as Vue)
/// - `date.*` → `Date.*` (same as Vue)
/// - `storage` / `event` / `router` → emit a TODO comment (HarmonyOS API
///   mapping is separate work; these are placeholders).
fn try_transpile_builtin_call(
    object: &Expr,
    method: &str,
    args: &Args,
    ctx: &ArkAdapterCtx,
    out: &mut Vec<u8>,
) -> bool {
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };

    match module {
        // json.parse(x) → JSON.parse(x); json.stringify(x) → JSON.stringify(x)
        "json" => {
            let js_method = match method {
                "parse" => "parse",
                "stringify" => "stringify",
                _ => return false,
            };
            write!(out, "JSON.{}(", js_method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // math.* → Math.* (ArkTS supports the standard Math object)
        "math" => {
            write!(out, "Math.{}(", method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // date.* → Date.*
        "date" => {
            write!(out, "Date.{}(", method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // storage — HarmonyOS API (e.g. @ohos.data.preferences) mapping TBD
        "storage" => {
            write!(out, "/* TODO(storage): map storage.{} to HarmonyOS prefs API */", method).ok();
            true
        }
        // event — HarmonyOS emitter API mapping TBD
        "event" => {
            write!(out, "/* TODO(event): map event.{} to HarmonyOS emitter API */", method).ok();
            true
        }
        // router — HarmonyOS router/navigation API mapping TBD
        "router" => {
            write!(
                out,
                "/* TODO(router): map router.{} to HarmonyOS router API */",
                method
            )
            .ok();
            true
        }
        _ => false,
    }
}

/// Try to transpile a field access on a builtin module (e.g. `router.path`).
/// Returns true if handled, false otherwise. Currently no Ark builtin fields
/// are mapped (HarmonyOS API work is separate).
fn try_transpile_builtin_field(
    _object: &Expr,
    _field: &str,
    _ctx: &ArkAdapterCtx,
    _out: &mut Vec<u8>,
) -> bool {
    false
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Arg, Args, Call, Expr, For, Iter, Name, Stmt, Store, StoreKind};

    fn ctx_empty() -> ArkAdapterCtx {
        ArkAdapterCtx::empty()
    }

    fn ctx_with(names: &[&str]) -> ArkAdapterCtx {
        ArkAdapterCtx::new(names.iter().map(|s| s.to_string()).collect())
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
    fn test_state_ref_dot_self() {
        // self.count → this.count
        let e = Expr::Dot(Box::new(ident("self")), Name::from("count"));
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "this.count");
    }

    #[test]
    fn test_state_ref_dot_dot() {
        // .count → this.count
        let e = Expr::Dot(Box::new(ident(".")), Name::from("count"));
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "this.count");
    }

    #[test]
    fn test_bare_state_ident() {
        // count → this.count when count ∈ state_fields
        let e = ident("count");
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_with(&["count"]), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "this.count");
    }

    #[test]
    fn test_bare_non_state_ident() {
        // x → x (no state field)
        let e = ident("x");
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        assert_eq!(String::from_utf8(out).unwrap(), "x");
    }

    #[test]
    fn test_print_becomes_console_log() {
        let stmt = Stmt::Expr(call("print", vec![Expr::Str("hi".into())]));
        let body = transpile_handler_body(&[stmt], &ctx_empty());
        assert!(body.contains("console.log("), "body = {}", body);
        assert!(!body.contains("await"), "no await in ArkTS: {}", body);
    }

    #[test]
    fn test_no_await_on_call() {
        let stmt = Stmt::Expr(call("fetchData", vec![]));
        let body = transpile_handler_body(&[stmt], &ctx_empty());
        assert!(
            !body.contains("await"),
            "ArkTS calls must not be prefixed with await: {}",
            body
        );
        assert!(body.contains("fetchData("), "body = {}", body);
    }

    #[test]
    fn test_store_let() {
        let store = Store {
            kind: StoreKind::Let,
            name: Name::from("x"),
            ty: Type::Unknown,
            expr: Expr::Int(42),
            attrs: Vec::new(),
        };
        let body = transpile_handler_body(&[Stmt::Store(store)], &ctx_empty());
        assert_eq!(body.trim(), "let x = 42;");
    }

    #[test]
    fn test_assignment_state_ref() {
        // self.count = self.count + 1
        let lhs = Expr::Dot(Box::new(ident("self")), Name::from("count"));
        let rhs = Expr::Bina(
            Box::new(Expr::Dot(Box::new(ident("self")), Name::from("count"))),
            auto_val::Op::Add,
            Box::new(Expr::Int(1)),
        );
        let expr = Expr::Bina(Box::new(lhs), auto_val::Op::Asn, Box::new(rhs));
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(
            body.contains("this.count = this.count + 1"),
            "body = {}",
            body
        );
    }

    #[test]
    fn test_math_builtin() {
        // math.floor(3.7) → Math.floor(3.7)
        let callee = Expr::Dot(Box::new(ident("math")), Name::from("floor"));
        let args = Args {
            args: vec![Arg::Pos(Expr::Float(3.7, "3.7".into()))],
        };
        let expr = Expr::Call(Call {
            name: Box::new(callee),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        });
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(body.contains("Math.floor("), "body = {}", body);
    }

    #[test]
    fn test_json_builtin() {
        // json.parse(s) → JSON.parse(s)
        let callee = Expr::Dot(Box::new(ident("json")), Name::from("parse"));
        let args = Args {
            args: vec![Arg::Pos(ident("s"))],
        };
        let expr = Expr::Call(Call {
            name: Box::new(callee),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        });
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(body.contains("JSON.parse("), "body = {}", body);
    }

    #[test]
    fn test_storage_todo_comment() {
        // storage.get('k') → /* TODO(storage): ... */
        let callee = Expr::Dot(Box::new(ident("storage")), Name::from("get"));
        let args = Args {
            args: vec![Arg::Pos(Expr::Str("k".into()))],
        };
        let expr = Expr::Call(Call {
            name: Box::new(callee),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        });
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(
            body.contains("TODO(storage)"),
            "storage should emit TODO comment: {}",
            body
        );
    }

    #[test]
    fn test_return_stmt() {
        let body = transpile_handler_body(
            &[Stmt::Return(Box::new(Expr::Int(7)))],
            &ctx_empty(),
        );
        assert_eq!(body.trim(), "return 7;");
    }

    #[test]
    fn test_return_nil() {
        let body = transpile_handler_body(
            &[Stmt::Return(Box::new(Expr::Nil))],
            &ctx_empty(),
        );
        assert_eq!(body.trim(), "return;");
    }

    #[test]
    fn test_for_named() {
        // for item in items { print(item) } → for (const item of items) { console.log(item); }
        let for_loop = For {
            iter: Iter::Named(Name::from("item")),
            range: ident("items"),
            body: Body::single_expr(call("print", vec![ident("item")])),
            new_line: false,
            init: None,
        };
        let body = transpile_handler_body(&[Stmt::For(for_loop)], &ctx_empty());
        assert!(
            body.contains("for (const item of items)"),
            "body = {}",
            body
        );
        assert!(body.contains("console.log(item)"), "body = {}", body);
    }

    #[test]
    fn test_for_cond_while() {
        // for i < 10 { ... } → while (i < 10) { ... }
        let cond = Expr::Bina(Box::new(ident("i")), auto_val::Op::Lt, Box::new(Expr::Int(10)));
        let for_loop = For {
            iter: Iter::Cond,
            range: cond,
            body: Body::single_expr(call("tick", vec![])),
            new_line: false,
            init: None,
        };
        let body = transpile_handler_body(&[Stmt::For(for_loop)], &ctx_empty());
        assert!(body.contains("while ("), "body = {}", body);
    }

    #[test]
    fn test_delegate_for_unknown_expr() {
        // Expr::Some(x) — not handled explicitly; should delegate without panic
        let e = Expr::Some(Box::new(ident("x")));
        let mut out = Vec::new();
        transpile_expr(&e, &ctx_empty(), &mut out);
        let s = String::from_utf8(out).unwrap();
        assert!(!s.is_empty(), "delegate should produce some output");
    }

    #[test]
    fn test_method_call_on_state_ref() {
        // self.notes.push(1) → this.notes.push(1)
        let callee = Expr::Dot(
            Box::new(Expr::Dot(Box::new(ident("self")), Name::from("notes"))),
            Name::from("push"),
        );
        let args = Args {
            args: vec![Arg::Pos(Expr::Int(1))],
        };
        let expr = Expr::Call(Call {
            name: Box::new(callee),
            args,
            ret: Type::Unknown,
            type_args: Vec::new(),
            pos: None,
        });
        let body = transpile_handler_body(&[Stmt::Expr(expr)], &ctx_empty());
        assert!(
            body.contains("this.notes.push(1)"),
            "body = {}",
            body
        );
    }
}
