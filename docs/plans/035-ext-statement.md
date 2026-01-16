# ext Statement Implementation Plan

## Implementation Status: 📋 **PLANNED**

**Priority:** HIGH - Required for OOP-style API design
**Dependencies:** None (builds on existing method system)
**Estimated Duration:** 2-3 weeks
**Complexity:** Medium

**Objectives:**
- ✅ Allow adding methods to built-in types (str, cstr, int, etc.)
- ✅ Support both static methods and instance methods
- ✅ Maintain Auto's `self` and `.prop` shorthand semantics
- ✅ Integrate with existing method lookup and TypeInfoStore
- ✅ Enable migration from standalone functions to methods

---

## Executive Summary

**Problem:** Auto's built-in types (str, cstr, int, bool, etc.) are defined in the Rust compiler implementation and cannot be directly modified to add methods. Currently, Plan 025 string functions are standalone functions (e.g., `str_len(s)`, `str_new("hello", 5)`) rather than methods.

**Solution:** Implement an `ext` statement (similar to Rust's `impl`) that allows adding methods to types AFTER their initial definition. This enables OOP-style APIs like `s.len()`, `s.append(other)` instead of `str_len(s)`, `str_append(s, other)`.

**Key Design Decisions:**
1. **Keyword**: `ext` (not `impl`) to distinguish from Rust
2. **Syntax**: Similar to Rust's `impl`, but with Auto's `self` convention
3. **Static vs Instance**: Methods with first parameter named `self` are instance methods; others are static
4. **Integration**: Extends existing `Fn.parent` field and TypeInfoStore mechanism

---

## Current State Analysis

### Existing Method System

**From previous exploration:**

1. **Method Call Syntax** (parser.rs:3509-3652):
```auto
// Method call: obj.method(args)
let result = s.append("world")

// Parsed as:
Expr::Call {
    name: Expr::Bina(left, Dot, right),  // s.append
    args: [...],
    ret: Type::Unknown
}
```

2. **Method Definition in Types** (parser.rs:2502-2515):
```auto
type Point {
    x int
    y int

    // Methods defined INSIDE type
    fn distance(self, other Point) int {
        let dx = .x - other.x  // .prop shorthand for self.prop
        let dy = .y - other.y
        return sqrt(dx*dx + dy*dy)
    }
}
```

3. **Auto-defined `self`** (parser.rs:2502-2515):
```rust
// Parser automatically defines `self` in method scope
if let Some(parent) = &fn_node.parent {
    // Define self variable pointing to the instance
    self.universe.borrow_mut().define_var("self".into(), instance.clone());
}
```

4. **`.prop` Shorthand** (parser.rs:749-757):
```rust
// Convert .prop to self.prop
if token.kind == TokenKind::Dot {
    if !is_self_defined {
        // Error: .prop can only be used in methods
    }
    // Transform .x to self.x
}
```

5. **Method Resolution Order** (eval.rs:1642-1703):
```rust
// Lookup order: Type methods → Parent methods → Has methods → Delegated methods
fn lookup_method(&self, type_name: &Name, method_name: &Name) -> Option<&Fn> {
    // 1. Check type's own methods
    // 2. Check parent (is) methods
    // 3. Check has composition methods
    // 4. Check delegated trait methods
}
```

### TypeInfoStore Mechanism

**From value.rs and builtin.rs:**

Built-in types use `TypeInfoStore` to register methods that cannot be declared in type definitions:

```rust
// crates/auto-val/src/value.rs
pub struct TypeInfoStore {
    // Maps type name → method functions
    methods: HashMap<AutoStr, HashMap<AutoStr, Value>>,
}

// Example: Str type methods
impl TypeInfoStore {
    pub fn register_str_methods(&mut self) {
        let mut methods = HashMap::new();

        // Register method: s.len()
        methods.insert("len".into(), Value::ExtFn(ExtFn {
            fun: str_len_method,
            name: "len".into()
        }));

        self.str_methods = methods;
    }
}
```

### Current Limitations

**Problem:** No mechanism to add methods AFTER type definition

```auto
// This WORKS: Methods inside type definition
type Point {
    x int
    fn distance() int {    // ← No self parameter!
        return sqrt(.x * .x + .y * .y)  // ← .prop shorthand
    }
}

// This DOESN'T WORK: No ext statement yet
ext Point {      // ← Error: Unknown keyword 'ext'
    fn area() int {
        return .x * .y
    }
}

// Built-in types CANNOT be modified at all
// str, cstr, int, bool are defined in Rust!
```

---

## Proposed Syntax

### Syntax Design

**Patterned after Rust's `impl`, adapted for Auto's method conventions:**

```auto
// ===== Instance Methods =====

ext str {
    // Instance method: s.len()
    // Note: No 'self' parameter - it's implicit!
    fn len() int {
        return .size  // .prop accesses self.prop
    }

    // Instance method: s.append(other)
    fn append(other str) {
        str_append(mut self, other)
    }

    // Instance method: s.slice(start, end)
    fn slice(start int, end int) str_slice {
        return as_slice(self)[start..end]
    }
}

// ===== Static Methods =====

// For static methods, we need a different syntax
// Options:
// 1. Use 'static' keyword (NEW)
// 2. Separate ext block with 'static ext'
// 3. Type-level functions (outside ext)

ext str {
    // Static constructor: str.new("hello")
    // Option: Add 'static' keyword
    static fn new(data *char, size int) str {
        return str_new(data, size)
    }

    // Static factory: str.from_cstr(cstr)
    static fn from_cstr(cs cstr) str {
        return to_str(cs)
    }
}

// ===== Usage =====

fn main() {
    // Static methods: Type.method()
    let s = str.new("hello", 5)
    let s2 = str.from_cstr(my_cstr)

    // Instance methods: instance.method()
    let len = s.len()
    s.append(" world")
    let slice = s.slice(0, 3)
}
```

### Key Differences from Rust

| Feature | Rust | Auto |
|---------|------|------|
| Keyword | `impl` | `ext` |
| Self Declaration | `&self`, `&mut self` in params | **Implicit** - not in params |
| Self Reference | `self.x`, `self.method()` | `.x`, `.method()` (shorthand) OR `self.x` |
| Static Syntax | `impl Type { fn method() }` | `ext Type { static fn method() }` |
| Self Binding | Reference (`&self`) | Value binding (move semantics) |

---

## Implementation Strategy

### Phase 1: AST Structure (1 day)

**File**: `crates/auto-lang/src/ast/stmt.rs`

Add new statement type:

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    // ... existing variants ...

    /// Type extension statement (like Rust's impl)
    /// ext str { fn len(self) int { ... } }
    Ext {
        target: Name,              // Type being extended (e.g., "str")
        methods: Vec<Fn>,          // Methods to add
        span: SourceSpan,
    },
}
```

**Rationale**:
- Reuses existing `Fn` structure (no new types needed)
- `Fn.parent` field distinguishes instance vs static methods
- Simple, consistent with existing AST design

**Success Criteria**:
- [ ] `Stmt::Ext` compiles
- [ ] Repr/Debug work correctly
- [ ] Unit tests for AST construction

---

### Phase 2: Parser Implementation (2-3 days)

**Files**: `crates/auto-lang/src/lexer.rs`, `crates/auto-lang/src/parser.rs`

#### 2.1 Add `ext` and `static` keywords to lexer

**File**: `crates/auto-lang/src/lexer.rs`

```rust
// Add to TokenKind enum
pub enum TokenKind {
    // ... existing tokens ...
    Ext,    // 'ext' keyword for type extensions
    Static, // 'static' keyword for static methods
}
```

#### 2.2 Add `parse_ext()` method

```rust
/// Parse ext statement: ext Type { methods... }
fn parse_ext(&mut self) -> AutoResult<Stmt> {
    let start_pos = self.cur.pos;

    // Expect 'ext' keyword
    self.expect_keyword("ext")?;

    // Parse target type name
    let target = self.parse_name()?;

    // Expect opening brace
    self.expect(TokenKind::LBrace)?;

    // Parse methods
    let mut methods = Vec::new();
    while !self.is_kind(TokenKind::RBrace) && !self.is_kind(TokenKind::Eof) {
        // Parse fn declaration
        let fn_node = self.parse_fn()?;

        // Set parent to target type
        fn_node.parent = Some(target.clone());

        methods.push(fn_node);
    }

    // Expect closing brace
    self.expect(TokenKind::RBrace)?;

    Ok(Stmt::Ext {
        target,
        methods,
        span: pos_to_span(start_pos),
    })
}
```

#### 2.2 Integrate into `parse_stmt()`

```rust
pub fn parse_stmt(&mut self) -> AutoResult<Stmt> {
    match self.cur.kind {
        // ... existing cases ...

        TokenKind::Ext => self.parse_ext(),  // NEW

        _ => self.parse_expr_stmt(),
    }
}
```

**Success Criteria**:
- [ ] `TokenKind::Ext` and `TokenKind::Static` added
- [ ] `parse_ext()` parses simple ext statements
- [ ] Error on missing target type
- [ ] Error on missing braces
- [ ] Methods have `parent` field set correctly
- [ ] `is_static` field correctly parsed
- [ ] Integration with `parse_stmt()` works

---

### Phase 3: Evaluator Integration (3-4 days)

**File**: `crates/auto-lang/src/eval.rs`

#### 3.1 Add `eval_ext()` method

```rust
fn eval_ext(&mut self, ext: &Ext) -> Value {
    // Lookup target type in universe
    let type_decl = match self.universe.borrow().lookup_type(&ext.target) {
        Some(decl) => decl,
        None => {
            // Built-in type - register methods in TypeInfoStore
            for method in &ext.methods {
                self.register_builtin_method(&ext.target, method);
            }
            return Value::Void;
        }
    };

    // User-defined type - add methods to TypeDecl
    let mut universe = self.universe.borrow_mut();
    let type_decl_mut = universe.lookup_type_mut(&ext.target).unwrap();

    for method in &ext.methods {
        // Check for duplicate method names
        if type_decl_mut.methods.iter().any(|m| m.name == method.name) {
            eprintln!("Warning: Method {} already defined on type {}",
                method.name, ext.target);
            continue;
        }

        // Add method to type
        type_decl_mut.methods.push(method.clone());
    }

    Value::Void
}

/// Register method for built-in type
fn register_builtin_method(&mut self, type_name: &Name, method: &Fn) {
    // Create ExtFn wrapper
    let ext_fn = Value::ExtFn(ExtFn {
        fun: self.create_method_wrapper(type_name, method),
        name: method.name.clone(),
    });

    // Register in TypeInfoStore
    let mut universe = self.universe.borrow_mut();
    universe.register_method(type_name.clone(), method.name.clone(), ext_fn);
}
```

#### 3.2 Method Wrapper Generation

```rust
/// Create wrapper that converts method call to function call
fn create_method_wrapper(&self, type_name: &Name, method: &Fn) -> ExtFnFunc {
    let method_clone = method.clone();
    let type_name_clone = type_name.clone();

    Box::new(move |args: &Args| -> Value {
        // Method implementation here
        // This is complex - need to evaluate method body with self bound

        // For now, store method for later lookup
        // Actual implementation in Phase 3.3
        Value::Void
    })
}
```

**Alternative Approach (Simpler)**:

Instead of generating wrappers, extend existing method lookup to check `ext` statements:

```rust
// In eval_call() method lookup
fn lookup_method_with_ext(&self, type_name: &Name, method_name: &Name) -> Option<Value> {
    // 1. Check type's own methods (existing)
    // 2. Check ext statements (NEW)
    for ext_stmt in &self.ext_statements {
        if ext_stmt.target == *type_name {
            for method in &ext_stmt.methods {
                if method.name == *method_name {
                    // Return closure that captures method body
                    return Some(self.create_method_closure(method));
                }
            }
        }
    }
    // 3. Check TypeInfoStore (existing built-in methods)
    None
}
```

**Success Criteria**:
- [ ] `eval_ext()` executes without errors
- [ ] User-defined types get methods added
- [ ] Built-in types register methods correctly
- [ ] Method lookup finds ext methods
- [ ] Duplicate method detection works

---

### Phase 4: Static vs Instance Method Detection (1 day)

**Files**: `crates/auto-lang/src/ast/fun.rs`, `crates/auto-lang/src/parser.rs`, `crates/auto-lang/src/eval.rs`

#### 4.1 Add `is_static` field to Fn struct

**File**: `crates/auto-lang/src/ast/fun.rs`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Fn {
    pub name: Name,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Expr,
    pub parent: Option<Name>,  // None = global function, Some(type) = method
    pub is_static: bool,       // NEW: true = static method, false = instance method
    pub span: SourceSpan,
}
```

#### 4.2 Parse `static` keyword

**File**: `crates/auto-lang/src/parser.rs`

```rust
fn parse_fn(&mut self) -> AutoResult<Fn> {
    let start_pos = self.cur.pos;

    // Check for 'static' keyword
    let is_static = self.is_keyword("static");
    if is_static {
        self.next(); // consume 'static'
    }

    self.expect_keyword("fn")?;
    let name = self.parse_name()?;
    // ... rest of parsing ...

    Ok(Fn {
        name,
        params,
        ret,
        body,
        parent,  // Set by ext statement
        is_static,
        span: pos_to_span(start_pos),
    })
}
```

#### 4.3 Distinguish static vs instance in eval_call

**File**: `crates/auto-lang/src/eval.rs`

```rust
// In eval_call():
fn eval_call(&mut self, call: &Call) -> Value {
    // Check if method call (obj.method())
    if let Expr::Bina(left, Op::Dot, right) = &*call.name {
        let instance = self.eval_expr(left)?;

        // Lookup method
        let method = self.lookup_method(&instance.get_type_name(), right)?;

        // Check if static or instance method
        if method.is_static {
            // Static method: Type.method(args)
            // Don't bind self, call with all args
            return self.eval_fn_call(&method, call.args.clone());
        } else {
            // Instance method: instance.method(args)
            // Bind self to instance
            self.universe.borrow_mut().define_var("self".into(), instance);
            return self.eval_fn_call(&method, call.args.clone());
        }
    }

    // Regular function call
    // ...
}
```

**Success Criteria**:
- [ ] `Fn.is_static` field added
- [ ] Parser recognizes `static fn` syntax
- [ ] Static methods don't bind `self` when called
- [ ] Instance methods automatically bind `self`
- [ ] `Type.static_method()` and `instance.method()` both work
- [ ] Instance methods called as `instance.method()`

---

### Phase 5: Testing & Migration (3-4 days)

**File**: `crates/auto-lang/test/a2c/035_ext_statement/`

#### 5.1 Basic Functionality Tests

**Test 1: Simple instance method**
```auto
// 035_ext_instance_method.at
ext int {
    fn double() int {
        return self * 2  // self is implicit
    }
}

fn main() {
    let x = 21
    print(x.double())  // Expected: 42
}
```

**Test 2: Static method**
```auto
// 035_ext_static_method.at
ext int {
    static fn from_str(s str) int {
        return parse_int(s)
    }
}

fn main() {
    let x = int.from_str("42")
    print(x)  // Expected: 42
}
```

**Test 3: Built-in type extension**
```auto
// 035_ext_builtin_type.at
ext str {
    fn len() int {
        return .size  // .prop accesses self.prop
    }

    static fn new(data *char, size int) str {
        return str_new(data, size)
    }
}

fn main() {
    let s = str.new("hello", 5)
    print(s.len())  // Expected: 5
}
```

**Test 4: Multiple ext blocks**
```auto
// 035_ext_multiple.at
ext str {
    fn len() int {
        return .size
    }
}

ext str {
    fn is_empty() bool {
        return .size == 0
    }
}

fn main() {
    let s = str.new("", 0)
    print(s.is_empty())  // Expected: true
}
```

**Test 5: Method with .prop shorthand**
```auto
// 035_ext_prop_shorthand.at
type Point { x int, y int }

ext Point {
    fn distance() int {
        let dx = .x * .x  // .x → self.x
        let dy = .y * .y
        return sqrt(dx + dy)
    }
}

fn main() {
    let p = Point { x: 3, y: 4 }
    print(p.distance())  // Expected: 5
}
```

#### 5.2 Migration from Plan 025 Functions

Convert standalone string functions to methods:

```auto
// Before (Plan 025):
let s = str_new("hello", 5)
let len = str_len(s)
str_append(mut s, "world")

// After (with ext):
ext str {
    static fn new(data *char, size int) str {
        return str_new(data, size)
    }

    fn len() int {
        return .size
    }

    fn append(other str) {
        str_append(mut self, other)
    }
}

// Usage:
let s = str.new("hello", 5)
let len = s.len()
s.append("world")
```

**Migration Strategy**:
1. Keep old functions as aliases for backward compatibility
2. Add new methods via `ext` blocks
3. Update documentation to recommend method syntax
4. Deprecate standalone functions in future version

**Success Criteria**:
- [ ] All 5 basic tests pass
- [ ] Migration tests demonstrate OOP-style API
- [ ] Backward compatibility maintained (old functions still work)
- [ ] Documentation updated with method examples

---

## Implementation Order

### Recommended Sequence

1. **Phase 1** (AST) - Foundation, no dependencies
2. **Phase 2** (Parser) - Depends on Phase 1
3. **Phase 4** (Static/Instance) - Can be done in parallel with Phase 3
4. **Phase 3** (Evaluator) - Depends on Phases 1-2
5. **Phase 5** (Testing) - Depends on Phases 1-4

### Minimum Viable Product (MVP)

**If time is limited, implement Phases 1-3 + basic tests**:
- ✅ AST structure for `ext` statement
- ✅ Parser recognition
- ✅ Evaluator integration
- ✅ Basic instance methods (with `self`)
- ❌ Advanced: Static methods, method overload resolution, generic methods

---

## Key Files

### Files to Modify

1. **`crates/auto-lang/src/ast/stmt.rs`** (Phase 1)
   - Add `Stmt::Ext` variant
   - Update derive macros

2. **`crates/auto-lang/src/lexer.rs`** (Phase 2)
   - Add `TokenKind::Ext` and `TokenKind::Static`

3. **`crates/auto-lang/src/ast/fun.rs`** (Phase 4)
   - Add `is_static: bool` field to `Fn` struct

4. **`crates/auto-lang/src/parser.rs`** (Phases 2-4)
   - Add `parse_ext()` method
   - Integrate into `parse_stmt()`
   - Parse `static` keyword in `parse_fn()`

5. **`crates/auto-lang/src/eval.rs`** (Phases 3-4)
   - Add `eval_ext()` method
   - Extend method lookup
   - Static/instance method detection

6. **`crates/auto-lang/src/universe.rs`** (Phase 3)
   - Extend TypeInfoStore with ext method registration

7. **`crates/auto-lang/test/a2c/035_ext_statement/`** (Phase 5)
   - Create 5+ test cases
   - Expected C/Rust output files

### Files to Create

8. **`stdlib/auto/ext.at`** (Phase 5)
   - Example ext blocks for str, cstr
   - Migration examples

9. **`docs/guides/ext-statement.md`** (Phase 5)
   - User guide for ext statement
   - Best practices
   - Migration guide from functions to methods

---

## Risk Analysis

### Risk 1: Breaking Existing Code
**Impact**: High
**Probability**: Low
**Mitigation**:
- Keep old functions as aliases
- Ext statement is opt-in, not breaking change
- Comprehensive test suite ensures backward compatibility

### Risk 2: Method Name Conflicts
**Impact**: Medium
**Probability**: Medium
**Mitigation**:
- Detect duplicate method names at parse/eval time
- Emit warnings, not errors (allow shadowing)
- Clear error messages guide users

### Risk 3: Performance Impact
**Impact**: Low
**Probability**: Low
**Mitigation**:
- Method lookup is O(n) where n = methods on type
- Ext methods added to existing lookup tables
- No runtime overhead compared to regular methods

### Risk 4: Complex Interaction with Traits
**Impact**: Medium
**Probability**: Low
**Mitigation**:
- Ext methods participate in existing MRO (Method Resolution Order)
- Delegation and inheritance work as expected
- Tests verify interaction with `is`, `has`, `as` keywords

---

## Verification Steps

### End-to-End Test

1. **Create test file** `test_ext.at`:
```auto
ext str {
    fn len() int {
        return .size  // .prop shorthand
    }

    static fn new(data *char, size int) str {
        return str_new(data, size)
    }
}

fn main() {
    let s = str.new("hello world", 11)
    print(s.len())  // Expected: 11
}
```

2. **Run evaluator**:
```bash
cargo run --release -- run test_ext.at
```

3. **Verify output**:
   - ✅ Prints `11`
   - ✅ No errors or warnings
   - ✅ Method calls work correctly

### Transpiler Tests

```bash
# Test C transpilation
cargo run --release -- c test_ext.at
# Verify generated C compiles

# Test Rust transpilation
cargo run --release -- rust test_ext.at
# Verify generated Rust compiles
```

### Regression Tests

```bash
# Run all a2c tests
cargo test -p auto-lang -- trans

# Run all evaluator tests
cargo test -p auto-lang

# Verify existing tests still pass
```

---

## Success Criteria

### Must Have (MVP)

- [ ] `ext` statement syntax parses correctly
- [ ] Methods can be added to built-in types (str, cstr, int)
- [ ] Instance methods (with `self`) work correctly
- [ ] Static methods (without `self`) work correctly
- [ ] `.prop` shorthand works in ext methods
- [ ] Basic test suite passes (5+ tests)
- [ ] No breaking changes to existing code

### Should Have

- [ ] Method lookup finds ext methods
- [ ] Duplicate method detection with warnings
- [ ] TypeInfoStore integration for built-in types
- [ ] Migration of Plan 025 functions to methods
- [ ] Documentation and user guide

### Could Have

- [ ] Generic methods: `ext T { fn method(self, arg T) }`
- [ ] Method overloading based on parameter types
- [ ] Default trait implementations via ext
- [ ] IDE integration (method completion)
- [ ] Performance optimization benchmarks

---

## Timeline Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: AST Structure | 1 day | None |
| Phase 2: Parser | 2-3 days | Phase 1 |
| Phase 4: Static/Instance | 1 day | Phase 1 |
| Phase 3: Evaluator | 3-4 days | Phases 1-2 |
| Phase 5: Testing | 3-4 days | Phases 1-4 |
| **Total (MVP)** | **10-13 days** | **Phases 1-4 + basic tests** |
| **Total (Complete)** | **2-3 weeks** | **All phases** |

---

## Next Steps

1. **Review and approve this plan**
2. **Start with Phase 1** (Add `Stmt::Ext` to AST)
3. **Implement incrementally** (test each phase)
4. **Refine based on feedback** (adjust syntax if needed)

---

**Plan End**
