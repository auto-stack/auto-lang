---
plan: 387
title: a2r-actor-task-translation
affects: [auto-lang/vm, auto-lang/a2r, a2r-std]
status: draft # draft | in-progress | complete
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

- [ ] `cargo test -p a2r-std` 全绿（task runtime 单测）
- [ ] `crates/a2r-actor-tests/` 6 个 VM 移植用例 stdout 与 VM `.expected.out` 逐字节相等
- [ ] `test/a2r/22_actors/` 文本黄金 + `a2r_tests.rs` 注册全绿
- [ ] `cargo test -p auto-lang` 不退化（VM actor 测试仍绿）
- [ ] Tier 2 新增用例（stop/singleton/enum 消息）通过
- [ ] 计划文件 status 翻 complete，加 `### W1—W5 — landed` 段，`git mv` 进 `docs/plans/archive/`

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
- [ ] W2-W5 实施（下一阶段）
