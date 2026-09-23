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
| `http` | `http.at`、`http_stream.at`、`http.vm.at`；没有 `http.rs.at` | `vm/ffi/http_server.rs` 以 Tokio TCP 和手写 HTTP/1 解析实现 `#[api]` 服务；`vm/ffi/stdlib.rs` 注册 HTTP shim | `auto-man/src/api_gen.rs` 从 `back/api.at` 生成 Axum 服务；`a2r-std/src/http.rs` 是 ureq 客户端，不是 VM server 的 Rust 实现 | VM 与生成 Axum 是两套 server；[HTTP Server Spec](http-server.md) 记录当前支持范围 |
| `json` | `json.at`、`json.vm.at`、`json.rs.at`；没有 `json.c.at` | `json.vm.at` 映射 VM JSON native | `.rs.at` 转译实现与 `crates/a2r-std/src/json.rs` 宿主实现 | 只覆盖各自已实现的符号，不推断完整 parity |
| `sse` | `sse.at` 公开 `parse_sse(chunk)`；没有 `sse.vm.at`、`sse.rs.at`、`sse.c.at` | `shim_sse_parse` 调用 `auto-lang/src/sse/parser.rs::parse_sse_chunk`；VM `#[api]` server 独立驱动 generator | `api_gen.rs` 为生成 Axum handler 装配 SSE；a2r HTTP stream 是客户端流读取 | 017-chat 的 publisher SSE 依赖宿主事件总线；VM `auto.bus.subscribe()` 仍是 compile seam，不能宣称 VM pubsub 已与 Axum 对齐 |

VM HTTP/SSE 的请求绑定、所有权、取消和错误行为详见 [HTTP Server Spec](http-server.md)。这些覆盖表是文件与调用路径事实，不是所有目标等价的承诺。

## `api.at` 服务与调用拓扑

| 拓扑 | 路径 | 行为界线 |
|---|---|---|
| VM server | `auto run --server vm` 或 AutoVM 内的 `http.server().listen()` | `#[api]` handler 由 VM HTTP server 调用；当前 VM owner 保持在同线程 `LocalSet`，HTTP 解析不走 Axum。 |
| 生成 Rust server | `auto-man` 消费 `src/back/api.at` 并生成 Axum 服务 | 独立于 VM server；`015-notes`、`017-chat`、`023-realworld` 是已验证示例。 |
| VM merge | 默认前后端 VM 合并 | 前端 handler 直接调用 `#[api]` 函数，得到函数返回值；此调用没有 HTTP status/header。 |
| VM split | `--no-merge` / `AUTO_VM_MERGE=0` | 符合条件的 `#[api]` 调用改写为 HTTP 请求，经选定的 server 执行。 |
| back-proxy | `crates/auto-lang/src/back_proxy.rs` 为 app 装载独立 AutoVM session，再由宿主 HTTP/SSE 路由分发 | 这是独立 session/进程代理路径，不等同于 VM merge、`serve_async` 或生成 Axum。 |

## PLAN-696 锁定的 VM HTTP 行为

- 请求头按字节读至结束标记，普通 body 按 `Content-Length` 读满；非法长度和短体返回 400，读超时返回 408，超过 10 MiB 返回 413。multipart 复用同一完整 body 与上限检查。
- `AutoVM` 是 `!Send`；VM 和 HTTP server 由同一线程的 `LocalSet` 持有，服务与连接任务通过 `Rc<AutoVM>` 保持生命周期。禁止把 VM 引用编码成整数跨线程转运。
- SSE generator 以有界指令批次执行并让出 LocalSet；SSE 专用 sleep 用唤醒期限等待，断连后取消 producer 并回收 iterator/task。当前 HTTP listener 没有优雅关闭 API。
- VM 与生成 Axum 的 CRUD、SSE 事件和认证样本有独立 live 验证；这不覆盖 VM `auto.bus.subscribe()` publisher parity。`https`、TLS、HTTP/2/3、WebSocket 与统一 Axum server 仍不属于当前支持面。
