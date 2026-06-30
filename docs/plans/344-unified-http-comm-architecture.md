# Plan 344: 统一 HTTP 通讯架构（同步/异步 × 流式/非流式，VM + a2r 双平台）

> **状态**：设计文档 / TODO（未实现）
> **前置**：Plan 340（VM+VM 分离 HTTP）、Plan 341（异步 SSE 客户端）
> **阻塞**：本计划的"异步非流式"部分依赖 VM 核心变更（AWAIT_FUTURE 真挂起）

## 背景与动机

### 当前现状（截至 Plan 341）

Auto 语言的 HTTP 通讯能力是分阶段拼凑出来的，存在以下缺口：

| 维度 | 同步 | 异步 |
|------|------|------|
| **非流式**（普通 GET/POST） | ✅ VM（`simple_http_request`，spawn+join 阻塞）<br>✅ a2r（`spawn_blocking`） | ❌ **缺口**：native 不能 yield，阻塞冻结 UI |
| **流式**（SSE） | ❌ 缺口 | ✅ VM 客户端（Plan 341，独立线程+channel）<br>✅ VM 服务端（`serve_async`，spawn_local+yield） |

### 核心矛盾（Plan 341 发现）

native shim 是同步签名 `fn(task, vm) -> Result`，运行在 `execute_task → execute_single_frame`（同步函数）内部，后者又从 `run_task_loop`（async fn）调用。

native **不能在内部 yield** 给 tokio runtime。当它需要等异步操作完成时：

- **阻塞线程**：`GLOBAL_RT` 单 worker，阻塞会卡死所有 task；若异步 future 在同 runtime → 死锁
- **非阻塞轮询返回 pending**：调用方 VM 代码不知道要重试；无"挂起 task 稍后恢复"机制

**根因**：`run_task_loop` 只认两种唤醒源——`wake_time`（sleep）和 `task_mailboxes`（actor 消息）。没有第三种：外部 future 完成。

### 目标

把 HTTP 通讯的四个维度做成**正交特性**，且在 **VM 和 a2r 两个平台**上都实现：

```
            同步              异步
非流式   sync-request      async-request   （单次请求/响应）
流式     sync-stream       async-stream    （SSE / 分块传输）
```

## 设计：VM 侧

### 第 1 层：VM 核心变更 —— 外部 Future 唤醒（解锁所有异步能力）

这是整个架构的基石。基础设施已**部分存在**：

- `AWAIT_FUTURE` opcode（engine.rs，opcode.rs:0xC1）
- `FutureValue` 结构 + `futures: DashMap<u32, Arc<RwLock<FutureValue>>>`（engine.rs:266）
- `StepResult::AwaitFuture { future_id, body_offset }`（engine.rs:192）
- `execute_future_body`（engine.rs:6380）——但目前是"同步内联执行 body 字节码"，不是"等待外部 future"

**需要改动的**：

1. **FutureValue 增加"外部 future"形态**：新增字段 `external_result: Arc<Mutex<Option<Result<Value>>>>`，用于外部 tokio task 回填结果。区分"内部 body 字节码 future"（现有）和"外部 future"（新增）。

2. **native 注册外部 future + 挂起**：
   - native 创建 FutureValue（external），分配 future_id
   - `tokio::spawn`（或独立线程）跑异步工作，完成后填 `external_result` + 设 Ready
   - native 返回 future_id（编码为 `(id << 8) | 0xF0`），VM 字节码执行 `AWAIT_FUTURE`
   - **关键**：native 本身不等待，立即返回；等待逻辑在 `AWAIT_FUTURE` opcode 里

3. **AWAIT_FUTURE 的 Pending 分支改造**（engine.rs:6255）：
   - 外部 future Pending 时，**不再内联执行 body_offset**
   - 改为：设 `task.status = TaskStatus::Waiting("future:<id>")`，记录等待的 future_id，`return Ok(StepResult::Yield)`
   - 由 `run_task_loop` 在 future Ready 后唤醒

4. **run_task_loop 新增 future-wake 唤醒分支**（engine.rs:1236 附近，仿照 wake_time/mailbox）：
   ```
   for task in tasks {
       if task.status == Waiting("future:<id>") {
           if futures[id].state == Ready {
               task.status = Ready  // 下轮调度时 AWAIT_FUTURE 取到 Ready 分支结果
           }
       }
   }
   ```

5. **（可选优化）tokio::sync::Notify 即时唤醒**：替代 10ms 轮询。外部 future 完成后 `notify.notify_waiters()`，`run_task_loop` 的 `sleep(10ms)` 换成 `notify.notified().await` 的 select。需处理 `AutoVM: !Send` 边界（`Notify: Send + Sync`，但 VM 在单 worker 上跑，OK）。

**完成第 1 层后**，所有 native 都能"发起异步操作 + 挂起 task + 完成后恢复"，四个维度全部解锁。

### 第 2 层：四象限 native API（VM 侧）

统一命名规范：`http.{sync|async}.{request|stream}_{method}`

| 维度 | native | 行为 |
|------|--------|------|
| **同步非流式** | `http.sync.get(url)` | spawn OS 线程 + join 阻塞（现有 `simple_http_request`） |
| **异步非流式** | `http.async.get(url) -> future_id` | 创建外部 FutureValue + tokio::spawn，返回 future_id；AWAIT_FUTURE 挂起/恢复 |
| **同步流式** | `http.sync.sse_stream(url) -> iterator` | 独立线程 + 阻塞 read（现有 `HttpStreamIterator`） |
| **异步流式** | `http.async.sse_stream(url) -> iterator` | 独立线程 + channel + try_recv（Plan 341 现状）；或外部 future 逐帧驱动 |

**语法层面**（Auto 语言）：
- 同步：`let resp = http.get(url)` 或 `for ev in http.sse_stream(url)`
- 异步：`let resp = await http.get(url)` 或 `for await ev in http.sse_stream(url)`
- 需要 `await`/`for await` 语法支持（或编译器据上下文推断）

### 第 3 层：SSE 服务端统一

现有 `serve_async` 已支持异步 SSE 服务端（generator yield + yield_now）。统一为：
- `http.serve_sse(path, handler)` —— handler 是 generator function
- 已可用，无需大改

## 设计：a2r 侧

a2r 把 Auto 代码转译成 Rust。Rust 原生支持 async/await，所以四象限映射更直接：

| Auto（VM native） | a2r 转译目标（Rust） |
|---|---|
| `http.sync.get` | `reqwest::blocking::get` |
| `http.async.get` | `reqwest::get(...).await`（async fn） |
| `http.sync.sse_stream` | `reqwest::blocking` + 手动 read 循环 |
| `http.async.sse_stream` | `reqwest::async` + `bytes_stream()` + `while let Some(chunk)` |

**a2r 需要**：
- async fn 转译（`async fn`、`.await`）
- `for await` → `while let Some(...) = stream.next().await`
- 与 VM 侧 native **同名**（`http.async.get` 等），保证一份 .at 源码两个平台都能跑

## 实施路线（分阶段）

### 阶段 A：VM 核心 —— 外部 Future 唤醒（最高优先级，解锁异步非流式）

1. FutureValue 增加 external 形态
2. AWAIT_FUTURE Pending 分支：外部 future → 挂起 task
3. run_task_loop future-wake 唤醒分支
4. 单测：一个 mock 异步操作，task 挂起 → 完成 → 恢复

### 阶段 B：异步非流式 native（依赖 A）

1. `http.async.{get,post,put,delete}(url[,body]) -> future_id`
2. native 内 tokio::spawn + 注册外部 future，返回 future_id
3. codegen：`await http.async.get(url)` → AWAIT_FUTURE
4. 修复 Plan 340 codegen 改写：分离模式用 `http.async.*` + AWAIT_FUTURE（不再阻塞 UI）

### 阶段 C：统一四象限 native 命名 + 旧名兼容

1. 新增 `http.sync.*` / `http.async.*` 统一名
2. 旧名（`auto.http.get_json` 等）保留为别名，指向 sync 变体
3. 文档：推荐用法

### 阶段 D：a2r 四象限转译

1. async fn / await 转译
2. for await → while let stream.next().await
3. `http.async.sse_stream` → reqwest bytes_stream 循环
4. 跨平台单测：同一份 .at，VM 和 a2r 输出一致

### 阶段 E（可选）：tokio::sync::Notify 即时唤醒

替代 10ms 轮询，降低延迟。处理 Send 边界。

## 验证矩阵

每个阶段完成后，用以下矩阵验证（所有组合都要有测试）：

```
              VM 客户端          a2r 客户端
              ↓                  ↓
VM 服务端     sync/async ×       sync/async ×
              流式/非流式         流式/非流式

              a2r 服务端
              ↓
同上
```

最小可用验证：015-notes 的 `list_notes`（非流式）+ 一个 SSE demo（流式），四个组合 × 两个平台。

## 关键文件

- `crates/auto-lang/src/vm/engine.rs` — FutureValue、AWAIT_FUTURE、run_task_loop（阶段 A 核心）
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — 四象限 native
- `crates/auto-lang/src/vm/ffi/http_server.rs` — SSE 服务端（已可用）
- `crates/auto-lang/src/vm/codegen.rs` — await / for await 转译
- `crates/auto-lang/src/trans/rust.rs` — a2r 四象限转译
- `crates/auto-lang/src/ast/` — `await` / `for await` AST 节点（新增）

## 与现有计划的关系

- **Plan 340**：VM+VM 分离 HTTP（同步非流式）—— 已实现，是本计划"同步非流式"象限
- **Plan 341**：异步 SSE 客户端（异步流式）—— 已实现，是本计划"异步流式"象限的 VM 客户端
- **本计划 344**：补齐剩余两象限（异步非流式、同步流式）+ 统一命名 + a2r 双平台 + VM 核心变更
