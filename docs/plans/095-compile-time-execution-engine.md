# Plan 095: Compile-Time Execution Engine (CTEE)

> **Status**: 📋 Proposed
> **Priority**: Critical (blocks self-hosting)
> **Dependencies**: Plan 094 (Hybrid FFI Bridge), Plan 081 (AutoVM Default Mode)
> **Design Doc**: [docs/design/compile-time-execution.md](../design/compile-time-execution.md) (Finalized)

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

## Implementation Phases

> **Aligned with**: [docs/design/compile-time-execution.md](../design/compile-time-execution.md) Phase 1-5

### Phase 1: Parser Support (Week 1-2)

**Goal**: Extend parser to recognize `#` prefix syntax

**From Design Doc**:
> Modify Parser, recognize `#` prefix. New AST node types: `ComptimeIfStmt`, `ComptimeForStmt`, `ComptimeBlockExpr` (`#{}`).

**New AST Nodes**:
```rust
// crates/auto-lang/src/ast.rs

/// Compile-time conditional (`#if`)
pub struct ComptimeIfStmt {
    pub condition: Expr,
    pub then_block: Vec<Stmt>,
    pub elif_branches: Vec<(Expr, Vec<Stmt>)>,
    pub else_block: Option<Vec<Stmt>>,
}

/// Compile-time loop unrolling (`#for`)
pub struct ComptimeForStmt {
    pub var: Name,
    pub range: Expr,  // start..end or start..=end
    pub body: Vec<Stmt>,
}

/// Compile-time pattern matching (`#is`)
pub struct ComptimeIsStmt {
    pub subject: Expr,
    pub cases: Vec<ComptimeCase>,
}

/// Compile-time evaluation block (`#{ ... }`)
pub struct ComptimeBlockExpr {
    pub body: Vec<Stmt>,
    pub result: Option<Expr>,
}

/// Compile-time interpolation (`#{expr}`)
pub struct ComptimeInterpolation {
    pub expr: Expr,
}
```

**Lexer Changes**:
```rust
// crates/auto-lang/src/token.rs

pub enum TokenKind {
    // ... existing tokens ...

    // Comptime prefix tokens
    HashIf,      // #if
    HashFor,     // #for
    HashIs,      // #is
    HashBrace,   // #{
}
```

**Deliverables**:
- [ ] Lexer recognizes `#if`, `#for`, `#is`, `#{` as tokens
- [ ] Parser builds `ComptimeIfStmt`, `ComptimeForStmt`, `ComptimeIsStmt`, `ComptimeBlockExpr`
- [ ] `#{expr}` parsed as `ComptimeInterpolation` within expressions
- [ ] AST nodes integrate with existing infrastructure

### Phase 2: Meta-Evaluator (Week 2-4)

**Goal**: Implement the compile-time interpreter using embedded AutoVM

**From Design Doc**:
> Implement a lightweight Tree-Walk Interpreter or Bytecode VM. Must simulate target platform data widths.

**CTEE Infrastructure**:
```rust
// crates/auto-lang/src/comptime/mod.rs

/// Compile-Time Execution Engine coordinator
pub struct CTEE {
    /// Embedded AutoVM (sandboxed)
    vm: AutoVM,

    /// Compile-time symbol table
    symbols: HashMap<String, CTEValue>,

    /// Resource limits
    limits: CTEELimits,

    /// Execution mode (deterministic vs full)
    mode: CTEEMode,

    /// Target platform configuration
    target: TargetInfo,
}

/// Target platform information for cross-compilation
pub struct TargetInfo {
    pub os: String,        // "windows", "linux", "macos"
    pub arch: String,      // "x64", "arm", "arm64"
    pub pointer_width: u8, // 32 or 64
}

/// Resource limits for compile-time execution
pub struct CTEELimits {
    pub max_time_ms: u64,
    pub max_memory: usize,
    pub max_recursion: u32,
    pub max_native_calls: u64,
}

/// Execution mode
pub enum CTEEMode {
    /// Deterministic: No I/O, no randomness, reproducible
    Deterministic,
    /// Full: Allow I/O and side effects (for build scripts)
    Full,
}

/// Compile-time evaluated value
pub enum CTEValue {
    Int(i64),
    Uint(u64),
    Float(f64),
    String(String),
    Bool(bool),
    Array(Vec<CTEValue>),
    Type(Type),  // Type values for computed types
}
```

**Deliverables**:
- [ ] `CTEE` struct with sandbox configuration
- [ ] `CTEELimits` enforcement (timeout, memory, recursion)
- [ ] Deterministic mode switch (blocks I/O, randomness)
- [ ] Target platform simulation (pointer width, etc.)
- [ ] Basic error handling with compile-time stack traces

### Phase 3: Transform Pass (Week 4-5)

**Goal**: Implement AST transformation (pruning, expansion, evaluation)

**From Design Doc**:
> Implement an AST Visitor.
> - `ComptimeIf`: Replace with Then-Block or Else-Block content
> - `ComptimeFor`: Copy Body N times and concatenate
> - `#{ expr }`: Evaluate and replace with `LiteralNode`

**Transform Implementation**:
```rust
// crates/auto-lang/src/comptime/transform.rs

impl CTEE {
    /// Transform AST by evaluating comptime constructs
    pub fn transform(&mut self, ast: &mut AST) -> AutoResult<()> {
        self.visit_ast(ast)
    }

    /// Visit and transform `#if` statement
    fn visit_comptime_if(&mut self, stmt: &mut ComptimeIfStmt) -> AutoResult<Option<Vec<Stmt>>> {
        // Evaluate condition in comptime mode
        let cond = self.eval_expr(&stmt.condition)?;

        match cond {
            CTEValue::Bool(true) => Ok(Some(stmt.then_block.clone())),
            CTEValue::Bool(false) => {
                // Check elif branches
                for (elif_cond, elif_block) in &stmt.elif_branches {
                    if self.eval_expr(elif_cond)? == CTEValue::Bool(true) {
                        return Ok(Some(elif_block.clone()));
                    }
                }
                // Fall through to else
                Ok(stmt.else_block.clone())
            }
            _ => Err(CTEEError::TypeError("Comptime condition must be bool")),
        }
    }

    /// Visit and transform `#for` statement
    fn visit_comptime_for(&mut self, stmt: &mut ComptimeForStmt) -> AutoResult<Vec<Stmt>> {
        let range = self.eval_range(&stmt.range)?;
        let mut expanded = Vec::new();

        for i in range {
            // Bind loop variable
            self.symbols.insert(stmt.var.clone(), CTEValue::Int(i));

            // Deep copy body and substitute #{var}
            for body_stmt in &stmt.body {
                let mut copy = body_stmt.clone();
                self.substitute_interpolation(&mut copy)?;
                expanded.push(copy);
            }
        }

        Ok(expanded)
    }

    /// Visit and transform `#{ ... }` block
    fn visit_comptime_block(&mut self, expr: &mut ComptimeBlockExpr) -> AutoResult<Expr> {
        // Execute all statements
        for stmt in &expr.body {
            self.exec_stmt(stmt)?;
        }

        // Evaluate result expression
        let value = match &expr.result {
            Some(result) => self.eval_expr(result)?,
            None => CTEValue::Void,
        };

        // Convert to literal expression
        Ok(self.value_to_literal(value))
    }

    /// Substitute `#{var}` with literal value
    fn substitute_interpolation(&mut self, node: &mut impl ASTNode) -> AutoResult<()> {
        // Walk AST and replace ComptimeInterpolation with Literal
    }
}
```

**Deliverables**:
- [ ] `#if`/`#elif`/`#else` AST pruning (only matching branch emitted)
- [ ] `#for` loop unrolling with `#{var}` interpolation
- [ ] `#{ ... }` block evaluation and literal conversion
- [ ] Integration with main compilation pipeline

### Phase 4: Stdlib & Reflection (Week 5-6)

**Goal**: Provide `std.meta` library for compile-time introspection

**From Design Doc**:
> Provide `std.meta` library with `os`, `arch`, `compiler_version` and type reflection API.

**Built-in Comptime Constants**:
```rust
// crates/auto-lang/src/comptime/builtins.rs

impl CTEE {
    pub fn init_builtins(&mut self) {
        // Target information
        self.symbols.insert("OS", CTEValue::String(self.target.os.clone()));
        self.symbols.insert("ARCH", CTEValue::String(self.target.arch.clone()));
        self.symbols.insert("POINTER_WIDTH", CTEValue::Int(self.target.pointer_width as i64));

        // Compiler information
        self.symbols.insert("COMPILER_VERSION", CTEValue::String(env!("CARGO_PKG_VERSION").to_string()));
        self.symbols.insert("AUTO_VERSION_MAJOR", CTEValue::Int(0));
        self.symbols.insert("AUTO_VERSION_MINOR", CTEValue::Int(10));
        self.symbols.insert("AUTO_VERSION_PATCH", CTEValue::Int(0));
    }
}
```

**Reflection API** (future):
```auto
// std/meta.at
type TypeInfo {
    name str
    fields []FieldInfo
    methods []MethodInfo
}

type FieldInfo {
    name str
    type Type
    offset int
}

// Reflection functions (comptime only)
fn type_of(val) Type
fn fields_of(t Type) []FieldInfo
fn has_field(t Type, name str) bool
```

**Deliverables**:
- [ ] Built-in constants: `OS`, `ARCH`, `POINTER_WIDTH`, `COMPILER_VERSION`
- [ ] `compile_error(msg)` intrinsic (halts compilation with error)
- [ ] (Future) Type reflection API

### Phase 5: Diagnostics (Week 6-7)

**Goal**: Distinguish compile-time errors from runtime errors

**From Design Doc**:
> Distinguish "compile-time execution error" and "code generation error".
> When `#{}` panics, report source location and compile-time stack trace.

**Error Reporting**:
```rust
/// Compile-time error with source location
pub struct ComptimeError {
    pub message: String,
    pub location: SourceSpan,
    pub comptime_stack: Vec<ComptimeFrame>,
}

pub struct ComptimeFrame {
    pub function: String,
    pub location: SourceSpan,
    pub locals: HashMap<String, CTEValue>,
}

impl Diagnostic for ComptimeError {
    fn code(&self) -> Option<Box<dyn std::fmt::Display>> {
        Some(Box::new("auto_comptime_E0001"))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        // Show both comptime location and original source
    }
}
```

**Deliverables**:
- [ ] `ComptimeError` with miette integration
- [ ] Compile-time stack traces
- [ ] Clear distinction: "Error in comptime block" vs "Error in generated code"
- [ ] `compile_error(msg)` intrinsic for user-triggered errors

### Phase 6: Integration with Transpilers (Week 7-8)

**Goal**: Embed CTEE in a2r and a2c transpilers

```rust
// crates/auto-lang/src/trans/rust.rs

pub struct RustTranspiler {
    /// Compile-Time Execution Engine
    ctee: CTEE,

    /// Generated Rust code
    output: String,
}

impl RustTranspiler {
    pub fn transpile(&mut self, ast: &mut AST) -> AutoResult<String> {
        // Phase 1: Transform AST (evaluate comptime constructs)
        self.ctee.transform(ast)?;

        // Phase 2: Type check transformed AST
        self.type_check(ast)?;

        // Phase 3: Generate Rust code
        self.codegen(ast)?;

        Ok(self.output.clone())
    }
}
```

**Deliverables**:
- [ ] CTEE embedded in a2r
- [ ] CTEE embedded in a2c
- [ ] Const value substitution in output
- [ ] Full end-to-end tests

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

### Implementing Determinism

```rust
impl AutoVM {
    /// Execute in sandboxed/deterministic mode
    pub fn execute_sandboxed(&mut self, bytecode: &[u8], limits: &CTEELimits) -> AutoResult<VMValue> {
        // Enable determinism checks
        self.deterministic = true;

        // Set resource limits
        self.limits = limits.clone();

        // Execute with monitoring
        let start = std::time::Instant::now();
        let result = self.execute_with_monitoring(bytecode, |vm| {
            // Check time limit
            if start.elapsed().as_millis() as u64 > limits.max_time_ms {
                return Err(VMError::Timeout);
            }

            // Check memory limit
            if vm.memory_usage() > limits.max_memory {
                return Err(VMError::MemoryLimitExceeded);
            }

            // Check recursion limit
            if vm.call_stack_depth() > limits.max_recursion {
                return Err(VMError::RecursionLimitExceeded);
            }

            Ok(())
        })?;

        Ok(result)
    }

    /// Native call with determinism check
    fn call_native(&mut self, id: u16) -> AutoResult<()> {
        if self.deterministic {
            // Check if native is allowed in deterministic mode
            if !self.is_deterministic_native(id) {
                return Err(VMError::NonDeterministicInComptime);
            }
        }

        // Call native
        self.natives.call(id, self)
    }

    /// Check if native is allowed in deterministic mode
    fn is_deterministic_native(&self, id: u16) -> bool {
        match id {
            // Allowed: pure operations
            NATIVE_PRINT_I32 | NATIVE_PRINT_STR => true,

            // Disallowed: I/O, randomness, external state
            NATIVE_FILE_READ | NATIVE_FILE_WRITE |
            NATIVE_TIME_NOW | NATIVE_RANDOM_INT |
            NATIVE_ENV_GET | NATIVE_PROCESS_SPAWN => false,

            _ => true, // Default: allow
        }
    }
}
```

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

### Configurable Limits

```auto
// In autoconfig.at:
comptime {
    limits {
        time_ms = 10000      // 10 seconds
        memory_mb = 200      // 200 MB
        recursion = 512      // 512 frames
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

### Error Recovery

1. **Timeout/Memory**: Abort comptime, emit error
2. **Non-determinism**: Suggest using `comptime!` (full mode)
3. **Type error**: Report with source location
4. **Constraint failure**: Report which constraint failed

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

## Success Criteria

### Phase 1 Complete (Parser Support)
- [ ] Lexer recognizes `#if`, `#for`, `#is`, `#{` tokens
- [ ] Parser builds `ComptimeIfStmt`, `ComptimeForStmt`, `ComptimeIsStmt`, `ComptimeBlockExpr`
- [ ] `#{expr}` parsed as interpolation within expressions

### Phase 2 Complete (Meta-Evaluator)
- [ ] CTEE coordinator implemented
- [ ] Sandbox infrastructure working
- [ ] Resource limits enforced (timeout, memory, recursion)
- [ ] Deterministic mode blocks I/O and randomness

### Phase 3 Complete (Transform Pass)
- [ ] `#if`/`#elif`/`#else` AST pruning works
- [ ] `#for` loop unrolling with `#{var}` interpolation
- [ ] `#{ ... }` block evaluation and literal conversion
- [ ] Integration with compilation pipeline

### Phase 4 Complete (Stdlib & Reflection)
- [ ] Built-in constants: `OS`, `ARCH`, `POINTER_WIDTH`, `COMPILER_VERSION`
- [ ] `compile_error(msg)` intrinsic works
- [ ] (Future) Type reflection API available

### Phase 5 Complete (Diagnostics)
- [ ] `ComptimeError` with miette integration
- [ ] Compile-time stack traces
- [ ] Clear distinction between comptime and generated code errors

### Phase 6 Complete (Integration)
- [ ] CTEE embedded in a2r
- [ ] CTEE embedded in a2c
- [ ] Full compilation pipeline works end-to-end

### Self-Hosting Ready
- [ ] All Tier 1 features implemented
- [ ] Auto compiler can compile itself
- [ ] `#if`, `#for`, `#{}` work in stdlib and compiler

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

## Design Documents

- [Compile-Time Execution Design](../design/compile-time-execution.md) - **Official syntax specification** (`#if`, `#for`, `#is`, `#{}`)

## References

- [Zig Comptime](https://ziglang.org/documentation/master/#comptime) - Inspiration
- [D CTFE](https://dlang.org/spec/consteval.html) - Compile-time function execution
- [Rust const fn](https://doc.rust-lang.org/reference/const_eval.html) - Const evaluation
