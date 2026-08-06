---
plan: 394
title: await-future-external-architecture
affects: [auto-lang/vm/engine, auto-lang/vm/task, auto-lang/vm/codegen]
status: draft # draft | in-progress | complete
---

# Plan 394: AWAIT_FUTURE 通用 future 架构 —— 外部异步源挂起/恢复（Plan 344 正统延续）

> **状态**：立项调研 / 设计文档（未实施）。承接 Plan 344（archive, TODO）的第 1 层「VM 核心
> 变更——外部 Future 唤醒」。Plan 349 步骤 7 已用 re-entry yield 范式 + 全局表收敛把所有
> HTTP/IO native 异步化，这是 Plan 344 的务实替代；本计划是**长远最优方案**，当业务真需要
> 「用户态 async block 组合多个 native async 源」或 `Future.all/race` 时启动。
>
> **目标定位**：不做妥协方案。如果做，就做成支持「`~{}` async block 内任意嵌套 await 外部
> future」的完整架构，而不是「仅顶层 await」的半成品限制。

## §1 背景与动机

### 1.1 当前已落地的异步能力（Plan 349）

所有 HTTP/IO native（`http.get/post/put/delete`、`*_json`、`RequestBuilder.send`、
`post_sync/post_bearer/get_sync`、`io.read_text_async` 等）经 **re-entry yield 范式**异步化：

- native 首次调用 spawn OS 线程 + 设 `task.waiting_http_request_id` + yield（CALL_NAT IP 回退）。
- `run_task_loop` wake source 5 轮询统一的 `ASYNC_RESULTS` 表（Plan 349 收敛 phase，6 表→2 表）。
- 结果就绪 → task Ready → CALL_NAT 重入 → native re-entry 分支取结果（`AsyncResult` enum）。

**这套范式解决了「单次 native async 调用不阻塞 UI」的核心问题**，且每加一种新 async 操作的
边际成本低（一个 shim + 复用 `ASYNC_RESULTS`，约 60 行，不碰 engine.rs 核心）。

### 1.2 re-entry yield 范式的能力边界（本计划的动机）

re-entry yield 有两个根本限制，正是本计划要突破的：

1. **一个 task 只有一个等待槽**（`waiting_http_request_id: Option<u64>`）。无法在同一个 task
   里并发等待多个异步源（如 `Future.all([http.get(url_a), http.get(url_b)])`）。
2. **`~{}` async block 内不能 await 外部异步源**。`~{}` body 由 `execute_future_body` 同步
   内联执行（借用同一 task 的 IP/栈/帧），body 内一旦 await 外部 future，`execute_future_body`
   没有挂起能力（它没有「暂停 body、让出 run_task_loop、稍后恢复」的机制）。

**因此**，用户无法写出这样的代码：

```auto
// 组合多个 async 源，并发执行
let combined = ~{
    let a = http.get_async(url_a).await   // 外部 async
    let b = http.get_async(url_b).await   // 外部 async
    merge(a, b)                            // 纯 AutoLang 计算
}
let result = combined.await
```

也无法写 `Future.all` / `Future.race` 这类并发原语。

### 1.3 Plan 344 原设计

Plan 344（`docs/plans/archive/344-unified-http-comm-architecture.md`）提出了正统架构：

- `FutureValue` 加 `external_result` 字段，区分「内部 body future」与「外部 future」。
- native 创建 external FutureValue + tokio::spawn 跑异步工作 + 返回 future_id；VM 跑 `AWAIT_FUTURE`。
- `AWAIT_FUTURE` Pending 分支不再内联 body，改为挂起 task + `run_task_loop` future-wake 唤醒。

Plan 344 标注「TODO 未实现」。Plan 349 的 re-entry yield 是其绕过 future 系统的替代实现。

## §2 核心技术难点（调研已确认）

本计划的复杂度不在「加字段加分支」，而在以下两个硬骨头。调研（2026-08-06 双路 Explore agent）
已精确确认。

### 2.1 execute_future_body 的「同步内联」语义

**当前实现**（`engine.rs:7294-7369`，约 75 行）：

`execute_future_body(task, future_id, body_offset, depth, max_depth)` 的语义是「用同一个 task、
同一个栈、同一个帧同步跑 body 字节码」：

1. 保存 `task.ip`，跳到 `body_offset`。
2. 循环调 `execute_single_frame(task, BODY_BUDGET=10_000)`：
   - body 跑完（Return）→ 标 Ready + 回填 result + 恢复 IP。
   - body 内嵌套 await（`FrameResult::AwaitFuture`）→ **递归调 `execute_future_body`**（depth+1），
     靠 Rust 函数调用栈实现嵌套。
   - Yielded/BudgetExhausted → continue（body 太长分批跑）。
3. 恢复 `task.ip`。

**关键**：整个 future 从 Pending 到 Ready 在**一个 execute_task tick 内**完成。`execute_task`
（`engine.rs:7371-7392`）捕获 `FrameResult::AwaitFuture` 后立即同步调 `handle_await_future`，
返回 `TaskStatus::Ready`——`run_task_loop` 永远看不到一个「await 中」的 future 任务。

**这意味着**：当前 future 系统**从来没有真正挂起过 task**。所谓「await 一个 future」=「同步
跳进 body 跑完跳回来」。嵌套 await 靠 Rust 递归栈，最深 `MAX_RECURSION_DEPTH = 64`。

### 2.2 难点：body 内嵌套外部 future 需要续点（continuation）

这是整个改造的复杂度天花板。

设想 `~{ let a = http.get_async(url).await; use(a) }`：

1. VM 进入 `execute_future_body` 跑 body。
2. body 执行到 `http.get_async(url).await` —— 这是一个**外部 future**（pending，需等网络）。
3. 此时 body 的执行状态是：IP 停在 await 之后的指令、栈上有 body 的局部变量帧。
4. 要真挂起，必须**保存 body 的续点**（IP + 栈帧布局），让出 `run_task_loop`。
5. 外部 future 完成后，`run_task_loop` 唤醒 task，**从续点恢复** body 执行（恢复 IP + 栈帧），
   把 await 的结果压栈，继续跑 `use(a)`。

**当前架构完全没有续点机制**。AutoTask 没有保存「被挂起的 body 的 IP/栈帧」的地方。task 的
`ip`/`bp`/`ram` 在同一时刻只反映一条执行线。如果 body 中途挂起，调用者（await 的外层）的
IP/栈帧和 body 的混在一起，无法分离恢复。

### 2.3 难点：AWAIT_FUTURE 重入的栈语义

外部 future 唤醒后，`AWAIT_FUTURE` 需要被**重入执行**（第二次走 Ready 分支取结果）。但当前
`AWAIT_FUTURE` 第一次执行时已经 `pop_i32()` 把 future_bits 弹掉（engine.rs:7059）。要么：

- (a) 像 CALL_NAT 那样 rewind IP（但要在 pop **之前**检测外部 Pending 并退出——重排 opcode
      处理逻辑），或
- (b) 在 task 上缓存 future_id，wake 后跳过「假 pop」。

这与 re-entry yield 当年踩的 wake source 5「误清 waiting_http_request_id → Stack Underflow」
是同类栈错位风险。

## §3 长远最优方案设计（不妥协）

本方案的目标：**完整支持 `~{}` async block 内任意嵌套 await 外部 future**，不设「仅顶层
await」的限制。核心是给 AutoTask 引入续点机制。

### 3.1 续点机制：AsyncFrame 栈

给 AutoTask 加一个**独立的 async 续点栈**，与正常的 call_stack 分离：

```rust
// task.rs 新增
pub struct AsyncFrame {
    /// 被挂起的 body 的续点 IP（await 之后的下一条指令）
    pub resume_ip: usize,
    /// 被挂起时 body 的 bp（栈帧基址）
    pub resume_bp: usize,
    /// 这个 await 对应的 future_id（挂起原因）
    pub future_id: u32,
    /// body 的 captures closure_id（恢复时重装）
    pub closure_id: u32,
}

pub struct AutoTask {
    // ... 现有字段 ...
    /// Plan 394: async body 续点栈。挂起外部 future 时 push，恢复时 pop。
    /// 空表示当前不在 async body 挂起态。
    pub async_frames: Vec<AsyncFrame>,
    /// Plan 394: 当前挂起等待的外部 future（单 future await；多 future 见 3.4）
    pub waiting_future_id: Option<u32>,
}
```

**挂起时**：body 内 await 外部 future（Pending）→ push AsyncFrame{resume_ip, resume_bp, ...}
到 `async_frames`，设 `waiting_future_id`，`task.status = Waiting("future")`，return Yield。
注意：`resume_ip` 是 await 之后的指令地址——body 的局部变量已经在栈上，**不要动栈**。

**恢复时**：`run_task_loop` future-wake 检测 future Ready → `task.status = Ready`。重新调度时，
execute_task 发现 `async_frames` 非空 → 不从正常 IP 继续，而是 pop AsyncFrame，恢复
`resume_ip`/`resume_bp`，把 future 结果压栈，继续跑 body。

### 3.2 FutureValue 二态分叉

```rust
pub struct FutureValue {
    pub body_offset: u32,                // 内部 future 用；外部 future 为 0
    pub state: FutureState,
    pub result: Option<auto_val::Value>,
    pub owner_task_id: TaskId,
    pub captures: HashMap<String, auto_val::Value>,
    // Plan 394 新增：
    pub kind: FutureKind,
}

pub enum FutureKind {
    /// 内部 future：body_offset 字节码，execute_future_body 同步内联（保持现状）。
    Internal,
    /// 外部 future：由 native spawn 的异步源回填。
    External {
        /// 外部源完成时写这里（Arc<Mutex> 跨线程共享）。
        result_slot: Arc<std::sync::Mutex<Option<Result<auto_val::Value, String>>>>,
    },
}
```

`AWAIT_FUTURE` 入口按 `kind` 分叉：Internal 走现有 `execute_future_body`（不变）；External 走
新的挂起/恢复路径（3.1）。

### 3.3 native 注册外部 future 的 API

给 AutoVM 加方法，让 native 能注册外部 future（替代直接写 `ASYNC_RESULTS` 表）：

```rust
impl AutoVM {
    /// native 调用：分配一个 external future，返回 future_id。
    /// native 的 worker 线程完成后调 resolve_external_future 回填。
    pub fn register_external_future(&self, owner_task: TaskId) -> u32 { ... }

    /// worker 线程完成时调用（跨线程安全）。
    pub fn resolve_external_future(&self, future_id: u32, result: Result<Value, String>) { ... }
}
```

native shim 改写（以 http.get_async 为例）：

```rust
pub fn shim_http_get_async(task, vm) {
    // 首次调用：pop url，注册 external future，spawn worker，push future_id，return。
    let url = pop_from_stack(task, vm)?;
    let fid = vm.register_external_future(task.id);
    let slot = vm.futures.get(&fid).unwrap().read().unwrap().external_slot();
    std::thread::spawn(move || {
        let result = simple_http(&url);
        vm.resolve_external_future(fid, Ok(result.into()));  // 或用 slot 直接写
    });
    push_future_id(task, fid);  // 编码 (fid << 8) | 0xF0
}

// codegen: let f = http.get_async(url)   → CALL_NAT (push future_id)
//          let r = f.await               → AWAIT_FUTURE (外部，挂起/恢复)
```

### 3.4 多 future 并发（Future.all/race）

续点机制 + External future 天然支持多 future：

```rust
// 多 future 等待：task 记录一组 pending futures，全部/任一 ready 后恢复。
pub struct AutoTask {
    pub waiting_future_ids: Vec<u32>,   // 替代 waiting_future_id（或并存）
    pub join_mode: FutureJoinMode,      // All | Race | Single
}
```

`Future.all([f1, f2])` 语义：注册一个「组合 future」，其 result_slot 在 f1、f2 都 ready 后
回填 `[r1, r2]`。`run_task_loop` future-wake 仍只查组合 future 的 state。

### 3.5 run_task_loop future-wake（wake source 6）

```rust
// run_task_loop 新增（仿 wake source 4/5）：
if let Some(fid) = task.waiting_future_id {
    let ready = self.futures.get(&fid)
        .and_then(|f| f.read().ok().map(|fv| fv.state != FutureState::Pending))
        .unwrap_or(true);  // future 消失 → 唤醒（报错兜底）
    if ready {
        task.waiting_future_id = None;
        task.status = TaskStatus::Ready;
    } else {
        alive_count += 1;
        continue;
    }
}
```

### 3.6 与现有 re-entry yield 的关系

**共存，非替换**。re-entry yield（`ASYNC_RESULTS` + `waiting_http_request_id`）继续服务「单次
native async 不需组合」的简单场景（低成本、已验证）。External future 服务「需组合/嵌套」的
场景。native 可以二选一：简单 native 用 re-entry yield，需 `await` 语法的 native 提供一个
`_async` 变体注册 external future。

长期看，可逐步把 re-entry yield native 迁到 external future（统一架构），但这是可选的渐进
清理，不阻塞本计划。

## §4 触发条件（什么时候启动本计划）

满足以下**任一**条件时启动：

1. **产品需求**：用户需要写 `~{ ... .await }` 组合多个 async 源（如同时请求多个 API 合并
   结果），或需要 `Future.all/race` 并发原语。
2. **多 future 并发**：单个 task 需并发等待多个异步操作（re-entry yield 的单等待槽成为瓶颈）。
3. **async/await 语法正式立项**：Auto 语言决定把 `async fn` / `.await` 作为一等公民语法
   支持（a2r 侧也要 async fn 转译）。

**当前（2026-08-06）判断**：以上条件均未出现。re-entry yield + 表收敛已满足现有 HTTP/IO 异步
需求。本计划**暂不启动**，留档待触发。

## §5 实施路线（触发后分阶段）

### Phase A — 续点机制 + 顶层外部 future（中等复杂度）

1. AutoTask 加 `async_frames: Vec<AsyncFrame>` + `waiting_future_id`。
2. `FutureValue` 加 `kind: FutureKind`（Internal/External）。
3. `AWAIT_FUTURE` 入口按 kind 分叉：Internal 走现状；External Pending → push AsyncFrame +
   挂起 + Yield；External Ready → 取结果压栈。
4. `execute_task` 检测 `async_frames` 非空时从续点恢复。
5. `run_task_loop` 加 future-wake（wake source 6）。
6. `AutoVM::register_external_future` / `resolve_external_future` API。
7. 一个示范 native（`http.get_async`）+ codegen（`expr.await`）。
8. **限制**：Phase A 只支持顶层 `expr.await`，不支持 `~{}` body 内 await 外部 future（body
   内仍只能 await 内部 future）。

**Phase A 验证点**：`let r = http.get_async(url).await` 工作；`run_task_loop` 真挂起/恢复。

### Phase B — `~{}` body 内嵌套外部 future（高复杂度，续点完整化）

1. `execute_future_body` 改造：body 内 `AWAIT_FUTURE` 遇到 External Pending 时，不再递归调
   `execute_future_body`，而是 push AsyncFrame（记录 body 续点 IP + bp）+ 挂起 + Yield。
2. 恢复时：pop AsyncFrame，恢复 body 的 IP/bp，继续跑 body（把外部 future 结果压栈）。
3. body 的局部变量在栈上的位置必须在挂起/恢复间保持一致——这是续点机制的正确性核心，需仔细
   验证栈帧布局不变。
4. 嵌套深度不再靠 Rust 递归栈（受 64 限制），改为 `async_frames` 栈深度（可远超 64）。

**Phase B 验证点**：`~{ let a = http.get_async(url).await; let b = http.get_async(url2).await; merge(a,b) }`
工作；多个外部 future 顺序 await；`run_task_loop` 在 body 挂起期间能调度其它 task。

### Phase C — Future.all/race + a2r async 转译

1. `Future.all([f1, f2])` / `Future.race(...)` native + 组合 future 机制（3.4）。
2. a2r：`async fn` → Rust `async fn`，`.await` → Rust `.await`，`~{}` → `async {}`。
3. 跨平台一致性测试（同一份 .at，VM 和 a2r 行为一致）。

### Phase D（可选）— re-entry yield 迁移 + 表收敛

1. 把现有 re-entry yield native（`http.get` 等）迁移到 external future 范式。
2. `ASYNC_RESULTS` 表 retired（结果改存 `FutureValue.result_slot`）。
3. wake source 5 retired（统一到 wake source 6）。

## §6 关键文件

- `crates/auto-lang/src/vm/engine.rs` — FutureValue（:318）、AWAIT_FUTURE（:7055）、
  execute_future_body（:7294）、handle_await_future（:7241）、execute_task（:7371）、
  run_task_loop（:1598）。Phase A/B 核心。
- `crates/auto-lang/src/vm/task.rs` — AutoTask（:37），加 async_frames/waiting_future_id。
- `crates/auto-lang/src/vm/codegen.rs` — AWAIT_FUTURE 发射（:8629）、`expr.await`、
  `~{}`（CREATE_FUTURE :6999）。Phase A/B codegen。
- `crates/auto-lang/src/vm/opcode.rs` — AWAIT_FUTURE = 0xC1（:271）。
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — native 改用 register_external_future（Phase A 示范）。
- `crates/auto-lang/src/ast/` — `await`/`for await` AST（Phase C，若需新语法）。
- `crates/auto-lang/src/trans/rust.rs` — a2r async 转译（Phase C）。

## §7 风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| 续点栈帧布局正确性（Phase B 核心） | 高 | body 挂起/恢复时栈帧必须位不变；设计专用的栈帧布局测试（mock body 含多层局部变量 + 嵌套 await） |
| AWAIT_FUTURE 重入 pop 顺序（2.3） | 中 | 在 pop 之前检测 External Pending；照搬 CALL_NAT 的 IP-rewind 经验（wake source 5 bug 教训） |
| execute_future_body 语义重构破坏现有 `~{}` | 中 | Phase A 保持 Internal future 走原路径不变；Phase B 改造时用现有 `~{}` 测试做回归 |
| AsyncFrames 内存（深度异步） | 低 | Vec 栈，恢复即 pop；自然回收 |
| 与 re-entry yield 共存的复杂度 | 低 | 共存设计（3.6），native 二选一；Phase D 才统一 |

## §8 调研备忘（2026-08-06）

- 双路 Explore agent 调研确认了 §2 的所有技术细节（execute_future_body 同步内联语义、续点
  缺失、AWAIT_FUTURE 重入栈语义）。
- Plan 349 步骤 7 的 re-entry yield + 表收敛是本计划的务实前置：它验证了「native async +
  run_task_loop 轮询唤醒」的调度模型可行，本计划的 wake source 6 是同模型的泛化。
- wake source 5 的 `waiting_http_request_id` 误清 bug（Plan 349 已修复）是本计划 Phase A
  「不要在 wake 时清等待状态」的直接前车之鉴。

## §9 关联

- **Plan 344**（archive, TODO）：本计划是其第 1 层的正统实施。
- **Plan 349**（步骤 7 + 收敛 phase）：re-entry yield 务实替代，已落地；Phase D 可迁移统一。
- **Plan 348/353**：异步流/IO 基础设施，本计划泛化其调度模型。
