# PLAN-724 T-01 决策报告：客户端调用矩阵、上下文与 runtime 方案冻结

- 阶段：work（T-01）
- 基线：worktree `D:/autostack/.wt/lang-724/auto-lang`，分支 `plan-724-dev`；
  实施起点 commit `986e765ac`（起草 HEAD）；关键交付节点 `7e7c6b3f8`（T-02）、
  `14e9cc389`（内核）、T-05/T-06 提交（见 plan 复审记录）。
- 性质：本报告冻结 §5.1 调用矩阵、同步边界、限额/排队、错误与解码子集。
  全部决策**有实编译/实运行证据**（kernel 单测 36 例、转译 golden 2 件、
  发射探针 3 例、编译运行 e2e 2 件、`plan712`/`plan707_client`/`plan707_decode`
  回归绿），不是纸面推断。

## 1. 调用与兼容矩阵（冻结）

「源符号/类型/元数 → 上下文 → 发射目标 → 返回/失败语义 → 证据」：

| # | Auto 源面 | 上下文 | 发射目标（a2r 产物） | 返回/失败语义 | 证据 |
|---|---|---|---|---|---|
| 1 | `http.get(url)` | 同步 fn | `a2r_std::http::get(url)` → 内核同步桥接 | Response；传输失败 status=0（历史哨兵），不吞成空成功 | 001 golden L12；e2e sync `get status: 200` |
| 2 | 同上 | async fn/main（`~` 返回或 main 含 `.await`） | `a2r_std::http::get_async(url).await` | 同上；等待让出执行线程 | 002 golden L20；e2e async |
| 3 | `http.post(url, body)`（两参） | 同步/async | `post` / `post_async`（元数分派） | Response 同上 | 探针 `plan724_probe_post_arity_dispatch_shapes` |
| 4 | `http.post(url, body, key)`（三参） | 任意 | `async { a2r_std::http::post(...).await; HttpResponse{..} }`（历史认证面，不变） | 4-tuple HttpResponse | 同上；`auto.http.post` 臂未动 |
| 5 | `http.put/delete` | 同步/async | `put/delete` / `*_async` | 同 #1 | 001 golden / 002 golden |
| 6 | `http.request(m,u).header().body().timeout().send()` | 同步 | `a2r_std::http::request(...)` 链 + `.send()` | Response | 001 golden；e2e sync `builder body: echo:builder-payload` |
| 7 | 同上（链或变量重绑定 `let b2 = b.header(..)`） | async | 链根识别 + `http_builder_vars` 分型 → `.send_async().await` | 同上 | 002 golden L34；探针 builder |
| 8 | `http.post_sync(url, body, key)` / `post_bearer_sync` / `get_sync` | 同步 | 同名 facade（内核同步桥接）；TLS `last_status` 由发射的 `set_last_status(__resp.0)` 写 | `(u32, String)` / `(u32,String)`；传输失败 `(0, "transport error: …")` | 001 golden；T-02 证据 |
| 9 | 同上 | async 上下文 | `post_sync_async` / `post_bearer_sync_async` / `get_sync_async` **`.await`** | 同上（async 面不阻塞 reactor） | 002 golden L48；e2e async `auth status: 200` |
| 10 | `http.post_bearer(url, body, key)` | 任意 | `post_bearer(...).await`（本就 async 面） | `(i32, String)` | 历史发射不变 |
| 11 | `http.last_status()` | 任意 | `a2r_std::http::last_status()` | TLS 快照；**只作同步兼容信息**，不承诺并发 task 关联；async 消费用返回值 own 状态 | 001 golden L47 |
| 12 | `http.get_stream/post_stream(url[,body])` | 同步 | `get_stream/post_stream` → facade `HTTPStream`（`Mutex<HttpClientStream>` 内部可变） | 流手柄；EOF 哨兵 ""（VM 契约） | 001 golden L38-44 |
| 13 | 同上 | async | `*_async(...).await` → `AsyncHTTPStream`（`Arc<tokio::Mutex<…>>`） | 同上 | 002 golden L39 |
| 14 | `http.post_stream_with_headers(url, body, headers)` | 同步/async | 同名 `*`/`*_async`；headers 按**行格式**（`Key: Value`）解析；格式错误=流终结错误（不按空 headers 发射） | 流手柄 | 002 golden；内核 headers 适配器测试 |
| 15 | `s.next()/s.is_done()/s.close()`（s 为流变量） | 同步 | 方法直发（`&self` 内部可变，let 无需 mut） | `str`（""=EOF）/int/void | 001 golden |
| 16 | 同上（s 在 async 上下文产出） | async | `stream_next_async(&s).await`（Option 在 facade 折叠为 "" 哨兵）；`is_done/close` 直发（try_lock，async 安全） | 同上 | 002 golden L41；内核 reactor 测试 |
| 17 | `stream_next/stream_is_done/stream_close(s)`（自由函数） | 同步/async | 按变量分型选 `stream_*` / `stream_*_async` | 同上 | 探针 `plan724_probe_stream_free_fn_typed_dispatch` |
| 18 | `for c in s.iter()` / `for c in stream_iter(s)` / `for c in http.get_stream(u)` | 同步/async | `loop { let c = stream_next[_async](&recv)[.await]; if c.is_empty() {break;} … }`；生产调用形态用临时拥有绑定 `__hs` | EOF 哨兵终止；**局部 break 不冒充关闭**（接收者仍归用户，显式 close/作用域 Drop 回收） | 001/002 golden；e2e `chunk:` 断言 |
| 19 | `Response.status_code()/header_get(k)/body_bytes()` | 任意 | 纯 Rust 方法（kernel Response typed） | 状态 u16；**大小写无关** header；body bytes | e2e `get header: alpha`；内核 roundtrip 测试 |
| 20 | `http.download/upload/download_resume` | 同步 | ureq 文件 helper（**不在本计划收敛**，§3 边界） | 状态码 | 未改 |
| 21 | `sse_open/sse_poll/sse_close/sse_error`（VM id 族） | VM | **不在 a2r 支持集**（VM 迭代 id 机制，PLAN-707 域；a2r 用流手柄 + for-in 表达） | — | 本报告 §4 边界 |

## 2. runtime 与同步边界（冻结）

1. **单内核**：`a2r_std::http::client` 一个固定 multi-thread runtime（默认 2
   worker，`AUTO_A2R_HTTP_WORKERS` 覆盖）+ 复用连接 `reqwest::Client`。两个
   facade（独立 `a2r_std::http` 与 `auto_lang::a2r_std::http`）全部网络执行经
   此；两 facade 与内核**零重复网络执行**（`reqwest::blocking`/`spawn_blocking`
   全仓扫描仅存于文档注释，`grep -rn` 在案）。
2. **async 消费**：`execute(req).await` / `stream.next().await` 只做通知等待；
   current-thread 与 multi-thread 消费者均不阻塞（内核
   `plan724_kernel_current_thread_consumer_reactor_stays_live` /
   `…_stream_wait_yields`：慢上游期间本地定时器持续 tick，健康事件先于 gate
   放行）。e2e 覆盖 `#[tokio::main]` multi-thread 形态。
3. **取消传播**：调用方丢弃 `execute` future → oneshot 关闭 → 内核 job 在当前
   await 点（许可等待/建立/读体）退出并归还许可（`plan724_kernel_cancel_dropped_future_stops_job_and_returns_permit`
   实证许可回基线）；流 close/Drop → finalize(Cancelled) + abort 生产者
   （`plan724_kernel_stream_close_cancels_producer_and_returns_permit`）。**不是
   每请求 runtime/线程/blocking client**。
4. **同步桥接 = 响亮边界**：`*_blocking`/同步 `HTTPStream::next` 仅在允许阻塞
   的边界（同步 fn、非 runtime 线程）合法；async 上下文内调用被 tokio 嵌套
   执行检测 **panic**（`plan724_kernel_sync_bridge_panics_loudly_in_async_ctx`
   钉住；不提供静默阻塞兜底，不支持位置给源诊断——转译期已按上下文选 async
   面，panic 只在绕过转译直接手写时可达）。
5. **async 上下文判定**（转译期）：返回 `~T`/`Future` 的 fn、含 `.await` 的
   main、generator body（async_stream）、`.go` spawn 块 = async；其余 = 同步。
   同步声明 helper 被异步路径调用属**不支持组合**——运行期得到上述响亮诊断。

## 3. 限额、排队与配置（冻结）

默认值对齐 705/707 交付值；env 首次使用读一次；测试用独立
`KernelInstance::new(ClientLimits, StreamLimits)` 小预算实例（不碰进程 env、
不受全局初始化顺序影响）：

| 维度 | 默认 | env | 预算上限说明 |
|---|---|---|---|
| 非流式 active | 8 | `AUTO_A2R_HTTP_MAX_ACTIVE` | 排队等待计入单 job 总期限（无独立排队期限旋钮——队满即拒绝，等待方有总期限兜底） |
| 非流式 queue | 64 | `AUTO_A2R_HTTP_QUEUE` | 队满 → 提交即 `QueueFull`（终结性，绝不临时 spawn） |
| 响应体预算 | 10 MiB | `AUTO_HTTP…BODY_LIMIT`→`AUTO_A2R_HTTP_BODY_LIMIT` | Content-Length 预检 + 增量累计双闸 |
| 单 job 总期限 | 30s | `AUTO_A2R_HTTP_TIMEOUT_MS` | 排队+建立+读体全程 |
| 流 active/queue | 16 / 32 | `AUTO_A2R_HTTP_STREAM_(ACTIVE\|QUEUE)` | |
| 流每流队列 | 16 条 × 256 KiB | `AUTO_A2R_HTTP_STREAM_MAX_QUEUED/MAX_ITEM` | 背压高水位；队满生产者停在 space 等待（不计入 idle） |
| raw 单块 | ≤16 KiB（取 `min(16KiB, item 预算)`） | 常量 `RAW_CHUNK_BYTES` | UTF-8 carry 后切分，码点边界安全 |
| 流建立/idle | 10s / 60s | `AUTO_A2R_HTTP_STREAM_(OPEN\|IDLE)_TIMEOUT_MS` | idle 只包上游 read |

**内存算式（单流）**：队列 16×256 KiB（4 MiB 峰值）+ 行/事件 carry ≤ 2×256 KiB
+ 当前网络块（reqwest/hyper 内部缓冲，**不入算式**）+ 保留 headers Vec。16 流
并发乘积上限 ≈ 72 MiB + headers。逐项有界生产，无整响应累积 Vec。

## 4. 错误与解码子集（冻结）

- **错误分层**（内核 typed，不吞成空成功）：`QueueFull`（提交即拒绝）/
  `Timeout`（总期限/单请求）/ `BodyTooLarge{limit}` / `Transport(msg)` /
  `ExecutorClosed` / `Cancelled`。流侧 `StreamItem::Data|Eof|Failed(msg)`
  单次终结；SSE 非 2xx → Failed（status 元数据仍如实呈现）。
- **headers 适配器**：Auto 表面 JSON 对象（`parse_json_headers`，VM shim 同
  形态）；历史 Rust 行格式（`parse_line_headers`）。坏 JSON/非字符串值/缺冒号/
  空键 = **终结性错误**（不按空 headers 成功发射）——内核/a2r 面收紧；VM shim
  的 `unwrap_or_default()` 旧行为本计划不动（VM 协议独立，712 文档修正归
  SD-04）。
- **SSE 子集**：共享 decoder = PLAN-707 `sse::decoder` **原样提取**至
  `a2r_std::sse`（`SseDecoder`/`SseDecodeError`/`Utf8Carry`/`SseEvent`），
  auto-lang 侧 facade 逐字段映射复用（`plan707_decode` 9 例全绿 = 行为零
  漂移）。子集规则与 legacy `sse::parser` 差异矩阵维持 707 冻结（BOM 一次、
  LF/CRLF/CR 三态、多 data \n 拼接、空 data 不分发、`[DONE]` 是数据、EOF 丢
  未闭合事件、256 KiB 行/事件预算）；自动重连 / Last-Event-ID replay /
  WebSocket / HTTP/2 不在本计划。
- **EOF/哨兵**：内核 typed `Eof` 先排空队列再终结（`is_finished` = 队列空 +
  终态，上游写完不丢尾）；Auto 手工接口 EOF 以 `""` 哨兵呈现（VM 契约），
  raw 分片天然非空，无歧义。
- **单次终结**：Eof/Failed/Cancelled 首个终态胜出；close 幂等可重复；取消
  丢弃未消费数据；迟到生产者被终态检查拒绝（不复活）。

## 5. 不支持/需诊断的边界（诚实清单）

- 同步声明 helper 从 async 路径调用：运行期响亮 panic（§2.4/2.5）；转译期
  不做传染改写（不做全语言隐式 async）。
- `stream_iter(s)` 只在 for-in 位置有语义；脱离 for-in 直接取 id 无 a2r 形态
  （facade 无此符号，编译期即失败——诚实诊断，不提供假 id）。
- `sse_open/sse_poll/sse_close/sse_error`（VM int id 族）不在 a2r 支持集
  （§1 #21）。
- 流 `Drop` = close（幂等取消）；「break 后继续用同一流」合法（未 close），
  「break 后认为已关」不成立——break 不冒充关闭。
- reqwest/Hyper/TLS 内部缓冲不计入内存上限（§3）；不承诺撤销对端已接受的
  POST。

## 6. 与 P707-R1 的关系

T-02（先于内核）已红转绿：e4 直驱臂未 `register_live_op` 被 707 取消竞态
守卫中止——测试改走生产协议（register→submit→consume/cancel 配对 + 未登记
反例钉），取消守卫零改动；`docs/plans/KNOWN-DEBT-AND-RISKS.md` P707-R1 已
凭证据清偿。本计划的内核是 a2r 面的独立实现，不受该修复影响。
