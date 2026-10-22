# auto-graphics

> **Status**: experimental
> 路径：`experimental/autographics-prototype/`（独立 Cargo workspace，自有 manifest/lock/资产；bin `graphics-probe` / `iced_spike`） | 技术栈：Rust + wgpu 27.0.1 + winit 0.30.13 + iced/iced_wgpu 0.14 + kira 0.9.6 + gltf 1.4.1 + fontdue
> **代码位置（2026-10-22 用户裁定）**：原型代码在 `autographics-dev` 集成分支（PLAN-829 落点 merge `ab56c32ec`，AGP 子树与复审冻结提交 `ddbde3e62857` 逐字节一致），**不合入 master**——待 814/815/816 系列（及后续）迁移工作全部完成后再议合回 master。master 侧仅沉淀本知识文档。后续 AutoGraphics 计划（830+）沿用同一集成分支承接模式（各自 plan 分支审毕后并入 `autographics-dev`）。

Module identity：实验性 wgpu 图形/游戏研究（AGP）。独立 workspace，不属生产 UI 栈，
不替代任何现行默认实现。

## Real entries（PLAN-829 证据）

- Probe runner `graphics-probe`（10 case：gpu-foundation / shared-runtime /
  ag2d-breakout / ag3d-risk-slice / ag3d-squash / lifecycle / metrics /
  verifier-negative-controls / iced-shared-device / independent-consumer）；
  独立交互宿主 `iced_spike`。
- 冻结样本：AG-2D-01 Bevy Breakout（`bevyengine/bevy@efe61cc5`）、
  AG-3D-01 Godot Squash the Creeps（`godot-demo-projects@b761b4cd`）——
  oracle 见 `research/sample-manifest.md`，golden trace 冻结于 `testdata/`。
- 机器 profile：Windows x64、键鼠；GPU/驱动/DPI 每次运行记录于报告
  （实测 RTX 4060 Ti / Vulkan / 驱动 617.14 / DPI 2.0）。

## Ownership & boundaries

- Owner 阶段：work（PLAN-829 T-01..09）→ review（T-10，R18 PASS）→
  merge（T-11，2026-10-22 落 `autographics-dev`）。
- 仓界：对 auto-lang/auto-ui/auto-man 私有源码零 path 依赖；
  iced_widget 0.14.2 **stock** 自 crates.io（r1 F09 去 vendoring——实测最小
  spike 无需生产 PLAN-043 patch，闭包内零 path 来源包）；无 junction/symlink；
  根 workspace/锁零改动（V-09 门）。
- 生产集成 PENDING（I1–I3），本卡不得读作生产能力；
  见 [integration-contract.md](integration-contract.md)。

## Non-goals（PLAN-829 不变）

Godot/Bevy 引擎兼容、第二渲染后端、生产 RQHost 接线、完整物理/动画编辑器、
Tilemap/骨骼/联网/XR。

## Proposed PLAN-833 consumer (review/merge pending)

Independent `experimental/autographics-auto-driver` consumes pinned pure SDK source and AGP's neutral scene/renderer interfaces. Real Auto owns the basic scene and frozen Breakout simulation; Rust owns rendering, fonts/audio and window routing. No production crate/root manifest/lock or SDK source is modified. Final review and frozen merge gates are pending. Profile and limitations: [auto-source-driver.md](auto-source-driver.md).
