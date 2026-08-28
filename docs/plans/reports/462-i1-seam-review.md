# 462 R4 接缝 I1 评审（386 复活入口 · 一次评审门）

> **结论：I1 通过（v1 接缝就位，附两个加法扩展点登记）。**
> 评审人：ZCode（2026-08-28，master 76dc48a02）；依据 Design 23 §7 I1：
> "386 复活时，453/454/455 的代码**零删除**——只发生在 R4 接缝之后被替换的
> 渲染叶子。若复活要改 WM/会话/事件路由，即接缝失效。"
> 评审对象：462 交付的接缝 v1（commit cdc07dcab）。

## 1. 检验 A：零删除（字面检验）

对 462 diff 逐删除行核查（`git show cdc07dcab`）：

- session.rs 删除 2 行 = 注释改写（"454 由 WM 接管"预言兑现）；无类型/能力删除。
- renderer.rs 删除段全部为三类：① boot 开窗循环**原样搬入** `RunMode::Standalone`
  分支（逐行保留）；② keyboard_subscription 签名扩展（过滤语义保留、
  identity 加 focused/desktop 位）；③ `window::oldest()` resize 退役（bug 修复，
  I2 五套 desktop_mcp 全绿证行为保持）。
- **判定：零删除成立**。453 会话层、459 daemon 多窗口、独立窗口模式全部原样可用。

## 2. 检验 B：叶子可替换性（386 复活时的改动面推演）

假设 386 的 RenderCommand 叶（路线 B：App 离屏渲染、桌面合成纹理）替换
Element 子树叶，逐层核查接触面：

| 层 | 接触面 | 386 复活时需改动？ |
|---|---|---|
| WM/会话 | `VWinState` 只持有几何/标题/z（session.rs:293），**不含任何渲染形态** | ❌ 零改动 |
| 消息外壳 | `DesktopMessage{App,Desktop,Window,Wm}` 与叶子形态无关（(AppId,·) 扇出已参数化） | ❌ 零改动 |
| view 组合 | 叶子构造点**单一**：desktop 分支 per-vwin 的 `dynamic_view(view, is_primary)`（renderer.rs:8510）→ 交给叶子无关的 `virtual_window_element`（定位/clip/chrome/把手只吃 `Element` + 几何） | ✅ 单点替换：该调用点换 RenderCommand 叶产出（纹理/图层元素） |
| chrome/事件 | Stack z 序命中、mouse_area 捕获、`__mouse_*`/GlobalPress 全在**叶子之外**的桌面层 | ❌ 零改动 |
| 输入投递 | 现接缝 v1 隐含"叶子在宿主 widget 树内"——widget 事件由树天然投递 | ⚠️ **加法扩展点 E1**（见 §3） |

## 3. 加法扩展点登记（不构成 I1 违背；386 复活时按此追加，不删既有代码）

- **E1 路线 B 事件再注入**：叶子不在宿主树内时，widget 级事件不可天然达——
  需在 `WmState` 命中矩形（已有）与 `DM::App` 扇出（已有）之间补一条
  "指针/键盘 → (AppId, event) 注入"通道。设计 23 R4 输入侧定义
  （"接缝的输入侧统一为 (AppId, event) 扇出与区域矩形"）即为此预留，
  实现为**新增**枚举臂/通道，不动 453/455 存量。
- **E2 AppWindow 枚举显式化**：v1 的叶子是隐式接缝（一个函数调用点）；
  Design 23 R4 的 `AppWindow` 枚举（Element | RenderCommand | Wayland | DOM）
  落型属加法定义，替换时把调用点泛型化即可。
- 465 的 DOM 叶落地时若先行实现 E1/E2（同构需求），386 复活成本进一步下降
  ——建议 E1/E2 挂 465 计划消费。

## 4. 判定

**I1 通过**：view 侧零删除替换可达（单点），WM/会话/消息外壳/事件路由核心
无需改动；输入侧扩展为 R4 预留的加法项。仪表盘 386 复活条件第 3 项据此核销
（剩余两项：常驻 App ≥ 3、内存实测）。

## 5. 关联

- Design 23 §7 I1、§8.2（386 重新定位 R7）；跟踪文件仪表盘。
- 462 spike 报告 §5（接缝 v1 事实清单）；465（E1/E2 建议承载者）。
