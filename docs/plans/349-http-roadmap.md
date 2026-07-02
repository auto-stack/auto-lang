# HTTP 库扩展 Roadmap

> **状态**：规划文档
> **基于**：v0.4.1 HTTP 能力盘点
> **关联计划**：Plan 344（统一通讯架构）、Plan 350-353（具体扩展）

## 当前能力矩阵（v0.4.1）

### Client

| 能力 | 状态 | API |
|------|------|-----|
| GET/POST/PUT/DELETE | ✅ | `auto.http.get/post/put/delete` |
| JSON body | ✅ | `auto.http.*_json` (Plan 340) |
| 自定义 Headers | ✅ | `RequestBuilder.header` |
| Bearer Token Auth | ✅ | `auto.http.post_bearer` |
| API Key Auth | ✅ | `auto.http.post_sync` |
| HTTP Streaming（分块） | ✅ | `http_stream.get_stream/stream_next` |
| SSE 异步流 | ✅ | `http.sse_get_stream` (Plan 348) |
| Request Builder | ✅ | `request/header/body/timeout/json/send` |
| Response 读取 | ✅ | `Response.body/status_code/header_get` |
| TCP 原始套接字 | ✅ | `net.tcp_*` |
| HTTPS（基础） | ✅ | reqwest 默认支持 `https://` URL |
| HTTPS（自定义 CA/客户端证书） | ❌ | — |
| 文件上传（multipart） | ❌ | — |
| 大文件下载 + 断点续传 | ❌ | — |
| 上传/下载进度回调 | ❌ | — |
| WebSocket | ❌ | — |
| Cookie 管理 | ❌ | — |
| 请求重试 | ❌ | — |
| HTTP/2 配置 | ❌ | — |
| gzip/brotli 压缩 | ❌ | — |

### Server

| 能力 | 状态 | API |
|------|------|-----|
| GET/POST/PUT/DELETE 路由 | ✅ | `server_get/post/put/delete` |
| 路径参数（:id） | ✅ | `match_route` |
| JSON 响应 | ✅ | `response` |
| 自定义状态码 | ✅ | `response_status` |
| 自定义响应头 | ✅ | `response_header` |
| HTML/Bytes 响应 | ✅ | `response_html/response_bytes` |
| 静态文件 | ✅ | `server_static` |
| SSE 流式响应 | ✅ | generator yield + `serve_async` |
| 异步多连接 | ✅ | tokio + `spawn_local` + `yield_now` |
| CORS | ⚠️ | Axum 后端有，AutoVM server 无 |
| WebSocket | ❌ | — |
| 中间件链 | ❌ | — |

### 普通 HTTP 请求异步化

| 状态 | 说明 |
|------|------|
| ❌ 未实现 | Plan 344 路径 B（AWAIT_FUTURE 真挂起）未实现。普通 GET/POST 仍用 `std::thread::spawn + join` 阻塞 |

## 扩展计划索引

| 计划 | 方向 | 难度 | 优先级 | 状态 |
|------|------|------|--------|------|
| **Plan 350** | HTTPS 证书配置 | 低 | 🔴 高 | 设计文档 |
| **Plan 351** | 文件上传 multipart | 中 | 🟡 中 | 设计文档 |
| **Plan 352** | 大文件下载 + 断点续传 + 进度 | 中 | 🟡 中 | 设计文档 |
| **Plan 353** | WebSocket 双向通讯 | 高 | 🟡 中 | 设计文档 |

## 实施路线

### 阶段 1：安全与信任（Plan 350）
HTTPS 证书配置是生产环境的基础需求。自签名证书、企业内网 CA、客户端证书认证等场景都需要。
- 自定义 CA 证书
- 跳过证书验证（开发环境）
- 客户端证书（mTLS）

### 阶段 2：文件传输（Plan 351 + 352）
文件上传和下载是 Web 应用的核心场景。
- multipart/form-data 上传（文件 + 表单字段）
- 大文件分块下载 + Range header 断点续传
- 上传/下载进度回调（用于 UI 进度条）

### 阶段 3：实时通讯（Plan 353）
WebSocket 补全了 HTTP 请求-响应模型之外的实时双向通讯能力。
- `ws.connect(url)` 建立连接
- `ws.send(text/binary)` 发送消息
- `for msg in ws.on_message()` 消费消息（复用 Plan 348 的非阻塞 yield 机制）
- 自动重连、心跳

### 阶段 4：异步化补全（Plan 344 路径 B）
让普通 GET/POST 也支持真异步（AWAIT_FUTURE 外部 future 挂起），消除 UI 冻结。

### 阶段 5：易用性增强
- Cookie 管理（`http.cookie_store(true)`）
- 请求重试（`request.retry(count, delay)`）
- 压缩支持（自动 gzip/brotli）
- 连接池配置

## 设计原则

1. **平台无关**：每个 native 在 VM 和 a2r 两个平台都要实现
2. **正交特性**：同步/异步 × 流式/非流式 × 安全/非安全 是正交的
3. **渐进式**：新 native 不破坏现有 API
4. **对称设计**：Client 和 Server 能力尽量对称（如 WebSocket client + server）
5. **复用现有基础设施**：Plan 348 的非阻塞 yield、Plan 344 的统一架构
