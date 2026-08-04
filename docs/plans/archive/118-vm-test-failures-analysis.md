# Plan 118: VM Test Failures Analysis & Fix Plan

## Status: Phase 7 - Generic & Void Function Fixes Applied

**Progress: 183 passing, 11 failing, 3 ignored** (2026-03-13)

### Phase 7 Fixes Applied (2026-03-13)

#### Fix 9: Generic Field Access Panic (Category B - 4 tests) ✅
**Problem**: `test_generic_field_access_x` and related tests panicked with "index out of bounds: the len is 0 but the index is 0"
**Root Cause**: `Type::substitute()` panicked when `params` matched a type parameter but `args` was empty (when type arguments weren't provided)
**Fix**: Added bounds check in `Type::substitute()` - if `idx >= args.len()`, keep the type parameter as-is instead of panicking
**Files**: `crates/auto-lang/src/ast/types.rs`
**Tests Fixed**: test_generic_field_access_x, test_generic_field_access_y, test_generic_field_addition, test_generic_type_instantiation

#### Fix 10: Void Function Return Type Inference ✅
**Problem**: `test_fn` - void functions like `fn hi(s str) { print(s); }` returned "0" instead of ""
**Root Cause**: Parser defaults all functions without explicit return type to `Void`, but standalone functions with implicit returns (like `fn add(a, b) { a + b }`) need to be distinguished from truly void functions
**Fix**:
1. After compiling function body, check `last_expr_type` to detect if body has implicit return
2. If parser set return type to Void but `last_expr_type != ObjectType::Void`, update `fn_return_types` to `Type::Unknown`
3. At call site, check `fn_return_types` for all functions (not just type methods) to set `last_expr_type = Void`
**Files**: `crates/auto-lang/src/vm/codegen.rs`
**Tests Fixed**: test_fn (all 3 assertions now pass)

### Phase 6 Fixes Applied (2026-03-13)

#### Fix 8: If-In-Array Support (Category I - 2 tests) ✅
**Problem**: `test_if_in_array` returned `[0, "osal", "al"]` instead of `["osal", "al"]`
**Root Cause**: If expressions without else branch didn't push any value when condition was false, leaving garbage on stack. The JMP to end had offset=0 (jumping to same position) because no code was generated for the else branch.
**Fix**:
1. Modified if expression compilation to push nil marker (i32::MIN + 1 = -2147483647) when no else branch exists
2. Modified CREATE_ARRAY opcode to filter out nil markers from array elements
3. Nil marker is a special value that won't conflict with legitimate 0/false values
**Files**: `crates/auto-lang/src/vm/codegen.rs`, `crates/auto-lang/src/vm/engine.rs`
**Tests Fixed**: test_if_in_array, test_if_with_bool (partially)

### Phase 5 Fixes Applied (2026-03-13)

#### Fix 6: Type Instance Field Access (Category A - 7+ tests) ✅
**Problem**: GET_GENERIC_FIELD returned "Invalid instance ID" errors
**Root Cause**: Type instances were created with CREATE_OBJ (heap objects) instead of NEW_INSTANCE (generic instances)
**Fix**:
1. Changed type constructor compilation to use NEW_INSTANCE + CONSTRUCT_INSTANCE opcodes
2. Fixed CONSTRUCT_INSTANCE to use correct field order (reversed from stack)
3. Fixed SET_GENERIC_FIELD stack handling (pop instance_id first, then value)
4. Fixed opcode emission order (opcode first, then operands)
5. Added field_type method to ClassType for type inference
**Files**: `crates/auto-lang/src/vm/codegen.rs`, `crates/auto-lang/src/vm/engine.rs`, `crates/auto-lang/src/vm/generic_registry.rs`
**Tests Fixed**: test_type_field_mutation, test_type_instance_field_value, test_simple_nested_type_instance_creation, test_nested_type_instance_field_access, test_nested_type_instance_positional_args, test_type_instance_nested_field_mutation, test_type_compose

#### Fix 7: Void Function Return Type Detection ✅
**Problem**: `test_fn` and `test_type_compose` - void functions returned "0" instead of ""
**Root Cause**: Codegen didn't check fn_return_types for type methods to set last_expr_type
**Fix**:
1. Added check for void return type in fn_return_types after CALL instruction
2. Only applies to type methods (function names containing '.') to avoid false positives
**Files**: `crates/auto-lang/src/vm/codegen.rs`

---

## Quick Fixes Applied Earlier (2026-03-13)

### Fix 1: test_add_u8 ✅
**Problem**: `u8` type inference in `infer/expr.rs` incorrectly returned `Type::Uint` instead of `Type::Int`
**Root Cause**: Line 67: `Expr::U8(_) => Type::Uint` was wrong
**Fix**: Changed to `Expr::U8(_) => Type::Int` (U8 arithmetic returns plain int with no 'u' suffix)
**File**: `crates/auto-lang/src/infer/expr.rs`

### Fix 2: test_nested_out_of_bounds_index ✅
**Problem**: Array out-of-bounds assignment didn't return error
**Root Cause**: `SET_ELEM` in `engine.rs` silently failed on OOB instead of returning error
**Fix**:
1. Added `last_error: Option<String>` to `AutoTask` struct (`vm/task.rs`)
2. Modified `run_task_loop` to store errors in `task.last_error` (`vm/engine.rs`)
3. Modified `SET_ELEM` handler to return proper `VMError` on OOB (`vm/engine.rs`)
4. Modified `run()` to check `task.last_error` and return `AutoError::Msg` (`lib.rs`)
**Files**: `crates/auto-lang/src/vm/task.rs`, `crates/auto-lang/src/vm/engine.rs`, `crates/auto-lang/src/lib.rs`

### Fix 3: test_fn ✅
**Problem**: Void functions (like `hi(s str) { print(s); }`) returned "0" instead of ""
**Root Cause Chain**:
1. Parser defaults functions without explicit return type to `Type::Void`
2. Native `print` function was pushing `0` as return value
3. Codegen wasn't marking print as void, so run() didn't know to return empty string
**Fix**:
1. Modified native print shims to NOT push return value (they are truly void)
2. Added `last_expr_type = ObjectType::Void` for print* native calls in codegen
3. Removed incorrect `CONST_0` emission before RET (it was overwriting real return values)
**Files**: `crates/auto-lang/src/vm/native.rs`, `crates/auto-lang/src/vm/codegen.rs`

### Fix 4: test_nested_invalid_field_access ✅
**Problem**: Accessing non-existent field `obj.inner.nonexistent = 20` didn't return error
**Root Cause**: `GET_FIELD` and `SET_FIELD` silently pushed 0 or created new fields instead of returning errors
**Fix**:
1. Modified `GET_FIELD` to return `VMError` when field not found (`vm/engine.rs`)
2. Modified `SET_FIELD` to return `VMError` when setting non-existent field (`vm/engine.rs`)
**Files**: `crates/auto-lang/src/vm/engine.rs`

### Fix 5: test_nested_type_mismatch ✅
**Problem**: Array assignment `obj.items.invalid_field = 10` didn't return error
**Root Cause**: When accessing `obj.items` (array) and then trying to set field `invalid_field`, the invalid object ID wasn't properly detected
**Fix**: Same as Fix 4 - `SET_FIELD` now returns error for non-existent fields
**Files**: `crates/auto-lang/src/vm/engine.rs`

---

## Phase 1-3: Completed (Previous Work)

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

### Category C: 输出格式问题 (1 test) - ✅ FIXED

**根本原因**: `infer/expr.rs` 中 `Expr::U8` 被推断为 `Type::Uint` 而不是 `Type::Int`
- 测试期望 `1u8 + 2u8` 返回 `"3"` (无后缀)
- 实际返回 `"3u"` (有后缀)

| 测试 | 代码片段 | 期望 | 修复前 | 状态 |
|------|---------|------|--------|------|
| `test_add_u8` | `1u8 + 2u8` | `"3"` | `"3u"` | ✅ FIXED |

**修复**: 在 `infer/expr.rs:67` 将 `Expr::U8(_) => Type::Uint` 改为 `Expr::U8(_) => Type::Int`

---

### Category D: 函数返回值丢失 (1 test) - ✅ FIXED

**根本原因**: 链式问题
1. Native `print` 函数 push 了 `0` 作为返回值
2. Void 函数没有被正确标记，导致 `run()` 返回 `"0"` 而不是 `""`

| 测试 | 代码片段 | 期望 | 修复前 | 状态 |
|------|---------|------|--------|------|
| `test_fn` (第3个断言) | `fn hi(s str) { print(s); }; hi("hello")` | `""` | `"0"` | ✅ FIXED |

**修复**:
1. Native `print` shims 不再 push 返回值
2. Codegen 在调用 `print*` native 函数后设置 `last_expr_type = Void`

---

### Category G: 运行时边界检查失败 (1 test) - ✅ FIXED

**根本原因**: 数组越界访问时 `SET_ELEM` 没有返回错误

| 测试 | 代码片段 | 期望 | 修复前 | 状态 |
|------|---------|------|--------|------|
| `test_nested_out_of_bounds_index` | `obj.items[10] = 100` | Error | Success | ✅ FIXED |

**修复**: 在 `engine.rs` 的 `SET_ELEM` 处理中，越界时返回 `VMError::RuntimeError`

---

### Category H: 类型不匹配检查失败 (2 tests) - ✅ FIXED

**根本原因**: 访问/设置不存在字段时没有返回错误

| 测试 | 代码片段 | 期望 | 修复前 | 状态 |
|------|---------|------|--------|------|
| `test_nested_invalid_field_access` | `obj.inner.nonexistent = 20` | Error | Success | ✅ FIXED |
| `test_nested_type_mismatch` | `obj.items.invalid_field = 10` | Error | Success | ✅ FIXED |

**修复**: 在 `GET_FIELD` 和 `SET_FIELD` 中，当字段不存在时返回 `VMError::RuntimeError`

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
