# AutoGraphics 实验：嵌套输入（SD-02 草稿）

> 状态：**实验合同草稿**（PLAN-830 work 阶段产出，待 review/merge 后正式沉淀）。
> 范围同 `surface-composition.md`：独立实验 workspace、单窗口、Windows 键鼠。

## 1. 坐标链

```
窗口物理像素
  → 顶层 View 内容逻辑像素（host 换算，可含 DPI 缩放）
  → 每层挂载：2D 仿射反算（offset/scale）+ clip 判定
              或 3D 射线（父 Surface 相机）→ 平面 UV → 子内容像素
  → 子 Surface 局部坐标（左上原点）
```

每层按实际挂载路径转换，不允许只减窗口偏移。映射往返自洽（实测误差 0.000px）。

## 2. 命中与派发

| 规则 | 约定 | 反例（必须拒绝/不交付） |
|---|---|---|
| 前后序 | View 内 Portal 按声明序绘制；命中按**逆声明序**取第一个接受的挂载 | 下层先收到点击 |
| clip | 2D 挂载：点必须在 clip 矩形且落在子内容边界内 | 内容越界仍被点击 |
| 遮挡 | 非交互挂载仍**吞掉**命中，不向下穿透；显式 passthrough 由挂载属性表达 | overlay 后下层重复派发 |
| 3D 平面 | 射线与平面矩形求交，UV 再乘子内容尺寸；未命中则该挂载不参与 | 平面外点进入子内容 |
| 一次派发 | 一次事件只交付给最深命中的 Surface 一次 | 父子重复收到同一事件 |
| 非法输入 | 非有限坐标（NaN/Inf）直接拒绝且不产生 capture | 传播 NaN 到子内容 |

## 3. 捕获（capture）

- 按下即捕获到**挂载路径**（不是仅 Surface），后继 move/up 沿同一路径交付，
  即使指针移出子矩形（坐标按挂载边界钳制）。
- 目标被销毁/卸载、窗口失焦、指针离开窗口 → **取消**：递增 generation 并记录
  `CancelReason`，迟到 release 不再命中任何对象。

## 4. 焦点与键盘

- 焦点是**唯一**目标，点击即迁移；同一时刻只有一个键盘接收者。
- 键盘事件只投递给焦点 Surface；非 iced 内容不接收文本事件。
- 焦点路径失效（挂载被删/内容销毁）即清除，不保留悬空编辑状态。

## 5. IME

- `Event::InputMethod::{Opened, Preedit, Commit, Closed}` 经焦点 Surface 的真实
  `text_input` 处理：preedit 不写入值、commit 生效、关闭后无残留。
- 候选窗锚点：由子内容局部坐标（如光标）沿挂载链**正向投影**到窗口像素
  （2D 仿射 / 3D 平面透视），再转屏幕坐标交给 OS
  （`ImmSetCompositionWindow` CFS_POINT）。锚点必须落在该挂载的窗口投影区域内，
  不能回落到离屏纹理原点。
- **证据边界**：合成 IME 事件只证明逻辑；OS 级中文组合同样需要真实输入法
  的屏幕证据。本 Plan 未取得该证据（见 `research/surface-result.md` §6），
  不得以合成事件代替。

## 6. 事件来源诚实标注

窗口模式优先真实前台输入（`mouse_event` / `SendKeys`）；会话拒绝前台时回退
`PostMessage`（仍是 winit 真实消息管线）。报告字段 `input_evidence` 记
`os_window_events` 或 `synthetic_routing`，不把合成事件冒充 OS 输入。
