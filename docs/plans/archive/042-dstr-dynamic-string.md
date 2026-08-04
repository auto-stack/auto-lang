# Plan 042: Dynamic String (dstr) Type Implementation

## Objective

Implement a dynamic string type `dstr` in AutoLang that provides:
- Dynamic growth (can push/pop bytes)
- UTF-8 encoding support
- O(1) length access
- Byte-level manipulation
- Pure AutoLang implementation (no [c] or [vm] functions)
- OOP-style method syntax

## Background

Previous exploration revealed:
1. ✅ **VM types (like `List`) CAN be struct fields** in pure Auto types
2. ✅ **Instance methods work** on user-defined types (see `File` type in io.at)
3. ❌ **Function-style API has ownership issues** - variables are moved on first call
4. ✅ **OOP-style methods don't have ownership issues** - VM types use reference semantics

The correct approach is to create a proper Auto type with a `List` field and instance methods.

## Design

### Type Definition

```auto
type dstr {
    data List  // Internal List<u8> storage
}
```

### API Design

#### Creation Methods
- `dstr.new()` - Create empty dstr
- `dstr.from_byte(b u8)` - Create from single byte
- `dstr.from_bytes(b1 u8, b2 u8)` - Create from two bytes
- `dstr.from_str(s str)` - Create from string literal (future)

#### Instance Methods
- `len() int` - Get byte count
- `is_empty() bool` - Check if empty
- `push(b u8)` - Append byte
- `pop() u8` - Remove and return last byte
- `get(index int) u8` - Get byte at index
- `set(index int, b u8) int` - Set byte at index
- `insert(index int, b u8)` - Insert byte at position
- `remove(index int) u8` - Remove byte at position
- `clear()` - Remove all bytes
- `reserve(capacity int)` - Pre-allocate space

### Usage Example

```auto
// Create
mut s = dstr.new()
s.push(72)  // 'H'
s.push(101) // 'e'
s.push(108) // 'l'
s.push(108) // 'l'
s.push(111) // 'o'

// Query
let len = s.len()  // 5
let first = s.get(0)  // 72

// Modify
s.set(0, 74)  // Change 'H' to 'J'
s.pop()  // Remove last char
s.insert(0, 72)  // Insert at start
```

## Implementation Plan

### Phase 1: Type Declaration (Core)
**File**: `stdlib/auto/dstr.at`

1. Define the `dstr` type with `data List` field
2. Implement static creation methods:
   - `fn dstr.new() dstr`
   - `fn dstr.from_byte(b u8) dstr`
   - `fn dstr.from_bytes(b1 u8, b2 u8) dstr`

**Success Criteria**: Type compiles, instances can be created

### Phase 2: Query Methods (Read-only)
**File**: `stdlib/auto/dstr.at`

Implement query methods that don't modify the string:
- `fn len(self dstr) int`
- `fn is_empty(self dstr) bool`
- `fn get(self dstr, index int) u8`

**Success Criteria**: Can query dstr properties without errors

### Phase 3: Modification Methods (Mutating)
**File**: `stdlib/auto/dstr.at`

Implement methods that modify the string:
- `fn push(mut self dstr, b u8)`
- `fn pop(mut self dstr) u8`
- `fn set(mut self dstr, index int, b u8) int`
- `fn insert(mut self dstr, index int, b u8)`
- `fn remove(mut self dstr, index int) u8`
- `fn clear(mut self dstr)`
- `fn reserve(mut self dstr, capacity int)`

**Success Criteria**: Can modify dstr without ownership issues

### Phase 4: Testing
**File**: `crates/auto-lang/src/dstr_tests.rs`

1. Replace function-style tests with OOP-style tests
2. Test creation: `test_dstr_new()`, `test_dstr_from_byte()`
3. Test queries: `test_dstr_len()`, `test_dstr_get()`
4. Test modifications: `test_dstr_push()`, `test_dstr_pop()`, `test_dstr_set()`
5. Test comprehensive usage: `test_dstr_comprehensive()`
6. Test indexing: `test_dstr_index_operator()`
7. Test iteration: `test_dstr_for_loop()`

**Success Criteria**: All tests pass

### Phase 5: Advanced Features (Future)
**File**: `stdlib/auto/dstr.at` (future work)

- `from_str(s str) dstr` - Convert from string literal
- `to_str(self dstr) str` - Convert to string
- `append(self dstr, other dstr)` - Concatenate
- `split(self dstr, delimiter u8) []dstr` - Split by delimiter
- Index operator support: `s[0]` → `s.get(0)`
- For loop support: `for b in s { ... }`

**Success Criteria**: N/A (future work)

## Technical Details

### Instance Method Syntax

Based on `File` type in `io.at`, instance methods:
- Use `self` parameter (no `self:` type annotation needed)
- Access fields using `.field` syntax (e.g., `.data`)
- Don't need explicit return type in some cases

```auto
type dstr {
    data List

    fn len() int {
        .data.len()
    }

    fn push(b u8) {
        .data.push(b)
    }
}
```

### Ownership Semantics

VM types (List, HashMap, etc.) use reference semantics:
- ✅ Method calls don't move the receiver: `s.push(65)`
- ✅ Can call multiple methods: `s.push(65); s.push(66)`
- ✅ Field access works: `s.data.len()`

This is different from function calls which move parameters.

### List Field Access

The `.data` field is a VM `List` instance:
- Created by `List.new()`
- Has methods like `.push()`, `.pop()`, `.len()`
- Stored as a field in dstr instance

## Testing Strategy

### Unit Tests

Each method gets its own test:
```rust
#[test]
fn test_dstr_push() {
    let code = r#"
        mut s = dstr.new()
        s.push(65)
        s.push(66)
        s.len()
    "#;
    assert_eq!(run(code).unwrap(), "2");
}
```

### Integration Tests

Test realistic usage patterns:
```rust
#[test]
fn test_dstr_comprehensive() {
    let code = r#"
        mut s = dstr.new()
        s.push(72)  // Build "Hello"
        s.push(101)
        s.push(108)
        s.push(108)
        s.push(111)

        let len = s.len()
        let first = s.get(0)
        // ... verify results
    "#;
    // assertions
}
```

### Edge Cases

- Empty dstr operations
- Single byte dstr
- Large dstr (test reserve)
- Out of bounds access
- Pop from empty dstr

## Success Criteria

1. ✅ Type `dstr` compiles without errors
2. ✅ Can create instances: `dstr.new()`, `dstr.from_byte(65)`
3. ✅ Can call methods: `s.push(65)`, `s.len()`, `s.get(0)`
4. ✅ No ownership issues - can call multiple methods
5. ✅ All unit tests pass
6. ✅ Works with existing List infrastructure
7. ✅ Pure AutoLang - no [c] or [vm] functions

## Non-Goals

❌ UTF-8 validation - stores raw bytes, caller ensures valid UTF-8
❌ Unicode character iteration - byte-level only (for now)
❌ String interning - each dstr has its own storage
❌ C transpilation - focus on evaluator/VM for now

## Related Work

- **Plan 025**: String Type Redesign - dstr complements existing str types
- **Plan 041**: List Dynamic Array - dstr builds on List foundation
- **io.at File type**: Reference for instance method syntax
- **data/list.at**: Not applicable - List is VM-only, no .at file

## Timeline

- Phase 1-2: Type declaration + query methods (30 min)
- Phase 3: Modification methods (30 min)
- Phase 4: Testing (30 min)
- **Total**: ~1.5 hours

## Status

🔄 **In Progress**: Creating plan document

## Next Steps

1. ✅ Create plan document (this file)
2. ⏳ Implement Phase 1: Type declaration
3. ⏳ Implement Phase 2: Query methods
4. ⏳ Implement Phase 3: Modification methods
5. ⏳ Implement Phase 4: Testing
6. ⏸️ Phase 5: Advanced features (deferred)
