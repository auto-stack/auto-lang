# 终端组件 chrome 与命中区归属规则

## 范围

终端组件的 chrome 层(菜单/滚动条/选区)与命中区归属。来源
PLAN-022(SD-01,T-06 收口定稿 2026-09-19);上层交互投递语义
(浮层间按压归谁)见 docs/specs/auto-lang/ui/design/
overlay-interaction.md(PLAN-023,本篇为其下层消费方)。

## 命中区归属规则

1. **内容矩形规则**:组件的鼠标命中区 = 自身内容矩形(与浮层
   opaque 捕获边界同律);布局件不拦截子件命中。
2. **分隔条(接缝)**:k=7..11 槽,8px 条专属命中(over→Press
   开拖,up→Drop 收);拖拽期 z-30 全窗捕获层吞按压为期望语义,
   Drop 后归零释放。
3. **终端命中**:pixel_to_cell 画布坐标映射(扣除快照窗位移与
   PAD),窗外(历史留白区)clamp 到边缘格。
4. **捷径命中即捕获**:shortcut_hit 命中发消息同时 capture_event
   ——分屏多终端挂同一捷径表时事件广播全树,不捕获则每实例各发
   一次(方向导航一次连跳多格);捕获后 Stack 停止向低层分发。
5. **mouse_interaction 恒 None**:终端在 Stack 浮层里,非 None
   交互形态令 Stack 对下层全部面板抬升光标(Cursor::Levitating
   → position()=None)致下层点击/滚轮/滚动条失联;悬停图标不变
   I-beam 为已接受代价。

## 已知限制

- 组件外点击失焦依赖 press-outside 分支;无窗口级焦点环回。
