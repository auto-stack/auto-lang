# Plan 039: VM Tests Migration to AutoVM Tests

## Context

用户要求将 `vm_tests.rs` 中的测试按复杂程度分类并移植到 `autovm_tests.rs`。用户强调要先确保基础测试通过，AutoVM 相对健壮后，再处理泛型等复杂问题。

**当前状态**：
- `autovm_tests.rs` 已有 13 个测试，其中 11 个通过（基础算术）
- `vm_tests.rs` 包含 127+ 个测试用例
- 需要按简单到复杂程度优先级移植

## 目标

**主要目标**：按复杂程度分层，将 `vm_tests.rs` 中的基础测试移植到 `autovm_tests.rs`，确保 AutoVM 的基础功能健壮。

**成功标准**：
- Level 1-3 的基础测试全部移植并通过
- Level 4+ 的复杂测试在基础功能稳定后再处理
- 测试断言使用精确的 `assert_eq!()` 而非模糊的 `.contains()`

## 测试分类（按复杂程度）

### Level 1: 基础字面量和运算（已在 autovm_tests.rs 中）

| 测试名 | 代码 | 状态 |
|--------|------|------|
| test_simple_int | `let x = 41; x` | ✅ 通过 |
| test_basic_add | `1 + 1` | ✅ 通过 |
| test_two_ints | `let a = 10000; let b = 20000; a + b` | ✅ 通过 |
| test_arithmetic | `1 + 2 * 3` | ✅ 通过 |
| test_unary | `-2 * 3` | ✅ 通过 |
| test_group | `(1 + 2) * 3` | ✅ 通过 |
| test_var_arithmetic | `let a = 12312; a * 10` | ✅ 通过 |

### Level 2: 条件判断（待移植）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_if | `if true { 1 } else { 2 }` | 🟢 简单 |
| test_if_else | `if false { 1 } else { 2 }` | 🟢 简单 |
| test_if_else_if | `if false { 1 } else if false { 2 } else { 3 }` | 🟡 中等 |
| test_comp | `1 < 2` | 🟢 简单 |
| test_if_with_bool | `var succ = true; if succ { ... }` | 🟡 中等 |
| test_if_in_array | `["osal", if is_lse {"EB"}, if is_rh {"al"}]` | 🟡 中等 |

### Level 3: 变量和赋值（待移植）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_var | `var a = 1; a + 2` | 🟢 简单 |
| test_var_assign | `var a = 1; a = 2; a` | 🟢 简单 |
| test_var_mut | `var x = 1; x = 10; x + 1` | 🟢 简单 |
| test_let | `let x = 41; x` | 🟢 简单 |
| test_let_asn | `let x = 41; x = 10; x` (应失败) | 🟡 中等 |
| test_var_reassignment | `let x = 41; var x = 10; x` | 🟡 中等 |
| test_var_if | `var x = if true { 1 } else { 2 }; x + 1` | 🟡 中等 |
| test_if_var | `var a = 10; if a > 10 { a + 1 } else { a - 1 }` | 🟡 中等 |
| test_asn_upper | `var a = 1; if true { a = 2 }; a` | 🟡 中等 |
| test_compound_assignment_* | `a += 1`, `a -= 3`, `a *= 3`, `a /= 4` | 🟡 中等 |

### Level 4: 数组和对象（待移植）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_array | `[1, 2, 3]` | 🟢 简单 |
| test_array_element | `var a = [1, 2, 3]; a[0] = 4; a[0]` | 🟡 中等 |
| test_array_of_objects | `[1, 2]` | 🟢 简单 |
| test_array_update | `var a = [1, 2, 3]; a[0] = 4; a` | 🟡 中等 |
| test_object | `var a = { name: "auto", age: 18 }; a.name` | 🟢 简单 |
| test_obj_set | `var a = { name: "Alice" }; a.name = "Bob"; a.name` | 🟡 中等 |

### Level 5: 函数（待移植）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_fn | `fn add(a, b) { a + b }; add(12, 2)` | 🟢 简单 |
| test_closure | `var add = (a, b) => a + b; add(1, 2)` | 🟡 中等 |
| test_closure_with_type_annotations | `let sub = (a int, b int) => a - b` | 🟡 中等 |
| test_simple_function_execution | `fn test() int { 42 }; test()` | 🟢 简单 |
| test_forward_declaration | `fn test() int; fn test() int { 42 }; test()` | 🟡 中等 |

### Level 6: 字符串和 F-string（待移植）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_fstr | `f"hello $name, now!"` | 🟡 中等 |
| test_fstr_with_expr | `f"a + b = ${a+b}"` | 🟡 中等 |
| test_str_index | `let a = "hello"; a[1]` | 🟡 中等 |

### Level 7: 嵌套结构（待移植，AutoVM 稳定后）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_nested_object_field_mutation | `mut obj = { inner: { x: 10 } }; obj.inner.x = 30` | 🔴 复杂 |
| test_array_element_field_mutation | `mut arr = [{x: 1}]; arr[0].x = 10` | 🔴 复杂 |
| test_nested_array_element_mutation | `mut matrix = [[1, 2]]; matrix[0][1] = 20` | 🔴 复杂 |
| test_deep_type | `type A { x int }; type B { a A }; B(a: A(x:1)).a.x` | 🔴 复杂 |

### Level 8: Loop（待移植，AutoVM 稳定后）

| 测试名 | 代码 | 复杂度 |
|--------|------|--------|
| test_for | `var sum = 0; for i in 0..10 { sum = sum + i }; sum` | 🔴 复杂 |
| test_range_eq | `var sum = 0; for i in 0..=10 { sum = sum + i }; sum` | 🔴 复杂 |

### Level 9: VM OOP API（暂时跳过）

这些测试需要完整的 VM OOP API 支持，包括 HashMap、HashSet、StringBuilder、List 等标准库泛型类型。

| 测试类型 | 测试数量 | 状态 |
|---------|-----------|------|
| HashMap OOP API | 7 个测试 | ⏸️ 暂缓 |
| HashSet OOP API | 5 个测试 | ⏸️ 暂缓 |
| StringBuilder OOP API | 5 个测试 | ⏸️ 暂缓 |
| List OOP API | 15 个测试 | ⏸️ 暂缓 |

### Level 10: 特殊功能（暂时跳过）

| 测试名 | 说明 |
|--------|------|
| test_type_decl | 类型声明和实例化 |
| test_type_with_method | 类型方法 |
| test_ext_statement_* | ext 语句扩展 |
| test_borrow_* | 借用检查 |
| test_str_slice_* | 字符串切片 |
| test_grid | Grid 表达式 |
| test_nodes | Node 表达式 |
| test_view_types | View 类型 |

## 移植优先级

### 第一批：Level 2 - 条件判断（优先移植）

```rust
#[test]
fn test_if() {
    let code = r#"fn main() int {
        if true { 1 } else { 2 }
    }"#;
    let result = run_autovm(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1");
}

#[test]
fn test_if_else() {
    let code = r#"fn main() int {
        if false { 1 } else { 2 }
    }"#;
    let result = run_autovm(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "2");
}

#[test]
fn test_if_else_if() {
    let code = r#"fn main() int {
        if false { 1 } else if false { 2 } else { 3 }
    }"#;
    let result = run_autovm(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "3");
}

#[test]
fn test_comp() {
    let code = r#"fn main() int {
        1 < 2
    }"#;
    let result = run_autovm(code);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "1");
}
```

### 第二批：Level 3 - 变量和赋值

### 第三批：Level 4 - 数组和对象

### 第四批：Level 5 - 函数

### 第五批：Level 6 - 字符串和 F-string

## 关键文件

### 目标文件
- `crates/auto-lang/src/tests/autovm_tests.rs` - 测试目标文件

### 参考文件
- `crates/auto-lang/src/tests/vm_tests.rs` - 测试源文件
- `crates/auto-lang/src/vm/engine.rs` - AutoVM 引擎实现
- `crates/auto-lang/src/vm/codegen.rs` - 字节码生成

## 实现步骤

### Phase 1: 移植 Level 2 条件判断测试

1. 移植 test_if, test_if_else, test_if_else_if
2. 移植 test_comp（比较运算）
3. 移植 test_if_with_bool, test_if_in_array
4. 运行测试并修复发现的问题

### Phase 2: 移植 Level 3 变量和赋值测试

1. 移植基础变量测试（test_var, test_var_assign）
2. 移植 let/var 区分测试（test_let, test_let_asn）
3. 移植复合赋值测试
4. 运行测试并修复发现的问题

### Phase 3: 移植 Level 4 数组和对象测试

1. 移植 test_array（数组字面量）
2. 移植 test_array_element, test_array_update
3. 移植 test_object, test_obj_set
4. 运行测试并修复发现的问题

### Phase 4: 移植 Level 5 函数测试

1. 移植 test_fn
2. 移植 test_closure 相关测试
3. 运行测试并修复发现的问题

### Phase 5: 验证和修复

1. 运行所有移植的测试
2. 修复发现的问题
3. 确保所有 Level 1-5 测试通过

## 验证步骤

### 运行移植的测试
```bash
# 运行所有 autovm 测试
cargo test -p auto-lang autovm_tests

# 运行特定测试
cargo test -p auto-lang test_if
cargo test -p auto-lang test_var

# 查看详细输出
cargo test -p auto-lang autovm_tests -- --nocapture
```

### 成功标准
- ✅ 所有 Level 1-5 的测试通过
- ✅ 测试失败时有清晰的错误消息
- ✅ 使用精确断言而非模糊匹配
- ✅ 无回归（原有 11 个测试仍通过）

## 风险评估

| 风险 | 概率 | 影响 | 缓解措施 |
|------|------|------|----------|
| **IF 指令未实现** | 中 | 高 | 先实现或跳过相关测试 |
| **变量存储问题** | 中 | 高 | 已修复 RESERVE_STACK 问题 |
| **数组索引未实现** | 中 | 高 | 检查 engine.rs 指令支持 |
| **函数调用未支持** | 低 | 高 | 跳过函数相关测试 |
| **测试数量过多** | 低 | 中 | 分批移植，优先基础 |

## 时间估算

| Phase | 内容 | 预估时间 |
|-------|------|----------|
| Phase 1 | 条件判断测试 | 1-2 小时 |
| Phase 2 | 变量和赋值测试 | 1-2 小时 |
| Phase 3 | 数组和对象测试 | 1-2 小时 |
| Phase 4 | 函数测试 | 1-2 小时 |
| Phase 5 | 验证和修复 | 1 小时 |
| **总计** | | **5-9 小时** |
