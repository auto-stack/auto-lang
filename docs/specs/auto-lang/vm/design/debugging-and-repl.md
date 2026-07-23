# 调试、ABT 与交互形态

## 范围

ABT 字节码文本格式、反汇编、交互式调试器、执行 trace、REPL/持久会话/守护进程、文件测试。对应代码：`vm/abt/`、`vm/disasm.rs`、`vm/debugger.rs`、`vm/trace.rs`、`autovm_repl.rs`、`autovm_persistent.rs`、`autovm_daemon.rs`、`autovm_client.rs`、`tests/vm_file_tests.rs`。

## 原则

- 会话状态持久、代码片段瞬时：VM 实例长存，字节码块编译-执行-丢弃，数据栈/全局变量/符号表保留（design/05 §Streaming）。
- 调试能力内置于引擎：单步走 `run_one_instruction`（engine.rs:1574），正常执行挂 `NoOpController` 零开销。
- AI-first：调试与交互暴露 JSON 接口，供 agent 驱动（plan-199、plan-265）。

## 细节

### ABT 与反汇编

- ABT 是 ABC 的文本形式：`AbtProgram`/`AbtInstruction`/`AbtOperand`（abt/mod.rs:18/35/48），配套 `asm.rs`（汇编）、`disasm.rs`（反汇编）、`parser.rs`；plan-226 Phase 1-3 完成，Phase 4 确认无需做。
- `vm/disasm.rs:DisasmLine` 供 `run_with_capture_and_bytecode`（lib.rs:292）返回字节码视图，Playground 字节码 tab 即消费此接口。

### 调试器

- `DebuggerController` trait（debugger.rs:41）三种实现：`NoOpController`（默认）、`GdbController`（GDB 风格 CLI）、`JsonAgentController`/`AgentController`（JSON agent API，plan-199）。
- 源码定位：codegen 发 SOURCE_LINE opcode，`AutoTask.current_line`/`current_source`/`call_stack`（task.rs）支撑断点与栈回溯；`Breakpoint`（debugger.rs:34）支持行/地址断点。
- `trace.rs:TraceCollector` 收集执行轨迹，`AutoVM.trace` 为 None 时零开销。

### REPL / 持久会话 / 守护进程

- 语句级栈平衡保证跨片段局部变量偏移稳定；"开放帧"技术把会话视为无限长函数，块边界用 SUSPEND 保帧（design/05）。
- `AutovmRepl`（autovm_repl.rs:22）为交互前端；`AutovmReplSession`（autovm_persistent.rs:36）为可复用持久会话，plan-069 落地全局变量持久化。
- 栈溢出教训：session.run 的 parse+compile+execute 整体放到 8MB 栈的独立 OS 线程，避免 tokio runtime 元数据耗尽调用线程栈（archive/plan-355）。
- 守护进程：`auto serve`/`auto req`（plan-269），`AutovmDaemon`（autovm_daemon.rs:74）走命名管道（Windows）/Unix socket，`AutovmClient`（autovm_client.rs:227）分 pipe/stdio 两种，支持跨进程会话共享、max-sessions 与超时。
- MCP 服务器（plan-265）暴露 7 个 JSON-RPC 工具（session create/reset、evaluate、typecheck、patch、snapshot、inspect），归 mcp 模块详述。

### 文件测试

- `tests/vm_file_tests.rs`（plan-177）：扫描 `test/vm/{category}/{NNN_name}/`，支持 `.expected.out`/`.expected.result`/`.expected.error` 三种断言；`AutoVM.output_buffer` 提供 stdout 捕获（`new_with_capture`）。
- 一致性测试：`docs/conformance/` 规范 + `test/a2r/conformance/` 对偶测试 + 随机程序差分（plan-266，见 architecture.md ADR-10）。

## 显式非目标

- AutoLive 热重载（增量 patch 写入 Hot Zone + GOT 重定向）：design/05 愿景，未实现。
- Playground/IDE 侧调试界面：属 ui/playground 模块，本模块只提供引擎侧 API。

> 来源: docs/design/05-vm-runtime.md（§Streaming/REPL、§Hot Linker）；crates/auto-lang/src/vm/{abt,disasm,debugger,trace}.rs*、autovm_*.rs、tests/vm_file_tests.rs；plan-069/177/199/226/265/266/269、archive/plan-355
