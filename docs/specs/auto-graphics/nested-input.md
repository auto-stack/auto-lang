# AutoGraphics 实验：嵌套输入（SD-02 草稿）

> 状态：**实验合同草稿**（PLAN-830 work 阶段产出，待 review/merge 后正式沉淀）。
> 范围同 `surface-composition.md`：独立实验 workspace、单窗口、Windows 键鼠。
> 2026-10-22 r6 work：按 review r4 F-830-R4-01 更新（取消先于 pending 渲染时
> 必须保持无编辑态；router 目标改为 Unset/Target/Cancelled 三态）。

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

**路由输出必须被真正的消费者使用**：router 返回的 `(surface, local)` 就是控件
事件/游戏动作的输入；调用方不得再用自己预设的坐标另走一条路径。正式场景门禁
同时断言：交付 Surface 正确、返回 local 与目标控件位置往返一致、且在目标内容
边界内。受控故障（把返回 local 改成 [9999,9999]）必须让对应断言失败并整体非零。

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
- **取消先于 pending 渲染**：宿主在 `about_to_wait` 才 flush 事件，因此 blur /
  卸载 revalidate 可能发生在已路由事件渲染之前。router 目标因此是**三态**：
  `Unset`（尚未路由，允许 host 自行 focus）、`Target(id)`（唯一真实目标）、
  `Cancelled`（失焦/路径销毁，**这之后的 pending 批次不得重新产生任何编辑态**）。
  取消后到新的合法点击/公开 focus 之前，所有 iced 面的 widget 焦点必须为空、
  无 caret/组合态；新的点击仍正常建立目标。

## 4. 焦点与键盘

- 焦点是**唯一**目标，点击即迁移；同一时刻只有一个键盘接收者。
- **合批输入以 router 终态为准**：宿主把多次 routed 点击攒进同一帧时，runtime
  不接受“本帧某个事件列表里有 ButtonPressed”这种无序推断，而是读取 router 记录
  的最终目标（`CompositionGraph::note_routed_focus/routed_focus`）并在该面绘制
  前清除其他 iced 面的真实焦点与组合态；A→B 与 B→A 合批都以最后一次 routed
  press 为准，且先前点击的动作仍被投递（只是其焦点效果被覆盖）。
- **拒绝无副作用**：公开 focus 对不存在/失效（stale）/跨 session/非 iced 的
  Surface 必须先校验后动作；拒绝时当前合法焦点与 IME 状态保持不变，后续合法
  focus 仍迁移唯一目标。
- **焦点迁移贯通到真实控件**：点击/公开 focus 调用把焦点移走后，运行时在**同一帧
  渲染前**释放其他 iced Surface 的真实 widget 焦点并关闭其组合态（`Closed`）。
  router 侧记录 `FocusTransfer{from,to}` 供宿主显式释放；即使宿主忘记，运行时也不
  允许两个真实控件同时保留编辑态。
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
- 候选窗锚点 = **真实 caret**：每帧在真实控件上取 iced `UserInterface::update`
  返回的 `InputMethod::Enabled.cursor`（由 text_input 依布局、光标 index、选择与
  滚动算出），沿**当前活跃焦点/捕获挂载路径**正向投影到窗口像素（2D 仿射 /
  3D 平面透视），再转屏幕坐标交给 OS（`ImmSetCompositionWindow` CFS_POINT）。
  共享内容有两个挂载时必须选用户正在编辑的那一个；根框返回自身局部坐标而非 None。
- 锚点必须落在**该挂载投影出的 caret 矩形内**（不是整个平面、不是整个窗口）；
  挂载移动、输入、光标移动、选择与滚动改变时重算并跟随。正式门禁用投影 caret
  矩形作 oracle，并断言它明显小于平面投影（不是把平面/固定点命名为 caret）。
- caret 由真实 widget 输出提供：宿主每帧注入 `RedrawRequested`（iced 正常帧语义），
  丢失 update `State` 视为实现缺陷。
- **证据**：已在真实窗口 + 系统中文 IME 下取得 OS 级组合提交
  （`surface-ime-os`：真实 caret 报告、typing/backspace 后 caret 跟随、锚点在投影
  caret 矩形内、commit=`abc你好你好`）；合成事件只作逻辑补充，不替代 OS 证据。
  候选窗逐像素模板比对仍未做。

## 6. 事件来源诚实标注

窗口模式优先真实前台输入（`mouse_event` / `SendKeys`）；会话拒绝前台时回退
`PostMessage`（仍是 winit 真实消息管线）。报告字段 `input_evidence` 记
`os_window_events` 或 `synthetic_routing`，不把合成事件冒充 OS 输入。

已知实现边界：
- iced 使用 `Null` 剪贴板（复制/粘贴/剪切不可用），不冒充完整编辑能力。
- iced 内容与 3D 场景每帧无条件重绘；dirty 跳过目前只对 2D canvas 生效，但
  适配器报告的“本帧输出变化”会在同帧传播给消费者。
- 3D 平面按声明序合成，未接深度缓冲（任意深度遮挡留给后续 profile）。
- `surface-ime-os` 的可执行断言与“环境不可用”分开分类：环境未就绪返回 4，
  可用环境下的产品断言失败返回 1，避免把产品缺陷误报为 blocked。
