---
plan_id: PLAN-696
status: drafting
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
current_step: 0
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
| T-01 | 无 | 盘点 `stdlib/auto/`、`crates/a2r-std/src/`、`crates/auto-lang/src/{compile.rs,autovm_persistent.rs,vm/ffi}`、`crates/auto-man/src/api_gen.rs`；新增 `docs/plans/reports/696-backend-inventory.md` | AC-01，附路径和缺项 | 文件/符号抽样 |
| T-02 | T-01 | `vm/ffi/http_server.rs` HTTP E2E 区新增原始 TCP 分段/短体/超限、慢 SSE + 并发请求红样 | AC-02/03 失败基线 | 定向 `cargo th`，记录结果 |
| T-03 | T-02 | `http_server.rs::handle_connection_async` 统一 header/body byte 读取、长度/上限/超时错误，保留 multipart 行为 | AC-02 | 定向 E2E + 既有 multipart/绑定测试 |
| T-04 | T-02 | `lib.rs` VM 服务入口与 `http_server.rs` 做 VM owner 可编译 spike；新增 `docs/plans/reports/696-vm-owner-decision.md` | AC-04 的所有权选择与关闭证明；硬约束则修订 plan_revision，不提交不安全 workaround | `cargo check -p auto-lang` + owner 小样 |
| T-05 | T-04 | 按 T-04 决策改 VM 生命周期/连接任务持有；移除跨线程 `usize` 地址恢复 | AC-04 | `cargo check -p auto-lang` + HTTP E2E |
| T-06 | T-03,T-05 | `http_server.rs` SSE 取帧改可 await 的有界结果桥；断连/结束信号回收 | AC-03 | 并发 E2E、线程/任务回落 |
| T-07 | T-03,T-06 | 用 `examples/ui/{015-notes,017-chat,023-realworld}/src/back/api.at` 和合并调用测试建立响应 parity，记录预存差异 | AC-05 | 集成/金样、`cargo th` |
| T-08 | T-01..T-07 | 验收、警告/格式/workaround 审计，确认 SD-01..03；执行 `/auto-plan:review`，通过后 `/auto-plan:merge` 更新 Specs/归档 | AC-06/07 | `cargo check -p auto-lang`、`cargo th`、`cargo tv`；必要时一次 `cargo tf` |

## 9. 复审记录

- 2026-09-23 `/auto-plan:new` handoff：`stage: new`，`PLAN-696` revision 1，`outcome: pass`（可进入 work）；`next: work`，任务 T-01..T-08、验收 AC-01..AC-07。此处是计划合同检查，不是代码复审。

## 10. 待澄清事项

1. 对外部署等级后续决定；本计划只保证本机/示例服务正确性，不以未定义的公网吞吐阈值验收。
2. VM owner 的最终线程/LocalSet 组织由 T-04 spike 裁定；若涉及不等价的核心运行时契约，先修订 revision 与影响。
3. HTTP/IPC/合并路径未必覆盖每个示例；T-07 按真实可启动拓扑选样，不伪造不存在的传输路径。
