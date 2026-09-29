---
plan_id: PLAN-706
status: archived               # drafting → executing → execution_done → reviewed → archived
completion_kind: delivered
feature_name: gallery-vue-display-fixes
author: [agent]
created_at: 2026-09-28
updated_at: 2026-09-28
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/architecture.md#ADR-25
  - docs/specs/auto-man/project.md#shadcn-脚手架传递依赖
touched_goals: [GOAL-007, GOAL-010]

affects: [auto-lang/ui, auto-os/widgets-gallery]
current_step: 8
total_steps: 8
---

# [PLAN-706] gallery-vue-display-fixes

## 0. 变更摘要

widgets-gallery Vue 臂实机走查（2026-09-28，Vite@127.0.0.1:4173）发现多处组件无法在画廊正常展示。本计划按根因分层修复：

1. **schema/vue 映射缺口**（auto-lang）：`sidebar_menu_sub_button` 缺 `vue: { component, import }`，导致生成 SFC 用到 `<SidebarMenuSubButton>` 却不 import。
2. **lucide 误映射**（auto-lang）：任意 kebab 字面量被当成 lucide 名；`icon(name: "sample")` 生成非法导出 `Sample`，kitchen-sink 整页炸掉。
3. **shadcn 脚手架无传递依赖**（auto-lang）：`form` 脚手架内部依赖 `@/components/ui/label`，materialize 不扩闭包 → Form 页 500。
4. **语料语法缺口**（auto-os）：`area_chart.at` 的 `memo (deps: .a, .b, …)`、`filetree.at` 的 `for … key:` 均解析失败。
5. **首页卡片漏展示**（auto-os）：侧栏有、首页网格无的 7 项。
6. **默认端口 3024** 落在 Windows 排除段（环境/配置），`auto run` 绑不上。

## 1. 目标

### 目标

- G1：widgets-gallery Vue 端 **Command / Combobox / Form / Kitchen-sink** 四页可完整打开并渲染预览。
- G2：**AreaChart** 预览区非空（SVG 面积图可见）；**FileTree** 组件解析通过，再生成后仍可渲染。
- G3：**Sidebar** 页 `SidebarMenuSubButton` 正常 import/render。
- G4：首页卡片与侧栏路由对齐（补齐 Absolute / CodeEditor / Charts×4 / FlowDiagram）。
- G5：lucide 未知字面量不再产生非法 import（回退 Circle 或跳过 import）。
- G6：`vue_shadcn::materialize` 对 `@/components/ui/*` 传递依赖自动扩包（至少 form→label）。

### 非目标

- 不重做 charts/diagram 渲染管线、不做 VM 轨对拍。
- 不改 nav 族退役决策、不动 kitchen-sink 的「全量 sample 演示」设计意图。
- 不强制改 Windows 端口排除策略；仅把画廊 `front_port` 挪到可用端口并在 README 注明。
- 不在本计划做 `/auto-plan:merge` 归档与 specs 沉淀（留给 review/merge）。

### 影响仓/模块

| 仓 | 模块 | 改动性质 |
|---|---|---|
| auto-lang | `schema/aura.at` | 补 `sidebar_menu_sub_button` vue 映射 |
| auto-lang | `crates/auto-lang/src/ui_gen/vue.rs` | lucide 名校验/回退 |
| auto-lang | `crates/auto-man/src/vue_shadcn.rs` | materialize 传递依赖闭包 |
| auto-os | `widgets-gallery/src/front/components/area_chart.at` | memo deps 语法 |
| auto-os | `widgets-gallery/src/front/components/filetree.at` | for-key 语法 |
| auto-os | `widgets-gallery/src/front/pages/index.at` | 首页卡片补齐 |
| auto-os | `widgets-gallery/pac.at` + `README.md` | front_port / 运行说明 |

### 成功判据

见 §7 验收标准 AC-01..AC-08。

## 2. 架构方案

分层修复原则：**能进生成器/schema 的不改语料**；语料只修「本就非法或首页信息架构」两类。

```text
                    ┌─────────────────────────────┐
                    │  schema/aura.at (元素契约)   │
                    │  sidebar_menu_sub_button +vue│
                    └──────────────┬──────────────┘
                                   │ apply_schema_vue_mappings
                                   ▼
┌──────────────┐    generate_shadcn_imports     ┌──────────────────┐
│  ui_gen/vue  │ ─────────────────────────────► │  SFC import 行    │
│ lucide 校验  │                                └────────┬─────────┘
└──────────────┘                                         │ detect_shadcn_components
                                                         ▼
┌──────────────────┐   materialize + 传递闭包   ┌──────────────────┐
│ vue_shadcn bundle│ ─────────────────────────► │ src/components/ui│
└──────────────────┘                            └──────────────────┘

语料侧（auto-os/widgets-gallery）:
  area_chart.at / filetree.at  →  语法合法化（memo / for-key）
  index.at                     →  首页卡片与侧栏对齐
  pac.at front_port            →  避开 Windows 排除端口
```

关键决策：

| ID | 决策 | 理由 |
|---|---|---|
| D1 | `sidebar_menu_sub_button` 补 schema `vue` 映射，而非在 vue.rs 打特判 | 与 548「schema 行挂靠 import 生成」机制一致；单一事实源 |
| D2 | lucide 字面量先查 `LUCIDE_ICONS`（`ui/iced/lucide_generated.rs`），未命中回退 `Circle` **且不插非法名** | 与 tree_icon「动态名 → Circle」降级同族；kitchen-sink sample 自动愈合 |
| D3 | materialize 扩传递依赖：扫描已选组件源码内 `@/components/ui/<name>`，并入本次 materialize 集合（有界，≤2 层） | form→label 是真实缺口；有界避免全量脚手架膨胀 |
| D4 | `area_chart` memo deps 改为 **列表字面量** `deps: [".gridPath", …]`；若 parser 仍拒则退 `memo ()`（Vue 轨 deps 本就不消费） | Vue 侧 memo 只透传子树；优先保语法合法 |
| D5 | filetree 放弃 `for … key:`，改在行节点上 `key: r.id`（Plan 008 既有惯用法） | 全仓唯一该语法；parser 无此形态，不为一页扩 parser |
| D6 | front_port 3024 → **4173**（实测可绑），README 写明排除段 | 3024 在 `netsh excludedportrange` 2992–3091 |

## 3. 技术栈

- Auto DSL / AURA schema（`schema/aura.at`）
- Rust：`auto-lang` ui_gen/vue.rs、`auto-man` vue_shadcn.rs（rust_embed 脚手架包）
- Vue 3 + Vite 5 + shadcn-vue（画廊 gen 臂）
- 验证：`cargo check -p auto-lang` / `cargo t`（按改动范围）+ 画廊 Vue 实机 Playwright 走查

## 4. 需求分析与背景调查

### 用户授权

- 2026-09-28：用户要求「根据发现的问题分析、总结修改方案、建立新计划（/auto-plan:new）、再用 /auto-plan:work 实施」。
- 范围授权：修复 widgets-gallery Vue 版「未正常展示」的组件；实施阶段允许在 worktree 内改 auto-lang 生成器/schema 与 auto-os 画廊语料。
- 未指定预算/自动续跑上限；实施以本计划任务清单为界。

### 实测证据（2026-09-28，Vite 127.0.0.1:4173）

| 症状 | 证据 |
|---|---|
| Command/Combobox 500 | `Failed to resolve import "@/components/ui/command"`；`gen/front/vue/src/components/ui/` 无 `command/` |
| Form 500 | `FormLabel.vue` → `@/components/ui/label` 不存在；`form/index.ts` 可拉但传递导入失败 |
| Kitchen-sink 炸 | `import { Circle, Sample } from 'lucide-vue-next'` — lucide 0.312 无 `Sample` |
| AreaChart 空预览 | `area_chart.at` parse: `expected Colon, found a0`（`memo (deps: .gridPath, .a0, …)`）；DOM 仅空 `<div data-auto-tag="area-chart">` |
| FileTree 解析失败 | `filetree.at:45` `for r in .ftRows key: r.id` → `expected LBrace, found key` |
| Sidebar 子按钮 | 生成 `sidebar.vue` 用了 `<SidebarMenuSubButton>`，import 列表无此项；schema `sidebar_menu_sub_button` 无 `vue:` 行（对照 `sidebar_menu_button:1406` 有） |
| 首页缺卡 | index.at Layout(12)/Form(15) 等无 Absolute、CodeEditor、Charts×4、FlowDiagram；侧栏均有 |
| 端口 | `netsh excludedportrange` 含 2992–3091（3024/3025 均 EACCES）；4173 可绑 |

### 规格基线

- `docs/specs/auto-lang/ui/overview.md`：widgets-gallery 为 shadcn 画廊语料；Plan 548/561 sidebar 族；Plan 408 §9 修复表。
- `crates/auto-man/src/vue_shadcn.rs`：PLAN-457 烘焙脚手架，write-if-missing；**当前无传递依赖**。
- `crates/auto-lang/src/ui_gen/vue.rs:17391` `generate_shadcn_imports`：按 widget spec 的 vue 映射反查 import——依赖 schema `vue:` 行。
- `crates/auto-lang/src/ui/iced/lucide_generated.rs`：lucide 0.312 全量表（>1000）。
- KNOWN-DEBT P695-D6：Vue 臂活体走查曾受阻——本计划在主检出/专用端口复证，不重复登记。

### 现存债务关联

- README「command/combobox/togglegroup 占位」已过时：ToggleGroup 可渲染，Command/Combobox 现为整页 500。本计划顺带改 README 口径。
- 不碰 P555-D4 charts 预存红（auto-lang 测试面）。

## 5. 详细设计

### 5.1 schema：sidebar_menu_sub_button vue 映射（D1）

**文件**：`schema/aura.at`（约 L1432）

before：
```
element sidebar_menu_sub_button {
    ...
    backends: { web: "native", iced: "full", gpui: "unknown" }
    props: [ { name: "class", ... } ]
```

after：
```
element sidebar_menu_sub_button {
    ...
    backends: { web: "component", iced: "full", gpui: "unknown" }
    vue: { component: "SidebarMenuSubButton", import: "@/components/ui/sidebar" }
    props: [
        { name: "class", ... }
        { name: "text", type: "string", description: "Button text" }
        { name: "to", type: "string", description: "Router target" }
        { name: "active", type: "bool", default: "false", description: "Active state" }
    ]
```

对齐 `sidebar_menu_button`（L1400）与 vue.rs `generate_sidebar_menu_button_html`（D2 to/active）。`web: "native"` → `"component"` 与 548 其余 sidebar 族一致。

### 5.2 lucide 字面量校验（D2）

**文件**：`crates/auto-lang/src/ui_gen/vue.rs`（icon 发射点 ~L1632 及同类 ~L1790/2250/6693/6784/7062/12600/13004）

统一收口为一个 helper（建议 `fn lucide_component_for(name: &str) -> Option<String>`）：

1. 保持现启发式：kebab/lowercase 才走 lucide 路径。
2. `kebab_to_pascal` 后查 `ui::iced::lucide_generated::LUCIDE_ICONS`（或 `LUCIDE_ICONS.iter().any`）。
3. 命中 → 原行为（插 import + `:icon-comp`）。
4. 未命中 → **不插该名**，改用 `Circle`（与动态名降级同族）；可选 `eprintln`/诊断通道一次警告（不刷屏）。

覆盖点：`icon(name:)`、nav-item icon、以及其他 `lucide_icons.insert` 发射点中由用户字面量驱动的路径；内部固定名（Search/ArrowRight/Check）跳过校验。

### 5.3 materialize 传递依赖（D3）

**文件**：`crates/auto-man/src/vue_shadcn.rs`

在 `materialize` 入口做有界闭包：

```text
requested = 本次 components
for depth in 0..2:
  for c in requested:
    读 bundle 内 c 的 .vue/.ts
    正则/扫描 `@/components/ui/([a-z0-9-]+)`
    命中且 is_bundled(名) → 加入 requested
再按最终集合拷贝（保持 write-if-missing）
```

已知最小集：`form → label`；顺带覆盖 `form → button|input`（若源码引用）。**不**递归进 `sidebar` 全家（组件内部自洽）。

测试：
- `materialize(&dir, &["form"])` 后 `ui/label/index.ts` 存在。
- 幂等/写保护测试保持通过。

### 5.4 area_chart memo deps（D4）

**文件**：`auto-os/widgets-gallery/src/front/components/area_chart.at:475`

优先：
```
memo (deps: [".gridPath", ".a0", ".a1", ".a2", ".a3", ".l0", ".l1", ".l2", ".l3", ".hovAr", ".visAr0", ".visAr1", ".visAr2", ".visAr3", ".grid", ".legendColor0", ".legendColor1", ".legendColor2", ".legendColor3"]) {
```
若 parser 仍拒（表达式列表），降级 `memo ()`（与 `datatable.at:76` 同型），并在注释写明 VM deps 优化延后。

另核对 L388-390 / L558 一带 `text .tipTitle` 是否在 memo 修复后可解析（错误链第 2 条 `Expected node name, got Dot(Ident("text"), "tipTitle")`）；若为同一语法族缺口，按 `text { .tipTitle }` 或既有 text 惯用法改写。

### 5.5 filetree for-key（D5）

**文件**：`auto-os/widgets-gallery/src/front/components/filetree.at:45`

before：`for r in .ftRows key: r.id {`
after：`for r in .ftRows {`，并在行 `row (…)` 上挂 `key: r.id`（Plan 008：key 在分支首子/行节点）。

对照 `treeview.at` 的 `for r in .rows` 无 key 形态；保持行为（展开/选中）不变。

### 5.6 首页卡片补齐（G4）

**文件**：`auto-os/widgets-gallery/src/front/pages/index.at`

| 分组 | 补卡片 | 计数修正 |
|---|---|---|
| Layout | Absolute | 12→13 |
| Form | CodeEditor | 15→16 |
| Charts（新分组） | LineChart, BarChart, AreaChart, DonutChart | +4 |
| Diagrams（新分组） | FlowDiagram | +1 |

同时把 hero「63 Widgets」改为与实际卡片数一致（或改为不写死数字）。侧栏已有项不得删减。

### 5.7 端口与文档

- `widgets-gallery/pac.at`：`front_port: 3024` → `4173`（或 `5173`）。
- `widgets-gallery/README.md`：运行端口说明 + 已知边界更新（command/combobox 不再是「占位 div」而是需 ui/command；ToggleGroup 已可渲染）。

### 规范增量

| delta_id | add/modify/retire | target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | `schema/aura.at` sidebar 族 | sidebar_menu_sub_button 无 vue 映射 / 有 vue+component | 与 548 import 生成机制对齐 | AC-03 |
| SD-02 | modify | `docs/specs/auto-lang/ui/overview.md`（或 vue 生成器小节） | 无「未知 lucide 字面量回退 Circle」规则 / 有 | 全应用统一降级，防非法 import | AC-05 |
| SD-03 | modify | `docs/specs/auto-man/`（vue 脚手架，若无则 overview） | materialize 无传递依赖 / 有界闭包 | form→label 类缺口根治 | AC-06 |
| SD-04 | modify | widgets-gallery README 已知边界 | 陈旧占位口径 / 与实机一致 | 文档可信 | AC-07 |

无 Spec 影响的纯语料语法/index 卡片改动不进规范增量（属样例内容）。

## 6. 测试设计

| 层 | 测什么 | 命令/方式 | 期望 |
|---|---|---|---|
| 编译 | auto-lang 语法/类型 | `cargo check -p auto-lang`（及 `cargo check -p auto-man`） | 0 error |
| 单测 | lucide 回退 | 新增/扩展 vue.rs 测试：`icon(name:"sample")` 不产生 `Sample` import，含 `Circle` | 通过 |
| 单测 | schema 映射 | schema_drift / 现有 sidebar import 测试：`SidebarMenuSubButton` 出现在 import | 通过 |
| 单测 | materialize 闭包 | `vue_shadcn` 测试：materialize form 后 label 存在 | 通过 |
| 语料 | area_chart/filetree 可解析 | `cargo t gallery_pages` 或页面编译冒烟 | 无 parse warning |
| 实机 | 四页+AreaChart+Sidebar | Playwright 打开对应路由，断言 h1 与预览非空 | 全部命中 |
| 实机 | 首页卡片数 | 计 `component-card` 数与侧栏链接数差 | 缺口 ≤0（厨房水槽页除外） |

快速迭代期不跑全量 `cargo tf`；合入前按 Category B 最终门禁跑一次。

## 7. 验收标准

| ID | 标准 | 验证方法 | 期望结果 |
|---|---|---|---|
| AC-01 | Command 页可打开且有预览 | Playwright `#/command` | h1=Command，无 500，预览区非空 |
| AC-02 | Combobox 页可打开且有预览 | Playwright `#/combobox` | h1=Combobox，无 500 |
| AC-03 | Form 页可打开 | Playwright `#/form` | h1=Form，FormLabel 渲染，无 label 模块错误 |
| AC-04 | Kitchen-sink 可打开 | Playwright `#/kitchen-sink` | 无 `Sample` import 错误，页标题正确 |
| AC-05 | AreaChart 预览非空 | Playwright `#/area-chart` | preview 内有 svg/path，非空 div |
| AC-06 | FileTree 源可解析且页可渲染 | gen 日志无 filetree parse error；`#/filetree` 有树行 | 双通过 |
| AC-07 | Sidebar 子按钮有 import | 生成 sidebar.vue 含 `SidebarMenuSubButton` 于 import | 通过 |
| AC-08 | 首页卡片含 Absolute/CodeEditor/Charts/FlowDiagram | 首页 DOM component-card 清单 | 7 项均在 |

## 8. 执行步骤

Worktree（Plan 529 分组平铺）：

```bash
# master 取号骨架已创建（PLAN-706）
git -C D:/autostack/auto-lang worktree add D:/autostack/.wt/lang-706/auto-lang -b plan-706-dev
# auto-os 语料
git -C D:/autostack/auto-os worktree add D:/autostack/.wt/lang-706/auto-os -b plan-706-os-dev
```

| ID | 任务 | 依赖 | 产出/文件 | 验证 | AC |
|---|---|---|---|---|---|
| T-01 | schema 补 `sidebar_menu_sub_button` vue 映射 + web:component + to/active props | — | `schema/aura.at` | schema_drift / cargo t schema 相关 | AC-07 |
| T-02 | vue.rs lucide helper + 各发射点收口 + 单测 | — | `ui_gen/vue.rs` | `cargo t lucide` 或 vue 相关单测 | AC-04, AC-05 |
| T-03 | vue_shadcn materialize 传递闭包 + 单测 | — | `vue_shadcn.rs` | `cargo t -p auto-man vue_shadcn` | AC-03 |
| T-04 | area_chart.at memo/tipTitle 语法合法化 | — | `widgets-gallery/.../area_chart.at` | 无 parse error；页面 svg | AC-05 |
| T-05 | filetree.at for-key → 行上 key | — | `filetree.at` | 无 parse error；页面树行 | AC-06 |
| T-06 | index.at 补 7 卡 + 计数/文案 | — | `index.at` | 首页 DOM 清单 | AC-08 |
| T-07 | pac.at front_port + README 口径 | — | `pac.at`, `README.md` | 文档一致 | — |
| T-08 | 全量再生成 + Playwright 实机走查四页/图/首页 | T-01..07 | 走查截图/日志 | AC-01..08 全过 | all |

执行顺序：T-01/02/03 与 T-04/05/06/07 可并行（分仓）；T-08 收口。

进度勾选（work 阶段）：

- [x] T-01 schema sidebar_menu_sub_button vue 映射（f06e2ec7a）
- [x] T-02 lucide_component_name 回退 Circle（f06e2ec7a）
- [x] T-03 vue_shadcn 传递闭包 form→label（f06e2ec7a，新测绿）
- [x] T-04 area_chart memo deps 列表化（bde65b0）
- [x] T-05 filetree for-key → 行上 key（bde65b0）
- [x] T-06 index.at 补 7 卡 + Charts/Diagrams 组（bde65b0）
- [x] T-07 pac.at front_port 4173 + README（bde65b0）
- [x] T-08 实机走查 AC-01..08 全过（证据 `D:/autostack/.wt/lang-706/evidence/`）：
  - AC-01 Command h1=Command 无 500
  - AC-02 Combobox h1=Combobox 无 500
  - AC-03 Form h1=Form，FormLabel×2 渲染
  - AC-04 Kitchen Sink h1=Kitchen Sink，icon=Circle（无 Sample）
  - AC-05 AreaChart 7 path / 5 svg，空预览 0
  - AC-06 FileTree 16 svg 可渲染；filetree.at 无 parse error
  - AC-07 sidebar.vue import 含 SidebarMenuSubButton；General/Billing 文案在页
  - AC-08 首页 7 卡 + 70 Widgets 全在

## 9. 复审记录

### stage: new
- Plan ID **PLAN-706**, revision **1**
- `outcome: pass`；`next: work`

### stage: review
- `plan_id: PLAN-706` `plan_revision: 1`
- `outcome: pass`
- `reviewed_commit: auto-lang f06e2ec7a / auto-os bde65b0`
- `base_commit: auto-lang 025fb192c (master@起草时) / auto-os 6c9c7ba (main@起草时)`
- `dependency_revisions: auto-down sibling worktree 3373a5c（path 依赖，未改）`
- `spec_inputs: schema/aura.at@f06e2ec7a（SD-01）；vue_shadcn.rs/vue.rs 内注释为 SD-02/03 规则源`
- `acceptance_results: AC-01..08 全 pass（Playwright 4180 实机，证据 .wt/lang-706/evidence/，**不入 git**）`
- `findings:`
  - `F-01 low: state_file::tests::lock_serializes_critical_sections 并行负载 flaky（单跑绿）——预存，非 706 回归`
  - `F-02 low: musk_vm_track_p053 4 红（computed/api warn）在 master 同红，预存；与 706（vue/schema/脚手架）无因果`
  - `F-03 med process: plan-706-os-dev 上叠了 plan047 提交 c63ea70（非 706 范围）；merge 仅落 bde65b0`
  - `F-04 process: 用户裁定截图不入 git；merge 忽略 evidence/*.png`
- `evidence: vue_shadcn 6/6；schema_drift 2/2；sidebar 16/16；lucide 11/11；生成日志 area_chart/filetree 无 Parse error；AC 截图见 evidence/`
- `next: merge`
- `plan_id: PLAN-706` `plan_revision: 1`
- `code_commit: auto-lang plan-706-dev f06e2ec7a / auto-os plan-706-os-dev bde65b0`
- `task_ids: T-01..T-07 done；T-08 部分（gallery_pages 冒烟在跑，无 area_chart/filetree parse error）`
- `evidence:`
  - vue_shadcn 6/6 含新测 `materialize_form_pulls_label_transitively`
  - schema_drift 2/2；sidebar 16/16；lucide 11/11
  - area_chart memo deps → 列表字面量；filetree for-key → 行上 key
  - index.at 补 Absolute/CodeEditor/Charts×4/FlowDiagram；front_port 4173
- `blockers: 无`
- `outcome: pass`（T-01..T-08 全完成，AC-01..08 实机证据在 `D:/autostack/.wt/lang-706/evidence/`）
- `next: review`

### stage: merge PLAN-706:r1
- `outcome: pass`
- `prepared: reviewed f06e2ec7a+bde65b0；SD-01 schema 已在代码提交；SD-02 ADR-25 + SD-03 materialize 传递依赖 写入 docs/specs`
- `landed:`
  - auto-lang master tip **3b0b1bc03**（ff-only；旧 f06e2ec7a → 新 7b8a3ccbc range-diff `=` 全等；+ specs 3b0b1bc03）
  - auto-os main tip **7eecc14**（cherry-pick bde65b0 only——plan-706-os-dev 上的 c63ea70/plan047 **未带入**）
- `ledger_refreshed: docs/specs/auto-lang/ui/architecture.md ADR-25 + docs/specs/auto-man/project.md shadcn 传递依赖节`
- `archived: docs/plans/archive/706-gallery-vue-display-fixes.md；completion_kind: delivered`
- `cleaned: wt-guard clean×2；worktree 移除（lang/os/down/backup）；分支 plan-706-dev / plan-706-os-dev / plan-706-os-land 已删；evidence/ 留盘不入 git`
- `deployment_observation: 桌面下次启动现场构建拾取；release auto.exe 未重建；widgets-gallery gen 需 `auto run -r vue` 再生后端口 4173`
- `F-03 处置: plan-706-os-dev 叠 047 提交已隔离，仅落 bde65b0`

## 10. 待澄清事项

| ID | 问题 | 影响 | 默认 |
|---|---|---|---|
| Q-01 | memo deps 列表字面量是否已被 parser 支持？ | D4 实现路径 | 不支持则 `memo ()` |
| Q-02 | lucide 未命中回退 Circle 还是跳过 icon？ | kitchen-sink 视觉 | 回退 Circle（与动态名一致） |
| Q-03 | Charts/Diagrams 首页独立分组 vs 塞进 Display？ | 信息架构 | 侧栏已有独立组，首页对齐侧栏 |
| Q-04 | front_port 统一 4173 还是 5173？ | 本机/CI | 4173（本次实测） |
