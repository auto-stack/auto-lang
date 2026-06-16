# Plan 312: AutoVM 一等 #[api] 路由 + 可用 HTTP Server

> **Status**: Draft
> **依赖**: 无前置依赖
> **关联**: Plan 313(TCP flush + SSE server)依赖本计划的 HTTP server 请求处理流程
> **验收样本**: `examples/ui/015-notes/src/back/api.at`(5 个标准 CRUD #[api] 端点)+ `db.at` + `pac.at`

---

## §1 背景与动机

`#[api(method, path)]` 当前只在**代码生成阶段**被消费(转译为 Axum/Rust server 或 Tauri/Vue 客户端),AutoVM 运行时不认识。auto-musk 项目要求用 AutoVM 脚本模式跑全栈后端,需让 `#[api]` 成为 VM 一等运行时能力:**含 `#[api]` 的 `.at` 文件用 AutoVM 直接运行,自动起 HTTP server,函数注册为路由,分发请求,注入路径参数,JSON 序列化响应。**

### 1.1 当前状态(已验证)

| 组件 | 现状 | 证据 |
|---|---|---|
| **解析器拒绝 `#[api]`** | `parse_fn_annotations` 白名单无 `api`,走 `_ =>` 返回 SyntaxError | `parser.rs:6588-6593` |
| **`FnAnnotations` 无 api 字段** | 只有 has_c/has_vm/has_rs/has_pub/has_test/store_attrs/with_params | `parser.rs:162-173` |
| **`Fn` AST 结构无注解字段** | is_pub/is_test 是布尔标志,注解 payload 丢失 | `ast/fun.rs:17-33` |
| **AST 提取是空壳** | `extract_endpoint` 不检测 `#[api]`,无条件捕获所有函数 | `api/mod.rs:135-157` |
| **正则 fallback 是唯一真实提取** | 绕过解析器,文本匹配 `#[api(...)]` | `api_gen.rs:789-852` |
| **分发无 vm 分支** | 只分发给 tauri/vue,Axum 代码生成存在但不可达 | `api_gen.rs:79-89` |
| **http_server_listen 不分发** | tokio+spawn 骨架存在,但硬编码 "Hello" 响应 | `stdlib.rs:2019-2065` |
| **get/post/put/delete 丢弃 handler** | 弹出 handler 不存储 | `stdlib.rs:1965-2008` |
| **TCP/Task 基础设施可用** | Net.tcp_*(同步 std net);Task.spawn(scheduler) | `stdlib.rs:1627-1871` |

### 1.2 关键差距清单

1. **解析器层**:必须让 `#[api(method, path)]` 合法,且 AST 保留 method/path。
2. **VM 路由表**:模块加载时扫描 `#[api]` 函数,建路由表 `{(method, path_pattern) → handler}`。
3. **HTTP server native**:listen 循环实现请求解析→路由匹配→调 handler→序列化响应。
4. **【核心难点】VM 函数作异步 HTTP handler 回调**:tokio 上下文里重新进入 VM 执行某函数并取返回值。当前无此 primitive。
5. **参数/响应自动序列化**:`:id` 注入参数;body JSON 解析;返回值 JSON 化。
6. **启动方式**:含 `#[api]` 的 `.at` 如何自动起 server。

---

## §2 核心难点论证:VM 函数作异步 HTTP handler 回调

这是决定整个方案成立与否的关键。**先论证设计,再编码。**

### 2.1 问题本质

tokio 的 async handler 是 `Future<Output = Response>`,在 tokio runtime 线程上 poll。VM 函数是**阻塞式字节码循环**(`BVMState::run` 里的 `loop { match opcode }`)。两者有阻抗:

- tokio handler 不能直接调 `vm.run()` —— 会阻塞 executor 线程(尤其 SSE 长连接)。
- VM 的 state(`BVMState` 含栈/堆/PC)不是 `Send` 的(含 `Rc<RefCell<>>`),不能跨 tokio task 传递。

### 2.2 方案对比

| 方案 | 做法 | 优势 | 风险 |
|---|---|---|---|
| **A. 独立 OS 线程** | listen 循环 accept 后,`std::thread::spawn` 调 VM(每连接一个 VM 实例或 clone) | 简单,VM 阻塞不影响 tokio | 线程开销;VM state 隔离需 clone;并发数受线程限制 |
| **B. tokio::task::spawn_blocking** | 用 spawn_blocking 包裹 VM 调用 | tokio 原生管理线程池 | spawn_blocking 适合短任务,长连接(SSE)会占满池 |
| **C. VM Task scheduler 集成** | 复用 `scheduler.rs:spawn_dynamic_task`,让每个 HTTP 请求成为一个 VM Task | 与现有并发模型统一 | scheduler 是面向 mailbox 的,需改造;复杂度高 |

### 2.3 推荐方案:A(独立 OS 线程,短连接)+ B(spawn_blocking)分场景

- **短连接(普通 CRUD)**:用 `tokio::task::spawn_blocking` 包裹 VM 函数调用。VM state 从路由表持有的"模板 state"clone 出来(每请求独立)。handler 返回后序列化为 Response,写回连接。
- **长连接(SSE,Plan 313)**:用 `std::thread::spawn`(独立 OS 线程),避免占满 spawn_blocking 池。

这个分场景策略让 Plan 312(短连接 CRUD)和 Plan 313(长连接 SSE)各得其所。

### 2.4 VM 可调用引用设计

需要一个"VM 函数句柄"类型,记录:
- 函数的 `FragId`(VM 内部函数地址)或函数名。
- 所属的模块/Database 引用。
- 参数签名(用于 JSON body → 参数映射)。

在 listen 循环的 handler dispatch 里:
1. 从路由表查到 `FragId`。
2. clone 一个 VM state(`BVMState::clone` 或新建子 frame)。
3. push 参数到栈(从路径参数 + JSON body 解析)。
4. `spawn_blocking` 执行 `vm.run_until_return()`。
5. 取返回值,JSON 序列化,写 HTTP response。

---

## §3 分阶段实施计划

### Phase 1 — 解析器 + AST:让 `#[api]` 合法且可提取(P0)

**目标**:解析器接受 `#[api(method, path)]`,AST 保留 method/path,VM 加载时可扫描。

**改动**:

| 文件 | 改动 |
|---|---|
| `parser.rs:162-173` | `FnAnnotations` 新增 `api_endpoints: Vec<ApiEndpoint>` 字段 |
| `parser.rs:6506-6587` | 白名单新增 `"api"` arm,解析 `method = "GET", path = "/..."` |
| `ast/fun.rs:17-33` | `Fn` 结构体新增 `api_attrs: Option<ApiAttrs>` 字段(method/path/auth) |
| `api/mod.rs:135-157` | `extract_endpoint` 改为从 `fn.api_attrs` 读取(不再无条件捕获) |

**`ApiAttrs` 结构**(新定义,放 `ast/fun.rs` 或 `api/mod.rs`):
```rust
pub struct ApiAttrs {
    pub method: String,       // GET/POST/PUT/DELETE
    pub path: String,         // "/api/notes/:id"
    pub auth: Option<String>, // 可选
}
```

**验收**:`examples/ui/015-notes/src/back/api.at` 能被 parser 成功解析(不再 SyntaxError);AST 里 5 个函数各有 `api_attrs`。

### Phase 2 — VM 路由表 + 函数句柄注册(P0)

**目标**:VM 模块加载时扫描 `#[api]` 函数,建路由表。

**改动**:

| 组件 | 改动 |
|---|---|
| VM 加载逻辑 | 模块编译后,遍历 AST 的 `Stmt::Fn`,对有 `api_attrs` 的建 `RouteEntry { method, path_pattern, frag_id, param_names }` |
| 路由表存储 | 全局 `HttpRouter { routes: Vec<RouteEntry> }`,存入 VM 的 Database 或 thread-local |
| 路径参数提取 | `:id` 模式匹配:把 `/api/notes/:id` 编译为 regex 或分段匹配;提取 `{id: "42"}` |
| `http.server()` 改造 | 创建 server 对象时,从全局路由表注入(不再只返回计数器) |
| `server_get/post/...` 改造 | 从参数取 (path, handler) 改为从路由表查;handler 存为 `FragId`(不再丢弃) |

**验收**:加载 api.at 后,路由表有 5 条记录,每条含正确的 method/path/frag_id。

### Phase 3 — HTTP server listen 请求分发(P0,核心)

**目标**:`http.server_listen` 实现真实请求分发。

**改动 `shim_http_server_listen`**(`stdlib.rs:2019-2065`):

1. **请求解析**:读 request line(`GET /api/notes/42 HTTP/1.1`)+ headers(Content-Length 等)+ body。
2. **路由匹配**:查路由表,找到 `(method, path)` 匹配的 handler。
3. **参数注入**:
   - 路径参数:`:id` → `id: 42`(int 类型转换)。
   - Body 参数:POST/PUT 的 JSON body → 函数参数(单结构体→整体;多参数→JSON 对象字段)。
4. **handler 调用**(见 §2.4):`spawn_blocking` → VM 函数执行 → 取返回值。
5. **响应序列化**:返回值 JSON 化([]T→array, ?T→null or object, bool→true/false);None/Err→合适状态码(404/500)。

**并发**:每请求 `spawn_blocking` 独立 VM state(从模板 clone)。

**验收**:`examples/ui/015-notes/src/back/api.at` 的 5 个端点,curl 全打通:
- `GET /api/notes` → 200 JSON array
- `GET /api/notes/:id` → 200 JSON or 404 null
- `POST /api/notes` → 201 created
- `PUT /api/notes/:id` → 200 updated
- `DELETE /api/notes/:id` → 200 true

### Phase 4 — 启动方式 + 零改动验收(P0)

**目标**:含 `#[api]` 的 `.at` 文件如何起 server。

**方案**:`auto run api.at` 时,VM 检测到模块含 `#[api]` 函数 → 自动创建 server + 注册路由 + listen(无需用户显式写 `http.server_listen`)。用户也可显式调用 `http.server_listen(port)` 覆盖端口。

**验收**:015-notes api.at 尽量零改动可跑;附一个最小 VM HTTP server 标准写法示例。

---

## §4 设计决策(待确认)

### 4.1 VM state 隔离策略

每个 HTTP 请求的 handler 调用需要独立的 VM state(避免并发污染)。两种选择:

| 策略 | 做法 | 风险 |
|---|---|---|
| **A. 完整 clone** | 每请求 `BVMState::clone`(栈/堆/PC 全复制) | 内存开销;但最安全 |
| **B. 子 frame** | 只 push 新 frame(共享全局堆,独立栈) | 更高效;但全局可变状态(db)需同步 |

**推荐 B**(子 frame + 共享全局堆),因为 db 操作需要共享状态。但需加锁或用 `Arc<Mutex>` 保护全局可变状态。

### 4.2 阻塞字节码循环 vs tokio

确认用 `spawn_blocking`(Phase 3 短连接)。如果 VM 函数里有 `Task.spawn` 或 await,需要 VM 支持在 spawn_blocking 线程内 yield —— 这可能需要额外设计。Phase 3 先假设 handler 是同步的(不 spawn task)。

---

## §5 风险与缓解

| # | 风险 | 缓解 |
|---|---|---|
| 1 | **VM state 非 Send**(`Rc<RefCell>`)| spawn_blocking 闭包捕获的不是 VM state 本身,而是 clone 后的独立 state;或用 `Arc<Mutex<>>` 重构(大改) |
| 2 | **handler 并发写全局状态** | 015-notes 的 db 是内存 map → 需要 `Arc<Mutex<HashMap>>`;或限制单线程串行处理(Phase 3 MVP) |
| 3 | **JSON 序列化覆盖不全** | AutoVM 有 auto_val → JSON 的转换?需验证 `[]T/?T/struct` 的 JSON 化路径 |
| 4 | **tcp_stream_read_line 丢字节** | Plan 313 修复 BufReader;Plan 312 的 listen 用自己的请求解析(不依赖 read_line) |

---

## §6 验收标准

1. `#[api(method, path)]` 在解析器中合法,AST 保留 method/path
2. VM 加载含 `#[api]` 的模块后,路由表正确建立
3. `http.server_listen` 实现真实请求解析 + 路由匹配 + handler 调用 + 响应序列化
4. 015-notes api.at 的 5 个端点 curl 全打通
5. api.at 尽量零改动(允许加一行显式 listen 调用)
6. 附最小 VM HTTP server 标准写法示例

---

## 附录:015-notes api.at 端点清单

| Method | Path | Handler | 返回值 | 测试 |
|---|---|---|---|---|
| GET | `/api/notes` | `list_notes()` | `[]Note` | curl → JSON array |
| GET | `/api/notes/:id` | `get_note(id int)` | `?Note` | curl :id → JSON or 404 null |
| POST | `/api/notes` | `create_note(title str, body str)` | `Note` | curl POST → 201 created |
| PUT | `/api/notes/:id` | `update_note(id int, title str, body str)` | `?Note` | curl PUT → 200 updated |
| DELETE | `/api/notes/:id` | `delete_note(id int)` | `bool` | curl DELETE → 200 true |
