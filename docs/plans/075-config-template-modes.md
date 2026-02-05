# Plan 075: ConfigCodegen and TemplateCodegen Implementation

**Status**: WIP (Phase 1 complete, Phase 2 pending)
**Created**: 2026-02-05
**Related**: Plan 073 (BigVM Migration), Plan 068 (BigVM Implementation)

---

## Objective

Implement **ConfigCodegen** and **TemplateCodegen** to support CONFIG and TEMPLATE execution modes in BigVM, completing the migration from the evaluator's three-mode system (SCRIPT, CONFIG, TEMPLATE) to a pure bytecode-based architecture.

**Goal**: Enable BigVM to fully replace the evaluator by supporting all three execution modes without polluting the VM with mode-aware logic.

---

## Background: Evaluator's Three Modes

The current evaluator ([eval.rs](d:\autostack\auto-lang\crates\auto-lang\src\eval.rs#L34-L38)) supports three execution modes:

```rust
pub enum EvalMode {
    SCRIPT,   // Normal evaluation: return last value, auto-call main()
    CONFIG,   // Accumulate all statements into single object/array structure
    TEMPLATE, // Convert all statements to strings, join with "\n", filter nil
}
```

### Mode Behaviors:

**SCRIPT Mode** (default):
- Executes all statements sequentially
- Returns the last statement's value
- Automatically calls `main()` if defined
- Used for: scripts, programs, tests

**CONFIG Mode**:
- Accumulates all top-level declarations into a single structure
- Merges objects/arrays with same names
- Returns a unified configuration object
- Used for: configuration files, structured data

**TEMPLATE Mode**:
- Executes all statements
- Converts each result to string
- Joins with "\n" separator
- Filters out nil values
- Used for: text generation, templating, file generation

### Examples:

```auto
// SCRIPT mode (script.at)
fn add(a int, b int) int { a + b }
let result = add(1, 2)
say(result)
// Returns: Value::Int(3)

// CONFIG mode (config.at)
server.host = "localhost"
server.port = 8080
database.name = "mydb"
database.pool = 10
// Returns: Value::Node({
//   server: { host: "localhost", port: 8080 },
//   database: { name: "mydb", pool: 10 }
// })

// TEMPLATE mode (template.at)
Hello, $name!
You have ${count} messages.
// Returns: Value::Str("Hello, World!\nYou have 5 messages.")
```

---

## Design Decision: Codegen-Level Implementation

### ✅ Chosen Approach: Compiler-Based Mode Handling

**DO NOT** add mode awareness to BigVM. Instead, implement three separate codegen strategies:

1. **Codegen** (existing) → SCRIPT mode
2. **ConfigCodegen** (new) → CONFIG mode
3. **TemplateCodegen** (new) → TEMPLATE mode

**Key Principle**: The VM remains mode-agnostic. All mode-specific logic lives in the compiler.

### Why This Approach?

| Aspect | VM-Level Modes | Codegen-Level Modes ✅ |
|--------|---------------|------------------------|
| **VM Complexity** | ❌ Mode checks everywhere | ✅ Zero mode awareness |
| **Opcode Space** | ❌ Needs mode-specific opcodes | ✅ Uses existing opcodes |
| **Performance** | ❌ Runtime branching overhead | ✅ Compile-time optimization |
| **Maintainability** | ❌ 3 code paths in VM engine | ✅ Separate in compiler |
| **Testing** | ❌ Test all modes × all opcodes | ✅ Test modes independently |
| **Extensibility** | ❌ Adding modes requires VM changes | ✅ Adding modes is just new codegen |

### Architecture Diagram:

```
┌─────────────────────────────────────────────────────────┐
│                    AutoLang Source                      │
│                  (config.at, template.at)                │
└─────────────────────────┬───────────────────────────────┘
                          │
                          ▼
         ┌────────────────────────────────────────┐
         │         Parser (mode-agnostic)          │
         │         Produces AST (Code)             │
         └────────────────────────────────────────┘
                          │
            ┌─────────────┼─────────────┐
            │             │             │
            ▼             ▼             ▼
    ┌───────────┐ ┌───────────┐ ┌──────────────┐
    │ Codegen   │ │ConfigCodeg│ │TemplateCodeg │
    │ (SCRIPT)  │ │  (CONFIG) │ │  (TEMPLATE)  │
    └─────┬─────┘ └─────┬─────┘ └──────┬───────┘
          │             │              │
          │             │              │
          ▼             ▼              ▼
    ┌────────────────────────────────────────────┐
    │           BigVM (mode-agnostic)            │
    │      Executes bytecode, returns Value      │
    └────────────────────────────────────────────┘
                          │
                          ▼
              ┌─────────────────────┐
              │  Post-Processing    │
              │  (format output)    │
              └─────────────────────┘
```

---

## Phase 1: ConfigCodegen Implementation

### 1.1 Overview

ConfigCodegen transforms configuration files into bytecode that builds a unified object structure.

### 1.2 Transformation Strategy

**Input Config:**
```auto
// database.at
server.host = "localhost"
server.port = 5432
database.name = "mydb"
database.pool = 10
debug = true
```

**Generated Bytecode:**
```rust
// Create root object
CREATE_OBJ                           // obj_id = 0

// Create nested server object
DUP                                 // Duplicate root for chaining
CREATE_OBJ                          // server object
LOAD_STR "localhost"
LOAD_STR "host"
SET_FIELD                           // server.host = "localhost"
CONST_I32 5432
LOAD_STR "port"
SET_FIELD                           // server.port = 5432
LOAD_STR "server"
SET_FIELD_NESTED                    // root.server = {host, port}

// Create nested database object
DUP
CREATE_OBJ                          // database object
LOAD_STR "mydb"
LOAD_STR "name"
SET_FIELD                           // database.name = "mydb"
CONST_I32 10
LOAD_STR "pool"
SET_FIELD                           // database.pool = 10
LOAD_STR "database"
SET_FIELD_NESTED                    // root.database = {name, pool}

// Set top-level field
CONST_1                              // true
LOAD_STR "debug"
SET_FIELD                           // root.debug = true

RET                                 // Return root object
```

### 1.3 ConfigCodegen Structure

**New file:** `crates/auto-lang/src/vm/config_codegen.rs`

```rust
use crate::ast::{Code, Stmt, Store, Expr};
use crate::vm::codegen::{Codegen, OpCode};
use crate::error::AutoResult;

pub struct ConfigCodegen {
    base: Codegen,
    /// Track nesting depth for nested object creation
    nesting_stack: Vec<Vec<String>>,
    /// Accumulate field paths (e.g., ["server", "host"])
    field_accumulator: Vec<Vec<String>>,
}

impl ConfigCodegen {
    pub fn new() -> Self {
        Self {
            base: Codegen::new(),
            nesting_stack: Vec::new(),
            field_accumulator: Vec::new(),
        }
    }

    /// Compile config file to bytecode
    pub fn compile_config(&mut self, code: &Code) -> AutoResult<()> {
        // Create root object
        self.base.emit(OpCode::CREATE_OBJ);

        // Process each statement
        for stmt in &code.stmts {
            self.compile_config_stmt(stmt)?;
        }

        // Return the accumulated config object
        self.base.emit(OpCode::RET);

        Ok(())
    }

    fn compile_config_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            // Parse field assignments: server.host = "localhost"
            Stmt::Store(store) => {
                self.compile_store_to_field(store)?;
            }
            // Evaluate expressions and add to config
            Stmt::Expr(expr) => {
                self.compile_expr_to_field(expr)?;
            }
            _ => {
                return Err(auto_lang::AutoError::Msg(
                    format!("Config mode does not support statement: {:?}", stmt)
                ));
            }
        }
        Ok(())
    }

    fn compile_store_to_field(&mut self, store: &Store) -> AutoResult<()> {
        // Extract field path from store.name
        // e.g., "server.host" -> ["server", "host"]
        let path = self.parse_field_path(&store.name);

        // Compile the value expression
        self.base.compile_expr(&store.expr)?;

        // Set nested field
        self.set_nested_field(&path)?;

        Ok(())
    }

    fn parse_field_path(&self, name: &str) -> Vec<String> {
        // "server.host" -> ["server", "host"]
        // "database" -> ["database"]
        name.split('.').map(|s| s.to_string()).collect()
    }

    fn set_nested_field(&mut self, path: &[String]) -> AutoResult<()> {
        match path.len() {
            1 => {
                // Top-level field: debug = true
                self.base.emit(OpCode::LOAD_STR);
                self.base.emit_string(&path[0]);
                self.base.emit(OpCode::SET_FIELD);
            }
            2 => {
                // Nested field: server.host = "localhost"
                self.base.emit(OpCode::LOAD_STR);
                self.base.emit_string(&path[0]); // "server"
                self.base.emit(OpCode::LOAD_STR);
                self.base.emit_string(&path[1]); // "host"
                self.base.emit(OpCode::SET_FIELD_NESTED);
            }
            _ => {
                // Deep nesting: server.config.max_conn = 100
                // Need to create intermediate objects
                for (i, part) in path.iter().enumerate() {
                    self.base.emit(OpCode::LOAD_STR);
                    self.base.emit_string(part);
                    if i < path.len() - 1 {
                        // Create intermediate object
                        self.base.emit(OpCode::CREATE_OBJ);
                    }
                }
                self.base.emit(OpCode::SET_FIELD_DEEP);
            }
        }
        Ok(())
    }

    fn compile_expr_to_field(&mut self, expr: &Expr) -> AutoResult<()> {
        // For expressions without explicit field names,
        // use hash of expression or assign to "_exprN"
        let field_name = format!("_expr{}", self.field_accumulator.len());

        self.base.compile_expr(expr)?;
        self.base.emit(OpCode::LOAD_STR);
        self.base.emit_string(&field_name);
        self.base.emit(OpCode::SET_FIELD);

        self.field_accumulator.push(vec![field_name]);
        Ok(())
    }

    pub fn finish(self, name: String) -> crate::vm::Module {
        self.base.finish(name)
    }
}
```

### 1.4 New Opcodes (if needed)

**Option 1: Use existing opcodes** (preferred)
- `CREATE_OBJ` - Create object
- `SET_FIELD` - Set field on object
- `DUP` - Duplicate for chaining

**Option 2: Add convenience opcodes**
```rust
SET_FIELD_NESTED = 0x40,  // Set nested field: obj.parent.child = value
SET_FIELD_DEEP = 0x41,    // Set deeply nested field (3+ levels)
```

**Recommendation**: Start with existing opcodes, add convenience opcodes only if performance is critical.

### 1.5 Testing

```rust
// crates/auto-lang/src/vm/config_tests.rs

#[test]
fn test_config_simple_fields() {
    let source = r#"
host = "localhost"
port = 8080
debug = true
"#;

    let mut configgen = ConfigCodegen::new();
    let code = parse(source);
    configgen.compile_config(&code).unwrap();

    let module = configgen.finish("test");
    let vm = BigVM::new();
    let result = vm.run(&module).unwrap();

    // Should return object: {host: "localhost", port: 8080, debug: true}
    assert!(matches!(result, Value::Instance(..)));
}

#[test]
fn test_config_nested_fields() {
    let source = r#"
server.host = "localhost"
server.port = 5432
database.name = "mydb"
"#;

    // Should return: {server: {host: "localhost", port: 5432}, database: {name: "mydb"}}
}

#[test]
fn test_config_with_expressions() {
    let source = r#"
max_connections = 10
timeout = max_connections * 2
"#;

    // Should evaluate expressions before storing
}
```

---

## Phase 2: TemplateCodegen Implementation

### 2.1 Overview

TemplateCodegen transforms template files into bytecode that builds strings by concatenating evaluated expressions.

### 2.2 Transformation Strategy

**Input Template:**
```auto
// email.at
Hello, $name!
You have ${count} messages.
Total: ${unread + read}
```

**Generated Bytecode:**
```rust
// Constant part
LOAD_STR "Hello, "

// Variable interpolation
LOAD_VAR "name"
TO_STR                                   // Convert to string
STR_CAT                                  // Concatenate

LOAD_STR "!\nYou have "
STR_CAT

LOAD_VAR "count"
TO_STR
STR_CAT

LOAD_STR " messages.\nTotal: "
STR_CAT

// Expression evaluation
LOAD_VAR "unread"
LOAD_VAR "read"
ADD
TO_STR
STR_CAT

// Check if nil before including (for optional fields)
DUP
IS_NIL
JMP_IF_NZ skip_nil                       // Skip if nil
LOAD_STR "\n"
STR_CAT
skip_nil:

RET                                      // Return final string
```

### 2.3 TemplateCodegen Structure

**New file:** `crates/auto-lang/src/vm/template_codegen.rs`

```rust
use crate::ast::{Code, Stmt, Expr};
use crate::vm::codegen::Codegen;
use crate::error::AutoResult;

pub struct TemplateCodegen {
    base: Codegen,
    separator: String,  // "\n" by default
    filter_nil: bool,   // true by default
}

impl TemplateCodegen {
    pub fn new() -> Self {
        Self {
            base: Codegen::new(),
            separator: "\n".to_string(),
            filter_nil: true,
        }
    }

    pub fn with_separator(mut self, sep: &str) -> Self {
        self.separator = sep.to_string();
        self
    }

    pub fn filter_nil(mut self, filter: bool) -> Self {
        self.filter_nil = filter;
        self
    }

    pub fn compile_template(&mut self, code: &Code) -> AutoResult<()> {
        let stmt_count = code.stmts.len();

        for (i, stmt) in code.stmts.iter().enumerate() {
            // Compile statement
            self.compile_template_stmt(stmt)?;

            // Convert to string
            self.emit_to_str()?;

            // Add separator (except for last statement)
            if i < stmt_count - 1 {
                self.emit_separator()?;
            }

            // Filter nil if enabled
            if self.filter_nil {
                self.emit_nil_check()?;
            }
        }

        // Return final string
        self.base.emit(OpCode::RET);

        Ok(())
    }

    fn compile_template_stmt(&mut self, stmt: &Stmt) -> AutoResult<()> {
        match stmt {
            Stmt::Expr(expr) => {
                self.base.compile_expr(expr)?;
            }
            Stmt::Store(store) => {
                // For stores, compile and ignore result
                self.base.compile_stmt(stmt)?;
                self.base.emit(OpCode::POP);
            }
            _ => {
                self.base.compile_stmt(stmt)?;
            }
        }
        Ok(())
    }

    fn emit_to_str(&mut self) -> AutoResult<()> {
        // Convert top of stack to string
        self.base.emit(OpCode::TO_STR);
        Ok(())
    }

    fn emit_separator(&mut self) -> AutoResult<()> {
        // Load separator string
        self.base.emit(OpCode::LOAD_STR);
        self.base.emit_string(&self.separator);

        // Concatenate
        self.base.emit(OpCode::STR_CAT);

        Ok(())
    }

    fn emit_nil_check(&mut self) -> AutoResult<()> {
        // Duplicate top value
        self.base.emit(OpCode::DUP);

        // Check if nil
        self.base.emit(OpCode::IS_NIL);

        // Skip next separator if nil
        let skip_placeholder = self.base.emit_placeholder_i16();

        // If not nil, add separator
        self.emit_separator()?;

        // Patch jump
        self.base.patch_jump(skip_placeholder);

        Ok(())
    }

    pub fn finish(self, name: String) -> crate::vm::Module {
        self.base.finish(name)
    }
}
```

### 2.4 New Opcodes

**String Manipulation Opcodes:**
```rust
TO_STR = 0x38,      // Convert any value to string
IS_NIL = 0x39,      // Check if value is nil
STR_CAT = 0x3A,     // Concatenate two strings (optimized)
```

**Implementation in engine.rs:**
```rust
OpCode::TO_STR => {
    let val = task.ram.pop_i32();
    let str_val = format!("{}", val);  // Or proper Value::Str conversion
    task.ram.push_str(str_val);
}

OpCode::IS_NIL => {
    let val = task.ram.pop_i32();
    task.ram.push_i32(if val == -1 { 1 } else { 0 });
}

OpCode::STR_CAT => {
    let right = task.ram.pop_str();
    let left = task.ram.pop_str();
    let result = format!("{}{}", left, right);
    task.ram.push_str(result);
}
```

### 2.5 Testing

```rust
// crates/auto-lang/src/vm/template_tests.rs

#[test]
fn test_template_simple() {
    let source = r#"
"Hello, World!"
"Goodbye!"
"#;

    let mut tgen = TemplateCodegen::new();
    let code = parse(source);
    tgen.compile_template(&code).unwrap();

    let module = tgen.finish("test");
    let vm = BigVM::new();
    let result = vm.run(&module).unwrap();

    // Should return: "Hello, World!\nGoodbye!"
    assert_eq!(result, Value::Str("Hello, World!\nGoodbye!"));
}

#[test]
fn test_template_with_interpolation() {
    let source = r#"
name = "World"
"Hello, $name!"
"#;

    // Should return: "World\nHello, World!"
}

#[test]
fn test_template_nil_filtering() {
    let source = r#"
"First"
nil
"Third"
"#;

    // Should return: "First\nThird" (nil filtered out)
}
```

---

## Phase 3: Integration & API Design

### 3.1 Unified Compilation API

**File:** `crates/auto-lang/src/lib.rs`

```rust
pub enum CompileMode {
    Script,
    Config,
    Template,
}

pub fn run_with_mode(source: &str, mode: CompileMode) -> AutoResult<String> {
    let mut parser = Parser::from(source);
    let code = parser.parse()?;

    let module = match mode {
        CompileMode::Script => {
            let mut codegen = Codegen::new();
            codegen.compile(&code)?;
            codegen.finish("script")
        }
        CompileMode::Config => {
            let mut configgen = ConfigCodegen::new();
            configgen.compile_config(&code)?;
            configgen.finish("config")
        }
        CompileMode::Template => {
            let mut tgen = TemplateCodegen::new();
            tgen.compile_template(&code)?;
            tgen.finish("template")
        }
    };

    let vm = BigVM::new();
    let result = vm.run(&module)?;

    Ok(format!("{:?}", result))
}
```

### 3.2 Auto-Detection from File Extension

```rust
pub fn run_file_with_auto_mode(path: &Path) -> AutoResult<String> {
    let source = std::fs::read_to_string(path)?;
    let mode = detect_mode_from_extension(path)?;
    run_with_mode(&source, mode)
}

fn detect_mode_from_extension(path: &Path) -> AutoResult<CompileMode> {
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    Ok(match ext {
        "config.at" => CompileMode::Config,
        "template.at" => CompileMode::Template,
        _ => CompileMode::Script,
    })
}
```

### 3.3 Shell Integration

**File:** `crates/auto-shell/src/main.rs`

```rust
// auto-shell commands
:mode script    // Switch to script mode
:mode config    // Switch to config mode
:mode template  // Switch to template mode

// Or auto-detect from file extension
auto run config.config.at      // Uses ConfigCodegen
auto run email.template.at     // Uses TemplateCodegen
auto run script.at             // Uses Codegen
```

---

## Phase 4: Implementation Roadmap

### Week 1: ConfigCodegen (3-4 days) ✅ COMPLETED (2026-02-06)
- [x] Day 1: Create `config_codegen.rs`, basic structure
- [x] Day 2: Implement field path parsing and nested object creation
- [x] Day 3: Implement expression evaluation and field assignment
- [x] Day 4: Testing and debugging

**Implementation Summary**:
- Created `crates/auto-lang/src/vm/config_codegen.rs` with ConfigCodegen struct
- Collects all field assignments and creates single object using CREATE_OBJ
- Added SET_FIELD opcode (0x2A) to opcode.rs for future use
- 4 unit tests passing (simple_fields, nested_fields, with_expressions, empty_config)
- **Known Limitation**: Dotted identifiers like `server.host = "localhost"` not supported yet
  - Parser treats `server.host` as binary expression with `.` operator
  - Workaround: Use underscored names (`server_host = "localhost"`) or object literals
  - TODO: Either modify parser or add custom config file parser for dotted paths

### Week 2: TemplateCodegen (3-4 days) ⏸️ NOT STARTED
- [ ] Day 1: Add TO_STR, IS_NIL, STR_CAT opcodes to VM
- [ ] Day 2: Create `template_codegen.rs`, basic structure
- [ ] Day 3: Implement string concatenation and nil filtering
- [ ] Day 4: Testing and debugging

### Week 3: Integration & Testing (3-4 days) ⏸️ NOT STARTED
- [ ] Day 1: Unified compilation API, mode detection
- [ ] Day 2: Shell integration, auto-detection
- [ ] Day 3-4: Comprehensive testing, documentation

**Total Effort**: 9-12 days (2-3 weeks)
**Progress**: Phase 1 complete (~33% done)

---

## Phase 5: Testing Strategy

### 5.1 Unit Tests

**ConfigCodegen Tests** (`config_tests.rs`):
- Simple field assignment
- Nested field assignment
- Expression evaluation
- Array values
- Object merging

**TemplateCodegen Tests** (`template_tests.rs`):
- Simple string concatenation
- Variable interpolation
- Expression evaluation
- Nil filtering
- Custom separators

### 5.2 Integration Tests

**Mode Comparison Tests**:
- Run same file with evaluator and BigVM in all three modes
- Verify outputs match

**Existing Test Migration**:
- Migrate config tests from evaluator to BigVM
- Migrate template tests from evaluator to BigVM

### 5.3 Performance Tests

Benchmark evaluator vs BigVM for each mode:
- Config mode: Large configuration files (1000+ fields)
- Template mode: Long templates with many interpolations

---

## Phase 6: Success Criteria

### 6.1 Functional Requirements
- ✅ ConfigCodegen compiles config files to equivalent bytecode
- ✅ TemplateCodegen compiles template files to equivalent bytecode
- ✅ Both modes produce identical results to evaluator
- ✅ Zero VM changes for mode awareness
- ✅ All existing config/template tests pass with BigVM

### 6.2 Performance Requirements
- Config mode: Performance within 2x of evaluator
- Template mode: Performance within 2x of evaluator
- No runtime mode checking overhead

### 6.3 Code Quality Requirements
- Zero compilation warnings
- 80%+ test coverage for new code
- Comprehensive documentation

---

## Risks and Mitigations

### Risk 1: Opcode Bloat
**Concern**: Adding too many convenience opcodes for config/template

**Mitigation**: Start with existing opcodes only. Add convenience opcodes (SET_FIELD_NESTED, STR_CAT) only if profiling shows they're needed.

### Risk 2: Complexity in Field Path Parsing
**Concern**: Complex nested field paths (a.b.c.d) are hard to compile

**Mitigation**: Implement 2-level nesting first (a.b), add deep nesting support only if needed. Use iterative object creation.

### Risk 3: String Concatenation Performance
**Concern**: Template mode with many concatenations is slow

**Mitigation**: Use StringBuilder pattern or optimize STR_CAT opcode. Profile and optimize bottlenecks.

### Risk 4: Behavior Mismatch with Evaluator
**Concern**: Subtle differences in mode behavior break compatibility

**Mitigation**: Comprehensive comparison tests. Test edge cases (nil handling, empty strings, nested objects).

---

## Open Questions

1. **Q**: Should we add SET_FIELD_NESTED opcode or use existing opcodes?
   **A**: Start with existing, add only if profiling shows benefit.

2. **Q**: How to handle circular references in config mode?
   **A**: Document that circular references are not supported in config mode.

3. **Q**: Should template mode support custom separators other than "\n"?
   **A**: Yes, add `TemplateCodegen::with_separator()` API.

4. **Q**: How to handle large template files (10000+ lines)?
   **A**: Optimize string concatenation, consider streaming output.

---

## Related Documents

- [Plan 068: AutoVM (BigVM) Implementation](068-autovm-bigvm.md)
- [Plan 073: BigVM Migration Roadmap](073-bigvm-migration-roadmap.md)
- [Evaluator: Three Mode System](d:\autostack\auto-lang\crates\auto-lang\src\eval.rs#L34-L38)

---

**Document Updated**: 2026-02-05
**Next Steps**: Implement ConfigCodegen first (lower complexity, higher value)
