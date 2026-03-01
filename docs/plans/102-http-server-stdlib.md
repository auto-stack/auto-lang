# Plan 102: HTTP Server 标准库实现

**Status**: 🔄 Phase 1-4 Complete, Phase 5 Not Started
**Priority**: P1 (Foundation for Web Backend & a2vue)
**Owner**: AutoLang Team
**Created**: 2026-03-01
**Phase 1-4 Completed**: 2026-03-01
**Phase 5 Planned**: TBD
**Related**:
- [docs/design/http-server-stdlib.md](../design/http-server-stdlib.md) - API 设计文档
- [docs/design/autovm-task-msg.md](../design/autovm-task-msg.md) - Task/Msg 架构
- [docs/plans/069-autovm-global-vars.md](069-autovm-global-vars.md) - Task/Msg 实现（已完成）
- [docs/design/frontend-backend-communication.md](../design/frontend-backend-communication.md) - 前后端通讯架构

---

## 1. Objective

基于已完成的 **Task/Msg 异步并发框架**（Plan 069），实现 Auto 语言的标准库模块，使其具备编写 HTTP Server 的能力。

### 前置条件

- ✅ Task/Msg 异步框架（Plan 069）已完成
- ✅ SPAWN, SLEEP, SEND, RECV, CHAN_NEW 等操作码已实现
- ✅ Tokio 运行时已集成

---

## 2. 模块依赖关系

```
                    ┌──────────┐
                    │   http   │
                    └────┬─────┘
                         │
         ┌───────────────┼───────────────┐
         │               │               │
         ▼               ▼               ▼
    ┌─────────┐    ┌─────────┐    ┌─────────┐
    │   net   │    │  json   │    │   url   │
    └────┬────┘    └─────────┘    └─────────┘
         │
         ▼
    ┌─────────┐
    │  async  │  ← Task/Msg 已完成 (Plan 069)
    └────┬────┘
         │
         ├───────────────┐
         │               │
         ▼               ▼
    ┌─────────┐    ┌─────────┐
    │   log   │    │   env   │
    └─────────┘    └─────────┘
```

---

## 3. Implementation Phases

### Phase 1: 基础设施模块 (Foundation Modules)

**Goal**: 实现最基础的 async 扩展和 log/env 模块

#### 1.1 Async 模块扩展

**文件**:
```
stdlib/auto/
├── async.at            # Auto API 定义
├── async.vm.at         # VM 实现（基于 Plan 069）
└── async.rs.at         # a2r 实现（tokio 封装）
```

**API 定义** (async.at):
```auto
// 基于现有操作码的高级封装

/// 创建异步任务（封装 SPAWN 操作码）
fn async.spawn(f fn()) void

/// 创建 Channel（封装 CHAN_NEW 操作码）
fn async.channel<T>() (Sender<T>, Receiver<T>)

/// 当前任务休眠（封装 SLEEP 操作码）
fn async.sleep(ms int) void

/// 获取当前任务 ID
fn async.task_id() int

/// 让出执行（封装 YIELD 操作码）
fn async.yield_now() void
```

**实现策略**:
- 大部分 API 直接映射到已实现的操作码
- 添加类型安全的封装

**测试**:
```auto
// tests/async_test.at
fn test_spawn() {
    let done = async.channel()
    async.spawn(fn() {
        done.send(42)
    })
    let result = done.recv()
    assert(result == 42)
}
```

**状态**: [ ] 未开始

---

#### 1.2 Log 模块

**文件**:
```
stdlib/auto/
├── log.at              # Auto API 定义
├── log.vm.at           # VM 实现
└── log.rs.at           # a2r 实现
```

**API 定义** (log.at):
```auto
/// 调试日志
fn log.debug(msg str) void

/// 信息日志
fn log.info(msg str) void

/// 警告日志
fn log.warn(msg str) void

/// 错误日志
fn log.error(msg str) void

/// 设置日志级别
fn log.set_level(level str) void
```

**Rust 后端**:
```rust
// log.vm.at
#[vm]
fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

#[vm]
fn log_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}
```

**状态**: [ ] 未开始

---

#### 1.3 Env 模块

**文件**:
```
stdlib/auto/
├── env.at              # Auto API 定义
├── env.vm.at           # VM 实现
└── env.rs.at           # a2r 实现
```

**API 定义** (env.at):
```auto
/// 获取环境变量
fn env.get(key str) str?

/// 设置环境变量
fn env.set(key str, value str) void

/// 获取命令行参数
fn env.args() List<str>

/// 获取当前工作目录
fn env.cwd() str

/// 退出程序
fn env.exit(code int) void
```

**状态**: [ ] 未开始

---

### Phase 2: 网络模块 (Network Module)

**Goal**: 实现 TCP 网络通信能力

#### 2.1 Net 模块

**文件**:
```
stdlib/auto/
├── net.at              # Auto API 定义
├── net.vm.at           # VM 实现
└── net.rs.at           # a2r 实现
```

**API 定义** (net.at):
```auto
// ═══════════════════════════════════════════════════════════
// TCP 监听器
// ═══════════════════════════════════════════════════════════

type TcpListener

/// 绑定地址并创建监听器
fn net.tcp_bind(addr str) TcpListener?

/// 接受新连接（异步）
fn TcpListener.accept_async() TcpStream?

/// 获取本地地址
fn TcpListener.local_addr() str

// ═══════════════════════════════════════════════════════════
// TCP 流
// ═══════════════════════════════════════════════════════════

type TcpStream

/// 读取数据
fn TcpStream.read(buf []byte) int

/// 写入数据
fn TcpStream.write(data []byte) int

/// 读取所有数据直到 EOF
fn TcpStream.read_all() []byte

/// 关闭连接
fn TcpStream.close() void

/// 设置读取超时
fn TcpStream.set_read_timeout(ms int) void

/// 获取对端地址
fn TcpStream.peer_addr() str
```

**Rust 后端 (VM)**:
```rust
// net.vm.at
use tokio::net::{TcpListener as TokioListener, TcpStream as TokioStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[vm]
async fn tcp_bind(addr: &str) -> Option<TcpListener> {
    TokioListener::bind(addr).await.ok().map(TcpListener)
}

#[vm]
async fn tcp_listener_accept_async(listener: &mut TcpListener) -> Option<TcpStream> {
    listener.0.accept().await.ok().map(|(s, _)| TcpStream(s))
}

#[vm]
async fn tcp_stream_read_all(stream: &mut TcpStream) -> Vec<u8> {
    let mut buf = Vec::new();
    stream.0.read_to_end(&mut buf).await.unwrap();
    buf
}

#[vm]
async fn tcp_stream_write(stream: &mut TcpStream, data: &[u8]) {
    stream.0.write_all(data).await.unwrap();
}
```

**a2r 后端**:
```rust
// net.rs.at - 直接使用 tokio
use tokio::net::{TcpListener, TcpStream};

pub fn tcp_bind(addr: &str) -> Option<TcpListener> {
    // 同步包装
    tokio::runtime::Handle::current().block_on(async {
        TcpListener::bind(addr).await.ok()
    })
}
```

**测试**:
```auto
// tests/net_test.at
fn test_echo_server() {
    let listener = net.tcp_bind("127.0.0.1:9999")

    // 启动服务端
    async.spawn(fn() {
        let stream = listener.accept_async()
        let data = stream.read_all()
        stream.write(data)
        stream.close()
    })

    // 客户端连接测试...
}
```

**状态**: [ ] 未开始

---

### Phase 3: 数据处理模块 (Data Processing Modules)

**Goal**: 实现 JSON 和 URL 处理

#### 3.1 JSON 模块

**文件**:
```
stdlib/auto/
├── json.at             # Auto API 定义
├── json.vm.at          # VM 实现 (serde_json)
└── json.rs.at          # a2r 实现 (serde_json)
```

**API 定义** (json.at):
```auto
/// 将值编码为 JSON 字符串
fn json.encode(value T) str

/// 将 JSON 字符串解码为值
fn json.decode<T>(s str) T

/// 从字符串解析为 JsonValue
fn json.parse(s str) JsonValue?

/// 格式化 JSON（美化输出）
fn json.prettify(s str) str

// JsonValue 动态类型
type JsonValue

fn JsonValue.type() str
fn JsonValue.as_string() str
fn JsonValue.as_number() float
fn JsonValue.as_array() List<JsonValue>
fn JsonValue.get(key str) JsonValue?
```

**Rust 后端**:
```rust
// json.vm.at
use serde_json::{Value, to_string, from_str};

#[vm]
fn json_encode(value: &Value) -> String {
    to_string(value).unwrap()
}

#[vm]
fn json_decode(s: &str) -> Option<Value> {
    from_str(s).ok()
}

#[vm]
fn json_prettify(s: &str) -> String {
    let v: Value = from_str(s).ok()?;
    to_string_pretty(&v).unwrap()
}
```

**状态**: [ ] 未开始

---

#### 3.2 URL 模块

**文件**:
```
stdlib/auto/
├── url.at              # Auto API 定义
├── url.vm.at           # VM 实现
└── url.rs.at           # a2r 实现
```

**API 定义** (url.at):
```auto
/// URL 编码
fn url.encode(s str) str

/// URL 解码
fn url.decode(s str) str

/// 编码查询参数
fn url.encode_query(params Map<str, str>) str

/// 解码查询参数
fn url.decode_query(query str) Map<str, str>

/// 解析 URL
fn url.parse(s str) Url?
```

**Rust 后端**:
```rust
// url.vm.at
use urlencoding::{encode, decode};

#[vm]
fn url_encode(s: &str) -> String {
    encode(s).to_string()
}

#[vm]
fn url_decode(s: &str) -> String {
    decode(s).unwrap().to_string()
}

#[vm]
fn url_decode_query(query: &str) -> HashMap<String, String> {
    // 解析 query string
    url::form_urlencoded::parse(query.as_bytes())
        .into_owned()
        .collect()
}
```

**状态**: [ ] 未开始

---

### Phase 4: HTTP 模块 (HTTP Module)

**Goal**: 实现 HTTP Server 和 Client

#### 4.1 HTTP 模块

**文件**:
```
stdlib/auto/
├── http.at             # Auto API 定义
├── http.vm.at          # VM 实现 (hyper/axum)
└── http.rs.at          # a2r 实现 (hyper/axum)
```

**API 定义** (http.at):
```auto
// ═══════════════════════════════════════════════════════════
// HTTP Server
// ═══════════════════════════════════════════════════════════

type Server

/// 创建 HTTP Server
fn http.server() Server

/// 添加路由 GET
fn Server.get(path str, handler fn(Request) Response) Server

/// 添加路由 POST
fn Server.post(path str, handler fn(Request) Response) Server

/// 添加静态文件路由
fn Server.static(prefix str, dir str) Server

/// 启动监听（异步）
fn Server.listen(addr str) void

// ═══════════════════════════════════════════════════════════
// HTTP Request
// ═══════════════════════════════════════════════════════════

type Request

fn Request.method() str
fn Request.path() str
fn Request.query(key str) str
fn Request.header(key str) str
fn Request.body() []byte
fn Request.text() str
fn Request.json<T>() T

// ═══════════════════════════════════════════════════════════
// HTTP Response
// ═══════════════════════════════════════════════════════════

type Response

fn http.response() Response
fn Response.status(code int) Response
fn Response.header(key str, value str) Response
fn Response.text(body str) Response
fn Response.json(data T) Response
fn Response.html(body str) Response

// ═══════════════════════════════════════════════════════════
// 预定义响应
// ═══════════════════════════════════════════════════════════

fn http.ok(body str) Response
fn http.not_found(msg str) Response
fn http.internal_error(msg str) Response
```

**Rust 后端 (axum)**:
```rust
// http.vm.at
use axum::{
    Router, routing::{get, post},
    extract::{Request, Path, Query},
    response::{Response, IntoResponse},
    body::Body,
    http::{StatusCode, header},
};
use tower_http::cors::CorsLayer;

#[vm]
fn http_server() -> Server {
    Server(Router::new())
}

#[vm]
fn server_get(server: &mut Server, path: &str, handler_id: u64) {
    let h = move |req: Request| async move {
        // 调用 Auto 处理函数
        call_auto_handler(handler_id, req).await
    };
    server.0 = server.0.clone().route(path, get(h));
}

#[vm]
async fn server_listen(server: Server, addr: &str) {
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, server.0).await.unwrap();
}
```

**完整示例**:
```auto
// examples/http_server.at

use auto.http
use auto.json

type User = {
    id: int
    name: str
    email: str
}

let users = [
    User { id: 1, name: "Alice", email: "alice@example.com" },
    User { id: 2, name: "Bob", email: "bob@example.com" },
]

fn main() {
    let server = http.server()

    // GET /
    server.get("/", fn(req Request) Response {
        http.ok("Welcome to Auto HTTP Server!")
    })

    // GET /users
    server.get("/users", fn(req Request) Response {
        http.response().json(users)
    })

    // GET /users/:id
    server.get("/users/:id", fn(req Request) Response {
        let id = req.param("id").to_int()
        for user in users {
            if user.id == id {
                return http.response().json(user)
            }
        }
        http.not_found("User not found")
    })

    // POST /users
    server.post("/users", fn(req Request) Response {
        let user = req.json<User>()
        // 保存用户...
        http.response().status(201).json(user)
    })

    // 静态文件
    server.static("/static", "./public")

    // 启动服务器
    log.info("Server running on http://127.0.0.1:8080")
    server.listen("127.0.0.1:8080")
}
```

**状态**: [ ] 未开始

---

## 4. Implementation Timeline

| Phase | Module | Priority | Estimated Effort | Dependency |
|-------|--------|----------|------------------|------------|
| 1.1 | async (封装) | P0 | 1-2 days | Plan 069 (已完成) |
| 1.2 | log | P2 | 0.5 days | None |
| 1.3 | env | P2 | 0.5 days | None |
| 2.1 | net (TCP) | P0 | 3-5 days | Phase 1.1 |
| 3.1 | json | P1 | 2-3 days | None |
| 3.2 | url | P2 | 1-2 days | None |
| 4.1 | http (Server) | P1 | 5-7 days | Phase 2.1, 3.1, 3.2 |

**Total Estimated Effort**: 13-20 days

---

## 5. File Structure

```
stdlib/auto/
├── async.at            # Phase 1.1
├── async.vm.at
├── async.rs.at
│
├── log.at              # Phase 1.2
├── log.vm.at
├── log.rs.at
│
├── env.at              # Phase 1.3
├── env.vm.at
├── env.rs.at
│
├── net.at              # Phase 2.1
├── net.vm.at
├── net.rs.at
│
├── json.at             # Phase 3.1
├── json.vm.at
├── json.rs.at
│
├── url.at              # Phase 3.2
├── url.vm.at
├── url.rs.at
│
├── http.at             # Phase 4.1
├── http.vm.at
└── http.rs.at
```

---

## 6. Technical Decisions

### 6.1 HTTP 框架选择

| 选项 | 优点 | 缺点 | 推荐 |
|-----|------|------|------|
| **axum** | Tokio 生态，类型安全，性能好 | 学习曲线 | ✅ 推荐 |
| actix-web | 性能最高 | 更复杂 | 备选 |
| hyper | 底层控制 | 太底层 | 不推荐 |

### 6.2 JSON 库选择

| 选项 | 优点 | 缺点 | 推荐 |
|-----|------|------|------|
| **serde_json** | 标准，成熟 | - | ✅ 推荐 |
| simd-json | 更快 | 兼容性 | 备选 |

### 6.3 异步模型

基于 **Plan 069** 已实现的 Task/Msg 框架：
- 使用 `SPAWN` 创建异步任务
- 使用 `SEND/RECV` 进行任务间通讯
- 使用 `SLEEP` 实现超时
- VM FFI 函数标记为 `async` 自动挂起

---

## 7. Success Criteria

### Phase 1 验证
- [ ] `async.spawn()` 能创建任务
- [ ] `async.channel()` 能创建 Channel
- [ ] `log.info()` 能输出日志
- [ ] `env.args()` 能获取命令行参数

### Phase 2 验证
- [ ] `net.tcp_bind()` 能创建 TCP 监听器
- [ ] 能接受 TCP 连接
- [ ] 能读写 TCP 数据

### Phase 3 验证
- [ ] `json.encode()` 能编码对象
- [ ] `json.decode()` 能解码字符串
- [ ] `url.encode()` 能编码 URL

### Phase 4 验证
- [ ] HTTP Server 能启动
- [ ] GET/POST 路由能工作
- [ ] Request/Response 能正常解析和构造
- [ ] JSON API 能正常工作

### 集成验证
- [ ] 完整的 REST API 示例能运行
- [ ] 能用 curl 测试 API
- [ ] 并发请求能正确处理

---

## 8. Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| 异步 FFI 接口复杂 | 高 | 复用 Plan 069 的 async 机制 |
| HTTP 协议细节多 | 中 | 使用成熟的 axum 框架 |
| 类型映射困难 | 中 | 先实现基本类型，复杂类型延后 |
| 性能问题 | 低 | 先保证功能正确，后续优化 |

---

## 9. Status

### Phase 1-4: 标准库实现 ✅ 完成

- [x] Phase 1.1: async 模块封装
- [x] Phase 1.2: log 模块
- [x] Phase 1.3: env 模块
- [x] Phase 2.1: net 模块 (TCP)
- [x] Phase 3.1: json 模块
- [x] Phase 3.2: url 模块
- [x] Phase 4.1: http 模块

### Phase 5: a2vue 双模式支持 ⏳ 未开始

- [ ] Phase 5.1: API 注解解析
- [ ] Phase 5.2: 双模式编译开关
- [ ] Phase 5.3: 前端 API 生成
- [ ] Phase 5.4: 集成测试

**Implementation Notes**:

### FFI Pattern
- Simple functions use `#[rust_fn]` macro (Log, Json, Url, Http helpers)
- Handle-based resources use manual shims (Net TCP, HTTP Response)
- See CLAUDE.md for the `#[rust_fn]` macro guideline

### Native ID Ranges
| Range | Module | Functions |
|-------|--------|-----------|
| 1000-1099 | File | 10 |
| 1100-1199 | Env | 3 |
| 1200-1299 | Time | 3 |
| 1300-1399 | Process | 5 |
| 1400-1499 | Path | 5 |
| 1500-1599 | String | 10 |
| 1600-1699 | Char | 7 |
| 1700-1799 | Math | 4 |
| 1800-1899 | Log | 4 |
| 1900-1999 | JSON | 18 |
| 2000-2099 | URL | 16 |
| 2100-2199 | Net (TCP) | 14 |
| 2200-2299 | HTTP | 24 |

### Files Created
| Module | Auto API | VM Declaration |
|--------|----------|----------------|
| async | stdlib/auto/async.at | stdlib/auto/async.vm.at |
| log | stdlib/auto/log.at | stdlib/auto/log.vm.at |
| env | stdlib/auto/env.at | (uses Process FFI) |
| net | stdlib/auto/net.at | stdlib/auto/net.vm.at |
| json | stdlib/auto/json.at | stdlib/auto/json.vm.at |
| url | stdlib/auto/url.at | stdlib/auto/url.vm.at |
| http | stdlib/auto/http.at | stdlib/auto/http.vm.at |

### Technical Details
- **TCP**: Uses `std::net` with thread-local handle registry (`TCP_LISTENERS`, `TCP_STREAMS`)
- **HTTP Client**: Simple blocking implementation, parses URL manually
- **HTTP Server**: Placeholder (route handlers need callback support)
- **HTTP Response**: Thread-local `HTTP_RESPONSES` registry with handle-based access

### Future Work
- [ ] HTTP Server route matching (needs VM callback support)
- [ ] JSON generic decode (`decode<T>`)
- [ ] URL query Map handling
- [ ] HTTPS support

---

## Phase 5: a2vue 双模式支持 (Tauri + Web)

**Goal**: 基于 Phase 1-4 的标准库，实现 a2vue 的 Tauri 模式和 Web 模式统一支持

**Status**: [ ] 未开始
**Priority**: P1 (a2vue 核心能力)
**Dependency**: Phase 1-4 (已完成)

### 5.1 API 注解解析

**Goal**: 识别 `#[api]` 注解，自动生成后端路由代码

#### Auto API 定义示例

```auto
// api/user.at

/// 用户信息
type User = {
    id: int
    name: str
    email: str
}

/// 获取用户信息
#[api]
fn get_user(id int) User {
    db.find_user(id)
}

/// 保存文件
#[api]
fn save_file(path str, content str) void {
    fs.write(path, content)
}

/// 获取文件列表
#[api(method = "GET", path = "/files")
fn list_files(dir str) List<str> {
    fs.list_dir(dir)
}
```

#### API 注解属性

| 属性 | 说明 | 示例 |
|-----|------|------|
| `method` | HTTP 方法 | `#[api(method = "POST")]` |
| `path` | 自定义路径 | `#[api(path = "/users/:id")]` |
| `name` | 自定义函数名 | `#[api(name = "getUserById")]` |
| `auth` | 需要认证 | `#[api(auth = true)]` |
| `cache` | 缓存时间(秒) | `#[api(cache = 60)]` |

#### 编译器扩展

```
crates/auto-lang/src/api/
├── mod.rs              # API 编译器入口
├── parser.rs           # 解析 #[api] 注解
├── types.rs            # API 类型定义
└── targets/
    ├── mod.rs          # Target trait
    ├── tauri.rs        # Tauri 命令生成
    ├── axum.rs         # Axum 路由生成
    └── typescript.rs   # TypeScript 类型生成
```

**状态**: [ ] 未开始

---

### 5.2 双模式编译开关

**Goal**: 支持 `--target tauri` 和 `--target web` 编译选项

#### CLI 设计

```bash
# Tauri 模式（单机应用）
auto build --target tauri --out ./src-tauri/src/api

# Web 模式（HTTP Server）
auto build --target web --out ./server/src/api

# 同时生成两种模式
auto build --target all --out ./generated
```

#### 生成的文件结构

**Tauri 模式** (`--target tauri`):
```
generated/tauri/
├── commands.rs         # #[tauri::command] 函数
├── types.rs            # Rust 类型定义
└── mod.rs              # 命令注册
```

```rust
// commands.rs (自动生成)
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[tauri::command]
pub fn get_user(id: i32) -> User {
    api::get_user(id)
}

#[tauri::command]
pub fn save_file(path: String, content: String) {
    api::save_file(&path, &content)
}

// 注册函数
pub fn register_handlers(builder: tauri::Builder<tauri::Wry>) -> tauri::Builder<tauri::Wry> {
    builder.invoke_handler(tauri::generate_handler![
        get_user,
        save_file,
        list_files
    ])
}
```

**Web 模式** (`--target web`):
```
generated/web/
├── routes.rs           # axum 路由定义
├── handlers.rs         # 异步处理函数
├── types.rs            # Rust 类型定义
└── mod.rs              # 路由注册
```

```rust
// routes.rs (自动生成)
use axum::{
    Router, routing::{get, post},
    extract::{Path, Query, Json},
    response::{Json as JsonResponse},
};

pub fn api_routes() -> Router {
    Router::new()
        .route("/users/:id", get(get_user_handler))
        .route("/files", post(save_file_handler))
        .route("/files", get(list_files_handler))
}

async fn get_user_handler(Path(id): Path<i32>) -> JsonResponse<User> {
    JsonResponse(api::get_user(id))
}

async fn save_file_handler(Json(payload): Json<SaveFileRequest>) {
    api::save_file(&payload.path, &payload.content);
}
```

**状态**: [ ] 未开始

---

### 5.3 前端 API 生成

**Goal**: 自动生成 TypeScript API 层，支持 Tauri IPC 和 HTTP 两种通讯方式

#### 生成文件结构

```
generated/frontend/
├── types.ts            # TypeScript 类型定义
├── api-interface.ts    # API 接口定义
├── api-tauri.ts        # Tauri IPC 实现
├── api-http.ts         # HTTP 实现
└── api.ts              # 自动选择实现
```

#### types.ts (类型定义)

```typescript
// 自动生成
export interface User {
    id: number;
    name: string;
    email: string;
}

export interface SaveFileRequest {
    path: string;
    content: string;
}
```

#### api-interface.ts (接口定义)

```typescript
// 自动生成
import type { User } from './types';

export interface IApi {
    getUser(id: number): Promise<User>;
    saveFile(path: string, content: string): Promise<void>;
    listFiles(dir: string): Promise<string[]>;
}
```

#### api-tauri.ts (Tauri IPC 实现)

```typescript
// 自动生成
import { invoke } from '@tauri-apps/api/tauri';
import type { IApi, User } from './types';

export const tauriApi: IApi = {
    getUser: (id) => invoke<User>('get_user', { id }),
    saveFile: (path, content) => invoke<void>('save_file', { path, content }),
    listFiles: (dir) => invoke<string[]>('list_files', { dir }),
};
```

#### api-http.ts (HTTP 实现)

```typescript
// 自动生成
import axios from 'axios';
import type { IApi, User } from './types';

const BASE_URL = '/api';

export const httpApi: IApi = {
    getUser: async (id) => {
        const res = await axios.get<User>(`${BASE_URL}/users/${id}`);
        return res.data;
    },
    saveFile: async (path, content) => {
        await axios.post(`${BASE_URL}/files`, { path, content });
    },
    listFiles: async (dir) => {
        const res = await axios.get<string[]>(`${BASE_URL}/files`, { params: { dir } });
        return res.data;
    },
};
```

#### api.ts (自动选择)

```typescript
// 自动生成
import { tauriApi } from './api-tauri';
import { httpApi } from './api-http';
import type { IApi } from './api-interface';

// 自动检测运行环境
const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// 导出适配的 API
export const api: IApi = isTauri ? tauriApi : httpApi;

// 也可以显式导入
export { tauriApi, httpApi };
export type { IApi };
```

#### Vue 组件使用示例

```vue
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { api } from '@/api'

const user = ref<User | null>(null)

onMounted(async () => {
    // 自动使用正确的通讯方式
    // Tauri 模式: invoke('get_user', { id: 1 })
    // Web 模式: fetch('/api/users/1')
    user.value = await api.getUser(1)
})
</script>
```

**状态**: [ ] 未开始

---

### 5.4 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                    Auto 后端代码（只写一份）                      │
│                                                                 │
│  #[api]                                                         │
│  fn get_user(id int) User { ... }                              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    编译器 (auto build --target)                  │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              后端生成 (Rust)                             │   │
│  │                                                         │   │
│  │  --target tauri          --target web                   │   │
│  │  ┌──────────────┐        ┌──────────────┐              │   │
│  │  │ #[tauri::    │        │ axum Router  │              │   │
│  │  │  command]    │        │              │              │   │
│  │  │ fn get_user  │        │ GET /users/  │              │   │
│  │  └──────────────┘        └──────────────┘              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              前端生成 (TypeScript)                       │   │
│  │                                                         │   │
│  │  types.ts  →  类型定义                                  │   │
│  │  api-interface.ts  →  IApi 接口                         │   │
│  │  api-tauri.ts  →  Tauri IPC 实现                        │   │
│  │  api-http.ts  →  HTTP 实现                              │   │
│  │  api.ts  →  自动选择                                    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
         ┌────────────────────┴────────────────────┐
         │                                         │
         ▼                                         ▼
┌─────────────────────┐                  ┌─────────────────────┐
│  Tauri 单机应用      │                  │  Web 应用            │
│                     │                  │                     │
│  WebView (Vue)      │                  │  Browser (Vue)      │
│      │              │                  │      │              │
│      │ IPC          │                  │      │ HTTP         │
│      ▼              │                  │      ▼              │
│  Rust Backend       │                  │  Rust Server        │
│  (tauri command)    │                  │  (axum routes)      │
│                     │                  │                     │
└─────────────────────┘                  └─────────────────────┘
```

---

### 5.5 Phase 5 实施时间线

| 任务 | 预估工时 | 优先级 |
|-----|---------|--------|
| 5.1 API 注解解析 | 2-3 天 | P1 |
| 5.2 双模式编译开关 | 1-2 天 | P1 |
| 5.3 前端 API 生成 | 2-3 天 | P1 |
| 5.4 集成测试 | 1-2 天 | P1 |

**Total**: 6-10 天

---

### 5.6 Phase 5 验证标准

- [ ] `#[api]` 注解能被正确解析
- [ ] Tauri 模式生成 `#[tauri::command]` 代码
- [ ] Web 模式生成 axum 路由代码
- [ ] TypeScript 类型正确生成
- [ ] `api.ts` 能自动检测环境
- [ ] Tauri 应用能通过 IPC 调用后端
- [ ] Web 应用能通过 HTTP 调用后端
- [ ] 同一份 Vue 代码在两种模式下都能工作

---

## 10. References

- [docs/design/http-server-stdlib.md](../design/http-server-stdlib.md) - 完整 API 设计
- [docs/design/autovm-task-msg.md](../design/autovm-task-msg.md) - Task/Msg 架构原理
- [docs/plans/069-autovm-global-vars.md](069-autovm-global-vars.md) - Task/Msg 实现详情
- [docs/design/frontend-backend-communication.md](../design/frontend-backend-communication.md) - 前后端通讯设计
