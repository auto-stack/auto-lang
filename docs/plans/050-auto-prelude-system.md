# Plan 050: Auto Prelude System

## Objective

Implement a Rust-inspired prelude system for AutoLang that automatically imports common symbols (especially `May<T>` for error handling) into every module, eliminating the need for repetitive `use` statements while maintaining backwards compatibility.

## Design Decisions (Based on User Feedback)

### Decision 1: Prelude Definition - `.at` File ✅
**Choice**: Use `stdlib/auto/prelude.at` file-based approach

**Rationale**:
- User-editable and discoverable
- Explicit control over what's auto-imported
- Can be extended without recompiling compiler
- Aligns with Auto's philosophy of self-hosting

**Implementation**:
```
stdlib/
└── auto/
    └── prelude.at       # Auto-imported into every module
```

### Decision 2: Project-Level Preludes - YES ✅
**Choice**: Allow users to define custom preludes

**Rationale**:
- Projects can define their own common imports
- Loaded after stdlib prelude (doesn't override, can shadow)
- Useful for domain-specific libraries

**Implementation**:
```
my-project/
├── prelude.at          # Project-specific prelude
├── src/
│   └── main.at
└── tests/
    └── test.at
```

### Decision 3: Backwards Compatibility - Deprecation Warning ✅
**Choice**: Warn about redundant explicit imports

**Rationale**:
- Smooth migration path for existing code
- Users can clean up imports at their own pace
- No breaking changes

**Example Warning**:
```
warning: redundant import
  --> src/main.at:3:5
   |
3  | use auto.may: May
   |     ^^^^^^^^^ help: 'May' is already in the prelude
   |
   = note: `#[deny(redundant_imports)]` to deny by default
```

### Decision 4: Prelude Scope - Every Module ✅
**Choice**: Auto-import into all `.at` files by default

**Rationale**:
- Consistent with Rust's behavior
- Predictable and familiar to Rust developers
- Simple mental model

**Opt-out Mechanism**:
```auto
#[no_prelude]
fn bare_module() {
    // No prelude symbols available here
}
```

## Current State

### Problem 1: Repetitive Imports for May<T>

Every file that uses `?T` syntax needs to import May:
```auto
use auto.may: May

fn divide(a int, b int) ?int {
    if b == 0 {
        May.err(1)
    } else {
        May.val(a / b)
    }
}
```

This is cumbersome because error handling is ubiquitous.

### Problem 2: No Auto-Wrapping for Return Values

Functions with `?T` return types require explicit wrapping:
```auto
fn get_value() ?int {
    May.val(42)  // Should auto-wrap: 42
}
```

### Problem 3: VM Tests Cannot Use .? Operator Without Auto-Wrapping

**Current Issue**: The `?T` syntax sugar (from Plan 049) parses correctly, but functions with `?T` return types don't automatically wrap return values in May tags.

**Example Test Failure** ([crates/auto-lang/src/tests/may_tests.rs](../../crates/auto-lang/src/tests/may_tests.rs)):
```auto
fn get_value() ?int {
    42  // Returns plain int 42, NOT May.val(42)
}

fn main() int {
    let result = get_value()
    result.?  // FAILS: result is plain int, not May<int>
}
```

**Root Cause**: The evaluator's `.?` operator expects a May tag with `is_some` property:
```rust
// From eval.rs:2838
match node.get_prop("is_some") {
    Value::Bool(true) => node.get_prop("value"),
    _ => { /* Early return - causes test to fail */ }
}
```

Since `get_value()` returns plain `42` (not wrapped in May), the `.?` operator triggers an early return.

**Solution**: Implement auto-wrapping (Step 4 of this plan) so that:
```auto
fn get_value() ?int {
    42  // Automatically becomes May.val(42)
}
```

## Proposed Solution

### Phase 1: Prelude Module Structure

**File: stdlib/auto/prelude.at**
```auto
// AutoLang Prelude - Automatically imported into every module
// This file defines symbols that are available without explicit imports

// ============================================================================
// Error Handling (Most Critical - addresses Plan 049)
// ============================================================================

use auto.may: May, nil, val, err

// ============================================================================
// Core Types
// ============================================================================

// Primitive type constructors (for generic type parameters)
use auto.types: str, int, uint, float, double, bool, char

// ============================================================================
// Core Traits (Future - when trait system exists)
// ============================================================================

// use auto.traits: Into, TryFrom, AsRef, FromStr, Display

// ============================================================================
// Collections
// ============================================================================

use auto.list: List

// ============================================================================
// I/O Operations (Commonly Used)
// ============================================================================

use auto.io: say, print, flush

// ============================================================================
// Common Constants
// ============================================================================

const true bool = true
const false bool = false
```

**Key Design Points**:
1. **Explicit re-exports**: Prelude doesn't define symbols, it re-exports them
2. **Modular**: Each section organized by purpose
3. **Minimal**: Only includes truly ubiquitous symbols
4. **Documented**: Comments explain why each symbol is included

### Phase 2: Compiler Integration

**File: crates/auto-lang/src/parser.rs**
```rust
// Around line 400 (in Parser struct)
impl<'a> Parser<'a> {
    pub fn parse_module(&mut self, name: Name, code: &str) -> AutoResult<Module> {
        // Phase 1: Parse and inject prelude
        self.inject_prelude()?;

        // Phase 2: Parse project prelude if exists
        self.inject_project_prelude()?;

        // Phase 3: Parse user code
        let module = self.parse_code(name, code)?;

        // Phase 4: Check for redundant imports
        self.check_redundant_imports(&module)?;

        Ok(module)
    }

    fn inject_prelude(&mut self) -> AutoResult<()> {
        // Load stdlib/auto/prelude.at
        let prelude_path = "stdlib/auto/prelude.at";
        let prelude_code = fs::read_to_string(prelude_path)
            .map_err(|e| SyntaxError::Generic {
                message: format!("Failed to load prelude: {}", e),
                span: SourceSpan::new(0.into(), 0.into()),
            })?;

        // Parse prelude as hidden module
        let prelude_module = self.parse_code(
            Name::from("auto::prelude"),
            &prelude_code,
        )?;

        // Merge prelude symbols into current scope
        self.scope.merge_prelude_symbols(prelude_module);

        Ok(())
    }

    fn inject_project_prelude(&mut self) -> AutoResult<()> {
        // Check for project/prelude.at
        if let Ok(project_prelude) = fs::read_to_string("prelude.at") {
            let module = self.parse_code(
                Name::from("project::prelude"),
                &project_prelude,
            )?;

            // Project prelude can shadow stdlib prelude
            self.scope.merge_prelude_symbols(module);
        }

        Ok(())
    }

    fn check_redundant_imports(&mut self, module: &Module) -> AutoResult<()> {
        for import in &module.imports {
            if self.is_prelude_symbol(&import.symbol) {
                self.emit_warning(Warning::RedundantImport {
                    symbol: import.symbol.clone(),
                    span: import.span,
                    suggestion: format!(
                        "'{}' is already in the prelude",
                        import.symbol
                    ),
                });
            }
        }

        Ok(())
    }

    fn is_prelude_symbol(&self, symbol: &Name) -> bool {
        // Check against known prelude symbols
        matches!(
            symbol.as_ref(),
            "May" | "nil" | "val" | "err" |
            "str" | "int" | "uint" | "float" | "double" | "bool" | "char" |
            "List" | "say" | "print" | "flush" |
            "true" | "false"
        )
    }
}
```

**File: crates/auto-lang/src/ast.rs**
```rust
// Around line 100 (in Module struct)
pub struct Module {
    pub name: Name,
    pub imports: Vec<Import>,
    pub stmts: Vec<Stmt>,
    pub prelude_injected: bool,  // NEW: Track if prelude was injected
}

// In Attribute enum (around line 200)
pub enum Attribute {
    #[serde(rename = "c")]
    C,
    #[serde(rename = "vm")]
    VM,
    #[serde(rename = "pub")]
    Pub,
    #[serde(rename = "no_prelude")]  // NEW: Opt-out of prelude
    NoPrelude,
}
```

### Phase 3: Auto-Wrapping for ?T Return Types

**File: crates/auto-lang/src/infer/expr.rs**
```rust
// Around line 500 (new function)
pub fn auto_wrap_may_return(
    ctx: &mut InferenceContext,
    fn_ret_type: &Type,
    return_expr: &Expr,
    span: SourceSpan,
) -> AutoResult<Expr> {
    match fn_ret_type {
        Type::Tag(tag) if is_may_type(tag) => {
            // Extract inner type T from May<T>
            let inner_type = extract_may_type_param(tag)?;

            // Infer return expression type
            let return_ty = infer_expr(ctx, return_expr)?;

            // Check if return expr matches T (or is already May<T>)
            if types_compatible(ctx, &return_ty, &inner_type) {
                // Auto-wrap: May.val(return_expr)
                Ok(Expr::Call {
                    callee: Box::new(Expr::Member {
                        object: Box::new(Expr::Ident(Name::from("May"))),
                        member: Name::from("val"),
                        span,
                    }),
                    args: vec![return_expr.clone()],
                    span,
                })
            } else if is_may_type_from_type(&return_ty) {
                // Already wrapped, return as-is
                Ok(return_expr.clone())
            } else {
                Err(TypeError::TypeMismatch {
                    expected: inner_type,
                    found: return_ty,
                    span,
                }.into())
            }
        }
        _ => Ok(return_expr.clone()),
    }
}

fn is_may_type(tag: &Tag) -> bool {
    tag.name.as_ref().starts_with("May_")
}

fn extract_may_type_param(tag: &Tag) -> AutoResult<Type> {
    // Find the 'val' field and extract its type
    tag.fields
        .iter()
        .find(|f| f.name.as_ref() == "val")
        .map(|f| f.ty.clone())
        .ok_or_else(|| TypeError::Generic {
            message: "May type missing 'val' field".to_string(),
            span: SourceSpan::new(0.into(), 0.into()),
        }.into())
}
```

**Usage in Type Checker**:
```rust
// In type checking for function returns
fn check_function_return(
    &mut self,
    fn_decl: &FnDecl,
    return_expr: &Expr,
) -> AutoResult<Expr> {
    let fn_ret_type = self.resolve_type(&fn_decl.ret_type)?;

    // Auto-wrap if ?T return type
    let wrapped_expr = auto_wrap_may_return(
        &mut self.ctx,
        &fn_ret_type,
        return_expr,
        return_expr.span(),
    )?;

    Ok(wrapped_expr)
}
```

### Phase 4: Transpiler Integration

**File: crates/auto-lang/src/trans/c.rs**
```rust
// Around line 1500 (in transpile_fn)
fn transpile_fn(&mut self, fn_decl: &FnDecl, sink: &mut String) -> AutoResult<()> {
    // Generate May<T> struct definition if needed
    if let Type::Tag(tag) = &fn_decl.ret_type {
        if is_may_type(tag) {
            // Generate May_int, May_str, etc. as needed
            self.ensure_may_struct_defined(tag, sink)?;
        }
    }

    // Transpile function body (auto-wrapping already done)
    self.transpile_block(&fn_decl.body, sink)?;

    Ok(())
}
```

**Generated C Code Example**:

**Input AutoLang**:
```auto
use auto.may: May  // WARNING: redundant import

fn divide(a int, b int) ?int {
    if b == 0 {
        May.err(1)
    } else {
        a / b  // Auto-wrapped to May.val(a / b)
    }
}

fn main() {
    let result = divide(10, 2)
    is result {
        May.val(v) => say("Result: $v")
        May.err(e) => say("Error: $e")
    }
}
```

**Generated C**:
```c
#include "auto/may.h"

typedef struct {
    int tag;
    union {
        int val;
        int err;
    } u;
} May_int;

May_int divide(int a, int b) {
    May_int _result;
    if (b == 0) {
        _result.tag = MAY_ERR;
        _result.u.err = 1;
    } else {
        _result.tag = MAY_VAL;
        _result.u.val = a / b;  // Auto-wrapped
    }
    return _result;
}

int main() {
    May_int result = divide(10, 2);
    if (result.tag == MAY_VAL) {
        say("Result: %d", result.u.val);
    } else {
        say("Error: %d", result.u.err);
    }
    return 0;
}
```

## Implementation Steps

### Step 1: Create Prelude Module (1-2 hours)
- [ ] Create `stdlib/auto/prelude.at`
- [ ] Add May exports (nil, val, err)
- [ ] Add core type exports
- [ ] Add List, I/O exports
- [ ] Add documentation comments

### Step 2: Parser Integration (2-3 hours)
- [ ] Add `inject_prelude()` to Parser
- [ ] Load `stdlib/auto/prelude.at`
- [ ] Merge prelude symbols into module scope
- [ ] Add `inject_project_prelude()` for custom preludes
- [ ] Handle missing prelude file gracefully

### Step 3: Redundant Import Detection (1-2 hours)
- [ ] Add `check_redundant_imports()` function
- [ ] Maintain list of prelude symbols
- [ ] Emit deprecation warnings
- [ ] Add test for warning emission

### Step 4: Auto-Wrapping Implementation (3-4 hours)
- [ ] Add `auto_wrap_may_return()` to type checker
- [ ] Detect `?T` return types
- [ ] Extract inner type parameter
- [ ] Generate `May.val()` wrapper calls
- [ ] Handle explicit `May.val()` and `May.err()` calls

### Step 5: Transpiler Updates (2-3 hours)
- [ ] Update C transpiler to handle auto-wrapped returns
- [ ] Ensure May<T> structs are generated
- [ ] Update Rust transpiler to use `Some()` directly
- [ ] Test generated code quality

### Step 6: No-Prelude Attribute (1 hour)
- [ ] Add `#[no_prelude]` attribute
- [ ] Modify parser to respect attribute
- [ ] Add tests for no-prelude modules

### Step 7: Testing & Documentation (2-3 hours)
- [ ] Add tests for prelude injection
- [ ] Add tests for auto-wrapping
- [ ] Add tests for redundant import warnings
- [ ] Add tests for project preludes
- [ ] Add tests for `#[no_prelude]`
- [ ] Update CLAUDE.md with prelude documentation
- [ ] Create migration guide for existing code

**Total**: 12-18 hours

## Test Cases

### Test 1: Basic Prelude Injection
```auto
// test_prelude_basic.at
// NO import statement needed!

fn get_value() ?int {
    42  // Should auto-wrap to May.val(42)
}

fn main() {
    let result = get_value()
    say("Got value")
}
```

**Expected**: Compiles without errors, May is available implicitly

### Test 2: Redundant Import Warning
```auto
// test_redundant_import.at
use auto.may: May  // WARNING: redundant

fn get_value() ?int {
    May.val(42)
}
```

**Expected**: Warning emitted, code still compiles

### Test 3: Project Prelude
```auto
// project/prelude.at
use mylib: Helper

// src/main.at
fn main() {
    Helper.do_something()  // Helper available from project prelude
}
```

**Expected**: Helper is available without explicit import

### Test 4: No Prelude Attribute
```auto
#[no_prelude]
fn bare_function() int {
    // May, List, say, etc. NOT available here
    42
}
```

**Expected**: Compilation fails if bare_function uses May without import

### Test 5: Auto-Wrapping with Expressions
```auto
fn divide(a int, b int) ?int {
    if b == 0 {
        May.err(1)  // Explicit error
    } else {
        a / b       // Auto-wrapped to May.val(a / b)
    }
}
```

**Expected**: Both branches work, return type is May_int

## Migration Guide for Existing Code

### Before (Current Code)
```auto
use auto.may: May

fn get_value() ?int {
    May.val(42)
}

fn main() {
    let result = get_value()
    use auto.io: say
    say("Hello")
}
```

### After (With Prelude)
```auto
// No imports needed!

fn get_value() ?int {
    42  // Auto-wrapped
}

fn main() {
    let result = get_value()
    say("Hello")  // say is in prelude
}
```

### Step-by-Step Migration

1. **Run compiler to find redundant imports**:
   ```bash
   auto build your_project.at
   ```

2. **Fix warnings by removing redundant imports**:
   ```bash
   # Before
   use auto.may: May

   # After
   # (remove the line)
   ```

3. **Remove explicit May.val() calls** (optional but cleaner):
   ```bash
   # Before
   fn get_value() ?int {
       May.val(42)
   }

   # After
   fn get_value() ?int {
       42  # Auto-wrapped
   }
   ```

4. **Test everything still works**:
   ```bash
   cargo test -p auto-lang
   ```

## Backwards Compatibility

### Guarantee: No Breaking Changes

1. **Existing imports continue to work**:
   - `use auto.may: May` still valid (just redundant)
   - Explicit `May.val()` calls still work
   - No code breaks, only warnings emitted

2. **Explicit wraps still allowed**:
   ```auto
   fn explicit() ?int {
       May.val(42)  // Still valid, just redundant
   }
   ```

3. **Opt-out available**:
   ```auto
   #[no_prelude]
   module legacy_code {
       use auto.may: May  // Must import explicitly
   }
   ```

## Dependencies

- **Required**: Plan 049 (May Operators to Generic Types) - ✅ COMPLETE
- **Required**: May<T> generic type system - ✅ COMPLETE
- **Required**: Type inference system - ✅ COMPLETE (Phase 1-2)
- **Optional**: Trait system (for future Prelude exports)

## Success Criteria

1. ✅ `stdlib/auto/prelude.at` created with May exports
2. ✅ Prelude auto-injected into every module
3. ✅ Redundant imports emit deprecation warnings
4. ✅ Auto-wrapping works for `?T` return types
5. ✅ Project preludes load after stdlib prelude
6. ✅ `#[no_prelude]` attribute disables prelude
7. ✅ All existing tests pass (backwards compatible)
8. ✅ New prelude tests pass
9. ✅ Documentation updated (CLAUDE.md)
10. ✅ Migration guide published

## Risks & Mitigations

### R1: Prelude Scope Creep

**Risk**: Prelude grows too large, becoming bloated

**Mitigation**:
- Strict criteria for prelude inclusion
- "Observed ubiquity" requirement (must be used in >50% of files)
- Regular prelude audits every 6 months
- Document rationale for each prelude symbol

### R2: Name Conflicts

**Risk**: User code conflicts with prelude symbols

**Mitigation**:
- Project preludes can shadow stdlib prelude
- Explicit imports can still override
- `#[no_prelude]` available for extreme cases
- Clear documentation of prelude symbols

### R3: Compilation Slowdown

**Risk**: Parsing prelude adds to compilation time

**Mitigation**:
- Cache parsed prelude AST
- Parse prelude once per compilation session
- Benchmark prelude loading (target: <10ms overhead)

### R4: Auto-Wrapping Surprises

**Risk**: Auto-wrapping behavior confuses users

**Mitigation**:
- Clear error messages when wrapping fails
- Optional compiler flag to disable auto-wrapping
- Documentation with before/after examples
- Migration guide for existing code

## Future Enhancements

### Post-Plan 050

1. **Prelude Versioning**:
   ```auto
   #[prelude_version("2024")]
   module my_module {
       // Use specific prelude version
   }
   ```

2. **Selective Prelude Imports**:
   ```auto
   #[prelude(may, io)]
   module my_module {
       // Only import may and io from prelude
   }
   ```

3. **Prelude Introspection**:
   ```auto
   use prelude: symbols  // List all prelude symbols
   ```

4. **Trait-Based Prelude** (when traits exist):
   ```auto
   // Prelude exports traits, not concrete types
   use auto.traits: From, Into
   ```

## Timeline Estimate

- **Step 1** (Create prelude): 1-2 hours
- **Step 2** (Parser integration): 2-3 hours
- **Step 3** (Import detection): 1-2 hours
- **Step 4** (Auto-wrapping): 3-4 hours
- **Step 5** (Transpiler updates): 2-3 hours
- **Step 6** (No-prelude attribute): 1 hour
- **Step 7** (Testing & docs): 2-3 hours

**Total**: 12-18 hours (spread over 2-3 days)

## Next Steps

1. ✅ Plan 050 document created
2. ⏸️ Await user approval
3. ⏸️ Create stdlib/auto/prelude.at
4. ⏸️ Implement parser integration
5. ⏸️ Add auto-wrapping to type checker
6. ⏸️ Update transpilers
7. ⏸️ Add comprehensive tests
8. ⏸️ Update documentation

## Status: 📝 PLANNING

**Current Phase**: Design document complete, awaiting approval

**Completed**:
- ✅ Design decisions finalized via user Q&A
- ✅ Implementation plan drafted
- ✅ Test cases defined
- ✅ Migration guide outlined
- ✅ Documented VM test failure (demonstrates need for auto-wrapping)

**Blocked**:
- Awaiting user approval to begin implementation

**Note**: The 17 VM tests in [may_tests.rs](../../crates/auto-lang/src/tests/may_tests.rs) currently fail because they use the `.?` operator without auto-wrapping. These tests will pass once Step 4 (Auto-Wrapping) is implemented.
