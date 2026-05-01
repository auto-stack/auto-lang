# Plan 231: 嵌套 mut fn 调用 + for 循环导致栈损坏

## 状态: 待实施

## 问题描述

`vmtest-08-sse-parser.at` 运行时 panic：

```
Memory Access Out of Bounds at virt_memory.rs:477
```

但只在以下条件同时满足时触发：
1. `mut fn push()` 内部调用 `mut fn drain_frames()`
2. `drain_frames()` 中有 `for frame != None` 条件循环

单独调用 `drain_frames()` 或在 `push()` 中不调用 `drain_frames()` 都不会 panic。

## 根因

### 最小复现

```auto
type SseParser {
    buffer str

    mut fn push(chunk str) {
        self.buffer = self.buffer + chunk
        self.drain()          // 嵌套 mut fn 调用
    }

    mut fn drain() {
        var frame = self.next_frame()
        for frame != None {   // 条件循环
            frame = self.next_frame()
        }
    }

    mut fn next_frame() ?str { ... }
}
```

### 可能的原因

1. **`self` 参数在嵌套调用时被覆盖**: `mut fn push` 的 `self` 是局部变量（参数 index 0）。当调用 `self.drain()` 时，`self` 的 instance_id 被压入栈作为参数。`drain()` 返回后，栈帧清理可能没有正确恢复 `push()` 的 `self` 参数。

2. **`for frame != None` 循环的栈操作**: 循环条件编译为 `LOAD frame + LOAD None + NE + JMP_IF_Z`。如果 `frame` 是枚举实例（heap object ID），而 `None` 也是枚举实例，`NE` 比较的是 i32 值（4 字节）。但枚举实例 ID 可能和其他栈上的值混淆。

3. **`frame = self.next_frame()` 的赋值**: `var frame` 的索引和 `self` 参数的索引可能冲突。

### 需要确认

- `for frame != None` 中 `None` 在栈上是如何编码的？（枚举实例 ID？特殊值？）
- 嵌套 `mut fn` 调用时，`self` 的 instance_id 是否被正确传递？
- 循环体内 `frame = self.next_frame()` 是否破坏了外层函数的栈帧？

## 修复方案

### 方案 A: 调试循环的栈操作（推荐）

1. 添加 debug log 追踪每次 `JMP_IF_Z` 前后的 SP（栈指针）
2. 比较嵌套调用前后的 SP 是否一致
3. 确认 `for frame != None` 循环的 `NE` 操作是否消费了正确的栈位置

### 方案 B: 修改 `for frame != None` 编译

`frame != None` 中的 `None` 可能被编译为创建一个 `Option.None` 枚举实例（heap allocation），这导致每次循环迭代都创建新对象。改为使用 `IS_VARIANT` 检查更高效也更安全。

### 方案 C: 检查 `self` 参数在嵌套调用中的保存

在 `mut fn push()` 调用 `self.drain()` 前，`self` 的 instance_id 可能在参数区域。调用返回后，检查 `self` 的值是否被保留。

## 诊断步骤

### Step 1: 添加栈追踪

在 `engine.rs` 中，对 `for` 循环的 `NE` + `JMP_IF_Z` 序列添加 SP 日志。

### Step 2: 检查 `None` 的编译方式

搜索 codegen 中 `for frame != None` 的 `None` 如何被编译（是 IS_VARIANT 还是 NE？）

### Step 3: 检查 `frame = expr` 的赋值路径

`var frame` 是可变局部变量，`frame = self.next_frame()` 应该是 `STORE_LOC`。确认 index 是否与 `self` 参数冲突。

## 修改文件

- `d:/autostack/auto-lang/crates/auto-lang/src/vm/engine.rs` — 可能需要修改循环/比较逻辑
- `d:/autostack/auto-lang/crates/auto-lang/src/vm/codegen.rs` — 可能需要修改 `for cond {}` 编译

## 风险

- 栈帧管理是 VM 的核心，修改可能影响所有函数调用
- 需要仔细追踪 SP 变化，避免引入新的栈不平衡
