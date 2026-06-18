# Plan 327: VM 真异步调度统一 — 调研报告 + 实施提案

> **Status**: 调研完成(2026-06-18);**Phase 1 已完成并合并**(actor handler 执行引擎,路径 B VM 内置调度);Phase 2-4 待实施
> **背景**: 用户期望 `yield`/`~Iter`、`~{}`/`~T`/await、Task/Msg actor 三套异步机制能在 AutoVM 里统一工作,以支撑 HTTP 异步服务(SSE、并发)。本报告用最小 reproducer 敲定了每个机制的真实状态。
> **关联**: Plan 312(HTTP server MVP,同步 std::net)、Plan 313(SSE Phase 3 未做)、Plan 321(yield/Iter,§5 明确不做异步)、Plan 121(Task/Msg 数据结构)、Plan 224(`~{}`/await codegen)

---

## §1 调研方法

在 `master` 上用 `run_with_capture` 跑 10 个最小 probe(每个针对一个断点),捕获 `result`(返回值 repr)和 `stdout`,判断机制是否真工作。语法疑点对照 `parser.rs` 源码逐一修正。probe 文件已清理(本次调研不留测试债)。

---

## §2 真实状态(逐断点)

### ✅ 工作的机制

| 断点 | 机制 | 证据 |
|---|---|---|
| **2** | `~T` 异步函数 + `.await` | `fn compute() ~int { return 89 }` + `compute().await` → stdout "got 89";带 body 逻辑 `let y=x*2; return y` → "got 42"。**完全工作。** |
| **3** | `TaskSystem.run(~{...})` | stdout 正确输出 "inside TaskSystem.run" + "after run"。**工作。** |
| **1a** | `~{}` async block 的 body 执行 | `let f = ~{ print("inside") }` → stdout 含 "inside async block"。**body 确实执行**(我先前关于 body_offset=0 占位的担心被推翻)。 |
| **7** | `yield`/`~Iter<T>` for-loop | `sum([1,2,3])` = 6(Plan 326 §1 修复后)。**工作**(但同步,见下)。 |

### ⚠️ 部分工作(有缺陷)

| 断点 | 机制 | 现象 | 缺陷 |
|---|---|---|---|
| **1b** | `~{}` + `.await` 取返回值 | `~{ 42 }` 经 `.await` → "got **0**"(应为 42) | body 执行了,但 `~{}` 创建的 future 没把 body 返回值存进 `result`,await 拿到 0。对比:断点 2 的 `~T` **函数** await 取值正确 —— 说明缺陷只在 `~{}` 字面量路径,不在 `~T` 函数路径。codegen.rs:6996 的 CREATE_FUTURE 占位与此一致。 |

### ❌ 不工作的机制

| 断点 | 机制 | 现象 | 根因 |
|---|---|---|---|
| **4** | 独立 `task` 定义 + spawn + handler 执行 | `Task.spawn("Counter",16)` + `h.send(Msg.Ping)` → stdout 只有 "main before/after",**start hook 和 message handler 都不执行**(无报错) | `scheduler.rs::execute_handler_fully`(206-209 行)`_ => skip unknown opcodes` —— 除 RET/HALT/NOP 外所有 opcode 被跳过。actor 能收消息、能匹配 pattern,但 handler body 里的字节码不执行。 |
| **4b** | `TaskSystem.start()` | **阻塞挂死**(测试超时 10min) | `shim_task_system_start` 注释自承 "blocks main thread, waits for Ctrl+C";测试环境无人发信号。即便不挂死,因断点 4 的 handler 执行缺陷,actor loop 也跑不出有用结果。 |
| **5** | producer/consumer 并发 | send 成功(stdout "sent messages"),但 Consumer 的 handler 不执行 | 同断点 4。mailbox 投递正常,handler 不跑。 |
| **6** | 原生 channel | `channel::unbounded()` parse error | **无原生 channel 类型**。stdlib 只有 crossbeam stub(转译用)和 oneshot reply。actor 通信只能靠 `TaskHandle.send`(返回 1=投递成功,但接收方不消费)。 |

### 🔗 互通性

| 断点 | 结论 |
|---|---|
| **7** | `yield`/Iter 与 future/await **完全无互通**。yield 走 engine.rs `run_one_instruction` + native.rs eager 收集;await 走 `execute_single_frame` + `FrameResult::AwaitFuture`。两套独立机制,不共享调度。`~Stream<T>` 无对应 `Iterator::Stream` 变体(engine.rs:144)。 |

---

## §3 三套异步机制的拓扑

```
┌─────────────────────────────────────────────────────────────┐
│ 机制 A:yield / ~Iter<T>(Plan 321)                           │
│   engine.rs: run_one_instruction + YIELD_VAL                │
│   native.rs: shim_iterator_next(eager 收集,同步)            │
│   状态: ✅ 工作(同步 pull 模型)                              │
└─────────────────────────────────────────────────────────────┘
          (与 B/C 不共享调度)

┌─────────────────────────────────────────────────────────────┐
│ 机制 B:~{} / ~T / .await(Plan 124/224)                      │
│   codegen.rs: CREATE_FUTURE(~{} 占位)/ AWAIT_FUTURE          │
│   engine.rs: execute_single_frame + FrameResult::AwaitFuture│
│   stdlib.rs: shim_task_system_run                            │
│   状态: ✅ ~T 函数 await 工作;⚠️ ~{} 字面量 await 取值缺陷   │
└─────────────────────────────────────────────────────────────┘
          (与 A/C 不共享调度)

┌─────────────────────────────────────────────────────────────┐
│ 机制 C:task / Task/Msg actor(Plan 121)                      │
│   parser.rs: parse_task(fn start()!{} / on{ Pat -> {} })    │
│   codegen.rs: Stmt::TaskDef(编译 hook + handler)            │
│   task_system.rs: TaskRegistry + start_scheduler + task_loop│
│   scheduler.rs: execute_handler_fully ← ❌ 占位骨架          │
│   状态: ❌ 数据结构/投递全通,handler 执行引擎不跑字节码       │
└─────────────────────────────────────────────────────────────┘
```

**核心结论**:三套机制各自造了一半,彼此不通。机制 B(await)最接近可用,机制 A(yield)同步可用,机制 C(actor)卡在执行引擎占位。

---

## §4 与 HTTP 异步服务的关系

用户最终目标:用现有异步系统组合 HTTP 异步服务(并发、SSE)。基于 §2/§3,可行路径:

| HTTP 能力 | 依赖的机制 | 当前可行性 |
|---|---|---|
| 并发请求处理 | actor(每请求一个 task)或 thread | ❌ actor handler 不执行;现状靠 unsafe 裸指针 + std::thread(lib.rs:762)串行 |
| SSE 推流(`~Stream<T>` handler) | 机制 B(await)+ 机制 A(yield)互通 | ❌ yield 不同步到 await;`~Stream<T>` 无 Iterator 变体 |
| 异步 handler(`fn h() ~T`) | 机制 B | ⚠️ `~T` await 工作,但 HTTP server 不调 await(http_server.rs 直接 call_fn_by_name) |

**结论**:HTTP 异步服务无法用现状直接组合,需要先把机制 C 接通,再考虑 A↔B 互通。

---

## §5 实施提案(待评审)

按"先通最低层、再向上接"的顺序,分 4 个 Phase。每个 Phase 独立可验收。

### Phase 1 — 接通 actor handler 执行引擎(P0,解锁 task 定义)

**目标**:让 `task` 的 `on` handler 和 `fn start()!{}` 真正执行字节码。

**根因**:`scheduler.rs::execute_handler_fully`(206 行)`_ => skip`。需改为调用真正的单指令执行(复用 engine.rs 的 opcode 分派,或调 `execute_single_op`)。

**验收**:probe 04 的 `Counter` task,`Task.spawn` + `send(Msg.Ping)` 后 stdout 出现 "Counter started" + "got Ping"。

**风险**:`execute_handler_fully` 在 tokio async 上下文(`task_loop` 是 `async fn`),而 engine.rs 的执行是同步阻塞循环。需确认能否在 async 里同步跑(或用 `spawn_blocking`)。这是 Plan 312 §2 论证的 `!Send` 阻抗的具体体现。

### Phase 2 — 修复 `~{}` async block 的 await 取值(P1)

**目标**:`~{ 42 }.await` 返回 42(当前返回 0)。

**根因**:codegen.rs:6996 CREATE_FUTURE 的 body_offset 占位为 0,`~{}` body 虽执行但返回值未存入 future.result。

**验收**:probe 01b 的 `~{ 42 }.await` → "got 42"。

**对照**:`~T` 函数路径(断点 2)await 取值正确,可参考其 future.result 写回逻辑。

### Phase 3 — `~Stream<T>` 异步流 + yield 互通(P1,解锁 SSE)

**目标**:`fn h() ~Stream<int> { yield 1; yield 2 }` 的 yield 值能被 await 消费(而非只被同步 for 消费)。

**改动**:
- engine.rs:144 `Iterator` 枚举新增 `Stream` 变体(异步,基于 future)
- native.rs:`~Stream<T>` 的 next() 返回 future(Pending 让出),而非同步收集
- http_server.rs:handler 返回 `~Stream<T>` → SSE 模式改用 await 拉取(替换当前的同步轮询)

**验收**:`GET /api/counter`(`~Stream<int>` handler)→ SSE `data: 1\r\ndata: 2\r\n`,且 handler 内可 `await` I/O。

### Phase 4 — HTTP 异步 server 接入(P2,组合层)

**目标**:把 Phase 1-3 的能力接到 HTTP server,实现并发 + 异步 SSE。

**选项**(需评审):
- A. 每请求 `Task.spawn` 一个 actor task(依赖 Phase 1)
- B. handler 返回 `~T`/`~Stream<T>`,server 调 await 消费(依赖 Phase 2/3)
- C. 引入 axum/tokio,用 `spawn_blocking` 桥接 VM(Plan 312 §2 方案 B)

---

## §6 不做(范围控制)

- **真 M:N 绿线程调度**:Plan 069 的 `tests_concurrency.rs` 被注释掉(`vm.rs:65`),M:N 调度是更大的架构改动,不在本计划。actor 用 OS 线程/spawn_blocking 即可起步。
- **原生 channel 类型**:actor 通信用 `TaskHandle.send` 已够;channel 留给后续。
- **HTTPS/TLS**:独立计划。

---

## §7 下一步建议

1. **评审本报告**,确认 Phase 划分和优先级(尤其 Phase 1 的 async/同步阻抗方案)。
2. 若认可,我**先用 EnterPlanMode 为 Phase 1 写详细实施计划**(它是 P0 且有架构难点,值得单独 plan)。
3. Phase 2 是小修复,可与 Phase 1 同 PR。
4. Phase 3/4 待 Phase 1 落地后再细化。

---

## §8 调研遗留

- probe 文件已清理(commit 98378f76),无测试债。
- 本报告基于 master `98378f76`(2026-06-18)的代码状态。
- 调研中发现的语法事实(供后续 plan 引用):
  - `task` lifecycle hook:**必须** `fn start() ! { }`(带 `!` 后缀),parser.rs:4621 强制
  - `on` 块 handler 用 **`->`**(`Arrow`),不是 `=>`(`DoubleArrow`),parser.rs:4699
  - `on` 块 pattern **不支持 `_`** 通配符;用具体 literal 或 Name 或 `else ->`
  - `on` 块 pattern 接受:string/int/uint/bool literal、Name、`Name(bindings)`、`name type`(type binding)

---

## §9 Phase 1 实施结果(2026-06-18,已完成)

**路径 B(VM 内置调度)** 选定并实施完成。actor 的 `fn start()!{}` hook 和
`on { Pat -> {} }` message handler 现在在 AutoVM 下真正执行字节码。

### 改动点(对应 §5 Phase 1 的 4 个断点)

| 断点 | 改动 | 文件 |
|---|---|---|
| 1. registry 空 | 新增 `AutoVM::load_task_handler_registry`(engine.rs:475);lib.rs 5 处编译入口 `std::mem::take(&mut codegen.task_handler_registry)` + load(仿 generic_registry) | engine.rs, lib.rs |
| 2. 无消息队列 | AutoVM 新增 `task_mailboxes: DashMap<TaskId, std::sync::Mutex<Vec<Value>>>`(engine.rs:269);**未放 AutoTask**(tokio Mutex 的 blocking_lock 在 sync native 里 panic) | engine.rs |
| 3. spawn/send 不碰 VM | 新增 vm-aware `shim_task_spawn_vm`/`shim_task_send_vm`(stdlib.rs:3460/3520);CALL_NAT 对 id 2300/2301 特判调用(engine.rs:4943) | stdlib.rs, engine.rs |
| 4. 不唤醒 message-loop task | run_task_loop 加 message-loop 唤醒检查(engine.rs:1243):drain 一条 mailbox 消息 → find_handler_offset → 设 ip + Ready | engine.rs |

### 附带修复(实施中发现)

- **TASK_LOOP 位置错误**:原 codegen 把 TASK_LOOP emit 在 TaskDef 末尾(主程序流),不在 start hook 内 → actor 跑完 start hook 直接 RET 终止。改为在 start hook body 末尾、RET 之前 emit TASK_LOOP(codegen.rs:3094,仅当有 on handlers)。
- **TASK_LOOP 不 return**:TASK_LOOP 设 Waiting 后继续执行下一条(RET),bp==0 触发 Terminated。改为 `return Ok(StepResult::Yield)`(engine.rs:5536)。
- **handler RET 后终止**:message-loop task 的 handler RET(bp==0)→ Terminated。run_task_loop 检测到 message-loop task 的 Terminated 时改回 Waiting(engine.rs:1286),让它等下一条消息。
- **idle actor 死循环**:Waiting 且 mailbox 空的 actor 让 run_task_loop 无限 sleep。加 `is_idle_actor` 检查(engine.rs:1267),idle actor 不计 alive_count,VM 可退出。
- **抽取 find_handler_offset**(engine.rs:482):HANDLE_MSG 和 run_task_loop 唤醒共用;含 else fallback(查 `"{type}#else"` export)。

### 验收

5 个回归测试(`actor_tests.rs`)全绿:
- `actor_start_hook_runs`:fn start hook 执行
- `actor_message_handler_runs`:on handler 匹配执行
- `actor_multiple_messages_dispatched`:多消息按序分派(1,2,1 → got one/got two/got one)
- `actor_else_handler_runs`:else fallback
- `actor_vm_exits_after_messages`:VM 正常退出(不死循环)

全量回归:**2907 passed / 8 failed / 81 ignored**。8 failed 全是 pre-existing(ui_gen + test_field_access_bool),**零新回归**。

### 已知遗留(Phase 1 未覆盖)

1. **task state 字段**(部分实现,2026-06-18):`task T { count = 0 }` 的 state
   field 通过新增 `LOAD_STATE_FIELD`/`STORE_STATE_FIELD` opcode(0xC3/0xC4) +
   `AutoTask.state_vars: Vec<NanoValue>` 实现。codegen 在 TaskDef 编译时为
   state field 分配 idx 并填充 `current_task_state_fields`,在 start hook 开头
   emit 初始化,在 handler 里把 state field 名的读取/赋值/复合赋值编译成对应
   opcode。**已验证**:声明、初始化、`count = count + 1` 递增、条件分支
   `if count == N`、跨 handler 持久保持(actor_state_tests.rs 2 测试全绿)。
   **已知限制**:state field 名作为 `print(count)` 参数或 `let c = count` RHS
   时,某些 intrinsic/let 的 codegen 路径绕过了 state field 检查,报 undefined
   variable。这是 codegen 变量解析分散导致的边缘问题,核心机制可用。
2. **producer/consumer 跨 actor**:单 actor 的 start + 消息处理工作了,但两个 actor 互相 send(h.send 给另一个 actor 的 handle)未验证 —— 需要 actor 能持有并传递 TaskHandle。这是 Phase 1 的自然延伸,待后续。
3. **scheduler.rs 路径未清理**:旧的 task_system mailbox + execute_handler_fully 占位路径仍在(dead code),本计划不动(避免扩大范围)。
4. **并发性**:run_task_loop 单线程协作式,actor 交错执行非真并发。对 MVP 足够。

### 下一步(Phase 2-4)

Phase 1 解锁了 actor 执行。后续:
- **Phase 2**(小修复):`~{}` async block 的 await 取值(断点 1b)
- **Phase 3**:`~Stream<T>` 异步流 + yield/await 互通
- **Phase 4**:HTTP 异步 server 接入(actor 处理请求 / handler 返回 ~Stream 做 SSE)

