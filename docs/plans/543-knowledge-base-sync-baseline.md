---
plan_id: PLAN-543
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: Knowledge Base Sync Baseline
author: [Codex]
created_at: 2026-09-04
updated_at: 2026-09-04

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/design/00-intro.md: 注册 Design 27，并停止把手工文档数量作为完整性门禁"
  - "docs/design/01-architecture.md: 校准 AutoVM、多后端、QueryEngine 与 self-host current-state"
  - "docs/architecture-clarification.md: 将 Evaluator/三后端/366 tests 降为 historical correction"
  - "docs/design/autoplan-spec-ledger.md: 将 Markdown 定为 canonical、specs.json 定为兼容投影"
  - "docs/specs/README.md: 增补五层职责、canonical-source 与 sibling-group 操作规约"
  - "docs/specs/overview.md: 刷新仓库规模、VM execution 与 PLAN-532/536 活跃线"
new_spec_components:
  - "docs/design/27-knowledge-base-lifecycle.md: 新增知识库生命周期 canonical design"
  - "docs/reports/knowledge-base-sync-audit-2026-09-04.md: 新增可复现同步审计基线"
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [global, process, architecture]
current_step: 12
total_steps: 12
---

# [PLAN-543] 知识库同步基线与生命周期设计

## 变更摘要

在不触碰 PLAN-532、PLAN-536 实施代码及其 worktree 的前提下，为 AutoLang 建立一份
可长期维护的知识库同步基线：固化 Design / ADR / Spec / Plan / Generated Catalog 的职责，
记录 2026-09-04 的代码—文档漂移审计，刷新仓库级架构入口，并把后续自动化与模块
rebaseline 工作拆成可独立执行的计划候选。

本计划是纯文档与流程设计任务，不修改 `crates/`、`packages/`、`auto/`、`examples/`
中的实现，不修改 `D:/autostack/auto-musk/.agents/skills`，也不创建 junction/symlink。

## 目标

1. 让新贡献者和 Agent 能从一个入口准确理解当前仓库的系统边界、模块与主要执行路径。
2. 明确“当前事实、未来设计、历史决策、执行过程、机器索引”各自唯一的承载层。
3. 把本轮审计中已经证实的仓库级漂移写入受版本控制的文档，而不是只保留在对话中。
4. 明确 `.autoos/specs.json` 当前的非 canonical 地位，并给出后续迁移目标与兼容期。
5. 为后续 lint/catalog 自动化、四技能升级、逐模块 spec rebaseline 提供边界清晰的拆分方案。
6. 保证本计划的 diff 与正在实施的 PLAN-532、PLAN-536 无路径重叠。

## 架构方案

采用“一个事实、一种职责”的五层知识模型：

| 层 | 职责 | 更新策略 |
|---|---|---|
| Design/RFC | 目标态、备选方案、约束 | 带 `status` 与 `describes`，可被 supersede |
| ADR | 已接受的重要决策、理由、代价 | 接受后不可改写，只能新增 superseding ADR |
| Spec | 已实现的当前态与公共契约 | 与代码同一变更内更新，可重写 |
| Plan | 一次任务的动作、证据、偏离与复审 | 执行期可变，归档后不可变 |
| Generated Catalog | crate/module/path/测试/追踪关系 | 从代码与 Git 跟踪文档生成，不手工维护 |

本计划只完成模型设计、审计基线和仓库级入口校准。生成器、CI 门禁和外部技能修改均作为
后续计划，不在本计划中顺手实现。

## 需求分析与背景调查

`docs/specs/overview.md` 将 AutoLang 描述为多目标语言、AutoVM、AutoUI/桌面生态与工具链
组成的 monorepo，并将 `auto-lang` 划分为 frontend、types、comptime、interpreter、vm、
trans、runtime、ui、mcp 九个逻辑模块。该总览仍记录 2026-08-28、463 个源码文件以及
PLAN-462～465 的活跃状态，和 2026-09-04 的仓库现状存在时间差。

本轮只读审计确认：

- 工作区约 20 个 Rust package，`crates/auto-lang/src` 约 528 个 Rust 文件；
- 旧架构入口仍包含已删除 Evaluator、QueryEngine 待接入、VM 32-bit/约 120 opcode 等陈述；
- 当前 VM spec 已记录 178 opcode 与 NaN-boxed `u64`；
- `python scripts/spec-lint.py --stale-days 7` 返回 0 errors、9 warnings，包含 3 个断链和
  6 个超过阈值的 module overview；
- `.autoos/specs.json` 受 `.gitignore` 的 `*.json` 规则影响且不在 Git 中，不能作为团队
  唯一事实源；
- 现有 lint 侧重 frontmatter、相对链接和按日期的 stale 判断，尚未验证代码清单、
  全局 Plan ID、代码符号引用或 Markdown/机器台账一致性；
- PLAN-520、原 PLAN-533 的实施已由用户确认完成并合并；当前 PLAN-532、PLAN-536
  正在其他进程中执行，本计划必须避免修改它们及其实现范围；
- 当前序列中 PLAN-540、541、542 已被占用，PLAN-543 由 `scripts/new-plan.sh` 原子分配。

## 详细设计

### 1. 新增知识库生命周期设计

新增 `docs/design/27-knowledge-base-lifecycle.md`，至少包含：

- 当前问题与非目标；
- 五层知识模型和 canonical-source 矩阵；
- current-state 与 target-state 的区分；
- 文档 frontmatter 最小字段；
- Plan → Spec/ADR → Archive 的写入顺序；
- change-based freshness、结构/引用/语义三类一致性检查；
- 并行 Plan 的 ID、owner、路径冲突约束；
- `.autoos/specs.json` 兼容期与目标态；
- 分阶段迁移方案、风险和回滚策略。

在 `docs/design/00-intro.md` 注册 Design 27，并将其标为本轮知识库治理的 canonical design。

### 2. 保存审计基线

新增 `docs/reports/knowledge-base-sync-audit-2026-09-04.md`，保存可复现的仓库规模、模块图、
已验证漂移、lint 输出摘要、同步度分层判断和审计命令。报告只记录证据，不承担规范职责。

### 3. 校准仓库级入口

只修订已经由代码或当前 spec 直接证实的仓库级事实：

- `docs/design/01-architecture.md`：移除旧 Evaluator 与 QueryEngine deferred 陈述；
- `docs/architecture-clarification.md`：更新执行后端和 self-host 状态，区分历史快照；
- `docs/specs/overview.md`：更新仓库规模、当前活跃轨道和维护日期；
- `docs/specs/README.md`：写明五层职责、canonical source 与 generated artifact 规则；
- `docs/design/autoplan-spec-ledger.md`：增加 Design 27 的兼容说明，不在本计划中重写技能实现。

不在本计划中批量改写九个 module overview；它们进入后续 rebaseline 计划。

### 4. 后续计划拆分

在 Design 27 中形成三个独立后续候选，不预占编号：

1. Knowledge lint/catalog：全局 Plan ID、代码清单、符号/链接、生成物一致性和 CI 门禁；
2. Auto-plan v3：调整 new/work/review/merge 的上下文与沉淀契约，涉及 auto-musk 时使用
   sibling-group 多仓 worktree；
3. Module spec rebaseline：按 types/trans/interpreter/runtime/comptime 等模块分批复核。

## 测试设计

本计划没有 Rust、schema 或文档生成器改动，按仓库 Category A 门禁禁止运行 `cargo t`、
`cargo tf` 和 `docs_gen`。验证仅包含：

- `python scripts/spec-lint.py --stale-days 7`：记录修改前后差异，不要求本计划清零所有
  module stale 告警，但不得新增 error、断链或 frontmatter 问题；
- `python scripts/spec-index.py` 后执行 `git diff --exit-code -- docs/specs/INDEX.md`，确认索引
  可重现；若脚本暴露既有格式问题，只记录为后续 lint/catalog 验收项，不越界修复；
- `rg` 验证旧的 Evaluator、QueryEngine deferred、463 文件、462/463 活跃线等已知仓库级
  陈述不再以“当前事实”出现；历史引用必须明确标注为 historical；
- `git diff --name-only` 确认 diff 只包含本计划列出的文档和计划跟踪文件；
- `git status --short` 确认未触碰 PLAN-532、PLAN-536 及实现目录。

## 验收标准

- [x] Design 27 被创建并在 `docs/design/00-intro.md` 注册。
- [x] 审计报告包含架构图、规模数据、至少五项代码—文档漂移和可复现命令。
- [x] Design/RFC、ADR、Spec、Plan、Generated Catalog 的职责与 canonical source 无歧义。
- [x] `.autoos/specs.json` 的当前身份、兼容策略和目标态被明确记录。
- [x] 仓库级架构入口不再把旧 Evaluator、待接入 QueryEngine、VM 32-bit/约 120 opcode
  或 PLAN-462～465 状态作为未经标注的当前事实。
- [x] 后续自动化、技能升级、模块 rebaseline 被拆成三个互不混杂的候选工作包。
- [x] `spec-lint` 没有新增 error 或 warning；现存 warning 有明确归属。
- [x] 没有修改 Rust/JS/Auto 实现、Cargo/npm 清单、PLAN-532、PLAN-536 或其 worktree。
- [x] 未创建 junction、symlink 或其他 reparse point。

## 执行步骤

1. 在 `D:/autostack/.wt/lang-543/auto-lang` 创建 `plan-543-dev` worktree，记录
   `git status --short` 与 `git worktree list --porcelain` 基线；验证：
   `git -C D:/autostack/.wt/lang-543/auto-lang status --short`。
   [✅ 已完成] worktree 位于 `D:/autostack/.wt/lang-543/auto-lang`，分支为
   `plan-543-dev`；`git status --short --untracked-files=no` 无输出，基线干净；
   532、536、539、542、544 均位于各自独立 worktree。
2. 在 `docs/reports/knowledge-base-sync-audit-2026-09-04.md` 写入仓库规模与 Git 快照；
   验证：`rg -n "工作区|Rust|design|spec|plan" docs/reports/knowledge-base-sync-audit-2026-09-04.md`。
   [✅ 已完成] 报告记录约 20 个 workspace package、528 个核心 Rust 文件、Design/Spec/Plan
   规模和 `432e15d` worktree 基线；关键词验证通过。
3. 在同一报告中写入实际模块图和主执行路径；验证：
   `rg -n "frontend|QueryEngine|AutoVM|Transpiler|AutoUI" docs/reports/knowledge-base-sync-audit-2026-09-04.md`。
   [✅ 已完成] 报告已给出 frontend → semantic → compile infra → VM/trans/UI → desktop
   的执行路径；五组关键节点验证通过。
4. 在同一报告中写入已验证漂移、lint 输出和复现命令；验证：
   `rg -n "spec-lint|Evaluator|178|\.autoos/specs.json" docs/reports/knowledge-base-sync-audit-2026-09-04.md`。
   [✅ 已完成] 报告列出 7 类高置信度漂移、0 errors/9 warnings 基线和完整复现命令；
   四组关键词验证通过。
5. 创建 `docs/design/27-knowledge-base-lifecycle.md` 的五层模型、canonical-source 矩阵和
   frontmatter 契约；验证：`rg -n "Design/RFC|ADR|Spec|Plan|Generated Catalog|canonical" docs/design/27-knowledge-base-lifecycle.md`。
   [✅ 已完成] Design 27 已定义五层职责、canonical-source 问答矩阵和渐进式最小元数据；
   六组关键词验证通过。
6. 为 Design 27 补充同步流水线、三类一致性检查、并行安全和迁移阶段；验证：
   `rg -n "structural|referential|semantic|change-based|并行|迁移" docs/design/27-knowledge-base-lifecycle.md`。
   [✅ 已完成] Design 27 已包含 structural/referential/semantic verification、change-based
   freshness、并行安全、四阶段迁移与回滚策略；关键词验证通过。
7. 更新 `docs/design/00-intro.md` 注册 Design 27；验证：
   `rg -n "27-knowledge-base-lifecycle" docs/design/00-intro.md`。
   [✅ 已完成] Design 27 已在知识资产入口和“流程与知识体系”表中注册为 canonical
   design；链接验证通过，下一域级章号推进为 28。
8. 更新 `docs/design/01-architecture.md` 与 `docs/architecture-clarification.md` 的仓库级
   当前事实，并为仍保留的历史描述加显式标签；验证：
   `rg -n "Evaluator|QueryEngine|historical|历史" docs/design/01-architecture.md docs/architecture-clarification.md`。
   [✅ 已完成] 两个入口现描述 AutoVM、多目标 transpiler、AutoUI 与已集成 QueryEngine；
   Evaluator、三后端和 366 tests 均仅以 historical correction 出现。
9. 更新 `docs/specs/overview.md` 的规模、日期与活跃轨道；验证：
   `rg -n "2026-09-04|528|PLAN-532|PLAN-536" docs/specs/overview.md`。
   [✅ 已完成] overview 已更新为 2026-09-04 快照、约 528 个核心 Rust 文件，并明确
   PLAN-532/PLAN-536 两条并行开发线；四组关键词验证通过。
10. 更新 `docs/specs/README.md` 与 `docs/design/autoplan-spec-ledger.md` 的职责和兼容说明；
    验证：`rg -n "canonical|Generated Catalog|Design 27|specs.json" docs/specs/README.md docs/design/autoplan-spec-ledger.md`。
    [✅ 已完成] 操作规约和 Design 26 已统一为 Markdown current-state canonical、
    specs.json 兼容投影、Generated Catalog 目标态，并记录 Design 27 与 sibling-group 路径。
11. 运行 `python scripts/spec-lint.py --stale-days 7` 和 `python scripts/spec-index.py`，记录
    结果并确认 `docs/specs/INDEX.md` 可重现；验证：
    `git diff --exit-code -- docs/specs/INDEX.md`。
    [✅ 已完成] spec-lint 保持基线 `0 errors, 9 warnings, 0 infos`，未新增问题；
    spec-index 生成 26 projects，`git diff --exit-code -- docs/specs/INDEX.md` 返回 0。
12. 执行范围与健康检查，确认仅修改计划列出的文档；验证：
    `git diff --check`、`git diff --name-only`、`git status --short`，并在计划中记录证据。
    [✅ 已完成] `432e15d..472b4a4` 范围只包含 8 个计划内文档；`git diff --check`
    返回 0，worktree `git status --short` 无输出。实现提交：`472b4a4f1`。

## 复审记录

### 2026-09-04 独立复审（Codex，`/auto-plan:review`）

复审对象：`plan-543-dev`，实现提交 `472b4a4f1`，基线 `432e15dabc`；实际 diff 为
8 个文档、474 insertions、88 deletions。按 Category A 门禁未运行 Cargo 测试。

| # | 结论 | 独立证据 |
|---|---|---|
| 1 | PASS | `docs/design/27-knowledge-base-lifecycle.md:14` 声明 canonical design；`docs/design/00-intro.md:124` 完成注册 |
| 2 | PASS | 审计报告 `:14/:28/:45/:82` 分别包含规模、模块图、7 项漂移和复现命令 |
| 3 | PASS | Design 27 `:49-65` 给出五层职责与 canonical-source 问答矩阵；Spec README 同步落规约 |
| 4 | PASS | Design 27 `:156-164` 明确 specs.json 为 ignored 本地投影、目标态单向生成 |
| 5 | PASS | 修改后的入口仅在 historical correction 中出现 Evaluator/deferred；overview `:87-95` 不再复述 462～465 为当前线；Design 05 的 32-bit/约 120 被审计报告显式列为待 module rebaseline 的旧文档 |
| 6 | PASS | Design 27 `:175-193/:208-212` 将 lint/catalog、Auto-plan v3、module rebaseline 分为独立阶段/工作包 |
| 7 | PASS | 重跑 `python scripts/spec-lint.py --stale-days 7`：`0 errors, 9 warnings, 0 infos`，与执行前基线一致；warning 分属阶段 B/D |
| 8 | PASS | `git diff --name-only 432e15d..HEAD` 仅列 8 个 docs 文件，无 Rust/JS/Auto/Cargo/npm/PLAN-532/536 路径 |
| 9 | PASS | `bash /mnt/d/autostack/wt-guard.sh /mnt/d/autostack/.wt/lang-543/auto-lang` 输出 `clean` |

补充门禁：`python scripts/spec-index.py` 重建 26 projects，INDEX 无内容 diff；
`git diff --check 432e15d..HEAD` 返回 0；复审结束时 worktree clean。

**遗漏/延后/workaround 猎查**：

- 遗漏：未发现计划步骤或验收子项缺失；8 个实际文件与计划声明一致。
- 延后：Generated Catalog/CI、auto-musk 技能升级、module rebaseline 均在已确认计划中显式
  划为后续工作包，不属于执行者静默缩 scope。
- Workaround：specs.json 双写仅作为有边界的兼容期存在，文档明确禁止其覆盖 canonical
  Markdown；不是隐藏 workaround。
- 非阻断债务：计数口径差异 P543-D1、Design 26 历史未勾选项 P543-D2，已登记
  `docs/plans/KNOWN-DEBT-AND-RISKS.md`，分别归属阶段 B/C。

**复审结论：PASS**。全部验收标准通过，无阻断债务；状态推进为 `reviewed`，可交给
`/auto-plan:merge`。

## 待澄清事项

无。默认采用“Git 跟踪的 Markdown/current spec 为 canonical，机器 JSON 为兼容期投影”的
方向；具体生成格式和 CI 实现留给后续 lint/catalog 计划，避免本计划把设计与实现混为一体。
