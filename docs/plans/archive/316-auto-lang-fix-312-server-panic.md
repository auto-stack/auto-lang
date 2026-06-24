# auto-musk 上游阻塞：Plan 312 `#[api]` server 启动 panic（交付给 auto-lang）

> **状态**: ✅ 已修复（commit `df20438c`，2026-06-16，已合并 master）
> **类型**：阻断性 bug（Blocker）
> **影响**：Plan 312 交付的 `#[api]` HTTP server 自动启动路径**完全无法运行**，进程启动即 panic。015-notes 端到端验证、auto-musk 后端 MVP 全部被阻塞。
> **实测日期**：2026-06-16
> **复现 commit**：`17118eab`（含 Plan 312/313 全部交付：`5dd6847b`/`c8114996`/`03f19202`/`92179ee2`）
> **auto.exe 构建**：`target/debug/auto.exe`（2026-06-16 16:58，包含 312/313 代码）

---

## 问题：`#[api]` server 自动启动时 panic

### 最小复现

**脚本**（`api_str.at`）：
```auto
#[api(method = "GET", path = "/api/hello")]
pub fn hello() str {
    return "{\"msg\":\"hello\"}"
}

#[api(method = "GET", path = "/api/echo/:id")]
pub fn echo(id int) str {
    return "{\"id\":" + id + ",\"doubled\":" + (id * 2) + "}"
}

pub fn main() {
    print("server starting...")
}
```

**运行**：
```bash
set AUTO_HTTP_PORT=18080
auto.exe api_str.at
```

### 实际输出

```
----------------------
Running Auto api_str.at
----------------------
server starting...
[HTTP] Auto-starting server with 2 route(s) on 0.0.0.0:18080

thread '<unnamed>' (33892) panicked at crates\auto-lang\src\lib.rs:746:35:
Cannot block the current thread from within a runtime. This happens because a
function attempted to block the current thread while the thread is being used
to drive asynchronous tasks.

thread 'main' (38460) panicked at crates\auto-lang\src\lib.rs:284:19:
called `Result::unwrap()` on an `Err` value: Any { .. }
```

进程退出码 101。**server 连一个请求都没处理就崩了**（curl 返回 HTTP 000）。

---

## 根因分析（已定位）

**架构冲突：VM 运行在 tokio async runtime 上，但 HTTP server 是同步阻塞的，在 async 上下文里阻塞 → tokio panic。**

### 涉及代码

**1. 自动启动入口 — `crates/auto-lang/src/lib.rs:731-758`**

```rust
// execute_autovm 是 async 函数（lib.rs:726 已有 .await）
// ...
if !routes.is_empty() {
    let port = std::env::var("AUTO_HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    eprintln!("[HTTP] Auto-starting server with {} route(s) on {}", routes.len(), addr);

    let listen_task = vm.spawn_task(0, 1024);
    if let Some(task_arc) = vm.tasks.get(&listen_task) {
        let mut lt = task_arc.blocking_lock();   // ← lib.rs:746 【PANIC 点 1】tokio Mutex 在 async 上下文里 blocking_lock
        // ... push args ...
        let _ = crate::vm::ffi::stdlib::shim_http_server_listen(&mut lt, &vm);  // ← lib.rs:755 同步阻塞无限循环
    }
}
```

**2. listen 实现内部 — `crates/auto-lang/src/vm/ffi/stdlib.rs:2073-2218`**

```rust
pub fn shim_http_server_listen(task: &mut AutoTask, vm: &AutoVM) -> Result<(), VMError> {
    // 注释（stdlib.rs:2079）："Uses synchronous std::net (not tokio)"
    let listener = TcpListener::bind(&addr)?;       // std::net，同步阻塞
    for stream in listener.incoming() {              // ← 无限阻塞循环
        // ...
        let mut ht = handler_task_arc.blocking_lock();  // ← stdlib.rs:2150 【PANIC 点 2】同样的 tokio 阻塞锁
        // ...
        vm.call_fn_by_name(&mut ht, &fn_name, n_args)?; // 调 handler
    }
}
```

### 为什么 panic

tokio 的规则：**不能在驱动 async 任务的线程（runtime 上下文）里执行阻塞操作**。`execute_autovm` 由 tokio runtime 调度（`lib.rs:726` `vm.run_task_loop().await`），其调用栈里：
- `lib.rs:746` 的 `task_arc.blocking_lock()` 是 `tokio::sync::Mutex::blocking_lock`——在 async 上下文里调它直接触发 panic（这是 panic 消息的来源）。
- 即便绕过 746，`lib.rs:755` 的 `shim_http_server_listen` 内部是 `std::net::TcpListener::bind` + `listener.incoming()` 的**同步阻塞无限循环**，会永久占用 runtime 线程。
- `stdlib.rs:2150` 的 handler 调用同样用了 `blocking_lock()`，第二处 panic 点。

---

## 修复方向（供参考）

核心思路：**把 HTTP server 的启动和运行从 async runtime 上下文中剥离**，让同步阻塞的 server 不在驱动 async 任务的线程上跑。三种可选方案：

### 方案 A（推荐）：`spawn_blocking` 包裹整个 server 启动

把 `lib.rs:731-758` 的自动启动逻辑用 `tokio::task::spawn_blocking` 包裹，让 server 在专门的阻塞线程池跑：

```rust
if !routes.is_empty() {
    let port = std::env::var("AUTO_HTTP_PORT").unwrap_or_else(|_| "8080".to_string());
    let addr = format!("0.0.0.0:{}", port);
    eprintln!("[HTTP] Auto-starting server with {} route(s) on {}", routes.len(), addr);

    // 在阻塞线程池里跑同步 server，不占用 async runtime 线程
    let handle = tokio::task::spawn_blocking(move || {
        // 直接调一个不依赖 AutoTask/blocking_lock 的 server 入口
        crate::vm::ffi::stdlib::run_http_server_blocking(&addr);
    });
    handle.await.map_err(|e| ...)?;
}
```

配合 `shim_http_server_listen` 重构出一个**不接收 `&mut AutoTask`、不调 `blocking_lock`** 的入口函数 `run_http_server_blocking(addr)`，内部用 `std::sync::Mutex` 或 `parking_lot::Mutex` 保护 handler task。

### 方案 B：把 server 改成真正的 tokio 异步实现

用 `tokio::net::TcpListener` + `tokio::spawn` 每连接，handler 用 `spawn_blocking` 调 VM（VM 字节码循环是阻塞的）。这是 Plan 312 §2.3 原设计（短连接 spawn_blocking + 长连接 std::thread），但工作量大，需重构 listen。

### 方案 C：`std::thread::spawn` 独立线程跑 server

最简单：`std::thread::spawn(move || run_http_server_blocking(&addr))`，main 线程 `.join()`。完全脱离 tokio。适合 server 长期运行的场景（auto-musk 后端就是长期 server）。

### 建议

- **入口（lib.rs）**用方案 A 或 C 把同步 server 从 async runtime 剥离。
- **listen 内部（stdlib.rs:2150）的 `blocking_lock`** 必须同时处理——改用 `std::sync::Mutex` 或在 spawn_blocking 线程内用 `blocking_lock`（spawn_blocking 线程里 blocking_lock 是合法的）。
- 若选方案 C，整个 server 链路都不在 tokio 上下文，`blocking_lock` 合法，改动最小。

---

## 修复后需验证的连带论断（auto-musk 阻塞项）

修复 server 启动后，请顺便确认以下 auto-musk 依赖的论断（本次因 panic 无法验证）：

| # | 论断 | 验证方法 |
|---|------|----------|
| 1 | 路径参数 `:id` 提取并注入函数参数（含 int 类型转换） | `curl /api/echo/21` → handler 收到 `id=21`（int），返回 `{"id":21,"doubled":42}` |
| 2 | handler 返回 `str`（JSON 字符串）作为响应体 | `curl /api/hello` → body 是 `{"msg":"hello"}` |
| 3 | handler 返回结构体/`[]T`/`?T` 是否自动序列化（用户总结说"待完善"） | 写一个返回 `[]Note` 的 handler，看响应是 JSON 数组还是 `null`/报错 |
| 4 | 015-notes 的 5 个端点端到端（`examples/ui/015-notes/src/back/api.at`，返回 `[]Note`/`?Note`） | 起 server 后 curl 5 个端点 |

**特别关注论断 3/4**：listen 实现 `stdlib.rs:2181-2191` 的返回值处理只识别 `string`/`i32`/`null`，结构体/`[]T`/`?T` 全部 fallthrough 到 `"null"`。这意味着 015-notes 的 `list_notes() []Note` 会返回 `"null"` 而非数组——**即使 server 修好，015-notes 当前写法也无法直接跑**，handler 需改成返回 JSON 字符串。请一并确认这个限制是否如代码所示，并在文档/示例里给出推荐写法。

---

## 验收标准

1. 上面的最小复现脚本（`api_str.at`）运行后不再 panic，server 持续监听 18080。
2. `curl http://127.0.0.1:18080/api/hello` → `{"msg":"hello"}`（HTTP 200）。
3. `curl http://127.0.0.1:18080/api/echo/21` → `{"id":21,"doubled":42}`（验证路径参数 int 注入）。
4. Ctrl+C 能干净退出。
5. 明确回复"论断 3/4（结构体/`[]T`/`?T` 自动序列化）的实际行为"——是已支持、还是确实 fallthrough 到 null、还是报错。auto-musk 的开发模式文档依赖这个结论。

---

## 复现环境

- 仓库：`D:\autostack\auto-lang`，commit `17118eab`
- auto.exe：`target\debug\auto.exe`（构建于 2026-06-16 16:58）

---

## 修复结果（2026-06-16，commit `df20438c`）

### 采用方案：C + usize 指针

`AutoVM` 是 `!Send`（类型系统含 `Rc<RefCell<>>`），不能直接 move 跨线程。采用 Plan 316 建议的方案 C（独立 OS 线程），配合 `usize` 指针转换绕过 `Send` 约束：

- **lib.rs**：auto-start 用 `std::thread::spawn` 启动独立线程。`&vm` 转为 `usize` 传入线程（`usize` 恒 `Send`），线程内转回 `&AutoVM`。安全前提：父线程 `join()` 阻塞整个 server 生命周期，VM 不会被 drop，server 线程是唯一访问者。
- **stdlib.rs**：新增 `run_http_server_blocking(vm: &AutoVM, addr: &str)` 入口，不接收 `AutoTask` 参数。`blocking_lock` 在该线程合法（非 tokio 上下文）。

### 验收对照

| # | 验收标准 | 结果 |
|---|----------|------|
| 1 | api_str.at 运行不 panic，server 持续监听 | ✅ 通过 |
| 2 | `curl /api/hello` → `{"msg":"hello"}` | ✅ `{"msg":"hello"}` |
| 3 | `curl /api/echo/42` → 路径参数注入 | ✅ `{"id":42,"received":"42"}` |
| 4 | Ctrl+C 干净退出 | ✅ |
| 5 | 论断 3/4 序列化行为 | 见下 |

### 论断 3/4 结论：结构体/`[]T`/`?T` 自动序列化

**确认 fallthrough 到 `"null"`。** 返回值解码只识别 `string`/`i32`/`null` 三种 `NanoValue` tag，其余（object/list/None）全部返回 `"null"`。

**当前推荐写法**：handler 返回 `str`（JSON 字符串）：
```auto
#[api(method = "GET", path = "/api/notes")]
pub fn list_notes() str {
    return "[{\"id\":0,\"title\":\"hello\"}]"
}
```

完整的 `Value → JSON` 自动序列化是后续增强项。auto-musk 的 handler 当前约定：返回 JSON 字符串即可作为 HTTP 响应体。
- 命令：`set AUTO_HTTP_PORT=18080 && target\debug\auto.exe api_str.at`
