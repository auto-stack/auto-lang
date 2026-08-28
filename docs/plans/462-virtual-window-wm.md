# Plan 462: VirtualWindow + WM 最小集（单 OS 窗口多 App，路线 A）

> **状态**: ✅ 完成（2026-08-28，T1–T7 全量 + 实机验收 + 全量回归绿）
> **来源**：产品需求「桌面端虚拟桌面」（Design 24 §1 N1）；里程碑 M2（Design 23 §6
> 提案编号 454，实际编号经程序跟踪文件「计划一览」解析为本号）。
> **架构依据**：`docs/design/23-autoui-virtual-desktop.md`（R1 WM-as-app、R2 单 OS 窗口
> 虚拟桌面、R4 AppWindow 接缝、R5 路线 A 先行、I2/I3）；`docs/design/24-autoui-desktop-shell-and-launcher.md`
> （R9 排布纯函数、R12 桌面热键、§6 风险）。
> **基线**: master ba17b6c75（453/459 已归档）。
> **本计划产出是 463/464/465 的共同地基**：VirtualWindow widget 契约（I4 登记源）。
> **T1 施工图**: `reports/462-t1-virtual-window-spike.md`（定案候选 B：组合
> Stack/container.clip/mouse_area + 全局事件状态机；iced 0.14 源码级依据 + 实机验证记录）。

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
| T1 | VirtualWindow spike | ✅ 完成：**定案候选 B**（组合方案，候选 A 否决）——Stack z 序事件短路 + container.clip + mouse_area 捕获 + 全局事件状态机；报告 `reports/462-t1-virtual-window-spike.md`（含 iced 0.14 源码依据与实机记录） | 实机 demo 全交互通过 |
| T2 | WM state + 消息 | ✅ 完成：`session.rs` `Wid/VWinState/WmState/WmInteraction/WmCommand`、`DesktopSession.host`（I3 配置位）、`DesktopMessage::Wm`、宿主窗 1:N 注册表、`split_*_at` desktop 分支；新增 5 个 WM 单测 | `cargo t session::` 19/19 绿 |
| T3 | VirtualWindow 组合层 | ✅ 完成：新 `ui/iced/virtual_window.rs`（desktop_root + virtual_window_element：定位包裹/窗体 clip+阴影+焦点描边/标题栏 chrome/客户区聚焦包裹/八向缩放把手）。**计划修正**：schema/WidgetRegistry 登记移至 465（v1 chrome 为 renderer 内部组合、无 .at 消费路径，单端登记即死代码；I4 双端同源随 DOM 叶落地） | 编译 + 实机渲染验证 |
| T4 | chrome | ✅ 完成：标题栏拖拽（`StartDrag` + `WmState` grab 状态机）、× 关闭（App 随窗移除）、八向缩放把手、点击聚焦置顶（客户区包裹 + `GlobalPress` 命中双通路） | 实机：拖拽/缩放/关闭/聚焦全过 |
| T5 | 桌面宿主入口 | ✅ 完成：`run_session(components, RunMode)` 单管线拆分（Standalone/Desktop 仅 boot 开窗与 view 组装分叉）；`run_dynamic_desktop` 入口；boot 单宿主窗 + N 虚拟窗级联；新 example `crates/auto-lang/examples/ui_desktop.rs`；**额外**：① `pending_window_resize` 按视图窗口直指（退役 `window::oldest()` 多窗猜测 bug）② desktop 模式 `__window_resized` 同步全体虚拟窗 window_size ③ `desktop_service_tick` 400ms 帧泵（MCP 截图消费链路）④ 修复 `take_screenshot_request` 无 sync_mcp 门控被非 primary App 窥窃的滞留 bug | `cargo run --example ui_desktop` 实机全交互 |
| T6 | 键盘/焦点/IME | ✅ 完成（结构部分）：订阅 focused 门控（identity 含 focused，焦点翻转重订阅，R12 订阅半边）；F12 DevTools 在 desktop 模式禁用（固定 widget id 同窗撞车，T8 语义延续）。**残留**：452 两项 IME 任务（组合中失焦 discard、preedit 落盘）未做——焦点分区结构已实测可用（真实点击+Unicode 键盘入 `value:"hi"`），IME 组合行为需分平台人工矩阵，转 463 前置确认项 | 实机键盘流通过 |
| T7 | 回归与收尾 | ✅ 完成：`cargo t` 3222/3222 绿；I2 五套 desktop_mcp 全绿（calculator 14、todo 11、notes 11、charts 19、dashboard 26，0 失败）；I3 复查（全部 desktop 分叉以 `host.is_some()` 配置位表达，无双路径）；panic 隔离虚拟窗粒度复验 | 见验收 |

## 4.1 实机验收记录（2026-08-28，Windows 11 / DPI 200%）

MCP 截图（autoui_screenshot）+ 真实鼠标键盘（ctypes SendInput）驱动：

1. 单 OS 窗口双虚拟窗口（DualApp + calculator），chrome/焦点描边/z 序正确；
2. 点击后排标题栏 → 置顶 + 焦点翻转（描边换色）；
3. 标题栏拖拽 +200/+150 物理px → 窗口精确随动；
4. SE 角把手缩放 → +80/+60 逻辑px 精确放大、左上锚定；
5. 真实点击虚拟窗内输入框 + Unicode 键盘 → `value: "hi"`；
6. × 关闭单窗 → App 随窗移除、余窗存活；全关 → `iced::exit` 进程退出；
7. `AUTOUI_PANIC_PROBE=1` Crash 注入 → update/view 双边界拦截（仅本 App），
   另一 App 真实点击继续可交互。

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
