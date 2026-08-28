# Plan 462: VirtualWindow + WM 最小集（单 OS 窗口多 App，路线 A）

> **状态**：已立项 2026-08-28，未开工（多 agent 可领取）
> **来源**：产品需求「桌面端虚拟桌面」（Design 24 §1 N1）；里程碑 M2（Design 23 §6
> 提案编号 454，实际编号经程序跟踪文件「计划一览」解析为本号）。
> **架构依据**：`docs/design/23-autoui-virtual-desktop.md`（R1 WM-as-app、R2 单 OS 窗口
> 虚拟桌面、R4 AppWindow 接缝、R5 路线 A 先行、I2/I3）；`docs/design/24-autoui-desktop-shell-and-launcher.md`
> （R9 排布纯函数、R12 桌面热键、§6 风险）。
> **基线**: master 1487b5c5d（453/459 已归档）。
> **本计划产出是 463/464/465 的共同地基**：VirtualWindow widget 契约（I4 登记源）。

## 1. 目标

把 459 的「每 App 一个 OS 窗口」推进为「**一个 OS 窗口内承载 N 个 App**」（路线 A）：

1. `VirtualWindow` 容器 widget：裁剪、平移、事件区域路由、焦点分区、z-order；
2. 最小 WM：窗口矩形/焦点/z 的 state 模型 + chrome（标题栏、关闭、拖拽移动、边缘 resize、
   点击聚焦）；
3. 桌面宿主入口：单 OS 窗口 + N 个虚拟窗口（demo：两个不同 App 各占一个可拖拽虚拟窗口）；
4. R4 接缝 v1 落形：叶子 = Element 子树；`DesktopMessage` 外壳扩 WM 变体；
5. I2 硬门槛：`auto run`（独立窗口模式，R3 退化桌面）行为零回归。

**非目标**（明确出界）：全屏/任务栏/launcher/排布/注册表（463/464）；vue 端（465）；
DesktopBus 完整跨 App 路由（463 只做生命周期命令子集）；MCP `(AppId, widget)` 寻址
（T8 冻结延续，desktop 模式 v1 指向焦点窗/主 App）；多工作区；路线 B（386）。

## 2. 关键事实（2026-08-28 代码盘点）

- **会话层**：`DesktopSession/AppSession`（`crates/auto-lang/src/ui/session.rs`，677 行）——
  `allocate_app`(L304)、`register_window/unregister_window`(L327/L340)、`app_of_window/
  window_of_app`(L345/L350)、三向借拆 `split_mut/split_ref[_at]`(L361-408)、
  `DesktopMessage{App,Desktop,Window}`(L257，注释 L255 已预留 454 复用)、
  `WindowEntry` 硬 1:1 app↔OS-window(L222 注释：454 由 WM 接管)。
- **入口**：`run_dynamic_iced_multi`（`crates/auto-lang/src/ui/iced/renderer.rs:6060`）——
  iced::daemon；boot 期 `window::open` 同步返回 Id(L6247-6274，级联偏移 80+48*i)；
  view 按窗口反查 App(`view_desktop_fn` L8246)；update 三分派(L8150-8239)；
  全窗关→`iced::exit()`(L8203)。
- **键盘**：per-App `keyboard_subscription(app, my_window, bindings)`
  (renderer.rs:5785，按 (AppId, window) 过滤的 filter_map)；`keyboard_event_message`
  (L5808，F12 DevTools 优先于 Captured 守卫)；**无桌面级（跨 App）键处理层**——
  R12 的拦截点需要新建。
- **焦点**：452 spike 结论（`docs/plans/reports/452-ime-spike.md`）——iced focus id
  体系可按虚拟窗口分区；候选框定位构造性正确（含全屏）。残留两项：组合中失焦 discard、
  preedit 落盘（§4 T6 消化）。
- **VirtualWindow/WM 现状为零**（grep 仅注释）；widget 登记体系：
  `WidgetRegistry`（`crates/auto-lang/src/ui_gen/widget/registry.rs`）+ `schema/aura.at`
  （I4 登记源；`virtual_desktop`/`app_window` meta 映射模式已在 Design 23 §5 预订）。
- **已知 latent bug**（顺手修复归位）：`pending_window_resize` 经 `iced::window::oldest()`
  消费（renderer.rs:8109-8118）——多窗口下会改错窗；随窗口上下文消息化一起修。
- **panic 隔离**：`AUTOUI_PANIC_PROBE`（update catch_unwind + view 崩溃页，459 T5）
  直接沿用，虚拟窗口粒度同样适用。

## 3. 设计要点与决策点

### 3.1 VirtualWindow 实现载体（T1 spike 定案，两候选）

- **候选 A（倾向）**：自定义 `Widget` 实现——完全掌控 `draw`（子树平移 + clip bounds）、
  `layout`、`on_event`（矩形命中测试：cursor 不在窗矩形内的事件吞掉/改道 WM，
  在窗内则透传子树并附加窗口上下文）、`focus` 分支。iced 0.14 自定义 Widget 面积可控
  （container 套 element 的子树直接递归）。
- **候选 B**：组合现有 widget（`container().clip()` + `translate`）+ 宿主级事件
  `listen_with` 做命中路由。实现快，但事件吞噬语义（点击穿透到下层窗）不可精确控制。
- spike 产出：两候选各一个最小可点 demo（窗内 text_input 可输入、点击窗外不聚焦窗内）、
  定案理由写入 `docs/plans/reports/462-t1-virtual-window-spike.md`。

### 3.2 WM 状态与消息

- `WindowState { app: AppId, title, rect: Rect, z: u64, minimized?: v1 无 }`，
  存 `DesktopSession`（新增 `wm: WmState`），**复用 459 的窗口注册表语义改造**：
  desktop 模式下 OS 窗口仅 1 个（宿主窗），`window_of_app` 语义退化为虚拟窗句柄
  （建议引入 `Wid(u64)` 虚拟窗口 id，`WindowEntry` 拆为「宿主窗 1:N 虚拟窗」）。
- `DesktopMessage` 扩第四变体 `DM::Wm(WmCommand)`（R12 拦截点与 chrome 回调的载体）：
  `Focus(Wid) / Close(Wid) / Move(Wid, Point) / Resize(Wid, Size) / Raise(Wid)`。
  chrome（标题栏按钮/拖拽）由 VirtualWindow widget 发出。

### 3.3 view 分层（desktop 模式）

`view_desktop_fn` 在 desktop 模式下组装：背景层 → `z-stack(VirtualWindow...)` →
overlay 预留槽（463 任务栏/464 launcher 的挂载点，v1 空元素）。每虚拟窗口 =
`split_ref_at(app, wid)` 子树包进 VirtualWindow（chrome 内建：标题 + 关闭按钮，
标题来自 App 的 widget_name / pac title）。

### 3.4 键盘路由改造（R12 前半）

459 的 per-App 键盘订阅在 desktop 模式下失效前提（所有 App 同一 OS 窗口）——
改为**桌面层路由**：`keyboard_event_message` 前置一个桌面级处理段（F12 等桌面键 →
WM 键 → 未捕获才投递焦点窗的 App bindings）。`desktop.current_modifiers` 机制保留。
**I2 保障**：独立模式（无 WM state）走原路径，代码上表现为「配置差异」而非分支复制
（I3：以 desktop 模式 = `wm.is_some()` 之类的配置开关表达，禁止复制粘贴双路径）。

### 3.5 接缝 v1（R4）

`AppWindow` 枚举第一形态 `Element(Bunch<AuraNode>)` 落为代码事实：`WmState.windows`
持有 `AppWindow`，view/事件/订阅均经由 `Wid` 寻址；后续 RenderCommand/Wayland/DOM
叶子只新增枚举臂。**I1 检验点**：本计划不删 453/459 任何能力。

## 4. 任务表

| # | 任务 | 内容 | 验证 |
|---|---|---|---|
| T1 | VirtualWindow spike | §3.1 两候选最小 demo + 定案报告 `reports/462-t1-virtual-window-spike.md` | `cargo run -p auto-lang --features ui-iced --example <spike>` 实机：窗内输入、窗外点击不穿透 |
| T2 | WM state + 消息 | `session.rs`：`Wid`/`WindowState`/`WmState`/`WmCommand`、宿主窗 1:N 虚拟窗注册表改造；`DesktopMessage::Wm` | `cargo check -p auto-lang --features ui-iced` + `cargo t session`（现有单测绿） |
| T3 | VirtualWindow widget | 按 T1 定案实现：clip/translate/on_event 命中路由/focus 分支/z；`registry.rs` + `schema/aura.at` 登记（I4，含 a2vue 占位说明，465 消费） | T2 命令 + `cargo t ui_gen`（登记漂移测试绿） |
| T4 | chrome | 标题栏/关闭/拖拽移动/边缘 8 向 resize/点击聚焦，发 `DM::Wm` | 并入 T5 demo 实机操作 |
| T5 | 桌面宿主入口 | `renderer.rs`：desktop 模式（单宿主窗 + N 虚拟窗 + z-stack view + overlay 预留槽）；`pending_window_resize` 窗口上下文化修复；新 example `crates/auto-lang/examples/ui_desktop.rs`（复用 `build_dynamic_component`，两个不同 .at：建议 `459-dual-app` + `011-calculator`） | `cargo run -p auto-lang --features ui-iced --example ui_desktop` 实机：两窗拖拽/缩放/关闭/聚焦互不串扰 |
| T6 | 键盘/焦点/IME | §3.4 桌面层路由；focus id 分区（AppId 前缀命名空间）；452 残留两项（组合失焦 discard、preedit 落盘） | 实机：两窗各含输入框，Tab/点击切换焦点不串扰；中文 IME 组合输入随焦点走 |
| T7 | 回归与收尾 | I2 复跑（5 套 desktop_mcp 全绿）；I3 grep 检查；`AUTOUI_PANIC_PROBE` 虚拟窗粒度复验；459 demo（独立模式）不回归 | `cargo t`（ui:: 子集 + desktop_mcp 五套）；grep 无 standalone/desktop 双路径 |

## 5. 验收

1. `ui_desktop` demo：**一个 OS 窗口**、两个不同 App 各占一个虚拟窗口，可拖拽/缩放/
   关闭/点击聚焦，焦点/输入/IME/panic 隔离互不串扰。
2. I2：独立模式五套 desktop_mcp（calculator/todo/notes/charts/dashboard）全绿；
   459 双窗口 demo 行为不变。
3. I3：无第二条 standalone/desktop 代码路径（配置差异表达）。
4. `virtual_window` widget 进入登记源（schema/aura.at + WidgetRegistry），带
   a2vue 金样占位（465 填充 DOM 实现）。

## 6. 风险

| 风险 | 缓解 |
|---|---|
| 自定义 Widget 事件/焦点深水区超预期 | T1 双候选 spike 先行，B 候选兜底；单 OS 窗口内先不做 IME 全覆盖，按 452 spike 结论兜底宿主级输入层 |
| session.rs 注册表改造波及 459 已验证行为 | T2 先行、独立提交；I2 五套 desktop_mcp 作硬门槛；`split_*_at` 已有窗口维度参数可复用 |
| chrome 拖拽与 App 内拖拽语义冲突（如 minesweeper） | VirtualWindow 命中测试只认 chrome 区域（标题栏/边缘把手），客户区事件全量透传 |
| renderer.rs 13.7k 行改造体量 | desktop 模式逻辑尽量新增模块（`wm.rs`），renderer 只留接缝；T5 单独提交可回滚 |

## 7. 并发边界（多 agent）

- **拥有**：`crates/auto-lang/src/ui/session.rs`、`crates/auto-lang/src/ui/wm.rs`（新）、
  `crates/auto-lang/examples/ui_desktop.rs`（新）、`schema/aura.at` 的
  `virtual_window` 项、renderer.rs 的 desktop 模式接缝段。
- **避让**：441/437 若并行在 renderer.rs/输入侧施工，按「先合先 rebase」错峰；
  vue/a2vue 侧只登记不实现（465 独占）。

## 8. 关联

- 依赖：459 ✅（daemon、AppId 分配、panic 边界、窗口注册表）。
- 下游：463（消费 WM state 与 `DM::Wm`）、464（消费 overlay 槽与焦点分区）、
  465（消费 widget 登记契约）。
- Design 23 §8.7「键盘归属、MCP 寻址」的键盘半边在本计划落地；MCP 半边登记残留。
