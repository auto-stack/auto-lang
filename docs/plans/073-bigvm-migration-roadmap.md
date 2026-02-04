# Plan 073: BigVM Migration Roadmap

**Status**: 🟡 In Progress - ~50-60% Complete
**Created**: 2025-02-04
**Last Updated**: 2025-02-04
**Related**: Plan 068 (BigVM Implementation), Plan 070 (BigVM Iterator), Plan 071 (BigVM Closures)

---

## Objective

用 BigVM (字节码 VM) 完全替代现有的 evaluator (TreeWalker 解释器 eval.rs),实现性能提升和架构优化。

## Current Status

**Overall Progress**: ~50-60% (updated from 40-50% after Plan 071 closure completion)

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

### ✅ Phase 8: Test Migration (Partial)
- ✅ Primitive and control flow tests (arithmetic, unary, comparisons, if/else)
- ✅ Function call tests (CALL/RET, locals, recursion)
- ⏸️ Complex type tests (list_tests.rs - partial, string_tests.rs, object_tests.rs - pending)

---

## Remaining Work

### 🔴 Phase 8.4: Complex Type Test Migration - **High Priority**
**Status**: Partially complete
**Required**:
- [ ] **list_tests.rs** - Requires closure support (NOW AVAILABLE via Plan 071)
- [ ] **string_tests.rs** - Basic strings supported, advanced features pending
- [ ] **object_tests.rs** - Requires Map implementation (Phase 6.3)

**Estimated Effort**: 3-5 days

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

**Currently Supported** (11 Expr:: variants):
```rust
✅ Int, Bool, Str
✅ Ident, GenName
✅ Unary, Bina (binary operations)
✅ Call (function calls)
✅ Dot (method calls obj.method())
✅ If (if expressions)
✅ Closure (closures - FULLY SUPPORTED via Plan 071)
✅ Array (arrays)
✅ Block (code blocks)
```

**Missing** (30+ variants):
```rust
❌ Uint, I8, U8, I64, Byte
❌ Float, Double
❌ Char, CStr
❌ Nil, Null
❌ Ref (references)
❌ View, Mut, Take (borrowing expressions) - AST parsed, but not compiled
❌ Hold (hold paths)
❌ Range (ranges)
❌ Pair (key-value pairs)
❌ Object (object literals)
❌ Node (nodes)
❌ Index (array indexing arr[i])
❌ Lambda (named lambdas)
❌ FStr (f-strings)
❌ Grid, Cover, Uncover (grid system)
❌ NullCoalesce (?? operator)
❌ ErrorPropagate (.? operator)
```

**Impact**: ~60% of expression types not implemented

---

### Statement Types Support

**Currently Supported** (6 Stmt:: variants):
```rust
✅ Expr (expression statements)
✅ Block (code blocks)
✅ If (if statements)
✅ Fn (function definitions)
✅ Store (variable declarations let x = ...)
✅ Return (return statements)
```

**Missing** (15+ variants):
```rust
❌ For (for loops)
❌ Is (pattern matching is statements)
❌ EnumDecl (enum declarations)
❌ TypeDecl (type declarations)
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
❌ Break (break statements)
❌ Ext (type extensions impl)
```

**Impact**: ~65% of statement types not implemented

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
| float, double | ✅ | ❌ | 15% | P1 |
| uint, i8, u8, i64 | ✅ | ❌ | 10% | P1 |
| char, cstr | ✅ | ❌ | 7% | P2 |
| **Expressions** | | | | |
| Arithmetic/Compare/Logical | ✅ | ✅ (partial) | 5% | P1 |
| Bitwise | ✅ | ❌ | 3% | P2 |
| Array indexing | ✅ | ❌ | 8% | P1 |
| Object/Node | ✅ | ❌ | 10% | P1 |
| F-strings | ✅ | ❌ | 5% | P2 |
| **Statements** | | | | |
| if/else, block | ✅ | ✅ | - | - |
| Function def/call | ✅ | ✅ | - | - |
| for loops | ✅ | ❌ | 12% | P1 |
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

**Overall Gap**: ~40-50% features still missing (updated from 50-60% due to closure completion)

---

## Technical Debt

### 1. Type System Completeness (P1)
**Problem**: BigVM only supports basic types (int, bool, str)
**Missing Types**:
- Floating-point: float, double (15% of evaluator tests)
- Integer variants: uint, i8, u8, i64 (10%)
- Characters: char (5%)
- C strings: cstr (2%)

**Estimated Effort**: 3-5 days

---

### 2. Expression Coverage (P1)
**Problem**: Missing 30+ expression types
**High Priority**:
- Index (array indexing `arr[i]`) - 8% impact
- Object (object literals) - 10% impact
- Range (ranges `0..10`) - 5% impact
- FStr (f-strings) - 5% impact

**Estimated Effort**: 5-7 days

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
- **Progress**: ~50-60% complete (updated from 40-50%)
- **Major Blocker Resolved**: Closures (Phase 7.1) NOW COMPLETE via Plan 071
- **Estimated Remaining Work**: 8-15 weeks

### Key Milestones
1. **Short-term** (1-2 months): Reach 70% feature parity
   - Complete type system expansion
   - Complete basic expressions/statements
   - Complete control flow

2. **Medium-term** (3-4 months): Reach 90% feature parity
   - Complete May/Question system
   - Complete advanced collections
   - Most tests passing

3. **Long-term** (5-6 months): 100% replacement
   - Complete borrowing system (optional)
   - Performance optimization
   - Production environment switch

### Priority Recommendations

**P0 (Immediate)**:
- ~~Closure implementation (Phase 7.1)~~ ✅ **COMPLETE** (Plan 071)
- List tests migration (NOW POSSIBLE with closures)

**P1 (High Priority)**:
- Type system expansion (float, double, uint, etc.)
- Array indexing, object literals
- For loops, pattern matching
- May/Question system

**P2 (Medium Priority)**:
- Bitwise operators
- F-strings
- Advanced collections

**P3 (Low Priority)**:
- Borrowing system (defer to future version)
- Performance optimization

### Next Steps
1. **Immediate**: Migrate list_tests.rs (now possible with Plan 071 closures)
2. **Parallel**: Start type system expansion (float, double)
3. **Planning**: Create detailed tickets for each missing feature

---

**Document Updated**: 2025-02-04
**Related Documents**:
- [Plan 068: AutoVM (BigVM) Implementation](068-autovm-bigvm.md)
- [Plan 070: BigVM Iterator](070-bigvm-iterator.md)
- [Plan 071: BigVM Closures](071-bigvm-closures.md)
- [Plan 064: Split Universe](064-split-universe.md)
- [Status Report](../status-bigvm-migration.md)
