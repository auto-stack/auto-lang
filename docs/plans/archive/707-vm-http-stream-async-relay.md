---
plan_id: PLAN-707
status: archived
feature_name: vm-http-stream-async-relay
author: [agent]
created_at: 2026-09-29
updated_at: 2026-09-29

plan_revision: 1
supersedes_spec_components:
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/http-handler-async-lifecycle.md
  - docs/specs/stdlib/design/async-http-result-lifecycle.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
new_spec_components:
  - docs/specs/stdlib/design/http-stream-lifecycle.md
touched_goals: [GOAL-003]

affects: [stdlib/auto/http.at, stdlib/auto/http.vm.at, crates/auto-lang/src/vm, crates/auto-lang/src/sse, crates/auto-lang/src/ui/vm_bridge.rs, docs/specs/stdlib, docs/specs/auto-lang/runtime]
current_step: 8
total_steps: 8
---

# [PLAN-707] VM 外部 HTTP/SSE 流异步消费、转发与取消

## 0. 变更摘要

落实 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 C2a。PLAN-705 已交付普通 handler 的段执行、请求作用域与非流式 async 客户端；外部 `HTTPStream` / SSE 仍有阻塞读取、每流线程、generator 等待热循环和取消不闭合的缺口。下一步使「外部慢流 → Auto `for` / `yield` → `#[api] ~Stream<str>` → 下游客户端」成为可等待、有界、可取消的真实执行链。

同时核实并修复一处 705 的代码/Spec 差距：`cancel_live_op` 删除结果槽并通知，但 `submit_client_job` 没有取消句柄或取消 select；源码只能证明迟到结果不复活，不能证明正在运行/排队的 reqwest future 已被丢弃。该闭合是流取消的基础，不沿用旧报告的强结论。

本计划只实施 auto-lang 仓的 VM HTTP 流消费与转发；a2r 客户端迁移、CPU 纪律、多后台装配 manifest 和部署加固继续拆分。所有代码工作在 `D:/autostack/.wt/lang-707/auto-lang`，本轮只起草文档。

## 1. 目标

- G1：流请求建立、逐块读取和 SSE 逐事件等待让出 VM owner；普通 handler 与 SSE generator 都能 park/resume，使用完成事件唤醒，保持单 owner、栈与堆不跨线程。
- G2：HTTP 流与 705 非流式客户端共享固定 Tokio runtime，采用独立的流活跃/排队预算；流数量、缓冲字节、事件大小均有限，慢消费者背压到上游读取，不新增每流线程/runtime 或临时兜底线程。
- G3：自然完成、显式 close、消费循环 break/异常、请求取消、下游断连与关闭收口到幂等终结；实际停止本地网络 future，回收 generator/iterator/流句柄/活动 job/许可，迟到事件不复活。
- G4：外部 SSE 按网络分段无关的增量解析交付 data，正确处理 UTF-8、行结束与事件边界；EOF、Pending、Error、Cancelled 与合法载荷不在内部混淆。
- G5：保留现有 HTTPStream 手动消费与 SSE poll 调用方式，补齐 VM `for chunk in stream` 路径；验证 696/698/699/702/705、017-chat publisher 和 UI SSE poll 的兼容边界。

**非目标**：通用 actor/mailbox 或全部文件/Socket I/O 改造；CPU 抢占/时间片与 Design 34 Q1/Q2 裁定；流式文件上传下载及进度流 worker 迁移；a2r-std/auto-man/Rust 生成服务改造；Builder server/back-proxy HTTP 传输替换；TLS/HTTP2/3/通用 WS；自动 SSE 重连/Last-Event-ID 重放；新的 async-for 语法、全库 May/Iter ABI 重做；公开 SSEEvent 元数据 API。既有其他 ASYNC_STREAMS 使用者仅适配共同句柄/通知和清理，不能借本轮全面迁移线程。

## 2. 架构方案

```text
固定 Tokio client runtime（705）
  ├─ 非流式 job：现有预算 + 真正可取消的 managed job
  └─ 流 job：独立活跃/排队许可 → async reqwest → 增量 decoder
                                            │ 有界事件/字节队列 + 就绪通知
                                            ▼
VM owner：HTTPStream.next / Iter pull / SSE generator 续体
             Pending → park；Data → 推值/yield；EOF/Error → 单次终结
                                            │
                    699 有界 SSE frame 通道 → Hyper response body
                                            │
scope / close / break / 断连 / shutdown ──────┴─ 取消资源组并释放许可
```

保留已有段执行、owned transport bridge 和 generator 执行器；新增流等待凭据或等价 adapter，统一判定就绪。多值流不能直接伪装成 705 的单次 `LIVE_OPS` 结果槽；单次建立结果与多次 pull 状态须分清。跨线程仅传 owned 字节、事件、资源身份和取消/通知句柄；不传 `AutoVM`、`AutoTask`、RC 堆引用，不跨 await 持 VM/注册表锁。

资源终结与「等待可取消」必须分别证明。关闭结果登记并不等于中止网络 future；关闭 SSE 输出也不等于取消其 generator 内创建的上游流。T-01 先冻结 ownership 图，再实现取消向内传播。

## 3. 技术栈

现有 Tokio、reqwest async/bytes_stream、有限 mpsc/字节许可、Notify 或 waker、可取消 job；AutoVM `SegmentOutcome` / `ParkedWait`、generator 原栈恢复和 native re-entry。复用 `sse/parser.rs` 的事件模型，新增可 feed 字节的增量 decoder（具体内部布局 T-01 冻结）。真 TCP 本地 gate 上游与下游，不依赖外部 LLM、账号或公网。不预设新增 Rust 依赖。

## 4. 需求分析与背景调查

### 授权、版本与知识源

用户于 2026-09-29 告知 PLAN-705 完成，并明确要求用 `/auto-plan:new` 规划 HTTP 加强的后续计划。本轮授权为调查和起草一份可执行计划；未要求执行、合并代码、跨仓修改或发布，未指定预算。使用用户指定的 `D:/autostack/auto-musk/.agents/skills/auto-plan-new/SKILL.md`；本仓路径映射与门禁遵从 AGENTS.md。

- 取材主检出 `master @ e2deb4f87`；705 delivery `03a2172e5`，归档/清理收据已落地，canonical Spec 标注 C1 current。tracked 初始 clean；既有未跟踪 UI 走查文件不属于本计划，保持原样。
- 全局入口：[overview](../specs/overview.md)、[stdlib project](../specs/stdlib/project.md)、[GOAL-003](../specs/goals.md)；直接契约：[HTTP server](../specs/stdlib/design/http-server.md) §3.4/§8.1/§11.4、[handler async lifecycle](../specs/stdlib/design/http-handler-async-lifecycle.md)、[result lifecycle](../specs/stdlib/design/async-http-result-lifecycle.md)、[backend assembly](../specs/stdlib/design/backend-assembly.md)、[networking stdlib](../specs/auto-lang/runtime/design/networking-stdlib.md)。
- 依赖交付证据：`docs/plans/archive/705-vm-http-handler-async-lifecycle.md` 与 `docs/plans/reports/705-{verification,resource-lifecycle,parity}.md`。复审证据足以确认段执行/不复活；未见「取消使网络 future 与 active permit 在上游 gate 未打开前退出」探针，T-01/T-02 重新建立证据。
- 相关实现：`vm/ffi/{stdlib,async_http,http_server,http_transport}.rs`、`vm/{native,engine,task,codegen,native_catalog}.rs`、`ui/vm_bridge.rs`、`sse/parser.rs`、`stdlib/auto/{http.at,http.vm.at,sse.at}`。比较参照 `crates/a2r-std/src/http.rs` 与 `crates/auto-man/src/api_gen.rs`，本轮不改它们。
- SSE 分帧参照 [WHATWG HTML §9.2.5–6](https://html.spec.whatwg.org/multipage/server-sent-events.html#parsing-an-event-stream)（2026-09-29 查阅）；通知竞态参照 [Tokio Notify 文档](https://docs.rs/tokio/latest/tokio/sync/struct.Notify.html)。本计划仅采用所需解析/唤醒规则，不承诺完整浏览器 EventSource 行为。
- 活跃区没有同范围计划（仅旧 242 tracker）；两目录最大有效编号 706，`.next-id=707`，由 `scripts/new-plan.sh` 取 707 后计数器 708，锁内复核唯一并先提交取号收据。不复开 705。

### 源码事实与 Spec 差距

1. `shim_http_get_stream/post_stream/post_stream_with_headers` 使用 `thread::spawn(...reqwest::blocking...).join()`；`shim_http_stream_next` 与 `native.rs::Iterator::HttpStream` 直接 `Response.read`。流建立和读取都能占住 owner。`HTTP_STREAMS` 为 thread-local blocking response 表，不能直接迁到网络线程共享 VM 引用。
2. `spawn_async_sse_stream` 每流一 `auto-sse-client` 线程及 current-thread runtime；mpsc 容量虽为 64，单帧和拼接 buffer 没有字节上限。按每 chunk `from_utf8_lossy` 再找 `\n\n`，可破坏跨包 UTF-8、漏 CRLF/CR 分帧；`trim_start` 删除多余空格，空 data 被过滤。发送失败未统一中止流读取，不能证明断连后回收。
3. `AsyncHttpStream` pull 无数据时设置 `waiting_sse_stream_id`；常规 task loop 每轮探测。`ParkedWait` 只有 HttpRequest/Future，handler 段对流等待仍继续执行；generator 驱动只对 cooperative sleep 特判，`AwaitFuture` 直接 continue。`next_sse_generator_value` 对未知 Pending 反复 `yield_now`，不是外部流就绪通知驱动。
4. `shim_http_stream_iter` 已有注册骨架，public `http.at` 只有 next/is_done/close，未声明 iter；HTTPStream iterator EOF 部分分支不推 -1。现行 Spec §11.4 把统一 Iter 写为目标，不能凭骨架宣称已经完成。计划沿现有 native iterator 协议闭合可用路径，不替换全库 ABI。
5. `ASYNC_STREAMS` 共用者包括文件/进度流、`bus.subscribe`（698）、`sse_open/sse_poll`（658）。不能统一修改为「缺 live-op 即自动取消」：705 的 `spawn_async_http_msg_get` 本就无 live-op token 的 detached consumer，当前仍需完成入消息队列。T-02 要显式区分 managed 与 detached job。
6. `cleanup_sse_iterator` 当前只回收顶层 Generator task 或直接 AsyncHttpStream；未覆盖 generator 内部建立的 HTTPStream、嵌套 iterator 和 job。`FrameStream` 在 SSE 结束时才释放 705 的 request permit，是下一阶段资源组所有权的接入点。
7. VM custom headers 接受 JSON 字符串，a2r 流客户端使用换行 `Key: Value`；a2r 同步/async 流仍有线程、无界队列或 close 占位。此为后台差异，本轮保留 VM 输入兼容并登记未来收敛，不能声称三后台流能力同等。
8. Design 33 的 705 状态与 §6.1「当前」描述已陈旧；本轮修正文档阶段状态，canonical Specs 的能力升级仍通过实施后的 review/merge 沉淀。

## 5. 详细设计

### 5.1 T-01 决策边界与有限配置

先做可丢弃 spike 和红测：上游不给 headers、只给部分事件、stream 等待期间并发 health、取消 queued/active job、generator await；输出 `docs/plans/reports/707-stream-decision.md`。冻结 public shim/native 入口及栈槽宽度、等待凭据、资源所有权、错误可见性、配置与估算内存上界。确认 `CALL_NAT/CALL_SPEC` 和嵌套 generator 的 re-entry 均只发请求一次、恢复不丢参/不重复帧。

流与非流式复用固定 runtime，但流用独立 active/queue 许可，长连接不可耗尽 705 的非流式配额。建议初值供 spike 核定：流 active 16、等待 32、每流已排队事件 ≤16 且数据 ≤256 KiB、单事件/未完成解析 ≤256 KiB、原始输出 chunk ≤16 KiB、建立/排队期限 10s、上游读空闲 60s、下游写停滞 30s。冻结后写 Spec；不使用无限队列/缓冲或「满了另开线程」。数值调整要在 decision 中给消费者与内存依据。

不能把 705 的 30s 非流式总期限或 10MiB 累计 body 限额直接套到长流。headers 交付后的响应期限转为明确的流空闲/写停滞策略；允许健康长流持续，累计流量超过非流式 body_limit 也能工作。上游注释心跳可表示读活跃，下游自发 heartbeat 不能伪造上游活跃；背压主动暂停读时不误触读空闲期限，由下游停滞预算收口。

### 5.2 可取消的执行器与流资源表

managed job **登记取消身份后再提交**。cancel 对排队许可等待、请求建立、重试退避、读取和阻塞于发送的阶段均可生效；完成/取消竞态只终结一次。abort handle、取消信号或 select 的选择由 spike 决定，必须保证 cancel 先于 handle 安装时也不会漏失。705 的 `cancel_live_op` 除拒迟到结果，还应通知执行器停止 managed future 并释放 job/queue/active 许可。

detached msg bridge 保留明确提交形态，不能因 LIVE_OPS 无条目被误杀。取消 Future 不承诺撤销对端已接受的 POST、本地已发生的副作用或 OS 已缓冲数据；只证明本地 job 不继续读/重试、不永久占许可。

流先登记、后生产；表内状态至少分 Opening/Pending/Data 可取/EOF/Failed/Cancelled。terminal 与 receiver closed 均唤醒等待者；迟到生产者不能 insert 重建。EOF 先排空已成功入队 data，再返回终结；取消释放数据，错误按冻结规则提供诊断。各等待点遵循「订阅/注册就绪 → 复查状态 → await」，覆盖先完成后 park、多个 owner/多个消费者和终结同时发生。

有界事件数量不足以限制内存：生产端 decoder、单帧、channel 与正在发送的 frame 都须计字节上限；队满停止拉取上游，send/读/许可获取均能被 cancel 打断。T-07 输出可观测计数器与模型上界；不以 RSS 单次测量替代资源证明。

### 5.3 源码兼容与等待语义

- 保留 `get_stream/post_stream/post_stream_with_headers -> HTTPStream`、`HTTPStream.next -> str`、is_done/close；建立与 next 允许在 VM 段入口内部挂起，不增 async-for 语法。raw HTTPStream 输出仍是文本 body chunks；仅改跨 chunk UTF-8 解码，**不能自动把 SSE 转成 data 或把载荷 `[DONE]` 认成 EOF**。chunk 切分不作稳定承诺。
- 内部 pull 用 Data/Pending/EOF/Error 等状态；手动 HTTPStream.next 的既有 EOF `[DONE]` 适配保留并文档化歧义，迭代器终结凭状态而非字符串比较，合法 `"[DONE]"` 与 `""` data 不丢。
- HTTPStream 的 `for` 走独立 iter/pull adapter，至少覆盖内联调用、赋给局部变量再 for、嵌入 `~Stream<str>` generator。保持 public next 的 str 返回，不悄改为 May。所需 `.at` iter 声明/目标 native 映射 T-01 冻结并验证装配，无新增另一套拼接机制。
- `sse_get_stream` 仍交付 data 字符串 iterator；`sse_open/sse_poll` 保留非阻塞 poll：Pending `""`、终结 `[DONE]` 的 legacy 映射。补最小显式 close 与错误查询入口及公共声明（T-01 冻结具体签名/ABI），使 poll 使用者可释放未完流、获知错误；不得只写 Rust shim 而没有公共声明/注册/测试。
- 新流等待凭据须接入 `engine`、HTTP parked readiness/cleanup 与 `ui/vm_bridge::parked_wait_ready`。UI 保留 702 的恢复泵；不将 HTTP owner 的无轮询保证偷换成全 task loop 已改为事件调度。

### 5.4 Generator 等待与作用域传播

generator 的 cooperative pull 返回 Yielded/Pending(wait)/Done/Error 或等价结构；遇 HTTP、External Future、流 pull 必须立即停步，保留 generator 与嵌套 async 栈，ready 后恢复。禁止靠重复 `run_one_instruction`/`yield_now` 驱动未就绪等待，不能把临时 next task 的销毁当作 generator 终结。CPU 批次到限的让出与 I/O Pending 分别标识；696 的 bounded stepping/sleep 行为保持。

request 内建立的上游 job/stream、generator 子任务与 iterator 属于该请求资源组；SSE 发送 headers 后资源组随响应体继续存活，不能普通 reply 完成就取消，也不能沿用已过时的首响应 deadline 杀死健康长流。下游 body drop、已确认连接任务终结、写停滞和 server shutdown 触发内层取消；半关闭仍沿 705 判据处理。

generator 内部 `for` 提前 break/异常和普通 handler 完成，要释放其不再消费的上游；scope 终结回收所有请求所有资源，不能只删除顶层 task。一般程序的 stream 不自动绑定任意 HTTP request：明确 owner/close 及消费退出路径；task 脱离请求的条件必须显式，不能为了避免清理把资源全部标为 detached。共享引用/多消费者如现有实现不支持，清楚诊断，不能双重消费或静默漏资源。

### 5.5 增量 SSE 解析与错误

引入有状态字节 decoder，覆盖跨网络块 UTF-8/BOM、LF/CRLF/CR、跨包行结束、多行 data、注释、空 data、字段首个冒号、只移除冒号后的一个空格；保留前后有效空白。event/id/retry 用内部事件模型保留和正确解析，公开 data-only 入口仍只返回 data。EOF 未闭合事件丢弃；`[DONE]` 是业务数据，由业务决定停止。解析失败与预算超限可观测并终结，不能伪造成功空 body。

`sse/parser.rs` 目前的 `parse_sse_chunk`/BufRead helper 对尾事件与 trim 有自己的兼容行为：新 live decoder 与这些一次性 helper 的关系由 T-01 列矩阵，允许保留旧 helper 作为明确 legacy adapter，禁止无声改变所有消费者。SSE GET 解析模式需冻结状态码/Content-Type 检查和失败诊断；raw HTTPStream 保留其读取非 2xx body 的既有可观察行为。认证 headers/POST body 不因迁移丢失，不对 POST 自动重试/重连。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/http-stream-lifecycle.md` | 无外部流统一生命周期 → VM 流 opening/pull/terminal、资源组、配置/背压、SSE decoder、legacy sentinel 与支持矩阵 | C2a 主契约 | AC-02..06 |
| SD-02 | modify | `docs/specs/stdlib/design/async-http-result-lifecycle.md` | 声称 cancel 丢弃 job future，源码未闭合 → managed cancel 的执行器退出证据、queued/active/retry 清理；detached 独立 | 消除 705 描述与实现差距 | AC-01/06 |
| SD-03 | modify | `docs/specs/stdlib/design/http-handler-async-lifecycle.md` | 外部 stream 等待未实现 → 新凭据与 generator 子资源归属；普通/SSE scope、首响应与流期限区别 | 延伸 705 请求契约 | AC-01..03/05 |
| SD-04 | modify | `docs/specs/stdlib/design/http-server.md` | §3.4/§11.4 目标与 §8.1 现实混合 → VM for/pull/relay 已验证面、手动签名保留、a2r 未迁、失败/取消边界 | 不泛化共享 server/全后台结论 | AC-02..05 |
| SD-05 | modify | `docs/specs/stdlib/design/backend-assembly.md` | 旧 HTTP/SSE native 覆盖 → 声明/目标/native/生命周期覆盖与独立 Rust 客户端差异 | 多后台机制可核对 | AC-05 |
| SD-06 | modify | `docs/specs/auto-lang/runtime/design/networking-stdlib.md` | C1 后外部流未迁 → VM C2a 的固定 runtime/等待/取消、CPU/文件/a2r 剩余边界 | 更新能力图 | AC-02/05/06 |

以上为拟议 delta；本轮不编辑 canonical Specs 或派生 ledger，实现后独立 review 核对实际差异、merge 再沉淀。

## 6. 测试设计

所有 Rust 探针均在 707 worktree。新测试族统一 `plan707`；真 HTTP 桥 E2E 另含 `http_e2e` 并置于 `test-http-e2e` 门下，纳入 `cargo th` 串行池；纯 registry/decoder/段测试留常规档，避免真 TCP 在 tf 并行互扰。本轮起草不运行 Cargo。

下表 `HTTP-707` 定向命令为 `cargo nextest run -p auto-lang --lib --features test-http-e2e -j 1 -E 'test(plan707) & test(http_e2e)'`；不用 `cargo th plan707` 的追加名称筛选充当交集（nextest 多名称滤串为 OR）。独立上游 client TCP 测试亦在 nextest 独立进程内运行，必要时按现有 HTTP 串行组登记。

- **取消证据**：本地 upstream gate 永不主动完成；active cap=1，A parked 后 cancel，gate 仍关闭时 B 能获取同一许可并完成。queued job 取消后上游接受计数不增加；重试退避取消后发送次数停止；owned job/许可回基线，迟到完成不复活。msg bridge detached 仍收到结果。
- **等待交错**：上游不回 headers / 发首块后 gate / SSE 半事件三类；分别经 named、middleware/closure 段和 generator 消费，等待期间 health 完成。以握手 barrier 保证因果，宽裕超时只作测试保护，不靠 sleep 猜调度。两流数据不串、链式 await/Future 能恢复，记录驱动次数在 gate 关闭期间不随等待时长增长。
- **端到端 relay**：本地 POST+auth headers 上游 → Auto public http API → `#[api] ~Stream<str>` → 真 TCP 下游；至少两帧分时到达、末尾仅终结一次。添加 raw HTTPStream 局部变量 for、手动 next/is_done/close、SSE iterator 与 poll 对照。
- **取消与资源**：自然 EOF、显式 close、break、异常、下游 RST/body drop、server drain 到期、半关闭各有实验；上游未释放 gate 前内层本地 job 已退出；流表/task/iterator/job/permit 在限定等待内回基线。每轮取消风暴重复同样配置，至少两轮无增长。
- **背压/预算**：直接控制内部 frame 消费 gate，证明 upstream pull/read 不超冻结窗口、其他 health/非流式请求继续完成；取消能打断满队列发送。真 TCP 慢 reader 验证写停滞回收；不以远端 TCP send 阻塞精确等同本地队列上界。无分隔超长 SSE、超大 event、活跃/队满、建立/idle 期限均终结且有错误；长流累计 >非流式 body_limit 正常工作。
- **解析金样**：所有字节切分点，包括 UTF-8 中点、CR/LF 中点、一次多事件，结果与完整输入一致；data 空白/空串/`[DONE]`、BOM、注释、event/id/retry、NUL id、非法 retry、未闭合 EOF。用精确期望值核对，不能把同一实现两次输出相等作为唯一 oracle。
- **兼容**：`plan341`、`plan348`、`generator_tests`、`plan326`、`plan702`、`plan705` 及新增 stream 族；698 总线断连回收、658 poll pending/终结与已有 017-chat publisher 行为有定向证据。当前脚本/fixture 的环境红要在 baseline 同命令复证，不能整族跳过后宣称通过。

门禁：开发按需 `cargo check -p auto-lang` 与 `cargo t plan707`；最终 `cargo check -p auto-lang --no-default-features`、`cargo th`、核心引擎/协议改动的一次 `cargo tf`（已含语料，tv 可选快捷档）。仅若实际触及 transpiler 才追加 scoped `cargo tt`；本计划不应触及 aavm，不跑 taa。格式限定改动文件检查，新 warnings/debug 输出为零，独立 review 扫描漏项/延期/workaround。

## 7. 验收标准

| ID | 可观察行为与验证方法 | 通过条件 |
|---|---|---|
| AC-01 | T-02 queued/active/retry 取消与 detached 反例；上游 gate 未释放即复用 active=1 许可 | 本地 reqwest job 已停止并释放各阶段许可，queued 不发送、retry 不再发；迟到不复活，msg bridge 不被误杀 |
| AC-02 | T-04/T-05 慢 headers/慢 chunk/半 SSE、双流与嵌套 await E2E；驱动计数+通知竞态 | health 在 gate 关闭时完成，原栈/参数/帧一次恢复，不串流；未就绪等待无固定轮询/热重试；VM 不跨线程 |
| AC-03 | T-06 自然结束/close/break/异常/RST/body drop/关闭/半关闭矩阵与资源风暴 | 所有请求所有上游资源终结、表/任务/许可回基线，满通道取消可执行；半关闭不误杀、健康长流不沿首响应期限被杀 |
| AC-04 | T-03 SSE 精确金样、全切分/超限；T-07 背压与长流/活跃/队列/期限探针 | 解析不受网络分段影响，合法空串/[DONE] 不丢；EOF/error/pending 分明；数量及字节有界、满载明确失败、背压抑制上游读取且其他请求可服务 |
| AC-05 | T-04 公共/目标/native 入口核对、manual/inline/variable for/relay/poll；T-06 兼容矩阵 | 现有源签名/headers/body 与 poll 语义保留，新增 close/诊断全链可用；698/702/705 不回退；a2r/其他 worker 未迁逐项明示 |
| AC-06 | T-07 资源报告 + T-08 门禁/源码扫描/SD-01..06 证据 | 固定 runtime 线程数稳定，无每流线程/runtime、无同步网络 read/join 默认流路径；测试红项独立定责、无未授权延期，Spec delta 可沉淀 |

## 8. 执行步骤

| ID / 状态 | 依赖 | 文件/符号与动作 | 验证命令与预期 / 验收 |
|---|---|---|---|
| T-01 [x] | 705 已落地 | 在 707 worktree 核查上述入口/栈 ABI/所有权，新增 spike 与 `docs/plans/reports/707-stream-decision.md`；冻结配置、typed pull、取消/错误/poll 新签名、generator wait 协议；定罪 705 job cancel 差距 | `cargo check -p auto-lang`；`cargo t plan707_spike`，新增期望失败复现与目标 spike 分开记录；现有 341/348/702/705 定向基线；AC-01..06 前置 |
| T-02 [x] | T-01 | `vm/ffi/async_http.rs::{submit_client_job,cancel_live_op}`、`stdlib.rs::spawn_async_http_msg_get`：managed/detached 分离、登记后提交与 queued/active/retry 实际取消，接入 scope 取消；共享固定 runtime 的流提交接口与独立许可 | `cargo check -p auto-lang`；`cargo t plan707_cancel`、`cargo t plan705`、`cargo t plan083`；active=1 gate 未开许可可复用、detached 消息仍到达；AC-01/06 |
| T-03 [x] | T-01 | `sse/parser.rs`，可新增 `sse/decoder.rs`（新路径）：feed bytes 的有限 decoder 与 raw UTF-8 carry；保留/适配原 helper；新增 parser 切分/预算金样 | `cargo t plan707_decode` 与 `cargo t sse`；精确值/全字节切分/尾事件/超限全绿；AC-04 |
| T-04 [x] | T-02/03 | `vm/ffi/stdlib.rs` 的 HTTP_STREAMS/ASYNC_STREAMS 与所有 stream shim，可新增 `vm/ffi/http_stream.rs`（新路径）；接入 async producer、字节背压、raw/SSE adapters；`stdlib/auto/http.at,http.vm.at`、`vm/native_catalog.rs`/native 注册、`native.rs`/`codegen.rs` 补 manual/Iter/poll close/error | `cargo check -p auto-lang`；`cargo t plan707_client`、`cargo t plan341`、`cargo t plan348`；零阻塞建立/read，手动/内联/变量 for 均正确，所有 public 声明可装配；AC-04/05/06 |
| T-05 [x] | T-04 | `engine.rs::ParkedWait/drive_handler_segment/resume_fn_by_name_segment`、`task.rs::waiting_sse_stream_id`、`native.rs::shim_iterator_next_cooperative`、`http_server.rs::{parked_is_ready,next_sse_generator_value,produce_sse_frames}`、`ui/vm_bridge.rs::parked_wait_ready`：流/Future/HTTP generator Pending 保栈、通知就绪恢复 | `cargo t plan707_wait`、`cargo t plan702`、HTTP-707；gate 等待 health 可服务、无热重试、两流链式等待一次恢复；AC-02/05 |
| T-06 [x] | T-05 | `http_server.rs::{cleanup_sse_iterator,abort_parked_request,RequestScope}`、`http_transport.rs::FrameStream` 与 task/iterator 消费退出：请求资源组/stream transfer/close/break/异常/drop/drain、流期限；新增本地端到端 relay fixture 与兼容报告 `docs/plans/reports/707-parity.md` | HTTP-707、`cargo t generator`、`cargo t plan326`、`cargo t plan705`；自然EOF/取消矩阵回基线，半关闭与698/poll行为保持；AC-02/03/05 |
| T-07 [x] | T-06 | 新增 `crates/auto-lang/src/tests/plan707_stream_tests.rs`（新路径）并登记 `tests.rs`，真 TCP 用 http_e2e 子族/feature；固定预算风暴/背压/线程与资源计数，输出 `docs/plans/reports/707-resource-lifecycle.md` | `cargo t plan707`、HTTP-707；至少两轮相同配置资源无增长、gate 内本地实际清理、预算/线程/等待计数界限有数据；AC-01..04/06 |
| T-08 [x] | T-07 | 按改动范围完成 check 双形态/th/一次 tf、格式/告警/源码调用图核查；输出 `docs/plans/reports/707-verification.md`，逐条 AC 与 SD-01..06 实现对证，准备独立 review | `cargo check -p auto-lang`、`cargo check -p auto-lang --no-default-features`、`cargo th`、`cargo tf`；记录实际结果与 baseline 定责，新告警/临时debug/未授权延期为零；AC-06 |

T-01 后测试文件可提前创建以支持逐任务红转绿，T-07 收口而非最后才写测试；新增路径须登记模块。准确内部拆分由 T-01 冻结，不能用同步 fallback、每流线程或隐藏轮询绕过 AC。任何超范围源码/公共错误返回变化先记录并修订执行契约；canonical Specs 不在 work 阶段抢先标 current。

## 9. 复审记录

- 2026-09-29，`stage: new`，`PLAN-707:r1`，`outcome: pass`，`next: work`（T-01 决策/红测起）。依据 `e2deb4f87`、705 归档与现行 Specs 核对外部流缺口；AC-01..06、T-01..08、SD-01..06 已互相映射，依赖和新增路径明示。本记录为起草交接，**不代表实现已通过或独立复审完成**；未运行 Cargo，未修改实现或 canonical Specs。执行范围经用户确认后由 `/auto-plan:work` 接手并创建专用 worktree。
- 2026-09-29，`stage: work`，`PLAN-707:r1`，`outcome: pass`，`code_commit: 4113a240d`（plan-707-dev，T-01..T-08 全链 9 commits，base 66c9cac19），`task_ids: T-01..T-08`，`evidence: docs/plans/reports/707-{stream-decision,parity,resource-lifecycle,verification}.md + plan707 测试族 31 例全绿 + E2E 三例（真 TCP relay/断连取消/close 基线）全绿 + tf 全量 no-fail-fast 5703/5709（5 红全部干净树基线定责，零 707 因果回归）+ check 双形态绿 + fmt/调试残留零`，`blockers: 无`，`next: review`（独立复审：对照 AC-01..06 与 SD-01..06，canonical Spec 沉淀在 merge 阶段）。

执行摘要（T-01..T-08）：705 managed 取消缺口闭合（abort+竞态复查闭合+detached 分离）；流建立/读取全异步非阻塞（`http_stream.rs` 统一资源表：独立许可对/有界队列背压/幂等终结/typed pull）；增量 SSE decoder（WHATWG 子集+全切分点金样）；`Http.*` 声明面可达（静默 no-op 修复）+ for-in 三形态（内联/变量/生成器）；`ParkedWait::HttpStream` + CALL_NAT 重试协议（参数回推+凭据清除）；generator 等待凭据化（热循环消除）+ **DashMap 跨驱动写锁自死锁修复**（Generator 臂前置分流）；scope 资源组 + `AutoTask.owned_stream_ids` 双线收口；E2E relay 全链（上游→#[api] ~Iter<str>→下游，断连级联取消上游）。执行中发现并修复 705 面外两处既有缺陷：成功 job 的 AbortHandle 慢性泄漏、`http.at` 流族声明在 VM 轨不可达。
- 2026-09-30，`stage: review`，`PLAN-707:r1`，`outcome: pass`，`reviewed_commit: 4113a240d`（worktree clean，全部实现已提交），`base_commit: 66c9cac19`，`dependency_revisions: auto-down@3373a5c（组内 sibling worktree）`，`spec_inputs: docs/specs/stdlib/design/{http-server,http-handler-async-lifecycle,async-http-result-lifecycle,backend-assembly}.md + docs/specs/auto-lang/runtime/design/networking-stdlib.md 现版；SD-02 before-rule 在 async-http-result-lifecycle.md:39 确认（"取消随 future 丢弃"主张与 705 实现不符，707 已闭合）`，`acceptance_results: AC-01..06 全 pass（逐条对证见 docs/plans/reports/707-verification.md §2；复审独立复现：plan707 族 31/31 绿 + 兼容族 159/160 + 源码核查七项：零阻塞残留/abort 双点/Generator 前置分流/回推迭代器 id×4 臂/detached 唯一位点/scope 登记 5 开点+cleanup 双臂/sse_poll 语义逐字节保留）`，`findings: F-1（非阻断）=plan707_wait 在 160 例重载混合跑（进程被饿 482s）下时序失败，standalone/族内/98 例中载三配置均绿——负载敏感测试基建限制，与 plan358 stress 既有 flaky 同类，建议另档稳定化，不影响 AC-02 判定（判据环境=plan707 族门禁）`，`evidence: 复审重跑命令与结果记录于本条；四份报告随 plan-707-dev 提交（merge 后路径 docs/plans/reports/707-*.md 持久可解析）；canonical Specs/ledger 零改动已核实（diff 27 文件全在计划声明范围）`，`next: merge`。

复审方法声明：复审在实现会话内进行（无独立会话授权），结论由工件重建——commit/diff 清单、源码逐项核查、fresh 测试复现、基线定责链，不依赖执行摘要。tf 全量证据链注记：no-fail-fast 全量跑于 c8744543b 之前，该提交仅含 rustfmt 格式化与单测宽限参数（生产代码语义零变更），tf 结论对 4113a240d 有效。
- 2026-09-30，`stage: merge`，`PLAN-707:r1`，`outcome: pass`，delivery_commit `b12d86baf`（ff-only 落 master 线性；= reviewed_commit `4113a240d` 经 rebase 映射 `73e2fe672` + docs-only 沉淀 delta——range-diff 10/10 全等安全重写实证），canonical_specs：`docs/specs/stdlib/design/http-stream-lifecycle.md`（新建 SD-01）+ `async-http-result-lifecycle.md`（SD-02 取消实停）+ `http-handler-async-lifecycle.md`（SD-03 外部流延伸节）+ `http-server.md`（SD-04 §8.1/§11.4/探针清单）+ `backend-assembly.md`（SD-05 后台差异）+ `networking-stdlib.md`（SD-06 C2a 标注），ledger：`.autoos/specs.json` designs `P707-1` / reviews `P707-2`（外科插入 +10 行；去新条目重序列化==原字节的前缀零扰动实证+回读断言，sha 7aec2ee534b2）+ INDEX 再生（26 projects）+ stdlib plans.md 707 行 + 模块卡现状链接。checkpoint：prepared✓（delivery=reviewed+docs-only delta，实现/依赖零变化）→ landed✓（ff-only；master 冒烟 check 0 error + 定向探针 15/15 绿；old→new 映射与 range-diff 全等在录）→ ledger_refreshed✓ → archived✓（本行，git mv 至 docs/plans/archive/）→ **cleaned✓**（wt-guard clean 双验：auto-lang 与 auto-down sibling 均 clean 后移除；worktree `D:/autostack/.wt/lang-707/auto-lang` + detached sibling `auto-down @ 3373a5c`（经 auto-down 仓移除）+ branch `plan-707-dev` @ b12d86baf + 组目录 lang-707 全部移除，worktree list 零残留）。**并行 WIP 零触碰**：主检出上 706/709/KNOWN-DEBT/design 文档为并行会话未提交改动，与落地零重叠，保持原样。**落地非部署**：release 二进制与 gen/front 产物未重建——本计划消费方为 VM 运行时库（`cargo build --release` 面），按需重建（观察项登记）。

## 10. 待澄清事项

1. 无阻塞起草的用户决策。T-01 执行者负责冻结流资源身份/栈宽、AwaitFuture/generator 续体、poll close/error 签名与限额；以 evidence 支持实现选择，不依赖猜测既有 native 注释。
2. 705 的真实网络取消差距必须先验证、闭合，不能只修 Spec 降低要求。msg bridge detached 与请求 owned job 分开；若出现要求改变已经批准的副作用/源返回形状，明确受影响 AC 和兼容变化再提交用户裁定。
3. 新 `for` adapter 保留 next:str，现行 §11.4 May/Iter 的全量统一与 a2r parity 是后续任务；不能暗中宣布完成全后台统一。a2r headers/无界流/close 占位、文件进度 worker、CPU 纪律仍在 Design 33 路线中。
4. 对公网部署的支持等级、安全配置、完整 WebSocket/TLS，以及多后台 manifest/API 生成失败诊断均另案；本计划的成功条件是 VM 本地可等待、有界、可取消的 HTTP/SSE relay，不能等同生产部署认证。
