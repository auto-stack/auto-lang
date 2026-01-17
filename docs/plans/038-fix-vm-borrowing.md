# Plan: Fix VM Borrowing for OOP Methods

## Problem Statement

The current VM infrastructure uses `RefCell<Universe>` to manage global state. When storing objects (like HashMap, HashSet, StringBuilder) and trying to mutate them through method calls, we encounter borrowing issues:

```rust
// Current problematic code in VM methods
pub fn hash_map_insert_str(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    let id = inst.fields.get("id");
    if let Some(Value::USize(id)) = id {
        let mut uni = uni.borrow_mut();  // OK - can get mutable borrow
        let b = uni.get_vmref(id);         // ERROR - returns Ref<'_, Box<dyn Any>>
        if let Some(b) = b {
            if let Some(mut map) = b.downcast_ref::<HashMapData>() {
                map.data.insert(...);  // ERROR - can't mutably borrow through Ref
            }
        }
    }
}
```

**Root Cause**: `get_vmref()` returns `Ref<'_, Box<dyn Any>>`, which creates an immutable borrow that cannot be made mutable.

## Analysis

### Current VM Data Storage

Located in `crates/auto-lang/src/universe.rs`:

```rust
pub struct Universe {
    vmrefs: HashMap<usize, Box<dyn Any>>,  // No interior mutability
}
```

### Current Borrowing Model

1. **Static methods** (like `HashMap.new()`): Work fine because they create new instances
2. **Instance methods** (like `map.insert()`): **BROKEN** - cannot mutate stored data

## Solutions

### Option 1: Add Interior Mutability (RECOMMENDED)

Wrap each VM data structure in `RefCell` or `Mutex`:

```rust
pub struct Universe {
    vmrefs: HashMap<usize, RefCell<Box<dyn Any>>>,
}

// VM method implementation
pub fn hash_map_insert_str(uni: Shared<Universe>, instance: &mut Value, args: Vec<Value>) -> Value {
    let id = inst.fields.get("id");
    if let Some(Value::USize(id)) = id {
        let uni = uni.borrow();
        let b = uni.get_vmref(id);
        if let Some(b) = b {
            // Now we can get mutable reference via RefCell
            let map = b.borrow_mut().downcast::<HashMapData>().unwrap();
            map.data.insert(...);
        }
    }
}
```

**Pros**:
- Follows Rust idioms for interior mutability
- Safe (runtime borrow checking)
- Minimal changes to VM architecture

**Cons**:
- Runtime borrow checking overhead (minimal for this use case)
- Need to ensure no overlapping borrows

### Option 2: Clone-Modify-Store

For each mutation operation:

```rust
pub fn hash_map_insert_str(...) -> Value {
    let id = inst.fields.get("id");
    if let Some(Value::USize(id)) = id {
        let mut uni = uni.borrow_mut();
        // Clone the data
        let old = uni.get_vmref(id).unwrap();
        let mut map = old.downcast::<HashMapData>().unwrap();
        map.data.insert(...);
        // Store back
        uni.set_vmref(id, Box::new(map));
    }
}
```

**Pros**:
- Simple to implement
- No runtime borrow checking overhead
- Works with current Universe structure

**Cons**:
- Inefficient (clones entire data structure on each mutation)
- Increases memory usage
- Need to implement `set_vmref()` in Universe

### Option 3: Use Raw Pointers (Unsafe)

```rust
pub fn hash_map_insert_str(...) -> Value {
    let id = inst.fields.get("id");
    if let Some(Value::USize(id)) = id {
        let uni = uni.borrow();
        let map = uni.get_vmref_raw(id) as *mut HashMapData; // UNSAFE
        (*map).data.insert(...);
    }
}
```

**Pros**:
- No overhead
- Zero-cost abstraction
- Similar to how C would handle this

**Cons**:
- **UNSAFE** - must ensure no aliasing
- Difficult to guarantee safety
- Last resort

## Implementation Plan

### Phase 1: Add Interior Mutality (1-2 hours)

**Files to modify**:
- `crates/auto-lang/src/universe.rs` - Wrap `vmrefs` in `RefCell`
- `crates/auto-lang/src/vm/collections.rs` - Update to use `borrow_mut()`
- `crates/auto-lang/src/vm/builder.rs` - Update to use `borrow_mut()`

**Steps**:

1. Update `Universe` structure:
```rust
pub struct Universe {
    vmrefs: RefCell<HashMap<usize, Box<dyn Any>>>,
}
```

2. Update all VM method implementations to use `borrow_mut()`:

```rust
// Before
let b = uni.get_vmref(id);
if let Some(b) = b {
    if let Some(mut map) = b.downcast_ref::<HashMapData>() {
        map.data.insert(...);
    }
}

// After
let b = uni.get_vmref(id).unwrap();
let mut map = b.borrow_mut().downcast::<HashMapData>().unwrap();
map.data.insert(...);
```

3. Add `borrow_mut()` method to Universe if it doesn't exist.

### Phase 2: Update All VM Methods (1 hour)

**Files to modify**:
- `vm/collections.rs` - Update all HashMap/HashSet methods
- `vm/builder.rs` - Update all StringBuilder methods
- Test compilation
- Run unit tests

### Phase 3: Add Missing VM Methods (1 hour)

**Check for missing methods**:
- HashMap: `insert_str`, `insert_int`, `get_str`, `get_int`, `contains`, `remove`, `size`, `clear`, `drop`
- HashSet: `insert`, `contains`, `remove`, `size`, `clear`, `drop`
- StringBuilder: `append`, `append_char`, `append_int`, `build`, `clear`, `len`, `drop`, `new`, `new_with_default`

**Ensure all methods handle**:
- Proper error checking
- Type conversions
- Argument parsing

### Phase 4: Testing (1 hour)

**Test execution**:
1. Compile without errors
2. Run unit tests (19 tests total)
3. Verify HashMap operations work
4. Verify HashSet operations work
5. Verify StringBuilder operations work

**Test cases**:
- Create instance
- Mutate operations
- Query operations
- Lifecycle (drop)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Borrow panics** | High | Careful testing of RefCell usage |
| **Memory leaks** | Medium | Ensure drop() methods work correctly |
| **Type safety** | Low | Downcast is safe due to VM registration |
| **Performance** | Low | RefCell overhead is minimal for this use case |

## Success Criteria

- ✅ All 19 OOP unit tests pass
- ✅ HashMap insert/get/remove/clear operations work
- ✅ HashSet insert/contains/remove/clear operations work
- ✅ StringBuilder append/build/clear/len operations work
- ✅ No compilation errors
- ✅ No runtime panics
- ✅ No memory leaks

## Estimated Time

- **Phase 1**: 1-2 hours
- **Phase 2**: 1 hour
- **Phase 3**: 1 hour
- **Phase 4**: 1 hour
- **Total**: 4-5 hours

## Related Plans

- [OOP.md](../design/OOP.md) - OOP design principles
- [021-single-inheritance.md](./021-single-inheritance.md) - Single inheritance implementation
- [019-spec-trait-system.md](./019-spec-trait-system.md) - Spec trait system

## Notes

- This fix is critical for the OOP system to work at runtime
- Current type declarations are correct, only VM implementation needs fixing
- The parsing tests work, which validates the API design
- After this fix, the OOP API will be fully functional
