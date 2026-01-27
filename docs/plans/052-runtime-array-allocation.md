# Plan 052: Storage-Based List Implementation

**Status**: 🔄 IN PROGRESS (Architecture Redesign)

**Phase 1** (Old Runtime Array): ⚠️ DEPRECATED - Wrong approach
**Phase 2** (Storage-Based List): ✅ NEW - Correct design

---

## ⚠️ Critical Design Correction

**Previous Approach** (WRONG):
- Implement `[expr]T` as language primitive
- Lock List<T> to heap allocation only
- ❌ "Managed language thinking"

**New Approach** (CORRECT):
- **NO** `[expr]T` language primitive
- Use **Strategy Pattern**: `List<T, S: Storage>`
- List directly operates on **raw pointers** (`*T`)
- Zero-overhead abstraction through **monomorphization**

This design follows **Rust `Vec<T>`** and **C++ `std::vector<T>`** philosophy.

---

## Objective

Implement **self-hosted `List<T, S: Storage>`** with pluggable storage strategies:

1. **`Heap<T>`** - PC heap allocation (dynamic growth via `malloc`/`realloc`)
2. **`Inline<T, N>`** - MCU stack allocation (static buffer, zero heap)
3. **Custom Storage** - User-defined strategies (Arena, Pool, etc.)

**Key Principle**: List logic is **storage-agnostic**. Storage strategies handle **raw memory**.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│ User Code                                                    │
│ let list1 List<int>                    // Auto: Heap          │
│ let list2 List<int, Inline<int, 64>>   // MCU: Stack         │
│ let list3 List<int, ArenaRef<int>>      // Game: Arena       │
├─────────────────────────────────────────────────────────────┤
│ List<T, S> (stdlib/auto/list.at)                               │
│ - push(), pop(), len(), capacity()                           │
│ - Logic is STORAGE-AGNOSTIC                                     │
├─────────────────────────────────────────────────────────────┤
│ Storage Trait (Duck Typing)                                   │
│ - data() *T            - Get raw pointer                      │
│ - capacity() u32       - Get physical capacity                  │
│ - try_grow(min_cap) bool - Try to grow (may fail)          │
├─────────────────────────────────────────────────────────────┤
│ Storage Implementations (stdlib/auto/storage.at)                 │
│                                                              │
│  Heap<T>                 Inline<T, N>         ArenaRef<T>       │
│  ┌──────────┐          ┌──────────┐          ┌──────────┐   │
│  │ ptr: *T  │          │ buffer:  │          │ ptr: *T  │   │
│  │ cap: u32 │          │ [N]T     │          │ cap: u32 │   │
│  └──────────┘          └──────────┘          │ arena: * │   │
│                                             └──────────┘   │
└─────────────────────────────────────────────────────────────┘
│   (realloc)              (fixed)             (arena alloc)   │
```

**Key Insight**: `List<T, S>` doesn't know WHERE memory is. It only calls `S.data()`, `S.capacity()`, `S.try_grow()`.

---

## Why NOT `[expr]T` (Language Primitive)

### The Wrong Approach

```auto
// ❌ Language-level runtime array
type List<T> {
    data [runtime]T  // ← Locks List to heap ONLY!
    len u32
    cap u32
}
```

**Problems**:
1. **Hidden Allocator**: Who allocates? `malloc`? GC? How to customize?
2. **No MCU Support**: Can't use static storage (embedded systems)
3. **Abstraction Inversion**: List depends on language feature instead of raw memory
4. **NOT Zero-Cost**: Extra layer between List and memory

### The Right Approach

```auto
// ✅ Strategy pattern with raw pointers
type List<T, S: Storage> {
    len: u32
    store: S  // ← Abstract storage, could be *T or [N]T
}

// Storage契约 (concept)
trait Storage<T> {
    fn data(*Self) *T
    fn capacity(*Self) u32
    fn try_grow(*mut Self, min_cap u32) bool
}

// 具体实现
type Heap<T> { ptr: *T, cap: u32 }
type Inline<T, N u32> { buffer: [N]T }
type ArenaRef<T> { ptr: *T, cap: u32, arena: *Arena }
```

**Advantages**:
1. ✅ **Zero-Cost**: Monomorphization generates specialized code
2. ✅ **Flexible**: Custom storage strategies
3. ✅ **MCU-Friendly**: `Inline<T, N>` = zero heap allocation
4. ✅ **Explicit**: Users see raw memory management in stdlib

---

## Storage Contract (The "Trait")

Every storage strategy MUST implement these methods:

```auto
// Storage concept (Duck typing)
concept Storage<T> {
    // Get raw pointer to data
    fn data(*Self) *T

    // Get physical capacity (may differ from len)
    fn capacity(*Self) u32

    // Try to grow to min_cap
    // - Heap: realloc to max(min_cap, cap * 2)
    // - Inline: return min_cap <= N
    // - Arena: alloc new block, copy data
    // Returns true on success, false if OOM
    fn try_grow(*mut Self, min_cap u32) bool
}
```

**Key Design**: These are **unsafe** operations, but they're **encapsulated** in stdlib. Users never call them directly - they use `List<T, S>` methods instead.

---

## Phase 1: Core Infrastructure

### Step 1.1: Pointer Type Support

**File**: `crates/auto-lang/src/ast/types.rs`

**Add pointer type**:
```rust
pub enum Type {
    // ... existing types ...
    Pointer(Box<Type>),      // NEW: *T (raw pointer)
    Reference(Box<Type>),    // NEW: &T (reference)
}
```

**Parser support** (`parser.rs`):
```rust
fn parse_pointer_type(&mut self) -> AutoResult<Type> {
    self.expect(TokenKind::Star)?;  // *
    let elem = self.parse_type()?;
    Ok(Type::Pointer(Box::new(elem)))
}
```

### Step 1.2: Unsafe Operations (VM Only)

**File**: `crates/auto-lang/src/eval.rs`

**Pointer operations** (VM implementation):
```rust
Expr::Deref(ptr) => {
    // *ptr - dereference pointer
    let ptr_val = self.eval_expr(ptr)?;
    match ptr_val {
        Value::Pointer(addr, typ) => {
            // Read from VM memory
            self.read_memory(addr, typ)
        }
        _ => Value::error("Cannot dereference non-pointer".into()),
    }
}

Expr::AddrOf(mut expr) => {
    // &mut expr - get address
    // For VM: return pointer to value
    // For A2C: generate &expr
}
```

### Step 1.3: Const Generic Parameters

**File**: `crates/auto-lang/src/ast/types.rs`

```rust
pub struct GenericType {
    pub name: Name,
    pub params: Vec<GenericParam>,  // <T, const N u32>
}

pub enum GenericParam {
    Type(Name),
    Const(Name, Expr),  // NEW: const N: u32
}
```

**Parser**:
```auto
// Parse Inline<T, 128>
type Inline<T, const N u32> {
    buffer: [N]T
}
```

---

## Phase 2: Storage Implementations

### Step 2.1: Create Storage Module

**File**: `stdlib/auto/storage.at` (新建)

```auto
/// Storage strategies for List<T>
/// This is a PRIVATE module - users should use List<T>, not Storage directly

// ============================================================================
// Storage Trait Marker
// ============================================================================

/// Marker trait for storage types
/// All storages must implement: data(), capacity(), try_grow()
type Storage {
}

// ============================================================================
// Heap Storage (PC Standard)
// ============================================================================

/// Heap-allocated storage using malloc/realloc
/// Suitable for: PC, server, systems with heap
type Heap<T>: Storage {
    ptr: *T
    cap: u32
}

ext Heap<T> {
    /// Create empty heap storage
    static fn new() Heap<T> {
        return Heap(ptr: nil as *T, cap: 0)
    }

    /// Get raw data pointer
    fn data() *T => .ptr

    /// Get physical capacity
    fn capacity() u32 => .cap

    /// Try to grow to minimum capacity
    /// Uses exponential growth: max(min_cap, cap * 2)
    fn try_grow(min_cap u32) bool {
        let new_cap = if .cap == 0 { 8 } else { .cap * 2 }
        if new_cap < min_cap { new_cap = min_cap }

        // Call C realloc
        let new_ptr = c.stdlib.realloc(.ptr, new_cap * sizeof(T))
        if new_ptr == nil { return false }  // OOM

        .ptr = new_ptr as *T
        .cap = new_cap
        return true
    }

    /// Free memory (called by List.drop())
    fn drop() {
        if .ptr != nil {
            c.stdlib.free(.ptr)
        }
    }
}

// ============================================================================
// Inline Storage (MCU Standard)
// ============================================================================

/// Stack-allocated storage with fixed capacity
/// Suitable for: MCU, embedded systems, performance-critical code
type Inline<T, const N u32>: Storage {
    buffer: [N]T
}

ext Inline<T, N> {
    /// Create inline storage (buffer is zero-initialized)
    static fn new() Inline<T, N> {
        return Inline(buffer: [0; N])
    }

    /// Get raw data pointer (pointer to first element)
    fn data() *T => .buffer.ptr

    /// Get physical capacity (compile-time constant)
    fn capacity() u32 => N

    /// Try to grow - always fails if exceeds N
    fn try_grow(min_cap u32) bool {
        // Inline storage CANNOT grow
        return min_cap <= N
    }

    /// No-op (stack-allocated)
    fn drop() { }
}

// ============================================================================
// ArenaRef Storage (Game Engine Standard)
// ============================================================================

/// Arena-backed storage
/// Suitable for: game engines, high-performance systems, batch allocations
type ArenaRef<T>: Storage {
    ptr: *T
    cap: u32
    arena: *Arena  // Borrowed from external Arena
}

ext ArenaRef<T> {
    /// Create arena-backed storage
    static fn new(arena *Arena) ArenaRef<T> {
        return ArenaRef(ptr: nil as *T, cap: 0, arena: arena)
    }

    fn data() *T => .ptr

    fn capacity() u32 => .cap

    /// Try to grow - allocates new block from arena
    fn try_grow(min_cap u32 T> bool {
        // Arena typically doesn't support realloc
        // Must alloc new, copy, leave old block leaked (or reset)
        let new_cap = max(min_cap, .cap * 2)
        let new_ptr = .arena.alloc(new_cap * sizeof(T))
        if new_ptr == nil { return false }

        // Copy old data
        if .ptr != nil {
            c.string.memcpy(new_ptr, .ptr, .cap * sizeof(T))
            // Note: Old block is leaked (Arena will reset)
        }

        .ptr = new_ptr as *T
        .cap = new_cap
        return true
    }

    /// No-op (Arena owns the memory)
    fn drop() { }
}

// ============================================================================
// Arena Type (Minimal)
// ============================================================================

type Arena {
    // ... implementation details ...
}

ext Arena {
    fn alloc(size usize) *void
}
```

### Step 2.2: C Transpiler Support

**File**: `crates/auto-lang/src/trans/c.rs`

**Monomorphization**:
```rust
// When encountering List<T, S>
fn transpile_list(&mut self, ty: &Type, out: &mut dyn Write) -> Result<()> {
    match ty {
        // List<T, Heap<T>>
        Type::List(elem, Type::Generic { name, params })
            if name == "Heap" => {
                // Generate:
                typedef struct {
                    uint32_t len;
                    T* ptr;
                    uint32_t cap;
                } List_T_Heap;
            }

        // List<T, Inline<T, N>>
        Type::List(elem, Type::Generic { name, params })
            if name == "Inline" && params.len() == 2 => {
                // Extract N from const param
                let n = self.eval_const_expr(&params[1])?;
                // Generate:
                typedef struct {
                    uint32_t len;
                    T buffer[N];  // ← Directly embedded!
                } List_T_Inline_N;
            }

        _ => { ... }
    }
}
```

---

## Phase 3: List<T, S> Implementation

### Step 3.1: List Definition

**File**: `stdlib/auto/list.at`

```auto
/// Dynamic array with pluggable storage strategy
///
/// ## Type Parameters
/// - `T` - Element type
/// - `S` - Storage strategy (default: environment-dependent)
///
/// ## Examples
/// ```auto
/// // Default (environment-adaptive)
/// let list1 List<int> = List.new()
///
/// // Explicit heap allocation (PC)
/// let list2 List<int, Heap<int>> = List.new()
///
/// // Explicit stack allocation (MCU)
/// let list3 List<int, Inline<int, 128>> = List.new()
/// ```
///
type List<T, S: Storage = std.env.DefaultStorage<T>> {
    len: u32
    store: S
}

ext List<T, S> {
    // ========================================================================
    // Construction
    // ========================================================================

    /// Create new empty list
    static fn new() List<T, S> {
        return List(len: 0, store: S.new())
    }

    /// Create list with initial capacity
    /// Note: May ignore hint for Inline storage (fixed capacity)
    static fn with_capacity(cap u32) List<T, S> {
        let list = List.new()
        // Try to pre-allocate
        if list.store.try_grow(cap) {
            // Storage grew successfully
        }
        return list
    }

    // ========================================================================
    // Element Access
    // ========================================================================

    /// Get element at index (panics if out of bounds)
    fn get(index u32) T {
        if index >= .len {
            panic("List::get(): index {} out of bounds (len: {})", index, .len)
        }
        let ptr = .store.data()
        // Unsafe pointer access - encapsulated safety
        ptr[index]
    }

    /// Set element at index (panics if out of bounds)
    fn set(index u32, value T) {
        if index >= .len {
            panic("List::set(): index {} out of bounds (len: {})", index, .len)
        }
        let ptr = .store.data()
        ptr[index] = value
    }

    // ========================================================================
    // Capacity Management
    // ========================================================================

    /// Number of elements
    fn len() u32 => .len

    /// Physical capacity
    fn capacity() u32 => .store.capacity()

    /// Is empty?
    fn is_empty() bool => .len == 0

    // ========================================================================
    // Modification
    // ========================================================================

    /// Add element to end (may trigger growth)
    fn push(elem T) {
        if .len >= .store.capacity() {
            // Try to grow (doubles capacity)
            if !.store.try_grow(.len + 1) {
                panic("List::push(): failed to grow - out of memory")
            }
        }

        let ptr = .store.data()
        ptr[.len] = elem
        .len += 1
    }

    /// Remove and return last element
    fn pop() T? {
        if .len == 0 {
            return nil
        }
        .len -= 1
        let ptr = .store.data()
        return ptr[.len]
    }

    /// Insert element at index (shifts elements right)
    fn insert(index u32, elem T) {
        if index > .len {
            panic("List::insert(): index {} out of bounds (len: {})", index, .len)
        }

        if .len >= .store.capacity() {
            if !.store.try_grow(.len + 1) {
                panic("List::insert(): failed to grow - out of memory")
            }
        }

        let ptr = .store.data()
        // Shift elements right
        var i = .len
        while i > index {
            ptr[i] = ptr[i - 1]
            i -= 1
        }
        ptr[index] = elem
        .len += 1
    }

    /// Remove element at index (shifts elements left)
    fn remove(index u32) T {
        if index >= .len {
            panic("List::remove(): index {} out of bounds (len: {})", index, .len)
        }

        let ptr = .store.data()
        let value = ptr[index]

        // Shift elements left
        var i = index
        while i < .len - 1 {
            ptr[i] = ptr[i + 1]
            i += 1
        }
        .len -= 1

        return value
    }

    /// Clear all elements
    fn clear() {
        .len = 0
    }

    // ========================================================================
    // View Conversion
    // ========================================================================

    /// Convert to slice view
    fn view() []T {
        let ptr = .store.data()
        return ptr[0 .. .len]
    }
}
```

### Step 3.2: C Transpiler Support

**File**: `crates/auto-lang/src/trans/c.rs`

**Storage strategy mapping**:
```rust
fn transpile_storage_struct(&mut self, storage: &Type, elem: &Type) -> String {
    match storage {
        // Heap<T> → { T* ptr; uint32_t cap; }
        Type::Generic { name, .. } if name == "Heap" => {
            let elem_c = self.c_type_name(elem);
            format!("{}* ptr;\nuint32_t cap;", elem_c)
        }

        // Inline<T, N> → { T buffer[N]; }
        Type::Generic { name, params, .. } if name == "Inline" => {
            let elem_c = self.c_type_name(elem);
            if let Some(n) = self.extract_const_param(params, 0) {
                format!("{} buffer[{}];", elem_c, n)
            } else {
                // Error: must be const
            }
        }

        // ArenaRef<T> → { T* ptr; uint32_t cap; Arena* arena; }
        // ... similar
    }
}

fn transpile_list_struct(&mut self, list_ty: &Type, out: &mut Write) -> Result<()> {
    // List<T, S>
    let (elem, storage) = self.extract_list_params(list_ty)?;
    let elem_c = self.c_type_name(elem);

    // Generate struct
    writeln!(out, "typedef struct {{")?;
    writeln!(out, "    uint32_t len;")?;
    writeln!(out, "    {}};", self.transpile_storage_struct(storage, elem)?)?;
    writeln!(out, "}} List_{}_{};", elem_c, self.storage_name(storage))?;

    Ok(())
}
```

---

## Phase 4: Environment Integration

### Step 4.1: Default Storage Selection

**File**: `stdlib/auto/prelude.at`

```auto
// ============================================================================
// Storage Selection (Environment-Dependent)
// ============================================================================

// Import storage types
use auto.storage: Storage, Heap, Inline, ArenaRef

// Default storage depends on target environment
// This is injected by compiler based on --target flag
type DefaultStorage<T>: Storage = std.env.DefaultStorage<T>

// ============================================================================
// Collections
// ============================================================================

use auto.list: List

// Users write List<T>, we expand to List<T, DefaultStorage<T>>
// - PC (--target pc): List<T, Heap<T>>
// - MCU (--target mcu): List<T, Inline<T, 64>>
```

### Step 4.2: Compiler Environment Injection

**File**: `crates/auto/src/main.rs`

```rust
#[arg(long)]
target: Option<TargetArg>,

#[derive(Clone, ValueEnum)]
enum TargetArg {
    Pc,
    Mcu { inline_size: Option<usize> },
    Auto,
}

fn main() {
    match args.target {
        Some(TargetArg::Pc) => {
            // Inject environment for PC
            // DefaultStorage<T> = Heap<T>
        }
        Some(TargetArg::Mcu { inline_size }) => {
            // Inject environment for MCU
            // DefaultStorage<T> = Inline<T, 64>
        }
        Some(TargetArg::Auto) => {
            // Auto-detect based on host
        }
        None => {
            // Default: PC
        }
    }
}
```

---

## Phase 5: Implementation Steps

### Step 5.1: Pointer Type (1-2 days)
**File**: `src/ast/types.rs`
- [ ] Add `Type::Pointer(Box<Type>)`
- [ ] Add `Type::Reference(Box<Type>)`

**File**: `src/parser.rs`
- [ ] `fn parse_pointer_type()`
- [ ] `fn parse_reference_type()`

### Step 5.2: Const Generic Parameters (2-3 days)
**File**: `src/ast/types.rs`
- [ ] Add `GenericParam::Const(Name, Expr)`
- [ ] Support `const N u32` in type params

**File**: `src/parser.rs`
- [ ] Parse `type Foo<T, const N u32>`

### Step 5.3: Storage Module (3-4 days)
**File**: `stdlib/auto/storage.at`
- [ ] Implement `Heap<T>`
- [ ] Implement `Inline<T, N>`
- [ ] Implement `ArenaRef<T>`
- [ ] Implement `Arena` (minimal)

### Step 5.4: List Implementation (3-4 days)
**File**: `stdlib/auto/list.at`
- [ ] Implement `List<T, S>` with all methods
- [ ] Integrate with Storage trait
- [ ] Add comprehensive documentation

### Step 5.5: C Transpiler (4-5 days)
**File**: `src/trans/c.rs`
- [ ] Monomorphization support
- [ ] Generate specialized structs for each (T, S) combination
- [ ] Inline `capacity()` calls for Inline (constant folding)
- [ ] Generate correct memory operations

### Step 5.6: Testing (3-4 days)
**VM Tests**:
- [ ] `test_list_heap_basic`
- [ ] `test_list_inline_basic`
- [ ] `test_list_growth`
- [ ] `test_list_push_pop`
- [ ] `test_list_monomorphization`

**A2C Tests**:
- [ ] `test_085_list_heap` - PC, heap allocation
- [ ] `test_086_list_inline` - MCU, stack allocation
- [ ] `test_087_list_growth` - Automatic growth
- [ ] `test_088_list_monomorph` - Different types, same code

### Step 5.7: Documentation (1 day)
- [ ] Update CLAUDE.md
- [ ] Add examples for each storage type
- [ ] Document MCU vs PC differences

---

## Success Criteria

### Phase 1: Infrastructure
1. ✅ Pointer type (`*T`) parses correctly
2. ✅ Const generic parameters (`const N u32`) work
3. ✅ VM evaluates pointer operations (unsafe)

### Phase 2: Storage
4. ✅ `Heap<T>` compiles and runs (malloc/realloc)
5. ✅ `Inline<T, N>` compiles and runs (stack buffer)
6. ✅ Storage trait (duck typing) enforced by compiler

### Phase 3: List
7. ✅ `List<T, Heap<T>>` works like PC vector
8. ✅ `List<T, Inline<T, 64>>` works like fixed buffer
9. ✅ All List methods (push, pop, get, set, insert, remove) work

### Phase 4: Integration
10. ✅ `List<int>` defaults to `Heap<int>` on PC
11. ✅ `List<int>` defaults to `Inline<int, 64>` on MCU
12. ✅ C transpiler generates optimized code

### Phase 5: Testing
13. ✅ All VM tests pass
14. ✅ All A2C tests generate correct C
15. ✅ Zero memory leaks (valgrind clean)

---

## C Code Examples

### `List<int, Heap<int>>` (PC)

**AutoLang**:
```auto
let list = List.new()
list.push(1)
list.push(2)
```

**Generated C**:
```c
typedef struct {
    uint32_t len;
    int* ptr;
    uint32_t cap;
} List_int_Heap;

void List_int_Heap_push(List_int_Heap* self, int val) {
    if (self->len >= self->cap) {
        uint32_t new_cap = self->cap == 0 ? 8 : self->cap * 2;
        int* new_ptr = realloc(self->ptr, sizeof(int) * new_cap);
        if (!new_ptr) { panic("OOM"); }
        self->ptr = new_ptr;
        self->cap = new_cap;
    }
    self->ptr[self->len] = val;
    self->len++;
}
```

### `List<int, Inline<int, 4>>` (MCU)

**AutoLang**:
```auto
let list = List.new()
list.push(1)
list.push(2)
list.push(3)
list.push(4)  // OK
list.push(5)  // PANIC (capacity exceeded)
```

**Generated C**:
```c
typedef struct {
    uint32_t len;
    int buffer[4];
} List_int_Inline_4;

void List_int_Inline_4_push(List_int_Inline_4* self, int val) {
    if (self->len >= 4) {  // Compile-time constant!
        panic("List::push(): capacity exceeded");
    }
    self->buffer[self->len] = val;
    self->len++;
}
```

---

## Comparison: Before vs After

### Before (Current - WRONG)

```auto
// ❌ Language-level runtime array
type List<T> {
    data [runtime]T  // ← Hidden malloc
    len int
    cap int
}
```

**Problems**:
- ❌ Can't use static storage (MCU)
- ❌ Can't customize allocation
- ❌ "Managed language" design

### After (NEW - CORRECT)

```auto
// ✅ Storage-based design
type List<T, S: Storage> {
    len: u32
    store: S  // ← Abstract storage
}

// Different storage strategies
type Heap<T> { ptr: *T, cap: u32 }         // PC
type Inline<T, N> { buffer: [N]T }           // MCU
type ArenaRef<T> { ptr: *T, cap: u32, arena }  // Game
```

**Benefits**:
- ✅ Works on MCU (Inline)
- ✅ Customizable storage
- ✅ Zero-cost (monomorphization)
- ✅ System-level language design

---

## Risks & Mitigations

### R1: Type System Complexity

**Risk**: Const generic parameters complex to implement

**Mitigation**:
- Start with `Inline<T, N>` where N is literal
- Add const expression evaluation later
- Clear error messages for non-const params

### R2: C Transpiler Monomorphization

**Risk**: Generating specialized code for each (T, S) combination

**Mitigation**:
- Cache generated structs
- Inline small methods
- Use linker to eliminate duplicates

### R3: Storage Trait Enforcement

**Risk**: Duck typing may be too permissive

**Mitigation**:
- Clear documentation of Storage contract
- Compile-time checks where possible
- Comprehensive unit tests

### R4: Backwards Compatibility

**Risk**: Breaking existing List<T> code

**Mitigation**:
- `List<T>` defaults to `List<T, DefaultStorage<T>>`
- DefaultStorage adapts to environment
- Phase out old implementation gradually

---

## Time Estimate

| Phase | Duration | Dependencies |
|-------|----------|--------------|
| Phase 1: Infrastructure | 3-4 days | Pointer types, const generics |
| Phase 2: Storage | 3-4 days | Heap, Inline, ArenaRef |
| Phase 3: List | 3-4 days | List<T, S> implementation |
| Phase 4: Integration | 2-3 days | Environment injection |
| Phase 5: C Transpiler | 4-5 days | Monomorphization |
| Phase 6: Testing | 3-4 days | VM + A2C tests |
| Phase 7: Documentation | 1 day | Examples, CLAUDE.md |
| **Total** | **19-27 days** | |

---

## References

### Similar Designs

**Rust Vec<T>**:
- `src/alloc/vec/mod.rs` - Vec implementation
- `src/alloc/rawvec.rs` - RawVec (pointer + capacity)

**C++ std::vector<T>**:
- Based on allocators
- Direct pointer manipulation

**Why NOT Managed Languages**:
- Java `ArrayList<T>`: Built on `Object[]` (language array)
- Go `slice`: Built on underlying array (language feature)
- ❌ Both hide memory management from users

**System Languages**:
- Rust: `Vec<T>` = RawVec<T, Alloc> (explicit control)
- C++: `vector<T>` = allocator + pointer (explicit control)
- ✅ AutoLang should follow this pattern

---

## Status

**Current Phase**: 🔄 **Architecture Redesign**

**Completed**:
- ✅ Plan 052 Phase 1 (Runtime Array Syntax) - **DEPRECATED**
- ✅ Plan 056 (Field Access) - ✅ Completed

**In Progress**:
- 🔄 Plan 052 Phase 2 (Storage-Based List) - **NEW APPROACH**
  - ⏸️ Pointer types
  - ⏸️ Const generics
  - ⏸️ Storage module
  - ⏸️ List<T, S> implementation
  - ⏸️ C transpiler monomorphization

**Blocked By**:
- Need pointer type support
- Need const generic parameters
- Need Storage trait enforcement

**Priority**: 🔄 **HIGH** - Required for true system-level stdlib

**Next Actions**:
1. Implement pointer types (`*T`)
2. Implement const generic parameters (`const N u32`)
3. Create `stdlib/auto/storage.at`
4. Implement `List<T, S>` based on Storage
5. Update C transpiler for monomorphization

---

## Notes

**Key Insight**: The previous approach (`[expr]T`) was "managed language thinking". This new approach follows Rust/C++ philosophy: **explicit memory management + zero-cost abstractions**.

**Design Philosophy**:
> "Abstract without cost" - Storage abstraction compiles away, leaving only raw memory operations.

**User Experience**:
- Normal users: Just use `List<T>` (auto-adapts to platform)
- Power users: Choose `List<T, Inline<T, 128>>` for MCU performance
- Library authors: Implement custom Storage for specialized needs

This is the **correct way** to build system-level languages.
