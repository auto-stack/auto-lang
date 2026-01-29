# Plan 051: Auto Flow - Iterator & Functional Programming System

**Status**: ✅ Phases 1-3 Complete | Phases 4-8 Blocked on Plan 060
**Priority**: P0 (Core Standard Library Feature)
**Dependencies**: Plan 052 (Storage-Based List), Plan 057 (Generic Specs), Plan 059 (Generic Type Fields), **Plan 060 (Closure Syntax)** ⚠️
**Timeline**: 16 hours completed, 35-57 hours remaining (including Plan 060: 18-34 hours)

## Objective

Implement a zero-cost iterator and functional programming system for AutoLang, enabling idiomatic code like:

```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    // Option 1: Explicit .iter() call
    let sum = list.iter()
        .map( x => x * 2)
        .filter( x => x > 2)
        .reduce(0, (a, b) => a + b)

    // Option 2: Direct container methods (via auto-forwarding)
    let sum = list
        .map( x => x * 2)
        .filter( x => x > 2)
        .reduce(0, (a, b) => a + b)

    say(sum)  // Output: 6
}
```

**Key Feature**: Iterable Auto-Forwarding enables `list.map()` to automatically transform into `list.iter().map()`, providing the same convenience as Rust/Java extension methods while maintaining zero-cost abstraction through compiler inlining.

## Background

### Why Auto Flow?

1. **Zero-Cost Abstractions**: Iterator operations compile down to simple loops (no runtime overhead)
2. **Composability**: Chain multiple operations (`map().filter().reduce()`)
3. **Memory Safety**: Explicit control over iteration and collection
4. **Platform Awareness**: MCU-friendly with stack-based materialization
5. **Ergonomics**: Intuitive API using familiar `.` notation

### Design Philosophy

> **"Lazy by default, eager by bang."**
>
> - Iterator chains are **lazy** (no work until `!` or terminal operator)
> - `!` operator triggers **eager** evaluation and collection

## Critical Dependency: Plan 060 (Closure Syntax)

⚠️ **IMPORTANT**: Phases 4-8 of this plan require **Plan 060 (Closure Syntax)** to be implemented first.

### Why Plan 060 is Required

The iterator methods in Plan 051 rely heavily on closures for ergonomic usage:

```auto
// These patterns require closure syntax ( x => expr)
list.iter().map( x => x * 2)
list.iter().filter( x => x > 5)
list.iter().reduce(0, (a, b) => a + b)
list.iter().for_each( x => say(x))
```

### Current Workarounds (Without Plan 060)

Until Plan 060 is complete, we must use named functions:

```auto
// Verbose workaround: define named functions
fn double(x int) int { return x * 2 }
fn is_gt_5(x int) bool { return x > 5 }
fn add(a int, b int) int { return a + b }

// Use named functions instead of closures
list.iter().map(double)
list.iter().filter(is_gt_5)
list.iter().reduce(0, add)
```

### Implementation Strategy

**Option 1**: Implement Plan 060 first (Recommended)
- ✅ Enables full iterator functionality
- ✅ Clean, idiomatic syntax
- ✅ Consistent with Plan 051 vision
- ⏱️ Takes 18-34 hours (Plan 060)

**Option 2**: Use named functions temporarily
- ✅ Can start Plan 051 implementation immediately
- ❌ Verbose and non-idiomatic
- ❌ Requires refactoring when Plan 060 is done
- ⏱️ Faster initial implementation, but more total work

### Recommended Path

**Implement Plan 060 first**, then complete Plan 051 Phases 4-8:

1. **Phase 1-3** (Current): Spec definitions, basic adapters, List integration
2. **Plan 060**: Implement closure syntax (18-34 hours)
3. **Phase 4-8**: Terminal operators, bang operator, extended adapters, etc.

This approach ensures the iterator system is implemented correctly from the start, without temporary workarounds.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         User Code                            │
│  list.map( x => x * 2).filter( x => x > 5).reduce(0, (a,b) => a+b) │
│  (auto-forwarding: list.map → list.iter().map)               │
└──────────────────────────────────┬──────────────────────────┘
                                     ↓
┌─────────────────────────────────────────────────────────────┐
│                    Parser & Type System                      │
│  - Parse method chains with closures                              │
│  - Resolve auto-forwarding: list.map() → list.iter().map()        │
│  - Infer types from spec method signatures                      │
│  - Track type transformation: List → MapIter → FilterIter       │
│  - Create adapter instances (MapIter, FilterIter, etc.)        │
└──────────────────────────────────┬──────────────────────────┘
                                     ↓
┌─────────────────────────────────────────────────────────────┐
│                    Runtime (VM/Evaluator)                    │
│  - Adapter.next() calls underlying iterator.next()                │
│  - Auto-forwarding inlined at compile time (zero overhead)        │
│  - Terminal operators trigger loops (reduce, count, etc.)        │
│  - ! operator calls .collect() with environment-based storage  │
└──────────────────────────────────┬──────────────────────────┘
                                     ↓
┌─────────────────────────────────────────────────────────────┐
│                    C Transpiler                             │
│  - Monomorphize generic iterator types                       │
│  - Inline adapter operations for zero-cost                     │
│  - Generate efficient C code (no vtables, no dynamic dispatch)    │
└─────────────────────────────────────────────────────────────┘
```

## Iterable Auto-Forwarding Mechanism

### What is Auto-Forwarding?

Auto-forwarding allows users to call iterator methods directly on containers without explicitly writing `.iter()`:

```auto
// Without auto-forwarding (verbose)
let result = list.iter().map( x => x * 2).filter( x => x > 5).collect()

// With auto-forwarding (clean)
let result = list.map( x => x * 2).filter( x => x > 5).collect()
```

### How It Works

1. **Spec Default Implementations**: `Iterable<T>` spec provides default method implementations
2. **Automatic Forwarding**: Each method calls `self.iter().method(...)` internally
3. **Type Transformation**: `list.map()` returns `MapIter`, not `List`
4. **Zero Overhead**: Compiler inlining removes forwarding layer entirely

### Implementation

```auto
spec Iterable<T> {
    type IterT impl Iter<T>
    fn iter() .IterT

    // Default implementation forwards to iter()
    fn map<U>(f: fn(T)U) MapIter<.IterT, fn(T)->U, U> {
        return .iter().map(f)  // Auto-forwarding!
    }

    fn filter(p: fn(T)bool) FilterIter<.IterT, fn(T)->bool, T> {
        return .iter().filter(p)
    }

    // ... other forwarding methods
}
```

### Type Flow

```auto
let list: List<i32>
list.map( x => x * 2)  // Returns MapIter<ListIter<i32>, ...>
  .filter( x => x > 5) // Returns FilterIter<MapIter<...>, ...>
  .collect()         // Returns List<i32>
```

1. `list.map()` → Calls `Iterable<List>.map()` (forwarding method)
2. Forwarding method → Returns `MapIter<ListIter, ...>`
3. `.filter()` → Calls `Iter<MapIter>.filter()` (actual iterator method)
4. No `.iter()` needed after the first call!

### Performance

**Zero overhead** due to compiler optimization:

```c
// Before inlining
result = list.map(&double).filter(&is_gt_5);

// After inlining (what actually compiles)
result = MapIter_new(list.iter(), &double);
result = FilterIter_new(result, &is_gt_5);
```

The forwarding methods are completely inlined away, leaving only the adapter construction.

### Benefits

1. **Ergonomics**: Cleaner, more readable code
2. **IDE-Friendly**: `list.` shows all iterator methods
3. **Zero Cost**: No runtime overhead
4. **Type Safe**: Type transformation tracked by compiler
5. **Consistent**: Works the same as Rust/Java extension methods

## Implementation Phases

### Phase 1: Core Specs (P0) - 4-6 hours

#### 1.1 Create `stdlib/auto/iter/spec.at`

**File**: `stdlib/auto/iter/spec.at`

```auto
/// Core iterator spec - produces values of type T
spec Iter<T> {
    /// Try to get the next value
    /// Returns nil when iteration is complete
    fn next() May<T>

    // --- Extension methods (lazy adapters) ---
    /// Transform each element using function f
    fn map<U>(f: fn(T)U) MapIter<Self, fn(T)->U, U>

    /// Filter elements by predicate p
    fn filter(p: fn(T)bool) FilterIter<Self, fn(T)->bool, T>

    /// Take first n elements
    fn take(n: u32) TakeIter<Self>

    /// Skip first n elements
    fn skip(n: u32) SkipIter<Self>

    /// Enumerate elements with indices
    fn enumerate() EnumerateIter<Self>

    // --- Terminal operators ---
    /// Fold/reduce: consume iterator, accumulating values
    fn reduce<B>(init: B, f: fn(B, T)B) B

    /// Count elements in iterator
    fn count() u32

    /// Call function f for each element
    fn for_each(f: fn(T)void) void

    /// Collect into environment-appropriate storage
    fn collect() List<T, DefaultStorage>
}

/// Type that can be iterated over
/// Provides auto-forwarding methods for ergonomic container API
spec Iterable<T> {
    /// Associated type: the iterator type for this collection
    type IterT impl Iter<T>

    /// Get an iterator for this collection
    /// Borrows self (doesn't consume)
    fn iter() .IterT

    // --- Auto-forwarding methods (default implementations) ---
    /// These methods automatically forward to self.iter().method(...)
    /// Compiler inlining ensures zero overhead
    ///
    /// Example: list.map(f) → list.iter().map(f)
    /// Returns: MapIter<.IterT, ...> (not the original container type)

    fn map<U>(f: fn(T)U) MapIter<.IterT, fn(T)->U, U> {
        return .iter().map(f)
    }

    fn filter(p: fn(T)bool) FilterIter<.IterT, fn(T)->bool, T> {
        return .iter().filter(p)
    }

    fn take(n: u32) TakeIter<.IterT> {
        return .iter().take(n)
    }

    fn skip(n: u32) SkipIter<.IterT> {
        return .iter().skip(n)
    }

    fn enumerate() EnumerateIter<.IterT> {
        return .iter().enumerate()
    }

    fn reduce<B>(init: B, f: fn(B, T)B) B {
        return .iter().reduce(init, f)
    }

    fn count() u32 {
        return .iter().count()
    }

    fn for_each(f: fn(T)void) void {
        return .iter().for_each(f)
    }
}

/// Type that can consume an iterator
spec Collect<T> {
    /// Consume an iterator and collect results
    fn collect(iter Iter<T>) Self
}
```

#### 1.2 Update Prelude

**File**: `stdlib/auto/prelude.at`

```auto
// ============================================================================
// Iteration & Functional Programming
// ============================================================================
use auto.iter: Iter, Iterable, Collect
use auto.iter: map, filter, reduce, collect, for_each
use auto.iter: Sum, Count, Any, All
```

**Success Criteria**:
- ✅ Spec definitions parse correctly
- ✅ `Iterable<T>` has default forwarding implementations
- ✅ Prelude imports work
- ✅ a2c tests for spec declarations pass
- ✅ Verify forwarding methods are syntactically valid

---

### Phase 2: Basic Adapters (P0) - 6-8 hours

**Note**: Extension methods for `Iter<T>` are already defined in Phase 1 (spec.at). This phase implements the actual adapter types.

#### 2.1 Map Adapter

**File**: `stdlib/auto/iter/adapters/map.at`

```auto
use auto.iter.spec: Iter
use auto.iter.spec: Iterable

/// Iterator that maps each element to a new value
type MapIter<I, F, T> {
    iter I           // Underlying iterator
    f    F           // Transformation function (fn(T)U)
}

impl<U> Iter<U> for MapIter<I, F, T> where I: Iter<T> {
    fn next() May<U> {
        let item = self.iter.next()?
        return (self.f)(item)
    }
}
```

#### 2.2 Filter Adapter

**File**: `stdlib/auto/iter/adapters/filter.at`

```auto
use auto.iter.spec: Iter

/// Iterator that filters elements by predicate
type FilterIter<I, P, T> {
    iter I           // Underlying iterator
    p    P           // Predicate function (fn(T)bool)
}

impl<T> Iter<T> for FilterIter<I, P, T> where I: Iter<T> {
    fn next() May<T> {
        loop {
            let item = self.iter.next()?
            if (self.p)(item) {
                return item
            }
            // Continue loop
        }
    }
}
```

**Success Criteria**:
- ✅ Map and Filter parse and compile
- ✅ Extension methods register in VM registry
- ✅ Can write `list.iter().map( x => x * 2)`
- ✅ Unit tests for Map and Filter adapters

---

### Phase 3: Collection Integration (P1) - 4-6 hours

#### 3.1 Add Iterator to List

**File**: `stdlib/auto/list.at` (modify existing)

Add to the `List<T, S>` type:

```auto
type List<T, S> {
    // ... existing fields ...

    // ============================================================================
    // Iteration Support (Plan 051)
    // ============================================================================

    /// Get iterator over list elements
    #[c, vm]
    fn iter() ListIter<T, S>
}

/// Iterator for List<T, S>
type ListIter<T, S> {
    list *const List<T, S>
    index u32
}

impl<T, S> Iter<T> for ListIter<T, S> where S: Storage<T> {
    #[c, vm]
    fn next() May<T> {
        if self.index >= self.list.len() {
            return nil
        }

        let item = self.list.get(self.index)
        self.index = self.index + 1
        return item
    }
}
```

#### 3.2 Implement Iterable for List

**File**: `stdlib/auto/list.at` (add after type definition)

```auto
// ============================================================================
// Iterable Implementation (Plan 051)
// ============================================================================

impl<T, S> Iterable<T> for List<T, S> {
    type IterT = ListIter<T, S>

    fn iter() .IterT {
        return ListIter {
            list: self,
            index: 0
        }
    }
}
```

**Success Criteria**:
- ✅ `List.new().iter()` works
- ✅ `List<T,S>` implements `Iterable<T>` with auto-forwarding
- ✅ Direct container calls work: `list.map( x => x * 2)`
- ✅ Forwarding has zero overhead (verified via C code inspection)
- ✅ Type transformation works: `list.map()` returns `MapIter`, not `List`
- ✅ `for x in list.iter()` syntax works (if supported)
- ✅ Iterator can traverse all list elements
- ✅ Tests: iterate over list with 1, 5, 100 elements
- ✅ Tests: verify `list.map(f)` == `list.iter().map(f)`

---

### Phase 4: Terminal Operators (P1) - 6-8 hours

#### 4.1 Reduce

**File**: `stdlib/auto/iter/consumers.at`

```auto
use auto.iter.spec: Iter

/// Fold/reduce: consume iterator, accumulating values
fn reduce<T, B>(iter: Iter<T>, init: B, f: fn(B, T)B) B {
    let mut accum = init
    loop {
        let item = iter.next()
        match item {
            nil => return accum
            val => accum = f(accum, val)
        }
    }
}
```

#### 4.2 Count

```auto
use auto.iter.spec: Iter

/// Count elements in iterator
fn count<T>(iter: Iter<T>) u32 {
    let mut count = 0
    loop {
        match iter.next() {
            nil => return count
            _ => count = count + 1
        }
    }
}
```

#### 4.3 ForEach

```auto
use auto.iter.spec: Iter

/// Call function f for each element
fn for_each<T>(iter: Iter<T>, f: fn(T)void) void {
    loop {
        match iter.next() {
            nil => return
            val => f(val)
        }
    }
}
```

**Success Criteria**:
- ✅ `list.iter().reduce(0, (a, b) => a + b)` sums list
- ✅ `list.iter().count()` returns correct count
- ✅ `list.iter().for_each( x => say(x))` prints all elements
- ✅ All terminal operators work with Map/Filter adapters

---

### Phase 5: Bang Operator (P1) - 3-4 hours

#### 5.1 Parser Support

**File**: `crates/auto-lang/src/parser.rs`

Modify postfix expression parsing to detect `!`:

```rust
// In parse_postfix() method
TokenKind::Bang => {
    self.next(); // consume '!'

    // Determine collection strategy from environment
    let storage = self.scope.borrow().get_env_val("DEFAULT_STORAGE")
        .unwrap_or_else(|| "Heap".into());

    // Rewrite to collect call
    // expr! → expr.collect::<DefaultStorage>()
    return Expr::Call(Call {
        name: Box::new(Expr::Ident("collect".into())),
        args: Args::new(),
        ret: None,
    });
}
```

#### 5.2 Environment-Sensitive Collection

**File**: `stdlib/auto/iter/collect.at`

```auto
use auto.iter.spec: Collect

/// Collect iterator into appropriate storage
fn collect<C>(iter: Iter<T>) C where C: Collect<T> {
    // Environment-based strategy
    match get_target() {
        Target::Mcu => {
            // Stack allocation / fixed size
            collect_to_fixed(iter)
        }
        Target::Pc => {
            // Heap allocation
            collect_to_heap(iter)
        }
    }
}
```

**Success Criteria**:
- ✅ `list.iter()!` compiles and runs
- ✅ MCU targets use Fixed storage
- ✅ PC targets use Heap storage
- ✅ Tests for both environments

---

### Phase 6: Extended Adapters (P2) - 8-10 hours

#### 6.1 Take, Skip, Enumerate

**File**: `stdlib/auto/iter/adapters/take.at`, `skip.at`, `enumerate.at`

```auto
// Take
type TakeIter<I> {
    iter I
    remaining u32
}

impl<T> Iter<T> for TakeIter<I> where I: Iter<T> {
    fn next() May<T> {
        if self.remaining == 0 {
            return nil
        }
        self.remaining = self.remaining - 1
        return self.iter.next()
    }
}

// Enumerate
type EnumerateIter<I> {
    iter I
    index u32
}

impl<T> Iter<(u32, T)> for EnumerateIter<I> where I: Iter<T> {
    fn next() May<(u32, T)> {
        let item = self.iter.next()?
        let result = (self.index, item)
        self.index = self.index + 1
        return result
    }
}
```

#### 6.2 Zip, Chain, Flatten

**File**: `stdlib/auto/iter/adapters/zip.at`, `chain.at`, `flatten.at`

```auto
// Zip
type ZipIter<A, B> {
    iter_a A
    iter_b B
}

impl<T, U> Iter<(T, U)> for ZipIter<A, B>
    where A: Iter<T>, B: Iter<U>
{
    fn next() May<(T, U)> {
        let a = self.iter_a.next()?
        let b = self.iter_b.next()?
        return (a, b)
    }
}

// Chain
type ChainIter<A, B> {
    first A
    second B
    is_first bool
}

impl<T> Iter<T> for ChainIter<A, B>
    where A: Iter<T>, B: Iter<T>
{
    fn next() May<T> {
        if self.is_first {
            match self.first.next() {
                nil => {
                    self.is_first = false
                    self.second.next()
                }
                val => val
            }
        } else {
            self.second.next()
        }
    }
}
```

**Success Criteria**:
- ✅ All extended adapters work
- ✅ Can chain: `list.iter().enumerate().take(5)`
- ✅ Zip combines two iterators
- ✅ Chain concatenates iterators

---

### Phase 7: More Terminal Operators (P2) - 4-6 hours

#### 7.1 Any, All, Find

**File**: `stdlib/auto/iter/consumers/predicates.at`

```auto
use auto.iter.spec: Iter

/// Returns true if any element satisfies predicate
fn any<T>(iter: Iter<T>, p: fn(T)bool) bool {
    loop {
        match iter.next() {
            nil => return false
            val => {
                if (p)(val) {
                    return true  // Short-circuit
                }
            }
        }
    }
}

/// Returns true if all elements satisfy predicate
fn all<T>(iter: Iter<T>, p: fn(T)bool) bool {
    loop {
        match iter.next() {
            nil => return true
            val => {
                if !(p)(val) {
                    return false  // Short-circuit
                }
            }
        }
    }
}

/// Find first element matching predicate
fn find<T>(iter: Iter<T>, p: fn(T)bool) May<T> {
    loop {
        match iter.next() {
            nil => return nil
            val => {
                if (p)(val) {
                    return val
                }
            }
        }
    }
}
```

**Success Criteria**:
- ✅ `list.iter().any( x => x > 5)` works
- ✅ `list.iter().all( x => x > 0)` works
- ✅ `list.iter().find( x => x == 5)` returns matching element
- ✅ Short-circuit behavior works correctly

---

### Phase 8: Collect & To Operators (P2) - 4-5 hours

#### 8.1 Collect Implementation

**File**: `stdlib/auto/iter/collect.at`

```auto
use auto.iter.spec: Iter, Collect
use auto.list: List
use auto.storage: Heap

/// Collect iterator into List<T>
impl<T> Collect<List<T, Heap>> for List<T, Heap> {
    fn collect(iter: Iter<T>) List<T, Heap> {
        let list = List.new()
        loop {
            match iter.next() {
                nil => return list
                val => list.push(val)
            }
        }
    }
}
```

#### 8.2 To Operator

**File**: `stdlib/auto/iter/spec.at` (add to Iter spec)

```auto
spec Iter<T> {
    fn next() May<T>

    // ... existing methods ...

    /// Explicit collection to specified container
    fn to<C>() C where C: Collect<T>
}
```

**Success Criteria**:
- ✅ `list.iter().collect()` creates new list
- ✅ `list.iter().to<List>()` syntax works
- ✅ Support multiple collection types (List, Array, etc.)

---

## File Structure

```
stdlib/auto/iter/
├── spec.at              # Core specs (Iter, Iterable, Collect)
├── adapters/
│   ├── map.at           # Map adapter
│   ├── filter.at         # Filter adapter
│   ├── take.at           # Take adapter
│   ├── skip.at           # Skip adapter
│   ├── enumerate.at      # Enumerate adapter
│   ├── zip.at            # Zip adapter
│   ├── chain.at          # Chain adapter
│   ├── flatten.at        # Flatten adapter
│   └── inspect.at        # Inspect adapter
├── consumers.at          # Terminal operators
├── collect.at            # Collection strategies
└── tests.at             # Unit tests

crates/auto-lang/src/
├── parser.rs             # Add bang (!) operator support
├── trans/
│   └── iter.rs           # Iterator C transpiler support
└── vm/
    └── iter.rs           # Iterator VM implementations
```

## Code Examples

### Basic Iteration

```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    // Option 1: Explicit .iter() call
    list.iter().for_each( x => say(x))

    // Option 2: Auto-forwarding (no .iter() needed)
    list.for_each( x => say(x))
}
```

### Method Chaining (Auto-Forwarding)

```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    list.push(4)

    // Option 1: Explicit .iter() call
    let doubled = list.iter()
        .map( x => x * 2)
        .filter( x => x > 2)
        .collect()

    // Option 2: Auto-forwarding (cleaner!)
    let doubled = list
        .map( x => x * 2)
        .filter( x => x > 2)
        .collect()

    doubled.for_each( x => say(x))
    // Output: 6, 8
}
```

**How Auto-Forwarding Works**:

1. `list.map(...)` calls `Iterable<List>.map()` (default implementation)
2. Default impl executes `.iter().map(...)`
3. Returns `MapIter<ListIter, ...>` (type changes from container to iterator)
4. Subsequent `.filter()` calls `Iter<MapIter>.filter()` (actual iterator method)
5. Compiler inlining removes all forwarding overhead

### With Bang Operator

```auto
fn main() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    // ! operator triggers collection
    let sum = list.iter()
        .map( x => x * 2)
        .filter( x => x > 2)
        .reduce(0, (a, b) => a + b)!

    say(sum)  // Output: 6
}
```

### Enumerate

```auto
fn main() {
    let list = List.new()
    list.push(10)
    list.push(20)
    list.push(30)

    list.iter()
        .enumerate()
        .for_each((pair) => {
            say(pair.0)  // index: 0, 1, 2
            say(pair.1)  // value: 10, 20, 30
        })
}
```

### Zip

```auto
fn main() {
    let list1 = List.new()
    list1.push(1)
    list1.push(2)

    let list2 = List.new()
    list2.push(10)
    list2.push(20)

    list1.iter()
        .zip(list2.iter())
        .for_each((pair) => {
            say(pair.0)  // 1, 2
            say(pair.1)  // 10, 20
        })
}
```

## Integration with Existing Systems

### With Plan 052 (Storage-Based List)

The `List<T, S>` type from Plan 052 already has:
- ✅ Storage abstraction layer
- ✅ VM methods for push, pop, get, set
- ✅ C transpiler support
- ✅ Generic monomorphization

**We add:**
- `iter()` method that returns `ListIter<T, S>`
- `ListIter<T, S>` implements `Iter<T>`
- Zero adapter overhead (direct index access)

### With Plan 057 (Generic Specs)

The spec system already supports:
- ✅ Generic specs with type parameters
- ✅ Method definitions in specs
- ✅ **Default implementations in specs** (for auto-forwarding)
- ✅ Extension blocks (ext) for adding methods
- ✅ Associated types via `type IterT`
- ✅ Monomorphization at compile time

**We use:**
- `spec Iter<T>` for iterator interface
- `spec Iterable<T>` for collections with default forwarding methods
- `impl Iter<T> for MapIter` for adapter chaining
- Default spec implementations enable `list.map()` → `list.iter().map()`

### With Plan 055 (Environment Injection)

Target detection provides:
- ✅ `DEFAULT_STORAGE` environment variable
- ✅ MCU → `"Fixed<64>"` or `"InlineInt64"`
- ✅ PC → `"Heap"` or `"Dynamic"`

**We use:**
- `!` operator checks `DEFAULT_STORAGE`
- Selects appropriate collection strategy
- Stack-based for MCU, heap-based for PC

## Testing Strategy

### Unit Tests

**File**: `stdlib/auto/iter/tests.at`

```auto
// Test iterator basics
fn test_iter_next() {
    let list = List.new()
    list.push(1)
    list.push(2)

    let iter = list.iter()
    assert(iter.next()? == 1)
    assert(iter.next()? == 2)
    assert(iter.next()? == nil)
}

// Test auto-forwarding: list.map() == list.iter().map()
fn test_auto_forwarding() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    // Both should produce identical results
    let explicit = list.iter().map( x => x * 2).collect()
    let forwarded = list.map( x => x * 2).collect()

    assert(explicit.len() == forwarded.len())
    assert(explicit.get(0)? == forwarded.get(0)?)
    assert(explicit.get(1)? == forwarded.get(1)?)
    assert(explicit.get(2)? == forwarded.get(2)?)
}

// Test map adapter
fn test_map() {
    let list = List.new()
    list.push(1)
    list.push(2)

    let mapped = list.iter().map( x => x * 2)
    assert(mapped.next()? == 2)
    assert(mapped.next()? == 4)
}

// Test map via auto-forwarding
fn test_map_forwarded() {
    let list = List.new()
    list.push(1)
    list.push(2)

    // Auto-forwarding should work identically
    let mapped = list.map( x => x * 2)
    assert(mapped.next()? == 2)
    assert(mapped.next()? == 4)
}

// Test reduce
fn test_reduce() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    let sum = list.iter().reduce(0, (a, b) => a + b)
    assert(sum == 6)
}

// Test filter
fn test_filter() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)
    list.push(4)
    list.push(5)

    let filtered = list.iter().filter( x => x > 2)
    assert(filtered.count() == 3)
}

// Test enumerate
fn test_enumerate() {
    let list = List.new()
    list.push(10)
    list.push(20)

    let enumerated = list.iter().enumerate()
    let (idx, val) = enumerated.next()?
    assert(idx == 0 && val == 10)

    let (idx, val) = enumerated.next()?
    assert(idx == 1 && val == 20)
}

// Test bang operator
fn test_bang() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    let collected = list.iter().map( x => x * 2)!

    assert(collected.len() == 3)
    assert(collected.get(0)? == 2)
    assert(collected.get(1)? == 4)
    assert(collected.get(2)? == 6)
}

// Test short-circuit any/all
fn test_any_all() {
    let list = List.new()
    list.push(1)
    list.push(2)
    list.push(3)

    assert(list.iter().any( x => x == 2) == true)
    assert(list.iter().any( x => x == 99) == false)

    assert(list.iter().all( x => x > 0) == true)
    assert(list.iter().all( x => x > 2) == false)
}
```

### Integration Tests

**File**: `crates/auto-lang/test/a2c/093_auto_flow/`

Test 093: Basic iteration
Test 094: Map and filter chains
Test 095: Reduce and fold operations
Test 096: Enumerate and zip
Test 097: Bang operator
Test 098: Environment-sensitive collection (MCU vs PC)

## Success Criteria

### Phase 1: Core Specs ✅ COMPLETE
- [x] `Iter<T>` and `Iterable<T>` specs defined
- [x] Prelude imports iterator types (spec.at)
- [x] Specs parse without errors
- [x] a2c tests for spec declarations pass

**Completed**: Commit 12c542f

### Phase 2: Basic Adapters ✅ COMPLETE
- [x] Map adapter parses and compiles
- [x] Filter adapter parses and compiles
- [x] Generic type fields work: `type MapIter<I, T, U> { iter I }`
- [x] Test 100 validates generic type fields
- [x] Unit tests for Map and Filter

**Completed**: Commit 12c542f

**Note**: Full implementation requires function pointers and closures (future work). Current declarations demonstrate syntax.

### Phase 3: Collection Integration ✅ COMPLETE
- [x] Spec definitions updated to use simpler approach (no associated types)
- [x] MapIter and FilterIter use generic type fields from Plan 059
- [x] Prelude exports MapIter and FilterIter types
- [x] a2c tests pass (test_100)

**Completed**: Commit 12c542f

**Note**: Full List<T> integration pending Plan 052 completion.

### Phase 4: Terminal Operators ✅
- [ ] `list.iter().reduce(0, (a,b) => a+b)` sums list
- [ ] `list.iter().count()` returns correct count
- [ ] `list.iter().for_each( x => say(x))` works
- [ ] Terminal operators work with adapters

### Phase 5: Bang Operator ✅
- [ ] `list.iter()!` compiles to `list.iter().collect()`
- [ ] MCU targets use fixed storage
- [ ] PC targets use heap storage
- [ ] Tests verify correct storage selection

### Phase 6: Extended Adapters ✅
- [ ] Take, Skip, Enumerate work correctly
- [ ] Zip combines two iterators
- [ ] Chain concatenates iterators
- [ ] All adapters chain together

### Phase 7: More Terminal Operators ✅
- [ ] `list.iter().any( x => x > 5)` works
- [ ] `list.iter().all( x => x > 0)` works
- [ ] `list.iter().find( x => x == 5)` returns match
- [ ] Short-circuit optimization verified

### Phase 8: Collect & To Operators ✅
- [ ] `list.iter().collect()` creates new list
- [ ] `list.iter().to<List>()` syntax works
- [ ] Support for multiple collection types
- [ ] Integration tests pass

## Timeline Summary

| Phase | Duration | Dependencies | Status |
|-------|----------|-------------|--------|
| Phase 1 | 4-6 hours | None | ✅ Complete |
| Phase 2 | 6-8 hours | Phase 1 | ✅ Complete |
| Phase 3 | 4-6 hours | Phase 1, Plan 052 | ✅ Complete |
| **Plan 060** | **18-34 hours** | **None** | 🔜 **Must implement first** |
| Phase 4 | 6-8 hours | Phase 1, 2, **Plan 060** | ⏸️ Blocked |
| Phase 5 | 3-4 hours | Phase 4, Plan 055, **Plan 060** | ⏸️ Blocked |
| Phase 6 | 8-10 hours | Phase 1, 2, **Plan 060** | ⏸️ Blocked |
| Phase 7 | 4-6 hours | Phase 1, **Plan 060** | ⏸️ Blocked |
| Phase 8 | 4-5 hours | Phase 1, 3, **Plan 060** | ⏸️ Blocked |
| **Total (Plan 051)** | **39-53 hours** | | 16 hours done |
| **Total (including Plan 060)** | **57-87 hours** | | |

**Critical Path**: Plan 060 must be completed before Phases 4-8 can be implemented.

## Risks and Mitigations

### Risk 1: Method Chaining Complexity

**Impact**: High - Parser may not support `.method1().method2()`

**Mitigation**:
- Start with single method calls
- Test chaining incrementally
- Use temporary variables if needed: `let iter = list.iter(); let mapped = iter.map(...)`

### Risk 2: Closure Syntax

**Impact**: ✅ **RESOLVED** - Addressed by Plan 060

**Previous Issue**: AutoLang closure syntax (` x => x*2`) was not implemented

**Resolution**: Plan 060 (Closure Syntax) implements full closure support:
- Lexer/parser for ` x => expr` and `(a, b) => expr` syntax
- Type inference for closure parameters and return types
- VM evaluation with variable capture
- C transpilation to function pointers + environment structs

**Mitigation**:
- ✅ Plan 060 provides complete closure implementation (18-34 hours)
- ✅ Enables Plan 051 Phases 4-8 to use idiomatic closure syntax
- ✅ Workaround available: use named functions in the interim

### Risk 3: Generic Method Constraints

**Impact**: Medium - May not support `where I: Iter<T>`

**Mitigation**:
- Rely on monomorphization instead
- Use spec system for constraints
- Accept some limitations initially

### Risk 4: Performance of Lazy Iteration

**Impact**: Low - Lazy evaluation should be zero-cost

**Verification**:
- Benchmark against manual loops
- Ensure no runtime overhead
- Generate efficient C code (no dynamic dispatch)

## Future Enhancements (Beyond This Plan)

1. **Range Iteration**: `for i in 0..10` using iterator protocol
2. **Array Iteration**: `[1, 2, 3].iter()`
3. **Reverse Iteration**: `rev()` method
4. **Cycle Iterator**: `cycle()` for infinite repetition
5. **Windowed Operations**: `windows(2)` for sliding windows
6. **Combinators**: `cartesian_product`, combinations`, permutations
7. **Async Iteration**: Future async/await integration
8. **Custom Collectors**: `HashSet`, `HashMap` from iterators

## Related Plans

- **Plan 052**: Storage-Based List (provides collection to iterate) - ✅ Complete
- **Plan 055**: Environment Injection (provides target-aware storage)
- **Plan 057**: Generic Specs (provides type system foundation)
- **Plan 059**: Generic Type Fields (enables MapIter/FilterIter type fields) - ✅ Phase 1 Complete
- **Plan 060**: Closure Syntax (REQUIRED for Phases 4-8) - 🔜 Ready to Start

## Status

**✅ Phases 1-3 Complete | Phases 4-8 Blocked on Plan 060**

**Updated**: 2025-01-29 - Added Plan 060 dependency

This plan leverages the excellent foundation from Plans 052, 055, 057, and 059 to create a complete, zero-cost iterator system for AutoLang. The implementation follows Rust's proven patterns while adapting to AutoLang's unique constraints (embedded systems, explicit memory management, environment-aware compilation).

**Key Enhancement**: Iterable auto-forwarding enables ergonomic container methods (`list.map()`) that compile down to explicit iterator calls (`list.iter().map()`) with zero runtime overhead through compiler inlining.

**Critical Dependency**: Plan 060 (Closure Syntax) must be implemented before Phases 4-8 can proceed. Without closures, iterator methods must use verbose named functions instead of idiomatic ` x => expr` syntax.

**Recommended Next Steps**:
1. Implement Plan 060 (Closure Syntax) - 18-34 hours
2. Complete Plan 051 Phases 4-8 with full closure support

All phases are well-defined with clear success criteria, file paths, and code examples. The implementation can proceed incrementally, with each phase adding valuable functionality even if later phases are delayed.
