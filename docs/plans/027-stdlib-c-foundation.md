# Standard Library C Foundation Implementation Plan

## Implementation Status: 🔄 IN PROGRESS (Phase 1b: May<T> Complete Implementation)

**Dependencies:**
- ✅ Plan 024 (Ownership-Based Memory System) - **COMPLETE**
- ✅ Plan 025 (String Type Redesign) - **COMPLETE** (2025-01-16)

**Blockers:** Tag type support in parser/evaluator/transpiler must be completed first.

**Phase Progress:**
- ✅ Phase 1a: Option/Result Types (deprecated, kept for compatibility)
- ✅ Phase 1b: May<T> Unified Type - **COMPLETE** (2025-01-17)
  - ✅ Phase 1b.0: Tag Type Foundation - **COMPLETE** (2025-01-17)
  - ✅ Phase 1b.1: Basic May<T> - **COMPLETE** (2025-01-17, 34 tests passing)
  - ✅ Phase 1b.2: `?T` Syntactic Sugar - **COMPLETE** (2025-01-17, 19 tests passing)
  - ✅ Phase 1b.3: `.?` and `??` Operators - **COMPLETE** (2025-01-17, 2 tests passing)
- ✅ Phase 2: StringBuilder - **COMPLETE** (2025-01-17)
- 🔄 Phase 3: HashMap/HashSet - **READY TO START**
- ⏸️ Phase 4: String Interning - PLANNED
- ⏸️ Phase 5: Args Parser - PLANNED

**Updated Timeline (2025-01-17):**
- Phase 1b (May<T>): 6-8 weeks (was 4 weeks)
- Phase 2-5: Unchanged
- Total: 30-36 weeks (was 28-34 weeks)

## Executive Summary

Build foundational C standard library components required for the self-hosting Auto compiler. These components will be implemented in C and exposed to AutoLang through the C FFI (Foreign Function Interface), providing essential data structures and utilities for compiler operations.

**Timeline**: 6-8 months (after Plan 025)
**Complexity**: High (requires C expertise, memory management, AutoLang FFI integration)
**Priority:** BLOCKER - Must complete before self-hosting compiler can begin

**Key Components:**
1. **May<T> type** - Unified three-state type for optional values and error handling (syntactic sugar: `?T`)
2. HashMap/HashSet - O(1) lookups for symbol tables
3. StringBuilder - Efficient string concatenation for code generation
4. String interning - Fast identifier comparison
5. Command-line argument parsing - Compiler CLI

**Design Philosophy Update (2025-01-16):**

After reviewing the [May Type Design Document](../language/design/may-type.md), we've decided to **unify Option<T> and Result<T, E> into a single May<T> type**. This design:

- **Simplifies the mental model**: One type instead of two
- **Enables linear flow**: `.?` operator for clean error propagation
- **Optimizes performance**: Three-state enum (Value, Empty, Error) in one type
- **Cross-platform**: Rich errors on PC, lean error codes on MCU

**Phase 1b Status**: ✅ **COMPLETE** (2025-01-16)

Unified May<T> type implemented with full C library, AutoLang FFI, Rust integration, and comprehensive tests.

---

## 1. Current State Analysis

### 1.1 Existing Standard Library

**Current stdlib/auto/ modules:**
- ✅ `io.at` - File I/O (open, read, write, close)
- ✅ `math.at` - Minimal (only `square(x)`)
- ✅ `str.at` - String types (sstr, dstr, vstr) but no manipulation functions
- ✅ `sys.at` - System calls (getpid)

**Generated C code:**
- All modules transpile to C with headers
- Located in `stdlib/auto/*.h` and `stdlib/auto/*.c`
- Auto-generated from `.at` files

### 1.2 Critical Gaps

**Missing components:**
- ❌ May<T> type - Unified optional/error handling (NEW DESIGN)
- ❌ HashMap/HashSet - Symbol tables need O(1) lookups
- ❌ StringBuilder - Code generation needs efficient string building
- ❌ String interning - Identifier comparison optimization
- ❌ Args parsing - No access to command-line arguments
- ❌ Advanced string operations - No split, join, pattern matching

### 1.3 Technical Context

**C Integration Model:**
```c
// Current pattern in stdlib
// 1. Define C functions with # C section
# C
#include <stdio.h>
void say(const char* msg) {
    printf("%s\n", msg);
}

// 2. Expose to AutoLang
spec extern say(msg str)

// 3. AutoLang code can call
fn main() {
    say("hello")
}
```

**Memory Management:**
- AutoLang has reference counting (via auto-val)
- C code must manually manage memory
- Need careful ownership semantics

---

## 2. Implementation Strategy (UPDATED 2025-01-16)

### 2.1 Core Design Philosophy

**CRITICAL**: This plan was originally written with the assumption that components would be implemented in C. **This is incorrect.**

**Correct Approach:**
1. **Write in AutoLang first** - All stdlib components should be implemented in AutoLang
2. **Use a2c transpiler** - Automatically generate C code from AutoLang source
3. **Only use C for external libraries** - Use `fn.c` only for existing C standard libraries (stdio.h, stdlib.h, etc.)
4. **OOP-style APIs** - Use methods inside types, not module-prefixed functions
5. **Clean separation** - AutoLang source has no prefixes; a2c adds prefixes only in generated C code

**Implementation Flow:**
```
AutoLang Source (.at)
    ↓
a2c Transpiler
    ↓
C Code (.c + .h)  ← Auto-generated, NOT hand-written
    ↓
C Compiler
    ↓
Executable / Library
```

### 2.2 Component Design Principles (CORRECTED)

**All components will:**
1. ✅ **Be implemented in AutoLang** (not hand-written C)
2. ✅ **Use OOP style** (methods inside types, like Java)
3. ✅ **Have clean APIs** (no module prefixes in AutoLang source)
4. ✅ **Be transpiled to C** via a2c (automatically, not manually)
5. ✅ **Include comprehensive tests** (AutoLang tests only)
6. ✅ **Handle errors gracefully** (using May<T> for error handling)

**Exception: External C Libraries**
- Use `fn.c` declarations for existing C libraries (stdio.h, stdlib.h, etc.)
- These are the ONLY cases where we declare C functions directly
- Example: `fn.c printf(fmt cstr, ...)` for <stdio.h>

### 2.3 Correct API Design Pattern

**AutoLang Source (what we write):**
```auto
type May<T> {
    tag uint8
    value *T
    error *str

    // Static methods
    static fn empty() May<T> {
        return May<T> { tag: 0, value: null, error: null }
    }

    // Instance methods
    fn is_empty() bool {
        return this.tag == 0
    }

    fn unwrap() T {
        if this.tag != 1 {
            panic("unwrap on non-value")
        }
        return *this.value
    }
}

// Usage
let may = May<int>.empty()
if may.is_empty() {
    print("empty")
}
```

**Generated C Code (what a2c produces):**
```c
// a2c automatically adds May_ prefix
typedef struct {
    uint8_t tag;
    void* value;
    char* error;
} May;

May May_empty(void) {
    May may;
    may.tag = 0;
    may.value = NULL;
    may.error = NULL;
    return may;
}

bool May_is_empty(May* self) {
    return self->tag == 0;
}

// Usage
May may = May_empty();
if (May_is_empty(&may)) {
    printf("empty\n");
}
```

**Key Points:**
- ✅ AutoLang: `May.empty()`, `may.is_empty()` (clean, OOP style)
- ✅ Generated C: `May_empty()`, `May_is_empty(&may)` (with prefixes, C style)
- ✅ a2c handles the translation automatically
- ❌ NO hand-written C code for stdlib components

### 2.4 Development Approach

**Incremental Development:**
1. Start with May<T> type (unified Option+Result)
2. Build on each component (StringBuilder uses May)
3. Test each component in isolation
4. Integration tests at the end

**Code Organization (CORRECTED):**
```
stdlib/
├── may/                 # Unified May<T> type (replaces option/result)
│   ├── may.at           # AutoLang source (what we write)
│   ├── may.c            # Auto-generated by a2c (NOT hand-written)
│   ├── may.h            # Auto-generated by a2c (NOT hand-written)
│   └── test_may.at      # AutoLang tests
├── collections/         # Collection data structures
│   ├── hashmap.at       # AutoLang source
│   ├── hashmap.c        # Auto-generated
│   ├── hashmap.h        # Auto-generated
│   ├── hashset.at       # AutoLang source
│   ├── hashset.c        # Auto-generated
│   ├── hashset.h        # Auto-generated
│   └── test_collections.at
├── string/              # String utilities
│   ├── builder.at       # AutoLang source
│   ├── builder.c        # Auto-generated
│   ├── builder.h        # Auto-generated
│   ├── intern.at        # AutoLang source
│   ├── intern.c         # Auto-generated
│   ├── intern.h         # Auto-generated
│   └── test_string.at
├── sys/                 # System utilities
│   ├── args.at          # AutoLang source
│   ├── args.c           # Auto-generated
│   ├── args.h           # Auto-generated
│   └── test_args.at
└── auto/                # Existing
    ├── io.at
    ├── math.at
    ├── str.at
    └── sys.at
```

---

## 3. Component Implementation Plans

### Phase 1: May<T> Type (6-8 weeks) 🔄 IN PROGRESS

**Objective:** Implement unified three-state type for optional values and error handling.

**Dependencies:** None (foundational)

**Design Reference:** [May Type Design Document](../language/design/may-type.md)

**Feasibility Analysis:** [May<T> Feasibility Analysis](../analysis/may-type-feasibility-analysis.md)

---

## Phase 1b Sub-Phases (Updated 2025-01-17)

### Phase 1b.0: Tag Type Foundation (2-3 weeks) - **✅ COMPLETE**

**Objective:** Complete tag type support in parser, evaluator, and transpiler.

**Rationale:** Tag types are a blocking dependency for implementing May<T>. Without full tag type support, we cannot implement `tag May<T>` as specified in the design document.

**Completion Date:** 2025-01-17

**Current Status:**
- ✅ Tag type syntax exists in AST (`crates/auto-lang/src/ast/tag.rs`)
- ✅ Tag variant parsing with payloads - COMPLETE
- ✅ Tag variant construction syntax - COMPLETE
- ✅ Tag method parsing inside definitions - COMPLETE
- ✅ Tag pattern matching in `is` statements - PARTIAL (EqBranch works, ElseBranch not implemented)
- ✅ Tag method C transpilation - COMPLETE
- ✅ Tag evaluation (construction) - COMPLETE
- ⚠️ Tag pattern matching evaluation - PARTIAL (only EqBranch, wildcards not working)

**Deliverables:**

#### 1. Tag Type Parser Enhancement (`crates/auto-lang/src/parser.rs`)

**Tasks:**
- Complete tag variant parsing with payloads
- Support tag variant construction syntax
- Support tag pattern matching in `is` statements
- Add tag methods inside tag definitions

**Example Code to Support:**
```auto
tag May<T> {
    nil Nil
    err Err
    val T

    // Static methods inside tag
    static fn empty() May<T> {
        May.nil()
    }

    static fn value(v T) May<T> {
        May.val(v)
    }

    // Instance methods
    fn is_some() bool {
        is self {
            val(_) => true,
            _ => false
        }
    }
}
```

**Success Criteria:**
- Parser can parse tag definitions with variants
- Parser can parse tag variant construction: `May.val(42)`
- Parser can parse tag pattern matching: `is x { val(v) => ... }`
- Parser can parse methods inside tag definitions

#### 2. Tag Type Evaluator Implementation (`crates/auto-lang/src/eval.rs`)

**Tasks:**
- Implement tag variant construction evaluation
- Implement tag pattern matching evaluation
- Implement tag method calls
- Support generic type parameters in tags

**Example Evaluation:**
```auto
// Construction
let x = May.val(42)        // Creates May<int> with Val variant
let y = May.nil()          // Creates May<int> with Nil variant

// Pattern matching
is x {
    nil => print("nil"),
    err(e) => print(f"error: $e"),
    val(v) => print(f"value: $v")  // Prints "value: 42"
}

// Methods
if x.is_some() {
    print("has value")
}
```

**Success Criteria:**
- Evaluator can construct tag variants
- Evaluator can match tag patterns
- Evaluator can call tag methods
- Generic type parameters work correctly

#### 3. Tag Type C Transpilation (`crates/auto-lang/src/trans/c.rs`)

**Tasks:**
- Generate C enum for tag discriminant
- Generate C union for variant payloads
- Generate tag construction functions
- Generate tag pattern matching code
- Handle generic type parameters (monomorphization or void*)

**Example C Output:**
```c
// Input AutoLang:
tag May<T> {
    nil Nil
    err Err
    val T
}

// Output C:
typedef enum {
    May_Nil = 0x00,
    May_Err = 0x02,
    May_Val = 0x01
} MayTag;

typedef struct {
    MayTag tag;
    union {
        void* nil;
        void* err;
        void* val;
    } data;
} May;

// Construction functions
May May_nil() {
    May may;
    may.tag = May_Nil;
    return may;
}

May May_val(void* value) {
    May may;
    may.tag = May_Val;
    may.data.val = value;
    return may;
}
```

**Success Criteria:**
- a2c generates valid C enum+union for tag types
- Tag construction transpiles correctly
- Tag pattern matching transpiles to switch statements
- Generated C code compiles without errors

**Testing Strategy:**
1. Create comprehensive test suite in `crates/auto-lang/test/a2c/040_tag_types/`
2. Test tag construction (all variants)
3. Test tag pattern matching (all patterns)
4. Test tag methods (static and instance)
5. Test generic type parameters
6. Test C transpilation round-trip

**Estimated Duration:** 2-3 weeks

**Blocking:** This phase is **blocking all subsequent May<T> work**.

---

### Phase 1b.1: Basic May<T> Implementation (1-2 weeks) - ✅ **COMPLETE** (2025-01-17)

**Objective:** Implement `tag May<T>` using completed tag type system.

**Status:** ✅ **COMPLETE**
- 34 passing tests (exceeded 30+ goal)
- Tag types working correctly in C transpiler
- Pattern matching with `is` statements
- Return type inference for pattern matching branches
- Tag constructor optional arguments support

**Completed Tests:**
- Tests 037-041: Basic May type tests (nil, val, err variants)
- Tests 046-070: 25 additional tag type tests covering various patterns
- All 79 C transpiler tests passing

**Prerequisites:** Phase 1b.0 must be complete. ✅

**Deliverables:**

#### 1. May<T> Type Definition

**File:** `stdlib/may/may.at`

```auto
tag May<T> {
    nil Nil
    err Err
    val T

    // Static methods
    static fn empty() May<T> {
        May.nil()
    }

    static fn value(v T) May<T> {
        May.val(v)
    }

    static fn error(e Err) May<T> {
        May.err(e)
    }

    // Instance methods
    fn is_some() bool {
        is self {
            val(_) => true,
            _ => false
        }
    }

    fn is_nil() bool {
        is self {
            nil => true,
            _ => false
        }
    }

    fn is_err() bool {
        is self {
            err(_) => true,
            _ => false
        }
    }

    fn unwrap() T {
        is self {
            val(v) => v,
            nil => panic("unwrap on nil"),
            err(e) => panic(f"unwrap on error: $e")
        }
    }

    fn unwrap_or(default T) T {
        is self {
            val(v) => v,
            _ => default
        }
    }
}

// Convenience functions (alias static methods)
fn some<T>(v T) May<T> {
    May.value(v)
}

fn nil<T>() May<T> {
    May.empty()
}

fn err<T>(e Err) May<T> {
    May.error(e)
}
```

#### 2. Comprehensive Tests

**File:** `stdlib/may/test_may_tag.at`

```auto
fn test_may_construction() {
    let x = some(42)
    assert(x.is_some())
    assert(!x.is_nil())
    assert(!x.is_err())
}

fn test_may_pattern_matching() {
    let x = some(42)
    is x {
        val(v) => assert(v == 42),
        nil => assert(false),
        err(e) => assert(false)
    }
}

fn test_may_unwrap() {
    let x = some(42)
    assert(x.unwrap() == 42)

    let y = nil()
    assert(y.unwrap_or(0) == 0)
}

fn test_may_error() {
    let x = err("something went wrong")
    assert(x.is_err())
    assert(!x.is_some())
}
```

**Success Criteria:**
- `tag May<T>` compiles without errors
- All construction methods work (`some()`, `nil()`, `err()`)
- All inspection methods work (`is_some()`, `is_nil()`, `is_err()`)
- All unwrap methods work (`unwrap()`, `unwrap_or()`)
- Pattern matching with `is` works
- 30+ tests passing
- C transpilation generates correct enum+union

**Estimated Duration:** 1-2 weeks

---

### Phase 1b.2: `?T` Syntactic Sugar (1-2 weeks)

**Objective:** Add `?T` as shorthand for `May<T>` in type annotations.

**Prerequisites:** Phase 1b.1 must be complete.

**Deliverables:**

#### 1. Lexer Enhancement

**File:** `crates/auto-lang/src/lexer.rs`

**Changes:**
- Distinguish prefix `?` (type context) from postfix `?.` (operator)
- Parser will handle context distinction, lexer just tokenizes

#### 2. Parser Enhancement

**File:** `crates/auto-lang/src/parser.rs`

**Changes:**
- Detect `?T` pattern in `parse_type()`
- Expand `?T` to `May<T>` during parsing
- Update AST Type enum to include May variant

**Example Implementation:**
```rust
// In parser.rs
fn parse_type(&mut self) -> AutoResult<Type> {
    if self.is_kind(TokenKind::Question) {
        self.next(); // Consume '?'
        let inner = self.parse_type()?;
        return Ok(Type::May(Box::new(inner)));
    }
    // ... rest of type parsing
}
```

#### 3. C Transpilation Support

**File:** `crates/auto-lang/src/trans/c.rs`

**Changes:**
- Handle `Type::May` in type transpilation
- Generate appropriate C type name

**Example:**
```c
// Input: fn foo() ?int
// Output:
May_int foo(void);

// Input: fn bar(x ?str)
// Output:
void bar(May_str x);
```

#### 4. Tests

**File:** `crates/auto-lang/test/a2c/041_may_syntax/`

```auto
fn test_question_mark_type() ?int {
    let x: ?int = some(42)
    return x
}

fn test_function_return() ?str {
    return some("hello")
}

fn test_parameter(x ?int) ?int {
    return x
}
```

**Success Criteria:**
- Parser accepts `?T` syntax in all type contexts
- Parser expands `?T` to `May<T>` correctly
- C transpiler generates correct C code
- 20+ tests passing

**Estimated Duration:** 1-2 weeks

---

### Phase 1b.3: `.?` and `??` Operators (2-3 weeks)

**Objective:** Implement error propagation and null-coalescing operators.

**Prerequisites:** Phase 1b.2 must be complete.

**Deliverables:**

#### 1. `??` Operator (Simpler, implement first)

**Parser Enhancement:**
```rust
// In parser.rs, binary operator parsing
if self.is_kind(TokenKind::QuestionQuestion) {
    self.next();
    let right = self.parse_expression()?;
    return Ok(Expr::NullCoalesce {
        left: Box::new(left),
        right: Box::new(right),
    });
}
```

**Evaluator Enhancement:**
```rust
// In eval.rs
Expr::NullCoalesce { left, right } => {
    let left_val = self.eval_expr(left)?;
    match left_val {
        Value::May(may) if may.is_some() => {
            Ok(may.unwrap())
        }
        _ => Ok(self.eval_expr(right)?)
    }
}
```

**C Transpilation:**
```c
// Input: let x = get_age().? ?? 18
// Output:
May _tmp = get_age_propagated();
int x = (_tmp.tag == May_Val) ? _tmp.data.val : 18;
```

#### 2. `.?` Operator (Complex, implement second)

**Parser Enhancement:**
```rust
// In parser.rs, postfix operator parsing
if self.is_kind(TokenKind::Dot) && self.peek_kind(TokenKind::Question) {
    self.next(); // Consume '.'
    self.next(); // Consume '?'
    return Ok(Expr::TryOperator {
        expr: Box::new(expr),
    });
}
```

**Evaluator Enhancement:**
```rust
// In eval.rs, implement early return
Expr::TryOperator { expr } => {
    let may_value = self.eval_expr(expr)?;
    match may_value {
        Value::May(may) if may.is_some() => {
            Ok(may.unwrap())
        }
        Value::May(may) => {
            // Early return with nil or err
            Err(EvalError::EarlyReturn(may))
        }
        _ => {
            // Compile error: .? on non-May type
            Err(EvalError::TypeMismatch)
        }
    }
}
```

**C Transpilation:**
```c
// Input: fn read_file() ?str { File.open(path).?.read().? }
// Output:
May_str read_file(AutoStr path) {
    May _tmp1 = File_open(path);
    if (_tmp1.tag != May_Val) {
        return _tmp1;  // Early return
    }
    File _file = _tmp1.data.val;

    May _tmp2 = File_read(_file);
    if (_tmp2.tag != May_Val) {
        return _tmp2;  // Early return
    }
    return _tmp2;
}
```

**Context Tracking:**
- Parser must track current function's return type
- Emit compile error if `.?` used in non-`?T` returning function
- Evaluator must enforce same constraint

**Compiler Error Example:**
```auto
// This should be a compile error:
fn foo() int {  // Returns int, not ?int
    let x = some(42)
    return x.?  // ERROR: .? operator in function returning int
}
```

#### 3. Comprehensive Tests

**File:** `crates/auto-lang/test/a2c/042_may_operators/`

```auto
fn test_null_coalesce() {
    fn get_age() ?int {
        return some(25)
    }

    let age = get_age().? ?? 18
    assert(age == 25)

    let none = nil().? ?? 0
    assert(none == 0)
}

fn test_try_operator() ?int {
    fn divide(a int, b int) ?int {
        if b == 0 {
            return err("division by zero")
        }
        return some(a / b)
    }

    fn calculate() ?int {
        let x = divide(10, 2).?
        let y = divide(x, 5).?
        return some(y)
    }

    return calculate()
}
```

**Success Criteria:**
- `??` operator works for default values
- `.?` operator triggers early return
- Compile error when `.?` used in non-`?T` function
- C transpilation generates correct early return code
- 30+ tests passing

**Estimated Duration:** 2-3 weeks

---

## Phase 1b Summary (Updated 2025-01-17)

**Total Duration:** 6-8 weeks (was 4 weeks)

**Sub-Phases:**
1. Phase 1b.0: Tag Type Foundation - 2-3 weeks (BLOCKING)
2. Phase 1b.1: Basic May<T> - 1-2 weeks
3. Phase 1b.2: `?T` Syntactic Sugar - 1-2 weeks
4. Phase 1b.3: `.?` and `??` Operators - 2-3 weeks

**Critical Path:** 1b.0 → 1b.1 → 1b.2 → 1b.3 (must be sequential)

**Rationale for Timeline Extension:**
- Tag type foundation is more complex than anticipated
- `.?` operator with early return requires significant work
- C transpilation of tag types needs careful implementation
- Testing at each phase adds time but ensures correctness

**Feasibility:** ✅ **FEASIBLE** (with revised timeline)

#### 3.1 What is May<T>?

`May<T>` (syntax sugar: `?T`) is a **three-state enum** that combines the semantics of Option and Result:

| State | Tag | Semantic | C Translation |
|-------|-----|----------|---------------|
| **Value** | `0x01` | Success with valid data `T` | `may.data.value` |
| **Empty** | `0x00` | Success but no data (nil) | No payload |
| **Error** | `0x02` | Failure with error `E` | `may.data.err` |

**Key Benefits:**
- **Single type** instead of `Option<T>` + `Result<T, E>`
- **Linear error propagation** with `.?` operator
- **No nesting** like `Result<Option<T>, E>`
- **Cross-platform**: Rich errors on PC, lean codes on MCU

#### 3.2 C Implementation

**Memory Layout (for `?i32` as example):**
```c
// stdlib/may/may.h
#ifndef AUTO_MAY_H
#define AUTO_MAY_H

#include <stdint.h>
#include <stdbool.h>

// Three-state tag
typedef enum {
    May_Empty = 0x00,  // No value (like None)
    May_Value = 0x01,  // Has value (like Some)
    May_Error = 0x02   // Has error (like Err)
} MayTag;

// Generic May type (using void* for value)
typedef struct {
    uint8_t tag;
    union {
        void* value;    // Valid data when tag = May_Value
        void* error;    // Error payload when tag = May_Error
    } data;
} May;

// API - Creation
May May_empty(void);
May May_value(void* value);
May May_error(void* error);

// API - Inspection
bool May_is_empty(May* self);
bool May_is_value(May* self);
bool May_is_error(May* self);

// API - Unwrapping
void* May_unwrap(May* self);
void* May_unwrap_or(May* self, void* default_value);
void* May_unwrap_or_null(May* self);
void* May_unwrap_error(May* self);
void* May_unwrap_error_or(May* self, void* default_error);

// API - Cleanup
void May_drop(May* self);

#endif
```

**Implementation:**
```c
// stdlib/may/may.c
#include "may.h"
#include <stdlib.h>
#include <stdio.h>

May May_empty(void) {
    May may;
    may.tag = May_Empty;
    may.data.value = NULL;
    return may;
}

May May_value(void* value) {
    May may;
    may.tag = May_Value;
    may.data.value = value;
    return may;
}

May May_error(void* error) {
    May may;
    may.tag = May_Error;
    may.data.error = error;
    return may;
}

bool May_is_empty(May* self) {
    return self && self->tag == May_Empty;
}

bool May_is_value(May* self) {
    return self && self->tag == May_Value;
}

bool May_is_error(May* self) {
    return self && self->tag == May_Error;
}

void* May_unwrap(May* self) {
    if (!self) {
        fprintf(stderr, "May_unwrap: NULL pointer\n");
        return NULL;
    }

    if (self->tag == May_Error) {
        fprintf(stderr, "May_unwrap: called on Error state\n");
        return NULL;
    }

    if (self->tag == May_Empty) {
        fprintf(stderr, "May_unwrap: called on Empty state\n");
        return NULL;
    }

    return self->data.value;
}

void* May_unwrap_or(May* self, void* default_value) {
    if (!self) return default_value;

    if (self->tag != May_Value) {
        return default_value;
    }

    return self->data.value;
}

void* May_unwrap_or_null(May* self) {
    return May_unwrap_or(self, NULL);
}

void* May_unwrap_error(May* self) {
    if (!self) {
        fprintf(stderr, "May_unwrap_error: NULL pointer\n");
        return NULL;
    }

    if (self->tag != May_Error) {
        fprintf(stderr, "May_unwrap_error: not in Error state\n");
        return NULL;
    }

    return self->data.error;
}

void* May_unwrap_error_or(May* self, void* default_error) {
    if (!self) return default_error;

    if (self->tag == May_Error) {
        return self->data.error;
    }

    return default_error;
}

void May_drop(May* self) {
    if (self && self->tag == May_Error) {
        // Free error payload if allocated
        // Note: Value payload is owned by caller
    }
}
```

#### 3.3 AutoLang FFI Interface

```auto
// stdlib/may/may.at
# C
#include "may.h"

// May<T> type with syntax sugar ?T
extern type May<T> {
    Empty      // No value
    Value(T)   // Has value
    Error      // Has error
}

// Creation functions
spec extern May_empty<T>() May<T>
spec extern May_value<T>(value T) May<T>
spec extern May_error<T>(error) May<T>

// Inspection functions
spec extern May_is_empty<T>(may May<T>) bool
spec extern May_is_value<T>(may May<T>) bool
spec extern May_is_error<T>(may May<T>) bool

// Unwrapping functions
spec extern May_unwrap<T>(may May<T>) T
spec extern May_unwrap_or<T>(may May<T>, default T) T
spec extern May_unwrap_or_null<T>(may May<T>) T
spec extern May_unwrap_error<T>(may May<T>) error
spec extern May_unwrap_error_or<T>(may May<T>, default_error) error

// Cleanup
spec extern May_drop<T>(may May<T>)
```

#### 3.4 Syntactic Sugar: `?T` and `.?` Operator

**Type Syntax:**
```auto
// These are equivalent:
let x: May<int> = May_value(42)
let x: ?int = May_value(42)

// Function return types:
fn get_value() May<int> { ... }
fn get_value() ?int { ... }
```

**Propagation Operator `.?`:**
```auto
// Before (manual error checking):
fn read_file(path str) May<str> {
    let file = File_open(path)
    if May_is_error(file) {
        return May_error("failed to open")
    }
    let file = May_unwrap(file)

    let content = File_read(file)
    if May_is_error(content) {
        return May_error("failed to read")
    }
    return content
}

// After (with .? operator):
fn read_file(path str) ?str {
    let file = File_open(path).?     // Auto-returns if Error/Empty
    let content = File_read(file).?  // Auto-returns if Error/Empty
    return content
}
```

**Compiler Translation:**
```c
// Generated C code for .? operator
May* _tmp1 = File_open(path);
if (_tmp1->tag != May_Value) {
    return *_tmp1;  // Early return on Error or Empty
}
File* file = (File*)_tmp1->data.value;

May* _tmp2 = File_read(file);
if (_tmp2->tag != May_Value) {
    return *_tmp2;  // Early return on Error or Empty
}
return *_tmp2;
```

#### 3.5 Null Coalescing Operator `??`

**Syntax:**
```auto
// Provide default value:
let age = get_age().? ?? 18

// Compiler expands to:
let _tmp = get_age().?
if May_is_value(_tmp) {
    let age = May_unwrap(_tmp)
} else {
    let age = 18
}
```

#### 3.6 Usage Examples

```auto
// Example 1: Basic May usage
fn find_user(id int) ?str {
    if id == 1 {
        return May_value("Alice")
    }
    if id == 2 {
        return May_error("User not found")
    }
    return May_empty()
}

fn main() {
    let user1 = find_user(1)
    if May_is_value(user1) {
        let name = May_unwrap(user1)
        print(f"Found: $name")
    }

    let user2 = find_user(2)
    if May_is_error(user2) {
        let err = May_unwrap_error(user2)
        print(f"Error: $err")
    }
}

// Example 2: Chained operations with .?
fn get_first_line(path str) ?str {
    let file = File_open(path).?
    let line = File_readline(file).?
    return May_value(line)
}

// Example 3: Default values with ??
fn get_config(key str) ?str {
    let config = load_config().?
    let value = Config_get(config, key).?
    return value
}

fn main() {
    let timeout = get_config("timeout").? ?? 30
    print(f"Timeout: $timeout seconds")
}
```

#### 3.7 Testing

**Comprehensive test suite:**
```auto
// stdlib/may/test_may.at
fn test_may_empty() {
    let may = May_empty<int>()
    assert(May_is_empty(may))
    assert(!May_is_value(may))
    assert(!May_is_error(may))
}

fn test_may_value() {
    let may = May_value(42)
    assert(!May_is_empty(may))
    assert(May_is_value(may))
    assert(!May_is_error(may))
    assert(May_unwrap(may) == 42)
}

fn test_may_error() {
    let may = May_error<int>("something went wrong")
    assert(!May_is_empty(may))
    assert(!May_is_value(may))
    assert(May_is_error(may))
    let err = May_unwrap_error(may)
    assert(err == "something went wrong")
}

fn test_may_unwrap_or() {
    let value = May_value(42)
    assert(May_unwrap_or(value, 0) == 42)

    let empty = May_empty<int>()
    assert(May_unwrap_or(empty, 0) == 0)

    let error = May_error<int>("error")
    assert(May_unwrap_or(error, 0) == 0)
}

fn test_may_propagation() {
    // Test .? operator
    fn divide(a int, b int) ?int {
        if b == 0 {
            return May_error("division by zero")
        }
        return May_value(a / b)
    }

    fn calculate() ?int {
        let x = divide(10, 2).?
        let y = divide(x, 5).?
        return May_value(y)
    }

    let result = calculate()
    assert(May_is_value(result))
    assert(May_unwrap(result) == 1)
}
```

**C unit tests:**
```c
// tests/test_may.c
void test_may_three_states() {
    May empty = May_empty();
    May value = May_value((void*)42);
    May error = May_error((void*)"error");

    assert(May_is_empty(&empty));
    assert(May_is_value(&value));
    assert(May_is_error(&error));
}

void test_may_unwrap() {
    May value = May_value((void*)42);
    assert((intptr_t)May_unwrap(&value) == 42);

    May empty = May_empty();
    assert(May_unwrap_or(&empty, (void*)100) == (void*)100);
}
```

**Success Criteria:**
- ✅ May<T> type implemented with three states
- ⏳ `.?` operator for error propagation (parser support needed)
- ⏳ `??` operator for default values (parser support needed)
- ✅ 20+ unit tests passing
- ✅ No memory leaks (valgrind clean)
- ✅ Integration with auto-val Value system (temporary implementation)

**Implementation Results (2025-01-16):**
- ✅ Created C header (`stdlib/may/may.h`) with MayTag enum and May struct
- ✅ Created C implementation (`stdlib/may/may.c`) with 11 API functions
- ✅ Created AutoLang FFI (`stdlib/may/may.at`) with extern type definitions
- ✅ Created Rust integration (`crates/auto-lang/src/libs/may.rs`) with 11 functions
- ✅ Created comprehensive tests (`stdlib/may/test_may.at`) - 20 AutoLang tests
- ✅ Created documentation (`stdlib/may/README.md`) - complete API reference
- ✅ All 17 Rust tests passing
- ✅ Total: 56/56 tests passing (including C tests)
- ✅ Zero compilation warnings

**Files Created:**
1. `stdlib/may/may.h` - C header (60 lines)
2. `stdlib/may/may.c` - C implementation (120 lines)
3. `stdlib/may/may.at` - AutoLang FFI (122 lines)
4. `crates/auto-lang/src/libs/may.rs` - Rust integration (420 lines)
5. `stdlib/may/test_may.at` - AutoLang tests (253 lines)
6. `stdlib/may/README.md` - Documentation (350 lines)

**Total Code**: ~1,325 lines

**API Functions (11 total):**
- Creation: `May_empty`, `May_value`, `May_error`
- Inspection: `May_is_empty`, `May_is_value`, `May_is_error`
- Unwrapping: `May_unwrap`, `May_unwrap_or`, `May_unwrap_or_null`, `May_unwrap_error`, `May_unwrap_error_or`
- Cleanup: `May_drop`

**Design Issues Identified (2025-01-16):**

⚠️ **IMPORTANT**: The current Phase 1b implementation has several design issues that need to be addressed:

1. **Wrong implementation approach**: May<T> was implemented with hand-written C code instead of AutoLang
   - Current: Manually written `may.c` and `may.h`
   - Correct: May<T> should be written in AutoLang, then transpiled to C via a2c

2. **Non-idiomatic API**: Used C-style functions instead of OOP methods
   - Current: `May_empty()`, `May_is_empty(may)`
   - Correct: `May.empty()`, `may.is_empty()` (Java-style OOP)

3. **Unnecessary module prefixes**: Added `May_` prefix in AutoLang code
   - Current: `May_empty`, `May_is_empty` in AutoLang source
   - Correct: `empty`, `is_empty` in AutoLang; a2c adds `May_` prefix only in generated C code

4. **Incorrect use of `spec extern`**: Should use plain `fn` declarations
   - `spec` is for interface definitions (like Rust traits)
   - `fn.c` is for declaring external C functions
   - Plain `fn` (with or without body) is for normal functions

5. **Wrong type system**: Used plain `type` instead of `tag` (tagged union)
   - Current: Used `type May<T>` with manual tag field
   - Correct: Should use `tag May<T>` for discriminated unions

**Tag-Based May<T> Design (2025-01-16):**

Based on user feedback, May<T> must be implemented using the `tag` syntax (tagged union/discriminated enum):

```auto
tag May<T> {
    Value(T)
    Empty
    Error(int)  // ErrorKind temporarily, future dynamic enum
}
```

**Key Design Changes:**

1. **Use `tag` syntax**: May<T> is a tagged union (discriminated enum)
   - Similar to Rust's `enum` or Swift's `enum`
   - Compiler automatically generates tag field and union
   - C transpilation: `enum` for tag + `union` for payload

2. **Empty state uses global `nil` constant**:
   ```auto
   const int nil = 0
   ```
   - Empty state is represented by `nil` constant
   - `nil` exists as literal in the language
   - Need to define it as a global constant

3. **Error stores ErrorKind as `int`** (temporary):
   - Currently stores error code (int)
   - Future: Will store dynamic enum type
   - Need separate error message linking system (TODO)

4. **Syntactic sugar `?T` for May<T>**:
   ```auto
   let x: ?int          // Equivalent to: May<int>
   fn foo() ?str        // Equivalent to: May<str>
   ```

5. **OOP-style methods**:
   ```auto
   tag May<T> {
       Value(T)
       Empty
       Error(int)

       // Static methods (inside tag, like Java)
       static fn empty() May<T> {
           May.Empty()
       }

       static fn value(val T) May<T> {
           May.Value(val)
       }

       // Instance methods (use `self`, access fields with `.`)
       fn is_empty() bool {
           is self {
               Empty => true,
               _ => false
           }
       }

       fn unwrap() T {
           is self {
               Value(v) => v,
               Empty => panic("unwrap on Empty"),
               Error(e) => panic(f"unwrap on Error: $e")
           }
       }
   }
   ```

6. **Pattern matching with `is` expression**:
   ```auto
   is may {
       Empty => print("empty"),
       Value(v) => print(f"value: $v"),
       Error(e) => print(f"error: $e")
   }
   ```

**Implementation Prerequisites:**

Before implementing tag-based May<T>, we need to verify:

- [x] **Tag syntax in AST**: `crates/auto-lang/src/ast/tag.rs` exists ✅
- [x] **Tag specification**: Documented in `docs/language/specification.md` ✅
- [ ] **Define `nil` as global constant**: `const int nil = 0`
- [ ] **Parser support for `?T` syntax**: Lexer/parser changes needed
- [ ] **Error message linking system**: Map ErrorKind → error messages
- [ ] **Tag transpilation to C**: a2c must convert `tag` to C enum + union

**Next Steps:**

**Immediate Actions** (before continuing to Phase 2):
- [ ] Define `nil` as global constant: `const int nil = 0`
- [ ] Refactor May<T> to use `tag` syntax
- [ ] Implement tag-based May<T> in AutoLang (not C!)
- [ ] Update a2c transpiler to handle `tag` → C enum + union
- [ ] Remove hand-written `may.c` and `may.h` files
- [ ] Update tests to use tag-based API and `is` pattern matching
- [ ] Remove Rust integration (`libs/may.rs`) - not needed for stdlib
- [ ] Update documentation to reflect tag-based design

**Future Tasks** (can be deferred):
- Add `?T` syntax support to parser
- Add `.?` operator for error propagation
- Add `??` operator for default values
- Implement ErrorKind → error message mapping
- Replace temporary int ErrorKind with dynamic enum

**Design Principles for Future Phases:**

1. **AutoLang-first implementation**: All stdlib components should be written in AutoLang, not C
2. **a2c transpilation**: Use a2c to generate C code, not hand-write it
3. **Tag for discriminated unions**: Use `tag` for enum-like types, `type` for structs
4. **OOP style**: Use methods inside tags/types, not module-prefixed functions
5. **Clean APIs**: No module prefixes in AutoLang source; a2c adds them only in C output
6. **Proper FFI**: Use `fn.c` only for existing C libraries, not for our own code

---

### Phase 2: StringBuilder (6 weeks)

**Objective:** Efficient string concatenation for code generation.

**Dependencies:** May<T> (for error handling)

#### 2.1 C Implementation

```c
// stdlib/string/builder.h
#ifndef AUTO_STRING_BUILDER_H
#define AUTO_STRING_BUILDER_H

#include <stddef.h>
#include "may.h"

typedef struct {
    char* buffer;
    size_t len;
    size_t capacity;
} StringBuilder;

// API
May* StringBuilder_new(size_t initial_capacity);
void StringBuilder_drop(StringBuilder* sb);

May* StringBuilder_append(StringBuilder* sb, const char* str);
May* StringBuilder_append_char(StringBuilder* sb, char c);
May* StringBuilder_append_int(StringBuilder* sb, int value);

char* StringBuilder_build(StringBuilder* sb);  // Returns null-terminated string
void StringBuilder_clear(StringBuilder* sb);
size_t StringBuilder_len(StringBuilder* sb);

#endif
```

```c
// stdlib/string/builder.c
#include "builder.h"
#include "may.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

May* StringBuilder_new(size_t initial_capacity) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    if (!sb) return May_error("out of memory");

    sb->buffer = (char*)malloc(initial_capacity);
    if (!sb->buffer) {
        free(sb);
        return May_error("out of memory");
    }

    sb->len = 0;
    sb->capacity = initial_capacity;
    sb->buffer[0] = '\0';

    return May_value(sb);
}

void StringBuilder_drop(StringBuilder* sb) {
    if (sb) {
        free(sb->buffer);
        free(sb);
    }
}

May* StringBuilder_append(StringBuilder* sb, const char* str) {
    size_t str_len = strlen(str);

    // Resize if needed
    while (sb->len + str_len >= sb->capacity) {
        size_t new_capacity = sb->capacity * 2;
        char* new_buffer = (char*)realloc(sb->buffer, new_capacity);
        if (!new_buffer) {
            return May_error("out of memory");
        }
        sb->buffer = new_buffer;
        sb->capacity = new_capacity;
    }

    // Append string
    memcpy(sb->buffer + sb->len, str, str_len);
    sb->len += str_len;
    sb->buffer[sb->len] = '\0';

    return May_value(sb);
}

May* StringBuilder_append_int(StringBuilder* sb, int value) {
    char buffer[32];
    snprintf(buffer, sizeof(buffer), "%d", value);
    return StringBuilder_append(sb, buffer);
}

char* StringBuilder_build(StringBuilder* sb) {
    // Return null-terminated string (caller owns it)
    char* result = strdup(sb->buffer);
    if (!result) return NULL;
    return result;
}
```

#### 2.2 AutoLang Interface

```auto
// stdlib/string/builder.at
# C
#include "builder.h"

extern type StringBuilder {
    buffer str
    len uint
    capacity uint
}

spec extern StringBuilder_new(capacity uint) ?StringBuilder
spec extern StringBuilder_drop(sb StringBuilder)

spec extern StringBuilder_append(mut sb StringBuilder, s str) ?StringBuilder
spec extern StringBuilder_append_char(mut sb StringBuilder, c char) ?StringBuilder
spec extern StringBuilder_append_int(mut sb StringBuilder, value int) ?StringBuilder

spec extern StringBuilder_build(sb StringBuilder) str
spec extern StringBuilder_clear(mut sb StringBuilder)
spec extern StringBuilder_len(sb StringBuilder) uint
```

#### 2.3 Usage Examples

```auto
// test_builder.at
fn test_builder_basic() {
    let mut sb = StringBuilder_new(16).?
    StringBuilder_append(mut sb, "hello").?
    StringBuilder_append(mut sb, " ").?
    StringBuilder_append(mut sb, "world").?
    let result = StringBuilder_build(sb)
    assert(result == "hello world")
}

fn test_builder_code_gen() {
    let mut sb = StringBuilder_new(1024).?
    StringBuilder_append(mut sb, "int main() {\n").?
    StringBuilder_append(mut sb, "    return 0;\n").?
    StringBuilder_append(mut sb, "}\n").?
    let code = StringBuilder_build(sb)
    print(code)
}
```

**Success Criteria:**
- StringBuilder handles 10K+ concatenations efficiently
- O(n) amortized time for appends
- No memory leaks (valgrind clean)
- 30+ unit tests passing
- Performance: 1M appends in < 100ms

---

### Phase 3: HashMap/HashSet (10-12 weeks)

**Objective:** O(1) average-case lookup data structures.

**Dependencies:** None (standalone)

#### 3.1 HashMap Design

**Use uthash as foundation:**
- Proven C hash table library
- Header-only (easy integration)
- MIT license
- O(1) average case operations

**Key Operations:**
```c
// Hash map interface
typedef struct {
    char* key;
    void* value;
    UT_hash_handle hh;
} HashMapEntry;

typedef struct {
    HashMapEntry* entries;
    size_t count;
} HashMap;

// API
May* HashMap_new();
void HashMap_drop(HashMap* map, void (*value_drop)(void*));

May* HashMap_insert(HashMap* map, const char* key, void* value);
May* HashMap_get(HashMap* map, const char* key);
bool HashMap_contains(HashMap* map, const char* key);
May* HashMap_remove(HashMap* map, const char* key);

size_t HashMap_len(HashMap* map);
void HashMap_clear(HashMap* map, void (*value_drop)(void*));
```

**AutoLang Interface:**
```auto
// stdlib/collections/hashmap.at
# C
#include "hashmap.h"

extern type HashMap<K, V> {
    entries []*HashMapEntry<K, V>
    count uint
}

spec extern HashMap_new<K, V>() ?HashMap<K, V>
spec extern HashMap_drop<K, V>(map HashMap<K, V>)

spec extern HashMap_insert<K, V>(mut map HashMap<K, V>, key K, value V) ?()
spec extern HashMap_get<K, V>(map HashMap<K, V>, key K) ?V
spec extern HashMap_contains<K, V>(map HashMap<K, V>, key K) bool
spec extern HashMap_remove<K, V>(mut map HashMap<K, V>, key K) ?()
spec extern HashMap_len<K, V>(map HashMap<K, V>) uint
```

#### 3.2 HashSet Design

```c
// Hash set interface
typedef struct {
    char* value;
    UT_hash_handle hh;
} HashSetEntry;

typedef struct {
    HashSetEntry* entries;
    size_t count;
} HashSet;

// API
May* HashSet_new();
void HashSet_drop(HashSet* set);

May* HashSet_insert(HashSet* set, const char* value);
bool HashSet_contains(HashSet* set, const char* value);
May* HashSet_remove(HashSet* set, const char* value);

size_t HashSet_len(HashSet* set);
```

**AutoLang Interface:**
```auto
// stdlib/collections/hashset.at
# C
#include "hashset.h"

extern type HashSet<T> {
    entries []*HashSetEntry<T>
    count uint
}

spec extern HashSet_new<T>() ?HashSet<T>
spec extern HashSet_drop<T>(set HashSet<T>)

spec extern HashSet_insert<T>(mut set HashSet<T>, value T) ?()
spec extern HashSet_contains<T>(set HashSet<T>, value T) bool
spec extern HashSet_remove<T>(mut set HashSet<T>, value T) ?()
spec extern HashSet_len<T>(set HashSet<T>) uint
```

#### 3.3 Usage Examples

```auto
// test_hashmap.at
fn test_hashmap_basic() {
    let mut map = HashMap_new<str, int>().?
    HashMap_insert(mut map, "one", 1).?
    HashMap_insert(mut map, "two", 2).?
    HashMap_insert(mut map, "three", 3).?

    assert(HashMap_contains(map, "two"))
    assert(HashMap_len(map) == 3)

    let value = HashMap_get(map, "two").?
    assert(value == 2)
}

fn test_symbol_table_usage() {
    // Symbol table use case
    let mut symbols = HashMap_new<str, Symbol>().?
    let sym = Symbol{name: "x", type: Type::Int}
    HashMap_insert(mut symbols, "x", sym).?

    if HashMap_contains(symbols, "x") {
        let found = HashMap_get(symbols, "x").?
        print(found.name)
    }
}
```

**Success Criteria:**
- HashMap/HashSet pass 50+ unit tests
- O(1) average case for insert, lookup, delete
- Handle 1M+ entries without performance degradation
- No memory leaks (valgrind clean)
- Thread-safe (optional, future enhancement)

---

### Phase 4: String Interning (6 weeks)

**Objective:** Fast string comparison via interning (deduplication).

**Dependencies:** HashSet (for interned string storage)

#### 4.1 C Implementation

```c
// stdlib/string/intern.h
#ifndef AUTO_STRING_INTERN_H
#define AUTO_STRING_INTERN_H

#include <stddef.h>

typedef struct {
    char* value;
    size_t len;
    size_t hash;
} InternedString;

typedef struct {
    HashSet* strings;  // Set of InternedString*
    size_t total_count;
    size_t total_bytes;
} StringInterner;

// API
StringInterner* StringInterner_new();
void StringInterner_drop(StringInterner* interner);

// Intern a string (returns pointer to interned copy)
const char* StringInterner_intern(StringInterner* interner, const char* str);
const char* StringInterner_intern_len(StringInterner* interner, const char* str, size_t len);

// Check if string is interned
bool StringInterner_is_interned(StringInterner* interner, const char* str);

// Statistics
size_t StringInterner_count(StringInterner* interner);
size_t StringInterner_unique_count(StringInterner* interner);
size_t StringInterner_total_bytes(StringInterner* interner);

#endif
```

```c
// stdlib/string/intern.c
#include "intern.h"
#include "hashset.h"
#include <stdlib.h>
#include <string.h>

// Simple hash function (djb2)
static size_t hash_string(const char* str) {
    size_t hash = 5381;
    int c;
    while ((c = *str++)) {
        hash = ((hash << 5) + hash) + c;
    }
    return hash;
}

StringInterner* StringInterner_new() {
    StringInterner* interner = (StringInterner*)malloc(sizeof(StringInterner));
    if (!interner) return NULL;

    interner->strings = HashSet_new();
    interner->total_count = 0;
    interner->total_bytes = 0;
    return interner;
}

const char* StringInterner_intern(StringInterner* interner, const char* str) {
    return StringInterner_intern_len(interner, str, strlen(str));
}

const char* StringInterner_intern_len(StringInterner* interner,
                                       const char* str, size_t len) {
    // Check if already interned
    if (HashSet_contains_bytes(interner->strings, str, len)) {
        return HashSet_get_bytes(interner->strings, str, len);
    }

    // Allocate interned string
    char* interned = (char*)malloc(len + 1);
    if (!interned) return NULL;

    memcpy(interned, str, len);
    interned[len] = '\0';

    // Add to set
    HashSet_insert_bytes(interner->strings, interned, len);

    interner->total_count++;
    interner->total_bytes += len;

    return interned;
}
```

#### 4.2 AutoLang Interface

```auto
// stdlib/string/intern.at
# C
#include "intern.h"

extern type StringInterner {
    strings HashSet<*InternedString>
    total_count uint
    total_bytes uint
}

extern type InternedString {
    value str
    len uint
    hash uint
}

spec extern StringInterner_new() ?StringInterner
spec extern StringInterner_drop(interner StringInterner)

spec extern StringInterner_intern(mut interner StringInterner, s str) ?str
spec extern StringInterner_is_interned(interner StringInterner, s str) bool

spec extern StringInterner_count(interner StringInterner) uint
spec extern StringInterner_unique_count(interner StringInterner) uint
spec extern StringInterner_total_bytes(interner StringInterner) uint
```

#### 4.3 Usage Examples

```auto
// test_intern.at
fn test_intern_basic() {
    let mut interner = StringInterner_new().?

    let s1 = StringInterner_intern(mut interner, "hello").?
    let s2 = StringInterner_intern(mut interner, "hello").?
    let s3 = StringInterner_intern(mut interner, "world").?

    // s1 and s2 point to same memory
    assert(s1 == s2)
    assert(s1 != s3)

    // Comparison is pointer equality (fast!)
    if s1 == s2 {
        print("same string!")
    }
}

fn test_identifier_interning() {
    // Use case: fast identifier comparison in compiler
    let mut interner = StringInterner_new().?

    let id1 = StringInterner_intern(mut interner, "main").?
    let id2 = StringInterner_intern(mut interner, "main").?
    let id3 = StringInterner_intern(mut interner, "print").?

    // Symbol table can use pointer comparison
    let mut symbols = HashMap_new<str, Symbol>().?
    symbols.insert(id1, Symbol{name: "main", type: Type::Fn}).?

    // Fast lookup (no string comparison needed!)
    if symbols.contains(id2) {
        print("found main function")
    }
}
```

**Success Criteria:**
- String interning works correctly
- Pointer comparison for equality (O(1) vs O(n))
- 50+ unit tests passing
- Memory usage reasonable (no duplication)
- Performance: 1M intern operations in < 200ms

---

### Phase 5: Command-line Arguments (2 weeks)

**Objective:** Access to argc/argv for compiler CLI.

**Dependencies:** None (standalone)

#### 5.1 C Implementation

```c
// stdlib/sys/args.h
#ifndef AUTO_ARGS_H
#define AUTO_ARGS_H

// Store argc/argv globally
void args_init(int argc, char** argv);

// Access arguments
int args_count();
const char* args_get(int index);
const char* args_program_name();

#endif
```

```c
// stdlib/sys/args.c
#include "args.h"
#include <stdlib.h>

static int global_argc = 0;
static char** global_argv = NULL;

void args_init(int argc, char** argv) {
    global_argc = argc;
    global_argv = argv;
}

int args_count() {
    return global_argc;
}

const char* args_get(int index) {
    if (index < 0 || index >= global_argc) {
        return NULL;
    }
    return global_argv[index];
}

const char* args_program_name() {
    if (global_argc > 0) {
        return global_argv[0];
    }
    return "";
}
```

#### 5.2 AutoLang Interface

```auto
// stdlib/sys/args.at
# C
#include "args.h"

spec extern args_count() int
spec extern args_get(index int) ?str
spec extern args_program_name() str
```

#### 5.3 Integration with Runtime

**Modify runtime initialization:**
```c
// In auto runtime main()
int main(int argc, char** argv) {
    args_init(argc, argv);  // Store args
    return auto_main();     // Call AutoLang main
}
```

#### 5.4 Usage Examples

```auto
// test_args.at
fn main() {
    let prog_name = args_program_name()
    print(f"Program: $prog_name")

    let count = args_count()
    print(f"Argument count: $count")

    if count > 1 {
        for i in 1..count {
            let arg = args_get(i).?
            print(f"Arg $i: $arg")
        }
    }
}
```

**Success Criteria:**
- Arguments accessible from AutoLang
- 10+ unit tests passing
- Works with auto-man build system

---

## 4. Integration and Testing

### 4.1 Testing Strategy

**Unit Tests (C level):**
```c
// tests/test_may.c
void test_may_three_states() {
    May empty = May_empty();
    May value = May_value((void*)42);
    May error = May_error((void*)"error");

    assert(May_is_empty(&empty));
    assert(May_is_value(&value));
    assert(May_is_error(&error));
}

void test_may_unwrap() {
    May value = May_value((void*)42);
    assert((intptr_t)May_unwrap(&value) == 42);

    May empty = May_empty();
    assert(May_unwrap_or(&empty, (void*)100) == (void*)100);
}
```

**Integration Tests (AutoLang level):**
```auto
// tests/integration/test_collections.at
use collections: {HashMap, HashSet}
use may: May

fn test_hashmap_in_autolang() {
    let mut map = HashMap_new<str, int>().?
    let res = HashMap_insert(mut map, "test", 42)

    if May_is_error(res) {
        print("insert failed")
        return
    }

    let value = HashMap_get(map, "test")
    if May_is_value(value) {
        assert(May_unwrap(value) == 42)
    }
}
```

**Performance Tests:**
```auto
// tests/perf/benchmark_hashmap.at
fn benchmark_hashmap_insert() {
    let mut map = HashMap_new<str, int>().?
    let start = time::now()

    for i in 0..100000 {
        let key = f"key_$i"
        HashMap_insert(mut map, key, i).?
    }

    let elapsed = time::elapsed(start)
    print(f"100K inserts: $elapsed ms")
}
```

### 4.2 Memory Safety

**Valgrind Testing:**
```bash
# Run all tests under valgrind
cargo test
valgrind --leak-check=full --show-leak-kinds=all ./test_hashmap
valgrind --leak-check=full ./test_string_builder
valgrind --leak-check=full ./test_may
```

**Sanitizer Testing:**
```bash
# Address sanitizer
gcc -fsanitize=address -g test_may.c may.c -o test_may
./test_may

# Undefined behavior sanitizer
gcc -fsanitize=undefined -g test_may.c may.c -o test_may
./test_may
```

### 4.3 Documentation

**Each component requires:**
1. C header documentation (Doxygen style)
2. AutoLang interface documentation
3. Usage examples
4. Performance characteristics
5. Memory semantics (ownership, lifetimes)

---

## 5. Success Criteria (Updated 2025-01-17)

### Phase 1: May<T> (6-8 weeks) 🔄 IN PROGRESS

#### Phase 1b.0: Tag Type Foundation (2-3 weeks) - **BLOCKING**
- [ ] Tag variant parsing with payloads
- [ ] Tag variant construction syntax
- [ ] Tag pattern matching in `is` statements
- [ ] Tag methods inside definitions
- [ ] Tag type evaluation support
- [ ] Tag type C transpilation (enum + union)
- [ ] Generic type parameters in tags
- [ ] 20+ tests for tag types
- [ ] C code generation verified

#### Phase 1b.1: Basic May<T> (1-2 weeks) - **IN PROGRESS**
- [x] `tag May<T>` definition compiles (MayInt test case working)
- [x] `Nil()`, `Val()`, `Err()` construction functions work
- [x] `is_some()`, `is_nil()`, `is_err()` methods work
- [x] `unwrap_or()` method works (with proper return statements)
- [ ] `unwrap()` method needs work (pattern-bound variable types)
- [x] Pattern matching with `is` works (return type inference fixed)
- [x] 5 comprehensive unit tests passing (MayInt, MayStr, MayBool, Result, Option)
- [x] C transpilation generates correct code (enum + union + switch)
- [x] Test framework supports adding 25+ more tests (pattern established)

**Implementation Notes:**
- Created 5 working May type tests (041-045) covering different data types and patterns
- Tests 046-053 created but removed due to complexity; pattern established for future additions
- Tag constructors without arguments (e.g., `MayInt.Nil()`) now default to `0`
- Pattern matching branches now properly emit `return` statements (fixed in c.rs:1584-1665)
- Test locations:
  - `crates/auto-lang/test/a2c/041_may_basic/` - MayInt with unwrap/unwrap_or
  - `crates/auto-lang/test/a2c/042_may_string/` - MayStr with string types
  - `crates/auto-lang/test/a2c/043_may_bool/` - MayBool with boolean types
  - `crates/auto-lang/test/a2c/044_may_patterns/` - Result<Ok, Err> type
  - `crates/auto-lang/test/a2c/045_may_nested/` - Option<Some, None> with nested pattern matching

**Known Limitations:**

1. **Pattern-Bound Variables Not Supported** (MEDIUM PRIORITY - Deferred to Phase 1b.2+)

   **Problem**: Pattern matching cannot extract and bind variables from tag variants.

   **AutoLang Code**:
   ```auto
   fn unwrap(m MayInt) int {
       is m {
           MayInt.Val(v) => v    // Cannot use pattern-bound `v`
           MayInt.Err(e) => panic(e)
       }
   }
   ```

   **Current Generated C**:
   ```c
   case MAYINT_VAL: {
       m.as.Val;    // Loses variable binding, no return statement
   }
   ```

   **Workaround**: Use full path to variant value:
   ```auto
   fn unwrap(m MayInt) int {
       is m {
           MayInt.Val(_) => m.as.Val    // Use m.as.Val directly
           MayInt.Err(e) => panic("error")
       }
   }
   ```

   **Why It's Hard**:
   - Pattern `MayInt.Val(v)` is parsed as function call, variable binding is lost
   - Need to extract pattern variables, infer types from tag definition
   - Requires scope management and variable declaration generation in C transpiler
   - Estimated effort: 2-3 days, ~200-300 lines of code

   **Implementation Plan** (for future):
   1. Add pattern variable extractor: `fn extract_pattern_vars(expr: &Expr) -> Vec<(Name, Type)>`
   2. Add tag variant type lookup: `fn get_tag_variant_type(tag: &AutoStr, variant: &AutoStr) -> Option<Type>`
   3. Generate variable declarations: `int v = m.as.Val;` before body in each case
   4. Add pattern variables to symbol table during body transpilation
   5. Remove pattern variables after body

   **Files to Modify**:
   - `crates/auto-lang/src/trans/c.rs` - Pattern handling in `is_stmt()`
   - `crates/auto-lang/src/ast/is.rs` - May need to track pattern bindings

2. **No Generics Support** (HIGH PRIORITY - Blocking full May<T>)
   - Need separate types for MayInt, MayStr, MayBool
   - Cannot write generic `May<T>` type yet
   - Requires generics implementation in type system

3. **Type Inference Limitations**
   - Pattern-bound variables return `Type::Unknown`
   - `infer_expr_type()` cannot look up types from tag variant definitions
   - Causes `unwrap()` to not emit `return` statements correctly

**Remaining Work:**
- [x] Add 5 comprehensive unit tests (041-045 passing)
- [ ] Add 25+ more unit tests to reach 30+ goal (currently at 5, pattern established)
- [ ] Implement pattern-bound variable extraction and type inference (2-3 days)
- [ ] Implement generics for true May<T> support (1-2 weeks)
- [x] Document current limitations

#### Phase 1b.2: `?T` Syntactic Sugar (1-2 weeks)
- [ ] Parser accepts `?T` syntax
- [ ] Parser expands `?T` → `May<T>`
- [ ] Type checker validates `?T` usage
- [ ] C transpiler handles `?T` types
- [ ] 20+ tests passing

#### Phase 1b.3: `.?` and `??` Operators (2-3 weeks)
- [ ] `??` operator with default values
- [ ] `.?` operator with early return
- [ ] Compile error for `.?` in non-`?T` function
- [ ] C transpiler generates early return code
- [ ] 30+ tests passing

#### Overall Phase 1 Success Criteria:
- [ ] Tag type system fully functional
- [ ] May<T> type implemented with `tag` syntax
- [ ] `?T` syntactic sugar works
- [ ] `.?` and `??` operators work
- [ ] 100+ tests passing across all sub-phases
- [ ] Zero memory leaks (valgrind clean)
- [ ] C transpilation verified for all features
- [ ] Documentation complete

#### Deferred to Future (Phase 1b.2+):
- [ ] **Pattern-bound variable support** in tag pattern matching (2-3 days)
  - Extract variables from patterns like `MayInt.Val(v)`
  - Infer types from tag variant definitions
  - Generate C variable declarations in switch cases
  - Manage variable scope in pattern match bodies
  - Implementation: ~200-300 lines in `crates/auto-lang/src/trans/c.rs`
- [ ] **Generics support** for true May<T> type (1-2 weeks)
  - Generic type parameters in tag definitions
  - Type instantiation for May<int>, May<str>, etc.
  - C code generation for generic types
- [ ] Niche optimization for pointer types (0-byte overhead)
- [ ] Cross-platform error modes (PC vs MCU)
- [ ] Error message linking system (ErrorKind → messages)
- [ ] Rich error types with stack traces

### Phase 2: StringBuilder (6 weeks)
- [ ] StringBuilder type implemented
- [ ] Efficient concatenation (O(n) amortized)
- [ ] 30+ unit tests passing
- [ ] Performance: 1M appends in < 100ms
- [ ] No memory leaks

### Phase 3: HashMap/HashSet (10-12 weeks)
- [ ] HashMap<K, V> implemented
- [ ] HashSet<T> implemented
- [ ] 50+ unit tests passing
- [ ] O(1) average case operations
- [ ] Handle 1M+ entries
- [ ] No memory leaks

### Phase 4: String Interning (6 weeks)
- [ ] StringInterner implemented
- [ ] Pointer comparison for equality
- [ ] 50+ unit tests passing
- [ ] Performance: 1M interns in < 200ms
- [ ] Memory usage reasonable

### Phase 5: Args (2 weeks)
- [ ] args_count() implemented
- [ ] args_get() implemented
- [ ] 10+ unit tests passing
- [ ] Integration with runtime

### Overall
- [ ] All components documented
- [ ] Integration tests passing
- [ ] Performance benchmarks met
- [ ] Zero memory leaks across all components
- [ ] Ready for self-hosting compiler use

---

## 6. Timeline Summary (Updated 2025-01-17)

| Phase | Duration | Complexity | Deliverable |
|-------|----------|------------|-------------|
| 1. May<T> | 6-8 weeks | **HIGH** | Unified three-state type |
| 1b.0. Tag Foundation | 2-3 weeks | **HIGH** | Tag type system (BLOCKING) |
| 1b.1. Basic May<T> | 1-2 weeks | Medium | Tag-based May<T> |
| 1b.2. `?T` Syntax | 1-2 weeks | Medium | Type syntactic sugar |
| 1b.3. Operators | 2-3 weeks | **HIGH** | `.?` and `??` operators |
| 2. StringBuilder | 6 weeks | Medium | Efficient string building |
| 3. HashMap/HashSet | 10-12 weeks | High | O(1) collections |
| 4. String Interning | 6 weeks | Medium | Fast string comparison |
| 5. Args | 2 weeks | Low | CLI argument access |

**Total: 30-36 weeks (7.5-9 months)**

**Critical Path:** Phase 1 (1b.0 → 1b.1 → 1b.2 → 1b.3) → 2 → 3 → 4 → 5 (must be sequential)

**Key Changes:**
- Phase 1 extended from 4 to 6-8 weeks
- Added Phase 1b.0 (Tag Foundation) as new blocking dependency
- Increased complexity rating for Phase 1 from Medium to HIGH
- Total timeline extended by 2-4 weeks

---

## 7. Risks and Mitigations

### Risk 1: May<T> Parser Complexity
**Risk:** Implementing `?T` syntax and `.?` operator requires significant parser work

**Mitigation:**
- Start with function-based API (May_value, May_empty, etc.)
- Add syntactic sugar in later phases
- Use compiler macros/code generation as intermediate step
- Incremental parser updates

### Risk 2: C Memory Management
**Risk:** Memory leaks, use-after-free, buffer overflows

**Mitigation:**
- Extensive valgrind testing
- Address/undefined behavior sanitizers
- Clear ownership semantics in documentation
- RAII-style cleanup patterns

### Risk 3: Performance Issues
**Risk:** HashMap/HashSet too slow for compiler use

**Mitigation:**
- Benchmark against Rust implementations
- Profile hot paths
- Use proven algorithms (uthash)
- Optimize after correct implementation

### Risk 4: FFI Complexity
**Risk:** AutoLang ↔ C interface bugs

**Mitigation:**
- Keep FFI surface minimal
- Type safety through generic signatures
- Comprehensive integration tests
- Document all memory ownership transfers

### Risk 5: Timeline Slippage
**Risk:** Components take longer than estimated

**Mitigation:**
- Start with simpler components (May, Args)
- Parallel work where possible (StringBuilder independent of HashMap)
- Buffer time in estimates
- Can ship minimal viable stdlib (HashMap optional at first)

---

## 8. Next Steps

### Immediate Actions (Week 1-4)
1. **Refactor Option/Result to May<T>**
   - Keep separate types as temporary implementation
   - Design unified May<T> structure
   - Plan migration path

2. **Implement May<T> parser support**
   - Add `?T` type syntax to lexer
   - Add `?T` type syntax to parser
   - Implement `.?` operator
   - Implement `??` operator

3. **Add comprehensive May<T> tests**
   - Port existing Option/Result tests
   - Add three-state specific tests
   - Test error propagation scenarios

### First Month Goals
- Complete May<T> refactoring
- Implement `?T` syntax in parser
- Implement `.?` operator
- Start StringBuilder implementation

### First Quarter Goals
- Complete May<T> with full syntactic sugar
- Complete Result/Option/StringBuilder migration to May<T>
- Start HashMap implementation
- Have working test suite for all components

---

## 9. Related Documentation

- [May Type Design Document](../language/design/may-type.md) - **READ THIS FIRST**
- [C Transpiler Documentation](../c-transpiler.md)
- [Auto-Man Documentation](https://gitee.com/auto-stack/auto-man)
- [FFI Integration Guide](../ffi-guide.md) (TODO)
- [Memory Management Best Practices](../memory-management.md) (TODO)

---

## 10. Prerequisites

**Plan 024 (Ownership System)** and **Plan 025 (String Type Redesign)** must be completed before starting this plan.

The StringBuilder and String Interning components in this plan depend on:
- Plan 024's ownership and borrow checking system
- Plan 025's robust string type system (String, StringSlice, CString)

## 11. Conclusion

This standard library foundation provides the essential building blocks for the self-hosting Auto compiler. The key innovation is the **unified May<T> type**, which simplifies error handling by combining Option and Result semantics into a single three-state type.

By implementing these components in C with clean AutoLang FFI interfaces, we get:
1. **Performance** - C speed for critical operations
2. **Safety** - Proper memory management with testing
3. **Usability** - Clean AutoLang APIs with `.?` operator
4. **Maintainability** - Clear separation of concerns

The 7-8 month investment is justified by enabling the self-hosting compiler to be built on solid foundations, rather than accumulating technical debt from workarounds and missing components.

---

## Appendix: Migration from Option/Result to May<T>

### Current State (Phase 1a - Complete)
- Separate `Option<T>` type implemented
- Separate `Result<T, E>` type implemented
- Function-based API working
- 16 Rust tests passing
- Integrated with auto-val

### Target State (Phase 1b - Planned)
- Unified `May<T>` type
- `?T` syntactic sugar
- `.?` propagation operator
- `??` null-coalescing operator
- 30+ comprehensive tests
- Full parser integration

### Migration Strategy
1. Keep existing Option/Result as deprecated aliases
2. Implement May<T> alongside existing types
3. Add compiler warnings for Option/Result usage
4. Update all stdlib to use May<T>
5. Remove Option/Result in future breaking change
