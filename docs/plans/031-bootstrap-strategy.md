# Bootstrap Strategy Implementation Plan

## Implementation Status: ⏳ PLANNED

**Priority:** CRITICAL - Resolves circular dependency for self-hosting
**Dependencies:** Plan 026 (Self-Hosting Compiler)
**Estimated Start:** Before starting Plan 024
**Timeline:** 4-6 weeks (planning + infrastructure)

## Executive Summary

Design and implement a three-stage bootstrap strategy to resolve the circular dependency: "compiler needs stdlib, stdlib needs compiler." This plan documents the bootstrapping process, validates the approach, and creates the infrastructure needed to achieve self-hosting.

**The Problem:**
```
Plan 026: Self-hosting compiler needs:
  → Plan 027: Stdlib C Foundation (HashMap, StringBuilder, String)
  → Plan 025: String Types
  → Plan 024: Ownership System

But Plan 027 needs:
  → AutoLang compiler to transpile .at files to C
  → C transpiler (a2c)
  → Which don't exist yet!
```

**The Solution:** Three-Stage Bootstrap

**Stage 1:** Rust compiler builds minimal Auto compiler
- Only core language features
- Manual memory management (no ownership system)
- No stdlib dependencies
- Output: `stage1_compiler` (C code + executable)

**Stage 2:** Stage 1 compiler builds stdlib
- HashMap, StringBuilder in pure C
- Exposed via FFI to AutoLang
- Output: `stdlib.a` + headers

**Stage 3:** Stage 1 compiler + stdlib builds full Auto compiler
- Now can use HashMap, StringBuilder, String
- Ownership system if available
- Feature-complete
- Output: `stage2_compiler` (self-hosted!)

**Stage 4:** Stage 2 compiler compiles itself
- Verify self-hosting works
- Stage 3 compiler should be identical to Stage 2
- Output: `stage3_compiler` (validation)

**Timeline:** 4-6 weeks to design and validate bootstrap approach
**Complexity:** High (requires careful dependency management, build orchestration)

---

## 1. Why Bootstrap Strategy is Critical

### 1.1 The Chicken-and-Egg Problem

**Without Bootstrap Strategy:**
- Can't build compiler without stdlib
- Can't build stdlib without compiler
- Deadlock! Project cannot proceed

**With Bootstrap Strategy:**
- Clear path from current state to self-hosting
- Minimal viable compiler breaks deadlock
- Incremental path to full self-hosting

### 1.2 Real-World Examples

**Go Language:**
```
Stage 1: Go 1.x compiler (written in Go) compiled with gcc
Stage 2: Bootstrap compiler compiled with Stage 1
Stage 3: Verify Stage 2 can compile itself
```

**Rust:**
```
Stage 1: rustc compiled with OCaml (early Rust)
Stage 2: rustc compiled with Stage 1
Stage 3: rustc compiled with Stage 2 (self-hosted)
```

**C Compiler (gcc):**
```
Stage 1: gcc compiled with existing C compiler
Stage 2: gcc compiled with Stage 1
Stage 3: Verify Stage 2 == Stage 1 (byte-for-byte)
```

**AutoLang will follow this proven pattern.**

---

## 2. Bootstrap Architecture

### 2.1 Stage 1: Minimal Compiler

**Constraints:**
- No stdlib dependencies (HashMap, StringBuilder, String)
- Manual memory management (malloc/free)
- Limited feature set
- Transpiles to C only

**Capabilities:**
```auto
// Core language support
fn main() {
    // Basic types
    let x int = 42
    let y float = 3.14
    let s str = "hello"  // C strings only

    // Arrays (fixed size)
    let arr [3]int = [1, 2, 3]

    // Functions
    fn add(a int, b int) int {
        return a + b
    }

    // Control flow
    if x > 0 {
        print("positive")
    }

    for i in 0..10 {
        print(i)
    }

    // Structs
    type Point {
        x int
        y int
    }

    let p = Point{x: 0, y: 0}

    // Manual memory management
    let data = malloc(100)
    // ... use data ...
    free(data)

    // FFI to C
    fn.c printf(fmt cstr) int
    printf("hello\n")
}
```

**NOT Supported in Stage 1:**
- ❌ HashMap, HashSet (not available yet)
- ❌ StringBuilder (not available yet)
- ❌ String type (only C strings)
- ❌ Ownership system (too complex)
- ❌ Borrow checker (not implemented)
- ❌ Pattern matching (limited)
- ❌ Generics (not implemented)
- ❌ Trait objects (not implemented)

**Implementation:**
- Written in AutoLang (`auto/compiler/stage1/*.at`)
- Uses C arrays for symbol tables (O(n) lookup, acceptable for small programs)
- Uses C strings (char*) for all string operations
- Manual malloc/free for memory management
- ~2000-3000 lines of AutoLang code

**Transpilation:**
```bash
# Use Rust compiler to transpile Stage 1
rustc → a2c → stage1/*.at → stage1/*.c
gcc → stage1/*.c → stage1_compiler.exe
```

### 2.2 Stage 2: Stdlib Build

**Objective:** Build stdlib components using Stage 1 compiler

**Components to Build:**

**HashMap (in pure C):**
```c
// stdlib/c/hashmap.c
struct HashMap {
    uint64_t* keys;
    void** values;
    size_t capacity;
    size_t size;
};

HashMap* HashMap_new(size_t capacity) {
    HashMap* map = malloc(sizeof(HashMap));
    map->keys = calloc(capacity, sizeof(uint64_t));
    map->values = calloc(capacity, sizeof(void*));
    map->capacity = capacity;
    map->size = 0;
    return map;
}

void HashMap_set(HashMap* map, uint64_t key, void* value) {
    size_t index = key % map->capacity;
    map->keys[index] = key;
    map->values[index] = value;
    map->size++;
}

void* HashMap_get(HashMap* map, uint64_t key) {
    size_t index = key % map->capacity;
    if (map->keys[index] == key) {
        return map->values[index];
    }
    return NULL;
}
```

**StringBuilder (in pure C):**
```c
// stdlib/c/string_builder.c
struct StringBuilder {
    char* data;
    size_t len;
    size_t cap;
};

StringBuilder* StringBuilder_new(size_t initial_cap) {
    StringBuilder* builder = malloc(sizeof(StringBuilder));
    builder->data = malloc(initial_cap);
    builder->len = 0;
    builder->cap = initial_cap;
    return builder;
}

void StringBuilder_append(StringBuilder* builder, const char* str) {
    size_t str_len = strlen(str);
    if (builder->len + str_len > builder->cap) {
        builder->cap *= 2;
        builder->data = realloc(builder->data, builder->cap);
    }
    memcpy(builder->data + builder->len, str, str_len);
    builder->len += str_len;
}

char* StringBuilder_build(StringBuilder* builder) {
    builder->data[builder->len] = '\0';
    return builder->data;
}
```

**String Type (if Plan 025 complete):**
```c
// stdlib/c/string.c
struct String {
    char* data;
    size_t len;
    size_t cap;
};

String* String_new(const char* utf8, size_t len) {
    String* s = malloc(sizeof(String));
    s->data = malloc(len + 1);
    memcpy(s->data, utf8, len);
    s->data[len] = '\0';
    s->len = len;
    s->cap = len;
    return s;
}

void String_drop(String* s) {
    free(s->data);
    free(s);
}
```

**Build Process:**
```bash
# Use Stage 1 compiler to build stdlib headers
# (stdio.h, stdlib.h, hashmap.h, string_builder.h, string.h)

# Compile C stdlib implementations
gcc -c stdlib/c/hashmap.c -o stdlib/obj/hashmap.o
gcc -c stdlib/c/string_builder.c -o stdlib/obj/string_builder.o
gcc -c stdlib/c/string.c -o stdlib/obj/string.o

# Package into static library
ar rcs stdlib/libstdlib.a \
    stdlib/obj/hashmap.o \
    stdlib/obj/string_builder.o \
    stdlib/obj/string.o

# Now AutoLang can use stdlib!
```

### 2.3 Stage 3: Full Compiler

**Objective:** Build full-featured compiler using Stage 1 + stdlib

**Now Supported:**
- ✅ HashMap, HashSet (from stdlib)
- ✅ StringBuilder (from stdlib)
- ✅ String type (from stdlib)
- ✅ Ownership system (if Plan 024 complete)
- ✅ Borrow checker (if Plan 024 Phase 3 complete)
- ✅ Pattern matching (if Plan 028 complete)
- ✅ Generics (if Plan 032 complete)

**Implementation:**
```auto
// auto/compiler/stage3/parser.at

use stdlib: {HashMap, StringBuilder}

type Parser {
    lexer Lexer
    cur Token
    errors []Error

    // Now we can use HashMap!
    symbols HashMap<str, Symbol>
}

fn parse_parser(mut parser Parser) Stmt? {
    // Use StringBuilder for efficient code generation
    let mut code = StringBuilder::new(1024)

    code.append("int main() {\n")

    // Parse and generate code
    // ...

    let result = code.build()

    return Some(result)
}
```

**Transpilation:**
```bash
# Use Stage 1 compiler to transpile Stage 3
stage1_compiler → stage3/*.at → stage3/*.c
gcc stage3/*.c -lstdlib -o stage2_compiler.exe
```

### 2.4 Stage 4: Validation

**Objective:** Verify self-hosting works

**Process:**
```bash
# Stage 2 compiler compiles itself
stage2_compiler → stage3/*.at → stage3_self/*.c
gcc stage3_self/*.c -lstdlib -o stage3_compiler.exe

# Compare Stage 2 and Stage 3
diff stage2_compiler.exe stage3_compiler.exe
# Should be identical (or functionally equivalent)

# Test on multiple files
stage2_compiler test1.at → test1.c
stage3_compiler test1.at → test1_self.c
diff test1.c test1_self.c
# Should be identical
```

**Success Criteria:**
- Stage 3 compiler compiles successfully
- Stage 3 produces identical output to Stage 2
- No runtime errors
- Performance within 2x of Stage 2

---

## 3. Implementation Phases

### Phase 1: Bootstrap Planning (1 week)

**Objective:** Document bootstrap architecture and identify dependencies

**Deliverables:**
1. Bootstrap architecture document
2. Dependency analysis for each stage
3. Feature matrix for Stage 1 compiler
4. Build orchestration plan

**Files to Create:**
```
docs/
└── bootstrap/
    ├── architecture.md      # Bootstrap design
    ├── stage1_spec.md       # Stage 1 compiler spec
    ├── stage2_plan.md       # Stdlib build plan
    └── validation.md        # Validation strategy
```

**Key Decisions:**
- What features in Stage 1 compiler?
- What stdlib components to build first?
- How to orchestrate multi-stage builds?
- How to validate self-hosting?

**Success Criteria:**
- Clear path from current state to self-hosting
- All dependencies documented
- Feature creep controlled (Stage 1 stays minimal)

---

### Phase 2: Stage 1 Compiler Specification (2 weeks)

**Objective:** Define exact feature set for Stage 1 compiler

**Deliverables:**
1. Stage 1 language spec (what IS supported)
2. Stage 1 exclusion list (what is NOT supported)
3. Stage 1 implementation structure
4. Stage 1 test plan

**Files to Create:**
```
auto/compiler/stage1/
├── README.md              # Stage 1 spec
├── lexer.at               # Minimal lexer
├── parser.at              # Minimal parser
├── transpiler.at          # C transpiler (minimal)
└── main.at                # Compiler driver
```

**Stage 1 Feature Specification:**

**Supported:**
```auto
// Types
int, uint, float, double, bool, char
str  // C strings only
[T]type  // Fixed arrays
type { ... }  // Structs

// Statements
let, mut, var
fn name(params...) ret_type { ... }
if cond { ... } else { ... }
for var in start..end { ... }
loop { ... } break
return expr

// Expressions
literals: 42, 3.14, "hello", 'a', true, false
identifiers
binary ops: +, -, *, /, %, ==, !=, <, >, <=, >=
unary ops: -, !, &
array indexing: arr[i]
struct field access: obj.field
function calls: fn(arg, ...)
array literals: [1, 2, 3]
struct literals: Point{x: 0, y: 0}

// FFI
fn.c printf(fmt cstr, ...) int
```

**NOT Supported:**
```auto
// No advanced types (Stage 1)
HashMap<T, U>  // Not available
StringBuilder  // Not available
String  // Not available (use C strings)
[T]type with runtime length  // Only fixed arrays

// No ownership (Stage 1)
take, edit, hold  // Not implemented

// No advanced features
is pattern matching  // Only basic version
generics<T>  // Not implemented
trait spec  // Not implemented
closures |x| x*2  // Not implemented

// No stdlib
use stdlib  // Doesn't exist yet
```

**Success Criteria:**
- Clear feature boundary documented
- No stdlib dependencies in Stage 1
- Can compile itself (eventually)

---

### Phase 3: Build Orchestration (1-2 weeks)

**Objective:** Create build system for multi-stage bootstrap

**Deliverables:**
1. Build script for Stage 1 → Stage 2 → Stage 3
2. Dependency tracking between stages
3. Validation automation
4. CI/CD integration

**Files to Create:**
```
scripts/
├── bootstrap.sh          # Main bootstrap script
├── build_stage1.sh       # Build Stage 1 compiler
├── build_stdlib.sh       # Build stdlib with Stage 1
├── build_stage3.sh       # Build Stage 3 compiler
└── validate.sh           # Validate self-hosting

auto-man.yaml             # Auto-Man integration
```

**Key Implementation:**

```bash
# scripts/bootstrap.sh
#!/bin/bash
set -e

echo "=== AutoLang Bootstrap Process ==="

# Stage 1: Build minimal compiler with Rust
echo "Stage 1: Building minimal compiler..."
./scripts/build_stage1.sh

# Stage 2: Build stdlib with Stage 1 compiler
echo "Stage 2: Building stdlib..."
./scripts/build_stdlib.sh

# Stage 3: Build full compiler with Stage 1 + stdlib
echo "Stage 3: Building full compiler..."
./scripts/build_stage3.sh

# Stage 4: Validate self-hosting
echo "Stage 4: Validating self-hosting..."
./scripts/validate.sh

echo "=== Bootstrap Complete! ==="
```

```bash
# scripts/build_stage1.sh
#!/bin/bash

# Use Rust compiler to transpile Stage 1
echo "Transpiling Stage 1 compiler..."
cargo run --release -- java-script \
    auto/compiler/stage1/*.at \
    -o build/stage1/

# Compile generated C code
echo "Compiling Stage 1 compiler..."
gcc -o build/stage1/compiler \
    build/stage1/*.c \
    -lm

echo "Stage 1 compiler: build/stage1/compiler"
```

```bash
# scripts/build_stdlib.sh
#!/bin/bash

# Build stdlib components in C
echo "Building stdlib..."

gcc -c stdlib/c/hashmap.c -o build/stdlib/hashmap.o
gcc -c stdlib/c/string_builder.c -o build/stdlib/string_builder.o
gcc -c stdlib/c/string.c -o build/stdlib/string.o

# Package into static library
ar rcs build/stdlib/libstdlib.a \
    build/stdlib/hashmap.o \
    build/stdlib/string_builder.o \
    build/stdlib/string.o

echo "Stdlib: build/stdlib/libstdlib.a"
```

```bash
# scripts/build_stage3.sh
#!/bin/bash

# Use Stage 1 compiler to transpile Stage 3
echo "Transpiling Stage 3 compiler..."
build/stage1/compiler \
    auto/compiler/stage3/*.at \
    -o build/stage3/

# Compile generated C code with stdlib
echo "Compiling Stage 3 compiler..."
gcc -o build/stage3/compiler \
    build/stage3/*.c \
    -Lbuild/stdlib -lstdlib \
    -lm

echo "Stage 3 compiler: build/stage3/compiler"
```

```bash
# scripts/validate.sh
#!/bin/bash

echo "Validating self-hosting..."

# Stage 3 compiler compiles itself
echo "Stage 3 compiling itself..."
build/stage3/compiler \
    auto/compiler/stage3/*.at \
    -o build/stage3_self/

gcc -o build/stage3_self/compiler \
    build/stage3_self/*.c \
    -Lbuild/stdlib -lstdlib \
    -lm

# Compare binaries
echo "Comparing Stage 3 and Stage 4..."
if diff build/stage3/compiler build/stage3_self/compiler; then
    echo "✓ Self-hosting validation passed!"
    exit 0
else
    echo "✗ Self-hosting validation failed!"
    echo "Binaries differ - investigate"
    exit 1
fi
```

**Auto-Man Integration:**
```yaml
# auto-man.yaml
project:
  name: "auto-lang"
  version: "0.1.0"

bootstrap:
  stage1:
    source: "auto/compiler/stage1/*.at"
    target: "build/stage1/compiler"
    builder: "rust"  # Use Rust compiler for Stage 1

  stdlib:
    source: "stdlib/c/*.c"
    target: "build/stdlib/libstdlib.a"
    builder: "gcc"

  stage3:
    source: "auto/compiler/stage3/*.at"
    target: "build/stage3/compiler"
    builder: "stage1"  # Use Stage 1 compiler
    depends: ["stage1", "stdlib"]

  validate:
    command: "./scripts/validate.sh"
    depends: ["stage3"]
```

**Success Criteria:**
- Bootstrap script runs end-to-end
- All stages build successfully
- Validation passes
- CI/CD integration works

---

### Phase 4: Testing & Validation (1 week)

**Objective:** Comprehensive testing of bootstrap process

**Deliverables:**
1. Stage 1 compiler test suite
2. Stdlib integration tests
3. Stage 3 compiler test suite
4. Self-hosting validation tests

**Files to Create:**
```
tests/bootstrap/
├── stage1/
│   ├── 001_hello.at
│   ├── 002_array.at
│   └── ...
├── stdlib/
│   ├── 001_hashmap.at
│   ├── 002_string_builder.at
│   └── ...
└── stage3/
    ├── 001_self_compile.at
    ├── 002_full_feature.at
    └── ...
```

**Validation Tests:**
```bash
# Test Stage 1 compiler
build/stage1/compiler tests/bootstrap/stage1/*.at

# Test stdlib
build/stage1/compiler tests/bootstrap/stdlib/*.at -lstdlib

# Test Stage 3 compiler
build/stage3/compiler tests/bootstrap/stage3/*.at -lstdlib

# Self-hosting test
build/stage3/compiler auto/compiler/stage3/*.at -o build/stage4/
# Should produce identical compiler
```

**Success Criteria:**
- All test suites pass
- No regressions between stages
- Self-hosting validated

---

## 4. Risk Mitigation

### Risk 1: Stage 1 Compiler Too Complex

**Problem:** Stage 1 grows beyond minimal capabilities

**Mitigation:**
- Strict feature gate
- Weekly review of Stage 1 scope
- "If in doubt, leave it out" philosophy
- Document every feature addition

### Risk 2: Stdlib Build Fails

**Problem:** Cannot build stdlib with Stage 1 compiler

**Mitigation:**
- Build stdlib components in pure C first (validate with gcc)
- Then integrate with Stage 1 compiler
- Keep C stdlib simple (no complex features)
- Extensive testing of stdlib components

### Risk 3: Bootstrap Breaking

**Problem:** Later stages cannot compile earlier stages

**Mitigation:**
- Freeze Stage 1 compiler after initial version
- Maintain compatibility contracts
- Version the compiler ABI
- Automated testing of all stages

### Risk 4: Performance Degradation

**Problem:** Each stage slower than previous

**Mitigation:**
- Benchmark each stage
- Performance budget: Stage N ≤ 1.2x Stage N-1
- Profile and optimize hot paths
- Accept slower initial stages

---

## 5. Success Criteria

### Phase 1 (Planning)
- [ ] Bootstrap architecture documented
- [ ] Dependencies identified
- [ ] Feature matrix defined
- [ ] Build plan approved

### Phase 2 (Stage 1 Spec)
- [ ] Stage 1 feature spec complete
- [ ] Exclusion list defined
- [ ] Implementation structure planned
- [ ] Test plan documented

### Phase 3 (Build Orchestration)
- [ ] Bootstrap script working
- [ ] All stages build successfully
- [ ] Auto-Man integration complete
- [ ] CI/CD pipeline working

### Phase 4 (Testing & Validation)
- [ ] All test suites pass
- [ ] Self-hosting validated
- [ ] Performance acceptable
- [ ] Documentation complete

### Overall
- [ ] Clear path from current state to self-hosting
- [ ] Bootstrap process reproducible
- [ ] All stages validated
- [ ] Can maintain and evolve bootstrap

---

## 6. Related Documentation

- **[Plan 024]:** Ownership-Based Memory System
- **[Plan 025]:** String Type Redesign
- **[Plan 026]:** Self-Hosting Compiler (depends on this plan)
- **[Plan 027]:** Stdlib C Foundation
- **[Go Bootstrap](https://golang.org/doc/install/source#bootstrap):** Reference
- **[Rust Bootstrap](https://rustc-dev-guide.rust-lang.org/building/how-to-build-and-run.html#building-the-compiler):** Reference

---

## 7. Open Questions

1. **Should Stage 1 be written in AutoLang or C?**
   - AutoLang: More authentic, validates language design
   - C: Faster to implement, less risky

2. **How to handle stdlib evolution?**
   - Keep Stage 1 stdlib minimal?
   - Allow stdlib to grow between stages?

3. **Should we support cross-compilation?**
   - Compile for different platforms in Stage 1?
   - Or keep it simple (single platform)?

4. **How to version the compiler ABI?**
   - Semantic versioning?
   - Compatibility guarantees?

5. **Should we cache intermediate build artifacts?**
   - Speed up rebuilds?
   - Or always rebuild from scratch?

---

## 8. Timeline Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 1. Bootstrap Planning | 1 week | Architecture doc |
| 2. Stage 1 Specification | 2 weeks | Stage 1 spec |
| 3. Build Orchestration | 1-2 weeks | Build scripts |
| 4. Testing & Validation | 1 week | Test suites |
| **Total** | **5-6 weeks** | **Complete bootstrap strategy** |

**Critical Path:** Phase 1 → 2 → 3 → 4

**Can Start:** Immediately (before or during Plan 024)

**Blocks:** Plan 026 (Self-Hosting Compiler)

---

## 9. Conclusion

This plan resolves the circular dependency problem through a proven three-stage bootstrap approach. By starting with a minimal compiler and incrementally adding features, we achieve self-hosting while maintaining project momentum.

**Key Benefits:**
1. **Clear path:** From current state to self-hosting
2. **Incremental:** Each stage builds on previous
3. **Validated:** Proven approach (Go, Rust, gcc)
4. **Maintainable:** Bootstrap process is automated and documented

**Next Steps:**
1. Review and approve bootstrap architecture
2. Begin Stage 1 compiler specification
3. Set up build orchestration
4. Start bootstrap process once Plan 024 begins

Once bootstrap strategy is complete, AutoLang will have a clear, validated path to self-hosting that can be executed incrementally while other plans (024-029, 032) are in progress.
