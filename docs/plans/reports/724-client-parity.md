# PLAN-724 T-07 对拍报告：VM / a2r / 原生 Rust 约定语义一致性

- 阶段：work（T-07）
- 载具：worktree `D:/autostack/.wt/lang-724/auto-lang`，分支 `plan-724-dev`
- 冻结依据：`724-client-decision.md`（T-01）

## 1. 覆盖腿与命令/退出码

| 腿 | 载体 | 命令 | 结果 |
|---|---|---|---|
| a2r 发射冻结（sync main） | `test/a2r/30_plan724/001_http_client_sync/` | `cargo t plan724_golden` | PASS（2/2 golden） |
| a2r 发射冻结（async main） | `test/a2r/30_plan724/002_http_client_async/` | 同上 | PASS |
| a2r 产物**实编实跑** | 临时 cargo 工程（path 依赖 worktree a2r-std，内容哈希缓存） | `cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan724`（即 `cargo th` 收集的 `http_e2e` 过滤器） | 3/3 PASS（sync 10.3s / async 7.8s / qualified 2.1s；产物 cargo run 退出码 0，wire 断言经回环 stub） |
| 发射形状探针 | `plan724_http_client_tests.rs` 探针三例 | `cargo t plan724` | 5/5 PASS |
| 原生 Rust typed 交叉核验 | `crates/a2r-std/tests/http_client.rs`（全局 facade + 内核直发） | `cargo test -p a2r-std --test http_client -- --test-threads=1` | 7/7 PASS |
| 第二 facade（`auto_lang::a2r_std::http`，生成服务限定名消费面） | `http_e2e_plan724_qualified_facade_kernel`（真 TCP） | 同 th 命令 | PASS（4-tuple 形状、500 值语义、同步桥接、传输失败 status 0） |
| 内核单测（独立小预算实例） | `src/http/client.rs` tests（36 例） | `cargo nextest run -p a2r-std` | 43/43 PASS（含 sse/sse facade 复用例） |
| VM 侧回归（707 面，共享 decoder 提取零漂移） | `plan707_client` / `plan707_decode` | `cargo t plan707_client && cargo t plan707_decode` | 8/8 + 9/9 PASS |
| P707-R1 / 712 错误分层回归 | e4 + plan712 | `cargo t default_headers… && cargo t plan712_http_error_semantics` | 绿（T-02 在案；2/2） |

## 2. 约定语义对拍结论（§5.1 必选集）

同源 Auto 样本（`http_client_sync.at` / `http_client_async.at`，`PLAN724_URL`
env 注入 stub 地址）在 a2r 转译产物中的行为与原生 Rust typed 直发一致：

- **普通请求**：get/post(2参)/put/delete → `Response{status 200, header
  大小写无关, body bytes}`；两参 post 不触三参认证分支（元数分派，探针钉住）。
- **builder**：`request().header().body().timeout()` 链与变量重绑定分型；
  sync `send()` / async `send_async().await`；实编实跑 wire 断言 echo 一致。
- **认证**：`post_sync/post_bearer_sync`（sync）与 `*_async`（async 上下文）
  tuple 形状逐字节保留；`x-api-key`+`anthropic-version` / `Authorization:
  Bearer` 落线（stub 回显断言）。
- **流**：`get_stream/post_stream/post_stream_with_headers` → 手柄；
  for-in EOF 哨兵 ""；**break 不冒充关闭**（break 后 `is_done()==0`，显式
  close 后 `==1`）；raw 模式跨帧 UTF-8 carry 无损（`data: 中文…` 分帧切码点）；
  SSE 子集（Sse 模式直发）事件序列 `["中文", "[DONE]"]`、CRLF 单终结符。
- **取消/背压**：60 流风暴后许可回基线（有界重试准入成功）；慢消费者高水位
  有界；current-thread/multi-thread 消费者定时器 tick 先于完成（reactor 不
  阻塞）；同步桥接在 async 上下文响亮 panic（诊断面）。
- **legacy 差异单列**（不冒充一致）：
  - VM 的 `sse_open/sse_poll/sse_close`（int id 族）为 VM 专有面，a2r 不
    支持（流手柄 + for-in 表达同等消费）；
  - VM shim 的 headers JSON 解析失败 `unwrap_or_default()`（空 headers 发射）
    旧行为未动（VM 协议独立）；a2r/内核面为终结性错误（冻结决策）；
  - `last_status` 仅同步兼容信息，不承诺并发关联（决策 §1 #11）。

## 3. 生成 db 限定名链接代表

api_gen 生成物以 `auto_lang::a2r_std` 限定名引用第二 facade（`crates/auto-man/src/api_gen.rs`
发射形态，本计划不改 generator）。代表性验证 = `http_e2e_plan724_qualified_facade_kernel`
直接链接并调用 `crate::a2r_std::http::{post, post_sync, post_bearer_sync}`，
路径存在性与行为均实证。**不宣称**任意 `#[api] ~Stream` handler 的 Rust 生成
已落地（另案范围，计划 §0 边界）。

## 4. 边界与未覆盖（诚实清单）

- VM 模式的同源运行未在本计划实跑（VM http 消费走 VM ffi 流/id 面，其回归
  由 `plan707_client`/`plan707_stream_*`/e4 家族覆盖；同源 .at 的 VM 臂复用
  同一 stdlib 声明，行为面差异已在 §2 单列）。
- 公网服务、TLS 服务端、HTTP/2/3、自动重连、WebSocket 不在覆盖（决策 §4）。
- 取消风暴针对流面；非流式 job 的取消由内核单测
  `plan724_kernel_cancel_dropped_future_stops_job_and_returns_permit` 与
  `plan724_kernel_queue_full_is_immediate_terminal` 覆盖。
