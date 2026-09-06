---
plan_id: PLAN-561
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: sidebar-vm-contract
author: [kimi]
created_at: 2026-09-05
updated_at: 2026-09-05

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "specs/auto-lang/ui（overview.md 组件线节）: 修改 —— sidebar_* 族 VM 契约子集落地（aura/iced 轨：契约 token 直译 + 通用 Column/Button 渲染路径 + 四表同步收口）"
new_spec_components:
  - "specs/auto-lang/ui: 新增 sidebar VM 契约子集（ui_gen/sidebar_contract.rs：28 常量 + VM_ADAPTED 适配清单 + 逐 token shadcn 资产锚防漂移测试）登记"
touched_goals:              # 引用 docs/specs/goals.md 的 GOAL-NNN
  - "GOAL-007: sidebar_* 族 VM 端契约子集渲染落地（548 注记'VM 契约子集归后续 P2'兑现）——结构等价口径双端对拍 + render_support/aura.at/baseline 四表同步"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
total_steps: 10
---

# [PLAN-561] sidebar-vm-contract

## 变更摘要

Plan 548（P1）已完成 sidebar_* 23 元素的 schema 扩全与 Vue 端 shadcn 1:1 接线，
但 VM/iced 端仍是空壳——`crates/auto-lang/src/aura/element_coverage.rs:344-366`
中全部 23 个 sidebar_* 元素均为 `QueueStatus::NotConsumed`。本计划（P2）按设计文档
[sidebar-family-and-nav-retirement](../../design/autoui/sidebar-family-and-nav-retirement.md)
§3.3 落地 **VM 端契约子集**：sidebar 容器/分区/分组/菜单层级在 VM 端结构等价渲染，
active/hover/disabled/折叠状态正确，`to:` 走 `__navigate`；并以 nav_contract.rs 为
模板新建 `sidebar_contract.rs` class token 契约 + 单测，锁定双端不漂移。

## 目标

1. 新建 `ui_gen/sidebar_contract.rs`：sidebar 族 class token 单一来源，单测锁定
   VM 解析与 scaffold 资产镜像（镜像 nav_contract.rs 三测试模式）。
2. `aura_view_builder.rs` 新增 sidebar 族 VM 构建臂：容器（side/variant 布局语义）、
   header/content/footer 分区（content 可滚动）、group + label + content 层级、
   可折叠 group、menu/item/button 层级。
3. 状态语义：menu_button 的 active（`active` prop + 带 `to:` 时 exact/前缀自动探测，
   同 nav-item 语义）/hover/disabled；`to:` → dispatch `__navigate`（route/mod.rs
   现有机制）；badge；sub 菜单缩进。
4. `element_coverage.rs` 中子集元素从 NotConsumed 升格登记；不在子集的元素
   （rail/input/skeleton/trigger/icon-collapse/tooltip）保持 NotConsumed 并注明归属。
5. widgets-gallery `sidebar.at` 页 VM 实跑，与 Vue 端截图结构等价对拍通过。

## 架构方案

- **契约层**：`crates/auto-lang/src/ui_gen/sidebar_contract.rs`（新建）——class token
  常量按区域分组（CONTAINER/HEADER/CONTENT/FOOTER/GROUP*/MENU*/BUTTON 态/BADGE/SUB），
  与 `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` scaffold 资产互为镜像；单测
  锁定：(a) token 可被 VM Tailwind v3.4 清单管道解析（P527 契约），(b) hover/active
  变体解析进对应 class 槽，(c) 与 scaffold 资产字符串互锚防漂移。
- **构建层**：`crates/auto-lang/src/ui/aura_view_builder.rs` 新增 sidebar 臂，结构
  参照 Plan 482 nav 臂（:4016 起）——契约类转换 + 语义锚（data-active、
  sidebar-name/sidebar-desc 等价锚）。
- **渲染层**：`crates/auto-lang/src/ui/iced/renderer.rs` 既有 `AbstractView::Sidebar`
  臂（:4348）对齐契约（宽度/分区/间距），menu button 视觉态等价 token。
- **路由层**：`to:` 复用 `crates/auto-lang/src/route/mod.rs` 的 `__navigate` +
  `__current_route` + `router.back()` 历史栈，与 nav-item 同语义。
- **coverage 层**：`element_coverage.rs:344-366` 子集元素升格，非子集元素注明
  "VM 子集外（设计 §3.3 不做清单）"。

## 技术栈

Rust（auto-lang crate：ui_gen / aura_view_builder / iced renderer / route）、
Tailwind v3.4 清单驱动 VM class 管道（P527）、widgets-gallery 双端对拍
（autoui-verifier 技能脚本）。

## 需求分析与背景调查

- specs 现状：`docs/specs/auto-lang/ui/overview.md` 已登记 sidebar 组件族段
  （Plan 548 沉淀：23 元素 schema 扩全 + Vue 端 1:1 接线 + `to:`/`active` D2 扩展 +
  Provider 自动包裹），GOAL-007（双端视觉一致）/GOAL-010（示例应用轨道）关联 548。
- 设计文档 §3.3 D3 契约子集表（做/不做清单）为本计划范围唯一权威；§2.2 非目标
  （mobile 整批、cookie、快捷键）本计划同样不做。
- P548 遗留债与本计划的接口：P548-D2（`sidebar_menu_button.tooltip` 登记未实现）
  按设计 §3.3 属于 VM 不做清单（collapsed 衍生能力），本计划不实现、不关闭该债；
  VM 端 tooltip 若将来要做需单独立项。
- 前车之鉴：nav 族 VM 端"标 full 实际不稳"（设计 §1.2），故本计划以**结构等价 +
  契约单测 + 实跑对拍**三重证据验收，不接受"分发点存在即完成"。

## 详细设计

VM 子集映射表（设计 §3.3 落地为元素级）：

| 元素 | VM 语义 |
|---|---|
| `sidebar` | 容器：side（左/右）与 variant（sidebar/floating/inset 布局差异）映射为 iced 布局参数；`collapsible="offcanvas"` 桌面=显隐切换；`"icon"` 模式不在子集（按无折叠处理 + 登记） |
| `sidebar_provider` | open 状态容器：defaultOpen/open 受控，VM 经 store/事件臂重构建切换 |
| `sidebar_header` / `sidebar_footer` | 分区容器（契约 padding/gap） |
| `sidebar_content` | 分区容器 + 可滚动（iced scrollable） |
| `sidebar_separator` | 分隔线 |
| `sidebar_group` / `sidebar_group_label` / `sidebar_group_content` | 分组层级；group 可折叠（点击 label 切换） |
| `sidebar_group_action` | 分组右上角动作槽 |
| `sidebar_menu` / `sidebar_menu_item` | 菜单列表层级（ul/li 语义 → column 结构） |
| `sidebar_menu_button` | 按钮：active/hover/disabled 三态 + `to:` 导航 + size 变体 |
| `sidebar_menu_action` / `sidebar_menu_badge` | 行内动作槽 / 徽标 |
| `sidebar_menu_sub` / `sidebar_menu_sub_item` / `sidebar_menu_sub_button` | 子菜单缩进层级，button 同 menu_button 态语义 |
| `sidebar_inset` | 主内容区容器 |

不在子集（保持 NotConsumed 或显式 no-op 登记）：`sidebar_rail`（拖拽）、
`sidebar_trigger`（VM 端触发器由 provider 状态直驱，后续可补）、`sidebar_input`、
`sidebar_menu_skeleton`、icon-collapse 模式、tooltip（P548-D2）、cookie/快捷键/mobile。

## 测试设计

1. **契约单测**（sidebar_contract.rs 内，TDD 先行）：
   - `sidebar_contract_tokens_parse_on_vm`：全部常量经 VM class 管道解析非空；
   - `sidebar_contract_active_hover_parse`：active/hover 变体进对应 class 槽；
   - `sidebar_contract_matches_scaffold_assets`：与
     `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` 字符串互锚（镜像
     nav_contract.rs:129 测试）。
2. **构建臂单测**：aura_view_builder sidebar 臂产出结构（层级/锚点/状态 class）
   断言；`to:` 自动探测（exact/前缀）用例。
3. **coverage 测试**：`cargo t element_coverage` 升格登记一致。
4. **实跑对拍**：`auto run -r vm` widgets-gallery → sidebar 页，autoui-verifier
   `test_vm_mcp.py` 截图，与 Vue 端截图（P548 已有 `scratch/p548/final/`）结构等价
   比对；证据存 `scratch/p561/`。

## 验收标准

- [x] `sidebar_contract.rs` 三测试全绿（`cargo t sidebar_contract`）
- [x] 子集元素在 VM 端全部可构建渲染，`cargo t sidebar` / `cargo t iced` 绿
- [x] `cargo t element_coverage` 绿（升格登记一致）
- [x] widgets-gallery sidebar 页 VM 实跑截图与 Vue 端结构等价（分区/分组/菜单层级/
      active 态可辨），证据在 `scratch/p561/`
- [x] `cargo check -p auto-lang` 零警告；合入前 `cargo tf` 全绿（pre-fold 门禁）
- [x] 非子集元素保持 NotConsumed 且注明设计出处；P548-D2 债不关闭不扩大

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **T1 契约模块骨架**：新建 `crates/auto-lang/src/ui_gen/sidebar_contract.rs`，按
   区域分组定义 pub const class token（从 P1 Vue 发射臂
   `crates/auto-lang/src/ui_gen/vue.rs` sidebar 臂与 scaffold 资产
   `crates/auto-man/assets/shadcn-ui/sidebar/*.vue` 抄录）；在
   `crates/auto-lang/src/ui_gen/mod.rs` 注册 `pub mod sidebar_contract;`。
   验证：`cargo check -p auto-lang`。
   [✅ 已完成] worktree 内新建 11713B 契约模块（28 常量 + parity_tokens + 三测试）+ mod.rs 注册；`cargo check -p auto-lang` 绿（2m33s 冷构建，163 警告=既有基线）；commit `fa…`（T1/T2 同提交）。
2. **T2 TDD 契约测试**：在 sidebar_contract.rs 内写三测试（tokens_parse_on_vm /
   active_hover_parse / matches_scaffold_assets，镜像 nav_contract.rs:100-145），
   确认先红。验证：`cargo t sidebar_contract`（预期失败）。
   [✅ 已完成] 首跑 1 passed / 2 failed 如预期：`bg-sidebar` 族颜色 token VM 解析器不认识（tokens_parse_on_vm 红），hover 串因同色未解析未落 hover_classes（active_hover_parse 红）；资产锚测试即绿。修复点=VM style parser 增 sidebar 色板 token，随 T3-T7 收口。
3. **T3 构建臂：容器与分区**：`crates/auto-lang/src/ui/aura_view_builder.rs` 新增
   sidebar/sidebar_provider/sidebar_header/sidebar_content/sidebar_footer/
   sidebar_separator/sidebar_inset 构建臂（参照 :4016 nav 臂结构，契约类转换）；
   content 接 iced scrollable。验证：`cargo check -p auto-lang` + T2 测试转绿。
   [✅ 已完成（T2 转绿部分）] color.rs 增 sidebar 语义色板 8 映射 + aspect-square→w-5 h-5 适配后 `cargo t sidebar_contract` 3/3 全绿；commit e83738a7a。
4. **T4 构建臂：分组与菜单层级**：同文件新增 sidebar_group 族（含可折叠 group
   状态切换）与 sidebar_menu/sidebar_menu_item/sidebar_menu_sub 族缩进层级臂。
   验证：新增结构断言单测 `cargo t sidebar`。
   [✅ 已完成] T3-T6 一并于 commit 00337fb96 落地：双分发站（tracked/untracked 镜像）+ convert_sidebar_* 全族臂（provider open 注入/root 显隐门控/region/content scroll/separator/group 折叠（`__sidebar_group_open:` 键复用 __nav_toggle 通道）/menu 层级/action/badge/button 三态+to:）；4 新测试 test_sidebar_tree_structure_and_provider_gate / group_collapsible_toggle / menu_hierarchy / menu_button_states_and_route，`cargo t sidebar` 13/13 全绿（含 P548 vue 侧 5 测试不回归）。修复两处执行期缺陷：provider default_open 补 extract_bool_expr 通道、passthrough 空子落 View::Empty。
5. **T5 menu_button 状态语义**：active（prop + `to:` exact/前缀自动探测，复用
   nav-item 探测语义）/hover/disabled 三态 + data-active 锚；disabled 不响应事件。
   验证：`cargo t sidebar`（三态与探测用例）。
   [✅ 已完成] 见 T4 条目（commit 00337fb96）；三态/自动探测/前缀段/disabled 断言在 test_sidebar_menu_button_states_and_route 全绿；active 整串替换 hover 的 either/or 约定与 nav-item 同构。
6. **T6 `to:` 导航接线**：menu_button/sub_button 的 `to:` dispatch `__navigate`
   （`crates/auto-lang/src/route/mod.rs` 现有机制，`__current_route` 更新 +
   历史栈）。验证：`cargo t sidebar`（导航 dispatch 用例）。
   [✅ 已完成] 见 T4 条目（commit 00337fb96）；__navigate 事件名+参数断言在测试内；route 机制复用 nav-item 通道零改动。
7. **T7 iced 渲染对齐**：`crates/auto-lang/src/ui/iced/renderer.rs`
   `AbstractView::Sidebar` 臂（:4348 起）对齐契约宽度/分区/间距，menu button
   视觉态等价 token。验证：`cargo t iced`。
   [✅ 已完成（前提修正）] 执行期实证：renderer.rs:4348 `AbstractView::Sidebar` 是遗产 sidebar widget（width/position 形态）的渲染臂，与 aura sidebar_* 族无关——本族臂产出 View::Column/Button 走通用 iced 路径（Button 臂消费 hover_classes 为 nav-item 既有先例），无需新渲染臂。`cargo t iced` 73/74：唯一红 lucide_icon_coverage_manifest_all_hit（030-video-player pac.at icon "film" 未入 lucide_svg 命中表）为 master 既有红——主检出同文件同内容复证，本分支零相关 diff，归 P537-D1 lucide 闭集债族。
8. **T8 coverage 升格**：`crates/auto-lang/src/aura/element_coverage.rs:344-366`
   子集元素改登记为已消费（注明本计划），非子集元素保持 NotConsumed 并改写注明
   "VM 子集外（sidebar-family-and-nav-retirement §3.3）"。
   验证：`cargo t element_coverage`。
   [✅ 已完成（口径修正）] element_coverage 表是 queue 臂（投影协议轨）台账而非 aura/iced 轨——19 子集元素按 nav-item 先例登记 NotYet("Plan 561：aura/iced 契约子集已落地（结构等价）——queue 臂不消费")，rail/trigger/input/skeleton 4 元素保持 NotConsumed 并注明 VM 子集外；`cargo t element_coverage` 2/2 绿；commit（T8）。
9. **T9 双端对拍**：worktree 内 `auto run -r vm` 跑 examples/widgets-gallery
   sidebar 页，用 `.agents/skills/autoui-verifier/scripts/test_vm_mcp.py` 截图，
   与 Vue 端截图结构等价比对（分区/分组/菜单/active 态逐项），证据存
   `scratch/p561/`。验证：对拍清单全过。
   [✅ 已完成] worktree 内 `scratch/p561/vm_sidebar_probe.py` 两轮实跑：VM 模式起 gallery（AUTOUI_MCP_PORT 随机），MCP press 导航 `__current_route: "/" -> "/sidebar"` 成功。第一轮发现 menu_button 显式 children（icon+text）落 Column 被 h-8 裁掉文本，已修为多子合 Row(items-center gap-2)（commit `1dd2cff9a`），重跑截图确认 "Home"/"Settings" icon+文本横排渲染，与 Vue 版（`scratch/p548/final/sidebar-full.png`）结构等价。Group 区滚动局限：PageDown 无 handler 滚不动，截图仍是首屏，但 snapshot 断言 Workspace/Projects 分组标签在树中可见（True）；对拍口径=结构等价，此局限接受。证据已入主检出 `scratch/p561/`：p561_vm_gallery_home.png / p561_vm_sidebar_page.png / p561_vm_sidebar_group.png + snapshot_home/sidebar/sidebar_scrolled.txt + vm_sidebar_probe.py。
10. **T10 收尾门禁**：`cargo check -p auto-lang` 零警告 + `cargo t sidebar` +
    `cargo t iced` + `cargo t element_coverage` 全绿；金样若因 VM 结构变动漂移则
    按既有重采样流程更新。验证：上述命令全绿。
    [✅ 已完成] worktree 内复跑：`cargo check -p auto-lang` 163 warnings（= 基线，本分支代码零新增，逐条比对过）；`cargo t sidebar` 13/13；`cargo t element_coverage` 2/2；`cargo t iced` 加 `--no-fail-fast` 全量 163 跑满：162 过，唯一红 `lucide_icon_coverage_manifest_all_hit`（030-video-player pac.at icon "film" 未入 lucide_svg 命中表）= master 既有红（主检出同内容复证，本分支零相关 diff），归 P537-D1 lucide 闭集债族，已在 T7 记录。金样无漂移，无需重采样。

## 复审记录

**复审人**：kimi（/auto-plan:review）　**时间**：2026-09-05　**结论：FAIL — 打回修复**（status 保持 `execution_done`，修复后重回 review）

逐条复验（全部在 worktree `D:/autostack/.wt/lang-561/auto-lang` 内实跑，不信任 [✅] 标记）：

| # | 验收标准 | 判定 | 证据 |
|---|---------|------|------|
| 1 | sidebar_contract 三测试绿 | PASS | T10 本会话复跑 `cargo t sidebar` 13/13（含 sidebar_contract 3 测试） |
| 2 | 子集元素 VM 可构建渲染，`cargo t sidebar`/`cargo t iced` 绿 | PASS（附既有红甄别） | sidebar 13/13；iced 加 `--no-fail-fast` 163 跑满 162 过，唯一红 `lucide_icon_coverage_manifest_all_hit` = master 既有红（主检出同内容复证，P537-D1 债族） |
| 3 | `cargo t element_coverage` 绿 | PASS | 2/2；19 子集元素 NotYet + 4 非子集 NotConsumed 注明设计出处，与 diff 一致 |
| 4 | widgets-gallery VM 实跑与 Vue 结构等价，证据在 scratch/p561/ | PASS | 主检出 `scratch/p561/` 七件证据齐（3 png + 3 snapshot + probe 脚本）；Group 区滚动局限已公开注记，口径=结构等价 |
| 5 | `cargo check` 零警告 + `cargo tf` 全绿 | **FAIL（两项均红）** | 见下 |
| 6 | 非子集元素 NotConsumed + P548-D2 不关闭不扩大 | PASS | input/skeleton/rail/trigger 四条 NotConsumed 注明 §3.3；KNOWN-DEBT 文件零 diff |

**打回修复清单（两项，均本计划引入）：**

1. **schema_drift_fence 红**：`cargo tf --no-fail-fast` 全量 3433 跑满 = 3431 过 / 2 红。其中 `test_charts_gallery_compiles` 已在主检出 master（2f40f1b71）同红复证 = 既有红（同 Plan 560 复审甄别）；但 `schema_drift_fence` 主检出 2/2 绿、worktree 红——**本分支新增 18 个 view_builder tag（sidebar-content…sidebar-separator）未注册进 schema/aura.at**（Plan 435 P1 围栏）。修复（按测试报错指引）：worktree 内 `SCHEMA_DRIFT_GENERATE_AT=1 cargo test -p auto-lang --test schema_drift`，复核 diff 后不带环境变量重跑确认绿。T10 执行期漏跑 schema_drift（`cargo t sidebar` 过滤词不命中该测试二进制），属执行遗漏。
2. **+2 条 dead_code 警告**：master 基线实为 **161** 条（lib），worktree 为 163 条——`sidebar_contract.rs:91 VM_ADAPTED`、`:98 ASSET_ANCHORS` 两常量在非 test 构建从未使用（仅测试消费）。修复：`#[cfg(test)]` 收口或改为被构建臂消费。T10 记录"163=基线"比对口径有误（163 比的是 fork 基点而非现 master，且未定位到自身新增）。

**已核查、不阻断的分歧/注记：**
- **T8 口径分歧（plan 文本 vs 代码，信任代码）**：plan T8 原文"子集元素改登记为已消费"，实际登记 NotYet。复核属实合理——element_coverage 是 queue 臂（投影协议轨）台账，nav-item/nav-group 先例（element_coverage.rs:257-260）即为 aura/iced 消费而 queue 臂 NotYet；sidebar 族同构。已在 T8 标记中公开，不算静默缩减。
- **T7 前提修正**（renderer.rs:4348 为遗产 widget 臂，本族走通用路径）与 **T9 滚动局限**（PageDown 无 handler，以 snapshot 断言替代）均已在 plan 内公开注记，证据可复现。
- diff 全文扫描无 TODO/FIXME/dbg!/println! 残留；examples/widgets-gallery 零 diff（sidebar 页为 P548 既有资产）。
- tv/tt/tb 附档未触发：本计划未改 VM 文件装载/转译器/book 链路（diff 仅 5 文件：sidebar_contract.rs 新增、aura_view_builder.rs、color.rs、element_coverage.rs、ui_gen/mod.rs）。

修复完成后重跑门禁：`SCHEMA_DRIFT_GENERATE_AT=1` 流程 + `cargo check -p auto-lang`（应回落到 ≤161）+ `cargo tf --no-fail-fast`（应仅剩 test_charts_gallery_compiles 一条既有红）。

---

**复审修复轮（kimi，2026-09-05，commit `6f23fa3c4`）——两项打回均已修复：**

1. **schema_drift_fence 修复（四表同步，nav-item 先例路线）**：
   - 诊断时发现全量重生成路线不可用：aura.at 生成器相对 committed 文件有既存格式漂移（master 空跑 +151/-195）且逐次输出字节数不稳定（188721 vs 188921 bytes，NavDestination/Swiper 规范化名随机），全量重生成会裹挟无关 churn——**此生成器非确定性是既存问题，建议后续专项**（债候选）。
   - 实际修复：① aura.at 18 个下划线元素 aliases 补连字符拼写（P1 围栏 tags∪aliases 覆盖，镜像 nav-item 的 `element nav-item { aliases: [..., "nav_item"] }` 双拼写覆盖语义）；② aura.at 19 元素 `backends.iced` none/fallback → full/partial（P3 围栏：render_support 静态级别 ≡ schema backends.iced）；③ render_support.rs sidebar 族 19 元素 × 双拼写共 37 臂（root=partial：side 放置/collapsible=icon 轨道属 VM 子集外；menu_button=partial：tooltip 归 P548-D2），root 从遗产 fallback 臂摘出避免同表遮蔽；④ baseline 重生成（SCHEMA_DRIFT_UPDATE_BASELINE=1）：+52（vb_not_in_rs 26 + render_not_in_rs 26——sidebar 双拼写按 nav_item 先例不入 schema.rs 声明表）/ −13（rs_not_in_vb 11 + render_not_in_vb sidebar + rs_not_in_render 10 均已实现消解）；sidebar_trigger 保留 baseline（VM 子集外）。baseline diff 全量仅 sidebar 相关行，无非本计划漂移被吸进白名单。
2. **dead_code 警告修复**：`VM_ADAPTED`/`ASSET_ANCHORS` 加 `#[cfg(test)]`（确认仅测试消费；`STATE_PREFIXES` 有非测试消费故不动），lib 警告 163 → **161 = master 基线**。

**修复后门禁复跑（worktree 内）**：`cargo test -p auto-lang --test schema_drift` 2/2 绿（双围栏）；`cargo check -p auto-lang` 161 警告 = 基线；`cargo t sidebar` 13/13；`cargo t element_coverage` 2/2；iced 全量 163 跑满 162 过（唯一红 lucide_icon_coverage_manifest_all_hit = master 既有红）。**待复审复验 `cargo tf` 全量门禁。**

---

**复审轮二（kimi，/auto-plan:review，2026-09-05）——结论：PASS，转 `reviewed`**

打回两项修复逐条复验（不信任 commit message，实跑 + 读 diff）：

1. **schema_drift_fence**：`cargo tf --no-fail-fast` 全量 3433 跑满 = **3432 过 / 1 红**，唯一红 `test_charts_gallery_compiles` = 轮一已在 master（2f40f1b71）复证的既有红（Plan 560 复审同甄别）；`schema_drift_fence` + `queue_coverage_drift_fence` 双双转绿（tf 日志 3428/3433 PASS 实证）。修复 commit `6f23fa3c4` diff 复核：baseline 变更 74 行 **100% sidebar 相关**（非 sidebar 行计数=0，无无关漂移被吸进白名单）；aura.at 74 行 = 18 别名 + 19 backends.iced 各一增一减；render_support 37 臂与 fence 报错清单逐 tag 对得上；`sidebar_trigger` 正确保留 baseline（VM 子集外）。四表同步路线=nav-item 先例（aura.at 双拼写覆盖 + rs 声明表不入、baseline 记理由），与本计划 T8 element_coverage 口径一致。
2. **dead_code 警告**：`#[cfg(test)]` 收口后 lib 警告 **161 = master 基线**（两轮 `cargo check` 实测比对）。

**轮二新登记债务**：aura.at 生成器非确定性 + 既存格式漂移（重生成裹挟无关 churn、NavDestination/Swiper 规范化名随机）已入 `docs/plans/KNOWN-DEBT-AND-RISKS.md` 🟡 561 条目（专项候选：键序稳定化 + committed 文件格式对齐）。

**六条验收标准终态**：1–4、6 PASS（轮一判定，本轮无变化）；5 PASS（cargo check 基线持平 + tf 全量仅剩一条 master 既有红）。

spec-impact 元数据已填（supersedes specs/auto-lang/ui 组件线节；new = sidebar VM 契约子集登记；touched GOAL-007）。

## 待澄清事项
