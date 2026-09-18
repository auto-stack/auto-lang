---
plan_id: PLAN-657
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: bp-admin-sample（L1 组装样板：四包全直连 admin 示例 + 两笔语料债顺带收口）
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/blueprint/project.md#消费面（L1 组装样板 047 登记与组装结论）
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 5
---

# [PLAN-657] bp-admin-sample —— L1 组装样板（admin 示例）

## 0. 变更摘要

蓝图层的机制（639）、目录（640，现 14 包）、词面（643 chart / 649 icon）、
版本裁定（647）、L1 直连（649 连字符探测）全部就位，但**从没有两个以上的包
在同一个应用里被 L1 直连组装过**——046 fixture 只消费 login 单包，070 消费的
是 filetree 单包。Design 16 的论点 "app = shell + route→blueprint selection +
blueprint data wiring" 至今没有实证样板。

本计划交付 **`examples/ui/047-bp-admin`**：用 `navigation/sidebar-shell` +
`data-display/data-table-crud` + `form/settings` + `feedback/empty-state`
四个包（全部含连字符 key，649 修复的直接受益者）以**纯 L1 直连**（`use bps.*`
导入，无 bind、无拷贝）组装一个 admin 界面——sidebar 壳承载导航，主区按导航
状态切换 表格 CRUD / 设置 / 空态演示位，四包的 props/actions/dataSource 契约
全部由 app 侧 mock `#[api]` 供给。顺带收两笔挂账：**P645-D1**（data-table-crud
语料 v-for 缺 `:key`）与 **form-field 半句复核**（640-D89 残余最后一句）；
并对齐 **PLAN-639-D2 债行现状**（070 已归档，widgets-gallery/musk 移交
auto-down DEBTS）。

这是蓝图工作线的**收官验证**：Tier 1 扩容、vm-component-parity 等后续项都以
本样板的组装摩擦信号为输入排优先级。

## 1. 目标

- **G1 样板交付**：`examples/ui/047-bp-admin` 双端可运行（`auto run` /
  `auto run -r vm`），四包区域可见、导航切换生效。
- **G2 纯 L1 直连零副本**：app.at 仅经 `use bps.<kind>.<name>.reference.<variant>`
  导入四包（连字符 key 走 649 探测），无组件拷贝、无 bind 工件——主通道形态
  的首次多包实证。
- **G3 契约全接线**：四包声明的 props/actions/dataSource（sidebar-shell 的
  nav_tree/user/counts/sign_out；data-table-crud 的 columns/query/create/
  update/delete；settings 的 sections/load/save/reset；empty-state 的
  primary/refresh）全部由 app 侧供给/注册（mock `#[api]`）。
- **G4 两债顺带收口**：P645-D1 核销（R006 清零）；form-field 半句复核落账。
- **G5 D2 债行现状对齐**：070 归档事实 + 残余归属（widgets-gallery/musk =
  auto-down DEBTS 两行）注记进本仓债行。

### 非目标

- 不接真实后端（mock `#[api]` 即可；持久化归消费方自己的事）。
- 不新增/修改 blueprint 包的 spec 契约（语料 `:key` 是 reference 实现修整，
  不动 frontmatter 契约面）。
- 不做路由系统（view 切换用状态机；路由机制是独立命题）。
- 不做 vm-component-parity（639-D1 三约束——若组装中撞上，登记实证后仍另立）。
- 不扩 Tier 1 目录（本样板是 Tier 1 的信号源，不是它本身）。

## 2. 架构方案

**形态**：`examples/ui/047-bp-admin/`（examples/ui 下一号，046-tabs-variants
之后），结构沿 046-bp-import 先例最小化：

```
047-bp-admin/
  pac.at                  # dep bps { path: "../../../blueprints" }（647 裁定：纯 path）
  src/front/app.at        # 四包 use 直连 + nav 状态机 + mock #[api] + view 装配
```

**装配**（T-01 定细节，骨架如下）：

- `sidebar-shell`（default 变体）为壳：`nav_tree` 注入四项（数据/设置/报告/
  关于），`user` mock，`counts` 走 `#[api]`，`sign_out` 注册为 no-op toast。
- 主区按 nav 选中项状态切换：**数据** → `data-table-crud`（with_dialog 变体，
  20 行 mock 分页数据，create/update/delete 全走 `#[api]`）；**设置** →
  `settings`（sections 三段含危险区，load/save/reset mock）；**报告** →
  `empty-state` 演示位（首次进入 first_use，查询无果切 no_result，按钮触发
  `refresh` 重试）；**关于** → 简单文本页（非 bp，对照组）。
- 状态机：scoped store 或 app 内 msg/model（046/041 先例形态，T-01 定）；
  契约 Q3 纪律——选中项属 app 状态，不落 bp 私有 store。

**验证臂**：scoped Rust 测试（仿 plan649 t08-t10 双轨断言形态）+ `auto build`
全链 + autoui-verifier 双端走查 + CI（examples/ui 面已由
`build-ui-examples.yml` paths 覆盖，T-01 确认 matrix 是否需显式登记 047）。

## 3. 技术栈

- `.at` 应用语料（pac.at / app.at / `use` 跨包导入 / `#[api]` / scoped store）。
- 语料整备：`blueprints/data-display/data-table-crud/reference/{minimal,with_dialog}.at`
  补 `:key`。
- Rust：仅 scoped 测试文件（`crates/auto-lang/src/plan657_bp_admin_tests.rs` +
  lib.rs 注册，feature `ui-iced` 门控）。
- 验证：`auto build` / `auto run`（vue）/ `auto run -r vm`（VM）/ autoui-verifier
  双端 / `cargo t plan657`。门禁 Category B（crates 仅测试）；语料改动触
  vm-files 面 → review 前裸 `cargo tv`。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-19 会话：用户确认起草"L1 组装样板"（后续清单中的 ②；地基
  643/647/649 已全数落地）。仓库=auto-lang 单仓；worktree
  `D:/autostack/.wt/lang-657/auto-lang`。
- 时序：无在途依赖（649 已归档；070 已在 auto-down 归档）。

### 证据（路径实勘，2026-09-19）

- **先例**：`examples/capability-tests/046-bp-import/{pac.at, src/front/app.at}`
  ——单包（login）L1 直连 + bind 工件形态；649 的 t08-t10 端到端已证
  empty-state 等连字符包直连双轨绿（`plan649_bp_tests`）。多包组装无先例。
- **四包契约面**（spec frontmatter 实读）：
  - `navigation/sidebar-shell`：props `nav_tree, user`；actions `sign_out`；
    dataSource `counts`；variants default/compact。
  - `data-display/data-table-crud`：props `columns`；actions `create, update,
    delete`；dataSource `query`；extension_points 含 toolbar/filters/row_actions；
    variants minimal/with_dialog。
  - `form/settings`：props `sections`；actions `save, reset`；dataSource `load`；
    危险区走 alert-dialog 确认（acceptance 自述）。
  - `feedback/empty-state`：props `illustration`；actions `primary`；
    dataSource `refresh`；variants first_use/no_result/error。
- **两笔挂账**：P645-D1（`/tmp/build645.log` R006 ×2：minimal/with_dialog 的
  v-for 缺 `:key`，债面明示"随下次消费该包的计划顺带"——即本计划）；
  640-D89 残余 form-field 半句（`form-field` 无 aura 映射的回避裁定，复核
  套路同 data-table：alias/映射实态 + 语料消费面盘点）。
- **D2 现状**：auto-down `docs/plans/archived/070-jade-autoedit-bp-convergence.md`
  已归档（jade web/desktop + 041 filetree 收敛交付）；残余 = auto-down DEBTS
  两行（widgets-gallery / musk 迁移后续）+ 本仓 D2 债行的 website "blocks"
  专题页与 ui-gallery `#/blocks` 路由裁定（随残余后续）。
- **examples 编号**：examples/ui 最高 046-tabs-variants → 047 空闲；
  CI `build-ui-examples.yml` paths 过滤 `examples/ui/**` + `auto build
  --gen-only` matrix（matrix 源 T-01 确认是否显式列表）。

### 风险

- 多包同时 `use` 导入的 pac 声明门控（PLAN-635）在库形态下的 strict 语义
  已由 645 软化（告警不炸）——预期无阻断，T-03 全链构建实证。
- sidebar-shell 与主区的布局组合（sidebar 布局族 + 内容槽）此前只有单包
  内部组合，无跨包布局嵌套先例——若撞 VM 轨约束（639-D1 族），登记实证
  不在本计划内修。

## 5. 详细设计

### 5.1 装配骨架（T-01 定稿）

app.at 分层：`use` 导入区（四包五变体）→ `#[api]` mock 区（counts/query/
create/update/delete/load/save 签名对齐各包 dataSource）→ nav 状态
（model 字段 `active_nav`）→ view 装配（sidebar-shell 壳 + 主区四分支
if/else）→ actions 注册（sign_out/save/reset/primary → 状态翻转或 toast）。

### 5.2 语料与债收口

- `:key` 补齐：data-table-crud 两变体的 v-for 行与选项列表（R006 清零，
  `auto build` 告警面复验）。
- form-field 复核：grep registry/aura/语料消费三面 → 按 data-table 模板改写
  债行（预期：映射缺位或折叠键另有实态；回避是否维持按证据落笔）。
- D2 债行注记：070 归档 + 残余两行归属 auto-down DEBTS + website/ui-gallery
  路由随其后续——本仓行改为"部分收口"。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/project.md（消费面） | before：三通道描述无多包直连实态记录；after：补"L1 组装样板"小节——047 形态（四包直连/mock 契约接线/nav 状态机）、组装摩擦结论（Tier 1/parity 的排期输入） | Design 16 "app = shell + route→bp + data wiring" 首个实证成文；后续目录扩容以样板摩擦为依据 | AC-06 |

（两债核销与 D2 注记为簿记非 spec。）

## 6. 测试设计

- **scoped Rust 测试**（`plan657_bp_admin_tests.rs`，仿 plan649 t08-t10）：
  047 app.at 过 VM 管道 view 结构断言（四区域关键文案/节点）+ vue 轨 SFC
  发射断言（四包 import 与零副本负断言——app 产物无组件源码拷贝）。
- **全链构建**：`auto build`（047 目录）绿；R006 告警清零（P645-D1 复验）。
- **双端走查**：autoui-verifier 双端模式——Vue 轨 Playwright DOM 断言
  （nav 切换、dialog CRUD、设置保存、空态重试各 ≥1 断言）+ VM 轨 snapshot，
  截图存档 `tests/screenshots/`。
- **门禁**：`cargo check -p auto-lang`；`cargo t plan657`；review 前裸
  `cargo tv`（语料+测试面）；不触发 taa/ta/docs_gen。

## 7. 验收标准

- **AC-01 样板运行**：`auto run` 与 `auto run -r vm` 双端启动，sidebar 壳 +
  四个主区视图可见，nav 切换生效。验证：双端走查截图留档。
- **AC-02 纯 L1 直连**：app.at 仅 `use bps.*` 导入，无组件拷贝、无 bind；
  `auto build` 全链绿。验证：SFC 零副本负断言 + build 输出。
- **AC-03 契约全接线**：四包 props/actions/dataSource 全部由 app 供给/注册，
  CRUD/保存/重试交互可走通（双端各 ≥4 断言）。验证：走查记录 + 测试。
- **AC-04 两债收口**：P645-D1 核销（R006 清零复验）；form-field 半句复核
  落账（DEBT 行改写）。验证：DEBT diff + build 告警面。
- **AC-05 D2 债行对齐**：070 归档事实与残余归属注记进债行。验证：DEBT diff。
- **AC-06 spec 沉淀**：SD-01 落地 + 组装摩擦结论记入（Tier 1/parity 排期
  输入）；merge 时 specs.json upsert + spec-index。
- **AC-07 scoped 测试与 CI**：`cargo t plan657` 绿；047 入 examples CI
  覆盖面（paths 已含；matrix 若显式则登记）。

## 8. 执行步骤

> worktree `D:/autostack/.wt/lang-657/auto-lang`（分支 `plan-657-dev`）；
> plan 簿记留主检出。

- **T-01 [有界调查] 装配形态定案**（0.5d）
  无依赖。操作：①examples/ui 编号与 CI matrix 形态确认（显式列表则 047 登记
  位）；②nav 状态机选型（app model vs scoped store，046/041 先例对齐）；
  ③`#[api]` mock 挂载形态（046/041 先例）；④四包 use 路径与变体选定清单；
  ⑤sidebar 布局族与主区组合的 widget 面预检（palette 交集无缺口）。产出：
  §5.1 装配草图定稿。验证：结论记入本节。→ AC-01/02/03 前置
- **T-02 语料债先行**：data-table-crud 两变体 `:key` 补齐；form-field
  三面复核（registry/aura/语料）。验证：`auto build`（046 fixture 全链）
  R006 清零；复核结论记入。→ AC-04
- **T-03 样板主体**：`examples/ui/047-bp-admin/{pac.at, src/front/app.at}`
  全量落笔（四包直连 + mock + 状态机 + view 装配）。验证：`auto build` 绿；
  双端启动冒烟。→ AC-01/02/03
- **T-04 测试面**：`plan657_bp_admin_tests.rs` + lib.rs 注册 + CI 登记（如需）。
  验证：`cargo check -p auto-lang` 零警告；`cargo t plan657` 绿。→ AC-07
- **T-05 双端走查与收口（review 前）**：autoui-verifier 双端交互断言 + 截图
  留档；P645-D1/form-field/D2 三债行落笔；裸 `cargo tv`；组装摩擦结论
  汇总（§SD-01 素材）。验证：输出留档本节。→ AC-01/04/05/06 及全 AC 兜底

依赖链：T-01 → T-03；T-02 独立可先行；T-03 → T-04 → T-05。

## 9. 复审记录

- 2026-09-19 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（三项，均有默认，不阻塞开工）。

## 10. 待澄清事项

1. **nav 状态机形态**（T-01 ②）：默认 app model 字段（最简、契约 Q3 纪律
   最清晰）；如倾向 scoped store（跨视图保持），T-01 内改选即可。
2. **empty-state 演示位的交互深度**：默认"报告"页做 first_use→no_result→
   refresh 重试的最小闭环；如希望演示 error 变体（三变体全展），T-03 加一
   个触发按钮，成本极小。
3. **（记录性）** 样板对 Tier 1 的信号收集口径：以 T-05 摩擦清单为准
   （缺的原语/难接的契约/双端差异），不预先设问卷。
