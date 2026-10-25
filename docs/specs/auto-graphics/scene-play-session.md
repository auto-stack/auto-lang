# Finite scene play session and real Auto behavior

Windows single-window stock Iced/wgpu authoring supports an explicitly finite
scene Run mode with one retained real Auto Session. Builtin6 uses samples/authoring_play/main.at. UserV1/profile7 adds one
finite user script via [behavior API](behavior-api.md), either accepted scene4
binding or an explicit temporary external source; see [behavior projects](behavior-project.md). Local Document and Registry come from accepted in-memory content, including
dirty/unsaved imports. Actor choice is a stable same-dimension leaf ID, one2D and
one3D; Builtin/temporary defaults choose the first eligible document leaf, absent dimension None; bound project roles remain explicit, including null. Empty worlds
are valid. UI names the chosen IDs and cycles eligible actors only in Editing.

Editing(no Play) → Starting → Running → Pausing/Paused → Stopping → Editing;
Faulted retains the last accepted runtime frame and closes the contaminated VM.
Window cancellation during Starting latches a pause request and reason, clears
input/clock and finishes initialization under the same30s budget. Only the complete
validated/prepared initialtick0 is published, entering Paused directly. It dispatches
no background ticks before explicit Resume. Starting Resume/Step stay rejected;
Stop and initialization faults still release the runtime VM. Later runtime/Step
transactions keep their original2s budget; cancellation does not extend it.
Run requires no pending edit/import/gesture/draft/unsaved-decision or unverified
save receipt. Failure before first runtime publication retains original editing
presentation. Stop closes VM/tasks without callbacks, clears input/clock/queued
widget events and restores retained accepted draws/instances/store and selection
without compile/import/resource prepare. New Run starts at initialtick0 with a
new positive nonce and new VM. Old commands are rejected by monotonic play command
tokens in addition to Editor epoch/revision. No queued busy command executes after
Stop. Close first Stops, then invokes the original accepted-content Save/Discard/
Cancel policy; cancel leaves the original editable content.

Profile6 gets full parent-first local graph, local visibility and resource mapping.
Reset(run,actor2,actor3), identity/visibility/bind_resource/append_field/derive build
the initial world. action(space,x,y,toggle) snapshots input at actual dispatch;
tick(next,dt) receives positive integer sequence and fixed f32 dt1/60. Auto owns
translation/visibility changes and world/effective recomputation.2D speed60 units/s,
3D speed3 units/s, independent axes without diagonal normalization:2D left/right
worldX, up/down worldY;3D left/right worldX, up/down minus/plus worldZ. V nonrepeat
edges toggle the focused actor; even edge parity before dispatch cancels.

Each complete transaction queries profile/run/tick/count and every node, including
hidden nodes: exact ordinal/kind/dimension/parent/resource, local/effective boolean,
local translation, returned world and all static fields. At128 nodes this is2820
typed scalar results. Integer identities remain integers. Only local translation
and local visible may vary; count/identity/parent/resources/kind/material/yaw/pitch/
scale remain fixed. Local Document validation checks original numeric/dimension/
forest limits; CPU hierarchy is only the rejection oracle for returned world and
effective. Renderer/bounds/presented snapshots consume the actual admitted outputs.
Any wrong type/nonce/sequence/resource/static field/numeric/world/effective rejects
the entire frame, keeps accepted tick/pixels and closes the VM. No clamping, skipped
node, proxy model, Rust behavior fallback or VM rollback/continuation is claimed.

One inflight transaction, at most four unissued tick credits, at most four dispatches
per event-loop round. All callbacks/queries share one8ms round target, using inherited
1000-step/2ms slices,64-instruction polling,100000 steps/120 resumes/250ms callback
budgets. Init30s includes compile followed by a deadline check/config/output/prepare;
complete runtime tick callback+queries+prepare has2s deadline. Synchronous compile,
GPU/native calls and OS scheduling are not hard-preemptible sandbox operations.
Display gap>250ms or credits>4 auto-Pauses with reason, discards wall-time debt/input,
and completes only the already-issued transaction before Paused (or Faulted).
Every completed tick is consumed; display batching never drops accepted output.
The positive i32 tick limit faults before another dispatch instead of wrapping.

Pause rejects new ticks, clears input/debt and waits for a complete inflight boundary.
Paused stays unchanged; Resume establishes a fresh wall-clock baseline. Single Step
only in idle Paused sends exactly one neutral-input actual Auto tick, staying Paused;
busy/replayed step tokens do not queue another. Histories/frames are not accumulated:
one private candidate and one last-good frame, bounded query queue and4096 scalar
timing/action summaries.60Hz is logic dt, not a real-time promise for arbitrary loads.

Acceptance: authoring_play, authoring_play_host, deadline/completion tests,
check_scene_play.py and check_play_owners.py, plus source/consumer/affected old cases.

Provenance: [PLAN-839](../../plans/archive/839-autographics-scene-play-mode-and-auto-runtime.md), [independent final R2](../../plans/attachments/839/review-r2/independent/review.md), [affected independent R3](../../plans/attachments/839/review-r3/independent/review.md), [merge receipt](../../plans/attachments/839/merge-r1/receipt.md). Source remains autographics-dev; production I2/I3 pending.

## UserV1/profile7 extension (PLAN-840)

Builtin6/profile6 and profiles1..5 retain their existing routes. Explicit UserV1/profile7 snapshots one local source, supplies the complete graph/resources/actors/neutral input, then calls app_start exactly once; app_update(dt float) is called once per dispatched tick. Both callbacks return Unit through all accepted expression paths; nested functions/closures keep their own return policy. User source never supplies main/reset/tick/output transport. Public graph/input queries and local translation/visibility writers are actual Auto functions. Derived reads observe earlier writes. Sticky first API error rejects the entire frame and closes the VM, keeping last-good. Source identity, source regions and immutable scene/assets/actors are independently checked. The original Starting cancellation, fixed clock, Pause/neutral Step/Resume, Stop restoration, whole-frame admission and ownership/budget limits remain unchanged. New Run creates fresh globals/VM; only temporary external sources reread files, while bound defaults use accepted bytes. See [exact behavior contract](behavior-api.md).

Provenance: [PLAN-840](../../plans/archive/840-autographics-public-behavior-api-and-user-scripts.md), [affected final review](../../plans/attachments/840/review-r2/review.md), [complete merge receipt](../../plans/attachments/840/merge-r1/receipt.md). Graphics source: autographics-dev. Production I2/I3 remain pending.

## Accepted project source (PLAN-845)

Scene4 behavior defaults to accepted immutable source+explicit nullable leaf roles, otherwise Builtin6. Run validates current binding and consumes its accepted bytes even after disk change/removal; errors never fall back to a different script. Temporary external/Builtin overrides preserve their existing routes without dirty/Save mutation. SourceSnapshot/Last Run/runtime callback diagnostics all show verified current-root blob provenance or explicit <memory:SHA256>.at for unconfirmed identities; geometry dirtiness does not make saved source unsaved. SaveAs/old-revision/pending provenance follows [behavior projects](behavior-project.md).

Original whole-frame validation, sticky API error, fixed dt/clock/credits/deadline, Starting cancellation, actual Auto dispatch, Stop restoration and VM/GPU ownership remain unchanged. No callback runs merely from bind/Open/Save; a runtime-Fault source can be saved and reopened before an explicit Run.

Provenance: [PLAN-845](../../plans/archive/845-autographics-bound-behavior-and-portable-projects.md), [independent final review](../../plans/attachments/845/review-r8-preparation/review.md), [merge receipt](../../plans/attachments/845/merge-r1/receipt.md). Source: autographics-dev; production I2/I3 pending.


## PLAN-846 finite spatial extension

Explicit UserSpatialV2/profile8 adds a bounded accepted geometry configuration before app_start. Its full base/descriptor identity is reconstructed from actual Host/Registry resources; malformed/foreign metadata cannot execute app_start or publish. Auto performs instant world queries and keeps original callback/whole-frame/CPU/Stop rules; geometry never becomes persisted runtime state.

Provenance: [PLAN-846](../../plans/archive/846-autographics-spatial-queries-and-trigger-interactions.md), [final affected review](../../plans/attachments/846/review-r6/review.md). SDK/root and graphics delivery identities are recorded in the Plan consolidation receipt; production I2/I3 remain pending.

## Finite segment extension (PLAN-848)

Explicit UserSegmentV3/profile9 receives full spatial configuration before callbacks. Every cast attempt and callback boundary invalidates its one result slot; errors retain whole-frame rejection/closed-VM/last-good/Stop rules. V3 alone specializes shared derive into parent-first dirty X/Y/Z/visibility passes in the existing arrays, preserving local+parent f32 order with a local parent cache per axis. move marks all axes, initialization/shape repair derives fully;11 result plus4dirty scalars total60B logical. Trusted numeric generated units use compact source lines under unchanged VM accounting. Native checks acknowledge accepted input and explicitly OS-resume clock pauses within the original phase deadline; production clock/focus/SingleStep policies are unchanged.

Provenance: [PLAN-848](../../plans/archive/848-autographics-segment-casts-and-nearest-hit-interactions.md), [independent affected final review](../../plans/attachments/848/review-r4-final/review.md), [frozen merge receipt](../../plans/attachments/848/merge-r2/receipt.md). Source: autographics-dev; production I2/I3 remain pending.
