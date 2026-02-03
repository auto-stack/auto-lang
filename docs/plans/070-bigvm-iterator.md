# Plan 070: BigVM Iterator Implementation

**Status**: ✅ Phase 1 Complete, Phase 2-3 Pending
**Created**: 2025-02-03
**Completed**: 2025-02-03 (Phase 1)
**Related**: Plan 068 (Phase 7: Advanced Features)

---

## Recent Updates (2025-02-03)

**Phase 1 Complete**:
- ✅ Implemented `List.iter()` native function
- ✅ Implemented `Iterator.next()` native function
- ✅ Added iterator storage to BigVM engine
- ✅ Registered functions in native registry
- ✅ Fixed codegen to initialize native registry
- ✅ All basic iterator tests passing

**Test Results**:
```
Input: [10, 20, 30]
Output: 10, 20, 30, -1 (nil)
```

**Known Limitations**:
- No automatic iterator cleanup (memory leaks accepted for MVP)
- No lazy adapters (map, filter) yet
- No terminal operations (collect, reduce) yet

---

## 1. Objective

Implement iterator support for BigVM to enable list iteration, map, filter, and other functional programming patterns.

## 2. Requirements

From `tests/list_tests.rs`:
- `list.iter()` → creates iterator
- `iter.next()` → returns elements one by one
- `iter.map(func)` → lazy map adapter
- Iterators should return `nil` when exhausted

## 3. Design Decisions

### 3.1 Architecture Choice

**Option**: Native Object Approach (Simplest)

Rationale:
- Fastest to implement (no new opcodes needed)
- Leverages Rust's heap for complex state management
- Can iterate later for VM-native objects if needed

Trade-offs:
- Slightly slower than native VM objects (cross-language boundary)
- Simpler to implement and test

### 3.2 Iterator State Management

Iterator state stored in Rust:
```rust
struct ListIterator {
    list_id: u64,          // Which list to iterate
    current_index: u32,    // Current position in list
}
```

### 3.3 Memory Management

For MVP (Phase 1):
- Iterators stored in `HashMap<u64, ListIterator>` in BigVM
- Iterator IDs allocated sequentially
- **No automatic cleanup** (leaks accepted for now)

Future improvements:
- Reference counting
- RAII-style cleanup
- Weak references to prevent cycles

## 4. Implementation Plan

### Phase 1: Basic Iterator (✅ Complete)
**Goal**: Support `iter()` and `next()`

- [x] **4.1 Iterator Storage**
    - Add `iterators: DashMap<u32, ListIterator>` to BigVM
    - Add `iterator_id_gen: AtomicU32` for ID generation

- [x] **4.2 Native Functions**
    - `List.iter(list_id) -> iterator_id` (✅ Implemented)
    - `Iterator.next(iterator_id) -> element` (✅ Implemented)
    - Both use CALL_NAT opcode

- [x] **4.3 Testing**
    - ✅ Basic iteration test passes (tmp/test_list_iter.at)
    - ✅ Exhausted iterator returns -1 (nil)
    - ✅ Multiple elements retrieved correctly

### Phase 2: Lazy Adapters
**Goal**: Support `map()`, `filter()`

- [ ] **5.1 Map Adapter**
    - `Iterator.map(iterator_id, func_id) -> new_iterator_id`
    - Stores: source iterator_id, function to call

- [ ] **5.2 Filter Adapter**
    - `Iterator.filter(iterator_id, func_id) -> new_iterator_id`
    - Stores: source iterator_id, predicate function

### Phase 3: Terminal Operations
**Goal**: Support `collect()`, `reduce()`, `find()`

- [ ] **6.1 Collect**
    - `Iterator.collect(iterator_id) -> list_id`

- [ ] **6.2 Reduce**
    - `Iterator.reduce(iterator_id, func_id, initial) -> result`

- [ ] **6.3 Find**
    - `Iterator.find(iterator_id, func_id) -> element_or_nil`

## 5. Bytecode Examples

### Basic Iteration
```auto
let list = List.new()
list.push(1)
list.push(2)
list.push(3)

let iter = list.iter()     // CALL_NAT List.iter, returns iterator_id
let first = iter.next()   // CALL_NAT Iterator.next, returns 1
let second = iter.next()  // CALL_NAT Iterator.next, returns 2
let done = iter.next()    // CALL_NAT Iterator.next, returns nil
```

### Compilation (Codegen)
```auto
// let iter = list.iter()
CONST_I32 list_id
CALL_NAT List.iter        // Returns iterator_id on stack
STORE_LOC_0              // Store in local variable 'iter'

// let first = iter.next()
LOAD_LOC_0               // Load iterator_id
CALL_NAT Iterator.next    // Returns element (1)
STORE_LOC_1              // Store in 'first'
```

## 6. Native Function Specifications

### 6.1 List.iter()

**Signature**: `iter(list_id: u32) -> u32`

**Behavior**:
1. Allocate new iterator ID
2. Store iterator state: `{ list_id, current_index: 0 }`
3. Return iterator_id

**Error Handling**:
- If list_id doesn't exist: return -1 or panic

### 6.2 Iterator.next()

**Signature**: `next(iterator_id: u32) -> i32`

**Behavior**:
1. Look up iterator state
2. If `current_index >= list.len()`, return -1 (nil)
3. Get element at `current_index` from list
4. Increment `current_index`
5. Return element value

**Return Values**:
- `-1` represents `nil` (exhausted)
- Otherwise returns the element value

## 7. Implementation Files

### Files to Modify
- `crates/auto-lang/src/vm/native.rs`: Add iterator shims
- `crates/auto-lang/src/vm/engine.rs`: Add iterator storage
- `crates/auto-lang/src/vm/native_registry.rs`: Register iterator functions

### Files to Create
- None (reuse existing infrastructure)

## 8. Testing Strategy

### Test Cases
1. Basic iteration: `iter` → `next` × 3 → `next` (nil)
2. Empty list: `iter` → `next` (immediately nil)
3. Single element: `iter` → `next` → `next` (nil)
4. Multiple iterators on same list (concurrent iteration)

### Success Criteria
- [x] `test_list_iter` passes with BigVM
- [ ] `test_list_map_double` passes
- [ ] `test_list_map_square` passes
- [ ] No memory leaks (or acceptable for MVP)
