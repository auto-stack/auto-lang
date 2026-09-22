# PLAN-683 T-00 可行性 spike —— GO 决策档

> 日期：2026-09-22 ｜ 分支：`plan-683-dev`（worktree `.wt/lang-683/auto-lang`）
> 结论：**GO**——headless iced 宿主全链路（UserInterface 直驱 → 原语截获 →
> DrawList v1 降格 → codec → paint_ops 回放）在零 wgpu/零窗口形态下跑通。

## 验证的三件事（设计文档 §7）

1. **`UserInterface` + 段落引擎无 wgpu 可用** ✅
   `iced_runtime::user_interface::UserInterface` 公开 API 直驱（iced_test
   同一驱动方式）；文本度量走 iced_graphics 全局 font_system（cosmic-text
   CPU 整形，Inter 三字重进程内装载）。全链零 GPU 面——内存论点保住。

2. **001 首帧录制 → 现有 paint_ops 回放** ✅
   001-helloworld headless 首帧 = 1×TextStyled（"Hello, World!"，run 级
   提取）+ clear 深底 `#090E1A`（`semantic_rgb(Background)` 单源）；wire
   51 字节；v1 codec round-trip 恒等；`paint_ops` 经 `()` canvas 后端
   （debug 断言档 `geometry::Renderer` 空实现）完整重放降格路径零异常。
   测试：`p683_t00_helloworld_headless_first_frame` / `p683_t00_converter_
   headless_first_frame`（003 交互树首帧同绿；单测 <0.13s）。

3. **渐变/Path 原语记录形态** —— v1 降格策略已落（gradient=停点均值近似
   + 观测行；mesh/canvas=T-01 承接），v2 原生形态勘定随 T-01。

## 关键架构发现（对原设计的修正）

**原设计**：手写 `iced_core::Renderer` + 文本子 trait 全表面的 RecordRenderer。

**实勘障碍**：`view.into_iced()`（仓内 3.5 万行适配层）产出的组件树在
构造期钉死具体 `iced::Renderer` 类型——自研 Renderer 无法喂入，除非把
适配层整体 Renderer 泛型化（不可行量级）。

**实际路线（T-00 落地）**：`iced::Renderer` 在本仓 feature 集下 =
`iced_renderer::fallback::Renderer<wgpu, tiny_skia>` **公开枚举**——

```rust
let renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(font, size));
// 纯 CPU 构造；与 into_iced() 组件树类型同源，UserInterface::draw 直驱
```

tiny_skia 后端为延迟栅格化设计：draw 期原语仅记录进 `layers()`
（`quads: Vec<(Quad, Background)>` / `text: Vec<Item<Text>>`（Weak
paragraph→cosmic Buffer）/ `images` / 各层 `bounds`）——**其 Layer 即现成
的"记录原语 display list"**，本计划截获面 = Layer→wire 降格层（~300 行，
`desktop_protocol/headless.rs`）。官方 CPU 后端长期维护，截获语义与自研
RecordRenderer 等价且免 trait 表面实现/组件覆盖爬坡。

## 附带观测

- 层序/裁剪：tiny_skia 光栅期按层自身 bounds 裁剪——v1 降格同口径
  （每 clip 层发 Scissor，层 0 全域）。
- quad.border 有 v1 近似（inset 单像素描边）；shadow/gradient/mesh/
  image 为 v1 词汇面外，观测行留痕（T-01 v2 原生承接）。
- 依赖增量：`iced_runtime`/`iced_tiny_skia` 两可选直接边（同 0.14 版本
  系，随 `ui-iced` feature；先例 = iced_widget/iced_futures/iced_wgpu）。
- 组内 auto-down 依赖位已建（detached @ fba6563 = 主检出 master HEAD）。

## 风险与缓解（沿设计文档 §1-§2 移交）

| 风险 | 缓解 |
|---|---|
| tiny_skia Layer 面 iced 版本 churn | 降格层隔离在 headless.rs 单点；方案 3 重估触发条件不变 |
| 文本 daemon 侧重整形 vs App 整形漂移 | 同 cosmic + 同 Inter 字体（FontBlob 通道下发）；T-01 字形级对照 |
| v1 近似面（border/gradient/shadow/mesh/image） | T-01 DisplayList v2 原生承接；观测行台账在册 |
