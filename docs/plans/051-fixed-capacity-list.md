# Plan 051: Fixed-Capacity List<T> Implementation (Interim Solution)

## Objective

Implement a **fixed-capacity** `List<T>` in pure AutoLang as an interim solution, working within current language limitations (no runtime array allocation).

## Motivation

**Why Fixed Capacity?**
- **Current blocker**: Cannot create `[new_cap]T` arrays at runtime (Plan 052 will solve this)
- **Immediate need**: List<T> implementation that works NOW
- **Valid approach**: Fixed-capacity arrays are a reasonable interim solution
- **Foundation**: Can upgrade to dynamic capacity once Plan 052 is complete

**Current State**:
- `List<T>` is implemented as VM functions in Rust
- No AutoLang source implementation exists
- Array mutation works (Phase 1 testing confirmed)

**Desired State**:
- Pure AutoLang `List<T>` implementation in `stdlib/auto/list.at`
- Fixed capacity (e.g., 100 elements)
- All basic operations working: push, pop, get, set, len, is_empty, clear

**Benefits**:
1. **Self-hosting**: List implemented in AutoLang itself
2. **Transpilation**: Works with C transpiler (no VM dependency)
3. **Educational**: Shows how to build data structures in AutoLang
4. **Foundation**: Can upgrade to dynamic capacity later

## Phase 1 Results: Array Capability Testing

**Completed**: 8 comprehensive array tests created

| Test | Status | Finding |
|------|--------|---------|
| array_declaration | ✅ PASS | Fixed arrays work perfectly |
| array_mutation | ✅ PASS | **Array mutation works!** |
| array_copy | ✅ PASS | Element assignment works |
| array_slice | ⚠️ PARTIAL | Parses, C output has comment |
| array_index_read | ❌ FAIL | Scope difference (not critical) |
| array_zero_size | ❌ FAIL | `[0]int` not supported |
| array_nested | ❌ FAIL | Nested arrays not working |
| array_loop | ❌ FAIL | `mut` in while loops fails |

**Critical Discovery**: **Array mutation (`arr[i] = value`) WORKS!** This is the foundation needed for List<T>.

## Design Constraints

Based on Phase 1 testing:

### ✅ What We Can Use
1. **Fixed arrays**: `[N]int` where N is compile-time constant
2. **Array mutation**: `arr[i] = value` works perfectly
3. **Array indexing**: `arr[i]` for read access
4. **Array literals**: `[1, 2, 3]` initialization

### ❌ What We Cannot Use (Yet)
1. **Runtime array size**: `[new_cap]T` fails if `new_cap` is variable
2. **Zero-sized arrays**: `[0]int` doesn't compile
3. **Slices**: Not properly implemented in C transpiler
4. **`mut` in while loops**: Causes compilation errors

## Implementation Strategy

### Design: Fixed-Capacity List

**Structure**:
```auto
type ListInt {
    data [100]int  // Fixed storage: 100 elements max
    len int        // Current element count
}
```

**Key Decisions**:
1. **Capacity**: Fixed at 100 elements (document limitation clearly)
2. **No generics**: Use concrete types initially (`ListInt`, `ListStr`)
3. **No iteration**: Avoid while loops with `mut`, use manual indexing
4. **Overflow handling**: Push returns 1 if full, pop returns 0 if empty
5. **Direct field access**: No `self.` syntax yet, use `ListInt_push(list, elem)`

**Why This Approach**:
- Works within current language constraints
- Provides immediate value to users
- Validates pure AutoLang implementation concept
- Can upgrade once runtime allocation is available

## Implementation Plan

### Phase 1: Create ListInt Type

**File**: `stdlib/auto/list.at`

**Implementation**:
```auto
/// Fixed-capacity List for integers (Plan 051 Interim Solution)
///
/// LIMITATIONS:
/// - Maximum 100 elements (no runtime allocation yet - see Plan 052)
/// - No iteration support (mut in while loops not working - see Plan 053)
/// - Concrete type only (generics require type system work)
///
/// USAGE:
/// ```
/// let list = ListInt_new()
/// let list = ListInt_push(list, 42)
/// let list = ListInt_push(list, 100)
/// let len = ListInt_len(list)  // Returns 2
/// let elem = ListInt_get(list, 0)  // Returns 42
/// let list = ListInt_pop(list)  // Returns 100
/// ```

type ListInt {
    data [100]int  // Fixed storage for up to 100 elements
    len int        // Current number of elements
}

// Create new empty list
fn ListInt_new() ListInt {
    ListInt {
        data: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
              0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
              0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
              0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
              0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
              0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        len: 0
    }
}

// Push element (returns modified list, check len separately for overflow)
fn ListInt_push(list ListInt, elem int) ListInt {
    if list.len < 100 {
        list.data[list.len] = elem
        list.len = list.len + 1
    }
    list
}

// Pop element (returns the element, or 0 if empty)
fn ListInt_pop(list ListInt) int {
    if list.len == 0 {
        0
    } else {
        list.len = list.len - 1
        list.data[list.len]
    }
}

// Get element at index
fn ListInt_get(list ListInt, index int) int {
    list.data[index]
}

// Set element at index (returns modified list)
fn ListInt_set(list ListInt, index int, elem int) ListInt {
    list.data[index] = elem
    list
}

// Get current length
fn ListInt_len(list ListInt) int {
    list.len
}

// Check if empty (returns 1 if empty, 0 otherwise)
fn ListInt_is_empty(list ListInt) int {
    if list.len == 0 { 1 } else { 0 }
}

// Clear all elements
fn ListInt_clear(list ListInt) ListInt {
    list.len = 0
    list
}

// Insert element at index (shifts elements right)
fn ListInt_insert(list ListInt, index int, elem int) ListInt {
    if list.len < 100 && index <= list.len {
        // Shift elements: data[index+1..len] = data[index..len-1]
        list.data[list.len] = list.data[list.len - 1]
        list.data[list.len - 1] = list.data[list.len - 2]
        list.data[list.len - 2] = list.data[list.len - 3]
        // ... (manual unrolling for simplicity)

        list.data[index] = elem
        list.len = list.len + 1
    }
    list
}
```

### Phase 2: Testing

**Test Directory**: `crates/auto-lang/test/a2c/081_list_int/`

**Test Cases**:
1. `list_new.at` - Create empty list
2. `list_push.at` - Push elements
3. `list_pop.at` - Pop elements
4. `list_get_set.at` - Get/set elements
5. `list_len.at` - Length operations
6. `list_is_empty.at` - Empty check
7. `list_clear.at` - Clear list
8. `list_insert.at` - Insert elements
9. `list_overflow.at` - Test capacity limit
10. `list_manual_iteration.at` - Manual indexing (no while loops)

**Example Test** (`list_push.at`):
```auto
fn main() int {
    let list = ListInt_new()
    let list = ListInt_push(list, 10)
    let list = ListInt_push(list, 20)
    let list = ListInt_push(list, 30)
    ListInt_len(list)
}
```

**Expected**: `3`

### Phase 3: Integration

**Update Prelude** (if tests pass):
```auto
// stdlib/auto/prelude.at
use auto.list: ListInt_new, ListInt_push, ListInt_pop
```

**Documentation Updates**:
- Update `CLAUDE.md` with ListInt usage
- Document limitations clearly
- Add migration path to future dynamic List

## Success Criteria

1. ✅ `ListInt` type compiles correctly
2. ✅ `ListInt_push` adds elements
3. ✅ `ListInt_pop` removes elements
4. ✅ `ListInt_get` accesses elements
5. ✅ `ListInt_set` modifies elements
6. ✅ `ListInt_len` returns correct length
7. ✅ `ListInt_clear` empties list
8. ✅ All tests pass in C transpiler
9. ✅ Clear documentation of limitations
10. ✅ Path to upgrade documented (Plan 052)

## Limitations (Clearly Documented)

1. **Fixed capacity**: Maximum 100 elements
2. **No generics**: Only `ListInt` implemented (not generic `List<T>`)
3. **No iteration**: Must manually index elements
4. **No automatic growth**: Push silently ignores overflow
5. **No error handling**: Returns 0 on empty pop, ignores full push

## Future Upgrades

Once **Plan 052** (Runtime Array Allocation) is complete:
1. Replace `[100]int` with dynamically allocated array
2. Implement automatic reallocation when full
3. Add proper error handling for overflow
4. Implement generics for `List<T>`

Once **Plan 053** (Mutable Variables in Loops) is complete:
1. Add `iterator` method
2. Add `map`, `filter`, `fold` methods
3. Support `for elem in list` syntax

## Timeline Estimate

- **Phase 1** (Implementation): 2-3 hours
- **Phase 2** (Testing): 2-3 hours
- **Phase 3** (Integration): 1 hour

**Total**: 5-7 hours

## Dependencies

- **Required**: Array mutation (✅ working from Phase 1)
- **Required**: Fixed array declaration (✅ working)
- **Required**: Function parameters (✅ working)
- **Optional**: Plan 052 (for upgrade to dynamic capacity)
- **Optional**: Plan 053 (for iteration support)

## Current Status

**Phase**: Ready to implement

**Completed**:
- ✅ Phase 1: Array capability testing (3/8 passing)
- ✅ Design document created
- ✅ Implementation strategy defined

**Next Steps**:
1. ⏸️ Implement `ListInt` in `stdlib/auto/list.at`
2. ⏸️ Create comprehensive tests
3. ⏸️ Update prelude if tests pass
4. ⏸️ Document limitations clearly

**Notes**:
- This is an interim solution, not the final List<T>
- Fixed capacity is acceptable for initial implementation
- Clear documentation of limitations is critical
- Upgrade path to Plans 052/053 must be planned
