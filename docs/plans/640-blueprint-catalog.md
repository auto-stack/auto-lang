---
plan_id: PLAN-640
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: blueprint-catalog（官方默认 Blueprint 集 Tier 0 扩容 + gallery 自动化）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/blueprint/contract.md#Q5-kind-词表
new_spec_components:
  - docs/specs/blueprint/project.md#官方默认集判定标准
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [blueprint]          # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-640] blueprint-catalog —— 官方默认 Blueprint 集（Tier 0）扩容

## 0. 变更摘要

`blueprints/` 官方目录从 4 包扩容到 **13 包**（9 个新包 + `form/login` 增第三变体），
选型对齐业界 Block 策展共识（shadcn/ui 小而精官方集、Ant Design Pro 页面脚手架词汇、
PatternFly/SAP Fiori 的 pattern 级词汇：查询表格/设置/向导/空态三分法/主从/结果页/仪表盘）。
同时把 Block-Gallery 的三笔欠账清掉：手写 manifest（2/4 包）改为 **Vite `import.meta.glob`
磁盘自动发现**（零生成物、零手写同步）、残留 Block 字样与坏链清零、gallery 接入 CI。
spec 沉淀两件事：kind 词表与磁盘现实对齐（contract.md Q5 已漂移：列了 5 种 kind，
磁盘实有 7 种）、**官方默认集判定标准五条**成文（project.md）。

机制面（registry 扫描 / L1-L2-L3 三通道 / `auto bp` CLI 语义）**零改动**——本计划是
内容 + 展示层计划，仅新增测试断言。

## 1. 目标

- **G1 目录扩容（主体）**：9 个新包 + login `two_column` 变体，全部满足包格式
  （spec.md 全字段契约 + reference/<v>.at + gotchas.md），palette 零漂移。
- **G2 gallery 零同步**：`examples/bps-gallery` 的 `src/bps.ts` 从手写 2/4 包改为
  glob 派生全部磁盘包；加包/删包零手改；接入 CI build 门禁。
- **G3 术语与文档卫生**：gallery 标题等残留 "Block" 字样清零；`blueprints/README.md`
  旧命令（`auto blueprint`）、坏路径（`examples/blueprints-gallery`）、越仓坏链（`../docs/...`）修复。
- **G4 spec 沉淀**：kind 词表终版（含治理规则）、判定标准五条、模块清单表 4→13、mermaid 更新。

### 非目标

- 不改 bp 机制本体：registry 扫描语义、L1/L2/L3 通道、CLI 子命令语义均不动（只加测试）。
- 不做 Tier 1（`editor/chat`、command-palette、kanban、detail-profile…）与 marketing 轨
  （hero/pricing）——后续计划。
- **不向 WidgetRegistry 注册新 widget tag**：chart 族（area/bar/line/donut-chart）与
  `data-table` 的注册缺口登记 DEBT（见 T-01/AC-09），不在本计划内补。
- 不做 gallery 实时渲染（README Scope note 维持 deferred）。
- `auto bp export-gallery-meta`（Plan 343 展望）由 glob 方案取代，正式放弃并记录理由（§5.2）。

## 2. 架构方案

纯内容 + 展示层自动化，三层不变（Widget / Blueprint / App；`blueprints/<kind>/<name>/`
包格式、`BlueprintRegistry::scan_dir` 自动发现、`palette_drift` 自动门禁对新包免费生效）：

1. **新包即纯磁盘资产**：`spec.md`（frontmatter 用满 Q1/Q2 契约字段：`props`/`actions`/
   `dataSource`/`extension_points`/`variants`/`acceptance`——现有 4 包仅 login 用了
   `dataSource`，新包把契约面补全，作为"六问可答"的示范集）+ 每变体一个
   `reference/<v>.at` + `gotchas.md`（3 条 {wrong, why, right}）。
2. **gallery 自动发现**：`import.meta.glob('../../../blueprints/**', { query: '?raw',
   eager: true })` 一次抓 spec/gotchas/reference 全部原文，路径推导 `kind/name/variant`，
   `BpEntry` 接口不变（App.vue 零改动）。无生成物、无 CLI 参与、无同步面。
3. **测试面**：`scan_tests` 扩存在性断言 + 新增 `plan640_bp_tests.rs`（feature `ui-iced`
   门控，仿 `plan639_bp_tests.rs`：代表包 VM 轨 view 结构断言 + vue 轨 SFC 发射断言）。
4. **图表面裁定（已实地核验）**：`schema/aura.at:2422-2724` 有 chart 元素声明，但
   `apply_schema_vue_mappings`（`crates/auto-lang/src/ui_gen/widget/registry.rs:88`）
   只对 Rust 已注册 widget overlay vue 映射，chart tag 未在 Rust 注册 → palette 引用必
   漂移。故 `dashboard/overview` 的 palette **不含 chart**，图表区降为 extension_point
   `charts`（EDIT region，消费方在 app 层引 official 图表包）；DEBT 登记（AC-09）。

## 3. 技术栈

- 内容：Markdown spec（`+++` TOML frontmatter，`BlueprintSpec` 字段集见
  `crates/auto-lang/src/ui_gen/bp/spec.rs:14-39`）+ `.at` widget DSL。
- Rust：仅测试改动（`scan_tests` 扩展 + 新测试文件 + `lib.rs` 注册行）。
- 前端：Vite 5 `import.meta.glob`（`examples/bps-gallery`，Vue3 + TS + vue-tsc）。
- 验证：`cargo check -p auto-lang`、`cargo t plan640`（scoped）、`auto bp list/show/check`、
  gallery `pnpm build`、autoui-verifier 双端抽查。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-18 会话：用户基于调研结论（Tier 0 目录 10~12 包 + gallery 补课 + spec 沉淀）
  确认起草本计划。仓库=auto-lang 单仓；worktree 走 Plan 529 分组平铺
  （`D:/autostack/.wt/lang-640/auto-lang`）。
- 门禁档位：crates/ 仅测试文件改动 → **Category B**（`cargo check` + scoped `cargo t`，
  不跑 tf/taa）。

### 业界调研结论（2026-09-18 检索，选型依据）

- **shadcn/ui Blocks**（官方集仅 4 类：Dashboard/Sidebar/Login/Signup，~十几个块）：
  小而精策展 + 页面级完整组合（dashboard-01 = sidebar + 交互图表 + data-table +
  section-cards + header 六件文件组）。
- **Tailwind Plus**：三轨道（marketing / **application UI** / ecommerce）——AutoUI 是
  app 框架，主轴取 application UI 轨，marketing 块降为后续可选轨。
- **Ant Design Pro blocks**：页面脚手架词汇（查询表格/高级表单/列表/详情/结果页），
  强契约（ProTable/ProForm）——与 bp 的 props/actions/dataSource 契约面最接近。
- **PatternFly / SAP Fiori / Material**：pattern 级词汇——primary-detail（≥720px 两栏
  裁定）、empty state 三分法（first-use / no-result / error）、onboarding/wizard。
- 共同规律：官方集小而精（十几为限）、生态集大而全；块=自洽功能区块而非整页。

### 现状证据（路径实勘）

- 4 包现状：`blueprints/{form/login(minimal,with_sso), data-display/note-list,
  editor/note-editor, navigation/sidebar-nav(default,compact)}`。
- registry 测试无包数硬断言（`registry.rs:219-249`），新包自动纳入扫描与
  `palette_has_no_drift`；`scans_default_packages` 仅断言 2 个 key 存在。
- spec 契约字段已支持但存量包未用满：login 仅 `dataSource`，无 `props`/`actions`/
  `acceptance`。
- gallery 欠账：`examples/bps-gallery/src/bps.ts` 手写 2/4 包；`kindOrder` 缺
  navigation/editor；`index.html:6` 标题 "AutoUI Blocks — gallery"；不在任何 CI。
- `blueprints/README.md`：`auto blueprint list`（弃用名）、`examples/blueprints-gallery`
  （坏路径，实为 `examples/bps-gallery`）、`../docs/design/blueprints/...`（越仓坏链，
  应为 `docs/...`）。
- kind 词表漂移：`docs/specs/blueprint/contract.md` Q5 列
  form/data-display/feedback/layout/composite，磁盘实有 navigation、editor。
- palette tag 核验（2026-09-18）：tabs/select/switch/alert-dialog/dialog/table/
  pagination/dropdown-menu/sidebar/avatar/header/callout/skeleton/list/card 及 chart 四
  tag 在 `schema/aura.at` 均有声明；`data-table`、`form-field` **无** aura 映射——
  palette 回避（用 `table`、`form` 族名）。

## 5. 详细设计

### 5.1 Tier 0 新包清单（9 新包 + 1 新变体）

palette 为候选集（以 `palette_has_no_drift` + aura 映射为准，执行时按 registry 实名微调；
palette_drift 接受 exact tag 或 `<tag>-*` 前缀族）：

| # | kind/name | 业界锚点 | palette（候选） | variants | 契约草案（props / actions / dataSource） |
|---|---|---|---|---|---|
| 1 | `form/signup` | shadcn Signup | button, checkbox, input, separator | minimal | actions: `submit`；dataSource: `register(creds) -> Account` |
| 2 | `form/settings` | 各家通用 | form, tabs, switch, select, input, button, separator, alert-dialog | default | props: `sections`；actions: `save`, `reset`；危险区走 alert-dialog 确认 |
| 3 | `form/wizard` | Material onboarding / AntD 高级表单 | button, input, text, separator, badge | default | props: `steps`；actions: `next`, `back`, `finish`；分步校验状态机 |
| 4 | `dashboard/overview` ⭐ | shadcn dashboard-01 | sidebar, table, card, badge, skeleton, text, button, image | default | dataSource: `metrics() -> Stats`；extension_points 含 `charts`（EDIT region，图表归 app 层 official 包，见 §2.4） |
| 5 | `data-display/data-table-crud` ⭐ | AntD 查询表格 | table, input, select, dialog, button, badge, pagination, dropdown-menu | minimal, with_dialog | dataSource: `query(filter, page) -> Rows`；actions: `create`, `update`, `delete`（行操作走 dropdown-menu） |
| 6 | `data-display/master-detail` | PatternFly primary-detail / Win 720epx | list, input, separator, badge, skeleton, text, button | default | props: `breakpoint`；dataSource: `fetch_list(q)`, `fetch_detail(id)`；空选中态为显式状态 |
| 7 | `feedback/empty-state` ⭐ | Fiori 三分法 | text, button, image, callout, separator | first_use, no_result, error | actions: `primary`；三分法直接落为变体 |
| 8 | `feedback/result-page` | AntD 结果页 | callout, button, badge, separator, text | success, error | actions: `primary`, `secondary`；403/404/500 语义并入变体正文 |
| 9 | `navigation/sidebar-shell` | shadcn Sidebar（页面级） | sidebar, header, avatar, dropdown-menu, separator, badge, button, text | default, compact | props: `nav_tree`, `user`；与既有 `sidebar-nav`（三段式导航内容）互补：本包是 app 壳（header + sidebar + 内容槽 + user menu） |
| 10 | `form/login` 增变体 | shadcn login-04 | （不变） | + two_column | spec `variants` 追加 + `reference/two_column.at`（图文分栏） |

⭐ = 最高优先。每包含 `gotchas.md`（3 条 {wrong, why, right}，首条通用反例精神同
login："Baking the endpoint into the blueprint"——dataSource/动作不得烧死在包内）。
存量 4 包不做回改（login 只加变体；note-list/note-editor/sidebar-nav 保持现状，
契约字段补全留待后续自然演进——避免本计划 scope 膨胀）。

### 5.2 gallery 自动发现设计（取代 export-gallery-meta）

`src/bps.ts` 重写为派生模块：

- `import.meta.glob('../../../blueprints/*/*/spec.md', { query: '?raw', import: 'default', eager: true })`
  推导包集合（路径给 kind/name）；reference 与 gotchas 各一个 glob，按路径归并。
- `BpEntry` 接口与导出名 `bps`/`kindOrder` 不变 → `App.vue` 零改动。
- `kindOrder` = 固定偏好序（form, navigation, dashboard, data-display, feedback, editor,
  layout, composite）∪ 其余发现项字母序兜底。
- gotchas 缺失文件渲染 "—"（registry 不强制）。
- **取代理由**（记录进 gallery README）：Plan 343 展望的 `auto bp export-gallery-meta`
  会引入"生成物同步"这一新漂移面；glob 方案零生成物零同步，registry CLI（`auto bp list`）
  仍是校验态目录的权威视图。
- 顺手项：`index.html:6` 标题改 "AutoUI Blueprints — gallery"；gallery README Scope note
  补 glob 机制说明。
- CI：新增 `.github/workflows/build-bps-gallery.yml`（镜像 `build-vue-gallery.yml`：
  paths 过滤 `examples/bps-gallery/**`、`blueprints/**`、workflow 自身；`pnpm install +
  pnpm build` 走 vue-tsc 门禁）。

### 5.3 测试设计（详见 §6）

`scan_tests` 扩展存在性断言（9 新 key + login 三变体）+ `plan640_bp_tests.rs`
（全包结构/契约断言 + 3 个代表包双轨断言）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/blueprint/contract.md（Q5 kind 词表） | before：kind = form/data-display/feedback/layout/composite；after：kind = form/navigation/dashboard/data-display/feedback/editor/layout/composite + 治理规则（新增 kind 须随 spec 沉淀并更新 gallery kindOrder 偏好序） | 词表已与磁盘漂移（navigation/editor 在盘未入表）；本计划新增 dashboard | AC-06 |
| SD-02 | modify | docs/specs/blueprint/project.md | before：模块清单 4 行、无判定标准；after：清单 13 行（目标态）+ mermaid 更新 + 新增"官方默认集判定标准"五条（①跨 app 出现频率 ②契约可成文 props/actions/dataSource 非平凡 ③palette ⊆ WidgetRegistry 且双端可发射 ④状态机含量 ≥三态 ⑤NL spec 可描述、可被 agent 组装） | 判定标准是目录持续扩容的准入门槛，防止官方集膨胀退化 | AC-06 |
| SD-03 | modify | blueprints/README.md + examples/bps-gallery/README.md | before：`auto blueprint list` 旧名、`examples/blueprints-gallery` 坏路径、`../docs/...` 越仓坏链；after：`auto bp list`、实路径、仓内相对链接 | 术语卫生（639 更名收尾） | AC-05 |

（无 retire 项。）

## 6. 测试设计

- **scan_tests 扩展**（`crates/auto-lang/src/ui_gen/bp/registry.rs` 内）：
  `scans_default_packages` 增 9 个新 key 存在断言；`login_has_two_references` 改名
  `login_has_three_references`（minimal/with_sso/two_column）。
- **plan640_bp_tests.rs**（新文件，`crates/auto-lang/src/plan640_bp_tests.rs`，
  `lib.rs` 仿 639 注册，feature `ui-iced` 门控）：
  - 全包断言：13 包 spec name↔目录一致、变体文件齐、gotchas 存在；9 新包
    `props`/`actions`/`dataSource`/`extension_points` 非空（契约完整面）。
  - 代表包 VM 轨（仿 plan639 login "Sign in/Email/Remember me" 模式）：
    `form/signup`、`data-display/data-table-crud`、`feedback/empty-state` 的
    reference 经 VM 管道后 view 结构断言（关键文案/节点存在）。
  - 代表包 vue 轨：同 3 包 SFC 发射断言（组件 import 与结构）。
- **CLI 冒烟**：`auto bp list`（13 包按 kind 分组）、`auto bp show <每包>`、
  `auto bp check`（全包绿）。
- **gallery**：`pnpm build`（vue-tsc + vite）绿；`pnpm dev` 手动走查 13 包 + login
  3 变体可见、spec/gotchas/源码高亮正常。
- **双端抽查（复审档）**：autoui-verifier 双端模式走查 ≥1 代表包
  （Vue 轨 Playwright DOM 断言 + VM 轨 snapshot），截图留档 `tests/screenshots/`。

## 7. 验收标准

- **AC-01 目录与 registry**：13 包全部通过 registry 扫描（spec name↔目录一致、
  声明变体必有 reference 文件、palette 零漂移）。验证：`cargo t plan640` 绿
  （含 scan_tests 扩展）；`auto bp list` 列出 13 包。
- **AC-02 新包契约完整**：9 新包 frontmatter 声明 `props`/`actions`/`dataSource`/
  `extension_points`/`acceptance` 非空，正文含 Intent / absorbs / guidance 节
  （六问可答）。验证：复审 checklist audit + `auto bp show <key>` 逐包核对。
- **AC-03 代表包双轨断言**：signup / data-table-crud / empty-state 的 VM 轨 view
  结构断言与 vue 轨 SFC 发射断言绿（plan640_bp_tests）。
- **AC-04 gallery 自动化**：glob 派生全部磁盘包（13 包、login 3 变体），
  bps.ts 零手写条目；`pnpm build` 绿；`.github/workflows/build-bps-gallery.yml`
  存在且路径过滤含 `blueprints/**`。
- **AC-05 术语与链接卫生**：`grep -ri "block" examples/bps-gallery/{src,index.html}`
  零命中（CodeBlock 类标识豁免）；`blueprints/README.md` 命令为 `auto bp *`、
  三个坏链全部指向实存路径（复审抽查点开）。
- **AC-06 spec 沉淀**：SD-01/SD-02 落地（contract.md kind 词表 + 治理规则；
  project.md 判定标准五条 + 13 行清单表 + mermaid）；merge 时 specs.json upsert
  + `python scripts/spec-index.py`。
- **AC-07 CLI 冒烟**：`auto bp list/show/check` 对全部 13 包工作正常。
- **AC-08 双端抽查留档**：≥1 代表包 autoui-verifier 双端走查通过，截图存档。
- **AC-09 chart 缺口成账**：`docs/plans/KNOWN-DEBT-AND-RISKS.md` 新增 DEBT 行
  （chart 四 tag 与 `data-table` 未入 WidgetRegistry；dashboard 图表面走
  extension_point `charts` 的裁定记录；后续若注册需另立计划）。

## 8. 执行步骤

> 全部在 worktree `D:/autostack/.wt/lang-640/auto-lang`（分支 `plan-640-dev`）执行；
> plan 簿记（[✅] 标记/frontmatter）留在主检出。

- **T-01 [有界调查] chart tag 注册状态终验**（0.5h）
  依赖：无。影响：调查记录（本文件 §5.1 表注 + KNOWN-DEBT 草稿）。
  操作：跑最小探针（测试或 `auto bp check` 临时 palette 试验）确认 chart 四 tag 在
  `WidgetRegistry::with_defaults()` 的注册状态，复核 Plan 484 注释语义
  （registry.rs:2162-2164）。产出：dashboard palette 终版裁定（预期=不含 chart）+
  DEBT 行草稿。验证：探针输出记录于本节。→ AC-09（及 AC-01 的 palette 终版）
- **T-02 spec 词表与判定标准先行**
  依赖：无。影响：`docs/specs/blueprint/contract.md`（Q5）、
  `docs/specs/blueprint/project.md`（判定标准五条 + 13 行目标态清单表 + mermaid）。
  验证：文档 diff 审读；kind 词表覆盖磁盘全部 kind。→ AC-06
- **T-03 form 批 A**：`blueprints/form/signup/{spec.md, reference/minimal.at, gotchas.md}`
  + `blueprints/form/login/spec.md`（variants 追加 two_column）+
  `blueprints/form/login/reference/two_column.at`。
  验证：`auto bp show form/signup`、`auto bp check form/signup form/login`。→ AC-01/02
- **T-04 form 批 B**：`form/settings`、`form/wizard`（同构三件套）。
  验证：`auto bp check form/settings form/wizard`。→ AC-01/02
- **T-05 data 批**：`data-display/data-table-crud`（minimal + with_dialog 两变体）、
  `data-display/master-detail`。验证：`auto bp check data-display/*`。→ AC-01/02
- **T-06 feedback 批**：`feedback/empty-state`（first_use/no_result/error 三变体）、
  `feedback/result-page`（success/error 两变体）。验证：`auto bp check feedback/*`。→ AC-01/02
- **T-07 navigation + dashboard 批**：`navigation/sidebar-shell`（default/compact）、
  `dashboard/overview`（default，palette 按 T-01 终版）。
  验证：`auto bp check navigation/sidebar-shell dashboard/overview`。→ AC-01/02
- **T-08 gallery 自动化 + 卫生 + CI**：重写 `examples/bps-gallery/src/bps.ts`（glob 派生）；
  `index.html` 标题；`examples/bps-gallery/README.md`（glob 机制 + Scope note）；
  `blueprints/README.md`（SD-03：命令/路径/链接）；新增
  `.github/workflows/build-bps-gallery.yml`。
  验证：`cd examples/bps-gallery && pnpm install && pnpm build` 绿；dev 走查 13 包。→ AC-04/05
- **T-09 测试面**：`registry.rs` scan_tests 扩展；新
  `crates/auto-lang/src/plan640_bp_tests.rs` + `lib.rs` 注册（feature `ui-iced`）。
  验证：`cargo check -p auto-lang` 零警告；`cargo t plan640` 绿。→ AC-01/03
- **T-10 门禁与抽查（review 前兜底）**：`auto bp list/show/check` 全量冒烟；
  autoui-verifier 双端抽查（empty-state 或 data-table-crud）截图存档；
  `KNOWN-DEBT-AND-RISKS.md` 落 DEBT 行（T-01 草稿定稿）；
  复核门禁档位（Category B：不触发 tf/taa）。
  验证：命令输出留档本节。→ AC-07/08/09

依赖链：T-07 依赖 T-01；T-03..T-07 依赖 T-02（kind 词表）；T-08/T-09 依赖 T-03..T-07；
T-10 最后。T-03/T-04/T-05/T-06 相互独立可乱序。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（两项，均有默认裁定，不阻塞开工）。

## 10. 待澄清事项

1. **chart 图表面**（默认裁定已给，如需 dashboard v0 内置真图表则扩 scope）：本计划
   走 extension_point + DEBT；若用户希望随本计划注册 chart 四 tag 进 WidgetRegistry
   （触及 Plan 484 语义），需明确批准（建议另立小计划）。
2. **kind `dashboard` 新增 vs 归 `composite`**：默认新增 `dashboard/`（对齐 shadcn
   Dashboard 一级类目，词表 delta 已含）；如倾向复用 `composite` 则 T-02 微调
   （仅词表与目录名，不影响其余任务）。
3. （记录性，无需裁决）`form/wizard` 的步进指示 v0 用 badge 组合实现，不新增
   stepper widget——若后续体验不足，走变体提升评审。
