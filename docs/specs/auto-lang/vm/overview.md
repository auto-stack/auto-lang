# vm（AutoVM）

> **Status**: implemented

## 职责

AutoVM 是 AutoLang 的默认执行后端，也是唯一可用的解释执行后端（plan-081 设为默认，plan-091 起 evaluator 选项弃用并重定向到 AutoVM）。职责覆盖：AST → ABC 字节码编译（codegen）、栈式字节码执行引擎、堆对象统一管理、泛型单态化、Task/Msg 并发与 async/await、native/FFI 接口、交互式调试器、ABT 字节码文本格式，以及 REPL / 持久会话 / 守护进程等交互形态。与 a2r 转译后端存在语义一致性要求（`docs/conformance/`，plan-266）。

## 现状

- 核心规模：`engine.rs` 6882 行、`codegen.rs` 11437 行、`opcode.rs` 178 个 opcode（`docs/design/05` 中的行数与"约 120 个 opcode"已过时）。
- 值表示为 NaN-boxing u64（`NanoValue`，plan-221 引入、plan-298 移除非 nanbox 路径）；design/05 的"32 位栈槽"描述已过时。
- 泛型走单态化 + 类型擦除存储（plan-076/087），堆对象统一进 `heap_objects` 注册表（plan-077，旧 list 注册表已在 Phase 6 移除）。
- 并发为 Tokio M:N 调度 + actor 消息（plan-121/127）；plan-317 Phase 1（actor handler 执行引擎）已合并，Phase 2-4 待实施。
- 文件测试框架已落地：`tests/vm_file_tests.rs`（907 行）+ `test/vm/` 分类目录（plan-177，plan-index 仍标 Planned，属索引滞后）。
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
- plan-report 07 的 plan 链接指向 `docs/plans/`，实际文件在 `docs/plans/old/`。

## 蒸馏来源（Phase 1）

- `docs/design/05-vm-runtime.md`
- `docs/plan-reports/07-vm-runtime.md`
- `docs/plan-indices/07-vm-runtime.md`
- `docs/conformance/`（README + 01/02/03/04/10）
- 代码核对：`crates/auto-lang/src/vm.rs`、`vm/`（engine/codegen/opcode/task/scheduler/heap_object/generic*/ffi/abt/debugger/virt_memory）、`autovm_*.rs`、`execution_engine.rs`、`crates/auto-val/src/nano_value.rs`
