# menubar 族（15 元素：menubar / menubar-menu / menubar-trigger / menubar-content / menubar-item / menubar-checkbox-item / menubar-separator / menubar-label / menubar-radio-group / menubar-radio-item / menubar-shortcut / menubar-sub / menubar-sub-trigger / menubar-sub-content / menubar-group）

来源计划：PLAN-695（2026-09-23，用户授权：shadcn-vue menubar 对齐完整组件族 + gallery 完整示例 + auto-edit 回灌；元素总数勘正 16→15——shadcn-vue menubar 全组件 = root+14 子件，起草期算术笔误，复审 R-1 勘定）。

## 契约

- **schema**：15 元素全量注册（`schema/aura.at` 权威面；`iced: component` 如实注记，root `menubar` 为 builtin_widget 其余 web_component）。schema.rs 硬编码 fallback 照 menubar_checkbox_item 先例不镜像新增件（围栏 rs 维度防孤儿）。
- **vue 臂**：全 kind 发射 reka-ui 标准件（`@/components/ui/menubar`，16 资产在库）。radio-group `value` → `:model-value` 单向绑定（回写走 radio-item onclick，与 checkbox `:checked` 同源语义）；item 的 shortcut prop 与独立 `menubar-shortcut` 子件均发射 `MenubarShortcut`（muted token——硬编码 zinc span 已退役）；静态 `disabled` bool → `disabled` 属性。
- **registry**：15 spec 别名纪律（`menu-bar-*` 官方 kebab / `menubar_*` schema snake / `menubar-*` DSL kebab）；vue 映射自 schema overlay 按折叠键灌入。
- **VM 臂（双解释态同源）**：
  - 面板/项/分隔线/trigger 配色全量消费语义 token：`bg-popover`/`text-popover-foreground`/`border-border`/`bg-border`/`text-foreground`/`text-muted-foreground`（`Color::Popover/PopoverForeground` 独立投影 registry 双盘，浅色白板深字可读——字面 `#16171B`/zinc 族全退役）。
  - 开合走 `MENUBAR_OPEN` 全局注册表（`__MenubarToggle`/`__MenubarClose`，menubar-snapshot 四规则承袭）；**复合键** `<menu_id>::sub-<idx>` 表 submenu 层级，外层开态判**前缀感知**（`o==id || o.starts_with(id::)`——单值注册表下子键曾连带关闭外层，实锤修正）。
  - hover-switch：开态下其他 trigger 包 `MouseArea(on_enter=toggle)`（关闭态悬停不开、已开不重入）；开态 trigger `bg-accent text-accent-foreground rounded-sm`。
  - **submenu = 浮动式（PLAN-698 T-03 终态，翻案 T-11）**：子面板为嵌套 `View::Popover`（placement `RightTop` 顶对齐右弹，T-06 变体；面板样式与顶层一致 w-44/popover chrome），chevron 随开合翻转。T-11 降级的真因勘定=iced 0.14 官方嵌套协议（runtime `overlay::Nested` 递归 `Overlay::overlay` 钩子）未被子面板 Panel 实现——钩子落地（content 子树 overlay 收集，绝对坐标零平移）后嵌套 Popover 正常注册渲染；P695-D2 债务划线。降级路径保留：`AUTO_MENU_SUBMENU_INLINE=1` 回旧内联形态（父面板内缩进节，判位/回退用）。Vue 臂 reka 真 submenu 不受影响。
  - disabled/无 onclick 项**置灰非隐藏**（muted 前景 + opacity-50，对齐 shadcn `data-[disabled]:opacity-50`）；radio 选中 `lucide:circle-dot`、checkbox 勾选 `lucide:check`（leading_icon 参数化）；label/独立 shortcut 为 muted 静态文本。
  - a2r 静态降层（`a2r_menubar_panel_children` 递归发射）与解释态同 kind 表、同复合键、同 token 词汇。
- **键盘**：Esc 关闭走 Popover 既有捕获；方向键/Enter 导航落债 P695-D1（需 iced 焦点基建，时间盒裁定不阻塞）。

## 已知边界

- VM submenu 浮动面板的像素级截图确认被 P695-D5 screenshot 通道族阻塞（任意 menubar 开态该通道 10s 超时，基线零改动同现——与浮动/内联形态无关，判位留痕 P698 报告 T03-submenu-floating.md）；结构面由契约单测+快照+进程存活佐证。交互细节（hover 换子菜单/键盘导航归 P695-D1）未打磨：子面板开态点击=项动作+菜单关闭并存。
- gallery `preview-card` 实况预览走 untracked 动态解析路径——新族元素须 tracked/untracked 双表同臂（D-GAP 纪律；本批 menubar 臂曾缺 untracked 侧致预览整树 Empty）。
- auto-edit VM 臂截图通道与 1s 状态栏 Tick 争用（预存）——菜单态截图不可得，验证走快照结构 + 单测 token 断言。

## 验证

- 单测：`p695_menubar_family_registry_complete`（15 canonical + 8 kebab 命中 + 组件名/import 对账）、`p695_view_menubar_full_family`（vue 全 kind 发射 + 零 zinc 色）、`p695_menubar_panel_tokens_and_disabled_dim`（解释态 token 类断言 + 置灰双通道）、`p695_menubar_hover_switch_and_open_highlight`（包裹差分 + accent）、`p695_menubar_submenu_radio_label_shortcut`（浮动 Popover 形态〔RightTop/闭态 open=false/面板 chrome〕+ circle-dot + muted 文本，PLAN-698 更新）、`p695_menubar_a2r_popover_tokens_and_dimmed` / `p695_menubar_a2r_submenu_radio_label_shortcut`（a2r 同构）。
- 围栏：schema_drift 2/2（含 queue_coverage 新件登记）；docs_gen 4/4（core.md 再生）。
- 实机：widgets-gallery `#/menubar`（四菜单全族 + Bound State 回显，VM 深色截图在案）；auto-edit VM 走查 编辑→行尾→CRLF 三段全真 + probe_menu.py exit 0。
