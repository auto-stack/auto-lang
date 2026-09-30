# DP-2 探针：iced_wgpu 0.14 交给 `Pipeline::new` 的目标格式（T-01）

- 日期：2026-09-30
- 方法：iced 0.14 源码级静态追踪（`~/.cargo/registry/src/.../iced_{graphics,wgpu,winit}-0.14.0`）
  + 本仓 feature 面核对（`Cargo.toml` / `Cargo.lock`）。探针结论对"独立窗 surface"与
  "桌面 layer"两形态同源成立——两者最终都经 iced_wgpu 的 window compositor 建 Engine。

## 追踪链

1. 自定义 Primitive 管线的格式来源：`iced_wgpu-0.14.0/src/primitive.rs:134`
   `P::Pipeline::new(device, queue, format)`——`format` 即 `Engine.format`。
2. `Engine.format` 来源：`iced_wgpu-0.14.0/src/window/compositor.rs:99-118`
   ——`surface.get_capabilities(adapter).formats` 里按 `color::GAMMA_CORRECTION`
   过滤：true 取**第一个 sRGB 格式**，false 取第一个非 sRGB。
3. `GAMMA_CORRECTION` 取值：`iced_graphics-0.14.0/src/color.rs:26-31`——
   **未开 `web-colors` feature 时恒为 `true`**。本仓 iced 依赖声明
   （`crates/auto-lang/Cargo.toml:217`）features =
   `["tokio","image","svg","advanced","canvas","unconditional-rendering"]`，无 `web-colors`。
4. Windows（DX12/Vulkan）surface capability 列表首项 sRGB 即 `Bgra8UnormSrgb`。
   headless 路径（`iced_wgpu-0.14.0/src/lib.rs:953-965`）同为
   `Rgba8UnormSrgb`（GAMMA_CORRECTION=true 臂）。

## 结论（DP-2 定案）

**`Pipeline::new` 收到的是 sRGB 目标格式（窗口面 Bgra8UnormSrgb / 离屏 Rgba8UnormSrgb）。**

> **⚠ 实机修正（2026-09-30 T-07 走查推翻上句，静态追踪漏了 feature 统一）**：
> `iced 0.14.0` 的 **default features 自带 `web-colors`**（`iced-0.14.0/Cargo.toml`
> default 块），本仓未关 ⇒ `GAMMA_CORRECTION=false` ⇒ compositor 取**首个非 sRGB**
> 格式。实机日志（`auto run -r vm` 030 首帧）：`Available formats` 无匹配项问题——
> **`Selected format: Bgra8Unorm`（非 sRGB）**。本构建形态下目标**恒为非 sRGB**，
> iced 自身 chrome 以 sRGB 编码字节直写（pack=identity，GAMMA_CORRECTION=false 臂）。
>
> 因此 video shader 的正确输出域随目标格式分支：**非 sRGB 目标须在 shader 内完成
> sRGB 编码**（`present.rs` `fs_main_raw_target`，生产臂）；sRGB 目标写显示线性
> 交硬件编码（`fs_main`，防御臂）。静态追踪的"sRGB 配对断裂"排除结论**不变**
> （目标格式由 iced 统一交付、非本 widget 可选），但方向反转：不是"目标意外非
> Srgb 导致压暗"，而是 **web-colors 形态下 sRGB 纹理采样 + 非 sRGB 直通目标构成
> 双重解码**（decode∘decode ≈ γ4.4+）——这才是 E-3 偏暗偏暖的真机理，见
> dp3-color-source.md 的闭环复算。

推论（上屏链路的色彩语义）：
- 帧纹理 `Rgba8UnormSrgb`（`ui/mpv/channel.rs`）采样线性化 → `present.rs` shader
  归一（T-03）→ 按目标格式编码——端到端与 Chromium `<video>` 字节域对齐。
- 机制候选 1（sRGB 配对断裂）的原始表述被修正为"web-colors 形态的双重解码"；
  色偏源最终在 mpv 转换侧 + 编码臂缺失（→ DP-3）。

## 备注

- iced_winit 0.14 在每个事件/消息批次后请求 `RedrawRequest::NextFrame`
  （`lib.rs:1165-1168`，`unconditional-rendering`），配合应用 tick 形成连续重绘，
  重绘时 `RedrawRequested` 事件**传播给整棵 widget 树**（`lib.rs:814-835`）——
  这同时是 DP-1（wrapper 事件泵 update 臂每帧可达）成立的前提，一并记录。
- iced_winit RedrawRequested 处理循环在「每帧都 publish 消息」时会连重建视图
  至 3 次并告警（`lib.rs:843-850`）——事件泵只在真有契约事件时 publish 即避开。
