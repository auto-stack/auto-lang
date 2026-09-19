---
plan_id: PLAN-657
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: bp-admin-sample（L1 组装样板：四包全直连 admin 示例 + 两笔语料债顺带收口）
author: [agent]
created_at: 2026-09-19
updated_at: 2026-09-19

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/blueprint/project.md#消费面（L1 组装样板 047 登记与组装结论）
touched_goals: [GOAL-011, GOAL-010]  # 复审修正（原空）：L1 主通道多包实证=GOAL-011 延续；047 入 examples/ui 矩阵=GOAL-010

affects: [blueprint]          # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 5
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

### 5.1 装配骨架（T-01 已定稿，2026-09-19 实勘）

**形态**：

```
047-bp-admin/
  pac.at               # scene ui / render vue / api "rust" / dep bps { path: "../../../blueprints" }
  src/back/api.at      # mock #[api]：counts/query/create/update/delete/load_config/save_config
  src/back/db.at       # 内存表数据（种子行 + CRUD 变更），013-todo 先例形态
  src/front/app.at     # use 四包五变体直连 + App 壳（nav 状态机 + 主区四分支）
```

**T-01 五项结论**：

1. **编号/CI**：examples/ui 最高 046-tabs-variants → 047 空闲；
   `build-ui-examples.yml` matrix 为**显式列表** → T-04 登记 047。附带发现：
   matrix 有 4 条死目录项（026-keyboard-mouse-events/027-native-css/
   028-dom-escape/029-external-imports——cfc1bfe23 重组迁往 capability-tests
   后未跟，预存红）；T-04 顺带摘除死项（CI-only 卫生修复，capability-tests
   覆盖面缺口另行记账）。
2. **nav 状态机**：app model 字段 `active_nav str = "data"`（§10.1 默认裁定；
   契约 Q3——选中项属 app 状态）。SidebarShell 上抛 `on_nav(id)` → app 切
   主区；shell 内部 active_nav 为 bp 私有选中态。
3. **mock 形态**：013-todo 先例——pac `api: "rust"` + `src/back/api.at`
   （`#[api]` 注解直调 db.at 内存数据）+ front `use back.api:` 直调；
   VM 轨解释器进程内解析 back/api.at（api.rs:574 注记），vue 走 `@/lib/api`
   客户端——双轨同源。
4. **四包五变体清单**：`bps.navigation.sidebar_shell.reference.default:
   SidebarShell`、`bps.data_display.data_table_crud.reference.with_dialog:
   DataTableCrudDialog`、`bps.form.settings.reference.default: SettingsScreen`、
   `bps.feedback.empty_state.reference.first_use: EmptyStateFirstUse`、
   `...no_result: EmptyStateNoResult`（error 变体不做，§10.2 默认）。
5. **核心发现——reference 未参数化（本计划最大组装摩擦）**：四包 reference
   均为无参自含脚手架（spec props 以注释标注），而语言对未声明 props 报
   compile_error（rust.rs:8520）。官方集 14 包仅 filetree 有参数化先例
   （`FileTree(nodes: List, ...)` + Init 播种 + `.prop` 命名空间，PLAN-536）。
   **裁定**：G3 的唯一可行接线 = 被消费的 5 个 reference 变体做参数化修整，
   对齐**各自 frontmatter 已声明的契约面**（props 数据参数 + Q2 action 点的
   msg 回调参数（k2/k3 契约：`on_x: msg` + 匹配变体 + `on_x()` 上抛）+
   dataSource 的 L1 物化参数（counts/rows/config——L1 无 bp→app 逆向调用
   通道，app 经 `#[api]` 取数后 props 回填，记摩擦信号））。**frontmatter
   契约面零改动**（非目标边界内——"reference 实现修整"类）；minimal 变体
   仅补 `:key` 不参数化（未被消费；变体间参数面一致性缺口记 Tier 1 信号）。
   sidebar content 区改 `slot` 出口**带 fallback**（保 plan649 t09 裸实例化
   文案绿）；t09 sidebar 用例 fixture 最小更新为传参实例化（意图不变：
   kebab 直连可编译可渲染）。
   - palette 预检：所需 widget 族（sidebar/header/avatar/dropdown-menu/
     badge/table/dialog/pagination/alert-dialog/switch/tabs/form/label/
     select/callout/image/separator）全部在 WidgetRegistry 注册面内；
     dialog/pagination/alert-dialog/switch 的 **VM 行为面** T-03 双端实证，
     撞 639-D1 族约束则登记不修（§4 风险既定）。

**app.at 分层**：`use` 导入区（四包五变体）→ App model（`active_nav` +
nav_tree/user/counts/columns/rows/total/sections 数据面，Init 经 `use back.api`
取数播种）→ view 装配（`SidebarShell(nav_tree, user, counts, on_nav, on_sign_out)`
壳，主区四分支 if/else 经 content slot 注入：data→DataTableCrudDialog（受控
rows/total + on_query/on_create/on_update/on_delete 回调→app 调 api 刷新）、
settings→SettingsScreen（sections + on_save/on_reset；config 经 Init 播种）、
reports→EmptyStateFirstUse/NoResult（app 状态机 first_use→no_result，
on_primary→refresh 重试闭环）、about→纯文本对照页（非 bp））→ actions
（sign_out→no-op toast（ui_note）；save/reset/primary→状态翻转）。

### 5.2 语料与债收口

- `:key` 补齐：data-table-crud 两变体的 v-for 行与选项列表（R006 清零，
  `auto build` 告警面复验）。
- form-field 复核：grep registry/aura/语料消费三面 → 按 data-table 模板改写
  债行（预期：映射缺位或折叠键另有实态；回避是否维持按证据落笔）。
- D2 债行注记：070 归档 + 残余两行归属 auto-down DEBTS + website/ui-gallery
  路由随其后续——本仓行改为"部分收口"。

### 5.3 组装摩擦结论（T-05 汇总，SD-01 素材 / Tier 1 与 vm-component-parity 排期输入）

Design 16 论点 "app = shell + route→blueprint selection + blueprint data wiring"
首次多包实证成立（vue 轨全链绿：nav 切换/CRUD 建改查/设置保存/空态三态
回环/对照页，9 张截图留档）。摩擦清单按信号强度排序：

1. **[P0·引擎] VM 轨跨 widget 回调载荷字面量化（P657-D1）**：vue 全绿、
   VM 回调链触发但载荷坏（`active_nav: "id"`、`created: ` 空名）。639-D1
   约束一族的新形态——**vm-component-parity 的第一优先实证**（没有它，
   任何"壳+路由+数据接线"样板只有单端完整性）。
2. **[P1·词汇面] reference 参数化缺位（T-01 核心发现）**：官方集 14 包
   仅 filetree 有参数化先例；spec frontmatter 的 props/actions/dataSource
   契约与 reference 实现是"两张皮"。Tier 1 扩容前置项：**参数化规范**
   （props→widget 参数、actions→`on_*: msg` 回调（k2/k3 契约）、
   dataSource→无逆向通道时 props 回填物化）成文进 contract.md 或
   reference 书写规约，避免每个消费计划重付参数化成本。
3. **[P1·方言] a2r back 转译窄面（P657-D3）**：f-string `$` 前缀/全局 str
   无参数写通道/`[0]`-while-单行 fn 无先验/str 参数 ctor 仅同名转换——
   agent 写 mock back 需要一份"方言速查"，或转译器收口转换。
4. **[P2·基建] examples npm 阶段双预存红（P657-D2）**：auto-sources 的
   run-写/build-读 phase-ordering + main.ts env types——master 全例基线
   红，修在 auto-man 生成器/脚手架面。
5. **[P2·词汇面] 结构体契约无跨包类型通道**：User/Config/Row 等契约结构
   无法作为 widget 参数类型跨 use 传递（front 语料无 map 字段点读先例）
   ——本样板全部退化为平铺原子参数/标量端点（user_name/user_email 三参、
   cfg 三读端点）。Tier 1 信号：跨包类型共享或 map 安全访问面。
6. **[P3·表现] view 发射小摩擦（P657-D5）**：div-in-table 嵌套警告/
   程序化 fill 后受控 input 显示回退/avatar alt 插值发射空。
7. **[P3·CI] capability-tests 生成面覆盖缺口（P657-D4）**+ matrix 死项
   （已摘）。

**Tier 1 排期建议**（输入，不裁）：②参数化规范与 ①vm-component-parity
并行为最高优先；③⑤随后；④⑥⑦搭车。

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

- **T-01 [x] [✅ 已完成] 装配形态定案（0.5d）**（2026-09-19 work 会话）
  无依赖。操作：①examples/ui 编号与 CI matrix 形态确认（显式列表则 047 登记
  位）；②nav 状态机选型（app model vs scoped store，046/041 先例对齐）；
  ③`#[api]` mock 挂载形态（046/041 先例）；④四包 use 路径与变体选定清单；
  ⑤sidebar 布局族与主区组合的 widget 面预检（palette 交集无缺口）。产出：
  §5.1 装配草图定稿。验证：结论记入本节。→ AC-01/02/03 前置
  **[✅ 证据：五项结论全部落 §5.1——047 空闲+matrix 显式（4 死目录项附带
  发现）；nav=app model（§10.1 默认）；mock=013-todo 形态（pac api:"rust"+
  back/api.at+db.at，VM 轨进程内解析实证 api.rs:574）；五变体清单定稿；
  核心发现=reference 未参数化→裁定被消费 5 变体参数化修整对齐自家
  frontmatter（filetree 先例/k2-k3 msg 回调契约/slot fallback 保 t09），
  frontmatter 零改动。]**
- **T-02 [x] [✅ 已完成] 语料债先行**（2026-09-19 work 会话）
  - `:key`：minimal/with_dialog 两变体 v-for 行补 `row (key: row.id)`（035-vfor-key/
    026-database `table-row (key: i)` 先例形态）。
  - form-field 三面复核：①registry——`FormField` 注册+alias `form-field` 在案
    （registry.rs:655-659）；②schema——`element form_field` 带 vue 映射
    `{ component: "FormField", import: "@/components/ui/form" }`（schema/aura.at:4341，
    datatable 同款折叠键形态）+ `iced: "none"`；③语料——blueprints/** 零提及。
    结论：债行两说（未注册/缺映射）皆不成立；回避真实理由=语料零消费+VM 轨
    iced:none（消费时需评估双端）。债行改写落 T-05。
  - 验证：`auto build`（046 fixture）**R006=0 清零**（/tmp/build657-t02b.log，
    worktree 自建 auto.exe）。附带归因：046 npm 阶段 5 个 vue-tsc 错误为
    **master 预存**（stash 对照两组错误集逐字一致 + 013-todo 基线亦红 3 个：
    auto-sources TS2307/main.ts env TS2339/pagination itemsPerPage——共享
    脚手架/发射面预存债，记 T-05 摩擦清单，不在本计划修；pagination
    itemsPerPage 属 data-table-crud 语料面，随 T-03 参数化顺带补
    `per_page`）。→ AC-04（债行落笔在 T-05）
- **T-03 [x] [✅ 已完成] 样板主体**（2026-09-19 work 会话）
  - 5 个被消费 reference 参数化（§5.1 裁定；frontmatter 零改动）+ minimal
    顺带 per_page；047 四文件全量落笔（pac/back api+db/front app）。
  - 验证：`auto build --gen-only` 绿（24 组件，R006=0）；全量 build 的
    vue-tsc 面 = 仅 2 个预存共享基建错（auto-select/overlay.ts TS2307 +
    main.ts TS2339——master 全例基线，013/046 同款；stash 对照归因）；
    back rust 服务编译通过并监听 8080（vite 3000 联动，`auto run` 实跑）；
    VM 轨 `auto run -r vm` 窗口起 + MCP 首状态同步（40s timeout 实证）。
  - 方言适配记录（back 转译窄面，均入摩擦清单）：f-string 须 `f"${x}"`
    （裸 `{x}` 降字面量）；全局 str 无参数写入通道（&str 借用逃逸）→
    配置改单 List<Row> + for-in 读/f-string 包裹强制 format! 产 String；
    `[0]`/while/单行 fn 无 back 语料先例。→ AC-01/02/03（VM/vue 断言进 T-04 测试）
- **T-04 [x] [✅ 已完成] 测试面**（2026-09-19 work 会话）
  - `plan657_bp_admin_tests.rs`（三锚：VM 全量四区域文案；vue 四包 import
    + on_* 回调绑定 + 零副本负断言；五 reference SFC 生成 + palette 零漂移
    + 047 dep 探测）+ lib.rs 注册。
  - plan649 fixture 适配：e2e_host 增传参实例化面；t09 sidebar 用例随参数化
    改传 nav_tree（意图不变：kebab 直连可编译可渲染）；**两处收集器补
    `View::Container` 遍历臂**——header 语义容器文案首次入断言面（master
    预存盲区：t09 旧断言从未覆盖 header 区，Acme 由此转绿）。
  - CI：matrix 登记 047-bp-admin（--gen-only 实证 EXIT=0）+ 摘除 4 条
    cfc1bfe23 重组死目录项（026-029）。
  - 验证：`cargo t plan657` 3/3 绿；`cargo t plan649` 10/10 绿；
    `cargo check -p auto-lang` 本计划文件零新增警告。→ AC-07
- **T-05 [x] [✅ 已完成] 双端走查与收口（review 前）**（2026-09-19 work 会话）
  - **vue 轨全链走查绿**：nav 四页切换 + CRUD（dialog 建+search 过滤重查）+
    设置（编辑+保存 toast）+ 空态三态回环（first_use→no_result→retry）+
    about 对照页——30 步动作零报错，9 截图留档
    `examples/ui/047-bp-admin/tests/screenshots/047_vue_*.png`（AI 视觉
    核验：nav badge=counts API 实值 20/3、CRUD toast、过滤单行、settings
    toast/sections/danger 区全部确认）。
  - **VM 轨走查**：四区域渲染 + dialog 交互 + 回调链触发 7/8 accounted
    （`vm_walkthrough_657.json`）+ 6 截图；**核心发现 P657-D1**：跨 widget
    回调载荷字面量化（`active_nav: "id"` state 探针实证，vue 绿）——639-D1
    族第四实证，vm-component-parity 第一优先输入。
  - **三债行落笔**：P645-D1 ✅ 核销；640 行 form-field 半句 ✅ 三面复核
    核销（三半句全清）；639-D2 部分收口注记（070 归档+残余归 auto-down）；
    新增 P657-D1..D5（2026-09-19 增补节）。
  - **裸 `cargo tv`**：2710/2712 过，2 红 = PLAN-656 在案 master 预存
    （mouse_area/display_family，非本计划域）。
  - **摩擦结论**：§5.3 汇总（7 项分级 + Tier 1 排期建议）——SD-01 素材。
  - 顺带修整：with_dialog 表头硬编码 Actions 删除（columns 供给）。→ AC-01/04/05/06 及全 AC 兜底

依赖链：T-01 → T-03；T-02 独立可先行；T-03 → T-04 → T-05。

## 9. 复审记录

- 2026-09-19 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（三项，均有默认，不阻塞开工）。
- 2026-09-19 work 收官（/auto-plan:work）：
  `stage: work | PLAN-657 | rev 1（含 T-01 有界调查内的语义修订——§5.1
  定稿 + §5.3 摩擦结论新增） | pass（execution_done） | code_commit:
  plan-657-dev @3e190da3a/ad2f99922/1f109f454（基线 b4b04c5cd，worktree
  D:/autostack/.wt/lang-657/auto-lang，依赖 auto-down @a615d69 detached） |
  task_ids: T-01..T-05 全勾 | evidence: cargo t plan657 3/3 + plan649
  10/10 + cargo check 零本计划警告 + 裸 cargo tv 2710/2712（2 红=PLAN-656
  在案 master 预存）+ auto build gen-only 绿（24 组件 R006=0）+ back 服务
  实跑（8080）+ 双端走查（vue 30 步全链 9 截图 AI 核验 / VM 7/8 accounted
  6 截图 + vm_walkthrough_657.json） | blockers: 无 | next: review（/auto-plan:review）`
  - AC 自检：AC-01/02/03/04/05/07 已证（AC-01/03 的 VM 侧 nav 切换受
    P657-D1 限制——§4 风险既定的"登记不修"路径，vue 侧全量成立）；
    AC-06 素材就绪（§5.3），SD-01 落地归 merge。
  - 预检披露：主检出一处他源 WIP（examples/rust-workspace/015-notes/…
    + Cargo.toml，非本计划路径）——未触碰未收纳，需其属主自行路由。

- 2026-09-19 独立复审（/auto-plan:review）：
  `stage: review | PLAN-657 | plan_revision 1 | **pass** | reviewed_commit:
  1f109f454b636fbbd3ca7c1fed6f5a3088366c49 | base: b4b04c5cd618b0a0dda652f5fa6de44377046ab6
  | dep: auto-down a615d693be9aca82922633858709856e196f4c5f (detached) |
  spec_inputs: docs/specs/blueprint/{project.md,contract.md} 现行版 |
  acceptance: AC-01 partial·vue 全量+VM 渲染/交互受 P657-D1 限（§4 风险条款
  预授权"登记不修"，非未验证）；AC-02/04/05/07 pass；AC-03 pass·vue ≥4 断言
  全链+VM 同 P657-D1 限；AC-06 pass（delta 已验，落地归 merge） |
  findings: F-01[P3 已修] touched_goals 空值→[GOAL-011, GOAL-010]（复审终局化）；
  F-02[P3 记录] rust-workspace/Cargo.toml 提交含 013-todo-back/012-clock 工具
  副作用（基线对照跑 13 所致，工具自管清单，无害）；F-03[P3 操作性] worktree
  含工具物化 junction（deps/bps）+pnpm 链接场——wt-guard 判非清洁，merge
  清理须按守门程序先 rmdir 仅链接；F-04[P2 已债] P657-D1 VM 回调载荷；
  F-05[P3 观测] 截图为工作树件（仓级 ignore 惯例），证据耐久性靠计划记录
  +可重跑脚本 | evidence（复审重跑非沿用自述）: cargo t plan657 3/3、
  cargo t plan649 10/10、cargo tf 2556/2557（唯一红 mouse_area 经
  b4b04c5cd 基线还原对照实验证为 master 预存；tv 2710/2712 的 display_family
  经双态单跑隔离证为顺序敏感型非回归）、046 全量 build R006=0 且 TS 面仅剩
  2 预存共享基建错（master 时代的 pagination TS2345 与 dialog 'id' 两错被
  本计划顺带修复——基线 5 错对照）、047 gen-only EXIT=0、frontmatter
  零改动（spec.md diff=0）、无 bind 工件（src/front/bps 不存在）、证据档
  17 件 + vm_walkthrough_657.json、探针零残留 | 独立性声明：与执行同会话，
  结论自工件重建（门禁重跑 + 基线归因实验 + diff 实读），未采信执行自述 |
  next: merge（/auto-plan:merge）`
- 2026-09-19 合并收据（/auto-plan:merge，`PLAN-657:r1`，completion_kind:
  delivered）：
  - `prepared`：delivery_commit **8e2f4102b**（reviewed_commit 1f109f454 的
    doc-only 后裔——SD-01 落 docs/specs/blueprint/project.md「消费面与组装
    样板」节 + goals.md GOAL-011/GOAL-010 回写 + 账本 P657-1（architecture）/
    P657-2（reviews→archive 路径）upsert（原子写+读回校验）；spec-index.py
    再生 INDEX 无内容 diff；冻结 delta 对照无越界（project.md +32/goals ±2/
    specs.json +27）。
  - `landed`：master merge **19ae53fc5**（ANCESTRY_OK——8e2f4102b ⊆ master）；
    双冲突并集解（KNOWN-DEBT：master 侧 642 终审+075 执行期两节保留 +
    657 节改号增补六；specs.json：ours 重放 P657-1/2，075 条目保留实证）；
    落地后验证：spec 节在案 + 账本读回（591 items 含 P657-1/2）+ 047
    app.at 六条 use bps 直连 + master 冒烟 `cargo t plan657` 3/3。
  - `ledger_refreshed`：`.autoos/specs.json`（workspace=主检出，P657-1/
    P657-2，来源=canonical project.md/archive plan，created/last_modified
    2026-09-19）。
  - `archived`：git mv → docs/plans/archive/657-bp-admin-sample.md，status:
    archived 本提交。
  - `cleaned`：✅——守门程序执行实录：先按 P075-D1 程序 `cmd /c rmdir` 摘除
    deps/bps junction ×2（047 + 046，链接本体删除、blueprints 完好实证）+
    GNU rm 清 node_modules 链接场（1087 链接→0）→ **wt-guard: clean** 复跑
    确认零 reparse point → worktree 移除（中途两处残留 vite 进程锁
    PID 10136/21620 定点清除后完成）→ 分支 plan-657-dev 删除（was
    8e2f4102b）→ auto-down 依赖位 guard clean 后移除 → 组目录
    D:/autostack/.wt/lang-657 整体移除（磁盘 + 两仓 worktree 注册零残留）。


## 10. 待澄清事项

1. **nav 状态机形态**（T-01 ②）：默认 app model 字段（最简、契约 Q3 纪律
   最清晰）；如倾向 scoped store（跨视图保持），T-01 内改选即可。
2. **empty-state 演示位的交互深度**：默认"报告"页做 first_use→no_result→
   refresh 重试的最小闭环；如希望演示 error 变体（三变体全展），T-03 加一
   个触发按钮，成本极小。
3. **（记录性）** 样板对 Tier 1 的信号收集口径：以 T-05 摩擦清单为准
   （缺的原语/难接的契约/双端差异），不预先设问卷。
