---
plan_id: PLAN-639
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: blueprint-tier（Block 层更名 Blueprint + 平台化地基）
author: [zhaopuming]
created_at: 2026-09-17
updated_at: 2026-09-17
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/blueprint/contract.md（Blueprint 六问契约：slot 树/action 点/状态归属/变体/打包解析/双形态语义）
touched_goals: ["GOAL-011: Blocks 一等公民生态（更名 Blueprint 并升级消费机制）"]

affects: [blueprint, autoui-skill, blocks]
current_step: 0
total_steps: 9
---

# [PLAN-639] blueprint-tier——Block 层更名 Blueprint 与平台化地基

## 0. 变更摘要

UI 三层（原 Widget / Block / App）的中层正式更名 **Blueprint**（代码缩写 `bp`），
消除三重撞名（auto-down 文档块 PBlock、语言 AST block、UI 组装层 Block），并把
Design 17 的 Skill 模型从"copy/eject 单轨消费"升级为**三通道分级消费**：

- **L1 import/bind（新增，平台化主通道）**：应用经跨包解析直接消费共享 bp 包 +
  声明式绑定（slot→布局、action 注入、token），不产生可漂移副本——固化产物是
  产物不是资产；
- **L2 copy reference（现有）**：`auto bp add --reference` 拷贝参考实现，落地文件
  归应用所有（eject 语义保留，用于离线/深度定制场景）；
- **L3 agent generation（现有）**：AI 组装，兜底结构性残余，产出后走**变体提升
  评审**（提升为 spec slot/action 点，下个应用免费用）。

工具链前置同步落地：VM/a2ts 双轨跨包 `.at` 解析、a2ts 向 vue 轨发射
`actions{}`/`menubar`/`toolbar` 配置（当前 vue 轨不消费，jade web 无命令系统的
直接根因）。VM 轨三约束（回调 props 退化/子树快照不可见/view fn 条件不求值）
本计划只做**调查与决策工件**，修复本体另立项。

消费应用侧（jade-garden / 041-auto-edit / musk / widgets-gallery 的副本收敛）
不在本计划，由 auto-down PLAN-070 及后续计划承接。

## 1. 目标

1. **术语与身份**：UI 组装层在活跃文档/spec/代码/CLI 中统一命名 Blueprint；
   三层正式定名 Widget / Blueprint / App。历史归档文档不改（保留历史语境）。
2. **契约固化**：Blueprint 契约六问落 `docs/specs/blueprint/contract.md`——
   ①输入（data props）②输出（events/actions 上抛）③状态归属（scoped store vs
   注入平台服务，服务单例）④参数与变体（样式钩子/内容参数化，小差异走参数
   不走 fork）⑤打包与解析（包内路径/跨包引用/依赖声明）⑥双形态同语义
   （vue 发射轨与 VM 解释轨行为一致，含 actions 配置双轨发射）。
3. **消费机制**：`auto bp` CLI 三通道可用；L1 bind 产出声明式绑定工件并标
   GENERATED；变体提升评审流程成文并接入 L3 工作流文档。
4. **双轨地基**：跨包 `.at` 解析（VM 轨 + a2ts 轨）可消费 blueprints 包；
   actions/menubar/toolbar 配置 vue 轨发射生效、VM 轨语义不回归。
5. **门禁收口**：既有 blocks 门禁（palette-drift guard 等）更名后全绿；
   新增重命名完整性 grep 门；DEBTS 登记 VM 三约束与消费侧迁移移交项。

**非目标**：VM 三约束修复本体；消费应用副本迁移（PLAN-070 承接）；运行时
动态插件/manifest 加载（终态另议，届时先裁扩展介质：VM .at 包 vs WASM vs
子进程）；auto-down 文档块（PBlock）概念不动；blueprint 的运行时生成式 UI。

## 2. 架构方案

```
共享 bp 包（blueprints/<kind>/<name>/：spec.md + reference/<v>.at + gotchas.md）
        │
        ├─ L1 import/bind：app pac.at 声明包依赖 → 跨包解析（VM 轨 + a2ts 轨）
        │     + 绑定声明（bind.at：slot→布局容器 / action 注入 / token 映射）
        │     → 产物=声明式绑定（GENERATED），零副本
        ├─ L2 copy reference：auto bp add --reference → 落地 .at 归应用所有（现状保留）
        └─ L3 agent generation：spec + 需求 → AI 组装 .at → acceptance 门
              → 变体提升评审（→ 反哺 spec 自由区/slot/action 点）

命名与代码落点：
  CLI          auto block  →  auto bp（旧名兼容别名 + 弃用提示）
  Registry     crates/auto-lang/src/ui_gen/block/registry.rs → bp/registry.rs
  CLI 实现     crates/auto/src/cmd_block.rs → cmd_bp.rs
  包库目录     blocks/ → blueprints/（kind 分类不变）
  画廊示例     examples/blocks-gallery → examples/bps-gallery
  设计文档     docs/design/blocks/ → docs/design/blueprints/（Design 17 改写保留归位注记）
  spec 模块    docs/specs/blocks/ → docs/specs/blueprint/（+ contract.md 新增）
```

跨包解析不另起炉灶：PLAN-635 已交付 style recipe 跨包引用机制（607 配方语言
层之上的消费侧），T-00 调查其解析面（依赖声明位置、解析序、缓存）能否承载
widget/bp 级 `.at` 引用，裁定"扩展"或"平行复用其模式"；`$AUTO_LANG_ROOT`
等 env 覆盖 → 组内 `../auto-lang` → 主检出的解析序（AGENTS.md worktree 红线
条款）沿用。

## 3. 需求分析与背景调查

**授权记录**：2026-09-17 用户裁定并授权起草本计划（auto-lang 仓），范围=更名
+ 三通道消费机制 + 双轨地基 + 调查工件；**仅起草，未授权执行**；预算/自动
续跑限制未指定。执行需另行 `/auto-plan:work`。

**证据路径**（2026-09-17 实勘）：

| 事实 | 位置 |
| --- | --- |
| 三层设计与 Skill 模型（Block 缺位论述、双产物模型） | `docs/design/blocks/blocks-first-class.md`（Design 17，Plan 468 归位注记） |
| 包格式 / agent 工作流 / 数据源约定 | `docs/design/blocks/{block-package-format,agent-generation-workflow,datasource-convention}.md` |
| 包库本体（form/data-display/editor/navigation） | `blocks/`（README：Skill-tier、两消费路径、eject 语义） |
| CLI 与 Registry | `crates/auto/src/cmd_block.rs`；`crates/auto-lang/src/ui_gen/block/{mod,registry}.rs` |
| 跨包机制先例 | PLAN-635 style recipe 跨包引用（637 计划 §0 引述） |
| 术语撞名实据 | auto-down `jade-garden` PBlock/blocks_store（文档块）；`.at` 语言 AST block |
| 消费侧副本漂移（本计划的动因，移交 PLAN-070） | jade desktop 与 041 的 filetree 四件 sha256 各异；`041` 的 `tree_chevron` 已分叉；jade README §9.5 tabs_store 孪生登记册 |
| vue 轨命令系统缺位根因 | a2ts 不消费 actions/menubar/toolbar 配置（jade `front/auto` grep 零命中；041 为 VM 轨范本） |
| 目标账 | `docs/specs/goals.md` GOAL-011（Blocks 一等公民生态，部分达成）、GOAL-008（Rung 2 = blocks 层）、GOAL-007（双端一致） |

**Specs 状况**：`docs/specs/blocks/`（project.md）现状只描述 copy/生成两通道，
无消费分级与产物纪律条目 → 由本计划 规范增量 补齐（SD-01..03）。

## 4. 详细设计

### 4.1 Blueprint 契约（六问裁定，写入 SD-01 新 spec）

| # | 问题 | 裁定 |
| --- | --- | --- |
| 1 | 输入 | spec.md frontmatter 声明 data props（名/型/必选/缺省）；L1 绑定与 L2/L3 落地均须满足声明，缺省可省 |
| 2 | 输出 | events 声明（名/负载）；行为统一走 action 点：bp 消费应用 `actions{}` 注册表注入的实现，bp 内只声明所需 action id 与语义契约 |
| 3 | 状态归属 | bp 可带 scoped store（局部 UI 状态）；workspace/tabs/theme/keybinding 等**平台服务单例注入，禁止 bp 私有副本**（jade §9.5 孪生教训成文） |
| 4 | 参数与变体 | 样式钩子走 token/recipe（635/607 机制）；结构性小差异走 slot；reference variants 是变体的物化样本 |
| 5 | 打包与解析 | 包 = blueprints/<kind>/<name>/；应用经 pac.at 声明依赖；解析序 env→组内→主检出；版本面 MVP=主检出单版本（多版本/锁面列待澄清） |
| 6 | 双形态 | 同一 bp 在 a2ts(vue) 与 AutoVM(iced) 双轨同语义；actions/menubar/toolbar 配置两轨均发射/解释（本计划补 vue 轨） |

### 4.2 L1 bind 工件形态

`auto bp add --bind <pkg>/<name>` 产出：应用侧 `src/front/bps/<name>.bind.at`
（声明式：slot→布局容器映射、action 注入引用、token 覆盖），标 GENERATED 头注；
构建期由解析器装配（VM 轨运行时解析 + a2ts 编译期内联）。不生成 bp 本体副本。
L2/L3 落地文件头注补登记行（来源 bp + 版本 + 日期），为未来回迁 L1 提供账面。

### 4.3 变体提升评审（轻量流程）

L3 产出落地后，若出现 spec 未声明的结构性差异：在 `blocks`→`blueprints` 包的
spec.md 增 `promotions:` 小节（差异描述 → slot/action 点提案），评审通过即进
spec；拒绝则该差异留应用侧并登记 DEBTS。流程写入 agent-generation-workflow.md。

### 4.4 术语更名范围（T-00 产出 manifest 后执行）

改：UI 层活跃代码标识（BlockRegistry/BlueprintRegistry、cmd_block/cmd_bp）、
CLI 面（`auto bp`，`auto block` 别名保留一版 + 弃用提示）、blocks/ 目录、
设计文档、spec 模块、goals.md GOAL-011 表述。不改：语言 AST block 语义、
历史归档 plans/design、auto-down 仓（其 PBlock 是另一概念）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | add | docs/specs/blueprint/contract.md | 无 → Blueprint 六问契约（§4.1 裁定表全文） | 中层契约首次成文 | AC-02 |
| SD-02 | modify | docs/specs/blocks/（→blueprint/ 更名）project.md | 消费=copy+生成 两通道 → L1/L2/L3 三通道分级 + 产物纪律（L1 零副本/GENERATED；L2/L3 登记行） | 平台化主通道缺失是副本漂移根因 | AC-05 |
| SD-03 | modify | docs/specs/goals.md GOAL-011 | "Blocks 一等公民生态" → "Blueprint 一等公民生态"（术语 + 三通道表述） | 全仓术语统一 | AC-01 |
| SD-04 | modify | docs/specs/autoui-skill/project.md | Block 术语 → Blueprint；补 vue 轨 actions 发射契约行 | 双形态语义闭合 | AC-04 |

## 5. 测试设计

- **重命名完整性门**：`grep -rniE '\bblock(s|registry)?\b'` 于活跃代码/文档/
  spec 白名单过滤后零命中（历史 archive、AST block 语义、auto-down 豁免清单
  由 T-00 manifest 固化）；CI/门禁脚本随更名更新。
- **跨包解析双轨门**：新增示例 app（examples/ui 内或 bps-gallery 扩展）经
  pac.at 依赖声明 import 共享 bp；VM 轨 `auto run -r vm` 渲染断言 + a2ts 构建
  vue 轨构建绿；断言点=bp 渲染结构快照（沿用现有组件快照门格式）。
- **actions vue 轨发射门**：带 actions/menubar/toolbar 的 demo app：vue 轨
  playwright DOM 断言（菜单项存在、action 触发可达）；VM 轨 vm-smoke 语义
  不回归（既有 041 范本门照跑）。
- **L1 bind 门**：bind 产物含 GENERATED 头注与声明面校验（slot/action id 与
  spec 一致性检查，缺位即构建错）；palette-drift guard 对 bp 消费语义沿用全绿。
- **回归基线**：blocks 既有 tests（342/343 谱系）、ui_gen 相关 parity、
  examples/ui 既有双端门全绿。

## 6. 验收标准

| ID | 可观察行为 | 验证方法 |
| --- | --- | --- |
| AC-01 | 活跃层术语统一 Blueprint/bp；`auto bp list/show/add/check` 可用 | grep 门（manifest 白名单）零残留 + CLI 四命令实跑 + `auto block` 别名打印弃用提示 |
| AC-02 | Blueprint 契约 spec 在案 | `docs/specs/blueprint/contract.md` 存在且六问齐备；review 记录引用 |
| AC-03 | 跨包消费双轨可用 | demo app pac.at 声明依赖 → VM 渲染断言过 + a2ts vue 构建绿（命令与期望结果写入任务证据） |
| AC-04 | vue 轨命令系统生效 | demo menubar DOM 断言过；VM 轨 vm-smoke 全绿无回归 |
| AC-05 | L1 bind 通道 v0 | `auto bp add --bind` 产出绑定工件（GENERATED 标记 + 一致性校验）；demo app 经 L1 消费零副本 |
| AC-06 | 门禁全绿 + 移交登记 | blocks 谱系 tests/parity 全绿；DEBTS 新增两行（VM 三约束调查结论指针、消费侧迁移→PLAN-070） |

## 7. 执行步骤

> 执行在 `D:/autostack/.wt/lang-639/auto-lang`（plan-639-dev 分支，Plan 529 布局）；
> 移除前过 `bash D:/autostack/wt-guard.sh`。跨仓解析序遵 AGENTS.md 红线。

- **T-00** [调查/决策工件] ①block 术语全量清单与更名 manifest（含豁免清单：
  AST block/历史归档/auto-down）；②PLAN-635 跨包机制覆盖面调查（依赖声明
  位置/解析序/缓存，能否承载 widget/bp 级 `.at`），裁定扩展或平行。
  产物：`docs/plans/attachments/639-rename-manifest.md` + 裁定小节。
  验证：manifest 覆盖 grep 全集；裁定有证据引用。依赖：无。→ AC-01/03
- **T-01** [新] `docs/specs/blueprint/contract.md`：§4.1 六问契约成文；
  `docs/specs/goals.md` GOAL-011 表述更新（SD-01/03）。依赖：T-00。→ AC-02
- **T-02** [改] 设计文档更名改写：`docs/design/blocks/` → `blueprints/`
  （git mv 保历史）；Design 17 增补三通道分级/产物纪律/变体提升评审节，
  保留归位注记与历史结论；agent-generation-workflow.md 接入提升流程。
  依赖：T-01。→ AC-02
- **T-03** [改] 代码与包库更名：`crates/auto/src/cmd_block.rs`→`cmd_bp.rs`
  （CLI `auto bp`，旧名别名+弃用提示）；`crates/auto-lang/src/ui_gen/block/`
  →`bp/`（BlockRegistry→BlueprintRegistry）；`blocks/`→`blueprints/`；
  `examples/blocks-gallery`→`bps-gallery`。按 T-00 manifest 执行，rename 后
  全量构建+既有 tests 绿。依赖：T-00。→ AC-01
- **T-04** [新] 跨包 `.at` 解析（VM+a2ts 双轨）：按 T-00 裁定扩展 635 机制
  或平行复用；pac.at 依赖声明 + 解析序（env→组内→主检出）+ 双轨消费。
  验证：AC-03 demo。依赖：T-00/T-03。→ AC-03
- **T-05** [新] a2ts 发射 `actions{}`/`menubar`/`toolbar` 到 vue 轨：
  `crates/auto-lang/src/ui_gen`（vue 发射面）+ demo app 双轨断言。
  依赖：T-04。→ AC-04
- **T-06** [新] L1 bind v0：`auto bp add --bind` 产出绑定工件 + 一致性校验 +
  GENERATED 头注；L2/L3 落地文件登记行。依赖：T-04/T-05。→ AC-05
- **T-07** [调查/决策工件] VM 三约束（回调 props 退化/子树快照不可见/
  view fn 条件不求值）：各约束最小复现 + 修复方案选项 + 工作量级估计，
  产物 `docs/plans/attachments/639-vm-constraints.md`；**不实现**。
  依赖：无（可并行）。→ AC-06（DEBTS 指针）
- **T-08** [改] 门禁收口：重命名完整性 grep 门入 CI/门禁脚本；DEBTS 两行；
  `uncompleted_plans.md`/INDEX 相关行更新。依赖：T-01..T-07。→ AC-06

## 8. 复审记录

- 2026-09-17 draft handoff：`stage: new`，PLAN-639 rev1。`outcome: pass`
  （起草授权范围内ready for review；执行未授权）。`next: work`（work 前须
  `/auto-plan:review` 通过；T-00/T-07 两调查任务为首批可执行项）。

## 9. 待澄清事项

| # | 事项 | 影响 | owner/下一步 |
| --- | --- | --- | --- |
| Q-1 | bp 包多版本/锁面机制（MVP=主检出单版本）是否满足近期消费方 | T-04 设计 | review 裁定；PLAN-070 消费前须有结论 |
| Q-2 | `auto block` 兼容别名保留期 | T-03 | review 裁定（建议一版后移除） |
| Q-3 | L1 bind 工件扩展名/位置约定（`.bind.at` vs pac.at 内联） | T-06 | T-00 调查随裁定 |
| Q-4 | examples/ui 041-auto-edit 是否在本计划内率先试运行 L1 消费（还是全部留 PLAN-070） | 范围 | review 裁定（本计划默认不含） |
