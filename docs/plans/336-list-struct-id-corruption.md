# Plan 336：List<Struct> 元素 ID 损坏 — nanbox tagging 修复

> **For Claude:** 本计划源于 015-notes `--render=vm`:notes 列表项渲染不出来。Plan 335 修了两层 VmRef 解引用(read_state_as_vec + materialize_obj_ref),数据 len=3 到达视图层,但每个 note 项的 ID 被损坏,`note.title` 解析失败。本计划定位并修复上游的 nanbox tagging 损坏。

## 触发现状(诊断已确认)

Plan 335 后,渲染层数据流:
- `read_state_as_vec("notes")` 返回 3 个元素 ✅(VmRef → ListData<Value> deref)
- 每个 note 项 = `Value::Int(3000000/3000001/3000002)` ❌
- `materialize_obj_ref(Int(3000000))` 查 `vm.objects` 失败 → 不 materialize → `note.title` 解析不出 → 列表项空白

诊断 dump(VM 运行时):
```
objects      = [1000000, 1000001, 1000002]   ← Note 实例真实存在,id 1M 段
heap_objects = [4000000, 4000001]             ← ListData(state + notes 列表)
arrays       = [2000000]
note 项 id  = Int(3000000/1/2)               ← 但列表里存的是 3M 段!
```

**关键矛盾**:Note 实例在 `objects` 是 1M(`object_id_gen` 从 1000000 起),但列表元素是 3M(`node_id_gen` 从 3000000 起)。**2M 的偏移 = 数据损坏**。

## 根因分析(三层,逐层确认)

### 根因①：CREATE_OBJ 用 push_i32 而非 push_nv(encode_object)

`engine.rs:1658`:
```rust
let obj_id = self.object_id_gen.fetch_add(1, ...);  // 1M 段
self.objects.insert(obj_id, ...);
task.ram.push_i32(obj_id as i32);   // ← 用 push_i32,编码为 encode_i32(非 object tag)
```

Note 的 1M id 被压栈时用 `push_i32`(裸 i32 编码),**未打 object tag**。

### 根因②：CREATE_ARRAY 读取时 is_object 判定为 false

`engine.rs:1671-1685`(CREATE_ARRAY 元素解码):
```rust
let value = if is_string(nv) { ... }
    else if is_object(nv) { Value::VmRef(decode_object(nv)) }   // ← 期望 object tag
    else if is_null(nv) { ... }
    else { Value::Int(decode_i32(nv)) };   // ← 落到这里(因未打 object tag)
```

因 Note id 用 `encode_i32` 压栈(非 `encode_object`),`is_object(nv)` 为 false,落到 `Value::Int(decode_i32(nv))`。**此处 decode 应返回 1M**,但……

### 根因③：3M 偏移的真正来源(待 Phase 0 确认)

`decode_i32(encode_i32(1M))` 应返回 1M,但列表里是 3M。可能的来源(需 Phase 0 实测确认):
- (a) Note 对象 id 实际不是 1M,而是经过某层后变成 3M(某 id_gen 混用)
- (b) `object_keys`/`object_types` 元数据类型错误,CREATE_OBJ 把 Note 当 Array/NestedObject 处理,走了 `Value::VmRef(id)` 但 id 来自 node_id_gen
- (c) `shim_list_new` 的 struct 分支(native.rs:922)在 clone array 时,元素已被前一步错误标记

**Phase 0 必须实测**:在 CREATE_OBJ(1658)和 CREATE_ARRAY(1684)各加一行 `eprintln` 打印实际 id/nv/tag,确认 1M 在哪一步变 3M。

## 解决方案

### Phase 0 — 精确定位 3M 来源(不动代码,< 30 min)

在 `engine.rs` 加临时诊断:
1. `CREATE_OBJ`(1658 后):`eprintln!("CREATE_OBJ id={} tag=object", obj_id)`
2. `CREATE_ARRAY`(1684 前):`eprintln!("CREATE_ARRAY elem nv={} is_object={} decode_i32={} decode_object={}", nv, is_object(nv), decode_i32(nv), decode_object(nv))`
3. `shim_list_new`(native.rs:912):打印收到的 array_id 和数组元素

跑 `auto run -r vm`,看 1M 在哪步变 3M。

**产出**:本计划追加「Phase 0 结论」,锁定确切损坏点。

### Phase 1 — 修复 CREATE_OBJ 对象标记

**文件**:`crates/auto-lang/src/vm/engine.rs`(CREATE_OBJ,~1658)

`object_id_gen` 分配的 id(1M 段)是**对象引用**,应压栈为 object tag,而非裸 i32:

```rust
// 之前:task.ram.push_i32(obj_id as i32);
// 之后:对象 id 用 object tag 编码,让 CREATE_ARRAY/GET_FIELD 等正确识别
task.ram.push_nv(auto_val::encode_object(obj_id as u32));
```

**风险**:`push_i32` 被多处读取代码假设(如 GET_FIELD 期望 i32)。需确认 GET_FIELD/CALL_SPEC 对 object tag 的解码路径。可能需要双向兼容:读取时既认 `is_object` 也认 `is_i32 >= 1000000`。

### Phase 2 — 双向兼容读取(若 Phase 1 破坏既有)

若 Phase 1 改 push 导致 GET_FIELD/现有测试回归,改为在**读取侧**兼容:CREATE_ARRAY 等读取时,对 `is_i32(nv) && decode_i32(nv) >= 1000000 && < 2000000` 也识别为对象引用(1M 段约定)。

### Phase 3 — 端到端验收 + 回归

- [ ] 015-notes vm:列表渲染 3 条 note(标题可见),`note.title`/`note.id` 字段访问正常
- [ ] 016-calendar vm 回归(int List 不受影响)
- [ ] handler_codegen 5/5
- [ ] 现有 struct 测试(field_access_tests 等)不回归
- [ ] `materialize_obj_ref` 对 Int(1M) 正确 materialize(查 vm.objects)

## 与已有修复的关系

| Plan/commit | 修了什么 | 状态 |
|---|---|---|
| `f21e7774` | `to_array()` identity | ✅(struct List 正确,int List 待 Phase 2) |
| Plan 335 Phase 1(`92fc3ee8`) | `read_state_as_vec` VmRef deref | ✅(len=3 到达) |
| `53b34c91` | `materialize_obj_ref` VmRef deref | ✅(必要,处理 GenericInstanceData VmRef) |
| **本计划 Plan 336** | **上游 nanbox tagging(1M→3M 损坏)** | ⚠️ **当前渲染阻塞点** |

前三个让数据流到视图层,但元素 ID 损坏导致 `materialize` 失败。本计划是打通渲染的最后一环。

## 风险与边界

- **nanbox tag 契约**:AutoVM 的 id 段约定(1M objects / 2M arrays / 3M nodes / 4M heap)是隐式契约,没有强类型保护。任何改 tag 的修复都要确认全链路(GET_FIELD/CALL_SPEC/iterator_next/push 等)的一致性。
- **范围**:仅修 List<Struct> 元素的 ID 标记/读取。不改 nanbox 编码本身(那是更大重构)。
- **回归面广**:CREATE_OBJ 是所有结构体字面量的入口,改动影响所有 struct 用法。Phase 3 必须跑全量 struct 相关测试。

## 验收标准(Definition of Done)

1. 015-notes vm 模式:列表渲染 3 条 note,标题可见
2. `note.title` / `note.id` 字段访问在 for 循环体内工作
3. CREATE_OBJ 创建的对象 id 在列表/数组中保持正确(无 1M→3M 损坏)
4. 016-calendar、struct 测试、handler_codegen 全部不回归
5. nanbox tag 在对象 id 路径上一致(object tag 或 1M 段约定二选一,文档化)
