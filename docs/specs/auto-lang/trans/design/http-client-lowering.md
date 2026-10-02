# a2r HTTP 客户端发射契约（http client lowering）

> **Status**: current（PLAN-724）| 层：trans/rust.rs | 2026-10-02

## 职责

`trans/rust.rs` 对 Auto `http.*` 客户端面的发射，从分散特判收敛为
**typed、上下文感知**的分派：按源模块/接收者分型/元数/当前 async 上下文
选择共享内核 facade 的正确符号与 await 形态。运行时契约由
[http-client-runtime](../../../a2r-std/design/http-client-runtime.md)
承载；本篇冻结发射面规则。

## async 上下文判定（in_async_ctx）

以下位置发射为 async 上下文：返回 `~T`/`Future` 的 fn、含 `.await` 的
main（`is_main_with_await`）、generator body（async_stream）、`.go`
spawn 块。其余为同步上下文——HTTP 调用走内核同步桥接面。同步声明的
helper 被 async 路径调用属**不支持组合**：运行期由内核同步桥接的 tokio
嵌套检测响亮 panic（不做静默阻塞兜底、不做全语言隐式 async 传染）。

## 分派规则（冻结）

| 源面 | 同步上下文发射 | async 上下文发射 |
|---|---|---|
| `http.get/put/delete(url)` | `a2r_std::http::{get,put,delete}(…)` | `…_async(…).await` |
| `http.post(url, body)`（两参） | `a2r_std::http::post(…)` | `a2r_std::http::post_async(…).await` |
| `http.post(url, body, key)`（三参） | 历史 `async { a2r_std::http::post(…).await; HttpResponse{..} }` 认证面（不变） | 同左（本就 async） |
| `http.post_sync/post_bearer_sync/get_sync` | 同名 facade（同步桥接）+ `set_last_status(__resp.0)` | `*_async(…).await` |
| `http.post_bearer` | —（本就 async 面） | `post_bearer(…).await` |
| `http.request(m,u).header()…` 链 | 链上方法直发，尾 `.send()` | 尾 `.send_async().await` |
| builder **变量重绑定**（`let b2 = b.header(..)`） | `.send()` | `.send_async().await`（链根 builder 分型保持） |
| `http.get_stream/post_stream/post_stream_with_headers` | 同步 facade → `HTTPStream` | `*_async(…).await` → `AsyncHTTPStream`（变量经 `http_stream_async_vars` 登记分型） |
| `s.next()/is_done()/close()`、自由函数 `stream_next/is_done/close(s)` | 同步面（`""` = EOF 哨兵） | `stream_next_async(&s).await`（Option 在 facade 折叠为 `""` 哨兵）；`is_done/close` 直发 |
| `for c in s.iter()` / `for c in stream_iter(s)` / `for c in http.get_stream(u)` | `loop { let c = stream_next(&recv); if c.is_empty() {break;} … }`（生产调用形态用临时拥有绑定 `__hs`） | 同形 + `.await`（`stream_next_async`） |
| `http.last_status()` / `Response.status_code/header_get/body_bytes` | 纯方法直发（kernel typed Response） | 同左 |

- **元数分派防碰撞**：两参普通 post 与三参认证 post 名称相同、语义不同，
  以实参元数在发射期区分（`plan724_probe_post_arity_dispatch_shapes` 钉）。
- **break 语义**：for-in 的 break 不冒充关闭——循环内不做 close，接收者
  仍归用户所有，显式 close 或作用域 Drop（facade RAII）回收。
- **不全局改写**：非流接收者的 `next/close`、非 builder 的 `send` 原样
  走通用方法路径（用户同名方法零打扰）。
- **边界诊断**：`stream_iter(s)` 仅 for-in 位置有语义（脱离 for-in 无
  facade 符号，编译期失败）；`sse_open/sse_poll` 等 VM int id 族不在 a2r
  支持集。

## 编译运行支持面

同源 Auto 样本（`test/a2r/30_plan724/001_http_client_sync`、
`002_http_client_async`）转译产物经临时 cargo 工程（path 依赖 a2r-std）
**实编实跑**（`http_e2e_plan724_{sync,async}_client_matrix`，`cargo th`
收集）；金样冻结发射文本（test-convention：golden 只验证 Rust 文本，
编译运行腿独立成立）。qualified 消费面（api_gen 发射的
`auto_lang::a2r_std::http` 限定名）由
`http_e2e_plan724_qualified_facade_kernel` 真 TCP 腿覆盖；任意
`#[api] ~Stream` handler 的 Rust 生成不在本契约（另案）。

## 关联

- [http-client-runtime](../../../a2r-std/design/http-client-runtime.md)
  （运行时契约：预算/取消/错误分层）
- [test-convention](test-convention.md)（golden 与编译运行腿的分工）
- PLAN-724（reports/724-client-decision.md §1 矩阵冻结 + verification）
