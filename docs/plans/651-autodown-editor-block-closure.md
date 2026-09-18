---
plan_id: PLAN-651
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: autodown-editor-block-closure（编辑器 block 类型三态矩阵与闭合）
author: [zhaopuming]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
# 说明：三态支持矩阵为工作工件（attachments/651-matrix.md）；若复审判定需
# 上升 canonical（design 31 系）再增补。对拍 gate 以测试套件交付。
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（编辑器=一致性最深单元，RC-E）"]

affects: [autodown-editor]
current_step: 0
total_steps: 4
---

# [PLAN-651] autodown-editor-block-closure——编辑器 block 类型三态矩阵与闭合

> 定位：三层 parity 机制的 **RC-E 单元（引擎对拍）**——jade 统一期（auto-down
> PLAN-072 盘点）中唯一归入 autodown lighthouse 流的单元；同时是 A' 战略
> （auto-down ARCHITECTURE §8.4）的**解冻触发条件**：autodown_editor 对核心
> markdown 子集达到 web 引擎（Tiptap 实现）级。
>
> 现状（用户实况 2026-09-18）：autodown 原生编辑器（edit/view/streaming
> 三态合一）**大部分 block 类型已可显示并编辑，少数类型未完善**——本计划
> 首要工作就是把"少数"枚举成清单并闭合。

## 0. 变更摘要

1. **三态支持矩阵成文**：block 类型 × view/edit/streaming × 双实现
   （autodown-engine TS ↔ autodown-core Rust）的逐格状态矩阵（绿/缺/差异），
   语料 = autodown/demo corpus + jade 实际文档集。
2. **红项闭合**：矩阵中的缺/差格子按优先级分两批闭合（第一批高优类型，
   第二批余项）；闭合 = 双实现对拍锚固化（沿用 parser parity/roundtrip
   纪律），不是单侧补丁。
3. **对拍 gate 常驻化**：矩阵驱动的外科对拍测试进套件；jade component-gallery
   的 RC-E 状态占位单元联动更新（与 auto-down PLAN-072 基建对齐）。

## 1. 目标

1. 矩阵在案且双实现交叉校验（防"单侧自以为绿"）。
2. 红项闭合：每格要么绿、要么经用户裁定转 DEBTS（逐项处置，无悬置）。
3. 对拍 gate 常驻（测试套件在库，CI/门序列可跑）。
4. jade gallery RC-E 单元状态联动（auto-down PLAN-072 基建的占位转实）。

**非目标**：jade app 层挂载/装配（PLAN-072/L3 域）；web 侧 Tiptap 实现的
功能增强（TS 侧仅作对拍基准，除非对拍暴露 TS 缺陷）；新 block 类型发明。

## 2. 架构方案

```
语料（demo corpus + jade 文档集）
   ↓ T-00 枚举
三态矩阵（attachments/651-matrix.md）
   block 类型 × {view, edit, streaming} × {engine-TS, core-Rust}
   ↓ 红项清单（缺/差/未定型）+ 优先级定价
T-01 第一批闭合（高优类型：以 jade 实际文档集高频类型先）
T-02 第二批闭合（余项）
   ↓ 每格闭合 = 双实现对拍锚（同一语料双跑比照，roundtrip/行为锚）
T-03 对拍 gate 常驻化（tests/ 套件）+ jade gallery RC-E 联动
```

- **双实现对拍纪律**：与 parser parity/roundtrip 金标同源——同一语料喂
  TS 与 Rust 实现，结构/行为逐锚比照；差异即矩阵红格的修复验收。
- **跨仓布局**：autodown-engine TS 与 autodown-core(Rust) 在 auto-down 仓
  （autodown/packages/engine），VM 原生件壳在 auto-lang——组 worktree
  `.wt/lang-651/{auto-lang, auto-down}`；autodown-core 修复的落地归
  auto-down 侧通道（随本计划折回或在 auto-down 侧对应通道，T-00 后明确）。

## 3. 需求分析与背景调查

**授权记录**：2026-09-18 用户裁定（三层机制 RC-E lighthouse 立项，"尽早
启动、最长跑"）；取号时 646/648/649 已被其他会话占用，顺延至 651。仅起草；
执行未授权。

**既有依据**：
- 用户实况（2026-09-18）：三态合一已实现，大部分类型可显示并编辑，少数
  未完善——矩阵预期以绿为主、红项有限。
- PLAN-069 slash manifest 23 类型冻结清单（Text/Heading 1-6/Bullet/Numbered/
  Quote/Code Block/TODO/DOING/DONE/NOW/LATER/Priority A-C/Divider/Table/
  Callout/Details）= 矩阵的类型集合起点，T-00 按语料实测增删。
- PLAN-068 时代"textarea+value 投影"记载已被 068→069 的演进超越（PLAN-070
  期间引述有滞后，PLAN-651 以 T-00 实测矩阵为准重立基线）。
- 对拍基建：parser parity/roundtrip 金标、demo 双轨、vm-smoke。

## 4. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add | docs/design/31-autodown-editor-parity-matrix.md（T-00 产出转正，若复审判定矩阵属工作工件则维持 attachments 并在此说明） | 无 → 三态×类型×双实现矩阵 + 对拍锚清单 | RC-E 的当前知识载体；A' 解冻条件的度量面 | AC-01 |

（矩阵转正与否 T-00 后定；转正则 SD-01 生效，否则本表在 review 时修订。）

## 5. 测试设计

- 矩阵交叉校验：同一语料双跑（TS/Rust）逐格比照，禁单侧自证。
- 闭合验收：每红格一个对拍锚（语料 + 期望结构/行为），锚进常驻套件。
- 回归：demo 双轨、vm-smoke、既有 parser parity 全绿（本计划不动 parser，
  仅编辑器面）。

## 6. 验收标准

| ID | 可观察行为 | 验证方法 |
| --- | --- | --- |
| AC-01 | 矩阵在案且双实现交叉校验 | attachments/651-matrix.md + 双跑记录 |
| AC-02 | 红项全部处置（闭合或用户裁定转 DEBTS） | 矩阵逐格状态 + 处置记录 |
| AC-03 | 对拍 gate 常驻 | tests 套件新增对拍组，门序列可跑 |
| AC-04 | jade gallery RC-E 单元联动 | auto-down PLAN-072 gallery 的 RC-E 占位状态更新（联动提交在 auto-down 侧） |

## 7. 执行步骤

> worktree：`.wt/lang-651/{auto-lang, auto-down}` 组布局（autodown-core 与
> engine TS 在 auto-down 仓）。

- **T-00** [调查/决策工件] 三态矩阵盘点：类型集合（069 manifest 23 起点 +
  语料实测增删）× 三态 × 双实现逐格标定；红项清单 + 优先级定价 + 工期估计
  （A' 解冻时间盒的输入）。产物：`attachments/651-matrix.md`。
  依赖：无。→ AC-01，并定价 T-01/T-02
- **T-01** [改/新] 红项闭合第一批（T-00 定价的高优类型；每格 = Rust 侧
  闭合 + TS 对照锚 + 对拍测试）。依赖：T-00。→ AC-02
- **T-02** [改/新] 红项闭合第二批（余项）。依赖：T-01。→ AC-02
- **T-03** [改] 对拍 gate 常驻化（矩阵驱动的外科对拍组进 tests 套件）+
  jade gallery RC-E 占位联动（auto-down 侧小改，随本计划折回通道）+
  回归（demo 双轨/vm-smoke/parser parity）。依赖：T-01/T-02。→ AC-03/04

（T-01/T-02 的具体红项在 T-00 产出后回填任务明细；若红项清单超出两批容量，
按批次追加任务并同步 total_steps——矩阵定价优先于本骨架。）

## 8. 复审记录

- 2026-09-18 draft handoff：`stage: new | plan_id: PLAN-651 | plan_revision: 1 |
  outcome: pass（起草完成；执行未授权） | next: review → work（T-00 可独立
  先行，其产出定价整个统一期时间盒）`。

## 9. 待澄清事项

| # | 事项 | 影响 | owner/下一步 |
| --- | --- | --- | --- |
| Q-1 | "完善"的判定口径（三态各自：view=渲染结构对拍/edit=行为锚/streaming=增量语义） | T-00 标尺 | T-00 矩阵首列裁定 |
| Q-2 | autodown-core 修复的落地通道（随本计划折回 vs auto-down 侧独立通道） | 跨仓布局 | T-00 后与 auto-down 侧对齐 |
| Q-3 | 红项中出现工具链级硬骨头（非类型覆盖而是引擎能力缺口）时的升级路径 | 时间盒 | 触发即回 A' 复查点（auto-down ARCHITECTURE §8.4） |
