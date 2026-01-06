# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AutoLang is a multi-purpose programming language designed for automation with three implementations:
- **Rust version** (`crates/`): Primary reference implementation with full features
- **C version** (`autoc/`): Embedded-friendly port targeting resource-constrained environments
- **Self-hosted version** (`auto/`): Auto compiler written in AutoLang itself (just started)

The C version (`autoc`) transpiles AutoLang to C, supports dynamic configuration, and can be embedded as a scripting engine. The self-hosted version (`auto/`) represents the bootstrap effort to implement the AutoLang compiler in its own language.

## Build Commands

### C Implementation (autoc/)

```bash
# Configure build (from autoc/ directory)
mkdir build && cd build
cmake ..

# Build all targets
cmake --build .

# Build specific targets
cmake --build . --target autoc          # Main compiler executable
cmake --build . --target autoc_test     # Legacy test suite
cmake --build . --target test_parser    # Parser markdown tests
cmake --build . --target test_lexer     # Lexer markdown tests

# Run tests
ctest                              # Run all CMake tests
./autoc_test                       # Run legacy test suite
./build/Debug/test_parser.exe      # Run parser tests (Windows)
./build/Debug/test_lexer.exe       # Run lexer tests (Windows)

# Run autoc REPL or execute files
./autoc                    # Start REPL
./autoc file.at            # Execute AutoLang file
./autoc -e "1 + 2"         # Evaluate expression
```

### Rust Implementation

```bash
# From project root
cargo build --release        # Build all crates
cargo run --release          # Run REPL
cargo test                   # Run tests
cargo test -p auto-lang      # Test specific crate
```

### Self-Hosted Implementation (auto/)

```bash
# From project root
autoc auto/auto.at           # Compile the Auto compiler with autoc
./auto                       # Run the compiled Auto compiler (if available)
```

**Status**: Early stage - this is the beginning of the self-hosting effort. The compiler is not yet feature-complete.

## Architecture Overview

### Compilation Pipeline (C Implementation)

```
Source Code (.at files)
    ↓
Lexer (lexer.c) → Tokens
    ↓
Parser (parser.c) → AST (ast.h)
    ↓
Evaluator (eval.c) OR Transpiler (trans_c.c)
    ↓
Output (Value OR C code)
```

### Core Components

#### 1. **Lexer** (`lexer.c/h`)
- Tokenizes AutoLang source code
- Handles f-strings with `$variable` and `${expression}` syntax
- Uses `in_fstr_expr` flag to prevent infinite recursion during `${...}` processing
- Token types defined in `token.h`

**Key Implementation Detail**: When lexing f-strings with `${expr}`, the lexer:
1. Sets `lexer->in_fstr_expr = true` before processing expressions
2. Collects tokens in a temporary array to avoid buffer conflicts
3. Clears the flag after completing expression parsing

#### 2. **Parser** (`parser.c/h`)
- Recursive descent parser consuming lexer tokens
- Builds AST nodes defined in `ast.h`
- Handles expression precedence and control flow
- Uses `AutoStr` for string memory management (see `astr.c`)

**Memory Management**: The parser manually manages AST memory. When creating multi-statement ASTs, each statement must be individually allocated and added to the `stmts` array.

#### 3. **AST** (`ast.c/h`)
- Unified representation for expressions and statements
- Expression types: `int`, `ident`, `binary`, `unary`, `if`, `array`, `call`, `index`, etc.
- Statement types: `expr`, `store`, `for`, `while`, `break`, `ret`, `use`, etc.
- Provides `expr_repr()`, `stmt_repr()`, and `code_repr()` for debugging

**Important**: AST nodes use discriminated unions (kind enum + union). Always check `node->kind` before accessing union members.

#### 4. **Evaluator** (`eval.c`)
- Interprets AST nodes to produce `Value` results
- Supports multiple evaluation modes (SCRIPT, CONFIG, TEMPLATE)
- Uses `Universe` for variable scoping (see `universe.c`)

#### 5. **Value System** (`value.c/h`)
- Dynamic typing with runtime type tags
- Types: `int`, `uint`, `float`, `bool`, `str`, `array`, `object`, `nil`, `func`, `native`
- Reference counting not implemented (manual memory management required)

#### 6. **Universe/Scope** (`universe.c/h`)
- Manages variable bindings and nested scopes
- Global and local variable lookup
- Used by evaluator for variable resolution

### Test Infrastructure

#### Markdown Test Framework (`test_markdown.c`)
Common test utilities for lexer and parser:
- `read_file()` - Read file contents
- `parse_markdown_tests()` - Parse test cases from markdown format
- `compare_exact()` - Exact string comparison
- `compare_ignore_ws()` - Whitespace-insensitive comparison
- `run_markdown_test_suite()` - Generic test runner

**Test Case Format**:
```markdown
## Test Name

input_code

---

expected_output
```

#### Test Files
- `tests/lexer_tests.md` - Lexer test cases (30 tests)
- `tests/parser_tests.md` - Parser test cases (18 tests)
- `test_lexer.c` - Lexer test runner
- `test_parser.c` - Parser test runner
- `test.c` - Legacy unit tests (run with `autoc_test`)

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

## C Implementation Specifics

### Memory Management
- All allocations use `malloc`/`free`
- `AutoStr` (in `astr.c`) provides dynamic string helpers
- No garbage collection - manual cleanup required
- AST nodes must be individually freed (see AST cleanup in test files)

### AutoStr Utilities
```c
AutoStr s = astr_new("hello");           // Create string
astr_append(&s, " world");                // Append
astr_free(&s);                            // Free (required)
```

### Error Handling
- Functions return error codes or NULL on failure
- `eval_error` creates error values with messages
- No exception mechanism - check return values

### Header Organization
Headers are split by concern:
- `common.h` - Shared types and utilities
- `token.h` - Token types and lexer interface
- `lexer.h` - Lexer interface
- `ast.h` - AST node types
- `parser.h` - Parser interface
- `universe.h` - Scope management
- `eval.h` - Evaluator interface
- `value.h` - Value types
- `astr.h` - String utilities

## Implementation Strategy

This project uses a **three-tier implementation approach**:

1. **Rust version** (`crates/`) - The canonical, feature-complete reference implementation
2. **C version** (`autoc/`) - Portability layer for embedded systems (lags behind Rust)
3. **Self-hosted version** (`auto/`) - Bootstrap effort to implement the compiler in AutoLang itself (early stage)

When working on features:
- **Rust version is canonical** - Refer to `crates/auto-lang/src/` for correct behavior
- **C version follows Rust** - Port features from Rust to C, test parity
- **Self-hosted version follows C** - Use the C compiler to bootstrap the AutoLang compiler

### Self-Hosting Strategy

The self-hosted compiler represents the final stage of the bootstrap process:
1. Rust compiler (`crates/`) → implements full language
2. C compiler (`autoc/`) → ported from Rust, used to compile AutoLang code
3. Auto compiler (`auto/`) → written in AutoLang, compiled by `autoc`

This creates a self-sustaining ecosystem where AutoLang can compile itself.

### Porting Rust Features to C
- Map Rust enums to C enums with discriminated unions
- Convert `Result<T, E>` to return codes + error strings
- Replace `Rc<RefCell<T>>` with manual memory management
- Use `AutoStr` instead of `String`

## Common Development Tasks

### Adding a New Test
```bash
# Add test case to tests/lexer_tests.md or tests/parser_tests.md
# Then run the corresponding test runner
./build/Debug/test_lexer.exe
./build/Debug/test_parser.exe
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
- `tests/*.md` - Markdown test case files
- `autoc/*.h` - Header files (organized by component)
- `autoc/*.c` - Implementation files
- `auto/*.at` - Self-hosted Auto compiler source files
- `stdlib/auto/` - Standard library AutoLang code
- `docs/` - Documentation and resources
