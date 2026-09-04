---
design_id: DESIGN-27
status: accepted
describes: target-state
scope: repository-knowledge-lifecycle
created_at: 2026-09-04
updated_at: 2026-09-04
implemented_by_plans: [PLAN-543]
supersedes: []
---

# 27 — Knowledge Base Lifecycle：代码、规格与决策的同步模型

> 本文是 AutoLang 知识库治理的 canonical design。它定义各类知识资产的职责和同步
> 生命周期；`docs/specs/README.md` 负责操作规约，当前审计证据见
> `docs/reports/knowledge-base-sync-audit-2026-09-04.md`。

## 1. 背景与问题

现有 Plan + Design + Spec 范式方向正确：Plan 管过程，Spec 管沉淀，Design 保存意图，
review/merge 负责把任务结论带回长期知识。然而实践中出现四类问题：

1. Design 同时描述未来方案和当前实现，失效陈述缺少状态标签；
2. Markdown spec 与 `.autoos/specs.json` 都可写，产生双重真相；
3. merge 倾向追加 Plan 叙事，而不是重写简洁的 current-state；
4. lint 按“多久没改”判断新鲜度，不能发现“代码改了但对应 spec 没改”。

目标不是增加更多文档，而是让每类事实只有一个权威承载层，并让同步失败可被自动发现。

## 2. 目标与非目标

### 2.1 目标

- 新贡献者和 Agent 能从仓库级入口逐层定位到 current-state、原因和执行证据；
- 未来设计与当前事实明确分开，历史决策可追溯且不可被静默改写；
- 结构、引用和关键语义能在 CI 中持续校验；
- 并行 Plan 不抢号、不覆盖同一路径、不依赖本地不可共享台账；
- 文档与代码同库、同评审、同一变更闭环。

### 2.2 非目标

- PLAN-543 不批量重写全部 module spec；
- PLAN-543 不修改 auto-musk 的四技能实现；
- PLAN-543 不引入外部知识库服务或搜索平台；
- Generated Catalog 不承载设计理由或人工解释。

## 3. 五层知识模型

| 层 | 唯一职责 | canonical source | 生命周期 |
|---|---|---|---|
| Design/RFC | target-state、约束、选项与方案 | `docs/design/` | proposed → accepted/rejected → implemented/superseded |
| ADR | 已接受的重要决定、理由、权衡与后果 | module `architecture.md` 或独立 ADR | accepted 后不可改写，只能 supersede |
| Spec | 已实现的 current-state 与公共契约 | `docs/specs/` Markdown | 随代码重写，始终面向当前版本 |
| Plan | 单次任务的动作、证据、偏离和复审 | `docs/plans/` + `archive/` | drafting → executing → execution_done → reviewed → archived |
| Generated Catalog | crate/module/path/API/test/追踪关系 | 从 Git 跟踪内容生成 | 不手工编辑，可随时重建 |

### 3.1 Canonical-source 矩阵

| 问题 | 去哪里找 | 不应由谁回答 |
|---|---|---|
| 系统现在怎样工作？ | Spec + executable tests | Plan、旧 Design |
| 为什么选择这种架构？ | accepted ADR | overview、commit message |
| 下一目标态是什么？ | accepted Design/RFC | current-state Spec |
| 某次变更怎样完成？ | archived Plan + Git diff | Spec 正文 |
| 仓库里实际有哪些实体？ | Generated Catalog | 手写数量表 |

## 4. Current-state 与 Target-state

Design 必须显式声明 `describes: target-state` 或 `describes: current-state`。新设计默认是
target-state；实现完成后不把原文伪装成现状，而是：

1. 将 Design 标为 `implemented`；
2. 把实际行为蒸馏到 current-state Spec；
3. 将重要取舍写为 ADR；
4. 保留 archived Plan 作为执行证据。

根级架构总览允许描述 current-state，但只能引用代码或 Spec，且应记录 `last_verified_at`
或 `last_verified_commit`。未经验证的未来内容必须链接到 target-state Design。

## 5. 最小元数据契约

新建或重大修订文档采用以下最小字段；旧文档在被触及时渐进补齐，不做一次性迁移。

```yaml
# Design/RFC
status: proposed | accepted | rejected | implemented | superseded
describes: target-state | current-state
implemented_by_plans: []
supersedes: []

# Current-state Spec
status: active | deprecated
component: <stable-id>
code_paths: []
owners: []
last_verified_commit: <git-sha>

# ADR
status: proposed | accepted | rejected | deprecated | superseded
decision_date: YYYY-MM-DD
supersedes: []

# Plan
plan_id: PLAN-NNN
status: drafting | executing | execution_done | reviewed | archived
affects: []
```

## 6. 同步流水线

```text
Requirement
  → Design/RFC（仅架构显著变更需要）
  → Plan（动作合同）
  → code + tests
  → independent review
  → current-state Spec 重写 + ADR 追加
  → Generated Catalog/index 再生
  → Plan archive
```

merge 的知识操作是“蒸馏”而不是“复制”：Spec 只保留当前事实，ADR 只保留重大决定，
Plan 保存完整过程。reviews/reports 可以索引 Plan，但不得把验收全文复制进每个 module。

## 7. 三类一致性检查

### 7.1 Structural verification

- Cargo workspace、npm workspace、顶层资源与 project 卡一致；
- `code_paths` 能解析且 component id 唯一；
- Plan ID 在 active、archive 和并行 worktree 中全局唯一；
- Generated Catalog 重建后无 diff。

### 7.2 Referential verification

- Markdown 相对链接和锚点有效；
- Spec 中引用的源码路径、类型、函数或命令仍存在；
- Plan、ADR、goal 和 component 的关联目标可解析。

### 7.3 Semantic verification

- 关键架构声明映射到 executable test、contract test 或 architecture fitness function；
- API/schema/协议变化必须更新对应 Spec；
- “无文档影响”需要在 Plan/review 中给出可审计理由。

## 8. Change-based freshness

日期只能提示风险，不能证明同步。目标态用 component manifest 建立
`code_paths → spec_paths → tests → owners` 映射：

- 代码路径变化且 Spec 未变化时，review 必须确认 current-state 是否受影响；
- Spec 声称的源码入口消失时，CI 直接失败；
- 长期未改但代码也未改的 Spec 不因年龄单独失败；
- 高频变化组件可以设置更严格的 review owner 和验证命令。

## 9. `.autoos/specs.json` 兼容策略

当前 `.autoos/specs.json` 被 Git ignore，且依赖 merge 技能本地 upsert，因此是兼容期的
本地投影/缓存，**不是 canonical source**。兼容期内继续允许技能写入，但每个结论必须同时
落到 Git 跟踪的 current-state Spec、ADR 或 Plan 索引。

目标态由后续 Knowledge lint/catalog 计划确定稳定 schema，并从 Git 跟踪的 Markdown
frontmatter、Cargo/npm metadata 和 Plan 索引单向生成机器 catalog。生成物是否提交由消费端
需求决定，但不得出现“Markdown 和 JSON 都由人手工修改”的双写模式。

## 10. 并行安全与所有权

- Plan 编号只能由 `scripts/new-plan.sh` 在默认检出原子分配；
- 分配时扫描 active + archive，CI 再做全局唯一性兜底；
- 每个 Plan 使用唯一 sibling-group worktree，禁止 junction/symlink；
- Plan frontmatter 的 `affects` 应在执行前展开为文件/组件范围，用于冲突预警；
- module spec 和其对应源码应共享 owner；知识入口自身也必须有 owner；
- 多仓修改使用同一 group 的 sibling worktree，不直接修改依赖仓主检出。

## 11. 迁移阶段

### 阶段 A：可信入口（PLAN-543）

沉淀审计、定义五层模型、修正仓库级已知 stale 陈述，不批量重写模块。

### 阶段 B：Knowledge lint/catalog

实现全局 Plan ID、代码清单、符号/链接、生成物一致性和 change-based CI 门禁。

### 阶段 C：Auto-plan v3

调整 `new/work/review/merge`：new 读取受影响 Spec/ADR 索引；work 支持显式 context refresh；
review 验证 docs impact；merge 先写 current-state/ADR，再生成 catalog，最后归档 Plan。

### 阶段 D：Module spec rebaseline

按 types、trans、interpreter、runtime、comptime 等模块分批复核，每批都有源码证据、owner
和 scoped verification，不组织一次性“大扫除”。

## 12. 风险与回滚

| 风险 | 缓解 |
|---|---|
| 新模型增加写文档成本 | 只对被触及组件增量补元数据，L0 保留轻量路径 |
| Spec 与 JSON 兼容期继续漂移 | 明确 Markdown canonical；后续改为单向生成 |
| 语义检查误报 | 先 warning 后 gate，按组件逐步启用 |
| 并行 Plan 触碰同一入口 | `affects` 冲突预警 + owner review + 小批量合入 |
| 历史 Design 被误删 | 只标 historical/superseded，不做破坏性清理 |

若后续自动化不可用，可回滚生成器和 CI gate，但本设计确立的职责分层、Markdown canonical
和 archived Plan 不可变原则仍然成立。

## 13. 后续工作包

1. **Knowledge lint/catalog**：实现 manifest、全局唯一性、代码引用、INDEX/catalog 再生与 CI。
2. **Auto-plan v3**：在 auto-lang/auto-musk sibling worktree 中同步升级四技能和仓内规约。
3. **Module spec rebaseline**：逐模块消除 stale current-state，并建立 owner/test 映射。
