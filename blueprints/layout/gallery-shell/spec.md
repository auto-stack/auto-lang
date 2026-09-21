+++
kind = "layout"
name = "gallery-shell"
palette = ["header", "aside", "sidebar_provider", "sidebar_header", "sidebar_menu", "sidebar_menu_item", "sidebar_menu_button", "scroll", "button", "text", "input", "icon", "col", "row", "separator", "badge", "popover", "popover-trigger", "popover-content"]
extension_points = ["brand", "header_actions", "filters", "content"]
variants = ["default"]
props = ["items", "active_id", "dark_mode", "accent_color", "empty_text"]
actions = ["select", "search", "set_theme", "set_accent"]

+++

# Intent

画廊/文档站**三段壳**骨架：顶 header（brand slot + 搜索框 + 右侧动作区 +
设置弹层）/ 左侧栏（aside w-72：筛选 pills slot + scroll 条目列表，active
态随 `active_id` 渲染）/ 弹性 content slot。综合 auto-os `widgets-gallery`
（routes 文档站 header/侧栏骨架）、auto-os `ui-gallery`（selected_id 状态
切换 + 分类 pills + 双行卡 + popover 设置弹层）与本仓 `charts-gallery`
（内容卡栈）三画廊的公共框架收敛而成（PLAN-676 用户裁定 2026-09-21），
首消费者为 `examples/bps-gallery` Auto 化改造。

判定依据（契约 Q5 准入五条）：#1 业界共识锚——docs 站三段壳（shadcn
docs sidebar、Docusaurus/GitBook 侧栏 + 内容骨架）；#2/#4 骨架 bp 例外
（PLAN-075 和解注记：布局骨架+slot 为契约本体；props 仅值面数据 +
on_* 回调，无业务语义）；#3 palette 全集 ⊆ WidgetRegistry（sidebar 族
P573 / popover Plan 422 / scroll P656 均双臂在册）；#5 本 spec 六问可答。

# Contract (六问摘要)

1. **输入 props**：`items []Item`（`{ id: str, title: str, category: str,
   badge: str }`，badge 空串不渲染）、`active_id str`、`dark_mode bool`、
   `accent_color str`（indigo/coral/ocean/sage/amber 五色板，Plan 409 §8
   约定）、`empty_text str`（空态文案，默认 "No entries."）。
2. **输出 actions**（on_* msg-ref 回调，宿主在 `on{}` 注册）：
   `select(item.id)` / `search(query)` / `set_theme("dark"|"light")` /
   `set_accent(name)`。
3. **状态归属**：`search_query`/`settings_open` 为 bp scoped（壳内
   model 自持）；**主题态归宿主 model**（契约 #3 平台服务禁 bp 私有
   副本——bp 仅镜像 dark_mode/accent_color 渲染 active 态，翻转经
   action 上归宿主，Plan 458 声明即 codegen 翻转在宿主侧生效）。
4. **变体**：`default` 单变体；结构性差异（routes 化导航、双栏内容等）
   走 promotions。
5. **打包**：kind=layout（词表既有）；磁盘 kebab 名 `gallery-shell`。
6. **双形态**：词汇面全部为双臂已验证 widget；动作上抛走 on_* msg-ref
   契约（PLAN-037 T3 TreeView / PLAN-528 W2 SettingsPanel 同款）。

# Acceptance

1. 骨架无应用侧文案：brand/条目/内容全经 slot 与 props 注入；
2. 条目点击上抛 `select(item.id)`，active 态随 `active_id` 渲染；
3. 搜索输入按键上抛 `search(query)`（vue/VM 同语义，无防抖差异面）；
4. 设置弹层切换 dark_mode / accent_color 经 action 通知宿主，bp 不持
   主题态；
5. `compact` 未声明；侧栏宽度 w-72 钉骨架内（sandwich gotchas#4 同款
   裁定：宽度=样式小差异，props 化须走 promotions）。

# What this blueprint absorbs (per-app variation)

brand 区内容（icon/标题富内容——brand slot）；header 右侧额外动作
（header_actions slot，位于搜索框与设置齿轮之间）；筛选 pills 的分类轴
与文案（filters slot）；content 区全部形态（默认出口——ui-gallery 视口
卡+tabs、bps-gallery spec 正文+源码+gotchas 各自填充）；条目数据与
选中语义（items/active_id 值面）。

# Assembly guidance

- 消费方经 `use bps.layout.gallery_shell.reference.default: GalleryShell`
  导入（L1 通道），调用位填 slot：`slot(name: "brand") {...}` /
  `slot(name: "header_actions") {...}` / `slot(name: "filters") {...}`，
  裸子节点落默认出口（content）。
- item schema 全键书写（缺键字段访问 = VM 硬错，filetree gotcha#3 同型）。
- 宿主回调绑定形参匹配：`on_select: .HostSelect`（HostSelect(str)）——
  载荷为 item.id / 查询串 / 模式串 / 色名。
- 满窗形态由消费方根决定（骨架根 `h-full w-full`，sandwich 同款）：
  页面壳外包 `h-screen overflow-hidden`（viewport-boundary 契约）；
  嵌套固定高单元 `h-full` 同语义。
- content 出口已内建 scroll 容器（`flex-1 min-h-0` 双界）；消费方在
  slot 内自行 max-width 包裹，不需要再套滚动。
- 设置弹层走 anchored popover（Plan 422，双臂）；VM 快照断言弹层内容
  需先触发齿轮（见 gotchas#3）。

# References

- `default` — header+侧栏+content 三段壳（reference/default.at，widget
  `GalleryShell`；settings 弹层内联 popover 形态）。

# Gotchas

See gotchas.md。
