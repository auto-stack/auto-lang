# PLAN-707 T-07 资源生命周期报告

- 日期：2026-09-29
- 证据命令与测试：`cargo nextest run -p auto-lang --lib -E 'test(plan707_stream)'`（5/5 绿）；
  E2E 面 `plan707_stream_e2e_tests`（3/3 绿，`--features test-http-e2e`）。
- 冻结值出处：`docs/plans/reports/707-stream-decision.md` D-10。

## 1. 线程模型（AC-06）

- 固定 runtime 共享：非流式 client executor（705 形态）+ 流生产者同 runtime；
  流侧仅新增**许可对**（active/queue Semaphore），无新线程池。
- 探针 `plan707_stream_thread_count_stable_across_streams`：预热稳态后批量
  12 条并发流建立→终结，进程线程数 10 → ≤12（容忍 +2 调度抖动；实测绿）。
  旧形态对照：`spawn_async_sse_stream` 每流 1 `auto-sse-client` 线程 + 1
  current-thread runtime——已退役。
- 已知每流非线程资源：1 reqwest 连接（runtime 上的 future）+ 1 mpsc（SSE 桥）。

## 2. 背压与内存上界（AC-04）

- 探针 `plan707_stream_backpressure_bounds_queue_and_stops_pull`：消费停读时
  队列钉在 `max_queued_items`（测试覆写 4；默认 16），上游 TCP 写端被反压，
  取消可打断满队列生产者（abort 生效于 space 等待）。
- 内存上界模型（每流）：
  - 事件队列 ≤16 × 单事件 ≤256 KiB（SSE）或 raw 文本块 ≤16 KiB/块 ×16；
  - 解码 carry ≤256 KiB（SSE 行预算）；
  - 发送帧 ≤帧大小（传输侧 699 冻结）。
  - 单流上界 ≈ 4.25 MiB（SSE 极端）／≈256 KiB（raw 默认）；16 活跃流 ≤~68 MiB
    事件缓冲上界。无无限队列/缓冲；队满=背压停读（非丢弃、非无界增长）。
- 预算终结：`plan707_stream_item_budget_terminal_failed`——单事件超限 →
  `Failed("stream item exceeds budget N")`（可观测诊断，非伪造成功）；终态条目
  保留至显式 close（消费方查询诊断），close 后出表。

## 3. 取消与回基线（AC-01/AC-03/AC-06）

| 路径 | 证据 | 结果 |
|---|---|---|
| managed job 取消（queued/active/retry 三阶段） | `plan707_cancel_*`（T-02） | 全绿：future 实停、许可归还、迟到不复活、detached 不误杀 |
| 流取消风暴两轮 | `plan707_stream_cancel_storm_two_rounds_baseline` | 全绿：流表/stream active 许可/live-op 表同配置两轮回基线无增长 |
| 显式 close（无 scope） | `http_e2e_plan707_stream_explicit_close_two_rounds_baseline` | 全绿：每轮出表 |
| 下游断连 → 上游取消 | `http_e2e_plan707_relay_downstream_disconnect_cancels_upstream` | 全绿：上游读端 ≤8s 观测连接关闭（非 timeout） |
| SSE 输出终结 → generator 内上游流 | `http_e2e_plan707_relay_frames_timed_single_termination`（资源断言段） | 全绿：5s 内流表/live-op 回基线 |
| abort 句柄表 | `plan707_stream_job_abort_table_reclaims` | 全绿：成功/取消 job 均出表（T-07 修复成功路径泄漏：原形态每个成功 job 滞留 1 AbortHandle） |

## 4. 资源组所有权（D-7 实装形态）

- **主组**：`RequestScope.resources`——dispatch/resume 段（`bind_current_scope`
  RAII）内 open 的流登记；`finalize_scope`（完成/取消/断连/关闭）逐流
  `stream_cancel`（abort 生产者 + 出表 + 通知）。
- **第二线**：`AutoTask.owned_stream_ids`——generator 体首次 pull 发生在 SSE
  serve 循环（scope 守卫已退出），open shim 同时登记到执行任务；收口点：
  - `cleanup_sse_iterator`（Generator 臂：任务回收前清空 owned 流）；
  - `abort_parked_request`（废弃请求的任务持有流取消）。
- **非 request 上下文**：显式 close/close 语义（E2E 第 3 例）；队满背压保证
  未关闭流的上游拉取已停止（不泄漏增长，仅连接保持）。
- 已知边界：abort 为协作式（下一 await 生效）——本计划所有 job/生产者均为
  网络 I/O 型（await 密集），无 CPU 密集不可打断面；general task 终结
  （run_task_loop Terminated）不清 owned 流（非 request 上下文由显式 close
  承担，未自动 GC——与 D-7 冻结一致）。

## 5. 建立与期限行为

- 建立期限 10s（`AUTO_HTTP_STREAM_OPEN_TIMEOUT_MS`）：黑洞上游 open 立即
  返回句柄（Opening），headers 超时 → `Failed("stream open timed out")`。
- 读空闲 60s 只包上游 read（背压等待不计时——`enqueue` 的 space 等待独立于
  idle 计时器）；写停滞 30s 属传输侧 FrameStream（未在本轮改动 699 冻结值，
  见 decision D-9/D-12；E2E relay 的 1s heartbeat 保持活性）。
