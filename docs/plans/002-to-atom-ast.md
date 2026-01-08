# Plan: Add `to_atom()` Methods to AutoLang AST

## Objective

Add a new ATOM format representation for all AST structs in the AutoLang codebase, complementing the existing S-expression Display format. This enables AST to be represented in Auto's ATOM format (with nodes, arrays, and objects) for ASTL (Auto Syntax Tree Language) support.

---

## Background

### Current State

- All AST structs (50+ across 20 modules) have `impl fmt::Display` for S-expression output
- Example: `(fn (name add) (params ...) (ret int) (body ...))`
- No ATOM format representation exists currently

### Target Format (from atom.md and astl.md)

ATOM format combines JSON and XML concepts:
- **Nodes**: `name(arg1:val1, arg2:val2) { children... }`
- **Arrays**: `[elem1, elem2, ...]`
- **Objects**: `{key: val, key2: val2}`
- **Pairs**: `key: val`

Example ATOM for C code:
```atom
fn(name: "main", return: int) {
  call(name: printf, args: ["Hello, World!\\n"])
  ret(0)
}
```

---

## ATOM Format Specification

### Design Principles

1. Use kebab-case for node names: `fn`, `let`, `binary-op`
2. Use snake_case for property names: `name`, `return_type`, `op`
3. Omit optional fields when `None`, include when `Some`
4. Use concise representation: `int(42)` not `int(value: 42)`
5. Preserve all information from S-expression format

### AST Type → ATOM Mapping

#### Core Types

**Code** (root container):
```atom
code {
  stmt(...)
  stmt(...)
}
```

**Expression Variants**:
- Literals → direct values or simple nodes
  - `Expr::Int(42)` → `int(42)`
  - `Expr::Str("hello")` → `str("hello")`
  - `Expr::Ident(x)` → `name("x")`
  - `Expr::Bool(true)` → `bool(true)`

- Binary/Unary operators:
  - `Expr::Bina(left, op, right)` → `bina(op: "+") { left; right }`
  - `Expr::Unary(op, expr)` → `una(op: "-") { expr }`

- Containers:
  - `Expr::Array([1, 2, 3])` → `array([1, 2, 3])`
  - `Expr::Object({key: val})` → `object { key: val }`

- Control flow:
  - `Expr::Call(call)` → `call { name; args([...]) }`
  - `Expr::Index(arr, idx)` → `index { arr; idx }`

**Statement Variants**:
- `Stmt::Store(store)` → `let(name: "x", type: int) { value }`
- `Stmt::Fn(fn)` → `fn(name: "add", return: int) { params; body }`
- `Stmt::If(if_)` → `if { branch(cond, body); else(body) }`
- `Stmt::For(for_)` → `for(iter: "x") { range; body }`
- `Stmt::Use(use_)` → `use(kind: "auto", paths: ["io"])`

**Struct Types**:
- `Fn` → `fn(name: "add", kind: "function", return: int) { params; body }`
- `Param` → `param(name: "x", type: int)`
- `If` → `if { branch { cond; body }; else { body } }`
- `Call` → `call { callee; args([...]) }`
- `Store` → `let/mut/var(name: "x", type: int) { value }`

---

## Implementation Strategy

### Method Signature

**Trait Definition**:
```rust
use auto_val::Value;

pub trait ToAtom {
    fn to_atom(&self) -> Value;
}
```

**Return Value**: `Value` from `auto_val` crate
- Reuses existing infrastructure (Node, Array, Obj, Pair)
- Consistent with `eval.rs` patterns (AST → Value conversions)
- No need for separate Atom type

### Implementation Approach

**Trait-based approach** (recommended):
```rust
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}

impl ToAtom for Expr { fn to_atom(&self) -> Value { ... } }
impl ToAtom for Stmt { fn to_atom(&self) -> Value { ... } }
// etc.
```

**Benefits**:
- Enables generic programming
- Consistent with existing patterns (`fmt::Display`, `Serialize`)
- Easier to add blanket implementations
- Can provide default implementations for common cases

### Implementation Order (Dependency-First)

#### Phase 1: Foundation Types (no dependencies)
1. `Type` (types.rs)
2. `Key`, `Pair` (types.rs)
3. `Param`, `Member` (types.rs, fun.rs)
4. `Arg`, `Args` (call.rs)
5. `Body` (body.rs)
6. `Branch` (branch.rs)

#### Phase 2: Expression Types (depend on Phase 1)
7. `Expr` enum - literals:
   - `Int`, `Uint`, `Float`, `Bool`, `Char`, `Str`, `Nil`, `Null`
8. `Expr` enum - identifiers:
   - `Ident`, `Ref`, `GenName`
9. `Expr` enum - operators:
   - `Unary`, `Bina`
10. `Expr` enum - containers:
    - `Array`, `Object`, `Pair`
11. `Expr` enum - complex:
    - `Call`, `If`, `Index`, etc.

#### Phase 3: Statement Types (depend on Phase 1-2)
12. `Store` (store.rs)
13. `If` (if_.rs)
14. `For` (for_.rs)
15. `Fn` (fun.rs)
16. `Call` (call.rs)
17. `Use` (use_.rs)
18. Other statement types

#### Phase 4: Top-Level Containers
19. `Code` (ast.rs)
20. `Stmt` enum (ast.rs)

#### Phase 5: Advanced Features
21. `Node` (node.rs)
22. `TypeDecl` (types.rs)
23. `EnumDecl` (enums.rs)
24. `Tag`, `Union`, remaining types

---

## File Organization

### Module Structure

Add `to_atom()` impls directly in each AST module file:
- `crates/auto-lang/src/ast/fun.rs`: `impl ToAtom for Fn { ... }`
- `crates/auto-lang/src/ast/types.rs`: `impl ToAtom for Type { ... }`
- etc.

**Rationale**:
- Follows existing pattern (Display impls are in each module)
- Easier to maintain (co-located with struct definition)
- Each file stays self-contained

### Helper Functions Module

Create `crates/auto-lang/src/ast/atom_helpers.rs`:
```rust
use auto_val::{Value, Node, Array, Obj, ValueKey};

/// Helper functions for ATOM construction
pub struct AtomBuilder;

impl AtomBuilder {
    /// Create a new node with given name
    pub fn node(name: &str) -> Node {
        Node::new(name)
    }

    /// Create an array from values
    pub fn array(items: Vec<Value>) -> Value {
        Value::array(Array::from_vec(items))
    }

    /// Create an object from pairs
    pub fn object(pairs: Vec<(ValueKey, Value)>) -> Value {
        let mut obj = Obj::new();
        for (key, value) in pairs {
            obj.set(key, value);
        }
        Value::Obj(obj)
    }

    /// Create a key-value pair
    pub fn pair(key: ValueKey, value: Value) -> Value {
        Value::Pair(key, Box::new(value))
    }

    // Specialized helpers for common patterns
    pub fn int_node(value: i32) -> Value {
        let mut node = Node::new("int");
        node.add_arg(Value::Int(value));
        Value::Node(node)
    }

    pub fn str_node(value: &str) -> Value {
        let mut node = Node::new("str");
        node.add_arg(Value::str(value));
        Value::Node(node)
    }

    pub fn ident_node(name: &str) -> Value {
        let mut node = Node::new("name");
        node.add_arg(Value::str(name));
        Value::Node(node)
    }
}
```

Add to `ast.rs`:
```rust
mod atom_helpers;
pub use atom_helpers::*;
```

---

## Testing Strategy

### Test File Organization

```
crates/auto-lang/tests/atom/
├── literals/
│   ├── int_test.rs
│   ├── str_test.rs
│   └── bool_test.rs
├── expressions/
│   ├── binary_op_test.rs
│   ├── array_test.rs
│   └── object_test.rs
├── statements/
│   ├── let_test.rs
│   ├── fn_test.rs
│   └── if_test.rs
└── integration/
    └── full_program_test.rs
```

Alternatively, add inline tests in each AST module file (follows existing pattern).

### Test Case Template

```rust
#[cfg(test)]
mod to_atom_tests {
    use super::*;
    use auto_val::Value;

    #[test]
    fn test_expr_to_atom_int() {
        let expr = Expr::Int(42);
        let atom = expr.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "int");
                assert_eq!(node.args.args.len(), 1);
                assert_eq!(node.args.args[0], Arg::Pos(Value::Int(42)));
            }
            _ => panic!("Expected Node, got {:?}", atom),
        }
    }

    #[test]
    fn test_fn_to_atom() {
        let fn_decl = Fn::new(
            FnKind::Function,
            "add".into(),
            None,
            vec![
                Param::new("a".into(), Type::Int, None),
                Param::new("b".into(), Type::Int, None),
            ],
            Body::new(),
            Type::Int,
        );

        let atom = fn_decl.to_atom();

        match atom {
            Value::Node(node) => {
                assert_eq!(node.name, "fn");
                assert_eq!(node.get_prop_str("name"), Some("add".into()));
                assert_eq!(node.get_prop_str("return"), Some("int".into()));
                // Verify params are children...
            }
            _ => panic!("Expected Node"),
        }
    }
}
```

### Comparison Tests

Compare `to_atom()` with existing `Display` output to ensure consistency:
```rust
#[test]
fn test_display_and_atom_consistency() {
    let expr = Expr::Bina(
        Box::new(Expr::Int(1)),
        Op::Add,
        Box::new(Expr::Int(2))
    );

    let display_str = format!("{}", expr);  // "(bina (int 1) (op +) (int 2))"
    let atom = expr.to_atom();

    // Verify ATOM contains same information
    match atom {
        Value::Node(node) => {
            assert_eq!(node.name, "bina");
            assert_eq!(node.get_prop_str("op"), Some("+".into()));
            // Verify children...
        }
        _ => panic!("Expected Node"),
    }
}
```

---

## Implementation Steps

### Step 1: Prepare Infrastructure (30 min)

1. **Add ToAtom trait** to `crates/auto-lang/src/ast.rs`:
   ```rust
   use auto_val::Value;

   /// Converts AST node to ATOM format Value.
   ///
   /// # ATOM Format
   ///
   /// The ATOM format represents AST as a tree of nodes, arrays, and objects.
   /// This is used for ASTL (Auto Syntax Tree Language) representation.
   ///
   /// # Example
   ///
   /// ```rust
   /// use auto_lang::ast::*;
   ///
   /// let expr = Expr::Int(42);
   /// let atom = expr.to_atom();
   /// // Returns: Value::Node(Node { name: "int", args: [42], ... })
   /// ```
   pub trait ToAtom {
       fn to_atom(&self) -> Value;
   }
   ```

2. **Create atom_helpers.rs** with helper functions
3. **Add test module structure** in tests/ directory
4. **Verify auto-val dependency** (should already exist)

### Step 2: Implement Foundation Types (2 hours)

1. `impl ToAtom for Type` (types.rs)
2. `impl ToAtom for Key` (types.rs)
3. `impl ToAtom for Pair` (types.rs)
4. `impl ToAtom for Param` (fun.rs)
5. `impl ToAtom for Arg` (call.rs)
6. `impl ToAtom for Args` (call.rs)
7. `impl ToAtom for Body` (body.rs)
8. `impl ToAtom for Branch` (branch.rs)
9. **Add tests** for each foundation type

### Step 3: Implement Simple Expressions (2 hours)

1. Implement `ToAtom for Expr` - literals:
   - `Int`, `Uint`, `Float`, `Double`, `Bool`, `Char`
   - `Str`, `CStr`, `Nil`, `Null`, `Void`

2. Implement `ToAtom for Expr` - identifiers:
   - `Ident`, `Ref`, `GenName`

3. **Add tests** for all simple expressions
4. **Run tests**: `cargo test -p auto-lang to_atom`

### Step 4: Implement Complex Expressions (3 hours)

1. Implement `ToAtom for Expr` - unary/binary:
   - `Unary`, `Bina`

2. Implement `ToAtom for Expr` - containers:
   - `Array`, `Object`

3. Implement `ToAtom for Expr` - control flow:
   - `If` (delegates to `If::to_atom()`)

4. Implement `ToAtom for Expr` - calls/index:
   - `Call`, `Index`

5. Implement remaining `Expr` variants

6. **Add tests** for all complex expressions
7. **Run tests**

### Step 5: Implement Statements (3 hours)

1. `impl ToAtom for Store` (store.rs)
2. `impl ToAtom for If` (if_.rs)
3. `impl ToAtom for For` (for_.rs)
4. `impl ToAtom for Fn` (fun.rs)
5. `impl ToAtom for Use` (use_.rs)
6. Implement remaining statement types
7. `impl ToAtom for Stmt` enum (ast.rs)
8. **Add tests** for all statements
9. **Run tests**

### Step 6: Implement Top-Level (1 hour)

1. `impl ToAtom for Code` (ast.rs)
2. `impl ToAtom for Range` (range.rs)
3. Implement remaining special types
4. **Add integration tests**
5. **Run full test suite**

### Step 7: Integration and Documentation (1 hour)

1. **Add documentation comments** to all `to_atom()` methods
2. **Create usage examples** in docs
3. **Update CLAUDE.md** with ToAtom usage guidelines
4. **Run full test suite**: `cargo test -p auto-lang --lib`
5. **Verify all 225+ tests pass**

---

## Common Implementation Patterns

### Pattern 1: Simple Wrapper Nodes

```rust
impl ToAtom for Expr {
    fn to_atom(&self) -> Value {
        match self {
            Expr::Int(i) => {
                let mut node = Node::new("int");
                node.add_arg(Value::Int(*i));
                Value::Node(node)
            }
            Expr::Str(s) => {
                let mut node = Node::new("str");
                node.add_arg(Value::str(s));
                Value::Node(node)
            }
            // ... similar for other literals
        }
    }
}
```

### Pattern 2: Container Nodes

```rust
impl ToAtom for Expr {
    fn to_atom(&self) -> Value {
        match self {
            Expr::Array(elems) => {
                let items: Vec<Value> = elems.iter()
                    .map(|e| e.to_atom())
                    .collect();
                let mut node = Node::new("array");
                node.add_arg(Value::array(Array::from_vec(items)));
                Value::Node(node)
            }
            Expr::Object(pairs) => {
                let mut obj = Obj::new();
                for pair in pairs {
                    let key = match &pair.key {
                        Key::NamedKey(k) => ValueKey::Str(k.clone()),
                        Key::IntKey(i) => ValueKey::Int(*i),
                        Key::BoolKey(b) => ValueKey::Bool(*b),
                    };
                    obj.set(key, pair.value.to_atom());
                }
                let mut node = Node::new("object");
                node.add_arg(Value::Obj(obj));
                Value::Node(node)
            }
            // ... other variants
        }
    }
}
```

### Pattern 3: Complex Nodes with Props

```rust
impl ToAtom for Fn {
    fn to_atom(&self) -> Value {
        let mut node = Node::new("fn");
        node.set_prop("name", Value::str(self.name.clone()));
        node.set_prop("kind", Value::str(format!("{:?}", self.kind)));

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", self.ret.to_atom());
        }

        // Add params as children
        for param in &self.params {
            if let Value::Node(param_node) = param.to_atom() {
                node.add_kid(param_node);
            }
        }

        // Add body
        if let Value::Node(body_node) = self.body.to_atom() {
            node.add_kid(body_node);
        }

        Value::Node(node)
    }
}
```

### Pattern 4: Conditional Properties

```rust
impl ToAtom for Store {
    fn to_atom(&self) -> Value {
        let node_name = match &self.kind {
            StoreKind::Let => "let",
            StoreKind::Mut => "mut",
            StoreKind::Var => "var",
            _ => "store",
        };

        let mut node = Node::new(node_name);
        node.set_prop("name", Value::str(self.name.clone()));

        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", self.ty.to_atom());
        }

        if let Value::Node(expr_node) = self.expr.to_atom() {
            node.add_kid(expr_node);
        }

        Value::Node(node)
    }
}
```

### Pattern 5: Recursive Calls

```rust
impl ToAtom for Expr {
    fn to_atom(&self) -> Value {
        match self {
            Expr::Bina(left, op, right) => {
                let mut node = Node::new("bina");
                node.set_prop("op", Value::str(op.to_string()));

                if let Value::Node(left_node) = left.to_atom() {
                    node.add_kid(left_node);
                }
                if let Value::Node(right_node) = right.to_atom() {
                    node.add_kid(right_node);
                }

                Value::Node(node)
            }
            // ... other variants
        }
    }
}
```

---

## Critical Design Decisions

### Naming Convention

- **Node names**: kebab-case: `fn`, `let`, `binary-op`, `type-decl`
- **Property names**: snake_case: `name`, `return_type`, `op`
- Consistent with AutoLang conventions

### Handling Optional Fields

- Omit if `None`: `if { branch(...) }`
- Include if `Some`: `if { branch(...); else(body) }`

### Type Representation

**Concise** (recommended):
```
int(42)
str("hello")
```

### Operator Representation

**String in args** (recommended):
```
bina(op: "+") { left; right }
```

Rationale: Consistent with Display format, preserves operator identity.

### Handling Nested Values

Recursively call `to_atom()` for child nodes:
```rust
Expr::Bina(left, op, right) => {
    let mut node = Node::new("bina");
    node.add_kid(left.to_atom().as_node());
    node.add_kid(right.to_atom().as_node());
    Value::Node(node)
}
```

---

## Documentation Requirements

### Rust Documentation

Add doc comments to trait:
```rust
/// Converts AST node to ATOM format Value.
///
/// # ATOM Format
///
/// The ATOM format represents AST as a tree of nodes, arrays, and objects.
/// This is used for ASTL (Auto Syntax Tree Language) representation.
///
/// # Example
///
/// ```rust
/// use auto_lang::ast::*;
///
/// let expr = Expr::Int(42);
/// let atom = expr.to_atom();
/// // Returns: Value::Node(Node { name: "int", args: [42], ... })
/// ```
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}
```

### CLAUDE.md Updates

Add section:
```markdown
## AST to ATOM Conversion

### ToAtom Trait

All AST types implement the `ToAtom` trait for converting to ATOM format:

\`\`\`rust
let code = parse("let x = 42").unwrap();
let atom = code.to_atom();
// atom is now Value::Node representing the AST in ATOM format
\`\`\`

### Usage Examples

1. **Serialize AST to ATOM**:
   \`\`\`rust
   use std::fs;
   let code = parse_file("main.at").unwrap();
   let atom = code.to_atom();
   fs::write("ast.atom", atom.to_string());
   \`\`\`

2. **Pass AST between tools**:
   \`\`\`rust
   let atom = ast.to_atom();
   transmit(&atom);  // Send to another process
   \`\`\`

3. **Debug AST structure**:
   \`\`\`rust
   println!("AST: {:#?}", ast.to_atom());
   \`\`\`
```

---

## Milestones and Validation

### Milestone 1: Foundation (Day 1)
- [ ] ToAtom trait defined in ast.rs
- [ ] Helper functions created (atom_helpers.rs)
- [ ] 8 basic types implemented (Type, Key, Pair, Param, Arg, Args, Body, Branch)
- [ ] Test infrastructure setup
- [ ] Tests pass for foundation types

### Milestone 2: Expressions (Day 2)
- [ ] All literal expressions implemented
- [ ] Binary/unary operators implemented
- [ ] Array/object expressions implemented
- [ ] 100% test coverage for expressions
- [ ] All expression tests pass

### Milestone 3: Statements (Day 3)
- [ ] All statement types implemented
- [ ] Function declarations implemented
- [ ] Control flow implemented
- [ ] 100% test coverage for statements
- [ ] All statement tests pass

### Milestone 4: Completion (Day 4)
- [ ] All 50+ structs implemented
- [ ] Integration tests passing
- [ ] Documentation complete
- [ ] Examples provided
- [ ] Full test suite passes (225+ tests)

---

## Critical Files for Implementation

1. **`crates/auto-lang/src/ast.rs`**
   - Main enums (Code, Stmt, Expr)
   - ToAtom trait definition
   - Core conversion logic for enum dispatch

2. **`crates/auto-lang/src/ast/atom_helpers.rs`** (new)
   - Helper functions for ATOM construction
   - Common patterns and utilities

3. **`crates/auto-val/src/value.rs`** (reference)
   - Value enum that all to_atom() methods return
   - Understanding available Value variants

4. **`crates/auto-val/src/node.rs`** (reference)
   - Node struct for ATOM node representation
   - Node construction methods

5. **`crates/auto-lang/src/ast/types.rs`**
   - Type, TypeDecl, Member, Pair, Key definitions
   - Foundation types (first to implement)

6. **`crates/auto-lang/src/ast/fun.rs`**
   - Fn, Param structs
   - Complex multi-property node example

---

## Success Criteria

### Functional Requirements
- ✅ All 50+ AST structs implement `ToAtom` trait
- ✅ All existing tests pass (225+ tests)
- ✅ New tests added for each AST type's to_atom() method
- ✅ ATOM format preserves all information from S-expression format

### Code Quality Metrics
- ✅ 100% trait coverage (all AST types implement ToAtom)
- ✅ Test coverage for to_atom() methods (minimum 80%)
- ✅ Documentation on all public to_atom() methods
- ✅ No code duplication (use helper functions)

### Maintainability
- ✅ Consistent ATOM format across all types
- ✅ Clear naming conventions followed
- ✅ Helper functions reduce duplication
- ✅ Examples provided for common patterns

---

## Edge Cases and Considerations

1. **Circular References**: Not applicable (AST trees are acyclic)

2. **Large Data Structures**: For large arrays, use iterators instead of collecting where possible

3. **Error Handling**: `to_atom()` should never fail (AST is always valid). Use `unwrap()` sparingly, prefer defaults.

4. **Performance**: Consider caching `to_atom()` results if called repeatedly.

5. **Serde Integration**: `ToAtom` complements `Serialize`, doesn't replace it:
   - `Serialize` → JSON/other formats
   - `ToAtom` → Auto-specific ATOM format

---

## Timeline Estimate

- **Step 1** (Infrastructure): 30 minutes
- **Step 2** (Foundation Types): 2 hours
- **Step 3** (Simple Expressions): 2 hours
- **Step 4** (Complex Expressions): 3 hours
- **Step 5** (Statements): 3 hours
- **Step 6** (Top-Level): 1 hour
- **Step 7** (Integration/Docs): 1 hour

**Total**: ~12-13 hours (1.5-2 days) for complete implementation
