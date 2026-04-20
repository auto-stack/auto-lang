# Error Message System (Rust Implementation)

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

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

**Current Implementation** (Complete):
- Error codes (e.g., `auto_syntax_E0001`)
- File location with line:column (e.g., `[test.at:1:3]`)
- Source code snippets with visual indicators
- Color-coded output (red for errors, yellow for warnings)
- Help text for common errors

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
