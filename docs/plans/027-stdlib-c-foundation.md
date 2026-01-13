# Standard Library C Foundation Implementation Plan

## Implementation Status: ⏳ PLANNED

**Dependencies:**
- Plan 024 (Ownership-Based Memory System) - Must complete first
- Plan 025 (String Type Redesign) - Must complete first
**Estimated Start:** After Plan 025 completion (~3-5 months from Plan 024 start)

## Executive Summary

Build foundational C standard library components required for the self-hosting Auto compiler. These components will be implemented in C and exposed to AutoLang through the C FFI (Foreign Function Interface), providing essential data structures and utilities for compiler operations.

**Timeline**: 6-8 months (after Plan 025)
**Complexity**: High (requires C expertise, memory management, AutoLang FFI integration)
**Priority:** BLOCKER - Must complete before self-hosting compiler can begin

**Key Components:**
1. HashMap/HashSet - O(1) lookups for symbol tables
2. StringBuilder - Efficient string concatenation for code generation
3. Result/Option types - Safe error handling
4. String interning - Fast identifier comparison
5. Command-line argument parsing - Compiler CLI

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
- ❌ HashMap/HashSet - Symbol tables need O(1) lookups
- ❌ StringBuilder - Code generation needs efficient string building
- ❌ Result/Option - Compiler pipeline needs error handling
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

## 2. Implementation Strategy

### 2.1 Component Design Principles

**All components will:**
1. Be implemented in C (for performance and control)
2. Expose clean AutoLang FFI interfaces
3. Use arena allocation where possible (reduce fragmentation)
4. Follow Rust implementation patterns as reference
5. Include comprehensive tests (AutoLang + C unit tests)
6. Handle errors gracefully (no segfaults)

### 2.2 Development Approach

**Incremental Development:**
1. Start with simplest component (Result/Option)
2. Build on each component (StringBuilder uses Result)
3. Test each component in isolation
4. Integration tests at the end

**Code Organization:**
```
stdlib/
├── collections/         # New: collection data structures
│   ├── hashmap.at
│   ├── hashmap.h
│   ├── hashmap.c
│   ├── hashset.at
│   ├── hashset.h
│   └── hashset.c
├── string/              # New: string utilities
│   ├── builder.at
│   ├── builder.h
│   ├── builder.c
│   ├── intern.at
│   ├── intern.h
│   └── intern.c
├── result/              # New: error handling
│   ├── result.at
│   ├── result.h
│   └── result.c
├── sys/                 # Enhanced: system utilities
│   ├── args.at          # New
│   ├── args.h           # New
│   └── args.c           # New
└── auto/                # Existing
    ├── io.at
    ├── math.at
    ├── str.at
    └── sys.at
```

---

## 3. Component Implementation Plans

### Phase 1: Result/Option Types (4 weeks)

**Objective:** Implement safe error handling types.

**Dependencies:** None (foundational)

#### 3.1 Option Type

**C Implementation:**
```c
// stdlib/result/option.h
#ifndef AUTO_OPTION_H
#define AUTO_OPTION_H

#include <stdbool.h>

typedef enum {
    Option_None,
    Option_Some
} OptionTag;

typedef struct {
    OptionTag tag;
    void* value;
} Option;

// API
Option Option_None_new();
Option Option_Some_new(void* value);
bool Option_is_some(Option* self);
bool Option_is_none(Option* self);
void* Option_unwrap(Option* self);
void Option_drop(Option* self);

#endif
```

**AutoLang Interface:**
```auto
// stdlib/result/option.at
# C
#include "option.h"

extern type Option<T> {
    None
    Some(value T)
}

spec extern Option_none<T>() Option<T>
spec extern Option_some<T>(value T) Option<T>
spec extern Option_is_some<T>(opt Option<T>) bool
spec extern Option_unwrap<T>(opt Option<T>) T
```

#### 3.2 Result Type

**C Implementation:**
```c
// stdlib/result/result.h
#ifndef AUTO_RESULT_H
#define AUTO_RESULT_H

#include <stdbool.h>

typedef enum {
    Result_Ok,
    Result_Err
} ResultTag;

typedef struct {
    ResultTag tag;
    void* value;
    char* error;
} Result;

// API
Result Result_Ok_new(void* value);
Result Result_Err_new(const char* error);
bool Result_is_ok(Result* self);
bool Result_is_err(Result* self);
void* Result_unwrap(Result* self);
char* Result_unwrap_err(Result* self);
void Result_drop(Result* self);

#endif
```

**AutoLang Interface:**
```auto
// stdlib/result/result.at
# C
#include "result.h"

extern type Result<T, E> {
    Ok(value T)
    Err(error E)
}

spec extern Result_ok<T, E>(value T) Result<T, E>
spec extern Result_err<T, E>(error E) Result<T, E>
spec extern Result_is_ok<T, E>(res Result<T, E>) bool
spec extern Result_unwrap<T, E>(res Result<T, E>) T
```

**Testing:**
```auto
// test_option_result.at
fn test_option_some() {
    let opt = Option_some(42)
    assert(Option_is_some(opt))
    assert(Option_unwrap(opt) == 42)
}

fn test_option_none() {
    let opt = Option_none<int>()
    assert(Option_is_none(opt))
}

fn test_result_ok() {
    let res = Result_ok<int, str>(42)
    assert(Result_is_ok(res))
    assert(Result_unwrap(res) == 42)
}

fn test_result_err() {
    let res = Result_err<int, str>("error message")
    assert(Result_is_err(res))
}
```

**Success Criteria:**
- Option and Result types work correctly
- No memory leaks (valgrind clean)
- 20+ unit tests passing
- Integration with existing auto-val types

---

### Phase 2: StringBuilder (6 weeks)

**Objective:** Efficient string concatenation for code generation.

**Dependencies:** Result (for error handling)

#### 2.1 C Implementation

```c
// stdlib/string/builder.h
#ifndef AUTO_STRING_BUILDER_H
#define AUTO_STRING_BUILDER_H

#include <stddef.h>

typedef struct {
    char* buffer;
    size_t len;
    size_t capacity;
} StringBuilder;

// API
StringBuilder* StringBuilder_new(size_t initial_capacity);
void StringBuilder_drop(StringBuilder* sb);

Result* StringBuilder_append(StringBuilder* sb, const char* str);
Result* StringBuilder_append_char(StringBuilder* sb, char c);
Result* StringBuilder_append_int(StringBuilder* sb, int value);

char* StringBuilder_build(StringBuilder* sb);  // Returns null-terminated string
void StringBuilder_clear(StringBuilder* sb);
size_t StringBuilder_len(StringBuilder* sb);

#endif
```

```c
// stdlib/string/builder.c
#include "builder.h"
#include "result.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

StringBuilder* StringBuilder_new(size_t initial_capacity) {
    StringBuilder* sb = (StringBuilder*)malloc(sizeof(StringBuilder));
    if (!sb) return NULL;

    sb->buffer = (char*)malloc(initial_capacity);
    if (!sb->buffer) {
        free(sb);
        return NULL;
    }

    sb->len = 0;
    sb->capacity = initial_capacity;
    sb->buffer[0] = '\0';
    return sb;
}

void StringBuilder_drop(StringBuilder* sb) {
    if (sb) {
        free(sb->buffer);
        free(sb);
    }
}

Result* StringBuilder_append(StringBuilder* sb, const char* str) {
    size_t str_len = strlen(str);

    // Resize if needed
    while (sb->len + str_len >= sb->capacity) {
        size_t new_capacity = sb->capacity * 2;
        char* new_buffer = (char*)realloc(sb->buffer, new_capacity);
        if (!new_buffer) {
            return Result_Err_new("out of memory");
        }
        sb->buffer = new_buffer;
        sb->capacity = new_capacity;
    }

    // Append string
    memcpy(sb->buffer + sb->len, str, str_len);
    sb->len += str_len;
    sb->buffer[sb->len] = '\0';

    return Result_Ok_new(sb);
}

Result* StringBuilder_append_int(StringBuilder* sb, int value) {
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

spec extern StringBuilder_new(capacity uint) Result<StringBuilder, str>
spec extern StringBuilder_drop(sb StringBuilder)

spec extern StringBuilder_append(mut sb StringBuilder, s str) Result<StringBuilder, str>
spec extern StringBuilder_append_char(mut sb StringBuilder, c char) Result<StringBuilder, str>
spec extern StringBuilder_append_int(mut sb StringBuilder, value int) Result<StringBuilder, str>

spec extern StringBuilder_build(sb StringBuilder) str
spec extern StringBuilder_clear(mut sb StringBuilder)
spec extern StringBuilder_len(sb StringBuilder) uint
```

#### 2.3 Usage Examples

```auto
// test_builder.at
fn test_builder_basic() {
    let mut sb = StringBuilder_new(16).unwrap()
    StringBuilder_append(mut sb, "hello")
    StringBuilder_append(mut sb, " ")
    StringBuilder_append(mut sb, "world")
    let result = StringBuilder_build(sb)
    assert(result == "hello world")
}

fn test_builder_code_gen() {
    let mut sb = StringBuilder_new(1024).unwrap()
    StringBuilder_append(mut sb, "int main() {\n")
    StringBuilder_append(mut sb, "    return 0;\n")
    StringBuilder_append(mut sb, "}\n")
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
HashMap* HashMap_new();
void HashMap_drop(HashMap* map, void (*value_drop)(void*));

Result* HashMap_insert(HashMap* map, const char* key, void* value);
Option* HashMap_get(HashMap* map, const char* key);
bool HashMap_contains(HashMap* map, const char* key);
Result* HashMap_remove(HashMap* map, const char* key);

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

spec extern HashMap_new<K, V>() HashMap<K, V>
spec extern HashMap_drop<K, V>(map HashMap<K, V>)

spec extern HashMap_insert<K, V>(mut map HashMap<K, V>, key K, value V) Result<(), str>
spec extern HashMap_get<K, V>(map HashMap<K, V>, key K) Option<V>
spec extern HashMap_contains<K, V>(map HashMap<K, V>, key K) bool
spec extern HashMap_remove<K, V>(mut map HashMap<K, V>, key K) Result<(), str>
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
HashSet* HashSet_new();
void HashSet_drop(HashSet* set);

Result* HashSet_insert(HashSet* set, const char* value);
bool HashSet_contains(HashSet* set, const char* value);
Result* HashSet_remove(HashSet* set, const char* value);

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

spec extern HashSet_new<T>() HashSet<T>
spec extern HashSet_drop<T>(set HashSet<T>)

spec extern HashSet_insert<T>(mut set HashSet<T>, value T) Result<(), str>
spec extern HashSet_contains<T>(set HashSet<T>, value T) bool
spec extern HashSet_remove<T>(mut set HashSet<T>, value T) Result<(), str>
spec extern HashSet_len<T>(set HashSet<T>) uint
```

#### 3.3 Usage Examples

```auto
// test_hashmap.at
fn test_hashmap_basic() {
    let mut map = HashMap_new<str, int>()
    HashMap_insert(mut map, "one", 1)
    HashMap_insert(mut map, "two", 2)
    HashMap_insert(mut map, "three", 3)

    assert(HashMap_contains(map, "two"))
    assert(HashMap_len(map) == 3)

    let value = HashMap_get(map, "two").unwrap()
    assert(value == 2)
}

fn test_symbol_table_usage() {
    // Symbol table use case
    let mut symbols = HashMap_new<str, Symbol>()
    let sym = Symbol{name: "x", type: Type::Int}
    HashMap_insert(mut symbols, "x", sym)

    if HashMap_contains(symbols, "x") {
        let found = HashMap_get(symbols, "x").unwrap()
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

spec extern StringInterner_new() StringInterner
spec extern StringInterner_drop(interner StringInterner)

spec extern StringInterner_intern(mut interner StringInterner, s str) str
spec extern StringInterner_is_interned(interner StringInterner, s str) bool

spec extern StringInterner_count(interner StringInterner) uint
spec extern StringInterner_unique_count(interner StringInterner) uint
spec extern StringInterner_total_bytes(interner StringInterner) uint
```

#### 4.3 Usage Examples

```auto
// test_intern.at
fn test_intern_basic() {
    let mut interner = StringInterner_new()

    let s1 = StringInterner_intern(mut interner, "hello")
    let s2 = StringInterner_intern(mut interner, "hello")
    let s3 = StringInterner_intern(mut interner, "world")

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
    let mut interner = StringInterner_new()

    let id1 = StringInterner_intern(mut interner, "main")
    let id2 = StringInterner_intern(mut interner, "main")
    let id3 = StringInterner_intern(mut interner, "print")

    // Symbol table can use pointer comparison
    let mut symbols = HashMap_new<str, Symbol>()
    symbols.insert(id1, Symbol{name: "main", type: Type::Fn})

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
spec extern args_get(index int) str
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
            let arg = args_get(i)
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
// tests/test_hashmap.c
void test_hashmap_insert_and_get() {
    HashMap* map = HashMap_new();
    HashMap_insert(map, "key", (void*)42);

    Option* result = HashMap_get(map, "key");
    assert(Option_is_some(result));
    assert((intptr_t)Option_unwrap(result) == 42);

    HashMap_drop(map, NULL);
}

void test_hashmap_collision() {
    // Test hash collision handling
    HashMap* map = HashMap_new();
    HashMap_insert(map, "abc", (void*)1);
    HashMap_insert(map, "def", (void*)2);  // Different key, same hash

    assert(HashMap_len(map) == 2);

    HashMap_drop(map, NULL);
}
```

**Integration Tests (AutoLang level):**
```auto
// tests/integration/test_collections.at
use collections: {HashMap, HashSet}
use result: Result

fn test_hashmap_in_autolang() {
    let mut map = HashMap_new<str, int>()
    let res = HashMap_insert(mut map, "test", 42)

    if Result_is_err(res) {
        print("insert failed")
        return
    }

    let value = HashMap_get(map, "test")
    if Option_is_some(value) {
        assert(Option_unwrap(value) == 42)
    }
}
```

**Performance Tests:**
```auto
// tests/perf/benchmark_hashmap.at
fn benchmark_hashmap_insert() {
    let mut map = HashMap_new<str, int>()
    let start = time::now()

    for i in 0..100000 {
        let key = f"key_$i"
        HashMap_insert(mut map, key, i)
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
```

**Sanitizer Testing:**
```bash
# Address sanitizer
gcc -fsanitize=address -g test_hashmap.c hashmap.c -o test_hashmap
./test_hashmap

# Undefined behavior sanitizer
gcc -fsanitize=undefined -g test_hashmap.c hashmap.c -o test_hashmap
./test_hashmap
```

### 4.3 Documentation

**Each component requires:**
1. C header documentation (Doxygen style)
2. AutoLang interface documentation
3. Usage examples
4. Performance characteristics
5. Memory semantics (ownership, lifetimes)

---

## 5. Success Criteria

### Phase 1: Result/Option (4 weeks)
- [ ] Option<T> type implemented
- [ ] Result<T, E> type implemented
- [ ] 20+ unit tests passing
- [ ] No memory leaks (valgrind clean)
- [ ] Integration with auto-val

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

## 6. Timeline Summary

| Phase | Duration | Complexity | Deliverable |
|-------|----------|------------|-------------|
| 1. Result/Option | 4 weeks | Medium | Safe error handling |
| 2. StringBuilder | 6 weeks | Medium | Efficient string building |
| 3. HashMap/HashSet | 10-12 weeks | High | O(1) collections |
| 4. String Interning | 6 weeks | Medium | Fast string comparison |
| 5. Args | 2 weeks | Low | CLI argument access |

**Total: 28-34 weeks (7-8.5 months)**

**Critical Path:** Phase 1 → 2 → 3 → 4 → 5 (must be sequential)

---

## 7. Risks and Mitigations

### Risk 1: C Memory Management
**Risk:** Memory leaks, use-after-free, buffer overflows

**Mitigation:**
- Extensive valgrind testing
- Address/undefined behavior sanitizers
- Clear ownership semantics in documentation
- RAII-style cleanup patterns

### Risk 2: Performance Issues
**Risk:** HashMap/HashSet too slow for compiler use

**Mitigation:**
- Benchmark against Rust implementations
- Profile hot paths
- Use proven algorithms (uthash)
- Optimize after correct implementation

### Risk 3: FFI Complexity
**Risk:** AutoLang ↔ C interface bugs

**Mitigation:**
- Keep FFI surface minimal
- Type safety through generic signatures
- Comprehensive integration tests
- Document all memory ownership transfers

### Risk 4: Timeline Slippage
**Risk:** Components take longer than estimated

**Mitigation:**
- Start with simpler components (Result, Args)
- Parallel work where possible (StringBuilder independent of HashMap)
- Buffer time in estimates
- Can ship minimal viable stdlib (HashMap optional at first)

---

## 8. Next Steps

### Immediate Actions (Week 1-2)
1. **Set up development environment**
   - Create stdlib/collections, stdlib/string, stdlib/result directories
   - Add uthash dependency
   - Set up test infrastructure

2. **Implement Option type**
   - Write option.h/option.c
   - Create option.at with FFI
   - Add unit tests
   - Verify no memory leaks

3. **Implement Result type**
   - Write result.h/result.c
   - Create result.at with FFI
   - Add unit tests
   - Integration with Option

### First Month Goals
- Complete Option and Result types
- Start StringBuilder implementation
- Set up CI for valgrind testing

### First Quarter Goals
- Complete Result/Option/StringBuilder
- Start HashMap implementation
- Have working test suite for all components

---

## 9. Related Documentation

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

This standard library foundation provides the essential building blocks for the self-hosting Auto compiler. By implementing these components in C with clean AutoLang FFI interfaces, we get:

1. **Performance** - C speed for critical operations
2. **Safety** - Proper memory management with testing
3. **Usability** - Clean AutoLang APIs
4. **Maintainability** - Clear separation of concerns

The 7-8 month investment is justified by enabling the self-hosting compiler to be built on solid foundations, rather than accumulating technical debt from workarounds and missing components.
