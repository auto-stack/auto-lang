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
- 互操作分发层（plan-555，脚本模式 W1）：`vm/interop.rs` 分发组合子六件（`obj_get/set/call/len/iter/type_name`，native ID 1860-1865，目录限定名+裸名双注册）——运行期 tag 分派：外对象经 `HeapObject::as_foreign_object` 默认钩子（None 默认，宿主覆写）取 `ForeignObject` 协议面（首实现=py_ffi PyObjectHandle 六操作臂，send/contains 预留位），Auto 值走原生方法表（str/list/map 索引与按名读写、ARRAY_LEN 语义、array 通道迭代回推、callable/iterable 守卫对标 550）；发射复用 CALL_PY 传输形态（带实参数字节，P555-D5 命名债）；py 三桥补齐 `py_setattr` 467/`py_len` 468/`py_type_name` 469（539 桥型）。契约见 [design/interop-dispatch.md](design/interop-dispatch.md)。
- 脚本模式 W2 糖批（plan-560）：py 桥扩至 475（470 contains/471 module/472 str/473 pow/474 truthy/475 is——GIL 通道）；`CALL_PY→CALL_NAT_COUNTED` 改名（带计数字节通用原生调用传输形态）；CALL_PY 错误出口 FFI→RuntimeError 统一（catch 拦值一致）；print shim 运行期 PyObjectHandle→GIL str() 臂；550 生产者门控**硬化**为诊断错误 auto_gate_E5501（文件上下文限定——内联/eval 无 path 不门，tv 文件测试走 run_with_capture_and_path 受门，.as/#[script] 豁免/#[rust] 压回）。
- 脚本模式收官波（plan-567）：**Err 值通道拦截**——ERROR_PROPAGATE 的
  Result.Err 传播遇当前帧 try handler 改跳 catch_pc 绑 `PyException <Type>:
  <msg>` 载荷（设计 §4.3 两通道汇合；null/None 是值不进 catch）；主边界
  未捕获 Err 带错退出（exit 1）；py 桥扩至 **480**（476/477 getattr/getitem
  may 变体、478 kwargs×may（P539-D5 销号）、479 py_raise 再抛、480 py_int；
  py_enter 返 `__enter__` 值）；py 桥错误统一 PyException 前缀族（py_exc）；
  CALL_NAT/COUNTED 非 FFI 错误原样传播加固（静默吞错回归修复）；ADD
  string-concat **先读后放**池纪律（P053-8 幻影+"got d"双根因，P539-D1
  销号）；namedtuple/structseq 封送 opaque 句柄（version_info 病灶，
  P560-D6 销号）；wrap_tos TAG_LIST/null 补臂。
- py 返回值方法分派动态化（plan-569，P539-D2 根治）：**codegen py-类型侧表**
  三件（`last_expr_may_py` 粘性位 / `py_typed_vars` 变量落盘 /
  `fn_may_py_returns` 单层流型回填）——方法分派决策核（`Expr::Dot` 臂首
  `receiver_may_py`）在 infer_object_type→"str.len" 限定名 peek 静态路由
  **之前**插队：py 可能接收者 `.len()` 改发 obj_len(1863)、其余方法改发
  obj_call(1862)（栈序 `[recv, method, args...]`，与 s2s A1 产物同构），
  运行期 tag 双通道（PyObjectHandle→GIL / Auto 值→原生语义）；类型谎言
  本体保留（顶层结果格式化依赖），侧表只覆盖"路由到哪"。三源传导=py-ffi
  调用点（py_native_map+py_modules 点调）/Ident 加载镜像成员资格/用户 fn
  调用点限定名双查；重置纪律四处（compile_expr 顶/函数体首/原生追踪链块首/
  Store 边界）。`shim_str_len` PyObjectHandle 兜底臂恒 0→GIL len（存量编译
  产物加固）；obj_call Auto 臂=TypeError 硬臂（550 守卫口径，可 catch）。
  回归载具 `tests/plan569_py_dispatch_tests.rs` ×6 + `test/vm/99_py_dispatch/`
  ×2 + `99_script_err/05`；rust 侧同族谎言与 .as lowering 括号重绑为登记债
  P569-D1/P569-R1。
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
