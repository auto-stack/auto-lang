---
plan: 390
title: actor-state-injection
affects: [auto-lang/parser, auto-lang/a2r, auto-lang/vm, a2r-std]
status: in-progress # draft | in-progress | complete
---

# Plan 390: Actor 状态注入机制 + a2r call-site spec 自动装箱修复

> **两个独立工作单元**，共享本计划与 worktree：
> - **§1–§7（Phase A–D）**：Actor 状态注入（spawn 带参）— 来源 auto-ai Plan 021 **§6.7**。
>   继承 Plan 389（task 作用域三修复）后的下一步：`(self.cb)(ev)` 语法已可用，但 app 无法把回调注入 actor。
>   对应 auto-ai 缺口 1 的 sink→app 转发收尾；非缺口 1 必需。
> - **§11（Phase E）**：a2r call-site spec 参数自动装箱缺陷修复 — 来源 auto-ai Plan 021 **缺口 2**。
>   2026-08-06 实证调研推翻"需泛型语法"前提：spec-param 机制已能实现 `register(my_tool)` 的体验，
>   缺陷在 call-site 不自动 `Box::new`。详见 §11。
>
> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p a2r-std`（runtime 单测）、
>   `cargo test -p a2r-actor-tests`（行为一致性，默认 `#[ignore]`，CI 显式开）。
> - 前置 skill：无；需熟悉 Plan 387（actor 转译 §16/§17）、Plan 389（scope 三修复），
>   以及 `parser.rs` 的 `parse_task_*` / `trans/rust.rs` 的 `emit_task_struct` / `task_system.rs` 的 `shim_task_spawn`。
> - 回归要求：现有 `cargo test -p auto-lang` 不退化；a2r `22_actors` 001-015 文本黄金逐字节不变；
>   VM actor 测试（`tests/actor_tests.rs`、`actor_state_tests.rs`）继续通过。
> - worktree：`plan-390/actor-state-injection`（实施时创建，本骨架先在 master 上 commit）。

---

## §1 Goal / 目标

为 Auto 的 actor 模型提供 **"从外部向 actor 注入状态/回调"** 的能力，使 EventSink 这类
"接收事件并转发到 app 提供的 channel/回调" 的模式可落地。当前 Plan 389 修复后 actor 内部
`(self.cb)(ev)` 语法可用，但 `Task.spawn` 无初始化参数、task state 默认值固定、无"设状态"
控制消息——app 拿到的 actor handle 无法注入 `cb`。本计划三选一（或多选）解除该阻塞。

**非目标**：Tier 3 ask/reply RPC（Plan 387 §9 已排除）、guards 真正执行、多线程抢占调度优化。
本计划不解决 sink→app 的 channel 类型推导（那是 driver.at / app 侧工作，见 §10）。

## §2 背景 / 问题陈述

### §2.1 现状（Plan 389 后）

auto-ai 的 `agent.at` 定义 `task EventSink`（Plan 021 Phase 1b）：

```auto
task EventSink {
    log = ""
    cb = noop_event      # fn(StreamEvent) -> () 默认值
    on { ev StreamEvent -> {
        log = f"${log}D:${ev.depth};"   # R3 修复后可用
        (self.cb)(ev)                   # R2 修复后 cb 字段类型推导为 fn(...) 正确
    } }
}
```

Plan 389 修复 R1/R2/R3 后，**actor 内部**转发链 `(self.cb)(ev)` 编译通过。但 app 侧：

```auto
fn main() {
    sink = Task.spawn("EventSink", 16)   # 无初始化参数！
    # ❌ 无法把 app 的 channel/SSE 回调传给 sink 的 cb 字段
    #    —— sink.cb 永远是 noop_event 默认值
    run_stream(task, cancel, sink)
}
```

### §2.2 三个阻塞点（全部实证于 auto-ai Plan 021 §6.7）

| # | 阻塞 | 现状 | 影响 |
|---|---|---|---|
| B1 | `Task.spawn` 无初始化参数 | `Task.spawn("Name", cap)` 签名只接收 name + capacity，无法 `Task.spawn("EventSink", 16, cb)` | app 无法在 spawn 时注入初始状态 |
| B2 | task state 默认值固定 | state 字段 `cb = noop_event` 在 task 声明里写死；无"覆盖默认值"的 spawn 后 API | actor 永远持有默认 cb |
| B3 | 无"设状态"控制消息 | `on` block 只能匹配消息；无 `on { __set_cb(cb) -> { self.cb = cb } }` 的"消息触发状态写入"语法 | 无法通过 handle.send 注入状态 |

**结果**：EventSink 永远把事件转发给 `noop_event`（什么都不做），app 的真实 channel 收不到事件。
缺口 1 的 sink→app 转发链断裂在此。

### §2.3 相关实现位置

- VM `Task.spawn`：`crates/auto-lang/src/vm/codegen.rs:7473-7495`（`shim_task_spawn(task_type, capacity)`，
  注入 task_type + capacity 两个参数，**无第三参数路径**）
- VM task 创建：`vm/task_system.rs:362,598-658`（`spawn_initial_tasks` → `TaskContext`，
  state 从 task 声明的默认值初始化）
- a2r 转译：`trans/rust.rs` `emit_task_struct`（Plan 387 §16）生成 `<Task>::new()` 无参构造；
  `Task.spawn("N", cap)` → `spawn_<task>(cap)`（Plan 387 §4 映射表）
- parser：`parse_task_with_attrs`（`parser.rs:5111`，Plan 389 §4.3 已在此加 push/pop scope）
- AST：`Stmt::TaskDef`（`ast.rs:202` / `ast/task.rs:45`）

## §3 Scope / 范围

本计划交付**三个候选机制中的至少一个**（推荐组合见 §4 决策），使 app 能向 actor 注入 cb/状态：

- **M1 — spawn 带初始化参数**：`Task.spawn("EventSink", 16, cb, init_val)` → spawn 时填充 state 字段
- **M2 — 状态写入控制消息**：语法支持 `on { __set_cb(new_cb) -> { self.cb = new_cb } }`，
  app 用 `sink.send(__set_cb(my_cb))` 注入
- **M3 — 只读访问器 / setter 方法**：task 声明支持 `fn set_cb(self, new_cb) -> { self.cb = new_cb }`，
  app 用 `sink.set_cb(my_cb)` 调用

**验收（任一机制满足即解锁 auto-ai §6.7）**：app 能把一个 `fn(StreamEvent)` 注入 EventSink，
后续事件转发到该回调，stdout 可观察。

## §4 Architecture / 设计决策

### D1 — 推荐机制：M1（spawn 带初始化参数）

**理由**：
1. 语义最自然——与 Rust 的 `Actor::new(cb).spawn()` / `tokio::spawn(async move { actor.run().await })` 对齐
2. VM 侧改动最小——`shim_task_spawn` 已注入 task_type + capacity，扩成 `shim_task_spawn(task_type, capacity, ...init_args)` 在 codegen.rs:7495 注入点追加即可
3. 无新语法——`Task.spawn("N", 16, arg1, arg2)` 是现有 call 语法的参数列表扩展，parser 无需新分支
4. 顺序正确——初始参数在 message loop 前注入，`fn start()` 能看到已注入的 cb（符合 Plan 387 §3 "spawn 后先 start 再 message loop"）

**D1 映射（M1）**：

| Auto 源 | 生成/执行 |
|---|---|
| `Task.spawn("EventSink", 16, cb)` | VM：`shim_task_spawn("EventSink", 16, cb)`，task 创建后、`fn start()` 前按位置赋值给 state 字段（需声明顺序约定）<br>a2r：`spawn_event_sink(16, cb)` → `EventSink::new_with_state(cb)` + `tokio::spawn` |
| `state cb = noop_event`（默认值） | 默认值仍保留（无参 spawn 时用）；带参 spawn 时按位置覆盖 |
| 字段-参数位置映射 | 按 task 声明里 state 字段的出现顺序，与 spawn 额外参数按位置对应。a2r 生成 `new_with_state(cb: fn(StreamEvent))` 形参名取自字段名 |

**D1 待决问题（W1 调研后定）**：
- **Q1 字段筛选**：是否所有 state 字段都可注入，还是用标注（如 `#[inject] cb = noop_event`）标记可注入字段？默认推荐"全部可注入，按声明顺序"，避免新语法。
- **Q2 类型推导**：spawn 调用点的额外参数类型如何匹配字段类型？a2r 侧用 `infer_type_from_expr`（Plan 389 §4.2 已扩展识 fn 指针）反推；VM 侧值是 opaque handle，运行时赋值即可。
- **Q3 capacity 位置**：保持 `Task.spawn(name, capacity, ...init)` 顺序，还是 `Task.spawn(name, ...init, capacity)`？推荐前者（capacity 是现有必填位，保持二进制兼容）。

### D2 — 备选机制：M2（状态写入消息）

若 D1 的 Q1/Q2 证明位置映射过复杂，退而用语法标注的"setter 消息"：

```auto
task EventSink {
    cb = noop_event
    on_set(cb) -> { self.cb = cb }     # 新语法：声明性 setter 消息
    on { ev StreamEvent -> { (self.cb)(ev) } }
}
fn main() {
    sink = Task.spawn("EventSink", 16)
    sink.send_set_cb(my_cb)            # 或 sink.send(SetCb(my_cb))，生成专用消息变体
}
```

**缺点**：需新 parser 分支（`on_set(field) -> {body}`）+ a2r 生成专用枚举变体 + VM 新消息类型；比 M1 工作量大。仅作退路。

### D3 — 备选机制：M3（setter 方法）

最贴近 OOP 但偏离 actor "消息传递" 语义：`task` 内允许声明 `fn set_cb(self, v) { self.cb = v }`，
app 直接 `sink.set_cb(v)` 同步调用。**不推荐**——破坏 actor 的消息隔离（直接调 self 方法 = 跨线程共享可变状态），
与 Plan 387 §3 的"actor 通过 mailbox 序列化所有访问"原则冲突。仅记录为备选。

## §5 实施任务（按 D1=M1 推进，W1 调研后可调）

### Phase A — VM 侧（M1）

- [ ] 5.1 `shim_task_spawn` 扩参：`shim_task_spawn(task_type, capacity, ...init_args)`；
      `codegen.rs:7473-7495` 注入点追加 init_args 的栈布局（先 task_type、capacity，再按位置推 init 值）
- [ ] 5.2 `task_system.rs` spawn 路径：task 创建后、`fn start()` 前，按 state 字段声明顺序用 init_args 覆盖默认值；
      init_args 不足时用默认值补齐（向后兼容现有 `Task.spawn("N", cap)` 两参调用）
- [ ] 5.3 parser：`Task.spawn` 调用的参数列表解析已通用（call 语法），确认 `> 2` 个实参不报错；
      若 parser 有 arity 检查（`native_catalog.rs:1353` 的 `("Task.spawn", 2300, Void)`）放宽为可变参
- [ ] 5.4 字段-参数顺序约定文档化：按 state 字段在 task 声明中的出现顺序映射 spawn 额外实参

### Phase B — a2r 侧（M1）

- [ ] 5.5 `emit_task_struct`：除生成 `<Task>::new()`（默认值构造，无参 spawn 用），新增 `<Task>::new_with_state(f1, f2, ...)`
      按字段顺序形参（形参名 = 字段名，类型 = Plan 389 §4.2 推导的字段类型）
- [ ] 5.6 `Task.spawn("Name", cap, ...args)` 的 `call()` 臂（Plan 387 §4 映射表）：args ≥ 1 时生成
      `spawn_<task>(cap, arg1, arg2)` → 内部 `let actor = <Task>::new_with_state(arg1, arg2); tokio::spawn(...)`
- [ ] 5.7 无参（仅 name+cap）spawn 保持现有 `spawn_<task>(cap)` → `<Task>::new()`，向后兼容

### Phase C — 验证

- [ ] 5.8 新增 a2r 用例 `22_actors/016_spawn_with_state.at`：task 带 fn 指针 state 字段，
      `Task.spawn("...", 16, real_cb)` 注入，handler 内 `(self.cb)(...)` 调用真实回调，stdout 确认
- [ ] 5.9 新增 a2r 用例 `22_actors/017_spawn_partial_init.at`：3 字段 task，spawn 只传 1 个，
      其余用默认值——验证位置映射 + 默认值补齐
- [ ] 5.10 VM 行为一致：上述用例在 VM（`cargo run`）与 a2r 转译版（独立 crate 编译运行）产生相同 stdout
- [ ] 5.11 回归：001-015 文本黄金逐字节不变；`cargo test -p auto-lang` 无新增失败；
        VM actor 测试全绿

### Phase D — 回 auto-ai 落地（解锁 §6.7，auto-ai 侧）

- [ ] 5.12 重建 auto.exe（合并 auto-lang Plan 390 + Plan 389 后）
- [ ] 5.13 auto-ai `agent.at` 的 EventSink：`run_inner` 内 `Task.spawn("EventSink", 16, app_cb)` 注入真实回调
       （app_cb 由 `run_stream` 调用方传入，替代当前 noop 默认值）
- [ ] 5.14 driver.at 恢复 rust-ref 等价：`Delta/Tool → PipelineEvent` 转发（Plan 021 Phase 6.6，
       依赖本计划的 cb 注入机制）
- [ ] 5.15 retranspile 0 错；端到端：agent 流式事件经 EventSink → app channel/SSE 可观察

## §6 风险与注意

- **B1 位置映射的脆弱性**：state 字段顺序变化会破坏 spawn 调用点（无编译期检查）。
  缓解：W1 调研是否用标注 `#[inject]` 显式标记可注入字段（D1 Q1），或文档强约束"task 字段顺序稳定"。
- **向后兼容**：现有 `Task.spawn("N", cap)` 两参调用必须无变化（用默认值补齐 init）。
  Phase A/B 的 5.2/5.7 明确保证。
- **VM 与 a2r 一致**：Plan 387 §10 已建立 a2r-actor-tests 行为一致性测试基建；
  本计划 5.10 沿用，必须 VM 与 a2r stdout 一致。
- **只构建 debug**（用户既定要求）：全程 `cargo build`，不跑 release。
- **不涉及 sink→app 的 channel 类型**：本计划只解决"注入 cb"，cb 是 `fn(StreamEvent)` 还是
  `mpsc::Sender<PipelineEvent>` 是 app/driver.at 侧的类型问题（auto-ai Plan 021 Phase 6.6）。

## §7 验收标准

**Phase A–D（缺口 1 收尾 / §6.7）**：
- [ ] app 能通过 `Task.spawn("EventSink", 16, cb)` 注入 `fn(StreamEvent)` 回调
- [ ] EventSink 后续事件转发到注入的 cb（stdout 可观察，非 noop）
- [ ] VM 与 a2r 转译版行为一致（5.10）
- [ ] 现有 actor 测试零回归（001-015 黄金 + VM actor_tests 全绿）
- [ ] 向后兼容：无参 spawn 行为不变
- [ ] auto-ai Plan 021 §6.7 解锁，Phase 6.6 可推进（回 auto-ai 侧，见 §5 Phase D）

**Phase E（缺口 2 / §11）**：
- [ ] `r.register(my_tool)`（具体结构体实参）转译为 `r.register(Box::new(my_tool))`，编译通过
- [ ] spec-bound ident 实参仍走 `.clone()`，行为不变
- [ ] 现有 spec 测试零回归；`cargo test -p auto-lang` 无新增失败

**Phase F（缺口 2 回 auto-ai / §12）**：
- [ ] `tool.at`/`agent.at` 加 `register(tool Tool)` / `register_tool(tool Tool)`，retranspile 0 错
- [ ] auto-ai-cli 的 `register_tool` call site 不再需手包箱
- [ ] auto-ai Plan 021 Phase 2 + 缺口 2 完成判定勾选

## §8 实施记录

> 待实施时填写。worktree：`plan-390/actor-state-injection`。

## §9 非目标 / 超出范围

- Tier 3 ask/reply RPC（Plan 387 §9）
- guards 真正执行（VM 自身未接通）
- 多线程抢占调度
- sink→app 的 channel/SSE 类型系统（app 侧工作，auto-ai Plan 021 Phase 6.6）
- 跨仓 driver.at 的 PipelineEvent 转发逻辑（auto-ai 侧，本计划只提供机制）
- **`fn register<T: Tool + 'static>` 泛型方法语法路线（缺口 2）** —— ❌ **明确否决**（2026-08-06 实证）。
  spec-param 机制（`fn register(tool Tool)` → `Box<dyn Tool>`）已达到"传入具名结构自动装箱"的体验目标，
  无需泛型语法。泛型方法 + `'static` lifetime 是 Plan 242 Item #1 / Plan 364 W3 之外的 L 级新工程，
  对缺口 2 属过度设计。缺口 2 的真缺陷（call-site 不自动 Box::new）见 §11 Phase E。

## §10 回 auto-ai 后的衔接

本计划（Plan 390）落地后，auto-ai 侧的收尾工作（Plan 021 §6.7 解除 + Phase 6.6 driver 转发）
**仍在 auto-ai 推进**，不属本计划范围。衔接路径：

1. auto-lang 合并 Plan 390 → 重建 auto.exe
2. 回 auto-ai：`agent.at` EventSink 改 spawn 带参注入 cb（021 §6.7）
3. driver.at 恢复 `Delta/Tool → PipelineEvent` 转发（021 Phase 6.6）
4. retranspile + 端到端验证 → 勾选 021 Phase 6.5/6.6 + 缺口 1 完成判定

auto-ai 021 §6.7 已回标"对应 auto-lang Plan 390"，闭环。

---

## §11 Phase E — 缺口 2：a2r call-site spec 参数自动装箱缺陷修复

> **2026-08-06 追加**。来源 auto-ai Plan 021 缺口 2（`register_tool<T>` 泛型）。本 Phase
> 在 auto-lang worktree 推进 a2r 转译器修复；Phase F 在 auto-ai 侧落地 API（§12）。

### §11.1 调研结论：泛型语法路线否决（spec-param 捷径已够）

Plan 021 缺口 2 原始判断："Auto 无 `<T: Tool>` 泛型方法语法，需 auto-lang 支持"。
**2026-08-06 实证推翻**：a2r 的 spec-param 机制（`fn register(tool Tool)`，`Tool` 是 spec）
已能实现 rust-ref `register<T: Tool>(tool: T) { Arc::new(tool) }` 的**全部人体工学价值**——
调用方写 `register(my_tool)` 即可，无需 `Arc::new(Box::new(...))` 手包箱。

**实证（最小用例，当前 auto.exe 转译）**：

```auto
spec Tool { fn name() str; fn run() }
pub mut fn register(tool Tool) void {
    let n = tool.name()
    let a = Arc(tool)            # Box<dyn Tool> → Arc<Box<dyn Tool>>
    self.tools.set(n, a)
}
type EchoTool as Tool { fn name() str { return "echo" }; fn run() { print("echo!") } }
fn main() { let r = Reg.new(); let t = EchoTool(); r.register(t) }
```

声明侧转译**正确**：
```rust
pub fn register(&mut self, tool: Box<dyn Tool>) {   // ✓ spec Tool → Box<dyn Tool>
    let n = tool.name();
    let a = Arc::new(tool);                          // ✓ Arc(tool) → Arc::new(tool)
    self.tools.insert(n, a);
}
```

**但 call site 不自动装箱**：`r.register(t)` 转译为 `r.register(t)`（无 `Box::new`）→
Rust 报 `expected Box<dyn Tool>, found EchoTool`（E0308）。这就是缺口 2 的**真缺陷**，
与泛型语法无关。详见 §9 非目标（泛型路线否决）。

### §11.2 根因（trans/rust.rs，三处协调缺陷）

| # | 缺陷 | 位置 | 影响 |
|---|---|---|---|
| D-A | `fn_spec_param_indices` 对方法未填充 | prescan `Stmt::Fn`（16476-16493）+ `Stmt::TypeDecl` 方法（16525-16557）——两处都填充了 str/int/struct/param_types/mut 的限定键 `"Type.method"`，**唯独跳过 spec** | 方法调用的 spec 标志查不到 |
| D-B | call-site 的 `spec_flags` 查找不处理方法调用 | 7247-7251：仅匹配 `Expr::Ident(fn_name)`，`Expr::Dot` 方法调用直接落 `else { None }`（对照同文件 `str_flags` 7205-7222 有 `last_seg` 回退，spec 无） | `r.register(t)` 的 `spec_flags` 恒为 None，根本不尝试装箱 |
| D-C | 装箱关闭括号无条件 `.clone()` | 7566-7568：`is_spec_param` 时写 `Box::new(<arg>.clone())`。对 spec-bound ident（已是 `Box<dyn Trait>`）正确；对**具体结构体值**（`EchoTool`）错误——应是 move 而非 clone | 即使 D-A/D-B 修好，具体值仍会因 `.clone()` 路径不对而编译失败/语义错 |

**对照工作范例**：数组元素装箱（9447-9464）写 `Box::new(<elem>)` 无 `.clone()`（pure move），
这是 call-site 应模仿的模式。

### §11.3 修复方案（仅 trans/rust.rs，无 parser/AST/语言变更）

**Fix A — 填充 `fn_spec_param_indices` 的方法限定键**：
- `Stmt::Fn` prescan（16476-16493）：在 int-flags insert（16490）后，仿照计算 `spec_param_flags: Vec<bool> = fn_decl.params.iter().map(|p| matches!(p.ty, Type::Spec(_))).collect()`，`insert(fn_decl.name.clone(), spec_param_flags)`
- `Stmt::TypeDecl` 方法 prescan（16525-16557）：仿 16541-16545（int 的限定键模式），计算 spec flags 后 `insert(qualified_key.clone(), ...)` + `insert(fn_decl.name.clone(), ...)`

**Fix B — call-site `spec_flags` 查找处理方法调用**（7247-7251）：替换为 `str_flags`（7205-7222）的 `last_seg` 回退模式——`Expr::Dot(_, field) => Some(field.as_str())`，再 `self.fn_spec_param_indices.get(seg).cloned()`。

**Fix C — 装箱关闭括号区分 spec-bound ident vs 具体值**（7566-7568）：
```rust
if is_spec_param {
    if let Arg::Pos(Expr::Ident(name)) = arg {
        if self.spec_bound_idents.contains(name) {
            write!(out, ".clone())")?;   // 已是 Box<dyn Trait>，clone 入箱
        } else {
            write!(out, ")")?;            // 具体结构体，move 入箱
        }
    } else {
        write!(out, ")")?;                // 非简单 ident 表达式，move 入箱
    }
}
```
开括号 `Box::new(`（7410-7412）无需改。

### §11.4 实施任务（Phase E，auto-lang worktree）

- [ ] E.1 worktree：`plan-390/actor-state-injection`（与 Phase A–D 同 worktree，§5 Phase A 之前先做 E，因 E 更小且独立）
- [ ] E.2 Fix A：`trans/rust.rs` 两处 prescan 补 `fn_spec_param_indices` 方法键
- [ ] E.3 Fix B：`trans/rust.rs` 7247-7251 `spec_flags` 查找加 `last_seg` 回退
- [ ] E.4 Fix C：`trans/rust.rs` 7566-7568 区分 spec-bound ident / 具体值
- [ ] E.5 新增 a2r 用例 `12_specs/010_spec_param_callsite/spec_param_callsite.at`：
       spec 参数的方法调用 + 具体结构体实参 + spec-bound ident 实参两个场景，期望输出含 `Box::new(arg)`
       （具体值）/ `Box::new(arg.clone())`（spec-bound）黄金对比
- [ ] E.6 回归：`cargo test -p auto-lang` 全绿（含 001-009 spec 黄金逐字节不变）；
       spike 最小用例（§11.1）retranspile → 独立 crate 编译运行 stdout 正确

### §11.5 验收（Phase E）

- [ ] `r.register(my_tool)`（`my_tool: EchoTool` 具体结构体）转译为 `r.register(Box::new(my_tool))`，编译通过
- [ ] spec-bound ident 实参仍走 `.clone()` 路径，行为不变（向后兼容）
- [ ] 现有 spec 测试（12_specs 001-009）零回归
- [ ] `cargo test -p auto-lang` 无新增失败

## §12 Phase F — 回 auto-ai 落地缺口 2 API（auto-ai 侧）

> Phase E（a2r 修复）合并 + 重建 auto.exe 后，auto-ai 侧的 API 落地。

- [ ] F.1 `crates/auto-ai-agent/src/tool.at`：`ext ToolRegistry` 加 `pub mut fn register(tool Tool) void`
       （body：`let a = Arc(tool); self.tools.set(tool.name(), a)`，复用 §11.1 实证形态）
- [ ] F.2 `crates/auto-ai-agent/src/agent.at`：加 `pub mut fn register_tool(tool Tool) void`
       （转发 `self.tools.register(tool)`），对齐 rust-ref 的 `register_tool<T>` 名称
- [ ] F.3 retranspile 0 错；`auto-ai-cli` 的 `agent.register_tool(tools::ReadFile)` 等
       call site 不再需要 `Arc::new(Box::new(...))` 手包箱（缺口 2 完成判定）
- [ ] F.4 勾选 auto-ai Plan 021 Phase 2 + 缺口 2 完成判定
- [ ] F.5 更新 021 §"缺口 2"章节：注明"泛型语法路线否决，改用 spec-param + Phase E call-site 修复"

**注**：存储类型仍是 `Arc<Box<dyn Tool>>`（双层包装，Plan 019/021 已知限制，Deref 链功能可用）。
转正时若需对齐 rust-ref 的单层 `Arc<dyn Tool>`，另立 a2r spec 返回位/存储位推导计划（非本计划范围）。
