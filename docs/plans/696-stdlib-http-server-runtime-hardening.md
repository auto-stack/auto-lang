---
plan_id: PLAN-696
status: reviewed
feature_name: stdlib-http-server-runtime-hardening
author: [agent]
created_at: 2026-09-23
updated_at: 2026-09-23
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-lang/runtime/design/networking-stdlib.md
  - docs/specs/stdlib/design/http-server.md
new_spec_components:
  - docs/specs/stdlib/design/backend-assembly.md
touched_goals: [GOAL-003]
affects: [docs/specs/stdlib, docs/specs/auto-lang/runtime, crates/auto-lang/vm, examples/ui]
current_step: 8
total_steps: 8
---

# [PLAN-696] 标准库后台契约与 VM HTTP P0 加固

## 0. 变更摘要

以 [Design 33](../design/33-stdlib-runtime-and-http.md) 的静态审计为输入，先修当前 VM `#[api]` 服务的三项 P0：普通 body 分段读错、慢 SSE 阻塞单线程 reactor、`!Send` VM 引用跨线程整数转运。建立标准库“公共 `.at` + 目标实现 + 宿主 shim/转译产物”的覆盖表，并把 `api.at` 的 HTTP/IPC/合并行为纳入验收矩阵。后续 Axum 传输替换、task 全链取消/背压、部署支持等级按 Design 33 阶段 B–D 另立 Plan。

## 1. 目标

- G1：VM HTTP 对任意合法 TCP 分段的普通 JSON 请求按 `Content-Length` 完整逐字节读取；畸形/超限请求明确拒绝。
- G2：慢 SSE producer 不能占住唯一 Tokio reactor；断连后及时回收取帧工作。
- G3：VM 服务的所有权与生命周期可由类型/线程边界证明，不再把 `&AutoVM` 转为 `usize` 交给其他线程。
- G4：标准库后台装配和 `api.at` 消费路径有真实、可追溯的现状文档与最小 parity 回归。

**非目标**：HTTP/2/3、TLS、公网生产部署、完整 WebSocket、Builder 与 `#[api]` 完全等价；不把现有 VM 手写 HTTP 入口宣布为长期架构。

## 2. 架构方案

短期以现有 `serve_async` 修 P0：请求头按 byte 读至 `\r\n\r\n`，再按 Content-Length 读满普通 body（multipart 共用上限/超时入口）；SSE 取帧结果通过可 await 的有界通道返回，连接断开触发取消；VM 对象在一个 owner 线程上创建和执行，连接任务持有同线程 `Rc`/等价非跨线程句柄。若现有调用栈无法直接容纳 owner 形态，T-04 先做可编译 spike 并记录决策，不能以另一处整数指针转运规避。长期 Axum/Hyper 协议层 + VM owner 桥见 Design 33。

## 3. 技术栈

Auto `.at` 公共接口；AutoVM / Rust native shim；Tokio current-thread `LocalSet`；原始 TCP E2E；`auto-man` 生成 Axum 对拍臂。生产代码只在 Plan 696 独立 worktree 修改。

## 4. 需求分析与背景调查

### 授权与约束

用户在 2026-09-23 授权调研 `stdlib/`，重点检查 HTTP/task、AutoUI `api.at` 的 HTTP/IPC/进程内路径，并产出设计与改良计划。本轮 `/auto-plan:new` 仅创建设计与执行合同；仓库规范规定执行前展示计划，后续代码须在 `D:/autostack/.wt/lang-696/auto-lang` 工作树进行。master 已有 `blueprints/**`、`back_prefix.rs` 的其他未提交改动，须保留。无用户指定预算或自动续跑限制。

### 代码与 Spec 证据（2026-09-23 静态快照）

- `stdlib/auto/{http,async,io}.at` 与 `.vm.at`，`io.c.at`：公共声明和目标实现；`http.rs.at`、`async.rs.at`、`io.rs.at` 当前不存在。Vue `use.web` 另走 UI 适配。
- `crates/auto-lang/src/compile.rs` 的 `.at` 后接 `.vm.at`；`autovm_persistent.rs` 有独立装载路径；`parser.rs::get_file_extensions` 标注未使用。`docs/specs/auto-lang/runtime/design/networking-stdlib.md` 关于 `http.rs.at` 和依赖图需纠正。
- `crates/auto-lang/src/lib.rs` 与 `vm/ffi/http_server.rs::{serve_async,handle_connection_async}`：LocalSet + Tokio TCP 手写解析；普通 body 只从首次 `buf` 的 `raw.lines()` 提取；SSE 用同步 `recv`/`join`；`!Send` VM 指针经 `usize` 传入线程/连接任务。
- `crates/auto-man/src/api_gen.rs` 从 `api.at` 生成 Axum，转译失败可 fallback 模板。`crates/a2r-std/src/http.rs` 是 ureq 客户端，`task.rs` 是 Tokio actor mailbox，均非 VM 服务端共享框架。
- `crates/auto/src/main.rs` 裁决 `--server vm|rust`、`--no-merge`、`pac.at api`；`crates/auto-lang/src/back_proxy.rs` 是额外 HTTP / 进程内 VM session 适配。015、017、020、023、027、031 示例有 `src/back/api.at`。
- `docs/specs/stdlib/design/http-server.md` §4.1.1 已明确 VM 参数绑定优先级；`docs/specs/stdlib/design/async-http-result-lifecycle.md` 已记录客户端近期修复，实施不得覆盖现有约定。

## 5. 详细设计

### 保留的公开语义

路径/body/query 按名绑定优先级（path > body > query）、缺参/转换失败 400、单参 whole-body 容忍、尾随元数据、`Response` 状态/headers、SSE 帧以现有 Spec/测试为基线。普通 JSON 不按行重组，不要求 body 与 headers 同一 TCP 包。header/body 上限适用于读满全过程。

### 后台覆盖表

为 `io/net/async/http/json/sse` 至少列出：公共 `.at` 符号、`.vm/.rs/.c` 文件、VM native、a2r-std/生成 Rust、Vue 适配、消费示例、缺口/错误策略。不能因历史设计稿存在就填“已实现”。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/stdlib/design/backend-assembly.md` | 无正式目标装配说明 → 记 `.at` 基层、VM/C/Rust 实装路径、web 适配独立维度、a2r-std 镜像与覆盖表 | 防把草案当保证 | AC-01, AC-06 |
| SD-02 | modify | `docs/specs/auto-lang/runtime/design/networking-stdlib.md` | `http.rs.at`/`http→net→async` 被写成已落地 → 区分声明、VM shim、生成 Axum 与目标依赖图 | 纠正事实 | AC-01, AC-06 |
| SD-03 | modify | `docs/specs/stdlib/design/http-server.md` | “VM/a2r 都封装 Axum”写作现状 → 列 VM TCP、a2r Axum、合并/IPC 和修复后支持范围 | 能力声明准确 | AC-05, AC-06 |

Spec 增量经 `/auto-plan:review` 复核后由 `/auto-plan:merge` 沉淀；不把未来 Axum 迁移记成 PLAN-696 已完成。

## 6. 测试设计

用原始 TCP 客户端显式控制 headers/body 分包与半关闭；慢 SSE 用可控等待的 producer，同时发起健康请求，并观察断连后 generator/VM task/OS 线程回落。对 015 CRUD、017 SSE、023 认证和一个合并调用样本做正常/错误响应对拍；Rust Axum/VM HTTP 比较 JSON/status/headers/SSE，IPC/合并只比较调用语义。记录命令、commit、平台和端口。改动 Rust/VM 后按仓库门禁运行 `cargo check -p auto-lang`、定向测试、`cargo th` 和 `cargo tv`；若改 VM 核心调度，最终一次 `cargo tf`。不触及 aavm 路径，不跑 `cargo taa`。本次文档起草为 Category A，不运行 Cargo 或 docs_gen。

## 7. 验收标准

| ID | 可观察结果 | 验证方法 |
|---|---|---|
| AC-01 | `io/net/async/http/json/sse` 覆盖表均能追溯当前文件/符号，显式标缺失 | 文件/符号抽样，零虚构 `http.rs.at` |
| AC-02 | headers/body 分开、body 分 2+ 段、body 含换行与一次写入等价；短体/超限有界 4xx | 原始 TCP E2E 红转绿，核对 `Content-Length` |
| AC-03 | 等待中的 SSE 不阻塞第二个健康请求；首连接断开后取帧工作停止 | 并发限时 E2E + 任务/线程回落 |
| AC-04 | VM HTTP 无 `&AutoVM`↔`usize` 跨线程转运，生命周期由持有关系约束 | 代码审查、编译约束、关闭测试 |
| AC-05 | 015/017/023 与合并调用样本正常/错误响应保持；无新静默 fallback | 跨形态金样/集成测试，记录既有差异 |
| AC-06 | Specs 准确表述多后台与 VM/Rust/IPC/合并现状 | review 对照源码与 SD-01..03；merge 时 `spec-index.py` 通过 |
| AC-07 | 范围内编译、HTTP/VM 回归门禁通过或既有红项有独立复现记录 | `cargo check -p auto-lang`、`cargo th`、`cargo tv`；必要时一次 `cargo tf` |

## 8. 执行步骤

| 任务 | 依赖 | 位置/动作 | 产出与验收 | 验证 |
|---|---|---|---|---|
| T-01 [x] | 无 | 盘点 `stdlib/auto/`、`crates/a2r-std/src/`、`crates/auto-lang/src/{compile.rs,autovm_persistent.rs,vm/ffi}`、`crates/auto-man/src/api_gen.rs`；新增 `docs/plans/reports/696-backend-inventory.md` | AC-01；2026-09-23：覆盖表已对照源码路径/符号，明确 `sse.at::parse_sse` 存在、`http.rs.at` 缺失及 SSE 服务路径为独立拼接。报告：`docs/plans/reports/696-backend-inventory.md`；验证：路径存在/缺失断言 + 符号抽样通过，基线 `051e7b5` | 文件/符号抽样 |
| T-02 [x] | T-01 | `vm/ffi/http_server.rs` HTTP E2E 区新增原始 TCP 分段/短体/超限、慢 SSE + 并发请求红样 | AC-02/03 红样已记录；`cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(/e2e_plan696/)' --no-fail-fast`：4 项中超限 413 通过，分段 body（服务端 400 后连接重置，未读到响应）、短体（错误地返回 200）、慢 SSE（健康请求等待 2.50s）失败，均复现预期基线；测试提交 `724096cd4`。全量 `cargo th` 在既有 `back_proxy_tests::http_e2e_back_proxy_real_031_native_ns_session`（`names` 为空）失败并取消后续用例 | 定向 `cargo th`，记录结果 |
| T-03 [x] | T-02 | `http_server.rs::handle_connection_async` 统一 header/body byte 读取、长度/上限/超时错误，保留 multipart 行为 | AC-02；已按 byte buffer 读完整 header/body，校验非法/重复 `Content-Length` 和不支持的 `Transfer-Encoding`，10 MiB 上限、10 秒总读取期限，短体 400/超时 408/超限 413；multipart 复用完整 body bytes。PLAN-696 原始 TCP 的 body 分段、短体、超限、畸形长度 4/4 通过；既有 multipart、body-by-name、whole-body 3/3 通过；仍红项仅慢 SSE（T-06）。实现提交 `065f39663` | 定向 E2E + 既有 multipart/绑定测试 |
| T-04 [x] | T-02 | `lib.rs` VM 服务入口与 `http_server.rs` 做 VM owner 可编译 spike；新增 `docs/plans/reports/696-vm-owner-decision.md` | AC-04；同线程 Tokio `LocalSet` 承载 VM 与服务，选 `Rc<AutoVM>` 作为非跨线程所有权句柄；关闭/lifetime 关系及现阶段限制已记入决策报告。`cargo check -p auto-lang` 通过；owner 小样分段 body + generator SSE 2/2 通过。提交 `7d431992e` | `cargo check -p auto-lang` + owner 小样 |
| T-05 [x] | T-04 | 按 T-04 决策改 VM 生命周期/连接任务持有；移除跨线程 `usize` 地址恢复 | AC-04；服务 future 与每个 `spawn_local` 连接任务都持有 `Rc<AutoVM>`；HTTP 服务入口及 SSE 帧拉取路径不再将 VM 指针转成整数/跨 OS 线程；通过 owner HTTP E2E。SSE 写失败后停止取帧由 T-06 覆盖 | `cargo check -p auto-lang` + HTTP E2E |
| T-06 [x] | T-03,T-05 | `http_server.rs` SSE 取帧改可 await 的有界结果桥；断连/结束信号回收 | AC-03；generator 逐次 4096 指令预算、SSE 专用 `Time.sleep_ms` wake deadline、有界帧通道、心跳探测断连并清理 VM iterator/task；慢 producer 的健康路由 <1.5s，2.5s 延迟保持，断连后副作用不执行。原始 TCP/SSE 定向 5/5 通过；提交 `86df0015f` | 原始 TCP/SSE E2E：5/5 |
| T-07 [x] | T-03,T-06 | 用 `examples/ui/{015-notes,017-chat,023-realworld}/src/back/api.at` 和合并调用测试建立响应 parity，记录预存差异 | AC-05；VM 对拍 015/017/023 3/3，生成器 metadata/inline return/SSE discriminator 4/4，Axum 015/017/023 live HTTP assertions 全通过；VM 017 pubsub 仍受 `auto.bus.subscribe` compile seam 限制，详见 `docs/plans/reports/696-response-parity.md`；提交 `bc17269a1` | VM E2E、Axum live server、`cargo th` |
| T-08 [x] | T-01..T-07 | 完成实现验收、警告/格式/workaround 审计并准备独立复审 handoff；SD-01..03 留给复审确认后由 merge workflow 沉淀 Specs/归档 | AC-07 门禁已执行并报告；`cargo check -p auto-lang` 和 `cargo th` 通过。`cargo tv`、`cargo tf` 均复现相同 3 项 P-053/UI 断言失败并在 fail-fast 下取消后续用例，细节见 `docs/plans/reports/696-verification.md`。`git diff --check` 通过；全仓 fmt check 被依赖/既有格式差异阻断。实现代码已提交，等待独立 review | `cargo check -p auto-lang`、`cargo th`、`cargo tv`、`cargo tf` |

## 9. 复审记录

- 2026-09-23 `/auto-plan:new` handoff：`stage: new`，`PLAN-696` revision 1，`outcome: pass`（可进入 work）；`next: work`，任务 T-01..T-08、验收 AC-01..AC-07。此处是计划合同检查，不是代码复审。
- 2026-09-23 work 执行基线：实现 worktree `D:/autostack/.wt/lang-696/auto-lang` / `plan-696-dev`，base `051e7b54023f420c3955c480d986e83197eb5899`；只读 Cargo 依赖 worktree `D:/autostack/.wt/lang-696/auto-down` detached at `3373a5cc6e3a00336613133db51906fb0940777d`。T-01 报告提交 `a227eebb3`，T-02 红样提交 `724096cd4`。
- T-04/T-05 owner decision：`cargo check -p auto-lang` exit 0；`cargo nextest run -p auto-lang --lib --features test-http-e2e -E 'test(/e2e_plan696_body_split_across_tcp_writes|e2e_sse_generator_handler/)' --no-fail-fast` 2/2 pass；实现与报告提交 `7d431992e`。编译输出有仓库既有 warning，T-08 继续对改动新增项和全局预存项分开审计。
- T-06：计划内分段 body/Content-Length 与 SSE 慢 producer/断连回归 5/5 pass；generator cleanup 删除 iterator 和 task。T-07：真实 api.at VM E2E 3/3、api_gen 定向单测 4/4、生成 Axum 的 015 notes / 017 chat-SSE / 023 auth+article live 断言全通过；详细数据和 VM bus 已知限制见 `docs/plans/reports/696-response-parity.md`。实现与 T-08 全档门禁尚待最终提交/记账。
- 2026-09-23 execution handoff：实现提交 `86df0015f`（SSE/HTTP owner）与 `bc17269a1`（Axum/API parity）；T-08 检查报告 `docs/plans/reports/696-verification.md`。`cargo th` 51/51 pass；`cargo tv` 为 1,518 pass / 3 fail / 508 skipped / 4,099 not run，`cargo tf` 为 1,414 pass / 3 fail / 112 skipped / 4,056 not run；同 3 个未修改 P-053 tests 另行逐项复现。`cargo check` pass，`cargo fmt --all -- --check` 因既有全仓格式差异退出 101。工作阶段到 `execution_done`；后续 `/auto-plan:review` 与 `/auto-plan:merge` 负责独立复核及 Spec 沉淀。

- 2026-09-23 `/auto-plan:review`：`stage: review | plan_id: PLAN-696 | plan_revision: 1 | outcome: pass | reviewed_commit: bc17269a1c172d4a58a2813d05912fb32baa8219 | base_commit: 051e7b54023f420c3955c480d986e83197eb5899 | dependency_revisions: auto-down=3373a5cc6e3a00336613133db51906fb0940777d`。
  - 复审基线 Plan 文件 SHA-256：`AE43BB6EA0A382ED6D392E41BE05706959E90F608494BFDE4D3C130D923DE0BC`；§5 `规范增量` 冻结快照 SHA-256：`5F10D76523D52CA073C65CFEBF1D896C3375D3752EF8ACA0A16348E39722A74B`。快照正文即本计划 §5 SD-01..03 表；复审期间未改规范正文或派生 ledger。
  - `spec_inputs`：`docs/specs/auto-lang/runtime/design/networking-stdlib.md` blob `3f3e9e8ae6e3088d15869884a5aa0d4e6c0474bf` / SHA-256 `AE2BFF58298B0E2DEF252AFE31DFC513C837B96A729C181007185B7334145E66`；`docs/specs/stdlib/design/http-server.md` blob `978d52c467a7b48d95363bc47f906eeaf853f637` / SHA-256 `6A007A89EF1973B32717A4E02C253EEF800A5773B21340E1403100B21726569B`；`docs/specs/goals.md` blob `3d267f1d0bf5c7f8c3140b8faa83c0e3f01f73d9`（GOAL-003 存在）；新目标 `docs/specs/stdlib/design/backend-assembly.md` 在基线不存在。派生 `.autoos/specs.json` 不作为权威输入。
  - `acceptance_results`：AC-01 pass（覆盖报告并抽查目标文件/符号；确认 `http.rs.at`、`sse.vm.at` 缺失）；AC-02 pass（复审重跑 `cargo th` 51/51，含 body 分段、短体、畸形长度、超限）；AC-03 pass（慢 SSE 健康请求与断连取消两项在 `cargo th` 通过）；AC-04 pass（`Rc<AutoVM>`/同线程 `LocalSet` 所有权审查，HTTP 路径原始指针/整数转运扫描零命中，编译通过）；AC-05 pass（015/017/023 VM 与 Axum 对拍及 api_gen 4/4 见 T-07 报告；复审定向 `plan370_015_behavior_tests::{d1_init_loads_seed_notes,d2_new_note_appends}` + `back_proxy_tests::{http_e2e_back_proxy_json_routes_and_session_state,http_e2e_back_proxy_missing_param_is_400}` 为 4/4，覆盖真实 015 合并调用正常返回及进程内 API 正常/400 错误响应；017 VM bus compile seam 已明确披露）；AC-06 pass（SD-01..03 的目标、事实边界、理由与 AC 对齐，GOAL-003 有效）；AC-07 pass（`cargo check -p auto-lang`、复审 `cargo th` 通过；`cargo tv`/`cargo tf` 的相同 3 项红测在计划基线独立复现，满足本 AC 的既有红项例外）。
  - `findings`：无阻断项。全档门禁历史结果沿用同一实现 commit、依赖 commit、Cargo.lock（SHA-256 `80C985E3A7CF2FAA314D3A14930BA148F00D3BF9E7A4039AA390F3838CAAF7C7`）及 nextest 配置（`nextest-full.toml` blob `5b2cf1b0adbceb609ebb9a83bc70a752198957ed`；daily blob `15db1b9a2ef69d3dfbe1d2978aec60ea818022e8`）。审阅者在同一实施会话中执行，独立上下文不可用；为降低依赖原 handoff 摘要，已按提交 diff、Specs 和可复现产物重建判断，并在计划基线源码快照中重跑 P-053 三项（结果与当前完全一致：0/3 pass，失败为 `widget_computed_passthrough_survives_reeval`、`widget_computed_store_arg_helper_chain`、`merged_mode_api_call_emits_warn_opcode`），故确认为既有失败而非 PLAN-696 回归。`cargo fmt --all -- --check` exit 101 的 730 个格式差异均来自 sibling AutoDown；PLAN-696 变更的 Rust 文件无格式差异。`git diff --check` pass；warning/debug-print 审计无新增告警或调试残留。
  - `evidence`：`docs/plans/reports/696-backend-inventory.md`、`696-vm-owner-decision.md`、`696-response-parity.md`、`696-verification.md`；复审命令 `cargo th` → 51/51、上述合并调用定向 nextest → 4/4、计划基线 P-053 定向 nextest → 同样 3 项失败。工作树 `D:/autostack/.wt/lang-696/auto-lang` 干净且 implementation changes 已提交。
  - `next: merge`。复审在同一实施会话内执行（独立上下文不可用）；结论由提交 diff、Specs、报告与计划基线重跑重建。

## 10. 待澄清事项

1. 对外部署等级后续决定；本计划只保证本机/示例服务正确性，不以未定义的公网吞吐阈值验收。
2. VM owner 的最终线程/LocalSet 组织由 T-04 spike 裁定；若涉及不等价的核心运行时契约，先修订 revision 与影响。
3. HTTP/IPC/合并路径未必覆盖每个示例；T-07 按真实可启动拓扑选样，不伪造不存在的传输路径。
