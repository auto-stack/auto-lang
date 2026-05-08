# Plan 230: 修复 vmtest-17 — f64 字段结构体字面量栈错位

## 状态: 已完成

## 问题描述

`Point(1.0, 2.0)` 运行时报错 `Invalid instance ID: 0xFFFFFFFFFFFFFFFE`。

## 根因

类型宽度不匹配：`1.0` 被 lexer 解析为 `Float`(f32, 1 slot)，但 `CONSTRUCT_INSTANCE` 对 `f64` 字段调用 `pop_f64()`(2 slot)，导致栈错位。

## 修复方案

采用方案 A：在 codegen 所有构造函数路径中添加 f32→f64 提升。

已有 `PROMOTE_F64` opcode (0xF1)：弹出 f32(1 slot)，压入 f64(2 slot)。

## 修复内容

### codegen.rs — 5 条构造函数路径全部添加 PROMOTE_F64

在编译每个构造函数参数后，检查字段声明类型与编译出的表达式类型：

```rust
if matches!(field_type, Type::Double) && self.last_expr_type == ObjectType::Float {
    self.emit(OpCode::PROMOTE_F64);
}
```

覆盖的 5 条代码路径：
1. **Expr::Node** struct literal（~line 3298）
2. **is_generic_constructor** 路径（~line 4600）
3. **is_enum_variant** 路径（~line 4710）
4. **type constructor call** `Point(1.0, 2.0)` 走的主路径（~line 4800）
5. **normal function call** fallback 路径（~line 5590）

### 附带修复：fn_return_obj expected.out

Plan 231 的 BUILD_FSTR 修复使 struct instance 正确格式化（`Point { x: 42 }` 而非 `4000000`），更新了 expected.out。

## 测试结果

- Plan 230 测试: 1/1 通过（001_struct_f64）
- 全量回归: 285/288 通过（3 失败均为已知 bug：hashmap + 2 SSE parser）
- 无回归

## 已知限制

- `print(f64)` 将 f64 当作 f32 处理（`NATIVE_PRINT_F32` 只弹 1 slot），无法正确打印 f64 值
- f-string `$expr.field` 不支持点访问语法

## 修改文件

- `crates/auto-lang/src/vm/codegen.rs` — 5 条路径添加 PROMOTE_F64
- `crates/auto-lang/test/vm/99_plan230/001_struct_f64/` — 测试用例
- `crates/auto-lang/test/vm/10_types/013_fn_return_obj/` — 更新 expected.out
- `crates/auto-lang/src/tests/vm_file_tests.rs` — 测试注册
