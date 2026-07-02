# HTTP 库扩展 Roadmap

> **状态**：规划文档（持续更新）
> **关联计划**：Plan 344（统一通讯架构）、Plan 350（WebSocket）

## 当前能力矩阵（v0.4.1 + Plan 350）

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
| HTTPS 自定义 CA 证书 | ✅ | `RequestBuilder.tls_ca_cert` (Plan 350) |
| HTTPS 跳过证书验证 | ✅ | `RequestBuilder.tls_skip_verify` (Plan 350) |
| HTTPS 客户端证书 (mTLS) | ⚠️ | `RequestBuilder.tls_client_cert` (API 已注册，PKCS12 实现待 feature) |
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

## 已实现扩展

### Plan 350: HTTPS 证书配置（已合并到此 roadmap）

**新增 3 个 TLS native**（`RequestBuilder` 链式 API）：
- `RequestBuilder.tls_ca_cert(path)` — 加载 PEM 格式自定义 CA 证书
- `RequestBuilder.tls_skip_verify(bool)` — 跳过证书验证
- `RequestBuilder.tls_client_cert(cert, key)` — 客户端证书（mTLS，API 已注册，PKCS12 实现待 reqwest `rustls-tls` feature）

同时重写了 `shim_request_builder_send`：完整使用 RequestBuilder 的所有配置项（headers + body + timeout + TLS），通过 `reqwest::blocking::ClientBuilder` 构建 TLS-aware client。

## 待实现扩展

### 1. 文件上传（multipart/form-data）

reqwest 已有 `.multipart()` 支持，只需暴露为 native。

**API 设计**：
- `http.upload(url, file_path) -> Response` — 单文件上传
- `http.upload_with_fields(url, file_path, fields_json) -> Response` — 文件 + 表单字段
- `RequestBuilder.multipart_file(field_name, file_path)` — 链式 API
- `RequestBuilder.multipart_text(field_name, value)` — 链式 API

**实现**：HttpRequestBuilderData 增加 `multipart_files` / `multipart_texts` 字段。send 时如有 multipart 数据，构造 `reqwest::blocking::multipart::Form`。

### 2. 大文件下载 + 断点续传 + 进度回调

**API 设计**：
- `http.download(url, file_path) -> bool` — 下载到文件
- `http.download_resume(url, file_path, offset) -> bool` — 断点续传（Range header）
- `http.download_with_progress(url, file_path) -> iterator` — 带进度的下载

**实现**：
- 断点续传用 HTTP `Range: bytes={offset}-` header
- 进度迭代器复用 Plan 348 的非阻塞 yield 机制（独立线程 + channel + AsyncHttpStream）
- 文件用 `std::fs::File` 逐 chunk 写入

### 3. WebSocket 双向通讯（独立计划 Plan 350）

见 `353-websocket.md`。复杂度高，需引入 tungstenite 依赖 + 新模块。

### 4. 普通 HTTP 请求异步化（Plan 344 路径 B）

让普通 GET/POST 也支持真异步（AWAIT_FUTURE 外部 future 挂起），消除 UI 冻结。

### 5. 易用性增强

- Cookie 管理（`http.cookie_store(true)`）
- 请求重试（`request.retry(count, delay)`）
- 压缩支持（自动 gzip/brotli）
- CORS（AutoVM server 端）

## 设计原则

1. **平台无关**：每个 native 在 VM 和 a2r 两个平台都要实现
2. **正交特性**：同步/异步 × 流式/非流式 × 安全/非安全 是正交的
3. **渐进式**：新 native 不破坏现有 API
4. **对称设计**：Client 和 Server 能力尽量对称
5. **复用现有基础设施**：Plan 348 的非阻塞 yield、Plan 344 的统一架构
