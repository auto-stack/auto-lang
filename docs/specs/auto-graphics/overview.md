# AutoGraphics 模块现状

实验源码位于 autographics-dev，master仅保存知识。

- [project](project.md)：独立wgpu/Iced原型与机器profile。
- [surface-composition](surface-composition.md) / [nested-input](nested-input.md)：830组合、输入和资源合同，两道合成门并列保留。
- [auto-source-driver](auto-source-driver.md) / [auto-3d-game-profile](auto-3d-game-profile.md)：833/834同一公开SDK/Session驱动真实Auto Breakout与完整冻结Squash，原轨迹/数值/原音画/120Hz输入/准入/寿命/资源计量闭环；I1原双游戏分母已交付。
- [prototype-profile](prototype-profile.md) / [integration-contract](integration-contract.md)：Windows单窗有限实验profile；I2生产AutoUI/RQHost迁移、I3 Forge/发行、AutoShader/CAP仍pending，不允许把graphics源码合入master。
- [plans](plans.md)：已交付历史；[834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md)。

- [scene-document](scene-document.md) / [authoring-session](authoring-session.md): PLAN-835 independent stock-Iced/real-Auto scene authoring, save/undo/redo/reopen and failure recovery delivered experimentally; I2/I3 production migration unchanged.

- [Viewport interaction](viewport-interaction.md): PLAN-836 finite picking/translation with real Auto preview, single history publication and accepted-only persistence.

- [Scene hierarchy](scene-hierarchy.md): PLAN-837 finite same-dimension translation/visibility groups, v2 persistence and actual Auto profile4.

- [Scene assets](scene-assets.md): PLAN-838 bounded static GLB admission, actual Auto profile5, immutable SHA resources, schema3 portable closure and resource-aware save/recovery.

- [Scene play session](scene-play-session.md): PLAN-839 accepted scene Run/Pause/neutral Step/Stop with retained real Auto profile6, fixed tick/input isolation and original editing recovery; schema3 historical; scene4 bound sources are described below.

- [Finite user behavior API](behavior-api.md): explicit source selection, actual Auto callbacks and graph/input functions, portable Move/Patrol projects; original profiles and production boundaries retained (PLAN-840).

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

- [Bound behavior projects](behavior-project.md): scene4 stable nullable roles, byte-exact source/history and verified complete source/GLB closure; default Run and honest memory/hash provenance, real UI-output relocation (PLAN-845).

- [Finite spatial queries](spatial-query-profile.md): explicit behavior/2/profile8, actual all-mesh bounds and Auto point/pair queries, portable zone2d/zone3d triggers; unchanged v1 and production I2/I3 boundaries (PLAN-846).

- [Finite segment queries](segment-query-profile.md): explicit v3/profile9, deterministic nearest/token/weighted32credits, portable Beam2D/Sight3D; original128/CPU and production boundaries retained (PLAN-848).

- [Finite swept AABB queries](swept-aabb-profile.md): explicit v4/profile10, typed shared cast/sweep latest result, actual Auto fixed-volume translation and Dash2D/Dash3D stop/clearance projects; original limits/denominator and I2/I3 production boundary retained (PLAN-849).

- [Bounded move-and-slide](bounded-motion-profile.md): explicit v5/profile11, maximum two contacts and actual f32 candidate admission; portable Maze2D/Corridor3D, original budgets/SDK/resource denominator and I2/I3 boundary retained (PLAN-851).

- [Research quality baseline](research-quality-baseline.md): named CPU/resource boundaries, bounded default-off observation, five real Host workloads and comparable off/on sampling; complete source/consumer/test/runner evidence (PLAN-853, M2/S0).

Provenance: [PLAN-853](../../plans/archive/853-autographics-m2-baselines-and-evidence.md), [independent final R3 review](../../plans/attachments/853/review-r3/review.md), [merge receipt](../../plans/attachments/853/merge-r1/receipt.md). Source: autographics-dev at c1961665d1a90a8705994085fd4344a1d6a9a564; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. This delivers M2/S0 research baselines; production I2/I3 remain pending.

- [Checked behavior assembly](behavior-api.md): single exhaustive trusted source recipe, explicit marker/component rejection and byte-identical five-version source/consumer evidence (PLAN-854, M2/S0).

Provenance: [PLAN-854](../../plans/archive/854-autographics-behavior-source-assembly.md), [independent final R1 review](../../plans/attachments/854/review-r1/review.md), [merge receipt](../../plans/attachments/854/merge-r1/receipt.md). Source: autographics-dev at 523d27b863dbe306c89af0ea546334225a29f8e9; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. M2/S0 compatibility/maintenance work; new M2 capabilities and production I2/I3 remain pending.

- [Authoring responsibilities](authoring-session.md), [Play seam](scene-play-session.md), [viewport input](viewport-interaction.md) and [current entries](auto-source-driver.md): one Host with private commands/panel/play_adapter/viewport impls, unchanged owners/state/contracts, explicit CLI/probe boundary and complete455-source consumer evidence (PLAN-855). M2/S0 optimization preparation is complete; new M2 features remain pending.

Provenance: [PLAN-855](../../plans/archive/855-autographics-authoring-responsibility-split.md), [independent final R3 review](../../plans/attachments/855/review-r3/review.md), [merge receipt](../../plans/attachments/855/merge-r1/receipt.md). Source: autographics-dev at 05fdc81c546aff735eefdb6c39fbc5fb35dc546a; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. M2/S0 responsibility and entrypoint maintenance; new M2 capabilities and production I2/I3 remain pending.

- [M2 game sample research](m2-game-sample-research.md): two original small-level denominators, pinned reference/source-asset identities, public capability gaps and follow-on acceptance requirements (PLAN-858). Research only; games, new public services and distribution remain pending.

Provenance: [PLAN-858](../../plans/archive/858-autographics-m2-game-samples-and-api-gaps.md), [independent final review](../../plans/attachments/858/review-r1/review.md). Graphics baseline `05fdc81c546aff735eefdb6c39fbc5fb35dc546a`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`.

- [Finite 2D editor viewport](editor-viewports.md): editing-only navigation, bounded world-aligned grid and optional world-origin snap, with a shared presented transform for drawing, picking and real Auto gestures (PLAN-856).

Provenance: [PLAN-856](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md), [independent final R2](../../plans/attachments/856/review-r2/review.md), [merge receipt](../../plans/attachments/856/merge-r1/receipt.md). Source: autographics-dev at `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Production I2/I3 and 3D editor navigation remain pending.

- [Finite 3D editor viewport](editor-viewports.md): editing-only orbit/pan/zoom with focus/reset and perspective/orthographic toggle, camera-true picking and perspective axis dragging, and a bounded world-aligned depth-correct ground grid with metered auxiliary resources and controlled build provenance (PLAN-857).
- [Game runtime feasibility](game-runtime-feasibility.md): real-Auto G2/G3 character-motion and runtime-budget probes under the unchanged hard limits; two bounded candidates both satisfy the necessary semantics (decision go for the follow-up public character profile, PLAN-860). Probe-only evidence; no public API, OS input or game claim.

Provenance: [PLAN-857](../../plans/archive/857-autographics-3d-viewport-navigation-and-grid.md), [independent final R4](../../plans/attachments/857/review-r4/review.md), [merge receipt](../../plans/attachments/857/merge-r1/receipt.md). Source: autographics-dev at `81b0974ac2512126790b0ef2af79235edf45b485`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Runtime game cameras and production I2/I3 remain pending.

- [Game actions and single 2D character](game-actions-and-character2d.md): the public `autographics.character/1` profile — explicit OS action input through one router, a single bounded 2D character on frozen static solids with geometric final support, all inside the unchanged budgets (PLAN-861).

Provenance: [PLAN-861](../../plans/archive/861-autographics-public-actions-and-character-control.md), [independent final R2](../../plans/attachments/861/review-r2/review.md). Source: autographics-dev at `0f53f05da`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. 3D character, runtime camera framing, dynamic objects and production I2/I3 remain pending.

- [Gallery basic six (out-of-repo examples)](gallery-basic-six.md): the local `ag-examples` catalog contract — six logical IDs / ten subentries, fixed frozen engine dependency via engine-lock, public-CLI-only launches, self-contained per-example packages with offline relocation, and the measured 2D composition-order semantics (PLAN-877).
