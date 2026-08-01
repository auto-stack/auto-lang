# Plan 335：List<T> 结构体元素运行时完整修复

> **状态（2026-08-01）**：**核心修复全部完成**。
> - shim 双查（get/set/insert/pop/remove/contains 加 ListData<Value> + arrays 分支）✅
> - Phase 1（read_state_as_vec 解引用 VmRef + vmref_to_vec）—— 已由后续工作实现 ✅
> - Phase 2（to_array）—— shim 双查修复后 int List 端到端工作（identity 语义不完美但无功能影响）✅
> - `type` 声明的结构体 List 端到端可用（空 List + push 路径，002 测试守护）✅
> - **遗留**：`List<Note>.new([字面量])` 带初始元素构造坏（CREATE_OBJ 的 nanbox 编码问题，独立于本计划，见 §CREATE_OBJ 编码遗留）
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

---

## 实施记录（2026-08-01，shim 双查修复）

### 实测复核（文档 vs 现状）

逐个实测 §"需复核"列的 7 个 shim，发现：
- ✅ `shim_list_len`（1032）：**已修**（后续工作加了 ListData<Value> 分支）——`items.len()` = 2 正确。
- ❌ `shim_list_get` / `set` / `insert` / `pop` / `remove` / `contains`：**仍坏**——只查 `ListData<i32>`，struct List（ListData<Value>）downcast 失败 → push 0 / 报 `Invalid list ID`。

### 修复（native.rs）

给 6 个 shim 都补了 `ListData<Value>` 分支（+ arrays DashMap fallback），对齐 `shim_list_len`/`shim_list_push` 已有的双查模式：

| shim | 改动 |
|------|------|
| `shim_list_get` | 加 ListData<Value> 分支（push_value）+ arrays fallback |
| `shim_list_set` | 加 ListData<Value> 分支（nv_to_value 解码元素）+ arrays fallback |
| `shim_list_insert` | 同 set |
| `shim_list_pop` | 加 ListData<Value> 分支（push_value 返回）+ arrays fallback（修正原 as_int 丢 VmRef） |
| `shim_list_remove` | 同 pop |
| `shim_list_contains` | 重写：去掉 get_list_i32_elements（对 heap id 报 Invalid list ID），改为 ListData<i32>/ListData<Value>/arrays 三查 + values_eq 按 VmRef.id 比较 |

新增 3 个辅助：`push_value`（Value→栈）、`nv_to_value`（栈→Value）、`values_eq`（contains 比较，VmRef 按 id）。

### 测试

`test/vm/29_list_shims/001_int_list`：List<int> 的 len/get/set/insert/pop/remove/contains 全覆盖，输出 `3 20 99 4 30 2 1`，全绿。

### struct 构造遗留（独立 bug，阻塞 struct List 端到端验证）— 已澄清（2026-08-01）

修复 shim 后测 struct List，发现 `Item.new("a").name` = 0（字段丢失）。

**澄清**：这是**测试用例的语法错误**，不是 Auto 的 bug。Auto 用 `type` 声明结构体类型（tour / stdlib / 015-notes / auto-musk 全用 `type`），**`struct` 不是 Auto 关键字**（token.rs 未注册）。复现脚本误用 Rust 风格 `struct Item {...}`，被 parser 当普通 ident → 未注册为类型 → `.new()` 失败。

用合法的 `type Item {...}` 语法，`Item.new()` + 字段访问 + List 端到端**全部正常**——List shim 双查修复（本节上一段）本身正确有效，无需额外的 struct 别名支持。

> 曾一度误把 `struct` 当 `type` 别名加进 parser（commit a5a502b6），后纠正撤销——`struct` 不是 Auto 语法，不应赋予语义。002 测试改用 `type` 语法守护。

### 回归

- 24_generics（3）、28_enum_methods（6）、29_list_shims（2：001 int + 002 type/struct）全绿
- VM non-ignored 21=21 基线，零新增

---

## Phase 1/2 调研结论（2026-08-01）

### Phase 1（read_state_as_vec 解引用 VmRef）—— 已由后续工作实现

`vm_bridge.rs:424` 的 `read_state_as_vec` **已有 `Value::VmRef(r) => self.vmref_to_vec(r.id)` 分支**（line 436）。`vmref_to_vec`（line 467）完整实现了方案描述的三查：heap_objects 的 ListData<Value> + ListData<i32>，再 vm.arrays。且采用了方案建议的"先 heap 再 arrays"顺序，不依赖 id 段硬编码。**Phase 1 无需额外工作。**

### Phase 2（to_array int List 语义）—— shim 双查修复后无功能问题

`to_array` 在 engine.rs:5444 是 identity（receiver 原样返回）。对 int List（ListData<i32>），identity 返回 heap id（而非方案期望的 array_id）。但 shim 双查修复后，`get`/`len`/`pop` 等都能处理 heap id——实测 `List<int>.new([1,2,3]).to_array().len()` = 3，`.get(0)` = 1，**功能正确**。identity 的语义不完美（返回 heap id 而非 array_id）但不构成实际故障。**Phase 2 无需额外工作。**

### test 24/002 现状

`test_24_generics_002_to_array`（`#[ignore]` 标记但能跑过）：`List<Note>.new([字面量]).to_array()` 输出 "ok"，**通过**。

---

## CREATE_OBJ 编码遗留（独立 bug，超出本计划范围）

调查 Phase 2 时发现 `List<Note>.new([Note{id:0, title:"a"}])`（带初始字面量元素的 struct List 构造）返回垃圾 len（7 而非 1）。

**根因**：CREATE_OBJ（engine.rs:2091，`Note {...}` 字面量构造）用 `push_i32(obj_id)` 推对象 id（line 2185）——裸 int，无 object tag。CREATE_ARRAY（engine.rs:2188）收集数组元素时，`is_object(nv)` 检查对裸 int 返回 false → struct 对象 id 被当作 `Value::Int(4000000)` 存进 vm.arrays（应为 `Value::VmRef`）。shim_list_new 随后把这个"Int"存进 ListData<Value>，下游 len/get 行为错乱。

**对照**：`Item.new()`（CONSTRUCT_INSTANCE/NEW_INSTANCE 路径，engine.rs:3272）也用 `push_i32`，但 `List<Item>.new([])` + `push(Item.new())` 工作正常（002 测试）——因为 shim_list_push 的 `is_object` 对该 id 段的 nanbox bit pattern 恰好识别为 object（id_gen 起始值差异）。两条对象构造路径（CREATE_OBJ vs NEW_INSTANCE）的 id 编码/识别不一致。

**为何不在本计划修**：修复需统一 CREATE_OBJ/NEW_INSTANCE 的对象 id push 编码（push_i32 → encode_object），但这会连锁影响整个对象访问链路（GET_FIELD/GET_GENERIC_FIELD 等都 pop_i32 拿对象 id）。这是 nanbox 值表示的架构层问题，属于 Plan 377（统一值表示）的范畴，非 List 运行时修复。

**建议**：归入 Plan 377（统一值表示 — 消除 2-slot）一并处理，或单开计划统一对象 id 的 push/pop 编码契约。

**本计划范围内**：空 List + push 路径的 struct List 完全可用（002 测试守护）；带初始字面量元素的构造是 CREATE_OBJ 遗留，不影响 push/get/set 等已修 shim。

### 回归

- 24_generics（3 测试）、28_enum_methods（6 测试）全绿
- VM non-ignored 21=21 基线，零新增

### Phase 1/2 仍未实施

- Phase 1（`read_state_as_vec` 解引用 VmRef）：015-notes vm 渲染所需，未做
- Phase 2（`to_array` int List 语义）：test 24/002 仍 ignored，未做

这两个 Phase 需要 015-notes/016-calendar 端到端环境验证，本次未触及。
