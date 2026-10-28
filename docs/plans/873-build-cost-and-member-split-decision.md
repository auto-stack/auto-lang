---
plan_id: PLAN-873
status: drafting
feature_name: 构建成本测量与 workspace member 拆分决策（THR-D1）
author: [DeepSeek Harness session]
created_at: 2026-10-28
updated_at: 2026-10-28
plan_revision: 1
current_step: 0
total_steps: 8
supersedes_spec_components: [docs/specs/language-sdk/project.md]
new_spec_components: [docs/specs/repository-boundaries/build-cost-baseline.md]
touched_goals: []              # 有意留空：只测量与决策，不改 GOAL
affects: [language-sdk, auto-ui, auto-man, repository-boundaries, test-infrastructure]
depends_on: [PLAN-867]
---

# [PLAN-873] 构建成本测量与 workspace member 拆分决策（THR-D1）

> **整 Plan 启动门 G873 = U867**（三仓完整档与基线就绪，测量才能用同一档位口径对照）。门未齐只做只读登记。
> 本计划是 [KNOWN-DEBT THR-D1](KNOWN-DEBT-AND-RISKS.md) 的落地：**"先测量，再决定是否按 crate 拆 workspace member"**，不承诺拆分本身。

## 0. 变更摘要

拆分把三仓分开后，**crate 粒度并没有变细**：语言侧仍由 `auto-lang` + `auto-lang-core` 两个根包承载 parser/codegen/engine/native/stdlib/http_server/trans 全部家族（843 只做了文件级分族），UI 侧由 5 包承载（`renderer.rs` 42,337 行仍在 `auto-ui-runtime`），Man 侧 9 成员 + `core/` 子 workspace。THR-D1 的原始观察是：任何改动都会触发整 crate 重编 + 测试二进制重链，2026-10-02 样点为 sccache 79% 命中下 `cargo check -p auto-lang` 57s、单个 worktree `target` 17G、non-cacheable 801 次；当时结论是**"不测不做"**。

本计划交付一份可复跑的构建成本测量协议与基线（冷/热、增量改一处、链接、nextest 启动、全档墙钟），据此建立"member 拆分收益 vs 成本"的决策模型，并给出**拆/不拆/试点**的明确结论与（若拆）下一份实施计划的范围。

## 1. 目标

- **G1 协议可复跑**：冷态/热态、sccache 命中/失效、并发/单实例、重复次数与统计量的定义固定，任何人按记录命令复跑得到同量级结果。
- **G2 基线数字**：三仓各包的 `check`（冷/热）、单文件增量 `check`、测试二进制重链、nextest 启动（含 registry 一次构建）、裸 `cargo t` 墙钟——全部带原始日志与哈希。
- **G3 变更站点测量**：在语言侧取 ≥4 个代表改点（parser / codegen / engine / trans 各一），UI 取 ≥2（compiler / runtime），Man 取 ≥1，分别测增量成本，说明"改动爆炸半径"。
- **G4 决策模型**：把"member 拆分"的预期收益（受影响编译单元缩小、测试二进制减少）与成本（manifest/feature 并集、跨 member 循环、CI 矩阵、构建缓存命中性、维护面）量化成同一张表，给出结论。
- **G5 结论可用**：结论含"若拆，先拆哪个 member、验收指标是什么、风险是什么"；若结论为不拆，须给出替代手段（增量/缓存/测试分组）与实测依据。

**非目标**：不在本计划执行 member 拆分（那要另一份实施计划）；不改 `.cargo/config.toml` 的 `jobs`/linker/`RUST_MIN_STACK` 等现网设置（除测量用环境覆盖）；不改产品代码；不承诺任何提速百分比（只报实测）；不重跑重型档（gallery 围栏/1M churn）作为测量对象，除非决策确需（届时单实例 + 预算记档）。

## 2. 架构方案

1. **测量协议（T-01）**：固定 ① 机器与磁盘状态（空闲、无并发 agent、target 位置、磁盘余量）；② sccache 状态（关/清空后的冷态、命中率读数）；③ 热态定义（同一 target 连跑第二次）；④ 统计（3 次取中位+极差）；⑤ 单实例锁（复用 `D:/autostack/.locks` 机制，避免与其它会话互毁）。
2. **分层测量（T-02/T-03）**：
   - L1 冷/热 `cargo check -p <包>`（三仓各包）；
   - L2 单文件增量：`touch` 一个家族文件后 `cargo check`（含 `-p auto-lang` 与 `-p auto-lang-core` 对照）；
   - L3 测试面：`cargo nextest list` 冷/热（注册表构建税）、测试二进制链接时间、裸 `cargo t` 墙钟；
   - L4 变更站点：按 §1-G3 的代表文件逐点重复 L2。
3. **模型（T-04）**：收益侧用"受影响 crate 的生产行/编译单元/测试 ID 占比"；成本侧用"manifest/feature 新增面、跨 member 依赖、CI 组合数、缓存键变化、维护人时"。给出三种方案（不拆／部分拆／全拆）的对照表。
4. **结论与交接（T-05/T-06）**：结论写入新规范页 `build-cost-baseline.md`；若"拆"，产出下一份实施计划的范围草案（成员划分、里程碑、验收指标）。

判定原则：

- 数字必须来自**同机同输入**的可复跑命令；不同时的历史数字只作对照，不混入同一张表。
- 测量期间不得有其它会话构建/测试（否则按重测处理，不解释成"波动"）。
- 收益模型不得用"理论上更快"替代实测；若某项无法测量（如 CI 矩阵成本），必须标注为估算并给依据。

## 3. 技术栈

cargo / rustc / sccache（`--show-stats`）/ `cargo nextest`（list 与 run）/ `cargo build --timings`（可选，用于编译单元拆分观察）/ PowerShell 计时与日志采集 / Python 汇总 / 机器级单实例锁。

## 4. 需求分析与背景调查

### 4.1 授权与轮次

- 用户 2026-10-28 要求把"867 之后的后续计划"写成文档；本计划是其中一份（THR-D1）。
- 本轮只写合同：未跑任何测量、未改配置、未建树。

### 4.2 已观察事实（2026-10-28）

| # | 事实 | 证据 |
|---|---|---|
| F-1 | THR-D1 原始观察：单 crate 76.6 万行/1,650 文件（2026-10-02）；sccache 79% 命中下 `cargo check -p auto-lang` 57s；单 worktree `target` 17G；non-cacheable 801 次；结论"不测不做" | [KNOWN-DEBT THR-D1](KNOWN-DEBT-AND-RISKS.md) |
| F-2 | 拆分后 crate 粒度未变细：语言侧 `auto-lang` + `auto-lang-core` 承载 parser/codegen/engine/native/stdlib/http_server/trans 全家族（843 为文件级分族） | [843 handoff](attachments/843/handoff.md) |
| F-3 | UI 侧 5 包 + `shell-pack`；`renderer.rs` 42,337 行（P total）在 `auto-ui-runtime`；生成物 19,687 行 | 844 T-01 基线（`net-slimming.md` 桶表） |
| F-4 | Man 侧根 9 成员 + `core/` 子 workspace（独立 lock/target） | [man-three-repository-delivery.md](../specs/repository-boundaries/man-three-repository-delivery.md) |
| F-5 | 现网构建设置：`.cargo/config.toml` `jobs=12`、`rust-lld`、`RUST_MIN_STACK=16MB`（与其注释一致） | 三仓 `.cargo/config.toml` |
| F-6 | 重型档（gallery 围栏 ~800s 冷、1M churn）与 nextest 注册表税（`ui_gen` 每进程 ~1.2s）是既有已知成本项 | `AGENTS.md` 全档资源表；`cargo tu` 注释 |
| F-7 | 867 将提供三仓完整档与档位口径；869 将提供 per-crate 行数/文件/测试归属（可作为收益模型输入） | [867](867-split-equivalence-test-closure.md)、[869](869-crate-inventory-and-subtraction-batch-2.md) |

### 4.3 规范依据

- [language-sdk/project.md](../specs/language-sdk/project.md)：语言包组与依赖闭包（本计划 SD 落点）。
- [repository-boundaries/project.md](../specs/repository-boundaries/project.md)：三仓身份与依赖方向。
- [net-slimming.md](../specs/repository-boundaries/net-slimming.md)：per-crate 行数口径（收益模型输入）。

### 4.4 假设与不声称

- **A1**：sccache 可被临时关闭/清空以取得冷态真实值（关闭仅限测量期，测后恢复原状并记录）。
- **A2**：`cargo timings` 能给出编译单元级视角；若不可用，退化到 `check`/`build` 墙钟与 target 增量大小。
- **不声称**：本轮未测任何数字；F-1 的 2026-10-02 样点只作历史对照，不是当前基线。

## 5. 详细设计

### 5.1 测量矩阵

| 层 | 对象 | 命令（示意） | 记录 |
|---|---|---|---|
| L1 冷/热 check | 三仓每包 | `cargo check -p <pkg>`（冷：清 sccache + 干净 target；热：同 target 重跑） | 墙钟（3 次中位+极差）、sccache 命中率、target 增量 |
| L2 单文件增量 | 语言 parser/codegen/engine/trans；UI compiler/runtime；Man runtime | `touch <file>` 后 `cargo check -p <pkg>` | 墙钟、是否触发同 crate 其它家族重编 |
| L3 测试面 | 每包 | `cargo nextest list -p <pkg>`（冷/热）、测试二进制链接、裸 `cargo t` | 列表耗时、链接耗时、全档墙钟 |
| L4 变更站点 | §1-G3 代表文件 | 同 L2，逐点 | 爆炸半径（受影响单元数/ID 数） |

### 5.2 决策模型

- **收益**：受影响 crate 的生产行/编译单元/测试 ID 在单点改动后的重编范围缩小比例；测试二进制数量下降带来的链接节省。
- **成本**：member 数量与 manifest 维护、feature 并集变化、跨 member 依赖可能引入的循环（需要拆 trait/接口）、CI 组合数、缓存键粒度变化、发布/安装清单变化。
- **方案对照**：① 不拆（保留现状 + 优化缓存/分组）；② 部分拆（只拆最热 crate 的稳定子域，如 parser 与 trans）；③ 全拆（per-family crate）。
- 结论必须给出推荐方案 + 触发条件（例如"若 L2 中 trans 单点改动 > X 秒且占全档 > Y%，则拆 trans"）。

### 5.3 结论与交接

- 写入 `docs/specs/repository-boundaries/build-cost-baseline.md`：协议、基线表、模型、结论、复跑命令、与 THR-D1 的闭合关系。
- 若结论为"拆"：产出实施计划范围（成员划分、依赖改造点、里程碑、验收指标：冷 check P50、单点增量 P50、裸 t 墙钟），交下一份计划；本计划不实施。

### 5.4 执行合同

| 字段 | 内容 |
|---|---|
| 规则与边界 | G1–G5；只测量与决策；不动现网设置；单实例 |
| 实际路径 | 协议 → 测量 → 原始日志 → 汇总表 → 模型 → 结论 → 规范页/交接 |
| 所有权与状态 | 基线表为验收输入（改需原因+版本）；测量 owner=本计划；结论 owner=用户确认（若涉及拆 member） |
| 区分性用例 | V-01..V-05：协议复跑一致、冷热区分、并发污染拒绝、历史数字隔离、结论可追溯 |
| Oracle 与关闭 | 每 AC 绑命令与期望；关闭需原始日志路径 + 机器状态记录 |

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/build-cost-baseline.md | 无构建成本基线 → 测量协议 + 三仓基线 + member 拆分决策模型与结论 | THR-D1 要求"数据支撑再立项" | AC-01..AC-04 |
| SD-02 | modify | docs/specs/language-sdk/project.md | 语言包组说明补"crate 粒度与增量成本现状 + 若拆的结论指向" | 现有页只说成员职责 | AC-02, AC-04 |

## 6. 测试设计

`owner_stage` 均为 `work`；本轮未执行。

| Case ID | AC / invariant | 输入或 setup | 真实入口 | 可观察期望 | 断言/证据 | 覆盖边界 |
|---|---|---|---|---|---|---|
| V-01 | AC-01 协议可复跑 | 同机空闲态 | 按协议完整重跑 L1 一轮 | 两次中位数差 ≤ 10%（超出需解释环境） | 原始日志 + 中位表 | 三仓各一包 |
| V-02 | AC-02 冷热可区分 | 清 sccache + 干净 target vs 同 target 重跑 | `cargo check` | 冷态显著高于热态（倍数记档），证明协议确实区分 | 两组数字 | 至少语言侧两包 |
| V-03 | AC-03 变更站点可信 | 并发会话构建中 | L2 测量 | 检测到并发即拒绝并重测（不计入基线） | 锁/进程检查日志 | 单实例语义 |
| V-04 | AC-04 模型可追溯 | 历史数字（F-1）与当前数字 | 汇总表 | 历史与当前分列，不混算；估算项标注依据 | 表 + 注释 | 全部行 |
| V-05 | AC-05 结论可用 | 模型输出 | 结论页 | 三方案对照 + 推荐 + 触发条件；若拆，范围可被独立执行 | 结论页 | member 决策 |

## 7. 验收标准

| ID | owner_stage | 可观察验收 |
|---|---|---|
| AC-01 | work | 测量协议冻结且可复跑（V-01/V-02 两态区分成立） |
| AC-02 | work | 三仓分层基线数字齐全，附原始日志、命令、机器状态、重复次数与统计量 |
| AC-03 | work | 变更站点测量覆盖 ≥7 点，给出爆炸半径；并发污染检测有效 |
| AC-04 | work | 决策模型三方案对照 + 明确推荐 + 触发条件；历史/当前分列、估算标注 |
| AC-05 | work | 结论写入 `build-cost-baseline.md`；若拆，交接范围可独立执行；THR-D1 闭合或明确转下阶段 |
| AC-06 | review | 独立终审复核协议、数字、模型与结论（含抽点复跑）；merge 门 pending |
| AC-07 | merge | SD/ledger 沉淀、归档与清理；若结论为拆，记录下一计划入口 |

## 8. 执行步骤

8 项 = 6 work + 1 review + 1 merge；G873=U867。

| 状态 | ID | owner_stage | 依赖/启动门 | 实际交付/验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | G873 | 协议冻结（冷/热、sccache 状态、并发检测、统计量、预算）；机器状态记录模板 | AC-01 |
| [ ] | T-02 | work | G873；T-01 | L1/L3 基线：三仓各包冷热 check、nextest list、测试链接、裸 t | AC-02 |
| [ ] | T-03 | work | G873；T-02 | L2/L4：≥7 个变更站点增量成本 + 爆炸半径 | AC-03 |
| [ ] | T-04 | work | G873；T-03 | 汇总表 + 三方案决策模型（收益/成本/触发条件） | AC-04 |
| [ ] | T-05 | work | G873；T-04 | 结论页 + 若拆的范围草案；THR-D1 闭合或转阶段 | AC-05 |
| [ ] | T-06 | work | G873；T-05 | SD-01/SD-02 落地 + `attachments/873/handoff.md` 冻结送审 | AC-02, AC-04 |
| [ ] | T-07 | review | G873；T-06 冻结提交 | 独立复审（抽点复跑、模型健全性、结论与数据一致） | AC-06 |
| [ ] | T-08 | merge | G873；T-07 pass | SD/ledger、归档、清理；记录后续入口 | AC-07 |

**实施树**：测量可在主检出只读执行（不改代码），若需临时改 `.cargo/config.toml` 做对照，必须用环境覆盖或独立 worktree（`D:/autostack/.wt/lang-873/auto-lang`，分支 `plan-873-dev`），测后恢复并留对照记录。
**门禁**：无产品代码改动（Category C/A 级）；测量本身按协议单实例执行，不与其它会话并发。

## 9. 复审记录

2026-10-28 / stage: new / plan_id: PLAN-873 / revision: 1 / outcome: **pass（合同可执行；实施等 U867）** / next: 门成立后 work T-01 / changed: 新建 T-01..T-08、AC-01..AC-07、V-01..V-05、SD-01..SD-02。

- 取号：`scripts/new-plan.sh build-cost-and-member-split-decision`（873 → 874）。
- 本轮只写合同：未测任何数字、未改配置、未建树。
- 说明：本计划不承诺拆分；结论可能是"不拆 + 优化缓存/分组"，届时 THR-D1 按结论闭合并把替代手段登记。

## 10. 待澄清事项

| ID | 事项 | 影响 | 处置 |
|---|---|---|---|
| D-1 | 是否允许测量期临时关闭/清空 sccache | 冷态真实性 vs 影响他人 | T-01 定义窗口并单实例执行；测后恢复并记录 |
| D-2 | 是否把 CI 矩阵/发布清单成本纳入模型 | 结论保守度 | 纳入但标注为估算 |
| D-3 | 若结论为"拆"，是否立即立项 | 排期 | 交用户决定；本计划只给范围草案 |
