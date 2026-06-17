# Auto HTTP/HTTPS Server 标准库 Spec

> **Status**: Draft v1
> **范围**: 定义 Auto 标准库中 HTTP/HTTPS Server 的统一 API,覆盖同步/异步 × 普通/流 四种 handler 模式
> **目标**: VM 模式和 A2R 模式共享同一套 API,底层封装 Axum/Tokio
> **关联**: Plan 321(generator 运行时)提供 yield/~Iter/~Stream 原语;本 Spec 定义 HTTP 层如何消费它们

---

## §1 设计原则

1. **API 统一**:用户写一套代码,VM 和 A2R 两种模式行为一致
2. **底层封装 Axum/Tokio**:标准库底层调 Axum,用户不直接接触 Rust 框架
3. **不染色**:同步 handler 不需要 async,异步 handler 用 `~T` 返回类型表达
4. **注解 = Builder 的语法糖**:`#[api]` 底层转换为 `http.server().get()` builder 代码
5. **未来可适配其他生态**:API 设计独立于 Rust(为 Kotlin/C/ArkTS 留空间)

---

## §2 两种 API 形式

### 2.1 注解模式(推荐,简洁)

```auto
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note {
    return db.all_notes()
}

#[api(method = "GET", path = "/api/notes/:id")]
pub fn get_note(id int) ?Note {
    return db.find(id)
}
```

### 2.2 Builder 模式(灵活,支持中间件等)

```auto
pub fn main() {
    http.server()
        .get("/api/notes", list_notes)
        .get("/api/notes/:id", get_note)
        .post("/api/notes", create_note)
        .listen("0.0.0.0:8080")
}
```

**注解模式在编译期/加载期展开为 Builder 模式**——两者底层等价。

---

## §3 四种 Handler 模式

handler 的返回类型决定其模式:

### 3.1 同步普通 `fn() T`

```auto
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() []Note { ... }
```

- **行为**:函数体同步执行,返回值自动 JSON 序列化为单次 HTTP 响应
- **返回类型**:`str`(原样文本) / `int`/`bool`(JSON 字面量) / `Note`(JSON 对象) / `[]Note`(JSON 数组) / `?Note`(200+JSON 或 404)
- **VM**:call_fn_by_name 同步执行 → 取返回值 → Value→JSON 序列化
- **A2R**:普通 `fn` → Axum `Json<T>` wrapper

### 3.2 同步流 `fn() ~Iter<T>`

```auto
#[api(method = "GET", path = "/api/counter")]
pub fn counter() ~Iter<int> {
    for i in 0..100 {
        yield i
    }
}
```

- **行为**:返回 generator,HTTP 层持续拉取 next() + 写 SSE 帧
- **返回类型**:`~Iter<T>` — T 是每帧的数据
- **HTTP 响应**:`Content-Type: text/event-stream`,持续推 `data: <T 的 JSON>\n\n`
- **VM**:Iterator::Generator next() 拉取 → 序列化 → write + flush
- **A2R**:`impl Iterator<Item = T>` → Axum `Sse<Stream>`
- **适用**:纯计算流(无需 await 外部 I/O)

### 3.3 异步普通 `fn() ~T`

```auto
#[api(method = "GET", path = "/api/proxy")]
pub fn fetch_proxy(url str) ~str {
    let resp = http.get(url)
    return resp.text()
}
```

- **行为**:函数体可能 await 外部 I/O,完成后返回单次响应
- **返回类型**:`~T` — 同步普通 T 的异步版本
- **VM**:Future + handle_await_future 执行 → 取值 → JSON
- **A2R**:`async fn` → Axum handler 直接 await
- **适用**:handler 内调外部 API 后返回

### 3.4 异步流 `fn() ~Stream<T>`

```auto
#[api(method = "POST", path = "/api/chat/:sid/stream")]
pub fn chat_stream(sid str) ~Stream<str> {
    let session = db.get_session(sid)
    for {
        let token = llm.next_token(session).await   // await 外部 I/O
        if token == "[DONE]" { return }
        yield token                                   // 推一帧
    }
}
```

- **行为**:函数体内 yield + await 混合 —— 既推流又等外部 I/O
- **返回类型**:`~Stream<T>` — 每帧数据,可能跨 await 点
- **HTTP 响应**:`Content-Type: text/event-stream`,持续推帧
- **VM**:需要 VM 真异步调度(yield 挂起 + await 恢复)—— 这是最终形态
- **A2R**:`impl Stream<Item = T>` → Axum `Sse`
- **适用**:**SSE 流式聊天**(auto-musk 核心场景)

### 3.5 模式判定规则

HTTP 层根据 handler 返回类型自动判定模式:

| 返回类型 | 模式 | HTTP 响应 |
|---|---|---|
| `T`(非 `~`) | 同步普通 | 单次 JSON |
| `~Iter<T>` | 同步流 | SSE 流 |
| `~T`(非 Iter/Stream) | 异步普通 | 单次 JSON(async) |
| `~Stream<T>` | 异步流 | SSE 流(async) |

**用户不需要指定模式——返回类型自动决定。**

---

## §4 Request/Response 对象(参考 Axum Extractor)

### 4.1 参数注入(编译器自动,无需 Request 对象)

handler 的参数由编译器/HTTP 层自动注入:

```auto
// 路径参数 :id → 直接注入为函数参数
#[api(method = "GET", path = "/api/notes/:id")]
pub fn get_note(id int) ?Note { ... }

// POST body JSON → 自动反序列化为函数参数
#[api(method = "POST", path = "/api/notes")]
pub fn create_note(title str, body str) Note { ... }
```

**注入规则**:
- 路径参数(`:id`):按名字匹配函数参数,自动类型转换(`int`/`str`)
- POST/PUT body:JSON 对象的字段按名字匹配函数参数

### 4.2 Request 对象(高级,显式访问)

当需要访问 query/header/cookie 时,handler 接收一个 `Request` 参数:

```auto
#[api(method = "GET", path = "/api/search")]
pub fn search(req Request) []Result {
    let q = req.query("q")
    let page = req.query_int("page")    // 带类型的 query
    let auth = req.header("Authorization")
    return db.search(q, page)
}
```

**Request API**(已声明于 `http.at:48-85`):
```auto
type Request
fn Request.method(self) str           // "GET"/"POST"/...
fn Request.path(self) str             // "/api/search"
fn Request.query(self, key) str       // query string 参数
fn Request.query_int(self, key) int   // 带类型的 query
fn Request.header(self, key) str      // HTTP header
fn Request.body(self) []byte          // 原始 body
fn Request.text(self) str             // body as text
fn Request.json[T](self) ?T           // body as typed JSON
fn Request.param(self, key) str       // 路径参数
```

**约定**:handler 如果声明了 `Request` 类型的参数,HTTP 层注入 Request 对象;否则只注入路径/body 参数。

### 4.3 Response 对象

handler 可以返回 `Response` 对象(而非裸值)来控制状态码/headers:

```auto
#[api(method = "GET", path = "/api/secret")]
pub fn secret(req Request) Response {
    if req.header("Authorization") == "" {
        return http.response().status(401).text("Unauthorized")
    }
    return http.response().status(200).json({ secret: "42" })
}
```

**Response API**(已声明于 `http.at:92-119`):
```auto
type Response
fn response() Response                            // 创建
fn Response.status(self, code int) Response       // 设状态码
fn Response.header(self, key, val str) Response   // 设 header
fn Response.text(self, body str) Response         // 文本 body
fn Response.html(self, body str) Response         // HTML body
fn Response.json[T](self, data T) Response        // JSON body
// 快捷函数
fn ok(body str) Response                          // 200
fn not_found(msg str) Response                    // 404
fn json_response[T](status int, data T) Response  // JSON + status
```

---

## §5 SSE 帧类型

### 5.1 Stream 的元素类型

`~Iter<T>` / `~Stream<T>` 的 `T` 决定每帧的 SSE 格式:

| `T` 的类型 | SSE 帧 | 示例 |
|---|---|---|
| `str` | `data: <str 的 JSON>\n\n` | `data: "hello"\n\n` |
| `int` / `bool` | `data: <JSON 字面量>\n\n` | `data: 42\n\n` |
| `Note`(struct) | `data: <struct 的 JSON>\n\n` | `data: {"id":1,"title":"x"}\n\n` |
| `SSEEvent`(结构体) | 完整 SSE 帧(event/id/data) | `event: token\ndata: "hi"\n\n` |

### 5.2 SSEEvent 类型(可选,高级 SSE)

需要自定义 event name / event id 时:

```auto
type SSEEvent {
    event str     // SSE event 字段(可选,默认 "message")
    data str      // SSE data 字段
    id str        // SSE id 字段(可选)
}

#[api(method = "GET", path = "/api/events")]
pub fn events() ~Iter<SSEEvent> {
    yield SSEEvent { event: "token", data: "hello" }
    yield SSEEvent { event: "done", data: "[DONE]" }
}
```

HTTP 层检测到 `T = SSEEvent` 时,按完整 SSE 协议分帧(含 event/id 字段)。

### 5.3 默认行为(简化)

**MVP**: `T` 一律 JSON 序列化,格式为 `data: <JSON>\n\n`。`SSEEvent` 结构体支持留后续。

---

## §6 HTTPS / TLS

### 6.1 API 形式

```auto
// HTTP
http.server().get("/api", handler).listen("0.0.0.0:8080")

// HTTPS
https.server()
    .cert("cert.pem")
    .key("key.pem")
    .get("/api", handler)
    .listen("0.0.0.0:443")
```

### 6.2 类型

```auto
type HttpsServer
fn https.server() HttpsServer
fn HttpsServer.cert(self, path str) HttpsServer   // TLS 证书路径
fn HttpsServer.key(self, path str) HttpsServer     // TLS 私钥路径
fn HttpsServer.get/post/put/delete(...)            // 同 Server
fn HttpsServer.listen(self, addr str) void
```

### 6.3 注解模式 + HTTPS

```auto
// 在 main() 里指定 HTTPS
pub fn main() {
    // #[api] 路由自动注册,只需指定 listen
    https.server().cert("cert.pem").key("key.pem").listen("0.0.0.0:443")
}
```

**MVP**: 先做 HTTP;HTTPS 留 Spec 定义,实现可后置。

---

## §7 Server 生命周期

### 7.1 自动启动(注解模式)

含 `#[api]` 的 `.at` 文件用 AutoVM 运行时,`main()` 执行后自动起 server:

```auto
#[api(method = "GET", path = "/api/hello")]
pub fn hello() str { return "hi" }

pub fn main() { print("starting") }
// main() 结束后自动 listen(默认 8080,或 AUTO_HTTP_PORT 环境变量)
```

### 7.2 手动启动(Builder 模式)

用户在 `main()` 里显式调 `listen`:

```auto
pub fn main() {
    http.server()
        .get("/api/hello", hello)
        .listen("0.0.0.0:3000")
}
```

### 7.3 优雅关闭

`Ctrl+C` / `SIGTERM` 时,server 停止 accept 新连接,等待活跃连接完成。

---

## §8 跨模式一致性

### 8.1 VM 模式 vs A2R 模式

| 行为 | VM 模式 | A2R 模式 | 一致性 |
|---|---|---|---|
| 路径参数 `:id` | 自动注入(int/str 转换) | Axum `Path<T>` | ✅ 行为相同 |
| Body JSON | `req.json[T]()` | Axum `Json<T>` | ✅ |
| 返回值序列化 | Value→JSON(需实现全类型) | serde_json | ✅(需补全 VM 序列化) |
| 流式响应 | Iterator::Generator next() | `Sse<impl Stream>` | ✅(SSE 帧格式统一) |
| 状态码 | Response.status() | Axum `StatusCode` | ✅ |
| HTTPS | 独立后续 | `axum-server` + `rustls` | ✅ |

### 8.2 统一实现层

```
Auto 标准库 http.at / https.at (API 声明)
         ↓
    ┌────┴────┐
    ↓         ↓
  VM shim   a2r 转译
(native)   (codegen)
    ↓         ↓
    └────┬────┘
         ↓
   Axum / Tokio (统一 Rust 实现)
```

VM shim 和 a2r 转译**底层都调 Axum/Tokio**,只是调用方式不同:
- VM: native Rust shim 封装 Axum,通过 `spawn_blocking`/专用线程桥接 `!Send` 的 VM
- A2R: 转译生成的 Rust 代码直接调 Axum(天然 `Send`)

---

## §9 实现优先级

| 优先级 | 能力 | 依赖 |
|---|---|---|
| **P0** | 同步普通 handler(str 返回) | Plan 312 已有 MVP |
| **P0** | 同步普通 handler(struct/[]T 自动序列化) | 需补 VM Value→JSON |
| **P1** | 同步流 handler(~Iter<T> + yield) | Plan 321 |
| **P1** | Request 对象注入(query/header) | 扩展 VM handler 调用 |
| **P2** | 异步普通 handler(~T) | VM 真异步调度 |
| **P2** | HTTPS | TLS 配置 |
| **P3** | 异步流 handler(~Stream<T>) | Plan 321 + VM 真异步 |
| **P3** | SSEEvent 类型 | SSE 协议扩展 |

---

## §10 不做(范围控制)

- **WebSocket**: 单独 Spec(双向通信,需要不同的连接模型)
- **文件上传(multipart)**: 后续增强
- **Session/Auth 中间件**: 后续增强
- **CORS**: 后续增强(或作为中间件)

---

## 附录 A:完整 API 声明(http.at)

现有 `http.at` 已声明了大部分 API(Server/Request/Response/HTTPStream)。本 Spec 的改动:

1. **新增**:`~Iter<T>` / `~Stream<T>` 作为 handler 返回类型(HTTP 层自动判定流模式)
2. **新增**:`SSEEvent` 类型(可选,高级 SSE)
3. **新增**:`HttpsServer` 类型 + `https.server()`
4. **确认**:Request/Response API 保持现有声明不变
5. **修正**:`Server.listen` 应标注为阻塞(void 返回,server 运行直到停止)

## 附录 B:auto-musk SSE 流式聊天写法(目标形态)

```auto
// POST /api/forge/chats/{sid}/stream
// 用户发消息 → 后端调 LLM → SSE 逐 token 推流
#[api(method = "POST", path = "/api/forge/chats/:sid/stream")]
pub fn chat_stream(sid str, message str) ~Stream<str> {
    let session = db.get_session(sid)
    let prompt = session.build_prompt(message)

    // 调 LLM 流式 API(http.post_stream 是 SSE 客户端)
    let llm_stream = http.post_stream("https://api.openai.com/v1/chat/completions", prompt)

    // 逐 token 转发
    for {
        let chunk = llm_stream.next()
        if llm_stream.is_done() {
            yield "[DONE]"
            return
        }
        let token = json_parse_token(chunk)
        yield token
    }
}
```

HTTP 层检测到 `~Stream<str>` 返回类型:
1. 设 `Content-Type: text/event-stream`
2. 持续调 generator 的 next()
3. 每个 yield 的 str → `data: <str>\n\n` + flush
4. generator return → 流结束,关闭连接
