# Ownership-Based Memory System Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** FOUNDATIONAL - Must complete before String Types (Plan 024)
**Dependencies:** None (this IS the foundation)
**Estimated Start:** Immediate (blocks Plan 024 and all subsequent plans)

## Executive Summary

Implement AutoLang's ownership-based memory management system as described in [new_memory.md](../language/design/new_memory.md). This provides the foundation for safe, zero-cost memory management without GC, enabling proper string types, collections, and concurrency.

**Timeline:** 4-5 months (hybrid approach)
**Complexity:** Very High (requires borrow checker, control flow analysis)
**Priority:** CRITICAL - Blocks Plan 024 (Strings) and Plan 025 (Stdlib)

---

## 1. Why Memory System Must Come First

### 1.1 Current State: No Memory Management

**Current Reality:**
- ✅ NO ownership system exists
- ✅ NO borrowing mechanisms
- ✅ NO lifetime tracking
- ✅ NO linear types
- ✅ Variables freely clone via `Rc<RefCell<ValueData>>`
- ✅ Evaluator clones values on every assignment

**How Variables Work Now:**
```rust
// From eval.rs:726-740
fn eval_store(&mut self, store: &Store) -> Value {
    let value = self.eval_expr(&store.expr);
    // Simply stores value by CLONING
    self.universe.borrow_mut().set_local_val(&store.name, value);
}
```

**Problem:** Every assignment copies data, no ownership semantics

### 1.2 Plan 024 Compatibility Issue

**Plan 024's Approach (Manual Lifetime Tracking):**
```c
// Manual lifetime tracking via global registry
StringSlice String_slice(String* s) {
    return (StringSlice){
        ._lifetime = s->_owner_id  // Manual tracking
    };
}

bool StringSlice_is_valid(StringSlice* sl) {
    return owner_is_alive(sl->_lifetime);  // Runtime check
}
```

**Ownership System Approach (Compile-Time Safety):**
```rust
// Compiler tracks lifetimes with NO runtime cost
let s = String_new("hello", 5);
let slice = take s;  // Compiler: slice borrows from s
// Compile-time guarantee: slice lives shorter than s
```

**The Conflict:**
| Aspect | Plan 024 | Ownership System |
|--------|----------|------------------|
| Lifetime checks | Runtime | Compile-time |
| Cleanup | Manual `drop()` | Automatic (linear types) |
| Tracking | Global array | Compiler analysis |
| Performance | Overhead per access | Zero-cost |
| Safety | Runtime panic | Compile-time error |

**Critical Finding:** Plan 024's manual lifetime tracking is **fundamentally incompatible** with the planned ownership system. 60-70% of Plan 024 would be throwaway work.

### 1.3 Strategic Decision

**Three Options:**

**Option A: Memory First (6 months)**
- ✅ Foundation for everything
- ✅ No rework
- ❌ Longer to working code

**Option B: Strings First (2.5 months)**
- ✅ Faster results
- ❌ 60-70% throwaway work
- ❌ Creates technical debt

**Option C: Hybrid (4 months) ⭐ RECOMMENDED**
- ✅ Minimal rework
- ✅ Incremental value
- ✅ Working strings in 3 months
- ✅ Matches long-term vision

---

## 2. Hybrid Approach: Three Phases

### Phase 1: Move Semantics (6 weeks)

**Objective:** Basic linear types with move-only semantics

**What It Provides:**
- Variables own data (no implicit cloning)
- Automatic move on "last use"
- `String` can be owned and moved
- NO borrowing yet (that's Phase 3)

**Key Features:**
```auto
let s = String_new("hello", 5)  // Owns string
let t = s                        // Move: s no longer valid
use(t)                            // Last use: automatic cleanup
```

**Deliverables:**
- Linear type system (moves only)
- "Last use" detection via control flow analysis
- Automatic cleanup on drop
- Stack unwinding for early returns

**Success Criteria:**
- Variables move on last use
- No memory leaks (valgrind clean)
- `String` type works with moves
- 50+ tests passing

---

### Phase 2: Owned Strings (6 weeks)

**Objective:** Implement owned String type using move semantics

**What It Provides:**
- `String` type with move semantics
- `CString` for C FFI
- UTF-8 support
- NO borrowing (no `StringSlice` yet)

**Key Features:**
```auto
let mut s = String_new("hello", 5)
String_push(mut s, ' ')
String_append(mut s, "world", 5)  // Takes ownership

let cs = CString_from_string(s)   // Move: s consumed
c_printf(CString_data(cs))
```

**String Type Design:**
```auto
extern type String {
    data *char     // Heap-allocated UTF-8
    len uint       // Byte length
    cap uint       // Capacity
}

// All operations consume or move
fn String_new(utf8 str, len uint) Result<String, str>
fn String_append(mut s String, other String) String  // Takes ownership
fn String_push(mut s String, c char)
fn String_drop(s String)  // Automatic via linear type
```

**Deliverables:**
- Owned `String` type (move semantics only)
- `CString` for C FFI
- UTF-8 validation
- 100+ tests passing

**Limitations:**
- No `StringSlice` (borrows not available)
- No substring views (must copy)
- No `take`/`edit` (that's Phase 3)

---

### Phase 3: Borrow Checker (8 weeks)

**Objective:** Complete ownership system with borrowing

**What It Provides:**
- `take` (immutable borrow)
- `edit` (mutable borrow)
- `StringSlice` with compile-time lifetimes
- `hold` path binding
- Zero-cost safety

**Key Features:**
```auto
let s = String_new("hello", 5)
let slice = take s              // Immutable borrow
let len = StringSlice_len(slice)

hold path s.data as bytes {
    edit bytes[0] = 'H'        // Mutable borrow via path
}
// s still valid here (borrows ended)
```

**StringSlice Design:**
```auto
extern type StringSlice {
    data *char     // Borrowed data
    len uint
    // NO _lifetime field - compiler tracks this!
}

fn String_slice(s String) StringSlice  // Compiler: s borrowed
fn StringSlice_len(sl StringSlice) uint
// Compiler guarantees sl lives shorter than source
```

**Deliverables:**
- Full borrow checker
- `take`/`edit` keywords
- `StringSlice` with compile-time lifetimes
- `hold` path binding
- 200+ tests passing
- Zero runtime overhead for borrows

---

## 3. Implementation Plan

### Phase 1: Move Semantics (6 weeks)

#### Week 1-2: Linear Types Foundation

**Files to Create:**
- `crates/auto-lang/src/ownership/mod.rs` - Ownership module
- `crates/auto-lang/src/ownership/linear.rs` - Linear type system

**Key Implementation:**

```rust
// ownership/linear.rs

/// Marker trait for linear types (move-only)
pub trait Linear {
    fn drop_linear(&mut self);
}

/// Tracks whether a value has been moved
#[derive(Debug, Clone, PartialEq)]
pub enum MoveState {
    Available,
    Moved,
}

/// Type-level move tracking
pub struct MoveTracker<T> {
    value: Option<T>,
    state: MoveState,
}

impl<T> MoveTracker<T> {
    pub fn new(value: T) -> Self {
        Self {
            value: Some(value),
            state: MoveState::Available,
        }
    }

    pub fn take(&mut self) -> T {
        assert!(self.state == MoveState::Available, "Use after move");
        self.state = MoveState::Moved;
        self.value.take().unwrap()
    }

    pub fn is_available(&self) -> bool {
        self.state == MoveState::Available
    }
}
```

**AST Changes:**

```rust
// ast/types.rs
pub enum Type {
    // ... existing types

    // New: Linear type marker
    Linear(Box<Type>),
}
```

#### Week 3-4: "Last Use" Detection

**Control Flow Analysis:**

```rust
// ownership/cfa.rs

use crate::ast::Stmt;
use crate::ast::Expr;

pub struct LastUseAnalyzer {
    last_uses: HashMap<Name, HashSet<ExprId>>,
}

impl LastUseAnalyzer {
    pub fn analyze(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let { name, init, .. } => {
                self.analyze_expr(init);
                self.mark_last_use(name);
            }
            Stmt::Expr { expr } => {
                self.analyze_expr(expr);
            }
            _ => {}
        }
    }

    fn mark_last_use(&mut self, name: &Name) {
        // Insert move operation after this expression
    }
}
```

**Evaluator Changes:**

```rust
// eval.rs - Modified to use move semantics

fn eval_store(&mut self, store: &Store) -> Value {
    let value = self.eval_expr(&store.expr)?;

    // Check if this is a reassignment
    if self.universe.borrow().has_local(&store.name) {
        // Old value is dropped here (last use)
        self.universe.borrow_mut().remove_local(&store.name);
    }

    // Store new value (takes ownership)
    self.universe.borrow_mut().set_local_val(&store.name, value);
    Ok(Value::Nil)
}
```

#### Week 5-6: Drop Glue & Testing

**Automatic Cleanup:**

```rust
// Ownership-aware scope exit

impl Universe {
    pub fn exit_scope(&mut self) {
        // Drop all locals in reverse order
        for (name, value) in self.locals.drain(..).rev() {
            if let Some(linear) = value.as_linear() {
                linear.drop_linear();
            }
        }
    }
}
```

**Testing:**

```auto
// tests/ownership/move_semantics.at

fn test_move_basic() {
    let s = String_new("hello", 5).unwrap()
    let t = s  // Move: s no longer valid
    assert(String_len(t) == 5)
}

fn test_last_use_detection() {
    let s = String_new("hello", 5).unwrap()
    consume(s)  // Last use: automatically dropped
}

fn consume(s String) {
    assert(String_len(s) == 5)
}
```

**Success Criteria:**
- [ ] Linear type trait implemented
- [ ] "Last use" detection working
- [ ] Move semantics enforced
- [ ] Automatic cleanup on scope exit
- [ ] 50+ tests passing

---

### Phase 2: Owned Strings (6 weeks)

#### Week 1-2: Core String Type

**Files to Create:**
- `stdlib/string/string.h` - C header
- `stdlib/string/string.c` - C implementation
- `stdlib/string/string.at` - AutoLang interface

**String Implementation:**

```c
// stdlib/string/string.h

#ifndef AUTO_STRING_H
#define AUTO_STRING_H

#include <stddef.h>
#include <stdbool.h>

typedef struct {
    char* data;      // UTF-8 encoded bytes
    size_t len;      // Byte length (not char count)
    size_t cap;      // Capacity (for growth)
} String;

// Lifecycle
String* String_new(const char* utf8, size_t len);
void String_drop(String* s);  // Called automatically by linear type system

// Accessors
const char* String_data(String* s);
size_t String_len(String* s);
size_t String_char_len(String* s);  // UTF-8 char count

// Modification (all take &self)
String* String_append(String* s, const char* utf8, size_t len);
String* String_push(String* s, char c);

// Conversion
char* String_to_cstr(String* s);  // Returns null-terminated copy

// UTF-8 validation
bool String_is_valid_utf8(String* s);

#endif
```

**AutoLang Interface:**

```auto
// stdlib/string/string.at
# C
#include "string.h"

extern type String {
    data *char
    len uint
    cap uint
}

// Constructors
spec extern String_new(utf8 str, len uint) Result<String, str>

// Accessors (no &self - all take ownership)
spec extern String_data(s String) *char
spec extern String_len(s String) uint
spec extern String_char_len(s String) uint

// Modification (take mut self)
spec extern String_append(mut s String, utf8 str, len uint) String
spec extern String_push(mut s String, c char) String

// Conversion
spec extern String_to_cstr(s String) *char
```

**Usage Examples:**

```auto
// tests/string/test_string.at

fn test_string_creation() {
    let s = String_new("hello", 5).unwrap()
    assert(String_len(s) == 5)
}

fn test_string_append() {
    let mut s = String_new("hello", 5).unwrap()
    s = String_append(mut s, " world", 6)  // Takes ownership
    assert(String_len(s) == 11)
}

fn test_string_move() {
    let s = String_new("hello", 5).unwrap()
    let t = s  // Move
    assert(String_len(t) == 5)
    // s no longer valid here
}
```

#### Week 3-4: String Operations

**Additional Functions:**

```c
// string.h

// Comparison
bool String_equals(String* a, String* b);

// Substring (returns new String - must copy)
String* String_substring(String* s, size_t start, size_t end);

// Splitting
String** String_split(String* s, char delimiter, size_t* count);

// UTF-8 operations
size_t String_char_count(String* s);  // Count Unicode codepoints
char* String_get_char(String* s, size_t byte_idx);
```

#### Week 5-6: C Integration

**CString Type:**

```c
// stdlib/string/cstring.h

typedef struct {
    char* data;      // Null-terminated
    size_t len;
} CString;

CString* CString_new(const char* data, size_t len);
void CString_drop(CString* cs);

const char* CString_data(CString* cs);  // Guaranteed null-terminated
size_t CString_len(CString* cs);

// Conversions
CString* CString_from_string(String* s);
String* CString_to_string(CString* cs);
```

**Usage:**

```auto
// tests/cstring/test_cstring.at

extern fn.c printf(fmt *char, ...)

fn test_c_ffi() {
    let s = String_new("Hello, world!\n", 14).unwrap()
    let cs = CString_from_string(s)
    printf(CString_data(cs))
    // cs is dropped here
}
```

**Success Criteria:**
- [ ] `String` type functional with move semantics
- [ ] `CString` for C FFI working
- [ ] UTF-8 validation
- [ ] 100+ tests passing
- [ ] Zero memory leaks

---

### Phase 3: Borrow Checker (8 weeks)

#### Week 1-3: Borrow Checking Core

**Files to Create:**
- `crates/auto-lang/src/ownership/borrow.rs` - Borrow checker
- `crates/auto-lang/src/ownership/lifetime.rs` - Lifetime inference

**Lifetime Inference:**

```rust
// ownership/lifetime.rs

use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Lifetime(u32);

impl Lifetime {
    pub const STATIC: Lifetime = Lifetime(0);
    pub fn new() -> Lifetime => Lifetime(1);
}

pub struct LifetimeContext {
    counter: u32,
    regions: HashMap<ExprId, Lifetime>,
}

impl LifetimeContext {
    pub fn new() -> Self {
        Self {
            counter: 1,
            regions: HashMap::new(),
        }
    }

    pub fn fresh_lifetime(&mut self) -> Lifetime {
        let l = Lifetime(self.counter);
        self.counter += 1;
        l
    }

    pub fn assign_lifetime(&mut self, expr_id: ExprId, lt: Lifetime) {
        self.regions.insert(expr_id, lt);
    }
}
```

**Borrow Checker:**

```rust
// ownership/borrow.rs

use crate::ast::Expr;
use crate::ownership::lifetime::Lifetime;

pub enum BorrowKind {
    Immutable(Take),
    Mutable(Edit),
}

pub struct Borrow {
    pub kind: BorrowKind,
    pub lifetime: Lifetime,
    pub expr: Expr,
}

pub struct BorrowChecker {
    borrows: Vec<Borrow>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            borrows: Vec::new(),
        }
    }

    pub fn check_borrow(&mut self, expr: &Expr, kind: BorrowKind) -> Result<(), String> {
        // Check if expr can be borrowed
        // Validate against existing borrows
        Ok(())
    }
}
```

**AST Extensions:**

```rust
// ast/expr.rs

pub enum Expr {
    // ... existing variants

    // New: Borrow expressions
    Take(Box<Expr>),      // Immutable borrow
    Edit(Box<Expr>),      // Mutable borrow
}
```

**Parser Changes:**

```rust
// parser.rs - Add take/edit parsing

fn parse_expr(&mut self) -> Result<Expr, Error> {
    // Check for take/edit keywords
    match self.cur.kind {
        TokenKind::Take => {
            self.advance();
            let expr = self.parse_expr()?;
            Ok(Expr::Take(Box::new(expr)))
        }
        TokenKind::Edit => {
            self.advance();
            let expr = self.parse_expr()?;
            Ok(Expr::Edit(Box::new(expr)))
        }
        _ => self.parse_expr_with_prec(0)
    }
}
```

**Usage:**

```auto
// tests/borrow/test_borrow.at

fn test_take_borrow() {
    let s = String_new("hello", 5).unwrap()
    let slice = take s  // Immutable borrow
    assert(String_len(slice) == 5)
    // s still valid here
}

fn test_edit_borrow() {
    let mut s = String_new("hello", 5).unwrap()
    {
        let view = edit s  // Mutable borrow
        String_push(mut view, 'X')  // Modify through borrow
    }
    // s now has 'X'
}
```

#### Week 4-5: String Slices

**StringSlice Implementation:**

```auto
// stdlib/string/slice.at

extern type StringSlice {
    data *char     // Borrowed data
    len uint
    // NO _lifetime field - compiler tracks this!
}

spec extern String_slice(s String) StringSlice  // Compiler tracks lifetime
spec extern StringSlice_len(sl StringSlice) uint
spec extern StringSlice_subslice(sl StringSlice, start uint, end uint) Result<StringSlice, str>
```

**Compiler Integration:**

```rust
// trans/c.rs - StringSlice transpilation

fn transpile_expr(&mut self, expr: &Expr) -> String {
    match expr {
        Expr::Take(inner) => {
            // Generate borrowed reference
            let inner = self.transpile_expr(inner)?;
            Ok(format!("&{}", inner))
        }
        _ => { /* ... */ }
    }
}
```

**Usage:**

```auto
// tests/slice/test_slice.at

fn test_string_slice() {
    let s = String_new("hello world", 11).unwrap()
    let slice = String_slice(s)
    assert(StringSlice_len(slice) == 11)
    // s still valid
}

fn test_subslice() {
    let s = String_new("hello world", 11).unwrap()
    let slice = String_slice(s)
    let sub = StringSlice_subslice(slice, 0, 5).unwrap()
    assert(StringSlice_len(sub) == 5)
}
```

#### Week 6-7: Path Binding (`hold`)

**Path Binding Implementation:**

```rust
// ast/stmt.rs

pub enum Stmt {
    // ... existing variants

    // New: Path binding
    Hold {
        path: Path,
        binding: Name,
        body: Box<Stmt>,
    },
}

pub enum Path {
    // Root access
    Root(Name),

    // Field access
    Field {
        base: Box<Path>,
        field: Name,
    },

    // Index access
    Index {
        base: Box<Path>,
        index: Box<Expr>,
    },
}
```

**Parser Changes:**

```rust
// parser.rs - Add hold parsing

fn parse_stmt(&mut self) -> Result<Stmt, Error> {
    match self.cur.kind {
        TokenKind::Hold => {
            self.advance();
            self.expect(TokenKind::Path)?;
            let path = self.parse_path()?;
            self.expect(TokenKind::As)?;
            let binding = self.parse_ident()?;
            let body = self.parse_block()?;
            Ok(Stmt::Hold { path, binding, body })
        }
        _ => { /* ... */ }
    }
}
```

**Transpilation:**

```rust
// trans/c.rs - Path binding

fn transpile_stmt(&mut self, stmt: &Stmt, indent: usize) -> String {
    match stmt {
        Stmt::Hold { path, binding, body } => {
            let mut output = String::new();

            // Generate temporary variables for path
            let path_code = self.transpile_path(path);

            // Lock path (compile-time, no runtime cost)
            output.push_str(&format!("{{ /* structural lock */\n"));
            output.push_str(&format!("    auto {} = {};\n", binding, path_code));
            output.push_str(&self.transpile_stmt(body, indent + 1));
            output.push_str(&format!("}} /* unlock */\n"));

            Ok(output)
        }
        _ => { /* ... */ }
    }
}
```

**Usage:**

```auto
// tests/hold/test_hold.at

type Point {
    x int
    y int
}

fn test_hold_path() {
    let mut p = Point{x: 10, y: 20}

    hold path p.x as value {
        // value is mutable view of p.x
        // Compiler verifies exclusive access
    }
}
```

#### Week 8: Integration & Testing

**Success Criteria:**
- [ ] Full borrow checker working
- [ ] `take`/`edit` keywords functional
- [ ] `StringSlice` with compile-time lifetimes
- [ ] `hold` path binding operational
- [ ] 200+ tests passing
- [ ] Zero runtime overhead for borrows

---

## 4. Success Criteria

### Phase 1: Move Semantics (6 weeks)
- [ ] Linear type system implemented
- [ ] "Last use" detection working
- [ ] Move semantics enforced
- [ ] Automatic cleanup on scope exit
- [ ] Stack unwinding for early returns
- [ ] 50+ tests passing

### Phase 2: Owned Strings (6 weeks)
- [ ] `String` type functional
- [ ] Move semantics working
- [ ] `CString` for C FFI
- [ ] UTF-8 validation
- [ ] 100+ tests passing
- [ ] Zero memory leaks

### Phase 3: Borrow Checker (8 weeks)
- [ ] Borrow checker implemented
- [ ] `take`/`edit` keywords working
- [ ] `StringSlice` with compile-time lifetimes
- [ ] `hold` path binding
- [ ] Zero runtime overhead
- [ ] 200+ tests passing

### Overall
- [ ] Matches [new_memory.md](../language/design/new_memory.md) vision
- [ ] Zero-cost safety achieved
- [ ] Ready for Plan 024 (Strings) to use this system
- [ ] Foundation for collections, async, etc.

---

## 5. Risks & Mitigations

### Risk 1: Complexity Underestimated
**Risk:** Move semantics harder than expected

**Mitigation:**
- Start with minimal implementation (Phase 1)
- Add features incrementally
- Can ship without borrowing initially

### Risk 2: Borrow Checker Difficulties
**Risk:** Borrow checker complex to implement

**Mitigation:**
- Reference Rust's borrow checker implementation
- Start with simple cases
- Can delay `hold` path binding if needed

### Risk 3: Delay to Working Code
**Risk:** Longer time to usable strings

**Mitigation:**
- Phase 2 delivers owned strings in 3 months
- Good enough for most use cases initially
- Borrowing (Phase 3) is enhancement

### Risk 4: Performance Regression
**Risk:** Ownership checks slow down code

**Mitigation:**
- All checks at compile-time
- Zero runtime overhead (by design)
- Benchmark against current implementation

---

## 6. Timeline Summary

| Phase | Duration | Complexity | Deliverable |
|-------|----------|------------|-------------|
| 1. Move Semantics | 6 weeks | Very High | Linear types, last use detection |
| 2. Owned Strings | 6 weeks | High | `String` with moves, `CString` |
| 3. Borrow Checker | 8 weeks | Very High | `take`/`edit`, `StringSlice`, `hold` |

**Total: 20 weeks (5 months)**

**Critical Path:** Phase 1 → 2 → 3 (sequential)

**Time to Usable Strings:** End of Phase 2 (3 months)

---

## 7. Why This Approach

### Comparison with Alternatives

| Approach | Time to Strings | Total Time | Rework | Safety |
|----------|-----------------|------------|--------|--------|
| A. Memory First (Full) | 6 months | 6 months | None | Compile-time |
| B. Strings First (Plan 024) | 2.5 months | 4+ months | 60-70% | Runtime |
| **C. Hybrid** | **3 months** | **5 months** | **<5%** | **Compile-time** |

### Key Benefits

1. **Minimal Rework:** Each phase builds on previous
2. **Incremental Value:** Working strings in 3 months
3. **Future-Proof:** Final system matches long-term vision
4. **Lower Risk:** Each phase independently valuable
5. **Zero-Cost:** Compile-time safety (no runtime overhead)

---

## 8. Next Steps

### Immediate Actions (Week 1-2)

1. **Set up ownership module structure**
   - Create `crates/auto-lang/src/ownership/` directory
   - Create ownership/mod.rs
   - Set up test infrastructure

2. **Implement Phase 1: Move Semantics**
   - Define `Linear` trait
   - Implement move tracker
   - Add "last use" detection
   - Update evaluator

3. **Prepare for Phase 2**
   - Design owned `String` type
   - Plan C FFI integration
   - Define UTF-8 validation strategy

### First Quarter Goals

- Complete Phase 1 (Move Semantics)
- Start Phase 2 (Owned Strings)
- Have working move-only strings

### First Year Goals

- Complete all three phases
- Full ownership system operational
- Ready for collections, concurrency, etc.
- Matches [new_memory.md](../language/design/new_memory.md) vision

---

## 9. Related Documentation

- **[new_memory.md](../language/design/new_memory.md)** - Memory system design vision
- **Plan 024**: String Type Redesign (BLOCKED - waits for this plan)
- **Plan 027**: Stdlib C Foundation (BLOCKED - waits for Plan 024)
- **Plan 026**: Self-Hosting Compiler (BLOCKED - waits for Plans 024 & 027)

---

## 10. Conclusion

This plan provides the foundation for AutoLang's core innovation: ownership-based memory management without GC. By implementing it in three phases, we minimize rework while delivering value incrementally.

**The hybrid approach gives us:**
1. **Working strings in 3 months** (Phase 2 complete)
2. **Full ownership system in 5 months** (all phases)
3. **Zero rework** (each phase builds on previous)
4. **Compile-time safety** (no runtime overhead)
5. **Alignment with vision** (matches `new_memory.md`)

This is the critical foundation that Plan 024 (Strings) and all subsequent plans depend on. Once complete, AutoLang will have Rust-level memory safety with GC-level ergonomics.
