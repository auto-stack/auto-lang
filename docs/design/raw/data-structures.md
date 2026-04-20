# Data Structures (Rust Implementation)

> Extracted from CLAUDE.md for reference. See CLAUDE.md for rules and quick reference.

### Node and NodeBody

The `Node` and `NodeBody` structures (in `crates/auto-val/src/node.rs`) use `IndexMap` for efficient, ordered storage of properties and child nodes.

**Key Implementation Details**:

- **IndexMap**: Uses `indexmap::IndexMap` instead of `BTreeMap` or `HashMap`
  - Provides O(1) lookups (better than BTreeMap's O(log n))
  - Preserves insertion order (unlike HashMap)
  - Eliminates need for separate index tracking

- **NodeBody Structure**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct NodeBody {
    pub map: IndexMap<ValueKey, NodeItem>,
}
```

- **Insertion Order Preservation**: Properties and children maintain insertion order
  - Display/serialization shows items in insertion order
  - No manual index synchronization needed
  - Tests verify order is preserved across operations

- **Usage Patterns**:
```rust
// Adding properties preserves order
node.set_prop("zebra", 1);  // Added first
node.set_prop("apple", 2);  // Added second
// Display shows: zebra first, then apple (not alphabetical)

// Adding children preserves order
node.add_kid(Node::new("kid1"));
node.add_kid(Node::new("kid2"));
// Iteration returns: kid1, then kid2
```

- **Performance Characteristics**:
  - Lookup: O(1) average case
  - Insertion: O(1) average case
  - Iteration: O(n) in insertion order
  - Memory: Single IndexMap instead of BTreeMap + Vec

### Obj Structure

The `Obj` structure (in `crates/auto-val/src/obj.rs`) also uses `IndexMap` for the same reasons:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Obj {
    values: IndexMap<ValueKey, Value>,
}
```

**Important**: IndexMap cannot be used in const contexts, so:
- `Obj::new()` is not const
- Use `OnceLock` for static Obj instances (see `value.rs:node_nil()` and `obj_empty()`)
- Removed `Obj::EMPTY` constant; use `Obj::new()` instead

### ListData Structure

The `ListData` structure (in `crates/auto-lang/src/universe.rs`) implements dynamic lists similar to Rust's `Vec<T>`:

```rust
#[derive(Debug)]
pub struct ListData {
    pub elems: Vec<Value>,
}
```

**Syntax**: `List` type transpiles to backend-specific types:
- C: `list_T*` (wrapper around dynamic array)
- Python: `list`
- Rust: `Vec<T>`

**Creation**: `let list = List.new()`

**Methods**:
- `push(elem)` - Add element to end
- `pop()` - Remove and return last element
- `len()` - Return number of elements
- `is_empty()` - Return 1 if empty, 0 otherwise
- `clear()` - Remove all elements
- `get(index)` - Get element at index (0-based)
- `set(index, elem)` - Set element at index
- `insert(index, elem)` - Insert element at index
- `remove(index)` - Remove and return element at index
- `reserve(additional)` - Reserve capacity for additional elements

**Example Usage**:
```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    let len = list.len()  // Returns 3
    let first = list.get(0)  // Returns 1

    list.set(0, 10)
    let updated = list.get(0)  // Returns 10

    list.insert(1, 5)  // [10, 5, 2, 3]

    let removed = list.remove(1)  // Returns 5, list is [10, 2, 3]
    let popped = list.pop()  // Returns 3, list is [10, 2]
}
```

---

### Storage-Based Lists (Plan 052)

AutoLang supports **storage-agnostic lists** with pluggable storage strategies via `List<T, S>`:

**Syntax**: `List<T, StorageStrategy>` where:
- `T` - Element type
- `S` - Storage strategy (implements `Storage<T>` spec)

**Available Storage Strategies**:

#### 1. Heap Storage (`List<T, Heap>`) - PC/Server

**Use Case**: Desktop applications, servers, any environment with heap allocator

```auto
let list List<int, Heap> = List.new()
list.push(1)
list.push(2)
list.push(3)

// Dynamic growth (unlimited capacity)
for i in 0..1000 {
    list.push(i)
}

let cap = list.capacity()  // Returns current capacity
```

**Characteristics**:
- Dynamic growth via malloc/realloc
- Limited only by available memory
- try_grow() may fail (returns false) if OOM
- Works with any type `T`

#### 2. Inline Storage (`List<T, InlineInt64>`) - MCU/Embedded

**Use Case**: Microcontrollers, embedded systems, no heap allocator

```auto
let list List<int, InlineInt64> = List.new()
list.push(1)
list.push(2)
list.push(3)

let cap = list.capacity()  // Returns 64 (fixed)
```

**Characteristics**:
- Zero heap usage (all on stack)
- Deterministic memory usage
- Fixed 64-element capacity (hard limit)
- Currently `int`-only (future: `Inline<T, N>`)

**Storage Spec** (`Storage<T>`):

```auto
spec Storage<T> {
    fn data() *T              // Get raw pointer to buffer
    fn capacity() u32         // Get physical capacity
    fn try_grow(min_cap u32) bool  // Try to grow
}
```

**When to Use Which**:

| Scenario | Storage Strategy | Reason |
|----------|-----------------|--------|
| Server application | `List<T, Heap>` | Dynamic growth, plenty of memory |
| Desktop GUI | `List<T, Heap>` | User data, unknown size |
| Game engine | `List<T, Heap>` | Entity lists, dynamic objects |
| **Microcontroller** | `List<T, InlineInt64>` | No heap, limited RAM |
| **Sensor readings** | `List<T, InlineInt64>` | Fixed buffer size, deterministic |
| **Real-time system** | `List<T, InlineInt64>` | No allocation failures |

**See Also**:
- [Plan 052 Implementation Summary](plan-052-implementation-summary.md) - Complete technical details
- [Storage Usage Guide](../stdlib/auto/default_storage.at) - Recommended patterns
