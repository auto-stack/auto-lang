# 共享 HTTP 客户端内核（http client runtime）

> **Status**: current（PLAN-724，Design 33 阶段 C2b）| 层：a2r-std http/client | 2026-10-02

## 执行模型

两条 Rust HTTP facade——独立 [`a2r_std::http`](../../a2r-std/project.md)
（a2r 转译产物的历史面）与 `auto_lang::a2r_std::http`（生成服务限定名
消费面，api_gen 发射 `auto_lang::a2r_std` 路径）——的**全部网络执行**经
单一共享内核 `a2r_std::http::client`：reqwest async + 固定 multi-thread
runtime（`OnceLock` 单例，默认 2 worker，`AUTO_A2R_HTTP_WORKERS` 覆盖）
+ 复用连接 Client。任何 facade 不得再持有 reqwest blocking client、
spawn_blocking 兜底、每请求线程或无界通道（PLAN-724 收口，全仓扫描守卫）。

```text
消费者 runtime（generated #[tokio::main] / 同步线程）
  │ execute(req).await / 流 next().await      （async 面；丢 future = 取消）
  │ execute_blocking(req) / 同步流 next()      （同步桥接；仅允许阻塞的边界）
  ▼
固定内核 runtime + 复用 Client
  │ 队列许可(try,满即 QueueFull) → 活跃许可(await) → 总期限包裹 → 增量读体
  ▼ owned HttpResponse { status, headers, body ≤ 预算 }

open_stream(spec)（非阻塞）→ 流队列许可(try) → spawn 生产者
  → 流活跃许可(await) → 建立(≤10s) → 读循环(idle 60s/chunk)
  → raw:Utf8Carry→≤16KiB / sse:SseDecoder→事件 → 有界队列(≤16×256KiB)
消费者 next().await → Data | Eof | Failed（单次终结）
close()/Drop → finalize(Cancelled) + abort 生产者（幂等、不复活）
```

## 调用与兼容矩阵

冻结矩阵（源符号/上下文/发射目标/失败语义）见
`docs/plans/archive/724-*.md` §5.1 与决策报告
`docs/plans/reports/724-client-decision.md` §1；要点：

- 普通动词 `get/post(两参)/put/delete` 与三参认证
  `post_sync/post_bearer(_sync)` **按元数分派**，返回形状独立
  （Response vs 认证 tuple），不冲突。
- 同步上下文走 `*_blocking`/同步桥接；async 上下文（`~T`/Future 返回 fn、
  含 `.await` 的 main、generator body、`.go` 块）由 a2r 发射器选择
  `*_async().await` 面（见 [http-client-lowering](../../auto-lang/trans/design/http-client-lowering.md)）。
- **同步桥接是响亮边界**：async 上下文内调用 `*_blocking`/同步流 next 被
  tokio 嵌套执行检测 panic（不做静默阻塞兜底、不给源位置诊断——转译期
  已按上下文选 async 面，panic 仅在绕过转译手写时可达）。
- `last_status` 线程局部仅作**同步兼容信息**；异步消费使用调用方 own 的
  返回值（owned metadata 归属本请求，不靠 TLS 关联并发请求）。
- headers 适配：Auto 表面 JSON 对象（`parse_json_headers`）与历史行格式
  （`parse_line_headers`）双形态入内核；坏 JSON/非字符串值/缺冒号/空键 =
  **终结性错误**（不按空 headers 发射）。VM shim 的旧行为
  （`unwrap_or_default()`）不在本约束内（VM 协议独立）。

## 错误分层

内核 typed，不吞成空成功：`QueueFull`（提交即拒）/ `Timeout`（总期限/
单请求）/ `BodyTooLarge{limit}` / `Transport(msg)` / `ExecutorClosed` /
`Cancelled`。流侧 `StreamItem::Data|Eof|Failed` 单次终结；SSE 非 2xx →
Failed（status 元数据仍如实呈现）；raw 流保留读取非 2xx body 的可观察
行为。facade 历史哨兵（传输失败 status=0、认证 tuple `(0, msg)`）在
适配层保留。

## 限额与内存算式（env 首次使用读一次；测试用独立 KernelInstance）

| 维度 | 默认 | env |
|---|---|---|
| 非流式 active / queue | 8 / 64 | `AUTO_A2R_HTTP_MAX_ACTIVE` / `_QUEUE` |
| 响应体预算 | 10 MiB（增量） | `AUTO_A2R_HTTP_BODY_LIMIT` |
| 单请求总期限 | 30 s（排队+建立+读体） | `AUTO_A2R_HTTP_TIMEOUT_MS` |
| 流 active / queue | 16 / 32 | `AUTO_A2R_HTTP_STREAM_ACTIVE` / `_QUEUE` |
| 每流队列 | 16 条 × 256 KiB | `AUTO_A2R_HTTP_STREAM_MAX_QUEUED` / `_MAX_ITEM` |
| raw 单块 | ≤16 KiB（取 min(16KiB, item 预算)） | — |
| 流建立 / idle | 10 s / 60 s（idle 只包上游 read） | `AUTO_A2R_HTTP_STREAM_(OPEN\|IDLE)_TIMEOUT_MS` |

单流峰值 = 队列 16×256 KiB（4 MiB）+ 行/事件 carry ≤ 2×256 KiB + 当前
网络块（reqwest/Hyper/TLS 内部缓冲**不入算式**）+ headers Vec；16 流
并发 ≈ 72 MiB + 非流式 8×10 MiB。逐项有界生产，无整响应累积。

## 取消与生命周期

- 非流式：调用方丢弃 `execute` future → oneshot 关闭 → 内核 job 在当前
  await 点（许可等待/建立/读体）退出并归还许可。
- 流：`close()`/`Drop` → finalize(Cancelled)（丢弃未消费数据）+ abort
  生产者；单次终结（首个终态胜出，迟到生产被拒）；EOF 先排空队列再终结
  （`is_finished` = 队列空 ∧ 终态）；abort 句柄 spawn 后登记，许可随
  任务退出异步归还。
- 局部 break 不冒充关闭（消费者变量仍归用户，显式 close 或作用域 Drop
  回收）；不承诺撤销对端已接受的 POST。

## SSE 解码（共享单源）

PLAN-707 的增量 decoder **原样提取**至 `a2r_std::sse`
（`SseDecoder`/`SseDecodeError`/`Utf8Carry`/`SseEvent`），auto-lang 侧
`crate::sse::decoder` 为逐字段映射 facade——VM 流生产者与 Rust 内核共用
同一实现（行为零漂移，`plan707_decode` 回归守卫）。707 冻结子集（BOM、
三态行终结、字段规则、`[DONE]` 是数据、EOF 丢未闭合事件、256 KiB 预算）
与 legacy `sse::parser` 的差异矩阵维持不动。

## 支持矩阵

- **已支持**：决策报告 §1 矩阵 21 面（普通动词/builder/认证
  sync+async/流 sync+async/自由函数/for-in/Response 访问器/last_status）；
  编译运行证据 = `test/a2r/30_plan724/` 金样 + `http_e2e_plan724_*`
  （含 `auto_lang::a2r_std` 限定名腿）+ `crates/a2r-std/tests/http_client.rs`
  原生矩阵。
- **非目标**：文件 helper（download/upload/download_resume）仍走 ureq
  （不为删除依赖扩展范围）；`sse_open/sse_poll/sse_close`（VM int id 族）
  不在 a2r 支持集；`stream_iter` 仅 for-in 位置有语义；TLS 服务端/
  HTTP/2/3/自动重连/WebSocket 不变（707 边界沿用）；任意 `#[api] ~Stream`
  handler 的 Rust 生成另案。

## 关联

- PLAN-724（reports: 724-client-decision / 724-client-parity /
  724-resource-lifecycle / 724-verification）
- [a2r-std project](../project.md)（模块卡）
- [http-stream-lifecycle](../../stdlib/design/http-stream-lifecycle.md)
  （VM 轨外部流；park/resume 协议为 VM 专有，Rust 侧为 await/通知等待）
- [async-http-result-lifecycle](../../stdlib/design/async-http-result-lifecycle.md)
  （VM managed job 协议；Rust 内核独立实现，协议同构）
- [http-client-lowering](../../auto-lang/trans/design/http-client-lowering.md)
  （发射面契约）
