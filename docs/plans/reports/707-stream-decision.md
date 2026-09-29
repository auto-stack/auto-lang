# PLAN-707 T-01 决策报告：VM 外部 HTTP/SSE 流异步消费、转发与取消

- 日期：2026-09-29
- 基线：master @ 66c9cac19（plan-707-dev worktree `D:/autostack/.wt/lang-707/auto-lang`）
- 证据：`cargo nextest run -p auto-lang --lib -E 'test(plan707_spike)'` → 3/3 期望失败
  （`plan707_spike_tests.rs`，红相细节见 §1）；其余为源码走读证据（标注 file:line）。
- 状态：T-01 冻结。后续任务（T-02..T-08）按本文执行；数值/签名调整必须回写本报告。

## 1. 现状定罪（红测 + 源码证据）

| # | 缺口 | 证据 |
|---|---|---|
| G-A | **705 managed job 取消不闭合**：`cancel_live_op` 只删结果槽并通知；`submit_client_job`（`async_http.rs:272`）spawn 的 wrapper 无 abort handle/取消 select——取消后 future 继续运行、active/queue 许可继续被占。红测 `plan707_spike_cancel_gap_active_job_future_not_stopped`：cancel + gate 打开后 job marker 置位。 | 红测 1 |
| G-B | **流建立阻塞 owner**：`shim_http_get_stream/post_stream/post_stream_with_headers`（`stdlib.rs:6323/6346/6634`）`thread::spawn(reqwest::blocking).join()`；`shim_http_stream_next`（`stdlib.rs:6375`）与 `Iterator::HttpStream`（`native.rs:4719`）直接 blocking read。红测 `plan707_spike_stream_open_blocks_owner_until_headers`：上游 2s 断开前 shim 不返回。 | 红测 2 |
| G-C | **http.at 声明面不可达**：`Http.get_stream/stream_is_done` 在 codegen 裸 Http web 协议族路由表（`codegen.rs:8374`）缺席 → 静默 no-op（HTTP_STREAMS 空、段"完成"）；`http_stream.*` 命名空间 → `Undefined variable`（白名单 `codegen.rs:6300` 无 http_stream）。http.at:233-248 的 public 声明在 VM 轨是死面。 | 红测 2b |
| G-D | **SSE 逐 chunk lossy 玷污跨包 UTF-8**：`spawn_async_sse_stream`（`stdlib.rs:4153-4172`）按 chunk `from_utf8_lossy` 再找 `\n\n`；CRLF/CR 分帧不支持（只找 `\n\n`）；`trim_start` 丢前导空白；空 data 被过滤。红测 `plan707_spike_sse_utf8_split_across_chunks_survives`：「中」字 E4 后切开 → U+FFFD。 | 红测 3 |
| G-E | **每流一线程+runtime**：`spawn_async_sse_stream`（`stdlib.rs:4126`）每流 `auto-sse-client` 线程 + current-thread runtime；无帧字节上限（mpsc 容量 64 但帧大小无限）。 | 源码 |
| G-F | **generator 外部流等待热循环**：generator 驱动（`native.rs:4662-4667`）对 `StepResult::Yield` 只特判 cooperative sleep（`wake_time`），外部流等待（`waiting_sse_stream_id` + IP rewind）落入 1M 步预算内同 shim 反复重试 → `next_sse_generator_value`（`http_server.rs:5124`）`Ok(None)` 臂 `yield_now` 自旋直到数据到达。驱动次数随等待时长线性增长。 | 源码 |
| G-I | **iterator EOF 不推 -1**：`Iterator::HttpStream` EOF 臂（`native.rs:4740`）`None` 时不推 -1 直接返回（`sp == stack_before` → cooperative 驱动误判 pending）；`AsyncStreamIterator` 的 future_done 分支同形（`native.rs:4797-4801` 推 -1 但仅 cooperative 下正确）。 | 源码 |
| G-J | **TASK_LOOP 常规轮询**：wake source 4（`engine.rs:3088-3109`）每轮扫 `ASYNC_STREAMS`——功能正确但属固定轮询面，计划要求"未就绪等待无固定轮询"仅约束 HTTP parked/段路径（见 D-6 范围说明），task loop 保留（702 泵同款范围）。 | 源码 |

705 的非流式段执行、live-op 单次终结协议、presence 守卫、notify owner loop（`http_server.rs:3509-3529`）经源码复核为已交付且不回退（`cargo t plan705` 为 T-02 门禁）。

## 2. 冻结决策

### D-1 取消机制：AbortHandle 为主，插入后复查闭合竞态

- **managed job**（705 全部等待消费族：get/post/handle/builder/auth/json + 新流提交）：
  `submit_client_job` 升级为登记 `tokio::task::AbortHandle`（`JOB_ABORTS: Mutex<HashMap<u64, AbortHandle>>`）。
  `cancel_live_op(req_id)` = 删结果槽 + 取出并 `abort()` + 通知。
- **竞态闭合**（cancel 先于 handle 安装不漏失）：spawn 返回 handle 后先插入 `JOB_ABORTS`，
  再复查 `LIVE_OPS.contains(req_id)`——条目已消失即 cancel 抢先 → 立即补 `abort()`。
  spawn→insert 窗口内的 cancel 由该复查捕获；insert 后的 cancel 由 `cancel_live_op` 直取。
- **abort 时点**：任意 await 点（许可等待/请求建立/重试退避 sleep/读体/队满发送等待），
  wrapper task 整树丢弃 → queue/active 许可随 Drop 释放。abort 是协作式（下一 await 生效）；
  所有 job 全程网络 I/O，await 密集，接受该语义（与 reqwest drop=关连接一致）。
- **detached job**：`submit_detached_client_job(job)`（内部自配 id，不写 live-op、不登记
  abort）。唯一使用者 `spawn_async_http_msg_get`（`stdlib.rs:7440`）。不被 LIVE_OPS 缺席误杀，
  不受 `cancel_live_op` 影响（`shim_http_get_msg` 语义保留）。
- Future 取消不承诺撤销对端已接受的 POST/OS 已缓冲数据；只证明本地 job 停止读/重试、
  许可归还（AC-01 措辞边界）。

### D-2 执行器拓扑：共享 runtime，独立许可对

- 流 job 与非流式 job 共用 705 的固定 `ClientExecutor.rt`（线程数恒定，无新增 runtime/线程）。
- 流许可独立于非流式对：`stream_active=16`、`stream_queue=32`（`STREAM_EXECUTOR` 内第二组
  Semaphore）。长流不得耗尽非流式配额；队满/活跃满 = 提交即终结性错误（不排队不兜底）。
- 建立（headers 送达）期限 10s 内的流处于 Opening；建立失败 → Failed 终态。

### D-3 流资源表（新 `vm/ffi/http_stream.rs`）

```text
STREAMS: Mutex<HashMap<u64, StreamCell>>
StreamCell {
  state: Opening | Ready(item) | Pending | Eof | Failed(msg) | Cancelled
  item_queue: VecDeque<String>      // ≤16 条；单条 ≤256 KiB（事件/文本块）
  ready: Notify                     // 生产→消费就绪通知
  aborted: AtomicBool               // 终结幂等标记
}
STREAM_ABORTS: Mutex<HashMap<u64, AbortHandle>>   // 生产者 future 句柄
```

- 生产者（executor runtime 上）：queue 许可（try_acquire，满=终结错误）→ active 许可
  （await，可 abort）→ 建立请求（10s 期限）→ 读循环：
  - raw：`bytes_stream` 增量 UTF-8 carry 解码（跨 chunk 码点无损），按 ≤16 KiB 文本块入队；
  - SSE：喂 `SseDecoder`，按事件 data 入队；
  - 每条入队 `ready.notify_waiters()` + `COMPLETION_NOTIFY.notify_waiters()`（owner 泵唤醒）；
  - 队满停止拉上游（`send` 等待消费腾位——背压不误触读空闲期限，空闲期限只包上游 read）；
  - 读空闲 60s（每 chunk 重置）→ Failed("stream idle timeout")。
- **先登记后生产**：shim 同步登记 STREAMS 条目（Opening）后再提交生产者；迟到生产者
  `insert` 被拒（条目缺席/Cancelled 即退出）。
- 消费者 pull（`stream_pull(stream_id) -> Pull`）：`Data(String) | Pending | Eof | Failed(String)`。
  Pending 时消费方挂 `ParkedWait::HttpStream`/任务等待；EOF 先排空已入队 data 再终结。
- 取消/终结幂等：`stream_finalize(id, terminal_state)` 单次迁移；释放数据、abort 生产者、
  `STREAM_ABORTS` 出表、通知等待者。`stream_cancel`（close/break/scope 断连/下游 RST）与
  自然 EOF/Failed 同一收口。

### D-4 public 面冻结（签名/ABI）

| 源签名（http.at） | native id | 变化 |
|---|---|---|
| `get_stream(url) HTTPStream` / `post_stream(url,body)` / `post_stream_with_headers(url,body,headers)` | 2240/2241/2255 | 内部改异步建立（立即返句柄），签名不变；codegen 补路由（G-C） |
| `HTTPStream.next(self) str` | 2242 | 无数据 → 段挂起（`waiting_http_stream_id`），EOF 仍 `[DONE]` 哨兵 |
| `HTTPStream.is_done(self) int` | 2243 | Opening/Pending → 0；终态 → 1（纯探测不推进） |
| `HTTPStream.close(self) void` | 2244 | 语义升级：真正 stream_finalize（连接关闭） |
| `HTTPStream.iter(self) int` | 2245 | **新增 .at 声明**（`for c in s.iter()` 与变量 for 路由目标） |
| `sse_get_stream(url) int`（iterator id） | 3106 | 生产者迁 STREAM_EXECUTOR + SseDecoder；iterator 协议保留 |
| `sse_open(url) int` / `sse_poll(id) str` | 3145/3146 | 语义逐字节保留（`""` pending / `[DONE]` 终结） |
| `sse_close(id) void` | 3147 | **新增**：显式释放未完流 |
| `sse_error(id) str` | 3148 | **新增**：终态错误查询（""=无错；Failed 原因；不消费状态） |

- headers 兼容：VM JSON 字符串头（post_stream_with_headers）原样进 async 请求；不自动重试。
- raw HTTPStream 非 2xx：保留"读取 body 文本"既有可观察行为（建立即返句柄，status 不校验）。
- SSE GET 状态检查：status ≥400 → Failed("sse upstream status {code}")；Content-Type 不强制
  （现实端点缺失常见，误杀面大于收益；诊断带 status）。
- 迭代器终结凭流状态（Eof/Failed/Cancelled → -1），禁止字符串比较判定终结；`[DONE]` 载荷
  与空 data 是合法业务数据不丢。
- a2r/auto-man 侧流差异（线程/无界/close 占位）不在本轮，SD-05 登记差异即可。

### D-5 for-in 三形态装配（G5/AC-05）

1. **内联调用**：`for c in http.sse_get_stream(u)` / `for c in Http.get_stream(u)`（后者经
   新路由）——既有 Call 源迭代器通道（`codegen.rs:2807` 名含 stream/sse_ → CALL_NAT 112 循环）。
2. **变量形态**：`var s = Http.get_stream(u); for c in s { }`——codegen `Iter::Named` 非
   Range/非 Call 分支补 HTTPStream 类型识别（`var_types` 命中 `Type::Named("HTTPStream")`
   或流 native 返回登记）→ 走迭代器通道；`.iter()` 显式形态（D-4 id 2245）兜底可写。
3. **生成器嵌入**：`~Stream<str>` 体内 for 流 → CALL_NAT 112 于 generator 任务上执行，
   Pending 停步（D-6），ready 通知恢复。

### D-6 等待凭据与唤醒语义

- `ParkedWait::HttpStream(u64)` 新臂：`parked_is_ready`（http_server）与 `parked_wait_ready`
  （vm_bridge）= `stream_ready(stream_id)`（有 data 或终态）。
- `AutoTask.waiting_http_stream_id: Option<u64>`；CALL_NAT yield 重试协议沿用
  `waiting_sse_stream_id` 形态（IP rewind + Yield）。
- wake 范围（与计划 §5.3 一致）：HTTP 段 parked 泵（`drain_ready_parked`）与 UI 恢复泵走
  **事件唤醒**（COMPLETION_NOTIFY 三段式，零固定间隔）；`run_task_loop` 的 task 级 SSE/stream
  wake source 保留既有轮询形态（非本计划范围，G-J 记录）。
- **generator**：cooperative 驱动遇外部流 Pending → 返回 pending 并携带等待凭据；
  `next_sse_generator_value` 的 `Ok(None)` 臂改为对流的 `ready`/`COMPLETION_NOTIFY` 做
  enable→检查→await（宽裕超时兜底），删除 `yield_now` 自旋；驱动次数在 gate 关闭期间
  不随等待时长增长（AC-02 红转绿判据）。CPU sleep 特判（wake_time）行为保持。
- 同步 busy-wait 段（`call_fn_by_name` allow_busy_wait）：`waiting_http_stream_id` 臂比照
  HttpRequest 5ms 轮询 + 30s 超时（既有形态，AC-02 的"无热重试"约束不适用于 legacy 同步驱动，
  报告中注明）。

### D-7 所有权与资源组

```text
RequestScope（705 生命期许可 + cancel_notify）
  └─ ScopeResources { streams: Vec<u64>, live_ops: Vec<u64> }
       登记：流 open/提交 shim 执行时按 CURRENT_SCOPE_ID（thread-local，
       dispatch/resume 臂 RAII 设置）写入；无 scope（UI/CLI 程序）不登记。
       收口：finalize_scope（完成/取消/断连/关闭）遍历 abort —— 全组终结。
AutoTask.owned_stream_ids: Vec<u64>   # 兜底第二线
  - generator 任务: 体内 open 的流登记在 generator task;
  - cleanup_sse_iterator 扩展: Generator→回收 task+其流组; AsyncHttpStream/
    HttpStream→stream_finalize。
  - abort_parked_request 扩展: 取消 task 持有流。
消费退出（break/异常）: handler 内 → scope 收口兜底; 非 request 上下文 →
  显式 close（文档化语义），队满背压已止住上游拉取，无泄漏增长。
```

- **等待可取消 ≠ 关闭登记**：T-02 先证 cancel 真停 future（AC-01），T-06 再证组收口
  （AC-03）——两者分别有独立探针，不互相当作证据。
- SSE 输出关闭（FrameStream drop → scope 完成）级联取消 generator 内建立的上游流（D-7 组）。

### D-8 增量 SSE decoder（新 `sse/decoder.rs`）

- `SseDecoder::feed(bytes, out) -> Result<(), SseDecodeError>` + `finish()`；有状态：
  BOM 首见剥离；行终结 LF/CRLF/CR 三态（CR 后跟 LF 合并为一个终结）；字段解析：
  首个冒号切分、冒号后恰一个空格移除、其余空白保留；`data` 多行 `\n` 拼接；注释行忽略；
  `event/id/retry` 解析（NUL id 忽略、retry 非数字忽略）；空行仅在有 data 缓冲时分发；
  EOF 未闭合事件丢弃；`[DONE]` 是数据。单行/事件 carry 预算 256 KiB → `BudgetExceeded`。
- 与既有 helper 关系矩阵：`parse_sse_chunk`/`SSEParser`（`sse/parser.rs`）**保留原样**为
  legacy adapter（消费方：`shim_sse_parse`、`sse_parser_from_bytes` 使用者），行为不变、
  不迁移；新 live 路径一律走 decoder。两套差异（trim 语义、尾事件兼容）在 SD-01 中列明，
  不做无声统一。
- raw 流的 UTF-8 carry 解码独立于 SSE decoder（`utf8_carry_push` 助手）。

### D-9 下游写停滞与 frame 通道

- `FrameStream`（`http_transport.rs:428`）加 `last_poll: Instant`：相邻 poll 间隔 >30s →
  `Ready(None)`（body 终结 → scope 收口 → 组取消）。1s heartbeat 帧保持健康流活性；
  heartbeat 是传输注入输出，不伪造上游读活跃（60s 读空闲只在上游 read await 上计时）。
- frame 通道容量与字节上限沿用 699 冻结值；本计划不改 server 传输拓扑。

### D-10 限额冻结（env 可覆盖，首次使用读一次）

| 项 | 值 | env |
|---|---|---|
| 流 active | 16 | AUTO_HTTP_STREAM_ACTIVE |
| 流 queue | 32 | AUTO_HTTP_STREAM_QUEUE |
| 每流排队事件 | 16 | AUTO_HTTP_STREAM_MAX_QUEUED |
| 单事件/文本块 | 256 KiB（raw 文本块 16 KiB） | AUTO_HTTP_STREAM_MAX_EVENT |
| 建立/排队期限 | 10 s | AUTO_HTTP_STREAM_OPEN_TIMEOUT_MS |
| 上游读空闲 | 60 s | AUTO_HTTP_STREAM_IDLE_TIMEOUT_MS |
| 下游写停滞 | 30 s | AUTO_HTTP_STREAM_WRITE_STALL_MS |

- 内存上界模型（T-07 校验）：每流 ≤ 16×256 KiB 队列 + ≤256 KiB carry + ≤16 KiB 发送帧
  ≈ 4.25 MiB；16 活跃流 ≤ ~68 MiB 事件缓冲上界。不使用无限队列/缓冲。

### D-11 测试与门禁映射（按计划 §6 执行，命名冻结）

- 常规档：`plan707_spike`（T-01 红→逐绿）、`plan707_cancel`（T-02）、`plan707_decode`（T-03）、
  `plan707_client`（T-04）、`plan707_wait`（T-05）——`cargo t plan707` 可整族跑。
- 真 TCP E2E：测试名含 `http_e2e`，`#[cfg(all(test, feature = "test-http-e2e"))]`
  模块 `plan707_stream_e2e_tests.rs`（T-06/T-07），`cargo th` 串行池；
  交集命令 `cargo nextest run -p auto-lang --lib --features test-http-e2e -j 1
  -E 'test(plan707) & test(http_e2e)'`。
- 兼容基线门禁：`plan341`、`plan348`、`generator_tests`、`plan326`、`plan702`、`plan705`、
  `plan083`、`cargo t sse`。环境红必须 baseline 同命令复证定责。

## 3. 所有权图（T-06 实现蓝本）

```text
net 线程（固定 runtime shared）                VM owner 线程
STREAM_EXECUTOR ── stream producer futures      shim (segment entry)
  queue sem(32) → active sem(16)                  │ open: 登记 STREAMS(Opening) → 提交
  → reqwest async → SseDecoder/utf8-carry         │ next/pull: Data→推值 / Pending→park
  → bounded queue + Notify ───────────────────▶   ▼
        ▲ abort ←──────────────────────────  ParkedWait::HttpStream / generator await
        │                                      │
ScopeResources::streams ◀─ CURRENT_SCOPE_ID ───┘
  finalize_scope（reply/SSE body drop/RST/shutdown/断连）
  → 逐流 stream_finalize → abort producer → 许可/表回基线
```

## 4. 705 取消差距结论

源码证据 + 红测 1 定罪：**cancel 只终结结果槽，不停止执行体**（§1 G-A）。705 报告的
"取消=丢弃 job future"（async-http-result-lifecycle.md）在实现面不成立，本计划 T-02 闭合：
`cancel_live_op` 升级为「删槽 + abort + 通知」，queued/active/retry-sleep 三阶段探针
（AC-01）建立新证据。SD-02 修改在 merge 阶段沉淀，work 阶段不动 canonical Spec。

## 5. 风险与边界

- `HTTP_STREAMS` thread-local 表退役后，`Iterator::HttpStream`/`HttpStreamIterator`
  （`engine.rs:180`）改为持流 id 直查 STREAMS（全局表），thread-local 语义消失——单线程
  VM owner 无观察差异（G5 兼容门禁覆盖）。
- `ASYNC_STREAMS` 共用者（bus.subscribe/文件进度流）**不迁移**：仅接受「句柄 Drop/清理 =
  线程收割」共同适配；全面线程迁移是后续计划（非目标已声明）。
- abort 的协作性：极端 CPU 密集 job 段（无 await）不可打断——现有 job 全部 I/O 型，
  T-07 资源报告记录该边界。
- `cargo tf` 全量在 T-08 一次；开发期 `cargo check` + 定向族。画廊围栏冷态 ~800s 不在本轮
  （无 UI 布局改动）。
