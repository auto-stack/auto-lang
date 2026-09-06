---
plan_id: PLAN-562
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: nav-to-sidebar-migration
author: [kimi]
created_at: 2026-09-05
updated_at: 2026-09-06

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/ui（overview.md 组件线节）: 修改 —— 导航组件线（plan-482 nav/nav-group/nav-item/nav-link）全部使用方迁移至 sidebar_* 族（仓内 015/018/019 + widgets-gallery 外壳与 navitem/navlink 两页 + 外仓 auto-musk/auto-os-config），schema 四元素 superseded_by 标注入库，组件线段改写为退役记录（实现移除留观察期，KNOWN-DEBT 登记移除小计划要点）"
new_spec_components:
  - "specs/auto-lang/ui: 新增 schema superseded_by 退役元数据机制——schema_loader/ElementMeta 字段解析 + docs_gen 生成物（core.md/kitchen-sink.at）过滤带标注元素 + DOC_EXCLUDE 退役口径；首个应用 = nav 族"
  - "specs/auto-lang/ui: 新增 Plain 模式（pac.at shadcn: off）sidebar_menu_button/sub_button 原生 <button> 语义保持臂（vue.rs map_tag 兜底，契约类不内联，Plain 哲学=项目自带样式）"
touched_goals:
  - "GOAL-007: nav→sidebar 迁移双端回归（015/019/gallery 双端实证、018 VM 腿+Vue 腿 master 既有破备案、os-config VM 冒烟），sidebar_* 族成为唯一导航组件线；gallery 侧栏滚动 ScrollArea 化双端同源"
  - "GOAL-010: 示例应用轨道 015-notes/018-book-reader/019-video-app 与 widgets-gallery 外壳/文档页迁移 sidebar 族，navitem/navlink 页退役"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 9
total_steps: 9
---

# [PLAN-562] nav-to-sidebar-migration

## 变更摘要

按设计文档
[sidebar-family-and-nav-retirement](../../design/autoui/sidebar-family-and-nav-retirement.md)
§3.4 D4 执行 nav 族迁移与退役：把全部 `nav-group`/`nav-item` 使用方迁移到
sidebar_* 族（Plan 548 Vue 端已 1:1、Plan 561 VM 契约子集），随后 schema 中
nav 族标记 superseded_by。实现移除（nav_contract.rs、scaffold 资产、构建/渲染臂）
按设计"观察一个周期后移除"**不属本计划**，拆后续小计划。

## 目标

1. 仓内三示例迁移：`examples/ui/015-notes/src/front/sidebar.at`、
   `examples/ui/018-book-reader/src/front/app.at`、
   `examples/ui/019-video-app/src/front/app.at` 的 nav-group/nav-item 用法
   改写为 sidebar_* 组合（group+label+content、menu+item+button、to:/active）。
2. widgets-gallery 文档页重写：`examples/widgets-gallery/src/front/pages/navitem.at`
   与 `navlink.at` 改写为 sidebar 族示例页（或裁定删除并由 sidebar.at 承接，
   执行时按 gallery 信息架构二选一并记录）。
3. 外仓迁移：`auto-musk`（052 应用）与 `auto-os-config`（012 应用），按 Plan 529
   分组平铺布局在 `D:/autostack/.wt/lang-562/{auto-musk,auto-os-config}` 开各自
   worktree（branch `lang-562-dev`），迁移完成后各 fold 回其 master。
4. `nav` 容器 `search: true` 内联搜索行：若有使用方依赖，平移为
   `sidebar_header` + `sidebar_input` 组合写法（VM 端 sidebar_input 不在 561 子集，
   该情形需先登记再裁定）。
5. schema 退役标注：`schema/aura.at:538-589` 的 `nav`/`nav-group`/`nav-item`
   标记 superseded_by sidebar_*（`nav-link` 已被 nav-item supersede，随族一并标注），
   用 `SCHEMA_DRIFT_UPDATE_BASELINE=1` 更新基线。

## 架构方案

- **迁移映射**（nav 族 → sidebar 族，设计 §3.4 + 548 D2 扩展）：
  - `nav` 容器 → `sidebar` + `sidebar_content`（Provider 由 codegen 自动包裹，548 已落地）
  - `nav-group` + label → `sidebar_group` + `sidebar_group_label` + `sidebar_group_content`
  - `nav-item`（icon/label+desc/badge、`to:`、active）→ `sidebar_menu` +
    `sidebar_menu_item` + `sidebar_menu_button`（`to:`/`active` 语义已对齐，
    badge → `sidebar_menu_badge`）
  - 嵌套分组 → `sidebar_menu_sub` / `sidebar_menu_sub_item` / `sidebar_menu_sub_button`
  - `nav(search:true)` 内联搜索行 → `sidebar_header` + `sidebar_input`（VM 端见目标 4 的裁定）
- **跨仓工作流**：组目录 `D:/autostack/.wt/lang-562/`，外仓 worktree 以本仓命名
  （branch `lang-562-dev`），消费验证后即 fold 回各仓 master，不留悬挂。
- **退役门控**：全部迁移方双端回归通过 → supersede 标注；实现移除留观察期，
  另立小计划执行（删 nav_contract.rs / scaffold Nav*.vue / aura_view_builder nav 臂 /
  iced 分发点 + P548-D3 旧臂死代码一并清理）。

## 技术栈

Auto 语言示例源码（.at）、auto-lang codegen 双端（Vue/VM）、跨仓 git worktree
（Plan 529 布局）、autoui-verifier 双端对拍、schema drift 基线流程。

## 需求分析与背景调查

- specs 现状：`docs/specs/auto-lang/ui/overview.md` 导航组件线（plan-482）与
  sidebar 组件族（plan-548）两段并存；本计划完成后导航组件线段整体改写为退役记录。
- 使用方清单（grep 实证 2026-09-05）：仓内 015-notes、018-book-reader、
  019-video-app 三例含 nav-group/nav-item；widgets-gallery 有 navitem.at/navlink.at
  两个 nav 族文档页（navigationmenu.at 是另一 shadcn 组件，不动）；外仓
  auto-musk/auto-os-config 按设计 §3.4 记载（执行时 grep 复核）。
- 前置依赖：Plan 561（VM 契约子集）完成后本计划的 VM 端回归才有意义；
  若 561 未完成，仓内迁移可先做 Vue 端验证，VM 回归留待 561 入库后补。
- GOAL-010（示例应用轨道）、GOAL-007（双端一致）关联。

## 详细设计

逐文件迁移要点：

| 文件 | 现状 | 迁移目标 |
|---|---|---|
| `examples/ui/015-notes/src/front/sidebar.at` | nav-group/nav-item 侧栏 | sidebar 族组合，笔记列表 active 态走 `to:`/`active` |
| `examples/ui/018-book-reader/src/front/app.at` | app.at 内 nav 用法 | 同上 |
| `examples/ui/019-video-app/src/front/app.at` | app.at 内 nav 用法 | 同上 |
| `examples/widgets-gallery/src/front/pages/navitem.at` | nav-item 文档页 | sidebar 族示例页或删除（sidebar.at 承接） |
| `examples/widgets-gallery/src/front/pages/navlink.at` | nav-link 文档页 | 同上 |
| auto-musk 052 / auto-os-config 012 | 外仓 nav 用法 | 同映射表，各自 worktree |

schema 标注：`schema/aura.at` nav 族三元素加 `superseded_by: "sidebar_*"` 元数据
（格式参照仓内既有 supersede 标注先例）；`docs/components/` 生成文档随
docs_gen 再生成同步。

## 测试设计

1. 每迁移一例：`auto run`（Vue）+ `auto run -r vm`（VM）实跑，导航点击/active
   态/折叠行为回归；截图证据存 `scratch/p562/<app>/`。
2. widgets-gallery：nav 页重写后 gallery_golden 重采样 + 双端对拍。
3. schema 标注后：`SCHEMA_DRIFT_UPDATE_BASELINE=1` 更新基线，
   `cargo t schema_drift` 绿 + `cargo test -p auto-lang --test docs_gen` 绿
   （Category C 门禁）。
4. 外仓：各仓自身验证命令 + auto-lang 侧消费验证（组内 `../auto-lang` 解析）。

## 验收标准

- [x] 仓内三示例 + 两个 gallery 页全部迁移，无双端回归（截图证据在 scratch/p562/）
- [x] auto-musk / auto-os-config 迁移完成并各 fold 回其 master；仓内 grep
      `nav-group|nav-group|nav_item` 在 examples 下零残留
- [x] `nav` search:true 依赖方已平移或实证无依赖方
- [x] schema nav 族 supersede 标注入库，schema_drift 基线更新、docs_gen 绿
- [x] 实现移除后续小计划已立项（含 P548-D3 清理）或在本计划 review 时显式裁定延期
- [x] 合入前 `cargo tf` 全绿（pre-fold 门禁）

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 使用方复核**：仓内 `grep -rn "nav[-_]\(group\|item\|link\)" examples/ examples/widgets-gallery/src` +
   外仓 `grep -rn` auto-musk/auto-os-config，输出最终迁移清单附 plan。
   验证：grep 输出非空且与本文清单对账。
   [✅ 已完成] 2026-09-05 grep 实证最终清单（比对正文清单，**多出两个仓内文件**，验收标准"examples 零残留"口径补齐进 T5 扩展）：
   仓内：① 015-notes sidebar.at 9 处（nav search:true 容器 + nav-group×5 + nav-item label/desc/active/onclick 块式）② 018 app.at 3 处（nav-item to:/exact ×2）③ 019 app.at 1 处 ④ gallery pages/navitem.at 15 处 ⑤ pages/navlink.at 7 处 ⑥ **gallery app.at 68 处**（外壳侧栏 nav-item to:/icon:/label:，正文漏列→T5 扩展）⑦ **pages/kitchen-sink.at 14 处**（nav 族 prop 矩阵演示节，正文漏列→T5 扩展）。
   外仓：⑧ auto-musk app.at 5 + chats_view.at 1 + nav_item.at 1（自定义复用组件）⑨ auto-os-config auto/src/front/sidebar.at 9 + modules_store.at 1。
   排除：navigationmenu/nav_menu 族（另一 shadcn 组件）；auto-os-config tmp/css-era（历史快照，非活体）。
   search:true 依赖方实证：仓内唯一 = 015-notes sidebar.at:59（nav search:true + onsearch）——T7 裁定平移对象；外仓 grep search: true 另见 T7。
2. **T2 迁移 015-notes**：改写 `examples/ui/015-notes/src/front/sidebar.at` 为
   sidebar 族组合。验证：`auto run` 实跑 + 导航 active 态截图（scratch/p562/015/）。
   [✅ 已完成] worktree commit ab1005444。双端实证（scratch/p562/dual_probe.py）：
   VM 腿 press "Quick Ideas" → SelectNote active_id 0→1、编辑器内容切换；
   Vue 腿截图 p562_015_vue_home/pressed.png —— 分组（Notes/Work/Personal）、
   双行 note 项、选中高亮、搜索框全部正常，点击切换笔记。迁移惯用法：
   nav 容器→sidebar_provider(class 覆盖 w-auto min-h-0) 包独子；nav(search:)→
   sidebar_header + 原生 input；nav-item desc: 双行→menu_button 内单 col 孩子
   + style h-auto py-1.5。
3. **T3 迁移 018-book-reader**：改写 `examples/ui/018-book-reader/src/front/app.at`
   nav 用法。验证：同 T2（scratch/p562/018/）。
   [✅ 已完成] 同 commit ab1005444。VM 腿实证：press "Settings" →
   `.App.__navigate /settings`，`__current_route: "/" -> "/settings"`，
   settings 页（Theme/About 节）渲染正常。**Vue 腿阻塞：master 既有破
   `unknown element <theme-toggle>`**（stash 本 plan 改动后复现，逃逸舱
   examples/ui/018-book-reader/vue/src/components/ThemeToggle.vue），与本迁移无关，
   018 仅验 VM 腿。
4. **T4 迁移 019-video-app**：改写 `examples/ui/019-video-app/src/front/app.at`
   nav 用法。验证：同 T2（scratch/p562/019/）。
   [✅ 已完成] 同 commit ab1005444。VM 腿：press "Home" → `.App.__navigate /`
   无错误；Vue 腿截图 p562_019_vue_home.png 侧栏 Home 菜单项 + 视频网格正常。
   附带修复：crates/auto-man/src/vue.rs SCAFFOLD_DEPS sidebar 六件闭包
   {button, sheet, input, separator, tooltip, skeleton}（第一版漏 button/separator
   致 019 构建红，全量审计 sidebar/*.vue 内部 import 后补齐）。
5. **T5 gallery 页重写**：改写/删除 `examples/widgets-gallery/src/front/pages/navitem.at`
   与 `navlink.at`（sidebar.at 承接），重采样金样。验证：`cargo t gallery_golden` 绿
   + gallery 实跑截图。
   [✅ 已完成] worktree commit 716391c58。范围按 T1 扩展口径补齐：app.at 外壳
   69 个 nav-item（机械转换脚本 scratch/p562/migrate_gallery_shell.py，67 项迁移 +
   navlink/navitem 2 项随页删除；mobile drawer :278 实证为空壳无复制项）+
   index.at NavLink 卡片删除（Navigation 计数 10→9）+ kitchen-sink.at nav 族三节
   改 sidebar 族最小 smoke 演示。金样重采样 diff 复核：app.at 膨胀（nav→sidebar 族
   展开）、index/kitchen-sink 哈希变、两页行删除，无他文件变动；GENERATION ERROR/
   import 围栏绿。验证：widgets_gallery_all_front_pages_compile PASS；
   gallery_vue_golden 复跑绿（注意：金样测试不经 nextest `cargo t` 默认档，
   须 `cargo test -p auto-lang --test gallery_golden`）；双端实跑：VM press LineChart
   → /line-chart 页内容渲染（快照实证），Vue 截图 p562_gallery_vue_home/pressed.png
   侧栏分组/图标/选中高亮/滚动全部正常。
   **既有红备案**：`ui_gen::vue::tests::test_charts_gallery_compiles` 在 master
   裸跑同红（charts-gallery，与本 plan 无关，复审时勿误判为本 plan 回归）。
   **探针修复**（dual_probe.py）：vite URL 嗅探只认 `Local:` 行（此前误抢后端
   8080）；VM press 定位回溯祖先 button（text 子节点不可 press）；expect-route
   改认 VM 服务日志 state_changes；vue_leg 加 AUTOUI_MCP_PORT（9247 被 auto-musk
   占用时内嵌 VM 重试卡死 vite）；gallery 需 `--vue-render vue`（pac.at 默认非 vue）。
6. **T6 外仓迁移**：`git -C D:/autostack/auto-musk worktree add
   D:/autostack/.wt/lang-562/auto-musk -b lang-562-dev`（auto-os-config 同），
   按映射表迁移 052/012 应用，各仓验证后 fold 回其 master 并 wt-guard 清理
   worktree。验证：外仓 grep 零残留 + 各仓自身构建/测试绿。
   [✅ 已完成] 两外仓均已迁移并 fold 回各自 master（worktree 已 wt-guard 清理）：
   - **auto-musk**（fold 5a29251）：app.at 主导航 4 项 nav-item →
     sidebar_provider + sidebar_menu + menu_button（onclick/active:/size 不变，
     icon/label 改 children）。验证：lang-562 auto.exe `auto build --gen-only`
     绿（54 组件），生成物 App.vue 含 SidebarProvider/Menu/MenuItem/MenuButton。
     nav_item.at（NavListItem 自定义组件）实证零 schema nav 族使用，不改；
     chats_view.at 引用的是 NavListItem，非 nav 族。
   - **auto-os-config**（fold ebf0076）：sidebar.at 全量迁移（nav(search:)→
     sidebar_header + 原生 input；nav-item×4 → menu_button；collapsible
     nav-group → sidebar_group + 显式组头按钮 + if g.open 包 content——
     Vue 548 臂无折叠组实现，store 保持单一真源，双端同语义）。
     **关键发现：该仓 pac.at `shadcn: off`（Plain 模式）**——sidebar 族此前
     坍缩为无语义 div 且契约类全死（其 tailwind 令牌表无 sidebar-accent/accent
     条目；旧 nav-item 的 hover:bg-accent 在此仓本来就是死类）。处置两件套：
     ① auto-lang map_tag Plain 兜底臂（commit b97bbbb8d）：
     sidebar_menu_button/sub_button → 原生 `<button>`（不内联契约类，Plain
     哲学项目自带样式）；② sidebar.at 每个按钮以 style if 表达式内联承载
     旧 nav 契约原样类串（active=bg-primary/10 text-primary font-medium；
     hover 由死类修活为 hover:bg-secondary，有意行为改进）。
     验证：regen 绿（7 组件）+ host `npm run build`（vue-tsc+vite）绿 +
     VM 实跑冒烟（press "System Overview" → .Sidebar.SelectOverview，组
     chevron ▾/▸ 渲染，20 按钮树正常，证据 scratch/p562/osconfig/）。
     **已知形态差**：Plain 臂通用属性透传不补 `type="button"`（旧 nav 臂有），
     非 form 上下文无实际影响——记 T9 移除小计划清理点。
     教训：pnpm install 会在 worktree node_modules 产生 301 个 junction，
     wt-guard 拦截后须先逐链接 rmdir 再 remove（本 plan 首次实操验证流程）。
7. **T7 search:true 裁定**：grep 仓内外 `search:\s*true` 的 nav 用法；有则平移为
   sidebar_header + sidebar_input（VM 端缺口登记 KNOWN-DEBT），无则在 plan 记录
   "无依赖方"。验证：grep 证据附 plan。
   [✅ 已完成] 2026-09-05 裁定落地：依赖方共两处——仓内 015-notes（T2 已平移）
   与外仓 auto-os-config sidebar.at:30（T6 已平移），均走 **sidebar_header +
   原生 input** 惯用法（sidebar_input 不在 561 VM 契约子集，原生 input 双端
   通吃；KNOWN-DEBT 缺口登记随 T9）。T6 完成后三仓复 grep `search:\s*true`
   零残留（examples/ + auto-musk/src + auto-os-config/auto/src）。
8. **T8 schema supersede 标注**：`schema/aura.at:538-589` nav/nav-group/nav-item
   加 superseded_by 标注，`SCHEMA_DRIFT_UPDATE_BASELINE=1 cargo t schema_drift`
   更新基线。验证：`cargo t schema_drift` +
   `cargo test -p auto-lang --test docs_gen` 全绿。
   [✅ 已完成] 2026-09-05（commit 403ac5a0f）：nav/nav-group/nav-item/nav-link
   四块加 superseded_by 标注；schema_loader/ElementMeta 新增 superseded_by
   字段并解析；docs_gen 生成器（core.md + kitchen-sink.at）过滤带标注元素，
   DOC_EXCLUDE 收 navitem/navlink（nav/navgroup 原有条目注释同步改为退役）。
   重生成 core.md（移除 nav 四节）与 kitchen-sink.at（38→35 节，T5 手加的
   sidebar smoke 块随再生成冲销，sidebar 由 /sidebar 专页承接），金样二次
   重采样仅 kitchen-sink.at 行 + TOTAL 变化。docs_gen 4 绿 + schema_drift
   2 绿 + gallery_golden 1 绿（均复跑确认）。
9. **T9 收尾门禁**：仓内 grep nav 族零残留复核 + `cargo check -p auto-lang` 零警告；
   在 KNOWN-DEBT 登记"nav 实现移除观察期"条目并起草后续移除小计划要点
   （含 P548-D3 清理）。验证：上述检查全绿。
   [✅ 已完成] 2026-09-05：① 三仓 grep 复核——仓内 examples 零 nav 族元素
   残留（余命中均为 `nav()` 路由函数/手写 Vue CSS 类/注释）；外仓发现并处
   置一处 T6 漏网：auto-musk `viewstate_router.ts` popstate 桥仍锚
   `.nav-item` 契约类且按钮经 li 包裹后 nth-of-type 失效——改
   `querySelectorAll('.app-rail [data-sidebar="menu-button"]')` 取序
   （tsc 单文件 strict 绿；musk 同组 worktree 修复 ff-fold 8d5f648 后清
   理）。② widgets-gallery README 摘除 navlink 页引用（commit
   28ed6e846）。③ `cargo check -p auto-lang` 警告集与 master 逐条比对
   一致（164=164，落点全部在 vm/trans/extract 既有文件，本 plan 触碰文件
   零新增）。④ KNOWN-DEBT 登记两条：nav 族实现移除观察期（含移除小计
   划要点：schema/生成臂/渲染臂删除 + P548-D3 清理 + Plain 臂
   type="button" 补全 + sidebar_input VM 缺口裁定）与 015 潜伏 bug
   （`.SelectNote(i)` 作用域外引用，原样保留待独立排查）。
   [✅ 增补] 2026-09-06（commit 83bfbff61，用户截图反馈）：gallery 侧栏导
   航滚动条为原生浏览器样式——sidebar_content（shadcn SidebarContent 内置
   overflow-auto）替换为 AutoUI `scroll` 组件（Vue=ScrollArea 主题化细滚动
   条，VM=Scrollable），style `flex-1 min-h-0 px-3 py-4`，子级零缩进机械
   换壳。双端实跑证据：scratch/p562/p562_sidebar_{hover,scrolled}.png
   （hover 显细滚动条、wheel 滚动正常）+ VM 快照（scrollable 节点在树、
   press DatePicker 导航到对应文档页正常）。金样 app.at 行二次重采样复跑
   绿。主内容区 outlet 的 sidebar_content(h-full) 不在本次范围（用户只指
   侧栏）。

## 复审记录

复审人：kimi（/auto-plan:review，2026-09-06）。复审在 worktree
`D:/autostack/.wt/lang-562/auto-lang`（plan-562-dev）内进行；复审起点先把
master（547 imagesurface 等 84 commits）merge 回分支（91501b213 无冲突，
生成物重导 5c978371c：kitchen-sink +imagesurface 节 35→36、金样同步，
docs_gen 4 绿 + gallery_golden 1 绿复跑确认）。

验收标准逐条复验（verify, don't trust）：

| 验收项 | 裁定 | 证据 |
|---|---|---|
| 仓内三示例 + 两 gallery 页迁移，无双端回归 | ✅ pass | 复 grep `nav[-_](group|item|link)` examples/*.at 零命中（注释除外）；scratch/p562/ 双端截图/快照在库；018 Vue 腿未验属 master 既有破（theme-toggle 逃逸舱，stash 复现备案，非本 plan 回归） |
| 外仓迁移 fold + examples 零残留 | ✅ pass | auto-musk master 5a29251（迁移）+ 8d5f648（T9 捕获的 popstate 选择器漏网修复）；auto-os-config master ebf0076；两外仓 .at grep 零命中（`use nav_item` 为 musk 自有组件同名，实证零 schema nav 族使用） |
| nav search:true 依赖方平移 | ✅ pass | 两处（015、os-config）均 sidebar_header+原生 input 平移；三仓 `search:\s*true` 复 grep 零残留 |
| schema supersede 标注 + 基线 + docs_gen | ✅ pass | aura.at 四处标注在案（merge 后 :774/:794/:819/:835）；schema_drift 2 绿；docs_gen 4 绿（merge 后复跑）；core.md 无 nav 节 |
| 移除小计划立项或显式裁定延期 | ✅ pass（显式裁定延期） | 设计文档原定"观察一个周期后移除"；KNOWN-DEBT 已登记观察期条目 + 移除小计划要点（schema/生成臂/渲染臂删除 + P548-D3 清理 + Plain 臂 type="button" 补全 + sidebar_input VM 缺口裁定）；用户已在会话中知悉（P2/P3 立项对话），本复审显式裁定延期成立 |
| 合入前 cargo tf 全绿 | ✅ pass（基线口径） | merge master 后 `cargo tf --no-fail-fast`：3444 跑 / 3443 过 / **唯一红 test_charts_gallery_compiles = master 台账基线**（KNOWN-DEBT P555-D4 行；Plan 567 终局门禁 master tf 同红，非本 plan 回归——562 diff 不触 charts-gallery 与裸名折叠路径） |

遗漏/延后/workaround 扫描：

- 延后：nav 族实现移除 —— 显式裁定延期（见上表），非静默。
- 遗漏捕获史：T1 复核出正文漏列两项（gallery app.at 68 处 + kitchen-sink 14
  处）已并入 T5；T9 收尾 grep 捕获 T6 漏网（musk popstate 选择器）已修复
  fold。两起均已闭环，无未闭环遗漏。
- workaround：os-config Plain 模式按钮 style-if 内联类串（有意，Plain 哲学）、
  sidebar_input VM 缺口走原生 input 惯用法（已登记 KNOWN-DEBT）——均已记录，
  无未申报 workaround。
- kitchen-sink.at 手改违例（T5）已在 T8 根治（生成器学 superseded_by 过滤，
  页面回归"勿手改"幂等）。
- 执行后增补（83bfbff61，用户截图反馈）：gallery 侧栏 sidebar_content →
  scroll（ScrollArea/Scrollable 双端），主内容区 sidebar_content 保持不动
  （用户明确范围）；金样二次重采样，双端实跑证据在 scratch/p562/。

债务候选：无新增未登记项（562 两条 KNOWN-DEBT 已在册：nav 实现移除观察期、
015 `.SelectNote(i)` 潜伏 bug）。

**裁定：通过，status → reviewed。** 移交 /auto-plan:merge（fold plan-562-dev +
worktree 清理 + spec 沉淀）。

## 待澄清事项
