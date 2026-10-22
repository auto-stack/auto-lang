# AutoGraphics 实验：嵌套输入（SD-02 草稿）

> 状态：**实验合同草稿**（PLAN-830 work 阶段产出，待 review/merge 后正式沉淀）。
> 范围同 `surface-composition.md`：独立实验 workspace、单窗口、Windows 键鼠。
> 2026-10-22 r3 work：按 review r1 F04/F05/F06/F07/F09 修复后的实际代码更新。

## 1. 坐标链

```
窗口物理像素
  → 顶层 View 内容逻辑像素（host 换算，可含 DPI 缩放）
  → 每层挂载：2D 仿射反算（offset/scale）+ clip 判定
              或 3D 射线（**该挂载的 host Surface** 相机）→ 平面 UV → 子内容像素
  → 子 Surface 局部坐标（左上原点）
```

每层按实际挂载路径转换，不允许只减窗口偏移。映射往返自洽（实测误差 0.000px）。
3D 射线只接受当前 host Surface 注册的相机：未注册即 miss，不借用其他父面相机的
命中（多父面/共享子面换序后结果确定）。

## 2. 命中与派发

| 规则 | 约定 | 反例（必须拒绝/不交付） |
|---|---|---|
| 前后序 | View 内 Portal 按声明序绘制；命中按**逆声明序**取第一个接受的挂载 | 下层先收到点击 |
| clip | 2D 挂载：点必须在 clip 矩形且落在子内容边界内 | 内容越界仍被点击 |
| 遮挡 | 非交互挂载仍**吞掉**命中，不向下穿透；该结果沿**整条递归链**传播，父内容/兄弟也不收到 | 内层 overlay 后父面回落收到点击 |
| 3D 平面 | 射线与平面矩形求交，UV 再乘子内容尺寸；未命中则该挂载不参与 | 平面外点进入子内容；借用错误相机 |
| 一次派发 | 一次事件只交付给最深命中的 Surface 一次 | 父子重复收到同一事件 |
| 非法输入 | 非有限坐标（NaN/Inf）直接拒绝且不产生 capture | 传播 NaN 到子内容 |

## 3. 捕获（capture）

- 按下即捕获到**挂载路径**（不是仅 Surface），后继 move/up 沿同一路径交付，
  即使指针移出子矩形（坐标按挂载边界钳制）。
- 目标被销毁/卸载、窗口失焦、指针离开窗口 → **取消**：递增 generation、清空
  capture **和键盘焦点**，记录 `CancelReason`；迟到 release 不再命中任何对象。
- 取消返回被清掉的焦点 Surface，宿主据此释放 iced 焦点/编辑态并关闭 IME。

## 4. 焦点与键盘

- 焦点是**唯一**目标，点击即迁移；同一时刻只有一个键盘接收者。
- 键盘事件只投递给焦点 Surface；非 iced 内容不接收文本事件。
- **无焦点不 fallback**：没有焦点目标时键盘/IME 事件被丢弃（并计数），不投给
  旧的编辑框。
- 焦点路径失效（挂载被删/内容销毁）即清除（`revalidate_focus` → `cancel`），
  宿主对结构变更执行 revalidate，不保留悬空编辑状态。
- `WindowEvent::Focused(false)` / `CursorLeft` 接到同一取消路径：capture+焦点
  清空、iced `unfocus`、发送 `Window::Unfocused`/`InputMethod::Closed`。

## 5. IME

- `Event::InputMethod::{Opened, Preedit, Commit, Closed}` 经焦点 Surface 的真实
  `text_input` 处理：preedit 不写入值、commit 生效、关闭后无残留。
- 候选窗锚点：由**当前活跃焦点/捕获挂载路径**（没有焦点时取顶层根 Surface）把
  子内容局部坐标（如光标）沿挂载链**正向投影**到窗口像素（2D 仿射 / 3D 平面
  透视），再转屏幕坐标交给 OS（`ImmSetCompositionWindow` CFS_POINT）。共享内容
  有两个挂载时必须选用户正在编辑的那一个；根框返回自身局部坐标而非 None。
- 锚点必须落在该挂载的窗口投影区域内（不只是“整个窗口内”）；挂载移动后锚点
  随之移动（`surface-ime-os` 的 `anchor_tracks_the_active_mount`）。
- **证据**：已在真实窗口 + 系统中文 IME 下取得 OS 级组合提交
  （`surface-ime-os` 7/7 + 新增焦点/锚点门禁，commit=`abc你好你好`）；
  合成事件只作逻辑补充，不替代 OS 证据。候选窗逐像素模板比对仍未做。

## 6. 事件来源诚实标注

窗口模式优先真实前台输入（`mouse_event` / `SendKeys`）；会话拒绝前台时回退
`PostMessage`（仍是 winit 真实消息管线）。报告字段 `input_evidence` 记
`os_window_events` 或 `synthetic_routing`，不把合成事件冒充 OS 输入。

已知实现边界：
- iced 使用 `Null` 剪贴板（复制/粘贴/剪切不可用），不冒充完整编辑能力。
- iced 内容与 3D 场景每帧无条件重绘；dirty 跳过目前只对 2D canvas 生效。
- 3D 平面按声明序合成，未接深度缓冲（任意深度遮挡留给后续 profile）。
- `surface-ime-os` 的可执行断言与“环境不可用”分开分类：环境未就绪返回 4，
  可用环境下的产品断言失败返回 1，避免把产品缺陷误报为 blocked。
