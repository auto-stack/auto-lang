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
