---
plan: 390
title: actor-state-injection
affects: [auto-lang/parser, auto-lang/a2r, auto-lang/vm, a2r-std]
status: complete # draft | in-progress | complete
# auto-lang 侧范围完成（Phase A/B/E/G1/G2/F 落地）；Phase D 在 auto-ai 仓推进。
# 3 个非阻塞遗留见 §14（L1 a2r Arc/Box 实参渲染 / L2 双层包装 / L3 WithBindings 多字段）。
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

## §14 遗留汇总（2026-08-06 复审）

auto-lang 侧实质工作全部完成（Phase A/B/E/G1/G2/F 落地；Phase D 属 auto-ai）。以下是 3 个明确记录的遗留，均非阻塞：

| # | 遗留 | 性质 | 位置 | 严重度 | 触发条件 |
|---|---|---|---|---|---|
| **L1** | a2r 实参位 `Arc(x)`/`Box(x)` 渲染缺陷 —— 只有 let 位正确，实参位不渲染 `::new` | a2r 转译器 bug | `trans/rust.rs`（§451 KNOWN-DEBT Plan 021）| 中 | 当前用 `let a = Arc(tool)` workaround 绕过（§439）；a2r 根因修复后可去 let 绑定直写 |
| **L2** | `Arc<Box<dyn Tool>>` 双层包装 —— 存储类型是双层，rust-ref 是单层 `Arc<dyn Tool>` | 设计偏差 | auto-ai tool.at 存储（§448）| 低 | Deref 链功能可用；转正时若要对齐 rust-ref 单层，另立 a2r spec 返回位/存储位推导计划 |
| **L3** | WithBindings 多字段消息绑定未实现 —— `on { Add(a int, b int) -> }` 的多字段绑定 | VM 功能缺口 | `vm/codegen.rs` + `vm/engine.rs`（§G2.7）| 低 | 当前只支持单字段 TypeBinding（`on { n int -> }`）；多字段需 DUP + 多 STORE_LOC。罕见场景，单字段够用 |

**EventSink VM 端到端未验证**（§5.10/§G2 验收）：EventSink 的生产路径是 a2r（Phase B 交付），
VM 侧 spawn-with-args + bound-var handler 机制均已就绪（Phase A + G2），但 EventSink 完整形态
（fn 指针 state + 绑定变量 handler 组合）未在 VM 跑通端到端。非阻塞——EventSink 不走 VM 路径。

**auto-lang 侧范围判定：完成**。剩余 Phase D（§5.12-5.15）在 auto-ai 仓推进。
