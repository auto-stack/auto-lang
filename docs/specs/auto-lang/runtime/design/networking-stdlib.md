# 网络标准库与服务集成（HTTP / SSE / 路由）

## 范围

docs/design/13 规划的 async/net/http/json/url/log/env 模块在 `stdlib/auto/` 的落地，
以及编译器侧的 `sse/`（SSE 解析）与 `route/`（混合路由）两个支撑目录。

## 原则

- 层次依赖：`async` 最底，`net` 建于其上，`http` 依赖 net/json/url，`log`/`env` 独立。
- 公共 API 与执行后端解耦：同一 `.at` API，AutoVM 走 FFI（`.vm.at`），a2r 走直译（`.rs.at`）。
- 路由约定优先、配置兜底覆盖。

## 细节

### 模块层次（13 章设计，已落地）

```text
http.Server（route/middleware/listen）
  ├─ http.Request / http.Response（headers/body/cookie/params）
  ├─ json / form / url（encode/decode）
  └─ net.TcpListener / TcpStream（bind/accept/read/write）
       └─ async（spawn/channel，tokio 承载）
log / env 为独立工具模块
```

实证（`stdlib/auto/`）：`http.at`+`http.vm.at`、`http_stream.at`、`net.at`+`net.vm.at`、
`async.at`+`async.vm.at`、`json.at`+`json.vm.at`+`json.rs.at`、`url.at`+`url.vm.at`、
`log.at`+`log.vm.at`、`env.at`+`env.vm.at`+`env.rs.at`、`sse.at`、`sse_server.at`。
全目录共 28 个 `.vm.at`/`.rs.at` 后端文件——13 章 "Planned: 双文件模式" 已成现实。

### 双模式执行（13 章 §Dual-Mode Execution）

| 文件 | 用途 | 消费方 |
|------|------|--------|
| `http.at` | Auto 公共 API | 两种模式 |
| `http.vm.at` | `#[vm]` FFI 绑定 | AutoVM |
| `http.rs.at` | `#[rust_fn]` 转译提示 | a2r |

### SSE（plan-152/154/313）

- 编译器侧 `sse/` 提供增量解析：`SSEParser<R: BufRead>` 逐事件产出 `SSEEvent`
  （id/event/data/retry），`parse_sse_chunk` 处理字符串块；空事件与完成标记有显式判定。
- 用户侧 stdlib：`sse.at`（客户端）、`sse_server.at`（服务端推送，plan-313 完成
  TCP flush + SSE 服务端 Phase 1-2）。

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
> docs/plans/old/102、114、152-streaming-http-sse、154、195；docs/plans/archive/312、313、344、353；docs/plans/328、329、349、350、352
