---
plan_id: PLAN-724
status: archived
feature_name: a2r-http-client-async-convergence
author: [agent]
created_at: 2026-10-01
updated_at: 2026-10-02

plan_revision: 1
supersedes_spec_components:
  - docs/specs/a2r-std/project.md
  - docs/specs/stdlib/design/backend-assembly.md
  - docs/specs/stdlib/design/http-stream-lifecycle.md
  - docs/specs/stdlib/design/async-http-result-lifecycle.md
  - docs/specs/auto-lang/trans/overview.md
new_spec_components:
  - docs/specs/a2r-std/design/http-client-runtime.md
  - docs/specs/auto-lang/trans/design/http-client-lowering.md
touched_goals: [GOAL-003]

affects: [crates/a2r-std, crates/auto-lang/src/a2r_std.rs, crates/auto-lang/src/trans/rust.rs, crates/auto-lang/src/sse, crates/auto-lang/src/vm/ffi/stdlib.rs, stdlib/auto/http.at, docs/specs/a2r-std, docs/specs/stdlib, docs/specs/auto-lang/trans]
current_step: 8
total_steps: 8
---

# [PLAN-724] Rust/a2r HTTP 客户端异步、流背压与生命周期收敛

## 0. 变更摘要

接续 [Design 33](../design/33-stdlib-runtime-and-http.md) C2b。PLAN-707 已交付 VM 外部流的共享 runtime、等待通知、有界缓冲和真实取消；这些能力尚未进入 Rust/a2r 客户端。本计划让独立 `a2r_std::http` 与 `auto_lang::a2r_std::http` 共用一个 Rust 客户端内核，并使 Auto 异步上下文实际调用 async I/O。

交付范围为客户端及必要的 a2r 发射、解码器复用、验证修复。HTTP 服务路由、任意 `#[api] ~Stream` handler 的 Rust 生成、HTTP/IPC/进程内调用统一、文件上传下载、CPU 调度、装配 manifest 和部署能力仍需后续计划。本计划不把客户端交付作为完整 Web 服务器交付。

同时处理已登记的 P707-R1：默认 headers 真 TCP 测试直接提交 managed job，却未先登记 live-op，静态看会被 707 的取消竞态守卫中止。执行时须先复现、验证根因，修复测试调用协议并保留真实公开入口覆盖。

## 1. 目标

- G1：两条 Rust runtime 的客户端网络执行只有一个实现源；消除本计划支持集中的名称、元数、返回类型和 header 编码漂移。
- G2：普通请求和外部流在异步上下文中使用 reqwest async；排队、建立、读体、逐块等待和取消均可让出执行线程，兼容 Tokio current-thread 与 multi-thread 消费者。
- G3：请求、流、响应体、缓冲和解析 carry 有可测上限；close、Drop、任务取消和超时停止本地上游读取并归还许可。
- G4：响应状态、headers、body 和终结错误属于对应请求/流；内部状态不混入业务字符串，不依赖线程局部 last_status 关联并发请求。
- G5：用真实转译、编译、运行验证 VM/a2r/原生 Rust 的约定行为；保留已用同步/认证兼容面，并修复 P707-R1 验证阻塞点。

## 2. 架构方案

```text
Auto 公共客户端声明 + 历史认证 helper
        │ 有类型与异步上下文的 a2r lowering
        ├─ a2r_std::http 适配器
        └─ auto_lang::a2r_std::http 兼容适配器
                       │
        crates/a2r-std 的共享 Rust 客户端内核
        reqwest async / 有限调度 / typed 结果 / RAII 取消
                       │
          复用连接的 Client + 固定 Tokio 执行环境
```

auto-lang 已依赖独立 a2r-std（Plan 673）；内核放在后者即可供前者调用，无需新增依赖环。两模块的整个标准库不在本计划合并，HTTP 历史返回形状通过薄适配器保留。VM 的资源表、park/resume 和 runtime 保持自身协议；必要时提取无 VM 依赖的字节解码逻辑供两侧复用。

Rust 内核采用固定、可复用的执行环境与 Client；async 消费者通过 Future/通知等待，不能同步 recv/join。同步兼容入口通过同一内核桥接，只在允许阻塞的边界使用。T-01 验证运行时创建/退出、跨 runtime 消费、取消传播与同步桥接；不能每请求创建 runtime、线程或 blocking client。调用者取消等待必须传给内核 job，丢弃 oneshot receiver 本身不等于终止 detached worker。

## 3. 技术栈

- Rust；仓库已有 reqwest 0.12、Tokio、serde_json；按需启用独立 a2r-std 的 async/stream 特性。文件 helper 仍用 ureq，不为删除依赖扩展范围。
- Tokio 有界通道、有限许可、通知与任务取消；owned 数据跨线程，禁止携带 AutoVM/AutoTask。
- Auto→Rust 发射器 `crates/auto-lang/src/trans/rust.rs`；既有 a2r golden 与编译运行夹具。
- 回环 TCP stub（`127.0.0.1:0`）、确定性 gate、有限截止时间和资源计数；不依赖公网服务、API key 或新测试框架。

## 4. 需求分析与背景调查

### 4.1 授权、版本与依赖

- 用户确认「707 已经实施完毕」并要求「继续规划下一个计划」。本次授权为起草一个可评审计划与更新 Design 33 路线；尚未开始代码实施。范围为 auto-lang 仓，未收到跨仓修改、预算或自动连续实施授权。
- 调研起点主检出 `142458d21`；规划期间其他文档落地主检出至 `7bd910712`，取号提交 `fa65db664`。T-01 记录实际实施基线及相关文件 hash，核对期间落地差异。
- 707 归档：`docs/plans/archive/707-vm-http-stream-async-relay.md`，交付提交 `b12d86baf`；current Spec 为 `docs/specs/stdlib/design/http-stream-lifecycle.md`（2026-09-30）。新计划不重开 707，也不沿用实施前设计作为现状。
- 712 r3 HTTP 错误分层已落地，相关提交 `bb80e0d49`；权威规则在 `docs/specs/auto-lang/ui/overview.md` SD-10。标准库 result Spec 第 3 条仍写传输错误返回 status:0 JSON，需在 merge 修正文档消费边界。
- 扫描活跃计划与 archive，无同范围在途计划；排他取号 724，计数器现为 725。实施目录 `D:/autostack/.wt/lang-724/auto-lang`，分支 `plan-724-dev`。

### 4.2 证据与缺口

| 来源 | 已确认事实 | 对本计划的要求 |
|---|---|---|
| `docs/specs/overview.md`、`stdlib/project.md`、`a2r-std/project.md`、`auto-lang/trans/architecture.md` | VM native、手写 Rust runtime 和生成服务是不同路径；a2r 目标为三方行为对齐 | 冻结客户端支持矩阵，不宣称全部后台或服务器已共享 |
| `stdlib/auto/http.at`、`http.vm.at`、`http_stream.at` | 普通 post 两参返回 Response；认证 post_sync/post_bearer 三参返回 str；Response 有 header_get；流有 next/is_done/close 与自由函数入口 | 分派依据模块、类型、元数和源符号，不能将所有 post 映射到认证 tuple |
| `crates/a2r-std/src/http.rs` | send_async 包裹 spawn_blocking，内部再 spawn/join；header_get 返回空；流无界通道、close 占位；状态为字符串控制帧；读体无界 | async 内核、响应元数据、typed 流、实际取消与兼容适配 |
| `crates/auto-lang/src/a2r_std.rs::http` | 第二套客户端为 reqwest::blocking + spawn_blocking；认证 tuple 与独立 crate 类型不同；另有 root http_post | 同内核供两 facade 使用，认证/返回形状分别适配 |
| `crates/auto-lang/src/trans/rust.rs` | 多处 HTTP 特判；builder/stream 发射同步函数；独立 crate 缺少部分被发射符号；别名/限定名分支 await 形状不同 | 必须做 typed lowering 与实际产物编译 |
| `crates/auto-man/src/api_gen.rs` | 生成物把 a2r_std 限定为 auto_lang::a2r_std；流 handler 当前主要为事件总线模板 | 验证限定名 consumer；任意外部流 api generator 重写另案 |
| `docs/specs/stdlib/design/http-stream-lifecycle.md`、`crates/auto-lang/src/sse/decoder.rs` | VM 增量 UTF-8/SSE decoder 已交付，有明确子集规则和 legacy 差异 | 复用冻结子集，明确 EOF/空 data/[DONE]，不声称完整 EventSource |
| `vm/ffi/stdlib.rs` e4、`async_http.rs::submit_client_job`、债务 P707-R1 | 生产 shim 先 register_live_op；e4 的 999_001/999_101/999_102 未登记；提交后缺席令牌触发 abort | 静态根因推断，本次未运行测试；T-02 红转绿核实，禁止回退取消守卫或改 detached |
| `docs/specs/auto-lang/trans/design/test-convention.md` | golden 只验证 Rust 文本，不保证链接/await/类型/执行 | 同源编译运行覆盖两种 runtime 名称 |

### 4.3 边界

不新增 http.rs.at、不改装载器、不迁移文件/进度 worker；task/actor mailbox、CPU 抢占、API 路由生成与 UI 四生成器不作为顺带重构对象。若必须改变公共返回类型、扩大跨函数 async 传播规则或重做 API generator，须形成具体差异并修订计划，不能用同步 fallback 或删掉验收替代。

## 5. 详细设计

### 5.1 调用与兼容矩阵

T-01 输出 `docs/plans/reports/724-client-decision.md`，逐项列「源符号/类型/元数 → sync 或 async 上下文 → 发射目标 → 返回/失败语义 → 样本」。必须覆盖：

| 调用面 | 本计划承诺 |
|---|---|
| Auto get/post/put/delete、request(...).header/body/json/timeout/send | 两参 post 不走认证三参分支；Response 状态、大小写无关 header_get、body_bytes 可用；异步上下文等待 async job，同步上下文有明确桥接 |
| Auto post_sync/post_bearer/last_status；历史 Rust 三参认证 helper、root http_post | header 与返回形状按实际 VM/已用 Rust 调用登记；保留兼容符号。last_status 只作同步兼容信息，不承诺并发 task 关联；异步消费使用 owned 状态，不靠 TLS |
| Auto get_stream/post_stream/post_stream_with_headers、next/is_done/close、自由函数 next/is_done/close | async 建立/读取让出线程；typed 内核区分 Data/EOF/Error。源手工接口 EOF 按 VM 哨兵适配；历史直接 Rust next 的空串/Option 形状经兼容层保留 |
| Auto stream_iter 与 for；异步函数/显式异步块的流消费 | 同源样本实际编译运行，覆盖变量/别名/自由函数。局部 break 不冒充关闭；显式 close 与离开拥有者作用域须回收；不能用同步 Iterator 阻塞读取冒充 async |
| 原生 Rust typed 流消费与 SSE 解码 | typed 读取与终结错误；raw 的 __status__:200、__done__、[DONE] 不猜状态；SSE 用 VM live decoder 冻结子集，分包不改变事件序列 |
| VM JSON shim 失败行为 | 保留 712：网络失败可 catch，非 2xx 为错误形状值；修正文档与验证，不把历史 Rust tuple 全部改为异常 |

headers：Auto 源现有 JSON 编码与历史 Rust `Key: Value` 行编码经明确 adapter 进入 typed headers；格式错误终结性报错，不可按空 headers 成功发射。正常状态、非 2xx、传输失败分别冻结结果，不能掩盖接口的历史形状差异。

### 5.2 内核、资源与预算

建议新增 `crates/a2r-std/src/http/client.rs`，保留 http.rs facade；实现 owned Request/Response/ClientError、可取消 job、准入许可与 Client 复用。两 HTTP 模块调用同一内核，无网络执行副本、每请求线程、队满临时线程或 async spawn_blocking 兜底。同步桥接边界由 T-01 明确，不能在 current-thread runtime 嵌套 block_on 或静默阻塞；T-06 对已知 async 上下文选 async 路径，不支持上下文给源位置诊断。

默认预算对齐 705/707 交付值：非流式 active=8、queue=64、body=10 MiB、总期限=30s；流 active=16、queue=32、数据队列=16 项、raw 单块≤16 KiB、SSE 单事件与解析 carry 各≤256 KiB，建立=10s、上游读 idle=60s。消费者背压等待不计入上游 idle。排队有准入截止时间与取消；T-01 冻结排队期限、配置载体及非法配置处理。测试用独立小预算实例，不修改进程 env 制造并发污染。

容量不是全部内存预算：报告须列单流队列、事件/UTF-8 carry、解码临时集合、当前网络块、保留 headers 上限与并发乘积。不能把 16×256 KiB 队列说成总计 256 KiB；大量事件逐项有界生产，不在临时 Vec 累积整响应。只声称能证明的应用保留内存上限，注明 reqwest/Hyper/TLS 内部缓冲不在算式中。

完成/错误/取消只终结一次；许可随实际执行体退出归还。Future Drop、close、Stream Drop、取消消费者 task、runtime shutdown 均须传至 producer。首包未到、idle、队满时取消都能结束本地 I/O，不承诺撤销对端接受的 POST。EOF 排空已接收数据再结束；失败可查；close 幂等并可丢弃待消费数据；is_done 不能因 producer 写完导致尾部丢失。

### 5.3 解码与发射

- 提取 `sse/decoder.rs` 的纯 decoder/UTF-8 carry 到共享 Rust 层，或证明可等价复用；VM facade 做类型适配，legacy `sse/parser.rs` 保持语义。不重定义 707 SSE 子集；提取前后对拍 BOM、CRLF/CR/LF、UTF-8 拆分、多 data、空 data、id/retry、未闭合 EOF 与超限，单列标准差异。
- HTTP lowering 依模块/接收者类型，防止 send/next/post 名称碰撞影响集合与用户方法；正确维护 async 函数/块/流循环上下文，不给所有 next 加 await。
- T-01 用可编译样本确定 existing ~T/异步块/流 for 与 helper 边界。直接调用、变量接收者、一层 helper 均在必选集内；不要求全语言隐式 async 传染。同步声明阻塞 helper 从 async 路径被调用时，采用已支持范围内适配或明确诊断，不能隐藏阻塞。
- 历史 Rust 不同签名以适配/命名隔离，不直接 re-export 造成二参普通 post 与三参认证 post 冲突；必要时加内部桥接符号，源 .at 名称/元数/返回约定保持。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/a2r-std/design/http-client-runtime.md`（新） | 无统一协议 → 两 facade 内核、兼容矩阵、typed 状态、预算/取消/同步边界 | Rust 可执行契约 | AC-01/02/03/04 |
| SD-02 | modify | `docs/specs/a2r-std/project.md` | ureq/镜像描述 → 客户端支持集与共享内核；文件 helper 剩余差异登记 | 现状准确 | AC-01/05 |
| SD-03 | add + modify | `docs/specs/auto-lang/trans/design/http-client-lowering.md`（新）、`docs/specs/auto-lang/trans/overview.md` | 分散特判 → typed 调用/上下文/返回映射与编译运行支持表 | 产物可用 | AC-01/02/05 |
| SD-04 | modify | `docs/specs/stdlib/design/async-http-result-lifecycle.md` | Err 统一 status:0 JSON 旧表述 → 槽终结与不同消费接口分开；JSON 按 712 可 catch；managed 先登记 | 文档与 P707-R1 验证协议 | AC-06 |
| SD-05 | modify | `docs/specs/stdlib/design/backend-assembly.md` | Rust 流/实现差异未收敛 → 共享客户端与已验支持集；独立 server/文件/装配缺口仍明示 | 不夸大覆盖 | AC-01/05 |
| SD-06 | modify | `docs/specs/stdlib/design/http-stream-lifecycle.md` | a2r 未迁移 → Rust 已验范围/decoder/adapter/预算/取消；VM park 与 Rust await 分开 | 707 接续 | AC-03/04/05 |

review 验证后由 merge 沉淀；本次不改 canonical Specs，不宣称覆盖 http.at 全部 server 符号。

## 6. 测试设计

1. **协议/产物矩阵**：普通请求、builder、两参 post/三参认证、别名/限定名、变量、自由函数/for；200/500、网络失败、headers、二进制 body。同源 Auto 样本 VM/a2r 对拍，原生 Rust typed 交叉核验；legacy 哨兵差异单列。
2. **单 reactor 健康**：current-thread/multi-thread 下 gate 控制慢建立/慢 body/无帧流；gate 开前健康 job/定时通知完成。断言事件顺序，截止时间只防挂，不把短帧间隔当正确性。
3. **背压与解析**：暂停消费者，高水位不超预算、应用上游读取停止推进；恢复后顺序完整。超长 body/事件/未闭合行可观察 Budget 错误；UTF-8/BOM/CRLF 切关键边界，控制串保留。
4. **取消**：queued/active/opening/idle/队满五阶段，close/Drop/外层 Future/task 取消；active=1 且旧上游 gate 未开时，新 job 在旧 job 退出后能准入。观测 producer 停止、许可/任务/handle 回基线，不能只查无结果。覆盖退出、竞态、EOF 排空、双 close。
5. **响应归属**：并发不同 status/header，跨 yield/线程迁移仍读自身 metadata；TLS 不参与断言；header 大小写、非 2xx body、读体失败不冒充空成功。
6. **P707-R1**：保存 e4 原失败，修复直接驱动 register/consume/cancel 配对，公开 native/.at 入口 headers/query/body 到 wire；未登记 managed job 仍中止。保留 712 catch 与 HTTP 500 值回归。

布局：内核单测在新 `crates/a2r-std/src/http/client.rs`；新串行 TCP 集成 `crates/a2r-std/tests/http_client.rs`；新桥接/转译探针 `crates/auto-lang/src/tests/plan724_http_client_tests.rs`，按既有形式注册于 `crates/auto-lang/src/tests.rs`；golden 在 `crates/auto-lang/test/a2r/` 按 test-convention 选分区。复用编译运行夹具，对 standalone 与 qualified facade 分别链接；不能仅改 expected.rs 验收。auto-lang 的真 TCP/编译后网络执行用例加 `test-http-e2e` feature 守卫，函数名含 `http_e2e_plan724`，确保 `cargo th` 的 `http_e2e` 过滤器实际收集；纯逻辑/发射探针可留日常档。

开发：`cargo check -p a2r-std -p auto-lang` 与 scoped 测试。正式复审在 worktree 串行跑 `cargo test -p a2r-std`、裸 `cargo t`、`cargo tt`、`cargo th`；触 VM/编译器 Rust 源加 `cargo tv`；确需改 auto-man 加其 scoped check/test；仅实际改 ui_gen/** 才加 `cargo tu`。不触 aavm，不跑 taa。`cargo tf` 是 merge 到期后由 `/auto-plan:regress` 在主检出单实例执行的批量档，不是 per-plan 复审门禁。本次起草不运行 Cargo。

TCP 用 OS 临时端口、有限 accept/read/关闭截止时间与清理守卫；共享 runtime/计数测试隔离或串行。日常档不引入无限 accept、全画廊编译或每例冷启动 Cargo；编译运行集中复用内容缓存，命令/退出码入报告。

## 7. 验收标准

| ID | 可观察的通过条件 | 方法/证据 |
|---|---|---|
| AC-01 | 两 facade 共用网络内核；§5.1 必选映射完整；普通 post/认证兼容不冲突；header/body/status 正确 | decision 报告、源码引用、两 facade 编译运行与 wire 断言 |
| AC-02 | 必选 async 函数/块/流循环不阻塞 reactor，无 blocking client/spawn_blocking/同步 recv/join 或每请求 runtime/thread；两种 Tokio runtime 的健康事件先于 gate 放行 | typed golden + 实际编译运行 + gate 顺序；不支持调用有源位置诊断 |
| AC-03 | 准入/体/队列/事件/carry 守预算，超限/超时/队满可观察终结；EOF 排空；控制串不作内核状态 | 小预算实例、高水位、拆包/超限/EOF/控制串测试与 resource 报告 |
| AC-04 | close/Drop/Future/task 取消五阶段实际停止 producer，许可/登记回基线；幂等/竞态不复活、错误可查、metadata 不串扰 | active=1 gate 未开取消复用；资源/producer 退出与并发 metadata 断言 |
| AC-05 | 必选同源 VM/a2r/原生 Rust 约定语义一致；两种 runtime 名称实际编译运行；遗留同步认证行为保持；未承诺范围明确 | parity 报告列源/产物/命令/退出码与差异；golden 不能单独满足 |
| AC-06 | P707-R1 确认根因后红转绿且有公开入口 wire 覆盖；缺 live-op 仍中止；712 错误分层保持；触面门禁无新增红与 Spec delta 可核对 | e4/plan712、门禁日志、verification 报告；verified 后清偿债务，禁止 ignore/detached |

## 8. 执行步骤

### T-01：冻结接口、上下文与 runtime 方案

- 依赖：707 archived。按 AGENTS 创建唯一 worktree、记录实施 hash；禁止 junction/symlink。
- 触面：stdlib/auto/http*.at、两 HTTP Rust 模块、trans/rust.rs HTTP 分派/函数上下文、auto-man/api_gen.rs consumer、sse/decoder.rs 与 §4 Specs。
- 做有界原型：两 facade 链接、~T/异步块/一层 helper/流 for 的真实转译编译、current-thread 等待与固定 runtime 桥接退出/取消；输出新 `docs/plans/reports/724-client-decision.md`，冻结矩阵、同步边界、排队期限/配置、错误与解码子集。
- 验证：fixture cargo check/运行成功、无嵌套 runtime panic；不能以「先直接 Rust、a2r 后做」结束。若无法满足 AC-02/05，提交具体 needs_replan，不静默减支持集。覆盖 AC-01/02/05、SD-01/03。

### T-02：核实并修复 P707-R1 验证协议

- 依赖：T-01 基线，可先于内核实现。
- 触面：vm/ffi/stdlib.rs e4 默认 headers 测试/直接提交点，async_http.rs register_live_op/submit_client_job，tests/plan712_http_error_semantics_tests.rs。
- `cargo t default_headers_reach_wire_on_plain_get` 确认原失败；修复测试登记/清理、有限 TCP 等待与公开入口覆盖；检查相同模式。若是实际派发缺陷，给最小复现并修复，禁止削弱取消守卫。
- 验证：上述 scoped 测试与 `cargo t plan712_http_error_semantics` 通过；登记/未登记反例符合协议，附 baseline/patched 日志。仅证据成立后清偿 `docs/plans/KNOWN-DEBT-AND-RISKS.md` P707-R1。覆盖 AC-06、SD-04。

### T-03：实现共享非流式 async 内核与完整响应

- 依赖：T-01。
- 文件：crates/a2r-std/Cargo.toml、src/http.rs、新 src/http/client.rs，必要 src/lib.rs；Cargo.lock 仅实际依赖变化时更新。
- Client 复用、有限准入、owned metadata、增量 body cap、deadline、取消与已验证同步桥接；JSON/行 headers 在 adapter 入参解码；错误不得吞成空成功。
- 验证：`cargo check -p a2r-std`、`cargo test -p a2r-std http_client`；body/header/status/500/transport/cancel/admission 通过。覆盖 AC-01/02/03/04、SD-01。

### T-04：实现有界流、共享解码与回收

- 依赖：T-03。
- 文件：a2r-std/src/http.rs、src/http/client.rs；共享 decoder 新路径由 T-01 记录；auto-lang/src/sse/decoder.rs 复用 facade，必要 sse/types.rs 适配、vm/ffi/http_stream.rs 必要适配。
- typed Data/EOF/Error、metadata、raw UTF-8 carry/SSE 子集、逐项有界生产/背压；close/Drop/cancel 到 job；源接口与直接 Rust 兼容层，禁字符串控制帧/无界通道/提前 done 丢尾。
- 验证：`cargo test -p a2r-std http_client`、`cargo t plan707_client`、`cargo t plan707_decode`；五阶段取消/预算/分包/EOF 通过。覆盖 AC-02/03/04/05、SD-01/06。

### T-05：第二套 Rust facade 接入同一内核

- 依赖：T-03/04。
- 触面：auto-lang/src/a2r_std.rs::http/http_post、独立 crate http facade/必要 root 桥接；已用调用按 T-01 清单核对。
- 认证 header、旧 tuple/int 与源返回分别适配；补必选桥接符号；async 去掉 blocking reqwest/spawn_blocking；同步 TLS 与 owned async metadata 分开。
- 验证：`cargo check -p a2r-std -p auto-lang`、`cargo t plan724_http_client`；两 facade normal/auth/500/failure 比较通过，全仓扫描无重复网络执行。覆盖 AC-01/02/04/05、SD-02/05。

### T-06：有类型的 a2r HTTP async lowering

- 依赖：T-01、T-04/05。
- 文件：trans/rust.rs HTTP 分派、接收者类型、函数/块 async 上下文和流循环；必要 tests/a2r_tests.rs、src/tests.rs；新 plan724 测试及 test/a2r fixtures。
- 普通/认证 post 区分；builder/send、流建立/next/for 生成可等待调用；变量/别名/自由函数/一层 helper 覆盖；不全局重写用户 next/send；不支持边界明确诊断。
- 验证：`cargo t plan724_http_client` 与 scoped a2r golden；必选 fixture 实际编译运行及 current-thread 慢上游健康顺序；非 HTTP 碰撞方法无漂移。覆盖 AC-01/02/05、SD-03。

### T-07：三方对拍与资源实验

- 依赖：T-02 至 T-06。
- 文件：新 crates/a2r-std/tests/http_client.rs、plan724 探针，复用编译运行基建；新 `docs/plans/reports/724-client-parity.md`、`724-resource-lifecycle.md`。
- §6 全矩阵含 headers/auth/error/UTF-8/SSE/break+close/EOF、两 facade、取消风暴/慢消费者/两 Tokio runtime。fixture 代表生成 db 限定名链接，不宣称任意 api SSE 生成已落地。
- 验证：`cargo test -p a2r-std --test http_client -- --test-threads=1`、`cargo t plan724_http_client`、`cargo nextest run -p auto-lang --lib --features test-http-e2e --test-threads=1 http_e2e_plan724`；同源编译命令/退出码、预算算式、高水位和基线入报告；断言 gate 顺序和状态，不靠短 sleep。覆盖全部 AC、SD-01/03/05/06。

### T-08：触面门禁、独立复审与 Spec 提案

- 依赖：T-01 至 T-07；所有代码/测试在 worktree。
- §6 正式门禁、实际 diff 格式/新增警告/调试输出检查；确定性红不得笼统归为环境原因。
- 新 `docs/plans/reports/724-verification.md` 逐项验 AC/SD，扫描遗漏/未批准延后/workaround；按 /auto-plan:review 独立复审并绑定 revision。准备 Spec delta，merge 才沉淀项目卡/设计/ledger、归档与到期批量回归。
- 验证：每项 AC/SD 有实际证据，无未批准缺项；P707-R1 处置可核对。覆盖全部 AC/SD。

## 9. 复审记录

### 起草交接（2026-10-01）

- stage: new
- plan: PLAN-724
- revision: 1
- outcome: pass
- next: work
- changed_tasks: T-01..T-08（新）
- changed_acceptance: AC-01..AC-06（新）
- 静态核对 707 归档/current Specs、712 失败语义、两 Rust runtime、HTTP 发射/consumer；任务覆盖所有 AC/SD。pass 表示文档可交给 work，不是实现复审通过。
- 本次仅起草计划、更新 Design 33/索引；文档编号结构、T/AC/SD 覆盖、已有引用路径与本次新增/修改链接、`git diff --check` 均已核对。未运行 Cargo、未创建实施 worktree、未改 canonical Specs。

### work 交接（2026-10-02，T-01..T-08 全部执行完毕）

- stage: work
- plan_id: PLAN-724
- plan_revision: 1
- outcome: pass
- code_commit: plan-724-dev `6fc4a6023`（实施基线 `986e765ac`；提交链 7e7c6b3f8 → 14e9cc389 → 93b012b08 → 53836eca1 → 96a9ed707 → 6fc4a6023）
- worktree: `D:/autostack/.wt/lang-724/auto-lang`（分支 `plan-724-dev`；组内 sibling `D:/autostack/.wt/lang-724/auto-down` 只读 detached @895f8d0f9，供跨仓 path 依赖解析，未修改）
- task_ids: T-01..T-08 全部完成（current_step 8/8）
- evidence:
  - T-02：P707-R1 红（30.0s 确定性超时复现）→绿（0.46s）——真因=测试直驱未 `register_live_op` 被 707 取消竞态守卫中止；修复走生产协议（register→submit→wire→消费配对+未登记反例钉）；取消守卫零改动（`plan707_cancel` 5/5、plan712 2/2 复证）；`KNOWN-DEBT-AND-RISKS.md` P707-R1 已销号。
  - T-03/T-04：`crates/a2r-std/src/http/client.rs` 共享内核（固定 runtime+复用 Client+有界准入+总期限+增量体预算+owned typed Response+真取消传播；流=有界队列/背压/建立与 idle 期限/UTF-8 carry/raw 16KiB/SSE 子集/单次终结/close+Drop 真取消）+ `crates/a2r-std/src/sse.rs`（707 decoder 原样提取，auto-lang 侧 facade 复用，plan707_decode 9/9 零漂移）；内核单测 36 例 + `KernelInstance` 独立小预算测试实例。
  - T-05：`auto_lang::a2r_std::http` 接同一内核（reqwest::blocking/spawn_blocking/每请求线程全部退役；tuple 形状与 TLS last_status 保留）；全仓扫描零重复网络执行。
  - T-06：trans/rust.rs typed 上下文感知 lowering（in_async_ctx 跟踪 ~T/Future/main-await/generator/.go；http.get/post/put/delete 两参族、post 元数分派、builder 链+重绑定 send/send_async、流生产者 sync/async 分型、流方法与自由函数分型、for-in ""哨兵循环 break 不冒充关闭）；golden 2 件冻结（30_plan724）+发射探针 3 例。
  - T-07/T-08：`crates/a2r-std/tests/http_client.rs` 串行原生矩阵 7 例；`http_e2e_plan724_{sync,async,qualified_facade_kernel}` 编译运行腿（cargo th 收集）3/3 绿；报告四件（client-decision/client-parity/resource-lifecycle/verification）。
  - 门禁：a2r-std 43/43、tv 162/162、tt 1831/1834（3 红=p053 族主检出同红复现，712 §10② 在案）、裸 t 红集与基线逐名一致（差异仅 e4 修复）、th 与基线同红（back_proxy/relay 帧时序在案）+ plan724 3 新绿；触面文件零新增警告/调试输出。
- blockers: 无
- next: review（`/auto-plan:review`；Spec delta SD-01..SD-06 待验证后由 merge 沉淀；复审通过前 worktree 保留）

### review 裁定（2026-10-02）

- stage: review
- plan_id: PLAN-724
- plan_revision: 1
- outcome: pass
- reviewed_commit: `6fc4a6023`（worktree `D:/autostack/.wt/lang-724/auto-lang`，分支 plan-724-dev，复审时零脏文件）
- base_commit: `986e765ac`（diff 基；master 侧 96c876ce 仅 plan 簿记）
- dependency_revisions: auto-down sibling @`895f8d0f9355`（detached 只读，供跨仓 path 解析，零修改）
- spec_inputs: 决策报告 `docs/plans/reports/724-client-decision.md`、对拍 `724-client-parity.md`、资源 `724-resource-lifecycle.md`、验证 `724-verification.md`（均随 reviewed_commit 入库）；canonical `docs/specs/**` 全 diff 零触碰（0 文件）
- 独立性声明：复审与实施同会话完成，裁定从工件重建（代码级核查 + 命令复跑），未采信实施期摘要。
- acceptance_results: AC-01..AC-06 全部 **pass**（核查+复跑：两 facade 零 blocking/spawn_blocking/每请求线程（grep 实证仅文档注释）；async_http.rs 取消守卫对基线 diff=0 行；e4 register→consume→cancel 配对与未登记反例在码（stdlib.rs:10413-10511）；内核 status 先行/raw min 切分/try_lock async 安全在码；复跑 a2r-std 43/43、plan724 5/5、http_e2e_plan724 3/3、e4 0.4s 绿、plan712 2/2、tt 1832/1835（同 3 红=p053 族，主检出基线同红复现在案）；SD 目标路径 2 add 正确缺席/5 modify 目标存在）
- findings: 无阻断发现。非阻断注记：①同会话复审局限（上述声明）；②p053/plan606 等 tt/t 日档红为 712 §10② 在案基线族，非本计划回归（主检出同红复现在案）。
- evidence: 本计划 §8 各任务证据行 + `docs/plans/reports/724-verification.md` §1-§4；复审复跑命令与结果同 verification 报告档位，代码级核查点（async_http 零 diff、facade 零 blocking、e4 协议行号、内核修复点行号）见本记录
- next: merge（`/auto-plan:merge`；Spec delta SD-01..SD-06 随 merge 沉淀）

### merge 收据（2026-10-02，PLAN-724:r1）

- stage: merge
- plan_id: PLAN-724
- plan_revision: 1
- outcome: pass（archived；cleaned 见下）
- prepared: reviewed_commit 6fc4a6023（rebase 前坐标）→ 纯文档后代 2ded9a1f5（SD-01..06 canonical 编辑，6fc4a6023..2ded9a1f5 零代码文件 diff 实证）→ **delivery_commit 候选成立**
- landed: rebase master（range-diff 986e765ac..2ded9a1f5 vs master..plan-724-dev **7/7 全等**）→ ff-only 合入 → master tip = **8a7fa3cee**（= delivery_commit）。旧→新映射：7e7c6b3f8→2b3736ccf、14e9cc389→d8b90e2ce、93b012b08→bd4f84b4e、53836eca1→92fadc8bf、96a9ed707→1e6155067、6fc4a6023→5cc8a936a、2ded9a1f5→8a7fa3cee。主检出 smoke：plan724 5/5、e4 绿、a2r-std 43/43。
- ledger_refreshed: specs.json 外科插入（store 写者 8080 不可达，循 711/713..720 先例）——designs P724-1..7（七投影，docsha:d059f123acac4347/23a29f0f534c934b/72640ccca1d690d5/f27c3442a1798746/de34e79043b00234/d3b46a525eccf9da/9ea9a622fa76d69c @8a7fa3cee）+ reviews P724-8 + reports P724-9；designs 126→133/reviews 189→190/reports 112→113；json 回读+四段字节零扰动+条目唯一性守卫过；spec-lint 0 错误（6 预存警告）；spec-index 再生。提交 479ff6be1。
- archived: docs/plans/archive/724-a2r-http-client-async-convergence.md，status: archived
- cleaned: （待清理后回填）
- tail: 产物/批量回归检查见下（待执行回填）

## 10. 待澄清事项

- 没有阻碍起草的用户输入缺项。T-01 负责 runtime/sync 桥接、helper async 边界、兼容映射的有界验证；不得自行改源 API/降低 AC，超范围语义变化须具体修订。
- SSE 子集与完整 EventSource 差异由 T-01 列清；自动重连/Last-Event-ID replay/通用 WebSocket、TLS 服务端与 HTTP/2/3 不在本计划。
- 完成后文件传输、API 多形态契约、装配 manifest、CPU 纪律、部署配置仍需分期；数量以各期调研后的执行合同为准。
