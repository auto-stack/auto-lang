# 原生调度车道（`mn.sched.runtime/1`，Windows AC 后端）

> PLAN-1067（M2-B）。本文件只记录**已验证**的支持、接口、终态与边界；未列出的能力不视为已支持。
> 语言面与语义合同沿用 [m2-scheduler.md](../ace-migration/m2-scheduler.md)（PLAN-968）冻结的
> `task`/`on`/`Task.spawn`/`send`/`ask`/`reply`/`await`/`go`/`sleep`/`stop` 语义与 `SCHED_*` MIR（0x41–0x4B）；
> 本文件记录这些语义在 **MIR-native（AC）** 车道上的真实机器执行实现、ABI、所有权/终态规则与证据。
> 平台范围：**仅 Windows x86-64（`x86_64-pc-windows-msvc`）**；Linux 为 NOT_RUN（见 §7）。

## 1. 入口与身份

| 项 | 值 |
|---|---|
| 源 profile | `auto.source.scheduler.draft`（aggregates 超集＋task actor 面；共享前门 `core::source::compile_scheduler_source`） |
| 共享前端→IR | resolver/typecheck → closures 家族 HIR → `lowering::lower_checked_closures` → `verify_mir` |
| 降层模式 | `native::lower::LowerMode::Sched`（数值值面＋console/1 `HostCall`＋`SCHED_*` 发射） |
| 运行时 ABI | `mn.sched.runtime/1`（`experimental/ace/crates/sched-runtime`，no_std＋kernel32，`staticlib` `ac_sched_runtime`） |
| 车道入口 | `native::sched::{sched_compile, sched_callees, sched_native_build, sched_native_run, audit_sched_pe_imports}` |
| 固定组 | `cargo t ace-native`（内含 `tests/sched_m2_1067.rs`）；AV oracle 组 `case-ace-m2-scheduler-968`；A2R 面 `ace-a2x` |
| 语料 | `crates/auto-lang/test/cases/ace-m2/scheduler/`（8 正例＋3 反例，与 AV/A2R 同源同字节） |

运行时静态库由 `native::sched_runtime::prepare()` 以**内容/构建绑定身份**准备：哈希覆盖
`sched-runtime` 与内嵌 `text-runtime` 的 `Cargo.toml`/源码、workspace `Cargo.toml`/`Cargo.lock`、
`.cargo` 配置、rustc 身份与相关 `RUSTFLAGS`；identity 的 `content_hash` 必须等于当前库字节，
否则 `mn.admit.runtime-content` 具名拒绝。**同一 PE 只链一个 Rust staticlib**：调度 runtime 以
依赖方式内嵌 owned-text runtime（其文本/分配 ABI 与编译器生成的 `mem*`），两个 staticlib 会在
panic/unwind 符号上冲突。

## 2. 能力（Windows，已验证）

| 能力 | 状态 | 实现要点 |
|---|---|---|
| `task` 声明/驱动 | ✓ | 共享 desugar 合成 `__task_<Name>(__init…) int`（start/stop 钩子＋`while true` 收/分派环）；本车道按 callee 表为每个 callee 生成 unit-local spawn thunk（`ac1067tk.<owner>.<unit>.<idx>`），解包实参后调 driver 自己的 unit 符号 |
| `Task.spawn(name, cap, inits…)` | ✓ | 真实 OS 线程（`CreateThread`）＋有界 FIFO 邮箱（`SRWLOCK`+`CONDITION_VARIABLE`）；handle 为 1 基 i32；容量实参即邮箱上界（**≥1**，0 为具名语言错误，三端一致） |
| `send` | ✓ | FIFO 投递；邮箱满则**发送者阻塞**（背压），无忙等；对已 stop/closed 任务为具名语言错误 |
| `ask` / `reply` / `await` | ✓ | 1 基单消费者回复端口；`reply` 仅在处理 ask 消息时合法；`await` 阻塞直到发布；端口单终态提交（迟到/重复 reply 丢弃，不复活） |
| `go f(…)` | ✓ | 无邮箱、无 handle 的一次性线程；退出时 join |
| `sleep(ms)` | ✓ | 真实 OS 睡眠分片（每 10ms 检查 drain/stop）；drain 期立即完成（AV 同规则） |
| `stop` | ✓ | 置 stop 标志并唤醒；**缓冲消息仍先投递**，stop 钩子在邮箱耗尽后执行 |
| 有序退出 | ✓ | 包装器在 entry 返回后调 `mn_sched_drain`：置 drain/stop→**循环重扫并 join**（drain 期间由 stop 钩子或缓冲消息处理合法启动的新链也会被 join，直到无未 join 链）→释放邮箱/端口/线程参数→审计；残值非零则**具名退出 72**（`sched.leak`），绝不冒成功 |
| 值类型跨交接 | ✓ i32/bool/owned `Str` | `SchedVal`＝{tag 0=i32, 1=bool, 2=owned text(ptr,len), spare}；owned text 在交接点**物化**（`ac_rt_own_clone_quiet` 走内嵌 owned-text 分配器），接收方 `ac_rt_own_free`/出口 sweep 释放 |
| 语言错误通道 | ✓ | `mn_sched_fail(code)` 写冻结消息到 stderr 后 `ExitProcess(70)`（=native trap 类）；与 AV 的 `ExecOutcome::Error` 同文本、不同通道（沿 [m2-control-effects](../ace-migration/m2-control-effects.md) 的既有分类差异） |
| console/1 `print` | ✓ | 调度单元的 `HostCall` 准入并链 console shim；owned-text runtime 的观察 trace 行由包装器在 entry 前关闭，stderr 与 AV 逐字节一致 |

## 3. 终态与取消规则（三端一致）

- **closed future**：生产者 driver 结束时，其已 latch 未回的 ask 端口、以及仍排在它邮箱里
  （再无人接收）的 ask 消息端口一律置终态；drain 期所有仍 Pending 的端口同样置终态。等待方得到
  `sched: task closed without replying`（native：stderr＋70；AV/A2R：同名错误），**不会自旋到 fuel 上限**。
- **迟到结果**：reply 在端口已被消费后到达则丢弃并释放其物化缓冲；不复活旧实例。
- **stop 与在途消息**：stop 不清空邮箱；驱动先消费缓冲消息，`stopping()` 仅在邮箱空时报告停止
  （一次真实投递会遮蔽一次检查，沿 A2R 同序）。
- **资源**：任务记录、邮箱、端口、线程参数、物化文本全部计入审计；drain 后非零即退出 72。

## 4. 依赖与 ABI 审计

- 车道 PE 的导入**仅** `kernel32.dll`，且每个符号必须在 `SCHED_KERNEL32_IMPORTS`（18 项：
  `CreateThread`/`WaitForSingleObject`/`CloseHandle`/`TlsAlloc`/`TlsSetValue`/`TlsGetValue`/
  `AcquireSRWLockExclusive`/`ReleaseSRWLockExclusive`/`SleepConditionVariableSRW`/
  `WakeConditionVariable`/`WakeAllConditionVariable`/`Sleep`/`GetProcessHeap`/`HeapAlloc`/`HeapFree`/
  `GetStdHandle`/`WriteFile`/`ExitProcess`）之内；禁止 ordinal 导入、CRT（`msvcrt`/`ucrtbase`/
  `vcruntime*`/`api-ms-win-crt*`）与其它 DLL。
- 审计入口 `native::sched::audit_sched_pe_imports`；共享 `native::platform_io::audit_pe_imports`
  保持原冻结表不变（已参数化为按车道 allowlist）。
- `platform.*`（clock/timer 等）在调度单元仍为**具名拒绝** `native.sched-platform`；源级
  `std.time.*` 模块入口属库/FFI 接面（PLAN-1068/T-03），本车道不声称支持。
- guest 不得经 AV/旧 VM/A2C 转译；本车道产物是真实 PE，业务与状态机均由机器码执行。

## 5. 证据（Windows，2026-11-17）

| 证据 | 入口 | 观测 |
|---|---|---|
| 8 正例逐字节＋零残值 | `cargo test -p ace-native --test sched_m2_1067 scheduler_corpus_runs_native_byte_exact` | `got one\\n` / `mycounter: 15\\n` / `got two\\ngot one\\n` / `reply:\\n42\\n` / `go:\\n7\\n` / `1\\n2\\n3\\n` / `one\\nstopped\\n` / `main\\nlater\\n`；exit 0、stderr 空 |
| owned 值跨交接 | 同上 `scheduler_owned_text_state_crosses_spawn`、`scheduler_typed_reply_crosses_the_await` | `my\n`（owned Str 状态）、`true\n`（bool reply 跨 await） |
| 终态/资源 | `scheduler_await_on_unreplied_ask_is_a_named_error`、`scheduler_unawaited_reply_is_released_at_drain`、`scheduler_exit_drain_releases_every_outstanding_owner` | trap 70＋具名消息、exit 0 零残值、<10s 收敛 |
| 真实 OS 等待 | `scheduler_sleep_is_a_real_os_wait` | PE 墙钟 ≥35ms（no-op 睡眠会是个位数 ms） |
| 仓外候选＋审计 | `scheduler_candidate_runs_out_of_repo_kernel32_only`、`..._audit_rejects_a_mutant_dependency` | kernel32-only、符号全在声明表内、`CreateThread` 在列、CRT 变异体被 `pe.audit.forbidden-dll` 拒绝、runtime 身份＝库字节 |
| 三端独立 oracle | 本文件 §2 能力面 × `sched_m2_1067`(AV `av_leg` + native) × `ace-a2x-worker` `sched_968_a2r_rust_emit`/`sched_1067_a2r_timer_and_closed_task` | 各端对**同一文档化 expected**，不以互相比较代替 oracle |

## 6. 拒绝与边界（具名）

`native.sched-unsupported`（非调度车道遇 `SCHED_*`）、`native.sched-platform`（`platform.*`）、
`native.sched-value`/`native.sched-arity`/`native.sched-callee`/`native.sched-context`（值/元数/
callee 表/上下文形状）、`mn.unsupported.pe.audit.forbidden-dll|forbidden-symbol|ordinal-import`、
`mn.admit.runtime-content`（未审计 runtime 内容）；语言面沿用 `source.sched-task`/
`source.sched-argument`/`source.task-*` 等，由共享前门给出。

## 7. 延期/未支持（NOT_RUN，不升 verified）

| 项 | 现状 | owner |
|---|---|---|
| Linux 调度腿 | 本 lane 只为冻结 Windows triple 降 COFF/kernel32，无 Linux 分支与用例 | 1063：写实现＋原完整用例后验；Windows 证据不外推 |
| owned-text **消息/回复** | 共享前端具名拒绝（内建签名要 owned `Str`，表面只产 borrowed view）：`source.sched-argument` | 1068/T-02：在 scheduler 参数投影处插入 `text_to_owned`；关闭时改本包边界钉并补三端正例 |
| `platform.*` / 源级 OS timer | 调度单元具名拒绝；`sleep` 已是真实 OS 等待 | 1068/T-03（库/FFI 接面） |
| 进程外取消源 | 内部终止语义已验（closed future、stop 不复活、drain 立即完成）；无宿主取消句柄/超时中断在跑任务 | 后续宿主/产品计划 |
| owned-text 循环内临时 | 循环体**第二次迭代**重建文本临时未逐次释放（退出 sweep 只释放最后一次写）；本车道 drain 审计以 exit 72 捕获（`print(f"a=${n}")` 收两条 → 72；改 `print(n)` → 0） | 共享 owned-text native 降层（774/870） |
