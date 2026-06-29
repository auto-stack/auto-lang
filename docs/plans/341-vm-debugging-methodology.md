# 015-notes VM 模式排查全记录 — 方法论分析

> **For Claude:** 本文档分析 015-notes `--render=vm` 从完全不可用到基本可用的全过程（Plan 333-338），识别排查模式、效率瓶颈和改进方向，并制定**VM 调试最佳实践**。

## 1. 总览

### 1.1 规模
- **计划数量**：6 个 Plan（333-338）
- **Commit 数**：~35 个
- **排查层数**：最深达 6 层（Plan 338 的 List\<Struct\> runtime）
- **总时长**：估计 8-12 小时（跨多个会话）

### 1.2 问题分类

| 类别 | 示例 | 占比 |
|------|------|------|
| **架构设计缺陷**（最耗时） | 单 VM vs 多 VM、Symbol 扁平化 | ~40% |
| **codegen 路径不一致** | 顶层 vs 函数内 push 走不同路径 | ~20% |
| **运行时方法缺位** | `to_array`/`filter`/`remove` 不支持 struct | ~15% |
| **简单源码 bug** | handler 参数名缺失、active_id 混淆 | ~10% |
| **VM 存储模型缺陷** | `ListData<i32>` vs `ListData<Value>` 双轨 | ~15% |

## 2. 排查效率分析

### 2.1 高效做法 ✅

#### ① VM 测试框架（最大效率提升）

**从"手动点击 UI" → "VM 脚本测试"**

```
# 之前：5-10 分钟/迭代
cd examples/ui/015-notes
auto run -r vm -B 3030
# 等待窗口...
# 手动点击按钮...
# 复制终端输出...
# 分析...

# 之后：<10 秒/迭代
cargo test -p auto-lang --lib plan337_tests -- --nocapture
```

**效果**：迭代速度从 5-10 分钟降到 <10 秒，约 **30-60x 提升**。

**关键**：UI 交互问题降级为纯 VM 测试问题是整个排查过程中最大的转折点。这个转折发生在 Plan 338 中期（用户建议"写一个模拟程序"）。

**改进方向**：**任何 UI bug 都应先尝试用 VM 脚本复现。** 如果 VM 脚本能复现，就永远不要在 UI 层排查。

#### ② 逐层 eprintln 诊断

**从外到内，每层缩小范围**：

```
Layer 1: PERF[NewNote] handler=16755ms → 卡顿在 handler
Layer 2: WARN[budget] call_depth=1250000 → 无限递归
Layer 3: DEBUG[linker] → 符号解析正常
Layer 4: DEBUG[codegen native_id] → push 走 CALL_NAT
Layer 5: DEBUG[shim_list_push] list_id=4000001 → 收到正确 id
Layer 6: type_tag=GenericInstance("Item") → heap object 类型错误 ← 根因
```

**效果**：6 层诊断，每层 1-2 个 `eprintln`，精确定位根因。

**改进方向**：**每层诊断后必须确认假设**，再深入下一层。不要同时加 10 个诊断然后迷失。

#### ③ type_tag 诊断

```rust
eprintln!("type_tag={:?}", guard.type_tag());
```

这一行直接暴露了 `GenericInstance("Item")` 而非 `ListData<Value>`，**跳过了一整层猜测**。

**改进方向**：**在所有 heap_object 相关的 shim 中超时，自动 dump type_tag。** 这可以作为 debug build 的默认行为。

#### ④ call_depth 递归检测

```rust
let call_depth = task.call_stack.len();
eprintln!("call_depth={}", call_depth);
```

`call_depth=1,250,000` 直接确认了无限递归，**避免了逐条指令追踪**。

**改进方向**：**在 call_fn_by_name 中，当 steps > 1000 且 call_depth > 100 时，自动打印警告。** 无需手动添加诊断。

### 2.2 低效做法 ❌

#### ① 在 UI 上手动测试（早期阶段）

**浪费**：Plan 333-335 期间，大量时间花在手动点击 UI 控件、等待编译、复制终端输出上。

**改进**：建立 VM 测试习惯。UI 测试只用于视觉确认（渲染是否正确），不用于逻辑排查。

#### ② 在同一 commit 中堆叠多个修复

**浪费**：Plan 335 至少改了 4 个不同的 bug（VmRef deref、materialize_obj_ref、Index 表达式、FieldAccess），每个都暴露了新问题，但在同一个 commit 里反复迭代。

**改进**：**先将单测写好，再修代码。** 每个 bug 独立 commit。

#### ③ 错误假设导致错误方向

**示例**：Plan 338 初期，假设 `to_array()` 是根因（修 identity），实际根因是 generic constructor 创建了错误的类型。

**浪费**：至少 2 个迭代在错误方向上。

**改进**：**在看到数据之前不要下结论。** 先加诊断，再形成假设。诊断 > 假设。

#### ④ 没有使用 budget/exhaustion 检测

**浪费**：New 按钮卡顿 10 秒，但早期没有意识到是预算耗尽（以为是真正的慢操作）。

**改进**：**call_fn_by_name 的 budget 检测应该默认开启**（至少 debug build），并在接近预算时打印警告。

## 3. VM 调试最佳实践（建议加入 CLAUDE.md）

### 3.1 通用原则

1. **永远先尝试在 VM 脚本层复现，绝不在 UI 层排查逻辑 bug**
2. **每层诊断后确认假设，再深入下一层**
3. **诊断 > 猜测**：先看数据，再形成假设
4. **单测驱动修复**：写好失败测试 → 修代码 → 验证
5. **隔离关注点**：每个 bug 独立 commit，不堆叠修复

### 3.2 VM 特定技巧

| 诊断工具 | 用法 | 适用场景 |
|----------|------|---------|
| `type_tag()` | dump heap object 类型 | 确认对象是 ListData/GenericInstance/... |
| `call_depth` | `task.call_stack.len()` | 检测无限递归 |
| `budget exhaustion` | `steps >= budget` 警告 | 检测慢操作 vs 死循环 |
| `heap_objects` keys | `self.heap_objects.iter()` | 看有哪些对象存在 |
| `arrays` keys | `self.arrays.iter()` | 看有哪些数组存在 |
| `global_vars` 内容 | `self.globals.iter()` | 看全局变量是否正确 |
| `exports_by_name` | `self.flash.exports_by_name` | 看导出了哪些符号 |
| codegen trace | `DEBUG[codegen] func_name=... native_id=...` | 确认走了哪条编译路径 |

### 3.3 代码改进建议

1. **call_fn_by_name 超时自动诊断**
   ```rust
   if steps > 1000 && call_depth > 100 {
       log::warn!("Possible infinite recursion: fn='{}' depth={}", fn_name, call_depth);
   }
   if steps > budget * 0.9 {
       log::warn!("Budget nearly exhausted: fn='{}' steps={}", fn_name, steps);
   }
   ```

2. **shim 方法自动 type_tag 报告**（debug build）
   ```rust
   #[cfg(debug_assertions)]
   if let Some(obj) = vm.get_heap_object(list_id) {
       log::debug!("shim_list_push: list_id={} type={:?}", list_id, obj.read().unwrap().type_tag());
   }
   ```

3. **generic constructor 类型名 trace**
   当 `is_generic_constructor` 为 true 时，打印类型名和字段数，帮助确认是否误触发了某个类型的构造。

4. **linker 符号表 dump 工具**
   添加一个 `vm.dump_symbols()` 方法，打印所有导出符号及其地址，便于排查符号冲突。

## 4. 最耗时的根因排名

| 排名 | 问题 | 耗时原因 | 可用什么避免 |
|------|------|---------|-------------|
| 1 | `List<Item>.new` 创建错误类型 | 需要拆解 6 层才能定位 | type_tag 自动诊断 |
| 2 | `create_note` 无限递归 | 隐蔽的符号冲突 + 预算耗尽 | call_depth 自动诊断 |
| 3 | 顶层 vs 函数内 codegen 路径不一致 | 两条路径不易对比 | codegen trace 工具 |
| 4 | bool 存为 Int 导致条件判断失败 | 类型系统缺位 | Value 类型断言宏 |
| 5 | 多 VM vs 单 VM 架构 | 设计缺陷，需要重构 | 提前设计评审 |

## 5. 未来类似问题的建议流程

```
第一阶段：VM 脚本复现（5 分钟）
  ├── 写最小复现代码（run_with_capture）
  ├── 跑测试确认复现
  └── 如果不复现 → 问题在 UI 层，转到第二阶段

第二阶段：逐层诊断（15-30 分钟）
  ├── Layer 1: handler 执行时间（PERF 诊断）
  ├── Layer 2: 递归检测（call_depth）
  ├── Layer 3: 代码路径确认（codegen trace）
  ├── Layer 4: 对象类型确认（type_tag）
  ├── Layer 5: 数据内容确认（dump values）
  └── 每层确认假设后再深入

第三阶段：写单测 + 修复（10-20 分钟）
  ├── 写失败测试
  ├── 最小修复
  ├── 验证测试通过
  └── 回归测试

第四阶段：UI 验收（5 分钟）
  ├── 跑 UI 确认视觉正确
  └── 手动测试核心交互
```

**预期总时间**：35-60 分钟（vs 当前 8-12 小时，**约 10x 提升**）。

## 6. 总结

015-notes vm 模式的排查过程，从"需要手动点击 UI 等待编译"迭代到"VM 脚本 <10 秒自动化测试"，效率提升了 30-60 倍。核心经验：

1. **VM 测试框架是最重要的工具**——能把任何 UI bug 转化成纯 VM bug
2. **逐层诊断比堆叠猜测高效**——每层确认后再深入
3. **type_tag、call_depth、budget exhaustion 三个诊断覆盖了 80% 的问题**
4. **架构缺陷（多 VM、Symbol 扁平化）是最大时间杀手**——应该在设计阶段就评审，而非在实现后修改
