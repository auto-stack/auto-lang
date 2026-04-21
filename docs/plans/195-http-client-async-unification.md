# Plan 195: HTTP Client 实现 + auto.http 统一 + 异步支持

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 实现完整的 HTTP Client（基于 reqwest），统一 auto.http 标准库 API，并为 HTTP Server/Client 添加 async 支持。

**Architecture:** 三层递进：先实现 HTTP Client FFI（替换手工 TCP 实现），再统一 http.at/http_stream.at/http.vm.at 的 API 设计，最后让 Server.listen 和 Client.send 支持 `~T` async 返回类型，通过 TaskSystem.run 桥接 tokio runtime。

**Tech Stack:** Rust (reqwest, tokio), AutoLang VM FFI, auto.http stdlib

---

## 现状分析

### 已有实现

| 模块 | 文件 | 状态 | 说明 |
|------|------|------|------|
| HTTP Client (简单) | `stdlib/auto/http.at` + `http.vm.at` | 声明已写 | API 完整但 Rust 实现是手工 TCP |
| HTTP Client (FFI) | `stdlib.rs:shim_http_get/post/put/delete` | 阻塞 | 用 `std::net::TcpStream` 手工拼 HTTP |
| HTTP Stream (SSE) | `stdlib/auto/http_stream.at` | 已实现 | 用 `reqwest::blocking` 做流式读取 |
| HTTP Server | `stdlib/auto/http.at` Server 部分 | 占位符 | `server_listen` 只打印日志 |
| RequestBuilder | `http.vm.at` 声明 | 未实现 | 无 Rust shim，ID 未分配 |
| async 系统 | `stdlib/auto/async.at` + Plan 124 | 已完成 | `~T`, `.await`, `TaskSystem.run` |

### 关键问题

1. **HTTP Client 是手工 TCP 实现** — 不支持 HTTPS、重定向、chunked encoding、连接池
2. **RequestBuilder 声明了但没实现** — NATIVE_HTTP_REQUEST (2234) 未分配
3. **HTTP Server 是占位符** — 需要路由匹配 + callback 机制
4. **HTTP Stream 和 HTTP Client 是两套独立实现** — 应该统一到 `auto.http` 下
5. **所有 HTTP 操作都是阻塞的** — 没有 async 支持

### Native ID 分配现状

```
2200-2206: HTTP Server (7 functions)
2210-2215: HTTP Response (6 functions)
2220-2224: Quick Response helpers (5 functions)
2230-2233: HTTP Client basic (4 functions)
2234-2239: 未分配 ← RequestBuilder
2240-2244: HTTP Stream (5 functions)
2255:      HTTP Stream with headers
```

---

## 设计决策

### 1. HTTP Client: reqwest 替换手工 TCP

**选择 reqwest::blocking 而非 reqwest::async**

原因：
- 当前 VM FFI 是同步模型（`fn shim_xxx(task, vm) -> Result`）
- TaskSystem.run 已提供 tokio runtime 桥接（Plan 124）
- 阻塞 API 更容易在 VM 中实现和测试
- 后续 Phase 3 async 支持可以无缝切换

### 2. 统一 auto.http 模块

将 `http.at`、`http_stream.at`、`http.vm.at` 合并为一个统一的 `auto.http` 模块：

```
auto.http
├── Server      (Phase 4 - 后续)
├── Client      (本计划)
├── Request     (本计划)
├── Response    (已有)
├── RequestBuilder (本计划)
└── HTTPStream  (已有，迁移)
```

### 3. Async 支持策略

利用已有的 `~T` + `.await` + `TaskSystem.run` 机制：

```auto
// 异步 HTTP 请求
TaskSystem.run(~{
    let res = http.get("https://api.example.com/data").await
    print(res.text())
})
```

实现方式：在 `TaskSystem.run` 内部调用 `tokio::runtime::Handle::block_on()` 执行 reqwest async 请求。

---

## Implementation Phases

### Phase 1: HTTP Client 基础 (reqwest::blocking)

**Goal:** 用 reqwest 替换手工 TCP 实现，支持 HTTPS、headers、timeout

#### Task 1.1: 升级 simple_http_request 为 reqwest

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs` (line 2249-2324)

**Step 1: 替换 simple_http_request 实现**

将当前的 `std::net::TcpStream` 手工实现替换为 `reqwest::blocking::Client`：

```rust
fn simple_http_request(method: &str, url: &str, body: Option<&str>) -> i64 {
    let client = reqwest::blocking::Client::new();
    let mut builder = match method {
        "GET" => client.get(url),
        "POST" => client.post(url),
        "PUT" => client.put(url),
        "DELETE" => client.delete(url),
        _ => client.get(url),
    };

    if let Some(b) = body {
        builder = builder.body(b.to_string());
    }

    match builder.send() {
        Ok(response) => {
            let status = response.status().as_u16();
            let headers: Vec<(String, String)> = response.headers().iter()
                .filter_map(|(k, v)| {
                    Some((k.to_string(), v.to_str().ok()?.to_string()))
                })
                .collect();
            let body_bytes = response.bytes().unwrap_or_default().to_vec();

            let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
            let resp_data = HttpResponseData { status, headers, body: body_bytes };
            HTTP_RESPONSES.with(|r| r.borrow_mut().insert(handle, resp_data));
            handle as i64
        }
        Err(e) => shim_http_internal_error(format!("HTTP {} failed: {}", method, e))
    }
}
```

**Step 2: 更新 Cargo.toml features**

确保 `reqwest` 启用 `blocking` feature（已有）。

**Step 3: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 现有测试通过

**Step 4: Commit**

```
feat(http): replace manual TCP with reqwest::blocking for HTTP client
```

#### Task 1.2: 实现 RequestBuilder

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs` (添加新的 native IDs 和 shims)

**Step 1: 分配 Native ID**

```rust
pub const NATIVE_HTTP_REQUEST: u16 = 2234;
pub const NATIVE_HTTP_REQUEST_BUILDER_HEADER: u16 = 2235;
pub const NATIVE_HTTP_REQUEST_BUILDER_BODY: u16 = 2236;
pub const NATIVE_HTTP_REQUEST_BUILDER_TIMEOUT: u16 = 2237;
pub const NATIVE_HTTP_REQUEST_BUILDER_JSON: u16 = 2238;
pub const NATIVE_HTTP_REQUEST_BUILDER_SEND: u16 = 2239;
```

**Step 2: 添加 RequestBuilder 数据结构和 thread-local 存储**

```rust
#[derive(Debug, Clone)]
struct HttpRequestBuilderData {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<String>,
    timeout_ms: Option<u64>,
}

thread_local! {
    static HTTP_REQUEST_BUILDERS: std::cell::RefCell<std::collections::HashMap<u64, HttpRequestBuilderData>> =
        std::cell::RefCell::new(std::collections::HashMap::new());
}
```

**Step 3: 实现 shim 函数**

```rust
// http_request(method, url) -> RequestBuilder handle
pub fn shim_http_request(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let url: String = VMConvertible::pop_from_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let method: String = VMConvertible::pop_from_stack(task, _vm).map_err(|e| VMError::RuntimeError(e.to_string()))?;

    let handle = NET_HANDLE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let data = HttpRequestBuilderData {
        method, url,
        headers: vec![],
        body: None,
        timeout_ms: None,
    };
    HTTP_REQUEST_BUILDERS.with(|b| b.borrow_mut().insert(handle, data));
    task.ram.push_i64(handle as i64);
    Ok(())
}

// request_builder_header(rb, key, value) -> rb
// request_builder_body(rb, body) -> rb
// request_builder_timeout(rb, ms) -> rb
// request_builder_json(rb, data) -> rb
// request_builder_send(rb) -> Response handle
```

**Step 4: 注册 native functions**

在 `register_all` 中添加注册。

**Step 5: 运行测试**

Run: `rtk cargo test -p auto-lang`
Expected: 通过

**Step 6: Commit**

```
feat(http): implement RequestBuilder FFI with headers, body, timeout support
```

#### Task 1.3: 增强 Response 访问方法

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs`

**Step 1: 添加 response_status_code 和 response_headers shim**

当前的 Response shim 只有 status/header/text/html/bytes。需要添加：

```rust
// response_status_code(res_handle) -> int
pub fn shim_response_status_code(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError>

// response_header_get(res_handle, key) -> str
pub fn shim_response_header_get(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError>

// response_body(res_handle) -> []byte
pub fn shim_response_body(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError>
```

分配 ID:
```rust
pub const NATIVE_RESPONSE_STATUS_CODE: u16 = 2216;
pub const NATIVE_RESPONSE_HEADER_GET: u16 = 2217;
pub const NATIVE_RESPONSE_BODY: u16 = 2218;
```

**Step 2: 更新 http.vm.at 添加声明**

**Step 3: 运行测试并 commit**

---

### Phase 2: 统一 auto.http 模块

**Goal:** 将 http.at、http_stream.at、http.vm.at 合并为统一模块

#### Task 2.1: 合并 http_stream.at 到 http.at

**Files:**
- Modify: `stdlib/auto/http.at` (添加 HTTPStream 类型和方法)
- Modify: `stdlib/auto/http.vm.at` (添加 HTTPStream VM 声明)
- Keep: `stdlib/auto/http_stream.at` (标记为 deprecated，re-export from http)

**Step 1: 在 http.at 中添加 HTTPStream 部分**

```auto
// ═══════════════════════════════════════════════════════════
// HTTP Stream (SSE)
// ═══════════════════════════════════════════════════════════

/// HTTP streaming response for SSE and chunked data
type HTTPStream

/// Create a streaming GET request
pub fn get_stream(url str) HTTPStream;

/// Create a streaming POST request
pub fn post_stream(url str, body str) HTTPStream;

/// Create a streaming POST request with custom headers
pub fn post_stream_with_headers(url str, body str, headers str) HTTPStream;

/// Read next chunk from stream
pub fn HTTPStream.next(self HTTPStream) str;

/// Check if stream is done (1 = done, 0 = active)
pub fn HTTPStream.is_done(self HTTPStream) int;

/// Close stream and release resources
pub fn HTTPStream.close(self HTTPStream) void;
```

**Step 2: 更新 http_stream.at 为 re-export**

```auto
/// DEPRECATED: Use `http` module directly instead.
/// This module re-exports HTTPStream types from http.
///
/// Migration guide:
///   use http_stream: { http_get_stream }  →  use http: { get_stream }
use http: { get_stream as http_get_stream, post_stream as http_post_stream,
            post_stream_with_headers as http_post_stream_with_headers }
```

**Step 3: Commit**

```
refactor(http): merge http_stream into unified auto.http module
```

#### Task 2.2: 更新 http.vm.at 统一声明

**Files:**
- Modify: `stdlib/auto/http.vm.at`

**Step 1: 添加缺失的 VM 声明**

添加 RequestBuilder 和 HTTPStream 的 VM 声明（之前分散在不同文件）：

```auto
// ═══════════════════════════════════════════════════════════
// Request Builder
// ═══════════════════════════════════════════════════════════

#[vm]
fn http_request(method str, url str) RequestBuilder

#[vm]
fn request_builder_header(rb RequestBuilder, key str, value str) RequestBuilder

#[vm]
fn request_builder_body(rb RequestBuilder, body str) RequestBuilder

#[vm]
fn request_builder_timeout(rb RequestBuilder, ms int) RequestBuilder

#[vm]
fn request_builder_json(rb RequestBuilder, data str) RequestBuilder

#[vm]
fn request_builder_send(rb RequestBuilder) Response

// ═══════════════════════════════════════════════════════════
// Response 访问方法
// ═══════════════════════════════════════════════════════════

#[vm]
fn response_status_code(res Response) int

#[vm]
fn response_header_get(res Response, key str) str

#[vm]
fn response_body(res Response) []byte

// ═══════════════════════════════════════════════════════════
// HTTP Stream
// ═══════════════════════════════════════════════════════════

#[vm]
fn http_get_stream(url str) HTTPStream

#[vm]
fn http_post_stream(url str, body str) HTTPStream

#[vm]
fn http_post_stream_with_headers(url str, body str, headers str) HTTPStream

#[vm]
fn http_stream_next(stream HTTPStream) str

#[vm]
fn http_stream_is_done(stream HTTPStream) int

#[vm]
fn http_stream_close(stream HTTPStream) void
```

**Step 2: Commit**

```
refactor(http): unify all HTTP VM declarations in http.vm.at
```

---

### Phase 3: Async HTTP 支持

> **BLOCKED — 需要完成以下前置实现后才能开始**

**Goal:** 让 HTTP Client/Server 操作支持 `~T` 异步模式，在 `TaskSystem.run(~{...})` 内部通过 tokio runtime 执行异步 I/O。

---

#### 前置条件分析

Phase 3 依赖以下 VM 基础设施，当前状态均为 **未完成**：

##### 前置 A: TaskSystem.run 同步→异步桥接

**当前状态:** 空壳（`shim_task_system_run` 直接返回 `Ok(0)`，stdlib.rs:2736）

**需要实现的功能：**
1. `shim_task_system_run(future_id)` 接收到 VM 传来的 `future_id`（即 `CREATE_FUTURE` 创建的 `FutureValue`）
2. 从 `AutoVM.futures`（`DashMap<u32, Arc<RwLock<FutureValue>>>`）中取出 future
3. 创建（或复用）一个 `tokio::runtime::Runtime`
4. 在该 runtime 上 `block_on` 执行 future 的 `body_offset` 对应的 bytecode
5. 将执行结果写回 `FutureValue.result`，设置 `FutureState::Ready`
6. 返回结果值

**核心难点：** `shim_task_system_run` 是 `#[rust_fn]` 宏标记的同步函数，签名为 `fn(future_id: i64) -> Result<i64, String>`，无法直接访问 `AutoVM` 实例。需要新增机制让 FFI shim 能访问 VM 的 `futures` 注册表，或者重构 `TaskSystem.run` 为手动 shim（非 `#[rust_fn]`）。

**涉及文件：**
- `crates/auto-lang/src/vm/ffi/stdlib.rs:2736` — `shim_task_system_run`
- `crates/auto-lang/src/vm/engine.rs:133` — `AutoVM.futures` 注册表
- `crates/auto-lang/src/vm/engine.rs:142-158` — `FutureValue` / `FutureState` 定义

##### 前置 B: AWAIT_FUTURE 真正执行 async body

**当前状态:** 半成品（engine.rs:3403-3477）

`AWAIT_FUTURE` 当前处理 `FutureState::Pending` 时的逻辑（line 3438-3467）：
- 保存 `task.ip`
- 跳转到 `body_offset`
- **但没有真正执行 bytecode**（注释写着 "In real implementation, we'd run the bytecode interpreter here"）
- 直接将 future 标记为 Ready，结果为 `Int(0)`（占位符）

**需要实现的功能：**
1. 当 `AWAIT_FUTURE` 遇到 `Pending` 状态时，需要递归调用 VM 执行引擎来执行 async body 的 bytecode
2. 或者更好的方案：将 async body 的执行交给一个独立的 VM 实例或执行上下文
3. 执行完成后将结果写入 `FutureValue.result`

**核心难点：** 这要求将 `AutoVM.execute()`（主循环）重构为可重入的，或者提取出一个 `execute_bytecode_range(vm, task, start_ip, end_ip)` 函数。当前的主执行循环在 `AutoVM` 的 `impl` 中，没有独立出来。

**涉及文件：**
- `crates/auto-lang/src/vm/engine.rs:3403-3477` — `AWAIT_FUTURE` opcode 处理
- `crates/auto-lang/src/vm/engine.rs` — VM 主执行循环（需要重构为可重入）

##### 前置 C: Async FFI（native 函数内执行异步操作）

**当前状态:** 不存在

当前所有 FFI shim 都是同步函数：`fn(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError>`。要支持 async FFI（比如在 VM 中调用 `http.get_async(url)` 时内部执行 `reqwest::Client::get().await`），需要：

**需要实现的功能：**
1. 新增 `AWAIT_EXT` opcode（scheduler.rs:201-204 已预留了注释）
2. 当 native function 需要执行异步操作时，返回一个 "pending" 状态而非立即返回
3. VM 检测到 pending 状态后，挂起当前 task，切换到其他 task
4. 当 async 操作完成时，恢复挂起的 task

**替代方案（简化版）：** 不引入 `AWAIT_EXT`，而是在 async FFI shim 中直接创建 `tokio::runtime::Runtime` 并 `block_on`。这样虽然不是真正的非阻塞，但功能上可以工作（代价是每个 async 调用创建一个 runtime，且会阻塞当前线程）。

**涉及文件：**
- `crates/auto-lang/src/vm/opcode.rs` — 新增 `AWAIT_EXT` opcode
- `crates/auto-lang/src/vm/engine.rs` — 处理 `AWAIT_EXT`（挂起/恢复 task）
- `crates/auto-lang/src/vm/scheduler.rs:201-204` — 已预留的注释位置

##### 前置 D: scheduler 的 execute_handler_fully 完善

**当前状态:** 骨架（scheduler.rs:173-219）

`execute_handler_fully` 是 scheduler 中的 async handler 执行器，当前只处理 `RET`、`HALT`、`NOP` 三个 opcode，其他全部跳过（line 206-209: "For now, skip unknown opcodes"）。

**需要实现的功能：**
1. 在 `execute_handler_fully` 中支持所有必要的 opcode（至少包括 NATIVE_CALL、PUSH、POP、LOAD、STORE 等）
2. 或者更好的方案：复用 `engine.rs` 的主执行循环，将 `execute_handler_fully` 改为调用 VM 的通用执行函数

**核心难点：** `execute_handler_fully` 需要访问 `NativeInterface` 来执行 native calls，而当前它只接收 `GlobalMeta`（包含 `native_interface`），但执行 opcode 需要完整的 `AutoVM` 上下文。

**涉及文件：**
- `crates/auto-lang/src/vm/scheduler.rs:173-219` — `execute_handler_fully`

---

#### 前置条件总结

| 前置 | 描述 | 复杂度 | 建议形式 |
|------|------|--------|----------|
| **A** | TaskSystem.run 桥接 tokio runtime | 中 | 本 Plan 的 Task 3.1 |
| **B** | AWAIT_FUTURE 真正执行 async body | 高 | **单独 Plan 196** |
| **C** | Async FFI（AWAIT_EXT 或 block_on 简化版） | 高 | **单独 Plan 196** |
| **D** | scheduler execute_handler_fully 完善 | 高 | **单独 Plan 196** |

**依赖链：**
```
Phase 3 (本 Plan)
├── Task 3.1: HTTP Server 基础 listen（无需 async）
│   └── 直接用 tokio::runtime::Runtime::block_on，阻塞式实现
│
└── Task 3.2: Async HTTP Client/Server
    ├── 前置 A: TaskSystem.run 桥接 ← 本 Plan 可做
    ├── 前置 B: AWAIT_FUTURE 执行 ← Plan 196
    ├── 前置 C: Async FFI 机制    ← Plan 196
    └── 前置 D: scheduler 完善     ← Plan 196
```

---

#### Task 3.1: HTTP Server 基础 listen（阻塞式，无 async 依赖）

> 此 Task 不依赖前置 A-D，可直接实现。

**Files:**
- Modify: `crates/auto-lang/src/vm/ffi/stdlib.rs` (shim_http_server_listen, line 1707)

**Step 1: 实现阻塞式 HTTP server**

使用 `tokio::runtime::Runtime::block_on` + TCP accept loop，不需要 VM async 支持：

```rust
pub fn shim_http_server_listen(task: &mut AutoTask, _vm: &AutoVM) -> Result<(), VMError> {
    let addr: String = VMConvertible::pop_from_stack(task, _vm)
        .map_err(|e| VMError::RuntimeError(e.to_string()))?;
    let _server_handle: i64 = task.ram.pop_i64();

    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| VMError::RuntimeError(format!("Failed to create runtime: {}", e)))?;

    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| VMError::RuntimeError(format!("Bind failed: {}", e)))?;
        eprintln!("[HTTP] Server listening on {}", addr);

        loop {
            match listener.accept().await {
                Ok((mut stream, _)) => {
                    tokio::spawn(async move {
                        let mut buf = vec![0u8; 4096];
                        if let Ok(n) = stream.read(&mut buf).await {
                            if n > 0 {
                                let response = "HTTP/1.1 200 OK\r\n\
                                    Content-Type: text/plain\r\n\
                                    Content-Length: 27\r\n\
                                    Connection: close\r\n\
                                    \r\n\
                                    Hello from Auto HTTP Server";
                                let _ = stream.write_all(response.as_bytes()).await;
                            }
                        }
                    });
                }
                Err(e) => eprintln!("[HTTP] Accept error: {}", e),
            }
        }
    });
    Ok(())
}
```

**Step 2: 运行测试并 commit**

```
feat(http): implement HTTP server listen with tokio TCP accept loop
```

---

#### Task 3.2: Async HTTP Client/Server（BLOCKED）

> **此 Task 被前置 A-D 阻塞，需要先完成 Plan 196 后才能实施。**

以下为完成前置后的实现设计，仅供参考：

**目标 API：**
```auto
// 异步 HTTP 请求（需要 Plan 196 完成后）
TaskSystem.run(~{
    let res = http.get("https://api.example.com/data").await
    print(res.text())
})

// 异步 HTTP Server 路由回调（需要 Plan 196 完成后）
let server = http.server()
server.get("/users", fn(req Request) Response {
    http.ok("Hello")
})
server.listen("127.0.0.1:8080")  // 非阻塞，在 TaskSystem.run 内运行
```

**需要的 Native IDs（预分配，暂不实现）：**
```rust
pub const NATIVE_HTTP_ASYNC_GET: u16 = 2260;
pub const NATIVE_HTTP_ASYNC_POST: u16 = 2261;
pub const NATIVE_HTTP_ASYNC_SEND: u16 = 2262;
```

**Plan 196 建议标题：** "VM Async Runtime: TaskSystem.run 桥接 + AWAIT_FUTURE 完善 + Async FFI"

---

## File Changes Summary

```
Modified:
├── crates/auto-lang/src/vm/ffi/stdlib.rs
│   ├── simple_http_request() → reqwest::blocking (Task 1.1)
│   ├── HttpRequestBuilderData + shims (Task 1.2)
│   ├── Response access shims (Task 1.3)
│   └── server_listen tokio 实现 (Task 3.1)
│
├── stdlib/auto/http.at
│   └── HTTPStream types (Task 2.1)
│
├── stdlib/auto/http.vm.at
│   ├── RequestBuilder declarations (Task 2.2)
│   ├── HTTPStream declarations (Task 2.2)
│   └── Response access declarations (Task 2.2)
│
└── stdlib/auto/http_stream.at
    └── Deprecated → re-export from http (Task 2.1)

Blocked (需要 Plan 196):
├── crates/auto-lang/src/vm/ffi/stdlib.rs
│   └── Async HTTP shims (Task 3.2)
└── stdlib/auto/http.at
    └── Async HTTP declarations (Task 3.2)
```

---

## Native ID Allocation

| Range | Module | Functions | Phase |
|-------|--------|-----------|-------|
| 2216-2218 | Response access | 3 (status_code, header_get, body) | 1.3 |
| 2234-2239 | RequestBuilder | 6 (new, header, body, timeout, json, send) | 1.2 |
| 2260-2262 | Async HTTP | 3 (async_get, async_post, async_send) | 3.2 (BLOCKED) |

---

## Timeline

| Phase | Task | Estimated Effort | Status |
|-------|------|------------------|--------|
| 1.1 | reqwest 替换 | 1-2h | Ready |
| 1.2 | RequestBuilder | 2-3h | Ready |
| 1.3 | Response 增强 | 1h | Ready |
| 2.1 | 合并 http_stream | 1h | Ready |
| 2.2 | 统一 VM 声明 | 1h | Ready |
| 3.1 | Server listen（阻塞式） | 2-3h | Ready |
| 3.2 | Async HTTP client/server | 4-6h | **BLOCKED** (需 Plan 196) |
| **Phase 1+2+3.1** | | **8-11h** | 可立即开始 |
| **Phase 3.2** | | **4-6h** | 需 Plan 196 完成 |

---

## Success Criteria

### Phase 1 验证
- [ ] `http.get("https://httpbin.org/get")` 返回正确 Response
- [ ] `http.post(url, body)` 支持 JSON body
- [ ] `http.request("GET", url).header("Auth", "token").send()` 链式调用
- [ ] `http.get("https://...")` 支持 HTTPS（手工实现不支持）

### Phase 2 验证
- [ ] `use http: { get_stream }` 正常工作
- [ ] `http_stream.at` 的 re-export 兼容旧代码
- [ ] 所有 HTTP VM 声明集中在 `http.vm.at`

### Phase 3 验证
- [ ] `server.listen("127.0.0.1:8080")` 启动并响应请求（阻塞式）
- [ ] curl 能访问 server

### Phase 3.2 验证 (BLOCKED — 需要 Plan 196)
- [ ] `TaskSystem.run(~{ let r = http.get_async(url).await })` 正常执行
- [ ] 异步 server 在 TaskSystem.run 内非阻塞运行
- [ ] 多个并发 async HTTP 请求不互相阻塞

---

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| Phase 3.2 被 Plan 196 阻塞 | High | Phase 1+2+3.1 可独立交付，不等待 |
| reqwest::blocking 在已有 tokio runtime 中调用死锁 | Medium | Phase 1 使用独立 runtime；Phase 3.2 改用 reqwest async |
| HTTP Server 路由匹配复杂 | Medium | Task 3.1 先实现简单 echo，路由匹配作为后续 Plan |
| http_stream.at 废弃影响现有代码 | Low | 使用 re-export 保持兼容 |
| `#[rust_fn]` 宏无法访问 AutoVM | Medium | TaskSystem.run 改为手动 shim（Plan 196 范围） |

---

## References

- [Plan 102](102-http-server-stdlib.md) — HTTP Server 原始设计
- [Plan 124](124-async-future-await.md) — ~T + .await + TaskSystem.run
- [Plan 127](127-autovm-task-system-execution.md) — scheduler + message dispatch
- [Plan 152](152-streaming-http-sse.md) — 流式 HTTP / SSE
- [Plan 154](154-real-http-streaming.md) — 真正的 HTTP 流式实现（reqwest::blocking）
- [docs/design/http-server-stdlib.md](../design/http-server-stdlib.md) — HTTP 设计文档

## 关联 Plan

- **Plan 196** (待创建): VM Async Runtime — TaskSystem.run 桥接 + AWAIT_FUTURE 完善 + Async FFI + scheduler execute_handler_fully
  - 完成后解除本 Plan Phase 3.2 的阻塞
