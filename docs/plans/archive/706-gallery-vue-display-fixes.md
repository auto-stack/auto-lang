---
plan_id: PLAN-706
status: archived              # 终态（2026-09-30 r2 merge pass——archived）
# r2 重开（2026-09-29）：r1 交付簇携入的 vm UI 脏标/确认弹层交互面回归——追加修复
# phase（T-09..T-11，§11）。立项=消费者仓 jade-edit PLAN-026 回执批冒烟败形实证 +
# 用户直接裁定；rev1 交付记录（§8 勾选/§9 既有收据）原样保留。
completion_kind: delivered     # r1+r2 双 phase delivered（r2 delivery 612514d5a）
feature_name: gallery-vue-display-fixes
author: [agent]
created_at: 2026-09-28
updated_at: 2026-09-30
plan_revision: 2

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/auto-lang/ui/architecture.md#ADR-25
  - docs/specs/auto-man/project.md#shadcn-脚手架传递依赖
touched_goals: [GOAL-007, GOAL-010]

affects: [auto-lang/ui, auto-os/widgets-gallery]
current_step: 11
total_steps: 11
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

### 2026-09-29 replan（r2 重开：vm UI 交互面回归修复 phase）

- `stage: new` | `plan_id: PLAN-706` | `plan_revision: 2` | `outcome: pass`（修订稿完成——用户直接裁定立项 2026-09-29「既然是 706 引发的回归问题，那么应当在计划 706 添加一个新的 phase 去解决」；自 archived 重开）。
- 立项依据：消费者仓 jade-edit PLAN-026 回执批冒烟败形（jade-edit commit `6243eb2` + 计划 §8/§9 全录）——**r1 交付簇（7b8a3ccbc gallery display fixes + 同窗携入的 plan047 memo 大宗）落 master 后，vm UI 脏标/确认弹层交互面出现消费者可复现的确定性回归**：新载体双 exe merged 臂四连断（debug 标脏弧进程死亡 ×2 同点 / release 脏关弧确认弹层栈积 ×2 同点），A/B 锚=旧 exe 同矩阵同 fixture 全绿在库。r1 的 AC-01..08 全部实机通过——本回归在其断言域之外（gallery 走查未覆盖「编辑→标脏→关闭确认」交互弧），属 r1 验证盲区非 r1 执行缺陷。
- changed：frontmatter（status archived→drafting、plan_revision 2、total_steps 8→11、updated_at）；added：§11（T-09..T-11、AC-09..11、域协调注记）；rev1 全部内容原样保留（§0..§10、§8 勾选、§9 既有 stage:new/review/merge 记录不动）。
- `next: work T-09`（bisect 定谳起）；落地序与 PLAN-709（aura_view_builder 域在飞起草）协调注记见 §11.4。

### stage: work | PLAN-706 | plan_revision 2 | outcome: pass（work 收口，2026-09-30）
- `code_commit`: plan-706-dev **9420eb7a2**（84befb240 T-09 定谳+T-10 值守卫 / f509b47ff T-10 死键守卫 / 9420eb7a2 定谳报告扩写；基 master 4dae03122）
- `task_ids`: T-09..T-11 全完成（AC-09 定谳+最小弧+机制实证；AC-10 定义性验收=消费者 merged 矩阵 16/16 ALL GREEN debug+release 双载体；AC-11 probe_mtime 定谳独立缺陷另立记账+家族 UI 面零回归）
- `evidence`:
  - 定谳：first-bad=432068075（plan047 T-06 信号网）；RC canary UAF panic 栈 rc.rs:742←vmref_to_vec←resolve_for_iterable 全文 `app-death-*.log`；机制=信号缓存裸 Value 无 RC 堆身份+死键版本冻结伪命中（双面同根因）
  - 修复：computed_signal_store 堆身份准入 + deps_unchanged 死键 miss（零新特性=pre-706 每帧重算退档语义）；plan706 单测×3（红相取证在案）
  - 消费者：vm_matrix merged 16/16 ALL GREEN（debug `p706-matrix-fixed2-debug.log` + release `p706-matrix-fixed2-release.log`，[10b] 弹层滞留消）；家族弧 CPU load 5/5（pre-fix 同条件 1/2 红）
  - 家族：ui:: 1616/1617（1 红 base 预存 old-carrier 同红）；gallery 四页路由真达 GREEN
  - 定谳为非本回归：probe_mtime release 挂起（old-carrier release 同挂→P706-D1）；消费者 e2e flaky（双载体同 flaky→P706-D2）；gallery 侧栏连按 quirk（双载体同→P706-D3）
- `blockers`: 无
- `next`: review（worktree 保留；spec delta=无新增——SD-01..04 已于 r1 沉淀，r2 为纯回归修复不动 canonical Specs 契约面）

### stage: review | plan_id: PLAN-706 | plan_revision: 2 | outcome: pass（2026-09-30）
- `reviewed_commit`: plan-706-dev **9420eb7a2**（84befb240+f509b47ff+9420eb7a2 三提交链）；`base_commit`: 4dae03122；`dependency_revisions`: auto-down 组兄弟 3373a5c detached（未改）
- 独立性声明：本复审在实现会话内进行——结论由工件重建（门禁全量重跑+基线对照），不依赖执行摘要
- `spec_inputs`: 无 canonical Specs 引用变更——r2 零规范增量（grep docs/specs 无 memo/信号网契约文本，plan046/047 未沉淀 canonical；准入守卫=代码内已成文降级哲学的同族应用）；frontmatter spec 元数据沿 r1 不动
- `acceptance_results`（复审基线全量重放）:
  - AC-09 **pass**：定谳报告在档（worktree 提交，merge 随链落 master）；A/B 锚与 panic 栈实证在案
  - AC-10 **pass**：消费者 merged 矩阵 16/16 ALL GREEN——复审基线重放第三绿（`p706-review-matrix.log`，HEAD 重建 exe）；家族弧 HEAD 3/3（2 轮带 load）
  - AC-11 **pass**：probe_mtime 定谳独立缺陷（P706-D1，old-carrier release 同挂）+家族 UI 面零回归=tf 红集基线对照证
- **tf 门禁（Category B 全量档，VM/engine 改动触发）**：`cargo tf` 全量普查 HEAD 18 红 vs base(4dae03122) 27 红——16 红双面同集（musk_vm_track p053/p054 族×5、docs_gen core_reference+kitchen_sink×2、plan394 c1、projector_counter、plan606 029、a2vue_desktop、p508_g2、plan358 stress、serve_once/enable_broker）；2 HEAD-only 红（broker_incubation_full_flow/desktop_connect_adopts_live_serve）单测复跑即绿，且 base 侧同族 11 红为 run-unique（desktop_protocol 进程族环境 flaky，双基线红集方差自证）⇒ **净回归=0，base-only 红 11 项=环境方差**。已知预存红中 musk p053 ×2 于 base 同红单测复证（复审首跑即遇）
- **tf 异常定谳（新债 P706-D4）**：`plan705_e2e_deadline_cancels_parked_and_reclaims` 在 tf 并行池内 >62min 挂起（全量套件无法收尾）；单测隔离 <3min 绿。根因=705 时代门禁违例：裸 `#[test]` 绑定固定端口 18511/18512 + 全局 env 变更（`AUTO_HTTP_REQUEST_TIMEOUT_MS`），其自家计划 §6 明示真 TCP e2e 应入 `test-http-e2e` 门/th 串行池——非 706 r2 引入。普查按 `-E 'not test(...)'` 排除后完成，排除项已单测复证
- `findings`:
  - `F-r2-01 medium(pre-existing): plan705 e2e 门禁违例入 tf 并行池（固定端口+env 污染）→ 全量套件挂起风险——登记 P706-D4，清偿归属 705 域治理（cfg 门收编 th 串行池），不阻塞本计划`
  - `F-r2-02 info(pre-existing): tf 红集家族（musk/docs_gen/desktop_protocol）双基线方差在案——base 27 红中 11 项 run-unique，红册治理归后续档位`
  - `F-r2-03 process: master 漂移至 b310acafd（707 code 落地+708/709 推进；709 同域 aura_view_builder）——merge 必须 merge-sync master 入分支后重跑定向门禁（plan706/047/046/memo/vm_bridge+双形态 check+arc/matrix 抽查）再落地`
- `evidence`: tf-review-full.log（HEAD 18 红）/tf-base-census.log（base 27 红）/red-diff 处置/arc rrun-1..3.log/p706-review-matrix.log（均组目录 .wt/lang-706/，不入 git；结论与命令已录本记录可复跑）
- `next`: merge（前置=merge-sync master 重验；无 canonical Specs 沉淀项——r2 零规范增量）

## 10. 待澄清事项

| ID | 问题 | 影响 | 默认 |
|---|---|---|---|
| Q-01 | memo deps 列表字面量是否已被 parser 支持？ | D4 实现路径 | 不支持则 `memo ()` |
| Q-02 | lucide 未命中回退 Circle 还是跳过 icon？ | kitchen-sink 视觉 | 回退 Circle（与动态名一致） |
| Q-03 | Charts/Diagrams 首页独立分组 vs 塞进 Display？ | 信息架构 | 侧栏已有独立组，首页对齐侧栏 |
| Q-04 | front_port 统一 4173 还是 5173？ | 本机/CI | 4173（本次实测） |

## 11. rev2 追加 phase（2026-09-29）：vm UI 脏标/确认弹层交互面回归修复

### 11.1 回归实证（消费者仓一手实录，jade-edit PLAN-026 执行窗 2026-09-29）

**断言面**：jade-edit `tests/vm_matrix.mjs` merged 臂（进程内直调，AUTOUI_MCP_PORT 驱动真 UI）——r1 载体（09-28 15:02 debug exe，master@c8f86ef92 时代）同矩阵同 fixture **merged 16/16 ALL GREEN**（jade-edit `p25-merge-smoke.log`，09-29 19:42）；新载体（66c9cac19，`cargo build -p auto` 双 exe）**确定性断裂 ×4**：

| # | 载体 | 断点 | 形态 | 证据 |
|---|---|---|---|---|
| 1 | debug 09-29 21:26 | `[7 tab]` 标脏弧（typeWholeDoc 标脏→切回） | **进程死亡**（[B baseline] PASS 后 MCP 控制通道 SocketError other side closed；vm_matrix:673/674 双跑同点） | jade-edit `p26-matrix-r1/r2.log` |
| 2 | release 09-29 21:19 | `[10b]` 脏关弧（关档触发关闭确认） | **确认弹层栈积 ×2 同点**（[7 tab]+[10 link] PASS 后 `button "Hello World" not found in region` @:774；fail-snap 实勘**双叠「有未保存的修改」确认弹层滞留**=脏态泄漏形） | jade-edit `p26-matrix-rel-r1/r2.log` + `fail-snap-1790689474787.txt` |
| 3 | release | probe_mtime Init | **`probe_cases` 调用挂起 ×2**（done 恒 false——release 特异，debug 同分钟绿 r2 对照）= 第三断裂面 | jade-edit `p26-probe-mtime-rel-r2.log` |
| 4 | 双载体 | 基线 v20 / probe_sb / probe 族 14 件 / build gen 面 | **全绿**（dump 结构面完好——回归居交互语义面非数据面；705 http 面已排除[probe HTTP 全绿]；vue.rs 生成面已排除[build ✓built 20.23s]） | jade-edit `p26-*` 全族 |

**域定位（候选，非定谳）**：载体 delta `c8f86ef92..66c9cac19` 中 **r1 合并携入的 UI 层大宗**——`aura_view_builder.rs +971` / `vm_bridge.rs +616`（plan047 memo_deps/vm_bridge 大宗同窗携带：43f02f195/ab84f1df9/ce71fff1b）/ `vue.rs` gallery display fixes（7b8a3ccbc：sidebar sub-button mapping/lucide fallback/scaffold transitive deps）。r1 主提交 7b8a3ccbc 自身 diff 不含 checkbox/menubar 域（消费者 grep 定谳）——**肇事面需 T-09 bisect 定谳**，不预设。

### 11.2 任务（T-09..T-11）

| ID | 任务 | 依赖 | 产出/文件 | 验证 | AC |
|---|---|---|---|---|---|
| T-09 | **bisect 定谳**：git-archive A/B 于 delta `c8f86ef92..66c9cac19`（候选集=051b892ac 携入批[7b8a3ccbc/43f02f195/ab84f1df9/ce71fff1b 族]+705 簇 d37019d60..71aa3d71d[预期排除——HTTP 面绿]）；再现弧=标脏（INPUT_TEXT 整文构造）→切 tab→关闭确认弹层（消费者 harness `jade-edit tests/vm_matrix.mjs` [7 tab]/[10b] 弧可直接复用，或家族侧最小驱动） | — | 定谳报告（肇事提交集+最小再现弧+触发机制初判） | 绿前红后（pre 提交绿/post 提交红同弧） | AC-09 |
| T-10 | **修复实现**：按 T-09 定谳——脏标投影/关闭确认弹层生命周期（开闭幂等/不栈积/关闭路径不致死）；范围=回归修复还原 rev1 前行为语义，零新特性；含家族侧单测（弹层开闭幂等/脏态切换弧） | T-09 | 修复提交（域随定谳） | 家族单测绿 + 消费者再现弧绿 | AC-10 前半 |
| T-11 | **收口验证**：家族面（cargo t UI 域 + gallery 实机抽查四页不回归）+ **消费者回执面**：重建 exe → jade-edit `tests/vm_matrix.mjs` merged 16/16 ALL GREEN（[7 tab] PASS + [10b] 无弹层滞留）+ e2e 全绿 + probe_mtime release 载体绿 | T-10 | 收口证据（家族+消费者双面） | 消费者矩阵 ALL GREEN = 本回归修复的定义性验收 | AC-10 后半, AC-11 |

进度勾选（r2 work 阶段，2026-09-30；worktree `D:/autostack/.wt/lang-706/auto-lang` branch `plan-706-dev`，基 master 4dae03122）：

- [x] T-09 bisect 定谳（84befb240）：家族最小驱动 `tab-arc.mjs`（组目录不入 git）复刻 [7 tab] 弧；A/B 锚 pre `c8f86ef92` GREEN / master tip RED(exit=101)；git bisect 定谳 **first-bad=`432068075`（plan047 T-06 computed 信号网）**，7b8a3ccbc/3a72d0f15/bb303e914 同弧全绿（消费者「7b8a3ccbc 非肇事」grep 预判复证；705 HTTP 簇预期排除成立）；panic 实证 `[RC canary] use-after-free: heap object 4000555` @rc.rs:742 ← vmref_to_vec ← resolve_for_iterable；定谳报告 `docs/plans/reports/706-bisect-verdict.md`（worktree 提交）
- [x] T-10 修复实现（84befb240 + f509b47ff 双守卫）：①`computed_signal_store` 堆身份准入守卫 `value_carries_heap_identity`（VmRef/ValueRef/≥4M 整数形直载+容器递归+不可证净复合保守拒收→不入网退回每帧重算=pre-706 行为，plan047 退档同族零新特性）；②`deps_unchanged` 死对象 dep 键伪命中守卫 `heap_dep_key_alive`（值守卫只断 UAF panic 面；全矩阵实机复跑暴露第二面=死键版本冻结伪命中→陈旧标量/视图回流 [10] 首页钮缺失/case13 ECONNRESET；`<HEAP_ID_BASE` 合成键与未分配 id 按存活保 plan047 门测试语义）；家族单测 `plan706_signal_store_skips_heap_identity_value`/`plan706_signal_store_hit_cycle_idempotent_and_fresh`/`plan706_deps_unchanged_dead_heap_key_misses`（红相 gate-off 环境开关取证：pre-fix 红已录，开关已移除）
- [x] T-11 收口验证（9420eb7a2）：**消费者回执面** vm_matrix merged **16/16 ALL GREEN 双载体**（修复 exe debug+release；[7 tab] PASS+[10b] 无弹层滞留）；家族面 ui:: 域 1617 测 1616 绿（1 红=base 预存，old-carrier 同红复证）+ plan706 3/plan047 24/plan046 38/memo 86/vm_bridge 55 + 家族弧 CPU load 5/5 + **gallery 实机四页抽查 GREEN**（Sidebar/Menubar/Area Chart/Command 四路由真达+渲染非空 74821/55750/36659/35335）；probe_mtime release 挂起面**定谳独立缺陷**（old-carrier c8f86ef92 release 同挂——非本回归，AC-11 后半臂另立记账 P706-D1）+ 修复 exe debug 全案通过；消费者 e2e 预存 flaky 定谳（双载体同 flaky：old 2/4 / new 1/4 无区分度，非 r2 回归，P706-D2）

### 11.3 验收标准（追加）

- **AC-09**：bisect 定谳报告在档——肇事提交集 + 最小再现弧 + 触发机制初判；pre/post A/B 同 harness 同弧绿红分明。
- **AC-10**：修复后新 exe 下，jade-edit 消费面 `tests/vm_matrix.mjs` **merged 16/16 ALL GREEN**（[7 tab] 标脏弧与 [10b] 脏关弧 PASS；fail-snap 无双叠确认弹层）。
- **AC-11**：probe_mtime release 载体挂起面复测绿（或定谳为独立缺陷另立记账）；家族既有 UI 测试面零回归。

### 11.3+ AC 收口（2026-09-30 work 阶段）

- **AC-09 PASS**：定谳报告 `docs/plans/reports/706-bisect-verdict.md` 在档（first-bad=432068075 + 最小再现弧 + RC canary UAF 机制实证 + 死键伪命中第二面）；pre `c8f86ef92` GREEN / post RED 同弧同驱动（家族 tab-arc.mjs；A/B 另有消费者 p25/p26 在库）。
- **AC-10 PASS（定义性验收）**：修复 exe（plan-706-dev 9420eb7a2 构建）下 jade-edit vm_matrix merged **16/16 ALL GREEN**——debug 与 release 双载体独立全绿；[7 tab] PASS、[10b] 无弹层滞留。
- **AC-11 PASS（后半臂）**：probe_mtime release 挂起面定谳为**独立预存缺陷**（old-carrier c8f86ef92 release 同挂 → P706-D1 另立记账；修复 exe debug 全案绿）；家族 UI 测试面零回归（ui:: 1616/1617，唯一红为 base 预存且 old-carrier 同红；plan047/plan046/memo/vm_bridge 定向全绿）。注：T-11 验证列的「e2e 全绿」定谳为消费方 harness 预存 flaky（双载体同 flaky 无区分度，P706-D2）——非本回归面，不在本 phase 修复域。

### 11.4 域协调与边界

- **PLAN-709 在飞**（native slot interaction——aura_view_builder 域，drafting rev1 未动 src）：T-09/T-10 以 master 为基；若 709 先落 src，T-10 基其结果 rebase 重验（同域注意）。
- **plan047 memo 大宗候选保留**：vm_bridge +616 与断裂域同窗，bisect 不因 7b8a3ccbc 是本计划主提交而预设其肇事——按证据定谳；若定谳落 plan047 提交，修复仍在本 phase 收口（回归经 706 合并窗携入消费者）。
- **零产品特性扩张**：本 phase 仅还原交互语义；gallery 新能力面不在范围。
- 消费者等待态：jade-edit PLAN-026 blocked-waiting（r1 裁定选项 a）——**AC-10 已兑现（2026-09-30，merged 16/16 ALL GREEN debug+release）= 其重入信号**。
- r2 work 落地注记：PLAN-709 未动 src（本 phase 基 master 4dae03122 无 rebase 需求）；plan047 提交定谳为肇事（432068075）——修复按 §11.4 预设在本 phase 收口，plan047 交付语义不变（其单测面全绿）。

### stage: merge PLAN-706:r2
- `outcome: pass`
- `prepared`: reviewed 9420eb7a2（复审 pass c948a3b13）；r2 零规范增量（复审 grep 实证 docs/specs 无 memo/信号网契约文本；准入守卫=代码内已成文降级哲学同族）→ ledger 零新条目（no-impact 有据）
- `rebase 映射`: master 两度漂移两度重验——①b310acafd（709 archive 链+708 S 档落地）：84befb240→f6758fb1d / f509b47ff→b716082d0 / 9420eb7a2→af8690984，range-diff `=` 全等；合并态 scope 重跑全绿（plan706 3/plan047 24/plan046 38/memo 91[含 708 新测]/vm_bridge 55+双形态 check 0 error+arc 2/2+消费者矩阵 16/16 ALL GREEN）；②6e4e1d07a（PLAN-711 簿记）：f6758fb1d→ed81a3295 / b716082d0→3776035e3 / af8690984→612514d5a，range-diff `=` 全等（区间纯 docs 簿记，代码验证沿用①）
- `landed`: master tip **612514d5a**（ff-only，零合并提交）；main smoke cargo check 0 error
- `ledger_refreshed`: n/a——r2 零规范增量（无 canonical Specs 改动、无账本条目增删；r1 的 ADR-25/shadcn 传递依赖条目不受影响）
- `archived`: docs/plans/archive/706-gallery-vue-display-fixes.md；status archived（终态）；completion_kind delivered（r1+r2 双 phase）
- `cleaned`: wt-guard clean×3（auto-lang/auto-down/old-carrier）；worktree×3+old-carrier 移除、分支 plan-706-dev 删（was 612514d5a=landed tip）；组目录 .wt/lang-706/ 留 evidence+驱动脚本不入 git
- `deployment_observation`: master release auto.exe 未重建（r1 同惯例）；桌面下次启动现场构建拾取；jade-edit 消费者以 master 612514d5a 重建双 exe 即得修复载体（其 PLAN-026 重入信号已生效=AC-10）
- `findings 承接`: P706-D1..D4 已在 KNOWN-DEBT-AND-RISKS.md 在册（probe_mtime release 挂起/消费者 e2e flaky/gallery 侧栏 quirk/705 e2e 门禁违例）
