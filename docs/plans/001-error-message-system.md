# Plan: Comprehensive Error Message System for AutoLang

## Objective

Implement a Rust-compiler-grade error reporting system for AutoLang (Rust implementation) that provides:
- Clear, actionable error messages with source locations
- Colorful, IDE-grade diagnostic output using `miette` or custom diagnostics
- Error codes and categories for easy searching
- Contextual information and suggestions
- Support for multiple error levels (error, warning, note, help)

## Target Implementation

**Primary focus**: Rust implementation (`crates/auto-lang/`) - the canonical, feature-complete reference
**Secondary**: C implementation (`autoc/`) - port features from Rust after they work

## Current State Analysis (Rust Implementation)

### Existing Error Handling

Let me explore the current error handling in the Rust codebase to understand what needs improvement.

### Known Gaps (Based on Common Issues)
- Limited source location information in errors
- Basic error messages without rich context
- No error codes for categorization
- Limited suggestions for fixes
- No multi-error collection and display

## Design Decisions

### 1. Use Existing Rust Ecosystem

Instead of building from scratch, leverage excellent Rust diagnostic libraries:
- **`miette`** - Modern diagnostic library (recommended)
  - Beautiful, IDE-grade error output
  - Source code snippets with highlights
  - Error codes and labels
  - Easy integration with `anyhow`/`thiserror`
- **`codespan`** - Alternative diagnostic library
- **Custom implementation** - Build our own if needed

### 2. Error Structure Design

Using `miette`:

```rust
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Diagnostic, Debug)]
#[error("syntax error")]
#[diagnostic(
    code(auto.syntax.E0001),
    help("Try removing this token")
)]
pub struct SyntaxError {
    #[source_code]
    src: String,

    #[label("unexpected token")]
    span: SourceSpan,
}

#[derive(Error, Diagnostic, Debug)]
#[diagnostic(code(auto.type.E0101))]
pub enum TypeError {
    #[error("type mismatch")]
    #[help("expected {expected}, found {found}")]
    Mismatch {
        expected: String,
        found: String,
        #[label("this expression has type {found}")]
        span: SourceSpan,
    },

    #[error("invalid operation for type")]
    InvalidOperation {
        op: String,
        ty: String,
        #[label("cannot {op} value of type {ty}")]
        span: SourceSpan,
    },
}
```

### 3. Error Code System

Organized by category (using diagnostic codes):
- **auto.syntax.E0001-E0099**: Syntax/Parser errors
  - `E0001`: Unexpected token
  - `E0002`: Expected token not found
  - `E0003`: Invalid expression syntax
- **auto.type.E0100-E0199**: Type errors
  - `E0101`: Type mismatch
  - `E0102`: Invalid operation for type
- **auto.name.E0200-E0299**: Name/Binding errors
  - `E0201`: Undefined variable
  - `E0202`: Duplicate definition
- **auto.runtime.E0300-E0399**: Runtime errors
  - `E0301`: Division by zero
  - `E0302`: Index out of bounds
- **auto.warning.W0000-W0099**: Warnings
  - `W0001`: Unused variable
  - `W0002`: Implicit type conversion

### 4. Output Format Design

Using `miette` provides beautiful output out of the_box:

```
error[E0101]: type mismatch
  └─> examples/test.at:5:10
   │
 5 │     let x: int = "hello";
   │              ^^^^^^^^^ expected `int`, found `str`
   │
   = help: remove type annotation or change value
```

Color scheme (handled by miette):
- Error: Red
- Warning: Yellow
- Note: Blue
- Help: Green
- Location info: Cyan
- Code highlights: Contextual

### 5. Error Collection and Reporting

**Batch error collection**:
- Use `Result<Vec<T>, Vec<Error>>` pattern or custom error collector
- Collect multiple errors before reporting
- Show all errors at end

**Error recovery**:
- Continue parsing after syntax errors
- Show all errors in file before aborting
- For evaluation: use `Result` propagation

## Implementation Plan (Phase 1: MVP)

**Goal**: Implement core diagnostic infrastructure with error recovery

### Step 1.1: Add Dependencies and Setup

- [ ] Add `miette` and `thiserror` to `Cargo.toml`
- [ ] Add `miette` feature flags (fancy-colors, etc.)
- [ ] Create error module structure (`src/error.rs` or `src/diag.rs`)

### Step 1.2: Define Error Types

- [ ] Define `SyntaxError` with diagnostic attributes
- [ ] Define `TypeError` enum with variants
- [ ] Define `NameError` (undefined/duplicate names)
- [ ] Define `RuntimeError` (division by zero, index errors)
- [ ] Define top-level `AutoError` enum combining all errors

### Step 1.3: Integrate with Parser

- [ ] Find current parser error handling
- [ ] Replace with `SyntaxError` variants
- [ ] Add span tracking to all AST nodes
- [ ] Implement error recovery (synchronize on statement boundaries)
- [ ] Collect multiple errors instead of returning early

### Step 1.4: Integrate with Type Checker (if exists)

- [ ] Replace type errors with `TypeError` variants
- [ ] Add source spans to type checking operations
- [ ] Provide helpful error messages and suggestions

### Step 1.5: Integrate with Evaluator

- [ ] Replace runtime errors with `RuntimeError` variants
- [ ] Ensure all operations return `Result<T, AutoError>`
- [ ] Add context to runtime errors (expression location)

### Step 1.6: Update Main Entry Point

- [ ] Replace basic error display with `miette` handler
- [ ] Configure `miette` with fancy colors
- [ ] Show all collected errors at end
- [ ] Return appropriate exit codes

### Step 1.7: Add Color Support

- [ ] Enable `fancy-colors` feature in miette
- [ ] Add `--color=always|never|auto` flag
- [ ] Auto-detect terminal capability (miette does this by default)

## Critical Files to Modify/Create

### New Files
- `crates/auto-lang/src/error.rs` - Main error definitions
- `crates/auto-lang/src/diag.rs` - Diagnostic helpers (optional)

### Modified Files (to explore)
- `crates/auto-lang/src/lib.rs` - Main entry point
- `crates/auto-lang/src/eval.rs` - Runtime error handling
- `crates/auto-lang/src/parser.rs` or similar - Parser errors
- `crates/auto-lang/src/ast.rs` - Add span tracking
- `crates/auto-lang/Cargo.toml` - Add dependencies
- `crates/auto-lang/src/main.rs` or CLI entry point - Error display

## Success Criteria (Phase 1 MVP)

### Minimal Viable Product
- ✅ All parser errors show file:line:column with code snippet
- ✅ All type errors show expected vs found types
- ✅ All runtime errors show expression location
- ✅ Color-coded error levels (handled by miette)
- ✅ Error code displayed (e.g., `auto::syntax::E0001`)
- ✅ Basic suggestions/help messages
- ✅ Error recovery to show multiple errors

## Implementation Strategy

### Leverage Existing Rust Ecosystem
1. **Use `miette`** - Battle-tested diagnostic library
2. **Use `thiserror`** - Derive macros for error enums
3. **Incremental migration** - Replace errors piece by piece
4. **Test with examples** - Verify error quality improves

### Error Recovery Strategy
- Synchronize at statement boundaries
- Skip tokens until synchronization point
- Continue parsing to collect more errors
- Report all errors at the end

### Testing Strategy
- Create test cases with invalid input
- Capture error output
- Verify error codes and spans are correct
- Check for helpful suggestions

## Design Considerations

### Why Use `miette`?

1. **Battle-tested**: Used by many Rust projects (cargo, etc.)
2. **Beautiful output**: IDE-grade error messages
3. **Easy integration**: Works with `thiserror`, `anyhow`
4. **Feature-rich**: Spans, labels, suggestions, error codes
5. **Customizable**: Can adapt to our needs

### Why Not Build From Scratch?

- **Reinventing the wheel**: Diagnostic libraries are hard to get right
- **Time-consuming**: Building quality diagnostics takes months
- **Less polish**: Unlikely to match miette's output quality
- **Maintenance burden**: More code to maintain

### Porting to C Implementation

After Rust implementation is complete:
1. Use Rust implementation as reference
2. Port error messages and codes to C
3. Implement simplified diagnostic formatting in C
4. Keep C implementation in sync

## Design Decisions Summary

All design decisions have been finalized:

✅ **Diagnostic Library**: `miette` (feature-rich, battle-tested)
✅ **Error Code Format**: Auto-style paths like `auto.syntax.E0001`
✅ **Span Tracking**: Add `Span` field to all AST nodes
✅ **Error Recovery**: Basic (synchronize at statement boundaries only)
✅ **Color Output**: Auto-detect terminal (handled by miette)
✅ **Implementation**: Rust-first (`crates/auto-lang/`)

## Next Steps

1. **Explore Rust implementation** - Understand current error handling
2. **Answer design questions** - Confirm library choice and strategy
3. **Start Step 1.1** - Add dependencies
4. **Incremental migration** - Replace errors one module at a time
5. **Test continuously** - Verify error quality improves

## References

- `miette` documentation: https://docs.rs/miette/
- `thiserror` documentation: https://docs.rs/thiserror/
- Rust Compiler Error Guide: https://rustc-dev-guide.rust-lang.org/diagnostics.html
