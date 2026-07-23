# Plan 340：List\<T\> 运行时完整方法支持 — filter/map/remove 等

> **For Claude:** 当前 List\<T\>（struct 元素类型，即 `ListData<Value>`）的许多方法仅支持 `ListData<i32>`（int 元素类型）。015-notes 通过 for-in 遍历绕过（delete_note、update_note），但 `filter`/`map`/`remove`/`foreach`/`find`/`contains`/`get` 等原生 shim 只处理 i32。本计划统一所有 List 方法对 `ListData<Value>` 的支持。

## 1. 问题

### 1.1 `ListData<i32>` vs `ListData<Value>` 双轨制

`List<T>.new(args)` 按元素类型分两条存储路径（Plan 335 文档详述）：

| 元素类型 | 存储对象 | id 段 | shim 支持 |
|----------|---------|-------|----------|
| 全 int | `heap_objects` → `ListData<i32>` | 4M | 所有方法 ✅ |
| 含 struct/str | `heap_objects` → `ListData<Value>` | 4M | push/len ✅（Plan 322），其余 ❌ |

### 1.2 015-notes 中的 workaround

`delete_note` 和 `update_note` 用 for-in 遍历 + push 重建列表，绕过不支持的 `filter` 方法：

```auto
// 当前 workaround（Plan 322）
pub fn delete_note(id int) bool {
    var new_notes List<Note> = List<Note>.new([])
    var i int = 0
    for note in notes {
        if i != id { new_notes.push(note) }
        i = i + 1
    }
    notes = new_notes
    return true
}
```

如果 `notes.filter(...)` 能工作，可以简化为一行：
```auto
pub fn delete_note(id int) bool {
    notes = notes.filter((n Note) => n.id != id)
    return true
}
```

### 1.3 受影响的 shim 方法

| 方法 | 文件:行 | i32-only 原因 | 使用场景 |
|------|---------|-------------|---------|
| `shim_list_filter` | native.rs:1357 | `get_list_i32_elements` + `create_list_from_i32` | delete/update |
| `shim_list_map` | native.rs:1322 | `get_list_i32_elements` + `create_list_from_i32` | 数据转换 |
| `shim_list_for_each` | native.rs:1380 | `get_list_i32_elements` | 副作用遍历 |
| `shim_list_find` | native.rs:1396 | `get_list_i32_elements` | 查找 |
| `shim_list_contains` | native.rs:1517 | `get_list_i32_elements` | 包含检查 |
| `shim_list_remove` | 不存在 | — | 按索引删除 |
| `shim_list_get` | native.rs:1114 | 仅检查 `ListData<i32>` | 按索引读取 |
| `CALL_SPEC List:count/len` | engine.rs:4589 | 仅检查 `ListData<i32>` | 获取长度（已修） |
| `CALL_SPEC List:push` | engine.rs:4624 | 仅检查 `vm.arrays` | 追加（已修） |
| `CALL_SPEC List:get` | engine.rs:4599 | 仅检查 `ListData<i32>` | 按索引读取 |

**模式总结**：两个辅助函数 `get_list_i32_elements` 和 `create_list_from_i32` 被 6 个 shim 使用，它们只处理 i32。而 CALL_SPEC 中另有 3 个方法（count/len/push/get）只查 `ListData<i32>`。

## 2. 方案

### 2.1 分层修复

**层 1：CALL_SPEC List 分支（已有部分修复）**

当前 Plan 322 已修复 `count/len` 和 `push`。待修复：
- `get`（engine.rs:4599）：只查 `ListData<i32>`，需加 `ListData<Value>` 回退

**层 2：原生 shim（需大量改造）**

`filter`/`map`/`for_each`/`find`/`contains` 全部依赖 `get_list_i32_elements`。核心思路：
- 新增 `get_list_value_elements(vm, list_id) -> Vec<Value>` 辅助函数
- 新增 `create_list_from_value(vm, elements) -> u64` 辅助函数
- 每个 shim 改为：先试 `get_list_i32_elements`（fast path），失败则用 Value 版本

**层 3：closure 调用的类型差异**

`filter`/`map`/`for_each`/`find` 都接受一个 **closure**。closure 的参数类型在 i32 模式下是裸 int，在 Value 模式下需要是 heap id。

关键问题：closure 编译时不知道元素是 i32 还是 struct。对于 `List<Note>.filter((n Note) => ...)`，closure 期望 `n` 是 Note 类型。但对于 `List<int>.filter((x) => ...)`，closure 期望 `x` 是 int。

VM 执行时，closure 的参数通过栈传递。当前 i32 路径 push_i32，closure 的 codegen 期望裸 i32 在栈上。对于 struct 路径，需要 push heap id（也是 i32），closure 通过 GET_FIELD 读取字段。

**这意味着：struct 元素的 closure 调用和 i32 元素的 closure 调用在栈层面是兼容的**（都是 push 一个 i32）。关键区别在于：shim 需要知道元素是 i32（裸值）还是 heap id（需要 GET_FIELD）。

最简单的实现：新增 Value 版本的 shim，在调用 closure 前 push `elem_val`（裸值或 heap id），让 codegen 处理差异。

### 2.2 实现计划

#### Phase 1 — 新增辅助函数

**文件**：`crates/auto-lang/src/vm/native.rs`

```rust
/// Try to get elements from any ListData<T> as Vec<Value>.
fn get_list_value_elements(vm: &AutoVM, list_id: u64) -> Option<Vec<Value>> {
    let obj = vm.get_heap_object(list_id)?;
    let guard = obj.read().unwrap();
    // Try ListData<Value> first
    if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
        return Some(list.elems.clone());
    }
    // Fallback: ListData<i32> → convert to Value
    if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
        return Some(list.elems.iter().map(|i| Value::Int(*i)).collect());
    }
    None
}

/// Create a new ListData<Value> and return its heap id.
fn create_list_from_value(vm: &AutoVM, elems: Vec<Value>) -> u64 {
    let list = ListData::from_iter(elems);
    vm.insert_heap_object(list)
}
```

#### Phase 2 — 改造 shim（逐个）

每个 shim 改为：先试 `get_list_i32_elements`（保持 int fast path）→ 失败则用 `get_list_value_elements`。

**filter 示例**：
```rust
pub fn shim_list_filter(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    let closure_id = task.ram.pop_i32() as u32;
    let list_id = task.ram.pop_i32() as u64;

    // Fast path: i32 elements
    if let Ok(elements) = get_list_i32_elements(vm, list_id) {
        let mut results = Vec::new();
        for elem in elements {
            task.ram.push_i32(elem);
            vm.call_closure(task, closure_id, 1)?;
            if vm_is_truthy(task.ram.pop_i32()) {
                results.push(elem);
            }
        }
        let new_id = create_list_from_i32(vm, results);
        task.ram.push_i32(new_id as i32);
        return Ok(());
    }

    // Value path: struct/Value elements
    if let Some(elements) = get_list_value_elements(vm, list_id) {
        let mut results: Vec<Value> = Vec::new();
        for elem in &elements {
            push_value(&mut task.ram, elem);  // push as tagged i32 or string
            vm.call_closure(task, closure_id, 1)?;
            if vm_is_truthy(task.ram.pop_i32()) {
                results.push(elem.clone());
            }
        }
        let new_id = create_list_from_value(vm, results);
        task.ram.push_i32(new_id as i32);
        return Ok(());
    }

    task.ram.push_i32(0); // fallback: empty list
    Ok(())
}
```

需要类似改造的 shim：`map`, `for_each`, `find`, `contains`。

#### Phase 3 — CALL_SPEC 补充

修复 `CALL_SPEC List:get`（engine.rs:4599）：加 `ListData<Value>` 回退。

#### Phase 4 — 回归 + 验收

- 015-notes：`delete_note` 和 `update_note` 可改为用 `filter`/`map`（可选，验证通过即可）
- `plan320_tests`：新增 `test_list_filter`、`test_list_map` 测试
- 016-calendar：回归正常

## 3. 范围

### 做
- `filter`/`map`/`for_each`/`find`/`contains` 的 `ListData<Value>` 支持
- `CALL_SPEC List:get` 的 `ListData<Value>` 支持
- VM 测试覆盖

### 不做
- `remove`/`set`/`insert`/`sort`（无立即需求，可按需后续添加）
- 统一 ListData\<i32\> 和 ListData\<Value\>（更大重构，非本计划目标）
- 双层存储统一化（Plan 335 已分析，留后续）

## 4. 与相关计划的关系

| Plan | 关系 |
|------|------|
| 335（List 存储双轨分析） | 分析了 `ListData<i32>` vs `ListData<Value>` 的根因 |
| 338（List 部分修复） | 已经修了 push/len + for-in workaround |
| 339（Symbol 命名空间） | 独立，互不影响 |

**建议顺序**：Symbol 命名空间（339）→ List 完整方法（340），因为 339 是架构基础，340 是功能增强。但两者可独立实施。
