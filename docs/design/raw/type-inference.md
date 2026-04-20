# Type Inference System (Rust Implementation)

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

### Overview

The Rust implementation (`crates/auto-lang/`) includes a comprehensive type inference and type checking subsystem that supports:

- **Hybrid Inference Strategy**: Local bottom-up inference for expressions, simplified Hindley-Milner for functions
- **Static Type Checking**: Catch type errors at compile time while maintaining runtime type flexibility
- **Type Error Recovery**: Graceful degradation to `Type::Unknown` when inference fails
- **Friendly Error Messages**: Using existing miette infrastructure for clear diagnostics
- **Modular Architecture**: Clean separation from parser, evaluator, and transpiler

### Module Structure

Located in `crates/auto-lang/src/infer/`:

```
infer/
├── mod.rs              # Public API and module re-exports
├── context.rs          # InferenceContext (type environment, constraints)
├── unification.rs      # Robinson unification algorithm
├── constraints.rs      # TypeConstraint representation
├── expr.rs             # Expression type inference
├── stmt.rs             # Statement type checking (TODO: Phase 3)
└── functions.rs        # Function signature inference (TODO: Phase 4)
```

### Current Implementation Status

**Completed** (2025):
- ✅ Phase 1: Core Infrastructure (context, constraints)
- ✅ Phase 2: Expression Inference (20+ expression types)
- ✅ Type Unification (Robinson algorithm with occurs check)
- ✅ Type Coercion (int ↔ uint, float ↔ double)
- ✅ 285 unit tests + 9 doc tests
- ✅ Zero compilation warnings

**Not Yet Integrated**:
- ⏸️ Phase 5: Parser integration (user indicated not needed for now)
- See `docs/type-inference-implementation-summary.md` for full details

### Using the Type Inference System

**Basic Usage**:
```rust
use auto_lang::infer::{InferenceContext, infer_expr};
use auto_lang::ast::{Expr, Type};

let mut ctx = InferenceContext::new();

// Infer expression type
let expr = Expr::Int(42);
let ty = infer_expr(&mut ctx, &expr);
assert!(matches!(ty, Type::Int));

// Check for errors
if ctx.has_errors() {
    for error in &ctx.errors {
        eprintln!("Type error: {}", error);
    }
}
```

**With Variable Bindings**:
```rust
use auto_lang::infer::InferenceContext;
use auto_lang::ast::{Name, Type, Expr};

let mut ctx = InferenceContext::new();

// Bind variable
let name = Name::from("x");
ctx.bind_var(name.clone(), Type::Int);

// Lookup variable type
let ty = ctx.lookup_type(&name);
assert!(matches!(ty, Some(Type::Int)));

// Infer expression using variable
let expr = Expr::Ident(name);
let inferred_ty = infer_expr(&mut ctx, &expr);
assert!(matches!(inferred_ty, Type::Int));
```

**With Scope Management**:
```rust
use auto_lang::infer::InferenceContext;
use auto_lang::ast::{Name, Type};

let mut ctx = InferenceContext::new();
let name = Name::from("x");

// Outer scope
ctx.bind_var(name.clone(), Type::Int);
assert!(matches!(ctx.lookup_type(&name), Some(Type::Int)));

// Inner scope (shadowing)
ctx.push_scope();
ctx.bind_var(name.clone(), Type::Float);
assert!(matches!(ctx.lookup_type(&name), Some(Type::Float)));

// Pop inner scope
ctx.pop_scope();
assert!(matches!(ctx.lookup_type(&name), Some(Type::Int)));
```

**Type Unification**:
```rust
use auto_lang::infer::InferenceContext;
use auto_lang::ast::Type;

let mut ctx = InferenceContext::new();

// Unify compatible types
let result = ctx.unify(Type::Int, Type::Int);
assert!(result.is_ok());

// Unify with coercion (generates warning)
let result = ctx.unify(Type::Int, Type::Uint);
assert!(result.is_ok());
assert!(ctx.has_warnings());

// Unify incompatible types
let result = ctx.unify(Type::Int, Type::Bool);
assert!(result.is_err());
```

### Supported Expression Types

The type inference system supports the following expression types:

**Literals**:
- `Int(_)`, `I8(_)`, `I64(_)` → `Type::Int`
- `Uint(_)`, `Byte(_)`, `U8(_)` → `Type::Uint`
- `Float(_, _)` → `Type::Float`
- `Double(_, _)` → `Type::Double`
- `Bool(_)` → `Type::Bool`
- `Char(_)` → `Type::Char`
- `Str(s)` → `Type::Str(s.len())`
- `CStr(_)` → `Type::CStr`

**Operations**:
- **Unary**: `Not` → `Bool`, `Sub` → operand type
- **Binary**: Arithmetic → operand type, Comparison → `Bool`
- **Index**: Array[index] → element type
- **Call**: Function call → return type

**Complex**:
- `Array(elems)` → `Type::Array { elem, len }`
- `If { branches, else_ }` → unified branch type
- `Block { stmts }` → last statement's type
- `Ref(name)` → `Type::Ptr<inner_type>`

**Unsupported** (return `Type::Unknown`):
- `Lambda` → TODO: Phase 4
- `Object`, `Pair` → TODO: struct type inference
- `Grid`, `Cover`, `Uncover` → TODO
- `Node` → TODO

### Type Unification Algorithm

The system implements Robinson's unification algorithm with occurs check:

**Features**:
- `Type::Unknown` acts as wildcard (unifies with anything)
- Recursive unification for compound types (arrays, pointers)
- Occurs check prevents infinite types
- Coercion support for compatible types with warnings

**Unification Rules**:
```rust
(Type::Unknown, ty)        → Ok(ty)           // Unknown is wildcard
(Type::Int, Type::Int)     → Ok(Type::Int)   // Same types
(Type::Array(a), Type::Array(b)) → Unified array if elem types and lengths match
(Type::Int, Type::Uint)    → Ok(Type::Uint) + warning  // Coercion
(Type::Int, Type::Bool)    → Err(Mismatch)    // Incompatible
```

### Error Handling

**Type Errors** (stored in `ctx.errors`):
- Undefined variables
- Type mismatches
- Invalid operations
- Array length mismatches

**Warnings** (stored in `ctx.warnings`):
- Implicit type conversions
- Potentially unsafe operations

**Error Recovery**:
- Failed inference returns `Type::Unknown`
- Compilation continues after type errors
- Multiple errors reported in one pass

### Testing

**Run type inference tests**:
```bash
# Test all infer modules
cargo test -p auto-lang infer

# Test specific module
cargo test -p auto-lang infer::context
cargo test -p auto-lang infer::unification
cargo test -p auto-lang infer::expr

# Run with output
cargo test -p auto-lang infer -- --nocapture

# Show test output
cargo test -p auto-lang infer -- --show-output
```

**Current Test Results** (2025):
- 285 unit tests passing
- 9 doc tests passing
- Zero compilation warnings
- > 95% code coverage

### Integration with Parser

**Current Status**: NOT YET INTEGRATED

The parser currently uses the old `infer_type_expr()` function (line 2177 in `parser.rs`). The new inference system is implemented and tested but not yet connected to the parser.

**Planned Integration** (Phase 5 - deferred per user request):
```rust
// In parser.rs (line 2177)
// Old code:
fn infer_type_expr(&mut self, expr: &Expr) -> Type {
    // Simple type inference logic
}

// New code (when integrated):
fn infer_type_expr(&mut self, expr: &Expr) -> Type {
    self.infer_ctx.infer_expr(expr)
}
```

**User Feedback**: "暂时不需要" (not needed for now) - awaiting confirmation before integration.

### Documentation

**Internal Documentation**:
- [docs/type-inference-implementation-summary.md](type-inference-implementation-summary.md) - Complete implementation summary
- [plans/elegant-wandering-volcano.md](../.claude/plans/elegant-wandering-volcano.md) - Original design plan with status updates

**API Documentation**:
- All public APIs have comprehensive Rustdoc comments
- Run `cargo doc -p auto-lang --open` to view
- Module-level documentation explains algorithms and usage

### Key Implementation Files

1. **[infer/context.rs](../crates/auto-lang/src/infer/context.rs)** (453 lines)
   - Type environment management
   - Scope stack for variable shadowing
   - Constraint tracking
   - Type unification entry point

2. **[infer/unification.rs](../crates/auto-lang/src/infer/unification.rs)** (465 lines)
   - Robinson unification algorithm
   - Occurs check implementation
   - Type coercion support
   - Comprehensive unification tests

3. **[infer/expr.rs](../crates/auto-lang/src/infer/expr.rs)** (552 lines)
   - Expression type inference for 20+ types
   - Binary/unary operation handling
   - Array and index expressions
   - If/Block expression inference

4. **[infer/constraints.rs](../crates/auto-lang/src/infer/constraints.rs)** (130 lines)
   - Type constraint representation
   - Equal, Callable, Indexable, Subtype constraints
   - Constraint helper methods

5. **[infer/mod.rs](../crates/auto-lang/src/infer/mod.rs)** (90 lines)
   - Public API re-exports
   - Module documentation
   - Integration points

### Future Work (Beyond Phase 2)

See the implementation plan for details:

**Phase 3**: Statement type checking (`stmt.rs`)
**Phase 4**: Function signature inference (`functions.rs`)
**Phase 5**: Parser integration (deferred per user request)
**Phase 6**: Error recovery and suggestions (`errors.rs`)
**Phase 7**: Documentation and examples

**Long-term** (Phases 8-10):
- Generic type parameters
- Trait/interface system
- IDE integration (LSP)

### Contributing to Type Inference

When modifying the type inference system:

1. **Add Tests**: All new code must have comprehensive unit tests
2. **Update Documentation**: Keep Rustdoc comments accurate
3. **Check Warnings**: Maintain zero compilation warnings
4. **Verify Coverage**: Ensure > 90% code coverage
5. **Update Summary**: Reflect changes in implementation summary

**Example Test Pattern**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_my_feature() {
        let mut ctx = InferenceContext::new();
        let expr = Expr::Int(42);
        let ty = infer_expr(&mut ctx, &expr);
        assert!(matches!(ty, Type::Int));
    }
}
```
