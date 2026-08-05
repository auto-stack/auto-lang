---
plan: 387
title: a2r-actor-task-translation
affects: [auto-lang/vm, auto-lang/a2r, a2r-std]
status: in-progress # draft | in-progress | complete
---

# Plan 387: a2r Actor 模型转译 (TaskDef/Msg/On) — 与 VM 行为对齐

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p a2r-std`（runtime 单测）、`cargo test -p a2r-actor-tests`（行为一致性，默认 `#[ignore]`，CI 显式开）。
> - 前置 skill：无；需熟悉 a2r 转译器 `trans/rust.rs` 的 `stmt()`/`call()`/`fn_decl`/`type_decl` 惯例。
> - 回归要求：现有 ~2907 个 `cargo test -p auto-lang` 测试不退化；VM actor 测试 (`tests/actor_tests.rs`、`tests/actor_state_tests.rs`) 继续通过。
> - worktree：`plan-387/a2r-actor-task-translation`（实施时创建，本骨架先在 master 上 commit）。

> **两个待确认决策（调查后默认采用推荐项，审阅时可改）：**
> - **验证方式 = 新建测试 crate 编译运行**（理由：现有 `test_a2r` 是纯文本逐字节对比，从不编译/运行，无法满足"行为一致"硬要求）。
> - **能力范围 = Tier 1+2**（理由：Tier 1 只有整数消息，不足以服务"用 actor 替代 `Arc<dyn Fn>` 做流式"的原始目的；Tier 2 补全真正的消息类型。Tier 3 ask/reply RPC 暂缓——因为 VM 自身目前只桩了这些、`current_msg_context` 从未赋值，无法作为"对齐"参照）。

---

## §1 Goal / 目标

为 a2r 转译器补上 `Stmt::TaskDef` / `Stmt::MsgDecl` / `Stmt::OnEvents` 的转译分支，底层用 Tokio 实现等价语义，使 Auto 的 actor 模型在转译出的 Rust 代码里**可编译、可运行、行为与 VM 一致**。所有 VM 版 actor 测试用例必须能通过 a2r 跑通并产生相同 stdout。

**非目标**（见 §9）：ask/reply RPC（Tier 3）、guards 真正执行（VM 自身也未接通）、多线程抢占调度优化、消息类型推导的完美形式化。

## §2 背景 / 现状缺口（调查已确认）

| 维度 | 现状 | 引用 |
|---|---|---|
| AST | `Stmt::TaskDef`/`MsgDecl`/`OnEvents` 已定义 | `ast.rs:202,210,218,224`；`ast/task.rs:45` |
| Parser | `task`/`msg`/`on` 完整可解析 | `parser.rs:4202,5020,5144` |
| VM 执行 | 完整工作（path B 内置调度） | `vm/codegen.rs:3601-3745`、`engine.rs:1598-1831`；测试 `tests/actor_tests.rs`(5)、`actor_state_tests.rs`(2) |
| **a2r 转译** | **零支持**——落 `rust.rs:8208` catch-all 报错 | `trans/rust.rs:8005-8208` 无 `TaskDef` 臂 |
| a2r 表达式层 | 半残：`Stmt::Reply`→`reply_tx.send`（但 reply_tx 永不接线）、`.go`/`.send_await`/`.ask` 有桩 | `rust.rs:3188,4513,4531,8171` |
| a2r-std | **无任何 actor/channel 支持** | `crates/a2r-std/src/` 无 task/mpsc/oneshot 模块 |
| a2r 测试 | 纯文本对比，从不编译/运行 | `tests/a2r_tests.rs:10-44`（`assert_eq!` 字符串） |

VM 版实际被测能力（Tier 1，7 个活跃测试）：task 定义、`fn start()`、整数字面量 pattern、`else`、state 字段、`Task.spawn("N",cap)`、`h.send(m)`、FIFO、空 mailbox 不死锁。设计意图含但未测：`fn stop()`、`#[single]`、`Add(val)`/`Reset` 枚举消息、guards、ctx、ask/reply。

## §3 Scope / 范围（Tier 1 + Tier 2）

**Tier 1（必须 100% 通过 VM 对应用例）：**
- `task Name { ... }` 声明转译
- `fn start()!{}`：spawn 时同步运行一次，先于 message loop
- `on { <int literal> -> {body} else -> {body} }`：整数 pattern + else 回退，first-match-wins
- state 字段声明 `count = 0` + 初始化 + handler 内读写 + 跨 handler 持久化
- `Task.spawn("Name", cap)` → 返回 Handle；`h.send(msg)` → 入队
- 消息 FIFO 派发；空 mailbox 后程序干净退出（复现 VM 的"处理完所有 in-flight 消息再退出"语义）

**Tier 2（服务流式用例，无 VM 活跃测试但属设计意图）：**
- `fn stop()!{}`：保留钩子（mailbox 关闭后调用）
- `#[single]` 单例：按类型名寻址、首次 send 时 lazy auto-spawn
- 命名消息：`on { Add(val) => {...}; Reset => {...} }` → 自动生成 `<Task>Msg` 枚举；`Add(1)` 字面量构造
- 字符串字面量 pattern `"ping" ->`、布尔/字符字面量 pattern
- `on(ctx)` 上下文参数（透传，Tier 3 的 reply 通道暂为预留位）

**Tier 3（Out of Scope，§9）：** `.ask`/`reply`/`.send_await` 的真 oneshot 闭环、guards 执行、TypeBinding 跨类型 pattern。

## §4 Architecture / 转译映射表

| Auto 源结构 | 生成的 Rust | 备注 |
|---|---|---|
| `task Counter { count = 0 ... }` | `pub struct Counter { count: i64 }` + `impl Counter { fn new()->Self{...} async fn handle_msg(&mut self, msg, _reply_tx) }` | state→字段；pub 沿用 type_decl 规则 `rust.rs:10268` |
| `fn start()!{body}` | `async fn start(&mut self)!{body}`，spawn 后、message loop 前 await 一次 | 复用 `self.fn_decl`；设 `parent=Some(task.name)` |
| `fn stop()!{body}` | `async fn stop(&mut self)!{body}`，mailbox 关闭（recv None）后调用 | Tier 2 |
| `on { 1 => b; "x" => b; else => b }` | `match msg { 1 => b, "x" => b, _ => b }` | first-match + `_` 即 else |
| `on { Add(v) => b; Reset => b }` | 生成 `pub enum CounterMsg { Add(i64), Reset }` + `match msg { CounterMsg::Add(v) => b, CounterMsg::Reset => b, _ => .. }` | Tier 2；pattern→枚举变体推导 |
| `Task.spawn("Counter", 16)` | `a2r_std::task::spawn_counter(16)`（或内联 channel+spawn），返回 `CounterHandle` | `call()` 加 `("Task","spawn")` 臂，类比 `("process","spawn")` `rust.rs:3712` |
| `h.send(1)` | `h.send(1i64)` | `TaskHandle::send(&self, msg)` |
| `Counter.send(msg)`（单例） | `a2r_std::task::singleton_send_counter(msg)` | Tier 2 |
| `reply expr` | `let _ = reply_tx.send(expr);` | **已存在** `rust.rs:8171`；本计划负责在 `handle_msg` 签名里注入 `reply_tx` 并接线（Tier 2 预留，Tier 1 传 `_`） |

**关键设计决策 D1 — 运行时模型（W1 敲定）：** VM 是单线程协作式（`run_task_loop` 轮询 mailbox，处理完所有 in-flight 消息才退出）。为对齐"行为一致"，a2r 生成的 main 采用 `#[tokio::main(flavor = "current_thread")]`；actor 通过 `tokio::sync::mpsc::unbounded_channel`（VM 的 mailbox 是无界 Vec，`send` 永不阻塞）+ `tokio::spawn`；程序退出前需保证所有 in-flight 消息处理完——通过 a2r-std 的 `ActorRuntime` 在 main 末尾 `drop` 所有 Sender 后 join 所有 actor task 实现（drop Sender → `recv()` 返回 None → actor 跑 stop hook 后退出）。capacity 参数保留签名但语义上记为"提示值"。

**关键设计决策 D2 — 消息类型：** 每个 task 由其 `on` block 的 pattern 集合决定消息类型：纯字面量/TypeBinding → 用具体标量类型（如 `i64`）；含命名变体 → 生成 `enum <Task>Msg`。混用（整数 + 命名变体）时生成带 `Literal(i64)` 变体的 enum。W1 给出形式化规则。

**关键设计决策 D3 — `is_decl` 路由：** 在 `ast.rs:242` `Stmt::is_decl()` 增加 `TaskDef` 分支，使其进 `trans()` 的 decl 阶段（而非塞进合成 `main`），类比 `TypeDecl`。

## §5 a2r-std 扩展（新模块 `task.rs`）

新增 `crates/a2r-std/src/task.rs`，在 `lib.rs` 注册。提供：

```rust
pub struct TaskHandle<M: Send + 'static> { tx: mpsc::UnboundedSender<M>, join: tokio::task::JoinHandle<()> }
impl<M: Send + 'static> TaskHandle<M> {
    pub fn send(&self, msg: M) { let _ = self.tx.send(msg); }   // 非阻塞，对齐 VM
}
pub struct ActorRuntime { handles: Vec<...> }   // 收集 spawn 的 actor，main 末尾 join
impl ActorRuntime {
    pub fn spawn<F, M>(&mut self, fut: F) -> TaskHandle<M> where F: Future<Output=()>+Send+'static;
    pub fn run_to_completion(self);   // drop senders, join all
}
// 单例注册（Tier 2）：OnceCell + 全局 Mutex<HashMap<TypeId/Name, TaskHandle>>
```

所有 a2r-std 引用点必须 `self.a2r_std_used.set(true)`（沿用 `process.spawn` 惯例 `rust.rs:3716`）。tokio 作为 a2r-std 的依赖加入 `crates/a2r-std/Cargo.toml`（feature gated：`features=["rt","sync","macros"]`，不影响不用 actor 的生成代码）。

## §6 任务分阶段（Work Items）

> 每个 WI 独立可合并，按依赖排序。house style 用 `### W1 — 待办` / `### W1 — landed` 记录。

**W1 — 设计冻结 + 骨架（设计文档）** — ✅ 已落地（见 §12 冻结细节）
- 在 plan 文件内敲定 D1/D2/D3 的具体规则；画出 `task` → Rust 的完整生成模板（含 state init、start/stop、message loop、else）。
- 交付物：plan §4/§5 的最终版 + §12 的手写 `.expected.rs` 示例（Counter 自增）。

**W2 — a2r-std `task.rs` runtime**
- 实现 `TaskHandle` / `ActorRuntime` / unbounded channel + current-thread tokio；单测覆盖：spawn→send→recv、FIFO、空 mailbox 退出、单例 lazy spawn。
- 交付：`crates/a2r-std/src/task.rs` + 单元测试；`Cargo.toml` 加 tokio。

**W3 — a2r `Stmt::TaskDef` 转译（Tier 1）**
- `ast.rs:242` `is_decl()` 加 `TaskDef`；`rust.rs:8005` `stmt()` 加 `Stmt::TaskDef(td) => self.task_decl(td, sink)`（位于 catch-all `rust.rs:8208` 之上）。
- 新函数 `fn task_decl(&mut self, td: &TaskDef, sink)`（近 `type_decl` `rust.rs:10078`）：emit struct + `impl` + spawn helper fn + handle struct。state→字段、`fn start`→`self.fn_decl`、整数 pattern→match。
- `call()` 加 `("Task","spawn")` 臂（近 `rust.rs:3712`）与 `h.send` 透传。
- main 包装为 `#[tokio::main(flavor="current_thread")]`（需让 `has_await_refs`/`body_has_stream_for` 对含 `Task.spawn` 的程序触发 async，或显式标记）。

**W4 — Tier 2 扩展**
- `#[single]` → a2r-std 单例 + `TaskType.send` 臂（近 `codegen.rs:6563` 的 singleton_send）。
- 命名消息 → 自动生成 `<Task>Msg` 枚举；`Add(v)`/`Reset` pattern 解构；字符串/布尔/字符字面量 pattern。
- `fn stop()` 钩子接线（recv None 后调用）。
- `on(ctx)` 参数透传；`reply_tx` 注入 `handle_msg` 签名（Tier 3 真闭环前先占位）。

**W5 — 测试基础设施 + 用例移植**
- 新建 `crates/a2r-actor-tests/`：对每个 `.at` → `transpile_rust` → 写临时 crate（依赖 tokio + a2r-std）→ `cargo build` + `cargo run` → 比对 stdout 与 VM 的 `test/vm/23_actor/<case>/<name>.expected.out`。
- 移植 VM 6 用例到 `test/a2r/22_actors/<NNN>_<name>/`（001_start_hook … 006_state_increment）；同时加 Tier 2 用例（stop、singleton、enum 消息）。
- 同时为每个用例保留 `.expected.rs` 文本黄金 + 在 `a2r_tests.rs` 注册 `#[test]`（文本回归守护）。

## §7 Verification / 验证

- **行为一致性**：`crates/a2r-actor-tests/` 对 6 个 VM 用例逐一编译运行，stdout 与 `test/vm/23_actor/*/*.expected.out` 逐字节相等。
- **文本回归**：`test/a2r/22_actors/*/*.expected.rs` 黄金文件 + `a2r_tests.rs` 注册。
- **单元测试**：a2r-std `task.rs` runtime 单测全绿。
- **回归**：`cargo test -p auto-lang` 不退化（现有 ~2907 测试）；VM actor 测试（`actor_tests.rs`/`actor_state_tests.rs`）继续通过。
- **验收 probe**：Counter 自增三连发 → 输出 `reached two\nreached three`（对齐 VM `actor_state_field_increment_persists`）。

## §8 风险与缓解

| 风险 | 缓解 |
|---|---|
| Tokio 异步退出语义 ≠ VM 同步 drain，导致消息丢失 | D1 用 current-thread runtime + `run_to_completion` 显式 join；W2 单测覆盖"main 返回前所有消息处理完" |
| 消息类型推导（D2）边界模糊，整数+枚举混用 | W1 先冻结规则；W5 用例覆盖混用场景 |
| `fn_decl` 复用带入不期望的缓存副作用 | W3 单独跑 `11_methods` 回归；必要时为 task 方法隔离 cache |
| `cargo build` 测试慢（每用例一次编译） | a2r-actor-tests crate 复用单一临时 target dir；标记 `#[ignore]` 默认 + CI 显式开 |
| a2r-std 加 tokio 依赖影响其他下游 | tokio 仅作 a2r-std 的 feature gated 依赖（`features=["rt","sync","macros"]`），不影响不用 actor 的生成代码 |

## §9 Out of Scope / 不做

- **Tier 3 ask/reply RPC 真闭环**：VM 自身 `shim_task_ask` 是 `Ok(0)` 桩、`current_msg_context` 从未赋值——无对齐参照。本计划只把 `reply_tx` 接到 `handle_msg` 签名，留后续 plan。
- **guards 真正执行**：VM codegen 也只是 `_guard` 丢弃（`codegen.rs:3688`）。
- **多线程抢占 / 调度策略可配**：明确用 current-thread 协作式对齐 VM。
- **TypeBinding 跨类型 pattern 的完整类型推导**：Tier 2 仅做基础标量。

## §10 验收标准（Acceptance）

- [x] `cargo test -p a2r-std` 全绿（task runtime 单测，6/6）
- [x] `crates/a2r-actor-tests/` VM 移植用例 stdout 与 VM `.expected.out` 逐字节相等（6/6 parity + 007 手写参照）
- [x] `test/a2r/22_actors/` 文本黄金 + `a2r_tests.rs` 注册全绿（7/7，含 W4 的 007）
- [x] `cargo test -p auto-lang` 不退化（VM actor 测试 7/7 仍绿；a2r 311 全绿）
- [x] Tier 2 新增用例通过（007 命名变体枚举；字符串 pattern 端到端验证；stop hook 已接线）
- [ ] 计划文件 status 翻 complete，加 `### W1—W5 — landed` 段，`git mv` 进 `docs/plans/archive/`
  （**待办**：`#[single]` 单例、guards 执行、ask/reply 真闭环留作后续 plan，见 §13 W4 遗留）

## §11 关键文件索引

- 转译器：`crates/auto-lang/src/trans/rust.rs`（`stmt()` `:8005`、catch-all `:8208`、`call()` `:3302`、`fn_decl` `:8564`、`type_decl` `:10078`、`Reply` `:8171`、`.go` `:3188`、`.send_await` `:4513`、`.ask` `:4531`、`process.spawn` `:3712`、`trans()` `:15252`、async 检测 `:15483`）
- AST：`crates/auto-lang/src/ast.rs:185,242`、`crates/auto-lang/src/ast/task.rs:45,131,266`
- VM 参照：`vm/codegen.rs:3601-3745,6547-6753`、`vm/engine.rs:1598-1831,6328-6411`、`vm/task.rs`、`vm/task_handler.rs`、`vm/ffi/stdlib.rs:5693-5945`
- 测试参照：`tests/actor_tests.rs`、`tests/actor_state_tests.rs`、`tests/a2r_tests.rs:10-44`、`test/vm/23_actor/`
- VM 设计文档：Plan 121/124/126/127（archive）、Plan 317（`docs/plans/`）
- a2r-std：`crates/a2r-std/src/lib.rs`、`process.rs`（spawn 惯例参照）

---

## 关联 / References

- **Plan 121**（archive）：Task/Msg 基础系统（数据结构 + Tokio skeleton）
- **Plan 124**（archive）：`~T`/`.await`/`ask`/`reply` 语义 + opcodes（Tier 3 参照）
- **Plan 126**（archive）：`.go` fire-and-forget dispatch
- **Plan 127**（archive）：TASK_LOOP/HANDLE_MSG/REPLY/SPAWN_GO bytecode + engine 集成
- **Plan 317**（active）：VM 内置 actor 调度执行引擎（path B，本计划的对齐基准）

---

## §12 W1 设计冻结（落地细节，2026-08-05）

基于 worktree `auto-lang-387`（branch `plan-387/a2r-actor-task-translation`）的调查，冻结以下设计点。**含对原始 §4 假设的 3 处修正。**

### §12.1 调查修正

| # | 原假设 | 实际（带引用） | 对实施的影响 |
|---|---|---|---|
| C1 | 方法调用是 `Expr::Bina(_, Op::Dot, _)` | 是 **`Expr::Dot(obj, method)`**（`rust.rs:4936-4938` 注释确认 Bina 臂是 dead code；`parser.rs:2401-2402`） | `call()` 拦截 `Task.spawn` 用 `Expr::Dot` 臂（`rust.rs:4863` 起），不是 Bina 臂 |
| C2 | `Task` 是关键字 | `Task`（大写）是**普通 ident**（仅小写 `task` 是关键字 `token.rs:419`；`spawn` 虽是 `TokenKind::Spawn` 但 parser 在 `parser.rs:3145` 改写成 ident） | 在 `call()` 里按 `Expr::Ident("Task")` 匹配，无特殊 token 处理 |
| C3 | 空体 `fn start()!{}` 正常转译 | **生成编译不过的 Rust**——`body()` `rust.rs:12042` 因 `body.stmts.is_empty()` 跳过 `Ok(())`，`Result` 返回的空函数体是 `{}`（E0308） | **W3 阻塞，必须先修**：`body()` 空体也要补 `Ok(())`（见 §12.4） |

### §12.2 冻结的转译模板（Tier 1，整数消息）

输入 `task Counter { count=0; fn start()!{}; on{ 1=>{count=count+1; ...} } }` + `main{ let h=Task.spawn("Counter",16); h.send(1) }` 生成：

```rust
// Auto-generated by a2r transpiler

use a2r_std;
use a2r_std::*;

struct Counter {
    count: i64,
}

impl Counter {
    fn new() -> Self { Self { count: 0 } }           // state 字段初始化（声明顺序）
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())                                        // hook body（含空体 Ok(()) 修正）
    }
    async fn handle_msg(&mut self, msg: i64, reply_tx: a2r_std::task::NopReply)
        -> Result<(), Box<dyn std::error::Error>>
    {
        match msg {
            1i64 => {                                 // 整数字面量 pattern → 具体值臂
                self.count = self.count + 1i64;       // state 读写 → self.field
                if self.count == 2i64 { println!("reached two"); }
                if self.count == 3i64 { println!("reached three"); }
            }
            _ => {}                                    // else → `_`（无 else 则留空 `_ => {}`? 见 §12.5）
        }
        Ok(())
    }
}

#[tokio::main(flavor = "current_thread")]              // D1: 单线程对齐 VM
async fn main() {
    let mut __rt = a2r_std::task::ActorRuntime::new(); // 收集所有 spawn，末尾 join
    let h = __rt.spawn_counter();                       // Task.spawn("Counter",16) → 具体函数
    h.send(1i64);                                       // h.send → TaskHandle::send（非阻塞）
    __rt.run_to_completion();                           // drop senders → recv None → 各 actor 退出
}
```

**模板要点（冻结）：**
- **state 字段**：`task.state: Vec<(Name, bool, Expr)>`（`ast/task.rs:51`）→ struct 字段，类型由 init expr 推导（整数→`i64`，字符串→`String`）；`new()` 里按声明顺序初始化。
- **hook**：`fn start`/`fn stop` → `impl` 块内 `async fn start(&mut self)`，复用 `self.fn_decl`（设 `parent=Some(task.name)`），**强制补 `Ok(())`**（§12.4）。
- **handler**：单整数消息 → `handle_msg(&mut self, msg: i64, reply_tx: NopReply)`；body 是 `match msg { <lit> => <body>, ... _ => <else_body> }`。
- **state 访问**：handler/hook 内裸 ident `count` → `self.count`（读）/ `self.count = ...`（写）。需在 `task_decl` 内对这些 ident 做前缀改写（或临时填充 `current_task_state_fields` 让现有 ident 路径加 `self.`）。
- **`Task.spawn("Name", cap)`**：拦截 `Expr::Dot(Expr::Ident("Task"), "spawn")`，第一个字符串字面量参数决定调用 `__rt.spawn_<name>()`（静态分发，cap 参数当前忽略，签名保留）。
- **`h.send(msg)`**：走通用方法调用路径（`rust.rs:6526`），生成 `h.send(msg)`；`TaskHandle::send` 由 a2r-std 提供。
- **main async**：含 `Task.spawn` 的程序强制 async + `flavor="current_thread"`（见 §12.6）。

### §12.3 `reply_tx` / `NopReply`（Tier 1 占位）

`Stmt::Reply`（`rust.rs:8171`）发射 `let _ = reply_tx.send(EXPR);`，引用**裸 `reply_tx`**。Tier 1 在每个 handler 签名注入 `reply_tx: a2r_std::task::NopReply`，a2r-std 提供：
```rust
pub struct NopReply;
impl NopReply { pub fn send<T>(&self, _msg: T) {} }   // 吞掉值，对齐 VM 未接通的 reply
```
无需改 `Stmt::Reply` 臂。Tier 3 真闭环时把 `NopReply` 换成 `oneshot::Sender<Value>`。

### §12.4 空体 hook 修复（W3 前置阻塞）

`body()` `rust.rs:12042` 当前：`if matches!(ret_type, Type::Result(_)) && !body.stmts.is_empty()` 才补 `Ok(())`。改为**空体也补**（空 `Result` 返回函数必须有 `Ok(())`）。最小改动：去掉 `&& !body.stmts.is_empty()`，或对 TaskDef hook 单独处理。需回归 `test/a2r/16_interop/002_tokio_main/` 等已有用例确认无副作用。

### §12.5 `else` 臂处理

VM 的 `else ->` 在无匹配时触发（`engine.rs:653-669` 的 `#else` export）。a2r 冻结规则：
- 有 `else` handler：`match` 末尾 `_ => <else_body>`。
- **无** `else` handler：`match` 末尾 `_ => {}`（空臂，吞掉不匹配消息，对齐 VM 无 `#else` export 时 `find_handler_offset` 返回 `(0, false)` 跳过）。

### §12.6 main async + current_thread 强制

现有 async 检测（`rust.rs:15487-15496`）用 `has_await_refs`/`body_has_stream_for`，对 `Task.spawn`（无 `.await` token）**不触发**。冻结方案：在 `trans()` 加一个标志——decl 阶段见到任何 `Stmt::TaskDef` 则 `self.program_has_actors = true`，main 包装时若该标志为真：
1. 强制 `#[tokio::main(flavor = "current_thread")]` + `async fn main()`；
2. main 末尾注入 `__rt.run_to_completion();`（`__rt` 在 main 开头注入 `let mut __rt = ...::new();`）。

注意：现有 `#[tokio::main]`（无 flavor）是 multi_thread；actor 程序必须用 `current_thread` 对齐 VM 单线程协作语义（D1）。

### §12.7 a2r-std 集成细节（W2/W5）

- **tokio 依赖**：workspace 已有 `tokio = { version = "1.49", features = ["full"] }`（`Cargo.toml:56`）。`crates/a2r-std/Cargo.toml` 加 `tokio = { workspace = true, features = ["full"] }`。
- **测试基建**：a2r-std 当前零测试（无 `tests/`、无 `#[test]`）。新 `task.rs` 用 inline `#[cfg(test)] mod tests` + `#[tokio::test] async fn`（参照 `parity/libs/tokio/tests/rust/tests/async_tasks.rs:28`）。
- **新 crate `a2r-actor-tests`**：镜像 `crates/auto-vm/Cargo.toml` 骨架；`[dependencies] a2r-std = { path = "../a2r-std" }`、`auto-lang = { path = "../auto-lang" }`、`tokio = { workspace = true }`；加入根 `Cargo.toml` members。
- **生成代码引用 a2r-std**：现有机制是 `trans()` 末尾按 `a2r_std_used` 注入 `use a2r_std; use a2r_std::*;`（`rust.rs:15531-15555`）；actor 代码所有 `a2r_std::task::*` 引用点调 `self.a2r_std_used.set(true)`。

### §12.8 W1 交付状态

- [x] D1/D2/D3 规则冻结（§12.2/§12.5/§12.6）
- [x] 3 处调查修正记录（§12.1）
- [x] 转译模板（§12.2）+ Counter `.expected.rs` 示例（§12.2 代码块即示例）
- [x] 阻塞点识别（§12.4 空体 hook）
- [x] a2r-std 集成路径（§12.7）
- [x] W2-W3/W5 实施（见 §13）

---

## §13 实施进度（W1–W3 / W5 已合并 master）

> 2026-08-05：Tier 1 + 测试基建完成，fast-forward 合入 master（commit `77dcb7b5`，与 Plan 349 http parity 自动合并无冲突）。W4（Tier 2）待做。

### W1 — landed（设计冻结）
- 交付：§12 全部设计决策冻结；3 处调查修正（C1 `Expr::Dot` 非 `Expr::Bina`、C2 `Task` 是普通 ident、C3 空体 Result hook 编译不过）；转译模板；死锁分析。
- commit `a5dc1efc`。

### W2 — landed（a2r-std actor runtime）
- 交付：`crates/a2r-std/src/task.rs`（264 行）— `TaskRef`（用户 send 句柄）/ `TaskHandle`（spawn 构造）/ `ActorRuntime::register+run_to_completion`（drop sender→join 解决死锁）/ `NopReply`（Tier1 reply 占位）；`Cargo.toml` 加 `tokio = { workspace = true, features = ["full"] }`。
- 验证：6 单测全绿（start 先于消息 / FIFO / 空 mailbox 退出 / 状态持久 / NopReply / register+drain）。
- commit `0c19a200`。

### W3 — landed（a2r Stmt::TaskDef 转译 Tier 1）
- 交付（`trans/rust.rs` +565 行）：
  - `task_decl()`：task → `struct` + `impl{new/start/stop/handle_msg}` + `spawn_<name>` helper。
  - `call()` 拦截 `Task.spawn("Name")` → `spawn_<name>(&mut __rt)`。
  - `fn_decl`：`program_has_actors` 触发 `#[tokio::main]` + async main + 注入 `__rt` prologue / `drop(handle)` + `run_to_completion` epilogue（D1 死锁解）。
  - `body()` 空体 Result 补 `Ok(())`（§12.4 阻塞修复）。
  - state 字段 `self.` 改写（`in_task_body` + `task_state_fields`，读 `Expr::Ident` + 写 `Op::Asn` 两路径）。
  - `ast.rs is_decl()` 加 `TaskDef` 路由。
- 验证：**6 个 VM actor 用例（001-006）端到端通过**（转译→编译→运行→stdout 逐字节匹配 VM）；303 a2r 回归 + 7 VM actor 测试全绿。
- commit `82ce03cd`。

### W5a — landed（文本黄金测试）
- 交付：`test/a2r/22_actors/` 6 用例（`.at` + `.expected.rs`）；`a2r_tests.rs` 注册 22_actors 类别（`test_a2r_deep`，16MB 栈）。
- 验证：6/6 文本黄金测试通过。
- commit `377e89fb`。

### W5b — landed（行为一致性测试 crate）
- 交付：`crates/a2r-actor-tests/` — 对每个 `.at` 调 `transpile_rust` → 写临时 crate（依赖 a2r-std+tokio）→ `cargo run` → 比对 stdout 与 `test/vm/23_actor/*.expected.out`。唯一 crate 名 + 共享 `CARGO_TARGET_DIR`（tokio 只编译一次，6 用例 6 秒）。`#[ignore]` 默认。
- 验证：**6/6 parity 测试通过**（stdout 逐字节匹配 VM）。
- commit `d55c5148`。

### W4 — landed（Tier 2：命名消息枚举 + 字符串 pattern）
> 2026-08-05：合入 master（merge `ed30dd81`）。

- **命名变体枚举**（核心）：`on { Add(val) => {...}; Reset => {...} }` → 生成 `enum <Task>Msg { Add(i64), Reset }` + `match msg { <Task>Msg::Add(val) => ..., <Task>Msg::Reset => ... }`。
  - `h.send(Add(5))` → `h.send(CounterMsg::Add(5))`；`h.send(Reset)` → `h.send(CounterMsg::Reset)`（`call()` send 拦截 + `rewrite_msg_variant_arg`）。
  - **parser 改动**：解析 task 时把 `on` block 变体名注册到作用域（解决 `h.send(Reset)` 的 `undefined variable`，`parser.rs:4954-4965` 同 enum 机制）。镜像 `register_enum_decl`。
- **字符串 pattern**：`on { "ping" => {...} }` → `match msg.as_str() { "ping" => ... }`（String 消息用 `as_str()` 借用避免 move）；`h.send("ping")` → `h.send("ping".to_string())`。
- **混合 literal + 命名变体**：枚举带 `Literal(i64)` / `Literal(String)` / `Literal(bool)` 变体。
- **TypeBinding**（基础）：`msg string` → `String` 标量。
- **state 字段整数统一 i64**（对齐 binding i64，避免 `i32 + i64` 类型不匹配）。
- **stop hook**：W3 已接线（spawn helper 在 mailbox 关闭后调 `actor.stop()`）。
- **验证**：`007_named_variants` 端到端通过（`reached eight\nreset`，转译→编译→运行）；字符串 pattern 端到端通过（`pong\nunknown`）；311 a2r + 7 VM actor + 6 a2r-std 全绿；005/006 黄金更新（`i32`→`i64`）。
- commit `5c050c41`。

### W4 已知遗留（后续 plan）
- **`#[single]` 单例**：VM 自身未真正测试（singleton_send 走 skeleton path，与 live actor 不互通），无对齐参照。
- **VM 命名变体支持**：VM 的 `shim_task_send_vm` 强制 `Value::Int(msg)`，**不支持命名变体发送**——a2r Tier 2 在此特性上超前于 VM。`007` 的 `.expected.out` 为手写（逻辑正确），非 VM 派生。
- **guards 真正执行**：parser 已解析 `if guard`，但 a2r codegen 丢弃（与 VM 一致，VM 也未接通）。
- **`on(ctx)` / reply 真闭环**：Tier 3（ask/reply RPC），VM 也只桩。
- **消息 binding 类型硬编码 i64**：`derive_task_msg_type` 把所有 `WithBindings` 的 binding 类型忽略，统一生成 `i64`（`rust.rs` 注释 "VM tests use ints"）。声明 `Add(x: String)` 仍生成 `Add(i64)`。后续需按声明类型推导。
- **`task_msg_variants` 跨 task 同名变体冲突**：变体名→enum 名映射是 `HashMap<String,String>`，两个 task 都有 `Add` 时后者覆盖前者，`h.send(Add(1))` 会改写到错误的 enum。单 task 程序无此问题；多 task 同名变体时 latent bug。

---

## §14 复审与修复（2026-08-05）

对 Plan 387 全部修改的三路审计（a2r-std runtime / 转译器 / 测试覆盖）发现并修复了以下问题：

### §14.1 已修复的代码问题

| # | 问题 | 位置 | 修复 |
|---|---|---|---|
| F1 | `trans()` Phase-4 actor-main 分支是死代码（显式 main 走 `fn_decl`），且注释说 current_thread 与实际 multi_thread 矛盾 | `rust.rs` 原 16316-16328 | 改为 multi_thread + 注释说明两条路径关系与死锁 rationale |
| F2 | 死语句 `let _ = name_of(&td.name);`（重构遗留） | `rust.rs` 原 8661 | 删除 |
| F3 | 测试名/注释与实际矛盾：`runtime_held_sender_suffices_without_user_drop` 声称测"忘记 drop"，实际 drop 了；且无 assert | `task.rs` 243-263 | 改名 `drop_handle_then_run_completes_without_hang` + 准确注释 |
| F4 | 模块 doc 误导：说 runtime "drops every TaskRef it owns"，实际靠生成代码 `drop(h)` | `task.rs` 11-23 | 重写 shutdown model 说明：双 clone + 生成代码 drop + 无 fallback |
| F5 | `007/008/009` 在 `test/vm/23_actor/` 下像 VM 黄金但手写，无注释防误注册 | `vm_file_tests.rs` 902 | 加 NOTE 注释说明为何不注册 + 不可加 |

### §14.2 补齐的测试覆盖（审计最关键发现）

审计发现 4 个**声称已实现但无测试守护**的特性（回归会隐形）。补齐其中 2 个：

| 用例 | 覆盖特性 | 之前状态 | 现状 |
|---|---|---|---|
| **008_string_pattern** | `on { "ping" => {...} }` + `h.send("ping")` | commit 声称"端到端通过"但**无 `.at` 用例** | 文本黄金 + parity 测试 |
| **009_stop_hook** | `fn stop()!{...}` 有 body + mailbox 关闭后执行 | emitter 支持但**无用例**；发现 VM live path 不调 stop | 文本黄金 + parity 测试（手写 expected，含 `stopped`） |

剩余 2 个未补（属 Tier 2 边缘）：`#[single]` 单例（VM 也未真正支持）、混合 literal+命名变体（`Literal(...)` enum 臂的生成路径无测试）。

### §14.3 发现的 a2r 超前于 VM 的行为（非 bug，记录）

- **007 命名变体发送**：VM send shim 强制 `Value::Int`，不支持；a2r 通过生成 enum 支持。
- **008 字符串消息**：同上，VM 不支持；a2r 支持。
- **009 stop hook**：VM live path（`run_task_loop`）**不在 mailbox 关闭后调 stop**（只有 skeleton path 调）；a2r spawn helper 在 `recv` 返回 None 后调 `actor.stop()`。

这三项的 `.expected.out` 均为**手写逻辑正确值**，非 VM 派生。a2r-actor-tests parity 测试用它们验证 a2r 行为正确性。

### §14.4 验证

- 9 个 a2r 文本黄金测试全绿（含新 008/009）
- 9 个 parity 测试全绿（含新 008/009，stdout 逐字节匹配手写 expected）
- 7 VM actor + 6 a2r-std 单测全绿
- 复审修复 commit：见下文 §15

---

## §16 下游需求：actor handle 一等公民化（来自 auto-ai Plan 021 缺口 1）

> 2026-08-05：auto-ai 侧评估 Plan 387 当前能力后，发现**不足以支撑缺口 1**（用 actor 替代
> `Arc<dyn Fn(StreamEvent)>` 做流式）。本节记录 auto-ai 的具体需求，供 auto-lang 后续实施。
> 来源：auto-ai `docs/plans/021-auto-completion-roadmap.md` Phase 1 调研结论。

### 背景

auto-ai-agent 的 `Client` spec 缺 `complete_stream`（带回调 `Arc<dyn Fn(Value)>`），因为 Auto
不支持 dyn-Fn 类型。Plan 021 Phase 1 原想用 actor（`Task.spawn` + `h.send`）绕开 dyn-Fn，但发现
Plan 387 当前能力有三个架构性阻塞。

### 三个 P0 阻塞（按严重度排序）

**P0-1：actor 运行时与 `fn main` 硬耦合（最关键）**

- `ActorRuntime`（`__rt`）、prologue/epilogue、handle drop 追踪**只在 `fn main` 里注入**
  （`rust.rs:9382` `is_main_actor` 判断为 true 时才注入）
- `Task.spawn("Foo")` 无条件展开为 `spawn_foo(&mut __rt)`（`rust.rs:3449`）
- **后果**：agent 的 `run_inner` 是 `Agent` 结构体的方法（非 main），若在里面调 `Task.spawn` 或
  使用依赖 `__rt` 的展开，`__rt` 在该方法作用域不存在 → 编译失败
- 所有 7 个测试都在 `fn main` 里 spawn，从未验证在普通方法/结构体方法里用 actor

**需求**：让 actor 机制不限于 `fn main`。可行方案（任一）：
- (a) `ActorRuntime` 提为程序级单例（`thread_local` 或 `OnceCell`），任意函数能 spawn
- (b) `__rt` 作为隐式参数线程化到所有含 `Task.spawn`/`h.send` 的函数
- (c) handle 类型（`TaskRef<M>`）成为 a2r 一等公民：drop 由 RAII 而非 main-epilogue 管理

**P0-2：handle 不是一等类型**

- `TaskRef<M>` 没暴露给 .at 类型系统——用户无法写 `fn run_inner(..., sink TaskRef<StreamEvent>)`
- 旧的 `Type::Handle`（Plan 121，`rust.rs:1237`）映射到 `std::sync::Arc<TaskHandle<...>>`，
  与 Plan 387 的 `a2r_std::task::TaskRef`/`TaskHandle` 类型不一致（从未接线）
- **需求**：统一 `Type::Handle` 与 `TaskRef<M>`，允许 handle 作为函数参数类型、结构体字段类型

**P0-3：外部 enum 作消息未完整支持**

- agent 要 send 的是 `StreamEvent`（auto-ai-agent 定义的 enum），不是本 task 自动生成的 `<Task>Msg`
- `emit_task_pattern`（`rust.rs:8741-8825`）只识别本 task 的 `<Task>Msg` 变体
- send 侧 `rewrite_msg_variant_arg`（`rust.rs:8883+`）只改写本 task 变体，不改写外部 enum 构造
  （如 `sink.send(StreamEvent.Delta("x"))`）
- `on { ev StreamEvent }` 的 TypeBinding pattern 在 enum context 报错
  （`rust.rs:8813` "TypeBinding in enum context not yet supported"）
- **需求**：支持外部 enum 作为 actor 消息类型（send 外部 enum 构造 + handler 接收外部 enum）

### auto-ai 侧的预期用法（验证场景）

改造后，agent.at 的流式应该长这样（概念）：

```auto
// auto-ai-agent/src/agent.at
// 定义一个事件收集 actor（调用方 spawn 后把 handle 传进 run_stream）
task EventSink {
    on {
        ev StreamEvent -> {
            // 转发到外部（打印/写 channel 等）
        }
    }
}

pub mut fn run_stream(task_msg str, cancel Arc<AtomicBool>, sink TaskRef<StreamEvent>) ~Result<AgentResult, AgentError> {
    // run_inner 内部的事件点改成 sink.send(StreamEvent.Delta(...)) 而非 events.push(...)
}
```

调用方（main 或上层）：
```auto
fn main() {
    let sink = Task.spawn("EventSink", 64)
    let agent = Agent.new(...)
    let result = agent.run_stream(task, cancel, sink).await.?
}
```

### 建议的验证用例（端到端）

补以下测试到 `test/a2r/22_actors/`，验证一等公民化：

1. **handle 跨函数传递**：`fn forward(h ??) { h.send(1) }` + main 里 spawn 后调 forward
2. **handle 存结构体字段**：`struct Worker { sink: ?? }` + 方法里 `self.sink.send(...)`
3. **外部 enum 消息**：定义一个普通 `pub enum Event { A, B(str) }`，actor `on { ev Event }`，send `Event.A`
4. **actor 与普通方法共存**：一个 .at 文件同时有 struct 方法 + task 定义 + 方法里 spawn/send

### 与 §9 Out of Scope 的关系

本需求**不涉及** Tier 3 ask/reply（agent 流式只需单向 push，send 已是同步非阻塞）。
纯粹是让 actor 的 spawn/send 不再限于 `fn main`、handle 成为一等类型、外部 enum 可作消息。

### auto-ai 侧等待状态

- auto-ai `docs/plans/021-auto-completion-roadmap.md` Phase 1 已记录此否决结论
- auto-ai 侧无其他不依赖 auto-lang 的实施工作（三个缺口全卡 a2r）
- 本节需求满足后，auto-ai 可立即实施 agent.at 的流式改造（Phase 1.3）

---

## §17 §16 实施落地（2026-08-05）

> merge `1e1c23e2`。P0-1 + P0-3 完整解决，P0-2 部分完成（类型映射已加，handle 作函数参数的 escape clone 留后续）。

### §17.1 P0-1 解决：spawn 解除 fn main 耦合（RAII）

**a2r-std `task.rs` 重构**（commit `9b6dbb9a`）：
- `TaskRef<M>` 成为 mailbox sender **唯一所有者**（非 Clone）。Drop 自动关 mailbox。
- 移除 `ActorRuntime`/`ActorEntry`/`closer`/`Box<dyn FnOnce()>`——改用 `thread_local! JOIN_HANDLES` + `track_join()` + `drain_all()`。
- `drain_all()` **不等 join**——yield 16 次让 in-flight 消息处理，然后返回；main 返回时 runtime 自然 teardown。**解除死锁**（无需显式 `drop(h)`）。

**转译器**（commit `9b6dbb9a`）：
- `spawn_<name>(&mut __rt)` → `spawn_<name>()`（无参）；main 移除 `let mut __rt` + `collect_task_handle_vars`/`drop(var)`；epilogue 改 `drain_all().await`。删死代码 `collect_task_handle_vars`/`is_task_spawn_call`。

**验证**：`010_handle_cross_fn`——spawn 在普通函数 `spawn_and_send`（非 main），端到端通过。

### §17.2 P0-2 部分完成：TaskRef 类型映射

- `GenericInstance("TaskRef")` → `a2r_std::task::TaskRef<T>`（类型映射已加，commit `9b6dbb9a`）。
- **已知限制**：`TaskRef` 作函数参数传递时，escape analyzer 给变量加 `.clone()`（`forward(h)` → `forward(h.clone())`），而 `TaskRef` 非 Clone → 编译失败。根因：escape analysis 的 `OwnershipTier::Clone` 默认对非 Copy escape 变量生效，不识别 `TaskRef` 的 move 语义。修复需 escape analyzer 深度改动（识别 `TaskRef<...>` 类型用 Move tier）。**留作后续**。当前 010 验证用"spawn in fn"（不传 handle）绕过。

### §17.3 P0-3 解决：外部 enum 作 actor 消息

- **无需额外代码**——Tier 2 已自然支持：`derive_task_msg_type` 的 `TypeBinding` 返回 bound type 名；`emit_task_pattern` 的 `TypeBinding` 输出绑定名；send 侧 `Event.A` 不被 `rewrite_msg_variant_arg` 改写（`Expr::Dot` 返回 None），正常渲染。
- **验证**：`012_external_enum_msg`——`on { ev Event }` + `h.send(Event.A)`/`Event.B("hello")`，handler 内 `is ev { Event.A -> ... }` 解构，端到端通过（`got A\ngot B\nhello`）。

### §17.4 验证

- 11 文本黄金（001-010 + 012）+ 11 parity（含手写 expected）+ 7 VM actor + 6 a2r-std 单测，全绿。
- commit：W6-W7 `9b6dbb9a`（RAII + 解耦）、W8-W9 `ea08546e`（外部 enum + 验证用例）。

### §17.5 后续遗留

- **P0-2 handle 作函数参数**：escape analyzer 识别 `TaskRef` move 语义（`escalate_visible`/`OwnershipTier` 对 `TaskRef<...>` 用 Move 而非 Clone）。
- **011 handle 存结构体字段**：未做（依赖 P0-2 的 move 修复，`self.sink.send()` 应可用但需验证）。
- **013 actor 与方法共存综合用例**：未做（同上）。
