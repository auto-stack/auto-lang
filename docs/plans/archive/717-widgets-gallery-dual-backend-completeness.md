---
plan_id: PLAN-717
status: archived
feature_name: widgets-gallery-dual-backend-completeness
author: [agent]
created_at: 2026-09-30T00:00:00+08:00
updated_at: 2026-10-01T12:10:55+08:00
plan_revision: 5
current_step: 8
total_steps: 8
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/widgets-gallery-parity.md"
touched_goals: [GOAL-007]
affects: [auto-lang/ui, auto-os/widgets-gallery, auto-down/autodown-engine]
---

# [PLAN-717] widgets-gallery 双端全量示例展示收敛

## 0. 变更摘要

对 widgets-gallery 的 Vue 与 VM 两个后端做完整页面和示例核查，修复导致组件缺席、示例空白、内容裁切或页面无法打开的问题。以 gallery 源路由和每页的 preview-card 示例为验收清单，逐页记录 Vue/VM 截图、问题归因和复验结果。若 gallery 直接依赖的组件导致页面挂载失败，允许在同组 auto-down worktree 修复该最小依赖缺陷。

PLAN-706 已交付 Vue 首轮问题及 VM 四页抽查；PLAN-711 将“至少 20 个样本、5 页全扫”留作后续导向波。本计划完成全量收口，不重开或改写已归档计划。

## 1. 目标

- G1：为当前 gallery 冻结完整页面、路由和 preview-card 示例清单；清单覆盖全部源页面，孤立页面和缺失路由均有明确结论。
- G2：清单中的每页在 Vue 和 VM 后端均可到达，标题、组件示例、属性说明及代码示例按页面设计出现。
- G3：每页中的所有示例在两端均可见；长页面滚动后也没有空白示例、意外裁切、遮挡或不可读内容。
- G4：画廊支持的明暗主题均通过全页显示核查；对交互后才出现的示例状态，执行对应的展开、选择或输入动作并留证。
- G5：所有缺陷按 Vue 生成、VM 渲染、共享 AutoUI 管线或 gallery 语料分类并修复；无未解释的页面错误、组件缺席或回归。
- G6：保存逐页核查矩阵和双端截图，并把可复用验收契约沉淀到规范。

### 非目标

- 不改写 widgets-gallery 的产品信息架构或新增组件。
- 不做与显示和示例可达性无关的深层业务行为测试。
- 不扩展到 auto-os 中其他应用；发现的跨应用通用缺口只登记为后续债务候选。

## 2. 架构方案

以 auto-os 中 widgets-gallery 的源码路由和页面内容为事实源；以 auto-lang 的生成器、schema、VM 视图构建与 iced 渲染为后端行为源。先比对路由文件、预览示例及既有临时走查证据，再按故障所在层修复，最后用相同清单逐页跑 Vue 和 VM。

| 层 | 责任范围 | 主要落点 |
|---|---|---|
| Gallery 语料 | 路由、页面结构、示例组件、文档和示例状态 | auto-os: widgets-gallery/src/front/app.at、pages/**、components/**、pac.at |
| Vue 后端 | Vue SFC 发射、组件映射、脚手架依赖闭包 | auto-lang: ui_gen/vue.rs、auto-man/vue_shadcn.rs、schema/aura.at |
| VM 后端 | 组件构建、布局、主题和屏幕绘制 | auto-lang: aura_view_builder.rs、ui/iced/** |
| 页面依赖 | gallery 实际依赖的编辑器生命周期与组件挂载 | auto-down: autodown/packages/engine/src/editor/menus/CodeBlockMenu.vue |
| 验收与规范 | 全路由双端矩阵、截图证据、稳定验收规则 | autoui-verifier 标准驱动、docs/plans/reports/717-widgets-gallery-matrix.md、widgets-gallery-parity.md |

修复以 T-00 归因报告为边界；不预设某一个组件或某一层是全部问题的根因。双端按组件含义、示例内容、可读性、布局完整性和交互可见性核对，并对支持的明暗主题分别验收。

## 3. 技术栈

- 仓库：auto-lang Rust workspace、auto-os widgets-gallery Auto 源码与前端资产。
- 后端：Vue 3/Vite/shadcn-vue；AutoUI VM/Iced。
- 驱动：.agents/skills/autoui-verifier/scripts/test_vm_mcp.py 与 test_vue_playwright.mjs；优先复用标准客户端和脚本。
- 页面资产：widgets-gallery/src/front/app.at、pages/*.at、components/*.at 与 pac.at。
- 报告：双端页面/示例矩阵 Markdown；逐页截图存放于 worktree 的忽略截图目录，交付前生成可浏览的总览图。

## 4. 需求分析与背景调查

### 授权记录

- 用户于 2026-09-30 提出目标：“widgets-gallery的Vue版和VM版的所有组件均显示正确，且示例都能显示出来。”
- 本计划范围为 auto-lang、auto-os 中的 gallery 及其必要的 AutoUI 生成/渲染行为；允许在计划 worktree 中修改相关源码、增加验收覆盖、运行相应验证并更新规范。
- 仓库 AGENTS.md §1.1 L1 要求先提交计划供用户确认，再执行实现。当前授权覆盖目标、调查和计划起草；代码实施在用户确认本计划后开始。
- 用户未给出时间、算力或自动续跑预算。

### 背景与现状证据

- 执行基线：auto-lang HEAD 57b9afa60；auto-os HEAD 0d5f5bf；只读构建依赖 auto-down HEAD 3373a5cc（主检出 clean）。auto-lang 的 Cargo.toml 通过相对路径要求同组 auto-down checkout。auto-lang 与 auto-os 主检出均有未提交工作，包含 VM 走查材料；auto-os 还修改了桌面脚本、测试素材和 widgets-gallery/.auto/ui-cache.json。计划 worktree 从提交基线建立，不携带这些未提交状态；主检出文件保持不动。
- auto-os/widgets-gallery/src/front/pages 当前有 70 个 .at 文件（包含 index.at），app.at 当前有 70 条路由并与 70 个页面文件一一映射。初始 index.at 有 68 个 component-card，app.at 侧栏有 69 个路由链接；两处都遗漏 /kitchen-sink。全目录有 159 个 preview-card 标记，index 与 kitchen-sink 页面没有 preview-card；旧版 Kitchen Sink 另列 54 个 schema 标题，其中 `scroll_test_content` 是 10,000,000px 的非公开测试夹具。当前生成器排除该夹具，故公共 Kitchen Sink 示例共 53 个。
- 主检出中的旧 VM 图片拼图将 Command/Combobox 标签与 AreaChart 内容放在同一画面，提示旧走查可能存在路由错配；这是临时文件里的间接证据，不作为当前失败或通过结论。仓内未提交的 .tmp-vm-gallery-walk.py 也只作调查线索；新基线须确认实际路由和页面标题后再计数。
- PLAN-706 已归档：Vue 首轮修复覆盖多个生成、语料和依赖问题；VM 实机只抽查四页。PLAN-711 的 gallery 导向波报告注明“至少 20 样本 × 5 页全扫”为后续项。
- 可参考：docs/plans/archive/706-gallery-vue-display-fixes.md、docs/plans/reports/711-runtime.md、docs/specs/auto-lang/ui/overview.md、docs/design/autoui/base-styles-and-visual-parity.md，以及当前 widgets-gallery 源文件。
- 取证限制：现有临时截图、脚本和 VM 输出均留在用户主检出；新基线及最终证据须在两个计划 worktree 中重建，不能依赖未提交文件。
- 2026-10-01 revision 4：更新 Kitchen Sink 样例并在 Vue 实机复验时，`autodown_editor` 内的 `CodeBlockMenu.vue` 在父编辑器 DOM 接入 document 前读取 `editor.view.dom.closest(...)`，使 `/kitchen-sink` 抛出 TypeError。填入示例内容仍可复现，故将修复边界扩到 gallery 直接依赖的 auto-down 编辑器菜单生命周期；修复限于挂载完成后查找 DOM，并安全处理 host 尚不可用。auto-down 基线 `3373a5cc6e3a00336613133db51906fb0940777d`，继续使用同组 worktree `D:/autostack/.wt/lang-717/auto-down`，将其从只读 Cargo 依赖切换为 `plan-717-dev` 实施 worktree；不修改 auto-down 主检出。

## 5. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-lang/ui/design/widgets-gallery-parity.md | before：gallery 由多个历史计划分别记录，缺少 Vue/VM 全页面与每个示例的统一验收规则 / after：以源码路由和 preview-card 清单为准，要求双端全页面、所有示例、明暗主题及可见交互状态留矩阵和截图证据 | 使“所有组件和示例均显示”可重复核查，避免只抽查少数页面 | AC-01..AC-07 |

### 任务设计

#### T-00 建立路由与示例基线

读取 auto-os/widgets-gallery/src/front/app.at、pages/*.at、components/*.at 和 pac.at，逐项核对 route、源页面、preview-card、示例控件与文档区。用标准 VM/Vue 驱动获取现状快照，将结果写入基线矩阵并把缺陷归为 Vue、VM、共享管线、gallery 语料或验证环境问题。基线记录全部页面，即使无法启动也要留下失败原因。

#### T-01 修复 Vue 路径缺陷

只修复 T-00 证据明确归因于 Vue 发射、组件映射、shadcn 脚手架或生成产物的问题。优先核对 crates/auto-lang/src/ui_gen/vue.rs、crates/auto-man/src/vue_shadcn.rs 与 schema/aura.at；具体符号以基线报告中的复现和当前代码为准。

#### T-02 修复 VM 路径缺陷

只修复 T-00 证据明确归因于 VM 的组件构建、属性消费、布局、主题或屏幕绘制的问题。核对 crates/auto-lang/src/ui/aura_view_builder.rs 与 crates/auto-lang/src/ui/iced/**，以复现页面和结构/截图证据定位精确符号。

#### T-03 修复 gallery 源页面与示例缺口

修复 T-00 确认为路由、Auto 源语法、页面内容、预览卡片、图标/资源或示例状态设置的问题。仅编辑 auto-os/widgets-gallery 中相关源文件；所有页面和组件案例都须保留在最终冻结清单中。

#### T-04 完成可复用的全量双端检查入口

复用 autoui-verifier 的标准 MCP/Playwright 客户端，形成 gallery 全路由运行入口或扩展其标准脚本。对每个路由记录导航结果、页面文本/组件信号、preview-card 数量、控制台或 VM 错误、截图路径及主题。脚本应从源路由清单生成检查项，不维护第二份手写路由表。

#### T-05 完成全量显示与明暗主题验收

在 Vue 与 VM 模式下分别逐页遍历冻结清单；长页面滚动检查所有 preview-card。对每页在支持的明、暗主题下留截图；对初始态不可见的组件示例，按页面入口动作展开后补截图。核对内容、裁切、遮挡、可读性和布局，记录预期差异与修复后的证据。

#### T-06 规范、门禁和复审准备

新增 SD-01 规范及 overview 索引引用；生成 docs/plans/reports/717-widgets-gallery-matrix.md；按实际触及代码运行分级门禁；逐项核对 AC 并记录已解决问题和债务候选。

## 6. 测试设计

- 全页功能面：标准 Playwright/Vue 驱动和 VM MCP 驱动从源路由表构建同一页面清单；每个页面须可导航、标题可见、主内容存在、全部 preview-card 可逐屏检查。
- 截图面：Vue 与 VM 对每页、每个受支持主题各留完整可审阅截图；对页面较长或组件依赖展开状态的情形补滚动/动作后截图。总览图用于快速人工复核。
- 错误面：Vue 捕获构建失败、页面异常、console error 与未解析依赖；VM 捕获生成/编译/运行错误、异常空页和截图无有效内容。任一行失败均不得记为 pass。
- 交互显示面：每个交互组件族至少验证其 gallery 示例能到达一个声明状态；逐页导航本身也须成功。深层业务逻辑不作为本计划验收。
- 页面直接依赖的编辑器菜单：挂载阶段无运行时异常；编辑器 DOM 可用后菜单绑定成功并保持原有定位/交互。修复仅覆盖阻断 gallery 页面的生命周期问题。
- Rust 改动迭代：cargo check -p auto-lang；按实际模块运行 cargo t <module>。改 UI 代码生成时复审运行 cargo tu；改 VM/编译器时运行 cargo tv。
- 复审门禁：若修改 crates/，运行裸 cargo t；ui_gen 改动加 cargo tu，VM/编译器改动加 cargo tv。仅改文档生成器、Schema 定义或语法参考时运行 cargo test -p auto-lang --test docs_gen。cargo tf 按批量回归到期规则在主检出单实例处理，不作为本计划的 per-plan 门禁。AAVM 不在预期触发范围内。
- 纯页面/素材修复不运行 cargo t 或 docs_gen；只执行完整的 Vue/VM gallery 验收。

## 7. 验收标准

- **AC-01 页面与入口清单完整**：报告列出源路由、页面文件、页面标题、Home 卡片、侧栏入口和每页的 preview-card/特殊示例；每个面向用户的页面均可从 gallery 导航入口到达，特殊页也有清楚入口。初始值为 70 个页面文件/路由，Home 68 卡、侧栏 69 链接，/kitchen-sink 两处缺席；完成值为 70 个页面/路由、Home 69 卡、侧栏 70 个入口，/kitchen-sink 已在两处出现。验证：源路由、页面、Home 和侧栏清单逐项对账，遗漏数为 0。
- **AC-02 Vue 全页可展示**：冻结清单内每个 route 均可打开；页面标题、组件示例、属性说明和代码区存在；没有页面级空白、缺失组件、未解析依赖或运行异常。验证：全页 Vue 驱动矩阵全部 pass，失败行数为 0。
- **AC-03 VM 全页可展示**：与 AC-02 同一清单中的每页均可从导航到达并正确绘制；无生成/运行异常、整页空白或示例组件缺席。验证：全页 VM MCP 矩阵全部 pass，失败行数为 0。
- **AC-04 每个示例均可见**：每页全部 preview-card 示例以及 kitchen-sink 中当前 53 个公共 schema 示例，都能在初始屏或滚动/页面声明动作后看到；不计入已标明的非公开 `scroll_test_content` 测试夹具。没有意外裁切、遮挡、不可读文字或只有标题没有示例的区域。Vue/VM 的示例数量与含义相符，页面中的有意后端差异有记录。验证：矩阵逐示例勾核并附截图。
- **AC-05 主题与状态可核**：每个路由的明暗主题截图齐全；需要交互才能展示的组件示例至少有一组状态截图；可见控件的基本尺寸、颜色、层级和布局符合 base-styles-and-visual-parity 及对应组件规范。验证：双端对照审阅记录无未解释差异。
- **AC-06 回归门禁通过**：按本计划改动范围，适用的 cargo check、模块档、cargo t、cargo tu、cargo tv 或 docs_gen 均符合 AGENTS.md；本计划新增失败数为 0。已在原始基线复现的三项 `cargo t` 与一项 `cargo tu` 金样失败单独记录；gallery Vue/VM 全量矩阵全绿；构建无新增未处理警告、无遗留调试输出。
- **AC-07 规范与证据完整**：widgets-gallery-parity.md、overview 索引引用和全页矩阵报告齐全；每一项缺陷都有归因、修复位置、验证结果或登记债务候选。

## 8. 执行步骤

| # | 任务 | 依赖 | 落点（核实路径） | 产出/意图 | AC | 验证（预期） |
|---|---|---|---|---|---|---|
| 0 | T-00 路由、页面与示例基线 | — | auto-os/widgets-gallery/src/front/app.at、pages/**、components/**、pac.at；auto-lang/.agents/skills/autoui-verifier/scripts/** | 冻结 70 条路由/70 个页面、68 个 Home 卡、69 个侧栏入口、preview-card 与 kitchen-sink 示例清单；双端标题核实并产出逐页基线报告 | AC-01 | 每个 route、入口和示例逐项对账；页面身份核验通过 |
| 1 | T-01 Vue 缺陷修复 | T-00 | crates/auto-lang/src/ui_gen/vue.rs、crates/auto-man/src/vue_shadcn.rs、schema/aura.at（仅在证据命中时） | 生成产物和组件依赖正确，复现页恢复展示 | AC-02/04 | 受影响 Vue 页面复跑全绿；适用 scoped t 通过 |
| 2 | T-02 VM 缺陷修复 | T-00 | crates/auto-lang/src/ui/aura_view_builder.rs、crates/auto-lang/src/ui/iced/** | VM 组件、属性与布局绘制恢复正确 | AC-03/04/05 | 受影响 VM 页面复跑全绿；适用 scoped t 通过 |
| 3 | T-03 gallery 源缺陷修复 | T-00 | auto-os/widgets-gallery/src/front/**、widgets-gallery/pac.at | 路由、语料或示例内容恢复；为 kitchen-sink 这类特殊示例页提供明确的导航入口 | AC-01/02/03/04 | 对应页面两端复跑全绿；特殊页可从导航到达 |
| 4 | T-04 复用式全量检查入口 | T-00 | auto-lang/.agents/skills/autoui-verifier/scripts/** 或 widgets-gallery 现有测试目录 | 从源路由生成双端检查项，统一记录截图、主题、错误与示例计数 | AC-01/02/03/07 | 同一路由表驱动 Vue 和 VM；重复运行结果稳定 |
| 5 | T-05 双端全量验收 | T-01..T-04 | 两个 worktree 的 widgets-gallery；忽略截图目录；docs/plans/reports/717-widgets-gallery-matrix.md | 完成所有页面、示例、主题与必要交互状态的最终矩阵及总览图 | AC-01..AC-05 | 全矩阵零失败；所有行有截图和核查记录 |
| 6 | T-06 规范、门禁与复审准备 | T-05 | docs/specs/auto-lang/ui/design/widgets-gallery-parity.md、ui/overview.md、docs/plans/reports/717-widgets-gallery-matrix.md、.autoos/specs.json | 沉淀 SD-01 与 P717-1，更新规范索引并记录风险/债务 | AC-06/07 | 规范回读正确；适用门禁通过；差异审计零遗漏 |
| 7 | T-07 修复编辑器菜单 DOM 挂载时序 | T-00 | auto-down: autodown/packages/engine/src/editor/menus/CodeBlockMenu.vue | 子组件生命周期内延后查找编辑器 DOM，并在 host 未就绪时安全等待；避免 Kitchen Sink 编辑器示例使整页崩溃 | AC-02/04/06 | Vue `/kitchen-sink` 无 pageerror；菜单绑定和相关 auto-down 定向门禁通过 |

实施位置：auto-lang 工作树 D:/autostack/.wt/lang-717/auto-lang；auto-os 工作树 D:/autostack/.wt/lang-717/auto-os；auto-down 实施工作树 D:/autostack/.wt/lang-717/auto-down（切换到 branch plan-717-dev，基于 clean HEAD 3373a5cc）。同组目录内不创建 junction 或 symlink。主检出中已有的用户改动与临时走查材料保持原样。

## 9. 复审记录

- 2026-09-30 起草 handoff：stage=new，PLAN-717，plan_revision=1。outcome=pass：两仓边界、全页面验收面、跨后端分层和验证要求均已成文；70 个页面文件与旧临时路由清单的差异显式交由 T-00 对账；既有主检出改动保持不动。next=work（须先按 AGENTS.md §1.1 L1 第 2 步取得用户对计划的确认）；覆盖 T-00..T-06、AC-01..AC-07、SD-01。
- 2026-10-01 revision 2：T-00 静态对账确认 70 route ↔ 70 页面；Home/侧栏分别少 kitchen-sink 一处；159 个 preview-card 之外另有 kitchen-sink 的旧版 54 个 schema 标题。完成期核对发现其中一个是 `scroll_test_content` 非公开测试夹具；当前生成器排除后公共示例数为 53，双端四主题均逐项核对可见。旧拼图显示 Command/Combobox 标签与 AreaChart 内容错位，现只作疑点，执行期须实时复现确认。修订 AC-01/04 与 T-00/T-03，不缩小目标；worktree 基线保持 57b9afa60 / 0d5f5bf。
- 2026-10-01 execution handoff：用户继续了同一 active goal；按该授权开始 PLAN-717 work，status=executing。auto-lang 与 auto-os 的当前代码 WIP 已向用户说明，并隔离在主检出；任何 WIP 均不带入分支，合并前需再次核对归属。
- 2026-10-01 revision 3：cargo build -p auto 在 auto-lang worktree 失败，证据显示工作区要求相对路径依赖 auto-down。auto-down 主检出 clean@3373a5cc，新增同组只读依赖 worktree 以满足既有 manifest 路径，不扩大代码改动范围。
- 2026-10-01 revision 4：Vue 浏览器实测确认 auto-down 菜单初始化在父 DOM 挂载前执行；此依赖缺陷阻断 Kitchen Sink 和其 53 个公共示例，因此增加 T-07，并将 auto-down 作为本计划第三个实现仓。
- 2026-10-01 revision 5：T-00..T-07 已执行。Vue/VM 各 70 条路由明暗主题矩阵全绿，159 个 preview-card 和四种模式下的 53 个 Kitchen Sink 公共 schema 示例均可见；报告与总览图写入 `docs/plans/reports/717-widgets-gallery-matrix.md`。`cargo tv` 162/162、`docs_gen` 4/4 通过；`cargo t` 的三项失败与 `cargo tu` 的桌面金样失败均在代码改动暂存、clean base 时重现，作为现有基线红项；独立复审与规范沉淀待执行。
- stage: review | plan_id: PLAN-717 | plan_revision: 5 | outcome: needs_fix | reviewed_commit: auto-lang `635963553ac66ecfaf44f81ef60997ffb0dd7a29`; auto-os `73d02b50f535543bd635c6f6f49dad6835efff45`; auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846` | base_commit: auto-lang `57b9afa60c34e2192ef55056d564c29709ee8a51`; auto-os `0d5f5bf37a76c27788e6fa262a8776553e8661ed`; auto-down `3373a5cc6e3a00336613133db51906fb0940777d` | dependency_revisions: auto-os same as base; auto-down same as base | spec_inputs: `docs/specs/auto-lang/ui/overview.md` SHA256 `891E296A8BEF4641601122223607393E037492E66402853FD16FE671864AAC02`; `docs/design/autoui/base-styles-and-visual-parity.md` SHA256 `E4651819F66B5130287BC751A5805A4483E2581D6CB30C18756CFE7C2F403997`; proposed target absent at review start | acceptance_results: AC-01 pass; AC-02 pass; AC-03 pass; AC-04 pass; AC-05 pass; AC-06 needs_fix; AC-07 partial pending canonical Spec deposit | findings: F-717-R1 (P2, AC-06/T-01): removing callback-use gating left nested `calls_callback` and `stmt_calls_callback` helpers unused in `ui_gen/vue.rs`, producing new dead-code warnings. Evidence: `cargo tu` output reports both helpers as never used at lines 1088 and 1113; clean-base comparison confirmed the unrelated desktop golden is baseline, but the new warnings are in the reviewed diff. | evidence: `docs/plans/reports/717-widgets-gallery-matrix.md`; full route/example manifest and screenshots under `docs/plans/reports/assets/717/`; gate excerpts recorded in this Plan | limitation: review performed in implementation session; verdict reconstructed from committed diffs, source manifests, screenshots, and clean-base test reproductions | next: work — remove obsolete callback-detection helpers, rerun the UI-generator warning/gate review, commit the correction, then repeat final review.
- stage: work | plan_id: PLAN-717 | plan_revision: 5 | outcome: pass | code_commit: auto-lang `7506d7d7251abedaa2246f8937d83f2b33a7e27e`; auto-os `73d02b50f535543bd635c6f6f49dad6835efff45`; auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846` | task_ids: F-717-R1, T-01, T-06 | evidence: removed unused `calls_callback`/`stmt_calls_callback`; `cargo tu` final run 858 pass/1 clean-base desktop golden fail/2 ignored with no diagnostics for removed helpers; `cargo check -p auto-lang` pass; `cargo build -p auto --bin auto` pass; `cargo tv` final retry 162/162 (after one transient `cb_web_mime` failure, isolated rerun and full retry passed); docs_gen 4/4; final `cargo t` 1447 pass and the same three clean-base failures | blockers: none | next: review — final review against commit `7506d7d7251abedaa2246f8937d83f2b33a7e27e`.
- stage: review | plan_id: PLAN-717 | plan_revision: 5 | outcome: pass | reviewed_commit: auto-lang `7506d7d7251abedaa2246f8937d83f2b33a7e27e`; auto-os `73d02b50f535543bd635c6f6f49dad6835efff45`; auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846` | base_commit: auto-lang `57b9afa60c34e2192ef55056d564c29709ee8a51`; auto-os `0d5f5bf37a76c27788e6fa262a8776553e8661ed`; auto-down `3373a5cc6e3a00336613133db51906fb0940777d` | dependency_revisions: auto-lang verified against auto-os `73d02b50f535543bd635c6f6f49dad6835efff45` and auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846` | spec_inputs: `docs/specs/auto-lang/ui/overview.md` SHA256 `891E296A8BEF4641601122223607393E037492E66402853FD16FE671864AAC02`; `docs/design/autoui/base-styles-and-visual-parity.md` SHA256 `E4651819F66B5130287BC751A5805A4483E2581D6CB30C18756CFE7C2F403997`; reviewed SD-01 draft `docs/plans/reports/assets/717/widgets-gallery-parity-spec-draft.md` SHA256 `C34DD993AA7E918BDCB44B99D8EF966ECC98B86852FF11ABBCFAA2793DE83555` | acceptance_results: AC-01 pass (70 routes/pages, 69 Home cards, 70 sidebar entries, zero omissions); AC-02 pass (Vue 70/70 light and dark, no page errors or non-favicon bad responses); AC-03 pass (VM 70/70 light and dark, all expected sidebar route transitions reached); AC-04 pass (159 preview-cards checked; 53/53 public schema examples visible in all four variants, full-page scroll captures); AC-05 pass (route/theme screenshots and representative states: Vue 15/15, VM 9/9; Tabs inner labels limitation is documented and route/screenshots pass); AC-06 pass with baseline-only failures documented (cargo check/build pass, cargo tv 162/162, docs_gen 4/4, cargo t 1447 passed before the three clean-base failures, cargo tu 858 passed with one clean-base golden failure and two ignored; no plan-attributable test failure or new warning remains); AC-07 pass for the reviewed SD-01 proposal and evidence contract; canonical Spec and overview projection remain merge checkpoints | findings: F-717-R1 resolved in `7506d7d`; no open plan-attributable findings. Existing P706-D3 VM sidebar intermittency remains registered debt; the PLAN-717 route driver follows actual sidebar buttons, allows up to three attempts, and all routes reached the expected state. No omitted acceptance item, unapproved deferral, or new workaround found. | evidence: `docs/plans/reports/717-widgets-gallery-matrix.md` SHA256 `3866C8D482028FA02D26510AE4047CDA9A87439FAB671F85274E08DEEDFA50BB`; route matrix `docs/plans/reports/assets/717/widgets-gallery-route-matrix.json` SHA256 `8BECB5C2AB8B3F147C664D9ADCFCB48A1CB6552B8F85F8E4F84EA99E1E9942D8`; visual assets under `docs/plans/reports/assets/717/`; three worktrees clean; `git diff --check` clean in all three; reviewed implementation diffs for dead code, warnings, debug output, and unapproved scope. | limitation: final review was conducted in the implementation session rather than by a separate agent; verdict was reconstructed from the committed diffs, source-derived route/schema manifests, visual evidence, recorded gate results and clean-base reproductions, not from the executor summary. | next: merge — prepare and verify canonical Spec/overview plus store-mediated ledger projection, land the three commits linearly, archive Plan, then guarded cleanup.
- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: prepared | reviewed_baseline: `PLAN-717:r5`, review pass above; auto-lang base `57b9afa60c34e2192ef55056d564c29709ee8a51`, auto-os base `0d5f5bf37a76c27788e6fa262a8776553e8661ed`, auto-down base `3373a5cc6e3a00336613133db51906fb0940777d` | default_tips_before_landing: auto-lang `8ba65e2a6e2017c07f3aceeebe6f0daad8532bce`; auto-os `0d5f5bf37a76c27788e6fa262a8776553e8661ed`; auto-down `3373a5cc6e3a00336613133db51906fb0940777d` | auto-lang_rebase: `635963553ac66ecfaf44f81ef60997ffb0dd7a29` → `5bad66e0fd93081906c2089cfdfd8d54768688ab`; `7506d7d7251abedaa2246f8937d83f2b33a7e27e` → `676d7e3c9f619cd3fa812262de594637f1471374`; `git range-diff` reported both patch pairs equal; the two default-branch commits since review base modify only `docs/plans/712-vm-desktop-defects.md` | prepared_spec_commit: `c48007bc5f4dec623aaf5050fcbcd99321b3d427`; documentation-only exact projection of reviewed SD-01; target `docs/specs/auto-lang/ui/design/widgets-gallery-parity.md` SHA256 `C34DD993AA7E918BDCB44B99D8EF966ECC98B86852FF11ABBCFAA2793DE83555`; overview SHA256 `D60137CC8779D7499FCA7A00F9484E0CED71A73429CEA3FBECFBEE2F5F78E4D1`; plans index SHA256 `AA1F563382C19E211B3E7F941A290FEA78B57AE52C1CF51737ADCC4F48015E4A`; spec-index regenerated with no unrelated index diff; spec-lint: 0 errors, 6 pre-existing broken-link warnings, none in PLAN-717 files | expected_delivery_commits: auto-lang `c48007bc5f4dec623aaf5050fcbcd99321b3d427`; auto-os `73d02b50f535543bd635c6f6f49dad6835efff45`; auto-down `895f8d0f9355c9f5ec3ce8fca268bdb768395846` | projection_targets: `designs` current-knowledge item for `docs/specs/auto-lang/ui/design/widgets-gallery-parity.md`; `reviews`/`reports` history references to PLAN-717, revision 5, reviewed/delivery commits and eventual archive path; preserve GOAL-007 status | guard: all three absolute plan worktrees reported clean with no reparse points before landing | next: land in dependency order auto-down → auto-lang → auto-os; then publish and read back ledger through store-mediated writer.
- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: landed | result: pass | default_branch_tips: auto-down master `895f8d0f9355c9f5ec3ce8fca268bdb768395846`; auto-lang master `c48007bc5f4dec623aaf5050fcbcd99321b3d427`; auto-os main `73d02b50f535543bd635c6f6f49dad6835efff45`; each tip equals its verified delivery commit after `--ff-only` | integration_check: merged-checkout `cargo check -p auto-lang` exit 0 (375 repository warnings remain; review found no new Plan-717 warning); `git diff --check` clean for all three landed ranges | preserved_concurrent_work: auto-lang `.autoos/specs.json`, `docs/plans/.next-id`, temporary VM evidence and website images remain unstaged; auto-os desktop scripts, `.auto/ui-cache.json`, and other user files remain unstaged; auto-down checkout clean | next: refresh the workspace ledger through its store writer, then archive and clean only after read-back succeeds.
- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: ledger_refreshed | result: blocked | target: workspace `D:\autostack\auto-lang`, `.autoos/specs.json`; no mutation attempted | evidence: no `read_specs`/`list_specs`/`write_spec`/`update_spec` capability is exposed in this session; documented local store API port `127.0.0.1:8080` has no listener. Direct JSON editing is forbidden by the merge workflow, and the tracked ledger file already contains unrelated user changes. | recovery: keep the Plan active at `status: reviewed`; do not archive or remove any worktree until a store-mediated writer is available and the item projection is read back. Canonical Specs and all three code branches are landed; `ledger_refreshed`, archival and cleanup remain outstanding.

- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: ledger_refreshed | result: pass | date: 2026-10-01T12:10+08:00 | reconciliation_first: 重跑 gate 核对——三仓 worktree 复验 clean;`plan-717-dev` c48007bc5 复验为 master 祖先(ahead=0);canonical spec 文件在主检出 SHA256 与冻结值全等(C34DD993…);8080 仍无监听且本会话无 spec 工具 → 循 `docs/specs/README.md` §5 路径映射表「musk 后端通常不可用→手工回退+§4 扩展」与 711/713/714/715 在案先例执行外科插入 | mutation: `.autoos/specs.json` 三段插入——designs `P717-1`(widgets-gallery-parity 现行知识投影,file=canonical doc,docsha:c34dd993aa7e918b,commit:c48007bc5)/reviews `P717-2`(r5 复审+三仓 ff-only 合入收据)/reports `P717-3`(交付摘要);counts designs 119→120/reviews 182→183/reports 107→108,goals 84/architecture 118/tests 95 零扰动 | guards: 双形态 roundtrip 字节守卫(committed 形 indent1+LF;worktree 形 indent2+CRLF)先证后写;五段零扰动回读断言;worktree 形 `P712-1` 在途 WIP 条目字节保全不裹挟提交(首次提交误以 `git commit --only` 裹挟 worktree 形,即觉 reset 重做,最终提交 77e54aeaf 净 42 行纯插入,blob f9edea3e1a26e211) | index: `scripts/spec-index.py` 再生内容零漂移(仅 CRLF 幻影,已还原);全局件:GOAL-007 按 projection_targets 保持原状,无结构性变化 overview.md 不动 | next: archive then guarded cleanup.
- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: archived | result: pass | archive_path: `docs/plans/archive/717-widgets-gallery-dual-backend-completeness.md`(本仓归档目录=archive/,非技能文档的 archived/,§5 路径映射);status flipped reviewed→archived;spec-sync 回写记录见文末 §11 | deployment_observation(landing≠deployment): 本计划触及 `ui_gen/vue.rs` 生成器(auto-lang c48007bc5)与 auto-os widgets-gallery 页面(73d02b5)——主检出 `auto` release 二进制与 auto-os desktop release 二进制相对落地为陈旧态(构建日期早于交付 commit),gen/front 生成束随下次构建再生,auto-down engine 为 JS 包无独立二进制随前端构建消费;按 714 先例记录观察项,不阻断归档,随下轮 release 周期重建 | batch_regression_due_check: `.last-batch-regression.json` last_covered_plan_id=715 @ 2026-09-30T16:20Z;717 非 5 的倍数,收据未满 48h → **不到期**,无需 `/auto-plan:regress`;下次到期=计划号 720 落地,或收据满 48h 且期间有任何合并。
- stage: merge | plan_id: PLAN-717 | plan_revision: 5 | checkpoint: cleaned | result: pending-guard

## 10. 待澄清事项

- 当前无阻塞问题。视觉判定采用组件/示例语义正确、内容完整可读、布局无意外裁切或遮挡，并参照既有 AutoUI 跨端样式规范；逐像素相等不是本计划的统一门槛。若执行中发现某类组件规范与目标冲突，先记录证据并修订本计划，再继续相关实现。

## 11. spec-sync 回写记录

- canonical：`docs/specs/auto-lang/ui/design/widgets-gallery-parity.md`（SD-01 全量投影，SHA256 `C34DD993…` 与复审冻结值全等，@c48007bc5）；`docs/specs/auto-lang/ui/overview.md` 现行态回写、`docs/specs/auto-lang/ui/plans.md` 追加 717 行、INDEX 再生——均随 prepared spec commit c48007bc5 落主。
- ledger：`.autoos/specs.json` designs `P717-1` / reviews `P717-2` / reports `P717-3` 外科插入（77e54aeaf）；INDEX 再生零漂移。
- 全局件：`goals.md` GOAL-007 状态保持（projection_targets 明示 preserve）；无新模块/新 crate/状态翻转，全局 `overview.md` 无需更新。
