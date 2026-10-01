# 程序化聚焦（ui.focus）

> PLAN-095 T-04 交付（auto-musk-095-dev@0cdf4b548）。本文是桌面轨
> 程序化聚焦的权威行为规则；web 轨 `ui.focus(sel)` 由 ts_adapter 直译
> `document.querySelector(sel)?.focus()`（与 dom.focus_first 同义）。

## 接口

- `ui.focus(target_key)`：target_key = 目标输入控件的 .at 输入 handler
  事件键（`.Input` / `.Input($event)` / `Widget.event` 限定形；解析时剥
  点前缀与 `$event` 实参尾）。**不猜测第一个控件**，不支持全 CSS 选择器。
- 覆盖面：`Input`（Id 主键 `auto_input_{widget}_{event}`，Plan 483）与
  `Textarea`（Id 主键 `textarea_{widget}_{event}`，PLAN-051 P2）；与构建
  臂派生严格同式。

## 链路与接线完整性（缺一即静默 no-op）

1. native `auto.ui.focus`（id 9920，9900+ 高段）——native.rs shim 写进程级
   请求槽 `UI_FOCUS_REQUEST`。
2. codegen 三处：stdlib 模块名单（`"ui"`）、func_name 重写映射
   `("ui","focus")→"auto.ui.focus"`、**NATIVE_ID_ENTRIES 白名单行**——
   缺白名单行时编译重写落空、运行时静默 no-op（实测记录）。
3. renderer update 尾部消费：take 槽 → 当前视图 `collect_focusable_inputs`
   解析 Id → `iced::widget::operation::focus` 尾任务 → 结果写
   `__focus_result`（`ok` / `miss:<key>`）可观察。

## miss 语义

- 视图中已存在其他可聚焦输入而目标缺席 = **立即 miss**（稳定缺失）。
- 视图尚无任何输入 = 挂载竞态窗口，5 轮重试（沿用 `__focus_input`
  口径；有更新周期才推进）。
- 不匹配的 CSS 选择器形 key 在 VM 轨一律 miss（无对应物，不猜）。

## 验收锚

`ui_focus_vm_tests`（VM 级链路）+ resolve 单测 + auto-musk
canvas-runtime-probe focus 探针（双目标 ok/miss 可定位）+ 聚焦后
text_input 渲染 Focused 边框（视觉实证，review 补证）。
