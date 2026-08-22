# Plan 422: 弹层原语(anchor 定位 popover)与 menubar/contextmenu 迁移

> **状态**: 📋 已立项待实施(2026-08-22,源自 414 §3"无 overlay 弹层原语——menubar 下拉的核心缺口" + 418 §8.3 手工估位 + 413 §5.5 contextmenu 只出回调)
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
