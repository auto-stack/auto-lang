# Plan 187: AURA-to-TypeScript Adapter via a2ts Delegation

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Vue generator's inline `stmt_to_js`/`expr_to_js` with delegation to the a2ts transpiler, so AURA handler bodies produce proper TypeScript instead of lossy JavaScript.

**Architecture:** Keep the original AutoLang AST `Stmt`/`Expr` nodes from `on` block extraction (instead of converting to `AuraStmt`/`AuraExpr` IR). Pass them to a2ts for TypeScript generation, with a thin adapter that handles UI-specific rewrites (StateRef → `.value`, MsgVariant → dispatch, API calls → async/await).

**Tech Stack:** Rust, a2ts transpiler (`crates/auto-lang/src/trans/ts_*`), Vue generator (`crates/auto-lang/src/ui_gen/vue.rs`)

---

## Current State

### Vue generator inline transpiler (vue.rs)

**`stmt_to_js`** (line 2690) handles 3 AuraStmt variants: Assign, Update, MethodCall.
**`expr_to_js`** (line 2575) handles 15 AuraExpr variants: Literal, Int, Float, Bool, StateRef, MsgVariant, Binary, Unary, MethodCall, Array, Object, Lambda, FieldAccess, NavCall, Constructor.

### a2ts transpiler

**`ts_stmt.rs`** handles 17 Stmt variants including: Store, Fn, If, For, Break, Return, Is (pattern matching), TypeDecl, EnumDecl.
**`ts_expr.rs`** handles 21+ Expr variants including: all literals, FStr (template strings), Ident, Bina, Dot, Unary, Call, Array, Index, Range, Object, Lambda, Closure, Block, Cover (tag construction).
**`ts_types.rs`** maps all 25 Type variants to TypeScript types.

### Gap analysis

What the Vue generator **cannot** do today but a2ts already supports:
- `if`/`else` conditionals in handler bodies
- `for` loops (range, for-each) in handler bodies
- `return` statements
- Pattern matching (`is` expressions)
- F-strings (template literals)
- Block expressions
- Closures (not just arrow functions)
- Array indexing
- Type annotations on locals

What a2ts does **not** handle (UI-specific):
- `StateRef` → Vue `.value` ref access
- `MsgVariant` → message dispatch (`Msg.Inc`)
- `NavCall` → `router.push(...)`
- API call detection → async/await
- `.len` → `.length` mapping

---

## Phase 1: Create the adapter module

**Files:**
- Create: `crates/auto-lang/src/ui_gen/ts_adapter.rs`

**Step 1: Create `ts_adapter.rs` with the adapter struct**

```rust
//! TypeScript adapter for AURA handler bodies.
//!
//! Wraps the a2ts transpiler to convert AutoLang AST fragments (from `on` blocks)
//! into TypeScript code, applying UI-specific rewrites:
//! - StateRef (identifiers starting with `.`) → `.value` ref access
//! - API function calls → async/await
//! - `.len` → `.length`
//!
//! Everything else (control flow, types, closures, pattern matching)
//! is handled by the a2ts transpiler directly.

use crate::ast::{Expr, Stmt};
use crate::trans::ts_expr::TsExpr;
use crate::trans::ts_stmt::TsStmt;
use crate::trans::ts_types;
use std::collections::HashSet;
use std::io::Write;

/// Context for UI-specific rewrites during TypeScript generation.
pub struct AuraTsContext {
    /// Names of reactive state variables (need `.value` in Vue).
    pub state_names: HashSet<String>,
    /// Known API function names (need `await` prefix).
    pub api_functions: &'static [&'static str],
}

impl AuraTsContext {
    pub fn new(state_names: HashSet<String>) -> Self {
        Self {
            state_names,
            api_functions: &[
                "listusers", "getuser", "getUser",
                "createUser", "updateUser", "deleteUser",
            ],
        }
    }
}

/// Transpile a list of AutoLang statements to TypeScript, with AURA rewrites.
pub fn transpile_handler_body(
    stmts: &[Stmt],
    ctx: &AuraTsContext,
) -> String {
    let mut out = Vec::new();
    for stmt in stmts {
        transpile_stmt(stmt, ctx, &mut out);
    }
    String::from_utf8(out).unwrap_or_default()
}

fn transpile_stmt(stmt: &Stmt, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match stmt {
        // For Store and Expr, apply AURA-aware expression rewriting
        Stmt::Store(store) => {
            // let x = value → const x = value (with StateRef rewriting on value)
            write!(out, "const {} = ", store.name.as_str()).ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out, ";").ok();
        }
        Stmt::Expr(expr) => {
            transpile_expr(expr, ctx, out);
            writeln!(out, ";").ok();
        }
        // For everything else, delegate to a2ts
        // (will be expanded in Phase 2)
        _ => {
            // Fallback: use a2ts statement transpiler
            let mut buf = Vec::new();
            let mut ts = TsExpr::new();
            // TODO: Phase 2 — delegate to TsStmt
            let _ = ts;
            let _ = buf;
            // For now, emit comment placeholder
            writeln!(out, "// TODO: unsupported statement").ok();
        }
    }
}
```

**Step 2: Implement `transpile_expr` with AURA rewrites**

This is the core adapter. It rewrites AutoLang `Expr` nodes before/instead of delegating to a2ts:

```rust
fn transpile_expr(expr: &Expr, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match expr {
        // === AURA-specific rewrites (not in a2ts) ===

        // StateRef: `.count` → `count.value`  (parsed as Expr::Dot(Ident("self"), "count"))
        Expr::Dot(obj, field) => {
            if let Expr::Ident(name) = obj.as_ref() {
                if name.as_str() == "self" {
                    let name = field.as_str();
                    if ctx.state_names.contains(name) {
                        write!(out, "{}.value", name).ok();
                    } else {
                        write!(out, "{}", name).ok();
                    }
                    return;
                }
            }
            // Regular field access — delegate to a2ts
            delegate_expr(expr, ctx, out);
        }

        // Identifier that's a state name → `.value`
        Expr::Ident(name) => {
            if ctx.state_names.contains(name.as_str()) {
                write!(out, "{}.value", name.as_str()).ok();
            } else {
                write!(out, "{}", name.as_str()).ok();
            }
        }

        // Function call — check for API functions and print
        Expr::Call(call) => {
            let func_name = call.get_name_text().to_string();
            if ctx.api_functions.contains(&func_name.as_str()) {
                write!(out, "await {}", func_name).ok();
            } else if func_name == "print" {
                write!(out, "console.log").ok();
            } else {
                write!(out, "{}", func_name).ok();
            }
            write!(out, "(").ok();
            for (i, arg) in call.args.args.iter().enumerate() {
                if i > 0 { write!(out, ", ").ok(); }
                transpile_expr(&arg.get_expr(), ctx, out);
            }
            write!(out, ")").ok();
        }

        // === Delegate everything else to a2ts ===
        _ => delegate_expr(expr, ctx, out),
    }
}
```

**Step 3: Implement `delegate_expr` for a2ts delegation**

```rust
fn delegate_expr(expr: &Expr, _ctx: &AuraTsContext, out: &mut Vec<u8>) {
    // Use a2ts expression transpiler for standard expressions
    // This handles: literals, binary ops, unary ops, arrays, objects,
    // lambdas, closures, f-strings, indexing, ranges, etc.
    let mut ts = TsExpr::new();
    ts.expr(expr, out).ok();
}
```

**Step 4: Register the module**

Add to `crates/auto-lang/src/ui_gen/mod.rs`:
```rust
pub mod ts_adapter;
```

**Step 5: Run `cargo build -p auto-lang`**

Expected: compiles clean (may need to make `TsExpr::new()` and `TsExpr::expr()` public if they aren't already).

---

## Phase 2: Full a2ts delegation for statements

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/ts_adapter.rs`

**Step 1: Make a2ts `TsStmt` and `TsExpr` usable as libraries**

Check if `TsStmt::stmt()` and `TsExpr::expr()` are public methods. If not, make them public. They need to accept `&Stmt`/`&Expr` and write to `&mut impl Write`.

The key issue: `TsStmt` is a method on `TypeScriptTrans`, which requires constructing the full transpiler. We need either:
- Extract `TsStmt` and `TsExpr` as standalone functions, or
- Create a lightweight `TypeScriptTrans` instance for fragment transpilation

Check the actual method signatures in `ts_stmt.rs` and `ts_expr.rs`. If they're `&mut self` methods on `TypeScriptTrans`, create a helper:

```rust
/// Create a minimal TypeScriptTrans for fragment-level transpilation.
fn ts_for_fragment() -> TypeScriptTrans {
    TypeScriptTrans::new("fragment".into())
}
```

**Step 2: Delegate all non-AURA statements to TsStmt**

Expand the `transpile_stmt` match to delegate:

```rust
fn transpile_stmt(stmt: &Stmt, ctx: &AuraTsContext, out: &mut Vec<u8>) {
    match stmt {
        // AURA-aware: Store with StateRef rewriting on the value
        Stmt::Store(store) => {
            let kw = match store.kind {
                crate::ast::StoreKind::Let => "const",
                crate::ast::StoreKind::Var => "let",
            };
            write!(out, "{} {} = ", kw, store.name.as_str()).ok();
            transpile_expr(&store.expr, ctx, out);
            writeln!(out, ";").ok();
        }
        // AURA-aware: Expression statements with API call / print rewriting
        Stmt::Expr(expr) => {
            transpile_expr(expr, ctx, out);
            writeln!(out, ";").ok();
        }
        // Delegate to a2ts: if/else, for, return, break, etc.
        Stmt::If(if_stmt) => {
            let mut ts = ts_for_fragment();
            ts.if_stmt(if_stmt, &mut *out).ok();
        }
        Stmt::For(for_loop) => {
            let mut ts = ts_for_fragment();
            ts.for_loop(for_loop, &mut *out).ok();
        }
        Stmt::Return(expr) => {
            write!(out, "return ").ok();
            if let Some(e) = expr {
                transpile_expr(e, ctx, out);
            }
            writeln!(out, ";").ok();
        }
        Stmt::Break => writeln!(out, "break;").ok(),
        // Fallback for any other statement type
        _ => {
            let mut ts = ts_for_fragment();
            ts.stmt(stmt, &mut *out).ok();
        }
    }
}
```

**Step 3: Handle nested expressions in delegated code**

The key challenge: when TsStmt delegates to TsExpr for sub-expressions, those sub-expressions won't get the AURA rewrites (StateRef, API calls). We need to ensure that our `transpile_expr` adapter is called at every expression level, not just the top level.

**Approach:** Rather than delegating whole statements to TsStmt, we keep full control of the output but call individual a2ts helpers for expression-level transpilation. The adapter `transpile_expr` handles AURA-specific cases and delegates the rest to `TsExpr::expr()`. For statements, we write the control flow scaffolding ourselves and call `transpile_expr` for all expression positions.

This means:
- `if`/`for`/`return` scaffolding is written by the adapter
- All expression positions within those statements go through `transpile_expr` (which applies AURA rewrites before delegating to a2ts)

**Step 4: Run `cargo build -p auto-lang` and `cargo test -p auto-lang --lib`**

---

## Phase 3: Integrate adapter into Vue generator

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`
- Modify: `crates/auto-lang/src/aura/extract.rs`

**Step 1: Change `LogicPayload` to carry original AST**

Currently `extract_on_block` converts AST to `AuraStmt` IR. Instead, keep the original `Stmt` nodes:

In `crates/auto-lang/src/aura/types.rs`, add a new variant to `LogicPayload`:
```rust
pub enum LogicPayload {
    AstBlock(Vec<AuraStmt>),      // existing — keep for backward compat
    AstStmts(Vec<Stmt>),           // new — original AST nodes
    Bytecode(Vec<u8>),
}
```

In `crates/auto-lang/src/aura/extract.rs`, change `extract_on_block` to also produce `AstStmts`:
```rust
fn extract_on_block(on: &OnBlock) -> ExtractResult<HashMap<String, LogicPayload>> {
    let mut handlers = HashMap::new();
    for handler in &on.handlers {
        let pattern = handler.pattern.clone();
        // Keep original AST stmts for a2ts delegation
        let original_stmts = handler.body.stmts.clone();
        handlers.insert(pattern, LogicPayload::AstStmts(original_stmts));
    }
    Ok(handlers)
}
```

**Step 2: Update `generate_handler_body` to use the adapter**

In `crates/auto-lang/src/ui_gen/vue.rs`, change `generate_handler_body`:

```rust
fn generate_handler_body(&self, payload: &LogicPayload) -> GenResult<String> {
    match payload {
        LogicPayload::AstStmts(stmts) => {
            let ctx = AuraTsContext::new(self.state_names.clone());
            Ok(crate::ui_gen::ts_adapter::transpile_handler_body(stmts, &ctx))
        }
        LogicPayload::AstBlock(stmts) => {
            // Legacy path — still works for simple cases
            let body: Vec<String> = stmts.iter()
                .map(|s| self.stmt_to_js(s))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(body.join("\n  "))
        }
        LogicPayload::Bytecode(_) => {
            Err(GenError::UnsupportedStmt("Bytecode not supported in Vue generator".to_string()))
        }
    }
}
```

**Step 3: Handle async detection with the new payload**

The `handler_has_api_calls` function currently inspects `LogicPayload::AstBlock`. Update it:

```rust
fn handler_has_api_calls(&self, payload: &LogicPayload) -> bool {
    match payload {
        LogicPayload::AstStmts(stmts) => {
            // Walk the original AST to check for API function calls
            Self::stmts_contain_api_call(stmts)
        }
        LogicPayload::AstBlock(stmts) => {
            stmts.iter().any(|s| self.stmt_has_api_call(s))
        }
        LogicPayload::Bytecode(_) => false,
    }
}

fn stmts_contain_api_call(stmts: &[Stmt]) -> bool {
    fn walk_expr(expr: &Expr, api_fns: &[&str]) -> bool {
        match expr {
            Expr::Call(call) => {
                let name = call.get_name_text().to_string();
                api_fns.contains(&name.as_str())
                    || call.args.args.iter().any(|a| walk_expr(&a.get_expr(), api_fns))
            }
            Expr::Bina(l, _, r) => walk_expr(l, api_fns) || walk_expr(r, api_fns),
            Expr::Unary(_, e) => walk_expr(e, api_fns),
            Expr::Dot(obj, _) => walk_expr(obj, api_fns),
            _ => false,
        }
    }
    let api_fns = ["listusers", "getuser", "getUser", "createUser", "updateUser", "deleteUser"];
    stmts.iter().any(|s| match s {
        Stmt::Expr(expr) => walk_expr(expr, &api_fns),
        Stmt::Store(store) => walk_expr(&store.expr, &api_fns),
        _ => false,
    })
}
```

**Step 4: Update NavCall handling**

NavCall (`router.push(...)`) is currently extracted into `AuraExpr::NavCall`. With original AST, router calls would appear as method calls. The adapter's `transpile_expr` should detect router method calls:

```rust
// In transpile_expr, add before the Call arm:
Expr::Dot(obj, method) => {
    // Check for router.push("path")
    if let Expr::Ident(name) = obj.as_ref() {
        if name.as_str() == "router" && method.as_str() == "push" {
            // This is handled by the Call arm since router.push() is Expr::Call
            // Fall through to delegate_expr
        }
    }
    delegate_expr(expr, ctx, out);
}
```

Actually, `router.push(...)` in the AST is likely `Expr::Call` with name being a dot expression. The existing NavCall extraction would need to stay, or the adapter needs to recognize the pattern. For now, keep NavCall in the legacy `AstBlock` path as a fallback — Phase 4 will handle this cleanly.

**Step 5: Run `cargo build -p auto-lang`**

**Step 6: Test with 006-hero-section**

Run `auto run` on `examples/ui/006-hero-section` and verify:
- `print("Getting started!")` → `console.log("Getting started!");`
- The generated App.vue has proper TypeScript

**Step 7: Create a test with conditionals**

Create a temporary test .at file with `if`/`else` in an `on` block to verify delegation works:
```auto
widget Test {
    msg Msg { Click }
    model { var count int = 0 }
    view {
        button "Click" { onclick: .Click }
    }
    on {
        .Click -> {
            if .count > 10 {
                .count = 0
            } else {
                .count = .count + 1
            }
        }
    }
}
```

Verify the generated handler contains proper TypeScript `if/else`.

---

## Phase 4: Clean up and remove legacy path

**Files:**
- Modify: `crates/auto-lang/src/ui_gen/vue.rs`
- Modify: `crates/auto-lang/src/aura/types.rs`
- Modify: `crates/auto-lang/src/aura/extract.rs`

**Step 1: Remove `AuraStmt` and `AuraExpr` from extraction path**

Once the adapter handles all cases, remove `extract_body_stmts`, `extract_assignment_from_expr`, `extract_fn_call_from_expr`, and `extract_expr` (the AURA-specific versions) from `extract.rs`. Keep the `extract_view_tree` and `extract_widget_from_decl` functions which still use `AuraExpr` for the view tree (not handler bodies).

**Step 2: Remove `stmt_to_js` and `expr_to_js` from vue.rs**

Delete these methods entirely — they're replaced by the adapter.

**Step 3: Remove `LogicPayload::AstBlock` variant**

All handler bodies now use `AstStmts`.

**Step 4: Run full test suite**

```bash
cargo test -p auto-lang --lib
cargo test -p auto-man
```

**Step 5: Test all UI examples**

Run `auto run` on each example in `examples/ui/` to verify no regressions.

---

## Key Decisions

- **Keep AuraExpr for view tree**: The view tree extraction (`extract_view_tree`) still uses `AuraExpr`/`AuraNode`. This refactor only affects handler body transpilation. The view tree has fundamentally different needs (HTML element mapping, event binding, shadcn component detection).
- **Adapter pattern, not fork**: We wrap a2ts, not fork it. The adapter only intercepts expressions that need AURA rewrites (StateRef, API calls, print) and delegates everything else to a2ts.
- **Incremental migration**: Phase 3 keeps the legacy `AstBlock` path as fallback. Phase 4 removes it once everything works.
- **NavCall stays special**: Router navigation (`NavCall`) is an AURA concept that doesn't map to standard AutoLang. It stays in the view tree extraction, not in handler bodies.

## Verification

1. `cargo build` — compiles clean
2. `cargo test -p auto-lang --lib` — all tests pass
3. `cargo test -p auto-man` — all tests pass
4. `auto run` on 006-hero-section — `print()` → `console.log()`
5. Handler with `if/else` — generates proper TypeScript conditionals
6. Handler with `for` loop — generates proper TypeScript loop
7. StateRef in complex expressions (`.count + 1`) — `.value` applied correctly
