---
plan_id: PLAN-699
status: archived
feature_name: vm-http-transport-axum-bridge
author: [agent]
created_at: 2026-09-24
updated_at: 2026-09-24
plan_revision: 1
supersedes_spec_components:
  - docs/specs/stdlib/design/http-server.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
new_spec_components: []
touched_goals: [GOAL-003]
affects: [crates/auto-lang/src/vm/ffi/http_server.rs, crates/auto-lang/src/lib.rs, crates/auto-lang/Cargo.toml, docs/specs/stdlib]
current_step: 8
total_steps: 8
---

# [PLAN-699] VM `#[api]` HTTP 传输替换与有界 VM 调用桥

## 0. 变更摘要

落实 [Design 33](../design/33-stdlib-runtime-and-http.md) 阶段 B：把 `auto run --server vm` 的 `#[api]` 默认 HTTP 入口从手写 HTTP/1 请求解析与响应拼接迁到 Axum/Hyper 承载的协议层；保留 `AutoVM: !Send` 的 owner 线程，通过有界、可取消的 owned 消息传递请求、普通响应和 SSE 帧。给连接/请求/队列设置可验证的资源边界，并补上服务级优雅关闭。`api.at` 的路由、参数绑定、返回值和错误语义以现行 Spec 与 PLAN-696 的真实对拍为准。

本计划处理 VM `#[api]` 服务入口。生成 Rust 服务已经用 Axum，但独立生成；`http.server().listen()`、`back_proxy.rs`、合并/IPC、客户端 `a2r-std` 暂不迁移。它们的现状与后续收敛路径要在 Spec 中明确，不能把本计划称为全后台统一服务器。

## 1. 目标

- G1：HTTP/1.1 请求分帧、连接复用和响应编码由 Axum/Hyper 负责；`#[api]` 默认入口不再执行手写 HTTP 解析器。
- G2：协议任务与 VM owner 之间只传 owned、`Send` 数据；有界排队、取消与 SSE 背压均可观测，VM 不跨线程转运。
- G3：服务有连接/请求/队列/body/header/时间预算及明确的超限/超时结果；支持停止接收、限时排空和取消 SSE。
- G4：015/017/023 的 `api.at` 现有语义、`Response`、中间件、CORS、限流、request ID、multipart 与 PLAN-698 的 publisher SSE 不回退；文档说清 VM、生成 Rust、Builder、back-proxy 的支持边界。

**非目标**：Auto `task`/`~T` 的通用异步 I/O 重做（Design 33 阶段 C）；生成 Rust server 的共享框架重构；HTTP/2/3、TLS、通用 WebSocket、公开互联网部署认证；Builder/back-proxy 传输层迁移；改变合并/IPC 的调用语义。保留现有监听地址默认值，安全默认值调整另立兼容性决策。

## 2. 架构方案

```text
客户端 ── Axum/Hyper HTTP/1.1 I/O ── 有界请求通道 ── VM owner LocalSet/Rc<AutoVM>
              │                         │                       │
              └── 普通响应 oneshot ◀─────┴── typed ApiReply ◀────┤
                  SSE body ◀── 有界帧通道 ◀── generator producer ─┘
                         断连/超时/关闭 → 取消并回收 iterator/task
```

Axum handler 要求 `Send` future，不能捕获 `Rc<AutoVM>`。当前 PLAN-696 让 VM 与 TCP listener 同处一个 `LocalSet`；迁移时优先让网络层在独立 Tokio runtime/线程上运行，VM owner 保持现有线程。先用编译/运行 spike 证明路由、启动/关闭、请求转发和取消可行，再固定桥接类型；如实测证明同 runtime 的安全适配更简单，允许采用等价的 `Send` 协议任务 + owner 消息边界，但必须通过 AC-02/03 的所有权和资源门禁。绝不以 `usize`/裸指针跨线程，亦不把同步 VM handler 塞进 `spawn_blocking` 后借此假装可并行。

协议层只构造请求元数据和有界 body；VM owner 复用现有 `match_route`、按名参数绑定、middleware/handler、`Response` 与 SSE generator 语义，输出结构化 status/headers/body 或流帧。不得把已完成的 HTTP 字符串再交给 Axum 解析，也不得在 Axum handler 等待时阻塞 runtime 线程。同步 CPU handler 的最大执行时长无法靠 Tokio 抢占，本计划须在报告中测量并明确此剩余边界；不把排队超时写成 VM 函数可中断的保证。

## 3. 技术栈

`crates/auto-lang` 的 Axum 0.8、Tokio、AutoVM `LocalSet`/`Rc`；`http_server.rs` 既有 API 绑定、middleware、SSE producer；原始 TCP 与真实 `examples/ui/{015-notes,017-chat,023-realworld}` E2E。`Cargo.toml` 中 Axum 当前是 `ui-interpreter` 牵入的 optional 依赖；T-01 必须确认纯 VM/无 UI feature 的编译与特性边界，避免新服务依赖隐式的 UI 默认 feature。生成 Rust server 的 Axum 版本/路由代码由 `auto-man/src/api_gen.rs` 单独维护，本计划不假定两者可直接链接共享。

## 4. 需求分析与背景调查

### 授权、依赖与来源

用户 2026-09-23 要求详查标准库 HTTP/task 与 AutoUI `api.at` 多服务形态并提出设计和改良计划；2026-09-24 明确询问 PLAN-696 完成后的下一份 HTTP 强化计划。本轮获授权在 master 起草计划与设计文档，不实施代码；无指定预算/自动续跑限制。实施须按 AGENTS.md 进入独立 `D:/autostack/.wt/lang-699/auto-lang` worktree，计划簿记留主检出。

- PLAN-696 已归档：分段 body、SSE 协作让出/断连回收与 `!Send` VM 所有权修复完成；`cargo th` 51/51。其 [owner 决策](reports/696-vm-owner-decision.md)、[对拍结果](reports/696-response-parity.md)、[验证报告](reports/696-verification.md) 为本计划基线。
- PLAN-698 当前 `execution_done`，第一次复审 `needs_fix`：VM `bus.subscribe` 转发线程断连回收 F-1 待修，且会修改同一 `http_server.rs` SSE 面与 HTTP Spec。**T-02 起的代码工作依赖 PLAN-698 修复、复审并折叠到 master**；T-01 可先做只读 spike/基线。折叠后重新核对 017 publisher 断连契约与测试，绝不从旧分支覆盖其改动。
- [全局总览](../specs/overview.md)、[stdlib 项目卡](../specs/stdlib/project.md)、[后台装配](../specs/stdlib/design/backend-assembly.md)、[HTTP Server Spec](../specs/stdlib/design/http-server.md) §4.1.1/§7.3/§8.1、[网络集成 Spec](../specs/auto-lang/runtime/design/networking-stdlib.md) 是现行知识源；[Design 33](../design/33-stdlib-runtime-and-http.md) 定义阶段 B 边界。

### 代码事实与差异

- `crates/auto-lang/src/lib.rs::execute_autovm_with_path` 读取 `AUTO_HTTP_PORT` 后调用 `http_server::serve_async(Rc::new(vm), &addr)`；`block_on_autovm_local` 建 `LocalSet`。
- `crates/auto-lang/src/vm/ffi/http_server.rs::serve_async` 是 Tokio TCP accept + `spawn_local`；`handle_connection_async` 同时做 HTTP 请求解析、CORS/限流、multipart、middleware、路由、VM 调用、响应字符串及 SSE。文件顶部仍称“VM 与 a2r 共用 Axum、spawn_blocking 直接调 VM”，与实际代码及安全约束冲突，须随实现纠正。
- 当前支持 `Content-Length`、最大 10 MiB body、读超时 10 秒；`Transfer-Encoding` 被拒绝，连接每次响应后关闭。迁移后 chunked 与 keep-alive 可能由 Hyper 自然支持，但要写入明确验收，不能在文档里凭框架名称推断已支持。
- `crates/auto-lang/src/vm/ffi/stdlib.rs::shim_http_server_listen` 和 `crates/auto-lang/src/back_proxy.rs` 是别的活服务路径；同名 `axum_adapter` 也不是 Axum 协议 server。迁移不得误改它们的公开行为。

## 5. 详细设计

### 请求/响应桥及资源边界

1. `ApiRequest` 仅含拥有所有权的 method/URI/允许的 header/body/peer/request ID 等数据；HTTP 协议对象和 socket 不进入 VM。`ApiReply` 为结构化 status/headers/普通 body 或 SSE 流。路由状态快照及 `__axum:` 合成路由调用仍由 VM owner 侧现有契约裁决。
2. request body 在协议层按可配置上限增量读取，不因 chunked/缺失 Content-Length 绕过上限。header 限额需要在 Hyper accept/connection 配置层证明可执行；若 Axum 高层入口做不到，T-01 先裁定 Hyper 服务驱动或受控连接入口，不能保留无界头。
3. 对在途请求/桥队列设置明确容量；满载时快速返回可测试的 503（`Retry-After` 若配置），不静默等待或无限分配。响应等待超时应解除排队/回传关系；若 VM handler 已开始同步执行，只标记取消并在返回后丢弃响应/释放资源，不声称抢占它。
4. SSE body 通过容量固定的帧通道送出，慢客户端阻止进一步产帧或触发明示的超时关闭；断连/关闭时取消 PLAN-696/698 的 generator、publisher 订阅资源。由 owner 线程负责 VM iterator/task 清理，不能在网络线程析构 VM 资源。
5. shutdown 接受可注入信号供测试，停止新连接和入队，给普通请求排空期限；SSE 主动通知取消并等待 owner 清理，期限后强制关闭网络任务。CLI 的 Ctrl+C/SIGTERM 接入需与现有 `auto run` 运行形态核对；Windows 使用 Ctrl+C 路径，SIGTERM 只在支持的平台验收。
6. 现有默认端口、地址和公开 `api.at` 签名保持；服务配置以宿主 env/内部结构承载，名称、缺省和无效配置行为在 T-01 决策报告中冻结，不能靠隐式无限默认。优先保证现有 `AUTO_HTTP_PORT`、`AUTO_CORS_ORIGIN`、`http.rate_limit` 可用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/stdlib/design/http-server.md` | §7.3/§8.1 的 VM 手写 TCP 与无关闭接口 → 已验证的 Axum/Hyper 协议、桥接、HTTP/1.1 支持/限制及关闭契约；目标态仍与实装分开 | 对外能力表准确 | AC-01..07 |
| SD-02 | modify | `docs/specs/stdlib/design/backend-assembly.md` | VM `#[api]` 使用手写 HTTP、Axum 只在生成 Rust 轨 → VM 与生成 Rust 均依赖 Axum 但为独立服务/handler 实现；Builder/back-proxy 状态单列 | 防混淆多后台装配 | AC-01, AC-06 |
| SD-03 | modify | `docs/specs/auto-lang/runtime/design/networking-stdlib.md` | VM HTTP server 不共享 Rust 协议层的说明 → 记录本计划实际协议层、Auto task 与 Tokio bridge 边界，不把 task 通用异步 I/O 写成已完成 | 保持 runtime 事实一致 | AC-02, AC-06 |

Spec delta 由 `/auto-plan:review` 验证后 `/auto-plan:merge` 沉淀。若 T-01 spike 必须改变公开语义或扩展到 Builder/back-proxy，先修订计划并呈报具体兼容性变化。

## 6. 测试设计

- **协议黑盒**：原始 TCP 客户端分段发送 headers/body、chunked body、同连接顺序两请求、非法 framing/重复长度、慢 headers/body、超过配置上限 body/header。按响应 status/body/连接行为判定；Hyper 自身拒绝报文的具体文案不作 parity 承诺。
- **桥与负载**：将队列/在途容量调到测试小值，保持一个慢 VM handler/慢 SSE，继续并发发健康请求及额外请求；检查 503 在时限内返回、队列有界、健康请求不被睡眠中的 SSE 拖住。同步 CPU handler 的阻塞影响和端到端延迟做可复现记录，不用单个偶然 p95 断言替代资源边界。
- **流与关闭**：真实 SSE 收帧、慢读/断连、publisher 订阅断连、shutdown 时普通请求排空与 SSE 退出；验证 iterator/task/订阅线程及服务线程回落、端口释放，无 VM 资源跨线程析构。
- **功能对拍**：重跑 PLAN-696 的 015/017/023 VM E2E 和 generated Axum live 对拍；覆盖 `Response` status/header、400 缺参/转换、404、CORS preflight、429、request ID、multipart、middleware、`__axum:` 路由。合并/IPC 只做调用语义抽样，避免把无 HTTP status 的路径误作网络 parity。
- **门禁**：`cargo check -p auto-lang`、定向 HTTP 测试、`cargo th`；本计划修改 VM/核心协议执行路径，末次 review 前 `cargo tv` 与必要的 `cargo tf` 按 AGENTS.md 门禁执行，记录已知独立红项。若不改 aavm 文件，不跑 `cargo taa`。文档起草阶段不跑 Cargo 或 docs_gen。

## 7. 验收标准

| ID | 可观察结果 | 验证方法与期望 |
|---|---|---|
| AC-01 | `auto run --server vm` 的 `#[api]` 入口由 Axum/Hyper 解析 HTTP/1.1；手写 `handle_connection_async` 不在默认调用图，正常请求可 keep-alive 与 chunked 传入 | 调用图审查；原始 TCP 同连接两请求/分块体 E2E 通过；纯 VM/无 UI feature 编译通过或明确 feature 契约 |
| AC-02 | Axum handler 不持有 `Rc<AutoVM>`/裸指针，桥只传 owned `Send` 类型；VM 始终由一个 owner 执行 | 类型编译、线程/Drop 探针、代码审查；无 `AutoVM` 地址整数洗白或 `spawn_blocking` 调 VM |
| AC-03 | 请求体/headers、连接/在途请求/队列与等候时间均有实际生效的上限，满载快速 503；非法/超限/超时有确定结果且内存不随输入无界增长 | 小限额实测与原始 TCP 慢读/超限/畸形输入；记录状态/关闭行为和资源峰值 |
| AC-04 | 慢 SSE 不妨碍健康请求；慢客户端、断连及关闭能停止 generator 和 PLAN-698 publisher 订阅并回收任务/线程 | 并发限时 E2E、断连/关闭资源计数与帧通道容量断言 |
| AC-05 | 外部关闭信号停止 accept，限时排空普通请求并取消 SSE，服务返回且端口可重绑 | 注入式关闭测试 + CLI Ctrl+C 实测；无挂起线程/iterator |
| AC-06 | 015/017/023、`Response`、参数绑定、middleware、CORS/限流/request ID/multipart、`__axum:` 路由无新增语义回退；多后台状态文档准确 | 现有与新增 E2E、generated Axum 实机对拍；SD-01..03 源码核验 |
| AC-07 | 计划范围门禁通过，独立复审核对全部 AC/债项和 Spec delta；已知红项单列 | `cargo check -p auto-lang`、`cargo th`、`cargo tv`/必要时 `cargo tf`、diff/format/告警检查与 review 记录 |

## 8. 执行步骤

| 任务 | 依赖 | 位置/动作与产出 | 验证（期望） |
|---|---|---|---|
| T-01 [x] | 无 | 只读核查 PLAN-698 折叠状态及 F-1 资源回收；在 `http_server.rs`/`lib.rs`/`Cargo.toml` 做 Axum `Send` bridge、header 限额、启动/关闭、feature 边界的有界 spike；新建 `docs/plans/reports/699-bridge-decision.md`，定默认限额/配置、线程拓扑、已知同步 handler 边界。（AC-01..05） | `[✅ 已完成]` worktree `5a77d1972`：698 折叠全链核查（F-1 b7ee6f302/归档 045aa37c7/cleaned b41e9aa31，worktree 已清）；spike 6/6（keep-alive/chunked/64KiB→431 原生/慢头 2s 关闭/SSE 帧流/graceful+端口复绑）；两坑实证（header_read_timeout 缺 timer 即 panic、try_recv 无 waker 饿死 body）；axum 转无条件依赖+hyper/hyper-util 新增；基线 `cargo th` 50/51（唯一红 back_proxy 031 在 master 同样红=预存环境红，范围外）；拓扑/限额/边界冻结于报告 |
| T-02 [x] | T-01、PLAN-698 复审/折叠 | 在 `crates/auto-lang/src/vm/ffi/http_server.rs`（可拆新增同目录内部模块）抽出 owned `ApiRequest`/`ApiReply` 与纯 VM 调用逻辑；把原解析/写字符串中的路由/绑定/middleware/响应语义保留。先保现有 E2E 绿。（AC-02/06） | `[✅ 已完成]` worktree `3420af7ed`（rebase 后 `01cd23192`）：`dispatch_api_request` 语义逐行镜像（顺序=旧内联路径），旧 TCP 路径改走同一核心；`cargo check` 0 error；`cargo th` 50/51=基线逐位一致（唯一红=预存 back_proxy/031） |
| T-03 [x] | T-02 | 用 Axum/Hyper 入口替换 `serve_async` 默认 TCP parser，`lib.rs` 接 VM owner 与网络 runtime；`Cargo.toml` 修正 Axum feature 依赖。header/body/frame 处理均由协议层承担。（AC-01/02） | `[✅ 已完成]` worktree `6de714eeb`（原 e798df2d7）：专属 `auto-http-net` 线程 + hyper-util auto Builder（http1_only+TokioTimer+64KiB+10s）+ GracefulShutdown；axum fallback → 有界 mpsc → owner 循环；`handle_connection_async`/write_api_reply 删除；chunked/keep-alive/分段 body E2E 过；`cargo check --no-default-features` 0 error |
| T-04 [x] | T-03 | 在桥和网络入口加连接/在途/队列/body/header/超时预算与拒绝映射，写小容量负载/慢头测试；保留现有 400/408/413/429 语义的可验证部分。（AC-03） | `[✅ 已完成]` 同上提交：413（to_bytes+LengthLimitError）/408（body 总期限）/431（hyper 原生，实测）/503+Retry-After（队列满，桥级单测无竞态）/503（回复等待 30s）；`e2e_plan699_oversized_headers_431`、`…slow_headers_timeout_close`（实测 10.6s）、`bridge_tests::full_queue_rejects_with_503` |
| T-05 [x] | T-03/04 | 将 PLAN-696 generator SSE 与已折叠的 PLAN-698 publisher 订阅接到有界网络帧流；断连/取消回送 owner 清理。不得引入另一个长期转发线程。（AC-04） | `[✅ 已完成]` 同上提交：696 producer 原样+capacity-1 帧通道；FrameStream（poll_recv 接 waker——try_recv 饿死为 spike 实证坑）+shutdown watch 收流；断连/关闭回收 iterator/task/698 订阅（owner 线程，无新转发线程）；`e2e_plan696_slow_sse_does_not_block_health`/`…sse_disconnect_cancels_generator` 新传输重跑绿 |
| T-06 [x] | T-04/05 | `serve_async` 增可注入关闭信号/服务句柄，接入 CLI Ctrl+C 与受支持平台 SIGTERM；停止接收、限时排空、取消流、释放端口。（AC-05） | `[✅ 已完成（实机 Ctrl+C 见待澄清#5）]` `serve_with` 注入 watch 信号；serve_async 缺省=ctrl_c(+unix SIGTERM)；`e2e_plan699_injected_shutdown_releases_port`（停 accept→限时排空→owner 退出→端口复绑）；headless harness 三种投递方式均无法使 tokio handler 接管（CTRL_BREAK 落默认处理器 0xC000013A），实机终端确认列入待澄清 |
| T-07 [x] | T-02..06 | 跑 015/017/023、middleware/Response/429/multipart/`__axum:` 完整回归；生成 Axum 和合并/IPC 抽样；清理默认调用图中不再需要的手写解析/错误注释，修正文档草案的“已共用 Axum”旧误述。（AC-01/06） | `[✅ 已完成]` `1b8bf6a3f`：`docs/plans/reports/699-parity.md` 矩阵——015/017/023 真实 app e2e/Response/绑定/multipart/CORS/429/request-id 全绿（新传输重跑）；手写解析/写串路径已删除、模块头 "spawn_blocking 直接调 VM" 旧误述已纠正；中间件（公共 .at 面缺失=预存缺口）与 `__axum:`（代码逐行保留、无仓内语料）矩阵如实注记；合并/IPC 抽样确认未触碰 |
| T-08 [x] | T-07 | 范围门禁、基准/资源记录、遗漏/延期扫描与独立复审准备；整理 SD-01..03 的最终证据，按 review/merge 沉淀而非执行期私改 canonical Specs。（AC-03..07） | `[✅ 已完成]` `docs/plans/reports/699-verification.md`：check 双形态 0 error / th 56/56 / tv 162/162 / tf 5704/5705（唯一红=plan358 负载抖动单跑双绿；同批 9 红 master 逐一复红=全预存，零本计划归因红）/ `git diff --check` 干净；执行期 rebase 至 master `4dc4d581a` 在案；SD-01..03 证据就绪待 review/merge 沉淀 |

## 9. 复审记录

- 2026-09-24，`stage: new`，`plan_id: PLAN-699`，`plan_revision: 1`，`outcome: pass`。PLAN-696/698 状态、现行 Specs、`serve_async`/`handle_connection_async` 与 Cargo feature 已核对；AC-01..07 均映射到 T-01..08，SD-01..03 均有验收项。`next: work`，先完成 T-01；T-02 起待 PLAN-698 F-1 修复、复审、折叠。此记录不是实施完成或独立复审通过。

- 2026-09-24，`stage: work`，`plan_id: PLAN-699`，`plan_revision: 1`，`outcome: pass`，`code_commit: 1b8bf6a3f`（worktree `D:/autostack/.wt/lang-699/auto-lang`，分支 `plan-699-dev`，rebase 至 master `4dc4d581a`；链=`d2de4d01b` T-01 spike+决策 / `01cd23192` T-02 桥+dispatch 核心 / `6de714eeb` T-03/04/05 传输+预算+SSE+关闭 / `d748e59be` 中间件 e2e 撤销 / `1b8bf6a3f` 报告）。`task_ids: T-01..T-08` 全清。`evidence`：决策报告 699-bridge-decision.md（spike 6/6+拓扑/限额冻结）；对拍矩阵 699-parity.md；验证报告 699-verification.md——check 双形态 0 error、th 56/56（6 个新协议探针：chunked/keep-alive/431/慢头关/注入关停/队列 503 单测）、tv 162/162、tf 5704/5705（唯一红=plan358 stress 单跑双绿=负载抖动；同批 9 红 master 逐一复红=全预存，零本计划归因红）、`--no-default-features` 纯 VM 编译过、`git diff --check` 净。执行期勘定四条：axum 转 unconditional（`ui-interpreter` 摘 dep:axum）+hyper/hyper-util 直依赖；并行会话落 `4dc4d581a` 后无冲突 rebase（顺带收口旧 back_proxy/031 红与基线 018 坏样）；hyper 响应头名小写规范化致三处旧断言改大小写不敏感（值不变）；TF 红册定责=detached 复现法（master 主检出热编译复跑）。`blockers`: 无（待澄清#5 为环境限制记录，非阻塞）。`next: review`（/auto-plan:review 独立复审 AC-01..07+SD-01..03；建议复审环境有交互终端时补一轮 CLI Ctrl+C 实机确认）。

- 2026-09-25，`stage: review`，`plan_id: PLAN-699`，`plan_revision: 1`，`outcome: pass`，`reviewed_commit: 2e0922710`，`base_commit: 4dc4d581a`，`dependency_revisions: auto-down 3373a5c (detach, 零改动)`。**独立性声明**：复审与实现同会话，结论由制品与可复现命令重建（非执行摘要采信）；复用 T-08 门禁数字的显式理由=1b8bf6a3f 与 d748e59be 之间仅 docs-only 差异（两份 reports/*.md），全部编译目标内容恒等；受审 HEAD 上另复跑 34/34 AC 探针子集与 F-1 修复后 6/6。`spec_inputs`: http-server.md §7.3/§8.1、backend-assembly.md `http` 行、networking-stdlib.md §细节（SHA 于 merge 时随 SD 沉淀冻结）。`acceptance_results`: AC-01 pass（调用图：`handle_connection_async` 已删除；chunked/keep-alive E2E；`--no-default-features` 0 error）；AC-02 pass（transport 侧零 `Rc<AutoVM>`/零 spawn_blocking；`as usize` 仅 WS 帧长解码）；AC-03 pass（431/慢头 10.6s 关/413/408/队列 503 在 HEAD 复绿）；AC-04 pass（慢 SSE 不碍健康请求+断连取消+帧通道容量 1+FrameStream 关闭臂）；AC-05 pass（注入信号 E2E 绿：停 accept→限时排空→owner 退出→端口复绑；**CLI Ctrl+C 实机在本 harness 被证明不可测**——仓外 20 行最小 tokio 探针（`#[tokio::main]`+`tokio::spawn(ctrl_c)` 生态标准形态）经 CTRL_BREAK 可靠投递通道同样落默认处理器 0xC000013A=本环境 tokio 控制台 handler 注册本身失效，三拓扑（LocalSet+spawn_local / LocalSet+tokio::spawn / 裸 multi-thread）全等复现，非本计划缺陷；人工终端复现步骤在 §10.5）；AC-06 pass（699-parity.md 矩阵逐面对证，SD-01..03 before 断言与现行 canonical 文本逐一吻合）；AC-07 pass（门禁表+红册定责）。`findings`: **F-1（trivial，已修）**=`http_server.rs:236` 限流注释仍引用已删除的 `handle_connection_async`（T-07 范围内残留）→ 修复于 `2e0922710`（纯注释，compile-neutral，探针 6/6 复绿）；**观察 O-1（非阻塞、范围外）**=`backend-assembly.md:22` "VM bus.subscribe 仍是 compile seam" 系 PLAN-698 折叠前的过时行（698 已翻案为真执行面），不属本计划 SD 范围，移交 merge 阶段裁定是否顺带沉淀。`evidence`: 本记录+三报告（随计划归档持久化）+`git diff master...2e0922710`。`next: merge`。

- 2026-09-25，`stage: merge`，`PLAN-699:r1`，`outcome: pass`，`completion_kind: delivered`。**收据（五 checkpoint）**——
  `prepared`：受审基线 2e0922710（rebase 前记法；见 landed 映射），SD-01..03 canonical 修订与账本三件套在工作树备妥为 docs-only 后代提交（`git diff 2e0922710..59c7a753a` 五文件纯 docs：http-server.md §7.3/§8.1、backend-assembly.md http 行/拓扑表/696·699 锁定行为节+O-1 reconcile、networking-stdlib.md 传输注记、stdlib/plans.md 699 行、specs.json P699-1/P699-2 语义 append 10 增 0 删；INDEX 再生零语义变化）。
  `landed`：plan-699-dev rebase 至 master bc92ba6c7（7 提交），`git range-diff 4dc4d581a..2e0922710 master..HEAD~1` **6/6 全等**（补丁等价证明）；旧→新映射 d2de4d01b→6625cf49d / 01cd23192→da5b95948 / 6de714eeb→e48d4e366 / d748e59be→64119b289 / 1b8bf6a3f→82a790996 / 2e0922710→3c462b6e5；`git merge --ff-only` 落库零合并提交，master tip=delivery commit `4324a5ce4`；主检出 smoke：`cargo check -p auto-lang` 0 error + 协议探针 3/3。
  `ledger_refreshed`：`.autoos/specs.json` P699-1（architecture，SD-01..03 契约）/P699-2（reviews，复审+合并收据）语义 append-only（indent=1 roundtrip 恒等）；`docs/specs/stdlib/plans.md` 699 行；`docs/specs/INDEX.md` 由 `scripts/spec-index.py` 再生（输出零变化）。canonical 沉淀目标=docs/specs/stdlib/design/{http-server,backend-assembly}.md + docs/specs/auto-lang/runtime/design/networking-stdlib.md。
  `archived`：`git mv` 至 `docs/plans/archive/699-vm-http-transport-axum-bridge.md` + `status: archived`。
  `cleaned`：随后补记（wt-guard 双 worktree + 分支/组目录清理）。


## 10. 待澄清事项

1. T-01 决策：Axum/Hyper 在本仓版本下的 header size/deadline 可控接入方式、请求 body 增量收集、关闭信号所有权、纯 VM feature 配置；若高层 API 不满足，选可编译的 Hyper 连接驱动，不降低 AC-03。（已决：hyper-util auto Builder 直驱，见 699-bridge-decision.md）
2. T-01 实测后冻结限额和默认值；现有 10 MiB/10 秒与监听地址保持兼容，新增连接/队列/关闭预算由可复现负载数据定，不臆设生产容量。（已冻结于决策报告 §2）
3. 同步 CPU handler 占住唯一 VM owner 时，排队与取消能限制外部资源，但无法强制抢占 VM 代码。若要求 handler 可中断/多 VM 并行，属于 Design 33 阶段 C 的 task 调度或后续架构修订，不以本计划未实现的能力作承诺。（保持）
4. `http.server().listen()`、back-proxy 与 VM `#[api]` 是否最终共用协议入口、以及默认 loopback/公开监听策略，留阶段 D 独立裁定；本计划必须在 Spec 精确区分它们。（保持，SD-02/SD-03 落点）
5. **CLI Ctrl+C 实机确认（非阻塞）**：本执行会话为 headless pty 控制台，三种事件投递方式（进程组 CTRL_C_EVENT / AttachConsole+GenerateConsoleCtrlEvent / CREATE_NEW_PROCESS_GROUP+CTRL_BREAK_EVENT）均无法使 tokio handler 接管（CTRL_BREAK 落默认处理器 0xC000013A，事件本身已到达=handler 注册在该环境未生效）。注入信号与 ctrl_c 汇入同一 watch，下游全路径已 E2E 证明（`e2e_plan699_injected_shutdown_releases_port`）；ctrl_c(+unix SIGTERM) 接线与仓内先例同构（`vm/task_system.rs:732`）。建议：复审或合入后由有交互终端的环境执行 `AUTO_HTTP_PORT=8080 auto examples/ui/015-notes/src/back/api.at` → Ctrl+C → 期望 stderr 出现 `Ctrl+C — shutting down gracefully` 与 `VM owner loop exited`、进程 exit 0、端口可复绑。
