# PLAN-724 T-07 资源与生命周期报告

- 阶段：work（T-07）
- 载具：worktree `D:/autostack/.wt/lang-724/auto-lang`，分支 `plan-724-dev`
- 预算冻结值见 `724-client-decision.md` §3；本报告给**实测证据**与内存算式。

## 1. 执行拓扑（实测形态）

- 单进程内**一个**内核 runtime（multi-thread，默认 2 worker，线程名
  `a2r-http-kernel`）+ 一个复用连接 `reqwest::Client`。全局单例经
  `OnceLock`；测试经 `KernelInstance::new` 构造独立实例（本计划测试**零**
  env 变更、零进程级全局污染）。
- 非流式 job：队列许可（try）→ 活跃许可（await）→ 总期限包裹 → 增量读体。
  无每请求线程/ blocking client/spawn_blocking（全仓扫描在案）。
- 流生产者：流队列许可（try）→ 流活跃许可（await）→ 建立期限 → 读循环
  （idle 只包上游 read）→ 逐项有界入队。abort 句柄 spawn 后登记；
  `close()/Drop` → finalize(Cancelled) + abort。

## 2. 预算断言（实测证据）

| 断言 | 测试（`cargo nextest run -p a2r-std`，43/43） |
|---|---|
| 队满即时终结（active=2+queue=4，第 7 提交 `QueueFull`） | `plan724_kernel_queue_full_is_immediate_terminal` |
| 调用方丢弃 future → job 退出、许可回基线 | `plan724_kernel_cancel_dropped_future_stops_job_and_returns_permit` |
| 响应体超预算可观察终结（>64KiB → `BodyTooLarge`） | `plan724_kernel_body_budget_observable_terminal` |
| 流背压高水位钉界（消费者暂停，队列 ≤ max_queued_items=4） | `plan724_kernel_stream_backpressure_high_watermark_bounded` |
| 单条目超预算（SSE 事件 > item 预算 → Failed 含 budget） | `plan724_kernel_stream_item_budget_terminal` |
| 建立超时终结（2s 无 headers） | `plan724_kernel_stream_open_timeout_terminal` |
| 流 queue 满 → typed Failed（不 panic 不排队） | `plan724_kernel_stream_queue_full_fails_typed` |
| close/Drop 取消生产者、许可归还、幂等 | `plan724_kernel_stream_close_cancels_producer_and_returns_permit` |
| 60 流风暴后许可回基线、可再准入（异步归还+有界重试） | `plan724_native_cancel_storm_returns_to_baseline`（integration） |
| 单次终结（Eof 后 next 恒 None；EOF 先排空队列） | `plan724_kernel_stream_raw_chunks_eof_and_status` 等 |

## 3. 内存算式（应用保留上限；reqwest/Hyper/TLS 内部缓冲不入算式）

**单流峰值** = 队列 16 条 × 256 KiB（4 MiB）+ 解码行 carry ≤ 256 KiB + 事件
构造 ≤ 256 KiB + raw Utf8Carry ≤ 256 KiB + 当前网络块 + 响应 headers Vec
（条目级，无上限刻度——上游 headers 有限；恶意超长 headers 由 hyper 解析
上限兜底，不属本预算）。

**并发乘积** = 流 active 16 × 单流 ≈ 64 MiB + 非流式 active 8 × 响应体
10 MiB = 80 MiB + 排队 job 的请求体（调用方 own，不入内核）+ 固定件
（2 worker 线程栈 + Client 连接池）。**逐项有界生产**：事件按条入队，
无整响应累积 Vec；`read_body_capped` 增量累计且超限即断。

**大量事件场景**：16 KiB/条 → 队列 16 条 ≈ 256 KiB（不是 16×256 KiB——
按条计不按预算乘满）；256 KiB/条 → 队列满 4 MiB。两者都低于「队列总量
4 MiB」上限；报告不以「总预算 256 KiB」误述（16×256 KiB 队列 ≠ 256 KiB 总量）。

## 4. 生命周期与终态语义（实测钉）

- **一次终结**：首个终态（Eof/Failed/Cancelled）胜出；`finalize` 幂等；
  迟到生产者在 enqueue 前复查终态被拒（无 insert 重建）。
- **EOF 排空**：上游 EOF 先交付已入队数据再置终态；`is_finished` =
  队列空 ∧ 终态（上游写完不丢尾）。
- **取消面**：close/Drop（流）、丢弃 execute future（非流式）→ 生产者/job
  在当前 await 点退出；队满时取消、首包未到取消、idle 取消均可在对应
  await 点终结（backpressure/idle/open 测试族）。abort 句柄随生产者退出
  出表（自卸），close 侧取用幂等；许可归还经 runtime worker 异步完成
  （storm 用例的有界重试即此语义的消费者可见形态）。
- **控制串不作内核状态**：`__status__`/`__done__` 仅存在于 legacy 兼容
  壳的历史协议叙述中；内核 typed 状态机零控制串（status 元数据为
  `Option<u16>` 字段）。
- **错误可查**：`terminal_error()`/`ClientError` 分层（§决策 §4）；
  失败不冒充空成功。

## 5. 观测探针（为后续计划保留的公共面）

- `KernelInstance::{active_available, stream_active_available}`（测试/资源
  报告；非测试也可用于运维诊断）。
- `HttpClientStream::{queued_len, is_finished, terminal_error, status}`。
- facade `HTTPStream::terminal_error`（707 同形三态）。
