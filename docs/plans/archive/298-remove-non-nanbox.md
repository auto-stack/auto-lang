# Plan 298: 移除 non-nanbox VM 架构

> **Status: ✅ COMPLETED** (2026-06-12) — All 204 non-nanbox blocks removed, 226 nanbox blocks unwrapped. 2739/2746 tests pass (7 pre-existing failures).

## 背景

Plan 221 引入 NaN-boxing 值表示方案，作为 AutoVM 的默认架构。迁移期间保留了 non-nanbox 路径作为安全网。当前状态：

- `nanbox` 是 `Cargo.toml` 的默认 feature，所有 CI 和测试都只跑 nanbox
- non-nanbox 路径从未在 CI 中测试，代码越来越陈旧
- 每次改 VM 核心逻辑需要同步维护两套代码（如本次 Plan 修 ADD 操作码）
- 原计划 Phase 4 明确要求清理：*Remove `#[cfg(not(feature = "nanbox"))]` blocks, make NaN-boxing the only implementation*

## 影响范围

| 文件 | `#[cfg(feature = "nanbox")]` | `#[cfg(not(feature = "nanbox"))]` |
|------|-----|------|
| `vm/engine.rs` | ~150 块 | ~120 块 |
| `vm/virt_memory.rs` | ~24 块 | ~20 块 |
| `vm/native.rs` | ~47 块 | ~30 块 |
| `vm/ffi/stdlib.rs` | ~12 块 | ~10 块 |
| `vm/ffi/convert.rs` | 1 块 | 0 块 |
| `lib.rs` | ~4 块 | ~4 块 |
| `interpreter/vm_interpreter.rs` | ~2 块 | ~2 块 |
| **合计** | **~240 块** | **~186 块** |

## 执行步骤

### Phase 1: 移除 `#[cfg(not(feature = "nanbox"))]` 块

逐文件删除所有 `#[cfg(not(feature = "nanbox"))]` 分支代码，只保留 nanbox 路径。同时将 `#[cfg(feature = "nanbox")]` 包裹的代码"解包"（去掉 cfg 属性，让代码无条件编译）。

**处理顺序**（从底层到上层）：

#### Step 1.1: `crates/auto-lang/src/vm/virt_memory.rs`
- 删除所有 `#[cfg(not(feature = "nanbox"))]` 块
- 删除 `#[cfg(feature = "nanbox")]` 属性（保留其内部代码）
- 保留相关注释但去掉 cfg 说明

#### Step 1.2: `crates/auto-lang/src/vm/engine.rs`
- 同上处理，这是最大的文件（~240 个 cfg 块）
- 重点注意 `ADD`/`SUB`/`MUL`/`DIV` 等算术操作码的双路径
- 保留 `push_tagged`/`pop_tagged` 等 nanbox 辅助函数

#### Step 1.3: `crates/auto-lang/src/vm/native.rs`
- 处理 native 函数的参数读取和返回值设置
- 注意 `encode_string_index`/`decode_string_index` 辅助函数

#### Step 1.4: `crates/auto-lang/src/vm/ffi/stdlib.rs`
- 处理 stdlib FFI 函数中的 nanbox 分支

#### Step 1.5: `crates/auto-lang/src/vm/ffi/convert.rs`
- 小文件，处理 f64 转换

#### Step 1.6: `crates/auto-lang/src/lib.rs`
- 处理栈调试/检查相关代码

#### Step 1.7: `crates/auto-lang/src/interpreter/vm_interpreter.rs`
- 处理结果格式化代码

### Phase 2: 移除 `#[cfg(feature = "nanbox")]` 属性

所有 `#[cfg(feature = "nanbox")]` 属性不再需要（因为 nanbox 已成为唯一路径），但**保留其内部代码**。

- 将 `#[cfg(feature = "nanbox")] { ... }` 替换为直接的 `...` 代码
- 如果同一函数有多个 cfg 块的替代实现，只保留 nanbox 版本

### Phase 3: 清理 Cargo.toml

#### Step 3.1: 移除 feature 定义
```toml
# 删除这行
nanbox = []

# 从 default features 中移除
default = ["with-file-history", "ui-iced"]  # 去掉 "nanbox"
```

#### Step 3.2: 清理相关的 cfg_attr 引用
检查是否有 `#[cfg_attr(feature = "nanbox", ...)]` 等属性引用需要移除。

### Phase 4: 清理注释和文档

- 移除代码中解释 nanbox vs non-nanbox 差异的注释
- 更新 `docs/plans/old/221-nanboxing-migration.md` 标记 Phase 4 已完成
- 检查 `docs/design/05-vm-runtime.md` 是否需要更新
- 移除 `crates/auto-lang/src/ui/vm_bridge.rs` 中 "works in both nanbox and non-nanbox modes" 等注释

### Phase 5: 清理 `crates/auto-val/src/nano_value.rs`

该文件是 nanbox 编码核心，移除 non-nanbox 后它成为唯一路径：
- 检查是否有 non-nanbox 相关的备用编码逻辑
- 确保 `NanoValue` 函数仍正确导出
- `crates/auto-val/` 本身没有 cfg 分支，无需改动，但确认 API 无变化

### Phase 6: 构建验证

```bash
cargo build                  # 确保编译通过
cargo test                   # 确保所有测试通过
cargo test -p auto-lang      # auto-lang 测试
cargo test -p auto-lang -- trans  # transpiler 测试
```

重点测试项：
- 基本算术运算（int, float, 混合类型）
- 字符串操作（拼接、比较）
- VM 栈操作（push/pop 各种类型）
- Native FFI 函数
- UI stopwatch 示例（`auto examples/ui/012-stopwatch/src/front/app.at`）

## 风险评估

- **风险低**：non-nanbox 路径从未在 CI 中测试，移除不影响任何现有功能
- **回滚简单**：所有改动在单个 commit 中完成，`git revert` 即可
- **收益明确**：减少 ~350-500 行维护代码，消除未来双路径同步问题

## 预计改动量

| 类型 | 数量 |
|------|------|
| 删除文件 | 0 |
| 修改 .rs 文件 | ~8 个 |
| 修改 Cargo.toml | 1 个 |
| 修改 .md 文件 | 1-2 个 |
| 预计删除代码行 | ~350-500 行 |
| 预计清理注释行 | ~40 行 |
