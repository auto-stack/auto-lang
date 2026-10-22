# AutoGraphics 实验：Surface 组合与 GPU 合成（SD-01 草稿）

> 状态：**实验合同草稿**（PLAN-830 work 阶段产出，待 review/merge 后正式沉淀）。
> 适用范围：`experimental/autographics-prototype/` 独立 workspace，Windows x64、
> 单原生窗口、wgpu 27.0.1 / iced 0.14。**不表示生产 AutoUI/RQHost 已支持。**

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
- 深度预算是**资源预算**而非产品上限：调用方可调 `max_depth`；正式用例覆盖
  两个三层方向与一次更深组合。

## 3. 调度与依赖

- 依赖边：父 View 的 Surface 依赖其挂载的子 Surface。
- `render_order()/tick_order()`：拓扑序（子先于父），每个 Surface 恰好出现一次
  （共享多挂载不重复内容穿透）。
- 脏传播：内容变更 → 该 Surface 与其消费者链置脏；无关兄弟保持干净。
- 不支持的反馈回路（含自环/间接环）在提交前拒绝，不做逐帧递归。

## 4. GPU 合同

| 项 | 约定 |
|---|---|
| 离屏目标 | `OffscreenTarget::new_sampled`：`RENDER_ATTACHMENT|COPY_SRC|TEXTURE_BINDING`；旧 `new`（非采样）语义保留，两者显式区分；`resize` 保持 usage |
| 颜色 | 内容纹理 sRGB 编码、**预乘 alpha**；采样得线性预乘；写 sRGB 目标由 wgpu 转码 |
| 混合 | 预乘 over：`src=One, dst=OneMinusSrcAlpha`（color 与 alpha 同） |
| 顺序 | 同一次提交内不采样正在写的纹理；跨提交由 DAG 顺序保证（子内容先提交） |
| 2D 裁剪 | `PortalQuadPipeline` 用 `set_scissor_rect` 裁剪到挂载 clip |
| 3D 平面 | `SurfacePlanePipeline`：透视相机（父 Surface 的 view_proj）+ 轴对齐矩形；画家序（声明序=层次） |
| 设备 | 单一 Device/Queue（宿主 `GpuDevice`）；无第二设备；`GpuDevice::new` 在渲染路径构造数=0（结构性审计） |
| readback | 稳态 0；仅验收捕获/显式导出计数并报告 |

## 5. 生命周期

- 关闭一个 View 会级联移除其 Portal，**不销毁**仍被引用的子 Surface 内容。
- 仍被挂载/观察的 Surface 不允许直接删除（`SurfaceInUse`）。
- 零尺寸目标暂停（`ZeroSizeTarget`），恢复后按现行布局重新映射，不污染最后可用状态。
- 被拒绝的结构变更保持最后可用图（`snapshot()` 不变）。

## 6. 非目标 / 未支持（本 profile）

曲面或任意网格 Portal、跨进程 GPU 共享、多窗口焦点、任意深度遮挡（当前平面
按声明序合成）、Web/Linux/移动平台、生产 AutoUI/RQHost/Forge 接线、
AutoShader/CAP 生成与热重载。
