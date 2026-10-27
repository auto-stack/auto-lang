# auto-graphics — integration contract

> Current status (PLAN-834): I1's original real Auto Breakout + Squash denominator is delivered experimentally on autographics-dev. Earlier PLAN-829/830/833 pending statements below record their historical stage. I2 production AutoUI/RQHost migration, I3 Forge/distribution and Shader/CAP remain pending. Master carries knowledge only; graphics source does not enter master.

> Status: experimental（PLAN-829 交接合同；代码在 `autographics-dev` 分支，
> 见 [project.md](project.md)）。**两道合成检测门必须并列保留，不可互替**
> （R5-03/R18 裁定，见 IH-01）。

## Verified（PLAN-829，probe 证据）

**IH-01（独立探针级）**：iced 0.14 应用可在**同一** wgpu device/queue 上承载
真实控件 + 2D/3D 图形视图（iced_wgpu 交给 custom primitive prepare 的实例）。
合成=UI 之下不透明图形、按控件裁剪；resize/DPI/退出已验证。同设备证明是
功能+架构双重的：wgpu 27 在 API 层不拒绝跨 device 绑定（全局资源 id）——
视图路径**永不自建设备**（永久审计测试）。

合成验证有**两道独立门**（[counter 门 ≠ 内容门]）：

1. **Counter 门**（带内合成计数，BLIT_DRAWS 仅紧邻真实 composite draw
   递增）：捕获**漏画/跳过最终绘制**变异（blank-blit 永久反控）。
   早期 GDI 窗口截取环境受阻的记录即此门的来源背景。
2. **Content 门**（iced 0.14 `window::screenshot` 三时刻像素断言 +
   `transparent_blit` 永久反控）：捕获**最终合成全透明**变异——R4-05 实测
   该变异下 counter 门保持绿（2→104→111）而内容门三时刻全红；即
   **counter 不能检测透明片元**，内容门不可弱化回 counter-only。

当前正控（三时刻全绿）与完整透明负控（三时刻全拒识）在冻结提交
`ddbde3e62857` 上由 R18 复审与 T-11 merge 门禁双重复核。Iced 不持游戏
业务状态。Stock iced_widget 0.14.2 满足本 spike；生产 PLAN-043 patch
在此不需要（r1 F09，已记录）。

**IH-05（仓界）**：独立 workspace、crates.io stock iced_widget 0.14.2
（r1 F09 去 vendoring，无 provenance-tracked patch 残留）、sha256 冻结
资产、根 workspace 零附带改动（V-09 门）。

## Pending（不得读作已交付）

- IH-02 Auto SDK：G-base 运行时合同已有（身份/时间/动作/事件/回放），
  但无已发布 SDK/profile；`.at` 入口是 I1（pending）。
- IH-03 RQHost：无 GPU 协议；现行 rqhost 仍为 CPU/软件路径。
- IH-04 嵌套/Forge：仅单窗口合成；无 UI→3D→UI 矩阵、pick-through、
  IME、Forge 编辑闭环。

## I1–I3 gates

- I1 真实 Auto 源码游戏入口：需已发布 SDK 符号+收据；验收=双冻结游戏经
  公开 SDK 由 `.at` 驱动。
- I2 生产 AutoUI/嵌套/RQHost：需迁移完成收据（M814-B 及后继）；
  iced 探针不抵扣。**I2 的迁移完成/写面交接同时是 `autographics-dev`
  合回 master 的裁定前提（2026-10-22 用户裁定）。**
- I3 Forge 与发行：需 I2 + 编辑闭环设计收据。

后继计划必须引用本文件，且只能以绑定收据的证据替换 "pending"。

## PLAN-830 实验增量（组合 Surface/Portal 与嵌套输入）

> 代码落 `autographics-dev`（交付提交 `764e87559`）；证据
> `docs/plans/attachments/830/work-r6/merge/`。以下**只**扩充 IH-01/IH-04 的
> 实验事实，不改变 IH-02/03/05 与 I1–I3 的 pending 状态，也不表示生产接线完成。

**IH-01 增量（同设备/离屏内容采样）**

- 真实 iced 控件内容以离屏采样目标被 3D 平面与 2D 四边形消费；预乘 alpha
  over、线性/sRGB 一致；2D 用 scissor 裁剪、3D 平面用挂载 host Surface 相机；
  DAG 顺序（子先于父）由运行时检查（`DependencyOrder`）。
- 同设备归属对 Iced/2D/3D adapter 逐一记录并校验；外设备内容具名拒绝
  （`DeviceMismatch`），无法验证归属的 raw-device 内容 `DeviceUnverifiable`。
- 失败事务：目标超限/尺寸非法先分类后提交，保持 last-good；resize 替换/丢弃
  准备/slot 删除的真实目标生命周期计数守恒。
- 计量：两条组合管线与内容 adapter 的真实 `write_buffer` 计数（portal 160B/
  次、平面 128B/次）；验收 oracle 要求分管线精确、总量=分项之和；省略计数的
  变异必须令门禁红（V-08 钩子）。

**IH-04 增量（嵌套输入）**

- 双向嵌套均实测：UI→3D→UI 全链路点击/焦点/DPI2/resize；3D→UI→2D（冻结
  Breakout 暂停/reset/键盘焦点）；共享 Squash 内容两 Portal 同像素、单 tick、
  每挂载一次输入。
- 命中：2D clip/仿射反算、3D 射线+UV（按 host 相机）、递归非交互遮挡吞掉、
  捕获绑挂载路径、销毁/失焦/离窗取消。
- 焦点：唯一真实编辑目标；宿主合批与“取消先于 pending 渲染”均由 router 三态
  （`Unset/Target/Cancelled`）贯通到真实 widget 缓存；新合法点击/公开 focus
  可恢复。非法 Surface（foreign/stale/非 iced）的 focus 请求拒绝且无副作用。
- IME：真实 `text_input` 的 `InputMethod::Enabled.cursor` 作为 caret，沿活跃
  挂载链投影为 OS 候选锚点并随输入/选择/滚动更新；真实 OS 中文 IME 提交已取证。
  候选窗逐像素模板比对不在本版证据范围。

**仍 pending（不得读作已交付）**

- IH-02 Auto SDK 入口、IH-03 RQHost GPU 协议、IH-05 生产发行边界保持原状；
  I1 真实 Auto 源码入口、I2 生产 AutoUI/嵌套/RQHost（迁移完成收据）、I3 Forge
  发行仍 pending；AutoShader/CAP 未实现。
- 云/多窗口/跨平台、曲面或任意网格 Portal、逐平面深度遮挡、iced 内部上传与
  驱动驻留内存计量、组合管线 error scope：均未支持/未测，见 prototype-profile。
## PLAN-833 delivered experimental slice

Adds a finite real Auto SDK/AutoVM driver for 2D/3D object control and Auto-owned Breakout, with checked output/event admission and original Surface/Portal consumption. See [auto-source-driver.md](auto-source-driver.md). This is only I1's first slice: Auto Squash and the original two-game denominator remain pending. IH-03, production I2/RQHost, I3/Forge/distribution, AutoShader/CAP and the migration gate for merging graphics code into master remain pending. PLAN-830 composition/input facts above are retained.

Delivery provenance: [PLAN-833](../../plans/archive/833-autographics-auto-source-minimal-loop.md), reviewed code 77302d855; production boundaries above remain unchanged.


## PLAN-834 delivered I1 dual-game evidence

The delivered experimental slice supplies the original second denominator member: real Auto Squash through the same pinned SDK/Session consumer as Auto Breakout. The exact frozen games, numerical/event tolerances and image/audio resources remain the denominator. Work evidence includes unchanged Breakout 10800 ticks/210 contacts, Squash 3600 ticks/45 spawns/91 events/135 random draws, actual GLB/HUD/Retry images, actual Kira playback and native input/focus/retry controls. Source hashes and the independent source consumer bind the delivered slice; tests that mutate Auto physics/animation and real rendering/input/accounting paths must fail their corresponding formal oracles.

I1 is delivered for its original experimental denominator: both frozen games run from real Auto through the same published SDK symbols. Independent T-09 R1 and frozen T-10 full driver/AGP/native/source-consumer gates passed. This completes I1 only, not a production migration. IH-03/RQHost, production I2, I3/Forge/distribution, AutoShader/CAP and the requirement for a separate authorized production migration before merging graphics source into master remain unchanged. The delivered source targets autographics-dev only. The two original composition counter/content gates remain distinct.

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

## PLAN-838 current static asset extension

Keep profile1/2 games and profile3/4 sample contracts. Asset documents explicitly
use new samples/authoring_assets/main.at profile5. Auto accepts parent-first
hierarchy plus candidate-local resource ordinals, derives world/effective state,
and returns every node's kind/parent/dimension/ordinal and complete fields. Rust
checks exact count, identity, resource mapping and fields, then consumes the actual
returned resource/world/effective data for rendering/pick. Wrong resource output
is a whole-candidate rejection. GLB decode and GPU ownership remain in the host.
The same pinned SDK9576b4d and independent two workspaces/locks remain, with the
driver's direct locked gltf1.4 dependency and extras feature recorded in source.
Current source identity is plan838-identity.json; .agscene text hashes normalize
LF/CRLF, fixing fresh-checkout reproduction of inherited837 fixture bytes.
CLI adds --asset-check/--asset-file/--asset-save/--wrong-asset/--asset-gpu-fault
and retains --drop-release/--frames/--inspect. Native/report/aggregate completeness
and actual content controls remain mandatory. The slice provides portable finite
scene/static-model authoring. Production I2/I3, general materials/animation,
multi-platform/public engine/Forge distribution remain pending.

Provenance: [PLAN-838](../../plans/archive/838-autographics-glb-import-and-portable-scene-assets.md), [independent final R2](../../plans/attachments/838/review-r2/review.md), [merge receipt](../../plans/attachments/838/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.

## PLAN-839 current scene play extension

Profile6 is a finite driver-local scene behavior sample alongside unchanged old
game and authoring profiles. Exact CLI: --play-check, --play-asset <GLB>,
--asset-save <scene>, --report <path>; negatives --wrong-play-focus,
--drop-play-tick, --play-gpu-fault transparent|missing-part|wrong-asset, --frames1.
Full positive reaches27 phases and six independent endpoint/origin/all-part pixel
states, actual Auto step+1, UI/V parity/hidden/shown/focus loss, Stop original canvas
hashes, fresh Run identity, Save and relocated independent-process reopen. Negative
exit/report/aggregate completeness agree; partial/absent oracle is failure. Real GPU
faults must fail the same pixel oracle despite valid protocol/state/mesh counters.

Source closure is plan839-identity.json and verify_plan839.py --controls; package
consumer exports only verified inputs plus pinned680 SDK files, with its own target.
Inherited worker stack16MiB and native PE32MiB settings remain required for a fresh
consumer. Existing model upload counters distinguish immutable geometry writes from
legal per-instance/content/composition writes. Report bounded Run/Stop/pump/full-frame
CPU/compile durations and Windows working set; exclude GPU timestamps, Iced internal
buffers/atlas and driver residency. No performance/production maturity is inferred.

No arbitrary scripts, dynamic graph/resource edits, persistence of runtime, Apply
runtime result, general TRS/ECS/physics/audio, SDK/AGP changes, production AutoUI/RQHost,
Forge distribution, ACE/AutoShader/CAP, Linux or new platforms. Source integration
remains autographics-dev only; master receives knowledge after the actual closeout.

Provenance: [PLAN-839](../../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md), [independent final R2](../../plans/attachments/839/review-r2/independent/review.md), [affected independent R3](../../plans/attachments/839/review-r3/independent/review.md), [merge receipt](../../plans/attachments/839/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.

## Finite user behavior extension (PLAN-840)

The experiment additionally supports [autographics.behavior/1](behavior-api.md), explicit UserV1/profile7 beside unchanged Builtin6/profile6 and profiles1..5. One trusted local .at file contains app_start()/app_update(dt float), Core helpers/state/control flow and finite pure initialized immutable values. Public Auto graph/input reads and local translation/visibility writers replace user-written transport. User bytes are unchanged. Limits: user256KiB, managed64KiB, combined320KiB,128nodes; original callback/frame/time/queue/owner limits remain. Source diagnostics and run snapshots bind raw/API/bridge/combined versions and hashes. Move and Patrol are distinct portable scene/assets/source projects on one runner; user-only60→90 speed and relocation/differentcwd require no Rust rebuild. SDK pin932ad4877e5968f1aa7227d9dcd7fb69b4c69a21 is old9576b4d plus exactly four scanner files. Source closure299, SDK681/479packages/17locals/29edges and both original locks are bound by receipts.

The full inherited driver/AGP16case/dual-game/authoring/resource/hierarchy/Play/native/owner/counter AND actual-content denominators remain required, plus user13case and independent source-consumer gates. No I2/I3/general engine/public production SDK claim, script packaging/persistence, hot reload, FFI/import/async, rotation/scale writer or arbitrary VM heap hard quota is added.

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

## Finite project persistence boundary (PLAN-845)

A locally saved scene4 declares GLB assets and one accepted content-addressed Auto source, with explicit nullable stable actors. This finite binding/persistence supersedes PLAN-840's former script-packaging exclusion, without expanding behavior/1 or changing production integration. [Behavior projects](behavior-project.md) is the current contract for source admission/owners/closure/Run provenance. Root/source loss after acceptance never causes a fallback reread; later Open/Save/Verify still detect bad disk closure. Actual UI project relocation and own-SDK/target consumers complement, rather than replace, all previous games/AGP16/authoring/hierarchy/assets/Play/owner and counter AND content gates. I2/I3 and full engine/installable distribution remain pending.

Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.


## PLAN-846 finite spatial extension

PLAN-846 adds the explicit finite spatial profile beside v1 and a scoped upstream auto-val nonfinite fix/pin. Graphics code lands only on autographics-dev; the isolated SDK correction and separately authorized behavior-preserving MN repairs/verification policy land master after affected review and root functional gates. Original163 AGP/158 driver/83 native denominators remain, with20 driver tests and spatial native/owner/source cases appended. Production I2/I3/Schema/ACE remain later work.

Provenance: [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md), [final affected review](../../plans/attachments/846/review-r6/review.md). SDK/root and graphics delivery identities are recorded in the Plan consolidation receipt; production I2/I3 remain pending.

## AutoGraphics incremental compilation acceptance (PLAN-846:r3)

At this experimental stage, AutoGraphics requires correct incremental compilation: edits invalidate the applicable units, unchanged units may reuse admitted cached artifacts, and the resulting program must actually link and run with the expected behavior. Incremental compilation latency, throughput and P95 targets are observations rather than delivery gates. Performance optimization is deferred until functionality is complete, particularly integration with the real ACE engine; it requires later explicit acceptance criteria.

The functional cases, sample inputs, cache/identity/rejection/cleanup assertions and supported input scope remain required. Runtime callback limits and resource/deadline safety contracts remain in force. The root MN verification harness can select the explicitly named `AUTO_LANG_AUTOGRAPHICS_INCREMENTAL_PERF=observe` policy for AutoGraphics verification; it records the policy and measured values while leaving functional assertions active. Other root verification retains its default performance policy.

This acceptance policy does not claim that the experimental graphics driver has already been integrated with the real ACE engine; that integration remains future work. Separately authorized root SDK/MN repairs and verification-policy changes land on master; graphics implementation continues to land only on autographics-dev.

## Finite segment extension (PLAN-848)

PLAN-848 delivers only the finite experimental world-segment/AABB user query profile and its two portable Auto examples on autographics-dev. It preserves128nodes/32credits/originalCPU, explicit versions and original geometry/owner/source/OS-GPU evidence. It does not complete production AutoUI/RQHost I2, Forge/distribution I3, ACE/HMR, triangle/OBB picking or physics/sweep/camera unprojection. Master receives canonical knowledge only.

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.

## Swept-query extension (PLAN-849)

PLAN-849 extends only the independent experimental driver with explicit Auto swept-AABB/profile10 and script-controlled stop/clearance projects. Original I1 game/graphics denominator and source/resource/owner/pixel controls remain; production AutoUI/RQHost I2, Forge/distribution I3, default UI/Schema/ACE/HMR and dynamic physics remain pending. Source lands only autographics-dev; master carries canonical knowledge.

Provenance: [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md), [independent final review](../../plans/attachments/849/review-r1-final/review.md), [frozen merge receipt](../../plans/attachments/849/merge-r1/receipt.md). Graphics source: autographics-dev; production I2/I3 remain pending.

## Bounded-motion extension (PLAN-851)

PLAN-851 extends only the experimental source driver with v5 bounded Auto motion and portable motion projects. Existing I1 games/graphics, source/SDK/assets/owner/content and fresh-consumer closure remain. No production AutoUI/RQHost I2, Forge/distribution I3, Schema/ACE/HMR, material/animation or general physics completion is claimed. Source lands autographics-dev; master carries canonical knowledge.

Provenance: [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md), [independent final review](../../plans/attachments/851/review-r4/review.md), [frozen merge receipt](../../plans/attachments/851/merge-r4/receipt.md). Source stays on autographics-dev; production I2/I3 remain pending.

## Complete evidence and inherited denominator (PLAN-853)

Retain every one of the85 historical gate groups, their original per-binary selected identities and semantic oracles. Original producer/consumer280 IDs remain inclusion requirements; a larger aggregate count cannot replace missing cases. Driver paths run at the current frozen baseline. The original23 AGP slots may reuse evidence only when complete source/assets/tests/configuration/dependency/lock/input and oracle equivalence is proved; an unchanged AGP directory alone does not justify reuse of changed driver paths.

Freeze actual executable bytes before testing under a SHA-addressed path and retain them outside the disposable tree. Reports identify the actual running executable. Native/owner/content results bind that payload even after owner controls rebuild target. Missing or changed payload, wrong runner/report hash, incomplete/partial/interrupted invocation, unexpected skip or required0selected rejects the aggregate. Expected negatives must hit the original named oracle and restore a positive; counters and actual content remain separate required checks.

Every announced binary requires a complete top-level libtest frame. Each frame owns announced/completed identity sets and at most one pending case: replacement, unowned completion, over-count starts or a summary with pending state reject. Completion requires identity/count equality at both parent and child exits; a child never implicitly completes its parent and child IDs do not inflate the parent denominator. Completed empty Doc-tests remain recorded separately. Unframed results/failures reject.

Final independent code/acceptance/Spec review precedes merge-owned complete producer and fresh own-SDK/empty-target consumer suites, original native/owner/content gates and new measurement/evidence gates. The independent experimental contract does not trigger root Cargo tiers. M2/S0 baseline completion does not deliver new M2 capabilities, production AutoUI/RQHost I2 or Forge/distribution I3.

Provenance: [PLAN-853](../../plans/archive/853-autographics-m2-baselines-and-evidence.md), [independent final R3 review](../../plans/attachments/853/review-r3/review.md), [merge receipt](../../plans/attachments/853/merge-r1/receipt.md). Source: autographics-dev at c1961665d1a90a8705994085fd4344a1d6a9a564; SDK 7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467. This delivers M2/S0 research baselines; production I2/I3 remain pending.

## Finite 2D editor viewport integration (PLAN-856)

Current source closure is research/plan856-identity.json with 462 source hashes and the fixed 681-file SDK; tools/verify_plan856.py, verify_source.py and package_consumer.py select this current closure without rewriting historical identities. The permanent editor_viewport2d entry has 11 cases and the authoring binary adds one winit physical-pixel handoff regression. tools/check_editor_viewport2d.py requires the actual OS→winit→Host→Auto and final composite readback path, all 20 phases, and rejection of partial reports. Original named test IDs, game/native/owner/content controls and 23 AGP slots remain the denominator; the merge receipt records current two-sided results and any qualified original-AGP reuse. Auxiliary command/CPU and dynamic payload costs are separate from immutable mesh uploads; measurements do not imply GPU timestamps or guaranteed frame rate.

Source lands only on autographics-dev; master records canonical knowledge. No production AutoUI/RQHost I2, Forge/distribution I3, 3D editor navigation or runtime camera is implied.

Provenance: [PLAN-856](../../plans/archive/856-autographics-2d-viewport-grid-and-navigation.md), [independent final R2](../../plans/attachments/856/review-r2/review.md), [merge receipt](../../plans/attachments/856/merge-r1/receipt.md). Source: autographics-dev at `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Production I2/I3 and 3D editor navigation remain pending.

## PLAN-857 3D editor viewport integration

Current source closure is research/plan857-identity.json with 480 source hashes and the fixed 681-file SDK; tools/verify_plan857.py, verify_source.py and package_consumer.py select this current closure without rewriting historical identities. Runner provenance is bound by controlled builds: tools/build_runner_proof.py records clean-commit builds with Cargo event capture, and tools/runner_provenance.py enforces exact-lock receipts (`autographics-auto-driver/Cargo.lock` and `autographics-prototype/Cargo.lock` byte-checked against receipts, both source manifests and actual files; aliasing, duplication and foreign keys rejected), revalidates build commit, artifacts, SDK, RUSTFLAGS/RUST_MIN_STACK/RUSTC_WRAPPER, rustc and cargo, marks proof-less runners unverified instead of attributing them to whichever checkout is open, and allows cross-commit runner reuse only with explicit approval while changes stay limited to validation tools and the identity manifest and the true build commit is retained; configuration changes are rejected.

tools/check_editor_viewport3d.py requires the actual OS→winit→Host→Auto path and final composite readback across 25 phases with partial-report rejection; the permanent forced-depth-Always, stale-camera and missing-readback mutations fail the same gate and restore green. tools/check_default_3d_baseline.py and test_default_3d_baseline.py keep the 856 frozen, default and Home baselines pixel-equal with named comparator negative controls for fake locks, old-runner reattribution, unapproved reuse and configuration changes. The original denominators (dual games, composition/nested inputs, authoring/gesture/hierarchy/assets/Play, source bindings, query/sweep/motion, owner/counter and actual-content gates) remain required; the 23-slot AGP reuse audit does not apply where AGP changed — the 857 merge reran the full AGP/composition/native faces for the added render3d editor_grid and optional surface_runtime mount. The independent consumer runs its own export; because the provenance model attributes runners only inside a git checkout, consumer-side real-window coverage is carried by its full suite (including the real-window authoring_probe_exit family) plus byte-identical export identity, and the standalone 2D/3D window gates run provenance-bound from the producer checkout at the same commit.

Source lands only on autographics-dev; master records canonical knowledge. No production AutoUI/RQHost I2, Forge/distribution I3 or runtime game camera is implied.

Provenance: [PLAN-857](../../plans/archive/857-autographics-3d-viewport-navigation-and-grid.md), [independent final R4](../../plans/attachments/857/review-r4/review.md), [merge receipt](../../plans/attachments/857/merge-r1/receipt.md). Source: autographics-dev at `81b0974ac2512126790b0ef2af79235edf45b485`; SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`. Production I2/I3 remain pending.
