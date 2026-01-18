# Plan 043: Generic Slice<T> Type Implementation

## Objective

Implement a generic `Slice<T>` type that provides an immutable view into contiguous sequences (arrays, strings, dstr, etc.), with `[]T` syntax sugar and range operator support.

## Design Requirements

### Type Specification

- **Generic**: `Slice<T>` where T can be any type (u8, char, int, etc.)
- **Immutable View**: Similar to Rust's `&[T]` or `&str` - cannot modify the underlying data
- **Lightweight**: Contains only pointer and length fields
- **Universal**: Can view any contiguous storage (arrays, str, dstr, cstr, etc.)

### Syntax Sugar

- **Type Annotation**: `[]T` represents `Slice<T>`
  - `[]u8` → `Slice<u8>` (byte slice)
  - `[]char` → `Slice<char>` (character slice)
  - `[]int` → `Slice<int>` (integer slice)

### Range Operators

Support Rust-style range syntax for creating slices:

- `[start..end]` - Half-open range (exclusive end)
- `[start..=end]` - Inclusive range
- `[start..]` - From start to end
- `[..end]` - From beginning to end
- `[]` - Full slice (entire sequence)

### Examples

```auto
// String slicing
let s = "hello"
let slice1 = s[0..3]  // "hel"
let slice2 = s[]      // "hello" (full slice)
let slice3 = s[1..]   // "ello"
let slice4 = s[..3]   // "hel"
let slice5 = s[0..=3] // "hell"

// dstr slicing
let mut ds = dstr.new()
ds.push(72)
ds.push(101)
ds.push(108)
ds.push(108)
ds.push(111)
let ds_slice = ds[1..3]  // [101, 108]

// Array slicing
let arr = [1, 2, 3, 4, 5]
let arr_slice = arr[1..3]  // [2, 3]
```

## Type System Integration

### Type Hierarchy

```
Storage Types:
- str (Value::Str, immutable EcoString)
  └─> []char (Slice<char>) via slicing

- dstr (user type with List<u8> field)
  └─> []u8 (Slice<u8>) via slicing

- cstr (C string, *const u8)
  └─> []u8 (Slice<u8>) via slicing (with null-termination check)

- [N]T (static array)
  └─> []T (Slice<T>) via slicing

Generic Containers:
- List<T> (dynamic list)
  └─> []T (Slice<T>) via slicing
```

### Slice Type Definition

```auto
type Slice[T] {
    ptr *T      // Pointer to first element
    len int     // Number of elements

    fn len() int {
        .len
    }

    fn is_empty() bool {
        .len == 0
    }

    fn get(index int) T {
        *(.ptr + index)
    }

    // Iterator support
    fn iter() SliceIter[T] {
        SliceIter(slice: self, index: 0)
    }
}
```

### Memory Safety

- **Borrow Checking**: Slice<T> should conceptually "borrow" from the underlying storage
- **Lifetime Tracking**: Ensure underlying storage outlives the slice
- **Bounds Checking**: All slice access should be bounds-checked at runtime
- **Null Safety**: Slice ptr should never be null for non-empty slices

## Implementation Phases

### Phase 1: Type System Changes

**Tasks**:
1. Add `Slice<T>` to `Type` enum in `ast.rs`
2. Add `[]T` syntax sugar parsing in `lexer.rs` and `parser.rs`
3. Update type resolution to expand `[]T` to `Slice<T>`
4. Add Slice type to C transpiler type mapping
5. Add Slice type to Rust transpiler type mapping

**Files to modify**:
- `crates/auto-lang/src/ast.rs` - Add `Type::Slice` variant
- `crates/auto-lang/src/parser.rs` - Parse `[]T` syntax
- `crates/auto-lang/src/trans/c.rs` - C code generation
- `crates/auto-lang/src/trans/rust.rs` - Rust code generation

**Expected outcome**: `[]T` syntax recognized, transpiles correctly

### Phase 2: Range Operator Implementation

**Tasks**:
1. Add range expression parsing (`..`, `..=`, `start..end`, etc.)
2. Implement index expression with range operator
3. Add bounds checking for slice operations
4. Implement slice creation from different source types

**Files to modify**:
- `crates/auto-lang/src/parser.rs` - Parse range expressions
- `crates/auto-lang/src/eval.rs` - Evaluate slice operations
- `crates/auto-lang/src/vm/slice.rs` - New file for Slice VM implementation

**Expected outcome**: `"hello"[0..3]` creates Slice<char> with "hel"

### Phase 3: VM Implementation

**Tasks**:
1. Create `vm/slice.rs` with Slice<T> VM implementation
2. Implement slice creation from str, dstr, cstr, arrays
3. Implement slice methods: len(), is_empty(), get()
4. Add bounds checking with proper error messages
5. Register Slice type in VM type system

**Files to create**:
- `crates/auto-val/src/slice.rs` - Slice value implementation

**Files to modify**:
- `crates/auto-lang/src/vm/mod.rs` - Add slice module
- `crates/auto-lang/src/interp.rs` - Load slice type

**Expected outcome**: Slice<T> values can be created and manipulated in VM

### Phase 4: String Integration

**Tasks**:
1. Implement slicing for str (Value::Str)
2. Implement slicing for dstr (List<u8>)
3. Implement slicing for cstr (*const u8 with length)
4. Add conversion methods: to_str(), from_str()
5. Handle UTF-8 encoding for str → []char conversion

**Files to modify**:
- `crates/auto-lang/src/eval.rs` - str slicing logic
- `stdlib/auto/dstr.at` - Add slice methods
- `crates/auto-lang/src/vm/str.rs` - str slicing helpers

**Expected outcome**: All string types support slicing with [] operator

### Phase 5: Transpiler Support

**Tasks**:
1. C transpiler: Generate `slice_T` struct with ptr and len
2. C transpiler: Generate bounds checking code for slice access
3. Rust transpiler: Generate `&[T]` slices
4. Add tests for both transpilers

**Files to modify**:
- `crates/auto-lang/src/trans/c.rs` - C code generation
- `crates/auto-lang/src/trans/rust.rs` - Rust code generation
- `crates/auto-lang/test/a2c/XXX_slice/` - Add test cases
- `crates/auto-lang/test/a2r/XXX_slice/` - Add test cases

**Expected outcome**: Slice<T> transpiles correctly to C and Rust

### Phase 6: Iterator Support

**Tasks**:
1. Implement SliceIter<T> type
2. Add `iter()` method to Slice<T>
3. Support `for x in slice` syntax
4. Add iterator methods: map(), filter(), collect()

**Files to modify**:
- `stdlib/auto/slice.at` - Iterator methods
- `crates/auto-lang/src/parser.rs` - Support for-in on slices

**Expected outcome**: Can iterate over slices with for loops

## API Design

### Core Methods

```auto
type Slice[T] {
    // Query methods
    fn len() int
    fn is_empty() bool

    // Element access
    fn get(index int) T
    fn first() May[T]
    fn last() May[T]

    // Slice operations
    fn split_at(index int) (Slice[T], Slice[T])
    fn range(start int, end int) Slice[T]

    // Conversion
    fn to_str() str          // If T == u8 or char
    fn to_array() [N]T       // If size known at compile time
    fn to_list() List[T]

    // Iterator
    fn iter() SliceIter[T]

    // Comparison
    fn eq(other Slice[T]) bool
    fn cmp(other Slice[T]) int
}

type SliceIter[T] {
    slice Slice[T]
    index int

    fn next() May[T]
    fn has_next() bool
}
```

### Conversion Methods (on source types)

```auto
// On str
type str {
    fn as_slice() []char     // View as character slice
    fn as_bytes() []u8       // View as byte slice
}

// On dstr
type dstr {
    fn as_slice() []u8       // View as byte slice
    fn to_str() str          // Convert to owned str (UTF-8 decode)
}

// On [N]T (arrays)
type [N]T {
    fn as_slice() []T        // View entire array
}
```

## Testing Strategy

### Unit Tests

1. **Type System Tests**:
   - Parse `[]T` syntax, verify it expands to `Slice<T>`
   - Type check slice operations
   - Verify bounds checking errors

2. **VM Tests**:
   - Create slices from str, dstr, arrays
   - Test len(), is_empty(), get() methods
   - Test out-of-bounds access
   - Test empty slices

3. **String Tests**:
   - str slicing: `"hello"[0..3]` → "hel"
   - dstr slicing: byte array slicing
   - cstr slicing: with null termination
   - UTF-8 encoding/decoding

### Integration Tests

1. **Transpiler Tests**:
   - a2c: C code generation for Slice<T>
   - a2r: Rust code generation for Slice<T>
   - Verify bounds checking code

2. **End-to-End Tests**:
   - Complex slicing expressions
   - Nested slicing
   - Slice mutation attempts (should fail)
   - Slice lifetime edge cases

### Test Cases

```auto
// Basic slicing
fn test_basic_str_slice() {
    let s = "hello"
    let slice = s[0..3]
    assert(slice.len() == 3)
    assert(slice.get(0) == 'h')
    assert(slice.get(1) == 'e')
    assert(slice.get(2) == 'l')
}

// Full slice
fn test_full_slice() {
    let s = "hello"
    let slice = s[]
    assert(slice.len() == 5)
}

// Open-ended ranges
fn test_open_ranges() {
    let s = "hello"
    let slice1 = s[1..]   // "ello"
    let slice2 = s[..3]   // "hel"
    assert(slice1.len() == 4)
    assert(slice2.len() == 3)
}

// dstr slicing
fn test_dstr_slice() {
    let mut ds = dstr.new()
    ds.push(72)
    ds.push(101)
    ds.push(108)
    let slice = ds[0..2]
    assert(slice.len() == 2)
    assert(slice.get(0) == 72)
    assert(slice.get(1) == 101)
}

// Bounds checking
fn test_bounds_check() {
    let s = "hello"
    let slice = s[0..3]
    // Should panic/return error
    let x = slice.get(5)
}

// Empty slice
fn test_empty_slice() {
    let s = "hello"
    let slice = s[0..0]
    assert(slice.is_empty())
}

// Inclusive range
fn test_inclusive_range() {
    let s = "hello"
    let slice = s[0..=2]  // "hel"
    assert(slice.len() == 3)
}
```

## Implementation Timeline

**Estimated Time**: 8-12 hours

- Phase 1: Type System - 2 hours
- Phase 2: Range Operator - 2 hours
- Phase 3: VM Implementation - 3 hours
- Phase 4: String Integration - 2 hours
- Phase 5: Transpiler Support - 2 hours
- Phase 6: Iterator Support - 1-2 hours

## Success Criteria

1. ✅ Syntax `[]T` recognized by parser and transpilers
2. ✅ Range operators `[start..end]`, `[start..=end]`, `[start..]`, `[..end]`, `[]` work correctly
3. ✅ str, dstr, cstr, and arrays support slicing
4. ✅ Slice<T> is immutable and lightweight
5. ✅ Bounds checking prevents out-of-bounds access
6. ✅ All tests pass (unit + integration)
7. ✅ C and Rust transpilers generate correct code
8. ✅ Zero compiler warnings

## Open Questions

1. **Lifetime Tracking**: How to ensure underlying storage outlives slice in AutoLang (no borrow checker)?
   - **Proposal**: Use runtime reference counting (VmRef) or document as unsafe

2. **Mutability**: Should there be a `MutSlice<T>` for mutable views?
   - **Proposal**: Start with immutable only, add mutable later if needed

3. **UTF-8 Handling**: How to handle invalid UTF-8 when converting []u8 to str?
   - **Proposal**: Return May[str] or panic with error message

4. **Copy vs Reference**: Should Slice<T> copy data or reference it?
   - **Proposal**: Reference only (ptr + len), no copying

5. **Range Precedence**: What is precedence of range operator vs index operator?
   - **Proposal**: `[index]` has higher precedence than `[range]`

## Risks and Mitigations

1. **Memory Safety**: Slices may outlive underlying data
   - **Mitigation**: Document lifetime requirements, use VmRef for reference types

2. **Performance**: Bounds checking overhead
   - **Mitigation**: Allow opt-out with `unsafe` blocks (future)

3. **Complexity**: Range syntax parsing is complex
   - **Mitigation**: Thorough testing, incremental implementation

4. **Transpiler Compatibility**: C and Rust handle slices differently
   - **Mitigation**: Generate appropriate code for each target

## References

- Rust slice documentation: https://doc.rust-lang.org/std/primitive.slice.html
- C++ std::string_view: https://en.cppreference.com/w/cpp/string/basic_string_view
- Go slices: https://go.dev/tour/moretypes/7
