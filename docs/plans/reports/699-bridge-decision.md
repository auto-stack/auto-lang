# PLAN-699 T-01 bridge decision（Axum/Hyper 传输拓扑与限额冻结）

> 状态：T-01 产出（2026-09-24）。本报告冻结桥接类型、线程拓扑、默认限额与已测得
> 的协议边界，是 T-02..T-06 的实现依据。证据全部来自本 worktree 的可复现 spike
> （`http_server.rs::tests::spike699_axum_transport`，6/6 通过）。

## 0. 前置核查：PLAN-698 折叠与 F-1

- F-1 修复 `b7ee6f302`（cleanup 回收 AsyncHttpStream 句柄 + Weak 转发线程自收割）
  已在 master；账本 `b586a402e`、归档 `045aa37c7`、cleaned 收据 `b41e9aa31`。
- `git worktree list` 无 lang-698 残留；`cleanup_sse_iterator` 的句柄回收测试
  （`plan698_f1_cleanup_tests::cleanup_reclaims_async_stream_handle`）在源内。
- **结论：T-02 起的依赖（PLAN-698 复审/折叠）已满足**；本计划的 SSE 改造以
  折叠后的 `ASYNC_STREAMS`/`bus_broadcast` 面为基线，不覆盖其改动。

## 1. 线程拓扑（选定）

```text
主线程（VM owner，现状不变）                      网络线程 "auto-http-net"（新建）
block_on_autovm_local(LocalSet)                  current_thread Tokio runtime
  execute_autovm_with_path → serve_async            accept 循环（tokio TcpListener）
  owner 循环: req_rx.recv()                          │ 每连接 tokio::spawn
    ├ 路由/中间件/按名绑定/call_fn_by_name            ▼
    ├ Response 对象/错误映射 → oneshot 回             hyper-util auto::Builder(http1_only)
    └ SSE: spawn_local producer → 帧通道               + TokioTimer + max_buf_size
                                                       + header_read_timeout
 Axum handler（网络线程，全 Send）                     + GracefulShutdown.watcher
  组装 owned ApiRequest ──有界 mpsc──▶ owner
  ◀── oneshot ApiReply（Full 或 Sse 帧通道）
  断连/超时/关闭 → drop 帧通道 → owner 收 closed → cleanup（iterator/task/订阅）
```

- **桥类型**：`ApiRequest`（全 owned：method/uri/选定 headers/body bytes/peer/request
  id）+ `ApiReply`（结构化 status/headers/body 或 SSE 帧通道）。HTTP 协议对象与
  socket 不进 VM；`usize` 洗指针与 `spawn_blocking` 调 VM 均不引入（AC-02）。
- **bind 错误回传**：网络线程经 ready 通道把 bind 结果（Ok(port)/Err(io::Error)）
  送回 owner，替代现状的 eprintln+return。
- **队列**：owner 入口有界 `mpsc::channel`；满载 `try_send` 失败立即 503（AC-03）。

### 否决的替代案

| 替代案 | 否决理由 |
|---|---|
| `axum::serve` 高层入口 | 不暴露 `max_buf_size`/`header_read_timeout`/`timer`；其 graceful shutdown 无强制关闭期限（卡死的 SSE 连接会无限阻塞关闭）。AC-03/05 要求显式上限与限时排空。 |
| 网络层与 VM 同 runtime（全局 runtime worker 跑网络任务） | 技术可行（网络 future 全 Send），但网络负载与 VM async 机器共享 worker；计划优先隔离，专属线程同时把 listener/连接生命周期收敛到单线程，teardown 语义清晰。 |
| `spawn_blocking` 调 VM | AC-02 明令禁止；`AutoVM: !Send`，阻塞线程池不产生真并行。`http_server.rs` 顶部旧注释声称的 "spawn_blocking directly calls the VM" 与事实不符，T-03 一并纠正。 |
| 保留手写解析器 | 即 G1 本身，无保留价值。 |

## 2. 冻结的限额与配置

| 项 | 值 | 依据 |
|---|---|---|
| header 总缓冲（`max_buf_size`） | 64 KiB | spike 实测：>64 KiB 头 → hyper 原生 **431 Request Header Fields Too Large**（非推断，报文已验证） |
| header 数量（`max_headers`） | hyper 默认 100 | hyper 文档：超限同样 431（与旧 64 KiB 字节上限互补） |
| 慢头超时（`header_read_timeout`） | 10 s | 与旧 `REQUEST_READ_TIMEOUT` 对齐；**必须**同时配 `Http1Builder::timer(TokioTimer::new())`，否则 armed timeout 即 panic（spike 实证，hyper `time.rs:80`）。超时行为=直接关闭连接（spike 实测 2.001s/2.013s 两轮精确关闭，无响应体） |
| body 上限 | 10 MiB（不变） | 桥侧 `axum::body::to_bytes(limit)` 有界增量读取，超限 413（旧语义）。chunked 请求体现在被支持（AC-01），同上限约束，无 Content-Length 不绕过上限 |
| body 接收总期限 | 10 s（408） | 旧代码对整请求读阶段一刀切 10 s；新代码以 `tokio::time::timeout` 包 body 收集复现同语义（hyper-util 0.1.20 的 Http1Builder 无 body read_timeout 暴露，见 §4 边界） |
| 在途请求队列（owner 入口） | 默认 64，env `AUTO_HTTP_MAX_INFLIGHT` | 满载 → 503 + `Retry-After`。解析失败或 0 → 回落默认 64（无隐式无限档） |
| 请求等待超时（排队+handler 回复） | 30 s，env `AUTO_HTTP_REQUEST_TIMEOUT_MS` | 超时回 503；若 handler 已同步开跑，仅丢弃迟到响应（无抢占，见 §5） |
| 监听地址/端口 | 不变：`0.0.0.0:$AUTO_HTTP_PORT`（默认 8080） | 计划非目标：安全缺省另立兼容性决策 |
| 排空期限（graceful shutdown） | 10 s | 停 accept → SSE 收到关闭信号收流 → 普通请求限时排空 → 超期强制关连接；端口释放已 spike 实证 |

## 3. SSE/关闭接线（T-05/T-06 依据）

- SSE reply = `Body::from_stream`（帧通道 receiver 适配器）。**适配器必须用
  `mpsc::Receiver::poll_recv(cx)` 接 waker**——spike 实证 naive `try_recv`+
  `Pending` 的 poll_fn 永不被唤醒，body 饿死。帧通道容量 1（与 PLAN-696 现状
  同背压），慢客户端阻塞 producer → generator 停批（旧语义保留）。
- 关闭信号用 `tokio::sync::watch` 广播到两个位置：(a) 帧通道适配器 select 关闭
  → 流正常收尾 → GracefulShutdown 能等完 SSE 连接（否则 SSE 永不"完成"，graceful
  挂死）；(b) owner 循环退出。断连（客户端走人）由 hyper drop body → rx drop →
  producer `closed()` 退出 → `SseIteratorCleanup` 回收 iterator/task/订阅（696/698
  语义原样，全部在 owner 线程析构）。
- CLI Ctrl+C：serve_async 默认装 `tokio::signal::ctrl_c()`（+ cfg(unix) SIGTERM）
  作为关闭信号；测试/内嵌用注入信号（可注入 `serve_with` 入参）。CLI 无需改动
  （`auto run` → `run_file_with_args` → `block_on_autovm_local` 已在 tokio 上下文）。

## 4. 已知边界（本计划内明确不解决）

1. **同步 CPU handler 无抢占**：owner 串行执行 handler；队列等待超时只能解除
   排队关系（客户端先收 503），正在同步执行的 VM 函数无法被 Tokio 中断。排队
   与队列上限保护的是网络层内存与等待时间，不是 handler 可中断性。handler
   可中断/多 VM 并行属 Design 33 阶段 C。
2. **HTTP/1.1 only**：auto Builder 钉 `http1_only()`，否则 h2c prior-knowledge
   连接会被 auto 检测悄悄升级协议面（超范围）。HTTP/2/3/TLS 不做。
3. **WebSocket echo**：现状是简化手写 echo（非通用 WS）。迁移后经 hyper upgrade
   路径原样保留（101 握手 + `hyper::upgrade::on` 后的裸帧 echo），能力边界不变；
   通用 WebSocket 支持另立（Design 33 §7.3）。
4. **body 慢滴**：body 空闲读超时 hyper-util Http1Builder 未暴露（仅 header 有）；
   以 body 总期限 10 s 兜底（超过即 408 断开），单字节/10s 的极端慢滴被总期限
   拒绝，不存在无界占用。
5. **错误响应的 keep-alive**：旧实现每次响应后关连接；新实现 4xx/5xx 默认保持
   连接（标准行为）。语义放松方向，E2E 断言按状态码/响应体判定，不受影响。

## 5. feature 边界

- `axum` 改为 auto-lang 无条件依赖（原 `ui-interpreter` 经 optional 隐式牵入）；
  `ui-interpreter` 特征表摘除 `dep:axum`（保留 `dep:ureq`）。`hyper`（仅
  `hyper::upgrade::on` 需要，default-features=false）与 `hyper-util`
  （`tokio/server-auto/server-graceful/service`）新增为直接依赖。
- 默认档（含 ui-iced）spike 编译+运行通过；`--no-default-features` 纯 VM 形态的
  编译核查在 T-03 验收执行（AC-01）。

## 6. Spike 证据清单

- 基线：master `d119881c5` + worktree `D:/autostack/.wt/lang-699/auto-lang`
  （分支 `plan-699-dev`，依赖位 `.wt/lang-699/auto-down` @ auto-down master `3373a5c` detach）。
- `cargo nextest run -p auto-lang --lib -E 'test(/spike699/)'`：**6/6 通过**
  （keep-alive 单连接双响应；chunked body 解码；>64 KiB 头 → 431；慢头 2 s 关闭；
  SSE 帧流经 Body::from_stream；graceful shutdown + 端口复绑）。
- `cargo th`（改动面外的既有 HTTP e2e 套件，spike 未触及旧服务路径）基线：
  见 `docs/plans/reports/699-verification.md`（T-08 汇总）与 plan 文档任务勾选行。
- 坑（沉淀）：nextest 默认 fail-fast，勘错轮须 `--no-fail-fast`；探针读响应的
  "单响应即 break" 假象（keep-alive 第二响应/SSE 流未等满）均系探针逻辑而非服务端。
