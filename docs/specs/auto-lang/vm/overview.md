# vm（AutoVM）

> **Status**: implemented

## 职责

AutoVM 是 AutoLang 的默认执行后端，也是唯一可用的解释执行后端（plan-081 设为默认，plan-091 起 evaluator 选项弃用并重定向到 AutoVM）。职责覆盖：AST → ABC 字节码编译（codegen）、栈式字节码执行引擎、堆对象统一管理、泛型单态化、Task/Msg 并发与 async/await、native/FFI 接口、交互式调试器、ABT 字节码文本格式，以及 REPL / 持久会话 / 守护进程等交互形态。与 a2r 转译后端存在语义一致性要求（`docs/conformance/`，plan-266）。

## 现状

- CALL_SPEC 内联数学分发已 nanbox 对齐（plan-474，plan011④ 根因）：一元分支接收者/结果按 NanoValue 透传（原 read_i32/push_i32/pop_i32 i32 化石把裸 f64 读成低 32 位、TAG_F32 读成 payload 位型），二元分支原地调用（rust_fn 宏逆序弹参，CALL_SPEC `[recv, arg0..argN-1]` 布局天然对齐）；回归载具 `tests/vm_json_float_read_tests.rs`（脚本/位级/widget handler 三层）。
- 核心规模：`engine.rs` 6882 行、`codegen.rs` 11437 行、`opcode.rs` 178 个 opcode（`docs/design/05` 中的行数与"约 120 个 opcode"已过时）。
- 值表示为 NaN-boxing u64（`NanoValue`，plan-221 引入、plan-298 移除非 nanbox 路径）；design/05 的"32 位栈槽"描述已过时。
- 泛型走单态化 + 类型擦除存储（plan-076/087），堆对象统一进 `heap_objects` 注册表（plan-077，旧 list 注册表已在 Phase 6 移除）。
- 并发为 Tokio M:N 调度 + actor 消息（plan-121/127）；plan-317 Phase 1（actor handler 执行引擎）已合并，Phase 2-4 待实施。
- 文件测试框架已落地：`tests/vm_file_tests.rs`（907 行）+ `test/vm/` 分类目录（plan-177，plan-index 仍标 Planned，属索引滞后）。
- stdlib 静态分发扩充（plan-504）：`Math.pow`（`f64::powf`）与 `Str.is_digit`（单字符 ASCII 数字谓词，多字符恒 false）Rust shim 入 `vm/ffi/stdlib.rs`，native_registry 自动扫描注册；Vue 端 ts_adapter 映射（`math.*`→`Math.*` 通配 / `is_digit`→`/^[0-9]$/.test`）；文件测试 `test/vm/18_ffi/056_math_pow`、`057_str_is_digit`。
- 字符串池记账自持（plan-510）：over-release 注入源全仓清偿——19 处无计数池引用统一收口 `add_string`/`intern_runtime_str`/`rc_push_str_idx` 咽喉（http_server 裸写/native 返回串/ffi/py_ffi），over-retain 家族配平（BUILD_FSTR/pop_tagged 6 消费点先拷贝后释放/StakeGuard 增池份额）；不变量=每条引用恰一次 retain/release、freelist 槽恒 rc==0，防线三层（dedup 内容校验/墓碑先行/弹出清扫）降级纵深防御、健康态不触发。可观测：`PoolHealth` 快照（underflow_events/phantom_drops/live_shares，rc.rs）+ soak 双档 `pool_soak_churn_short`（日常门禁 800 轮）/`pool_soak_churn_long`（`P510_SOAK_ITERS` 显式档；复审 2M 轮幻影 0/下溢 0/live_shares 归零）。顺手清偿 P499-6/7（Log 族 native ID 移段 1805-1808 脱撞 Shell 1800-1803；kitchen-sink 生成器对 link/tag/use 视图关键字名禁发标签简写）。
- null 家族守卫全景（plan-550，脚本模式 W0）：null 参与算术/拼接/索引/调用/迭代从静默位模式垃圾翻转为可 try-catch 捕获的 Python 风格 TypeError——算术族经 `virt_memory.rs` 共享弹栈助手 `pop_arith_pair_non_null`/`pop_arith_operand_non_null`（+`_F/_D/_U64/MOD` peek 前缀守卫）；拼接病灶在 STR_CAT 臂（codegen 对含 str 的 `+` 静态路由）；GET_ELEM null 对象 not subscriptable + 越界（ListData 四型）`IndexError: index N out of range`（负索引语义保留，tv 存量零撞击）；迭代病灶在 ARRAY_LEN 静默 0 臂（array 通道 for-in 长度探针，顺带翻 null.len()）；CALL_CLOSURE 动态 callee not callable（正常模式被编译期 E0401 先拦）；TYPE_TO_I32/F64 null 静默 -1/-1.0 臂翻案 TypeError（仅 Expr::To 显式转换发射，无内部哨兵依赖）；TYPE_TO_STR null → "None" + print shim（a2py str(None) 三方对齐）。守卫边界=TAG_NULL only（null/nil/None 三拼写经 PUSH_NIL 归一，PLAN-053；历史 i32 哨兵 -1/MIN+1 与真实整数不可区分不守卫，P550-D3）。单测 `engine.rs tests_null_guards` 13 例；语义契约见 [design/null-family.md](design/null-family.md)。
- 未实现：AutoLive 热重载、MicroVM C 实现、Tier-2 JIT、多语言 FFI 插件（design/05 Open Questions）。

## 关键入口

- `crates/auto-lang/src/lib.rs:run_autovm` / `run_with_capture` — 执行入口
- `crates/auto-lang/src/execution_engine.rs:ExecutionEngine` — 引擎选择（恒为 AutoVM）
- `crates/auto-lang/src/vm/codegen.rs:Codegen` — AST → ABC 编译
- `crates/auto-lang/src/vm/opcode.rs:OpCode` — 指令集定义
- `crates/auto-lang/src/vm/engine.rs:AutoVM` — 共享运行时（flash、字符串池、各注册表）
- `crates/auto-lang/src/vm/engine.rs:AutoVM::run_task_loop` / `run_one_instruction` — 派发循环
- `crates/auto-lang/src/vm/task.rs:AutoTask` — 每任务执行上下文
- `crates/auto-lang/src/vm/virt_memory.rs:VirtualFlash` / `VirtualRAM` — 数字孪生内存模型
- `crates/auto-lang/src/vm/heap_object.rs:HeapObject` — 统一堆对象 trait
- `crates/auto-lang/src/vm/monomorphize.rs:Monomorphizer` / `generic_registry.rs:GenericRegistry` — 泛型
- `crates/auto-lang/src/vm/scheduler.rs:GlobalMeta` / `task_system.rs:TaskRegistry` — 调度与 actor
- `crates/auto-lang/src/vm/native_registry.rs:AutoVMNativeRegistry` / `native.rs:NativeInterface` — native 函数
- `crates/auto-lang/src/vm/ffi/c_ffi.rs:CFfiRuntime` — C FFI 动态加载
- `crates/auto-lang/src/vm/debugger.rs:DebuggerController` — 调试器（GDB/JSON agent 两种控制器）
- `crates/auto-lang/src/vm/abt/mod.rs:AbtProgram` — ABT 汇编/反汇编
- `crates/auto-lang/src/autovm_persistent.rs:AutovmReplSession`、`autovm_daemon.rs:AutovmDaemon`、`autovm_client.rs:AutovmClient` — 持久会话与守护进程
- `crates/auto-val/src/nano_value.rs:NanoValue` — NaN-boxed 值表示

## 使用示例

```bash
cargo test -p auto-lang -- vm_file_tests        # 文件测试（test/vm/ 下 .at + .expected.*）
auto run script.at                              # CLI 执行（默认 AutoVM）
auto serve / auto req                           # 守护进程 + 跨进程会话（plan-269）
```

## 已知坑

- UI bug 先降级为纯 VM 脚本复现再分层定位（plan-341 方法论）。
- `VirtualRAM.raw: Vec<i32>` 是 nanbox 迁移残留，运行时栈走 `raw_nv: Vec<NanoValue>`；读旧代码注意区分。
- 与 a2r 的行为漂移对照 `docs/conformance/` 与 plan-242 gap tracker。
- plan-report 07 的 plan 链接指向 `docs/plans/`，实际文件在 `docs/plans/archive/`。

## 蒸馏来源（Phase 1）

- `docs/design/05-vm-runtime.md`
- `docs/plan-reports/07-vm-runtime.md`
- `docs/plan-indices/07-vm-runtime.md`
- `docs/conformance/`（README + 01/02/03/04/10）
- 代码核对：`crates/auto-lang/src/vm.rs`、`vm/`（engine/codegen/opcode/task/scheduler/heap_object/generic*/ffi/abt/debugger/virt_memory）、`autovm_*.rs`、`execution_engine.rs`、`crates/auto-val/src/nano_value.rs`
