# Design 34: VM store 所有权与 handler 阻塞纪律（决策工件）

- 状态：📝 决策工件（PLAN-702 T-07 产出，**待用户裁定**；本文不落任何实现）
- 日期：2026-09-25
- 来源：PLAN-702（ui-handler-async）落地后浮现的两条架构分叉
- 关联：[Design 33](33-stdlib-runtime-and-http.md)（§8.1 VM 行）、PLAN-702、
  PLAN-083（消息桥先例）、PLAN-699（Axum 传输）

## 1. 问题陈述

PLAN-702 把 UI handler 驱动切换为段执行（park/resume）后，VM 执行模型出现
**两种并发形态并存**：

1. **段执行形态**（PLAN-702 落地）：handler 遇异步等待即 park，恢复泵在
   iced update 内续跑。同一时刻至多一个"连续段"在执行（单 VM 串行），
   但**逻辑上**多个 handler 段可交错存活（A park 期间 B 首段可跑）。
2. **actor 任务形态**（Plan 224/317 既有）：`task on {}` 消息循环 +
   mailbox + `TaskStatus::Waiting`，任务独占自己的解释栈。

两种形态共享同一 `AutoVM` 堆（heap_objects / strings 池 / state 对象），
且 store（module 级 var / model 字段）都是**直接读写**（SET_FIELD 落共享
堆），没有任何所有权边界。本工件裁定两件事：

- **Q1（store 所有权）**：parked 段与 actor 任务并存后，store 归属走
  Erlang 式还是 Go 式？
- **Q2（handler 阻塞纪律）**：handler 内发起异步等待已从"忙等冻结"变为
  "park 交还"——但 CPU-bound 长循环仍会冻结 UI（段驱动无抢占）。禁阻塞
  调用/长循环的纪律走类型系统还是 lint？

## 2. 现状事实（2026-09-25 实装勘定）

| # | 事实 | 出处 |
|---|---|---|
| F1 | 全部 VM 执行经 iced update 串行化（架构裁断 1）；恢复泵也在 update 内 | PLAN-702 T-03（`__parked_resume_tick` 臂） |
| F2 | parked 段的 task（ip/bp/栈）只住 UI 线程的 bridge 注册表（RefCell），无跨线程共享 | `ui/vm_bridge.rs` `parked_tasks` |
| F3 | worker（HTTP 池线程）只写进程级 `ASYNC_RESULTS` 表；**VM 栈永不跨线程**——shim 在 UI 线程重入消费 | `vm/ffi/stdlib.rs` shim 重入臂 |
| F4 | store/model = 共享堆上的 GenericInstanceData；handler/computed/store facade 全部直写（PLAN-442 facade、PLAN-685 回调根修都在此面） | `read_state/write_state` 族 |
| F5 | PLAN-083 消息桥（`Http.get_msg/post_msg` + http_msg_subscription）已是"发起段即返回、回填走事件泵"的手工两段式 | `stdlib.rs:7434+` |
| F6 | actor 任务的 mailbox 通道（AutoChannel）与段执行互不感知；`task_loop` 从不触碰 parked 注册表 | `vm/scheduler.rs` |
| F7 | 段驱动无抢占：CPU-bound handler 一次 update 跑满 10M 步预算才停（WARN 后返回 Ok，帧未完）——UI 冻结面仍在 | `drive_handler_segment` budget 臂 |

## 3. Q1：store 所有权——Erlang 式 vs Go 式

### 3a. Erlang 式（store 归 UI task，worker 消息交账）

store 只被 UI 线程的 VM 上下文读写；一切后台产物（HTTP 响应、文件 IO、
计算结果）以**消息**进入（PLAN-083 桥的推广形态），由 handler/store
handler 显式落账。

- 优点：单写者天然成立；与 iced 单线程 update 模型同构（F1）；与 actor
  形态语义统一（都是 mailbox 交账）；RC/账本纪律（PLAN-062）只在单线程
  发生，无锁化空间。
- 缺点：迁移面大——现有 `#[api]` 直接赋值形态（`resp = Http.post_json(..)`
  的直写链，PLAN-080 F-2③ 的编译期改写）要改写为两段式或编译器自动 CPS；
  生态（musk、画廊 demo、auto-edit）全是直写风格。

### 3b. Go 式（共享 + 同步，现状延伸）

store 保持共享堆直写；并发安全由"全部写点都在 UI 线程"这条**执行纪律**
保证（F1 已天然成立），跨线程只交数据不交栈（F3 已成立）。

- 优点：现状即此形态且已被 PLAN-702 验证可持续（段交错不破坏一致性，
  中途态可见性同级于同步顺序写）；零迁移；PLAN-442 facade/685 根修资产
  全部保留。
- 缺点：纪律靠约定（未来任何"worker 直接改堆"的捷径都会破坏它）；中途态
  对 view 可见（同一 handler 的多次 SET_FIELD 可被渲染拆开看到）——语义上
  需要接受"handler 不是原子事务"（现状已如此，段化只是让它更显性）。

### 3c. 裁定建议（供用户裁定）

**建议 Go 式（共享 + 单写者纪律），把 Erlang 式留给 actor 任务域**。理由：

1. F1+F3 已经构成 Go 式的全部安全性来源——**它不是妥协而是既成事实的正确
   形态**；Erlang 化需要重写整个 `#[api]` 直写生态，收益只有理论纯度。
2. 两种形态分区共存语义清晰：**widget handler/store = Go 式共享直写**；
   **`task on {}` actor = Erlang 式 mailbox**。桥接（actor → UI store）走
   既有 bus.subscribe/POST publisher（PLAN-698 已验证的带外通道）。
3. Erlang 化的唯一硬需求场景——"后台任务直接写 store"——应被 Q2 的纪律
   禁止而非支持。

**若采纳 Go 式，需在 spec 沉淀一条不变量**：*AutoTask 的栈（ram/ip/bp）
只在其属主线程被触碰；跨线程只允许经 `ASYNC_RESULTS`/mailbox 等
值语义通道交割。*（现状实现已满足；沉淀为契约防回退。）

## 4. Q2：handler 阻塞纪律——类型化 vs ui_lint

### 4a. 面向的问题

段化解决的是**等待**（异步 IO），不解决**计算**（CPU-bound 循环占住
update，F7）。另外"handler 里写死循环/长同步 IO"与"worker 直改堆"同属
纪律破坏面。

### 4b. 类型化方案

handler 合成期（handler_codegen）把 handler 体视为受限子语言：禁
`while true`、禁 timer/sleep 原语、对 `#[api]` 外的阻塞 FFI 白名单化。
优点=编译期硬错；缺点=表达能力受限（合法的长计算场景被误伤），需要
逃生舱语法，语法面膨胀。

### 4c. ui_lint 方案

不锁语言，出 lint 层（可挂 handler_codegen 后的 AST 检查 / 运行期预算
观测双臂）：静态臂标红可疑模式（无界循环、sync HTTP 调用），运行臂用
既有 PLAN-026 预算观测（`AUTO_VM_API_BUDGET` / budget WARN）升级为
节流告警。优点=零语法破坏、可渐进；缺点=非强制。

### 4d. 裁定建议（供用户裁定）

**建议 ui_lint 优先 + 段切预算兜底（分两步）**：

1. **第一步（低成本）**：lint 静态臂 + 把 F7 的 budget-exhausted 从
   "WARN 后静默吞"升级为"WARN + handler 失败通道上报"（复用
   `[VM-HANDLER] ... failed` 面），让长计算至少**可见**。
2. **第二步（视反馈）**：BudgetExhausted 段切（cooperative yield：段满
   即 park 回注册表、恢复泵续跑同一段）——这是把现有段驱动自然延伸到
   CPU-bound 的正解，不需要类型系统介入。真正的时间片抢占仍不建议
   （单 VM 串行一致性会碎）。

类型化路线只在"生态出现系统性滥用"时再评估（ YAGNI ）。

## 5. 迁移面评估（两案）

| 案 | 触及面 | 量级 | 风险 |
|---|---|---|---|
| Q1 Go 式（建议） | spec 不变量沉淀 + 无代码 | ~0 | 无 |
| Q1 Erlang 式 | `#[api]` 改写链（codegen emit_api_http_call）→ CPS/两段式；musk/auto-edit/画廊全生态 .at 改写 | 周级，跨仓 | 高（直写生态全断） |
| Q2 ui_lint（建议） | handler_codegen 后置 AST lint + budget 臂升级 | 1 个小计划 | 低 |
| Q2 类型化 | handler 子语言 + 逃生舱语法 + parser/codegen 三层 | 中-大计划 | 中高（语法面膨胀） |
| Q2 段切预算（第二步） | drive_handler_segment 的 budget 臂接 parked 路径 + 恢复泵复用 | 小计划（段驱动已就位） | 低-中（帧中途可见性需验收） |

## 6. 决策请求

1. Q1 store 所有权：**Go 式（建议）** / Erlang 式 —— 裁定后 SD 沉淀。
2. Q2 handler 纪律：**ui_lint + 段切预算两步走（建议）** / 类型化。
3. 裁定产出后由后续计划承接（本工件不实现）。
