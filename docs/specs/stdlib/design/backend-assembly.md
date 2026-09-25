# 标准库后台装配与覆盖

> **Status**: current-state inventory（PLAN-696）
> **范围**: `stdlib/auto/` 的公共 `.at` API、目标文件、VM native、a2r Rust runtime、生成服务与 AutoUI 消费路径。
> **关联**: [stdlib 项目卡](../project.md)、[HTTP Server Spec](http-server.md)、[网络标准库 Spec](../../auto-lang/runtime/design/networking-stdlib.md)

## 装配规则

- `stdlib/auto/` 中的公共 `.at` 声明与 `.vm.at`、`.rs.at`、`.c.at` 目标文件是不同层。目标文件缺失必须按缺口记录；相同 API 名称不自动代表行为一致。
- 编译装载以 `crates/auto-lang/src/compile.rs` 的实际路径为准：公共 `.at` 先进入编译上下文，再按目标装载相应变体；`autovm_persistent.rs` 有独立的 VM 目标装载路径。`parser.rs::get_file_extensions` 是未使用的 helper，不作为装载事实来源。
- AutoVM native shim、`a2r-std` 宿主库、由 `auto-man` 从 `back/api.at` 生成的 Axum 服务、Vue/AutoUI 适配器是不同后台路径；不存在一个跨所有路径自动共享的实现层。

## 能力覆盖

| 能力 | 公共 API 与目标文件 | VM / native 路径 | Rust / 其他后端 | 消费端与当前边界 |
|---|---|---|---|---|
| `io` | `io.at`、`io.vm.at`、`io.c.at`；没有 `io.rs.at` | `io.vm.at` 与 `vm/ffi/stdlib.rs` 中的标准库 shim | `io.c.at` 是 C 目标实现；没有对称的 a2r `io` 模块 | 不能从 C/VM 文件推断 Rust 后端存在 |
| `net` | `net.at`、`net.vm.at`；没有 `net.rs.at` | `vm/ffi/stdlib.rs` 注册 `auto.net.*` TCP native | `crates/a2r-std/src/` 没有对称的 `net` 模块 | 供 VM `.at` 调用；没有通用跨目标网络层 |
| `async` / task | `async.at`、`async.vm.at` | VM 目标声明 spawn、yield、sleep、channel、recv 等调用 | `crates/a2r-std/src/task.rs` 是 Tokio actor mailbox 与 drain | 名称或用途相近不表示调度和生命周期相同 |
| `http` | `http.at`、`http_stream.at`、`http.vm.at`；没有 `http.rs.at` | `vm/ffi/http_transport.rs` 以 Axum/Hyper 承载 `#[api]` 服务的 HTTP/1.1 传输（PLAN-699），`vm/ffi/http_server.rs::dispatch_api_request` 在 VM owner 线程做路由/绑定/中间件/handler（owned 桥）；`vm/ffi/stdlib.rs` 注册 HTTP shim | `auto-man/src/api_gen.rs` 从 `back/api.at` 生成 Axum 服务；`a2r-std/src/http.rs` 是 ureq 客户端，不是 VM server 的 Rust 实现 | VM 与生成 Axum 都依赖 Axum 但为**两套独立服务实现**（各自的路由/handler 装配，不共享代码）；[HTTP Server Spec](http-server.md) 记录当前支持范围 |
| `json` | `json.at`、`json.vm.at`、`json.rs.at`；没有 `json.c.at` | `json.vm.at` 映射 VM JSON native | `.rs.at` 转译实现与 `crates/a2r-std/src/json.rs` 宿主实现 | 只覆盖各自已实现的符号，不推断完整 parity |
| `sse` | `sse.at` 公开 `parse_sse(chunk)`；没有 `sse.vm.at`、`sse.rs.at`、`sse.c.at` | `shim_sse_parse` 调用 `auto-lang/src/sse/parser.rs::parse_sse_chunk`；VM `#[api]` server 经 `dispatch_api_request` 独立驱动 generator | `api_gen.rs` 为生成 Axum handler 装配 SSE；a2r HTTP stream 是客户端流读取 | 017-chat 的 publisher SSE 依赖宿主事件总线；VM `auto.bus.subscribe()` 自 PLAN-698 起为真实执行面（进程内广播总线+转发线程，见 [http-server §8.1](http-server.md)），但 VM 总线与生成 Axum 的 publisher 事件互不相通，不构成跨后台对齐 |

VM HTTP/SSE 的请求绑定、所有权、取消和错误行为详见 [HTTP Server Spec](http-server.md)。这些覆盖表是文件与调用路径事实，不是所有目标等价的承诺。

## `api.at` 服务与调用拓扑

| 拓扑 | 路径 | 行为界线 |
|---|---|---|
| VM server | `auto run --server vm` 或 AutoVM 内的 `http.server().listen()` | `#[api]` handler 由 VM HTTP server 调用；HTTP/1.1 传输为 Axum/Hyper（专属网络线程 + owned 桥，PLAN-699），VM owner 保持单线程 `LocalSet`。 |
| 生成 Rust server | `auto-man` 消费 `src/back/api.at` 并生成 Axum 服务 | 独立于 VM server；`015-notes`、`017-chat`、`023-realworld` 是已验证示例。 |
| VM merge | 默认前后端 VM 合并 | 前端 handler 直接调用 `#[api]` 函数，得到函数返回值；此调用没有 HTTP status/header。 |
| VM split | `--no-merge` / `AUTO_VM_MERGE=0` | 符合条件的 `#[api]` 调用改写为 HTTP 请求，经选定的 server 执行。 |
| back-proxy | `crates/auto-lang/src/back_proxy.rs` 为 app 装载独立 AutoVM session，再由宿主 HTTP/SSE 路由分发 | 这是独立 session/进程代理路径，不等同于 VM merge、`serve_async` 或生成 Axum。 |

## PLAN-696/699 锁定的 VM HTTP 行为

- HTTP/1.1 分帧、keep-alive 连接复用与 chunked 请求体解码由 Axum/Hyper 承担（PLAN-699 专属网络线程 + hyper-util auto Builder `http1_only`）；请求头缓冲 64 KiB + ≤100 头（超限 431）、慢头 10s 读超时（关闭）、body 10 MiB（413）+ 收满总期限 10s（408）。multipart 复用同一完整 body 与上限检查。
- `AutoVM` 是 `!Send`；VM owner 循环独占 VM（单线程 `LocalSet`），网络层与 owner 之间只传 owned `Send` 的 `ApiRequest`/`ApiReply`（有界队列 + oneshot）。禁止把 VM 引用编码成整数跨线程转运、禁止 `spawn_blocking` 调 VM。
- SSE generator 以有界指令批次执行并让出 LocalSet；帧通道容量 1 背压，断连/关闭经 `FrameStream` 收流并在 owner 线程回收 iterator/task/订阅。服务有优雅关闭（§7.3 引 [http-server](http-server.md)）。
- VM 与生成 Axum 的 CRUD、SSE 事件和认证样本有独立 live 验证；VM `auto.bus.subscribe()` publisher 面（PLAN-698）在 017-chat 有全环实测，但 VM 总线与生成侧事件互不相通。`https`、TLS、HTTP/2/3、通用 WebSocket（现有为简化 echo）与统一 Axum server 仍不属于当前支持面。
