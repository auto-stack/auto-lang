# AutoGraphics 实验：Surface 组合与 GPU 合成（SD-01 草稿）

> 状态：**实验合同草稿**（PLAN-830 work 阶段产出，待 review/merge 后正式沉淀）。
> 适用范围：`experimental/autographics-prototype/` 独立 workspace，Windows x64、
> 单原生窗口、wgpu 27.0.1 / iced 0.14。**不表示生产 AutoUI/RQHost 已支持。**
> 2026-10-22 r6 work：按 review r4 F-830-R4-02 收紧上传计量 oracle（分管线精确
> 断言 + 总量=分项之和 + 省略计数的负控）。

## 1. 概念

- **Surface**：有自身状态与更新的内容实例（iced UI、2D 游戏、3D 场景）。
  每个 Surface 拥有一个采样离屏目标（`Rgba8UnormSrgb`，预乘 alpha）。
- **View**：某个 Surface 的一种观察/输出；`presents_to_window = true` 的 View
  是窗口呈现源。
- **Portal**：父 View 到子 Surface 的挂载，决定呈现与输入映射；**不是**窗口。

同一 Surface 可被多个 Portal 引用：内容共享、模拟每 tick 只推进一次，各挂载
有独立呈现位置与命中路径。不同 Surface 实例即使 renderer/类型相同也不共享状态。

## 2. 身份与提交

- 身份复用 `core::id` 的代际 arena（index+generation+session），不引入第二套
  身份源；跨 session 或已释放（stale）的 id 一律拒绝。
- 组合以**批次**提交：先在副本上逐项校验（引用、挂载有限性/可逆性、环、深度
  预算），全部通过才整状态替换。失败的批次不改变图、不消耗 id 空间。
- 拒绝均具名：`CycleDetected{path}`、`DepthBudgetExceeded{depth,budget}`、
  `InvalidMount{detail}`、`InvalidIdentity{detail,token}`、`SurfaceInUse{token}`。
- GPU 资源失败同样具名且**先于提交**：目标尺寸超设备限额返回
  `InvalidSurface{detail}`（底层 `RendererError::TextureTooLarge`），图/身份
  空间不变；内容来自另一设备返回 `DeviceMismatch{expected,actual}`。
- **设备归属对所有内容 adapter 生效**：Iced/2D/3D 各自记录创建时的 device
  ordinal；无法证明归属的内容（raw device 句柄构造）不是静默放行，而是
  `DeviceUnverifiable{detail}`。同设备三种内容仍有合法正控。
- 消费顺序被破坏（脏子内容尚未在本帧渲染就被父采样）返回
  `DependencyOrder{detail}`，不静默合成旧纹理。
- 深度预算是**资源预算**而非产品上限：调用方可调 `max_depth`；正式用例覆盖
  两个三层方向与一次更深组合。

## 3. 调度与依赖

- 依赖边：父 View 的 Surface 依赖其挂载的子 Surface。
- `render_order()/tick_order()`：拓扑序（子先于父），每个 Surface 恰好出现一次
  （共享多挂载不重复内容穿透）。
- 脏传播：内容变更 → 该 Surface 与其消费者链置脏；无关兄弟保持干净。
- **内容适配器在本帧产生的输出变更同样传播**：真实 iced message/被捕获事件/
  重绘请求/布局变化、3D 内容指纹变化都会在**同一帧**把该 Surface 的全部消费者
  置为本帧需重采样（消费链传递闭包），不依赖调用者事后手工 `mark_content_dirty`。
  静态帧不会因此反复叠层（父面仍只在自身内容重绘后合成）。
- **结构性变更同样置脏**：`AddPortal/RemovePortal/SetMount/AddView/RemoveView/
  SetSurfaceSize` 失效实际受影响的 owner Surface 及其消费者链。卸载后父面必须
  重绘（旧 footprint 消失），resize 后新目标必须有内容。
- 合成只对**本帧重新渲染过内容**的父面执行：干净父面保留上一帧已合成的正确
  像素，不把同一半透明层再次叠加上去（跨帧稳定）。
- 不支持的反馈回路（含自环/间接环）在提交前拒绝，不做逐帧递归。

## 4. GPU 合同

| 项 | 约定 |
|---|---|
| 离屏目标 | `OffscreenTarget::new_sampled`：`RENDER_ATTACHMENT|COPY_SRC|TEXTURE_BINDING`；旧 `new`（非采样）语义保留，两者显式区分；`resize` 保持 usage |
| 尺寸/失败 | 创建前检查 `max_texture_dimension_2d`；超限 = 具名 `TextureTooLarge`，不触发 wgpu panic；`add_surface`/`resize_surface` 先备资源后提交，失败保 last-good |
| 颜色 | 内容纹理 sRGB 编码、**预乘 alpha**；采样得线性预乘；写 sRGB 目标由 wgpu 转码 |
| 混合 | 预乘 over：`src=One, dst=OneMinusSrcAlpha`（color 与 alpha 同）；负控可切到 opaque replace 证明门禁读的是真实混合 |
| 顺序 | 同一次提交内不采样正在写的纹理；跨提交由 DAG 顺序保证（子内容先提交），并有 `DependencyOrder` 运行时检查 |
| 2D 裁剪 | `PortalQuadPipeline` 用 `set_scissor_rect` 裁剪到挂载 clip；负控可关 scissor 证明溢出像素可观测 |
| 3D 平面 | `SurfacePlanePipeline`：透视相机（**该挂载的 host Surface** 的 view_proj）+ 轴对齐矩形；画家序（声明序=层次） |
| 设备 | 单一 Device/Queue（宿主 `GpuDevice`）；Iced/2D/3D 内容都在创建时记录 device ordinal，运行时入口逐 adapter 校验；第二设备内容被具名拒绝（`DeviceMismatch`），无法证明归属的 raw-device 内容被 `DeviceUnverifiable` 拒绝 |
| readback | 稳态 0；仅验收捕获/显式导出计数并报告；每帧 readback 负控可观测 |
| 生命周期计量 | `OffscreenTarget::build_seq` + runtime `target_creations/target_drops` 是真实目标生命周期计数：创建成功即计，resize 替换旧目标、被丢弃的失败准备、slot 删除都计入 drop；相同尺寸 resize 为 no-op，不虚增创建/释放。`live_targets()==live_target_slots()==live_surfaces()` 为无外部 pin 时的守恒断言 |
| 上传/内存计量（V-09） | 上传 = 真实 `queue.write_buffer` 记账：2D portal 每次 160B、3D 平面每次 128B（组合管线**各自独立**计数），另有内容适配器（2D quad / 3D mesh）计数；`SurfaceRuntime::{plane_uploads, portal_quad_uploads, content_uploads, upload_stats}` 分别暴露。验收 oracle 必须**分管线精确**（平面/quad 的改写次数与字节分别等于其 composite 数 × 128/160）、**总量严格等于三项之和**，并用 V-08 钩子证明“省略平面计数/省略内容汇总”会使对应门禁红；报告给采样点/单位/公式与排除项（iced 内部上传、纹理与 bind group 创建、驱动侧拷贝不计）。内存 = 进程工作集（OS 驻留，含驱动/iced/资产）与**运行时拥有的离屏目标字节**（`w*h*4`，按真实尺寸）分开报告；GPU/驱动驻留明确记为不可用及原因（wgpu 27 无可移植查询），不以进程 RSS 代替归属内存 |

## 5. 生命周期

- 关闭一个 View 会级联移除其 Portal，**不销毁**仍被引用的子 Surface 内容
  （GPU slot 与内容适配器保留）。
- 仍被挂载/观察的 Surface 不允许直接删除（`SurfaceInUse`）。
- **零尺寸**：组合图拒绝 `logical_size<=0`（`InvalidSurface`），不把“零尺寸
  暂停”当作模型已支持行为；窗口 target 的零尺寸暂停/恢复沿用 829 的
  `WindowTarget` 语义（`SuspendedZeroSize`，保留最后有效配置）。
- 被拒绝的结构变更保持最后可用图（`snapshot()` 不变）。
- 结构性变更后宿主必须 revalidate/cancel 输入路径；销毁内容不留下悬空焦点。

## 6. 非目标 / 未支持（本 profile）

曲面或任意网格 Portal、跨进程 GPU 共享、多窗口焦点、任意 3D 深度遮挡（当前
平面按声明序合成）、Web/Linux/移动平台、生产 AutoUI/RQHost/Forge 接线、
AutoShader/CAP 生成与热重载。

已知实现边界：iced 内容与 3D 场景当前每帧无条件重绘（2D canvas 才走 dirty
跳过），因此“本帧输出是否变化”由 adapter 事实（message/捕获事件/重绘请求/
布局/真实 caret、3D 内容指纹）判定并传播给消费者；不做逐像素比较、不做每帧
readback。快照/候选窗逐像素模板比对不在本版证据范围内。
