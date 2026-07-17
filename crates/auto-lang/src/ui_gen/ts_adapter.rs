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
use crate::trans::Sink;
use crate::trans::typescript::TypeScriptTrans;
use std::collections::HashSet;
use std::io::Write;

/// Context for UI-specific rewrites during TypeScript generation.
pub struct AuraTsContext {
    /// Names of reactive state variables (need `.value` in Vue).
    pub state_names: HashSet<String>,
    /// Names of component props (need `props.` prefix in script, no `.value`).
    pub prop_names: HashSet<String>,
    /// Known API function names (need `await` prefix).
    api_functions: Vec<String>,
}

/// Default API function names (fallback when no dynamic list is provided)
const DEFAULT_API_FUNCTIONS: &[&str] = &[
    "listusers",
    "getuser",
    "getUser",
    "createUser",
    "updateUser",
    "deleteUser",
];

impl AuraTsContext {
    pub fn new(state_names: HashSet<String>) -> Self {
        Self {
            state_names,
            prop_names: HashSet::new(),
            api_functions: DEFAULT_API_FUNCTIONS.iter().map(|s| s.to_string()).collect(),
        }
    }

    pub fn with_props(mut self, prop_names: HashSet<String>) -> Self {
        self.prop_names = prop_names;
        self
    }

    /// Set custom API function names (from project's api.at)
    pub fn with_api_functions(mut self, functions: Vec<String>) -> Self {
        self.api_functions = functions;
        self
    }

    fn is_state(&self, name: &str) -> bool {
        self.state_names.contains(name)
    }

    fn is_prop(&self, name: &str) -> bool {
        self.prop_names.contains(name)
    }

    fn is_api(&self, name: &str) -> bool {
        self.api_functions.iter().any(|f| f == name)
    }
}

/// Convert a snake_case identifier to camelCase (for TS/JS output).
/// e.g. `list_notes` → `listNotes`, `create_note` → `createNote`
pub fn snake_to_camel(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = false;
    for c in name.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

/// Transpile a list of AutoLang statements to TypeScript, with AURA rewrites.
pub fn transpile_handler_body(stmts: &[Stmt], ctx: &AuraTsContext) -> String {
    let mut out = Vec::new();
    for (i, stmt) in stmts.iter().enumerate() {
        if i > 0 {
            write!(out, " ").ok();  // space separator (each stmt already ends with ;)
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
                StoreKind::Let => "let",
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
            // Plan 354: NavCall in handler body → router.push(path)
            if let Expr::NavCall { path, .. } = expr {
                write!(out, "router.push(").ok();
                transpile_expr(path, ctx, out);
                write!(out, ")").ok();
            } else {
                transpile_expr(expr, ctx, out);
            }
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
            // If returning nil/null in a void function, emit bare return
            match expr.as_ref() {
                Expr::Nil | Expr::Null => {
                    writeln!(out, "return;").ok();
                }
                _ => {
                    write!(out, "return ").ok();
                    transpile_expr(expr, ctx, out);
                    writeln!(out, ";").ok();
                }
            }
        }

        // Break
        Stmt::Break => {
            writeln!(out, "break;").ok();
        }

        // Fallback — delegate to a2ts for anything else
        _ => {
            let mut ts = TypeScriptTrans::new("fragment".into());
            let mut sink = Sink::new("fragment".into());
            let _ = ts.stmt(stmt, &mut sink);
            let _ = out.write_all(&sink.body);
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
        Iter::Destructured(key, val) => {
            // for (k, v) in map -> for (const [k, v] of Object.entries(map))
            write!(out, "for (const [{}, {}] of Object.entries(", key.as_str(), val.as_str()).ok();
            transpile_expr(&for_loop.range, ctx, out);
            write!(out, ")) {{").ok();
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
                if name.as_str() == "self" || name.as_str() == "." {
                    let field_name = field.as_str();
                    if ctx.is_prop(field_name) {
                        // Props need `props.` prefix in script
                        write!(out, "props.{}", field_name).ok();
                    } else if ctx.is_state(field_name) {
                        write!(out, "{}.value", field_name).ok();
                    } else {
                        write!(out, "{}", field_name).ok();
                    }
                    return;
                }
            }
            // General field access: object.field
            if try_transpile_builtin_field(obj, field.as_str(), ctx, out) {
                return;
            }
            transpile_expr(obj, ctx, out);
            write!(out, ".{}", field.as_str()).ok();
        }

        // Identifier — check if it's a reactive state variable
        Expr::Ident(name) => {
            if ctx.is_state(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else if name.as_str() == "self" || name.as_str() == "." {
                // In Vue <script setup>, self/this is not needed
                // Skip output — the field access will be handled by Expr::Dot
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }

        // String literal — escape newlines and quotes for JS single-quoted strings
        Expr::Str(s) => {
            let escaped = s
                .replace("\\", "\\\\")
                .replace("'", "\\'")
                .replace("\n", "\\n")
                .replace("\r", "\\r")
                .replace("\t", "\\t");
            write!(out, "'{}'", escaped).ok();
        }

        // Function call — API detection, print, builtins, method calls
        Expr::Call(call) => {
            match call.name.as_ref() {
                // Method call: object.method(args)
                Expr::Dot(object, method) => {
                    // D3 fix: self-method call (.MethodName()) — when object is
                    // "." or "self", this is a store sibling action call.
                    // Generate as bare MethodName() instead of .MethodName().
                    let is_self = matches!(object.as_ref(), Expr::Ident(name) if name.as_str() == "." || name.as_str() == "self");
                    if is_self {
                        // Check if it's a known builtin first
                        if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                            return;
                        }
                        // Generate as bare function call (store sibling action)
                        write!(out, "{}(", method.as_str()).ok();
                        for (i, arg) in call.args.args.iter().enumerate() {
                            if i > 0 {
                                write!(out, ", ").ok();
                            }
                            transpile_expr(&arg.get_expr(), ctx, out);
                        }
                        write!(out, ")").ok();
                        return;
                    }
                    if try_transpile_builtin_call(object, method.as_str(), &call.args, ctx, out) {
                        return;
                    }
                    // Handle common method call conversions
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
                        // Plan 345 (gap N1): Auto `.contains` -> JS `.includes`
                        // (JS strings and arrays both use .includes, not .contains).
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
                            // AutoLang notes.remove(idx) → TypeScript notes.value.splice(idx, 1)
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
                    } else if ctx.is_prop(func_name) {
                        // Plan 345 (gap K2/N4): callback prop call -> props.<name>(...)
                        write!(out, "props.{}", func_name).ok();
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
                Op::Add => {
                    // D2 fix: array concat. If RHS is an array literal,
                    // use spread syntax to avoid JS string coercion.
                    if matches!(rhs.as_ref(), Expr::Array(_)) {
                        transpile_expr(lhs, ctx, out);
                        write!(out, ".concat(").ok();
                        transpile_expr(rhs, ctx, out);
                        write!(out, ")").ok();
                    } else {
                        transpile_expr(lhs, ctx, out);
                        write!(out, " + ").ok();
                        transpile_expr(rhs, ctx, out);
                    }
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

        // Error propagation .?
        Expr::ErrorPropagate(expr) => {
            transpile_expr(expr, ctx, out);
            write!(out, "?.").ok();
        }

        // If expression (appears when parser treats if as RHS of let)
        Expr::If(if_expr) => {
            // Convert to IIFE so it works in expression position
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

        // Array index access: arr[idx] → arr.value[idx.value] for Vue refs
        Expr::Index(array, index) => {
            transpile_expr(array, ctx, out);
            write!(out, "[").ok();
            transpile_expr(index, ctx, out);
            write!(out, "]").ok();
        }

        // Closure: x => expr or (a, b) => expr
        // Must use transpile_expr (not delegate) so StateRef gets .value inside closures
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
            // Handle nested state ref: notes[.active_id].body → notes.value[active_id.value].body
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
    let mut sink = Sink::new("fragment".into());
    let _ = ts.expr(expr, &mut sink);
    let _ = out.write_all(&sink.body);
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
        // router.param("id") → (useRoute().params as any)["id"]
        // router.query("q") → (useRoute().query as any)["q"]
        "router" => {
            match method {
                "param" => {
                    write!(out, "(useRoute().params as any)[").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "]").ok();
                    true
                }
                "query" => {
                    write!(out, "(useRoute().query as any)[").ok();
                    for (i, arg) in args.args.iter().enumerate() {
                        if i > 0 {
                            write!(out, ", ").ok();
                        }
                        transpile_expr(&arg.get_expr(), ctx, out);
                    }
                    write!(out, "]").ok();
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

/// Try to transpile a field access on a builtin module (e.g. `route.path`).
/// Returns true if handled, false otherwise.
fn try_transpile_builtin_field(
    object: &Expr,
    field: &str,
    _ctx: &AuraTsContext,
    out: &mut Vec<u8>,
) -> bool {
    let module = match object {
        Expr::Ident(name) => name.as_str(),
        _ => return false,
    };

    match module {
        "router" => {
            match field {
                "path" => {
                    write!(out, "useRoute().path").ok();
                    true
                }
                _ => false,
            }
        }
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// API call detection (for async handler detection)
// ---------------------------------------------------------------------------

/// Check if any statement in the list contains an API function call.
pub fn stmts_contain_api_call(stmts: &[Stmt]) -> bool {
    let default_fns: Vec<String> = DEFAULT_API_FUNCTIONS.iter().map(|s| s.to_string()).collect();
    stmts_contain_api_call_with(stmts, &default_fns)
}

/// Check if any statement in the list contains an API function call (with custom function list).
pub fn stmts_contain_api_call_with(stmts: &[Stmt], api_fns: &[String]) -> bool {
    fn walk_expr(expr: &Expr, api_fns: &[String]) -> bool {
        match expr {
            Expr::Call(call) => {
                // Only simple identifier calls can be API functions;
                // method calls (Dot names) are never API calls.
                let is_api = call.get_name_text_safe()
                    .map(|name| api_fns.iter().any(|f| f == name.as_str()))
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

    fn check_stmts(stmts: &[Stmt], api_fns: &[String]) -> bool {
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

    check_stmts(stmts, api_fns)
}

// ---------------------------------------------------------------------------
// Route access detection (for useRoute import)
// ---------------------------------------------------------------------------

/// Check if Auto statements contain route access (Plan 235)
pub fn stmts_have_route_access(stmts: &[Stmt]) -> bool {
    fn walk_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(object, _) = call.name.as_ref() {
                    if let Expr::Ident(name) = object.as_ref() {
                        if name.as_str() == "route" {
                            return true;
                        }
                    }
                }
                call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
            }
            Expr::Dot(object, _) => {
                if let Expr::Ident(name) = object.as_ref() {
                    if name.as_str() == "route" {
                        return true;
                    }
                }
                walk_expr(object)
            }
            Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
            Expr::Unary(_, e) => walk_expr(e),
            Expr::Array(items) => items.iter().any(walk_expr),
            Expr::NavCall { .. } => true,
            _ => false,
        }
    }

    fn walk_stmt(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => walk_expr(expr),
            Stmt::Store(store) => walk_expr(&store.expr),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond) || b.body.stmts.iter().any(walk_stmt)),
            _ => false,
        }
    }

    stmts.iter().any(walk_stmt)
}

/// Check if Auto statements contain router navigation (Plan 235)
/// Detects router.push() or router.replace() calls to trigger useRouter import.
pub fn stmts_have_router_nav(stmts: &[Stmt]) -> bool {
    fn walk_expr(expr: &Expr) -> bool {
        match expr {
            Expr::Call(call) => {
                if let Expr::Dot(object, method) = call.name.as_ref() {
                    if let Expr::Ident(name) = object.as_ref() {
                        if name.as_str() == "router" && (method.as_str() == "push" || method.as_str() == "replace") {
                            return true;
                        }
                    }
                }
                call.args.args.iter().any(|a| walk_expr(&a.get_expr()))
            }
            Expr::Bina(l, _, r) => walk_expr(l) || walk_expr(r),
            Expr::Unary(_, e) => walk_expr(e),
            Expr::Array(items) => items.iter().any(walk_expr),
            Expr::NavCall { .. } => true,
            _ => false,
        }
    }

    fn walk_stmt(stmt: &Stmt) -> bool {
        match stmt {
            Stmt::Expr(expr) => walk_expr(expr),
            Stmt::Store(store) => walk_expr(&store.expr),
            Stmt::If(if_stmt) => if_stmt
                .branches
                .iter()
                .any(|b| walk_expr(&b.cond) || b.body.stmts.iter().any(walk_stmt)),
            _ => false,
        }
    }

    stmts.iter().any(walk_stmt)
}
