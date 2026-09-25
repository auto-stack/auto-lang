# 网络标准库与服务集成（HTTP / SSE / 路由）

## 范围

docs/design/13 规划的 async/net/http/json/url/log/env 模块在 `stdlib/auto/` 的落地，
以及编译器侧的 `sse/`（SSE 解析）与 `route/`（混合路由）两个支撑目录。

## 原则

- **目标层次**：逻辑上由 `async` 支撑 `net`，HTTP API 再组合网络、JSON 与 URL 能力；这是模块设计关系，不代表各目标已有同构实现或共享传输层。
- **按目标核实现状**：`.at` 公共声明、`.vm.at`/`.rs.at` 等目标文件、native shim、生成 Rust 服务与宿主适配是不同覆盖维度。具体文件和缺口见 [stdlib 后台装配表](../../../stdlib/design/backend-assembly.md)。
- 路由约定优先、配置兜底覆盖。

## 细节

### 模块逻辑层次（设计视图；不表示共享实现已落地）

```text
http.Server（route/middleware/listen）
  ├─ http.Request / http.Response（headers/body/cookie/params）
  ├─ json / form / url（encode/decode）
  └─ net.TcpListener / TcpStream（bind/accept/read/write）
       └─ async（spawn/channel，tokio 承载）
log / env 为独立工具模块
```

此图表示公共能力的逻辑关系，不表示 VM HTTP server 与 Rust/Axum server 共用实现。传输层现状（PLAN-699）：VM `#[api]` server 的 HTTP/1.1 由 Axum/Hyper 承载——专属 `auto-http-net` 线程驱动 hyper-util 连接，经 owned `Send` 桥（有界队列 + oneshot）把请求交给 VM owner 线程的 `dispatch_api_request`；生成 Rust 服务（auto-man）仍是独立装配的 Axum handler，两者不共享路由/handler 代码。Auto `task`/`~T` 的通用异步 I/O 重做（Design 33 阶段 C）未落地：VM handler 仍在 owner 线程同步执行，无抢占；HTTP 层的队列/body/超时预算（详见 stdlib [http-server §8.1](../../../stdlib/design/http-server.md)）约束的是排队与传输资源，不是 handler 可中断性。
实现目录的文件实证包括：`http.at`+`http.vm.at`、`http_stream.at`、`net.at`+`net.vm.at`、
`async.at`+`async.vm.at`、`json.at`+`json.vm.at`+`json.rs.at`、`url.at`+`url.vm.at`、
`log.at`+`log.vm.at`、`env.at`+`env.vm.at`+`env.rs.at`、`sse.at`、`sse_server.at`。
`stdlib/auto/` 有多份 `.vm.at`/`.rs.at` 目标文件；它们不构成每个公共 API 都有配对实现的承诺。

### 双模式执行（13 章 §Dual-Mode Execution）

| 文件 | 用途 | 消费方 |
|------|------|--------|
| `http.at` | Auto 公共 API 声明 | VM；Rust server 从 `back/api.at` 独立生成 |
| `http.vm.at` | `#[vm]` FFI 绑定 | AutoVM |

`stdlib/auto/http.rs.at` 当前不存在。`auto-man` 从 `back/api.at` 生成 Axum 服务；
`crates/a2r-std/src/http.rs` 是 HTTP 客户端实现，不是上述 VM server 的 Rust 后端。

### SSE（plan-152/154/313）

- 编译器侧 `sse/` 提供增量解析：`SSEParser<R: BufRead>` 逐事件产出 `SSEEvent`
  （id/event/data/retry），`parse_sse_chunk` 处理字符串块；空事件与完成标记有显式判定。
- 用户侧 stdlib：`sse.at`（客户端）、`sse_server.at`（服务端推送，plan-313 完成
  TCP flush + SSE 服务端 Phase 1-2）。
- `sse.at` 中的解析 API、`sse_server.at` 的推送示例、`#[api]` handler 的 VM SSE
  与生成 Axum 的 SSE 是不同路径；一个路径存在不代表其他路径具备相同功能。

### 混合路由（plan-114）

- 解析顺序：扫 `routes/` 目录 → `routes {}` 配置块 → 合并（配置覆盖约定）→
  生成平台导航代码。
- 命名约定：`index.at`→`/`、`about.at`→`/about`、`user/[id].at`→`/user/:id`、
  嵌套目录映射嵌套路径；`RouteDef.source`（Convention/Config）记录来源，
  `meta` 携带 layout/auth 等配置元数据。

### 后续演进（在途）

plan-344（统一 HTTP 通讯架构：同步/异步 × 流式/非流式 × VM/a2r）、plan-349（HTTP 扩展
roadmap）、plan-350（WebSocket）、plan-352（中间件/session/SSR/OpenAPI）均为设计态，
未实现（各 plan 文件自述状态）。

## 显式非目标

- WebSocket、TLS/HTTPS、HTTP/2/3：13 章 Open Questions，至今未实现（plan-350 仍设计态）。
- CORS 处理策略未定（13 章 Open Question）。
- `sse/` 只做解析，不做网络 IO；连接管理在 stdlib/VM 层。
- json 流式（JSON Lines）大数据集支持未做（13 章 Open Question）。

> 来源: docs/design/13-networking.md；crates/auto-lang/src/sse/、route/；stdlib/auto/；
> docs/plans/archive/102、114、152-streaming-http-sse、154、195；docs/plans/archive/312、313、344、353；docs/plans/328、329、349、350、352
