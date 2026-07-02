# Web Framework Gap Analysis：Auto HTTP 库 vs 成熟 Web Framework

> **状态**：调研文档
> **基于**：v0.4.1 + Plan 349/350（截至 2026-07-02）
> **对比目标**：Axum (Rust)、Spring Boot (Java)、Express (Node.js)、Django/FastAPI (Python)

## 一、当前能力盘点

### 已有 Server 能力

| 能力 | 实现 | 说明 |
|------|------|------|
| **路由** | ✅ | `:param` 路径参数（`/api/notes/:id`）|
| **HTTP 方法** | ✅ | GET/POST/PUT/DELETE |
| **`#[api]` 声明式路由** | ✅ | 从注解自动注册路由 |
| **JSON 请求/响应** | ✅ | body 解析 + JSON 序列化 |
| **路径参数提取** | ✅ | `build_handler_args` 把 `:id` 推入 handler 栈 |
| **自定义状态码** | ✅ | `response_status` |
| **自定义响应头** | ✅ | `response_header` |
| **HTML/Bytes 响应** | ✅ | `response_html` / `response_bytes` |
| **静态文件服务** | ✅ | `server_static` |
| **SSE 流式推送** | ✅ | generator yield |
| **WebSocket echo** | ✅ | 帧解析 + 回显 |
| **异步多连接** | ✅ | tokio + spawn_local + yield_now |
| **CORS（a2r 后端）** | ✅ | Axum tower-http |
| **TLS/HTTPS** | ✅ | client 侧（reqwest）；server 侧靠反向代理 |

### 已有 Client 能力

| 能力 | 实现 |
|------|------|
| GET/POST/PUT/DELETE | ✅ |
| HTTPS + TLS 配置 | ✅ |
| Cookie 管理 | ✅ |
| gzip 压缩 | ✅ |
| 文件上传 multipart | ✅ |
| 文件下载 + 断点续传 + 进度 | ✅ |
| SSE 异步消费 | ✅ |
| WebSocket 客户端 | ✅ |
| 普通请求异步化 | ✅ |
| Bearer/API Key Auth | ✅ |

## 二、与成熟 Web Framework 的差距

### 🔴 核心缺失（阻塞生产使用）

| # | 能力 | 对比 | 影响 |
|---|------|------|------|
| 1 | **Query 参数解析** | Axum `Query<T>`、Express `req.query` | `?page=1&size=10` 无法解析 |
| 2 | **Request body → 强类型** | Axum `Json<T>`、Spring `@RequestBody` | body 只能当原始字符串处理 |
| 3 | **错误处理中间件** | Axum `catch_unwind`、Express `errorHandler` | handler panic = 连接断开，无统一错误响应 |
| 4 | **请求日志/指标** | Spring Actuator、Express morgan | 无请求耗时/状态码日志 |
| 5 | **输入验证** | Spring `@Valid`、FastAPI Pydantic | 无字段类型/范围验证 |

### 🟡 中等缺失（影响开发体验）

| # | 能力 | 对比 | 影响 |
|---|------|------|------|
| 6 | **路由分组/嵌套** | Axum `Router::nest()`、Express `router.use()` | 无法 `/api/v1/...` 分组管理 |
| 7 | **中间件链** | Axum `from_fn()`、Express `app.use()` | 无认证/日志/CORS 中间件复用 |
| 8 | **Cookie 读取（服务端）** | Express `req.cookies`、Spring `@CookieValue` | 无法读请求里的 Cookie |
| 9 | **Session 管理** | Spring Session、Express `express-session` | 无登录会话 |
| 10 | **文件上传接收（服务端）** | Express `multer`、Spring `MultipartFile` | 服务端不能解析 multipart 请求 |
| 11 | **URL 重定向** | Express `res.redirect()`、Spring `redirect:` | 无 `302 Redirect` |
| 12 | **请求 ID / 链路追踪** | Spring Sleuth、Axum `RequestId` | 无分布式追踪支持 |

### 🟢 低优先级（锦上添花）

| # | 能力 | 对比 | 影响 |
|---|------|------|------|
| 13 | **模板引擎（SSR）** | Django Templates、Tera、Handlebars | 无服务端 HTML 渲染（但有 response_html） |
| 14 | **OpenAPI/Swagger 自动生成** | FastAPI、SpringDoc | 无 API 文档自动生成 |
| 15 | **Rate Limiting** | Express `rate-limit`、Spring `@RateLimiter` | 无请求频率限制 |
| 16 | **WebSocket 消息路由（服务端）** | Axum channels、Spring STOMP | 当前只有 echo，无业务消息分发 |
| 17 | **HTTP/2 推送** | — | 无 |
| 18 | **Graceful Shutdown** | Axum `shutdown_signal`、Spring `preStop` | 无优雅停机 |
| 19 | **静态文件缓存头** | Express `maxAge`、Spring `Cache-Control` | 无 ETag/Cache-Control |
| 20 | **请求体大小限制** | Express `limit`、Spring `maxFileSize` | 无上传大小保护 |

## 三、实施路线建议

### 阶段 1：请求解析（最高优先级）

让 handler 能拿到结构化的请求数据：

**1a. Query 参数解析**
```auto
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes(page int, size int) []Note {
    // page 和 size 从 ?page=1&size=10 自动提取
}
```
改动：`build_handler_args` 增加 query string 解析，`match_route` 提取 query 参数。

**1b. Request body → JSON 对象**
```auto
#[api(method = "POST", path = "/api/notes")]
pub fn create_note(note Note) Note {
    // note 从 request body JSON 自动反序列化
}
```
改动：`build_handler_args` 检测 Content-Type: application/json，把 body 解析为参数。

**1c. 表单解析（urlencoded）**
```
POST /api/contact
name=Alice&email=alice@example.com
```
改动：`build_handler_args` 检测 Content-Type: application/x-www-form-urlencoded。

### 阶段 2：中间件 & 错误处理

**2a. 中间件链**
```auto
http.server.use(fn(req, next) {
    // 认证、日志、CORS 等
    next(req)
})
```

**2b. 统一错误处理**
handler panic 时返回 `500 Internal Server Error`（而非断开连接）。

**2c. 请求日志**
自动记录每个请求的 method/path/status/duration。

### 阶段 3：路由增强

**3a. 路由分组**
```auto
http.server.group("/api/v1", fn(group) {
    group.get("/notes", list_notes)
    group.post("/notes", create_note)
})
```

**3b. 通配符路由**
`/static/*filepath` 捕获剩余路径。

**3c. URL 重定向**
`http.response.redirect(url, 302)`。

### 阶段 4：会话 & 认证

**4a. Cookie 读取（服务端）**
**4b. Session 管理**
**4c. JWT 验证中间件**

### 阶段 5：生产化

**5a. 文件上传接收（服务端 multipart）**
**5b. 请求体大小限制**
**5c. Graceful Shutdown**
**5d. OpenAPI 自动生成**
**5e. Rate Limiting**

## 四、总结

Auto 当前的 HTTP 库已经具备了 **Web Framework 的骨架**——路由、handler dispatch、JSON 序列化、SSE、WebSocket、异步 IO。距离一个**生产可用的 Web Framework**，最核心的差距是：

1. **请求解析**（query/body/form → 强类型参数）——阻塞 API 开发
2. **中间件链**——阻塞认证/日志/CORS 复用
3. **错误处理**——handler panic 导致连接断开
4. **文件上传接收**——只有 client 上传，server 不能接收

建议按"阶段 1 → 2 → 3"的顺序实施。阶段 1 改动最小（集中在 `build_handler_args`），收益最大。
