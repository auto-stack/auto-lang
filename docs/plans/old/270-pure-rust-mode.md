# Plan 270: Pure Rust Mode — a2r 条件化 a2r_std 导入

## 目标

当 Auto 代码不使用任何 Auto 标准库（只用 `use.rust` + Rust 原生类型）时，
生成的 Rust 代码完全不依赖 `auto_lang` crate，可独立编译。

## 关键发现

经过深入分析，发现当前架构中**只有一个问题**需要解决：

- `crate::` 路径 → **不是问题**，Rust 中 `crate::` 指向当前 crate 自身，生成的 Cargo.toml 已是独立的
- 172 处 `a2r_std::` 拦截点 → **不是问题**，只有实际调用 Auto stdlib 函数时才触发
- **唯一问题**：`emit_a2r_stdlib()` 无条件生成 `use auto_lang::a2r_std::*;`，即使代码中一个 `a2r_std` 符号都没用

## 实施步骤

### Step 1: 添加 `a2r_std_used` 追踪字段

**文件**: `crates/auto-lang/src/trans/rust.rs`

在 `RustTrans` 结构体中添加 `a2r_std_used: bool` 字段（默认 `false`）。

### Step 2: 在所有 `a2r_std::` 生成点标记追踪

在 172 处生成 `a2r_std::xxx` 的代码中，每次触发时设 `self.a2r_std_used = true`。

实现方式：添加辅助方法 `fn mark_a2r_used(&mut self)`，
在每个 `write!(out, "a2r_std::...")` 调用前调用它。

### Step 3: 条件化 `emit_a2r_stdlib()`

将 `emit_a2r_stdlib()` 中的导入改为条件性：
- 如果 `a2r_std_used == false` 且 `!merge_mode` → 跳过 `use auto_lang::a2r_std::*;`
- 否则 → 保持当前行为

### Step 4: 添加测试

1. 纯 Rust 代码（无 Auto stdlib）→ 验证输出不含 `a2r_std`
2. 混合代码（有 Auto stdlib）→ 验证输出包含 `a2r_std`（不回归）
3. 现有 a2r 测试全部通过（不回归）

## 成功标准

1. Auto 代码只使用 `use.rust` + Rust 原生类型 → 生成的 Rust 不含任何 `auto_lang` / `a2r_std` 引用
2. Auto 代码使用了 Auto stdlib → 生成的 Rust 正常包含 `a2r_std`（行为不变）
3. 所有现有 a2r 测试通过
