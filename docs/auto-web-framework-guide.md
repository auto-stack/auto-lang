# Auto Web Framework 设计文档与使用指南

> **版本**：v0.4.1+ (2026-07)
> **状态**：生产可用（VM 侧完整，a2r 侧 client 完整）

## 一、概述

Auto 内置了一套完整的 HTTP Web Framework，覆盖 Server 和 Client 两侧，支持同步/异步、流式/非流式、HTTP/WebSocket。开发者用 Auto 语言（`.at` 文件）编写后端逻辑，通过 `#[api]` 注解声明路由，前端通过 HTTP/WebSocket 调用。

### 架构

```
┌──────────────────────────────────────────────────────────┐
│                    Auto Web Framework                     │
├──────────────────┬───────────────────────────────────────┤
│     Server       │              Client                   │
├──────────────────┼───────────────────────────────────────┤
│ #[api] 路由声明   │ http.get/post/put/delete             │
│ 路径参数 (:id)    │ RequestBuilder 链式 API              │
│ Query 参数解析    │ HTTPS + TLS 配置                     │
│ Cookie/Auth 解析  │ Cookie 管理 + gzip                  │
│ JSON 响应        │ 文件上传 (multipart)                 │
│ SSE 流式推送     │ 文件下载 + 断点续传 + 进度           │
│ WebSocket echo   │ SSE 异步消费                         │
│ 静态文件         │ WebSocket 客户端                     │
│ 请求日志         │ 异步非阻塞 (不冻结 UI)               │
│ 错误处理 (500)   │                                      │
│ 请求体限制 (10MB)│                                      │
└──────────────────┴───────────────────────────────────────┘
```

## 二、Server 端使用

### 2.1 声明式路由 (`#[api]`)

在 `.at` 文件中使用 `#[api]` 注解声明 HTTP 端点：

```auto
/// 获取所有笔记
/// GET /api/notes
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    return db.all_notes()
}

/// 获取单个笔记
/// GET /api/notes/:id
#[api(method = "GET", path = "/api/notes/:id")]
pub fn get_note(id int) ?Note {
    return db.find_note(id)
}

/// 创建笔记
/// POST /api/notes
#[api(method = "POST", path = "/api/notes")]
pub fn create_note(title str, body str) Note {
    return db.create_note(title, body)
}

/// 更新笔记
/// PUT /api/notes/:id
#[api(method = "PUT", path = "/api/notes/:id")]
pub fn update_note(id int, title str, body str) ?Note {
    return db.update_note(id, title, body)
}

/// 删除笔记
/// DELETE /api/notes/:id
#[api(method = "DELETE", path = "/api/notes/:id")]
pub fn delete_note(id int) bool {
    return db.delete_note(id)
}
```

### 2.2 参数类型

| 参数来源 | 示例 | 说明 |
|---------|------|------|
| **路径参数** | `/api/notes/:id` → `id int` | 从 URL 路径提取，自动类型推断（int/str） |
| **Query 参数** | `?page=1&size=10` | 自动解析为 JSON 对象推入 handler |
| **Request body** | POST/PUT body | JSON 原样推入；urlencoded 转为 JSON |
| **Cookie/Auth** | `Cookie:` / `Authorization:` | 解析为 `{"cookies":{},"auth":""}` 推入 |

### 2.3 响应类型

| 返回值 | HTTP 响应 |
|--------|----------|
| `[]Note` / `List<T>` | `200 OK` + JSON 数组 |
| `Note` / struct | `200 OK` + JSON 对象 |
| `?Note` (Some) | `200 OK` + JSON 对象 |
| `?Note` (None) | `200 OK` + `null` |
| `bool` | `200 OK` + `true`/`false` |
| `int` | `200 OK` + 数字 |
| generator/~Iter | `200 OK` + SSE 流（`text/event-stream`） |
| handler panic | `500` + `{"error":"internal server error","detail":"..."}` |

### 2.4 SSE 流式推送

Handler 返回一个 generator 时自动变为 SSE：

```auto
#[api(method = "GET", path = "/api/stream")]
pub fn live_updates() ~Iter<str> {
    for i in 0..10 {
        yield "Event " + i.to_string()
    }
}
```

客户端收到：
```
data: Event 0

data: Event 1

...
```

### 2.5 请求日志

每个请求自动记录到 stderr：
```
[HTTP] GET /api/notes → 200 (3ms)
[HTTP] POST /api/notes → 200 (5ms)
[HTTP] GET /api/stream → 200 SSE (2ms)
[HTTP] GET /api/notes/999 → 500 (handler 'get_note' error: ..., 1ms)
```

## 三、Client 端使用

### 3.1 基础请求

```auto
// 简单 GET
let resp = http.get("https://api.example.com/data")

// JSON GET（返回 JSON 字符串）
let json_str = http.get_json("http://localhost:8080/api/notes")
let notes = json.to_value(json_str)  // 解析为 VM 值

// POST JSON
let resp = http.post_json("http://localhost:8080/api/notes", "{\"title\":\"Hello\"}")
```

### 3.2 RequestBuilder 链式 API

```auto
let resp = http.request("GET", "https://api.example.com/data")
    .header("Authorization", "Bearer token123")
    .header("Accept", "application/json")
    .timeout(5000)
    .cookie_store(true)
    .gzip(true)
    .tls_skip_verify(true)    // 开发环境跳过证书验证
    .tls_ca_cert("/path/to/ca.pem")  // 自定义 CA
    .send()
```

### 3.3 文件上传

```auto
// 简单上传
let resp = http.upload("https://api.example.com/upload", "/path/to/file.png")

// 链式上传（带表单字段）
let resp = http.request("POST", url)
    .multipart_file("file", "/path/to/avatar.png")
    .multipart_text("description", "My profile photo")
    .send()
```

### 3.4 文件下载

```auto
// 下载到文件
http.download("https://example.com/large-file.zip", "/downloads/file.zip")

// 断点续传
http.download_resume("https://example.com/large-file.zip", "/downloads/file.zip", 1024)

// 带进度条（非阻塞，不冻结 UI）
for progress in http.download_with_progress(url, "/downloads/file.zip") {
    let p = json.to_value(progress)
    print("Downloaded: " + p["percent"].to_string() + "%")
}
```

### 3.5 SSE 客户端

```auto
for event in http.sse_get_stream("http://localhost:8080/api/stream") {
    print("Received: " + event)
}
```

### 3.6 WebSocket

```auto
let conn = ws.connect("wss://api.example.com/realtime")
ws.send(conn, "Hello, WebSocket!")
for msg in ws.on_message(conn) {
    print("Received: " + msg)
}
ws.close(conn)
```

## 四、运行模式

### 4.1 八种运行模式

| 命令 | 前端 | 后端 | 通讯 |
|------|------|------|------|
| `auto run --render vm` | VM | VM | 同进程直调 |
| `auto run --render vm --no-merge` | VM | VM server | HTTP |
| `auto run --render vm --server rust --no-merge` | VM | Rust axum | HTTP |
| `auto run --render rust` | Rust (a2r) | Rust | 同进程直调 |
| `auto run --render rust --no-merge` | Rust (a2r) | Rust axum | HTTP |
| `auto run --render vue --server rust --no-merge` | Vue | Rust axum | HTTP |
| `auto run --render vue --server vm --no-merge` | Vue | VM server | HTTP |
| `auto run --render vue` | Vue | Rust axum | HTTP |

### 4.2 端口配置

```bash
# 后端端口（-B）
auto run --render vm --no-merge -B 7777

# 前端端口（-F，仅 Vite）
auto run --render vue -F 3001
```

## 五、015-notes 示例可用的新特性

当前 015-notes 示例可以用以下新特性增强：

### 5.1 Query 参数分页（已支持）

```auto
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    // query params 自动解析，可用分页
    return db.all_notes()
}
// GET /api/notes?page=1&size=10 → query params 自动提取
```

### 5.2 请求日志（自动启用）

所有请求自动记录，无需改动代码。

### 5.3 错误处理（自动启用）

handler 出错自动返回 `500 + {"error":"..."}`。

### 5.4 Cookie/Auth（可用于认证）

handler 最后一个参数接收 `{"cookies":{},"auth":""}` 元数据：
```auto
pub fn list_notes(meta str) []Note {
    let m = json.to_value(meta)
    let token = m["auth"]
    if token == "" {
        return []  // 未认证
    }
    return db.all_notes()
}
```

### 5.5 异步 HTTP（不冻结 UI）

前端 `http.get_json` 已改为非阻塞，UI 不会冻结。

## 六、Web Framework 能力对照

| 能力 | Auto | Axum | Spring Boot | Express |
|------|:----:|:----:|:-----------:|:-------:|
| 路由 | ✅ | ✅ | ✅ | ✅ |
| 路径参数 | ✅ | ✅ | ✅ | ✅ |
| Query 参数 | ✅ | ✅ | ✅ | ✅ |
| Body 解析 (JSON) | ✅ | ✅ | ✅ | ✅ |
| Body 解析 (表单) | ✅ | ✅ | ✅ | ✅ |
| Cookie 读取 | ✅ | ✅ | ✅ | ✅ |
| Auth header | ✅ | ✅ | ✅ | ✅ |
| JSON 响应 | ✅ | ✅ | ✅ | ✅ |
| 自定义状态码/头 | ✅ | ✅ | ✅ | ✅ |
| 重定向 | ✅ | ✅ | ✅ | ✅ |
| 静态文件 | ✅ | ✅ | ✅ | ✅ |
| SSE | ✅ | ✅ | ✅ | ✅ |
| WebSocket | ✅ | ✅ | ✅ | ✅ |
| 请求日志 | ✅ | ✅ | ✅ | ✅ |
| 错误处理 | ✅ | ✅ | ✅ | ✅ |
| 请求体限制 | ✅ | ✅ | ✅ | ✅ |
| 中间件链 | ❌ | ✅ | ✅ | ✅ |
| Session 管理 | ❌ | ✅ | ✅ | ✅ |
| 模板引擎 (SSR) | ❌ | ❌ | ✅ | ✅ |
| OpenAPI/Swagger | ❌ | ❌ | ✅ | ❌ |
| Rate Limiting | ❌ | ❌ | ✅ | ✅ |
| Graceful Shutdown | ❌ | ✅ | ✅ | ❌ |

## 七、后续路线图

| 优先级 | 特性 | 难度 |
|--------|------|------|
| 🟡 | 中间件链（`http.server.use()`） | 中 |
| 🟡 | Session 管理（服务端 Cookie-Session） | 中 |
| 🟢 | Graceful Shutdown（Ctrl+C → 完成进行中请求） | 低 |
| 🟢 | OpenAPI 自动生成（从 #[api] 注解） | 中 |
| 🟢 | Rate Limiting | 低 |
| 🟢 | 模板引擎（SSR HTML 渲染） | 中 |
| 🟢 | 服务端 multipart 文件上传接收 | 中 |
