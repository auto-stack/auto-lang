# Unify body, body_ref, and nodes into `kids`

**Objective**: Simplify Node structure by unifying three child-related fields (`body`, `body_ref`, `nodes`) into a single `kids` field using IndexMap.

## Current State Analysis

### Existing Fields

1. **`nodes: Vec<Node>`** (crates/auto-val/src/node.rs:118)
   - Simple vector of child nodes
   - Used for immediate child nodes
   - No type distinction (all nodes mixed together)

2. **`body: NodeBody`** (crates/auto-val/src/node.rs:116)
   - `IndexMap<ValueKey, NodeItem>` where `NodeItem` enum:
     - `Prop(Pair)` - Property definitions
     - `Node(Node)` - Child node definitions
   - Can hold both properties and nodes
   - Maintains insertion order
   - Used for parsed/evaluated node bodies

3. **`body_ref: MetaID`** (crates/auto-val/src/node.rs:117)
   - Reference to lazily evaluated body in universe
   - Points to `Meta::Body` in global scope
   - Used for deferred evaluation (LAZY tempo mode)

### Current Usage Patterns

```rust
// nodes field
self.nodes.push(node);              // Add child node
self.nodes.iter()                    // Iterate children
node.nodes.len()                     // Count children

// body field  
self.body.add_prop(key, value);     // Add property to body
self.body.add_kid(node);             // Add node to body
self.body.is_empty()                 // Check if body has content
self.body.get_prop_of(key);          // Get property from body

// body_ref field
self.body_ref != MetaID::Nil         // Check if has lazy reference
write!(f, "{}", self.body_ref)       // Display reference
```

### Problems with Current Design

1. **Redundancy**: Three different ways to store children
2. **Confusion**: Unclear when to use `nodes` vs `body`
3. **Type Mixing**: `body` can contain both props and nodes, but props should be in `props` field
4. **Inefficiency**: Converting between `nodes` and `body` (see `fill_node_body()`)
5. **Complex Display Logic**: Display code checks all three fields separately

## Proposed Solution

### New Design: Single `kids` Field

```rust
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub num_args: usize,
    pub args: Args,           // DEPRECATED: Will be removed later
    props: Obj,              // Properties (including args with num_args tracking)
    kids: Kids,              // NEW: Unified children storage
    pub text: AutoStr,
    pub body_ref: MetaID,    // Keep for lazy evaluation references
}
```

Where `Kids` is:

```rust
pub struct Kids {
    // Primary storage: ordered map of children
    // Key can be:
    // - AutoStr (explicit node ID/name)
    // - ValueKey::Str (string key)
    // - ValueKey::Int (numeric key)
    // - ValueKey::Bool (boolean key)  
    map: IndexMap<ValueKey, Kid>,
    
    // Lazy reference (kept separate for efficiency)
    lazy: Option<MetaID>,
}

pub enum Kid {
    Node(Node),
    Lazy(MetaID),
}
```

### Why This Design Works

1. **Single Source**: All children in one IndexMap
2. **Ordered**: Preserves definition order (via IndexMap)
3. **Type-Safe**: `Kid` enum distinguishes eager vs lazy children
4. **Efficient**: O(1) lookups by key
5. **Clear Semantics**: Props in `props` field, children in `kids` field

## Implementation Plan

### Phase 1: Data Structure (Day 1)

#### Step 1.1: Create Kids and Kid types

**File**: `crates/auto-val/src/kids.rs` (NEW)

```rust
use indexmap::IndexMap;
use crate::*;

/// Child storage for Node
#[derive(Debug, Clone, PartialEq)]
pub struct Kids {
    map: IndexMap<ValueKey, Kid>,
    lazy: Option<MetaID>,
}

impl Kids {
    pub fn new() -> Self {
        Self {
            map: IndexMap::new(),
            lazy: None,
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.map.is_empty() && self.lazy.is_none()
    }
    
    pub fn len(&self) -> usize {
        self.map.len()
    }
    
    pub fn iter(&self) -> impl Iterator<Item = (&ValueKey, &Kid)> {
        self.map.iter()
    }
    
    pub fn add_node(&mut self, key: impl Into<ValueKey>, node: Node) {
        self.map.insert(key.into(), Kid::Node(node));
    }
    
    pub fn add_lazy(&mut self, key: impl Into<ValueKey>, meta: MetaID) {
        self.map.insert(key.into(), Kid::Lazy(meta));
    }
    
    pub fn set_lazy_ref(&mut self, meta: MetaID) {
        self.lazy = Some(meta);
    }
    
    pub fn get(&self, key: &ValueKey) -> Option<&Kid> {
        self.map.get(key)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kid {
    Node(Node),
    Lazy(MetaID),
}
```

#### Step 1.2: Update Node struct

**File**: `crates/auto-val/src/node.rs`

```rust
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub num_args: usize,
    pub args: Args,           // DEPRECATED
    props: Obj,
    kids: Kids,              // NEW: Unified children storage
    pub text: AutoStr,
    pub body_ref: MetaID,    // DEPRECATED: Use kids.lazy instead
}
```

### Phase 2: API Methods (Day 1)

#### Step 2.1: Add Kids API methods

```rust
impl Node {
    // Kids management
    pub fn add_kid(&mut self, node: Node) {
        // Use node's ID as key, or auto-generate
        let key = if !node.id.is_empty() {
            ValueKey::Str(node.id.clone())
        } else {
            ValueKey::Str(node.name.clone())
        };
        self.kids.add_node(key, node);
    }
    
    pub fn add_kid_with_key(&mut self, key: impl Into<ValueKey>, node: Node) {
        self.kids.add_node(key, node);
    }
    
    pub fn get_kid(&self, key: &ValueKey) -> Option<&Kid> {
        self.kids.get(key)
    }
    
    pub fn kids_iter(&self) -> impl Iterator<Item = (&ValueKey, &Kid)> {
        self.kids.iter()
    }
    
    pub fn kids_count(&self) -> usize {
        self.kids.len()
    }
    
    pub fn has_kids(&self) -> bool {
        !self.kids.is_empty()
    }
    
    // Migration helpers (temporary)
    pub fn fill_node_body(&mut self) -> &mut Self {
        // Migrate nodes -> kids
        for node in self.nodes.drain(..) {
            self.add_kid(node);
        }
        
        // Migrate body -> kids
        for (key, item) in self.body.map.drain() {
            match item {
                NodeItem::Node(node) => self.add_kid_with_key(key.clone(), node),
                NodeItem::Prop(pair) => {
                    // Props should be in props field, not kids
                    self.props.set(pair.key, pair.value);
                }
            }
        }
        
        self
    }
}
```

### Phase 3: Update Display (Day 2)

```rust
impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.id.is_empty() {
            write!(f, " {}", self.id)?;
        }
        
        // Display args (from unified props)
        // ... existing code ...
        
        // Display kids
        if !self.kids.is_empty() {
            write!(f, " {{")?;
            for (i, (key, kid)) in self.kids.iter().enumerate() {
                match kid {
                    Kid::Node(node) => {
                        write!(f, "{}", node)?;
                    }
                    Kid::Lazy(meta) => {
                        write!(f, "@{}", meta)?;
                    }
                }
                if i < self.kids.len() - 1 {
                    write!(f, "; ")?;
                }
            }
            write!(f, "}}")?;
        }
        
        // Display lazy reference (if any)
        if let Some(ref lazy_ref) = self.kids.lazy {
            write!(f, " @{}", lazy_ref)?;
        }
        
        Ok(())
    }
}
```

### Phase 4: Update Downstream Code (Day 2-3)

#### Files to Update:

1. **eval.rs** (~20 changes)
   - Line 2315: `nodes.push(node)` → `nd.add_kid(node)`
   - Line 2426: `nd.nodes = nodes` → remove
   - Line 2430: `nd.body_ref = body` → `nd.kids.set_lazy_ref(body)`

2. **universe.rs** (~30 changes)
   - Update `deref_val()` to handle `kids` field
   - Copy `kids.map` when creating new node

3. **parser.rs** (~5 changes)
   - Already uses `nodes` vector, no changes needed initially

4. **transpiler** (~10 changes)
   - Update C/Rust transpilation to handle new structure

### Phase 5: Remove Old Fields (Day 3)

#### Step 5.1: Mark old fields as deprecated

```rust
pub struct Node {
    pub name: AutoStr,
    pub id: AutoStr,
    pub num_args: usize,
    #[deprecated(note = "Use unified kids API instead")]
    pub nodes: Vec<Node>,       // DEPRECATED
    #[deprecated(note = "Use unified kids API instead")]
    pub body: NodeBody,         // DEPRECATED
    #[deprecated(note = "Use kids.lazy instead")]
    pub body_ref: MetaID,       // DEPRECATED
    props: Obj,
    kids: Kids,                 // NEW: Use this
    pub text: AutoStr,
}
```

#### Step 5.2: Remove deprecated fields (Future)

After all downstream code migrated:
- Remove `nodes`, `body`, `body_ref` fields
- Remove `NodeBody` and `NodeItem` types
- Clean up imports

### Phase 6: Testing (Day 3-4)

#### Test Categories

1. **Kids API Tests** (15 tests)
   - `test_add_kid()`
   - `test_add_kid_with_key()`
   - `test_get_kid()`
   - `test_kids_iter()`
   - `test_kids_order_preserved()`

2. **Migration Tests** (10 tests)
   - `test_fill_node_body_from_nodes()`
   - `test_fill_node_body_from_body()`
   - `test_fill_node_body_mixed()`

3. **Integration Tests** (20 tests)
   - All existing tests should pass
   - Display output tests
   - Eval tests with kids
   - Parser tests

## Success Criteria

- ✅ All 285+ auto-lang tests pass
- ✅ All 40+ auto-val tests pass
- ✅ Zero compiler warnings
- ✅ Kids field properly stores all children
- ✅ Display correctly shows kids
- ✅ Performance maintained or improved
- ✅ Code simplified (fewer fields)

## Estimated Timeline

- **Phase 1**: Day 1 (4-6 hours)
- **Phase 2**: Day 1 (2-3 hours)
- **Phase 3**: Day 2 (2-3 hours)
- **Phase 4**: Day 2-3 (6-8 hours)
- **Phase 5**: Day 3 (4-6 hours)
- **Phase 6**: Day 3-4 (4-6 hours)

**Total**: 3-4 days

## Key Design Decisions

### Decision 1: Keep `body_ref` temporarily

**Why**: Some code still checks `body_ref != MetaID::Nil`. We'll keep it but deprecate it in favor of `kids.lazy`.

**Future**: Remove entirely once all code uses `kids.lazy`.

### Decision 2: Kid enum with Node/Lazy variants

**Why**: Type-safe way to distinguish eager nodes from lazy references.

**Alternative considered**: Store all as `Value` and check type at runtime. Rejected because less type-safe.

### Decision 3: IndexMap with ValueKey keys

**Why**: 
- Consistent with `props` field design
- Supports both string keys and numeric keys
- Preserves insertion order
- O(1) lookups

**Alternative considered**: Vec<Kid> with separate key tracking. Rejected because harder to use.

### Decision 4: Two-phase migration

**Why**: 
1. Add `kids` field alongside old fields
2. Migrate code incrementally
3. Remove old fields once all code updated

**Risk**: Temporary storage overhead (acceptable for migration period).

## Open Questions

1. **Q**: Should we auto-generate keys for nodes without IDs?
   **A**: Yes, use `node.name` as key, or auto-increment

2. **Q**: How to handle key collisions?
   **A**: IndexMap will overwrite. This is consistent with current behavior.

3. **Q**: Should we support removing kids?
   **A**: Yes, add `remove_kid(key)` method

4. **Q**: What about lazy kids - should they be stored in map or separately?
   **A**: In map with `Kid::Lazy` variant, plus optional `kids.lazy` for reference

## Next Steps

1. **Review and approve this plan**
2. **Create kids.rs module**
3. **Start Phase 1 implementation**
