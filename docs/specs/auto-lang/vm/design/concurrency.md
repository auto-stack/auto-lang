# 并发：Task/Msg 与调度

## 范围

AutoVM 的任务系统、Tokio M:N 调度、actor 消息路由、async/await、channel 与 `.go`。对应代码：`vm/task.rs`、`vm/scheduler.rs`、`vm/task_system.rs`、`vm/task_handler.rs`、`vm/message_context.rs`、`vm/channel.rs`、`vm/pattern_matcher.rs`、`engine.rs` 并发 opcode 段。

## 原则

- 共享运行时与任务上下文分离：`AutoVM` 持有全局注册表，`AutoTask` 持有私有栈/帧/IP（ADR-08）。
- Actor 私有状态，无共享锁；消息经类型化 mailbox + 模式匹配路由（design/05 四阶段模型）。
- 全局可变状态只走受控通道：`AutoVM.globals: DashMap<String, NanoValue>` 承载模块级 `var`（engine.rs 内 plan-317 注释）。

## 细节

### 任务与调度

- 任务经 `tokio::spawn` 调度，存于 `DashMap<TaskId, Arc<Mutex<AutoTask>>>`；`scheduler.rs:GlobalMeta` 为跨任务共享只读元数据，`spawn_task`（scheduler.rs:448）创建任务；daemon 循环处理 `SystemCommand`（scheduler.rs:111）管理生命周期。
- 9 个并发 opcode：SPAWN 0x80、TASK_ID 0x81、YIELD 0x82、SLEEP 0x83、JOIN 0x84、CHAN_NEW 0x85、SEND 0x86、RECV 0x87、TRY_RECV 0x88（report 07）。
- SLEEP 通过 `AutoTask.wake_time` + 状态机挂起；JOIN 等待任务终态。

### Actor 消息（四阶段模型，design/05 §Concurrency）

- Phase 1 静态 `task` 块 + 隐式 mailbox；`TaskSystem.start()` 把主线程交给调度器（task_system.rs: TaskRegistry/TaskInstance/TaskHandle）。
- Phase 2 `~T` 异步蓝图、`.await`、`ask/reply` 经隐式 oneshot；plan-224 落地 `TaskSystem.run` 同步桥与 AWAIT_FUTURE 重入执行，`FutureValue`（engine.rs:316）登记挂起 future。
- Phase 3 `on` 块隐式 union、`on(ctx)` 显式 MessageContext（message_context.rs）、字面量/类型捕获/guard 由 `pattern_matcher.rs` 路由；`TaskHandlerTable`（task_handler.rs）把消息模式映射到字节码地址，`ctx.reply()` 走 FFI shim + oneshot。
- Phase 4 `.go` 后缀派发到后台 worker（SPAWN_GO，plan-127），PC 上映射 `tokio::spawn`。

### plan-317 之后的结构事实

- 每任务消息队列挂在 `AutoVM.task_mailboxes: DashMap<u64, Mutex<Vec<Value>>>`，刻意不放 `AutoTask`——后者在 tokio Mutex 后，会引发 `blocking_lock` panic（engine.rs 注释）。
- Actor 状态字段存 `AutoTask.state_vars: Vec<NanoValue>`，独立于 bp，跨 handler 调用存活；经 LOAD_STATE_FIELD/STORE_STATE_FIELD 访问（task.rs 注释）。
- 等待外部事件的挂起走 `waiting_sse_stream_id`（plan-348）/`waiting_http_request_id`（plan-349），由 run_task_loop 检查唤醒。

### 已知限制

- SEND/RECV 为 busy-wait + yield，非真 tokio mpsc await（report 07 Open Questions；plan-317 Phase 2-4 待实施）。
- ask/reply 阻塞同步在 plan-127 时 deferred，plan-224 提供的是 `~{}`/`await` 桥。

## 显式非目标

- 真异步 channel 操作与统一异步调度（plan-317 Phase 2-4，未实施）。
- MCU/RTOS 后端映射（xTaskCreate 等）：design/05 的 ISA 兼容愿景，本模块只有 Tokio 实现。
- `.go` 边界的闭包借用捕获（design/05 Open Questions，逃逸分析见 plan-310）。

> 来源: docs/design/05-vm-runtime.md（§Concurrency Model）；docs/plan-reports/07-vm-runtime.md（§Concurrency and Task System）；crates/auto-lang/src/vm/{task,scheduler,task_system,task_handler,message_context}.rs、engine.rs 注释；plan-121/124/127/128/224/317
