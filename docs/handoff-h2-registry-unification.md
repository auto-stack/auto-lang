# 交接：Plan 390 §15 Phase H2 — VM 对象 registry 统一（方案 B 核心）

> **创建**：2026-08-07。承接 Plan 390 §15 Phase H（`docs/plans/390-actor-state-injection.md`）。
> H1 已完成并 push（`23422b97`）；本交接覆盖 H2（核心高风险步骤）+ H3（清理）+ H4（L3 打通）。
>
> **更新（2026-08-06，H2 已完成）**：H2 + H4 在分支 `plan-390/h2-registry-unify` 落地
> （commits `14103b51`/`9f2fba50`/`250faf64`），L3 多字段消息打通，回归零新增。详见 §四'
> 及计划文档 §15.7。**剩余：H3（arrays/nodes 物理迁移 + 删魔数）**。

## 一、背景（为什么做这个）

### 起因
Plan 390 的遗留 **L3（WithBindings 多字段消息）** 实施时发现 VM 有 **4 套对象 registry** 且栈编码不一致，
导致 `h.send(Add(3,5))` 的多字段消息全链路（send→mailbox→wake→handler）断裂。

### 4 套 registry（调研实证，2026-08-07）

| Registry | engine.rs 定义 | id 段位 | 栈编码 | 存什么 |
|---|---|---|---|---|
| `objects` | :260 `DashMap<u64, Arc<RwLock<ObjectData>>>` | 1,000,000+ | `push_i32(id)` 裸 i32 | CREATE_OBJ 产出的 ObjectData |
| `arrays` | :264 `DashMap<u64, Arc<RwLock<Vec<Value>>>>` | 2,000,000+ | `push_i32(id)` | CREATE_ARRAY |
| `nodes` | :268 `DashMap<u64, Arc<RwLock<Node>>>` | 3,000,000+ | `push_i32(id)` | CREATE_NODE |
| `heap_objects` | :273 `DashMap<u64, Arc<RwLock<dyn HeapObject>>>` | 4,000,000+ | `encode_object(id)` TAG_OBJECT | List/Map/GenericInstance/RustStdlib/BigInt |

**核心问题**：栈上 `push_i32(obj_id)` 与 scalar i32 无法区分 → 消费端靠**魔数判断**（`>= 4_000_000`）
+ **试探探测**（`objects.get(&id)` 试一下）。这反复制造 bug（L3 是最新一例）。

### 方案 B（统一）vs 补丁（WRAP_MSG）
- 补丁：新增 `WRAP_MSG` opcode 只解决 L3 一个症状，加深技术债。
- **方案 B（选定）**：统一到 `heap_objects`（唯一 registry）+ `encode_object`（唯一栈编码），根治。

## 二、已完成（H1，commit `23422b97`，已 push）

- `ObjectData`（types.rs:147）impl `HeapObject` + 新增 `TypeTag::ObjectData`。
- 这是纯加法（零行为变更）：不改栈编码、不删旧 registry、不改任何 consumer。
- 意义：ObjectData 现在可以存进 `heap_objects`（`dyn HeapObject`），为 H2 的 CREATE_OBJ 迁移铺路。

## 三、已落盘的 L3 代码（Step 1-4，commit `95c06ab7`，已 push）

以下代码在 master 上，在 H2 栈编码统一后即可工作：

| Step | 文件 | 内容 |
|---|---|---|
| 1 | `codegen.rs` | TaskDef 收集 `task_variants: HashMap<String, Vec<(String, Vec<String>)>>`（:335）+ `Add(3,5)` 在 `Expr::Call` 识别为 variant 构造，emit CREATE_OBJ 产出 `Obj{__variant, fields}`（:6245+）|
| 2 | `stdlib.rs` | `shim_task_send_vm`（:5977）用 `pop_nv` + `is_object` 区分 Obj/scalar，Obj 存为 `Value::VmRef` |
| 3 | `codegen.rs` | handler codegen WithBindings 遍历 bindings，DUP+GET_FIELD+STORE_LOC 绑定每个字段（:3748+）|
| 4 | `engine.rs` | 消息唤醒 VmRef push `encode_object`（:1653+）|

**当前阻塞点**：CREATE_OBJ 产出 `push_i32(obj_id)`（:2242）而非 `encode_object` → send 的 `is_object`
检测失败（栈顶是 i32 不是 object tag）。**H2 统一栈编码后此阻塞自动解除**。

测试用例已写好：`actor_state_tests.rs::actor_withbindings_multi_field`（:177+），待 H2 后通过。

## 四、待实施：Phase H2（核心高风险步骤）

### H2 目标
所有对象引用的栈编码从 `push_i32(id)` 统一为 `push_nv(encode_object(id))`。

### H2.1 — Producer 改动（~15 处，`push_i32(obj_id)` → `encode_object`）

所有把 registry id 以裸 i32 push 到栈的 opcode 路径。**精确清单**（行号基于 2026-08-07 master）：

| Opcode | engine.rs 行号 | 当前 push | 改为 |
|---|---|---|---|
| CREATE_OBJ | 2242 | `push_i32(obj_id)` | `push_nv(encode_object(obj_id as u32))` |
| CREATE_ARRAY | 2293 | `push_i32(array_id)` | 同上 |
| CREATE_NODE | 2042 | `push_i32(node_id)` | 同上 |
| CREATE_OK | 3133 | `push_i32(instance_id)` | 同上 |
| CREATE_ERR | 3142 | `push_i32(instance_id)` | 同上 |
| CREATE_LIST_INT/STR/BOOL | 3238/3247/3256 | `push_i32(list_id)` | 同上 |
| CREATE_LIST_*_INLINE | 3267/3277/3287 | `push_i32(list_id)` | 同上 |
| NEW_INSTANCE | 3328 | `push_i32(instance_id)` | 同上 |
| CONSTRUCT_INSTANCE 末尾 | 3478 | `push_i32(instance_id)` | 同上 |
| CREATE_TUPLE | 3866 | `push_i32(instance_id)` | 同上 |
| slice-result push | 3837 | `push_i32(new_id)` | 同上 |
| GET_FIELD VmRef 结果 | 4346, 4392 | `push_i32(vm_ref.id)` | 同上 |
| GET_ELEM VmRef 结果 | 4035, 4078 | `push_i32(r.id)` | 同上 |
| inject_value Obj/Array/Node/VmRef | 532, 544, 555, 559 | `encode_i32(id)` | `encode_object(id as u32)` |

**注意**：`arrays`/`nodes` 的迁移需要先给 `Vec<Value>`/`Node` impl HeapObject（或用 ListData/NodeData 包装），
或者保持它们的 registry 但只改栈编码。H2 可分两批：先 objects（ObjectData 已 impl）+ heap_objects 的
producer，再 arrays/nodes。

### H2.2 — 消费者改动

**A. 仅假设 i32 的消费者（~8 处，必须改，否则统一后会坏）**：

| Opcode | engine.rs 行号 | 当前代码 | 改为 |
|---|---|---|---|
| ARRAY_LEN | 2340-2366 | 仅 `is_i32` 分支 | 加 `is_object` 回退 |
| SET_ELEM | 4110 | `pop_i32() as u64` | `pop_nv` + `is_object`/`is_i32` 双解码 |
| CONSTRUCT_INSTANCE | 3348 | `pop_i32() as u64` | 同上 |
| GET_TUPLE_FIELD | 3874 | `pop_i32() as u64` | 同上 |
| CREATE_ERR 值 pop | 3139 | `pop_i32()` | 同上 |
| CALL_SPEC 类型名 is_i32 分支 | 4910 | 仅 `is_i32` | 加 `is_object` |
| TO_STR/STR_CAT/LT 原始 decode_i32 回退 | 3051, 3080, 6680 | 无 tag 检查 | 加 tag 检查 |

**B. 双解码消费者（~15 处，可简化但非必须）**：

GET_FIELD(4282-4297)、SET_FIELD(4150-4155)、GET_ELEM(3957-3961)、GET_GENERIC_FIELD(3543-3550)、
IS_VARIANT(3500-3508)、TYPE_TO_STR(2764-2771)、EQ/NE/GT(6597/6634/6691)、
CALL_SPEC 类型名(4910+4940)、CALL_SPEC to_string(5629)、CALL_SPEC List 分发(5274-5281)、
5 个数组分发点(5351/5371/5387/5428/5473)、decode_tagged_nv(472-485)。

这些已有 `is_object` + `is_i32` 双路径——H2 后可删除 `>= 4_000_000` 魔数判断，合并成单一
`is_object -> decode_object`。**但这一步可推迟到 H3（不影响正确性）**。

### H2.3 — 关键注意事项

1. **inject_value / decode_tagged_nv**（engine.rs:472-485, 532-559, 879-927）：这是 Value↔栈 的
   桥接。inject_value 的 Obj/Array/Node 分支用 `encode_i32(id)`，统一后改 `encode_object`。
   decode_tagged_nv 靠 `is_i32 && >= 4_000_000` + `is_object` 双路径——统一后简化为单一 `is_object`。

2. **GET_GENERIC_FIELD 的 `v >= 4_000_000` 硬编码**（engine.rs:3547）：这是对 id 段位的硬假设。
   统一后所有 id 都在 heap_objects（4M+），这个判断变得无意义——可删。但迁移期间（objects 的 1M id
   还在）需保留 `get_any_object` 式的路由。

3. **matcher 期望 `Value::Obj` 内联**（match_message_pattern_vm :1534）：它消费的是
   `decode_tagged_nv` 重建的 `Value::Obj`，不是 registry 引用。只要 decode 路径正确就无需改。
   send 存 `Value::VmRef`，wake 时 decode_tagged_nv 重建 `Value::Obj` 给 matcher。

4. **http_server.rs:312 和 stdlib.rs:7372 直接读 `vm.objects`**：迁移时一并改为 `heap_objects`。

### H2.4 验证

```bash
# 全量回归（必须零新增失败；22 个预存失败是 dstr/ark/vue/codegen_if）
cargo test -p auto-lang --lib 2>&1 | grep "test result:"

# 对象字段访问专项（最容易因栈编码变更而坏）
cargo test -p auto-lang --lib -- object field struct enum list map

# a2r 回归
cargo test -p auto-man --lib

# L3 多字段（H2 后应通过）
cargo test -p auto-lang --lib actor_withbindings_multi_field
```

## 四'、H2 完成纪要（2026-08-06，分支 `plan-390/h2-registry-unify`）

H2 + H4 已落地，L3 多字段消息打通。**实际实施与本文档原计划的两点偏差**（详见计划 §15.7）：

1. **H1 漏交 `get_any_object`**：H1 commit `23422b97` 只做了 `impl HeapObject for ObjectData`，
   本文档 §四"H2.1"列的 `get_any_object` 并不存在。H2 Step 0 补上（engine.rs:734，按 id 段路由，
   当前仅覆盖 heap_objects 4M+ 段；objects/arrays/nodes 待 H3 物理迁移后并入）。
2. **G2 handler 帧结构性 bug**：本文档假设"H2 栈编码统一后 L3 Step 1-4 自然打通"。实测 G2 方案 A
   （handler_frame_base 仅复位 sp、bp=0）在 WithBindings ≥2 bindings 时 handler locals（bp+1..）
   与 message/表达式临时值重叠 → `Add(3,5)` 算出 10 而非 8。修复：消息唤醒前预留
   `HANDLER_LOCALS_BAND=16` 槽位（engine.rs `run_task_loop`），handler_frame_base 锚定其上。
   对单变量 G2 行为不变（已通过测试不受影响）。

**实施批次**（每批改 → 编译 → 回归）：
| Commit | 内容 | 回归 |
|---|---|---|
| `14103b51` | H2.1 objects 栈编码统一 + matcher VmRef 重水化 + handler 帧修复（解锁 L3） | 22 failed（基线 23，L3 转为通过，零新增）|
| `9f2fba50` | H2.2 heap_objects producer 统一（Result/List/Instance/Tuple）+ 受影响消费者 | 22 failed（零新增）|
| `250faf64` | H2.3 VmRef re-push 统一（GET_ELEM/GET_FIELD 6 处）+ inject_value VmRef | 22 failed（零新增）|

**未改（H3 推迟）**：CREATE_ARRAY/CREATE_NODE/POP_ACCUM/SLICE 仍 `push_i32`（arrays/nodes 物理存储 +
栈编码均不动）；所有 `>= 4_000_000`/`>= 1_000_000` 魔数判断保留；`decode_tagged_nv` 双路径保留；
IS_OK/UNWRAP_*/STR_CAT/LT 回退保留。

**H3 入口**（清理，H2 验证通过后）：见本文档 §五原计划，加两项 H2 实施时确认的收尾——
- `get_any_object` 接管所有消费者（替代显式查 objects/arrays/nodes）。
- IS_OK/UNWRAP_OK/UNWRAP_ERR 改严谨（当前靠 decode_i32(encode_object(id)) 低 32 位 == id 巧合工作）。



- `CREATE_OBJ`/`CREATE_ARRAY`/`CREATE_NODE` 改为 `insert_heap_object`（走 heap_object_id_gen）。
- 删除 `objects`/`arrays`/`nodes` 字段 + `object_id_gen`/`array_id_gen`/`node_id_gen`。
- 删除所有 `>= 1_000_000`/`>= 4_000_000` 魔数判断。
- 验证：全量回归零新增失败。

## 六、待实施：Phase H4（L3 打通，H2 后自动生效）

H2 完成后，L3 的 Step 1-4 代码自然打通（producer 统一 encode_object → send is_object 检测成功 →
mailbox VmRef → wake push encode_object → handler GET_FIELD 绑定）。

测试用例（§15.4 的 #7-#11）：
- 单字段 WithBindings：`h.send(Add(3))` → 输出 3
- 多字段：`h.send(Add(3,5))` → 输出 8
- 多次 send 不 stale
- Simple variant：`h.send(Reset)`
- 混合 pattern

## 七、关键文件清单

| 文件 | 角色 |
|---|---|
| `crates/auto-lang/src/vm/engine.rs` | H2 核心：4 个 registry 定义 + 所有 producer/consumer opcode |
| `crates/auto-lang/src/vm/heap_object.rs` | HeapObject trait + TypeTag（H1 已加 ObjectData）|
| `crates/auto-lang/src/vm/types.rs` | ObjectData 定义（H1 已 impl HeapObject）|
| `crates/auto-lang/src/vm/codegen.rs` | L3 Step 1+3（task_variants + variant 构造 + handler 绑定）|
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | L3 Step 2（send shim）+ http_server/stdlib 直接读 objects |
| `crates/auto-val/src/nano_value.rs` | encode_object/is_object/decode_object 定义 |
| `crates/auto-lang/src/tests/actor_state_tests.rs` | L3 测试用例 |
| `docs/plans/390-actor-state-injection.md` | §15 Phase H 完整方案 |

## 八、建议实施顺序

1. **建分支** `plan-390/h2-registry-unify`。
2. **H2.1 先做 objects（ObjectData 已 impl）**：CREATE_OBJ 改 `insert_heap_object` + `encode_object`；
   inject_value Obj 分支改 `encode_object`。**验证**：对象字段访问测试不坏。
3. **H2.2 改"仅假设 i32"消费者**（GET_FIELD/SET_FIELD 已有双解码可不动；重点 ARRAY_LEN/SET_ELEM/
   CONSTRUCT_INSTANCE/GET_TUPLE_FIELD）。**验证**：全量回归。
4. **H2.3 验证 L3**：跑 `actor_withbindings_multi_field`，应通过。
5. **H2.4 做 heap_objects 的 producer**（NEW_INSTANCE/CREATE_LIST 等已有的 push_i32 改 encode_object）。
6. **H3 清理**（arrays/nodes 迁移 + 删旧 registry + 删魔数）。
7. **H4 L3 完整测试**。

**风险控制**：每改一批 producer 就跑一次回归，定位栈错位。最容易出问题的是 inject_value/
decode_tagged_nv（Value↔栈桥接）和 GET_FIELD（最频繁的字段访问）。

## 九、当前仓库状态

- 分支：`master`（`23422b97`），已 push。
- L3 分支已合并删除。
- 工作区干净。
- 预存失败：22 个（dstr/ark/vue/codegen_if），与本次无关。
