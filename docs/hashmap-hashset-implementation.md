# HashMap and HashSet for AutoVM

## Overview

HashMap and HashSet are now fully implemented in AutoVM using Rust's standard library collections (`std::collections::HashMap` and `std::collections::HashSet`).

## Files Created

### 1. Type Definitions

**`stdlib/auto/hashmap.at`** (3.1 KB)
- Complete type definition for `HashMap<K, V>`
- All public methods with `#[vm]` annotations
- Comprehensive documentation and usage examples

**`stdlib/auto/hashset.at`** (2.2 KB)
- Complete type definition for `HashSet<T>`
- All public methods with `#[vm]` annotations
- Comprehensive documentation and usage examples

### 2. Prelude Integration

**`stdlib/auto/prelude.at`** (updated)
- Added `use auto.hashmap: HashMap`
- Added `use auto.hashset: HashSet`
- HashMap and HashSet now available without explicit imports

### 3. Test Suite

**`crates/auto-lang/src/vm/tests_collections.rs`** (new)
- 12 comprehensive tests covering:
  - Basic insertion and retrieval
  - Integer keys
  - Contains/missing keys
  - Size tracking
  - Remove operations
  - Clear operations
- Tests for both HashMap and HashSet

## Implementation Pattern

This follows the **canonical AutoVM pattern** for Rust-backed types:

```rust
// 1. Data Wrapper (vm/collections.rs)
pub struct HashMapData {
    pub data: StdHashMap<String, Value>,  // Rust's HashMap!
}

// 2. VM Method Signature
pub fn hash_map_insert(
    _evaler: &mut Evaler,    // VM context
    instance: &mut Value,     // The HashMap instance
    args: Vec<Value>         // Method arguments
) -> Value

// 3. Type Definition (.at file)
type HashMap<K, V> {
    #[vm, pub]
    static fn new() HashMap<K, V>

    #[vm, pub]
    fn insert_str(key str, value V)

    // ... more methods
}

// 4. Registration (vm.rs)
hashmap_type.methods.insert(
    "insert_str".into(),
    collections::hash_map_insert_str as VmMethod,
);
```

## Available Methods

### HashMap<K, V>

**Static Methods:**
- `HashMap.new()` - Create empty map

**Insertion:**
- `insert_str(key, value)` - Insert with string key
- `insert_int(key, value)` - Insert with integer key

**Lookup:**
- `get_str(key)` - Get value by string key (returns nil if not found)
- `get_int(key)` - Get value by integer key (returns nil if not found)
- `contains(key)` - Check if key exists (returns bool)

**Modification:**
- `remove(key)` - Remove key-value pair
- `clear()` - Clear all entries

**Query:**
- `size()` - Get number of entries

**Lifecycle:**
- `drop()` - Free resources

### HashSet<T>

**Static Methods:**
- `HashSet.new()` - Create empty set

**Modification:**
- `insert(value)` - Insert element
- `remove(value)` - Remove element
- `clear()` - Clear all elements

**Query:**
- `contains(value)` - Check membership (returns bool)
- `size()` - Get number of elements

**Lifecycle:**
- `drop()` - Free resources

## Usage Examples

### HashMap Example

```auto
// Create a new HashMap
let map = HashMap.new()

// Insert key-value pairs
map.insert_str("name", "Alice")
map.insert_str("age", 30)
map.insert_str("city", "NYC")

// Get values
let name = map.get_str("name")        // Returns "Alice"
let age = map.get_str("age")          // Returns 30
let missing = map.get_str("missing")  // Returns nil

// Check if key exists
let has_age = map.contains("age")     // Returns true
let has_zip = map.contains("zip")     // Returns false

// Get size
let count = map.size()                // Returns 3

// Remove entry
map.remove("city")
let count_after = map.size()         // Returns 2

// Clear all entries
map.clear()
let count_final = map.size()         // Returns 0

// Clean up
map.drop()
```

### HashSet Example

```auto
// Create a new HashSet
let set = HashSet.new()

// Insert elements
set.insert("apple")
set.insert("banana")
set.insert("cherry")

// Check membership
let has_apple = set.contains("apple")   // Returns true
let has_grape = set.contains("grape")   // Returns false

// Get size
let count = set.size()                  // Returns 3

// Remove element
set.remove("banana")
let count_after = set.size()            // Returns 2

// Clear all elements
set.clear()
let count_final = set.size()            // Returns 0

// Clean up
set.drop()
```

## Integration with Existing Code

HashMap and HashSet integrate seamlessly with existing AutoVM infrastructure:

- **Memory Management**: Uses VM reference counting via `universe`
- **Type System**: Values can be any AutoLang type (int, str, bool, List, etc.)
- **Method Dispatch**: Registered in VM registry for fast lookup
- **Storage**: Backed by Rust's efficient `std::collections::HashMap`

## Future Enhancements

Potential improvements for future work:

1. **Generic Keys**: Currently uses String keys, could support generic key types
2. **Iterators**: Add `keys()`, `values()`, `iter()` methods
3. **Additional Methods**: `is_empty()`, `reserve()`, `shrink_to_fit()`
4. **Entry API**: `entry()` method for more efficient insert-or-update
5. **Merge Operations**: `extend()` to merge multiple maps/sets

## Testing

Run the test suite:

```bash
cargo test -p auto-lang vm::tests_collections
```

Tests cover:
- ✅ Basic insertion and retrieval
- ✅ Integer keys (for HashMap)
- ✅ Contains/missing keys
- ✅ Size tracking
- ✅ Remove operations
- ✅ Clear operations

## Architecture Benefits

This implementation demonstrates the **strength of AutoVM's architecture**:

1. **Zero-Cost Abstraction**: Direct use of Rust's optimized collections
2. **Type Safety**: Rust's type system ensures memory safety
3. **Performance**: O(1) average case for all operations
4. **Simplicity**: Clean separation between AutoLang types and Rust implementations
5. **Extensibility**: Easy to add new collection types following this pattern

## Related Files

- **Implementation**: `crates/auto-lang/src/vm/collections.rs` (371 lines)
- **Registration**: `crates/auto-lang/src/vm.rs` (init_collections_module)
- **Storage**: `crates/auto-lang/src/universe.rs` (VmRefData enum)
- **Similar Types**:
  - `stdlib/auto/list.at` - List<T, S> with pluggable storage
  - `stdlib/auto/storage.at` - Storage strategies
