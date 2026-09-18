---
plan_id: PLAN-645
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: toolchain-parity-debt（bps fn 转译缺口 + with_defaults 扫描根）
author: [zhaopuming]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/blueprint/contract.md（增补：bps 扫描 fn 转译契约行 + 扫描根解析序行）
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（parity 三层机制的 L2 前置）", "GOAL-011: Blueprint 一等公民生态"]

affects: [blueprint, auto-gen]
current_step: 0
total_steps: 4
---

# [PLAN-645] toolchain-parity-debt——bps fn 转译缺口与扫描根修正

> 背景设计：docs/design/30-autoui-parity-three-layer.md（三层 parity 机制）。
> 本计划偿还其中标记的两笔 auto-lang 工具链债，是三层机制 **Layer 2
> （Blueprint gallery parity）的第零任务**。

## 0. 变更摘要

偿还 DEBTS 070 登记的两笔工具链限制（均为 PLAN-070 执行期实勘发现）：

1. **vue 轨 bps 扫描不转译跨文件 fn 导入**：bp reference 含
   `use <mod>: fn`（bare 或 bps 限定）时，生成的 SFC 只有调用无定义
   （`Cannot find name 'flatten_tree'`，046-bp-import 构建断裂复现）——
   组合形态 bp 无法上 vue 轨。修复 = bps 扫描发射路径补 plan522 式
   helper 转译（按 base_dir/parent 解析模块源并内联被引符号）。
2. **BlueprintRegistry::with_defaults 扫描根为编译期 CARGO_MANIFEST_DIR**：
   `auto bp list/show/add/check` 恒扫构建机主检出 `<repo>/blueprints`，
   worktree/外部检出内的包不可见。修复 = 扫描根改为运行时解析
   （git toplevel 优先，`AUTO_BLUEPRINTS_ROOT` env 覆盖，回落编译期值）。

## 1. 目标

1. 组合形态 bp 在 vue 轨可构建：bp reference 的跨文件 fn 依赖被内联转译，
   生成的 SFC 自包含（filetree 组合形态回归启用即本目标的验收样本）。
2. `auto bp` 命令族在 worktree/任意检出内可见本仓 blueprints 包；
   `AUTO_BLUEPRINTS_ROOT` 可覆盖。
3. 既有面零回归：app 壳 ui_config 合成（PLAN-639 T-05）、046 语料、
   jade front/auto 构建。

**非目标**：VM 轨增量（本计划纯 vue/CLI 面）；bp 组合语义变更；多版本包。

## 2. 架构方案

- **fn 转译**：bps 扫描点（ui_gen/vue.rs 的 bp reference 构建路径）接入
  plan522 的 helper 转译臂——解析 `use` 语句目标模块源码，将被引用符号的
  fn 体按依赖序内联进 SFC script（与 `use { fn: ... from "*.ts" }` 的 ext
  内联同层，区别在源是 .at 模块）。解析复用 resolve_module_path
  （base_dir→parent→deps 探测，PLAN-635 门控沿用）。
- **扫描根**：with_defaults 改为 `AUTO_BLUEPRINTS_ROOT` env →
  cwd 向上找 pac.at/blueprints 的 git toplevel → 编译期值兜底（三级解析，
  与 AGENTS 跨仓解析序同构）。
- 回归护栏：046-bp-import（app 壳合成路径）+ filetree 组合形态启用样本
  （本计划新增，恢复自 af8c72a84 的 reference/default.at 并适配转译语义）
  + jade front/auto 构建。

## 3. 需求分析与背景调查

**授权记录**：2026-09-18 用户裁定（Q-8 后续：三层 parity 机制的 L2 前置
在本计划偿还，不另立计划）。仅起草；执行未授权。预算未指定。

**证据**：DEBTS 070 两行；PLAN-070 T-01/T-03 勾记（046 断裂复现、
with_defaults 实勘）；PLAN-522（use fn 双端同源转译先例）、PLAN-635
（声明门控）。

## 4. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/blueprint/contract.md | L1/组合形态"暂缓"节 → 组合形态启用；增补"bps 扫描 fn 转译"契约行（跨文件 fn 依赖内联，GENERATED 语义不变） | L2 parity 的前置能力 | AC-01 |
| SD-02 | modify | docs/specs/blueprint/contract.md（或 project.md） | 扫描根=编译期常量 → env/toplevel 运行时解析三级序 | worktree/外部检出包可见性 | AC-02 |

## 5. 测试设计

- 新增组件级测试（plan639_bp_tests 谱系）：bp reference 跨文件 fn 导入 →
  SFC 含内联 fn 体（正）；未导入符号仍报错（负）。
- with_defaults 三级序测试（env 设置/未设置 × git toplevel/编译期值）。
- 046-bp-import 全量构建回归；filetree 组合形态样本构建回归。
- jade front/auto `pnpm build` 回归（消费方零影响）。

## 6. 验收标准

| ID | 可观察行为 | 验证方法 |
| --- | --- | --- |
| AC-01 | 组合形态 bp（含跨文件 fn 依赖）vue 轨构建绿，SFC 自包含 | filetree 组合形态回归样本 `auto build` + vue-tsc 零该类错误；DEBTS 070 第二行销号 |
| AC-02 | `auto bp list` 在 worktree 内可见本仓包；env 覆盖生效 | worktree 内 `auto bp list` 含 navigation/filetree；`AUTO_BLUEPRINTS_ROOT` 指向他处时列举跟随 |
| AC-03 | 零回归：046 + jade front/auto + 既有 plan639 测试 | 三面全绿 |

## 7. 执行步骤

> 执行 worktree：`D:/autostack/.wt/lang-645/auto-lang`（plan-645-dev）。
> 注意：jade 侧消费样本在 auto-down 仓（组 worktree
> `.wt/lang-645/{auto-lang,auto-down}` 或临时借用 jade front 构建面）。

- **T-01** [改] with_defaults 三级解析根 + 测试。依赖：无。→ AC-02
- **T-02** [改] bps 扫描 fn 转译臂 + 正/负测试。依赖：无（可与 T-01 并行）。→ AC-01
- **T-03** [新] filetree 组合形态回归样本（恢复 reference/default.at，
  适配转译语义；`use` 形态按 T-02 实现定 bare/bps 限定）。依赖：T-02。→ AC-01
- **T-04** [改] 回归三面（046/jade/既有测试）+ DEBTS 两行销号 +
  contract.md SD 落笔。依赖：T-01..T-03。→ AC-03

## 8. 复审记录

- 2026-09-18 draft handoff：`stage: new | plan_id: PLAN-645 | plan_revision: 1 |
  outcome: pass（起草完成；执行未授权） | next: review → work`。

## 9. 待澄清事项

| # | 事项 | 影响 | owner/下一步 |
| --- | --- | --- | --- |
| Q-1 | fn 内联的体积上限（超大支撑件全量内联 vs 仅被引符号闭包） | T-02 实现细节 | T-02 实测裁定（默认：仅被引符号闭包） |
| Q-2 | filetree 组合形态启用后是否回填 jade（web file_tree 谱系仍独立，Q-7=A 不变） | 范围 | 维持 Q-7=A；组合形态仅作为 bp 能力样本与未来消费方基建 |
