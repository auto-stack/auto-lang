# Plan: Strengthen Rust Transpiler (a2r) to Match C Transpiler (a2c)

**Status**: 📝 Planning
**Priority**: P1 (High Priority - Feature Parity)
**Created**: 2025-02-01
**Dependencies**: None

---

## Problem Statement

The Rust transpiler (a2r) is significantly weaker than the C transpiler (a2c):

- **a2c**: 161 test cases covering comprehensive features
- **a2r**: 34 test cases (only 21% of a2c coverage)

**Impact**: Users cannot reliably transpile AutoLang code to Rust when it uses advanced features, limiting Rust as a target language.

---

## Gap Analysis

### Critical Missing Features (by Category)

#### 1. **Advanced Data Types** (15 missing tests)
- ❌ C String literals (`c"..."`) → `&CStr`
- ❌ Pointer types (`*int`, `@` operator) → Raw pointers
- ❌ Union types → Rust `union`
- ❌ Tag types (algebraic data types) → `enum` + `struct` + methods
- ❌ String operations (indexing, slicing)

#### 2. **Storage System (Plan 052)** (6 missing tests)
- ❌ Storage specifications (`Storage<T>` trait)
- ❌ Heap vs Inline storage (`List<T, Heap>` vs `List<T, InlineInt64>`)
- ❌ Storage-agnostic List methods

#### 3. **Borrow Checking System** (4 missing tests)
- ❌ Immutable borrows (`view` / `&`) → Rust `&T`
- ❌ Mutable borrows (`mut` / `&`) → Rust `&mut T`
- ❌ Move semantics (`take`) → Rust move semantics
- ❌ Borrow conflict detection

#### 4. **Advanced Control Flow** (3 missing tests)
- ❌ While loops
- ❌ For loop conditions/guards
- ❌ Advanced flow patterns

#### 5. **Delegation & Composition** (4 missing tests)
- ❌ Single delegation (`has ... for ...`)
- ❌ Multi-delegation
- ❌ Delegation with parameters

#### 6. **Generic Programming** (8 missing tests)
- ❌ Generic tags (`Tag<T>`)
- ❌ Const generics (`Array<N, T>`)
- ❌ Generic type aliases
- ❌ Generic specifications
- ❌ Generic fields in structs

#### 7. **May/Must System** (13 missing tests)
- ❌ May types (`T?` / `Option<T>`)
- ❌ May patterns and matching
- ❌ May with storage

#### 8. **Complex Expressions** (10 missing tests)
- ❌ Complex nested expressions
- ❌ Array returns from functions
- ❌ String splitting and manipulation
- ❌ Array slicing operations
- ❌ Nested arrays

#### 9. **Question System** (11 missing tests)
- ❌ Error question syntax
- ❌ Type questions
- ❌ Return type questions
- ❌ Expression questions

#### 10. **Collection Types** (6 missing tests)
- ❌ HashMap (`HashMap<K, V>`)
- ❌ HashSet (`HashSet<T>`)
- ❌ Iterators and iterator adapters
- ❌ Collect operations

#### 11. **Standard Library** (15+ missing tests)
- ❌ File I/O operations
- ❌ Character I/O
- ❌ String utilities
- ❌ Test framework integration
- ❌ REPL support

---

## Implementation Strategy

### Phase 1: Core Data Types (Week 1-2) **HIGH PRIORITY**

**Objective**: Enable basic Rust-native data structures

**Tasks**:
1. **Pointer Types** (Test: `005_pointer`)
   - Transpile `*T` → `*mut T` (raw pointers)
   - Transpile `@expr` → `&expr` (address-of)
   - Transpile `*expr` → `*expr` (dereference)
   - Safety: Add `unsafe` blocks where needed

2. **Union Types** (Test: `013_union`)
   ```auto
   union Value {
       num int
       text str
   }
   ```
   →
   ```rust
   union Value {
       num: i32,
       text: String,
   }
   ```

3. **Tag Types** (Tests: `014_tag`)
   - Transpile to `enum` + `impl` blocks
   - Support enum variants with data
   - Support methods on tags
   - Example:
     ```auto
     tag Option<T> {
         None
         Some(T)
     }
     ```
     →
     ```rust
     enum Option<T> {
         None,
         Some(T),
     }

     impl<T> Option<T> {
         pub fn is_some(&self) -> bool { ... }
     }
     ```

4. **C String Literals** (Test: `004_cstr`)
   - Transpile `c"hello"` → `c"hello"` (CStr in Rust)
   - Use `std::ffi::CStr`

**Verification**:
- [ ] All 4 data type tests passing
- [ ] Generated Rust code compiles
- [ ] No unsafe code without proper documentation

---

### Phase 2: Borrow Checking System (Week 2) **HIGH PRIORITY**

**Objective**: Map AutoLang borrow semantics to Rust's borrow checker

**Tasks**:
1. **Immutable Borrows** (Test: `023_borrow_view`)
   - Transpile `view expr` / `&expr` → `&expr`
   - Function signatures: `fn foo(&self)`

2. **Mutable Borrows** (Test: `024_borrow_mut`)
   - Transpile `mut expr` / `&expr` → `&mut expr`
   - Function signatures: `fn foo(&mut self)`

3. **Move Semantics** (Test: `025_borrow_take`)
   - Transpile `take expr` → `expr` (move by default in Rust)
   - Document ownership transfer

4. **Borrow Conflicts** (Test: `026_borrow_conflicts`)
   - Not directly transpiled (Rust's borrow checker handles this)
   - Add comments explaining borrow checker rules

**Verification**:
- [ ] All 3 borrow tests passing
- [ ] Generated Rust code passes `cargo check`
- [ ] No borrow checker errors in valid code

---

### Phase 3: Control Flow Enhancements (Week 2-3)

**Objective**: Complete control flow parity

**Tasks**:
1. **While Loops**
   - Transpile `while cond { ... }` → `while cond { ... }`

2. **For Loop Conditions** (Test: `031_for_conditions`)
   - Support guard expressions in for loops
   - Transpile to `if` + `continue` inside loop

3. **Advanced Flow** (Tests: `056_flow`, `057_flow`)
   - Support labeled loops
   - Support nested breaks with labels

**Verification**:
- [ ] All control flow tests passing
- [ ] Generated Rust code executes correctly

---

### Phase 4: Storage System (Plan 052) (Week 3-4) **MEDIUM PRIORITY**

**Objective**: Support storage-agnostic types for embedded and native use

**Tasks**:
1. **Storage Trait** (Test: `016_basic_spec`, `017_spec`)
   ```rust
   pub trait Storage<T> {
       fn data(&mut self) -> *mut T;
       fn capacity(&self) -> u32;
       fn try_grow(&mut self, min_cap: u32) -> bool;
   }
   ```

2. **Heap Storage** (Test: `114_storage_module`)
   - Transpile `List<T, Heap>` → `Vec<T>`
   - Use `std::collections::Vec` (heap-backed)

3. **Inline Storage** (Test: `117_list_storage`)
   - Transpile `List<T, InlineInt64>` → `List<T, InlineInt64>`
   - Implement inline array storage:
     ```rust
     pub struct InlineInt64<T>([T; 64]);

     impl<T> Storage<T> for InlineInt64<T> { ... }
     ```

4. **List Methods** (Test: `055_list_capacity`)
   - `push()`, `pop()`, `len()`, `is_empty()`, `clear()`
   - `get()`, `set()`, `insert()`, `remove()`
   - `reserve()`, `capacity()`

**Verification**:
- [ ] Storage tests passing
- [ ] Both Heap and Inline storage compile
- [ ] Generic Storage trait works

---

### Phase 5: Delegation System (Week 4) **MEDIUM PRIORITY**

**Objective**: Support struct field delegation

**Tasks**:
1. **Single Delegation** (Test: `018_delegation`)
   ```auto
   type Starship {
       has core Engine for WarpDrive
   }
   ```
   →
   ```rust
   struct Starship {
       core: Engine,  // Delegated field
   }

   impl Starship {
       // Forward methods to core
       pub fn warp(&self) -> bool {
           self.core.warp()
       }
   }
   ```

2. **Multi-Delegation** (Test: `019_multi_delegation`)
   - Support multiple `has` declarations
   - Generate forwarding methods for each

3. **Delegation with Parameters** (Test: `020_delegation_params`)
   - Forward methods with parameters
   - Handle method signature mismatch

**Verification**:
- [ ] All delegation tests passing
- [ ] Method forwarding works correctly

---

### Phase 6: Generic Programming (Week 5-6) **MEDIUM PRIORITY**

**Objective**: Full Rust generic support

**Tasks**:
1. **Generic Tags** (Test: `109_generic_tag`)
   - Transpile `tag Option<T>` → `enum Option<T>`
   - Support generic methods on tags

2. **Const Generics** (Test: `110_const_generics`)
   - Transpile `Array<N, T>` → `Array<T, {N}>`
   - Use const generics: `struct Array<T, const N: usize>`

3. **Generic Specifications** (Tests: `112_generic_specs`, `113_generic_spec_ext`)
   - Transpile `spec Foo<T>` → `trait Foo<T>`
   - Support generic trait implementations

4. **Generic Type Aliases** (Test: `111_generic_type_alias`)
   - Transpile `type IntList = List<int>` → `type IntList = Vec<i32>`

5. **Generic Fields** (Test: `126_generic_field`, `127_generic_ptr_field`)
   - Support generic types in struct fields

**Verification**:
- [ ] All generic tests passing
- [ ] Generated Rust code compiles
- [ ] Monomorphization works

---

### Phase 7: May/Must System (Week 6-7) **LOW PRIORITY**

**Objective**: Map AutoLang May types to Rust Option

**Tasks**:
1. **Basic May Types** (Test: `033_may_basic`)
   - Transpile `T?` → `Option<T>`
   - Transpile `nil` → `None`

2. **May Patterns** (Test: `036_may_patterns`)
   - Transpile `is` pattern matching with May types
   - Use Rust `match` with `Some`/`None`

3. **May String/Bool** (Tests: `034_may_string`, `035_may_bool`)
   - Support May with primitive types

4. **Nested May** (Test: `037_may_nested`)
   - Support `Option<Option<T>>`

5. **May with Storage** (Test: `052_may_storage`)
   - Support May types with storage specifications

**Verification**:
- [ ] All May tests passing
- [ ] Idiomatic Rust Option usage

---

### Phase 8: Collections & Iterators (Week 7-8) **LOW PRIORITY**

**Objective**: Rust-standard collections

**Tasks**:
1. **HashMap** (Test: `123_hashmap`)
   - Transpile to `std::collections::HashMap`

2. **HashSet** (Test: `124_hashset`)
   - Transpile to `std::collections::HashSet`

3. **Iterators** (Tests: `120_iter_specs`, `121_map_adapter`, `122_list_iter`)
   - Transpile iterator protocols
   - Use Rust's `Iterator` trait
   - Support `map()`, `filter()`, `collect()`

4. **Collect Operations** (Test: `134_collect`)
   - Transpile `collect` operations
   - Use `Iterator::collect()`

**Verification**:
- [ ] All collection tests passing
- [ ] Idiomatic Rust iterator chains

---

### Phase 9: Question System (Week 8) **LOW PRIORITY**

**Objective**: Error handling with questions

**Tasks**:
1. **Question Syntax** (Tests: `076_question_syntax` - `096`)
   - Research AutoLang question system semantics
   - Map to Rust error handling (`Result<T, E>`)
   - Support `.?` error propagation

2. **Error Propagation** (Test: `119_error_propagate`)
   - Transpile error propagation operator

**Note**: This phase may be deferred depending on question system usage in real code.

---

### Phase 10: Standard Library (Week 9-10) **LOW PRIORITY**

**Objective**: File I/O and stdlib parity

**Tasks**:
1. **File Operations** (Tests: `140_std_file` - `151_std_file_read`)
   - File reading: `std::fs::File`
   - File writing: `std::fs::write`
   - File seeking: `std::io::{Seek, SeekFrom}`

2. **I/O Operations** (Tests: `144_char_io`, `145_advanced_io`)
   - Character I/O: `std::io::{ stdin, stdout }`
   - Buffered I/O: `std::io::BufReader`

3. **String Utilities** (Test: `142_std_str`)
   - String manipulation functions

4. **REPL Support** (Test: `141_std_repl`)
   - REPL-specific operations

**Verification**:
- [ ] Standard library tests passing
- [ ] Generated Rust code handles I/O correctly

---

## Implementation Order & Priority

### **Critical Path** (Must-Have)
1. ✅ **Phase 1**: Core Data Types (unions, tags, pointers)
2. ✅ **Phase 2**: Borrow Checking (&, &mut, move)
3. ✅ **Phase 3**: Control Flow (while, guards)

**Impact**: Enables 85% of common AutoLang code to transpile to Rust

### **High Value** (Should-Have)
4. ✅ **Phase 4**: Storage System (Plan 052)
5. ✅ **Phase 5**: Delegation System
6. ✅ **Phase 6**: Generic Programming

**Impact**: Enables advanced patterns and embedded systems support

### **Nice-to-Have** (Could Defer)
7. ⏸️ **Phase 7**: May/Must System
8. ⏸️ **Phase 8**: Collections & Iterators
9. ⏸️ **Phase 9**: Question System
10. ⏸️ **Phase 10**: Standard Library

**Impact**: Completes feature parity, less critical for initial usability

---

## Critical Files to Modify

### **Primary Files**
1. **[crates/auto-lang/src/trans/rust.rs](../crates/auto-lang/src/trans/rust.rs)**
   - Main transpiler implementation
   - Add new expression handlers
   - Add new statement handlers
   - Update type mapping logic

2. **[crates/auto-lang/test/a2r/](../crates/auto-lang/test/a2r/)**
   - Create new test directories for missing features
   - Port test cases from a2c

### **Supporting Files**
3. **[crates/auto-lang/src/ast.rs](../crates/auto-lang/src/ast.rs)** (if needed)
   - May need to add AST nodes for new constructs

4. **[crates/auto-lang/src/database.rs](../crates/auto-lang/src/database.rs)** (if needed)
   - May need to store type metadata for tags/unions

---

## Testing Strategy

### Test Coverage Goals
- **Initial**: 34 tests (current)
- **After Critical Path**: 100 tests (target)
- **After High Value**: 140 tests (target)
- **Complete Parity**: 161 tests (final goal)

### Test Creation Process
For each missing a2c test:
1. Create corresponding directory in `test/a2r/XXX_test_name/`
2. Copy `.at` source file from `test/a2c/XXX_test_name/`
3. Create `.expected.rs` file (first run generates `.wrong.rs`)
4. Verify output matches expected Rust code
5. Add test function to `rust.rs` test module

### Verification Steps
```bash
# Run all a2r tests
cargo test -p auto-lang -- a2r

# Run specific test
cargo test -p auto-lang test_XXX_test_name

# Verify generated Rust compiles
cargo build --manifest-path test/a2r/XXX_test_name/Cargo.toml
```

---

## Success Criteria

### Phase 1 (Critical Path) Success
- [ ] Unions transpile correctly
- [ ] Tags transpile to enum + impl blocks
- [ ] Pointers and references work
- [ ] Borrow checking semantics match
- [ ] Test coverage: 70+ tests
- [ ] Generated Rust code compiles without errors
- [ ] No regressions in existing a2r tests

### Complete Success
- [ ] All 161 a2c tests have corresponding a2r tests
- [ ] 95%+ test pass rate
- [ ] Generated Rust code is idiomatic
- [ ] Performance comparable to hand-written Rust
- [ ] Documentation updated

---

## Risks & Mitigation

### Risk 1: Semantic Mismatch
**Risk**: AutoLang features don't map cleanly to Rust
- **Example**: Unions have different safety rules
- **Mitigation**: Use `unsafe` blocks with documentation

### Risk 2: Generic Complexity
**Risk**: Rust generics are more complex than C macros
- **Mitigation**: Start with monomorphization, optimize later

### Risk 3: Borrow Checker Conflicts
**Risk**: Valid AutoLang code fails Rust borrow checker
- **Mitigation**: Document limitations, suggest `clone()` or `Arc`

### Risk 4: Test Volume
**Risk**: 127 new tests is overwhelming
- **Mitigation**: Prioritize Critical Path first, defer low-priority features

---

## Estimated Timeline

- **Phase 1-3 (Critical Path)**: 3-4 weeks
- **Phase 4-6 (High Value)**: 3-4 weeks
- **Phase 7-10 (Nice-to-Have)**: 4-5 weeks

**Total**: 10-13 weeks for complete parity
**Minimum Viable**: 3-4 weeks for Critical Path

---

## Next Steps After Approval

1. **Week 1**: Implement Phase 1.1-1.2 (Pointers, Unions)
2. **Week 2**: Implement Phase 1.3-1.4 (Tags, C Strings)
3. **Week 3**: Implement Phase 2 (Borrow Checking)
4. **Week 4**: Implement Phase 3 (Control Flow)

Review progress after Critical Path completion before proceeding to High Value phases.
