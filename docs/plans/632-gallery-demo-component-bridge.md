---
plan_id: PLAN-632
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: gallery-demo-component-bridge
author: [zhaopuming, agent]
created_at: 2026-09-15
updated_at: 2026-09-15

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/vm, auto-lang/ui, auto-man]
current_step: 0
total_steps: 6
---

# [PLAN-632] gallery-demo-component-bridge

## 0. 变更摘要

画廊 VM 内嵌 demo 语境下，**模块组件实例不桥接**：demo 源内 `use <mod>: Component` 引入的组件（006-hero-section 的 SettingsPopover、016-calendar 的 CalendarStore）在实时内嵌视口中实例降级——006 弹层面板 0×0 空壳（vtree 实证），016 的 `.store.*` 绑定求值失败致 `${}` 原样输出、月网格空。本计划定位并修复 VM 桥接在该语境下的组件视图展开与参数/store 状态注入。

## 1. 目标

- 内嵌 demo 里的模块组件实例：视图展开、参数（props）双向可达、组件自身 model 状态与 store 提供的状态（`.store.*`）注入消费方作用域。
- 修复后 006 的 ⚙ Theme 弹出面板可开合，016 的月网格渲染真实日期、无 raw `${}`。
- 非目标：全栈 demo 后端挂载（PLAN-633）；Vue 臂行为变更；006 独立运行（standalone）形态改造。

## 2. 架构方案

两组件症候、共同根因待 T-01 实证：

1. **参数型视图组件**（006 SettingsPopover，`use settings: SettingsPopover`）：组件实例在 demo 编译语境未展开为子 widget 子树（vtree 全部 bbox 0×0 空壳）。假设：`use.web` 组件走 `.vm.at` 探测装载（PLAN-051 C4）可用，而 `use <mod>:` 模块组件缺少等价的实例展开路径。
2. **store 型组件**（016 CalendarStore，`use calendar_store: CalendarStore`）：`resolve_use_module`（auto-lang/src/lib.rs:2924）仅对**字面名为 `store`** 的模块走 `StoreFiles` 分支；自定义模块名的 store 组件不触发 store 状态播种（PLAN-522 机制），`.store.*` 求值落空。

修复方向（按 T-01 结论择取/组合）：
- A1：store 识别泛化——`use <mod>: Item` 的 Item 为 store 声明（或模块文件含 store 声明）时走 StoreFiles 等价路径；
- A2：模块组件实例展开——demo 编译 walks（lib.rs import walk / `load_ext_imports_for_vm` / `synthesize_widget_module` 的 child_widgets 集合）把 `use <mod>: Component` 的组件声明纳入 child 实例化，参数按实例 props 绑定、状态独立 scope；
- A3：内嵌 demo 的 Init/状态播种缺口排查（每次启动的 `App.Init failed (state may be unpopulated): handler not found: Init` 警告与症状同现，须定性是否相关）。

## 3. 技术栈

Rust（auto-lang vm/ui 桥接、auto-man 发射器）；.at 语料；验证走 cargo nextest + AutoUI MCP（`AUTOUI_MCP_PORT` 驱动 snapshot/vtree/screenshot）。

## 4. 需求分析与背景调查

- **授权与范围**：用户在 ui-gallery VM 保真度核查会话（2026-09-15）裁定"两个深水区立 plan 处理"；本计划覆盖组件/store 状态桥接。允许仓库：auto-lang（运行时+发射器）；auto-os/ui-gallery 仅作验证靶（不提交）。
- **实测证据**（2026-09-15 会话，auto-os/ui-gallery + auto-lang master 构建）：
  - 006：`[VM_HANDLER_CALL] widget=Demo006HeroSection event=ToggleSettings` → `VM_HANDLER_OK`，demo 自身状态翻转；vtree 中 SettingsPopover 子树全部 bbox 0×0。
  - 016：壳渲染（表头/Today/Settings 按钮），`${month_label}`/`${year}`/`Selected: ${selected_date}` 原样输出（app.at L28 `month_label => month_name(.store.month)`，L48/49/156 绑定 `.store.*`）；月网格空。
  - 对照：002-counter（demo 根 widget 自身状态）插值正常；006 根 col 的 `style: if .dark_mode` 分支生效——demo 根 widget 的 model 桥接无恙。
  - 每次启动均有 `[VM-HANDLER] App.Init failed (state may be unpopulated): handler not found: Init`。
- **判别线索**：`resolve_use_module`（auto-lang/src/lib.rs:2924）仅 `module == "store"` 走 StoreFiles；PLAN-533/534/536（overlay/反应性/子件 prop 批）为最近相关运行时修复；P536-D1 登记 slot-fill×hoist 限制。
- **相关 Spec**：docs/specs/auto-lang/ui/architecture.md、docs/specs/auto-lang/vm/architecture.md、docs/specs/auto-man/project.md（画廊发射器）。
- **未决**：006/016 **standalone** VM 运行是否同样降级（判别"内嵌特有" vs "模块组件通病"）→ T-01 有界调查。

## 5. 详细设计

### 规范增量

| delta_id | add/modify | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/architecture.md | before: 模块组件（`use <mod>: Component`）在 VM 渲染目标无实例化契约，内嵌语境降级为空容器；after: 模块组件实例必须展开视图子树、按实例 props 桥接参数，store 型组件（模块含 store 声明）的状态注入消费方作用域——与 `use.web` 组件同保真 | 内嵌 demo 保真缺口根因 | AC-01, AC-02 |
| SD-02 | modify | docs/specs/auto-man/project.md | before: 发射器仅保证单文件 demo 自包含；after: `demos/` 级联发射保证模块组件源可达（deps 型项目解析、自有模块相邻拷贝），组件状态桥接为运行时职责 | 发射器与运行时职责分界 | AC-01, AC-02 |

### 设计要点

- 修复落点以 T-01/T-02 结论为准，预期集中在：auto-lang/src/lib.rs（`resolve_use_module` store 分支泛化 / import walk 组件收集）、auto-lang/src/ui/vm_bridge.rs + dynamic.rs（child 实例化与状态 scope）、auto-lang/src/ui/handler_codegen.rs（组件 handler 编译进单一 VM 的 alias 面）。
- 发射器（auto-man/src/vue.rs `emit_gallery_vm_demos`）不随本计划改动（模块级联发射已在前置会话落地）；若 T-01 判定需要 demo 侧伴随物（如实例化清单），在该文件内小步补充。
- 回归护栏：全部既有 loadable demo（001-031 除独立件）修复后 MCP 巡检不回退。

## 6. 测试设计

- 单测（新，auto-lang）：最小语料——`use <mod>: Component` + 组件 model/params 的桥接断言（组件实例存在、param 双向、store 状态可见），归 `auto-lang/src/ui/` 既有测试模块风格。
- 定性探针：capability-tests 语料页（若有 slot/store 先例页复用）。
- 实机 MCP：ui-gallery 双靶（006 popover 开合截图 + vtree bbox 非零；016 网格 snapshot 文本断言无 raw `${}`、含当日邻域日期）。
- 门禁：`cargo tv`（改 VM/编译器后必跑）+ `cargo t`；不触 aavm 面（`taa` 零触发）。

## 7. 验收标准

- **AC-01**：画廊内嵌 006，MCP `autoui_action` 点击 ⚙ Theme 后 Settings 面板可见（截图）且 vtree 该子树 bbox 非零；再点关闭。验证：MCP 脚本 + 截图入 `src/front/tmp/`（gitignore）。
- **AC-02**：画廊内嵌 016，主标题渲染月份名（非 `${month_label}`），网格含 28-31 个日期格，`Selected:` 行随点击更新。验证：MCP snapshot 文本断言 + 截图。
- **AC-03**：新增最小语料单测覆盖模块组件 param/store 桥接，`cargo nextest run -p auto-lang <filter>` 绿。
- **AC-04**：`cargo tv` 无新增红（存量红以 master 基线为准，如 charts_gallery）。
- **AC-05**：MCP 巡检 002/003/011 内嵌渲染与状态不回退（回归护栏）。

## 8. 执行步骤

- **T-01**（有界调查）：006/016 **standalone** `auto run -r vm` 对照实验，MCP 抓 popover/store 状态；产出根因判定记录（写入 §9）。涉及：只读运行 + 截图。→ AC-01/AC-02 前置。验证：判定记录落档，明确"内嵌特有 / 模块组件通病 / store 命名分支"三选一或组合。
- **T-02**（新）：最小复现单测骨架（`auto-lang/src/ui/` 下新测试模块）：模块组件桥接语料 + 当前失败断言（红→后续转绿）。验证：`cargo nextest run -p auto-lang -E 'test(<new>)'` 如期红。→ AC-03。
- **T-03**：按 T-01 结论实现修复（A1 store 命名分支泛化 / A2 模块组件实例展开，可组合）；T-02 单测转绿。验证：同上命令绿 + `cargo check -p auto-lang` 零新增警告。→ AC-01/AC-02/AC-03。
- **T-04**：内嵌实机 MCP 验证（006 开合、016 网格、002/003/011 回归巡检），证据截图落档。验证：AC-01/AC-02/AC-05 逐条核对。→ AC-01/AC-02/AC-05。
- **T-05**：`cargo tv` 门禁 + App.Init 警告定性（若与本修复相关顺手清偿，否则登记 KNOWN-DEBT）。验证：tv 无新增红。→ AC-04。
- **T-06**：计划簿记（勾选、frontmatter、spec delta 回填）+ 复审移交。验证：`/auto-plan:review`。

## 9. 复审记录

- 2026-09-15 draft 新建（stage: new, PLAN-632 rev1）：基于当日 ui-gallery VM 核查实测证据起草；handoff `next: work`，T-01 为首任务。

## 10. 待澄清事项

- 006 standalone VM 的 popover 形态未知（T-01 首查）。
- `App.Init failed` 警告是否与组件状态播种同源（T-05 定性；若无关仅登记）。
- store 命名泛化（A1）若与 PLAN-522 既有 `use store:` 契约冲突，回滚为"仅泛化到 store 声明识别"窄修——在 T-03 决策点记录。
