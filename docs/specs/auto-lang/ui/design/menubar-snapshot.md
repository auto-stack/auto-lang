# menubar 快照可见性与开合持久性（PLAN-664 U-1 current-state）

> 计划：`docs/plans/664-jade-consumer-upstream-package.md`
> 相关：[overlay-interaction.md](overlay-interaction.md)（浮层事件投递）；
> [nested-timesource.md](nested-timesource.md)（TimeSource 泵语义——本篇的
> 关闭判据排除面引用其收集表）
> 消费锚：auto-edit 041 MCP 动作矩阵（`examples/ui/041-auto-edit/tests/
> desktop_mcp.py`，Plan 418 Phase 1）、jade-garden desktop menubar 同族。

## 问题（已定罪，2026-09-20）

041 矩阵六失败（39/6 口径）：MCP 打开 menubar 后展开项不进 `autoui_snapshot`
——菜单项按文本找不到。根因**不是**快照遍历缺条件分支（Popover 遍历 Plan 422
已覆盖，见下），而是**开合状态活不到下一次快照**：renderer 的"任意非 `__`
前缀消息关菜单"自动关闭判据把 DSL TimeSource 泵事件（`Tick`/`timer{}`
处理器名，天然无 `__` 前缀）误判为用户交互——041 状态栏时钟每秒一条
`Tick`，把刚打开的菜单在消费方拍快照前关掉。矩阵 39/5↔39/6 波动 = 拍快照
与 Tick 的竞态；jade exe 实机可见 = 其应用无周期性非内部消息。

## 硬规则

1. **Popover 子树全量进快照**。`view_to_vtree_with_paths`
   （`vnode_converter.rs`）对 `View::Popover` 的 widget 锚形态收录
   `[anchor, content]` 两子（Plan 422 契约，子序 = snapshot/probe 约定
   0/1）。menubar 降级（`convert_menubar_component`，PLAN-630 声明式
   组件族 / Plan 418 actions 合成同机制）把展开项建在 content 面板内
   ——菜单开着，项就在快照里。
2. **开合状态是渲染器本地态**（`action_config::MENUBAR_OPEN` 进程级
   Mutex），不是 DSL model 态；toggle/close 走 `__menubar_toggle(id)`/
   `__menubar_close` 内部消息（`␟s␟<id>` 载荷编码，`decode_payload` 解）。
3. **自动关闭判据 = 用户交互近似**：`menubar_open().is_some()` 时，非
   `__` 前缀消息关菜单——**但 TimeSource 泵事件除外**（`DynamicComponent::
   is_timesource_event`，对 `timesources` 声明全集按事件名匹配，Tick 与
   自定义 timer 处理器名均覆盖）。框架周期消息不是用户交互；真正的
   关闭面 = 点击菜单外（Popover `on_dismiss` → `__menubar_close`）、
   Esc/窗口失焦、以及任意用户 handler 派发（非 `__` 且非泵）。
4. **MCP press 通道带参可达**：合成点击把 `DynamicMessage::Typed` 的
   args 经 `encode_payload` 编进事件串（PLAN-403 需求 1b 修复族），
   `__menubar_toggle` 类带参内部消息经 MCP 派发不丢参。

## 验证锚

- e2e：041 矩阵（修后 49/1、47/1 两轮——menubar 六项稳定通过；残余单失败
  每轮不同 = 预存 render 时序抖动族，非本契约面）。
- 单测：`ui::dynamic::plan664_timesource_event_name_*`（泵名/用户处理器
  名/内部名三态判定）。
- 机制复现件：`examples/ui/041-auto-edit/tests/probe_menubar.py`（点击链
  五段定位法：触发按钮 → click 响应 → 快照 diff → 上下文 dump → 状态尾）。
