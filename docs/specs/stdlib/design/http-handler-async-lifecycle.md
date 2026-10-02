# HTTP handler 异步等待与请求生命周期(http handler async lifecycle)

> **Status**: current(PLAN-705,Design 33 阶段 C1;PLAN-707 C2a 外部流延伸) | 层:vm ffi http_server/engine | 2026-09-30

## 执行模型

VM `#[api]` HTTP handler（named `fn() T` / `fn() ~T`、middleware 链、
`__axum:` fn-ref closure）由**段驱动**执行：handler 在等待上游异步资源
（client HTTP / External Future）时 `Yield` → 请求以
`ParkedRequest`（任务 + `ParkedSegment` 续体 + 就绪凭据 + 作用域）挂入
owner parked 表，**owner loop 立即空出**服务后续请求；完成端经全局
`COMPLETION_NOTIFY`（`Notify::notify_waiters`）唤醒，owner 侧
`notified().enable()` → 检查 → await 三段式登记（零固定间隔轮询、零丢
唤醒）。每执行段内顺序执行；同一时刻仍只有一个 VM 段在跑。

**跨 await 非原子事务**：A 等待上游时 B 可读写同一 VM 共享状态；A 已
提交的副作用不因取消/交错回滚。业务需在 `.at` 源码显式处理跨 await
状态一致性。

## 入口矩阵

| 入口 | 段形态 | 边界 |
|---|---|---|
| named handler | `call_fn_by_name_segment` | legacy 同步 `call_fn_by_name` 仅存于非默认调用图（串行 stdnet server，源码 gate 锁定恰 1 处保留位） |
| middleware 链 | 逐个段驱动；middleware park 挂起整个请求，恢复后续链（Err→视为无响应继续，352 语义） | 同上 |
| `__axum:` closure | `call_closure_segment`（帧建立与 `call_closure` 逐字节同） | 全链路 fixture 依赖 musk sibling；段入口以合成 fn-ref closure 单元覆盖 |
| RequestBuilder `.send` | 段模式跳过同步 drain：rewind ip−6 + Yield park；重入凭据=`waiting_http_request_id`（与 CALL_NAT shim 重入协议同构） | 忙等 drain 保留给 legacy 调用图 |
| `~T` 返回 | 编组期**元数据门**（`API_ASYNC_RETURNS`，codegen 按 `Future<T>` 声明发布；`__axum:` 经 func_addr→exports 反查）+ future bits 形态 + 注册表存在性三重闸 | 普通 int 位模式（如 240=0xF0）永不误判为 future |
| `~T` 三形态 | External 挂起（AwaitReturnFuture）/ Internal 体就地驱动（`handle_await_future`）/ 体挂外层转 AsyncReturnBody | Failed/缺失→null（Phase A 语义） |

## 请求作用域与取消

- **生命期许可**：`AUTO_HTTP_MAX_INFLIGHT` 自 PLAN-705 起为
  **queued+running+parked 生命期总上限**（旧"队列容量"语义升级，兼容性
  变化已明示）；桥入队前 `try_acquire`，满载 503+`Retry-After` 零排队。
  mpsc 队列容量仍同值（队满 503 先发生）。
- **取消三类确证判据**（仅此三类）：回复 deadline 到期（桥 select 臂，
  `AUTO_HTTP_REQUEST_TIMEOUT_MS`）、shutdown、连接任务确证终结
  （net 侧 watcher 挂 serve_connection，任务退出=证据；同连接多请求一并
  取消）。**半关闭/输入 EOF 不判取消**（hyper `half_close(true)`——客户端
  半关写端仍读响应）。
- **取消语义**：`cancel_scope` 幂等终结（许可释放+登记移除+取消信号）；
  owner 出队臂对已失效请求**跳过业务函数**直接 503；parked 失效废弃并
  回收 live-op（迟到完成被 presence 守卫丢弃，永不复活）。TCP 黑洞类
  不可观测断连由 deadline 有界收口。
- **SSE**：许可随 scope 移交响应体（`FrameStream` Drop 释放）；流结束/
  断连/关闭均触发。排水窗内自然完成的请求照常回复（
  `AUTO_HTTP_SHUTDOWN_DRAIN_MS` 可调，默认 10s）。

## 完成通知与结果通道

- 完成端统一 `complete_live_op`（见
  [async-http-result-lifecycle](async-http-result-lifecycle.md)）+ 通知；
  External Future 完成统一 `complete_external_future`。
- owner parked 就绪探测：HttpRequest→live-op 表、Future→`vm.futures`
  （缺席=唤醒，引擎恢复臂同款 nil fallback）、**HttpStream→流资源表**
  （PLAN-707：有 data 或终态即就绪；CALL_NAT rewind 重试协议同构——
  挂起臂回推被弹的迭代器 id、就绪出口清除凭据，残留标志=热循环根因）。

## 外部流等待与资源组（PLAN-707 C2a）

- **流等待**：外部 HTTP/SSE 流的建立/读取全部异步非阻塞（统一资源表 +
  固定共享 runtime + 独立许可对 + 有界队列背压）——详见
  [http-stream-lifecycle](http-stream-lifecycle.md)。handler/generator
  段遇流无数据即 park（`ParkedWait::HttpStream`），零阻塞读、零热转
  （gate 关闭期间驱动次数不随等待时长增长）。
- **generator 等待凭据化**：cooperative 驱动遇流/External Future 等待
  立即停步（AwaitFuture 走 `handle_await_future` 处理）；SSE serve 的
  `next_sse_generator_value` 对流做 enable→检查→await，删除 yield_now
  自旋。
- **请求资源组**：段内打开的上游流登记进 `RequestScope.resources`
  （finalize_scope 逐流取消——组内资源不越过请求生命期）；generator
  体首次 pull 发生在 SSE serve 循环（scope 守卫已退出），以
  `AutoTask.owned_stream_ids` 第二线承载（`cleanup_sse_iterator`/
  `abort_parked_request` 收口）。下游断连 → scope 收口 → 上游连接
  真实关闭（级联取消，E2E 观测 ≤8s）。
- **期限区别**：首响应 deadline（`AUTO_HTTP_REQUEST_TIMEOUT_MS`）只
  覆盖排队+handler 到回复；SSE 回复后 scope 移交 `FrameStream` 代持，
  健康长流不被首响应期限杀死——上游读空闲（60s，背压等待不计时）/
  下游写停滞（30s，传输侧）分别收口。

## 支持矩阵（边界明示）

- **已支持**：上述全部（探针族 `plan705` 19 项 + 门禁 gate）。
- **非目标（本轮明确不做）**：CPU-bound handler 抢占/时间片（段预算
  10M 指令跑完不可中断）；全后台收敛/全量 task actor mailbox 改造；
  a2r 客户端迁移；TLS/HTTP2/WS 扩展；多 VM/多 owner 并行。~~外部
  SSE/HTTPStream 的异步等待~~（PLAN-707 C2a 已落地——见
  [http-stream-lifecycle](http-stream-lifecycle.md)；a2r 侧流客户端
  迁移仍为缺口）。
- **legacy 保留**：`serve_blocking_stdnet` / `run_http_server_blocking` /
  `shim_http_server_listen` 串行同步形态；`*_sync` 客户端显式同步 API
  （不入 request scope）。

## 关联

- PLAN-705（reports：705-async-decision / 705-parity / 705-resource-lifecycle / 705-verification）
- PLAN-707（reports：707-stream-decision / 707-parity / 707-resource-lifecycle / 707-verification；
  [http-stream-lifecycle](http-stream-lifecycle.md) 为本篇外部流延伸契约）
- [http-server §8.1](http-server.md)（协议预算/装配）、
  [async-http-result-lifecycle](async-http-result-lifecycle.md)（结果通道单次终结）
- [networking-stdlib](../../auto-lang/runtime/design/networking-stdlib.md)（阶段 C1 落地面）


## PLAN-729：文件 body 的 scope 与期限

- 文件回复与 SSE 同形：scope/许可随响应体代持（`FileBodyAdapter` Drop/pump finish
  恰一次幂等终结），**不沿用普通 Text 回复的立即 `complete_scope` 分支**。
- 两期限交接：handler 段沿用既有 30s deadline；回复送达后文件准备
  （等许可+open+metadata+seek）取 min(30s, scope 剩余)；headers 之后转入 body
  idle watchdog（60s 默认，独立计时任务覆盖不被 poll 的客户端）+ 可选总期限。
  大下载不被 handler deadline 误杀。
- 在途 FS 读不强制中断：取消 = 停止 issuance + 等收口；active 执行槽随 pump 退出
  归还（逻辑取消与 FS 实际退出分开计数）。契约详见
  [http-server-files §5](http-server-files.md)。

## 服务端上传资源组与分阶段期限（PLAN-730）

- **body capability / 上传资源组**：上传请求的原始 body 不在桥侧消费——
  `UploadRequest`（注入句柄）/`UploadSession`（会话）/`UploadReceipt`（收据）三类
  id 登记请求 scope 资源组，`finalize_scope` 幂等收口（取消在途接收、清理 staged、
  释放未消费 body 能力、移除闲置收据）。收据构造即登记（未编组收据不滞留注册表）。
- **分阶段期限**：普通请求期限（30s）覆盖 header/鉴权/队列；进入 receive 后经
  scope deadline watch 切换到上传 total（10min，headers 入 scope 起，排队计入；
  桥 reply 等待循环重臂，不保留旧捕获值）；接收 idle 60s 逐 chunk 强制；
  staged lease 30s（业务判定窗口）。准入阶段（queue+active 等待）统一受队列
  期限（默认 30s）——满额下一笔 503 于期限内，不吊到 total。
- **仲裁**：commit gate=hard_link 原语本身——gate 前取消胜出（不发布+清理），
  gate 后迟到取消等实际发布结果（成功保留）；scope 终结 ≠ 在途 FS 立即退出
  （writer 收口后清柄/腾槽）。
- 详细契约见 [http-server-uploads](http-server-uploads.md)。
