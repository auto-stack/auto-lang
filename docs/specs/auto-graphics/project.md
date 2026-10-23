# auto-graphics

> Current status (PLAN-834): I1's original real Auto Breakout + Squash denominator is delivered experimentally on autographics-dev. Earlier PLAN-829/830/833 pending statements below record their historical stage. I2 production AutoUI/RQHost migration, I3 Forge/distribution and Shader/CAP remain pending. Master carries knowledge only; graphics source does not enter master.

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

## PLAN-833 Auto source consumer

Independent `experimental/autographics-auto-driver` consumes pinned pure SDK source and AGP's neutral scene/renderer interfaces. Real Auto owns the basic scene and frozen Breakout simulation; Rust owns rendering, fonts/audio and window routing. No production crate/root manifest/lock or SDK source is modified. Independent R2 and frozen merge gates passed; source lands only on autographics-dev. Profile and limitations: [auto-source-driver.md](auto-source-driver.md).

Delivery provenance: [PLAN-833](../../plans/archive/833-autographics-auto-source-minimal-loop.md), reviewed code 77302d855; production boundaries above remain unchanged.


## PLAN-834 delivered second real Auto game

`experimental/autographics-auto-driver --sample squash` adds the original frozen 3D game as a separate explicit profile on the same SDK/Session bridge, alongside unchanged Auto Breakout. Finite 3D data and whole-packet consumer admission expose original models, animation, HUD/music and 120Hz instance-owned input/lifetime; details are in [auto-3d-game-profile.md](auto-3d-game-profile.md). Implementation remains confined to the two experimental workspaces and canonical Specs. SDK/root production crates/manifests/locks are unchanged. Independent R1 and frozen full gates passed, closing I1 experimental dual-game evidence; production I2/I3 and migration remain pending.

Delivery provenance: [PLAN-834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md), reviewed ebeab12d2888b9c9fb7e00e3c24479fb3c028011; [full merge receipt](../../plans/attachments/834/merge-r1/receipt.md). Frozen limits and original tolerances remain unchanged.

## PLAN-835 delivered scene authoring extension

Delivered experimental authoring entry `autographics-authoring` in the existing driver workspace. See [scene-document](scene-document.md) for finite schema/resources/local save/reopen and [authoring-session](authoring-session.md) for real stock-Iced commands, explicit Auto profile3, transactional preview/history/dirty, failures and tests. Profile1/2, original game goldens/GLB/audio and shared composition/input contracts retain their meaning. One same-device window, two fixed2D/3D views, bounded document/history, local same-volume single writer; no production UI/compiler/SDK expansion. I1 dual-game delivery remains; I2 AutoUI/RQHost and I3 Forge/production distribution remain pending. Final R2 and frozen driver/AGP/native/source gates passed; source lands on autographics-dev only.

Delivery: [PLAN-835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md), [review R2](../../plans/attachments/835/review-r2/review.md), [merge receipt](../../plans/attachments/835/merge-r1/receipt.md). Production I2/I3 remain pending.

## PLAN-836 finite viewport profile

Extend the finite independent autographics-authoring experiment with this viewport profile. Document new --interaction-check/--interaction-probe and negative modes, source identity/consumer commands, resident7 meshes, actual CPU/working-set/upload measures and exclusions. Frozen SDK9576b4d/profile1/2/default driver/AGP/original assets/goldens/root workspace are unchanged. Final R2 approves this contract; frozen complete/native/counter+content/consumer gates and actual landing are recorded in the merge receipt. Source lands only autographics-dev; master receives knowledge after reviewed landing. I1 remains delivered and I2 production AutoUI/RQHost plus I3 Forge/production engine remain pending. Delivery is established only by that receipt.

SD-04 clarification after F-836-R1-01 (existing no-skip acceptance): self-check modes cannot succeed on incomplete event-loop return. They determine completeness and failure before report serialization, with phase/frame reason and nonzero exit on truncation/early close; true complete positive remains exit0/passed=true. Observational bounded windows remain a separate non-self-check mode. The native fixture prepares pointer/focus before input and preserves every real cancellation condition; this is not a support relaxation.

Measurements from the accepted work native run: CPU cold frame39386us, warm p50/p95 16473/19612us, synchronous compile26629us, working set380551168bytes, scale2.0. Candidate compile23904..25676us and pump745..803us (samples, not guarantees). Resident7 meshes/14 vertex-index writes136952bytes; repeated gestures preserve GPU Arc/store/target identity. Excludes Iced atlas/internal buffers, driver allocations and GPU timestamps. Source/consumer commands are in experimental/autographics-auto-driver/README.md: prepare_sdk.py --repository <pinned repo> --verify-only; verify_source.py; verify_plan836.py --controls; package_consumer.py <external-directory>.


Provenance: [PLAN-836](../../plans/archive/836-autographics-viewport-picking-and-translation.md), [final R2](../../plans/attachments/836/review-r2/review.md), [merge receipt](../../plans/attachments/836/merge-r1/receipt.md). Source integration is autographics-dev only; production I2/I3 remain pending.

## PLAN-837 current finite hierarchy profile

Document explicit profile4, current v2 finite forest and exact experimental CLI:
--hierarchy-check, --wrong-parent, --drop-release, --frames 1 incomplete, --inspect.
Self-check completion is resolved before serialization; incomplete/fault exits nonzero
and passed=false, full positive exits0/passed=true. New-process reconstruction stays independently checked.
Actual final-composition endpoint/origin pixels are compared to independent initial
leaf-color/background controls at temporary/move/undo/redo/hidden/show/saved states;
missing readback rejects. Native failure is serialized before nonzero exit and the
aggregate requires all seven state records. A test-only native copy injecting real
transparent endpoint or an old-position ghost must fail that same gate despite
valid Auto/state/commands, before paired reopen hashes can hide shared rendering
faults. These are finite2D fixture controls, not a whole-image/3D pixel-equivalence
claim. F-837-R1-01 corrected the earlier unimplemented visible-state oracle claim. Source identity is
plan837-identity.json; tools/verify_plan837.py --controls, check_hierarchy.py, check_hierarchy_pixel_faults.py and
package_consumer.py provide reproducible inputs. Fixed SDK9576b4d, locks, original
profile1/2 games, AGP/assets/goldens unchanged.7 resident meshes/14 initial writes
136952B, target/pipeline/mesh warm reuse; actual20 hierarchy rounds and retained-owner/
fresh-upload controls. CPU compile/frame/working-set measurements explicitly exclude
GPU timestamps, driver residency and Iced internal allocations. Source integration
only autographics-dev; master knowledge after reviewed landing. I1 remains delivered,
I2 production AutoUI/RQHost and I3 formal Forge/distribution remain pending. Finite
scene organization does not establish general TRS/Prefab/assets/game editing, public
stable engine APIs, production migration, Linux/Shader/CAP/ACE or full engine completion.
Final complete driver/AGP/native/composite/source/consumer gate results and actual T-10 closeout are recorded in the merge receipt.

Provenance: [PLAN-837](../../plans/archive/837-autographics-scene-hierarchy-and-group-authoring.md), [final R2](../../plans/attachments/837/review-r2/review.md), [merge receipt](../../plans/attachments/837/merge-r1/receipt.md). Source integration remains autographics-dev; production I2/I3 pending.
