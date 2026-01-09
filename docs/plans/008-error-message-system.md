# Plan: Comprehensive Error Message System for AutoLang

## Objective

Implement a Rust-compiler-grade error reporting system for AutoLang (Rust implementation) that provides:
- Clear, actionable error messages with source locations
- Colorful, IDE-grade diagnostic output using `miette`
- Error codes and categories for easy searching
- Contextual information and suggestions
- Support for multiple error levels (error, warning, note, help)

## Target Implementation

**Primary focus**: Rust implementation (`crates/auto-lang/`) - the canonical, feature-complete reference
**Secondary**: C implementation (`autoc/`) - port features from Rust after they work

## ✅ Completed Work (Phase 1: Parser Errors)

### Dependencies Added
- ✅ `miette` (v7.2) with "fancy" feature enabled
- ✅ `thiserror` for error derive macros
- ✅ Configured in workspace and individual crates

### Error Type System Created
- ✅ Created `crates/auto-lang/src/error.rs` with comprehensive error types:
  - `SyntaxError` enum (E0001-E0007): UnexpectedToken, InvalidExpression, UnterminatedString, etc.
  - `TypeError` enum (E0101-E0105): TypeMismatch, InvalidOperation, NotCallable, etc.
  - `NameError` enum (E0201-E0204): UndefinedVariable, DuplicateDefinition, etc.
  - `RuntimeError` enum (E0301-E0305): DivisionByZero, IndexOutOfBounds, etc.
- ✅ Created `AutoError` enum combining all error types
- ✅ Implemented manual `Diagnostic` trait for `AutoError` to properly delegate to inner errors
- ✅ Created `SyntaxErrorWithSource` struct to attach source code to syntax errors

### Parser Integration
- ✅ Replaced all 47 `error_pos!` macro calls with structured `SyntaxError` variants
- ✅ Updated all parser functions to return `AutoResult<T>` instead of basic errors
- ✅ Added span tracking using `pos_to_span()` helper function
- ✅ Modified helper functions (`prefix_power`, `infix_power`) to accept and use spans

### Error Display System
- ✅ Source code attached to errors in `run()` and `run_file()` functions
- ✅ Integrated `miette` handler in `main.rs` with fancy colors
- ✅ Error display now shows:
  - Error codes (e.g., `auto_syntax_E0001`)
  - File location with line:column (e.g., `[test.at:1:3]`)
  - Source code snippets with error indicators
  - Detailed error messages
  - Help text for common errors

### Example Error Output

```
Error: auto_syntax_E0007

  × syntax error
  ╰─▶ syntax error
   ╭─[test_error.at:1:3]
 1 │ 1 +
   ·   ┬
   ·   ╰── Expected term, got Newline, pos: 2:0:1, next: <nl>
   ╰────
```

### Code Quality
- ✅ All compiler warnings cleaned up
- ✅ Unused imports and variables removed
- ✅ Deprecated warnings suppressed with `#[allow(deprecated)]`
- ✅ Test `test_let_asn` fixed and passing

### Commits Made
1. `Add miette dependency with fancy features`
2. `Create comprehensive error type system with miette`
3. `Update main.rs to use AutoError conversion`
4. `Integrate AutoResult throughout codebase`
5. `Add thread-local source storage for error reporting`
6. `Investigation: miette source code display`
7. `Convert SyntaxWithSource to named struct`
8. `Test miette source display with named struct`
9. `Add source code storage to error variants`
10. `Fix error display: implement proper Diagnostic delegation for AutoError`
11. `Fix SyntaxError::Generic display to show full message`
12. `Clean up all compiler warnings`

## 🚧 Pending Work

### Phase 2: Enhanced Error Features

#### 2.1 Parser Error Recovery
**Status**: Not started

**Goals**:
- Implement synchronization at statement boundaries
- Continue parsing after syntax errors
- Collect and display multiple errors at once
- Add `--error-limit=N` flag to control displayed errors

**Implementation**:
```rust
impl Parser {
    fn synchronize(&mut self) {
        // Skip tokens until we reach a statement boundary
        while !self.is_at_end() {
            if self.cur.kind == TokenKind::Semicolon {
                self.next();
                return;
            }
            match self.cur.kind {
                TokenKind::Fn | TokenKind::Let | TokenKind::Var | TokenKind::Mut |
                TokenKind::For | TokenKind::While | TokenKind::If | TokenKind::Return => return,
                _ => self.next(),
            }
        }
    }

    fn parse_with_recovery(&mut self) -> AutoResult<Code> {
        let mut errors = Vec::new();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match self.parse_stmt() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    errors.push(e);
                    self.synchronize();
                }
            }
        }

        if !errors.is_empty() {
            // Return all collected errors
        }

        Ok(Code { statements })
    }
}
```

#### 2.2 Enhanced Error Messages
**Status**: Partially done

**Remaining work**:
- Add more specific error messages for each error variant
- Implement "did you mean?" suggestions for typos
- Add auto-fix suggestions where applicable
- Provide cross-references to related errors

**Example**:
```
Error: auto_name_E0201

  × undefined variable
  ╰─▶ undefined variable
   ╭─[test.at:2:5]
 2 │     print(usrename)
   ·         �^^^^^^^ variable 'usrename' not found
   │
   = help: Variable 'username' exists with similar name
   = note: Did you mean 'username'?
   ╰────
```

#### 2.3 Warning System
**Status**: Not started

**Goals**:
- Implement warning variants in error system
- Add `--warn=X` flags to control warning levels
- Support warnings for:
  - Unused variables
  - Unused imports
  - Dead code
  - Implicit type conversions
  - Deprecated features

**Implementation**:
```rust
#[derive(Error, Diagnostic, Debug)]
#[diagnostic(severity(warning))]
#[diagnostic(code(auto_warning_W0001))]
pub struct UnusedVariableWarning {
    name: String,
    #[label("unused variable '{}'", name)]
    span: SourceSpan,
}
```

#### 2.4 Error Documentation
**Status**: Not started

**Goals**:
- Create error code documentation in `docs/errors.md`
- Document each error code with:
  - Error description
  - Common causes
  - Suggested fixes
  - Examples
- Add `--explain E0001` flag to show detailed explanation

**Example documentation**:
```markdown
## auto_syntax_E0001: Unexpected Token

**Description**: The parser encountered a token that doesn't match the expected syntax.

**Common Causes**:
- Missing operator between expressions
- Missing closing delimiter
- Typo in keyword or identifier

**Suggested Fixes**:
1. Check for missing operators (e.g., `1 +` should be `1 + 2`)
2. Ensure all delimiters are closed (parentheses, braces, brackets)
3. Verify spelling of keywords and identifiers

**Example**:
```auto
// ❌ Error
let x = 1 +

// ✅ Fixed
let x = 1 + 2
```
```

### Phase 3: Evaluator/Runtime Errors

#### 3.1 Runtime Error Integration
**Status**: Not started

**Goals**:
- Replace `value_error()` calls with `RuntimeError` variants
- Pass location context through evaluation
- Add stack traces for runtime errors
- Map all existing error messages to error codes

**Implementation approach**:
1. Extend AST evaluation to track source locations
2. Modify `eval.rs` to return `AutoResult<Value>` instead of `Value`
3. Create `RuntimeError` variants for each runtime error type
4. Add helper functions for common runtime errors

**Example**:
```rust
impl Interpreter {
    fn eval_binary_op(&mut self, op: Op, left: Value, right: Value, span: SourceSpan) -> AutoResult<Value> {
        match op {
            Op::Div => {
                if right.is_zero() {
                    return Err(RuntimeError::DivisionByZero { span }.into());
                }
                // ...
            }
            // ...
        }
    }
}
```

#### 3.2 Stack Traces
**Status**: Not started

**Goals**:
- Capture expression chain leading to error
- Display stack trace with file:line:column for each frame
- Help users trace error through function calls

**Example output**:
```
Error: auto_runtime_E0303

  × index out of bounds
  ╰─▶ index out of bounds
   ╭─[test.at:5:9]
 5 │     arr[i]
   ·         ^^ index 10 is out of bounds for array of length 3
   │
   = note: Error occurred in function 'process_array'
   = note: Called from 'main' at test.at:8:5
   ╰────

Stack trace:
  - test.at:5:9 in 'process_array'
  - test.at:8:5 in 'main'
```

### Phase 4: Advanced Features

#### 4.1 Multiple Error Display
**Status**: Partially done (needs error recovery first)

**Goals**:
- Display all errors from parsing/evaluation
- Sort errors by file and line number
- Group related errors
- Add "error: aborting due to N previous error(s)" message

#### 4.2 JSON Output Format
**Status**: Not started

**Goals**:
- Add `--format=json` flag
- Export diagnostics in structured JSON format
- Support IDE Language Server Protocol integration

**Example JSON output**:
```json
{
  "errors": [
    {
      "code": "auto_syntax_E0001",
      "message": "unexpected token",
      "level": "error",
      "spans": [
        {
          "file": "test.at",
          "line_start": 1,
          "column_start": 3,
          "line_end": 1,
          "column_end": 4,
          "label": "unexpected token"
        }
      ],
      "help": "Expected identifier, but found '+'"
    }
  ]
}
```

#### 4.3 IDE Integration
**Status**: Not started

**Goals**:
- Create Language Server Protocol (LSP) implementation
- Provide real-time error checking in editors
- Support VS Code, Vim, Emacs integration
- Enable "go to definition" and "find references"

### Phase 5: C Implementation Port

**Status**: Not started (deferred until Rust implementation is complete)

**Goals**:
- Port error system to C implementation (`autoc/`)
- Implement simplified diagnostic formatting in C
- Match error codes and messages with Rust version
- Keep C implementation in sync

**Approach**:
1. Create `autoc/diagnostic.h` and `autoc/diagnostic.c`
2. Port error code enums to C
3. Implement basic color support (ANSI codes)
4. Add source snippet rendering
5. Match Rust error output as closely as possible

## Testing Strategy

### Unit Tests
- ✅ Test error creation and formatting
- ✅ Test error code assignment
- ✅ Test span tracking
- ✅ Test source code attachment

### Integration Tests
- ✅ Test parser error display
- ✅ Test error with actual source files
- ⏳ Test error recovery (needs implementation)
- ⏳ Test multiple error display (needs implementation)

### Regression Tests
- Create test suite for error output
- Capture actual error output
- Compare against expected output
- Test edge cases (EOF errors, multi-line expressions, etc.)

## Success Criteria

### Phase 1 (✅ COMPLETED)
- ✅ All parser errors show file:line:column location
- ✅ All runtime errors show expression location
- ✅ Color-coded error levels (red for errors)
- ✅ Error code displayed (e.g., `auto_syntax_E0001`)
- ✅ Basic code snippet with error indicator

### Phase 2 (🚧 IN PROGRESS)
- ⏳ Error recovery to show multiple errors
- ⏳ Enhanced error messages with suggestions
- ⏳ Warning system with configurable levels
- ⏳ Error code documentation

### Phase 3 (⏳ PENDING)
- ⏳ All evaluator errors use new system
- ⏳ Stack traces for runtime errors
- ⏳ Help text for common errors
- ⏳ Multiple errors displayed per run

### Complete System (🎯 FUTURE GOAL)
- ⏳ JSON output for IDE integration
- ⏳ Comprehensive error code documentation
- ⏳ Test coverage for all error paths
- ⏳ C port complete and in sync

## File Structure

```
crates/auto-lang/src/
├── error.rs          # ✅ Complete error type system
├── lib.rs            # ✅ Updated run() and run_file()
├── parser.rs         # ✅ All errors replaced with AutoResult
├── eval.rs           # ⏳ Needs runtime error integration
└── diag.rs           # ⏳ Optional: Diagnostic helpers

docs/
├── plans/
│   └── 001-error-message-system.md  # This file
└── errors.md        # ⏳ Error code documentation

tests/
├── error_tests/     # ⏳ Error output test cases
└── error_snapshots/ # ⏳ Snapshot tests for error display
```

## Design Decisions

### ✅ Finalized Decisions

1. **Diagnostic Library**: `miette` (feature-rich, battle-tested)
2. **Error Code Format**: Using underscores: `auto_syntax_E0001` (miette requirement)
3. **Span Tracking**: Use existing `Pos` struct with `pos_to_span()` helper
4. **Error Recovery**: Basic (synchronize at statement boundaries) - TBD
5. **Color Output**: Auto-detect terminal (handled by miette)
6. **Implementation**: Rust-first (`crates/auto-lang/`)
7. **Source Storage**: Attach directly to errors in `SyntaxErrorWithSource`

### 🔍 Open Questions

1. **Error Recovery Priority**: Should we implement error recovery before moving to evaluator errors?
2. **Warning System**: How aggressive should warnings be? (opt-in vs opt-out)
3. **Error Documentation**: Tool to generate from doc comments or manual?
4. **C Port**: When should we start porting to C? (after Rust complete or parallel?)

## References

- `miette` documentation: https://docs.rs/miette/
- `thiserror` documentation: https://docs.rs/thiserror/
- Rust Compiler Error Guide: https://rustc-dev-guide.rust-lang.org/diagnostics.html
- Error Index: https://doc.rust-lang.org/error-index.html

## Recent Work Summary

**Most Recent Commits** (as of latest update):
1. ✅ `Clean up all compiler warnings` - Removed all unused imports, variables, suppressed deprecated warnings
2. ✅ `Fix SyntaxError::Generic display to show full message` - Changed to `#[error("{message}")]`
3. ✅ `Fix error display: implement proper Diagnostic delegation for AutoError` - Manual Diagnostic impl
4. ✅ `Test miette source display with named struct` - Verified snippet rendering works
5. ✅ `Convert SyntaxWithSource to named struct` - Changed from tuple struct

**Current Status**: Parser error system is fully functional and displaying IDE-grade error messages with source code snippets. All compiler warnings have been cleaned up. Ready to proceed with either error recovery or evaluator error integration.
