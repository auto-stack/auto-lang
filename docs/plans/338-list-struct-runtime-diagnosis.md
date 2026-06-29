# Plan 338：List<Struct> 运行时修复 — 排查总结 + 后续

> **For Claude:** 本文档是 Plan 337（单 VM widget 树）过程中对 List<Struct> 运行时问题的完整排查记录。目的是：(1) 记录根因和修复方向；(2) 总结 VM bug 排查方法论，以便优化未来类似问题的排查效率。

## 排查过程（时间线）

### 第一阶段：UI 层排查（低效）
在 015-notes vm 模式下手动点击 UI 控件，复制终端输出。每次迭代需要：
1. 编译 auto.exe
2. 启动 `auto run -r vm`
3. 手动点击按钮
4. 复制终端输出
5. 分析

**效率极低**——每个迭代 5-10 分钟，且无法精确控制。

### 第二阶段：VM 测试框架（高效）
用 `plan337_tests.rs` 在 VM 层直接复现，不需要 UI：
```rust
cargo test -p auto-lang --lib plan337_tests -- --nocapture
```
每个迭代降到 **10 秒以内**。可以自由加诊断、改测试代码、快速验证假设。

**这是最大的效率改进**——把 UI 交互问题降级为纯 VM 测试问题。

### 第三阶段：逐层定位
用 eprintln 诊断逐层缩小范围：

1. `PERF[NewNote] handler=16755ms render=2ms` → 卡顿在 handler 执行
2. `WARN[budget] fn='create_note' call_depth=1250000` → 无限递归
3. `DEBUG[linker] 'db.create_note' resolved via prefix-stripped 'create_note'` → 符号解析正常
4. `DEBUG[codegen native_id] func_name='auto.list.push' native_id=Some(101)` → push 走 CALL_NAT
5. `DEBUG[shim_list_push] list_id=4000001 in_heap=true` → shim 收到正确 list_id
6. `DEBUG[shim_list_len] type_tag=GenericInstance("Item")` → **heap object 是 Item 实例，不是 List！**

## 根因总结

### 核心根因：`List<Item>.new([])` 创建了错误类型的 heap object

`List<Item>.new([])` 被 codegen 的 generic constructor 分发逻辑误处理：
- 预期：创建 `ListData<Value>`（空列表）
- 实际：创建了 `GenericInstanceData("Item")`（Item 结构体实例）

因此：
- `items` 全局变量 = Item 实例的 heap id（4000001），不是 List 的 heap id
- `items.push(item)` → push 到非 List 对象 → 静默失败
- `items.len()` → 查非 List 对象 → 返回 0
- 函数内 `create_note` → `notes.push(note)` → push 失败 → handler 不返回 → call_fn_by_name 预算耗尽（UI 模式下表现为 10 秒卡顿）

### 附带发现：顶层 vs 函数内 push 走不同路径

**同一个 `items.push(item)` 操作，在顶层和函数内编译成不同的操作码：**

| 作用域 | `func_name` | 解析结果 | 路径 |
|--------|-------------|----------|------|
| 顶层（脚本） | `"items.push"` | None（原始名不在 registry） | CALL_SPEC → List 方法分发 |
| 函数内 | `"auto.list.push"` | Some(101)（类型推断规范化） | CALL_NAT → `shim_list_push` |

**原因**：类型推断信息的有无导致 `func_name` 不同。
- 顶层：`items` 不在 `var_types` → `func_name` 保持原始名 `"items.push"` → native lookup 失败 → 走 CALL_SPEC
- 函数内：`items` 在 `var_types`（编译器推断为 `List<Item>`）→ `func_name` 被规范化为 `"auto.list.push"` → native lookup 成功 → 走 CALL_NAT

**这是设计缺陷**：同一个操作应走同一路径。两条路径（CALL_SPEC List 分支 vs shim_list_push）对 ListData<Value> 的支持不一致，导致行为差异。

## 修复方向

### 优先级 1：修复 generic constructor（解除卡顿）

`List<Item>.new([])` 应该创建 `ListData<Value>`，而不是 `GenericInstanceData("Item")`。

**位置**：`crates/auto-lang/src/vm/codegen.rs` 的 generic constructor 分发（~line 5190）。

**问题**：`List<Item>.new(args)` 被当作泛型类型 `List<Item>` 的构造函数，但它应该走 `shim_list_new`（创建 ListData）。

**修复方向**：
- 在 generic constructor 检查中，特判 `List<T>.new` → 走 shim_list_new（而非 CONSTRUCT_INSTANCE）
- 或：在 CONSTRUCT_INSTANCE 执行时，如果 mono_name 以 `List` 开头，创建 ListData 而非 GenericInstanceData

### 优先级 2：统一 push/len 路径（消除顶层/函数内不一致）

两条路径应统一：
- **方案 A**：让 CALL_SPEC List 分支也处理 ListData<Value>（已部分完成）
- **方案 B**：让 shim_list_push/shim_list_len 也处理 ListData<Value>（已完成）
- **最终**：确保两条路径对 ListData<Value> 的行为一致

当前状态：两个 shim 都已修支持 ListData<Value>，但因为 generic constructor bug 创建了错误类型，所以仍然不工作。

### 优先级 3：其他附带修复（已提交）

- `shim_list_new`：空列表 → `ListData<Value>`（已修）
- `CALL_SPEC List` 分支：统一 list_id + ListData<Value> 支持（已修）
- app.at：NewNote/SaveNote/DeleteNote 刷新 .notes（已修）

## 排查方法论总结

### 高效排查的关键
1. **降级复现**：UI 交互问题 → VM 测试用例（`run_with_capture`）。从 5-10 分钟/迭代降到 <10 秒。
2. **逐层 eprintln**：从外到内加诊断，每层缩小范围。不要一次加太多。
3. **type_tag 诊断**：对 heap object 打 `type_tag().name()`，立刻看出存储类型是否正确。
4. **call_depth 诊断**：判断是死循环（depth 大）还是慢操作（depth 小）。
5. **budget exhaustion 诊断**：给 call_fn_by_name 加 step counter，确认是否达到预算上限。

### 低效排查的教训
1. **不要在 UI 上手动测试**：除非问题只在 UI 层出现。VM 层能复现的就用 VM 测试。
2. **不要一次加太多诊断**：逐层加，每层确认后再深入。
3. **注意 codegen 的多路径分发**：同一个操作（push）可能走 CALL_NAT 或 CALL_SPEC，取决于类型推断。排查时要检查走的是哪条路径。

## 当前测试状态

`plan337_tests.rs`（7 个测试）：
- ✅ `test_basic_print`
- ✅ `test_basic_struct`
- ✅ `test_list_struct_push`（push 后访问字段）
- ✅ `test_list_struct_len_toplevel`（顶层 len）
- ✅ `test_list_struct_push_toplevel`（顶层 push + len）
- ✅ `test_list_push_int_basic`（int list push + len）
- ❌ `test_list_struct_push_then_len`（**函数内 push + len** — 等 generic constructor 修复）
