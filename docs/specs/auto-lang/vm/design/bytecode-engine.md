# 字节码与执行引擎

## 范围

ABC（AutoByteCode）指令集、编码约定、AutoVM 执行引擎、虚拟内存模型与栈帧不变量。对应代码：`vm/opcode.rs`、`vm/engine.rs`、`vm/task.rs`、`vm/virt_memory.rs`。

## 原则

- ABC 是编译后端与 VM 之间的契约：栈式、1 字节 opcode + 变长小端操作数，为 flash XIP 场景优化（design/05）。
- VM 模式无关：SCRIPT/CONFIG/TEMPLATE 差异全部在 codegen 侧消化（ADR-07）。
- 值表示统一为 NaN-boxed u64（`NanoValue`，ADR-06），f64 直存零开销。

## 细节

### 指令集

`OpCode` 共 178 个变体（`opcode.rs`，design/05 称"约 120 个"已过时）。按区间分类：

| 区间 | 类别 | 代表指令 |
|------|------|---------|
| 0x00-0x0F | 栈操作 | NOP, POP, DUP, SWAP |
| 0x10-0x1F | 常量 | CONST_I32, CONST_F64, LOAD_STR |
| 0x20-0x2F | 变量/对象 | LOAD_LOCAL, STORE_LOCAL, GET_FIELD, CREATE_ARRAY |
| 0x30-0x4F | 算术逻辑 | ADD..MOD（含 I32_TO_F32 0x46 / I64_TO_F64 0x47 转换） |
| 0x50-0x5F | 比较 | EQ, NE, LT, GT, LE, GE |
| 0x60-0x6F | 控制流 | JMP, JMP_IF_Z, JMP_IF_NZ, JMP_L |
| 0x70-0x7F | 调用/数据 | CALL, RET, CALL_NAT, BUILD_FSTR, ERROR_PROPAGATE |
| 0x80-0x8F | 并发 | SPAWN 0x80 .. TRY_RECV 0x88 |
| 0x90+ | 闭包 | CLOSURE 0x90, LOAD_CAPTURED 0x92, STORE_CAPTURED 0x93, CALL_CLOSURE 0x94 |
| 0xE0-0xFF | 扩展 | Option/Result、类型转换、RET_D（双槽返回） |

编码约定：RET 为 callee-cleanup（带参数个数清栈）；跳转均为 16 位有符号相对偏移，位置无关；CALL_NAT 索引 native 函数表（design/05）。类型强转由 codegen 在混合算术时显式插入（plan-117：int/float 混合运算不加 I32_TO_F32 会把 int 位模式当 float 解释）。

### 内存模型（数字孪生）

- `VirtualFlash`（virt_memory.rs:36）：只读 `Vec<u8>` 代码区 + 常量，附 `object_keys`/`object_types` 元数据，模拟 MCU XIP。
- `VirtualRAM`（virt_memory.rs:247）：每任务读写栈，`raw_nv: Vec<NanoValue>`，`sp` 越界时按 2 倍（最小 256）扩容。`raw: Vec<i32>` 是 nanbox 迁移残留，不再承载运行时数据。
- 堆对象不进 VirtualRAM，全部走 `AutoVM` 上的 DashMap 注册表（见 generics-heap.md）。

### 执行引擎

- `AutoVM`（engine.rs:232）= 共享运行时：flash、字符串池、native 接口，以及 tasks/channels/iterators/closures/objects/arrays/nodes/heap_objects/futures/task_mailboxes/globals 等注册表，外加 debugger、trace、output_buffer、generic_registry、host 桥。
- `AutoTask`（task.rs:37）= 每任务上下文：ram、ip/bp/num_locals、status（Ready/Running/Waiting/Terminated）、wake_time、闭包 ID 对、FN_PROLOG 元数据、last_result_type、last_error、消息循环字段、actor state_vars、调试 call_stack、try/catch handler_stack。
- 派发循环：`run_task_loop`（engine.rs:1334）驱动任务；`run_one_instruction`（engine.rs:1574）执行单条并返回 `StepResult`，供调试器单步。

### 栈帧不变量

- 函数入口为局部变量压入 dummy CONST_0，使 `sp` 从 `n_locals` 起算（plan-080：bp=0 时栈与局部变量共享区域的 bug 修复）。
- 语句级栈平衡：每条语句执行后 `sp == bp + num_locals`；表达式结果打印后弹出（design/05 §Streaming）。
- 闭包调用：CALL_CLOSURE 设 `current_closure_id`，RET 恢复 `saved_closure_id`，支撑嵌套闭包（plan-071）。

## 显式非目标

- MicroVM 的 C99 实现、Tier-2 JIT、AutoLive 热重载（GOT patch）：均未实现，见 design/05 Open Questions，不在本模块代码内。
- ART 跨平台 shim 层（`art_*` C API）是设计愿景，本模块无对应代码。

> 来源: docs/design/05-vm-runtime.md（§ABC、§AutoVM、§Streaming）；docs/plan-reports/07-vm-runtime.md（§Core Engine、§Bug Fixes）；crates/auto-lang/src/vm/{opcode,engine,task,virt_memory}.rs；plan-068/080/117/221/298
