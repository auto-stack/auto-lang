# T-03 轨C submenu 浮动式 spike 决策工件（PLAN-698 / 2026-09-23）

## T-03a 结论：**feasible** —— 浮动式恢复落地（内联降为降级路径）

### 根因勘定（P695-D2 的真实死因）

iced 0.14 的 overlay 体系**本有官方嵌套协议**：
- `iced_runtime::user_interface` 把根 overlay 包成 `overlay::Nested`，
  递归调 `Overlay::overlay(layout, renderer)` 挂子 overlay（nested.rs
  `recurse`：layout/update/draw 全递归）；
- `Overlay::overlay()` 默认返回 `None`——此前 **Panel 未实现该钩子**，
  面板 content 子树里的嵌套 Popover widget 的 overlay 永不注册：
  浮动子菜单不渲染。P695 三次"子菜单开态 screenshot 恒超时"走查即
  撞在此处（等待永不出现的像素）；"进程静默死亡"一次未复现（本轮
  全部实验含基线对照共 6+ 次 spawn 均存活至主动终止）。

### 修复（两件）

1. **popover.rs**：Panel 实现 `Overlay::overlay()`——收集 content 子树
   的 widget overlays。坐标协议：Panel::layout 已 move_to 面板绝对位置，
   子树 bounds 即窗内绝对坐标，平移恒取 `Vector::ZERO`，嵌套 Popover 的
   anchor_position（layout.position()+translation）天然正确。
2. **aura_view_builder.rs**：浮动式恢复——submenu trigger 包装为嵌套
   `View::Popover { placement: RightTop(T-06 顶对齐右弹), open: sub_open,
   on_dismiss: None }`，面板样式与顶层一致（w-44/popover chrome）。
   内联臂降为降级路径：`AUTO_MENU_SUBMENU_INLINE=1` 走旧形态（判位/
   回退用）。P695-D2 债务划线在即（T-05 对账）。

### 验证

- 契约测试 `p695_menubar_submenu_radio_label_shortcut` 更新到浮动契约：
  闭态=nested Popover（RightTop、open=false、content Column 含面板项+
  popover chrome）；开态（复合键 file::sub-3）=open 翻转、项数不变。
  `cargo t p695_menubar` 6/6 绿；`cargo t popover` 18/18 绿。
- 实机（widgets-gallery VM 臂，menubar 页 File→Share 两级）：
  - 开/闭态 aura snapshot 全程可达，submenu 子树在 view tree
    （浮动 Popover 形态）；
  - 进程全程存活（6+ 次实验含基线，无一静默死亡）。

### P695-D5 判位留痕（不在本计划修复面）

**screenshot 通道超时与 submenu 形态无关**：基线对照（stash 全部本轨
改动后重建）单层菜单开态的 `autoui_screenshot` 同样 10s 超时（iced
thread 不应答），浮动版/内联降级版三者一致；关态截图正常（7.3s 冷首）。
超时位在"任意 menubar 开态+此 MCP 截图管道"或环境窗口状态，属 D5
041/render 时序族的显形。按计划待澄清4口径：AC-03 判定面=渲染+快照，
截图超时只判位留痕不修复（另线清偿）。

### 残余边界（诚实账）

- 像素级确认（浮动面板截图）被 D5 通道阻塞，未取得；结构面（Nested
  协议落地+契约测试+快照面+存活）构成 feasible 判定依据。
- 交互细节未打磨：父面板把子面板开态点击按"面板内"转发（子项点击
  动作+菜单关闭并存）；hover 换菜单/键盘导航归 P695-D1 另批。
