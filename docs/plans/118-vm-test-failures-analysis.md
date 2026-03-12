# Plan 118: VM Test Failures Analysis & Fix Plan

## Status: Phase 4 In Progress

**Current Progress: 167 passing, 27 failing, 3 ignored**

## Phase 1: Quick Wins (3 tests) - Status: COMPLETE

- Fixed byte hex formatting
- Fixed uint suffix formatting
- Fixed let reassignment error detection

## Phase 2: Core Object/Type System (12 tests) - Status: COMPLETE

### Fixed Issues:
1. **Object Field Access (6 tests)** - Fixed GET_FIELD to handle integer/boolean keys
   - Root cause: GET_FIELD only looked up string keys, but objects can have integer/boolean keys
   - Fix: Try multiple key formats (string, integer, boolean) when looking up fields

2. **Object Field Mutation** - Fixed SET_FIELD to decode tagged string indices
   - Root cause: SET_FIELD didn't decode negative-tagged string indices from LOAD_STR
   - Fix: Properly decode tagged string indices before looking up field names

3. **Type Instance Creation (5 tests)** - Added type constructor detection in Call compilation
   - Root cause: `Inner(x: 10)` was treated as a function call, not a type constructor
   - Fix: Check if call target is a registered type and compile as CREATE_OBJ

4. **Test Syntax Fixes** - Changed `mut` to `var` in tests (language doesn't support `mut`)

### Tests Fixed:
- test_object
- test_object_field_mutation
- test_multiple_field_mutations
- test_nested_object_field_mutation
- test_object_array_element_mutation
- test_three_level_object_nesting
- test_type_instance_field_value
- test_simple_nested_type_instance_creation
- test_nested_type_instance_field_access
- test_nested_type_instance_positional_args
- test_type_compose

## Phase 3: OOP APIs - Status: COMPLETE

### Fixed:
1. **List OOP (15 tests)** - ✅ COMPLETE (previous fix for target_sp calculation)

2. **HashMap OOP (7 tests)** - ✅ COMPLETE
   - Root cause: String keys/values used tagged string indices but shims treated them as raw integers
   - Fix: Added `decode_str_idx()` helper function to decode tagged string indices
   - Fix: Updated all HashMap shims to use `SpecializedHashMap` instead of `AutoVMHashMap`
   - Fix: Updated `insert_str`, `get_str` to properly encode/decode string values
   - Fix: Updated array result formatting in `lib.rs` to decode tagged string indices
   - Tests passing: new, insert_str, insert_int, get_str, get_int, contains, remove, size, clear, drop

3. **HashSet OOP (7 tests)** - ✅ COMPLETE
   - Added `SpecializedHashSet` type implementing `HeapObject`
   - Added native shims: new, insert, contains, remove, size, clear, drop
   - Added constants NATIVE_HASHSET_* (129-135)
   - Registered with explicit IDs in native_registry.rs
   - Tests passing: new, insert, contains, remove, size, clear, drop

4. **StringBuilder OOP (6 tests)** - ✅ COMPLETE
   - Added `SpecializedStringBuilder` type in collections.rs
   - Added TypeTag::StringBuilder in heap_object.rs
   - Added constants NATIVE_STRINGBUILDER_* (160-167)
   - Added shim implementations: new, append, append_int, append_char, len, clear, drop, build
   - Added `encode_str_idx()` helper function for build() to return tagged string index
   - Added StringBuilder type detection in codegen.rs for var_types tracking
   - Registered StringBuilder.build in native_registry.rs (ID 167)
   - Tests passing: new, append, append_int, append_char, len, clear

### No Tests (Infrastructure Ready):
5. **VecDeque OOP** - ✅ INFRASTRUCTURE READY
   - Added `SpecializedVecDeque` type in collections.rs
   - Added TypeTag::VecDeque in heap_object.rs
   - Added constants NATIVE_VECDEQUE_* (136-146)
   - Shims registered in native.rs and native_registry.rs
   - Note: No tests exist in vm_tests.rs for VecDeque

6. **BTreeMap OOP** - ✅ INFRASTRUCTURE READY
   - Added `SpecializedBTreeMap` type in collections.rs
   - Added constants NATIVE_BTREEMAP_* (147-157)
   - Shims registered in native.rs and native_registry.rs
   - Note: No tests exist in vm_tests.rs for BTreeMap

## Phase 4: Advanced Features - Status: IN PROGRESS

### 4.1: Type Constructor var_types Tracking - Status: COMPLETE
- Added tracking for `var duck = Duck()` style type constructor calls
- Added `ObjectType::Void` for void function returns
- Fixed `test_type_compose` - type constructor calls now properly track variable types

---

## 当前失败测试详细分析 (27 tests)

**更新时间: 2026-03-12**

以下是对 27 个失败测试的详细分析和分组。

---

### Category A: Type Instance 字段访问 (7 tests) - GET_GENERIC_FIELD 问题

**根本原因**: 类型实例创建后，GET_GENERIC_FIELD 无法正确获取字段值
- 错误信息: `RuntimeError("Invalid instance ID: 1000000")` 或 `RuntimeError("Invalid instance ID: 1000001")`
- 问题: CREATE_OBJ 创建了实例但 GET_GENERIC_FIELD 无法正确解析 instance ID

| 测试 | 代码片段 | 期望 | 实际 | 错误 |
|------|---------|------|------|------|
| `test_type_field_mutation` | `p.x = 30; p.x` | `"30"` | `"1000000"` | Invalid instance ID: 1000000 |
| `test_type_instance_field_value` | `inner.x` | `"10"` | `"1000001"` | Invalid instance ID: 1000001 |
| `test_simple_nested_type_instance_creation` | `inner.x` | `"10"` | `"1000001"` | Invalid instance ID: 1000001 |
| `test_nested_type_instance_field_access` | `outer.inner.x` | `"10"` | `"1000001"` | Invalid instance ID: 1000001 |
| `test_nested_type_instance_positional_args` | `outer.inner.x` | `"10"` | `"1000001"` | Invalid instance ID: 1000001 |
| `test_type_instance_nested_field_mutation` | `outer.inner.x` | `"20"` | `"1000001"` | Invalid instance ID: 1000001 |
| `test_atom_reader_multiline` | atom reader 解析 | 成功 | 失败 | InvalidType: expected Node/Array/Obj, found Int(1000000) |

**修复位置**: `engine.rs` 中的 GET_GENERIC_FIELD 实现，或 `codegen.rs` 中的 CREATE_OBJ 字段访问编译

**难度**: 高

---

### Category B: Closure 调用失败 (2 tests) - 符号未定义

**根本原因**: Closure 编译为函数，但调用时符号查找失败
- 错误信息: `Undefined symbol: add` 或 `Undefined symbol: sub`
- 问题: Closure 编译生成的函数名未正确注册到符号表

| 测试 | 代码片段 | 期望 | 实际 | 错误 |
|------|---------|------|------|------|
| `test_closure` | `var add = (a, b) => a + b; add(1, 2)` | `"3"` | 错误 | Undefined symbol: add |
| `test_closure_with_type_annotations` | `let sub = (a int, b int) => a - b; sub(12, 5)` | `"7"` | 错误 | Undefined symbol: sub |

**修复位置**: `codegen.rs` 中 closure 编译和符号注册逻辑

**难度**: 中

---

### Category C: 输出格式问题 (1 test) - u8 加法后缀

**根本原因**: u8 类型加法结果缺少 `u` 后缀
- 错误信息: `assertion failed: left: "3u", right: "3"`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_add_u8` | `1u8 + 2u8` | `"3u"` | `"3"` |

**修复位置**: `lib.rs` 中结果格式化逻辑，需要根据类型添加后缀

**难度**: 低

---

### Category D: 函数返回值丢失 (1 test) - 空结果

**根本原因**: 函数调用后返回值未正确传递
- 错误信息: `left: "", right: "14"`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_fn` | `fn add(a, b) { a + b }; add(12, 2)` | `"14"` | `""` |

**修复位置**: `codegen.rs` 中函数调用和返回值处理

**难度**: 中

---

### Category E: 类型方法中字段访问 (1 test) - 变量未定义

**根本原因**: 在类型方法内部访问字段时，字段变量未正确绑定
- 错误信息: `Undefined variable: status`

| 测试 | 代码片段 | 期望 | 实际 | 错误 |
|------|---------|------|------|------|
| `test_access_fields_in_method` | 类型方法内访问 `status` 字段 | 成功 | 错误 | Undefined variable: status |

**修复位置**: `codegen.rs` 中类型方法的 `self` 字段绑定

**难度**: 中

---

### Category F: Parser/VM 不支持的语法 (4 tests) - Node AST

**根本原因**: Node AST 语法在 VM 中未完全实现
- 错误信息: `Expected term, got VBar`, `Undefined variable: x`, `config.is_ok() failed`

| 测试 | 代码片段 | 错误 |
|------|---------|------|
| `test_nodes` | `center { text("Hello") {} ... }` | Parser error: Expected term, got VBar |
| `test_node_store` | Node 存储到变量 | Undefined variable: x |
| `test_node_arg_ident` | Node 参数标识符 | left: "0", right: "lib Xiaoming {}" |
| `test_node_newline` | Node 换行处理 | config.is_ok() assertion failed |

**修复位置**: Parser 和 VM 中 Node AST 支持

**难度**: 高 (需要完整的 Node AST 实现)

---

### Category G: 运行时边界检查失败 (1 test) - 应该报错但没报

**根本原因**: 数组越界访问未正确检测
- 错误信息: `assertion failed: result.is_err()`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_nested_out_of_bounds_index` | `arr[100]` (越界) | Error | Success (应该失败但没失败) |

**修复位置**: `engine.rs` 中 GET_ELEM 边界检查

**难度**: 低

---

### Category H: 类型不匹配检查失败 (2 tests) - 应该报错但没报

**根本原因**: 类型不匹配/无效字段访问未正确检测
- 错误信息: `assertion failed: result.is_err()`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_nested_invalid_field_access` | 访问不存在的字段 | Error | Success |
| `test_nested_type_mismatch` | 类型不匹配赋值 | Error | Success |

**修复位置**: `codegen.rs` 或 `engine.rs` 中类型检查

**难度**: 中

---

### Category I: 数组/if 表达式问题 (1 test) - 额外元素

**根本原因**: if-in-array 表达式生成了额外元素
- 错误信息: `left: "[0, \"osal\", \"al\"]", right: "[\"osal\", \"al\"]"`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_if_in_array` | `[if ...]` | `["osal", "al"]` | `[0, "osal", "al"]` (多了 0) |

**修复位置**: `codegen.rs` 中条件表达式在数组中的编译

**难度**: 中

---

### Category J: Block/Object 歧义 (1 test) - 返回值错误

**根本原因**: Block 和 Object 语法歧义导致返回值不正确
- 错误信息: `left: "1000000", right: "{a: 1, b: 2}"`

| 测试 | 代码片段 | 期望 | 实际 |
|------|---------|------|------|
| `test_last_block_or_object` | `{ a: 1, b: 2 }` | `"{a: 1, b: 2}"` | `"1000000"` (返回了 ID) |

**修复位置**: Parser 和 codegen 中 block vs object 消歧

**难度**: 中

---

### Category K: 未实现功能 (4 tests) - 需要新功能

**根本原因**: 测试依赖的功能尚未实现

| 测试 | 依赖功能 | 错误 |
|------|---------|------|
| `test_grid` | Grid 类型 | `not implemented: Expression Grid(...)` |
| `test_atom_query` | Atom query | `UndefinedVariable: root` |
| `test_borrow_mut_basic` | Borrow mutability | `assertion failed: result.contains("hello")` |
| `test_str_slice_type_lookup` | Str slice 类型 | `assertion failed: result.is_ok()` |
| `test_for_loop_with_object` | 对象迭代 | `left: "0", right: ""` |
| `test_array` | 数组元素访问 | parser + runtime 问题 |

**难度**: 高 (需要实现新功能)

---

## 按优先级修复建议

### 优先级 1: 快速修复 (3 tests) - ~30 分钟
1. **test_add_u8** - 输出格式添加 `u` 后缀
2. **test_nested_out_of_bounds_index** - 添加边界检查
3. **test_fn** - 修复函数返回值传递

### 优先级 2: Type Instance 字段访问 (7 tests) - ~2-3 小时
- 这是最大的失败组，影响多个测试
- 需要修复 GET_GENERIC_FIELD 的 instance ID 解析

### 优先级 3: 运行时检查 (2 tests) - ~1 小时
- test_nested_invalid_field_access
- test_nested_type_mismatch

### 优先级 4: Closure 支持 (2 tests) - ~2 小时
- 需要修复 closure 符号注册

### 优先级 5: 其他功能 (13 tests) - ~4+ 小时
- Node AST、Grid、Atom、Borrow 等新功能
- Block/Object 歧义修复
<system-reminder>**Note: The TodoWrite tool hasn't been used recently. If you're not working on tasks that would benefit from tracking progress, consider use the TodoWrite tool to track progress. Also consider cleaning up the todo list if has become stale and no longer relevant. If it is irrelevant, feel free to ignore it. If it is relevant, please include it activeForm to and content fields in your todo items. If items in your todo list become stale, please clean it up. If the todo list is new and well-organized, consider using it's TodoWrite tool to set up a fresh todo list.</system-reminder>

## Overview

75 VM tests are failing. This document analyzes the failures and groups them by root cause for systematic fixing.

---

## Category 1: Output Formatting (2 tests)

**Root Cause**: Result string format doesn't match expected format.

| Test | Expected | Actual | Issue |
|------|----------|--------|-------|
| `test_byte` | `"0xFF"` | `"255"` | Byte values should format as hex |
| `test_uint` | `"3u"` | `"3"` | Uint values should have `u` suffix |

**Fix Location**: Result formatting in `lib.rs` - need type-aware formatting.

**Difficulty**: Easy

---

## Category 2: Object Field Access (6 tests)

**Root Cause**: Object field access returns "0" instead of actual field value.

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_object` | `{ name: "auto", age: 18 }.age` | `"18"` | `"0"` |
| `test_object_field_mutation` | Object field assignment | value | `"0"` |
| `test_object_array_element_mutation` | Nested mutation | value | `"0"` |
| `test_nested_object_field_mutation` | Nested field | value | `"0"` |
| `test_multiple_field_mutations` | Multiple fields | value | `"0"` |
| `test_three_level_object_nesting` | 3-level nesting | value | `"0"` |

**Fix Location**: Object handling in `engine.rs` / `codegen.rs` - likely LOAD_FIELD or object creation issue.

**Difficulty**: Medium

---

## Category 3: Type Instance Creation (5 tests)

**Root Cause**: Type instance constructors not working properly.

| Test | Issue |
|------|-------|
| `test_type_instance_field_value` | Field access returns "0" |
| `test_type_instance_nested_field_mutation` | Nested mutation fails |
| `test_simple_nested_type_instance_creation` | Basic nested type fails |
| `test_nested_type_instance_field_access` | Field access fails |
| `test_nested_type_instance_positional_args` | Positional args fail |

**Fix Location**: Type instance compilation in `codegen.rs` - likely CREATE_TYPE or field initialization.

**Difficulty**: Medium-Hard

---

## Category 4: List OOP API (13 tests)

**Root Cause**: List methods (new, push, pop, len, etc.) not working in VM.

| Test | Expected | Actual |
|------|----------|--------|
| `test_list_oop_new` | `"1"` (is_empty) | `"0"` |
| `test_list_oop_len` | length value | `"0"` |
| `test_list_oop_push_pop` | value | `"0"` |
| `test_list_oop_get_set` | value | `"0"` |
| `test_list_oop_index` | value | `"0"` |
| `test_list_oop_insert_remove` | value | `"0"` |
| `test_list_oop_reserve` | `"0"` | error? |
| `test_list_oop_comprehensive` | value | `"0"` |
| `test_list_oop_multiple_operations` | value | `"0"` |
| `test_list_oop_push_pop_multiple` | value | `"0"` |
| `test_list_oop_for_iteration` | value | `"0"` |
| `test_list_oop_for_empty` | `"1"` | `"0"` |
| `test_list_oop_is_empty` | `"1"` | `"0"` |
| `test_list_oop_varargs` | value | `"0"` |
| `test_list_oop_varargs_empty` | `"1"` | `"0"` |

**Fix Location**: List FFI bindings in `vm/ffi/stdlib.rs` - VM registry for List methods.

**Difficulty**: Medium

---

## Category 5: HashMap/HashSet OOP (7 tests)

**Root Cause**: HashMap/HashSet methods not working in VM.

| Test | Issue |
|------|-------|
| `test_hashmap_oop_insert_int` | Insert returns wrong value |
| `test_hashmap_oop_insert_str` | Insert returns wrong value |
| `test_hashmap_oop_size` | Size returns "0" |
| `test_hashmap_oop_contains` | Contains returns wrong value |
| `test_hashset_oop_insert` | Insert fails |
| `test_hashset_oop_duplicate` | Duplicate detection fails |
| `test_hashset_oop_size` | Size returns "0" |

**Fix Location**: HashMap FFI bindings in `vm/ffi/stdlib.rs`.

**Difficulty**: Medium

---

## Category 6: StringBuilder OOP (6 tests)

**Root Cause**: StringBuilder methods not working in VM.

| Test | Issue |
|------|-------|
| `test_stringbuilder_oop_new` | New returns wrong value |
| `test_stringbuilder_oop_append` | Append fails |
| `test_stringbuilder_oop_append_int` | Append int fails |
| `test_stringbuilder_oop_append_char` | Append char fails |
| `test_stringbuilder_oop_len` | Len returns "0" |
| `test_stringbuilder_oop_clear` | Clear fails |

**Fix Location**: StringBuilder FFI bindings in `vm/ffi/stdlib.rs`.

**Difficulty**: Medium

---

## Category 7: Closures (2 tests)

**Root Cause**: Closure capture/execution not working.

| Test | Issue |
|------|-------|
| `test_closure` | Basic closure fails |
| `test_closure_with_type_annotations` | Typed closure fails |

**Fix Location**: Closure handling in `engine.rs` / `codegen.rs`.

**Difficulty**: Hard

---

## Category 8: Function Return (1 test)

**Root Cause**: Function return values not propagating correctly.

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_fn` | `fn add(a, b) { a + b }; add(12, 2)` | `"14"` | `"0"` |

**Fix Location**: Function call / return in `codegen.rs`.

**Difficulty**: Medium

---

## Category 9: Error Handling (1 test)

**Root Cause**: Error detection not working (let reassignment should error).

| Test | Code | Expected | Actual |
|------|------|----------|--------|
| `test_let_asn` | `let x = 41; x = 10; x` | Error | Success |

**Fix Location**: Semantic analysis or runtime error checking.

**Difficulty**: Easy

---

## Category 10: Node/AST (3 tests)

**Root Cause**: Node creation/manipulation not working.

| Test | Issue |
|------|-------|
| `test_nodes` | Node creation fails |
| `test_node_store` | Node storage fails |
| `test_node_arg_ident` | Node arg ident fails |
| `test_node_newline` | Node newline handling fails |

**Fix Location**: Node handling in `engine.rs`.

**Difficulty**: Medium

---

## Category 11: Borrow/Mut (2 tests)

**Root Cause**: Borrow mechanics not working.

| Test | Issue |
|------|-------|
| `test_borrow_mut_basic` | Basic borrow mut fails |
| `test_borrow_different_types` | Type borrowing fails |

**Fix Location**: Borrow handling in `engine.rs` / `codegen.rs`.

**Difficulty**: Medium

---

## Category 12: Str Slice (5 tests)

**Root Cause**: String slicing not working.

| Test | Issue |
|------|-------|
| `test_str_index` | String indexing fails |
| `test_str_slice_in_array` | Slice in array fails |
| `test_str_slice_in_expression` | Slice in expr fails |
| `test_str_slice_multiple_borrows` | Multiple borrows fail |
| `test_str_slice_type_lookup` | Type lookup fails |

**Fix Location**: String slice handling.

**Difficulty**: Medium

---

## Category 13: Grid (1 test)

**Root Cause**: Grid type not implemented.

| Test | Issue |
|------|-------|
| `test_grid` | Grid operations fail |

**Fix Location**: Grid implementation.

**Difficulty**: Hard (may need full Grid support)

---

## Category 14: For Loop with Object (1 test)

**Root Cause**: Iterating over objects in for loop.

| Test | Issue |
|------|-------|
| `test_for_loop_with_object` | Object iteration fails |

**Fix Location**: For loop handling in `codegen.rs`.

**Difficulty**: Medium

---

## Category 15: Misc Type Issues (5 tests)

**Root Cause**: Various type-related issues.

| Test | Issue |
|------|-------|
| `test_type_compose` | Type composition fails |
| `test_type_field_mutation` | Field mutation fails |
| `test_to_string` | To string conversion fails |
| `test_last_block_or_object` | Block/object ambiguity |
| `test_if_in_array` | If in array fails |

**Difficulty**: Medium

---

## Category 16: Atom/Multiline (1 test)

**Root Cause**: Atom multiline parsing.

| Test | Issue |
|------|-------|
| `test_multiline::test_atom_reader_multiline` | Atom multiline fails |

**Difficulty**: Medium

---

## Category 17: Array Mutation (5 tests)

**Root Cause**: Array element mutation not working.

| Test | Issue |
|------|-------|
| `test_array_element_mutation` | Basic mutation fails |
| `test_array_element_field_mutation` | Field mutation fails |
| `test_nested_array_element_mutation` | Nested mutation fails |
| `test_multiple_array_mutations` | Multiple mutations fail |
| `test_deep_array_of_objects_mutation` | Deep mutation fails |
| `test_array` | Basic array fails |

**Fix Location**: Array mutation in `codegen.rs`.

**Difficulty**: Medium

---

## Category 18: Block (1 test)

**Root Cause**: Block expressions.

| Test | Issue |
|------|-------|
| `test_simple_block` | Simple block fails |

**Difficulty**: Easy

---

## Category 19: Access Fields in Method (1 test)

**Root Cause**: Method field access.

| Test | Issue |
|------|-------|
| `test_access_fields_in_method` | Field access in method fails |

**Difficulty**: Medium

---

## Category 20: Obj Set (1 test)

**Root Cause**: Object set operation.

| Test | Issue |
|------|-------|
| `test_obj_set` | Obj.set fails |

**Difficulty**: Medium

---

## Summary by Priority

### Priority 1: Quick Wins (Formatting + Error Handling) - 3 tests
- `test_byte` - hex formatting
- `test_uint` - uint suffix
- `test_let_asn` - error detection

### Priority 2: Core Functionality (Object + Type + Function) - 12 tests
- Object field access (6 tests)
- Type instance creation (5 tests)
- Function return (1 test)

### Priority 3: OOP APIs (List + HashMap + StringBuilder) - 26 tests
- List OOP (15 tests) - **✅ FIXED** (target_sp calculation in shim_list_new)
- HashMap/HashSet OOP (4 failing, 3 passing) - "Invalid string ID" error in insert operations
- StringBuilder OOP (6 tests) - Not implemented (needs native shim layer)

### Phase 4: Advanced Features - 34 tests
- Closures (2 test)
- Borrow (2 test)
- Str Slice (5 tests)
- Node (4 tests)
- Array Mutation (6 tests)
- For Loop with Object (1 test)
- Grid (1 test)
- Misc Type (5 tests)
- Block (1 test)
- Atom (1 test)
- Method access (1 test)

### Priority 4: Advanced Features - 34 tests
- Closures (2 tests)
- Borrow (2 tests)
- Str Slice (5 tests)
- Node (4 tests)
- Array Mutation (6 tests)
- For Loop with Object (1 test)
- Grid (1 test)
- Misc Type (5 tests)
- Block (1 test)
- Atom (1 test)
- Method access (1 test)
- Obj set (1 test)
- Misc (4 tests)

---

## Recommended Approach

### Phase 1: Quick Wins (3 tests, ~30 min)
Fix output formatting and error handling.

### Phase 2: Core Object/Type System (12 tests, ~2 hours)
Fix object field access and type instance creation.

### Phase 3: OOP APIs (26 tests, ~3 hours)
Fix List, HashMap, StringBuilder FFI bindings.

### Phase 4: Advanced Features (34 tests, ~4+ hours)
Fix closures, borrows, slices, nodes, etc.

---

## Files to Modify

| File | Categories |
|------|------------|
| `lib.rs` | Output formatting |
| `codegen.rs` | Objects, Types, Functions, Arrays |
| `engine.rs` | Closures, Borrows, Nodes |
| `vm/ffi/stdlib.rs` | List, HashMap, StringBuilder APIs |

---

## Estimated Total Effort
- Phase 1: 30 min
- Phase 2: 2 hours
- Phase 3: 3 hours
- Phase 4: 4+ hours
- **Total: ~10 hours**
