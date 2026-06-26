# Plan 335：List<T> 结构体元素运行时完整修复

> **For Claude:** 本计划源于 015-notes `--render=vm`：notes 列表渲染为空。已修根因①(`to_array` 未实现,commit `f21e7774`),但列表仍空——根因②是渲染层 `read_state_as_vec` 不解引用 `VmRef`。本计划做 List<T>(T=结构体/混合类型)的**完整运行时语义修复**,并扫查 VM 中其它同类缺口。

## 触发现状（015-notes vm+vm 合并模式）

```
Init: .notes = list_notes()    # list_notes → db.all_notes → notes.to_array()
```

诊断数据(commit f21e7774 前后):
- 修复前:`state.notes = Int(0)`(nil,因 `to_array` 未实现,push nil)
- 修复后:`state.notes = VmRef { id: 4000001 }`(to_array identity 返回接收者,数据到达 state)
- 渲染:`read_state_as_vec("notes")` **报错**——它只认 `Value::Array` 和 `Value::Int(id) >= 2000000`,**不认 `Value::VmRef`** → 列表渲染为空

## 根因分析

### List<T> 的双重存储机制（关键背景）

`shim_list_new`(native.rs:888)按元素类型分两条路:

| 元素类型 | 存储位置 | 返回的 id | nanbox 解码 |
|---|---|---|---|
| 全 int | `heap_objects`(`ListData<i32>`) | heap id(4000000+ 段) | `is_object` → VmRef |
| 含 struct/str/混合 | `vm.arrays`(`Vec<Value>`) | array_id(2000000+ 段) | `is_i32` → Int |

> 注意:这两种 id 段(4000000 heap / 2000000 arrays)**没有统一**。一个方法要支持两种存储,就得同时查 `heap_objects` 和 `vm.arrays`。这是所有"struct 不工作"的统一根因:**方法只查了一种存储**。

### 根因①：`to_array()` 未实现（已修,f21e7774）

`CALL_SPEC` 分发的 identity-ops 列表(engine.rs:4787)不含 `to_array`,掉进"未知方法→push nil"分支。修复:加入 identity 列表(对 `vm.arrays` 存储的 List,接收者就是 array_id,identity 正确)。

**遗留**:对 `ListData<i32>` 存储的 List,`to_array` 应把 int 元素转成 `vm.arrays` 的数组并返回 array_id——目前 identity 返回 heap id,语义不完全对(int List.to_array() 会返回一个 ListData 的 heap id 而非数组)。低优先级,因为 015-notes 是 struct List。

### 根因②：`read_state_as_vec` 不解引用 VmRef（本计划核心）

```rust
// vm_bridge.rs:275
pub fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<Value>> {
    let val = self.read_state(field_name)?;
    match val {
        Value::Array(arr) => Ok(arr.values),
        Value::Int(id) if id >= 2000000 => { /* 读 vm.arrays */ }
        other => Err(...),   // ← VmRef 走这里,渲染失败
    }
}
```

`Value::VmRef { id }` 不在匹配里。`List<Note>` 的 struct 存储返回 VmRef(heap id),`read_state_as_vec` 无法读取。**修复**:加 `Value::VmRef` 分支,按 id 段分别查 `heap_objects`(ListData<Value> 或 ListData<i32>)和 `vm.arrays`,转成 `Vec<Value>`。

### 根因③：`shim_iterator_next` 已支持 struct(验证 — 非问题)

`for note in notes` 走 `shim_iterator_next`(native.rs:1670)。**它已经支持 struct**:1697-1716 行有 `vm.arrays` fallback,struct 元素正确编码为 VmRef.id(int)。所以 **for-in 迭代 struct List 本身工作**——渲染缺口的唯一原因是 `read_state_as_vec`。

## 解决方案

### Phase 1 — `read_state_as_vec` 解引用 VmRef(核心,小改)

**文件**:`crates/auto-lang/src/ui/vm_bridge.rs`(`read_state_as_vec`,~275 行)

加 `Value::VmRef` 分支,按 id 段分别解引用:

```rust
pub fn read_state_as_vec(&self, field_name: &str) -> Result<Vec<Value>> {
    let val = self.read_state(field_name)?;
    match val {
        Value::Array(arr) => Ok(arr.values),
        Value::Int(id) if id >= 2000000 => self.array_to_vec(id as u64),
        Value::VmRef(r) => self.vmref_to_vec(r.id),   // ← 新增
        other => Err(VmBridgeError::InvalidState(format!(
            "Expected array for field '{}', got {:?}", field_name, other))),
    }
}

// 新增辅助:VmRef → Vec<Value>(按 id 段分别处理)
fn vmref_to_vec(&self, id: usize) -> Result<Vec<Value>> {
    // arrays 段(2000000+)
    if id >= 2_000_000 && id < 4_000_000 {
        return self.array_to_vec(id as u64);
    }
    // heap_objects 段(4000000+): ListData<i32> 或 ListData<Value>
    if let Some(obj) = self.vm.get_heap_object(id as u64) {
        let guard = obj.read().unwrap();
        if let Some(list) = guard.as_any().downcast_ref::<ListData<i32>>() {
            return Ok(list.elems.iter().map(|i| Value::Int(*i)).collect());
        }
        if let Some(list) = guard.as_any().downcast_ref::<ListData<Value>>() {
            return Ok(list.elems.clone());
        }
    }
    Err(VmBridgeError::InvalidState(format!("VmRef {} not a readable list", id)))
}
```

**验收**:015-notes vm 模式列表渲染出 3 条 notes(Welcome / Shopping List / Meeting Notes)。

### Phase 2 — `to_array()` 对 int List 的正确语义(中优先级)

当前 identity 对 struct List 正确(返回 array_id),但对 `ListData<i32>` 返回 heap id——语义应为"转成数组返回 array_id"。

**文件**:`crates/auto-lang/src/vm/engine.rs`(CALL_SPEC,~4787 identity 分支)

把 `to_array` 从 identity 列表移出,加专门分支:
- 若接收者在 `heap_objects` 是 `ListData<i32>`:把元素拷进新 `vm.arrays` 条目,push 新 array_id
- 若接收者在 `vm.arrays`(struct List):identity(返回 array_id)
- 否则:identity(兜底)

**验收**:取消 `test 24_generics/002_to_array` 的 `#[ignore]`,通过。

### Phase 3 — 回归 + 端到端

- [ ] 015-notes vm+vm 合并:列表渲染 3 条,点击切换/新建/删除/保存
- [ ] 016-calendar vm 回归(纯 int List):窗口正常、网格渲染正常
- [ ] `to_array` 单测(Phase 2 后取消 ignore)
- [ ] handler_codegen 5/5
- [ ] vue/rust 模式不受影响(不同代码路径)

## 其它类似问题扫描

### ✅ 已验证无问题
- **`for note in notes`**(struct 迭代):`shim_iterator_next` 的 `vm.arrays` fallback(1697-1716)已支持 struct → VmRef。**工作正常**。
- **`notes.push(note)`**(struct push):`shim_list_push`(954)有 `vm.arrays` fallback + 完整 Value 解码(959-972)。**工作正常**。
- **`note.id`(struct 字段访问)**:GET_FIELD 通过 heap object id 读取 GenericInstanceData。**工作正常**(Plan 326/333 验证)。

### ⚠️ 需复核(可能只对 int 工作)
逐个检查下列 shim 的 `heap_objects`/`arrays` 双查:
- `shim_list_pop`(999)、`shim_list_get`(1114)、`shim_list_set`(1142)、`shim_list_insert`(1165)、`shim_list_remove`(1198)、`shim_list_len`(1032)、`shim_list_contains`(1517)

**`shim_list_len` 初查**(1032):需确认它查 `vm.arrays`(`Vec<Value>` 的 `.len()`)还是只查 `ListData<i32>.len()`。若只查后者,`notes.len()`(struct List)会返回错误长度。**Phase 3 验收时复核**——015-notes 的 `.notes.len() > 0`(app.at:38)若失败,正是此问题。

### ⚠️ identity-ops 列表复核(engine.rs:4787)
当前 identity 列表:`collect | rev | filter_map | flatten | into_iter | iter | iter_mut | par_iter | par_iter_mut | for_each | map | filter | find | any | all | reduce | fold | to_array`。

其中 `map | filter | find | any | all | reduce | fold` 对 struct List 可能不应 identity(它们应真正执行谓词)。但这些走的是 `Iterator::Map/Filter`(native.rs Iterator 分支),identity 只是"返回接收者"作为链式调用占位——需确认是否在别处真正执行。**Phase 3 复核**——015-notes 的 `notes.filter(...)`(db.at:66)若行为异常,正是此问题。

### ✅ 已知大规模 `#[ignore]`(非 List 相关,记录)
`a2c_tests.rs` 整文件 `#[ignore]`(C 转译器测试,与 List 无关)。不在本计划范围。

## 风险与边界

- **VmRef id 段假设**:方案依赖 id 段约定(2M arrays / 4M heap)。若 `array_id_gen`/`heap_object_id_gen` 起始值变化,`vmref_to_vec` 的段判断需同步。建议:Phase 1 用 `heap_objects.get(id)` 先查 heap,失败再查 arrays,避免硬编码段值(更稳健)。**Phase 1 实现时采用此顺序而非段判断**。
- **数据所有权**:`read_state_as_vec` 返回 `Vec<Value>` 的 clone(`ListData<Value>.elems.clone()`),不影响 VM 内部状态。
- **范围**:仅修 List<T> 运行时 + 渲染层 VmRef。不改存储机制统一化(那是更大的重构)。

## 验收标准(Definition of Done)

1. 015-notes vm+vm 合并模式:列表渲染出 3 条 notes,CRUD 正常
2. `read_state_as_vec` 支持 `Value::VmRef`(heap + arrays 两种解引用)
3. `to_array` 对 int List 语义正确(Phase 2,取消 test 002 ignore)
4. 016-calendar 回归正常(struct 字段访问、int List 渲染)
5. `notes.len()` / `notes.filter()` 在 struct List 上行为正确(Phase 3 复核)
