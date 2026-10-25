---
plan_id: PLAN-850
status: drafting
feature_name: Restore cross-workspace full regression ownership
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
plan_revision: 1
current_step: 0
total_steps: 5
supersedes_spec_components:
  - docs/specs/auto-ui/project.md
  - docs/specs/auto-lang/project.md
new_spec_components: []
touched_goals: []
affects: [test-tiering, auto-ui]
---

# [PLAN-850] 恢复迁仓后的跨 workspace 全量回归编排

## 0. 变更摘要

847恢复optional probe feature门控并通过独立审查及冻结裸t（3181/3181），落地主提交2ff1286c5。随后主检出batch的tf在测试执行前暴露新的跨workspace配置错误；本计划承接B847-FULL-OWNER-01，不把它混入847或降低覆盖。当前仅起草，尚未获实施授权。

## 1. 目标

让正式完整批量入口在真实语言仓/UI独立仓布局下执行原语言+UI全部测试面，保留1M churn、语料族、画廊围栏、UI生成族与既有时限、P95、资源隔离和逐ID覆盖；所有子门失败必须传播，不能零选择假绿。完成独立审查及主检出单实例全档后，才更新全局绿色锚点。

非目标：修改生产代码、把UI重新并回语言workspace、复制测试源码、删除迁出包测试、改为只跑UI日常档以替代原full、放宽/ignore/降低输入或阈值、修复本计划之外新发现的产品错误。

## 2. 架构方案

保留真实owner：语言测试在auto-lang workspace，UI编译器/方言/运行测试在auto-ui workspace。采用有界跨仓调度（优先复用已有脚本；必要时新增scripts/run-full-regression.py）协调实际Cargo manifest和原full配置，串行运行并聚合退出码。T01先实证原/新完整测试ID和配置解析，再定具体入口；禁止仅从tf删去三个UI包。独立UI的t/tu/ti/tn合同保持；是否给UI新增full入口由T01按完整选择与可维护性证据决定，不自动扩大至生产实现。

仓根解析复用env覆盖→同组兄弟→主检出规则，不建symlink/junction。所有代码/配置修改在同一lang-850专用树；如必须修改UI配置，需同组独立UI worktree、各自提交/审查/落地，不直接改主检出。

## 3. 技术栈

Cargo/nextest配置、Python标准库或仓内既有编排工具、Windows单实例回归。

## 4. 需求分析与背景调查

- 授权：847批量技能要求新红建立fix-forward，因此允许起草本合同；用户原847授权不自动扩为修改档位配置。实施待用户明确同意，不新建实现树。
- 原始失败：docs/plans/attachments/847/batch-r1/tf.log.txt，main冻结2ff1286c5，Cargo报auto-ui-compiler requires dev-dependencies and is not a member of the workspace，exit101/零测试。
- 引入：831切源abd34a355从语言workspace移出五UI包，调整t/tt/tb/tu但tf仍选external auto-ui-compiler/dialect/runtime；原.tf完整语义不能靠删除包恢复。831合同§5要求全量测试语义随owner迁移并由跨仓编排保持，不能从根别名简单删迁出包。
- 当前UI owner：D:/autostack/auto-ui aee499e87093ff727f68ac8467c506e9ecf6ecdc，.cargo/config.toml只有t/tu/ti/tn；UI日常排除ui_gen，不能冒充full。
- Specs：docs/specs/overview.md、auto-ui/project.md测试与档位、auto-ui-compiler/project.md原ID/feature/integration/单进程生成约束、auto-ui-runtime/project.md资源/测试owner、auto-lang/project.md保留原输入/时限与完整选择面。
- 原ID参考：831迁移证据与attachments/846/merge-r2/auto_ui_{compiler,dialect,runtime}-pre831-root-binary-list.txt；不能只用源码函数计数替代真实nextest/test二进制选择面。
- 847完整batch已结束：tf在编译选择前拒绝；tt3553执行=3547pass/5fail/1timeout；tb3109/3109pass。tt的6项包含4个报告写入Windows1224、1个stdout-cap控制变会话wall、1个FFI018超时；相同原测试已在847原树裸t通过，FFI018又在本轮tb1.243s通过。不得把这些当tf配置修复已覆盖或冒称tt绿；T01须区分执行状态/负载与产品缺陷，产品代码需要修复时另建明确fix-forward，不能搭便车。主原始收据attachments/847/batch-r1/receipt.json.txt。

## 5. 详细设计

T01先闭合上述批量执行状态归因，保留每条失败和重放条件；报告输出必须保护现有WIP，既存时限/分类/P95不能放宽。再输出一份可重放owner/manifest/profile/feature→实际测试ID映射，确认原tf缺失问题和其余ta/taa/t3等同形external-package隐患；本计划首先恢复当前批量tf/tt/tb合同，其他别名仅做明确范围判定，不能悄悄保留已知同形缺口或改门语义。若发现不能在原支持内保留完整面，先修订本计划，不默认缩面。

编排须逐子门验证非零且预期ID全集；真实子进程退出码、超时/中断与失败停止/收集策略必须可观察，环境/目标目录不得改变worker既有产物发现。主检出full全程单实例，work/review只运行选择清点和与编排故障相关定向控制，不能反复启动真实全档。

### 规范增量

| ID | 操作 | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD01 | modify | docs/specs/auto-ui/project.md | 现owner档位→明确跨workspace完整批量入口和UI full责任，保留旧ID/feature | 补迁仓后遗漏的可执行编排 | AC01..03 |
| SD02 | modify | docs/specs/auto-lang/project.md | 原全量/隔离纪律保持，登记完整owner调度和失败传播 | 防止仅删迁出测试假绿 | AC01..04 |

## 6. 测试设计

- 原反例：原tf完整命令必须在当前两仓布局确定性拒绝external成员；修复入口须选中语言和UI两真实owner的原完整ID，原输入/profile不变。
- 正控：每个子门执行正确且原ID全覆盖时聚合exit0；负控：丢一个UI owner、零选择、manifest缺失、子门非零/超时/中断均不得绿；不需运行全量即可用真实选择清点+有界进程故障控制验证调度器。
- ID守恒按实际编译后的完整列表和迁前/迁后映射逐项核对，新增ID单列、合法重命名须明确来源；不以总数相等代替集合。
- work/review不运行整档t/tf/tt/tb/tu/taa；合入后主检出单实例tf/tt/tb（以及实际必要UI full子门）完整运行，全部通过才推进绿色锚点。

## 7. 验收标准

| ID | owner_stage | 标准 |
|---|---|---|
| AC01 | work | 原external-member反例关闭；完整语言/UI ID与profile/feature逐项映射零丢失；无删除/ignore/降低输入时限 |
| AC02 | work | 调度正负控、零选择/缺owner/失败传播/单实例路径可观察；真实owner与目标布局正确，无symlink |
| AC03 | review | 同一专用树独立上下文复审实际代码/ID/对照/SD；merge完整门明确pending |
| AC04 | merge | 冻结必要日常门、主检出完整批量及原UI full面实际通过；若新红先归因建fix-forward，不更新绿色锚点；Specs/ledger/归档/guard完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 结果 | AC |
|---|---|---|---|---|
| T01 | work | 用户授权 | 在原新两仓布局验证owner/manifest/profile/ID全映射，冻结边界与设计 | AC01 |
| T02 | work | T01 | 专用树实施有界编排，配置入口与根解析，提交可审查修复 | AC01/02 |
| T03 | work | T02 | 逐ID清点及失败传播、零选择、路径等定向控制闭合，冻结提交 | AC01/02 |
| T04 | review | T03 | 独立审查代码/验收/SD，所有work gap关闭 | AC03 |
| T05 | merge | T04 pass | 必要日常门、落地、主检出完整批量、知识与归档guard清理；成功才更新覆盖锚点 | AC04 |

## 9. 复审记录

起草阶段：原始tf失败与831配置差异已核，未实施/测试新入口/独立审查。不得把拟议映射与控制当实测。因847 batch冻结主HEAD不变，本草案与.next-id待本轮batch结束后立即提交，未创建实现树。

## 10. 待澄清事项

需用户批准从847局部门控修复扩至本计划的跨workspace测试编排；确认前仅完成847与其batch归因收尾。所有原测试规模/预算/时限保持。
