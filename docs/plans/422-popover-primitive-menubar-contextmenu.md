# Plan 422: 弹层原语(anchor 定位 popover)与 menubar/contextmenu 迁移

> **状态**: ✅ 已实施(2026-08-23,分支 422-popover;P1-P4 全部落地,矩阵 29/29 + 行为语义测试 13/13;两项人工验收待办见 §6)
> **来源**: 三计划共同指向的同一架构缺口:AutoUI 没有锚定弹层原语,menubar 下拉靠 absolute+像素估算,点击捕获靠 2000px 隐形按钮,右键菜单无处落地
> **关联**: 418(合成 menubar 迁移方)/ 413(editor oncontextmenu)/ 412(toast 层先例)/ 409(overlay hoist Stack)

---

## 0. 一句话结论

**做一个 iced `overlay` 机制的真弹层原语(anchor 按钮定位 + overlay 自带置顶/点击捕获),menubar 迁移上去**——估位偏移与 2000px catch hack 一并退役;contextmenu 复用同一原语。

## 1. 现状盘点(三方缺口同一根因)

- **menubar(418)**:面板 `absolute z-50 top-[33px] left-[8+Σ(字符×12+28)]px` 估算——按钮文字变宽即错位;click-outside 靠 `w-[2000px] h-[2000px]` 隐形按钮;probe 嵌套路径(§8.4① 刚对齐)绑死此结构。
- **contextmenu(413)**:`on_context_menu` 回调消息链路通(renderer 已透传),但无弹层容器可挂——回调后应用层只能自己拼 absolute hack。
- **iced 0.14 overlay 语义(第五会话已实读)**:`Widget::overlay` 返回 `overlay::Element`;overlay 先于基础树收事件,且 overlay 报告鼠标交互时基础树 cursor 置 Unavailable——**tooltip 已是先例**(EE03 即 overlay 弹层);iced_test 布局测试台可断言弹层 bounds(§8.9 基座)。
- toast 层(412)= Stack 常驻槽方案,点击穿透——与本原语(事件捕获)互补,不冲突。

## 2. 方案要点

- **View 层**:`View::Popover { anchor: 锚引用(widget id), placement: Bottom/Top/…, content, on_dismiss }`——builder 把 menubar 面板从"absolute 兄弟"改为挂在触发按钮的 Popover 上;DSL 侧提供 `popover (anchor: "btn-id", placement: bottom) {}` 标签。
- **渲染层**:自定义 wrapper widget 的 `overlay()` 实现(参照 iced Tooltip 的 overlay 结构:content overlay + 位置按 anchor bounds 计算,snap_within_viewport);**overlay 默认捕获落在其 bounds 内的点击**(取代 2000px catch);Esc/失焦 dismiss(on_dismiss → `__menubar_close` 同型内部消息)。
- **menubar 迁移(convert_menubar)**:面板改 Popover 挂触发按钮;`MENUBAR_OPEN` 本地态机制不变;left 估算整段删除;probe 记录路径随结构更新(面板不再是行级子节点——§8.4① 的 onclick 锁定测试需同步改)。
- **contextmenu**:editor `on_context_menu(pos)` 消息携带坐标 → 应用 handler 打开 Popover(anchor 为动态坐标——原语需支持"坐标锚"变体);示例先落 041(右键编辑器弹 复制/全选 菜单)。
- **后续复用**:select/dropdown、tooltip(EE03 可迁移为统一原语的轻量变体,非必须)。

## 3. Phases

- **P1 原语 MVP**:View::Popover + overlay wrapper(按钮锚)+ 捕获/dismiss;iced_test 定位断言(锚下方、右缘 snap)。
- **P2 menubar 迁移**:删估位/catch;矩阵 29/31 全绿(含 §8.4① 锁定项随结构调整更新);实机真实点击开合。
- **P3 坐标锚 + contextmenu 示例(041)**。
- **P4 文档与复用清单**(弹层原语进 gallery /position 类页面说明)。

## 4. 验收

- 按钮文字任意宽度,面板不错位(layout_tests 断言 + 实机);点击面板外即关(无 2000px hack);Esc 关。
- 041 右键编辑器弹菜单,项可点;MCP 矩阵全绿;snapshot 面板 onclick 属性仍在(§8.4①)。

## 5. 风险

- overlay 与基础树的光标交互语义(Unavailable 传导——第五会话踩过 tooltip 遮蔽分析,需按此设计自测:弹层打开时按钮 hover 态预期)。
- 与 409 absolute hoist 的 z 序叠加(面板内再弹子菜单的远期场景,本期不承诺)。

---

## 6. 实施记录(2026-08-23,分支 422-popover)

### 落地内容

- **P1 原语**(`ui/iced/popover.rs` + `View::Popover`):Wrapper 模式(Tooltip 同型)——layout 委托锚,`overlay()` 在 open 时产出面板;8 向 placement + viewport snap;坐标锚(`PopoverAnchor::Point`,contextmenu 变体)。Panel overlay 实现了 `operate` 转发 → iced_test selector 可见面板节点(tooltip 不可见正因缺此)。
- **捕获语义**(取代 2000px catch):面板内点击转发+兜底捕获;**锚上点击 dismiss+捕获**(点触发器=关,不透传 toggle);外部点击 dismiss 但放行(menubar 切换语义);Esc 捕获;窗口失焦通知。锚点击分支最初漏发 on_dismiss——由新增的 Simulator 消息级测试抓出修复(MCP 点击直派 handler 不经 iced 事件流,矩阵测不到该路径)。
- **P2 menubar 迁移**:convert_menubar 改 Popover(BottomStart 左缘对齐),`left` 像素估算与 catch 按钮整段删除;面板结构每帧恒定(open 驱动显隐);`find_view_by_path`(MCP 分派)补 Popover 臂——缺它时 menubar 触发按钮的 MCP 点击静默失败。
- **P3 contextmenu**:code_editor `on_context_menu` 坐标转视口系(加 widget origin),消息携带 (x,y)(encode_payload f64);DSL `popover` 标签(坐标锚形态);041 右键弹 剪切/复制/全选 菜单(EditorCtx/CtxClose/Ctx* 处理器)。
- **P4 复用**:DSL popover 支持 shadcn 嵌套形态(popover-trigger/popover-content,自管开合:内部 `__popover_toggle/close` 消息 + action_config::POPOVER_OPEN 注册表,menubar 同型)与 first-child 形态;gallery popover.at 零改动获得真弹层行为。tooltip(EE03)未迁移(非必须,原样保留)。

### 验证

- iced_test 定位断言 4 项(锚下方对齐/右缘 snap/坐标锚/关闭态)+ 行为语义断言 4 项(面板项点击发消息/外部 dismiss/锚 dismiss 不透传/Esc 捕获)——layout_tests 13/13。
- 041 MCP 矩阵 29/29(含 §8.4① `__menubar_toggle("file")` onclick 锁定项)。
- gallery vm 模式启动正常。

### 人工验收待办(实机)

1. 041 右键编辑器:弹菜单落在光标处、项可点、点击外部/Esc 关(MCP 无鼠标事件注入,无法自动化)。
2. gallery popover 页:点击 "Open Popover" 开合、Esc 关。

### 已知边界

- 面板内再弹子菜单(z 序嵌套)本期不承诺(与 409 hoist 的叠加待后续)。
- rust 模式 codegen 未覆盖 popover 标签(ui_gen/rust.rs)——VM/iced 语义完整,vue 端沿用既有 shadcn 映射。
- convert_view_messages 的 `_ => Empty` 兜底仍吞 View::Overlay(Popover 已显式处理)——历史遗留,与本案无关。
