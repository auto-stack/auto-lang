# Plan 352: Web Framework 四项缺失能力设计与实施计划

> **状态**：设计文档
> **前置**：Plan 346（Web Framework Gap Analysis 五阶段已完成）
> **目标**：中间件链、Session 管理、模板引擎（SSR）、OpenAPI 自动生成

## 现状

Plan 346 五阶段完成后，Auto Web Framework 已具备 16/20 项核心能力。
本计划补齐剩余 4 项，使其与 Axum/Spring Boot/Express 全面对齐。

---

## 能力 1：中间件链（Middleware Chain）

### 需求

让开发者注册在路由处理**之前**执行的通用逻辑：认证、日志增强、CORS、请求 ID 等。

### 设计

**API 语法**（Auto 语言层）：
```auto
// 注册全局中间件
http.server.use(fn(req, next) {
    // 认证检查
    let token = req.cookies["auth"]
    if token == "" {
        return http.response.status(401)
    }
    // 调用下一个 handler
    next(req)
})

// CORS 中间件
http.server.use(fn(req, next) {
    let resp = next(req)
    resp.header("Access-Control-Allow-Origin", "*")
    return resp
})
```

**实现方案**：

1. **新 native `auto.http.server_use`**：注册一个中间件函数（VM 闭包/fn 地址）到全局 `MIDDLEWARE_CHAIN: Vec<u32>`（函数地址列表）。

2. **`handle_connection_async` 改造**：在路由匹配成功后、`call_fn_by_name` 之前，按顺序执行中间件链。每个中间件接收请求元数据 + 一个 `next` 回调。中间件可以：
   - 短路返回（如 401/403）
   - 修改请求（添加 header/上下文）
   - 修改响应（添加 CORS header）
   - 调用 `next` 继续链

3. **请求上下文对象**：中间件需要一个结构化的请求对象（method, path, headers, cookies, body）。新增 `auto.http.request.create` native 构造一个 Request 对象推入堆。

4. **执行模型**：中间件链在 VM task 内同步执行（和 handler 一样），不走异步。每个中间件是一个 VM fn，通过 `call_fn_by_name` 调用。

**关键文件**：
- `http_server.rs` — `handle_connection_async` 加中间件链执行
- `stdlib.rs` — `shim_http_server_use` native
- `native_catalog.rs` — 注册 `auto.http.server_use`

**复杂度**：中（需要设计 Request 对象 + next 回调机制）

---

## 能力 2：Session 管理

### 需求

基于 Cookie 的服务端会话：登录后颁发 session ID，后续请求自动关联用户。

### 设计

**API 语法**：
```auto
#[api(method = "POST", path = "/api/login")]
pub fn login(username str, password str, meta str) str {
    // 验证凭据...
    let session_id = session.create({"user": username, "role": "admin"})
    return session_id  // 客户端存为 Cookie
}

#[api(method = "GET", path = "/api/profile")]
pub fn profile(meta str) str {
    let m = json.to_value(meta)
    let sid = m["cookies"]["session_id"]
    let data = session.get(sid)
    if data == null {
        return http.response.redirect("/login", 302)
    }
    return data["user"]
}
```

**实现方案**：

1. **新 native `auto.session.create`**：生成随机 session ID（UUID），存入全局 `SESSIONS: Mutex<HashMap<String, Value>>`，返回 session ID 字符串。

2. **新 native `auto.session.get(id)`**：从全局表中查找 session 数据。

3. **新 native `auto.session.set(id, key, value)`**：更新 session 数据。

4. **新 native `auto.session.destroy(id)`**：删除 session。

5. **Session 存储**：`lazy_static Mutex<HashMap<String, serde_json::Value>>`（内存存储，进程重启后失效。后续可加 Redis/文件持久化）。

6. **Cookie 集成**：handler 通过 `meta` 参数（已在 Plan 346 阶段4 实现）读取 `session_id` Cookie，调 `session.get()` 恢复会话。

**关键文件**：
- `stdlib.rs` — 4 个 session native shim
- `native_catalog.rs` — 注册 native

**复杂度**：低（纯内存 HashMap + 4 个简单 shim）

---

## 能力 3：模板引擎（SSR HTML 渲染）

### 需求

服务端渲染 HTML 页面（模板 + 数据 → HTML 字符串），用于 SSR、邮件模板、报告生成。

### 设计

**API 语法**：
```auto
let html = template.render("notes_page", {
    "title": "My Notes",
    "notes": notes,
    "user": "Alice"
})
http.response.html(html)
```

**实现方案**：

采用**字符串模板**方案（简单、不引入外部依赖）：

1. **模板语法**：`{{variable}}` 插值 + `{{#if cond}}...{{/if}}` 条件 + `{{#each items}}...{{/each}}` 循环。

2. **新 native `auto.template.compile(name, template_str)`**：编译模板字符串存入全局 `TEMPLATES: HashMap<String, CompiledTemplate>`。

3. **新 native `auto.template.render(name, data_json)`**：用 JSON 数据渲染编译后的模板，返回 HTML 字符串。

4. **CompiledTemplate**：解析后的模板 AST（Vec<TemplateNode>），支持：
   - `Text(String)` — 纯文本
   - `Variable(String)` — `{{var}}` 插值，从 data JSON 取值
   - `If(String, Vec<TemplateNode>, Vec<TemplateNode>)` — 条件块
   - `Each(String, Vec<TemplateNode>)` — 循环块

5. **模板渲染**：递归遍历 AST，用 `serde_json::Value` 查值。

**示例模板**：
```html
<h1>{{title}}</h1>
<ul>
{{#each notes}}
    <li>{{this.title}} - {{this.time}}</li>
{{/each}}
</ul>
{{#if user}}
    <p>Welcome, {{user}}!</p>
{{/if}}
```

**关键文件**：
- `stdlib.rs` 或新 `template.rs` 模块 — 模板编译 + 渲染
- `native_catalog.rs` — 注册 `auto.template.compile/render`

**复杂度**：中（模板解析 + 条件/循环 AST + JSON 查值）

---

## 能力 4：OpenAPI 自动生成

### 需求

从 `#[api]` 注解自动生成 OpenAPI 3.0 规范文档，提供 Swagger UI。

### 设计

**API 语法**：
```auto
// 在 server 启动时自动暴露 /openapi.json 和 /docs
#[api(method = "GET", path = "/openapi.json")]
pub fn openapi_spec() str {
    return openapi.generate()
}
```

**实现方案**：

1. **新 native `auto.openapi.generate()`**：遍历全局 `HTTP_ROUTES`（已注册的 #[api] 路由），生成 OpenAPI 3.0 JSON。

2. **从 `#[api]` 注解提取信息**：
   - `method` + `path` → OpenAPI paths
   - `:param` → path parameters
   - 函数参数名 + 类型 → request body schema / query parameters
   - 返回类型 → response schema
   - 函数文档注释 → description

3. **Swagger UI**：`server_static` 挂载一个内置的 Swagger UI HTML 页面（从 CDN 加载 swagger-ui），指向 `/openapi.json`。

4. **生成的 JSON 格式**：
```json
{
  "openapi": "3.0.0",
  "info": {"title": "Notes API", "version": "1.0.0"},
  "paths": {
    "/api/notes": {
      "get": {
        "summary": "List all notes",
        "responses": {"200": {"description": "OK"}}
      },
      "post": {
        "summary": "Create a new note",
        "requestBody": {"content": {"application/json": {...}}},
        "responses": {"200": {"description": "OK"}}
      }
    },
    "/api/notes/{id}": {
      "get": {
        "parameters": [{"name": "id", "in": "path", "required": true, "schema": {"type": "integer"}}]
      }
    }
  }
}
```

**关键文件**：
- `stdlib.rs` — `shim_openapi_generate` native
- `http_server.rs` — 可选：自动注册 `/openapi.json` 和 `/docs` 路由
- `native_catalog.rs` — 注册 native

**复杂度**：中（JSON 拼接 + 路由元信息提取）

---

## 实施路线

| 阶段 | 能力 | 难度 | 预计工作量 | 优先级 |
|------|------|------|-----------|--------|
| 1 | Session 管理 | 低 | 小 | 🟡 中 |
| 2 | 中间件链 | 中 | 中 | 🟡 中 |
| 3 | 模板引擎（SSR） | 中 | 中 | 🟢 低 |
| 4 | OpenAPI 自动生成 | 中 | 中 | 🟢 低 |

建议按 1→2→3→4 顺序实施（从简单到复杂）。

## 不在本计划范围

- Graceful Shutdown（Ctrl+C → 完成进行中请求）—— 独立小改
- Rate Limiting —— 独立小改
- 服务端 multipart 文件上传接收 —— 独立中改
- Redis/数据库 Session 持久化 —— 后续
- 模板引擎 include/inherit —— 后续
