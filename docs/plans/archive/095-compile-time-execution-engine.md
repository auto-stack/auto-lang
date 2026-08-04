# Plan 095: Compile-Time Execution Engine (CTEE)

> **Status**: ✅ Complete
> **Priority**: Critical (blocks self-hosting)
> **Dependencies**: Plan 094 (Hybrid FFI Bridge), Plan 081 (AutoVM Default Mode)
> **Design Doc**: [docs/design/compile-time-execution.md](../design/compile-time-execution.md) (Finalized)

## Implementation Progress

| Phase | Task | Status | Notes |
|-------|------|--------|-------|
| 1 | Lexer & Tokens | ✅ Complete | `HashIf`, `HashFor`, `HashIs`, `HashBrace` tokens |
| 2 | AST Nodes | ✅ Complete | `HashIf`, `HashFor`, `HashIs`, `HashBrace` structs |
| 3 | Parser | ✅ Complete | All comptime constructs parsed |
| 4 | CTEE Module | ✅ Complete | Using `VmInterpreter` for evaluation |
| 5 | Integration | ✅ Complete | Integrated into `run_autovm`, `transpile_c`, `transpile_rust` |
| 6 | Error Reporting | ✅ Complete | `ComptimeError` type with codes E0401-E0406 |

---

# Part A: High-Level Architecture

## Executive Summary

The Auto Compiler requires a **Compile-Time Execution Engine (CTEE)** that embeds the AutoVM to execute Auto code during compilation. This is **indispensable** for self-hosting because:

1. **const evaluation**: Compile-time constant expressions (`#{ ... }`)
2. **Comptime control flow**: `#if`, `#for`, `#is` constructs
3. **Type constraints**: Generic where-clause evaluation
4. **Code generation**: Macro-like metaprogramming

**Key Insight**: The a2r (Auto-to-Rust) transpiler is NOT just a transpiler—it's a **transpiler + embedded AutoVM**. Without the AutoVM, the compiler cannot evaluate compile-time expressions.

**Syntax Reference**: See [docs/design/compile-time-execution.md](../design/compile-time-execution.md) for the complete syntax specification using the `#` prefix.

## Motivation

### The Problem

Consider this AutoLang code using the official `#` syntax:

```auto
// Compile-time constant with evaluation block
const BUFFER_SIZE = 1024 * 2  // Must be evaluated at compile time

// Compile-time conditional (only one branch is emitted)
#if OS == "windows" {
    fn init() { init_win32() }
} elif OS == "linux" {
    fn init() { init_linux() }
} else {
    compile_error("Unsupported OS")
}

// Compile-time loop unrolling
#for i in 0..3 {
    print(#{i})  // #{i} interpolates compile-time value
}

// Complex compile-time computation
const CRC_TABLE = #{
    var t = [0; 256]
    for i in 0..256 {
        var c = i
        for j in 0..8 {
            if (c & 1) != 0 { c = (c >> 1) ^ POLY }
            else { c = c >> 1 }
        }
        t[i] = c
    }
    t  // Return to const
}
```

For this to work, the compiler needs to:
1. Evaluate `1024 * 2` to get `2048` (const evaluation)
2. Execute `#if` branches and emit only the matching one (AST pruning)
3. Unroll `#for` loops and interpolate `#{i}` values (AST expansion)
4. Execute `#{ ... }` blocks and convert results to literals (evaluation)

### Why AutoVM is Required

```
┌─────────────────────────────────────────────────────────────────────┐
│                     Compilation Pipeline with CTEE                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   Auto Source (.at)                                                  │
│        ↓                                                             │
│   Lexer → Parser → AST                                               │
│        ↓                                                             │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  Compile-Time Execution Engine (CTEE)                      │    │
│   │                                                            │    │
│   │  ┌──────────────────────────────────────────────────────┐ │    │
│   │  │  Embedded AutoVM (sandboxed, deterministic)          │ │    │
│   │  │                                                      │ │    │
│   │  │  • Evaluate const expressions                        │ │    │
│   │  │  • Execute comptime blocks                           │ │    │
│   │  │  • Evaluate type constraints                         │ │    │
│   │  │  • Run metaprogramming code                          │ │    │
│   │  └──────────────────────────────────────────────────────┘ │    │
│   │                                                            │    │
│   │  Output: Computed values, generated AST nodes, facts      │    │
│   └────────────────────────────────────────────────────────────┘    │
│        ↓                                                             │
│   ┌────────────────────────────────────────────────────────────┐    │
│   │  Transpilation (a2r / a2c)                                 │    │
│   │                                                            │    │
│   │  • Generate Rust/C code using compile-time results        │    │
│   └────────────────────────────────────────────────────────────┘    │
│        ↓                                                             │
│   Native Binary                                                      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Compile-Time Execution Engine                     │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                     CTEE Coordinator                         │   │
│   │                                                             │   │
│   │  • Orchestrates compile-time execution                      │   │
│   │  • Manages sandbox boundaries                               │   │
│   │  • Handles errors and limits                                │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                    Embedded AutoVM                           │   │
│   │                                                             │   │
│   │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐ │   │
│   │  │  Codegen        │  │  Bytecode       │  │  Runtime    │ │   │
│   │  │  (AST → ABC)    │→ │  Executor       │→ │  Values     │ │   │
│   │  └─────────────────┘  └─────────────────┘  └─────────────┘ │   │
│   │                                                             │   │
│   │  + Deterministic Mode (no I/O, no randomness)              │   │
│   │  + Resource Limits (time, memory, recursion)               │   │
│   │  + Sandbox Isolation (separate from runtime VM)            │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                              │                                       │
│                              ▼                                       │
│   ┌─────────────────────────────────────────────────────────────┐   │
│   │                    CTEE Output                               │   │
│   │                                                             │   │
│   │  • Computed const values (for codegen)                      │   │
│   │  • Generated AST nodes (for macros)                         │   │
│   │  • Type facts (for type checking)                           │   │
│   │  • Side effects (for comptime blocks)                       │   │
│   └─────────────────────────────────────────────────────────────┘   │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### Integration Points

| Component | Integration Point | Description |
|-----------|------------------|-------------|
| **Parser** | `const` expressions | Evaluate after parsing |
| **Type Checker** | `where` clauses | Evaluate constraint predicates |
| **Codegen** | Array sizes, const values | Substitute computed values |
| **a2r/a2c** | Full CTEE access | Embedded in transpiler |

## Minimum AutoVM Feature Set for Self-Hosting

### Tier 1: Critical (MUST have for basic self-hosting)

| Feature | Description | Use Case |
|---------|-------------|----------|
| **Arithmetic** | `+`, `-`, `*`, `/`, `%` | const expressions |
| **Comparison** | `==`, `!=`, `<`, `>`, `<=`, `>=` | const conditions |
| **Boolean logic** | `and`, `or`, `not` | const conditions |
| **Integer literals** | `i32`, `i64`, `u8`, etc. | const values |
| **String literals** | `"..."` | const strings |
| **Variables** | `let`, `mut` | local state in comptime |
| **Functions** | `fn` | reusable compile-time code |
| **Control flow** | `if/else`, `loop`, `for` | comptime logic |
| **Arrays** | `[T; N]` | const arrays |
| **Basic FFI** | Minimal native calls | compile-time helpers |

**Estimated Coverage**: ~60% of AutoVM features

### Tier 2: Important (SHOULD have for comfortable self-hosting)

| Feature | Description | Use Case |
|---------|-------------|----------|
| **Structs/Types** | `type` declarations | type introspection |
| **Methods** | `fn method(self)` | type methods in comptime |
| **Generics** | `List<T>` | generic type checking |
| **Pattern matching** | `is` expressions | type analysis |
| **String operations** | concat, format | code generation |
| **Lists** | `List<T>` dynamic | AST manipulation |
| **Maps** | `HashMap<K, V>` | symbol tables |

**Estimated Coverage**: ~80% of AutoVM features

### Tier 3: Nice-to-Have (for advanced metaprogramming)

| Feature | Description | Use Case |
|---------|-------------|----------|
| **Reflection** | Type introspection | derive macros |
| **AST manipulation** | Quote/unquote | procedural macros |
| **Code generation** | Generate functions | macros |
| **File I/O** | Read/write files | build scripts |
| **Process spawn** | Run external tools | build automation |

**Estimated Coverage**: ~95% of AutoVM features

### Current AutoVM Coverage Analysis

Based on `crates/auto-lang/src/vm/opcode.rs` and `engine.rs`:

| Category | Current Support | CTEE Required | Gap |
|----------|-----------------|---------------|-----|
| **Stack Ops** | ✅ Full | ✅ Needed | None |
| **Arithmetic** | ✅ Full (i32, f32, f64, i64) | ✅ Needed | None |
| **Comparison** | ✅ Full | ✅ Needed | None |
| **Control Flow** | ✅ JMP, JMP_IF_*, CALL, RET | ✅ Needed | None |
| **Locals** | ✅ LOAD_LOCAL, STORE_LOCAL | ✅ Needed | None |
| **Strings** | ✅ LOAD_STR, BUILD_FSTR | ✅ Needed | None |
| **Arrays** | ✅ CREATE_ARRAY, GET/SET_ELEM | ✅ Needed | None |
| **Objects** | ✅ CREATE_OBJ, GET/SET_FIELD | ⚠️ Partial | Type info needed |
| **Closures** | ✅ Plan 071 | ⚠️ Maybe | Capture in comptime |
| **Generics** | ✅ Plan 076/087 | ✅ Needed | Monomorphization |
| **Native calls** | ✅ CALL_NAT | ⚠️ Sandboxed | Limit I/O |
| **Iterators** | ✅ List/Map/Filter | ⚠️ Maybe | For AST traversal |
| **Reflection** | ❌ Not implemented | 🔮 Future | Phase 2+ |

**Conclusion**: AutoVM is ~75% ready for CTEE Tier 1. Missing pieces:
1. **Sandbox mode** (deterministic execution)
2. **Resource limits** (timeout, memory cap)
3. **CTEE coordinator** (orchestration layer)

## Deterministic Execution

### Why Deterministic?

Compile-time execution must be **reproducible**:
- Same source → same output (no randomness)
- Cache invalidation (hash-based)
- Build reproducibility (no external dependencies)

### Determinism Rules

| Operation | Deterministic Mode | Full Mode |
|-----------|-------------------|-----------|
| `Time.now()` | ❌ Error | ✅ Allowed |
| `Random.int()` | ❌ Error | ✅ Allowed |
| `File.read()` | ❌ Error | ✅ Allowed |
| `Process.spawn()` | ❌ Error | ✅ Allowed |
| `Env.get()` | ❌ Error | ✅ Allowed |
| Pure computation | ✅ Allowed | ✅ Allowed |

## Resource Limits

### Default Limits

```rust
impl Default for CTEELimits {
    fn default() -> Self {
        Self {
            max_time_ms: 5000,        // 5 seconds
            max_memory: 100 * 1024 * 1024, // 100 MB
            max_recursion: 256,       // 256 frames
            max_native_calls: 10000,  // 10k calls
        }
    }
}
```

## Error Handling

### CTEE Errors

```rust
/// Compile-Time Execution Engine errors
#[derive(Debug, Error)]
pub enum CTEEError {
    #[error("Compile-time execution timeout after {0}ms")]
    Timeout(u64),

    #[error("Memory limit exceeded: used {used} bytes, limit is {limit} bytes")]
    MemoryLimitExceeded { used: usize, limit: usize },

    #[error("Recursion limit exceeded: {depth} frames, limit is {limit}")]
    RecursionLimitExceeded { depth: u32, limit: u32 },

    #[error("Non-deterministic operation in comptime: {0}")]
    NonDeterministic(String),

    #[error("Const evaluation failed: {0}")]
    ConstEvalFailed(String),

    #[error("Type constraint not satisfied: {0}")]
    ConstraintNotSatisfied(String),
}
```

## File Structure

```
crates/auto-lang/src/comptime/
├── mod.rs              # Public API, CTEE coordinator
├── const_eval.rs       # Const expression evaluation
├── comptime_exec.rs    # Comptime block execution
├── type_constraints.rs # Generic constraint checking
├── limits.rs           # Resource limit enforcement
├── deterministic.rs    # Determinism checking
├── value.rs            # CTEValue type
├── effects.rs          # Side effect tracking
└── tests.rs            # Comprehensive tests
```

## Dependencies

- Plan 094: Hybrid FFI Bridge (for native calls in comptime)
- Plan 081: AutoVM Default Mode (VM already exists)
- Plan 087: Generic Type Support (for type constraints)

## Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Performance regression | Medium | High | Benchmark, optimize hot paths |
| Sandbox escape | Low | Critical | Security audit, defense in depth |
| Infinite loops in comptime | Medium | Medium | Timeout enforcement |
| Memory exhaustion | Medium | Medium | Memory limits, allocation tracking |
| Non-determinism bugs | Low | High | Extensive testing, logging |

## Related Plans

- [Plan 094: Hybrid FFI Bridge](./094-hybrid-ffi-bridge.md) - Native function support
- [Plan 081: AutoVM Default Mode](./081-autovm-default-mode.md) - VM foundation
- [Plan 033: Self-Hosting Compiler](./033-self-hosting-compiler.md) - Ultimate goal
- [Plan 031: Bootstrap Strategy](./031-bootstrap-strategy.md) - Bootstrap roadmap

## References

- [Zig Comptime](https://ziglang.org/documentation/master/#comptime) - Inspiration
- [D CTFE](https://dlang.org/spec/consteval.html) - Compile-time function execution
- [Rust const fn](https://doc.rust-lang.org/reference/const_eval.html) - Const evaluation

---
---

# Part B: Detailed Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use `superpowers:executing-plans` to implement this plan task-by-task.

**Goal:** Implement a compile-time execution system using `#` prefix syntax for conditional compilation, loop unrolling, and constant evaluation.

**Architecture:** Two-stage compilation where Stage 1 (Meta-Eval) executes `#`-marked code using an embedded interpreter, transforming the AST before Stage 2 (Codegen). Uses existing AutoVM in sandboxed mode.

**Tech Stack:** Rust, miette (error reporting), existing AutoVM infrastructure

**Design Doc:** [docs/design/compile-time-execution.md](../design/compile-time-execution.md)

---

## Phase 1: Lexer & Token Support

### Task 1.1: Add Comptime Token Types

**Files:**
- Modify: `crates/auto-lang/src/token.rs:14-160`

**Step 1: Add new token kinds for comptime keywords**

In `crates/auto-lang/src/token.rs`, add after line 74 (`Hash`):

```rust
    // Comptime keywords (compile-time execution)
    HashIf,      // #if
    HashFor,     // #for
    HashIs,      // #is
    HashBrace,   // #{
```

**Step 2: Add Display implementations for new tokens**

In the `impl fmt::Display for Token` block (around line 201), add:

```rust
            TokenKind::HashIf => write!(f, "<#if>"),
            TokenKind::HashFor => write!(f, "<#for>"),
            TokenKind::HashIs => write!(f, "<#is>"),
            TokenKind::HashBrace => write!(f, "<#{>"),
```

**Step 3: Run tests to verify compilation**

Run: `rtk cargo build -p auto-lang`
Expected: Compiles successfully with no errors

**Step 4: Commit**

```bash
rtk git add crates/auto-lang/src/token.rs
rtk git commit -m "feat(token): add comptime token kinds (#if, #for, #is, #{)"
```

---

### Task 1.2: Update Lexer to Recognize Comptime Keywords

**Files:**
- Modify: `crates/auto-lang/src/lexer.rs:660-670`

**Step 1: Write the failing test**

Create `crates/auto-lang/src/lexer_comptime_test.rs`:

```rust
#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use crate::token::TokenKind;

    #[test]
    fn test_hash_if_token() {
        let mut lexer = Lexer::new("#if");
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashIf);
    }

    #[test]
    fn test_hash_for_token() {
        let mut lexer = Lexer::new("#for");
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashFor);
    }

    #[test]
    fn test_hash_is_token() {
        let mut lexer = Lexer::new("#is");
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashIs);
    }

    #[test]
    fn test_hash_brace_token() {
        let mut lexer = Lexer::new("#{");
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::HashBrace);
    }

    #[test]
    fn test_hash_alone_still_works() {
        // #[...] annotation syntax should still work
        let mut lexer = Lexer::new("#[");
        let tok = lexer.next().unwrap();
        assert_eq!(tok.kind, TokenKind::Hash);
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `rtk cargo test -p auto-lang lexer_comptime_test`
Expected: FAIL - tests not found or wrong token kind

**Step 3: Modify lexer to recognize comptime keywords**

In `crates/auto-lang/src/lexer.rs`, find the `#` handling around line 660 and replace:

```rust
                '#' => {
                    // Look ahead to check for comptime keywords
                    if self.peek('i') {
                        let mut iter = self.chars.clone();
                        iter.next(); // skip 'i'
                        if let Some('f') = iter.next() {
                            let next_next = iter.clone().next();
                            if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                self.chars.next(); // skip 'i'
                                self.chars.next(); // skip 'f'
                                return Ok(Token::new(TokenKind::HashIf, self.pos(3), "#if".into()));
                            }
                        }
                        // Check for #is
                        let mut iter_s = self.chars.clone();
                        iter_s.next(); // skip 'i'
                        if let Some('s') = iter_s.next() {
                            let next_next = iter_s.clone().next();
                            if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                self.chars.next(); // skip 'i'
                                self.chars.next(); // skip 's'
                                return Ok(Token::new(TokenKind::HashIs, self.pos(3), "#is".into()));
                            }
                        }
                    } else if self.peek('f') {
                        let mut iter = self.chars.clone();
                        iter.next(); // skip 'f'
                        if let Some('o') = iter.next() {
                            if let Some('r') = iter.next() {
                                let next_next = iter.clone().next();
                                if next_next.map(|c| !c.is_alphanumeric()).unwrap_or(true) {
                                    self.chars.next(); // skip 'f'
                                    self.chars.next(); // skip 'o'
                                    self.chars.next(); // skip 'r'
                                    return Ok(Token::new(TokenKind::HashFor, self.pos(4), "#for".into()));
                                }
                            }
                        }
                    } else if self.peek('{') {
                        self.chars.next(); // skip '{'
                        return Ok(Token::new(TokenKind::HashBrace, self.pos(2), "#{".into()));
                    }
                    // Default: return Hash for #[...] annotations
                    return Ok(self.single(TokenKind::Hash, c));
                }
```

**Step 4: Run tests to verify they pass**

Run: `rtk cargo test -p auto-lang lexer_comptime_test`
Expected: PASS - all 5 tests pass

**Step 5: Commit**

```bash
rtk git add crates/auto-lang/src/lexer.rs crates/auto-lang/src/lexer_comptime_test.rs
rtk git commit -m "feat(lexer): recognize comptime keywords (#if, #for, #is, #{)"
```

---

## Phase 2: AST Nodes for Comptime

### Task 2.1: Create Comptime AST Node Types

**Files:**
- Create: `crates/auto-lang/src/ast/comptime.rs`
- Modify: `crates/auto-lang/src/ast.rs` (add mod and re-export)

**Step 1: Create comptime.rs with AST node definitions**

Create `crates/auto-lang/src/ast/comptime.rs` with the following structures:

```rust
//! Compile-time execution AST nodes
//!
//! Supports: #if, #for, #is, #{ }, #{expr}

use super::{Body, Expr, Name, ToAtom, ToNode, AtomWriter, ToAtomStr};
use auto_val::{AutoStr, Node as AutoNode};
use std::{fmt, io as stdio};

/// Compile-time conditional (`#if`)
#[derive(Debug, Clone)]
pub struct ComptimeIf {
    pub branches: Vec<ComptimeBranch>,
    pub else_: Option<Body>,
}

#[derive(Debug, Clone)]
pub struct ComptimeBranch {
    pub condition: Expr,
    pub body: Body,
}

/// Compile-time loop (`#for`)
#[derive(Debug, Clone)]
pub struct ComptimeFor {
    pub var: Name,
    pub range_start: Expr,
    pub range_end: Expr,
    pub inclusive: bool,
    pub body: Body,
}

/// Compile-time pattern match (`#is`)
#[derive(Debug, Clone)]
pub struct ComptimeIs {
    pub subject: Expr,
    pub cases: Vec<ComptimeCase>,
    pub else_: Option<Body>,
}

#[derive(Debug, Clone)]
pub struct ComptimeCase {
    pub pattern: Expr,
    pub body: Body,
}

/// Compile-time evaluation block (`#{ }`)
#[derive(Debug, Clone)]
pub struct ComptimeBlock {
    pub body: Body,
    pub result: Option<Expr>,
}

/// Compile-time interpolation (`#{expr}`)
#[derive(Debug, Clone)]
pub struct ComptimeInterpolate {
    pub expr: Expr,
}
```

Include full `Display`, `AtomWriter`, `ToNode`, `ToAtom` implementations for each struct.

**Step 2: Add mod and re-export to ast.rs**

In `crates/auto-lang/src/ast.rs`, add after existing mod declarations:

```rust
mod comptime;
pub use comptime::*;
```

**Step 3: Add Comptime variants to Stmt enum**

Add to `Stmt` enum:

```rust
    // Comptime statements
    ComptimeIf(ComptimeIf),
    ComptimeFor(ComptimeFor),
    ComptimeIs(ComptimeIs),
```

**Step 4: Add ComptimeBlock variant to Expr enum**

Add to `Expr` enum:

```rust
    // Comptime expressions
    ComptimeBlock(ComptimeBlock),
    ComptimeInterpolate(ComptimeInterpolate),
```

**Step 5: Build to verify**

Run: `rtk cargo build -p auto-lang`
Expected: Compiles successfully

**Step 6: Commit**

```bash
rtk git add crates/auto-lang/src/ast/comptime.rs crates/auto-lang/src/ast.rs
rtk git commit -m "feat(ast): add comptime AST nodes (ComptimeIf, ComptimeFor, ComptimeIs, ComptimeBlock)"
```

---

## Phase 3: Parser Support

### Task 3.1: Parse `#if` Statement

**Files:**
- Modify: `crates/auto-lang/src/parser.rs`

**Step 1: Add parse_comptime_if method**

```rust
    /// Parse: #if cond { } elif cond { } else { }
    fn parse_comptime_if(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::HashIf)?;

        let mut branches = Vec::new();
        let condition = self.expr()?;
        self.skip_newlines();
        let body = self.parse_body()?;
        branches.push(ComptimeBranch { condition, body });

        // Parse elif branches
        self.skip_newlines();
        while self.is_kind(TokenKind::Ident) && self.cur.text == "elif" {
            self.next();
            let elif_cond = self.expr()?;
            self.skip_newlines();
            let elif_body = self.parse_body()?;
            branches.push(ComptimeBranch { condition: elif_cond, body: elif_body });
            self.skip_newlines();
        }

        // Parse else
        let mut else_body = None;
        if self.is_kind(TokenKind::Else) {
            self.next();
            self.skip_newlines();
            else_body = Some(self.parse_body()?);
        }

        Ok(Stmt::ComptimeIf(ComptimeIf { branches, else_: else_body }))
    }
```

**Step 2: Integrate into parse_stmt**

Add case in `parse_stmt()`:

```rust
            TokenKind::HashIf => self.parse_comptime_if()?,
```

**Step 3: Write and run tests**

```rust
    #[test]
    fn test_parse_comptime_if_simple() {
        let code = r#"#if OS == "windows" { init_win32() }"#;
        let mut parser = Parser::new(code, CompileDest::Interp);
        let ast = parser.parse().unwrap();
        assert!(matches!(&ast.stmts[0], Stmt::ComptimeIf(_)));
    }
```

**Step 4: Commit**

```bash
rtk git commit -am "feat(parser): parse #if comptime conditional"
```

---

### Task 3.2: Parse `#for` Statement

**Step 1: Add parse_comptime_for method**

```rust
    /// Parse: #for var in start..end { }
    fn parse_comptime_for(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::HashFor)?;
        let var = self.expect_ident()?;
        self.expect(TokenKind::In)?;

        let start = self.expr()?;
        let inclusive = if self.is_kind(TokenKind::RangeEq) {
            self.next();
            true
        } else {
            self.expect(TokenKind::Range)?;
            false
        };
        let end = self.expr()?;

        self.skip_newlines();
        let body = self.parse_body()?;

        Ok(Stmt::ComptimeFor(ComptimeFor {
            var: var.into(),
            range_start: start,
            range_end: end,
            inclusive,
            body,
        }))
    }
```

**Step 2: Integrate into parse_stmt**

```rust
            TokenKind::HashFor => self.parse_comptime_for()?,
```

**Step 3: Commit**

```bash
rtk git commit -am "feat(parser): parse #for comptime loop"
```

---

### Task 3.3: Parse `#is` Statement

**Step 1: Add parse_comptime_is method**

```rust
    /// Parse: #is subject { pattern => { } else => { } }
    fn parse_comptime_is(&mut self) -> AutoResult<Stmt> {
        self.expect(TokenKind::HashIs)?;
        let subject = self.expr()?;

        self.skip_newlines();
        self.expect(TokenKind::LBrace)?;

        let mut cases = Vec::new();
        let mut else_body = None;

        self.skip_newlines();
        while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::EOF) {
            if self.is_kind(TokenKind::Else) {
                self.next();
                self.expect(TokenKind::DoubleArrow)?;
                self.skip_newlines();
                else_body = Some(self.parse_body()?);
                break;
            }

            let pattern = self.expr()?;
            self.expect(TokenKind::DoubleArrow)?;
            self.skip_newlines();
            let body = self.parse_body()?;
            cases.push(ComptimeCase { pattern, body });
            self.skip_newlines();
        }

        self.expect(TokenKind::RBrace)?;
        Ok(Stmt::ComptimeIs(ComptimeIs { subject, cases, else_: else_body }))
    }
```

**Step 2: Integrate and commit**

---

### Task 3.4: Parse `#{ }` Block and Interpolation

**Step 1: Add parse_comptime_block_expr for expressions**

In primary expression parsing:

```rust
            TokenKind::HashBrace => {
                self.next(); // skip #{
                // Check if followed by } immediately - that's interpolation
                // Otherwise it's a block
                ...
            }
```

**Step 2: Commit**

```bash
rtk git commit -am "feat(parser): parse #{ } comptime block and interpolation"
```

---

## Phase 4: CTEE Evaluator (Reuse VmInterpreter)

> **IMPORTANT**: We reuse the existing `VmInterpreter` for compile-time code execution.
> This avoids code duplication and ensures all language features are supported.
> The key additions are:
> 1. **Comptime mode flag** - Distinguishes compile-time vs runtime execution
> 2. **Built-in constants** - OS, ARCH, DEBUG, VERSION available at compile time
> 3. **AST transform pass** - Prunes/expands `#if`, `#for`, `#is`, `#{}` constructs

### Task 4.1: Create CTEE Module Structure (Simplified)

**Files:**
- Create: `crates/auto-lang/src/comptime/mod.rs`
- Create: `crates/auto-lang/src/comptime/transformer.rs` (AST transform only)
- Modify: `crates/auto-lang/src/interpreter/vm_interpreter.rs` (add comptime mode)

**Step 1: Create comptime module with transformer**

The transformer uses `VmInterpreter` to evaluate conditions, not a custom evaluator:

```rust
// crates/auto-lang/src/comptime/mod.rs
pub mod transformer;

pub use transformer::*;

// crates/auto-lang/src/comptime/transformer.rs
use crate::ast::{Code, Stmt, HashIf, HashFor, HashIs, HashBrace};
use crate::interpreter::VmInterpreter;
use crate::error::AutoResult;

/// Compile-Time Execution Engine
///
/// Transforms AST by evaluating `#if`, `#for`, `#is`, `#{}` constructs.
/// Uses VmInterpreter for expression evaluation.
pub struct CTEE {
    /// Embedded VM interpreter for expression evaluation
    vm: VmInterpreter,
    /// Built-in compile-time constants (OS, ARCH, DEBUG, etc.)
    builtins: HashMap<String, Value>,
    /// Target platform
    target_os: String,
    target_arch: String,
}

impl CTEE {
    pub fn new() -> Self {
        let mut ctee = Self {
            vm: VmInterpreter::new(),
            builtins: HashMap::new(),
            target_os: "windows".to_string(),
            target_arch: "x64".to_string(),
        };
        ctee.init_builtins();
        ctee
    }

    /// Initialize built-in compile-time constants
    fn init_builtins(&mut self) {
        self.builtins.insert("OS".into(), Value::Str(self.target_os.clone().into()));
        self.builtins.insert("ARCH".into(), Value::Str(self.target_arch.clone().into()));
        self.builtins.insert("DEBUG".into(), Value::Bool(true));
        self.builtins.insert("VERSION".into(), Value::Str("0.1.0".into()));
    }

    /// Transform AST by evaluating all comptime constructs
    pub fn transform(&mut self, code: &mut Code) -> AutoResult<()> {
        let mut new_stmts = Vec::new();
        for stmt in code.stmts.drain(..) {
            let transformed = self.transform_stmt(stmt)?;
            new_stmts.extend(transformed);
        }
        code.stmts = new_stmts;
        Ok(())
    }

    /// Evaluate a compile-time expression using VmInterpreter
    fn eval_expr(&mut self, expr: &Expr) -> AutoResult<Value> {
        // Set built-in constants as globals
        for (name, value) in &self.builtins {
            self.vm.set_global(name, value.clone());
        }

        // Convert expression to code string and run
        let code = format!("{}\n", expr);
        self.vm.run(&code)
    }

    fn transform_stmt(&mut self, stmt: Stmt) -> AutoResult<Vec<Stmt>> {
        match stmt {
            Stmt::HashIf(hash_if) => self.transform_hash_if(hash_if),
            Stmt::HashFor(hash_for) => self.transform_hash_for(hash_for),
            Stmt::HashIs(hash_is) => self.transform_hash_is(hash_is),
            Stmt::HashBrace(hash_brace) => self.transform_hash_brace(hash_brace),
            other => Ok(vec![other]),
        }
    }

    // ... transform methods for #if, #for, #is, #{}
}
```

**Step 2: Add comptime mode to VmInterpreter**

In `crates/auto-lang/src/interpreter/vm_interpreter.rs`:

```rust
pub struct VmInterpreter {
    rt: tokio::runtime::Runtime,
    exports: StdHashMap<String, u32>,
    /// Compile-time mode: disables non-deterministic operations
    comptime_mode: bool,
}

impl VmInterpreter {
    pub fn new() -> Self {
        Self {
            rt: tokio::runtime::Runtime::new().expect("Failed to create tokio runtime"),
            exports: StdHashMap::new(),
            comptime_mode: false,
        }
    }

    /// Enable compile-time mode (disables I/O, random, time)
    pub fn set_comptime_mode(&mut self, enabled: bool) {
        self.comptime_mode = enabled;
    }

    /// Check if in compile-time mode
    pub fn is_comptime_mode(&self) -> bool {
        self.comptime_mode
    }
}
```

**Step 3: Commit**

```bash
rtk git add crates/auto-lang/src/comptime/ crates/auto-lang/src/interpreter/
rtk git commit -m "feat(comptime): create CTEE transformer using VmInterpreter"
```

---

### Task 4.2: Implement AST Transform Pass

**Step 1: Implement transform methods using VmInterpreter**

```rust
impl CTEE {
    /// Transform #if - evaluate condition using VM, keep matching branch
    fn transform_hash_if(&mut self, hash_if: HashIf) -> AutoResult<Vec<Stmt>> {
        // Use VmInterpreter to evaluate condition
        let cond_value = self.eval_expr(&hash_if.cond)?;

        if cond_value.is_truthy() {
            // Keep then branch, recursively transform
            let mut result = Vec::new();
            for stmt in hash_if.then_block.stmts {
                result.extend(self.transform_stmt(stmt)?);
            }
            Ok(result)
        } else if let Some(else_block) = hash_if.else_block {
            // Handle else branch
            match else_block {
                HashIfElse::Block(body) => {
                    let mut result = Vec::new();
                    for stmt in body.stmts {
                        result.extend(self.transform_stmt(stmt)?);
                    }
                    Ok(result)
                }
                HashIfElse::ElseIf(nested_if) => {
                    self.transform_hash_if(*nested_if)
                }
            }
        } else {
            Ok(vec![])
        }
    }

    /// Transform #for - unroll loop at compile time
    fn transform_hash_for(&mut self, hash_for: HashFor) -> AutoResult<Vec<Stmt>> {
        // Evaluate iterable (range or array) using VM
        let iter_value = self.eval_expr(&hash_for.iter)?;

        // Get iteration values
        let values = self.value_to_iter(&iter_value)?;

        // Set loop variable and unroll
        let var_name = hash_for.var.to_string();
        let mut result = Vec::new();

        for value in values {
            // Set loop variable as global in VM
            self.builtins.insert(var_name.clone(), value.clone());

            // Transform body with loop variable set
            for stmt in hash_for.body.stmts.clone() {
                result.extend(self.transform_stmt(stmt)?);
            }
        }

        // Remove loop variable
        self.builtins.remove(&var_name);
        Ok(result)
    }

    /// Convert Value to iterator values
    fn value_to_iter(&self, value: &Value) -> AutoResult<Vec<Value>> {
        match value {
            Value::Array(arr) => Ok(arr.iter().cloned().collect()),
            Value::Int(end) => {
                Ok((0..*end).map(Value::Int).collect())
            }
            _ => Err(AutoError::Generic(
                format!("Cannot iterate over {:?}", value)
            )),
        }
    }

    /// Transform #is - pattern match at compile time
    fn transform_hash_is(&mut self, hash_is: HashIs) -> AutoResult<Vec<Stmt>> {
        let target_value = self.eval_expr(&hash_is.target)?;

        for branch in hash_is.branches {
            match branch {
                HashIsBranch::EqBranch(pattern, body) => {
                    let pattern_value = self.eval_expr(&pattern)?;
                    if target_value == pattern_value {
                        let mut result = Vec::new();
                        for stmt in body.stmts {
                            result.extend(self.transform_stmt(stmt)?);
                        }
                        return Ok(result);
                    }
                }
                HashIsBranch::IfBranch(cond, body) => {
                    let cond_value = self.eval_expr(&cond)?;
                    if cond_value.is_truthy() {
                        let mut result = Vec::new();
                        for stmt in body.stmts {
                            result.extend(self.transform_stmt(stmt)?);
                        }
                        return Ok(result);
                    }
                }
                HashIsBranch::ElseBranch(body) => {
                    let mut result = Vec::new();
                    for stmt in body.stmts {
                        result.extend(self.transform_stmt(stmt)?);
                    }
                    return Ok(result);
                }
            }
        }
        Ok(vec![])
    }

    /// Transform #{} - evaluate and substitute result
    fn transform_hash_brace(&mut self, hash_brace: HashBrace) -> AutoResult<Vec<Stmt>> {
        let value = self.eval_expr(&hash_brace.expr)?;
        let expr = self.value_to_expr(&value);
        Ok(vec![Stmt::Expr(expr)])
    }

    /// Convert Value back to Expr literal
    fn value_to_expr(&self, value: &Value) -> Expr {
        match value {
            Value::Nil => Expr::Nil,
            Value::Int(i) => Expr::I64(*i),
            Value::Bool(b) => Expr::Bool(*b),
            Value::Str(s) => Expr::Str(s.clone()),
            Value::Float(f) => Expr::Double(*f, false),
            // ... other types
            _ => Expr::Nil,
        }
    }
}
```

**Step 2: Add comptime mode checks to VM FFI**

In `crates/auto-lang/src/vm/ffi/`, add checks for non-deterministic operations:

```rust
// In stdlib.rs or similar FFI handler
pub fn shim_time_now(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    if vm.is_comptime_mode() {
        return Err(VMError::RuntimeError(
            "Time.now() is not allowed in compile-time mode".to_string()
        ));
    }
    // ... normal implementation
}
```

**Step 3: Commit**

```bash
rtk git commit -am "feat(comptime): implement AST transform pass using VmInterpreter"
```

---

## Phase 5: Integration

### Task 5.1: Integrate CTEE into Compilation Pipeline

**Files:**
- Modify: `crates/auto-lang/src/lib.rs`

**Step 1: Add comptime transform to run()**

```rust
pub fn run(code: &str) -> AutoResult<String> {
    let mut parser = Parser::new(code, CompileDest::Interp);
    let mut ast = parser.parse()?;

    // NEW: Apply comptime transformations
    let mut ctee = comptime::CTEE::new();
    ctee.transform(&mut ast)?;

    let result = eval::eval(&ast)?;
    Ok(result)
}
```

**Step 2: Write end-to-end tests**

**Step 3: Commit**

```bash
rtk git commit -am "feat: integrate CTEE into compilation pipeline"
```

---

### Task 5.2: Add compile_error() Intrinsic

**Step 1: Handle compile_error in transform**

```rust
    fn compile_error(&self, msg: &str) -> AutoResult<()> {
        Err(SyntaxError::Generic {
            message: format!("compile_error: {}", msg),
            span: SourceSpan::new(0.into(), 0.into()),
        }.into())
    }
```

**Step 2: Commit**

---

## Phase 6: Error Reporting

### Task 6.1: Add Comptime-Specific Error Types

**Files:**
- Modify: `crates/auto-lang/src/error.rs`

**Step 1: Add ComptimeError with miette integration**

```rust
#[derive(Debug, Error)]
#[error("Compile-time error: {message}")]
pub struct ComptimeError {
    pub message: String,
    #[source_code]
    pub source: Option<String>,
    #[label("error occurred here")]
    pub span: SourceSpan,
    pub comptime_stack: Vec<String>,
}

impl Diagnostic for ComptimeError {
    fn code(&self) -> Option<Box<dyn std::fmt::Display>> {
        Some(Box::new("auto_comptime_E0001"))
    }
}
```

**Step 2: Commit**

```bash
rtk git commit -am "feat(error): add ComptimeError with source location"
```

---

## Success Criteria Checklist

### Phase 1 Complete (Lexer & Tokens) ✅
- [x] Lexer recognizes `#if`, `#for`, `#is`, `#{` tokens
- [x] All comptime token tests pass

### Phase 2 Complete (AST Nodes) ✅
- [x] `HashIf`, `HashFor`, `HashIs`, `HashBrace` structs defined
- [x] `ToAtom` and `ToNode` implementations work
- [x] `Stmt` enum has `HashIf`, `HashFor`, `HashIs`, `HashBrace` variants

### Phase 3 Complete (Parser) ✅
- [x] `#if ... else { }` parses correctly
- [x] `#for var in iter { }` parses correctly
- [x] `#is target { pattern => body }` parses correctly
- [x] `#{ expr }` parses as statement

### Phase 4 Complete (CTEE) ✅
- [x] CTEE transformer using VmInterpreter (not custom evaluator)
- [x] Built-in constants (OS, ARCH, DEBUG, VERSION) available at compile time
- [x] `transform()` modifies AST in-place
- [x] `#if` condition evaluation works (via VmInterpreter)
- [x] `#for` loop unrolling works
- [x] `#is` pattern matching works
- [ ] VmInterpreter comptime mode blocks non-deterministic ops (TODO)

### Phase 5 Complete (Integration) ⏳
- [ ] CTEE integrated into `run()` pipeline
- [ ] `compile_error()` intrinsic works

### Phase 6 Complete (Error Reporting) ⏳
- [ ] `ComptimeError` with miette integration
- [ ] Clear error messages for comptime failures

### Self-Hosting Ready
- [ ] All Tier 1 features implemented
- [ ] Auto compiler can compile itself
- [ ] `#if`, `#for`, `#{}` work in stdlib and compiler
