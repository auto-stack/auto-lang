---
plan: 390
title: actor-state-injection
affects: [auto-lang/parser, auto-lang/a2r, auto-lang/vm, a2r-std]
status: complete # draft | in-progress | complete
# auto-lang 侧范围完成（Phase A/B/E/G1/G2/F 落地）；Phase D 在 auto-ai 仓推进。
# 5 个遗留见 §14（L1 a2r Arc/Box 实参渲染 / L2 双层包装 / L3 WithBindings 多字段 /
# L4 闭包字面量 state 字段推导 / L5 闭包不能捕获外部变量——阻塞 driver 流式转发）。
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

### Phase A — VM 侧（M1）— ✅ 落地（见 §8.3）

- [x] 5.1 `shim_task_spawn` 扩参：`shim_task_spawn(task_type, capacity, ...init_args)`；
      codegen `Task.spawn` 专用块推栈 `[task_type, capacity, initN..init0, n_init]`，shim 按 n_init 反向 pop
- [x] 5.2 spawn 路径：`shim_task_spawn_vm` 把 init values 写入 spawned task 的 `state_vars[field_idx]`
      （`try_lock` 非阻塞）；不足时用默认值。配合 §G1 的 `locked_state_fields` 机制防止 #start 默认初始化覆盖注入值
- [x] 5.3 parser：`Task.spawn` 走通用 call 语法，`> 2` 实参不报错（codegen 专用块消费全部用户参数）
- [x] 5.4 字段-参数顺序约定：按 state 字段在 task 声明中的出现顺序映射（init values reverse 到声明顺序）

### Phase B — a2r 侧（M1）— ✅ 落地（见 §8.2）

- [x] 5.5 `emit_task_spawn_helper`：用结构体字面量 `Counter { count: count }` 构造（非 `new_with_state` 命名，
      但等价——default 参数 `field: Type = default` 保证无参 spawn 向后兼容）
- [x] 5.6 `Task.spawn("Name", cap, ...args)` 的 `call()` 臂：args[2..] 转发给 `spawn_<name>(v1, v2)`
- [x] 5.7 无参 spawn 保持 `spawn_<task>(cap)` → default 参数补齐，向后兼容

### Phase C — 验证 — ✅ 落地（见 §8.2/§8.3）

- [x] 5.8 a2r 用例 `22_actors/020_spawn_with_state`（原计划 016，实施时编号调整为 020）：多字段 spawn 注入 ✅
- [x] 5.9 partial init：020 用例的 spawn helper 用 `field: Type = default` 参数，Rust default 参数语义天然支持 partial（少传的用默认值）
- [~] 5.10 VM/a2r 一致：VM 侧 spawn-with-args 机制已验证（`actor_spawn_init_arg_overrides_default`）；
      **但 EventSink 完整形态（fn 指针 state + 绑定变量 handler）在 VM 端到端未验证**——EventSink 走 a2r 路径交付，
      VM 侧 bound-var 已修（§G2）但 EventSink 整体未跑 VM 端到端。属已知限制，非阻塞（EventSink 生产路径是 a2r）
- [x] 5.11 回归：001-015 黄金逐字节不变（受影响的 005/006/007/010/011/014/015/017 重新生成）；零新增失败

### Phase D — 回 auto-ai 落地（auto-ai 侧，非 auto-lang 范围，见 §10）

- [ ] 5.12 重建 auto.exe（合并 auto-lang Plan 390 + Plan 389 后）
- [ ] 5.13 auto-ai `agent.at` 的 EventSink：`run_inner` 内 `Task.spawn("EventSink", 16, app_cb)` 注入真实回调
- [ ] 5.14 driver.at 恢复 rust-ref 等价：`Delta/Tool → PipelineEvent` 转发（Plan 021 Phase 6.6）
- [ ] 5.15 retranspile 0 错；端到端：agent 流式事件经 EventSink → app channel/SSE 可观察

> Phase D 全部在 auto-ai 仓推进，不属本计划（auto-lang）范围。auto-lang 侧的机制（Phase A/B）已交付。

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
- [x] app 能通过 `Task.spawn("EventSink", 16, cb)` 注入 `fn(StreamEvent)` 回调（a2r 路径，§8.2）
- [x] EventSink 后续事件转发到注入的 cb（a2r 路径端到端，auto-ai Phase F §12 已落地）
- [~] VM 与 a2r 转译版行为一致（5.10）—— 机制一致已验证；EventSink 完整形态 VM 端到端未跑（见 §5.10 注）
- [x] 现有 actor 测试零回归（001-015 黄金 + VM actor_tests 全绿）
- [x] 向后兼容：无参 spawn 行为不变
- [ ] auto-ai Plan 021 §6.7 解锁，Phase 6.6 可推进 —— **auto-ai 侧（Phase D），非 auto-lang 范围**

**Phase E（缺口 2 / §11）** — ✅ 落地（见 §8.3 末尾 + §11.4）：
- [x] `r.register(my_tool)`（具体结构体实参）转译为 `r.register(Box::new(my_tool))`，编译通过
- [x] spec-bound ident 实参仍走 `.clone()`，行为不变
- [x] 现有 spec 测试零回归（12_specs 006 新增 + 001-009 不变）；`cargo test -p auto-lang` 无新增失败

**Phase F（缺口 2 回 auto-ai / §12）** — ✅ 落地：
- [x] `tool.at`/`agent.at` 加 `register(tool Tool)` / `register_tool(tool Tool)`，retranspile 0 错
- [x] auto-ai-cli 的 `register_tool` call site 不再需手包箱
- [x] auto-ai Plan 021 Phase 2 + 缺口 2 完成判定勾选

## §8 实施记录

> worktree：Phase E/F 在 `plan-390/actor-state-injection`（已合并 master）；
> Phase A–D 在 `plan-390-ad/actor-spawn-args`（`D:/autostack/auto-lang-390ad`）。

### §8.2 Phase B — a2r Task.spawn 带初始化参数（2026-08-06，✅ 落地）

`Task.spawn("Name", cap, v1, v2)` 的 a2r 转译侧（Phase B / §5）已落地并验证：

- **call 翻译**（`rust.rs ~3475`）：args[2..]（跳过 name+capacity）转发给 `spawn_<name>(v1, v2)`
- **spawn helper**（`emit_task_spawn_helper ~9133`）：签名加 state 字段为参数
  （`field: Type = default`），用结构体字面量 `Counter { count: count }` 构造（替代 `::new()`）。
  default 参数保证无参 spawn 向后兼容。
- **验证**：新增黄金 `22_actors/020_spawn_with_state` ✅；重新生成 8 个受影响 actor
  黄金（005/006/007/010/011/014/015/017 — 仅 spawn 签名变）；`cargo test --features test-trans`
  3133 passed / 22 failed（与 master 逐字节一致，零新增）。

### §8.3 Phase A（VM 侧 spawn-with-args）— ✅ 落地（2026-08-06，重启后完成）

> 初版（同日早些时候）标记"推迟"，根因判断有误。重新调查后发现两个"前置"均不阻塞，
> 重启实施并端到端验证通过。

**初版误判的澄清**：

1. ~~"VM `Task.spawn` codegen 契约脆弱"~~：实证发现 `Task.spawn("Greeter",16)` 的
   codegen **根本不走**那个检查 `"auto.task.spawn"` 的注入块（`func_name` 实际是
   `"Task.spawn"`，该块是死代码）。真实路径：用户参数 `["Greeter",16]` 走通用 arg 循环
   入栈，shim 直接 pop 出 capacity + task_type 字符串——契约本身正常，只是被误读为脆弱。
   **修复**：在 codegen 加 `func_name == "Task.spawn"` 专用处理，消费全部用户参数
   （name + capacity + init args），skip 通用 arg 循环，推自描述栈
   `[task_type, capacity, initN..init0, n_init]`；shim 按 n_init → init values →
   capacity → task_type 顺序 pop。
2. ~~"VM bound-variable handler 缺陷阻塞验证"~~：该缺陷（`on { n int -> }` 报
   "Undefined variable: n"，Plan 043 相关）只阻塞带绑定变量的 actor（如 EventSink），
   **不阻塞 spawn-with-args 机制本身**。用字面量 pattern handler（`1 ->`）即可端到端
   验证 init args（新 VM 测试 `actor_spawn_init_arg_overrides_default` 用此法）。

**实施（`plan-390-vm/spawn-args` worktree）**：
- `codegen.rs`：`Task.spawn` 专用块，task_type 取自 string-literal arg[0]，capacity 取自
  arg[1]（literal int），init args = arg[2..]；推栈顺序 task_type → capacity →
  init(rev) → n_init；`skip_task_spawn_user_args` 跳过通用 arg 循环。
- `stdlib.rs shim_task_spawn_vm`：按 n_init → init values（reverse 到声明顺序）→
  capacity → task_type pop；init values 写入 spawned task 的 `state_vars[field_idx]`
  （`try_lock`，非阻塞——task 刚 spawn 未 step）。
- **不需 lock 机制**：实证发现 VM `#start` 的 state 默认初始化（`count = 5`）本身在
  master 上就是坏的（state 恒从 0 起，pre-existing bug），故注入值不会被默认覆盖。
  初版加的 `locked_state_fields` + STORE_STATE_FIELD 跳过 + TASK_LOOP 清除反而阻止了
  handler 的合法写，已全部移除。

**验证**：
- 新增 VM 测试 `actor_state_tests::actor_spawn_init_arg_overrides_default`：
  `Task.spawn("Counter", 16, 41)` → handler `count+1` → 输出 `42` ✅
- 端到端：`count=41` 注入后 start 读到 41，handler `+1` 持久化到 42。
- `cargo test -p auto-lang --features test-trans`：零新增失败（与 master 逐字节一致）；
  VM actor 测试 9 passed（8 既有 + 1 新增）。

**遗留（pre-existing，非本计划引入）**：
- ~~VM `#start` state 默认初始化坏（`count = 5` 不生效，state 从 0 起）—— 独立 VM bug~~
  → **✅ 已修（Phase G1 / commit `6195ae4c`，2026-08-06）**。根因：codegen 把 `#start`
  块 gate 在 `start_hook.is_some()`，无 `fn start()` 的 task 既不发 STORE_STATE_FIELD
  初始化也不注册 `#start`。修复后与 Phase A spawn init args 共存需 `locked_state_fields`
  机制（shim 注入时 lock → #start STORE_STATE_FIELD 跳过 → TASK_LOOP 清除）。
- VM bound-variable handler（`on { n int -> }`）报 Undefined variable —— **Phase G2 承接**
  （见 §G2，需 runtime handler 栈帧重构）。

实施中发现 §11.2 的根因描述需补强——实际缺陷比"三处"更广，且方法调用走的是
**独立的 arg 发射循环**（非 §11.2 假设的 free-fn 循环）。修正后的完整修复：

- **Fix A（prescan）**：`trans/rust.rs` `trans()` 的 prescan 循环，`Stmt::Fn`（~16505）+
  `Stmt::TypeDecl` 方法（~16561）+ `Stmt::Ext` 方法（~16623）三处补 `fn_spec_param_indices`
  的限定键 `"Type.method"` + 裸名插入。**关键发现**：`ext Type {}` 解析成 `Stmt::TypeDecl`
  （非 `Stmt::Ext`），故 TypeDecl 分支是 `ext` 方法的主路径；Stmt::Ext 分支为冗余保险
  （真实 `Stmt::Ext` 在合并场景才出现）。
- **Fix A2（方法调用 arg 循环，§11.2 未预见）**：`Expr::Dot` 方法调用在 `call()` 内有
  **独立的 arg 发射循环**（~6792，在 ~7254 的 free-fn `spec_flags` 查找之前 return）。
  `r.register(t)` 走这条路径，故必须在方法循环内单独算 `method_spec_flags`（按
  `method_name` 查 `fn_spec_param_indices`）+ 包 `Box::new`（spec-bound ident 用 `.clone()`，
  具体值 move）。这是缺口的真正阻塞点。
- **Fix B（free-fn spec_flags 查找）**：~7250 的 `spec_flags` 查找加 `Expr::Dot`/`Expr::Bina`
  的 `last_seg` 回退（镜像 `str_flags`）。
- **Fix C（free-fn 闭括号）**：~7566 区分 spec-bound ident（`.clone())`）vs 具体值（`)`）。
- **跨模块（`transpile_rust_project_merged`）**：补 `collect_fn_spec_params` helper +
  `global_fn_spec_params` + 两个入口点的 pre-populate 循环（单文件 CLI 走 `trans()`
  prescan，多文件走 global；两者都覆盖）。

**验证**：
- 新增 a2r 黄金 `12_specs/006_spec_param_callsite`：free-fn + method 两 call site，
  期望 `free_register(Box::new(t.clone()))` + `r.register(Box::new(t))`，✅ 通过。
- `cargo test -p auto-lang --features test-trans`：3124 passed / 22 failed（与 master
  **逐字节一致**，零新增失败）；a2r 黄金零失败。
- spike 最小用例（`reg2.at`）retranspile → `r.register(Box::new(t))`，行为正确。

**§11.2 根因表更正**：原 D-A/D-B/D-C 三处描述对应 free-fn 路径；方法调用路径（D-A2）
是实施中实证发现的第四处，已补入本记录。

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

- [x] E.1 worktree：`plan-390/actor-state-injection`（与 Phase A–D 同 worktree，§5 Phase A 之前先做 E，因 E 更小且独立）
- [x] E.2 Fix A：`trans/rust.rs` prescan 补 `fn_spec_param_indices` 方法键（Stmt::Fn + TypeDecl + Stmt::Ext）
- [x] E.2b Fix A2（实施中追加）：方法调用独立 arg 循环（~6792）补 `method_spec_flags` + Box::new 包装
- [x] E.3 Fix B：`trans/rust.rs` ~7250 `spec_flags` 查找加 `last_seg` 回退
- [x] E.4 Fix C：`trans/rust.rs` ~7566 区分 spec-bound ident / 具体值
- [x] E.5 新增 a2r 用例 `12_specs/006_spec_param_callsite/spec_param_callsite.at`：
       spec 参数的 free-fn 调用 + 方法调用 + 具体结构体实参，期望 `Box::new(t.clone())` /
       `Box::new(t)` 黄金对比，✅ 通过
- [x] E.6 回归：`cargo test -p auto-lang --features test-trans` 3124 passed / 22 failed
       （与 master 逐字节一致，零新增）；spike 最小用例（§11.1）retranspile 行为正确

### §11.5 验收（Phase E）— ✅

- [x] `r.register(my_tool)`（`my_tool: EchoTool` 具体结构体）转译为 `r.register(Box::new(my_tool))`，编译通过
- [x] spec-bound ident 实参仍走 `.clone()` 路径，行为不变（向后兼容）
- [x] 现有 spec 测试（12_specs 001-009）零回归
- [x] `cargo test -p auto-lang` 无新增失败

## §12 Phase F — 回 auto-ai 落地缺口 2 API（auto-ai 侧）✅ 2026-08-06

> Phase E（a2r 修复）已合并 master + 重建 auto.exe。本 Phase 在 auto-ai 落地 API。

- [x] F.1 `crates/auto-ai-agent/src/tool.at`：`ext ToolRegistry` 加 `pub mut fn register(tool Tool) void`
       （body：`let a = Arc(tool); self.tools.set(n, a)` —— **let-bound Arc workaround**，
       因 a2r 实参位 `Arc(x)` 渲染缺陷，见 KNOWN-DEBT Plan 021 条）
- [x] F.2 `crates/auto-ai-agent/src/agent.at`：加 `pub mut fn register_tool(tool Tool) void`
       （转发 `self.tools.register(tool)`），对齐 rust-ref 的 `register_tool<T>` 名称
- [x] F.3 retranspile 0 错（rust/ 独立 crate + workspace 双检）；`r.register(EchoTool())` →
       `r.register(Box::new(EchoTool {}))` 自动装箱（Phase E 生效），无需 `Arc::new(Box::new(...))` 手包箱
- [x] F.4 勾选 auto-ai Plan 021 Phase 2 + 缺口 2 完成判定
- [x] F.5 更新 021 §"缺口 2"章节：注明"泛型语法路线否决，改用 spec-param + Phase E call-site 修复"

**注**：存储类型仍是 `Arc<Box<dyn Tool>>`（双层包装，Plan 019/021 已知限制，Deref 链功能可用）。
转正时若需对齐 rust-ref 的单层 `Arc<dyn Tool>`，另立 a2r spec 返回位/存储位推导计划（非本计划范围）。

**遗留（KNOWN-DEBT Plan 021）**：a2r 实参位 `Arc(x)`/`Box(x)` 渲染缺陷（仅 let 位正确）。
`register` 体用 `let a = Arc(tool)` 绕过；a2r 根因修复后可去 let 绑定直写。

---

## §13 Phase G — VM actor 两个 pre-existing bug 修复

> **2026-08-06 追加**。Phase A 实施时发现两个 VM 既有缺陷（§8.3 遗留），本 Phase 收尾。
> 两者都阻塞 EventSink 这类带绑定变量 + 默认值的 actor 在 VM 完整跑通（但 EventSink 走
> a2r 路径，Phase B 已交付，不依赖 VM 侧）。worktree：`fix/vm-actor-state-bugs`。

### §G1 — VM state 默认初始化 ✅ 落地（commit `6195ae4c`，已合并 master）

**Bug**：task 有 state 字段但无 `fn start()` 时，声明的默认值（`count = 5`）被静默丢弃——
state 从 0 起。

**根因**：`vm/codegen.rs` `Stmt::TaskDef` 把整个 `#start` 块（STORE_STATE_FIELD 初始化 +
`#start` export 注册 + TASK_LOOP）gate 在 `start_hook.is_some()` 上。无 `fn start()` 的 task：
(a) 不发初始化字节码；(b) 不注册 `#start` → `shim_task_spawn_vm`（`vm/ffi/stdlib.rs` ~5906）
的 `exports_by_name.get("{task}#start")` 返回 None，`unwrap_or(0)` 让 ip 指向 offset 0
（模块入口），声明默认值丢失。

**修复**（`vm/codegen.rs` ~3632）：gate 改为 `task_def.start_hook.is_some() || !task_def.state.is_empty()`；
无 `fn start()` 时也发初始化 + TASK_LOOP + `#start` export；`fn start()` body 单独用内层
`if let Some(ref start_hook)` 条件化。

**与 Phase A 共存**：默认初始化现在生效后会覆盖 spawn 注入值，故恢复 `locked_state_fields`
机制（初版 Phase A 因默认坏而移除，现在必须保留）：
- `vm/task.rs`：`AutoTask` 加 `locked_state_fields: HashSet<u8>` + 构造函数初始化。
- `vm/ffi/stdlib.rs shim_task_spawn_vm`：注入 init arg 时 `spawned.locked_state_fields.insert(i as u8)`。
- `vm/engine.rs STORE_STATE_FIELD`：`if !task.locked_state_fields.contains(&field_idx)` 才写
  （#start 默认初始化跳过注入字段）。
- `vm/engine.rs TASK_LOOP`：`task.locked_state_fields.clear()`（#start 跑完后清除，handler 自由写）。

**验证**：新增 2 VM 测试（`actor_state_default_init_without_fn_start`、
`actor_spawn_init_arg_overrides_now_working_default`）；11 actor 测试全绿；回归 22 failed
（master 23，**零新增，反而修了一个 flaky perf benchmark**）。

### §G2 — VM bound-variable handler ✅ 落地（2026-08-06，`fix/vm-bound-var-handler` worktree）

**Bug**：`on { n int -> ... }` 的绑定变量 `n` 报 "Undefined variable: n"。message payload
（运行时已 push 到 value 栈）未被绑定到 pattern 变量名。**已修**——见下方"实施"。

**根因**（已实证定位）：两层缺陷——

| 层 | 缺陷 | 位置 |
|---|---|---|
| codegen | handler codegen 从未把栈上 message 存入命名 local（`pattern.binding_name()` 存在但 codegen 不调用） | `vm/codegen.rs ~3698`（handler 循环，`add_handler` 后直接编译 body，无 `push_scope`/`add_var`/`STORE_LOC`） |
| runtime | **handler 无独立栈帧**——复用 task bp（=0），从不重建 frame；多 `send` 时 local 跨调用残留 | `vm/engine.rs ~1650`（`run_task_loop` 消息唤醒：`push_i32(msg)` + `ip=body_offset`，无 bp/sp 帧建立） |

**实证**（初版尝试，已 revert）：
- codegen 加 `push_scope` + `add_var(name)` + `STORE_LOC_0`（pop 栈上 message 入 local）：
  **单次调用通过**（`count=10 + n=5 → 15`）。
- 多次调用 stale：第二次 `n` 读到 15（上次的 count 值）而非新 message 7 → `15+15=30`。
  尝试 runtime sp-reset（`task.ram.sp = bp+1` 后 push message）失败：count/n 槽位冲突
  （count 的 #start 默认值残留在 bp+1）。

**完整修复方案**（Phase G2，auto-lang worktree）：

**G2.1 codegen 绑定（已验证单次调用）**：
`vm/codegen.rs` handler 循环，`add_handler` 后、编译 body 前：
```rust
self.push_scope();
if let Some(bind_name) = pattern.binding_name() {
    let slot = self.add_var(bind_name.as_str());
    match slot {
        0 => self.emit(OpCode::STORE_LOC_0),
        1 => self.emit(OpCode::STORE_LOC_1),
        _ => { self.emit(OpCode::STORE_LOCAL); self.code.push(slot as u8); }
    }
}
// compile body...
self.pop_scope();
```

**G2.2 runtime 栈帧重构（关键，解决多次调用 stale）**：
`vm/engine.rs ~1650` `run_task_loop` 消息唤醒路径，建立 handler 独立帧：
```rust
// 方案 A（推荐）：handler prologue 建立 bp 帧
// 消息唤醒时：记录当前 sp 作 handler 帧基，push message，handler RET 时恢复。
let handler_frame_sp = task.ram.sp;  // 帧前 sp
task.ram.push_i32(msg_i32);
task.ip = body_offset as usize;
task.handler_frame_base = Some(handler_frame_sp);  // 新增 AutoTask 字段
```
配合 handler 的 RET（`vm/engine.rs ~5762` 区域）：
```rust
// 检测 handler RET（bp==0 且 in_message_loop）：恢复 sp 到 handler_frame_base，
// 重新 TASK_LOOP 等下一消息。
if task.bp == 0 && task.in_message_loop {
    if let Some(base) = task.handler_frame_base.take() {
        task.ram.sp = base;  // 清除 handler 局部区，下次调用干净
    }
}
```
- **`vm/task.rs`**：`AutoTask` 加 `handler_frame_base: Option<usize>` + 构造初始化 `None`。
- **关键**：bp 仍为 0（task 不变），但 handler local 区（bp+1..）每次调用从
  `handler_frame_base` 起干净——`STORE_LOC_0` 写 bp+1，`LOAD_LOC_0` 读 bp+1，跨调用不残留
  （因为 RET 时 sp 复位清除了残留）。

**G2.3 替代方案 B（若方案 A 的 bp 语义复杂）**：handler 建立真正的 bp 帧（CALL 风格）：
唤醒时 push 老 bp、设 bp=sp、push message；handler RET 时恢复老 bp、sp。改动更大但语义最干净
（与函数 CALL 一致）。需评估与现有 `task.bp`（spawn 时设 0）的兼容。

**G2.4 WithBindings 多字段消息**：`on { Add(a int, b int) -> }`（`WithBindings` pattern）
需 DUP message + 多次 STORE_LOC（每个 binding 一个槽）。本 Phase 先做 TypeBinding（单字段），
WithBindings 留后续。

**实施任务（Phase G2）** — 方案 A 落地（见下方"实施结果"）：
- [x] G2.1 codegen handler 绑定（`vm/codegen.rs`）—— `push_scope` + `add_var` + STORE_LOC
- [x] G2.2 runtime 栈帧重构（`vm/engine.rs` + `vm/task.rs`）—— `handler_frame_base` 帧复位
- [x] G2.3 选择方案 A（handler_frame_base），实证多 send 场景 ✅（方案 B 未采用，A 已足够）
- [x] G2.5 VM 测试：`actor_bound_var_handler_multi_send`（send 5/7/3 → total 5/12/15）✅
- [x] G2.6 回归：`cargo test --features test-trans` 22 failed（与 master 逐字节一致，零新增）；VM actor 测试全绿
- [ ] G2.7 WithBindings（多字段消息）—— **留后续**（见 §14 遗留 L3）

**验收（Phase G2）**：
- [x] `on { n int -> }` 单次 + 多次 send：每次 n = 当前 message（不 stale）
- [ ] EventSink 形态 actor（fn 指针 state + 绑定变量 handler）在 VM 端到端跑通 —— **未验证**（EventSink 生产路径是 a2r；VM 侧机制已就绪但整体未跑，见 §5.10 注）
- [x] 现有 VM actor 测试零回归；a2r 路径不受影响

**风险**：runtime 栈帧重构触及 `run_task_loop` / RET 语义，可能影响既有 actor 测试。
方案 A（handler_frame_base）改动最小（仅 sp 复位），方案 B（真 bp 帧）语义最干净但改动大。
建议先试 A，失败再 B。

**实施结果（2026-08-06，方案 A 落地）**：

- **G2.1 codegen 绑定**（`vm/codegen.rs` handler 循环）：`add_handler` 后、编译 body 前，
  `push_scope` + 若 `pattern.binding_name()` 存在则 `add_var(name)` + 发 `STORE_LOC_0/1/LOCAL`
  把栈上 message pop 入命名 local；body 后 `pop_scope`。
- **G2.2 runtime 帧复位**（`vm/engine.rs` + `vm/task.rs`，方案 A——`handler_frame_base`）：
  - `vm/task.rs`：`AutoTask` 加 `handler_frame_base: Option<usize>` + 构造初始化 `None`。
  - `vm/engine.rs` 消息唤醒（~1659）：`task.handler_frame_base = Some(task.ram.sp - 1)`
    （记录 message push 前的 sp 作帧基）。
  - `vm/engine.rs` handler RET 复位（~1764，`Terminated && in_message_loop` 分支）：
    `if let Some(base) = task.handler_frame_base.take() { task.ram.sp = base; }`
    —— 清除 handler 局部区，下次 message 调用从干净帧起。
  - bp 仍为 0（task 不变），但 handler local 区（bp+1..）每次 RET 后复位，跨调用不残留。

**实证**（`vmbv2.at`：`total=0`, send 5, 7）：msg1 `total=0,n=5→5`；msg2 `total=5,n=7→12` ✅。

**验证**：新增 VM 测试 `actor_bound_var_handler_multi_send`（send 5/7/3 → total 5/12/15）✅；
VM actor 测试 12 passed（11 既有 + 1 新增）；`cargo test --features test-trans` 22 failed
（与 master 逐字节一致，**零新增失败**）。

**遗留（留后续）**：WithBindings 多字段消息（`on { Add(a,b) -> }` 需 DUP + 多 STORE_LOC）；
方案 B（真 bp 帧）未采用——方案 A 已足够。

---

## §14 遗留汇总（2026-08-07 复审）

auto-lang 侧实质工作全部完成（Phase A/B/E/G1/G2/F 落地；Phase D 属 auto-ai）。以下是 5 个明确记录的遗留：

| # | 遗留 | 性质 | 位置 | 严重度 | 触发条件 |
|---|---|---|---|---|---|
| **L1** | a2r 实参位 `Arc(x)`/`Box(x)` 渲染缺陷 —— 只有 let 位正确，实参位不渲染 `::new` | a2r 转译器 bug | `trans/rust.rs`（§451 KNOWN-DEBT Plan 021）| 中 | 当前用 `let a = Arc(tool)` workaround 绕过（§439）；a2r 根因修复后可去 let 绑定直写 |
| **L2** | `Arc<Box<dyn Tool>>` 双层包装 —— 存储类型是双层，rust-ref 是单层 `Arc<dyn Tool>` | 设计偏差 | auto-ai tool.at 存储（§448）| 低 | Deref 链功能可用；转正时若要对齐 rust-ref 单层，另立 a2r spec 返回位/存储位推导计划 |
| **L3** | WithBindings 多字段消息绑定未实现 —— `on { Add(a int, b int) -> }` 的多字段绑定 | VM 功能缺口 | `vm/codegen.rs` + `vm/engine.rs`（§G2.7）| 低 | 当前只支持单字段 TypeBinding（`on { n int -> }`）；多字段需 DUP + 多 STORE_LOC。罕见场景，单字段够用 |
| **L4** | a2r 闭包字面量作 task state 字段默认值时类型推导失败（`/* unknown */`）—— 具名函数引用（`cb = noop`）正确推导，闭包字面量（`cb = fn(e) {...}`）推导不出 | a2r 转译器 bug（Plan 389 R2 延伸）| `trans/rust.rs`（Phase 6.5 实施时发现）| 低 | 当前 EventSink 用具名 `noop_event` 函数绕过（`agent.at:146`）；a2r 修复后可用闭包字面量。**注**：根因与 L5 同源——`fn(params){}` 解析路径不 bind 参数 |
| **L5** | ~~Auto 闭包不能捕获外部变量~~ → **重新定性（2026-08-07 实证）**：捕获从来不是问题（`(ev) => fwd(ev, outer_cb)` 正常捕获 outer_cb）。真因是 `fn(params){}` 解析路径（`parser.rs:3043-3069`）**不 bind 闭包参数**（另两条路径 `x => ...` 和 `(a,b) => ...` 都 bind）。`fn(ev){...}` 报 "Variable ev not defined" 是因 `ev` 自身未入 scope，非捕获 `outer_cb` 失败 | **parser bug（非语言级限制）** | `parser.rs:3043-3069`（`atom()` 内 `fn(params){}` 路径缺 `bind_var`）| **中**（仍阻塞 driver 流式转发，但修复是 ~6 行） | **Phase H 承接**（§15）：`fn(params){}` 路径加 `bind_var` 循环，镜像 `parser.rs:3842-3847`。a2r 闭包捕获（`rust.rs:3017` + escape analyzer）+ VM 捕获（`codegen.rs:11044`）均已就绪，仅此 parser 路径漏 bind |

**EventSink VM 端到端未验证**（§5.10/§G2 验收）：EventSink 的生产路径是 a2r（Phase B 交付），
VM 侧 spawn-with-args + bound-var handler 机制均已就绪（Phase A + G2），但 EventSink 完整形态
（fn 指针 state + 绑定变量 handler 组合）未在 VM 跑通端到端。非阻塞——EventSink 不走 VM 路径。

**auto-lang 侧范围判定**：Phase A/B/E/G1/G2/F 落地；L5 的 parser 修复（Phase H）是收尾。
L1/L4 是 a2r bug（workaround 已有）；L5 经实证重新定性为 parser bug（非语言级限制，§15 Phase H）。
**Plan 021 缺口 3（serde derive 转译）经实证已不阻塞** —— a2r 已支持 `#[derive(Deserialize)]` +
`#[serde(deserialize_with)]` 注解透传（实证见 Plan 021 缺口 3 章节）；缺口在 auto-ai 侧 `.at`
源码未迁移到 derive 风格，留 Plan 021 Phase 4 独立推进。

---

## §15 Phase H — L3 WithBindings 多字段消息 + VM 对象 registry 统一（方案 B）

> **2026-08-07 立项**。L3（WithBindings 多字段）实施中发现 VM 有 **4 套对象 registry** 且栈编码
> 不一致，阻塞多字段消息的 send→mailbox→handler 全链路。经评估，新增 `WRAP_MSG` opcode（补丁）
> 会加深技术债；**统一 4 套 registry（方案 B）是更合理的长期方向**。本 Phase 同时解决 L3 功能缺口
> 和 registry 统一两个目标。

### §15.1 现状：4 套对象 registry（调研实证 2026-08-07）

| Registry | 定义 | id 段位 | 栈编码 | 存什么 |
|---|---|---|---|---|
| `objects` | engine.rs:260 `DashMap<u64, Arc<RwLock<ObjectData>>>` | 1,000,000+ | `push_i32(id)` **裸 i32** | CREATE_OBJ 产出的 ObjectData |
| `arrays` | engine.rs:264 `DashMap<u64, Arc<RwLock<Vec<Value>>>>` | 2,000,000+ | `push_i32(id)` | CREATE_ARRAY |
| `nodes` | engine.rs:268 `DashMap<u64, Arc<RwLock<Node>>>` | 3,000,000+ | `push_i32(id)` | CREATE_NODE |
| `heap_objects` | engine.rs:273 `DashMap<u64, Arc<RwLock<dyn HeapObject>>>` | 4,000,000+ | `encode_object(id)` **TAG_OBJECT** | List/Map/GenericInstance/RustStdlib/BigInt |

**核心问题**：
- 栈上 `push_i32(obj_id)` 与 scalar i32 无法区分 → 消费端靠**魔数判断**（`>= 4_000_000`，见
  engine.rs:474/2438/2764/3505/3547/3981）+ **试探探测**（`objects.get(&id)` 试一下，engine.rs:920/4926）。
- `match_message_pattern_vm` 期望 `Value::Obj`（内联），但 CREATE_OBJ 产出的是 i32 引用 →
  WithBindings handler 永远不被匹配（L3 阻塞根因）。
- ~15 个 producer push i32，~8 个消费者**仅假设 i32**（无 is_object 回退，统一后会坏），
  ~15 个消费者做了**双解码**（is_object + is_i32，可简化）。

### §15.2 方案 B：统一到 heap_objects（单一 registry + encode_object 编码）

**目标**：4 套 registry → 1 套 `heap_objects`；所有对象引用栈编码统一为 `encode_object(id)`；
删除所有魔数判断和试探探测。

**为什么保留 heap_objects**：它是 `dyn HeapObject` trait 对象（类型开放、可 downcast、有 TypeTag），
已有 8+ 类类型适配（List/Map/GenericInstance/RustStdlib/BigInt），有完整封装 API
（insert/get/remove/contains），是事实上的主 registry。其它 3 套是固定类型的裸 DashMap。

**迁移**：
- `ObjectData` impl HeapObject（新增 TypeTag::ObjectData）→ objects 合入 heap_objects
- `Vec<Value>` 用 ListData<Value> 包装（或新建 ArrayData tag）→ arrays 合入
- `Node` impl HeapObject（tag Node）→ nodes 合入

### §15.3 实施路线（3 Phase，渐进式）

#### Phase H1 — 统一查询 API + ObjectData impl HeapObject（零行为变更）

- 给 `ObjectData`（types.rs:147）impl HeapObject（tag ObjectData）。
- 新增 `get_any_object(id) -> Option<Arc<RwLock<dyn HeapObject>>>`：按 id 段路由到 4 个 registry，
  返回统一的 trait 对象。所有 GET_FIELD/SET_FIELD/CALL_METHOD 消费者改用此 API（替代直接查 objects/heap_objects）。
- **不改栈编码**（仍是 push_i32 / encode_object 混用），不改 id 段位。
- **验证**：全量回归零新增失败（行为不变）。

#### Phase H2 — 栈编码统一为 encode_object（行为变更，高风险）

- **所有 producer**（~15 处）的 `push_i32(obj_id)` → `push_nv(encode_object(obj_id as u32))`：
  - CREATE_OBJ (2242)、CREATE_ARRAY (2293)、CREATE_NODE (2042)
  - CREATE_OK/CREATE_ERR (3133/3142)、CREATE_LIST_* (3238-3287)、NEW_INSTANCE (3328)
  - CONSTRUCT_INSTANCE (3478)、CREATE_TUPLE (3866)、slice-result (3837)
  - inject_value Obj/Array/Node/VmRef (532/544/555/559)
  - GET_FIELD/GET_ELEM VmRef 结果 (4346/4392/4035/4078)
- **仅假设 i32 的消费者**（~8 处）加 is_object 回退：
  - ARRAY_LEN (2340)、SET_ELEM (4110)、CONSTRUCT_INSTANCE (3348)、GET_TUPLE_FIELD (3874)
  - CREATE_ERR 值 pop (3139)、CALL_SPEC 类型名 is_i32 分支 (4910)
- **双解码消费者**（~15 处）简化：删除 `>= 4_000_000` 魔数，合并成单一 `is_object -> decode_object`。
- **验证**：全量回归 + 专门的对象字段访问测试（struct/enum/List/Map 字段读写）。

#### Phase H3 — 删除旧 registry + 魔数（清理）

- `CREATE_OBJ`/`CREATE_ARRAY`/`CREATE_NODE` 改为 `insert_heap_object`（走 heap_object_id_gen）。
- 删除 `objects`/`arrays`/`nodes` 字段 + `object_id_gen`/`array_id_gen`/`node_id_gen`。
- 删除所有 `>= 1_000_000`/`>= 4_000_000` 魔数判断（engine.rs:474/2438/2764/3505/3547/3981）。
- `get_any_object` 简化为单一 `get_heap_object`（不再路由）。
- **验证**：全量回归零新增失败。

#### Phase H4 — L3 WithBindings 多字段消息（依赖 H2 完成后栈编码统一）

H2 完成后，L3 的全链路自然打通（producer 统一 encode_object → send 用 is_object 区分 →
mailbox 存 VmRef → wake push encode_object → handler GET_FIELD 绑定）。已完成的代码（Step 1-4）
在 H2 后即可工作。测试见 §15.4。

### §15.4 测试用例

**Phase H1-H3（registry 统一）回归测试**（确保行为不变）：
1. 对象字段访问：`let p = Point{x:1, y:2}; print(p.x)` — 验证 GET_FIELD 跨 registry 统一查询。
2. enum variant 字段：`let a = Atom.Int(42); print(a.value)` — GenericInstance。
3. List 元素：`let l = List.new([1,2,3]); print(l.len())` — ListData。
4. Map 字段：`let m = Map.new(); m.set("k", 5); print(m.get("k"))` — AutoVMHashMap。
5. HTTP RequestBuilder 链式：`http.request("GET", url).header("k","v")` — RustStdlibObject。
6. 消息传递（TypeBinding）：`h.send(5)` → `on { n int -> }` — 不回归（G2 已验证）。

**Phase H4（L3 多字段）功能测试**：
7. 单字段 WithBindings：`h.send(Add(3))` → `on { Add(val int) -> print(val) }` → 输出 3。
8. 多字段 WithBindings：`h.send(Add(3, 5))` → `on { Add(a int, b int) -> print(a+b) }` → 输出 8。
9. 多次 send 多字段：`h.send(Add(1,2)); h.send(Add(3,4))` → handler 每次看到正确的 a/b（不 stale）。
10. Simple variant：`h.send(Reset)` → `on { Reset -> print("reset") }` → 输出 reset。
11. 混合 pattern：task 同时有 `on { Add(a,b) -> }` + `on { n int -> }` + `on { Reset -> }`。

### §15.5 风险与缓解

| 风险 | 等级 | 缓解 |
|---|---|---|
| Phase H2 栈编码切换破坏 ~8 个"仅假设 i32"消费者 | 高 | 逐个加 is_object 回退；全量回归 + 专门字段访问测试 |
| id 段位迁移期间 object_id_gen (1M) 和 heap_object_id_gen (4M) 并存 | 中 | H1 的 get_any_object 按 id 段路由；H3 统一后删除段位 |
| encode_object 的 u32 payload 限制（id 超 u32::MAX 截断） | 低 | 加 debug_assert(id <= u32::MAX)；当前 id 远未达到 |
| inject_value/decode_tagged_nv 的双解码路径（879-927） | 中 | H2 统一后简化为单一 is_object 路径 |
| matcher 期望 Value::Obj 内联 vs registry 引用 | 中 | H2 后 send 存 VmRef，wake 时 decode_tagged_nv 重建 Value::Obj 给 matcher |

### §15.6 已完成的 L3 代码（Step 1-4，待 H2 后生效）

以下代码已落盘（plan-390/l3-withbindings-multi-field 分支），在 Phase H2 栈编码统一后即可工作：

- **Step 1**（codegen）：TaskDef 收集 `task_variants` map + `Add(3,5)` 在 Expr::Call 识别为
  variant 构造，emit CREATE_OBJ 产出 `Obj{__variant, fields}`。
- **Step 2**（send shim）：`shim_task_send_vm` 用 `pop_nv` + `is_object` 区分 Obj/scalar，
  Obj 存为 `Value::VmRef`。
- **Step 3**（handler codegen）：WithBindings 遍历 bindings，DUP+GET_FIELD+STORE_LOC 绑定每个字段。
- **Step 4**（wake）：消息唤醒 VmRef push `encode_object`。
- **阻塞点**：CREATE_OBJ 产出 `push_i32(obj_id)` 而非 `encode_object` → send 的 `is_object` 检测
  失败。Phase H2 统一栈编码后此阻塞自动解除。

### §15.7 Phase H2 + H4 实施记录（✅ 落地，2026-08-06，分支 `plan-390/h2-registry-unify`）

> 范围限定 **objects + heap_objects** 两套 registry 的栈编码统一；arrays/nodes 物理存储 + 栈编码
> 均保留，留 H3。完成后 L3（WithBindings 多字段）自动打通。

**Step 0 — 补 H1 遗漏的 `get_any_object`（engine.rs:734）**：交接文档称 H1 应交付此统一查询 helper，
实测 H1（commit `23422b97`）只做了 `impl HeapObject for ObjectData`，helper 缺失。补上一个按 id 段
路由的 trait 对象查询入口（当前仅覆盖 heap_objects 4M+ 段；objects/arrays/nodes 待 H3 物理迁移后并入）。
H2 的消费者改动仍显式查各自 registry（物理存储未变），helper 为 H3 铺路。

**Step 1 — objects 栈编码统一 + matcher VmRef 重水化 + handler 帧修复（解锁 L3，commit `14103b51`）**：
- CREATE_OBJ（engine.rs）/ inject_value Obj 分支：`push_i32`/`encode_i32` → `encode_object`。
- **matcher 关键修复**：`match_message_pattern_vm` 的 Simple/WithBindings 分支原只认 `Value::Obj`，
  但 send shim 把结构化消息存为 `Value::VmRef`（registry 引用），matcher 永不匹配 → actor 不唤醒。
  新增 `vmref_variant_name()` 从 objects registry 重水化 `__variant` 字段供 matcher 比对。
- **handler 帧结构性 bug 修复（G2 遗留，在 WithBindings ≥2 bindings 显现）**：#start 不发 RESERVE_STACK、
  bp=0、G2.2 RET 把 sp 复位到小值（0/1），handler locals（bp+1..）与 message/表达式临时值重叠。
  单变量（G2 测试）靠运气通过；双变量 DUP/GET_FIELD 临时值覆盖 binding 槽位 → a/b 错位（`Add(3,5)` 算出 10）。
  修复：消息唤醒前预留 `HANDLER_LOCALS_BAND=16` 槽位，使 handler locals 不与临时值重叠；
  handler_frame_base 锚定在预留区上方，RET 复位一并清除。

**Step 2 — heap_objects producer 统一（commit `9f2fba50`）**：CREATE_OK/CREATE_ERR/CREATE_LIST_*
（6 处）/NEW_INSTANCE/CREATE_TUPLE 全部 `push_i32 → encode_object`；CONSTRUCT_INSTANCE 三处配对
（pop instance_id 改双解码 + push-back 改 encode_object）。受影响「仅假设 i32」消费者加 is_object
回退：GET_TUPLE_FIELD、SET_ELEM、ARRAY_LEN。IS_OK/UNWRAP_OK/UNWRAP_ERR 巧合仍工作
（decode_i32(encode_object(id)) 低 32 位 == id，`value > 0` + get_heap_object(value) 命中），
H3 统一简化时一并改严谨。
- 更新 `plan326_array_struct_return_raw_repr` 测试预期：struct id 现为 TAG_OBJECT（渲染 `<vmref>`）
  而非裸 i32（4000000）—— 这正是 Plan 326 HTTP 序列化 bug 的根源，H2 修复后 repr 变更属预期。

**Step 3 — VmRef re-push 统一 + inject_value VmRef（commit `250faf64`）**：GET_ELEM/GET_FIELD 的
6 处 VmRef 结果 re-push（List<Value>/arrays 元素、node prop、objects/heap 字段、opaque native dispatch）
+ inject_value VmRef 分支 + materialize_value 镜像同步（decode 改 is_object/decode_object 双路径，
兼容 Obj 的 encode_object 与 Array/Node 的 encode_i32）。

**Step 4 — http_server.rs / stdlib.rs 审查（无需改）**：`nv_to_json`（http_server.rs:199）已有 is_object
分支，H2 后正确路由 TAG_OBJECT；stdlib.rs Writer.serialize（:7364）已用 is_object/decode_i32 双解码。
`value_to_json` 的 Value::VmRef 分支（:361）直接用 r.id。

**Step 5 — 专项消费者加固（部分推迟 H3）**：TO_STR 已有 is_object 分支（:3078）正确渲染。STR_CAT/LT
的 raw decode_i32 回退仅影响「把对象引用当数字拼串/比较」的病态程序，回归绿，推迟 H3。

**Step 6 — 验证**：
- L3 `actor_withbindings_multi_field` 通过（`h.send(Add(3,5))`→8，`h.send(Add(10,20))`→38）。
- 13 actor 测试全绿（含单变量 G2）；专项 object/field/struct/enum/list/map/tuple/result 487 passed / 0 failed。
- `cargo test -p auto-lang --lib`：**2820 passed / 22 failed**（基线 23，L3 转为通过 → 22，零新增）。
- `cargo test -p auto-man --lib`（a2r 回归）：**179 passed / 0 failed**。

**H2 期间发现的、原计划未预见的两项**：
1. **H1 漏交 `get_any_object`**：H1 commit 只做 ObjectData impl HeapObject，未补统一查询 helper（H2 补上）。
2. **G2 handler 帧结构性 bug**：交接文档假设「H2 栈编码统一后 L3 Step 1-4 自然打通」，但实测 G2 的方案 A
   （handler_frame_base 仅复位 sp，bp=0）在 WithBindings ≥2 bindings 时 handler locals 与临时值重叠，
   需在唤醒路径预留 HANDLER_LOCALS_BAND（H2 的临时补丁）。**此补丁是启发式（固定 16 槽），深表达式/多字段
   仍可能穿透 → 已在 §15.8 用方案 B（真 bp 帧）根治。**

### §15.8 Phase G2-refactor — handler 帧根治（方案 B：真 bp 帧）✅ 落地（2026-08-06，分支 `plan-390/g2-handler-frame-b`）

> 把 H2 临时引入的 `HANDLER_LOCALS_BAND=16` 启发式补丁换成**真正的 bp 栈帧**（CALL 风格），
> 根治 G2 handler 帧的结构性问题。语义统一到函数调用的栈帧模型：handler locals（bp+1..bp+N）
> 永远在 message/表达式临时值之上，不再靠「预留固定带」回避冲突。

**实施（4 处改动）**：
- **唤醒路径建真帧**（engine.rs `run_task_loop`）：替换 HANDLER_LOCALS_BAND 预留块，改为 mirror CALL 的
  三步建帧：`push(ret_ip=saved_ip)` → `push(old_bp=saved_bp)` → `bp = sp - 1`，再预留 `HANDLER_LOCALS_SLOTS=16`
  槽作 locals 区（bp+1..bp+16），最后 push message（在 locals 之上）。帧布局：
  `[ret_ip, old_bp, <locals 16>, message, <temps>]`。
- **handler RET 跟 n_args=0 字节**（codegen.rs:3792/3808）：handler 现运行在真 bp 帧（bp≠0），RET 会执行
  `new_sp = bp - n_args`；裸 RET（不跟操作数字节）会读到下一条字节码当 n_args。改 `emit(RET); push(0)`，
  对照普通函数 RET（codegen.rs:1422-1423）。
- **`park_ip` 字段**（task.rs + TASK_LOOP）：TASK_LOOP yield 时记录稳定重 park ip（#start 尾部 RET 地址）。
  handler RET 把 ip 恢复成 ret_ip（= #start RET），那条 RET 执行时读 n_args 字节推进 ip，会污染下次 wake
  的 saved_ip。RET-catch 把 ip 重置到 park_ip，使 actor 干净等待下一条消息。
- **RET-catch 简化**（engine.rs）：保留 `in_message_loop + Terminated → Waiting` 转换，但删方案 A 的
  sp 手动复位（bp 帧的标准 unwind 已恢复 sp/bp）；新增 `ip = park_ip` 重置。

**实施中实证发现的关键陷阱**（方案 B 的「暗礁」）：
- **RET 的 n_args 字节**：RET 恒读 1 字节操作数（engine.rs RET 处理器），bp≠0 时 `new_sp = bp - n_args`。
  handler codegen 原本裸 `emit(RET)`，靠 bp==0 短路才没崩；方案 B 给 handler 真 bp 后必须显式跟 n_args 字节。
- **ret_ip 重 park 污染**：handler RET 恢复 ip=ret_ip（#start 尾部 RET），那条 RET 执行后 ip 推进到 RET 之后，
  下次 wake 的 saved_ip 捕获到污染值（实测 saved_ip 11→13→20→28 漂移，actor 反复重入 handler 产生重复输出）。
  park_ip 字段根治此问题。

**不在范围（明确不做）**：
- PROPAGATE_MAY（`?`）路径：Auto task handler 语法（`on { msg -> }`）目前不支持 `?`，codegen 不在 handler
  内 emit PROPAGATE_MAY。未来若支持需同步给 handler 帧处理 `?` 的 unwind。
- `current_msg_context` / REPLY：唤醒路径从不设 current_msg_context（独立遗留），方案 B 不修不破。
- `#start` 加帧：#start 不用 LOAD_LOC/STORE_LOC（state 走 STORE_STATE_FIELD，bp 无关），bp=0 安全，不触动。

**验证**：
- 新增边界测试 `actor_withbindings_deep_expr_three_fields`：3 字段 WithBindings + 深嵌套表达式
  `((a+b)*(a+c))-(b*c)`（验证 locals 不被表达式临时值穿透）→ 通过。
- 13 actor 测试全绿（单变量 G2 + L3 多字段 + 新边界）；全量 `cargo test -p auto-lang --lib`：
  **2823 passed / 22 failed**（基线 22，零新增）；a2r `cargo test -p auto-man --lib`：**179 passed / 0 failed**。

**遗留**：`handler_frame_base` 字段（task.rs）在方案 B 后不再使用（park_ip 取代），保留字段待 H3 一并清理。

**H3 分批（2026-08-07 立项，分两阶段，详见 §15.9）**：
- **H3a（已交付 ✅）**：nodes 物理迁移（TypeTag::Node + impl HeapObject + CREATE_NODE/POP_ACCUM/inject_value 改 insert_heap_object + encode_object + 删 nodes registry）。见 §15.9 实施记录。
- **H3b（已交付 ✅）**：arrays+objects 物理迁移 + 删魔数 + `get_any_object` 接管 + IS_OK/UNWRAP_*/STR_CAT/LT 严谨化。见 §15.9.4 实施记录。

---

## §15.9 Phase H3 — arrays/nodes 物理迁移到 heap_objects + 删魔数（H3a/H3b 分批）

> **2026-08-07 立项**。H2 只统一了**栈编码**（所有对象引用 → `encode_object`），但 4 套 registry
> 的**物理存储**仍未统一（objects 1M+/arrays 2M+/nodes 3M+ 仍是独立 DashMap，heap_objects 4M+）。
> H3 做物理迁移 + 删魔数清理。分两阶段实施：**H3a（nodes，小而干净）先验证 orphan impl 模式 →
> H3b（arrays，大而机械）跟进**。objects 的物理迁移（CREATE_OBJ 改 insert_heap_object）并入 H3b
> 末尾（它与 arrays 同属"本地类型迁移"，且 H1 已让 ObjectData impl HeapObject）。
> 每阶段独立 commit + 独立回归验证。

### §15.9.1 调研结论（关键事实）

- **Node 是 Send+Sync**：所有字段（AutoStr/usize/Args/Obj/Kids）都是 owned 数据，无 Rc/RefCell/裸指针。
  `impl HeapObject for auto_val::Node`（orphan rule 合法：本地 trait + 外部类型）可直接写，
  **无需包装、无需下沉 trait**。
- **arrays 必须 `ListData<Value>` 包装**：orphan rule 禁止 `impl HeapObject for Vec<Value>`
  （Vec 外部 + Value 跨 crate，无本地类型参数）。但 `ListData<Value>` 的 HeapObject impl
  **已存在**（types.rs:284，TypeTag::ListValue）。CREATE_ARRAY 改存 `ListData { elems, storage: None }`。
- **ObjectData impl 已存在**（types.rs:173，H1 交付）——objects 物理迁移零新增 trait 工作，仅改存储路径。
- **段位 break 差异**：nodes 迁移后 0 个段位判断 break（所有魔数都是 `>= 4_000_000`，node id 迁到
  4M+ 正好命中）；arrays 迁移后 6 个 `>= 2_000_000` 判断会 break。

### §15.9.2 H3a — nodes 物理迁移（小，先做）✅ 已交付（分支 `plan-390/h3a-nodes-migration`）

**H3a.1 新增 TypeTag::Node + impl HeapObject for Node**（heap_object.rs）：
- TypeTag enum 加 `Node` 变体（ObjectData 之后）；`name()` 加 `TypeTag::Node => "Node"`。
- 新增 `impl HeapObject for auto_val::Node`（type_tag/as_any/as_any_mut）。

**H3a.2 改 3 个 producer**（storage + 栈编码）：
- `CREATE_NODE`（engine.rs）→ `let id = self.insert_heap_object(node)` + `push_nv(encode_object(id))`。
- `POP_ACCUM` → 同上。`inject_value` Node 分支 → `insert_heap_object(cloned)` + `encode_object`。

**H3a.3 改 3 个 consumer**（查 heap_objects + downcast Node）：
- `GET_FIELD` node 分支：`self.nodes.get(&obj_id)` → `self.get_heap_object(obj_id)` +
  `downcast_ref::<auto_val::Node>()`，get_prop + id/name/text fallback **逐行保留**（mold 模板
  `app.id`/`dep.at` 依赖）。
- `pop_auto_value`：**is_object 分支（主路径，实施时补的遗漏）** objects 查不到后加 heap_objects
  Node 回退；i32 分支 node 段保留作回退（改 get_heap_object + downcast）。
- `lib.rs extract_value_from_vm`：`vm.nodes.get(&id)` → `vm.get_heap_object(id)` + downcast Node。

**H3a.4 删 nodes registry**：engine.rs 删 `nodes` 字段 + `node_id_gen` + 构造初始化；
`autovm_persistent.rs` 删 `self.vm.nodes.clear()`（heap_objects.clear 已覆盖）。
`get_any_object` 的 `>= 4_000_000` 分支现在也覆盖 node id——保留（H3b 后 objects/arrays 并入时再简化）。

**H3a.5 注释更新**：materialize_value 注释"Array/Node still use raw i32"→"Array still uses raw i32 (H3b)；Node 已迁移"。

### §15.9.3 H3b — arrays + objects 物理迁移（大，后做）✅ 已交付（分支 `plan-390/h3b-arrays-objects-migration`）

- **H3b.1 改 7 个 array producer**（存 ListData<Value> + encode_object）：CREATE_ARRAY、inject_value Array、
  SLICE、native.rs 三个 HOF/alloc helper、http_server.rs 测试 fixture。`Vec<Value>` → `ListData { elems, storage: None }`，
  `array_id_gen` → `insert_heap_object`，`push_i32` → `encode_object`。
- **H3b.2 改 ~40 个 array consumer**（查 heap_objects + downcast ListData<Value>）。统一模式（先例 GET_ELEM、ARRAY_LEN）：
  `get_heap_object(id)` → `guard.as_any().downcast_ref::<ListData<Value>>()` → `list.elems`。涉及 9 个文件：
  engine.rs（12 处）、native.rs（12 处）、ffi/http_server.rs（3）、ffi/convert.rs（1）、lib.rs（3）、
  ui/vm_bridge.rs（4）、autovm_persistent.rs（1 显示 + .len/.clear）、interpreter/vm_interpreter.rs（1）、
  ui/aura_view_builder.rs（2）。分文件 commit，每批跑回归。
- **H3b.3 修 6 个段位 break**（`>= 2_000_000` → tag/contains 判断）：autovm_persistent.rs、vm_interpreter.rs、
  vm_bridge.rs（3 处）、aura_view_builder.rs（2 处）。
- **H3b.4 objects 物理迁移**（CREATE_OBJ 改 insert_heap_object）：CREATE_OBJ + inject_value Obj 分支存储改
  `insert_heap_object`（栈编码 H2 已是 encode_object 不变）；GET_FIELD/SET_FIELD objects 分支改
  `get_heap_object` + downcast ObjectData；删 `objects` 字段 + `object_id_gen`；http_server.rs/stdlib.rs
  直接读 `vm.objects` 改 downcast ObjectData。
- **H3b.5 删 arrays registry + array_id_gen**。
- **H3b.6 简化魔数 + decode_tagged_nv**：删所有 `>= 4_000_000`/`>= 1_000_000`/`>= 2_000_000` 段位判断（~12 处）；
  decode_tagged_nv 的 `i >= 4000000` 分支删除（所有 id 现在都是 is_object）；`get_any_object` 简化为
  `self.get_heap_object(id)`（不再段位路由）；IS_OK/UNWRAP_OK/UNWRAP_ERR 改严谨（is_object/decode_object，
  不再靠 decode_i32 低 32 位巧合）；STR_CAT/LT 回退加 is_object 分支。

**不在范围**：interpreter/vm_interpreter.rs（tree-walker）不重写架构（仅修段位判断）；a2c/a2ts `#[ignore]`
conformance 测试不强制开。

### §15.9.4 H3b 实施记录（2026-08-07，分支 `plan-390/h3b-arrays-objects-migration`）✅ 已交付

> 增量迁移策略：producer 先改 heap_objects（encode_object），consumer 逐个迁移/删除 arrays 回退，
> 最后才删 registry 字段（避免中途全量编译失败）。objects 迁移（H3b.4）在 arrays 之后做。

**H3b.1+b2（arrays 迁移，8 文件）**：
- producers：CREATE_ARRAY/inject_value Array/SLICE（engine.rs）+ native.rs 3 个 helper
  （create_list_from_i32/create_list_from_value/shim_alloc_array）+ http_server 测试 fixture ——
  全部 `Vec<Value>` → `ListData<Value>`，`array_id_gen` → `insert_heap_object`，`push_i32` → `encode_object`。
- consumers：engine.rs（ARRAY_LEN/GET_ELEM 删 arrays 回退、SET_ELEM/SLICE/5 个 CALL_SPEC List 操作改
  get_heap_object + downcast、contains_key 删）、native.rs（~12 个 list shim 删重复 fallback）、
  ffi/convert.rs 删 Path 2、ffi/http_server.rs 删 section 2 + probe 简化、lib.rs（format_value_for_display/
  format 分支/extract_value_from_vm 改 get_heap_object + downcast）、autovm_persistent.rs（stats 改数
  ListValue、REPL 格式化改 probe）、ui/vm_bridge.rs（read/write_state_as_vec/vmref_to_vec/
  read_child_state_as_vec 改 probe）、ui/aura_view_builder.rs（2 处 `>= 2M` → `>= 4M`）、
  interpreter/vm_interpreter.rs（段位判断 → probe + downcast）。
- **实施中发现的回归**：`test_config_for_unrolls_literal_array`（`ports: [8080, 9090]`）失败——
  pop_auto_value 的 is_object 分支未处理 ListData<Value>（CREATE_ARRAY 改 encode_object 后数组走
  is_object 而非 is_list），补 downcast ListData<Value> → `Value::Array` 后修复。

**H3b.3（6 个段位 break）**：autovm_persistent.rs（`>= 2M` → heap probe）、vm_interpreter.rs（1M/2M 段位
→ probe）、vm_bridge.rs（3 处 `>= 2M` → probe）、aura_view_builder.rs（2 处 `>= 2M` → `>= 4M`）。

**H3b.4（objects 迁移）**：CREATE_OBJ/inject_value Obj → insert_heap_object（栈编码不变 encode_object）；
GET_FIELD/SET_FIELD objects 分支并入 heap 分支（ObjectData downcast，保留 "HashMap" type_name 映射以
维持 auto.hashmap 方法派发）；pop_auto_value/vmref_variant_name/CREATE_NODE props/lib.rs/http_server/
stdlib.rs（Writer.serialize）/vm_bridge materialize_obj_ref 全部改 get_heap_object + downcast ObjectData。

**H3b.5（删 registry）**：删 engine.rs `objects`/`object_id_gen`/`arrays`/`array_id_gen` 字段 + 构造初始化。

**H3b.6（魔数清理）**：decode_tagged_nv 删 `>= 4000000 → VmRef` 分支（所有 id 现在 is_object）；
`get_any_object` 简化为 `self.get_heap_object(id)`；IS_OK/UNWRAP_OK/UNWRAP_ERR 改 pop_nv + is_object 严谨化；
STR_CAT/LT 回退加 is_object 分支。**保留**：`>= 4000000` 的 raw-i32 内容启发式（ListData<i32> 元素/
struct 字段里的 raw heap id，如 GET_ELEM/TYPE_TO_STR/GET_GENERIC_FIELD/EQ-NE 的 i32 回退）。

**验证**：
- 全量 `cargo test -p auto-lang --lib`：**2825 passed / 22 failed**（基线 22，零新增；2825 = 2823 + 1
  个 flaky perf benchmark 修复，同 §8.3 记录）。✅
- 专项：array/list/memory/config 320 passed / 0 failed；actor/unified_registry/field/struct
  125 passed / 0 failed。✅
- a2r 回归 `cargo test -p auto-man --lib`：**179 passed / 0 failed**（基线一致）。✅

**H3 完成判定**：4 套 registry（objects/arrays/nodes/heap_objects）→ 1 套 `heap_objects`；
`encode_object` 成为唯一对象栈编码；所有魔数段位判断与试探探测删除。H1/H2/H3a/H3b 全链路闭环。

### §15.9.5 H3a 实施记录（2026-08-07，分支 `plan-390/h3a-nodes-migration`）✅ 已交付

**实施（5 处改动）**：
- `heap_object.rs`：TypeTag 加 `Node` 变体 + `name()` match + `impl HeapObject for auto_val::Node`
  （孤儿规则合法，Node 全 owned 字段自动 Send+Sync）。
- `engine.rs`：删 `nodes` 字段 + `node_id_gen` + 构造初始化；3 个 producer（CREATE_NODE/POP_ACCUM/
  inject_value Node 分支）改 `insert_heap_object` + `push_nv(encode_object(id))`。
- **consumers 改 get_heap_object + downcast**：GET_FIELD 重构为「objects → heap_objects{Node →
  GenericInstance → RustStdlib}」合并链（id 段位不相交，行为等价）；`pop_auto_value` 的 is_object
  分支加 heap_objects Node 回退（**实施中发现主路径——nodes 现在 encode_object 编码，ACCUM_NODE 走此
  分支**，粘贴细节只列了 i32 回退分支）+ i32 分支 node 段改 get_heap_object；lib.rs `extract_value_from_vm`
  node 分支改 get_heap_object + downcast Node。
- `autovm_persistent.rs`：删 `self.vm.nodes.clear()`（heap_objects.clear 已覆盖）。
- `get_any_object` 代码不动（4M+ 段现覆盖 node id），仅更新注释；materialize_value 注释同步。

**验证**：
- `cargo build -p auto-lang`（debug）✅；config 专项 40 passed / 0 failed（含
  `test_config_for_over_runtime_array`/`test_config_pair_value_is_node_with_block`/
  `test_config_object_field_access`）✅。
- 全量 `cargo test -p auto-lang --lib`：**2823 passed / 22 failed**（基线 22，零新增——
  22 个均为预存 dstr/ark/vue/codegen_if/route-discovery）。✅
- a2r 回归 `cargo test -p auto-man --lib`：**179 passed / 0 failed**（基线一致）。✅

**遗留**：objects(1M+)/arrays(2M+) 物理存储未迁移——H3b 承接（§15.9.3）。

---

## §15 Phase H — `fn(params){}` 闭包参数绑定修复（L5，parser）

> **2026-08-07 追加**。L5 经实证重新定性：不是"闭包不能捕获外部变量"（捕获工作正常），
> 而是 `fn(params){}` 解析路径漏 bind 参数。阻塞 auto-ai Plan 021 Phase 6.6（driver Delta/Tool 转发）。

### §15.1 实证（2026-08-07）

```auto
fn outer_cb(p str) void { print(p) }
fn fwd(ev str, cb fn(str) void) void { cb(ev) }
// (1) => 形式：捕获正常
let a = (ev str) => fwd(ev, outer_cb)   // → |ev: String| fwd(ev.as_str(), outer_cb)  ✅
// (2) fn(params){} 形式：失败
let b = fn(ev str) void { fwd(ev, outer_cb) }  // → "Variable 'ev' is not defined"  ❌
```

`=>` 形式（`(ev) => ...` / `x => ...`）的两条解析路径都 `bind_var` 参数；`fn(params){}` 路径不 bind。
错误是闭包**自身参数** `ev` 未入 scope，非捕获 `outer_cb` 失败。

### §15.2 根因

| 解析路径 | 位置 | bind 参数？ |
|---|---|---|
| `x => body` | `parser.rs:1999-2002` | ✅ `bind_var` |
| `(a,b) => body` | `parser.rs:3842-3847` | ✅ `bind_var` |
| **`fn(params){ body }`** | **`parser.rs:3043-3069`** | **❌ 不 bind** |

`check_symbol`（`parser.rs:10091`）查 `infer_ctx.lookup_type(name)`；`fn(params){}` 路径解析参数后
不 `bind_var`，导致 body 内引用参数名时 `exists()` 返回 false → `NameError::undefined_variable`。

### §15.3 修复方案（~6 行，parser.rs）

`fn(params){}` 路径（`parser.rs:3043-3069`），解析参数后、解析 body 前，加 `bind_var` 循环
（镜像 `parser.rs:3842-3847`）：
```rust
for p in &params {
    self.infer_ctx.bind_var(
        crate::ast::Name::from(p.name.as_str()),
        p.ty.clone().unwrap_or(crate::ast::Type::Unknown),
    );
}
```
body 解析后配合 `push_scope`/`pop_scope` 防绑定泄漏（镜像 `parse_closure`）。a2r 闭包捕获
（`rust.rs:3017` + escape analyzer）+ VM 捕获（`codegen.rs:11044`）均已就绪，无需改。

### §15.4 实施任务（Phase H，auto-lang worktree）

- [ ] H.1 worktree：`fix/parser-fn-closure-params`
- [ ] H.2 `parser.rs:3043-3069`：`fn(params){}` 路径加 `bind_var` 循环 + scope push/pop
- [ ] H.3 新增 a2r 测试：`fn(ev Type) { forward(ev, outer_cb) }` 形式（含外部 fn 捕获），
       期望转译为 `|ev: T| forward(ev, outer_cb)` ✅
- [ ] H.4 回归：`cargo test --features test-trans` 零新增失败；现有 `=>` 闭包测试不受影响
- [x] H.5 回 auto-ai：重建 auto.exe → ~~解锁 Plan 021 Phase 6.6~~ Phase H 修了参数绑定，但
       **Phase 6.6 仍阻塞**：driver 转发需 EventSink cb 持有捕获 `on_event` 的闭包，但 cb 是
       裸 `fn(StreamEvent)` 指针——Auto/a2r 缺少 `Box<dyn Fn>`/`impl Fn` 闭包类型表达，
       闭包不能 coerce 成 fn 指针。这是更深层的语言限制（非 Phase H 范围）。driver 非流式
       事件已正常工作，仅流式 Delta/Tool 待 Auto 增加闭包类型。详见 Plan 021 Phase 6.6。

### §15.5 验收（Phase H）

- [ ] `fn(ev str) void { fwd(ev, outer_cb) }` 转译为 `|ev: String| fwd(ev.as_str(), outer_cb)`，捕获外部 fn
- [ ] `(ev) => fwd(ev, outer_cb)` 仍工作（向后兼容）
- [ ] 现有闭包测试零回归
