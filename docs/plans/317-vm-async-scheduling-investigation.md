# Plan 317: VM 真异步调度统一 — 调研报告 + 实施提案

> 原编号 327；2026-07-23 因编号冲突改为 317（原号保留给 327-015-notes-vm-render）
> **Status**: ✅ **Phase 1-4 全部完成**(2026-08-06 核查回填);🟡 **Phase 5-11 遗留债修复进行中**(§11)。调研完成(2026-06-18);Phase 1(actor handler 执行引擎,路径 B VM 内置调度)+Phase 3(lazy yield/SSE)于 2026-06-18 完成;Phase 2(`~{}`.await 取值)由 **Plan 348 Task 22** 顺手修复(CREATE_FUTURE 重写为 out-of-line 真字节码);Phase 4(HTTP 异步 server `serve_async`)已实施并接入 `lib.rs:1250` 活路径。**遗留债核实与修复计划见 §11(Phase 5-11)**。
> **实测状态(2026-08-06 核查)**: ✅ Phase 1-4 全闭环。Phase 1(`actor_tests`+`actor_state_tests` 13 测试绿)+Phase 3(lazy yield/SSE)+Phase 4(`serve_async` 并发,间接 generator SSE 路径已补测试)经测试验证;Phase 2(`~{42}.await`→42)经 `plan348_concurrency_tests::test_task22_*` 3 测试验证。**原 2026-08-04 自述"Phase 2 跳过"有误**——实际 Plan 348 Task 22 已修复,本文档此前未回填;Phase 4"待实施"亦已落地为 `serve_async`。**遗留债逐条核实见 §11 速览表**(P2 已解决 / P3 仍复现 / P4 精炼出 P4' 真 bug / P5 不做 / P6 范围外 / P7 待 CI 接入)。
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

### 下一步(Phase 2-4) — ✅ 全部完成(2026-08-06 回填)

Phase 1 解锁了 actor 执行。后续各 Phase 的真实归宿:
- **Phase 2**(✅ 已完成,由 Plan 348 Task 22 修复):`~{}` async block 的 await 取值。
  原判断"需 codegen 重构(body 隔离)、超出小修复"被 Plan 348 Task 22 推翻——它把
  `Expr::AsyncBlock` body 编译为 out-of-line 真字节码(像闭包),喂给 CREATE_FUTURE
  真实 body 地址,`~{42}.await`→42。见 `codegen.rs:8617`、`plan348_concurrency_tests::test_task22_*`。
- **Phase 3**(✅ 已完成,见 §10):真异步 generator(lazy yield)
- **Phase 4**(✅ 已完成):HTTP 异步 server 接入——`serve_async`(`http_server.rs:1125`)
  用 tokio `spawn_local`+`yield_now` 实现并发 + 交错 SSE,接入 `lib.rs:1250` 活路径。
  间接 generator SSE 路径(原 §10 已知遗留 2)已补 `e2e_sse_indirect_generator` 测试。

---

## §10 Phase 3 实施结果(2026-06-18,已完成)

**真异步 generator(lazy yield)** 实施完成。

### 改动

`shim_iterator_next` 的 Generator 分支(native.rs)从 **eager**(首次 next() 跑完整个
body 收集所有 yield 到 stack_snapshot,销毁 task)改为 **lazy**(每次 next() 只跑到
下一个 YIELD_VAL,task 跨 next() 保持存活):

- 首次 next():spawn task + 设帧 + 跑到第一个 GeneratorYield(peek yield 值,task
  挂起 Waiting,不销毁)
- 后续 next():恢复 task(从其保存的 ip/bp/sp)+ 跑到下一个 GeneratorYield
- Terminated:设 done + 销毁 task + push -1(nil sentinel,Plan 326 §1 修复保留)
- GeneratorYield 时 **peek**(不 pop)yield 值——codegen 在 YIELD_VAL 后 emit POP
  (ExprStmt 丢弃 yield 返回值),peek 让那个 POP 能正确消费

### 验收

- Plan 326 generator_tests(sum/no-dup/string-yields)3 测试仍全绿(lazy 不改变观察行为)
- SSE e2e 测试(`GET /api/counter`,inline yield handler)产生正确 SSE 帧
  (data: 1, data: 2, data: 3)——lazy 下每次 next() 只跑一个 yield,天然增量 flush
- 全量回归:2910 passed / 8 failed / 82 ignored(8 pre-existing,零新回归)

### 已知限制

1. **infinite generator(`for { yield x }`)**:卡死。根因:codegen 的 `for{}` 循环
   字节码每次迭代净消耗 1 个栈元素(栈不平衡),lazy 跨 next() 恢复时累积导致下溢。
   eager 不暴露(一次跑完 RET 清栈)。finite generator(有 RET 的序列)工作正常。
   修复需 codegen 的循环栈平衡,超出 Phase 3 范围。
2. **handler 间接调 generator**:handler `fn h() ~Iter<int> { counter() }`(h 自己不
   yield,调另一个 generator 函数)的 SSE 路径未通(h 返回的不是 iter_id)。inline
   yield 的 handler 工作。间接调用路径待后续。
3. **`~Stream<T>` 无独立 Iterator 变体**:继续走 `Iterator::Generator`(lazy 后语义
   足够)。独立变体是未来增强。
4. **yield/await 仍不同步**:generator next() 是 lazy 但同步 pull。真"yield 返回
   future 让出"是更大的异步调度改造,不在 Phase 3。

---

## §11 遗留债核实与修复 Phase(2026-08-06 起调查,Phase 5-11)

> 本节是对前述"已知遗留/限制"的逐条**实测核实 + 根因定位 + 修复计划**。
> 核实方法:用临时 `plan317_debt_probe` 测试(已删除)对每条遗留跑最小 reproducer,
> 区分"已自然解决/仍有缺陷/范围外"。核实结论可能推翻前述文档的自述。

### 核实结论速览

| 编号 | 遗留项 | 文档自述 | 实测结论(2026-08-06) |
|---|---|---|---|
| **P1** | scheduler.rs 死代码 | §9 #3"本计划不动" | 🟡 部分死代码:`execute_handler_fully`/`task_loop`/`SystemCommand` 无外部调用者(路径 A 被 run_task_loop 路径 B 取代);但 `GlobalMeta`/`TaskContext` 仍被 `task_system.rs` 用(`TaskSystem.start` 路径,文档 §2 断点 4b 已知挂死)。需谨慎拆分,非纯删除。 |
| **P2** | task state field 读取(print/let RHS) | §9 #1"报 undefined variable" | ✅ **已自然解决**。`print(count)`→"1"、`let c = count`→"1" 全部正常。`Expr::Ident` 分支(codegen.rs:5113)已正确检查 `current_task_state_fields`。仅需写回归测试固化。 |
| **P3** | infinite generator(`for { yield x }`) | §10 #1"卡死,栈不平衡累积" | ❌ **仍复现**(测试挂死超时)。根因已精确定位(见 Phase 7)。 |
| **P4** | producer/consumer 跨 actor | §9 #2"未验证" | 🟡 **部分解决,根因被精炼**。多 actor 共存 + 各自收发(P4a)正常;cross-actor 互发 handle 未测(语法限制)。但发现更深的 P4':**无 `fn start()` 且无 state field 的 task,payload binding 绑到 0**(`h.send(42)` → handler 里 `n` 是 0,不是 42)。这是跨 actor 通信的隐性阻塞,见 Phase 8。 |
| **P5** | `~Stream<T>` 独立 Iterator 变体 | §10 #3"装饰性" | 🟢 **确认不必要**。parser 已识别 `~Stream<T>`(parser.rs:10268),engine 折叠到 `Iterator::Generator`,lazy 后语义足够。降级为"不做",见 Phase 9。 |
| **P6** | yield↔await 真异步互通 | §10 #4"大改造" | 🟢 **范围外**。lazy generator SSE 已满足当前 HTTP 需求;真"yield 返回 future"需重写调度器,单独立项。本计划不做,见 Phase 10。 |
| **P7** | Phase 4 SSE/concurrent 测试 `#[ignore]` | 未列 | 🟡 实测存在:`e2e_sse_*`、`e2e_concurrent_sse`、`e2e_notes_crud` 等均 `#[ignore]`(起真实 TCP),常规 `cargo test` 不跑,回归风险无门禁。见 Phase 11。 |

---

### Phase 5(P2)— state field 读取回归测试 [✅ 已完成 2026-08-06]

**状态**:遗留已自然解决,本 Phase 仅固化。

**改动**:
- 在 `crates/auto-lang/src/tests/actor_state_tests.rs` 新增 2 测试:
  - `actor_state_field_print_direct`:`print(count)`(state field 作 intrinsic 参数)
  - `actor_state_field_let_rhs`:`let c = count`(state field 作 let RHS)
- 断言两者输出正确递增值(覆盖 §9 #1 的两个具体场景)。

**验收**:2 新测试绿;全量回归零新失败。

---

### Phase 6(P1)— scheduler.rs 死代码清理 [需谨慎分析]

**目标**:移除路径 A 的真死代码,保留仍被引用的类型。

**核实**:路径 A(tokio `task_loop` + `execute_handler_fully`)被路径 B(`run_task_loop` + `shim_task_spawn_vm`)完全取代,但:
- **可删**:`execute_handler_fully`(`_ => skip unknown opcodes` 占位,无外部调用者)、`execute_handler_with_vm`(同)、`SystemCommand`(无外部用)、`try_match_pattern`(仅 scheduler 内部)、`task_loop`(仅 task_system.rs 调,但 task_system.rs 自身是否还活需先确认)、`TaskContext` 大部分方法。
- **保留**:`GlobalMeta`(task_system.rs:78 / loader.rs:12 用)、`TaskContext` 结构(若 task_system.rs 仍活)。
- **前置确认**:`task_system.rs` 的 `start_scheduler`/`TaskSystem.start` 路径是否还有任何活调用(文档 §2 断点 4b 称其挂死;stdlib.rs:6291 `shim_task_system_start` 仍注册为 native 2305)。若 `TaskSystem.start` 是死路径,可连带清理 task_system.rs 的 tokio 调度部分,只留 `TaskRegistry`/`TaskHandle`(路径 B 复用)。

**风险**:`TaskSystem.start` 可能有用户代码依赖(虽挂死)。先标记 `#[deprecated]` + 保留,或彻底删。需先 grep examples/parity 确认无 .at 用例。

**验收**:删除后 `cargo build` + 全量测试零新失败;`TaskSystem.start` 若删,确认无 `.at` 例程调用。

---

### Phase 7(P3)— infinite generator 挂死 [✅ 已完成 2026-08-06]

**症状**:`fn counter() ~Iter<int> { var i = 0; for { yield i; i = i + 1 } }` 在 lazy generator 模式下,`for n in counter() { ... break }` 挂死(测试超时被杀)。

**根因(实测定位,与初判不同)**:初判"Stmt::Expr 在 Yield 后误 POP 下溢"被推翻——实测 `YIELD_VAL` 不 pop 值,driver 在 `sp-1` PEEK 并把任务设为 `Waiting("generator_suspended")`,resume 后那条 POP 正是消费 PEEK 值、清栈,**正确且必要**(finite generator 全绿证明)。真正的根因在 **`run_task_loop` 的退出条件**(engine.rs:1913):被消费者 `break` 抛弃的 generator 任务停留在 `Waiting("generator_suspended")`,但它不像 idle actor 那样被排除,被计入 `alive_count` → `alive_count != 0` → 循环在 `sleep(10ms)` 上无限转 → VM 永不退出。finite generator 不暴露此 bug 因其 `return` → Terminated → driver 返回 -1 → 消费者自然退出,任务随后被清。

**修复**:在 `run_task_loop` 的 idle-actor 检查(engine.rs:1767)之后,新增一个排除项——`Waiting("generator_suspended")` 的任务不计入 alive_count(它只被外部 `iter.next()` 经 try_lock 驱动,永不自唤醒)。与 idle-actor 同语义、同处理。

**改动文件**:`vm/engine.rs`(run_task_loop,~10 行)。

**验收**:
- 新测试 `generator_infinite_break`(sum 累计到 6 后 break,输出 "6")+ `generator_infinite_take_three`(取 3 个 1,输出 "3")全绿,不挂死。
- Plan 326 generator_tests 3 测试(finite)仍绿。
- 回归:`generator_` + `actor_` + `plan348_concurrency` + `iterator` 共 62 测试全绿,1 ignored,零新失败。

---

### Phase 8(P4')— 无 `#start` 导出的 task 的 payload binding 失效 [高优先]

**症状**:`task Solo { on { n int -> { print(n) } } }; let h = Task.spawn("Solo", 0); h.send(42)` → 输出 "0"(应为 42)。加 `fn start()!` 或 state field 后正常。

**根因(实测定位)**:
1. codegen.rs:3667 — `#start` 导出仅在 `start_hook.is_some() || has_state` 时 emit。无 `fn start()` 且无 state field 的 task 不生成 `#start` 导出。
2. stdlib.rs:`shim_task_spawn_vm` 的 `start_offset = vm.flash.exports_by_name.get(&start_key).copied().unwrap_or(0)` —— 缺 `#start` 时 fallback 到 **0**(程序入口,错误)。
3. 任务从 offset 0 起跑,`in_message_loop` 未正确建立,handler 唤醒路径(engine.rs:1698)的帧/payload 推送与 binding STORE_LOC 错位 → `n` 读到 0。

**修复方案**:让无 `fn start()` 的 task 也生成一个最小 `#start` 导出(只含 state 初始化 + TASK_LOOP,无用户 body),使 `shim_task_spawn_vm` 拿到正确 `start_offset`。具体:放宽 codegen.rs:3667 的条件为"有 on handlers 就 emit `#start`(含 TASK_LOOP)",或 shim fallback 到 task 的 TASK_LOOP offset 而非 0。

**改动文件**:`vm/codegen.rs`(TaskDef 的 #start emit 条件)、可能 `vm/ffi/stdlib.rs`(shim_task_spawn_vm fallback)。

**验收**:
- 新测试 `actor_no_start_no_state_binds_payload`:`task Solo { on { n int -> print(n) } }` + send(42) → "42"。
- `actor_bound_var_handler_multi_send` 等既有测试仍绿。
- 全量回归零新失败。
- P4 跨 actor(P4a)已有验证,本 Phase 修复后 cross-actor 互发 handle 可作延伸验证(若语法支持把 handle 当 Value 传递)。

---

### Phase 9(P5)— `~Stream<T>` 独立变体 [降级:不做]

**结论**:核实后确认不必要。`~Stream<T>` 在 parser 已识别,engine 统一折叠到 `Iterator::Generator`,lazy 后 SSE 语义完整。独立变体仅是类型层"装饰",无功能收益,徒增维护面。

**决议**:不做。文档记录此结论即可。

---

### Phase 10(P6)— yield↔await 真异步互通 [范围外:独立计划]

**结论**:真"yield 返回 future 让出调度"需重写 engine 调度模型(把 generator 从同步 pull 改为 future-based push),是架构级改动。当前 lazy generator + `serve_async` 的交错 SSE 已满足 HTTP 异步服务需求。

**决议**:本计划不做。若未来需要"generator 内 await I/O"或"yield 让出到其他 task",单独立项(候选:Plan 069 M:N 调度复活,或新建)。

---

### Phase 11(P7)— Phase 4 SSE/concurrent 测试去 `#[ignore]` 接入 CI [中优先]

**现状**:`http_server.rs` 的 `plan326_tests` 模块里 `e2e_sse_generator_handler`、`e2e_sse_indirect_generator`、`e2e_concurrent_sse`、`e2e_notes_crud`、`e2e_notes_list_generic` 均 `#[ignore]`(起真实 TCP 监听,有端口冲突/挂死风险)。CI(`auto-lsp-ci.yml` 只跑 `cargo test -p auto-lsp`,且未 `--ignored`)从不覆盖,Phase 4 的并发 SSE 回归无门禁。

**修复方案**:
- 给这组测试分配**动态端口**(每测试取一个空闲端口,或用 `portpicker` crate),消除端口冲突。
- 加超时防护(测试自身 `tokio::time::timeout` 包裹,防 server 挂死)。
- 新建 feature `http-e2e`(默认关),在 CI 加一个 job:`cargo test -p auto-lang --features http-e2e -- --ignored` 专跑这组(与常规测试隔离,避免日常 `cargo test` 拖慢)。
- 或:把 server 起在专用线程 + 短超时,直接去 `#[ignore]`(若能证明不拖慢/不冲突)。

**改动文件**:`vm/ffi/http_server.rs`(测试改造)、`Cargo.toml`(feature)、`.github/workflows/auto-lsp-ci.yml`(新 job,可选)。

**验收**:`cargo test --features http-e2e -- --ignored plan326_tests::` 全绿;常规 `cargo test` 不受影响。

---

### §11 实施顺序

按价值/风险/依赖排:
1. **Phase 5**(P2 回归测试)——零风险,先固化已解决的。
2. **Phase 7**(P3 infinite generator)——高价值(解锁常见 generator 模式),根因已定位,改动小。
3. **Phase 8**(P4' payload binding)——高价值(修真 bug),根因已定位,改动小。
4. **Phase 11**(P7 CI 接入)——中价值(防回归),改动中等。
5. **Phase 6**(P1 死代码清理)——低风险但需谨慎分析,最后做。
6. Phase 9/10 —— 不做(记录决议)。

每个 Phase 独立可验收,单独提交。Phase 7/8 是核心(真 bug 修复),优先做。
