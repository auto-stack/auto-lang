# Unify Args and Props: IndexMap Boundary Approach

**Project**: auto-val Node structure simplification
**Current Version**: 0.2.0 (separate `Args` and `props`)
**Target Version**: 0.3.0 (unified `props` with `num_args` boundary)
**Estimated Duration**: 3-4 days
**Status**: Planning Phase

## Objective

Eliminate the separate `Args` structure and unify it with `props` using IndexMap's insertion order and a `num_args` boundary counter. This simplifies the Node structure while preserving the semantic distinction between arguments (provided at node instantiation) and body properties (defined in the node body).

## Current State Analysis

### Current Node Structure

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub args: Args,        // SEPARATE: Vec<Arg> with position/named variants
    props: Obj,            // SEPARATE: IndexMap of body properties
    pub nodes: Vec<Node>,
    pub text: AutoStr,
    pub body: NodeBody,
    pub body_ref: MetaID,
}

// Current Args structure (to be eliminated)
#[derive(Debug, Clone, PartialEq)]
pub struct Args {
    pub args: Vec<Arg>,
}

pub enum Arg {
    Pos(Value),           // Positional argument
    Name(Pair),           // Named argument (key: value)
}
```

### Current Usage Pattern

```auto
// In AutoLang code:
mynode(a: 1, b: 2) {        // a, b are args
    d: 4;
    e: 5;
}

// Creates:
node.args = [Arg::Name("a", 1), Arg::Name("b", 2)]
node.props = {"d": 4, "e": 5}
```

### Problem Statement

1. **Redundant storage**: Both `args` and `props` store key-value pairs
2. **Inconsistent access**: Different APIs for args vs props
3. **Duplicate lookups**: Need to check both `args` and `props` when getting values
4. **Complex code**: `get_arg()`, `get_prop()`, `main_arg()` all handle both cases
5. **Inefficient**: Maintains two separate data structures for similar data

## Proposed Solution: Option 3 - Boundary Counter

### New Node Structure

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub num_args: usize,     // NEW: Number of arg keys in props
    props: Obj,              // UNIFIED: Args first, then body props
    pub nodes: Vec<Node>,
    pub text: AutoStr,
    pub body: NodeBody,
    pub body_ref: MetaID,
}
```

### Data Layout

```
IndexMap (props):
┌─────────────┬───────┬──────────────┐
│ Index       │ Key   │ Source       │
├─────────────┼───────┼──────────────┤
│ 0           │ "a"   │ Arg          │
│ 1           │ "b"   │ Arg          │
│ ← boundary  │       │ num_args = 2 │
│ 2           │ "d"   │ Body prop    │
│ 3           │ "e"   │ Body prop    │
└─────────────┴───────┴──────────────┘
```

### API Changes

#### New Methods

```rust
impl Node {
    /// Check if a key is an arg (within num_args boundary)
    pub fn is_arg(&self, key: &str) -> bool {
        self.props.keys()
            .iter()
            .take(self.num_args)
            .any(|k| k.to_astr() == key)
    }
    
    /// Get arg value (only searches arg region)
    pub fn get_arg(&self, key: &str) -> Option<Value> {
        if self.is_arg(key) {
            self.props.get(key)
        } else {
            None
        }
    }
    
    /// Get body prop value (only searches body prop region)
    pub fn get_body_prop(&self, key: &str) -> Option<Value> {
        if !self.is_arg(key) {
            self.props.get(key)
        } else {
            None
        }
    }
    
    /// Iterate over args only
    pub fn args_iter(&self) -> impl Iterator<Item = (&ValueKey, &Value)> {
        self.props.iter().take(self.num_args)
    }
    
    /// Iterate over body props only
    pub fn body_props_iter(&self) -> impl Iterator<Item = (&ValueKey, &Value)> {
        self.props.iter().skip(self.num_args)
    }
    
    /// Get all arg keys
    pub fn arg_keys(&self) -> Vec<ValueKey> {
        self.props.keys().iter()
            .take(self.num_args)
            .cloned()
            .collect()
    }
    
    /// Get all body prop keys
    pub fn body_prop_keys(&self) -> Vec<ValueKey> {
        self.props.keys().iter()
            .skip(self.num_args)
            .cloned()
            .collect()
    }
    
    /// Add an arg (must be added before any body props)
    pub fn add_arg(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        // Invariant: args must come first
        let key = key.into();
        self.props.set(key.clone(), value);
        self.num_args += 1;
    }
    
    /// Add a body prop
    pub fn add_body_prop(&mut self, key: impl Into<ValueKey>, value: impl Into<Value>) {
        self.props.set(key, value);
    }
    
    /// Set main argument (updates or creates first arg)
    pub fn set_main_arg(&mut self, arg: impl Into<Value>) {
        if self.num_args == 0 {
            // Create first arg
            self.add_arg("", arg);
        } else {
            // Update first arg's value
            let first_key = self.props.keys().next().unwrap();
            self.props.set(first_key, arg);
        }
    }
}
```

#### Methods to Update

```rust
// OLD: Returns from args OR props
pub fn get_prop(&self, key: &str) -> Value {
    // Search args first, then props
}

// NEW: Still searches both but using unified props
pub fn get_prop(&self, key: &str) -> Value {
    self.props.get(key).unwrap_or(Value::Nil)
}

// OLD: Checks args for main arg
pub fn main_arg(&self) -> Value {
    if !self.args.is_empty() {
        // Get from args
    }
    // Fallback to props...
}

// NEW: Checks first element in props
pub fn main_arg(&self) -> Value {
    if self.num_args > 0 {
        self.props.values().next().unwrap().clone()
    } else {
        Value::Nil
    }
}

// OLD: Returns args title
pub fn title(&self) -> AutoStr {
    if self.args.is_empty() {
        self.name.clone()
    } else {
        format!("{}({})", self.name, self.args.args[0].to_string()).into()
    }
}

// NEW: Uses first arg
pub fn title(&self) -> AutoStr {
    if self.num_args == 0 {
        self.name.clone()
    } else {
        let first_arg = self.props.values().next().unwrap();
        format!("{}({})", self.name, first_arg).into()
    }
}
```

## Implementation Plan

### Phase 1: Preparation (Day 1)

**Step 1.1**: Add `num_args` field to Node
- Keep `args` field temporarily
- Initialize `num_args = 0` in all constructors
- Update `Debug` and `PartialEq` derives

**Step 1.2**: Add new API methods
- Implement `is_arg()`, `get_arg()`, `get_body_prop()`
- Implement `args_iter()`, `body_props_iter()`
- Implement `arg_keys()`, `body_prop_keys()`
- Add comprehensive unit tests

**Step 1.3**: Update constructor logic
- `Node::new()`: Set `num_args = 0`
- `Node::empty()`: Set `num_args = 0`
- Document invariant: args must be added first

### Phase 2: Parser Integration (Day 1-2)

**Step 2.1**: Update parser to use new API
- File: `crates/auto-lang/src/parser.rs`
- Find node instantiation code
- Replace `node.args.args.push(...)` with `node.add_arg(...)`
- Replace `node.props.set(...)` with `node.add_body_prop(...)`
- Maintain insertion order: args first, then props

**Step 2.2**: Update AST to Atom conversion
- File: `crates/auto-lang/src/ast/call.rs` or similar
- Ensure args are converted before body props
- Update `node_to_atom()` or similar functions

### Phase 3: Update Method Implementations (Day 2)

**Step 3.1**: Refactor `get_prop()` and `get_prop_of()`
- Simplify to single lookup in unified props
- Remove fallback logic between args/props

**Step 3.2**: Refactor `main_arg()`
- Use first element from props if num_args > 0
- Simplify logic

**Step 3.3**: Refactor `title()`
- Use first arg from props
- Simplify string formatting

**Step 3.4**: Update other arg/prop methods
- `has_prop()`: Check unified props
- `get_prop_names()`: Return all keys from props
- `props_iter()`: Return iterator over all props

### Phase 4: Update Downstream Code (Day 2-3)

**Step 4.1**: Update evaluator
- File: `crates/auto-lang/src/eval.rs`
- Replace `node.args.get_arg()` calls with `node.get_arg()`
- Replace direct `node.args.args` access with `node.args_iter()`

**Step 4.2**: Update universe
- File: `crates/auto-lang/src/universe.rs`
- Update any direct args access
- Use new iterator methods

**Step 4.3**: Update transpiler
- Files: `crates/auto-lang/src/trans/`
- Update C/Rust transpilation
- Generate code that respects arg/body prop distinction

**Step 4.4**: Update tests
- Fix test assertions that access `node.args`
- Update to use new API
- Add tests for boundary behavior

### Phase 5: Remove Args Structure (Day 3)

**Step 5.1**: Remove `Args` and `Arg` types
- File: `crates/auto-val/src/meta.rs` or similar
- Delete `Args` struct
- Delete `Arg` enum
- Remove from lib.rs exports

**Step 5.2**: Clean up imports
- Remove unused `Args` imports
- Remove unused `Arg` imports

**Step 5.3**: Update documentation
- Update CLAUDE.md with new Node structure
- Document the boundary pattern
- Add usage examples

### Phase 6: Testing and Validation (Day 3-4)

**Step 6.1**: Unit tests
- Test `is_arg()` with various keys
- Test `args_iter()` returns only args
- Test `body_props_iter()` returns only body props
- Test boundary conditions (num_args = 0, all args, all props)

**Step 6.2**: Integration tests
- Test node creation with args and props
- Test node serialization maintains order
- Test evaluator with arg access
- Test transpiler output

**Step 6.3**: Performance tests
- Verify no regression vs current implementation
- Benchmark unified access vs separate args/props

**Step 6.4**: Edge cases
- Node with no args (num_args = 0)
- Node with only args (no body props)
- Node with duplicate keys (arg and body prop with same name)
- Node modifications (add/remove args and props)

## Migration Guide

### For Library Users

**Before:**
```rust
let node = Node::new("mynode");
node.args.args.push(Arg::Name("a".into(), 1.into()));
node.props.set("d", 4);
```

**After:**
```rust
let mut node = Node::new("mynode");
node.add_arg("a", 1);  // Args must be added first
node.add_body_prop("d", 4);
```

### Access Patterns

**Before:**
```rust
// Check args first, then props
let value = node.args.get_arg("a")
    .or_else(|| node.props.get("a"))
    .unwrap_or(Value::Nil);
```

**After:**
```rust
// Unified access
let value = node.get_prop("a");  // Searches all props
```

**Distinguishing args from props:**
```rust
// Before
if node.args.args.iter().any(|a| matches!(a, Arg::Name(k, _) if k == "a")) {
    // it's an arg
}

// After
if node.is_arg("a") {
    // it's an arg
}
```

## Benefits

1. **Simplified structure**: One field instead of two
2. **Consistent API**: Single access pattern for all key-value pairs
3. **Better performance**: No need to check two structures
4. **Preserves semantics**: Args still distinguishable from body props
5. **Maintains order**: IndexMap preserves insertion order
6. **Type-safe**: Compile-time guarantees via methods

## Risks and Mitigation

### Risk 1: Breaking Changes

**Impact**: High - Direct `args` field access will break

**Mitigation**:
- Provide deprecated accessor methods during transition
- Clear migration guide
- Update all internal code first

### Risk 2: Insertion Order Bugs

**Impact**: Medium - Args added out of order break boundary

**Mitigation**:
- Add runtime invariant checks in debug builds
- Document clearly: "Args must be added before body props"
- Add `add_arg()` method that enforces this

### Risk 3: Performance Regression

**Impact**: Low - Should be same or better performance

**Mitigation**:
- Benchmark before/after
- Optimize hot paths if needed

## Success Criteria

- [ ] All tests passing (349+ tests)
- [ ] `Args` and `Arg` types removed
- [ ] No direct `args` field access in codebase
- [ ] `num_args` correctly maintained in all code paths
- [ ] Args and body props distinguishable via API
- [ ] Documentation updated
- [ ] Migration guide provided
- [ ] Zero compiler warnings

## Next Steps

1. **Review and approve this plan**
2. **Start Phase 1**: Add `num_args` field and new API
3. **Create tracking issue** for each phase
4. **Begin implementation**

---

**Plan Status**: Ready for Implementation
**Estimated Completion**: 3-4 days from approval
**Complexity**: Medium (requires careful coordination but well-scoped)
