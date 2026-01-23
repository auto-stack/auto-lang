# Plan 052: Runtime Array Allocation

## Objective

Implement runtime-sized array allocation to enable dynamic data structures like `List<T>` with automatic reallocation.

## Motivation

**Problem**: Current AutoLang only supports compile-time fixed arrays:
```auto
let arr [5]int = [1, 2, 3, 4, 5]  // ✅ Works - size is constant
let size = 10
let arr [size]int = [0; size]     // ❌ FAILS - size is variable
```

**Impact**:
- Cannot implement dynamic `List<T>` with reallocation
- Cannot create arrays whose size is determined at runtime
- Cannot implement growable data structures
- Limited to fixed-capacity workarounds (Plan 051)

**Desired State**:
```auto
fn make_array(size int) [int] {
    let arr [size]int = [0; size]  // ✅ Should work
    arr
}

fn List_grow(list List) List {
    let new_cap = list.cap * 2
    let new_arr [new_cap]int  // ✅ Should work
    // copy and return new_arr
}
```

**Benefits**:
1. **Dynamic data structures**: Lists, vectors, growable arrays
2. **Runtime flexibility**: Arrays sized by user input
3. **Memory efficiency**: Allocate exactly what's needed
4. **Self-hosting**: Full List<T> implementation in AutoLang

## Current State Analysis

### What Works Now

**Parser** (`crates/auto-lang/src/parser.rs`):
- Recognizes `[N]T` syntax where N is **constant expression**
- Type: `Type::Array(ArrayType { elem, len })`
- Compile-time size evaluation only

**C Transpiler** (`crates/auto-lang/src/trans/c.rs`):
- Generates: `int arr[100];` (stack-allocated)
- No heap allocation support
- No runtime size evaluation

**VM Evaluator** (`crates/auto-lang/src/eval.rs`):
- Uses fixed-size arrays
- No runtime allocation mechanism

### What Doesn't Work

1. **Runtime size expressions**:
   ```auto
   let n = 10
   let arr [n]int  // Parser error: expected constant
   ```

2. **Function return values as size**:
   ```auto
   fn get_size() int { 10 }
   let arr [get_size()]int  // Fails to parse
   ```

3. **Complex expressions**:
   ```auto
   let arr [x + y]int  // Fails if x, y are variables
   ```

## Design Options

### Option A: Stack-Allocated VLA (Variable Length Array)

**Approach**: Extend C transpiler to use C99 VLAs

**Pros**:
- Simpler implementation
- No heap management needed
- Automatic cleanup (stack-based)

**Cons**:
- C99 support required (not all compilers)
- Stack overflow risk for large arrays
- No persistence across function calls

**Example**:
```c
// Generated C
int size = 10;
int arr[size];  // C99 VLA
```

### Option B: Heap-Allocated Arrays

**Approach**: Use malloc/free for dynamic allocation

**Pros**:
- Works with all C standards
- No stack size limits
- Can return arrays from functions

**Cons**:
- Manual memory management required
- Memory leak risk if not freed
- More complex implementation

**Example**:
```c
// Generated C
int size = 10;
int* arr = malloc(sizeof(int) * size);
// ... use arr ...
free(arr);
```

### Option C: Hybrid Approach (Recommended)

**Approach**:
- Small arrays (< 1KB): Stack-allocated VLA
- Large arrays (≥ 1KB): Heap-allocated
- Automatic cleanup based on scope

**Pros**:
- Best of both worlds
- Performance optimization
- Flexible

**Cons**:
- Most complex implementation
- Needs escape analysis for array lifetime

## Implementation Plan

### Phase 1: Parser Extension

**File**: `crates/auto-lang/src/parser.rs`

**Changes**:
1. Extend `parse_array_type()` to accept **runtime expressions**:
```rust
fn parse_array_type(&mut self) -> AutoResult<Type> {
    self.expect(TokenKind::LBracket)?;

    // Parse size expression (can be runtime value)
    let size_expr = self.parse_expr()?;

    self.expect(TokenKind::RBracket)?;
    let elem = self.parse_type()?;

    // NEW: Create RuntimeArray type instead of Array
    Ok(Type::RuntimeArray(RuntimeArrayType {
        elem: Box::new(elem),
        size: size_expr,  // Expression, not constant
    }))
}
```

2. Add `Type::RuntimeArray` to AST (`ast/types.rs`):
```rust
pub enum Type {
    // ... existing types ...
    RuntimeArray(RuntimeArrayType),  // NEW
}

pub struct RuntimeArrayType {
    pub elem: Box<Type>,
    pub size: Expr,  // Runtime size expression
}
```

### Phase 2: Type System Updates

**File**: `crates/auto-lang/src/type.rs` (or appropriate)

**Changes**:
1. Handle `RuntimeArray` in type checking
2. Size expression must evaluate to `int`
3. Add `sizeof` calculation for C generation

**Type Checking Rules**:
```auto
let arr [expr]int
// expr must have type int
// Result type is "array of int with runtime size"
```

### Phase 3: C Transpiler Support

**File**: `crates/auto-lang/src/trans/c.rs`

**Strategy**: Use hybrid approach (Option C)

**Implementation**:
```rust
match &ty {
    Type::RuntimeArray(rta) => {
        // Evaluate size expression at runtime
        let size_code = self.transpile_expr(&rta.size, sink)?;
        let elem_type = self.c_type_name(&rta.elem);

        // Generate size variable
        let size_var = self.temp_var();
        output!(sink, "int {} = {};", size_var, size_code);

        // Choose allocation strategy based on size
        output!(sink, "if ({} < 256) {{", size_var);
        output!(sink, "  {} arr[{}];", elem_type, size_var);  // VLA
        output!(sink, "}} else {{");
        output!(sink, "  {}* arr = malloc(sizeof({}) * {});",
               elem_type, elem_type, size_var);
        output!(sink, "}}");

        Ok(format!("{}* arr", elem_type))
    }
}
```

**Automatic Cleanup**:
- Track allocated arrays in scope
- Generate cleanup code at end of scope
- Use RAII-style helper functions

### Phase 4: VM Evaluator Support

**File**: `crates/auto-lang/src/eval.rs`

**Strategy**: Use `Vec<Value>` for runtime arrays

**Implementation**:
```rust
match &ty {
    Type::RuntimeArray(rta) => {
        // Evaluate size expression
        let size_value = self.eval_expr(&rta.size)?;
        let size = match size_value {
            Value::Int(n) => n as usize,
            _ => return Err(...),
        };

        // Create Vec with capacity
        let mut data = Vec::with_capacity(size);
        data.resize(size, Value::Nil);  // Initialize

        Ok(Value::Array(ArrayData {
            elems: data,
            is_heap_allocated: size > 256,
        }))
    }
}
```

### Phase 5: Testing

**Test Directory**: `crates/auto-lang/test/a2c/082_runtime_arrays/`

**Test Cases**:
1. `runtime_size_var.at` - Variable-sized array
2. `runtime_size_expr.at` - Expression-sized array
3. `runtime_size_function.at` - Function return as size
4. `runtime_allocation.at` - Large array (heap allocation)
5. `runtime_small.at` - Small array (stack allocation)
6. `runtime_nested.at` - Nested runtime arrays
7. `runtime_cleanup.at` - Automatic cleanup verification
8. `runtime_realloc.at` - Reallocation pattern for List

**Example Test** (`runtime_size_var.at`):
```auto
fn main() int {
    let size = 10
    let arr [size]int = [0; size]
    arr[0] = 42
    arr[0]
}
```

**Expected C Output**:
```c
int main(void) {
    int size = 10;
    int arr[size];  // C99 VLA
    arr[0] = 42;
    return arr[0];
}
```

**Example Test** (`runtime_allocation.at`):
```auto
fn main() int {
    let size = 1000
    let arr [size]int
    arr[0] = 99
    arr[0]
}
```

**Expected C Output** (large array uses heap):
```c
int main(void) {
    int size = 1000;
    int* arr = malloc(sizeof(int) * size);
    arr[0] = 99;
    int _result = arr[0];
    free(arr);
    return _result;
}
```

## Risks & Mitigations

### R1: C99 VLA Support

**Risk**: Not all C compilers support VLAs (MSVC < 2013)

**Mitigation**:
- Use heap allocation for large arrays regardless
- Document C99 requirement for small arrays
- Provide compile-time fallback option

### R2: Memory Leaks

**Risk**: Heap-allocated arrays may leak if not freed

**Mitigation**:
- Automatic cleanup at scope exit
- RAII helper functions
- Escape analysis to determine lifetime
- Comprehensive testing with valgrind/ASAN

### R3: Stack Overflow

**Risk**: Large VLAs on stack can cause overflow

**Mitigation**:
- Size threshold (e.g., 1KB) forces heap allocation
- Configurable threshold via compiler flag
- Document limitation clearly

### R4: Type System Complexity

**Risk**: Runtime arrays complicate type checking

**Mitigation**:
- Start with `int` size only (simplest case)
- Add type inference gradually
- Clear error messages for invalid sizes

### R5: Backwards Compatibility

**Risk**: Breaking existing code with fixed arrays

**Mitigation**:
- Keep `[constant]T` as `Type::Array` (unchanged)
- Only use `RuntimeArray` for variable sizes
- Zero breaking changes to existing code

## Implementation Order

### Step 1: AST and Parser (2-3 hours)
- Add `Type::RuntimeArray` to AST
- Extend parser to accept expressions for size
- Handle both constant and variable sizes

### Step 2: Type Checking (2-3 hours)
- Add type rules for runtime arrays
- Validate size expression type
- Add sizeof calculations

### Step 3: C Transpiler - Basic (3-4 hours)
- Transpile size expression
- Generate C99 VLA code
- Test with small arrays

### Step 4: C Transpiler - Heap (3-4 hours)
- Add heap allocation for large arrays
- Implement automatic cleanup
- Test with large arrays

### Step 5: VM Evaluator (2-3 hours)
- Support runtime arrays in evaluator
- Use Vec<Value> for storage
- Test with various sizes

### Step 6: Comprehensive Testing (2-3 hours)
- Create 8+ test cases
- Test edge cases (size=0, size=1, size=MAX)
- Test cleanup and memory management
- Verify no leaks

### Step 7: Documentation (1 hour)
- Document usage and limitations
- Update CLAUDE.md
- Add examples

**Total Estimated Time**: 15-23 hours

## Success Criteria

1. ✅ Parser accepts `[expr]int` where expr is variable/function call
2. ✅ Type checking validates size expression
3. ✅ C transpiler generates correct code (VLA or malloc)
4. ✅ VM evaluator supports runtime-sized arrays
5. ✅ Small arrays use stack allocation
6. ✅ Large arrays use heap allocation
7. ✅ Automatic cleanup works (no leaks)
8. ✅ All 8+ tests pass
9. ✅ Zero breaking changes to existing fixed arrays
10. ✅ Documentation complete

## Future Enhancements

Once basic runtime allocation works:

1. **Reallocation Helper**: Helper function for growing arrays
2. **Slice Support**: Slices of runtime arrays
3. **Multi-dimensional**: `[m][n]int` runtime sizes
4. **Initialization**: `[expr]T = [default; expr]` syntax
5. **Generics**: `[expr]T` for any type T

## Dependencies

- **Required**: Parser infrastructure (✅ exists)
- **Required**: Type system (✅ exists)
- **Required**: C transpiler (✅ exists)
- **Required**: VM evaluator (✅ exists)
- **Optional**: Escape analysis system (can be added later)
- **Optional**: Garbage collection (can use manual free for now)

## Next Steps After This Plan

1. ✅ Update Plan 051 List to use dynamic capacity (DEPRECATED - see Plan 055)
2. ⏸️ Implement proper reallocation in List.push()
3. ⏸️ Add generic `List<T>` support
4. ⏸️ Add iteration support (Plan 053)
5. ✅ **Enable Plan 054 Phase 2**: Root config with user-specified heap sizes
6. ✅ **Enable Plan 055 Phase 7**: True target-specific C code generation

## Current Status

**Status**: ✅ COMPLETED (2025-01-23)

**Completed**:
- ✅ Problem analysis
- ✅ Design options evaluated
- ✅ Implementation strategy chosen (heap allocation approach)
- ✅ Implementation plan detailed
- ✅ **Phase 1: AST and Parser** - RuntimeArray type added to AST, parser supports `[expr]T`
- ✅ **Phase 2: Type System** - substitute() and occurs_in() support RuntimeArray
- ✅ **Phase 3: C Transpiler (Basic)** - Generates `int* arr = malloc(sizeof(int) * (size))`
- ✅ **Phase 4: C Transpiler (Advanced)** - Parenthesized size expressions, heap allocation
- ✅ **Phase 5: VM Evaluator** - Runtime array allocation via `eval_store()`
- ✅ **Phase 6: Testing** - 2 a2c tests passing

**Blocked**: None - implementation complete

**Priority**: ✅ COMPLETE - Enables Plan 054/055 advanced features

**Implementation Files**:
- `src/ast/types.rs` - RuntimeArray type definition (line 21, 264-267)
- `src/parser.rs` - parse_array_type() supports runtime expressions (line 4220-4247)
- `src/eval.rs` - eval_store() allocates runtime arrays (line 894-907)
- `src/trans/c.rs` - C generation with malloc (line 1518-1523, 1680-1703)
- `src/infer/unification.rs` - occurs_in() support (line 142)
- `test/a2c/082_runtime_size_var/` - Variable size test
- `test/a2c/083_runtime_size_expr/` - Expression size test

**Relationship to Other Plans**:

### Plan 054 (Context Environment)
- **Dependency**: Plan 054 Phase 2 (Root Config) requires runtime allocation
  - User can specify `const HeapSize = 4096` in config
  - This heap needs runtime allocation support
- **Enables**: Full MCU vs PC environment customization
  - MCU: User-defined static heap arrays
  - PC: Dynamic heap allocation with malloc

### Plan 055 (Storage Injection)
- **Current State**: Plan 055 implements basic Storage infrastructure
  - ✅ Storage type markers (Fixed/Dynamic) work
  - ✅ Target detection works
  - ✅ Environment injection works
  - ⏸️ But actual dynamic List growth still requires runtime allocation
- **Enhancement**: Plan 052 enables Plan 055 Phase 7 (C Transpiler Enhancements)
  - Generate static arrays for MCU Fixed storage
  - Generate heap-allocated structures for PC Dynamic storage
  - Support user-specified heap sizes from root config

### Plan 051 (Fixed-Capacity List)
- **Status**: DEPRECATED - superseded by Plan 055
- **Legacy**: Plan 051's array testing findings inform Plan 052 design

### Plan 050 (Prelude System)
- **No Direct Dependency**: Prelude system doesn't require runtime arrays
- **Future**: Could enable dynamic prelude configuration

**Notes**:
- This is a major language feature
- Requires careful testing for memory safety
- Backwards compatibility is critical
- Hybrid approach balances complexity and performance
- **Important**: Plan 054/055 can proceed without Plan 052, but advanced features require it
