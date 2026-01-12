# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AutoLang is a multi-purpose programming language designed for automation with one canonical implementation:
- **Rust implementation** (`crates/`): Primary compiler with full features including transpilation to C and Rust

The AutoLang compiler supports multiple execution modes:
- **Evaluator**: Interprets AutoLang code directly (REPL, script execution)
- **C Transpiler** (a2c): Transpiles AutoLang to C for embedded systems
- **Rust Transpiler** (a2r): Transpiles AutoLang to Rust for native applications

The self-hosted version (`auto/`) represents a future bootstrap effort to implement the AutoLang compiler in its own language.

## Build Commands

### Rust Implementation (Primary Compiler)

```bash
# From project root
cargo build --release        # Build all crates
cargo run --release          # Run REPL
cargo test                   # Run all tests
cargo test -p auto-lang      # Test specific crate
cargo test -p auto-lang -- trans  # Run transpiler tests
```

### Self-Hosted Implementation (auto/)

```bash
# From project root (future work - not yet implemented)
auto auto/auto.at            # Compile the Auto compiler with itself
./auto                       # Run the compiled Auto compiler
```

**Status**: Early stage - this is the beginning of the self-hosting effort. The compiler is not yet feature-complete.

## Architecture Overview

### Compilation Pipeline

The AutoLang Rust compiler supports three execution modes:

```
Source Code (.at files)
    ↓
Lexer (lexer.rs) → Tokens
    ↓
Parser (parser.rs) → AST (ast.rs)
    ↓
├─→ Evaluator (eval.rs) → Value (REPL/execution)
├─→ C Transpiler (trans/c.rs) → C code
└─→ Rust Transpiler (trans/rust.rs) → Rust code
```

### Core Components (Rust Implementation)

#### 1. **Lexer** (`crates/auto-lang/src/lexer.rs`)
- Tokenizes AutoLang source code
- Handles f-strings with `$variable` and `${expression}` syntax
- Token types defined in `token.rs`

#### 2. **Parser** (`crates/auto-lang/src/parser.rs`)
- Recursive descent parser consuming lexer tokens
- Builds AST nodes defined in `ast.rs`
- Handles expression precedence and control flow
- Uses `AutoStr` for string memory management

#### 3. **AST** (`crates/auto-lang/src/ast.rs`)
- Unified representation for expressions and statements
- Expression types: `int`, `ident`, `binary`, `unary`, `if`, `array`, `call`, `index`, etc.
- Statement types: `expr`, `store`, `for`, `while`, `break`, `ret`, `use`, etc.

#### 4. **Evaluator** (`crates/auto-lang/src/eval.rs`)
- Interprets AST nodes to produce `Value` results
- Supports multiple evaluation modes (SCRIPT, CONFIG, TEMPLATE)
- Uses `Universe` for variable scoping

#### 5. **Value System** (`crates/auto-val/src/`)
- Dynamic typing with runtime type tags
- Types: `int`, `uint`, `float`, `bool`, `str`, `array`, `object`, `nil`, `func`, `native`
- Node-based data structures for complex values

#### 6. **Transpilers** (`crates/auto-lang/src/trans/`)
- **C Transpiler** (`c.rs`): Transpiles AutoLang to C for embedded systems
- **Rust Transpiler** (`rust.rs`): Transpiles AutoLang to Rust for native apps

### Test Infrastructure

#### a2c (Auto-to-C) Tests
Located in `crates/auto-lang/test/a2c/`:
- Test cases organized by number (e.g., `000_hello/`, `021_type_error/`)
- Each test has `.at` source file and `.expected.c`/`.expected.h` output files
- Run with: `cargo test -p auto-lang -- trans`

#### a2r (Auto-to-Rust) Tests
Located in `crates/auto-lang/test/a2r/`:
- Test cases organized by number (e.g., `000_hello/`, `029_composition/`)
- Each test has `.at` source file and `.expected.rs` output file
- Run with: `cargo test -p auto-lang -- trans`

## Language Features

### Storage Types
- `let` - Immutable binding
- `mut` - Mutable binding with type inference
- `const` - Global constant
- `var` - Dynamic type (script mode only)

### Control Flow
- `if/else if/else` - Conditional branching
- `for x in start..end` - Range loops
- `loop` - Infinite loops with `break`
- `is` - Pattern matching

### Key Syntax
- **F-strings**: `f"hello $name"` or `f"result: ${1 + 2}"`
- **Ranges**: `0..10` (exclusive) or `0..=10` (inclusive)
- **Arrays**: `[1, 2, 3]` with indexing `arr[0]`
- **Objects**: `{key: value, ...}` with field access `obj.key`
- **Functions**: `fn add(a int, b int) int { a + b }`
- **Imports**: `use math::add` or `use c <stdio.h>`

## Implementation Strategy

### Primary Implementation: Rust (`crates/`)

The Rust implementation in `crates/` is the canonical AutoLang compiler with:
- Full language feature support
- Three execution modes (evaluator, C transpiler, Rust transpiler)
- Comprehensive error reporting with miette
- Type inference and type checking (see [Type Inference System](#type-inference-system-rust-implementation) below)

### Self-Hosting Strategy (Future)

The self-hosted compiler represents a future bootstrap effort:
1. Rust compiler (`crates/`) → implements full language
2. Auto compiler (`auto/`) → written in AutoLang, compiled by itself

This will create a self-sustaining ecosystem where AutoLang can compile itself.

## Data Structures (Rust Implementation)

### Node and NodeBody

The `Node` and `NodeBody` structures (in `crates/auto-val/src/node.rs`) use `IndexMap` for efficient, ordered storage of properties and child nodes.

**Key Implementation Details**:

- **IndexMap**: Uses `indexmap::IndexMap` instead of `BTreeMap` or `HashMap`
  - Provides O(1) lookups (better than BTreeMap's O(log n))
  - Preserves insertion order (unlike HashMap)
  - Eliminates need for separate index tracking

- **NodeBody Structure**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct NodeBody {
    pub map: IndexMap<ValueKey, NodeItem>,
}
```

- **Insertion Order Preservation**: Properties and children maintain insertion order
  - Display/serialization shows items in insertion order
  - No manual index synchronization needed
  - Tests verify order is preserved across operations

- **Usage Patterns**:
```rust
// Adding properties preserves order
node.set_prop("zebra", 1);  // Added first
node.set_prop("apple", 2);  // Added second
// Display shows: zebra first, then apple (not alphabetical)

// Adding children preserves order
node.add_kid(Node::new("kid1"));
node.add_kid(Node::new("kid2"));
// Iteration returns: kid1, then kid2
```

- **Performance Characteristics**:
  - Lookup: O(1) average case
  - Insertion: O(1) average case
  - Iteration: O(n) in insertion order
  - Memory: Single IndexMap instead of BTreeMap + Vec

### Obj Structure

The `Obj` structure (in `crates/auto-val/src/obj.rs`) also uses `IndexMap` for the same reasons:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    values: IndexMap<ValueKey, Value>,
}
```

**Important**: IndexMap cannot be used in const contexts, so:
- `Obj::new()` is not const
- Use `OnceLock` for static Obj instances (see `value.rs:node_nil()` and `obj_empty()`)
- Removed `Obj::EMPTY` constant; use `Obj::new()` instead

## Common Development Tasks

### ⚠️ CRITICAL: Test Expectation Rules

**IMPORTANT**: When fixing failing tests, you have two options:

1. **Fix the implementation** to match the expected output (PREFERRED)
2. **Ask for permission** before changing test expectations

**NEVER modify test expected output without explicit user permission.**

Test expectations define the correct behavior. If tests fail, the implementation is wrong, not the test. Always fix the code to match the test, unless the test itself is demonstrably incorrect (e.g., testing for a bug).

### Creating Plans for Complex Tasks

When working on complex features or refactoring tasks that require planning:

1. **Create a plan file** in `docs/plans/` with:
   - Consecutive numbering (e.g., `006-my-plan.md`)
   - Descriptive name (kebab-case)
   - Comprehensive but concise content

2. **Plan file format**:
   - Objective/Goal
   - Current state/Problem
   - Proposed solution/Design
   - Implementation steps
   - Testing strategy
   - Success criteria

3. **Example plan names**:
   - `006-to-atom-comprehensive-tests.md`
   - `007-refactor-parser-memory.md`

**Why**: Plans provide context, enable review, and create a historical record of design decisions.

### ⚠️ CRITICAL: Never Edit Generated C Files

**DO NOT manually edit `.c` or `.h` files in `stdlib/auto/`** - These are **auto-generated** by the C transpiler from `.at` source files.

**Example:**
- ❌ **WRONG**: Edit `stdlib/auto/io.c` or `stdlib/auto/io.h`
- ✅ **RIGHT**: Edit `stdlib/auto/io.at` → Transpiler generates `.c`/`.h` automatically

**Why:** The C transpiler regenerates these files from `.at` sources. Manual edits will be overwritten!

**How to add C functions:**
1. Edit the `.at` file in `stdlib/auto/` (e.g., `io.at`)
2. Add your function in the `# C` section
3. Run `auto.exe c your_file.at` to regenerate C code
4. The transpiler will create/update the corresponding `.c` and `.h` files

### Commit Message Guidelines

**Keep commit messages concise and focused.**

**Example:**
```
Fix C transpiler: void params and type inference
```

**Not:**
```
Fix C transpiler to generate void for functions with no parameters
and infer return types from method calls like File.read_text()
```

**Why:** Shorter commit messages are easier to read in git logs and PR histories. Focus on what changed and why, not implementation details.

### Adding a New Test
```bash
# Add test case to tests/lexer_tests.md or tests/parser_tests.md
# Then run the corresponding test runner
./build/Debug/test_lexer.exe
./build/Debug/test_parser.exe
```

### Adding a2c (Auto-to-C) Test Cases

The C transpiler test framework (a2c tests) validates AutoLang-to-C transpilation through numbered test cases.

**Test Location**: `crates/auto-lang/test/a2c/`

**Directory Structure**:
```
crates/auto-lang/test/a2c/
├── 000_hello/
│   ├── hello.at              # AutoLang source input
│   ├── hello.expected.c      # Expected C output
│   └── hello.expected.h      # Expected header output
├── 100_std_hello/
│   ├── std_hello.at
│   ├── std_hello.expected.c
│   └── std_hello.expected.h
└── ...
```

**Test Naming Convention**:
- `000-099_*`: Core language features (hello, array, func, struct, etc.)
- `100-199_*`: Standard library tests (std_hello, std_getpid, std_file, etc.)

**How Tests Work**:
1. Test functions are defined in `crates/auto-lang/src/trans/c.rs` as `test_XXX_name()`
2. Each test calls `test_a2c("XXX_name")` with the test case identifier
3. The test runner:
   - Reads the `.at` source file
   - Transpiles it to C using `transpile_c()`
   - Compares generated C code with `.expected.c` and `.expected.h`
   - If output differs, creates `.wrong.c` and `.wrong.h` files for comparison

**Creating a New Test**:
```bash
# 1. Create test directory
mkdir crates/auto-lang/test/a2c/106_my_test

# 2. Create input file
# Edit: crates/auto-lang/test/a2c/106_my_test/my_test.at

# 3. Generate expected output (first run - will create .wrong files)
cargo test -p auto-lang test_106_my_test

# 4. Review .wrong.c and .wrong.h, if correct rename to .expected.*
mv crates/auto-lang/test/a2c/106_my_test/my_test.wrong.c \
   crates/auto-lang/test/a2c/106_my_test/my_test.expected.c
mv crates/auto-lang/test/a2c/106_my_test/my_test.wrong.h \
   crates/auto-lang/test/a2c/106_my_test/my_test.expected.h

# 5. Add test function to crates/auto-lang/src/trans/c.rs
# Add at end of test module:
#[test]
fn test_106_my_test() {
    test_a2c("106_my_test").unwrap();
}
```

**Running Tests**:
```bash
# Run all a2c tests
cargo test -p auto-lang -- trans

# Run specific test
cargo test -p auto-lang test_100_std_hello

# Run test and see comparison if it fails
cargo test -p auto-lang test_106_my_test
# Then compare: fc /b my_test.wrong.c my_test.expected.c (Windows)
# Or use: diff my_test.wrong.c my_test.expected.c (Unix)
```

**Test Case Example** (`100_std_hello`):
- **Input** (`std_hello.at`):
  ```auto
  use auto.io: say
  fn main() { say("hello!") }
  ```
- **Expected C** (`std_hello.expected.c`):
  ```c
  #include "std_hello.h"
  int main(void) { say("hello!"); return 0; }
  ```
- **Expected Header** (`std_hello.expected.h`):
  ```c
  #pragma once
  #include "auto/io.h"
  ```

### Debugging Tokenization
```c
// Use test_fstr_simple for isolated lexer testing
./build/Debug/test_fstr_simple.exe
```

### Checking AST Output
```c
// AST repr functions provide human-readable debugging
char* ast_str = code_repr(ast);
printf("%s\n", ast_str);
free(ast_str);
```

## Known Issues and Limitations

1. **For loop variable access** - Accessing loop variable inside loop body may return garbage data
2. **String literal parsing** - Some string edge cases show garbage characters
3. **Unary operations** - Operator representation may be incorrect
4. **If expressions** - Currently parsed as statements, not expressions
5. **F-string prefix** - `f"` is tokenized as `<ident:f>` followed by f-string tokens (not yet unified)

## File Structure Conventions

- `.at` extension - AutoLang source files
- `crates/auto-lang/` - Main compiler implementation (Rust)
- `crates/auto-val/` - Value system and data structures (Rust)
- `crates/auto-lang/src/trans/` - Transpilers (c.rs for C, rust.rs for Rust)
- `crates/auto-lang/test/a2c/` - Auto-to-C transpiler tests
- `crates/auto-lang/test/a2r/` - Auto-to-Rust transpiler tests
- `auto/` - Self-hosted compiler source files (.at files)
- `stdlib/auto/` - Standard library AutoLang code
- `docs/` - Documentation and resources

## Error Message System (Rust Implementation)

### Overview

The Rust implementation (`crates/auto-lang/`) features a comprehensive error reporting system powered by `miette` and `thiserror`, providing IDE-grade diagnostic output similar to Rust's compiler.

### Error Types

Located in `crates/auto-lang/src/error.rs`:

- **SyntaxError** (E0001-E0007): Parser errors
  - `UnexpectedToken`, `InvalidExpression`, `UnterminatedString`
  - `InvalidEscapeSequence`, `MissingDelimiter`, `Generic`
  
- **TypeError** (E0101-E0105): Type checking errors
  - `TypeMismatch`, `InvalidOperation`, `NotCallable`
  - `InvalidIndexType`, `InvalidArraySize`
  
- **NameError** (E0201-E0204): Variable/binding errors
  - `UndefinedVariable`, `DuplicateDefinition`
  - `ImmutableAssignment`, `UndefinedFunction`
  
- **RuntimeError** (E0301-E0305): Runtime evaluation errors
  - `DivisionByZero`, `ModuloByZero`, `IndexOutOfBounds`
  - `InvalidAssignmentTarget`, `BreakOutsideLoop`

### Error Display Features

**Current Implementation** (✅ Complete):
- ✅ Error codes (e.g., `auto_syntax_E0001`)
- ✅ File location with line:column (e.g., `[test.at:1:3]`)
- ✅ Source code snippets with visual indicators
- ✅ Color-coded output (red for errors, yellow for warnings)
- ✅ Help text for common errors

**Example Error Output**:
```
Error: auto_syntax_E0007

  × syntax error
  ╰─▶ syntax error
   ╭─[test_error.at:1:3]
 1 │ let x = 1; x = 2
   ·          ┬
   ·          ╰── Syntax error: Assignment not allowed for let store: x
   ╰────
```

### Error Result Type

All functions now return `AutoResult<T>` instead of basic types:

```rust
pub type AutoResult<T> = std::result::Result<T, AutoError>;
```

**Usage in parser**:
```rust
use crate::error::{pos_to_span, SyntaxError};
use crate::error::AutoResult;

pub fn expect(&mut self, kind: TokenKind) -> AutoResult<()> {
    if self.is_kind(kind) {
        self.next();
        Ok(())
    } else {
        let span = pos_to_span(self.cur.pos);
        Err(SyntaxError::UnexpectedToken {
            expected: format!("{:?}", kind),
            found: self.cur.text.to_string(),
            span,
        }.into())
    }
}
```

### Source Code Attachment

Syntax errors automatically include source code for snippet display:

```rust
// In lib.rs run() and run_file()
Err(AutoError::Syntax(err)) => {
    Err(AutoError::with_source(err, "<input>".to_string(), code.to_string()))
}
```

### Working with Errors

**Creating errors**:
```rust
// Simple error with message
Err(SyntaxError::Generic {
    message: "Invalid syntax".to_string(),
    span: pos_to_span(self.cur.pos),
}.into())

// Structured error with details
Err(SyntaxError::UnexpectedToken {
    expected: "Identifier".to_string(),
    found: "+".to_string(),
    span: pos_to_span(self.cur.pos),
}.into())
```

**Handling errors in main.rs**:
```rust
use miette::{MietteHandlerOpts, Result};

fn main() -> Result<()> {
    // Set up miette for beautiful error reporting
    miette::set_hook(Box::new(|_| {
        Box::new(MietteHandlerOpts::new()
            .terminal_links(true)
            .build())
    })).ok();
    
    // Errors automatically display with source snippets
    let result = auto_lang::run_file(&path)?;
    println!("{}", result);
    Ok(())
}
```

### Diagnostic Implementation Details

**Critical**: `AutoError` implements `Diagnostic` trait manually to properly delegate to inner errors:

```rust
impl Diagnostic for AutoError {
    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        match self {
            AutoError::SyntaxWithSource(e) => e.source_code(),
            _ => None,
        }
    }
    
    fn labels<'a>(&'a self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + 'a>> {
        match self {
            AutoError::Syntax(e) => e.labels(),
            AutoError::SyntaxWithSource(e) => e.labels(),
            // ... delegate to inner errors
        }
    }
}
```

This manual implementation is necessary because the `#[error(transparent)]` derive macro doesn't automatically forward `source_code()` and `labels()` methods.

### Span Tracking

Convert lexer `Pos` to miette `SourceSpan`:

```rust
pub fn pos_to_span(pos: Pos) -> SourceSpan {
    SourceSpan::new(pos.pos.into(), pos.len.into())
}
```

The `Pos` struct (from lexer) tracks:
- `line`: Line number (1-based)
- `at`: Column number (1-based)
- `pos`: Absolute byte offset
- `len`: Token length in bytes

### Testing Error Output

**Run a file with syntax errors**:
```bash
# Create test file
echo "1 + " > test_error.at

# Run to see error display
auto.exe run test_error.at
```

**Expected output**:
- Error code at top
- Color-coded error message
- Source code snippet with arrow pointing to error
- Detailed error message in label

### Future Enhancements

See `docs/plans/001-error-message-system.md` for full roadmap:

**High Priority**:
- Parser error recovery (show multiple errors at once)
- Evaluator/runtime error integration
- Stack traces for runtime errors

**Medium Priority**:
- Enhanced error messages with "did you mean?" suggestions
- Warning system (unused variables, dead code)
- Error code documentation

**Low Priority**:
- JSON output format for IDEs
- Language Server Protocol integration
- C implementation port

### Common Patterns

**Returning errors from parser**:
```rust
// Always use .into() to convert SyntaxError -> AutoError
return Err(SyntaxError::Generic {
    message: format!("Invalid expression: {}", op),
    span: pos_to_span(self.cur.pos),
}.into());

// In match arms, use Err(...) directly
match token.kind {
    TokenKind::Number => Ok(...),
    _ => Err(SyntaxError::UnexpectedToken {
        expected: "Number".to_string(),
        found: format!("{:?}", token.kind),
        span: pos_to_span(token.pos),
    }.into()),
}
```

**Adding new error variants**:
```rust
// 1. Add variant to error enum
#[derive(Error, Diagnostic, Debug)]
pub enum SyntaxError {
    #[error("my new error")]
    #[diagnostic(code(auto_syntax_E0008))]
    MyNewError {
        #[label("here's the problem")]
        span: SourceSpan,
    },
}

// 2. Use in parser
Err(SyntaxError::MyNewError {
    span: pos_to_span(self.cur.pos),
}.into())
```

### Troubleshooting

**Error: source_code() not being called**
- Ensure `AutoError::Diagnostic` implementation delegates to inner error
- Check that error is wrapped in `SyntaxWithSource` before returning
- Verify `source_code()` returns `Some(&self.source)` not `None`

**Error: labels not displaying**
- Verify `SyntaxError` variant has `#[label("...", ...)]` attribute
- Check that span is within source code bounds
- Ensure `labels()` method returns `Some(Box<...>)` not `None`

**Error codes not showing**
- Verify `#[diagnostic(code(...))]` attribute is present
- Use underscores not dots: `auto_syntax_E0001` not `auto.syntax.E0001`
- Check that diagnostic code is unique across all error types

## Type Inference System (Rust Implementation)

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
