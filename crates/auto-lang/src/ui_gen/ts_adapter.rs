//! TypeScript adapter for AURA handler bodies.
//!
//! Wraps the a2ts transpiler to convert AutoLang AST fragments (from `on` blocks)
//! into TypeScript code, applying UI-specific rewrites:
//! - StateRef (`.count` / `self.count`) → `count.value` ref access
//! - API function calls → `await` prefix
//! - `print()` → `console.log()`
//!
//! Everything else (control flow, types, closures, pattern matching)
//! is delegated to the a2ts transpiler for standard expressions.

use crate::ast::*;
use crate::trans::typescript::TypeScriptTrans;
use std::collections::HashSet;
use std::io::Write;

/// Context for UI-specific rewrites during TypeScript generation.
pub struct AuraTsContext {
    /// Names of reactive state variables (need `.value` in Vue).
    pub state_names: HashSet<String>,
    /// Known API function names (need `await` prefix).
    api_functions: &'static [&'static str],
}

impl AuraTsContext {
    pub fn new(state_names: HashSet<String>) -> Self {
        Self {
            state_names,
            api_functions: &[
                "listusers",
                "getuser",
                "getUser",
                "createUser",
                "updateUser",
                "deleteUser",
            ],
        }
    }

    fn is_state(&self, name: &str) -> bool {
        self.state_names.contains(name)
    }

    fn is_api(&self, name: &str) -> bool {
        self.api_functions.contains(&name)
    }
}

/// Transpile a list of AutoLang statements to TypeScript, with AURA rewrites.
pub fn transpile_handler_body(stmts: &[Stmt], ctx: &AuraTsContext) -> String {
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

fn transpile_stmt(stmt: &Stmt, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match stmt {
        // Variable declarations — AURA-aware value rewriting
        Stmt::Store(store) => {
            let kw = match store.kind {
                StoreKind::Let => "const",
                StoreKind::Var => "let",
                StoreKind::Const => "const",
                _ => "let", // Shared, CVar, Field — shouldn't appear in handlers
            };
            write!(out, "{} {} = ", kw, store.name.as_str()).ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // Expression statements — AURA-aware (API calls, print, etc.)
        Stmt::Expr(expr) => {
            transpile_expr(expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // If/else if/else — write scaffolding, AURA-rewrite all expressions
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

        // Return
        Stmt::Return(expr) => {
            write!(out, "return").ok();
            write!(out, " ").ok();
            transpile_expr(expr, ctx, out);
            writeln!(out, ";").ok();
        }

        // Break
        Stmt::Break => {
            writeln!(out, "break;").ok();
        }

        // Fallback — delegate to a2ts for anything else
        _ => {
            let mut ts = TypeScriptTrans::new("fragment".into());
            let _ = ts.stmt(stmt, out);
        }
    }
}

fn transpile_body(body: &Body, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    for stmt in &body.stmts {
        transpile_stmt(stmt, ctx, out);
    }
}

fn transpile_for(for_loop: &For, ctx: &AuraTsContext, out: &mut Vec<u8>) {
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
            // for i, item in range { ... } → range.forEach((item, i) => { ... })
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ".forEach(({}, {}) => {{", iter_name.as_str(), index_name.as_str()).ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}});").ok();
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
            write!(out, ")").ok();
            write!(out, ") {{").ok();
            transpile_body(&for_loop.body, ctx, out);
            writeln!(out, "}}").ok();
        }
    }
}

// ---------------------------------------------------------------------------
// Expression transpilation — AURA-aware with a2ts delegation
// ---------------------------------------------------------------------------

fn transpile_expr(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match expr {
        // === AURA-specific rewrites ===

        // StateRef: `.count` is parsed as Expr::Dot(Ident("self"), "count")
        // → `count.value` for Vue reactive refs
        // General field access: object.field → transpile object, then emit .field
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" {
                    let field_name = field.as_str();
                    if ctx.is_state(field_name) {
                        write!(out, "{}.value", field_name).ok();
                    } else {
                        write!(out, "{}", field_name).ok();
                    }
                    return;
                }
            }
            // General field access: object.field
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }

        // Identifier — check if it's a reactive state variable
        Expr::Ident(name) => {
            if ctx.is_state(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else if name.as_str() == "self" {
                write!(out, "this").ok();
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }

        // Function call — API detection, print, builtins, method calls
        Expr::Call(call) => {
            match call.name.as_ref() {
                // Method call: object.method(args)
                Expr::Dot(object, method) => {
                    if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                        return;
                    }
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
                // Regular function call
                Expr::Ident(name) => {
                    let func_name = name.as_str();
                    // API calls need `await`
                    if ctx.is_api(func_name) {
                        write!(out, "await {}", func_name).ok();
                    } else if func_name == "print" {
                        write!(out, "console.log").ok();
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
                // Fallback — delegate to a2ts for complex call names
                _ => delegate_expr(expr, ctx, out),
            }
        }

        // Binary ops — AURA-aware on both sides
        Expr::Bina(lhs, op, rhs) => {
            // Handle assignment operators specially (target needs StateRef check)
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
                Op::Dot => {
                    // Field access: already handled by Expr::Dot arm above,
                    // but Op::Dot can also appear in Bina. Delegate.
                    delegate_expr(expr, ctx, out);
                }
                _ => {
                    // Standard binary op
                    transpile_expr(lhs, ctx, out);
                    write!(out, " {} ", op.op()).ok();
                    transpile_expr(rhs, ctx, out);
                }
            }
        }

        // Unary ops — AURA-aware on operand
        Expr::Unary(op, operand) => {
            let op_str = match op {
                auto_val::Op::Sub => "-",
                auto_val::Op::Not => "!",
                _ => "",
            };
            write!(out, "{}", op_str).ok();
            transpile_expr(operand, ctx, out);
        }

        // === Delegate to a2ts for everything else ===
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Extract assignment target with StateRef rewriting.
fn transpile_assign_target(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match expr {
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" {
                    let field_name = field.as_str();
                    if ctx.is_state(field_name) {
                        write!(out, "{}.value", field_name).ok();
                    } else {
                        write!(out, "{}", field_name).ok();
                    }
                    return;
                }
            }
            delegate_expr(expr, ctx, out);
        }
        Expr::Ident(name) => {
            if ctx.is_state(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }
        _ => delegate_expr(expr, ctx, out),
    }
}

/// Delegate expression to a2ts transpiler for standard transpilation.
/// Handles: literals, arrays, objects, lambdas, closures, f-strings,
/// indexing, ranges, tag construction, etc.
fn delegate_expr(expr: &Expr, _ctx: &AuraTsContext, out: &mut Vec<u8>) {
    let mut ts = TypeScriptTrans::new("fragment".into());
    let _ = ts.expr(expr, out);
}

// ---------------------------------------------------------------------------
// Builtin module transpilation (Plan 235: storage, event, json, math, date)
// ---------------------------------------------------------------------------

/// Try to transpile a method call on a builtin module (e.g. `json.parse(x)`).
/// Returns true if the call was handled, false otherwise.
fn try_transpile_builtin_call(
    object: &Expr,
    method: &str,
    args: &Args,
    ctx: &AuraTsContext,
    out: &mut Vec<u8>,
) -> bool {
    // Extract the module name from the object expression
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
        // storage.get(x) → localStorage.getItem(x); storage.set(x, y) → localStorage.setItem(x, y)
        "storage" => {
            let js_method = match method {
                "get" => "getItem",
                "set" => "setItem",
                "remove" => "removeItem",
                "clear" => "clear",
                _ => return false,
            };
            write!(out, "localStorage.{}(", js_method).ok();
            for (i, arg) in args.args.iter().enumerate() {
                if i > 0 {
                    write!(out, ", ").ok();
                }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
            true
        }
        // event.dispatch(name) → window.dispatchEvent(new CustomEvent(name))
        // event.dispatch(name, detail) → window.dispatchEvent(new CustomEvent(name, detail))
        "event" => {
            match method {
                "dispatch" => {
                    write!(out, "window.dispatchEvent(new CustomEvent(").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "))").ok();
                    true
                }
                "listen" => {
                    // event.listen(name, handler) → window.addEventListener(name, handler)
                    write!(out, "window.addEventListener(").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, ")").ok();
                    true
                }
                _ => false,
            }
        }
        // math.random() → Math.random(); math.floor(x) → Math.floor(x)
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
        // date.now() → Date.now()
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
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// API call detection (for async handler detection)
// ---------------------------------------------------------------------------

/// Check if any statement in the list contains an API function call.
pub fn stmts_contain_api_call(stmts: &[Stmt]) -> bool {
    const API_FNS: &[&str] = &[
        "listusers",
        "getuser",
        "getUser",
        "createUser",
        "updateUser",
        "deleteUser",
    ];

    fn walk_expr(expr: &Expr, api_fns: &[&str]) -> bool {
        match expr {
            Expr::Call(call) => {
                // Only simple identifier calls can be API functions;
                // method calls (Dot names) are never API calls.
                let is_api = call.get_name_text_safe()
                    .map(|name| api_fns.contains(&name.as_str()))
                    .unwrap_or(false);
                is_api || call.args.args.iter().any(|a| walk_expr(&a.get_expr(), api_fns))
            }
            Expr::Bina(l, _, r) => walk_expr(l, api_fns) || walk_expr(r, api_fns),
            Expr::Unary(_, e) => walk_expr(e, api_fns),
            Expr::Dot(obj, _) => walk_expr(obj, api_fns),
            Expr::Array(items) => items.iter().any(|e| walk_expr(e, api_fns)),
            _ => false,
        }
    }

    fn check_stmts(stmts: &[Stmt], api_fns: &[&str]) -> bool {
        stmts.iter().any(|s| match s {
            Stmt::Expr(expr) => walk_expr(expr, api_fns),
            Stmt::Store(store) => walk_expr(&store.expr, api_fns),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond, api_fns) || check_stmts(&b.body.stmts, api_fns)),
            _ => false,
        })
    }

    check_stmts(stmts, API_FNS)
}
