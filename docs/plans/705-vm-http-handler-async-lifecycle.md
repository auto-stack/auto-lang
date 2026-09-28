---
plan_id: PLAN-705
status: drafting
feature_name: vm-http-handler-async-lifecycle
author: [agent]
created_at: 2026-09-28
updated_at: 2026-09-28
plan_revision: 1
supersedes_spec_components:
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/async-http-result-lifecycle.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
new_spec_components:
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/vm/engine.rs, crates/auto-lang/src/vm/task.rs, crates/auto-lang/src/vm/codegen.rs, crates/auto-lang/src/vm/ffi, docs/specs/stdlib, docs/specs/auto-lang/runtime]
current_step: 0
total_steps: 8
---

# [PLAN-705] VM HTTP handler 异步等待、请求取消与结果生命周期

## 0. 变更摘要

落实 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 C1。PLAN-699 已把 HTTP/1.1 协议迁到 Axum/Hyper，但 `serve_with` 仍串行调用同步 `dispatch_api_request`，handler 内等待外部 HTTP 会占住 VM owner；网络侧回复超时只丢弃接收端，已排队请求仍可能执行。PLAN-702 已交付同一 AutoVM 的 handler 段执行（park/resume），本计划把它用于服务端请求，并建立请求作用域、完成通知、取消和有界非流式 HTTP 客户端执行。

重点交付：等待上游时仍能服务健康请求；超时/取消的请求不再无效排队和复活结果槽；非流式客户端不靠每请求线程或队满临时线程承载。保留 `api.at` 源码写法与公开返回/错误形式。大范围 task actor/mailbox 改造、CPU 时间片、外部 SSE 流转发、a2r 客户端改造和全后台收敛分后续计划处理。

## 1. 目标

- G1：VM HTTP 普通 handler（`fn() T` 与 `fn() ~T`）遇异步 HTTP/External Future 等待即交还 owner；完成事件恢复原栈，`~T` 解析为最终 T 后响应，允许多个请求挂起，始终只有一个 VM 执行段运行。
- G2：请求作用域覆盖排队、执行、等待、完成、取消；可确认的连接任务销毁、服务回复 deadline、server shutdown 能取消相关等待并释放任务/结果/许可。
- G3：异步结果“登记→完成→消费/取消”单次终结；迟到回调不能重建已取消条目，无完成先于登记的覆写或丢唤醒。
- G4：非流式 HTTP 的 JSON、Response handle 与 RequestBuilder 发送族使用固定 runtime、有限在途/队列与响应体预算；满载有终结性失败，不开临时兜底线程。
- G5：696/698/699 的 API/SSE/关闭语义与 702 的 UI 段驱动保持；Specs 分开描述 HTTP 协议预算、VM 挂起等待和 actor mailbox。

**非目标**：CPU-bound handler 抢占/时间片及 ui_lint（Design 34 Q2 待另案）；多 VM/多 owner 并行、请求原子事务或已发生副作用回滚；取消程序级 detached actor；修改 Auto `task on {}` mailbox 和 `async.at` 的全量 API；通用文件/Socket 异步化；HTTPStream 的异步 Iter/外部 SSE relay；a2r-std/生成 Rust 客户端迁移；Builder/back-proxy 服务器迁移、TLS/HTTP2/WS、补中间件公共 `.at` 声明。

## 2. 架构方案

```text
Axum/Hyper 请求 ── owned request + scope/截止时间 + 生命周期许可 ── VM owner
      │                                                        │
      │          begin/resume → Completed ── ApiReply           │
      │                         Parked ── 续体注册表             │
      │                                         ▲              │
      └─ 确认取消/超时/关闭 ── scope cancel        │ 完成事件       │
                                  └─ client job/future completion handle
                                      固定 Tokio runtime + async reqwest
```

复用 `call_fn_by_name_segment` / `resume_fn_by_name_segment` / `SegmentOutcome`，补齐 HTTP 调用阶段需要的 closure/middleware 段入口与清理边界。HTTP dispatch 拆成可恢复的状态机：预处理/绑定、middleware 链、handler 调用、返回值编组；不在 owner 上同步 sleep 轮询 HTTP 结果，不复制另一套 VM interpreter。跨线程只传 owned 数据、通知/取消句柄，禁止 AutoVM/AutoTask/RC 堆引用跨线程。

本计划新契约：**一个执行段内顺序执行，跨 await 的整个请求不是原子事务**。A 等待上游时 B 可读写同一 VM 共享状态；A 已提交的副作用不会因取消回滚。每个请求独立任务栈与续体，业务需在源码中显式处理跨 await 的状态一致性。此约定写进新 Spec 与示例；不借 HTTP 改造裁定 Design 34 的 UI store/Erlang/Go 分叉。

## 3. 技术栈

AutoVM 段驱动、Tokio LocalSet/mpsc/oneshot/通知与取消句柄、reqwest 0.12 的 async 请求；PLAN-699 的 `http_server.rs` / `http_transport.rs` owned 桥；原始 TCP 和可控本地上游 E2E。实现可新增 `vm/ffi/async_http.rs` 内部模块（新路径），统一结果登记/完成入口；其最终文件拆分由 T-01 决策。Auto 标准库源码签名保持；UI 可继续用既有就绪查询/16ms 恢复泵，新注册表要兼容其消费方式。

## 4. 需求分析与背景调查

### 授权、状态与证据基线

用户要求持续加强标准库 HTTP/task 能力，并于 2026-09-28 告知 PLAN-699 已实现、询问下一个计划。本轮起草下一份计划与更新 Design 33，不执行代码；无指定预算/自动续跑限制。实施按 AGENTS.md 在 `D:/autostack/.wt/lang-705/auto-lang`，计划簿记留 master。

- 主检出取材版本：`9b5a10e51`（2026-09-28），工作区初始 clean。PLAN-699 已 `archived`，delivery commit `4324a5ce4`，复审 `pass`；见 [验证报告](reports/699-verification.md)、[桥接决策](reports/699-bridge-decision.md)、[归档计划](archive/699-vm-http-transport-axum-bridge.md)。当时 `cargo th` 56/56，tf 的既有红/负载抖动有独立定责。本计划重新取自己的基线，不沿用旧数字宣称新代码通过。
- PLAN-702 已归档并合入：`engine.rs` 的 named handler 段执行与 `ui/vm_bridge.rs` 的 parked 注册表/恢复泵已经可复用；[UI Spec ADR-24](../specs/auto-lang/ui/architecture.md) 是现行契约。Design 34 的 store/CPU 纪律建议仍待用户裁定，本计划不将其当成已批准要求。
- 本轮没有同范围 active Plan；主编号最大为已归档 704，`.next-id` 却为 704。经两目录核对、纠正到 705 后以 `scripts/new-plan.sh` 取号（最终计数器 706）；不修改已归档计划。
- 知识源：[全局总览](../specs/overview.md)、[stdlib 项目卡](../specs/stdlib/project.md)、[HTTP Server Spec](../specs/stdlib/design/http-server.md) §7.3/§8.1、[异步结果生命周期](../specs/stdlib/design/async-http-result-lifecycle.md)、[网络集成 Spec](../specs/auto-lang/runtime/design/networking-stdlib.md)、[Design 33](../design/33-stdlib-runtime-and-http.md)。

### 代码事实与待验证风险

- `http_server.rs::serve_with` 从有界队列取请求后同步 `dispatch_api_request`；middleware 经 `call_fn_by_name`，named handler 同样同步，`__axum:` 路由经 `call_closure`。`http_transport.rs::bridge_handler` 的回复超时丢弃 oneshot receiver，owner 没有入队取消判定和 parked 请求范围。
- `engine.rs::drive_handler_segment` 的 legacy `allow_busy_wait=true` 对 HTTP 就绪每 5ms sleep，30s 后删除结果槽；702 段入口已能保留 `HttpRequest` / `Future` 续体，但其恢复触发目前由 UI 查询就绪。CPU 段仍有 10M 指令预算；本计划不许把异步等待公平性写成 CPU 抢占保证。
- `stdlib.rs` 的 `ASYNC_RESULTS` 是全局结果表；`drop_async_result` 只 remove，而 `run_http_json_job`、`spawn_async_http_handle` 等完成端直接 insert。**源码显示迟到结果可在取消后重新插入**，现行 lifecycle §2 的“超时即删、泄漏归零”强结论未覆盖此竞态；T-02 须以确定性顺序实验核实并修复，不凭注释宣称已解决。
- JSON 请求是 2 worker + 容量 64，同步池满后临时 spawn；handle 请求每次 spawn + blocking reqwest；RequestBuilder 另有同步 drain；异步文件 IO/External Future 也有独立完成端。需全量列完成写点以保证共用注册表不留下复活路径，但本计划不把所有 IO worker 一起迁到 async executor。
- `FutureValue` 有 Pending/Ready/Failed 与 owner_task_id；部分 native 直接持 Arc 写状态。新增 owned completion 通知须覆盖本计划可等待的 External Future；保留 Failed→null 的既有值语义，改变错误形状须另列契约修订。

## 5. 详细设计

### 5.1 请求作用域与有限并发

请求入队前获得全生命周期许可，计数覆盖 queued + running + parked，防止开始 park 后有界 queue 变成无界活动任务表；若返回 SSE，许可移交流 body，直至正常结束/断连/关闭释放。owner 开始前检查 scope deadline/取消，失效请求不调用业务函数；执行段结束、重新 park、回传、取消均检查状态。同一 request 只回传一次。复用 `AUTO_HTTP_MAX_INFLIGHT` 时须明确由“队列容量”升级为“请求生命期总上限”的兼容性变化，测试固定小限额。

named handler、既有 middleware 阶段和 `__axum:` closure 均不得留下等待 HTTP 的同步 fallback；T-01 分别证明段入口可行，不能以未知路径绕过 AC。`~T` 的调用/返回表示也须先核实，必要时由 `codegen.rs` 发布最小返回模式元数据，按声明与有效资源身份 await 最终值；禁止把普通 int 的位模式猜成 future ID 后误消费。多个逻辑请求交错而所有 VM 栈/堆操作仍在 owner；不跨 await 持 task/DashMap/Mutex guard，清理先释放读锁再删资源，避免同 shard 自死锁。

### 5.2 完成/消费/取消协议

先登记 live 操作及 scope，后提交 worker；完成端只对仍 live 的令牌提交一次结果并通知。take 仅消费 Ready，探测/提前重入不得删除 Pending；cancel、deadline、scope drop 是幂等终结入口。已缺席/已终结的迟到完成丢弃数据，禁止任意 map.insert 重建。注册就绪通知时复查状态，覆盖“先完成再 park”“取消与完成同时发生”；没有丢唤醒，也不通过固定间隔全表轮询掩盖它。

保留 `async_http_result_ready`/旧 shim 重入接口供 UI/普通 task 消费；所有写点通过同一终结协议，包括仍保留的文件 IO 等消费者。External Future completion 使用可跨线程的 owned handle/通知，不能传 AutoVM。是否将两种结果表合并不预设；T-01 以最小可证明 adapter 决定，不为统一名称扩大 ABI。

### 5.3 客户端执行与取消边界

非流式 JSON、Response handle 和 RequestBuilder.send 走共享固定 runtime + async reqwest；有限活跃操作、有限等待队列、可配置总期限与增量响应体限额，队满立即给现有返回形式的终结性错误（JSON 保持 `{error,status:0}`；handle/builder 保持既有失败通道）。取消丢弃异步请求 future/响应体，重试退避可被取消，绝不队满临时 spawn。默认 headers/query、base URL、headers/body/状态、cookie/proxy/认证/重试/timeout 等既有 builder 行为先列矩阵再迁移，不能只迁成功 GET。

同步显式 `*_sync` 仍是同步 API，不伪装成可取消调用；它们及程序级 actor 不自动归属 request scope。不可中断的遗留 native 在报告中明示，worker 迟到产物仍不得复活注册表。取消不能撤销对端已接受的 POST 或本地 await 前的写入。

### 5.4 断连、deadline 与关闭

网络响应等待 future/连接任务的**已确认销毁或错误**经 scope guard 发取消；回复 deadline 继续保持 699 的 503，关闭信号取消 parked I/O 并释放 owner 注册表。仅 request 输入 EOF/半关闭不能直接判为取消（客户可能半关写端仍读响应）；T-01 要证明 Hyper 生命周期信号，并分别做 TCP RST/任务 drop 与半关闭测试。TCP 关闭不能即时观测的情况由有限 deadline 收口，不声称任意 FIN 后立刻中止。

异步等待期间响应期限/关闭可执行；同步 CPU/阻塞 native 正在执行时只能在下一段边界观察取消，本计划不保证强制抢占。既有 SSE producer/channel 保持，并验证其断连/关闭；HTTPStream/外部 SSE 等待本轮只列缺口，阶段 C2 再实现。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | 无服务端普通 handler 挂起/作用域契约 → 单 owner 段执行、完成唤醒、交错非事务、request scope、取消边界与支持矩阵 | 承接阶段 C1 | AC-01..05 |
| SD-02 | modify | `docs/specs/stdlib/design/async-http-result-lifecycle.md` | remove 即认为永久回收、JSON 队满可 spawn → live token 单次完成、取消不复活、有界 async executor/响应体及兼容消费形式 | 修源码与规则不一致及资源缺口 | AC-02/03/05 |
| SD-03 | modify | `docs/specs/stdlib/design/http-server.md` | §8.1 owner 同步 handler，超时只丢回复 → 普通等待可 park/resume，请求总上限/失效队列跳过，503/关闭/SSE 边界与 CPU 非目标明示 | 协议与执行预算分层 | AC-01/04/05 |
| SD-04 | modify | `docs/specs/auto-lang/runtime/design/networking-stdlib.md` | 阶段 C 整体未落地 → C1 普通等待/取消已验证，actor/mailbox、外部流/a2r 仍独立未迁 | 避免扩大支持声明 | AC-05/06 |

执行期只记录拟议 delta；独立 review 核对真实代码与支持表，merge 时沉淀 canonical Specs/ledger。不得提前把全量 task/流式能力标为完成。

## 6. 测试设计

- 可控本地上游 gate 延迟响应，两个 VM handler park，再发 `/health`：上游未解除前健康请求已完成；解除后原请求依各自结果返回。覆盖 `T`/`~T` 最终值（不返回 future ID）、普通 int 位模式反例、chained await、栈/局部值、外部 Future 和错误；不靠长固定 sleep 断言并发。
- 结果通道竞态用屏障精确控制：register→complete→consume、Pending 时探测/重入仍保 live、complete-before-wait、cancel-before-complete、complete/cancel 争用、重复 complete/drop；反复执行后 pending/completed/tasks/permits 均回基线，无迟到结果槽。
- 队列/active/client budget 设小值；记录总许可数量、客户端活跃 job 数、固定 runtime 线程数量、超限响应；巨大/缓慢上游响应触发有限体积/期限失败，取消重试不继续发送。
- request deadline、确认断连、shutdown 均有挂起请求清理测试；排队失效后 handler 的副作用计数不增长。半关闭请求仍得到正常响应；已执行 await 前副作用保留（非回滚契约）。
- 兼容矩阵：JSON/handle/builder 各正常/4xx/5xx/连接失败/timeout/retry/headers/base URL/body；middleware/`__axum:` 复用仓内可运行 adapter fixture；015/017/023 及 696/698/699 SSE 回归。共享 engine/result 变化需复跑 702 engine/UI 段测试，不因本计划改其重入/UI tick 策略。
- 门禁按 2026-09-28 AGENTS.md：迭代 `cargo check -p auto-lang` 和定向测试；最终 `cargo th` 与**一次 `cargo tf`**（已包含语料族，PLAN-700）。`cargo tv` 仅可选定向快捷档，不重复当全量必经；不触及 aavm 则不跑 `cargo taa`。本次 docs-only 起草不运行 Cargo/docs_gen。

## 7. 验收标准

| ID | 可观察结果 | 验证方法与期望 |
|---|---|---|
| AC-01 | 普通 HTTP `T`/`~T` handler/middleware/合成 closure 的受支持异步等待交还 owner；两个等待中请求与 health 并存，原栈正确续跑且返回最终 T | gate 上游 E2E（health 在 gate 解除前返回）；`~T` 最终值/普通 int 反例、chained await/closure/External Future 段测试；默认 HTTP 调用图无忙等 fallback |
| AC-02 | 取消/消费操作只终结一次，迟到完成不复活结果表，不丢唤醒 | 屏障竞态测试、所有共用完成写点审查；重复取消/完成后注册表回基线 |
| AC-03 | 非流式 JSON/handle/builder 无每请求线程及队满 spawn；client job/queue/body/期限有实际生效边界 | 小容量饱和/巨大慢上游/重试取消探针；线程稳定为固定 runtime，满载错误可消费且无永久 Waiting |
| AC-04 | queued+running+parked 总量有限；队列失效不执行函数；可确认断连/响应 deadline/关闭取消 parked 等待，任务/许可回收 | 小许可多请求+副作用计数+取消/关闭 E2E；半关闭仍正常；CPU 正在执行的例外明示 |
| AC-05 | API 返回/错误、绑定、Response、SSE 和 UI 段执行无新增回退；request 跨 await 非事务语义明确 | 015/017/023、th、702 定向回归、客户端兼容矩阵；单 owner/no VM 跨线程审查；SD-01..04 源码对证 |
| AC-06 | 范围门禁/健康扫描通过，全部 AC、完成写点、支持缺口与 Spec delta 有独立复审证据 | check 双 feature 形态、th、一次 tf、diff/格式/新告警核查；报告记录既有独立红项 |

## 8. 执行步骤

验证命令：快速编译为 `cargo check -p auto-lang`（纯 VM 另跑 `--no-default-features`）；新增探针统一含 `plan705` 名，定向执行 `cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(/plan705/)'`；共用结果/段驱动回归用同命令过滤 `test(/p027|engine_segment|plan702/)`；末次门禁用 `cargo th` 与一次 `cargo tf`。T-01/T-02 新探针未建立前先运行所依赖的现有定向项，不能拿空过滤结果当通过。

| 任务 | 依赖 | 精确位置/动作与产出 | 验证与关联 |
|---|---|---|---|
| T-01 [ ] | 无 | 在专用 worktree 核查 `http_server.rs::serve_with/dispatch_api_request`、`http_transport.rs::bridge_handler`、`engine.rs` 段/closure/RequestBuilder drain、`codegen.rs` 的 `~T` handler 返回表示、`stdlib.rs` 所有结果/Future 完成点；做最小可编译 park/notify/cancel spike；新增 `docs/plans/reports/705-async-decision.md` 冻结入口矩阵、作用域/总许可、默认 client 限额、Hyper 断连判据 | check/定向 spike；本轮 gate 的预期红、兼容消费矩阵；AC-01..05。无法覆盖某生产等待路径必须 replan，不得隐藏同步 fallback |
| T-02 [ ] | T-01 | 在 `stdlib.rs`（可新增 `vm/ffi/async_http.rs`、改 `ffi/mod.rs`）集中 live 操作登记/完成/take/cancel 与 owned 通知；全部 ASYNC_RESULTS 完成端经安全提交，包括保留的文件 IO；适配 External Future 完成句柄，保 UI 就绪消费接口 | 竞态/迟到完成/重复操作定向单测绿；702 段测试；AC-02/05 |
| T-03 [ ] | T-02 | 非流式 JSON/handle/builder 使用固定 async reqwest executor；从 `engine.rs` RequestBuilder send drain 移除 handler 等待的同步 drain；实现排队/active/body/期限/重试取消上限，保 builder 属性；去掉池满线程兜底 | check；客户端矩阵、饱和/大响应/取消重试探针；AC-03/05 |
| T-04 [ ] | T-01..03 | `engine.rs` 复用/补充可恢复调用段入口；`http_server.rs` 将 middleware/handler 编组改成可恢复阶段、解析 `~T` 最终值（必要时 `codegen.rs` 发布返回模式）；`serve_with` 接完成通知与公平入队处理，全部 VM 状态仅 owner 访问 | 上游 gate+health、`~T` 最终值/普通 int 反例、两个 park/chained await/closure/失败恢复 E2E；AC-01/05 |
| T-05 [ ] | T-04 | `http_transport.rs` 桥携 scope guard/deadline/生命期许可；`http_server.rs` 跳过失效队列、取消 parked 任务/操作、关闭排空；定义 permit 与返值/SSE producer 生命周期交界 | cancel/deadline/shutdown/半关闭 E2E，资源计数回基线与 await 前副作用保留；AC-04/05 |
| T-06 [ ] | T-03..05 | 在现有 `http_server.rs` HTTP E2E 区扩齐 named/middleware/closure/client 兼容探针；重跑 015/017/023、696/698/699 SSE/关闭及 702 engine/UI；新增 `docs/plans/reports/705-parity.md` | `cargo th`、702/async result 定向测试；矩阵每行追溯命令/commit；AC-01..05 |
| T-07 [ ] | T-06 | 固定小限额压测、取消风暴、晚完成回收/线程数量探针，记录时间与资源界限；核查所有完成写点/默认 HTTP 调用图无同步忙等；输出 `docs/plans/reports/705-resource-lifecycle.md` | 同样配置重复可复现；无资源斜率/permit 漏失与假并发；AC-02..04 |
| T-08 [ ] | T-07 | 按当前门禁 check（含 `--no-default-features`）/th/一次 tf；diff/格式/新告警检查、延期与 workaround 扫描；准备 SD-01..04 与独立复审，新增 `docs/plans/reports/705-verification.md` | 验收项逐一有证据，独立红项定责；review/merge 另阶段执行；AC-05/06 |

## 9. 复审记录

- 2026-09-28，`stage: new`，`PLAN-705:r1`，`outcome: pass`，`next: work`（从 T-01 决策/基线开始）。已核对 699/702 已交付、当前 Specs/执行路径、编号唯一；AC-01..06 与 SD-01..04 均映射到 T-01..08。此为计划起草交接，未实施/测试代码，亦非独立复审通过。实施以本计划范围确认后创建的专用 worktree 为准。

## 10. 待澄清事项

1. T-01 执行者负责裁定 named/closure/middleware 段状态机、External Future 通知与 RequestBuilder yield 入口；不预设结果表物理合并、不增语言语法。
2. client/response body/许可限额值由 T-01 对现有消费者与小规模负载核查后冻结；本计划要求有限边界与明确失败，不能取无限默认逃避 AC。若默认失败面影响真实消费者或需改变返回形状，修订 plan_revision 并呈报具体兼容性变化。
3. Hyper 可确认的连接任务生命周期与 TCP 半关闭不同，T-01 先证明取消条件；无法即时观测的断连以 deadline 有界收口，不能声称任意 EOF=取消。
4. CPU 时间片/长同步 FFI、全量 `task` actor、外部 SSE/HTTPStream 与 a2r async 客户端均不在本轮；新支持矩阵应逐项标出剩余能力，后续 C2/D 从实际交付另立计划。
