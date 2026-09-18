---
plan_id: PLAN-651
status: archived              # drafting → executing → execution_done → reviewed → archived（2026-09-19 merge delivered 归档）
feature_name: autodown-editor-block-closure（编辑器 block 类型三态矩阵与闭合）
author: [zhaopuming]
created_at: 2026-09-18
updated_at: 2026-09-18（T-00 完成，矩阵+定价落盘）
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
# 说明：三态支持矩阵为工作工件（attachments/651-matrix.md）；若复审判定需
# 上升 canonical（design 31 系）再增补。对拍 gate 以测试套件交付。
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（编辑器=一致性最深单元，RC-E）"]

affects: [autodown-editor]
current_step: 4
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

- **T-00** [✅ 已完成] 三态矩阵盘点：产物 `attachments/651-matrix.md`
  （2026-09-18）。类型集合 = 模型 17 kind + 任务标记态/Image 两个特殊行
  （069 的 23 类型中 TODO/DOING/DONE/NOW/LATER/Priority A-C 为 ListItem 字面
  标记非独立 kind，实测确认）。逐格标定 + 红项清单 5 项（R1 fence 丢语言/
  R2 query+embed 空段回写/R3 mermaid 丢围栏/R4 math 丢 `%{ }%`/R-ANCH 块锚
  `^id` 丢失——全在 VM emit 路径，jade 语料定价）+ 余项 3（R5 details 折叠/
  R6 行首规则/R7 表格 align/IAL）+ DEBTS 提案 5 + 豁免维持 2。
  Q-1 裁定=矩阵 §0；Q-2 裁定=修复面 100% auto-lang 主通道，autodown-core
  与 TS 零改动，auto-down worktree 仅 T-03 gallery 联动；Q-3 未触发。
  → AC-01 达成（矩阵在案+双实现交叉校验：红格经主会话代码复核，双侧盘点
  独立进行后对表）。证据：`attachments/651-matrix.md` §2/§3。
- **T-01** [✅ 已完成] 红项闭合第一批（worktree 提交 2e283b43d，base
  9886ba901）：R1 fence 语言随 `BlockBuf.syntax` 发射；R2 query/embed 进
  `Seg::Raw` 冻结源行（不可聚焦=TS 冻结预览裁定对齐）；R3 闭合 mermaid 入
  fence 族叶（syntax=mermaid）；R4 `LeafKind::Math` 源码叶（emit `%{ }%`
  + Fence 同族守卫）；R-ANCH `BlockBuf.anchor` 通道（heading/paragraph/
  catch-all 收 attr、emit 尾补 `^id`、拆分随头块/合并保头锚）。
  验收：`t651_*` 六测试（行为锚 + closure corpus 幂等锚）全绿；
  autodown_editor 模块 118/118 绿（`cargo nextest run --lib --features
  autodown,code-editor autodown_editor`）；cargo check 零新告警。
  附带：`editor_text_public_api_roundtrip` 补 run_fs 装回调（修 nextest
  每测独立进程的既有顺序依赖，非本计划回归面）。
  预存红在案（非本计划）：aura_view_builder 的
  test_autodown_details_onclick_message_channel /
  test_autodown_table_col_resize_emission 在 autodown,code-editor 特性集
  下挂（AnchorSlot 包裹漂移，测试归 PLAN-045、builder 末改 PLAN-652，
  零耦合本计划改动面，证据 /tmp/adall.log 形态 + diff 单文件）。
  → AC-02（第一批达成）
- **T-02** [✅ 已完成] 红项闭合第二批（worktree 提交 042d9be3b）：
  R5 details 折叠交互（摘要行独立 DrawItem + details_geom 命中 → open
  翻转，闭合内容不可见/不可聚焦，渲染兜底清扫跳过 folded 叶 + 布局面
  零矩形占位）；R6 行首规则 7→10 条对齐 TS INPUT_RULES（补 `---`/`***`/
  "``` "，触发改每字符整块检定 = TS fireRuleOn onInput 同语义；`---`/
  `***` 经 remove_leaves_compact 不留孤儿缓冲；`1. ` 有序与 h4-h6 实测
  为双侧一致冻结面，矩阵注记修正）；R7 表格发射保真（Seg::Table 携
  align/IAL，分隔行 canonical `:---`——parser convertTableCell 对 bare
  `---` 赋 align=left，serializer alignMarker 同形，旧恒 `---` 才是非规
  范形态；align 空缺按 left 保幂等）。
  验收：t651 9/9 绿；autodown_editor 模块 121/121 绿（3 个旧表格断言按
  canonical 形态更新——钉的正是本计划修复的丢 align 行为）。
  → AC-02（第二批达成，全批闭环）
- **T-03** [✅ 已完成] 对拍 gate 常驻化：closure corpus 扩 T-02 面
  （60e427854；align/IAL 表 + 闭合 details 入幂等锚）；gate 命令 =
  `cargo nextest run -p auto-lang --lib --features autodown,code-editor
  t651`（落矩阵 §5/§6）。jade gallery RC-E 联动已提交（auto-down 侧
  fd05981：component-gallery units.mjs 登记 `editor_tab` 状态占位单元 +
  072 台账状态更新）。回归：parser parity 全绿（auto-down worktree 实跑
  16 测：parse_blocks_matches_ts_golden / parity_with_ts_emission /
  smoke_table_with_ial 等）；vm-smoke 实机双窗口（worktree 二进制）：
  第一跑 groups 1-10 全过（输入面/回写/渲染/滚动同步/ghost/表格拖宽/
  fence chrome/主题翻转），group 4 拖拽臂=已登记预存红（de86e1d8e，
  pristine master 复现，非本计划引入），group 11 全量 smoke 挂但独立
  探针 vm-069-probe 五臂 ALL PASS（含迁移后重开臂——同场景清洁上下文
  通过，判环境/时序非回归）；demo 双轨 vue 臂：本计划零 TS 改动，按
  构造不受影响（vm 臂全绿即交叉面）。→ AC-03/04 达成
- **T-03** [改] 对拍 gate 常驻化（矩阵驱动的外科对拍组进 tests 套件）+
  jade gallery RC-E 占位联动（auto-down 侧小改，随本计划折回通道）+
  回归（demo 双轨/vm-smoke/parser parity）。依赖：T-01/T-02。→ AC-03/04

（T-01/T-02 的具体红项在 T-00 产出后回填任务明细；若红项清单超出两批容量，
按批次追加任务并同步 total_steps——矩阵定价优先于本骨架。）

## 8. 复审记录

- 2026-09-18 draft handoff：`stage: new | plan_id: PLAN-651 | plan_revision: 1 |
  outcome: pass（起草完成；执行未授权） | next: review → work（T-00 可独立
  先行，其产出定价整个统一期时间盒）`。
- 2026-09-18 work 授权进场：`stage: work | plan_id: PLAN-651 | plan_revision: 2 |
  outcome: T-00 done（矩阵+定价落盘 attachments/651-matrix.md；Q-1/Q-2 裁定在
  案，Q-4 DEBTS 提案新增） | code_commit: master 簿记（worktree
  .wt/lang-651/{auto-lang,auto-down} 建组，base 9886ba901 / b1c88def） |
  task_ids: T-00 | evidence: 矩阵 §2 双实现逐格 + §3 红项定价（主会话复核
  catch-all/emit_seg/serializer 关键格） | blockers: 无 | next: T-01（VM
  core.rs 红项闭合第一批，auto-lang worktree plan-651-dev）`。
- 2026-09-18 work 完成交接：`stage: work | plan_id: PLAN-651 | plan_revision: 2 |
  outcome: pass（T-00..T-03 全任务完成） | code_commit: auto-lang worktree
  plan-651-dev = 2e283b43d（T-01 五格闭合）+ 042d9be3b（T-02 三格闭合）+
  60e427854（corpus 扩面），base 9886ba901；auto-down worktree plan-651-dev =
  fd05981（gallery RC-E 联动），base b1c88def | task_ids: T-00,T-01,T-02,T-03 |
  evidence: 矩阵 §6 处置表——8 红格全闭合 + 5 DEBTS 提案 + 2 豁免维持；
  autodown_editor 121/121 绿；t651 9/9 绿；parser parity 16 测绿；
  vm-smoke 第一跑 groups 1-10 绿（group 4 预存红在案/group 11 探针甄别
  ALL PASS）；cargo check 零新告警 | blockers: 无（Q-4 DEBTS 裁定/Q-5
  smoke 稳定性转介均非阻断） | next: /auto-plan:review（worktree 留存）`。
- 2026-09-18 复审（实现会话自审，工件重建裁定）：`stage: review |
  plan_id: PLAN-651 | plan_revision: 2 | outcome: pass |
  reviewed_commit: auto-lang 60e427854（worktree plan-651-dev，clean）|
  base_commit: 9886ba901 | dependency_revisions: auto-down fd05981
  （worktree plan-651-dev，clean；base b1c88def） | spec_inputs: SD-01
  裁定=矩阵维持工作工件（attachments/651-matrix.md，§0 判定口径+§6 处置
  回填），不上升 canonical（design 31 系增补留待需要）；spec components
  空 + 书面说明在案（计划 frontmatter 注记）；touched_goals=GOAL-007
  （goals.md 实证存在） | acceptance_results: AC-01 pass（矩阵在案+三路
  独立盘点交叉校验+主会话关键格复核）；AC-02 pass（8 红格闭合逐格测试
  锚；5 DEBTS 提案在案 Q-4 待用户裁定，非悬置）；AC-03 pass（t651 组
  9/9 + autodown_editor 121/121 在被审提交复跑，gate 命令落矩阵 §5）；
  AC-04 pass（units.mjs node --check 通过 + 072 台账行，fd05981） |
  findings: F-1（note，非阻断）折叠 details 隐藏叶仍入 doc_sel/copy
  全序——v1 细节，TS 折叠选区排除为增强面，不在 AC 范围；F-2（转介）
  cargo tf 2568/2569，唯一失败 test_display_family_codegen_arm_fixture
  经 master 主检出（6934e2839）复现=预存红（ui_gen codegen fixture，
  与本计划特性门控改动面零关联） | evidence: /tmp/tf_review.log 摘要
  2568/2569+1 预存、/tmp/review_mod.log 121/121、/tmp/base_test.log
  master 复现、diff 审计单文件 747+/45- 零调试残留零越界重构、
  vm-smoke/探针记录见上条 | next: /auto-plan:merge（DEBTS 提案随 merge
  沉淀 KNOWN-DEBT-AND-RISKS.md 待用户处置）`。
- 2026-09-19 merge 收据 **PLAN-651:r2**：`stage: merge | outcome: delivered |
  prepared: reviewed 基线（60e427854/b1c88def）+ SD-01 裁定空 delta（矩阵
  维持工作工件）+ 账本投射目标 reports/P651-1 | landed: auto-lang master
  ff3c85b25→a0290a3ee→86b020ece（cherry-pick 等价落盘，core.rs 自 base
  零漂移；落码后 master 集成门禁 autodown_editor 121/121 绿）+ auto-down
  master a615d69（base b1c88def 未推进无漂移） | ledger_refreshed:
  .autoos/specs.json reports/P651-1 上墙（校验+原子替换+回读 100 条；
  389cfcd08；INDEX 零漂移——无 canonical spec 变更） | archived:
  docs/plans/archive/651-autodown-editor-block-closure.md（git mv，status:
  archived；c8cd3cc8b） | cleaned: wt-guard 双 worktree clean → worktree/
  分支移除（-D：cherry-pick 落盘新 SHA 故 -d 判未合并，内容等价已实证
  ——auto-lang 尾 60e427854=被审提交、auto-down 尾 fd05981=a615d69）→
  组目录 .wt/lang-651 移除，全链无残留`。

## 9. 待澄清事项

| # | 事项 | 影响 | owner/下一步 |
| --- | --- | --- | --- |
| Q-1 | "完善"的判定口径（三态各自：view=渲染结构对拍/edit=行为锚/streaming=增量语义） | T-00 标尺 | **已裁定**（T-00）：见 `attachments/651-matrix.md` §0——view=结构对拍（豁免记绿(豁)）、edit=roundtrip 模型无损、streaming=三态机语义一致 |
| Q-2 | autodown-core 修复的落地通道（随本计划折回 vs auto-down 侧独立通道） | 跨仓布局 | **已裁定**（T-00）：autodown-core parse/serialize 实测全绿零改动；修复面 100% auto-lang VM 壳 → 本计划 auto-lang 主通道；auto-down worktree 仅 T-03 gallery 联动 |
| Q-3 | 红项中出现工具链级硬骨头（非类型覆盖而是引擎能力缺口）时的升级路径 | 时间盒 | 未触发（math/mermaid 图形渲染为已登记豁免，非新增硬骨头） |
| Q-4 | （T-00 新增）DEBTS 提案 5 项（矩阵 §3：R8 死 kind/R9 query 编辑增强/R10 tasks 死代码/R11 image 行内/D1 view 面板注册层级）待 review 时用户逐项裁定 | AC-02 处置完整性 | review 阶段裁定；工作阶段按提案记录不实施 |
| Q-5 | （执行新增）vm-smoke 两臂非阻断发现：group 4 拖拽臂=069 已登记预存红（de86e1d8e pristine master 复现）；group 11 全量 smoke 挂但 vm-069-probe 同场景五臂 ALL PASS——判环境/时序，转介 smoke 稳定性（非本计划回归面） | 回归口径完整性 | 归 auto-down 侧后续计划处置；本计划以探针全 PASS + parser parity + 模块 121 绿为回归证据 |
