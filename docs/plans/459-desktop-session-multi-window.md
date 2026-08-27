# Plan 459: DesktopSession 多窗口化（daemon 迁移 + 双 App 验收）

> **状态**: 🔄 已立项（2026-08-28），未开工
> **来源**: Plan 453 T7b 移交。453 的会话运行时代码工作已全部落地
> （T1–T6 + T4c 翻转 + T7a 守门零回归），M1 验收的最后一环——双 AppSession
> 双 OS 窗口 demo——因 iced 入口形态问题独立立项，验收完成后 453 一并归档。
> **架构依据**: `docs/design/23-autoui-virtual-desktop.md`（R3 退化桌面、
> §6 里程碑 M1；不变式 I2/I3）；453 计划 §2.1/§2.3（DesktopMessage /
> 订阅路由形状）；T4c 施工图 `reports/453-t4c-session-flip-blueprint.md`。
> **基线**: master a8a369544（453 全量 + T7a 守门记录）

## 1. 目标

把 `run_dynamic_iced` 的 iced 入口从 `application` 迁移到 `daemon`，让同一
DesktopSession 服务多个 OS 窗口、每窗口渲染各自的 AppSession，完成 453
遗留的 M1 验收：

1. 双 AppSession 双 OS 窗口 demo（`examples/` 或 `examples/ui/`）；
2. 隔离性验证：输入值 / DevTools 选择 / 日志互不串扰；
3. panic 隔离验证：注入 panic 只落当前窗口崩溃页，另一窗口持续可交互
   （453 §4 验收第 4 条）；
4. I2 复跑：T7a 已绿的 5 套 desktop_mcp 保持全绿。

**非目标**：VirtualWindow 容器与 WM（454）；跨 App 业务路由 DesktopBus
（454）；MCP `(AppId, widget)` 寻址实现（仅保证 single-app 语义不回归，
453 T8 已冻结）。

## 2. 关键事实与设计要点

### 2.1 为什么必须迁移 daemon（已实证）

iced 0.14 两个入口的 ViewFn 签名：

- `iced::application`：`ViewFn::view(&self, state: &State)` —— **不带
  window::Id**，全进程一棵视图，无法按窗口渲染不同 AppSession；
- `iced::daemon`：`ViewFn::view(&self, state: &State, window: window::Id)`
  —— 每窗口一视图，多窗口一等公民。

T7b 的第一个决定就是迁移 `iced::daemon`（或接受"第二窗口占位"的降级
demo，不满足 453 验收，不取）。

### 2.2 迁移方案要点

- **view 路由**：`daemon` 的 view 按 `session.app_of_window(win)` 反查
  AppId → `split_ref(app_id)` 构造拆借视图（T4c 基建直接复用）；
  未登记窗口（Opened 事件未达）回退占位元素。
- **update 不动**：T4-core 的 `DesktopMessage` 外壳（DM::App/DM::Desktop
  分派 + catch_unwind + Task::map）与入口形态解耦，预期原样保留。
- **subscription**：daemon 的订阅闭包同样收 `&State`（核对 0.14 签名），
  逐 App 订阅生成（453 T5 的原设计）顺路落地：widget_tick / 键盘绑定按
  AppId 标签化。
- **窗口生命周期**：windows 注册表、register/unregister、focused_window
  均已就绪（453 T4b/T4c），daemon 下多窗口只是消费者变多。

### 2.3 解除 `desktop_app_id()` 硬编码

- **AppId 分配**：Opened 事件按递增计数器分配新 AppId 并登记
  （`DesktopSession` 增分配器；不再预设 AppId(1) 必为主 App——主窗口
  语义 = 注册表首个登记窗口，454 由 WM 接管）。
- **消息打标改造**：现 `map_to_app` 硬编码 AppId(1)。业务 listen_with
  回调实际已收 window_id 三参——Resized/mouse/modifiers 等事件消息改为
  带窗口上下文（消息形状需携带 window::Id 或改道 DM::Desktop）；
  widget 回调（view 产 IcedMessage，无窗口上下文）在 daemon 下 view 按
  窗口构建，可经 per-window 打标包装解决（T1 施工图定案）。
- **键盘/定时器等 App 级订阅**：按 AppId 生成多份（453 T5 原案）。

### 2.4 demo 的第二个 App 从哪来

两案 T1 定夺：① 同一 `.at` 组件双实例（改动最小，证明状态隔离即可）；
② 两个不同 `.at`（更接近终态，验证组件注册表/路由互不污染）。
建议 demo 先行 ①，② 作为 T4 可选加深。

## 3. 任务表

| # | 任务 | 内容 |
|---|---|---|
| T1 | daemon 迁移施工图 | 只读评估：daemon 的 boot/update/view/subscription/title/theme 全签名、退出语义（全窗口关闭才退）、window::open 通路、MCP 截图 N1（per-window）、`run_dynamic_iced` 调用链（lib.rs 组装段）影响面；产出读点清单与提交序列 |
| T2 | shell 迁移 | `run_dynamic_iced` 切 `iced::daemon`；view 按 `app_of_window` 路由；单窗口行为不回归（I2：5 套 desktop_mcp 复跑） |
| T3 | AppId 分配与打标 | §2.3：递增分配、map_to_app 改造、订阅按 App 标签化（453 T5 一并落地） |
| T4 | 双窗口 demo | `examples/` 双 AppSession 双 OS 窗口示例；隔离性人工/脚本验证 |
| T5 | panic 隔离验证 | 注入 panic（DevTools 手法或示例内置开关）只落单窗口崩溃页；453 §4-4 |
| T6 | 收尾 | 453/459 文档核销、跟踪文件仪表盘更新、453 归档 |

## 4. 验收

1. demo：一个进程、两个 OS 窗口、各渲染一个 AppSession，输入/DevTools/
   日志互不串扰。
2. panic 隔离：单窗口 panic 不落进程、不波及另一窗口。
3. I2：T7a 已绿的 5 套 desktop_mcp（calculator/todo/notes/charts/
   dashboard）全绿；minesweeper/041 存量失败不扩大（其修复在独立任务，
   不在本计划门禁内）。
4. I3 复查：grep 无 standalone/desktop 双路径分支；`cargo test -p
   auto-lang --features ui-iced` ui:: 子集回归绿（plan411 既有失败豁免）。

## 5. 风险

| 风险 | 缓解 |
|---|---|
| daemon 迁移触碰启动链路（lib.rs 组装段 500 行） | T1 施工图先行；T2 单独提交可回滚；单窗口行为作 I2 硬门槛 |
| 无窗口上下文的消息（widget 回调）路由歧义 | view per-window 打标包装；T1 定案消息形状，必要时过渡期主窗口缺省 |
| 第二 App 组件来源影响面（同源双实例 vs 异源） | demo 先 ① 同源双实例；② 异源加深为可选任务 |
| MCP/DevTools 多窗口行为未知（N1 截图、焦点） | T1 评估列出；不阻塞 demo，缺口记 454 |

## 6. 关联

- **Plan 453**：本计划是其 T7b 的载体；完成后 453 一并归档。
- **Design 23**：M1 里程碑收口；R4 接缝与 454 的前置。
- **存量问题（不属本计划）**：minesweeper/041 快照 onclick 绑定行缺失
  （T7a 已基线定性，8/22 后、T4c 前引入）——需独立小计划修复。
