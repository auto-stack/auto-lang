# PLAN-696 T-01: 标准库后台装配盘点

- 源码快照：`051e7b54023f420c3955c480d986e83197eb5899`（2026-09-23，Plan 696 基线）
- 范围：`io / net / async / http / json / sse` 的公共 `.at`、目标实现、VM FFI、Rust 后台、AutoUI 消费端与示例。
- 规则：路径标为“缺失”表示本快照未发现相应源文件；相似命名的宿主模块不等价于 `.at` 目标实现。

## 装配路径

`crates/auto-lang/src/compile.rs` 的编译上下文装载公共 `.at` 并按 VM 目标加载 `.vm.at`；`stdlib/auto/` 当前同时包含 `.vm.at`、`.rs.at` 与 `.c.at` 的分目标实现。`crates/auto-lang/src/autovm_persistent.rs` 有独立 `.vm.at` 装载路径。`crates/auto-lang/src/parser.rs::get_file_extensions` 列出 Interp/TransC/TransRust 对应扩展名，源码注释标明该 helper 未被调用；实际编译装载逻辑应以 `compile.rs` 为准。

## 覆盖表

| 能力 | 公共 `.at` / 目标源文件 | VM / native 路径 | Rust 后台 / 其他目标 | 消费端与示例 | 当前事实与缺口 |
|---|---|---|---|---|---|
| `io` | `stdlib/auto/io.at`；`io.vm.at`、`io.c.at` | `io.vm.at` 定义 VM 目标实现；`vm/ffi/stdlib.rs` 注册标准库 shim | `io.c.at` 是 C 目标实现；未发现 `io.rs.at` | 标准库调用方；本计划抽样的三个 `api.at` 不直接覆盖 | 存在 VM 与 C 目标源；不能据此推断 Rust 实现存在 |
| `net` | `stdlib/auto/net.at`；`net.vm.at` | `vm/ffi/stdlib.rs::{shim_net_tcp_bind,shim_net_tcp_listener_accept,...}` 注册 `auto.net.*` native | 未发现 `net.rs.at`；`crates/a2r-std/src/` 未见专属 `net.rs` 模块 | VM TCP shim 供 `.at` 使用 | `.at` API 有 VM/native 路径；没有与之对称的 `a2r-std` 网络模块 |
| `async` / task | `stdlib/auto/async.at`；`async.vm.at` | `async.vm.at` 声明 spawn、yield、sleep、channel、recv 等 VM 调用 | `crates/a2r-std/src/task.rs` 提供 Tokio actor mailbox 与 drain；不是 VM shim 的同一实现 | VM task/channel 调用方；`017-chat` 的 stream producer 可形成异步事件 | API 名称和实现模型不同；没有 `async.rs.at` |
| `http` | `stdlib/auto/http.at`、`http_stream.at`；`http.vm.at` | `vm/ffi/http_server.rs` 实现 `serve_async`、`handle_connection_async`、请求绑定、响应与 SSE；`vm/ffi/stdlib.rs` 注册 `auto.http.server_*` shim | 未发现 `http.rs.at`。`crates/a2r-std/src/http.rs` 是基于 ureq 的 HTTP 客户端（含同步/异步发送与流式读取），没有共享 VM 服务端。`crates/auto-man/src/api_gen.rs` 从 `back/api.at` 生成 Axum Rust 服务；`--server=vm` 会跳过 Rust 服务生成 | `015-notes` CRUD、`023-realworld` 认证/文章 API；`api.at` 也可经进程代理调用 | VM 的 `#[api]` 服务是手写 TCP 解析；生成 Rust 服务是 Axum；a2r HTTP client 是独立客户端实现。未发现计划背景中所称 `http.rs.at` |
| `json` | `stdlib/auto/json.at`；`json.vm.at`、`json.rs.at` | `json.vm.at` 映射 VM JSON native | `json.rs.at` 提供转译实现；`crates/a2r-std/src/json.rs` 是宿主 Rust 实现 | `http.at` 请求/响应 JSON 与 API 参数绑定 | 有 VM/Rust 目标文件；未发现 `json.c.at`，覆盖范围不可由文件名推为完整 parity |
| `sse` | `stdlib/auto/sse.at` 仅公开 `#[vm] parse_sse(chunk)`；未发现 `sse.vm.at`、`sse.rs.at` 或 `sse.c.at`。流式客户端 API 在 `http.at` / `http_stream.at`，producer 类型来自 `~Stream<T>` | `vm/ffi/stdlib.rs::shim_sse_parse` 调用 `auto-lang/src/sse/parser.rs::parse_sse_chunk`；服务端 `http_server.rs` 对 `#[api]` 返回的迭代器取帧；`musk_response_ctor.rs::sse_frame_from_nv` 序列化 Event/标量帧 | `auto-man/api_gen.rs` 从流返回类型生成 Axum `Sse`；`crates/a2r-std/src/http.rs` 的 HTTPStream 是客户端流读取 | `017-chat/src/back/api.at::stream() ~Stream<ChatEvent>`；`ui_gen/api.rs` 从 `back/api.at` 发现流端点，`ui_gen/ts_adapter.rs` / `ui_gen/vue.rs` 发出前端 EventSource 接线；`back_proxy.rs` 另有 SSE/HTTP 代理路径 | 存在一个 VM-only 的解析器 API；服务端 SSE 是多个独立能力拼接的路径，VM 与 Axum 两条服务实现独立 |

## `api.at` 消费拓扑抽样

- `015-notes/src/back/api.at`：GET/POST/PUT/DELETE/PATCH 的 CRUD handler；VM `#[api]` 路由与 `auto-man` Axum 代码生成均能以该文件为输入。
- `017-chat/src/back/api.at`：CRUD/command handler 加 `stream() ~Stream<ChatEvent>`；前端生成器消费流签名，VM/Axum 各自负责服务端 SSE。
- `023-realworld/src/back/api.at`：登录、注册、Bearer 身份及文章/评论等 HTTP handler；适合检验按名绑定、认证 metadata 与 JSON 返回。
- `crates/auto-lang/src/back_proxy.rs` 提供独立进程中的 HTTP/SSE 代理/调用路径；它不等于 VM `serve_async` 或生成 Axum 服务。具体 IPC/合并模式需按调用方和运行配置分别验收。

## 可追溯源码入口

- 装载：`crates/auto-lang/src/compile.rs`、`autovm_persistent.rs`、`parser.rs::get_file_extensions`
- VM 网络/HTTP：`crates/auto-lang/src/vm/ffi/stdlib.rs::{shim_net_tcp_bind,shim_http_server_listen,run_http_server_blocking}`、`crates/auto-lang/src/vm/ffi/http_server.rs::{serve_async,handle_connection_async}`
- VM SSE：`crates/auto-lang/src/vm/native.rs::shim_iterator_next`、`crates/auto-lang/src/vm/ffi/musk_response_ctor.rs::sse_frame_from_nv`
- Rust 服务生成：`crates/auto-man/src/api_gen.rs::generate_api`
- Rust HTTP client/task：`crates/a2r-std/src/http.rs`、`crates/a2r-std/src/task.rs`
- AutoUI 流接线：`crates/auto-lang/src/ui_gen/api.rs`、`crates/auto-lang/src/back_proxy.rs`

本表描述该快照的源码事实，不声明目标间功能等价。VM HTTP P0 与 Spec 修订由 T-02..T-08 继续落实。
