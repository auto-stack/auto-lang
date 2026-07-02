# HTTP 库扩展 Roadmap

> **状态**：规划文档（持续更新）
> **关联计划**：Plan 344（统一通讯架构）、Plan 350（WebSocket）

## 当前能力矩阵（v0.4.1 + Plan 349/350）

### Client

| 能力 | VM | a2r | API |
|------|:--:|:---:|-----|
| GET/POST/PUT/DELETE | ✅ | ✅ | `auto.http.get/post/put/delete` |
| JSON body | ✅ | ✅ | `auto.http.*_json` (Plan 340) |
| 自定义 Headers | ✅ | ✅ | `RequestBuilder.header` |
| Bearer Token Auth | ✅ | ❌ | `auto.http.post_bearer` |
| API Key Auth | ✅ | ❌ | `auto.http.post_sync` |
| HTTP Streaming（分块） | ✅ | ❌ | `http_stream.get_stream/stream_next` |
| SSE 异步流 | ✅ | ❌ | `http.sse_get_stream` (Plan 348) |
| Request Builder | ✅ | ✅ | `request/header/body/timeout/json/send` |
| Response 读取 | ✅ | ✅ | `Response.body/status_code/header_get` |
| TCP 原始套接字 | ✅ | ❌ | `net.tcp_*` |
| HTTPS（基础） | ✅ | ✅ | reqwest/ureq 默认支持 |
| HTTPS 自定义 CA 证书 | ✅ | ❌ | `RequestBuilder.tls_ca_cert` (Plan 349) |
| HTTPS 跳过证书验证 | ✅ | ❌ | `RequestBuilder.tls_skip_verify` (Plan 349) |
| HTTPS 客户端证书 (mTLS) | ⚠️ | ❌ | `RequestBuilder.tls_client_cert` (API 已注册，PKCS12 待 feature) |
| 文件上传（multipart） | ✅ | ❌ | `http.upload` / `RequestBuilder.multipart_file/text` (Plan 349) |
| 文件下载 | ✅ | ❌ | `http.download` (Plan 349) |
| 断点续传 | ✅ | ❌ | `http.download_resume` (Plan 349) |
| 下载进度回调 | ✅ | ❌ | `http.download_with_progress` (Plan 349) |
| WebSocket 客户端 | ✅ | ❌ | `ws.connect/send/on_message/close` (Plan 350) |

### Server

| 能力 | VM | a2r | API |
|------|:--:|:---:|-----|
| GET/POST/PUT/DELETE 路由 | ✅ | ✅ | `server_get/post/put/delete` |
| 路径参数（:id） | ✅ | ✅ | `match_route` |
| JSON 响应 | ✅ | ✅ | `response` |
| 自定义状态码 | ✅ | ✅ | `response_status` |
| 自定义响应头 | ✅ | ✅ | `response_header` |
| HTML/Bytes 响应 | ✅ | ✅ | `response_html/response_bytes` |
| 静态文件 | ✅ | ✅ | `server_static` |
| SSE 流式响应 | ✅ | ✅ | generator yield + `serve_async` |
| 异步多连接 | ✅ | ✅ | tokio + `spawn_local` + `yield_now` |
| WebSocket echo | ✅ | ❌ | `serve_async` 内嵌帧解析 (Plan 350) |
| CORS | ⚠️ | ✅ | Axum 后端有，AutoVM server 无 |

## 已完成扩展

### Plan 349: HTTP Client 扩展（VM 侧已完成）

**HTTPS 证书配置**（3 个 native）：
- `RequestBuilder.tls_ca_cert(path)` — PEM 格式自定义 CA 证书
- `RequestBuilder.tls_skip_verify(bool)` — 跳过证书验证
- `RequestBuilder.tls_client_cert(cert, key)` — mTLS 客户端证书（PKCS12，API 已注册）
- 重写 `shim_request_builder_send`：完整使用 headers + body + timeout + TLS

**文件上传 multipart**（3 个 native）：
- `http.upload(url, file_path)` — 单文件上传
- `RequestBuilder.multipart_file(field_name, file_path)` — 链式 API
- `RequestBuilder.multipart_text(field_name, value)` — 链式 API
- `shim_request_builder_send` 增加 multipart 优先于 JSON body

**文件下载 + 断点续传 + 进度**（3 个 native）：
- `http.download(url, file_path)` — 阻塞式下载到文件
- `http.download_resume(url, file_path, offset)` — Range header 断点续传
- `http.download_with_progress(url, file_path)` — 非阻塞进度迭代器
  （独立线程 + 异步 reqwest + bytes_stream + channel，复用 Plan 348 非阻塞 yield）

### Plan 350: WebSocket（VM 侧已完成）

**WebSocket 客户端**（4 个 native）：
- `ws.connect(url)` — 独立线程 tungstenite sync client，mpsc channel 推消息
- `ws.send(handle, message)` — 通过 mpsc::Sender 发送
- `ws.on_message(handle)` — AsyncHttpStream 迭代器（复用 Plan 348 非阻塞 yield）
- `ws.close(handle)` — 关闭连接

**WebSocket 服务端 echo**：
- `serve_async` 检测 `Upgrade: websocket` header
- 手动握手（SHA1 + Base64 计算 Sec-WebSocket-Accept）
- 手动帧解析（opcode、mask、扩展长度），不依赖 tungstenite socket 包装
- Echo 模式：文本帧原样返回，支持 Ping/Pong/Close
- 协作式 yield_now（多连接并发）

## 待实现步骤

### 步骤 1：a2r HTTPS TLS 配置适配

**目标**：让 Rust 前端生成的 API client 支持 TLS 配置。

**改动**：
- `rust_ui.rs` `generate_endpoint_fn`：GET/POST 函数体从 `ureq` 改为 `reqwest::blocking`，支持 `ClientBuilder.danger_accept_invalid_certs` / `add_root_certificate`
- 新增 `generate_tls_config_snippet`：从环境变量（`AUTO_TLS_CA_CERT` / `AUTO_TLS_SKIP_VERIFY`）读取 TLS 配置，生成 client builder 代码
- 或在 `wrap_example` 里生成全局 TLS-aware client

**文件**：`crates/auto-man/src/rust_ui.rs`

### 步骤 2：a2r 文件上传 multipart 适配

**目标**：让 Rust 前端能生成 multipart 上传函数。

**改动**：
- `rust_ui.rs` 新增 `generate_upload_fn`：生成 `reqwest::blocking::multipart::Form` 代码
- 从 api.at 解析 endpoint 判断是否有文件参数
- 或生成通用 `fn upload(url, file_path) -> Value` 函数

**文件**：`crates/auto-man/src/rust_ui.rs`

### 步骤 3：a2r 文件下载适配

**目标**：让 Rust 前端能生成下载函数。

**改动**：
- `rust_ui.rs` 新增 `generate_download_fn`：生成 `reqwest::blocking` 流式下载代码
- 支持断点续传（Range header）
- 进度回调：生成 `tokio::task::spawn_blocking` + channel 模式

**文件**：`crates/auto-man/src/rust_ui.rs`

### 步骤 4：a2r WebSocket 客户端适配

**目标**：让 Rust 前端能生成 WebSocket 客户端代码。

**改动**：
- `rust_ui.rs` 新增 `generate_ws_functions`：生成 `tungstenite` 客户端代码
- `Cargo.toml` 模板增加 `tungstenite` 依赖
- 生成 `ws_connect/ws_send/ws_on_message/ws_close` 函数

**文件**：`crates/auto-man/src/rust_ui.rs` + `generate_cargo_toml`

### 步骤 5：VM 测试用例

**目标**：为所有新 HTTP 特性编写 VM 测试。

**改动**：
- 新增 `plan349_tests.rs`（或扩展 plan340_tests）
- 测试 TLS skip_verify（对 HTTPS URL 发请求）
- 测试 multipart upload（mock server 接收文件）
- 测试 download + resume（mock server 支持 Range）
- 测试 download_with_progress（验证进度事件格式）
- 测试 WebSocket echo（启动 echo server → connect → send → on_message）

### 步骤 6：a2r 测试用例

**目标**：验证 a2r 生成的 Rust 代码能正确编译和运行。

**改动**：
- 对 015-notes 项目跑 `auto run --render rust`，验证 API client 编译
- 扩展测试：验证生成的 Rust 代码包含 TLS/multipart/download/ws 函数
- 检查 `Cargo.toml` 依赖是否正确

### 步骤 7：普通 HTTP 请求异步化（Plan 344 路径 B）

**目标**：普通 GET/POST 也支持真异步（AWAIT_FUTURE 外部 future 挂起），消除 UI 冻结。

**改动**：
- engine.rs FutureValue 加 external_result 字段
- AutoTask 加 waiting_future_id 字段
- AWAIT_FUTURE Pending 分支分外部/内部两条路
- run_task_loop 加 future-wake 轮询
- 见 Plan 344 详细设计

### 步骤 8：易用性增强

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
