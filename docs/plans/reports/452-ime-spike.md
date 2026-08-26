# Plan 452 T6 报告：IME / 焦点 Spike

> **日期**：2026-08-26
> **环境**：Windows 11（26200），Microsoft 拼音（`0804:{3D02CAB6-…}`），
> iced 0.14.0 / iced_widget 0.14.2 / winit 0.30.13 / wgpu 27（与主 workspace 同版本同 feature 集），
> Chrome（Web 端）。
> **原型**：`scratch/ime-spike/`（Rust 双虚拟窗口原型 + Web 嵌套 div 页 + Win32 驱动脚本）。
> **证据**：`docs/plans/reports/assets/452-ime-spike/01–07.jpg`。
> **方法 caveat**：键盘输入经 `keybd_event` 合成注入（非真人敲击）。合成 VK 输入
> 会正常进入 TSF/IME 处理链（组合、候选、提交均发生），但不排除与真人输入在
> 边缘行为上有差异；标注 ⚠️ 的结论建议后续人工复测一次。

## 结论一览

| # | 验证项（Design 23 §9） | 定性 | 证据 |
|---|---|---|---|
| ① | iced 0.14 单窗口中文 IME 基线 | ✅ 通过（⚠️ preedit 语义缺陷记录在案） | 01 |
| ② | 组合中切换虚拟窗口 | ⚠️ 受限（详见下） | 03/04/07 |
| ③ | 全屏模式候选框定位 | ✅ 通过（跟随光标） | 05 |
| ④ | 双虚拟窗口焦点分区 | ✅ 通过 | 02/04 |
| ⑤ | Web 端嵌套 div 组合输入 | ✅ 通过（语义正确） | 07 |
| ⑥ | 宿主级输入层降级预案 | ✅ 通过（可行性确认） | 06 |

**无"阻断"级结论。** 453（多 App 会话运行时）立项条件满足；宿主级输入层
从"降级预案"升级为"已验证的可选模式"（不作为 454 硬需求）。

## 源码级基线（iced 0.14 IME 事件链路）

调研本地 registry 缓存源码确认的链路，纠正了两处常见误解：

```
winit 0.30 WindowEvent::Ime(Enabled|Preedit|Commit|Disabled)
  → iced_winit conversion.rs:320  转为
Event::InputMethod(input_method::Event::{Opened, Preedit(String, Option<Range>), Commit(String), Closed})
  → iced_widget 0.14.2 text_input.rs:1267 / text_editor.rs:780  消费
    （text_input 通过 input_method() 上报 preedit 区域 → 候选框定位）
```

- **误解 1**：IME 事件不是 `window::Event::Ime`——`iced_core::window::Event`
  **没有** Ime 变体；正确入口是顶层 `Event::InputMethod`（iced_core event.rs:29）。
- **误解 2**：text_input 不是没有 IME 支持——0.14.2 的 text_input 与 text_editor
  均有完整处理（preedit 状态、Commit 插入、Purpose 上报）。
- 其他 API 事实：`window::Id` 无 `MAIN` 常量（主窗口 id 由 shell 内部
  `window::open` 生成并丢弃，须经 `Event::Window(Opened)` + `listen_with`
  第三参数捕获——**这是 453 会话层必须自己管理窗口 id 的直接证据**）；
  `iced::application()` 0.14 首参为 boot 函数（非标题串）。

## 各验证项详情

### ① 单窗口 IME 基线（Windows / MS 拼音）

- 组合输入期间按键不产生 `Keyboard::KeyPressed`（被 TSF 吞掉），应用侧只见
  `IME Preedit("n", None)` … `IME Preedit("nihao", None)` 事件流；text_input
  内联渲染 preedit，候选条出现。
- 空格提交产生 `IME Commit("你好")`，文本落入输入框。
- **⚠️ 缺陷**：提交后输入框内容为 `nihao你好`——**preedit 字面文本未被清除，
  与提交文本叠加**（预期应为仅 `你好`）。02/04/06 号截图中 A/B 框均可见此现象。
  定性：iced 0.14 在 Windows 的 preedit 生命周期管理有缺陷（或与 TSF 内联
  组合的交互有缺陷）。**453/454 需跟踪**：焦点/提交时主动清除 preedit state，
  或上游修复。建议人工复测确认非合成输入伪影。

### ② 组合中切换虚拟窗口（双形态结论）

- **Tab 切换：不可用**。组合期间 Tab 被 IME 吞进 preedit（日志出现
  `IME Preedit("ni\t", None)`，"ni" 变 "ni\t"，焦点未移动）——证据 03。
  这是 IME 通用行为，非 iced 缺陷。
- **点击切换：可用但组合静默终止**：
  - iced 端：preedit 字面落盘（A 框残留 `ni\t`），无 Commit 事件，B 获得干净
    焦点（`Click → 聚焦 B` 由 mouse_area 路由记录）——证据 04。
  - Web 端：`A compositionend data="ni"` 且**无后续 input 事件 → preedit 被
    丢弃**，A 框只剩已提交内容——证据 07。
- **对 454 的裁定输入**：虚拟窗口的焦点丢失语义应以 **Web 的 discard 语义为
  规范**（组合中失焦 → 终止组合并丢弃 preedit）；iced 端需在 VirtualWindow
  的焦点管理里补齐该语义（配合 ① 的 preedit 清除）。

### ③ 全屏候选框定位

- F11 进入 `Mode::Fullscreen` 后在 B 框组合输入，**候选条紧跟 B 输入框光标
  位置**（屏幕右半区输入框下方），不回退到屏幕原点/中心——证据 05。
- 机制：text_input 上报的 preedit 区域已是窗口坐标，TSF 据此定位候选条。
- **推论（对 454 关键）**：单树嵌入下虚拟窗口的平移/布局由 iced 统一计算，
  候选框定位**天然正确，无需额外工作**——虚拟桌面任意摆放 App 时 IME 候选
  跟随是构造性保证。

### ④ 双虚拟窗口焦点分区

- Tab 循环（非组合态）在按钮/A 输入框/B 输入框间正确流转；点击任一虚拟窗口
  聚焦其输入框（mouse_area `on_press` 路由生效）；IME 组合始终跟随焦点窗口，
  未出现串窗口。验证 ②④ 期间无任何跨窗口 preedit 泄漏。

### ⑤ Web 端嵌套 div 组合输入

- 事件序列完整且按窗口正确归属：`A focusin → A compositionstart →
  A compositionupdate("n"…"nihao")`；提交时 `A compositionend data="nihao"` +
  `A input isComposing=false value="你好"`——**preedit 被提交文本正确替换**。
- 组合中 input 事件携带 `isComposing=true`，可用于区分组合态。
- 嵌套 div 容器（虚拟窗口）对组合零干扰；候选条跟随输入框。

### ⑥ 宿主级输入层

- F3 打开宿主层 → 自动聚焦宿主输入框（`widget::focus` 任务生效）→ 组合
  输入"nihao" → 空格提交"你好"至宿主层 → Enter 触发 `HOST→B(右) 提交 "你好"`
  （B 框追加文本）→ 宿主层清空并重新聚焦——全链路工作，证据 06。
- 结论：宿主级输入层是**可行的输入兜底模式**（体验取决于目标区域定位精度，
  本原型用"最近聚焦窗口"策略即可用）。

## 附带发现（453/454 设计输入）

- **F1 焦点任务时序**：窗口启动后首个 F1（`widget::operation::focus`）未生效，
  之后 F3 的同名任务正常——疑与首帧 widget 树就绪时序有关，453 会话层应把
  焦点请求做成可重放/延迟应用。
- **F2 a11y 实证**：iced 应用在 Windows UIA 树中只暴露窗口框架（标题/系统按钮），
  widget 树完全不可见——Design 23 §8 风险 3 的直接证据，虚拟桌面 a11y 需要
  自建 AccessKit 桥接（排期后补）。
- **F3 任务通路**：订阅消息返回的 `window::set_mode` / `widget::focus` 任务在
  纯 iced 0.14 应用中正常执行——renderer.rs:6048 的"Task 不执行"wart 是其动态
  渲染器包装层的局部问题，非 iced 0.14 通用缺陷（453 重构时可预期任务通路可用）。
- **F4 事件日志延迟**：InputMethod 事件的日志刷新存在可见滞后（UI 日志面板在
  后续交互后才显示此前事件），疑与 iced redraw 调度合批有关；不影响功能，
  但说明多 App 事件路由（453）需注意 redraw 触发时机。
- **F5 桌面级键盘监听天然可用（2026-08-26 用户实测补充）**：无任何 widget
  聚焦时（最近聚焦 "-"、两输入框皆空），全部按键仍被 `listen_with` 订阅层
  完整捕获（Key 'n'…'o' 五连记录）——桌面层不依赖焦点即可看到所有键盘事件。
  对 453：任务栏全局快捷键/桌面级热键路由无需额外机制；对 454：VirtualWindow
  只需处理"聚焦分区"，全局通道已在手。

## 资产与处置

- `scratch/ime-spike/`：Rust 原型（Cargo.toml/src/main.rs）、web/index.html、
  驱动脚本（drive.ps1 / drive_web.ps1 / fg.ps1）。按计划约定**不并入产品路径**，
  保留在 scratch 供复测。
- 证据截图：`docs/plans/reports/assets/452-ime-spike/01–07.jpg`。

## 对程序下一步的影响

1. 453 立项条件满足（本报告即 T6 交付物）。
2. Design 23 §8 风险 1（IME）从"最大风险"降级为"已知受限"：候选框定位构造性
   正确、宿主层兜底已验证；剩余工作明确（preedit 清除 + discard 语义）落在 454。
3. 建议在 454 的任务表中新增：「VirtualWindow 焦点丢失 → 组合 discard 语义
   （对齐 Web 行为）」与「preedit 字面落盘修复（含 iced 上游跟踪）」两条。
