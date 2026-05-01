# Plan 230: 修复 vmtest-17 — f64 字段结构体字面量栈错位

## 问题描述

`vmtest-17-struct-literal.at` 运行时报错：

```
RuntimeError("Invalid instance ID: 18446744073709551614") at line 18
```

其中 `18446744073709551614 = 0xFFFFFFFFFFFFFFFE`，即 `u64::MAX - 1`（或 `i32(-2)` 零扩展后的值）。

## 根因

### 类型宽度不匹配

```
type Point {
    x f64      // Double = 2 slot（f64 需要 8 字节）
    y f64
}

let p = Point(1.0, 2.0)   // 1.0 被编译为 CONST_F32（1 slot）
```

**编译时**：codegen 将浮点字面量 `1.0` 编译为 `CONST_F32`（4 字节，1 slot），因为 lexer 将 `1.0` 解析为 `Float`（f32）token。

**运行时**：`CONSTRUCT_INSTANCE` opcode 执行时：
1. 读取字段类型 → `x: f64, y: f64`
2. 对每个 `f64` 字段调用 `task.ram.pop_f64()`（弹出 8 字节 = 2 slot）
3. 但栈上只有 `CONST_F32` 压入的 4 字节值
4. 第二个 `pop_f64()` 弹出超出栈范围的数据 → 读取到垃圾值
5. 后续 `instance_id` 也从错位的栈中读取 → 得到 `0xFFFFFFFFFFFFFFFE`

### 错误位置

- **engine.rs:2277-2281** — `CONSTRUCT_INSTANCE` 处理 `Type::Double` 字段时调用 `pop_f64()`
- **codegen.rs:3316-3325** — 编译字段值表达式时没有检查字段类型是否需要类型提升

### 相关代码

engine.rs 中 CONSTRUCT_INSTANCE 的字段值弹出：

```rust
Some(crate::ast::Type::Double) => {
    let val_f64 = task.ram.pop_f64();  // ← 需要 2 slot
    Value::Double(val_f64)
}
```

codegen.rs 中字段值编译（没有 f32→f64 提升）：

```rust
for (i, value_expr) in field_values.iter().enumerate() {
    self.compile_expr(value_expr)?;  // ← CONST_F32，只压 1 slot
    // 缺少：检查字段是否为 f64，如果是则 emit PROMOTE_F64
}
```

## 修复方案

### 方案 A：codegen 添加 f32→f64 类型提升（推荐）

在编译结构体字段值后，检查字段声明类型与编译出的表达式类型：

- 字段类型 `Type::Double` + 表达式类型 `ObjectType::Float` → emit `PROMOTE_F64` 或 `F32_TO_F64`
- 字段类型 `Type::Float` + 表达式类型 `ObjectType::Double` → emit `F64_TO_F32`（如果有）

### 方案 B：CONSTRUCT_INSTANCE 使用更安全的栈操作

让 `CONSTRUCT_INSTANCE` 根据实际压入的 slot 数量来弹出，而不是根据字段声明类型。但这会丢失类型信息。

### 方案 C：让浮点字面量 `1.0` 默认为 f64 而非 f32

修改 lexer，将 `1.0` 解析为 `Double`（f64）而非 `Float`（f32）。

**优点**：和 Rust 行为一致（`1.0` 默认是 `f64`）
**缺点**：影响面大，可能破坏已有的 f32 相关逻辑

## 推荐：方案 A + C 长期

短期实施方案 A（codegen 提升）。长期考虑方案 C（默认 f64）。

## 实施步骤

### Step 1: 确认是否存在 F32_TO_F64 opcode

搜索 `d:/autostack/auto-lang/crates/auto-lang/src/vm/opcode.rs` 中是否有浮点提升 opcode。

如果没有，需要添加一个。如果 `CONST_F32` 压入 1 slot 而 `pop_f64()` 需要 2 slot，最简单的修复可能是：
- 直接在 codegen 中将 f64 字段的表达式编译为 `CONST_F64`（而非 `CONST_F32` 后提升）

### Step 2: 修改 codegen struct literal 编译

**文件**：`d:/autostack/auto-lang/crates/auto-lang/src/vm/codegen.rs`
**位置**：编译 `Point(1.0, 2.0)` 的 CONSTRUCT_INSTANCE 路径

在编译字段值后添加类型检查：

```rust
for (i, value_expr) in field_values.iter().enumerate() {
    self.compile_expr(value_expr)?;

    // 类型提升：字段是 f64 但表达式编译为 f32
    if let Some(field_type) = field_types.get(i) {
        if matches!(field_type, Type::Double) &&
           matches!(self.last_expr_type, ObjectType::Float) {
            // 需要将 f32 提升为 f64
            self.emit_f32_to_f64_conversion();
        }
    }
}
```

### Step 3: 实现 f32→f64 转换

有两种实现方式：

**方式 1**：添加新 opcode `PROMOTE_F64`
```rust
// opcode.rs
PROMOTE_F64 = 0x..., // pop f32 (1 slot), push f64 (2 slots)

// engine.rs
OpCode::PROMOTE_F64 => {
    let val = task.ram.pop_f32();
    task.ram.push_f64(val as f64);
}
```

**方式 2**：在 codegen 中直接编译为 `CONST_F64`
```rust
// 检测到 Float 字面量 + Double 字段时，直接 emit CONST_F64
if matches!(value_expr, Expr::Float(_)) && is_double_field {
    let val = /* extract f64 from expr */;
    self.emit(OpCode::CONST_F64);
    self.emit_f64(val);
}
```

方式 1 更通用（处理非字面量场景如函数返回 f32 赋值给 f64 字段），推荐方式 1。

### Step 4: 同步检查其他 opcode 的 f64 处理

确认以下场景是否也有栈宽度问题：
- `SET_GENERIC_FIELD` 对 f64 字段
- `GET_GENERIC_FIELD` 对 f64 字段
- 函数参数传递中 f32→f64 的隐式转换

### Step 5: 验证

1. vmtest-17-struct-literal.at 通过（`Point(1.0, 2.0)` 正确创建）
2. 所有 24 个 VM 测试无回归
3. `cargo test --lib` 无回归
4. 测试 `p.x` 和 `p.y` 输出 `1.0` 和 `2.0`（不是位模式整数）

## 修改文件

- `d:/autostack/auto-lang/crates/auto-lang/src/vm/codegen.rs` — struct literal 编译
- `d:/autostack/auto-lang/crates/auto-lang/src/vm/opcode.rs` — 可能需要新 opcode
- `d:/autostack/auto-lang/crates/auto-lang/src/vm/engine.rs` — 可能需要新 opcode 处理

## 风险

- f64 在 VM 栈上占 2 slot 可能影响其他 opcode 的栈偏移计算
- 如果其他地方也假设浮点值是 1 slot（f32），修复可能需要系统性审查
- `Point(1.0, 2.0)` 中 `1.0` 在 Auto 语义上是 f64（虽然 lexer 给的是 f32），这涉及 Auto 语言的类型系统设计决策
