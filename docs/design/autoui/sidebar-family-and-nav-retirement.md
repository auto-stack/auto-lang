# Sidebar 组件族做实与 Nav 组件退役（shadcn 1:1 复刻路线）

> 需求级设计（Plan 468 归位规则，slug 不带号）。
> 起因：2026-09 讨论确认 `nav-group`/`nav-item`（Plan 482）融合设计粒度过粗、VM 端不稳，
> 决定转向把 schema 中已登记的 `sidebar_*` 族按 shadcn-vue Sidebar 一比一（视觉/结构层面）做实，
> 待其双端完善后迁移全部 nav-* 使用方并退役 nav-group/nav-item。

---

## 1. 背景与问题

### 1.1 现状：两族组件并存

- **nav 族**（`nav` / `nav-group` / `nav-item`，schema/aura.at:538–589，Plan 482）：
  把 shadcn Sidebar 的五六个零件融合成两个 widget，双端一等公民
  （`NavItem.vue`/`NavGroup.vue` ↔ `nav_contract.rs` class token 契约，单测防漂移）。
- **sidebar 族**（`sidebar`、`sidebar_content`、`sidebar_header`、`sidebar_footer`、
  `sidebar_group*`、`sidebar_menu*`、`sidebar_provider`、`sidebar_trigger` 等，
  schema/aura.at:4335 起）：shadcn-vue `Sidebar*.vue` 的逐组件登记，
  但 props 多为 "TBD"，`backends` 多为 `web: "none"` / `iced: "fallback|none"`——空壳。

### 1.2 nav 族的实际痛点

1. **粒度太粗**：融合设计砍掉了 MenuSub、GroupAction、MenuBadge 独立槽位等组合点，
   表达不了的结构只能再包一层 div/col——大量生成代码实测在 NavGroup/NavItem 外面又套一层。
2. **VM 端不稳**：iced 端虽标 `full`，实际使用中经常出问题。
3. **不符合生成侧直觉**：LLM 训练语料里全是 shadcn 写法，细粒度组合零件
   （SidebarMenu/SidebarMenuButton/asChild）是 agent 的天然表达模式。

### 1.3 关键事实：资产已在仓内

`crates/auto-man/assets/shadcn-ui/sidebar/` 下 shadcn 原版 25 个 `.vue` 文件已随快照入库
（Provider/Trigger/Rail/Inset/Menu*/Group* 全套 + `utils.ts`）。
**Vue 端"1:1 复刻"本质是 codegen/schema 接线 + 对拍验证，不是从零造组件。**

## 2. 目标与非目标

### 2.1 目标

1. sidebar_* 族在 **Vue 端桌面宽屏模式**下达到与 shadcn 原版视觉/结构一致（截图对拍可验收）。
2. VM 端渲染**契约子集**（见 §3.3），结构等价、状态（active/hover/disabled/折叠）正确。
3. 迁移全部 nav-group/nav-item 使用方后**退役** nav 族（schema 标记 superseded → 移除）。

### 2.2 非目标（明确砍掉）

- **mobile 机制整批不做**（含 Vue 端）：mobile Sheet 渲染、`isMobile` 检测、
  `SIDEBAR_WIDTH_MOBILE`、openMobile 状态。当前 Vue 端整体也未支持 mobile，不为本族破例。
- **cookie 持久化**（`SIDEBAR_COOKIE_NAME`/`sidebar_state`）：延后，有需要再单独立项。
- **键盘快捷键**（ctrl+b，`SIDEBAR_KEYBOARD_SHORTCUT`）：延后。
- VM 端**不追求** 1:1：offcanvas 滑入滑出、icon-collapse 模式、Rail 拖拽均不在 VM 子集内。

## 3. 设计决策

### 3.1 D1：Vue 端范围 = 桌面模式全零件

接线清单（资产均现成）：

| 区域 | 组件 |
|---|---|
| 骨架 | `sidebar_provider`（仅桌面 open 状态）、`sidebar`、`sidebar_inset`、`sidebar_trigger`、`sidebar_rail` |
| 分区 | `sidebar_header`、`sidebar_content`、`sidebar_footer`、`sidebar_separator`、`sidebar_input` |
| 分组 | `sidebar_group`、`sidebar_group_label`、`sidebar_group_action`、`sidebar_group_content` |
| 菜单 | `sidebar_menu`、`sidebar_menu_item`、`sidebar_menu_button`、`sidebar_menu_action`、`sidebar_menu_badge`、`sidebar_menu_skeleton`、`sidebar_menu_sub`、`sidebar_menu_sub_item`、`sidebar_menu_sub_button` |

Provider 的 `defaultOpen`/`open`/`onOpenChange` 受控接口保留（桌面折叠需要）；
内部 mobile 分支直接按桌面路径处理（`isMobile` 恒 false）。

`sidebar` 的 props（side/variant/collapsible）按原版支持；`collapsible="offcanvas"` 在
桌面模式即原版行为，`"icon"` 模式为 CSS 态切换，不含 mobile 语义，可保留。

### 3.2 D2：`to:` prop 扩展（视觉 1:1，API 不 1:1）

`sidebar_menu_button` / `sidebar_menu_sub_button` 增加 `to:` prop：

- Vue：渲染为 RouterLink（hash URL 更新），替代原版 asChild + `<a>` 的写法；
- VM：dispatch `__navigate` 更新 `__current_route`（沿用 nav-item 的现有机制）；
- `active` prop 映射原版 `isActive`，带 `to:` 时支持自动探测（exact/前缀段匹配，同 nav-item 语义）。

取舍：偏离 shadcn API 但贴合我们 codegen 的表达习惯；**1:1 限定在视觉与结构层**。

### 3.3 D3：VM 端契约子集

| 做 | 不做 |
|---|---|
| sidebar 容器（side/variant 的布局语义） | offcanvas 动画、移动端任何行为 |
| header/content/footer 分区（content 可滚动） | icon-collapse 模式、Rail 拖拽 |
| group + label + content 层级、可折叠 group | cookie/快捷键 |
| menu/item/button 层级，active/hover/disabled 态 | tooltip（collapsed 态衍生能力） |
| `to:` → `__navigate`；badge；sub 菜单缩进 | — |

VM 端以 nav_contract.rs 的教训为鉴：契约常量需有单测锁定双端 class token 不漂移；
iced 实现按"结构等价"验收，不追求像素级复刻 web。

### 3.4 D4：迁移与退役次序

1. 迁移方清单（Plan 482 D5 记载）：**015-notes**（仓内 examples）、**auto-musk**、
   **auto-os-config**（外仓，需对应 worktree 组），以及 widgets-gallery 的
   `navitem.at`/`navlink.at`/`sidebar.at` 三个文档页重写。
2. 全部迁移完成 + 双端对拍通过后：schema 中 `nav-group`/`nav-item` 标记
   superseded_by sidebar_*，再观察一个周期后移除实现（`nav/` 资产、nav_contract.rs、
   iced 分发点）。
3. 顺带决策：`nav-link`（已被 nav-item supersede）与 `nav` 容器的 search 集成
   （482 的 search:true 内联搜索行）去留——迁移时若有使用方依赖 search，需先把它
   平移到 sidebar_header + sidebar_input 组合写法，再退役。

## 4. Plan 分解建议

| Plan | 内容 | 验收 |
|---|---|---|
| P1（Vue 接线） | schema props 补全 + codegen 映射到 `@/components/ui/sidebar` 资产 + `to:` prop + gallery sidebar.at 重写为完整示例页 | autoui-verifier：与 shadcn 原版官方示例截图对拍；web 端全零件可用 |
| P2（VM 子集） | aura_view_builder / iced 渲染：§3.3 子集 + class token 契约单测 | `cargo t iced`；同一 gallery 页双端截图对拍（结构等价） |
| P3（迁移退役） | 015-notes / auto-musk / auto-os-config 迁移；gallery nav 页重写；nav 族 supersede 标注 → 移除 | 三 app 双端回归；schema drift baseline 更新 |

P1 先行（纯 web，无 VM 阻塞）；P2 是最重的一块，必要时再拆。

## 5. 风险与开放问题

- **Provider 上下文贯穿**：shadcn sidebar 零件大量依赖 `useSidebar()` 注入（data-state、
  collapsed 宽度等）。只做桌面模式也要保留 Provider 外壳，砍掉 mobile 后其复杂度大降，
  但 codegen 需强制/自动包裹 Provider——倾向 codegen 在检测到 `sidebar` 根时自动补
  `sidebar_provider`，降低使用门槛。
- **gallery 预览卡片内嵌**：sidebar 原版假设全页布局，gallery 文档页是卡片内嵌预览，
  可能需要 preview 作用域样式修正（现有 sidebar.at 已在卡片内用）。
- **外仓迁移**：auto-musk / auto-os-config 不在本仓，P3 需跨仓 worktree 组（Plan 529 布局）。
