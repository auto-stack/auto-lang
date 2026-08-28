# 462 T1 Spike 报告：VirtualWindow 实现载体定案

> **结论：采纳候选 B（组合现有 widget），候选 A（自定义 Widget）否决。**
> 状态：定案（2026-08-28，实机双窗 demo 全交互验证后回填）。

## 1. 两候选回顾（计划 462 §3.1）

- **候选 A（原倾向）**：自定义 `Widget` 完全掌控 draw/layout/on_event。
- **候选 B（定案）**：组合 `Stack` + `container.clip` + `mouse_area` +
  全局事件状态机，位置用 padding 定位包裹。

## 2. 定案依据（iced 0.14 源码实证）

| 需求语义 | iced 0.14 事实（源码行号见 iced_widget-0.14.2） | 载体 |
|---|---|---|
| z 序事件路由（顶层优先、捕获即停、空区穿透） | `Stack::update` 自顶向下逐层转发、`shell.is_event_captured()` 即停（stack.rs::update） | **Stack 免费提供** |
| 窗口矩形外事件不可达 | Stack 层布局原点在桌面左上（`Limits::new(ZERO, size)`），层内 padding 定位 + 定尺寸子树；命中天然按 widget 树矩形 | container + Padding |
| 空白点击聚焦 + 阻断穿透（不挡 App 组件） | `mouse_area`：子组件优先捕获，未捕获时 `on_press` 发布并 `shell.capture_event()`（mouse_area.rs::update） | 客户区包裹 mouse_area(Focus) |
| 子树绘制裁剪 | `container.clip(true)` 以 viewport 交集裁剪子树 draw（container.rs:351-366） | 窗体容器 clip |
| 拖拽/缩放 | `mouse_area` **无 on_drag**；改由全局事件状态机：chrome `on_press` 发 `StartDrag/StartResize`，移动/松开复用既有 `__mouse_moved`/`__mouse_released` 订阅（update 壳层拦截，`WmState.apply_cursor/end_interaction`） | DM::Wm + WmState |
| 点击聚焦（任意区域） | `ButtonPressed` 全局订阅发 `DM::Wm(GlobalPress)`，update 侧按 `WmState.last_cursor` 做 z 序命中（光标坐标由 CursorMoved 持续回写） | listen_with 半边 |

候选 A 被否决的理由：上述每一项语义候选 B 都已构造性达成，自定义 Widget 的
增量收益（像素级事件控制）为零，而成本（深水区 API、draw 正确性风险）不成立。

## 3. 实机验证（ui_desktop demo，Windows 11 / DPI 200%）

MCP 截图 + 真实鼠标/键盘（ctypes SendInput）驱动，全链路通过：

1. 单 OS 窗口双虚拟窗口渲染（chrome：标题栏/标题/×；焦点窗 accent 描边）；
2. 点击后排标题栏 → 置顶 + 焦点翻转；
3. 标题栏拖拽 → 窗口精确随动（左上锚定）；
4. SE 角把手缩放 → 精确放大（+80/+60 逻辑 px，左上不动）；
5. 真实点击输入框 + Unicode 键盘输入 → `value: "hi"`（Stack 层级命中 +
   焦点分区 + 键盘路由全链路）；
6. × 关闭单窗 → App 随窗移除，余窗存活；全关 → 进程退出（daemon 语义）；
7. `AUTOUI_PANIC_PROBE=1`：Crash 按钮后 update/view 双边界各拦截一次 panic
   （仅 AppId(1)），另一 App 继续可交互（真实点击出 7）。

## 4. 配套发现与修复（ spike 期间实测暴露）

1. **MCP 截图请求窃取 bug（已修，`renderer.rs` dynamic_view 收尾段）**：
   `take_screenshot_request()` 原先无 `sync_mcp` 门控——desktop 模式下非
   primary App 的视图会偷走共享请求，而它们几乎收不到 update，请求滞留至
   工具超时。修复后仅 primary 拾取（T8 单 App 语义对齐）。
2. **空闲帧泵（新增 `desktop_service_tick`）**：desktop 模式订 400ms
   `ServiceTick`（可 `AUTOUI_MCP_DISABLE=1` 关闭）——MCP 截图请求靠
   "update 拾取 / view 投递"，空闲时无消息流动则永不消费；独立模式不订
   （保持空闲零开销，I2 不受影响）。

## 5. 对后续计划的影响

- 463 排布引擎直接消费 `WmState.rect`（位置唯一事实源已就位）；
- 464 launcher overlay 挂载点 = desktop_root 的层列表追加位（Stack 顶层）；
- 465 的 DOM 叶按本报告的语义清单（穿透/聚焦/裁剪/拖拽）逐条对拍；
- `virtual_window` 的 schema/WidgetRegistry 登记随 465 一并落（本计划
  chrome 为 renderer 内部组合，无 .at 消费路径，单端登记即死代码）。
