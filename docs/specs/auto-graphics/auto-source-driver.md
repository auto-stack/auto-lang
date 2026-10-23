# Auto source driver (PLAN-833)

> Current status (PLAN-834): I1's original real Auto Breakout + Squash denominator is delivered experimentally on autographics-dev. Earlier PLAN-829/830/833 pending statements below record their historical stage. I2 production AutoUI/RQHost migration, I3 Forge/distribution and Shader/CAP remain pending. Master carries knowledge only; graphics source does not enter master.

Status: experimental delivered on autographics-dev; independent R2 pass and frozen T-10 gates passed. Production migration remains pending.
Source workspace: `experimental/autographics-auto-driver`, bin `autographics-auto-driver`.
Frozen graphics upstream: `764e87559cff016d9e0d6b2141e5f1f9903398c5`.
Frozen SDK upstream: `9576b4dad5ceb8b201adedd46f9689f25c2dd5f9`.

## Actual execution and ownership

`auto-lang-core::create_vm_from_source` compiles one VM per instance. Public Parser AST signatures gate the root scalar API; public AutoTask/VirtualRAM values cross the boundary left-to-right. `AutoVM::call_fn_by_name_cpu_slice`/`resume_fn_by_name_cpu_slice` execute real functions and checked returned NanoValue tags. The high-level interpreter call remains a Nil stub and is unused. VM and task stay on their creation/event-loop thread; GPU/stock Iced/Kira stay host-owned. Scalar queries retain Auto-owned globals/lists; no stdout protocol, custom parser, SDK/VM changes or Rust gameplay substitution.

`Script` stages action → background/changed slots/events → `Publication` complete admission → `SceneConsumer` AGP Scene/DrawCmd/MeshInstance/TextOverlay → original SurfaceRuntime renderer. Every completed callback output is queued and consumed, including multi-tick display frames. Shared Portal mounts observe one instance; fixed tick IDs are accepted once per instance at 1/60. Two instances have separate state, queues, sessions and output identities. Actual CompositionInput surface/local/focus is consumed; text focus excludes game keys. Reset clears queues/maps and compiles a new session. Old session/sequence/generation output is rejected.

## Finite data contract

Protocol 1: `init/update/pointer/key/destroy`; int/float root signatures for `protocol/background/object_count/changed/object_alive/generation/shape/field/integer/event_count/event_kind/event_a/event_b`. Float means f32; unit is an actual completed callback. Object fields are translation xyz, positive scale xyz, RGBA. Shapes are the existing circle/quad/cylinder; no arbitrary resource/native pointer is exposed. Up to 128 slots, 256 output events, 64 queued actions and 64 completed outputs. Finite numeric components have magnitude at most 100000; RGBA and background 0..1. `changed=0` retains the last accepted slot; changes/creation/removal must be marked by Auto. Decreasing object count removes trailing slots. A destroyed slot needs a newer positive generation to return; changing a live generation is rejected. Resource keys hash actual procedural mesh bytes/material; GPU resources are reused while identity stays unchanged. Every quad and text resource is independently checked (either foreign direction and raw/unverifiable quads are refused). Text-overlay provenance is checked at surface admission and again before every frame; foreign adapters are rejected before GPU work, retaining the previous target.

Event triples represent Contact(collider,side), Sound(cumulative ordinal,0), BrickGone(brick,score). Per-callback transient buffers, contact/sound pairing, known prior destroyed identities, valid ranges/identities and exact monotonic sound sequence/count are checked before publication. Mixed good/bad output changes neither last-good scene, score, events nor audio. All output queries, including event collection, run through the same bounded pump; no partial output is published.

## Scheduling, failures and close

Per slice: 1000 steps, 2ms, clock polling every 64 instructions. One script pump resumes within an 8ms round. Cumulative callback limit: 100000 steps, 120 resumptions, or 250ms; first dispatch resets the public task counter, resume never does. Runnable is not completion. Compile/call/protocol/consumer errors remain distinct. Errors or budgets close task/VM, clear pending work and retain host last-good content. Parked means unsupported-wait and closes rather than polling. The Parked policy branch is covered with a constructed SDK outcome; async/IO script execution itself is excluded. Session error history is bounded (128 host errors; 4096 recent events). Closing clears function signatures/queues/maps; weak string/heap owner witnesses verify last-owner release and detect retained pins.

Trusted synchronous source only. Compilation is synchronous. No malicious native/FFI/IO security sandbox or blocking-system-call preemption is promised; no artificial syntax-depth exclusion was added.

## Delivered scope and oracles

A-01 real Auto creates/moves/colors/destroys 2D quads and an existing 3D cylinder. Source-color edits alter actual GPU pixels. A-02 layout, movement, collision order, destruction/score and events execute in Auto; unchanged 10800-tick golden, original 1e-3 tolerance and 210 contact details remain authoritative. Consumer-side replay checks actual AGP objects/events; speed/collision mutations must fail there as well, catching host Rust substitution. Original FiraMono-Medium/33px renderer and OGG/Kira consume outputs. Fixed normalization uses finite Auto f32 arithmetic, bit-equal to the original velocity; it does not claim general SDK double→float/native math repair.

Work assertions include real GPU pixels/3D hashes, wrong coordinates/duplicate delivery/dropped-output controls, shared single tick, native-window text-focus positive and bypass negative, actual Runnable fault with other surfaces still advancing, 20 resets and last-owner release. Input probes use existing single-shot posted OS click helpers and real winit key events. Source export is pinned and hash-checked (17 local packages/29 edges); no UI/Man/old facade in the SDK closure. The prepared consumer carries source/locks/assets, no target/rlib.

See workspace README, research decision and main Plan work receipts for exact commands, source/dependency identities and current measurements. No GPU timestamps are available. Independent final R2 and frozen full driver/AGP compatibility/window/resource gates passed; see the archived Plan for the bound evidence. I1 still requires two Auto games; Auto Squash, production AutoUI/RQHost, Forge/distribution, AutoShader/CAP and new ACE integration remain pending.

### Text-overlay upload accounting (PLAN-833 R1 repair)

Counters are recorded at the actual TextRenderer camera and glyph-instance write_buffer calls: camera 64B per rendered overlay, plus 48B per glyph when nonempty. Five glyphs therefore add two rewrites / 304B; an otherwise empty Game2d totals three / 368B. Empty glyphs still write the camera (one / 64B); a clean warm frame writes neither. Per-content, aggregate content and runtime totals include these counters once per child render, including shared Portal mounts. Creation/atlas uploads, Iced internal uploads and GPU timestamps retain the existing measurement exclusions.

Delivery provenance: [PLAN-833](../../plans/archive/833-autographics-auto-source-minimal-loop.md), reviewed code 77302d855; production boundaries above remain unchanged.


## PLAN-834 delivered explicit profile 2

The unchanged `Session` public SDK slice bridge is reused by `SquashScript`; profile 1 remains the original 60Hz protocol. `--sample squash` explicitly selects `profile() == 2`, never a guess based on payload shape. Source `samples/squash/{math,main}.at` owns initial gameplay state, movement/jump/gravity, seeded spawn decisions, two collision channels, score/death, birth ordering, scene bob/pitch/yaw and screen-exit decisions. The host supplies only explicitly requested raw xorshift64* values and the frozen camera matrix. A random candidate commits to the accepted host stream only with successful whole-packet publication; callback failure closes the modified VM and preserves last-good. No Rust SquashDriver is called by this mode.

`Packet3d` contains session/sequence/profile, explicit resource role, slot/generation/game identity, translation/yaw/pitch/roll/scale, score/dead and four-integer spawn/squash/despawn/death events. `Publication3d` independently runs at producer and actual scene consumer. It enforces monotonic spawn ordinals, tombstones, unique live slots/identities, at-most-one squash, event/state/order agreement and finite positive transforms. All fixed mesh keys, every GPU mesh/store/pipeline, HUD and overlay are checked before consumer publication; a rejected packet changes no accepted scene/HUD/audio. The renderer receives only matrices and fixed original resources, never an instruction to calculate gameplay. A bounded event log consumes every accepted packet, independently of display batch/mount count.

See [auto-3d-game-profile.md](auto-3d-game-profile.md) for the complete staged root API, resource and numerical domain, fixed 120Hz/clock/focus/retry behavior, limits and current work evidence. Mesh upload accounting now includes actual vertex/index initialization writes (not just per-frame pipeline writes). The original TextRenderer 64B + 48B/glyph contract is retained; atlas initialization, owned buffers/textures and decoded music bytes have separate actual-object measurements. Historical PLAN-833 upload totals describe that revision only.

Delivery provenance: [PLAN-834](../../plans/archive/834-autographics-auto-squash-and-dual-game-sdk.md), reviewed ebeab12d2888b9c9fb7e00e3c24479fb3c028011; [full merge receipt](../../plans/attachments/834/merge-r1/receipt.md). Frozen limits and original tolerances remain unchanged.

## PLAN-835 delivered scene authoring extension

Delivered experimental authoring entry `autographics-authoring` in the existing driver workspace. See [scene-document](scene-document.md) for finite schema/resources/local save/reopen and [authoring-session](authoring-session.md) for real stock-Iced commands, explicit Auto profile3, transactional preview/history/dirty, failures and tests. Profile1/2, original game goldens/GLB/audio and shared composition/input contracts retain their meaning. One same-device window, two fixed2D/3D views, bounded document/history, local same-volume single writer; no production UI/compiler/SDK expansion. I1 dual-game delivery remains; I2 AutoUI/RQHost and I3 Forge/production distribution remain pending. Final R2 and frozen driver/AGP/native/source gates passed; source lands on autographics-dev only.

Delivery: [PLAN-835](../../plans/archive/835-autographics-scene-authoring-save-and-undo.md), [review R2](../../plans/attachments/835/review-r2/review.md), [merge receipt](../../plans/attachments/835/merge-r1/receipt.md). Production I2/I3 remain pending.
