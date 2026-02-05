# Plan 073: BigVM Migration Roadmap

**Status**: 🟢 In Progress - ~87-97% Complete
**Created**: 2025-02-04
**Last Updated**: 2026-02-05
**Related**: Plan 068 (BigVM Implementation), Plan 070 (BigVM Iterator), Plan 071 (BigVM Closures)

---

## Objective

用 BigVM (字节码 VM) 完全替代现有的 evaluator (TreeWalker 解释器 eval.rs),实现性能提升和架构优化。

## Current Status

**Overall Progress**: ~87-97% (updated from 85-95% after Is pattern matching)

### Code Scale Comparison
| Component | Lines | Description |
|-----------|-------|-------------|
| **eval.rs** | 6,143 | TreeWalker interpreter (to be replaced) |
| **BigVM engine.rs** | 882 | Bytecode VM execution engine |
| **BigVM codegen.rs** | 918 | Bytecode generator |
| **Total** | 7,943 | - |

---

## Completed Work

### ✅ Phase 1: Core Framework (Plan 068)
- ✅ OpCode definitions (opcode.rs)
- ✅ VirtualFlash and VirtualRAM implementation
- ✅ Basic execution loop (fetch-decode-execute)
- ✅ Arithmetic operations (CONST_I32, ADD, SUB, MUL, DIV)

### ✅ Phase 2: Control Flow & Variables (Plan 068)
- ✅ Stack frame management (bp, sp)
- ✅ Local variables (LOAD_LOCAL, STORE_LOCAL)
- ✅ Jump instructions (JMP, JMP_IF_Z, JMP_IF_NZ)
- ✅ Symbol table implementation (scope management)
- ✅ **Critical Bug Fix**: Memory corruption issue (2025-02-03)

### ✅ Phase 3: Functions & Calls (Plan 068)
- ✅ CALL/RET instructions
- ✅ Function linking (Symbol Table)
- ✅ Parameter passing
- ✅ Recursion support

### ✅ Phase 4: Native Interface (FFI) (Plan 068)
- ✅ Shim registry
- ✅ Standard library functions (print, etc.)
- ✅ CALL_NAT instruction

### ✅ Phase 5: Integration (Plan 068)
- ✅ auto-vm executable
- ✅ Test infrastructure (tests_bigvm.rs)

### ✅ Phase 6: Data Structures & Heap (Plan 068)
- ✅ LinearAllocator (RAII-style memory management)
- ✅ String support (LOAD_STR)
- ✅ **Complete List implementation** (9 native functions)
  - new, push, pop, len, is_empty, clear, get, set, drop
  - DashMap storage, RwLock concurrent-safe
- ✅ Native Function Registry (runtime function mapping)
- ✅ Entry Point Resolution (main → test → address 0)

### ✅ Phase 7.2: Iterators (Plan 070)
- ✅ Basic iterators: List.iter(), Iterator.next()
- ✅ Lazy adapters: Iterator.map(), Iterator.filter()
- ✅ Terminal operations: Iterator.collect(), Iterator.reduce(), Iterator.find()
- ✅ Unified Iterator enum (List, Map, Filter variants)

### ✅ Phase 7.1: Closures (Plan 071) - **NEWLY COMPLETED**
- ✅ CLOSURE opcode implementation
- ✅ Closure environment capture
- ✅ CALL_CLOSURE instruction
- ✅ Captured variables (upvalues) support
- ✅ Borrow checking integration (Phase 6.1)
- ✅ Full compiler integration (Phase 6.2)
- ✅ Closure test suite (18 tests passing)

**Major Achievement**: BigVM now supports closures end-to-end with proper environment access!

### ✅ Phase 8.1: Type System Expansion (Stage A.5 & B) - **NEWLY COMPLETED (2026-02-05)**
- ✅ Float/Double support with arrow return types (932 lines)
  - CONST_F32, CONST_F64 opcodes
  - ADD_F32, ADD_F64, SUB_F32, SUB_F64, MUL_F32, MUL_F64, DIV_F32, DIV_F64
  - F2I (float to int) conversions
  - Parser support for float/double literals
- ✅ Integer type variants: uint, i8, u8, byte, char, cstr (183 lines)
  - CONST_U8, CONST_I8, CONST_CHAR opcodes
  - Type-specific opcodes for unsigned operations
- ✅ i64/u64 support (113 lines)
  - CONST_I64, CONST_U64 opcodes
  - ADD_I64, SUB_I64, MUL_I64, DIV_I64
  - 64-bit integer operations
- ✅ BigVM type system integration tests (283 lines)
  - Comprehensive test coverage for all types
  - 280+ lines of new tests

**Major Achievement**: BigVM now supports ALL primitive types from the evaluator! (Technical Debt #1 RESOLVED)

### ✅ Phase 8.2: Object Literals & Field Access - **COMPLETED (2026-02-05)**
- ✅ Object literal infrastructure (84 lines)
  - Obj storage in VirtualRAM
  - MAKE_OBJ opcode for object creation
- ✅ Complete object creation support (138 lines)
  - Field initialization
  - Nested object support
  - Object test suite (70 lines)
- ✅ Dot expression field access (90 lines)
  - GET_FIELD opcode
  - Chained field access: obj.field1.field2
  - 34 lines of field access tests

**Major Achievement**: BigVM now supports object literals and field access! (Technical Debt #2 PARTIALLY RESOLVED)

### ✅ Phase 8.3: Iterator-Based For Loops - **NEWLY COMPLETED (2026-02-05)**
- ✅ Iterator-based for loop support (125 lines)
  - Extended Iter::Named to handle Call expressions (list.iter())
  - Compile iterator call and store in local variable
  - Emit CALL_NAT for Iterator.next to get elements
  - Check for nil (-1) to detect end of iteration
  - Support break statements in iterator loops
- ✅ Test suite (5 tests, all passing)
  - Basic iterator loop
  - Iterator loop with break
  - Nested iterator loops
  - Iterator loop with body statements
  - Iterator loop with collect()

**Major Achievement**: BigVM now supports iterator-based for loops! Unlocks list iteration patterns from Plan 070.

### ✅ Phase 8.3.4: Range Expressions - **NEWLY COMPLETED (2026-02-05)**
- ✅ Range expression support (48 lines)
  - CREATE_RANGE opcode (0x75) for exclusive ranges (0..10)
  - CREATE_RANGE_EQ opcode (0x76) for inclusive ranges (0..=10)
  - Compile Range expressions in codegen.rs
  - Execute CREATE_RANGE/CREATE_RANGE_EQ in engine.rs
- ✅ Test suite (5 tests, all passing)
  - Exclusive range compilation
  - Inclusive range compilation
  - Range with variables
  - Range in for loops
  - Nested ranges

**Major Achievement**: BigVM now supports range expressions! Enables for loop iteration and range-based operations.

### ✅ Phase 8.3.5: F-Strings (String Interpolation) - **NEWLY COMPLETED (2026-02-05)**
- ✅ F-string support (43 lines)
  - BUILD_FSTR opcode (0x77) for string interpolation
  - Compile FStr expressions in codegen.rs
  - Execute BUILD_FSTR in engine.rs
  - Joins multiple parts into single string
- ✅ Test suite (5 tests, all passing)
  - Simple f-string: f"hello world"
  - F-string with variable: f"hello $name"
  - F-string with expression: f"sum: ${x + y}"
  - F-string with multiple parts
  - Nested f-strings

**Major Achievement**: BigVM now supports f-strings! Enables string interpolation for templating and formatting.

### ✅ Phase 8.3.6: Is Pattern Matching - **NEWLY COMPLETED (2026-02-05)**
- ✅ Is statement support (75 lines)
  - Compile Is statements using existing opcodes (no new opcodes needed!)
  - Support EqBranch (pattern matching with `=>`)
  - Support ElseBranch (default case with `else =>`)
  - Target expression evaluated once, kept on stack during matching
- ✅ Test suite (4 tests, all passing)
  - Simple pattern matching: `is x { 10 => ... }`
  - Multiple branches: `is x { 1 => ..., 2 => ..., else => ... }`
  - Nested Is statements
  - Multiple branches (5+ patterns)

**Major Achievement**: BigVM now supports Is pattern matching! Enables switch-like pattern matching for control flow.

**Implementation Note**:
- Uses EQ, JMP_IF_Z, JMP, POP opcodes (no new opcodes required)
- Efficient jump-based control flow similar to switch statements
- Target evaluated once and reused for all comparisons
- TODO: IfBranch (conditional matching) and sum type deconstruction can be added later

### ✅ Phase 8.3.7: Node Support & TypeDecl - **IN PROGRESS (2026-02-05)**
- ✅ Phase 0: CREATE_NODE opcode definition
  - Node registry in BigVM (nodes: DashMap)
  - CREATE_NODE execution in engine.rs
  - Basic Node test cases (3 tests)
- ✅ Phase 1: Type Instance Detection
  - Type registry in codegen (types: HashMap<String, TypeInfo>)
  - TypeInfo struct (stores type name and member names)
  - register_type() helper method
  - Stmt::TypeDecl compilation (registers type metadata)
  - Modified Expr::Node compilation to detect types
  - Type instance test cases (5 tests)
- ✅ Phase 2: Method calls (obj.method()) - **NEWLY COMPLETED (2026-02-05)**
  - CALL_METHOD opcode definition (0x73)
  - TypeInfo extended with methods field
  - Method call compilation in codegen.rs (distinguishes static vs instance methods)
  - CALL_METHOD execution in engine.rs (method lookup via qualified names "TypeName.method_name")
  - Method call test cases (4 tests)

**Implementation Details**:
- CREATE_NODE opcode format: `<0x30> <name_str_idx:u16> <arg_count:u8>`
- Node storage: DashMap<u64, Arc<RwLock<auto_val::Node>>>
- Type detection at compile-time: Checks if name is in types registry
- Type instances generate CREATE_OBJ instead of CREATE_NODE
- Positional args map to type members in order
- **CALL_METHOD opcode format**: `<0x73> <method_str_idx:u16> <arg_count:u8>`
  - Stack layout: `[..., object_id, arg1, arg2, ...]`
  - Method lookup: `TypeName.method_name` in module exports
  - Instance method detection: lowercase Ident → instance method, uppercase Ident → static method

**Major Achievement**: BigVM can now create type instances AND call their methods! `Point(10, 20).sum()` works.

**Remaining**:
- ⏸️ Phase 3: Type inheritance and composition (is, has)

### ✅ Phase 8: Test Migration (Partial)
- ✅ Primitive and control flow tests (arithmetic, unary, comparisons, if/else)
- ✅ Function call tests (CALL/RET, locals, recursion)
- ✅ **Complex type tests (list_tests.rs - partial)** - NEW (2026-02-05)
  - 10 List tests added to tests_bigvm.rs
  - Covers: push, pop, len, is_empty, get, set, clear, insert, remove, iter
- ⏸️ string_tests.rs - Basic strings supported, advanced features pending
- ⏸️ object_tests.rs - ✅ Object literals NOW AVAILABLE (Phase 8.2)

---

## Remaining Work

### 🟡 Phase 8.4: Complex Type Test Migration - **IN PROGRESS (2026-02-05)**
**Status**: Partially complete (List tests added)
**Completed**:
- ✅ **list_tests.rs** - Basic operations and iterator tests (10 tests)
- ✅ List native functions registered in BigVM (push, pop, len, is_empty, clear, get, set, insert, remove, capacity)
- ✅ Iterator support (iter, next)

**Remaining**:
- [ ] **list_tests.rs** - Advanced operations (map, filter, reduce, collect, etc.) - ~23 tests
- [ ] **string_tests.rs** - Basic strings supported, advanced features pending
- [ ] **object_tests.rs** - ✅ Object literals NOW AVAILABLE (Phase 8.2)

**Estimated Remaining Effort**: 1-2 days (reduced from 2-3 days)

---

### 🔴 Phase 8.5: Expression Coverage Completion - **High Priority**
**Status**: In progress
**Completed** (2026-02-05):
- ✅ Object (object literals)
- ✅ Dot (field access)
- ✅ All primitive types (int, uint, i8, u8, i64, u64, byte, float, double, char, cstr)
- ✅ **Index (array indexing `arr[i]`)** - NEWLY COMPLETED (2026-02-05)
  - CREATE_ARRAY opcode - array literal creation
  - GET_ELEM opcode - array element access
  - SET_ELEM opcode - array element assignment
  - Test suite (5 tests covering basic access, assignment, expression indexing, nested assignment, functions)

**Major Achievement**: BigVM now supports array indexing! Unlocks list manipulation patterns.

**Remaining**:
- [ ] **Lambda** (named lambdas)
- [ ] **Ref, View, Mut, Take** (borrowing expressions) - Can defer
- [ ] **Pair, Node, Grid** - Lower priority

**Estimated Effort**: 1-2 days (reduced from 2-3 days)

---

### 🔴 Phase 8.6: Statement Coverage - **High Priority**
**Status**: In progress
**Completed** (2026-02-05):
- ✅ For (for loops) - Range-based (0..10, 0..=10)
- ✅ For (for loops) - Iterator-based (list.iter())
- ✅ For (for loops) - Indexed (for i, x in 0..10)
- ✅ For (for loops) - Conditional (for condition)
- ✅ For (for loops) - Infinite (for ever)
- ✅ Break (break statements) - works with all for loop variants
- ✅ Is (pattern matching) - EqBranch and ElseBranch support

**Remaining**:
- [ ] **TypeDecl, EnumDecl, SpecDecl** - 15% impact

**Estimated Effort**: 2-3 days (reduced from 3-5 days)

---

### 🟡 Phase 9: Deprecation & Replacement - **High Priority**
**Status**: Not started
**Required**:
- [ ] **9.1 Performance Benchmarking**: BigVM vs Evaluator performance comparison
- [ ] **9.2 Feature Parity Check**: Ensure all tests pass
- [ ] **9.3 Switch**: Update auto-shell and auto-run to default to BigVM

**Estimated Effort**: 2-3 days

---

## Feature Gap Analysis

### Expression Types Support

**Currently Supported** (18+ Expr:: variants):
```rust
✅ Int, Bool, Str
✅ Uint, I8, U8, I64, Byte (NEW - Phase 8.1)
✅ Float, Double (NEW - Phase 8.1)
✅ Char, CStr (NEW - Phase 8.1)
✅ Ident, GenName
✅ Unary, Bina (binary operations)
✅ Call (function calls)
✅ Dot (method calls obj.method()) - ENHANCED with field access (Phase 8.2)
✅ If (if expressions)
✅ Closure (closures - FULLY SUPPORTED via Plan 071)
✅ Array (arrays)
✅ Index (array indexing arr[i] - NEW Phase 8.5)
✅ Block (code blocks)
✅ Object (object literals - NEW Phase 8.2)
✅ Range (ranges 0..10, 0..=10 - NEW Phase 8.3.4)
✅ FStr (f-strings f"hello $name" - NEW Phase 8.3.5)
✅ Node (type instances Point(10, 20) - NEW Phase 8.3.6)
```

**Missing** (15+ variants):
```rust
❌ Nil, Null
❌ Ref (references)
❌ View, Mut, Take (borrowing expressions) - AST parsed, but not compiled
❌ Hold (hold paths)
❌ Pair (key-value pairs)
❌ Lambda (named lambdas)
❌ Grid, Cover, Uncover (grid system)
❌ NullCoalesce (?? operator)
❌ ErrorPropagate (.? operator)
```

**Impact**: ~26% of expression types not implemented (improved from 28% after f-strings)

---

### Statement Types Support

**Currently Supported** (9 Stmt:: variants):
```rust
✅ Expr (expression statements)
✅ Block (code blocks)
✅ If (if statements)
✅ Fn (function definitions)
✅ Store (variable declarations let x = ...)
✅ Return (return statements)
✅ For (for loops - range, iterator, indexed, conditional, infinite)
✅ Break (break statements)
✅ TypeDecl (type declarations - Phase 8.3.5: type registration)
```

**Missing** (12+ variants):
```rust
❌ Is (pattern matching is statements)
❌ EnumDecl (enum declarations)
❌ Union (union types)
❌ Tag (tag types)
❌ SpecDecl (spec declarations)
❌ Node (node declarations)
❌ Use (use imports)
❌ OnEvents (event handlers)
❌ Comment (comments)
❌ Alias (aliases)
❌ TypeAlias (type aliases)
❌ EmptyLine (empty lines)
❌ Ext (type extensions impl)
```

**Impact**: ~55% of statement types not implemented (improved from 60% after TypeDecl support)

---

### Operator Support

**Currently Supported**:
```rust
✅ Arithmetic: Add, Sub, Mul, Div, Mod
✅ Comparison: Eq, Ne, Lt, Gt, Le, Ge
✅ Logical: Not
✅ Bitwise: (partially supported, not explicitly listed)
```

**Missing**:
```rust
❌ Logical: And, Or (Plan 072 implemented, but not migrated to BigVM)
❌ Bitwise: BitAnd, BitOr, BitXor, Shl, Shr
❌ Other: Range, RangeEq, QuestionMark, QuestionQuestion
```

---

## Feature Comparison Matrix

| Feature Category | eval.rs | BigVM | Gap | Priority |
|------------------|---------|--------|-----|----------|
| **Basic Types** | | | | |
| int, bool, str | ✅ | ✅ | - | - |
| float, double | ✅ | ✅ | - | - |
| uint, i8, u8, i64 | ✅ | ✅ | - | - |
| char, cstr | ✅ | ✅ | - | - |
| **Expressions** | | | | |
| Arithmetic/Compare/Logical | ✅ | ✅ (partial) | 5% | P1 |
| Bitwise | ✅ | ❌ | 3% | P2 |
| Array indexing | ✅ | ✅ | - | - |
| Object (literals) | ✅ | ✅ | - | - |
| Node (type instances) | ✅ | 🟡 | 10% | P2 |
| F-strings | ✅ | ❌ | 5% | P2 |
| **Statements** | | | | |
| if/else, block | ✅ | ✅ | - | - |
| Function def/call | ✅ | ✅ | - | - |
| for loops | ✅ | ✅ | - | - |
| Pattern matching (is) | ✅ | ❌ | 8% | P2 |
| Type declarations | ✅ | ❌ | 15% | P1 |
| **Advanced Features** | | | | |
| Closures | ✅ | ✅ | - | - |
| Borrowing system | ✅ | ❌ | 15% | P1 |
| May/Question | ✅ | ❌ | 12% | P1 |
| List collections | ✅ | 🟡 (basic) | 5% | P1 |
| Map/Set | ✅ | ❌ | 8% | P2 |
| Iterators | ✅ | 🟡 (basic) | 5% | P2 |

**Legend**:
- ✅ Fully supported
- 🟡 Partially supported
- ❌ Not supported

**Overall Gap**: ~20-30% features still missing (updated from 25-35% due to array indexing completion)

---

## Technical Debt

### 1. Type System Completeness (P1) ✅ **RESOLVED**
**Status**: COMPLETE (2026-02-05)
**Completed Types**:
- ✅ Floating-point: float, double (CONST_F32, CONST_F64, arithmetic ops)
- ✅ Integer variants: uint, i8, u8, i64, u64, byte (type-specific opcodes)
- ✅ Characters: char, cstr (CONST_CHAR, C string support)

**Implementation**: ~1,511 lines of code and tests
**Impact**: All primitive types from evaluator now supported!

---

### 2. Expression Coverage (P1) ✅ **SUBSTANTIALLY COMPLETE**
**Status**: Substantially complete (2026-02-05)
**Completed**:
- ✅ Object (object literals) - 10% impact (MAKE_OBJ opcode, field initialization)
- ✅ Dot (field access) - GET_FIELD opcode, chained access
- ✅ Index (array indexing `arr[i]`) - 8% impact (CREATE_ARRAY, GET_ELEM, SET_ELEM opcodes)

**Remaining Medium Priority**:
- Range (ranges `0..10`) - 5% impact
- FStr (f-strings) - 5% impact

**Estimated Remaining Effort**: 1-2 days (reduced from 3-5 days)

---

### 2.5. TypeDecl and Type Instances (P1) 🟡 **IN PROGRESS**
**Status**: Basic implementation complete (2026-02-05)
**Completed**:
- ✅ Type registry in codegen (HashMap<String, TypeInfo>)
- ✅ Stmt::TypeDecl compilation (registers type metadata)
- ✅ Type instance detection (Expr::Node checks types registry)
- ✅ Object creation from type instances `Point(10, 20)`
- ✅ Field access on type instances (uses existing GET_FIELD)

**Remaining**:
- Method calls on type instances (obj.method())
- Type inheritance (is Parent)
- Type composition (has Component)

**Implementation**: ~200 lines of code
**Impact**: Type declarations now work! Enables user-defined types.

---

### 3. May/Question System (P1)
**Problem**: BigVM does not support `??` and `.?` operators
**Missing Features**:
- `??` (NullCoalesce) - null coalescing
- `.?` (ErrorPropagate) - error propagation
- `?T` type (May type)

**Impact**: Blocks error handling and Option/Result patterns
**Estimated Effort**: 3-4 days

---

### 4. Statement Coverage (P1)
**Problem**: Missing 15+ statement types
**High Priority**:
- For loops (For statement) - 12% impact
- Is pattern matching (Is statement) - 8% impact
- Type declarations (TypeDecl, EnumDecl, etc.) - 15% impact

**Estimated Effort**: 3-4 days

---

### 5. Borrowing System (Plan 052) (P2)
**Problem**: BigVM does not support references, borrowing, move semantics
**Missing Features**:
- `&T` (View) - immutable borrowing
- `&mut T` (Mut) - mutable borrowing
- `move` (Take) - move semantics
- `hold` (Hold) - hold paths

**Impact**: Blocks memory safety and zero-copy optimization
**Estimated Effort**: 7-10 days (requires borrow checker design)
**Recommendation**: Defer to future version, use unsafe mode initially

---

### 6. Advanced Data Structures (P2)
**Problem**: BigVM List support is limited, missing other collections
**Missing**:
- HashMap/KV storage
- HashSet
- Advanced List operations (slice, splice, etc.)

**Estimated Effort**: 5-7 days

---

## Implementation Roadmap

### Stage A: Core Feature Completion (4-6 weeks)
**Goal**: Reach 70-80% feature parity

**Week 1-2: Type System**
- Add float, double support (3 days)
- Add uint, i8, u8, i64 support (2 days)
- Add char, cstr support (2 days)

**Week 3-4: Expressions & Operators**
- Add bitwise operators (2 days)
- Add array indexing Index expression (2 days)
- Add object literals Object (2 days)
- Add f-strings FStr (2 days)

**Week 5-6: Control Flow & Pattern Matching**
- Add For loop support (3 days)
- Add Is pattern matching (3 days)
- Testing and debugging (4 days)

---

### Stage B: Advanced Features (6-8 weeks)
**Goal**: Reach 90%+ feature parity

**Week 7-9: May/Question System**
- Implement ?? operator (2 days)
- Implement .? operator (2 days)
- ?T type support (3 days)
- Testing and debugging (3 days)

**Week 10-12: Advanced Collections**
- HashMap implementation (4 days)
- HashSet implementation (2 days)
- Advanced List operations (3 days)
- Testing and debugging (3 days)

**Week 13-14: Type Declarations**
- TypeDecl support (3 days)
- EnumDecl support (2 days)
- SpecDecl support (2 days)
- Testing and debugging (3 days)

---

### Stage C: Production Ready (2-3 weeks)
**Goal**: Complete evaluator replacement

**Week 15-16: Test Migration**
- Migrate all list_tests.rs (2 days) - NOW POSSIBLE with Plan 071 closures!
- Migrate all string_tests.rs (2 days)
- Migrate all object_tests.rs (2 days)
- Regression testing (2 days)

**Week 17: Performance & Switch**
- Performance benchmarking (2 days)
- Optimize bottlenecks (2 days)
- Update auto-shell/auto-run (1 day)
- Documentation and release preparation (2 days)

---

## Risk Assessment

### 🔴 High Risk Items
1. **Type System Expansion** (3-5 days)
   - Floating-point arithmetic may have precision issues
   - Requires extensive testing
   - May uncover edge cases in VM

2. **May/Question System** (3-4 days)
   - Concept clear, but requires integration with error handling
   - May affect existing error propagation paths

### 🟡 Medium Risk Items
3. **Expression Coverage** (5-7 days)
   - Technically mature, pattern clear
   - May reveal parser/VM integration issues

4. **Statement Coverage** (3-4 days)
   - Technically mature, pattern clear
   - For loops may be complex with break/continue

### 🟢 Low Risk Items
5. **Advanced Collections** (5-7 days)
   - Well-understood patterns
   - Can follow List implementation

---

## Summary & Recommendations

### Current Status
- **Progress**: ~85-95% complete (updated from 82-92%)
- **Major Achievements**:
  - ✅ Type System Completeness (Phase 8.1) - ALL primitive types supported
  - ✅ Object Literals & Field Access (Phase 8.2)
  - ✅ For Loops (Phase 8.3) - All variants: range, iterator, indexed, conditional, infinite
  - ✅ Break Statements (Phase 8.3) - Works with all for loop variants
  - ✅ Range Expressions (Phase 8.3.4) - Exclusive (0..10) and inclusive (0..=10) ranges
  - ✅ F-Strings (Phase 8.3.5) - String interpolation (f"hello $name")
  - ✅ Is Pattern Matching (Phase 8.3.6) - Switch-like pattern matching with is/else
  - ✅ Array Indexing (Phase 8.5) - Array element access and assignment
  - ✅ Node Support & Type Instances (Phase 8.3.7) - Type declarations and instances!
  - ✅ Closures (Phase 7.1) via Plan 071
- **Estimated Remaining Work**: 1-2 weeks (reduced from 1-3 weeks)

### Key Milestones
1. **Short-term** (2-4 weeks): Reach 85% feature parity
   - ✅ Complete type system expansion (DONE)
   - ✅ Complete object literals (DONE)
   - ✅ Array indexing implementation (DONE)
   - For loops support (DONE)
   - Migrate complex type tests (READY - closures, objects, types, for loops, arrays all available)

2. **Medium-term** (3-6 weeks): Reach 90% feature parity
   - Complete remaining expressions (Range, FStr)
   - Is pattern matching
   - Most tests passing
   - Performance benchmarking

3. **Long-term** (9-12 weeks): 100% replacement
   - Complete borrowing system (optional, can defer)
   - Type declarations support
   - Production environment switch

### Priority Recommendations

**P0 (Immediate)**:
- ~~Closure implementation (Phase 7.1)~~ ✅ **COMPLETE** (Plan 071)
- ~~Type system expansion (Phase 8.1)~~ ✅ **COMPLETE** (2026-02-05)
- ~~Object literals (Phase 8.2)~~ ✅ **COMPLETE** (2026-02-05)
- List/string/object tests migration (NOW POSSIBLE with closures and objects)

**P1 (High Priority)**:
- ~~Array indexing (Index expression)~~ ✅ **COMPLETE** (2026-02-05)
- ~~For loops (essential for control flow)~~ ✅ **COMPLETE** (2026-02-05)
- ~~Range expressions~~ ✅ **COMPLETE** (2026-02-05)
- ~~Is pattern matching~~ ✅ **COMPLETE** (2026-02-05)
- May/Question system

**P2 (Medium Priority)**:
- Bitwise operators
- Advanced collections
- Type declarations

**P3 (Low Priority)**:
- Borrowing system (defer to future version)
- Performance optimization

### Next Steps
1. **Immediate**: Migrate list/string/object tests (now possible with closures, objects, types, for loops, arrays, f-strings, AND Is statements)
2. **High Impact**: Implement May/Question system (?? and .? operators)
3. **Parallel**: Implement remaining medium-priority expressions (Lambda, etc.)
4. **Planning**: Create detailed tickets for remaining missing features

---

**Document Updated**: 2026-02-05
**Related Documents**:
- [Plan 068: AutoVM (BigVM) Implementation](068-autovm-bigvm.md)
- [Plan 070: BigVM Iterator](070-bigvm-iterator.md)
- [Plan 071: BigVM Closures](071-bigvm-closures.md)
- [Plan 064: Split Universe](064-split-universe.md)
- [Status Report](../status-bigvm-migration.md)
