---
plan_id: PLAN-522
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: helper-fn-into-vue-sfc（模块级 helper fn 进 vue SFC）
author: [zhaopuming]
created_at: 2026-09-02
updated_at: 2026-09-02

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui_gen]   # vue 生成器 + 可能的 use_scanner 消费接线
current_step: 0
total_steps: 6
---

# [PLAN-522] helper-fn-into-vue-sfc：模块级 helper fn 发射进 vue SFC

> **来源**：Plan 448 §7 H.3（裁决划出独立立项）——H1/H2（computed 块体
> 双端修复）已合并 master（merge `ec9c16fcd`），本计划是解锁其语料价值
> （016/024/025 的 handler 内重复重算消除）的最后一环。
> **纲领引用**：无（独立特性计划，非 401 示例升级子计划——但与并行
> 519/520/521 示例升级线存在语料协调点，见 §风险）。

## 变更摘要

`use` 导入的模块级 helper fn（如 016 的 `calendar_util.at`
`build_month_grid`、024 donut 页的 `dc/ds`）目前**不发射进生成的 vue
SFC**——computed/handler 体内调用它们在 Vue 侧无函数可调（vue-tsc
TS2304），迫使用户把派生逻辑内联重算进每个 handler（437 §0.6.E-3
记录的缺口，donut 页被迫改直调 `math.cos` 绕过）。VM 侧无此问题
（handler 合成把 import_stmts 编进同一 VM 模块，PLAN-051 C3 的
import_aliases 补了裸名解析）。本计划补齐 Vue 侧：按需把被引用的
helper fn 转译为 TS 函数发射进消费方 SFC。

## 目标

1. widget 的 computed/handler 体内调用 `use` 导入的模块级 fn 时，该 fn
   （及其依赖闭包）以 TS 函数形态出现在生成的 SFC script 中，vue-tsc
   零 TS2304。
2. 表达式/块体 computed 均可调用 helper fn（与 Plan 448 H1/H2 的产出
   组合，派生态不再需要 handler 内联重算）。
3. 语料验收：016-calendar 四 handler 的 `month_label/days` 重算链改为
   computed（消 ×4 重复）；024 donut 页恢复 helper 形态（437 被迫
   绕过点回正，视转译覆盖面裁定）。

## 架构方案

**方向 A（推荐先行）：按需发射进消费方 SFC。**
widget 生成期收集 computed/handler 体引用的 use 导入符号
（use_scanner 扫 use 语句 → 裸名集合；对 import_stmts 的 `Stmt::Fn`
池按名拉取），经 ts_adapter 转译为 TS 函数（复用 store composable
路径的既有转译层），发射进 SFC script 尾部。要点：
- **依赖闭包**：被拉 fn 体内引用的其他导入 fn 一并拉入（迭代到
  不动点）；去重（同名只发一份）。
- **命名**：保持裸名（调用点零改写）；与 state/computed/emit 名冲突
  时编译期诊断（v1 边界：冲突即警告 + 跳过发射，提示用 ext_imports）。
- **与 ext_imports（Plan 051 手写 TS 逃逸口）优先级**：ext_imports
  已声明同名符号时不发射（手写优先）。

**方向 B（演进预留）：共享 utils 模块文件**（生成
`src/front/utils/<module>.ts`，SFC import）——跨 SFC 去重干净，但
类型导出/跨文件依赖/HMR 面更大；规模超阈值（同一 fn 被 ≥2 个 SFC
引用）再演进。

## 需求分析与背景调查

（调研于 2026-09-02，立项时点实证）

- **缺口实证**：437 归档 §0.6.E-3——"页面级模块 fn 不发射进 SFC，
  dc/ds 生成物中无定义（vue-tsc 报 TS2304）；此前 dist 正常系陈旧
  产物"，被迫改直调 `math.cos/sin`。025 头注自认"几何/排序为 handler
  内联重算（437 §0.6.E-3），与 024 同款已知重复模式"。
- **转译件已存在**：ts_adapter 的 `transpile_stmt`/语句体转译在 store
  composable 路径（`generate_store_composable*`）已在用——fn 体 → TS
  的能力不是从零建。
- **VM 侧已对齐**：handler 合成把 import_stmts 编进同一 VM 模块
  （`synthesize_widget_module`），PLAN-051 C3 的 import_aliases（裸名
  → 模块限定名）+ `call_vm_fn` 补了 computed 求值面的裸 fn 调用。
  本计划完成后双端语义对齐。
- **既有管道**：`use_scanner::scan_use_statements`（api.rs:385 已在
  用）；vue.rs 有 `project_api_functions`/`ext_imports` 机制。
- **语料规模**：016（calendar_util 4 fn）、024（四图表组件页级 fn
  族）、025（几何 6 串 + 排序）、018（calendar/book 等 store 之外的
  纯算术 helper）——首批受益面。

## 详细设计

（执行期按方向 A 细化；以下为既定要点）

1. **符号收集**：vue.rs 生成 script 时扫描 computed 表达式 + handler
   bodies 里的 `Expr::Call` 裸名，交 use_scanner 结果过滤出导入符号。
2. **fn 拉取与闭包**：import_stmts 池按裸名匹配（Plan 339 6b 的
   bare→qualified 规则同源）；体内引用迭代收敛。
3. **TS 发射**：复用 store composable 的 fn 转译（参数/返回/async
   判定同规则）；发射位置 = script 尾部、defineEmits 之后。
4. **转译覆盖面边界（v1 明示）**：纯算术/字符串/列表 fn 支持；rc/堆
   语义、泛型、模式匹配 fn 不保证——转译失败时回退现状（不发射 +
   R013 警告），不阻塞生成。
5. **去重与优先级**：SFC 内去重；ext_imports 同名优先；store 方法名
   不冲突（store fn 走 composable 文件）。

## 测试设计

- vue 单测：use 导入 fn 被 computed（表达式 + 块体）调用 → SFC 含
  转译 fn + 调用点零改写；依赖闭包（A 引 B）双发；ext_imports 同名
  抑制；未被引用的导入 fn 不发射。
- 转译边界：模式匹配 fn → 警告 + 不发射（现状回退）。
- 双端一致性：016 迁移后 `auto run`（Vue）与 `auto run -r vm` 派生
  值一致（MCP snapshot 对拍 month_label/days）。
- 回归：gallery_golden / vue_capabilities / docs_gen 基线不动（无
  use-helper 语料变化时不重生成）。

## 验收标准

1. 016-calendar：`month_label`/`days` 改 computed（调用
   build_month_grid/month_name），四 handler 的重算行删除；Vue 构建
   vue-tsc 零错、VM 模式三难度档 + Today/翻月行为与迁移前一致。
2. 新增 vue 单测全绿；全量套件与 master 基线零新增失败。
3. （尽力项）024 donut 页 dc/ds helper 形态恢复——若转译覆盖面不足，
   记边界留证据。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. T1：符号收集 + fn 拉取闭包（vue.rs + use_scanner 接线）→ 单测：
   被引 fn 发射/未引不发射/闭包双发。
2. T2：TS 发射 + 去重/ext_imports 优先级/转译失败回退 → 单测三态。
3. T3：016 语料迁移（computed 化）+ 双端对拍（Vue 构建 + VM MCP
   snapshot 三档棋盘 81/256/480 与 month_label）。
4. T4：（尽力）024 donut helper 恢复或边界记录。
5. T5：全量回归（默认 lib/ui-iced/gallery_golden/vue_capabilities/
   docs_gen）+ master 基线对照。
6. T6：复审（验收清单逐条 + 遗留扫描 + spec-impact 元数据）。

## 风险

- **与并行 519/520/521 示例升级线的语料协调**：本计划动 codegen +
  016/024 语料；016 的 app.at 近期被 064115a76（浅色默认+设置弹层）
  动过——合并时注意 rebase 顺序，其余文件无交集。
- **fn 体 TS 转译覆盖面**：rc/堆语义在 TS 无对应、泛型/模式匹配
  复杂形态——v1 明示边界（失败回退现状 + R013 警告，不阻塞生成），
  不追求全量覆盖。
- **SFC 内命名冲突**：helper fn 与 state/computed/handler 同名——v1
  冲突即警告跳发（ext_imports 手写口兜底）。
- **产物膨胀**：按需拉取（未引用不发射）控制面；同一 fn 被 ≥2 SFC
  引用时产物重复——阈值触发再演进方向 B（共享 utils 文件）。

## 复审记录
