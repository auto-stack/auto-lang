# Source Mapping for Self-Hosting Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** HIGH - Enables IDE-grade error messages in transpiled code
**Dependencies:** Plan 033 (Self-Hosting Compiler)
**Estimated Start:** During Plan 033 Phase 7 (Code Generation)
**Timeline:** 6-8 weeks

## Executive Summary

Implement source mapping system for AutoLang compiler that maps generated C code back to original .at source files. This enables IDE-grade error messages with file locations, source code snippets, and visual indicators - matching the quality of the Rust implementation's miette-based error reporting.

**Current State:**
- ✅ Rust implementation uses miette for beautiful error messages
- ✅ Error tracking with `SourceSpan` (line:column)
- ✅ Source code attachment for snippets
- ❌ No source mapping from AutoLang → C
- ❌ Cannot map C runtime errors back to AutoLang source
- ❌ Transpiled code has no location tracking

**Target State:**
- ✅ Generate source maps alongside C code
- ✅ Map C line numbers to AutoLang line:column
- ✅ Pretty-print errors with source snippets
- ✅ Support for multi-line errors
- ✅ IDE integration (Language Server Protocol)

**Timeline:** 6-8 weeks
**Complexity:** High (requires source tracking through transpilation, error formatting in C)

---

## 1. Why Source Mapping is Critical

### 1.1 Error Reporting Quality

**Without Source Mapping:**
```
Error: Division by zero
  File: generated.c, Line 42

  // User has no idea where this is in their .at file
  // Must manually search through C output
```

**With Source Mapping:**
```
Error: auto_runtime_E0301

  × Division by zero
  ╰─▶ Attempted to divide by zero
   ╭─[test.at:10:5]
10 │     let result = x / 0
   ·                  ──┬──
   ·                    ╰── Division by zero
   ╰────
```

### 1.2 Compiler Requirements

The self-hosted compiler needs to report errors on:

**Syntax Errors:**
```auto
// Parser error - needs line:column
fn main() {
    let x = 1 +  // Missing operand
}
```

**Type Errors:**
```auto
// Type checker error - needs expression location
fn add(a int, b str) int {  // Type mismatch
    return a + b
}
```

**Runtime Errors:**
```auto
// Runtime error - needs to map from C to AutoLang
fn main() {
    let x = 10
    let y = 0
    let z = x / y  // Runtime error in generated C
}
```

### 1.3 Current Rust Implementation

**Error Tracking (Rust):**
```rust
// error.rs
#[derive(Debug, Clone)]
pub struct SyntaxError {
    pub message: String,
    pub span: SourceSpan,  // Byte offset + length
}

// miette integration
impl Diagnostic for SyntaxError {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.source)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan>>> {
        Some(Box::new(std::iter::once(LabeledSpan::new(
            Some(self.message.clone()),
            self.span,
        ))))
    }
}
```

**Output:**
```
Error: auto_syntax_E0001

  × syntax error
  ╰─▶ unexpected token
   ╭─[test.at:1:10]
 1 │ fn main() {
   ·          ┬
   ·          ╰── Expected identifier, found `{`
   ╰────
```

**AutoLang needs equivalent capability** but transpiled to C.

---

## 2. Source Mapping Design

### 2.1 Source Map Format

**Option A: Custom JSON Format**
```json
{
  "version": 1,
  "source_file": "test.at",
  "generated_file": "test.c",
  "mappings": [
    {
      "c_line": 42,
      "c_col": 5,
      "at_line": 10,
      "at_col": 15,
      "at_end_line": 10,
      "at_end_col": 16,
      "context": "let result = x / 0"
    }
  ]
}
```

**Option B: Standard Source Map (simplified)**
```json
{
  "version": 3,
  "file": "test.c",
  "sourceRoot": "",
  "sources": ["test.at"],
  "names": [],
  "mappings": "AAAA,GAAG,GAAG,CAAC;IACF,CAAC;EACH,CAAC"
}
```

**Option C: Line Directives in C**
```c
// Auto-generated - do not edit
#line 10 "test.at"
int result = x / 0;

// Error will be reported at test.at:10
```

**Recommendation:** **Option A (Custom JSON)** + **Option C (Line Directives)**

- Use JSON for programmatic access (IDE, tools)
- Use `#line` directives for C compiler errors (gcc/clang)

### 2.2 Source Map Structure

**Data Structure:**
```auto
// Source map representation
type SourceMapping {
    c_file str
    at_file str

    // Mappings: c_line → AutoLang location
    mappings []Mapping
}

type Mapping {
    c_line uint
    c_col uint

    at_line uint
    at_col uint
    at_end_line uint  // For multi-line expressions
    at_end_col uint

    // Context for error display
    context str
}
```

**C Representation:**
```c
// source_map.h
typedef struct {
    const char* at_file;
    int at_line;
    int at_col;
    int at_end_line;
    int at_end_col;
    const char* context;
} SourceMapping;

typedef struct {
    int capacity;
    int size;
    SourceMapping* mappings;
} SourceMap;

// Create source map
SourceMap* SourceMap_new(int capacity);

// Add mapping
void SourceMap_add(SourceMap* map,
                   int c_line, int c_col,
                   int at_line, int at_col,
                   int at_end_line, int at_end_col,
                   const char* context);

// Lookup AutoLang location from C line
SourceMapping* SourceMap_lookup(SourceMap* map, int c_line);

// Free source map
void SourceMap_drop(SourceMap* map);
```

### 2.3 Error Display

**Error Formatter:**
```c
// error_formatter.h
typedef enum {
    ERROR_LEVEL_ERROR,
    ERROR_LEVEL_WARNING,
    ERROR_LEVEL_INFO,
} ErrorLevel;

typedef struct {
    ErrorLevel level;
    const char* code;
    const char* message;
    const char* at_file;
    int at_line;
    int at_col;
    int at_end_line;
    int at_end_col;
    const char* source_snippet;
    const char* hint;
} CompilerError;

// Format error with source snippet
char* format_error(CompilerError* err, const char* source);

// Print error to stderr
void print_error(CompilerError* err, const char* source);
```

**Usage:**
```c
// Example: Runtime division by zero
void runtime_division_by_zero(int c_line, SourceMap* map) {
    // Look up AutoLang location
    SourceMapping* mapping = SourceMap_lookup(map, c_line);

    // Create error
    CompilerError err = {
        .level = ERROR_LEVEL_ERROR,
        .code = "auto_runtime_E0301",
        .message = "Division by zero",
        .at_file = mapping->at_file,
        .at_line = mapping->at_line,
        .at_col = mapping->at_col,
        .at_end_line = mapping->at_end_line,
        .at_end_col = mapping->at_end_col,
        .source_snippet = mapping->context,
        .hint = "Check the divisor before division",
    };

    // Print beautiful error
    print_error(&err, read_source(mapping->at_file));
}
```

---

## 3. Implementation Phases

### Phase 1: Source Tracking in Transpiler (2-3 weeks)

**Objective:** Track source locations through transpilation

**Deliverables:**
1. Location tracking in C transpiler
2. Source map generation
3. Line directive insertion

**Files to Modify:**
```
crates/auto-lang/src/trans/c.rs
```

**Key Implementation:**

```rust
// trans/c.rs
pub struct CTranspiler {
    // Existing fields...
    pub name: AutoStr,
    pub scope: Shared<Universe>,

    // New: Source tracking
    source_map: SourceMap,
    current_c_line: usize,
}

impl CTranspiler {
    pub fn new(name: AutoStr) -> Self {
        CTranspiler {
            name,
            scope,
            source_map: SourceMap::new(),
            current_c_line: 1,
        }
    }

    // Track location when writing C code
    fn write_with_location(
        &mut self,
        out: &mut Sink,
        at_pos: Pos,
        data: &[u8],
    ) -> AutoResult<()> {
        // Write C code
        out.write(data)?;

        // Add mapping
        self.source_map.add_mapping(
            self.current_c_line,
            1,  // C column (simplified)
            at_pos.line,
            at_pos.at,
            at_pos.line,
            at_pos.at + at_pos.len,
            self.get_source_context(at_pos),
        );

        // Update line count
        self.current_c_line += data.iter().filter(|&&b| b == b'\n').count();

        Ok(())
    }

    // Get source context for error display
    fn get_source_context(&self, pos: Pos) -> AutoStr {
        // Read source file
        let source = self.load_source(&self.name)?;

        // Extract line
        let line = source.lines().nth(pos.line - 1).unwrap_or("");

        // Return context (the actual line)
        AutoStr::from(line)
    }

    // Generate line directives
    fn write_line_directive(
        &mut self,
        out: &mut Sink,
        at_pos: Pos,
    ) -> AutoResult<()> {
        writeln!(out, "#line {} \"{}\"", at_pos.line, self.name)?;
        Ok(())
    }
}
```

**Example Usage:**
```rust
// When transpiling an expression
fn transpile_expr(&mut self, out: &mut Sink, expr: &Expr) -> AutoResult<()> {
    match expr {
        Expr::Int { value, pos } => {
            // Track location
            self.write_with_location(
                out,
                *pos,
                format!("{}", value).as_bytes(),
            )?;
        }

        Expr::Binary { op, left, right, pos } => {
            self.write_with_location(out, *pos, b"(")?;
            self.transpile_expr(out, left)?;
            self.write_with_location(out, *pos, b" ")?;
            self.write_with_location(out, *pos, op.to_c().as_bytes())?;
            self.write_with_location(out, *pos, b" ")?;
            self.transpile_expr(out, right)?;
            self.write_with_location(out, *pos, b")")?;
        }

        _ => { /* ... */ }
    }
}
```

**Generated C with Line Directives:**
```c
// Auto-generated from test.at

#line 1 "test.at"
int main(void) {
#line 2 "test.at"
    int x = 10;
#line 3 "test.at"
    int y = 0;
#line 4 "test.at"
    int z = x / y;  // Error will be reported at test.at:4
#line 5 "test.at"
    return 0;
}
```

**Success Criteria:**
- Source tracking works for all AST nodes
- Line directives inserted correctly
- Source map generates valid JSON
- Unit tests pass

---

### Phase 2: Source Map Generation (1-2 weeks)

**Objective:** Generate and serialize source maps

**Deliverables:**
1. Source map JSON generation
2. Source map file writing
3. Source map validation

**Files to Create:**
```
crates/auto-lang/src/trans/source_map.rs
```

**Key Implementation:**

```rust
// trans/source_map.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMap {
    pub version: u32,
    pub source_file: String,
    pub generated_file: String,
    pub mappings: Vec<Mapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mapping {
    pub c_line: u32,
    pub c_col: u32,
    pub at_line: u32,
    pub at_col: u32,
    pub at_end_line: u32,
    pub at_end_col: u32,
    pub context: String,
}

impl SourceMap {
    pub fn new(source_file: &str, generated_file: &str) -> Self {
        SourceMap {
            version: 1,
            source_file: source_file.to_string(),
            generated_file: generated_file.to_string(),
            mappings: Vec::new(),
        }
    }

    pub fn add_mapping(
        &mut self,
        c_line: u32,
        c_col: u32,
        at_line: u32,
        at_col: u32,
        at_end_line: u32,
        at_end_col: u32,
        context: &str,
    ) {
        self.mappings.push(Mapping {
            c_line,
            c_col,
            at_line,
            at_col,
            at_end_line,
            at_end_col,
            context: context.to_string(),
        });
    }

    pub fn write_to_file(&self, path: &Path) -> AutoResult<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &Path) -> AutoResult<Self> {
        let json = std::fs::read_to_string(path)?;
        let map: SourceMap = serde_json::from_str(&json)?;
        Ok(map)
    }

    /// Lookup C line to find AutoLang location
    pub fn lookup(&self, c_line: u32) -> Option<&Mapping> {
        // Binary search for closest match
        let idx = self.mappings
            .binary_search_by_key(&c_line, |m| m.c_line);

        match idx {
            Ok(i) => Some(&self.mappings[i]),
            Err(0) => None,
            Err(i) => Some(&self.mappings[i - 1]),
        }
    }
}
```

**Integration with Transpiler:**
```rust
// trans/c.rs
impl Trans for CTranspiler {
    fn trans(&mut self, ast: Code, sink: &mut Sink) -> AutoResult<()> {
        // ... transpile code ...

        // Write source map
        let source_map_path = sink.path().with_extension("cmap");
        self.source_map.write_to_file(&source_map_path)?;

        Ok(())
    }
}
```

**Success Criteria:**
- Source map generates valid JSON
- Source map lookup works correctly
- Source map files written alongside C output
- Unit tests pass

---

### Phase 3: Error Formatting in C (2-3 weeks)

**Objective:** Implement pretty error printing in C

**Deliverables:**
1. Error formatter in C stdlib
2. Source snippet loading
3. Multi-line error display
4. ANSI color support

**Files to Create:**
```
stdlib/c/
├── error.h
├── error.c
├── source_map.h
└── source_map.c
```

**Key Implementation:**

```c
// error.h
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef enum {
    ERROR_LEVEL_ERROR,
    ERROR_LEVEL_WARNING,
    ERROR_LEVEL_INFO,
} ErrorLevel;

typedef struct {
    ErrorLevel level;
    const char* code;
    const char* message;
    const char* at_file;
    int at_line;
    int at_col;
    int at_end_line;
    int at_end_col;
    const char* source_snippet;
    const char* hint;
} CompilerError;

// Format error with source snippet
char* format_error(CompilerError* err, const char* source);

// Print error to stderr with colors
void print_error(CompilerError* err, const char* source);

// ANSI color codes
#define COLOR_RED     "\x1b[31m"
#define COLOR_YELLOW  "\x1b[33m"
#define COLOR_BLUE    "\x1b[34m"
#define COLOR_RESET   "\x1b[0m"
#define COLOR_BOLD    "\x1b[1m"
```

```c
// error.c
#include "error.h"

void print_error(CompilerError* err, const char* source) {
    // Print error header with color
    if (err->level == ERROR_LEVEL_ERROR) {
        fprintf(stderr, COLOR_RED "Error:" COLOR_RESET);
    } else if (err->level == ERROR_LEVEL_WARNING) {
        fprintf(stderr, COLOR_YELLOW "Warning:" COLOR_RESET);
    } else {
        fprintf(stderr, COLOR_BLUE "Info:" COLOR_RESET);
    }

    fprintf(stderr, " %s\n\n", err->code);

    // Print error message
    fprintf(stderr, COLOR_BOLD "  × %s" COLOR_RESET "\n", err->message);
    fprintf(stderr, "   ╰─▶ %s\n", err->message);

    // Print source location
    fprintf(stderr, "   ╭─[%s:%d:%d]\n",
            err->at_file,
            err->at_line,
            err->at_col);

    // Print source lines
    char* source_copy = strdup(source);
    char* line = strtok(source_copy, "\n");
    int line_num = 1;

    while (line != NULL && line_num <= err->at_end_line + 1) {
        if (line_num >= err->at_line - 1) {
            fprintf(stderr, " %d │ %s\n", line_num, line);

            // Print error indicator
            if (line_num == err->at_line) {
                fprintf(stderr, "   ·");
                for (int i = 1; i < err->at_col; i++) {
                    fprintf(stderr, " ");
                }
                for (int i = err->at_col; i < err->at_end_col; i++) {
                    fprintf(stderr, "─");
                }
                fprintf(stderr, "┬\n");

                fprintf(stderr, "   ·");
                for (int i = 1; i < err->at_col; i++) {
                    fprintf(stderr, " ");
                }
                fprintf(stderr, "╰── %s\n", err->message);
            }
        }

        line = strtok(NULL, "\n");
        line_num++;
    }

    fprintf(stderr, "   ╰────\n");

    // Print hint if available
    if (err->hint != NULL) {
        fprintf(stderr, "\n" COLOR_BLUE "💡 Hint:" COLOR_RESET " %s\n", err->hint);
    }

    free(source_copy);
}

// Example usage
void example_division_by_zero(SourceMap* map, int c_line) {
    // Look up source location
    SourceMapping* mapping = SourceMap_lookup(map, c_line);

    // Read source file
    char* source = read_source_file(mapping->at_file);

    // Create error
    CompilerError err = {
        .level = ERROR_LEVEL_ERROR,
        .code = "auto_runtime_E0301",
        .message = "Division by zero",
        .at_file = mapping->at_file,
        .at_line = mapping->at_line,
        .at_col = mapping->at_col,
        .at_end_line = mapping->at_end_line,
        .at_end_col = mapping->at_end_col,
        .source_snippet = mapping->context,
        .hint = "Check the divisor before division",
    };

    // Print beautiful error
    print_error(&err, source);

    free(source);
}
```

**Success Criteria:**
- Errors display with source snippets
- ANSI colors work correctly
- Multi-line errors handled
- Cross-platform (Windows, Linux, macOS)

---

### Phase 4: IDE Integration (Optional, 1 week)

**Objective:** Integrate with Language Server Protocol (LSP)

**Deliverables:**
1. LSP server for AutoLang
2. Go-to-definition support
3. Error highlighting
4. Source map lookup for IDE

**Files to Create:**
```
tools/
└── auto_lang_server/
    ├── main.rs
    └── lsp.rs
```

**Key Implementation:**

```rust
// Simple LSP server
use tokio::net::TcpListener;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4389").await?;

    loop {
        let (socket, _) = listener.accept().await?;

        tokio::spawn(async move {
            // Handle LSP requests
            // - textDocument/diagnostics
            // - textDocument/definition
            // - textDocument/hover
        });
    }
}
```

**Success Criteria:**
- LSP server responds to requests
- Errors shown in IDE
- Go-to-definition works

---

## 4. Testing Strategy

### 4.1 Unit Tests

**Source Map Tests:**
```rust
#[test]
fn test_source_map_lookup() {
    let mut map = SourceMap::new("test.at", "test.c");

    map.add_mapping(1, 1, 10, 5, 10, 10, "let x = 1 + 2");
    map.add_mapping(2, 1, 11, 5, 11, 15, "let y = x * 3");

    let mapping = map.lookup(1).unwrap();
    assert_eq!(mapping.at_line, 10);
}
```

### 4.2 Integration Tests

**Error Display Tests:**
```bash
# Test error output
cargo test -p auto-lang error_display

# Expected output:
# Error: auto_runtime_E0301
#   × Division by zero
#    ╭─[test.at:10:5]
# 10 │     let z = x / y
#    ·              ──┬──
#    ·                ╰── Division by zero
#    ╰────
```

---

## 5. Success Criteria

### Phase 1 (Source Tracking)
- [ ] Source tracking in transpiler
- [ ] Line directives inserted
- [ ] Unit tests pass

### Phase 2 (Source Map Generation)
- [ ] Source map generates valid JSON
- [ ] Source map lookup works
- [ ] Files written correctly

### Phase 3 (Error Formatting)
- [ ] Pretty error printing in C
- [ ] ANSI colors work
- [ ] Multi-line errors handled

### Phase 4 (IDE Integration)
- [ ] LSP server working
- [ ] IDE errors displayed
- [ ] Go-to-definition functional

### Overall
- [ ] Error quality matches Rust implementation
- [ ] Can map C errors to AutoLang source
- [ ] IDE integration works
- [ ] Performance acceptable

---

## 6. Related Documentation

- **[Plan 033]:** Self-Hosting Compiler
- **[Rust miette](https://docs.rs/miette/):** Reference for error display
- **[Source Map Spec](https://sourcemaps.info/spec.html):** Source map format
- **[LSP Spec](https://microsoft.github.io/language-server-protocol/):** Language Server Protocol

---

## 7. Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Source Tracking | 2-3 weeks | Location tracking in transpiler |
| 2. Source Map Generation | 1-2 weeks | JSON source map files |
| 3. Error Formatting | 2-3 weeks | Pretty error printing in C |
| 4. IDE Integration | 1 week | LSP server |
| **Total** | **6-9 weeks** | **Complete source mapping** |

**Critical Path:** Phase 1 → 2 → 3

**Can Start:** During Plan 033 Phase 7 (Code Generation)

---

## 8. Conclusion

This plan implements source mapping for AutoLang, enabling IDE-grade error messages that match the quality of the Rust implementation. By tracking source locations through transpilation and formatting errors in C, we provide developers with clear, actionable error messages.

**Key Benefits:**
1. **Better DX:** Clear error messages with source snippets
2. **Faster debugging:** Know exactly where errors occur
3. **IDE integration:** Language Server Protocol support
4. **Professional quality:** Matches Rust compiler's error reporting

Once complete, AutoLang will have world-class error reporting suitable for production use.
