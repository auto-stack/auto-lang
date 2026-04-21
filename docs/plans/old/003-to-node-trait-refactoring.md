# Plan: Refactor ToAtom to ToNode Trait

## Objective

Refactor the AST representation layer to return `auto_val::Node` directly instead of wrapping it in `Value::Node`, eliminating redundant type conversions and improving API clarity.

---

## Background

### Current State

- **ToAtom trait** returns `Value` for all AST types
- **32 types** return `Value::Node` (If, For, Fn, Store, etc.)
- **3 types** return other Value variants:
  - `Type` → `Value::Str`
  - `Key` → `Value::Int/Bool/Str`
  - `Pair` → `Value::Pair`
- **42 call sites** use `.to_atom().to_node()` pattern across 15 files
- **All 245 tests pass** with current implementation

### Problem Statement

The current API has redundancy:
```rust
// Current: returns Value, but almost always Value::Node
let atom = expr.to_atom();           // Value
let node = atom.to_node().unwrap();  // Node (with unwrap())

// Desired: return Node directly for node-producing types
let node = expr.to_node();           // Node
```

This creates:
1. **Unnecessary type wrapping**: Node wrapped in Value enum
2. **Unwrapping boilerplate**: `.to_atom().to_node()` everywhere
3. **Type safety issues**: `unwrap()` calls in 42 locations
4. **API confusion**: Not clear which types return which Value variants

---

## Implementation Strategy

### Overview

Create a **new ToNode trait** that returns `auto_val::Node` directly for types that naturally produce node structures. Keep the existing ToAtom trait for primitive/atomic types (Type, Key, Pair).

### Two-Trait System

```rust
// For primitive/atomic values (Type, Key, Pair)
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}

// For complex AST structures (If, For, Fn, Store, etc.)
pub trait ToNode {
    fn to_node(&self) -> Node;
}
```

### Benefits

1. **Type safety**: No `unwrap()` calls needed
2. **Clearer API**: ToNode for nodes, ToAtom for atoms
3. **Better performance**: Eliminates redundant Value wrapping
4. **Delegation**: ToAtom can delegate to ToNode for efficiency

---

## Step-by-Step Implementation

### Step 1: Add ToNode Trait Definition

**File**: `crates/auto-lang/src/ast.rs`

Add the new trait alongside ToAtom:

```rust
use auto_val::{Node, Value};

/// Converts AST node to ATOM format Value (for primitive/atomic types)
pub trait ToAtom {
    fn to_atom(&self) -> Value;
}

/// Converts AST node to ATOM format Node directly (for complex structures)
///
/// # When to Implement ToNode vs ToAtom
///
/// - **ToNode**: For AST types that are naturally represented as nodes
///   with children, properties, and arguments (If, For, Fn, Store, etc.)
///
/// - **ToAtom**: For primitive/atomic types that map to simple values
///   (Type → Value::Str, Key → Value::Int/Bool/Str, Pair → Value::Pair)
///
/// # Example
///
/// ```rust
/// use auto_lang::ast::*;
///
/// let if_stmt = If { ... };
/// let node = if_stmt.to_node();  // Returns Node directly
///
/// let ty = Type::Int;
/// let value = ty.to_atom();      // Returns Value::Str("int")
/// ```
pub trait ToNode {
    fn to_node(&self) -> Node;
}
```

**Time**: 5 minutes

---

### Step 2: Implement ToNode for Foundation Types

#### 2.1: Param (fun.rs)

```rust
impl ToNode for Param {
    fn to_node(&self) -> Node {
        let mut node = Node::new("param");
        node.set_prop("name", Value::str(self.name.as_str()));
        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", self.ty.to_atom());
        }
        if let Some(default) = &self.default {
            node.add_kid(default.to_atom().to_node());
        }
        node
    }
}
```

#### 2.2: Arg (call.rs)

```rust
impl ToNode for Arg {
    fn to_node(&self) -> Node {
        match self {
            Arg::Pos(expr) => expr.to_atom().to_node(),
            Arg::Name(name) => {
                let mut node = Node::new("name");
                node.add_arg(Value::Str(name.clone()));
                node
            }
            Arg::Pair(key, expr) => {
                let mut node = Node::new("pair");
                node.add_arg(Value::str(key.as_str()));
                node.add_arg(expr.to_atom());
                node
            }
        }
    }
}
```

#### 2.3: Args (call.rs)

```rust
impl ToNode for Args {
    fn to_node(&self) -> Node {
        let mut node = Node::new("args");
        let items: Vec<Value> = self.args.iter().map(|arg| arg.to_atom()).collect();
        node.add_arg(Value::array(Array::from_vec(items)));
        node
    }
}
```

#### 2.4: Body (body.rs)

```rust
impl ToNode for Body {
    fn to_node(&self) -> Node {
        let mut node = Node::new("body");
        for stmt in &self.stmts {
            node.add_kid(stmt.to_node());
        }
        node
    }
}
```

#### 2.5: Branch (branch.rs)

```rust
impl ToNode for Branch {
    fn to_node(&self) -> Node {
        let mut node = Node::new("branch");
        node.add_kid(self.cond.to_atom().to_node());
        node.add_kid(self.body.to_node());
        node
    }
}
```

**Time**: 30 minutes

---

### Step 3: Implement ToNode for Statement Types

#### 3.1: Store (store.rs)

```rust
impl ToNode for Store {
    fn to_node(&self) -> Node {
        let name = match &self.kind {
            StoreKind::Let => "let",
            StoreKind::Mut => "mut",
            StoreKind::Var => "var",
            StoreKind::Const => "const",
            _ => "store",
        };

        let mut node = Node::new(name);
        node.set_prop("name", Value::str(self.name.as_str()));

        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", self.ty.to_atom());
        }

        node.add_kid(self.expr.to_atom().to_node());
        node
    }
}
```

#### 3.2: If (if_.rs)

```rust
impl ToNode for If {
    fn to_node(&self) -> Node {
        let mut node = Node::new("if");

        // Add all branches
        for branch in &self.branches {
            node.add_kid(branch.to_node());
        }

        // Add else branch if present
        if let Some(else_body) = &self.else_branch {
            let mut else_node = Node::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }

        node
    }
}
```

#### 3.3: For (for_.rs)

```rust
impl ToNode for For {
    fn to_node(&self) -> Node {
        let mut node = Node::new("for");
        node.set_prop("iter", Value::str(self.var.as_str()));

        node.add_kid(self.range.to_node());

        if !self.body.stmts.is_empty() {
            node.add_kid(self.body.to_node());
        }

        node
    }
}
```

#### 3.4: Fn (fun.rs)

```rust
impl ToNode for Fn {
    fn to_node(&self) -> Node {
        let mut node = Node::new("fn");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("kind", Value::str(format!("{:?}", self.kind)));

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", self.ret.to_atom());
        }

        // Add params as children
        for param in &self.params {
            node.add_kid(param.to_node());
        }

        // Add body
        if !self.body.stmts.is_empty() {
            node.add_kid(self.body.to_node());
        }

        node
    }
}
```

#### 3.5: Use (use_.rs)

```rust
impl ToNode for Use {
    fn to_node(&self) -> Node {
        let mut node = Node::new("use");
        node.set_prop("kind", Value::str(format!("{:?}", self.kind)));

        let paths: Vec<Value> = self.paths
            .iter()
            .map(|p| Value::str(p.as_str()))
            .collect();

        node.add_arg(Value::array(Array::from_vec(paths)));
        node
    }
}
```

**Time**: 45 minutes

---

### Step 4: Implement ToNode for Call-related Types

#### 4.1: Call (call.rs)

```rust
impl ToNode for Call {
    fn to_node(&self) -> Node {
        let mut node = Node::new("call");
        node.add_kid(self.name.to_atom().to_node());
        node.add_kid(self.args.to_node());

        if !matches!(self.ret, Type::Unknown) {
            node.set_prop("return", self.ret.to_atom());
        }

        node
    }
}
```

**Time**: 15 minutes

---

### Step 5: Implement ToNode for Special Statement Types

#### 5.1: Is (is.rs)

```rust
impl ToNode for Is {
    fn to_node(&self) -> Node {
        let mut node = Node::new("is");
        node.add_kid(self.expr.to_atom().to_node());

        for (key, body) in &self.branches {
            let mut branch = Node::new("branch");
            branch.add_arg(match key {
                Key::NamedKey(k) => Value::str(k.as_str()),
                Key::IntKey(i) => Value::Int(*i),
                Key::BoolKey(b) => Value::Bool(*b),
            });
            branch.add_kid(body.to_node());
            node.add_kid(branch);
        }

        if let Some(else_body) = &self.else_branch {
            let mut else_node = Node::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }

        node
    }
}
```

#### 5.2: On (on.rs)

```rust
impl ToNode for On {
    fn to_node(&self) -> Node {
        let mut node = Node::new("on");
        node.set_prop("event", Value::str(self.event.as_str()));

        for (key, body) in &self.branches {
            let mut branch = Node::new("branch");
            branch.add_arg(match key {
                Key::NamedKey(k) => Value::str(k.as_str()),
                Key::IntKey(i) => Value::Int(*i),
                Key::BoolKey(b) => Value::Bool(*b),
            });
            branch.add_kid(body.to_node());
            node.add_kid(branch);
        }

        if let Some(else_body) = &self.else_branch {
            let mut else_node = Node::new("else");
            else_node.add_kid(else_body.to_node());
            node.add_kid(else_node);
        }

        node
    }
}
```

**Time**: 30 minutes

---

### Step 6: Implement ToNode for Type Declarations

#### 6.1: TypeDecl (types.rs)

```rust
impl ToNode for TypeDecl {
    fn to_node(&self) -> Node {
        let mut node = Node::new("type");
        node.set_prop("name", Value::str(self.name.as_str()));

        for member in &self.members {
            node.add_kid(member.to_node());
        }

        node
    }
}
```

#### 6.2: Member (types.rs)

```rust
impl ToNode for Member {
    fn to_node(&self) -> Node {
        let mut node = Node::new("member");
        node.set_prop("name", Value::str(self.name.as_str()));

        if !matches!(self.ty, Type::Unknown) {
            node.set_prop("type", self.ty.to_atom());
        }

        if let Some(default) = &self.default {
            node.add_kid(default.to_atom().to_node());
        }

        node
    }
}
```

**Time**: 20 minutes

---

### Step 7: Implement ToNode for Advanced Types

#### 7.1: Tag (tag.rs)

```rust
impl ToNode for Tag {
    fn to_node(&self) -> Node {
        let mut node = Node::new("tag");
        node.set_prop("name", Value::str(self.name.as_str()));

        for (name, ty) in &self.items {
            let mut item = Node::new("item");
            item.set_prop("name", Value::str(name.as_str()));
            if !matches!(ty, Type::Unknown) {
                item.set_prop("type", ty.to_atom());
            }
            node.add_kid(item);
        }

        node
    }
}
```

#### 7.2: Union (union.rs)

```rust
impl ToNode for Union {
    fn to_node(&self) -> Node {
        let mut node = Node::new("union");
        node.set_prop("name", Value::str(self.name.as_str()));

        for (name, ty) in &self.items {
            let mut item = Node::new("item");
            item.set_prop("name", Value::str(name.as_str()));
            if !matches!(ty, Type::Unknown) {
                item.set_prop("type", ty.to_atom());
            }
            node.add_kid(item);
        }

        node
    }
}
```

#### 7.3: EnumDecl (enums.rs)

```rust
impl ToNode for EnumDecl {
    fn to_node(&self) -> Node {
        let mut node = Node::new("enum");
        node.set_prop("name", Value::str(self.name.as_str()));

        for item in &self.items {
            node.add_kid(item.to_node());
        }

        node
    }
}
```

#### 7.4: EnumItem (enums.rs)

```rust
impl ToNode for EnumItem {
    fn to_node(&self) -> Node {
        let mut node = Node::new("item");
        node.set_prop("name", Value::str(self.name.as_str()));
        node.set_prop("value", Value::Int(self.value));
        node
    }
}
```

**Time**: 30 minutes

---

### Step 8: Implement ToNode for Remaining Types

#### 8.1: Node (node.rs)

```rust
impl ToNode for Node {
    fn to_node(&self) -> Node {
        let mut node = auto_val::Node::new("node");
        node.set_prop("name", Value::str(self.name.as_str()));

        if !self.id.is_empty() {
            node.set_prop("id", Value::str(self.id.as_str()));
        }

        if !self.args.is_empty() {
            node.add_kid(self.args.to_node());
        }

        if !self.body.stmts.is_empty() {
            node.add_kid(self.body.to_node());
        }

        node
    }
}
```

#### 8.2: Alias (alias.rs)

```rust
impl ToNode for Alias {
    fn to_node(&self) -> Node {
        let mut node = Node::new("alias");
        node.set_prop("name", Value::str(self.alias.as_str()));
        node.set_prop("target", Value::str(self.target.as_str()));
        node
    }
}
```

#### 8.3: Range (range.rs)

```rust
impl ToNode for Range {
    fn to_node(&self) -> Node {
        let mut node = Node::new("range");
        node.set_prop("inclusive", Value::Bool(self.inclusive));
        node.add_kid(self.start.to_atom().to_node());
        node.add_kid(self.end.to_atom().to_node());
        node
    }
}
```

#### 8.4: Code (ast.rs)

```rust
impl ToNode for Code {
    fn to_node(&self) -> Node {
        let mut node = Node::new("code");
        for stmt in &self.stmts {
            node.add_kid(stmt.to_node());
        }
        node
    }
}
```

**Time**: 25 minutes

---

### Step 9: Implement ToNode for Stmt Enum

**File**: `crates/auto-lang/src/ast.rs`

```rust
impl ToNode for Stmt {
    fn to_node(&self) -> Node {
        match self {
            Stmt::Store(store) => store.to_node(),
            Stmt::Fn(fn) => fn.to_node(),
            Stmt::If(if_) => if_.to_node(),
            Stmt::For(for_) => for_.to_node(),
            Stmt::Use(use_) => use_.to_node(),
            Stmt::Is(is) => is.to_node(),
            Stmt::On(on) => on.to_node(),
            Stmt::TypeDecl(decl) => decl.to_node(),
            Stmt::EnumDecl(decl) => decl.to_node(),
            Stmt::Tag(tag) => tag.to_node(),
            Stmt::Union(union) => union.to_node(),
            Stmt::Alias(alias) => alias.to_node(),
            Stmt::Node(node) => node.to_node(),
            Stmt::Expr(expr) => expr.to_atom().to_node(),
            Stmt::Break => Node::new("break"),
            Stmt::Continue => Node::new("continue"),
            Stmt::Ret(ret) => {
                let mut node = Node::new("ret");
                if let Some(expr) = ret {
                    node.add_kid(expr.to_atom().to_node());
                }
                node
            }
        }
    }
}
```

**Time**: 15 minutes

---

### Step 10: Update ToAtom to Delegate to ToNode

For types implementing both ToAtom and ToNode, update ToAtom to delegate:

```rust
// Example for If (if_.rs)
impl ToAtom for If {
    fn to_atom(&self) -> Value {
        Value::Node(self.to_node())  // Delegate to ToNode
    }
}
```

**Files to update** (all 32 types from previous steps):
- fun.rs (Fn, Param)
- call.rs (Call, Args, Arg)
- body.rs (Body)
- branch.rs (Branch)
- store.rs (Store)
- if_.rs (If)
- for_.rs (For)
- use_.rs (Use)
- is.rs (Is)
- on.rs (On)
- types.rs (TypeDecl, Member)
- tag.rs (Tag)
- union.rs (Union)
- enums.rs (EnumDecl, EnumItem)
- node.rs (Node)
- alias.rs (Alias)
- range.rs (Range)
- ast.rs (Code)

**Time**: 45 minutes

---

### Step 11: Update Call Sites - ast.rs

**8 occurrences**:

```rust
// Before
node.add_kid(self.body.to_atom().to_node());

// After
node.add_kid(self.body.to_node());
```

**Time**: 10 minutes

---

### Step 12: Update Call Sites - Other Files

**Remaining 34 occurrences** across 14 files:

- **branch.rs**: 2 occurrences
- **call.rs**: 2 occurrences
- **for_.rs**: 3 occurrences
- **enums.rs**: 1 occurrence
- **fun.rs**: 2 occurrences
- **if_.rs**: 2 occurrences
- **is.rs**: 11 occurrences
- **node.rs**: 2 occurrences
- **on.rs**: 2 occurrences
- **range.rs**: 2 occurrences
- **store.rs**: 1 occurrence
- **tag.rs**: 1 occurrence
- **types.rs**: 2 occurrences
- **union.rs**: 1 occurrence

Pattern:
```rust
// Before
.to_atom().to_node()
// After
.to_node()
```

**Time**: 30 minutes

---

### Step 13: Update Tests

**Files with affected tests**:
- `crates/auto-lang/src/ast/call.rs` (lines 230-371)
- `crates/auto-lang/src/ast/enums.rs` (lines 54-79)
- `crates/auto-lang/src/ast/alias.rs` (lines 16-28)
- `crates/auto-lang/src/ast/node.rs` (lines 54-78)

Update test assertions to use `to_node()`:

```rust
// Before
#[test]
fn test_call_to_atom_simple() {
    let call = Call { ... };
    let atom = call.to_atom();
    match atom {
        Value::Node(node) => { ... }
        _ => panic!("Expected Node"),
    }
}

// After
#[test]
fn test_call_to_node_simple() {
    let call = Call { ... };
    let node = call.to_node();
    assert_eq!(node.name, "call");
    // Direct access, no unwrap needed
}
```

**Time**: 30 minutes

---

### Step 14: Final Verification

1. **Run all tests**:
   ```bash
   cargo test -p auto-lang --lib
   ```
   Expected: All 245+ tests pass

2. **Check compilation**:
   ```bash
   cargo build --release
   ```
   Expected: No warnings, no errors

3. **Verify no unwrap() calls** for `.to_atom().to_node()`:
   ```bash
   grep -r "to_atom().to_node()" crates/auto-lang/src/
   ```
   Expected: No results (all replaced with `.to_node()`)

4. **Run clippy**:
   ```bash
   cargo clippy -p auto-lang
   ```
   Expected: No new warnings

**Time**: 15 minutes

---

### Step 15: Optional Cleanup

#### 15.1: Update Documentation

Update doc comments to reflect ToNode usage:
```rust
/// # Example
///
/// ```rust
/// let if_stmt = If { ... };
/// let node = if_stmt.to_node();  // Returns Node directly
/// ```
```

#### 15.2: Add ToNode Tests

Add dedicated tests for ToNode implementations:
```rust
#[test]
fn test_if_to_node() {
    let if_ = If { ... };
    let node = if_.to_node();
    assert_eq!(node.name, "if");
    assert!(node.has_prop("branches"));
}
```

**Time**: 30 minutes (optional)

---

## Types NOT Implementing ToNode

These types return primitive/atomic Value variants and **should NOT** implement ToNode:

1. **Type** → `Value::Str` (e.g., `Type::Int` → `"int"`)
2. **Key** → `Value::Int/Bool/Str` (enum variants)
3. **Pair** → `Value::Pair(key, value)` (key-value pair)

**Rationale**: These are primitive values, not node structures with children/properties.

---

## Testing Strategy

### Test Coverage

1. **Unit tests**: Each ToNode implementation should have tests
2. **Integration tests**: Verify Code.to_node() produces complete AST
3. **Regression tests**: Ensure existing ToAtom behavior unchanged

### Test Template

```rust
#[cfg(test)]
mod to_node_tests {
    use super::*;

    #[test]
    fn test_type_to_node_not_applicable() {
        // Type should use to_atom(), not to_node()
        let ty = Type::Int;
        let value = ty.to_atom();
        match value {
            Value::Str(s) => assert_eq!(s, "int"),
            _ => panic!("Type should return Value::Str"),
        }
    }

    #[test]
    fn test_if_to_node() {
        let if_ = If::new(...);
        let node = if_.to_node();
        assert_eq!(node.name, "if");
        assert!(node.nodes.len() > 0);
    }

    #[test]
    fn test_to_atom_delegates_to_to_node() {
        let if_ = If::new(...);
        let atom = if_.to_atom();
        let node = if_.to_node();

        match atom {
            Value::Node(atom_node) => {
                assert_eq!(atom_node.name, node.name);
            }
            _ => panic!("Expected Value::Node"),
        }
    }
}
```

---

## Risk Assessment

### Low Risk

- **ToAtom delegation**: ToAtom calls ToNode internally, maintaining compatibility
- **Comprehensive tests**: 245+ existing tests provide safety net
- **Incremental changes**: Can implement one type at a time

### Medium Risk

- **42 call sites**: Must update all consistently
- **Expr enum**: Special handling needed for variant types

### Mitigation

1. **Run tests after each step** to catch issues early
2. **Use git commits** after each major step for easy rollback
3. **Keep ToAtom implementations** as delegation pattern

---

## Rollback Plan

If issues arise:

1. **Revert changes**:
   ```bash
   git revert HEAD  # Rollback last commit
   ```

2. **Alternative approach**: Keep ToAtom as-is, add convenience methods:
   ```rust
   impl ToAtom for T {
       fn to_atom(&self) -> Value { ... }

       fn to_node_direct(&self) -> Node {
           self.to_atom().to_node().unwrap()
       }
   }
   ```

---

## Timeline Estimate

| Step | Description | Time |
|------|-------------|------|
| 1 | Add ToNode trait | 5 min |
| 2 | Foundation types | 30 min |
| 3 | Statement types | 45 min |
| 4 | Call-related types | 15 min |
| 5 | Special statement types | 30 min |
| 6 | Type declarations | 20 min |
| 7 | Advanced types | 30 min |
| 8 | Remaining types | 25 min |
| 9 | Stmt enum | 15 min |
| 10 | Update ToAtom delegation | 45 min |
| 11 | Update call sites (ast.rs) | 10 min |
| 12 | Update call sites (others) | 30 min |
| 13 | Update tests | 30 min |
| 14 | Final verification | 15 min |
| 15 | Optional cleanup | 30 min |
| **Total** | | **~5 hours** |

---

## Success Criteria

### Functional Requirements
- ✅ ToNode trait defined in ast.rs
- ✅ 32 AST types implement ToNode
- ✅ Type, Key, Pair keep ToAtom only
- ✅ All 42 call sites updated
- ✅ All 245+ tests pass

### Code Quality Metrics
- ✅ No `.to_atom().to_node().unwrap()` patterns remain
- ✅ No new compiler warnings
- ✅ ToAtom delegates to ToNode for efficiency
- ✅ Clear documentation on trait usage

### API Improvements
- ✅ Type-safe: No unwrap() calls
- ✅ Clearer: ToNode for nodes, ToAtom for atoms
- ✅ Efficient: Eliminates redundant wrapping

---

## Files Modified

**Summary**:
- 18 AST module files modified
- 1 new trait added (ToNode)
- 32 types implement ToNode
- 42 call sites updated
- 20+ test functions updated

**File List**:
1. `crates/auto-lang/src/ast.rs` - ToNode trait, Stmt enum, Code
2. `crates/auto-lang/src/ast/body.rs` - Body
3. `crates/auto-lang/src/ast/branch.rs` - Branch
4. `crates/auto-lang/src/ast/call.rs` - Call, Args, Arg
5. `crates/auto-lang/src/ast/enums.rs` - EnumDecl, EnumItem
6. `crates/auto-lang/src/ast/for_.rs` - For
7. `crates/auto-lang/src/ast/fun.rs` - Fn, Param
8. `crates/auto-lang/src/ast/if_.rs` - If
9. `crates/auto-lang/src/ast/is.rs` - Is
10. `crates/auto-lang/src/ast/node.rs` - Node
11. `crates/auto-lang/src/ast/on.rs` - On
12. `crates/auto-lang/src/ast/range.rs` - Range
13. `crates/auto-lang/src/ast/store.rs` - Store
14. `crates/auto-lang/src/ast/tag.rs` - Tag
15. `crates/auto-lang/src/ast/types.rs` - TypeDecl, Member
16. `crates/auto-lang/src/ast/union.rs` - Union
17. `crates/auto-lang/src/ast/use_.rs` - Use
18. `crates/auto-lang/src/ast/alias.rs` - Alias

---

## Reference: Existing ToAtom Implementations

See `docs/plans/002-to-atom-ast.md` for complete details on the original ToAtom implementation that this refactoring builds upon.

---

**Plan Status**: Ready for implementation
**Next Action**: User approval required
