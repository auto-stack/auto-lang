# auto-graphics — prototype profile

> Current status (PLAN-834): I1's original real Auto Breakout + Squash denominator is delivered experimentally on autographics-dev. Earlier PLAN-829/830/833 pending statements below record their historical stage. I2 production AutoUI/RQHost migration, I3 Forge/distribution and Shader/CAP remain pending. Master carries knowledge only; graphics source does not enter master.

> Status: experimental（PLAN-829 实测事实；代码在 `autographics-dev` 分支，
> 见 [project.md](project.md)）。以下为单机（RTX 4060 Ti / Vulkan / 驱动
> 617.14 / DPI 2.0）probe 实测值，不外推跨平台。

## Supported surface（已验证）

- Window（winit 0.30）+ 离屏 RGBA8-sRGB target，同一 wgpu 27.0.1
  device/queue；最小化暂停呈现；resize 仅重配 target。
- 2D：正交 quads/circles、alpha blend、fontdue 文本（Breakout HUD 33px、
  squash HUD 32px）。
- 3D：正交/透视相机（z 0..1）、深度、光照网格、2048² 方向阴影贴图、
  无纹理 GLB（gltf 1.4）节点变换烘焙、程序化环境。
- 音频：kira/cpal；一次性 SFX + 循环流。
- 固定步长时钟：1/60s（Breakout）、1/120s（squash 物理）；冻结动作流+
  种子确定性回放（与独立参考实现 bit-exact）。

## Resource/update/lifecycle 合同

- 对象/资源身份 session 域 + 代际；失效/复用 ID 拒绝；跨 session 迟到
  事件拒绝。
- 有界身份键缓存：未改资源零重上传（warm 后 created-counter 平坦；
  3D 实测 uploads/frame 0）。
- 局部编辑只触碰具名对象（touch-set + revision 追踪）。
- 失败分类：zero_size_target / missing_file / bad_shader / bad_model_parse /
  stale_identity / cache_full / resource_released / surface_lost；失败不
  覆盖 last-good 帧。
- 重启/关闭：双游戏各 20× 重启基线稳定（真实游戏资源在环：GLB mesh store
  建后平坦、循环音频跨 retry、按 session 缓存）；视图增删由 drop-witness
  派生（mem::forget 目标必红）；关闭中加载等待 abort 回调 ack；teardown
  事实在 device 销毁**之后**采样。运行时不自建 owned worker 线程
  （iced spike 输入回退的短命 helper 线程已记录）。

## Replay/measurement 方法

- Canonical 回放：冻结流+种子在 `testdata/`；golden trace 由独立参考实现
  生成；回放按冻结容差比对（位置 1e-3 / 1e-2；事件精确）。
- 计量：sim+encode+submit 的 CPU wall、P50/P95、cold/warm、双视图；
  **GPU 计时不可用**（无 timestamp query）——必须如实标注，不得编造。
  原始报告在 `scratch/829/`（worktree，gitignored）+ 主检出附件。
- 验证器：门断言于独立状态（事件重计、哈希、device token、session id）；
  skip 即失败（sabotage 演示在案）。

## Profile 差异（已知，未复刻）

手柄/右键输入、Bevy stepping 工具、gl_compatibility 分支、生产 AutoUI。
无跨平台或竞品性能结论。

## PLAN-830 扩展（组合 Surface/Portal 与嵌套输入，Windows 单窗实测）

> 代码同样在 `autographics-dev`（830 落点 `764e87559`），master 仅知识文档。
> 证据目录：`docs/plans/attachments/830/work-r6/merge/`（release 报告，fingerprint
> `764e87559cff`）。

**已验证（在 829 单窗 probe 之上新增）**

- 组合模型：Surface（有自身状态的 iced UI / 2D canvas / 3D 场景）+ View +
  Portal（父 View 到子 Surface 的挂载）。有限无环多层组合，两层方向均实测：
  UI→3D→UI（S-01）、3D→UI→2D（S-02）；S-03 为同一 Squash 内容两个 Portal。
- 离屏 UI 作为被采样内容：每 Surface 一个 `Rgba8UnormSrgb` 采样目标
  （`RENDER_ATTACHMENT|COPY_SRC|TEXTURE_BINDING`），预乘 over 合成；2D 挂载用
  scissor 裁剪、3D 平面用挂载 host Surface 的透视相机；DAG 顺序保证先产出后被采样。
- 嵌套输入：窗口→视图→挂载链（2D 仿射 / 3D 射线+UV）→子内容像素；捕获绑挂载
  路径；焦点为唯一真实编辑目标（含合批与取消三态语义）；真实 caret 的 IME 锚点
  随挂载链投影；真实 OS 中文 IME 组合/提交已取证。
- 单设备归属对 **三种内容 adapter**（Iced/2D/3D）逐一校验；无法证明归属的
  raw-device 内容 fail closed（`DeviceUnverifiable`）。
- 计量：2D portal 每次 composite 真实写 160B uniform、3D 平面 128B（两条组合
  管线独立计数，另计内容 adapter）；验收要求分管线精确、总量严格等于分项之和，
  并可用 V-08 钩子证明“省略计数”会被门禁识别。PortalQuad 的 160B 是真实
  `write_buffer` 传输长度（有效字段 24B + 对齐/未用区），上传计量按 160B，
  不按 24B 折算。
- 内存事实：owned 离屏目标 payload = Σ`w*h*4`（按真实尺寸，排除驱动对齐/开销）
  与进程工作集分开报告；GPU/驱动驻留**不可测**（wgpu 27 无可移植查询）。

**边界（本 profile 未支持/未外推）**

- 仅 Windows 单原生窗口、平面（Flat2d/Plane3d）挂载；无曲面/任意网格 Portal、
  无跨进程 GPU 共享、无多窗口焦点。
- 3D 平面按声明序画家式合成（无逐平面深度遮挡）。
- iced 内容与 3D 场景每帧重绘（dirty 跳过仅 2D canvas）；稳态零 GPU→CPU
  readback（仅验收捕获）。
- iced 内部上传与驱动侧内存不可测；GPU timestamp 不可用（`gpu_timing_available
  = false`），不编造。
- 候选窗逐像素模板比对不在本 profile 证据范围；Null 剪贴板；组合管线创建尚未
  安装 wgpu error scope（缺口具名保留）。
- 生产 AutoUI/RQHost/Forge 接线、AutoShader/CAP、I1–I3 仍 pending。
## PLAN-833 Auto source profile

Windows x64 single-window keyboard/mouse; original GPU/Iced/stock widget/font/audio resource versions. Two retained thread-confined SDK instances; two Portal mounts share the first instance and advance it once per fixed tick. Auto output/event/scalar limits, slice/round budgets and error/close behavior are explicit in [auto-source-driver.md](auto-source-driver.md). GPU pixels/3D hashes, source changes, golden replay, actual OS-input focus control and reset/owner witnesses supplement the inherited 829/830 evidence; independent R2 and T-10 closeout bind the delivered experimental slice. Main Plan receipts bind exact device, compile/pump timings, memory/upload/target/pipeline counts. GPU timestamps are unavailable. Full engine, Auto Squash, production RQHost/AutoUI and Shader/CAP stay pending.

PLAN-833 R1 repair: content-upload accounting includes TextRenderer camera/glyph writes (64B + 48B/glyph, empty glyphs omit instance write). Counts follow child renders, not mount count; five glyphs add 304B / two rewrites. All quad/text resources must independently match the runtime at admission and each frame before any allocation/frame/GPU side effect. See auto-source-driver.md and the current repair receipt for exact controls and revised measurements.

Delivery provenance: [PLAN-833](../../plans/archive/833-autographics-auto-source-minimal-loop.md), reviewed code 77302d855; production boundaries above remain unchanged.

Standalone Windows source consumers must explicitly retain the frozen 32MB main-thread stack profile; this is not inherited outside the repository. Build with `cargo rustc --locked --offline --bin autographics-auto-driver -- -C link-arg=/STACK:33554432`, then run the built executable. The default external stack overflowed; the same32MB profile passed isolated native replay without source changes.


## PLAN-834 Auto Squash profile

Windows x64, one native window, the same locked SDK/wgpu/Iced/Kira versions. The original player/mob GLBs retain all three meshes, baked child transforms/material colors, orthographic camera, directional light/shadow, ground/cylinders, Montserrat 32px Score/Retry and the single original OGG loop. Auto generates actor translation/yaw/pitch/bob and the complete frozen limited-physics gameplay. Input and fixed 120Hz actions belong to an instance, independent of its one/two mounts. The actual native probe exercises movement, one jump press, text-focus exclusion, both retry keys and repeat isolation; the wrong-focus control must fail the same oracle.

The inherited WindowTarget skips zero-size/minimized presentation. Simulation consumes fixed steps when the host is scheduled; long redraw gaps execute at most four due steps and discard wall-clock debt beyond32ms, rather than freezing the UI with an unbounded catch-up loop. Stock Pause prevents new steps; already accepted work may finish. Unmount revalidates the actual focus path and clears held keys/edges. A retained host instance is not destroyed by removing one view; explicit reset/close governs script ownership. No hard realtime guarantee is claimed under OS preemption or declared CPU-budget refusal.

Frozen work measurement: cold host1.120s; three 60-sample CPU submission wall P50/P95 cases2.940/3.690ms(shared mounts/two instances),4.398/5.198ms(single mounts/two instances),3.842/4.712ms(single mounts/one active instance). These cover consecutive different tick intervals and are not a causal speed comparison. Explicit selected owned payload57,065,320B includes both profile2 content buffers/depth/shadow/font atlases, all five offscreen targets9,109,600B and decoded music8,888,832B. Each instance uploads original mesh payload137,124B once. Process working set304,046,080→309,542,912B is separate. These receipts bind unchanged resource/measurement code and configuration; later implementation changes require new evidence; GPU timestamps/driver residency, Iced internals, swapchain and allocator overhead are not measured. See the Plan and standalone source/stack profile above.

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


## PLAN-846 finite spatial extension

Explicit behavior/2/profile8 provides finite all-mesh AABB proxies and two portable trigger examples, as defined in spatial-query-profile.md. V1/Builtin6/profiles1..5, old games and picking remain. This is not precise physics, production widgets/Schema/ACE/Forge or a full engine claim.

Provenance: [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md), [final affected review](../../plans/attachments/846/review-r6/review.md). SDK/root and graphics delivery identities are recorded in the Plan consolidation receipt; production I2/I3 remain pending.

## Finite segment extension (PLAN-848)

The finite authoring profile includes explicit behavior/3/profile9 segment queries and Beam2D/Sight3D alongside unchanged old profiles. AABB proxies are not exact model picking, physics, sweep or camera unprojection. All original128-node/32-credit/CPU/owner/source/input limits and production I2/I3 exclusions remain; see [segment profile](segment-query-profile.md).

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.

## Swept-query extension (PLAN-849)

The finite experiment now includes behavior/4/profile10 swept fixed world-axis AABB queries and Dash2D/Dash3D. Query does not move nodes; clearance/stop feedback belongs to user Auto, not implicit engine skin. Original limits/resources/older profiles remain. No rotational/OBB/dynamic collision, slide/penetration solver or production migration is claimed.

Provenance: [PLAN-849](../../plans/archive/849-autographics-swept-aabb-and-stop-before-impact.md), [independent final review](../../plans/attachments/849/review-r1-final/review.md), [frozen merge receipt](../../plans/attachments/849/merge-r1/receipt.md). Graphics source: autographics-dev; production I2/I3 remain pending.

## Bounded-motion extension (PLAN-851)

The finite experiment adds explicit v5/profile11 and [bounded move-and-slide](bounded-motion-profile.md): maximum two contacts, explicit user clearance, fixed world-axis AABB and distinct InitialOverlap/PrecisionStop. Maze2D/Corridor3D remain under original128node/CPU/managed/GPU boundaries. This does not deliver gravity, slopes, grounding, jumping, dynamic objects, general physics or production migration.

Provenance: [PLAN-851](../../plans/archive/851-autographics-bounded-move-and-slide.md), [independent final review](../../plans/attachments/851/review-r4/review.md), [frozen merge receipt](../../plans/attachments/851/merge-r4/receipt.md). Source stays on autographics-dev; production I2/I3 remain pending.
